---
phase: 55-touch-ui-adaptation
plan: "06"
subsystem: hp41-gui
tags: [ios, touch, alpha-input, xeq-modal, tdd, TOUCH-04, keypad-only-entry]
dependency_graph:
  requires:
    - phase: 55-04
      provides: AlphaTouchInput component + render gate
  provides:
    - keypad-only-name-entry-for-text-label-modals
    - enter-types-N-alpha-terminates-in-text-label-modals
  affects:
    - hp41-gui/src/App.tsx
    - hp41-gui/src/App.test.tsx
    - hp41-gui/src/AlphaTouchInput.tsx
    - hp41-gui/src/AlphaTouchInput.test.tsx
tech-stack:
  added: []
  patterns:
    - tdd-red-green
    - frontend-modal-state-machine-routing
    - ios-gated-render-gate

key-files:
  created: []
  modified:
    - hp41-gui/src/App.tsx
    - hp41-gui/src/App.test.tsx
    - hp41-gui/src/AlphaTouchInput.tsx
    - hp41-gui/src/AlphaTouchInput.test.tsx

key-decisions:
  - "On-screen ENTER key types letter N (its ALPHA character) in text-label modals (xeq_name/clp/assign_label); on-screen ALPHA key terminates/submits — HP-41-faithful keypad behavior"
  - "Physical keyboard Enter still terminates for D-25.6 CLI/desktop parity — accepted on-screen-vs-physical divergence (user decision)"
  - "No iOS software keyboard bar for text-label modals — rejected on-device (keyboard pushes layout; blind entry); keypad-only approach supersedes"
  - "AlphaTouchInput.tsx restored to original two-mode form (isAlphaMode + isModalLabelMode only); plain ALPHA register and backend modal-label prompt modes unchanged"
  - "Rejected approach commits 498848e/a3b7dcd/4f0ea4d superseded in develop"

duration: ~30min
completed: 2026-06-03
---

# Phase 55 Plan 06: TOUCH-04 — Keypad-Only Name Entry (Supersedes iOS Keyboard Bar Approach)

**On-device rejection of the iOS software-keyboard bar for XEQ/GTO/LBL/CLP/ASN modals led to a redesign: keypad-only entry with ENTER typing 'N' and ALPHA terminating.**

## Root Cause (Unchanged from Original)

The `AlphaTouchInput` render gate in App.tsx checked:
```
isIos && (calcState.annunciators.alpha || calcState.modal_requires_alpha_label)
```
`calcState.modal_requires_alpha_label` is only set by BACKEND `ModalProgram` module name-prompts (Advantage/Time/Stat/Math). The XEQ/GTO/LBL/CLP/ASN prompts are FRONTEND `pendingInput` modals (`xeq_name`, `clp`, `assign_label`) that do NOT set that backend flag.

**Core problem:** On iOS, pressing XEQ/GTO/LBL showed no input mechanism. The on-screen ENTER key (which terminates label entry) contains the ALPHA letter 'N' — without a way to type 'N', function names like TONE, SIN, RUN were unreachable.

## First Attempt — Rejected On-Device

Commits `498848e` (RED), `a3b7dcd` (GREEN), `4f0ea4d` (integration tests) implemented an iOS software-keyboard bar (`isFrontendModalMode` prop in AlphaTouchInput) that appeared when a text-label `pendingInput` was active.

**On-device rejection reasons:**
1. iOS software keyboard pushes the whole calculator layout up — bad UX (occludes the HP-41 display and stack).
2. Entry is "blind": typed characters appear in the iOS text field, not in the calculator display (the user sees a separate line, not the accumulating "XEQ TONE_" display they expect).

## Final Design — Keypad-Only Name Entry

**User decisions (authoritative):**
1. No iOS software keyboard for text-label modals. Name entry via on-screen HP-41 keypad only.
2. On-screen **ENTER** key (alphaChar `'N'`) → types letter **N** in text-label modals. Live display updates: `XEQ T_` → `XEQ TO_` → `XEQ TON_` → `XEQ TONE_`.
3. On-screen **ALPHA** key (`alpha_toggle`) → terminates/submits the name entry. HP-41-faithful (ALPHA exits alpha-entry mode on the hardware).
4. Plain ALPHA-register mode and backend module prompts: **unchanged** — AlphaTouchInput bar still appears for those.
5. **Physical keyboard**: Enter still terminates for D-25.6 CLI/desktop parity — accepted on-screen-vs-physical divergence.

## Implementation (TDD RED → GREEN)

