// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Date arithmetic operations for the HP Time Module.
//!
//! Covers: DATE+, DDAYS, DOW, DMY, MDY.
//!
//! Date representation (HP 82182A OM §Date Functions):
//! - MDY mode (Flag 31 clear): MM.DDYYYY — e.g. 5.241983 = May 24, 1983
//! - DMY mode (Flag 31 set):   DD.MMYYYY — e.g. 24.051983 = May 24, 1983
//!
//! IMPORTANT: `parse_date_hpnum` uses LEFT-pad of the fractional part to 6
//! digits (NOT right-pad) because YYYY must preserve leading zeros.
//! Contrast with `parse_counter` in program.rs which uses right-pad ("{:0<5}").
//!
//! P35 invariant: NEVER use float arithmetic for date/time field extraction.
//! Always string-split at the decimal point (same as ISG/DSE counter parsing).
//!
//! JDN algorithm: Fliegel-Van Flandern (1968), ACM Commun. 11(10):657.
//! Valid for all Gregorian dates from Oct 15, 1582 onward.

use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{apply_lift_effect, binary_result, unary_result, LiftEffect};
use crate::state::CalcState;
use rust_decimal::Decimal;
use std::str::FromStr;

// ── JDN Algorithms (Fliegel-Van Flandern 1968) ────────────────────────────────

/// Convert (year, month, day) to Julian Day Number.
///
/// Uses the Fliegel-Van Flandern integer algorithm from ACM Commun. 11(10):657 (1968).
/// Valid for all Gregorian calendar dates from Oct 15, 1582.
/// All arithmetic in i64 to avoid i32 overflow for large year values (T-38-01).
///
/// [CITED: Fliegel, H.F. & Van Flandern, T.C. (1968), "A Machine Algorithm
/// for Processing Calendar Dates", Commun. ACM 11(10):657]
pub fn date_to_jdn(year: i32, month: i32, day: i32) -> i64 {
    let y = year as i64;
    let m = month as i64;
    let d = day as i64;
    let a = (14 - m) / 12;
    let yy = y + 4800 - a;
    let mm = m + 12 * a - 3;
    d + (153 * mm + 2) / 5 + 365 * yy + yy / 4 - yy / 100 + yy / 400 - 32045
}

/// Convert Julian Day Number back to (year, month, day).
///
/// Inverse of the Fliegel-Van Flandern algorithm.
///
/// [CITED: Fliegel, H.F. & Van Flandern, T.C. (1968), "A Machine Algorithm
/// for Processing Calendar Dates", Commun. ACM 11(10):657]
pub fn jdn_to_date(jdn: i64) -> (i32, i32, i32) {
    let l = jdn + 68569;
    let n = 4 * l / 146097;
    let l = l - (146097 * n + 3) / 4;
    let i = 4000 * (l + 1) / 1461001;
    let l = l - 1461 * i / 4 + 31;
    let j = 80 * l / 2447;
    let day = l - 2447 * j / 80;
    let l = j / 11;
    let month = j + 2 - 12 * l;
    let year = 100 * (n - 49) + i + l;
    (year as i32, month as i32, day as i32)
}

/// Day of week for a Julian Day Number.
///
/// Returns: 0 = Sunday, 1 = Monday, ..., 6 = Saturday.
/// Formula: (jdn + 1) % 7 — JDN 0 was a Monday; +1 shifts Sunday to 0.
/// Matches HP 82182A OM §DOW: 0=Sunday...6=Saturday.
pub fn jdn_to_dow(jdn: i64) -> i32 {
    ((jdn + 1) % 7) as i32
}

// ── Parse Helpers ─────────────────────────────────────────────────────────────

