# Project Research Summary

**Project:** HP-41 Calculator Emulator — v3.1 Stat 1 Pac Emulation
**Domain:** Behavioral emulation of second HP-41 XROM application module (HP Stat 1 Pac, part 00041-14001)
**Researched:** 2026-05-21
**Confidence:** HIGH

## Executive Summary

The HP-41 Stat 1 Pac (XROM ID 2, OM 00041-90030, June 1979) is a FOCAL user-code ROM delivering 13 named programs across ~24 Op variants. Like Math Pac I, it runs as FOCAL-level behavioral emulation — not M-code emulation. The v3.0 XROM framework was explicitly designed as reusable infrastructure for this second module: the `xrom_resolve()` bit-1 stub comment, `XromModule` struct, and resolver-last chain are all already in place. The recommended approach is direct activation of the reserved bit-1 arm, followed by building a new `hp41-core/src/ops/stat1/` module tree that mirrors `ops/math1/` in structure, then propagating the 4-way exhaustive-match invariant through CLI and GUI exactly as Phases 28–31 did for Math Pac I.

The numerical surface is larger than Math Pac I in one dimension — three hand-coded f64-bridge functions are needed (`norm_cdf_inv_f64` via Acklam/AS 241, `gamma_regularized_f64` via AS 239, `beta_regularized_f64` via AS 63) — but zero new runtime dependencies are required. `rust_decimal 1.42` already covers normal CDF/PDF, log, exp, sqrt, and trig. The `statrs` crate was evaluated and rejected: its mandatory `approx` dep would land in the production graph, its f64-only interface requires a conversion shim as long as the hand-coded alternative, and its modern Lanczos-gamma algorithm may diverge from the 1979-era Abramowitz and Stegun approximations the OM specifies. The ~850 LOC estimate across 9 new files is narrower than Math Pac I's scope (~1,500–2,000 LOC), consistent with Stat 1's simpler workflow structure — no complex numbers, no matrix overlay mode, no ODE solver.

The primary risks are operational discipline, not algorithmic difficulty. P21 (Σ-register layout must be read from the OM "Storage Registers" section before any ANOVA or higher-moment Op is implemented) is the single most dangerous silent-wrong-answer trap. P24 (the `default_xrom_modules` default must change to `0b0000_0011` with a startup migration for v3.0 save files) is a backward-compat wrinkle with a clear fix. P20 (RNG seed must use `#[serde(default)]` without `skip`) is the exception to the normal transient-field pattern and easy to get wrong by muscle memory. The anti-feature list is firmly established: F-distribution, Binomial, Poisson, Hypergeometric, and histogram programs are explicitly absent from the Stat 1 Pac per the NPS document — adding them would be behavioral incorrectness, not completeness.

## Key Findings

### Recommended Stack

Zero new runtime dependencies in `hp41-core`. The three distribution-function primitives that `rust_decimal 1.42` does not provide are hand-coded at ~140 LOC in `ops/stat1/distributions.rs` using the established f64-bridge pattern from `num.rs`. The workspace MSRV (1.88), the full dependency set, and both CI workflows are unchanged.

**Core technologies:**
- `rust_decimal 1.42`: Already covers normal CDF/PDF (`norm_cdf()`, `norm_pdf()`), erf, log, exp, sqrt, trig — direct use for most Stat 1 ops; no gaps for closed-form CDF/PDF paths
- **Hand-coded f64-bridge** (Acklam/AS 241, AS 239, AS 63): ~140 LOC for inverse normal, incomplete gamma, incomplete beta; same pattern as `checked_asin`/`checked_acos`/`checked_atan` in `num.rs`
- **HP-41 LCG RNG** (`FRC(9821·x + 0.211327)`): 5-line deterministic implementation; no external crate; seed stored in designated data register per OM
- `approx 0.5.1` (dev-dep only): Already in `hp41-core/Cargo.toml` line 17 — no addition needed
- **Rejected:** `statrs 0.18.0` (mandatory `approx` production dep + f64 shim overhead + algorithmic fidelity risk vs. 1979-era OM approximations); `rand`/`rand_distr` (5-line LCG makes them unnecessary)

### Expected Features

Confirmed from QRC 00041-90061 (read directly as 2-page PDF) and NPS document NPS55-84-003 (authoritative cross-reference confirming anti-features).

