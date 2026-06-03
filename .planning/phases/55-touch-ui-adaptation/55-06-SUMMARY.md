---
phase: 55-touch-ui-adaptation
plan: "06"
subsystem: hp41-gui
tags: [ios, touch, alpha-input, xeq-modal, tdd, gap-fix, TOUCH-04]
dependency_graph:
  requires:
    - phase: 55-04
      provides: AlphaTouchInput component + render gate
  provides:
    - frontend-modal-ios-keyboard-bar
    - AlphaTouchInput-frontend-modal-mode
  affects:
    - hp41-gui/src/AlphaTouchInput.tsx
    - hp41-gui/src/AlphaTouchInput.test.tsx
    - hp41-gui/src/App.tsx
    - hp41-gui/src/App.test.tsx
tech-stack:
  added: []
  patterns:
    - tdd-red-green
    - frontend-modal-state-machine-routing
    - ios-gated-render-gate

key-files:
  created: []
  modified:
    - hp41-gui/src/AlphaTouchInput.tsx
    - hp41-gui/src/AlphaTouchInput.test.tsx
    - hp41-gui/src/App.tsx
    - hp41-gui/src/App.test.tsx

key-decisions:
  - "Frontend modal mode (isFrontendModalMode) is a third mode alongside isAlphaMode and isModalLabelMode; per-char callbacks route through App.tsx handleModalKey/applyModalResult so pending.acc is the single source of truth"
  - "ENTER=terminator design in pending_input.ts and CLI parity unchanged; only the render gate and char-routing are added"
  - "Desktop (isIos=false): all new props ignored; byte-for-byte unchanged"
  - "Prompt strings derived from pendingInput.kind: xeq_name+xeq→XEQ NAME?, gto→GTO NAME?, lbl→LBL NAME?; clp→CLP NAME?; assign_label→ASN NAME?"

duration: ~15min
completed: 2026-06-03
---

# Phase 55 Plan 06: Gap Fix — iOS AlphaTouchInput Bar for Frontend XEQ/GTO/LBL/CLP/ASN Modals

**Gap fix discovered during on-device verification (TOUCH-04): iOS showed no software-keyboard bar for XEQ/GTO/LBL/CLP/ASN text-label modals. Fixed with TDD.**

## Bug Description

**Root cause:** The `AlphaTouchInput` render gate in App.tsx checked only:
```
isIos && (calcState.annunciators.alpha || calcState.modal_requires_alpha_label)
```
`calcState.modal_requires_alpha_label` is set only by BACKEND `ModalProgram` module name-prompts (Advantage/Time/Stat/Math). The XEQ/GTO/LBL/CLP/ASN "FUNCTION NAME?" prompts are FRONTEND `pendingInput` modals (kinds: `xeq_name`, `clp`, `assign_label`) that do NOT set that backend flag.

**Consequence on iOS:** Pressing XEQ/GTO/LBL showed no software-keyboard bar. The on-screen calculator keypad was left, where the ENTER key terminates label entry (established design). Because ENTER terminates, the user could not type the ALPHA letter 'N' (which lives on the ENTER key), and tapping ENTER submitted an empty/partial label → hp41-core returned `HpError::InvalidOp` → "invalid operation" toast.

## Fix Applied (TDD RED → GREEN)

### RED Phase: Failing Tests

Added 5 new tests to `AlphaTouchInput.test.tsx` (tests 6-10) for the `isFrontendModalMode` path:
- Renders bar when `isFrontendModalMode=true` with `frontendModalPrompt` header
- Renders nothing when `isFrontendModalMode=false` (and others false)
- Typing calls `onFrontendModalChar` with uppercased char
- Backspace calls `onFrontendModalBackspace`
- Done button calls `onFrontendModalDone`

Added 2 integration tests to `App.test.tsx` (Group L):
- L1: isIos + XEQ touch overlay click → AlphaTouchInput bar renders with "XEQ NAME?"
- L2: isIos + no pendingInput → bar is absent

### GREEN Phase: Implementation

**`AlphaTouchInput.tsx`:** Added third mode `isFrontendModalMode` with props:
- `isFrontendModalMode: boolean` — gates the third render path
- `frontendModalPrompt: string | null` — derived from pendingInput.kind + dispatchPrefix
- `onFrontendModalChar(ch: string)` — per-char callback (input cleared after each; state lives in App.tsx `pendingInput.acc`)
- `onFrontendModalBackspace()` — routes Backspace through handleModalKey/applyModalResult
- `onFrontendModalDone()` — routes Enter through handleModalKey/applyModalResult (ENTER=terminator path unchanged)

Guard updated: `if (!isAlphaMode && !isModalLabelMode && !isFrontendModalMode) return null`

**`App.tsx`:** Replaced static render gate with IIFE that:
1. Computes `frontendLabelModal = isIos && pendingInput !== null && (kind ∈ {xeq_name, clp, assign_label})`
2. Derives `frontendModalPrompt` string per kind (XEQ/GTO/LBL NAME?, CLP NAME?, ASN NAME?)
3. Shows bar when any of the three conditions is true
4. Passes callbacks that call `handleModalKey(ch/Backspace/Enter, pendingInput, shiftActive)` → `applyModalResult(result)` — the existing frontend modal state machine handles all routing including the collect-for-modal path

## Behavioral Invariants Preserved

- ENTER=terminator design in `pending_input.ts` unchanged — `handleModalKey('Enter', ...)` is the established path
- CLI parity (D-25.6) unchanged — no changes to pending_input.ts or CLI
- Desktop (isIos=false): render gate still returns null; `frontendLabelModal` is always false
- Plain ALPHA register path (isAlphaMode) and backend module prompt path (isModalLabelMode) unchanged
- macOS/desktop builds byte-for-byte unchanged

## Commits

1. **RED:** `498848e` — `test(55-06): add failing tests for frontend-modal mode in AlphaTouchInput`
2. **GREEN:** `a3b7dcd` — `feat(55-06): fix TOUCH-04 — iOS AlphaTouchInput bar for frontend XEQ/GTO/LBL modals`
3. **Integration tests:** `4f0ea4d` — `test(55-06): add iOS integration tests for frontend modal AlphaTouchInput bar`

## Verification

| Check | Result |
|-------|--------|
| `npm test -- AlphaTouchInput` (11 tests) | 11/11 pass |
| `npm test` (full suite) | 267/267 pass |
| `just gui-ci` | clean (release build + all tests) |
| Desktop (isIos=false) unchanged | verified by reading App.tsx IIFE gate |
| ENTER=terminator in pending_input.ts | unchanged (verified by reading) |

## Known Stubs

None. Fix is complete; device-level verification (bar appears above iOS keyboard for XEQ/GTO/LBL/CLP/ASN; typing accumulates including 'N'; Done submits) is part of the Plan 06 on-device checkpoint.

## Threat Flags

None. The new routing path (`onFrontendModalChar` → `handleModalKey` → `applyModalResult` → `dispatch_op`) reuses the existing validated dispatch chain. No new IPC surface introduced.

## Self-Check: PASSED

- `hp41-gui/src/AlphaTouchInput.tsx` — modified with isFrontendModalMode props ✓
- `hp41-gui/src/AlphaTouchInput.test.tsx` — 5 new tests for frontend modal mode ✓
- `hp41-gui/src/App.tsx` — IIFE render gate + frontend modal callbacks ✓
- `hp41-gui/src/App.test.tsx` — Group L integration tests ✓
- Commit 498848e (RED) — EXISTS ✓
- Commit a3b7dcd (GREEN) — EXISTS ✓
- Commit 4f0ea4d (integration tests) — EXISTS ✓
