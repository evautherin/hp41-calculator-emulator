// Phase 36 Plan 36-02 — LCD-alternation routing tests for Stat1Step modal prompts.
//
// Verifies that CalcStateView::from_state correctly routes all 5 Stat1Step
// modal-prompt strings through the LCD-alternation branch (D-31.5 priority:
// modal_program.is_some() && entry_buf.is_empty() && modal_prompt.is_some()
// → display_str = truncate_with_continuation(modal_prompt)).
//
// This is the STAT-GUI-04 verification: all 5 Stat1Step variants (NormdModeChoice,
// ChisqdNuPrompt, ChisqdModeChoice, PolypDegreePrompt, SeedPrompt) route OM
// strings through CalcStateView, including the ΣCHISQD MODE? truncation case.
//
// Uses ModalProgram::Stat1(Stat1Step) — the math1/ freeze-exception variant
// per D-33.3b (ADR-v3.1-004 + ADR-v3.1-005); `Stat1Step` semantics live in
// hp41-core/src/ops/stat1/modal.rs (outside the freeze boundary).
//
// Template: hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt.rs

#![allow(clippy::unwrap_used)]

use hp41_core::{
    ops::math1::modal::ModalProgram,
    ops::stat1::modal::Stat1Step,
    CalcState,
};
use hp41_gui_lib::types::CalcStateView;

/// ΣNORMD MODE? prompt (12 chars) renders verbatim — no truncation.
///
/// Stat1Step::NormdModeChoice → current_prompt returns "\u{03A3}NORMD MODE?" (12 chars).
/// LCD_WIDTH = 12, so no truncation marker is added.
///
/// Catches: Stat1Step routing missing from the LCD-alternation branch, or
/// NormdModeChoice prompt string drifted from the OM-cited 12-char form.
#[test]
fn normd_mode_choice_prompt_renders_verbatim() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Stat1(Stat1Step::NormdModeChoice));
    calc.modal_prompt = Some("\u{03A3}NORMD MODE?".to_string()); // 12 chars, no truncation
    assert!(calc.entry_buf.is_empty(), "entry_buf must be empty for LCD-alternation branch to fire");

    let view = CalcStateView::from_state(&calc, vec![], vec![]);

    assert_eq!(
        view.display_str, "\u{03A3}NORMD MODE?",
        "ΣNORMD MODE? (12 chars) must render verbatim — boundary: no truncation"
    );
}

/// ν=? prompt (3 chars) renders verbatim — well under LCD width.
///
/// Stat1Step::ChisqdNuPrompt → current_prompt returns "\u{03BD}=?" (3 chars).
///
/// Catches: Stat1Step routing missing from the LCD-alternation branch.
#[test]
fn chisqd_nu_prompt_renders_verbatim() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Stat1(Stat1Step::ChisqdNuPrompt));
    calc.modal_prompt = Some("\u{03BD}=?".to_string()); // 3 chars — fits comfortably in 12
    assert!(calc.entry_buf.is_empty(), "entry_buf must be empty for LCD-alternation branch to fire");

    let view = CalcStateView::from_state(&calc, vec![], vec![]);

    assert_eq!(
        view.display_str, "\u{03BD}=?",
        "ν=? (3 chars) must render verbatim (no truncation)"
    );
}

/// ΣCHISQD MODE? prompt (13 chars) TRUNCATES to first 11 chars + ≡ continuation marker.
///
/// Stat1Step::ChisqdModeChoice → current_prompt returns "\u{03A3}CHISQD MODE?" (13 chars).
/// LCD_WIDTH = 12 → truncate to take(11) + "\u{2261}" (≡, U+2261 continuation marker).
/// Expected: "\u{03A3}CHISQD MOD\u{2261}" (12 chars total).
///
/// This is Pitfall 1 (LCD truncation): the raw prompt is 13 chars; CalcStateView must
/// apply truncate_with_continuation before placing it in display_str.
///
/// Catches: truncation not applied to Stat1Step prompts, wrong continuation char,
/// off-by-one in LCD_WIDTH (12) boundary, wrong truncation point.
#[test]
fn chisqd_mode_choice_prompt_truncates() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Stat1(Stat1Step::ChisqdModeChoice));
    calc.modal_prompt = Some("\u{03A3}CHISQD MODE?".to_string()); // 13 chars — EXCEEDS LCD_WIDTH
    assert!(calc.entry_buf.is_empty(), "entry_buf must be empty for LCD-alternation branch to fire");

    // Verify test fixture: confirm the raw prompt is 13 chars.
    assert_eq!(
        "\u{03A3}CHISQD MODE?".chars().count(), 13,
        "test fixture: ΣCHISQD MODE? must be exactly 13 chars"
    );

    let view = CalcStateView::from_state(&calc, vec![], vec![]);

    // Truncated: first 11 chars (Σ=1, C=2, H=3, I=4, S=5, Q=6, D=7, ' '=8, M=9, O=10, D=11)
    // + ≡ continuation marker (U+2261) = 12 chars total.
    assert_eq!(
        view.display_str, "\u{03A3}CHISQD MOD\u{2261}",
        "ΣCHISQD MODE? (13 chars) must truncate to first 11 chars + ≡ continuation marker"
    );
    assert_eq!(
        view.display_str.chars().count(), 12,
        "truncated display_str must be exactly 12 chars (LCD_WIDTH)"
    );
}

/// DEGREE=? prompt (8 chars) renders verbatim — well under LCD width.
///
/// Stat1Step::PolypDegreePrompt(0) → current_prompt returns "DEGREE=?" (8 chars).
///
/// Catches: PolypDegreePrompt routing missing from LCD-alternation branch.
#[test]
fn polyp_degree_prompt_renders_verbatim() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Stat1(Stat1Step::PolypDegreePrompt(0)));
    calc.modal_prompt = Some("DEGREE=?".to_string()); // 8 chars — fits in 12
    assert!(calc.entry_buf.is_empty(), "entry_buf must be empty for LCD-alternation branch to fire");

    let view = CalcStateView::from_state(&calc, vec![], vec![]);

    assert_eq!(
        view.display_str, "DEGREE=?",
        "DEGREE=? (8 chars) must render verbatim (no truncation)"
    );
}

/// SEED? prompt (5 chars) renders verbatim — well under LCD width.
///
/// Stat1Step::SeedPrompt → current_prompt returns "SEED?" (5 chars).
///
/// Catches: SeedPrompt routing missing from LCD-alternation branch.
#[test]
fn seed_prompt_renders_verbatim() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Stat1(Stat1Step::SeedPrompt));
    calc.modal_prompt = Some("SEED?".to_string()); // 5 chars — fits in 12
    assert!(calc.entry_buf.is_empty(), "entry_buf must be empty for LCD-alternation branch to fire");

    let view = CalcStateView::from_state(&calc, vec![], vec![]);

    assert_eq!(
        view.display_str, "SEED?",
        "SEED? (5 chars) must render verbatim (no truncation)"
    );
}
