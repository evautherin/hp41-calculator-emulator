---
phase: 55-touch-ui-adaptation
plan: "04"
subsystem: hp41-gui
tags: [ios, alpha-input, touch, modal, visualViewport, keyboard-inset, tdd]
dependency_graph:
  requires:
    - phase: 55-01
      provides: is_ios-command + isIos frontend flag
    - phase: 55-03
      provides: haptics wiring + audio resume (haptics.ts mocks needed in test)
  provides:
    - AlphaTouchInput-component
    - alpha-touch-input-bar-CSS
    - modal-label-touch-entry
    - App.tsx-AlphaTouchInput-render-gate
  affects:
    - hp41-gui/src/AlphaTouchInput.tsx
    - hp41-gui/src/AlphaTouchInput.test.tsx
    - hp41-gui/src/App.css
    - hp41-gui/src/App.tsx
tech-stack:
  added: []
  patterns:
    - tdd-red-green
    - visualViewport-keyboard-tracking
    - early-return-null (SettingsPanel analog)
    - controlled-input-char-dispatch
    - ios-gated-render-gate

key-files:
  created:
    - hp41-gui/src/AlphaTouchInput.tsx
    - hp41-gui/src/AlphaTouchInput.test.tsx
  modified:
    - hp41-gui/src/App.css
    - hp41-gui/src/App.tsx

key-decisions:
  - "AlphaTouchInput separated into outer wrapper (early-return null) + AlphaTouchInputInner (hooks run unconditionally per Rules of Hooks)"
  - "ALPHA mode: each character dispatched immediately via alpha_<X> and input cleared; Backspace → clx; Done → alpha_toggle — mirrors the desktop physical-keyboard resolveKeyId path"
  - "Modal-label mode: accumulate in local state; Backspace removes last char without dispatch; Done → onSubmitLabel(accumulated) → submit_modal_with_label — avoids duplicating pendingInput state machine (D-55.2 / RESEARCH Open Question 3)"
  - "visualViewport keyboard formula: window.innerHeight - vv.height - vv.offsetTop (not vv.height alone) — Tauri #10631 workaround"
  - "App.tsx render gate: isIos && (annunciators.alpha || modal_requires_alpha_label) — covers both ALPHA-register and modal-label prompts (D-55.2 scope: FUNCTION NAME? / LBL / XEQ / GTO / CLP / ASN all covered)"

requirements-completed: [TOUCH-04]

duration: ~10min
completed: 2026-06-03
---

# Phase 55 Plan 04: ALPHA Touch Input Summary

**AlphaTouchInput component + visualViewport keyboard tracking + iOS-gated App.tsx render gate, covering both ALPHA-register and modal-label prompts via existing alpha_<X> + submit_modal_with_label dispatch paths.**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-06-03T08:22:00Z
- **Completed:** 2026-06-03T08:26:06Z
- **Tasks:** 2 (Task 1 TDD: RED + GREEN; Task 2)
- **Files modified:** 4

## Accomplishments

- `AlphaTouchInput.tsx` component: iOS soft-keyboard text entry bar for both ALPHA-register mode and modal-label prompts (FUNCTION NAME?, LBL, XEQ, GTO, CLP, ASN) — D-55.2
- `visualViewport` resize/scroll listener computing `window.innerHeight - vv.height - vv.offsetTop` for Tauri #10631-safe keyboard height tracking
- `App.css` `.alpha-touch-input-bar` + nested input (16px font-size — iOS auto-zoom prevention, Pitfall 5) + `.alpha-touch-input-label` classes
- App.tsx render gate: `isIos && (annunciators.alpha || modal_requires_alpha_label)` — desktop path byte-for-byte unchanged
- 6 new Vitest tests (all green); full suite 256/256 pass; `just gui-ci` clean

## Task Commits

1. **Task 1 RED: Failing tests** - `02502a3` (test)
2. **Task 1 GREEN: AlphaTouchInput + CSS** - `782916a` (feat)
3. **Task 2: App.tsx wiring** - `8af8dd6` (feat)

**Plan metadata:** (docs commit follows)

_TDD: test(55-04) commit is RED gate; feat(55-04) implement is GREEN gate._

## Files Created/Modified

