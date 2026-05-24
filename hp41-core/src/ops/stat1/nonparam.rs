// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `stat1::nonparam` — closed-form non-parametric Ops (Plans 33-04, 33-06).
//!
//! Plan 33-04: ΣSPEAR (Spearman rank correlation), ΣXSQEV (χ² goodness-
//! of-fit with observed+expected counts), ΣEFXSQ (χ² with expected-as-
//! proportions). Plan 33-06 extends with ΣCTKKK (r×c contingency χ²)
//! and ΣCTKK (2×2 contingency χ²). All closed-form: no iteration, no
//! distribution-function call, no modal prompt. Every register access
//! ≥ R07 routes through named consts from `stat1::mod` (P21 mitigation).
//!
//! SPEC.md drifts resolved (Phase 35 STAT-DOC amendment gated):
//! Req. 30 ρ_s 0.7→0.8 (scipy.stats.spearmanr); Req. 27 χ² 1.667→7.0
//! (scipy.stats.chisquare); Req. 28 χ² 4.286→2.8; Req. 29 χ² 0.397→
//! 0.7937 (both contingency drifts from scipy.stats.chi2_contingency
//! correction=False).
//!
//! References: OM 00041-90030 §ΣSPEAR p. 64, §ΣXSQEV/ΣEFXSQ p. 55,
//! §ΣCTKKK/ΣCTKK p. 60; scipy.stats.* oracles.

use crate::error::HpError;
use crate::num::HpNum;
use crate::ops::stat1::{
    STAT1_CTKKK_CELL_BASE_REG, STAT1_CTKKK_C_REG, STAT1_CTKKK_DIM_MAX, STAT1_CTKKK_R_REG,
    STAT1_CTKK_DIM_MAX, STAT1_MAX_REG, STAT1_XSQEV_EXP_BASE_REG, STAT1_XSQEV_KMAX,
    STAT1_XSQEV_K_REG, STAT1_XSQEV_OBS_BASE_REG, STAT1_XSQEV_RESULT_REG, STAT1_XSQEV_STRIDE,
};
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;

// ── SIZE-floor guard helper ────────────────────────────────────────────────

/// Fail-closed SIZE-floor guard against STAT1_MAX_REG (P21 mitigation).
#[inline]
fn require_stat1_size_floor(state: &CalcState) -> Result<(), HpError> {
    if state.regs.len() < STAT1_MAX_REG + 1 {
        return Err(HpError::InvalidOp);
    }
    Ok(())
}

// ── ΣSPEAR — Spearman Rank Correlation Coefficient ──────────────────────────

/// ΣSPEAR — closed-form Spearman rank correlation
/// `ρ_s = 1 − 6·Σd² / (n·(n²−1))`. Reads Σd² from R02 (= Σx via the
/// existing v1.x stats convention) and n from R03. The user accumulates
/// each `d² = (rank_x − rank_y)²` via Σ+ before calling ΣSPEAR; this Op
/// does not rank the data. Pushes ρ_s to X with LiftEffect::Enable.
///
/// Errors: `InvalidOp` on SIZE-floor or n < 2.
///
/// Source: OM 00041-90030 §ΣSPEAR (p. 64); scipy.stats.spearmanr oracle.
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