**Must have — table stakes for "OM 00041-90030 feature-complete" claim (~24 Op variants, 14 entry points):**
- ΣBSTAT / ΣBSTG — extended univariate/bivariate summary from Σ-registers (LOW complexity)
- ΣMMTUG / ΣMMTGD — 3rd/4th central moments, skewness, kurtosis; needs Σx³/Σx⁴ registers (MEDIUM)
- ΣAOVONE / ΣAOVTWO / ΣANOCOV — one-way, two-way, covariance ANOVA; most register-intensive programs (MEDIUM to MEDIUM-HIGH)
- ΣLIN / ΣEXP / ΣLOGI / ΣPOW — 4 curve-fitting transforms on Σ-registers; delegates to existing `op_sigma_plus()` (LOW-MEDIUM)
- ΣMLRXY / ΣMLRXYZ — 2- and 3-variable multiple regression; self-contained Gauss-Jordan 2×2/3×3 solve (MEDIUM-HIGH)
- ΣPOLYP / ΣPOLYC — polynomial regression fit and prediction; degree prompt pending OM verification (MEDIUM-HIGH)
- ΣPTST / ΣTSTAT — one-sample and two-sample t-tests; CDF via `beta_regularized_f64` (MEDIUM)
- ΣXSQEV / ΣEFXSQ — chi-square goodness-of-fit accumulator; 8 program steps; no distribution function (LOW)
- ΣCTKKK / ΣCTKK — r×c contingency table chi-square; needs marginal-total registers (MEDIUM)
- ΣSPEAR — Spearman rank correlation; only 3 program steps; pure Σ-register arithmetic (LOW)
- ΣNORMD — normal CDF/PDF/inverse; three label-key dispatch modes; CDF/PDF are direct `rust_decimal` calls (LOW-MEDIUM)
- ΣCHISQD — chi-square CDF/PDF; `gamma_regularized_f64` required; ν storage convention from OM (LOW-MEDIUM)

**Should have — bonus pending OM verification:**
- RAND/RNDMU subroutine — QRC does not list it as a top-level program; NPS confirms the formula; implement only if OM listing shows it as a named callable sub

**Explicitly defer — anti-features confirmed absent from Stat 1 Pac:**
- F-distribution evaluator — NPS (p. 49): "missing from STAT PAC"
- Binomial / Poisson / Hypergeometric distributions — NPS (p. 42): "no binomial program in STAT PAC"
- Histogram / frequency-table program — not in QRC; NPS uses standalone user programs for this
- P(N,R) / C(N,R) permutations/combinations — user programs; built-in `FACT` covers n!
- Confidence interval computations — user combines ΣTSTAT output with quantile lookup
- Welch t-test, Mann-Whitney, non-parametric tests beyond Spearman — not in QRC

### Architecture Approach

Stat 1 slots into the v3.0 framework with surgical precision. No framework infrastructure changes are needed beyond activating the already-reserved bit-1 arm, adding the `STAT_1: XromModule` const, and building the new `ops/stat1/` module tree. The XROM resolver chain, modal machinery, user-callback infrastructure, `request_cancel` channel, JSON pipeline, and 4-way exhaustive-match invariant all reuse unchanged. Distribution quantile functions iterate privately (the POLY pattern, not the SOLVE/INTG user-callback pattern), so no new `*_state` CalcState fields are required. Likely zero net-new `CalcState` fields: all state lives in existing data registers and the existing transient `modal_program` field.

**Major components:**
1. `hp41-core/src/ops/stat1/` — 9-file module tree (~850 LOC): `mod.rs`, `distributions.rs` (3 special functions), `basic_stats.rs`, `anova.rs`, `regression.rs`, `tests.rs`, `nonparam.rs`, `normd.rs`, `chisqd.rs`
2. `hp41-core/src/ops/math1/xrom.rs` — surgical extension only: `STAT_1` const + `stat1_resolve()` + bit-1 arm in `xrom_resolve()`; rest of `math1/` remains frozen per CLAUDE.md invariant
3. `docs/hp41-stat1-functions.json` — third sibling JSON file per D-29.1 precedent; drives CLI help-overlay third section, docs-matrix output, and existing JSON parity CI tests automatically
4. `scripts/docs-matrix/src/main.rs` + `justfile` — minimal extension: one new title-dispatch branch, two new `just` recipes (`docs-stat1`, `docs-stat1-check`)
5. `hp41-cli/src/help_data.rs` + both `prgm_display.rs` files — third `OnceLock` (`STAT1_HELP_ENTRIES`) and new Op variant arms completing the 4-way match

