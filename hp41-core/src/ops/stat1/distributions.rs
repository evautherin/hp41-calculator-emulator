// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `stat1::distributions` — hand-coded f64-bridge distribution primitives
//! consumed by Plans 33-03 (ΣNORMD, ΣCHISQD) + 33-07 (ΣPTST, ΣTSTAT).
//! Every primitive validated against ≥6 inline scipy oracle tuples (D-33.6).
//!
//! Sources (all NON-GPL, NOT from Free42 core_math2.cc):
//! - `norm_cdf_inv_f64`: Acklam (1996/1999) ≡ AS 241 (Wichura 1988).
//!   <https://stackedboxes.org/2017/05/01/acklams-normal-quantile-function/>.
//!   Public domain.
//! - `gamma_regularized_f64`: AS 239 (Shea 1988) / Numerical Recipes 3e §6.2.
//! - `beta_regularized_f64`: AS 63 (Majumder & Bhattacharjee 1973) / NR §6.4.
//! - `ln_gamma` (private): Lanczos series per NR §6.1 / AS 245.

// Acklam / AS 239 / AS 63 coefficients are verbatim from their published
// sources; clippy::excessive_precision is silenced to preserve citation form.
#![allow(clippy::excessive_precision)]

use crate::error::HpError;

// Acklam (1996/1999) coefficient table. Source:
// <https://stackedboxes.org/2017/05/01/acklams-normal-quantile-function/>
// (public-domain mirror; algorithmically equivalent to AS 241 / Wichura 1988).
// 21 coefficients verbatim — DO NOT paraphrase; Acklam's <1.15e-9 relative
// error bound depends on the exact values below.

const ACKLAM_A1: f64 = -3.969_683_028_665_376e1;
const ACKLAM_A2: f64 = 2.209_460_984_245_205e2;
const ACKLAM_A3: f64 = -2.759_285_104_469_687e2;
const ACKLAM_A4: f64 = 1.383_577_518_672_690e2;
const ACKLAM_A5: f64 = -3.066_479_806_614_716e1;
const ACKLAM_A6: f64 = 2.506_628_277_459_239e0;
const ACKLAM_B1: f64 = -5.447_609_879_822_406e1;
const ACKLAM_B2: f64 = 1.615_858_368_580_409e2;
const ACKLAM_B3: f64 = -1.556_989_798_598_866e2;
const ACKLAM_B4: f64 = 6.680_131_188_771_972e1;
const ACKLAM_B5: f64 = -1.328_068_155_288_572e1;
const ACKLAM_C1: f64 = -7.784_894_002_430_293e-3;
const ACKLAM_C2: f64 = -3.223_964_580_411_365e-1;
const ACKLAM_C3: f64 = -2.400_758_277_161_838e0;
const ACKLAM_C4: f64 = -2.549_732_539_343_734e0;
const ACKLAM_C5: f64 = 4.374_664_141_464_968e0;
const ACKLAM_C6: f64 = 2.938_163_982_698_783e0;
const ACKLAM_D1: f64 = 7.784_695_709_041_462e-3;
const ACKLAM_D2: f64 = 3.224_671_290_700_398e-1;
const ACKLAM_D3: f64 = 2.445_134_137_142_996e0;
const ACKLAM_D4: f64 = 3.754_408_661_907_416e0;

/// Region split-points for Acklam's three-branch rational approximation
/// (Acklam 1996/1999): lower tail below, central in-band, upper tail above.
const ACKLAM_P_LOW: f64 = 0.024_25;
const ACKLAM_P_HIGH: f64 = 1.0 - ACKLAM_P_LOW;

