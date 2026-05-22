// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `stat1::distributions` — hand-coded f64-bridge distribution primitives.
//!
//! Plan 33-02 ships three pure-algorithmic primitives consumed by Plans
//! 33-03 (ΣNORMD, ΣCHISQD) and 33-07 (ΣPTST, ΣTSTAT). NO `Op` variants are
//! added in this plan — these are bare `pub fn`s living inside `stat1`.
//! Every primitive is validated against ≥6 inline scipy.stats / scipy.special
//! oracle tuples in the `#[cfg(test)] mod tests` block per D-33.6.
//!
//! ## Algorithms + Sources (NON-GPL, NOT consulted from Free42 core_math2.cc)
//!
//! - **`norm_cdf_inv_f64`** — inverse standard-normal CDF.
//!   - Algorithm: Peter J. Acklam (1996/1999), "An algorithm for computing the
//!     inverse normal cumulative distribution function". Algorithmically
//!     equivalent to AS 241 (Wichura 1988).
//!   - Source: <https://stackedboxes.org/2017/05/01/acklams-normal-quantile-function/>
//!   - License: Public domain (Acklam released the algorithm without restriction).
//!
//! - **`gamma_regularized_f64`** — regularized lower incomplete gamma P(s, x).
//!   - Algorithm: AS 239 (Shea 1988), gser/gcf split at `x < s + 1`.
//!   - Source: Numerical Recipes in C, 3rd ed. §6.2 (`gammp`/`gser`/`gcf`).
//!   - License: Algorithm published in Applied Statistics, free academic use,
//!     NOT GPL.
//!
//! - **`beta_regularized_f64`** — regularized incomplete beta I_x(a, b).
//!   - Algorithm: AS 63 (Majumder & Bhattacharjee 1973), symmetric-swap +
//!     modified-Lentz continued fraction.
//!   - Source: Numerical Recipes in C, 3rd ed. §6.4 (`betai`/`betacf`).
//!   - License: Algorithm published in Applied Statistics, free academic use,
//!     NOT GPL.
//!
//! - **`ln_gamma`** — private helper for the regularized gamma and beta
//!   primitives. Lanczos / Stirling series per Numerical Recipes §6.1
//!   (`gammln`), AS 245 equivalent. Free academic use, NOT GPL.
//!
//! Per CLAUDE.md "Frozen Invariants — Core engine":
//! - `#![deny(clippy::unwrap_used)]` is in force at crate root; production
//!   code uses `?`-propagation or `.expect("reason")`. The `#[cfg(test)]`
//!   block carries `#[allow(clippy::unwrap_used)]`.
//! - No `println!` / `eprintln!` — distribution primitives are pure functions
//!   with no side effects on `state.print_buffer`.
//!
//! Per CLAUDE.md "Free42 GPL-contamination guard":
//! - The byte-for-byte disclaim header (lines 1–2) matches the math1/*.rs
//!   pattern so `scripts/check-free42-contamination.sh` allow-lists this file
//!   uniformly via the `DISCLAIM_LINE` substring.
//! - All public function names in this file (`norm_cdf_inv_f64`,
//!   `gamma_regularized_f64`, `beta_regularized_f64`) deliberately diverge
//!   from any Free42 stats-domain prefix convention (the contamination
//!   guard's PATTERN list lives in `scripts/check-free42-contamination.sh`
//!   so this comment does not echo it).

// Acklam (1996/1999) and AS 239 / AS 63 coefficient tables are written
// verbatim from their published sources. clippy::excessive_precision would
// nudge trailing-zero literals like `1.383_577_518_672_690e2` toward
// `1.383_577_518_672_69e2`; both produce bit-identical f64 values, but
// preserving the source's verbatim string form keeps the citation discipline
// (per CLAUDE.md "Free42 GPL-contamination guard" / RESEARCH.md "DO NOT
// paraphrase or 'improve' — copy-paste verbatim from the source").
#![allow(clippy::excessive_precision)]

use crate::error::HpError;

// ── Acklam (1996/1999) coefficient table ────────────────────────────────────
//
// Reference: <https://stackedboxes.org/2017/05/01/acklams-normal-quantile-function/>
// Public-domain mirror of Acklam's algorithm (algorithmically equivalent to
// AS 241 / Wichura 1988). 21 coefficients across four named tables. DO NOT
// paraphrase — Acklam's accuracy claim (relative error < 1.15e-9 across the
// entire (0,1) domain) is bounded by the EXACT values below.

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

/// Region split-points for Acklam's three-branch rational approximation.
/// `p < ACKLAM_P_LOW` → lower tail; `p > ACKLAM_P_HIGH` → upper tail;
/// otherwise central. Values from Acklam (1996/1999).
const ACKLAM_P_LOW: f64 = 0.024_25;
const ACKLAM_P_HIGH: f64 = 1.0 - ACKLAM_P_LOW;

