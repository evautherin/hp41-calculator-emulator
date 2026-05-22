---
phase: 33
slug: hp41-core-xrom-activation-distribution-primitives-all-stat-1
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-22
---

# Phase 33 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `33-RESEARCH.md` §"Validation Architecture" (line 1157+).
> The planner will fill the per-task verification map in §"Per-Task Verification Map" after PLAN.md files are written.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust 1.88, MSRV-pinned) + `proptest` 1.x for property tests |
| **Config file** | `hp41-core/Cargo.toml` (workspace) + `[dev-dependencies]` already provides `proptest`, `criterion` (advisory) |
| **Quick run command** | `just core-test` (currently `cargo test -p hp41-core --lib`) — runs all `hp41-core` lib unit tests, ~5 s |
| **Module-scoped run** | `cargo test -p hp41-core --lib ops::stat1` — runs only the new Stat 1 Pac module tests |
| **Full suite command** | `just ci-core` (currently `cargo test -p hp41-core --all-targets && cargo clippy -p hp41-core --all-targets -- -D warnings`) — ~30 s including the accuracy suite |
| **Accuracy suite** | `cargo test -p hp41-core --test stat1_accuracy` — new integration test target mirroring `tests/numerical_accuracy.rs` (≥ 100 oracle cases per RESEARCH.md §"Validation Architecture") |
| **Coverage gate** | `just coverage` (cargo-llvm-cov) — `hp41-core` must remain ≥ 95 % lines / ≥ 93 % regions |
| **Free42 contamination gate** | `scripts/check-free42-contamination.sh` (CI parallel job + `just license-audit`) — must return zero matches for the 12 distinctive identifiers |
| **Estimated runtime** | Quick: ~5 s · Module-scoped: ~2 s · Full: ~30 s · Coverage: ~90 s |

---

## Sampling Rate

- **After every task commit:** Run `just core-test` (or `cargo test -p hp41-core --lib ops::stat1` for module-local iteration).
- **After every plan wave:** Run `just ci-core` (full lib + integration + clippy gate).
- **Before `/gsd:verify-work`:** Full suite green + `just license-audit` zero matches + `just coverage` reports ≥ 95 % / ≥ 93 % on `hp41-core` + the new `stat1_accuracy` integration test must pass at the documented tolerance bands.
- **Max feedback latency:** ≤ 30 s (full suite) — sub-second for module-scoped iteration.

---

## Per-Task Verification Map

*Filled by `gsd-planner` during plan creation. Each task in every PLAN.md must surface here with an executable verify command or an explicit Wave 0 dependency.*

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| _to be populated by planner_ | | | | | | | | | |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Wave 0 establishes the algorithmic + module-wiring foundation BEFORE any user-facing Op is implemented. Per RESEARCH.md the recommended split is:

- [ ] **Plan 33-00 (Algorithmic primitives)** — `hp41-core/src/ops/stat1/distributions.rs` with `norm_cdf_f64`, `norm_cdf_inv_f64` (Acklam/AS 241), `gamma_regularized_f64` (AS 239 / Numerical Recipes 3e gser + gcf split at `x < a+1`), `beta_regularized_f64` (AS 63 / Numerical Recipes 3e betacf, symmetric rewrite at `x ≥ (a+1)/(a+b+2)`), and the LCG RNG primitive. Each primitive ships with the oracle table from RESEARCH.md §"Validation Architecture" (18 tuples) wired as `#[test]` cases.
- [ ] **Plan 33-01 (Module skeleton)** — `hp41-core/src/ops/stat1/{mod,xrom,modal}.rs`, `docs/hp41-stat1-functions.json`, verbatim OM disclaim header, `xrom_resolve` wiring, `builtin_card_op` extension 12 → 13, 4-way exhaustive-match landed for all stub Ops, `STAT1_HELP_ENTRIES` loaded via `OnceLock`, parity test `function_matrix_parity.rs` extended.
- [ ] **`tests/stat1_accuracy.rs` skeleton** — at minimum one assertion per success criterion (#1 through #5 from ROADMAP.md). Real cases land plan-by-plan as Ops are implemented.

*If none: "Existing infrastructure covers all phase requirements."* — **NOT THE CASE FOR PHASE 33.** Stat 1 Pac is a brand-new XROM module; Wave 0 is mandatory.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| OM divergence document accuracy | STAT-FW-04 (if applicable) — divergences sidecar | RESEARCH.md §"Pitfall 8" flags a freeze-exception amendment (`ModalProgram::Stat1` in `math1/modal.rs`) that needs operator sign-off because it touches a frozen module. The mechanical assertion (file exists, has disclaim header) is automatable; the semantic correctness of the divergences listed in `docs/hp41-stat1-divergences.md` requires a human cross-check against OM 00041-90094 by the operator before sign-off. | After Plan 33-09 (docs) lands, open `docs/hp41-stat1-divergences.md` side-by-side with the relevant OM section scans; confirm every emulator extension and every documented divergence is paired with a primary-source citation. |
| Oracle anomaly resolution: ΣSPEAR test value (0.7 vs 0.8) | STAT-HYP-* (Spearman rank correlation) | RESEARCH.md Open Question 4 flags an oracle/expected-value disagreement that needs operator decision once Plan 33-04 generates the scipy oracle. Automatable test exists, but the **decision** about which value is canonical is manual. | Run the scipy oracle in Plan 33-04; if 0.8 returns, the SPEC.md requirement note is updated; if 0.7 returns, the test expectation is the canonical value. Operator confirms in DISCUSSION-LOG. |

*All other phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (algorithmic primitives + module skeleton)
- [ ] No watch-mode flags
- [ ] Feedback latency < 30 s (full suite) / < 5 s (quick)
- [ ] Coverage gate ≥ 95 % lines / ≥ 93 % regions on `hp41-core` (post-merge regression check)
- [ ] `scripts/check-free42-contamination.sh` returns zero matches
- [ ] `tests/stat1_accuracy.rs` integration suite passes with documented tolerance bands (1e-9 for inverse normal, 1e-7 for chi-square CDF, 1e-7 for incomplete beta, full bit-equality for LCG sequence after save/load)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
