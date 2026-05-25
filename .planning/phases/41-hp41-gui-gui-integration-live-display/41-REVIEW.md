---
phase: 41-hp41-gui-gui-integration-live-display
reviewed: 2026-05-25T12:00:00Z
depth: standard
files_reviewed: 13
files_reviewed_list:
  - hp41-core/src/ops/program.rs
  - hp41-core/tests/op_catalog_xrom.rs
  - hp41-gui/src-tauri/src/commands.rs
  - hp41-gui/src-tauri/src/types.rs
  - hp41-gui/src-tauri/src/lib.rs
  - hp41-gui/src-tauri/src/prgm_display.rs
  - hp41-gui/src-tauri/permissions/tick-time.toml
  - hp41-gui/src-tauri/capabilities/default.json
  - hp41-gui/src/help_data.ts
  - hp41-gui/src/HelpOverlay.tsx
  - hp41-gui/src/HelpOverlay.test.tsx
  - hp41-gui/src/App.tsx
  - hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt_time.rs
findings:
  critical: 2
  warning: 3
  info: 2
  total: 7
status: issues_found
---

# Phase 41: Code Review Report

**Reviewed:** 2026-05-25
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

Phase 41 wires 35 Time Pac ops into the Tauri GUI, adds live clock/stopwatch display via a new `tick_time` Tauri command and a 100ms `setInterval`, and extends the help overlay and CATALOG 2 to cover the Time Module. The mechanical parts (prgm_display arms, help overlay, Vite JSON import, Tauri permission, capability entry) are correct and follow established Phase 36/39 patterns precisely. The CATALOG 2 generic loop refactor is a clean improvement.

Two correctness bugs exist: (1) the `setInterval` cleanup on every `calcState` change is structurally incorrect for React's effect model — it creates and immediately destroys the interval on every state update triggered by a tick, effectively defeating the live-display entirely after the first tick; (2) the `alarm:interrupting:deferred` event prefix from `hp41-core` falls into the generic `showToast` branch, surfacing the raw event string to the user. Both are regressions from the published D-41 decisions. Three warnings and two info items round out the findings.

## Critical Issues

### CR-01: setInterval is created and immediately cleared on every tick update

**File:** `hp41-gui/src/App.tsx:270-291`

**Issue:** The `useEffect` managing the live-display interval has `calcState` in its dependency array (line 291). Every time `tick_time` resolves and calls `setCalcState(view)`, React re-runs this effect. The cleanup function returned at line 285 fires BEFORE the new effect body runs. Because `liveTickRef.current` is non-null at cleanup time, the cleanup clears the interval and sets `liveTickRef.current = null` (lines 286-289). On the next effect body run, `liveTickRef.current === null` and `needsTick` is still true, so a new interval is created. This means the effective tick rate is limited to one IPC call per `calcState` change — not 100ms steady-state. During the gap between the old interval clearing and the new one starting (one JS microtask), no interval is running at all.

For stopwatch display requiring ≥10 Hz (TIME-SW-08), the interval being recreated on every state update is effectively a stuttering single-shot, not a 100ms periodic. The live display will update at whatever rate the React render cycle completes rather than truly polling at 100ms.

**Fix:** Move the interval management to a separate `useEffect` that depends only on the boolean condition, not the full `calcState` object. Derive the condition into a stable boolean via `useMemo` or a separate state:

```tsx
// Derive the stable boolean outside the effect.
const needsTick = !!(calcState?.clock_active || calcState?.stopwatch_keyboard_mode);

// Effect: start/stop the interval based only on the boolean.
// Does NOT depend on `calcState` directly — avoids recreation on every tick.
useEffect(() => {
  if (!needsTick) {
    if (liveTickRef.current !== null) {
      clearInterval(liveTickRef.current);
      liveTickRef.current = null;
    }
    return;
  }
  if (liveTickRef.current === null) {
    liveTickRef.current = setInterval(() => {
      if (busyRef.current) return;
      invoke<CalcStateView>('tick_time')
        .then(view => { setCalcState(view); setErrorMessage(null); })
        .catch(err => showToast(extractErrMessage(err)));
    }, 100);
  }
  return () => {
    if (liveTickRef.current !== null) {
      clearInterval(liveTickRef.current);
      liveTickRef.current = null;
    }
  };
}, [needsTick, showToast]); // NOT calcState — stable deps only
```

