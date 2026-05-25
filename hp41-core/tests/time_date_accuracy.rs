// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Oracle-verified date arithmetic accuracy suite for TIME-QUAL-02.
//!
//! Coverage:
//! - DATE+, DDAYS, DOW via `dispatch(Op::TimeDatePlus)`, `dispatch(Op::TimeDdays)`,
//!   `dispatch(Op::TimeDow)`
//! - Leap year 2000 (divisible by 400 — IS leap), century 2100 (divisible by 100
//!   but NOT 400 — NOT leap), century 2400 (divisible by 400 — IS leap)
//! - Gregorian calendar start: Oct 15, 1582 (first valid Gregorian date)
//! - Y2K boundary, large offset, DMY mode, same-date DDAYS, negative DDAYS,
//!   DATE+ with negative day count, known historical DOW values
//!
//! Oracle source (D-42.11): Python `datetime` module used for derivation;
//! Meeus "Astronomical Algorithms" JDN tables used as sanity-check for boundaries.
//! All assertions are exact-match (assert_eq!) per D-42.4 — calendar arithmetic
//! is deterministic integer math with no floating-point tolerance required.
#![allow(clippy::unwrap_used)]

use hp41_core::ops::time::date_arith::{date_to_jdn, jdn_to_date, jdn_to_dow, parse_date_hpnum};
use hp41_core::ops::{dispatch, Op};
use hp41_core::state::CalcState;
use hp41_core::HpNum;
use rust_decimal::Decimal;
use std::str::FromStr;

// ── Helper: extract Decimal to i64 (for DDAYS / DOW) ────────────────────────

fn hpnum_to_i64(n: &HpNum) -> i64 {
    use rust_decimal::prelude::ToPrimitive;
    n.inner().trunc().to_i64().expect("integer HpNum value")
}

// ── Helper: set up state for DATE+ (Y=date, X=days, MDY/DMY via flag 31) ────

fn setup_date_state(y_date_str: &str, x_val_str: &str, dmy: bool) -> CalcState {
    let mut state = CalcState::new();
    state.flags = if dmy { 1u64 << 31 } else { 0 };
    state.stack.y = HpNum::from(Decimal::from_str(y_date_str).unwrap());
    state.stack.x = HpNum::from(Decimal::from_str(x_val_str).unwrap());
    state.stack.lift_enabled = true;
    state
}

// ── Helper: set up state for DDAYS (Y=date1, X=date2) ───────────────────────

fn setup_ddays_state(y_date_str: &str, x_date_str: &str, dmy: bool) -> CalcState {
    let mut state = CalcState::new();
    state.flags = if dmy { 1u64 << 31 } else { 0 };
    state.stack.y = HpNum::from(Decimal::from_str(y_date_str).unwrap());
    state.stack.x = HpNum::from(Decimal::from_str(x_date_str).unwrap());
    state.stack.lift_enabled = true;
    state
}

// ── Helper: set up state for DOW (X=date) ───────────────────────────────────

fn setup_dow_state(date_str: &str, dmy: bool) -> CalcState {
    let mut state = CalcState::new();
    state.flags = if dmy { 1u64 << 31 } else { 0 };
    state.stack.x = HpNum::from(Decimal::from_str(date_str).unwrap());
    state.stack.lift_enabled = true;
    state
}

// ═══════════════════════════════════════════════════════════════════════════
// SECTION 1: Leap year tests (Year 2000 — IS leap, divisible by 400)
// Oracle: Python datetime(2000, 2, 29) exists; datetime(2000, 3, 1) = Feb29+1
// ═══════════════════════════════════════════════════════════════════════════

/// DATE+: Feb 28 2000 + 1 = Feb 29 2000 (year 2000 IS a leap year)
#[test]
fn date_accuracy_01_date_plus_feb28_2000_plus1_leap() {
    let mut state = setup_date_state("2.282000", "1", false);
    dispatch(&mut state, Op::TimeDatePlus).unwrap();
    let (year, month, day) = parse_date_hpnum(&state.stack.x, false).unwrap();
    assert_eq!((year, month, day), (2000, 2, 29), "Year 2000 is a leap year; Feb 28 + 1 = Feb 29");
}

/// DATE+: Feb 29 2000 + 1 = Mar 1 2000 (day after leap day in year 2000)
#[test]
fn date_accuracy_02_date_plus_feb29_2000_plus1() {
    let mut state = setup_date_state("2.292000", "1", false);
    dispatch(&mut state, Op::TimeDatePlus).unwrap();
    let (year, month, day) = parse_date_hpnum(&state.stack.x, false).unwrap();
    assert_eq!((year, month, day), (2000, 3, 1), "Feb 29 2000 + 1 = Mar 1 2000");
}

