---
phase: 33
slug: hp41-core-xrom-activation-distribution-primitives-all-stat-1
status: planned
nyquist_compliant: true
wave_0_complete: false
created: 2026-05-22
updated_by_planner: 2026-05-22
---

# Phase 33 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `33-RESEARCH.md` §"Validation Architecture" (line 1157+).
> Per-task verification map populated by `gsd-planner` 2026-05-22 after Plans 33-00 through 33-08 created.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust 1.88, MSRV-pinned) + `proptest` 1.x for property tests |
| **Config file** | `hp41-core/Cargo.toml` (workspace) + `[dev-dependencies]` already provides `proptest`, `criterion` (advisory) |
| **Quick run command** | `just core-test` (currently `cargo test -p hp41-core --lib`) — runs all `hp41-core` lib unit tests, ~5 s |
| **Module-scoped run** | `cargo test -p hp41-core --lib ops::stat1` — runs only the new Stat 1 Pac module tests |
| **Full suite command** | `just ci-core` (currently `cargo test -p hp41-core --all-targets && cargo clippy -p hp41-core --all-targets -- -D warnings`) — ~30 s including the accuracy suite |
| **Accuracy suite** | `cargo test -p hp41-core --test stat1_accuracy` — new integration test target (Phase 37 work; Phase 33 ships only per-Op `#[cfg(test)] mod tests`) |
| **Coverage gate** | `just coverage` (cargo-llvm-cov) — `hp41-core` must remain ≥ 95 % lines / ≥ 93 % regions (held by Phase 37; Plan-by-plan local check) |
| **Free42 contamination gate** | `scripts/check-free42-contamination.sh` (CI parallel job + `just license-audit`) — must return zero matches; Plan 33-00 extends PATTERN to ≥18 tokens + STAT1_DIR scan |
| **Estimated runtime** | Quick: ~5 s · Module-scoped: ~2 s · Full: ~30 s · Coverage: ~90 s |

---

## Sampling Rate

- **After every task commit:** Run `just core-test` (or `cargo test -p hp41-core --lib ops::stat1` for module-local iteration).
- **After every plan wave:** Run `just ci-core` (full lib + integration + clippy gate) + `bash scripts/check-free42-contamination.sh`.
- **Before `/gsd:verify-work`:** Full suite green + `just license-audit` zero matches + `just coverage` reports ≥ 95.39 % / ≥ 94.26 % on `hp41-core` (Phase 37 gates this; Phase 33 ships only the unit tests).
- **Max feedback latency:** ≤ 30 s (full suite) — sub-second for module-scoped iteration.

---

## Per-Task Verification Map

