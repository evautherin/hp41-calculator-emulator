// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `stat1::anova` — ANOVA family Ops (Plan 33-06): ΣAOVONE (one-way),
//! ΣAOVTWO (two-way no replications), ΣANOCOV (one-way ANCOVA) per OM
//! 00041-90030 (p. 20, 23, 28). All three are consumers of pre-
//! accumulated Σ-block state. Per-Op layouts via named consts (P21):
//! ΣAOVONE per-group stride 4 at [`STAT1_AOV_GROUP_BASE_REG`]; ΣAOVTWO
//! row+col sums after R05; ΣANOCOV per-group stride 5 at R07.
//!
//! SPEC.md Req. 11 drift: F=100.0 claimed; scipy.stats.f_oneway and
//! manual both confirm F=50.0 for [1..5]/[6..10]/[11..15] (SSB=250,
//! SSW=30, df=2/12, F=125/2.5=50). Phase 35 amendment gated.
//!
//! Refs: OM 00041-90030; NPS55-84-003 §ZA-3; scipy.stats.f_oneway.

use crate::error::HpError;
use crate::num::HpNum;
use crate::ops::stat1::{
    STAT1_ANOCOV_GRAND_SUMSQ_X_REG, STAT1_ANOCOV_GROUP_BASE_REG, STAT1_ANOCOV_GROUP_N_OFFSET,
    STAT1_ANOCOV_GROUP_STRIDE, STAT1_ANOCOV_GROUP_SUMSQ_Y_OFFSET, STAT1_ANOCOV_GROUP_SUM_XY_OFFSET,
    STAT1_ANOCOV_GROUP_SUM_X_OFFSET, STAT1_ANOCOV_GROUP_SUM_Y_OFFSET, STAT1_AOVTWO_C_REG,
    STAT1_AOVTWO_DIM_MAX, STAT1_AOVTWO_GRAND_SUMSQ_REG, STAT1_AOVTWO_GRAND_SUM_REG,
    STAT1_AOVTWO_ROW_BASE_REG, STAT1_AOVTWO_R_REG, STAT1_AOV_GRAND_SUMSQ_REG,
    STAT1_AOV_GRAND_SUM_REG, STAT1_AOV_GROUP_BASE_REG, STAT1_AOV_GROUP_N_OFFSET,
    STAT1_AOV_GROUP_STRIDE, STAT1_AOV_GROUP_SUMSQ_OFFSET, STAT1_AOV_GROUP_SUM_OFFSET,
    STAT1_AOV_KMAX, STAT1_AOV_K_REG, STAT1_AOV_N_REG, STAT1_MAX_REG,
};
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;
use rust_decimal::prelude::ToPrimitive;

// ── SIZE-floor guard helper ────────────────────────────────────────────────

#[inline]
fn require_stat1_size_floor(state: &CalcState) -> Result<(), HpError> {
    if state.regs.len() < STAT1_MAX_REG + 1 {
        return Err(HpError::InvalidOp);
    }
    Ok(())
}

/// Decode the group count k from an HpNum register value. Truncates
/// toward zero per HP-41 INT convention and validates the bound
/// `2 ≤ k ≤ cap` (one-way ANOVA needs at least 2 groups). Same P21
/// mitigation pattern as [`crate::ops::stat1::nonparam::decode_category_count`].
fn decode_group_count(k_num: &HpNum, cap: usize) -> Result<usize, HpError> {
    let k_int = k_num.trunc_int().inner();
    let k = k_int.to_usize().ok_or(HpError::Domain)?;
    if !(2..=cap).contains(&k) {
        return Err(HpError::Domain);
    }
    Ok(k)
}

// ── ΣAOVONE — One-way ANOVA F-ratio ────────────────────────────────────────

