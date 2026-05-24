# Requirements: HP-41 Calculator Emulator — v3.1 Stat 1 Pac Emulation

**Defined:** 2026-05-22
**Milestone:** v3.1 Stat 1 Pac Emulation
**Core Value:** Faithful HP-41 RPN fidelity — the four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to the original hardware; everything else is secondary. v3.x extends that fidelity to HP-published XROM application modules.

**Scope locked 2026-05-21:** Behavioral emulation of the HP-41C **Stat 1 Pac** (HP part 00041-14001, OM 00041-90030, QRC 00041-90061, June 1979) as the second XROM application module — XROM ID 2 — mirroring the v3.0 Math Pac I footprint. Authoritative source = the HP Stat 1 Pac Owner's Manual + QRC; community archives (NPS55-84-003, HP Museum, hpcalc.org) as cross-checks; Free42 NEVER copied (CI-gated by `scripts/check-free42-contamination.sh`).

**Anti-features locked per NPS document p. 42/49:** F-distribution, Binomial, Poisson, Hypergeometric, histograms are NOT in Stat 1 Pac and MUST NOT be added (see Out of Scope).

---

## v1 Requirements

Requirements for v3.1 Stat 1 Pac Emulation. Each maps to exactly one phase during roadmap creation.

### XROM Framework Activation (STAT-FW)

- [x] **STAT-FW-01**: `STAT_1` `XromModule` const registered with `id = 2`; `xrom_resolve()` fires LAST in the resolver chain after `MATH_1` (Pitfall 1 — Stat 1 ops never shadow built-in mnemonics or Math Pac I entry points)
- [x] **STAT-FW-02**: `default_xrom_modules()` returns `0b0000_0011` enabling both `MATH_1` and `STAT_1`; v3.0 save files (with `xrom_modules: 1`) migrate at startup to set bit 1 (Pitfall 24 — backward-compat migration; `#[serde(default)]` does not retroactively change stored values)
- [x] **STAT-FW-03**: 4-way exhaustive-match invariant honored for every new `Op` variant — landing in `dispatch()` (`hp41-core/src/ops/mod.rs`) + `execute_op()` (`hp41-core/src/ops/program.rs`) + `op_display_name()` (`hp41-cli/src/prgm_display.rs`) + `op_display_name()` (`hp41-gui/src-tauri/src/prgm_display.rs`) before any caller compiles
- [x] **STAT-FW-04**: All 14 entry points across the 13 QRC programs callable via XEQ-by-name from CLI keyboard, hp41-gui app, and within user programs (`run_program` / `run_loop`); per-call lift effect declared per HP-41 stack-lift semantics

### Univariate Stats (STAT-UNI)

- [x] **STAT-UNI-01**: User can compute extended univariate stats (weighted mean, coefficient of variation σ/μ, etc.) via `XEQ "ΣBSTAT"` and `XEQ "ΣBSTG"` — bivariate summary forms also covered
- [x] **STAT-UNI-02**: User can compute third + fourth central moments + skewness γ₁ = μ₃/σ³ + kurtosis γ₂ = μ₄/σ⁴ − 3 via `XEQ "ΣMMTUG"` (ungrouped) / `XEQ "ΣMMTGD"` (grouped, frequency-weighted)
- [x] **STAT-UNI-03**: ΣMMTUG / ΣMMTGD extend the Σ-register layout per OM "Storage Registers" section; layout transcribed from OM in Phase 33 spec phase before implementation (Pitfall 21 critical path); SIZE-floor guard in `hp41-core/src/ops/stats.rs` raised to cover the new maximum register index
- [x] **STAT-UNI-04**: User-initiated correction via `[C]` label key (undoing the most recent accumulation) works correctly for all univariate accumulation paths

### ANOVA Family (STAT-AOV)

- [x] **STAT-AOV-01**: User can compute one-way ANOVA F-ratio + group means via `XEQ "ΣAOVONE"`
- [x] **STAT-AOV-02**: User can compute two-way ANOVA (no replications) row + column F-ratios via `XEQ "ΣAOVTWO"`
- [x] **STAT-AOV-03**: User can compute one-way ANCOVA F-ratio with covariate adjustment via `XEQ "ΣANOCOV"`
- [x] **STAT-AOV-04**: ANOVA programs respect OM-verified register layout for between-group / within-group accumulators and group-count registers (Pitfall 21 — register layout cannot be guessed; OM verification gates implementation)

