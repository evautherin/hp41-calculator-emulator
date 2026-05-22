# Phase 33: hp41-core — XROM Activation + Distribution Primitives + All Stat 1 Ops — Specification

**Created:** 2026-05-22
**Ambiguity score:** 0.113 (gate: ≤ 0.20)
**Requirements:** 46 locked

## Goal

`hp41-core` activates the STAT_1 XROM module (id = 2, bit 1) and ships all 14 Stat 1 Pac entry points (~24 `Op` variants across the 13 QRC programs) plus the three hand-coded f64-bridge distribution primitives (Acklam/AS 241, AS 239, AS 63) — `XEQ "ΣNORMD"`, `XEQ "ΣCHISQD"`, `XEQ "ΣSPEAR"`, etc. dispatched from within the core library return correct numerical results validated against scipy.stats oracle constants, with RAND/SEED RNG state surviving save/load via `#[serde(default)]` (no `skip`).

## Background

v3.0 (Math Pac I, XROM id 7) shipped 2026-05-21 with the resolver chain extended to fire `xrom_resolve` LAST, bit 0 of `xrom_modules: u8` arming Math 1, and `default_xrom_modules() = 0b0000_0001`. The bit-1 arm of `xrom_resolve` (`hp41-core/src/ops/math1/xrom.rs:133–134`) is a documented stub: `// if modules & 0b0000_0010 != 0 { stat1_resolve(name) }`. No `hp41-core/src/ops/stat1/` module exists. `hp41-core/src/ops/stats.rs` provides the v1.x R01–R06 Σ-register accumulators (`op_sigma_plus`, `op_sigma_minus`, `op_mean`, etc.) that Stat 1 Pac extends but never modifies. The contamination guard `scripts/check-free42-contamination.sh` (12 distinctive symbols, math1/-scoped) does NOT yet cover `hp41-core/src/ops/stat1/`. CONTEXT.md (commit 1f46b9d) captured nine D-33.* decisions; this SPEC.md converts the seven LOCKED items from D-33.1 plus the 39 v3.1 Phase-33 requirements into a falsifiable contract.

## Requirements

### XROM Framework Activation

1. **STAT_1 module const** (STAT-FW-01): `STAT_1: XromModule` registered with `id: u8 = 2`.
   - Current: only `MATH_1: XromModule { id: 7, name: "MATH 1A", ops: &[...52 entries] }` exists in `hp41-core/src/ops/math1/xrom.rs:43`.
   - Target: `pub const STAT_1: XromModule { id: 2, name: "STAT 1B", ops: &[...] }` lives in the same file (math1/ freeze exception per D-33.3a, limited to `xrom.rs`).
   - Acceptance: `STAT_1.id == 2` and `STAT_1.name == "STAT 1B"` asserted by a unit test parallel to `math1_const_id_and_name` at `hp41-core/src/ops/math1/xrom.rs:271`.

2. **Resolver fires LAST after MATH_1** (STAT-FW-01): `xrom_resolve()` calls `stat1_resolve()` AFTER `math1_resolve()` when bit 1 is set.
   - Current: `xrom_resolve` at `hp41-core/src/ops/math1/xrom.rs:127` resolves only `math1_resolve` under bit 0.
   - Target: bit-1 arm at line 134 calls `stat1_resolve(name)`; returns LAST in the chain (after `builtin_card_op`, before `Err(InvalidOp)`).
   - Acceptance: `xrom_shadowing.rs` CI gate extended for `STAT_1.ops`; every Stat 1 mnemonic disjoint from `MATH_1.ops` AND from every `builtin_card_op` mnemonic. Test asserts: `xrom_resolve("ΣNORMD", 0b0000_0011)` returns `Some(Op::SigmaNormd)`; `xrom_resolve("ΣNORMD", 0b0000_0001)` returns `None` (bit-1 isolation).

3. **default_xrom_modules() flips to 0b0000_0011** (STAT-FW-02): Math 1 + Stat 1 both pre-loaded by default.
   - Current: `fn default_xrom_modules() -> u8 { 0b0000_0001 }` at `hp41-core/src/state.rs:228`.
   - Target: `fn default_xrom_modules() -> u8 { 0b0000_0011 }` at the same site; serde-default preserved.
   - Acceptance: a freshly-constructed `CalcState::new()` has `xrom_modules == 0b0000_0011`; bit 0 AND bit 1 both set.

4. **Startup migration of v3.0 save files** (STAT-FW-02, STAT-QUAL-10, P24): `CalcState::migrate_after_load()` sets bit 1 unconditionally.
   - Current: no migration method; v3.0 save files with `"xrom_modules": 1` would deserialize unchanged.
   - Target: `impl CalcState { pub fn migrate_after_load(&mut self) { if self.xrom_modules & 0b0000_0010 == 0 { self.xrom_modules |= 0b0000_0010; } } }` in `hp41-core/src/state.rs`; called once at the end of `hp41-cli/src/persistence.rs::load_state` and `hp41-gui/src-tauri/src/persistence.rs` deserialize paths (CLI + GUI sites are Phase-34 / Phase-36 work; the core method is Phase 33).
   - Acceptance: deserializing a JSON blob containing `"xrom_modules": 1` (no other v3.1 fields) followed by `migrate_after_load()` yields `state.xrom_modules == 0b0000_0011`; deserializing `"xrom_modules": 3` is a no-op (idempotent). Test in `hp41-core/src/state.rs::tests`.

5. **4-way exhaustive-match invariant items 1+2** (STAT-FW-03): every new Stat 1 `Op` variant lands in `dispatch()` AND `execute_op()` before Phase 34/36.
   - Current: 4-way invariant enforced for Math Pac I; no Stat 1 variants exist.
   - Target: ~24 new `Op::Sigma*` (or named) variants exhaustively matched in `hp41-core/src/ops/mod.rs::dispatch()` AND `hp41-core/src/ops/program.rs::execute_op()`. No `_ =>` catch-all permitted.
   - Acceptance: `cargo check -p hp41-core` compiles; `cargo clippy -p hp41-core -- -D warnings` clean. Phase 37 cross-checks via `xrom_shadowing.rs` (STAT_1.ops length matches resolver match-arm count).

6. **XEQ-by-name dispatch from CLI + GUI + program-runtime** (STAT-FW-04): all 14 entry points callable.
   - Current: only Math Pac I + built-ins dispatch via XEQ-by-name.
   - Target: every Stat 1 mnemonic in §"Stat 1 Pac Mnemonics" below resolves through `xrom_resolve` and dispatches to the correct Op when called from `run_program` / `run_loop` AND from a CLI keyboard XEQ invocation (the CLI keyboard wiring is Phase 34; Phase 33 ensures `run_program`-level dispatch).
   - Acceptance: integration test `stat1_xeq_by_name_dispatch.rs` runs a synthetic program containing `XEQ "ΣNORMD"` and asserts the appropriate Op fires; one test per mnemonic.

