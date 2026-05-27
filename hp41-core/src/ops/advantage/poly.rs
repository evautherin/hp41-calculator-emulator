// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `poly` — ADV MATH: PLY (polynomial evaluation) + RTS (root output).
//!
//! XROM module id 24 (ADV_MATH_B, bit-4 of `CalcState::xrom_modules`).
//!
//! Operations: PLY / RTS
//!
//! PLY evaluates a polynomial whose coefficients are stored in registers R01..R(n+1),
//! using Horner's method. RTS (roots) outputs the roots computed by a prior FROOT run.
//!
//! ## Entry convention (D-43.8):
//!
//! - PLY: `state.regs[0]` (R00) = degree; `state.stack.x` = evaluation point x;
//!   `state.regs[1..=degree+1]` = coefficients, highest-degree first.
//!
//! - RTS: requires prior FROOT state in `state.adv_froot_state`. Outputs roots
//!   sequentially on each call. When exhausted, wraps to first root.
//!
//! ## Coefficient register layout (D-43.8):
//!
//! For a polynomial of degree n:
//!   P(x) = c_n * x^n + c_{n-1} * x^{n-1} + ... + c_1 * x + c_0
//!
//! Storage (highest-degree first):
//!   R01 = c_n   (leading coefficient)
//!   R02 = c_{n-1}
//!   ...
//!   R(n+1) = c_0 (constant term)
//!
//! This matches FROOT's coefficient convention (D-43.8 unified layout).

use crate::{
    error::HpError,
    num::HpNum,
    stack::{apply_lift_effect, enter_number, unary_result, LiftEffect},
    state::CalcState,
};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

/// Maximum polynomial degree (T-43-13 domain guard, shared with solvers.rs).
const PLY_MAX_DEGREE: usize = 100;

/// ADV PLY — evaluate polynomial at X using Horner's method.
///
/// ## Entry convention (D-43.8):
/// - `state.stack.x` = x (evaluation point)
/// - `state.regs[0]` (R00) = degree (0..=100)
/// - `state.regs[1]` (R01) through `state.regs[degree+1]` = coefficients,
///   highest-degree first.
///
/// ## Algorithm (Horner):
///
/// ```text
/// result = coeffs[0]
/// for i in 1..=degree:
///   result = result * x + coeffs[i]
/// ```
///
/// ## Stack effect:
/// Pops X (evaluation point), pushes P(x) to X. `LiftEffect::Enable`.
///
/// # Errors
/// Returns `HpError::Domain` if degree > 100.
/// Returns `HpError::Overflow` on numerical failure (x or coefficient not representable).
pub fn op_adv_ply(state: &mut CalcState) -> Result<(), HpError> {
    let x_val = state.stack.x.inner().to_f64().ok_or(HpError::Overflow)?;

    let degree_raw = state
        .regs
        .first()
        .map(|r| r.numeric_or_zero().inner().to_u32().unwrap_or(0))
        .unwrap_or(0);
    let degree = degree_raw as usize;

    // T-43-13 domain guard
    if degree > PLY_MAX_DEGREE {
        return Err(HpError::Domain);
    }

    // Read leading coefficient from R01
    let leading = state
        .regs
        .get(1)
        .map(|r| r.numeric_or_zero().inner().to_f64().unwrap_or(0.0))
        .unwrap_or(0.0);

    // Horner evaluation: start with leading coefficient, then multiply + add each next coeff
    let mut result = leading;
    for i in 1..=degree {
        let reg_idx = i + 1; // R02, R03, ...
        let coeff = state
            .regs
            .get(reg_idx)
            .map(|r| r.numeric_or_zero().inner().to_f64().unwrap_or(0.0))
            .unwrap_or(0.0);
        result = result * x_val + coeff;
    }

    let result_hp = HpNum::from(Decimal::from_f64(result).ok_or(HpError::Overflow)?);
    unary_result(state, result_hp);
    apply_lift_effect(state, LiftEffect::Enable);

    Ok(())
}

