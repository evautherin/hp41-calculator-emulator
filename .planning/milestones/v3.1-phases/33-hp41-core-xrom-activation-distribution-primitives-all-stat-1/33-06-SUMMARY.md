---
phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1
plan: 06
subsystem: hp41-core
tags: [stat1, moments, anova, ancova, contingency-table, om-register-layout, p21-named-consts, stat-uni-04, spec-md-drift]

requires:
  - phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1
    plan: 00
    provides: hp41-core/src/ops/stat1/ skeleton + STAT1_MAX_REG + per-program SIZE-floor consts + Free42 contamination guard
  - phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1
    plan: 01
    provides: XROM framework activation + STAT_1 module const + stat1_resolve + bit-1 arm + Op::Stat1Stub placeholder
  - phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1
    plan: 04
    provides: hp41-core/src/ops/stat1/nonparam.rs with ΣSPEAR + ΣXSQEV + ΣEFXSQ Ops (extended in this plan with ΣCTKKK + ΣCTKK)
  - phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1
    plan: 05
    provides: hp41-core/src/ops/stat1/basic_stats.rs + regression.rs (cumulative stub-swap baseline of 9 before this plan)
provides:
  - hp41-core/src/ops/stat1/moments.rs (199 LOC production + 8 unit tests)
  - hp41-core/src/ops/stat1/anova.rs (295 LOC production + 8 unit tests)
  - hp41-core/src/ops/stat1/nonparam.rs extended (+95 LOC production + 7 new ΣCTKK tests, total 24 nonparam tests)
  - hp41-core/src/ops/stats.rs::op_sigma_minus extended for STAT-UNI-04 [C] correction-key round-trip
  - hp41-core/src/ops/stat1/mod.rs::STAT1_MMTUG_CUBE_REG + STAT1_MMTUG_QUAD_REG (2 new per-slot named consts for Σx³/Σx⁴)
  - hp41-core/src/ops/stat1/mod.rs::STAT1_AOV_* family (8 new per-slot named consts for ANOVA per-group blocks)
  - hp41-core/src/ops/stat1/mod.rs::STAT1_CTKKK_* family (5 new per-slot named consts for contingency-table cells)
  - Op::SigmaMmtug + Op::SigmaMmtgd (third + fourth moments accumulators)
  - Op::SigmaAovone + Op::SigmaAovtwo + Op::SigmaAnocov (ANOVA family F-ratios)
  - Op::SigmaCtkkk + Op::SigmaCtkk (contingency-table χ²)
  - 7 of 26 STAT_1.ops + stat1_resolve stub references swapped to real Sigma* variants (cumulative 16 of 26 after this plan + Plans 33-03/04/05)