### Stat 1 Pac Mnemonics (14 entry points)

Locked from QRC 00041-90061 (June 1979) and Stat 1 Pac OM 00041-90030. Final string matches OM exactly; any mismatch becomes a documented divergence in `docs/hp41-stat1-divergences.md` (Phase 35).

| Mnemonic | Op variant (tentative) | Source file | Plan |
|----------|------------------------|-------------|------|
| ΣBSTAT | `Op::SigmaBstat` | `ops/stat1/basic_stats.rs` | 33-05 |
| ΣBSTG | `Op::SigmaBstg` | `ops/stat1/basic_stats.rs` | 33-05 |
| ΣMMTUG | `Op::SigmaMmtug` | `ops/stat1/moments.rs` | 33-06 |
| ΣMMTGD | `Op::SigmaMmtgd` | `ops/stat1/moments.rs` | 33-06 |
| ΣAOVONE | `Op::SigmaAovone` | `ops/stat1/anova.rs` | 33-06 |
| ΣAOVTWO | `Op::SigmaAovtwo` | `ops/stat1/anova.rs` | 33-06 |
| ΣANOCOV | `Op::SigmaAnocov` | `ops/stat1/anova.rs` | 33-06 |
| ΣLIN | `Op::SigmaLin` | `ops/stat1/regression.rs` | 33-05 |
| ΣEXP | `Op::SigmaExp` | `ops/stat1/regression.rs` | 33-05 |
| ΣLOGI | `Op::SigmaLogi` | `ops/stat1/regression.rs` | 33-05 |
| ΣPOW | `Op::SigmaPow` | `ops/stat1/regression.rs` | 33-05 |
| ΣMLRXY | `Op::SigmaMlrxy` | `ops/stat1/regression.rs` | 33-08 |
| ΣMLRXYZ | `Op::SigmaMlrxyz` | `ops/stat1/regression.rs` | 33-08 |
| ΣPOLYP | `Op::SigmaPolypWorkflow` | `ops/stat1/regression.rs` | 33-08 |
| ΣPOLYC | `Op::SigmaPolyc` | `ops/stat1/regression.rs` | 33-08 |
| ΣPTST | `Op::SigmaPtst` | `ops/stat1/hypothesis.rs` | 33-07 |
| ΣTSTAT | `Op::SigmaTstat` | `ops/stat1/hypothesis.rs` | 33-07 |
| ΣXSQEV | `Op::SigmaXsqev` | `ops/stat1/nonparam.rs` | 33-04 |
| ΣEFXSQ | `Op::SigmaEfxsq` | `ops/stat1/nonparam.rs` | 33-04 |
| ΣCTKKK | `Op::SigmaCtkkk` | `ops/stat1/nonparam.rs` | 33-06 |
| ΣCTKK | `Op::SigmaCtkk` | `ops/stat1/nonparam.rs` | 33-06 |
| ΣSPEAR | `Op::SigmaSpear` | `ops/stat1/nonparam.rs` | 33-04 |
| ΣNORMD | `Op::SigmaNormdWorkflow` | `ops/stat1/normd.rs` | 33-03 |
| ΣCHISQD | `Op::SigmaChisqdWorkflow` | `ops/stat1/chisqd.rs` | 33-03 |
| RAND | `Op::Rand` (emulator extension) | `ops/stat1/rand.rs` | 33-08 |
| SEED | `Op::Seed` (emulator extension) | `ops/stat1/rand.rs` | 33-08 |

Final Op variant identifiers are plan-phase territory; SPEC.md locks the mnemonic strings only.

### Univariate Stats

7. **ΣBSTAT / ΣBSTG extended summaries** (STAT-UNI-01): weighted mean, coefficient of variation σ/μ, plus bivariate summary variants.
   - Current: `op_mean`, `op_stddev` etc. in `hp41-core/src/ops/stats.rs` cover only the v1.x basic Σ-register summary.
   - Target: `XEQ "ΣBSTAT"` consumes the R01–R06 Σ-register state populated by existing `op_sigma_plus` accumulator; pushes weighted mean and CV onto the stack per OM "Results" section.
   - Acceptance: against the scipy-derived oracle `data = [(1,2,3,4,5), (10,20,30,40,50)]`, weighted mean of x with weights y is 3.6667, CV = 0.4083; both within 1e-9 relative tolerance.

8. **ΣMMTUG / ΣMMTGD third + fourth moments + skewness + kurtosis** (STAT-UNI-02): γ₁ = μ₃/σ³, γ₂ = μ₄/σ⁴ − 3.
   - Current: no third/fourth moment Op.
   - Target: ΣMMTUG ungrouped (single-stream samples); ΣMMTGD grouped (frequency-weighted); register layout from OM "Storage Registers" (per Req. 9).
   - Acceptance: against oracle dataset `[1,2,3,4,5,6,7,8,9,10]`, μ₃ = 0.0, μ₄ = 33.0, γ₁ = 0.0, γ₂ ≈ −1.224 (excess kurtosis); within 1e-9 relative tolerance.

9. **OM Storage-Registers transcription as Plan-33-00 single source of truth** (STAT-UNI-03, P21): the OM register layout MUST be transcribed verbatim into `hp41-core/src/ops/stat1/mod.rs` as a `//!`-doc-comment header AND a `pub const STAT1_MAX_REG: usize` BEFORE any `Op` that reads from those registers is implemented.
   - Current: `hp41-core/src/ops/stat1/` does not exist; OM register layout exists only in OM 00041-90030 PDF.
   - Target: `ops/stat1/mod.rs` ships in Plan 33-00 with the OM "Storage Registers" section as a `//!` block plus `pub const STAT1_MAX_REG: usize = <OM-table-derived>` and a SIZE-floor guard reuse hook for `ops/stats.rs`.
   - Acceptance: `ops/stat1/mod.rs` exists; `STAT1_MAX_REG` is a `const` with a value taken from the transcribed OM table (the doc comment cites OM page numbers); `cargo check` succeeds. Any Op that reads register index ≥ R06 (current `ops/stats.rs` floor) lands AFTER this file. **Lock:** SPEC.md does NOT inline the OM layout itself — Plan 33-00 is the single source of truth; this requirement gates downstream plans.

10. **`[C]` correction-key undo for univariate accumulation** (STAT-UNI-04): the existing `op_sigma_minus` path correctly reverses the last `op_sigma_plus` call for every Stat 1 univariate accumulation flow.
    - Current: `op_sigma_minus` reverses R01–R06 increments only; new ΣMMTUG/ΣMMTGD registers from Req. 9 may not be covered.
    - Target: every register index touched by Stat 1 accumulation has a matching reversal in `op_sigma_minus` (extended if Req. 9 expands the register set).
    - Acceptance: round-trip test `sigma_plus_then_minus_restores_state.rs` extended with Stat 1 register coverage; final state equal (within 1e-12 relative) to pre-`sigma_plus` state.

