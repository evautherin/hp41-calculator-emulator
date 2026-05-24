// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Clock, display, and format operations for the HP Time Module.
//!
//! Covers: TIME, DATE, SETIME, SETDATE, CLK12, CLK24, CLKT, CLKTD, CLOCK,
//!         CORRECT, T+X.
//!
//! Time representation: HH.MMSScc (hours.minutessecondscentiseconds).
//! Date representation: depends on Flag 31:
//!   - Flag 31 clear (MDY mode): MM.DDYYYY
//!   - Flag 31 set   (DMY mode): DD.MMYYYY
//!
//! Phase 38 ships stub implementations. Full clock logic (SystemTime + offset
//! computation, formatting) lands in Wave 2 plans.

use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;
use serde::{Deserialize, Serialize};

/// Clock display mode for `CalcState::clock_display_mode`.
///
/// `Off`: no continuous clock display.
/// `TimeOnly`: CLKT — display time continuously.
/// `TimeAndDate`: CLKTD — display time and date alternating.
///
/// Default: `Off`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum ClockDisplayMode {
    #[default]
    Off,
    TimeOnly,
    TimeAndDate,
}

/// TIME — Read current time (adjusted by `time_offset_secs`) and push as
/// HH.MMSScc HpNum onto stack X.
///
/// Phase 38 stub: pushes 0 (no SystemTime call yet).
pub fn op_time(state: &mut CalcState) -> Result<(), HpError> {
    // Stub: push zero as placeholder; real SystemTime logic in Wave 2.
    enter_number(state, HpNum::zero());
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// DATE — Read current date (adjusted by `time_offset_secs`) and push as
/// date HpNum (MM.DDYYYY or DD.MMYYYY depending on Flag 31) onto stack X.
///
/// Phase 38 stub: pushes 0.
pub fn op_date(state: &mut CalcState) -> Result<(), HpError> {
    // Stub: push zero as placeholder; real SystemTime logic in Wave 2.
    enter_number(state, HpNum::zero());
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// SETIME — Set clock time from stack X (HH.MMSScc format).
///
/// Computes `time_offset_secs = entered_unix_secs - SystemTime::now()`.
/// Phase 38 stub: opens the SetTimePrompt modal.
pub fn op_setime(state: &mut CalcState) -> Result<(), HpError> {
    state.modal_program = Some(crate::ops::math1::modal::ModalProgram::Time(
        crate::ops::time::modal::TimeStep::SetTimePrompt,
    ));
    state.modal_prompt = Some("TIME?".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// SETDATE — Set clock date from stack X (date decimal format per Flag 31).
///
/// Phase 38 stub: opens the SetDatePrompt modal.
pub fn op_setdate(state: &mut CalcState) -> Result<(), HpError> {
    state.modal_program = Some(crate::ops::math1::modal::ModalProgram::Time(
        crate::ops::time::modal::TimeStep::SetDatePrompt,
    ));
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

/// CLKT — Enable continuous time-only display.
///
/// Sets `clock_display_mode = TimeOnly` and `clock_active = true`.
/// Phase 38 stub: mode flag set; frontend rendering deferred to Wave 2.
pub fn op_clkt(state: &mut CalcState) -> Result<(), HpError> {
    state.clock_display_mode = ClockDisplayMode::TimeOnly;
    state.clock_active = true;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CLKTD — Enable continuous time+date display.
///
/// Sets `clock_display_mode = TimeAndDate` and `clock_active = true`.
/// Phase 38 stub: mode flag set; frontend rendering deferred to Wave 2.
pub fn op_clktd(state: &mut CalcState) -> Result<(), HpError> {
    state.clock_display_mode = ClockDisplayMode::TimeAndDate;
    state.clock_active = true;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CLOCK — Disable continuous clock display.
///
/// Sets `clock_display_mode = Off` and `clock_active = false`.
pub fn op_clock(state: &mut CalcState) -> Result<(), HpError> {
    state.clock_display_mode = ClockDisplayMode::Off;
    state.clock_active = false;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CORRECT — Apply clock accuracy correction from stack X.
///
/// Phase 38 stub: reads X, stores into `state.accuracy_factor`.
pub fn op_correct(state: &mut CalcState) -> Result<(), HpError> {
    state.accuracy_factor = state.stack.x.clone();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// T+X — Add X seconds to the clock time offset.
///
/// Phase 38 stub: adds X (as i64 seconds) to `time_offset_secs`.
pub fn op_tplusx(state: &mut CalcState) -> Result<(), HpError> {
    use rust_decimal::prelude::ToPrimitive;
    let delta = state
        .stack
        .x
        .inner()
        .to_i64()
        .ok_or(HpError::InvalidOp)?;
    state.time_offset_secs = state.time_offset_secs.saturating_add(delta);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

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

    #[test]
    fn op_clkt_sets_mode_and_active() {
        let mut state = CalcState::new();
        op_clkt(&mut state).unwrap();
        assert_eq!(state.clock_display_mode, ClockDisplayMode::TimeOnly);
        assert!(state.clock_active);
    }

    #[test]
    fn op_clktd_sets_time_and_date_mode() {
        let mut state = CalcState::new();
        op_clktd(&mut state).unwrap();
        assert_eq!(state.clock_display_mode, ClockDisplayMode::TimeAndDate);
        assert!(state.clock_active);
    }

    #[test]
    fn op_clock_disables_display() {
        let mut state = CalcState::new();
        state.clock_display_mode = ClockDisplayMode::TimeOnly;
        state.clock_active = true;
        op_clock(&mut state).unwrap();
        assert_eq!(state.clock_display_mode, ClockDisplayMode::Off);
        assert!(!state.clock_active);
    }

    #[test]
    fn op_time_stub_pushes_zero() {
        let mut state = CalcState::new();
        op_time(&mut state).unwrap();
        assert_eq!(state.stack.x, HpNum::zero());
    }

    #[test]
    fn op_date_stub_pushes_zero() {
        let mut state = CalcState::new();
        op_date(&mut state).unwrap();
        assert_eq!(state.stack.x, HpNum::zero());
    }

    #[test]
    fn op_correct_stores_accuracy_factor() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(2i32);
        op_correct(&mut state).unwrap();
        assert_eq!(state.accuracy_factor, HpNum::from(2i32));
    }
}
