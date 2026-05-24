//! Phase 34 / Plan 34-02 Task 4 — Stat 1 Pac modal-prompt routing tests (STAT-CLI-05).
//!
//! Asserts that every Stat1Step variant surfaces its OM-cited prompt via
//! `ModalProgram::Stat1(_).current_prompt()` AND that NONE of them require an
//! alpha label (so the post-dispatch CollectForModal auto-open hook does NOT
//! fire for Stat 1 — confirmed by CONTEXT §"In scope" item 7 + CONTEXT
//! §"Specifics" `requires_alpha_label` bullet).
//!
//! Verification strategy: cross-checks at the CARRIER enum level
//! (`ModalProgram::Stat1(_)`), NOT at the inner `stat1::modal::current_prompt`
//! level directly. This proves the Phase 33 / D-33.3b carrier-enum dispatch
//! routes Stat 1 prompts correctly through the same `ModalProgram` enum the
//! CLI / GUI / `ui::pending_prompt` consume. A regression in the dispatch arm
//! in `hp41-core/src/ops/math1/modal.rs` would surface here.
//!
//! Prompt strings per CONTEXT §"Specifics" (cross-referenced against
//! `hp41-core/src/ops/stat1/modal.rs:333-343`):
//! - NormdModeChoice → "ΣNORMD MODE?"
//! - ChisqdNuPrompt → "ν=?"
//! - ChisqdModeChoice → "ΣCHISQD MODE?"
//! - PolypDegreePrompt(_) → "DEGREE=?" (degree value is OPAQUE to prompt)
//! - SeedPrompt → "SEED?"

#![allow(clippy::unwrap_used)]

use hp41_core::ops::math1::modal::ModalProgram;
use hp41_core::ops::stat1::modal::Stat1Step;

#[test]
fn stat1_normd_mode_choice_prompt() {
    let prog = ModalProgram::Stat1(Stat1Step::NormdModeChoice);
    assert_eq!(
        prog.current_prompt(),
        Some("\u{03A3}NORMD MODE?".to_string()),
        "ModalProgram::Stat1(NormdModeChoice).current_prompt() must \
         return 'ΣNORMD MODE?' (Phase 33 OM-cited per stat1/modal.rs:335)"
    );
    assert!(
        !prog.requires_alpha_label(),
        "NormdModeChoice must NOT require alpha label — numeric mode choice only"
    );
}

#[test]
fn stat1_chisqd_nu_prompt() {
    let prog = ModalProgram::Stat1(Stat1Step::ChisqdNuPrompt);
    assert_eq!(
        prog.current_prompt(),
        Some("\u{03BD}=?".to_string()),
        "ModalProgram::Stat1(ChisqdNuPrompt).current_prompt() must \
         return 'ν=?' (Phase 33 OM-cited per stat1/modal.rs:336)"
    );
    assert!(
        !prog.requires_alpha_label(),
        "ChisqdNuPrompt must NOT require alpha label — numeric ν entry only"
    );
}

#[test]
fn stat1_chisqd_mode_choice_prompt() {
    let prog = ModalProgram::Stat1(Stat1Step::ChisqdModeChoice);
    assert_eq!(
        prog.current_prompt(),
        Some("\u{03A3}CHISQD MODE?".to_string()),
        "ModalProgram::Stat1(ChisqdModeChoice).current_prompt() must \
         return 'ΣCHISQD MODE?' (Phase 33 OM-cited per stat1/modal.rs:337)"
    );
    assert!(
        !prog.requires_alpha_label(),
        "ChisqdModeChoice must NOT require alpha label — numeric mode choice only"
    );
}

#[test]
fn stat1_polyp_degree_prompt() {
    // Inner u8 is opaque — the prompt must be identical for any degree.
    for degree in [0u8, 1, 3, 5, 99] {
        let prog = ModalProgram::Stat1(Stat1Step::PolypDegreePrompt(degree));
        assert_eq!(
            prog.current_prompt(),
            Some("DEGREE=?".to_string()),
            "ModalProgram::Stat1(PolypDegreePrompt({degree})).current_prompt() \
             must return 'DEGREE=?' regardless of the inner degree value \
             (CONTEXT §Specifics: inner u8 is OPAQUE to the rendered prompt; \
             D-34.3 divergence note)"
        );
        assert!(
            !prog.requires_alpha_label(),
            "PolypDegreePrompt({degree}) must NOT require alpha label — \
             numeric degree entry only"
        );
    }
}

#[test]
fn stat1_seed_prompt() {
    let prog = ModalProgram::Stat1(Stat1Step::SeedPrompt);
    assert_eq!(
        prog.current_prompt(),
        Some("SEED?".to_string()),
        "ModalProgram::Stat1(SeedPrompt).current_prompt() must \
         return 'SEED?' (Phase 33 OM-cited per stat1/modal.rs:342)"
    );
    assert!(
        !prog.requires_alpha_label(),
        "SeedPrompt must NOT require alpha label — numeric seed value only \
         (per CONTEXT §Already locked D-29.7/D-29.9: all 5 Stat1Step \
         variants are NUMERIC-input steps, so CollectForModal auto-open \
         never fires for Stat 1)"
    );
}
