---
phase: 42-test-hardening-quality-gates
plan: "01"
subsystem: hp41-core/tests
tags:
  - meta-gate
  - lint
  - time-pac
  - xrom
  - wave1
dependency_graph:
  requires:
    - phase: 39-hp41-cli-cli-integration-live-display
      provides: XROM shadowing 3-module extension (D-39.14), function_matrix_parity 4-pool extension
    - phase: 38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core
      provides: Free42 contamination guard extended to time/ tree (18 tokens)
  provides:
    - xrom-op-test-count-unified-gate
    - lint-xrom-assertions-unified-gate
    - time-lint-exempt-annotations
  affects:
    - hp41-core/tests/
    - hp41-core/src/ops/time/
tech-stack:
  added: []
  patterns:
    - Unified three-module XROM meta-gate (math1 + stat1 + time in single file)
    - time_variant_to_fn_name() strips Time module prefix before PascalCase->snake conversion
    - THREE-MODULE collection for lint gate (extends TWO-PASS from Phase 37)
    - LINT-EXEMPT pre-annotation for false positives in time inline tests
    - D-42-01-A precedent: meta-gate intentionally reports Time coverage gaps at Wave 1 (mirrors D-37-01-B)

key-files:
  created:
    - hp41-core/tests/xrom_op_test_count.rs
    - hp41-core/tests/lint_xrom_assertions.rs
  modified:
    - hp41-core/src/ops/time/alarm.rs
    - hp41-core/src/ops/time/alpha_time.rs
    - hp41-core/src/ops/time/clock.rs
    - hp41-core/src/ops/time/date_arith.rs
    - hp41-core/src/ops/time/stopwatch.rs
  deleted:
    - hp41-core/tests/math1_op_test_count.rs
    - hp41-core/tests/stat1_op_test_count.rs
    - hp41-core/tests/lint_math1_assertions.rs
    - hp41-core/tests/lint_stat1_assertions.rs

key-decisions:
  - "D-42-01-A: meta-gate intentionally reports Time coverage gaps at Wave 1 (30/35 variants below 5-test threshold); Wave 2 coverage-gap tests will close the gap — mirrors Phase 37 D-37-01-B precedent"
  - "D-42-01-B: time_variant_to_fn_name() strips leading 'Time' module prefix before PascalCase->snake conversion (op_time not op_time_time) — Time Op variant names carry module prefix unlike Math1/Stat1"
  - "D-42-01-C: 12 LINT-EXEMPT pre-annotations added to src/ops/time/*.rs inline tests for Pitfall 14/17 false positives (4 f64 stopwatch tolerance, 8 integer/exact-decimal assertions) — analogous to Phase 37 stat1_rand_determinism.rs pattern"

patterns-established:
  - "Unified XROM meta-gate: single xrom_op_test_count.rs scans all 3 modules via brace-depth scoped fn resolver scanning"
  - "Time module prefix stripping: time_variant_to_fn_name() strips 'Time' before snake conversion"
  - "LINT-EXEMPT for f64 stopwatch fields: stopwatch_accumulated and stopwatch_split are raw f64 (not HpNum), tolerance comparisons are correct"

requirements-completed:
  - TIME-QUAL-03
  - TIME-QUAL-05
  - TIME-QUAL-08

duration: 12min
completed: 2026-05-25
---

# Phase 42 Plan 01: Wave 1 Meta-Gate Infrastructure Summary

**Unified XROM per-Op test-count meta-gate (106 variants across Math 1 + Stat 1 + Time) and assertion-discipline lint gate replacing 4 per-module files, with LINT-EXEMPT pre-annotations on 12 Time Pac inline test false positives**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-05-25T10:54:23Z
- **Completed:** 2026-05-25T11:07:15Z
- **Tasks:** 3
- **Files modified:** 9 (2 created, 5 LINT-EXEMPT annotated, 4 deleted)

## Accomplishments

- Created unified `xrom_op_test_count.rs` scanning Math 1 (45) + Stat 1 (26) + Time (35) = 106 Op variants via brace-depth scoped resolver scanning and dual-scan strategy (D-42.1, D-42.10)
- Created unified `lint_xrom_assertions.rs` enforcing Pitfall 14/17 discipline across all three XROM module test surfaces (external + inline cfg(test) blocks)
- Deleted 4 old per-module meta-gate files (`math1_op_test_count.rs`, `stat1_op_test_count.rs`, `lint_math1_assertions.rs`, `lint_stat1_assertions.rs`) per D-42.14 atomic replacement
- Pre-annotated 12 LINT-EXEMPT sites in `src/ops/time/*.rs` inline tests (4 f64 stopwatch tolerance patterns, 8 integer/exact-decimal equality patterns)
- Re-verified XROM shadowing (10 tests), function_matrix_parity (14 tests), and Free42 contamination guard all pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Create unified xrom_op_test_count.rs and delete old per-module files** - `96be80f` (test)
2. **Task 2: Create unified lint_xrom_assertions.rs and delete old per-module files** - `97755a0` (test)
3. **Task 3: Re-verify XROM shadowing and Free42 contamination gates** - (no code changes, verification only — subsumed in Task 2 commit context)

## Files Created/Modified