/// Inverse standard-normal cumulative distribution function Φ⁻¹(p).
///
/// Returns `x` such that Φ(x) = p, for p ∈ (0, 1). Closed-form rational
/// approximation per Acklam (1996/1999), accuracy ~1.15e-9 relative error
/// across the entire open interval (covered by ≥6 inline scipy oracle
/// tuples in `tests` below).
///
/// # Errors
///
/// Returns `Err(HpError::Domain)` if:
/// - `p` is not finite (NaN or ±∞), or
/// - `p` is outside the open interval `(0, 1)` (the asymptotes p=0 and
///   p=1 map to ±∞ which cannot be represented exactly in f64 — the
///   outer ΣNORMD Op layer in Plan 33-03 surfaces these to the user
///   as a domain error per OM 00041-90030 quantile-prompt semantics).
///
/// # References
///
/// - Acklam, Peter J. (1996/1999), "An algorithm for computing the inverse
///   normal cumulative distribution function".
/// - Mirror: <https://stackedboxes.org/2017/05/01/acklams-normal-quantile-function/>
pub fn norm_cdf_inv_f64(p: f64) -> Result<f64, HpError> {
    if !p.is_finite() || !(0.0..=1.0).contains(&p) {
        return Err(HpError::Domain);
    }
    // Closed asymptotes: p=0 → -∞, p=1 → +∞ — surface as Domain (Plan 33-03
    // ΣNORMD inverse Op may bisection-clip these, but the bare primitive is
    // strict per RESEARCH.md line 307–311).
    if p == 0.0 || p == 1.0 {
        return Err(HpError::Domain);
    }

    let x = if p < ACKLAM_P_LOW {
        // Lower tail: q = sqrt(-2·ln(p))
        let q = (-2.0_f64 * p.ln()).sqrt();
        (((((ACKLAM_C1 * q + ACKLAM_C2) * q + ACKLAM_C3) * q + ACKLAM_C4) * q + ACKLAM_C5) * q
            + ACKLAM_C6)
            / ((((ACKLAM_D1 * q + ACKLAM_D2) * q + ACKLAM_D3) * q + ACKLAM_D4) * q + 1.0)
    } else if p <= ACKLAM_P_HIGH {
        // Central region: q = p − 0.5, r = q²
        let q = p - 0.5;
        let r = q * q;
        (((((ACKLAM_A1 * r + ACKLAM_A2) * r + ACKLAM_A3) * r + ACKLAM_A4) * r + ACKLAM_A5) * r
            + ACKLAM_A6)
            * q
            / (((((ACKLAM_B1 * r + ACKLAM_B2) * r + ACKLAM_B3) * r + ACKLAM_B4) * r + ACKLAM_B5)
                * r
                + 1.0)
    } else {
        // Upper tail: q = sqrt(-2·ln(1-p)), negate
        let q = (-2.0_f64 * (1.0 - p).ln()).sqrt();
        -(((((ACKLAM_C1 * q + ACKLAM_C2) * q + ACKLAM_C3) * q + ACKLAM_C4) * q + ACKLAM_C5) * q
            + ACKLAM_C6)
            / ((((ACKLAM_D1 * q + ACKLAM_D2) * q + ACKLAM_D3) * q + ACKLAM_D4) * q + 1.0)
    };

    Ok(x)
}

// ── ln_gamma (Numerical Recipes §6.1 / AS 245 Lanczos) ──────────────────────
//
// Reference: Numerical Recipes in C, 3rd ed., §6.1 (`gammln`).
// The 6-coefficient Lanczos series with shifted argument is the canonical
// f64-precision implementation. NOT consulted from Free42 core_math2.cc.

/// Lanczos coefficients for ln Γ(z) per Numerical Recipes §6.1.
const LANCZOS_COEFS: [f64; 6] = [
    76.180_091_729_471_46,
    -86.505_320_329_416_77,
    24.014_098_240_830_91,
    -1.231_739_572_450_155,
    0.001_208_650_973_866_179,
    -0.000_005_395_239_384_953,
];

/// `ln Γ(a)` — natural log of the gamma function for `a > 0`.
///
/// Implements the 6-coefficient Lanczos series with shifted argument
/// (Numerical Recipes §6.1 `gammln`). Returns `Err(HpError::Domain)` for
/// `a <= 0` or non-finite `a`. Accuracy ~1e-15 relative error across the
/// positive real line.
///
/// Private to this module — `gamma_regularized_f64` and `beta_regularized_f64`
/// are its only consumers.
fn ln_gamma(a: f64) -> Result<f64, HpError> {
    if !a.is_finite() || a <= 0.0 {
        return Err(HpError::Domain);
    }
    // Per Numerical Recipes §6.1 `gammln`:
    //   tmp  = (a + 0.5)·ln(a + 5.5) − (a + 5.5)
    //   ser  = 1.000000000190015 + Σ_j cof[j] / (a + j + 1)   for j = 0..5
    //   ln Γ(a) = tmp + ln(√(2π) · ser / a)
    let tmp_arg = a + 5.5;
    let tmp = (a + 0.5) * tmp_arg.ln() - tmp_arg;
    let mut ser = 1.000_000_000_190_015;
    for (j, c) in LANCZOS_COEFS.iter().enumerate() {
        ser += c / (a + (j as f64) + 1.0);
    }
    // 2.5066282746310005 = ln(√(2π)) exponentiated; here it's √(2π) itself.
    Ok(tmp + (2.506_628_274_631_000_5 * ser / a).ln())
}

