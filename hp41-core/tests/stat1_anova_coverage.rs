// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Coverage-gap closure tests for `hp41-core/src/ops/stat1/anova.rs`
//! (Plan 37-03, STAT-QUAL-03).
//!
//! ## Coverage target
//!
//! At Plan 37-03 start, `stat1/anova.rs` line coverage was 86.50% (98 missed
//! lines, 19 missed regions). This file adds targeted integration tests for the
//! three families of uncovered branches:
//!
//! 1. ΣAOVONE: empty-group guard, N==k edge, SSW==0 (DivideByZero), 4-group path
//! 2. ΣAOVTWO: dimension-overflow guard, DivideByZero (perfect additive fit)
//! 3. ΣANOCOV: N-k-1 ≤ 0 guard, SSWx==0 guard, SST_x==0 guard,
//!    alt_residual==0 guard, bounds guard
//!
//! ## Architecture
//!
//! Tests use `CalcState::new()`, populate the required Σ registers using the
//! named consts from `hp41_core::ops::stat1`, dispatch the Op, and assert on
//! the result variant. Floating-point comparisons use `approx::assert_relative_eq!`
//! at 1e-9 tolerance for closed-form paths; error-variant assertions use `assert_eq!`.

#![allow(clippy::unwrap_used)]

use approx::assert_relative_eq;
use hp41_core::error::HpError;
use hp41_core::num::HpNum;
use hp41_core::ops::stat1::{
    STAT1_ANOCOV_GRAND_SUMSQ_X_REG, STAT1_ANOCOV_GROUP_BASE_REG, STAT1_ANOCOV_GROUP_N_OFFSET,
    STAT1_ANOCOV_GROUP_STRIDE, STAT1_ANOCOV_GROUP_SUMSQ_Y_OFFSET, STAT1_ANOCOV_GROUP_SUM_XY_OFFSET,
    STAT1_ANOCOV_GROUP_SUM_X_OFFSET, STAT1_ANOCOV_GROUP_SUM_Y_OFFSET, STAT1_AOVTWO_C_REG,
    STAT1_AOVTWO_GRAND_SUMSQ_REG, STAT1_AOVTWO_GRAND_SUM_REG, STAT1_AOVTWO_ROW_BASE_REG,
    STAT1_AOVTWO_R_REG, STAT1_AOV_GROUP_BASE_REG, STAT1_AOV_GROUP_N_OFFSET, STAT1_AOV_GROUP_STRIDE,
    STAT1_AOV_GROUP_SUMSQ_OFFSET, STAT1_AOV_GROUP_SUM_OFFSET, STAT1_AOV_K_REG, STAT1_MAX_REG,
};
use hp41_core::ops::{dispatch, Op};
use hp41_core::state::CalcState;
use rust_decimal::prelude::ToPrimitive;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn to_f64(v: &HpNum) -> f64 {
    v.inner().to_f64().expect("finite")
}

/// Load a per-group block at index i (Σxᵢ, Σxᵢ², nᵢ) into the SIZE-020
/// layout for ΣAOVONE.
fn load_aovone_group(state: &mut CalcState, i: usize, sum: i32, sumsq: i32, n: i32) {
    let base = STAT1_AOV_GROUP_BASE_REG + STAT1_AOV_GROUP_STRIDE * i;
    state.regs[base + STAT1_AOV_GROUP_SUM_OFFSET] = HpNum::from(sum);
    state.regs[base + STAT1_AOV_GROUP_SUMSQ_OFFSET] = HpNum::from(sumsq);
    state.regs[base + STAT1_AOV_GROUP_N_OFFSET] = HpNum::from(n);
}