*Populated by `gsd-planner` 2026-05-22 after Plans 33-00 through 33-08 were written. Each task in every PLAN.md surfaces here with an executable verify command. Status updates as tasks execute.*

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 33-00-T1 | 33-00 | 0 | STAT-QUAL-09 | T-33-free42-contamination | PATTERN extended ≥18 tokens; STAT1_DIR scan loop added | script | `bash scripts/check-free42-contamination.sh` | ✓ existing script extended | ⬜ |
| 33-00-T2 | 33-00 | 0 | STAT-UNI-03 | T-33-numerical-precision | OM Storage Registers in `mod.rs` `//!` + STAT1_MAX_REG const + named per-program register consts | compile | `cargo check -p hp41-core` | ❌ Wave 0 creates | ⬜ |
| 33-00-T3 | 33-00 | 0 | STAT-QUAL-09 + STAT-UNI-03 | T-33-free42-contamination | `pub mod stat1;` mounted; contamination guard exits 0 on clean, exits 1 on probe token | script + compile | `cargo check -p hp41-core && bash scripts/check-free42-contamination.sh` (plus probe-round-trip per acceptance) | ✓ after T1+T2 | ⬜ |
| 33-01-T1 | 33-01 | 1 | STAT-FW-02 + STAT-RNG-03 | T-33-state-persistence | rand_seed serde shape unique; default_xrom_modules→0b0000_0011; migrate_after_load idempotent | unit | `cargo test -p hp41-core --lib state::tests::xrom_modules_default_is_three state::tests::v3_0_save_loads_with_stat_1_after_migration state::tests::migrate_after_load_idempotent state::tests::rand_seed_serde_round_trip` | ❌ Wave 1 creates | ⬜ |
| 33-01-T2 | 33-01 | 1 | STAT-FW-03 + STAT-FW-04 | T-33-freeze-violation-creep | ModalProgram::Stat1 variant (D-33.3b carve-out); Stat1Step skeleton; 3 dispatch arms | compile | `cargo check -p hp41-core && cargo clippy -p hp41-core -- -D warnings` | ❌ Wave 1 (math1/modal.rs delta + new stat1/modal.rs) | ⬜ |
| 33-01-T3 | 33-01 | 1 | STAT-FW-01 + STAT-FW-04 | T-33-freeze-violation-creep | STAT_1 const id=2, name="STAT 1B"; stat1_resolve match; bit-1 arm fires LAST | unit | `cargo test -p hp41-core --lib ops::math1::xrom::tests::stat1_const_id_and_name ops::math1::xrom::tests::stat1_ops_has_correct_entry_count ops::math1::xrom::tests::stat1_ops_mnemonics_resolve_consistently ops::math1::xrom::tests::resolve_uses_bit_1_for_stat1` | ❌ Wave 1 modifies math1/xrom.rs per D-33.3 | ⬜ |
| 33-01-T4 | 33-01 | 1 | STAT-FW-03 | T-33-numerical-precision | Op::Stat1Stub scaffolding + 4-way invariant items 1+2 + xrom_shadowing.rs extension (disjoint from MATH_1.ops AND builtins) | unit + integration | `cargo check -p hp41-core && cargo test -p hp41-core --test xrom_shadowing` | ❌ Wave 1 | ⬜ |
| 33-02-T1 | 33-02 | 2 | STAT-DST-06 | T-33-numerical-precision | Acklam norm_cdf_inv_f64 + 6 oracle tuples within 1e-9 | unit | `cargo test -p hp41-core --lib ops::stat1::distributions::tests::norm_cdf_inv && bash scripts/check-free42-contamination.sh` | ❌ Wave 2 creates | ⬜ |
| 33-02-T2 | 33-02 | 2 | STAT-DST-06 | T-33-numerical-precision | AS 239 gamma_regularized_f64 (gser+gcf split at x<a+1) + ≥6 oracle tuples including x≈a+1 boundary | unit | `cargo test -p hp41-core --lib ops::stat1::distributions::tests::gamma` | ❌ | ⬜ |
| 33-02-T3 | 33-02 | 2 | STAT-DST-06 | T-33-numerical-precision | AS 63 beta_regularized_f64 + ≥6 oracle tuples + file ≤300 LOC | unit | `cargo test -p hp41-core --lib ops::stat1::distributions::tests` (≥18 tests across 3 primitives) | ❌ | ⬜ |
| 33-03-T1 | 33-03 | 2 | STAT-DST-07 | T-33-numerical-precision | HpError::Cancelled/ConvergenceFailed available; quantile_threshold(Fix(6))=1e-7 | unit | `cargo test -p hp41-core --lib ops::stat1::distributions::tests::quantile_threshold` | ❌ | ⬜ |
| 33-03-T2 | 33-03 | 2 | STAT-DST-01..05 | T-33-state-persistence | Stat1Step real variants (NormdModeChoice, ChisqdNuPrompt, ChisqdModeChoice) replace Plan-33-01 placeholder | compile | `cargo check -p hp41-core` | ❌ | ⬜ |
| 33-03-T3 | 33-03 | 2 | STAT-DST-01 + STAT-DST-02 + STAT-DST-03 | T-33-numerical-precision | ΣNORMD CDF=Q(1.96)≈0.025 (1e-9); PDF=φ(0)≈0.399 (1e-9); inverse Φ⁻¹(0.025)≈-1.96 (1e-7) | unit | `cargo test -p hp41-core --lib ops::stat1::normd::tests` | ❌ | ⬜ |
| 33-03-T4 | 33-03 | 2 | STAT-DST-04 + STAT-DST-05 + STAT-DST-07 | T-33-numerical-precision + T-33-state-persistence | ΣCHISQD CDF≈0.95 (1e-7); PDF≈0.154 (1e-9); cancel_requested integration test < 1 iter | unit + integration | `cargo test -p hp41-core --lib ops::stat1::chisqd::tests && cargo test -p hp41-core --test stat1_cancellation` | ❌ | ⬜ |
| 33-04-T1 | 33-04 | 2 | STAT-HYP-07 | T-33-numerical-precision | ΣSPEAR ρ_s=0.8 within 1e-9 (SPEC.md Req. 30 oracle 0.7 vs scipy 0.8 — discrepancy resolved inline) | unit | `cargo test -p hp41-core --lib ops::stat1::nonparam::tests::spear` | ❌ | ⬜ |
| 33-04-T2 | 33-04 | 2 | STAT-HYP-03 | T-33-numerical-precision | ΣXSQEV χ²=2.667 within 1e-9; zero-expected → Domain error | unit | `cargo test -p hp41-core --lib ops::stat1::nonparam::tests::xsqev` | ❌ | ⬜ |
| 33-04-T3 | 33-04 | 2 | STAT-HYP-04 | T-33-numerical-precision | ΣEFXSQ χ² within 1e-9; proportions sum-to-1 validation | unit | `cargo test -p hp41-core --lib ops::stat1::nonparam::tests::efxsq` | ❌ | ⬜ |
| 33-05-T1 | 33-05 | 2 | STAT-UNI-01 | T-33-numerical-precision | ΣBSTAT weighted mean + CV within 1e-9; ΣBSTG bivariate variant | unit | `cargo test -p hp41-core --lib ops::stat1::basic_stats::tests` | ❌ | ⬜ |
| 33-05-T2 | 33-05 | 2 | STAT-REG-01..04 | T-33-numerical-precision | ΣLIN (a=0,b=2 ±1e-9); ΣEXP/LOGI/POW log-linearization via op_sigma_plus delegate | unit | `cargo test -p hp41-core --lib ops::stat1::regression::tests` (Plan-33-05 subset) | ❌ | ⬜ |
| 33-06-T1 | 33-06 | 3 | STAT-UNI-02 + STAT-UNI-04 | T-33-numerical-precision + T-33-state-persistence | ΣMMTUG/MMTGD γ₁/γ₂ within 1e-9; op_sigma_minus extension proves [C] round-trip | unit | `cargo test -p hp41-core --lib ops::stat1::moments::tests && cargo test -p hp41-core --test stats_tests` | ❌ | ⬜ |
| 33-06-T2 | 33-06 | 3 | STAT-AOV-01..04 | T-33-numerical-precision (P21) | ΣAOVONE F=100.0 within 1e-9; ΣAOVTWO row+col F within 1e-7; ΣANOCOV NPS ZA-3 within 1e-7; named-const register access only (no literal-integer indices ≥7) | unit + grep | `cargo test -p hp41-core --lib ops::stat1::anova::tests && (grep -nE 'state.regs\\[[7-9]\\]\|state.regs\\[[1-9][0-9]\\]' hp41-core/src/ops/stat1/anova.rs; test $? -ne 0)` | ❌ | ⬜ |
| 33-06-T3 | 33-06 | 3 | STAT-HYP-05 + STAT-HYP-06 | T-33-numerical-precision | ΣCTKKK 2×3 χ²≈4.286 within 1e-9; ΣCTKK 2×2 χ²≈0.397 within 1e-9 | unit | `cargo test -p hp41-core --lib ops::stat1::nonparam::tests::ctkk` | ❌ | ⬜ |
| 33-07-T1 | 33-07 | 3 | STAT-HYP-01 | T-33-numerical-precision | ΣPTST t=0, p=1 for μ₀=mean(data) within 1e-7; beta_regularized_f64 bridge | unit | `cargo test -p hp41-core --lib ops::stat1::hypothesis::tests::ptst` | ❌ | ⬜ |
| 33-07-T2 | 33-07 | 3 | STAT-HYP-02 | T-33-numerical-precision | ΣTSTAT pooled-variance t≈-5.0, p≈0.0010534 within 1e-7; integer df; Welch excluded | unit | `cargo test -p hp41-core --lib ops::stat1::hypothesis::tests::tstat` | ❌ | ⬜ |
| 33-08-T1 | 33-08 | 3 | STAT-REG-05..09 | T-33-numerical-precision | ΣMLRXY/MLRXYZ/POLYP/POLYC local Gauss elimination; `grep -nE 'ops::math1::matrix' hp41-core/src/ops/stat1/` empty (SPEC.md Req. 21) | unit + grep | `cargo test -p hp41-core --lib ops::stat1::regression::tests && (grep -nE 'ops::math1::matrix' hp41-core/src/ops/stat1/; test $? -ne 0)` | ❌ | ⬜ |
| 33-08-T2 | 33-08 | 3 | STAT-RNG-01 + STAT-RNG-02 + STAT-RNG-04 | T-33-state-persistence + T-33-numerical-precision | RAND deterministic sequence; SEED modal round-trip; rand_seed survives save/load; Op::Stat1Stub DELETED | unit + integration + grep | `cargo test -p hp41-core --lib ops::stat1::rand::tests && cargo test -p hp41-core --test stat1_rand_determinism && (grep -c 'Op::Stat1Stub' hp41-core/src/; test $? -ne 0)` | ❌ | ⬜ |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Sampling continuity check:** every task in every plan has at least one `<automated>` verify (either `cargo test` or `bash`/`grep` script). No 3 consecutive tasks without automated verify. Wave 0 creates the algorithmic + structural foundation (Plans 33-00 + 33-01); all subsequent waves consume the Wave 0/1 outputs via the `depends_on:` frontmatter.

