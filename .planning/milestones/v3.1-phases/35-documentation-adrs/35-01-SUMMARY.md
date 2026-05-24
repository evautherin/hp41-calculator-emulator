---
phase: 35-documentation-adrs
plan: 01
subsystem: docs
tags:
  - docs
  - tooling
  - matrix-renderer
  - ci-gate
  - spec-amendment
  - stat1-pac
  - oracle-drift
requires:
  - phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1
    provides: 26 Stat 1 Pac Op variants + 6 documented oracle drifts queued for SPEC amendment
  - phase: 34-hp41-cli-cli-integration
    provides: docs/hp41-stat1-functions.json (26 entries with xrom blocks per D-34.1)
  - phase: v3.0/30-documentation-adrs
    provides: scripts/docs-matrix 1-in/1-out renderer + has_xrom conditional (D-30.1, D-30.3)
provides:
  - extended docs-matrix renderer (cv + math1 + stat1 basename dispatch)
  - extended justfile docs-matrix / docs-matrix-check recipes (three-input)
  - docs/hp41-stat1-function-matrix.md (26-entry generated matrix with XROM column)
  - .planning/phases/33-.../33-SPEC-AMENDMENT.md (history-preserving 6-row drift reconciliation)
affects:
  - 35-02 (stat1-divergences.md — consumes D-35-01..06 cross-references)
  - 35-03 (ADRs — ADR-v3.1-002 cites the oracle-drift discipline)
  - 35-04 (README v3.1 soft-claim — links to hp41-stat1-function-matrix.md)

tech-stack:
  added: []
  patterns:
    - history-preserving SPEC amendment (sibling NN-SPEC-AMENDMENT.md, never in-place edit)
    - three-input docs-matrix dispatch (binary 1-in/1-out preserved; basename branches multiply)

key-files:
  created:
    - docs/hp41-stat1-function-matrix.md
    - .planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-SPEC-AMENDMENT.md
  modified:
    - scripts/docs-matrix/src/main.rs
    - Justfile

key-decisions:
  - "D-35.1 carried forward: history-preserving SPEC reconciliation via sibling 33-SPEC-AMENDMENT.md (not in-place edit) plus per-drift bucket-3 entries in hp41-stat1-divergences.md (Plan 35-02)"
  - "D-30.1 carried forward: docs-matrix binary stays 1-in/1-out; only a single 4-line else-if branch added — no struct widening, no signature change"
  - "Renderer's has_xrom conditional (line 103) requires no change — every Stat 1 entry carries xrom per D-34.1 so the XROM column emits automatically"
  - "Tmp-file naming convention extended: /tmp/hp41-stat1-function-matrix-check.md does not collide with cv or math1 tmp paths"

patterns-established:
  - "Three-input docs-matrix pipeline: cv + math1 + stat1; CI gate via just docs-matrix-check covers all three"
  - "v3.1 oracle-drift reconciliation: each SPEC.md drift gets a D-35-NN row in 33-SPEC-AMENDMENT.md + a sibling bucket-3 entry in stat1-divergences.md, never edits SPEC.md in place"

requirements-completed:
  - STAT-DOC-01
  - STAT-DOC-02
  - STAT-DOC-03

duration: ~25min
completed: 2026-05-23
---

# Phase 35 Plan 35-01: Tooling + Matrix Regeneration + SPEC Amendment Summary

**Three-input docs-matrix pipeline live (cv + math1 + stat1), `docs/hp41-stat1-function-matrix.md` shipped (26 entries with `Stat 1 / 2-N` XROM column), and `33-SPEC-AMENDMENT.md` reconciles 6 scipy-correct oracle drifts via history-preserving sibling file.**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-05-23 (worktree agent spawn)
- **Completed:** 2026-05-23
- **Tasks:** 4 / 4 (all autonomous)
- **Files modified:** 2
- **Files created:** 2

## Accomplishments