/// DOW: Feb 29 2000 = Tuesday = 2
/// Oracle: Python datetime(2000, 2, 29).weekday() = 1 (Mon=0) → HP 0=Sun → Tuesday=2
#[test]
fn date_accuracy_03_dow_feb29_2000_tuesday() {
    let mut state = setup_dow_state("2.292000", false);
    dispatch(&mut state, Op::TimeDow).unwrap();
    // Python: datetime(2000,2,29).isoweekday() = 2 (Tue); HP: Tue=2
    assert_eq!(hpnum_to_i64(&state.stack.x), 2, "Feb 29 2000 is Tuesday (DOW=2)");
}

/// DDAYS: across leap day 2000 (Feb 28 to Mar 1 = 2 days because Feb 29 exists)
#[test]
fn date_accuracy_04_ddays_across_leap_day_2000() {
    // Y = Mar 1 2000, X = Feb 28 2000 → Y - X = 2
    let mut state = setup_ddays_state("3.012000", "2.282000", false);
    dispatch(&mut state, Op::TimeDdays).unwrap();
    assert_eq!(hpnum_to_i64(&state.stack.x), 2, "Mar 1 2000 - Feb 28 2000 = 2 days (leap year)");
}

// ═══════════════════════════════════════════════════════════════════════════
// SECTION 2: Century 2100 — NOT a leap year (divisible by 100 but NOT 400)
// Oracle: Python calendar.isleap(2100) = False
// ═══════════════════════════════════════════════════════════════════════════

/// DATE+: Feb 28 2100 + 1 = Mar 1 2100 (year 2100 is NOT a leap year)
#[test]
fn date_accuracy_05_date_plus_feb28_2100_not_leap() {
    let mut state = setup_date_state("2.282100", "1", false);
    dispatch(&mut state, Op::TimeDatePlus).unwrap();
    let (year, month, day) = parse_date_hpnum(&state.stack.x, false).unwrap();
    assert_eq!((year, month, day), (2100, 3, 1), "Year 2100 is NOT a leap year; Feb 28 + 1 = Mar 1");
}

/// DDAYS: across Feb 28/Mar 1 in 2100 = 1 day (no Feb 29)
#[test]
fn date_accuracy_06_ddays_2100_no_leap_day() {
    // Y = Mar 1 2100, X = Feb 28 2100 → Y - X = 1 (no leap day)
    let mut state = setup_ddays_state("3.012100", "2.282100", false);
    dispatch(&mut state, Op::TimeDdays).unwrap();
    assert_eq!(hpnum_to_i64(&state.stack.x), 1, "Mar 1 2100 - Feb 28 2100 = 1 day (non-leap century)");
}

// ═══════════════════════════════════════════════════════════════════════════
// SECTION 3: Century 2400 — IS a leap year (divisible by 400)
// Oracle: Python calendar.isleap(2400) = True
// ═══════════════════════════════════════════════════════════════════════════

/// DATE+: Feb 28 2400 + 1 = Feb 29 2400 (year 2400 IS a leap year)
#[test]
fn date_accuracy_07_date_plus_feb28_2400_leap_century() {
    let mut state = setup_date_state("2.282400", "1", false);
    dispatch(&mut state, Op::TimeDatePlus).unwrap();
    let (year, month, day) = parse_date_hpnum(&state.stack.x, false).unwrap();
    assert_eq!((year, month, day), (2400, 2, 29), "Year 2400 IS a leap year; Feb 28 + 1 = Feb 29");
}

/// DDAYS: across leap day 2400 (Feb 28 to Mar 1 = 2 days)
#[test]
fn date_accuracy_08_ddays_across_leap_day_2400() {
    let mut state = setup_ddays_state("3.012400", "2.282400", false);
    dispatch(&mut state, Op::TimeDdays).unwrap();
    assert_eq!(hpnum_to_i64(&state.stack.x), 2, "Mar 1 2400 - Feb 28 2400 = 2 (leap century)");
}

// ═══════════════════════════════════════════════════════════════════════════
// SECTION 4: Gregorian calendar start — Oct 15, 1582
// Oracle: Meeus "Astronomical Algorithms" Table 7.a; JDN = 2299161
// ═══════════════════════════════════════════════════════════════════════════

