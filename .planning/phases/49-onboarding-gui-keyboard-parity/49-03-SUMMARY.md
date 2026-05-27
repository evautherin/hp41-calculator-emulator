---
phase: 49-onboarding-gui-keyboard-parity
plan: "03"
subsystem: ui
tags: [react, typescript, vitest, testing-library, css, onboarding, help-overlay, keyboard]

# Dependency graph
requires:
  - phase: 49-01
    provides: docs/keyboard-shortcuts.json (61-entry JSON single source of truth)
  - phase: 49-02
    provides: example/notes fields enriched in all 5 HP-41 function JSON files

provides:
  - OnboardingWizard component: 5-panel quick-start overlay with first-run/re-open Esc behavior
  - HelpOverlay KEYBOARD SHORTCUTS section: collapsible, collapsed by default, not searchable
  - HelpOverlay expandable entries: per-entry toggle for example/notes detail blocks
  - help_data.ts: KeyboardShortcut interface + getKeyboardShortcuts() accessor
  - help_data.ts: HelpEntry.example and HelpEntry.notes optional fields
  - All wizard/shortcut/expandable CSS classes in App.css per UI-SPEC

affects:
  - 49-04 (App.tsx integration: wire OnboardingWizard + SettingsPanel Quick Start section)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Standalone kbdExpanded state (not widening SectionDef union) for non-searchable KBD section"
    - "expandedEntries: Set<string> tracking op_variant keys; resets on overlay close"
    - "PANELS const array with ReactNode content (HelpOverlay SECTIONS pattern adaptation)"
    - "isFirstRun prop: conditional Esc handler (open && !isFirstRun)"

key-files:
  created:
    - hp41-gui/src/OnboardingWizard.tsx
    - hp41-gui/src/OnboardingWizard.test.tsx
  modified:
    - hp41-gui/src/help_data.ts
    - hp41-gui/src/HelpOverlay.tsx
    - hp41-gui/src/HelpOverlay.test.tsx
    - hp41-gui/src/App.css

key-decisions:
  - "Standalone kbdExpanded state in HelpOverlay instead of widening SectionDef union (PATTERNS.md divergence / D-49.10) — cleaner separation since KBD section has different content shape and is not searchable"
  - "ReactNode used for PANELS content (not JSX.Element) — namespace-safe in React 18 TypeScript config"
  - "node_modules symlink in worktree needed for vitest to resolve @testing-library/react during task verification"

patterns-established:
  - "Non-searchable collapsible section: standalone state + render before sectionGroups.map()"
  - "Expandable entry toggle: Set<string> of op_variant keys, reset in open useEffect"

requirements-completed: [ONBOARD-01, ONBOARD-03, ONBOARD-04, KBD-03]

# Metrics
duration: ~18min
completed: 2026-05-27
---

# Phase 49 Plan 03: OnboardingWizard + HelpOverlay Extensions Summary

**5-panel OnboardingWizard overlay with first-run/re-open Esc semantics, HelpOverlay extended with collapsible Keyboard Shortcuts section (collapsed by default, not searchable) and per-entry expand toggles for example/notes fields; all 208 Vitest tests green.**

## Performance

- **Duration:** ~18 min
- **Started:** 2026-05-27T18:35:00Z
- **Completed:** 2026-05-27T18:44:00Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Created OnboardingWizard.tsx: 5-panel RPN quick-start guide overlay following HelpOverlay overlay pattern exactly — first-run blocks Esc/click-outside, re-open allows both; panel counter "N of 5"; reset to panel 1 on each open (D-49.9); 11 unit tests all pass
- Extended HelpOverlay.tsx: KEYBOARD SHORTCUTS section renders as standalone block above function sections, collapsed by default, sourced from `getKeyboardShortcuts()`; expandable entries with toggle for `example`/`notes` fields; `kbdExpanded` and `expandedEntries` both reset on overlay open
- Extended help_data.ts: `HelpEntry` now has optional `example?: string` and `notes?: string` fields (D-49.5); `KeyboardShortcut` interface and `getKeyboardShortcuts()` accessor added with Vite static JSON-import pattern
- Added all required CSS classes to App.css per UI-SPEC: wizard overlay, shortcut table, expandable entry detail, settings action button; all colors via CSS custom properties

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend help_data.ts + create OnboardingWizard component + tests** - `73c6855` (feat)
2. **Task 2: Extend HelpOverlay with KBD shortcuts section + expandable entries + CSS** - `18d38a9` (feat)

**Plan metadata:** (to be added by docs commit)

## Files Created/Modified

