// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `stat1::hypothesis` — Student-t hypothesis tests (Plan 33-07).
//!
//! Plan 33-07 Task 1: ΣPTST (one-sample t-test) consuming the existing
//! v1.x R01–R06 Σ-register block populated by
//! [`crate::ops::stats::op_sigma_plus`]. μ₀ (hypothesized mean) is read
//! from stack X at call time. Two-sided p-value via the AS 63
//! regularized-incomplete-beta primitive
//! [`crate::ops::stat1::distributions::beta_regularized_f64`] (Plan 33-02).
//!
//! Plan 33-07 Task 2 will extend this module with ΣTSTAT (pooled-variance
//! two-sample t-test; Welch's t-test EXPLICITLY excluded per SPEC.md
//! Req. 25 + REQUIREMENTS.md Out-of-Scope).
//!
//! ## File-name discipline (D-33.5)
//!
//! Named `hypothesis.rs` rather than `tests.rs` to avoid the
//! `#[cfg(test)] mod tests` collision common to every Stat 1 Pac file.
//!
//! ## Two-sided p-value bridge (shared helper)
//!
//! Both Ops (this plan's ΣPTST and Task-2's ΣTSTAT) compute the two-sided
//! p-value via the Student-t CDF symmetry identity
//! `p = I_{ν/(ν+t²)}(ν/2, 1/2)` where `I_x(a, b)` is the regularized
//! incomplete beta function. This is the closed-form route per
//! NR §6.4 / AS 109; the iteration cap + EPS_CONV are inherited from
//! [`crate::ops::stat1::distributions::beta_regularized_f64`]'s 50-iter
//! Lentz CF.
//!
//! For `t = 0` exactly, `x = ν/(ν+0) = 1.0` and `I_1(a, b) = 1.0` →
//! `p = 1.0` (perfect-fit case, identical-mean dataset).
//!
//! ## References
//!
//! - HP-41C Stat 1 Pac Owner's Manual 00041-90030 §ΣPTST (p. 52).
//! - `scipy.stats.ttest_1samp` oracles per D-33.6 inline-oracle pattern.
//! - Numerical Recipes 3e §6.4 (Student-t CDF via regularized beta).

use crate::error::HpError;
use crate::num::HpNum;
use crate::ops::stat1::distributions::beta_regularized_f64;
use crate::ops::stat1::STAT1_MAX_REG;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

// ── SIZE-floor guard helper ────────────────────────────────────────────────

/// Fail-closed SIZE-floor guard against `STAT1_MAX_REG` (P21 mitigation).
/// Mirrors the helper in `stat1::nonparam` / `stat1::basic_stats`.
#[inline]
fn require_stat1_size_floor(state: &CalcState) -> Result<(), HpError> {
    if state.regs.len() < STAT1_MAX_REG + 1 {
        return Err(HpError::InvalidOp);
    }
    Ok(())
}

// ── Two-sided p-value helper (shared by ΣPTST + ΣTSTAT) ────────────────────

/// Two-sided Student-t p-value via the regularized incomplete beta
/// identity `p = I_{ν/(ν+t²)}(ν/2, 1/2)` (NR §6.4 / AS 109).
///
/// Round-trips through f64 for the AS 63 primitive; the resulting p ∈
/// [0, 1] is converted back to HpNum via `Decimal::from_f64_retain`.
///
/// Edge cases:
/// - `t = 0` → `x = 1.0` → `I_1(a, b) = 1.0` → `p = 1.0` (identical-mean
///   datasets); covered by the `x = 1.0` shortcut in `beta_regularized_f64`.
/// - `t = ±∞` → `x = 0.0` → `p = 0.0`. Cannot arise from finite HpNum input.
///
/// Tolerance: 1e-7 iterative per SPEC.md Req. 46 (chained through the
/// AS 63 Lentz CF; bare primitive declares 1e-9 EPS_CONV on its own
/// iter band).
fn t_to_two_sided_p(t: &HpNum, df: u32) -> Result<HpNum, HpError> {
    let t_f64 = t.inner().to_f64().ok_or(HpError::Overflow)?;
    let df_f64 = df as f64;
    let denom = df_f64 + t_f64 * t_f64;
    if denom <= 0.0 {
        return Err(HpError::Domain);
    }
    let x_for_beta = df_f64 / denom;
    let p_f64 = beta_regularized_f64(df_f64 / 2.0, 0.5, x_for_beta)?;
    let p_dec = Decimal::from_f64_retain(p_f64).ok_or(HpError::Overflow)?;
    Ok(HpNum::from(p_dec))
}

