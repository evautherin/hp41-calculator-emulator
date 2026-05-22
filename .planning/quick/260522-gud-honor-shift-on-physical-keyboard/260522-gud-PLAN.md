---
quick_id: 260522-gud
slug: honor-shift-on-physical-keyboard
description: GUI bug fix — physical-keyboard input ignores `shiftActive` after Tab (HP-41 `f`-prefix). Honor the one-shot SHIFT for physical keys, mirroring the on-screen-click path.
date: "2026-05-22"
status: planned
must_haves:
  truths:
    - "Physical keyboard: Tab → 0 → π enters the X register (mirrors HP-41 `f` + `0` → π)"
    - "Physical keyboard: Tab → any key that has a `.shifted` KEY_DEFS entry resolves to the shifted Op; SHIFT one-shot is consumed (cleared)"
    - "Physical keyboard: Tab → key with NO shifted variant: SHIFT stays armed (mirrors `handleClick`'s on-screen behavior, GUI internal consistency)"
    - "Physical keyboard: Tab → `s` outside PRGM: the regular `shifted` variant (`sq` / x²) wins, dispatched verbatim — proves the swap path runs"
    - "ALPHA mode pass-through is unaffected (ALPHA overrides SHIFT — `resolveKeyId` returns `alpha_<X>` BEFORE the new SHIFT block runs)"
    - "Modal-open path (`pendingInput !== null`) is unaffected — it already passes `shiftActive` into `handleModalKey`"
    - "No backend, IPC, SC-4, or 4-way exhaustive-match change. CLI ↔ GUI parity D-25.6 improved (closes physical-keyboard divergence)."
    - "`npm test` (hp41-gui) green; new regression cases added in App.test.tsx; existing 166 tests unchanged"
    - "`npx tsc --noEmit` green"
---

# Quick Task 260522-gud — GUI Physical-Keyboard SHIFT Honor

## Goal

Close a GUI behavioral defect surfaced by the user:

> Tab (SHIFT) → `0` on the physical keyboard yields **digit 0** instead of **π**.

The on-screen-click path correctly resolves shifted ops via `handleClick`'s
SHIFT block (`hp41-gui/src/App.tsx:324`). The physical-keyboard path
deliberately bypasses it (per the comment at `App.tsx:257-259`:
*"Physical-keyboard dispatch (option B): string-id path, no SHIFT/ALPHA
frontend mediation"*) — but this contradicts user expectation and the CLI
behavior (`hp41-cli/src/app.rs:484` → `shifted_key_to_op`).

## Root cause

`handleKey` (App.tsx, end of `useCallback`):

```ts
const keyId = resolveKeyId(e, calcState);
if (keyId === null) return;
e.preventDefault();
dispatchKeyId(keyId);
```

`resolveKeyId` returns primary op-ids; `shiftActive` is never consulted,
never consumed.

## Design

Insert a SHIFT-resolution block between `resolveKeyId` and `dispatchKeyId`,
mirroring `handleClick`'s logic (rule 3 — lines 324-341):

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
      setShiftActive(false);   // consume the one-shot (matches handleClick)
    }
  }
}

e.preventDefault();
dispatchKeyId(keyId);
```

### Semantics (intentional)

- **Swap-on-match only:** consume `shiftActive` only when a `.shifted`
  (or `.shiftedInPrgm` under PRGM) variant exists. Matches `handleClick`'s
  `consumesShift = true` gate — keeps physical and on-screen paths
  bit-for-bit identical in this dimension.
- **No SHIFT-clear on no-match:** preserves the existing GUI invariant. A
  minor divergence from the CLI (which clears unconditionally) but the
  alternative breaks `handleClick` parity. Re-aligning ALL three paths
  (CLI / on-screen / physical-keyboard) would touch `handleClick` and is
  out of scope for this quick-task.
- **ALPHA-override is preserved:** `resolveKeyId` returns `alpha_<X>` at
  line 114 BEFORE we get here, so the new block doesn't see alpha keys.
  CLAUDE.md's "ALPHA overrides SHIFT" invariant stays intact.
- **Modal-open path is untouched:** the `pendingInput !== null` branch
  (lines 528-542) already passes `shiftActive` into `handleModalKey`.

### Import

`KEY_DEFS` is exported from `Keyboard.tsx` for the existing W9 sentinel
test. Add it to the existing import on App.tsx line 4:

```ts
import { Keyboard, KEY_DEFS, type KeyDef } from './Keyboard';
```

## Verification

1. `npx tsc --noEmit` (hp41-gui) — green.
2. `npm test` (hp41-gui) — existing 166 tests pass + 3 new physical-keyboard
   regression cases pass.
3. Visual: Vite dev mode — Tab + 0 displays `3.14159…`; Tab + Tab cancels;
   Tab + (unmapped key) leaves SHIFT armed.

## Files modified

- `hp41-gui/src/App.tsx` — add SHIFT-resolution block in `handleKey`; add
  `KEY_DEFS` to existing Keyboard import.
- `hp41-gui/src/App.test.tsx` — add 3 regression cases (Tab+0→pi; Tab+key-in-
  PRGM→shiftedInPrgm; Tab+key-without-shifted→shiftActive stays true).

## Out-of-scope

- `handleClick` semantics (unchanged).
- CLI behavior (already correct).
- Backend, IPC, `key_map.rs`, `Op` enum.
- Re-aligning CLI/GUI shift-consumption-on-no-match.
- **MODAL_OPENERS intercept on the physical-keyboard path.** Surfaced
  during test design: physical-keyboard `Tab + s` in PRGM mode now
  correctly swaps to `keyId="clp_prompt"`, but the physical-keyboard
  branch of `handleKey` lacks the `MODAL_OPENERS[effectiveId]` intercept
  that `handleClick` performs at line 408-421. Result: `clp_prompt` is
  dispatched to the backend instead of opening the CLP modal locally.
  This is a separate pre-existing divergence between the on-screen and
  physical-keyboard paths and affects ALL modal-opener ids (not just
  SHIFT-resolved ones). Tracked for a future quick-task.

## Risks

- **D-25.6 (CLI ↔ GUI shift mirror):** improved, not violated. The bit-
  for-bit state mirror still holds; behavior on the physical keyboard
  now matches the CLI for the swap case.
- **W2 IND-toggle (shift-0 inside an open Flag/Register modal):** safe —
  the new block lives in the non-modal branch (after the `pendingInput
  !== null` early return).
- **Test snapshot drift:** none. All existing tests assert on `mockInvoke`
  call patterns; this fix only changes which `keyId` is forwarded when
  SHIFT is armed, which is what the new tests will assert positively.
