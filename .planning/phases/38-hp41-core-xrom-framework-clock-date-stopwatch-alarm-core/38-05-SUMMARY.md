---
phase: 38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core
plan: "05"
subsystem: hp41-core/ops/time/alarm
tags: [alarm, catalog, xyzalm, rclalm, check_alarms, time-module, xrom-26]
dependency_graph:
  requires: [38-01, 38-02, 38-03]
  provides: [complete alarm catalog system, check_alarms drain, acknowledge_alarm helper]
  affects: [hp41-core/src/ops/time/alarm.rs, CalcState::alarms, CalcState::event_buffer]
tech_stack:
  added: []
  patterns:
    - JDN arithmetic for Unix epoch ↔ calendar date conversion (Fliegel-Van Flandern)
    - partition_point for sorted insertion into alarm catalog
    - check_alarms drain pattern (mirrors print_buffer drain cadence)
key_files:
  created: []
  modified:
    - hp41-core/src/ops/time/alarm.rs
decisions:
  - "Use HpError::InvalidInput for out-of-range alarm number and capacity overflow (HpError::Data does not exist in the enum)"
  - "parse_alarm_type checks >> before > (Pitfall 9 - longer match wins)"
  - "Y=0 in XYZALM means today's date; uses current_unix_secs + epoch_secs_to_date"
  - "check_alarms sets past_due flag but does not remove alarms; removal is via acknowledge_alarm"
metrics:
  duration: "~45 minutes"
  completed: "2026-05-24T21:47:39Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 1
---

# Phase 38 Plan 05: Alarm Catalog System Summary

Complete alarm catalog system replacing Wave 1 stubs: XYZALM stores alarms with type detection, cap enforcement, and chronological sort; RCLALM recalls to stack/ALPHA; clear ops work by number/name/all; check_alarms drain detects past-due alarms and pushes typed events; repeating alarms reschedule correctly after acknowledgment.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | XYZALM + RCLALM + alarm utility helpers | a767d89 | hp41-core/src/ops/time/alarm.rs |
| 2 | Clear ops + check_alarms + ALMNOW + ALMCAT | a767d89 | hp41-core/src/ops/time/alarm.rs |

Note: Tasks 1 and 2 are committed together because the implementation of Task 2 functions (check_alarms, op_clalma, op_clalmx, op_clralms, op_almnow) depends on helpers introduced in Task 1 (dispatch_alarm_event, alarm_matches_alpha, decompose_alarm_unix). A single atomic commit was appropriate.

## What Was Built

### hp41-core/src/ops/time/alarm.rs

Replaced the Phase 38 Wave 1 stubs (9 stub functions + placeholder test) with a complete 580-line implementation:

**Private helpers:**
- `parse_alarm_type(alpha)`: parses ALPHA register content into AlarmType; checks `>>` before `>` (Pitfall 9)
- `current_unix_secs(offset)`: SystemTime::now() + time_offset_secs for HP-41 wall clock
- `time_date_to_unix(h, m, s, y, mo, d)`: calendar + time to Unix epoch via JDN arithmetic
- `epoch_secs_to_date(epoch)`: Unix seconds to (year, month, day) via JDN arithmetic
- `make_time_hpnum(h, m, s)`: builds HH.MMSScc HpNum from components
- `make_date_hpnum(y, mo, d, dmy)`: builds MM.DDYYYY or DD.MMYYYY HpNum per Flag 31
- `repeat_hpnum_to_secs(hpnum)`: HHHH.MMSScc → total seconds (D-38.11 repeat interval)
- `decompose_alarm_unix(trigger_unix)`: trigger timestamp → (y, mo, d, h, m, s)
- `dispatch_alarm_event(state, alarm)`: pushes typed event to event_buffer + print_buffer
- `alarm_matches_alpha(alarm_type, alpha)`: CLALMA matching helper

**Public op functions:**
- `op_xyzalm`: X=time, Y=date (0=today), Z=repeat → stores alarm sorted chronologically; 253-entry cap enforced
- `op_rclalm`: N → time/date/repeat to X/Y/Z stack + alarm label/message to ALPHA register
- `op_almcat`: sets alarm_catalog_mode=true; first alarm formatted to print_buffer
- `op_almnow`: triggers oldest past-due alarm; reschedules repeating, removes one-shot
- `op_rclaf`: pushes alarm count to X
- `op_setaf`: stub (alarm flags management deferred)
- `op_clalma`: clears alarm matching ALPHA content (message text or >label/>>label form)
- `op_clalmx`: clears alarm at 1-indexed position X
- `op_clralms`: clears all alarms

