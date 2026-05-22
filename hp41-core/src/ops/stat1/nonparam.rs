// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `stat1::nonparam` — closed-form non-parametric Ops (Plan 33-04).
//!
//! Ships ΣSPEAR (Spearman rank correlation), ΣXSQEV (chi-square
//! goodness-of-fit with observed + expected counts), and ΣEFXSQ
//! (chi-square with expected-as-proportion entry). All three are
//! closed-form: no iteration, no distribution-function call, no modal
//! prompt. Compatible with [`crate::ops::math1::xrom::STAT_1`] via the
//! bit-1 arm of `xrom_resolve` (D-33.3 freeze exception).
//!
//! Every register access ≥ R07 routes through a named const from
//! `stat1::mod` (P21 mitigation). ΣSPEAR reads R02 (= Σx = Σd² per
//! existing v1.x stats convention) and R03 (= n) directly — explicitly
//! allowed per CLAUDE.md "Core engine" stats convention.
//!
//! ## SPEC.md oracle drift (resolved in this plan)
//!
//! - SPEC.md Req. 30 claims `ρ_s = 0.7` for `ranks_x=[1,2,3,4,5],
//!   ranks_y=[2,1,3,5,4]`. Manual + `scipy.stats.spearmanr` both confirm
//!   `ρ_s = 0.8`. SPEC.md is wrong; this plan ships 0.8 and flags the
//!   discrepancy for Phase 35 (STAT-DOC) amendment.
//! - SPEC.md Req. 27 claims `χ² ≈ 1.667` for proportions [0.2,0.3,0.5]
//!   vs observed [10,30,60]. Manual + `scipy.stats.chisquare` both
//!   confirm `χ² = 7.0` (Σf = 100, expected = [20,30,50]). SPEC.md is
//!   wrong; this plan ships 7.0 and flags the discrepancy for Phase 35.
//!
//! ## References
//!
//! - HP-41C Stat 1 Pac OM 00041-90030 (1979) §ΣSPEAR (p. 64),
//!   §ΣXSQEV / ΣEFXSQ (p. 55).
//! - `scipy.stats.spearmanr` and `scipy.stats.chisquare` (oracles).

use crate::error::HpError;
use crate::num::HpNum;
use crate::ops::stat1::{
    STAT1_MAX_REG, STAT1_XSQEV_EXP_BASE_REG, STAT1_XSQEV_KMAX, STAT1_XSQEV_K_REG,
    STAT1_XSQEV_MAX_REG, STAT1_XSQEV_OBS_BASE_REG, STAT1_XSQEV_STRIDE,
};
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

// ── ΣSPEAR — Spearman Rank Correlation Coefficient ──────────────────────────

