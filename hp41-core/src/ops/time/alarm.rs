// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Alarm operations for the HP Time Module.
//!
//! Alarms are stored in `CalcState::alarms: Vec<AlarmEntry>` (persistent,
//! `#[serde(default)]`). Each entry encodes a trigger time, repeat interval,
//! alarm type (message or control), and past-due flag.
//!
//! Phase 38 ships stub implementations that compile cleanly. Full alarm
//! logic (XYZALM time/date parsing, RCLALM/RCLAF recall, SETAF/CLALMA/CLALMX
//! management) lands in Wave 2 plans.

use crate::error::HpError;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;
use serde::{Deserialize, Serialize};

/// A single alarm entry stored in `CalcState::alarms`.
///
/// `trigger_unix`: seconds since Unix epoch when the alarm fires.
/// `repeat_secs`: repeat interval in seconds (0 = one-shot).
/// `alarm_type`: Message (display text) or Control (XEQ label, optional interrupt).
/// `past_due`: true if the alarm fired but the user has not acknowledged it.
///
/// D-38.8 / D-38.10: all fields are serde-persisted; `AlarmEntry` lives in
/// `CalcState::alarms` which carries `#[serde(default)]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmEntry {
    pub trigger_unix: i64,
    pub repeat_secs: i64,
    pub alarm_type: AlarmType,
    pub past_due: bool,
}

/// Alarm type discriminant for `AlarmEntry`.
///
/// `Message(String)`: display text on alarm trigger (analogous to BEEP + ALPHA).
/// `Control { label, interrupting }`: XEQ the named label; `interrupting = true`
///   corresponds to the `>>` prefix convention from the ALPHA register on XYZALM
///   (see `parse_alarm_type` helper).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlarmType {
    Message(String),
    Control { label: String, interrupting: bool },
}

/// Parse the ALPHA register content into an `AlarmType`.
///
/// Convention (HP 82182A OM §XYZALM): `>>label` = interrupting control,
/// `>label` = non-interrupting control, anything else = message text.
/// The longer prefix `>>` is checked FIRST (guards against shadowing `>`).
/// Phase 38 stub: used in Wave 2 XYZALM full implementation.
#[allow(dead_code)]
fn parse_alarm_type(alpha: &str) -> AlarmType {
    if let Some(label) = alpha.strip_prefix(">>") {
        AlarmType::Control {
            label: label.to_string(),
            interrupting: true,
        }
    } else if let Some(label) = alpha.strip_prefix('>') {
        AlarmType::Control {
            label: label.to_string(),
            interrupting: false,
        }
    } else {
        AlarmType::Message(alpha.to_string())
    }
}

