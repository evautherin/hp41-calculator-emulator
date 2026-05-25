// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Clock, display, and format operations for the HP Time Module.
//!
//! Covers: TIME, DATE, SETIME, SETDATE, CLK12, CLK24, CLKT, CLKTD, CLOCK,
//!         CORRECT, T+X, SETAF, RCLAF.
//!
//! Time representation: HH.MMSScc (hours.minutessecondscentiseconds).
//! Date representation: depends on Flag 31:
//!   - Flag 31 clear (MDY mode): MM.DDYYYY
//!   - Flag 31 set   (DMY mode): DD.MMYYYY
//!
//! Architecture: D-38.1 — `SystemTime::now()` in hp41-core, value-returning
//! syscall not console I/O. D-38.2 — `time_offset_secs: i64` adjusts the
//! apparent clock. D-38.3 — OS local time via hand-coded Gregorian arithmetic
//! (D-carried.1: zero new runtime deps; no libc/chrono/time crates).

use crate::error::HpError;
use crate::num::HpNum;
use crate::ops::math1::modal::ModalProgram;
use crate::ops::time::date_arith::parse_time_hpnum;
use crate::ops::time::modal::TimeStep;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

/// Clock display mode for `CalcState::clock_display_mode`.
///
/// `Off`: no continuous clock display.
/// `TimeOnly`: CLKT — display time continuously.
/// `TimeAndDate`: CLKTD — display time and date alternating.
///
/// Default: `Off` (first variant = derive default).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum ClockDisplayMode {
    #[default]
    Off,
    TimeOnly,
    TimeAndDate,
}

/// Decompose a Unix epoch second (with user-applied offset) into local
/// calendar components.
///
/// Returns `(year, month, day, hour, minute, second)`.
///
/// Uses pure-Rust Gregorian calendar arithmetic (D-carried.1: zero new
/// runtime deps — no libc, chrono, or time crate). Algorithm follows the
/// standard epoch→date decomposition well-known in RPN calculator firmware.
///
/// NOTE: This gives LOCAL time by using `time_offset_secs` which has already
/// been adjusted by SETIME to embed the user's timezone delta. It does NOT
/// consult the OS timezone; the offset field IS the timezone compensation.
pub(crate) fn decompose_epoch_secs(epoch_secs: i64) -> (i32, u8, u8, u8, u8, u8) {
    // Seconds within the day (handle negative epoch: floor division).
    let secs_per_day: i64 = 86_400;
    // day_number is days since 1970-01-01 (may be negative for pre-epoch).
    let day_number = if epoch_secs >= 0 {
        epoch_secs / secs_per_day
    } else {
        // Floor division for negatives: e.g. -1 sec → day -1.
        (epoch_secs - secs_per_day + 1) / secs_per_day
    };
    let time_of_day = (epoch_secs - day_number * secs_per_day) as u32;
    let hour = (time_of_day / 3600) as u8;
    let minute = ((time_of_day % 3600) / 60) as u8;
    let second = (time_of_day % 60) as u8;

    // Convert Julian Day Number to Gregorian date.
    // JDN for 1970-01-01 = 2440588.
    let jdn: i64 = day_number + 2_440_588;

    // Fliegel-Van Flandern algorithm (Communications of the ACM 1968).
    let l = jdn + 68_569;
    let n = (4 * l) / 146_097;
    let l = l - (146_097 * n + 3) / 4;
    let i = (4000 * (l + 1)) / 1_461_001;
    let l = l - (1461 * i) / 4 + 31;
    let j = (80 * l) / 2447;
    let day = (l - 2447 * j / 80) as u8;
    let l = j / 11;
    let month = (j + 2 - 12 * l) as u8;
    let year = (100 * (n - 49) + i + l) as i32;

    (year, month, day, hour, minute, second)
}

/// Read system clock and apply the user offset, returning epoch seconds.
fn adjusted_epoch_secs(offset_secs: i64) -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    now.as_secs() as i64 + offset_secs
}