/// ΣSPEAR — closed-form Spearman rank correlation coefficient.
///
/// `ρ_s = 1 − 6·Σd² / (n·(n²−1))`
///
/// where `dᵢ = rank(xᵢ) − rank(yᵢ)` for each ranked pair. The user
/// accumulates `dᵢ²` values as single-variable Σ+ samples BEFORE calling
/// ΣSPEAR; this Op reads `Σd² = state.regs[2]` (R02 = Σx in the existing
/// v1.x stats convention) and `n = state.regs[3]` (R03 = n).
///
/// Pre-condition (per OM p. 64): the user has already ranked the data
/// and accumulated each `d² = (rank_x − rank_y)²` via Σ+. ΣSPEAR does
/// NOT rank the data for the user (this matches HP's documented OM
/// contract; mid-rank tie-breaking is the user's responsibility per the
/// research pitfall list P21).
///
/// Result: ρ_s is pushed to stack X with `LiftEffect::Enable`.
///
/// # Errors
///
/// - `HpError::InvalidOp` if `state.regs.len() < STAT1_MAX_REG + 1`
///   (SIZE-floor guard).
/// - `HpError::InvalidOp` if `n < 2` (Spearman is undefined for n ≤ 1
///   because `n² − 1 == 0` produces a division-by-zero).
/// - `HpError::Overflow` / `HpError::DivideByZero` propagated from
///   `rust_decimal` arithmetic via the `?` operator.
///
/// # Source
///
/// HP-41C Stat 1 Pac Owner's Manual 00041-90030 (1979) §ΣSPEAR (p. 64).
/// Manual derivation cross-checked against
/// `scipy.stats.spearmanr([1,2,3,4,5], [2,1,3,5,4]).statistic` (oracle).
pub fn op_sigma_spear(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;

    // R02 = Σx = Σd² (v1.x convention from ops/stats.rs); R03 = n.
    let sum_d_sq = state.regs[2].clone();
    let n = state.regs[3].clone();

    // n < 2 → undefined (n² − 1 would be 0 or negative).
    let one = HpNum::from(1i32);
    let two = HpNum::from(2i32);
    if n.checked_sub(&two)?.inner() < rust_decimal::Decimal::ZERO {
        return Err(HpError::InvalidOp);
    }

    // ρ_s = 1 − 6·Σd² / (n·(n² − 1))
    let six = HpNum::from(6i32);
    let n_sq = n.checked_sq()?;
    let n_sq_minus_one = n_sq.checked_sub(&one)?;
    let denom = n.checked_mul(&n_sq_minus_one)?;
    let numer = six.checked_mul(&sum_d_sq)?;
    let ratio = numer.checked_div(&denom)?;
    let rho_s = one.checked_sub(&ratio)?;

    // Push ρ_s to X with stack-lift enabled (matches op_corr / op_mean
    // convention in ops/stats.rs).
    state.stack.lift_enabled = true;
    enter_number(state, rho_s);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

// ── ΣXSQEV — Chi-Square Goodness-of-Fit (Observed + Expected Counts) ────────

/// ΣXSQEV — closed-form chi-square goodness-of-fit statistic.
///
/// `χ² = Σ (O_i − E_i)² / E_i`
///
/// Reads `k` (number of categories) from R00 and interleaved
/// observed/expected pairs starting at R01 (O₀=R01, E₀=R02, O₁=R03,
/// E₁=R04, ..., scratch=R07). χ² is pushed to stack X with
/// `LiftEffect::Enable` AND written to R07 (OM-faithful "result also
/// stored in R07" per p. 55).
///
/// Standard goodness-of-fit `df = k − 1` (informational; p-value
/// computation is the caller's responsibility via ΣCHISQD).
///
/// # Errors
///
/// - `HpError::InvalidOp` on SIZE-floor guard.
/// - `HpError::Domain` if `k < 1` or `k > STAT1_XSQEV_KMAX` (= 3 for
///   SIZE 008), or if any expected count `E_i == 0`.
///
/// # Source
///
/// OM 00041-90030 §ΣXSQEV (p. 55). Oracle:
/// `scipy.stats.chisquare([10,20,30], f_exp=[15,20,25]).statistic`
/// returns `2.666666666666667` (≈ 8/3).
pub fn op_sigma_xsqev(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;
    let chi_sq = compute_chi_square_from_counts(state)?;

    // Write to scratch slot R07 (OM "Result" register) AND push to X.
    state.regs[STAT1_XSQEV_MAX_REG] = chi_sq.clone();
    state.stack.lift_enabled = true;
    enter_number(state, chi_sq);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// Compute χ² from interleaved O/E pairs in the SIZE 008 block. Shared
/// reducer used by [`op_sigma_xsqev`] directly; [`op_sigma_efxsq`]
/// (Task 3) will call this AFTER converting proportions to expected
/// counts in-place.
fn compute_chi_square_from_counts(state: &CalcState) -> Result<HpNum, HpError> {
    // Read k (number of categories) from R00 and validate domain.
    let k_num = state.regs[STAT1_XSQEV_K_REG].clone();
    let k = decode_category_count(&k_num)?;

    let mut chi_sq = HpNum::zero();
    for i in 0..k {
        let obs_reg = STAT1_XSQEV_OBS_BASE_REG + STAT1_XSQEV_STRIDE * i;
        let exp_reg = STAT1_XSQEV_EXP_BASE_REG + STAT1_XSQEV_STRIDE * i;
        let obs = state.regs[obs_reg].clone();
        let exp = state.regs[exp_reg].clone();
        if exp.is_zero() {
            return Err(HpError::Domain);
        }
        let diff = obs.checked_sub(&exp)?;
        let term = diff.checked_sq()?.checked_div(&exp)?;
        chi_sq = chi_sq.checked_add(&term)?;
    }
    Ok(chi_sq)
}

/// Decode `k` (number of categories) from an `HpNum` register value.
/// Truncates toward zero per HP-41 INT convention, then validates the
/// `1 ≤ k ≤ STAT1_XSQEV_KMAX` domain. Returns `HpError::Domain` for
/// out-of-range values (matches HP-41's "out of bounds" semantics; the
/// closed-form Op cannot produce a meaningful χ² for k < 1 or for k
/// exceeding the 8-register block's cell capacity).
///
/// P21 mitigation: never uses `.floor()` or `.fmod()` on an f64 — the
/// integer extraction routes through `HpNum::trunc_int()` which uses
/// `Decimal::trunc()`. See CLAUDE.md "Frozen Invariants — Core engine"
/// ISG/DSE counter rule.
fn decode_category_count(k_num: &HpNum) -> Result<usize, HpError> {
    use rust_decimal::prelude::ToPrimitive;
    let k_int_dec = k_num.trunc_int().inner();
    // Must be non-negative and within usize range.
    let k = k_int_dec.to_usize().ok_or(HpError::Domain)?;
    if !(1..=STAT1_XSQEV_KMAX).contains(&k) {
        return Err(HpError::Domain);
    }
    Ok(k)
}

// ── ΣEFXSQ — Chi-Square with Expected-as-Proportions ────────────────────────

/// Tolerance for the ΣEFXSQ proportion-sum constraint `|Σ p_i − 1.0| ≤ TOL`.
///
/// 1e-9 matches SPEC.md Req. 27's `max_relative` tolerance — the user is
/// expected to enter proportions accurate to closed-form 1e-9 precision.
/// Out-of-tolerance sums return `HpError::Domain` rather than being
/// silently renormalized (Pitfall 21 mitigation: silent renormalization
/// produces silently wrong χ² values).
///
/// `Decimal::from_parts(1, 0, 0, false, 9)` = mantissa 1 × 10⁻⁹ = 1e-9.
const PROPORTION_SUM_TOL_DEC: rust_decimal::Decimal =
    rust_decimal::Decimal::from_parts(1, 0, 0, false, 9);

/// ΣEFXSQ — closed-form chi-square goodness-of-fit with expected proportions.
///
/// Reads `k` from R00 and interleaved O/p pairs from R01 (O₀=R01, p₀=R02,
/// O₁=R03, p₁=R04, ...). Validates `|Σ p_i − 1.0| ≤ 1e-9` (OM "Inputs"
/// sum-to-1 contract), converts each `p_i` to expected count
/// `E_i = Σf · p_i` IN PLACE (mutating R02/R04/R06 per OM p. 55), then
/// delegates to the shared `Σ (O − E)² / E` reducer.
///
/// Out-of-tolerance proportion sums and non-positive `p_i` both return
/// `HpError::Domain` — silent renormalization would produce silently
/// wrong χ² values (Pitfall 21).
///
/// # Errors
///
/// - `HpError::InvalidOp` if SIZE-floor guard fires.
/// - `HpError::Domain` if `k` out of range, any `p_i ≤ 0`,
///   `|Σ p_i − 1.0| > 1e-9`, or any computed `E_i == 0`.
///
/// # Source
///
/// OM 00041-90030 §ΣEFXSQ (p. 55). SPEC.md Req. 27 oracle drift:
/// `scipy.stats.chisquare(f_obs=[10,30,60], f_exp=[20,30,50]).statistic`
/// returns `7.0` exactly (SPEC says ≈ 1.667 — see module-level doc).
pub fn op_sigma_efxsq(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;

    // Read k and validate.
    let k_num = state.regs[STAT1_XSQEV_K_REG].clone();
    let k = decode_category_count(&k_num)?;

    // Compute Σf = Σ O_i AND validate Σ p_i = 1.0 within tolerance.
    // Reject p_i ≤ 0 (would produce zero or negative expected count
    // after multiplication by Σf).
    let mut sum_f = HpNum::zero();
    let mut sum_p = HpNum::zero();
    for i in 0..k {
        let obs_reg = STAT1_XSQEV_OBS_BASE_REG + STAT1_XSQEV_STRIDE * i;
        let prop_reg = STAT1_XSQEV_EXP_BASE_REG + STAT1_XSQEV_STRIDE * i;
        sum_f = sum_f.checked_add(&state.regs[obs_reg])?;
        let p = state.regs[prop_reg].clone();
        if p.inner() <= rust_decimal::Decimal::ZERO {
            return Err(HpError::Domain);
        }
        sum_p = sum_p.checked_add(&p)?;
    }

    // |Σ p_i − 1.0| ≤ tolerance. Uses Decimal::abs() directly — avoids
    // any f64 round-trip that could mis-classify edge values.
    let one = HpNum::from(1i32);
    let diff_p = sum_p.checked_sub(&one)?;
    let abs_diff = diff_p.inner().abs();
    if abs_diff > PROPORTION_SUM_TOL_DEC {
        return Err(HpError::Domain);
    }

    // Convert proportions in-place to expected counts: E_i = Σf · p_i.
    for i in 0..k {
        let prop_reg = STAT1_XSQEV_EXP_BASE_REG + STAT1_XSQEV_STRIDE * i;
        let p = state.regs[prop_reg].clone();
        let exp = sum_f.checked_mul(&p)?;
        state.regs[prop_reg] = exp;
    }

    // Delegate to the shared O/E reducer (Task 2 helper).
    let chi_sq = compute_chi_square_from_counts(state)?;
    state.regs[STAT1_XSQEV_MAX_REG] = chi_sq.clone();
    state.stack.lift_enabled = true;
    enter_number(state, chi_sq);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use rust_decimal::prelude::ToPrimitive;

    /// Helper: convert `state.stack.x` (HpNum) to f64 for `assert_relative_eq!`.
    fn x_as_f64(state: &CalcState) -> f64 {
        state
            .stack
            .x
            .inner()
            .to_f64()
            .expect("HpNum→f64 must succeed for finite oracle values")
    }

    // ── ΣSPEAR tests ────────────────────────────────────────────────────────

    /// SPEC.md Req. 30 oracle (corrected from 0.7 to 0.8 per scipy/manual).
    ///
    /// `ranks_x=[1,2,3,4,5], ranks_y=[2,1,3,5,4]`:
    ///   d = [-1, 1, 0, -1, 1]; d² = [1, 1, 0, 1, 1]; Σd² = 4; n = 5.
    ///   ρ_s = 1 − 6·4 / (5·(25−1)) = 1 − 24/120 = 1 − 0.2 = 0.8.
    ///
    /// `scipy.stats.spearmanr([1,2,3,4,5], [2,1,3,5,4]).statistic`
    ///   returns `0.7999999999999999` (= 0.8 within f64 precision).
    ///
    /// SPEC.md Req. 30 says `ρ_s = 0.7` — that value is WRONG. This test
    /// asserts the mathematically correct 0.8; SPEC.md amendment is
    /// gated to Phase 35 (STAT-DOC) — discrepancy documented inline
    /// here and in `33-04-SUMMARY.md`.
    #[test]
    fn spear_basic_5_pairs() {
        let mut state = CalcState::new();
        // R02 = Σd² = 4; R03 = n = 5 (v1.x stats convention; pre-loaded
        // by the user via Σ+ on each d² value).
        state.regs[2] = HpNum::from(4i32);
        state.regs[3] = HpNum::from(5i32);
        op_sigma_spear(&mut state).expect("ΣSPEAR with valid Σd²=4, n=5 must succeed");
        // ρ_s = 0.8 exactly (closed-form on integer inputs).
        assert_relative_eq!(x_as_f64(&state), 0.8, max_relative = 1e-9);
    }

    /// Perfect positive correlation: ranks identical → Σd² = 0 → ρ_s = 1.
    #[test]
    fn spear_perfect_positive_correlation() {
        let mut state = CalcState::new();
        state.regs[2] = HpNum::zero();
        state.regs[3] = HpNum::from(5i32);
        op_sigma_spear(&mut state).unwrap();
        assert_relative_eq!(x_as_f64(&state), 1.0, max_relative = 1e-9);
    }

    /// Perfect negative correlation: ranks reversed → Σd² = n(n²−1)/3 →
    /// ρ_s = −1. For n=5: Σd² = 5·24/3 = 40 → ρ_s = 1 − 6·40/(5·24)
    /// = 1 − 240/120 = 1 − 2 = −1.
    #[test]
    fn spear_perfect_negative_correlation() {
        let mut state = CalcState::new();
        state.regs[2] = HpNum::from(40i32);
        state.regs[3] = HpNum::from(5i32);
        op_sigma_spear(&mut state).unwrap();
        assert_relative_eq!(x_as_f64(&state), -1.0, max_relative = 1e-9);
    }

    /// SIZE-floor guard fires when `state.regs.len() < STAT1_MAX_REG + 1`.
    #[test]
    fn spear_size_floor_guard() {
        let mut state = CalcState::new();
        state.regs.truncate(STAT1_MAX_REG); // one too few
        assert_eq!(op_sigma_spear(&mut state).unwrap_err(), HpError::InvalidOp);
    }

    /// n < 2 returns InvalidOp (division by zero defended).
    #[test]
    fn spear_n_too_small() {
        let mut state = CalcState::new();
        state.regs[2] = HpNum::zero();
        state.regs[3] = HpNum::from(1i32);
        assert_eq!(op_sigma_spear(&mut state).unwrap_err(), HpError::InvalidOp);
    }

    // ── ΣXSQEV tests ────────────────────────────────────────────────────────

    /// SPEC.md Req. 26 oracle: obs=[10,20,30], exp=[15,20,25] → χ² = 8/3
    /// ≈ 2.6666666666666665.
    ///
    /// Manual: (10−15)²/15 + (20−20)²/20 + (30−25)²/25
    ///       = 25/15 + 0 + 25/25 = 5/3 + 0 + 1 = 8/3 ≈ 2.6667.
    /// `scipy.stats.chisquare([10,20,30], f_exp=[15,20,25]).statistic`
    ///   returns `2.666666666666667`.
    #[test]
    fn xsqev_basic() {
        let mut state = CalcState::new();
        // R00 = k = 3
        state.regs[STAT1_XSQEV_K_REG] = HpNum::from(3i32);
        // Interleaved O/E: (R01,R02), (R03,R04), (R05,R06)
        state.regs[1] = HpNum::from(10i32);
        state.regs[2] = HpNum::from(15i32);
        state.regs[3] = HpNum::from(20i32);
        state.regs[4] = HpNum::from(20i32);
        state.regs[5] = HpNum::from(30i32);
        state.regs[6] = HpNum::from(25i32);
        op_sigma_xsqev(&mut state).unwrap();
        assert_relative_eq!(x_as_f64(&state), 8.0 / 3.0, max_relative = 1e-9);
        // Result also lands in R07 scratch slot.
        assert_relative_eq!(
            state.regs[STAT1_XSQEV_MAX_REG].inner().to_f64().unwrap(),
            8.0 / 3.0,
            max_relative = 1e-9
        );
    }

    /// Perfect fit (O == E) → χ² = 0.
    #[test]
    fn xsqev_perfect_fit() {
        let mut state = CalcState::new();
        state.regs[STAT1_XSQEV_K_REG] = HpNum::from(3i32);
        state.regs[1] = HpNum::from(20i32);
        state.regs[2] = HpNum::from(20i32);
        state.regs[3] = HpNum::from(30i32);
        state.regs[4] = HpNum::from(30i32);
        state.regs[5] = HpNum::from(50i32);
        state.regs[6] = HpNum::from(50i32);
        op_sigma_xsqev(&mut state).unwrap();
        assert_eq!(state.stack.x, HpNum::zero());
    }

    /// Zero expected count → Domain error (would divide by zero).
    #[test]
    fn xsqev_zero_expected_returns_domain_error() {
        let mut state = CalcState::new();
        state.regs[STAT1_XSQEV_K_REG] = HpNum::from(2i32);
        state.regs[1] = HpNum::from(10i32);
        state.regs[2] = HpNum::zero(); // expected = 0 → undefined
        state.regs[3] = HpNum::from(20i32);
        state.regs[4] = HpNum::from(15i32);
        assert_eq!(op_sigma_xsqev(&mut state).unwrap_err(), HpError::Domain);
    }

    /// SIZE-floor guard fires.
    #[test]
    fn xsqev_size_floor_guard() {
        let mut state = CalcState::new();
        state.regs.truncate(STAT1_MAX_REG); // one too few
        assert_eq!(op_sigma_xsqev(&mut state).unwrap_err(), HpError::InvalidOp);
    }

    /// k out of domain (k = 0 and k > KMAX both rejected).
    #[test]
    fn xsqev_k_out_of_range_returns_domain_error() {
        let mut state = CalcState::new();
        state.regs[STAT1_XSQEV_K_REG] = HpNum::zero();
        assert_eq!(op_sigma_xsqev(&mut state).unwrap_err(), HpError::Domain);
        state.regs[STAT1_XSQEV_K_REG] = HpNum::from(STAT1_XSQEV_KMAX as i32 + 1);
        assert_eq!(op_sigma_xsqev(&mut state).unwrap_err(), HpError::Domain);
    }

    /// Single-category (k=1) trivial case: obs=10, exp=10 → χ² = 0.
    #[test]
    fn xsqev_single_category() {
        let mut state = CalcState::new();
        state.regs[STAT1_XSQEV_K_REG] = HpNum::from(1i32);
        state.regs[1] = HpNum::from(10i32);
        state.regs[2] = HpNum::from(10i32);
        op_sigma_xsqev(&mut state).unwrap();
        assert_eq!(state.stack.x, HpNum::zero());
    }

    // ── ΣEFXSQ tests ────────────────────────────────────────────────────────

    /// Helper: HpNum literal `0.X` (one decimal digit).
    fn p_one_tenth(digit_tenths: i64) -> HpNum {
        HpNum::from(rust_decimal::Decimal::new(digit_tenths, 1))
    }

    /// SPEC.md Req. 27 oracle drift: stated 1.667, actual χ² = 7.0.
    ///
    /// Proportions [0.2, 0.3, 0.5], observed [10, 30, 60]:
    ///   Σf = 100; expected = [20, 30, 50].
    ///   χ² = (10−20)²/20 + (30−30)²/30 + (60−50)²/50
    ///      = 100/20 + 0 + 100/50 = 5 + 0 + 2 = 7.0.
    /// `scipy.stats.chisquare([10,30,60], f_exp=[20,30,50]).statistic`
    ///   returns `7.0` exactly.
    ///
    /// SPEC.md Req. 27 says `≈ 1.667` — that value is WRONG. This test
    /// asserts the scipy-confirmed 7.0; SPEC.md amendment is gated to
    /// Phase 35 (STAT-DOC), discrepancy documented in 33-04-SUMMARY.md.
    #[test]
    fn efxsq_basic() {
        let mut state = CalcState::new();
        state.regs[STAT1_XSQEV_K_REG] = HpNum::from(3i32);
        // Interleaved O/p: (R01, R02), (R03, R04), (R05, R06)
        state.regs[1] = HpNum::from(10i32);
        state.regs[2] = p_one_tenth(2); // 0.2
        state.regs[3] = HpNum::from(30i32);
        state.regs[4] = p_one_tenth(3); // 0.3
        state.regs[5] = HpNum::from(60i32);
        state.regs[6] = p_one_tenth(5); // 0.5
        op_sigma_efxsq(&mut state).unwrap();
        assert_relative_eq!(x_as_f64(&state), 7.0, max_relative = 1e-9);
        // Result also lands in R07 scratch slot.
        assert_relative_eq!(
            state.regs[STAT1_XSQEV_MAX_REG].inner().to_f64().unwrap(),
            7.0,
            max_relative = 1e-9
        );
        // In-place conversion: R02/R04/R06 now hold expected COUNTS, not
        // proportions (per OM "in-place conversion" convention p. 55).
        assert_relative_eq!(
            state.regs[2].inner().to_f64().unwrap(),
            20.0,
            max_relative = 1e-9
        );
        assert_relative_eq!(
            state.regs[4].inner().to_f64().unwrap(),
            30.0,
            max_relative = 1e-9
        );
        assert_relative_eq!(
            state.regs[6].inner().to_f64().unwrap(),
            50.0,
            max_relative = 1e-9
        );
    }

    /// Σ p_i ≠ 1.0 → Domain error (sum-to-1 validated, NOT silently
    /// renormalized — Pitfall 21).
    #[test]
    fn efxsq_proportions_not_sum_to_one_returns_domain_error() {
        let mut state = CalcState::new();
        state.regs[STAT1_XSQEV_K_REG] = HpNum::from(3i32);
        state.regs[1] = HpNum::from(10i32);
        state.regs[2] = p_one_tenth(2); // 0.2
        state.regs[3] = HpNum::from(30i32);
        state.regs[4] = p_one_tenth(3); // 0.3
        state.regs[5] = HpNum::from(60i32);
        state.regs[6] = p_one_tenth(6); // 0.6 → sum = 1.1
        assert_eq!(op_sigma_efxsq(&mut state).unwrap_err(), HpError::Domain);
    }

    /// Zero (or negative) proportion → Domain error.
    #[test]
    fn efxsq_zero_proportion_returns_domain_error() {
        let mut state = CalcState::new();
        state.regs[STAT1_XSQEV_K_REG] = HpNum::from(2i32);
        state.regs[1] = HpNum::from(10i32);
        state.regs[2] = HpNum::zero(); // p₀ = 0 → undefined (would yield E=0)
        state.regs[3] = HpNum::from(20i32);
        state.regs[4] = p_one_tenth(10); // 1.0
        assert_eq!(op_sigma_efxsq(&mut state).unwrap_err(), HpError::Domain);
    }

    /// SIZE-floor guard fires.
    #[test]
    fn efxsq_size_floor_guard() {
        let mut state = CalcState::new();
        state.regs.truncate(STAT1_MAX_REG);
        assert_eq!(op_sigma_efxsq(&mut state).unwrap_err(), HpError::InvalidOp);
    }

    /// Two-category proportions [0.5, 0.5] vs observed [50, 50] → χ² = 0.
    #[test]
    fn efxsq_perfect_fit() {
        let mut state = CalcState::new();
        state.regs[STAT1_XSQEV_K_REG] = HpNum::from(2i32);
        state.regs[1] = HpNum::from(50i32);
        state.regs[2] = p_one_tenth(5); // 0.5
        state.regs[3] = HpNum::from(50i32);
        state.regs[4] = p_one_tenth(5); // 0.5
        op_sigma_efxsq(&mut state).unwrap();
        assert_eq!(state.stack.x, HpNum::zero());
    }

    /// k out of domain for ΣEFXSQ (mirrors ΣXSQEV behavior).
    #[test]
    fn efxsq_k_out_of_range_returns_domain_error() {
        let mut state = CalcState::new();
        state.regs[STAT1_XSQEV_K_REG] = HpNum::zero();
        assert_eq!(op_sigma_efxsq(&mut state).unwrap_err(), HpError::Domain);
        state.regs[STAT1_XSQEV_K_REG] = HpNum::from(STAT1_XSQEV_KMAX as i32 + 1);
        assert_eq!(op_sigma_efxsq(&mut state).unwrap_err(), HpError::Domain);
    }
}
