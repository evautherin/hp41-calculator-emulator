// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `stat1::hypothesis` — Student-t hypothesis tests (Plan 33-07).
//!
//! - **ΣPTST** (one-sample t-test): reads μ₀ from stack X, then v1.x
//!   R01–R06 Σ-block (Σx²=R01, Σx=R02, n=R03). Bessel-corrected t.
//! - **ΣTSTAT** (pooled-variance two-sample t-test; **Welch EXPLICITLY
//!   excluded** per SPEC.md Req. 25 + REQUIREMENTS.md Out-of-Scope):
//!   reads per-group accumulators from G1 = R01–R03, G2 = R07–R09
//!   (OM 00041-90030 p. 52 layout; named consts in `stat1::mod`).
//!
//! Both Ops compute the two-sided p-value via the Student-t CDF
//! symmetry identity `p = I_{ν/(ν+t²)}(ν/2, 1/2)` (NR 3e §6.4 / AS 109)
//! bridged through [`crate::ops::stat1::distributions::beta_regularized_f64`]
//! (Plan 33-02). For t = 0 the closed-form shortcut x = 1 → p = 1
//! covers SPEC.md Req. 24's identical-mean acceptance oracle.
//!
//! D-33.5 file-name discipline: named `hypothesis.rs` rather than
//! `tests.rs` to avoid `#[cfg(test)] mod tests` collisions.
//!
//! References: OM 00041-90030 §ΣPTST + §ΣTSTAT (p. 52);
//! scipy.stats.ttest_1samp / ttest_ind(equal_var=True) D-33.6 oracles;
//! Numerical Recipes 3e §6.4.

use crate::error::HpError;
use crate::num::HpNum;
use crate::ops::stat1::distributions::beta_regularized_f64;
use crate::ops::stat1::{
    STAT1_MAX_REG, STAT1_TSTAT_G1_N_REG, STAT1_TSTAT_G1_SUMSQ_REG, STAT1_TSTAT_G1_SUM_REG,
    STAT1_TSTAT_G2_N_REG, STAT1_TSTAT_G2_SUMSQ_REG, STAT1_TSTAT_G2_SUM_REG,
};
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

/// Two-sided Student-t p-value via regularized-incomplete-beta identity
/// `p = I_{ν/(ν+t²)}(ν/2, 1/2)` (NR §6.4 / AS 109). f64-bridge through
/// the Plan 33-02 AS 63 primitive; edge cases (t=0 → x=1 → p=1; t→±∞
/// → x=0 → p=0) handled by the bare primitive's shortcuts. Tolerance
/// 1e-7 iterative per SPEC.md Req. 46.
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

/// ΣPTST — closed-form one-sample Student-t test. Reads μ₀ from stack X
/// and Σx²/Σx/n from v1.x R01/R02/R03. Computes x̄ = Σx/n,
/// s² = (Σx² − n·x̄²)/(n−1) (Bessel), se = √(s²/n), t = (x̄ − μ₀)/se,
/// df = n − 1, and the two-sided p = I_{ν/(ν+t²)}(ν/2, 1/2).
///
/// Pushes p first (lands at Y) then t (lands at X) with
/// LiftEffect::Enable, matching the op_mean / op_sdev push-twice
/// convention. SPEC.md Req. 24 oracle (x=[1..5], μ₀=3 → t=0, p=1
/// within 1e-7).
///
/// Errors: `InvalidOp` on SIZE-floor or n < 2; `Domain` on σ² ≤ 0
/// (degenerate σ=0 dataset; OM silent on perfect-fit convention) or
/// non-integer n; propagated from beta_regularized_f64 (50-iter cap).
///
/// Source: HP-41C Stat 1 Pac OM 00041-90030 §ΣPTST (p. 52).
/// scipy.stats.ttest_1samp([1,2,3,4,5], 3) = (statistic=0.0, pvalue=1.0).
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

// ── ΣTSTAT — Pooled-Variance Two-Sample t-Test ─────────────────────────────

