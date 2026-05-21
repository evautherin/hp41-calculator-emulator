# Feature Landscape: HP-41 Stat 1 Pac Emulation (v3.1)

**Domain:** Behavioral emulation of the HP-41C Stat 1 Pac plug-in application module
**Researched:** 2026-05-21
**Scope:** v3.1 — Stat 1 Pac (HP part 00041-14001, OM 00041-90030, QRC 00041-90061, June 1979) only.
Time Pac / Advanced Matrix / Advantage are deferred to v3.2+.

**Primary sources:**
- HP-41C Stat Pac Quick Reference Card 00041-90061 (June 1979) — read directly as 2-page PDF image. All 13 programs, their initialization mnemonics, input/correction/results columns confirmed from the QRC image.
- Naval Postgraduate School report NPS55-84-003 "HP-41C Programs and Instructions for Probability and Statistics" by Peter W. Zehna (February 1984, DTIC AD-A140573) — confirms STAT PAC function usage, RNG formula, ΣNORMD / ΣCHISQD workflow, ΣBSTG initialization display, t-distribution degrees-of-freedom entry.
- STACK.md and ARCHITECTURE.md from this research cycle (2026-05-21) — confirmed 13 programs and XROM ID 2.

---

## Authoritative Program List (from QRC 00041-90061)

The QRC lists exactly **13 programs** in two table pages. The table below transcribes every program with its initialization mnemonic(s), program-step count (SIZE column on QRC), and the initialization / input / correction / results columns as read directly from the QRC image.

| # | Program Name | Init Mnemonic(s) | SIZE | Domain |
|---|---|---|---|---|
| 1 | Basic Statistics for Two Variables | ΣBSTAT, ΣBSTG | 012 | Extended univariate + bivariate summary |
| 2 | Moments, Skewness and Kurtosis | ΣMMTUG, ΣMMTGD | 012 | 3rd and 4th central moments |
| 3 | Analysis of Variance (One Way) | ΣAOVONE | 020 | One-way ANOVA F-ratio, group means |
| 4 | Analysis of Variance (Two Way, No Replications) | ΣAOVTWO | 018 | Two-way ANOVA row/column F-ratios |
| 5 | Analysis of Covariance (One Way) | ΣANOCOV | 026 | One-way ANCOVA F-ratio |
| 6 | Curve Fitting | ΣLIN, ΣEXP, ΣLOGI, ΣPOW | 016 | 4 curve types; predict ŷ |
| 7 | Multiple Linear Regression | ΣMLRXY, ΣMLRXYZ | 045 | 2-variable and 3-variable; partial regression |
| 8 | Polynomial Regression | ΣPOLYP, ΣPOLYC | 045 | Polynomial fit; predict ŷ |
| 9 | t Statistics | ΣPTST, ΣTSTAT | 015 | One-sample and two-sample t-tests |
| 10 | Chi-Square Evaluation | ΣXSQEV, ΣEFXSQ | 008 | Chi-square goodness-of-fit |
| 11 | Contingency Table | ΣCTKKK, ΣCTKK | 015 | r×c contingency chi-square |
| 12 | Spearman's Rank Correlation Coefficient | ΣSPEAR | 003 | Rank correlation |
| 13 | Normal and Inverse Normal Distribution | ΣNORMD | 019 | CDF Q(x), PDF f(x), inverse Q(x)→x |
| 14 | Chi-Square Distribution | ΣCHISQD | 007 | CDF f(x) and P(x) |

**Note:** The QRC lists 13 programs but ΣCHISQD is on a separate row on page 2 of the QRC — the Stat Pac delivers 14 named entry points across 13 conceptual programs (Chi-Square Evaluation and Chi-Square Distribution are distinct: ΣXSQEV/ΣEFXSQ compute a test statistic from observed/expected counts, while ΣCHISQD evaluates the chi-square CDF at a given x with ν degrees of freedom).

---

## (a) Top-Level XEQ-by-Name Programs — Prompts and Workflow

### Program 1: Basic Statistics for Two Variables (ΣBSTAT / ΣBSTG)
**SIZE:** 012 steps

**What it computes:** Extended summary statistics beyond the built-in MEAN/SDEV/L.R. — provides weighted mean, standard error, coefficient of variation, or bivariate summary (correlation coefficient, regression coefficients) drawn from the Σ-register block (R01–R06). On the QRC the initialization column shows two separate entry points:

- **ΣBSTAT** — Initialization for ungrouped two-variable data (the NPS document calls this form `ΣBSTG` for grouped data; both share the same 12-step program). The QRC "Initialization" column shows `XEQ ΣBSTAT` followed by `XEQ ΣBSTG` as two distinct paths.
- Data input column: `x_i [ENTER+] y_i [A]` and `x_i [ENTER+] y_i f_i [A]` (the f_i form is the grouped/frequency-weighted variant).
- Correction: `x_k [ENTER+] y_k [C]` and `x_k [ENTER+] y_k f_k [C]`.
- Results: `[E], [R/S], ...` (sequential display of multiple statistics).
- Re-initialization: orange key then [A].

**ALPHA prompts visible from QRC:** None — all input is via stack (ENTER+) and label key [A]. The NPS document (p. 36) confirms initialization displays `ΣBSTG` on the LCD (the module name flash, not an ALPHA prompt for user input).

**Dependencies:** Uses existing Σ-registers R01–R06 (already in `hp41-core/src/ops/stats.rs`). Reads `Σx`, `Σx²`, `n`, `Σy²`, `Σy`, `Σxy` accumulated by prior `Σ+` calls.

**Algorithmic complexity:** Register-accumulating (closed-form from Σ-registers). Complexity: LOW — all computations are arithmetic over six already-accumulated register values. No iteration.

---

### Program 2: Moments, Skewness and Kurtosis (ΣMMTUG / ΣMMTGD)
**SIZE:** 012 steps

**What it computes:** Third and fourth central moments; skewness (γ₁ = μ₃/σ³) and kurtosis (γ₂ = μ₄/σ⁴ − 3). The QRC shows two entry points:

- **ΣMMTUG** — ungrouped data variant
- **ΣMMTGD** — grouped data variant (frequency-weighted)

