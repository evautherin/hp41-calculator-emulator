// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `complex_ext` — ADV MATH complex extensions: transcendental functions + arithmetic.
//!
//! XROM module id 24 (ADV_MATH_B, bit-4 of `CalcState::xrom_modules`).
//!
//! Operations: e^Z / LNZ / Z^N / Z^(1/N) / Z^W / Z^(1/W) / |Z| /
//!             SIN Z / COS Z / TAN Z / A^Z / ADV C+ / ADV C- / ADV CINV / ADV C* / ADV C/ /
//!             AIP (Alpha Integer Print — converts integer in X to ASCII char in ALPHA)
//!
//! ## Delegation Design
//!
//! Operations that are functionally identical to Math Pac I delegate directly to
//! `crate::ops::math1::complex` functions. The Advantage Pac variants are separate
//! `Op` enum entries but share implementations.
//!
//! ## Z^(1/W) — New in Advantage Pac
//!
//! Z^(1/W) computes z^(1/w) = exp(ln(z) / w) using the complex exponential and
//! natural logarithm (via the f64 bridge). Not present in Math Pac I.
//!
//! ## AIP — Alpha Integer Print
//!
//! AIP reads X (truncated to integer), converts to a UTF-8 char via its codepoint,
//! and appends to `state.alpha_reg`. Emulator extension: HP-41 AIP works on 7-bit
//! ASCII only; we accept the full Unicode scalar range. LiftEffect::Neutral.

use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

use crate::{
    error::HpError,
    num::HpNum,
    ops::math1::complex::{
        op_a_pow_z, op_c_div, op_c_minus, op_c_plus, op_c_times, op_cinv, op_cos_z, op_exp_z,
        op_ln_z, op_log_z, op_magz, op_sin_z, op_tan_z, op_z_pow_1_n, op_z_pow_n, op_z_pow_w,
    },
    stack::{apply_lift_effect, LiftEffect},
    state::CalcState,
};

// ── Delegating wrappers (identical to Math Pac I) ────────────────────────────

/// ADV e^Z — complex exponential e^(X+iY).
///
/// Delegates to `crate::ops::math1::complex::op_exp_z`.
///
/// # Errors
/// Propagates any error from the delegate.
pub fn op_adv_exp_z(state: &mut CalcState) -> Result<(), HpError> {
    op_exp_z(state)
}

/// ADV LNZ — complex natural logarithm ln(X+iY).
///
/// Delegates to `crate::ops::math1::complex::op_ln_z`.
/// Guard: (0+0i) → HpError::Domain.
///
/// # Errors
/// Propagates any error from the delegate.
pub fn op_adv_ln_z(state: &mut CalcState) -> Result<(), HpError> {
    op_ln_z(state)
}

/// ADV LOG Z — complex base-10 logarithm log10(X+iY).
///
/// Delegates to `crate::ops::math1::complex::op_log_z`.
/// Guard: (0+0i) → HpError::Domain.
///
/// # Errors
/// Propagates any error from the delegate.
pub fn op_adv_log_z(state: &mut CalcState) -> Result<(), HpError> {
    op_log_z(state)
}

/// ADV Z^N — raise complex Z=(Y+iZ) to integer power N from X-register.
///
/// Delegates to `crate::ops::math1::complex::op_z_pow_n`.
///
/// # Errors
/// Propagates any error from the delegate.
pub fn op_adv_z_pow_n(state: &mut CalcState) -> Result<(), HpError> {
    op_z_pow_n(state)
}

/// ADV Z^(1/N) — Nth complex root of Z=(Y+iZ), N from X.
///
/// Delegates to `crate::ops::math1::complex::op_z_pow_1_n`.
///
/// # Errors
/// Propagates any error from the delegate.
pub fn op_adv_z_pow_1n(state: &mut CalcState) -> Result<(), HpError> {
    op_z_pow_1_n(state)
}

