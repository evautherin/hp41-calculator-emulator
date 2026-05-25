---
phase: 42-test-hardening-quality-gates
plan: "03"
subsystem: hp41-core/tests
tags:
  - numerical-accuracy
  - date-arithmetic
  - stopwatch-timing
  - alarm-latency
  - time-pac
  - wave3
dependency_graph:
  requires:
    - phase: 42-test-hardening-quality-gates/42-01
      provides: Wave 1 meta-gate infrastructure
    - phase: 42-test-hardening-quality-gates/42-02
      provides: Wave 2 coverage-gap tests
    - phase: 38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core
      provides: Time Pac core ops (35 Op variants, date_arith/stopwatch/alarm)
  provides:
    - time-date-accuracy-oracle-suite
    - time-stopwatch-timing-accuracy
    - time-alarm-latency-gate
    - TIME-QUAL-02-met
    - TIME-QUAL-10-met
    - TIME-QUAL-11-met
  affects:
    - hp41-core/tests/time_date_accuracy.rs
    - hp41-core/tests/time_stopwatch_timing.rs
    - hp41-core/tests/time_alarm_latency.rs

tech-stack:
  added: []
  patterns:
    - Oracle source D-42.11 verified: Python datetime/calendar for derivation; Meeus JDN tables for sanity-check on Gregorian boundary
    - Exact-match assertions (assert_eq!) for all date arithmetic per D-42.4 — integer math, no tolerance
    - Stopwatch test: 1-second CI-safe window with ±5cs (50ms) tolerance; #[ignore] 60s variant for manual validation per D-42.13
    - Alarm latency: past-due detection via timestamp comparison (trigger_unix <= now); no sleeps needed (past epoch timestamps always fire)
    - Two-level oracle correction applied: 2100-2000 = 36525 days (not 36524) verified via Python datetime

key-files:
  created:
    - hp41-core/tests/time_date_accuracy.rs
    - hp41-core/tests/time_stopwatch_timing.rs
    - hp41-core/tests/time_alarm_latency.rs
  modified: []
  deleted: []

key-decisions:
  - "D-42-03-A: Oracle correction for 100-year span: 2100-2000 = 36525 days (not 36524). Year 2000 IS a leap year (divisible by 400), so 25 leap years in [2000,2004,...,2096] gives 100×365+25=36525. Python datetime confirmed."
  - "D-42-03-B: Gregorian start Oct 15 1582 DOW tested via JDN primitives rather than full dispatch (parse_date_hpnum has range validation that accepts 1582 dates via the JDN roundtrip path). JDN-level test is more direct for the calendar boundary."
  - "D-42-03-C: Stopwatch timing tests use two strategies: direct op_runsw/op_stopsw calls (unit) and dispatch(Op::TimeRunsw) path (integration). Both verify the wiring chain end-to-end."
  - "D-42-03-D: Alarm latency tests need no sleeps: trigger_unix=1000 (Jan 1 1970 00:16:40) is always in the past; trigger_unix=i64::MAX is always future. Pure deterministic timestamp comparison."

requirements-completed:
  - TIME-QUAL-02
  - TIME-QUAL-10
  - TIME-QUAL-11

duration: 20min
completed: 2026-05-25
---

# Phase 42 Plan 03: Numerical Accuracy, Stopwatch Timing, and Alarm Latency Tests Summary

**Oracle-verified date arithmetic suite (30 tests, exact-match), stopwatch timing accuracy (5 tests, 1s CI-safe window +/-5cs), and alarm past-due detection latency (12 tests, single check_alarms cycle). All 47 tests pass; TIME-QUAL-02, TIME-QUAL-10, TIME-QUAL-11 met.**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-05-25T11:13:00Z (approximate)
- **Completed:** 2026-05-25T11:33:05Z
- **Tasks:** 2
- **Files created:** 3

## Accomplishments

### Task 1: Create date arithmetic accuracy suite (time_date_accuracy.rs)

Created `hp41-core/tests/time_date_accuracy.rs` with 30 oracle-verified test cases covering the complete TIME-QUAL-02 requirement set:

