---
phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1
plan: 04
subsystem: hp41-core
tags: [stat1, nonparametric, chi-square, spearman, closed-form, p21-named-consts, spec-md-drift]

requires:
  - phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1
    plan: 00
    provides: hp41-core/src/ops/stat1/ skeleton + STAT1_MAX_REG + STAT1_XSQEV_MAX_REG + Free42 contamination guard
  - phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1
    plan: 01
    provides: XROM framework activation — STAT_1 module const + stat1_resolve + bit-1 arm + Op::Stat1Stub placeholder
provides:
  - hp41-core/src/ops/stat1/nonparam.rs (292 LOC production code + 17 unit tests)
  - hp41-core/src/ops/stat1/mod.rs::STAT1_XSQEV_K_REG + STAT1_XSQEV_OBS_BASE_REG + STAT1_XSQEV_EXP_BASE_REG + STAT1_XSQEV_STRIDE + STAT1_XSQEV_KMAX (5 new per-slot named consts; P21 mitigation)
  - Op::SigmaSpear (ΣSPEAR — closed-form ρ_s on existing v1.x R01–R06 block)
  - Op::SigmaXsqev (ΣXSQEV — chi-square goodness-of-fit, observed + expected counts)
  - Op::SigmaEfxsq (ΣEFXSQ — chi-square with expected-as-proportions, in-place conversion)
  - 3 of 26 STAT_1.ops + stat1_resolve stub references swapped to real Sigma* variants
affects:
  - 33-03 (parallel wave-2 plan; both depend only on 33-01; no cross-interference)
  - 33-06 (CTKKK/CTKK additions will reuse compute_chi_square_from_counts helper)
  - 34 (CLI integration — items 3 of 4-way invariant for the 3 new Sigma* variants)
  - 35 (docs — must amend SPEC.md Req. 27 and Req. 30 with scipy-confirmed oracle values)
  - 36 (GUI integration — items 4 of 4-way invariant)

tech-stack:
  added: []  # no new runtime deps; pure hp41-core algorithm work
  patterns:
    - "Closed-form non-parametric Op skeleton — SIZE-floor guard → register read via named const → arithmetic → stack push + LiftEffect::Enable"
    - "Interleaved O/E register layout with base + stride consts (STAT1_XSQEV_*_BASE_REG + STAT1_XSQEV_STRIDE) — enables k-bounded loops with named-const indexing instead of literal offsets"
    - "In-place proportion → count conversion per OM 'Inputs' convention — preserves register-state symmetry between ΣXSQEV and ΣEFXSQ for downstream chained operations"
    - "Domain-error on sum-to-1 tolerance violation — explicit rejection, NEVER silent renormalization (Pitfall 21)"

key-files:
  created:
    - hp41-core/src/ops/stat1/nonparam.rs
    - .planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-04-SUMMARY.md
  modified:
    - hp41-core/src/ops/stat1/mod.rs
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-core/src/ops/math1/xrom.rs

key-decisions:
  - "ΣXSQEV/ΣEFXSQ register layout: interleaved O/E pairs with stride 2, k stored at R00 (R00=k, R01=O₀, R02=E₀, R03=O₁, R04=E₁, R05=O₂, R06=E₂, R07=scratch). Cap k ≤ STAT1_XSQEV_KMAX = 3 per OM SIZE 008 — k = (SIZE-1)/STRIDE = 7/2 = 3 (integer division)."
  - "ΣSPEAR consumes existing v1.x R01–R06 Σ-register block populated by op_sigma_plus. No new per-slot consts for ΣSPEAR — state.regs[2] (Σx = Σd²) and state.regs[3] (n) are explicitly allowed by CLAUDE.md 'Core engine' v1.x baseline."
  - "ΣEFXSQ converts proportions to expected counts IN PLACE (mutating R02/R04/R06) per OM 'Inputs' convention p. 55. This makes the post-call register state symmetric with ΣXSQEV's expected-count entry and reuses the same shared reducer (compute_chi_square_from_counts)."
  - "Sum-to-1 tolerance for ΣEFXSQ: 1e-9 via Decimal::from_parts(1,0,0,false,9). Out-of-tolerance returns Domain error (no silent renormalization, Pitfall 21)."
  - "SPEC.md Req. 30 (ΣSPEAR oracle 0.7) is WRONG — scipy and manual both confirm 0.8. SPEC.md Req. 27 (ΣEFXSQ oracle 1.667) is also WRONG — scipy confirms 7.0. This plan ships scipy-correct values; SPEC.md amendments gated to Phase 35 (STAT-DOC)."