/// Load a per-group block for ΣANOCOV (stride 5: Σy, Σy², n, Σx, Σxy).
fn load_anocov_group(
    state: &mut CalcState,
    i: usize,
    sum_y: i32,
    sumsq_y: i32,
    n: i32,
    sum_x: i32,
    sum_xy: i32,
) {
    let base = STAT1_ANOCOV_GROUP_BASE_REG + STAT1_ANOCOV_GROUP_STRIDE * i;
    state.regs[base + STAT1_ANOCOV_GROUP_SUM_Y_OFFSET] = HpNum::from(sum_y);
    state.regs[base + STAT1_ANOCOV_GROUP_SUMSQ_Y_OFFSET] = HpNum::from(sumsq_y);
    state.regs[base + STAT1_ANOCOV_GROUP_N_OFFSET] = HpNum::from(n);
    state.regs[base + STAT1_ANOCOV_GROUP_SUM_X_OFFSET] = HpNum::from(sum_x);
    state.regs[base + STAT1_ANOCOV_GROUP_SUM_XY_OFFSET] = HpNum::from(sum_xy);
}

// ── ΣAOVONE branch coverage ───────────────────────────────────────────────────

/// Empty-group guard: one group has n_i = 0.
///
/// ΣAOVONE checks n_i.is_zero() inside the first-pass loop and returns
/// InvalidOp immediately. This covers the `if n_i.is_zero()` branch at
/// anova.rs line 83.
#[test]
fn aovone_empty_group_returns_invalid_op() {
    let mut state = CalcState::new();
    state.regs[STAT1_AOV_K_REG] = HpNum::from(2i32);
    // group 0: populated
    load_aovone_group(&mut state, 0, 10, 30, 3);
    // group 1: n = 0 (empty group — triggers the n_i.is_zero() guard)
    load_aovone_group(&mut state, 1, 0, 0, 0);
    let err = dispatch(&mut state, Op::SigmaAovone).unwrap_err();
    // LINT-EXEMPT: error-type comparison — no HpNum involved
    assert_eq!(err, HpError::InvalidOp);
}

/// N == k edge: two groups each with one sample → df_within = N - k = 0.
///
/// ΣAOVONE returns InvalidOp when df_within ≤ 0 (no within-group degrees
/// of freedom). Covers the `if df_within.inner() <= ZERO` branch at
/// anova.rs line 91.
#[test]
fn aovone_nk_equal_returns_invalid_op() {
    let mut state = CalcState::new();
    state.regs[STAT1_AOV_K_REG] = HpNum::from(2i32);
    // Two groups of n=1 each: N=2, k=2, df_within = N-k = 0
    load_aovone_group(&mut state, 0, 5, 25, 1);
    load_aovone_group(&mut state, 1, 10, 100, 1);
    let err = dispatch(&mut state, Op::SigmaAovone).unwrap_err();
    // LINT-EXEMPT: error-type comparison
    assert_eq!(err, HpError::InvalidOp);
}

/// SSW == 0: all values within each group are identical → ms_within = 0.
///
/// ΣAOVONE returns DivideByZero when ms_within is exactly zero. This covers
/// the `if ms_within.is_zero()` branch at anova.rs line 121.
/// Dataset: 2 groups of 3, each group has all-equal values but different means.
/// Group 0: [5,5,5] → Σx=15, Σx²=75, n=3; SSW_0 = 75 - 3·5² = 75-75 = 0
/// Group 1: [10,10,10] → Σx=30, Σx²=300, n=3; SSW_1 = 300 - 3·10² = 0
/// Total SSW = 0 → DivideByZero.
#[test]
fn aovone_ssw_zero_returns_divide_by_zero() {
    let mut state = CalcState::new();
    state.regs[STAT1_AOV_K_REG] = HpNum::from(2i32);
    // Group 0: three 5s → Σx=15, Σx²=75, n=3
    load_aovone_group(&mut state, 0, 15, 75, 3);
    // Group 1: three 10s → Σx=30, Σx²=300, n=3
    load_aovone_group(&mut state, 1, 30, 300, 3);
    let err = dispatch(&mut state, Op::SigmaAovone).unwrap_err();
    // LINT-EXEMPT: error-type comparison
    assert_eq!(err, HpError::DivideByZero);
}

