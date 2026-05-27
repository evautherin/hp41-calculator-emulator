//! Phase 44 Plan 02 Task 2 — Advantage Pac modal-prompt routing tests (ADV-CLI-07).
//!
//! Asserts that every AdvantageStep variant surfaces its prompt via
//! `ModalProgram::Advantage(step).current_prompt()` AND that alpha-label
//! steps return `requires_alpha_label() == true` while numeric/toggle steps return false.
//!
//! Verification strategy: cross-checks at the CARRIER enum level
//! (`ModalProgram::Advantage(_)`), NOT at the inner `advantage::modal::current_prompt`
//! level directly. This proves the Phase 43 D-43.6 carrier-enum dispatch routes
//! Advantage prompts correctly through the same `ModalProgram` enum the CLI / GUI /
//! `ui::pending_prompt` consume.
//!
//! Prompt strings verified against `hp41-core/src/ops/advantage/modal.rs`:
//! - TvmN          -> "N=?",     requires_alpha = false
//! - TvmI          -> "I%YR=?",  requires_alpha = false
//! - TvmPv         -> "PV=?",    requires_alpha = false
//! - TvmPmt        -> "PMT=?",   requires_alpha = false
//! - TvmFv         -> "FV=?",    requires_alpha = false
//! - TvmBeginEnd   -> None,      requires_alpha = false
//! - MatrixNamePrompt  -> "MNAME?",   requires_alpha = true
//! - MatrixDimRowPrompt -> "ROWS=?",  requires_alpha = false
//! - MatrixDimColPrompt -> "COLS=?",  requires_alpha = false
//! - MatrxOperationChoice -> "MATRX OP?", requires_alpha = false
//! - MtrNamePrompt -> "MTR NAME?",    requires_alpha = true
//! - FdifeqOrderPrompt -> "ORDER=?",  requires_alpha = false
//! - FdifeqFunctionNamePrompt -> "F NAME?", requires_alpha = true
//! - MeditElementPrompt(r, c) -> "[r,c]=?",  requires_alpha = false
//! - CmeditElementPrompt(r, c) -> "C[r,c]=?", requires_alpha = false
//! - VeComponentPrompt(k) -> "V[k]=?",    requires_alpha = false

#![allow(clippy::unwrap_used)]

use hp41_core::ops::advantage::modal::AdvantageStep;
use hp41_core::ops::math1::modal::ModalProgram;

// ── TVM steps ────────────────────────────────────────────────────────────────

#[test]
fn advantage_tvm_n_prompt() {
    let prog = ModalProgram::Advantage(AdvantageStep::TvmN);
    assert_eq!(
        prog.current_prompt(),
        Some("N=?".to_string()),
        "ModalProgram::Advantage(TvmN).current_prompt() must return 'N=?' \
         (per advantage/modal.rs::current_prompt TvmN arm)"
    );
    assert!(
        !prog.requires_alpha_label(),
        "TvmN must NOT require alpha label — numeric N entry only"
    );
}

#[test]
fn advantage_tvm_i_prompt() {
    let prog = ModalProgram::Advantage(AdvantageStep::TvmI);
    assert_eq!(
        prog.current_prompt(),
        Some("I%YR=?".to_string()),
        "ModalProgram::Advantage(TvmI).current_prompt() must return 'I%YR=?' \
         (per advantage/modal.rs::current_prompt TvmI arm)"
    );
    assert!(
        !prog.requires_alpha_label(),
        "TvmI must NOT require alpha label — numeric I%YR entry only"
    );
}

#[test]
fn advantage_tvm_pv_prompt() {
    let prog = ModalProgram::Advantage(AdvantageStep::TvmPv);
    assert_eq!(
        prog.current_prompt(),
        Some("PV=?".to_string()),
        "ModalProgram::Advantage(TvmPv).current_prompt() must return 'PV=?' \
         (per advantage/modal.rs::current_prompt TvmPv arm)"
    );
    assert!(
        !prog.requires_alpha_label(),
        "TvmPv must NOT require alpha label — numeric PV entry only"
    );
}