**Public drain function:**
- `check_alarms(state)`: scans alarms for past-due (`trigger_unix <= now`); marks `past_due=true`; dispatches events:
  - Message: `"alarm:message:{text}"` → event_buffer + text → print_buffer
  - Non-interrupting control: `"alarm:xeq:{label}"` → event_buffer
  - Interrupting control: `"alarm:interrupting:deferred"` → event_buffer (D-38.4)

**Public acknowledge helper:**
- `acknowledge_alarm(state, index)`: if repeat_secs > 0 → reschedule (trigger_unix += repeat_secs, past_due=false); else remove

**Tests:** 51 tests covering all ops, parse_alarm_type edge cases, round-trip date/time conversion, cap enforcement, sorted insertion, RCLALM stack layout, check_alarms event dispatch, repeat scheduling, catalog management.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] HpError::Data does not exist in the codebase**
- **Found during:** Initial compilation attempt
- **Issue:** Plan 38-05 specified `HpError::Data` for out-of-range and capacity errors. The actual `HpError` enum in `hp41-core/src/error.rs` has no `Data` variant. Available variants include `InvalidInput`, `InvalidOp`, `OutOfRange`, `Overflow`, etc.
- **Fix:** Used `HpError::InvalidInput` for all out-of-range alarm number validation and the 253-entry capacity cap. This is semantically correct — the user input (alarm number or catalog full) is invalid.
- **Files modified:** hp41-core/src/ops/time/alarm.rs
- **Test assertions:** Updated all test assertions from `HpError::Data` to `HpError::InvalidInput`
- **Commit:** a767d89

**2. [Rule 1 - Bug] Rust pattern binding deref mismatch**
- **Found during:** Initial compilation
- **Issue:** In `dispatch_alarm_event`, pattern `AlarmType::Control { label, interrupting }` when matching `&alarm.alarm_type` — Rust match ergonomics makes `interrupting: bool` directly (not `&bool`), so `*interrupting` fails with "type `bool` cannot be dereferenced"
- **Fix:** Removed the `*` dereference; use `interrupting` directly in the `if` condition
- **Files modified:** hp41-core/src/ops/time/alarm.rs
- **Commit:** a767d89

**3. [Rule 1 - Bug] Type comparison mismatch for zero detection**
- **Found during:** Initial compilation
- **Issue:** Original code `y_inner == &rust_decimal::Decimal::ZERO` where `y_inner` is `Decimal` (by value from `.inner()`), comparing against `&Decimal::ZERO` gives mismatched types
- **Fix:** Used `state.stack.y.is_zero()` helper method (exists on `HpNum`) instead of manual Decimal comparison
- **Files modified:** hp41-core/src/ops/time/alarm.rs
- **Commit:** a767d89

## Verification

```
cargo test -p hp41-core -- alarm
Result: 51 passed, 2199 filtered out

cargo test -p hp41-core
Result: 2249 passed, 1 ignored (79 suites)

grep "253" hp41-core/src/ops/time/alarm.rs
Result: 3 matches (doc comment + cap comment + enforcement check)

grep "strip_prefix" hp41-core/src/ops/time/alarm.rs
Result: 2 matches (>> prefix then > prefix — correct order per Pitfall 9)
```

## Known Stubs

None. All alarm operations are fully implemented. `op_setaf` is a no-op by design — alarm flag management (SETAF) is deferred to Phase 39/41 (frontend integration); the stub is correct for the core layer.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. The alarm catalog is stored in `CalcState::alarms` (already declared in Plan 38-01). The `check_alarms` function reads `state.time_offset_secs` to compare `trigger_unix <= now`, which is the standard pattern established in clock.rs.

Threat model mitigations from plan verified:
- T-38-13 (catalog DoS): `state.alarms.len() >= 253` cap enforced at line 219
- T-38-14 (ALPHA prefix injection): `parse_alarm_type` uses string prefix matching only; no eval/exec
- T-38-15 (event_buffer xeq format): Core pushes string format; frontend parses and executes; core never runs the label

## Self-Check: PASSED

- alarm.rs: FOUND at hp41-core/src/ops/time/alarm.rs
- SUMMARY.md: FOUND at .planning/phases/38-.../38-05-SUMMARY.md
- Commit a767d89: FOUND in git log
- 51 alarm tests pass, 2249 total pass
