// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `stat1::basic_stats` — closed-form univariate extended summaries (Plan 33-05).
//!
//! Ships ΣBSTAT and ΣBSTG: both consume the existing v1.x R01–R06 Σ-register
//! block populated by [`crate::ops::stats::op_sigma_plus`] (D-03 layout).
//! Read-only over R01–R06; no Stat-1 extended-register slots required for
//! these two Ops (the SIZE 012 footprint declared by OM 00041-90030 p. 11
//! is the program's own scratch / intermediate-result allocation, not the
//! data input, mirroring the ΣSPEAR convention from Plan 33-04).
//!
//! ## Semantic split
//!
//! - **ΣBSTAT — univariate extended summary.** Pushes (X-channel) the
//!   coefficient of variation `CV_x = σ_x / μ_x` (sample standard
//!   deviation with Bessel correction divided by sample mean) and
//!   (Y-channel) the sample mean `μ_x`.
//! - **ΣBSTG — bivariate weighted summary.** Treats the v1.x Σ-block as a
//!   (data, weight) pair: Σx = data totals, Σy = weight totals,
//!   Σxy = data⋅weight cross-products. Pushes (X-channel) the weighted
//!   mean `μ_w = Σxy / Σy` (i.e. Σ(x_i·w_i) / Σ(w_i)) and (Y-channel)
//!   the unweighted sample mean `μ_x` for comparison.
//!
//! Both Ops apply [`crate::stack::LiftEffect::Enable`] and use the
//! `enter_number` push-twice pattern from `op_mean` / `op_sdev` in
//! `crate::ops::stats` so that downstream Z, T stack slots are preserved
//! per HP-41 hardware convention.
//!
//! ## SPEC.md oracle drift (resolved in this plan)
//!
//! SPEC.md Req. 7 claims `μ_w = 3.6667` (✓ — matches `Σxy/Σy = 550/150`)
//! AND `CV = 0.4083` for the dataset `x=[1,2,3,4,5], y=[10,20,30,40,50]`.
//! The CV value `0.4083` cannot be reproduced from the standard sample
//! `CV_x = σ_x / μ_x`:
//!
//! ```text
//!   Σx² = 55, Σx = 15, n = 5
//!   σ_x² = (Σx² − (Σx)²/n) / (n − 1) = (55 − 225/5) / 4 = 10/4 = 2.5
//!   σ_x  = √2.5 ≈ 1.5811
//!   μ_x  = Σx/n = 3.0
//!   CV_x = σ_x / μ_x = 0.5270  (NOT 0.4083)
//! ```
//!
//! 0.4083 = `√2.5 / √15 = σ_x / √(Σx)`, which is not a standard CV
//! definition. Manual + `numpy.std(x, ddof=1) / numpy.mean(x)` both
//! confirm 0.5270. The Op ships the mathematically correct 0.5270 value
//! and flags the discrepancy for Phase 35 (STAT-DOC) amendment — same
//! convention as the Plan 33-04 ΣSPEAR `ρ_s = 0.8 vs SPEC = 0.7` drift.
//!
//! ## References
//!
//! - HP-41C Stat 1 Pac Owner's Manual 00041-90030 (1979) §ΣBSTAT, §ΣBSTG
//!   (p. 11–14); program data block at OM Appendix A p. 73.
//! - `numpy.std(ddof=1)` / `numpy.average(weights=...)` oracles for the
//!   inline test tuples (D-33.6 inline-oracle pattern).

use crate::error::HpError;
use crate::num::HpNum;
use crate::ops::stat1::STAT1_MAX_REG;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;

// ── SIZE-floor guard helper ────────────────────────────────────────────────

/// Fail-closed SIZE-floor guard. Returns `Err(HpError::InvalidOp)` if
/// `state.regs` cannot address every slot up to and including
/// `STAT1_MAX_REG` (Plan 33-00 single source of truth; CalcState::new()
/// allocates 100 slots so this is defensive against future SIZE shrink).
#[inline]
fn require_stat1_size_floor(state: &CalcState) -> Result<(), HpError> {
    if state.regs.len() < STAT1_MAX_REG + 1 {
        return Err(HpError::InvalidOp);
    }
    Ok(())
}

// ── ΣBSTAT — Univariate Extended Summary ───────────────────────────────────

