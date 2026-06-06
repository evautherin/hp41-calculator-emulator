---
phase: 55-touch-ui-adaptation
plan: "05"
subsystem: hp41-gui
tags: [ios, bottom-sheet, collapsible-stack, overscroll, touch, tdd]
dependency_graph:
  requires:
    - phase: 55-01
      provides: is_ios-command + isIos frontend flag
    - phase: 55-04
      provides: AlphaTouchInput bar + z-index 80 reference
  provides:
    - BottomSheet-component
    - bottom-sheet-CSS
    - collapsible-stack-panel
    - stack-panel-CSS
    - TOUCH-09-TOUCH-10-TOUCH-11-satisfied
  affects:
    - hp41-gui/src/BottomSheet.tsx
    - hp41-gui/src/BottomSheet.test.tsx
    - hp41-gui/src/App.css
    - hp41-gui/src/App.tsx
tech-stack:
  added: []
  patterns:
    - tdd-red-green
    - early-return-null (SettingsPanel analog)
    - isIos-gated-render-gate
    - max-height-transition (collapsible)

key-files:
  created:
    - hp41-gui/src/BottomSheet.tsx
    - hp41-gui/src/BottomSheet.test.tsx
  modified:
    - hp41-gui/src/App.css
    - hp41-gui/src/App.tsx

key-decisions:
  - "BottomSheet follows SettingsPanel early-return-null pattern: visible=false returns null; App.tsx gates mounting with isIos so desktop never mounts the component"
  - "Stack panel collapsible: X row always visible on iOS; Y/Z/T/L wrapped in .stack-panel-collapsible.collapsed; stackExpanded is local state (not persisted)"
  - "Desktop paths byte-for-byte unchanged: isIos ternary keeps inline .prgm-panel and .print-panel on desktop; stack always expanded on desktop"
  - "overscroll-behavior-y: contain in .bottom-sheet-content for inner scroll without outer bounce (TOUCH-11); body overscroll-behavior: none retained in index.css"

requirements-completed: [TOUCH-07, TOUCH-09, TOUCH-10, TOUCH-11]

duration: ~10min
completed: 2026-06-03
---

# Phase 55 Plan 05: Bottom Sheets + Collapsible Stack Summary

**Pull-up BottomSheet component for iOS print log and PRGM listing, collapsible stack panel with 44pt chevron, and overscroll suppression — all iOS-gated so desktop layout is unchanged.**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-06-03T10:29:00Z
- **Completed:** 2026-06-03T10:32:18Z
- **Tasks:** 2 (Task 1 TDD: RED + GREEN; Task 2)
- **Files modified:** 4 (2 new, 2 modified)

## Accomplishments

- `BottomSheet.tsx` component: iOS pull-up sheet with tap-to-toggle expand/collapse, empty-state copy, overscroll-contained inner scroll — analogous to SettingsPanel early-return pattern
- `BottomSheet.test.tsx`: 4 TDD behavior tests (null when hidden, renders title+children, toggle expand/collapse, empty-state copy)
- `App.css`: added `.bottom-sheet` + `.expanded` (max-height transition 300ms), `.bottom-sheet-header` / `.bottom-sheet-title` / `.bottom-sheet-handle` (48pt tap zone) / `.bottom-sheet-content` (overscroll-behavior-y: contain) / `.bottom-sheet-empty`
- `App.css`: added `.stack-panel-collapsible` + `.collapsed` (max-height 200ms transition), `.stack-panel-toggle` + `.stack-panel-toggle-btn` (44pt chevron)
- `App.tsx`: `stackExpanded` local state (default collapsed on iOS); iOS-gated BottomSheet for PRGM (visible when annunciators.prgm) and print log (visible when printLog.length > 0); collapsible Y/Z/T/L with 44pt chevron toggle; desktop paths preserved byte-for-byte via isIos ternary
- `index.css` `overscroll-behavior: none` verified retained (TOUCH-11)

## Task Commits

1. **Task 1 RED: Failing tests** - `676f77c` (test)
2. **Task 1 GREEN: BottomSheet component + CSS** - `788382f` (feat)
3. **Task 2: App.tsx wiring** - `29b9fd1` (feat)

**Plan metadata:** (docs commit follows)

_TDD: test(55-05) commit is RED gate; feat(55-05) implement is GREEN gate._

## Files Created/Modified