Data input column: `x_i [A]` for ungrouped; `y_i [ENTER+] f_i [A]` for grouped.
Correction: `x_k [C]` for ungrouped; `y_k [ENTER+] f_k [C]` for grouped.
Results: `[E], [R/S], [R/S], [R/S], [R/S], [R/S]` — sequential display of mean, variance, skewness, kurtosis (multiple R/S presses).

**ALPHA prompts:** None visible on QRC — input via stack and [A] label key.

**Data storage:** The program accumulates Σx, Σx², Σx³, Σx⁴ into registers beyond R01–R06. The OM "Storage Registers" section (not read directly — requires Phase 33 OM verification) specifies the exact register layout. ARCHITECTURE.md notes these are likely R07–R12 or similar extension beyond the built-in Σ-block. This must be verified before implementing (Pitfall 21).

**Dependencies:** Extends (does NOT replace) the built-in Σ-block. Uses `rust_decimal`'s existing `pow()` / multiplication for the power-sum accumulation.

**Algorithmic complexity:** Register-accumulating (accumulates Σx³, Σx⁴ in parallel with Σ+ calls). Final computation: closed-form from four accumulated sums. Complexity: MEDIUM — requires extending the register layout beyond R01–R06 but no iteration. The variance cancellation pitfall (Pitfall 18) applies: the raw-power-sum formula `(n·Σx² − (Σx)²) / (n·(n−1))` is adequate because data is accumulated one-step-at-a-time into HP BCD registers (not computed in bulk from a register block), so each accumulation step is exact.

---

### Program 3: Analysis of Variance — One Way (ΣAOVONE)
**SIZE:** 020 steps

**What it computes:** One-way ANOVA: computes the F-ratio and associated group means. The user enters group data one value at a time, with group membership encoded by the ALPHA register.

Data input column: `x_{ij} [A], [R/S], [R/S], [R/S]` — three R/S presses per observation suggest the program displays intermediate counts or group tallies between entries.
Correction: `x_{im} [C]`.
Results: `[E], [R/S], ...` — F-ratio and group means displayed sequentially.
Re-initialization: orange key then [A].

**ALPHA prompts:** The QRC initialization column shows `XEQ ΣAOVONE` with no explicit ALPHA prompt text. However, the three-R/S-per-entry pattern implies the program displays group-count progress (similar to the built-in Σ+ which shows n). OM verification required for exact prompt text.

**Dependencies:** Uses Σ-registers for between-group and within-group sum accumulation. Likely uses registers beyond R06 for group counts and sums-of-squares. OM register table verification required (Pitfall 21).

**Algorithmic complexity:** Register-accumulating (multi-group accumulation). Final step: closed-form F = MSB/MSW. Complexity: MEDIUM — the data entry loop is O(N) where N is total observations; F computation is closed-form from accumulated sums.

---

### Program 4: Analysis of Variance — Two Way, No Replications (ΣAOVTWO)
**SIZE:** 018 steps

**What it computes:** Two-way ANOVA without replication. Computes row F-ratio and column F-ratio for a two-way layout x_{ij} where i = row, j = column.

Data input:
- Row-wise entry: `x_{ij} [A], [R/S], [R/S]`
- Column-wise entry: `x_{kj} [C]` (QRC shows separate row and column entry paths)

Results: `[E], [R/S], ...`
Re-initialization: orange key then [A].

**ALPHA prompts:** QRC shows `Row-wise: x_{ij} [A], [R/S], [R/S]` and `Column-wise: x_{kj} [C]`. No explicit ALPHA prompt strings visible. OM verification required.

**Dependencies:** Requires the user to organize data in row × column layout before entry. Uses Σ-registers and extended registers for row/column sum accumulation.

**Algorithmic complexity:** Register-accumulating. Closed-form F computation from row/column/total sums. Complexity: MEDIUM.

---

### Program 5: Analysis of Covariance — One Way (ΣANOCOV)
**SIZE:** 026 steps

**What it computes:** One-way ANCOVA. Adjusts group means for a covariate and computes the ANCOVA F-ratio. This is the most complex ANOVA-family program (largest SIZE at 26 steps vs. 18–20 for the others).

Data input column: `x_{ij} [ENTER+] y_{ij} [A], [R/S], [R/S], [R/S], [R/S]` — four R/S presses after each (x,y) pair entry.
Correction: `x_{im} [ENTER+] y_{im} [C]`.
Results: `[E], [R/S], ...`.

**ALPHA prompts:** No explicit ALPHA prompt strings on QRC.

**Dependencies:** Uses paired (x, y) data entry — covariate x and response y. Needs registers for group x-sums, y-sums, xy-sums, x²-sums beyond R06.

**Algorithmic complexity:** Register-accumulating then closed-form ANCOVA computation. Complexity: MEDIUM-HIGH (more involved than one-way ANOVA because of covariate adjustment; 26 steps vs. 20 suggests additional computation in the results phase).

---

### Program 6: Curve Fitting (ΣLIN / ΣEXP / ΣLOGI / ΣPOW)
**SIZE:** 016 steps

**What it computes:** Fits four curve models to (x, y) data using least-squares linear regression applied to transformed variables. The four entry points correspond to the four transformations:

| Mnemonic | Model | Transformation |
|---|---|---|
| **ΣLIN** | Linear: ŷ = a + bx | No transformation (direct Σ+ of x, y) |
| **ΣEXP** | Exponential: ŷ = ae^(bx) | Accumulate (x, ln y) into Σ-registers |
| **ΣLOGI** | Logarithmic: ŷ = a + b·ln(x) | Accumulate (ln x, y) into Σ-registers |
| **ΣPOW** | Power: ŷ = ax^b | Accumulate (ln x, ln y) into Σ-registers |

Data input: `x_i [ENTER+] y_i [A]` — one mnemonic initializes, data entered via ENTER+/[A].
Correction: `x_k [ENTER+] y_k [C]`.
Results: `[E], [R/S]` for coefficients a, b; then `x [R/S] → ŷ` for prediction.

**ALPHA prompts:** None visible on QRC — data entered via stack protocol.

**Dependencies:** All four variants delegate to the existing Σ-register infrastructure (R01–R06) after applying the coordinate transformation. The transformation calls `checked_ln()` (already in `hp41-core/src/num.rs`) on x and/or y before calling `op_sigma_plus()`. After fitting, prediction uses the existing `op_yhat()` mechanism on the transformed scale, then back-transforms. The ARCHITECTURE.md Pattern 3 exactly describes this delegate pattern.