### RED Phase
Commits `fd642e0` — added failing Group M tests to `App.test.tsx`:
- **M1**: On-screen ENTER in xeq_name modal must append 'N', display "XEQ N_" — FAILS (ENTER terminates)
- **M2**: Type TONE via keypad (9=T, CHS=O, ENTER=N, LN=E), ALPHA submits xeq_TONE — FAILS (ALPHA was no-op)
- **M3**: isIos + xeq_name modal → NO AlphaTouchInput bar — FAILS (bar was shown)
- **M3b**: isIos + alpha annunciator → bar IS shown — PASSES (regression guard)
- **M4**: ENTER in FMT modal (numeric) → display stays "FIX _" — PASSES (ENTER=N gated to text-label kinds only)

### GREEN Phase
Commit `26f8b4e` — implementation in four files:

**`AlphaTouchInput.tsx`** — restored byte-for-byte to original two-mode form (isAlphaMode + isModalLabelMode). Removed `isFrontendModalMode` and related props. Verified identical to commit `8af8dd6`.

**`App.tsx` render gate** — restored to original simple form:
```tsx
{isIos && (calcState.annunciators.alpha || calcState.modal_requires_alpha_label) && (
  <AlphaTouchInput ... />
)}
```
Text-label modals (xeq_name/clp/assign_label) do NOT trigger the bar per user decision.

**`App.tsx` handleClick modal-routing block** — added two branches in priority order BEFORE the generic `enter → 'Enter'` branch:
```typescript
const isTextLabelKind =
  pendingInput.kind === 'xeq_name' ||
  pendingInput.kind === 'clp' ||
  pendingInput.kind === 'assign_label';

// (c) TOUCH-04: on-screen ALPHA terminates text-label modals
} else if (isTextLabelKind && effectiveId === 'alpha_toggle') {
  routedKey = 'Enter';

// (d) TOUCH-04: on-screen keys with alphaChar type their letter
//     ENTER (alphaChar='N') types 'N'; priority over enter→'Enter' below
} else if (isTextLabelKind && key.alphaChar) {
  routedKey = key.alphaChar;

// (e) unchanged: non-text-label modal ENTER terminates
} else if (effectiveId === 'enter') {
  routedKey = 'Enter';
```

A code comment documents the accepted on-screen-vs-physical divergence referencing the user decision.

**`AlphaTouchInput.test.tsx`** — removed tests 6-10 (isFrontendModalMode path).

**`App.test.tsx`** — removed Group L (rejected iOS bar approach). Updated tests A2, C1, F1, G3, G4 to use `alpha_toggle` instead of `enter` to submit text-label modals (now HP-41-faithful).

## Behavioral Invariants Preserved

| Invariant | Status |
|-----------|--------|
| `pending_input.ts` unchanged — Enter terminates, Backspace pops, printable chars append | Unchanged |
| Physical keyboard Enter terminates text-label modals (D-25.6 CLI parity) | Unchanged |
| Plain ALPHA register mode bar | Unchanged |
| Backend `modal_requires_alpha_label` mode bar | Unchanged |
| Desktop (isIos=false) — no bar rendered | Unchanged |
| Backspace (← key) deletes a char in text-label modals | Unchanged |
| Non-text-label modals (register, flag, fmt, single_digit) | Unchanged |

## Live Display Build-Up

The display updating live as characters are appended is existing behavior via `handleModalKey → applyModalResult → setPendingInput`. `renderModalLcd` for `xeq_name` returns `padCursor('XEQ ' + acc, 1)` which produces `XEQ TONE_` etc. Test M2 verifies each intermediate state.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Existing tests G3, G4, C1, A2, F1 used on-screen ENTER to submit text-label modals**
- **Found during:** GREEN phase implementation
- **Issue:** These tests expected `clickKey(container, 'enter')` to terminate/submit label modals. The new TOUCH-04 behavior makes on-screen ENTER type 'N' instead.
- **Fix:** Updated those tests to use `clickKey(container, 'alpha_toggle')` for submission — correctly reflects the new HP-41-faithful behavior (ALPHA terminates, ENTER types N).
- **Files modified:** `hp41-gui/src/App.test.tsx`

## Commits

| Phase | Hash | Message |
|-------|------|---------|
| RED | `fd642e0` | `test(55-06): add RED tests for TOUCH-04 keypad-only name entry` |
| GREEN | `26f8b4e` | `feat(55-06): TOUCH-04 keypad-only name entry — ENTER types N, ALPHA terminates` |

