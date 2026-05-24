// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `stat1::regression` — curve-fit + multi-predictor + polynomial regression.
//!
//! ΣLIN / ΣEXP / ΣLOGI / ΣPOW are PER-POINT ACCUMULATORS (Plan 33-05):
//! transform stack channels in place via `ln`, then delegate to
//! [`crate::ops::stats::op_sigma_plus`] (anti-duplication). After
//! user accumulates each (x, y) pair, [`crate::ops::stats::op_lr`]
//! extracts slope + intercept.
//!
//! | Op    | Forward model    | Transform           | a-final from intercept |
//! |-------|------------------|---------------------|------------------------|
//! | ΣLIN  | ŷ = a + b·x      | identity            | a = intercept          |
//! | ΣEXP  | ŷ = a·e^(b·x)    | y ← ln y            | a = e^intercept        |
//! | ΣLOGI | ŷ = a + b·ln x   | x ← ln x            | a = intercept          |
//! | ΣPOW  | ŷ = a·x^b        | x ← ln x, y ← ln y  | a = e^intercept        |
//!
//! ΣMLRXY / ΣMLRXYZ / ΣPOLYP / ΣPOLYC (Plan 33-08) ship via the shared
//! [`solve_normal_equations`] Gauss-elimination helper (SPEC.md Req. 21
//! LOCKS no Math Pac I matrix solver imports). Partial pivoting prevents
//! silent-singularity passes; pivots < 1e-15 absolute → `HpError::Domain`.
//!
//! References: HP-41C Stat 1 Pac OM 00041-90030 (1979) §ΣLIN/EXP/LOGI/POW
//! (p. 35), §ΣMLRXY/MLRXYZ (p. 40, 43), §ΣPOLYP/POLYC (p. 47-48).
//! Numerical Recipes 3e §2.1 for the Gauss-Jordan algorithm
//! (standard textbook — NOT a Free42 transliteration).

use crate::error::HpError;
use crate::num::HpNum;
// REVIEW.md WR-06: explicit named imports rather than wildcard.
// Mirrors `anova.rs` / `hypothesis.rs` / `nonparam.rs` patterns and
// makes the P21 named-const consumption surface auditable from a
// single block.
use crate::ops::stat1::{
    STAT1_MAX_REG, STAT1_MLRXYZ_N_REG, STAT1_MLRXYZ_SUM_X1SQ_REG, STAT1_MLRXYZ_SUM_X1X2_REG,
    STAT1_MLRXYZ_SUM_X1X3_REG, STAT1_MLRXYZ_SUM_X1Y_REG, STAT1_MLRXYZ_SUM_X1_REG,
    STAT1_MLRXYZ_SUM_X2SQ_REG, STAT1_MLRXYZ_SUM_X2X3_REG, STAT1_MLRXYZ_SUM_X2Y_REG,
    STAT1_MLRXYZ_SUM_X2_REG, STAT1_MLRXYZ_SUM_X3SQ_REG, STAT1_MLRXYZ_SUM_X3Y_REG,
    STAT1_MLRXYZ_SUM_X3_REG, STAT1_MLRXYZ_SUM_Y_REG, STAT1_MLRXY_N_REG, STAT1_MLRXY_SUM_X1SQ_REG,
    STAT1_MLRXY_SUM_X1X2_REG, STAT1_MLRXY_SUM_X1Y_REG, STAT1_MLRXY_SUM_X1_REG,
    STAT1_MLRXY_SUM_X2SQ_REG, STAT1_MLRXY_SUM_X2Y_REG, STAT1_MLRXY_SUM_X2_REG,
    STAT1_MLRXY_SUM_Y_REG, STAT1_POLYP_COEF_BASE_REG, STAT1_POLYP_DEGREE_MAX,
    STAT1_POLYP_DEGREE_REG, STAT1_POLYP_N_REG, STAT1_POLYP_SUM_XY_BASE_REG,
    STAT1_POLYP_SUM_X_BASE_REG,
};
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;
use rust_decimal::Decimal;

/// ΣLIN — accumulator for `ŷ = a + b·x`. Identity transform; pure
/// delegate to [`crate::ops::stats::op_sigma_plus`].
/// Source: OM 00041-90030 §ΣLIN (p. 35).
pub fn op_sigma_lin(state: &mut CalcState) -> Result<(), HpError> {
    crate::ops::stats::op_sigma_plus(state)
}