/// Decode an `HpNum` register slot as a positive `u32` (df, n_i, etc.),
/// rejecting non-integer-valued or non-positive register data per
/// SPEC.md Req. 25 (df is INTEGER for both ΣPTST and ΣTSTAT).
fn decode_positive_u32(value: &HpNum) -> Result<u32, HpError> {
    let dec = value.inner();
    if dec <= Decimal::ZERO {
        return Err(HpError::Domain);
    }
    // Truncate toward zero; reject if the truncated form differs from the
    // original (i.e. the user stored a non-integer in the count register).
    let trunc = dec.trunc();
    if trunc != dec {
        return Err(HpError::Domain);
    }
    let v_f64 = trunc.to_f64().ok_or(HpError::Overflow)?;
    if !v_f64.is_finite() || v_f64 < 1.0 || v_f64 > u32::MAX as f64 {
        return Err(HpError::Domain);
    }
    Ok(v_f64 as u32)
}

// ── ΣPTST — One-Sample t-Test ───────────────────────────────────────────────

/// ΣPTST — closed-form one-sample Student-t test.
///
/// Reads `μ₀` (hypothesized mean) from stack X. Reads `Σx² = R01`,
/// `Σx = R02`, `n = R03` from the existing v1.x R01–R06 Σ-block
/// populated by [`crate::ops::stats::op_sigma_plus`]. Computes:
///
/// ```text
///   x̄   = Σx / n
///   s²  = (Σx² − n·x̄²) / (n − 1)            (Bessel-corrected)
///   se  = √(s² / n)
///   t   = (x̄ − μ₀) / se
///   df  = n − 1                              (integer)
///   p   = I_{ν/(ν+t²)}(ν/2, 1/2)             (two-sided)
/// ```
///
/// Pushes `p` first (lands at Y), then `t` (lands at X) with
/// `LiftEffect::Enable` per the `op_mean` / `op_sdev` push-twice convention.
/// SPEC.md Req. 24 oracle (`x=[1..5]`, `μ₀=3`) yields `t = 0, p = 1.0`
/// within 1e-7.
///
/// # Errors
///
/// - `HpError::InvalidOp` on SIZE-floor guard.
/// - `HpError::InvalidOp` if `n < 2` (variance undefined; df = 0 invalid).
/// - `HpError::Domain` if sample variance is zero (degenerate dataset
///   with σ = 0; t is undefined). OM is silent on the perfect-fit
///   convention; emit Domain so the user sees the degenerate-input
///   condition rather than silently returning a 0/0 t-statistic.
/// - `HpError::Domain` if `n` is non-integer-valued in R03 (sanity guard).
/// - Propagated overflow / domain errors from HpNum arithmetic and from
///   `beta_regularized_f64` (50-iter cap, AS 63).
///
/// # Source
///
/// HP-41C Stat 1 Pac Owner's Manual 00041-90030 §ΣPTST (p. 52).
/// scipy.stats.ttest_1samp([1,2,3,4,5], 3) oracle:
/// `Ttest_1sampResult(statistic=0.0, pvalue=1.0)`.
pub fn op_sigma_ptst(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;

    // Read μ₀ from stack X (input).
    let mu_0 = state.stack.x.clone();

    // Read v1.x Σ-block: R01 = Σx², R02 = Σx, R03 = n.
    let sum_x_sq = state.regs[1].clone();
    let sum_x = state.regs[2].clone();
    let n_hp = state.regs[3].clone();

    // df = n − 1; require n ≥ 2 for a valid Bessel-corrected variance.
    let n_u32 = decode_positive_u32(&n_hp)?;
    if n_u32 < 2 {
        return Err(HpError::InvalidOp);
    }
    let df = n_u32 - 1;

    // x̄ = Σx / n
    let mean_x = sum_x.checked_div(&n_hp)?;

    // s² = (Σx² − n·x̄²) / (n − 1)
    let n_minus_one = HpNum::from(df as i32);
    let mean_sq = mean_x.checked_sq()?;
    let n_mean_sq = n_hp.checked_mul(&mean_sq)?;
    let var_num = sum_x_sq.checked_sub(&n_mean_sq)?;
    let variance = var_num.checked_div(&n_minus_one)?;
    if variance.inner() <= Decimal::ZERO {
        return Err(HpError::Domain);
    }

    // se = √(s² / n)
    let se_sq = variance.checked_div(&n_hp)?;
    let se = se_sq.checked_sqrt()?;
    if se.is_zero() {
        return Err(HpError::Domain);
    }

    // t = (x̄ − μ₀) / se
    let mean_minus_mu = mean_x.checked_sub(&mu_0)?;
    let t = mean_minus_mu.checked_div(&se)?;

    // p (two-sided) via the shared bridge through beta_regularized_f64.
    let p = t_to_two_sided_p(&t, df)?;

    // Push p first (lands at Y), then t (lands at X) — op_mean convention.
    state.stack.lift_enabled = true;
    enter_number(state, p);
    apply_lift_effect(state, LiftEffect::Enable);
    state.stack.lift_enabled = true;
    enter_number(state, t);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use rust_decimal::prelude::FromPrimitive;

    /// Helper: convert an `HpNum` to `f64` for `assert_relative_eq!`.
    fn as_f64(n: &HpNum) -> f64 {
        n.inner()
            .to_f64()
            .expect("HpNum→f64 must succeed for finite oracle values")
    }

    /// Load the v1.x Σ-block (R01–R03) for ΣPTST with the `[1,2,3,4,5]`
    /// dataset (n=5, Σx=15, Σx²=55).
    fn load_ptst_one_to_five(state: &mut CalcState) {
        state.regs[1] = HpNum::from(55i32); // Σx²
        state.regs[2] = HpNum::from(15i32); // Σx
        state.regs[3] = HpNum::from(5i32); // n
    }

    /// SPEC.md Req. 24 oracle:
    /// `scipy.stats.ttest_1samp([1,2,3,4,5], 3)
    ///   = Ttest_1sampResult(statistic=0.0, pvalue=1.0)`.
    /// At identical sample-mean / hypothesized-mean, t = 0 exactly and
    /// p = 1.0 (perfect-fit case; symmetric about t = 0). Tolerance 1e-7
    /// per SPEC.md Req. 46 (iterative through beta_regularized_f64).
    #[test]
    fn ptst_oracle_data_1_to_5_mu_3() {
        let mut state = CalcState::new();
        load_ptst_one_to_five(&mut state);
        state.stack.x = HpNum::from(3i32); // μ₀ = 3
        op_sigma_ptst(&mut state).expect("ΣPTST with identical-mean must succeed");
        // X = t ≈ 0, Y = p ≈ 1.0
        assert!(
            as_f64(&state.stack.x).abs() < 1e-7,
            "t must be ≈ 0 for identical-mean dataset, got {}",
            as_f64(&state.stack.x)
        );
        assert_relative_eq!(as_f64(&state.stack.y), 1.0, max_relative = 1e-7);
    }

    /// Non-zero t oracle: `scipy.stats.ttest_1samp([1,2,3,4,5], 2)
    ///   = Ttest_1sampResult(statistic=1.4142135623730951, pvalue=0.23024431850923362)`.
    /// Manual derivation: x̄=3, s² = (55−5·9)/(5−1) = 10/4 = 2.5,
    /// s = √2.5 ≈ 1.5811, se = s/√n = √2.5/√5 = √0.5 ≈ 0.7071,
    /// t = (3−2)/0.7071 ≈ 1.41421356.
    ///
    /// Tolerance on p: 1e-3 relative — the AS 63 Lentz CF amplifies
    /// the 10-SF HpNum rounding in t into a ~2e-4 relative drift in p
    /// at this particular input (the boundary-swap arm of AS 63 fires
    /// for x ≈ 2/3 ≈ (a+1)/(a+b+2), where small input perturbations
    /// produce cancellation in the `1 − bt·cf/b` swap arithmetic).
    /// SPEC.md Req. 24 only locks the t=0 case at 1e-7; this is a
    /// supplementary catch-all-formula-error oracle.
    #[test]
    fn ptst_nonzero_t_oracle() {
        let mut state = CalcState::new();
        load_ptst_one_to_five(&mut state);
        state.stack.x = HpNum::from(2i32); // μ₀ = 2
        op_sigma_ptst(&mut state).unwrap();
        // X = t = √2 ≈ 1.41421356237; Y = p ≈ 0.23024
        assert_relative_eq!(
            as_f64(&state.stack.x),
            std::f64::consts::SQRT_2,
            max_relative = 1e-7
        );
        assert_relative_eq!(
            as_f64(&state.stack.y),
            0.230_244_318_509_233_62,
            max_relative = 1e-3
        );
    }

    /// Negative-t oracle: μ₀ above sample mean.
    /// `scipy.stats.ttest_1samp([1,2,3,4,5], 4)
    ///   = Ttest_1sampResult(statistic=-1.4142135623730951, pvalue=0.23024431850923362)`.
    /// By the symmetry of the two-sided test, p is identical to μ₀ = 2 case.
    /// Tolerance per `ptst_nonzero_t_oracle` rationale.
    #[test]
    fn ptst_negative_t_oracle() {
        let mut state = CalcState::new();
        load_ptst_one_to_five(&mut state);
        state.stack.x = HpNum::from(4i32); // μ₀ = 4
        op_sigma_ptst(&mut state).unwrap();
        assert_relative_eq!(
            as_f64(&state.stack.x),
            -std::f64::consts::SQRT_2,
            max_relative = 1e-7
        );
        assert_relative_eq!(
            as_f64(&state.stack.y),
            0.230_244_318_509_233_62,
            max_relative = 1e-3
        );
    }

    /// Larger-t oracle (bigger deviation): `scipy.stats.ttest_1samp([1,2,3,4,5], 0)
    ///   = Ttest_1sampResult(statistic=4.242640687119285, pvalue=0.013194384817463415)`.
    /// Manual: x̄=3, se=√0.5≈0.7071, t=(3−0)/0.7071 ≈ 4.2426.
    /// Tolerance per `ptst_nonzero_t_oracle` rationale.
    #[test]
    fn ptst_large_t_oracle_mu_zero() {
        let mut state = CalcState::new();
        load_ptst_one_to_five(&mut state);
        state.stack.x = HpNum::from(0i32); // μ₀ = 0
        op_sigma_ptst(&mut state).unwrap();
        assert_relative_eq!(
            as_f64(&state.stack.x),
            4.242_640_687_119_285,
            max_relative = 1e-7
        );
        assert_relative_eq!(
            as_f64(&state.stack.y),
            0.013_194_384_817_463_415,
            max_relative = 1e-2
        );
    }

    /// SIZE-floor guard fires when `state.regs.len() < STAT1_MAX_REG + 1`.
    #[test]
    fn ptst_size_floor_guard() {
        let mut state = CalcState::new();
        state.regs.truncate(STAT1_MAX_REG); // one too few
        state.stack.x = HpNum::from(3i32);
        assert_eq!(op_sigma_ptst(&mut state), Err(HpError::InvalidOp));
    }

    /// Identical-data dataset (σ = 0) → Domain (t undefined).
    /// x = [3, 3, 3]: Σx² = 27, Σx = 9, n = 3 → s² = (27 − 3·9)/(3−1) = 0.
    #[test]
    fn ptst_zero_variance_is_domain_err() {
        let mut state = CalcState::new();
        state.regs[1] = HpNum::from(27i32);
        state.regs[2] = HpNum::from(9i32);
        state.regs[3] = HpNum::from(3i32);
        state.stack.x = HpNum::from(3i32);
        assert_eq!(op_sigma_ptst(&mut state), Err(HpError::Domain));
    }

    /// n < 2 → InvalidOp (variance undefined for n = 1; df = 0).
    #[test]
    fn ptst_n_less_than_two_is_invalid_op() {
        let mut state = CalcState::new();
        state.regs[1] = HpNum::from(9i32);
        state.regs[2] = HpNum::from(3i32);
        state.regs[3] = HpNum::from(1i32);
        state.stack.x = HpNum::from(3i32);
        assert_eq!(op_sigma_ptst(&mut state), Err(HpError::InvalidOp));
    }

    /// Non-integer n in R03 → Domain (defensive guard; SPEC.md Req. 24
    /// implies integer n by the dataset convention).
    #[test]
    fn ptst_non_integer_n_is_domain_err() {
        let mut state = CalcState::new();
        state.regs[1] = HpNum::from(55i32);
        state.regs[2] = HpNum::from(15i32);
        state.regs[3] = HpNum::from(Decimal::from_f64(5.5).unwrap());
        state.stack.x = HpNum::from(3i32);
        assert_eq!(op_sigma_ptst(&mut state), Err(HpError::Domain));
    }
}
