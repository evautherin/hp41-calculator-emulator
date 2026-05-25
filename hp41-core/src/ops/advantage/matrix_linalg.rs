// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `matrix_linalg` — ADV MTRX high-level linear algebra operations.
//!
//! XROM module id 22 (ADV_MATH_A, bit-3 of `CalcState::xrom_modules`).
//!
//! D-43.5 ISOLATION INVARIANT: No function in this file reads or writes
//! `state.matrix_dim` or `state.matrix_active_reg`.  Named-matrix storage
//! uses `state.adv_matrices` exclusively.
//!
//! Operations: MDET / MINV / MSYS / M*M / MAT+ / MAT- / MAT*c / MAT/c / TRNPS / MMOVE
//!
//! Full implementations in Plan 43-05 (linear algebra).

use crate::{
    error::HpError,
    state::CalcState,
};

/// ADV MDET — compute determinant of current square matrix (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_mdet(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MINV — compute in-place matrix inverse (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_minv(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MSYS — solve linear system Ax = b (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_msys(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV M*M — multiply two named matrices (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_m_mul_m(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MAT+ — add two named matrices element-wise (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_mat_plus(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MAT- — subtract two named matrices element-wise (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_mat_minus(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MAT*c — multiply matrix by scalar X (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_mat_scalar_mul(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MAT/c — divide matrix by scalar X (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_mat_scalar_div(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV TRNPS — transpose current matrix in-place (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_trnps(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MMOVE — copy elements of source matrix into destination matrix (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_mmove(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Catches: all ADV MTRX linalg stubs return InvalidOp (skeleton phase invariant)
    #[test]
    fn matrix_linalg_stubs_return_invalid_op() {
        let mut state = CalcState::new();
        assert!(matches!(op_adv_mdet(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_minv(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_msys(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_m_mul_m(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_mat_plus(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_mat_minus(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_mat_scalar_mul(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_mat_scalar_div(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_trnps(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_mmove(&mut state), Err(HpError::InvalidOp)));
    }
}