- `hp41-gui/src/AlphaTouchInput.tsx` — New component: outer null-guard + AlphaTouchInputInner with visualViewport tracking, char dispatch, Done/Backspace routing
- `hp41-gui/src/AlphaTouchInput.test.tsx` — 6 behavior tests: render (alpha + modal), no-render (both false), alpha_A dispatch, clx Backspace, onSubmitLabel on Done
- `hp41-gui/src/App.css` — Added `.alpha-touch-input-bar` (fixed, z-index 80), `.alpha-touch-input-bar input` (16px), `.alpha-touch-input-label` in Phase 55 Touch Adaptation section
- `hp41-gui/src/App.tsx` — Added `import AlphaTouchInput` + iOS-gated render block after `<Keyboard>`

## Decisions Made

- **Wrapper + Inner split:** Outer `AlphaTouchInput` handles the early-return-null guard; `AlphaTouchInputInner` runs hooks unconditionally (Rules of Hooks). Same as SettingsPanel analog.
- **ALPHA mode char routing:** Each char dispatched immediately via `alpha_${ch}` (uppercased, filtered to [A-Z0-9 ]) and input cleared — mirrors desktop `resolveKeyId` behavior. Backspace → `clx`. Done → `alpha_toggle`.
- **Modal-label accumulation:** Local state accumulated; Done → `onSubmitLabel(inputValue)` → `invoke('submit_modal_with_label', { label })`. Avoids duplicating the `pendingInput` state machine (RESEARCH Open Question 3 recommendation followed).
- **16px font-size:** Non-negotiable per Pitfall 5 — iOS auto-zooms viewport on inputs below 16px, fighting `computeScale`.

## Deviations from Plan

None — plan executed exactly as written. The component, CSS, tests, and App.tsx render gate all implemented per the plan spec and patterns from RESEARCH/PATTERNS.

## Verification Results

| Check | Result |
|-------|--------|
| `grep 'visualViewport' AlphaTouchInput.tsx` | ✓ 3 occurrences |
| `grep 'window.innerHeight - vv.height - vv.offsetTop' AlphaTouchInput.tsx` | ✓ formula present (Pitfall 4 workaround) |
| `grep 'font-size: 16px' App.css` (inside alpha input rule) | ✓ present |
| `grep 'alpha-touch-input-bar' App.css` | ✓ class present |
| `grep "alpha_" AlphaTouchInput.tsx` | ✓ alpha_${ch} dispatch + alpha_toggle |
| `grep 'onSubmitLabel' AlphaTouchInput.tsx` | ✓ prop + call sites |
| `grep -c 'AlphaTouchInput' App.tsx` | ✓ 3 (import + comment + render) |
| `grep 'modal_requires_alpha_label' App.tsx` (render gate) | ✓ present in gate condition |
| `grep "submit_modal_with_label" App.tsx` (onSubmitLabel wiring) | ✓ present |
| `cd hp41-gui && npm test -- AlphaTouchInput` (6 tests) | ✓ 6/6 pass |
| `cd hp41-gui && npm test` (full suite) | ✓ 256/256 pass |
| `cd hp41-gui && just gui-ci` | ✓ release build clean, 256 tests pass |

## Known Stubs

None. AlphaTouchInput fully wired to existing IPC paths. Device-level verification (bar stays above keyboard, characters commit, modal label submits, Done exits ALPHA) batched in Plan 06 per D-55.4.

## Threat Flags

No new threat surface beyond the plan's `<threat_model>`:
- T-55-07 (mitigated): Each char uppercased and filtered to [A-Z0-9 ] before forming `alpha_<X>`; routed through `dispatch_op` which Rust validates via `key_map::resolve`.
- T-55-08 (mitigated): Label submitted verbatim to existing `submit_modal_with_label` command which validates HP-41 label length in Rust.

## TDD Gate Compliance

- RED gate: commit `02502a3` (`test(55-04): add failing tests...`) — tests failed as expected (module not found)
- GREEN gate: commit `782916a` (`feat(55-04): implement AlphaTouchInput component + CSS (GREEN)`) — all 6 tests pass

## Self-Check: PASSED

- `hp41-gui/src/AlphaTouchInput.tsx` — EXISTS ✓
- `hp41-gui/src/AlphaTouchInput.test.tsx` — EXISTS ✓
- `hp41-gui/src/App.css` (`.alpha-touch-input-bar` + 16px input) — VERIFIED ✓
- `hp41-gui/src/App.tsx` (import + render gate) — VERIFIED ✓
- Commit 02502a3 (Task 1 RED) — EXISTS ✓
- Commit 782916a (Task 1 GREEN) — EXISTS ✓
- Commit 8af8dd6 (Task 2) — EXISTS ✓