/// 4-group ΣAOVONE reaches STAT1_AOV_KMAX.
///
/// Tests the maximum group-count path (k = 4) with well-formed data to
/// exercise the group-iteration loop at its boundary.
/// Groups: [1], [2], [3], [4] (each n=1) would fail df_within = 4-4 = 0.
/// Use n=2 each: groups with (Σx=3,Σx²=5,n=2), (Σx=7,Σx²=25,n=2),
/// (Σx=11,Σx²=61,n=2), (Σx=15,Σx²=113,n=2).
/// Group means: 1.5, 3.5, 5.5, 7.5; grand N=8, grand mean=4.5.
/// SSW per group: 5-9/2=0.5, 25-49/2=0.5, 61-121/2=0.5, 113-225/2=0.5 → SSW=2
/// SSB = 2·(1.5-4.5)²+2·(3.5-4.5)²+2·(5.5-4.5)²+2·(7.5-4.5)² = 2·(9+1+1+9) = 40
/// df_between=3, df_within=4, F=(40/3)/(2/4)=80/6·4/2 ... = (40/3)/(0.5) = 80/3
#[test]
fn aovone_four_groups_kmax() {
    let mut state = CalcState::new();
    state.regs[STAT1_AOV_K_REG] = HpNum::from(4i32);
    load_aovone_group(&mut state, 0, 3, 5, 2);
    load_aovone_group(&mut state, 1, 7, 25, 2);
    load_aovone_group(&mut state, 2, 11, 61, 2);
    load_aovone_group(&mut state, 3, 15, 113, 2);
    dispatch(&mut state, Op::SigmaAovone).unwrap();
    // F = (SSB/df_between)/(SSW/df_within) = (40/3)/(2/4) = (40/3)/(0.5) = 80/3
    assert_relative_eq!(to_f64(&state.stack.x), 80.0 / 3.0, max_relative = 1e-9);
}

// ── ΣAOVTWO branch coverage ───────────────────────────────────────────────────

/// ΣAOVTWO dimension-overflow guard: r + c exceeds the register footprint.
///
/// ΣAOVTWO checks `STAT1_AOVTWO_ROW_BASE_REG + r + c > STAT1_MAX_REG + 1`
/// before accessing row/col marginal sums. r=4, c=4 → 5 + 4 + 4 = 13 ≤ 45
/// so the 4×4 case does NOT overflow. We need a contrived case: set SIZE down.
///
/// The guard fires when r + c + ROW_BASE_REG > STAT1_MAX_REG + 1 = 45.
/// With r=4, c=4: 4+4+5 = 13 ≤ 45 — no overflow.
/// We can only trigger the guard by truncating the register array.
///
/// Alternative: directly test via the truncated-register approach used by
/// other SIZE-guard tests. Truncate to exactly STAT1_AOVTWO_ROW_BASE_REG + r + c - 1
/// registers so the bounds check triggers. The check reads
/// `STAT1_AOVTWO_ROW_BASE_REG + r + c > STAT1_MAX_REG + 1`, where
/// STAT1_MAX_REG = 44 (so +1 = 45). For r=4, c=4: need state.regs to have <
/// 5+4+4=13 entries to fail the SIZE-floor (but the SIZE-floor is STAT1_MAX_REG+1=45,
/// not the per-slice bound). Closer examination:
///
/// The bound check `STAT1_AOVTWO_ROW_BASE_REG + r + c > STAT1_MAX_REG + 1`
/// compares constants only (based on r, c values). For r=3, c=4: 5+7=12≤45, passes.
/// The guard would fire only if r=20, c=20 (5+40=45, still equal). The only way
/// to fire it is with a contrived state where STAT1_MAX_REG itself is reduced —
/// but STAT1_MAX_REG is a constant. The check appears to be defensive for future
/// larger DIM_MAX changes but cannot fire under current DIM_MAX=4 constraints.
///
/// We test the SIZE-floor guard instead (which CAN fire and which the 3x4
/// oracle test leaves uncovered from the external test perspective).
#[test]
fn aovtwo_size_floor_guard_external() {
    let mut state = CalcState::new();
    state.regs.truncate(STAT1_MAX_REG); // one below the floor
    let err = dispatch(&mut state, Op::SigmaAovtwo).unwrap_err();
    // LINT-EXEMPT: error-type comparison
    assert_eq!(err, HpError::InvalidOp);
}

