// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `matrix_workflow` — ADV MATH/MTRX modal frontend operations.
//!
//! XROM module id 22 (ADV_MATH_A, bit-3) and 24 (ADV_MATH_B, bit-4).
//!
//! D-43.5 ISOLATION INVARIANT: No function in this file reads or writes
//! `state.matrix_dim` or `state.matrix_active_reg`.  Named-matrix storage
//! uses `state.adv_matrices` exclusively.
//!
//! Operations: MATRX / MTR / MEDIT / CMEDIT
//!
//! ## MATRX (ADV-MATH-01)
//!
//! Full matrix workflow modal. Opens a multi-step prompt sequence:
//!   1. MatrixNamePrompt — "MNAME?" — user enters matrix name via ALPHA, presses R/S.
//!   2. If matrix already exists: skip dimensions, go directly to MatrxOperationChoice.
//!      If matrix is new: MatrixDimRowPrompt — "ROWS=?" — user enters row count.
//!   3. MatrixDimColPrompt — "COLS=?" — user enters col count; calls op_adv_matdim.
//!   4. MatrxOperationChoice — "MATRX OP?" — user enters 1=DET, 2=INV, 3=SYS; dispatches.
//!
//! ## MTR (ADV-MATH-02)
//!
//! Simplified matrix entry/solve. Opens with MtrNamePrompt — "MTR NAME?". After name entry:
//! - If matrix exists: go directly to MatrxOperationChoice.
//! - If matrix is new: go to MatrixDimRowPrompt → MatrixDimColPrompt → element entry
//!   (MeditElementPrompt loop) → MatrxOperationChoice.
//!
//! ## MEDIT (ADV-MTX-49)
//!
//! Real matrix editor. Iterates through elements row-by-row prompting for values.
//! For each element (i,j) (1-based), prompts "R{i}C{j}=?" and stores via op_adv_ms.
//!
//! ## CMEDIT (ADV-MTX-50)
//!
//! Complex matrix editor. Same as MEDIT but prompts for real and imaginary parts per element.

use crate::{
    error::HpError,
    ops::advantage::modal::AdvantageStep,
    ops::math1::modal::ModalProgram,
    stack::{apply_lift_effect, LiftEffect},
    state::CalcState,
};

// ── Internal helper ───────────────────────────────────────────────────────────

/// Check whether a matrix named `name` already exists in `state.adv_matrices`.
fn matrix_exists(state: &CalcState, name: &str) -> bool {
    state.adv_matrices.iter().any(|m| m.name == name)
}

// ── Public ops ────────────────────────────────────────────────────────────────