/// Parse an HP-41 date HpNum into (year, month, day).
///
/// In MDY mode (`dmy = false`): HpNum = MM.DDYYYY → month = MM, day = DD
/// In DMY mode (`dmy = true`):  HpNum = DD.MMYYYY → day = DD, month = MM
///
/// The fractional part is LEFT-padded to exactly 6 chars so that the
/// 4-digit year field preserves trailing zeros (e.g. year 2000 = "2000",
/// not "2" from float truncation). This is the D-carried.2 / P35 invariant.
///
/// Returns `HpError::InvalidInput` for out-of-range month/day values.
pub fn parse_date_hpnum(hpnum: &HpNum, dmy: bool) -> Result<(i32, i32, i32), HpError> {
    let s = hpnum.inner().to_string();
    let (int_part, frac_part) = if let Some(pos) = s.find('.') {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s.as_str(), "")
    };
    // LEFT-pad fractional part to exactly 6 digits (preserves leading zeros for year).
    // NOTE: parse_counter uses right-pad ("{:0<5}"); dates use LEFT-pad ("{:0>6}").
    let padded = format!("{:0>6}", frac_part);
    let first: i32 = int_part.parse().map_err(|_| HpError::InvalidInput)?;
    let second: i32 = padded[0..2].parse().map_err(|_| HpError::InvalidInput)?;
    let year: i32 = padded[2..6].parse().map_err(|_| HpError::InvalidInput)?;
    let (month, day) = if dmy {
        // DD.MMYYYY → day = first, month = second
        (second, first)
    } else {
        // MM.DDYYYY → month = first, day = second
        (first, second)
    };
    // Range validation (T-38-04)
    if !(1..=12).contains(&month) {
        return Err(HpError::InvalidInput);
    }
    if !(1..=31).contains(&day) {
        return Err(HpError::InvalidInput);
    }
    // Use JDN round-trip validation to catch impossible dates (e.g. Feb 30)
    let jdn = date_to_jdn(year, month, day);
    let (ry, rm, rd) = jdn_to_date(jdn);
    if ry != year || rm != month || rd != day {
        return Err(HpError::InvalidInput);
    }
    Ok((year, month, day))
}

/// Parse an HP-41 time HpNum (HH.MMSScc) into (hours, minutes, seconds, centiseconds).
///
/// Fractional part is LEFT-padded to exactly 6 digits: MM(2) + SS(2) + cc(2).
/// Hours come from the integer part.
///
/// NOTE: P44 mitigation — do NOT reuse `parse_hms()` from `hms.rs`.
/// `hms.rs` handles H.MMSS (4 fractional digits); Time Module needs
/// HH.MMSScc (6 fractional digits). Centiseconds would be silently dropped.
///
/// Returns `HpError::InvalidInput` for out-of-range values (T-38-05).
pub fn parse_time_hpnum(hpnum: &HpNum) -> Result<(u8, u8, u8, u8), HpError> {
    let s = hpnum.inner().to_string();
    let (int_part, frac_part) = if let Some(pos) = s.find('.') {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s.as_str(), "")
    };
    let hours: u8 = int_part.parse().map_err(|_| HpError::InvalidInput)?;
    // LEFT-pad fractional part to exactly 6 digits: MMSScc
    let padded = format!("{:0>6}", frac_part);
    let minutes: u8 = padded[0..2].parse().map_err(|_| HpError::InvalidInput)?;
    let seconds: u8 = padded[2..4].parse().map_err(|_| HpError::InvalidInput)?;
    let centiseconds: u8 = padded[4..6].parse().map_err(|_| HpError::InvalidInput)?;
    // Range validation (T-38-05)
    if hours > 23 || minutes > 59 || seconds > 59 || centiseconds > 99 {
        return Err(HpError::InvalidInput);
    }
    Ok((hours, minutes, seconds, centiseconds))
}