- `hp41-gui/src/OnboardingWizard.tsx` — 5-panel quick-start wizard overlay component (new)
- `hp41-gui/src/OnboardingWizard.test.tsx` — 11 unit tests for wizard navigation, Esc, panel counter (new)
- `hp41-gui/src/help_data.ts` — Added `example?`/`notes?` to HelpEntry; KeyboardShortcut interface + getKeyboardShortcuts()
- `hp41-gui/src/HelpOverlay.tsx` — KEYBOARD SHORTCUTS collapsible section + expandable function entries
- `hp41-gui/src/HelpOverlay.test.tsx` — 7 new tests (KBD section + expandable entries); section count 6→7
- `hp41-gui/src/App.css` — ~150 lines of new CSS: wizard overlay, shortcut table, expand detail, settings action btn

## Decisions Made

- **Standalone kbdExpanded state** (not widening SectionDef union): PATTERNS.md suggested widening the SectionDef id union with a `'kbd'` key, but the plan explicitly diverges from this. The KBD section has different content shape (shortcut rows, not HelpEntry rows), is not searchable, and would require special-casing at every render site. Standalone `kbdExpanded: boolean` state + a separate render block before the `sectionGroups.map()` loop is cleaner.
- **ReactNode type for PANELS content**: `JSX.Element` fails TypeScript strict mode in the project's React 18 config (`JSX` namespace not found); `ReactNode` is the correct type for renderable React children.
- **node_modules symlink for worktree**: Vitest in the worktree can't resolve `@testing-library/react` without `node_modules`; created a symlink from worktree's `hp41-gui/node_modules` → main repo's `hp41-gui/node_modules`. This is a worktree environment pattern (already gitignored by `node_modules/` rule).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated sectionButtons.length assertion from 6 to 7 in HelpOverlay.test.tsx**
- **Found during:** Task 2 (first test run after adding KEYBOARD SHORTCUTS section)
- **Issue:** The existing test `clicking section heading toggles aria-expanded (D-31.8)` had `expect(sectionButtons.length).toBe(6)` — accurate before Phase 49, but the new KEYBOARD SHORTCUTS section heading is also rendered with `.help-overlay-section-heading` class
- **Fix:** Updated assertion to `expect(sectionButtons.length).toBe(7)` with updated comment
- **Files modified:** hp41-gui/src/HelpOverlay.test.tsx
- **Verification:** All 54 tests in HelpOverlay.test.tsx + OnboardingWizard.test.tsx pass
- **Committed in:** 18d38a9 (Task 2 commit)

**2. [Rule 1 - Bug] Fixed JSX.Element -> ReactNode in OnboardingWizard.tsx**
- **Found during:** Task 1 verification (TypeScript check after tests passed)
- **Issue:** `interface PanelDef { content: JSX.Element }` caused TS2503: "Cannot find namespace 'JSX'" — React 18 TypeScript config doesn't expose the global JSX namespace
- **Fix:** Added `type ReactNode` import from react; changed `content: JSX.Element` to `content: ReactNode`
- **Files modified:** hp41-gui/src/OnboardingWizard.tsx
- **Verification:** `node_modules/.bin/tsc --noEmit` exits 0 (clean)
- **Committed in:** 18d38a9 (Task 2 commit, fix applied before staging)

---

**Total deviations:** 2 auto-fixed (2 Rule 1 bugs)
**Impact on plan:** Both auto-fixes necessary for correctness. No scope creep.

## Issues Encountered

Vitest in the git worktree cannot resolve `@testing-library/react` because `node_modules` only exists in the main repo checkout. Created a symlink `hp41-gui/node_modules -> /main-repo/hp41-gui/node_modules` to enable test execution. This is already excluded by the root `.gitignore` (`node_modules/` pattern). Plan 02 SUMMARY documented the same worktree limitation.

## Known Stubs

None — all rendered content is real data from JSON files or static text. No placeholder text, no hardcoded empty values, no TODO/FIXME in new code.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries introduced. All new surface is static component rendering and JSON data display.

## Self-Check: PASSED

Files exist:
- FOUND: hp41-gui/src/OnboardingWizard.tsx
- FOUND: hp41-gui/src/OnboardingWizard.test.tsx
- FOUND: hp41-gui/src/HelpOverlay.tsx
- FOUND: hp41-gui/src/HelpOverlay.test.tsx
- FOUND: hp41-gui/src/help_data.ts
- FOUND: hp41-gui/src/App.css

Commits exist:
- FOUND: 73c6855 (feat(49-03): extend help_data.ts + create OnboardingWizard component + tests)
- FOUND: 18d38a9 (feat(49-03): extend HelpOverlay with KBD shortcuts section + expandable entries + CSS)

Tests: 208/208 pass (vitest run — all 7 test files)
TypeScript: clean (tsc --noEmit exit 0)
CSS classes: wizard-overlay + shortcut-table + help-entry-expand-btn + settings-action-btn all present in App.css (grep count ≥ 4)

## Next Phase Readiness

- Plan 49-04 (App.tsx integration) can import and wire `OnboardingWizard` immediately
- `getKeyboardShortcuts()` is exported from help_data.ts and ready for consumption
- HelpEntry.example/notes fields are available — HelpOverlay already renders them
- SettingsPanel placeholder comment at line 66 ready for Quick Start section injection
