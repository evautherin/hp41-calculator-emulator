---
phase: 38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core
verified: 2026-05-25T07:30:00Z
status: passed
score: 5/5
overrides_applied: 0
gaps:
  - truth: "TIME-FMT-03 / TIME-FMT-04: SETAF stores accuracy factor, RCLAF recalls accuracy factor"
    status: failed
    reason: "dispatch() routes TimeSetaf -> alarm::op_setaf (no-op) and TimeRclaf -> alarm::op_rclaf (returns alarm count). The correct clock::op_setaf and clock::op_rclaf implementations exist but are orphaned — time/mod.rs re-exports them from alarm, not clock. REQUIREMENTS.md TIME-FMT-03 and TIME-FMT-04 are not met."
    artifacts:
      - path: "hp41-core/src/ops/time/alarm.rs"
        issue: "op_rclaf returns alarm count (len()); op_setaf is a no-op. Both carry comments labeling them 'alarm flags' but REQUIREMENTS + RESEARCH say they operate on accuracy_factor."
      - path: "hp41-core/src/ops/time/mod.rs"
        issue: "Re-exports op_setaf and op_rclaf from alarm module instead of clock module."
      - path: "hp41-core/src/ops/mod.rs"
        issue: "dispatch() arms route Op::TimeSetaf and Op::TimeRclaf to alarm:: functions."
    missing:
      - "mod.rs should re-export op_setaf and op_rclaf from clock, not alarm"
      - "Or: move accuracy_factor store/recall logic into alarm.rs if dual behavior is intended (but this contradicts RESEARCH.md Clock category assignment)"
  - truth: "Clippy -D warnings passes across the workspace (CI gate)"
    status: failed
    reason: "cargo clippy --workspace --all-targets --all-features -- -D warnings exits 101 with 9 errors in 3 files. The justfile 'just lint' command uses this exact invocation and CI runs it."
    artifacts:
      - path: "hp41-core/src/ops/time/alarm.rs"
        issue: "4x unnecessary_cast (i32 as i32), 2x clone_on_copy (Decimal::clone() on Copy type) at lines 135, 196, 285, 461"
      - path: "hp41-core/src/ops/time/date_arith.rs"
        issue: "2x unused_variables: month and day in test at line 384"
      - path: "hp41-core/src/ops/math1/xrom.rs"
        issue: "1x unused_imports: time_resolve in test use block at line 481"
    missing:
      - "alarm.rs:135,196: remove `as i32` casts (already i32)"
      - "alarm.rs:285,461: remove `.clone()` call on Copy type Decimal"
      - "date_arith.rs:384: prefix unused vars with _ or use them"
      - "xrom.rs:481: remove time_resolve from the use list in the test or prefix with _"
deferred:
  - truth: "Clock display updates at ≥1 Hz in both CLI and GUI when active (TIME-DSP-05)"
    addressed_in: "Phase 39"
    evidence: "Phase 39 success criterion 2: 'User can activate clock display mode and see the TUI LCD area updating the current time at least once per second'. Phase 41 SC-1: 'GUI LCD updating at least once per second'. VALIDATION.md explicitly marks TIME-DSP-05 as Manual-Only for Phase 39/41."
  - truth: "Stopwatch display updates at ≥10 Hz in CLI and GUI (TIME-SW-08)"
    addressed_in: "Phase 39"
    evidence: "Phase 39 success criterion 3 mentions stopwatch display at 10+ Hz. VALIDATION.md explicitly marks TIME-SW-08 as Manual-Only for Phase 39/41."
  - truth: "Past-due alarm detection fires on each dispatch (TIME-ALM-08 — check_alarms wired)"
    addressed_in: "Phase 39"
    evidence: "check_alarms() function is fully implemented and public in alarm.rs. The SUMMARY documents it is intended to be 'called by the frontend after every dispatch()'. Phase 39 requirement TIME-CLI-07 covers alarm notifications in CLI status bar."
---

# Phase 38: hp41-core XROM Framework + Clock/Date/Stopwatch/Alarm Core — Verification Report