/// ΣAOVTWO r out-of-range: r = 1 (below minimum) returns Domain.
///
/// `decode_group_count` rejects k < 2, so r = 1 returns Domain immediately.
/// This covers the failure arm of `decode_group_count` for the r dimension.
#[test]
fn aovtwo_r_out_of_range_domain() {
    let mut state = CalcState::new();
    state.regs[STAT1_AOVTWO_R_REG] = HpNum::from(1i32); // r < 2
    state.regs[STAT1_AOVTWO_C_REG] = HpNum::from(3i32);
    let err = dispatch(&mut state, Op::SigmaAovtwo).unwrap_err();
    // LINT-EXEMPT: error-type comparison
    assert_eq!(err, HpError::Domain);
}

/// ΣAOVTWO c out-of-range: c exceeds DIM_MAX returns Domain.
///
/// `decode_group_count` rejects k > cap (DIM_MAX = 4), so c = 5 returns Domain.
/// This covers the c-decode failure arm and the decode_group_count upper-bound branch.
#[test]
fn aovtwo_c_out_of_range_domain() {
    let mut state = CalcState::new();
    state.regs[STAT1_AOVTWO_R_REG] = HpNum::from(2i32);
    state.regs[STAT1_AOVTWO_C_REG] = HpNum::from(5i32); // c > DIM_MAX = 4
    let err = dispatch(&mut state, Op::SigmaAovtwo).unwrap_err();
    // LINT-EXEMPT: error-type comparison
    assert_eq!(err, HpError::Domain);
}

/// ΣAOVTWO DivideByZero: perfect additive fit → SS_error == 0.
///
/// The canonical additive dataset where every cell = i + j (see module docs
/// in anova.rs) produces SS_error = 0, triggering DivideByZero.
/// 2×2 dataset: cells = (1+0, 1+1, 2+0, 2+1) = (1, 2, 2, 3).
/// N=4, grand Σx=8, grand Σx²=18, grand mean=2.
/// Row sums: R0=[1+2]=3 (mean=1.5), R1=[2+3]=5 (mean=2.5)
/// Col sums: C0=[1+2]=3 (mean=1.5), C1=[2+3]=5 (mean=2.5)
/// SS_total = 18 − 4·4 = 2
/// SS_row = 2·((1.5−2)² + (2.5−2)²) = 2·(0.25+0.25) = 1
/// SS_col = 2·((1.5−2)² + (2.5−2)²) = 2·(0.25+0.25) = 1
/// SS_error = 2 − 1 − 1 = 0 → DivideByZero.
#[test]
fn aovtwo_2x2_additive_yields_divide_by_zero() {
    let mut state = CalcState::new();
    state.regs[STAT1_AOVTWO_R_REG] = HpNum::from(2i32);
    state.regs[STAT1_AOVTWO_C_REG] = HpNum::from(2i32);
    // grand Σx² = 1+4+4+9 = 18; grand Σx = 1+2+2+3 = 8
    state.regs[STAT1_AOVTWO_GRAND_SUMSQ_REG] = HpNum::from(18i32);
    state.regs[STAT1_AOVTWO_GRAND_SUM_REG] = HpNum::from(8i32);
    // row sums: R05=3, R06=5
    state.regs[STAT1_AOVTWO_ROW_BASE_REG] = HpNum::from(3i32);
    state.regs[STAT1_AOVTWO_ROW_BASE_REG + 1] = HpNum::from(5i32);
    // col sums: R07=3, R08=5 (base + r = 5+2 = 7)
    state.regs[STAT1_AOVTWO_ROW_BASE_REG + 2] = HpNum::from(3i32);
    state.regs[STAT1_AOVTWO_ROW_BASE_REG + 3] = HpNum::from(5i32);
    let err = dispatch(&mut state, Op::SigmaAovtwo).unwrap_err();
    // LINT-EXEMPT: error-type comparison
    assert_eq!(err, HpError::DivideByZero);
}

// ── ΣANOCOV branch coverage ───────────────────────────────────────────────────

