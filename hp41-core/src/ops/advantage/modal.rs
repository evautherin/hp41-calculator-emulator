// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `modal` — AdvantageStep enum and modal workflow state machine.
//!
//! Implements multi-key sequences for TVM, matrix naming/dimensioning/editing,
//! vector-entry, MATRX/MTR operation selection, and FDIFEQ configuration.
//! All `submit_step` arms are stubs (return `Err(HpError::InvalidOp)`) for v3.3
//! skeleton; Plans 43-06 and 43-08 implement the actual logic.

use crate::{
    error::HpError,
    state::CalcState,
};

/// Modal-workflow step discriminant for the Advantage Pac (ADV-FW-03 / D-43.6).
///
/// Each variant represents a single prompt in a multi-keystroke Advantage Pac
/// workflow. No `_ =>` catch-all is permitted in any exhaustive match over this
/// enum (4-way invariant extension per Phase 43 design).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AdvantageStep {
    /// TVM: enter N (number of periods).
    TvmN,
    /// TVM: enter I (periodic interest rate, %).
    TvmI,
    /// TVM: enter PV (present value).
    TvmPv,
    /// TVM: enter PMT (payment).
    TvmPmt,
    /// TVM: enter FV (future value).
    TvmFv,
    /// TVM: toggle BEGIN/END payment timing.
    TvmBeginEnd,
    /// MNAME/MATRX/MTR: enter matrix name from ALPHA register.
    MatrixNamePrompt,
    /// MATDIM row-entry: enter number of rows.
    MatrixDimRowPrompt,
    /// MATDIM col-entry: enter number of columns.
    MatrixDimColPrompt,
    /// MEDIT element entry: display/enter element at row `r`, col `c` (1-based).
    MeditElementPrompt(u8, u8),
    /// CMEDIT complex element entry: display/enter complex element at row `r`, col `c` (1-based).
    CmeditElementPrompt(u8, u8),
    /// VE: enter vector component `k` (1-based).
    VeComponentPrompt(u8),
    /// MATRX: choose matrix operation (modal menu choice).
    MatrxOperationChoice,
    /// MTR: enter target matrix name.
    MtrNamePrompt,
    /// FDIFEQ order entry: enter order of differential equation.
    FdifeqOrderPrompt,
    /// FDIFEQ function name entry: enter label/name of derivative function.
    FdifeqFunctionNamePrompt,
}

/// Return the display prompt string for the given `AdvantageStep`, or `None` if
/// the step requires no text prompt (e.g. a toggle or automatic action).
///
/// All matches are exhaustive; no `_ =>` catch-all.
pub fn current_prompt(step: &AdvantageStep) -> Option<String> {
    match step {
        AdvantageStep::TvmN => Some("N=?".to_string()),
        AdvantageStep::TvmI => Some("I%YR=?".to_string()),
        AdvantageStep::TvmPv => Some("PV=?".to_string()),
        AdvantageStep::TvmPmt => Some("PMT=?".to_string()),
        AdvantageStep::TvmFv => Some("FV=?".to_string()),
        AdvantageStep::TvmBeginEnd => None,
        AdvantageStep::MatrixNamePrompt => Some("MNAME?".to_string()),
        AdvantageStep::MatrixDimRowPrompt => Some("ROWS=?".to_string()),
        AdvantageStep::MatrixDimColPrompt => Some("COLS=?".to_string()),
        AdvantageStep::MeditElementPrompt(r, c) => {
            Some(format!("[{},{}]=?", r, c))
        }
        AdvantageStep::CmeditElementPrompt(r, c) => {
            Some(format!("C[{},{}]=?", r, c))
        }
        AdvantageStep::VeComponentPrompt(k) => {
            Some(format!("V[{}]=?", k))
        }
        AdvantageStep::MatrxOperationChoice => Some("MATRX OP?".to_string()),
        AdvantageStep::MtrNamePrompt => Some("MTR NAME?".to_string()),
        AdvantageStep::FdifeqOrderPrompt => Some("ORDER=?".to_string()),
        AdvantageStep::FdifeqFunctionNamePrompt => Some("F NAME?".to_string()),
    }
}