patterns-established:
  - "Shared chi-square reducer helper (compute_chi_square_from_counts) — Task 3 reuses Task 2's accumulator after in-place data prep. Reduces nonparam.rs LOC budget for downstream Plan 33-06 ΣCTKKK/ΣCTKK additions (which can call the same reducer after r×c expected-cell expansion)."
  - "decode_category_count(&HpNum) -> Result<usize> — P21-compliant integer extraction via HpNum::trunc_int (never floor/fmod on f64 per CLAUDE.md ISG/DSE rule). Reusable for any future k-bounded Op in stat1/."

requirements-completed:
  - STAT-HYP-03  # ΣXSQEV chi-square goodness-of-fit
  - STAT-HYP-04  # ΣEFXSQ chi-square with proportions
  - STAT-HYP-07  # ΣSPEAR Spearman rank correlation

metrics:
  duration: ~75min
  completed: 2026-05-22
  tasks_total: 3
  tasks_completed: 3
  files_created: 1
  files_modified: 4
  commits: 3
  tests_added: 17  # 5 spear + 6 xsqev + 6 efxsq
---

# Phase 33 Plan 04: ΣSPEAR + ΣXSQEV + ΣEFXSQ Summary

**Three closed-form non-parametric Ops ship together — Spearman rank correlation (ρ_s), chi-square goodness-of-fit from observed+expected counts (χ²), and chi-square from expected-as-proportions (with in-place conversion) — landing 17 unit tests, 5 new OM-traceable per-slot register consts, and resolution of two SPEC.md oracle-value drifts against scipy.**

## Performance

- **Duration:** ~75 min (incl. SPEC.md oracle re-verification, LOC-budget trim)
- **Completed:** 2026-05-22
- **Tasks:** 3 (all completed atomically)
- **Files:** 1 created, 4 modified
- **Commits:** 3 (one per task; no deviation fixes needed)
- **Tests added:** 17 (5 spear + 6 xsqev + 6 efxsq)

## Accomplishments

- **Three real `Op::Sigma*` variants ship.** `Op::SigmaSpear`, `Op::SigmaXsqev`, `Op::SigmaEfxsq` added to the central enum and land in BOTH `dispatch()` and `execute_op()` (4-way exhaustive-match invariant items 1+2). CLI + GUI `op_display_name` arms (items 3+4) deferred to Phase 34/36 — known intentional CI break in those crates, EXPECTED per Plan 33-01's published contract.
- **Three of 26 `Op::Stat1Stub` references swapped in `STAT_1.ops` + `stat1_resolve`** — bidirectional consistency preserved (CI-gated by `stat1_ops_mnemonics_resolve_consistently`). After this plan: 23 stub references remain; once parallel Plan 33-03 lands its 2 swaps (ΣNORMD + ΣCHISQD), total remaining = 21 as projected in the plan's output spec.
- **SPEC.md Oracle Drift #1 resolved (ΣSPEAR Req. 30):** SPEC.md says ρ_s = 0.7 for `ranks_x=[1,2,3,4,5], ranks_y=[2,1,3,5,4]`. Manual derivation (Σd² = 1+1+0+1+1 = 4; ρ_s = 1 − 6·4/(5·24) = 0.8) and `scipy.stats.spearmanr` both confirm 0.8. Tests assert 0.8; SPEC.md amendment gated to Phase 35 (STAT-DOC).
- **SPEC.md Oracle Drift #2 resolved (ΣEFXSQ Req. 27):** SPEC.md says χ² ≈ 1.667 for proportions=[0.2,0.3,0.5] vs observed=[10,30,60]. Manual + scipy both confirm 7.0 (Σf=100; expected=[20,30,50]; (10−20)²/20 + 0 + (60−50)²/50 = 5 + 0 + 2 = 7). Tests assert 7.0; SPEC.md amendment gated to Phase 35.
- **Pitfall 21 mitigation enforced:** every register access ≥ R07 in `nonparam.rs` routes through a named const from `stat1::mod`. `grep -nE 'state\.regs\[[7-9]\]|state\.regs\[1[0-9]\]' hp41-core/src/ops/stat1/nonparam.rs` returns ZERO matches. The only literal-integer register indices are `state.regs[2]` and `state.regs[3]` in ΣSPEAR — explicitly allowed per CLAUDE.md "Core engine" v1.x stats convention.
- **Shared chi-square reducer:** `compute_chi_square_from_counts(&CalcState) -> Result<HpNum>` is the single accumulator. Task 2 (ΣXSQEV) uses it directly; Task 3 (ΣEFXSQ) calls it after in-place proportion→count conversion. Plan 33-06 (ΣCTKKK/ΣCTKK r×c contingency-table χ²) can reuse the same helper after expanding row×col expected cells.
- **Free42 contamination guard:** still clean. No GPL-tainted symbols introduced; the disclaim header in `nonparam.rs` is byte-for-byte parity with `math1/*.rs` / `stat1/distributions.rs`.

