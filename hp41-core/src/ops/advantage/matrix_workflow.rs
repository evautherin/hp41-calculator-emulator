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
//! MATRX opens the matrix-operation-choice modal (ADV MATH integration).
//! MTR opens the target-matrix-name modal.
//! MEDIT opens the element-editing loop for a real matrix.
//! CMEDIT opens the element-editing loop for a complex matrix.
//!
//! Full modal implementations in Plan 43-08 (matrix workflow / MEDIT/CMEDIT).

use crate::{
    error::HpError,
    state::CalcState,
};

/// ADV MATRX — open matrix-operation-choice modal (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_matrx(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MTR — open target-matrix-name modal for copy operations (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_mtr(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MEDIT — enter real-matrix element-editing loop at (I,J) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_medit(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV CMEDIT — enter complex-matrix element-editing loop at (I,J) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_cmedit(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Catches: matrix workflow stubs return InvalidOp (skeleton phase invariant)
    #[test]
    fn matrix_workflow_stubs_return_invalid_op() {
        let mut state = CalcState::new();
        assert!(matches!(op_adv_matrx(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_mtr(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_medit(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_cmedit(&mut state), Err(HpError::InvalidOp)));
    }
}