### CR-02: `alarm:interrupting:deferred` surfaced as raw event string in toast

**File:** `hp41-gui/src/App.tsx:647-664`

**Issue:** `hp41-core/src/ops/time/alarm.rs:dispatch_alarm_event` pushes three distinct event prefix forms into `event_buffer`:
- `"alarm:message:{text}"`
- `"alarm:xeq:{label}"`
- `"alarm:interrupting:deferred"` (D-38.4)

App.tsx at line 653 only recognises the first two. The third prefix (`alarm:interrupting:deferred`) falls through to the `else` branch at line 659 and is passed verbatim to `showToast`. The user sees the raw internal event string `"alarm:interrupting:deferred"` as a toast, not a user-facing message.

Additionally, the `alarm:xeq` branch at line 655 dispatches `dispatch_op` but does not set `busyRef.current = true` before the invoke, and does not reset it after. This means a control alarm XEQ fires outside the busyRef guard and can race with an in-flight `dispatch_op` from a simultaneous keypress.

**Fix:**

```tsx
for (const line of calcState.event_buffer) {
  if (line.startsWith('alarm:message:')) {
    showToast(line.slice('alarm:message:'.length));
  } else if (line.startsWith('alarm:xeq:')) {
    const label = line.slice('alarm:xeq:'.length);
    if (!busyRef.current) {        // guard against concurrent dispatch
      busyRef.current = true;
      invoke<CalcStateView>('dispatch_op', { keyId: `xeq_${label}` })
        .then(view => setCalcState(view))
        .catch(err => showToast(extractErrMessage(err)))
        .finally(() => { busyRef.current = false; });
    }
  } else if (line === 'alarm:interrupting:deferred') {
    // Interrupting control alarm deferred per D-38.4 — no-op in GUI;
    // the alarm remains past_due and will re-fire on next check.
    // Optionally surface a brief toast for debug/polish:
    // showToast('Alarm (interrupting): acknowledged');
  } else {
    // BEEP, TONE, PAUSE, or other non-alarm events.
    showToast(line);
  }
}
```

## Warnings

### WR-01: Stale `calcState` closure in `setInterval` callback

**File:** `hp41-gui/src/App.tsx:274-279`

**Issue:** The `setInterval` callback at line 274 captures `showToast` from the closure at effect-creation time. `showToast` is stable (wrapped in `useCallback` with no deps), so this is fine. However, the callback calls `invoke<CalcStateView>('tick_time').then(view => { setCalcState(view); ... })` — the `setCalcState` setter is stable too. This part is correct.

The separate concern: if `needsTick` flips false while a `tick_time` IPC call is in flight (the user exits clock mode mid-tick), the `.then(view => setCalcState(view))` still fires after the interval was cleared, re-setting `calcState` to the tick response. This sets `clock_active = false` in the view (correct behavior since the core cleared it), but it also means one extra state update after the user explicitly left clock mode. This is a cosmetic race, not data loss, but can produce a visible flicker on the display transition.

**Fix:** Introduce a cancelled flag inside the effect or check `liveTickRef.current !== null` before applying the result:

```tsx
liveTickRef.current = setInterval(() => {
  if (busyRef.current) return;
  invoke<CalcStateView>('tick_time')
    .then(view => {
      if (liveTickRef.current !== null) { // only apply if interval still active
        setCalcState(view);
        setErrorMessage(null);
      }
    })
    .catch(err => showToast(extractErrMessage(err)));
}, 100);
```

### WR-02: `allEntries` memo never re-computes (empty dep array)

**File:** `hp41-gui/src/HelpOverlay.tsx:97-99`

**Issue:** The `allEntries` memo at line 97 has an empty dependency array (`[]`). `helpEntriesAll()` returns a value derived from Vite static imports — module-level constants that never change at runtime. So the behavior is correct today. However, the empty dep array silences the exhaustive-deps lint rule (if enabled) and makes future reviewers believe `helpEntriesAll()` is a side-effect-free constant, when it is in fact a function call. If `helpEntriesAll()` were ever changed to return non-static data, the stale closure would silently return the first evaluation forever.