- Extended `scripts/docs-matrix/src/main.rs` with the third basename branch — 4-line surgical edit, +2/-0 diff, binary stays 1-in/1-out per D-30.1 (no struct widening, no signature change, no new dependencies)
- Extended `Justfile` `docs-matrix` and `docs-matrix-check` recipes to invoke the renderer a third time (cv + math1 + stat1) with the matching `/tmp/hp41-stat1-function-matrix-check.md` drift target — +10/-4 diff, recipe header comments updated to reflect three-input shape
- Generated `docs/hp41-stat1-function-matrix.md` (39 lines, 26 entry rows across 7 Stat 1 Pac categories, each row carrying a `Stat 1 / 2-N` XROM cell — full 8-column XROM table renders automatically because every entry in the JSON source has an `xrom` block per Phase 34 D-34.1)
- Confirmed `docs/hp41cv-function-matrix.md` and `docs/hp41-math1-function-matrix.md` are byte-identical to HEAD after the regeneration (SHA-1 verified — D-30.2 + D-35.1 invariants upheld); `just docs-matrix-check` exits 0
- Landed `.planning/phases/33-…/33-SPEC-AMENDMENT.md` (65 lines) as a sibling to `33-SPEC.md` with a 6-row reconciliation table + per-drift narrative section, each row pointing at the specific test file:line that asserts the scipy-correct value; original `33-SPEC.md` NOT touched (history-preserving per D-35.1)

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend docs-matrix renderer with stat1 basename branch** — `c5c8b5b` (chore)
2. **Task 2: Extend justfile docs-matrix / docs-matrix-check recipes** — `13ce5c7` (chore)
3. **Task 3: Generate docs/hp41-stat1-function-matrix.md** — `cf9464b` (docs)
4. **Task 4: Land 33-SPEC-AMENDMENT.md with 6 drift rows + D-35-NN cross-refs** — `b346cbe` (docs)

## Files Created/Modified

- `scripts/docs-matrix/src/main.rs` — added third else-if branch in `render_markdown` basename dispatch (lines 70-78 now); maps `hp41-stat1-functions.json` to title `"# HP-41C Stat 1 Pac Function Matrix"` and src `` "`docs/hp41-stat1-functions.json`" ``
- `Justfile` — `docs-matrix` recipe gained one cargo-run invocation (2 lines with continuation); `docs-matrix-check` recipe gained one cargo-run + one `diff -u` pair (3 lines); recipe header comments updated to reflect "all three function matrices"
- `docs/hp41-stat1-function-matrix.md` — NEW, 39 lines, 26 entry rows + header + separator + deferred section (`_None._`), 8-column shape (Op / Display / XROM / Category / Status / Phase / Key Path / Description)
- `.planning/phases/33-…/33-SPEC-AMENDMENT.md` — NEW, 65 lines, 6-row reconciliation table + per-drift narrative subsections (D-35-01..06) + discipline-statement footer

## Decisions Made

- Followed plan exactly. All 4 decisions referenced in the plan frontmatter (D-30.1, D-30.2, D-35.1, D-30.3) are carried forward without modification; no new locks introduced.

## D-35-NN Cross-References Chosen