/// ADV Z^W — complex power Z^W where Z=(X+iY) (base), W=(Z+iT) (exponent).
///
/// Delegates to `crate::ops::math1::complex::op_z_pow_w`.
///
/// # Errors
/// Propagates any error from the delegate.
pub fn op_adv_z_pow_w(state: &mut CalcState) -> Result<(), HpError> {
    op_z_pow_w(state)
}

/// ADV |Z| — complex modulus |X+iY| = sqrt(X^2 + Y^2).
///
/// Delegates to `crate::ops::math1::complex::op_magz`.
///
/// # Errors
/// Propagates any error from the delegate.
pub fn op_adv_magz(state: &mut CalcState) -> Result<(), HpError> {
    op_magz(state)
}

/// ADV SIN Z — complex sine sin(X+iY).
///
/// Delegates to `crate::ops::math1::complex::op_sin_z`.
///
/// # Errors
/// Propagates any error from the delegate.
pub fn op_adv_sin_z(state: &mut CalcState) -> Result<(), HpError> {
    op_sin_z(state)
}

/// ADV COS Z — complex cosine cos(X+iY).
///
/// Delegates to `crate::ops::math1::complex::op_cos_z`.
///
/// # Errors
/// Propagates any error from the delegate.
pub fn op_adv_cos_z(state: &mut CalcState) -> Result<(), HpError> {
    op_cos_z(state)
}

/// ADV TAN Z — complex tangent tan(X+iY).
///
/// Delegates to `crate::ops::math1::complex::op_tan_z`.
///
/// # Errors
/// Propagates any error from the delegate.
pub fn op_adv_tan_z(state: &mut CalcState) -> Result<(), HpError> {
    op_tan_z(state)
}

/// ADV A^Z — complex power a^Z where a=(Z+iT), Z=(X+iY).
///
/// Delegates to `crate::ops::math1::complex::op_a_pow_z`.
///
/// # Errors
/// Propagates any error from the delegate.
pub fn op_adv_a_pow_z(state: &mut CalcState) -> Result<(), HpError> {
    op_a_pow_z(state)
}

// ── Advantage-only complex arithmetic (same algorithm as Math Pac I) ─────────

/// ADV C+ — complex addition (X+iY) + (Z+iT): result in X+iY.
///
/// Delegates to `crate::ops::math1::complex::op_c_plus`.
///
/// # Errors
/// Propagates any error from the delegate.
pub fn op_adv_c_plus(state: &mut CalcState) -> Result<(), HpError> {
    op_c_plus(state)
}

/// ADV C- — complex subtraction (X+iY) - (Z+iT): result in X+iY.
///
/// Delegates to `crate::ops::math1::complex::op_c_minus`.
///
/// # Errors
/// Propagates any error from the delegate.
pub fn op_adv_c_minus(state: &mut CalcState) -> Result<(), HpError> {
    op_c_minus(state)
}

/// ADV CINV — complex reciprocal 1/(X+iY).
///
/// Delegates to `crate::ops::math1::complex::op_cinv`.
/// Guard: (0+0i) → HpError::DivideByZero.
///
/// # Errors
/// Propagates any error from the delegate.
pub fn op_adv_cinv(state: &mut CalcState) -> Result<(), HpError> {
    op_cinv(state)
}

/// ADV C* — complex multiplication (X+iY) * (Z+iT): result in X+iY.
///
/// Delegates to `crate::ops::math1::complex::op_c_times`.
///
/// # Errors
/// Propagates any error from the delegate.
pub fn op_adv_c_mul(state: &mut CalcState) -> Result<(), HpError> {
    op_c_times(state)
}

/// ADV C/ — complex division (X+iY) / (Z+iT): result in X+iY.
///
/// Delegates to `crate::ops::math1::complex::op_c_div`.
/// Guard: (0+0i) divisor → HpError::DivideByZero.
///
/// # Errors
/// Propagates any error from the delegate.
pub fn op_adv_c_div(state: &mut CalcState) -> Result<(), HpError> {
    op_c_div(state)
}

