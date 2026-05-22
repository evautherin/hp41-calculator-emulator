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
//! at or above R07 routes through a named const declared in THIS file — no
//! literal-integer register indices may appear in `stat1/*.rs` algorithm code
//! for slots ≥ R07. The Phase 37 lint extension (STAT-QUAL-06) enforces this
//! at CI gate time.
//!
//! ## v1.x R01–R06 literal-index exemption (REVIEW.md WR-02)
//!
//! Registers R01–R06 are the FOUNDATIONAL v1.x Σ-block (Σx², Σx, n,
//! Σy², Σy, Σxy per `hp41-core/src/ops/stats.rs` module header) and
//! pre-date the P21 named-const policy. `ops/stats.rs::op_sigma_plus`
//! and `op_sigma_minus` themselves access these slots via literal
//! `state.regs[1..=6]` for backward compatibility with v1.0 save files.
//! Stat 1 Pac modules that delegate to / consume this block —
//! `basic_stats` (ΣBSTAT/ΣBSTG), `moments` (ΣMMTUG/ΣMMTGD), `hypothesis`
//! (ΣPTST's v1.x Σ-block reader) — are PERMITTED to access R01–R06
//! via literal index for symmetry with the canonical layout source.
//! Stat-1-specific slots that happen to fall in the R00–R06 range
//! (e.g. ΣAOVTWO's r/c at R00/R01, grand sums at R03/R04, row base at
//! R05) DO route through named consts because they are NOT part of
//! the v1.x Σ-block — they are program-specific addresses chosen by
//! the OM Stat 1 Pac layout (per WR-01 fix). Future migration to
//! named-const aliases for R01–R06 is OPTIONAL and would require
//! parallel changes to `ops/stats.rs`; the present project policy
//! treats literal R01–R06 access in v1.x-consuming code as canonical.
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
pub mod anova; // Plan 33-06 (ΣAOVONE + ΣAOVTWO + ΣANOCOV)
pub mod basic_stats; // Plan 33-05 (ΣBSTAT + ΣBSTG univariate / weighted summaries)
pub mod chisqd; // Plan 33-03 (ΣCHISQD ν-prompt + PDF/CDF dispatcher)
pub mod distributions; // Plan 33-02
pub mod hypothesis; // Plan 33-07 (ΣPTST; renamed from tests.rs per D-33.5; ΣTSTAT added in Task 2)
pub mod modal; // Plan 33-01 (modal-prompt step carrier for Stat 1 workflows)
pub mod moments; // Plan 33-06 (ΣMMTUG + ΣMMTGD third/fourth moments)
pub mod nonparam; // Plan 33-04 (ΣSPEAR + ΣXSQEV / ΣEFXSQ closed-form non-parametric Ops); Plan 33-06 extends with ΣCTKKK + ΣCTKK
pub mod normd; // Plan 33-03 (ΣNORMD 3-mode dispatcher: CDF / PDF / inverse)
pub mod rand; // Plan 33-08 (RAND / SEED — emulator extension per D-33.4)
pub mod regression; // Plan 33-05 (ΣLIN/EXP/LOGI/POW curve fits via op_sigma_plus delegate); Plan 33-08 extends with ΣMLRXY/MLRXYZ + ΣPOLYP/POLYC

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

// ── Plan 33-04 Task 2: ΣXSQEV per-slot register consts (P21 mitigation) ────
//
// ΣXSQEV / ΣEFXSQ both use SIZE 008 (R00..R07) per OM 00041-90030 p. 55.
// The 8-register block is INTERLEAVED observed/expected pairs for up to 3
// categories, matching the typical HP-41 Stat Pac convention for
// goodness-of-fit accumulators:
//
//   R00 = k (number of categories; 1 ≤ k ≤ STAT1_XSQEV_KMAX = 3)
//   R01 = O₀  (observed count for cell 0)
//   R02 = E₀  (expected count for cell 0; for ΣEFXSQ, expected PROPORTION)
//   R03 = O₁
//   R04 = E₁
//   R05 = O₂
//   R06 = E₂
//   R07 = scratch / χ² accumulator (output, also placed on stack X)
//
// The base + stride pair (`STAT1_XSQEV_OBS_BASE_REG = 1`,
// `STAT1_XSQEV_STRIDE = 2`) lets each Op address cell `i` as
// `regs[OBS_BASE + STRIDE * i]` (observed) and
// `regs[EXP_BASE + STRIDE * i]` (expected) — all bounded by the
// per-program SIZE-floor `STAT1_XSQEV_MAX_REG` (= 7).