/// DOW: Oct 15 1582 = Friday = 5
/// Oracle: JDN 2299161, (2299161 + 1) % 7 = 2299162 % 7 = 5 (Friday)
/// Python gregorian proleptic: datetime(1582, 10, 15) (ISO 8601 extension)
#[test]
fn date_accuracy_09_dow_gregorian_start_oct15_1582_friday() {
    let jdn = date_to_jdn(1582, 10, 15);
    let dow = jdn_to_dow(jdn);
    assert_eq!(dow, 5, "Oct 15 1582 (Gregorian start) is Friday (DOW=5)");
}

/// JDN roundtrip: Oct 15 1582 — first Gregorian calendar date
#[test]
fn date_accuracy_10_jdn_roundtrip_gregorian_start_1582() {
    let jdn = date_to_jdn(1582, 10, 15);
    let (year, month, day) = jdn_to_date(jdn);
    assert_eq!((year, month, day), (1582, 10, 15), "JDN roundtrip for Gregorian start date");
}

/// DATE+: Oct 15 1582 + 1 = Oct 16 1582 (next valid Gregorian date)
#[test]
fn date_accuracy_11_date_plus_gregorian_start_forward() {
    // MDY format: 10.151582
    let mut state = setup_date_state("10.151582", "1", false);
    dispatch(&mut state, Op::TimeDatePlus).unwrap();
    let (year, month, day) = parse_date_hpnum(&state.stack.x, false).unwrap();
    assert_eq!((year, month, day), (1582, 10, 16), "Oct 15 1582 + 1 = Oct 16 1582");
}

// ═══════════════════════════════════════════════════════════════════════════
// SECTION 5: Y2K boundary
// Oracle: Python datetime(1999,12,31) + timedelta(1) = datetime(2000,1,1)
// ═══════════════════════════════════════════════════════════════════════════

/// DDAYS: Dec 31 1999 to Jan 1 2000 = 1 day
#[test]
fn date_accuracy_12_ddays_y2k_boundary_1_day() {
    // Y = Jan 1 2000, X = Dec 31 1999 → Y - X = 1
    let mut state = setup_ddays_state("1.012000", "12.311999", false);
    dispatch(&mut state, Op::TimeDdays).unwrap();
    assert_eq!(hpnum_to_i64(&state.stack.x), 1, "Jan 1 2000 - Dec 31 1999 = 1 day");
}

/// DATE+: Dec 31 1999 + 1 = Jan 1 2000
#[test]
fn date_accuracy_13_date_plus_y2k_crossing() {
    let mut state = setup_date_state("12.311999", "1", false);
    dispatch(&mut state, Op::TimeDatePlus).unwrap();
    let (year, month, day) = parse_date_hpnum(&state.stack.x, false).unwrap();
    assert_eq!((year, month, day), (2000, 1, 1), "Dec 31 1999 + 1 = Jan 1 2000");
}

/// DOW: Jan 1 2000 = Saturday = 6
/// Oracle: Python datetime(2000,1,1).isoweekday() = 6 (Sat)
#[test]
fn date_accuracy_14_dow_jan1_2000_saturday() {
    let mut state = setup_dow_state("1.012000", false);
    dispatch(&mut state, Op::TimeDow).unwrap();
    assert_eq!(hpnum_to_i64(&state.stack.x), 6, "Jan 1 2000 is Saturday (DOW=6)");
}

// ═══════════════════════════════════════════════════════════════════════════
// SECTION 6: Large offset test (100 years including leap days)
// Oracle: Python datetime(2000,1,1) + timedelta(36525) = datetime(2100,1,1)
// Note: 100 years from Jan 1 2000 = 36524 days to Jan 1 2100 (2100 is non-leap)
//   but we verify exact count: date(2100,1,1) - date(2000,1,1) via DDAYS
// ═══════════════════════════════════════════════════════════════════════════

/// DDAYS: Jan 1 2000 to Jan 1 2100 = 36525 days (100 years, includes leap years 2000..2096)
/// Oracle: Python (datetime(2100,1,1) - datetime(2000,1,1)).days = 36525
/// Note: 2000 IS a leap year; 2100 is NOT — so 25 leap years in [2000,2004,...,2096]
/// 100*365 + 25 = 36525
#[test]
fn date_accuracy_15_ddays_100_years_2000_to_2100() {
    // Y = Jan 1 2100, X = Jan 1 2000
    let mut state = setup_ddays_state("1.012100", "1.012000", false);
    dispatch(&mut state, Op::TimeDdays).unwrap();
    assert_eq!(hpnum_to_i64(&state.stack.x), 36525, "Jan 1 2100 - Jan 1 2000 = 36525 days");
}

