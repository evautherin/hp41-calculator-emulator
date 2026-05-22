// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `stat1` — HP Statistics Pac 1 / "STAT 1B" (HP 00041-90030, 1979) operations.
//!
//! XROM module id: 2 (bit 1 of `CalcState::xrom_modules`).
//! Activated in Plan 33-01; this module-hub file ships first in Plan 33-00
//! as the SINGLE SOURCE OF TRUTH for Σ-register layout (P21 mitigation;
//! SPEC.md Req. 9).
//!
//! ## Σ Storage Register Layout — verbatim from OM 00041-90030 Appendix A "Program Data"
//!
//! All Stat 1 Pac programs use a contiguous register block starting at R00;
//! Appendix A (OM page 73–74) enumerates the highest 0-indexed register slot
//! ("DATA REGISTERS" column) and the corresponding SIZE directive each
//! program issues on initialization. The maximum across all 14 entry-point
//! mnemonics drives [`STAT1_MAX_REG`] below.
//!
//! ```text
//!                                                       #REG. TO     DATA          DISPLAY
//!   PROGRAM                                              COPY      REGISTERS  FLAGS  FORMAT
//!   Basic Statistics for Two Variables                    50       00 ~ 11   ...    FIX 2
//!   Moments, Skewness, and Kurtosis                       36       00 ~ 11   ...    FIX 2
//!   Analysis of Variance (One Way)                        29       00 ~ 19   ...    FIX 2
//!   Analysis of Variance (Two Way)                        33       00 ~ 17   ...    FIX 2
//!   Analysis of Covariance (One Way)                      60       00 ~ 25   ...    FIX 2
//!   Curve Fitting                                         34       00 ~ 15   ...    FIX 2
//!   Multiple Linear Regression                           157       00 ~ 44   ...    FIX 2
//!   Polynomial Regression                                102       00 ~ 44   ...    FIX 2
//!   t Statistics                                          29       00 ~ 14   ...    FIX 2
//!   Chi-Square Evaluation                                 21       00 ~ 07   ...    FIX 2
//!   Contingency Table                                     33       00 ~ 14   ...    FIX 2
//!   Spearman's Rank Correlation Coefficient               13       00 ~ 02   ...    FIX 2
//!   Normal and Inverse Normal Distribution                47       00 ~ 18   ...    FIX 2
//!   Chi-Square Distribution                               21       00 ~ 06   ...    FIX 2
//! ```
//!
//! Flag block uniformly `00~03, 21, 27, 29` for every program (OM Appendix A).
//!
//! ### Inline SIZE: directives (verbatim from per-program OM pages)
//!
//! - ΣBSTAT / ΣBSTG     → SIZE 012  (OM p. 11)
//! - ΣMMTUG / ΣMMTGD    → SIZE 012  (OM p. 15)  — same block layout as ΣBSTAT
//! - ΣAOVONE            → SIZE 020  (OM p. 20)
//! - ΣAOVTWO            → SIZE 018  (OM p. 23)
//! - ΣANOCOV            → SIZE 026  (OM p. 28)
//! - ΣLIN/EXP/LOGI/POW  → SIZE 016  (OM p. 35) — Curve Fitting program
//! - ΣMLRXY / ΣMLRXYZ   → SIZE 045  (OM p. 40, 41, 43, 44) — Multiple Linear Regression
//! - ΣPOLYP / ΣPOLYC    → SIZE 045  (OM p. 47, 48) — Polynomial Regression
//! - ΣPTST / ΣTSTAT     → SIZE 015  (OM p. 52)
//! - ΣXSQEV / ΣEFXSQ    → SIZE 008  (OM p. 55)
//! - ΣCTKKK / ΣCTKK     → SIZE 015  (OM p. 60)
//! - ΣSPEAR             → SIZE 003  (OM p. 64) — closed-form, smallest footprint
//! - ΣNORMD             → SIZE 019  (OM p. 67)
//! - ΣCHISQD            → SIZE 007  (OM p. 71)
//!
//! Per-slot semantics (Σx, Σx², n, group sums, cross-products, marginal totals,
//! etc.) are decoded in the corresponding `stat1/<program>.rs` files when each
//! Op is implemented (Plans 33-04 onward) by reading the program-listing
//! section of OM 00041-90030. P21 mitigation: every Stat 1 register access
//! routes through a named const declared in THIS file — no literal-integer
//! register indices may appear in `stat1/*.rs` algorithm code. The Phase 37
//! lint extension (STAT-QUAL-06) enforces this at CI gate time.
//!
//! ## Submodule structure (per D-33.5; 10 algorithm modules + this hub)
//!
//! - `distributions` — 3 hand-coded f64-bridge primitives (Acklam/AS 241,
//!   AS 239, AS 63) — Plan 33-02
//! - `normd`         — ΣNORMD 3-mode dispatcher (CDF / PDF / inverse) — Plan 33-03
//! - `chisqd`        — ΣCHISQD ν-prompt + PDF / CDF — Plan 33-03
//! - `basic_stats`   — ΣBSTAT / ΣBSTG — Plan 33-05
//! - `moments`       — ΣMMTUG / ΣMMTGD — Plan 33-06
//! - `anova`         — ΣAOVONE / ΣAOVTWO / ΣANOCOV — Plan 33-06
//! - `regression`    — ΣLIN/EXP/LOGI/POW + ΣMLRXY/MLRXYZ + ΣPOLYP/POLYC — Plans 33-05, 33-08
//! - `hypothesis`    — ΣPTST / ΣTSTAT (renamed from tests.rs per D-33.5) — Plan 33-07
//! - `nonparam`      — ΣSPEAR + ΣXSQEV/ΣEFXSQ + ΣCTKKK/ΣCTKK — Plans 33-04, 33-06
//! - `rand`          — RAND / SEED (emulator extension per D-33.4) — Plan 33-08
//!
//! Submodule `pub mod` declarations are intentionally commented out in
//! Plan 33-00; each plan uncomments its sibling line as the corresponding
//! algorithm file lands. This keeps `cargo check -p hp41-core` GREEN at every
//! plan boundary (per CLAUDE.md "Frozen Invariants — Workspace structure":
//! `hp41-core` must compile clean before any caller).
//!
//! ## References
//!
//! - HP-41C Stat 1 Pac Owner's Manual 00041-90030 Rev. E (HP Portable
//!   Computer Division, Corvallis OR, 1979; printed Singapore 8/84).
//! - HP-41C Stat Pac Quick Reference Card 00041-90061 (June 1979).
//! - NPS55-84-003 (Zehna, Naval Postgraduate School, Feb 1984;
//!   DTIC AD-A140573) — independent algorithm cross-check.

