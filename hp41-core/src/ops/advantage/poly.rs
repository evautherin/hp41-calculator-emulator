// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `poly` — ADV MATH: PLY (polynomial evaluation) + RTS (root output).
//!
//! XROM module id 24 (ADV_MATH_B, bit-4 of `CalcState::xrom_modules`).
//!
//! Operations: PLY / RTS
//!
//! PLY evaluates a polynomial whose coefficients are stored in the named
//! current matrix, using Horner's method.  RTS (roots) outputs the roots
//! computed by a prior PLY run.
//!
//! Full implementations in Plan 43-07 (polynomial root finding).

use crate::{
    error::HpError,
    state::CalcState,
};

/// ADV PLY — evaluate polynomial at X using Horner's method; coefficients from
/// current named matrix (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_ply(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV RTS — recall roots computed by PLY onto the stack (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_rts(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Catches: poly stubs return InvalidOp (skeleton phase invariant)
    #[test]
    fn poly_stubs_return_invalid_op() {
        let mut state = CalcState::new();
        assert!(matches!(op_adv_ply(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_rts(&mut state), Err(HpError::InvalidOp)));
    }
}