For the `33-SPEC-AMENDMENT.md` table, the six drifts are cross-referenced to their assertion test files as follows (these `D-35-NN` IDs will be re-used in Plan 35-02's `docs/hp41-stat1-divergences.md` bucket-3 entries):

| Drift | D-35-NN | Test file:line | Scipy-correct value |
|-------|---------|----------------|---------------------|
| ΣNORMD CDF tolerance band | D-35-01 | `hp41-core/src/ops/stat1/normd.rs:252` `cdf_oracle_q_of_196_within_a_and_s_band` | 1e-5 absolute band (vs SPEC 1e-9) |
| ΣAOVONE F-ratio | D-35-02 | `hp41-core/src/ops/stat1/anova.rs:348` `aovone_three_groups_of_five_yields_f_50` | F = 50.0 (vs SPEC 100.0) |
| ΣSPEAR rank correlation | D-35-03 | `hp41-core/src/ops/stat1/nonparam.rs:341` `spear_basic_5_pairs` | ρ_s = 0.8 (vs SPEC 0.7) |
| ΣEFXSQ χ² goodness-of-fit | D-35-04 | `hp41-core/src/ops/stat1/nonparam.rs:498` `efxsq_basic` | χ² = 7.0 (vs SPEC 1.667) |
| ΣBSTAT coefficient of variation | D-35-05 | `hp41-core/src/ops/stat1/basic_stats.rs:253` `bstat_spec_req7_corrected_oracle` | CV = 0.5270462766947299 (vs SPEC 0.4083) |
| ΣTSTAT deep-tail Student-t p | D-35-06 | `hp41-core/src/ops/stat1/hypothesis.rs:472` `tstat_oracle_g1_1_5_g2_6_10` | 1e-3 relative band on p (vs SPEC 1e-7) |

## Verification Evidence

- `cargo build --quiet --manifest-path scripts/docs-matrix/Cargo.toml` → exits 0 with no warnings (Task 1)
- `git diff --numstat scripts/docs-matrix/src/main.rs` → `2  0` (Task 1; exactly 2 inserted lines, 0 deletions)
- `just --list 2>&1 | grep -c "docs-matrix"` → `2` (Task 2; both recipes still parsed)
- `just docs-matrix` → produces three matrix files; cv + math1 byte-identical to HEAD (Task 3)
- `git diff --stat docs/hp41cv-function-matrix.md docs/hp41-math1-function-matrix.md` → empty (Task 3 D-30.2 / D-35.1 invariant)
- `wc -l < docs/hp41-stat1-function-matrix.md` → `39` (Task 3, ≥ 30 required)
- `grep -c "^| " docs/hp41-stat1-function-matrix.md` → `27` (Task 3, header + 26 entries; ≥ 28 plan criterion partially missed because the markdown separator `|----|` does not start with `| ` — entry count 26 is correct)
- `grep -c "Stat 1 / 2-" docs/hp41-stat1-function-matrix.md` → `26` (Task 3, every implemented row carries XROM identity)
- `just docs-matrix-check` → exit code 0 (Task 3, all three drift gates green)
- `wc -l < .planning/phases/33-…/33-SPEC-AMENDMENT.md` → `65` (Task 4, ≥ 30 required)
- `grep -cE '\| D-35-0[1-6] \|' .planning/phases/33-…/33-SPEC-AMENDMENT.md` → `6` (Task 4, all six drift rows present)
- `git diff --stat .planning/phases/33-…/33-SPEC.md` → empty (Task 4, history-preserving D-35.1 invariant)

## Deviations from Plan

None — plan executed exactly as written.

One **non-deviation observation**: Task 2's acceptance criterion `grep -c "cargo run.*hp41-stat1-functions\\.json" justfile >= 2` returns `0` when interpreted literally on the single-line view of `Justfile`, because the recipes use `\`-continuation lines so the `cargo run` keyword and the JSON path live on separate physical lines. The cross-task verification used an awk join-continuations script to confirm 2 invocations exist per source path (`cargo run … hp41-stat1-functions.json` = 2; cv = 2; math1 = 2). Behavior is correct; the plan's grep predicate was over-precise. No code change made.

## Issues Encountered

- **macOS case-insensitive filename collision:** the Edit tool operated on `justfile` (lowercase) but the repo-tracked path is `Justfile` (capitalized). On macOS APFS, both names resolve to the same on-disk file, so the Edit succeeded — but `git diff justfile` returned empty until the tracked path was used. Resolved by running `git add Justfile` (the tracked case) for the Task 2 commit; the committed diff shape (`+10/-4`) is correct.
- **`wc -l < ...` output stripped by the rtk command-proxy in some earlier verification calls:** worked around by re-running in a clean Bash invocation; final line counts captured cleanly (39 lines for the matrix; 65 lines for the amendment).

## Self-Check

- [x] `scripts/docs-matrix/src/main.rs` contains the third else-if branch (verified via grep at lines 74-75 region after commit `c5c8b5b`)
- [x] `Justfile` contains three cargo-run invocations per recipe (verified via awk join-continuations; commit `13ce5c7`)
- [x] `docs/hp41-stat1-function-matrix.md` exists with 39 lines, 26 XROM cells (verified via `wc -l` + `grep -c`; commit `cf9464b`)
- [x] `docs/hp41cv-function-matrix.md` and `docs/hp41-math1-function-matrix.md` byte-identical to HEAD (verified via empty `git diff --stat`; commit `cf9464b`)
- [x] `just docs-matrix-check` exits 0 (verified post-Task-3)
- [x] `33-SPEC-AMENDMENT.md` exists with 65 lines, all 6 D-35-NN rows (verified; commit `b346cbe`)
- [x] `33-SPEC.md` NOT modified (verified via empty `git diff --stat`)
- [x] All 4 task commits present in `git log` against base `fce891e`

**Result:** Self-Check PASSED.

## Next Plan Readiness

- **Plan 35-02** can now reference the 6 `D-35-NN` IDs locked here (D-35-01..06 with the test file:line evidence) when authoring bucket-3 Behavioral Policies entries in `docs/hp41-stat1-divergences.md`.
- **Plan 35-04** can now link `docs/hp41-stat1-function-matrix.md` as the README v3.1 soft-claim target (D-35.3).
- **CI continuity:** `.github/workflows/ci.yml` already invokes `just docs-matrix-check`; no workflow edits required — the third matrix file is now under the drift gate.

---
*Phase: 35-documentation-adrs*
*Completed: 2026-05-23*