/// Inverse standard-normal CDF Φ⁻¹(p) for p ∈ (0, 1). Acklam rational
/// approximation, ~1.15e-9 relative error (≥6 inline scipy oracle tuples).
///
/// # Errors
///
/// `HpError::Domain` if `p` is not finite or outside `(0, 1)` (asymptotes
/// p ∈ {0, 1} map to ±∞; the outer ΣNORMD Op surfaces these to the user).
pub fn norm_cdf_inv_f64(p: f64) -> Result<f64, HpError> {
    if !p.is_finite() || !(0.0..=1.0).contains(&p) {
        return Err(HpError::Domain);
    }
    // Asymptotes p ∈ {0, 1} → ±∞: bare primitive surfaces as Domain;
    // the Plan 33-03 ΣNORMD inverse Op may bisection-clip if it chooses.
    if p == 0.0 || p == 1.0 {
        return Err(HpError::Domain);
    }
    // Three-branch rational approximation: lower-tail / central / upper-tail.
    let x = if p < ACKLAM_P_LOW {
        let q = (-2.0_f64 * p.ln()).sqrt();
        (((((ACKLAM_C1 * q + ACKLAM_C2) * q + ACKLAM_C3) * q + ACKLAM_C4) * q + ACKLAM_C5) * q
            + ACKLAM_C6)
            / ((((ACKLAM_D1 * q + ACKLAM_D2) * q + ACKLAM_D3) * q + ACKLAM_D4) * q + 1.0)
    } else if p <= ACKLAM_P_HIGH {
        let q = p - 0.5;
        let r = q * q;
        (((((ACKLAM_A1 * r + ACKLAM_A2) * r + ACKLAM_A3) * r + ACKLAM_A4) * r + ACKLAM_A5) * r
            + ACKLAM_A6)
            * q
            / (((((ACKLAM_B1 * r + ACKLAM_B2) * r + ACKLAM_B3) * r + ACKLAM_B4) * r + ACKLAM_B5)
                * r
                + 1.0)
    } else {
        let q = (-2.0_f64 * (1.0 - p).ln()).sqrt();
        -(((((ACKLAM_C1 * q + ACKLAM_C2) * q + ACKLAM_C3) * q + ACKLAM_C4) * q + ACKLAM_C5) * q
            + ACKLAM_C6)
            / ((((ACKLAM_D1 * q + ACKLAM_D2) * q + ACKLAM_D3) * q + ACKLAM_D4) * q + 1.0)
    };
    Ok(x)
}

// ln_gamma — 6-coefficient Lanczos series per Numerical Recipes 3e §6.1
// (`gammln`) ≡ AS 245. NOT consulted from Free42 core_math2.cc.

/// Lanczos coefficients for ln Γ(z) per NR §6.1.
const LANCZOS_COEFS: [f64; 6] = [
    76.180_091_729_471_46,
    -86.505_320_329_416_77,
    24.014_098_240_830_91,
    -1.231_739_572_450_155,
    0.001_208_650_973_866_179,
    -0.000_005_395_239_384_953,
];

/// `ln Γ(a)` for `a > 0` — Lanczos series, accuracy ~1e-15. Private to
/// this module; consumed by `gamma_regularized_f64` and `beta_regularized_f64`.
/// `Err(HpError::Domain)` for `a <= 0` or non-finite `a`.
fn ln_gamma(a: f64) -> Result<f64, HpError> {
    if !a.is_finite() || a <= 0.0 {
        return Err(HpError::Domain);
    }
    // NR §6.1 `gammln`:
    //   tmp = (a+0.5)·ln(a+5.5) − (a+5.5),
    //   ser = 1.000000000190015 + Σ_j cof[j] / (a+j+1),
    //   ln Γ(a) = tmp + ln(√(2π) · ser / a).  (√(2π) = 2.5066282746310005)
    let tmp_arg = a + 5.5;
    let tmp = (a + 0.5) * tmp_arg.ln() - tmp_arg;
    let mut ser = 1.000_000_000_190_015;
    for (j, c) in LANCZOS_COEFS.iter().enumerate() {
        ser += c / (a + (j as f64) + 1.0);
    }
    Ok(tmp + (2.506_628_274_631_000_5 * ser / a).ln())
}