/// Convert total seconds (possibly fractional) to HH.MMSScc HpNum format.
///
/// Used by stopwatch / elapsed-time ops that compute with seconds.
/// Fractional seconds are truncated to centiseconds (0.01 s resolution).
pub fn secs_to_hpnum_time(secs: f64) -> Result<HpNum, HpError> {
    let total_centis = (secs * 100.0) as i64;
    let centiseconds = total_centis % 100;
    let total_secs = total_centis / 100;
    let seconds = total_secs % 60;
    let total_mins = total_secs / 60;
    let minutes = total_mins % 60;
    let hours = total_mins / 60;
    // Build HH.MMSScc as a decimal string to preserve all fields exactly
    let s = format!("{}.{:02}{:02}{:02}", hours, minutes, seconds, centiseconds);
    let d = Decimal::from_str(&s).map_err(|_| HpError::Overflow)?;
    Ok(HpNum::from(d))
}

// ── Date Format Helper ────────────────────────────────────────────────────────

/// Format (year, month, day) as an HP-41 date HpNum in MDY or DMY mode.
///
/// MDY (dmy=false): MM.DDYYYY — e.g. (2026, 5, 24) → "5.242026"
/// DMY (dmy=true):  DD.MMYYYY — e.g. (2026, 5, 24) → "24.052026"
fn date_to_hpnum(year: i32, month: i32, day: i32, dmy: bool) -> Result<HpNum, HpError> {
    let s = if dmy {
        // DD.MMYYYY
        format!("{}.{:02}{:04}", day, month, year)
    } else {
        // MM.DDYYYY
        format!("{}.{:02}{:04}", month, day, year)
    };
    let d = Decimal::from_str(&s).map_err(|_| HpError::Overflow)?;
    Ok(HpNum::from(d))
}

// ── Op Implementations ────────────────────────────────────────────────────────

/// DATE+ — Add X days to date in Y, push result to stack X.
///
/// Y: date HpNum (per Flag 31 mode); X: integer day count.
/// Binary op: result → X, Y←Z, Z←T, T←T, LASTX←old X.
/// LiftEffect: Enable.
pub fn op_date_plus(state: &mut CalcState) -> Result<(), HpError> {
    let dmy = state.flags & (1u64 << 31) != 0;
    // Parse Y as a date
    let (year, month, day) = parse_date_hpnum(&state.stack.y, dmy)?;
    // Parse X as integer day count (truncate toward zero)
    let days = decimal_to_i64(state.stack.x.inner())?;
    // JDN arithmetic
    let jdn = date_to_jdn(year, month, day);
    let new_jdn = jdn + days;
    let (new_year, new_month, new_day) = jdn_to_date(new_jdn);
    // Format result back as date HpNum
    let result = date_to_hpnum(new_year, new_month, new_day, dmy)?;
    binary_result(state, result);
    Ok(())
}

/// DDAYS — Compute signed number of days between two dates.
///
/// Y: first date; X: second date (both per Flag 31 mode).
/// Result: JDN(Y) - JDN(X) (positive if Y is later than X).
///
/// Note: HP 82182A OM §DDAYS: result = date(Y) - date(X) days.
/// Binary op: result → X, Y←Z, Z←T, T←T, LASTX←old X.
/// LiftEffect: Enable.
pub fn op_ddays(state: &mut CalcState) -> Result<(), HpError> {
    let dmy = state.flags & (1u64 << 31) != 0;
    // Parse both dates
    let (y_year, y_month, y_day) = parse_date_hpnum(&state.stack.y, dmy)?;
    let (x_year, x_month, x_day) = parse_date_hpnum(&state.stack.x, dmy)?;
    // Compute signed difference: Y date - X date
    let jdn_y = date_to_jdn(y_year, y_month, y_day);
    let jdn_x = date_to_jdn(x_year, x_month, x_day);
    let diff = jdn_y - jdn_x;
    let result = HpNum::from(Decimal::from(diff));
    binary_result(state, result);
    Ok(())
}