/// ΣXSQEV / ΣEFXSQ: register holding `k`, the number of categories
/// (1 ≤ k ≤ 3 for SIZE 008 block).
///
/// OM 00041-90030 p. 55 (Chi-Square Evaluation).
pub const STAT1_XSQEV_K_REG: usize = 0;

/// ΣXSQEV / ΣEFXSQ: base register for the first observed-count cell.
///
/// Cell `i` observed value is at `regs[STAT1_XSQEV_OBS_BASE_REG + STAT1_XSQEV_STRIDE * i]`.
///
/// OM 00041-90030 p. 55.
pub const STAT1_XSQEV_OBS_BASE_REG: usize = 1;

/// ΣXSQEV / ΣEFXSQ: base register for the first expected-count cell
/// (or expected-proportion cell for ΣEFXSQ).
///
/// Cell `i` expected value is at `regs[STAT1_XSQEV_EXP_BASE_REG + STAT1_XSQEV_STRIDE * i]`.
///
/// OM 00041-90030 p. 55.
pub const STAT1_XSQEV_EXP_BASE_REG: usize = 2;

/// ΣXSQEV / ΣEFXSQ: stride between successive observed (or expected) cells
/// in the interleaved 8-register layout (R00..R07).
///
/// OM 00041-90030 p. 55.
pub const STAT1_XSQEV_STRIDE: usize = 2;

/// ΣXSQEV / ΣEFXSQ: maximum number of categories `k` the SIZE-008 block can
/// accommodate (P21 hard-defensive cap: any larger k is a domain error).
///
/// Derived from `STAT1_XSQEV_MAX_REG + 1` minus the `k`-slot (R00) and
/// divided by `STAT1_XSQEV_STRIDE` (2 slots per cell — O and E). With
/// SIZE 008 / R00..R07: (8 − 1) / 2 = 3 (integer division; the trailing
/// R07 scratch slot is the output and is not used as data).
pub const STAT1_XSQEV_KMAX: usize = 3;

/// ΣXSQEV / ΣEFXSQ: register slot the computed χ² result is written to
/// (in addition to being pushed onto stack X).
///
/// Per OM 00041-90030 p. 55 ("Result" slot), the χ² value lands in R07.
/// Today this coincides numerically with [`STAT1_XSQEV_MAX_REG`] — both
/// evaluate to 7 — but the two play distinct semantic roles:
///
/// - [`STAT1_XSQEV_MAX_REG`] is the SIZE-floor sentinel ("highest
///   0-indexed register slot the SIZE 008 block needs to address");
/// - `STAT1_XSQEV_RESULT_REG` is the OM-cited result-register address.
///
/// REVIEW.md CR-02 mitigation: the pre-fix code wrote the χ² value to
/// `state.regs[STAT1_XSQEV_MAX_REG]`, conflating SIZE-floor with
/// result-slot. Any future bump of `STAT1_XSQEV_MAX_REG` (e.g., raising
/// `STAT1_XSQEV_KMAX` to support more categories) would have silently
/// moved the result-write target, breaking the OM-faithful R07 contract
/// and any downstream consumer reading the result. The two constants
/// are now decoupled so a future bump cannot drift the result address.
pub const STAT1_XSQEV_RESULT_REG: usize = 7;

