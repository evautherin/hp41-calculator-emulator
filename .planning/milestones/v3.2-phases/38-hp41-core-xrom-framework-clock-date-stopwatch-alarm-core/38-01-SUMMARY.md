---
phase: 38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core
plan: "01"
subsystem: hp41-core
tags: [time-module, xrom, scaffolding, serde, migration]
dependency_graph:
  requires:
    - 37-07  # Phase 37 test hardening (hp41-core baseline)
  provides:
    - TIME-FW-01  # Time Module (XROM 26) registered as bit-2 XROM module
    - TIME-FW-02  # v3.1 save file migration to v3.2 (xrom_modules 3→7)
    - TIME-FW-03  # 35 Op::Time* variants wired in dispatch() and execute_op()
    - TIME-FW-04  # CalcState 12 new fields (8 persistent + 4 transient) serde-correct
    - TIME-FW-05  # Free42 contamination guard extended to time/ directory
    - TIME-FW-06  # ModalProgram::Time(TimeStep) variant for clock/date/alarm modals
  affects:
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-core/src/ops/math1/xrom.rs
    - hp41-core/src/ops/math1/modal.rs
    - hp41-core/src/ops/math1/mod.rs
    - hp41-core/src/state.rs
    - scripts/check-free42-contamination.sh
tech_stack:
  added: []
  patterns:
    - "XROM bit-2 arm in xrom_resolve() (fires after bit-1 Stat 1, before Err)"
    - "D-38.6 stopwatch Running→Stopped freeze on deserialization in migrate_after_load()"
    - "Phase 38 intentional CI break: items 3+4 (CLI/GUI prgm_display.rs) deferred to Phase 39/41"
key_files:
  created:
    - hp41-core/src/ops/time/mod.rs
    - hp41-core/src/ops/time/clock.rs
    - hp41-core/src/ops/time/date_arith.rs
    - hp41-core/src/ops/time/alpha_time.rs
    - hp41-core/src/ops/time/stopwatch.rs
    - hp41-core/src/ops/time/alarm.rs
    - hp41-core/src/ops/time/modal.rs
  modified:
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-core/src/ops/math1/xrom.rs
    - hp41-core/src/ops/math1/modal.rs
    - hp41-core/src/ops/math1/mod.rs
    - hp41-core/src/state.rs
    - scripts/check-free42-contamination.sh
    - hp41-core/tests/stat1_backward_compat.rs
    - hp41-core/tests/v3_save_compat.rs
decisions:
  - "TIME_MODULE id=26 name='TIME 2C' — HP 82182A CATALOG 2 display string per plan spec"
  - "math1/mod.rs submit_modal_input needed Time arm (not in plan) — Rule 1 auto-fix (compile error)"
  - "parse_alarm_type marked #[allow(dead_code)] — Phase 38 stub, used by Wave 2 XYZALM implementation"
  - "backward compat tests in stat1_backward_compat.rs and v3_save_compat.rs updated to 0b0000_0111"
metrics:
  duration_minutes: 45
  completed: "2026-05-24T20:47:00Z"
  tasks_completed: 3
  files_created: 7
  files_modified: 10
---

# Phase 38 Plan 01: Time Module Framework Scaffolding Summary