affects:
  - 33-07 (ΣPTST + ΣTSTAT — hypothesis tests; may consume compute_moments helper from this plan for μ₃ / γ₁ if Welch's-skewness-correction is invoked)
  - 33-08 (extends regression.rs with ΣMLRXY/MLRXYZ + ΣPOLYP/POLYC + Gauss elimination — independent of this plan's deliverables)
  - 34 (CLI integration — items 3 of 4-way invariant for the 7 new Sigma* variants)
  - 35 (docs — must amend SPEC.md Req. 8 oracle μ₄ = 33 → 120.8625; Req. 11 F = 100 → 50; Req. 28 χ² ≈ 4.286 → 2.8; Req. 29 χ² ≈ 0.397 → 0.7937)
  - 36 (GUI integration — items 4 of 4-way invariant)

tech-stack:
  added: []  # pure hp41-core algorithm work; no new runtime deps
  patterns:
    - "Atomic-write discipline extended to extended-slot writers (Pitfall 5): compute Σx³/Σx⁴ before either register write; ANOVA two-pass loops compute totals first then SSB/SSW; contingency-table reducer computes marginals + grand total before χ² accumulator"
    - "Anti-duplication via op_sigma_plus delegate for ΣMMTUG (PATTERNS.md Pattern 4); ΣMMTGD writes all five slots atomically without delegate (frequency-weighted accumulator has no v1.x analog)"
    - "Shared contingency-table reducer (compute_contingency_chi_sq) — both ΣCTKKK and ΣCTKK route through the same r×c χ² formula with only the dimension cap differing per Op (mirrors compute_chi_square_from_counts pattern from Plan 33-04)"
    - "STAT-UNI-04 backward-compat: op_sigma_minus extension gated by `state.regs.len() > STAT1_MAX_REG` check — v1.x users under shrunk SIZE never touch the extended slots, preserving the Plan 22 D-22.11.1 SIZE-shrink contract"
    - "Two-pass ANOVA accumulator (first pass: grand totals; second pass: SSB/SSW) keeps the implementation read-only over input slots and atomic on grand-block writes — mirrors PATTERNS.md atomic-write discipline at the function level"
    - "P21 mitigation enforced for every register access ≥ R07: production-code grep against `state\\.regs\\[[7-9]\\]|state\\.regs\\[1[0-9]\\]` returns ZERO matches in moments.rs / anova.rs / nonparam.rs (test fixtures excluded)"

key-files:
  created:
    - hp41-core/src/ops/stat1/moments.rs
    - hp41-core/src/ops/stat1/anova.rs
    - .planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-06-SUMMARY.md
  modified:
    - hp41-core/src/ops/stat1/mod.rs (added MMTUG + AOV + CTKKK per-slot consts; uncommented anova + moments submodule decls)
    - hp41-core/src/ops/stat1/nonparam.rs (added ΣCTKKK + ΣCTKK via shared compute_contingency_chi_sq reducer)
    - hp41-core/src/ops/stats.rs (extended op_sigma_minus for STAT-UNI-04 round-trip)
    - hp41-core/src/ops/mod.rs (added 7 new Sigma* Op variants + dispatch arms)
    - hp41-core/src/ops/program.rs (added 7 new Sigma* execute_op arms)
    - hp41-core/src/ops/math1/xrom.rs (7 STAT_1.ops + 7 stat1_resolve stub-to-real swaps)

key-decisions:
  - "ΣMMTUG/ΣMMTGD register layout decision: Σx³ at R07 (STAT1_MMTUG_CUBE_REG) and Σx⁴ at R08 (STAT1_MMTUG_QUAD_REG) within SIZE 012 block. R01..R06 reserved for the v1.x Σ-block delegated to op_sigma_plus per anti-duplication. R09..R11 are program-internal scratch per OM p. 15."
  - "ΣMMTGD writes all five Σ-block slots atomically (Σ(f·x²), Σ(f·x), Σf=N, Σ(f·x³), Σ(f·x⁴)) without delegating to op_sigma_plus — the delegate has no frequency-weighting API. Atomic-write discipline preserved by computing all five contributions before any register update."
  - "STAT-UNI-04 [C] correction-key extension landed in-place in `hp41-core/src/ops/stats.rs::op_sigma_minus` (Rule 3 — stats.rs is v1.x code, NOT in the math1/ freeze list per CLAUDE.md). Chose IN-PLACE EDIT over a new `op_sigma_minus_stat1` function because: (1) STAT-UNI-04 is the universal [C] correction-key behavior — splitting it would create per-loaded-XROM dispatch complexity; (2) the backward-compat guard `state.regs.len() > STAT1_MAX_REG` means v1.x users (with R01..R06 only) never touch the extended slots; (3) ΣMMTUG accumulation only writes the extended slots when SIZE supports them, so the symmetric reversal is also conditional."
  - "ΣAOVONE per-group block stride 4 (Σxᵢ, Σxᵢ², nᵢ, scratch) at base R04 — accommodates STAT1_AOV_KMAX = 4 groups within SIZE 020 (R00..R19). Layout re-derived from OM p. 20 Inputs section. ANOVA grand-block (R01..R03) is RECOMPUTED from per-group blocks for atomic correctness — Op writes R01..R03 with the recomputed values for downstream consumers."
  - "ΣAOVONE oracle drift: SPEC.md Req. 11 claims F = 100.0 for [1..5]/[6..10]/[11..15] groups. Manual derivation (SSB=250, SSW=30, df_between=2, df_within=12, F=125/2.5=50) and scipy.stats.f_oneway both confirm F=50.0. SPEC's 100.0 value reverse-engineers to non-standard normalization. Test asserts the scipy-correct 50.0; SPEC.md amendment gated to Phase 35 (STAT-DOC) per the same convention as Plans 33-04 ΣSPEAR/ΣEFXSQ + 33-05 CV drifts."
  - "ΣMMTUG μ₄ oracle drift: SPEC.md Req. 8 claims μ₄ = 33.0 (or 70.3725 in a sister-doc reading); scipy.stats.moment([1..10], moment=4) and the population-estimator manual derivation (Σ(devⁱ⁴)/n = 1208.625/10 = 120.8625) both confirm μ₄ = 120.8625. γ₂ matches SPEC's ≈ −1.224 (= μ₄/μ₂² − 3 = 120.8625/68.0625 − 3). Test asserts the scipy-correct 120.8625; Phase 35 (STAT-DOC) amendment gated."
  - "ΣCTKKK / ΣCTKK oracle drift: SPEC.md Req. 28 claims χ² ≈ 4.286 for 2×3 [[10,20,30],[40,50,60]]; scipy.stats.chi2_contingency (correction=False) and manual derivation both confirm χ² ≈ 2.8. SPEC.md Req. 29 claims χ² ≈ 0.397 for 2×2 [[10,20],[30,40]]; scipy returns ≈ 0.7937. OM p. 60 is silent on Yates' correction — Op-side decision: NO Yates' correction (oracle = scipy correction=False) since OM does not document it. SPEC.md amendments gated to Phase 35."
  - "ΣCTKKK 2×3 oracle tolerance loosened from 1e-9 to 1e-7 per SPEC.md Req. 46 'iterative cross-product chains' policy — the chained Decimal divisions for non-integer expected counts (e.g. 60·50/210 = 14.2857...) accumulate last-digit rounding through the rust_decimal 10-significant-digit floor. Closed-form integer-input case (2×2 with integer expected) holds at 1e-9."
  - "Three commits per task (a3aeb64, e00f837, 02820a9) achieved via incremental restoration: revert to baseline → apply Task 1 changes only → commit → apply Task 2 → commit → apply Task 3 → commit. Each intermediate state compiles green via `cargo check -p hp41-core` and tests green via `cargo test -p hp41-core --lib ops::stat1::<module>`."

patterns-established:
  - "compute_moments(&CalcState) -> (μ₃, μ₄, γ₁, γ₂) public helper — Plan 33-07 hypothesis tests can reuse for skewness-corrected t-statistic variants if needed"
  - "Two-pass accumulator pattern (first pass: grand totals; second pass: derived statistics) — Plan 33-08 ΣMLRXY/MLRXYZ multiple-regression Ops can mirror for the normal-equation building phase"
  - "Shared contingency-table reducer (compute_contingency_chi_sq) — extensible to higher-dimensional contingency tables in future milestones if needed"

requirements-completed:
  - STAT-UNI-02  # ΣMMTUG + ΣMMTGD third + fourth moments
  - STAT-UNI-04  # [C] correction-key round-trip for univariate accumulation
  - STAT-AOV-01  # ΣAOVONE one-way ANOVA
  - STAT-AOV-02  # ΣAOVTWO two-way ANOVA
  - STAT-AOV-03  # ΣANOCOV one-way ANCOVA
  - STAT-AOV-04  # OM-verified ANOVA register layout (P21 mitigation via named consts)
  - STAT-HYP-05  # ΣCTKKK general r×c contingency χ²
  - STAT-HYP-06  # ΣCTKK smaller-table contingency χ²

metrics:
  duration: ~95min
  completed: 2026-05-22
  tasks_total: 3
  tasks_completed: 3
  files_created: 2
  files_modified: 6
  commits: 3
  tests_added: 23  # 8 moments + 8 anova + 7 nonparam-ctkk additions
---

# Phase 33 Plan 06: ΣMMTUG/MMTGD + ANOVA Family + ΣCTKKK/CTKK Summary

**Wave 3 OM-register-layout-dependent cluster ships — third + fourth moment accumulators (ΣMMTUG, ΣMMTGD), one-way + two-way + ANCOVA F-ratio Ops (ΣAOVONE, ΣAOVTWO, ΣANOCOV), and r×c + 2×2 contingency-table χ² Ops (ΣCTKKK, ΣCTKK). STAT-UNI-04 [C] correction-key round-trip lands in `op_sigma_minus` for symmetric reversal of the extended Σx³ / Σx⁴ slots. 23 new unit tests; four SPEC.md oracle drifts documented and gated to Phase 35 (STAT-DOC). Cumulative 16 of 26 `Op::Stat1Stub` references swapped after this plan.**

## Performance

- **Duration:** ~95 min (incl. doc-comment trim cycles for the 300-LOC budget acceptance; per-task commit reconstruction)
- **Completed:** 2026-05-22
- **Tasks:** 3 (all completed atomically with green compile + tests at every commit boundary)
- **Files:** 2 created (moments.rs, anova.rs), 6 modified
- **Commits:** 3 (one per task; reconstructed via baseline-restore-and-replay so each intermediate state compiles green)
- **Tests added:** 23 (8 moments + 8 anova + 7 contingency-table additions to nonparam.rs)

## Accomplishments

- **Seven real `Op::Sigma*` variants ship.** `Op::SigmaMmtug`, `Op::SigmaMmtgd`, `Op::SigmaAovone`, `Op::SigmaAovtwo`, `Op::SigmaAnocov`, `Op::SigmaCtkkk`, `Op::SigmaCtkk` all added to the central `Op` enum and land in BOTH `dispatch()` (item 1) and `execute_op()` (item 2 of the 4-way exhaustive-match invariant). Items 3+4 (CLI + GUI `op_display_name`) remain deferred to Phase 34/36 per the documented Plan 33-01 known intentional CI break.
- **Seven of 26 `Op::Stat1Stub` references swapped** in `STAT_1.ops` slice + `stat1_resolve` (bidirectional consistency CI-gated by `stat1_ops_mnemonics_resolve_consistently`). After this plan: **16 of 26 stubs swapped**; 10 stub references remain (covering ΣMLRXY, ΣMLRXYZ, ΣPOLYP, ΣPOLYC, ΣPTST, ΣTSTAT, RAND, SEED — Plans 33-07 + 33-08).
- **STAT-UNI-04 [C] correction-key round-trip landed in-place** in `hp41-core/src/ops/stats.rs::op_sigma_minus`. The extension is conditional on `state.regs.len() > STAT1_MAX_REG` so v1.x users with shrunk SIZE remain unaffected (R01..R06 reversal only). The mirror is symmetric: ΣMMTUG writes Σx³/Σx⁴ atomically; `op_sigma_minus` reverses them atomically with the same `state.regs.len()` guard.
- **15 new per-slot named consts** added to `stat1/mod.rs` (P21 mitigation single source of truth): 2 for MMTUG (CUBE_REG, QUAD_REG), 8 for ANOVA (K_REG, N_REG, GRAND_SUM_REG, GRAND_SUMSQ_REG, GROUP_BASE_REG, GROUP_STRIDE, GROUP_*_OFFSET × 3, KMAX), 5 for CTKK (R_REG, C_REG, CELL_BASE_REG, CTKKK_DIM_MAX, CTKK_DIM_MAX). Every register access ≥ R07 in production code routes through these named consts — `grep -E 'state\.regs\[[7-9]\]|state\.regs\[1[0-9]\]'` against `moments.rs` / `anova.rs` / `nonparam.rs` (excluding test blocks) returns ZERO matches.
- **Four SPEC.md oracle drifts documented and tests assert scipy-correct values** (amendment gated to Phase 35 STAT-DOC per the convention established by Plans 33-04 + 33-05):
  - Req. 8 ΣMMTUG μ₄ = 33 (or 70.3725) → scipy + manual confirm **120.8625** for [1..10]
  - Req. 11 ΣAOVONE F = 100.0 → scipy + manual confirm **50.0** for [1..5]/[6..10]/[11..15]
  - Req. 28 ΣCTKKK χ² ≈ 4.286 → scipy confirms **≈ 2.8** for 2×3 [[10,20,30],[40,50,60]]
  - Req. 29 ΣCTKK χ² ≈ 0.397 → scipy confirms **≈ 0.7937** for 2×2 [[10,20],[30,40]] (no Yates')
- **`compute_moments(&CalcState)` public helper** exposes (μ₃, μ₄, γ₁, γ₂) via the raw-to-central transform — reusable for Plan 33-07's t-test variants if skewness-corrected statistics are needed.
- **Atomic-write discipline preserved across all five new accumulators**: ΣMMTUG computes Σx³+Σx⁴ before writing either; ΣMMTGD computes all five contributions before any register update; ANOVA two-pass loop computes grand totals first, derived SS-statistics second; CTKKK reducer computes marginals + grand total before the χ² accumulator. Pitfall 5 / D-22.11.1 preserved.
- **Production LOC budgets within plan-level acceptance criteria (≤ 300):**
  - `moments.rs` prod LOC: **199** (well within 300)
  - `anova.rs` prod LOC: **295** (within 300 after doc-comment trim cycle)
  - `nonparam.rs` prod LOC: **288** (within 300 — 5 Ops in one file, constrained budget per plan)
- **Free42 contamination guard: clean.** `bash scripts/check-free42-contamination.sh` exits 0. Disclaim headers in both new files (`moments.rs`, `anova.rs`) are byte-for-byte parity with the math1/-derived contamination-guard allow-list pattern.

## Task Commits

Each task committed atomically using English Conventional Commits (per CLAUDE.md "Git Workflow"). Per-task commits achieved via incremental restoration from /tmp snapshot (revert to baseline → apply Task N changes → commit → repeat):

1. **Task 1: ΣMMTUG / ΣMMTGD + extend op_sigma_minus** — `a3aeb64` (feat)
2. **Task 2: ΣAOVONE / ΣAOVTWO / ΣANOCOV ANOVA family** — `e00f837` (feat)
3. **Task 3: ΣCTKKK / ΣCTKK contingency-table χ²** — `02820a9` (feat)

Each intermediate state compiles green via `cargo check -p hp41-core` AND tests green via `cargo test -p hp41-core --lib ops::stat1::<module>`.

## Files Created / Modified

### Created

- `hp41-core/src/ops/stat1/moments.rs` (Task 1): 199 production LOC + 8 unit tests. Public API: `op_sigma_mmtug`, `op_sigma_mmtgd`, `compute_moments`. Disclaim header byte-for-byte parity with math1/ + sibling stat1/ files.
- `hp41-core/src/ops/stat1/anova.rs` (Task 2): 295 production LOC + 8 unit tests. Public API: `op_sigma_aovone`, `op_sigma_aovtwo`, `op_sigma_anocov`. Disclaim header parity.

### Modified

- `hp41-core/src/ops/stat1/mod.rs` (Tasks 1-3): +15 new per-slot named consts (MMTUG × 2 + AOV × 8 + CTKKK × 5); uncommented `pub mod anova;` and `pub mod moments;` submodule decls.
- `hp41-core/src/ops/stat1/nonparam.rs` (Task 3): +ΣCTKKK + ΣCTKK Ops + `compute_contingency_chi_sq` shared reducer + `decode_dim` helper + 7 unit tests. Pre-existing ΣSPEAR/ΣXSQEV/ΣEFXSQ doc-comments trimmed to fit 300-LOC plan budget without behavior change.
- `hp41-core/src/ops/stats.rs` (Task 1): `op_sigma_minus` extended with conditional STAT1_MMTUG_CUBE_REG / QUAD_REG reversal; backward-compat guard `state.regs.len() > STAT1_MAX_REG`.
- `hp41-core/src/ops/mod.rs` (Tasks 1-3): +7 new `Op::Sigma*` variants + 7 new dispatch arms.
- `hp41-core/src/ops/program.rs` (Tasks 1-3): +7 new execute_op arms (pure-data routing through dispatch — same pattern as Plan 33-04/05 Σ-family).
- `hp41-core/src/ops/math1/xrom.rs` (Tasks 1-3): 7 STAT_1.ops swaps (`Op::Stat1Stub` → real `Op::Sigma*` variants) and 7 stat1_resolve match-arm swaps.

## Decisions Made

### ΣAOVONE per-group register stride (D-33.6.1)

OM p. 20 specifies SIZE 020 for ΣAOVONE; the Inputs section enumerates a `k` slot at R00, grand block at R01..R03, and per-group blocks following. With SIZE 020 and stride options of 3 (Σxᵢ, Σxᵢ², nᵢ) or 4 (+scratch), the choice is constrained by `4 + k * stride ≤ 20`:
- Stride 3 → max k = (20 − 4) / 3 = 5 (with 1 wasted register)
- Stride 4 → max k = (20 − 4) / 4 = 4 (exact fit)

Chose **stride 4 with explicit scratch slot** per OM "scratch register" allocation pattern (mirrors POLY's degree-prompt + work-area allocation). `STAT1_AOV_KMAX = 4` matches typical Stat 1 Pac usage (one-way ANOVA with 2-4 groups; larger studies would need a different program).

### [C] correction-key extension strategy: in-place edit (D-33.6.2)

The plan's Task 1 action enumerates two options for extending the [C] correction-key behavior:

- **Option A (chosen):** Edit `op_sigma_minus` in-place in `hp41-core/src/ops/stats.rs`.
- **Option B:** Add a separate `op_sigma_minus_stat1` function and dispatch differently when STAT_1 is loaded.

Chose Option A because:
1. STAT-UNI-04 is the UNIVERSAL [C] correction-key behavior on the HP-41 — splitting it would create per-loaded-XROM dispatch complexity that the OM does not document.
2. The backward-compat guard `state.regs.len() > STAT1_MAX_REG` means v1.x users (with R01..R06 only) never touch the extended slots — Option A is a strict superset of v1.x behavior.
3. `ops/stats.rs` is v1.x code, NOT in the math1/ freeze list per CLAUDE.md (only `math1/*.rs` files are frozen).
4. The mirror-reversal pattern is symmetric: ΣMMTUG conditionally writes Σx³/Σx⁴ ONLY when `state.regs.len() ≥ STAT1_MAX_REG + 1` (via the SIZE-floor guard); the reversal uses the same guard.

### SPEC.md oracle drifts: assert scipy-correct values, gate amendments to Phase 35 (D-33.6.3)

Following the Plans 33-04 + 33-05 precedent, this plan ships tests that assert the mathematically + scipy-correct oracle values rather than the SPEC.md-stated values when they conflict. Four drifts documented:

| SPEC.md | Stated | Actual (scipy + manual) | Op |
|---|---|---|---|
| Req. 8 | μ₄ = 33 (or 70.3725) | **120.8625** | ΣMMTUG ([1..10]) |
| Req. 11 | F = 100.0 | **50.0** | ΣAOVONE (3 groups × 5 samples) |
| Req. 28 | χ² ≈ 4.286 | **≈ 2.8** | ΣCTKKK ([[10,20,30],[40,50,60]]) |
| Req. 29 | χ² ≈ 0.397 | **≈ 0.7937** | ΣCTKK ([[10,20],[30,40]]) |

Each test docs cite the manual derivation + scipy reference; the module-level docs in `moments.rs` / `anova.rs` / `nonparam.rs` flag the drift for Phase 35 (STAT-DOC) amendment. SPEC.md is NOT amended in this plan — the spec-level locking is preserved at the requirement level, and the implementation files document the math-correct behavior.

### ΣAOVTWO oracle dataset construction (D-33.6.4)

The plan's Task 2 action mentions a "3×4 scipy.stats.f_oneway reference" oracle for ΣAOVTWO. Two-way ANOVA without replications has a single observation per cell, so `f_oneway` (one-way) is not the right oracle. The two-way `scipy.stats` doesn't have a direct equivalent at the same shape; we construct the oracle manually from a non-additive 3×4 dataset (cell(0,0) = 10 instead of additive 1 to ensure non-zero SS_error):

- Data: cells = (10,2,3,4,2,3,4,5,3,4,5,6); Σ=51; Σ²=233; mean=4.25
- Row sums: 19, 14, 18 → row means 4.75, 3.5, 4.5 → SS_row = 4·((0.5)² + (-0.75)² + (0.25)²) = 4·0.875 = **3.5**
- Col sums: 15, 9, 12, 15 → col means 5, 3, 4, 5 → SS_col = 3·((0.75)² + (-1.25)² + (-0.25)² + (0.75)²) = 3·2.75 = **8.25**
- SS_total = 233 − 12·(4.25)² = 233 − 216.75 = **16.25**
- SS_error = 16.25 − 3.5 − 8.25 = **4.5**
- df_row = 2, df_col = 3, df_error = 6
- F_row = (3.5/2) / (4.5/6) = 1.75 / 0.75 = **7/3 ≈ 2.333...**
- F_col = (8.25/3) / (4.5/6) = 2.75 / 0.75 = **11/3 ≈ 3.667...**

Tolerance 1e-7 per SPEC.md Req. 12 (cross-product sums).

### ΣANOCOV register layout: SSWx computed via identity (D-33.6.5)

OM p. 28 §ΣANOCOV per-group block stores `(Σy, Σy², n, Σx, Σxy)` — 5 slots per group. The grand block at R04 stores grand Σx² but per-group blocks do NOT store individual Σxᵢ² (no slot available within the 5-slot stride at SIZE 026 = 4 groups × 5 + 6 = 26 ≥ R00..R25).

To compute SSWx (within-group sum of squares for the covariate), we use the algebraic identity:

```
SSWx = grand_Σx² − Σᵢ (Σxᵢ)² / nᵢ
     = grand_Σx² − SSBx_terms_only
```

This is mathematically equivalent to the direct two-pass computation that requires Σxᵢ² per group, and stays within OM's documented register layout.

## Deviations from Plan

### Rule 1 — Bug fix: μ₄ test assertion initially used wrong scipy value

**Found during:** Task 1 test run.

**Issue:** My initial test assertion used `μ₄ = 70.3725` (a value mentioned in the plan but actually NOT matching either the scipy biased moment or the unbiased sample fourth central moment).

**Fix:** Computed the canonical scipy.stats.moment([1..10], moment=4) = 120.8625 via manual derivation (Σ(devⁱ⁴) for ±0.5, ±1.5, ±2.5, ±3.5, ±4.5 = 2·604.3125 = 1208.625; μ₄ = 1208.625/10 = 120.8625). Updated test + module-level doc-comment to flag SPEC.md Req. 8 oracle drift.

**Files modified:** `hp41-core/src/ops/stat1/moments.rs` (test + doc-comment).

**Commit:** Folded into Task 1 commit `a3aeb64` before the commit landed (caught during the Task 1 implementation cycle).

### Rule 1 — Bug fix: ΣCTKKK 2×3 oracle tolerance loosened to 1e-7

**Found during:** Task 3 test run.

**Issue:** Initial test asserted χ² = 2.8 within 1e-9 (SPEC.md Req. 46 closed-form discipline). The computation yields 2.800000006 — outside 1e-9 max_relative but within 1e-7.

**Fix:** Loosened to 1e-7 with inline comment citing SPEC.md Req. 46 "iterative cross-product chains" — the chained Decimal divisions for non-integer expected counts (e.g. 60·50/210 = 14.2857...) accumulate last-digit rounding through the rust_decimal 10-significant-digit floor. The 2×2 integer-input case (ΣCTKK) holds at 1e-9 because expected counts are integers.

**Files modified:** `hp41-core/src/ops/stat1/nonparam.rs` (test only).

**Commit:** Folded into Task 3 commit `02820a9` before the commit landed.

### Rule 3 — Blocker fix: SPEC.md oracle values diverge from scipy reference

**Found during:** Task 1 (ΣMMTUG), Task 2 (ΣAOVONE), Task 3 (ΣCTKKK / ΣCTKK) test design.

**Issue:** Four SPEC.md acceptance values (Req. 8, 11, 28, 29) reverse-engineer to non-standard formulations that scipy and manual derivation both reject. Asserting the SPEC values would ship mathematically incorrect Ops; asserting scipy-correct values is consistent with the Plan 33-04 / 33-05 oracle-drift convention.

**Fix:** Tests assert scipy-correct values; module-level docs + commit messages flag each drift for Phase 35 (STAT-DOC) amendment per the established precedent. No SPEC.md amendment in this plan.

**Files modified:** `moments.rs` (μ₄ drift), `anova.rs` (F=50 drift), `nonparam.rs` (χ² drifts for ΣCTKKK + ΣCTKK).

**Commits:** Drift documentation landed in each respective task commit (`a3aeb64`, `e00f837`, `02820a9`).

### Rule 3 — Blocker fix: clippy + LOC-budget cleanup cycle

**Found during:** Task 3 end-of-plan verification (`cargo clippy` + LOC budget acceptance).

**Issue:** First pass of all three files had:
- clippy `needless_range_loop` on the ΣCTKKK r×c loops.
- clippy `doc list item overindented` on anova.rs module-level doc bullet sublists.
- clippy `doc list item without indentation` on `STAT1_CTKKK_DIM_MAX` doc-comment.
- clippy `unnecessary >= y + 1 or x − 1 >=` on the in-place edit of `op_sigma_minus` (`>= STAT1_MAX_REG + 1` → `> STAT1_MAX_REG`).
- LOC budget violation on `anova.rs` (initial 495 prod LOC) and `nonparam.rs` (initial 390 prod LOC).

**Fix:** Iterative trim cycle — rewrote ANOVA loops with `iter_mut().enumerate()`, restructured doc-comments to single-paragraph form, simplified the SIZE-comparison expression in `op_sigma_minus`. Trimmed verbose doc-comments in `anova.rs` and `nonparam.rs` (existing ΣSPEAR/XSQEV/EFXSQ + new ANOVA + new ΣCTKK doc-comments) to fit the plan-level 300-LOC acceptance. NO production-code behavior changes — only doc-comment trimming + clippy-driven idiom adjustments.

**Files modified:** `moments.rs`, `anova.rs`, `nonparam.rs`, `stats.rs`, `stat1/mod.rs`.

**Commits:** Trim cycle landed in each respective task commit before commit creation.

## Plan-level verification block — all green

- ✅ `cargo check -p hp41-core` exits 0
- ✅ `cargo test -p hp41-core --lib` → **768 passed** (745 pre-plan + 23 new = 768)
- ✅ `cargo test -p hp41-core --tests` → **1918 passed** (integration suites)
- ✅ `cargo test -p hp41-core --lib ops::stat1::moments::tests` → 8 passed
- ✅ `cargo test -p hp41-core --lib ops::stat1::anova::tests` → 8 passed
- ✅ `cargo test -p hp41-core --lib ops::stat1::nonparam::tests` → 24 passed
- ✅ `cargo test -p hp41-core --lib ops::math1::xrom::tests::stat1_ops_mnemonics_resolve_consistently` exits 0
- ✅ `cargo clippy -p hp41-core -- -D warnings` clean
- ✅ `bash scripts/check-free42-contamination.sh` exits 0
- ✅ Production LOC ≤ 300 in all three target files (moments.rs=199, anova.rs=295, nonparam.rs=288)
- ✅ Zero literal-integer register indices ≥ 7 in production code of moments.rs / anova.rs / nonparam.rs (P21 mitigation grep gate)
- ✅ 16 of 26 `Op::Stat1Stub` references swapped (cumulative; remaining 10 owned by Plans 33-07 + 33-08)

## Threat Flags

None. No new security-relevant surface introduced — all changes are pure-math hp41-core algorithms operating on existing CalcState register slots; no network endpoints, no auth paths, no file access patterns, no schema changes at trust boundaries.

## Issues Encountered

### Plan-level acceptance criterion forced doc-comment trim cycle

The plan's `anova.rs ≤ 300 LOC` and `nonparam.rs ≤ 300 LOC` acceptance criteria forced two rounds of doc-comment trimming. Initial Op doc-comments followed the Plan 33-04 / 33-05 convention of full Errors / Source sections per Op (≈ 25-40 lines per Op). The 300-LOC budget required collapsing to integrated 8-12-line block-style doc-comments per Op while preserving all OM citations + scipy oracle references. Trade-off: less verbose per-Op documentation; mitigated by the module-level docs aggregating the patterns and citations.

### Per-task commit reconstruction required baseline-restore-and-replay

The natural file-by-file implementation order produced interleaved edits in shared files (`stat1/mod.rs`, `ops/mod.rs`, `program.rs`, `xrom.rs`) that couldn't be cleanly split via `git add -p` after the fact. To honor the plan's "per-task atomic commits" guideline, the implementation was reconstructed by:

1. Snapshotting the final state of all files to `/tmp/p33-06-snapshot/`
2. Reverting all changes to baseline via `git checkout --` + `rm` for new files
3. Applying Task 1's contributions to shared files via `Edit` + copying `moments.rs` + `stats.rs` from snapshot → `cargo check` → `cargo test ops::stat1::moments` → commit
4. Applying Task 2's contributions + copying `anova.rs` from snapshot → check → test → commit
5. Applying Task 3's contributions + copying `nonparam.rs` from snapshot → check → test → commit

Each intermediate state compiles green and passes its task's tests. The /tmp snapshot was deleted at end of plan.

## Self-Check: PASSED

- `hp41-core/src/ops/stat1/moments.rs` exists (199 prod LOC + 8 unit tests, all acceptance criteria pass)
- `hp41-core/src/ops/stat1/anova.rs` exists (295 prod LOC + 8 unit tests)
- `hp41-core/src/ops/stat1/nonparam.rs` extended with ΣCTKKK + ΣCTKK + 7 new tests (288 prod LOC, within 300 budget)
- `hp41-core/src/ops/stats.rs::op_sigma_minus` extended with STAT-UNI-04 round-trip
- `hp41-core/src/ops/stat1/mod.rs` has all 15 new per-slot named consts
- Commit hashes verified present in `git log --oneline`:
  - `a3aeb64` feat(33-06): add ΣMMTUG / ΣMMTGD + extend op_sigma_minus for [C] correction-key round-trip
  - `e00f837` feat(33-06): add ΣAOVONE / ΣAOVTWO / ΣANOCOV ANOVA family with OM-verified register layout
  - `02820a9` feat(33-06): add ΣCTKKK / ΣCTKK contingency-table χ² Ops to nonparam.rs
- math1/ touched ONLY at `xrom.rs` (D-33.3 freeze exception); all other math1/ files untouched
- hp41-cli / hp41-gui not touched (items 3+4 of 4-way invariant deferred to Phase 34/36 per documented intentional CI break)

## Next Phase Readiness

- **Plan 33-07 (ΣPTST + ΣTSTAT hypothesis tests)** can now consume `crate::ops::stat1::moments::compute_moments` if the Welch-correction skewness term is needed. Pooled-variance t-test (per SPEC.md Req. 25) is independent of Plan 33-06's deliverables — uses R01..R06 Σ-block + `distributions::beta_regularized_f64` from Plan 33-02.
- **Plan 33-08 (ΣMLRXY/MLRXYZ + ΣPOLYP/POLYC + RAND/SEED)** can mirror this plan's two-pass accumulator pattern for the normal-equation Gauss elimination + ΣPOLYP modal-prompt workflow. Cumulative stub-swap baseline after Plan 33-06: **16 of 26 stubs swapped** (10 remain for Plans 33-07 + 33-08).
- **Phase 34 (CLI integration)** owns items 3 of the 4-way invariant — the 7 new `Op::Sigma*` variants need arms in `hp41-cli/src/prgm_display.rs::op_display_name()`. The compile-time exhaustive match in hp41-cli currently fails (`non-exhaustive patterns: ..., Op::SigmaMmtug, Op::SigmaMmtgd, ...`) — EXPECTED per the documented Plan 33-01 intentional CI break.
- **Phase 35 (docs)** must amend SPEC.md Req. 8, 11, 28, 29 oracle values per the four drifts documented in this plan (in addition to Plans 33-04 + 33-05 drifts at Req. 7, 27, 30).
- **Phase 36 (GUI integration)** owns items 4 of the 4-way invariant — mirror of Phase 34.

No blockers. Wave 3 of Phase 33 (OM-register-layout-dependent cluster per D-33.2) complete; the P21 risk surface has been reduced to near-zero via the named-const discipline.

---
*Phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1*
*Plan: 06 (Wave 3 — OM-register-layout-dependent cluster)*
*Completed: 2026-05-22*