/// ΣANOCOV SIZE-floor guard (external dispatch path).
///
/// Complements the inline unit test `anocov_size_floor_guard` by exercising
/// the guard via the external `dispatch` path.
#[test]
fn anocov_size_floor_guard_external() {
    let mut state = CalcState::new();
    state.regs.truncate(STAT1_MAX_REG);
    let err = dispatch(&mut state, Op::SigmaAnocov).unwrap_err();
    // LINT-EXEMPT: error-type comparison
    assert_eq!(err, HpError::InvalidOp);
}

/// ΣANOCOV k out of range returns Domain.
///
/// `decode_group_count` rejects k < 2 for ΣANOCOV. This covers the
/// Domain error arm of `decode_group_count` for ΣANOCOV's k dimension.
#[test]
fn anocov_k_out_of_range_domain() {
    let mut state = CalcState::new();
    state.regs[STAT1_AOV_K_REG] = HpNum::from(1i32); // k < 2
    let err = dispatch(&mut state, Op::SigmaAnocov).unwrap_err();
    // LINT-EXEMPT: error-type comparison
    assert_eq!(err, HpError::Domain);
}

/// ΣANOCOV N−k−1 ≤ 0: df_within_adj exhausted (insufficient df).
///
/// With k=2 groups and each group having n=1, df_within_adj = N−k−1 = 2−2−1 = −1.
/// ΣANOCOV returns InvalidOp when df_within_adj ≤ 0.
/// This covers the `df_within_adj.inner() <= ZERO` branch at anova.rs.
#[test]
fn anocov_df_within_adj_exhausted_returns_invalid_op() {
    let mut state = CalcState::new();
    state.regs[STAT1_AOV_K_REG] = HpNum::from(2i32);
    // grand Σx² for covariate
    state.regs[STAT1_ANOCOV_GRAND_SUMSQ_X_REG] = HpNum::from(5i32);
    // Group 0: Σy=2, Σy²=4, n=1, Σx=1, Σxy=2
    load_anocov_group(&mut state, 0, 2, 4, 1, 1, 2);
    // Group 1: Σy=4, Σy²=16, n=1, Σx=2, Σxy=8
    load_anocov_group(&mut state, 1, 4, 16, 1, 2, 8);
    // N=2, k=2, df_within_adj = 2-2-1 = -1 ≤ 0 → InvalidOp
    let err = dispatch(&mut state, Op::SigmaAnocov).unwrap_err();
    // LINT-EXEMPT: error-type comparison
    assert_eq!(err, HpError::InvalidOp);
}

/// ΣANOCOV SSWx == 0 triggers DivideByZero.
///
/// SSWx = grand_Σx² − Σᵢ(Σxᵢ)²/nᵢ. If all groups share the same x-mean
/// (i.e., the covariate has zero within-group variability), SSWx = 0.
///
/// Construct k=2 groups where each group's Σx = grand_Σx²'s contribution
/// leaves SSWx = 0:
/// Group 0: n=2, Σx=4 → (Σx)²/n = 16/2 = 8
/// Group 1: n=2, Σx=4 → (Σx)²/n = 16/2 = 8
/// grand_Σx² = 8+8 = 16; SSWx = 16 − (8+8) = 0 → DivideByZero.
/// Need enough df: N=4, k=2, df_within_adj = 4-2-1 = 1 > 0.
#[test]
fn anocov_sswx_zero_returns_divide_by_zero() {
    let mut state = CalcState::new();
    state.regs[STAT1_AOV_K_REG] = HpNum::from(2i32);
    // grand Σx² = 16 (computed to make SSWx = 0)
    state.regs[STAT1_ANOCOV_GRAND_SUMSQ_X_REG] = HpNum::from(16i32);
    // Group 0: Σy=10, Σy²=52, n=2, Σx=4, Σxy=20
    load_anocov_group(&mut state, 0, 10, 52, 2, 4, 20);
    // Group 1: Σy=14, Σy²=100, n=2, Σx=4, Σxy=28
    load_anocov_group(&mut state, 1, 14, 100, 2, 4, 28);
    // SSWx = 16 − (16/2 + 16/2) = 16 − 16 = 0 → DivideByZero
    let err = dispatch(&mut state, Op::SigmaAnocov).unwrap_err();
    // LINT-EXEMPT: error-type comparison
    assert_eq!(err, HpError::DivideByZero);
}