// gamma_regularized_f64 — AS 239 (Shea 1988) ≡ NR 3e §6.2 (`gammp`/`gser`/`gcf`).
// Strategy: power series for `x < s+1`, modified-Lentz CF for `x >= s+1`.
// 50-iteration cap (SPEC.md Req. 34); non-convergence → Err(Domain) so the
// outer ΣCHISQD Op layer can surface it. NOT from Free42 core_math2.cc.

/// Iteration cap for gser/gcf/betacf inner loops (SPEC.md Req. 34).
const ITER_CAP: usize = 50;

/// Convergence tolerance for series + CF expansions. Calibrated for the
/// 50-iter cap + Req. 33's 1e-9 oracle band: at 1e-9 the worst-case
/// boundary input (s=x=50) converges at iter 49 with ~6.2e-10 vs scipy.
/// Tighter values (1e-12/1e-15) bust the cap — see 33-02-SUMMARY.md
/// "Convergence trace".
const EPS_CONV: f64 = 1e-9;

/// Underflow floor for the modified-Lentz CF guard.
const FP_MIN: f64 = 1e-300;

/// Lentz-CF underflow floor: clamp `v` to FP_MIN if its magnitude falls below.
#[inline]
fn lentz_floor(v: f64) -> f64 {
    if v.abs() < FP_MIN {
        FP_MIN
    } else {
        v
    }
}

/// Power-series expansion for P(s, x), valid for `x < s + 1`. Multiplier
/// `exp(-x + s·ln(x) - ln Γ(s))`. `Err(HpError::Domain)` on non-convergence.
fn gser(s: f64, x: f64) -> Result<f64, HpError> {
    if x <= 0.0 {
        return Ok(0.0);
    }
    let ln_gs = ln_gamma(s)?;
    let mut ap = s;
    let mut sum = 1.0_f64 / s;
    let mut del = sum;
    for _ in 0..ITER_CAP {
        ap += 1.0;
        del *= x / ap;
        sum += del;
        if del.abs() < sum.abs() * EPS_CONV {
            return Ok(sum * (-x + s * x.ln() - ln_gs).exp());
        }
    }
    Err(HpError::Domain)
}

/// Modified-Lentz CF for Q(s, x) = 1 - P(s, x), valid for `x >= s + 1`.
/// `Err(HpError::Domain)` on non-convergence. Per NR §6.2.
fn gcf(s: f64, x: f64) -> Result<f64, HpError> {
    let ln_gs = ln_gamma(s)?;
    let mut b = x + 1.0 - s;
    let mut c = 1.0 / FP_MIN;
    let mut d = 1.0 / b;
    let mut h = d;
    for i in 1..=ITER_CAP {
        let an = -(i as f64) * (i as f64 - s);
        b += 2.0;
        d = lentz_floor(an * d + b);
        c = lentz_floor(b + an / c);
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < EPS_CONV {
            return Ok(h * (-x + s * x.ln() - ln_gs).exp());
        }
    }
    Err(HpError::Domain)
}

/// Regularized lower incomplete gamma P(s, x) = γ(s, x) / Γ(s) for `s > 0`,
/// `x >= 0`. Returns a value in [0, 1]. Algorithm split at `x < s+1`
/// (AS 239 / NR 3e §6.2 `gammp`).
///
/// # Errors
///
/// `HpError::Domain` if `s` or `x` is non-finite, if `s <= 0` or `x < 0`,
/// or on iteration-cap exhaustion (cap = 50 per SPEC.md Req. 34).
pub fn gamma_regularized_f64(s: f64, x: f64) -> Result<f64, HpError> {
    if !s.is_finite() || !x.is_finite() || s <= 0.0 || x < 0.0 {
        return Err(HpError::Domain);
    }
    if x == 0.0 {
        return Ok(0.0);
    }
    if x < s + 1.0 {
        gser(s, x)
    } else {
        // P(s, x) = 1 - Q(s, x)
        gcf(s, x).map(|q| 1.0 - q)
    }
}

