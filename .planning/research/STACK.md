# Technology Stack — v3.1 Stat 1 Pac

**Project:** HP-41 Calculator Emulator
**Researched:** 2026-05-21
**Scope:** NEW dependencies (or zero additions, justified) needed to add HP-41 Stat 1 Pac behavioral emulation on top of the validated v3.0 Math Pac I stack.

---

## Executive Summary

**Recommendation: ZERO new runtime dependencies in `hp41-core` for v3.1.**

The Stat 1 Pac (XROM 2, part 00041-14001, manual 00041-90030, June 1979) is FOCAL user-code stored in ROM — the same architectural pattern as Math Pac I. The QRC (00041-90061) was reviewed directly and lists 13 programs. The numerical surface introduced by these programs is:

1. **Normal distribution** — CDF and inverse CDF (NORMD). `rust_decimal 1.42` already provides `erf()` and `norm_cdf()` via `MathematicalOps`. The inverse normal (probit) is not in rust_decimal but is a 30-line hand-coded rational approximation (Acklam / AS 241 algorithm), the same f64-bridge pattern already used for `checked_asin`/`checked_acos`/`checked_atan`.

2. **Chi-square distribution** — CDF (CHISQD). Requires the regularized incomplete gamma function P(k/2, x/2). This is a 40–60 line hand-coded continued-fraction algorithm; rust_decimal has no `gamma_regularized` method. A rational-approximation or series/CF implementation using the existing f64-bridge is standard textbook (AS 239 / Numerical Recipes ch. 6).

3. **Student's t test** (PTSTAT). Requires the regularized incomplete beta function I_x(a, b), which reduces to the incomplete gamma for half-integer degrees of freedom. Same 40–60 line hand-coded treatment.

4. **Random numbers** (part of some programs, not an explicit named top-level program on the QRC). The HP-41 conventional RNG is the well-known multiplicative congruential `x_{n+1} = FRC(9821 * x_n + 0.211327)` stored in a user register. This is a 5-line implementation over existing HpNum arithmetic.

5. **Moments / skewness / kurtosis**, **ANOVA**, **curve fitting** (4 curve types: LIN/EXP/LOG/POW), **multiple regression**, **polynomial regression**, **contingency table**, **Spearman rank correlation** — all reduce to existing `Σ`-register arithmetic, linear algebra over the user register block (same pattern as Matrix Pac), and the existing SDEV / LR / CORR infrastructure already in `hp41-core/src/ops/stats.rs`.

The only statistical functions that require numerical primitives beyond what rust_decimal already provides are the normal CDF inverse and the regularized incomplete gamma/beta — both are well-understood 30–60 line implementations using the established f64 bridge. `statrs 0.18.0` was evaluated as a candidate for these functions but is rejected because: (a) its default features pull in `nalgebra 0.33` and `rand 0.8`, (b) even with `default-features = false` it forces `approx` into the normal dep graph (not dev-only), and (c) its interface works in f64 values rather than HpNum, requiring a conversion shim at every boundary that defeats its convenience.

---

## Verified Source Material

**HP-41C Stat Pac Quick Reference Card (00041-90061, June 1979) — read directly.**

All 13 programs confirmed from the QRC image:

| Program Name | QRC Mnemonic(s) | SIZE | Domain |
|---|---|---|---|
| Basic Statistics for Two Variables | ΣBSTAT / ΣBSTG | 012 | Extended σ summary |
| Moments, Skewness and Kurtosis | ΣMMTUG / ΣMMTGD | 012 | 3rd and 4th central moments |
| Analysis of Variance (One Way) | ΣAOVONE | 020 | F-ratio, group means |
| Analysis of Variance (Two Way, No Replications) | ΣAOVTWO | 018 | Row/column F-ratios |
| Analysis of Covariance (One Way) | ΣANOCOV | 026 | ANCOVA F-ratio |
| Curve Fitting | ΣLIN / ΣEXP / ΣLOGI / ΣPOW | 016 | 4 curve types, predict ŷ |
| Multiple Linear Regression | ΣMLRXY / ΣMLRXYZ | 045 | 2- and 3-variable; partial regression |
| Polynomial Regression | ΣPOLYP / ΣPOLYC | 045 | Polynomial fit, predict ŷ |
| t Statistics | ΣPTST / ΣTSTAT | 015 | One-sample and two-sample t |
| Chi-Square Evaluation | ΣXSQEV / ΣEFXSQ | 008 | Chi-square goodness-of-fit |
| Contingency Table | ΣCTKKK / ΣCTKK | 015 | r×c contingency chi-square |
| Spearman's Rank Correlation Coefficient | ΣSPEAR | 003 | Rank correlation |
| Normal and Inverse Normal Distribution | ΣNORMD | 019 | CDF Q(x), PDF f(x), inverse Q(x)→x |
| Chi-Square Distribution | ΣCHISQD | 007 | CDF f(x), P(x) |