**Key patterns to follow (established in v3.0):**
- **XROM registration pattern**: `pub const STAT_1: XromModule` with `ops: &[(&str, Op)]` slice driving both resolver and `xrom_shadowing.rs` CI test
- **Self-contained iteration pattern** (POLY, not SOLVE/INTG): distribution quantile loops check `state.cancel_requested` every 64 iterations; no user callback, no new CalcState scratch fields
- **Σ-register delegate pattern**: curve fitting transforms x/y then calls `op_sigma_plus()` — never duplicates Σ arithmetic from `ops/stats.rs`

### Critical Pitfalls

1. **P21: Σ-register layout — read OM before writing any ANOVA/moment/regression code** — The register layout for ΣMMTUG (Σx³, Σx⁴ storage), ΣAOVONE/ΣAOVTWO/ΣANOCOV (group sums), ΣMLRXY (cross-products), and ΣCTKKK (marginal totals) must be transcribed from the Stat 1 Pac OM "Storage Registers" section before Phase 33 begins. Hard-coding a guessed layout produces silently wrong results. Add `STAT1_MAX_REG: usize` as a `pub const` derived from the OM table and guard every extended-register access with fail-closed `state.regs.len() < STAT1_MAX_REG + 1`.

2. **P24: `xrom_modules` default migration** — `default_xrom_modules()` must change from `0b0000_0001` to `0b0000_0011` in Phase 33 Phase 0. V3.0 save files that explicitly stored `xrom_modules: 1` will deserialize with Stat 1 disabled (serde uses stored value, not new default). Add a startup migration in the persistence layer: if `xrom_modules & 0b0000_0010 == 0` after deserialization, set the bit and re-save. Add serde round-trip test with a v3.0 fixture.

3. **P20: RNG serde — `#[serde(default)]` NOT skip** — If RAND is implemented, `rand_seed` must survive save/load for reproducible simulations. The `#[serde(default, skip)]` pattern used for all other transient fields is wrong for RNG state. Add a `tests/stat1_rand_determinism.rs` round-trip test. Also: `hp41-core` must never import `rand` or `getrandom` — add both to the Free42 contamination guard's blocked-imports list.

4. **P19: Quantile inversion — initial guess and convergence from OM before writing loops** — Inverse normal fails near tails (p ≤ 0.001 or p ≥ 0.999) with x₀ = 0 starting guess. Use A&S §26.2.17 rational approximation as starting point. Tie convergence threshold to `state.display_mode` exactly as `integ_threshold(mode)` does. Add bisection fallback when Newton step leaves valid domain. Cap at 50 Newton + 50 bisection steps before returning `HpError::Domain`.

5. **P22: Mnemonic shadowing — no STAT_1 mnemonic may duplicate a v2.2 built-in** — `MEAN`, `SDEV`, `CORR`, `LR`, `YHAT` are reserved v2.2 built-in names; the resolver gives built-ins priority. Confirm every `STAT_1.ops` mnemonic against `docs/hp41cv-functions.json` before registering in `xrom.rs`. Extend `tests/xrom_shadowing.rs` to cover all `STAT_1.ops` entries — a 5-line addition.

## Implications for Roadmap

Based on combined research, a 5-phase structure starting at Phase 33 mirrors the v3.0 Math Pac I pattern (Phases 28–32) exactly. The critical dependency ordering: `distributions.rs` must be built and validated first (prerequisite for ΣNORMD, ΣCHISQD, ΣPTST/ΣTSTAT), and the OM register layout (P21) must be locked before ANOVA, higher-moment, multiple-regression, and contingency-table Ops are written.

### Phase 33: hp41-core — Framework Extension and Core Ops

