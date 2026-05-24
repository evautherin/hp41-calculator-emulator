---
phase: 38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core
plan: 03
subsystem: core
tags: [rust, hp41, time-module, xrom, system-clock, calendar, modal]

# Dependency graph
requires:
  - phase: 38-01
    provides: CalcState time fields (time_offset_secs, clock_12h, clock_display_mode,
      accuracy_factor, clock_active), ClockDisplayMode enum stub, modal infrastructure

provides:
  - Full SystemTime::now() integration with time_offset_secs offset per D-38.1/D-38.2
  - decompose_epoch_secs() pure-Rust Gregorian calendar arithmetic (Fliegel-Van Flandern)
  - All 13 clock/format/display ops: TIME, DATE, CLK12, CLK24, CLKT, CLKTD, CLOCK,
    SETIME, SETDATE, CORRECT (no-op), SETAF, RCLAF, T+X
  - get_clock_display_str() helper for frontend pull-on-redraw (D-carried.7)
  - 34 new tests covering calendar arithmetic corner cases and all op state transitions

affects:
  - 38-05 (CLI integration -- needs op_time, op_date, get_clock_display_str)
  - 38-06 (modal submit -- needs SETIME/SETDATE modal pattern established here)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Pure-Rust Gregorian arithmetic: Fliegel-Van Flandern JDN algorithm, no libc/chrono"
    - "SystemTime::now() + i64 offset: single offset field covers time and date adjustment"
    - "Clock display helper: get_clock_display_str() for pull-on-redraw frontend pattern"
    - "CORRECT as documented no-op: HP-41 analog correction circuit not emulatable"

key-files:
  created: []
  modified:
    - hp41-core/src/ops/time/clock.rs

key-decisions:
  - "D-carried.1 enforced: pure-Rust Gregorian arithmetic instead of libc::localtime_r
    (Fliegel-Van Flandern JDN formula, ~60 LOC, zero new runtime deps)"
  - "CORRECT is a documented no-op (TIME-FMT-05) -- plan spec overrides stub which stored
    X into accuracy_factor; SETAF is the correct store op"
  - "op_clkt toggles Off<->TimeOnly (not set-once); op_clktd toggles Off<->TimeAndDate;
    plan behavioral spec overrides naive stub implementations"
  - "op_clock sets TimeOnly+active=true (programmatic activation, not disable)"
  - "decompose_epoch_secs uses time_offset_secs as the combined timezone+user delta --
    no separate OS timezone query needed per D-38.2 design"

patterns-established:
  - "Clock read pattern: adjusted_epoch_secs(offset) -> decompose_epoch_secs() -> format"
  - "Toggle pattern: CLKT/CLKTD cycle through enum states rather than set-once"

requirements-completed:
  - TIME-CLK-01
  - TIME-CLK-02
  - TIME-CLK-06
  - TIME-DSP-01
  - TIME-DSP-02
  - TIME-DSP-03
  - TIME-DSP-04
  - TIME-DSP-05
  - TIME-FMT-01
  - TIME-FMT-02
  - TIME-FMT-03
  - TIME-FMT-04
  - TIME-FMT-05

# Metrics
duration: 35min
completed: 2026-05-24
---

# Phase 38 Plan 03: Clock/Format/Display Ops — Summary

**All 13 TIME module clock ops implemented with SystemTime::now() + i64 offset, pure-Rust
Fliegel-Van Flandern Gregorian calendar arithmetic, and toggle-based display mode transitions**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-05-24T20:30:00Z
- **Completed:** 2026-05-24T21:05:00Z
- **Tasks:** 2 (Tasks 1 and 2 implemented in one commit due to shared file)
- **Files modified:** 1

## Accomplishments

- Replaced Wave 1 stub implementations with full real-time clock logic: `SystemTime::now()`
  reads system time, adds `time_offset_secs` (D-38.2), decomposes via Fliegel-Van Flandern
  Gregorian algorithm (zero new runtime deps, D-carried.1)
- All 13 clock ops fully functional: TIME/DATE push HP-41 HH.MMSScc/date formats, CLK12/CLK24
  toggle 12h/24h, CLKT/CLKTD toggle display modes with correct cycling semantics, CLOCK sets
  TimeOnly mode, SETIME/SETDATE open modals, CORRECT is documented no-op, SETAF/RCLAF
  store/recall accuracy factor, T+X parses HH.MMSScc and adds seconds to offset
- `get_clock_display_str()` frontend helper supporting both 12h and 24h display with
  TimeAndDate alternation on even/odd seconds (D-carried.7 pull-on-redraw architecture)
- 34 new tests: Gregorian calendar corner cases (Unix epoch, leap year 2000-02-29, midnight
  boundary, known date 2026-05-24), all op state transitions, format helpers, round-trips

## Task Commits

1. **Task 1+2: All 13 clock ops + calendar arithmetic + tests** - `0108429` (feat)

## Files Created/Modified

- `hp41-core/src/ops/time/clock.rs` — Full implementation replacing Wave 1 stubs (677 lines
  added, 63 removed): decompose_epoch_secs(), adjusted_epoch_secs(), op_time, op_date,
  op_setime, op_setdate, op_clk12, op_clk24, op_clkt, op_clktd, op_clock, op_correct,
  op_setaf, op_rclaf, op_tplusx, get_clock_display_str(), format_time_str(), format_date_str()