/// ΣANOCOV SST_x == 0 triggers DivideByZero (zero total covariate variance).
///
/// SST_x = grand_Σx² − (grand_Σx)² / N. If all observations share the same
/// x value, SST_x = 0.
/// k=2 groups: Group 0 (n=2, x=[2,2]), Group 1 (n=2, x=[2,2]).
/// grand_Σx = 8, grand_Σx² = 16. SST_x = 16 - 64/4 = 16-16 = 0.
///
/// But to reach the SST_x check, SSWx must be non-zero. Let grand_Σx² > Σ(Σxᵢ²/nᵢ):
/// If per-group x values are same within groups but across-group different:
/// Group 0: n=2, Σx=4 (all x=2) → Σ(Σxᵢ²/nᵢ) term = 16/2 = 8
/// Group 1: n=2, Σx=4 (all x=2) → 16/2 = 8; grand_Σx²=16+0 terms...
/// SSWx = 0 in this case — still divides by zero first.
///
/// Use a different setup: make per-group x varied (SSWx > 0) but all groups
/// at the same grand-mean (SST_x = 0). That requires grand_Σx² = N·x̄².
/// k=2, n=3 each, all 6 x values equal 2: grand_Σx=12, grand_Σx²=24.
/// SST_x = 24 - 144/6 = 24-24 = 0. But then SSWx = 24 - (4/1+4/1+... for k=6)...
/// More directly: 2 groups, n=3 each.
/// Group 0: x=[1,2,3] → Σx=6, Σx²=14; SSWx_0 = (Σx²-Σx²/n) = 14-36/3=14-12=2
/// Group 1: x=[1,2,3] → Σx=6, Σx²=14; SSWx_1 = 2
/// grand_Σx² = 28; grand_Σx = 12; SST_x = 28-144/6 = 28-24 = 4; SSWx = 2+2 = 4.
/// That gives SST_x = 4 ≠ 0; we need grand_Σx²=grand_mean².
///
/// Construct a case where Σxᵢ for each group varies but grand_Σx²/N cancels:
/// Group 0: n=2, Σx=2 (x=[2,0] → Σx²=4); Σx²/n=4/2=2; SSWx_0=4-2=2
/// Group 1: n=2, Σx=2 (x=[2,0] → Σx²=4); Σx²/n=2; SSWx_1=2
/// grand_Σx=4, grand_Σx²=8; SST_x=8-16/4=8-4=4; SSWx=4 > 0. Still non-zero.
///
/// Approach: feed grand_Σx² = (grand_Σx)²/N exactly.
/// k=2, n=2 each, N=4. Need grand_Σx²=grand_Σx²/N.
/// Let grand_Σx = 0: all x sum to 0. Group 0: Σx=1; Group 1: Σx=-1.
/// grand_Σx²: feed it explicitly; SSWx = grand_Σx² - Σ(Σxᵢ²/nᵢ).
/// Group 0: Σx=1, n=2 → Σxᵢ²/n=1/2=0.5; Group 1: Σx=-1, n=2 → 1/2=0.5.
/// Σ(Σxᵢ²/nᵢ) = 1.0. SSWx = grand_Σx² - 1.0.
/// grand_Σx=0; SST_x = grand_Σx² - 0 = grand_Σx². Set grand_Σx²=1 → SST_x=1>0.
///
/// The SST_x == 0 case is only reachable when grand_Σx is exactly proportional
/// to N. This requires HpNum arithmetic to produce exact zero.
/// Set grand_Σx = 4, N = 2 groups × 2 each = 4, grand_Σx² = 4 (= 4²/4):
/// k=2, each n=2. Group 0: Σx=2, Σx²=2; Group 1: Σx=2, Σx²=2.
/// Σ(Σxᵢ²/nᵢ) = 2/2 + 2/2 = 2. SSWx = 4 - 2 = 2 (non-zero, OK).
/// grand_Σx = 4; grand_Σx² = 4; SST_x = 4 - 16/4 = 4-4 = 0 → DivideByZero.
#[test]
fn anocov_sstx_zero_returns_divide_by_zero() {
    let mut state = CalcState::new();
    state.regs[STAT1_AOV_K_REG] = HpNum::from(2i32);
    // grand_Σx² = 4 (feeds SSWx and SST_x)
    state.regs[STAT1_ANOCOV_GRAND_SUMSQ_X_REG] = HpNum::from(4i32);
    // Group 0: Σy=10, Σy²=52, n=2, Σx=2, Σxy=20
    // SSWx_0 = grand_Σx²contribution: Σx²/n = 4/2 = 2; SSWx = 4 - (2+2) = 0 ...
    // Need to avoid SSWx=0: use Σx=2, n=2 → (Σx)²/n = 4/2 = 2; sum over groups = 4 = grand_Σx²
    // SSWx = 4 - 4 = 0 → that would fire DivideByZero on SSWx first.
    // Adjust: use grand_Σx² = 6 with n=3 each:
    // Group 0: n=3, Σx=3 → (Σx)²/n=9/3=3; Group 1: n=3, Σx=3 → 3.
    // SSWx = 6 - (3+3) = 0. Still zero.
    // The only way SSWx ≠ 0 when SST_x = 0 requires per-group x to have variance
    // but sum to zero across the grand: contrived but achievable.
    // Group 0: n=2, Σx=0, Σx²=2 → (Σx)²/n=0; SSWx_0=2
    // Group 1: n=2, Σx=0, Σx²=2 → (Σx)²/n=0; SSWx_1=2
    // grand_Σx=0, grand_Σx²=4; SST_x=4-0/4=4 ≠ 0. Doh.
    //
    // Actually SST_x=0 requires grand_Σx²=grand_Σx²/N, which means all x equal.
    // If all x equal (say x=c), each group has Σx=n·c, Σx²=n·c²; within-group variance=0.
    // So SSWx=0 fires first, and SST_x=0 is unreachable directly.
    //
    // We can force it by providing grand_Σx² < Σ(Σxᵢ²/nᵢ) explicitly to make
    // the SSWx check pass (it only checks is_zero, not negative), then setting
    // grand parameters so SST_x = 0. But SSWx = grand_Σx² - ssbx_terms:
    // if grand_Σx² < ssbx_terms, SSWx < 0, still not zero (is_zero passes).
    // To make SST_x = 0: grand_Σx² = (grand_Σx)²/grand_n.
    //
    // Concrete construction: grand_Σx²=100, ssbx_terms=99 (SSWx=1>0),
    // grand_Σx=20, grand_n=4; SST_x = 100 - 400/4 = 100-100 = 0.
    // k=2, Group 0: n=2, Σx=10, Σx²=50; Group 1: n=2, Σx=10, Σx²=49.
    // ssbx_terms = 100/2 + 100/2 = 100. SSWx=100-100=0 → fires first.
    // Group 0: n=2, Σx=10, (Σx)²/n=50; Group 1: n=2, Σx=10, (Σx)²/n=50.
    // Need asymmetric groups to get ssbx_terms ≠ grand_Σx²:
    // Group 0: n=1, Σx=10, (Σx)²/n=100; Group 1: n=3, Σx=10, (Σx)²/n=100/3≈33.3.
    // ssbx_terms = 100+100/3 = 133.3; SSWx = 100-133.3 < 0 (not zero, but
    // is_zero=false → proceeds). grand_Σx=20, grand_n=4;
    // SST_x = 100 - 400/4 = 0. → DivideByZero on SST_x!
    state.regs[STAT1_ANOCOV_GRAND_SUMSQ_X_REG] = HpNum::from(100i32);
    // Group 0: Σy=5, Σy²=25, n=1, Σx=10, Σxy=50
    load_anocov_group(&mut state, 0, 5, 25, 1, 10, 50);
    // Group 1: Σy=15, Σy²=83, n=3, Σx=10, Σxy=75
    load_anocov_group(&mut state, 1, 15, 83, 3, 10, 75);
    // SSWx = 100 - (100/1 + 100/3) = 100 - 133.33... < 0, not zero → proceeds
    // grand_Σx = 20, grand_n = 4; SST_x = 100 - 400/4 = 0 → DivideByZero
    let err = dispatch(&mut state, Op::SigmaAnocov).unwrap_err();
    // LINT-EXEMPT: error-type comparison
    assert_eq!(err, HpError::DivideByZero);
}

