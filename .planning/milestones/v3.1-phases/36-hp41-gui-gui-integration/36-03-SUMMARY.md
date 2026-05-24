---
phase: 36-hp41-gui-gui-integration
plan: "03"
subsystem: hp41-gui/tests
tags:
  - vitest
  - help-overlay
  - stat1-pac
  - test-coverage
  - stat-gui-03
  - stat-gui-04
dependency_graph:
  requires:
    - 36-01  # prgm_display Stat 1 arms (4-way invariant item 4)
    - 36-02  # helpEntriesStat1 + helpEntriesAll 3-pool + HelpOverlay 3rd section
  provides:
    - vitest coverage for Stat 1 section rendering and data-layer integrity
  affects:
    - hp41-gui/src/HelpOverlay.test.tsx
tech_stack:
  added: []
  patterns:
    - drift-catch test pattern (JSON entry count equality assertion)
    - xrom-field loop assertion (module + module_id per entry)
    - aria-expanded toggle test via fireEvent.click
    - substring search result assertion via fireEvent.change
key_files:
  created: []
  modified:
    - hp41-gui/src/HelpOverlay.test.tsx
decisions:
  - "sectionButtons.length assertion updated from 2 to 3 (Pitfall 2 in plan — critical existing test break)"
  - "helpEntriesAll 3-partition loop mirrors the Rust help_entries_all() 3-pool structure"
  - "stat1 category heading check uses OR across univariate/distributions/anova (flexible, category-name-agnostic)"
metrics:
  duration: "~10 minutes"
  completed: "2026-05-24"
  tasks_completed: 1
  tasks_total: 1
  files_changed: 1
---

# Phase 36 Plan 03: HelpOverlay vitest Stat 1 Extension Summary

Vitest suite for `HelpOverlay.test.tsx` extended with Stat 1 Pac data-layer assertions and component section tests. The update closes CI test coverage for STAT-GUI-03 (Stat 1 Pac overlay third section) and STAT-GUI-04 (help_data.ts 3-pool data integrity).

## Objective

Extend `hp41-gui/src/HelpOverlay.test.tsx` with tests that assert:

1. `helpEntriesStat1()` returns 26 entries from `docs/hp41-stat1-functions.json` (drift-catch)
2. All Stat 1 entries carry `xrom.module === 'Stat 1'` and `xrom.module_id === 2`
3. `helpEntriesAll()` length equals the 3-pool sum (built-in + Math 1 + Stat 1)
4. The HelpOverlay renders all three section headings including `Stat 1 Pac (XROM 2)`
5. Stat 1 category headings appear under the third section
6. Clicking the Stat 1 section heading toggles `aria-expanded`
7. Searching `"NORMD"` returns at least one Stat 1 result row

## Tasks Completed

### Task 1: Extend HelpOverlay.test.tsx with Stat 1 data-layer tests + section tests

**Commit:** `e7c7b1d`

**Changes made:**

**Part A — Imports:**
- Added `helpEntriesStat1` to the destructured import from `./help_data`
- Added `import stat1Json from '../../docs/hp41-stat1-functions.json'`

**Part B — Data-layer tests added:**
- `helpEntriesStat1 returns all entries from docs/hp41-stat1-functions.json (drift-catch)` — asserts `.length === stat1Json.length` and `>= 26`
- `helpEntriesStat1 entries all have xrom field with module "Stat 1"` — loops all entries asserting `xrom.module === 'Stat 1'` and `xrom.module_id === 2`

**Part C — Updated existing test (critical):**
- `helpEntriesAll returns concatenation of built-in + Math 1 + Stat 1 entries` (renamed from 2-pool)
- Length assertion updated to include `helpEntriesStat1().length`
- 3-partition loop: indices `[0..cv)` no xrom, `[cv..cv+math1)` module === 'Math 1', `[cv+math1..end)` module === 'Stat 1'

**Part D — Critical existing test update (Pitfall 2):**
- `sectionButtons.length` assertion changed from `.toBe(2)` to `.toBe(3)` (line 227 equivalent)

**Part E — New Stat 1 section tests:**
- `renders three top-level sections including Stat 1 Pac (XROM 2)` — asserts all three headings present
- `Stat 1 Pac section contains a Stat 1 category heading (D-34.1)` — checks univariate/distributions/anova
- `clicking Stat 1 Pac section heading toggles aria-expanded (D-31.8)` — true → false → true cycle
- `search for "NORMD" returns Stat 1 entries` — fireEvent.change + row text content assertion

**Verification:**
```
Tests  174 passed (174)    # cd hp41-gui && node_modules/.bin/vitest run
```
`just gui-ci` exited 0: TypeScript compile + Rust cargo test (60 unit + 17 integration tests) + release build + vitest 174 tests all green.

## Deviations from Plan

### node_modules symlink (Rule 3 — Auto-fix)
- **Found during:** Task 1 verification
- **Issue:** The worktree `hp41-gui/` directory had no `node_modules`; `npx vitest` resolved to the global node path which lacked `vite`, causing a startup error
- **Fix:** Created a temporary symlink `hp41-gui/node_modules -> {main-repo}/hp41-gui/node_modules` to run vitest; the symlink was subsequently replaced by real `node_modules` when `just gui-ci` executed `npm ci`
- **Impact:** No file changes; worktree is clean; node_modules are now properly installed in the worktree for future test runs
- **Files modified:** None (symlink removed and replaced by npm ci)

## Known Stubs

None — test file is pure assertions over production code already shipped in Plans 36-01 and 36-02.

## Threat Flags

No new security-relevant surface introduced. Test-only changes per plan threat model.

## Self-Check

### Created files exist:
- SUMMARY.md: FOUND (this file)

### Commits exist:
- `e7c7b1d` — test(36-03): extend HelpOverlay vitest suite with Stat 1 Pac section tests

### Test results:
- `cd hp41-gui && npm test` exits 0 — 174 tests pass (5 test files)
- `just gui-ci` exits 0 — full pipeline green

## Self-Check: PASSED