### ANOVA Family

11. **ΣAOVONE one-way ANOVA F-ratio + group means** (STAT-AOV-01): consumes group-keyed Σ-register accumulators per OM layout (Req. 9).
    - Current: no ANOVA Op.
    - Target: `XEQ "ΣAOVONE"` computes between-group sum-of-squares SSB, within-group SSW, F = (SSB/dfB) / (SSW/dfW); pushes F onto stack; group means available via OM-specified result-output channel (stack push vs print buffer per Plan 33-06 OM "Results" read).
    - Acceptance: against oracle dataset (3 groups of 5 samples each: `[1,2,3,4,5]`, `[6,7,8,9,10]`, `[11,12,13,14,15]`), F = 100.0 within 1e-9 relative tolerance.

12. **ΣAOVTWO two-way ANOVA row + column F-ratios** (STAT-AOV-02): no replications.
    - Current: no two-way ANOVA Op.
    - Target: `XEQ "ΣAOVTWO"` computes F_row and F_col against OM-specified r×c layout per Req. 9.
    - Acceptance: against oracle 3×4 dataset (scipy.stats.f_oneway reference), F_row and F_col within 1e-7 relative tolerance (iterative path because internal cross-product sums chain decimal accumulator multiplication).

13. **ΣANOCOV one-way ANCOVA F-ratio with covariate** (STAT-AOV-03): covariate adjustment per OM formula.
    - Current: no ANCOVA Op.
    - Target: `XEQ "ΣANOCOV"` computes adjusted F-ratio per OM 00041-90030 § "ANCOVA"; register layout per Req. 9.
    - Acceptance: against oracle dataset matching NPS ZA-3 example, F within 1e-7 relative tolerance.

14. **OM-verified ANOVA register layout** (STAT-AOV-04, P21): no register-index guessing.
    - Current: register indices not transcribed.
    - Target: Plan 33-06 reads OM "Storage Registers" subsection for ANOVA and uses the indices recorded in Plan 33-00's `ops/stat1/mod.rs` transcription. NO floor()-derived or fmod()-derived register indices.
    - Acceptance: code review confirms that every ANOVA register access uses a named constant (e.g. `STAT1_AOV_SSB_REG`) defined in `ops/stat1/mod.rs` and traceable to an OM page citation.

### Regression Family

15. **ΣLIN linear regression ŷ = a + bx** (STAT-REG-01).
    - Current: no curve-fit Op (existing `op_linear_reg` from v1.x covers raw Σ-register a, b but no XROM-named entry point).
    - Target: `XEQ "ΣLIN"` computes a, b from R01–R06 Σ-register state and pushes results per OM "Results".
    - Acceptance: oracle `x=[1,2,3,4,5], y=[2,4,6,8,10]` yields a = 0.0, b = 2.0; within 1e-9 relative tolerance.

16. **ΣEXP exponential ŷ = a·e^(bx)** (STAT-REG-02): accumulates (x, ln y).
    - Current: no exponential fit Op.
    - Target: `XEQ "ΣEXP"` applies ln-transform internally; a, b returned per OM.
    - Acceptance: oracle `x=[1,2,3], y=[e, e², e³]` yields a ≈ 1.0, b ≈ 1.0; within 1e-9 relative tolerance.

17. **ΣLOGI logarithmic ŷ = a + b·ln(x)** (STAT-REG-03): accumulates (ln x, y).
    - Current: no logarithmic fit Op.
    - Target: `XEQ "ΣLOGI"` applies ln-transform on x.
    - Acceptance: oracle `x=[1, e, e²], y=[2, 3, 4]` yields a = 2.0, b = 1.0; within 1e-9 relative tolerance.

18. **ΣPOW power ŷ = a·x^b** (STAT-REG-04): accumulates (ln x, ln y).
    - Current: no power fit Op.
    - Target: `XEQ "ΣPOW"` applies double-ln transform.
    - Acceptance: oracle `x=[1,2,4,8], y=[1,4,16,64]` yields a = 1.0, b = 2.0; within 1e-9 relative tolerance.

19. **ΣMLRXY 2-predictor multiple linear regression** (STAT-REG-05).
    - Current: no multiple-regression Op.
    - Target: `XEQ "ΣMLRXY"` solves 2×2 normal equations via self-contained Gauss elimination (NOT Math Pac I MATRIX solver per Req. 21).
    - Acceptance: oracle dataset (scipy.stats.linregress two-predictor reference), partial coefficients within 1e-7 relative tolerance.

20. **ΣMLRXYZ 3-predictor multiple linear regression** (STAT-REG-06).
    - Current: no 3-predictor Op.
    - Target: `XEQ "ΣMLRXYZ"` solves 3×3 normal equations via self-contained Gauss elimination.
    - Acceptance: oracle dataset (scipy reference), partial coefficients within 1e-7 relative tolerance.

21. **Self-contained Gauss elimination — not MATRIX** (STAT-REG-09).
    - Current: Math Pac I `MATRIX/SIMEQ` exists but couples STAT_1 to MATH_1 if reused.
    - Target: ΣMLRXY / ΣMLRXYZ / ΣPOLYP implement (d+1)×(d+1) Gauss elimination locally in `ops/stat1/regression.rs`; no `ops::math1::matrix::*` imports.
    - Acceptance: `grep -n "ops::math1::matrix" hp41-core/src/ops/stat1/` returns empty.

22. **ΣPOLYP polynomial regression with degree prompt** (STAT-REG-07): tentative prompt `DEGREE=?` per Math Pac I `POLY` precedent.
    - Current: no polynomial-fit XROM entry.
    - Target: `XEQ "ΣPOLYP"` opens a modal prompt asking for degree d via the existing `print_buffer` + `modal_program` infrastructure; R/S submits d; subsequent (x, y) accumulations feed (d+1)×(d+1) normal-equation solve.
    - Acceptance: with d = 2 and oracle `x=[1,2,3,4,5], y=[1,4,9,16,25]` the polynomial fit yields coefficients (a₀=0, a₁=0, a₂=1) within 1e-7 relative tolerance. **OM override:** if OM 00041-90030 §ΣPOLYP shows a different prompt string, plan-phase substitutes that string; SPEC.md does not lock the literal `DEGREE=?` text, only that a modal-prompt-driven degree-entry workflow exists.