- `hp41-gui/src/BottomSheet.tsx` — New component: early-return-null guard, expanded/collapsed toggle state, empty-state copy, 48pt tap zone on handle
- `hp41-gui/src/BottomSheet.test.tsx` — 4 behavior tests: null when hidden, renders content, expand/collapse toggle, empty-state copy
- `hp41-gui/src/App.css` — Added bottom-sheet class family + stack-panel-collapsible / stack-panel-toggle-btn (Phase 55 Touch Adaptation section)
- `hp41-gui/src/App.tsx` — Added `import BottomSheet`, `stackExpanded` state, iOS-gated BottomSheet renders for prgm + print, iOS-gated collapsible stack with chevron

## Decisions Made

- **early-return-null gate:** BottomSheet returns null when `visible=false`. App.tsx mounts it only under `isIos` — this double-gate means desktop never has the component in the DOM.
- **Stack panel X always visible:** On iOS, the X register row is rendered outside the collapsible wrapper so the current value is always legible. Y/Z/T/L are in the collapsible div.
- **stackExpanded not persisted:** Local React state only — resets to collapsed on each app launch on iOS (per TOUCH-10 contract). No GuiPrefs entry needed.
- **Desktop ternary pattern:** Both prgm panel and print panel use `isIos ? <BottomSheet/> : <existingInlinePanel/>` — this is the cleanest way to keep desktop paths byte-for-byte unchanged without conditional imports.
- **z-index coherence:** bottom-sheet z-index: 50 is below AlphaTouchInput (z-index: 80) and below help/wizard/settings (z-index: 60+). No collision.

## Deviations from Plan

None — plan executed exactly as written. BottomSheet component, CSS classes, test coverage, and App.tsx wiring all implemented per the plan spec, UI-SPEC contract, and PATTERNS file.

## Verification Results

| Check | Result |
|-------|--------|
| `grep '.bottom-sheet' App.css` | ✓ .bottom-sheet + .bottom-sheet.expanded + header/title/handle/content/empty |
| `grep '.stack-panel-collapsible' App.css` | ✓ class + .collapsed modifier |
| `grep 'overscroll-behavior-y: contain' App.css` | ✓ in .bottom-sheet-content (TOUCH-11) |
| `grep 'min-height: 44px' App.css` | ✓ 2 matches (key-touch-target + stack-panel-toggle-btn) |
| `grep 'bottom-sheet' BottomSheet.tsx` | ✓ className uses bottom-sheet + expanded |
| `grep 'expanded' BottomSheet.tsx` | ✓ useState + className conditional |
| `grep -c 'BottomSheet' App.tsx` | ✓ 5 (import + comment + 2 renders + type reference) |
| `grep -c 'stack-panel-collapsible' App.tsx` | ✓ 2 (class + collapsed conditional) |
| `grep -c 'stackExpanded' App.tsx` | ✓ 6 (state declaration + setters + renders) |
| `grep 'overscroll-behavior: none' index.css` | ✓ retained (TOUCH-11) |
| `cd hp41-gui && npm test -- BottomSheet` | ✓ 4/4 pass |
| `cd hp41-gui && npm test` | ✓ 260/260 pass |
| `cd hp41-gui && just gui-ci` | ✓ release build clean, 260 tests pass |

## Known Stubs

None. BottomSheet is fully wired to existing CalcStateView fields (printLog, program_steps, pc, annunciators.prgm). Device-level verification (sheet pull-up, collapse, no rubber-band, legibility) batched in Plan 06 per D-55.4.

## Threat Flags

No new threat surface beyond the plan's `<threat_model>`:
- T-55-09 (accepted): Bottom sheets and stack panel are read-only presentation of already-validated CalcStateView fields; no new write path to Rust.
- T-55-10 (accepted): Print "Clear" reuses existing clear path (session-only buffer).

## TDD Gate Compliance

- RED gate: commit `676f77c` (`test(55-05): add failing BottomSheet tests (TDD RED)`) — module not found, tests failed as expected
- GREEN gate: commit `788382f` (`feat(55-05): implement BottomSheet component + bottom-sheet/stack CSS (TDD GREEN)`) — all 4 tests pass

## Self-Check: PASSED

- `hp41-gui/src/BottomSheet.tsx` — EXISTS ✓
- `hp41-gui/src/BottomSheet.test.tsx` — EXISTS ✓
- `hp41-gui/src/App.css` (.bottom-sheet + .stack-panel-collapsible) — VERIFIED ✓
- `hp41-gui/src/App.tsx` (import + stackExpanded + BottomSheet renders) — VERIFIED ✓
- Commit 676f77c (Task 1 RED) — EXISTS ✓
- Commit 788382f (Task 1 GREEN) — EXISTS ✓
- Commit 29b9fd1 (Task 2) — EXISTS ✓