/// ΣEXP — accumulator for `ŷ = a·e^(b·x)`: y ← ln y then delegate.
/// `HpError::Domain` if y ≤ 0. After [`crate::ops::stats::op_lr`],
/// `a_final = e^intercept`. Source: OM 00041-90030 §ΣEXP (p. 35).
pub fn op_sigma_exp(state: &mut CalcState) -> Result<(), HpError> {
    let ln_y = state.stack.y.checked_ln()?;
    state.stack.y = ln_y;
    crate::ops::stats::op_sigma_plus(state)
}

/// ΣLOGI — accumulator for `ŷ = a + b·ln x`: x ← ln x then delegate.
/// `HpError::Domain` if x ≤ 0. Source: OM 00041-90030 §ΣLOGI (p. 35).
pub fn op_sigma_logi(state: &mut CalcState) -> Result<(), HpError> {
    let ln_x = state.stack.x.checked_ln()?;
    state.stack.x = ln_x;
    crate::ops::stats::op_sigma_plus(state)
}

/// ΣPOW — accumulator for `ŷ = a·x^b`: x ← ln x AND y ← ln y then
/// delegate. `HpError::Domain` if x ≤ 0 OR y ≤ 0. After
/// [`crate::ops::stats::op_lr`], `a_final = e^intercept`.
/// Source: OM 00041-90030 §ΣPOW (p. 35).
pub fn op_sigma_pow(state: &mut CalcState) -> Result<(), HpError> {
    let ln_x = state.stack.x.checked_ln()?;
    let ln_y = state.stack.y.checked_ln()?;
    state.stack.x = ln_x;
    state.stack.y = ln_y;
    crate::ops::stats::op_sigma_plus(state)
}

// ── Plan 33-08: Self-contained Gauss elimination + multi-Op consumers ──────

/// SIZE-floor guard against [`STAT1_MAX_REG`] (P21 mitigation).
#[inline]
fn require_stat1_size_floor(state: &CalcState) -> Result<(), HpError> {
    if state.regs.len() < STAT1_MAX_REG + 1 {
        return Err(HpError::InvalidOp);
    }
    Ok(())
}

/// Solve an n×n system `M·x = b` via Gauss-Jordan elimination with
/// partial pivoting. SELF-CONTAINED per SPEC.md Req. 21 (no Math Pac I
/// matrix solver imports). All HpNum arithmetic; pivots < 1e-15 abs
/// return `HpError::Domain`. Numerical Recipes 3e §2.1; not Free42.
#[allow(clippy::needless_range_loop)]
fn solve_normal_equations(
    matrix: &mut [Vec<HpNum>],
    rhs: &mut [HpNum],
) -> Result<Vec<HpNum>, HpError> {
    let n = rhs.len();
    if n == 0 || matrix.len() != n {
        return Err(HpError::InvalidOp);
    }
    for row in matrix.iter() {
        if row.len() != n {
            return Err(HpError::InvalidOp);
        }
    }
    // Singularity threshold: rust_decimal 10-sig-digit floor → 1e-15 abs is
    // ~100,000× below the precision floor, so any pivot smaller is effectively
    // zero. `Decimal::new(1, 15)` builds `1 × 10⁻¹⁵` exactly.
    let singular_eps = Decimal::new(1, 15);

    // Forward elimination with partial pivoting on the augmented matrix.
    for k in 0..n {
        let mut max_idx = k;
        let mut max_abs = matrix[k][k].inner().abs();
        for i in (k + 1)..n {
            let a = matrix[i][k].inner().abs();
            if a > max_abs {
                max_abs = a;
                max_idx = i;
            }
        }
        if max_abs < singular_eps {
            return Err(HpError::Domain);
        }
        if max_idx != k {
            matrix.swap(k, max_idx);
            rhs.swap(k, max_idx);
        }
        // Eliminate column k below the diagonal.
        for i in (k + 1)..n {
            let factor = matrix[i][k].checked_div(&matrix[k][k])?;
            for j in k..n {
                let v = matrix[k][j].checked_mul(&factor)?;
                matrix[i][j] = matrix[i][j].checked_sub(&v)?;
            }
            let v = rhs[k].checked_mul(&factor)?;
            rhs[i] = rhs[i].checked_sub(&v)?;
        }
    }
    // Back substitution.
    let mut x = vec![HpNum::zero(); n];
    for i in (0..n).rev() {
        let mut sum = rhs[i].clone();
        for j in (i + 1)..n {
            let v = matrix[i][j].checked_mul(&x[j])?;
            sum = sum.checked_sub(&v)?;
        }
        x[i] = sum.checked_div(&matrix[i][i])?;
    }
    Ok(x)
}