**ALPHA-prompt mnemonic collision alert:** `ΣLIN` → The regression is linear in transformed space. The built-in `L.R.` / `YHAT` ops already implement linear regression on raw Σ-register data. `ΣLIN` is distinct from `L.R.` — ΣLIN accumulates data (via [A]) while L.R. reads pre-accumulated Σ-registers. No namespace collision with built-in `LR` because the Stat Pac mnemonic is `ΣLIN` (with Σ prefix) — but this must be verified against Pitfall 22 (mnemonic shadowing). The QRC mnemonic spellings are `ΣLIN`, `ΣEXP`, `ΣLOGI`, `ΣPOW` — confirmed distinct from built-in `LR`, `CORR`, `YHAT`.

**Algorithmic complexity:** Closed-form — accumulation is O(N) data entry, coefficient extraction is closed-form linear algebra from the six Σ-register values, back-transformation is O(1). Complexity: LOW-MEDIUM.

---

### Program 7: Multiple Linear Regression (ΣMLRXY / ΣMLRXYZ)
**SIZE:** 045 steps

**What it computes:** Multiple linear regression with 2 independent variables (ΣMLRXY) and 3 independent variables (ΣMLRXYZ). Computes partial regression coefficients and prediction.

Data input (ΣMLRXY): `x_i [ENTER+] [ENTER+] t_i [A]` (three-value entry: x, y into stack, then t via ALPHA sequence). The QRC shows `x_i [ENTER+] [ENTER+] t_i [A]` with a subscript indicating this is a 3-level stack input.
Data input (ΣMLRXYZ): `x_i [ENTER+] [ENTER+] z_i [ENTER+] t_i [A]` (four-level: x, y, z, t).
Correction: `x_k [ENTER+] y_k [C]` style.
Results: `[E], [R/S], [R/S], [R/S], x [ENTER+] y [R/S] → ŷ` and similar multi-step displays.

**ALPHA prompts:** No explicit ALPHA prompt strings visible on QRC — data entry is stack-protocol driven.

**Dependencies:** Requires accumulating cross-products (Σx₁x₂, Σx₁y, Σx₂y, Σx₁², Σx₂², etc.) beyond the 6 standard Σ-registers. The 45-step size indicates substantial register usage — likely extends into R07–R20 or higher. OM register table required for exact layout (Pitfall 21 critical path).

**Algorithmic complexity:** MEDIUM-HIGH. Data accumulation: O(N). Coefficient computation: Gauss-Jordan on the (2×2 or 3×3) normal equations. The normal-equation matrix is small and fixed-size (2×2 or 3×3), so it is NOT using the Math Pac I matrix solver (which handles up to 14×14). A self-contained 2×2 or 3×3 Gauss elimination in ~30 lines of Rust. No external user callback needed.

---

### Program 8: Polynomial Regression (ΣPOLYP / ΣPOLYC)
**SIZE:** 045 steps

**What it computes:** Polynomial regression: fits a polynomial ŷ = a₀ + a₁x + a₂x² + ... to data (ΣPOLYP fits the polynomial; ΣPOLYC predicts ŷ for a given x).

Data input (ΣPOLYP): `x_i [ENTER+] y_i [A]` — accumulation into a Vandermonde-style extended register block.
Results (ΣPOLYP): `[E], [R/S], [R/S]` — coefficients displayed sequentially.
Results (ΣPOLYC): `x [R/S] → ŷ` — prediction.

