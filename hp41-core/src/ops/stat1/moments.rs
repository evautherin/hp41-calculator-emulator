// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `stat1::moments` — third/fourth moments, skewness, kurtosis (Plan 33-06).
//!
//! Ships ΣMMTUG (ungrouped) and ΣMMTGD (grouped/frequency-weighted) per
//! OM 00041-90030 §"Moments, Skewness, and Kurtosis" (p. 15). Both are
//! per-point accumulators consuming `x` from X (and frequency `f` from
//! Y for ΣMMTGD). Result extraction via the public helper
//! [`compute_moments`] which derives central moments μ₃, μ₄, skewness
//! γ₁, excess kurtosis γ₂ from the accumulated raw-moment Σ-block.
//!
//! Register layout (SIZE 012, R00..R11 per OM p. 15):
//! R01=Σx², R02=Σx, R03=n (delegated to v1.x op_sigma_plus for ΣMMTUG);
//! R07=Σx³ ([`STAT1_MMTUG_CUBE_REG`]), R08=Σx⁴
//! ([`STAT1_MMTUG_QUAD_REG`]).
//!
//! Anti-duplication: ΣMMTUG delegates Σx²/Σx/n updates to
//! [`crate::ops::stats::op_sigma_plus`] per PATTERNS.md Pattern 4;
//! ΣMMTGD writes all five slots atomically (no frequency-weighted
//! delegate exists). Atomic-write discipline per Pitfall 5.
//!
//! STAT-UNI-04 [C] correction-key: `op_sigma_minus` was extended in
//! this plan to mirror every register written by ΣMMTUG, gated by the
//! `state.regs.len() > STAT1_MAX_REG` check so v1.x users are unaffected.
//!
//! References: OM 00041-90030 (p. 15); `scipy.stats.moment`,
//! `scipy.stats.skew(bias=True)`, `scipy.stats.kurtosis(fisher=True,
//! bias=True)` (oracles).

use crate::error::HpError;
use crate::num::HpNum;
use crate::ops::stat1::{STAT1_MAX_REG, STAT1_MMTUG_CUBE_REG, STAT1_MMTUG_QUAD_REG};
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;

// ── SIZE-floor guard helper ────────────────────────────────────────────────

/// Fail-closed SIZE-floor guard. Returns `Err(HpError::InvalidOp)` if
/// `state.regs` cannot address every slot up to and including
/// `STAT1_MAX_REG`. Mirrors the [`crate::ops::stats::op_sigma_plus`]
/// SIZE 7 floor extended to the Stat-1 single-source-of-truth max.
#[inline]
fn require_stat1_size_floor(state: &CalcState) -> Result<(), HpError> {
    if state.regs.len() < STAT1_MAX_REG + 1 {
        return Err(HpError::InvalidOp);
    }
    Ok(())
}

// ── ΣMMTUG — Ungrouped Third + Fourth Moment Accumulator ───────────────────

/// ΣMMTUG — accumulate `x` (from stack X) into the Σ-block extended
/// with Σx³ (R07) and Σx⁴ (R08). Σx²/Σx/n updates delegated to
/// `op_sigma_plus`; cube/quad slots updated atomically before delegate.
///
/// (Stack Y is unused by the compute step — set Y=0 to avoid polluting
/// the v1.x Σy/Σxy slots written by the delegate, or use ΣMMTGD.)
///
/// Errors: `InvalidOp` on SIZE-floor; `Overflow` on extreme magnitudes.
///
/// Source: OM 00041-90030 §ΣMMTUG (p. 15). Raw moments only; central
/// moments via [`compute_moments`].
pub fn op_sigma_mmtug(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;

    let x = state.stack.x.clone();
    // Compute Σx³ and Σx⁴ atomically before write (atomic-write discipline
    // matches op_sigma_plus per Pitfall 5).
    let x_sq = x.checked_sq()?;
    let x_cube = x.checked_mul(&x_sq)?;
    let x_quad = x_sq.checked_sq()?;
    let new_cube = state.regs[STAT1_MMTUG_CUBE_REG].checked_add(&x_cube)?;
    let new_quad = state.regs[STAT1_MMTUG_QUAD_REG].checked_add(&x_quad)?;

    // Atomic write of the extended slots BEFORE delegating to op_sigma_plus.
    // (Order is independent — both writes succeed once both values were
    // computed above without overflow.)
    state.regs[STAT1_MMTUG_CUBE_REG] = new_cube;
    state.regs[STAT1_MMTUG_QUAD_REG] = new_quad;

    // Delegate Σx², Σx, n updates to v1.x op_sigma_plus.
    crate::ops::stats::op_sigma_plus(state)
}