// ── Submodule declarations (uncommented per plan as algorithm files land) ──
// pub mod anova;        // Plan 33-06
// pub mod basic_stats;  // Plan 33-05
// pub mod chisqd;       // Plan 33-03
// pub mod distributions; // Plan 33-02
// pub mod hypothesis;   // Plan 33-07 (renamed from tests.rs per D-33.5)
pub mod modal; // Plan 33-01 (modal-prompt step carrier for Stat 1 workflows)
               // pub mod moments;      // Plan 33-06
               // pub mod nonparam;     // Plan 33-04
               // pub mod normd;        // Plan 33-03
               // pub mod rand;         // Plan 33-08
               // pub mod regression;   // Plans 33-05 + 33-08

// ── Σ-register layout constants (single source of truth; OM Appendix A) ──

/// Highest 0-indexed Σ-register slot touched by any Stat 1 Pac accumulator.
///
/// Value `44` derived from OM 00041-90030 Appendix A "Program Data" table
/// (OM p. 73–74): Multiple Linear Regression and Polynomial Regression both
/// declare `SIZE: 045` and use registers `00 ~ 44`. Every other program in
/// the Pac uses a strict subset of this range.
///
/// SIZE-floor guard pattern at the entry of every Stat 1 Op:
/// ```ignore
/// if state.regs.len() < hp41_core::ops::stat1::STAT1_MAX_REG + 1 {
///     return Err(HpError::InvalidOp);
/// }
/// ```
///
/// This mirrors the existing R06 floor at `ops/stats.rs:25` for v1.x Σ+/Σ−.
pub const STAT1_MAX_REG: usize = 44;

/// Per-program SIZE-floor: ΣBSTAT / ΣBSTG (Basic Statistics for Two Variables).
///
/// Highest 0-indexed register slot is `11`; SIZE 012 (OM p. 11).
/// Layout: R00–R11 hold Σ, Σ², n, group-frequency totals — per-slot semantics
/// in `stat1/basic_stats.rs` (Plan 33-05).
pub const STAT1_BSTAT_MAX_REG: usize = 11;

/// Per-program SIZE-floor: ΣMMTUG / ΣMMTGD (Moments, Skewness, Kurtosis).
///
/// Highest 0-indexed register slot is `11`; SIZE 012 (OM p. 15). Shares the
/// Σx/Σx² block with ΣBSTAT and extends to Σx³ + Σx⁴ for third + fourth
/// raw moments. Per-slot semantics in `stat1/moments.rs` (Plan 33-06).
pub const STAT1_MMTUG_MAX_REG: usize = 11;