**XROM module ID: 2** — confirmed via `calc.fjk.ch` HP-41 Module Database ("Statistics Pac 1B", XROM #2).

**Architecture: FOCAL user-code stored in ROM** — the "SIZE" column on the QRC gives program-step counts (012 = 12 steps), exactly the same proof pattern as Math Pac I. This is behavioral emulation of documented OM algorithms, not M-code (machine-code) function emulation.

---

## The Three Decisions Driving v3.1 Stack

### Decision 1: HpNum discipline is unchanged — no new runtime deps

Every Stat 1 op has the same signature shape as existing Math Pac I ops:

```rust
pub fn op_normd(state: &mut CalcState) -> Result<(), HpError>
pub fn op_chisqd(state: &mut CalcState) -> Result<(), HpError>
```

Inputs are consumed from `state.stack.x / .y / .z / .t` or user registers (the same `state.regs[N]` vector used by the existing `stats.rs`). Outputs go to `state.stack` via `enter_number()` + `apply_lift_effect()`.

**The HpNum f64-bridge pattern** (established in v1.0 for `checked_asin`, used throughout Math Pac I) applies to all distribution-function internals:

```rust
// pattern (from num.rs:184–192):
let v = self.0.to_f64().ok_or(HpError::Overflow)?;
// ... f64 computation ...
Decimal::from_f64(result)
    .map(HpNum::rounded)
    .ok_or(HpError::Overflow)
```

This gives ~15.9 significant digits of intermediate precision, rounded to 10 via `HpNum::rounded()`. This is the same precision model the HP-41 hardware used (56-bit BCD → 10-digit display). It is sufficient: the `numerical_accuracy.rs` harness already validates 10-sig-digit fidelity across 763 cases at 99.3% pass rate using this bridge.

### Decision 2: Normal CDF uses rust_decimal's built-in; inverse normal is hand-coded

**Normal CDF (`NORMD` forward direction):**

`rust_decimal 1.42` already provides `Decimal::norm_cdf()` via `MathematicalOps`. This computes `Φ(x) = (1 + erf(x / √2)) / 2` for the standard normal. For NORMD with arbitrary μ and σ, standardize first: `z = (x - μ) / σ`, then call `norm_cdf()`. Result: ZERO new deps for the forward CDF direction.

The QRC shows `NORMD` delivers: `Q(x) → x` (quantile lookup), `x → f(x)` (PDF), `x → Q(x)` (CDF). "Q(x)" in HP-41 terminology is the upper-tail probability `1 - Φ(x)`, matching `1.0 - norm_cdf()`.

**Inverse normal (probit, QRC: `Q(x) → x`):**

rust_decimal has no `norm_cdf_inverse` method. The `MathematicalOps` trait list (verified via docs.rs/rust_decimal/latest) contains: `exp`, `powi`, `powu`, `powf`, `powd`, `sqrt`, `ln`, `log10`, `erf`, `norm_cdf`, `norm_pdf`, `sin`, `cos`, `tan` — no inverse normal or erfc.

Hand-roll Acklam's rational approximation (AS 241: "Algorithm AS 241: The percentage points of the normal distribution") — a well-documented 30-line piece of code using two rational polynomials (degree-6 numerator / denominator pairs for central and tail regions). This is standard practice for calculator-class implementations. The HP-41 Stat Pac itself used a similar rational approximation per Abramowitz and Stegun §26.2.22 (the only approach available on 1979 hardware). Our implementation re-derives from the OM specification rather than copying any third-party code — Free42 GPL-contamination guard continues to apply.

Confidence: **HIGH** — algorithm is public-domain textbook content, multiple independent formulations exist.

### Decision 3: Chi-square CDF and t-distribution — hand-coded regularized incomplete gamma via f64

**Chi-square CDF (`CHISQD`):**

The chi-square CDF with k degrees of freedom is `P(k/2, x/2)` where P is the regularized lower incomplete gamma function. `rust_decimal` does not provide this. Options:

- **`statrs 0.18.0`** — provides `ChiSquared::cdf(x: f64) -> f64` via `ContinuousCDF` trait. REJECTED (see Alternatives Considered below).
- **Hand-coded regularized incomplete gamma** via the f64 bridge. Series expansion converges well for most practical values; continued fraction expansion handles the tail. Combined series + CF algorithm (AS 239 / Numerical Recipes §6.2): ~50 lines of Rust over f64. Result rounded to HpNum.

**t-distribution (PTSTAT):**

The t-distribution CDF uses the regularized incomplete beta function `I_x(a, b)` where `x = ν/(ν + t²)`, `a = ν/2`, `b = 1/2`. For the two-sided test, `p = 2 * I_x(ν/2, 1/2)`. Hand-coded via the continued fraction expansion for the regularized incomplete beta (AS 63, Numerical Recipes §6.4): ~60 lines.

**Why not `statrs 0.18.0`:**

statrs 0.18.0 (released 2024-12-03, MSRV 1.65) covers all needed distributions: `Normal`, `StudentsT`, `ChiSquared`, `FisherSnedecor`, all implementing `ContinuousCDF<f64, f64>` with `.cdf()` and `.inverse_cdf()`. It is a well-maintained library with correct algorithms.

However, the cost–benefit does not justify it for this calculator emulator:

1. **Default features include nalgebra 0.33 and rand 0.8.** To avoid those, you must declare `statrs = { version = "0.18", default-features = false }`. Even then, `approx 0.5.0` is a MANDATORY (non-optional) dependency of statrs — it would land in hp41-core's production dep graph, not just dev-deps.

2. **Interface works in f64, not HpNum.** Every call site needs:
   ```rust
   let chi = ChiSquared::new(df.0.to_f64().unwrap_or(0.0))
       .map_err(|_| HpError::Domain)?;
   let p = chi.cdf(x.0.to_f64().ok_or(HpError::Overflow)?);
   Decimal::from_f64(p).map(HpNum::rounded).ok_or(HpError::Overflow)
   ```
   That conversion shim is as long as the statrs call itself. The hand-coded gamma/beta function in the same f64 bridge is equivalent effort and produces a thinner dep graph.

3. **Binary size.** `approx 0.5.0` + `num-traits 0.2.14` as mandatory runtime deps versus zero new deps — not catastrophic but unnecessary for a calculator that already has a validated numerical bridge.

4. **Algorithmic fidelity.** The HP-41 Stat Pac CHISQD and PTSTAT programs compute specific approximations (Abramowitz and Stegun, 1979-era textbook algorithms). Re-deriving from the OM spec yields outputs that match the hardware. Using statrs's modern Lanczos-gamma + Halley-iterate inverse CDF may produce slightly different last-digit results that fail the `numerical_accuracy.rs` gate.

Confidence: **HIGH** — this mirrors the exact reasoning used for Math Pac I (hand-coded matrix Gauss-Jordan, polynomial root-finder, Simpson integrator) and was validated by 763 accuracy cases.

---

## What rust_decimal 1.42 Already Covers for Stat 1

Verified from `MathematicalOps` trait (docs.rs, 2026-05-21):

| Statistical Need | rust_decimal method | Coverage |
|---|---|---|
| Normal PDF | `norm_pdf()` | FULL — direct use |
| Normal CDF Φ(x) | `norm_cdf()` | FULL — direct use |
| Error function erf(x) | `erf()` | FULL — building block |
| Square root (for σ computations) | `sqrt()` | FULL |
| Natural log (for LOG curve fit, log-likelihood) | `ln()` | FULL |
| Log base 10 (for LOG10 curve fit) | `log10()` | FULL |
| Exponential (for EXP curve fit, e^x) | `exp()` | FULL |
| Power y^x (for POW curve fit) | `powd()` | FULL |
| Trig (used in some ANOVA F-table programs) | `sin`, `cos`, `tan` | FULL |

**NOT available in rust_decimal (require hand-coded f64-bridge):**

| Statistical Need | Missing from rust_decimal | Solution |
|---|---|---|
| Inverse normal / probit Φ⁻¹(p) | not in MathematicalOps | Hand-code Acklam/AS 241 (30 lines) |
| Regularized incomplete gamma P(a, x) | not in MathematicalOps | Hand-code series+CF (50 lines) |
| Regularized incomplete beta I_x(a, b) | not in MathematicalOps | Hand-code continued fraction (60 lines) |
| Complementary erf `erfc(x)` | not in MathematicalOps | `1.0 - erf(x)` or hand-code |

---

## XROM Framework: Bit 1 Extension, ZERO New Deps

The existing `xrom_resolve()` in `hp41-core/src/ops/math1/xrom.rs` already has the placeholder:

```rust
// Future v3.1+ modules go here:
// if modules & 0b0000_0010 != 0 { stat1_resolve(name) }
```

v3.1 activates bit 1. The `modules: u8` field on `CalcState` (already present with `#[serde(default)]`) gains a second used bit. No new fields, no new structs — only:

1. `pub const STAT_1: XromModule = XromModule { id: 2, name: "STAT 1B", ops: &[...] };`
2. `fn stat1_resolve(name: &str) -> Option<Op>` match block
3. Uncommenting the `if modules & 0b0000_0010 != 0` arm in `xrom_resolve()`
4. A third `OnceLock<Vec<HelpEntry>>` in `hp41-cli/src/help_data.rs` loaded from `docs/hp41-stat1-functions.json`

The 4-way exhaustive-match invariant (CLAUDE.md Frozen Invariants) applies: every new `Op` variant lands simultaneously in `dispatch()`, `execute_op()`, both `op_display_name()` copies, before any caller compiles.

---

## Random Number Generation

The QRC does not list a standalone `RAND` program, but statistical sampling programs in the Stat Pac (e.g., random sampling from distributions) may use the conventional HP-41 RNG formula. Based on community research (hpmuseum.org, hp41programs.yolasite.com):

**HP-41 standard RNG:** `x_{n+1} = FRC(9821 * x_n + 0.211327)`

This is a 5-line implementation over existing HpNum arithmetic:
```rust
// Using HpNum: multiply by 9821, add 0.211327, take fractional part
let seed = state.regs[seed_reg].checked_mul(&HpNum::from(9821i32))?
    .checked_add(&HpNum::from_str("0.211327")?)?;
let new_seed = HpNum::from(seed.0.fract()); // FRC = fractional part
```

Seed is stored in a designated user register (per program convention). **Confidence: MEDIUM** — the formula is widely cited for HP-41 programs; the specific register location used by the Stat Pac programs requires OM verification during implementation.

No `rand` crate needed. The HP-41 RNG is deterministic and BCD-based by design.

---

## Moments, Skewness, Kurtosis — Welford vs. Two-Pass

The Stat Pac `ΣMMTUG`/`ΣMMTGD` programs compute skewness and kurtosis. On the HP-41 hardware, these accumulate into user registers just like `Σ+` accumulates into R01–R06. The programs maintain higher-order sums (Σx³, Σx⁴) in additional registers.

**No Welford online algorithm is needed.** The HP-41 accumulator pattern is the two-pass (or register-accumulation) approach: `Σx`, `Σx²`, `Σx³`, `Σx⁴` are accumulated directly. Skewness and kurtosis are computed from those sums at the `E` (display result) step. This is the hardware-faithful approach and matches the program architecture visible from the QRC (separate init / input / result phases).

**Welford's online algorithm** (numerically stable variance) is what you'd choose when building a new statistical library. We are emulating a 1979 FOCAL program that accumulates raw power sums. Using Welford would produce subtly different intermediate-step values and fail behavioral fidelity. **Do not use Welford.**

**Numerical stability of raw power sums:** For data sets with large offsets, the `n*Σx² - (Σx)²` formula (already used in `op_sdev`) can lose precision. This is a known limitation of the original HP-41 hardware behavior. Our emulation should match the hardware, including this limitation, except where the OM explicitly documents a corrected formula.

---

## Alternatives Considered

| Category | Candidate | Version | MSRV | Decision | Reason |
|---|---|---|---|---|---|
| Distribution functions | `statrs` | 0.18.0 | 1.65 | REJECT (runtime) | `approx` mandatory dep in production graph; f64 interface requires conversion shim; algorithmic divergence risk for OM fidelity; default features pull nalgebra + rand |
| Distribution functions | `puruspe` | 0.3.0 | unknown | REJECT | Small crate, limited vetting; incomplete gamma only; same f64-bridge need |
| Distribution functions | `scilib` | 0.8.x | unknown | REJECT | Broad science library (physics constants, astronomy, etc.); same footprint problem as peroxide in v3.0 |
| RNG | `rand` | 0.9.0 | 1.65 | REJECT | HP-41 RNG is a 5-line BCD formula, not a cryptographic or statistical-quality generator; importing rand for `FRC(9821*x + 0.211327)` is nonsensical |
| RNG | `rand_distr` | 0.5.1 | 1.60 | REJECT | Same reasoning as rand |
| Numerical stability | custom Welford | — | — | REJECT (behavioral) | Hardware used raw power sums; Welford changes intermediate outputs and fails OM fidelity |
| Test assertions | `approx` | 0.5.1 | 1.36 | ACCEPT (dev-dep only) | Already in hp41-core dev-deps (Cargo.toml line 17); no addition needed |

---

## Dependency Delta for v3.1

### Runtime additions to `hp41-core` [dependencies]: ZERO

```toml
# hp41-core/Cargo.toml [dependencies] — UNCHANGED from v3.0
rust_decimal = { workspace = true, features = ["maths", "serde-with-str"] }
thiserror = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

### Dev-dependency additions: ZERO

`approx 0.5.1` is already present in `hp41-core/Cargo.toml [dev-dependencies]` (line 17). No addition needed.

### Workspace root Cargo.toml: UNCHANGED

```toml
# [workspace.dependencies] — same as v3.0
rust_decimal = "1.42"
thiserror = "2.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### CLI additions: ZERO

Third JSON help source (`docs/hp41-stat1-functions.json`) loaded via existing `include_str!` + `OnceLock` pattern in `hp41-cli/src/help_data.rs`. No new crates.

### GUI additions: ZERO

CATALOG 2 XROM enumeration extension and Help-Overlay third section follow the v3.0 GUI Phase 31 pattern. No new crates.

---

## New Modules in hp41-core/src/ops/ for v3.1

| Module | Purpose | Key functions | LOC estimate |
|---|---|---|---|
| `ops/stat1/mod.rs` | STAT_1 XromModule const, stat1_resolve(), xrom_resolve() bit-1 arm | ~30 | 30 |
| `ops/stat1/distributions.rs` | `norm_cdf_inv_f64()` (Acklam AS 241), `gamma_regularized_f64()` (AS 239), `beta_regularized_f64()` (AS 63) | ~140 | 140 |
| `ops/stat1/basic_stats.rs` | BSTAT (extended summary), MMTUG/MMTGD (moments/skewness/kurtosis) | ~80 | 80 |
| `ops/stat1/anova.rs` | AOVONE, AOVTWO, ANOCOV | ~150 | 150 |
| `ops/stat1/regression.rs` | ΣLIN/ΣEXP/ΣLOGI/ΣPOW (curve fit), ΣMLRXY/ΣMLRXYZ (multiple), ΣPOLYP/ΣPOLYC (polynomial) | ~200 | 200 |
| `ops/stat1/tests.rs` | PTST/TSTAT (t-test), XSQEV/EFXSQ (chi-square), CTKKK/CTKK (contingency) | ~120 | 120 |
| `ops/stat1/nonparam.rs` | SPEAR (Spearman rank) | ~40 | 40 |
| `ops/stat1/normd.rs` | NORMD (normal distribution CDF/PDF/inverse) | ~50 | 50 |
| `ops/stat1/chisqd.rs` | CHISQD (chi-square distribution CDF/PDF) | ~40 | 40 |

**Total new LOC estimate:** ~850 lines across 9 files. This is narrower than Math Pac I's scope (v3.0 had ~1,500–2,000 LOC across math1/), consistent with Stat 1 Pac's simpler workflow structure (no complex numbers, no matrix algebra, no ODE solver — all programs build on the existing Σ-register infrastructure).

---

## MSRV Impact: NONE

| Crate | MSRV | Project MSRV | Status |
|---|---|---|---|
| `rust_decimal` 1.42 | 1.85 | 1.88 | Compatible, already pinned |
| `approx` 0.5.1 (dev-dep) | 1.36 | 1.88 | Compatible, already present |
| Hand-coded stat modules | n/a | 1.88 | Stable Rust only |

Workspace MSRV 1.88 (declared at `[workspace.package]`) is unchanged.

---

## Zero-Panic Compatibility Audit

`#![deny(clippy::unwrap_used)]` in `hp41-core/src/lib.rs` continues to apply.

| New module | Panic risk | Mitigation |
|---|---|---|
| `stat1/mod.rs` | None — pure static lookup | n/a |
| `stat1/distributions.rs` | `to_f64()` returns `None` on extreme Decimal values | `.ok_or(HpError::Overflow)?` at every bridge boundary |
| `stat1/distributions.rs` | `gamma_regularized_f64` non-convergence (malformed inputs) | Iteration cap → `HpError::Domain` |
| `stat1/distributions.rs` | `norm_cdf_inv_f64` with p ≤ 0 or p ≥ 1 | Guard at entry → `HpError::Domain` |
| `stat1/regression.rs` | Singular normal equations (ill-conditioned X'X matrix) | Pivot threshold guard → `HpError::Domain` ("DATA ERROR" per OM) |
| `stat1/tests.rs` | n = 0 or n = 1 in t-test (denominator zero) | Guard at entry → `HpError::InvalidOp` |
| `stat1/nonparam.rs` | Non-integer or tied ranks | Tie-correction implemented as floating-point average → no panic |
| `approx` (dev-only) | `assert_relative_eq!` panics on inequality | Confined to `#[cfg(test)]` with `#![allow(clippy::unwrap_used)]` |

---

## Coverage Implications

v3.0 baseline: 95.39 % lines / 94.26 % regions. Gate: ≥ 95 % lines / ≥ 93 % regions.

~850 new LOC requires ~90–110 new test cases to maintain 95 % line coverage. Plan:
- `hp41-core/tests/stat1_distributions.rs` — normal CDF, inverse normal, chi-square CDF, t-distribution CDF: ~30 cases
- `hp41-core/tests/stat1_regression.rs` — curve fit (4 types), multiple regression: ~20 cases
- `hp41-core/tests/stat1_tests_and_anova.rs` — t-test, chi-square, ANOVA: ~25 cases
- `hp41-core/tests/stat1_nonparam.rs` — Spearman, moments: ~15 cases
- `numerical_accuracy.rs` extension — distribution function ground-truth cases (Python scipy.stats as oracle for forward-direction verification): ~30 additional cases

**Oracle selection:** scipy.stats (Python) for distribution CDFs. For inverse normal: `scipy.stats.norm.ppf()`. For chi-square CDF: `scipy.stats.chi2.cdf()`. For t-test: `scipy.stats.t.cdf()`. These are industry-standard references. The Free42 contamination guard does NOT apply to scipy.stats — it is a Python library with BSD license, not a Free42/HP GPL-adjacent codebase.

---

## Free42 GPL-Contamination Guard Extension

The existing `scripts/check-free42-contamination.sh` 12-symbol grep gate (`just license-audit` + `ci.yml::license-audit` job) requires no changes for Stat 1 Pac.

The statistical algorithms used (Acklam/AS 241, AS 239, AS 63) are standard 1970s–1980s textbook algorithms published in Applied Statistics (Royal Statistical Society), all public domain. They share no identifying symbols with Free42, Intel BID, decNumber, or GPL/AGPL codebases.

New distinctive identifiers to consider **excluding** from the contamination pattern (i.e., should NOT be flagged): `gamma_regularized`, `beta_regularized`, `norm_cdf_inv`, `acklam`, `as241`. These are generic algorithm names, not Free42-specific. No change to the grep pattern needed.

---

## Integration Checklist for Roadmap

These items flow from stack decisions to specific phase requirements:

- [ ] `xrom_resolve()` bit-1 arm: uncomment and extend with `stat1_resolve(name)`
- [ ] `STAT_1: XromModule` const with id=2, name="STAT 1B", ops=&[all 13+ mnemonics]
- [ ] `CalcState.modules` bit-1 semantics documented (bit 0 = MATH_1, bit 1 = STAT_1)
- [ ] `distributions.rs`: `norm_cdf_inv_f64(p)`, `gamma_regularized_f64(a, x)`, `beta_regularized_f64(x, a, b)` — all internal f64, no HpNum in signature
- [ ] Each public `op_*` function wraps the f64 bridge with HpNum I/O per the checked_asin pattern
- [ ] `docs/hp41-stat1-functions.json` canonical source file created before CLI/GUI integration
- [ ] Third `OnceLock` in `help_data.rs` with `STAT1_HELP_ENTRIES` merged into `help_entries_all()`
- [ ] `scripts/docs-matrix` extended to accept three JSON inputs (already two-input from v3.0)
- [ ] `xrom_shadowing.rs` test extended with STAT_1.ops entries
- [ ] `numerical_accuracy.rs` extended with Stat 1 distribution test cases (scipy.stats oracle)

---

## Sources

- HP-41C Stat Pac Quick Reference Card (00041-90061, June 1979) — read directly as PDF via literature.hpcalc.org; all 13 programs verified from QRC image
- HP-41 Module Database (calc.fjk.ch): XROM #2 for "Statistics Pac 1B" — confirmed
- `rust_decimal::MathematicalOps` trait: docs.rs/rust_decimal/latest/rust_decimal/trait.MathematicalOps.html — full method list verified 2026-05-21; no `norm_cdf_inverse`, `gamma_regularized`, or `beta_regularized`
- `statrs 0.18.0` Cargo.toml: docs.rs/crate/statrs/0.18.0/source/Cargo.toml — MSRV 1.65, mandatory `approx 0.5.0` dep, optional `nalgebra 0.33` + `rand 0.8`, default features include both optional deps
- HP-41 RNG formula `FRC(9821*x + 0.211327)`: multiple independent sources (hpcalc.org/hp48/docs/misc/rand.txt, HP Museum forum discussions)
- Acklam's inverse normal algorithm: stackedboxes.org/2017/05/01/acklams-normal-quantile-function/ (public-domain, based on AS 241)
- AS 239 (incomplete gamma): statrs source code reference + Numerical Recipes §6.2 (standard textbook)
- AS 63 (incomplete beta): statrs source code reference + Numerical Recipes §6.4 (standard textbook)
- `hp41-core/Cargo.toml` lines 1–22 — current dep list (4 runtime, 4 dev); `approx 0.5.1` already at line 17
- `hp41-core/src/num.rs` lines 184–211 — established f64-bridge pattern (`checked_asin`, `checked_acos`, `checked_atan`)
- `hp41-core/src/ops/math1/xrom.rs` lines 127–136 — existing `xrom_resolve()` with bit-1 placeholder comment
- `hp41-core/src/ops/stats.rs` — existing Σ register infrastructure (MEAN, SDEV, L.R., CORR, YHAT) that Stat 1 programs build on
- `.planning/research/STACK.md` (v3.0, 2026-05-16) — confirmed all v3.0 crate rejection rationale remains valid for v3.1
