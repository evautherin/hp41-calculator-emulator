// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `matrix_ops` — ADV MTRX element access, lifecycle, and reduction operations.
//!
//! XROM module id 22 (ADV_MATH_A, bit-3 of `CalcState::xrom_modules`).
//!
//! D-43.5 ISOLATION INVARIANT: No function in this file reads or writes
//! `state.matrix_dim` or `state.matrix_active_reg`.  Those fields belong
//! exclusively to the Math Pac I matrix (Plan 28-06).  Named-matrix storage
//! uses `state.adv_matrices` + `state.adv_matrix_i` + `state.adv_matrix_j`
//! exclusively.
//!
//! Operations: I+ / I- / J+ / J- / MR / MS / MRIJ / MSIJ / MSIJR /
//!             MRC+ / MRC- / MRR+ / MRR- / MSR+ / MSC+ / MSWAP /
//!             MNAME? / DIM? / MATDIM / MP / PIV /
//!             R<>R / R>R? / SUM / SUMAB / MAX / MAXAB / MIN / RMAXAB / RNRM / RSUM / FNRM
//!
//! Full implementations in Plan 43-04 (element access + lifecycle) and
//! Plan 43-05 (reductions + norms + row ops).

use crate::{
    error::HpError,
    state::CalcState,
};

/// ADV I+ — increment current row index by 1 (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_i_plus(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV I- — decrement current row index by 1 (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_i_minus(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV J+ — increment current column index by 1 (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_j_plus(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV J- — decrement current column index by 1 (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_j_minus(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MR — matrix recall: push element (I,J) of current matrix onto stack (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_mr(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MS — matrix store: store X into element (I,J) of current matrix (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_ms(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MRIJ — matrix recall with I/J from Y/X (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_mrij(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MSIJ — matrix store with I/J from Y/X (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_msij(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MSIJR — matrix store with I/J from Y/X with auto-increment row (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_msijr(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MRC+ — matrix recall column-element at (I,J) then increment J (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_mrc_plus(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MRC- — matrix recall column-element at (I,J) then decrement J (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_mrc_minus(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MRR+ — matrix recall row-element at (I,J) then increment I (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_mrr_plus(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MRR- — matrix recall row-element at (I,J) then decrement I (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_mrr_minus(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MSR+ — matrix store row-element at (I,J) then increment I (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_msr_plus(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MSC+ — matrix store column-element at (I,J) then increment J (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_msc_plus(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MSWAP — swap element (I,J) and element (I',J') between matrices (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_mswap(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MNAME? — push name of current matrix onto ALPHA register (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_mname_query(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV DIM? — push rows and cols of current matrix (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_dim_query(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MATDIM — dimension/re-dimension current matrix to R rows x C cols (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_matdim(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MP — matrix pointer: set I and J from Y and X (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_mp(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV PIV — pivot element at (I,J) by partial-pivoting (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_piv(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV R<>R — exchange rows I and Y in current matrix (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_r_exchange_r(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV R>R? — skip-if-row-index-not-equal (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_r_gt_r_query(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV SUM — compute sum of all elements in current matrix (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_sum(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV SUMAB — compute sum of absolute values of all elements (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_sumab(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MAX — find maximum element of current matrix (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_max(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MAXAB — find maximum absolute-value element (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_maxab(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV MIN — find minimum element of current matrix (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_min(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV RMAXAB — maximum absolute value in current row (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_rmaxab(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV RNRM — row-norm (maximum row-sum of absolute values) of current matrix (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_rnrm(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV RSUM — sum of current row elements (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_rsum(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV FNRM — Frobenius norm of current matrix (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_fnrm(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Catches: all ADV MTRX element-access stubs return InvalidOp (skeleton phase invariant)
    #[test]
    fn matrix_ops_index_stubs_return_invalid_op() {
        let mut state = CalcState::new();
        assert!(matches!(op_adv_i_plus(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_i_minus(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_j_plus(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_j_minus(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_mr(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_ms(&mut state), Err(HpError::InvalidOp)));
    }

    // Catches: all ADV MTRX reduction stubs return InvalidOp
    #[test]
    fn matrix_ops_reduction_stubs_return_invalid_op() {
        let mut state = CalcState::new();
        assert!(matches!(op_adv_sum(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_sumab(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_max(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_maxab(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_min(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_rmaxab(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_rnrm(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_rsum(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_fnrm(&mut state), Err(HpError::InvalidOp)));
    }

    // Catches: lifecycle stubs return InvalidOp
    #[test]
    fn matrix_ops_lifecycle_stubs_return_invalid_op() {
        let mut state = CalcState::new();
        assert!(matches!(op_adv_mname_query(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_dim_query(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_matdim(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_mp(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_piv(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_r_exchange_r(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_r_gt_r_query(&mut state), Err(HpError::InvalidOp)));
    }
}