**Rationale:** The three hand-coded distribution primitives are the deepest dependency. XROM framework activation (bit-1, `STAT_1` const, `default_xrom_modules` update to `0b0000_0011`) is a prerequisite for all subsequent phases. P21 register-layout resolution is the single most dangerous silent-wrong-answer risk and must precede any ANOVA/moment/regression Op implementation.
**Delivers:** All ~24 Op variants in `hp41-core`, distribution primitives validated against scipy.stats oracle, `xrom_resolve` bit-1 arm active, `CalcState.xrom_modules` default `0b0000_0011`, `docs/hp41-stat1-divergences.md` stub with D-33-NN entries, 4-way match items 1+2 complete (dispatch + execute_op), Free42 contamination guard extended with stats-domain identifiers
**Implementation order within phase:** (1) read OM "Storage Registers" + transcribe layout into `stats1/mod.rs` header; (2) `stat1/distributions.rs` + scipy.stats validation; (3) ΣNORMD + ΣCHISQD; (4) ΣSPEAR + ΣXSQEV/ΣEFXSQ; (5) ΣBSTAT/ΣBSTG + ΣLIN/ΣEXP/ΣLOGI/ΣPOW; (6) ΣMMTUG/ΣMMTGD + ANOVA family + ΣCTKKK/ΣCTKK; (7) ΣPTST/ΣTSTAT; (8) ΣMLRXY/ΣMLRXYZ + ΣPOLYP/ΣPOLYC; (9) RAND if OM confirms
**Avoids:** P18 (Welford for raw-block variance ops), P19 (convergence from OM), P20 (RNG serde annotation locked), P21 (OM layout first), P22 (mnemonic check before xrom.rs registration), P25 (cancel_requested in every iterative loop), P27 (contamination guard updated before first stats1 source file is written)
**Research flag:** NEEDS `/gsd-plan-phase --research-phase 33` — OM 00041-90030 "Storage Registers" section, RAND subroutine presence, quantile convergence criteria, ΣPOLYP degree prompt, ΣCHISQD ν storage convention, ΣTSTAT pooled vs. Welch assumption — all must be resolved before Op implementation begins.

### Phase 34: hp41-cli — CLI Integration

**Rationale:** After all Op variants land in hp41-core (4-way invariant items 1+2), CLI requires item 3. The `xeq_by_name_local_resolve` path already routes through `xrom_resolve` — no resolver changes. `docs/hp41-stat1-functions.json` must exist before the `OnceLock` is wired in `help_data.rs`.
**Delivers:** 4-way invariant item 3 complete, third help-overlay section ("Stat 1 Pac (XROM 2)"), `docs/hp41-stat1-functions.json` canonical source, `xrom_shadowing.rs` extended to STAT_1.ops, `function_matrix_parity.rs` and `key_coverage.rs` pick up Stat 1 automatically via `help_entries_all()` chain
**Avoids:** P22 (xrom_shadowing CI gate active)
**Research flag:** Standard patterns — SKIP research phase. Phase 29 (Math Pac I CLI) is the exact playbook.

### Phase 35: Documentation and ADRs

**Rationale:** Divergence catalog entries, docs-matrix extension, and ADR authoring require Phase 33+34 implementation decisions to be final. Analogous to Phase 30 in the Math Pac I roadmap.
**Delivers:** `docs/hp41-stat1-divergences.md` complete with D-33-NN entries (register layout, RNG algorithm, convergence criteria, CLΣSTAT extension policy, xrom_modules migration), `docs/hp41-stat1-function-matrix.md` generated, `justfile` with `docs-stat1` and `docs-stat1-check` recipes, new ADRs for v3.1 architectural decisions, README and CLAUDE.md v3.1 section
**Avoids:** P26 (correct D-33-NN identifiers; file never conflated with math1 divergences)
**Research flag:** Standard patterns — SKIP research phase. D-29.1 precedent fully established.

### Phase 36: hp41-gui — GUI Integration

**Rationale:** GUI integration completes the 4-way exhaustive-match invariant (item 4). CATALOG 2 must gain the STAT_1 entry. Pattern is identical to Phase 31 (Math Pac I GUI).
**Delivers:** 4-way invariant item 4 complete, CATALOG 2 shows "STAT 1B" entry, help overlay gains "Stat 1 Pac (XROM 2)" section parallel to "Math 1 Pac (XROM 7)", `key_map.rs` unchanged (all Stat Pac ops via XEQ-by-name), SC-4 invariant maintained (zero stat computation in GUI sources)
**Research flag:** Standard patterns — SKIP research phase. Phase 31 (Math Pac I GUI) is the exact playbook.

### Phase 37: Test Hardening and Quality Gates