// ── Plan 33-06 Task 1: ΣMMTUG / ΣMMTGD per-slot register consts (P21) ──────
//
// ΣMMTUG (ungrouped) and ΣMMTGD (grouped / frequency-weighted) both use
// SIZE 012 (R00..R11) per OM 00041-90030 p. 15. The first six registers
// R01..R06 mirror the existing v1.x Σ-block (Σx²/Σx/n/Σy²/Σy/Σxy) so
// that the accumulator can delegate to `op_sigma_plus` for Σx, Σx², n
// updates and merely ADDS its own writes for the third + fourth moments
// (Σx³, Σx⁴). The new slots use R07 and R08 (within the SIZE 012 block);
// R09..R11 are program-internal scratch per OM.
//
//   R00 = (unused by accumulator)
//   R01 = Σx²            (v1.x via op_sigma_plus)
//   R02 = Σx             (v1.x via op_sigma_plus)
//   R03 = n              (v1.x via op_sigma_plus)
//   R04 = Σy² (unused by ΣMMTUG ungrouped path; ΣMMTGD reuses for Σ(f·x²))
//   R05 = Σy  (ΣMMTGD: Σf — total frequency)
//   R06 = Σxy (ΣMMTGD: Σ(f·x))
//   R07 = Σx³            (Plan 33-06 — third moment slot)
//   R08 = Σx⁴            (Plan 33-06 — fourth moment slot)
//   R09..R11 = scratch (intermediate results during compute)

/// ΣMMTUG / ΣMMTGD: register holding Σx³ (sum of cubes, OR sum of
/// frequency-weighted cubes `Σ(f·x³)` for ΣMMTGD).
///
/// OM 00041-90030 p. 15 (Moments, Skewness, Kurtosis); slot chosen as
/// R07 to land within the SIZE 012 block (R00..R11) without colliding
/// with the v1.x Σ-block delegated to `op_sigma_plus` (R01..R06).
pub const STAT1_MMTUG_CUBE_REG: usize = 7;

/// ΣMMTUG / ΣMMTGD: register holding Σx⁴ (sum of fourth powers, OR
/// frequency-weighted `Σ(f·x⁴)` for ΣMMTGD).
///
/// OM 00041-90030 p. 15; slot R08 within SIZE 012 block.
pub const STAT1_MMTUG_QUAD_REG: usize = 8;

// ── Plan 33-06 Task 2: ΣAOVONE per-group register consts (P21) ─────────────
//
// ΣAOVONE (one-way ANOVA) uses SIZE 020 (R00..R19) per OM 00041-90030
// p. 20. The block is organized as a single global accumulator block
// PLUS K per-group sub-blocks of 4 registers each. With SIZE 020 we
// can accommodate up to K = 4 groups:
//
//   R00 = k (group count; 1 ≤ k ≤ STAT1_AOV_KMAX = 4)
//   R01 = Σx²    (grand sum of squares; v1.x convention)
//   R02 = Σx     (grand sum)
//   R03 = N      (grand sample count)
//
//   Per-group block at base `STAT1_AOV_GROUP_BASE_REG + STAT1_AOV_GROUP_STRIDE * i`:
//     +0 = Σxᵢ      (group sum)
//     +1 = Σxᵢ²     (group sum of squares)
//     +2 = nᵢ       (group sample count)
//     +3 = scratch  (intermediate: x̄ᵢ or group SS during compute)
//
//   With base = 4 and stride = 4: groups 0..3 occupy R04..R19 = 16 regs.
//   Total: 4 (global) + 16 (per-group) = 20 = SIZE 020. ✓
//
// This shape mirrors the typical HP-41 Stat Pac ANOVA accumulator. The
// per-slot semantics are re-derived from OM p. 20 (Inputs section).

/// ΣAOVONE: register holding `k`, the number of groups
/// (1 ≤ k ≤ STAT1_AOV_KMAX = 4 for SIZE 020).
pub const STAT1_AOV_K_REG: usize = 0;

/// ΣAOVONE: register holding the grand sample count N = Σ nᵢ.
pub const STAT1_AOV_N_REG: usize = 3;

/// ΣAOVONE: register holding the grand sum Σx (across all groups).
pub const STAT1_AOV_GRAND_SUM_REG: usize = 2;

/// ΣAOVONE: register holding the grand sum-of-squares Σx² (across all groups).
pub const STAT1_AOV_GRAND_SUMSQ_REG: usize = 1;

/// ΣAOVONE: base register for the first per-group sub-block.
///
/// Group `i` (0-indexed) occupies registers
/// `STAT1_AOV_GROUP_BASE_REG + STAT1_AOV_GROUP_STRIDE * i` through
/// `STAT1_AOV_GROUP_BASE_REG + STAT1_AOV_GROUP_STRIDE * i + 3` (4 regs).
pub const STAT1_AOV_GROUP_BASE_REG: usize = 4;