// ── ΣMMTGD — Grouped (Frequency-Weighted) Variant ──────────────────────────

/// ΣMMTGD — grouped/frequency-weighted accumulator. Each call consumes
/// `x` from X and frequency `f` from Y, contributing `f·xᵏ` to Σxᵏ for
/// k=1..4 and updating N by adding f. Writes all five slots
/// (R01=Σ(f·x²), R02=Σ(f·x), R03=N=Σf, R07=Σ(f·x³), R08=Σ(f·x⁴))
/// atomically — no `op_sigma_plus` delegate (the delegate has no
/// frequency-weighting API).
///
/// Errors: `InvalidOp` on SIZE-floor; `Overflow` on extreme magnitudes.
///
/// Source: OM 00041-90030 §ΣMMTGD (p. 15).
pub fn op_sigma_mmtgd(state: &mut CalcState) -> Result<(), HpError> {
    require_stat1_size_floor(state)?;

    let x = state.stack.x.clone();
    let f = state.stack.y.clone();

    // Compute all five contributions atomically before any write.
    let fx = f.checked_mul(&x)?;
    let fx_sq = fx.checked_mul(&x)?;
    let fx_cube = fx_sq.checked_mul(&x)?;
    let fx_quad = fx_cube.checked_mul(&x)?;

    // v1.x R01–R06 exemption (REVIEW.md WR-02): R01=Σx², R02=Σx, R03=n
    // (or Σf for ΣMMTGD frequency-weighting) per the canonical v1.x
    // Σ-block layout in `ops/stats.rs`. Stat-1-specific slots
    // (STAT1_MMTUG_CUBE_REG=R07, STAT1_MMTUG_QUAD_REG=R08) route
    // through named consts per P21.
    let new_r1 = state.regs[1].checked_add(&fx_sq)?; // Σ(f·x²)
    let new_r2 = state.regs[2].checked_add(&fx)?; // Σ(f·x)
    let new_r3 = state.regs[3].checked_add(&f)?; // N = Σf
    let new_cube = state.regs[STAT1_MMTUG_CUBE_REG].checked_add(&fx_cube)?;
    let new_quad = state.regs[STAT1_MMTUG_QUAD_REG].checked_add(&fx_quad)?;

    // Atomic write of all five slots.
    state.regs[1] = new_r1;
    state.regs[2] = new_r2;
    state.regs[3] = new_r3.clone();
    state.regs[STAT1_MMTUG_CUBE_REG] = new_cube;
    state.regs[STAT1_MMTUG_QUAD_REG] = new_quad;

    // Push count Σf into X (matches op_sigma_plus convention).
    state.stack.lift_enabled = true;
    enter_number(state, new_r3);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

// ── Moments Compute Step — μ₃, μ₄, γ₁, γ₂ ──────────────────────────────────

/// Compute central moments μ₃, μ₄ and standardized skewness γ₁ + excess
/// kurtosis γ₂ from the raw Σ-block (n=R03, Σx=R02, Σx²=R01, Σx³=R07,
/// Σx⁴=R08). Returns `(μ₃, μ₄, γ₁, γ₂)` where γ₂ = μ₄/μ₂² − 3 is the
/// FISHER (excess) kurtosis per scipy.stats.kurtosis(fisher=True).
/// Formulas: μ₂ = Σx²/n − μ²; μ₃ = Σx³/n − 3μ·μ₂_raw + 2μ³;
/// μ₄ = Σx⁴/n − 4μ·Σx³/n + 6μ²·Σx²/n − 3μ⁴; γ₁ = μ₃/μ₂^(3/2);
/// γ₂ = μ₄/μ₂² − 3.
///
/// Errors: `InvalidOp` if n < 2; `Domain` if μ₂ ≤ 0 (degenerate variance).
pub fn compute_moments(state: &CalcState) -> Result<(HpNum, HpNum, HpNum, HpNum), HpError> {
    // v1.x R01–R06 exemption (REVIEW.md WR-02): R01=Σx², R02=Σx, R03=n
    // per the canonical v1.x Σ-block layout. ΣMMTUG/ΣMMTGD extend
    // the block with Σx³ at R07 and Σx⁴ at R08 — those slots route
    // through named consts per the P21 named-const policy.
    let n = state.regs[3].clone();
    let sum_x = state.regs[2].clone();
    let sum_x_sq = state.regs[1].clone();
    let sum_x_cube = state.regs[STAT1_MMTUG_CUBE_REG].clone();
    let sum_x_quad = state.regs[STAT1_MMTUG_QUAD_REG].clone();

    let two = HpNum::from(2i32);
    if n.checked_sub(&two)?.inner() < rust_decimal::Decimal::ZERO {
        return Err(HpError::InvalidOp);
    }

    let mu = sum_x.checked_div(&n)?; // μ = Σx / n
    let mu_sq = mu.checked_sq()?; // μ²
    let mu_cube = mu.checked_mul(&mu_sq)?; // μ³
    let mu_quad = mu_sq.checked_sq()?; // μ⁴

    // REVIEW.md WR-05 removed: a redundant `let m1 = sum_x.checked_div(&n)?;
    // let _ = m1;` pair that duplicated the `mu` computation on the
    // line above. The discarded division was likely a refactoring
    // leftover from when `m1` was the raw first moment name; the
    // second division on the same inputs is identical to `mu` and
    // the discarded value carried no side effect worth preserving.
    let m2 = sum_x_sq.checked_div(&n)?; // Σx²/n
    let m3 = sum_x_cube.checked_div(&n)?; // Σx³/n
    let m4 = sum_x_quad.checked_div(&n)?; // Σx⁴/n

    // μ₂ = Σx²/n − μ²    (population variance — biased estimator,
    //                     matches scipy.stats.moment with default bias=True)
    let mu_2 = m2.checked_sub(&mu_sq)?;

    // μ₃ = Σx³/n − 3·μ·(Σx²/n) + 2·μ³
    let three = HpNum::from(3i32);
    let term_3a = three.checked_mul(&mu)?.checked_mul(&m2)?;
    let term_3b = two.checked_mul(&mu_cube)?;
    let mu_3 = m3.checked_sub(&term_3a)?.checked_add(&term_3b)?;

    // μ₄ = Σx⁴/n − 4·μ·(Σx³/n) + 6·μ²·(Σx²/n) − 3·μ⁴
    let four = HpNum::from(4i32);
    let six = HpNum::from(6i32);
    let term_4a = four.checked_mul(&mu)?.checked_mul(&m3)?;
    let term_4b = six.checked_mul(&mu_sq)?.checked_mul(&m2)?;
    let term_4c = three.checked_mul(&mu_quad)?;
    let mu_4 = m4
        .checked_sub(&term_4a)?
        .checked_add(&term_4b)?
        .checked_sub(&term_4c)?;

    // γ₁ = μ₃ / μ₂^(3/2) — requires μ₂ > 0.
    if mu_2.inner() <= rust_decimal::Decimal::ZERO {
        return Err(HpError::Domain);
    }
    let mu_2_sqrt = mu_2.checked_sqrt()?;
    let mu_2_pow_3_2 = mu_2.checked_mul(&mu_2_sqrt)?;
    let gamma_1 = mu_3.checked_div(&mu_2_pow_3_2)?;

    // γ₂ = μ₄ / μ₂² − 3  (excess kurtosis; scipy fisher=True)
    let mu_2_sq = mu_2.checked_sq()?;
    let gamma_2 = mu_4.checked_div(&mu_2_sq)?.checked_sub(&three)?;

    Ok((mu_3, mu_4, gamma_1, gamma_2))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use rust_decimal::prelude::ToPrimitive;

    fn h(f: f64) -> HpNum {
        HpNum::from(rust_decimal::Decimal::from_f64_retain(f).expect("finite"))
    }

    fn to_f64(v: &HpNum) -> f64 {
        v.inner().to_f64().expect("finite")
    }

    /// Accumulate the dataset [1..10] via op_sigma_mmtug; assert the
    /// raw Σ-block values land in the expected slots.
    ///
    /// Manual reference for [1..10]:
    ///   Σx = 55, Σx² = 385, Σx³ = 3025, Σx⁴ = 25333, n = 10.
    /// `numpy.sum(arr**k for k=1..4)` confirms.
    #[test]
    fn mmtug_accumulates_raw_moments() {
        let mut state = CalcState::new();
        state.stack.y = HpNum::zero();
        for v in 1..=10 {
            state.stack.x = HpNum::from(v);
            op_sigma_mmtug(&mut state).unwrap();
        }
        assert_relative_eq!(to_f64(&state.regs[1]), 385.0, max_relative = 1e-9);
        assert_relative_eq!(to_f64(&state.regs[2]), 55.0, max_relative = 1e-9);
        assert_relative_eq!(to_f64(&state.regs[3]), 10.0, max_relative = 1e-9);
        assert_relative_eq!(
            to_f64(&state.regs[STAT1_MMTUG_CUBE_REG]),
            3025.0,
            max_relative = 1e-9
        );
        assert_relative_eq!(
            to_f64(&state.regs[STAT1_MMTUG_QUAD_REG]),
            25333.0,
            max_relative = 1e-9
        );
    }

    /// SPEC.md Req. 8 oracle: dataset [1..10]; assert central moments
    /// and skewness / kurtosis match scipy.stats.
    ///
    /// scipy oracle values (Python 3, biased / population estimator):
    /// ```python
    /// data = list(range(1, 11))
    /// scipy.stats.moment(data, moment=3)            #   0.0
    /// scipy.stats.moment(data, moment=4)            # 120.8625
    /// scipy.stats.skew(data, bias=True)             #   0.0
    /// scipy.stats.kurtosis(data, fisher=True, bias=True)  # -1.2242424242424244
    /// ```
    ///
    /// Manual derivation:
    ///   deviations from μ = 5.5: ±4.5, ±3.5, ±2.5, ±1.5, ±0.5
    ///   Σ(devⁱ⁴) = 2·(4.5⁴ + 3.5⁴ + 2.5⁴ + 1.5⁴ + 0.5⁴)
    ///            = 2·(410.0625 + 150.0625 + 39.0625 + 5.0625 + 0.0625)
    ///            = 2·604.3125 = 1208.625
    ///   μ₄ = 1208.625 / 10 = 120.8625
    ///   μ₂ = (Σ(devⁱ²)) / 10 = 82.5 / 10 = 8.25
    ///   γ₂ = μ₄ / μ₂² − 3 = 120.8625 / 68.0625 − 3 = 1.7757... − 3 = −1.2242...
    ///
    /// **SPEC.md drift:** SPEC.md Req. 8 claims μ₄ = 33.0 — that value
    /// reverse-engineers to `(Σx⁴ − ...) / something else`, not the
    /// standard fourth central moment. scipy and manual derivation both
    /// confirm μ₄ = 120.8625. γ₂ matches SPEC's −1.224. SPEC.md amendment
    /// gated to Phase 35 (STAT-DOC) per the same convention as Plans
    /// 33-04 + 33-05 oracle drifts.
    #[test]
    fn mmtug_third_fourth_moments_one_to_ten() {
        let mut state = CalcState::new();
        state.stack.y = HpNum::zero();
        for v in 1..=10 {
            state.stack.x = HpNum::from(v);
            op_sigma_mmtug(&mut state).unwrap();
        }
        let (mu_3, mu_4, gamma_1, gamma_2) = compute_moments(&state).unwrap();
        assert_relative_eq!(to_f64(&mu_3), 0.0, epsilon = 1e-9);
        assert_relative_eq!(to_f64(&mu_4), 120.8625, max_relative = 1e-9);
        assert_relative_eq!(to_f64(&gamma_1), 0.0, epsilon = 1e-9);
        assert_relative_eq!(to_f64(&gamma_2), -1.2242424242424244, max_relative = 1e-7);
    }

    /// ΣMMTGD grouped variant: data=[1,2,3] with frequencies f=[2,3,1].
    /// Effective dataset: [1,1,2,2,2,3] → n_effective=6, mean=2.0.
    ///
    /// Manual reference:
    ///   Σf = 6, Σ(f·x) = 2+6+3 = 11, x̄=11/6 ≈ 1.8333...
    ///   Σ(f·x²) = 2+12+9 = 23
    ///   Σ(f·x³) = 2+24+27 = 53
    ///   Σ(f·x⁴) = 2+48+81 = 131
    #[test]
    fn mmtgd_grouped_accumulates_weighted_sums() {
        let mut state = CalcState::new();
        let data = [(1, 2), (2, 3), (3, 1)];
        for (x_i, f_i) in data {
            state.stack.y = HpNum::from(f_i);
            state.stack.x = HpNum::from(x_i);
            op_sigma_mmtgd(&mut state).unwrap();
        }
        assert_relative_eq!(to_f64(&state.regs[1]), 23.0, max_relative = 1e-9);
        assert_relative_eq!(to_f64(&state.regs[2]), 11.0, max_relative = 1e-9);
        assert_relative_eq!(to_f64(&state.regs[3]), 6.0, max_relative = 1e-9);
        assert_relative_eq!(
            to_f64(&state.regs[STAT1_MMTUG_CUBE_REG]),
            53.0,
            max_relative = 1e-9
        );
        assert_relative_eq!(
            to_f64(&state.regs[STAT1_MMTUG_QUAD_REG]),
            131.0,
            max_relative = 1e-9
        );
    }

    /// STAT-UNI-04 [C] correction-key round-trip: every register written
    /// by ΣMMTUG accumulation must be reversed by op_sigma_minus.
    ///
    /// Construct fresh state; accumulate x=5; verify slot values; call
    /// op_sigma_minus with the same x in stack; assert every Σ-block
    /// register including the extended slots returns to zero within
    /// 1e-12 relative tolerance.
    #[test]
    fn mmtug_accumulate_then_minus_restores_state() {
        let mut state = CalcState::new();
        state.stack.y = HpNum::zero();
        state.stack.x = HpNum::from(5);
        op_sigma_mmtug(&mut state).unwrap();
        // Now we have: R01=25, R02=5, R03=1, R07=125, R08=625.
        assert_eq!(state.regs[1], h(25.0));
        assert_eq!(state.regs[STAT1_MMTUG_CUBE_REG], h(125.0));
        assert_eq!(state.regs[STAT1_MMTUG_QUAD_REG], h(625.0));

        // Reset the stack to the original (x=5, y=0) AND call op_sigma_minus.
        // op_sigma_plus pushed n=1 into X, so we need to reload x for
        // the symmetric reversal.
        state.stack.x = HpNum::from(5);
        state.stack.y = HpNum::zero();
        crate::ops::stats::op_sigma_minus(&mut state).unwrap();

        // Every register that was incremented must return to zero.
        assert_eq!(state.regs[1], HpNum::zero());
        assert_eq!(state.regs[2], HpNum::zero());
        assert_eq!(state.regs[3], HpNum::zero());
        assert_eq!(state.regs[STAT1_MMTUG_CUBE_REG], HpNum::zero());
        assert_eq!(state.regs[STAT1_MMTUG_QUAD_REG], HpNum::zero());
    }

    /// SIZE-floor guard fires when state.regs cannot address STAT1_MAX_REG.
    #[test]
    fn mmtug_size_floor_guard() {
        let mut state = CalcState::new();
        state.regs.truncate(STAT1_MAX_REG); // one slot short of STAT1_MAX_REG + 1
        state.stack.x = HpNum::from(5);
        state.stack.y = HpNum::zero();
        assert_eq!(op_sigma_mmtug(&mut state).unwrap_err(), HpError::InvalidOp);
    }

    /// SIZE-floor guard for ΣMMTGD as well.
    #[test]
    fn mmtgd_size_floor_guard() {
        let mut state = CalcState::new();
        state.regs.truncate(STAT1_MAX_REG);
        state.stack.x = HpNum::from(5);
        state.stack.y = HpNum::from(2);
        assert_eq!(op_sigma_mmtgd(&mut state).unwrap_err(), HpError::InvalidOp);
    }

    /// compute_moments fails on n < 2 (variance undefined).
    #[test]
    fn compute_moments_n_too_small() {
        let mut state = CalcState::new();
        state.regs[3] = HpNum::from(1);
        assert_eq!(compute_moments(&state).unwrap_err(), HpError::InvalidOp);
    }

    /// compute_moments fails when μ₂ ≤ 0 (all samples identical →
    /// degenerate variance).
    #[test]
    fn compute_moments_zero_variance_returns_domain_error() {
        let mut state = CalcState::new();
        // n=5; Σx=10; Σx²=20 → μ=2, μ₂ = 20/5 − 4 = 0
        state.regs[1] = HpNum::from(20);
        state.regs[2] = HpNum::from(10);
        state.regs[3] = HpNum::from(5);
        state.regs[STAT1_MMTUG_CUBE_REG] = HpNum::from(40);
        state.regs[STAT1_MMTUG_QUAD_REG] = HpNum::from(80);
        assert_eq!(compute_moments(&state).unwrap_err(), HpError::Domain);
    }
}
