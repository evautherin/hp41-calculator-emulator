# Phase 37: Test Hardening & Quality Gates - Context

**Gathered:** 2026-05-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Quality infrastructure and CI enforcement for the Stat 1 Pac (XROM 2) surface shipped in Phases 33–36. No new user-facing features — every deliverable is a test, meta-gate, CI enforcement, or quality-gate verification. 12 requirements: STAT-QUAL-01..11 + STAT-GUI-05.

**What this phase delivers:**

1. **Coverage hold** (STAT-QUAL-01/02/03): `hp41-core` line coverage ≥ 95.39 % and region coverage ≥ 94.26 % (v3.0 baseline preserved); every `hp41-core/src/ops/stat1/*.rs` file individually ≥ 90 % (mirrors the math1 per-file policy).

2. **Numerical accuracy extension** (STAT-QUAL-04): `numerical_accuracy.rs` extended with ~30 freshly scipy-derived Stat 1 oracle cases at ≥ 98 % pass rate; v1.x 503-case floor 498/503 and v3.0 768-case floor 763/768 both preserved.

3. **Two-level tolerance discipline** (STAT-QUAL-05): 1e-9 relative for closed-form ops, 1e-7 relative for iterative paths — enforced via the assertion lints.

4. **Assertion-discipline lint** (STAT-QUAL-06): `lint_stat1_assertions.rs` blocking `assert_eq!(decimal, decimal)` on iterated results — Pitfall 14/17 discipline extended to Stat 1.

5. **Per-Op test-count meta-gate** (STAT-QUAL-07): `stat1_op_test_count.rs` enforcing ≥ 5 tests per Stat 1 Op variant at CI time (Pitfall 16).

6. **XROM-shadowing cross-check** (STAT-QUAL-08): `xrom_shadowing.rs` gate extended to verify STAT_1.ops against an OM-derived allowlist — no Math Pac I or built-in mnemonic shadowed.