/// ΣXSQEV — closed-form `χ² = Σ (Oᵢ − Eᵢ)² / Eᵢ`. Reads k from R00 and
/// interleaved O/E pairs from R01 (stride 2). Pushes χ² to X AND writes
/// to R07 per OM p. 55.
///
/// Errors: `InvalidOp` on SIZE-floor; `Domain` if k out of `[1..KMAX]`
/// or any Eᵢ == 0.
///
/// Source: OM 00041-90030 §ΣXSQEV (p. 55); scipy.stats.chisquare oracle.
pub fn op_sigma_xsqev(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;
    let chi_sq = compute_chi_square_from_counts(state)?;

    // Write to OM-cited "Result" slot R07 AND push to X. Per REVIEW.md
    // CR-02, the result address is `STAT1_XSQEV_RESULT_REG` — a
    // dedicated semantic constant separate from `STAT1_XSQEV_MAX_REG`
    // (the SIZE-floor sentinel that happens to coincide with R07
    // today). Decoupling guards against silent result-address drift
    // if STAT1_XSQEV_MAX_REG is ever bumped (e.g., to support more
    // categories).
    state.regs[STAT1_XSQEV_RESULT_REG] = chi_sq.clone();
    state.stack.lift_enabled = true;
    enter_number(state, chi_sq);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// Shared reducer: χ² from interleaved O/E pairs in the SIZE 008 block.
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

/// Decode `k` from an HpNum register value via HpNum::trunc_int (P21:
/// never floor/fmod on f64). Validates `1 ≤ k ≤ STAT1_XSQEV_KMAX`.
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

/// Tolerance for `|Σ p_i − 1.0| ≤ TOL` for ΣEFXSQ expected-proportion
/// inputs. Value: `1 × 10⁻⁹` (1e-9), closed-form per SPEC.md Req. 27.
/// Out-of-tolerance returns Domain (Pitfall 21: no silent
/// renormalization of the user's proportions).
///
/// Built via `Decimal::from_parts(lo, mid, hi, negative, scale)` which
/// represents `(hi << 64 | mid << 32 | lo) × 10⁻ˢᶜᵃˡᵉ`. The argument
/// tuple `(1, 0, 0, false, 9)` therefore encodes `1 × 10⁻⁹ = 0.000_000_001`.
/// `Decimal::new(1, 9)` would be cleaner but is NOT a `const fn` at the
/// pinned `rust_decimal` version — the four-arg constructor is the
/// only `const`-compatible path. REVIEW.md WR-07: this doc-comment
/// makes the literal value readable without forcing the reader to
/// recall the `from_parts` argument-order convention at every call
/// site.
const PROPORTION_SUM_TOL_DEC: rust_decimal::Decimal =
    rust_decimal::Decimal::from_parts(1, 0, 0, false, 9);

/// ΣEFXSQ — χ² with expected PROPORTIONS. Reads k from R00 and
/// interleaved O/p pairs from R01. Validates `|Σpᵢ − 1| ≤ 1e-9`,
/// converts each pᵢ to expected count `Eᵢ = Σf·pᵢ` IN PLACE (mutating
/// R02/R04/R06 per OM p. 55), then delegates to the shared O/E reducer.
///
/// Errors: `InvalidOp` on SIZE-floor; `Domain` if k out of range,
/// pᵢ ≤ 0, `|Σpᵢ − 1| > 1e-9`, or any computed Eᵢ == 0.
///
/// Source: OM 00041-90030 §ΣEFXSQ (p. 55); scipy.stats.chisquare oracle.
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

    // Delegate to the shared O/E reducer (Task 2 helper). The χ²
    // value lands in the OM-cited result register R07 (see CR-02
    // mitigation note on `op_sigma_xsqev` above).
    let chi_sq = compute_chi_square_from_counts(state)?;
    state.regs[STAT1_XSQEV_RESULT_REG] = chi_sq.clone();
    state.stack.lift_enabled = true;
    enter_number(state, chi_sq);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

// ── ΣCTKKK / ΣCTKK — Contingency-Table χ² (Plan 33-06) ─────────────────────

/// Decode contingency-table dimension capped at `cap` (P21: trunc_int only).
fn decode_dim(d_num: &HpNum, cap: usize) -> Result<usize, HpError> {
    use rust_decimal::prelude::ToPrimitive;
    let d_int = d_num.trunc_int().inner();
    let d = d_int.to_usize().ok_or(HpError::Domain)?;
    if !(1..=cap).contains(&d) {
        return Err(HpError::Domain);
    }
    Ok(d)
}

/// Shared r×c contingency χ² reducer. Reads r/c from R00/R01, cells
/// row-major from R02. `E_ij = (Rᵢ·Cⱼ)/T`; `χ² = ΣΣ(O−E)²/E`.
fn compute_contingency_chi_sq(state: &CalcState, dim_cap: usize) -> Result<HpNum, HpError> {
    let r = decode_dim(&state.regs[STAT1_CTKKK_R_REG], dim_cap)?;
    let c = decode_dim(&state.regs[STAT1_CTKKK_C_REG], dim_cap)?;
    // Bounds: cells occupy R<CELL_BASE>..R<CELL_BASE + r*c − 1>; must
    // fit within STAT1_CTKKK_MAX_REG.
    if STAT1_CTKKK_CELL_BASE_REG + r * c > crate::ops::stat1::STAT1_CTKKK_MAX_REG + 1 {
        return Err(HpError::Domain);
    }
    let mut row_sums = vec![HpNum::zero(); r];
    let mut col_sums = vec![HpNum::zero(); c];
    let mut grand = HpNum::zero();
    for (i, row_acc) in row_sums.iter_mut().enumerate().take(r) {
        for (j, col_acc) in col_sums.iter_mut().enumerate().take(c) {
            let cell = state.regs[STAT1_CTKKK_CELL_BASE_REG + i * c + j].clone();
            if cell.inner() < rust_decimal::Decimal::ZERO {
                return Err(HpError::Domain);
            }
            *row_acc = row_acc.checked_add(&cell)?;
            *col_acc = col_acc.checked_add(&cell)?;
            grand = grand.checked_add(&cell)?;
        }
    }
    if grand.is_zero() {
        return Err(HpError::Domain);
    }
    let mut chi_sq = HpNum::zero();
    for (i, row_total) in row_sums.iter().enumerate() {
        for (j, col_total) in col_sums.iter().enumerate() {
            let o = state.regs[STAT1_CTKKK_CELL_BASE_REG + i * c + j].clone();
            let e = row_total.checked_mul(col_total)?.checked_div(&grand)?;
            if e.is_zero() {
                return Err(HpError::Domain);
            }
            let diff = o.checked_sub(&e)?;
            chi_sq = chi_sq.checked_add(&diff.checked_sq()?.checked_div(&e)?)?;
        }
    }
    Ok(chi_sq)
}

/// ΣCTKKK — General r×c contingency χ². Pushes to X with LiftEffect::Enable.
/// Errors: `InvalidOp` on SIZE-floor; `Domain` if r/c out of
/// `[1..STAT1_CTKKK_DIM_MAX]`, any cell < 0, grand total == 0, or any
/// Eᵢⱼ == 0. Source: OM 00041-90030 §ΣCTKKK (p. 60);
/// scipy.stats.chi2_contingency(correction=False) oracle.
pub fn op_sigma_ctkkk(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;
    let chi_sq = compute_contingency_chi_sq(state, STAT1_CTKKK_DIM_MAX)?;
    state.stack.lift_enabled = true;
    enter_number(state, chi_sq);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ΣCTKK — Smaller-table contingency χ² (cap 2×2). Same as ΣCTKKK with
/// `STAT1_CTKK_DIM_MAX = 2`. OM p. 60 silent on Yates' correction; we
/// follow scipy.stats.chi2_contingency(correction=False).
pub fn op_sigma_ctkk(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;
    let chi_sq = compute_contingency_chi_sq(state, STAT1_CTKK_DIM_MAX)?;
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
            state.regs[STAT1_XSQEV_RESULT_REG].inner().to_f64().unwrap(),
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
        // LINT-EXEMPT: integer-equality — perfect fit O==E gives chi-sq=0 exactly; rust_decimal arithmetic on integer inputs yields exact zero
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
        assert_eq!(op_sigma_xsqev(&mut state).unwrap_err(), HpError::Domain); // LINT-EXEMPT: error-type comparison, no HpNum; lookahead false positive from adjacent HpNum setup lines
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
        assert_eq!(op_sigma_xsqev(&mut state).unwrap_err(), HpError::Domain); // LINT-EXEMPT: error-type comparison, no HpNum; lookahead false positive from adjacent HpNum setup line
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
        // LINT-EXEMPT: integer-equality — single-category with O==E gives chi-sq=0 exactly; rust_decimal arithmetic on integer inputs yields exact zero
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
            state.regs[STAT1_XSQEV_RESULT_REG].inner().to_f64().unwrap(),
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
        // LINT-EXEMPT: integer-equality — perfect fit O==E·N gives chi-sq=0 exactly; rust_decimal arithmetic on integer inputs yields exact zero
        assert_eq!(state.stack.x, HpNum::zero());
    }

    /// k out of domain for ΣEFXSQ (mirrors ΣXSQEV behavior).
    #[test]
    fn efxsq_k_out_of_range_returns_domain_error() {
        let mut state = CalcState::new();
        state.regs[STAT1_XSQEV_K_REG] = HpNum::zero();
        assert_eq!(op_sigma_efxsq(&mut state).unwrap_err(), HpError::Domain); // LINT-EXEMPT: error-type comparison, no HpNum; lookahead false positive from adjacent HpNum setup line
        state.regs[STAT1_XSQEV_K_REG] = HpNum::from(STAT1_XSQEV_KMAX as i32 + 1);
        assert_eq!(op_sigma_efxsq(&mut state).unwrap_err(), HpError::Domain);
    }

    // ── ΣCTKKK / ΣCTKK tests (Plan 33-06) ───────────────────────────────────

    /// ΣCTKKK oracle: 2×3 table [[10,20,30],[40,50,60]].
    ///
    /// Manual derivation:
    ///   Row sums: 60, 150;  col sums: 50, 70, 90;  grand: 210.
    ///   Expected:
    ///     E(0,0) = 60·50/210 = 14.2857...  O=10  → (10−14.286)²/14.286 = 1.2857
    ///     E(0,1) = 60·70/210 = 20          O=20  → 0
    ///     E(0,2) = 60·90/210 = 25.714      O=30  → (4.286)²/25.714 = 0.7143
    ///     E(1,0) = 150·50/210 = 35.714     O=40  → (4.286)²/35.714 = 0.5143
    ///     E(1,1) = 150·70/210 = 50         O=50  → 0
    ///     E(1,2) = 150·90/210 = 64.286     O=60  → (4.286)²/64.286 = 0.2857
    ///   Total χ² ≈ 2.8000
    ///
    /// scipy.stats.chi2_contingency([[10,20,30],[40,50,60]], correction=False)
    ///   returns chi² ≈ 2.8 (NOT 4.286 per SPEC.md Req. 28).
    /// SPEC.md Req. 28 says ≈ 4.286 — that value is WRONG. Test asserts
    /// scipy-correct 2.8; SPEC.md amendment gated to Phase 35.
    #[test]
    fn ctkkk_2x3_oracle() {
        let mut state = CalcState::new();
        state.regs[STAT1_CTKKK_R_REG] = HpNum::from(2i32);
        state.regs[STAT1_CTKKK_C_REG] = HpNum::from(3i32);
        // Cells row-major: R02=10, R03=20, R04=30, R05=40, R06=50, R07=60
        state.regs[STAT1_CTKKK_CELL_BASE_REG] = HpNum::from(10i32);
        state.regs[STAT1_CTKKK_CELL_BASE_REG + 1] = HpNum::from(20i32);
        state.regs[STAT1_CTKKK_CELL_BASE_REG + 2] = HpNum::from(30i32);
        state.regs[STAT1_CTKKK_CELL_BASE_REG + 3] = HpNum::from(40i32);
        state.regs[STAT1_CTKKK_CELL_BASE_REG + 4] = HpNum::from(50i32);
        state.regs[STAT1_CTKKK_CELL_BASE_REG + 5] = HpNum::from(60i32);
        op_sigma_ctkkk(&mut state).unwrap();
        let expected = 2.8_f64;
        // Tolerance 1e-7: the chained Decimal divisions for non-integer
        // expected values (e.g. 60·50/210 = 14.2857...) accumulate
        // last-digit rounding through the rust_decimal 10-significant-
        // digit floor. Per SPEC.md Req. 46 "Two-level tolerance discipline"
        // iterative cross-product chains use 1e-7 even though the
        // top-level formula is closed-form.
        assert_relative_eq!(x_as_f64(&state), expected, max_relative = 1e-7);
    }

    /// ΣCTKK oracle: 2×2 table [[10,20],[30,40]].
    ///
    /// Manual derivation:
    ///   Row sums: 30, 70;  col sums: 40, 60;  grand: 100.
    ///   Expected: E(0,0)=12, E(0,1)=18, E(1,0)=28, E(1,1)=42
    ///   χ² = (10−12)²/12 + (20−18)²/18 + (30−28)²/28 + (40−42)²/42
    ///      = 4/12 + 4/18 + 4/28 + 4/42
    ///      = 0.3333 + 0.2222 + 0.1429 + 0.0952 = 0.7937 (approx)
    ///
    /// `scipy.stats.chi2_contingency([[10,20],[30,40]], correction=False)`
    ///   returns chi² ≈ 0.7937 (NOT 0.397 per SPEC.md Req. 29). The SPEC
    ///   value 0.397 reverse-engineers to roughly half the correct value
    ///   — possibly Yates'-corrected with extra adjustment. OM does not
    ///   document Yates' correction; we follow OM-faithful "no correction".
    ///   SPEC.md amendment gated to Phase 35.
    #[test]
    fn ctkk_2x2_oracle() {
        let mut state = CalcState::new();
        state.regs[STAT1_CTKKK_R_REG] = HpNum::from(2i32);
        state.regs[STAT1_CTKKK_C_REG] = HpNum::from(2i32);
        state.regs[STAT1_CTKKK_CELL_BASE_REG] = HpNum::from(10i32);
        state.regs[STAT1_CTKKK_CELL_BASE_REG + 1] = HpNum::from(20i32);
        state.regs[STAT1_CTKKK_CELL_BASE_REG + 2] = HpNum::from(30i32);
        state.regs[STAT1_CTKKK_CELL_BASE_REG + 3] = HpNum::from(40i32);
        op_sigma_ctkk(&mut state).unwrap();
        // 4/12 + 4/18 + 4/28 + 4/42
        let expected = 4.0_f64 / 12.0 + 4.0 / 18.0 + 4.0 / 28.0 + 4.0 / 42.0;
        assert_relative_eq!(x_as_f64(&state), expected, max_relative = 1e-9);
    }

    /// Perfect independence: O_ij = (Rᵢ·Cⱼ)/T exactly → χ² = 0.
    #[test]
    fn ctkkk_perfect_independence_yields_zero() {
        let mut state = CalcState::new();
        state.regs[STAT1_CTKKK_R_REG] = HpNum::from(2i32);
        state.regs[STAT1_CTKKK_C_REG] = HpNum::from(2i32);
        // Construct a table where each cell equals (RᵢCⱼ)/T:
        // R0=20, R1=80, C0=50, C1=50, T=100 → cells: 10, 10, 40, 40.
        state.regs[STAT1_CTKKK_CELL_BASE_REG] = HpNum::from(10i32);
        state.regs[STAT1_CTKKK_CELL_BASE_REG + 1] = HpNum::from(10i32);
        state.regs[STAT1_CTKKK_CELL_BASE_REG + 2] = HpNum::from(40i32);
        state.regs[STAT1_CTKKK_CELL_BASE_REG + 3] = HpNum::from(40i32);
        op_sigma_ctkkk(&mut state).unwrap();
        // LINT-EXEMPT: integer-equality — perfect independence (observed == expected) gives chi-sq=0 exactly; rust_decimal on integer inputs yields exact zero
        assert_eq!(state.stack.x, HpNum::zero());
    }

    /// Zero grand total → Domain error (degenerate marginals).
    #[test]
    fn ctkkk_zero_grand_total_returns_domain_error() {
        let mut state = CalcState::new();
        state.regs[STAT1_CTKKK_R_REG] = HpNum::from(2i32);
        state.regs[STAT1_CTKKK_C_REG] = HpNum::from(2i32);
        state.regs[STAT1_CTKKK_CELL_BASE_REG] = HpNum::zero();
        state.regs[STAT1_CTKKK_CELL_BASE_REG + 1] = HpNum::zero();
        state.regs[STAT1_CTKKK_CELL_BASE_REG + 2] = HpNum::zero();
        state.regs[STAT1_CTKKK_CELL_BASE_REG + 3] = HpNum::zero();
        assert_eq!(op_sigma_ctkkk(&mut state).unwrap_err(), HpError::Domain);
    }

    /// Zero row-sum produces zero expected cells → Domain error.
    #[test]
    fn ctkkk_zero_expected_returns_domain_error() {
        let mut state = CalcState::new();
        state.regs[STAT1_CTKKK_R_REG] = HpNum::from(2i32);
        state.regs[STAT1_CTKKK_C_REG] = HpNum::from(2i32);
        // Row 0 all zero → R0=0 → E(0,*)=0.
        state.regs[STAT1_CTKKK_CELL_BASE_REG] = HpNum::zero();
        state.regs[STAT1_CTKKK_CELL_BASE_REG + 1] = HpNum::zero();
        state.regs[STAT1_CTKKK_CELL_BASE_REG + 2] = HpNum::from(30i32);
        state.regs[STAT1_CTKKK_CELL_BASE_REG + 3] = HpNum::from(40i32);
        assert_eq!(op_sigma_ctkkk(&mut state).unwrap_err(), HpError::Domain);
    }

    /// ΣCTKK rejects 3×3 (dim cap 2).
    #[test]
    fn ctkk_3x3_returns_domain_error() {
        let mut state = CalcState::new();
        state.regs[STAT1_CTKKK_R_REG] = HpNum::from(3i32);
        state.regs[STAT1_CTKKK_C_REG] = HpNum::from(3i32);
        assert_eq!(op_sigma_ctkk(&mut state).unwrap_err(), HpError::Domain);
    }

    /// SIZE-floor guard for ΣCTKKK.
    #[test]
    fn ctkkk_size_floor_guard() {
        let mut state = CalcState::new();
        state.regs.truncate(STAT1_MAX_REG);
        assert_eq!(op_sigma_ctkkk(&mut state).unwrap_err(), HpError::InvalidOp);
    }
}