Time Module (XROM 26) registered as third XROM application module — 35 stub Op variants wired through dispatch/execute_op, TIME_MODULE const with bit-2 resolver, CalcState extended with 12 new fields, Free42 guard extended to scan time/ directory.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1+2  | Time module directory + Op enum + XROM wiring | ad260d0 | 14 files (7 new time/*.rs + 7 modified) |
| 3    | CalcState serde + migration integration tests | 5c3c387 | 3 files (state.rs + 2 test files) |

## What Was Built

### New Module: `hp41-core/src/ops/time/`

7 Rust source files forming the Time Module (XROM 26) scaffold:

- **`mod.rs`** — module hub; `pub mod` declarations + re-exports of all types needed by state.rs and ops/mod.rs
- **`clock.rs`** — `ClockDisplayMode` enum (Off/TimeOnly/TimeAndDate) + 11 stub clock/display ops (TIME, DATE, SETIME, SETDATE, CLK12, CLK24, CLKT, CLKTD, CLOCK, CORRECT, T+X)
- **`date_arith.rs`** — `parse_date_hpnum`, `parse_time_hpnum` (LEFT-pad per P35 invariant), stub JDN helpers + 5 date arithmetic ops (DATE+, DDAYS, DOW, DMY, MDY)
- **`alpha_time.rs`** — 3 stub alpha ops (ATIME, ATIME24, ADATE) with 24-char ALPHA register limit
- **`stopwatch.rs`** — `StopwatchMode` enum (Idle/Running/Stopped), `secs_to_hpnum_time()` converter + 7 stopwatch ops (RUNSW, STOPSW, RCLSW, SETSW, SW, SWPT, STPW)
- **`alarm.rs`** — `AlarmEntry` struct + `AlarmType` enum (Message/Control), `parse_alarm_type()` helper + 8 alarm ops (XYZALM, ALMCAT, ALMNOW, RCLALM, RCLAF, SETAF, CLALMA, CLALMX, CLRALMS)
- **`modal.rs`** — `TimeStep` enum (SetTimePrompt/SetDatePrompt/XyzalmTimePrompt) + stub dispatch functions

### CalcState Extensions (12 new fields)

Persistent (8, `#[serde(default)]`):
- `time_offset_secs: i64` — clock offset from system time (seconds)
- `clock_12h: bool` — 12h/24h display mode (CLK12/CLK24)
- `clock_display_mode: ClockDisplayMode` — CLKT/CLKTD/CLOCK state
- `accuracy_factor: HpNum` — CORRECT accuracy factor
- `alarms: Vec<AlarmEntry>` — alarm registry
- `stopwatch_mode: StopwatchMode` — Idle/Running/Stopped
- `stopwatch_accumulated: f64` — elapsed time not in current run
- `stopwatch_split: f64` — SWPT split reference

Transient (4, `#[serde(default, skip)]`):
- `stopwatch_start: Option<std::time::Instant>` — current lap start
- `clock_active: bool` — continuous display active flag
- `stopwatch_keyboard_mode: bool` — SW keyboard mode
- `alarm_catalog_mode: bool` — ALMCAT mode flag

### XROM Registration

- `TIME_MODULE: XromModule` const (id=26, name="TIME 2C", 35 entries)
- `time_resolve(name: &str) -> Option<Op>` with 35 match arms
- Bit-2 arm in `xrom_resolve()` fires AFTER bit-1 (Stat 1) — Pitfall 1 + Pitfall 22 preserved
- 3 unit tests: `time_module_const_id_and_name`, `time_module_ops_has_correct_entry_count`, `resolve_uses_bit_2_for_time_module`

### Migration

- `default_xrom_modules()` updated: `0b0000_0011` → `0b0000_0111`
- `migrate_after_load()` extended: bit-2 arm (v3.1→v3.2) + D-38.6 stopwatch Running→Stopped freeze

### Integration Tests (4 new + 6 updated)

New: `time_fields_serde_round_trip`, `time_v31_save_migration`, `time_stopwatch_running_freeze_on_load`, `default_xrom_modules_returns_0b111`

Updated: `xrom_modules_default_is_seven`, `default_construction_phase28_fields`, `v3_0_save_loads_with_stat_1_after_migration`, `migrate_after_load_idempotent`, `loads_synthetic_v22_save_without_v3_fields`, `v30_save_loads_with_stat1_migration`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Missing `ModalProgram::Time` arm in `math1/mod.rs`**
- **Found during:** Task 2 compilation
- **Issue:** `submit_modal_input()` in `math1/mod.rs` had an exhaustive match over `ModalProgram` variants that didn't cover the newly-added `Time(_)` arm — compile error
- **Fix:** Added `ModalProgram::Time(step) => crate::ops::time::modal::submit_step(state, step)` arm
- **Files modified:** `hp41-core/src/ops/math1/mod.rs`
- **Commit:** ad260d0 (included in Task 1+2 commit)

**2. [Rule 1 - Bug] `parse_alarm_type` dead_code warning**
- **Found during:** Task 1 cargo check
- **Issue:** `parse_alarm_type()` private helper in alarm.rs triggered `dead_code` warning since no op function calls it yet (Wave 2 stub)
- **Fix:** Added `#[allow(dead_code)]` with doc comment explaining it's used in Wave 2 XYZALM implementation
- **Files modified:** `hp41-core/src/ops/time/alarm.rs`
- **Commit:** ad260d0

**3. [Rule 1 - Bug] Three existing backward-compat test files asserting old default**
- **Found during:** Task 3 full test suite run
- **Issue:** `stat1_backward_compat.rs`, `v3_save_compat.rs`, and `state.rs` tests still expected `0b0000_0011` after Phase 38 flipped default to `0b0000_0111`
- **Fix:** Updated assertions in all three files to 0b0000_0111
- **Files modified:** `hp41-core/tests/stat1_backward_compat.rs`, `hp41-core/tests/v3_save_compat.rs`, `hp41-core/src/state.rs`
- **Commit:** 5c3c387

## Known Stubs

All 35 Time Module op functions are Phase 38 stubs — this is intentional per plan design. Wave 2 plans implement individual groups:
- `op_time`, `op_date` — push zero (SystemTime logic in Wave 2)
- `op_setime`, `op_setdate` — open modal prompt (full parsing in Wave 2)
- `date_to_jdn`, `jdn_to_date`, `jdn_to_dow` — return 0/(1,1,1970)/0 (Julian Day formula in Wave 2)
- `op_date_plus`, `op_ddays`, `op_dow` — push zero (JDN arithmetic in Wave 2)
- `op_atime`, `op_atime24`, `op_adate` — append placeholder string to ALPHA register
- All 8 alarm ops — return Ok(()) / open modal (full alarm system in Wave 2)

These stubs are documented by plan design and do not prevent the plan's goal (compile-time framework).

## Threat Surface Scan

No new network endpoints, auth paths, or file access patterns introduced. CalcState deserialization threat (T-38-01) mitigated by `#[serde(default)]` on all 8 new persistent fields and `#[serde(skip)]` on all 4 transient fields. `alarms: Vec<AlarmEntry>` is unbounded (T-38-02 accepted per plan — cap enforcement deferred to Plan 05 XYZALM implementation).

## Self-Check: PASSED

Files exist:
- `hp41-core/src/ops/time/mod.rs` — FOUND
- `hp41-core/src/ops/time/clock.rs` — FOUND
- `hp41-core/src/ops/time/date_arith.rs` — FOUND
- `hp41-core/src/ops/time/alpha_time.rs` — FOUND
- `hp41-core/src/ops/time/stopwatch.rs` — FOUND
- `hp41-core/src/ops/time/alarm.rs` — FOUND
- `hp41-core/src/ops/time/modal.rs` — FOUND

Commits exist:
- `ad260d0` — feat(38-01): scaffold Time Module framework — FOUND
- `5c3c387` — test(38-01): serde + migration tests — FOUND

Test suite: 2094 passed, 0 failed
Free42 contamination guard: exits 0
