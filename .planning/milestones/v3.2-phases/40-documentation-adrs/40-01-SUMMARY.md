---
phase: 40-documentation-adrs
plan: 01
subsystem: docs
tags: [docs-matrix, time-pac, xrom, divergences, ci]

# Dependency graph
requires:
  - phase: 38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core
    provides: Time Pac Op implementations (all 35 ops, D-38.x decisions)
  - phase: 39-hp41-cli-cli-integration-live-display
    provides: CLI integration confirming Time Pac ops in scope
provides:
  - docs/hp41-time-function-matrix.md — generated Time Pac function matrix (35 entries, XROM 26)
  - docs/hp41-time-divergences.md — Time Pac divergences catalog (3-bucket, 6 D-40-NN entries)
  - scripts/docs-matrix pipeline extended for 4th JSON source (D-40.1)
  - justfile docs-matrix and docs-matrix-check recipes extended to 4 invocations (D-40.2)
affects:
  - 40-02-PLAN (ADRs for v3.2 decisions)
  - 40-03-PLAN (narrative docs, architecture-history, README hard-claim)
  - phase 41 (GUI integration — may reference divergences for CORRECT/SETAF behavior)
  - phase 42 (test hardening — divergences catalog is oracle for behavioral policy tests)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "docs-matrix pipeline extended once per new XROM module (D-30.1 carry-forward)"
    - "divergences catalog 3-bucket structure with D-40-NN phase-origin numbering (per D-35.4)"
    - "five-field entry shape (OM citation / Our behavior / OM behavior / Rationale / See)"
    - "justfile 4-invocation pattern for docs-matrix and docs-matrix-check"

key-files:
  created:
    - docs/hp41-time-function-matrix.md
    - docs/hp41-time-divergences.md
  modified:
    - scripts/docs-matrix/src/main.rs
    - justfile

key-decisions:
  - "docs-matrix binary signature stays 1-in/1-out per D-30.1; only basename dispatch branch added (D-40.1)"
  - "4th justfile invocation appended after stat1 in both docs-matrix and docs-matrix-check (D-40.2)"
  - "SW interactive stopwatch mode documented as emulator extension (D-40-06) not OM feature"
  - "CORRECT/SETAF accuracy factor documented as no-op behavioral policy (D-40-01) per D-38.1"
  - "Interrupting control alarms documented as deferred (D-40-04) per D-38.4 — data model forward-compat"
  - "Stopwatch freeze-on-save documented as behavioral policy (D-40-03) per D-38.6 — Instant not serializable"

patterns-established:
  - "D-40-NN numbering convention for Time Pac divergences (phase-origin, parallel to D-35-NN for Stat 1)"
  - "docs-matrix-check extended alongside docs-matrix (4 drift checks = 4 generation invocations)"

requirements-completed:
  - TIME-DOC-01
  - TIME-DOC-02

# Metrics
duration: 10min
completed: 2026-05-25
---

# Phase 40 Plan 01: Documentation & ADRs Summary

**Time Pac docs-matrix pipeline extended (4th invocation) and divergences catalog authored with 6 D-40-NN entries covering host-clock backing, stopwatch freeze-on-save, interrupting alarm deferral, accuracy-factor no-op, centisecond resolution, and SW emulator extension**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-05-25T06:24:00Z
- **Completed:** 2026-05-25T06:35:38Z
- **Tasks:** 2 of 2
- **Files modified:** 4 (scripts/docs-matrix/src/main.rs, justfile, docs/hp41-time-function-matrix.md, docs/hp41-time-divergences.md)

## Accomplishments

- Extended docs-matrix pipeline to generate the fourth function matrix (Time Pac, 35 entries, XROM 26)
- Verified all four matrices pass `just docs-matrix-check` drift check with zero diff
- Authored Time Pac divergences catalog (`docs/hp41-time-divergences.md`) with 3-bucket structure and 6 D-40-NN numbered entries covering all known Phase 38/39 implementation decisions
- Existing cv/math1/stat1 matrices confirmed bit-for-bit unchanged after regeneration (D-30.2 invariant preserved)

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend docs-matrix pipeline for Time Pac (4th invocation)** - `d7e4c19` (feat)
2. **Task 2: Author Time Pac divergences catalog** - `758d948` (docs)