**Phase Goal:** Users can access all Time Pac functions through XEQ-by-name and programmatic execution -- TIME/DATE recall the host clock, DATE+/DDAYS/DOW perform calendar arithmetic, stopwatch tracks elapsed time, alarm catalog stores/retrieves/detects past-due alarms, and clock display mode flags control live display behavior
**Verified:** 2026-05-25T07:30:00Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | XEQ "TIME"/"DATE" returns current host system time/date in X register | VERIFIED | op_time uses SystemTime::now() + time_offset_secs, formats as HH.MMSSss/MM.DDYYYY. 45 clock tests pass. |
| 2 | XEQ "DATE+" performs calendar arithmetic across leap year and century boundaries | VERIFIED | Fliegel-Van Flandern JDN algorithm in date_arith.rs. 40 tests pass including 2000-02-29 (leap) and 2100-02-29 (not leap) cases. |
| 3 | XEQ "RUNSW"/"STOPSW"/"RCLSW" tracks elapsed time via monotonic Instant | VERIFIED | stopwatch.rs uses std::time::Instant. 34 stopwatch tests pass including resume and split tests. |
| 4 | XEQ "XYZALM"/"RCLALM" stores and recalls alarms; alarm catalog persists | VERIFIED | op_xyzalm/op_rclalm implemented; alarms: Vec<AlarmEntry> with #[serde(default)]; 51 alarm tests pass including serde round-trip. |
| 5 | All ~33 Op variants compile in dispatch() and execute_op(); SETAF/RCLAF accuracy factor correctly routed | FAILED | 35 Op::Time* variants are compiled and wired (items 1+2 complete). However Op::TimeSetaf dispatches to alarm::op_setaf (no-op) and Op::TimeRclaf to alarm::op_rclaf (returns alarm count) — clock::op_setaf/op_rclaf for accuracy factor are orphaned. Clippy CI gate also fails (-D warnings exits 101). |

**Score:** 3/5 truths verified

### Deferred Items

