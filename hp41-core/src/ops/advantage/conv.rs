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
//! 36-bit fixed word size per D-43.9 (`ADV_WORD_MASK`).
//! Overflow inputs are silently truncated to 36 bits per D-43.10.

use crate::{
    error::HpError,
    num::HpNum,
    ops::advantage::ADV_WORD_MASK,
    stack::{apply_lift_effect, binary_result, enter_number, unary_result, LiftEffect},
    state::CalcState,
};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

// ── Helper ────────────────────────────────────────────────────────────────────

/// Convert `HpNum` to a `u64` masked to 36 bits.
///
/// Converts via f64 round-trip (sufficient for 36-bit integers — f64 has ~53-bit
/// integer precision, far exceeding the 36-bit ADV word size). Takes the absolute
/// value before masking (negative integers are truncated, not sign-extended).
///
/// # Errors
/// Returns `HpError::Domain` if the value is NaN or exceeds f64 integer precision
/// (> 2^53 ≈ 9.007e15). Values in range are silently masked to 36 bits (D-43.10).
fn x_to_u64_masked(x: &HpNum) -> Result<u64, HpError> {
    let f = x.inner().to_f64().ok_or(HpError::Domain)?;
    if !f.is_finite() {
        return Err(HpError::Domain);
    }
    // 2^53 is the limit for exact integer representation in f64.
    const F64_INT_LIMIT: f64 = 9_007_199_254_740_992.0; // 2^53
    if f.abs() > F64_INT_LIMIT {
        return Err(HpError::Domain);
    }
    // Round to nearest integer, take absolute value, then mask to 36 bits (D-43.10).
    let i = f.round() as i64;
    let u = i.unsigned_abs();
    Ok(u & ADV_WORD_MASK)
}

/// Convert a `u64` (already masked, ≤ 36 bits) back to `HpNum`.
///
/// Uses `Decimal::from_u64` and the `HpNum(Decimal)` constructor directly to
/// avoid the `HpNum::rounded()` path which would lose precision for values
/// with more than 10 significant decimal digits.
/// `ADV_WORD_MASK` = 2^36 - 1 = 68,719,476,735 has 11 significant decimal
/// digits — exactly representable in `Decimal` but truncated to 10 by
/// `rounded()`. Bypassing `rounded()` is safe because u64-to-Decimal is
/// always lossless.
fn u64_to_hpnum(val: u64) -> HpNum {
    HpNum(Decimal::from_u64(val).unwrap_or(Decimal::ZERO))
}

// ── Input ops (ALPHA → X) ────────────────────────────────────────────────────