/// Returns `true` if this step expects the user to supply a value via the ALPHA
/// register (e.g. a matrix name), `false` for numeric entry or toggle steps.
///
/// All matches are exhaustive; no `_ =>` catch-all.
pub fn requires_alpha_label(step: &AdvantageStep) -> bool {
    match step {
        AdvantageStep::TvmN => false,
        AdvantageStep::TvmI => false,
        AdvantageStep::TvmPv => false,
        AdvantageStep::TvmPmt => false,
        AdvantageStep::TvmFv => false,
        AdvantageStep::TvmBeginEnd => false,
        AdvantageStep::MatrixNamePrompt => true,
        AdvantageStep::MatrixDimRowPrompt => false,
        AdvantageStep::MatrixDimColPrompt => false,
        AdvantageStep::MeditElementPrompt(_, _) => false,
        AdvantageStep::CmeditElementPrompt(_, _) => false,
        AdvantageStep::VeComponentPrompt(_) => false,
        AdvantageStep::MatrxOperationChoice => false,
        AdvantageStep::MtrNamePrompt => true,
        AdvantageStep::FdifeqOrderPrompt => false,
        AdvantageStep::FdifeqFunctionNamePrompt => true,
    }
}

/// Advance the modal workflow by one step given the current `AdvantageStep`.
///
/// Stub implementation: all arms return `Err(HpError::InvalidOp)`.
/// Plans 43-06 (TVM) and 43-08 (matrix/solver workflows) implement the full logic.
///
/// # Errors
/// Always returns `Err(HpError::InvalidOp)` in this skeleton phase.
pub fn submit_step(
    _state: &mut CalcState,
    step: AdvantageStep,
) -> Result<(), HpError> {
    match step {
        AdvantageStep::TvmN => Err(HpError::InvalidOp),
        AdvantageStep::TvmI => Err(HpError::InvalidOp),
        AdvantageStep::TvmPv => Err(HpError::InvalidOp),
        AdvantageStep::TvmPmt => Err(HpError::InvalidOp),
        AdvantageStep::TvmFv => Err(HpError::InvalidOp),
        AdvantageStep::TvmBeginEnd => Err(HpError::InvalidOp),
        AdvantageStep::MatrixNamePrompt => Err(HpError::InvalidOp),
        AdvantageStep::MatrixDimRowPrompt => Err(HpError::InvalidOp),
        AdvantageStep::MatrixDimColPrompt => Err(HpError::InvalidOp),
        AdvantageStep::MeditElementPrompt(_, _) => Err(HpError::InvalidOp),
        AdvantageStep::CmeditElementPrompt(_, _) => Err(HpError::InvalidOp),
        AdvantageStep::VeComponentPrompt(_) => Err(HpError::InvalidOp),
        AdvantageStep::MatrxOperationChoice => Err(HpError::InvalidOp),
        AdvantageStep::MtrNamePrompt => Err(HpError::InvalidOp),
        AdvantageStep::FdifeqOrderPrompt => Err(HpError::InvalidOp),
        AdvantageStep::FdifeqFunctionNamePrompt => Err(HpError::InvalidOp),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Catches: current_prompt returns Some for numeric-entry steps
    #[test]
    fn tvm_steps_have_prompts() {
        assert!(current_prompt(&AdvantageStep::TvmN).is_some());
        assert!(current_prompt(&AdvantageStep::TvmI).is_some());
        assert!(current_prompt(&AdvantageStep::TvmPv).is_some());
        assert!(current_prompt(&AdvantageStep::TvmPmt).is_some());
        assert!(current_prompt(&AdvantageStep::TvmFv).is_some());
    }

    // Catches: TvmBeginEnd is a toggle with no prompt
    #[test]
    fn tvm_begin_end_has_no_prompt() {
        assert!(current_prompt(&AdvantageStep::TvmBeginEnd).is_none());
    }

    // Catches: requires_alpha_label returns true for name-entry steps only
    #[test]
    fn alpha_label_steps() {
        assert!(requires_alpha_label(&AdvantageStep::MatrixNamePrompt));
        assert!(requires_alpha_label(&AdvantageStep::MtrNamePrompt));
        assert!(requires_alpha_label(&AdvantageStep::FdifeqFunctionNamePrompt));
        assert!(!requires_alpha_label(&AdvantageStep::TvmN));
        assert!(!requires_alpha_label(&AdvantageStep::MatrixDimRowPrompt));
    }

    // Catches: MeditElementPrompt prompt contains row/col in display
    #[test]
    fn medit_prompt_format() {
        let p = current_prompt(&AdvantageStep::MeditElementPrompt(3, 7)).unwrap();
        assert!(p.contains('3'));
        assert!(p.contains('7'));
    }

    // Catches: VeComponentPrompt prompt contains component index
    #[test]
    fn ve_prompt_format() {
        let p = current_prompt(&AdvantageStep::VeComponentPrompt(2)).unwrap();
        assert!(p.contains('2'));
    }
}
