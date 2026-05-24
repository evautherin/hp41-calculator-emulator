---
phase: 38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core
plan: "06"
subsystem: hp41-core
tags: [time-module, modal, setime, setdate, clock-offset, pm-shorthand]
dependency_graph:
  requires:
    - 38-01  # Time Module framework scaffold (TimeStep enum, modal stubs)
    - 38-03  # clock.rs with decompose_epoch_secs + adjusted_epoch_secs
  provides:
    - TIME-DSP-03  # SETIME modal submit computes time_offset_secs delta
    - TIME-DSP-04  # SETDATE modal submit computes date offset delta
  affects:
    - hp41-core/src/ops/time/modal.rs
tech_stack:
  added: []
  patterns:
    - "Modal-clear-before-compute pattern (modal_program=None; modal_prompt=None BEFORE offset computation)"
    - "PM shorthand normalization: -1..-11 maps to 13h..23h; -12 returns Domain (T-38-16)"
    - "time_offset_secs delta accumulation: state += entered_seconds - current_seconds"
    - "Date offset via JDN difference: day_delta * 86400 added to time_offset_secs"
key_files:
  created: []
  modified:
    - hp41-core/src/ops/time/modal.rs
key_decisions:
  - "Modal-clear pattern applied before offset computation (matching Stat1 precedent from Plan 33)"
  - "PM shorthand: -1..-11 mapped to 13h..23h per T-38-16; -12 returns HpError::Domain"
  - "XyzalmTimePrompt kept as forward-compat stub returning Ok (XYZALM reads all stack regs at once without multi-step modal)"
  - "normalize_pm_shorthand rejects fractional negative values (not PM shorthand)"
  - "submit_set_date_clears_modal_state test updated to provide valid date in X (Rule 1 fix: stub-to-real transition)"

requirements-completed:
  - TIME-DSP-03
  - TIME-DSP-04

metrics:
  duration_minutes: 35
  completed: "2026-05-24T22:30:00Z"
  tasks_completed: 1
  files_created: 0
  files_modified: 1
---

# Phase 38 Plan 06: Time Module Modal Dispatch Logic Summary

**TimeStep submit_step fully implemented: SETIME computes `time_offset_secs` delta from HH.MMSScc entry with PM shorthand support, SETDATE computes day delta via JDN arithmetic respecting Flag 31**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-05-24T22:00:00Z
- **Completed:** 2026-05-24T22:35:00Z
- **Tasks:** 1 (TDD: RED commit + GREEN commit)
- **Files modified:** 1

## Accomplishments

- Replaced 3 Wave-1 stubs in `time/modal.rs::submit_step` with full offset-computation logic
- SETIME: reads X as HH.MMSScc, handles PM shorthand (-1..-11 to 13h..23h, -12 to Domain per T-38-16), computes `delta = entered_secs - current_secs`, updates `state.time_offset_secs`
- SETDATE: reads X per Flag 31 (MDY/DMY), computes `day_delta * 86400` via JDN arithmetic, updates `state.time_offset_secs`
- Modal-clear pattern: both ops clear `modal_program` and `modal_prompt` BEFORE computing offset (matching Stat1 precedent)
- Verified `ModalProgram::Time` dispatch arm already wired in `math1/mod.rs` from Plan 01 (no changes needed)
- 8 new tests covering offset update, PM shorthand, Flag 31 DMY, invalid date, and math1 dispatch path

## Task Commits

TDD task had two phases:

1. **RED: Failing tests for TimeStep submit_step** - `7919974` (test)
2. **GREEN: TimeStep submit_step offset computation** - `7b2d975` (feat)

## Files Created/Modified

- `hp41-core/src/ops/time/modal.rs` — replaced 3 stubs with full SetTimePrompt/SetDatePrompt logic; added normalize_pm_shorthand helper + current_adjusted_epoch helper; added 8 new tests; updated existing stub-era test to provide valid date

## Decisions Made

- **Modal-clear before compute:** Following the established Stat1 pattern (stat1/modal.rs), modal state is cleared BEFORE computing the offset. If parsing fails, the modal is already gone — user must re-invoke SETIME/SETDATE.
- **PM shorthand range -1..-11 only:** Per T-38-16, only integer values -1 through -11 are valid PM shorthand. -12 returns `HpError::Domain` (12+12=24 is not a valid hour). Fractional negatives are also rejected.
- **XyzalmTimePrompt stub retained:** XYZALM reads all stack registers at once (time from X, date from Y, repeat from Z) without a multi-step modal per the QRC stack layout. The prompt variant exists as forward-compatibility for a potential future interactive mode.
- **`current_adjusted_epoch` wraps SystemTime:** Reuses the same pattern as `clock.rs::adjusted_epoch_secs` to avoid duplicating the SystemTime call. Does not export (private helper).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated stub-era test `submit_set_date_clears_modal_state`**
- **Found during:** Task 1 GREEN phase
- **Issue:** Existing test from Plan 01 called `submit_step(SetDatePrompt)` with default `CalcState` having `stack.x = 0.0`. After implementing real logic, parsing `0.0` as a date (month=0) returns `HpError::InvalidInput` — test panicked.
- **Fix:** Added `state.stack.x = "5.242026"` (May 24, 2026 MDY) + `state.flags = 0` to provide valid input. Test now validates both modal-clear behavior AND that a valid date succeeds.
- **Files modified:** `hp41-core/src/ops/time/modal.rs`
- **Verification:** All 16 modal tests pass after fix.
- **Committed in:** `7b2d975` (GREEN task commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - stub-to-real test update)
**Impact on plan:** Required for correctness — stub test no longer valid after real implementation.

## Issues Encountered

- The 38-05 alarm plan is on branch `worktree-agent-ab6e0ea302b139951` (not yet merged to develop). This plan's worktree is based on `8ab2603` (develop at wave 1+2 boundary) which does NOT include 38-05 alarm tests. Test count 2206 (vs. 2214 on 38-05 branch) is expected — no regressions.

## Known Stubs

`XyzalmTimePrompt` in `submit_step` returns `Ok(())` after clearing modal state. This is an intentional forward-compat stub — XYZALM op reads all stack registers simultaneously rather than using a multi-step modal. Documented in code comment.

## Threat Surface Scan

No new network endpoints, auth paths, or file access patterns introduced. `normalize_pm_shorthand` enforces T-38-16 (only -1..-11 valid; others return Domain). `time_offset_secs` accumulation uses `saturating_add` (T-38-17 accepted).

## Self-Check: PASSED

Files:
- `hp41-core/src/ops/time/modal.rs` — FOUND

Commits:
- `7919974` (test) — FOUND
- `7b2d975` (feat) — FOUND

Tests: 2206 passed, 0 failed (hp41-core full suite)