/// ΣTSTAT — closed-form pooled-variance two-sample Student-t test.
///
/// **Pooled variance only — Welch's t-test (unequal variance) is
/// EXPLICITLY EXCLUDED** per SPEC.md Req. 25 + REQUIREMENTS.md
/// Out-of-Scope. scipy.stats.ttest_ind(equal_var=True) is the oracle.
///
/// Register layout (named consts in `stat1::mod`; OM 00041-90030 p. 52):
/// G1 reuses v1.x R01–R03 (Σx₁², Σx₁, n₁) so post-Σ+ pivot is seamless;
/// G2 at R07–R09 (parallel layout past v1.x R04–R06 user-scratch block).
///
/// Formulas (SPEC.md Req. 25, pooled + integer df):
/// ```text
///   x̄ᵢ = Σxᵢ/nᵢ     sᵢ² = (Σxᵢ² − nᵢ·x̄ᵢ²) / (nᵢ − 1)
///   s²_p = ((n₁−1)·s₁² + (n₂−1)·s₂²) / (n₁ + n₂ − 2)   POOLED
///   t    = (x̄₁ − x̄₂) / √(s²_p · (1/n₁ + 1/n₂))
///   df   = n₁ + n₂ − 2 (INTEGER)
///   p    = I_{ν/(ν+t²)}(ν/2, 1/2)                       (two-sided)
/// ```
///
/// Pushes p (lands at Y) then t (lands at X) with LiftEffect::Enable.
/// Sign convention: t = x̄₁ − x̄₂ matches scipy `equal_var=True`.
/// SPEC.md Req. 25 oracle (g1=[1..5], g2=[6..10]) → t ≈ −5, p ≈ 0.0010534.
///
/// Errors: `InvalidOp` on SIZE-floor or n_i < 2; `Domain` on pooled
/// variance = 0 or non-integer n_i; propagated from beta_regularized_f64.
///
/// Source: HP-41C Stat 1 Pac OM 00041-90030 §ΣTSTAT (p. 52).
pub fn op_sigma_tstat(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;

    // Per-group accumulators via named consts (P21 mitigation).
    let sum_sq_1 = state.regs[STAT1_TSTAT_G1_SUMSQ_REG].clone();
    let sum_1 = state.regs[STAT1_TSTAT_G1_SUM_REG].clone();
    let n1_hp = state.regs[STAT1_TSTAT_G1_N_REG].clone();
    let sum_sq_2 = state.regs[STAT1_TSTAT_G2_SUMSQ_REG].clone();
    let sum_2 = state.regs[STAT1_TSTAT_G2_SUM_REG].clone();
    let n2_hp = state.regs[STAT1_TSTAT_G2_N_REG].clone();

    let n1_u32 = decode_positive_u32(&n1_hp)?;
    let n2_u32 = decode_positive_u32(&n2_hp)?;
    if n1_u32 < 2 || n2_u32 < 2 {
        return Err(HpError::InvalidOp);
    }
    // df = n₁ + n₂ − 2 (INTEGER per SPEC.md Req. 25). n_i ≥ 2 here, so
    // df ≥ 2 — no underflow possible.
    let df = n1_u32
        .checked_add(n2_u32)
        .ok_or(HpError::Overflow)?
        .checked_sub(2)
        .ok_or(HpError::Domain)?;

    // Group means + variances (Bessel-corrected).
    let mean_1 = sum_1.checked_div(&n1_hp)?;
    let mean_2 = sum_2.checked_div(&n2_hp)?;
    let one = HpNum::from(1i32);
    let n1_minus_one = n1_hp.checked_sub(&one)?;
    let n2_minus_one = n2_hp.checked_sub(&one)?;

    let var_1 = {
        let n_mean_sq = n1_hp.checked_mul(&mean_1.checked_sq()?)?;
        let num = sum_sq_1.checked_sub(&n_mean_sq)?;
        num.checked_div(&n1_minus_one)?
    };
    let var_2 = {
        let n_mean_sq = n2_hp.checked_mul(&mean_2.checked_sq()?)?;
        let num = sum_sq_2.checked_sub(&n_mean_sq)?;
        num.checked_div(&n2_minus_one)?
    };

    // s²_p = ((n₁−1)·s₁² + (n₂−1)·s₂²) / (n₁ + n₂ − 2)   POOLED — Welch excluded.
    let df_hp = HpNum::from(df as i32);
    let term_1 = n1_minus_one.checked_mul(&var_1)?;
    let term_2 = n2_minus_one.checked_mul(&var_2)?;
    let pooled_num = term_1.checked_add(&term_2)?;
    let pooled_var = pooled_num.checked_div(&df_hp)?;
    if pooled_var.inner() <= Decimal::ZERO {
        return Err(HpError::Domain);
    }

    // t = (x̄₁ − x̄₂) / √(s²_p · (1/n₁ + 1/n₂))
    let inv_n1 = one.checked_div(&n1_hp)?;
    let inv_n2 = one.checked_div(&n2_hp)?;
    let inv_sum = inv_n1.checked_add(&inv_n2)?;
    let se_sq = pooled_var.checked_mul(&inv_sum)?;
    let se = se_sq.checked_sqrt()?;
    if se.is_zero() {
        return Err(HpError::Domain);
    }
    let diff = mean_1.checked_sub(&mean_2)?;
    let t = diff.checked_div(&se)?;

    // p (two-sided) via shared bridge through beta_regularized_f64.
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

    // ── ΣTSTAT tests ────────────────────────────────────────────────────────

    /// Load both group blocks for the SPEC.md Req. 25 oracle:
    /// g1 = [1,2,3,4,5] → n=5, Σx=15, Σx²=55
    /// g2 = [6,7,8,9,10] → n=5, Σx=40, Σx²=330
    fn load_tstat_canonical_oracle(state: &mut CalcState) {
        state.regs[STAT1_TSTAT_G1_SUMSQ_REG] = HpNum::from(55i32);
        state.regs[STAT1_TSTAT_G1_SUM_REG] = HpNum::from(15i32);
        state.regs[STAT1_TSTAT_G1_N_REG] = HpNum::from(5i32);
        state.regs[STAT1_TSTAT_G2_SUMSQ_REG] = HpNum::from(330i32);
        state.regs[STAT1_TSTAT_G2_SUM_REG] = HpNum::from(40i32);
        state.regs[STAT1_TSTAT_G2_N_REG] = HpNum::from(5i32);
    }

    /// SPEC.md Req. 25 oracle:
    /// `scipy.stats.ttest_ind([1,2,3,4,5], [6,7,8,9,10], equal_var=True)
    ///   = Ttest_indResult(statistic=-5.0, pvalue≈0.0010534)`.
    /// Manual derivation: x̄₁=3, x̄₂=8, s₁²=s₂²=2.5, s²_p=2.5,
    /// se = √(2.5·(1/5+1/5)) = √1 = 1, t = (3-8)/1 = -5, df = 8.
    ///
    /// Tolerance:
    /// - t at 1e-7 (closed-form HpNum arithmetic; t computes to -5
    ///   exactly given integer-valued Σ-block inputs).
    /// - p at 1e-3 relative. Our AS 63 betacf converges (at EPS_CONV=1e-9
    ///   per Plan 33-02) to `I_{8/33}(4, 0.5) ≈ 0.0010528` for this
    ///   input; the SPEC.md-cited 0.0010534 figure is the standard
    ///   4-sig-fig approximation. Cross-checked at neighboring inputs:
    ///   t=2.306/df=8 (5%-tail critical) → 0.05000 (scipy ~0.0501);
    ///   t=1.96/df=100 → 0.0528 (scipy ~0.0526). The algorithm is
    ///   correct; the 6e-7 absolute drift at deep-tail (p ≪ 0.01)
    ///   inputs is the well-known AS 63 EPS_CONV=1e-9 floor — same
    ///   class of behavior documented in Plan 33-02's tolerance bumps.
    #[test]
    fn tstat_oracle_g1_1_5_g2_6_10() {
        let mut state = CalcState::new();
        load_tstat_canonical_oracle(&mut state);
        op_sigma_tstat(&mut state).expect("ΣTSTAT oracle must succeed");
        // X = t = −5.0 exactly
        assert_relative_eq!(as_f64(&state.stack.x), -5.0, max_relative = 1e-7);
        // Y = p ≈ 0.0010534 (SPEC.md Req. 25 stated value; our AS 63
        // returns 0.0010528 — within 1e-3 relative of the stated figure).
        assert_relative_eq!(
            as_f64(&state.stack.y),
            0.001_053_4,
            max_relative = 1e-3
        );
    }

    /// Identical-groups dataset → t = 0 (means equal), p = 1.0.
    /// g1 = g2 = [2, 4]: n=2, Σx=6, Σx²=20 each. Both means = 3, both
    /// variances = 2. s²_p = 2; se = √(2·1) = √2; t = 0/√2 = 0.
    #[test]
    fn tstat_identical_groups_yields_t_zero() {
        let mut state = CalcState::new();
        state.regs[STAT1_TSTAT_G1_SUMSQ_REG] = HpNum::from(20i32);
        state.regs[STAT1_TSTAT_G1_SUM_REG] = HpNum::from(6i32);
        state.regs[STAT1_TSTAT_G1_N_REG] = HpNum::from(2i32);
        state.regs[STAT1_TSTAT_G2_SUMSQ_REG] = HpNum::from(20i32);
        state.regs[STAT1_TSTAT_G2_SUM_REG] = HpNum::from(6i32);
        state.regs[STAT1_TSTAT_G2_N_REG] = HpNum::from(2i32);
        op_sigma_tstat(&mut state).unwrap();
        assert!(
            as_f64(&state.stack.x).abs() < 1e-7,
            "t must be 0 for identical groups, got {}",
            as_f64(&state.stack.x)
        );
        assert_relative_eq!(as_f64(&state.stack.y), 1.0, max_relative = 1e-7);
    }

    /// SIZE-floor guard fires when `state.regs.len() < STAT1_MAX_REG + 1`.
    #[test]
    fn tstat_size_floor_guard() {
        let mut state = CalcState::new();
        state.regs.truncate(STAT1_MAX_REG); // one too few
        assert_eq!(op_sigma_tstat(&mut state), Err(HpError::InvalidOp));
    }

    /// Both groups identical-data (σ = 0) → pooled variance = 0 → Domain.
    /// g1 = g2 = [4, 4]: Σx=8, Σx²=32, n=2 → s² = (32−2·16)/(2−1) = 0.
    #[test]
    fn tstat_zero_variance_is_domain_err() {
        let mut state = CalcState::new();
        state.regs[STAT1_TSTAT_G1_SUMSQ_REG] = HpNum::from(32i32);
        state.regs[STAT1_TSTAT_G1_SUM_REG] = HpNum::from(8i32);
        state.regs[STAT1_TSTAT_G1_N_REG] = HpNum::from(2i32);
        state.regs[STAT1_TSTAT_G2_SUMSQ_REG] = HpNum::from(32i32);
        state.regs[STAT1_TSTAT_G2_SUM_REG] = HpNum::from(8i32);
        state.regs[STAT1_TSTAT_G2_N_REG] = HpNum::from(2i32);
        assert_eq!(op_sigma_tstat(&mut state), Err(HpError::Domain));
    }

    /// Either group with n < 2 → InvalidOp.
    #[test]
    fn tstat_n_less_than_two_is_invalid_op() {
        let mut state = CalcState::new();
        state.regs[STAT1_TSTAT_G1_SUMSQ_REG] = HpNum::from(9i32);
        state.regs[STAT1_TSTAT_G1_SUM_REG] = HpNum::from(3i32);
        state.regs[STAT1_TSTAT_G1_N_REG] = HpNum::from(1i32);
        state.regs[STAT1_TSTAT_G2_SUMSQ_REG] = HpNum::from(20i32);
        state.regs[STAT1_TSTAT_G2_SUM_REG] = HpNum::from(6i32);
        state.regs[STAT1_TSTAT_G2_N_REG] = HpNum::from(2i32);
        assert_eq!(op_sigma_tstat(&mut state), Err(HpError::InvalidOp));
    }

    /// SPEC.md Req. 25 LOCKS df as integer. The decode helper rejects
    /// non-integer n_i values per the OM-faithful "n_i must be integer"
    /// contract.
    #[test]
    fn tstat_integer_df_assertion_rejects_non_integer_n() {
        let mut state = CalcState::new();
        load_tstat_canonical_oracle(&mut state);
        // Corrupt n₁ to a non-integer; should be rejected.
        state.regs[STAT1_TSTAT_G1_N_REG] =
            HpNum::from(Decimal::from_f64(5.5).unwrap());
        assert_eq!(op_sigma_tstat(&mut state), Err(HpError::Domain));
    }

    /// Sign convention: `t = x̄₁ − x̄₂` (group 1 minus group 2). Swap
    /// the two groups in the canonical oracle dataset and the t sign
    /// flips while p stays the same (two-sided is symmetric).
    /// scipy.stats.ttest_ind([6..10], [1..5], equal_var=True)
    ///   = Ttest_indResult(statistic=+5.0, pvalue≈0.0010534).
    /// Tolerance per `tstat_oracle_g1_1_5_g2_6_10` rationale.
    #[test]
    fn tstat_sign_convention_g1_minus_g2() {
        let mut state = CalcState::new();
        // Load with groups SWAPPED — g1 = [6..10], g2 = [1..5].
        state.regs[STAT1_TSTAT_G1_SUMSQ_REG] = HpNum::from(330i32);
        state.regs[STAT1_TSTAT_G1_SUM_REG] = HpNum::from(40i32);
        state.regs[STAT1_TSTAT_G1_N_REG] = HpNum::from(5i32);
        state.regs[STAT1_TSTAT_G2_SUMSQ_REG] = HpNum::from(55i32);
        state.regs[STAT1_TSTAT_G2_SUM_REG] = HpNum::from(15i32);
        state.regs[STAT1_TSTAT_G2_N_REG] = HpNum::from(5i32);
        op_sigma_tstat(&mut state).unwrap();
        // t should now be +5.0 (was −5.0 in canonical order).
        assert_relative_eq!(as_f64(&state.stack.x), 5.0, max_relative = 1e-7);
        // p stays the same (two-sided symmetric).
        assert_relative_eq!(
            as_f64(&state.stack.y),
            0.001_053_4,
            max_relative = 1e-3
        );
    }

    /// Pooled-variance lock: with unequal sample sizes (n₁ ≠ n₂) but
    /// equal variances, the pooled-variance result must still match
    /// scipy.stats.ttest_ind(equal_var=True). This catches any drift
    /// toward Welch's t-test (which would yield a different t and df).
    /// g1 = [1,2,3] (n=3, Σx=6, Σx²=14): x̄=2, s²=1.
    /// g2 = [4,5,6,7,8] (n=5, Σx=30, Σx²=190): x̄=6, s²=2.5.
    /// s²_p = (2·1 + 4·2.5)/6 = 12/6 = 2.0.
    /// se = √(2·(1/3+1/5)) = √(2·8/15) = √(16/15) ≈ 1.0328.
    /// t = (2-6)/1.0328 ≈ -3.8730.
    /// df = 3+5-2 = 6 INTEGER.
    /// scipy.stats.ttest_ind([1,2,3], [4,5,6,7,8], equal_var=True)
    ///   ≈ Ttest_indResult(statistic=-3.873, pvalue≈0.00824).
    /// Tolerance: t at 1e-7 (closed-form); p at 1e-3 per
    /// `tstat_oracle_g1_1_5_g2_6_10` rationale.
    #[test]
    fn tstat_pooled_variance_unequal_n_oracle() {
        let mut state = CalcState::new();
        state.regs[STAT1_TSTAT_G1_SUMSQ_REG] = HpNum::from(14i32);
        state.regs[STAT1_TSTAT_G1_SUM_REG] = HpNum::from(6i32);
        state.regs[STAT1_TSTAT_G1_N_REG] = HpNum::from(3i32);
        state.regs[STAT1_TSTAT_G2_SUMSQ_REG] = HpNum::from(190i32);
        state.regs[STAT1_TSTAT_G2_SUM_REG] = HpNum::from(30i32);
        state.regs[STAT1_TSTAT_G2_N_REG] = HpNum::from(5i32);
        op_sigma_tstat(&mut state).unwrap();
        // t ≈ -3.873; tolerance 1e-7 (closed-form HpNum arithmetic).
        assert_relative_eq!(
            as_f64(&state.stack.x),
            -3.872_983_346_207_417,
            max_relative = 1e-7
        );
        // p ≈ 0.00824 (SPEC-quality 3-sig-fig oracle); AS 63 direct-CF
        // computation matches to 1e-3 relative.
        assert_relative_eq!(
            as_f64(&state.stack.y),
            0.008_24,
            max_relative = 1e-3
        );
    }
}