7. **Free42 contamination re-verification** (STAT-QUAL-09): already extended to 18 tokens in Phase 33 Plan 33-00; Phase 37 re-verifies the gate holds across the final stat1/*.rs tree and documents the final token count.

8. **Backward-compat migration test** (STAT-QUAL-10): a v3.0 save file with `"xrom_modules": 1` loads into v3.1 without error; `migrate_after_load` activates Stat 1 (bit 1).

9. **E2E smoke extension** (STAT-QUAL-11): WebdriverIO smoke.spec.js extended with one Stat 1 Pac workflow on Ubuntu-only (ci-gui.yml::e2e-linux): `XEQ "ΣNORMD"` with x = 1.96 returning Q ≈ 0.0250 on the GUI LCD.

10. **STAT-GUI-05 resolution** (from Phase 36 reassignment per D-36.2/D-36.3): bounded 50-iter distribution primitives complete in microseconds — cancellation is not needed. Phase 37 formally resolves the requirement.

**In scope:**
- `hp41-core/tests/stat1_op_test_count.rs` (new, analog of `math1_op_test_count.rs`)
- `hp41-core/tests/lint_stat1_assertions.rs` (new, analog of `lint_math1_assertions.rs`)
- `hp41-core/tests/numerical_accuracy.rs` extension (~30 freshly scipy-derived Stat 1 cases)
- `hp41-core/tests/stat1_*.rs` coverage-gap test files (as needed per per-file coverage measurement)
- `hp41-cli/tests/xrom_shadowing.rs` STAT_1 extension (or `hp41-core` depending on current location)
- `hp41-core/tests/stat1_backward_compat.rs` (v3.0 save-file migration test)
- `hp41-gui/e2e/smoke.spec.js` Stat 1 workflow extension (Ubuntu-only)
- STAT-GUI-05 formal resolution (documented waiver + divergence-catalog entry or lightweight sanity test)
- Quality-gate programmatic verification (`just ci` + `just gui-ci` + coverage gates pass)
- README hard-claim graduation decision (Phase 37 final plan or `/gsd-complete-milestone`)

**Out of scope (explicit):**
- Any `hp41-core/src/ops/stat1/*.rs` implementation changes — Stat 1 Pac algorithms are frozen since Phase 33 ship
- Any `hp41-core/src/ops/math1/` changes — math1/ freeze remains in force
- Any `hp41-cli/src/` changes — CLI integration sealed by Phase 34
- Any `hp41-gui/src-tauri/src/` changes — GUI integration sealed by Phase 36
- Any `hp41-gui/src/` (React frontend) changes beyond E2E spec extension
- `docs/` changes beyond potential hp41-stat1-divergences.md behavioral-policy entry for STAT-GUI-05
- New XROM modules or Op variants
- Signed binary releases — deferred to v3.1.x / v3.2

**Mandated by ROADMAP cross-cutting constraints + CLAUDE.md frozen invariants:**
- **SC-4 invariant** trivially preserved — Phase 37 adds test files only; no `hp41-gui/src-tauri/src/op_*` functions
- **`#![deny(clippy::unwrap_used)]`** continues in `hp41-core`; new test files carry `#[allow(clippy::unwrap_used)]` at file scope per established pattern
- **MSRV 1.88** unchanged. Zero new runtime or dev-dependencies.
- **Free42 GPL contamination guard:** already extended to 18 tokens; re-verified in Phase 37

</domain>

<decisions>
## Implementation Decisions

### Already locked in PROJECT.md / REQUIREMENTS.md / prior CONTEXT.md files (carried forward — NOT re-decided here)

- **D-36.2 / D-36.3 (STAT-GUI-05 reassignment):** bounded 50-iter distribution primitives (`norm_cdf_inv_f64` closed-form Acklam, `gamma_regularized_f64` gser/gcf ITER_CAP=50, `beta_regularized_f64` betacf ITER_CAP=50) complete in microseconds; cancellation via `request_cancel` `Arc<AtomicBool>` is not applicable. Requirement reassigned Phase 36 → Phase 37 for formal resolution.
- **D-33.8 / STAT-QUAL-09:** Free42 contamination guard already extended from 12→18 tokens in Phase 33 Plan 33-00 (D-32.7 reassignment). Both `math1/` and `stat1/` trees scanned. Phase 37 re-verifies but does not add new tokens.
- **D-32.1 / math1_op_test_count.rs pattern:** scan `xrom.rs` for variant names registered in module-level `_resolve()` function, then count test-function-body mentions across module-prefixed test files. Word-boundary matching to avoid substring inflation (WR-03). ≥ 5 mentions per variant.
- **D-32.1 / lint_math1_assertions.rs pattern:** two lints — `no_decimal_assert_eq` (Pitfall 17) and `no_manual_tolerance_pattern` (Pitfall 14). Line-level grep with LINT-EXEMPT annotations. Multi-line assert_eq detection (WR-02). Scans module-prefixed `tests/math1_*.rs` files.
- **D-27.13 / E2E smoke shape:** WebdriverIO + Mocha + tauri-driver on `ci-gui.yml::e2e-linux`. Plain `.js` spec. `data-key-id` for keyboard clicks, `data-text` attribute on LCD for assertions (14-segment SVG has no getText()).
- **STAT-QUAL-05 (two-level tolerance):** 1e-9 relative for closed-form ops (ΣNORMD CDF/PDF, ΣSPEAR, ΣBSTAT/BSTG, ΣLIN/EXP/LOGI/POW, ΣXSQEV/EFXSQ); 1e-7 relative for iterative paths (probit Φ⁻¹, regularized incomplete gamma/beta, t-CDF, multiple-regression normal-equation solve). Locked in REQUIREMENTS.md.
- **D-35.3 (README soft-claim graduation gating):** hard-claim graduation conditioned on STAT-QUAL-04 (numerical_accuracy ≥ 98 % with Stat 1 cases) + STAT-QUAL-11 (E2E smoke extended). Mirrors v3.0 D-30.9 → D-32.5 graduation pattern.

### Discussed and decided in this session (D-37.1 — D-37.5)

#### Numerical accuracy oracle sourcing

- **D-37.1: Fresh scipy derivation for all ~30 Stat 1 accuracy cases.** Every oracle reference value is independently derived from scipy.stats (numpy/scipy Python session), NOT extracted from existing Phase 33 inline tests. This provides a fully independent validation layer — Phase 33 tests validate the implementation against scipy; Phase 37 accuracy suite re-validates against a SEPARATELY DERIVED scipy baseline. The two sets may agree on reference values, but the derivation process is independent.
  - **Why:** The accuracy suite is the FINAL quality gate before the "feature-complete" hard-claim. An independent derivation catches any Phase 33 oracle transcription errors that might have propagated into inline tests. Mirrors the v3.0 approach where `numerical_accuracy.rs` cases were derived independently of inline math1 tests.

#### E2E smoke platform scope

- **D-37.2: Ubuntu-only for the Stat 1 E2E smoke extension.** Extend the existing `smoke.spec.js` in `ci-gui.yml::e2e-linux`. The E2E spec is ΣNORMD with x = 1.96 → Q ≈ 0.0250 asserted on the GUI LCD. No extension to macOS/Windows CI matrix.
  - **Why:** matches v3.0 D-27.13 scope. WebdriverIO + tauri-driver E2E smoke runs on Ubuntu only (the `e2e-linux` job). macOS/Windows CI jobs cover build+test but not E2E (headless X11 dependency). ROI of cross-platform E2E doesn't justify the CI infrastructure cost for a behavioral-verification gate.

### Claude's Discretion

- **D-37.3 (accuracy case file shape):** Claude decides whether Stat 1 accuracy cases extend the existing 768-case array inline in `numerical_accuracy.rs` or live in a separate `stat1_numerical_accuracy.rs`. Recommendation: inline extension (single unified pass-rate gate, mirrors the v3.0 Math 1 extension pattern where cases were appended to the same array). The combined pass-rate denominator grows from 768 to ~798; the ≥ 98 % gate applies to the full set. v1.x 503-case floor and v3.0 768-case floor are preserved as sub-gates.
- **D-37.4 (accuracy case allocation by algorithm family):** Claude allocates cases by algorithmic risk. Iterative paths (ΣNORMD inverse/CDF, ΣCHISQD CDF, ΣTSTAT p-value, ΣMLRXY/ΣMLRXYZ normal-equation solve, ΣPOLYP polynomial regression) get more cases (they're the paths most likely to drift across platforms); closed-form paths (ΣBSTAT/BSTG means, ΣSPEAR rank correlation, ΣLIN/EXP/LOGI/POW simple regression, ΣXSQEV/EFXSQ chi-square sums) get fewer. Distribution of ~30 cases: ~12-15 distribution/iterative paths, ~15-18 across remaining families.
- **D-37.5 (per-case vs family-rigid tolerance):** Claude decides. Recommendation: per-case tolerance field (each case carries either 1e-9 or 1e-7) using the existing `case!` macro `tol` parameter. This mirrors the existing `WIDE_TOL` pattern and allows edge-case tolerance tuning (e.g., a normal CDF case at extreme tails may need wider tolerance than the standard 1e-7 iterative band).
- **D-37.6 (coverage gap closure strategy):** Claude decides. Recommendation: measure first via `cargo llvm-cov` per-file, then write targeted tests only for files below 90%. Existing inline test density is substantial (189 tests across 12 source files); many files likely already meet the 90% floor. Write per-module external test files (e.g., `stat1_anova.rs`, `stat1_regression.rs`) only for gap files — mirrors the math1 pattern but doesn't create empty busywork files for already-covered modules.
- **D-37.7 (stat1_op_test_count.rs scan scope):** Claude decides. Recommendation: scan BOTH inline tests inside `stat1/*.rs` source files AND external `tests/stat1_*.rs` files. This reflects actual test density (189 inline tests exist). Differ from math1 pattern (which scans only external `tests/math1_*.rs`) because Stat 1 has proportionally more inline coverage from Phase 33's implementation-time testing.
- **D-37.8 (lint_stat1_assertions.rs — new file vs extend existing):** Claude decides. Recommendation: new file `lint_stat1_assertions.rs` scanning `stat1_*.rs` files (both inline modules in `src/ops/stat1/` and external `tests/stat1_*.rs`). Clean separation mirrors the math1/stat1 sibling convention used everywhere else. No rename of the existing `lint_math1_assertions.rs`.
- **D-37.9 (STAT-GUI-05 resolution approach):** Claude decides. Recommendation: documented N/A waiver in VERIFICATION.md with the bounded-iter rationale (D-36.2). Add a `D-35-NN` behavioral-policy entry in `docs/hp41-stat1-divergences.md` documenting the bounded-vs-open-ended iteration design choice. No code changes — the rationale is engineering-correct and documented per the divergence-catalog convention. A lightweight wall-time sanity test may be included at planner discretion if it fits naturally into the coverage-gap test infrastructure.
- **D-37.10 (plan count):** Claude decides. Recommendation: 7–8 plans based on blast-radius analysis. Natural slicing:
  - Wave 1: Meta-gate infrastructure (stat1_op_test_count.rs + lint_stat1_assertions.rs + xrom_shadowing.rs STAT_1 extension) — must land BEFORE coverage-gap tests so new test files are correctly scanned
  - Wave 2: Coverage gap closure (per-module test files as needed per coverage measurement) + STAT-GUI-05 resolution + backward-compat test + Free42 re-verification
  - Wave 3: Numerical accuracy suite extension (~30 scipy-derived cases)
  - Wave 4: E2E smoke extension + quality-gate graduation verification (final `just ci` + `just gui-ci` + coverage gate check)
- **D-37.11 (README hard-claim graduation):** Claude decides. Recommendation: include in Phase 37's final plan (mirrors v3.0 Phase 32's D-32.5 graduation where the same phase that verified quality gates also graduated the README claim). Keeps v3.1 self-contained; `/gsd-complete-milestone` handles the MILESTONES.md finalization and tag cut but not the README rewrite.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-level (always-on)

- `.planning/PROJECT.md` — v3.1 milestone scope; v3.0 archived block; key decisions ledger; quality metrics table
- `.planning/REQUIREMENTS.md` — 66 v3.1 requirements; Phase 37 maps to STAT-QUAL-01..11 + STAT-GUI-05 (rows in the traceability table); STAT-GUI-05 line annotated with D-36.2 bounded-iter rationale
- `.planning/ROADMAP.md` — Phase 37 section (Goal + 5 success criteria); cross-cutting constraints; estimated 6–10 plans
- `.planning/STATE.md` — v3.1 phase overview; performance metrics table (quality-gate baselines); critical implementation traps
- `CLAUDE.md` (repo root) — Frozen Invariants section (all preserved); Quality Gates table (current targets); Key Files table; `### v3.1 additions` block (Phases 33–35 populated; Phase 36 shipped; Phase 37 stub `(in progress)`)

### Phase 33 (the surface Phase 37 tests)

- `.planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-CONTEXT.md` — Phase 33 decisions D-33.3..D-33.8; carries forward as Phase 37 read-only context
- `.planning/phases/33-…/33-SPEC.md` — 39 LOCKED Stat 1 Pac requirements; Req. 34 (ITER_CAP = 50 for gser/gcf/betacf — cited in D-36.2 bounded-iter rationale); tolerance specifications
- `.planning/phases/33-…/33-VERIFICATION.md` — 6 oracle drifts already reconciled in Phase 35; baseline accuracy metrics
- `.planning/phases/33-…/33-SPEC-AMENDMENT.md` — 6 scipy-corrected oracle values (Phase 37 accuracy cases must use the corrected values, not the original SPEC estimates)

### Phase 36 (STAT-GUI-05 reassignment source)

- `.planning/phases/36-hp41-gui-gui-integration/36-CONTEXT.md` — D-36.2 (bounded-iter cancellation reassessment) + D-36.3 (STAT-GUI-05 reassignment bookkeeping) — the formal rationale Phase 37 resolves

### v3.0 Phase 32 (the direct analog — STRUCTURE TEMPLATE)

- `.planning/milestones/v3.0-phases/32-test-hardening/32-CONTEXT.md` — Phase 32 decisions; structurally identical phase one milestone earlier (Math Pac I test hardening)
- `.planning/milestones/v3.0-phases/32-test-hardening/32-RESEARCH.md` — Per-Op Test Count Audit + coverage analysis methodology; structural template for Phase 37 research
- `.planning/milestones/v3.0-phases/32-test-hardening/32-01-PLAN.md` through `32-09-PLAN.md` — the 9-plan Math Pac I test-hardening execution; structural templates for Phase 37 plan slicing

### Existing test infrastructure (Phase 37 extends or mirrors)

- `hp41-core/tests/math1_op_test_count.rs` — direct structural template for `stat1_op_test_count.rs`; word-boundary matching (WR-03), variant detection from xrom.rs, per-file glob scan
- `hp41-core/tests/lint_math1_assertions.rs` — direct structural template for `lint_stat1_assertions.rs`; `no_decimal_assert_eq` + `no_manual_tolerance_pattern` lints; LINT-EXEMPT annotation convention; multi-line detect (WR-02)
- `hp41-core/tests/numerical_accuracy.rs` — 768-case suite (8348 LOC); `case!` macro with per-case `tol` parameter; `AccuracyCase` struct; Phase 37 extends inline
- `hp41-core/tests/stat1_rand_determinism.rs` — existing Stat 1 test (serde round-trip for `rand_seed`); Phase 37 backward-compat test can reuse the fixture pattern
- `hp41-core/tests/stat1_cancellation.rs` — existing Stat 1 test (cancellation flag behavior); Phase 37 does NOT modify
- `hp41-core/tests/synthetic_tests.rs` — v2.0 backward-compat test shape (`v20-autosave.json` fixture load); Phase 37 backward-compat test mirrors this pattern for a v3.0 fixture
- `hp41-core/tests/fixtures/` — directory for save-file fixtures; Phase 37 adds a `v30-autosave.json` fixture
- `hp41-cli/tests/function_matrix_parity.rs` — 3-pool parity walk (Phase 34); Phase 37 verifies it still passes
- `hp41-gui/e2e/smoke.spec.js` — existing E2E spec (WebdriverIO + Mocha); Phase 37 extends with Stat 1 workflow
- `hp41-gui/wdio.conf.cjs` — WebdriverIO config; Phase 37 may need no changes if the existing config covers the extended spec

### hp41-core/src/ops/stat1/ source tree (the coverage-measurement targets)

- `hp41-core/src/ops/stat1/anova.rs` (509 LOC, 8 inline tests)
- `hp41-core/src/ops/stat1/basic_stats.rs` (446 LOC, 14 inline tests)
- `hp41-core/src/ops/stat1/chisqd.rs` (384 LOC, 12 inline tests)
- `hp41-core/src/ops/stat1/distributions.rs` (829 LOC, 47 inline tests)
- `hp41-core/src/ops/stat1/hypothesis.rs` (608 LOC, 16 inline tests)
- `hp41-core/src/ops/stat1/mod.rs` (825 LOC, 2 inline tests)
- `hp41-core/src/ops/stat1/modal.rs` (532 LOC, 10 inline tests)
- `hp41-core/src/ops/stat1/moments.rs` (405 LOC, 8 inline tests)
- `hp41-core/src/ops/stat1/nonparam.rs` (724 LOC, 24 inline tests)
- `hp41-core/src/ops/stat1/normd.rs` (397 LOC, 12 inline tests)
- `hp41-core/src/ops/stat1/rand.rs` (362 LOC, 11 inline tests)
- `hp41-core/src/ops/stat1/regression.rs` (803 LOC, 25 inline tests)

Total: 6824 LOC source, 189 inline tests across 12 files.

### Scripts and CI

- `scripts/check-free42-contamination.sh` — 18-token grep (Phase 33 extended); Phase 37 re-verifies
- `.github/workflows/ci.yml` — CLI + license-audit CI; `license-audit` job runs the contamination guard
- `.github/workflows/ci-gui.yml` — 3-OS matrix (path-filtered) + `e2e-linux` job (Ubuntu, WebdriverIO + tauri-driver)
- `justfile` — `just ci`, `just gui-ci`, `just gui-dev`, `just docs-matrix-check` recipes

### Divergence documentation

- `docs/hp41-stat1-divergences.md` — Phase 35 shipped; 12 `D-35-NN` entries across 3 buckets; Phase 37 may add a behavioral-policy entry for the bounded-iter cancellation waiver (D-37.9)

### HP Stat 1 Pac primary sources (OM-style citations only)

- HP-41C Stat 1 Pac Owner's Manual (HP 00041-90030, 1979) — OM reference for all accuracy oracle derivations
- HP-41C Stat 1 Pac Quick Reference Card (HP 00041-90061, 1979) — mnemonic catalog for xrom_shadowing cross-check

### Project-local CLAUDE.md guidance

- CLAUDE.md `## Git Workflow` — commits use `/git-workflow:commit --with-skills` only; English-only subject + body
- CLAUDE.md `## Quality Gates` — current targets table (coverage, accuracy, panics, cold-start, key latency, contamination, MSRV, CI)
- CLAUDE.md `## Frozen Invariants → Core engine` — `#![deny(clippy::unwrap_used)]` at crate root; test modules `#[allow]` per established pattern

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`hp41-core/tests/math1_op_test_count.rs`** — direct structural template for `stat1_op_test_count.rs`. Key adaptations: scan `xrom.rs` for the `STAT_1.ops` slice (lines 141–186) instead of `math1_resolve` match block; glob `stat1_*.rs` test files + inline tests in `src/ops/stat1/*.rs` (D-37.7 dual-scan); word-boundary matching unchanged.
- **`hp41-core/tests/lint_math1_assertions.rs`** — direct structural template for `lint_stat1_assertions.rs`. Key adaptations: scan both `tests/stat1_*.rs` AND inline `#[cfg(test)] mod tests` in `src/ops/stat1/*.rs` (D-37.8 scope); LINT-EXEMPT annotation convention unchanged; multi-line detect window unchanged.
- **`hp41-core/tests/numerical_accuracy.rs` `case!` macro + `AccuracyCase` struct** — reuse verbatim for Stat 1 cases. Per-case `tol` field carries either `1e-9` (closed-form) or `1e-7` (iterative). The `WIDE_TOL` constant pattern is already established for cases needing wider tolerance.
- **`hp41-core/tests/synthetic_tests.rs::test_backward_compat_v10_load`** — structural template for the v3.0 backward-compat test. Pattern: load a fixture JSON, assert `CalcState` fields, verify no error. Phase 37 adds a `v30-autosave.json` fixture and asserts `xrom_modules` bit 1 is set after `migrate_after_load`.
- **`hp41-gui/e2e/smoke.spec.js`** — existing spec clicks SVG keyboard buttons by `data-key-id`, reads LCD via `data-text` attribute. Stat 1 workflow extension: type `XEQ` + alpha-mode mnemonic entry + x value entry + verify LCD output. The `extractErrMessage` helper (WR-07 fix) is reusable.
- **`hp41-core/src/ops/math1/xrom.rs:141-186` `STAT_1.ops` slice** — the 26-entry authoritative variant list the `stat1_op_test_count.rs` meta-gate scans.
- **`hp41-cli/tests/function_matrix_parity.rs`** — 3-pool walk already covers Stat 1; Phase 37 verifies it still passes (no changes needed).

### Established Patterns

- **Meta-gate-before-coverage pattern (v3.0 Phase 32):** `math1_op_test_count.rs` and `lint_math1_assertions.rs` landed BEFORE coverage-gap tests were written. This ensured new test files were immediately scanned by the meta-gates. Phase 37 follows the same wave ordering.
- **Coverage-measurement-then-targeted-gap-closure (v3.0 Phase 32 RESEARCH.md):** Phase 32 ran `cargo llvm-cov` per-file first, identified the 14 math1 test files' coverage per source file, then wrote targeted error-branch and edge-case test files only for gap files. Phase 37 mirrors this approach.
- **`#[allow(clippy::unwrap_used)]` at file scope in test files:** every test file in `hp41-core/tests/` carries this. Phase 37 test files follow the same pattern.
- **Fixture-based backward-compat tests:** `tests/fixtures/v20-autosave.json` is the v2.0 fixture. Phase 37 adds `tests/fixtures/v30-autosave.json` — a real CalcState JSON from v3.0 with `"xrom_modules": 1`.
- **E2E spec shape:** describe block per workflow, `browser.execute` + `$('[data-key-id="..."]').click()` + `$('[data-testid="lcd-display"]').getAttribute('data-text')` for assertion. The existing `extractErrMessage` helper handles Tauri error shapes.

### Integration Points

- **stat1_op_test_count.rs scan targets:** reads `hp41-core/src/ops/math1/xrom.rs` for STAT_1.ops variant names + globs `hp41-core/tests/stat1_*.rs` + reads `hp41-core/src/ops/stat1/*.rs` inline test modules. Must use `include_str!` for xrom.rs and `std::fs::read_to_string` for glob-matched files (same as math1 analog).
- **lint_stat1_assertions.rs scan targets:** globs `hp41-core/tests/stat1_*.rs` + reads `hp41-core/src/ops/stat1/*.rs` for inline `#[cfg(test)]` modules. Two lint passes: `no_decimal_assert_eq` + `no_manual_tolerance_pattern`.
- **numerical_accuracy.rs inline extension:** ~30 new `case!` entries appended after the existing 768 cases. Test function remains `test_numerical_accuracy_suite`. Pass-rate thresholds: v1.x floor 498/503 preserved, v3.0 floor 763/768 preserved, combined floor ≥ 98 % of total (~798 cases → ≥ 782 pass).
- **E2E smoke.spec.js extension:** new `it()` block inside the existing `describe()` (or a new `describe('Stat 1 Pac', ...)` block). Keyboard interaction: XEQ button → alpha-mode mnemonic typing → numeric entry → R/S → LCD assertion. The modal-prompt flow (ΣNORMD MODE? → mode selection → x entry) needs the exact key sequence mapped from `data-key-id` values.
- **Coverage gate programmatic check:** Phase 37 final plan runs `cargo llvm-cov` and verifies ≥ 95.39 % lines / ≥ 94.26 % regions on `hp41-core` + per-file ≥ 90 % on `stat1/*.rs`. The check is manual (run + verify) rather than a new CI step — the existing `just ci` gate covers clippy/test/audit but not coverage (coverage is measured locally per the v3.0 established workflow).

</code_context>

<specifics>
## Specific Ideas

- **v3.0 save-file fixture (STAT-QUAL-10):** create `hp41-core/tests/fixtures/v30-autosave.json` by serializing a `CalcState` with `xrom_modules: 1` (v3.0 default — only Math 1 enabled). The backward-compat test loads it, calls `migrate_after_load`, asserts `state.xrom_modules == 0b0000_0011` (both Math 1 and Stat 1 enabled), and verifies XEQ "ΣNORMD" resolves via `xrom_resolve`.

- **E2E Stat 1 workflow key sequence (STAT-QUAL-11):** The ΣNORMD upper-tail CDF evaluation of x = 1.96:
  1. Click XEQ button (`data-key-id="xeq"`)
  2. Enter alpha mnemonic: Σ N O R M D (via alpha-mode key clicks)
  3. Wait for modal prompt "ΣNORMD MODE?" on LCD
  4. Press E key (upper-tail CDF mode selection)
  5. Enter 1.96 via digit keys
  6. Press R/S
  7. Assert LCD `data-text` contains `0.0250` (within display precision)

- **STAT-GUI-05 divergence entry (D-37.9):** if documented-waiver approach is used, add a `D-35-13` (or next available number) entry in `docs/hp41-stat1-divergences.md` bucket 3 (Behavioral Policies):
  ```
  ### D-35-13: Bounded-iteration distribution primitives do not wire cancel_requested
  
  **OM citation:** N/A (emulator-internal design choice)
  **Our behavior:** Distribution primitives (norm_cdf_inv_f64, gamma_regularized_f64, beta_regularized_f64) are bounded at ITER_CAP=50 and complete in microseconds. No per-loop cancel_requested check.
  **OM behavior:** N/A (HP-41 hardware has no cancellation mechanism for built-in math)
  **Rationale:** The Phase 31 cancel_requested channel was designed for user-driven OPEN-ENDED iterative paths (INTG/SOLVE/DIFEQ). Bounded 50-iter f64 primitives complete faster than a user can press R/S. See D-36.2, ADR-v3.1-002.
  **See:** hp41-core/src/ops/stat1/distributions.rs (ITER_CAP constant), 36-CONTEXT.md D-36.2
  ```

- **stat1_op_test_count.rs variant detection heuristic:** scan `xrom.rs` for lines containing `Some(Op::` inside the `STAT_1` block (lines 141–186). Extract variant names. Then count test-function-body mentions across: (a) `hp41-core/tests/stat1_*.rs` files AND (b) `#[cfg(test)]` modules inside `hp41-core/src/ops/stat1/*.rs` files. ≥ 5 mentions per variant.

- **Recommended plan wave structure (D-37.10):**
  - **Wave 1:** Plan 37-01 (meta-gates: stat1_op_test_count.rs + lint_stat1_assertions.rs + xrom_shadowing.rs STAT_1 extension)
  - **Wave 2:** Plan 37-02 (coverage measurement + gap analysis → report which files need new tests), Plan 37-03..05 (coverage-gap test files, grouped by algorithmic family), Plan 37-06 (backward-compat test + STAT-GUI-05 resolution + Free42 re-verification)
  - **Wave 3:** Plan 37-07 (numerical accuracy ~30 scipy cases)
  - **Wave 4:** Plan 37-08 (E2E smoke extension + quality-gate graduation + README hard-claim)

</specifics>

<deferred>
## Deferred Ideas

- **Signed binary releases (cargo-dist CLI + tauri-action GUI)** — deferred to v3.1.x / v3.2 per PROJECT.md lock 2026-05-21
- **CI-automated coverage gate** — currently coverage is measured locally per established workflow; a CI step that automatically gates on ≥ 95% is a future ergonomic improvement but not required for v3.1 ship
- **Cross-platform E2E smoke** — Ubuntu-only per D-37.2; macOS/Windows E2E would require headless X11 equivalent setup in CI
- **lint_xrom_assertions.rs unification** — D-37.8 decided on separate files (lint_math1_assertions.rs + lint_stat1_assertions.rs); if a third XROM module lands in v3.2, consider unifying into a single `lint_xrom_assertions.rs` with per-module scan configuration
- **stat1_op_test_count.rs → xrom_op_test_count.rs unification** — same rationale as lint; defer until third module

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 37-test-hardening-quality-gates*
*Context gathered: 2026-05-24*