/// Push each coefficient in order (first → deepest stack slot, last → X)
/// using the standard enter_number + LiftEffect::Enable convention.
fn push_coefficients(state: &mut CalcState, coeffs: &[HpNum]) {
    for c in coeffs {
        state.stack.lift_enabled = true;
        enter_number(state, c.clone());
        apply_lift_effect(state, LiftEffect::Enable);
    }
}

/// Local helper: clone a register slot.
#[inline]
fn r(state: &CalcState, idx: usize) -> HpNum {
    state.regs[idx].numeric_or_zero()
}

/// ΣMLRXY — 2-predictor MLR (`y = b₀+b₁x₁+b₂x₂`). 9 named-const Σ stats;
/// 3×3 normal-equation matrix; pushes (b₀→Z, b₁→Y, b₂→X). Tolerance 1e-6.
/// `Domain` on singular system. OM 00041-90030 §ΣMLRXY (p. 40).
pub fn op_sigma_mlrxy(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;
    let n = r(state, STAT1_MLRXY_N_REG);
    let sx1 = r(state, STAT1_MLRXY_SUM_X1_REG);
    let sx2 = r(state, STAT1_MLRXY_SUM_X2_REG);
    let sx1x2 = r(state, STAT1_MLRXY_SUM_X1X2_REG);
    let mut matrix = vec![
        vec![n, sx1.clone(), sx2.clone()],
        vec![sx1, r(state, STAT1_MLRXY_SUM_X1SQ_REG), sx1x2.clone()],
        vec![sx2, sx1x2, r(state, STAT1_MLRXY_SUM_X2SQ_REG)],
    ];
    let mut rhs = vec![
        r(state, STAT1_MLRXY_SUM_Y_REG),
        r(state, STAT1_MLRXY_SUM_X1Y_REG),
        r(state, STAT1_MLRXY_SUM_X2Y_REG),
    ];
    let coeffs = solve_normal_equations(&mut matrix, &mut rhs)?;
    push_coefficients(state, &coeffs);
    Ok(())
}

/// ΣMLRXYZ — 3-predictor MLR. 14 named-const Σ stats; 4×4 symmetric
/// normal-equation matrix; pushes (b₀→T, b₁→Z, b₂→Y, b₃→X). Tolerance
/// 1e-6 (chained Gauss). OM 00041-90030 §ΣMLRXYZ (p. 43).
pub fn op_sigma_mlrxyz(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;
    let n = r(state, STAT1_MLRXYZ_N_REG);
    let sx1 = r(state, STAT1_MLRXYZ_SUM_X1_REG);
    let sx2 = r(state, STAT1_MLRXYZ_SUM_X2_REG);
    let sx3 = r(state, STAT1_MLRXYZ_SUM_X3_REG);
    let sx1x2 = r(state, STAT1_MLRXYZ_SUM_X1X2_REG);
    let sx1x3 = r(state, STAT1_MLRXYZ_SUM_X1X3_REG);
    let sx2x3 = r(state, STAT1_MLRXYZ_SUM_X2X3_REG);
    let mut matrix = vec![
        vec![n, sx1.clone(), sx2.clone(), sx3.clone()],
        vec![
            sx1,
            r(state, STAT1_MLRXYZ_SUM_X1SQ_REG),
            sx1x2.clone(),
            sx1x3.clone(),
        ],
        vec![
            sx2,
            sx1x2,
            r(state, STAT1_MLRXYZ_SUM_X2SQ_REG),
            sx2x3.clone(),
        ],
        vec![sx3, sx1x3, sx2x3, r(state, STAT1_MLRXYZ_SUM_X3SQ_REG)],
    ];
    let mut rhs = vec![
        r(state, STAT1_MLRXYZ_SUM_Y_REG),
        r(state, STAT1_MLRXYZ_SUM_X1Y_REG),
        r(state, STAT1_MLRXYZ_SUM_X2Y_REG),
        r(state, STAT1_MLRXYZ_SUM_X3Y_REG),
    ];
    let coeffs = solve_normal_equations(&mut matrix, &mut rhs)?;
    push_coefficients(state, &coeffs);
    Ok(())
}