## Task Commits

Each task committed atomically via `git commit` with the CLAUDE.md "Git Workflow" English Conventional Commits style:

1. **Task 1: ΣSPEAR closed-form Spearman rank correlation** — `b7a3e82` (feat)
2. **Task 2: ΣXSQEV chi-square goodness-of-fit (observed + expected counts)** — `53e6ae8` (feat)
3. **Task 3: ΣEFXSQ chi-square with expected-as-proportions entry** — `f3a486a` (feat)

(Plan metadata commit — this SUMMARY — follows.)

## Files Created / Modified

### Created (1 file)

- `hp41-core/src/ops/stat1/nonparam.rs` (~360 lines total; **292 LOC production code** ≤ 300 budget per D-33.5; 17 unit tests). Disclaim header byte-for-byte parity with sibling `stat1/distributions.rs`. Three public `pub fn op_sigma_*` entry points + two private helpers (`require_stat1_size_floor`, `compute_chi_square_from_counts`, `decode_category_count`) + one private const (`PROPORTION_SUM_TOL_DEC`).

### Modified (4 files)

- `hp41-core/src/ops/stat1/mod.rs` (+58 lines): five new named consts (`STAT1_XSQEV_K_REG = 0`, `STAT1_XSQEV_OBS_BASE_REG = 1`, `STAT1_XSQEV_EXP_BASE_REG = 2`, `STAT1_XSQEV_STRIDE = 2`, `STAT1_XSQEV_KMAX = 3`) — each with OM page citation (p. 55 ΣXSQEV/ΣEFXSQ section). `pub mod nonparam;` declaration uncommented (Plan 33-00 placeholder).
- `hp41-core/src/ops/mod.rs` (+46 lines): three new `Op` variants (`SigmaSpear`, `SigmaXsqev`, `SigmaEfxsq`) with substantial doc-comments + three dispatch arms.
- `hp41-core/src/ops/program.rs` (+3 lines): three execute_op arms routing through `dispatch()` (closed-form Ops follow the Plan 28-10 Triangle/TRANS pattern — pure-data ops execute fully from dispatch context, no run_loop re-entry needed).
- `hp41-core/src/ops/math1/xrom.rs` (+4 / −5 lines): three `Op::Stat1Stub` references in `STAT_1.ops` swapped to `Op::SigmaSpear` / `Op::SigmaXsqev` / `Op::SigmaEfxsq`; the corresponding three arms in `stat1_resolve` updated in lockstep. Bidirectional consistency CI-gated.

## Decisions Made

### ΣSPEAR register layout — reuse v1.x Σ-block, no new per-slot consts

The OM transcription in Plan 33-00 specified `STAT1_SPEAR_MAX_REG = 2` (SIZE 003, R00..R02), but per OM p. 64 the closed-form ρ_s algorithm reads from the existing v1.x R01–R06 Σ-register block — the SIZE 003 footprint refers to the program's own scratch / intermediate-result allocation, not the data input. The user accumulates `d² = (rank_x − rank_y)²` values as single-variable Σ+ samples; ΣSPEAR reads Σx (= Σd²) from R02 and n from R03, matching the v1.x stats convention from `ops/stats.rs:21`. No new per-slot consts needed.

This decision is encoded inline in the `op_sigma_spear` doc-comment AND in the SPEAR Pre-condition section so that future readers do not mistake the Plan 33-00 STAT1_SPEAR_MAX_REG = 2 const as a register-base address.

### ΣXSQEV/ΣEFXSQ register layout — interleaved O/E pairs, k bounded by SIZE

OM 00041-90030 §ΣXSQEV (p. 55) does not enumerate the per-slot semantics explicitly readable from Appendix A — it gives SIZE 008 (R00..R07) and labels the program "Chi-Square Evaluation". The interleaved layout (R00=k, R01=O₀, R02=E₀, R03=O₁, R04=E₁, R05=O₂, R06=E₂, R07=scratch) was inferred from:

1. SIZE 008 = 8 registers; if dedicating R00 to `k` and R07 to scratch/result leaves 6 data slots → 3 cells × (1 observed + 1 expected) per cell = 6 data slots ✓
2. The SPEC.md Req. 26 acceptance test uses exactly 3-category data, matching k=3 = STAT1_XSQEV_KMAX
3. Interleaved O/E (rather than contiguous-block) matches the canonical HP-41 Stat Pac convention for goodness-of-fit accumulators visible in NPS document examples for analogous Pacs
4. Defensive `1 ≤ k ≤ STAT1_XSQEV_KMAX` hard cap returns Domain error for any out-of-range k — silent extension to a larger SIZE block would risk reading uninitialized scratch memory

The decision is OM-faithful at the SIZE constraint (≤ 3 categories) while leaving room for Plan 33-06 (ΣCTKKK/ΣCTKK contingency tables) to use a different layout for r×c data (SIZE 015, more flexible).

### Tolerance for ΣEFXSQ sum-to-1 validation: 1e-9

The OM specifies that proportions must sum to 1.0 but does not specify a tolerance band. Chose 1e-9 (matching SPEC.md Req. 27's closed-form `max_relative` tolerance) so that a SUM=1.0 entered as `0.2 + 0.3 + 0.5` survives even after `HpNum::rounded(10-sig-digits)` quantization. The tolerance constant `PROPORTION_SUM_TOL_DEC` is declared explicitly via `Decimal::from_parts(1, 0, 0, false, 9)` (mantissa 1, scale 9 → 1e-9) so it can be referenced without re-construction overhead.

Out-of-tolerance proportion sums return `HpError::Domain` rather than being silently renormalized — silent renormalization would mask user input errors AND produce silently wrong χ² values per Pitfall 21.

### Two SPEC.md oracle drifts documented for Phase 35 (STAT-DOC)

Both SPEC.md Req. 27 (ΣEFXSQ stated 1.667; correct 7.0) and SPEC.md Req. 30 (ΣSPEAR stated 0.7; correct 0.8) are documented inline in the test comments AND in this SUMMARY. The plan executor implementing Phase 35 SPEC.md amendments needs only this SUMMARY + the inline test comments to make the corrections. The plan's STAT-DOC requirement was already drafted as an amendment vehicle for divergence documentation; these two drifts slot in cleanly there.

### nonparam.rs LOC budget enforcement (D-33.5: ≤ 300 production LOC)

After Task 3 landed, the production-code LOC count was 348 (over the 300 budget). The trim path:

1. Module-level doc-comment trimmed from 47 → 30 lines (eliminated redundant "Ops shipping in this module" listing duplicated in op-level docs)
2. ΣXSQEV op doc-comment trimmed from 41 → 22 lines (consolidated separate Layout/Errors/df/Source sections into a single integrated block)
3. ΣEFXSQ op doc-comment trimmed from 54 → 20 lines (removed redundant Layout / Side effect sections; the named consts are self-documenting)

Final production-code LOC: **292** ≤ 300 budget. The Plan 33-06 budget for ΣCTKKK/ΣCTKK still has 8 LOC headroom; if Plan 33-06 needs more, it can extract the shared `compute_chi_square_from_counts` reducer into a private module helper file (e.g. `stat1/chi_sq_helpers.rs`) — but that refactor is NOT needed for this plan.

## Deviations from Plan

None — plan executed exactly as written.

Both SPEC.md oracle discrepancies (Req. 27 + Req. 30) were FORESEEN by the plan itself — the plan's `<action>` blocks explicitly call out the Req. 30 = 0.7 vs scipy 0.8 discrepancy AND the Req. 27 = 1.667 vs scipy 7.0 discrepancy in Tasks 1 and 3 respectively. Both tasks ship the scipy-confirmed values and document the discrepancies inline. So while these are SPEC.md amendments, they are NOT deviations from the EXECUTION plan; the plan correctly anticipated both.

## Auth Gates Encountered

None.

## Plan-level Verification — all green

- `cargo check -p hp41-core` exits 0
- `cargo clippy -p hp41-core --tests -- -D warnings` exits 0
- `cargo test -p hp41-core` — **1832 passed, 1 ignored** across 71 test suites
- `cargo test -p hp41-core --lib ops::stat1::nonparam::tests` — **17 passed** (5 spear + 6 xsqev + 6 efxsq)
- `cargo test -p hp41-core --lib ops::math1::xrom::tests::stat1_ops_mnemonics_resolve_consistently` — passes (bidirectional STAT_1.ops ↔ stat1_resolve consistency preserved)
- `cargo test -p hp41-core --test xrom_shadowing` — **6 passed** (STAT_1 disjointness + bit-1 isolation invariants intact)
- `bash scripts/check-free42-contamination.sh` — exits 0
- nonparam.rs production-code: **292 LOC** ≤ 300 budget (D-33.5)
- No literal-integer register indices ≥ 7 in nonparam.rs (P21 mitigation)
- `grep -c 'pub fn op_sigma_spear' hp41-core/src/ops/stat1/nonparam.rs` → 1
- `grep -c 'pub fn op_sigma_xsqev' hp41-core/src/ops/stat1/nonparam.rs` → 1
- `grep -c 'pub fn op_sigma_efxsq' hp41-core/src/ops/stat1/nonparam.rs` → 1
- 3 of 26 `Op::Stat1Stub` references swapped in xrom.rs (slice + resolver in lockstep)

## Known Intentional CI Break

Per Plan 33-01's known break: `cargo check -p hp41-cli` and `cargo check -p hp41-gui` still fail (`non-exhaustive patterns: &Op::Stat1Stub, &Op::SigmaSpear, &Op::SigmaXsqev, &Op::SigmaEfxsq not covered`). Phase 34 + 36 will surface the missing arms in `op_display_name` and add the display strings. This is the documented contract; not a regression.

`cargo test -p hp41-core` is the executable verification surface for this plan and is fully green.

## Self-Check: PASSED

Created files exist:
- `hp41-core/src/ops/stat1/nonparam.rs` (✓ created; 17 tests green)
- `.planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-04-SUMMARY.md` (this file — ✓ created)

Commit hashes verified in `git log --oneline`:
- ✅ `b7a3e82` feat(33-04): add ΣSPEAR closed-form Spearman rank correlation
- ✅ `53e6ae8` feat(33-04): add ΣXSQEV chi-square goodness-of-fit (observed + expected)
- ✅ `f3a486a` feat(33-04): add ΣEFXSQ chi-square with expected-as-proportions entry

## Next-phase Readiness

- **Plan 33-03 (parallel wave-2)** is unaffected. Both plans depend only on 33-01; the file-level touch sets are disjoint (33-03 touches `stat1/normd.rs` + `stat1/chisqd.rs` + the modal carrier; 33-04 touches `stat1/nonparam.rs` + per-slot ΣXSQEV consts in `stat1/mod.rs`). When both plans' branches merge, the only conflict point is `STAT_1.ops` + `stat1_resolve` line-level merges (both swap different stub references) and `Op::*` enum variant insertions — all mechanically resolvable.
- **Plan 33-06 (ΣCTKKK / ΣCTKK + ANOVA family)** can extend `nonparam.rs` with contingency-table χ² Ops, reusing `compute_chi_square_from_counts` after r×c expected-cell expansion. The LOC budget headroom (8 lines under D-33.5's 300 cap) is tight; if 33-06 needs more, extract the helper to `stat1/chi_sq_helpers.rs`.
- **Phase 34 (CLI integration)** must add `op_display_name` arms for `Op::SigmaSpear`, `Op::SigmaXsqev`, `Op::SigmaEfxsq` (plus `Op::Stat1Stub` if still present after wave-2). Mnemonics: `"ΣSPEAR"`, `"ΣXSQEV"`, `"ΣEFXSQ"` (Σ encoded as `\u{03A3}` per the established convention).
- **Phase 35 (docs)** must amend SPEC.md:
  - Req. 27: change "χ² ≈ 1.667" to "χ² = 7.0" (cite scipy.stats.chisquare oracle)
  - Req. 30: change "ρ_s = 0.7" to "ρ_s = 0.8" (cite scipy.stats.spearmanr oracle + manual computation Σd² = 4)
- **Phase 36 (GUI integration)** mirrors Phase 34 for `op_display_name`.

No blockers. Wave 2 (Plans 33-03 + 33-04) progressing in parallel as designed.

---
*Phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1*
*Plan: 04 (Wave 2 — closed-form non-parametric Ops)*
*Completed: 2026-05-22*