23. **ΣPOLYC predict ŷ from x after polynomial fit** (STAT-REG-08).
    - Current: no polynomial-prediction Op.
    - Target: `XEQ "ΣPOLYC"` Horner-evaluates the most recent ΣPOLYP coefficient set at the X-register value.
    - Acceptance: chained after the Req. 22 oracle, ΣPOLYC at x = 6 yields ŷ = 36.0 within 1e-9 relative tolerance.

### Hypothesis Tests

24. **ΣPTST one-sample t-test** (STAT-HYP-01): returns t-statistic + p-value.
    - Current: no t-test Op.
    - Target: `XEQ "ΣPTST"` against R01–R06 Σ-register state computes t = (x̄ − μ₀)/(s/√n) (μ₀ on stack at call time); p-value via two-sided regularized incomplete beta `beta_regularized_f64` (Req. 32).
    - Acceptance: oracle `data=[1,2,3,4,5], μ₀=3`, t = 0.0, p = 1.0; within 1e-7 relative tolerance.

25. **ΣTSTAT pooled-variance two-sample t-test** (STAT-HYP-02).
    - Current: no two-sample t-test Op.
    - Target: `XEQ "ΣTSTAT"` computes pooled-variance s²_p = ((n₁−1)s₁² + (n₂−1)s₂²)/(n₁+n₂−2), t = (x̄₁−x̄₂)/√(s²_p(1/n₁+1/n₂)), df = n₁+n₂−2 integer. **Welch's t-test is explicitly excluded** per REQUIREMENTS.md Out-of-Scope table.
    - Acceptance: oracle `g1=[1,2,3,4,5], g2=[6,7,8,9,10]` yields t ≈ −5.0, p ≈ 0.0010534; within 1e-7 relative tolerance. **OM override:** if OM 00041-90030 §ΣTSTAT contradicts pooled-variance and specifies Welch instead (highly unlikely per NPS ZS-4/5 + Stat 1 Pac era), it becomes a documented divergence in `docs/hp41-stat1-divergences.md` (Phase 35) — NOT a SPEC.md change.

26. **ΣXSQEV chi-square goodness-of-fit from observed + expected counts** (STAT-HYP-03).
    - Current: no χ² Op.
    - Target: `XEQ "ΣXSQEV"` computes χ² = Σ(O − E)²/E over R01–R06 accumulator state (observed in even registers, expected in odd, or per OM layout from Req. 9).
    - Acceptance: oracle `obs=[10,20,30], exp=[15,20,25]` yields χ² = 2.667; within 1e-9 relative tolerance.

27. **ΣEFXSQ chi-square with expected-as-proportion entry** (STAT-HYP-04).
    - Current: no proportion-entry χ² Op.
    - Target: `XEQ "ΣEFXSQ"` accepts expected probabilities (sum-to-1) per OM "Σ-Register Layout" Req. 9; internally converts to expected counts via Σf.
    - Acceptance: oracle proportions `p=[0.2, 0.3, 0.5]` against observed `[10, 30, 60]` yields χ² ≈ 1.667 within 1e-9 relative tolerance.

28. **ΣCTKKK general r×c contingency χ²** (STAT-HYP-05).
    - Current: no contingency-table χ² Op.
    - Target: `XEQ "ΣCTKKK"` computes χ² over arbitrary r×c marginal totals per OM Req. 9 layout.
    - Acceptance: oracle 2×3 table `[[10,20,30],[40,50,60]]` yields χ² ≈ 4.286; within 1e-9 relative tolerance.

29. **ΣCTKK smaller-table contingency χ²** (STAT-HYP-06).
    - Current: no compact contingency Op.
    - Target: `XEQ "ΣCTKK"` per OM; smaller-table variant of Req. 28.
    - Acceptance: oracle 2×2 table `[[10,20],[30,40]]` yields χ² ≈ 0.397; within 1e-9 relative tolerance.

30. **ΣSPEAR Spearman rank correlation ρ_s** (STAT-HYP-07): closed-form via Σ-block, SIZE 003.
    - Current: no rank-correlation Op.
    - Target: `XEQ "ΣSPEAR"` consumes existing R01–R06 Σ-register state holding ranked pairs; ρ_s = 1 − 6Σd²/(n(n²−1)).
    - Acceptance: oracle `ranks_x=[1,2,3,4,5], ranks_y=[2,1,3,5,4]` yields ρ_s = 0.7; within 1e-9 relative tolerance. Simplest of the 13 programs — closed form only.

### Distribution Evaluators

31. **ΣNORMD three modes** (STAT-DST-01, STAT-DST-02, STAT-DST-03): upper-tail CDF Q(x) = 1 − Φ(x), PDF φ(x), inverse Φ⁻¹(p).
    - Current: no normal-distribution Op.
    - Target: `XEQ "ΣNORMD"` opens a mode dispatcher; modes per OM (tentative: `[E]` = CDF Q(x), `[C]` = PDF φ(x), `[A]` = inverse Φ⁻¹(p)) per OM "Modes" section; inverse uses Acklam/AS 241 plus bisection refinement for tails (Req. 33).
    - Acceptance:
      - Q(1.96) ≈ 0.0250 within 1e-9 (closed-form, `rust_decimal::MathematicalOps::norm_cdf`)
      - φ(0) ≈ 0.3989 within 1e-9
      - Φ⁻¹(0.025) ≈ −1.96 within 1e-7 (iterative)