/// ADV RTS — recall roots computed by FROOT sequentially.
///
/// Each call pushes the next root's real part to X. If the root is complex,
/// the imaginary part is pushed to Y (X = real, Y = imag, so Y is on top below X).
/// Advances the root cursor in `adv_froot_state`.
///
/// ## Cursor behavior:
/// - Cursor starts at 0 (set by FROOT on completion).
/// - Each RTS call advances cursor by 1.
/// - When cursor reaches end of root list, wraps to 0.
///
/// ## Stack effect:
/// For real root: pushes real part to X. LiftEffect::Enable.
/// For complex root: pushes imaginary part first (to Y), then real part (to X).
///
/// # Errors
/// Returns `HpError::InvalidOp` if `adv_froot_state` is None or root list is empty.
pub fn op_adv_rts(state: &mut CalcState) -> Result<(), HpError> {
    let (re, im, next_index) = match state.adv_froot_state {
        None => return Err(HpError::InvalidOp),
        Some(ref mut froot) => {
            if froot.roots_found.is_empty() {
                return Err(HpError::InvalidOp);
            }
            let idx = froot.root_index;
            if idx >= froot.roots_found.len() {
                // Wrap to first root
                froot.root_index = 1;
                let (re, im) = froot.roots_found[0];
                (re, im, 1usize)
            } else {
                let (re, im) = froot.roots_found[idx];
                let next = if idx + 1 >= froot.roots_found.len() {
                    0
                } else {
                    idx + 1
                };
                (re, im, next)
            }
        }
    };

    // Update cursor
    if let Some(ref mut froot) = state.adv_froot_state {
        froot.root_index = next_index;
    }

    let re_hp = HpNum::from(Decimal::from_f64(re).unwrap_or(Decimal::ZERO));

    if im.abs() < 1e-10 {
        // Real root: push to X
        unary_result(state, re_hp);
        apply_lift_effect(state, LiftEffect::Enable);
    } else {
        // Complex root: push im to Y, re to X
        let im_hp = HpNum::from(Decimal::from_f64(im).unwrap_or(Decimal::ZERO));
        state.stack.lift_enabled = true;
        enter_number(state, im_hp);
        apply_lift_effect(state, LiftEffect::Enable);
        state.stack.lift_enabled = true;
        enter_number(state, re_hp);
        apply_lift_effect(state, LiftEffect::Enable);
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Catches: PLY dispatch stubs broken
    #[test]
    fn poly_ops_compile() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(0i32);
        state.regs[0] = HpNum::from(0i32).into();
        state.regs[1] = HpNum::from(5i32).into();
        // PLY of constant 5 at x=0 → 5
        let result = op_adv_ply(&mut state);
        assert!(result.is_ok(), "PLY constant: {result:?}");
        let val = state.stack.x.inner().to_f64().unwrap();
        // LINT-EXEMPT: pure-f64 Horner result; degree-0 constant 5, exact
        assert!((val - 5.0).abs() < 1e-9, "PLY(5)=5, got {val}");
    }

    // Catches: PLY degree-2 Horner evaluation wrong
    #[test]
    fn poly_horner_degree2() {
        // P(x) = 2x^2 + 3x + 1 at x=2 → 15
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(2i32);
        state.stack.lift_enabled = false;
        state.regs[0] = HpNum::from(2i32).into();
        state.regs[1] = HpNum::from(2i32).into();
        state.regs[2] = HpNum::from(3i32).into();
        state.regs[3] = HpNum::from(1i32).into();

        let result = op_adv_ply(&mut state);
        assert!(result.is_ok());
        let val = state.stack.x.inner().to_f64().unwrap();
        // LINT-EXEMPT: pure-f64 Horner result; integer coefficients at x=2, exact
        assert!((val - 15.0).abs() < 1e-9, "PLY(2x^2+3x+1, 2) = {val}");
    }

    // Catches: PLY degree-3 Horner wrong
    #[test]
    fn poly_horner_degree3() {
        // P(x) = x^3 + 2x^2 + 3x + 4 at x=1 → 10
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(1i32);
        state.stack.lift_enabled = false;
        state.regs[0] = HpNum::from(3i32).into();
        state.regs[1] = HpNum::from(1i32).into();
        state.regs[2] = HpNum::from(2i32).into();
        state.regs[3] = HpNum::from(3i32).into();
        state.regs[4] = HpNum::from(4i32).into();

        let result = op_adv_ply(&mut state);
        assert!(result.is_ok());
        let val = state.stack.x.inner().to_f64().unwrap();
        // LINT-EXEMPT: pure-f64 Horner result; degree-3 integer coefficients at x=1, exact
        assert!((val - 10.0).abs() < 1e-9, "PLY(x^3+2x^2+3x+4, 1) = {val}");
    }

    // Catches: PLY domain rejection missing for degree > 100
    #[test]
    fn poly_domain_rejection() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(1i32);
        state.regs[0] = HpNum::from(101i32).into(); // > max
        let result = op_adv_ply(&mut state);
        assert_eq!(result, Err(HpError::Domain));
    }

    // Catches: RTS with no prior FROOT state not returning InvalidOp
    #[test]
    fn rts_no_state_returns_invalid_op() {
        let mut state = CalcState::new();
        assert!(state.adv_froot_state.is_none());
        assert_eq!(op_adv_rts(&mut state), Err(HpError::InvalidOp));
    }

    // Catches: RTS sequential output broken
    #[test]
    fn rts_sequential() {
        let mut state = CalcState::new();
        state.adv_froot_state = Some(crate::ops::advantage::solvers::FrootState {
            degree: 2,
            roots_found: vec![(2.0, 0.0), (-2.0, 0.0)],
            root_index: 0,
        });
        state.stack.lift_enabled = true;

        // First RTS → 2.0
        let result = op_adv_rts(&mut state);
        assert!(result.is_ok());
        let val = state.stack.x.inner().to_f64().unwrap();
        // LINT-EXEMPT: pure-f64 root bridge; exact f64 literal 2.0 stored in FrootState
        assert!((val - 2.0).abs() < 1e-9, "RTS[0] = {val}");

        // Second RTS → -2.0
        state.stack.lift_enabled = true;
        let result = op_adv_rts(&mut state);
        assert!(result.is_ok());
        let val = state.stack.x.inner().to_f64().unwrap();
        // LINT-EXEMPT: pure-f64 root bridge; exact f64 literal -2.0 stored in FrootState
        assert!((val - (-2.0)).abs() < 1e-9, "RTS[1] = {val}");
    }

    // Catches: RTS cursor wrap-around broken
    #[test]
    fn rts_cursor_wraps() {
        let mut state = CalcState::new();
        state.adv_froot_state = Some(crate::ops::advantage::solvers::FrootState {
            degree: 1,
            roots_found: vec![(3.0, 0.0)],
            root_index: 0,
        });
        state.stack.lift_enabled = true;

        // First call → 3.0
        let _ = op_adv_rts(&mut state);
        // Second call wraps → 3.0 again
        state.stack.lift_enabled = true;
        let result = op_adv_rts(&mut state);
        assert!(result.is_ok(), "RTS wrap: {result:?}");
        let val = state.stack.x.inner().to_f64().unwrap();
        // LINT-EXEMPT: pure-f64 root bridge; exact f64 literal 3.0 stored in FrootState
        assert!((val - 3.0).abs() < 1e-9, "RTS wrap to first: {val}");
    }

    // Catches: PLY evaluation at x=0 (constant term check)
    #[test]
    fn ply_at_x_zero() {
        // P(0) = constant term regardless of higher terms
        // P(x) = 5x^2 + 3x + 7, at x=0 → 7
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(0i32);
        state.stack.lift_enabled = false;
        state.regs[0] = HpNum::from(2i32).into();
        state.regs[1] = HpNum::from(5i32).into();
        state.regs[2] = HpNum::from(3i32).into();
        state.regs[3] = HpNum::from(7i32).into();

        let result = op_adv_ply(&mut state);
        assert!(result.is_ok());
        let val = state.stack.x.inner().to_f64().unwrap();
        // LINT-EXEMPT: pure-f64 Horner result; x=0 collapses to constant term 7, exact
        assert!((val - 7.0).abs() < 1e-9, "PLY at x=0 = {val}");
    }
}