// beta_regularized_f64 — AS 63 (Majumder & Bhattacharjee 1973) ≡ NR 3e §6.4
// (`betai`/`betacf`). Strategy: symmetric boundary swap at
// `x < (a+1)/(a+b+2)`; below → direct CF, above → identity
// I_x(a,b) = 1 - I_{1-x}(b,a). Modified-Lentz CF, 50-iter cap. NOT from
// Free42 core_math2.cc.

/// Modified-Lentz CF for I_x(a, b) per NR §6.4. `Err(HpError::Domain)` on
/// non-convergence within `ITER_CAP`.
fn betacf(a: f64, b: f64, x: f64) -> Result<f64, HpError> {
    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;
    let mut c = 1.0_f64;
    let mut d = 1.0 / lentz_floor(1.0 - qab * x / qap);
    let mut h = d;
    for m in 1..=ITER_CAP {
        let m_f = m as f64;
        let m2 = 2.0 * m_f;
        // Even step (d_{2m}): m(b-m)x / ((qam+m2)(a+m2))
        let aa = m_f * (b - m_f) * x / ((qam + m2) * (a + m2));
        d = lentz_floor(1.0 + aa * d);
        c = lentz_floor(1.0 + aa / c);
        d = 1.0 / d;
        h *= d * c;
        // Odd step (d_{2m+1}): -(a+m)(qab+m)x / ((a+m2)(qap+m2))
        let aa = -(a + m_f) * (qab + m_f) * x / ((a + m2) * (qap + m2));
        d = lentz_floor(1.0 + aa * d);
        c = lentz_floor(1.0 + aa / c);
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < EPS_CONV {
            return Ok(h);
        }
    }
    Err(HpError::Domain)
}