### Regression Family (STAT-REG)

- [x] **STAT-REG-01**: User can fit linear curve `ŷ = a + bx` via `XEQ "ΣLIN"`
- [x] **STAT-REG-02**: User can fit exponential curve `ŷ = a·e^(bx)` via `XEQ "ΣEXP"` (accumulates `(x, ln y)`)
- [x] **STAT-REG-03**: User can fit logarithmic curve `ŷ = a + b·ln(x)` via `XEQ "ΣLOGI"` (accumulates `(ln x, y)`)
- [x] **STAT-REG-04**: User can fit power curve `ŷ = a·x^b` via `XEQ "ΣPOW"` (accumulates `(ln x, ln y)`)
- [x] **STAT-REG-05**: User can compute 2-predictor multiple linear regression via `XEQ "ΣMLRXY"` (partial regression coefficients + prediction)
- [x] **STAT-REG-06**: User can compute 3-predictor multiple linear regression via `XEQ "ΣMLRXYZ"`
- [x] **STAT-REG-07**: User can fit polynomial regression via `XEQ "ΣPOLYP"` with OM-confirmed degree-prompt protocol (tentative `DEGREE=?` per Math Pac I `POLY` precedent — verified in Phase 33)
- [x] **STAT-REG-08**: User can predict `ŷ` for a given x via `XEQ "ΣPOLYC"` after polynomial fit
- [x] **STAT-REG-09**: Multiple-regression and polynomial-regression normal equations solved via self-contained 2×2 / 3×3 / (d+1)×(d+1) Gauss elimination in `ops/stat1/` — NOT the Math Pac I MATRIX solver (decoupled to avoid cross-XROM dependency)

### Hypothesis Tests (STAT-HYP)

- [x] **STAT-HYP-01**: User can perform one-sample t-test via `XEQ "ΣPTST"` — returns t-statistic and p-value
- [x] **STAT-HYP-02**: User can perform two-sample t-test via `XEQ "ΣTSTAT"` with OM-confirmed pooled-variance convention (Welch convention rejected if OM specifies pooled — verify in Phase 33 spec phase)
- [x] **STAT-HYP-03**: User can compute χ² goodness-of-fit statistic via `XEQ "ΣXSQEV"` from observed + expected counts
- [x] **STAT-HYP-04**: User can compute χ² with expected-frequency-as-proportion entry path via `XEQ "ΣEFXSQ"`
- [x] **STAT-HYP-05**: User can compute general r×c contingency χ² via `XEQ "ΣCTKKK"`
- [x] **STAT-HYP-06**: User can compute smaller (2×c or comparable) contingency χ² via `XEQ "ΣCTKK"`
- [x] **STAT-HYP-07**: User can compute Spearman's rank correlation ρ_s via `XEQ "ΣSPEAR"` (closed-form via Σ-block, SIZE 003 — simplest of all 13 programs)

### Distribution Evaluators (STAT-DST)

- [x] **STAT-DST-01**: User can evaluate standard normal upper-tail CDF Q(x) = 1 − Φ(x) via `XEQ "ΣNORMD"` mode `[E]` (HP-41 Q(x) convention per OM + NPS p. 33)
- [x] **STAT-DST-02**: User can evaluate standard normal PDF φ(x) via `XEQ "ΣNORMD"` mode `[C]`
- [x] **STAT-DST-03**: User can compute inverse normal quantile Φ⁻¹(p) via `XEQ "ΣNORMD"` mode `[A]` — Acklam/AS 241 rational approximation with bisection refinement for tails
- [x] **STAT-DST-04**: User can evaluate chi-square PDF f(x; ν) via `XEQ "ΣCHISQD"` mode `[C]` with OM-confirmed ν entry protocol (tentative: ν entered via `[A]` before x evaluation — verified in Phase 33)
- [x] **STAT-DST-05**: User can evaluate chi-square CDF P(x; ν) via `XEQ "ΣCHISQD"` mode `[E]` — regularized lower incomplete gamma function
- [x] **STAT-DST-06**: `hp41-core/src/ops/stat1/distributions.rs` provides three hand-coded f64-bridge primitives: `norm_cdf_inv_f64` (Acklam/AS 241, ~30 lines), `gamma_regularized_f64` (AS 239 series + continued-fraction, ~50 lines for ΣCHISQD), `beta_regularized_f64` (AS 63, ~60 lines for ΣTSTAT) — total ~140 LOC; built and validated against scipy.stats oracle BEFORE any program `Op` is implemented
- [x] **STAT-DST-07**: Iterative-quantile paths honor `request_cancel` (per-loop AtomicBool check) and use display-mode-tied convergence threshold (10^(-decimals - 1)) — same termination shape as Math Pac I `INTG` / `SOLVE` (Pitfall 19 — Newton + bisection hybrid for tail probabilities p near 0 or 1)

