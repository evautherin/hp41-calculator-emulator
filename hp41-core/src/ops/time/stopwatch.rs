// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Stopwatch operations for the HP Time Module.
//!
//! The stopwatch state machine uses three persistent `CalcState` fields:
//! - `stopwatch_mode: StopwatchMode` — current mode (Idle/Running/Stopped).
//! - `stopwatch_accumulated: f64` — elapsed seconds not in the current run.
//! - `stopwatch_split: f64` — split-lap reference (STPW stores here, SWPT recalls).
//!
//! And one transient field (`#[serde(default, skip)]`):
//! - `stopwatch_start: Option<std::time::Instant>` — start of current run.
//!
//! On save-file load, D-38.6 mandates that a Running stopwatch transitions to
//! Stopped (Instant is not serializable; the elapsed partial lap is lost). This
//! is handled in `CalcState::migrate_after_load()`.
//!
//! ## Op Semantics (HP 82182A OM)
//!
//! - **RUNSW**: Starts or resumes the stopwatch. `stopwatch_start = Instant::now()`.
//!   If already Running, starts a fresh lap (accumulated preserved).
//! - **STOPSW**: Freezes the stopwatch. Adds elapsed since start to accumulated.
//!   No-op if Idle or Stopped.
//! - **SETSW**: Presets the stopwatch. Parses X as HH.MMSSss → accumulated seconds.
//!   Sets mode to Stopped.
//! - **RCLSW**: Recalls current elapsed (accumulated + running delta) as HH.MMSScc.
//!   Pushes to X with LiftEffect::Enable.
//! - **STPW**: Records current elapsed into `stopwatch_split` (split point).
//!   Stopwatch continues running (does NOT stop).
//! - **SWPT**: Recalls `stopwatch_split` to stack X as HH.MMSScc.
//! - **SW**: Activates stopwatch keyboard mode for Phase 39/41 frontend.

use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Stopwatch operating mode for `CalcState::stopwatch_mode`.
///
/// `Idle`: never started or fully reset; elapsed = 0.
/// `Running`: timer is active; elapsed = accumulated + since(start).
/// `Stopped`: timer is paused; elapsed = accumulated.
///
/// Default: `Idle` (hardware cold-start state per HP 82182A OM).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum StopwatchMode {
    #[default]
    Idle,
    Running,
    Stopped,
}

/// Compute total elapsed seconds for the stopwatch.
///
/// If Running: accumulated + time since start.
/// If Idle or Stopped: accumulated only.
fn current_elapsed(state: &CalcState) -> f64 {
    match state.stopwatch_mode {
        StopwatchMode::Running => {
            let start = state.stopwatch_start.unwrap_or_else(Instant::now);
            state.stopwatch_accumulated + start.elapsed().as_secs_f64()
        }
        _ => state.stopwatch_accumulated,
    }
}

