// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `curve_fit` — ADV MATH: curve fitting operations.
//!
//! XROM module id 24 (ADV_MATH_B, bit-4 of `CalcState::xrom_modules`).
//!
//! Operations: CFIT / AS / DS / BFIT / FIT / Y?X / SZ?
//!
//! Full implementations in Plan 43-09 (curve fitting + statistical fitting).

use crate::{
    error::HpError,
    state::CalcState,
};

/// ADV CFIT — clear curve-fit statistical accumulators (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_cfit(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV AS — add a (X,Y) data point to curve-fit accumulators (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_as(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV DS — remove a (X,Y) data point from curve-fit accumulators (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_ds(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV BFIT — choose the best-fitting model from linear/log/exp/power (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_bfit(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV FIT — compute curve-fit parameters (slope, intercept, correlation) for
/// the selected model (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_fit(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV Y?X — estimate Y from X using the current curve-fit model (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_y_query_x(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV SZ? — push the current sample size (n) onto the stack (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_sz_query(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Catches: all ADV MATH curve-fit stubs return InvalidOp (skeleton phase invariant)
    #[test]
    fn curve_fit_stubs_return_invalid_op() {
        let mut state = CalcState::new();
        assert!(matches!(op_adv_cfit(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_as(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_ds(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_bfit(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_fit(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_y_query_x(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_sz_query(&mut state), Err(HpError::InvalidOp)));
    }
}
