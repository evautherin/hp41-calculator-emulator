---
phase: 56-app-lifecycle-clock
reviewed: 2026-06-03T00:00:00Z
depth: standard
files_reviewed: 2
files_reviewed_list:
  - hp41-gui/src-tauri/tauri.ios.conf.json
  - hp41-gui/src/App.tsx
findings:
  critical: 1
  warning: 1
  info: 2
  total: 4
status: issues_found
---

# Phase 56: Code Review Report

**Reviewed:** 2026-06-03
**Depth:** standard
**Files Reviewed:** 2
**Status:** issues_found

## Summary

Reviewed the two phase-56 deltas: the new `tauri.ios.conf.json` iOS config override
and the App.tsx changes (`needsTickRef` ref-sync, the `visible` branch in the empty-deps
visibilitychange listener, and the touch-path stopwatch interception block).

The JSON override is correct — it repeats all six base window fields
(`title`/`width`/`height`/`resizable`/`decorations`/`visible`) plus `backgroundThrottling`,
so the JSON Merge Patch array-replacement preserves the full base window object. Valid JSON.

The touch stopwatch block correctly mirrors the physical `handleKey` block: it sits above
the modal (`pendingInput`) and modal-opener routing so an active stopwatch mode is never
discarded, guards with `busyRef`, consumes shift in `finally`, and uses `xeq_`-prefixed ids.
The "any other tap → `sw_exit`" divergence from the physical path (which no-ops unmapped keys)
is intentional and documented.

The `needsTickRef` ref-sync pattern itself is correct, and the empty-deps visibilitychange
listener correctly keeps empty deps and uses `tick_time` (not `get_state`) in the resume branch.

However, the resume branch gates on `isIos` read from a **stale closure**, which silently
disables the entire LIFE-02 foreground-tick feature on iOS — the one platform it targets.

## Critical Issues

### CR-01: `visible` resume branch reads stale `isIos` closure — LIFE-02 tick never fires on iOS

**File:** `hp41-gui/src/App.tsx:1063` (effect at 1056-1074; `isIos` declared at 288, set at 533-535)
**Issue:**
The visibilitychange `useEffect` has empty deps (`[]`, line 1074) so it registers its
handler exactly once on mount and never re-runs. Inside it, the resume branch reads `isIos`
directly from the captured closure:

```ts
} else if (document.visibilityState === 'visible' && isIos && needsTickRef.current) {
```

`isIos` is `useState(false)` (line 288), and is set to its real value **asynchronously after
mount** via `invoke<boolean>('is_ios').then(setIsIos)` (lines 533-535). When `setIsIos(true)`
resolves it triggers a re-render, but the empty-deps effect does **not** re-run, so the
registered closure permanently holds `isIos === false`. The condition `isIos && ...` therefore
always evaluates to `false`, and the gated `tick_time` on foreground return **never fires on iOS**.

The inline comment ("isIos (closure — set once on mount, stable)" at line 1053, and line 1055
"Desktop/macOS: isIos is false → the 'visible' branch never fires spurious IPC") is factually
wrong about `isIos` being "set once on mount" — it is `false` at mount and updated later, so the
listener's view of it is stuck at the mount-time `false` on every platform including iOS.

Net effect: the LIFE-02 feature (correct clock/stopwatch display within one frame on foreground
return) is dead code on its only target platform. This is the same stale-closure trap that the
adjacent `needsTickRef` was introduced to avoid — but it was applied to `needsTick` and not to `isIos`.

**Fix:** Mirror the established ref-sync pattern for `isIos` (or fold it into the existing
`is_ios` effect), and read the ref inside the listener:

```ts
const isIosRef = useRef(false);
// in the existing is_ios effect (lines 533-535):
useEffect(() => {
  invoke<boolean>('is_ios')
    .then(v => { setIsIos(v); isIosRef.current = v; })
    .catch(() => { setIsIos(false); isIosRef.current = false; });
}, []);

// in the visibilitychange listener (line 1063):
} else if (document.visibilityState === 'visible' && isIosRef.current && needsTickRef.current) {
```

Update the comments at lines 1053-1055 and 1074 to state the listener reads `isIosRef.current`
(live) rather than a "stable closure".

## Warnings

### WR-01: Resume `tick_time` branch omits `setErrorMessage(null)`, leaving stale error banners

**File:** `hp41-gui/src/App.tsx:1067-1069`
**Issue:**
Every other success path that applies a `tick_time` / `dispatch_op` view clears the error
banner: the live-tick interval (`setCalcState(view); setErrorMessage(null);`, line 498) and the
touch stopwatch block (`setCalcState(view); setErrorMessage(null);`, lines 698-699) both do so.
The new resume branch only does `setCalcState(view)`:

```ts
invoke<CalcStateView>('tick_time')
  .then(view => { setCalcState(view); })
  .catch((err: unknown) => showToast(extractErrMessage(err)));
```

If the app was backgrounded while an error banner was displayed, the foreground-return tick
refreshes the calc view but leaves a stale `errorMessage` rendered on top of now-current state —
an inconsistency the sibling success paths explicitly avoid. (Lower-impact than CR-01 because it
is currently unreachable while CR-01 keeps the branch dead, but it is wrong on its own merits and
will surface the moment CR-01 is fixed.)
**Fix:** Match the sibling paths: `.then(view => { setCalcState(view); setErrorMessage(null); })`.

## Info

### IN-01: Touch stopwatch comment mislabels `xeq_STPW` as "split" with ambiguous wording

**File:** `hp41-gui/src/App.tsx:677` and `690-691`
**Issue:** The block comment says "ENTER→STPW split". Per `hp41-core/src/ops/time/stopwatch.rs`,
`STPW` records the current elapsed time as the split point and `SWPT` recalls it — so `enter →
xeq_STPW` is functionally the canonical split/lap record, which is correct. The wording is fine,
but note the **physical** block's adjacent comment (line 936, pre-existing, out of phase-56 scope)
mislabels `xeq_STPW` as "reset", which can mislead a future reader comparing the two paths.
**Fix:** Optionally align the comment wording across both blocks (e.g. "STPW = record split point")
to avoid the physical-vs-touch terminology mismatch. Non-blocking.

### IN-02: Comment block at lines 1053-1055 documents incorrect runtime behavior

**File:** `hp41-gui/src/App.tsx:1053-1055`
**Issue:** Independent of the CR-01 code fix, the comment asserts `isIos` is "set once on mount,
stable" and that on Desktop/macOS "isIos is false → the 'visible' branch never fires spurious IPC".
The first claim is false (`isIos` is set asynchronously, not at mount); the second is accidentally
true only because the stale closure pins it to `false` everywhere. Leaving this comment as-is would
mislead the next maintainer into believing the closure-read is safe.
**Fix:** Rewrite the comment to reflect the ref-based fix from CR-01 once applied.

---

_Reviewed: 2026-06-03_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
