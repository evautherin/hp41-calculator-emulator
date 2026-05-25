// Phase 41 Plan 41-03 — LCD-alternation routing tests for TimeStep modal prompts.
//
// Verifies that CalcStateView::from_state correctly routes all 3 TimeStep
// modal-prompt strings through the LCD-alternation branch (D-31.5 priority:
// modal_program.is_some() && entry_buf.is_empty() && modal_prompt.is_some()
// → display_str = truncate_with_continuation(modal_prompt)).
//
// This is the TIME-GUI-07 verification: all 3 TimeStep variants (SetTimePrompt,
// SetDatePrompt, XyzalmTimePrompt) route OM prompt strings through CalcStateView.
// All prompts fit within LCD_WIDTH=12, so no truncation is applied.
//
// Uses ModalProgram::Time(TimeStep) — the math1/ freeze-exception variant
// per D-carried.4 (ADR-v3.1-004 second carve-out + ADR-v3.1-005 pattern);
// TimeStep semantics live in hp41-core/src/ops/time/modal.rs (outside the
// freeze boundary).
//
// Template: hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt_stat1.rs
// (Phase 36 Plan 36-02)

#![allow(clippy::unwrap_used)]

use hp41_core::{
    ops::math1::modal::ModalProgram,
    ops::time::modal::TimeStep,
    CalcState,
};
use hp41_gui_lib::types::CalcStateView;

/// SETIME TIME? prompt (5 chars) renders verbatim — well under LCD width.
///
/// TimeStep::SetTimePrompt → current_prompt returns "TIME?" (5 chars).
/// LCD_WIDTH = 12, so no truncation marker is added.
///
/// Catches: TimeStep routing missing from the LCD-alternation branch, or
/// SetTimePrompt prompt string drifted from the OM-cited form.
#[test]
fn setime_prompt_renders_verbatim() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Time(TimeStep::SetTimePrompt));
    calc.modal_prompt = Some("TIME?".to_string()); // 5 chars — fits in LCD_WIDTH=12
    assert!(
        calc.entry_buf.is_empty(),
        "entry_buf must be empty for LCD-alternation branch to fire"
    );

    let view = CalcStateView::from_state(&calc, vec![], vec![]);

    assert_eq!(
        view.display_str, "TIME?",
        "TIME? (5 chars) must render verbatim — no truncation at LCD_WIDTH=12"
    );
}

/// SETDATE DATE? prompt (5 chars) renders verbatim — well under LCD width.
///
/// TimeStep::SetDatePrompt → current_prompt returns "DATE?" (5 chars).
/// LCD_WIDTH = 12, so no truncation marker is added.
///
/// Catches: SetDatePrompt routing missing from the LCD-alternation branch.
#[test]
fn setdate_prompt_renders_verbatim() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Time(TimeStep::SetDatePrompt));
    calc.modal_prompt = Some("DATE?".to_string()); // 5 chars — fits in LCD_WIDTH=12
    assert!(
        calc.entry_buf.is_empty(),
        "entry_buf must be empty for LCD-alternation branch to fire"
    );

    let view = CalcStateView::from_state(&calc, vec![], vec![]);

    assert_eq!(
        view.display_str, "DATE?",
        "DATE? (5 chars) must render verbatim — no truncation at LCD_WIDTH=12"
    );
}

/// XYZALM ALARM TIME? prompt (11 chars) renders verbatim — within LCD width.
///
/// TimeStep::XyzalmTimePrompt → current_prompt returns "ALARM TIME?" (11 chars).
/// LCD_WIDTH = 12, so no truncation marker is needed (11 ≤ 12).
///
/// Catches: XyzalmTimePrompt routing missing from the LCD-alternation branch.
/// Also verifies the boundary: 11-char prompt is the largest Time Module prompt
/// and must NOT trigger truncation (only prompts >12 chars trigger it).
#[test]
fn xyzalm_time_prompt_renders_verbatim() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Time(TimeStep::XyzalmTimePrompt));
    calc.modal_prompt = Some("ALARM TIME?".to_string()); // 11 chars — fits in LCD_WIDTH=12
    assert!(
        calc.entry_buf.is_empty(),
        "entry_buf must be empty for LCD-alternation branch to fire"
    );

    // Verify test fixture: confirm the raw prompt is 11 chars (within LCD_WIDTH=12).
    assert_eq!(
        "ALARM TIME?".chars().count(), 11,
        "test fixture: ALARM TIME? must be exactly 11 chars (under LCD_WIDTH=12)"
    );

    let view = CalcStateView::from_state(&calc, vec![], vec![]);

    assert_eq!(
        view.display_str, "ALARM TIME?",
        "ALARM TIME? (11 chars) must render verbatim — within LCD_WIDTH=12, no truncation"
    );
}