/// ΣBSTAT — closed-form univariate extended summary.
///
/// Reads `n = R03`, `Σx = R02`, `Σx² = R01` from the existing v1.x
/// Σ-register block (D-03 layout per `crate::ops::stats`). Computes
/// the sample mean `μ_x = Σx / n` and the Bessel-corrected sample
/// standard deviation `σ_x = √((Σx² − (Σx)²/n) / (n − 1))`. Pushes
/// the coefficient of variation `CV_x = σ_x / μ_x` to stack X with
/// `LiftEffect::Enable`, and `μ_x` to Y immediately above.
///
/// Pre-condition (per OM p. 11): the user has accumulated each data
/// point via Σ+ before calling ΣBSTAT.
///
/// # Errors
///
/// - `HpError::InvalidOp` on SIZE-floor guard.
/// - `HpError::InvalidOp` if `n < 2` (variance undefined for n ≤ 1).
/// - `HpError::InvalidOp` if `μ_x == 0` (CV undefined for zero mean).
/// - `HpError::Domain` if `σ_x²` is negative (degenerate accumulator
///   state; propagated by `checked_sqrt`).
///
/// # Source
///
/// HP-41C Stat 1 Pac Owner's Manual 00041-90030 §ΣBSTAT (p. 11). See
/// module-level docs for the documented SPEC.md Req. 7 oracle drift.
pub fn op_sigma_bstat(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;

    // v1.x R01–R06 exemption (REVIEW.md WR-02): R01=Σx², R02=Σx, R03=n
    // per the canonical v1.x Σ-block layout documented in
    // `ops/stats.rs`. Stat-1-specific slots ≥ R07 are accessed via
    // named consts (P21); the foundational v1.x block uses literal
    // indices for symmetry with `op_sigma_plus`.
    let sum_x_sq = state.regs[1].clone();
    let sum_x = state.regs[2].clone();
    let n = state.regs[3].clone();

    // n < 2 → variance undefined (n − 1 == 0 would divide by zero).
    let one = HpNum::from(1i32);
    let n_minus_one = n.checked_sub(&one)?;
    if n_minus_one.inner() <= rust_decimal::Decimal::ZERO {
        return Err(HpError::InvalidOp);
    }

    // μ_x = Σx / n
    let mu_x = sum_x.checked_div(&n)?;
    if mu_x.is_zero() {
        return Err(HpError::InvalidOp); // CV undefined for zero mean
    }

    // σ_x² = (Σx² − (Σx)²/n) / (n − 1)
    let sum_x_squared_over_n = sum_x.checked_sq()?.checked_div(&n)?;
    let var_num = sum_x_sq.checked_sub(&sum_x_squared_over_n)?;
    let variance = var_num.checked_div(&n_minus_one)?;
    let sigma_x = variance.checked_sqrt()?;

    // CV_x = σ_x / μ_x
    let cv = sigma_x.checked_div(&mu_x)?;

    // Push μ_x first (lands at Y after next push), then CV (lands at X).
    state.stack.lift_enabled = true;
    enter_number(state, mu_x);
    apply_lift_effect(state, LiftEffect::Enable);
    state.stack.lift_enabled = true;
    enter_number(state, cv);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

// ── ΣBSTG — Bivariate Weighted Summary ─────────────────────────────────────

/// ΣBSTG — closed-form bivariate weighted summary.
///
/// Treats the v1.x R01–R06 Σ-block as (data, weight) pairs: each Σ+
/// call accumulates one (x_i, w_i) pair so that R06 = Σ(x_i·w_i)
/// (cross-products), R05 = Σw_i (weight totals), and R02 = Σx_i
/// (data totals). Computes:
///
/// - Weighted mean `μ_w = Σ(x_i·w_i) / Σw_i = R06 / R05`
/// - Unweighted mean `μ_x = Σx_i / n = R02 / R03` (for comparison)
///
/// Pushes `μ_x` first (lands at Y) then `μ_w` (lands at X) with
/// `LiftEffect::Enable`, mirroring the `op_mean` push-twice pattern in
/// `crate::ops::stats`.
///
/// # Errors
///
/// - `HpError::InvalidOp` on SIZE-floor guard.
/// - `HpError::InvalidOp` if `n == 0` (μ_x undefined).
/// - `HpError::InvalidOp` if `Σw == 0` (weighted mean undefined —
///   degenerate weights).
///
/// # Source
///
/// HP-41C Stat 1 Pac Owner's Manual 00041-90030 §ΣBSTG (p. 11–14).
/// Manual derivation cross-checked against `numpy.average(x, weights=w)`.
pub fn op_sigma_bstg(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;

    // v1.x R01–R06 exemption (REVIEW.md WR-02): R02=Σx (data totals),
    // R03=n, R05=Σy (reinterpreted as weight totals Σw), R06=Σxy
    // (reinterpreted as Σx·w cross-products). The v1.x layout is the
    // canonical source; literal indices preserve symmetry with
    // `op_sigma_plus`. Stat-1-specific slots ≥ R07 still route through
    // named consts (P21).
    let sum_x = state.regs[2].clone();
    let n = state.regs[3].clone();
    let sum_w = state.regs[5].clone(); // Σy in v1.x layout — interpreted as weight sum
    let sum_xw = state.regs[6].clone(); // Σxy in v1.x layout — interpreted as Σx·w

    if n.is_zero() {
        return Err(HpError::InvalidOp);
    }
    if sum_w.is_zero() {
        return Err(HpError::InvalidOp);
    }

    // Unweighted mean μ_x = Σx / n
    let mu_x = sum_x.checked_div(&n)?;
    // Weighted mean μ_w = Σ(x·w) / Σw
    let mu_w = sum_xw.checked_div(&sum_w)?;

    // Push μ_x first (lands at Y after next push), then μ_w (lands at X).
    state.stack.lift_enabled = true;
    enter_number(state, mu_x);
    apply_lift_effect(state, LiftEffect::Enable);
    state.stack.lift_enabled = true;
    enter_number(state, mu_w);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use rust_decimal::prelude::ToPrimitive;

    /// Helper: convert an `HpNum` to `f64` for `assert_relative_eq!`.
    fn as_f64(n: &HpNum) -> f64 {
        n.inner()
            .to_f64()
            .expect("HpNum→f64 must succeed for finite oracle values")
    }

    /// Helper: load the v1.x Σ-block with the SPEC.md Req. 7 dataset.
    /// `x = [1,2,3,4,5]`, `y = [10,20,30,40,50]`:
    ///   n = 5
    ///   Σx = 15, Σx² = 55
    ///   Σy = 150, Σy² = 1·100 + 4·100 + 9·100 + 16·100 + 25·100 = ... ;
    ///        but for Σy² (R04) we need Σ(y_i²) = 100+400+900+1600+2500 = 5500
    ///   Σxy = 1·10 + 2·20 + 3·30 + 4·40 + 5·50 = 10+40+90+160+250 = 550
    fn load_spec_req7_dataset(state: &mut CalcState) {
        state.regs[1] = HpNum::from(55i32); // Σx²
        state.regs[2] = HpNum::from(15i32); // Σx
        state.regs[3] = HpNum::from(5i32); // n
        state.regs[4] = HpNum::from(5500i32); // Σy²
        state.regs[5] = HpNum::from(150i32); // Σy
        state.regs[6] = HpNum::from(550i32); // Σxy
    }

    // ── ΣBSTAT tests ────────────────────────────────────────────────────────

    /// SPEC.md Req. 7 dataset:
    /// - μ_x = 15 / 5 = 3.0
    /// - σ_x = √((55 − 225/5)/(5−1)) = √(10/4) = √2.5 ≈ 1.5811388300841898
    /// - CV_x = σ_x / μ_x ≈ 0.5270462766947299
    ///
    /// SPEC.md states `CV = 0.4083` — that value is WRONG. Manual +
    /// `numpy.std([1,2,3,4,5], ddof=1) / numpy.mean([1,2,3,4,5])`
    /// confirm `0.5270462766947299`. SPEC.md amendment is gated to
    /// Phase 35 (STAT-DOC); discrepancy documented at module level.
    #[test]
    fn bstat_spec_req7_corrected_oracle() {
        let mut state = CalcState::new();
        load_spec_req7_dataset(&mut state);
        op_sigma_bstat(&mut state).expect("ΣBSTAT with valid n=5 must succeed");
        // X = CV_x ≈ 0.5270
        assert_relative_eq!(
            as_f64(&state.stack.x),
            0.5270462766947299,
            max_relative = 1e-9
        );
        // Y = μ_x = 3.0
        assert_relative_eq!(as_f64(&state.stack.y), 3.0, max_relative = 1e-9);
    }

    /// Simple identity dataset: x=[1,2,3,4,5], μ=3, σ=√2.5, CV=0.5270.
    /// numpy.std([1,2,3,4,5], ddof=1) = 1.5811388300841898
    /// numpy.mean([1,2,3,4,5]) = 3.0
    /// CV = 1.5811388300841898 / 3.0 = 0.5270462766947299
    #[test]
    fn bstat_arithmetic_sequence() {
        let mut state = CalcState::new();
        state.regs[1] = HpNum::from(55i32); // Σx² = 1+4+9+16+25
        state.regs[2] = HpNum::from(15i32); // Σx
        state.regs[3] = HpNum::from(5i32); // n
        op_sigma_bstat(&mut state).unwrap();
        assert_relative_eq!(
            as_f64(&state.stack.x),
            0.5270462766947299,
            max_relative = 1e-9
        );
        assert_relative_eq!(as_f64(&state.stack.y), 3.0, max_relative = 1e-9);
    }

    /// Constant data σ = 0 → CV = 0.
    /// x = [4, 4, 4], μ = 4, σ = 0, CV = 0.
    /// Σx = 12, Σx² = 48, n = 3.
    #[test]
    fn bstat_constant_data_zero_cv() {
        let mut state = CalcState::new();
        state.regs[1] = HpNum::from(48i32);
        state.regs[2] = HpNum::from(12i32);
        state.regs[3] = HpNum::from(3i32);
        op_sigma_bstat(&mut state).unwrap();
        assert_relative_eq!(as_f64(&state.stack.x), 0.0, epsilon = 1e-12);
        assert_relative_eq!(as_f64(&state.stack.y), 4.0, max_relative = 1e-9);
    }

    /// Two-point dataset: x = [2, 4], μ = 3, σ = √2 ≈ 1.4142, CV ≈ 0.4714.
    /// numpy.std([2,4], ddof=1) = 1.4142135623730951
    /// CV = 1.4142135623730951 / 3.0 = 0.4714045207910317
    #[test]
    fn bstat_two_point_dataset() {
        let mut state = CalcState::new();
        state.regs[1] = HpNum::from(20i32); // 4 + 16
        state.regs[2] = HpNum::from(6i32); // 2 + 4
        state.regs[3] = HpNum::from(2i32);
        op_sigma_bstat(&mut state).unwrap();
        assert_relative_eq!(
            as_f64(&state.stack.x),
            0.4714045207910317,
            max_relative = 1e-9
        );
        assert_relative_eq!(as_f64(&state.stack.y), 3.0, max_relative = 1e-9);
    }

    /// SIZE-floor guard fires when `state.regs.len() < STAT1_MAX_REG + 1`.
    #[test]
    fn bstat_size_floor_guard() {
        let mut state = CalcState::new();
        state.regs.truncate(STAT1_MAX_REG); // one too few
        assert_eq!(op_sigma_bstat(&mut state).unwrap_err(), HpError::InvalidOp);
    }

    /// n < 2 returns InvalidOp (Bessel-corrected variance undefined).
    #[test]
    fn bstat_n_too_small() {
        let mut state = CalcState::new();
        state.regs[1] = HpNum::from(1i32);
        state.regs[2] = HpNum::from(1i32);
        state.regs[3] = HpNum::from(1i32);
        assert_eq!(op_sigma_bstat(&mut state).unwrap_err(), HpError::InvalidOp);
    }

    /// μ_x == 0 returns InvalidOp (CV undefined — division by zero).
    /// x = [-1, 1] → Σx = 0, μ = 0.
    #[test]
    fn bstat_zero_mean_returns_invalid() {
        let mut state = CalcState::new();
        state.regs[1] = HpNum::from(2i32); // 1 + 1
        state.regs[2] = HpNum::zero(); // -1 + 1 = 0
        state.regs[3] = HpNum::from(2i32);
        assert_eq!(op_sigma_bstat(&mut state).unwrap_err(), HpError::InvalidOp);
    }

    // ── ΣBSTG tests ─────────────────────────────────────────────────────────

    /// SPEC.md Req. 7 dataset, ΣBSTG branch:
    /// - μ_x = 15/5 = 3.0
    /// - μ_w = Σ(x·w) / Σw = 550/150 = 3.6666666666...
    ///
    /// `numpy.average([1,2,3,4,5], weights=[10,20,30,40,50])`
    ///   returns `3.6666666666666665`.
    /// `numpy.mean([1,2,3,4,5])` returns `3.0`.
    #[test]
    fn bstg_spec_req7_weighted_mean() {
        let mut state = CalcState::new();
        load_spec_req7_dataset(&mut state);
        op_sigma_bstg(&mut state).expect("ΣBSTG with valid n=5 must succeed");
        // X = μ_w = 550 / 150 ≈ 3.6667
        assert_relative_eq!(
            as_f64(&state.stack.x),
            3.6666666666666665,
            max_relative = 1e-9
        );
        // Y = μ_x = 15 / 5 = 3.0
        assert_relative_eq!(as_f64(&state.stack.y), 3.0, max_relative = 1e-9);
    }

    /// Equal weights → weighted mean = unweighted mean.
    /// x = [1, 2, 3], w = [1, 1, 1]:
    ///   Σx = 6, n = 3, Σw = 3, Σ(x·w) = 1+2+3 = 6.
    ///   μ_x = 6/3 = 2.0; μ_w = 6/3 = 2.0.
    #[test]
    fn bstg_equal_weights() {
        let mut state = CalcState::new();
        state.regs[2] = HpNum::from(6i32); // Σx
        state.regs[3] = HpNum::from(3i32); // n
        state.regs[5] = HpNum::from(3i32); // Σw
        state.regs[6] = HpNum::from(6i32); // Σxw
        op_sigma_bstg(&mut state).unwrap();
        assert_relative_eq!(as_f64(&state.stack.x), 2.0, max_relative = 1e-9);
        assert_relative_eq!(as_f64(&state.stack.y), 2.0, max_relative = 1e-9);
    }

    /// Skewed weights drive μ_w toward the heavily-weighted element.
    /// x = [1, 100], w = [99, 1]:
    ///   Σx = 101, n = 2, Σw = 100, Σ(x·w) = 99 + 100 = 199.
    ///   μ_w = 199 / 100 = 1.99 (close to 1, pulled toward heavy weight).
    ///   μ_x = 101 / 2 = 50.5.
    /// `numpy.average([1,100], weights=[99,1])` = 1.99.
    #[test]
    fn bstg_skewed_weights() {
        let mut state = CalcState::new();
        state.regs[2] = HpNum::from(101i32);
        state.regs[3] = HpNum::from(2i32);
        state.regs[5] = HpNum::from(100i32);
        state.regs[6] = HpNum::from(199i32);
        op_sigma_bstg(&mut state).unwrap();
        assert_relative_eq!(as_f64(&state.stack.x), 1.99, max_relative = 1e-9);
        assert_relative_eq!(as_f64(&state.stack.y), 50.5, max_relative = 1e-9);
    }

    /// Single (x, w) pair: μ_x = x; μ_w = (x·w)/w = x. Both equal.
    /// x = [7], w = [3]: Σx = 7, n = 1, Σw = 3, Σxw = 21.
    /// μ_x = 7/1 = 7; μ_w = 21/3 = 7.
    #[test]
    fn bstg_single_pair() {
        let mut state = CalcState::new();
        state.regs[2] = HpNum::from(7i32);
        state.regs[3] = HpNum::from(1i32);
        state.regs[5] = HpNum::from(3i32);
        state.regs[6] = HpNum::from(21i32);
        op_sigma_bstg(&mut state).unwrap();
        assert_relative_eq!(as_f64(&state.stack.x), 7.0, max_relative = 1e-9);
        assert_relative_eq!(as_f64(&state.stack.y), 7.0, max_relative = 1e-9);
    }

    /// SIZE-floor guard fires.
    #[test]
    fn bstg_size_floor_guard() {
        let mut state = CalcState::new();
        state.regs.truncate(STAT1_MAX_REG);
        assert_eq!(op_sigma_bstg(&mut state).unwrap_err(), HpError::InvalidOp);
    }

    /// n == 0 returns InvalidOp.
    #[test]
    fn bstg_zero_n_returns_invalid() {
        let mut state = CalcState::new();
        state.regs[5] = HpNum::from(1i32); // Σw nonzero to avoid that branch
                                           // Σx and n both zero (default).
        assert_eq!(op_sigma_bstg(&mut state).unwrap_err(), HpError::InvalidOp);
    }

    /// Σw == 0 returns InvalidOp (weighted mean undefined).
    #[test]
    fn bstg_zero_weight_sum_returns_invalid() {
        let mut state = CalcState::new();
        state.regs[2] = HpNum::from(3i32);
        state.regs[3] = HpNum::from(2i32);
        // Σw and Σxw both zero (default).
        assert_eq!(op_sigma_bstg(&mut state).unwrap_err(), HpError::InvalidOp);
    }
}