/// Per-program SIZE-floor: ΣAOVONE (Analysis of Variance, One Way).
///
/// Highest 0-indexed register slot is `19`; SIZE 020 (OM p. 20). Holds
/// per-group Σx, Σx², n_i across `k` groups for between-group / within-group
/// sums of squares. Per-slot semantics in `stat1/anova.rs` (Plan 33-06).
pub const STAT1_AOVONE_MAX_REG: usize = 19;

/// Per-program SIZE-floor: ΣAOVTWO (Analysis of Variance, Two Way, No Replications).
///
/// Highest 0-indexed register slot is `17`; SIZE 018 (OM p. 23). Holds
/// row + column marginal sums of squares. Per-slot semantics in
/// `stat1/anova.rs` (Plan 33-06).
pub const STAT1_AOVTWO_MAX_REG: usize = 17;

/// Per-program SIZE-floor: ΣANOCOV (Analysis of Covariance, One Way).
///
/// Highest 0-indexed register slot is `25`; SIZE 026 (OM p. 28). Holds
/// group-keyed sums and covariate cross-products. Per-slot semantics in
/// `stat1/anova.rs` (Plan 33-06).
pub const STAT1_ANOCOV_MAX_REG: usize = 25;

/// Per-program SIZE-floor: ΣLIN / ΣEXP / ΣLOGI / ΣPOW (Curve Fitting).
///
/// Highest 0-indexed register slot is `15`; SIZE 016 (OM p. 35). All four
/// curve-fit variants share the same register block (transforms applied
/// before accumulation). Per-slot semantics in `stat1/regression.rs`
/// (Plan 33-05).
pub const STAT1_CURVEFIT_MAX_REG: usize = 15;

/// Per-program SIZE-floor: ΣMLRXY / ΣMLRXYZ (Multiple Linear Regression).
///
/// Highest 0-indexed register slot is `44`; SIZE 045 (OM p. 40, 41, 43, 44).
/// Holds the (d+1)×(d+1) normal-equation augmented matrix for `d = 2` or
/// `d = 3` predictors plus per-predictor sums and cross-products. Drives
/// [`STAT1_MAX_REG`]. Per-slot semantics in `stat1/regression.rs` (Plan 33-08).
pub const STAT1_MLR_MAX_REG: usize = 44;

/// Per-program SIZE-floor: ΣPOLYP / ΣPOLYC (Polynomial Regression).
///
/// Highest 0-indexed register slot is `44`; SIZE 045 (OM p. 47, 48). Shares
/// the 45-register footprint with multiple regression — same normal-equation
/// Gauss-elimination machinery for cubic / parabolic fits. Per-slot semantics
/// in `stat1/regression.rs` (Plan 33-08).
pub const STAT1_POLYP_MAX_REG: usize = 44;

/// Per-program SIZE-floor: ΣPTST / ΣTSTAT (t Statistics).
///
/// Highest 0-indexed register slot is `14`; SIZE 015 (OM p. 52). Holds the
/// two-sample sufficient statistics for pooled-variance t (Welch explicitly
/// excluded per SPEC.md Req. 25). Per-slot semantics in `stat1/hypothesis.rs`
/// (Plan 33-07).
pub const STAT1_TSTAT_MAX_REG: usize = 14;

/// Per-program SIZE-floor: ΣXSQEV / ΣEFXSQ (Chi-Square Evaluation).
///
/// Highest 0-indexed register slot is `7`; SIZE 008 (OM p. 55). Holds
/// observed / expected counts (or expected proportions for ΣEFXSQ).
/// Per-slot semantics in `stat1/nonparam.rs` (Plan 33-04).
pub const STAT1_XSQEV_MAX_REG: usize = 7;

/// Per-program SIZE-floor: ΣCTKKK / ΣCTKK (Contingency Table).
///
/// Highest 0-indexed register slot is `14`; SIZE 015 (OM p. 60). Holds the
/// r×c cell counts plus row + column marginal totals. Per-slot semantics in
/// `stat1/nonparam.rs` (Plan 33-06).
pub const STAT1_CTKKK_MAX_REG: usize = 14;

/// Per-program SIZE-floor: ΣSPEAR (Spearman's Rank Correlation Coefficient).
///
/// Highest 0-indexed register slot is `2`; SIZE 003 (OM p. 64). The smallest
/// footprint in the Pac — closed-form ρ_s reads the existing v1.x R01–R06
/// Σ-register block populated by `op_sigma_plus` and accumulates only `Σd²`
/// in the local 3-register window. Per-slot semantics in `stat1/nonparam.rs`
/// (Plan 33-04).
pub const STAT1_SPEAR_MAX_REG: usize = 2;