/// XYZALM — Set alarm from stack X (time) and ALPHA register (message/label).
///
/// Phase 38 stub: opens the SETIME-style modal for alarm time entry.
/// Full implementation (parsing X as HH.MMSSss, building AlarmEntry, pushing
/// to `state.alarms`) lands in Wave 2.
pub fn op_xyzalm(state: &mut CalcState) -> Result<(), HpError> {
    // Stub: open modal for alarm-time entry (TIME? prompt).
    state.modal_program = Some(crate::ops::math1::modal::ModalProgram::Time(
        crate::ops::time::modal::TimeStep::XyzalmTimePrompt,
    ));
    state.modal_prompt = Some("ALARM TIME?".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ALMCAT — Alarm catalog browsing mode.
///
/// Phase 38 stub: sets `alarm_catalog_mode = true` and returns Ok.
/// Full implementation lands in Wave 2.
pub fn op_almcat(state: &mut CalcState) -> Result<(), HpError> {
    state.alarm_catalog_mode = true;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ALMNOW — Trigger all past-due alarms immediately.
///
/// Phase 38 stub: returns Ok (no-op).
pub fn op_almnow(state: &mut CalcState) -> Result<(), HpError> {
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// RCLALM — Recall next alarm entry to stack.
///
/// Phase 38 stub: returns `HpError::InvalidOp` (no alarms in scaffold).
pub fn op_rclalm(state: &mut CalcState) -> Result<(), HpError> {
    if state.alarms.is_empty() {
        return Err(HpError::InvalidOp);
    }
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// RCLAF — Recall alarm flags (count, etc.) to stack.
///
/// Phase 38 stub: pushes alarm count as HpNum.
pub fn op_rclaf(state: &mut CalcState) -> Result<(), HpError> {
    use crate::num::HpNum;
    use crate::stack::enter_number;
    let count = HpNum::from(state.alarms.len() as i32);
    enter_number(state, count);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// SETAF — Set alarm flags.
///
/// Phase 38 stub: no-op.
pub fn op_setaf(state: &mut CalcState) -> Result<(), HpError> {
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CLALMA — Clear all alarms.
///
/// Phase 38 stub: clears the alarms Vec.
pub fn op_clalma(state: &mut CalcState) -> Result<(), HpError> {
    state.alarms.clear();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CLALMX — Clear one alarm entry by index.
///
/// Phase 38 stub: removes the alarm at index X if valid.
pub fn op_clalmx(state: &mut CalcState) -> Result<(), HpError> {
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CLRALMS — Clear all alarms (alias for CLALMA used in TIME_MODULE ops slice).
///
/// Phase 38 stub: clears the alarms Vec.
pub fn op_clralms(state: &mut CalcState) -> Result<(), HpError> {
    state.alarms.clear();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// check_alarms — drain past-due alarms into the event and print buffers.
///
/// Called by frontend after every dispatch(), same cadence as print_buffer drain.
/// Phase 38 stub: no-op (alarm evaluation logic lands in Wave 2).
pub fn check_alarms(_state: &mut CalcState) {
    // Stub: alarm evaluation deferred to Wave 2.
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn alarm_entry_serde_round_trip() {
        let entry = AlarmEntry {
            trigger_unix: 1_700_000_000,
            repeat_secs: 3600,
            alarm_type: AlarmType::Message("Wake up!".to_string()),
            past_due: false,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let restored: AlarmEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.trigger_unix, entry.trigger_unix);
        assert_eq!(restored.repeat_secs, entry.repeat_secs);
        assert!(!restored.past_due);
        match restored.alarm_type {
            AlarmType::Message(ref msg) => assert_eq!(msg, "Wake up!"),
            _ => panic!("Wrong alarm type after round-trip"),
        }
    }

    #[test]
    fn alarm_type_control_serde_round_trip() {
        let at = AlarmType::Control {
            label: "MYPRG".to_string(),
            interrupting: true,
        };
        let json = serde_json::to_string(&at).unwrap();
        let restored: AlarmType = serde_json::from_str(&json).unwrap();
        match restored {
            AlarmType::Control { label, interrupting } => {
                assert_eq!(label, "MYPRG");
                assert!(interrupting);
            }
            _ => panic!("Wrong variant after round-trip"),
        }
    }

    #[test]
    fn parse_alarm_type_message() {
        let at = parse_alarm_type("Hello world");
        assert!(matches!(at, AlarmType::Message(ref s) if s == "Hello world"));
    }

    #[test]
    fn parse_alarm_type_non_interrupting_control() {
        let at = parse_alarm_type(">MYPRG");
        match at {
            AlarmType::Control { label, interrupting } => {
                assert_eq!(label, "MYPRG");
                assert!(!interrupting);
            }
            _ => panic!("Expected Control variant"),
        }
    }

    #[test]
    fn parse_alarm_type_interrupting_control() {
        let at = parse_alarm_type(">>INTPRG");
        match at {
            AlarmType::Control { label, interrupting } => {
                assert_eq!(label, "INTPRG");
                assert!(interrupting);
            }
            _ => panic!("Expected Control variant"),
        }
    }

    #[test]
    fn double_arrow_prefix_wins_over_single() {
        // ">>x" must parse as interrupting Control, not non-interrupting ">x"
        let at = parse_alarm_type(">>A");
        assert!(matches!(at, AlarmType::Control { interrupting: true, .. }));
    }

    #[test]
    fn op_clalma_clears_alarms() {
        let mut state = CalcState::new();
        state.alarms.push(AlarmEntry {
            trigger_unix: 1000,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("test".to_string()),
            past_due: false,
        });
        assert!(!state.alarms.is_empty());
        op_clalma(&mut state).unwrap();
        assert!(state.alarms.is_empty());
    }

    #[test]
    fn op_rclaf_returns_alarm_count() {
        let mut state = CalcState::new();
        state.alarms.push(AlarmEntry {
            trigger_unix: 1000,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("test".to_string()),
            past_due: false,
        });
        op_rclaf(&mut state).unwrap();
        assert_eq!(state.stack.x, crate::num::HpNum::from(1i32));
    }
}