/// ΣAOVONE: stride between successive per-group sub-blocks.
///
/// Each group occupies 4 consecutive registers (Σxᵢ, Σxᵢ², nᵢ, scratch);
/// stride = 4.
pub const STAT1_AOV_GROUP_STRIDE: usize = 4;

/// ΣAOVONE per-group offset: Σxᵢ (group sum).
pub const STAT1_AOV_GROUP_SUM_OFFSET: usize = 0;

/// ΣAOVONE per-group offset: Σxᵢ² (group sum of squares).
pub const STAT1_AOV_GROUP_SUMSQ_OFFSET: usize = 1;

/// ΣAOVONE per-group offset: nᵢ (group sample count).
pub const STAT1_AOV_GROUP_N_OFFSET: usize = 2;

/// ΣAOVONE: maximum number of groups `k` the SIZE 020 block can
/// accommodate. Derived from `(STAT1_AOVONE_MAX_REG + 1 −
/// STAT1_AOV_GROUP_BASE_REG) / STAT1_AOV_GROUP_STRIDE = 16 / 4 = 4`.
pub const STAT1_AOV_KMAX: usize = 4;

// ── REVIEW.md WR-01: ΣAOVTWO / ΣANOCOV per-slot register consts (P21) ──────
//
// ΣAOVTWO (Two-way ANOVA, No Replications) uses SIZE 018 (R00..R17) per
// OM 00041-90030 p. 23. The layout starts with two scalar dimensions
// (r, c), the cell count N, grand sums, then row + col marginal sums:
//
//   R00 = r (number of rows; 1 ≤ r ≤ STAT1_AOVTWO_DIM_MAX = 4)
//   R01 = c (number of columns; 1 ≤ c ≤ STAT1_AOVTWO_DIM_MAX = 4)
//   R02 = N = r·c (grand sample count)
//   R03 = grand Σx² (sum of squares across all cells)
//   R04 = grand Σx  (grand total)
//   R05.. R<5+r-1> = row marginal sums  (r consecutive slots)
//   R<5+r>..       = column marginal sums (c consecutive slots)
//
// ΣANOCOV (Analysis of Covariance, One Way) uses SIZE 026 (R00..R25)
// per OM 00041-90030 p. 28. It reuses [`STAT1_AOV_K_REG`] for the group
// count, shares R04 with ΣAOVTWO as the grand-Σx² covariate slot, then
// uses a per-group stride-5 block from R07 carrying `(Σy, Σy², n, Σx,
// Σxy)` for each group i ∈ [0..k]:
//
//   R04 = grand Σx² of the covariate (reused; OM-cited)
//   R07 = first per-group block; subsequent groups at +5 strides
//   Per-group offset:  0=Σy, 1=Σy², 2=n, 3=Σx, 4=Σxy

/// ΣAOVTWO: register holding `r`, the number of rows.
///
/// OM 00041-90030 p. 23 (Analysis of Variance, Two Way).
pub const STAT1_AOVTWO_R_REG: usize = 0;

/// ΣAOVTWO: register holding `c`, the number of columns.
///
/// OM 00041-90030 p. 23.
pub const STAT1_AOVTWO_C_REG: usize = 1;

/// ΣAOVTWO: register holding the grand sum of squares Σx².
///
/// OM 00041-90030 p. 23.
pub const STAT1_AOVTWO_GRAND_SUMSQ_REG: usize = 3;

/// ΣAOVTWO: register holding the grand sum Σx.
///
/// OM 00041-90030 p. 23.
pub const STAT1_AOVTWO_GRAND_SUM_REG: usize = 4;

/// ΣAOVTWO: base register of the first row-marginal sum.
///
/// Row `i` marginal sum sits at `STAT1_AOVTWO_ROW_BASE_REG + i` for
/// `0 ≤ i < r`. Column marginal sums follow at
/// `STAT1_AOVTWO_ROW_BASE_REG + r + j` for `0 ≤ j < c`.
///
/// OM 00041-90030 p. 23.
pub const STAT1_AOVTWO_ROW_BASE_REG: usize = 5;