**Plan metadata:** committed in SUMMARY commit (docs: complete plan)

## Files Created/Modified

- `scripts/docs-matrix/src/main.rs` — Added `hp41-time-functions.json` branch in `render_markdown()` before else fallback (D-40.1)
- `justfile` — Updated `docs-matrix` recipe (3→4 invocations + comment), `docs-matrix-check` recipe (3→4 drift checks) (D-40.2)
- `docs/hp41-time-function-matrix.md` — Generated (35 entries, XROM column shows `Time / 26-N`, 7 categories: Alarm / Alpha / Clock / Date Arithmetic / Display / Format / Stopwatch)
- `docs/hp41-time-divergences.md` — Time Pac divergences catalog: bucket 1 (0 OM divergences), bucket 2 (1 emulator extension: D-40-06 SW keyboard mode), bucket 3 (5 behavioral policies: D-40-01..D-40-05)

## Decisions Made

- D-40.1: Binary signature stays 1-in/1-out; only a new `else if` basename branch added in `render_markdown()` — no new CLI arguments, no new crate structure changes.
- D-40.2: 4th `cargo run` invocation appended after stat1 in both `docs-matrix` and `docs-matrix-check`; comment updated from "three" to "four" function matrices.
- D-40-06 (SW) classified as emulator extension (bucket 2) rather than behavioral policy — it adds functionality not present in the OM, not a policy divergence from OM-specified behavior.
- D-40-04 (interrupting alarms) documented as behavioral policy (bucket 3) with forward-compatible data model note — `interrupting: bool` field preserved in `AlarmEntry` for future implementation.

## Deviations from Plan

None — plan executed exactly as written. Both tasks completed with all acceptance criteria verified:
- `scripts/docs-matrix/src/main.rs` contains `hp41-time-functions.json` in basename dispatch branch
- `justfile` `docs-matrix` recipe has 4 `cargo run` invocations (cv, math1, stat1, time)
- `justfile` `docs-matrix-check` recipe has 4 `cargo run` + 4 `diff -u` blocks
- `docs/hp41-time-function-matrix.md` exists and contains `# HP-41C Time Pac Function Matrix`
- `just docs-matrix-check` exits 0 (all four matrices match)
- `docs/hp41-time-divergences.md` contains 6 D-40-NN entries with all 5 fields per entry
- Every OM citation references HP 00041-90035 (1982) or carries `N/A -- emulator policy/extension`
- Every See field cross-references at least one D-38.x decision ID and source file path

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required. Documentation-only plan.

## Next Phase Readiness

- TIME-DOC-01 satisfied: `docs/hp41-time-function-matrix.md` generated, `just docs-matrix-check` CI gate covers all four matrices
- TIME-DOC-02 satisfied: `docs/hp41-time-divergences.md` authored with 3-bucket structure, 6 D-40-NN entries
- Ready for Plan 40-02 (ADRs for v3.2 architectural decisions)
- The divergences catalog's D-40-NN cross-references provide anchor IDs for any ADR authoring in Plan 40-02

## Self-Check: PASSED

- `docs/hp41-time-function-matrix.md` exists: confirmed (created in Task 1 commit d7e4c19)
- `docs/hp41-time-divergences.md` exists: confirmed (created in Task 2 commit 758d948)
- `scripts/docs-matrix/src/main.rs` has `hp41-time-functions.json` branch: confirmed
- `justfile` has 4 invocations in both recipes: confirmed
- `just docs-matrix-check` exits 0: confirmed (all four matrices pass drift check)
- Commits d7e4c19 and 758d948 exist in git log: confirmed

---
*Phase: 40-documentation-adrs*
*Completed: 2026-05-25*
