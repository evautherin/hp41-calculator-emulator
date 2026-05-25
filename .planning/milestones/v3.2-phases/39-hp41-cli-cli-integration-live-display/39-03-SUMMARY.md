---
phase: 39-hp41-cli-cli-integration-live-display
plan: 03
subsystem: cli-tests
tags: [rust, integration-tests, time-pac, help-data, alarm, event-buffer, xrom]

# Dependency graph
requires:
  - phase: 39-01
    provides: help_entries_time(), help_entries_all() 4-pool chain, docs/hp41-time-functions.json
  - phase: 39-02
    provides: drain_event_buffer in app.rs, check_alarms call site
  - phase: 38
    provides: Op::TimeAlmnow, AlarmEntry/AlarmType, check_alarms in alarm.rs
provides:
  - hp41-cli/tests/phase39_help_data_time.rs (13 smoke tests for Time Pac JSON pipeline)
  - hp41-cli/tests/phase39_key_ref_includes_time.rs (3 right-panel exclusion guards)
  - hp41-cli/tests/phase39_alarm_drain.rs (5 behavioral tests for alarm event_buffer drain)
affects: [ci-quality-gate, TIME-CLI-01, TIME-CLI-02, TIME-CLI-03, TIME-CLI-04, TIME-CLI-07]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Phase 34 Stat 1 test pattern mirrored exactly for Time Pac — count==35, module_id==26, categories prefix 'Time '"
    - "Option C alarm drain strategy: test at core dispatch level (Op::TimeAlmnow + check_alarms direct call)"
    - "check_alarms tested via hp41_core::ops::time::alarm::check_alarms (not re-exported from time/mod.rs, accessed via full path)"

key-files:
  created:
    - hp41-cli/tests/phase39_help_data_time.rs
    - hp41-cli/tests/phase39_key_ref_includes_time.rs
    - hp41-cli/tests/phase39_alarm_drain.rs
  modified: []

key-decisions:
  - "alarm_drain tests use Option C (core dispatch) — App constructor requires terminal which is not available in CI integration tests; CalcState-level testing covers the behavioral contract completely"
  - "check_alarms accessed via hp41_core::ops::time::alarm::check_alarms full path (not re-exported at time/mod.rs level)"
  - "5 alarm drain tests (exceeds 3 minimum): adds control alarm xeq: prefix test and explicit drain mechanic test"
  - "13 help data smoke tests (12 required + 1 bonus cross-pool uniqueness guard)"

requirements-completed: [TIME-CLI-01, TIME-CLI-02, TIME-CLI-03, TIME-CLI-04, TIME-CLI-07]

# Metrics
duration: 10min
completed: 2026-05-25
---

# Phase 39 Plan 03: Integration Test Files for Time Pac CLI Summary

**21 new integration tests across 3 files verify the Time Pac JSON pipeline end-to-end and the alarm event_buffer drain behavioral contract**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-05-25T05:21:00Z
- **Completed:** 2026-05-25T05:31:49Z
- **Tasks:** 3
- **Files created:** 3

## Accomplishments

- Created `hp41-cli/tests/phase39_help_data_time.rs` with 13 smoke tests verifying: help_entries_time() loads (not empty), count==35, display_name/description present, all status=="implemented", module_id==26 for all, categories start with "Time ", function_ids dense 1..=35, key_paths in XEQ form, exactly 4 divergences (SETAF/RCLAF/CORRECT/SW), 4-pool chain >= 236 entries, overlay rows include Time headers, cross-pool op_variant uniqueness
- Created `hp41-cli/tests/phase39_key_ref_includes_time.rs` with 3 regression guards: TIME excluded from key_ref_entries(), CLKT excluded, v2.2 Add (+) still present — mirrors phase34_key_ref_includes_stat1.rs exactly for Time Pac
- Created `hp41-cli/tests/phase39_alarm_drain.rs` with 5 behavioral tests: ALMNOW dispatch populates event_buffer with alarm:message: prefix, prefix contains non-empty text (exact match), check_alarms() with trigger_unix=0 fires on past-due alarm, drain(..) clears buffer and yields all entries, control alarm produces alarm:xeq: prefix

## Task Commits

Each task was committed atomically:

1. **Task 1: phase39_help_data_time.rs** - `2c278fb` (test)
2. **Task 2: phase39_key_ref_includes_time.rs** - `9cb6565` (test)
3. **Task 3: phase39_alarm_drain.rs** - `fd58176` (test)

## Files Created

- `hp41-cli/tests/phase39_help_data_time.rs` - 13 smoke tests for Time Pac JSON pipeline (count, schema, categories, xrom, divergences, 4-pool chain, overlay integration, cross-pool uniqueness)
- `hp41-cli/tests/phase39_key_ref_includes_time.rs` - 3 right-panel exclusion regression guards (TIME, CLKT excluded; v2.2 + preserved)
- `hp41-cli/tests/phase39_alarm_drain.rs` - 5 behavioral tests for alarm event_buffer drain (alarm:message:, alarm:xeq:, check_alarms, drain mechanic)

## Decisions Made

- `App` requires a terminal for construction so alarm drain tests use Option C (CalcState-level + Op::TimeAlmnow dispatch) — covers the behavioral contract that `drain_event_buffer` in app.rs relies on
- `check_alarms` accessed via full path `hp41_core::ops::time::alarm::check_alarms` (public function but not re-exported at `time/mod.rs` level)
- 5 alarm drain tests instead of required 3 minimum — adds control alarm and drain mechanic coverage for defense in depth

## Deviations from Plan

None — plan executed exactly as written. The three test files mirror the Phase 34 pattern for Stat 1 Pac exactly. All 21 tests pass. The pre-existing `key_coverage_implemented_entries_dispatch` failure in `key_coverage.rs` is out of scope (existed at base commit before this plan; it tests XEQ resolver coverage for Time Pac entries that are wired in Plan 39-02 not Plan 39-03).

## Deferred Items

- `key_coverage.rs::key_coverage_implemented_entries_dispatch` failure: Time Pac XEQ resolver coverage gap — pre-existing at base commit (Plan 39-02's drain_event_buffer wiring task), out of scope for this plan. Logged to `deferred-items.md` for tracking.

## Known Stubs

None — test files contain no stub patterns. All tests verify real behavior via actual dispatch and OnceLock loading.

## Threat Flags

None — test-only plan; no trust boundaries crossed. Tests validate data shapes and counts only; no secrets or PII involved.

## Self-Check: PASSED

- FOUND: hp41-cli/tests/phase39_help_data_time.rs
- FOUND: hp41-cli/tests/phase39_key_ref_includes_time.rs
- FOUND: hp41-cli/tests/phase39_alarm_drain.rs
- FOUND: commit 2c278fb (Task 1 — help data smoke tests)
- FOUND: commit 9cb6565 (Task 2 — key ref exclusion tests)
- FOUND: commit fd58176 (Task 3 — alarm drain behavioral tests)
- 21 tests pass across all 3 files

---
*Phase: 39-hp41-cli-cli-integration-live-display*
*Completed: 2026-05-25*
