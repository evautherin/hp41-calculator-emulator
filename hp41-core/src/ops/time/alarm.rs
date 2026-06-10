// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Alarm operations for the HP Time Module.
//!
//! Alarms are stored in `CalcState::alarms: Vec<AlarmEntry>` (persistent,
//! `#[serde(default)]`). Each entry encodes a trigger time, repeat interval,
//! alarm type (message or control), and past-due flag.
//!
//! The alarm catalog supports up to 253 entries per HP 82182A hardware limit
//! (T-38-13 mitigated in `op_xyzalm`).
//!
//! Prefix parsing for ALPHA → AlarmType (per HP 82182A OM §XYZALM):
//!   `>>label` = interrupting control alarm
//!   `>label`  = non-interrupting control alarm
//!   anything else = message alarm
//!
//! The `check_alarms` function is called by the frontend after every dispatch(),
//! draining past-due alarms into `event_buffer` and `print_buffer` using the
//! same cadence as `print_buffer` drain (D-38.9).

use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::clock;
use super::date_arith::{date_to_jdn, parse_date_hpnum, parse_time_hpnum};

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

// ── Alarm Unix epoch base for JDN arithmetic ─────────────────────────────────

/// JDN for the Unix epoch date (1970-01-01).
/// Used to convert HP-41 calendar dates to Unix seconds.
const UNIX_EPOCH_JDN: i64 = 2_440_588;

// ── Private helpers ───────────────────────────────────────────────────────────

/// Parse the ALPHA register content into an `AlarmType`.
///
/// Convention (HP 82182A OM §XYZALM): `>>label` = interrupting control,
/// `>label` = non-interrupting control, anything else = message text.
///
/// IMPORTANT: The `>>` prefix is checked FIRST to avoid shadowing — if we
/// checked `>` first then `>>MYPROG` would parse as `>MYPROG` Control with
/// label `>MYPROG`. (Pitfall 9 — longer match wins.)
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

fn current_unix_secs(offset: i64) -> i64 {
    clock::adjusted_epoch_secs(offset)
}

/// Convert a calendar date + time of day to Unix epoch seconds.
///
/// Uses JDN arithmetic (Fliegel-Van Flandern 1968) to compute days since
/// Unix epoch (JDN 2440588), then adds time-of-day in seconds.
fn time_date_to_unix(hours: u8, minutes: u8, seconds: u8, year: i32, month: i32, day: i32) -> i64 {
    let jdn = date_to_jdn(year, month, day);
    let days_since_epoch = jdn - UNIX_EPOCH_JDN;
    let time_of_day_secs = (hours as i64) * 3600 + (minutes as i64) * 60 + (seconds as i64);
    days_since_epoch * 86_400 + time_of_day_secs
}

fn epoch_secs_to_date(epoch_secs: i64) -> (i32, i32, i32) {
    let (year, month, day, _, _, _) = clock::decompose_epoch_secs(epoch_secs);
    (year, month as i32, day as i32)
}

/// Build a time HpNum (HH.MMSScc) from components.
///
/// `centiseconds` is always 0 when reconstructing from Unix seconds (1-second
/// resolution). Uses Decimal string construction per P35 invariant.
fn make_time_hpnum(hours: u8, minutes: u8, seconds: u8) -> Result<HpNum, HpError> {
    let s = format!("{hours}.{minutes:02}{seconds:02}00");
    let d = Decimal::from_str(&s).map_err(|_| HpError::Overflow)?;
    Ok(HpNum::from(d))
}

/// Build a date HpNum from (year, month, day), using Flag 31 for MDY/DMY mode.
///
/// MDY (Flag 31 clear): MM.DDYYYY
/// DMY (Flag 31 set):   DD.MMYYYY
fn make_date_hpnum(year: i32, month: i32, day: i32, dmy: bool) -> Result<HpNum, HpError> {
    let s = if dmy {
        format!("{day}.{month:02}{year:04}")
    } else {
        format!("{month}.{day:02}{year:04}")
    };
    let d = Decimal::from_str(&s).map_err(|_| HpError::Overflow)?;
    Ok(HpNum::from(d))
}

/// Convert a repeat interval HpNum (HHHH.MMSScc) to total seconds (i64).
///
/// Uses the same field extraction as `parse_time_hpnum` but allows hours > 23
/// since repeat intervals can span multiple days (D-38.11).
fn repeat_hpnum_to_secs(hpnum: &HpNum) -> Result<i64, HpError> {
    let s = hpnum.inner().to_string();
    let (int_part, frac_part) = if let Some(pos) = s.find('.') {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s.as_str(), "")
    };
    let hours: i64 = int_part.parse().map_err(|_| HpError::InvalidInput)?;
    let padded = format!("{frac_part:0>6}");
    let minutes: i64 = padded[0..2].parse().map_err(|_| HpError::InvalidInput)?;
    let seconds: i64 = padded[2..4].parse().map_err(|_| HpError::InvalidInput)?;
    if minutes > 59 || seconds > 59 {
        return Err(HpError::InvalidInput);
    }
    Ok(hours * 3600 + minutes * 60 + seconds)
}

fn decompose_alarm_unix(trigger_unix: i64) -> (i32, i32, i32, u8, u8, u8) {
    let (year, month, day, hour, minute, second) = clock::decompose_epoch_secs(trigger_unix);
    (year, month as i32, day as i32, hour, minute, second)
}

// ── Alarm Op Implementations ──────────────────────────────────────────────────