---

## Wave 0 Requirements

Wave 0 establishes the algorithmic + module-wiring foundation BEFORE any user-facing Op is implemented. Per the 9-plan slice D-33.2:

- [ ] **Plan 33-00 (Foundation)** — `hp41-core/src/ops/stat1/mod.rs` with OM `//!` Storage-Registers transcription + `STAT1_MAX_REG` const + per-program named consts; `scripts/check-free42-contamination.sh` PATTERN extended (≥18 tokens) + STAT1_DIR scan loop. NO Op code.
- [ ] **Plan 33-01 (XROM framework activation)** — `STAT_1: XromModule { id: 2, name: "STAT 1B" }` + bit-1 arm in `xrom_resolve` + `stat1_resolve` (D-33.3 freeze exception) + `ModalProgram::Stat1(Stat1Step)` (D-33.3b second freeze exception) + `default_xrom_modules() = 0b0000_0011` + `CalcState::migrate_after_load()` + `rand_seed: HpNum` with `#[serde(default)]` (NO skip) + `Op::Stat1Stub` scaffolding for 26 STAT_1.ops entries + 4-way invariant items 1+2 + `xrom_shadowing.rs` extension.
- [ ] **Plan 33-02 (Distribution primitives)** — `hp41-core/src/ops/stat1/distributions.rs` with `norm_cdf_inv_f64` (Acklam/AS 241), `gamma_regularized_f64` (AS 239), `beta_regularized_f64` (AS 63) + ≥18 inline `(input, scipy_expected, tolerance)` oracle tuples. All GREEN BEFORE Plan 33-03 ships any Op.

