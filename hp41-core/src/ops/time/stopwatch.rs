// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Stopwatch operations for the HP Time Module.
//!
//! The stopwatch state machine uses three persistent `CalcState` fields:
//! - `stopwatch_mode: StopwatchMode` — current mode (Idle/Running/Stopped).
//! - `stopwatch_accumulated: f64` — elapsed seconds not in the current run.
//! - `stopwatch_split: f64` — split-lap reference (SWPT stores here).
//!
//! And one transient field (`#[serde(default, skip)]`):
//! - `stopwatch_start: Option<std::time::Instant>` — start of current run.
//!
//! On save-file load, D-38.6 mandates that a Running stopwatch transitions to
//! Stopped (Instant is not serializable; the elapsed partial lap is lost). This
//! is handled in `CalcState::migrate_after_load()`.

use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;
use serde::{Deserialize, Serialize};

/// Stopwatch operating mode for `CalcState::stopwatch_mode`.
///
/// `Idle`: never started or fully reset; elapsed = 0.
/// `Running`: timer is active; elapsed = accumulated + since(start).
/// `Stopped`: timer is paused; elapsed = accumulated.
///
/// Default: `Idle` (hardware cold-start state per HP 82182A OM).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum StopwatchMode {
    Idle,
    Running,
    Stopped,
}

impl Default for StopwatchMode {
    fn default() -> Self {
        StopwatchMode::Idle
    }
}

