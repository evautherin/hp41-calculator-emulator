// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `modal` — AdvantageStep enum and modal workflow state machine.
//!
//! Implements multi-key sequences for TVM, matrix naming/dimensioning/editing,
//! vector-entry, MATRX/MTR operation selection, and FDIFEQ configuration.
//! TVM modal steps (TvmN/TvmI/TvmPv/TvmPmt/TvmFv/TvmBeginEnd) are fully
//! implemented in this plan (43-09). All other arms remain stubs.

use crate::{
    error::HpError,
    ops::math1::modal::ModalProgram,
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
/// TVM steps (TvmN/TvmI/TvmPv/TvmPmt/TvmFv/TvmBeginEnd) are fully implemented:
/// each arm stores the current X register value into the appropriate TVM register
/// and advances to the next modal step (or closes the modal after TvmBeginEnd).
///
/// Non-TVM steps remain stubs (return `Err(HpError::InvalidOp)`); Plans 43-08
/// and later implement matrix/solver workflow steps.
///
/// # Errors
/// - TVM arms: forwarded from TVM register-store ops.
/// - Non-TVM arms: `Err(HpError::InvalidOp)` (stub).
pub fn submit_step(
    state: &mut CalcState,
    step: AdvantageStep,
) -> Result<(), HpError> {
    match step {
        // ── TVM register-entry workflow ────────────────────────────────────────
        // Each arm:
        //   1. Stores X into the TVM register via the canonical op function.
        //   2. Advances modal_program to the next step (or clears it at end).
        //   3. Updates modal_prompt for the next step (or clears at end).
        AdvantageStep::TvmN => {
            crate::ops::advantage::tvm::op_adv_tvm_n(state)?;
            state.modal_program = Some(ModalProgram::Advantage(AdvantageStep::TvmI));
            state.modal_prompt = Some("I%YR=?".to_string());
            Ok(())
        }
        AdvantageStep::TvmI => {
            // I%YR is stored as the annual rate (user enters percent; we store raw percent).
            // The *I solver stores periodic rate; this prompt stores annual rate directly.
            let x = state.stack.x.clone();
            {
                let tvm = if state.adv_tvm_state.is_none() {
                    state.adv_tvm_state = Some(crate::ops::advantage::tvm::TvmState::default());
                    state.adv_tvm_state.as_mut().expect("just initialized")
                } else {
                    state.adv_tvm_state.as_mut().expect("checked Some above")
                };
                tvm.i = x;
            }
            state.modal_program = Some(ModalProgram::Advantage(AdvantageStep::TvmPv));
            state.modal_prompt = Some("PV=?".to_string());
            Ok(())
        }
        AdvantageStep::TvmPv => {
            crate::ops::advantage::tvm::op_adv_tvm_pv(state)?;
            state.modal_program = Some(ModalProgram::Advantage(AdvantageStep::TvmPmt));
            state.modal_prompt = Some("PMT=?".to_string());
            Ok(())
        }
        AdvantageStep::TvmPmt => {
            crate::ops::advantage::tvm::op_adv_tvm_pmt(state)?;
            state.modal_program = Some(ModalProgram::Advantage(AdvantageStep::TvmFv));
            state.modal_prompt = Some("FV=?".to_string());
            Ok(())
        }
        AdvantageStep::TvmFv => {
            crate::ops::advantage::tvm::op_adv_tvm_fv(state)?;
            state.modal_program = Some(ModalProgram::Advantage(AdvantageStep::TvmBeginEnd));
            state.modal_prompt = Some("BEG/END?".to_string());
            Ok(())
        }
        AdvantageStep::TvmBeginEnd => {
            // Toggle BEGIN/END mode (any non-zero X = BEGIN, X=0 = END).
            let x_val = state.stack.x.inner();
            {
                let tvm = if state.adv_tvm_state.is_none() {
                    state.adv_tvm_state = Some(crate::ops::advantage::tvm::TvmState::default());
                    state.adv_tvm_state.as_mut().expect("just initialized")
                } else {
                    state.adv_tvm_state.as_mut().expect("checked Some above")
                };
                tvm.begin_mode = !x_val.is_zero();
            }
            // Workflow complete — clear modal state
            state.modal_program = None;
            state.modal_prompt = None;
            Ok(())
        }
        // ── Non-TVM stubs ──────────────────────────────────────────────────────
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
    use crate::state::CalcState;
    use crate::num::HpNum;
    use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
    use rust_decimal::Decimal;

    // Helper: push a value to X
    fn push_x(state: &mut CalcState, v: f64) {
        state.stack.x = HpNum::from(Decimal::from_f64(v).unwrap());
    }

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

    // Catches: TVM modal workflow advances N→I→PV→PMT→FV→BeginEnd→done
    #[test]
    fn tvm_modal_workflow_complete_cycle() {
        let mut state = CalcState::new();

        // Step 1: submit N=12
        push_x(&mut state, 12.0);
        submit_step(&mut state, AdvantageStep::TvmN).unwrap();
        assert_eq!(state.modal_prompt, Some("I%YR=?".to_string()));
        let tvm = state.adv_tvm_state.as_ref().unwrap();
        assert!((tvm.n.inner().to_f64().unwrap() - 12.0).abs() < 1e-9);

        // Step 2: submit I=6
        push_x(&mut state, 6.0);
        submit_step(&mut state, AdvantageStep::TvmI).unwrap();
        assert_eq!(state.modal_prompt, Some("PV=?".to_string()));
        let tvm = state.adv_tvm_state.as_ref().unwrap();
        assert!((tvm.i.inner().to_f64().unwrap() - 6.0).abs() < 1e-9);

        // Step 3: submit PV=1000
        push_x(&mut state, 1000.0);
        submit_step(&mut state, AdvantageStep::TvmPv).unwrap();
        assert_eq!(state.modal_prompt, Some("PMT=?".to_string()));

        // Step 4: submit PMT=-100
        push_x(&mut state, -100.0);
        submit_step(&mut state, AdvantageStep::TvmPmt).unwrap();
        assert_eq!(state.modal_prompt, Some("FV=?".to_string()));

        // Step 5: submit FV=0
        push_x(&mut state, 0.0);
        submit_step(&mut state, AdvantageStep::TvmFv).unwrap();
        assert_eq!(state.modal_prompt, Some("BEG/END?".to_string()));

        // Step 6: submit BeginEnd=0 (END mode)
        push_x(&mut state, 0.0);
        submit_step(&mut state, AdvantageStep::TvmBeginEnd).unwrap();
        assert!(state.modal_program.is_none(), "modal cleared after TvmBeginEnd");
        assert!(state.modal_prompt.is_none(), "prompt cleared after TvmBeginEnd");
        let tvm = state.adv_tvm_state.as_ref().unwrap();
        assert!(!tvm.begin_mode, "X=0 → END mode");
    }

    // Catches: TvmBeginEnd with non-zero X sets begin_mode=true
    #[test]
    fn tvm_begin_mode_toggled_by_nonzero_x() {
        let mut state = CalcState::new();
        push_x(&mut state, 1.0); // non-zero → BEGIN mode
        submit_step(&mut state, AdvantageStep::TvmBeginEnd).unwrap();
        let tvm = state.adv_tvm_state.as_ref().unwrap();
        assert!(tvm.begin_mode, "non-zero X → begin_mode=true");
    }

    // Catches: non-TVM stubs still return InvalidOp
    #[test]
    fn non_tvm_steps_still_stub() {
        let mut state = CalcState::new();
        assert!(matches!(
            submit_step(&mut state, AdvantageStep::MatrixNamePrompt),
            Err(HpError::InvalidOp)
        ));
        assert!(matches!(
            submit_step(&mut state, AdvantageStep::FdifeqFunctionNamePrompt),
            Err(HpError::InvalidOp)
        ));
    }
}