Wave 1 (Plan 33-01) depends on Wave 0 (Plan 33-00) per the `depends_on:` chain. Plans 33-02..33-05 depend only on Plan 33-01 (Wave 2 parallel). Plans 33-06..33-08 depend on Plans 33-01 + their immediate prerequisites (Wave 3 parallel).

**`tests/stat1_accuracy.rs` is NOT a Phase 33 deliverable** — Phase 37 ships the consolidated accuracy suite. Phase 33 ships only per-Op `#[cfg(test)] mod tests` blocks + 2 integration tests (`stat1_cancellation.rs` from Plan 33-03 + `stat1_rand_determinism.rs` from Plan 33-08).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| OM divergence document accuracy | STAT-DOC-* (Phase 35 deliverable; Phase 33 records discrepancies for handoff) | The mechanical assertion (file exists, has disclaim header, lists D-33-NN entries) is automatable; the semantic correctness of each documented divergence requires a human cross-check against OM 00041-90030 scans. | After Phase 35 ships `docs/hp41-stat1-divergences.md`, open side-by-side with OM scans; confirm every Phase-33-surfaced divergence (ΣSPEAR 0.7 vs 0.8, ΣEFXSQ 1.667 vs scipy, ΣBSTAT CV, D-33.3 + D-33.3b math1/ freeze carve-outs, D-33.4 RAND emulator-extension, etc.) has a primary-source citation. |
| Oracle anomaly resolution: ΣSPEAR (0.7 vs 0.8) | STAT-HYP-07 / SPEC.md Req. 30 | Decision about which value is canonical (SPEC.md vs scipy) is manual. | Plan 33-04 runs `scipy.stats.spearmanr([1,2,3,4,5], [2,1,3,5,4]).statistic` and lands the scipy-confirmed value 0.8 in the test; SPEC.md amendment is queued for Phase 35 (gsd-planner did NOT amend SPEC.md in place — SPEC.md is locked input per D-33.1). |
| OM read for RAND/SEED ROM-presence (D-33.4 vs D-33.4a routing) | STAT-RNG-04 / SPEC.md Req. 38 | Plan 33-08 reads OM 00041-90030 to determine whether RAND/SEED are top-level ROM entries; the SUMMARY records the result for Phase 35 divergences-catalog routing (emulator-extension bucket vs OM-feature-complete). | Plan 33-08 SUMMARY documents the OM read; operator confirms the routing for Phase 35's `docs/hp41-stat1-divergences.md` section. |

