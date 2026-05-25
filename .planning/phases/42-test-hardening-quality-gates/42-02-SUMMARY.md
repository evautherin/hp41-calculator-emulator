---
phase: 42-test-hardening-quality-gates
plan: "02"
subsystem: hp41-core/tests
tags:
  - coverage-gap
  - time-pac
  - test-supplement
  - wave2
dependency_graph:
  requires:
    - phase: 42-test-hardening-quality-gates/42-01
      provides: Wave 1 meta-gate infrastructure (xrom_op_test_count.rs, lint_xrom_assertions.rs)
    - phase: 38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core
      provides: Time Pac core ops (35 Op variants)
  provides:
    - time-coverage-gap-closure
    - time-op-test-count-supplement
    - TIME-QUAL-01-met
  affects:
    - hp41-core/tests/time_coverage_supplement.rs

tech-stack:
  added: []
  patterns:
    - All 7 time/*.rs source files measured before writing any tests (D-carried.8 precedent)
    - Coverage already >= 90% for all files; minimal supplementary tests per plan spec
    - 79 integration tests covering 30 Time Op variants below Pitfall 16 threshold
    - Imports via hp41_core::ops::time::{AlarmEntry, AlarmType, ClockDisplayMode, StopwatchMode}

key-files:
  created:
    - hp41-core/tests/time_coverage_supplement.rs
  modified: []
  deleted: []

key-decisions:
  - "D-42-02-A: All 7 time/*.rs files already exceeded 90% region coverage (alarm.rs 96.50%, others 98%+); created supplementary test file per plan spec branch 'if coverage is already >= 90% for all files'"
  - "D-42-02-B: Aggregate hp41-core region coverage improved from 95.75% baseline to 96.01% after 79 new tests — TIME-QUAL-01 met with 3% margin above 93% gate"
  - "D-42-02-C: Used hp41_core::ops::time re-exports (AlarmEntry, AlarmType, StopwatchMode, ClockDisplayMode) rather than hp41_core::state direct path — state.rs only re-imports privately"

requirements-completed:
  - TIME-QUAL-01

duration: 10min
completed: 2026-05-25
---

# Phase 42 Plan 02: Wave 2 Coverage-Gap Tests for Time Module Summary

**All 7 time/*.rs source files already exceed 90% region coverage; 79 supplementary integration tests added for 30 Time Op variants below Pitfall 16 threshold; aggregate hp41-core region coverage 96.01% (>= 93% gate met)**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-05-25T11:13:51Z
- **Completed:** 2026-05-25T11:24:43Z
- **Tasks:** 2
- **Files created:** 1 (79 tests)

## Accomplishments

### Task 1: Per-file region coverage measurement (no code changes)

Coverage measured for all 7 time/*.rs source files after running lib tests + all integration tests:

| File | Lines uncovered | Functions uncovered | Regions uncovered | Region % |
|------|----------------|---------------------|-------------------|----------|
| alarm.rs | 61 / 1284 | 2 / 74 | 30 / 858 | 96.50% |
| alpha_time.rs | 6 / 488 | 0 / 38 | 3 / 256 | 98.83% |
| clock.rs | 4 / 800 | 0 / 62 | 1 / 492 | 99.80% |
| date_arith.rs | 17 / 863 | 0 / 54 | 2 / 387 | 99.48% |
| modal.rs | 4 / 394 | 0 / 21 | 1 / 237 | 99.58% |
| stopwatch.rs | 9 / 598 | 0 / 43 | 1 / 343 | 99.71% |
| mod.rs | (100% — no coverable lines shown) | — | — | 100% |

**All files already exceed 90% region coverage.**

Aggregate hp41-core region coverage baseline: **95.75%** (>= 93% gate met before any new tests).

Uncovered branches identified (for targeted test writing):
- `alarm.rs` lines 120-129: `epoch_secs_to_date` negative timestamp (pre-1970) branch
- `alarm.rs` line 147: `make_date_hpnum` DMY mode (dmy=true) branch
- `alarm.rs` lines 164, 171: `repeat_hpnum_to_secs` no-decimal and invalid-minutes branches
- `alarm.rs` lines 224-225: `op_xyzalm` Y=0 (today's date) branch
- `alarm.rs` line 355: `op_almcat` Control alarm type display
- `alarm.rs` line 388: `op_almnow` no past-due but has upcoming alarm
- `alpha_time.rs` lines 64-66: negative Unix timestamp in `decompose_unix_secs`
- `clock.rs` line 66: negative epoch in `decompose_epoch_secs`
- `date_arith.rs` lines 95, 115: no-decimal-point and out-of-range day branches
- Additional test code unreachable branches (panic! arms in match — not a coverage gap)

### Task 2: Create time_coverage_supplement.rs (79 tests)

Created `hp41-core/tests/time_coverage_supplement.rs` with:
- Header documenting all 7 files meet the 90% threshold (per plan spec)
- 79 targeted tests covering 30 Time Op variants below the 5-test Pitfall 16 threshold
- Covers previously uncovered production branches in alarm.rs
- Free42 disclaim header per time module convention

**Time Op variants now at or approaching 5-test threshold:**

| Op | Tests added in this file | Branch covered |
|----|--------------------------|----------------|
| op_time | 2 | format, lift effect |
| op_date | 3 | MDY/DMY modes, lift effect |
| op_setime | 4 | modal open, prompt, idempotent, dispatch |
| op_setdate | 4 | modal open, prompt, consecutive, dispatch |
| op_clk12 | 4 | set, idempotent, no-stack, toggle |
| op_clk24 | 4 | clear, idempotent, no-stack, toggle |
| op_clkt | 2 | TimeAndDate→TimeOnly step-down, Off→TimeOnly |
| op_clktd | 2 | TimeOnly→TimeAndDate step-up, Off→TimeAndDate |
| op_clock | 3 | sets mode, idempotent, no-stack |
| op_correct | 3 | no-op ok, no state change, lift neutral |
| op_tplusx | 1 | positive delta adds seconds |
| op_ddays | 1 | same-date returns zero |
| op_dow | 2 | Saturday (DOW=6), Monday (DOW=1) |
| op_dmy | 3 | sets flag 31, idempotent, no-stack |
| op_mdy | 3 | clears flag 31, idempotent, round-trip |
| op_atime | 2 | 12h AM/PM present, 24h no AM/PM |
| op_atime24 | 2 | no AM/PM regardless of clock_12h, colon format |
| op_adate | 1 | DMY slash format |
| op_rclsw | 3 | idle→zero, lift enable, after setsw |
| op_setsw | 2 | zero accumulated, mode=Stopped |
| op_sw | 4 | sets mode, clears clock_active, idempotent, no-stack |
| op_swpt | 2 | no-split→zero, after stpw |
| op_stpw | 2 | zero split when idle, stopwatch keeps running |
| op_almcat | 2 | empty catalog (no print), control alarm shows label |
| op_almnow | 2 | empty catalog no-op, upcoming alarm triggered |
| op_rclaf | 3 | default zero, recalls setaf value, lift enable |
| op_setaf | 3 | stores x, overwrites, y unchanged |
| op_clalma | 1 | no match returns error |
| op_clalmx | 2 | removes first of two, negative x error |
| op_clralms | 2 | empties full catalog, empty catalog ok |

**Aggregate hp41-core region coverage after new tests: 96.01%** (up from 95.75% baseline).

## Task Commits

1. **Task 1: Measure per-file region coverage** — no commit (measurement only, no code changes)
2. **Task 2: Create time_coverage_supplement.rs** — `b3c30e1` (test)

## Files Created/Modified

- `hp41-core/tests/time_coverage_supplement.rs` — 79 supplementary tests for Time Pac coverage gap closure

## Decisions Made

- **D-42-02-A:** All 7 time/*.rs files already exceeded 90% region coverage at Wave 2 measurement time. The plan spec says: "If coverage is already >= 90% for all files, this task creates a minimal file with a comment explaining all files meet the threshold and a few supplementary edge-case tests for the lowest-coverage file." Applied this branch while also adding tests for all 30 Op variants below the Pitfall 16 threshold (combining both objectives).
- **D-42-02-B:** Aggregate 96.01% region coverage exceeds the 93% TIME-QUAL-01 gate by a 3% margin. Denominator dilution from ~4636 Time Pac LOC is more than offset by the well-covered existing inline tests.
- **D-42-02-C:** `AlarmEntry`, `AlarmType`, `StopwatchMode`, `ClockDisplayMode` are not exported via `hp41_core::state` — they live in `hp41_core::ops::time::{alarm, clock, stopwatch}` and are re-exported from `hp41_core::ops::time`. Used the correct path.

## Deviations from Plan

None — plan executed exactly as written. The "if coverage >= 90% for all files" branch was taken (per plan Task 2 instructions), and tests were written targeting the lowest-coverage file (alarm.rs) while also covering all 30 Op variants below threshold.

## Known Stubs

None — this plan creates test infrastructure only; no user-facing stubs.

## Self-Check

### Files exist:
- `hp41-core/tests/time_coverage_supplement.rs`: FOUND

### Commits exist:
- `b3c30e1`: FOUND (Task 2)

### Quality gates:
- `cargo test -p hp41-core --test time_coverage_supplement`: PASS (79 tests)
- Aggregate hp41-core region coverage: 96.01% (>= 93% TIME-QUAL-01 gate)
- All 7 time/*.rs files >= 90% region coverage: PASS (lowest is alarm.rs at 97.67%)
- `cargo test -p hp41-core --test xrom_shadowing`: PASS (unchanged)

## Self-Check: PASSED

## Next Phase Readiness

- Wave 2 coverage-gap tests complete and committed
- When merged with Wave 1 (xrom_op_test_count.rs), the unified meta-gate will scan
  `time_coverage_supplement.rs` automatically and count test mentions for all 30 Time variants
- Wave 3 (Plan 42-03) will add numerical accuracy oracle cases for Time Pac
- Wave 4 (Plan 42-04) will add backward compatibility test + E2E smoke + README hard-claim

---
*Phase: 42-test-hardening-quality-gates*
*Completed: 2026-05-25*
