// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `stat1::regression` — curve-fit Ops via log-linearization + Σ-block delegate (Plan 33-05).
//!
//! Ships the four log-linearizing curve-fit accumulators ΣLIN / ΣEXP /
//! ΣLOGI / ΣPOW. Each call is an ACCUMULATOR (interpretation A per
//! PATTERNS.md Pattern 4): the Op transforms `state.stack.x` (x-coord)
//! and/or `state.stack.y` (y-coord) in place via `ln`, then DELEGATES to
//! [`crate::ops::stats::op_sigma_plus`] for the actual R01–R06 update.
//! NEVER duplicates Σ arithmetic (anti-duplication invariant). After
//! the user has accumulated every (x, y) pair, the existing v1.x
//! [`crate::ops::stats::op_lr`] extracts slope (→ Y) + intercept (→ X).
//!
//! | Op    | Forward model    | Transform           | a-final from intercept |
//! |-------|------------------|---------------------|------------------------|
//! | ΣLIN  | ŷ = a + b·x      | identity            | a = intercept          |
//! | ΣEXP  | ŷ = a·e^(b·x)    | y ← ln y            | a = e^intercept        |
//! | ΣLOGI | ŷ = a + b·ln x   | x ← ln x            | a = intercept          |
//! | ΣPOW  | ŷ = a·x^b        | x ← ln x, y ← ln y  | a = e^intercept        |
//!
//! ## References
//!
//! - HP-41C Stat 1 Pac OM 00041-90030 (1979) §ΣLIN/ΣEXP/ΣLOGI/ΣPOW (p. 35).
//! - `numpy.polyfit(x, y, 1)` oracles for the inline test tuples (D-33.6).

use crate::error::HpError;
use crate::state::CalcState;

/// ΣLIN — accumulator for `ŷ = a + b·x`. Identity transform; pure
/// delegate to [`crate::ops::stats::op_sigma_plus`]. After per-point
/// accumulation, extract (b, a) via [`crate::ops::stats::op_lr`].
///
/// Errors: propagated from `op_sigma_plus` (SIZE-floor, Overflow).
///
/// Source: HP-41C Stat 1 Pac OM 00041-90030 §ΣLIN (p. 35).
pub fn op_sigma_lin(state: &mut CalcState) -> Result<(), HpError> {
    crate::ops::stats::op_sigma_plus(state)
}

/// ΣEXP — accumulator for `ŷ = a·e^(b·x)`. Linearizes via
/// `ln ŷ = ln a + b·x`: transforms `state.stack.y` (y-coordinate) by
/// `ln` then delegates to [`crate::ops::stats::op_sigma_plus`]. Final
/// `a = e^intercept` after [`crate::ops::stats::op_lr`].
///
/// Errors: `HpError::Domain` if y ≤ 0; SIZE-floor / Overflow from delegate.
///
/// Source: HP-41C Stat 1 Pac OM 00041-90030 §ΣEXP (p. 35).
pub fn op_sigma_exp(state: &mut CalcState) -> Result<(), HpError> {
    let ln_y = state.stack.y.checked_ln()?;
    state.stack.y = ln_y;
    crate::ops::stats::op_sigma_plus(state)
}

/// ΣLOGI — accumulator for `ŷ = a + b·ln x`. Linearizes via the
/// substitution `u = ln x`: transforms `state.stack.x` (x-coordinate)
/// by `ln` then delegates to [`crate::ops::stats::op_sigma_plus`].
/// Final (b, a) extracted directly from [`crate::ops::stats::op_lr`].
///
/// Errors: `HpError::Domain` if x ≤ 0; SIZE-floor / Overflow from delegate.
///
/// Source: HP-41C Stat 1 Pac OM 00041-90030 §ΣLOGI (p. 35).
pub fn op_sigma_logi(state: &mut CalcState) -> Result<(), HpError> {
    let ln_x = state.stack.x.checked_ln()?;
    state.stack.x = ln_x;
    crate::ops::stats::op_sigma_plus(state)
}