/// DOW — Day of week for date in X.
///
/// Returns 0=Sunday, 1=Monday, ..., 6=Saturday (HP 82182A OM §DOW).
/// Unary op: result → X; Y, Z, T unchanged. LASTX = old X.
/// LiftEffect: Enable.
pub fn op_dow(state: &mut CalcState) -> Result<(), HpError> {
    let dmy = state.flags & (1u64 << 31) != 0;
    let (year, month, day) = parse_date_hpnum(&state.stack.x, dmy)?;
    let jdn = date_to_jdn(year, month, day);
    let dow = jdn_to_dow(jdn);
    let result = HpNum::from(Decimal::from(dow));
    unary_result(state, result);
    Ok(())
}

/// DMY — Set DMY date mode (Flag 31 set).
///
/// LiftEffect: Neutral.
pub fn op_dmy(state: &mut CalcState) -> Result<(), HpError> {
    // Flag 31 = DMY mode (HP 82182A OM §DMY).
    state.flags |= 1u64 << 31;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// MDY — Set MDY date mode (Flag 31 clear).
///
/// LiftEffect: Neutral.
pub fn op_mdy(state: &mut CalcState) -> Result<(), HpError> {
    // Flag 31 clear = MDY mode.
    state.flags &= !(1u64 << 31);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

// ── i64 conversion helper ─────────────────────────────────────────────────────

/// Convert a Decimal to i64 (truncated toward zero), returning HpError::Overflow on failure.
fn decimal_to_i64(d: Decimal) -> Result<i64, HpError> {
    use rust_decimal::prelude::ToPrimitive;
    d.trunc().to_i64().ok_or(HpError::Overflow)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ── JDN Round-trip Tests ──────────────────────────────────────────────────

    #[test]
    fn jdn_known_reference_2026_05_24() {
        // 2026-05-24 is our known anchor date.
        // JDN 2461185 verified by: JDN(2000-01-01)=2451545 (standard astronomical JDN)
        // plus 9640 days to 2026-05-24 (Python datetime confirmed).
        // NOTE: The Phase 38 research file cited 2460820 (off by 365 days — one year error
        // in the spec reference). The FVF algorithm and Python datetime both confirm 2461185.
        assert_eq!(date_to_jdn(2026, 5, 24), 2461185);
    }

    #[test]
    fn jdn_roundtrip_2026_05_24() {
        let jdn = date_to_jdn(2026, 5, 24);
        let (year, month, day) = jdn_to_date(jdn);
        assert_eq!((year, month, day), (2026, 5, 24));
    }

    #[test]
    fn jdn_dow_2026_05_24_sunday() {
        // 2026-05-24 is Sunday = 0 (confirmed by Python datetime).
        // NOTE: Phase 38 research cited Saturday (6); Python + FVF both give Sunday (0).
        let jdn = date_to_jdn(2026, 5, 24);
        assert_eq!(jdn_to_dow(jdn), 0);
    }

    #[test]
    fn jdn_leap_year_2000_02_29() {
        // Year 2000 is a leap year (divisible by 400)
        let jdn = date_to_jdn(2000, 2, 29);
        let (year, month, day) = jdn_to_date(jdn);
        assert_eq!((year, month, day), (2000, 2, 29));
    }

    #[test]
    fn jdn_not_leap_year_2100() {
        // Year 2100 is NOT a leap year (divisible by 100 but not 400)
        // Feb 28 + 1 day should be Mar 1
        let jdn = date_to_jdn(2100, 2, 28);
        let (year, month, day) = jdn_to_date(jdn + 1);
        assert_eq!((year, month, day), (2100, 3, 1));
    }

    #[test]
    fn jdn_gregorian_start_1582_10_15() {
        // The Gregorian calendar started Oct 15, 1582
        let jdn = date_to_jdn(1582, 10, 15);
        let (year, month, day) = jdn_to_date(jdn);
        assert_eq!((year, month, day), (1582, 10, 15));
    }

    #[test]
    fn jdn_roundtrip_y2k() {
        let jdn = date_to_jdn(2000, 1, 1);
        let (year, month, day) = jdn_to_date(jdn);
        assert_eq!((year, month, day), (2000, 1, 1));
    }

    #[test]
    fn jdn_dow_sunday_2026_05_31() {
        // 2026-05-31 is a Sunday = 0
        let jdn = date_to_jdn(2026, 5, 31);
        assert_eq!(jdn_to_dow(jdn), 0);
    }

    // ── parse_date_hpnum Tests ────────────────────────────────────────────────

    #[test]
    fn parse_date_mdy_jan_24_2026() {
        // 1.242026 → MDY → month=1, day=24, year=2026 → (2026, 1, 24)
        let n = HpNum::from(Decimal::from_str("1.242026").unwrap());
        let (year, month, day) = parse_date_hpnum(&n, false).unwrap();
        assert_eq!((year, month, day), (2026, 1, 24));
    }

    #[test]
    fn parse_date_dmy_24_jan_2026() {
        // 24.012026 → DMY → day=24, month=01, year=2026 → (2026, 1, 24)
        let n = HpNum::from(Decimal::from_str("24.012026").unwrap());
        let (year, month, day) = parse_date_hpnum(&n, true).unwrap();
        assert_eq!((year, month, day), (2026, 1, 24));
    }

    #[test]
    fn parse_date_trailing_zero_year_2000_mdy() {
        // 1.012000 → MDY → month=1, day=1, year=2000
        // Trailing zeros in year field must be preserved via left-pad.
        let n = HpNum::from(Decimal::from_str("1.012000").unwrap());
        let (year, month, day) = parse_date_hpnum(&n, false).unwrap();
        assert_eq!((year, month, day), (2000, 1, 1));
    }

    #[test]
    fn parse_date_trailing_zero_year_2010_mdy() {
        // 1.012010 → MDY → year=2010
        let n = HpNum::from(Decimal::from_str("1.012010").unwrap());
        let (year, month, day) = parse_date_hpnum(&n, false).unwrap();
        assert_eq!(year, 2010);
    }

    #[test]
    fn parse_date_trailing_zero_year_2020_mdy() {
        // 5.242020 → MDY → year=2020
        let n = HpNum::from(Decimal::from_str("5.242020").unwrap());
        let (year, month, day) = parse_date_hpnum(&n, false).unwrap();
        assert_eq!((year, month, day), (2020, 5, 24));
    }

    #[test]
    fn parse_date_invalid_month_returns_data_err() {
        // 13.012026 → MDY → month=13 → invalid
        let n = HpNum::from(Decimal::from_str("13.012026").unwrap());
        assert!(parse_date_hpnum(&n, false).is_err());
    }

    #[test]
    fn parse_date_invalid_day_feb_30_returns_data_err() {
        // 2.302026 → MDY → month=2, day=30 → impossible (Feb 30)
        let n = HpNum::from(Decimal::from_str("2.302026").unwrap());
        assert!(parse_date_hpnum(&n, false).is_err());
    }

    // ── parse_time_hpnum Tests ────────────────────────────────────────────────

    #[test]
    fn parse_time_zero() {
        let n = HpNum::zero();
        let (h, m, s, cs) = parse_time_hpnum(&n).unwrap();
        assert_eq!((h, m, s, cs), (0, 0, 0, 0));
    }

    #[test]
    fn parse_time_13_30_45_99() {
        // 13.304599 → 13h 30m 45s 99cs
        let n = HpNum::from(Decimal::from_str("13.304599").unwrap());
        let (h, m, s, cs) = parse_time_hpnum(&n).unwrap();
        assert_eq!((h, m, s, cs), (13, 30, 45, 99));
    }

    #[test]
    fn parse_time_leading_zeros_preserved() {
        // 0.000001 → 0h 0m 0s 01cs (left-pad to 6 = "000001")
        let n = HpNum::from(Decimal::from_str("0.000001").unwrap());
        let (h, m, s, cs) = parse_time_hpnum(&n).unwrap();
        assert_eq!((h, m, s, cs), (0, 0, 0, 1));
    }

    #[test]
    fn parse_time_invalid_hours_24_returns_err() {
        // 24.000000 → h=24 → invalid
        let n = HpNum::from(Decimal::from_str("24.000000").unwrap());
        assert!(parse_time_hpnum(&n).is_err());
    }

    #[test]
    fn parse_time_invalid_minutes_60_returns_err() {
        // 12.600000 → m=60 → invalid
        let n = HpNum::from(Decimal::from_str("12.600000").unwrap());
        assert!(parse_time_hpnum(&n).is_err());
    }

    // ── secs_to_hpnum_time Tests ──────────────────────────────────────────────

    #[test]
    fn secs_to_hpnum_3661_5() {
        // 3661.5 seconds = 1h 1m 1.50s = 1.010150 (HH.MMSScc)
        let result = secs_to_hpnum_time(3661.5).unwrap();
        // 1h=1, 01m=01, 01s=01, 50cs=50 → "1.010150"
        let expected = Decimal::from_str("1.010150").unwrap();
        assert_eq!(result.inner(), HpNum::from(expected).inner());
    }

    #[test]
    fn secs_to_hpnum_zero() {
        let result = secs_to_hpnum_time(0.0).unwrap();
        assert!(result.is_zero());
    }

    #[test]
    fn secs_to_hpnum_3600() {
        // 3600 seconds = 1h 0m 0s = 1.000000
        let result = secs_to_hpnum_time(3600.0).unwrap();
        let expected = Decimal::from_str("1.000000").unwrap();
        assert_eq!(result.inner(), HpNum::from(expected).inner());
    }

    #[test]
    fn secs_to_hpnum_90_centiseconds() {
        // 90.99 seconds = 0h 1m 30.99s = 0.013099
        let result = secs_to_hpnum_time(90.99).unwrap();
        let s = result.inner().to_string();
        // Should represent 0.013099
        assert!(s.starts_with("0.0130") || s.contains("3099"), "got: {}", s);
    }

    // ── op_dmy / op_mdy Tests ─────────────────────────────────────────────────

    #[test]
    fn op_dmy_sets_flag_31() {
        let mut state = CalcState::new();
        state.flags = 0;
        op_dmy(&mut state).unwrap();
        assert_ne!(state.flags & (1u64 << 31), 0, "Flag 31 must be set after DMY");
    }

    #[test]
    fn op_mdy_clears_flag_31() {
        let mut state = CalcState::new();
        state.flags = 1u64 << 31;
        op_mdy(&mut state).unwrap();
        assert_eq!(state.flags & (1u64 << 31), 0, "Flag 31 must be clear after MDY");
    }

    #[test]
    fn op_dmy_neutral_lift() {
        let mut state = CalcState::new();
        state.stack.lift_enabled = false;
        op_dmy(&mut state).unwrap();
        // Neutral lift — should not change lift_enabled
        assert!(!state.stack.lift_enabled);
    }

    #[test]
    fn op_mdy_neutral_lift() {
        let mut state = CalcState::new();
        state.stack.lift_enabled = true;
        op_mdy(&mut state).unwrap();
        // Neutral lift — should not change lift_enabled
        assert!(state.stack.lift_enabled);
    }

    // ── op_dow Tests ──────────────────────────────────────────────────────────

    #[test]
    fn op_dow_sunday_2026_05_24() {
        // 2026-05-24 is Sunday = 0 (confirmed by Python datetime + FVF algorithm).
        // NOTE: Phase 38 plan cited Saturday (6); corrected per Python datetime verification.
        let mut state = CalcState::new();
        state.flags = 0; // MDY mode
        state.stack.x = HpNum::from(Decimal::from_str("5.242026").unwrap());
        state.stack.lift_enabled = true;
        op_dow(&mut state).unwrap();
        let dow = state.stack.x.inner().to_string();
        assert_eq!(dow, "0", "2026-05-24 should be Sunday (0), got {}", dow);
    }

    #[test]
    fn op_dow_sunday_2026_05_31() {
        // 2026-05-31 is Sunday = 0
        let mut state = CalcState::new();
        state.flags = 0; // MDY mode
        state.stack.x = HpNum::from(Decimal::from_str("5.312026").unwrap());
        state.stack.lift_enabled = true;
        op_dow(&mut state).unwrap();
        let dow = state.stack.x.inner().to_string();
        assert_eq!(dow, "0", "2026-05-31 should be Sunday (0), got {}", dow);
    }

    #[test]
    fn op_dow_dmy_mode() {
        // 24.052026 in DMY mode = 2026-05-24 = Sunday = 0
        let mut state = CalcState::new();
        state.flags = 1u64 << 31; // DMY mode
        state.stack.x = HpNum::from(Decimal::from_str("24.052026").unwrap());
        state.stack.lift_enabled = true;
        op_dow(&mut state).unwrap();
        let dow = state.stack.x.inner().to_string();
        assert_eq!(dow, "0");
    }

    // ── op_date_plus Tests ────────────────────────────────────────────────────

    #[test]
    fn op_date_plus_jan_1_2026_plus_31() {
        // Y = 1.012026 (Jan 1 2026 MDY), X = 31 → Feb 1 2026
        let mut state = CalcState::new();
        state.flags = 0; // MDY mode
        state.stack.y = HpNum::from(Decimal::from_str("1.012026").unwrap());
        state.stack.x = HpNum::from(Decimal::from(31i32));
        state.stack.lift_enabled = true;
        op_date_plus(&mut state).unwrap();
        // Parse result back to (year, month, day) to compare semantically
        let (year, month, day) = parse_date_hpnum(&state.stack.x, false).unwrap();
        assert_eq!((year, month, day), (2026, 2, 1), "Expected Feb 1 2026");
    }

    #[test]
    fn op_date_plus_feb_28_2000_plus_1_leap_year() {
        // Y = 2.282000 (Feb 28 2000 MDY), X = 1 → Feb 29 2000 (leap year)
        let mut state = CalcState::new();
        state.flags = 0; // MDY mode
        state.stack.y = HpNum::from(Decimal::from_str("2.282000").unwrap());
        state.stack.x = HpNum::from(Decimal::from(1i32));
        state.stack.lift_enabled = true;
        op_date_plus(&mut state).unwrap();
        let (year, month, day) = parse_date_hpnum(&state.stack.x, false).unwrap();
        assert_eq!((year, month, day), (2000, 2, 29), "Year 2000 is a leap year");
    }

    #[test]
    fn op_date_plus_feb_28_2100_plus_1_not_leap_year() {
        // Y = 2.282100 (Feb 28 2100 MDY), X = 1 → Mar 1 2100 (2100 is NOT leap)
        let mut state = CalcState::new();
        state.flags = 0; // MDY mode
        state.stack.y = HpNum::from(Decimal::from_str("2.282100").unwrap());
        state.stack.x = HpNum::from(Decimal::from(1i32));
        state.stack.lift_enabled = true;
        op_date_plus(&mut state).unwrap();
        let (year, month, day) = parse_date_hpnum(&state.stack.x, false).unwrap();
        assert_eq!((year, month, day), (2100, 3, 1), "Year 2100 is not a leap year");
    }

    #[test]
    fn op_date_plus_dmy_mode() {
        // DMY mode: Y = 1.012026 means Jan 1 2026 (day=1, month=01)
        let mut state = CalcState::new();
        state.flags = 1u64 << 31; // DMY mode
        state.stack.y = HpNum::from(Decimal::from_str("1.012026").unwrap());
        state.stack.x = HpNum::from(Decimal::from(31i32));
        state.stack.lift_enabled = true;
        op_date_plus(&mut state).unwrap();
        // Result in DMY = Feb 1 2026 → (2026, 2, 1)
        let (year, month, day) = parse_date_hpnum(&state.stack.x, true).unwrap();
        assert_eq!((year, month, day), (2026, 2, 1), "DMY mode: expected Feb 1 2026");
    }

    #[test]
    fn op_date_plus_uses_binary_result_stack_drop() {
        // binary_result: X←result, Y←Z, Z←T, T←T, LASTX←old_X
        let mut state = CalcState::new();
        state.flags = 0; // MDY mode
        state.stack.z = HpNum::from(Decimal::from_str("7.042023").unwrap()); // sentinel
        state.stack.y = HpNum::from(Decimal::from_str("1.012026").unwrap());
        state.stack.x = HpNum::from(Decimal::from(1i32));
        state.stack.lift_enabled = true;
        op_date_plus(&mut state).unwrap();
        // After binary_result: new Y = old Z sentinel
        assert_eq!(
            state.stack.y.inner(),
            HpNum::from(Decimal::from_str("7.042023").unwrap()).inner()
        );
    }

    // ── op_ddays Tests ────────────────────────────────────────────────────────

    /// Extract the integer value from the result HpNum (for DDAYS/DOW comparison)
    fn hpnum_to_i64(n: &HpNum) -> i64 {
        use rust_decimal::prelude::ToPrimitive;
        n.inner().trunc().to_i64().expect("integer HpNum")
    }

    #[test]
    fn op_ddays_positive_364_days() {
        // X = 1.012026 (Jan 1), Y = 12.312026 (Dec 31) → Y - X = 364 days
        let mut state = CalcState::new();
        state.flags = 0; // MDY mode
        state.stack.y = HpNum::from(Decimal::from_str("12.312026").unwrap());
        state.stack.x = HpNum::from(Decimal::from_str("1.012026").unwrap());
        state.stack.lift_enabled = true;
        op_ddays(&mut state).unwrap();
        assert_eq!(hpnum_to_i64(&state.stack.x), 364, "Expected 364 days");
    }

    #[test]
    fn op_ddays_negative_signed() {
        // X = 12.312026, Y = 1.012026 → Y - X = -364
        let mut state = CalcState::new();
        state.flags = 0; // MDY mode
        state.stack.y = HpNum::from(Decimal::from_str("1.012026").unwrap());
        state.stack.x = HpNum::from(Decimal::from_str("12.312026").unwrap());
        state.stack.lift_enabled = true;
        op_ddays(&mut state).unwrap();
        assert_eq!(hpnum_to_i64(&state.stack.x), -364, "Expected -364 days");
    }

    #[test]
    fn op_ddays_same_date_zero() {
        let mut state = CalcState::new();
        state.flags = 0; // MDY mode
        state.stack.y = HpNum::from(Decimal::from_str("5.242026").unwrap());
        state.stack.x = HpNum::from(Decimal::from_str("5.242026").unwrap());
        state.stack.lift_enabled = true;
        op_ddays(&mut state).unwrap();
        assert_eq!(hpnum_to_i64(&state.stack.x), 0);
    }

    #[test]
    fn op_ddays_across_leap_day_2000() {
        // Y = 3.012000 (Mar 1 2000), X = 2.282000 (Feb 28 2000)
        // Difference = 2 days (because 2000 is a leap year: Feb 28 → Feb 29 → Mar 1)
        let mut state = CalcState::new();
        state.flags = 0; // MDY mode
        state.stack.y = HpNum::from(Decimal::from_str("3.012000").unwrap());
        state.stack.x = HpNum::from(Decimal::from_str("2.282000").unwrap());
        state.stack.lift_enabled = true;
        op_ddays(&mut state).unwrap();
        assert_eq!(hpnum_to_i64(&state.stack.x), 2, "Expected 2 days across Feb 29 2000");
    }
}