- `hp41-core/tests/xrom_op_test_count.rs` — Unified per-Op test-count meta-gate for all 3 XROM modules
- `hp41-core/tests/lint_xrom_assertions.rs` — Unified assertion-discipline lint for all 3 XROM module test surfaces
- `hp41-core/src/ops/time/stopwatch.rs` — 4 LINT-EXEMPT annotations on f64 tolerance comparisons + 2 on integer equality
- `hp41-core/src/ops/time/date_arith.rs` — 3 LINT-EXEMPT annotations (2 Decimal string equality, 1 multi-line tuple false positive)
- `hp41-core/src/ops/time/alpha_time.rs` — 1 LINT-EXEMPT annotation (integer equality via HpNum::from(42i32))
- `hp41-core/src/ops/time/alarm.rs` — 1 LINT-EXEMPT annotation (HpNum::zero() equality)
- `hp41-core/src/ops/time/clock.rs` — 2 LINT-EXEMPT annotations (integer equality via Decimal::from(N))

## Decisions Made

- **D-42-01-A:** Meta-gate intentionally reports Time coverage gaps at Wave 1 (30/35 Time variants below 5-test threshold); this IS the designed behavior — Wave 2 coverage-gap tests will close the gap. Mirrors Phase 37 D-37-01-B precedent exactly.
- **D-42-01-B:** `time_variant_to_fn_name()` strips leading `Time` module prefix before PascalCase→snake conversion. Time Op variants carry a `Time` module prefix in their names (`TimeTime`, `TimeSetdate`) but the internal functions drop this prefix (`op_time`, `op_setdate`). This differs from Math 1 and Stat 1 which have no module prefix in variant names.
- **D-42-01-C:** 12 LINT-EXEMPT pre-annotations added to `src/ops/time/*.rs` inline tests. 4 are for `f64` stopwatch field comparisons (`stopwatch_accumulated`, `stopwatch_split`) which correctly use tolerance (not HpNum); 8 are for integer/exact-Decimal equality comparisons where `HpNum::from(integer)` or `HpNum::zero()` is exact by construction.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Pre-annotated LINT-EXEMPT for Time Pac inline test violations**
- **Found during:** Task 2 (Create unified lint_xrom_assertions.rs)
- **Issue:** `lint_xrom_assertions.rs` Pitfall 14 gate flagged 4 f64 stopwatch tolerance comparisons; Pitfall 17 gate flagged 8 integer/exact-Decimal assertions in `src/ops/time/*.rs` inline tests. Without LINT-EXEMPT annotations, the lint gate would fail on legitimate code patterns.
- **Fix:** Added `// LINT-EXEMPT: <reason>` annotations adjacent to each flagged assertion, with specific rationale (mirrors Phase 37 stat1_rand_determinism.rs pattern documented in D-37.8)
- **Files modified:** `alarm.rs`, `alpha_time.rs`, `clock.rs`, `date_arith.rs`, `stopwatch.rs`
- **Verification:** `cargo test -p hp41-core --test lint_xrom_assertions` exits 0 (2 tests pass)
- **Committed in:** `97755a0` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 2 - pre-annotation of LINT-EXEMPT sites)
**Impact on plan:** Required for correctness of the lint gate. No scope creep — analogous to Phase 37 pre-annotation step (D-37.8).

## Issues Encountered

The `xrom_op_test_count.rs` meta-gate reports 30/35 Time variants below the 5-test threshold at Wave 1. This is the designed behavior per D-42-01-A (mirrors Phase 37 D-37-01-B). The coverage-gap tests for Time Pac will be added in Wave 2 (Plan 42-02), at which point the gate will pass fully.

Test counts for passing Time variants: `TimeRunsw`, `TimeStopsw`, `TimeXyzalm`, `TimeRclalm`, `TimeAlmnow` meet the threshold via inline tests. All Math 1 (45/45) and Stat 1 (26/26) variants continue to pass.

## Known Stubs

None — this plan creates meta-gate test infrastructure only; no user-facing stubs.

## Self-Check

### Files exist:
- `hp41-core/tests/xrom_op_test_count.rs`: FOUND
- `hp41-core/tests/lint_xrom_assertions.rs`: FOUND
- `hp41-core/tests/math1_op_test_count.rs`: correctly DELETED
- `hp41-core/tests/stat1_op_test_count.rs`: correctly DELETED
- `hp41-core/tests/lint_math1_assertions.rs`: correctly DELETED
- `hp41-core/tests/lint_stat1_assertions.rs`: correctly DELETED

### Commits exist:
- `96be80f`: FOUND (Task 1)
- `97755a0`: FOUND (Task 2)

### Quality gates:
- `cargo test -p hp41-core --test lint_xrom_assertions`: PASS (2 tests)
- `cargo test -p hp41-core --test xrom_shadowing`: PASS (10 tests)
- `cargo test -p hp41-cli --test function_matrix_parity`: PASS (14 tests)
- `bash scripts/check-free42-contamination.sh`: PASS (exits 0)
- `cargo test -p hp41-core --test xrom_op_test_count`: 15 unit tests pass; `each_xrom_op_has_at_least_5_tests` intentionally reports Time coverage gaps (D-42-01-A)

## Self-Check: PASSED

## Next Phase Readiness

- Wave 1 meta-gate infrastructure complete and committed
- Wave 2 (Plan 42-02) will measure per-file coverage for `src/ops/time/*.rs` and add targeted gap-closure tests for the 30 Time variants below threshold
- The unified meta-gate scans ALL external `tests/time_*.rs` files added in Wave 2 automatically (no changes needed to `xrom_op_test_count.rs`)

---
*Phase: 42-test-hardening-quality-gates*
*Completed: 2026-05-25*
