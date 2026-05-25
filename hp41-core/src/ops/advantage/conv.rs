// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `conv` — ADV CONV: binary/octal/hex I/O and bitwise operations.
//!
//! XROM module id 22 (ADV_MATH_A, bit-3 of `CalcState::xrom_modules`).
//!
//! Operations: BININ / BINVIEW / OCTIN / HEXIN / HEXVIEW / CVTVIEW /
//!             NOT / AND / OR / XOR / ROTXY / BIT?
//!
//! 36-bit fixed word size per D-43.9 (`ADV_WORD_MASK`).  Full implementations
//! in Plan 43-02 (BININ/BINVIEW/OCTIN/HEXIN/HEXVIEW) and Plan 43-03
//! (NOT/AND/OR/XOR/ROTXY/BIT?/CVTVIEW).

use crate::{
    error::HpError,
    state::CalcState,
};

/// ADV BININ — enter binary integer from ALPHA register into X (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_binin(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV BINVIEW — display X as 36-bit binary in ALPHA register (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_binview(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV OCTIN — enter octal integer from ALPHA register into X (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_octin(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV HEXIN — enter hexadecimal integer from ALPHA register into X (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_hexin(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV HEXVIEW — display X as hexadecimal in ALPHA register (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_hexview(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV CVTVIEW — display conversions of X (binary/octal/hex) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_cvtview(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV NOT — bitwise NOT of X using 36-bit word size (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_not(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV AND — bitwise AND of X and Y using 36-bit word size (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_and(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV OR — bitwise OR of X and Y using 36-bit word size (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_or(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV XOR — bitwise XOR of X and Y using 36-bit word size (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_xor(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV ROTXY — rotate X left/right by Y bit positions within 36-bit word (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_rotxy(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV BIT? — test bit Y of X; skip-if-clear or skip-if-set (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_bit_test(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Catches: all ADV CONV stubs return InvalidOp (skeleton phase invariant)
    #[test]
    fn conv_stubs_return_invalid_op() {
        let mut state = CalcState::new();
        assert!(matches!(op_adv_binin(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_binview(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_octin(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_hexin(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_hexview(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_cvtview(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_not(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_and(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_or(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_xor(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_rotxy(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_bit_test(&mut state), Err(HpError::InvalidOp)));
    }
}