/// DATE+: Jan 1 2000 + 36525 = Jan 1 2100
#[test]
fn date_accuracy_16_date_plus_large_offset_100_years() {
    let mut state = setup_date_state("1.012000", "36525", false);
    dispatch(&mut state, Op::TimeDatePlus).unwrap();
    let (year, month, day) = parse_date_hpnum(&state.stack.x, false).unwrap();
    assert_eq!((year, month, day), (2100, 1, 1), "Jan 1 2000 + 36525 days = Jan 1 2100");
}

// ═══════════════════════════════════════════════════════════════════════════
// SECTION 7: Known historical DOW values (oracle: Python datetime.isoweekday())
// HP DOW: 0=Sun, 1=Mon, 2=Tue, 3=Wed, 4=Thu, 5=Fri, 6=Sat
// Python isoweekday: 1=Mon…7=Sun → map: HP = (iso % 7) i.e. Sun=7→0, Mon=1→1
// ═══════════════════════════════════════════════════════════════════════════

/// DOW: July 4, 1776 = Thursday = 4
/// Oracle: Python datetime(1776, 7, 4).isoweekday() = 4 (Thu)
#[test]
fn date_accuracy_17_dow_july4_1776_thursday() {
    let mut state = setup_dow_state("7.041776", false);
    dispatch(&mut state, Op::TimeDow).unwrap();
    assert_eq!(hpnum_to_i64(&state.stack.x), 4, "July 4 1776 is Thursday (DOW=4)");
}

/// DOW: Sep 11, 2001 = Tuesday = 2
/// Oracle: Python datetime(2001, 9, 11).isoweekday() = 2 (Tue)
#[test]
fn date_accuracy_18_dow_sep11_2001_tuesday() {
    let mut state = setup_dow_state("9.112001", false);
    dispatch(&mut state, Op::TimeDow).unwrap();
    assert_eq!(hpnum_to_i64(&state.stack.x), 2, "Sep 11 2001 is Tuesday (DOW=2)");
}

/// DOW: May 25, 1977 (Star Wars opening) = Wednesday = 3
/// Oracle: Python datetime(1977, 5, 25).isoweekday() = 3 (Wed)
#[test]
fn date_accuracy_19_dow_may25_1977_wednesday() {
    let mut state = setup_dow_state("5.251977", false);
    dispatch(&mut state, Op::TimeDow).unwrap();
    assert_eq!(hpnum_to_i64(&state.stack.x), 3, "May 25 1977 is Wednesday (DOW=3)");
}

// ═══════════════════════════════════════════════════════════════════════════
// SECTION 8: DMY mode tests (Flag 31 set)
// ═══════════════════════════════════════════════════════════════════════════

/// DMY mode — DATE+: Jan 1 2026 + 31 = Feb 1 2026
/// DMY format: DD.MMYYYY → 1.012026 = Jan 1 2026
#[test]
fn date_accuracy_20_date_plus_dmy_mode_jan_to_feb() {
    let mut state = setup_date_state("1.012026", "31", true);
    dispatch(&mut state, Op::TimeDatePlus).unwrap();
    let (year, month, day) = parse_date_hpnum(&state.stack.x, true).unwrap();
    assert_eq!((year, month, day), (2026, 2, 1), "DMY: Jan 1 2026 + 31 = Feb 1 2026");
}

/// DMY mode — DDAYS: Feb 28 2000 to Mar 1 2000 = 2 days (leap year)
/// DMY: 28.022000 = Feb 28 2000; 1.032000 = Mar 1 2000
#[test]
fn date_accuracy_21_ddays_dmy_mode_across_leap_2000() {
    let mut state = setup_ddays_state("1.032000", "28.022000", true);
    dispatch(&mut state, Op::TimeDdays).unwrap();
    assert_eq!(hpnum_to_i64(&state.stack.x), 2, "DMY: Mar 1 2000 - Feb 28 2000 = 2 (leap)");
}

/// DMY mode — DOW: Jan 1 2000 = Saturday (same date, different notation)
/// DMY: 1.012000 = Jan 1 2000
#[test]
fn date_accuracy_22_dow_dmy_mode_jan1_2000() {
    let mut state = setup_dow_state("1.012000", true);
    dispatch(&mut state, Op::TimeDow).unwrap();
    assert_eq!(hpnum_to_i64(&state.stack.x), 6, "DMY: Jan 1 2000 is Saturday (DOW=6)");
}

