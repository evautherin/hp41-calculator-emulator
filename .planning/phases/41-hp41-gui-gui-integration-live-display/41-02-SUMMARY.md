---
phase: 41-hp41-gui-gui-integration-live-display
plan: "02"
subsystem: hp41-gui/frontend
tags: [help-overlay, time-pac, xrom-26, vite-json-import, vitest, typescript]
dependency_graph:
  requires:
    - 39-hp41-cli-cli-integration-live-display (docs/hp41-time-functions.json authored)
  provides:
    - hp41-gui/src/help_data.ts helpEntriesTime() + 4-pool helpEntriesAll()
    - hp41-gui/src/HelpOverlay.tsx 4-section overlay including Time Pac (XROM 26)
    - hp41-gui/src/HelpOverlay.test.tsx vitest assertions for Time Pac data + rendering
  affects:
    - TIME-GUI-03 (help overlay fourth section completed)
tech_stack:
  added: []
  patterns:
    - Vite static JSON import (4th file: docs/hp41-time-functions.json)
    - HelpEntry readonly accessor pattern (4th accessor: helpEntriesTime)
    - 4-pool helpEntriesAll() concatenation
    - SectionDef 4-entry SECTIONS array with 'time' id
key_files:
  created: []
  modified:
    - hp41-gui/src/help_data.ts
    - hp41-gui/src/HelpOverlay.tsx
    - hp41-gui/src/HelpOverlay.test.tsx
decisions:
  - Predicate uses `e.xrom?.module === 'Time'` (not 'TIME', 'Time Pac', or 'TIME 2C') per D-39.9 xrom.module field value confirmed in docs/hp41-time-functions.json
  - helpEntriesAll() updated in-place per Pitfall 5 (no helpEntriesAll4() parallel variant)
  - node_modules installed in worktree's hp41-gui/ — no new packages, standard npm install of existing package.json
metrics:
  duration: "~10 minutes"
  completed: "2026-05-25T09:05:55Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 3
  tests_added: 8
  tests_total_passing: 180
requirements_completed:
  - TIME-GUI-03
---

# Phase 41 Plan 02: Help Overlay Time Pac Fourth Section Summary

Time Pac (XROM 26) fourth section wired into GUI help overlay: Vite static JSON import for docs/hp41-time-functions.json, helpEntriesTime() accessor + 4-pool helpEntriesAll(), Time Pac section in HelpOverlay.tsx, and vitest drift-catch + section rendering tests — all 180 tests pass.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | help_data.ts 4th Vite import + helpEntriesTime() + 4-pool helpEntriesAll() | 6e2f86a | hp41-gui/src/help_data.ts |
| 2 | HelpOverlay 4th section + vitest extensions | 1b50fca | hp41-gui/src/HelpOverlay.tsx, hp41-gui/src/HelpOverlay.test.tsx |

## What Was Built

### Task 1 — help_data.ts

- Added `import timeFunctions from '../../docs/hp41-time-functions.json'` at line 20 (4th Vite static JSON import, D-carried.8)
- Added `helpEntriesTime()` accessor returning `timeFunctions as readonly HelpEntry[]`, with doc comment referencing Phase 41 Plan 41-02, D-carried.8, 35-entry count per D-39.9, and Vite hard-build-blocker semantics per D-25.17
- Updated `helpEntriesAll()` from 3-pool to 4-pool: `[...helpEntries(), ...helpEntriesMath1(), ...helpEntriesStat1(), ...helpEntriesTime()]`
- Updated doc comment from "3-pool" to "4-pool" referencing Phase 41 / Phase 39 D-39.12 fourth OnceLock parallel
- TypeScript compiles clean (`npx tsc --noEmit` exits 0)

### Task 2 — HelpOverlay.tsx + HelpOverlay.test.tsx

**HelpOverlay.tsx:**
- Widened `SectionDef.id` union from `'hp41cv' | 'math1' | 'stat1'` to `'hp41cv' | 'math1' | 'stat1' | 'time'`
- Added 4th `SECTIONS` entry: `{ id: 'time', heading: 'Time Pac (XROM 26)', predicate: (e) => e.xrom?.module === 'Time' }`
- Widened `expanded` state type to include `time: boolean`, initial value `time: true`, `useEffect` reset includes `time: true`
- Widened `toggleSection` parameter type to include `'time'`
- No JSX changes needed — `sectionGroups.map()` render loop handles 4th section automatically

**HelpOverlay.test.tsx:**
- Added `import { helpEntriesTime }` and `import timeJson from '../../docs/hp41-time-functions.json'`
- Added drift-catch test: `helpEntriesTime().length === timeJson.length` and `>= 35`
- Added xrom field test: `module === 'Time'`, `module_id === 26`
- Updated `helpEntriesAll` test from 3-pool to 4-pool sum including `helpEntriesTime().length`
- Updated `sectionButtons.length` assertion from `.toBe(3)` to `.toBe(4)`
- Updated 2-section heading test to check all 4 sections
- Added Time Pac section rendering test (4 headings present)
- Added Time Pac category heading test (checks for "time clock", "time stopwatch", etc.)
- Added Time Pac toggle aria-expanded test (true → false → true cycle)
- Added `search for "SETIME" returns Time entries` test

## Verification

All plan verification criteria confirmed:

```
grep -c "helpEntriesTime" hp41-gui/src/help_data.ts  => 2 (definition + usage)
grep "Time Pac (XROM 26)" hp41-gui/src/HelpOverlay.tsx  => heading: 'Time Pac (XROM 26)'
grep "toBe(4)" hp41-gui/src/HelpOverlay.test.tsx  => 1 match
npx tsc --noEmit  => exits 0
vitest run  => 180 tests passed (5 test files)
```

## Deviations from Plan

**npm install deviation (Rule 3 — blocking issue):**
- **Found during:** Task 2 verification step
- **Issue:** Worktree's hp41-gui/ directory had no `node_modules/` installed; `vitest` binary not available
- **Fix:** Ran `npm install` in the worktree's hp41-gui/ directory — no new packages installed, only populated the existing `package.json` lockfile dependencies in the worktree filesystem
- **Impact:** None (standard setup step; all existing package.json deps, no new packages added)
- **Commits:** N/A (node_modules not committed per .gitignore)

No other deviations — plan executed as written.

## Known Stubs

None. All 35 Time Pac entries from `docs/hp41-time-functions.json` are wired into the overlay via `helpEntriesTime()`. The predicate `e.xrom?.module === 'Time'` correctly filters them into the "Time Pac (XROM 26)" section.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. All changes are frontend-only Vite build-time JSON imports and React component type extensions. The Vite hard-build-blocker (malformed JSON fails the build) is the only security surface, consistent with the existing pattern for the three prior JSON imports.

## Self-Check

### Created Files
- `.planning/phases/41-hp41-gui-gui-integration-live-display/41-02-SUMMARY.md` — this file

### Modified Files
- `hp41-gui/src/help_data.ts` — confirmed present
- `hp41-gui/src/HelpOverlay.tsx` — confirmed present
- `hp41-gui/src/HelpOverlay.test.tsx` — confirmed present

### Commits
- `6e2f86a` — feat(41-02): add helpEntriesTime() + 4-pool helpEntriesAll() to help_data.ts
- `1b50fca` — feat(41-02): add Time Pac 4th section to HelpOverlay + vitest extensions

## Self-Check: PASSED
