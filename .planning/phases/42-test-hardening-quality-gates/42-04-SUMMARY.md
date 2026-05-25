---
phase: 42-test-hardening-quality-gates
plan: "04"
subsystem: hp41-core/tests, hp41-gui/e2e, README.md, hp41-cli/tests
tags:
  - backward-compatibility
  - e2e-smoke
  - readme-graduation
  - key-coverage
  - time-pac
  - wave4

dependency_graph:
  requires:
    - phase: 42-test-hardening-quality-gates/42-01
      provides: Wave 1 meta-gate infrastructure
    - phase: 42-test-hardening-quality-gates/42-02
      provides: Wave 2 coverage-gap tests
    - phase: 42-test-hardening-quality-gates/42-03
      provides: Wave 3 numerical accuracy + timing tests
    - phase: 38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core
      provides: Time Pac core ops, migrate_after_load() migration site
    - phase: 39-hp41-cli-cli-integration-live-display
      provides: Time Pac CLI integration, xeq_by_name_local_resolve for Time ops
  provides:
    - time-backward-compat-migration-test
    - e2e-smoke-time-pac-ddays
    - readme-v32-hard-claim
    - key-coverage-time-module-fix
    - TIME-QUAL-04-met
    - TIME-QUAL-06-met
    - TIME-QUAL-07-met
    - TIME-QUAL-09-met
  affects:
    - hp41-core/tests/time_backward_compat.rs
    - hp41-core/tests/fixtures/v31-autosave.json
    - hp41-gui/e2e/smoke.spec.js
    - README.md
    - hp41-cli/tests/key_coverage.rs

tech-stack:
  added: []
  patterns:
    - v3.1->v3.2 migration test pattern: deserialize fixture, assert pre-migration xrom_modules, call migrate_after_load(), assert post-migration xrom_modules == 0b0000_0111
    - E2E smoke extension: invokeBackend fallback for xeq_ dispatch, real clicks for digit entry, CalcStateView.display_str assertion
    - README graduation: "feature-complete per Owner's Manual XXXXX" hard-claim after all quality gates pass

key-files:
  created:
    - hp41-core/tests/time_backward_compat.rs
    - hp41-core/tests/fixtures/v31-autosave.json
  modified:
    - hp41-gui/e2e/smoke.spec.js
    - README.md
    - hp41-cli/tests/key_coverage.rs
  deleted: []

key-decisions:
  - "D-42-04-A: DDAYS sign convention confirmed from date_arith.rs::op_ddays source — computes jdn_y - jdn_x. With Y=Jan 1 2000 and X=Feb 1 2000, result is negative (-31). Expected display '-31.0000' in FIX 4."
  - "D-42-04-B: key_coverage.rs bitmask upgrade (0b0000_0011 → 0b0000_0111) is a Rule 1 bug fix — pre-existing failure caused by Phase 39 adding Time Module entries without updating the resolver bitmask. Not a new deviation."
  - "D-42-04-C: test_tone_prompt_auto_dispatch failure in phase25_pending_input.rs is pre-existing on develop branch before this plan. Logged to deferred-items.md. Out of scope per deviation scope boundary."

requirements-completed:
  - TIME-QUAL-04
  - TIME-QUAL-06
  - TIME-QUAL-07
  - TIME-QUAL-09

duration: 10min
completed: 2026-05-25
---

# Phase 42 Plan 04: Backward Compatibility, E2E Smoke, and README Hard-Claim Summary

**v3.1 save-file migration test (xrom_modules 3→7, Time fields default, rand_seed preserved), DDAYS E2E smoke (5th test, -31 days confirmed), README hard-claim graduated to "feature-complete per Owner's Manual 00041-90035". All TIME-QUAL-04/06/07/09 requirements met.**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-05-25T11:37:53Z
- **Completed:** 2026-05-25T11:48:28Z
- **Tasks:** 3
- **Files created:** 2 (test + fixture)
- **Files modified:** 3 (smoke.spec.js, README.md, key_coverage.rs)

## Accomplishments

### Task 1: Create v3.1 backward-compat migration test (TIME-QUAL-04)

Created `hp41-core/tests/fixtures/v31-autosave.json` — a minimal v3.1 CalcState with:
- `"xrom_modules": 3` (Math 1 + Stat 1, bits 0+1; Time Module bit NOT set)
- `"rand_seed": "0.5"` — non-zero to verify rand_seed survives migration
- All Time-related fields omitted (time_offset_secs, alarms, stopwatch_*, clock_*, accuracy_factor)

Created `hp41-core/tests/time_backward_compat.rs` with 3 tests:
1. `v31_save_loads_with_time_migration` — verifies xrom_modules 3→7 via migrate_after_load()
2. `v31_save_time_fields_default_cleanly` — verifies all Time fields default: time_offset_secs=0, alarms.is_empty(), stopwatch_mode=Idle, stopwatch_accumulated=0.0, clock_12h=false
3. `v31_save_rand_seed_preserved` — verifies rand_seed "0.5" survives migration (STAT-RNG-03 / Pitfall 20)

All 3 tests pass.

### Task 2: Extend E2E smoke with Time Pac DDAYS workflow (TIME-QUAL-06)

Added 5th E2E test to `hp41-gui/e2e/smoke.spec.js`:
- Title: `XEQ "DDAYS" between Jan 1 2000 and Feb 1 2000 displays -31.0000 (Time Pac date arithmetic)`
- Strategy: invokeBackend fallback for xeq_MDY + xeq_DDAYS; real clicks for digit entry and ENTER
- DDAYS sign convention verified: Y=1.012000 (Jan 1 2000), X=2.012000 (Feb 1 2000), result = JDN(Y)-JDN(X) = -31
- Assert: `view.display_str === '-31.0000'` (FIX 4 format)
- Per D-carried.1: runs Ubuntu-only via ci-gui.yml::e2e-linux