/// ΣAOVONE — One-way ANOVA F-ratio across k groups (closed-form).
///
/// Reads k from R00 and per-group `(Σxᵢ, Σxᵢ², nᵢ)` from
/// `STAT1_AOV_GROUP_BASE_REG + STAT1_AOV_GROUP_STRIDE * i` (stride 4).
/// `SSB = Σ nᵢ(x̄ᵢ−x̄)²`, `SSW = Σ(Σxᵢ² − nᵢ·x̄ᵢ²)`; df_between=k−1,
/// df_within=N−k; `F = (SSB/df_between)/(SSW/df_within)`. Pushes F to X.
///
/// Errors: `InvalidOp` on SIZE-floor guard or N ≤ k; `Domain` if k out
/// of `[2..STAT1_AOV_KMAX]`; `DivideByZero` if SSW == 0.
///
/// Source: OM 00041-90030 §ΣAOVONE (p. 20). Oracle: scipy.stats.f_oneway.
pub fn op_sigma_aovone(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;
    let k = decode_group_count(
        &state.regs[STAT1_AOV_K_REG].numeric_or_zero(),
        STAT1_AOV_KMAX,
    )?;

    // First pass: grand totals (N, Σx, Σx²) — recomputed for atomic
    // correctness (do NOT trust R01..R03 contents).
    let mut grand_n = HpNum::zero();
    let mut grand_sum = HpNum::zero();
    let mut grand_sumsq = HpNum::zero();
    for i in 0..k {
        let base = STAT1_AOV_GROUP_BASE_REG + STAT1_AOV_GROUP_STRIDE * i;
        let n_i = state.regs[base + STAT1_AOV_GROUP_N_OFFSET].numeric_or_zero();
        if n_i.is_zero() {
            return Err(HpError::InvalidOp);
        }
        grand_n = grand_n.checked_add(&n_i)?;
        grand_sum = grand_sum
            .checked_add(&state.regs[base + STAT1_AOV_GROUP_SUM_OFFSET].numeric_or_zero())?;
        grand_sumsq = grand_sumsq
            .checked_add(&state.regs[base + STAT1_AOV_GROUP_SUMSQ_OFFSET].numeric_or_zero())?;
    }

    let k_hp = HpNum::from(rust_decimal::Decimal::from(k));
    let df_within = grand_n.checked_sub(&k_hp)?;
    if df_within.inner() <= rust_decimal::Decimal::ZERO || grand_n.is_zero() {
        return Err(HpError::InvalidOp);
    }
    let grand_mean = grand_sum.checked_div(&grand_n)?;

    // Second pass: SSB and SSW.
    let mut ssb = HpNum::zero();
    let mut ssw = HpNum::zero();
    for i in 0..k {
        let base = STAT1_AOV_GROUP_BASE_REG + STAT1_AOV_GROUP_STRIDE * i;
        let sum_i = state.regs[base + STAT1_AOV_GROUP_SUM_OFFSET].numeric_or_zero();
        let sumsq_i = state.regs[base + STAT1_AOV_GROUP_SUMSQ_OFFSET].numeric_or_zero();
        let n_i = state.regs[base + STAT1_AOV_GROUP_N_OFFSET].numeric_or_zero();
        let mean_i = sum_i.checked_div(&n_i)?;
        let dev_i = mean_i.checked_sub(&grand_mean)?;
        ssb = ssb.checked_add(&n_i.checked_mul(&dev_i.checked_sq()?)?)?;
        ssw = ssw.checked_add(&sumsq_i.checked_sub(&n_i.checked_mul(&mean_i.checked_sq()?)?)?)?;
    }

    // Update grand-block registers for downstream consumers.
    state.regs[STAT1_AOV_GRAND_SUMSQ_REG] = grand_sumsq.into();
    state.regs[STAT1_AOV_GRAND_SUM_REG] = grand_sum.into();
    state.regs[STAT1_AOV_N_REG] = grand_n.into();

    let df_between = k_hp.checked_sub(&HpNum::from(1i32))?;
    if df_between.is_zero() {
        return Err(HpError::InvalidOp);
    }
    let ms_between = ssb.checked_div(&df_between)?;
    let ms_within = ssw.checked_div(&df_within)?;
    if ms_within.is_zero() {
        return Err(HpError::DivideByZero);
    }
    let f = ms_between.checked_div(&ms_within)?;
    state.stack.lift_enabled = true;
    enter_number(state, f);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

// ── ΣAOVTWO — Two-way ANOVA (No Replications) ──────────────────────────────

/// ΣAOVTWO — Two-way ANOVA without replications.
///
/// Reads r/c from R00/R01, N=r·c from R02, grand Σx² from R03, grand Σx
/// from R04, row sums from R05.. (r regs), col sums immediately after
/// (c regs). Computes SS_total, SS_row, SS_col, SS_error; F_row and
/// F_col via standard two-way ANOVA formulas. Pushes F_row to Y and
/// F_col to X via the `op_mean` push-twice convention.
///
/// Errors: `InvalidOp` on SIZE-floor; `Domain` if r/c out of `[2..4]`;
/// `DivideByZero` if SS_error == 0.
///
/// Source: OM 00041-90030 §ΣAOVTWO (p. 23).
pub fn op_sigma_aovtwo(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;
    // P21 mitigation (REVIEW.md WR-01): row/col counts now read via
    // named consts STAT1_AOVTWO_R_REG / STAT1_AOVTWO_C_REG; the
    // dimension cap is the OM-cited STAT1_AOVTWO_DIM_MAX rather than
    // a bare `4` literal.
    let r = decode_group_count(
        &state.regs[STAT1_AOVTWO_R_REG].numeric_or_zero(),
        STAT1_AOVTWO_DIM_MAX,
    )?;
    let c = decode_group_count(
        &state.regs[STAT1_AOVTWO_C_REG].numeric_or_zero(),
        STAT1_AOVTWO_DIM_MAX,
    )?;
    // Bounds check: row + col marginal sums occupy
    // STAT1_AOVTWO_ROW_BASE_REG .. +(r + c) and must fit within the
    // global Stat 1 register footprint.
    if STAT1_AOVTWO_ROW_BASE_REG + r + c > STAT1_MAX_REG + 1 {
        return Err(HpError::Domain);
    }
    let grand_sumsq = state.regs[STAT1_AOVTWO_GRAND_SUMSQ_REG].numeric_or_zero();
    let grand_sum = state.regs[STAT1_AOVTWO_GRAND_SUM_REG].numeric_or_zero();
    let n_hp = HpNum::from(rust_decimal::Decimal::from(r * c));
    if n_hp.is_zero() {
        return Err(HpError::InvalidOp);
    }
    let grand_mean = grand_sum.checked_div(&n_hp)?;
    let c_hp = HpNum::from(rust_decimal::Decimal::from(c));
    let r_hp = HpNum::from(rust_decimal::Decimal::from(r));
    let ss_total = grand_sumsq.checked_sub(&n_hp.checked_mul(&grand_mean.checked_sq()?)?)?;
    let row_base = STAT1_AOVTWO_ROW_BASE_REG;
    let col_base = row_base + r;
    // SS_row = c · Σᵢ(Rᵢ/c − x̄)²;  SS_col = r · Σⱼ(Cⱼ/r − x̄)²
    let mut ss_row = HpNum::zero();
    for i in 0..r {
        let row_mean = state.regs[row_base + i]
            .numeric_or_zero()
            .checked_div(&c_hp)?;
        ss_row = ss_row.checked_add(&row_mean.checked_sub(&grand_mean)?.checked_sq()?)?;
    }
    ss_row = c_hp.checked_mul(&ss_row)?;
    let mut ss_col = HpNum::zero();
    for j in 0..c {
        let col_mean = state.regs[col_base + j]
            .numeric_or_zero()
            .checked_div(&r_hp)?;
        ss_col = ss_col.checked_add(&col_mean.checked_sub(&grand_mean)?.checked_sq()?)?;
    }
    ss_col = r_hp.checked_mul(&ss_col)?;
    let ss_error = ss_total.checked_sub(&ss_row)?.checked_sub(&ss_col)?;
    let df_row = HpNum::from(rust_decimal::Decimal::from(r - 1));
    let df_col = HpNum::from(rust_decimal::Decimal::from(c - 1));
    let df_error = HpNum::from(rust_decimal::Decimal::from((r - 1) * (c - 1)));
    if df_error.is_zero() {
        return Err(HpError::InvalidOp);
    }
    let ms_error = ss_error.checked_div(&df_error)?;
    if ms_error.is_zero() {
        return Err(HpError::DivideByZero);
    }
    let f_row = ss_row.checked_div(&df_row)?.checked_div(&ms_error)?;
    let f_col = ss_col.checked_div(&df_col)?.checked_div(&ms_error)?;
    // Push F_row then F_col (F_col lands at X, F_row at Y).
    state.stack.lift_enabled = true;
    enter_number(state, f_row);
    apply_lift_effect(state, LiftEffect::Enable);
    state.stack.lift_enabled = true;
    enter_number(state, f_col);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

// ── ΣANOCOV — One-way ANCOVA F-ratio ───────────────────────────────────────

/// ΣANOCOV — One-way ANCOVA F-ratio. Per-group stride 5 from R07:
/// `(Σy, Σy², n, Σx, Σxy)`; grand Σx² at R04. Computes within-group
/// SSy/SSx/SSxy + total SST_y/SST_x/SST_xy. F numerator is the
/// difference between two residuals (Searle 1971 §10.3):
/// null_resid = `SST_y − SST_xy²/SST_x` (common slope+intercept),
/// alt_resid = `SSWy − SSWxy²/SSWx` (common slope, group-specific
/// intercept). Then `SSB_adj = null − alt` and
/// `F = (SSB_adj/(k−1)) / (alt_resid/(N−k−1))`. Pushes F to X.
///
/// Errors: `InvalidOp` on SIZE-floor / N−k−1 ≤ 0; `Domain` if k out
/// of `[2..STAT1_AOV_KMAX]`; `DivideByZero` if SSWx == 0 or SST_x == 0.
///
/// Source: OM 00041-90030 §ΣANOCOV (p. 28); NPS55-84-003 §ZA-3.
pub fn op_sigma_anocov(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;

    let k_num = state.regs[STAT1_AOV_K_REG].numeric_or_zero();
    let k = decode_group_count(&k_num, STAT1_AOV_KMAX)?;

    // Per-group blocks: stride 5, base R07 (Σy, Σy², n, Σx, Σxy).
    // Grand block: R04 = grand Σx² (covariate). OM ANCOVA computes
    // SSWx via the identity SSWx = grand_Σx² − Σᵢ(Σxᵢ)²/nᵢ.
    //
    // P21 mitigation (REVIEW.md WR-01): per-group base/stride and the
    // grand covariate Σx² register address now route through named
    // consts STAT1_ANOCOV_GROUP_BASE_REG / _GROUP_STRIDE /
    // _GRAND_SUMSQ_X_REG / _GROUP_*_OFFSET rather than bare literals.
    let group_base = STAT1_ANOCOV_GROUP_BASE_REG;
    let group_stride = STAT1_ANOCOV_GROUP_STRIDE;
    if group_base + group_stride * k > STAT1_MAX_REG + 1 {
        return Err(HpError::Domain);
    }
    let grand_sumsq_x = state.regs[STAT1_ANOCOV_GRAND_SUMSQ_X_REG].numeric_or_zero();

    let mut grand_n = HpNum::zero();
    let mut grand_sum_y = HpNum::zero();
    let mut grand_sum_x = HpNum::zero();
    let mut grand_sumsq_y = HpNum::zero();
    let mut grand_sum_xy = HpNum::zero();
    let mut ssbx_terms = HpNum::zero();
    let mut ssw_y = HpNum::zero();
    let mut ssw_xy = HpNum::zero();
    for i in 0..k {
        let base = group_base + group_stride * i;
        let sum_y = state.regs[base + STAT1_ANOCOV_GROUP_SUM_Y_OFFSET].numeric_or_zero();
        let sumsq_y = state.regs[base + STAT1_ANOCOV_GROUP_SUMSQ_Y_OFFSET].numeric_or_zero();
        let n_i = state.regs[base + STAT1_ANOCOV_GROUP_N_OFFSET].numeric_or_zero();
        let sum_x = state.regs[base + STAT1_ANOCOV_GROUP_SUM_X_OFFSET].numeric_or_zero();
        let sum_xy = state.regs[base + STAT1_ANOCOV_GROUP_SUM_XY_OFFSET].numeric_or_zero();
        if n_i.is_zero() {
            return Err(HpError::InvalidOp);
        }
        ssw_y =
            ssw_y.checked_add(&sumsq_y.checked_sub(&sum_y.checked_sq()?.checked_div(&n_i)?)?)?;
        ssw_xy = ssw_xy
            .checked_add(&sum_xy.checked_sub(&sum_x.checked_mul(&sum_y)?.checked_div(&n_i)?)?)?;
        ssbx_terms = ssbx_terms.checked_add(&sum_x.checked_sq()?.checked_div(&n_i)?)?;
        grand_n = grand_n.checked_add(&n_i)?;
        grand_sum_y = grand_sum_y.checked_add(&sum_y)?;
        grand_sum_x = grand_sum_x.checked_add(&sum_x)?;
        grand_sumsq_y = grand_sumsq_y.checked_add(&sumsq_y)?;
        grand_sum_xy = grand_sum_xy.checked_add(&sum_xy)?;
    }

    if grand_n.is_zero() {
        return Err(HpError::InvalidOp);
    }
    let k_hp = HpNum::from(rust_decimal::Decimal::from(k));
    let one = HpNum::from(1i32);
    let df_within_adj = grand_n.checked_sub(&k_hp)?.checked_sub(&one)?;
    if df_within_adj.inner() <= rust_decimal::Decimal::ZERO {
        return Err(HpError::InvalidOp);
    }
    let df_between = k_hp.checked_sub(&one)?;
    if df_between.is_zero() {
        return Err(HpError::InvalidOp);
    }
    let ssw_x = grand_sumsq_x.checked_sub(&ssbx_terms)?;
    if ssw_x.is_zero() {
        return Err(HpError::DivideByZero);
    }
    let sst_x = grand_sumsq_x.checked_sub(&grand_sum_x.checked_sq()?.checked_div(&grand_n)?)?;
    let sst_y = grand_sumsq_y.checked_sub(&grand_sum_y.checked_sq()?.checked_div(&grand_n)?)?;
    let sst_xy = grand_sum_xy.checked_sub(
        &grand_sum_x
            .checked_mul(&grand_sum_y)?
            .checked_div(&grand_n)?,
    )?;
    if sst_x.is_zero() {
        return Err(HpError::DivideByZero);
    }
    let null_residual = sst_y.checked_sub(&sst_xy.checked_sq()?.checked_div(&sst_x)?)?;
    let alt_residual = ssw_y.checked_sub(&ssw_xy.checked_sq()?.checked_div(&ssw_x)?)?;
    let ssb_adj = null_residual.checked_sub(&alt_residual)?;
    let ms_between = ssb_adj.checked_div(&df_between)?;
    let ms_within_adj = alt_residual.checked_div(&df_within_adj)?;
    if ms_within_adj.is_zero() {
        return Err(HpError::DivideByZero);
    }
    let f = ms_between.checked_div(&ms_within_adj)?;
    state.stack.lift_enabled = true;
    enter_number(state, f);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn to_f64(v: &HpNum) -> f64 {
        v.inner().to_f64().expect("finite")
    }

    /// Load a per-group block at index i (Σxᵢ, Σxᵢ², nᵢ) into the
    /// SIZE-020 layout for ΣAOVONE.
    fn load_group(state: &mut CalcState, i: usize, sum: f64, sumsq: f64, n: i32) {
        let base = STAT1_AOV_GROUP_BASE_REG + STAT1_AOV_GROUP_STRIDE * i;
        state.regs[base + STAT1_AOV_GROUP_SUM_OFFSET] =
            HpNum::from(rust_decimal::Decimal::from_f64_retain(sum).expect("finite")).into();
        state.regs[base + STAT1_AOV_GROUP_SUMSQ_OFFSET] =
            HpNum::from(rust_decimal::Decimal::from_f64_retain(sumsq).expect("finite")).into();
        state.regs[base + STAT1_AOV_GROUP_N_OFFSET] = HpNum::from(n).into();
    }

    /// ΣAOVONE oracle: 3 groups of 5 samples [1..5], [6..10], [11..15].
    ///
    /// scipy.stats.f_oneway returns F = 50.0 (NOT 100.0 per SPEC.md Req. 11).
    /// See module-level docs for the SPEC.md drift documentation; this
    /// test asserts the scipy-correct value.
    #[test]
    fn aovone_three_groups_of_five_yields_f_50() {
        let mut state = CalcState::new();
        state.regs[STAT1_AOV_K_REG] = HpNum::from(3i32).into();
        // group 0: [1..5] → Σ=15, Σ²=55, n=5
        load_group(&mut state, 0, 15.0, 55.0, 5);
        // group 1: [6..10] → Σ=40, Σ²=330, n=5
        load_group(&mut state, 1, 40.0, 330.0, 5);
        // group 2: [11..15] → Σ=65, Σ²=855, n=5
        load_group(&mut state, 2, 65.0, 855.0, 5);
        op_sigma_aovone(&mut state).unwrap();
        assert_relative_eq!(to_f64(&state.stack.x), 50.0, max_relative = 1e-9);
    }

    /// Two equal groups → F = 0 (no between-group variance).
    #[test]
    fn aovone_identical_groups_yields_f_zero() {
        let mut state = CalcState::new();
        state.regs[STAT1_AOV_K_REG] = HpNum::from(2i32).into();
        load_group(&mut state, 0, 15.0, 55.0, 5);
        load_group(&mut state, 1, 15.0, 55.0, 5);
        op_sigma_aovone(&mut state).unwrap();
        // LINT-EXEMPT: integer-equality — identical groups produce SSB=0 exactly; rust_decimal arithmetic on integer inputs yields exact zero
        assert_eq!(state.stack.x, HpNum::zero());
    }

    /// SIZE-floor guard for ΣAOVONE.
    #[test]
    fn aovone_size_floor_guard() {
        let mut state = CalcState::new();
        state.regs.truncate(STAT1_MAX_REG);
        assert_eq!(op_sigma_aovone(&mut state).unwrap_err(), HpError::InvalidOp);
    }

    /// k out of range rejected (k = 1 and k > KMAX).
    #[test]
    fn aovone_k_out_of_range_returns_domain_error() {
        let mut state = CalcState::new();
        state.regs[STAT1_AOV_K_REG] = HpNum::from(1i32).into();
        assert_eq!(op_sigma_aovone(&mut state).unwrap_err(), HpError::Domain); // LINT-EXEMPT: error-type comparison, no HpNum; lookahead false positive from adjacent HpNum line
        state.regs[STAT1_AOV_K_REG] = HpNum::from(STAT1_AOV_KMAX as i32 + 1).into();
        assert_eq!(op_sigma_aovone(&mut state).unwrap_err(), HpError::Domain);
    }

    /// ΣAOVTWO oracle: 3×4 dataset.
    ///
    /// Construct a simple additive dataset: cell(i,j) = i + j + 1.
    /// Rows i = 0..2; cols j = 0..3. Cells:
    ///   (1,2,3,4)
    ///   (2,3,4,5)
    ///   (3,4,5,6)
    /// Σall = 42, Σall² = 138, N = 12, mean = 3.5.
    /// Row sums: 10, 14, 18 → mean 2.5, 3.5, 4.5
    /// Col sums: 6, 9, 12, 15 → mean 2, 3, 4, 5
    /// SS_total = 138 − 12·3.5² = 138 − 147 = −9 (!?) — recompute.
    /// Actually: Σ_x² for cells (1,2,3,4,2,3,4,5,3,4,5,6)
    ///   = 1+4+9+16+4+9+16+25+9+16+25+36 = 170
    /// So R03 = grand Σx² = 170; SS_total = 170 − 147 = 23.
    ///   SS_row = 4·((2.5−3.5)² + 0 + (4.5−3.5)²) = 4·(1+0+1) = 8
    ///   SS_col = 3·((2−3.5)² + (3−3.5)² + (4−3.5)² + (5−3.5)²)
    ///          = 3·(2.25+0.25+0.25+2.25) = 3·5 = 15
    ///   SS_error = 23 − 8 − 15 = 0 → DivideByZero (perfect additive fit)
    /// Use a non-additive dataset instead — add noise to one cell.
    ///
    /// Replace cell(0,0)=1 with cell(0,0)=10:
    /// Cells: (10,2,3,4,2,3,4,5,3,4,5,6); Σ = 51; Σ² = 233; N = 12; mean = 4.25.
    /// Row sums: 19, 14, 18; col sums: 15, 9, 12, 15.
    /// SS_total = 233 − 12·4.25² = 233 − 216.75 = 16.25.
    /// SS_row = 4·((19/4 − 4.25)² + (14/4 − 4.25)² + (18/4 − 4.25)²)
    ///        = 4·((0.5)² + (-0.75)² + (0.25)²) = 4·(0.25+0.5625+0.0625) = 3.5
    /// SS_col = 3·((15/3 − 4.25)² + (9/3 − 4.25)² + (12/3 − 4.25)² + (15/3 − 4.25)²)
    ///        = 3·((0.75)² + (-1.25)² + (-0.25)² + (0.75)²)
    ///        = 3·(0.5625 + 1.5625 + 0.0625 + 0.5625) = 3·2.75 = 8.25
    /// SS_error = 16.25 − 3.5 − 8.25 = 4.5
    /// df_row=2, df_col=3, df_error=6
    /// F_row = (3.5/2) / (4.5/6) = 1.75/0.75 = 2.3333...
    /// F_col = (8.25/3) / (4.5/6) = 2.75/0.75 = 3.6667...
    #[test]
    fn aovtwo_3x4_oracle() {
        let mut state = CalcState::new();
        state.regs[0] = HpNum::from(3i32).into(); // r
        state.regs[1] = HpNum::from(4i32).into(); // c
        state.regs[2] = HpNum::from(12i32).into(); // N
                                                   // grand Σx² and Σx
        state.regs[3] = HpNum::from(rust_decimal::Decimal::from(233i32)).into();
        state.regs[4] = HpNum::from(rust_decimal::Decimal::from(51i32)).into();
        // row sums R05, R06, R07 = 19, 14, 18
        state.regs[5] = HpNum::from(19i32).into();
        state.regs[6] = HpNum::from(14i32).into();
        state.regs[7] = HpNum::from(18i32).into();
        // col sums R08, R09, R10, R11 = 15, 9, 12, 15
        state.regs[8] = HpNum::from(15i32).into();
        state.regs[9] = HpNum::from(9i32).into();
        state.regs[10] = HpNum::from(12i32).into();
        state.regs[11] = HpNum::from(15i32).into();
        op_sigma_aovtwo(&mut state).unwrap();
        // F_col lands at X, F_row at Y
        assert_relative_eq!(to_f64(&state.stack.x), 11.0 / 3.0, max_relative = 1e-7);
        assert_relative_eq!(to_f64(&state.stack.y), 7.0 / 3.0, max_relative = 1e-7);
    }

    /// SIZE-floor guard for ΣAOVTWO.
    #[test]
    fn aovtwo_size_floor_guard() {
        let mut state = CalcState::new();
        state.regs.truncate(STAT1_MAX_REG);
        assert_eq!(op_sigma_aovtwo(&mut state).unwrap_err(), HpError::InvalidOp);
    }

    /// ΣANOCOV oracle: 2 groups with linear-relationship covariate.
    ///
    /// Group 0 (x, y) pairs: (1,2), (2,4), (3,6)
    ///   n=3, Σx=6, Σy=12, Σx²=14, Σy²=56, Σxy=28
    /// Group 1: (4,3), (5,5), (6,7)
    ///   n=3, Σx=15, Σy=15, Σx²=77, Σy²=83, Σxy=78
    /// Grand: N=6, Σx=21, Σy=27, Σx²=91, Σy²=139, Σxy=106
    ///
    /// SSW_y = (56 − 144/3) + (83 − 225/3) = 8 + 8 = 16
    /// SSW_x = (14 − 36/3) + (77 − 225/3) = 2 + 2 = 4
    /// SSW_xy = (28 − 6·12/3) + (78 − 15·15/3) = 4 + 3 = 7
    /// SST_y = 139 − 27²/6 = 139 − 121.5 = 17.5
    /// SST_x = 91 − 21²/6 = 91 − 73.5 = 17.5
    /// SST_xy = 106 − 21·27/6 = 106 − 94.5 = 11.5
    ///
    /// alt_residual = 16 − 49/4 = 16 − 12.25 = 3.75
    /// null_residual = 17.5 − 132.25/17.5 = 17.5 − 7.557142857... = 9.942857...
    /// SSB_adj = 9.942857... − 3.75 = 6.192857...
    /// df_between = 1; df_within_adj = 6 − 2 − 1 = 3
    /// F = (6.192857/1) / (3.75/3) = 6.192857 / 1.25 = 4.954285...
    #[test]
    fn anocov_two_group_oracle() {
        let mut state = CalcState::new();
        state.regs[STAT1_AOV_K_REG] = HpNum::from(2i32).into();
        // grand-block at R01..R06; R01 unused by ANCOVA (use 0).
        // R02 = grand Σy² (unused — Op recomputes from per-group),
        // R03 = grand Σy  (recomputed),
        // R04 = grand Σx² → 91
        state.regs[4] = HpNum::from(91i32).into();
        // Per-group blocks at R07..R16 (k=2, stride=5):
        // Group 0: Σy=12, Σy²=56, n=3, Σx=6, Σxy=28
        state.regs[7] = HpNum::from(12i32).into();
        state.regs[8] = HpNum::from(56i32).into();
        state.regs[9] = HpNum::from(3i32).into();
        state.regs[10] = HpNum::from(6i32).into();
        state.regs[11] = HpNum::from(28i32).into();
        // Group 1: Σy=15, Σy²=83, n=3, Σx=15, Σxy=78
        state.regs[12] = HpNum::from(15i32).into();
        state.regs[13] = HpNum::from(83i32).into();
        state.regs[14] = HpNum::from(3i32).into();
        state.regs[15] = HpNum::from(15i32).into();
        state.regs[16] = HpNum::from(78i32).into();
        op_sigma_anocov(&mut state).unwrap();
        let expected = (17.5 - (11.5_f64 * 11.5) / 17.5 - 3.75) / 1.0 / (3.75 / 3.0);
        assert_relative_eq!(to_f64(&state.stack.x), expected, max_relative = 1e-7);
    }

    /// SIZE-floor guard for ΣANOCOV.
    #[test]
    fn anocov_size_floor_guard() {
        let mut state = CalcState::new();
        state.regs.truncate(STAT1_MAX_REG);
        assert_eq!(op_sigma_anocov(&mut state).unwrap_err(), HpError::InvalidOp);
    }
}