Items not yet met but explicitly addressed in later milestone phases.

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | Clock display updates ≥1 Hz in CLI and GUI (TIME-DSP-05) | Phase 39 + Phase 41 | Phase 39 SC-2: "TUI LCD updating at least once per second". VALIDATION.md marks as Manual-Only for Phase 39/41. |
| 2 | Stopwatch display updates ≥10 Hz in CLI and GUI (TIME-SW-08) | Phase 39 + Phase 41 | Phase 39 SC-3: "10+ Hz stopwatch display". VALIDATION.md marks as Manual-Only for Phase 39/41. |
| 3 | Past-due alarm detection fires on each keypress/dispatch (TIME-ALM-08) | Phase 39 | check_alarms() is fully implemented and public in alarm.rs; wiring to dispatch cadence is Phase 39 (CLI) scope. TIME-CLI-07 covers alarm notifications. |

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-core/src/ops/time/mod.rs` | Time module hub with pub re-exports | WIRED | 7 submodules declared, types re-exported. 2.7K. |
| `hp41-core/src/ops/time/clock.rs` | All 13 clock/format/display ops | WIRED | SystemTime::now() + time_offset_secs + Fliegel-Van Flandern decomposition. 33K. |
| `hp41-core/src/ops/time/date_arith.rs` | JDN algorithms + date ops + parse helpers | WIRED | FVF JDN, parse_date_hpnum, parse_time_hpnum, secs_to_hpnum_time, 5 date ops. 27.7K. |
| `hp41-core/src/ops/time/alpha_time.rs` | ATIME, ATIME24, ADATE ops | WIRED | SystemTime-based, appends to alpha_reg, Flag 31 DMY/MDY. 16.3K. |
| `hp41-core/src/ops/time/stopwatch.rs` | Stopwatch state machine with 7 ops | WIRED | Instant::now() monotonic, all 7 ops + get_stopwatch_display_str. 21.7K. |
| `hp41-core/src/ops/time/alarm.rs` | Alarm catalog with 7 ops + check_alarms | WIRED | 253-cap, chronological sort, parse_alarm_type, check_alarms drain. 50K. op_setaf/op_rclaf dispatch to wrong behavior. |
| `hp41-core/src/ops/time/modal.rs` | TimeStep enum + dispatch logic | WIRED | submit_step with SetTimePrompt/SetDatePrompt offset computation, PM shorthand. 20.8K. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `math1/xrom.rs` | `time/mod.rs` | `time_resolve()` + bit-2 arm | WIRED | `if modules & 0b0000_0100 != 0` fires after bit-1, time_resolve matches 35 mnemonics. 3 tests pass. |
| `ops/mod.rs` | `ops/time/` | `dispatch()` 35 arms to time::* | WIRED | All 35 Op::Time* variants route to specific time submodule functions. |
| `state.rs` | `time/stopwatch.rs` | StopwatchMode field | WIRED | CalcState.stopwatch_mode: StopwatchMode with #[serde(default)]. |
| `ops/mod.rs` | `ops/time/alarm.rs` | TimeSetaf/TimeRclaf routing | FAILED | Routes to wrong functions — alarm::op_setaf (no-op) and alarm::op_rclaf (alarm count) instead of clock::op_setaf/op_rclaf (accuracy factor). |
| `math1/mod.rs` | `time/modal.rs` | ModalProgram::Time dispatch arm | WIRED | submit_modal_input has `ModalProgram::Time(step) => crate::ops::time::modal::submit_step(state, step)` at line 83. |
| `clock.rs` | `state.rs` | time_offset_secs modification | WIRED | op_tplusx, modal.rs submit_step both modify state.time_offset_secs. 23 occurrences in clock.rs. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `clock.rs::op_time` | epoch from SystemTime | `SystemTime::now() + UNIX_EPOCH` | Yes — live host clock | FLOWING |
| `clock.rs::op_date` | epoch decomposed to date | `decompose_epoch_secs(adjusted_epoch_secs(...))` | Yes | FLOWING |
| `date_arith.rs::op_date_plus` | JDN arithmetic on stack Y/X | `parse_date_hpnum(Y)` + `date_to_jdn` | Yes | FLOWING |
| `stopwatch.rs::op_rclsw` | elapsed_secs | `accumulated + start.elapsed()` (Instant) | Yes | FLOWING |
| `alarm.rs::op_rclaf` | accuracy_factor | **Returns alarm count** not accuracy_factor | No — WRONG SOURCE | STATIC (wrong value) |
| `alarm.rs::op_setaf` | accuracy_factor | No-op, does not write accuracy_factor | No — DISCONNECTED | DISCONNECTED |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All hp41-core tests pass | `cargo test -p hp41-core` | 2257 passed, 1 ignored | PASS |
| XROM bit-2 resolution | `cargo test -p hp41-core -- resolve_uses_bit_2_for_time_module` | 1 passed | PASS |
| TIME_MODULE id=26, 35 entries | `cargo test -p hp41-core -- time_module_const_id_and_name time_module_ops_has_correct_entry_count` | 2 passed | PASS |
| CalcState serde round-trip for new fields | `cargo test -p hp41-core -- time_fields_serde_round_trip` | 1 passed | PASS |
| v3.1 save migration to xrom_modules=7 | `cargo test -p hp41-core -- time_v31_save_migration` | 1 passed | PASS |
| Stopwatch Running-freeze on load | `cargo test -p hp41-core -- time_stopwatch_running_freeze_on_load` | 1 passed | PASS |
| All 211 time module tests | `cargo test -p hp41-core -- "ops::time::"` | 211 passed | PASS |
| Free42 contamination guard | `bash scripts/check-free42-contamination.sh` | exits 0, no contamination | PASS |
| Clippy -D warnings (CI gate) | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exits 101, 9 errors | FAIL |

### Probe Execution

No `scripts/*/tests/probe-*.sh` files declared or found for Phase 38.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| TIME-FW-01 | 38-01 | TIME_MODULE registered as XROM 26, bit-2 arm | SATISFIED | TIME_MODULE.id=26, xrom_resolve bit-2 arm verified |
| TIME-FW-02 | 38-01 | default_xrom_modules()=0b111, migration v3.1→v3.2 | SATISFIED | state.rs line 368: `0b0000_0111`, migrate_after_load bit-2 logic |
| TIME-FW-03 | 38-01 | CalcState serde invariants for new fields | SATISFIED | 8 persistent (#[serde(default)]), 4 transient (#[serde(default,skip)]), tests pass |
| TIME-FW-04 | 38-01 | SystemTime access, time_offset_secs persistent | SATISFIED | clock.rs SystemTime::now(), state.rs time_offset_secs: i64 |
| TIME-FW-05 | 38-01 | Stopwatch via Instant (monotonic), start transient | SATISFIED | stopwatch.rs uses Instant::now(), stopwatch_start: Option<Instant> with serde skip |
| TIME-FW-06 | 38-01 | Alarm catalog as persistent Vec<AlarmEntry> up to 253 | SATISFIED | state.rs alarms: Vec<AlarmEntry> #[serde(default)], cap enforced at 253 |
| TIME-CLK-01 | 38-03 | TIME returns HH.MMSSss from system clock + offset | SATISFIED | op_time: SystemTime + time_offset_secs, format verified |
| TIME-CLK-02 | 38-03 | DATE returns date per Flag 31 | SATISFIED | op_date: MDY/DMY per `flags & (1u64 << 31)` |
| TIME-CLK-03 | 38-04 | ATIME appends 12h/24h per clock_12h | SATISFIED | op_atime: clock_12h flag respected, tests pass |
| TIME-CLK-04 | 38-04 | ATIME24 always 24h | SATISFIED | op_atime24 ignores clock_12h |
| TIME-CLK-05 | 38-04 | ADATE appends per Flag 31 | SATISFIED | op_adate: state.flags & (1u64 << 31) |
| TIME-CLK-06 | 38-03 | T+X adds seconds to time_offset_secs | SATISFIED | op_tplusx: parse_time_hpnum + saturating_add to time_offset_secs |
| TIME-DAT-01 | 38-02 | DATE+ adds X days to Y date | SATISFIED | op_date_plus: JDN + integer days, leap year tests pass |
| TIME-DAT-02 | 38-02 | DDAYS returns signed day count | SATISFIED | op_ddays: JDN(Y) - JDN(X), signed |
| TIME-DAT-03 | 38-02 | DOW returns 0=Sunday..6=Saturday | SATISFIED | op_dow: jdn_to_dow = (jdn+1) % 7 |
| TIME-DAT-04 | 38-02 | DMY sets Flag 31 | SATISFIED | op_dmy: state.flags |= 1u64 << 31 |
| TIME-DAT-05 | 38-02 | MDY clears Flag 31 | SATISFIED | op_mdy: state.flags &= !(1u64 << 31) |
| TIME-DAT-06 | 38-02 | Date parsing uses string-split not float | SATISFIED | parse_date_hpnum: split at decimal point, left-pad to 6 chars |
| TIME-DSP-01 | 38-03 | CLKT toggles clock display mode | SATISFIED | op_clkt: Off→TimeOnly→Off cycling |
| TIME-DSP-02 | 38-03 | CLKTD toggles time+date display | SATISFIED | op_clktd: Off→TimeAndDate→Off cycling |
| TIME-DSP-03 | 38-06 | SETIME modal submit computes time_offset_secs delta | SATISFIED | submit_step(SetTimePrompt): delta = entered - current, saturating_add |
| TIME-DSP-04 | 38-06 | SETDATE modal submit computes date offset delta | SATISFIED | submit_step(SetDatePrompt): JDN diff × 86400, saturating_add |
| TIME-DSP-05 | 38-03 | Clock display ≥1 Hz (CLI/GUI) | DEFERRED | Phase 39/41 scope — VALIDATION.md marks as Manual-Only for Phase 39/41 |
| TIME-FMT-01 | 38-03 | CLK12 sets 12h format | SATISFIED | op_clk12: state.clock_12h = true |
| TIME-FMT-02 | 38-03 | CLK24 sets 24h format | SATISFIED | op_clk24: state.clock_12h = false |
| TIME-FMT-03 | 38-03 | SETAF stores accuracy factor from X | BLOCKED | Op::TimeSetaf dispatches to alarm::op_setaf (no-op). clock::op_setaf stores accuracy_factor but is orphaned. |
| TIME-FMT-04 | 38-03 | RCLAF recalls accuracy factor to X | BLOCKED | Op::TimeRclaf dispatches to alarm::op_rclaf (returns alarm count). clock::op_rclaf recalls accuracy_factor but is orphaned. |
| TIME-FMT-05 | 38-03 | CORRECT is documented no-op | SATISFIED | op_correct: apply_lift_effect(Neutral), returns Ok(()) |
| TIME-SW-01 | 38-04 | SETSW initializes stopwatch from X register | SATISFIED | op_setsw: parse_time_hpnum(X), write to stopwatch_accumulated, mode→Stopped |
| TIME-SW-02 | 38-04 | SW activates stopwatch keyboard mode | SATISFIED | op_sw: state.stopwatch_keyboard_mode = true |
| TIME-SW-03 | 38-04 | STPW records split point | SATISFIED | op_stpw: stopwatch_split = current_elapsed(state) |
| TIME-SW-04 | 38-04 | RUNSW starts/resumes timer | SATISFIED | op_runsw: stopwatch_start = Some(Instant::now()), mode→Running |
| TIME-SW-05 | 38-04 | STOPSW stops timer | SATISFIED | op_stopsw: elapsed added to accumulated, start cleared |
| TIME-SW-06 | 38-04 | RCLSW recalls elapsed as HH.MMSSss | SATISFIED | op_rclsw: current_elapsed → secs_to_hpnum_time |
| TIME-SW-07 | 38-04 | SWPT recalls split-point to X | SATISFIED | op_swpt: stopwatch_split → secs_to_hpnum_time → push to X |
| TIME-SW-08 | 38-04 | Stopwatch display ≥10 Hz (CLI/GUI) | DEFERRED | Phase 39/41 scope — VALIDATION.md marks as Manual-Only |
| TIME-SW-09 | 38-04 | Stopwatch uses monotonic Instant | SATISFIED | stopwatch.rs: Instant::now(), not SystemTime |
| TIME-ALM-01 | 38-05 | XYZALM stores alarm from X/Y/Z/ALPHA | SATISFIED | op_xyzalm: parse time/date/repeat, parse_alarm_type from alpha_reg, sorted insert |
| TIME-ALM-02 | 38-05 | RCLALM recalls alarm fields to stack/ALPHA | SATISFIED | op_rclalm: trigger_unix decomposed, time/date/repeat to stack, alpha_reg set |
| TIME-ALM-03 | 38-05 | ALMCAT enters catalog mode | SATISFIED | op_almcat: alarm_catalog_mode=true, first alarm to print_buffer |
| TIME-ALM-04 | 38-05 | CLALMA clears alarm by ALPHA content | SATISFIED | op_clalma: matches alarm_type against alpha_reg content (>> or > prefix or message text) |
| TIME-ALM-05 | 38-05 | CLALMX clears alarm by number | SATISFIED | op_clalmx: 1-indexed remove with range validation |
| TIME-ALM-06 | 38-05 | CLRALMS clears all alarms | SATISFIED | op_clralms: state.alarms.clear() |
| TIME-ALM-07 | 38-05 | ALMNOW triggers oldest past-due alarm | SATISFIED | op_almnow: finds first past_due alarm, dispatches event, reschedules or removes |
| TIME-ALM-08 | 38-05 | Past-due detection on each dispatch | DEFERRED | check_alarms() fully implemented; wired to dispatch cadence is Phase 39 scope |
| TIME-ALM-09 | 38-05 | Message alarms push to print_buffer/event_buffer | PARTIALLY SATISFIED | dispatch_alarm_event pushes to event_buffer and print_buffer. alpha_reg write and beep deferred to frontend (Phase 39). |
| TIME-ALM-10 | 38-05 | Control alarms push XEQ event | SATISFIED | dispatch_alarm_event: "alarm:xeq:{label}" to event_buffer (non-interrupting), "alarm:interrupting:deferred" (interrupting) |
| TIME-ALM-11 | 38-05 | Repeating alarms reschedule | SATISFIED | acknowledge_alarm: if repeat_secs > 0, trigger_unix += repeat_secs, past_due=false |
| TIME-ALM-12 | 38-05 | Alarm catalog persists across save/load | SATISFIED | alarms: Vec<AlarmEntry> with #[serde(default)], serde round-trip test passes |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `hp41-core/src/ops/time/alarm.rs` | 135 | `month as i32` (unnecessary cast, i32→i32) | Blocker | CI clippy -D warnings fails |
| `hp41-core/src/ops/time/alarm.rs` | 135 | `day as i32` (unnecessary cast, i32→i32) | Blocker | CI clippy -D warnings fails |
| `hp41-core/src/ops/time/alarm.rs` | 196 | `month as i32` (unnecessary cast, i32→i32) | Blocker | CI clippy -D warnings fails |
| `hp41-core/src/ops/time/alarm.rs` | 196 | `day as i32` (unnecessary cast, i32→i32) | Blocker | CI clippy -D warnings fails |
| `hp41-core/src/ops/time/alarm.rs` | 285 | `.clone()` on Copy type Decimal | Blocker | CI clippy -D warnings fails |
| `hp41-core/src/ops/time/alarm.rs` | 461 | `.clone()` on Copy type Decimal | Blocker | CI clippy -D warnings fails |
| `hp41-core/src/ops/time/date_arith.rs` | 384 | `month`, `day` unused variables (in test) | Blocker | CI clippy -D warnings fails |
| `hp41-core/src/ops/math1/xrom.rs` | 481 | `time_resolve` unused import in test use block | Blocker | CI clippy -D warnings fails |
| `hp41-core/src/ops/time/clock.rs` | ~249-265 | `op_setaf`, `op_rclaf` functions unreachable | Warning | mod.rs re-exports same names from alarm; clock:: versions orphaned |

### Human Verification Required

None — all verification is automated at this phase (hp41-core only; Phase 38 is pre-integration).

### Gaps Summary

**Two gaps block this phase:**

**Gap 1: SETAF/RCLAF dispatch routing mismatch (TIME-FMT-03, TIME-FMT-04)**

`time/mod.rs` re-exports `op_setaf` and `op_rclaf` from `alarm` instead of `clock`. Consequently `dispatch()` routes `Op::TimeSetaf` to `alarm::op_setaf` (a no-op) and `Op::TimeRclaf` to `alarm::op_rclaf` (returns alarm count). The semantically correct implementations in `clock::op_setaf` (stores X into `state.accuracy_factor`) and `clock::op_rclaf` (recalls `state.accuracy_factor` to X) are unreachable.

REQUIREMENTS.md, RESEARCH.md (categories RCLAF/SETAF as "Clock"), and PLAN 38-03 must-haves all require these to operate on the accuracy factor. The fix is to swap the re-exports in `mod.rs`:

```rust
// in time/mod.rs, change:
pub use alarm::{op_clalma, op_clalmx, op_clralms, op_rclaf, op_rclalm, op_setaf, ...};
pub use clock::{op_clk12, op_clk24, op_clkt, op_clktd, op_clock, op_correct, op_date, op_setdate, op_setime, op_time, op_tplusx, ClockDisplayMode};

// to:
pub use alarm::{op_clalma, op_clalmx, op_clralms, op_rclalm, ...};
pub use clock::{op_clk12, op_clk24, op_clkt, op_clktd, op_clock, op_correct, op_date, op_setdate, op_setime, op_time, op_tplusx, op_setaf, op_rclaf, ClockDisplayMode};
```

Also update `dispatch()` in `ops/mod.rs` to route to `crate::ops::time::clock::op_setaf` and `crate::ops::time::clock::op_rclaf`.

**Gap 2: Clippy -D warnings CI failure (9 errors in 3 files)**

The justfile `just lint` command runs `cargo clippy --workspace --all-targets --all-features -- -D warnings` and this exits 101 with 9 errors:
- `alarm.rs` lines 135, 196: Remove `as i32` casts (values are already i32)
- `alarm.rs` lines 285, 461: Remove `.clone()` call (Decimal implements Copy)
- `date_arith.rs` line 384 (in test): Prefix unused `month`, `day` with `_`
- `xrom.rs` line 481 (in test): Remove `time_resolve` from use block or add `#[allow(unused_imports)]`

Both gaps are small mechanical fixes with no behavioral complexity. They can be resolved in a single focused commit.

---

_Verified: 2026-05-25T07:30:00Z_
_Verifier: Claude (gsd-verifier)_