#[test]
fn advantage_tvm_pmt_prompt() {
    let prog = ModalProgram::Advantage(AdvantageStep::TvmPmt);
    assert_eq!(
        prog.current_prompt(),
        Some("PMT=?".to_string()),
        "ModalProgram::Advantage(TvmPmt).current_prompt() must return 'PMT=?' \
         (per advantage/modal.rs::current_prompt TvmPmt arm)"
    );
    assert!(
        !prog.requires_alpha_label(),
        "TvmPmt must NOT require alpha label — numeric PMT entry only"
    );
}

#[test]
fn advantage_tvm_fv_prompt() {
    let prog = ModalProgram::Advantage(AdvantageStep::TvmFv);
    assert_eq!(
        prog.current_prompt(),
        Some("FV=?".to_string()),
        "ModalProgram::Advantage(TvmFv).current_prompt() must return 'FV=?' \
         (per advantage/modal.rs::current_prompt TvmFv arm)"
    );
    assert!(
        !prog.requires_alpha_label(),
        "TvmFv must NOT require alpha label — numeric FV entry only"
    );
}

#[test]
fn advantage_tvm_begin_end_no_prompt() {
    let prog = ModalProgram::Advantage(AdvantageStep::TvmBeginEnd);
    assert_eq!(
        prog.current_prompt(),
        None,
        "ModalProgram::Advantage(TvmBeginEnd).current_prompt() must return None \
         (TvmBeginEnd is a toggle step with no text prompt — per advantage/modal.rs)"
    );
    assert!(
        !prog.requires_alpha_label(),
        "TvmBeginEnd must NOT require alpha label — toggle step only"
    );
}

// ── Alpha-label steps (requires_alpha_label == true) ─────────────────────────

#[test]
fn advantage_matrix_name_prompt_requires_alpha_label() {
    let prog = ModalProgram::Advantage(AdvantageStep::MatrixNamePrompt);
    assert_eq!(
        prog.current_prompt(),
        Some("MNAME?".to_string()),
        "ModalProgram::Advantage(MatrixNamePrompt).current_prompt() must return 'MNAME?' \
         (per advantage/modal.rs::current_prompt MatrixNamePrompt arm)"
    );
    assert!(
        prog.requires_alpha_label(),
        "MatrixNamePrompt MUST require alpha label — matrix name entered via ALPHA register"
    );
}

#[test]
fn advantage_mtr_name_prompt_requires_alpha_label() {
    let prog = ModalProgram::Advantage(AdvantageStep::MtrNamePrompt);
    assert_eq!(
        prog.current_prompt(),
        Some("MTR NAME?".to_string()),
        "ModalProgram::Advantage(MtrNamePrompt).current_prompt() must return 'MTR NAME?' \
         (per advantage/modal.rs::current_prompt MtrNamePrompt arm)"
    );
    assert!(
        prog.requires_alpha_label(),
        "MtrNamePrompt MUST require alpha label — target matrix name entered via ALPHA register"
    );
}

#[test]
fn advantage_fdifeq_function_name_prompt_requires_alpha_label() {
    let prog = ModalProgram::Advantage(AdvantageStep::FdifeqFunctionNamePrompt);
    assert_eq!(
        prog.current_prompt(),
        Some("F NAME?".to_string()),
        "ModalProgram::Advantage(FdifeqFunctionNamePrompt).current_prompt() must return 'F NAME?' \
         (per advantage/modal.rs::current_prompt FdifeqFunctionNamePrompt arm)"
    );
    assert!(
        prog.requires_alpha_label(),
        "FdifeqFunctionNamePrompt MUST require alpha label — function label entered via ALPHA register"
    );
}

// ── Non-parameterized numeric steps ──────────────────────────────────────────

#[test]
fn advantage_matrix_dim_row_prompt() {
    let prog = ModalProgram::Advantage(AdvantageStep::MatrixDimRowPrompt);
    assert_eq!(
        prog.current_prompt(),
        Some("ROWS=?".to_string()),
        "ModalProgram::Advantage(MatrixDimRowPrompt).current_prompt() must return 'ROWS=?' \
         (per advantage/modal.rs::current_prompt MatrixDimRowPrompt arm)"
    );
    assert!(
        !prog.requires_alpha_label(),
        "MatrixDimRowPrompt must NOT require alpha label — numeric row count entry"
    );
}

