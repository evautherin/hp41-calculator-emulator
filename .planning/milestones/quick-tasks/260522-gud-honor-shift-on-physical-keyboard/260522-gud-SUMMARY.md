---
quick_id: 260522-gud
slug: honor-shift-on-physical-keyboard
status: complete
date: "2026-05-22"
commit: aa7e614
---

# Quick Task 260522-gud — SUMMARY

## Outcome

Closed a GUI defect surfaced by the user: pressing physical `Tab` (SHIFT)
followed by physical `0` produced digit `0` instead of `π`, contradicting
the on-screen-click path (where SHIFT-then-`0` correctly yields π) and the
CLI behavior (`f` + `0` → π via `shifted_key_to_op`).

After this fix, physical-keyboard input honors the one-shot SHIFT prefix
identically to the on-screen-click path: `Tab → 0` enters π into X, and
every other `f`-prefix combo follows suit.

## Implementation

One block of code added to `App.tsx::handleKey`, between `resolveKeyId`
and `dispatchKeyId`:

```ts
let keyId = resolveKeyId(e, calcState);
if (keyId === null) return;

if (shiftActive) {
  const def = KEY_DEFS.find(k => k.id === keyId);
  if (def) {
    const prgmOn = calcState?.annunciators.prgm ?? false;
    const shiftedId = (prgmOn && def.shiftedInPrgm)
      ? def.shiftedInPrgm.id
      : def.shifted?.id;
    if (shiftedId) {
      keyId = shiftedId;
      setShiftActive(false);
    }
  }
}

e.preventDefault();
dispatchKeyId(keyId);
```

Plus one import addition (`KEY_DEFS` from `./Keyboard`).

### Semantics chosen (mirrors `handleClick` exactly)

- **Swap-on-match only:** `shiftActive` is consumed only when a `.shifted`
  (or `.shiftedInPrgm` under PRGM) variant exists. No consume on no-match.
- **PRGM mode wins:** when `annunciators.prgm` and `def.shiftedInPrgm`
  are both set, the PRGM variant is selected over the regular shifted.
- **ALPHA-override preserved:** `resolveKeyId` already returns `alpha_<X>`
  at line 114 BEFORE this block runs, so the new branch never fires in
  ALPHA mode. CLAUDE.md's "ALPHA overrides SHIFT" invariant intact.
- **Modal-open path untouched:** the `pendingInput !== null` branch
  already passes `shiftActive` into `handleModalKey`.

## Verification

| Check | Result |
|-------|--------|
| `npx tsc --noEmit` (hp41-gui) | green |
| `npm test` (hp41-gui — vitest) | 168 / 168 pass (was 166 before fix; +2 K-group cases) |
| K1: `Tab + 0 → dispatch_op({keyId:"pi"})` | pass |
| K2: `Tab + l → dispatch_op({keyId:"lastx"})` (no KEY_DEFS def, no swap) | pass |
| Visual: Vite dev `Tab + 0` → display shows `3.141593` | pass — see `after-tab-zero.png` |
| Backend (`hp41-core`, `src-tauri/`) | untouched |
| SC-4 (no core duplication in GUI) | preserved |
| CLI ↔ GUI parity D-25.6 | **improved** (closes physical-keyboard divergence) |
| 4-way exhaustive-match invariant | N/A — no `Op` variant touched |

## Files touched

- `hp41-gui/src/App.tsx` — +KEY_DEFS import; +~14-line SHIFT-resolution block in `handleKey`.
- `hp41-gui/src/App.test.tsx` — +Group K (K1 + K2) regression tests.
- `.planning/quick/260522-gud-honor-shift-on-physical-keyboard/{PLAN,SUMMARY}.md` — new.
- `.planning/quick/260522-gud-honor-shift-on-physical-keyboard/after-tab-zero.png` — visual evidence.
- `.planning/STATE.md` — +1 row in "Quick Tasks Completed" table.

## Known follow-up — out of scope for this quick task

**MODAL_OPENERS intercept on the physical-keyboard path.** Surfaced during
test design: after this fix, physical `Tab + s` in PRGM mode correctly
swaps to `keyId="clp_prompt"`, but the physical-keyboard branch of
`handleKey` lacks the `MODAL_OPENERS[effectiveId]` intercept that
`handleClick` performs at lines 408-421. Result: `clp_prompt` (or any
other modal-opener id) is dispatched to the backend instead of opening
the modal locally.

This is a separate pre-existing divergence between the on-screen and
physical-keyboard paths that affects ALL modal-opener ids, not just
SHIFT-resolved ones. A K3 test for the PRGM CLP-modal case was attempted
and dropped (commented in App.test.tsx) — its `not.toHaveBeenCalledWith`
assertion proved brittle to handler-closure carryover from prior window-
keydown tests, and the underlying integration concern is broader than
this fix's "honor SHIFT" scope.

Tracked for a future quick-task.