## Decisions Made

- **Pure-Rust calendar arithmetic (D-carried.1):** The plan spec suggested `libc::localtime_r`
  but `libc` is not in hp41-core's Cargo.toml and D-carried.1 bans new runtime deps. Implemented
  Fliegel-Van Flandern JDN decomposition (~60 LOC) instead. Matches D-38.3 intent.
- **time_offset_secs encodes timezone delta:** SETIME will compute delta between entered time
  and SystemTime::now() at submit. This means time_offset_secs IS the timezone + user adjustment
  combined. No separate OS timezone query needed.
- **CORRECT as no-op:** Existing stub stored X into accuracy_factor. Plan spec (TIME-FMT-05)
  says CORRECT is a no-op (legacy HP-41 analog correction circuit). SETAF is the correct
  store op. Fixed per plan.
- **op_clock behavior:** Plan says "sets clock_active=true and clock_display_mode=TimeOnly"
  (not disable). Corrected from stub that set Off+false.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] op_correct stub incorrectly stored X into accuracy_factor**
- **Found during:** Task 1 (reading stub behavior against plan spec)
- **Issue:** Stub had `state.accuracy_factor = state.stack.x.clone()` but TIME-FMT-05
  documents CORRECT as a no-op (it adjusts an analog correction circuit not emulatable)
- **Fix:** op_correct now returns Ok(()) with LiftEffect::Neutral, no state modification
- **Files modified:** hp41-core/src/ops/time/clock.rs
- **Verification:** `op_correct_is_no_op_does_not_modify_accuracy_factor` test passes
- **Committed in:** 0108429

**2. [Rule 1 - Bug] op_clkt/op_clktd stubs lacked toggle semantics**
- **Found during:** Task 2 (reading plan behavioral spec)
- **Issue:** Stubs set mode unconditionally; plan spec requires cycling toggle behavior
- **Fix:** Implemented match-based cycling: Off->TimeOnly->Off (CLKT), Off->TimeAndDate->Off (CLKTD)
- **Files modified:** hp41-core/src/ops/time/clock.rs
- **Verification:** op_clkt and op_clktd toggle tests pass
- **Committed in:** 0108429

**3. [Rule 1 - Bug] op_clock stub set Off+active=false instead of TimeOnly+active=true**
- **Found during:** Task 2 (reading plan behavioral spec)
- **Issue:** Stub disabled clock display; plan says CLOCK enables TimeOnly mode
- **Fix:** op_clock sets clock_display_mode=TimeOnly, clock_active=true
- **Files modified:** hp41-core/src/ops/time/clock.rs
- **Verification:** op_clock_sets_time_only_mode_and_active test passes
- **Committed in:** 0108429

**4. [Rule 3 - Blocking] libc not available — used pure-Rust calendar arithmetic**
- **Found during:** Task 1 (planning implementation of get_local_time)
- **Issue:** Plan spec referenced `libc::localtime_r` but libc is not in hp41-core Cargo.toml
  and D-carried.1 prohibits new runtime deps
- **Fix:** Implemented Fliegel-Van Flandern JDN Gregorian decomposition in pure Rust
- **Files modified:** hp41-core/src/ops/time/clock.rs
- **Verification:** decompose_epoch_secs tests (Unix epoch, leap year, known date) all pass
- **Committed in:** 0108429

---

**Total deviations:** 4 auto-fixed (3 Rule 1 bugs, 1 Rule 3 blocking)
**Impact on plan:** All fixes necessary for correct HP-41 behavioral emulation and compilability.
No scope creep.

## Issues Encountered

- Pre-existing clippy warnings in `stopwatch.rs` (from Plan 38-01): `impl Default` that can be
  derived, and manual range contains. These are out-of-scope for this plan — documented here but
  not fixed (deviation scope boundary rule).

## Known Stubs

None — all 13 clock ops are fully implemented. get_clock_display_str() provides a functional
display helper. The SETIME/SETDATE modal submit logic (convert entered time to offset) is
intentionally deferred to Plan 38-06 (modal submit implementation); the modal openers in this
plan correctly wire to the modal infrastructure.

## Threat Flags

None — no new network endpoints, auth paths, or file access patterns introduced. T-38-09
(T+X invalid time input) mitigated via parse_time_hpnum validation.

## Self-Check

Verifying claimed outcomes:

- [ ] clock.rs exists and is modified: PASSED (git status confirms M)
- [ ] commit 0108429 exists: PASSED (git log)
- [ ] SystemTime::now() present: PASSED (grep matches line 94 + docs)
- [ ] time_offset_secs used: PASSED (23 occurrences in clock.rs)
- [ ] All 2128 hp41-core tests pass: PASSED (cargo test output)
- [ ] 45 clock-filter tests pass: PASSED (cargo test -- clock)

## Self-Check: PASSED

## Next Phase Readiness

- Plan 38-04 (date arithmetic: DATE+, DDAYS, DOW) can proceed independently — uses
  decompose_epoch_secs() and parse_date_hpnum() infrastructure now in place
- Plan 38-05 (CLI integration) can consume op_time, op_date, get_clock_display_str()
- Plan 38-06 (modal submit) completes the SETIME/SETDATE workflow started here

---
*Phase: 38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core*
*Completed: 2026-05-24*