/// XYZALM — Set alarm from stack registers + ALPHA.
///
/// Stack layout (HP 82182A OM §XYZALM):
///   X = trigger time as HH.MMSScc
///   Y = trigger date as MM.DDYYYY or DD.MMYYYY (Flag 31); 0 = today
///   Z = repeat interval as HHHH.MMSScc (0 = no repeat)
///
/// ALPHA register content determines alarm type:
///   `>>label` → interrupting control alarm
///   `>label`  → non-interrupting control alarm
///   anything  → message alarm
///
/// Cap: 253 alarms maximum (HP 82182A hardware limit, T-38-13).
/// Insertion: alarms are kept sorted chronologically by `trigger_unix`.
/// LiftEffect: Neutral.
pub fn op_xyzalm(state: &mut CalcState) -> Result<(), HpError> {
    // Enforce 253 alarm cap (T-38-13).
    if state.alarms.len() >= 253 {
        return Err(HpError::InvalidInput);
    }

    // Parse time from X.
    let (hours, minutes, seconds, _centiseconds) = parse_time_hpnum(&state.stack.x)?;

    // Parse date from Y. If Y is zero, use today's date.
    let dmy = state.flags & (1u64 << 31) != 0;
    let (year, month, day) = {
        let is_zero = state.stack.y.is_zero();
        if is_zero {
            // Y=0: use current date.
            let now = current_unix_secs(state.time_offset_secs);
            epoch_secs_to_date(now)
        } else {
            parse_date_hpnum(&state.stack.y, dmy)?
        }
    };

    // Parse repeat interval from Z.
    let repeat_secs = {
        let is_zero = state.stack.z.is_zero();
        if is_zero {
            0i64
        } else {
            repeat_hpnum_to_secs(&state.stack.z)?
        }
    };

    // Compute trigger_unix.
    let trigger_unix = time_date_to_unix(hours, minutes, seconds, year, month, day);

    // Parse alarm type from ALPHA register.
    let alarm_type = parse_alarm_type(&state.alpha_reg.clone());

    // Create alarm entry.
    let entry = AlarmEntry {
        trigger_unix,
        repeat_secs,
        alarm_type,
        past_due: false,
    };

    // Insert sorted chronologically (ascending trigger_unix).
    let pos = state
        .alarms
        .partition_point(|a| a.trigger_unix <= trigger_unix);
    state.alarms.insert(pos, entry);

    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// RCLALM — Recall alarm N fields to stack and ALPHA register.
///
/// X = alarm number (1-indexed, integer portion used).
/// After recall:
///   X = trigger time (HH.MMSScc)
///   Y = trigger date (MM.DDYYYY or DD.MMYYYY per Flag 31)
///   Z = repeat interval (HHHH.MMSScc; 0 if one-shot)
///   ALPHA = message text or `>label` / `>>label` form
///
/// Returns HpError::Data if alarm number is out of range.
/// LiftEffect: Enable.
pub fn op_rclalm(state: &mut CalcState) -> Result<(), HpError> {
    // Read X as 1-indexed alarm number (truncate to integer).
    let x_val = state.stack.x.inner();
    let num_i64 = x_val
        .trunc()
        .to_string()
        .parse::<i64>()
        .map_err(|_| HpError::InvalidInput)?;
    if num_i64 < 1 || num_i64 as usize > state.alarms.len() {
        return Err(HpError::InvalidInput);
    }
    let idx = (num_i64 - 1) as usize;
    let alarm = state.alarms[idx].clone();

    // Decompose trigger_unix to calendar + time fields.
    let (year, month, day, hour, minute, second) = decompose_alarm_unix(alarm.trigger_unix);

    // Build time HpNum.
    let time_hpnum = make_time_hpnum(hour, minute, second)?;

    // Build date HpNum using Flag 31.
    let dmy = state.flags & (1u64 << 31) != 0;
    let date_hpnum = make_date_hpnum(year, month, day, dmy)?;

    // Build repeat interval HpNum (0 if one-shot).
    let repeat_hpnum = if alarm.repeat_secs == 0 {
        HpNum::zero()
    } else {
        let total = alarm.repeat_secs;
        let h = total / 3600;
        let m = (total % 3600) / 60;
        let s = total % 60;
        let s_str = format!("{h}.{m:02}{s:02}00");
        let d = Decimal::from_str(&s_str).map_err(|_| HpError::Overflow)?;
        HpNum::from(d)
    };

    // Push Z (repeat), Y (date), X (time) — need three pushes.
    // First push repeat into Z slot by lifting the stack three times.
    // Use enter_number with lift semantics: push repeat first, then date, then time.
    // HP-41 RCL pattern: enable lift, push Z value, push Y value, push X value.
    state.stack.lift_enabled = true;
    enter_number(state, repeat_hpnum); // becomes X
    state.stack.lift_enabled = true;
    enter_number(state, date_hpnum); // becomes X, repeat goes to Y
    state.stack.lift_enabled = true;
    enter_number(state, time_hpnum); // becomes X, date goes to Y, repeat goes to Z

    // Set ALPHA register.
    state.alpha_reg = match &alarm.alarm_type {
        AlarmType::Message(msg) => msg.clone(),
        AlarmType::Control {
            label,
            interrupting,
        } => {
            if *interrupting {
                format!(">>{label}")
            } else {
                format!(">{label}")
            }
        }
    };

    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ALMCAT — Alarm catalog browsing mode.
///
/// Sets `alarm_catalog_mode = true` and writes the first alarm's display
/// string to `print_buffer` if any alarms exist. Frontend handles navigation
/// (Phase 39/41).
/// LiftEffect: Neutral.
pub fn op_almcat(state: &mut CalcState) -> Result<(), HpError> {
    state.alarm_catalog_mode = true;
    if let Some(first) = state.alarms.first() {
        let (year, month, day, hour, minute, second) = decompose_alarm_unix(first.trigger_unix);
        let msg = match &first.alarm_type {
            AlarmType::Message(m) => m.clone(),
            AlarmType::Control { label, .. } => label.clone(),
        };
        let line = format!("{day:02}.{month:02}.{year:04} {hour:02}:{minute:02}:{second:02} {msg}");
        state.print_buffer.push(line);
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ALMNOW — Trigger the oldest past-due alarm immediately.
///
/// If a past-due alarm exists, dispatch its event:
///   Message alarm → "alarm:message:{msg}" to event_buffer + msg to print_buffer
///   Non-interrupting control → "alarm:xeq:{label}" to event_buffer
///   Interrupting control → "alarm:xeq:{label}" to event_buffer
///     (defer_to_run_loop=false — ALMNOW owns its own synchronous ack; D-38.4 / Phase 63)
///
/// After dispatch: if repeating, advance trigger_unix and reset past_due.
/// If not repeating, remove from catalog.
///
/// If no past-due alarm, find the next upcoming alarm and trigger it immediately.
/// LiftEffect: Neutral.
pub fn op_almnow(state: &mut CalcState) -> Result<(), HpError> {
    // Find first past-due alarm.
    let idx_opt = state.alarms.iter().position(|a| a.past_due);

    let idx = if let Some(i) = idx_opt {
        i
    } else {
        // No past-due: find the next upcoming alarm (lowest trigger_unix not past-due).
        match state.alarms.iter().position(|a| !a.past_due) {
            Some(i) => i,
            None => return Ok(()), // No alarms at all.
        }
    };

    // Trigger the alarm at idx.
    let alarm = state.alarms[idx].clone();
    // defer_to_run_loop=false: op_almnow owns its own synchronous ack (lines below).
    // Passing true here would let dispatch_alarm_event set pending_interrupt when
    // is_running==true, then run_loop's RTN-ack would try to ack an index that
    // op_almnow has already removed or rescheduled — double-ack / stale-index hazard.
    // ALMNOW is an explicit "fire now" command; its ack path is self-contained.
    dispatch_alarm_event(state, &alarm, idx, false);

    // Acknowledge: reschedule or remove.
    if alarm.repeat_secs > 0 {
        state.alarms[idx].trigger_unix += alarm.repeat_secs;
        state.alarms[idx].past_due = false;
    } else {
        state.alarms.remove(idx);
    }

    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CLALMA — Clear alarm matching ALPHA register content.
///
/// Finds first alarm whose `alarm_type` matches `state.alpha_reg`:
///   - Message alarm: message text == alpha_reg
///   - Control alarm: ">label" or ">>label" form matches alpha_reg
///
/// Returns HpError::Data if no matching alarm found.
/// LiftEffect: Neutral.
pub fn op_clalma(state: &mut CalcState) -> Result<(), HpError> {
    let alpha = state.alpha_reg.clone();
    let idx_opt = state
        .alarms
        .iter()
        .position(|a| alarm_matches_alpha(&a.alarm_type, &alpha));
    match idx_opt {
        Some(idx) => {
            state.alarms.remove(idx);
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        None => Err(HpError::InvalidInput),
    }
}

/// CLALMX — Clear alarm at position X (1-indexed).
///
/// Removes alarm at 1-indexed position given by integer portion of X.
/// Returns HpError::InvalidInput if position is out of range.
/// LiftEffect: Neutral.
pub fn op_clalmx(state: &mut CalcState) -> Result<(), HpError> {
    let x_val = state.stack.x.inner();
    let num_i64 = x_val
        .trunc()
        .to_string()
        .parse::<i64>()
        .map_err(|_| HpError::InvalidInput)?;
    if num_i64 < 1 || num_i64 as usize > state.alarms.len() {
        return Err(HpError::InvalidInput);
    }
    let idx = (num_i64 - 1) as usize;
    state.alarms.remove(idx);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// CLRALMS — Clear all alarms.
///
/// Empties `state.alarms`.
/// LiftEffect: Neutral.
pub fn op_clralms(state: &mut CalcState) -> Result<(), HpError> {
    state.alarms.clear();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

// ── check_alarms drain function ───────────────────────────────────────────────

/// Drain past-due alarms into the event and print buffers.
///
/// Called by the frontend after every `dispatch()`, same cadence as
/// `print_buffer` drain (D-38.9). Scans all alarms for any with
/// `!past_due && trigger_unix <= now`. For each newly past-due alarm:
///
///   Message alarm → `"alarm:message:{text}"` pushed to `event_buffer`
///                   + `text` pushed to `print_buffer`
///   Non-interrupting control → `"alarm:xeq:{label}"` to `event_buffer`
///   Interrupting control (Phase 63):
///     Running + no pending + no solver/modal → `pending_interrupt = Some(label)` (D-12)
///     Idle / already-pending / solver-demoted → `"alarm:xeq:{label}"` to `event_buffer` (D-10/D-13)
///
/// The alarm's `past_due` flag is set to true but the alarm is NOT removed
/// (that happens on acknowledgment via `acknowledge_alarm`).
pub fn check_alarms(state: &mut CalcState) {
    let now = current_unix_secs(state.time_offset_secs);
    let len = state.alarms.len();
    for i in 0..len {
        if !state.alarms[i].past_due && state.alarms[i].trigger_unix <= now {
            state.alarms[i].past_due = true;
            let alarm_type = state.alarms[i].alarm_type.clone();
            dispatch_alarm_event(
                state,
                &AlarmEntry {
                    trigger_unix: state.alarms[i].trigger_unix,
                    repeat_secs: state.alarms[i].repeat_secs,
                    alarm_type,
                    past_due: true,
                },
                i,
                true, // defer_to_run_loop: check_alarms defers interrupting alarms to run_loop
            );
        }
    }
}

/// Acknowledge an alarm by index.
///
/// If the alarm has `repeat_secs > 0`, advance `trigger_unix += repeat_secs`
/// and reset `past_due = false` (reschedule). Otherwise remove from catalog.
///
/// Public helper for frontend acknowledge actions (Phase 39/41).
pub fn acknowledge_alarm(state: &mut CalcState, index: usize) -> Result<(), HpError> {
    if index >= state.alarms.len() {
        return Err(HpError::InvalidInput);
    }
    if state.alarms[index].repeat_secs > 0 {
        let repeat = state.alarms[index].repeat_secs;
        state.alarms[index].trigger_unix += repeat;
        state.alarms[index].past_due = false;
    } else {
        state.alarms.remove(index);
    }
    Ok(())
}

// ── Private dispatch helper ───────────────────────────────────────────────────

/// Push the correct event string(s) for an alarm trigger.
///
/// Called from both `check_alarms` and `op_almnow`.
///
/// Parameters:
/// - `alarm`: cloned `AlarmEntry` snapshot at trigger time.
/// - `index`: index of this alarm in `state.alarms` (for `pending_interrupt_alarm_index`).
/// - `defer_to_run_loop`: when `true`, an interrupting alarm that meets the injection
///   criteria sets `pending_interrupt` and defers execution to `run_loop` (Phase 63).
///   When `false` (op_almnow), the caller owns its own ack — falling through to the
///   `alarm:xeq:{label}` event push avoids a double-ack / stale-index hazard.
///
/// Routing (Phase 63 — D-12 / D-10 / D-13 / DNT-05):
///   Message alarm      → `"alarm:message:{msg}"` + print_buffer (unchanged).
///   Non-interrupting   → `"alarm:xeq:{label}"` to event_buffer (DNT-05, unchanged).
///   Interrupting + defer_to_run_loop=true + is_running + no pending + no solver/modal
///                      → `pending_interrupt = Some(label)` + `pending_interrupt_alarm_index = Some(index)`.
///   Interrupting otherwise (idle D-13, nested D-nesting, solver/modal D-10, defer=false)
///                      → `"alarm:xeq:{label}"` to event_buffer (same as non-interrupting).
fn dispatch_alarm_event(
    state: &mut CalcState,
    alarm: &AlarmEntry,
    index: usize,
    defer_to_run_loop: bool,
) {
    match &alarm.alarm_type {
        AlarmType::Message(msg) => {
            // DNT-05: message arm unchanged.
            state.event_buffer.push(format!("alarm:message:{msg}"));
            state.print_buffer.push(msg.clone());
        }
        AlarmType::Control {
            label,
            interrupting,
        } => {
            if *interrupting && defer_to_run_loop {
                // D-13/D-10: only inject into run_loop when a program is running,
                // no interrupt already pending, and no solver/modal is mid-execution.
                // Otherwise demote to the proven non-interrupting event path:
                //   - idle-fire (D-13): is_running == false → alarm:xeq:{label}
                //   - nesting guard: pending_interrupt already Some → alarm:xeq:{label}
                //   - solver/modal demotion (D-10): avoid corrupting re-entrancy state
                let solver_active = state.integ_state.is_some()
                    || state.solve_state.is_some()
                    || state.difeq_state.is_some()
                    || state.modal_program.is_some();
                if state.is_running && state.pending_interrupt.is_none() && !solver_active {
                    // D-12: set the pending interrupt; run_loop injects the synthetic frame.
                    state.pending_interrupt = Some(label.clone());
                    // D-06a: pair the alarm index so run_loop can ack the right one after RTN.
                    state.pending_interrupt_alarm_index = Some(index);
                } else {
                    // Demote: idle, already-pending, or solver/modal active.
                    state.event_buffer.push(format!("alarm:xeq:{label}"));
                }
            } else {
                // DNT-05: non-interrupting arm unchanged; or defer_to_run_loop=false (op_almnow).
                state.event_buffer.push(format!("alarm:xeq:{label}"));
            }
        }
    }
}

/// Check whether an `AlarmType` matches the given ALPHA register content.
///
/// Message alarm: matches if `message == alpha`.
/// Control alarm: matches if the `>label` or `>>label` form equals `alpha`.
fn alarm_matches_alpha(alarm_type: &AlarmType, alpha: &str) -> bool {
    match alarm_type {
        AlarmType::Message(msg) => msg == alpha,
        AlarmType::Control {
            label,
            interrupting,
        } => {
            let expected = if *interrupting {
                format!(">>{label}")
            } else {
                format!(">{label}")
            };
            expected == alpha
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::num::HpNum;
    use crate::stack::enter_number;
    use crate::state::CalcState;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    // ── Serde round-trip (from stub, preserved) ───────────────────────────────

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
            AlarmType::Control {
                label,
                interrupting,
            } => {
                assert_eq!(label, "MYPRG");
                assert!(interrupting);
            }
            _ => panic!("Wrong variant after round-trip"),
        }
    }

    // ── parse_alarm_type ─────────────────────────────────────────────────────

    #[test]
    fn parse_alarm_type_message() {
        let at = parse_alarm_type("Hello world");
        assert!(matches!(at, AlarmType::Message(ref s) if s == "Hello world"));
    }

    #[test]
    fn parse_alarm_type_non_interrupting_control() {
        let at = parse_alarm_type(">MYPRG");
        match at {
            AlarmType::Control {
                label,
                interrupting,
            } => {
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
            AlarmType::Control {
                label,
                interrupting,
            } => {
                assert_eq!(label, "INTPRG");
                assert!(interrupting);
            }
            _ => panic!("Expected Control variant"),
        }
    }

    #[test]
    fn double_arrow_prefix_wins_over_single() {
        // ">>x" must parse as interrupting Control, not non-interrupting ">x"
        // (Pitfall 9 — longer match checked first)
        let at = parse_alarm_type(">>A");
        assert!(matches!(
            at,
            AlarmType::Control {
                interrupting: true,
                ..
            }
        ));
    }

    #[test]
    fn parse_alarm_type_single_arrow_edge_case() {
        // ">" with nothing after → non-interrupting with empty label
        let at = parse_alarm_type(">");
        match at {
            AlarmType::Control {
                label,
                interrupting,
            } => {
                assert_eq!(label, "");
                assert!(!interrupting);
            }
            _ => panic!("Expected Control variant"),
        }
    }

    #[test]
    fn parse_alarm_type_double_arrow_edge_case() {
        // ">>" with nothing after → interrupting with empty label
        let at = parse_alarm_type(">>");
        match at {
            AlarmType::Control {
                label,
                interrupting,
            } => {
                assert_eq!(label, "");
                assert!(interrupting);
            }
            _ => panic!("Expected Control variant"),
        }
    }

    #[test]
    fn parse_alarm_type_empty_string_is_message() {
        let at = parse_alarm_type("");
        assert!(matches!(at, AlarmType::Message(ref s) if s.is_empty()));
    }

    // ── time_date_to_unix / decompose_alarm_unix round-trip ──────────────────

    #[test]
    fn time_date_to_unix_round_trip() {
        // 2026-05-24 08:30:00
        let ts = time_date_to_unix(8, 30, 0, 2026, 5, 24);
        let (year, month, day, hour, min, sec) = decompose_alarm_unix(ts);
        assert_eq!((year, month, day), (2026, 5, 24));
        assert_eq!((hour, min, sec), (8, 30, 0));
    }

    #[test]
    fn time_date_to_unix_unix_epoch() {
        // 1970-01-01 00:00:00 = Unix epoch = 0
        let ts = time_date_to_unix(0, 0, 0, 1970, 1, 1);
        assert_eq!(ts, 0);
    }

    // ── op_xyzalm ────────────────────────────────────────────────────────────

    /// Helper: push a Decimal string onto X, shifting stack.
    fn push_x(state: &mut CalcState, s: &str) {
        let d = Decimal::from_str(s).unwrap();
        state.stack.lift_enabled = true;
        enter_number(state, HpNum::from(d));
    }

    /// Set up stack X=time, Y=date, Z=repeat for XYZALM.
    fn setup_xyzalm_stack(state: &mut CalcState, z_repeat: &str, y_date: &str, x_time: &str) {
        // Push in order: Z first, then Y, then X (so X is on top)
        push_x(state, z_repeat); // Z
        push_x(state, y_date); // Y
        push_x(state, x_time); // X
    }

    #[test]
    fn op_xyzalm_stores_message_alarm() {
        let mut state = CalcState::new();
        state.alpha_reg = "Wake up".to_string();
        // X = 8.000000 (8:00:00), Y = 1.012026 (Jan 01 2026 MDY), Z = 0 (no repeat)
        setup_xyzalm_stack(&mut state, "0", "1.012026", "8.000000");
        op_xyzalm(&mut state).unwrap();
        assert_eq!(state.alarms.len(), 1);
        let alarm = &state.alarms[0];
        assert_eq!(alarm.repeat_secs, 0);
        assert!(!alarm.past_due);
        match &alarm.alarm_type {
            AlarmType::Message(msg) => assert_eq!(msg, "Wake up"),
            _ => panic!("Expected Message alarm"),
        }
        // Verify time: Jan 1 2026 08:00:00
        let (year, month, day, hour, min, sec) = decompose_alarm_unix(alarm.trigger_unix);
        assert_eq!((year, month, day), (2026, 1, 1));
        assert_eq!((hour, min, sec), (8, 0, 0));
    }

    #[test]
    fn op_xyzalm_stores_non_interrupting_control() {
        let mut state = CalcState::new();
        state.alpha_reg = ">MYPROG".to_string();
        setup_xyzalm_stack(&mut state, "0", "5.242026", "14.30");
        op_xyzalm(&mut state).unwrap();
        assert_eq!(state.alarms.len(), 1);
        match &state.alarms[0].alarm_type {
            AlarmType::Control {
                label,
                interrupting,
            } => {
                assert_eq!(label, "MYPROG");
                assert!(!interrupting);
            }
            _ => panic!("Expected Control alarm"),
        }
    }

    #[test]
    fn op_xyzalm_stores_interrupting_control() {
        let mut state = CalcState::new();
        state.alpha_reg = ">>INTPROG".to_string();
        setup_xyzalm_stack(&mut state, "0", "5.242026", "9.000000");
        op_xyzalm(&mut state).unwrap();
        match &state.alarms[0].alarm_type {
            AlarmType::Control {
                label,
                interrupting,
            } => {
                assert_eq!(label, "INTPROG");
                assert!(*interrupting);
            }
            _ => panic!("Expected interrupting Control alarm"),
        }
    }

    #[test]
    fn op_xyzalm_enforces_253_cap() {
        let mut state = CalcState::new();
        // Fill to 253 alarms.
        for i in 0..253i64 {
            state.alarms.push(AlarmEntry {
                trigger_unix: 1_700_000_000 + i,
                repeat_secs: 0,
                alarm_type: AlarmType::Message("x".to_string()),
                past_due: false,
            });
        }
        state.alpha_reg = "overflow".to_string();
        setup_xyzalm_stack(&mut state, "0", "5.242026", "9.000000");
        let result = op_xyzalm(&mut state);
        assert!(
            matches!(result, Err(crate::error::HpError::InvalidInput)),
            "Expected HpError::Data for overflow"
        );
    }

    #[test]
    fn op_xyzalm_inserts_sorted_chronologically() {
        let mut state = CalcState::new();
        // Insert later alarm first.
        state.alpha_reg = "late".to_string();
        setup_xyzalm_stack(&mut state, "0", "5.242026", "18.000000"); // 18:00
        op_xyzalm(&mut state).unwrap();

        // Insert earlier alarm second — should sort before the late one.
        state.alpha_reg = "early".to_string();
        setup_xyzalm_stack(&mut state, "0", "5.242026", "8.000000"); // 08:00
        op_xyzalm(&mut state).unwrap();

        assert_eq!(state.alarms.len(), 2);
        // First alarm should be the earlier one.
        match &state.alarms[0].alarm_type {
            AlarmType::Message(m) => assert_eq!(m, "early"),
            _ => panic!("Expected message"),
        }
        match &state.alarms[1].alarm_type {
            AlarmType::Message(m) => assert_eq!(m, "late"),
            _ => panic!("Expected message"),
        }
    }

    #[test]
    fn op_xyzalm_with_repeat_interval() {
        let mut state = CalcState::new();
        state.alpha_reg = "hourly".to_string();
        // Z = 1.000000 (1 hour repeat = 3600 secs), Y = date, X = time
        setup_xyzalm_stack(&mut state, "1.000000", "5.242026", "9.000000");
        op_xyzalm(&mut state).unwrap();
        assert_eq!(state.alarms[0].repeat_secs, 3600);
    }

    #[test]
    fn op_xyzalm_lift_effect_neutral() {
        let mut state = CalcState::new();
        state.stack.lift_enabled = true;
        state.alpha_reg = "test".to_string();
        setup_xyzalm_stack(&mut state, "0", "5.242026", "9.000000");
        op_xyzalm(&mut state).unwrap();
        // Neutral: lift_enabled unchanged from true
        assert!(state.stack.lift_enabled);
    }

    // ── op_rclalm ────────────────────────────────────────────────────────────

    #[test]
    fn op_rclalm_recalls_message_alarm() {
        let mut state = CalcState::new();
        // Create alarm: 2026-05-24 08:30:00, no repeat, message
        let ts = time_date_to_unix(8, 30, 0, 2026, 5, 24);
        state.alarms.push(AlarmEntry {
            trigger_unix: ts,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("Hello".to_string()),
            past_due: false,
        });

        // Put X=1 (first alarm).
        state.stack.x = HpNum::from(1i32);
        op_rclalm(&mut state).unwrap();

        // X should be time: 8.303000 (8h30m0s centisecs=0) → "8.303000"
        // Actually: HH.MMSScc → 8.300000
        let x_str = state.stack.x.inner().to_string();
        assert!(x_str.starts_with("8.3"), "Time X: {x_str}");

        // Y should be date: 5.242026 (MDY: month=5, day=24, year=2026)
        let y_str = state.stack.y.inner().to_string();
        assert!(y_str.starts_with("5.24"), "Date Y: {y_str}");

        // Z should be 0 (no repeat).
        // LINT-EXEMPT: exact integer equality — HpNum::zero() is Decimal(0), no f64 bridge or drift
        assert_eq!(state.stack.z, HpNum::zero());

        // ALPHA should be "Hello".
        assert_eq!(state.alpha_reg, "Hello");
    }

    #[test]
    fn op_rclalm_recalls_control_alarm_to_alpha() {
        let mut state = CalcState::new();
        let ts = time_date_to_unix(12, 0, 0, 2026, 6, 1);
        state.alarms.push(AlarmEntry {
            trigger_unix: ts,
            repeat_secs: 7200,
            alarm_type: AlarmType::Control {
                label: "PROG1".to_string(),
                interrupting: false,
            },
            past_due: false,
        });
        state.stack.x = HpNum::from(1i32);
        op_rclalm(&mut state).unwrap();
        assert_eq!(state.alpha_reg, ">PROG1");
    }

    #[test]
    fn op_rclalm_recalls_interrupting_control_to_alpha() {
        let mut state = CalcState::new();
        let ts = time_date_to_unix(9, 0, 0, 2026, 6, 1);
        state.alarms.push(AlarmEntry {
            trigger_unix: ts,
            repeat_secs: 0,
            alarm_type: AlarmType::Control {
                label: "IPROG".to_string(),
                interrupting: true,
            },
            past_due: false,
        });
        state.stack.x = HpNum::from(1i32);
        op_rclalm(&mut state).unwrap();
        assert_eq!(state.alpha_reg, ">>IPROG");
    }

    #[test]
    fn op_rclalm_recalls_repeat_interval() {
        let mut state = CalcState::new();
        let ts = time_date_to_unix(7, 0, 0, 2026, 5, 1);
        state.alarms.push(AlarmEntry {
            trigger_unix: ts,
            repeat_secs: 3600, // 1 hour
            alarm_type: AlarmType::Message("daily".to_string()),
            past_due: false,
        });
        state.stack.x = HpNum::from(1i32);
        op_rclalm(&mut state).unwrap();
        // Z should be repeat: 1.000000 (1h 0m 0s)
        let z_str = state.stack.z.inner().to_string();
        assert!(z_str.starts_with("1"), "Repeat Z: {z_str}");
    }

    #[test]
    fn op_rclalm_out_of_range_returns_data_error() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(1i32); // No alarms.
        let result = op_rclalm(&mut state);
        assert!(matches!(result, Err(crate::error::HpError::InvalidInput)));
    }

    #[test]
    fn op_rclalm_zero_returns_data_error() {
        let mut state = CalcState::new();
        state.alarms.push(AlarmEntry {
            trigger_unix: 1_700_000_000,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("x".to_string()),
            past_due: false,
        });
        state.stack.x = HpNum::from(0i32);
        let result = op_rclalm(&mut state);
        assert!(matches!(result, Err(crate::error::HpError::InvalidInput)));
    }

    #[test]
    fn op_rclalm_lift_effect_enable() {
        let mut state = CalcState::new();
        state.stack.lift_enabled = false;
        let ts = time_date_to_unix(8, 0, 0, 2026, 5, 24);
        state.alarms.push(AlarmEntry {
            trigger_unix: ts,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("m".to_string()),
            past_due: false,
        });
        state.stack.x = HpNum::from(1i32);
        op_rclalm(&mut state).unwrap();
        assert!(state.stack.lift_enabled);
    }

    // ── op_clralms ───────────────────────────────────────────────────────────

    #[test]
    fn op_clralms_clears_all_alarms() {
        let mut state = CalcState::new();
        for _ in 0..3 {
            state.alarms.push(AlarmEntry {
                trigger_unix: 1000,
                repeat_secs: 0,
                alarm_type: AlarmType::Message("x".to_string()),
                past_due: false,
            });
        }
        assert_eq!(state.alarms.len(), 3);
        op_clralms(&mut state).unwrap();
        assert!(state.alarms.is_empty());
    }

    #[test]
    fn op_clralms_on_empty_is_ok() {
        let mut state = CalcState::new();
        assert!(op_clralms(&mut state).is_ok());
        assert!(state.alarms.is_empty());
    }

    // ── op_clalmx ────────────────────────────────────────────────────────────

    #[test]
    fn op_clalmx_removes_by_index() {
        let mut state = CalcState::new();
        state.alarms.push(AlarmEntry {
            trigger_unix: 1000,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("first".to_string()),
            past_due: false,
        });
        state.alarms.push(AlarmEntry {
            trigger_unix: 2000,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("second".to_string()),
            past_due: false,
        });
        // Remove alarm at position 1 (first).
        state.stack.x = HpNum::from(1i32);
        op_clalmx(&mut state).unwrap();
        assert_eq!(state.alarms.len(), 1);
        match &state.alarms[0].alarm_type {
            AlarmType::Message(m) => assert_eq!(m, "second"),
            _ => panic!("Wrong alarm after clalmx"),
        }
    }

    #[test]
    fn op_clalmx_out_of_range_returns_data_error() {
        let mut state = CalcState::new();
        state.alarms.push(AlarmEntry {
            trigger_unix: 1000,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("x".to_string()),
            past_due: false,
        });
        state.stack.x = HpNum::from(2i32); // Only 1 alarm.
        let result = op_clalmx(&mut state);
        assert!(matches!(result, Err(crate::error::HpError::InvalidInput)));
    }

    #[test]
    fn op_clalmx_zero_returns_data_error() {
        let mut state = CalcState::new();
        state.alarms.push(AlarmEntry {
            trigger_unix: 1000,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("x".to_string()),
            past_due: false,
        });
        state.stack.x = HpNum::from(0i32);
        let result = op_clalmx(&mut state);
        assert!(matches!(result, Err(crate::error::HpError::InvalidInput)));
    }

    // ── op_clalma ────────────────────────────────────────────────────────────

    #[test]
    fn op_clalma_removes_by_message_text() {
        let mut state = CalcState::new();
        state.alarms.push(AlarmEntry {
            trigger_unix: 1000,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("Wake up".to_string()),
            past_due: false,
        });
        state.alarms.push(AlarmEntry {
            trigger_unix: 2000,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("Lunch".to_string()),
            past_due: false,
        });
        state.alpha_reg = "Wake up".to_string();
        op_clalma(&mut state).unwrap();
        assert_eq!(state.alarms.len(), 1);
        match &state.alarms[0].alarm_type {
            AlarmType::Message(m) => assert_eq!(m, "Lunch"),
            _ => panic!("Wrong alarm remaining"),
        }
    }

    #[test]
    fn op_clalma_removes_by_control_label_non_interrupting() {
        let mut state = CalcState::new();
        state.alarms.push(AlarmEntry {
            trigger_unix: 1000,
            repeat_secs: 0,
            alarm_type: AlarmType::Control {
                label: "MYPROG".to_string(),
                interrupting: false,
            },
            past_due: false,
        });
        state.alpha_reg = ">MYPROG".to_string();
        op_clalma(&mut state).unwrap();
        assert!(state.alarms.is_empty());
    }

    #[test]
    fn op_clalma_removes_by_control_label_interrupting() {
        let mut state = CalcState::new();
        state.alarms.push(AlarmEntry {
            trigger_unix: 1000,
            repeat_secs: 0,
            alarm_type: AlarmType::Control {
                label: "IPROG".to_string(),
                interrupting: true,
            },
            past_due: false,
        });
        state.alpha_reg = ">>IPROG".to_string();
        op_clalma(&mut state).unwrap();
        assert!(state.alarms.is_empty());
    }

    #[test]
    fn op_clalma_no_match_returns_data_error() {
        let mut state = CalcState::new();
        state.alarms.push(AlarmEntry {
            trigger_unix: 1000,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("Wake up".to_string()),
            past_due: false,
        });
        state.alpha_reg = "Nonexistent".to_string();
        let result = op_clalma(&mut state);
        assert!(matches!(result, Err(crate::error::HpError::InvalidInput)));
        // Alarm should still be there.
        assert_eq!(state.alarms.len(), 1);
    }

    // ── op_almcat ────────────────────────────────────────────────────────────

    #[test]
    fn op_almcat_sets_catalog_mode() {
        let mut state = CalcState::new();
        assert!(!state.alarm_catalog_mode);
        op_almcat(&mut state).unwrap();
        assert!(state.alarm_catalog_mode);
    }

    #[test]
    fn op_almcat_pushes_first_alarm_to_print_buffer() {
        let mut state = CalcState::new();
        let ts = time_date_to_unix(8, 0, 0, 2026, 5, 24);
        state.alarms.push(AlarmEntry {
            trigger_unix: ts,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("morning".to_string()),
            past_due: false,
        });
        op_almcat(&mut state).unwrap();
        assert!(!state.print_buffer.is_empty());
        assert!(
            state.print_buffer[0].contains("morning"),
            "Print buffer: {:?}",
            state.print_buffer
        );
    }

    #[test]
    fn op_almcat_empty_alarms_no_print() {
        let mut state = CalcState::new();
        op_almcat(&mut state).unwrap();
        assert!(state.alarm_catalog_mode);
        assert!(state.print_buffer.is_empty());
    }

    // ── check_alarms ─────────────────────────────────────────────────────────

    #[test]
    fn check_alarms_marks_past_due_and_pushes_message_event() {
        let mut state = CalcState::new();
        // Alarm in the past (trigger_unix = 1000 seconds after epoch).
        state.alarms.push(AlarmEntry {
            trigger_unix: 1000,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("alert!".to_string()),
            past_due: false,
        });
        check_alarms(&mut state);
        assert!(state.alarms[0].past_due, "Alarm should be marked past_due");
        assert!(
            state
                .event_buffer
                .contains(&"alarm:message:alert!".to_string()),
            "event_buffer: {:?}",
            state.event_buffer
        );
        assert!(
            state.print_buffer.contains(&"alert!".to_string()),
            "print_buffer: {:?}",
            state.print_buffer
        );
    }

    #[test]
    fn check_alarms_non_interrupting_control_pushes_xeq_event() {
        let mut state = CalcState::new();
        state.alarms.push(AlarmEntry {
            trigger_unix: 500,
            repeat_secs: 0,
            alarm_type: AlarmType::Control {
                label: "MYPROG".to_string(),
                interrupting: false,
            },
            past_due: false,
        });
        check_alarms(&mut state);
        assert!(state.alarms[0].past_due);
        assert!(
            state.event_buffer.contains(&"alarm:xeq:MYPROG".to_string()),
            "event_buffer: {:?}",
            state.event_buffer
        );
    }

    #[test]
    fn check_alarms_interrupting_control_idle_queues_xeq_event() {
        // Phase 63 (v4.3) — replaces the old "alarm:interrupting:deferred" stub.
        //
        // When is_running == false (idle path, D-13), an interrupting alarm must
        // route to event_buffer as "alarm:xeq:{label}" — same as a non-interrupting
        // alarm. The run_loop injection path (pending_interrupt) requires is_running.
        let mut state = CalcState::new();
        // is_running is false by default (idle path)
        state.alarms.push(AlarmEntry {
            trigger_unix: 500,
            repeat_secs: 0,
            alarm_type: AlarmType::Control {
                label: "IPROG".to_string(),
                interrupting: true,
            },
            past_due: false,
        });
        check_alarms(&mut state);
        assert!(state.alarms[0].past_due);
        // Idle path → alarm:xeq:{label} to event_buffer (D-13)
        assert!(
            state.event_buffer.contains(&"alarm:xeq:IPROG".to_string()),
            "idle interrupting alarm must push alarm:xeq:IPROG; event_buffer: {:?}",
            state.event_buffer
        );
        // pending_interrupt must NOT be set (is_running was false)
        assert!(
            state.pending_interrupt.is_none(),
            "pending_interrupt must stay None when is_running == false"
        );
        // No print_buffer entry for control alarms
        assert!(
            state.print_buffer.is_empty(),
            "print_buffer should be empty for interrupting control"
        );
    }

    #[test]
    fn check_alarms_interrupting_control_running_sets_pending_interrupt() {
        // Phase 63 (v4.3) — D-12 run-loop injection path.
        //
        // When is_running == true and no interrupt is pending and no solver/modal
        // is active, an interrupting alarm sets pending_interrupt = Some(label)
        // and pending_interrupt_alarm_index = Some(idx) for run_loop to consume.
        let mut state = CalcState::new();
        state.is_running = true; // simulate running program
        state.alarms.push(AlarmEntry {
            trigger_unix: 500,
            repeat_secs: 0,
            alarm_type: AlarmType::Control {
                label: "HANDLER".to_string(),
                interrupting: true,
            },
            past_due: false,
        });
        check_alarms(&mut state);
        assert!(state.alarms[0].past_due);
        // Running path → pending_interrupt = Some(label); NOT event_buffer
        assert_eq!(
            state.pending_interrupt.as_deref(),
            Some("HANDLER"),
            "pending_interrupt must be Some(HANDLER) when running"
        );
        assert_eq!(
            state.pending_interrupt_alarm_index,
            Some(0),
            "pending_interrupt_alarm_index must be Some(0)"
        );
        // event_buffer must NOT have the old deferred stub or any alarm event
        assert!(
            !state.event_buffer.iter().any(|e| e.starts_with("alarm:")),
            "event_buffer must be empty for run-loop injection path; got: {:?}",
            state.event_buffer
        );
        state.is_running = false; // cleanup
    }

    #[test]
    fn check_alarms_future_alarm_not_marked() {
        let mut state = CalcState::new();
        // Alarm far in the future.
        state.alarms.push(AlarmEntry {
            trigger_unix: i64::MAX,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("future".to_string()),
            past_due: false,
        });
        check_alarms(&mut state);
        assert!(
            !state.alarms[0].past_due,
            "Future alarm should not be past-due"
        );
        assert!(state.event_buffer.is_empty());
    }

    #[test]
    fn check_alarms_already_past_due_not_re_fired() {
        let mut state = CalcState::new();
        // Already-past-due alarm should not fire again.
        state.alarms.push(AlarmEntry {
            trigger_unix: 1000,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("once".to_string()),
            past_due: true, // already acknowledged
        });
        check_alarms(&mut state);
        assert!(
            state.event_buffer.is_empty(),
            "Already past-due alarm should not re-fire"
        );
    }

    // ── acknowledge_alarm ─────────────────────────────────────────────────────

    #[test]
    fn acknowledge_alarm_repeating_reschedules() {
        let mut state = CalcState::new();
        state.alarms.push(AlarmEntry {
            trigger_unix: 1000,
            repeat_secs: 3600,
            alarm_type: AlarmType::Message("hourly".to_string()),
            past_due: true,
        });
        acknowledge_alarm(&mut state, 0).unwrap();
        assert_eq!(
            state.alarms.len(),
            1,
            "Repeating alarm should stay in catalog"
        );
        assert_eq!(state.alarms[0].trigger_unix, 1000 + 3600);
        assert!(
            !state.alarms[0].past_due,
            "past_due should be reset after reschedule"
        );
    }

    #[test]
    fn acknowledge_alarm_non_repeating_removes() {
        let mut state = CalcState::new();
        state.alarms.push(AlarmEntry {
            trigger_unix: 1000,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("once".to_string()),
            past_due: true,
        });
        acknowledge_alarm(&mut state, 0).unwrap();
        assert!(
            state.alarms.is_empty(),
            "One-shot alarm should be removed after acknowledgment"
        );
    }

    #[test]
    fn acknowledge_alarm_out_of_bounds_returns_err() {
        let mut state = CalcState::new();
        assert!(acknowledge_alarm(&mut state, 5).is_err());
        assert!(state.alarms.is_empty());
    }

    // ── op_almnow ─────────────────────────────────────────────────────────────

    #[test]
    fn op_almnow_triggers_past_due_message_alarm() {
        let mut state = CalcState::new();
        state.alarms.push(AlarmEntry {
            trigger_unix: 1000,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("urgent".to_string()),
            past_due: true,
        });
        op_almnow(&mut state).unwrap();
        assert!(state.alarms.is_empty(), "One-shot alarm should be removed");
        assert!(state
            .event_buffer
            .contains(&"alarm:message:urgent".to_string()));
    }

    #[test]
    fn op_almnow_repeating_alarm_reschedules() {
        let mut state = CalcState::new();
        state.alarms.push(AlarmEntry {
            trigger_unix: 1000,
            repeat_secs: 900,
            alarm_type: AlarmType::Message("repeat".to_string()),
            past_due: true,
        });
        op_almnow(&mut state).unwrap();
        assert_eq!(state.alarms.len(), 1);
        assert_eq!(state.alarms[0].trigger_unix, 1000 + 900);
        assert!(!state.alarms[0].past_due);
    }

    #[test]
    fn op_almnow_no_alarms_is_ok() {
        let mut state = CalcState::new();
        assert!(op_almnow(&mut state).is_ok());
    }

    // ── Previously existing tests preserved ──────────────────────────────────

    #[test]
    fn op_clalma_clears_alarms_legacy() {
        // NOTE: op_clalma now clears by ALPHA match, not all alarms.
        // This test is replaced by specific clalma tests above.
        // op_clralms is the "clear all" function.
        let mut state = CalcState::new();
        state.alarms.push(AlarmEntry {
            trigger_unix: 1000,
            repeat_secs: 0,
            alarm_type: AlarmType::Message("test".to_string()),
            past_due: false,
        });
        assert!(!state.alarms.is_empty());
        // Use op_clralms (clear all) for this test.
        op_clralms(&mut state).unwrap();
        assert!(state.alarms.is_empty());
    }
}