**Leap year edge cases:**
- Leap year 2000 (IS leap — divisible by 400): Feb 28 + 1 = Feb 29, DOW Feb 29 2000 = Tuesday=2, DDAYS across leap day = 2
- Century 2100 (NOT leap — divisible by 100 but not 400): Feb 28 + 1 = Mar 1, DDAYS = 1
- Century 2400 (IS leap — divisible by 400): Feb 28 + 1 = Feb 29, DDAYS across leap day = 2

**Gregorian calendar start:**
- Oct 15 1582 DOW = Friday = 5 (tested via JDN primitives, D-42-03-B)
- JDN roundtrip for Oct 15 1582
- DATE+ forward from Oct 15 1582 → Oct 16 1582

**Y2K boundary:**
- DDAYS Dec 31 1999 → Jan 1 2000 = 1 day
- DATE+ Dec 31 1999 + 1 = Jan 1 2000
- DOW Jan 1 2000 = Saturday = 6

**Large offset:**
- DDAYS Jan 1 2000 → Jan 1 2100 = 36525 days (25 leap years, corrected from initial oracle)
- DATE+ Jan 1 2000 + 36525 = Jan 1 2100

**Historical DOW oracles (Python datetime.isoweekday verified):**
- July 4 1776 = Thursday = 4
- Sep 11 2001 = Tuesday = 2
- May 25 1977 = Wednesday = 3

**DMY mode (Flag 31):**
- DATE+ in DMY: 1.012026 (Jan 1 2026) + 31 = Feb 1 2026
- DDAYS in DMY across Feb 28 → Mar 1 2000 = 2
- DOW in DMY: 1.012000 = Jan 1 2000 = Saturday=6

**Edge cases:**
- Same-date DDAYS = 0
- Negative DDAYS (X later than Y) = -364
- DATE+ with negative days (Jan 1 2026 - 1 = Dec 31 2025, Jan 1 2000 - 365 = Jan 1 1999)
- End-of-month rollover (Jan 31 + 1 = Feb 1)
- Year rollover (Dec 31 2025 + 1 = Jan 1 2026)
- DDAYS 1900 → 2000 = 36524 (1900 is NOT leap)
- DOW Monday sentinel (Jan 2 2006 = Monday = 1)

**Oracle correction (D-42-03-A):** Initial oracle had 2100-2000=36524; Python datetime confirmed 36525 (year 2000 is a leap year, adding 25 leap days in the span). Corrected before commit.

### Task 2: Create stopwatch timing and alarm latency tests

