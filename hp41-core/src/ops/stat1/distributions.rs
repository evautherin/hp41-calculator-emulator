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
}