/// ΣANOCOV empty group returns InvalidOp.
///
/// The per-group loop checks `n_i.is_zero()` and returns `InvalidOp` immediately.
/// This covers the guard branch in the per-group iteration loop in ΣANOCOV.
#[test]
fn anocov_empty_group_returns_invalid_op() {
    let mut state = CalcState::new();
    state.regs[STAT1_AOV_K_REG] = HpNum::from(2i32);
    state.regs[STAT1_ANOCOV_GRAND_SUMSQ_X_REG] = HpNum::from(10i32);
    // Group 0: valid
    load_anocov_group(&mut state, 0, 6, 14, 3, 6, 14);
    // Group 1: n = 0 (empty — triggers n_i.is_zero() guard)
    load_anocov_group(&mut state, 1, 0, 0, 0, 0, 0);
    let err = dispatch(&mut state, Op::SigmaAnocov).unwrap_err();
    // LINT-EXEMPT: error-type comparison
    assert_eq!(err, HpError::InvalidOp);
}

/// ΣANOCOV ms_within_adj == 0 triggers DivideByZero.
///
/// ms_within_adj = alt_residual / df_within_adj. If alt_residual = 0,
/// ms_within_adj = 0. alt_residual = SSW_y − SSW_xy²/SSW_x.
/// For alt_residual = 0: SSW_y = SSW_xy²/SSW_x, i.e., perfect within-group
/// linear relationship (r_within = 1).
///
/// k=2, n=3 each:
/// Group 0: x=[1,2,3], y=[2,4,6] (perfect y=2x within group)
///   Σx=6, Σy=12, Σx²=14, Σy²=56, Σxy=28, n=3
///   SSW_y_0 = 56-144/3=56-48=8; SSW_xy_0=28-72/3=28-24=4; SSW_x_0=14-36/3=14-12=2
/// Group 1: x=[4,5,6], y=[8,10,12] (perfect y=2x)
///   Σx=15, Σy=30, Σx²=77, Σy²=308, Σxy=154, n=3
///   SSW_y_1=308-900/3=308-300=8; SSW_xy_1=154-450/3=154-150=4; SSW_x_1=77-225/3=77-75=2
/// SSW_y=16, SSW_xy=8, SSW_x=4
/// alt_residual = 16 - 64/4 = 16-16 = 0 → ms_within_adj division by zero.
/// grand_Σx²=14+77=91; grand_Σx=21; grand_Σy=42; N=6; k=2; df_within_adj=6-2-1=3
/// (but ms_within_adj fires before df check... no, df check is first).
/// Actually df_within_adj=3>0, so we reach ms_within_adj.
#[test]
fn anocov_ms_within_adj_zero_returns_divide_by_zero() {
    let mut state = CalcState::new();
    state.regs[STAT1_AOV_K_REG] = HpNum::from(2i32);
    // grand Σx² = 14 + 77 = 91
    state.regs[STAT1_ANOCOV_GRAND_SUMSQ_X_REG] = HpNum::from(91i32);
    // Group 0: x=[1,2,3] y=2x; Σy=12, Σy²=56, n=3, Σx=6, Σxy=28
    load_anocov_group(&mut state, 0, 12, 56, 3, 6, 28);
    // Group 1: x=[4,5,6] y=2x; Σy=30, Σy²=308, n=3, Σx=15, Σxy=154
    load_anocov_group(&mut state, 1, 30, 308, 3, 15, 154);
    // alt_residual = SSW_y - SSW_xy²/SSW_x = 16 - 64/4 = 0 → DivideByZero
    let err = dispatch(&mut state, Op::SigmaAnocov).unwrap_err();
    // LINT-EXEMPT: error-type comparison
    assert_eq!(err, HpError::DivideByZero);
}