Previous approach commits (`498848e`, `a3b7dcd`, `4f0ea4d`) remain in git history for reference but are fully superseded.

## Verification

| Check | Result |
|-------|--------|
| `npm test` (full suite) | 265/265 pass |
| `just gui-ci` | clean (Rust build + all tests + SC-4 invariant) |
| `AlphaTouchInput.tsx` identical to original `8af8dd6` | verified (diff empty) |
| M1: ENTER types N in xeq_name modal | PASS |
| M2: TONE spelled T(9)+O(CHS)+N(ENTER)+E(LN), ALPHA submits xeq_TONE | PASS |
| M3: No AlphaTouchInput bar when xeq_name modal active on iOS | PASS |
| M3b: AlphaTouchInput bar shown for plain alpha mode on iOS | PASS |
| M4: ENTER in FMT modal stays no-op (not text-label kind) | PASS |

## Known Stubs

None. The fix is complete for on-screen keypad entry. Device-level verification (live display updates, ENTER=N, ALPHA=terminate) is the verification checkpoint.

## Threat Flags

None. Changes are confined to the on-screen click routing in `handleClick` — a frontend-only code path that translates on-screen key IDs to the existing `handleModalKey`/`applyModalResult` dispatch chain. No new IPC surface. No Rust changes.

## Self-Check: PASSED

- `hp41-gui/src/App.tsx` modified — new isTextLabelKind + alpha_toggle/alphaChar branches ✓
- `hp41-gui/src/App.test.tsx` modified — Group M added, Group L removed, A2/C1/F1/G3/G4 updated ✓
- `hp41-gui/src/AlphaTouchInput.tsx` restored to original ✓
- `hp41-gui/src/AlphaTouchInput.test.tsx` tests 6-10 removed ✓
- Commit `fd642e0` (RED) — EXISTS ✓
- Commit `26f8b4e` (GREEN) — EXISTS ✓
- All tests pass (265/265) ✓
- `just gui-ci` clean ✓

## On-Device Verification (2026-06-03) — PASSED

Authoritative on-device run (D-55.4). Device: **iPhone 15 Pro** (paired `DS`) — the planned iPhone SE was unavailable; all touch behaviors are valid on the 15 Pro, with the caveat that 44pt fat-finger accuracy (TOUCH-01) is slightly more forgiving on the larger screen. Release IPA built via `just ios-build` (signed, team 2P4R8QSWT4), installed + cold-launched via `xcrun devicectl`.

**Task 1 — automated gate:** PASS — `npm test` (260), `cargo check --target aarch64-apple-ios` clean (cfg(mobile) guard), `just gui-ci` clean.

**Tasks 2–5 — human checkpoints:** all **approved** by user on device:
- TOUCH-01/03/06: 44pt hit accuracy, instant press feedback (no tap-delay/flash), audio on first use — PASS
- TOUCH-04/05/08: haptic tiers (digit<ENTER<SHIFT), distinct error haptic, ALPHA + label entry — PASS
- TOUCH-07/09/10/11: legibility, print/PRGM bottom sheets, collapsible stack, no rubber-band overscroll — PASS

**Two defects found on device and fixed during verification:**
1. **TOUCH-04 keypad name entry (this plan).** XEQ/GTO/LBL `FUNCTION NAME?` prompts are frontend `pendingInput` modals; the iOS keyboard bar (keyed off the backend `modal_requires_alpha_label`) never appeared for them, so the on-screen ENTER key (which terminated entry) could not type its ALPHA letter 'N'. Final design (user-decided): **keypad-only** entry with live display build-up — on-screen ENTER types 'N', ALPHA terminates, in `xeq_name`/`clp`/`assign_label` modals; shared `handleClick` so desktop GUI + iOS are identical. The earlier iOS-keyboard-bar attempt was reverted (user UX rejection). Commits `fd642e0`/`26f8b4e`.
2. **← digit-backspace (cross-cutting, NOT a TOUCH requirement).** Pre-existing fidelity bug: ← always did CLX in both CLI and GUI instead of deleting the last keyed digit. Added shared core helper `hp41_core::ops::backspace_entry` (entry_buf-aware: pop last char mid-entry, else CLX), wired identically into CLI + GUI (parity D-25.6, no duplication SC-4). Commits `1369e10`…`95cb448`. `just ci` (3024+455) + `just gui-ci` (268) green.

All D-55.4 manual-only verifications recorded as PASS. Phase 55 touch UI confirmed on hardware.