/// ΣAOVTWO: maximum row/column dimension (`1 ≤ r, c ≤ DIM_MAX`).
///
/// Hard-defensive cap aligned with the SIZE 018 block; chosen 4 to
/// match the documented OM convention (sufficient for the canonical
/// 3×4 / 4×4 worked examples).
pub const STAT1_AOVTWO_DIM_MAX: usize = 4;

/// ΣANOCOV: register holding the grand sum-of-squares Σx² for the
/// covariate (reused R04 slot — same as `STAT1_AOVTWO_GRAND_SUM_REG`
/// numerically, distinct semantically).
///
/// OM 00041-90030 p. 28 (Analysis of Covariance, One Way).
pub const STAT1_ANOCOV_GRAND_SUMSQ_X_REG: usize = 4;

/// ΣANOCOV: base register of the first per-group block (k=0).
///
/// Group `i` block occupies registers
/// `STAT1_ANOCOV_GROUP_BASE_REG + STAT1_ANOCOV_GROUP_STRIDE * i`
/// through `..+4` (5 regs: Σy, Σy², n, Σx, Σxy).
///
/// OM 00041-90030 p. 28.
pub const STAT1_ANOCOV_GROUP_BASE_REG: usize = 7;

/// ΣANOCOV: stride between successive per-group blocks (5 regs each).
///
/// OM 00041-90030 p. 28.
pub const STAT1_ANOCOV_GROUP_STRIDE: usize = 5;

/// ΣANOCOV per-group offset: Σy (group sum of dependents).
pub const STAT1_ANOCOV_GROUP_SUM_Y_OFFSET: usize = 0;

/// ΣANOCOV per-group offset: Σy² (group sum of squared dependents).
pub const STAT1_ANOCOV_GROUP_SUMSQ_Y_OFFSET: usize = 1;

/// ΣANOCOV per-group offset: n (group sample count).
pub const STAT1_ANOCOV_GROUP_N_OFFSET: usize = 2;

/// ΣANOCOV per-group offset: Σx (group sum of covariate).
pub const STAT1_ANOCOV_GROUP_SUM_X_OFFSET: usize = 3;

/// ΣANOCOV per-group offset: Σxy (group sum of x·y cross-products).
pub const STAT1_ANOCOV_GROUP_SUM_XY_OFFSET: usize = 4;

// ── Plan 33-06 Task 3: ΣCTKKK / ΣCTKK per-slot register consts (P21) ───────
//
// ΣCTKKK (general r×c contingency) and ΣCTKK (smaller variant, 2×2 cap)
// both use SIZE 015 (R00..R14) per OM 00041-90030 p. 60. The block is
// organized as two scalar dimensions plus a packed cell matrix in
// row-major order:
//
//   R00 = r (number of rows; 1 ≤ r ≤ STAT1_CTKKK_DIM_MAX = 3 for ΣCTKKK,
//            1 ≤ r ≤ STAT1_CTKK_DIM_MAX = 2 for ΣCTKK)
//   R01 = c (number of columns; same per-Op cap as r)
//   R02..R<n+1> = O_ij cells, row-major (cell (i,j) at
//                  STAT1_CTKKK_CELL_BASE_REG + i*c + j)
//
//   For a 3×3 maximum table the cells occupy R02..R10 (9 cells); the
//   remaining R11..R14 are program-internal scratch (row/col marginals
//   and the χ² accumulator per OM p. 60).
//
// ΣCTKK is the smaller-table variant per OM p. 60 — capped at 2×2 (4
// cells, R02..R05) so the program-internal scratch overhead is reduced.
//
// Both variants share the same register layout; the only difference is
// the dimension cap enforced at decode time.

/// ΣCTKKK / ΣCTKK: register holding `r`, the number of rows.
pub const STAT1_CTKKK_R_REG: usize = 0;

/// ΣCTKKK / ΣCTKK: register holding `c`, the number of columns.
pub const STAT1_CTKKK_C_REG: usize = 1;

/// ΣCTKKK / ΣCTKK: base register for the first cell `O_{0,0}`.
///
/// Cell `(i, j)` is at `STAT1_CTKKK_CELL_BASE_REG + i * c + j` (row-major).
pub const STAT1_CTKKK_CELL_BASE_REG: usize = 2;