*All other phase behaviors have automated verification (per the Per-Task Verification Map above — 25 tasks across 9 plans, every task has an `<automated>` verify).*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify (per the Per-Task Verification Map; 25 of 25 tasks).
- [x] Sampling continuity: no 3 consecutive tasks without automated verify.
- [x] Wave 0 covers all MISSING references (Plan 33-00 ships contamination-guard + mod.rs skeleton; Plan 33-01 ships XROM framework + serde fields).
- [x] No watch-mode flags.
- [x] Feedback latency < 30 s (full suite) / < 5 s (quick).
- [ ] Coverage gate ≥ 95.39 % lines / ≥ 94.26 % regions on `hp41-core` (Phase 37 enforces; Phase 33 ships unit tests that contribute to coverage).
- [ ] `scripts/check-free42-contamination.sh` returns zero matches (Plan 33-00 extends; every plan acceptance includes the re-run).
- [ ] Coverage hardening, `lint_stat1_assertions.rs`, `stat1_op_test_count.rs`, full `stat1_accuracy.rs` suite — **Phase 37** (NOT Phase 33).
- [x] `nyquist_compliant: true` set in frontmatter (every task has automated verification).

**Approval:** planner-approved 2026-05-22; awaiting executor (`/gsd:execute-phase 33`).