32. **ΣCHISQD modal-prompt PDF + CDF** (STAT-DST-04, STAT-DST-05): ν entered via ALPHA-prompt `ν=?` per Math Pac I POLY precedent.
    - Current: no χ²-distribution Op.
    - Target: `XEQ "ΣCHISQD"` opens an ALPHA-prompt `ν=?` (reusing the existing `print_buffer` + `modal_program` machinery, identical pattern to POLY's `ORDER=?`); R/S submits ν as an integer; subsequent mode selection (`[C]` = PDF, `[E]` = CDF) evaluates against cached ν using `gamma_regularized_f64` (Req. 32). **OM override:** if OM 00041-90030 §ΣCHISQD shows a different ν-entry convention (e.g. [A]-key non-modal), plan-phase substitutes the OM convention and SPEC.md is amended; the SPEC.md lock is "ν is captured via a deterministic prompt-and-submit workflow that reuses existing modal infrastructure with no new transient CalcState fields."
    - Acceptance:
      - PDF f(7.815; ν=3) within 1e-9 relative tolerance vs scipy.stats.chi2.pdf
      - CDF P(7.815; ν=3) ≈ 0.9500 within 1e-7 relative tolerance vs scipy.stats.chi2.cdf

33. **Three hand-coded f64-bridge distribution primitives** (STAT-DST-06).
    - Current: `rust_decimal::MathematicalOps` provides `norm_cdf`, `norm_pdf`, `erf`; no `norm_cdf_inv`, no `gamma_regularized`, no `beta_regularized`.
    - Target: `hp41-core/src/ops/stat1/distributions.rs` ships three functions:
      - `norm_cdf_inv_f64(p: f64) -> Result<f64, HpError>` — Acklam/AS 241 rational approximation, ~30 LOC
      - `gamma_regularized_f64(s: f64, x: f64) -> Result<f64, HpError>` — AS 239 series + continued-fraction, ~50 LOC
      - `beta_regularized_f64(a: f64, b: f64, x: f64) -> Result<f64, HpError>` — AS 63 continued-fraction, ~60 LOC
      - Total: ~140 LOC. NO `statrs` runtime dependency.
    - Acceptance: each primitive carries ≥ 6 inline-constant `(input, scipy_expected, tolerance)` oracle tuples in `#[cfg(test)] mod tests`; ALL oracle tests pass BEFORE Plan 33-03 ΣNORMD/ΣCHISQD Op code lands. Tolerance per STAT-QUAL-05: 1e-9 closed-form, 1e-7 iterative.

34. **Iterative-quantile convergence + cancellation** (STAT-DST-07, P19, P25): tolerance display-mode-tied, hard iteration cap, `cancel_requested` per-iteration.
    - Current: Math Pac I `INTG` uses `integ_threshold()` = display-mode-tied; no quantile loops exist.
    - Target: ΣNORMD inverse (Newton + bisection hybrid for tails) and ΣCHISQD CDF (continued-fraction inside `gamma_regularized_f64`) use:
      - **Tolerance:** `10^(-FIX_decimals − 1)` (mirrors v3.0 `integ_threshold()` pattern; falls back to `1e-10` when display mode is not FIX)
      - **Iteration cap:** hard stop at 50 iterations, returns `Err(HpError::ConvergenceFailed)` on overflow
      - **Cancellation:** `cancel_requested.load(Relaxed)` check inside every iteration; on `true`, returns `Err(HpError::Cancelled)` and releases the AppState Mutex (Pitfall 11 extended)
    - Acceptance: synthetic test sets `cancel_requested = true` before invoking ΣNORMD inverse and asserts the call returns `Err(HpError::Cancelled)` within < 1 iteration; second test sets FIX 6 display mode and verifies converged-tolerance is `1e-7`.

### RNG Bonus Utility

35. **RAND pseudorandom uniform [0,1)** (STAT-RNG-01).
    - Current: no RNG Op.
    - Target: `XEQ "RAND"` returns `r_{n+1} = FRC(9821 · r_n + 0.211327)` per NPS p. 21 community-LCG formula (Don Malm, HP-65 User's Library); push onto stack.
    - Acceptance: with `state.rand_seed = HpNum::from(0.5)`, the first three RAND calls return `FRC(4910.711327)`, `FRC(9821·that + 0.211327)`, etc. within 1e-15 relative tolerance (decimal-exact, no f64 conversion).

36. **SEED set RNG seed via ALPHA prompt `SEED?`** (STAT-RNG-02).
    - Current: no seeding Op.
    - Target: `XEQ "SEED"` opens an ALPHA prompt `SEED?` via the existing `print_buffer` + `modal_program` machinery; R/S submits the seed; `state.rand_seed` is updated.
    - Acceptance: `XEQ "SEED"` with submitted value `0.5` followed by `XEQ "RAND"` is deterministic; running the same sequence in a second test instance yields identical outputs.

37. **`rand_seed: HpNum` survives save/load** (STAT-RNG-03, P20).
    - Current: no `rand_seed` field.
    - Target: `CalcState::rand_seed: HpNum` with `#[serde(default)]` WITHOUT `#[serde(skip)]`. This is the ONLY new v3.1 CalcState field with this serde shape; all other v3.1 transient fields use `skip`.
    - Acceptance: round-trip test serializes a state with `rand_seed = HpNum::from(0.7)`, deserializes a fresh instance, asserts `state.rand_seed == HpNum::from(0.7)`; subsequent `XEQ "RAND"` produces the same value as before serialization.

38. **RAND/SEED documented as v3.1 emulator extension** (STAT-RNG-04, D-33.4): regardless of OM ROM-presence.
    - Current: no Stat 1 documentation exists.
    - Target: per D-33.4 — if OM 00041-90030 ROM-listing read in Plan 33-08 confirms RAND IS a top-level Stat 1 Pac ROM entry, treat as OM-feature-complete (move out of the divergences "emulator extensions" bucket per D-33.4a); if NOT, document as "v3.1 emulator extension" in `docs/hp41-stat1-divergences.md` (Phase 35) with NPS p. 21 + Don Malm + HP-41C Standard Applications p. 24 as the three-source confirmation. RAND/SEED ship in both cases.
    - Acceptance: Plan 33-08 commit message records the OM read result; `docs/hp41-stat1-divergences.md` (Phase 35) carries the corresponding entry; SPEC.md does NOT block on the OM read outcome.

### Free42 Contamination Guard (D-33.8 reassignment)

39. **Stats-domain identifiers added to contamination-guard pattern** (STAT-QUAL-09 reassigned to Phase 33 per D-33.8).
    - Current: `scripts/check-free42-contamination.sh:28` `PATTERN='phloat|Phloat|bid128_|decNumber|decContext|vartype|arg_struct|prgm_lines|bcd_t|Thomas Okken|AGPL|GNU General Public License'` — math1/-scoped only.
    - Target: Plan 33-00 extends both the PATTERN and the scanned-directory list:
      - **Scanned dirs:** `hp41-core/src/ops/math1` AND `hp41-core/src/ops/stat1` (new sibling)
      - **Pattern symbol classes** (≥ 6 new tokens; final list extracted by Plan 33-00 from `github.com/thomasokken/free42/blob/master/common/core_math2.cc`):
        - (a) BID/Phloat/decNumber statistical-domain extensions (e.g. `math_normal_*`, `math_chi2_*`, `math_t_dist_*` if present in core_math2.cc)
        - (b) Stats-domain Free42 function-name conventions discovered by the Plan-33-00 read
        - (c) Any additional GPL/AGPL header strings missed by the math1-scoped 12-symbol set
    - Acceptance: extended `check-free42-contamination.sh` runs in `just license-audit` AND in `.github/workflows/ci.yml::license-audit`; pattern contains ≥ 18 total tokens (12 existing + ≥ 6 new); script exits 0 on a freshly-created `ops/stat1/mod.rs` (Plan 33-00 deliverable) AND exits 1 when a probe-token (e.g. `math_normal_cdf_xxxxprobe`) is inserted into a temp file inside `ops/stat1/`.

### CalcState Migration Method (D-33.7)

40. **`migrate_after_load()` method on `CalcState`** (P24).
    - Current: no migration logic exists. v3.0 save files with `"xrom_modules": 1` would deserialize unchanged.
    - Target: `impl CalcState { pub fn migrate_after_load(&mut self) { ... } }` added in `hp41-core/src/state.rs`. Body: `if self.xrom_modules & 0b0000_0010 == 0 { self.xrom_modules |= 0b0000_0010; }`. Idempotent.
    - Acceptance: see Req. 4 acceptance; method exists, is callable from CLI + GUI persistence loaders (those wiring sites are Phase 34/36 but the core method must exist in Phase 33).

### File-tree Skeleton (D-33.5)

41. **`hp41-core/src/ops/stat1/` 11-file layout exists end-of-phase**.
    - Current: directory does not exist.
    - Target: end of Phase 33, `hp41-core/src/ops/stat1/` contains:
      - `mod.rs` — module hub, exhaustive `pub use`, OM Σ-register layout transcription as `//!` header comment, `STAT1_MAX_REG: usize` const
      - `distributions.rs` — three hand-coded f64-bridge primitives (Acklam/AS 241, AS 239, AS 63)
      - `basic_stats.rs` — ΣBSTAT / ΣBSTG
      - `moments.rs` — ΣMMTUG / ΣMMTGD
      - `anova.rs` — ΣAOVONE / ΣAOVTWO / ΣANOCOV
      - `regression.rs` — ΣLIN / ΣEXP / ΣLOGI / ΣPOW + ΣMLRXY / ΣMLRXYZ + ΣPOLYP / ΣPOLYC
      - `hypothesis.rs` — ΣPTST / ΣTSTAT (renamed from `tests.rs` to avoid `#[cfg(test)] mod tests` collisions)
      - `nonparam.rs` — ΣSPEAR + ΣXSQEV / ΣEFXSQ + ΣCTKKK / ΣCTKK
      - `normd.rs` — ΣNORMD 3-mode dispatcher
      - `chisqd.rs` — ΣCHISQD ν-prompt dispatcher
      - `rand.rs` — RAND / SEED
    - Acceptance: `ls hp41-core/src/ops/stat1/` lists all 11 files; each file ≤ 300 LOC excluding tests (mirroring math1/ per-file budget).

### math1/ Freeze Exception (D-33.3)

42. **`hp41-core/src/ops/math1/xrom.rs` is the SOLE freeze exception**.
    - Current: `hp41-core/src/ops/math1/` frozen since Plan 25-01.
    - Target: ONLY `xrom.rs` may be edited in Phase 33 — to add `STAT_1` const, `stat1_resolve()` function, and the bit-1 arm in `xrom_resolve()`. All other files in `math1/` (complex.rs, difeq.rs, four.rs, hyperbolics.rs, integ.rs, matrix.rs, mod.rs, modal.rs, poly.rs, solve.rs, trans.rs, tri.rs) MUST NOT be modified.
    - Acceptance: `git diff develop -- hp41-core/src/ops/math1/` post-Phase-33 touches only `xrom.rs`. **CLAUDE.md exception clause added in Phase 35** (STAT-DOC-05) documenting the carve-out per D-33.3a.

### Op-strategy A (carry-over)

43. **One `Op` variant per Stat 1 entry point** (continuing v3.0 ADR-001).
    - Current: Math Pac I uses Op-strategy A.
    - Target: ~24 new `Op` variants — one per program entry point + per-mode dispatcher carriers (ΣNORMD workflow Op + sub-mode Ops, ΣCHISQD workflow Op + sub-mode Ops; final shape decided in plan-phase). NO `Op::XromCall(u16)` table dispatch.
    - Acceptance: `Op` enum gains ~24 named variants; no integer-keyed XROM-call variant exists.

### Resolver-chain shadow-prevention

44. **Mnemonic-shadow CI gate extended to STAT_1.ops** (STAT-QUAL-08 partial; full coverage in Phase 37).
    - Current: `xrom_shadowing.rs` covers MATH_1.ops × built-ins.
    - Target: extend `hp41-core/tests/xrom_shadowing.rs` to assert STAT_1.ops mnemonics are disjoint from both built-ins AND MATH_1.ops. Phase 33 ships the test extension; Phase 37 verifies it remains green with the full Op set.
    - Acceptance: `cargo test --test xrom_shadowing -p hp41-core` passes with STAT_1 + MATH_1 + built-ins loaded.

### scipy.stats oracle data format (D-33.6)

45. **Inline `(input, scipy_expected, tolerance)` tuples in `#[cfg(test)] mod tests`** of `stat1/distributions.rs` and per-Op test modules.
    - Current: math1/poly.rs and math1/integ.rs use the inline-oracle pattern.
    - Target: no external fixture file, no Python toolchain dependency. Each oracle line carries a Python comment above it: `# scipy.stats.norm.ppf(0.025) = -1.959963984540054`. Oracle is frozen at write time.
    - Acceptance: `grep -rn "scipy.stats" hp41-core/src/ops/stat1/ | wc -l` ≥ 30 (oracle comments); `grep -rn "fixtures/" hp41-core/src/ops/stat1/` returns empty (no external fixtures).

### Tolerance discipline (carry-over from STAT-QUAL-05)

46. **Two-level tolerance applied per-Op in Phase-33 tests**.
    - Current: math1/-tests use the two-level pattern (closed-form vs iterative).
    - Target: every Stat 1 acceptance test uses:
      - 1e-9 relative for closed-form ops: ΣNORMD CDF/PDF, ΣSPEAR, ΣBSTAT/BSTG, ΣLIN/EXP/LOGI/POW, ΣXSQEV/EFXSQ, ΣAOVONE
      - 1e-7 relative for iterative ops: ΣNORMD inverse, ΣCHISQD CDF, ΣTSTAT/ΣPTST, ΣMLRXY/ΣMLRXYZ/ΣPOLYP normal-equation solve, ΣAOVTWO/ΣANOCOV (chained cross-products)
    - Acceptance: `lint_stat1_assertions.rs` (or extension of `lint_math1_assertions.rs`, Phase 37) blocks `assert_eq!(decimal, decimal)` on any iterated result; Phase 33 ships only the tolerance discipline IN tests (the lint extension itself is Phase 37 STAT-QUAL-06).

## Boundaries

**In scope (Phase 33 produces):**
- `STAT_1: XromModule` const + `stat1_resolve()` + bit-1 arm in `hp41-core/src/ops/math1/xrom.rs` (sole math1/ freeze exception)
- `default_xrom_modules() = 0b0000_0011` + `CalcState::migrate_after_load()` + `rand_seed: HpNum` field (with `#[serde(default)]` NOT `skip`)
- New `hp41-core/src/ops/stat1/` directory with 11 files (Req. 41) — closed-form + iterative implementations of all 13 QRC programs + RAND/SEED
- Three hand-coded f64-bridge primitives in `stat1/distributions.rs` (Acklam/AS 241, AS 239, AS 63) with inline scipy.stats oracle tests
- 4-way exhaustive-match invariant items 1+2 honored for ~24 new `Op` variants (dispatch + execute_op)
- `scripts/check-free42-contamination.sh` extended for stat1/ + stats-domain symbol classes
- OM "Storage Registers" verbatim transcription in `ops/stat1/mod.rs` as the SINGLE source of truth for register indices
- `hp41-core/tests/xrom_shadowing.rs` extension for STAT_1.ops disjointness

**Out of scope (defers / excludes):**
- **CLI integration** — Phase 34 (op_display_name arms in hp41-cli/src/prgm_display.rs; JSON help pool wiring; `?` overlay "Stat 1 Pac (XROM 2)" section; modal-prompt routing through CLI status bar)
- **Documentation & ADRs** — Phase 35 (`docs/hp41-stat1-divergences.md`, docs-matrix three-input regenerator, ADRs for RNG-state placement + AS-primitives policy, CLAUDE.md `### v3.1 additions` block + math1/ freeze-exception clause, README v3.1 section, `docs/architecture-history.md` v3.1 narrative)
- **GUI integration** — Phase 36 (op_display_name arms in hp41-gui/src-tauri/src/prgm_display.rs; CATALOG 2 enumeration of STAT_1; help overlay "Stat 1 Pac (XROM 2)" section; LCD-alternation modal prompts; `request_cancel` GUI wiring extension)
- **Test hardening / quality gates** — Phase 37 (coverage gates ≥ 95.39 % lines / ≥ 94.26 % regions held; per-file ≥ 90 % floor; numerical_accuracy.rs ≥ 98 % gate with stat1 cases; `lint_stat1_assertions.rs` CI extension; `stat1_op_test_count.rs` ≥ 5/Op floor; backward-compat round-trip tests; E2E ΣNORMD smoke)
- **F-distribution / Binomial / Poisson / Hypergeometric / Histograms / P(N,R) / C(N,R) as XROM entry points** — NPS-confirmed absent from Stat 1 Pac; permanently excluded per REQUIREMENTS.md Out-of-Scope table
- **Welch's t-test (unequal variance)** — REQUIREMENTS.md Out-of-Scope; ΣTSTAT is pooled-variance only (Req. 25)
- **Trimmed mean, median, Mann-Whitney, confidence intervals** — Not in QRC; ZS-4/5 user-program territory per NPS
- **`statrs` runtime crate** — rejected per STACK.md research; AS 239/63/241 hand-coded replaces it
- **Signed binary releases** — deferred to v3.1.x / v3.2 per user direction (memory note "Binary releases — next milestone" remains active)
- **HP-copyrighted ROM-image redistribution** — permanently excluded across all v3.x milestones (behavioral emulation only)
- **Cycle-accurate Nut CPU simulation** — permanently out of scope per v1.0 Key Decision
- **Modification of any file in `hp41-core/src/ops/math1/` other than `xrom.rs`** — frozen since Plan 25-01 (Req. 42)
- **Reuse of Math Pac I `MATRIX/SIMEQ`** — cross-XROM coupling rejected per Req. 21 (Gauss elimination is local to `stat1/regression.rs`)
- **SOLVE/INTG user-callback re-entry for distribution-quantile loops** — locked rejected per CONTEXT.md; quantile loops use self-contained iteration with per-loop `cancel_requested` check (Req. 34)

## Constraints

- **Numerical tolerance two-level discipline** (STAT-QUAL-05): 1e-9 relative for closed-form ops, 1e-7 relative for iterative ops (full list in Req. 46)
- **Iteration cap:** ΣNORMD inverse + ΣCHISQD CDF return `Err(HpError::ConvergenceFailed)` at 50 iterations (Req. 34)
- **Convergence tolerance:** display-mode-tied `10^(-FIX_decimals − 1)` mirroring v3.0 `integ_threshold()` pattern; falls back to `1e-10` when display mode is not FIX (Req. 34)
- **Cancellation:** per-iteration `cancel_requested.load(Relaxed)` check inside every distribution-quantile loop; releases AppState Mutex on cancel (Pitfall 11 extended)
- **No new transient `CalcState` fields beyond `rand_seed`** — modal prompts reuse `print_buffer` + `modal_program` infrastructure (CONTEXT.md D-33.5 / Req. 22, 31, 32, 36)
- **`rand_seed` serde shape is unique:** `#[serde(default)]` without `#[serde(skip)]`. NO other v3.1 field shares this shape (P20 muscle-memory trap)
- **MSRV 1.88** — Stat 1 Pac code must compile on Rust 1.88 (workspace MSRV; CI MSRV job parallel-runs)
- **`#![deny(clippy::unwrap_used)]`** at `hp41-core/src/lib.rs` — `.expect("reason")` or `?`-propagation only; test modules carry `#[allow(clippy::unwrap_used)]`; f64-bridge primitives return `Result<f64, HpError>` instead of panicking
- **No `println!` / `eprintln!`** in `hp41-core` — modal prompts and result strings route through `print_buffer`
- **No external Python toolchain dependency at build/test time** — scipy.stats oracle values are inline-frozen at write time (Req. 45)
- **No reuse of Math Pac I `MATRIX/SIMEQ`** — `ops/stat1/regression.rs` carries its own (d+1)×(d+1) Gauss elimination (Req. 21)
- **OM "Storage Registers" layout is the SINGLE source of truth for register indices** — `ops/stat1/mod.rs` ships first (Plan 33-00) with the OM-transcription header; every register access uses named constants from that file (P21 mitigation; Req. 9, 14)

## Acceptance Criteria

- [ ] `STAT_1: XromModule` const exists with `id = 2`, `name = "STAT 1B"` in `hp41-core/src/ops/math1/xrom.rs`
- [ ] `xrom_resolve("ΣNORMD", 0b0000_0011)` returns `Some(Op::SigmaNormdWorkflow)` (or final variant name) — bit-1 arm fires after MATH_1
- [ ] `xrom_resolve("ΣNORMD", 0b0000_0001)` returns `None` — bit-1 isolation
- [ ] `default_xrom_modules()` returns `0b0000_0011`
- [ ] `CalcState::migrate_after_load()` sets bit 1 unconditionally on stored states with bit 1 clear (idempotent on bit 1 set)
- [ ] A JSON blob containing `"xrom_modules": 1` followed by `migrate_after_load()` yields `state.xrom_modules == 0b0000_0011`
- [ ] All 14 entry-point mnemonics resolve to distinct `Op` variants via `xrom_resolve` (validated by `xrom_shadowing.rs` extension)
- [ ] `hp41-core/src/ops/stat1/mod.rs` exists with OM "Storage Registers" verbatim transcription as `//!` block AND `pub const STAT1_MAX_REG: usize` derived from the OM table
- [ ] `hp41-core/src/ops/stat1/distributions.rs` ships `norm_cdf_inv_f64`, `gamma_regularized_f64`, `beta_regularized_f64` with inline `(input, scipy_expected, tolerance)` oracle tuples (≥ 6 per primitive) and ALL oracle tests pass BEFORE Plan 33-03 Op code lands
- [ ] ΣNORMD CDF Q(1.96) within 1e-9 of `scipy.stats.norm.sf(1.96) ≈ 0.024998`
- [ ] ΣNORMD PDF φ(0) within 1e-9 of `0.3989422804014327`
- [ ] ΣNORMD inverse Φ⁻¹(0.025) within 1e-7 of `-1.959963984540054`
- [ ] ΣCHISQD CDF P(7.815; ν=3) within 1e-7 of `scipy.stats.chi2.cdf(7.815, 3) ≈ 0.9500`
- [ ] ΣAOVONE on 3 groups of 5 samples returns F = 100.0 ± 1e-9 relative (oracle dataset Req. 11)
- [ ] ΣLIN/ΣEXP/ΣLOGI/ΣPOW/ΣSPEAR/ΣBSTAT/ΣXSQEV/ΣEFXSQ pass closed-form acceptance with 1e-9 relative tolerance
- [ ] ΣMLRXY/ΣMLRXYZ/ΣPOLYP/ΣPOLYC/ΣPTST/ΣTSTAT pass iterative acceptance with 1e-7 relative tolerance
- [ ] ΣTSTAT uses pooled variance (`df = n₁+n₂−2` integer; Welch explicitly excluded)
- [ ] `XEQ "RAND"` returns the LCG sequence; `state.rand_seed` survives a serde save/load round trip; second-instance replay yields identical values
- [ ] `grep -n "ops::math1::matrix" hp41-core/src/ops/stat1/` returns empty (no cross-XROM coupling)
- [ ] `grep -n "println!\|eprintln!" hp41-core/src/ops/stat1/` returns empty
- [ ] `git diff develop -- hp41-core/src/ops/math1/` post-Phase-33 touches only `xrom.rs` (math1/ freeze respected)
- [ ] `scripts/check-free42-contamination.sh` PATTERN contains ≥ 18 tokens (12 existing + ≥ 6 new); script scans both `math1/` and `stat1/`; exits 0 on the Plan-33-00 skeleton AND exits 1 when a probe-token is inserted into `stat1/`
- [ ] `cargo check -p hp41-core` succeeds; `cargo clippy -p hp41-core -- -D warnings` clean
- [ ] `hp41-core/tests/xrom_shadowing.rs` asserts STAT_1.ops disjoint from MATH_1.ops AND from built-ins
- [ ] Synthetic test sets `cancel_requested = true` before ΣNORMD inverse and asserts `Err(HpError::Cancelled)` returned within 1 iteration
- [ ] `hp41-core/src/ops/stat1/` contains exactly 11 files (Req. 41); each ≤ 300 LOC excluding tests

## Ambiguity Report

| Dimension          | Score | Min  | Status | Notes                                                                                          |
|--------------------|-------|------|--------|------------------------------------------------------------------------------------------------|
| Goal Clarity       | 0.92  | 0.75 | ✓      | 14 entry points enumerated; 24 Op variants budgeted; 11-file layout locked                     |
| Boundary Clarity   | 0.90  | 0.70 | ✓      | math1/ freeze exception, MATRIX-reuse rejected, Welch excluded, RAND-as-extension locked       |
| Constraint Clarity | 0.85  | 0.65 | ✓      | Two-level tolerance, 50-iter cap, display-mode-tied convergence, cancel_requested per-iter     |
| Acceptance Criteria| 0.85  | 0.70 | ✓      | 25 falsifiable acceptance criteria with concrete scipy.stats oracle values + 1e-9/1e-7 budgets |
| **Ambiguity**      | 0.113 | ≤0.20| ✓      | Below gate; all dimensions exceed minimum                                                      |

## Interview Log

| Round | Perspective              | Question summary                                             | Decision locked                                                                                                                                                              |
|-------|--------------------------|--------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 0     | (scout, pre-interview)   | What exists today? What's missing?                           | XROM framework + Math Pac I shipped v3.0; bit-1 stub at `xrom.rs:134`; `default_xrom_modules() = 0b0000_0001`; no `stat1/`; contamination guard math1/-scoped only            |
| 1     | Researcher               | OM Storage-Registers layout — how does SPEC.md handle it?    | Locked as Plan-33-00 deliverable (P21 single source of truth in `ops/stat1/mod.rs` `//!` header + `STAT1_MAX_REG` const); SPEC.md does NOT inline OM table — Req. 9          |
| 1     | Researcher               | ΣPOLYP degree-prompt wording                                 | Locked tentative `DEGREE=?` per Math Pac I POLY precedent; SPEC.md does NOT lock literal string — only the modal-prompt workflow; OM-divergence is documented divergence — Req. 22 |
| 1     | Researcher               | ΣTSTAT variance assumption                                   | Locked pooled-variance; Welch explicitly excluded per Out-of-Scope; OM-Welch-finding (highly unlikely) becomes documented divergence in Phase 35 — Req. 25                    |
| 2     | Researcher + Simplifier  | ΣCHISQD ν-entry convention                                   | Locked ALPHA-prompt `ν=?` reusing existing `print_buffer` + `modal_program` infrastructure (Math Pac I POLY precedent); no new transient CalcState fields — Req. 32          |
| 2     | Researcher + Simplifier  | Quantile-loop convergence criteria                           | Locked display-mode-tied `10^(-FIX_decimals − 1)` + max 50 iterations + per-iter `cancel_requested` check (Pitfall 11 extended); fallback `1e-10` when not FIX — Req. 34     |
| 2     | Researcher + Simplifier  | Free42 stats-domain identifiers / Plan 33-00 contamination   | Locked symbol-class lock: pattern grows by ≥ 6 tokens covering (a) BID/Phloat/decNumber statistical extensions, (b) `core_math2.cc` stats functions, (c) GPL/AGPL headers — Req. 39 |
| gate  | Seed Closer              | Gate passed (0.113) — proceed?                               | User confirmed "Yes — write SPEC.md"; D-33.1 #5 RAND ROM-presence already pre-locked by CONTEXT.md D-33.4 (RAND/SEED as v3.1 emulator extension regardless) — Req. 38         |

---

*Phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1*
*Spec created: 2026-05-22*
*Next step: /gsd-discuss-phase 33 — implementation decisions already captured in 33-CONTEXT.md (9 decisions); discuss-phase will detect SPEC.md and skip locked-by-spec items.*
*Then: /gsd-plan-phase 33 — produces the 9 plans per D-33.2.*