/// ADV BININ — parse a binary string from ALPHA into X (ADV-CONV-01).
///
/// Reads `state.alpha_reg`, trims whitespace, parses as base-2, masks to
/// `ADV_WORD_MASK` (36 bits, D-43.10 silent truncation), writes result to X.
/// LiftEffect: Enable (number entry).
///
/// # Errors
/// Returns `HpError::Domain` if ALPHA is empty or contains characters other
/// than '0' and '1'.
pub fn op_adv_binin(state: &mut CalcState) -> Result<(), HpError> {
    let s = state.alpha_reg.trim().to_string();
    if s.is_empty() {
        return Err(HpError::Domain);
    }
    let val = u64::from_str_radix(&s, 2).map_err(|_| HpError::Domain)?;
    let masked = val & ADV_WORD_MASK;
    apply_lift_effect(state, LiftEffect::Enable);
    enter_number(state, u64_to_hpnum(masked));
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV OCTIN — parse an octal string from ALPHA into X (ADV-CONV-03).
///
/// Reads `state.alpha_reg`, trims whitespace, parses as base-8, masks to
/// `ADV_WORD_MASK`, writes result to X.  LiftEffect: Enable.
///
/// # Errors
/// Returns `HpError::Domain` if ALPHA is empty or contains non-octal characters.
pub fn op_adv_octin(state: &mut CalcState) -> Result<(), HpError> {
    let s = state.alpha_reg.trim().to_string();
    if s.is_empty() {
        return Err(HpError::Domain);
    }
    let val = u64::from_str_radix(&s, 8).map_err(|_| HpError::Domain)?;
    let masked = val & ADV_WORD_MASK;
    apply_lift_effect(state, LiftEffect::Enable);
    enter_number(state, u64_to_hpnum(masked));
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV HEXIN — parse a hexadecimal string from ALPHA into X (ADV-CONV-05).
///
/// Reads `state.alpha_reg`, trims whitespace, parses as base-16 (case-insensitive),
/// masks to `ADV_WORD_MASK`, writes result to X.  LiftEffect: Enable.
///
/// # Errors
/// Returns `HpError::Domain` if ALPHA is empty or contains non-hex characters.
pub fn op_adv_hexin(state: &mut CalcState) -> Result<(), HpError> {
    let s = state.alpha_reg.trim().to_string();
    if s.is_empty() {
        return Err(HpError::Domain);
    }
    let val = u64::from_str_radix(&s, 16).map_err(|_| HpError::Domain)?;
    let masked = val & ADV_WORD_MASK;
    apply_lift_effect(state, LiftEffect::Enable);
    enter_number(state, u64_to_hpnum(masked));
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

// ── View ops (X → ALPHA + print_buffer) ─────────────────────────────────────

/// ADV BINVIEW — display X as binary in ALPHA register and print buffer (ADV-CONV-02).
///
/// Converts X to a 36-bit-masked integer, formats as binary digits (no leading
/// zeros beyond what the value requires), writes to `state.alpha_reg` and pushes
/// to `state.print_buffer`.  LiftEffect: Neutral (display only, X unchanged).
///
/// # Errors
/// Returns `HpError::Domain` if X cannot be converted to a valid integer.
pub fn op_adv_binview(state: &mut CalcState) -> Result<(), HpError> {
    let val = x_to_u64_masked(&state.stack.x)?;
    let s = format!("{val:b}");
    state.alpha_reg = s.clone();
    state.print_buffer.push(s);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV HEXVIEW — display X as uppercase hex in ALPHA register and print buffer (ADV-CONV-06).
///
/// Converts X to a 36-bit-masked integer, formats as uppercase hexadecimal (no
/// leading zeros beyond what the value requires), writes to `state.alpha_reg` and
/// pushes to `state.print_buffer`.  LiftEffect: Neutral (display only, X unchanged).
///
/// # Errors
/// Returns `HpError::Domain` if X cannot be converted to a valid integer.
pub fn op_adv_hexview(state: &mut CalcState) -> Result<(), HpError> {
    let val = x_to_u64_masked(&state.stack.x)?;
    let s = format!("{val:X}");
    state.alpha_reg = s.clone();
    state.print_buffer.push(s);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV CVTVIEW — display X in all four bases in the print buffer (ADV-CONV-04).
///
/// Converts X to a 36-bit-masked integer and pushes four lines to `state.print_buffer`:
/// `BIN: <binary>`, `OCT: <octal>`, `DEC: <decimal>`, `HEX: <hex>`.
/// LiftEffect: Neutral (display only, X unchanged).
///
/// # Errors
/// Returns `HpError::Domain` if X cannot be converted to a valid integer.
pub fn op_adv_cvtview(state: &mut CalcState) -> Result<(), HpError> {
    let val = x_to_u64_masked(&state.stack.x)?;
    state.print_buffer.push(format!("BIN: {val:b}"));
    state.print_buffer.push(format!("OCT: {val:o}"));
    state.print_buffer.push(format!("DEC: {val}"));
    state.print_buffer.push(format!("HEX: {val:X}"));
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

// ── Bitwise unary ops ─────────────────────────────────────────────────────────

/// ADV NOT — bitwise NOT of X using 36-bit word size (ADV-CONV-07).
///
/// Computes `(x_as_u64 XOR ADV_WORD_MASK)` (complement within 36 bits) and
/// writes the result to X.  LiftEffect: Enable (result is a new value).
///
/// `NOT(0)` → `ADV_WORD_MASK` (68,719,476,735).
/// `NOT(ADV_WORD_MASK)` → 0.
///
/// # Errors
/// Returns `HpError::Domain` if X cannot be converted to a valid integer.
pub fn op_adv_not(state: &mut CalcState) -> Result<(), HpError> {
    let val = x_to_u64_masked(&state.stack.x)?;
    let result = val ^ ADV_WORD_MASK;
    unary_result(state, u64_to_hpnum(result));
    Ok(())
}

// ── Bitwise binary ops (X op Y → X, stack drops) ─────────────────────────────

/// ADV AND — bitwise AND of X and Y using 36-bit word size (ADV-CONV-08).
///
/// Pops X and Y (Y is consumed, stack drops), computes `y & x`, writes result
/// to X.  LiftEffect: Enable (result via `binary_result`).
///
/// # Errors
/// Returns `HpError::Domain` if X or Y cannot be converted to valid integers.
pub fn op_adv_and(state: &mut CalcState) -> Result<(), HpError> {
    let x_val = x_to_u64_masked(&state.stack.x)?;
    let y_val = x_to_u64_masked(&state.stack.y)?;
    let result = x_val & y_val;
    binary_result(state, u64_to_hpnum(result));
    Ok(())
}

/// ADV OR — bitwise OR of X and Y using 36-bit word size (ADV-CONV-09).
///
/// Pops X and Y (Y is consumed, stack drops), computes `y | x`, writes result
/// to X.  LiftEffect: Enable (result via `binary_result`).
///
/// # Errors
/// Returns `HpError::Domain` if X or Y cannot be converted to valid integers.
pub fn op_adv_or(state: &mut CalcState) -> Result<(), HpError> {
    let x_val = x_to_u64_masked(&state.stack.x)?;
    let y_val = x_to_u64_masked(&state.stack.y)?;
    let result = x_val | y_val;
    binary_result(state, u64_to_hpnum(result));
    Ok(())
}

/// ADV XOR — bitwise XOR of X and Y using 36-bit word size (ADV-CONV-10).
///
/// Pops X and Y (Y is consumed, stack drops), computes `y ^ x`, writes result
/// to X.  LiftEffect: Enable (result via `binary_result`).
///
/// # Errors
/// Returns `HpError::Domain` if X or Y cannot be converted to valid integers.
pub fn op_adv_xor(state: &mut CalcState) -> Result<(), HpError> {
    let x_val = x_to_u64_masked(&state.stack.x)?;
    let y_val = x_to_u64_masked(&state.stack.y)?;
    let result = x_val ^ y_val;
    binary_result(state, u64_to_hpnum(result));
    Ok(())
}

// ── Rotation op ───────────────────────────────────────────────────────────────

/// ADV ROTXY — circular rotation of Y by X bits within the 36-bit word (ADV-CONV-11).
///
/// X specifies the rotation count (positive = left, negative = right).
/// Y is the value to rotate. Y is consumed (stack drops), result in X.
/// The shift amount is taken modulo 36 (no over-rotation). LiftEffect: Enable.
///
/// Examples:
/// - Y=1, X=1  → rotate left 1  → result = 2
/// - Y=1, X=-1 → rotate right 1 → result = 2^35 (bit 35 set)
/// - Y=1, X=36 → rotate left 36 → result = 1 (full cycle)
///
/// # Errors
/// Returns `HpError::Domain` if X or Y cannot be converted to valid integers.
pub fn op_adv_rotxy(state: &mut CalcState) -> Result<(), HpError> {
    // X = shift count (may be negative), Y = value to rotate.
    let x_f = state.stack.x.inner().to_f64().ok_or(HpError::Domain)?;
    if !x_f.is_finite() {
        return Err(HpError::Domain);
    }
    let shift_raw = x_f.round() as i64;
    let val = x_to_u64_masked(&state.stack.y)?;

    // Reduce shift to [0, 36) range; handle negative by converting to equivalent left shift.
    const WORD_BITS: u32 = 36;
    // rem_euclid ensures positive remainder even for negative shift_raw.
    let shift = shift_raw.rem_euclid(WORD_BITS as i64) as u32;

    let result = if shift == 0 {
        val
    } else {
        // Circular left rotation within 36-bit word.
        ((val << shift) | (val >> (WORD_BITS - shift))) & ADV_WORD_MASK
    };

    binary_result(state, u64_to_hpnum(result));
    Ok(())
}

// ── Bit test op ───────────────────────────────────────────────────────────────

/// ADV BIT? — test a specific bit of a value (ADV-CONV-12).
///
/// Y specifies the bit number (0 = LSB, 35 = MSB). X is the value to test.
/// X and Y are consumed (stack drops). Result in X: `1.0` if bit is set,
/// `0.0` if bit is clear. LiftEffect: Enable (result via `binary_result`).
///
/// The "skip-if-set" semantics follow the same conditional-test pattern as
/// `TestKind::XNeZero` — programmers test `X ≠ 0` after BIT? to skip when
/// the bit was set.
///
/// # Errors
/// Returns `HpError::Domain` if X or Y cannot be converted to valid integers,
/// or if Y specifies a bit number outside [0, 35].
pub fn op_adv_bit_test(state: &mut CalcState) -> Result<(), HpError> {
    // Y = bit number (0-35), X = value to test.
    let y_f = state.stack.y.inner().to_f64().ok_or(HpError::Domain)?;
    if !y_f.is_finite() {
        return Err(HpError::Domain);
    }
    let bit_num = y_f.round() as i64;
    if !(0..=35).contains(&bit_num) {
        return Err(HpError::Domain);
    }
    let val = x_to_u64_masked(&state.stack.x)?;
    let is_set = (val >> (bit_num as u32)) & 1 == 1;
    let result = if is_set {
        HpNum::from(1i32)
    } else {
        HpNum::zero()
    };
    binary_result(state, result);
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Helper: set X register via f64 (for values ≤ 10^9, within 10-digit HpNum precision).
    fn set_x(state: &mut CalcState, val: f64) {
        state.stack.x = HpNum::from(
            rust_decimal::Decimal::from_f64(val).unwrap_or(rust_decimal::Decimal::ZERO),
        );
    }

    // Helper: set X register exactly from u64 (bypasses rounded(), safe for all u64).
    fn set_x_u64(state: &mut CalcState, val: u64) {
        state.stack.x =
            HpNum(rust_decimal::Decimal::from_u64(val).unwrap_or(rust_decimal::Decimal::ZERO));
    }

    // Helper: set Y register via f64 (for values ≤ 10^9, within 10-digit HpNum precision).
    fn set_y(state: &mut CalcState, val: f64) {
        state.stack.y = HpNum::from(
            rust_decimal::Decimal::from_f64(val).unwrap_or(rust_decimal::Decimal::ZERO),
        );
    }

    // Helper: read X as f64.
    fn get_x_f64(state: &CalcState) -> f64 {
        state.stack.x.inner().to_f64().unwrap_or(f64::NAN)
    }

    // Helper: read X as u64 (for 36-bit integer results).
    fn get_x_u64(state: &CalcState) -> u64 {
        state.stack.x.inner().to_u64().unwrap_or(0)
    }

    // ── ADV_WORD_MASK ─────────────────────────────────────────────────────────

    // Catches: ADV_WORD_MASK must have exactly 36 set bits (D-43.9)
    #[test]
    fn adv_word_mask_count_ones() {
        assert_eq!(ADV_WORD_MASK.count_ones(), 36);
    }

    // ── BININ ─────────────────────────────────────────────────────────────────

    // Catches: BININ parses binary string correctly
    #[test]
    fn adv_binin_basic() {
        let mut state = CalcState::new();
        state.alpha_reg = "1010".to_string();
        op_adv_binin(&mut state).unwrap();
        assert_eq!(get_x_f64(&state), 10.0, "BININ(\"1010\") must yield 10");
    }

    // Catches: BININ empty ALPHA → Domain error
    #[test]
    fn adv_binin_empty_alpha_domain_error() {
        let mut state = CalcState::new();
        state.alpha_reg = String::new();
        assert!(matches!(op_adv_binin(&mut state), Err(HpError::Domain)));
    }

    // Catches: BININ with invalid binary digit → Domain error
    #[test]
    fn adv_binin_invalid_digit_domain_error() {
        let mut state = CalcState::new();
        state.alpha_reg = "102".to_string();
        assert!(matches!(op_adv_binin(&mut state), Err(HpError::Domain)));
    }

    // Catches: BININ overflow → silent mask to 36 bits (D-43.10)
    #[test]
    fn adv_binin_overflow_truncated_to_36_bits() {
        let mut state = CalcState::new();
        // 37-bit string: 1 followed by 36 zeros = 2^36, which exceeds ADV_WORD_MASK.
        // After masking: 2^36 & (2^36 - 1) = 0.
        let s = "1".to_string() + &"0".repeat(36);
        state.alpha_reg = s;
        op_adv_binin(&mut state).unwrap();
        assert_eq!(
            get_x_f64(&state),
            0.0,
            "37-bit string must be masked to 36 bits"
        );
    }

    // Catches: BININ with whitespace trims correctly
    #[test]
    fn adv_binin_trims_whitespace() {
        let mut state = CalcState::new();
        state.alpha_reg = "  1111  ".to_string();
        op_adv_binin(&mut state).unwrap();
        assert_eq!(get_x_f64(&state), 15.0);
    }

    // ── OCTIN ─────────────────────────────────────────────────────────────────

    // Catches: OCTIN parses octal string correctly
    #[test]
    fn adv_octin_basic() {
        let mut state = CalcState::new();
        state.alpha_reg = "77".to_string();
        op_adv_octin(&mut state).unwrap();
        assert_eq!(get_x_f64(&state), 63.0, "OCTIN(\"77\") must yield 63");
    }

    // Catches: OCTIN empty ALPHA → Domain error
    #[test]
    fn adv_octin_empty_alpha_domain_error() {
        let mut state = CalcState::new();
        state.alpha_reg = String::new();
        assert!(matches!(op_adv_octin(&mut state), Err(HpError::Domain)));
    }

    // Catches: OCTIN with invalid octal digit → Domain error
    #[test]
    fn adv_octin_invalid_digit_domain_error() {
        let mut state = CalcState::new();
        state.alpha_reg = "89".to_string();
        assert!(matches!(op_adv_octin(&mut state), Err(HpError::Domain)));
    }

    // ── HEXIN ─────────────────────────────────────────────────────────────────

    // Catches: HEXIN parses hex string correctly (uppercase)
    #[test]
    fn adv_hexin_uppercase() {
        let mut state = CalcState::new();
        state.alpha_reg = "FF".to_string();
        op_adv_hexin(&mut state).unwrap();
        assert_eq!(get_x_f64(&state), 255.0, "HEXIN(\"FF\") must yield 255");
    }

    // Catches: HEXIN parses hex string correctly (lowercase)
    #[test]
    fn adv_hexin_lowercase() {
        let mut state = CalcState::new();
        state.alpha_reg = "ff".to_string();
        op_adv_hexin(&mut state).unwrap();
        assert_eq!(get_x_f64(&state), 255.0);
    }

    // Catches: HEXIN empty ALPHA → Domain error
    #[test]
    fn adv_hexin_empty_alpha_domain_error() {
        let mut state = CalcState::new();
        state.alpha_reg = String::new();
        assert!(matches!(op_adv_hexin(&mut state), Err(HpError::Domain)));
    }

    // Catches: HEXIN with invalid hex character → Domain error
    #[test]
    fn adv_hexin_invalid_digit_domain_error() {
        let mut state = CalcState::new();
        state.alpha_reg = "GG".to_string();
        assert!(matches!(op_adv_hexin(&mut state), Err(HpError::Domain)));
    }

    // ── BINVIEW ───────────────────────────────────────────────────────────────

    // Catches: BINVIEW displays 10 as binary "1010"
    #[test]
    fn adv_binview_basic() {
        let mut state = CalcState::new();
        set_x(&mut state, 10.0);
        op_adv_binview(&mut state).unwrap();
        assert_eq!(state.alpha_reg, "1010");
        assert!(state.print_buffer.contains(&"1010".to_string()));
    }

    // Catches: BINVIEW for 0 displays "0"
    #[test]
    fn adv_binview_zero() {
        let mut state = CalcState::new();
        set_x(&mut state, 0.0);
        op_adv_binview(&mut state).unwrap();
        assert_eq!(state.alpha_reg, "0");
    }

    // Catches: BINVIEW does not modify X (Neutral lift effect)
    #[test]
    fn adv_binview_x_unchanged() {
        let mut state = CalcState::new();
        set_x(&mut state, 10.0);
        op_adv_binview(&mut state).unwrap();
        assert_eq!(get_x_f64(&state), 10.0, "BINVIEW must not modify X");
    }

    // ── HEXVIEW ───────────────────────────────────────────────────────────────

    // Catches: HEXVIEW displays 255 as "FF"
    #[test]
    fn adv_hexview_basic() {
        let mut state = CalcState::new();
        set_x(&mut state, 255.0);
        op_adv_hexview(&mut state).unwrap();
        assert_eq!(state.alpha_reg, "FF");
        assert!(state.print_buffer.contains(&"FF".to_string()));
    }

    // Catches: HEXVIEW for 0 displays "0"
    #[test]
    fn adv_hexview_zero() {
        let mut state = CalcState::new();
        set_x(&mut state, 0.0);
        op_adv_hexview(&mut state).unwrap();
        assert_eq!(state.alpha_reg, "0");
    }

    // Catches: HEXVIEW does not modify X (Neutral lift effect)
    #[test]
    fn adv_hexview_x_unchanged() {
        let mut state = CalcState::new();
        set_x(&mut state, 255.0);
        op_adv_hexview(&mut state).unwrap();
        assert_eq!(get_x_f64(&state), 255.0, "HEXVIEW must not modify X");
    }

    // ── CVTVIEW ───────────────────────────────────────────────────────────────

    // Catches: CVTVIEW pushes 4 lines for X=10
    #[test]
    fn adv_cvtview_basic() {
        let mut state = CalcState::new();
        set_x(&mut state, 10.0);
        op_adv_cvtview(&mut state).unwrap();
        assert!(
            state.print_buffer.iter().any(|l| l.contains("1010")),
            "BIN line must contain 1010"
        );
        assert!(
            state.print_buffer.iter().any(|l| l.contains("12")),
            "OCT line must contain 12"
        );
        assert!(
            state.print_buffer.iter().any(|l| l.contains("10")),
            "DEC line must contain 10"
        );
        assert!(
            state.print_buffer.iter().any(|l| l.contains('A')),
            "HEX line must contain A"
        );
    }

    // Catches: CVTVIEW pushes exactly 4 lines
    #[test]
    fn adv_cvtview_four_lines() {
        let mut state = CalcState::new();
        set_x(&mut state, 255.0);
        op_adv_cvtview(&mut state).unwrap();
        assert_eq!(state.print_buffer.len(), 4);
    }

    // ── NOT ───────────────────────────────────────────────────────────────────

    // Catches: NOT(0) == ADV_WORD_MASK (2^36 - 1 = 68,719,476,735 — 11 digits, needs u64 helpers)
    #[test]
    fn adv_not_zero_gives_word_mask() {
        let mut state = CalcState::new();
        set_x(&mut state, 0.0);
        op_adv_not(&mut state).unwrap();
        assert_eq!(
            get_x_u64(&state),
            ADV_WORD_MASK,
            "NOT(0) must equal ADV_WORD_MASK"
        );
    }

    // Catches: NOT(ADV_WORD_MASK) == 0 (use u64 helper to set 11-digit value precisely)
    #[test]
    fn adv_not_word_mask_gives_zero() {
        let mut state = CalcState::new();
        set_x_u64(&mut state, ADV_WORD_MASK);
        op_adv_not(&mut state).unwrap();
        assert_eq!(get_x_u64(&state), 0, "NOT(ADV_WORD_MASK) must equal 0");
    }

    // Catches: NOT is its own inverse (for values fitting in 10 digits, f64 helpers are fine)
    #[test]
    fn adv_not_double_negation() {
        let mut state = CalcState::new();
        set_x(&mut state, 42.0);
        op_adv_not(&mut state).unwrap();
        op_adv_not(&mut state).unwrap();
        assert_eq!(
            get_x_f64(&state),
            42.0,
            "double NOT must restore original value"
        );
    }

    // ── AND ───────────────────────────────────────────────────────────────────

    // Catches: AND(0xFF, 0x0F) == 0x0F, Y consumed
    #[test]
    fn adv_and_basic() {
        let mut state = CalcState::new();
        set_x(&mut state, 0xFF_u64 as f64); // X = 255
        set_y(&mut state, 0x0F_u64 as f64); // Y = 15
        let y_before = state.stack.z.inner(); // track stack drop
        op_adv_and(&mut state).unwrap();
        assert_eq!(
            get_x_f64(&state),
            15.0,
            "AND(0xFF, 0x0F) must yield 0x0F = 15"
        );
        // Verify stack dropped: new Y = old Z
        assert_eq!(
            state.stack.y.inner(),
            y_before,
            "AND must drop Y (stack drop)"
        );
    }

    // Catches: AND with 0 yields 0
    #[test]
    fn adv_and_with_zero() {
        let mut state = CalcState::new();
        set_x(&mut state, 0.0);
        set_y(&mut state, 255.0);
        op_adv_and(&mut state).unwrap();
        assert_eq!(get_x_f64(&state), 0.0);
    }

    // ── OR ────────────────────────────────────────────────────────────────────

    // Catches: OR(0xF0, 0x0F) == 0xFF
    #[test]
    fn adv_or_basic() {
        let mut state = CalcState::new();
        set_x(&mut state, 0xF0_u64 as f64); // X = 240
        set_y(&mut state, 0x0F_u64 as f64); // Y = 15
        op_adv_or(&mut state).unwrap();
        assert_eq!(
            get_x_f64(&state),
            0xFF_u64 as f64,
            "OR(0xF0, 0x0F) must yield 0xFF = 255"
        );
    }

    // Catches: OR with 0 is identity
    #[test]
    fn adv_or_with_zero_is_identity() {
        let mut state = CalcState::new();
        set_x(&mut state, 0.0);
        set_y(&mut state, 42.0);
        op_adv_or(&mut state).unwrap();
        assert_eq!(get_x_f64(&state), 42.0);
    }

    // ── XOR ───────────────────────────────────────────────────────────────────

    // Catches: XOR(0xFF, 0x0F) == 0xF0
    #[test]
    fn adv_xor_basic() {
        let mut state = CalcState::new();
        set_x(&mut state, 0xFF_u64 as f64); // X = 255
        set_y(&mut state, 0x0F_u64 as f64); // Y = 15
        op_adv_xor(&mut state).unwrap();
        assert_eq!(
            get_x_f64(&state),
            0xF0_u64 as f64,
            "XOR(0xFF, 0x0F) must yield 0xF0 = 240"
        );
    }

    // Catches: XOR with itself yields 0
    #[test]
    fn adv_xor_self_gives_zero() {
        let mut state = CalcState::new();
        set_x(&mut state, 42.0);
        set_y(&mut state, 42.0);
        op_adv_xor(&mut state).unwrap();
        assert_eq!(get_x_f64(&state), 0.0, "XOR(n, n) must yield 0");
    }

    // ── ROTXY ─────────────────────────────────────────────────────────────────

    // Catches: ROTXY rotate left by 1 (X=1, Y=1 → 2)
    #[test]
    fn adv_rotxy_left_by_one() {
        let mut state = CalcState::new();
        set_x(&mut state, 1.0); // shift = 1 (left)
        set_y(&mut state, 1.0); // value = 1
        op_adv_rotxy(&mut state).unwrap();
        assert_eq!(get_x_f64(&state), 2.0, "rotate left 1: 1 → 2");
    }

    // Catches: ROTXY rotate right by 1 (X=-1, Y=1 → 2^35 = 34,359,738,368 — 11 digits, use u64)
    #[test]
    fn adv_rotxy_right_by_one() {
        let mut state = CalcState::new();
        set_x(&mut state, -1.0); // shift = -1 (right by 1 = left by 35 within 36-bit word)
        set_y(&mut state, 1.0); // value = 1
        op_adv_rotxy(&mut state).unwrap();
        let expected = 1u64 << 35; // 2^35 = 34,359,738,368
        assert_eq!(
            get_x_u64(&state),
            expected,
            "rotate right 1: bit 0 wraps to bit 35"
        );
    }

    // Catches: ROTXY full cycle (shift 36) is identity
    #[test]
    fn adv_rotxy_full_cycle_is_identity() {
        let mut state = CalcState::new();
        set_x(&mut state, 36.0); // shift = 36 = full cycle
        set_y(&mut state, 170.0); // value = 0b1010_1010 = 170
        op_adv_rotxy(&mut state).unwrap();
        assert_eq!(get_x_f64(&state), 170.0, "rotate by 36 bits = identity");
    }

    // Catches: ROTXY shift 0 is identity
    #[test]
    fn adv_rotxy_shift_zero_is_identity() {
        let mut state = CalcState::new();
        set_x(&mut state, 0.0); // shift = 0
        set_y(&mut state, 7.0); // value = 7
        op_adv_rotxy(&mut state).unwrap();
        assert_eq!(get_x_f64(&state), 7.0, "rotate by 0 bits = identity");
    }

    // Catches: ROTXY multiple left shifts accumulate
    #[test]
    fn adv_rotxy_left_by_four() {
        let mut state = CalcState::new();
        set_x(&mut state, 4.0); // shift = 4 left
        set_y(&mut state, 1.0); // value = 1
        op_adv_rotxy(&mut state).unwrap();
        assert_eq!(get_x_f64(&state), 16.0, "1 << 4 = 16");
    }

    // ── BIT? ──────────────────────────────────────────────────────────────────

    // Catches: BIT?(X=5, Y=0) → 1 (bit 0 of 5 = 101 is set)
    #[test]
    fn adv_bit_test_bit_set_returns_one() {
        let mut state = CalcState::new();
        set_x(&mut state, 5.0); // X = 5 = 0b101
        set_y(&mut state, 0.0); // Y = bit 0
        op_adv_bit_test(&mut state).unwrap();
        assert_eq!(
            get_x_f64(&state),
            1.0,
            "bit 0 of 5 (0b101) is set → result 1"
        );
    }

    // Catches: BIT?(X=5, Y=1) → 0 (bit 1 of 5 = 101 is clear)
    #[test]
    fn adv_bit_test_bit_clear_returns_zero() {
        let mut state = CalcState::new();
        set_x(&mut state, 5.0); // X = 5 = 0b101
        set_y(&mut state, 1.0); // Y = bit 1
        op_adv_bit_test(&mut state).unwrap();
        assert_eq!(
            get_x_f64(&state),
            0.0,
            "bit 1 of 5 (0b101) is clear → result 0"
        );
    }

    // Catches: BIT? bit 35 (MSB of 36-bit word — 2^35 = 34,359,738,368 = 11 digits, use u64 helper)
    #[test]
    fn adv_bit_test_msb() {
        let mut state = CalcState::new();
        let msb_val = 1u64 << 35; // 34,359,738,368 — 11 decimal digits
        set_x_u64(&mut state, msb_val);
        set_y(&mut state, 35.0); // Y = bit 35
        op_adv_bit_test(&mut state).unwrap();
        assert_eq!(get_x_f64(&state), 1.0, "MSB (bit 35) must be detectable");
    }

    // Catches: BIT? bit number out of range [0, 35] → Domain error
    #[test]
    fn adv_bit_test_out_of_range_domain_error() {
        let mut state = CalcState::new();
        set_x(&mut state, 1.0);
        set_y(&mut state, 36.0); // bit 36 is out of 36-bit range
        assert!(matches!(op_adv_bit_test(&mut state), Err(HpError::Domain)));
    }

    // Catches: BIT? with negative bit number → Domain error
    #[test]
    fn adv_bit_test_negative_bit_domain_error() {
        let mut state = CalcState::new();
        set_x(&mut state, 1.0);
        set_y(&mut state, -1.0);
        assert!(matches!(op_adv_bit_test(&mut state), Err(HpError::Domain)));
    }

    // Catches: BIT? on 0 value, bit 0 → 0 (clear)
    #[test]
    fn adv_bit_test_zero_value_all_clear() {
        let mut state = CalcState::new();
        set_x(&mut state, 0.0);
        set_y(&mut state, 0.0);
        op_adv_bit_test(&mut state).unwrap();
        assert_eq!(get_x_f64(&state), 0.0, "all bits of 0 are clear");
    }

    // Catches: BIT? stack drop — Y is consumed
    #[test]
    fn adv_bit_test_stack_drops() {
        let mut state = CalcState::new();
        state.stack.z = HpNum::from(99i32);
        set_x(&mut state, 5.0);
        set_y(&mut state, 0.0);
        op_adv_bit_test(&mut state).unwrap();
        // After binary_result: new Y = old Z = 99
        assert_eq!(
            state.stack.y.inner().to_f64().unwrap(),
            99.0,
            "BIT? must drop Y (stack drop)"
        );
    }

    // Catches: x_to_u64_masked handles max 36-bit value (ADV_WORD_MASK = 2^36-1 = 11 digits).
    // Uses the exact constructor (HpNum(Decimal)) to bypass HpNum::rounded() truncation.
    #[test]
    fn x_to_u64_masked_max_36_bit_value() {
        // Direct constructor bypasses rounded() — same path as u64_to_hpnum().
        let n = HpNum(rust_decimal::Decimal::from_u64(ADV_WORD_MASK).unwrap());
        let result = x_to_u64_masked(&n).unwrap();
        assert_eq!(result, ADV_WORD_MASK);
    }
}