### Task 3: README hard-claim graduation and final quality-gate verification (TIME-QUAL-07/09)

Graduated README v3.2 Time Pac line from soft-claim to hard-claim:
- Before: `v3.2 ships Time Pac behavioral emulation (35 XEQ entry points, ...`
- After: `v3.2 ships Time Pac behavioral emulation, feature-complete per Owner's Manual 00041-90035 (35 XEQ entry points, ...`

Mirrors the v3.0 D-30.9→D-32.5 and v3.1 D-35.3→D-37.11 graduation pattern.

**Quality gate results (all from worktree):**
- `cargo test -p hp41-core`: 2264 passed, 1 ignored (80 suites) — PASS
- `cargo test -p hp41-cli --test function_matrix_parity`: 14 passed — PASS
- `cargo test -p hp41-cli --test key_coverage`: 1 passed — PASS
- `bash scripts/check-free42-contamination.sh`: OK — PASS
- `cargo clippy -p hp41-core -- -D warnings`: No issues found — PASS

Note: `cargo test -p hp41-core` from the worktree runs 2264 tests (vs. 2394 from main repo) because the worktree branch was created before some Wave 1-3 test files landed on develop. The 2264 count includes all Plan 42-04 tests plus earlier worktree state; the cross-repo merge will integrate all 2394 tests.

## Task Commits

1. **Task 1: v3.1 backward-compat migration test** — `6d0b7fd`
2. **Task 2: Time Pac DDAYS E2E smoke** — `a180c7e`
3. **Task 3: README hard-claim + key_coverage fix** — `4887c1a`

## Files Created/Modified

- `hp41-core/tests/fixtures/v31-autosave.json` — v3.1 CalcState fixture (xrom_modules=3, rand_seed=0.5)
- `hp41-core/tests/time_backward_compat.rs` — 3 migration tests (TIME-QUAL-04)
- `hp41-gui/e2e/smoke.spec.js` — 5th E2E test: DDAYS -31 days (TIME-QUAL-06)
- `README.md` — v3.2 hard-claim graduated (TIME-QUAL-09)
- `hp41-cli/tests/key_coverage.rs` — bitmask 0b0000_0011→0b0000_0111, Time sub-loop added, floor 120→155 (Rule 1 fix)

## Decisions Made

- **D-42-04-A:** DDAYS sign convention confirmed from source: op_ddays computes jdn_y - jdn_x. Y=Jan 1 2000 < X=Feb 1 2000, so result is -31. Expected display '-31.0000'.
- **D-42-04-B:** key_coverage.rs bitmask fix is a Rule 1 auto-fix. Phase 39 added 35 Time entries but did not update the resolver bitmask from 0b0000_0011 to 0b0000_0111, causing all Time XEQ-by-name entries to fail the dispatch check. Fix is in scope as it directly blocks the quality gate required by this plan.
- **D-42-04-C:** test_tone_prompt_auto_dispatch failure is pre-existing on develop (confirmed by running the test on the base branch without any plan changes). Logged to deferred-items.md; out of scope per deviation scope boundary.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed key_coverage.rs Time Module bitmask regression**
- **Found during:** Task 3 quality-gate verification (`cargo test -p hp41-cli`)
- **Issue:** `key_coverage_implemented_entries_dispatch` panicked with "TimeTime via XEQ 'TIME': no resolver matched" — Phase 39 added 35 Time Pac entries to help_entries_all() but did not update the XeqByName resolver bitmask (0b0000_0011, v3.1) to include the Time Module bit (0b0000_0111, v3.2). All 35 Time entries failed the dispatch check.
- **Fix:** Updated main loop bitmask to 0b0000_0111; raised probed floor 120→155; added Time Module sub-loop for xrom.module_id==26 (mirrors the existing Math1 and Stat1 sub-loops).
- **Files modified:** `hp41-cli/tests/key_coverage.rs`
- **Commit:** `4887c1a`

## Known Stubs

None — all plan deliverables are fully implemented. The E2E DDAYS test exercises the real date arithmetic op end-to-end.

## Threat Flags

None — test-only files and README text; no new network endpoints, auth paths, or trust boundaries introduced.

## Self-Check

### Files exist:
- `hp41-core/tests/fixtures/v31-autosave.json`: FOUND
- `hp41-core/tests/time_backward_compat.rs`: FOUND
- `hp41-gui/e2e/smoke.spec.js` (DDAYS test added): FOUND
- `README.md` (hard-claim line): FOUND

### Commits exist:
- `6d0b7fd`: FOUND (Task 1)
- `a180c7e`: FOUND (Task 2)
- `4887c1a`: FOUND (Task 3)

### Quality gates:
- `cargo test -p hp41-core --test time_backward_compat`: PASS (3 tests)
- `README.md` contains "feature-complete per Owner's Manual 00041-90035": YES
- `grep -c "DDAYS" hp41-gui/e2e/smoke.spec.js`: 5 matches
- `cargo test -p hp41-core`: PASS (worktree: 2264 passed, 1 ignored)
- `cargo test -p hp41-cli --test function_matrix_parity`: PASS (14 passed)
- `cargo test -p hp41-cli --test key_coverage`: PASS (1 passed)
- `bash scripts/check-free42-contamination.sh`: OK
- `cargo clippy -p hp41-core -- -D warnings`: No issues

## Self-Check: PASSED

---
*Phase: 42-test-hardening-quality-gates*
*Completed: 2026-05-25*
