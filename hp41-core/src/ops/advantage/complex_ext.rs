// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `complex_ext` — ADV MATH complex extensions: transcendental functions + arithmetic.
//!
//! XROM module id 24 (ADV_MATH_B, bit-4 of `CalcState::xrom_modules`).
//!
//! Operations: e^Z / LNZ / Z^N / Z^(1/N) / Z^W / Z^(1/W) / |Z| /
//!             SIN Z / COS Z / TAN Z / A^Z / ADV C+ / ADV C- / ADV CINV / ADV C* / ADV C/ /
//!             AIP (Alpha Integer Print — converts integer in ALPHA to real)
//!
//! Full implementations in Plan 43-07 (complex math).
//!
//! Note: `AIP` is listed here (in `complex_ext`) because it is dispatched
//! by the ADV MATH module (XROM 24) in the OM.  The actual op function is
//! re-exported in `mod.rs` alongside complex_ext.

use crate::{
    error::HpError,
    state::CalcState,
};

/// ADV e^Z — complex exponential e^(X+iY) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_exp_z(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV LNZ — complex natural logarithm ln(X+iY) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_ln_z(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV LOG Z — complex base-10 logarithm log10(X+iY) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_log_z(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV Z^N — raise complex Z=(X+iY) to integer power N from Z-register (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_z_pow_n(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV Z^(1/N) — nth complex root of Z=(X+iY) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_z_pow_1n(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV Z^W — complex power Z^W where W=(T+iZ), Z=(Y+iX) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_z_pow_w(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV Z^(1/W) — complex root Z^(1/W) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_z_pow_1w(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV |Z| — complex modulus |X+iY| = sqrt(X^2 + Y^2) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_magz(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV SIN Z — complex sine sin(X+iY) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_sin_z(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV COS Z — complex cosine cos(X+iY) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_cos_z(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV TAN Z — complex tangent tan(X+iY) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_tan_z(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV A^Z — complex power a^Z where a=real(T-register), Z=(Y+iX) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_a_pow_z(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV C+ — complex addition (Y+iX) + (T+iZ) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_c_plus(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV C- — complex subtraction (T+iZ) - (Y+iX) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_c_minus(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV CINV — complex reciprocal 1/(X+iY) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_cinv(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV C* — complex multiplication (Y+iX) * (T+iZ) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_c_mul(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV C/ — complex division (T+iZ) / (Y+iX) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_c_div(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV AIP — convert integer string in ALPHA register to real number in X (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_aip(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Catches: all ADV MATH complex extension stubs return InvalidOp (skeleton phase invariant)
    #[test]
    fn complex_ext_transcendental_stubs_return_invalid_op() {
        let mut state = CalcState::new();
        assert!(matches!(op_adv_exp_z(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_ln_z(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_log_z(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_z_pow_n(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_z_pow_1n(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_z_pow_w(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_z_pow_1w(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_magz(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_sin_z(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_cos_z(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_tan_z(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_a_pow_z(&mut state), Err(HpError::InvalidOp)));
    }

    // Catches: arithmetic complex extension stubs return InvalidOp
    #[test]
    fn complex_ext_arithmetic_stubs_return_invalid_op() {
        let mut state = CalcState::new();
        assert!(matches!(op_adv_c_plus(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_c_minus(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_cinv(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_c_mul(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_c_div(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_aip(&mut state), Err(HpError::InvalidOp)));
    }
}