/// Per-program SIZE-floor: ΣNORMD (Normal and Inverse Normal Distribution).
///
/// Highest 0-indexed register slot is `18`; SIZE 019 (OM p. 67). Holds
/// intermediate Newton-iteration state plus the cached polynomial-approximation
/// coefficient block. Per-slot semantics in `stat1/normd.rs` (Plan 33-03).
pub const STAT1_NORMD_MAX_REG: usize = 18;

/// Per-program SIZE-floor: ΣCHISQD (Chi-Square Distribution).
///
/// Highest 0-indexed register slot is `6`; SIZE 007 (OM p. 71). Holds ν
/// (degrees of freedom, integer) plus the series-approximation accumulators.
/// Per-slot semantics in `stat1/chisqd.rs` (Plan 33-03).
pub const STAT1_CHISQD_MAX_REG: usize = 6;

// ── Phase 33 Plan 33-01 scaffolding (TO BE REMOVED by end of Phase 33) ─────

use crate::error::HpError;
use crate::state::CalcState;

/// Placeholder dispatch handler for `Op::Stat1Stub`.
///
/// Plan-33-01 scaffolding: every entry in `STAT_1.ops` (in
/// `hp41-core/src/ops/math1/xrom.rs`) currently maps to `Op::Stat1Stub`,
/// and `stat1_resolve` in the same file routes every Stat 1 mnemonic to
/// the same stub. Plans 33-03..33-08 incrementally replace those
/// references with real `Op::Sigma*` variants and DELETE `Op::Stat1Stub`
/// + this function once the last reference is gone (Plan 33-08).
///
/// Returns `Err(HpError::InvalidOp)` so any accidental dispatch surfaces
/// the missing-feature state visibly (rather than silently mapping to a
/// no-op — Pitfall 22 / resolver-never-discard invariant per CLAUDE.md
/// "Resolver chain + never-discard").
///
/// Per `#![deny(clippy::unwrap_used)]`: this function uses `Err(...)`
/// propagation; no panics, no `.unwrap()`.
pub fn op_stat1_stub(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// STAT1_MAX_REG must dominate every per-program max-reg const, since
    /// it is the SIZE-floor used by `state.regs.len() < STAT1_MAX_REG + 1`
    /// guards across all Stat 1 Pac Ops (P21 mitigation: single source of
    /// truth for the floor).
    #[test]
    fn stat1_max_reg_dominates_per_program_consts() {
        let per_program = [
            ("BSTAT", STAT1_BSTAT_MAX_REG),
            ("MMTUG", STAT1_MMTUG_MAX_REG),
            ("AOVONE", STAT1_AOVONE_MAX_REG),
            ("AOVTWO", STAT1_AOVTWO_MAX_REG),
            ("ANOCOV", STAT1_ANOCOV_MAX_REG),
            ("CURVEFIT", STAT1_CURVEFIT_MAX_REG),
            ("MLR", STAT1_MLR_MAX_REG),
            ("POLYP", STAT1_POLYP_MAX_REG),
            ("TSTAT", STAT1_TSTAT_MAX_REG),
            ("XSQEV", STAT1_XSQEV_MAX_REG),
            ("CTKKK", STAT1_CTKKK_MAX_REG),
            ("SPEAR", STAT1_SPEAR_MAX_REG),
            ("NORMD", STAT1_NORMD_MAX_REG),
            ("CHISQD", STAT1_CHISQD_MAX_REG),
        ];
        for (name, max_reg) in per_program {
            assert!(
                max_reg <= STAT1_MAX_REG,
                "{name}: per-program max-reg {max_reg} exceeds STAT1_MAX_REG {STAT1_MAX_REG}"
            );
        }
    }

    /// STAT1_MAX_REG must equal the max of all per-program consts (so SIZE 045
    /// is not over-allocated by accident). OM Appendix A pins it at 44 via
    /// MLR + POLYP.
    #[test]
    fn stat1_max_reg_equals_max_per_program() {
        let max_per_program = [
            STAT1_BSTAT_MAX_REG,
            STAT1_MMTUG_MAX_REG,
            STAT1_AOVONE_MAX_REG,
            STAT1_AOVTWO_MAX_REG,
            STAT1_ANOCOV_MAX_REG,
            STAT1_CURVEFIT_MAX_REG,
            STAT1_MLR_MAX_REG,
            STAT1_POLYP_MAX_REG,
            STAT1_TSTAT_MAX_REG,
            STAT1_XSQEV_MAX_REG,
            STAT1_CTKKK_MAX_REG,
            STAT1_SPEAR_MAX_REG,
            STAT1_NORMD_MAX_REG,
            STAT1_CHISQD_MAX_REG,
        ]
        .into_iter()
        .max()
        .expect("non-empty per-program list");
        assert_eq!(STAT1_MAX_REG, max_per_program);
    }
}