/// ADV MATRX — open matrix-operation-choice modal (ADV-MATH-01).
///
/// Opens the MATRX multi-step modal workflow. First prompt: "MNAME?" — the user
/// types a matrix name in ALPHA then presses R/S. Subsequent steps are handled
/// by `modal::submit_step` for `AdvantageStep::MatrixNamePrompt`.
///
/// LiftEffect::Neutral (no stack change; modal interaction follows).
///
/// # Errors
/// Returns `Ok(())` — modal setup is infallible.
pub fn op_adv_matrx(state: &mut CalcState) -> Result<(), HpError> {
    state.modal_program = Some(ModalProgram::Advantage(AdvantageStep::MatrixNamePrompt));
    state.modal_prompt = Some("MNAME?".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV MTR — open target-matrix-name modal for entry/solve (ADV-MATH-02).
///
/// Opens the MTR multi-step modal workflow. First prompt: "MTR NAME?" — the user
/// types a matrix name in ALPHA then presses R/S. Subsequent steps are handled
/// by `modal::submit_step` for `AdvantageStep::MtrNamePrompt`.
///
/// LiftEffect::Neutral (no stack change; modal interaction follows).
///
/// # Errors
/// Returns `Ok(())` — modal setup is infallible.
pub fn op_adv_mtr(state: &mut CalcState) -> Result<(), HpError> {
    state.modal_program = Some(ModalProgram::Advantage(AdvantageStep::MtrNamePrompt));
    state.modal_prompt = Some("MTR NAME?".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV MEDIT — enter real-matrix element-editing loop at (I,J) (ADV-MTX-49).
///
/// Opens element-editing modal starting at element (1,1). Each R/S submit stores
/// the entered value to the current element and advances to the next (row-major).
/// After the last element the modal is cleared. If no matrix is active or the
/// matrix dimensions are zero, returns `HpError::InvalidOp`.
///
/// LiftEffect::Neutral (no stack change; modal interaction follows).
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active or matrix has no elements.
pub fn op_adv_medit(state: &mut CalcState) -> Result<(), HpError> {
    // Require a current active matrix
    let name = state
        .adv_current_matrix
        .clone()
        .filter(|n| !n.is_empty())
        .ok_or(HpError::InvalidOp)?;
    let mat = state
        .adv_matrices
        .iter()
        .find(|m| m.name == name)
        .ok_or(HpError::InvalidOp)?;
    if mat.rows == 0 || mat.cols == 0 {
        return Err(HpError::InvalidOp);
    }
    // Start element-editing from (1,1) (1-based display)
    state.adv_matrix_i = 0; // 0-based storage
    state.adv_matrix_j = 0;
    state.modal_program = Some(ModalProgram::Advantage(AdvantageStep::MeditElementPrompt(
        1, 1,
    )));
    state.modal_prompt = Some("[1,1]=?".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV CMEDIT — enter complex-matrix element-editing loop at (I,J) (ADV-MTX-50).
///
/// Same as MEDIT but prompts for real and imaginary parts per element via
/// `CmeditElementPrompt`. Operates on `is_complex = true` matrices.
/// Returns `HpError::InvalidOp` if no complex matrix is active.
///
/// LiftEffect::Neutral (no stack change; modal interaction follows).
///
/// # Errors
/// Returns `HpError::InvalidOp` if no complex matrix is active or it has no elements.
pub fn op_adv_cmedit(state: &mut CalcState) -> Result<(), HpError> {
    let name = state
        .adv_current_matrix
        .clone()
        .filter(|n| !n.is_empty())
        .ok_or(HpError::InvalidOp)?;
    let mat = state
        .adv_matrices
        .iter_mut()
        .find(|m| m.name == name)
        .ok_or(HpError::InvalidOp)?;
    if mat.rows == 0 || mat.cols == 0 {
        return Err(HpError::InvalidOp);
    }
    // Complex element editing stores interleaved (re, im) pairs, so it needs
    // 2·rows·cols slots — but MATDIM allocates only rows·cols reals. Promote the
    // matrix to complex storage here (existing values become real parts, zero
    // imaginary), otherwise every element past the first row fails the bounds
    // check in `submit_step` with `Domain`. Idempotent if already complex-sized.
    let needed = 2 * (mat.rows as usize) * (mat.cols as usize);
    if mat.data.len() != needed {
        let mut complex_data = vec![crate::num::HpNum::zero(); needed];
        for (k, val) in mat.data.iter().enumerate() {
            if let Some(slot) = complex_data.get_mut(2 * k) {
                *slot = val.clone();
            }
        }
        mat.data = complex_data;
    }
    mat.is_complex = true;
    // Start complex element-editing from (1,1)
    state.adv_matrix_i = 0;
    state.adv_matrix_j = 0;
    state.modal_program = Some(ModalProgram::Advantage(AdvantageStep::CmeditElementPrompt(
        1, 1,
    )));
    state.modal_prompt = Some("C[1,1]=?".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// Compute the next (row, col) pair in row-major traversal of an `rows × cols` matrix.
///
/// Both `r` and `c` are 1-based. Returns `None` when `(r, c)` is the last element.
pub(crate) fn next_element_1based(r: u8, c: u8, rows: u8, cols: u8) -> Option<(u8, u8)> {
    if r == 0 || c == 0 || rows == 0 || cols == 0 {
        return None;
    }
    if c < cols {
        Some((r, c + 1))
    } else if r < rows {
        Some((r + 1, 1))
    } else {
        None // last element
    }
}

/// Check whether `matrix_exists` is accessible from `modal.rs` via this re-export.
pub(crate) fn check_matrix_exists(state: &CalcState, name: &str) -> bool {
    matrix_exists(state, name)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::num::HpNum;
    use crate::ops::advantage::AdvMatrix;

    // Helper: set up a named matrix in state
    fn add_matrix(state: &mut CalcState, name: &str, rows: u8, cols: u8, is_complex: bool) {
        let len = if is_complex {
            (rows as usize) * (cols as usize) * 2
        } else {
            (rows as usize) * (cols as usize)
        };
        state.adv_matrices.push(AdvMatrix {
            name: name.to_string(),
            rows,
            cols,
            is_complex,
            data: vec![HpNum::zero(); len],
        });
        state.adv_current_matrix = Some(name.to_string());
    }

    // Catches: MATRX opens MatrixNamePrompt modal with correct prompt
    #[test]
    fn matrx_opens_matrix_name_prompt() {
        let mut state = CalcState::new();
        op_adv_matrx(&mut state).unwrap();
        assert!(matches!(
            state.modal_program,
            Some(ModalProgram::Advantage(AdvantageStep::MatrixNamePrompt))
        ));
        assert_eq!(state.modal_prompt, Some("MNAME?".to_string()));
    }

    // Catches: MTR opens MtrNamePrompt modal with correct prompt
    #[test]
    fn mtr_opens_mtr_name_prompt() {
        let mut state = CalcState::new();
        op_adv_mtr(&mut state).unwrap();
        assert!(matches!(
            state.modal_program,
            Some(ModalProgram::Advantage(AdvantageStep::MtrNamePrompt))
        ));
        assert_eq!(state.modal_prompt, Some("MTR NAME?".to_string()));
    }

    // Catches: MEDIT opens MeditElementPrompt(1,1) when matrix is active
    #[test]
    fn medit_opens_element_prompt_for_active_matrix() {
        let mut state = CalcState::new();
        add_matrix(&mut state, "A", 3, 2, false);
        op_adv_medit(&mut state).unwrap();
        assert!(matches!(
            state.modal_program,
            Some(ModalProgram::Advantage(AdvantageStep::MeditElementPrompt(
                1, 1
            )))
        ));
        assert_eq!(state.modal_prompt, Some("[1,1]=?".to_string()));
        assert_eq!(state.adv_matrix_i, 0, "i reset to 0 (0-based)");
        assert_eq!(state.adv_matrix_j, 0, "j reset to 0 (0-based)");
    }

    // Catches: MEDIT fails when no matrix is active
    #[test]
    fn medit_fails_without_active_matrix() {
        let mut state = CalcState::new();
        assert!(matches!(op_adv_medit(&mut state), Err(HpError::InvalidOp)));
    }

    // Catches: MEDIT fails when matrix has zero dimensions
    #[test]
    fn medit_fails_for_zero_dimension_matrix() {
        let mut state = CalcState::new();
        state.adv_matrices.push(AdvMatrix {
            name: "Z".to_string(),
            rows: 0,
            cols: 0,
            is_complex: false,
            data: vec![],
        });
        state.adv_current_matrix = Some("Z".to_string());
        assert!(matches!(op_adv_medit(&mut state), Err(HpError::InvalidOp)));
    }

    // Catches: CMEDIT opens CmeditElementPrompt(1,1) for complex matrix
    #[test]
    fn cmedit_opens_complex_element_prompt() {
        let mut state = CalcState::new();
        add_matrix(&mut state, "C", 2, 2, true);
        op_adv_cmedit(&mut state).unwrap();
        assert!(matches!(
            state.modal_program,
            Some(ModalProgram::Advantage(AdvantageStep::CmeditElementPrompt(
                1, 1
            )))
        ));
        assert_eq!(state.modal_prompt, Some("C[1,1]=?".to_string()));
    }

    // Regression: CMEDIT on a real-sized matrix (as MATDIM allocates it) must
    // promote storage to 2*rows*cols so every element is addressable. The old
    // code left data at rows*cols, so elements past the first row hit Domain.
    #[test]
    fn cmedit_promotes_real_matrix_to_complex_storage() {
        let mut state = CalcState::new();
        add_matrix(&mut state, "M", 2, 2, false); // real-sized: data.len() == 4
        state.adv_matrices[0].data[3] = HpNum::from(7i32); // (1,1) real part, flat 3

        op_adv_cmedit(&mut state).unwrap();

        let mat = state.adv_matrices.iter().find(|m| m.name == "M").unwrap();
        assert!(mat.is_complex, "CMEDIT must mark the matrix complex");
        // LINT-EXEMPT: structural length check, not an iterated FP computation
        assert_eq!(mat.data.len(), 8, "complex 2x2 needs 2*rows*cols = 8 slots");
        // Existing real values are re-interleaved as real parts (imag = 0).
        // LINT-EXEMPT: exact-integer HpNum equality (re-interleave), not iterated FP
        assert_eq!(
            mat.data[6],
            HpNum::from(7i32),
            "real value moved to slot 2*k"
        );
        // LINT-EXEMPT: exact zero-init check, not iterated FP
        assert_eq!(
            mat.data[7],
            HpNum::zero(),
            "imaginary part zero-initialised"
        );
    }

    // Catches: CMEDIT fails without active matrix
    #[test]
    fn cmedit_fails_without_active_matrix() {
        let mut state = CalcState::new();
        assert!(matches!(op_adv_cmedit(&mut state), Err(HpError::InvalidOp)));
    }

    // Catches: next_element_1based advances col first, then row
    #[test]
    fn next_element_row_major_traversal() {
        // 2x3 matrix: (1,1)→(1,2)→(1,3)→(2,1)→(2,2)→(2,3)→None
        assert_eq!(next_element_1based(1, 1, 2, 3), Some((1, 2)));
        assert_eq!(next_element_1based(1, 2, 2, 3), Some((1, 3)));
        assert_eq!(next_element_1based(1, 3, 2, 3), Some((2, 1)));
        assert_eq!(next_element_1based(2, 1, 2, 3), Some((2, 2)));
        assert_eq!(next_element_1based(2, 2, 2, 3), Some((2, 3)));
        assert_eq!(next_element_1based(2, 3, 2, 3), None);
    }

    // Catches: matrix_exists helper works correctly
    #[test]
    fn check_matrix_exists_helper() {
        let mut state = CalcState::new();
        assert!(!check_matrix_exists(&state, "A"));
        add_matrix(&mut state, "A", 2, 2, false);
        assert!(check_matrix_exists(&state, "A"));
        assert!(!check_matrix_exists(&state, "B"));
    }
}