#[test]
fn advantage_matrix_dim_col_prompt() {
    let prog = ModalProgram::Advantage(AdvantageStep::MatrixDimColPrompt);
    assert_eq!(
        prog.current_prompt(),
        Some("COLS=?".to_string()),
        "ModalProgram::Advantage(MatrixDimColPrompt).current_prompt() must return 'COLS=?' \
         (per advantage/modal.rs::current_prompt MatrixDimColPrompt arm)"
    );
    assert!(
        !prog.requires_alpha_label(),
        "MatrixDimColPrompt must NOT require alpha label — numeric column count entry"
    );
}

#[test]
fn advantage_matrx_operation_choice_prompt() {
    let prog = ModalProgram::Advantage(AdvantageStep::MatrxOperationChoice);
    assert_eq!(
        prog.current_prompt(),
        Some("MATRX OP?".to_string()),
        "ModalProgram::Advantage(MatrxOperationChoice).current_prompt() must return 'MATRX OP?' \
         (per advantage/modal.rs::current_prompt MatrxOperationChoice arm)"
    );
    assert!(
        !prog.requires_alpha_label(),
        "MatrxOperationChoice must NOT require alpha label — numeric operation code entry"
    );
}

#[test]
fn advantage_fdifeq_order_prompt() {
    let prog = ModalProgram::Advantage(AdvantageStep::FdifeqOrderPrompt);
    assert_eq!(
        prog.current_prompt(),
        Some("ORDER=?".to_string()),
        "ModalProgram::Advantage(FdifeqOrderPrompt).current_prompt() must return 'ORDER=?' \
         (per advantage/modal.rs::current_prompt FdifeqOrderPrompt arm)"
    );
    assert!(
        !prog.requires_alpha_label(),
        "FdifeqOrderPrompt must NOT require alpha label — numeric ODE order entry"
    );
}

// ── Parameterized steps (verify prompt contains coordinates) ─────────────────

#[test]
fn advantage_medit_element_prompt_format() {
    // MeditElementPrompt(r, c) -> "[r,c]=?" — verify format contains row and col.
    let prog = ModalProgram::Advantage(AdvantageStep::MeditElementPrompt(3, 7));
    let prompt = prog.current_prompt().unwrap_or_else(|| {
        panic!("MeditElementPrompt(3, 7).current_prompt() must return Some(...)")
    });
    assert!(
        prompt.contains('3'),
        "MeditElementPrompt(3, 7) prompt '{prompt}' must contain row index '3'"
    );
    assert!(
        prompt.contains('7'),
        "MeditElementPrompt(3, 7) prompt '{prompt}' must contain col index '7'"
    );
    assert!(
        !prog.requires_alpha_label(),
        "MeditElementPrompt must NOT require alpha label — numeric element value entry"
    );
}

#[test]
fn advantage_cmedit_element_prompt_format() {
    // CmeditElementPrompt(r, c) -> "C[r,c]=?" — verify format contains row and col.
    let prog = ModalProgram::Advantage(AdvantageStep::CmeditElementPrompt(1, 2));
    let prompt = prog.current_prompt().unwrap_or_else(|| {
        panic!("CmeditElementPrompt(1, 2).current_prompt() must return Some(...)")
    });
    assert!(
        prompt.contains('1'),
        "CmeditElementPrompt(1, 2) prompt '{prompt}' must contain row index '1'"
    );
    assert!(
        prompt.contains('2'),
        "CmeditElementPrompt(1, 2) prompt '{prompt}' must contain col index '2'"
    );
    assert!(
        !prog.requires_alpha_label(),
        "CmeditElementPrompt must NOT require alpha label — complex element value entry (X=real, Y=imag)"
    );
}

#[test]
fn advantage_ve_component_prompt_format() {
    // VeComponentPrompt(k) -> "V[k]=?" — verify format contains component index.
    let prog = ModalProgram::Advantage(AdvantageStep::VeComponentPrompt(2));
    let prompt = prog
        .current_prompt()
        .unwrap_or_else(|| panic!("VeComponentPrompt(2).current_prompt() must return Some(...)"));
    assert!(
        prompt.contains('2'),
        "VeComponentPrompt(2) prompt '{prompt}' must contain component index '2'"
    );
    assert!(
        !prog.requires_alpha_label(),
        "VeComponentPrompt must NOT require alpha label — numeric vector component entry"
    );
}