**Fix:** Either inline the constant outside the component (since it truly never changes):

```tsx
// Outside the component — evaluated once at module load, never stale.
const ALL_HELP_ENTRIES = helpEntriesAll().filter(e => e.key_path !== null);

// Inside HelpOverlay:
// const allEntries = ALL_HELP_ENTRIES; // no useMemo needed
```

Or keep the `useMemo` but add the stable dep explicitly:

```tsx
const allEntries = useMemo(() =>
  helpEntriesAll().filter(e => e.key_path !== null),
[/* helpEntriesAll is a stable import — empty is fine, but consider the above */]);
```

### WR-03: `catalog_2_with_math1_loaded` test comment claims wrong name

**File:** `hp41-core/tests/op_catalog_xrom.rs:54`

**Issue:** The assertion message at line 54 reads `"Module header should contain 'MATH 1A': {module_header:?}"`, referencing the string `'MATH 1A'`. The actual `MATH_1.name` field value (per the xrom module definitions) is `"MATH 1A"` — but line 52 asserts `module_header.contains("MATH 1")`, not `"MATH 1A"`. The comment on line 54 is internally inconsistent: the `assert!` on line 51 checks for `"XROM 7"` (correct), while the `assert!` on line 52 checks for `"MATH 1"` (a substring of both `"MATH 1A"` and any other name containing those characters). The wrong comment creates a misleading false sense of precision.

This is a minor issue but the comment on line 54 (`'MATH 1A'`) implies the assertion is tighter than it is — the code only checks the substring `"MATH 1"`, not `"MATH 1A"`.

**Fix:**
```rust
// Line 52 — tighten to match the actual module name:
assert!(
    module_header.contains(MATH_1.name),
    "Module header should contain MATH_1.name ('{}') : {module_header:?}",
    MATH_1.name
);
```

## Info

### IN-01: `tick_time` Tauri permission identifier uses underscore, not kebab-case

**File:** `hp41-gui/src-tauri/permissions/tick-time.toml:6`

**Issue:** The `commands.allow` field at line 6 uses `"tick_time"` (underscore), which is the Rust function name. The Tauri v2.11 documentation indicates that commands are registered in `generate_handler!` by their Rust function name with underscores, while the permission `identifier` field (line 3) uses kebab-case (`"allow-tick-time"`). The `commands.allow` entry must match the Rust function name, so `"tick_time"` is correct. However, the project comment in CLAUDE.md says "for inline app commands, Tauri does NOT auto-generate `allow-<cmd>` permissions" — this is consistent with the implementation. No bug here, but the inconsistency between the identifier format (`allow-tick-time`, kebab) and the commands.allow format (`tick_time`, underscore) is worth a clarifying comment in the TOML.

**Fix:** Add a comment:
```toml
[[permission]]
identifier = "allow-tick-time"   # kebab-case per Tauri permission identifier convention
description = "Allows the tick_time command."
commands.allow = ["tick_time"]    # Rust fn name (underscore) — NOT kebab
```

### IN-02: `HelpOverlay.test.tsx` has a duplicate section-count test

**File:** `hp41-gui/src/HelpOverlay.test.tsx:261-270` and `372-380`

**Issue:** There are two nearly identical tests:
- Line 261: `'renders four top-level sections with HP-41CV, Math 1 Pac, Stat 1 Pac, and Time Pac headings (D-31.8)'`
- Line 372: `'renders four top-level sections including Time Pac (XROM 26)'`

Both assert the presence of the same four section heading strings. The second test (line 372) adds no new assertion beyond what the first covers. This duplication means a regression in section rendering would surface two failing tests with the same root cause, adding noise to CI output without increasing coverage.

The existing Phase 36 test at line 311 (`'renders three top-level sections including Stat 1 Pac (XROM 2)'`) is also superseded by the four-section test at line 261 — it now asserts a subset of what line 261 already covers.

**Fix:** Remove the redundant test at line 372 and consider removing the now-superseded three-section test at line 311, or update line 311 to test something the four-section test does not (e.g., a stat1-specific entry predicate).

---

_Reviewed: 2026-05-25_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