// ── New Advantage Pac operations ─────────────────────────────────────────────

/// ADV Z^(1/W) — complex root z^(1/w) = exp(ln(z) / w).
///
/// Stack layout: z = X+iY (base), w = Z+iT (exponent).
/// Binary op: consumes ζ AND τ (all 4 registers), result in X+iY.
/// T-replicate: new Z and T both receive old T (HP-41 hardware T-replicate).
///
/// **Guards:**
/// - z = (0+0i) AND Re(w) ≤ 0 → HpError::Domain (ln(0) is -∞)
/// - z = (0+0i) AND Re(w) > 0 → returns (0+0i)
/// - w = (0+0i) → HpError::DivideByZero (division by zero in exponent)
///
/// LiftEffect: Enable. Sets complex_mode = true.
///
/// # Errors
/// Returns `HpError::Domain` or `HpError::DivideByZero` on invalid input.
pub fn op_adv_z_pow_1w(state: &mut CalcState) -> Result<(), HpError> {
    let z_re = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?; // re(z) = X
    let z_im = state.stack.y.inner().to_f64().ok_or(HpError::Overflow)?; // im(z) = Y
    let w_re = state.stack.z.inner().to_f64().ok_or(HpError::Overflow)?; // re(w) = Z
    let w_im = state.stack.t.inner().to_f64().ok_or(HpError::Overflow)?; // im(w) = T

    let old_t = state.stack.t.clone();

    // Guard: w = (0+0i) → DivideByZero (can't divide ln(z) by zero)
    if w_re == 0.0 && w_im == 0.0 {
        return Err(HpError::DivideByZero);
    }

    // Guard: z = (0+0i) with Re(w) ≤ 0 → Domain
    if z_re == 0.0 && z_im == 0.0 {
        if w_re <= 0.0 {
            return Err(HpError::Domain);
        }
        // z=0, Re(w) > 0: 0^(1/w) = 0
        state.complex_mode = true;
        state.stack.x = HpNum::zero();
        state.stack.y = HpNum::zero();
        state.stack.z = old_t.clone();
        state.stack.t = old_t;
        apply_lift_effect(state, LiftEffect::Enable);
        return Ok(());
    }

    state.complex_mode = true;

    // ln(z) = ln|z| + i·arg(z)
    let z_mag = (z_re * z_re + z_im * z_im).sqrt();
    let ln_z_re = z_mag.ln();
    let ln_z_im = z_im.atan2(z_re);

    // 1/w (complex reciprocal): 1/(w_re + i·w_im) = (w_re - i·w_im) / (w_re² + w_im²)
    let w_norm_sq = w_re * w_re + w_im * w_im;
    let inv_w_re = w_re / w_norm_sq;
    let inv_w_im = -w_im / w_norm_sq;

    // u = ln(z) / w = ln(z) * (1/w) = (ln_z_re + i·ln_z_im) * (inv_w_re + i·inv_w_im)
    let u_re = ln_z_re * inv_w_re - ln_z_im * inv_w_im;
    let u_im = ln_z_re * inv_w_im + ln_z_im * inv_w_re;

    // exp(u) = e^u_re · (cos(u_im) + i·sin(u_im))
    let exp_u_re = u_re.exp();
    let new_re = exp_u_re * u_im.cos();
    let new_im = exp_u_re * u_im.sin();

    let new_x = Decimal::from_f64(new_re)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    let new_y = Decimal::from_f64(new_im)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;

    state.stack.x = new_x;
    state.stack.y = new_y;
    // T-replicate
    state.stack.z = old_t.clone();
    state.stack.t = old_t;

    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV AIP — Alpha Integer Print.
///
/// Reads X (truncated to a non-negative integer), interprets it as a Unicode
/// scalar value, and appends the corresponding character to `state.alpha_reg`.
/// If the codepoint is not a valid Unicode scalar, silently skips (no error).
/// If X is negative, returns `HpError::Domain` (negative codepoints are invalid).
///
/// LiftEffect: Neutral (no stack modification).
///
/// # Errors
/// Returns `HpError::Domain` if X is negative.
pub fn op_adv_aip(state: &mut CalcState) -> Result<(), HpError> {
    let x_f = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;

    // Negative codepoint is invalid
    if x_f < 0.0 {
        return Err(HpError::Domain);
    }

    let codepoint = x_f.trunc() as u32;

    if let Some(ch) = char::from_u32(codepoint) {
        state.alpha_reg.push(ch);
    }

    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use rust_decimal::prelude::ToPrimitive;

    fn make_state(x: &str, y: &str, z: &str, t: &str) -> CalcState {
        let mut state = CalcState::new();
        state.stack.x = parse_hpnum(x);
        state.stack.y = parse_hpnum(y);
        state.stack.z = parse_hpnum(z);
        state.stack.t = parse_hpnum(t);
        state
    }

    fn parse_hpnum(s: &str) -> HpNum {
        let d = rust_decimal::Decimal::from_str_exact(s)
            .or_else(|_| rust_decimal::Decimal::from_scientific(s))
            .unwrap();
        HpNum::rounded(d)
    }

    fn get_x_f64(state: &CalcState) -> f64 {
        state.stack.x.inner().to_f64().unwrap()
    }

    fn get_y_f64(state: &CalcState) -> f64 {
        state.stack.y.inner().to_f64().unwrap()
    }

    // ── ADV MAGZ tests ───────────────────────────────────────────────────────

    /// Catches: MAGZ(3+4i) must return 5.0.
    /// Source: standard 3-4-5 Pythagorean triple.
    #[test]
    fn adv_magz_3_4_is_5() {
        let mut s = make_state("3", "4", "0", "0");
        op_adv_magz(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 5.0, max_relative = 1e-7);
    }

    /// Catches: MAGZ sets complex_mode.
    #[test]
    fn adv_magz_sets_complex_mode() {
        let mut s = make_state("3", "4", "0", "0");
        op_adv_magz(&mut s).unwrap();
        assert!(s.complex_mode);
    }

    /// Catches: MAGZ of pure real (4+0i) = 4.
    #[test]
    fn adv_magz_pure_real() {
        let mut s = make_state("4", "0", "0", "0");
        op_adv_magz(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 4.0, max_relative = 1e-7);
    }

    /// Catches: MAGZ of zero = 0.
    #[test]
    fn adv_magz_zero() {
        let mut s = make_state("0", "0", "0", "0");
        op_adv_magz(&mut s).unwrap();
        assert!(s.stack.x.is_zero());
    }

    /// Catches: MAGZ disables lift (unary op).
    #[test]
    fn adv_magz_disables_lift() {
        let mut s = make_state("3", "4", "0", "0");
        s.stack.lift_enabled = true;
        op_adv_magz(&mut s).unwrap();
        assert!(!s.stack.lift_enabled);
    }

    // ── ADV E^Z tests ────────────────────────────────────────────────────────

    /// Catches: Euler's formula e^(i*π) = -1+0i.
    /// Source: HP 00041-90034 ~p.25.
    #[test]
    fn adv_exp_z_euler_formula() {
        let pi_str = "3.14159265358979";
        let mut s = make_state("0", pi_str, "0", "0");
        op_adv_exp_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), -1.0, max_relative = 1e-6);
        assert_relative_eq!(get_y_f64(&s).abs(), 0.0, epsilon = 1e-6);
    }

    /// Catches: e^(0+0i) = 1+0i.
    #[test]
    fn adv_exp_z_zero_is_one() {
        let mut s = make_state("0", "0", "0", "0");
        op_adv_exp_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 1.0, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s).abs(), 0.0, epsilon = 1e-10);
    }

    /// Catches: e^(1+0i) = e.
    #[test]
    fn adv_exp_z_pure_real() {
        let mut s = make_state("1", "0", "0", "0");
        op_adv_exp_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), std::f64::consts::E, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s).abs(), 0.0, epsilon = 1e-10);
    }

    /// Catches: exp_z sets complex_mode.
    #[test]
    fn adv_exp_z_sets_complex_mode() {
        let mut s = make_state("0", "0", "0", "0");
        op_adv_exp_z(&mut s).unwrap();
        assert!(s.complex_mode);
    }

    /// Catches: exp_z disables lift.
    #[test]
    fn adv_exp_z_disables_lift() {
        let mut s = make_state("0", "0", "0", "0");
        s.stack.lift_enabled = true;
        op_adv_exp_z(&mut s).unwrap();
        assert!(!s.stack.lift_enabled);
    }

    // ── ADV LNZ tests ────────────────────────────────────────────────────────

    /// Catches: LNZ(1+0i) = 0+0i.
    #[test]
    fn adv_ln_z_one_is_zero() {
        let mut s = make_state("1", "0", "0", "0");
        op_adv_ln_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s).abs(), 0.0, epsilon = 1e-7);
    }

    /// Catches: LNZ(0+0i) → Domain error.
    #[test]
    fn adv_ln_z_zero_is_domain_error() {
        let mut s = make_state("0", "0", "0", "0");
        let result = op_adv_ln_z(&mut s);
        assert!(matches!(result, Err(HpError::Domain)));
    }

    /// Catches: LNZ(e+0i) = 1+0i.
    #[test]
    fn adv_ln_z_e_is_one() {
        let e_str = "2.71828182845905";
        let mut s = make_state(e_str, "0", "0", "0");
        op_adv_ln_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 1.0, max_relative = 1e-5);
    }

    /// Catches: LNZ disables lift.
    #[test]
    fn adv_ln_z_disables_lift() {
        let mut s = make_state("1", "0", "0", "0");
        s.stack.lift_enabled = true;
        op_adv_ln_z(&mut s).unwrap();
        assert!(!s.stack.lift_enabled);
    }

    /// Catches: LNZ sets complex_mode.
    #[test]
    fn adv_ln_z_sets_complex_mode() {
        let mut s = make_state("1", "0", "0", "0");
        op_adv_ln_z(&mut s).unwrap();
        assert!(s.complex_mode);
    }

    // ── ADV SINZ tests ───────────────────────────────────────────────────────

    /// Catches: SINZ(0+0i) = 0+0i.
    #[test]
    fn adv_sin_z_zero_is_zero() {
        let mut s = make_state("0", "0", "0", "0");
        op_adv_sin_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s).abs(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(get_y_f64(&s).abs(), 0.0, epsilon = 1e-10);
    }

    /// Catches: sin(π/2) = 1.
    #[test]
    fn adv_sin_z_pure_real_pi_over_2() {
        let pi_over_2 = "1.5707963267949";
        let mut s = make_state(pi_over_2, "0", "0", "0");
        op_adv_sin_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 1.0, max_relative = 1e-6);
    }

    /// Catches: SINZ sets complex_mode.
    #[test]
    fn adv_sin_z_sets_complex_mode() {
        let mut s = make_state("0", "0", "0", "0");
        op_adv_sin_z(&mut s).unwrap();
        assert!(s.complex_mode);
    }

    /// Catches: SINZ disables lift.
    #[test]
    fn adv_sin_z_disables_lift() {
        let mut s = make_state("0", "0", "0", "0");
        s.stack.lift_enabled = true;
        op_adv_sin_z(&mut s).unwrap();
        assert!(!s.stack.lift_enabled);
    }

    /// Catches: SINZ(1+1i) — non-trivial case matches identity.
    #[test]
    fn adv_sin_z_complex_case() {
        let mut s = make_state("1", "1", "0", "0");
        op_adv_sin_z(&mut s).unwrap();
        // sin(1+i) = sin(1)cosh(1) + i*cos(1)sinh(1)
        let expected_re = 1.0_f64.sin() * 1.0_f64.cosh();
        let expected_im = 1.0_f64.cos() * 1.0_f64.sinh();
        assert_relative_eq!(get_x_f64(&s), expected_re, max_relative = 1e-6);
        assert_relative_eq!(get_y_f64(&s), expected_im, max_relative = 1e-6);
    }

    // ── ADV C+ tests ─────────────────────────────────────────────────────────

    /// Catches: ADV C+ (1+2i) + (3+4i) = (4+6i).
    /// Source: HP 00041-90034 p.24.
    #[test]
    fn adv_c_plus_basic() {
        let mut s = make_state("1", "2", "3", "4");
        op_adv_c_plus(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 4.0, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s), 6.0, max_relative = 1e-7);
    }

    /// Catches: ADV C+ sets complex_mode.
    #[test]
    fn adv_c_plus_sets_complex_mode() {
        let mut s = make_state("1", "2", "3", "4");
        op_adv_c_plus(&mut s).unwrap();
        assert!(s.complex_mode);
    }

    /// Catches: ADV C+ T-replicate.
    #[test]
    fn adv_c_plus_t_replicate() {
        let mut s = make_state("1", "2", "3", "4");
        op_adv_c_plus(&mut s).unwrap();
        assert_relative_eq!(s.stack.z.inner().to_f64().unwrap(), 4.0, max_relative = 1e-7);
        assert_relative_eq!(s.stack.t.inner().to_f64().unwrap(), 4.0, max_relative = 1e-7);
    }

    /// Catches: ADV C+ enables lift.
    #[test]
    fn adv_c_plus_enables_lift() {
        let mut s = make_state("1", "2", "3", "4");
        s.stack.lift_enabled = false;
        op_adv_c_plus(&mut s).unwrap();
        assert!(s.stack.lift_enabled);
    }

    /// Catches: ADV C+ zero+zero = zero.
    #[test]
    fn adv_c_plus_zero_identity() {
        let mut s = make_state("0", "0", "0", "0");
        op_adv_c_plus(&mut s).unwrap();
        assert!(s.stack.x.is_zero());
        assert!(s.stack.y.is_zero());
    }

    // ── ADV CINV tests ───────────────────────────────────────────────────────

    /// Catches: ADV CINV(1+0i) = 1+0i.
    #[test]
    fn adv_cinv_one_is_one() {
        let mut s = make_state("1", "0", "0", "0");
        op_adv_cinv(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 1.0, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s).abs(), 0.0, epsilon = 1e-10);
    }

    /// Catches: ADV CINV(0+1i) = 0-1i.
    #[test]
    fn adv_cinv_i_is_neg_i() {
        let mut s = make_state("0", "1", "0", "0");
        op_adv_cinv(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s).abs(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(get_y_f64(&s), -1.0, max_relative = 1e-7);
    }

    /// Catches: ADV CINV(0+0i) → DivideByZero.
    #[test]
    fn adv_cinv_zero_is_divide_by_zero() {
        let mut s = make_state("0", "0", "0", "0");
        assert!(matches!(op_adv_cinv(&mut s), Err(HpError::DivideByZero)));
    }

    /// Catches: ADV CINV(2+0i) = 0.5+0i.
    #[test]
    fn adv_cinv_two_is_half() {
        let mut s = make_state("2", "0", "0", "0");
        op_adv_cinv(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 0.5, max_relative = 1e-7);
    }

    /// Catches: CINV sets complex_mode.
    #[test]
    fn adv_cinv_sets_complex_mode() {
        let mut s = make_state("1", "0", "0", "0");
        op_adv_cinv(&mut s).unwrap();
        assert!(s.complex_mode);
    }

    // ── ADV Z^N tests ────────────────────────────────────────────────────────

    /// Catches: Z^N with n=2 and z=(1+1i) = (0+2i).
    #[test]
    fn adv_z_pow_n_1_plus_i_squared() {
        // X=2 (exponent), complex base Y+iZ = 1+1i
        let mut s = make_state("2", "1", "1", "0");
        op_adv_z_pow_n(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 0.0, max_relative = 1e-6);
        assert_relative_eq!(get_y_f64(&s), 2.0, max_relative = 1e-6);
    }

    /// Catches: Z^0 = (1+0i).
    #[test]
    fn adv_z_pow_n_zero_exponent() {
        let mut s = make_state("0", "3", "4", "0");
        op_adv_z_pow_n(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 1.0, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&s).abs(), 0.0, epsilon = 1e-10);
    }

    /// Catches: Z^N sets complex_mode.
    #[test]
    fn adv_z_pow_n_sets_complex_mode() {
        let mut s = make_state("2", "1", "1", "0");
        op_adv_z_pow_n(&mut s).unwrap();
        assert!(s.complex_mode);
    }

    /// Catches: Z^N disables lift.
    #[test]
    fn adv_z_pow_n_disables_lift() {
        let mut s = make_state("2", "1", "1", "0");
        s.stack.lift_enabled = true;
        op_adv_z_pow_n(&mut s).unwrap();
        assert!(!s.stack.lift_enabled);
    }

    /// Catches: Z^(-1) = 1/z = CINV.
    #[test]
    fn adv_z_pow_n_neg_one_is_cinv() {
        // X=-1, base = Y+iZ = 2+0i; result = 0.5+0i
        let mut s = make_state("-1", "2", "0", "0");
        op_adv_z_pow_n(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 0.5, max_relative = 1e-6);
    }

    // ── ADV AIP tests ────────────────────────────────────────────────────────

    /// Catches: AIP with X=65 appends 'A' to ALPHA.
    #[test]
    fn adv_aip_65_is_a() {
        let mut s = make_state("65", "0", "0", "0");
        op_adv_aip(&mut s).unwrap();
        assert_eq!(s.alpha_reg, "A");
    }

    /// Catches: AIP with X=72 appends 'H'.
    #[test]
    fn adv_aip_72_is_h() {
        let mut s = make_state("72", "0", "0", "0");
        op_adv_aip(&mut s).unwrap();
        assert_eq!(s.alpha_reg, "H");
    }

    /// Catches: AIP appends (doesn't replace) existing ALPHA content.
    #[test]
    fn adv_aip_appends_to_existing() {
        let mut s = make_state("80", "0", "0", "0"); // 'P'
        s.alpha_reg = "H".to_string();
        op_adv_aip(&mut s).unwrap();
        assert_eq!(s.alpha_reg, "HP");
    }

    /// Catches: AIP with negative X → Domain error.
    #[test]
    fn adv_aip_negative_is_domain() {
        let mut s = make_state("-1", "0", "0", "0");
        assert!(matches!(op_adv_aip(&mut s), Err(HpError::Domain)));
    }

    /// Catches: AIP is LiftEffect::Neutral (doesn't change lift_enabled).
    #[test]
    fn adv_aip_neutral_lift() {
        let mut s = make_state("65", "0", "0", "0");
        s.stack.lift_enabled = false;
        op_adv_aip(&mut s).unwrap();
        assert!(!s.stack.lift_enabled, "AIP must be LiftEffect::Neutral");
        let mut s2 = make_state("65", "0", "0", "0");
        s2.stack.lift_enabled = true;
        op_adv_aip(&mut s2).unwrap();
        assert!(s2.stack.lift_enabled, "AIP must be LiftEffect::Neutral");
    }

    // ── ADV Z^(1/W) tests ────────────────────────────────────────────────────

    /// Catches: Z^(1/W) with z=e, w=1 → e^1 = e (ln(e)/1 = 1; exp(1) = e).
    #[test]
    fn adv_z_pow_1w_e_to_1_is_e() {
        let e_str = "2.71828182845905";
        // z = e+0i (X+iY), w = 1+0i (Z+iT)
        let mut s = make_state(e_str, "0", "1", "0");
        op_adv_z_pow_1w(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), std::f64::consts::E, max_relative = 1e-5);
        assert_relative_eq!(get_y_f64(&s).abs(), 0.0, epsilon = 1e-7);
    }

    /// Catches: Z^(1/W) with w=(0+0i) → DivideByZero.
    #[test]
    fn adv_z_pow_1w_zero_exponent_is_divide_by_zero() {
        let mut s = make_state("2", "0", "0", "0"); // z=2, w=0
        assert!(matches!(op_adv_z_pow_1w(&mut s), Err(HpError::DivideByZero)));
    }

    /// Catches: Z^(1/W) with z=(0+0i) and Re(w)>0 → (0+0i).
    #[test]
    fn adv_z_pow_1w_zero_base_pos_w_is_zero() {
        let mut s = make_state("0", "0", "1", "0");
        op_adv_z_pow_1w(&mut s).unwrap();
        assert!(s.stack.x.is_zero());
        assert!(s.stack.y.is_zero());
    }

    /// Catches: Z^(1/W) with z=(0+0i) and Re(w)<=0 → Domain.
    #[test]
    fn adv_z_pow_1w_zero_base_neg_w_is_domain() {
        let mut s = make_state("0", "0", "-1", "0");
        assert!(matches!(op_adv_z_pow_1w(&mut s), Err(HpError::Domain)));
    }

    /// Catches: Z^(1/W) sets complex_mode.
    #[test]
    fn adv_z_pow_1w_sets_complex_mode() {
        let e_str = "2.71828182845905";
        let mut s = make_state(e_str, "0", "1", "0");
        op_adv_z_pow_1w(&mut s).unwrap();
        assert!(s.complex_mode);
    }

    /// Catches: Z^(1/W) enables lift.
    #[test]
    fn adv_z_pow_1w_enables_lift() {
        let e_str = "2.71828182845905";
        let mut s = make_state(e_str, "0", "1", "0");
        s.stack.lift_enabled = false;
        op_adv_z_pow_1w(&mut s).unwrap();
        assert!(s.stack.lift_enabled);
    }

    // ── ADV LOGZ tests ───────────────────────────────────────────────────────

    /// Catches: LOGZ(10+0i) = 1+0i (log base 10 of 10 = 1).
    #[test]
    fn adv_log_z_ten_is_one() {
        let mut s = make_state("10", "0", "0", "0");
        op_adv_log_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s), 1.0, max_relative = 1e-6);
    }

    /// Catches: LOGZ(0+0i) → Domain.
    #[test]
    fn adv_log_z_zero_is_domain() {
        let mut s = make_state("0", "0", "0", "0");
        assert!(matches!(op_adv_log_z(&mut s), Err(HpError::Domain)));
    }

    /// Catches: LOGZ sets complex_mode.
    #[test]
    fn adv_log_z_sets_complex_mode() {
        let mut s = make_state("10", "0", "0", "0");
        op_adv_log_z(&mut s).unwrap();
        assert!(s.complex_mode);
    }

    /// Catches: LOGZ(1+0i) = 0+0i.
    #[test]
    fn adv_log_z_one_is_zero() {
        let mut s = make_state("1", "0", "0", "0");
        op_adv_log_z(&mut s).unwrap();
        assert_relative_eq!(get_x_f64(&s).abs(), 0.0, epsilon = 1e-7);
    }

    /// Catches: LOGZ disables lift.
    #[test]
    fn adv_log_z_disables_lift() {
        let mut s = make_state("10", "0", "0", "0");
        s.stack.lift_enabled = true;
        op_adv_log_z(&mut s).unwrap();
        assert!(!s.stack.lift_enabled);
    }
}