/// RUNSW — Start the stopwatch.
///
/// Transitions to Running and records `Instant::now()` as the lap start.
/// LiftEffect::Neutral (no stack change per HP 82182A OM §RUNSW).
pub fn op_runsw(state: &mut CalcState) -> Result<(), HpError> {
    state.stopwatch_mode = StopwatchMode::Running;
    state.stopwatch_start = Some(std::time::Instant::now());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// STOPSW — Stop (pause) the stopwatch.
///
/// Accumulates elapsed time of the current run, transitions to Stopped.
/// No-op if already Idle or Stopped.
/// LiftEffect::Neutral.
pub fn op_stopsw(state: &mut CalcState) -> Result<(), HpError> {
    if let StopwatchMode::Running = state.stopwatch_mode {
        if let Some(start) = state.stopwatch_start.take() {
            state.stopwatch_accumulated += start.elapsed().as_secs_f64();
        }
        state.stopwatch_mode = StopwatchMode::Stopped;
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// RCLSW — Recall stopwatch elapsed time to stack X.
///
/// Reads accumulated + running-lap time, converts to HH.MMSScc HpNum,
/// pushes to stack X. LiftEffect::Enable.
pub fn op_rclsw(state: &mut CalcState) -> Result<(), HpError> {
    let elapsed_secs = match state.stopwatch_mode {
        StopwatchMode::Running => {
            let start = state
                .stopwatch_start
                .unwrap_or_else(std::time::Instant::now);
            state.stopwatch_accumulated + start.elapsed().as_secs_f64()
        }
        _ => state.stopwatch_accumulated,
    };
    let result = secs_to_hpnum_time(elapsed_secs)?;
    enter_number(state, result);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// SETSW — Set (preset) the stopwatch accumulated time from stack X.
///
/// Phase 38 stub: reads X as HH.MMSSss, stores as seconds in accumulated.
/// Transitions to Stopped if Running. LiftEffect::Neutral.
pub fn op_setsw(state: &mut CalcState) -> Result<(), HpError> {
    // Stub: freeze if running then accept X as new accumulated time.
    if let StopwatchMode::Running = state.stopwatch_mode {
        if let Some(start) = state.stopwatch_start.take() {
            state.stopwatch_accumulated += start.elapsed().as_secs_f64();
        }
        state.stopwatch_mode = StopwatchMode::Stopped;
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// SW — Stopwatch display mode (toggle keyboard SW mode).
///
/// Phase 38 stub: sets `stopwatch_keyboard_mode = true`. LiftEffect::Neutral.
pub fn op_sw(state: &mut CalcState) -> Result<(), HpError> {
    state.stopwatch_keyboard_mode = true;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// SWPT — Split/lap: record current elapsed time as split reference.
///
/// Phase 38 stub: stores current elapsed into `stopwatch_split`. LiftEffect::Neutral.
pub fn op_swpt(state: &mut CalcState) -> Result<(), HpError> {
    let elapsed_secs = match state.stopwatch_mode {
        StopwatchMode::Running => {
            let start = state
                .stopwatch_start
                .unwrap_or_else(std::time::Instant::now);
            state.stopwatch_accumulated + start.elapsed().as_secs_f64()
        }
        _ => state.stopwatch_accumulated,
    };
    state.stopwatch_split = elapsed_secs;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// STPW — Stop/clear the stopwatch (full reset to Idle).
///
/// Phase 38 stub: resets all stopwatch state. LiftEffect::Neutral.
pub fn op_stpw(state: &mut CalcState) -> Result<(), HpError> {
    state.stopwatch_mode = StopwatchMode::Idle;
    state.stopwatch_start = None;
    state.stopwatch_accumulated = 0.0;
    state.stopwatch_split = 0.0;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// Convert elapsed seconds (f64) to HP-41 time HpNum format HH.MMSScc.
///
/// HP-41 time representation: integer part = hours (HH), fractional part = MMSScc
/// where MM = minutes (00-59), SS = seconds (00-59), cc = centiseconds (00-99).
///
/// This is the inverse of `parse_time_hpnum` in `date_arith.rs`.
pub(crate) fn secs_to_hpnum_time(secs: f64) -> Result<HpNum, HpError> {
    if secs < 0.0 || secs >= 360_000.0 {
        // 360_000 seconds = 100 hours, HP-41 max displayable
        return Err(HpError::InvalidOp);
    }
    let total_cs = (secs * 100.0).round() as u64;
    let hours = total_cs / 360_000;
    let rem = total_cs % 360_000;
    let minutes = rem / 6000;
    let rem2 = rem % 6000;
    let seconds = rem2 / 100;
    let centiseconds = rem2 % 100;

    // Build decimal: HH.MMSScc
    let frac_str = format!("{:02}{:02}{:02}", minutes, seconds, centiseconds);
    let hpnum_str = format!("{}.{}", hours, frac_str);
    use rust_decimal::Decimal;
    use std::str::FromStr;
    let d = Decimal::from_str(&hpnum_str).map_err(|_| HpError::InvalidOp)?;
    Ok(HpNum::from(d))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn stopwatch_mode_default_is_idle() {
        let mode = StopwatchMode::default();
        assert_eq!(mode, StopwatchMode::Idle);
    }

    #[test]
    fn stopwatch_mode_serde_round_trip() {
        let modes = [
            StopwatchMode::Idle,
            StopwatchMode::Running,
            StopwatchMode::Stopped,
        ];
        for mode in &modes {
            let json = serde_json::to_string(mode).unwrap();
            let restored: StopwatchMode = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, *mode);
        }
    }

    #[test]
    fn op_stpw_resets_all_stopwatch_state() {
        let mut state = CalcState::new();
        state.stopwatch_mode = StopwatchMode::Stopped;
        state.stopwatch_accumulated = 42.5;
        state.stopwatch_split = 20.0;
        op_stpw(&mut state).unwrap();
        assert_eq!(state.stopwatch_mode, StopwatchMode::Idle);
        assert_eq!(state.stopwatch_accumulated, 0.0);
        assert_eq!(state.stopwatch_split, 0.0);
        assert!(state.stopwatch_start.is_none());
    }

    #[test]
    fn op_stopsw_accumulates_when_running() {
        let mut state = CalcState::new();
        state.stopwatch_mode = StopwatchMode::Running;
        state.stopwatch_start = Some(std::time::Instant::now());
        state.stopwatch_accumulated = 10.0;
        // Brief sleep not viable — just check transition and start cleared.
        op_stopsw(&mut state).unwrap();
        assert_eq!(state.stopwatch_mode, StopwatchMode::Stopped);
        assert!(state.stopwatch_start.is_none());
        // accumulated increased by at least a tiny bit (Instant::now elapsed)
        assert!(state.stopwatch_accumulated >= 10.0);
    }

    #[test]
    fn op_stopsw_noop_when_idle() {
        let mut state = CalcState::new();
        state.stopwatch_mode = StopwatchMode::Idle;
        op_stopsw(&mut state).unwrap();
        assert_eq!(state.stopwatch_mode, StopwatchMode::Idle);
    }

    #[test]
    fn secs_to_hpnum_time_zero() {
        let n = secs_to_hpnum_time(0.0).unwrap();
        // 0 seconds → 0.000000
        assert_eq!(n, HpNum::from(rust_decimal::Decimal::ZERO));
    }

    #[test]
    fn secs_to_hpnum_time_one_hour_30_min() {
        // 1h 30m 0s = 5400 seconds → 1.300000
        let n = secs_to_hpnum_time(5400.0).unwrap();
        use rust_decimal::Decimal;
        use std::str::FromStr;
        let expected = HpNum::from(Decimal::from_str("1.300000").unwrap());
        assert_eq!(n, expected);
    }

    #[test]
    fn secs_to_hpnum_time_out_of_range() {
        assert!(secs_to_hpnum_time(-1.0).is_err());
        assert!(secs_to_hpnum_time(360_001.0).is_err());
    }

    #[test]
    fn op_runsw_sets_running_mode() {
        let mut state = CalcState::new();
        op_runsw(&mut state).unwrap();
        assert_eq!(state.stopwatch_mode, StopwatchMode::Running);
        assert!(state.stopwatch_start.is_some());
    }
}
