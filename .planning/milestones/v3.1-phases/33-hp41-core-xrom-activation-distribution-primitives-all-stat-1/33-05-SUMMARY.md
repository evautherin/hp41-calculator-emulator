---
phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1
plan: 05
subsystem: hp41-core
tags: [stat1, basic-stats, regression, curve-fit, delegate-pattern, anti-duplication, log-linearization, spec-md-drift]

requires:
  - phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1
    plan: 00
    provides: hp41-core/src/ops/stat1/ skeleton + STAT1_MAX_REG + Free42 contamination guard
  - phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1
    plan: 01
    provides: XROM framework activation + STAT_1 const + stat1_resolve + Op::Stat1Stub placeholder
provides:
  - hp41-core/src/ops/stat1/basic_stats.rs (199 LOC production code + 14 unit tests)
  - hp41-core/src/ops/stat1/regression.rs (85 LOC production code + 13 unit tests)
  - Op::SigmaBstat (ΣBSTAT — univariate extended summary; CV + sample mean)
  - Op::SigmaBstg (ΣBSTG — bivariate weighted summary; weighted mean + unweighted mean)
  - Op::SigmaLin / SigmaExp / SigmaLogi / SigmaPow (4 curve-fit accumulators via log-linearization + op_sigma_plus delegate)
  - 6 of 26 STAT_1.ops + stat1_resolve stub references swapped to real Sigma* variants (cumulative 9 of 26 after this plan + Plan 33-04's 3)
affects:
  - 33-03 (parallel wave-2 plan; both depend only on 33-01; no cross-interference — disjoint file touch sets except stat1/mod.rs which is sequentially editable)
  - 33-06 (ΣMMTUG/ΣMMTGD ship to moments.rs which will accumulate Σx³/Σx⁴ into the Stat-1 extended-register block — disjoint from basic_stats.rs's R01-R06-only consumption)
  - 33-07 (parallel wave-2 — hypothesis.rs uses distributions primitives from 33-02 + R01-R06 reads — no shared state with this plan)
  - 33-08 (extends regression.rs in place with ΣMLRXY/MLRXYZ + ΣPOLYP/POLYC + Gauss elimination + RAND/SEED — LOC budget headroom verified at 85 LOC ≤ 100; can grow to ~300 LOC for the multi-predictor + polynomial work)
  - 34 (CLI integration — items 3 of 4-way invariant for the 6 new Sigma* variants)
  - 35 (docs — must amend SPEC.md Req. 7 CV oracle from 0.4083 to the scipy-confirmed 0.5270)
  - 36 (GUI integration — items 4 of 4-way invariant)

tech-stack:
  added: []  # no new runtime deps; pure hp41-core algorithm work
  patterns:
    - "Anti-duplication delegate pattern (PATTERNS.md Pattern 4) — every curve-fit Op transforms stack channels via HpNum::checked_ln then calls op_sigma_plus; the 4 grep-counted occurrences satisfy the CI gate against hand-rolled Σ arithmetic"
    - "Push-twice stack output convention from op_mean / op_sdev — ΣBSTAT / ΣBSTG mirror this for (μ_x in Y, CV/μ_w in X) ordering with LiftEffect::Enable on each push"
    - "Bessel-corrected sample standard deviation σ_x = √((Σx² − (Σx)²/n) / (n − 1)) via checked_sqrt on rust_decimal Decimal — closed-form for ΣBSTAT CV"
    - "SPEC.md drift documentation via inline test-doc comments + module-level `## SPEC.md oracle drift` section (mirrors Plan 33-04's ΣSPEAR/ΣEFXSQ precedent)"

key-files:
  created:
    - hp41-core/src/ops/stat1/basic_stats.rs
    - hp41-core/src/ops/stat1/regression.rs
    - .planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-05-SUMMARY.md
  modified:
    - hp41-core/src/ops/stat1/mod.rs
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-core/src/ops/math1/xrom.rs

key-decisions:
  - "Interpretation A confirmed for ΣLIN/EXP/LOGI/POW: each XEQ call is a PER-POINT ACCUMULATOR. ΣLIN is a pure alias for op_sigma_plus; ΣEXP transforms y → ln y then accumulates; ΣLOGI transforms x → ln x; ΣPOW transforms both. Final (b, intercept) extraction is the user's responsibility via XEQ to the existing v1.x L.R. (op_lr) — this matches the OM contract and reuses the entire v1.x linear-regression toolchain without modification. Interpretation B (compute-from-already-accumulated-block) was rejected because PATTERNS.md Pattern 4 (lines 334–346) explicitly anchors interpretation A as the OM-faithful contract."
  - "HP-41 Σ+ stack convention clarified: state.stack.x is the INDEPENDENT variable (x-coordinate) and state.stack.y is the DEPENDENT variable (y-coordinate). This is the convention encoded in v1.x ops/stats.rs lines 28-29 + 64-65 (op_sigma_plus reads `let x = state.stack.x.clone(); let y = state.stack.y.clone()`). PATTERNS.md line 339-345 used `state.stack.y` to mean the y-coordinate (dependent) — consistent with v1.x. Initial test wiring inverted this (caught immediately as 7 test failures with slope = 1/expected); fixed via Rule-1 bug fix during Task 2 implementation."
  - "ΣBSTAT CV formula uses the standard sample CV_x = σ_x / μ_x with Bessel-corrected σ_x. SPEC.md Req. 7 claims CV = 0.4083 for x=[1..5], y=[10..50]; manual + numpy.std(x, ddof=1)/numpy.mean(x) both confirm 0.5270462766947299. SPEC.md value 0.4083 reverse-engineers to σ_x / √(Σx) = 1.5811/√15 — a non-standard ratio. Op ships the mathematically correct 0.5270 and flags the discrepancy for Phase 35 (STAT-DOC) amendment — same convention as Plan 33-04 ΣSPEAR (0.8 vs SPEC 0.7) and ΣEFXSQ (7.0 vs SPEC 1.667)."
  - "ΣBSTG semantic: bivariate weighted summary. Interprets the v1.x Σ-block as (data, weight) pairs where R02 = Σx (data totals), R05 = Σy (interpreted as Σw, weight totals), R06 = Σxy (interpreted as Σ(x·w)). Returns weighted mean μ_w = Σ(x·w)/Σw in X and unweighted μ_x = Σx/n in Y for comparison. This is OM-faithful per p. 11–14 and matches numpy.average(x, weights=w) on the SPEC.md Req. 7 dataset (3.6667)."
  - "Two-level tolerance per SPEC.md Req. 46 applied per Op family. Closed-form Ops (ΣBSTAT, ΣBSTG, ΣLIN, ΣLOGI happy-path tests) use 1e-9; transcendental-transform Ops (ΣEXP with f64-constructed e, ΣPOW with double-ln chain) use 1e-7 to absorb f64 round-off in the test-input construction (Decimal::from_f64_retain on e ≈ 2.71828 accumulates ULPs that propagate through ln/exp inverse). The Op arithmetic itself uses rust_decimal exclusively — tolerance relaxation is purely a test-construction artifact, not a precision degradation."
  - "regression.rs LOC budget enforcement: trimmed from 137 LOC to 85 LOC by consolidating per-Op doc-comments from full Errors/Source sections to integrated blocks. The 15-LOC headroom under the plan's 100-LOC cap leaves Plan 33-08 ~215 LOC to extend the same file with ΣMLRXY (3×3 Gauss), ΣMLRXYZ (4×4 Gauss), ΣPOLYP modal opener + (d+1)×(d+1) Gauss, and ΣPOLYC Horner evaluation — well within the math1/ per-file 300-LOC convention."

patterns-established:
  - "Σ+ accumulator delegate (op_sigma_lin pure alias + 3 log-linearizing variants) — every Plan 33-08 multi-predictor accumulator (ΣMLRXY/MLRXYZ) MUST follow this same delegate pattern (no hand-rolled Σ arithmetic in regression.rs)"
  - "Bessel-corrected sample variance via checked_sqrt — reusable for any future univariate-summary Op in basic_stats.rs / moments.rs (Plan 33-06 ΣMMTUG can call back into op_sigma_bstat's σ_x formula for the kurtosis denominator σ⁴)"
  - "Two-level tolerance per Op family — Plan 33-08 polynomial-regression tests will need 1e-7 throughout because of the multi-step Gauss-elimination chain"

requirements-completed:
  - STAT-UNI-01  # ΣBSTAT + ΣBSTG univariate/weighted extended summaries
  - STAT-REG-01  # ΣLIN linear regression accumulator
  - STAT-REG-02  # ΣEXP exponential curve-fit accumulator
  - STAT-REG-03  # ΣLOGI logarithmic curve-fit accumulator
  - STAT-REG-04  # ΣPOW power curve-fit accumulator

metrics:
  duration: ~55min
  completed: 2026-05-22
  tasks_total: 2
  tasks_completed: 2
  files_created: 2
  files_modified: 4
  commits: 2
  tests_added: 27  # 8 bstat + 6 bstg + 13 regression (3 lin + 3 exp + 3 logi + 4 pow incl. edge cases)
---

# Phase 33 Plan 33-05: ΣBSTAT/ΣBSTG + ΣLIN/EXP/LOGI/POW Summary

**Six low-risk closed-form Ops ship in parallel with Plans 33-03 / 33-04 / 33-06 / 33-07 — two univariate / weighted summaries (ΣBSTAT, ΣBSTG) reading R01–R06 read-only, four log-linearizing curve-fit accumulators (ΣLIN, ΣEXP, ΣLOGI, ΣPOW) delegating to op_sigma_plus via the anti-duplication pattern. 27 unit tests, one SPEC.md oracle drift documented (CV = 0.5270 not 0.4083), one stack-convention bug fixed inline. Cumulative 9 of 26 Op::Stat1Stub references swapped after this plan.**

## Performance

- **Duration:** ~55 min (incl. stack-convention bug fix in Task 2 + LOC-budget trim)
- **Completed:** 2026-05-22
- **Tasks:** 2 (both completed atomically)
- **Files:** 2 created, 4 modified
- **Commits:** 2 (one per task; one Rule-1 bug fix folded into Task 2 commit)
- **Tests added:** 27 (8 ΣBSTAT + 6 ΣBSTG + 13 regression)

## Accomplishments

- **Six real `Op::Sigma*` variants ship.** `Op::SigmaBstat`, `Op::SigmaBstg`, `Op::SigmaLin`, `Op::SigmaExp`, `Op::SigmaLogi`, `Op::SigmaPow` all added to the central `Op` enum (`hp41-core/src/ops/mod.rs`) and land in BOTH `dispatch()` and `execute_op()` (4-way exhaustive-match invariant items 1+2). CLI + GUI `op_display_name` arms (items 3+4) deferred to Phase 34/36 per the documented Plan 33-01 known intentional CI break.
- **Six of 26 `Op::Stat1Stub` references swapped in `STAT_1.ops` + `stat1_resolve`** (bidirectional consistency CI-gated by `stat1_ops_mnemonics_resolve_consistently`). After this plan: 17 stub references remain (Plans 33-03 / 33-06 / 33-07 / 33-08 own the rest).
- **Anti-duplication pattern enforced for curve fits.** `grep -c 'crate::ops::stats::op_sigma_plus' regression.rs` returns 10 occurrences (4 in pub fns + 6 in tests), well above the ≥ 4 CI gate. Zero hand-rolled Σ arithmetic in `regression.rs`; every accumulator routes through the existing v1.x `op_sigma_plus` post-transform.
- **SPEC.md Oracle Drift #3 resolved (ΣBSTAT Req. 7 CV):** SPEC.md states `CV = 0.4083` for `x=[1..5], y=[10..50]`. Manual + `numpy.std([1,2,3,4,5], ddof=1) / numpy.mean([1,2,3,4,5])` both confirm `0.5270462766947299`. The SPEC value `0.4083` ≈ `σ_x / √(Σx)` — a non-standard ratio. Tests assert the mathematically correct 0.5270; SPEC.md amendment gated to Phase 35 (STAT-DOC) — discrepancy documented at the module-level `## SPEC.md oracle drift` section in `basic_stats.rs` and inline in the test comments. The `μ_w = 3.6667` part of SPEC.md Req. 7 is correctly reproduced by ΣBSTG (`Σ(x·w)/Σw = 550/150`).
- **HP-41 Σ+ stack convention clarified during Task 2 implementation.** Initial regression.rs Op bodies and test helpers misinterpreted `state.stack.x` as the DEPENDENT (y-coord) variable; this produced 7 test failures with slope = 1/expected. Inspection of v1.x `op_sigma_plus` (`ops/stats.rs:28-29, 64-65`) confirmed `state.stack.x` = INDEPENDENT (x-coord), `state.stack.y` = DEPENDENT (y-coord); fix was Rule-1 (bug in initial implementation) and the corrected stack channels are documented in each Op's doc-comment.
- **Free42 contamination guard:** still clean. No GPL-tainted symbols introduced; the disclaim headers in `basic_stats.rs` + `regression.rs` are byte-for-byte parity with `math1/*.rs` / `stat1/distributions.rs` / `stat1/nonparam.rs`.
- **LOC budget hit:** `basic_stats.rs` = 199 LOC ≤ 300 budget; `regression.rs` = 85 LOC ≤ 100 budget (leaving 215 LOC headroom for Plan 33-08 ΣMLRXY/MLRXYZ + ΣPOLYP/POLYC + Gauss elimination).

## Task Commits

Each task committed atomically via `git commit` with the CLAUDE.md "Git Workflow" English Conventional Commits style:

1. **Task 1: ΣBSTAT + ΣBSTG closed-form univariate / weighted summaries** — `ce03e6e` (feat)
2. **Task 2: ΣLIN + ΣEXP + ΣLOGI + ΣPOW curve-fit accumulators via op_sigma_plus delegate** — `2ed841d` (feat)

(Plan metadata commit — this SUMMARY — follows.)

## Files Created / Modified

### Created (2 files)

- `hp41-core/src/ops/stat1/basic_stats.rs` (~290 lines total; **199 LOC production code** ≤ 300 budget; 14 unit tests). Disclaim header byte-for-byte parity with sibling `stat1/nonparam.rs`. Two public `pub fn op_sigma_*` entry points + one private helper (`require_stat1_size_floor`). Module-level `## SPEC.md oracle drift` block documents the CV = 0.5270 vs SPEC 0.4083 discrepancy.
- `hp41-core/src/ops/stat1/regression.rs` (~210 lines total; **85 LOC production code** ≤ 100 budget per plan, leaving headroom for Plan 33-08; 13 unit tests). Disclaim header parity. Four public `pub fn op_sigma_*` entry points; the four-row transform table is documented at module level.

### Modified (4 files)

- `hp41-core/src/ops/stat1/mod.rs` (+2 / -2 lines): uncommented `pub mod basic_stats;` and `pub mod regression;` per the Plan 33-00 phased uncomment plan.
- `hp41-core/src/ops/mod.rs` (+72 / -0 lines): six new `Op` variants (`SigmaBstat`, `SigmaBstg`, `SigmaLin`, `SigmaExp`, `SigmaLogi`, `SigmaPow`) with substantial doc-comments + six dispatch arms.
- `hp41-core/src/ops/program.rs` (+8 / -0 lines): six execute_op arms routing through `dispatch()` (closed-form Ops follow the Plan 28-10 Triangle/TRANS pattern — pure-data ops execute fully from dispatch context).
- `hp41-core/src/ops/math1/xrom.rs` (+8 / -6 lines): six `Op::Stat1Stub` references in `STAT_1.ops` swapped to real `Op::Sigma*` variants; the corresponding six arms in `stat1_resolve` updated in lockstep. Bidirectional consistency CI-gated by `stat1_ops_mnemonics_resolve_consistently`.

## Decisions Made

### Interpretation A (accumulator) confirmed for ΣLIN/EXP/LOGI/POW

The PATTERNS.md "Pattern 4: Σ-Register Delegate" excerpt (lines 334–346) anchors interpretation A as the OM-faithful contract: each XEQ call is a per-point ACCUMULATOR. ΣLIN is a pure alias for `op_sigma_plus`; ΣEXP transforms y → ln y then accumulates; ΣLOGI transforms x → ln x; ΣPOW transforms both. After the user has called the accumulator once per data point, they extract (b, intercept) via the existing v1.x `op_lr` and post-process the intercept for EXP/POW (a_final = e^intercept). Interpretation B (compute-from-already-accumulated-Σ-block) was rejected because it would require either (a) a separate "compute" Op family that bypasses the standard XEQ → Σ+ user workflow, or (b) modifying `op_sigma_plus` itself to know about transforms — both violate the anti-duplication invariant.

The cost is that the user must understand that ΣEXP's `op_lr` result needs `e^intercept` for the final amplitude; this matches OM 00041-90030 §ΣEXP user-flow documentation per Plan 33-08's pending OM verification of the LR-result post-processing.

### ΣBSTAT CV oracle discrepancy with SPEC.md Req. 7

The SPEC.md Req. 7 claim that `CV = 0.4083` for the canonical `x=[1..5], y=[10..50]` dataset is mathematically incorrect for the standard sample `CV_x = σ_x / μ_x`:

- `σ_x²` (Bessel-corrected) = `(Σx² − (Σx)²/n) / (n − 1) = (55 − 45) / 4 = 2.5`
- `σ_x = √2.5 ≈ 1.5811388300841898`
- `μ_x = Σx/n = 15/5 = 3.0`
- `CV_x = σ_x / μ_x ≈ 0.5270462766947299`

The value `0.4083` reverse-engineers to `σ_x / √(Σx) = 1.5811 / √15 = 0.4083` — a non-standard ratio that may have been a calculator-display approximation or transcription error in the SPEC drafting. `numpy.std([1,2,3,4,5], ddof=1) / numpy.mean([1,2,3,4,5])` independently confirms `0.5270462766947299`.

ΣBSTAT ships the mathematically correct `0.5270` value. SPEC.md amendment to Req. 7 is gated to Phase 35 (STAT-DOC) — same convention as Plan 33-04's ΣSPEAR (0.8 vs SPEC 0.7) and ΣEFXSQ (7.0 vs SPEC 1.667). The `μ_w = 3.6667` portion of SPEC.md Req. 7 IS correctly reproduced by ΣBSTG (`Σ(x·w)/Σw = 550/150`); only the CV value diverges.

### Two-level tolerance applied per Op family

SPEC.md Req. 46 establishes two tolerance buckets: 1e-9 for closed-form Ops, 1e-7 for iterative ops. ΣBSTAT, ΣBSTG, ΣLIN, ΣLOGI (with integer-valued log inputs) are closed-form on rust_decimal arithmetic — they hit 1e-9 reliably. ΣEXP (with f64 `std::f64::consts::E` accumulating multiplicative round-off as `e * e * e`) and ΣPOW (with double-ln chains on f64-constructed test inputs) require 1e-7 to absorb the f64 → Decimal round-trip round-off accumulated in test-input construction.

The Op-internal arithmetic itself uses `rust_decimal` exclusively via `HpNum::checked_ln` / `checked_exp` — no precision degradation in the production code. The 1e-7 relaxation applies ONLY to tests where the test author had to construct transcendental input values from f64 constants; the relaxation is a test-construction artifact, not a precision claim about the Op.

### regression.rs LOC budget enforcement (≤ 100 LOC for this plan)

After the initial Task 2 implementation landed at 137 LOC (with full per-Op doc-comments containing separate Errors / Source / behavior sections), the trim path consolidated the doc-comments into integrated blocks:

1. Module-level doc-comment trimmed from 47 → 24 lines (eliminated the verbose "interpretation A vs B" justification, retaining only the canonical transform table)
2. Per-Op doc-comments trimmed from ~12 lines each to ~5 lines each (Errors + Source sections consolidated as single inline statements)

Final production-code LOC: **85** ≤ 100 budget. Plan 33-08 has ~215 LOC headroom in `regression.rs` for ΣMLRXY (2-predictor 3×3 Gauss), ΣMLRXYZ (3-predictor 4×4 Gauss), ΣPOLYP modal opener + (d+1)×(d+1) Gauss elimination, ΣPOLYC Horner evaluation — well within the math1/ per-file 300-LOC convention even with all four additions.

## Deviations from Plan

**Rule 1 — Bug fix during Task 2:** initial regression.rs implementation inverted the HP-41 Σ+ stack convention (treated `state.stack.x` as DEPENDENT instead of INDEPENDENT). Fixed inline during the test-failure investigation: the v1.x `op_sigma_plus` body in `ops/stats.rs:28-29, 64-65` (`let x = state.stack.x.clone(); let y = state.stack.y.clone()`) unambiguously establishes `stack.x` = x-coordinate (independent), `stack.y` = y-coordinate (dependent). PATTERNS.md line 339-345 was correct ("transform Y in place" = transform y-coordinate = transform `state.stack.y`); the initial misreading of "Y channel" as "X register" was the error. All four curve-fit Ops + all four affected test helpers + three domain-error edge case tests updated; final fix folded into the Task 2 commit message (the wrongly-implemented Op never landed as a committed regression).

No other deviations. Both deferred Phase-35 SPEC.md amendments (CV oracle drift, μ_w confirmation) were foreseen by the plan's `<action>` block line 127, which explicitly anticipated the discrepancy and instructed: "if SPEC.md value disagrees with scipy, document the discrepancy inline as in Plan 33-04 ΣSPEAR. Use the OM-confirmed formula and value." This is the documented contract; not a deviation from the EXECUTION plan.

## Auth Gates Encountered

None.

## Plan-level Verification — all green

- `cargo check -p hp41-core` exits 0
- `cargo clippy -p hp41-core --tests -- -D warnings` exits 0
- `cargo test -p hp41-core` — **1859 passed, 1 ignored** across 71 test suites (was 1832 at end of Plan 33-04; +27 net new tests this plan)
- `cargo test -p hp41-core --lib ops::stat1::basic_stats::tests` — **14 passed** (8 ΣBSTAT + 6 ΣBSTG)
- `cargo test -p hp41-core --lib ops::stat1::regression::tests` — **13 passed** (3 ΣLIN + 3 ΣEXP + 3 ΣLOGI + 4 ΣPOW including domain edge cases)
- `cargo test -p hp41-core --lib ops::math1::xrom::tests::stat1_ops_mnemonics_resolve_consistently` — passes (bidirectional STAT_1.ops ↔ stat1_resolve consistency preserved through 6 stub swaps)
- `cargo test -p hp41-core --test xrom_shadowing` — passes (STAT_1 disjointness + bit-1 isolation invariants intact)
- `bash scripts/check-free42-contamination.sh` — exits 0
- `basic_stats.rs` production-code: **199 LOC** ≤ 300 budget
- `regression.rs` production-code: **85 LOC** ≤ 100 budget per plan
- `grep -c 'crate::ops::stats::op_sigma_plus' regression.rs` returns **10** (≥ 4 anti-duplication CI gate)
- `grep -c 'pub fn op_sigma_bstat\|pub fn op_sigma_bstg' basic_stats.rs` returns **2**
- `grep -cE 'pub fn op_sigma_(lin|exp|logi|pow)' regression.rs` returns **4**
- 6 of 26 `Op::Stat1Stub` references swapped in xrom.rs (slice + resolver in lockstep)
- **Cumulative 9 of 26 stubs swapped** (3 from Plan 33-04 + 6 from this plan); 17 stubs remain for Plans 33-03 / 33-06 / 33-07 / 33-08

## Known Intentional CI Break

Per Plan 33-01's known break: `cargo check -p hp41-cli` and `cargo check -p hp41-gui` still fail with `non-exhaustive patterns: &Op::Stat1Stub, &Op::SigmaSpear, &Op::SigmaXsqev, &Op::SigmaEfxsq, &Op::SigmaBstat, &Op::SigmaBstg, &Op::SigmaLin, &Op::SigmaExp, &Op::SigmaLogi, &Op::SigmaPow not covered`. Phase 34 + 36 will surface the missing arms in `op_display_name` and add the display strings. This is the documented contract; not a regression.

`cargo test -p hp41-core` is the executable verification surface for this plan and is fully green.

## Self-Check: PASSED

Created files exist:
- `hp41-core/src/ops/stat1/basic_stats.rs` (✓ created; 14 tests green)
- `hp41-core/src/ops/stat1/regression.rs` (✓ created; 13 tests green)
- `.planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-05-SUMMARY.md` (this file — ✓ created)

Commit hashes verified in `git log --oneline`:
- ✅ `ce03e6e` feat(33-05): add ΣBSTAT + ΣBSTG closed-form univariate / weighted summaries
- ✅ `2ed841d` feat(33-05): add ΣLIN / ΣEXP / ΣLOGI / ΣPOW curve-fit accumulators

## Next-phase Readiness

- **Plan 33-03 (parallel wave-2)** is unaffected. Both plans depend only on 33-01; the file-level touch sets are disjoint (33-03 touches `stat1/normd.rs` + `stat1/chisqd.rs` + the modal carrier; 33-05 touches `stat1/basic_stats.rs` + `stat1/regression.rs`). When both plans' branches merge, the only conflict points are (a) line-level merges in `STAT_1.ops` + `stat1_resolve` (both swap different stub references), (b) `Op::*` enum variant insertions in `ops/mod.rs`, and (c) `pub mod` line additions in `ops/stat1/mod.rs` — all mechanically resolvable.
- **Plan 33-06 (ΣMMTUG/MMTGD + ANOVA + ΣCTKKK/CTKK)** can extend `basic_stats.rs` (or sibling `moments.rs`) with third/fourth-moment Ops; the ΣBSTAT σ_x formula is reusable as the kurtosis denominator σ⁴. LOC headroom in `basic_stats.rs`: 101 lines (199 of 300 used). ΣMMTUG's extended Σx³/Σx⁴ register block belongs to Plan 33-06's new register slots (NOT R01-R06).
- **Plan 33-08 (ΣMLRXY/MLRXYZ + ΣPOLYP/POLYC + RAND/SEED)** will extend `regression.rs` in place. The 85-LOC current footprint leaves ~215 LOC headroom under the math1/ 300-LOC per-file convention — enough for all four Plan 33-08 additions. The delegate pattern established here (transform stack channels then call op_sigma_plus) must be maintained for ΣMLRXY/MLRXYZ multi-predictor accumulators.
- **Phase 34 (CLI integration)** must add `op_display_name` arms for the 6 new variants. Mnemonics: `"ΣBSTAT"`, `"ΣBSTG"`, `"ΣLIN"`, `"ΣEXP"`, `"ΣLOGI"`, `"ΣPOW"` (Σ encoded as `\u{03A3}` per the established Plan-28 convention).
- **Phase 35 (docs)** must amend SPEC.md Req. 7:
  - Change "CV = 0.4083" to "CV = 0.5270462766947299" (cite numpy/scipy oracle; preserve μ_w = 3.6667 which is correct)
  - The amendment slots cleanly alongside the Plan 33-04 amendments (ΣSPEAR 0.8, ΣEFXSQ 7.0) — three SPEC.md oracle corrections in total to land in STAT-DOC.
- **Phase 36 (GUI integration)** mirrors Phase 34 for `op_display_name`.

No blockers. Wave 2 (Plans 33-03 + 33-04 + 33-05) progressing in parallel as designed; Plans 33-06 + 33-07 + 33-08 can resume from the cumulative 9/26 stub-swap baseline.

---
*Phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1*
*Plan: 05 (Wave 2 — univariate summaries + log-linearizing curve fits)*
*Completed: 2026-05-22*
