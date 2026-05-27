---
phase: 46-hp41-gui-gui-integration
plan: "02"
subsystem: hp41-gui
tags:
  - gui
  - help-overlay
  - advantage-pac
  - xrom-22
  - xrom-24
  - json-pipeline
dependency_graph:
  requires:
    - "44-01: docs/hp41-advantage-functions.json authored (5th JSON source-of-truth)"
  provides:
    - "help_data.ts: helpEntriesAdvantage() + 5-pool helpEntriesAll()"
    - "HelpOverlay.tsx: Advantage Pac (XROM 22) and Advantage Pac (XROM 24) sections"
  affects:
    - "hp41-gui/src/help_data.ts"
    - "hp41-gui/src/HelpOverlay.tsx"
tech_stack:
  added: []
  patterns:
    - "Vite static JSON import (fifth pool: hp41-advantage-functions.json)"
    - "React useState type widening (adv22 + adv24 boolean fields)"
key_files:
  created: []
  modified:
    - hp41-gui/src/help_data.ts
    - hp41-gui/src/HelpOverlay.tsx
decisions:
  - "xrom.module predicates use exact JSON field values: 'Adv Conv' (XROM 22) and 'Adv Math' (XROM 24) — NOT 'ADV 22A'/'ADV 24B' hardware display names"
  - "5-pool helpEntriesAll() updated in-place per Pitfall 5 (no parallel helpEntriesAll5() created)"
  - "Both Advantage sections default to expanded:true matching all prior sections"
metrics:
  duration_minutes: 3
  completed_date: "2026-05-26"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 2
---

# Phase 46 Plan 02: Help Overlay Advantage Pac Integration Summary

Fifth JSON pool wired into GUI help overlay: `helpEntriesAdvantage()` + 5-pool `helpEntriesAll()` in `help_data.ts`; two new collapsible sections "Advantage Pac (XROM 22)" and "Advantage Pac (XROM 24)" in `HelpOverlay.tsx` with correct `xrom.module` predicates ("Adv Conv" / "Adv Math").

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add fifth JSON import and 5-pool helpEntriesAll() to help_data.ts | a2ff64a | hp41-gui/src/help_data.ts |
| 2 | Add Advantage Pac sections to HelpOverlay.tsx | ef599b0 | hp41-gui/src/HelpOverlay.tsx |

## Changes Made

### Task 1: help_data.ts (commit a2ff64a)

- Added `import advantageFunctions from '../../docs/hp41-advantage-functions.json'` as fifth Vite static JSON import
- Added `helpEntriesAdvantage()` accessor function (114 entries, 7-category convention per D-44.1) with doc comment referencing Phase 46 Plan 46-02 and D-44.1
- Updated `helpEntriesAll()` from 4-pool to 5-pool concatenation: `[...helpEntries(), ...helpEntriesMath1(), ...helpEntriesStat1(), ...helpEntriesTime(), ...helpEntriesAdvantage()]`
- Updated JSDoc comments on file header + `helpEntriesAll()` to reference 5-pool chain and Advantage Pac addition
- Hard-build-blocker semantics preserved per D-25.17

### Task 2: HelpOverlay.tsx (commit ef599b0)

- Widened `SectionDef.id` union: `'hp41cv' | 'math1' | 'stat1' | 'time' | 'adv22' | 'adv24'`
- Added two new SECTIONS array entries:
  - `{ id: 'adv22', heading: 'Advantage Pac (XROM 22)', predicate: e.xrom?.module === 'Adv Conv' }`
  - `{ id: 'adv24', heading: 'Advantage Pac (XROM 24)', predicate: e.xrom?.module === 'Adv Math' }`
- Widened `expanded` useState type to include `adv22: boolean; adv24: boolean` (both default `true`)
- Updated `useEffect` reset to include `adv22: true, adv24: true`
- Widened `toggleSection` parameter: `'hp41cv' | 'math1' | 'stat1' | 'time' | 'adv22' | 'adv24'`
- Updated component header comment to note Phase 46 Plan 46-02 fifth+sixth sections

## Verification Results

1. `npx tsc --noEmit` — PASSED (TypeScript compiles without errors)
2. `grep -c "Advantage" hp41-gui/src/HelpOverlay.tsx` — returns 9 (≥ 2 required)
3. `grep -c "advantage" hp41-gui/src/help_data.ts` — returns 5 (≥ 3 required)
4. SECTIONS array has exactly 6 entries: hp41cv, math1, stat1, time, adv22, adv24
5. Vite build (via `npx tsc --noEmit`) confirms JSON import path is valid

## Requirements Satisfied

- **ADV-GUI-02**: HelpOverlay displays "Advantage Pac (XROM 22)" and "Advantage Pac (XROM 24)" sections; search spans all five JSON pools

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. All 114 Advantage Pac entries (63 XROM 22 + 51 XROM 24) are wired from the canonical JSON source authored in Phase 44 Plan 44-01.

## Threat Surface Scan

No new security-relevant surface introduced. Changes are build-time JSON import + frontend rendering only:
- JSON file is checked into git; Vite validates at build time; no runtime fetch (T-46-04 per plan threat model — accepted)
- All displayed data is public HP-41 documentation (T-46-05 — accepted)
- No new npm packages added (T-46-SC — mitigated by plan design)

## Self-Check: PASSED

- `hp41-gui/src/help_data.ts` — modified, confirmed contains `advantageFunctions` import + `helpEntriesAdvantage()` + 5-pool `helpEntriesAll()`
- `hp41-gui/src/HelpOverlay.tsx` — modified, confirmed contains `adv22`/`adv24` section IDs + "Advantage Pac (XROM 22/24)" headings
- Commit `a2ff64a` — verified via `git log`
- Commit `ef599b0` — verified via `git log`
