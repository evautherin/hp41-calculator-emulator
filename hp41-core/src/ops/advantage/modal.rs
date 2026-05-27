// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `modal` — AdvantageStep enum and modal workflow state machine.
//!
//! Implements multi-key sequences for TVM, matrix naming/dimensioning/editing,
//! vector-entry, MATRX/MTR operation selection, and FDIFEQ configuration.
//! TVM modal steps (TvmN/TvmI/TvmPv/TvmPmt/TvmFv/TvmBeginEnd) are fully
//! implemented (Plan 43-09 pattern). Matrix workflow steps implemented in Plan 43-08.

use crate::{error::HpError, num::HpNum, ops::math1::modal::ModalProgram, state::CalcState};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

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
    /// On submit, reads X = real part and Y = imaginary part (HP-41 complex stack convention).
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
        AdvantageStep::MeditElementPrompt(r, c) => Some(format!("[{},{}]=?", r, c)),
        AdvantageStep::CmeditElementPrompt(r, c) => Some(format!("C[{},{}]=?", r, c)),
        AdvantageStep::VeComponentPrompt(k) => Some(format!("V[{}]=?", k)),
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

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Truncate X-register value to u8; validate > 0 and <= 255.
///
/// Used for dimension entry in matrix workflows (T-43-14 mitigation).
fn x_to_dim(state: &CalcState) -> Result<u8, HpError> {
    let trunc = state.stack.x.trunc_int();
    let v = trunc.inner().to_u8().ok_or(HpError::Domain)?;
    if v == 0 {
        return Err(HpError::Domain);
    }
    Ok(v)
}

/// Standard HP-41 stack drop: X←Y, Y←Z, Z←T, T unchanged.
fn stack_drop(state: &mut CalcState) {
    state.stack.x = state.stack.y.clone();
    state.stack.y = state.stack.z.clone();
    state.stack.z = state.stack.t.clone();
    // T unchanged — HP-41 hardware convention
}