/// ΣCTKKK: maximum row/column dimension.
///
/// Derived from `(STAT1_CTKKK_MAX_REG + 1 − STAT1_CTKKK_CELL_BASE_REG) ≥ DIM²`.
/// With SIZE 015 / 13 cells available: floor(√13) = 3 (a 3×3 table fits;
/// 4×4 needs 16 cells but only 13 are available).
pub const STAT1_CTKKK_DIM_MAX: usize = 3;

/// ΣCTKK: maximum row/column dimension. Smaller-table variant per OM
/// p. 60 — capped at 2×2 (4 cells, R02..R05) to reduce scratch overhead.
pub const STAT1_CTKK_DIM_MAX: usize = 2;

// ── Plan 33-07 Task 2: ΣTSTAT per-group register consts (P21) ──────────────
//
// ΣTSTAT (pooled-variance two-sample t-test) uses SIZE 015 (R00..R14)
// per OM 00041-90030 p. 52. Layout decision for this plan:
//
//   - Group 1 reuses the v1.x R01–R03 slots (Σx², Σx, n) so a user who
//     has just finished an Σ+ accumulation for group 1 can pivot
//     directly to group 2 without copying data.
//   - Group 2 lives at R07–R09 (parallel layout one block past R04–R06,
//     which stays free for the user's own scratch or paired-y data).
//   - R10..R14 are program-internal scratch per OM (intermediate s²_p,
//     t, p, etc. — used by this Op via local HpNum vars, not stored).
//
//   R01 = Σx₁²   (Group 1 sum of squares)
//   R02 = Σx₁    (Group 1 sum)
//   R03 = n₁     (Group 1 sample count, integer)
//   R04..R06     (v1.x Σy² / Σy / Σxy block; UNUSED by ΣTSTAT — user scratch)
//   R07 = Σx₂²   (Group 2 sum of squares)
//   R08 = Σx₂    (Group 2 sum)
//   R09 = n₂     (Group 2 sample count, integer)
//   R10..R14 = scratch
//
// Welch's t-test is EXPLICITLY EXCLUDED per SPEC.md Req. 25 +
// REQUIREMENTS.md Out-of-Scope; only pooled variance ships.

/// ΣTSTAT: Group 1 Σx² register (reuses v1.x R01 slot per the layout
/// decision above so post-Σ+ pivot to group 2 is seamless).
///
/// OM 00041-90030 p. 52 (t Statistics).
pub const STAT1_TSTAT_G1_SUMSQ_REG: usize = 1;

/// ΣTSTAT: Group 1 Σx register (reuses v1.x R02 slot).
///
/// OM 00041-90030 p. 52.
pub const STAT1_TSTAT_G1_SUM_REG: usize = 2;

/// ΣTSTAT: Group 1 n register (reuses v1.x R03 slot; integer-valued).
///
/// OM 00041-90030 p. 52.
pub const STAT1_TSTAT_G1_N_REG: usize = 3;

/// ΣTSTAT: Group 2 Σx² register (R07; parallel to G1's R01).
///
/// OM 00041-90030 p. 52.
pub const STAT1_TSTAT_G2_SUMSQ_REG: usize = 7;

/// ΣTSTAT: Group 2 Σx register (R08; parallel to G1's R02).
///
/// OM 00041-90030 p. 52.
pub const STAT1_TSTAT_G2_SUM_REG: usize = 8;

/// ΣTSTAT: Group 2 n register (R09; parallel to G1's R03; integer-valued).
///
/// OM 00041-90030 p. 52.
pub const STAT1_TSTAT_G2_N_REG: usize = 9;

// ── Plan 33-08 Task 1: ΣMLRXY per-slot register consts (2-predictor MLR) ───
//
// ΣMLRXY (2-predictor multiple linear regression) uses SIZE 045 per
// OM 00041-90030 p. 40. With 2 predictors, the normal-equation system
// is 3×3 (intercept b₀ + 2 slope coefficients b₁, b₂). The Op reads
// 9 sufficient statistics:
//
//   R00 = n        (sample count)
//   R01 = Σy       (sum of dependent)
//   R02 = Σx₁      (sum of predictor 1)
//   R03 = Σx₂      (sum of predictor 2)
//   R04 = Σx₁²     (sum of squares of predictor 1)
//   R05 = Σx₂²     (sum of squares of predictor 2)
//   R06 = Σx₁x₂    (cross-product of predictors)
//   R07 = Σx₁y     (cross-product of predictor 1 with y)
//   R08 = Σx₂y     (cross-product of predictor 2 with y)