/// ΣPOW — accumulator for `ŷ = a·x^b`. Linearizes via
/// `ln ŷ = ln a + b·ln x`: transforms BOTH `state.stack.x` and
/// `state.stack.y` by `ln` then delegates to
/// [`crate::ops::stats::op_sigma_plus`]. Final `a = e^intercept` after
/// [`crate::ops::stats::op_lr`].
///
/// Errors: `HpError::Domain` if x ≤ 0 OR y ≤ 0; SIZE-floor / Overflow.
///
/// Source: HP-41C Stat 1 Pac OM 00041-90030 §ΣPOW (p. 35).
pub fn op_sigma_pow(state: &mut CalcState) -> Result<(), HpError> {
    let ln_x = state.stack.x.checked_ln()?;
    let ln_y = state.stack.y.checked_ln()?;
    state.stack.x = ln_x;
    state.stack.y = ln_y;
    crate::ops::stats::op_sigma_plus(state)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::num::HpNum;
    use crate::ops::stat1::STAT1_MAX_REG;
    use approx::assert_relative_eq;
    use rust_decimal::prelude::ToPrimitive;

    /// Helper: convert `HpNum` to `f64` for `assert_relative_eq!`.
    fn as_f64(n: &HpNum) -> f64 {
        n.inner()
            .to_f64()
            .expect("HpNum→f64 must succeed for finite oracle values")
    }

    /// Helper: load (x, y) into stack (x → X channel = independent;
    /// y → Y channel = dependent; per HP-41 Σ+ convention where
    /// `op_sigma_plus` accumulates `R02 = Σ(stack.x)` and `R05 =
    /// Σ(stack.y)`) and call the given accumulator Op.
    fn accumulate(
        state: &mut CalcState,
        x: f64,
        y: f64,
        op: fn(&mut CalcState) -> Result<(), HpError>,
    ) -> Result<(), HpError> {
        state.stack.x = HpNum::from(rust_decimal::Decimal::from_f64_retain(x).unwrap());
        state.stack.y = HpNum::from(rust_decimal::Decimal::from_f64_retain(y).unwrap());
        op(state)
    }

    /// Helper: extract `(intercept_X, slope_Y)` via op_lr.
    fn linear_reg(state: &mut CalcState) -> (f64, f64) {
        crate::ops::stats::op_lr(state).expect("op_lr after accumulation must succeed");
        (as_f64(&state.stack.x), as_f64(&state.stack.y))
    }

    // ── ΣLIN tests (SPEC.md Req. 15) ────────────────────────────────────────

    /// SPEC.md Req. 15 oracle: x=[1..5], y=[2,4,6,8,10] → a=0, b=2.
    /// `numpy.polyfit([1,2,3,4,5], [2,4,6,8,10], 1)` returns `[2.0, 0.0]`.
    #[test]
    fn lin_y_eq_2x() {
        let mut state = CalcState::new();
        for (x, y) in [(1.0, 2.0), (2.0, 4.0), (3.0, 6.0), (4.0, 8.0), (5.0, 10.0)] {
            accumulate(&mut state, x, y, op_sigma_lin).unwrap();
        }
        let (intercept, slope) = linear_reg(&mut state);
        assert_relative_eq!(intercept, 0.0, epsilon = 1e-9);
        assert_relative_eq!(slope, 2.0, max_relative = 1e-9);
    }

    /// SIZE-floor guard fires (delegated to op_sigma_plus check).
    #[test]
    fn lin_size_floor_guard() {
        let mut state = CalcState::new();
        state.regs.truncate(STAT1_MAX_REG - 40); // well below R01–R06 floor of 7
        state.stack.x = HpNum::from(1i32);
        state.stack.y = HpNum::from(2i32);
        assert_eq!(op_sigma_lin(&mut state).unwrap_err(), HpError::InvalidOp);
    }

    /// Non-zero intercept: y = 3 + 2x for x=[0,1,2] → y=[3,5,7].
    /// `numpy.polyfit([0,1,2], [3,5,7], 1)` → `[2.0, 3.0]`.
    #[test]
    fn lin_with_nonzero_intercept() {
        let mut state = CalcState::new();
        for (x, y) in [(0.0, 3.0), (1.0, 5.0), (2.0, 7.0)] {
            accumulate(&mut state, x, y, op_sigma_lin).unwrap();
        }
        let (intercept, slope) = linear_reg(&mut state);
        assert_relative_eq!(intercept, 3.0, max_relative = 1e-9);
        assert_relative_eq!(slope, 2.0, max_relative = 1e-9);
    }

    // ── ΣEXP tests (SPEC.md Req. 16) ────────────────────────────────────────

    /// SPEC.md Req. 16 oracle: x=[1,2,3], y=[e, e², e³] → a≈1.0, b≈1.0.
    /// After transform, accumulated pairs are (1, 1), (2, 2), (3, 3) —
    /// perfect identity → slope=1, intercept=0, a_final = e^0 = 1.0.
    #[test]
    fn exp_y_eq_e_pow_x() {
        let mut state = CalcState::new();
        let e = std::f64::consts::E;
        for (x, y) in [(1.0, e), (2.0, e * e), (3.0, e * e * e)] {
            accumulate(&mut state, x, y, op_sigma_exp).unwrap();
        }
        let (ln_a, b) = linear_reg(&mut state);
        // b is the exponent rate (no inverse transform).
        assert_relative_eq!(b, 1.0, max_relative = 1e-9);
        // a_final = e^ln_a.
        let a_final = ln_a.exp();
        assert_relative_eq!(a_final, 1.0, max_relative = 1e-9);
    }

    /// ΣEXP with y ≤ 0 returns Domain error (transforms y-coordinate
    /// via ln, which rejects non-positive arguments).
    #[test]
    fn exp_negative_y_returns_domain() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(1i32); // x = 1 (any positive)
        state.stack.y = HpNum::from(-1i32); // y = -1 → ln undefined
        assert_eq!(op_sigma_exp(&mut state).unwrap_err(), HpError::Domain);
    }

    /// Non-unit a: y = 2·e^x for x=[1,2,3]: y = 2e, 2e², 2e³.
    /// After ln: pairs are (1, ln(2e)) = (1, 1+ln2), (2, 2+ln2), (3, 3+ln2).
    /// Slope = 1, intercept = ln 2 ≈ 0.6931, a_final = e^ln2 = 2.0.
    ///
    /// SPEC.md Req. 46 two-level tolerance: ΣEXP transforms via
    /// transcendental `ln` of f64-constructed `e^n` values, accumulating
    /// f64 round-off; classify as iterative for the 1e-7 tolerance bucket.
    #[test]
    fn exp_with_nonunit_amplitude() {
        let mut state = CalcState::new();
        let e = std::f64::consts::E;
        for (x, y) in [(1.0, 2.0 * e), (2.0, 2.0 * e * e), (3.0, 2.0 * e * e * e)] {
            accumulate(&mut state, x, y, op_sigma_exp).unwrap();
        }
        let (ln_a, b) = linear_reg(&mut state);
        assert_relative_eq!(b, 1.0, max_relative = 1e-7);
        assert_relative_eq!(ln_a.exp(), 2.0, max_relative = 1e-7);
    }

    // ── ΣLOGI tests (SPEC.md Req. 17) ───────────────────────────────────────

    /// SPEC.md Req. 17 oracle: x=[1, e, e²], y=[2, 3, 4] → a=2.0, b=1.0.
    /// After ln(x): pairs are (0, 2), (1, 3), (2, 4) — slope=1, intercept=2.
    #[test]
    fn logi_y_eq_2_plus_ln_x() {
        let mut state = CalcState::new();
        let e = std::f64::consts::E;
        for (x, y) in [(1.0, 2.0), (e, 3.0), (e * e, 4.0)] {
            accumulate(&mut state, x, y, op_sigma_logi).unwrap();
        }
        let (intercept, slope) = linear_reg(&mut state);
        assert_relative_eq!(intercept, 2.0, max_relative = 1e-9);
        assert_relative_eq!(slope, 1.0, max_relative = 1e-9);
    }

    /// ΣLOGI with x ≤ 0 returns Domain error.
    #[test]
    fn logi_negative_x_returns_domain() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(-1i32); // x = -1 → ln undefined
        state.stack.y = HpNum::from(1i32); // y = 1
        assert_eq!(op_sigma_logi(&mut state).unwrap_err(), HpError::Domain);
    }

    /// Negative slope: y = 5 − 2·ln x for x=[1, e, e²] → y=[5, 3, 1].
    /// After ln(x): pairs (0, 5), (1, 3), (2, 1) — slope=−2, intercept=5.
    #[test]
    fn logi_with_negative_slope() {
        let mut state = CalcState::new();
        let e = std::f64::consts::E;
        for (x, y) in [(1.0, 5.0), (e, 3.0), (e * e, 1.0)] {
            accumulate(&mut state, x, y, op_sigma_logi).unwrap();
        }
        let (intercept, slope) = linear_reg(&mut state);
        assert_relative_eq!(intercept, 5.0, max_relative = 1e-9);
        assert_relative_eq!(slope, -2.0, max_relative = 1e-9);
    }

    // ── ΣPOW tests (SPEC.md Req. 18) ────────────────────────────────────────

    /// SPEC.md Req. 18 oracle: x=[1,2,4,8], y=[1,4,16,64] → a=1.0, b=2.0.
    /// After double ln: pairs (0,0), (ln2, ln4), (ln4, ln16), (ln8, ln64).
    /// slope = ln(y)/ln(x) = 2 throughout → b=2, intercept=0, a_final=e^0=1.
    ///
    /// SPEC.md Req. 46 two-level tolerance: ΣPOW transforms via two `ln`
    /// calls per data point + linear regression on the transformed pairs;
    /// classify as iterative for the 1e-7 tolerance bucket.
    #[test]
    fn pow_y_eq_x_sq() {
        let mut state = CalcState::new();
        for (x, y) in [(1.0, 1.0), (2.0, 4.0), (4.0, 16.0), (8.0, 64.0)] {
            accumulate(&mut state, x, y, op_sigma_pow).unwrap();
        }
        let (ln_a, b) = linear_reg(&mut state);
        assert_relative_eq!(b, 2.0, max_relative = 1e-7);
        // a_final = e^intercept = 1.0 (intercept ≈ 0).
        assert_relative_eq!(ln_a.exp(), 1.0, max_relative = 1e-7);
    }

    /// ΣPOW with x ≤ 0 returns Domain error (X-channel ln triggers first).
    #[test]
    fn pow_negative_x_returns_domain() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(-1i32); // x = -1 → ln undefined
        state.stack.y = HpNum::from(1i32);
        assert_eq!(op_sigma_pow(&mut state).unwrap_err(), HpError::Domain);
    }

    /// ΣPOW with y ≤ 0 returns Domain error (positive x; Y-channel ln
    /// is the failing transform).
    #[test]
    fn pow_negative_y_returns_domain() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(1i32);
        state.stack.y = HpNum::from(-1i32); // y = -1 → ln undefined
        assert_eq!(op_sigma_pow(&mut state).unwrap_err(), HpError::Domain);
    }

    /// Non-unit a: y = 3·x² for x=[1,2,4] → y=[3, 12, 48].
    /// After ln: (0, ln 3), (ln 2, ln 12), (ln 4, ln 48) — slope = 2,
    /// intercept = ln 3, a_final = 3.0.
    /// `numpy.polyfit([0, ln2, ln4], [ln3, ln12, ln48], 1)` → slope=2, intercept=ln3.
    #[test]
    fn pow_with_nonunit_amplitude() {
        let mut state = CalcState::new();
        for (x, y) in [(1.0, 3.0), (2.0, 12.0), (4.0, 48.0)] {
            accumulate(&mut state, x, y, op_sigma_pow).unwrap();
        }
        let (ln_a, b) = linear_reg(&mut state);
        assert_relative_eq!(b, 2.0, max_relative = 1e-9);
        assert_relative_eq!(ln_a.exp(), 3.0, max_relative = 1e-9);
    }
}