**ALPHA prompts:** Likely includes a degree prompt (analogous to Math Pac I's `ORDER=?` or `DEGREE=?` for the POLY program). The QRC initialization column shows `XEQ ΣPOLYP` / `XEQ ΣPOLYC` without visible ALPHA text — the degree prompt is likely in the results/init flow not shown on the condensed QRC. OM verification required. **Tentative prompt:** `DEGREE=?` or `DEG?` based on Math Pac I precedent — flag as "verification pending Phase 33 spec phase."

**Dependencies:** Requires extended register storage for Σx^k sums up to Σx^(2d) where d is the polynomial degree. Likely shares register structure with Multiple Regression program. Gauss-Jordan on the (d+1)×(d+1) normal equations.

**Algorithmic complexity:** MEDIUM-HIGH. Accumulation: O(N). Normal equation construction: O(N·d). Gauss-Jordan solve: O(d³). For practical d ≤ 4 (matching STAT PAC's capabilities), this is fast but requires careful register layout planning.

---

### Program 9: t Statistics (ΣPTST / ΣTSTAT)
**SIZE:** 015 steps

**What it computes:** Student's t-tests for means:
- **ΣPTST** — One-sample t-test: given x̄, s, n; tests H₀: μ = μ₀. Displays t-statistic and associated p-value (or critical region).
- **ΣTSTAT** — Two-sample t-test: given x̄₁, s₁, n₁, x̄₂, s₂, n₂; tests H₀: μ₁ = μ₂.

Data input (from QRC):
- One-sample path: `x_i [ENTER+] y_i [A]` then `x_i [A], [R/S]` and `y_i [A]`.
- Correction: `x_k [ENTER+] y_k [C]`.
- Results: `d [E], [R/S]` — displays the t-statistic (d) then sequential results.

**ALPHA prompts:** The NPS document (p. 49, ZS-4/5 program) confirms that for the related t-test workflow, degrees of freedom (ν) is computed automatically after data entry, and the user enters `t_{α/2}` via a prompted step. The STAT PAC ΣTSTAT likely displays something like `ν=?` or `T=?` — **verification pending Phase 33 spec phase**. The QRC's compressed notation does not show explicit prompt strings for this program.

**Distribution requirement:** Computes the two-sided t-distribution p-value via the regularized incomplete beta function I_x(ν/2, 1/2). This is a hand-coded ~60-line f64-bridge implementation (Pitfall 23 tolerance: 1e-7 for the iterative incomplete-beta path). The NPS document confirms STAT PAC provides this: "the value of the CDF P(t) may be found by storing degrees of freedom ν in R₁₅, entering t and then [XEQ][TF] in ZSTAT." The STAT PAC's own internal `ΣTSTAT` uses this same distribution function internally.

**Dependencies:** Uses the incomplete beta function from `stat1/distributions.rs`. Does NOT use the Math Pac I `SOLVE` user-callback infrastructure — the t-CDF is self-contained iteration (Pitfall 25 applies: `cancel_requested` must be wired).

**Algorithmic complexity:** MEDIUM. Data entry: O(N) for two-sample variant. t-statistic: closed-form. CDF: iterative incomplete beta (~20–50 Newton steps). Total: MEDIUM.

---

### Program 10: Chi-Square Evaluation (ΣXSQEV / ΣEFXSQ)
**SIZE:** 008 steps

**What it computes:** Chi-square goodness-of-fit test statistic: χ² = Σ (O_i − E_i)² / E_i.
- **ΣXSQEV** — enters observed count O_i and expected count E_i per cell, accumulates χ².
- **ΣEFXSQ** — enters expected frequency E_i as a proportion of n (so E_i = n · f_i), may provide the alternative expected-frequency entry path.

Data input: `O_i [ENTER+] E_i [A]` (ΣXSQEV) and `O_i [A]` (ΣEFXSQ for pre-computed E_i path).
Correction: `O_k [ENTER+] E_k [C]` / `O_k [C]`.
Results: `[E]` — the χ² test statistic.

**ALPHA prompts:** No ALPHA prompts visible on QRC — straightforward stack entry, [A] label.

**Dependencies:** Uses a running accumulator register (beyond R01–R06) for the χ² sum. Simpler than ANOVA because no separate group register blocks needed. Does NOT evaluate the chi-square CDF — that is ΣCHISQD's role.

**Algorithmic complexity:** LOW. Pure accumulation + closed-form χ² computation.

---

### Program 11: Contingency Table (ΣCTKKK / ΣCTKK)
**SIZE:** 015 steps

**What it computes:** r×c contingency table chi-square test. Computes χ² for a contingency table of observed counts.
- **ΣCTKK** — 2×2 or smaller contingency table
- **ΣCTKKK** — general r×c contingency table (larger)

Data input (ΣCTKK): `x_{ij} [ENTER+] x_{2j} [A]` — paired column-entry pattern.
Data input (ΣCTKKK): `x_{ij} [ENTER+] x_{2j} [ENTER+] x_{3j} [A]` — 3-column entry per row.
Correction: `x_{1k} [ENTER+] x_{2k} [C]` / `x_{1k} [ENTER+] x_{2k} [ENTER+] x_{3k} [C]`.
Results: `[E], [R/S], [R/S], [R/S], [R/S], [R/S]` — sequential display.

**ALPHA prompts:** No explicit ALPHA prompt strings visible on QRC. The multi-column entry pattern suggests the program counts rows automatically.

**Dependencies:** Accumulates row sums and column sums alongside the cell counts. Requires additional registers beyond the Σ-block for the marginal totals. OM register table verification required.

**Algorithmic complexity:** MEDIUM. Accumulation: O(rows × cols). χ² computation from marginals: closed-form.

---

### Program 12: Spearman's Rank Correlation Coefficient (ΣSPEAR)
**SIZE:** 003 steps

**What it computes:** Spearman's rank correlation coefficient ρ_s. Given two sets of ranks R_i and S_i (already converted to ranks by the user), computes ρ_s = 1 − 6·Σd²/(n·(n²−1)) where d_i = R_i − S_i.

Data input: `R_i [ENTER+] S_i [A]`
Correction: `R_k [ENTER+] S_k [C]`
Results: `[E], [R/S]` — the rank correlation and n.

**ALPHA prompts:** None — pure stack entry. SIZE 003 means this is the simplest program (only 3 program steps — effectively just Σ+/Σ− of the squared rank differences).

**Dependencies:** Uses the built-in Σ-register block (R01–R06) with the interpretation that Σxy = Σ(R·S), Σx = ΣR, Σy = ΣS, etc. The final formula combines Σ-register values. This program reuses existing infrastructure most directly of all 13 programs.

**Algorithmic complexity:** LOW. Pure register-accumulating, closed-form formula.

---

### Program 13: Normal and Inverse Normal Distribution (ΣNORMD)
**SIZE:** 019 steps

**What it computes:** Three modes for the standard normal distribution (mean=0, σ=1 by convention on the HP-41 Stat Pac, with general μ/σ via standardization):

| Mode | Input | Output |
|---|---|---|
| Inverse: Q(x) → x | Q value [A] | x (the quantile) |
| PDF: x → f(x) | x [C] | f(x) = φ(x) |
| CDF: x → Q(x) | x [E] | Q(x) = 1 − Φ(x) (upper-tail prob) |

**Confirmed from QRC (page 2, direct read):**
- Results column shows: `Q(x) [A] → x`, `x [C] → f(x)`, `x [E] → Q(x)`.
- No "Input Data" or "Correction" columns — ΣNORMD is a one-shot distribution evaluator with no data accumulation phase.
- No re-initialization column — each call is independent.

**ALPHA prompts:** None visible on QRC — input is direct stack value and label key press.

**HP-41 Q(x) convention:** In HP-41 Stat Pac terminology, `Q(x)` is the upper-tail probability = 1 − Φ(x). This matches the NPS document (p. 33): "the program ΣNORMD in STAT PAC may be used to calculate Q(z). Try z = 2.695 as on page 24 to see that .9964 is the value of P(z)." — confirming that `[E]` returns the lower-tail CDF P(z) = Φ(z) and `Q(x)` is the upper tail.

**Confirmed from NPS (p. 33):** "Program ST-19 may be replaced entirely by using the N routine in ZS-2 with μ=0 and σ=1. Alternatively, program ΣNORMD in STAT PAC may be used to calculate Q(z)."

**Implementation requirement:** Three distinct paths from a single XROM entry point:
1. **Inverse normal (probit):** Rational approximation (Acklam/AS 241, ~30 lines). Input: p = Q(x) ∈ (0, 1). Pitfall 19 (non-convergence near tails) applies.
2. **Normal PDF:** `norm_pdf()` from `rust_decimal::MathematicalOps` — direct use, O(1).
3. **Normal CDF:** `1.0 - norm_cdf()` from `rust_decimal::MathematicalOps` — direct use, O(1).

**RNG connection:** The NPS document (p. 21, Section 4.4) confirms the HP-41 RNG formula used with the Stat Pac: `r_{n+1} = FRC(9821 · r_n + 0.211327)`. When used with ΣNORMD for normal deviate generation, the RNG initializes with a `SEED?` prompt (confirmed from ZP4 program, page 22: `[I] → SEED?`, then seed entered via `[R/S]`). The STAT PAC's ΣNORMD does NOT itself contain the RNG — but ΣNORMD is invoked from user programs that include the RNG seed step.

**Dependencies:**
- CDF and PDF: `rust_decimal::MathematicalOps::norm_cdf()` and `norm_pdf()` — zero new code.
- Inverse (probit): hand-coded Acklam/AS 241 rational approximation in `stat1/distributions.rs`.

**Algorithmic complexity:** CDF/PDF path: O(1) closed-form. Inverse path: O(1) rational approximation (no iteration for central region; bisection refinement for tails, ≤ 10 steps). Complexity: LOW for CDF/PDF, LOW-MEDIUM for inverse.

---

### Program 14: Chi-Square Distribution (ΣCHISQD)
**SIZE:** 007 steps

**What it computes:** Chi-square distribution evaluation given degrees of freedom ν:
- `ν [A] → f(x)` — PDF at x (the chi-square density function)
- `ν [E] → P(x)` — CDF at x (the cumulative probability)

**Confirmed from QRC (page 2, direct read):**
- Input data column: `ν [A]` — degrees of freedom entered via [A].
- No "Input Data" initialization separate from results; the QRC shows `p [A]` as input and `x [C] → f(x)`, `x [E] → P(x)` as results.

Wait — re-reading the QRC image more carefully: The QRC Results column shows `x [C] → f(x)` and `x [E] → P(x)`. The Input Data column shows `p [A]`. This indicates:
- The program expects ν (degrees of freedom) stored before the call (likely in a register designated by the OM — the NPS document p. 49 confirms "degrees of freedom ν in R₁₅, entering t and then [XEQ][TF] in ZSTAT" — the STAT PAC ΣCHISQD may use a similar convention).
- `p [A]` may be the initialization step (storing ν = p into a scratch register).
- `x [C] → f(x)` evaluates the chi-square PDF.
- `x [E] → P(x)` evaluates the chi-square CDF.

**OM verification required** for exact degrees-of-freedom storage convention. **Tentative:** ν entered via [A] before x evaluation. Flag as "verification pending Phase 33 spec phase."

**Implementation requirement:** Chi-square CDF = regularized lower incomplete gamma function P(ν/2, x/2). Hand-coded in `stat1/distributions.rs` via f64 bridge (AS 239 series+CF algorithm, ~50 lines). PDF = x^(ν/2−1) · e^(−x/2) / (2^(ν/2) · Γ(ν/2)) — uses `rust_decimal::ln()`, `exp()`, and the log-gamma function (hand-coded or approximated via Stirling).

**Dependencies:** `gamma_regularized_f64()` from `stat1/distributions.rs`. The `cancel_requested` check is needed only if the incomplete gamma series does not converge for pathological inputs (convergence is guaranteed for well-formed ν and x).

**Algorithmic complexity:** Closed-form PDF. CDF: series expansion (fast convergence for typical ν and x, ~15–30 terms). Complexity: LOW-MEDIUM.

---

## (b) Univariate Stats Beyond Built-in Σ-Block

The Stat 1 Pac extends built-in univariate statistics as follows:

| Feature | Where in Stat 1 Pac | Beyond Built-in | Notes |
|---|---|---|---|
| Weighted/grouped mean and SD | ΣBSTAT / ΣBSTG | YES — frequency weights f_i | Built-in Σ+ is unweighted |
| Coefficient of variation (σ/μ) | ΣBSTAT results sequence | YES | Not in v2.2 |
| Skewness γ₁ = μ₃/σ³ | ΣMMTUG / ΣMMTGD | YES | Not in v2.2 |
| Kurtosis γ₂ = μ₄/σ⁴ − 3 | ΣMMTUG / ΣMMTGD | YES | Not in v2.2 |
| Trimmed mean | Not in Stat 1 Pac | — | Not documented in QRC or OM |
| Median | Not in Stat 1 Pac | — | Not documented in QRC |

**No trimmed mean or median is provided by the Stat 1 Pac.** These are user-computed.

---

## (c) Bivariate / Linear Regression Extensions Beyond Built-in L.R.

| Feature | Where in Stat 1 Pac | Notes |
|---|---|---|
| Multiple regression (2 predictors) | ΣMLRXY | NEW — not in v2.2 |
| Multiple regression (3 predictors) | ΣMLRXYZ | NEW |
| Polynomial regression (arbitrary degree) | ΣPOLYP / ΣPOLYC | NEW |
| Curve fitting — linear | ΣLIN | Delegates to Σ-block then L.R.; same as built-in but with data-entry workflow |
| Curve fitting — exponential | ΣEXP | NEW (transforms y→ln(y) before accumulation) |
| Curve fitting — logarithmic | ΣLOGI | NEW (transforms x→ln(x) before accumulation) |
| Curve fitting — power | ΣPOW | NEW (transforms both x→ln(x), y→ln(y)) |
| Confidence intervals on slope/intercept | NOT in Stat 1 Pac | Not documented in QRC; this is Stat 2 Pac / user-program territory |
| Partial correlation coefficients | ΣMLRXY / ΣMLRXYZ results | Displayed as part of multiple regression output |
| Rank correlation (Spearman) | ΣSPEAR | NEW |

---

## (d) Curve Fitting — Confirmed 4 Transforms

The QRC explicitly lists four curve-fitting mnemonic entry points: **ΣLIN, ΣEXP, ΣLOGI, ΣPOW**. All share the same 16-step SIZE, confirming they are variants of a single program.

- **Linear (ΣLIN):** ŷ = a + bx — raw Σ+ accumulation, same as built-in L.R.
- **Exponential (ΣEXP):** ŷ = a·e^(bx) — accumulates (x, ln y); ŷ = e^(a + bx) at prediction.
- **Logarithmic (ΣLOGI):** ŷ = a + b·ln(x) — accumulates (ln x, y); direct prediction.
- **Power (ΣPOW):** ŷ = a·x^b — accumulates (ln x, ln y); ŷ = e^(a + b·ln x) at prediction.

**No other curve types (hyperbolic, logistic, Gompertz, etc.) appear in Stat 1 Pac.** Those belong to the Advantage Pac or user programs.

---

## (e) Distributions — Confirmed Presence in Stat 1 Pac

| Distribution | In Stat 1 Pac? | Entry Point | Notes |
|---|---|---|---|
| Normal CDF Φ(x) | YES | ΣNORMD [E] | Returns upper-tail Q(x) = 1 − Φ(x) |
| Normal PDF φ(x) | YES | ΣNORMD [C] | |
| Inverse normal / probit Φ⁻¹(p) | YES | ΣNORMD [A] | Rational approximation required |
| Chi-square CDF | YES | ΣCHISQD [E] | P(x) with ν from register |
| Chi-square PDF | YES | ΣCHISQD [C] | f(x) with ν from register |
| Student's t CDF | YES (indirect) | ΣTSTAT internal | CDF used to compute p-value in t-test |
| t-distribution table lookup | YES (via ZSTAT in NPS) | [XEQ][TF] in ZSTAT | STAT PAC t-CDF is internal to ΣTSTAT |
| F-distribution | NOT in Stat 1 Pac | — | NPS document (p. 49): "Several key programs [are] missing [from STAT PAC], such as the t and F distributions" — F is NOT in STAT PAC directly |
| Binomial PMF/CDF | NOT in Stat 1 Pac | — | NPS: "there is no binomial program in STAT PAC" (p. 42) |
| Poisson PMF/CDF | NOT in Stat 1 Pac | — | Not in QRC |
| Hypergeometric | NOT in Stat 1 Pac | — | Not in QRC |
| Exponential distribution | NOT in Stat 1 Pac | — | Not in QRC |
| Uniform distribution | NOT in Stat 1 Pac | — | Not in QRC |

**Critical finding:** The NPS document explicitly states: "it will be necessary to supply several key programs that would otherwise have been used, as well as to supply several key programs, such as the t and F distributions that were missing" from STAT PAC. The Stat 1 Pac does NOT provide a standalone t-distribution CDF entry point or F-distribution — the t distribution is used INTERNALLY by ΣTSTAT but not exposed as a separate callable program. The F-distribution is entirely absent.

---

## (f) Random Number Generation

**Confirmed from NPS document (p. 21, Section 4.4):**

The HP-41 RNG formula is:
```
r_{n+1} = FRC(9821 · r_n + 0.211327)
```

This formula is explicitly confirmed as "one developed by Don Malm for the HP-65 User's Library and is referred to on page 24 of the HP-41C Standard Applications manual."

**SEED? prompt confirmed:** The NPS document (p. 22, ZP4 user instructions, Step E5a/N6a): pressing [I] displays `SEED?`, user enters seed (0 ≤ Seed < 1) via [R/S].

**Is RAND a standalone Stat 1 Pac program?** NO. The QRC does not list a standalone `RAND` or `SEED` program among the 13 programs. The RNG is a subroutine embedded in programs that need random number generation (primarily continuous distribution sampling). The QRC shows ΣNORMD has NO random-number functionality (it is a pure CDF/PDF/inverse evaluator). The NPS document uses the RNG in standalone programs (ZP4, ZS-2) separate from STAT PAC.

**Implication for v3.1:** The RNG `FRC(9821·x + 0.211327)` should be implemented as a helper function in `stat1/` if any Stat 1 program internally uses random sampling (none of the 13 QRC programs appear to do so based on the QRC alone). More likely the RNG is a v3.1 utility callable from user programs via `XEQ "RAND"` as a bonus convenience function. **OM verification required** to confirm whether the Stat 1 Pac ROM actually includes a RAND subroutine or whether it was purely a user-program convention.

**Tentative decision:** Implement the RNG formula as `op_stat1_rand()` callable via a short `RAND` or `RNDMU` mnemonic for completeness. The `rand_seed` field on `CalcState` must use `#[serde(default)]` but NOT `#[serde(skip)]` per Pitfall 20. Store seed in a designated register (OM to specify) or in the `rand_seed: HpNum` CalcState field.

---

## (g) Permutations, Combinations, Factorial

**Permutations P(N,R) and Combinations C(N,R)** are NOT listed in the Stat 1 Pac QRC.

However:
- The NPS document (p. 3–4, ZP2 program) confirms that HP's built-in `FACT` function handles factorial (n! for 0 ≤ n ≤ 69) and is accessible directly.
- The NPS document shows P(N,R) and C(N,R) computed via user program ZP2 (not STAT PAC) with the ALPHA prompts `N=?` and `R=?` confirmed from the user instructions table (p. 8, Step 7 and Step 8).

**Implication for v3.1:** P(N,R) and C(N,R) are NOT features of the Stat 1 Pac module. `FACT` is already in hp41-core (v2.2 Phase 20). The NPS program uses user-mode label keys [b] and [c] for P and C respectively — these are user programs, not XROM functions. DO NOT add P/C to the STAT_1.ops table.

---

## (h) Hypothesis Testing and Confidence Intervals

| Feature | In Stat 1 Pac | Entry Point | Notes |
|---|---|---|---|
| One-sample t-test (μ test) | YES | ΣPTST | Returns t-statistic and p-value |
| Two-sample t-test | YES | ΣTSTAT | Equal/pooled variance assumed |
| Chi-square goodness-of-fit | YES | ΣXSQEV / ΣEFXSQ | Computes χ² test statistic only |
| Chi-square CDF for p-value | YES | ΣCHISQD | Used after ΣXSQEV to get p-value |
| Contingency table χ² | YES | ΣCTKKK / ΣCTKK | Full χ² for r×c table |
| One-way ANOVA F-test | YES | ΣAOVONE | F-ratio computed |
| Two-way ANOVA F-test | YES | ΣAOVTWO | Row and column F-ratios |
| ANCOVA F-test | YES | ΣANOCOV | With covariate adjustment |
| Confidence intervals for μ | NOT in Stat 1 Pac | — | CI computation is user-program task using ΣTSTAT output + t-table |
| Confidence intervals for σ | NOT in Stat 1 Pac | — | Uses χ² table (user-program) |
| F-distribution p-value | NOT in Stat 1 Pac | — | NPS confirms F distribution absent from STAT PAC |
| Welch's t-test (unequal variance) | NOT in Stat 1 Pac | — | Not in QRC |
| Mann-Whitney / non-parametric | NOT in Stat 1 Pac | — | Only Spearman rank correlation |

**Confirmed: CI computation is NOT part of Stat 1 Pac.** The NPS document's ZS-4/5 program shows confidence intervals computed as a separate user program that calls STAT PAC distribution functions. The Stat Pac provides the statistical test results; the user combines them with quantile values for confidence intervals.

---

## (i) Histograms and Frequency Tables

**Histograms are NOT in the Stat 1 Pac QRC.** The QRC lists no histogram or frequency-table program.

The NPS document's histogram functionality is in program ST-07/9 (a standalone user program, not STAT PAC) that uses the built-in Σ-register system for cell frequency accumulation with prompts `CELLS?`, `XMIN?`, `W=?` (cell width).

**Implication:** DO NOT add histogram functionality to the STAT_1 XROM module. This is explicitly an anti-feature (would be incorrect behavioral emulation of the OM).

---

## Table Stakes vs. Differentiators vs. Anti-Features

### Table Stakes (must-have for "feature-complete per OM 00041-90030" claim)

These are the 13 QRC programs + their named entry points. All are required:

| Feature | Complexity | Op Variants Needed | Notes |
|---|---|---|---|
| ΣBSTAT / ΣBSTG | LOW | 2 | Reuses existing Σ-registers |
| ΣMMTUG / ΣMMTGD | MEDIUM | 2 | Needs extended register layout from OM |
| ΣAOVONE | MEDIUM | 1 | Extended register layout required |
| ΣAOVTWO | MEDIUM | 1 | Extended register layout required |
| ΣANOCOV | MEDIUM-HIGH | 1 | Most complex ANOVA variant |
| ΣLIN / ΣEXP / ΣLOGI / ΣPOW | LOW-MEDIUM | 4 | Curve fitting via Σ-transform |
| ΣMLRXY / ΣMLRXYZ | MEDIUM-HIGH | 2 | Normal equation 2×2 / 3×3 solve |
| ΣPOLYP / ΣPOLYC | MEDIUM-HIGH | 2 | Degree-to-be-confirmed; normal equation |
| ΣPTST / ΣTSTAT | MEDIUM | 2 | Incomplete beta for t-CDF |
| ΣXSQEV / ΣEFXSQ | LOW | 2 | Pure accumulation |
| ΣCTKKK / ΣCTKK | MEDIUM | 2 | Extended register layout |
| ΣSPEAR | LOW | 1 | 3 program steps, pure Σ |
| ΣNORMD | LOW-MEDIUM | 1 (3 modes) | CDF/PDF: direct; inverse: rational approx |
| ΣCHISQD | LOW-MEDIUM | 1 (2 modes) | Incomplete gamma function |

**Total Op variants: ~24** (some programs have multiple entry-point Ops, others dispatch modes via label keys rather than separate Ops).

### Differentiators (optional post-v3.1 polish — not in OM)

| Feature | Rationale | Defer To |
|---|---|---|
| RAND / RNDMU subroutine | QRC does not list it; NPS uses it as user-program; useful but not OM-mandated | v3.1 bonus if OM verification confirms it's in the ROM |
| F-distribution CDF | Not in Stat 1 Pac but commonly needed with ANOVA | v3.2 or user-program guidance |
| Normal sample generation | Not in Stat 1 Pac (NPS used separate ZP4 program) | User-program guidance in docs |
| Confidence interval helpers | Not in Stat 1 Pac | User-program guidance in docs |

### Anti-Features (explicitly do NOT add to STAT_1)

| Anti-Feature | Why Avoid | What to Do Instead |
|---|---|---|
| Binomial / Poisson / Hypergeometric distributions | Not in Stat 1 Pac (NPS confirms absent) | Document absence in hp41-stat1-divergences.md |
| F-distribution evaluator | Not in Stat 1 Pac (NPS confirms absent) | Defer to v3.2 |
| Histogram / frequency table program | Not in Stat 1 Pac | Document absence |
| P(N,R) / C(N,R) permutations/combinations | Not in Stat 1 Pac (built-in FACT + user programs) | Document that FACT covers n! |
| Confidence interval computations | Not in Stat 1 Pac | User combines ΣTSTAT output with quantile lookup |
| Welch t-test or non-parametric tests beyond Spearman | Not in Stat 1 Pac | Defer to Advantage Pac scope |
| Normal deviate generator as standalone XROM | Not in Stat 1 Pac QRC | Bonus only if OM verification confirms |

---

## Feature Dependencies

```
ΣBSTAT / ΣBSTG → existing Σ-registers R01–R06 (already in hp41-core/src/ops/stats.rs)

ΣMMTUG / ΣMMTGD → extended Σ-registers R07+ (OM layout verification required)

ΣLIN / ΣEXP / ΣLOGI / ΣPOW → existing op_sigma_plus() + checked_ln() (R01–R06)
  ΣLOGI / ΣEXP / ΣPOW → rust_decimal::ln() already available

ΣMLRXY / ΣMLRXYZ → normal equations → Gauss-Jordan 2×2 or 3×3 (self-contained)

ΣPOLYP / ΣPOLYC → normal equations → Gauss-Jordan (d+1)×(d+1)

ΣPTST / ΣTSTAT → stat1/distributions.rs::beta_regularized_f64()
  beta_regularized_f64 → stat1/distributions.rs::gamma_regularized_f64() [for integer ν]

ΣXSQEV / ΣEFXSQ → accumulator register (no distribution function)

ΣCHISQD → stat1/distributions.rs::gamma_regularized_f64()

ΣCTKKK / ΣCTKK → contingency sum registers (extended block)

ΣAOVONE / ΣAOVTWO / ΣANOCOV → extended Σ registers (R07+, OM layout required)

ΣSPEAR → existing Σ-registers R01–R06 (Σxy = ΣR·S, Σx = ΣR, etc.)

ΣNORMD → rust_decimal::norm_cdf() + norm_pdf() [CDF/PDF path, zero new code]
         → stat1/distributions.rs::norm_cdf_inv_f64() [inverse path, Acklam AS 241]

RAND (if implemented) → stat1/rng.rs::rng_step() (FRC formula) → CalcState::rand_seed
```

---

## MVP Recommendation

**Phase 33 priority order (hardest dependencies first):**

1. **stat1/distributions.rs** — the three numerical primitives (`norm_cdf_inv_f64`, `gamma_regularized_f64`, `beta_regularized_f64`) are required by multiple programs. Build and test these first with a scipy.stats reference harness before any program Op is implemented.

2. **ΣNORMD + ΣCHISQD** — the two distribution-evaluation programs that purely consume distribution primitives. Quick to implement once distributions are done. Good end-to-end test of the entire pipeline.

3. **ΣSPEAR + ΣXSQEV/ΣEFXSQ** — simplest programs (LOW complexity). Good integration smoke tests.

4. **ΣBSTAT/ΣBSTG + ΣLIN/ΣEXP/ΣLOGI/ΣPOW** — build on existing Σ-register infrastructure. No new numerical primitives.

5. **ΣMMTUG/ΣMMTGD + ΣAOVONE/ΣAOVTWO/ΣANOCOV + ΣCTKKK/ΣCTKK** — require OM register layout. Must read the OM "Storage Registers" section before starting. Block on Pitfall 21 resolution.

6. **ΣPTST/ΣTSTAT** — requires beta_regularized_f64 (from step 1) plus register layout.

7. **ΣMLRXY/ΣMLRXYZ + ΣPOLYP/ΣPOLYC** — largest programs (SIZE 045). Self-contained Gauss-Jordan. Do last.

**Defer:** RAND/RNG until OM verification confirms it is in the Stat 1 Pac ROM.

---

## Open Questions (Verification Pending Phase 33 Spec Phase)

| Question | Impact | Source |
|---|---|---|
| Exact register layout for ΣMMTUG, ΣAOVONE, ΣAOVTWO, ΣANOCOV, ΣMLRXY, ΣCTKKK | CRITICAL — determines which registers hold Σx³, Σx⁴, group sums, cross-products | Stat 1 Pac OM 00041-90030 "Storage Registers" section |
| Does the Stat 1 Pac ROM contain a RAND / RNDMU subroutine? | MEDIUM — determines whether RNG is a STAT_1 Op or user-program only | OM listing or community FOCAL disassembly |
| Exact degree-prompt wording for ΣPOLYP ("DEG?" / "DEGREE=?" / other?) | LOW — affects ALPHA prompt routing | OM user instructions for Polynomial Regression |
| Exact degrees-of-freedom storage convention for ΣCHISQD (register number) | HIGH — determines CalcState interaction | OM user instructions for Chi-Square Distribution |
| Exact prompt strings for ΣPTST / ΣTSTAT ("T=?" / "DF=?" / etc.) | MEDIUM — affects modal-prompt routing | OM user instructions for t Statistics |
| Does ΣTSTAT assume equal or unequal variances (pooled or Welch)? | HIGH — affects formula and degrees of freedom | OM user instructions for t Statistics |
| OM version: 00041-90030 (confirmed) or different number for some editions? | LOW — citation accuracy | literature.hpcalc.org item 800 (August 1984 scan) |

---

## Algorithmic Complexity Summary for Roadmapper

| Program | Complexity | Iteration? | New Numerical Primitive? | Pitfall Flag |
|---|---|---|---|---|
| ΣBSTAT / ΣBSTG | LOW | No | No | P21 (register layout) |
| ΣMMTUG / ΣMMTGD | MEDIUM | No | No | P21 (register layout), P18 (variance) |
| ΣAOVONE | MEDIUM | No | No | P21 (register layout) |
| ΣAOVTWO | MEDIUM | No | No | P21 (register layout) |
| ΣANOCOV | MEDIUM-HIGH | No | No | P21 (register layout) |
| ΣLIN / ΣEXP / ΣLOGI / ΣPOW | LOW-MEDIUM | No | No | P22 (mnemonic collision check) |
| ΣMLRXY / ΣMLRXYZ | MEDIUM-HIGH | No (fixed-size Gauss) | No | P21 (register layout) |
| ΣPOLYP / ΣPOLYC | MEDIUM-HIGH | No (fixed-size Gauss) | No | P21 (register layout) |
| ΣPTST / ΣTSTAT | MEDIUM | Yes (incomplete beta) | YES: beta_regularized_f64 | P19 (convergence), P25 (cancel_requested) |
| ΣXSQEV / ΣEFXSQ | LOW | No | No | None |
| ΣCTKKK / ΣCTKK | MEDIUM | No | No | P21 (register layout) |
| ΣSPEAR | LOW | No | No | None |
| ΣNORMD (CDF/PDF) | LOW | No | No (rust_decimal) | P23 (tolerance: 1e-9 for CDF/PDF) |
| ΣNORMD (inverse) | LOW-MEDIUM | No (rational approx) | YES: norm_cdf_inv_f64 | P19 (tail convergence), P23 |
| ΣCHISQD | LOW-MEDIUM | Yes (incomplete gamma) | YES: gamma_regularized_f64 | P25 (cancel_requested for non-convergence) |
| stat1/distributions.rs | HIGH (build first) | Mixed | ALL THREE primitives | P19 (convergence), P23 (tolerance) |

---

## Sources

- HP-41C Stat Pac Quick Reference Card 00041-90061 (June 1979) — all 13 programs directly read from 2-page PDF image at `literature.hpcalc.org/community/hp41-pac-stat-qrc-en.pdf`. HIGH confidence: primary source.
- Naval Postgraduate School NPS55-84-003, Peter W. Zehna (February 1984, DTIC AD-A140573, `literature.hpcalc.org/community/hp41-probability-statistics.pdf`) — confirms RNG formula, ΣNORMD workflow, ΣBSTG display, ΣCHISQD degrees-of-freedom convention, confirms F-distribution and Binomial absent from STAT PAC. HIGH confidence: authoritative secondary source with direct STAT PAC integration.
- STACK.md (this research cycle, 2026-05-21) — confirms XROM ID 2, 13 programs from QRC, all numerical algorithm decisions. HIGH confidence.
- ARCHITECTURE.md (this research cycle, 2026-05-21) — confirms data storage patterns, component boundaries, register conventions. HIGH confidence.
- PITFALLS.md (this research cycle, 2026-05-21) — Pitfalls 18–27 cross-referenced throughout. HIGH confidence.
- `hp41-core/src/ops/stats.rs` (codebase) — confirms R01–R06 Σ-register layout. HIGH confidence.
- `hp41-core/src/ops/math1/xrom.rs` (codebase) — confirms bit-1 stub comment for STAT_1. HIGH confidence.
- `rust_decimal::MathematicalOps` (docs.rs, 2026-05-21) — confirms norm_cdf(), norm_pdf(), erf() availability; absence of incomplete gamma, incomplete beta, inverse normal. HIGH confidence.
