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
//! Phase 38 ships stub op functions and helper stubs. Full implementations
//! (Julian Day Number arithmetic, leap year, day-of-week) land in Wave 2.

use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;

/// Parse an HP-41 date HpNum into (first_field, second_field, year).
///
/// In MDY mode (`dmy = false`): first = month (MM), second = day (DD).
/// In DMY mode (`dmy = true`):  first = day (DD),   second = month (MM).
///
/// The fractional part is LEFT-padded to 6 chars: first_field(2) + second_field(2) + year_partial(2).
/// For full 4-digit year: the integer part contributes the high digits.
///
/// Phase 38 stub: returns Data error for all inputs (full parsing in Wave 2).
pub fn parse_date_hpnum(hpnum: &HpNum, _dmy: bool) -> Result<(u8, u8, i32), HpError> {
    let s = hpnum.inner().to_string();
    let (int_part, frac_part) = if let Some(pos) = s.find('.') {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s.as_str(), "")
    };
    // LEFT-pad fractional part to exactly 6 digits (preserves leading zeros for year).
    // NOTE: parse_counter uses right-pad ("{:0<5}"); dates use LEFT-pad ("{:0>6}").
    let padded = format!("{:0>6}", frac_part);
    let first_field: u8 = int_part.parse().map_err(|_| HpError::InvalidOp)?;
    let second_field: u8 = padded[0..2].parse().map_err(|_| HpError::InvalidOp)?;
    let year_lo: i32 = padded[2..6].parse().map_err(|_| HpError::InvalidOp)?;
    // Year encoding: if year_lo < 100, it represents a 2-digit year (ambiguous).
    // Full 4-digit year reconstruction deferred to Wave 2 algorithm implementation.
    Ok((first_field, second_field, year_lo))
}

/// Parse an HP-41 time HpNum (HH.MMSScc) into (hours, minutes, seconds, centiseconds).
///
/// Fractional part is LEFT-padded to 6 digits: MM(2) + SS(2) + cc(2).
/// Hours come from the integer part.
///
/// Phase 38 stub: returns Data error for negative input.
pub fn parse_time_hpnum(hpnum: &HpNum) -> Result<(u8, u8, u8, u8), HpError> {
    let s = hpnum.inner().to_string();
    let (int_part, frac_part) = if let Some(pos) = s.find('.') {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s.as_str(), "")
    };
    let hours: u8 = int_part.parse().map_err(|_| HpError::InvalidOp)?;
    let padded = format!("{:0>6}", frac_part); // 6 digits: MMSScc
    let minutes: u8 = padded[0..2].parse().map_err(|_| HpError::InvalidOp)?;
    let seconds: u8 = padded[2..4].parse().map_err(|_| HpError::InvalidOp)?;
    let centiseconds: u8 = padded[4..6].parse().map_err(|_| HpError::InvalidOp)?;
    Ok((hours, minutes, seconds, centiseconds))
}

/// Convert a date triple (day, month, year) to Julian Day Number.
///
/// Uses the standard Julian Day Number formula (Jean Meeus, "Astronomical Algorithms").
/// Phase 38 stub: returns 0 for all inputs (full formula in Wave 2).
pub fn date_to_jdn(_day: u8, _month: u8, _year: i32) -> i64 {
    0
}

/// Convert a Julian Day Number to (day, month, year).
///
/// Phase 38 stub: returns (1, 1, 1970).
pub fn jdn_to_date(_jdn: i64) -> (u8, u8, i32) {
    (1, 1, 1970)
}

/// Convert a Julian Day Number to day-of-week (0 = Sunday, 1 = Monday, ..., 6 = Saturday).
///
/// Phase 38 stub: returns 0.
pub fn jdn_to_dow(_jdn: i64) -> u8 {
    0
}

/// DATE+ — Add X days to date in Y, push result to stack X.
///
/// Y: date HpNum (per Flag 31 mode); X: integer day count.
/// Phase 38 stub: pushes 0 to stack X.
pub fn op_date_plus(state: &mut CalcState) -> Result<(), HpError> {
    // Stub: push zero placeholder.
    enter_number(state, HpNum::zero());
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// DDAYS — Compute number of days between two dates.
///
/// Y: first date; X: second date (both per Flag 31 mode).
/// Result: X - Y in days (positive if X is later).
/// Phase 38 stub: pushes 0 to stack X.
pub fn op_ddays(state: &mut CalcState) -> Result<(), HpError> {
    // Stub: push zero placeholder.
    enter_number(state, HpNum::zero());
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// DOW — Day of week for date in X.
///
/// Pushes 1–7 (1 = Monday per HP 82182A OM §DOW) onto stack X.
/// Phase 38 stub: pushes 0 to stack X.
pub fn op_dow(state: &mut CalcState) -> Result<(), HpError> {
    // Stub: push zero placeholder.
    enter_number(state, HpNum::zero());
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// DMY — Set DMY date mode (Flag 31 set).
///
/// LiftEffect::Neutral.
pub fn op_dmy(state: &mut CalcState) -> Result<(), HpError> {
    // Flag 31 = DMY mode (HP 82182A OM §DMY).
    state.flags |= 1u64 << 31;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// MDY — Set MDY date mode (Flag 31 clear).
///
/// LiftEffect::Neutral.
pub fn op_mdy(state: &mut CalcState) -> Result<(), HpError> {
    // Flag 31 clear = MDY mode.
    state.flags &= !(1u64 << 31);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn parse_time_hpnum_zero() {
        let n = HpNum::zero();
        let (h, m, s, cs) = parse_time_hpnum(&n).unwrap();
        assert_eq!((h, m, s, cs), (0, 0, 0, 0));
    }

    #[test]
    fn parse_time_hpnum_12_30_45_50() {
        // 12.303045 → 12h 30m 30s (wait — 12.303045 = 12h, 30m, 30s, 45cs)
        // Actually HH.MMSScc: 12.303045 → h=12, frac="303045" → mm=30, ss=30, cc=45
        use rust_decimal::Decimal;
        use std::str::FromStr;
        let n = HpNum::from(Decimal::from_str("12.303045").unwrap());
        let (h, m, s, cs) = parse_time_hpnum(&n).unwrap();
        assert_eq!(h, 12);
        assert_eq!(m, 30);
        assert_eq!(s, 30);
        assert_eq!(cs, 45);
    }

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
    fn op_date_plus_stub_pushes_zero() {
        let mut state = CalcState::new();
        op_date_plus(&mut state).unwrap();
        assert_eq!(state.stack.x, HpNum::zero());
    }

    #[test]
    fn op_ddays_stub_pushes_zero() {
        let mut state = CalcState::new();
        op_ddays(&mut state).unwrap();
        assert_eq!(state.stack.x, HpNum::zero());
    }

    #[test]
    fn op_dow_stub_pushes_zero() {
        let mut state = CalcState::new();
        op_dow(&mut state).unwrap();
        assert_eq!(state.stack.x, HpNum::zero());
    }
}