// ═══════════════════════════════════════════════════════════════════════════
// SECTION 9: Negative/same-date DDAYS and DATE+ with negative days
// ═══════════════════════════════════════════════════════════════════════════

/// DDAYS: same date = 0
#[test]
fn date_accuracy_23_ddays_same_date_zero() {
    let mut state = setup_ddays_state("5.242026", "5.242026", false);
    dispatch(&mut state, Op::TimeDdays).unwrap();
    assert_eq!(hpnum_to_i64(&state.stack.x), 0, "Same date DDAYS = 0");
}

/// DDAYS: X later than Y → negative result
/// Y = Jan 1 2026, X = Dec 31 2026 → Y - X = -364
#[test]
fn date_accuracy_24_ddays_negative_x_later_than_y() {
    let mut state = setup_ddays_state("1.012026", "12.312026", false);
    dispatch(&mut state, Op::TimeDdays).unwrap();
    assert_eq!(hpnum_to_i64(&state.stack.x), -364, "Jan 1 - Dec 31 = -364 (negative DDAYS)");
}

/// DATE+ with negative days: Jan 1 2026 + (-1) = Dec 31 2025 (going backward)
#[test]
fn date_accuracy_25_date_plus_negative_days_backward() {
    let mut state = setup_date_state("1.012026", "-1", false);
    dispatch(&mut state, Op::TimeDatePlus).unwrap();
    let (year, month, day) = parse_date_hpnum(&state.stack.x, false).unwrap();
    assert_eq!((year, month, day), (2025, 12, 31), "Jan 1 2026 + (-1) = Dec 31 2025");
}

/// DATE+ with large negative days: Jan 1 2000 + (-365) = Jan 1 1999
#[test]
fn date_accuracy_26_date_plus_large_negative_backward() {
    let mut state = setup_date_state("1.012000", "-365", false);
    dispatch(&mut state, Op::TimeDatePlus).unwrap();
    let (year, month, day) = parse_date_hpnum(&state.stack.x, false).unwrap();
    assert_eq!((year, month, day), (1999, 1, 1), "Jan 1 2000 + (-365) = Jan 1 1999");
}

// ═══════════════════════════════════════════════════════════════════════════
// SECTION 10: Additional edge cases
// ═══════════════════════════════════════════════════════════════════════════

/// DATE+: end-of-month rollover (Jan 31 + 1 = Feb 1)
#[test]
fn date_accuracy_27_date_plus_end_of_month_rollover() {
    let mut state = setup_date_state("1.312026", "1", false);
    dispatch(&mut state, Op::TimeDatePlus).unwrap();
    let (year, month, day) = parse_date_hpnum(&state.stack.x, false).unwrap();
    assert_eq!((year, month, day), (2026, 2, 1), "Jan 31 2026 + 1 = Feb 1 2026");
}

/// DATE+: year rollover (Dec 31 2025 + 1 = Jan 1 2026)
#[test]
fn date_accuracy_28_date_plus_year_rollover() {
    let mut state = setup_date_state("12.312025", "1", false);
    dispatch(&mut state, Op::TimeDatePlus).unwrap();
    let (year, month, day) = parse_date_hpnum(&state.stack.x, false).unwrap();
    assert_eq!((year, month, day), (2026, 1, 1), "Dec 31 2025 + 1 = Jan 1 2026");
}

/// DDAYS: large positive span — Jan 1 1900 to Jan 1 2000 = 36524 days
/// Oracle: Python (datetime(2000,1,1) - datetime(1900,1,1)).days = 36524
/// Note: 1900 is NOT a leap year (div by 100, not 400); same as 2100
#[test]
fn date_accuracy_29_ddays_century_1900_to_2000() {
    let mut state = setup_ddays_state("1.012000", "1.011900", false);
    dispatch(&mut state, Op::TimeDdays).unwrap();
    assert_eq!(hpnum_to_i64(&state.stack.x), 36524, "Jan 1 2000 - Jan 1 1900 = 36524 days");
}

/// DOW: Monday sentinel — Jan 2 2006 = Monday = 1
/// Oracle: Python datetime(2006, 1, 2).isoweekday() = 1 (Mon)
#[test]
fn date_accuracy_30_dow_monday_sentinel_jan2_2006() {
    let mut state = setup_dow_state("1.022006", false);
    dispatch(&mut state, Op::TimeDow).unwrap();
    assert_eq!(hpnum_to_i64(&state.stack.x), 1, "Jan 2 2006 is Monday (DOW=1)");
}
