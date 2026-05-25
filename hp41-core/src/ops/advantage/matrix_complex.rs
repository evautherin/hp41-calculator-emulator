// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `matrix_complex` — ADV MTRX complex-matrix operations.
//!
//! XROM module id 22 (ADV_MATH_A, bit-3 of `CalcState::xrom_modules`).
//!
//! D-43.5 ISOLATION INVARIANT: No function in this file reads or writes
//! `state.matrix_dim` or `state.matrix_active_reg`.  Named-matrix storage
//! uses `state.adv_matrices` exclusively.
//!
//! Operations: C<>C / CMAXAB / CNRM / CSUM / YC+C
//!
//! Full implementations in Plan 43-05 (complex matrix operations).

use crate::{
    error::HpError,
    state::CalcState,
};

/// ADV C<>C — exchange complex element (I,J) with complex element (I',J') (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_c_exchange_c(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV CMAXAB — maximum absolute-value complex element in current matrix (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_cmaxab(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV CNRM — complex Frobenius norm of current matrix (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_cnrm(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV CSUM — sum of complex elements in current matrix (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_csum(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV YC+C — add scalar complex (Y:X) to every element of current matrix (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_yc_plus_c(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Catches: all ADV MTRX complex stubs return InvalidOp (skeleton phase invariant)
    #[test]
    fn matrix_complex_stubs_return_invalid_op() {
        let mut state = CalcState::new();
        assert!(matches!(op_adv_c_exchange_c(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_cmaxab(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_cnrm(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_csum(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_yc_plus_c(&mut state), Err(HpError::InvalidOp)));
    }
}