/// ΣMLRXY: register holding `n`, the sample count.
pub const STAT1_MLRXY_N_REG: usize = 0;
/// ΣMLRXY: register holding `Σy`.
pub const STAT1_MLRXY_SUM_Y_REG: usize = 1;
/// ΣMLRXY: register holding `Σx₁`.
pub const STAT1_MLRXY_SUM_X1_REG: usize = 2;
/// ΣMLRXY: register holding `Σx₂`.
pub const STAT1_MLRXY_SUM_X2_REG: usize = 3;
/// ΣMLRXY: register holding `Σx₁²`.
pub const STAT1_MLRXY_SUM_X1SQ_REG: usize = 4;
/// ΣMLRXY: register holding `Σx₂²`.
pub const STAT1_MLRXY_SUM_X2SQ_REG: usize = 5;
/// ΣMLRXY: register holding `Σx₁x₂`.
pub const STAT1_MLRXY_SUM_X1X2_REG: usize = 6;
/// ΣMLRXY: register holding `Σx₁y`.
pub const STAT1_MLRXY_SUM_X1Y_REG: usize = 7;
/// ΣMLRXY: register holding `Σx₂y`.
pub const STAT1_MLRXY_SUM_X2Y_REG: usize = 8;

// ── Plan 33-08 Task 1: ΣMLRXYZ per-slot register consts (3-predictor MLR) ──
//
// ΣMLRXYZ (3-predictor multiple linear regression) uses SIZE 045 per
// OM 00041-90030 p. 43. With 3 predictors, the normal-equation system
// is 4×4 (intercept b₀ + 3 slope coefficients b₁, b₂, b₃). The Op reads
// 14 sufficient statistics:
//
//   R00 = n          (sample count)
//   R01 = Σy
//   R02 = Σx₁     R03 = Σx₂     R04 = Σx₃
//   R05 = Σx₁²    R06 = Σx₂²    R07 = Σx₃²
//   R08 = Σx₁x₂   R09 = Σx₁x₃   R10 = Σx₂x₃
//   R11 = Σx₁y    R12 = Σx₂y    R13 = Σx₃y

/// ΣMLRXYZ: register holding `n`.
pub const STAT1_MLRXYZ_N_REG: usize = 0;
/// ΣMLRXYZ: register holding `Σy`.
pub const STAT1_MLRXYZ_SUM_Y_REG: usize = 1;
/// ΣMLRXYZ: register holding `Σx₁`.
pub const STAT1_MLRXYZ_SUM_X1_REG: usize = 2;
/// ΣMLRXYZ: register holding `Σx₂`.
pub const STAT1_MLRXYZ_SUM_X2_REG: usize = 3;
/// ΣMLRXYZ: register holding `Σx₃`.
pub const STAT1_MLRXYZ_SUM_X3_REG: usize = 4;
/// ΣMLRXYZ: register holding `Σx₁²`.
pub const STAT1_MLRXYZ_SUM_X1SQ_REG: usize = 5;
/// ΣMLRXYZ: register holding `Σx₂²`.
pub const STAT1_MLRXYZ_SUM_X2SQ_REG: usize = 6;
/// ΣMLRXYZ: register holding `Σx₃²`.
pub const STAT1_MLRXYZ_SUM_X3SQ_REG: usize = 7;
/// ΣMLRXYZ: register holding `Σx₁x₂`.
pub const STAT1_MLRXYZ_SUM_X1X2_REG: usize = 8;
/// ΣMLRXYZ: register holding `Σx₁x₃`.
pub const STAT1_MLRXYZ_SUM_X1X3_REG: usize = 9;
/// ΣMLRXYZ: register holding `Σx₂x₃`.
pub const STAT1_MLRXYZ_SUM_X2X3_REG: usize = 10;
/// ΣMLRXYZ: register holding `Σx₁y`.
pub const STAT1_MLRXYZ_SUM_X1Y_REG: usize = 11;
/// ΣMLRXYZ: register holding `Σx₂y`.
pub const STAT1_MLRXYZ_SUM_X2Y_REG: usize = 12;
/// ΣMLRXYZ: register holding `Σx₃y`.
pub const STAT1_MLRXYZ_SUM_X3Y_REG: usize = 13;