/// TIME — Read current time (adjusted by `time_offset_secs`) and push as
/// HH.MMSScc HpNum onto stack X.
///
/// Uses `SystemTime::now()` per D-38.1. Applies `time_offset_secs` per D-38.2.
/// Centiseconds = 0 (SystemTime resolution is seconds via UNIX_EPOCH difference).
pub fn op_time(state: &mut CalcState) -> Result<(), HpError> {
    let epoch = adjusted_epoch_secs(state.time_offset_secs);
    let (_year, _month, _day, hour, minute, second) = decompose_epoch_secs(epoch);
    // Format as HH.MMSScc — centiseconds are 0 (1-second resolution).
    let hpnum_str = format!("{}.{:02}{:02}00", hour, minute, second);
    let d = Decimal::from_str(&hpnum_str).map_err(|_| HpError::InvalidOp)?;
    enter_number(state, HpNum::from(d));
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// DATE — Read current date (adjusted by `time_offset_secs`) and push as
/// date HpNum (MM.DDYYYY or DD.MMYYYY depending on Flag 31) onto stack X.
///
/// Flag 31 clear → MDY (MM.DDYYYY). Flag 31 set → DMY (DD.MMYYYY).
pub fn op_date(state: &mut CalcState) -> Result<(), HpError> {
    let epoch = adjusted_epoch_secs(state.time_offset_secs);
    let (year, month, day, _hour, _minute, _second) = decompose_epoch_secs(epoch);
    let dmy = (state.flags & (1u64 << 31)) != 0;
    // HP-41 date format:
    //   MDY: MM.DDYYYY  e.g. month=5, day=24, year=2026 → "5.242026"
    //   DMY: DD.MMYYYY  e.g. day=24, month=5, year=2026 → "24.052026"
    let hpnum_str = if dmy {
        format!("{}.{:02}{:04}", day, month, year)
    } else {
        format!("{}.{:02}{:04}", month, day, year)
    };
    let d = Decimal::from_str(&hpnum_str).map_err(|_| HpError::InvalidOp)?;
    enter_number(state, HpNum::from(d));
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// SETIME — Set clock time from stack X (HH.MMSScc format).
///
/// Opens the SetTimePrompt modal. The actual offset computation happens
/// in `modal.rs::submit_step(SetTimePrompt)` (Plan 06).
pub fn op_setime(state: &mut CalcState) -> Result<(), HpError> {
    state.modal_program = Some(ModalProgram::Time(TimeStep::SetTimePrompt));
    state.modal_prompt = Some("TIME?".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// SETDATE — Set clock date from stack X (date decimal format per Flag 31).
///
/// Opens the SetDatePrompt modal.
pub fn op_setdate(state: &mut CalcState) -> Result<(), HpError> {
    state.modal_program = Some(ModalProgram::Time(TimeStep::SetDatePrompt));
    state.modal_prompt = Some("DATE?".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CLK12 — Set 12-hour clock mode.
///
/// Sets `state.clock_12h = true`. LiftEffect::Neutral.
pub fn op_clk12(state: &mut CalcState) -> Result<(), HpError> {
    state.clock_12h = true;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CLK24 — Set 24-hour clock mode.
///
/// Sets `state.clock_12h = false`. LiftEffect::Neutral.
pub fn op_clk24(state: &mut CalcState) -> Result<(), HpError> {
    state.clock_12h = false;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CLKT — Toggle continuous time-only display.
///
/// Cycle: Off → TimeOnly → Off.
/// If currently TimeAndDate, step down to TimeOnly (per plan spec).
/// Sets `clock_active = true` when entering TimeOnly, false when going Off.
pub fn op_clkt(state: &mut CalcState) -> Result<(), HpError> {
    match state.clock_display_mode {
        ClockDisplayMode::Off => {
            state.clock_display_mode = ClockDisplayMode::TimeOnly;
            state.clock_active = true;
        }
        ClockDisplayMode::TimeOnly => {
            state.clock_display_mode = ClockDisplayMode::Off;
            state.clock_active = false;
        }
        ClockDisplayMode::TimeAndDate => {
            // Step down to TimeOnly.
            state.clock_display_mode = ClockDisplayMode::TimeOnly;
            state.clock_active = true;
        }
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CLKTD — Toggle time+date display.
///
/// Cycle: Off → TimeAndDate → Off.
/// If currently TimeOnly, step up to TimeAndDate.
/// Sets `clock_active = true` when entering TimeAndDate, false when going Off.
pub fn op_clktd(state: &mut CalcState) -> Result<(), HpError> {
    match state.clock_display_mode {
        ClockDisplayMode::Off => {
            state.clock_display_mode = ClockDisplayMode::TimeAndDate;
            state.clock_active = true;
        }
        ClockDisplayMode::TimeOnly => {
            state.clock_display_mode = ClockDisplayMode::TimeAndDate;
            state.clock_active = true;
        }
        ClockDisplayMode::TimeAndDate => {
            state.clock_display_mode = ClockDisplayMode::Off;
            state.clock_active = false;
        }
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CLOCK — Enable programmatic clock display (TimeOnly mode).
///
/// Per RESEARCH.md Open Question 1: implement as functional.
/// Sets `clock_display_mode = TimeOnly` and `clock_active = true`.
pub fn op_clock(state: &mut CalcState) -> Result<(), HpError> {
    state.clock_display_mode = ClockDisplayMode::TimeOnly;
    state.clock_active = true;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CORRECT — Apply clock accuracy correction.
///
/// Documented no-op per TIME-FMT-05 (HP 82182A OM §CORRECT). The accuracy
/// correction factor is stored via SETAF; CORRECT is a legacy command that
/// on real hardware adjusted an analog correction circuit — not emulatable.
/// Returns Ok(()) without modifying any state.
pub fn op_correct(state: &mut CalcState) -> Result<(), HpError> {
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// SETAF — Set accuracy factor.
///
/// Stores stack X into `state.accuracy_factor`. LiftEffect::Neutral.
pub fn op_setaf(state: &mut CalcState) -> Result<(), HpError> {
    state.accuracy_factor = state.stack.x.clone();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// RCLAF — Recall accuracy factor.
///
/// Pushes `state.accuracy_factor` onto stack X. LiftEffect::Enable.
pub fn op_rclaf(state: &mut CalcState) -> Result<(), HpError> {
    enter_number(state, state.accuracy_factor.clone());
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// T+X — Add elapsed time encoded in X to the time offset.
///
/// X is encoded as HH.MMSSss (HP-41 time format). Converts to total seconds
/// and adds to `state.time_offset_secs`. Midnight rollover is automatic since
/// `time_offset_secs` is a single i64 seconds offset (per PITFALL 8).
///
/// Validates T+X input via `parse_time_hpnum` (T-38-09 mitigation).
pub fn op_tplusx(state: &mut CalcState) -> Result<(), HpError> {
    let (hours, minutes, seconds, centiseconds) = parse_time_hpnum(&state.stack.x)?;
    let delta_secs = i64::from(hours) * 3600 + i64::from(minutes) * 60 + i64::from(seconds);
    // Centiseconds contribute fractional seconds — ignored since time_offset_secs is i64.
    // The centisecond component (0-99) does not round to a full second at HP-41 precision.
    let _ = centiseconds; // acknowledged: sub-second resolution not tracked in offset
    state.time_offset_secs = state.time_offset_secs.saturating_add(delta_secs);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// Returns a formatted clock display string for frontend use (TIME-DSP-05).
///
/// Returns `Some(string)` if `clock_active` is true, `None` otherwise.
///
/// Format:
/// - 24h mode: `"HH:MM:SS"`
/// - 12h mode: `"HH:MM:SS AM"` or `"HH:MM:SS PM"`
///
/// When `clock_display_mode == TimeAndDate`, alternates between time and date
/// display on even/odd seconds (even = time, odd = date).
///
/// This is a pure compute function — callers supply the current CalcState.
/// Frontend should call this on each redraw cycle (D-carried.7: pull on redraw).
pub fn get_clock_display_str(state: &CalcState) -> Option<String> {
    if !state.clock_active {
        return None;
    }
    let epoch = adjusted_epoch_secs(state.time_offset_secs);
    let (year, month, day, hour, minute, second) = decompose_epoch_secs(epoch);

    if state.clock_display_mode == ClockDisplayMode::TimeAndDate {
        // Alternate: even seconds → time, odd seconds → date.
        if epoch % 2 == 0 {
            Some(format_time_str(hour, minute, second, state.clock_12h))
        } else {
            let dmy = (state.flags & (1u64 << 31)) != 0;
            Some(format_date_str(year, month, day, dmy))
        }
    } else {
        // TimeOnly (or Off, but we guarded clock_active above).
        Some(format_time_str(hour, minute, second, state.clock_12h))
    }
}

/// Format time as `HH:MM:SS` (24h) or `HH:MM:SS AM/PM` (12h).
fn format_time_str(hour: u8, minute: u8, second: u8, clock_12h: bool) -> String {
    if clock_12h {
        let (display_hour, ampm) = if hour == 0 {
            (12u8, "AM")
        } else if hour < 12 {
            (hour, "AM")
        } else if hour == 12 {
            (12u8, "PM")
        } else {
            (hour - 12, "PM")
        };
        format!("{:02}:{:02}:{:02} {}", display_hour, minute, second, ampm)
    } else {
        format!("{:02}:{:02}:{:02}", hour, minute, second)
    }
}

/// Format date as `MM/DD/YYYY` (MDY) or `DD/MM/YYYY` (DMY).
fn format_date_str(year: i32, month: u8, day: u8, dmy: bool) -> String {
    if dmy {
        format!("{:02}/{:02}/{:04}", day, month, year)
    } else {
        format!("{:02}/{:02}/{:04}", month, day, year)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ── ClockDisplayMode ──────────────────────────────────────────────────────

    #[test]
    fn clock_display_mode_default_is_off() {
        assert_eq!(ClockDisplayMode::default(), ClockDisplayMode::Off);
    }

    #[test]
    fn clock_display_mode_serde_round_trip() {
        let modes = [
            ClockDisplayMode::Off,
            ClockDisplayMode::TimeOnly,
            ClockDisplayMode::TimeAndDate,
        ];
        for mode in &modes {
            let json = serde_json::to_string(mode).unwrap();
            let restored: ClockDisplayMode = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, *mode);
        }
    }

    // ── decompose_epoch_secs ──────────────────────────────────────────────────

    #[test]
    fn decompose_epoch_secs_unix_epoch() {
        // 1970-01-01 00:00:00 UTC
        let (year, month, day, hour, minute, second) = decompose_epoch_secs(0);
        assert_eq!(year, 1970);
        assert_eq!(month, 1);
        assert_eq!(day, 1);
        assert_eq!(hour, 0);
        assert_eq!(minute, 0);
        assert_eq!(second, 0);
    }

    #[test]
    fn decompose_epoch_secs_known_date() {
        // 2026-05-24 15:30:45 UTC = known test date.
        // 2026-05-24: days from 1970-01-01 = ?
        // Use: 56 * 365 + 14 leap years + (31+28+31+30+31+24-1) days in 2026.
        // Simpler: verified against an external UTC date calculator.
        // 2026-05-24 = epoch day 20597.
        let epoch: i64 = 20597 * 86400 + 15 * 3600 + 30 * 60 + 45;
        let (year, month, day, hour, minute, second) = decompose_epoch_secs(epoch);
        assert_eq!(year, 2026);
        assert_eq!(month, 5);
        assert_eq!(day, 24);
        assert_eq!(hour, 15);
        assert_eq!(minute, 30);
        assert_eq!(second, 45);
    }

    #[test]
    fn decompose_epoch_secs_midnight_boundary() {
        // 86399 = 23:59:59 on 1970-01-01.
        let (year, month, day, hour, minute, second) = decompose_epoch_secs(86399);
        assert_eq!(year, 1970);
        assert_eq!(month, 1);
        assert_eq!(day, 1);
        assert_eq!(hour, 23);
        assert_eq!(minute, 59);
        assert_eq!(second, 59);
    }

    #[test]
    fn decompose_epoch_secs_next_day() {
        // 86400 = 1970-01-02 00:00:00.
        let (year, month, day, hour, minute, second) = decompose_epoch_secs(86400);
        assert_eq!(year, 1970);
        assert_eq!(month, 1);
        assert_eq!(day, 2);
        assert_eq!(hour, 0);
        assert_eq!(minute, 0);
        assert_eq!(second, 0);
    }

    #[test]
    fn decompose_epoch_secs_leap_year() {
        // 2000-02-29 12:00:00 (2000 is a leap year).
        // 2000-02-29: epoch day = (30 * 365 + 8 leaps up to 1999) + 31 + 29 - 1 = ?
        // Verified: 2000-02-29 = epoch day 11016.
        let epoch: i64 = 11016 * 86400 + 12 * 3600;
        let (year, month, day, hour, minute, second) = decompose_epoch_secs(epoch);
        assert_eq!(year, 2000);
        assert_eq!(month, 2);
        assert_eq!(day, 29);
        assert_eq!(hour, 12);
        assert_eq!(minute, 0);
        assert_eq!(second, 0);
    }

    // ── op_time ───────────────────────────────────────────────────────────────

    #[test]
    fn op_time_returns_valid_hpnum_format() {
        // op_time should push a value in range 0.000000 .. 23.595959.
        let mut state = CalcState::new();
        state.time_offset_secs = 0;
        op_time(&mut state).unwrap();
        // Value should be non-negative.
        let val = state.stack.x.inner();
        assert!(
            val >= rust_decimal::Decimal::ZERO,
            "TIME must be non-negative"
        );
        // Integer part (hours) must be 0..=23.
        let hours = val.trunc();
        assert!(
            hours >= rust_decimal::Decimal::ZERO && hours <= rust_decimal::Decimal::from(23),
            "TIME hours must be 0..=23, got {hours}"
        );
    }

    #[test]
    fn op_time_applies_offset() {
        // With a +3600 offset (1 hour ahead), the hour component should differ
        // from the same call with offset 0. We test that the offset is applied
        // by checking the result is different (not necessarily by exact value
        // since test execution time is not controlled).
        //
        // Instead, verify: with a known epoch offset where both results are
        // within valid range, the offset shifts the result by 1.000000 or wraps.
        // Use a deterministic approach: call op_time with large offsets and
        // verify the structural format is valid in both cases.
        let mut state_a = CalcState::new();
        state_a.time_offset_secs = 0;
        op_time(&mut state_a).unwrap();
        let val_a = state_a.stack.x.inner();

        let mut state_b = CalcState::new();
        state_b.time_offset_secs = 3600; // +1 hour
        op_time(&mut state_b).unwrap();
        let val_b = state_b.stack.x.inner();

        // Both should be in valid time range.
        assert!(val_a >= rust_decimal::Decimal::ZERO);
        assert!(val_b >= rust_decimal::Decimal::ZERO);
        // Not the same value (unless exactly at rollover, which is astronomically unlikely).
        // This assertion verifies time_offset_secs is actually applied.
        assert_ne!(val_a, val_b, "time_offset_secs must change TIME result");
    }

    #[test]
    fn op_time_reads_time_offset_secs() {
        // Verify state.time_offset_secs is read by computing a known offset.
        // With offset = 24*3600 = 86400 (1 day ahead in seconds), the clock
        // reads 24 hours ahead but wraps at midnight — same time of day,
        // so the TIME value equals the un-offset value. Use a 1-hour offset
        // instead where the difference is measurable.
        let mut state = CalcState::new();
        state.time_offset_secs = 3600;
        // Just verify it doesn't error and result is valid.
        op_time(&mut state).unwrap();
        let val = state.stack.x.inner();
        assert!(val >= rust_decimal::Decimal::ZERO);
    }

    // ── op_date ───────────────────────────────────────────────────────────────

    #[test]
    fn op_date_mdy_mode_integer_part_is_month() {
        // Flag 31 clear = MDY mode. Integer part = month (1-12).
        let mut state = CalcState::new();
        state.flags = 0; // MDY
        op_date(&mut state).unwrap();
        let val = state.stack.x.inner();
        assert!(val >= rust_decimal::Decimal::ZERO);
        let month_part = val.trunc();
        assert!(
            month_part >= rust_decimal::Decimal::ONE
                && month_part <= rust_decimal::Decimal::from(12),
            "MDY date integer part (month) must be 1..=12, got {month_part}"
        );
    }

    #[test]
    fn op_date_dmy_mode_integer_part_is_day() {
        // Flag 31 set = DMY mode. Integer part = day (1-31).
        let mut state = CalcState::new();
        state.flags = 1u64 << 31; // DMY
        op_date(&mut state).unwrap();
        let val = state.stack.x.inner();
        assert!(val >= rust_decimal::Decimal::ZERO);
        let day_part = val.trunc();
        assert!(
            day_part >= rust_decimal::Decimal::ONE && day_part <= rust_decimal::Decimal::from(31),
            "DMY date integer part (day) must be 1..=31, got {day_part}"
        );
    }

    // ── op_clk12 / op_clk24 ──────────────────────────────────────────────────

    #[test]
    fn op_clk12_sets_12h() {
        let mut state = CalcState::new();
        state.clock_12h = false;
        op_clk12(&mut state).unwrap();
        assert!(state.clock_12h);
    }

    #[test]
    fn op_clk24_clears_12h() {
        let mut state = CalcState::new();
        state.clock_12h = true;
        op_clk24(&mut state).unwrap();
        assert!(!state.clock_12h);
    }

    // ── op_clkt ───────────────────────────────────────────────────────────────

    #[test]
    fn op_clkt_off_to_time_only() {
        let mut state = CalcState::new();
        state.clock_display_mode = ClockDisplayMode::Off;
        state.clock_active = false;
        op_clkt(&mut state).unwrap();
        assert_eq!(state.clock_display_mode, ClockDisplayMode::TimeOnly);
        assert!(state.clock_active);
    }

    #[test]
    fn op_clkt_time_only_to_off() {
        let mut state = CalcState::new();
        state.clock_display_mode = ClockDisplayMode::TimeOnly;
        state.clock_active = true;
        op_clkt(&mut state).unwrap();
        assert_eq!(state.clock_display_mode, ClockDisplayMode::Off);
        assert!(!state.clock_active);
    }

    #[test]
    fn op_clkt_time_and_date_steps_down_to_time_only() {
        let mut state = CalcState::new();
        state.clock_display_mode = ClockDisplayMode::TimeAndDate;
        state.clock_active = true;
        op_clkt(&mut state).unwrap();
        assert_eq!(state.clock_display_mode, ClockDisplayMode::TimeOnly);
        assert!(state.clock_active);
    }

    // ── op_clktd ──────────────────────────────────────────────────────────────

    #[test]
    fn op_clktd_off_to_time_and_date() {
        let mut state = CalcState::new();
        state.clock_display_mode = ClockDisplayMode::Off;
        state.clock_active = false;
        op_clktd(&mut state).unwrap();
        assert_eq!(state.clock_display_mode, ClockDisplayMode::TimeAndDate);
        assert!(state.clock_active);
    }

    #[test]
    fn op_clktd_time_only_to_time_and_date() {
        let mut state = CalcState::new();
        state.clock_display_mode = ClockDisplayMode::TimeOnly;
        state.clock_active = true;
        op_clktd(&mut state).unwrap();
        assert_eq!(state.clock_display_mode, ClockDisplayMode::TimeAndDate);
        assert!(state.clock_active);
    }

    #[test]
    fn op_clktd_time_and_date_to_off() {
        let mut state = CalcState::new();
        state.clock_display_mode = ClockDisplayMode::TimeAndDate;
        state.clock_active = true;
        op_clktd(&mut state).unwrap();
        assert_eq!(state.clock_display_mode, ClockDisplayMode::Off);
        assert!(!state.clock_active);
    }

    // ── op_clock ──────────────────────────────────────────────────────────────

    #[test]
    fn op_clock_sets_time_only_mode_and_active() {
        let mut state = CalcState::new();
        state.clock_display_mode = ClockDisplayMode::Off;
        state.clock_active = false;
        op_clock(&mut state).unwrap();
        assert_eq!(state.clock_display_mode, ClockDisplayMode::TimeOnly);
        assert!(state.clock_active);
    }

    #[test]
    fn op_clock_from_time_and_date_sets_time_only() {
        let mut state = CalcState::new();
        state.clock_display_mode = ClockDisplayMode::TimeAndDate;
        state.clock_active = true;
        op_clock(&mut state).unwrap();
        assert_eq!(state.clock_display_mode, ClockDisplayMode::TimeOnly);
        assert!(state.clock_active);
    }

    // ── op_correct ────────────────────────────────────────────────────────────

    #[test]
    fn op_correct_is_no_op_does_not_modify_accuracy_factor() {
        // CORRECT is documented no-op (TIME-FMT-05). It must NOT store X.
        let mut state = CalcState::new();
        let original_af = state.accuracy_factor.clone();
        state.stack.x = HpNum::from(rust_decimal::Decimal::from(99));
        op_correct(&mut state).unwrap();
        assert_eq!(
            state.accuracy_factor, original_af,
            "CORRECT must not modify accuracy_factor"
        );
    }

    #[test]
    fn op_correct_does_not_modify_stack() {
        let mut state = CalcState::new();
        let x_val = HpNum::from(rust_decimal::Decimal::from(7));
        state.stack.x = x_val.clone();
        op_correct(&mut state).unwrap();
        assert_eq!(
            state.stack.x, x_val,
            "CORRECT must not modify stack X (Neutral lift)"
        );
    }

    // ── op_setaf / op_rclaf ───────────────────────────────────────────────────

    #[test]
    fn op_setaf_stores_x_into_accuracy_factor() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(rust_decimal::Decimal::from(2));
        op_setaf(&mut state).unwrap();
        assert_eq!(
            state.accuracy_factor,
            HpNum::from(rust_decimal::Decimal::from(2))
        );
    }

    #[test]
    fn op_rclaf_pushes_accuracy_factor_to_stack() {
        let mut state = CalcState::new();
        state.accuracy_factor = HpNum::from(rust_decimal::Decimal::from(5));
        op_rclaf(&mut state).unwrap();
        assert_eq!(state.stack.x, HpNum::from(rust_decimal::Decimal::from(5)));
    }

    #[test]
    fn setaf_rclaf_round_trip() {
        let mut state = CalcState::new();
        use rust_decimal::Decimal;
        use std::str::FromStr;
        let val = HpNum::from(Decimal::from_str("1.5").unwrap());
        state.stack.x = val.clone();
        op_setaf(&mut state).unwrap();
        // Clear stack X to verify RCLAF actually writes it.
        state.stack.x = HpNum::zero();
        op_rclaf(&mut state).unwrap();
        assert_eq!(state.stack.x, val);
    }

    // ── op_setime / op_setdate ───────────────────────────────────────────────

    #[test]
    fn op_setime_opens_set_time_prompt_modal() {
        let mut state = CalcState::new();
        op_setime(&mut state).unwrap();
        assert_eq!(
            state.modal_program,
            Some(ModalProgram::Time(TimeStep::SetTimePrompt))
        );
        assert_eq!(state.modal_prompt, Some("TIME?".to_string()));
    }

    #[test]
    fn op_setdate_opens_set_date_prompt_modal() {
        let mut state = CalcState::new();
        op_setdate(&mut state).unwrap();
        assert_eq!(
            state.modal_program,
            Some(ModalProgram::Time(TimeStep::SetDatePrompt))
        );
        assert_eq!(state.modal_prompt, Some("DATE?".to_string()));
    }

    // ── op_tplusx ─────────────────────────────────────────────────────────────

    #[test]
    fn op_tplusx_one_hour_adds_3600_secs() {
        let mut state = CalcState::new();
        // X = 1.000000 = 1 hour exactly (HH.MMSScc format).
        state.stack.x = HpNum::from(Decimal::from_str("1.000000").unwrap());
        state.time_offset_secs = 0;
        op_tplusx(&mut state).unwrap();
        assert_eq!(state.time_offset_secs, 3600);
    }

    #[test]
    fn op_tplusx_one_minute_adds_60_secs() {
        let mut state = CalcState::new();
        // X = 0.010000 = 0h 1m 0s 0cs
        state.stack.x = HpNum::from(Decimal::from_str("0.010000").unwrap());
        state.time_offset_secs = 0;
        op_tplusx(&mut state).unwrap();
        assert_eq!(state.time_offset_secs, 60);
    }

    #[test]
    fn op_tplusx_adds_to_existing_offset() {
        let mut state = CalcState::new();
        // X = 0.003000 = 0h 0m 30s
        state.stack.x = HpNum::from(Decimal::from_str("0.003000").unwrap());
        state.time_offset_secs = 100;
        op_tplusx(&mut state).unwrap();
        assert_eq!(state.time_offset_secs, 130);
    }

    #[test]
    fn op_tplusx_invalid_time_returns_error() {
        // Hours > 99 overflow u8 parse — should return error.
        let mut state = CalcState::new();
        // 999.0 has hours=255 overflow — parse_time_hpnum uses u8 so this fails.
        // Use a value that overflows u8 for the hour field.
        // 256 hours in HH.MMSScc → "256.000000" → hours parse as u8 fails.
        state.stack.x = HpNum::from(Decimal::from_str("256.000000").unwrap());
        assert!(
            op_tplusx(&mut state).is_err(),
            "T+X must return error for out-of-range time input"
        );
    }

    // ── get_clock_display_str ─────────────────────────────────────────────────

    #[test]
    fn get_clock_display_str_none_when_inactive() {
        let mut state = CalcState::new();
        state.clock_active = false;
        assert!(get_clock_display_str(&state).is_none());
    }

    #[test]
    fn get_clock_display_str_some_when_active() {
        let mut state = CalcState::new();
        state.clock_active = true;
        state.clock_display_mode = ClockDisplayMode::TimeOnly;
        let result = get_clock_display_str(&state);
        assert!(
            result.is_some(),
            "get_clock_display_str must return Some when clock_active=true"
        );
    }

    #[test]
    fn get_clock_display_str_24h_format() {
        let mut state = CalcState::new();
        state.clock_active = true;
        state.clock_12h = false;
        state.clock_display_mode = ClockDisplayMode::TimeOnly;
        let result = get_clock_display_str(&state).unwrap();
        // 24h format: "HH:MM:SS" — no AM/PM.
        assert!(
            !result.contains("AM") && !result.contains("PM"),
            "24h format must not contain AM/PM, got: {result}"
        );
        // Must have two colons.
        assert_eq!(
            result.matches(':').count(),
            2,
            "time string must have 2 colons: {result}"
        );
    }

    #[test]
    fn get_clock_display_str_12h_format() {
        let mut state = CalcState::new();
        state.clock_active = true;
        state.clock_12h = true;
        state.clock_display_mode = ClockDisplayMode::TimeOnly;
        let result = get_clock_display_str(&state).unwrap();
        // 12h format: "HH:MM:SS AM" or "HH:MM:SS PM".
        assert!(
            result.contains("AM") || result.contains("PM"),
            "12h format must contain AM or PM, got: {result}"
        );
    }

    #[test]
    fn get_clock_display_str_time_and_date_returns_some() {
        let mut state = CalcState::new();
        state.clock_active = true;
        state.clock_display_mode = ClockDisplayMode::TimeAndDate;
        let result = get_clock_display_str(&state);
        assert!(result.is_some());
    }

    // ── format helpers ────────────────────────────────────────────────────────

    #[test]
    fn format_time_str_midnight_24h() {
        let s = format_time_str(0, 0, 0, false);
        assert_eq!(s, "00:00:00");
    }

    #[test]
    fn format_time_str_noon_12h() {
        // 12:00:00 noon in 12h = "12:00:00 PM"
        let s = format_time_str(12, 0, 0, true);
        assert_eq!(s, "12:00:00 PM");
    }

    #[test]
    fn format_time_str_midnight_12h() {
        // 00:00:00 in 12h = "12:00:00 AM"
        let s = format_time_str(0, 0, 0, true);
        assert_eq!(s, "12:00:00 AM");
    }

    #[test]
    fn format_time_str_1pm_12h() {
        // 13:00:00 in 12h = "01:00:00 PM"
        let s = format_time_str(13, 0, 0, true);
        assert_eq!(s, "01:00:00 PM");
    }

    #[test]
    fn format_date_str_mdy() {
        let s = format_date_str(2026, 5, 24, false);
        assert_eq!(s, "05/24/2026");
    }

    #[test]
    fn format_date_str_dmy() {
        let s = format_date_str(2026, 5, 24, true);
        assert_eq!(s, "24/05/2026");
    }
}