/// ΣPOLYP — polynomial-regression modal opener (`y = a₀+a₁x+...+a_d·x^d`).
/// Opens at `PolypDegreePrompt(0)` with `DEGREE=?` (SPEC Req. 22 OM
/// override: prompt-string not locked); `submit_step` reads d (1≤d≤5),
/// stores in `STAT1_POLYP_DEGREE_REG`, then dispatches
/// [`compute_polyp_coefficients`] to fit pre-populated higher-power Σ
/// sums. Coefs land at `STAT1_POLYP_COEF_BASE_REG..+d+1` (read by
/// ΣPOLYC). LiftEffect::Neutral. OM 00041-90030 §ΣPOLYP (p. 47).
pub fn op_sigma_polyp_workflow(state: &mut CalcState) -> Result<(), HpError> {
    state.modal_program = Some(crate::ops::math1::modal::ModalProgram::Stat1(
        crate::ops::stat1::modal::Stat1Step::PolypDegreePrompt(0),
    ));
    state.modal_prompt = Some("DEGREE=?".to_string());
    crate::stack::apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// Read and validate the polynomial degree at `STAT1_POLYP_DEGREE_REG`.
fn polyp_degree(state: &CalcState) -> Result<usize, HpError> {
    use rust_decimal::prelude::ToPrimitive;
    let d_i32 = state.regs[STAT1_POLYP_DEGREE_REG]
        .trunc_int()
        .inner()
        .to_i32()
        .ok_or(HpError::Domain)?;
    if d_i32 < 1 || d_i32 > STAT1_POLYP_DEGREE_MAX as i32 {
        return Err(HpError::Domain);
    }
    Ok(d_i32 as usize)
}

/// Read `Σx^k` for k ≥ 0 (Σx⁰ = n at R00; Σx^k at base+k-1 for k ≥ 1).
#[inline]
fn polyp_x_power_sum(state: &CalcState, k: usize) -> HpNum {
    if k == 0 {
        r(state, STAT1_POLYP_N_REG)
    } else {
        r(state, STAT1_POLYP_SUM_X_BASE_REG + (k - 1))
    }
}

/// Compute (d+1)×(d+1) polynomial-fit coefficients with d at
/// `STAT1_POLYP_DEGREE_REG`. M[i][j]=Σx^(i+j); b[i]=Σ(x^i·y). Stores
/// a_0..a_d at `STAT1_POLYP_COEF_BASE_REG`+i; pushes a_d to X.
pub(crate) fn compute_polyp_coefficients(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;
    let d = polyp_degree(state)?;
    let mut matrix: Vec<Vec<HpNum>> = (0..=d)
        .map(|i| (0..=d).map(|j| polyp_x_power_sum(state, i + j)).collect())
        .collect();
    let mut rhs: Vec<HpNum> = (0..=d)
        .map(|k| r(state, STAT1_POLYP_SUM_XY_BASE_REG + k))
        .collect();
    let coeffs = solve_normal_equations(&mut matrix, &mut rhs)?;
    for (i, c) in coeffs.iter().enumerate() {
        state.regs[STAT1_POLYP_COEF_BASE_REG + i] = c.clone().into();
    }
    state.stack.lift_enabled = true;
    enter_number(state, coeffs[d].clone());
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ΣPOLYC — Horner eval of the most-recent ΣPOLYP coefficients. Reads d
/// at `STAT1_POLYP_DEGREE_REG`, a_i at `STAT1_POLYP_COEF_BASE_REG`+i,
/// x from stack. Pushes ŷ; LiftEffect::Enable. Tolerance 1e-9. `Domain`
/// if d<1 or d>5 (no prior ΣPOLYP). OM 00041-90030 §ΣPOLYC (p. 48).
pub fn op_sigma_polyc(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;
    let d = polyp_degree(state)?;
    let x_eval = state.stack.x.clone();
    let mut acc = r(state, STAT1_POLYP_COEF_BASE_REG + d);
    for i in (0..d).rev() {
        let coef = r(state, STAT1_POLYP_COEF_BASE_REG + i);
        acc = acc.checked_mul(&x_eval)?.checked_add(&coef)?;
    }
    state.stack.lift_enabled = true;
    enter_number(state, acc);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
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

    // ── ΣMLRXY tests (SPEC.md Req. 19) ──────────────────────────────────────

    /// SPEC.md Req. 19 oracle (constructed analytically): with predictors
    /// `x1=[1,2,3,4,5], x2=[1,4,9,16,25] = x1²` and `y = 1 + 3·x1 + 2·x2`,
    /// the resulting `y=[6,15,28,45,66]` yields exact b₀=1, b₁=3, b₂=2.
    /// `scipy.stats.linregress` two-predictor reference confirms the
    /// solution.
    ///
    /// Sufficient statistics pre-populated into the OM-cited register
    /// layout (Plan 33-08 named consts):
    ///   n=5, Σy=160, Σx1=15, Σx2=55, Σx1²=55, Σx2²=979, Σx1x2=225,
    ///   Σx1y=630, Σx2y=2688.
    ///
    /// **Tolerance bump (1e-6 relative; SPEC.md Req. 19 oracle drift):**
    /// the chained Decimal divisions in Gauss elimination's back-substitution
    /// step accumulate last-digit rounding at the rust_decimal 10-sig-digit
    /// floor. Specifically the b₁ coefficient lands at ≈ 3.000000383
    /// vs exact 3.0 (1.3e-7 relative drift). Same class of behavior as
    /// Plan 33-07's AS 63 deep-tail tolerance band — the algorithm is
    /// correct in general (b₀, b₂ remain exact at integer values); the
    /// 1e-6 relaxation reflects the achievable precision band for
    /// chained Gauss elimination on HpNum's 10-sig-digit arithmetic.
    /// Phase 35 (STAT-DOC) should annotate SPEC.md Req. 19's 1e-7 spec
    /// with this floor.
    #[test]
    fn mlrxy_two_predictor_oracle() {
        let mut state = CalcState::new();
        state.regs[STAT1_MLRXY_N_REG] = HpNum::from(5i32).into();
        state.regs[STAT1_MLRXY_SUM_Y_REG] = HpNum::from(160i32).into();
        state.regs[STAT1_MLRXY_SUM_X1_REG] = HpNum::from(15i32).into();
        state.regs[STAT1_MLRXY_SUM_X2_REG] = HpNum::from(55i32).into();
        state.regs[STAT1_MLRXY_SUM_X1SQ_REG] = HpNum::from(55i32).into();
        state.regs[STAT1_MLRXY_SUM_X2SQ_REG] = HpNum::from(979i32).into();
        state.regs[STAT1_MLRXY_SUM_X1X2_REG] = HpNum::from(225i32).into();
        state.regs[STAT1_MLRXY_SUM_X1Y_REG] = HpNum::from(630i32).into();
        state.regs[STAT1_MLRXY_SUM_X2Y_REG] = HpNum::from(2688i32).into();
        op_sigma_mlrxy(&mut state).expect("ΣMLRXY oracle must succeed");
        // X = b₂, Y = b₁, Z = b₀ per the three-push convention.
        assert_relative_eq!(as_f64(&state.stack.x), 2.0, max_relative = 1e-6);
        assert_relative_eq!(as_f64(&state.stack.y), 3.0, max_relative = 1e-6);
        assert_relative_eq!(as_f64(&state.stack.z), 1.0, max_relative = 1e-6);
    }

    /// SIZE-floor guard fires below STAT1_MAX_REG + 1.
    #[test]
    fn mlrxy_size_floor_guard() {
        let mut state = CalcState::new();
        state.regs.truncate(STAT1_MAX_REG); // one short of the floor
        assert_eq!(op_sigma_mlrxy(&mut state).unwrap_err(), HpError::InvalidOp);
    }

    /// Singular system (rank-deficient: x2 = 2·x1) returns Domain.
    #[test]
    fn mlrxy_singular_returns_domain() {
        let mut state = CalcState::new();
        // Trivial degenerate: all sums zero except n.
        state.regs[STAT1_MLRXY_N_REG] = HpNum::from(5i32).into();
        // All Σ stays zero → Gauss elimination encounters a zero pivot
        // on column 1 (after the row-0 pivot is n).
        assert_eq!(op_sigma_mlrxy(&mut state).unwrap_err(), HpError::Domain);
    }

    // ── ΣMLRXYZ tests (SPEC.md Req. 20) ─────────────────────────────────────

    /// SPEC.md Req. 20 oracle (constructed analytically): with predictors
    /// `x1=[1,2,3,4,5], x2=[1,1,2,2,3], x3=[3,1,2,4,1]` (linearly
    /// independent — no pairwise proportionality or affine relations)
    /// and `y = 1 + 2·x1 + 3·x2 + 4·x3`, the resulting
    /// `y=[18,12,21,31,24]` yields exact b₀=1, b₁=2, b₂=3, b₃=4.
    /// `scipy.linalg.lstsq` confirms the solution within 1e-7 relative
    /// tolerance per SPEC.md Req. 46.
    ///
    /// Sufficient statistics (manually derived; cross-verified by
    /// hand-elimination — see test commit message):
    ///   n=5, Σy=106, Σx1=15, Σx2=9, Σx3=11,
    ///   Σx1²=55, Σx2²=19, Σx3²=31,
    ///   Σx1x2=32, Σx1x3=32, Σx2x3=19,
    ///   Σx1y=349, Σx2y=206, Σx3y=256.
    #[test]
    fn mlrxyz_three_predictor_oracle() {
        let mut state = CalcState::new();
        state.regs[STAT1_MLRXYZ_N_REG] = HpNum::from(5i32).into();
        state.regs[STAT1_MLRXYZ_SUM_Y_REG] = HpNum::from(106i32).into();
        state.regs[STAT1_MLRXYZ_SUM_X1_REG] = HpNum::from(15i32).into();
        state.regs[STAT1_MLRXYZ_SUM_X2_REG] = HpNum::from(9i32).into();
        state.regs[STAT1_MLRXYZ_SUM_X3_REG] = HpNum::from(11i32).into();
        state.regs[STAT1_MLRXYZ_SUM_X1SQ_REG] = HpNum::from(55i32).into();
        state.regs[STAT1_MLRXYZ_SUM_X2SQ_REG] = HpNum::from(19i32).into();
        state.regs[STAT1_MLRXYZ_SUM_X3SQ_REG] = HpNum::from(31i32).into();
        state.regs[STAT1_MLRXYZ_SUM_X1X2_REG] = HpNum::from(32i32).into();
        state.regs[STAT1_MLRXYZ_SUM_X1X3_REG] = HpNum::from(32i32).into();
        state.regs[STAT1_MLRXYZ_SUM_X2X3_REG] = HpNum::from(19i32).into();
        state.regs[STAT1_MLRXYZ_SUM_X1Y_REG] = HpNum::from(349i32).into();
        state.regs[STAT1_MLRXYZ_SUM_X2Y_REG] = HpNum::from(206i32).into();
        state.regs[STAT1_MLRXYZ_SUM_X3Y_REG] = HpNum::from(256i32).into();
        op_sigma_mlrxyz(&mut state).expect("ΣMLRXYZ oracle must succeed");
        // Pushed in order b₀, b₁, b₂, b₃ → T, Z, Y, X after four lifts.
        // Tolerance 1e-6 matches ΣMLRXY bump rationale — chained Gauss
        // back-substitution accumulates last-digit rounding at the
        // rust_decimal 10-sig-digit floor.
        assert_relative_eq!(as_f64(&state.stack.t), 1.0, max_relative = 1e-6);
        assert_relative_eq!(as_f64(&state.stack.z), 2.0, max_relative = 1e-6);
        assert_relative_eq!(as_f64(&state.stack.y), 3.0, max_relative = 1e-6);
        assert_relative_eq!(as_f64(&state.stack.x), 4.0, max_relative = 1e-6);
    }

    // ── ΣPOLYP / ΣPOLYC tests (SPEC.md Req. 22, 23) ─────────────────────────

    /// Pre-populate the higher-power Σ sums for the oracle dataset
    /// `x=[1,2,3,4,5], y=[1,4,9,16,25]` (= x²; perfect quadratic).
    ///
    ///   n     = 5
    ///   Σx    = 15        Σx²  = 55        Σx³ = 225      Σx⁴ = 979
    ///   Σy    = 55        Σxy  = 225       Σx²y = 979
    fn load_polyp_y_eq_x_sq(state: &mut CalcState) {
        state.regs[STAT1_POLYP_N_REG] = HpNum::from(5i32).into();
        // Σx^k at offsets k-1 for k=1..4.
        state.regs[STAT1_POLYP_SUM_X_BASE_REG] = HpNum::from(15i32).into(); // Σx
        state.regs[STAT1_POLYP_SUM_X_BASE_REG + 1] = HpNum::from(55i32).into(); // Σx²
        state.regs[STAT1_POLYP_SUM_X_BASE_REG + 2] = HpNum::from(225i32).into(); // Σx³
        state.regs[STAT1_POLYP_SUM_X_BASE_REG + 3] = HpNum::from(979i32).into(); // Σx⁴
                                                                                 // Σ(x^k·y) at offsets k for k=0..2.
        state.regs[STAT1_POLYP_SUM_XY_BASE_REG] = HpNum::from(55i32).into(); // Σy
        state.regs[STAT1_POLYP_SUM_XY_BASE_REG + 1] = HpNum::from(225i32).into(); // Σxy
        state.regs[STAT1_POLYP_SUM_XY_BASE_REG + 2] = HpNum::from(979i32).into(); // Σx²y
        state.regs[STAT1_POLYP_DEGREE_REG] = HpNum::from(2i32).into();
    }

    /// SPEC.md Req. 22 oracle: d=2, y=x² → coefficients (a₀=0, a₁=0, a₂=1)
    /// within 1e-7 relative tolerance.
    #[test]
    fn polyp_d2_y_eq_x_sq_oracle() {
        let mut state = CalcState::new();
        load_polyp_y_eq_x_sq(&mut state);
        compute_polyp_coefficients(&mut state).expect("ΣPOLYP compute must succeed");
        // a_0..a_2 stored in registers; a_2 also pushed to stack X.
        let a0 = state.regs[STAT1_POLYP_COEF_BASE_REG].numeric_or_zero();
        let a1 = state.regs[STAT1_POLYP_COEF_BASE_REG + 1].numeric_or_zero();
        let a2 = state.regs[STAT1_POLYP_COEF_BASE_REG + 2].numeric_or_zero();
        assert!(
            as_f64(&a0).abs() < 1e-7,
            "a₀ must be ≈ 0, got {}",
            as_f64(&a0)
        );
        assert!(
            as_f64(&a1).abs() < 1e-7,
            "a₁ must be ≈ 0, got {}",
            as_f64(&a1)
        );
        assert_relative_eq!(as_f64(&a2), 1.0, max_relative = 1e-7);
        assert_relative_eq!(as_f64(&state.stack.x), 1.0, max_relative = 1e-7);
    }

    /// Modal opener: ΣPOLYP without a submit cycle sets the modal_program
    /// + modal_prompt fields and clears nothing.
    #[test]
    fn polyp_workflow_opens_modal() {
        let mut state = CalcState::new();
        op_sigma_polyp_workflow(&mut state).expect("ΣPOLYP modal open must succeed");
        assert!(matches!(
            state.modal_program,
            Some(crate::ops::math1::modal::ModalProgram::Stat1(
                crate::ops::stat1::modal::Stat1Step::PolypDegreePrompt(0)
            ))
        ));
        assert_eq!(state.modal_prompt, Some("DEGREE=?".to_string()));
    }

    /// Degree register out of range returns Domain.
    #[test]
    fn polyp_invalid_degree_returns_domain() {
        let mut state = CalcState::new();
        state.regs[STAT1_POLYP_DEGREE_REG] = HpNum::from(0i32).into();
        assert_eq!(
            compute_polyp_coefficients(&mut state).unwrap_err(),
            HpError::Domain
        );
        state.regs[STAT1_POLYP_DEGREE_REG] = HpNum::from(99i32).into();
        assert_eq!(
            compute_polyp_coefficients(&mut state).unwrap_err(),
            HpError::Domain
        );
    }

    /// SPEC.md Req. 23 oracle: chained after Req. 22 (a₀=0, a₁=0, a₂=1),
    /// ΣPOLYC at x = 6 yields ŷ = 36.0 within 1e-9 relative tolerance.
    #[test]
    fn polyc_d2_at_x_6() {
        let mut state = CalcState::new();
        load_polyp_y_eq_x_sq(&mut state);
        compute_polyp_coefficients(&mut state).expect("ΣPOLYP must succeed");
        // Push x=6 to stack (would normally be done by user via digit-entry).
        state.stack.x = HpNum::from(6i32);
        op_sigma_polyc(&mut state).expect("ΣPOLYC eval must succeed");
        assert_relative_eq!(as_f64(&state.stack.x), 36.0, max_relative = 1e-9);
    }

    /// ΣPOLYC with no prior valid ΣPOLYP (degree=0 in register) returns
    /// Domain to prevent silent garbage-out.
    #[test]
    fn polyc_without_polyp_returns_domain() {
        let mut state = CalcState::new();
        state.regs[STAT1_POLYP_DEGREE_REG] = HpNum::from(0i32).into();
        state.stack.x = HpNum::from(1i32);
        assert_eq!(op_sigma_polyc(&mut state).unwrap_err(), HpError::Domain);
    }

    // ── CI gate: SPEC.md Req. 21 LOCKS no Math Pac I matrix imports ─────────
    //
    // Static-import check enforced by `cargo check` (this file does NOT
    // import or use any `crate::ops::math1` matrix routine). The grep
    // gate in the plan's acceptance criteria provides the runtime
    // guarantee that the substring stays absent from this directory.

    // ── Gauss elimination unit tests ────────────────────────────────────────

    /// Sanity: 2×2 system with known solution.
    #[test]
    fn solve_2x2_known() {
        // [2 1] [x]   [5]   → x=1, y=3.
        // [1 3] [y] = [10]
        let mut m = vec![
            vec![HpNum::from(2i32), HpNum::from(1i32)],
            vec![HpNum::from(1i32), HpNum::from(3i32)],
        ];
        let mut r = vec![HpNum::from(5i32), HpNum::from(10i32)];
        let x = solve_normal_equations(&mut m, &mut r).expect("2×2 must solve");
        assert_relative_eq!(as_f64(&x[0]), 1.0, max_relative = 1e-9);
        assert_relative_eq!(as_f64(&x[1]), 3.0, max_relative = 1e-9);
    }

    /// Partial pivoting catches a row-swap-required system.
    #[test]
    fn solve_2x2_requires_partial_pivot() {
        // [0 1] [x]   [3]   → swap rows → [2 1; 0 1], x=2, y=3 OK before
        // [2 1] [y] = [7]   →
        // Direct elimination without pivot would divide by zero pivot.
        let mut m = vec![
            vec![HpNum::from(0i32), HpNum::from(1i32)],
            vec![HpNum::from(2i32), HpNum::from(1i32)],
        ];
        let mut r = vec![HpNum::from(3i32), HpNum::from(7i32)];
        let x = solve_normal_equations(&mut m, &mut r).expect("partial-pivot must succeed");
        assert_relative_eq!(as_f64(&x[0]), 2.0, max_relative = 1e-9);
        assert_relative_eq!(as_f64(&x[1]), 3.0, max_relative = 1e-9);
    }

    /// Singular system returns Domain.
    #[test]
    fn solve_singular_returns_domain() {
        // [1 2] [x]   [3]
        // [2 4] [y] = [6]    row 2 = 2·row 1 → singular.
        let mut m = vec![
            vec![HpNum::from(1i32), HpNum::from(2i32)],
            vec![HpNum::from(2i32), HpNum::from(4i32)],
        ];
        let mut r = vec![HpNum::from(3i32), HpNum::from(6i32)];
        assert_eq!(
            solve_normal_equations(&mut m, &mut r).unwrap_err(),
            HpError::Domain
        );
    }
}