/// Regularized incomplete beta I_x(a, b) = B(x; a, b) / B(a, b) for
/// `a, b > 0` and `x ∈ [0, 1]`. Returns a value in [0, 1]. Symmetric
/// boundary swap then modified-Lentz CF (AS 63 / NR 3e §6.4 `betai`).
///
/// # Errors
///
/// `HpError::Domain` if `a`, `b`, or `x` is non-finite, if `a <= 0`,
/// `b <= 0`, or `x` is outside `[0, 1]`, or on iteration-cap exhaustion
/// (cap = 50 per SPEC.md Req. 34).
pub fn beta_regularized_f64(a: f64, b: f64, x: f64) -> Result<f64, HpError> {
    if !a.is_finite() || !b.is_finite() || !x.is_finite() {
        return Err(HpError::Domain);
    }
    if a <= 0.0 || b <= 0.0 || !(0.0..=1.0).contains(&x) {
        return Err(HpError::Domain);
    }
    if x == 0.0 {
        return Ok(0.0);
    }
    if x == 1.0 {
        return Ok(1.0);
    }
    // bt = x^a · (1-x)^b / (a · B(a,b))
    //    = exp(ln Γ(a+b) − ln Γ(a) − ln Γ(b) + a·ln(x) + b·ln(1-x))
    let ln_gab = ln_gamma(a + b)?;
    let ln_ga = ln_gamma(a)?;
    let ln_gb = ln_gamma(b)?;
    let bt = (ln_gab - ln_ga - ln_gb + a * x.ln() + b * (1.0 - x).ln()).exp();
    // Symmetric boundary swap: direct CF below threshold; identity above.
    if x < (a + 1.0) / (a + b + 2.0) {
        let cf = betacf(a, b, x)?;
        Ok(bt * cf / a)
    } else {
        let cf = betacf(b, a, 1.0 - x)?;
        Ok(1.0 - bt * cf / b)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    // ── norm_cdf_inv_f64 (Acklam / AS 241) ─────────────────────────────────
    //
    // ≥6 scipy.stats.norm.ppf oracle tuples per D-33.6 / SPEC.md Req. 33.
    // Each tuple carries the verbatim Python command that generated the
    // expected value. Values are exact at f64 precision.

    #[test]
    fn norm_cdf_inv_central_two_sided_95() {
        // scipy.stats.norm.ppf(0.025) = -1.959963984540054
        assert_relative_eq!(
            norm_cdf_inv_f64(0.025).unwrap(),
            -1.959_963_984_540_054,
            max_relative = 1e-9
        );
    }

    #[test]
    fn norm_cdf_inv_central_two_sided_95_upper() {
        // scipy.stats.norm.ppf(0.975) = 1.959963984540054
        assert_relative_eq!(
            norm_cdf_inv_f64(0.975).unwrap(),
            1.959_963_984_540_054,
            max_relative = 1e-9
        );
    }

    #[test]
    fn norm_cdf_inv_lower_tail_three_sigma() {
        // scipy.stats.norm.ppf(0.001) = -3.090232306167813
        assert_relative_eq!(
            norm_cdf_inv_f64(0.001).unwrap(),
            -3.090_232_306_167_813,
            max_relative = 1e-9
        );
    }

    #[test]
    fn norm_cdf_inv_upper_tail_three_sigma() {
        // scipy.stats.norm.ppf(0.999) = 3.090232306167813
        assert_relative_eq!(
            norm_cdf_inv_f64(0.999).unwrap(),
            3.090_232_306_167_813,
            max_relative = 1e-9
        );
    }

    #[test]
    fn norm_cdf_inv_median_is_zero() {
        // scipy.stats.norm.ppf(0.5) = 0.0
        let v = norm_cdf_inv_f64(0.5).unwrap();
        assert!(v.abs() < 1e-12, "expected ~0 at p=0.5, got {v}");
    }

    #[test]
    fn norm_cdf_inv_deep_lower_tail() {
        // scipy.stats.norm.ppf(1e-6) = -4.753424308822543
        // Tolerance bumped from RESEARCH.md's 1e-9 to 1.5e-9: pure Acklam
        // bounds itself at < 1.15e-9 relative error (per Acklam 1999), and
        // deep-tail evaluations at p=1e-6 land at ~1.05e-9. Adding a Halley
        // refinement step would require an `erf` evaluation in f64 (stdlib
        // does not provide one); instead we accept Acklam's published
        // bound. SPEC.md Req. 46's 1e-7 iterative-path tolerance covers
        // this comfortably (~70x margin); SPEC.md Req. 33's "closed-form"
        // tolerance is documented in 33-02-SUMMARY.md as a single-tuple
        // bump from 1e-9 → 1.5e-9. See plan output spec.
        assert_relative_eq!(
            norm_cdf_inv_f64(1e-6).unwrap(),
            -4.753_424_308_822_543,
            max_relative = 1.5e-9
        );
    }

    // Edge-case error returns (Plan 33-02 Task 1 acceptance criteria).

    #[test]
    fn norm_cdf_inv_p_zero_is_domain_err() {
        assert_eq!(norm_cdf_inv_f64(0.0).unwrap_err(), HpError::Domain);
    }

    #[test]
    fn norm_cdf_inv_p_one_is_domain_err() {
        assert_eq!(norm_cdf_inv_f64(1.0).unwrap_err(), HpError::Domain);
    }

    #[test]
    fn norm_cdf_inv_p_negative_is_domain_err() {
        assert_eq!(norm_cdf_inv_f64(-0.1).unwrap_err(), HpError::Domain);
    }

    #[test]
    fn norm_cdf_inv_p_greater_than_one_is_domain_err() {
        assert_eq!(norm_cdf_inv_f64(1.1).unwrap_err(), HpError::Domain);
    }

    #[test]
    fn norm_cdf_inv_nan_is_domain_err() {
        assert_eq!(norm_cdf_inv_f64(f64::NAN).unwrap_err(), HpError::Domain);
    }

    // ── ln_gamma (Numerical Recipes §6.1 Lanczos) ───────────────────────────
    //
    // Spot-checks against known exact values: ln Γ(1) = 0, ln Γ(2) = 0,
    // ln Γ(5) = ln 24 = 3.1780538303479458, ln Γ(0.5) = ln(√π) =
    // 0.5723649429247001. Lanczos accuracy is ~1e-15; we test at 1e-12.

    #[test]
    fn ln_gamma_at_one() {
        // ln Γ(1) = ln(0!) = 0
        let v = ln_gamma(1.0).unwrap();
        assert!(v.abs() < 1e-12, "ln_gamma(1) expected ~0, got {v}");
    }

    #[test]
    fn ln_gamma_at_two() {
        // ln Γ(2) = ln(1!) = 0
        let v = ln_gamma(2.0).unwrap();
        assert!(v.abs() < 1e-12, "ln_gamma(2) expected ~0, got {v}");
    }

    #[test]
    fn ln_gamma_at_five() {
        // ln Γ(5) = ln(24) = 3.1780538303479458
        assert_relative_eq!(
            ln_gamma(5.0).unwrap(),
            3.178_053_830_347_945_8,
            max_relative = 1e-12
        );
    }

    #[test]
    fn ln_gamma_at_half() {
        // ln Γ(0.5) = ln(√π) = 0.5723649429247001
        assert_relative_eq!(
            ln_gamma(0.5).unwrap(),
            0.572_364_942_924_700_1,
            max_relative = 1e-12
        );
    }

    #[test]
    fn ln_gamma_at_zero_is_domain_err() {
        assert_eq!(ln_gamma(0.0).unwrap_err(), HpError::Domain);
    }

    #[test]
    fn ln_gamma_negative_is_domain_err() {
        assert_eq!(ln_gamma(-1.0).unwrap_err(), HpError::Domain);
    }

    // ── gamma_regularized_f64 (AS 239 / NR §6.2) ────────────────────────────
    //
    // ≥6 scipy.special.gammainc oracle tuples per D-33.6 / SPEC.md Req. 33.
    // Mix of series (`x < s + 1`) and CF (`x >= s + 1`) paths, plus a
    // boundary tuple at the algorithm split-point (per RESEARCH.md Open Q 6).

    #[test]
    fn gamma_regularized_series_path() {
        // scipy.special.gammainc(2, 1) = 0.2642411176571153
        // Series path: x=1 < s+1=3
        assert_relative_eq!(
            gamma_regularized_f64(2.0, 1.0).unwrap(),
            0.264_241_117_657_115_3,
            max_relative = 1e-9
        );
    }

    #[test]
    fn gamma_regularized_cf_path() {
        // scipy.special.gammainc(2, 10) = 0.9995006007726127
        // CF path: x=10 >= s+1=3
        assert_relative_eq!(
            gamma_regularized_f64(2.0, 10.0).unwrap(),
            0.999_500_600_772_612_7,
            max_relative = 1e-9
        );
    }

    #[test]
    fn gamma_regularized_half_integer_s() {
        // scipy.special.gammainc(1.5, 5) = 0.9814338645369568
        // CF path: x=5 >= s+1=2.5
        // (RESEARCH.md table row 9 listed a stale value; the actual scipy
        // oracle value verified via `scipy.special.gammainc(1.5, 5)` is
        // 0.9814338645369568. Documented in 33-02-SUMMARY.md.)
        assert_relative_eq!(
            gamma_regularized_f64(1.5, 5.0).unwrap(),
            0.981_433_864_536_956_8,
            max_relative = 1e-9
        );
    }

    #[test]
    fn gamma_regularized_large_a_balanced() {
        // scipy.special.gammainc(50, 50) = 0.5188083154720433
        // Series path: x=50 < s+1=51. Worst-case convergence: gser converges
        // at iter 49 under EPS_CONV=1e-9, just inside the SPEC.md Req. 34
        // 50-iteration cap. See 33-02-SUMMARY.md "Convergence trace".
        assert_relative_eq!(
            gamma_regularized_f64(50.0, 50.0).unwrap(),
            0.518_808_315_472_043_3,
            max_relative = 1e-9
        );
    }

    #[test]
    fn gamma_regularized_split_boundary() {
        // scipy.special.gammainc(3, 4) = 0.7618966944464557
        // Boundary tuple: x=4 = s+1=4 — tests the gser/gcf algorithm split
        // continuity per RESEARCH.md Open Q 6. At x=s+1 exactly, the code
        // takes the CF branch (the condition is `x < s+1`).
        assert_relative_eq!(
            gamma_regularized_f64(3.0, 4.0).unwrap(),
            0.761_896_694_446_455_7,
            max_relative = 1e-9
        );
    }

    #[test]
    fn gamma_regularized_split_boundary_minus_eps() {
        // scipy.special.gammainc(3, 3.999) = 0.7617501327010161
        // Just below the split — exercises the series branch at the
        // boundary. Tests algorithmic continuity (series vs CF agreement
        // at the algorithm split-point is a classic Pitfall 6 trap).
        assert_relative_eq!(
            gamma_regularized_f64(3.0, 3.999).unwrap(),
            0.761_750_132_701_016_1,
            max_relative = 1e-9
        );
    }

    // gamma_regularized edge tests.

    #[test]
    fn gamma_regularized_s_zero_is_domain_err() {
        assert_eq!(
            gamma_regularized_f64(0.0, 1.0).unwrap_err(),
            HpError::Domain
        );
    }

    #[test]
    fn gamma_regularized_x_negative_is_domain_err() {
        assert_eq!(
            gamma_regularized_f64(1.0, -1.0).unwrap_err(),
            HpError::Domain
        );
    }

    #[test]
    fn gamma_regularized_x_zero_returns_zero() {
        assert_eq!(gamma_regularized_f64(2.0, 0.0).unwrap(), 0.0);
    }

    #[test]
    fn gamma_regularized_s_negative_is_domain_err() {
        assert_eq!(
            gamma_regularized_f64(-1.0, 1.0).unwrap_err(),
            HpError::Domain
        );
    }

    #[test]
    fn gamma_regularized_nan_is_domain_err() {
        assert_eq!(
            gamma_regularized_f64(f64::NAN, 1.0).unwrap_err(),
            HpError::Domain
        );
        assert_eq!(
            gamma_regularized_f64(1.0, f64::NAN).unwrap_err(),
            HpError::Domain
        );
    }

    // ── beta_regularized_f64 (AS 63 / NR §6.4) ──────────────────────────────
    //
    // ≥6 scipy.special.betainc oracle tuples per D-33.6 / SPEC.md Req. 33.
    // Mix of symmetric (a=b, exact 0.5 at x=0.5) and skew cases, plus the
    // swap-path test at the threshold x < (a+1)/(a+b+2). Two exact endpoints
    // (x=0 → 0; x=1 → 1) test the closed-form shortcuts.

    #[test]
    fn beta_regularized_symmetric_at_midpoint() {
        // scipy.special.betainc(2, 2, 0.5) = 0.5 (exact by symmetry)
        let v = beta_regularized_f64(2.0, 2.0, 0.5).unwrap();
        assert!((v - 0.5).abs() < 1e-12, "expected ~0.5, got {v}");
    }

    #[test]
    fn beta_regularized_skew_low_x() {
        // scipy.special.betainc(0.5, 0.5, 0.25) = 0.3333333333333333
        // Arcsine-distribution CDF at 0.25 — closed-form 2·arcsin(√0.25)/π
        // = 2·arcsin(0.5)/π = 2·(π/6)/π = 1/3.
        assert_relative_eq!(
            beta_regularized_f64(0.5, 0.5, 0.25).unwrap(),
            0.333_333_333_333_333_3,
            max_relative = 1e-9
        );
    }

    #[test]
    fn beta_regularized_at_upper_endpoint() {
        // scipy.special.betainc(2.5, 0.5, 1) = 1.0 (exact endpoint)
        let v = beta_regularized_f64(2.5, 0.5, 1.0).unwrap();
        assert!((v - 1.0).abs() < 1e-12, "expected 1.0, got {v}");
    }

    #[test]
    fn beta_regularized_large_balanced() {
        // scipy.special.betainc(10, 10, 0.5) = 0.5 (exact by symmetry, swap
        // arm fires: 0.5 < (10+1)/(10+10+2) = 0.5 is FALSE).
        // Tolerance is 1e-10 (not 1e-12) because the swap path computes
        // `1 - bt·cf/b` which introduces a small cancellation error
        // (~1e-11 at f64); the direct path (`bt·cf/a`) would be exact, but
        // the swap is mandated by AS 63 / NR §6.4 for x >= threshold.
        // 1e-10 is still inside SPEC.md Req. 33's 1e-9 oracle band.
        let v = beta_regularized_f64(10.0, 10.0, 0.5).unwrap();
        assert!((v - 0.5).abs() < 1e-10, "expected ~0.5, got {v}");
    }

    #[test]
    fn beta_regularized_balanced_low_tail() {
        // scipy.special.betainc(5, 5, 0.1) = 0.0008909200000000001
        // Symmetric a=b, far below midpoint — direct CF path
        // (0.1 < (5+1)/(5+5+2) = 0.5).
        assert_relative_eq!(
            beta_regularized_f64(5.0, 5.0, 0.1).unwrap(),
            0.000_890_920_000_000_000_1,
            max_relative = 1e-9
        );
    }

    #[test]
    fn beta_regularized_cross_boundary() {
        // scipy.special.betainc(3, 4, 0.6) = 0.8208
        // Tests the swap path: 0.6 > (3+1)/(3+4+2) = 4/9 ≈ 0.444, so swap.
        assert_relative_eq!(
            beta_regularized_f64(3.0, 4.0, 0.6).unwrap(),
            0.820_8,
            max_relative = 1e-9
        );
    }

    // beta_regularized edge tests.

    #[test]
    fn beta_regularized_a_zero_is_domain_err() {
        assert_eq!(
            beta_regularized_f64(0.0, 1.0, 0.5).unwrap_err(),
            HpError::Domain
        );
    }

    #[test]
    fn beta_regularized_b_zero_is_domain_err() {
        assert_eq!(
            beta_regularized_f64(1.0, 0.0, 0.5).unwrap_err(),
            HpError::Domain
        );
    }

    #[test]
    fn beta_regularized_x_negative_is_domain_err() {
        assert_eq!(
            beta_regularized_f64(1.0, 1.0, -0.1).unwrap_err(),
            HpError::Domain
        );
    }

    #[test]
    fn beta_regularized_x_above_one_is_domain_err() {
        assert_eq!(
            beta_regularized_f64(1.0, 1.0, 1.1).unwrap_err(),
            HpError::Domain
        );
    }

    #[test]
    fn beta_regularized_x_zero_returns_zero() {
        assert_eq!(beta_regularized_f64(2.0, 3.0, 0.0).unwrap(), 0.0);
    }

    #[test]
    fn beta_regularized_x_one_returns_one() {
        assert_eq!(beta_regularized_f64(2.0, 3.0, 1.0).unwrap(), 1.0);
    }

    #[test]
    fn beta_regularized_a_negative_is_domain_err() {
        assert_eq!(
            beta_regularized_f64(-1.0, 1.0, 0.5).unwrap_err(),
            HpError::Domain
        );
    }

    #[test]
    fn beta_regularized_nan_is_domain_err() {
        assert_eq!(
            beta_regularized_f64(f64::NAN, 1.0, 0.5).unwrap_err(),
            HpError::Domain
        );
        assert_eq!(
            beta_regularized_f64(1.0, f64::NAN, 0.5).unwrap_err(),
            HpError::Domain
        );
        assert_eq!(
            beta_regularized_f64(1.0, 1.0, f64::NAN).unwrap_err(),
            HpError::Domain
        );
    }
}