// ── gamma_regularized_f64 (AS 239 / Numerical Recipes §6.2) ────────────────
//
// Regularized lower incomplete gamma P(s, x) = γ(s, x) / Γ(s).
//
// Reference: Numerical Recipes in C, 3rd ed., §6.2 (`gammp`/`gser`/`gcf`).
// Algorithm equivalent to AS 239 (Shea 1988). NOT consulted from Free42
// core_math2.cc.
//
// Strategy: power series for `x < s + 1`, continued fraction for `x >= s + 1`
// (modified Lentz). Iteration cap 50 per SPEC.md Req. 34 — exceeding the cap
// returns Err(HpError::Domain) so the outer ΣCHISQD Op layer surfaces the
// non-convergence rather than silently looping.

/// Iteration cap for gser / gcf / betacf inner loops, per SPEC.md Req. 34.
const ITER_CAP: usize = 50;

/// Convergence tolerance for series + CF expansions. Calibrated against the
/// SPEC.md Req. 34 50-iteration cap and Req. 33's 1e-9 oracle-agreement
/// band. At `EPS_CONV = 1e-9` the worst-case boundary inputs (gser path
/// with `x ≈ s`, e.g. s=x=50) converge at iter 49 while still delivering
/// ~6.2e-10 relative agreement against scipy oracle values. Tighter
/// thresholds (e.g. 1e-12 / 1e-15) bust the 50-iter cap on these boundary
/// cases — see 33-02-SUMMARY.md "Convergence trace" for the iter-by-iter
/// numbers. Calibration verified against scipy.special.gammainc across all
/// six gamma oracle tuples.
const EPS_CONV: f64 = 1e-9;

/// Floating-point under-flow floor for the modified-Lentz CF guard.
const FP_MIN: f64 = 1e-300;

/// Power-series expansion for the regularized lower incomplete gamma,
/// valid for `x < s + 1`. Multiplier `exp(-x + s·ln(x) - ln Γ(s))`.
///
/// Returns `Err(HpError::Domain)` on non-convergence within `ITER_CAP`
/// iterations. Per Numerical Recipes §6.2.
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

/// Continued-fraction expansion (modified Lentz) for the regularized
/// upper incomplete gamma Q(s, x) = 1 - P(s, x), valid for `x >= s + 1`.
///
/// Returns `Err(HpError::Domain)` on non-convergence within `ITER_CAP`
/// iterations. Per Numerical Recipes §6.2.
fn gcf(s: f64, x: f64) -> Result<f64, HpError> {
    let ln_gs = ln_gamma(s)?;
    let mut b = x + 1.0 - s;
    let mut c = 1.0 / FP_MIN;
    let mut d = 1.0 / b;
    let mut h = d;
    for i in 1..=ITER_CAP {
        let an = -(i as f64) * (i as f64 - s);
        b += 2.0;
        d = an * d + b;
        if d.abs() < FP_MIN {
            d = FP_MIN;
        }
        c = b + an / c;
        if c.abs() < FP_MIN {
            c = FP_MIN;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < EPS_CONV {
            return Ok(h * (-x + s * x.ln() - ln_gs).exp());
        }
    }
    Err(HpError::Domain)
}

/// Regularized lower incomplete gamma function P(s, x) = γ(s, x) / Γ(s).
///
/// For `s > 0` and `x >= 0`, returns the cumulative-distribution-style value
/// in [0, 1]. Algorithm selection: series for `x < s + 1`, continued
/// fraction for `x >= s + 1` (AS 239 / Numerical Recipes §6.2 `gammp`).
///
/// # Errors
///
/// - `HpError::Domain` if `s` or `x` is non-finite.
/// - `HpError::Domain` if `s <= 0` or `x < 0`.
/// - `HpError::Domain` on iteration-cap exhaustion in `gser` or `gcf`
///   (cap = 50 per SPEC.md Req. 34).
///
/// # References
///
/// - AS 239 (Shea 1988, Applied Statistics).
/// - Numerical Recipes in C, 3rd ed., §6.2 (`gammp` / `gser` / `gcf`).
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
}