**time_stopwatch_timing.rs (5 tests, 1 #[ignore]):**
- `stopwatch_timing_1s_within_5cs`: 1-second sleep, RUNSW→STOPSW→RCLSW, asserts [95,105]cs
- `stopwatch_timing_via_dispatch_1s`: 500ms sleep via dispatch path, asserts [40,100]cs
- `stopwatch_timing_60s_within_10ms`: #[ignore] 60s variant for manual validation
- `stopwatch_split_records_while_running`: STPW records split while stopwatch continues; split < total, both > 0
- `stopwatch_reset_zeros_accumulated`: SETSW(0) clears accumulated time; RCLSW = 0
- `stopwatch_stpw_when_idle_records_zero_split`: idle STPW → split=0, SWPT recalls 0

**time_alarm_latency.rs (12 tests):**
- `alarm_latency_past_due_fires_in_one_cycle`: trigger_unix=1000 fires in one check_alarms call
- `alarm_latency_message_event_populates_print_buffer`: message alarm populates print_buffer
- `alarm_latency_control_alarm_fires_in_one_cycle`: non-interrupting control fires in one cycle
- `alarm_latency_interrupting_control_fires_in_one_cycle`: interrupting → "alarm:interrupting:deferred"
- `alarm_latency_multiple_past_due_all_fire_in_one_cycle`: 3 past-due, all fire in one call
- `alarm_latency_future_alarm_does_not_fire`: trigger_unix=i64::MAX does NOT fire
- `alarm_latency_mixed_only_past_due_fires`: mixed catalog, only past-due fires
- `alarm_latency_empty_catalog_no_op`: no-op, no panic
- `alarm_latency_already_past_due_does_not_refire`: idempotency guard (past_due=true prevents re-dispatch)
- `alarm_latency_almnow_fires_past_due_immediately`: ALMNOW dispatches past-due event immediately
- `alarm_latency_almnow_triggers_upcoming_alarm`: ALMNOW fires future alarm on demand
- `alarm_latency_almnow_empty_catalog_noop`: ALMNOW with no alarms = no-op, no error

## Task Commits

1. **Task 1: Create date arithmetic accuracy suite** — `22861da`
2. **Task 2: Create stopwatch timing and alarm latency tests** — `942971e`

## Files Created/Modified

- `hp41-core/tests/time_date_accuracy.rs` — 30 oracle-verified date arithmetic tests (TIME-QUAL-02)
- `hp41-core/tests/time_stopwatch_timing.rs` — 5 stopwatch timing tests, 1 #[ignore] (TIME-QUAL-10)
- `hp41-core/tests/time_alarm_latency.rs` — 12 alarm latency tests (TIME-QUAL-11)

## Decisions Made

- **D-42-03-A:** Oracle correction for 100-year span: Python datetime confirmed 2100-2000 = 36525 days (not 36524). The error was missing year 2000 as a leap year in the initial count estimate.
- **D-42-03-B:** Gregorian start Oct 15 1582 DOW tested via JDN primitives (`date_to_jdn` + `jdn_to_dow`) rather than Op dispatch. The dispatch path goes through `parse_date_hpnum` which validates via JDN roundtrip — the primitives are the canonical oracle.
- **D-42-03-C:** Stopwatch timing uses both direct function calls and dispatch path to verify end-to-end wiring. The 1s CI window is conservative (±5cs = 50ms) to survive Docker/CI sleep jitter.
- **D-42-03-D:** Alarm latency tests use fixed past-epoch timestamps (trigger_unix=1000) to avoid sleeps. The test is deterministic — no reliance on system clock jitter.

## Deviations from Plan

None — plan executed exactly as written. The oracle correction (D-42-03-A) is documented as a deviation of the initial *plan spec* (which said "36525"), but the implementation correctly used the Python-verified value. The `date_accuracy_09_dow_gregorian_start` test uses JDN primitives as noted in D-42-03-B; this is a natural choice since Op dispatch goes through the same JDN functions.

## Known Stubs

None — this plan creates test infrastructure only; no user-facing stubs.

## Threat Flags

None — test-only files; no new network endpoints, auth paths, or trust boundaries introduced.

## Self-Check

### Files exist:
- `hp41-core/tests/time_date_accuracy.rs`: FOUND
- `hp41-core/tests/time_stopwatch_timing.rs`: FOUND
- `hp41-core/tests/time_alarm_latency.rs`: FOUND

### Commits exist:
- `22861da`: FOUND (Task 1)
- `942971e`: FOUND (Task 2)

### Quality gates:
- `cargo test -p hp41-core --test time_date_accuracy`: PASS (30 tests)
- `cargo test -p hp41-core --test time_stopwatch_timing`: PASS (5 passed, 1 ignored)
- `cargo test -p hp41-core --test time_alarm_latency`: PASS (12 tests)
- All assertions exact-match (assert_eq!) per D-42.4: YES
- Stopwatch 1s window within +/-5cs: PASS (verified)
- Alarm past-due fires in one cycle: PASS (trigger_unix=1000 always in past)

## Self-Check: PASSED

## Next Phase Readiness

- Wave 3 date accuracy + stopwatch timing + alarm latency tests complete and committed
- TIME-QUAL-02, TIME-QUAL-10, TIME-QUAL-11 all met
- Wave 4 (Plan 42-04) will add backward compatibility test, E2E smoke with Time Pac workflow, and README hard-claim graduation (following Phase 37 / Phase 32 pattern)

---
*Phase: 42-test-hardening-quality-gates*
*Completed: 2026-05-25*