**Rationale:** Coverage gate maintenance (≥ 95% lines / ≥ 93% regions) requires ~90–110 new test cases for ~850 new LOC. Stat 1 introduces new pitfall categories (variance stability, quantile convergence, RNG determinism, cancellation) needing dedicated test files.
**Delivers:** `stat1_accuracy.rs` with two-level tolerance policy (1e-9 closed-form distribution ops, 1e-7 iterative quantile ops), `stat1_variance_stability.rs` (P18 regression), `stat1_rand_determinism.rs` serde round-trip (P20), `stat1_cancellation.rs` (P25), `xrom_shadowing.rs` STAT_1 extension (P22), `v30_save_loads_with_stat1_off` backward-compat regression (P24), `numerical_accuracy.rs` extended with ~30 scipy.stats oracle cases, `stat1_op_test_count` meta-gate, E2E smoke for one Stat Pac XEQ workflow, Free42 contamination check against all new stats1 source files
**Research flag:** Standard patterns — SKIP research phase. Phase 32 (Math Pac I test hardening) established all patterns.

### Phase Ordering Rationale

- `distributions.rs` precedes ΣNORMD, ΣCHISQD, ΣPTST/ΣTSTAT because those programs consume the primitives directly
- OM register layout (P21) precedes the 5 most register-intensive programs (ΣMMTUG, ANOVA family, ΣMLRXY, ΣCTKKK); implementing them without the OM table produces silently wrong results that are hard to diagnose post-hoc
- CLI (Phase 34) precedes GUI (Phase 36) because `xrom_shadowing.rs` CI gate validates the STAT_1.ops mnemonic list and catches P22 collisions before GUI integration
- Docs (Phase 35) runs after CLI because the divergence catalog entries are informed by Phase 33–34 implementation decisions
- Tests (Phase 37) runs last to sweep all 5 preceding phases; the `stat1_op_test_count` gate requires all Op variants to be present before counting

### Research Flags

**Needs `/gsd-plan-phase --research-phase N` during planning:**
- **Phase 33:** NEEDS research phase — OM 00041-90030 "Storage Registers" section not yet fully read; RAND subroutine presence unconfirmed; quantile convergence criteria not yet extracted; ΣPOLYP degree prompt and ΣCHISQD ν convention unconfirmed; ΣTSTAT pooled vs. Welch assumption unresolved

**Standard patterns — skip research phase:**
- **Phase 34 (CLI):** Phase 29 playbook applies directly
- **Phase 35 (Docs):** Phase 30 / D-29.1 precedent applies directly
- **Phase 36 (GUI):** Phase 31 playbook applies directly
- **Phase 37 (Tests):** Phase 32 patterns apply directly

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | QRC read directly; rust_decimal MathematicalOps verified from docs.rs 2026-05-21; statrs rejection is codebase-verifiable; f64-bridge pattern validated by 763 numerical_accuracy.rs cases at 99.3% pass rate |
| Features | HIGH | QRC 00041-90061 read directly; NPS55-84-003 confirms anti-feature list (F, Binomial, histograms confirmed absent); 14 entry points confirmed; open gaps are explicitly flagged and bounded |
| Architecture | HIGH | Codebase read directly: bit-1 stub in xrom.rs confirmed, XromModule struct confirmed, ops/stats.rs R01–R06 layout confirmed, OnceLock pattern in help_data.rs confirmed, docs-matrix title-dispatch confirmed; XROM ID 2 confirmed from two independent external sources |
| Pitfalls | MEDIUM-HIGH | P18/P21/P22/P24/P25 are codebase-verifiable (HIGH); P19/P23 depend on OM convergence spec not yet fully read (MEDIUM); P20 is architecturally certain (HIGH); P27 Free42 stats-domain identifiers are hypothetical candidates pending core_math2.cc verification (LOW) |

**Overall confidence:** HIGH

### Gaps to Address

- **OM "Storage Registers" section (P21 — critical path):** Register layout for ΣMMTUG, ΣAOVONE/ΣAOVTWO/ΣANOCOV, ΣMLRXY, ΣCTKKK not yet confirmed. MUST be resolved in Phase 33 Phase 0 before those Ops are written. Source: OM 00041-90030 at literature.hpcalc.org.

- **RAND subroutine presence in ROM:** QRC does not list RAND as a top-level program. Confirm from OM listing before adding to STAT_1.ops. Defer implementation until Phase 33 Phase 0 resolves this.

