// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Alarm past-due detection latency tests for TIME-QUAL-11.
//!
//! Verifies that `check_alarms` fires a past-due alarm within one dispatch cycle
//! (i.e., within a single call to `check_alarms` for an alarm with trigger_unix
//! already in the past), and that future alarms do NOT fire prematurely.
//!
//! Also verifies ALMNOW (Op::TimeAlmnow) immediately triggers the oldest
//! past-due alarm's event within one dispatch.
//!
//! Oracle: `trigger_unix = 1000` (Jan 1 1970 00:16:40) is far in the past;
//! `trigger_unix = i64::MAX` is far in the future. No sleeps needed.
#![allow(clippy::unwrap_used)]

use hp41_core::ops::time::alarm::{check_alarms, AlarmEntry, AlarmType};
use hp41_core::ops::{dispatch, Op};
use hp41_core::state::CalcState;

// ── Helper: create a past-due alarm entry ────────────────────────────────────

fn past_due_message_alarm(msg: &str) -> AlarmEntry {
    AlarmEntry {
        trigger_unix: 1000, // Jan 1 1970 00:16:40 — always in the past
        repeat_secs: 0,
        alarm_type: AlarmType::Message(msg.to_string()),
        past_due: false,
    }
}

fn future_alarm(msg: &str) -> AlarmEntry {
    AlarmEntry {
        trigger_unix: i64::MAX, // Far future
        repeat_secs: 0,
        alarm_type: AlarmType::Message(msg.to_string()),
        past_due: false,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SECTION 1: Past-due alarm fires within one check_alarms call
// ═══════════════════════════════════════════════════════════════════════════

/// Past-due message alarm fires in one check_alarms call.
///
/// A message alarm with trigger_unix=1000 (far past) fires immediately when
/// check_alarms is called: event_buffer contains "alarm:message:{text}" and
/// past_due is set to true.
#[test]
fn alarm_latency_past_due_fires_in_one_cycle() {
    let mut state = CalcState::new();
    state.alarms.push(past_due_message_alarm("alert!"));

    // Single check_alarms call
    check_alarms(&mut state);

    assert!(
        state.alarms[0].past_due,
        "Alarm must be marked past_due after one check_alarms call"
    );
    assert!(
        state
            .event_buffer
            .contains(&"alarm:message:alert!".to_string()),
        "event_buffer must contain alarm:message:alert! after one cycle; got: {:?}",
        state.event_buffer
    );
}

/// Past-due message alarm pushes text to print_buffer within one check_alarms call.
#[test]
fn alarm_latency_message_event_populates_print_buffer() {
    let mut state = CalcState::new();
    state.alarms.push(past_due_message_alarm("morning alert"));

    check_alarms(&mut state);

    assert!(
        state.print_buffer.contains(&"morning alert".to_string()),
        "print_buffer must contain alarm message; got: {:?}",
        state.print_buffer
    );
}

/// Past-due non-interrupting control alarm fires in one check_alarms call.
/// event_buffer must contain "alarm:xeq:{label}".
#[test]
fn alarm_latency_control_alarm_fires_in_one_cycle() {
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

    assert!(
        state.alarms[0].past_due,
        "Control alarm must be marked past_due after one cycle"
    );
    assert!(
        state.event_buffer.contains(&"alarm:xeq:MYPROG".to_string()),
        "event_buffer must contain alarm:xeq:MYPROG; got: {:?}",
        state.event_buffer
    );
}

/// Past-due interrupting control alarm fires in one check_alarms call.
/// event_buffer must contain "alarm:interrupting:deferred" (D-38.4).
#[test]
fn alarm_latency_interrupting_control_fires_in_one_cycle() {
    let mut state = CalcState::new();
    state.alarms.push(AlarmEntry {
        trigger_unix: 100,
        repeat_secs: 0,
        alarm_type: AlarmType::Control {
            label: "IPROG".to_string(),
            interrupting: true,
        },
        past_due: false,
    });

    check_alarms(&mut state);

    assert!(
        state.alarms[0].past_due,
        "Interrupting control alarm must be marked past_due after one cycle"
    );
    assert!(
        state
            .event_buffer
            .contains(&"alarm:interrupting:deferred".to_string()),
        "event_buffer must contain alarm:interrupting:deferred; got: {:?}",
        state.event_buffer
    );
}

/// Multiple past-due alarms: all fire within one check_alarms call.
#[test]
fn alarm_latency_multiple_past_due_all_fire_in_one_cycle() {
    let mut state = CalcState::new();
    state.alarms.push(past_due_message_alarm("first"));
    state.alarms.push(past_due_message_alarm("second"));
    state.alarms.push(past_due_message_alarm("third"));

    check_alarms(&mut state);

    // All three must be marked past_due
    for (i, alarm) in state.alarms.iter().enumerate() {
        assert!(alarm.past_due, "Alarm {} must be past_due", i);
    }
    // All three events in event_buffer
    assert!(state
        .event_buffer
        .contains(&"alarm:message:first".to_string()));
    assert!(state
        .event_buffer
        .contains(&"alarm:message:second".to_string()));
    assert!(state
        .event_buffer
        .contains(&"alarm:message:third".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════
// SECTION 2: Future alarm does NOT fire when check_alarms is called
// ═══════════════════════════════════════════════════════════════════════════

/// Future alarm (trigger_unix = i64::MAX) does NOT fire on check_alarms.
#[test]
fn alarm_latency_future_alarm_does_not_fire() {
    let mut state = CalcState::new();
    state.alarms.push(future_alarm("future event"));

    check_alarms(&mut state);

    assert!(
        !state.alarms[0].past_due,
        "Future alarm must NOT be marked past_due"
    );
    assert!(
        state.event_buffer.is_empty(),
        "event_buffer must be empty for future alarm; got: {:?}",
        state.event_buffer
    );
}

/// Mixed past-due and future alarms: only past-due fires.
#[test]
fn alarm_latency_mixed_only_past_due_fires() {
    let mut state = CalcState::new();
    state.alarms.push(past_due_message_alarm("past"));
    state.alarms.push(future_alarm("future"));

    check_alarms(&mut state);

    // Past alarm fired
    assert!(state.alarms[0].past_due, "Past alarm must be past_due");
    // Future alarm did not fire
    assert!(
        !state.alarms[1].past_due,
        "Future alarm must NOT be past_due"
    );
    // Only one event in buffer
    assert_eq!(
        state.event_buffer.len(),
        1,
        "Only one event expected; got: {:?}",
        state.event_buffer
    );
    assert!(state
        .event_buffer
        .contains(&"alarm:message:past".to_string()));
}

/// Empty catalog: check_alarms is a no-op (no panic, no events).
#[test]
fn alarm_latency_empty_catalog_no_op() {
    let mut state = CalcState::new();
    check_alarms(&mut state);
    assert!(state.event_buffer.is_empty());
    assert!(state.alarms.is_empty());
}

/// Already-past-due alarm does NOT re-fire on second check_alarms call.
/// (Idempotency guard — past_due=true prevents double-dispatch.)
#[test]
fn alarm_latency_already_past_due_does_not_refire() {
    let mut state = CalcState::new();
    state.alarms.push(AlarmEntry {
        trigger_unix: 1000,
        repeat_secs: 0,
        alarm_type: AlarmType::Message("once".to_string()),
        past_due: true, // already marked
    });

    check_alarms(&mut state);

    // Should NOT fire again — event_buffer stays empty
    assert!(
        state.event_buffer.is_empty(),
        "Already-past-due alarm must not re-fire; got: {:?}",
        state.event_buffer
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// SECTION 3: ALMNOW fires immediately (event_buffer populated after one dispatch)
// ═══════════════════════════════════════════════════════════════════════════

/// ALMNOW on a past-due message alarm populates event_buffer immediately.
#[test]
fn alarm_latency_almnow_fires_past_due_immediately() {
    let mut state = CalcState::new();
    state.alarms.push(AlarmEntry {
        trigger_unix: 1000,
        repeat_secs: 0,
        alarm_type: AlarmType::Message("urgent".to_string()),
        past_due: true,
    });

    dispatch(&mut state, Op::TimeAlmnow).unwrap();

    assert!(
        state
            .event_buffer
            .contains(&"alarm:message:urgent".to_string()),
        "ALMNOW must push event immediately; got: {:?}",
        state.event_buffer
    );
    // One-shot alarm must be removed after ALMNOW dispatch
    assert!(
        state.alarms.is_empty(),
        "One-shot alarm must be removed after ALMNOW"
    );
}

/// ALMNOW on a future alarm (non-past-due) triggers it immediately.
/// This is ALMNOW's "fire now" semantic: if no past-due alarm exists,
/// it triggers the first upcoming alarm regardless of schedule.
#[test]
fn alarm_latency_almnow_triggers_upcoming_alarm() {
    let mut state = CalcState::new();
    state.alarms.push(future_alarm("upcoming"));

    dispatch(&mut state, Op::TimeAlmnow).unwrap();

    // The upcoming alarm should have been triggered
    assert!(
        state
            .event_buffer
            .contains(&"alarm:message:upcoming".to_string()),
        "ALMNOW must trigger first upcoming alarm; got: {:?}",
        state.event_buffer
    );
}

/// ALMNOW with no alarms at all is a no-op (no panic).
#[test]
fn alarm_latency_almnow_empty_catalog_noop() {
    let mut state = CalcState::new();
    let result = dispatch(&mut state, Op::TimeAlmnow);
    assert!(result.is_ok(), "ALMNOW on empty catalog must not error");
    assert!(state.event_buffer.is_empty());
}