### RNG Bonus Utility (STAT-RNG)

- [x] **STAT-RNG-01**: User can generate next pseudorandom uniform in [0, 1) via `XEQ "RAND"` using the HP-41 community formula `r_{n+1} = FRC(9821 · r_n + 0.211327)` (confirmed by NPS p. 21; attributed to Don Malm, HP-65 User's Library, referenced by HP-41C Standard Applications manual p. 24)
- [x] **STAT-RNG-02**: User can seed the generator via `XEQ "SEED"` with ALPHA prompt `SEED?` (confirmed by NPS p. 22 ZP4 program convention)
- [x] **STAT-RNG-03**: `rand_seed: HpNum` field on `CalcState` uses `#[serde(default)]` WITHOUT `#[serde(skip)]` — survives save/load so reproducible simulations work across sessions (Pitfall 20 — the ONLY new v3.1 `CalcState` field with this serde shape; all other XROM transient fields use `skip`)
- [x] **STAT-RNG-04**: README + `docs/hp41-stat1-divergences.md` document RAND/SEED as a v3.1 emulator extension — NOT part of the "feature-complete per OM 00041-90030" claim (the OM does not list a standalone RAND/SEED XROM entry point per current research; this is community convention)

### CLI Integration (STAT-CLI)

- [x] **STAT-CLI-01**: `xeq_by_name_local_resolve` (in `hp41-cli/src/keys.rs`) falls through to the existing `xrom_resolve` — no CLI code change beyond confirming the fall-through still fires AFTER built-in resolution (Pitfall 1 invariant preserved)
- [x] **STAT-CLI-02**: `docs/hp41-stat1-functions.json` loaded via `include_str!` into a third `OnceLock<Vec<HelpEntry>>` in `hp41-cli/src/help_data.rs`; `help_entries_all()` chains all three pools (`hp41cv-functions.json` + `hp41-math1-functions.json` + `hp41-stat1-functions.json`); malformed JSON panics at first access (hard build-blocker by design, matching Math Pac I pattern)
- [x] **STAT-CLI-03**: ~24 new `op_display_name` arms land in `hp41-cli/src/prgm_display.rs` — exhaustive match preserved, NO `_ =>` catch-all (4-way invariant item 3)
- [x] **STAT-CLI-04**: `?` help overlay surfaces a "Stat 1 Pac (XROM 2)" section parallel to the existing "Math 1 Pac (XROM 7)" section; search treats both XROM sections as first-class
- [x] **STAT-CLI-05**: Modal-prompt routing for Stat 1 Pac multi-step workflows (e.g. degree prompt for ΣPOLYP, seed prompt for SEED) reuses existing `print_buffer` + `modal_program` infrastructure — no new transient `CalcState` fields required beyond `rand_seed` (STAT-RNG-03)

### GUI Integration (STAT-GUI)

- [ ] **STAT-GUI-01**: `hp41-gui/src-tauri/src/prgm_display.rs` gains the same ~24 new `op_display_name` arms — SC-4 (no core duplication in GUI) preserved (4-way invariant item 4)
- [ ] **STAT-GUI-02**: CATALOG 2 XROM enumeration discovers and displays `STAT_1` alongside `MATH_1` — both XROM module IDs visible to the user from within the calculator
- [ ] **STAT-GUI-03**: `?` keyboard-shortcut overlay parallel-load adds a third JSON-import section "Stat 1 Pac (XROM 2)" — search across all three XROM sections + built-ins
- [ ] **STAT-GUI-04**: LCD-alternation modal prompts (e.g. `SEED?` for RAND/SEED) reuse the existing modal-prompt infrastructure with the v2.1 frontend-only one-shot SHIFT (`shiftActive`) untouched
- [ ] **STAT-GUI-05**: `request_cancel` cancellation channel (introduced in v3.0 Phase 31) reused for iterative-quantile paths in ΣNORMD inverse + ΣCHISQD CDF — per-loop AtomicBool check + lock release (Pitfall 11 mitigation extended to Stat 1) -- reassessed Phase 36 planning: bounded 50-iter primitives (Acklam closed-form / gser+gcf with ITER_CAP=50) do not need cancellation; deferred to Phase 37 STAT-QUAL block per 36-CONTEXT D-36.2

### Documentation (STAT-DOC)

- [x] **STAT-DOC-01**: `docs/hp41-stat1-divergences.md` written — three-bucket catalog (OM divergences / emulator extensions including RAND/SEED + the XROM-7 vs XROM-2 mnemonic-prefix convention / behavioral policies); parallel to `docs/hp41-math1-divergences.md`
- [x] **STAT-DOC-02**: `scripts/docs-matrix/` extended to three-input (cv + math1 + stat1) regenerator; `just docs-matrix` rebuilds all three function matrices; `just docs-matrix-check` CI drift gate covers all three
- [x] **STAT-DOC-03**: `docs/hp41-stat1-function-matrix.md` regenerated from canonical `docs/hp41-stat1-functions.json` via `just docs-matrix`
- [x] **STAT-DOC-04**: ADRs written for any new architectural decisions in Phase 33 — at minimum: (a) RNG-state placement (CalcState field with non-skip serde), (b) ANOVA / multiple-regression register-layout finalization, (c) hand-coded distribution primitives policy (statrs rejected, AS 239/63/241 chosen)
- [x] **STAT-DOC-05**: README v3.1 section + CLAUDE.md `### v3.1 additions` block describe Stat 1 Pac integration, RNG bonus utility scope, and updated Op-variant count
- [x] **STAT-DOC-06**: `docs/architecture-history.md` expanded with v3.1 phase narrative + decision rationale (per the v3.0 history-pattern); milestone-summary one-liner in `.planning/MILESTONES.md` follows v3.0 shape

### Quality Gates (STAT-QUAL)

- [ ] **STAT-QUAL-01**: `hp41-core` line coverage ≥ 95.39 % (no regression vs v3.0 baseline)
- [ ] **STAT-QUAL-02**: `hp41-core` region coverage ≥ 94.26 % (no regression vs v3.0 baseline)
- [ ] **STAT-QUAL-03**: Per-file `hp41-core/src/ops/stat1/*.rs` coverage floor ≥ 90 % (mirrors math1 per-file policy)
- [ ] **STAT-QUAL-04**: `numerical_accuracy.rs` extended with Stat 1 Pac cases at ≥ 98 % pass rate; v1.x 503-case baseline floor 498/503 preserved AND v3.0 768-case floor 763/768 preserved
- [ ] **STAT-QUAL-05**: Two-level tolerance discipline — 1e-9 relative for closed-form ops (ΣNORMD CDF/PDF, ΣSPEAR, ΣBSTAT/BSTG, ΣLIN/EXP/LOGI/POW, ΣXSQEV/EFXSQ); 1e-7 relative for iterative paths (probit Φ⁻¹, regularized incomplete gamma/beta, t-CDF, multiple-regression normal-equation solve)
- [ ] **STAT-QUAL-06**: `lint_stat1_assertions.rs` (or extension of `lint_math1_assertions.rs`) blocks `assert_eq!(decimal, decimal)` on iterated results — Pitfall 14 / 17 discipline extended to Stat 1
- [ ] **STAT-QUAL-07**: `stat1_op_test_count.rs` analog of `math1_op_test_count.rs` enforces ≥ 5 tests per Stat 1 Op variant at CI time (Pitfall 16)
- [ ] **STAT-QUAL-08**: `xrom_shadowing.rs` gate extended to cross-check `STAT_1.ops` against an OM-derived allowlist — no Math Pac I mnemonic shadowed, no built-in mnemonic shadowed
- [ ] **STAT-QUAL-09**: `scripts/check-free42-contamination.sh` extended with stats-domain identifiers (Free42's `core_math2.cc` identifiers + decNumber stats primitives) — Pitfall 27 mitigation; verified BEFORE first stat1/*.rs file lands
- [ ] **STAT-QUAL-10**: Backward-compat — all v1.0 / v1.1 / v2.0 / v2.1 / v2.2 / v3.0 save files load into v3.1 without error; `xrom_modules` startup migration sets bit 1 on stored states with `0b0000_0001` (Pitfall 24)
- [ ] **STAT-QUAL-11**: E2E smoke (`ci-gui.yml::e2e-linux`) extended with at least one Stat 1 Pac workflow on Ubuntu — candidate: `ΣNORMD` upper-tail CDF evaluation of x = 1.96 (expected Q ≈ 0.0250) or `ΣSPEAR` rank correlation on a small fixed dataset

---

## v2 Requirements (deferred)

Tracked but explicitly NOT in this milestone's roadmap.

### Signed Binary Releases (STAT-BIN — deferred to v3.1.x or v3.2)

- **STAT-BIN-01**: cargo-dist wired for `hp41-cli` signed cross-platform binaries (Win10+ / macOS 12+ / Ubuntu 22.04+)
- **STAT-BIN-02**: tauri-action wired for `hp41-gui` signed bundles using Apple Developer cert ($99/yr already procured)
- **STAT-BIN-03**: Release-workflow gate on green `ci.yml` + `ci-gui.yml` + tag push
- **STAT-BIN-04**: SHA-256 manifests published per release artifact

### Time Pac (v3.2)

- **TIME-XX**: HP-41CX clock functions (XROM TBD) — separate milestone

### Advanced Matrix Pac (v3.2+)

- **MATX-XX**: M+ / MAT* / INV-as-transpose / V+ / VDOT / IDN — separate milestone

### Advantage Pac (v3.3+)

- **ADVT-XX**: PROOT / CABS / CARG / CCHS / CCONJ / Romberg-INTG / CY^X — separate milestone

---

## Out of Scope

Explicitly excluded from v3.1. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| F-distribution CDF/quantile | NPS document p. 49: "Several key programs missing [from STAT PAC], such as the t and F distributions" — F is NOT in Stat 1 Pac; emulating it would be incorrect behavioral emulation |
| Binomial PMF/CDF | NPS document p. 42: "there is no binomial program in STAT PAC"; absent from QRC |
| Poisson PMF/CDF | Not in QRC; not documented in any HP Stat 1 Pac source |
| Hypergeometric distribution | Not in QRC |
| Exponential / Uniform / Gamma / Beta distributions (standalone evaluators) | Not in QRC — gamma and beta primitives exist only as private helpers for ΣCHISQD and ΣTSTAT, not as user-callable XEQ entry points |
| Histograms / frequency tables | Not in QRC; NPS document treats histograms as user-program territory (ST-07/9), not Stat 1 Pac |
| Trimmed mean / median | Not in QRC; not documented in any HP Stat 1 Pac source |
| Confidence intervals (μ, σ, slope, intercept) | NPS confirms CI computation is user-program territory using Stat Pac distribution outputs + quantile values (ZS-4/5 pattern) |
| Welch's t-test (unequal variance) | Not in QRC; ΣTSTAT assumed pooled-variance per OM (verify Phase 33) |
| Mann-Whitney / non-parametric tests beyond Spearman | Not in QRC |
| Permutations P(N,R) and Combinations C(N,R) as XROM entry points | NPS p. 8 confirms these are user programs (ZP2), not STAT PAC entry points; `FACT` already in hp41-core covers n! |
| Signed binary releases | Deferred to v3.1.x or v3.2 per user direction 2026-05-21 (memory note "Binary releases — next milestone" remains active for a future cycle) |
| `statrs` 0.18.0 runtime dependency | Pulls `approx 0.5.0` into runtime deps; f64-only interface forces conversion shims; risks last-digit OM divergence vs hand-rolled AS 239/63/241 implementations |
| HP-copyrighted ROM-image redistribution | Permanently excluded — legal risk; behavioral emulation only across all v3.x milestones |
| Stat 2 Pac functions (extended ANOVA, regression diagnostics) | Separate HP module; if pursued, separate future milestone |
| Cycle-accurate Nut CPU simulation | Permanently out of scope per v1.0 Key Decision — high effort, low user value vs. behavioral emulation |

---

## Traceability

Which phases cover which requirements. Filled by `gsd-roadmapper` during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| STAT-FW-01 | Phase 33 | Complete |
| STAT-FW-02 | Phase 33 | Complete |
| STAT-FW-03 | Phase 33 | Complete |
| STAT-FW-04 | Phase 33 | Complete |
| STAT-UNI-01 | Phase 33 | Complete |
| STAT-UNI-02 | Phase 33 | Complete |
| STAT-UNI-03 | Phase 33 | Complete |
| STAT-UNI-04 | Phase 33 | Complete |
| STAT-AOV-01 | Phase 33 | Complete |
| STAT-AOV-02 | Phase 33 | Complete |
| STAT-AOV-03 | Phase 33 | Complete |
| STAT-AOV-04 | Phase 33 | Complete |
| STAT-REG-01 | Phase 33 | Complete |
| STAT-REG-02 | Phase 33 | Complete |
| STAT-REG-03 | Phase 33 | Complete |
| STAT-REG-04 | Phase 33 | Complete |
| STAT-REG-05 | Phase 33 | Complete |
| STAT-REG-06 | Phase 33 | Complete |
| STAT-REG-07 | Phase 33 | Complete |
| STAT-REG-08 | Phase 33 | Complete |
| STAT-REG-09 | Phase 33 | Complete |
| STAT-HYP-01 | Phase 33 | Complete |
| STAT-HYP-02 | Phase 33 | Complete |
| STAT-HYP-03 | Phase 33 | Complete |
| STAT-HYP-04 | Phase 33 | Complete |
| STAT-HYP-05 | Phase 33 | Complete |
| STAT-HYP-06 | Phase 33 | Complete |
| STAT-HYP-07 | Phase 33 | Complete |
| STAT-DST-01 | Phase 33 | Complete |
| STAT-DST-02 | Phase 33 | Complete |
| STAT-DST-03 | Phase 33 | Complete |
| STAT-DST-04 | Phase 33 | Complete |
| STAT-DST-05 | Phase 33 | Complete |
| STAT-DST-06 | Phase 33 | Complete |
| STAT-DST-07 | Phase 33 | Complete |
| STAT-RNG-01 | Phase 33 | Complete |
| STAT-RNG-02 | Phase 33 | Complete |
| STAT-RNG-03 | Phase 33 | Complete |
| STAT-RNG-04 | Phase 33 | Complete |
| STAT-CLI-01 | Phase 34 | Complete |
| STAT-CLI-02 | Phase 34 | Complete |
| STAT-CLI-03 | Phase 34 | Complete |
| STAT-CLI-04 | Phase 34 | Complete |
| STAT-CLI-05 | Phase 34 | Complete |
| STAT-GUI-01 | Phase 36 | Pending |
| STAT-GUI-02 | Phase 36 | Pending |
| STAT-GUI-03 | Phase 36 | Pending |
| STAT-GUI-04 | Phase 36 | Pending |
| STAT-GUI-05 | Phase 37 | Pending |
| STAT-DOC-01 | Phase 35 | Complete |
| STAT-DOC-02 | Phase 35 | Complete |
| STAT-DOC-03 | Phase 35 | Complete |
| STAT-DOC-04 | Phase 35 | Complete |
| STAT-DOC-05 | Phase 35 | Complete |
| STAT-DOC-06 | Phase 35 | Complete |
| STAT-QUAL-01 | Phase 37 | Pending |
| STAT-QUAL-02 | Phase 37 | Pending |
| STAT-QUAL-03 | Phase 37 | Pending |
| STAT-QUAL-04 | Phase 37 | Pending |
| STAT-QUAL-05 | Phase 37 | Pending |
| STAT-QUAL-06 | Phase 37 | Pending |
| STAT-QUAL-07 | Phase 37 | Pending |
| STAT-QUAL-08 | Phase 37 | Pending |
| STAT-QUAL-09 | Phase 37 | Pending |
| STAT-QUAL-10 | Phase 37 | Pending |
| STAT-QUAL-11 | Phase 37 | Pending |

**Coverage:**
- v1 requirements: 66 total
- Mapped to phases: 66 ✓
- Unmapped: 0 ✓

| Phase | Requirements mapped | Count |
|-------|---------------------|-------|
| Phase 33 | STAT-FW-01..04, STAT-UNI-01..04, STAT-AOV-01..04, STAT-REG-01..09, STAT-HYP-01..07, STAT-DST-01..07, STAT-RNG-01..04 | 39 |
| Phase 34 | STAT-CLI-01..05 | 5 |
| Phase 35 | STAT-DOC-01..06 | 6 |
| Phase 36 | STAT-GUI-01..04 | 4 |
| Phase 37 | STAT-QUAL-01..11, STAT-GUI-05 | 12 |
| **Total** | | **66** |

---

*Requirements defined: 2026-05-22*
*Last updated: 2026-05-24 — STAT-GUI-05 reassigned Phase 36 → Phase 37 per 36-CONTEXT D-36.2 (bounded 50-iter distribution primitives do not need cancellation at microsecond timescale)*