// ── Plan 33-08 Task 1: ΣPOLYP/ΣPOLYC per-slot register consts ──────────────
//
// ΣPOLYP (polynomial regression) and ΣPOLYC (Horner evaluation) use
// SIZE 045 per OM 00041-90030 p. 47-48. For degree d (1 ≤ d ≤ 5), the
// normal-equation system is (d+1)×(d+1). The Op needs Σx^k for
// k=1..2d (= up to 10 sums for d=5) and Σ(x^k · y) for k=0..d
// (= up to 6 sums for d=5), giving 16 input sums + n + degree slot
// + (d+1) ≤ 6 coefficient output slots = up to ~25 registers.
//
// Layout chosen to keep mechanical translation between Σ-block and
// polyp registers obvious:
//
//   R00          = n (sample count)
//   R01..R10     = Σx^k for k = 1..10 (10 slots; STAT1_POLYP_SUM_X_BASE_REG..)
//   R11..R16     = Σ(x^k · y) for k = 0..5 (6 slots; k=0 is Σy)
//   R20..R25     = a_0..a_5 (coefficient output, up to degree 5)
//   R30          = degree d (1 ≤ d ≤ 5)
//
// R17..R19 + R26..R29 + R31..R44 = scratch / unused (within SIZE 045).
// All slot consts are pinned per Plan 33-08; future plans extending
// polynomial-regression machinery (none currently planned) must respect
// these positions to preserve save/load layout compatibility.

/// ΣPOLYP: register holding `n` (sample count).
pub const STAT1_POLYP_N_REG: usize = 0;
/// ΣPOLYP: base register for `Σx^k` with k = 1 at offset 0, k = 2 at
/// offset 1, ..., k = 10 at offset 9 (10 slots total).
///
/// `Σx^k` for k ≥ 1 lives at `regs[STAT1_POLYP_SUM_X_BASE_REG + (k-1)]`.
pub const STAT1_POLYP_SUM_X_BASE_REG: usize = 1;
/// ΣPOLYP: base register for `Σ(x^k · y)` with k = 0 at offset 0,
/// k = 1 at offset 1, ..., k = 5 at offset 5 (6 slots total).
///
/// `Σ(x^k · y)` for k ≥ 0 lives at `regs[STAT1_POLYP_SUM_XY_BASE_REG + k]`.
/// (Offset 0 = Σy, offset 1 = Σxy, etc.)
pub const STAT1_POLYP_SUM_XY_BASE_REG: usize = 11;
/// ΣPOLYP/ΣPOLYC: base register for coefficient output.
///
/// `a_i` lives at `regs[STAT1_POLYP_COEF_BASE_REG + i]` for 0 ≤ i ≤ d.
/// ΣPOLYP writes; ΣPOLYC reads for Horner evaluation.
pub const STAT1_POLYP_COEF_BASE_REG: usize = 20;
/// ΣPOLYP: register holding the polynomial degree d (1 ≤ d ≤
/// STAT1_POLYP_DEGREE_MAX).
///
/// Written by `submit_step(Stat1Step::PolypDegreePrompt(_))` after user
/// submits via R/S; read by both ΣPOLYP (during compute) and ΣPOLYC
/// (during Horner eval).
pub const STAT1_POLYP_DEGREE_REG: usize = 30;
/// ΣPOLYP: hard-defensive cap on polynomial degree.
///
/// Chosen 5 to match the Math Pac I POLY precedent (DEGREE 2..=5) and
/// to keep the (d+1)×(d+1) Gauss elimination within reasonable LOC.
/// `d = 0` is degenerate (constant fit) and rejected as Domain.
pub const STAT1_POLYP_DEGREE_MAX: u8 = 5;

// Plan 33-08 final cleanup: the Plan 33-01 scaffolding Op variant + its
// helper function have been removed. All 26 STAT_1.ops entries point to
// real `Op::Sigma*` / `Op::Rand` / `Op::Seed` variants per the
// bidirectional consistency CI gate
// (`stat1_ops_mnemonics_resolve_consistently` in math1/xrom.rs tests).

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