- **Quantile convergence criteria:** Whether Stat 1 uses display-mode-tied threshold (like Math Pac I `integ_threshold()`) or a fixed tolerance is unconfirmed. Extract from OM user instructions for ΣNORMD (inverse path) and ΣCHISQD in Phase 33 Phase 0; document as D-33-NN entry.

- **ΣPOLYP degree prompt and ΣCHISQD ν storage convention:** QRC condensed notation does not show prompt strings. Tentative: "DEGREE=?" (Math Pac I precedent) and ν entered via [A] before x evaluation. Verify from OM in Phase 33 Phase 0.

- **ΣTSTAT equal vs. unequal variance assumption:** QRC shows two-sample t-test without specifying pooled or Welch. NPS ZS-4/5 assumes pooled. Confirm from OM user instructions before implementing the degrees-of-freedom formula.

- **Free42 stats-domain identifiers for contamination guard (P27):** Specific identifiers to add to `check-free42-contamination.sh` are hypothetical. Verify against `github.com/thomasokken/free42/blob/master/common/core_math2.cc` before Phase 33 Phase 0 to update the script.

## Sources

### Primary (HIGH confidence)
- HP-41C Stat Pac Quick Reference Card (00041-90061, June 1979) — all 13 programs, mnemonics, SIZE column, initialization/input/correction/results columns; read directly as 2-page PDF image at literature.hpcalc.org/community/hp41-pac-stat-qrc-en.pdf
- Naval Postgraduate School NPS55-84-003 (Zehna, February 1984, DTIC AD-A140573) — confirms RNG formula, ΣNORMD workflow, anti-feature list (F, Binomial, histograms absent), t-test variant, ΣCHISQD ν convention
- `calc.fjk.ch/db/hp41mod.php` HP-41 Module Database — "Statistics Pac 1B", XROM #2, 4k; confirms module ID
- `rust_decimal::MathematicalOps` trait (docs.rs, 2026-05-21) — full method list verified; norm_cdf/norm_pdf/erf confirmed present; inverse normal, incomplete gamma, incomplete beta confirmed absent
- Codebase direct reads: `hp41-core/src/ops/math1/xrom.rs` (bit-1 stub), `hp41-core/src/state.rs` (xrom_modules bitfield, default_xrom_modules()), `hp41-core/src/ops/stats.rs` (R01–R06 Σ-register layout), `hp41-cli/src/help_data.rs` (OnceLock pattern), `scripts/docs-matrix/src/main.rs` (title-dispatch pattern), `hp41-core/src/num.rs` lines 184–211 (f64-bridge pattern)

### Secondary (MEDIUM confidence)
- `statrs 0.18.0` Cargo.toml (docs.rs) — dep list verified: mandatory `approx 0.5.0`, optional `nalgebra 0.33` + `rand 0.8`; MSRV 1.65 confirmed
- HP-41 XROM numbering scheme (NN,FF): Math Pac = XROM 1,NN; Statistics Pac = XROM 2,NN — confirmed via multiple community sources; HP Museum xroms.htm was inaccessible during research
- HP-41 RNG formula `FRC(9821·x + 0.211327)` — hpcalc.org/hp48/docs/misc/rand.txt, HP Museum forum, NPS document p. 21 (all three sources agree)
- Acklam/AS 241 inverse normal rational approximation — stackedboxes.org/2017/05/01/acklams-normal-quantile-function/ (public-domain, well-known)
- AS 239 (incomplete gamma series+CF) and AS 63 (incomplete beta CF) — Royal Statistical Society Applied Statistics; Numerical Recipes §6.2 and §6.4

### Tertiary (LOW confidence — verify in Phase 33 Phase 0)
- HP Stat 1 Pac Owner's Manual (00041-90030) "Storage Registers" section — not yet read directly; access via literature.hpcalc.org item 800 needs verification
- Free42 `core_math2.cc` statistics identifiers — hypothetical candidates for contamination guard extension (P27); verify against github.com/thomasokken/free42 before Phase 33 Phase 0
- RAND subroutine presence in Stat 1 Pac ROM — cited as community convention from NPS user programs; needs OM listing confirmation before adding to STAT_1.ops

---
*Research completed: 2026-05-21*
*Ready for roadmap: yes*