/// Advance the modal workflow by one step given the current `AdvantageStep`.
///
/// ## TVM steps (TvmN/TvmI/TvmPv/TvmPmt/TvmFv/TvmBeginEnd)
/// Each arm stores the current X register value into the appropriate TVM register
/// and advances to the next modal step (or closes the modal after TvmBeginEnd).
///
/// ## Matrix workflow steps
///
/// ### MatrixNamePrompt (MATRX workflow)
/// Reads ALPHA register as matrix name. If matrix already exists: skip dimension
/// entry and advance to MatrxOperationChoice. If matrix is new: advance to
/// MatrixDimRowPrompt for dimension entry.
///
/// ### MatrixDimRowPrompt
/// Reads X as row count (validated > 0, ≤ 255). Stores to
/// `state.pending_adv_matrix_rows`. Advances to MatrixDimColPrompt.
///
/// ### MatrixDimColPrompt
/// Reads X as col count (validated > 0, ≤ 255). Uses pending_adv_matrix_rows +
/// pending_adv_matrix_name to construct the matrix via op_adv_matdim. Then
/// advances to MatrxOperationChoice.
///
/// ### MatrxOperationChoice
/// Reads X = integer choice (1=DET, 2=INV, 3=SYS). Clears modal, calls op.
///
/// ### MtrNamePrompt (MTR workflow)
/// Reads ALPHA register as matrix name. If exists: go to MatrxOperationChoice.
/// If new: go to MatrixDimRowPrompt → MatrixDimColPrompt → MeditElementPrompt loop.
///
/// ### MeditElementPrompt(r, c)
/// Stores X to element (r-1, c-1) via op_adv_ms. Advances to next element or
/// clears modal on last element.
///
/// ### CmeditElementPrompt(r, c)
/// Stores Y (imaginary) and X (real) to complex element at (r-1, c-1) in
/// interleaved layout. Advances to next element or clears modal on last element.
///
/// ### VeComponentPrompt(k)
/// Stores X to vector A register R(19+k). Advances to next component or clears
/// modal after component 3.
///
/// ### FdifeqOrderPrompt
/// Reads X as ODE order (1 or 2). Stores to pending state. Advances to
/// FdifeqFunctionNamePrompt.
///
/// ### FdifeqFunctionNamePrompt
/// Reads ALPHA as function name. Sets up AdvFdifeqState. Clears modal.
///
/// # Errors
/// - TVM arms: forwarded from TVM register-store ops.
/// - Matrix arms: `HpError::Domain` for invalid dimensions/choices; `HpError::InvalidOp`
///   for missing matrix.
/// - Non-implemented arms: `Err(HpError::InvalidOp)`.
pub fn submit_step(state: &mut CalcState, step: AdvantageStep) -> Result<(), HpError> {
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
                let tvm = state
                    .adv_tvm_state
                    .get_or_insert_with(crate::ops::advantage::tvm::TvmState::default);
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
                let tvm = state
                    .adv_tvm_state
                    .get_or_insert_with(crate::ops::advantage::tvm::TvmState::default);
                tvm.begin_mode = !x_val.is_zero();
            }
            // Workflow complete — clear modal state
            state.modal_program = None;
            state.modal_prompt = None;
            Ok(())
        }

        // ── MATRX name-entry (first step of MATRX workflow) ───────────────────
        AdvantageStep::MatrixNamePrompt => {
            // Read matrix name from ALPHA register (T-43-14: validate non-empty)
            let name = state.alpha_reg.trim().to_string();
            if name.is_empty() {
                return Err(HpError::InvalidOp);
            }
            // Store name for subsequent dim steps
            state.pending_adv_matrix_name = Some(name.clone());

            // If matrix already exists: skip dimension entry, go to operation choice
            if crate::ops::advantage::matrix_workflow::check_matrix_exists(state, &name) {
                state.adv_current_matrix = Some(name);
                state.modal_program =
                    Some(ModalProgram::Advantage(AdvantageStep::MatrxOperationChoice));
                state.modal_prompt = Some("MATRX OP?".to_string());
            } else {
                // New matrix: prompt for dimensions
                state.modal_program =
                    Some(ModalProgram::Advantage(AdvantageStep::MatrixDimRowPrompt));
                state.modal_prompt = Some("ROWS=?".to_string());
            }
            Ok(())
        }

        // ── Dimension row-entry ───────────────────────────────────────────────
        AdvantageStep::MatrixDimRowPrompt => {
            // Read row count from X; validate (T-43-14)
            let rows = x_to_dim(state)?;
            state.pending_adv_matrix_rows = Some(rows);
            // Drop X (consumed)
            stack_drop(state);
            state.modal_program = Some(ModalProgram::Advantage(AdvantageStep::MatrixDimColPrompt));
            state.modal_prompt = Some("COLS=?".to_string());
            Ok(())
        }

        // ── Dimension col-entry + MATDIM ──────────────────────────────────────
        AdvantageStep::MatrixDimColPrompt => {
            // Read col count from X; validate (T-43-14)
            let _cols = x_to_dim(state)?; // validated; actual cols used by op_adv_matdim via stack.x
            let rows = state
                .pending_adv_matrix_rows
                .take()
                .ok_or(HpError::InvalidOp)?;
            let name = state
                .pending_adv_matrix_name
                .clone()
                .ok_or(HpError::InvalidOp)?;

            // Push rows to Y (X already has cols), set alpha_reg to name, call op_adv_matdim
            // op_adv_matdim: reads alpha_reg=name, Y=rows, X=cols; drops Y+X
            // We need: Y=rows, X=cols — currently X=cols (user entered it), push rows to Y
            let rows_hp = HpNum::from(Decimal::from_u8(rows).ok_or(HpError::Domain)?);
            // Lift stack: Z←Y, Y←X (cols), then set Y=rows
            // Actually we need to push rows onto the stack so Y=rows, X=cols
            // Save current X (cols) temporarily
            let cols_hp = state.stack.x.clone();
            // Push rows: T←Z, Z←Y, Y←X (cols), X = rows — then re-set X = cols
            // Standard lift then enter rows, then cols to X
            // Simplest: manually set Y=rows, X=cols (op_adv_matdim reads these two)
            state.stack.y = rows_hp;
            state.stack.x = cols_hp;
            state.alpha_reg = name.clone();

            // Clear modal BEFORE calling op_adv_matdim (stat1 pattern)
            state.pending_adv_matrix_name = None;
            state.modal_program = None;
            state.modal_prompt = None;

            crate::ops::advantage::matrix_ops::op_adv_matdim(state)?;
            // op_adv_matdim: sets adv_current_matrix = name, resets I=0, J=0

            // Advance to operation choice
            state.modal_program =
                Some(ModalProgram::Advantage(AdvantageStep::MatrxOperationChoice));
            state.modal_prompt = Some("MATRX OP?".to_string());
            Ok(())
        }

        // ── MATRX operation selection ─────────────────────────────────────────
        AdvantageStep::MatrxOperationChoice => {
            // Read operation code from X: 1=DET, 2=INV, 3=SYS (T-43-14: validate)
            let trunc = state.stack.x.trunc_int();
            let choice = trunc.inner().to_i32().ok_or(HpError::Domain)?;
            // Drop X (consumed)
            stack_drop(state);
            // Clear modal state BEFORE dispatching (stat1 pattern)
            state.modal_program = None;
            state.modal_prompt = None;
            match choice {
                1 => crate::ops::advantage::matrix_linalg::op_adv_mdet(state),
                2 => crate::ops::advantage::matrix_linalg::op_adv_minv(state),
                3 => crate::ops::advantage::matrix_linalg::op_adv_msys(state),
                _ => Err(HpError::Domain),
            }
        }

        // ── MTR name-entry (first step of MTR workflow) ────────────────────────
        AdvantageStep::MtrNamePrompt => {
            let name = state.alpha_reg.trim().to_string();
            if name.is_empty() {
                return Err(HpError::InvalidOp);
            }
            state.pending_adv_matrix_name = Some(name.clone());

            if crate::ops::advantage::matrix_workflow::check_matrix_exists(state, &name) {
                // Matrix exists: go to operation choice
                state.adv_current_matrix = Some(name);
                state.modal_program =
                    Some(ModalProgram::Advantage(AdvantageStep::MatrxOperationChoice));
                state.modal_prompt = Some("MATRX OP?".to_string());
            } else {
                // New matrix: prompt for dimensions, then element entry loop
                state.modal_program =
                    Some(ModalProgram::Advantage(AdvantageStep::MatrixDimRowPrompt));
                state.modal_prompt = Some("ROWS=?".to_string());
            }
            Ok(())
        }

        // ── MEDIT element entry ───────────────────────────────────────────────
        AdvantageStep::MeditElementPrompt(r, c) => {
            // Get current matrix name + dimensions for bounds check
            let name = state
                .adv_current_matrix
                .clone()
                .filter(|n| !n.is_empty())
                .ok_or(HpError::InvalidOp)?;
            let (rows, cols) = {
                let mat = state
                    .adv_matrices
                    .iter()
                    .find(|m| m.name == name)
                    .ok_or(HpError::InvalidOp)?;
                (mat.rows, mat.cols)
            };

            // Set I/J to 0-based element position (r, c are 1-based)
            state.adv_matrix_i = r.saturating_sub(1);
            state.adv_matrix_j = c.saturating_sub(1);

            // Store X to element (I,J) — op_adv_ms uses adv_matrix_i/j + current_matrix_name
            crate::ops::advantage::matrix_ops::op_adv_ms(state)?;

            // Advance to next element or clear modal
            match crate::ops::advantage::matrix_workflow::next_element_1based(r, c, rows, cols) {
                Some((nr, nc)) => {
                    state.adv_matrix_i = nr.saturating_sub(1);
                    state.adv_matrix_j = nc.saturating_sub(1);
                    state.modal_program = Some(ModalProgram::Advantage(
                        AdvantageStep::MeditElementPrompt(nr, nc),
                    ));
                    state.modal_prompt = Some(format!("[{nr},{nc}]=?"));
                }
                None => {
                    // Last element — clear modal
                    state.modal_program = None;
                    state.modal_prompt = None;
                }
            }
            Ok(())
        }

        // ── CMEDIT complex element entry ──────────────────────────────────────
        AdvantageStep::CmeditElementPrompt(r, c) => {
            // Complex element: X = real part, Y = imaginary part
            // (HP-41 complex stack convention: Y+iX or X is real, Y is imag — we use X=real, Y=imag)
            let name = state
                .adv_current_matrix
                .clone()
                .filter(|n| !n.is_empty())
                .ok_or(HpError::InvalidOp)?;
            let (rows, cols) = {
                let mat = state
                    .adv_matrices
                    .iter()
                    .find(|m| m.name == name)
                    .ok_or(HpError::InvalidOp)?;
                (mat.rows, mat.cols)
            };

            let real_val = state.stack.x.clone();
            let imag_val = state.stack.y.clone();

            // Write interleaved complex data: data[2*(i*cols+j)] = real, data[2*(i*cols+j)+1] = imag
            let i = r.saturating_sub(1) as usize;
            let j = c.saturating_sub(1) as usize;
            let flat = 2 * (i * (cols as usize) + j);

            let mat = state
                .adv_matrices
                .iter_mut()
                .find(|m| m.name == name)
                .ok_or(HpError::InvalidOp)?;

            if flat + 1 >= mat.data.len() {
                return Err(HpError::Domain);
            }
            mat.data[flat] = real_val;
            mat.data[flat + 1] = imag_val;
            mat.is_complex = true;

            // Update I/J to next element or clear modal
            match crate::ops::advantage::matrix_workflow::next_element_1based(r, c, rows, cols) {
                Some((nr, nc)) => {
                    state.adv_matrix_i = nr.saturating_sub(1);
                    state.adv_matrix_j = nc.saturating_sub(1);
                    state.modal_program = Some(ModalProgram::Advantage(
                        AdvantageStep::CmeditElementPrompt(nr, nc),
                    ));
                    state.modal_prompt = Some(format!("C[{nr},{nc}]=?"));
                }
                None => {
                    state.modal_program = None;
                    state.modal_prompt = None;
                }
            }
            Ok(())
        }

        // ── VE component entry ────────────────────────────────────────────────
        AdvantageStep::VeComponentPrompt(k) => {
            // Store X to vector A component k (1-based → register 19+k)
            // Vector A: R20=comp1, R21=comp2, R22=comp3 (VEC_A_BASE=20)
            let component = state.stack.x.clone();
            let reg_idx = 19usize + (k as usize); // k=1→R20, k=2→R21, k=3→R22
            if reg_idx >= state.regs.len() {
                return Err(HpError::InvalidOp);
            }
            state.regs[reg_idx] = crate::num::HpValue::Numeric(component);
            // Drop X (consumed by component entry)
            stack_drop(state);

            if k < 3 {
                let next_k = k + 1;
                state.modal_program = Some(ModalProgram::Advantage(
                    AdvantageStep::VeComponentPrompt(next_k),
                ));
                state.modal_prompt = Some(format!("V[{next_k}]=?"));
            } else {
                // All 3 components entered — clear modal
                state.modal_program = None;
                state.modal_prompt = None;
            }
            Ok(())
        }

        // ── FDIFEQ order entry ────────────────────────────────────────────────
        AdvantageStep::FdifeqOrderPrompt => {
            // Read ODE order from X: must be 1 or 2
            let trunc = state.stack.x.trunc_int();
            let order = trunc.inner().to_u8().ok_or(HpError::Domain)?;
            if order != 1 && order != 2 {
                return Err(HpError::Domain);
            }
            // Drop X
            stack_drop(state);
            // Initialize FDIFEQ state with order
            let fdifeq = crate::ops::advantage::AdvFdifeqState {
                order,
                x: HpNum::zero(),
                y: vec![HpNum::zero(); order as usize],
                h: HpNum::zero(),
                function_label: String::new(),
                steps: 0,
                max_steps: 1000,
            };
            state.adv_fdifeq_state = Some(fdifeq);
            // Advance to function name entry
            state.modal_program = Some(ModalProgram::Advantage(
                AdvantageStep::FdifeqFunctionNamePrompt,
            ));
            state.modal_prompt = Some("F NAME?".to_string());
            Ok(())
        }

        // ── FDIFEQ function name entry ────────────────────────────────────────
        AdvantageStep::FdifeqFunctionNamePrompt => {
            let label = state.alpha_reg.trim().to_string();
            if label.is_empty() {
                return Err(HpError::InvalidOp);
            }
            // Store label in FDIFEQ state
            if let Some(ref mut fdifeq) = state.adv_fdifeq_state {
                fdifeq.function_label = label;
            } else {
                return Err(HpError::InvalidOp);
            }
            // Clear modal — FDIFEQ is ready to run
            state.modal_program = None;
            state.modal_prompt = None;
            Ok(())
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::num::HpNum;
    use crate::ops::advantage::AdvMatrix;
    use crate::state::CalcState;
    use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
    use rust_decimal::Decimal;

    // Helper: push a value to X
    fn push_x(state: &mut CalcState, v: f64) {
        state.stack.x = HpNum::from(Decimal::from_f64(v).unwrap());
    }

    // Helper: push values to Y and X (Y=y, X=x)
    fn push_yx(state: &mut CalcState, y: f64, x: f64) {
        state.stack.y = HpNum::from(Decimal::from_f64(y).unwrap());
        state.stack.x = HpNum::from(Decimal::from_f64(x).unwrap());
    }

    // Helper: add a named matrix to state
    fn add_matrix(state: &mut CalcState, name: &str, rows: u8, cols: u8) {
        let len = (rows as usize) * (cols as usize);
        state.adv_matrices.push(AdvMatrix {
            name: name.to_string(),
            rows,
            cols,
            is_complex: false,
            data: vec![HpNum::zero(); len],
        });
        state.adv_current_matrix = Some(name.to_string());
    }

    // ── Prompt helpers ────────────────────────────────────────────────────────

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
        assert!(requires_alpha_label(
            &AdvantageStep::FdifeqFunctionNamePrompt
        ));
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

    // ── TVM modal workflow ────────────────────────────────────────────────────

    // Catches: TVM modal workflow advances N→I→PV→PMT→FV→BeginEnd→done
    #[test]
    fn tvm_modal_workflow_complete_cycle() {
        let mut state = CalcState::new();

        // Step 1: submit N=12
        push_x(&mut state, 12.0);
        submit_step(&mut state, AdvantageStep::TvmN).unwrap();
        // LINT-EXEMPT: String comparison on modal_prompt (Option<String>), not HpNum/Decimal
        assert_eq!(state.modal_prompt, Some("I%YR=?".to_string()));
        let tvm = state.adv_tvm_state.as_ref().unwrap();
        // LINT-EXEMPT: pure-f64 tolerance on to_f64() extraction of exact integer (12.0), no iterative computation
        assert!((tvm.n.inner().to_f64().unwrap() - 12.0).abs() < 1e-9);

        // Step 2: submit I=6
        push_x(&mut state, 6.0);
        submit_step(&mut state, AdvantageStep::TvmI).unwrap();
        // LINT-EXEMPT: String comparison on modal_prompt (Option<String>), not HpNum/Decimal
        assert_eq!(state.modal_prompt, Some("PV=?".to_string()));
        let tvm = state.adv_tvm_state.as_ref().unwrap();
        // LINT-EXEMPT: pure-f64 tolerance on to_f64() extraction of exact integer (6.0), no iterative computation
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
        assert!(
            state.modal_program.is_none(),
            "modal cleared after TvmBeginEnd"
        );
        assert!(
            state.modal_prompt.is_none(),
            "prompt cleared after TvmBeginEnd"
        );
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

    // ── MATRX modal workflow ──────────────────────────────────────────────────

    // Catches: MatrixNamePrompt for new matrix advances to DimRowPrompt
    #[test]
    fn matrx_name_prompt_new_matrix_to_dim_row() {
        let mut state = CalcState::new();
        state.alpha_reg = "A".to_string();
        submit_step(&mut state, AdvantageStep::MatrixNamePrompt).unwrap();
        assert!(matches!(
            state.modal_program,
            Some(ModalProgram::Advantage(AdvantageStep::MatrixDimRowPrompt))
        ));
        assert_eq!(state.modal_prompt, Some("ROWS=?".to_string()));
        assert_eq!(state.pending_adv_matrix_name, Some("A".to_string()));
    }

    // Catches: MatrixNamePrompt for existing matrix skips to MatrxOperationChoice
    #[test]
    fn matrx_name_prompt_existing_matrix_skips_dim() {
        let mut state = CalcState::new();
        add_matrix(&mut state, "MYMAT", 2, 3);
        state.alpha_reg = "MYMAT".to_string();
        submit_step(&mut state, AdvantageStep::MatrixNamePrompt).unwrap();
        assert!(matches!(
            state.modal_program,
            Some(ModalProgram::Advantage(AdvantageStep::MatrxOperationChoice))
        ));
        assert_eq!(state.modal_prompt, Some("MATRX OP?".to_string()));
        assert_eq!(state.adv_current_matrix, Some("MYMAT".to_string()));
    }

    // Catches: MatrixNamePrompt rejects empty ALPHA
    #[test]
    fn matrx_name_prompt_rejects_empty_alpha() {
        let mut state = CalcState::new();
        state.alpha_reg = "  ".to_string(); // whitespace only
        assert!(matches!(
            submit_step(&mut state, AdvantageStep::MatrixNamePrompt),
            Err(HpError::InvalidOp)
        ));
    }

    // Catches: DimRowPrompt stores row count in pending field and advances
    #[test]
    fn dim_row_prompt_stores_rows_and_advances() {
        let mut state = CalcState::new();
        state.pending_adv_matrix_name = Some("A".to_string());
        push_x(&mut state, 3.0);
        submit_step(&mut state, AdvantageStep::MatrixDimRowPrompt).unwrap();
        assert_eq!(state.pending_adv_matrix_rows, Some(3u8));
        assert!(matches!(
            state.modal_program,
            Some(ModalProgram::Advantage(AdvantageStep::MatrixDimColPrompt))
        ));
        assert_eq!(state.modal_prompt, Some("COLS=?".to_string()));
    }

    // Catches: DimRowPrompt rejects zero dimension (T-43-14)
    #[test]
    fn dim_row_prompt_rejects_zero() {
        let mut state = CalcState::new();
        state.pending_adv_matrix_name = Some("A".to_string());
        push_x(&mut state, 0.0);
        assert!(matches!(
            submit_step(&mut state, AdvantageStep::MatrixDimRowPrompt),
            Err(HpError::Domain)
        ));
    }

    // Catches: DimColPrompt creates matrix and advances to MatrxOperationChoice
    #[test]
    fn dim_col_prompt_creates_matrix_and_advances() {
        let mut state = CalcState::new();
        state.pending_adv_matrix_name = Some("B".to_string());
        state.pending_adv_matrix_rows = Some(2);
        push_x(&mut state, 4.0); // cols=4
        submit_step(&mut state, AdvantageStep::MatrixDimColPrompt).unwrap();
        // Matrix should have been created
        let mat = state.adv_matrices.iter().find(|m| m.name == "B").unwrap();
        assert_eq!(mat.rows, 2);
        assert_eq!(mat.cols, 4);
        assert_eq!(mat.data.len(), 8);
        // Pending state cleared
        assert!(state.pending_adv_matrix_name.is_none());
        assert!(state.pending_adv_matrix_rows.is_none());
        // Modal advanced to operation choice
        assert!(matches!(
            state.modal_program,
            Some(ModalProgram::Advantage(AdvantageStep::MatrxOperationChoice))
        ));
    }

    // Catches: DimColPrompt rejects zero col (T-43-14)
    #[test]
    fn dim_col_prompt_rejects_zero() {
        let mut state = CalcState::new();
        state.pending_adv_matrix_name = Some("C".to_string());
        state.pending_adv_matrix_rows = Some(3);
        push_x(&mut state, 0.0);
        assert!(matches!(
            submit_step(&mut state, AdvantageStep::MatrixDimColPrompt),
            Err(HpError::Domain)
        ));
    }

    // Catches: MATRX full workflow name→rows→cols→operation
    #[test]
    fn matrx_full_workflow_name_to_dim_to_op() {
        let mut state = CalcState::new();

        // Step 1: enter name
        state.alpha_reg = "TEST".to_string();
        crate::ops::advantage::matrix_workflow::op_adv_matrx(&mut state).unwrap();
        assert!(matches!(
            state.modal_program,
            Some(ModalProgram::Advantage(AdvantageStep::MatrixNamePrompt))
        ));

        // Step 2: submit name
        submit_step(&mut state, AdvantageStep::MatrixNamePrompt).unwrap();
        assert!(matches!(
            state.modal_program,
            Some(ModalProgram::Advantage(AdvantageStep::MatrixDimRowPrompt))
        ));

        // Step 3: submit rows=3
        push_x(&mut state, 3.0);
        submit_step(&mut state, AdvantageStep::MatrixDimRowPrompt).unwrap();
        assert_eq!(state.pending_adv_matrix_rows, Some(3u8));

        // Step 4: submit cols=3
        push_x(&mut state, 3.0);
        submit_step(&mut state, AdvantageStep::MatrixDimColPrompt).unwrap();

        // Matrix should exist 3x3
        let mat = state
            .adv_matrices
            .iter()
            .find(|m| m.name == "TEST")
            .unwrap();
        assert_eq!(mat.rows, 3);
        assert_eq!(mat.cols, 3);

        // Modal at operation choice
        assert!(matches!(
            state.modal_program,
            Some(ModalProgram::Advantage(AdvantageStep::MatrxOperationChoice))
        ));

        // Step 5: submit op=1 (DET - will return InvalidOp since linalg is stub)
        push_x(&mut state, 1.0);
        // MDET is a stub that returns InvalidOp - workflow should reach it
        let _ = submit_step(&mut state, AdvantageStep::MatrxOperationChoice);
        // Modal is cleared regardless
        assert!(state.modal_program.is_none());
    }

    // Catches: MatrxOperationChoice with invalid choice returns Domain
    #[test]
    fn matrx_op_choice_invalid_returns_domain() {
        let mut state = CalcState::new();
        add_matrix(&mut state, "A", 2, 2);
        push_x(&mut state, 99.0); // invalid choice
        assert!(matches!(
            submit_step(&mut state, AdvantageStep::MatrxOperationChoice),
            Err(HpError::Domain)
        ));
        // Modal cleared on domain error too
        assert!(state.modal_program.is_none());
    }

    // ── MTR modal workflow ────────────────────────────────────────────────────

    // Catches: MtrNamePrompt for existing matrix goes to MatrxOperationChoice
    #[test]
    fn mtr_name_prompt_existing_matrix() {
        let mut state = CalcState::new();
        add_matrix(&mut state, "MAT", 3, 3);
        state.alpha_reg = "MAT".to_string();
        submit_step(&mut state, AdvantageStep::MtrNamePrompt).unwrap();
        assert!(matches!(
            state.modal_program,
            Some(ModalProgram::Advantage(AdvantageStep::MatrxOperationChoice))
        ));
    }

    // Catches: MtrNamePrompt for new matrix advances to dimension entry
    #[test]
    fn mtr_name_prompt_new_matrix() {
        let mut state = CalcState::new();
        state.alpha_reg = "NEW".to_string();
        submit_step(&mut state, AdvantageStep::MtrNamePrompt).unwrap();
        assert!(matches!(
            state.modal_program,
            Some(ModalProgram::Advantage(AdvantageStep::MatrixDimRowPrompt))
        ));
    }

    // Catches: MtrNamePrompt rejects empty ALPHA
    #[test]
    fn mtr_name_prompt_rejects_empty() {
        let mut state = CalcState::new();
        state.alpha_reg = String::new();
        assert!(matches!(
            submit_step(&mut state, AdvantageStep::MtrNamePrompt),
            Err(HpError::InvalidOp)
        ));
    }

    // ── MEDIT element workflow ────────────────────────────────────────────────

    // Catches: MeditElementPrompt stores X and advances to next element
    #[test]
    fn medit_element_prompt_stores_and_advances() {
        let mut state = CalcState::new();
        add_matrix(&mut state, "M", 2, 2);
        // Push value 42.0 to X for element (1,1)
        push_x(&mut state, 42.0);
        submit_step(&mut state, AdvantageStep::MeditElementPrompt(1, 1)).unwrap();

        // Matrix element [0][0] should be 42
        let mat = state.adv_matrices.iter().find(|m| m.name == "M").unwrap();
        // LINT-EXEMPT: pure-f64 tolerance on to_f64() extraction of exact integer (42.0), no iterative computation
        assert!((mat.data[0].inner().to_f64().unwrap() - 42.0).abs() < 1e-9);

        // Modal advanced to (1,2)
        assert!(matches!(
            state.modal_program,
            Some(ModalProgram::Advantage(AdvantageStep::MeditElementPrompt(
                1, 2
            )))
        ));
        assert_eq!(state.modal_prompt, Some("[1,2]=?".to_string()));
    }

    // Catches: MeditElementPrompt clears modal on last element
    #[test]
    fn medit_last_element_clears_modal() {
        let mut state = CalcState::new();
        add_matrix(&mut state, "M1", 2, 2);
        push_x(&mut state, 7.0);
        submit_step(&mut state, AdvantageStep::MeditElementPrompt(2, 2)).unwrap();
        // Last element (2,2) in 2x2 → modal cleared
        assert!(state.modal_program.is_none());
        assert!(state.modal_prompt.is_none());
    }

    // Catches: MEDIT iterates through all elements in row-major order
    #[test]
    fn medit_full_element_iteration() {
        let mut state = CalcState::new();
        add_matrix(&mut state, "M2", 2, 3); // 2x3 matrix = 6 elements

        let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        // Element order: (1,1),(1,2),(1,3),(2,1),(2,2),(2,3)
        let order: &[(u8, u8)] = &[(1, 1), (1, 2), (1, 3), (2, 1), (2, 2), (2, 3)];

        for (i, (r, c)) in order.iter().enumerate() {
            push_x(&mut state, values[i]);
            submit_step(&mut state, AdvantageStep::MeditElementPrompt(*r, *c)).unwrap();
        }
        assert!(
            state.modal_program.is_none(),
            "modal cleared after last element"
        );

        let mat = state.adv_matrices.iter().find(|m| m.name == "M2").unwrap();
        for (i, v) in values.iter().enumerate() {
            // LINT-EXEMPT: pure-f64 tolerance on to_f64() extraction of exact test values, no iterative computation
            assert!(
                (mat.data[i].inner().to_f64().unwrap() - v).abs() < 1e-9,
                "element {i} mismatch"
            );
        }
    }

    // ── CMEDIT complex element workflow ───────────────────────────────────────

    // Catches: CmeditElementPrompt stores real (X) and imag (Y) interleaved
    #[test]
    fn cmedit_stores_complex_element() {
        let mut state = CalcState::new();
        // Create a complex 1x1 matrix manually
        state.adv_matrices.push(AdvMatrix {
            name: "CM".to_string(),
            rows: 1,
            cols: 1,
            is_complex: true,
            data: vec![HpNum::zero(); 2], // 1 complex element = 2 slots
        });
        state.adv_current_matrix = Some("CM".to_string());

        push_yx(&mut state, 3.0, 5.0); // Y=3.0 (imag), X=5.0 (real)
        submit_step(&mut state, AdvantageStep::CmeditElementPrompt(1, 1)).unwrap();

        let mat = state.adv_matrices.iter().find(|m| m.name == "CM").unwrap();
        // LINT-EXEMPT: pure-f64 tolerance on to_f64() extraction of exact integer (5.0), no iterative computation
        assert!(
            (mat.data[0].inner().to_f64().unwrap() - 5.0).abs() < 1e-9,
            "real part"
        );
        // LINT-EXEMPT: pure-f64 tolerance on to_f64() extraction of exact integer (3.0), no iterative computation
        assert!(
            (mat.data[1].inner().to_f64().unwrap() - 3.0).abs() < 1e-9,
            "imag part"
        );
        assert!(state.modal_program.is_none(), "1x1 → modal cleared");
    }

    // Catches: CmeditElementPrompt advances through 2x1 complex matrix
    #[test]
    fn cmedit_advances_to_next_complex_element() {
        let mut state = CalcState::new();
        state.adv_matrices.push(AdvMatrix {
            name: "CM2".to_string(),
            rows: 2,
            cols: 1,
            is_complex: true,
            data: vec![HpNum::zero(); 4], // 2 complex elements = 4 slots
        });
        state.adv_current_matrix = Some("CM2".to_string());

        push_yx(&mut state, 1.0, 2.0); // element (1,1): real=2, imag=1
        submit_step(&mut state, AdvantageStep::CmeditElementPrompt(1, 1)).unwrap();

        assert!(matches!(
            state.modal_program,
            Some(ModalProgram::Advantage(AdvantageStep::CmeditElementPrompt(
                2, 1
            )))
        ));
    }

    // ── VeComponentPrompt workflow ────────────────────────────────────────────

    // Catches: VeComponentPrompt stores component and advances
    #[test]
    fn ve_component_stores_and_advances() {
        let mut state = CalcState::new();
        // Ensure state.regs has enough entries (VEC_A_BASE=20, need R20-R22)
        while state.regs.len() < 30 {
            state.regs.push(crate::num::HpValue::default());
        }
        push_x(&mut state, 1.5);
        submit_step(&mut state, AdvantageStep::VeComponentPrompt(1)).unwrap();
        // R20 = 1.5
        if let crate::num::HpValue::Numeric(ref v) = state.regs[20] {
            // LINT-EXEMPT: pure-f64 tolerance on to_f64() extraction of 1.5, no iterative computation
            assert!((v.inner().to_f64().unwrap() - 1.5).abs() < 1e-9);
        } else {
            panic!("R20 should be numeric");
        }
        assert!(matches!(
            state.modal_program,
            Some(ModalProgram::Advantage(AdvantageStep::VeComponentPrompt(2)))
        ));
    }

    // Catches: VeComponentPrompt clears modal after component 3
    #[test]
    fn ve_last_component_clears_modal() {
        let mut state = CalcState::new();
        while state.regs.len() < 30 {
            state.regs.push(crate::num::HpValue::default());
        }
        push_x(&mut state, 9.0);
        submit_step(&mut state, AdvantageStep::VeComponentPrompt(3)).unwrap();
        assert!(state.modal_program.is_none());
    }

    // ── FDIFEQ workflow ───────────────────────────────────────────────────────

    // Catches: FdifeqOrderPrompt stores order and advances to function name
    #[test]
    fn fdifeq_order_prompt_stores_and_advances() {
        let mut state = CalcState::new();
        push_x(&mut state, 2.0); // order = 2
        submit_step(&mut state, AdvantageStep::FdifeqOrderPrompt).unwrap();
        assert!(state.adv_fdifeq_state.is_some());
        assert_eq!(state.adv_fdifeq_state.as_ref().unwrap().order, 2);
        assert!(matches!(
            state.modal_program,
            Some(ModalProgram::Advantage(
                AdvantageStep::FdifeqFunctionNamePrompt
            ))
        ));
    }

    // Catches: FdifeqOrderPrompt rejects invalid order (e.g. 3)
    #[test]
    fn fdifeq_order_prompt_rejects_invalid_order() {
        let mut state = CalcState::new();
        push_x(&mut state, 3.0);
        assert!(matches!(
            submit_step(&mut state, AdvantageStep::FdifeqOrderPrompt),
            Err(HpError::Domain)
        ));
    }

    // Catches: FdifeqFunctionNamePrompt stores label and clears modal
    #[test]
    fn fdifeq_function_name_stores_label_and_clears() {
        let mut state = CalcState::new();
        // Set up FDIFEQ state first
        state.adv_fdifeq_state = Some(crate::ops::advantage::AdvFdifeqState {
            order: 1,
            x: HpNum::zero(),
            y: vec![HpNum::zero()],
            h: HpNum::zero(),
            function_label: String::new(),
            steps: 0,
            max_steps: 1000,
        });
        state.alpha_reg = "DERIV".to_string();
        submit_step(&mut state, AdvantageStep::FdifeqFunctionNamePrompt).unwrap();
        assert_eq!(
            state.adv_fdifeq_state.as_ref().unwrap().function_label,
            "DERIV"
        );
        assert!(state.modal_program.is_none());
        assert!(state.modal_prompt.is_none());
    }

    // Catches: FdifeqFunctionNamePrompt rejects empty ALPHA
    #[test]
    fn fdifeq_function_name_rejects_empty() {
        let mut state = CalcState::new();
        state.adv_fdifeq_state = Some(crate::ops::advantage::AdvFdifeqState::default());
        state.alpha_reg = String::new();
        assert!(matches!(
            submit_step(&mut state, AdvantageStep::FdifeqFunctionNamePrompt),
            Err(HpError::InvalidOp)
        ));
    }

    // Catches: FdifeqFunctionNamePrompt fails if no FDIFEQ state initialized
    #[test]
    fn fdifeq_function_name_fails_without_state() {
        let mut state = CalcState::new();
        state.adv_fdifeq_state = None;
        state.alpha_reg = "FUNC".to_string();
        assert!(matches!(
            submit_step(&mut state, AdvantageStep::FdifeqFunctionNamePrompt),
            Err(HpError::InvalidOp)
        ));
    }
}
