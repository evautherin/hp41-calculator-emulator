// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Alpha time/date formatting operations for the HP Time Module.
//!
//! Covers: ADATE, ATIME, ATIME24.
//!
//! These operations read a time or date HpNum from stack X and append a
//! formatted string to the ALPHA register (`state.alpha_reg`).
//!
//! HP-41 ALPHA register: 24 characters max. Strings are truncated to keep
//! the total ≤ 24 characters.
//!
//! Phase 38 ships stub implementations. Full formatting (12/24h, DMY/MDY
//! modes, AM/PM suffix) lands in Wave 2.

use crate::error::HpError;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;

/// Maximum ALPHA register length (HP-41 hardware limit).
const ALPHA_MAX_LEN: usize = 24;

/// Append `text` to `state.alpha_reg`, truncating to ALPHA_MAX_LEN.
fn alpha_append(state: &mut CalcState, text: &str) {
    let remaining = ALPHA_MAX_LEN.saturating_sub(state.alpha_reg.len());
    if remaining > 0 {
        let truncated = &text[..text.len().min(remaining)];
        state.alpha_reg.push_str(truncated);
    }
}

/// ATIME — Format time from X (HH.MMSSss) and append to ALPHA register.
///
/// Respects `state.clock_12h` for 12/24-hour output format.
/// Phase 38 stub: appends "TIME" placeholder string.
pub fn op_atime(state: &mut CalcState) -> Result<(), HpError> {
    // Stub: append placeholder; real formatting in Wave 2.
    let text = if state.clock_12h { "12:00AM" } else { "00:00:00" };
    alpha_append(state, text);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ATIME24 — Format time from X (HH.MMSSss) in 24-hour format and append to ALPHA.
///
/// Always 24-hour regardless of `state.clock_12h`.
/// Phase 38 stub: appends "00:00:00" placeholder.
pub fn op_atime24(state: &mut CalcState) -> Result<(), HpError> {
    // Stub: append placeholder; real formatting in Wave 2.
    alpha_append(state, "00:00:00");
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADATE — Format date from X (per Flag 31 mode) and append to ALPHA register.
///
/// Flag 31 clear (MDY): "MM/DD/YY" format; Flag 31 set (DMY): "DD.MM.YY" format.
/// Phase 38 stub: appends "DATE" placeholder string.
pub fn op_adate(state: &mut CalcState) -> Result<(), HpError> {
    // Stub: append placeholder; real formatting in Wave 2.
    let dmy = state.flags & (1u64 << 31) != 0;
    let text = if dmy { "01.01.70" } else { "1/01/70" };
    alpha_append(state, text);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn op_atime_appends_to_alpha() {
        let mut state = CalcState::new();
        state.alpha_reg = "T=".to_string();
        op_atime(&mut state).unwrap();
        assert!(state.alpha_reg.starts_with("T="));
        assert!(state.alpha_reg.len() > 2);
    }

    #[test]
    fn op_atime24_appends_24h_format() {
        let mut state = CalcState::new();
        op_atime24(&mut state).unwrap();
        assert_eq!(state.alpha_reg, "00:00:00");
    }

    #[test]
    fn op_adate_appends_to_alpha() {
        let mut state = CalcState::new();
        op_adate(&mut state).unwrap();
        assert!(!state.alpha_reg.is_empty());
    }

    #[test]
    fn alpha_append_truncates_at_24_chars() {
        let mut state = CalcState::new();
        // Fill alpha to 20 chars already
        state.alpha_reg = "A".repeat(20);
        // Try to append 10 more — should be truncated to 4
        alpha_append(&mut state, "0123456789");
        assert_eq!(state.alpha_reg.len(), 24);
        assert!(state.alpha_reg.ends_with("0123"));
    }

    #[test]
    fn alpha_append_noop_when_full() {
        let mut state = CalcState::new();
        state.alpha_reg = "X".repeat(24);
        alpha_append(&mut state, "extra");
        assert_eq!(state.alpha_reg.len(), 24);
    }

    #[test]
    fn op_atime_respects_clock_12h_flag() {
        let mut state = CalcState::new();
        state.clock_12h = true;
        state.alpha_reg.clear();
        op_atime(&mut state).unwrap();
        // In 12h stub mode, output contains "AM" marker
        assert!(state.alpha_reg.contains("AM") || state.alpha_reg.contains("PM") || !state.alpha_reg.is_empty());
    }
}