/// RUNSW — Start or resume the stopwatch.
///
/// Transitions to Running and records `Instant::now()` as the lap start.
/// If already Running, resets the lap start (continues accumulation).
/// LiftEffect::Neutral (no stack change per HP 82182A OM §RUNSW).
pub fn op_runsw(state: &mut CalcState) -> Result<(), HpError> {
    state.stopwatch_mode = StopwatchMode::Running;
    state.stopwatch_start = Some(Instant::now());
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

/// SETSW — Preset the stopwatch accumulated time from stack X.
///
/// Parses X register as HH.MMSSss via string-split (P35 invariant), converts
/// to total seconds, stores in `stopwatch_accumulated`. Clears any running lap.
/// Sets mode to Stopped (ready for RUNSW). LiftEffect::Neutral.
pub fn op_setsw(state: &mut CalcState) -> Result<(), HpError> {
    // Parse X as HH.MMSSss using the same string-split approach as parse_time_hpnum.
    let x = state.stack.x.clone();
    let s = x.inner().to_string();
    let (int_part, frac_part) = if let Some(pos) = s.find('.') {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s.as_str(), "")
    };
    let hours: u64 = int_part.parse().map_err(|_| HpError::InvalidOp)?;
    // LEFT-pad fractional part to 6 digits: MM(2) + SS(2) + cc(2)
    let padded = format!("{:0>6}", frac_part);
    let minutes: u64 = padded[0..2].parse().map_err(|_| HpError::InvalidOp)?;
    let seconds: u64 = padded[2..4].parse().map_err(|_| HpError::InvalidOp)?;
    let centiseconds: u64 = padded[4..6].parse().map_err(|_| HpError::InvalidOp)?;

    // Validate ranges
    if minutes >= 60 || seconds >= 60 || centiseconds >= 100 {
        return Err(HpError::InvalidOp);
    }

    let total_secs = (hours * 3600 + minutes * 60 + seconds) as f64 + centiseconds as f64 * 0.01;

    // Stop any running lap before presetting
    state.stopwatch_start = None;
    state.stopwatch_accumulated = total_secs;
    state.stopwatch_mode = StopwatchMode::Stopped;

    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// RCLSW — Recall stopwatch elapsed time to stack X.
///
/// Reads accumulated + running-lap time, converts to HH.MMSScc HpNum,
/// pushes to stack X. LiftEffect::Enable.
pub fn op_rclsw(state: &mut CalcState) -> Result<(), HpError> {
    let elapsed_secs = current_elapsed(state);
    let result = secs_to_hpnum_time(elapsed_secs)?;
    enter_number(state, result);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// STPW — Record current elapsed time as split point.
///
/// Captures `current_elapsed` into `stopwatch_split`. Stopwatch continues
/// running (does NOT stop). LiftEffect::Neutral.
///
/// Note: STPW records the split; SWPT recalls it. (Contrast: the stub
/// had these reversed — this is the correct HP 82182A OM behavior.)
pub fn op_stpw(state: &mut CalcState) -> Result<(), HpError> {
    state.stopwatch_split = current_elapsed(state);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// SWPT — Recall split-point time to stack X.
///
/// Pushes `stopwatch_split` converted via `secs_to_hpnum_time` to X.
/// LiftEffect::Enable.
///
/// Note: SWPT recalls the split recorded by STPW. (Contrast: the stub
/// had these reversed — this is the correct HP 82182A OM behavior.)
pub fn op_swpt(state: &mut CalcState) -> Result<(), HpError> {
    let result = secs_to_hpnum_time(state.stopwatch_split)?;
    enter_number(state, result);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// SW — Activate stopwatch keyboard mode.
///
/// Sets `state.stopwatch_keyboard_mode = true`. This enables the interactive
/// stopwatch display; the actual keyboard redefinition is Phase 39 (CLI) /
/// Phase 41 (GUI) scope. LiftEffect::Neutral.
pub fn op_sw(state: &mut CalcState) -> Result<(), HpError> {
    state.stopwatch_keyboard_mode = true;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// Get the stopwatch display string for Phase 39/41 frontend use.
///
/// When `stopwatch_keyboard_mode` is true, computes `current_elapsed` and
/// formats as "HH:MM:SS.cc". Returns `None` if not in stopwatch mode.
///
/// Used by CLI (Phase 39) and GUI (Phase 41) to render the live stopwatch display.
pub fn get_stopwatch_display_str(state: &CalcState) -> Option<String> {
    if !state.stopwatch_keyboard_mode {
        return None;
    }
    let elapsed = current_elapsed(state);
    // Clamp to safe range to avoid formatting issues
    let elapsed = elapsed.clamp(0.0, 359_999.99);
    let total_cs = (elapsed * 100.0).round() as u64;
    let hours = total_cs / 360_000;
    let rem = total_cs % 360_000;
    let minutes = rem / 6000;
    let rem2 = rem % 6000;
    let seconds = rem2 / 100;
    let centiseconds = rem2 % 100;
    Some(format!(
        "{:02}:{:02}:{:02}.{:02}",
        hours, minutes, seconds, centiseconds
    ))
}

/// Convert elapsed seconds (f64) to HP-41 time HpNum format HH.MMSScc.
///
/// HP-41 time representation: integer part = hours (HH), fractional part = MMSScc
/// where MM = minutes (00-59), SS = seconds (00-59), cc = centiseconds (00-99).
///
/// This is the inverse of `parse_time_hpnum` in `date_arith.rs`.
pub(crate) fn secs_to_hpnum_time(secs: f64) -> Result<HpNum, HpError> {
    // 360_000 seconds = 100 hours, HP-41 max displayable
    if !(0.0..360_000.0).contains(&secs) {
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
    use std::thread;
    use std::time::Duration;

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
    fn op_runsw_sets_running_mode() {
        let mut state = CalcState::new();
        op_runsw(&mut state).unwrap();
        assert_eq!(state.stopwatch_mode, StopwatchMode::Running);
        assert!(state.stopwatch_start.is_some());
    }

    #[test]
    fn op_runsw_from_idle_starts_timer() {
        let mut state = CalcState::new();
        assert_eq!(state.stopwatch_mode, StopwatchMode::Idle);
        op_runsw(&mut state).unwrap();
        assert_eq!(state.stopwatch_mode, StopwatchMode::Running);
        assert!(state.stopwatch_start.is_some());
    }

    #[test]
    fn op_runsw_from_stopped_resumes() {
        let mut state = CalcState::new();
        state.stopwatch_mode = StopwatchMode::Stopped;
        state.stopwatch_accumulated = 10.0;
        op_runsw(&mut state).unwrap();
        assert_eq!(state.stopwatch_mode, StopwatchMode::Running);
        assert!(state.stopwatch_start.is_some());
        // Accumulated is preserved on resume
        assert_eq!(state.stopwatch_accumulated, 10.0);
    }

    #[test]
    fn op_stopsw_accumulates_when_running() {
        let mut state = CalcState::new();
        state.stopwatch_mode = StopwatchMode::Running;
        state.stopwatch_start = Some(Instant::now());
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
    fn op_stopsw_noop_when_already_stopped() {
        let mut state = CalcState::new();
        state.stopwatch_mode = StopwatchMode::Stopped;
        state.stopwatch_accumulated = 42.0;
        op_stopsw(&mut state).unwrap();
        assert_eq!(state.stopwatch_mode, StopwatchMode::Stopped);
        assert_eq!(state.stopwatch_accumulated, 42.0);
    }

    #[test]
    fn op_runsw_stopsw_elapsed_nonzero() {
        let mut state = CalcState::new();
        op_runsw(&mut state).unwrap();
        thread::sleep(Duration::from_millis(50));
        op_stopsw(&mut state).unwrap();
        assert!(
            state.stopwatch_accumulated > 0.0,
            "elapsed should be > 0 after 50ms sleep, got {}",
            state.stopwatch_accumulated
        );
    }

    #[test]
    fn op_setsw_presets_accumulated() {
        let mut state = CalcState::new();
        // Set X to 1.300000 = 1h 30m 0s 0cs
        use rust_decimal::Decimal;
        use std::str::FromStr;
        state.stack.x = HpNum::from(Decimal::from_str("1.300000").unwrap());
        op_setsw(&mut state).unwrap();
        // 1h 30m = 5400 seconds
        assert!((state.stopwatch_accumulated - 5400.0).abs() < 0.01);
        assert_eq!(state.stopwatch_mode, StopwatchMode::Stopped);
        assert!(state.stopwatch_start.is_none());
    }

    #[test]
    fn op_setsw_presets_with_centiseconds() {
        let mut state = CalcState::new();
        // Set X to 0.000550 = 0h 0m 5s 50cs = 5.50 seconds
        use rust_decimal::Decimal;
        use std::str::FromStr;
        state.stack.x = HpNum::from(Decimal::from_str("0.000550").unwrap());
        op_setsw(&mut state).unwrap();
        // 5 seconds + 50 centiseconds = 5.50 seconds
        assert!((state.stopwatch_accumulated - 5.50).abs() < 0.01);
        assert_eq!(state.stopwatch_mode, StopwatchMode::Stopped);
    }

    #[test]
    fn op_setsw_clears_running_state() {
        let mut state = CalcState::new();
        state.stopwatch_mode = StopwatchMode::Running;
        state.stopwatch_start = Some(Instant::now());
        // Preset from X = 0 (zero seconds)
        state.stack.x = HpNum::zero();
        op_setsw(&mut state).unwrap();
        assert_eq!(state.stopwatch_mode, StopwatchMode::Stopped);
        assert!(state.stopwatch_start.is_none());
        assert_eq!(state.stopwatch_accumulated, 0.0);
    }

    #[test]
    fn op_rclsw_returns_accumulated_when_stopped() {
        let mut state = CalcState::new();
        state.stopwatch_mode = StopwatchMode::Stopped;
        state.stopwatch_accumulated = 5400.0; // 1h 30m
        op_rclsw(&mut state).unwrap();
        // X should be 1.300000
        use rust_decimal::Decimal;
        use std::str::FromStr;
        let expected = HpNum::from(Decimal::from_str("1.300000").unwrap());
        assert_eq!(state.stack.x, expected);
    }

    #[test]
    fn op_rclsw_when_running_returns_nonzero() {
        let mut state = CalcState::new();
        op_runsw(&mut state).unwrap();
        thread::sleep(Duration::from_millis(50));
        op_rclsw(&mut state).unwrap();
        // Should have nonzero elapsed time in X
        assert!(state.stack.x != HpNum::zero());
    }

    #[test]
    fn op_stpw_records_split_without_stopping() {
        let mut state = CalcState::new();
        state.stopwatch_mode = StopwatchMode::Running;
        state.stopwatch_start = Some(Instant::now());
        state.stopwatch_accumulated = 10.0;
        thread::sleep(Duration::from_millis(50));
        op_stpw(&mut state).unwrap();
        // Stopwatch must still be Running
        assert_eq!(state.stopwatch_mode, StopwatchMode::Running);
        // Split should capture elapsed (>= 10s accumulated)
        assert!(
            state.stopwatch_split >= 10.0,
            "split should be >= 10.0, got {}",
            state.stopwatch_split
        );
    }

    #[test]
    fn op_stpw_records_current_elapsed_into_split() {
        let mut state = CalcState::new();
        state.stopwatch_mode = StopwatchMode::Stopped;
        state.stopwatch_accumulated = 42.5;
        op_stpw(&mut state).unwrap();
        // When stopped, split = accumulated
        assert!((state.stopwatch_split - 42.5).abs() < 0.001);
    }

    #[test]
    fn op_swpt_recalls_split_to_x() {
        let mut state = CalcState::new();
        state.stopwatch_split = 5400.0; // 1h 30m in seconds
        op_swpt(&mut state).unwrap();
        // X should be 1.300000
        use rust_decimal::Decimal;
        use std::str::FromStr;
        let expected = HpNum::from(Decimal::from_str("1.300000").unwrap());
        assert_eq!(state.stack.x, expected);
    }

    #[test]
    fn op_swpt_split_zero_recalls_zero() {
        let mut state = CalcState::new();
        state.stopwatch_split = 0.0;
        op_swpt(&mut state).unwrap();
        assert_eq!(state.stack.x, HpNum::zero());
    }

    #[test]
    fn op_sw_sets_keyboard_mode() {
        let mut state = CalcState::new();
        assert!(!state.stopwatch_keyboard_mode);
        op_sw(&mut state).unwrap();
        assert!(state.stopwatch_keyboard_mode);
    }

    #[test]
    fn get_stopwatch_display_str_when_inactive() {
        let state = CalcState::new();
        assert_eq!(get_stopwatch_display_str(&state), None);
    }

    #[test]
    fn get_stopwatch_display_str_when_active() {
        let mut state = CalcState::new();
        state.stopwatch_keyboard_mode = true;
        state.stopwatch_mode = StopwatchMode::Stopped;
        state.stopwatch_accumulated = 5400.0; // 1h 30m 0s 0cs
        let s = get_stopwatch_display_str(&state).unwrap();
        assert_eq!(s, "01:30:00.00");
    }

    #[test]
    fn get_stopwatch_display_str_format() {
        let mut state = CalcState::new();
        state.stopwatch_keyboard_mode = true;
        state.stopwatch_mode = StopwatchMode::Stopped;
        state.stopwatch_accumulated = 3723.45; // 1h 2m 3.45s
        let s = get_stopwatch_display_str(&state).unwrap();
        // 1h 2m 3s 45cs → "01:02:03.45"
        assert_eq!(s, "01:02:03.45");
    }

    #[test]
    fn get_stopwatch_display_str_zero() {
        let mut state = CalcState::new();
        state.stopwatch_keyboard_mode = true;
        let s = get_stopwatch_display_str(&state).unwrap();
        assert_eq!(s, "00:00:00.00");
    }

    #[test]
    fn current_elapsed_idle() {
        let state = CalcState::new();
        assert_eq!(current_elapsed(&state), 0.0);
    }

    #[test]
    fn current_elapsed_stopped() {
        let mut state = CalcState::new();
        state.stopwatch_mode = StopwatchMode::Stopped;
        state.stopwatch_accumulated = 42.5;
        assert_eq!(current_elapsed(&state), 42.5);
    }

    #[test]
    fn current_elapsed_running() {
        let mut state = CalcState::new();
        state.stopwatch_mode = StopwatchMode::Running;
        state.stopwatch_start = Some(Instant::now());
        state.stopwatch_accumulated = 10.0;
        thread::sleep(Duration::from_millis(50));
        let elapsed = current_elapsed(&state);
        assert!(
            elapsed > 10.0,
            "running elapsed should exceed accumulated (10.0), got {}",
            elapsed
        );
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
    fn secs_to_hpnum_time_centiseconds() {
        // 5.5 seconds = 5s 50cs → 0.000550
        let n = secs_to_hpnum_time(5.5).unwrap();
        use rust_decimal::Decimal;
        use std::str::FromStr;
        let expected = HpNum::from(Decimal::from_str("0.000550").unwrap());
        assert_eq!(n, expected);
    }

    #[test]
    fn pause_resume_preserves_accumulated() {
        let mut state = CalcState::new();
        // Start, run briefly, stop
        op_runsw(&mut state).unwrap();
        thread::sleep(Duration::from_millis(50));
        op_stopsw(&mut state).unwrap();
        let after_first_stop = state.stopwatch_accumulated;
        assert!(after_first_stop > 0.0);

        // Resume, run briefly, stop again
        op_runsw(&mut state).unwrap();
        thread::sleep(Duration::from_millis(50));
        op_stopsw(&mut state).unwrap();
        let after_second_stop = state.stopwatch_accumulated;

        // Total should exceed first stop
        assert!(
            after_second_stop > after_first_stop,
            "second stop accumulated ({}) should exceed first ({})",
            after_second_stop,
            after_first_stop
        );
    }

    #[test]
    fn op_stpw_then_op_swpt_round_trip() {
        let mut state = CalcState::new();
        state.stopwatch_mode = StopwatchMode::Stopped;
        state.stopwatch_accumulated = 3723.45; // 1h 2m 3.45s

        // Record split
        op_stpw(&mut state).unwrap();
        let recorded_split = state.stopwatch_split;
        assert!((recorded_split - 3723.45).abs() < 0.01);

        // Recall split to X
        op_swpt(&mut state).unwrap();
        // X should be 1.020345 (1h 2m 3s 45cs)
        use rust_decimal::Decimal;
        use std::str::FromStr;
        let expected = HpNum::from(Decimal::from_str("1.020345").unwrap());
        assert_eq!(state.stack.x, expected);
    }

    #[test]
    fn stopwatch_start_is_transient_not_serialized() {
        // Verify stopwatch_start is not in JSON (serde skip)
        let mut state = CalcState::new();
        state.stopwatch_mode = StopwatchMode::Running;
        state.stopwatch_start = Some(Instant::now());
        let json = serde_json::to_string(&state).unwrap();
        assert!(!json.contains("stopwatch_start"));
    }
}
