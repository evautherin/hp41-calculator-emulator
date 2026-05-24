---
phase: 37-test-hardening-quality-gates
plan: "01"
subsystem: hp41-core/tests
tags:
  - meta-gate
  - lint
  - stat1-pac
  - wave1
dependency_graph:
  requires: []
  provides:
    - stat1-op-test-count-gate
    - lint-stat1-assertions-gate
    - stat1-rand-determinism-lint-exempt
  affects:
    - hp41-core/tests/
    - hp41-core/src/ops/stat1/
tech_stack:
  added: []
  patterns:
    - LINT-EXEMPT annotation discipline (D-32.1 carry-forward to Stat 1 Pac)
    - Two-pass test scanner (external tests/ + inline #[cfg(test)] blocks)
    - Dual-token variant matching (Op::VariantName + op_snake_case fn name)
    - PascalCase-to-snake_case converter for stat1 variant-to-function mapping
key_files:
  created:
    - hp41-core/tests/stat1_op_test_count.rs
    - hp41-core/tests/lint_stat1_assertions.rs
  modified:
    - hp41-core/tests/stat1_rand_determinism.rs
    - hp41-core/tests/stat1_cancellation.rs
    - hp41-core/src/ops/stat1/anova.rs
    - hp41-core/src/ops/stat1/chisqd.rs
    - hp41-core/src/ops/stat1/distributions.rs
    - hp41-core/src/ops/stat1/modal.rs
    - hp41-core/src/ops/stat1/moments.rs
    - hp41-core/src/ops/stat1/nonparam.rs
decisions:
  - "D-37-01-A: Two-pass + dual-token strategy required for stat1_op_test_count — stat1 inline tests use direct fn calls (op_sigma_bstat) not Op:: enum dispatch; math1 analog only needed Pass 1 (all tests external)"
  - "D-37-01-B: meta-gate intentionally fails at Wave 1 installation — 16 variants below 5-test threshold; this IS the designed behavior (surfaces coverage gaps for Wave 2 plans)"
  - "D-37-01-C: 12 LINT-EXEMPT sites across 6 source files for Pitfall 17/14 false positives — all are integer-equality, error-type, or boundary-value tests; lookahead window (3 lines) causes false positives when HpNum appears in setup/teardown lines adjacent to non-HpNum assertions"
metrics:
  duration: "~35 minutes"
  completed: "2026-05-24"
  tasks: 3
  files: 10
---

# Phase 37 Plan 01: Wave 1 Meta-Gate Infrastructure Summary

Wave 1 meta-gate infrastructure for Stat 1 Pac quality enforcement. Three test files (one modified, two new) enforce Pitfall 14/16/17 discipline over the entire stat1 test surface, mirroring the v3.0 Phase 32 pattern with adaptations for Stat 1 Pac's inline test architecture.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Add LINT-EXEMPT annotations to stat1_rand_determinism.rs | a177ebf | hp41-core/tests/stat1_rand_determinism.rs |
| 2 | Create stat1_op_test_count.rs meta-gate (STAT-QUAL-07) | 07da9d5 | hp41-core/tests/stat1_op_test_count.rs |
| 3 | Create lint_stat1_assertions.rs + LINT-EXEMPT annotations (STAT-QUAL-06) | f7cd452 | hp41-core/tests/lint_stat1_assertions.rs + 6 stat1 source files |

## Success Criteria Status

| Criterion | Status | Notes |
|-----------|--------|-------|
| stat1_op_test_count detects 26 STAT_1 variants | PASS | Sanity assertion passes; meta-gate fires on 16 variants below threshold (designed behavior) |
| lint_stat1_assertions Pitfall 14 gate | PASS | no_manual_tolerance_pattern_in_stat1_tests passes |
| lint_stat1_assertions Pitfall 17 gate | PASS | no_decimal_assert_eq_in_stat1_tests passes |
| xrom_shadowing STAT_1 tests (STAT-QUAL-08) | PASS | 6 tests pass; Phase 33 attestation confirmed |
| stat1_rand_determinism LINT-EXEMPT annotations | PASS | 6 annotations; all 3 tests still pass |

## Test Gate Status (end of Wave 1)

```
stat1_rand_determinism:   3 passed ✓
lint_stat1_assertions:    2 passed ✓  (Pitfall 14 + Pitfall 17 gates)
xrom_shadowing:           6 passed ✓  (STAT-QUAL-08 attested)
stat1_op_test_count:      1 FAILED ✓  (meta-gate working as designed — 16 variants below 5-test threshold)
```

The `stat1_op_test_count` failure is **intentional** per STAT-QUAL-07 design: the gate surfaces coverage gaps for Wave 2 plans (37-02..37-05) to address.

## Variants Below 5-Test Threshold (Wave 2 Targets)

16 of 26 STAT_1 variants have fewer than 5 test function mentions:

| Variant | Count | Target Wave 2 Plan |
|---------|-------|--------------------|
| SigmaMmtug | 4 | 37-02 |
| SigmaMmtgd | 2 | 37-02 |
| SigmaAovone | 4 | 37-02 |
| SigmaAovtwo | 2 | 37-02 |
| SigmaAnocov | 2 | 37-02 |
| SigmaLin | 3 | 37-03 |
| SigmaExp | 3 | 37-03 |
| SigmaLogi | 3 | 37-03 |
| SigmaPow | 4 | 37-03 |
| SigmaMlrxy | 3 | 37-03 |
| SigmaMlrxyz | 1 | 37-03 |
| SigmaPolypWorkflow | 2 | 37-03 |
| SigmaPolyc | 2 | 37-03 |
| SigmaCtkk | 2 | 37-04 |
| SigmaNormdWorkflow | 4 | 37-04 |
| SigmaChisqdWorkflow | 2 | 37-04 |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing annotations] LINT-EXEMPT annotations required in 6 stat1 source files**
- **Found during:** Task 3 (lint_stat1_assertions.rs installation)
- **Issue:** The lint correctly flagged 15 assertion sites across 6 source files that were legitimate exceptions needing LINT-EXEMPT annotations. Categories: (a) integer-equality via HpNum::zero() or HpNum::from(Ni32) — exact by construction; (b) error-type HpError::Domain/InvalidOp assertions falsely flagged due to 3-line lookahead hitting HpNum in setup code; (c) distribution primitive boundary tests using tight abs() tolerance on mathematically exact values
- **Fix:** Added 12 LINT-EXEMPT annotations across anova.rs, chisqd.rs, distributions.rs, modal.rs, moments.rs, nonparam.rs + 2 annotations in stat1_cancellation.rs
- **Files modified:** 7 files
- **Commit:** f7cd452

**2. [Rule 2 - Missing functionality] Dual-token matching required in stat1_op_test_count.rs**
- **Found during:** Task 2 (first run of stat1_op_test_count showed 0 mentions for all variants)
- **Issue:** Stat 1 Pac inline tests call internal functions directly (op_sigma_bstat) rather than going through dispatch(Op::SigmaBstat). Math1's external-tests-only approach doesn't apply here. The plan description of "two-pass scan" implied the OpToken scan would find them, but the inline tests use a different naming convention.
- **Fix:** Added PascalCase→snake_case converter (variant_to_fn_name) + dual-token matching in line_mentions_variant_or_fn, checking both Op::VariantName and op_<snake_case>. Added 5 unit tests for the converter covering SigmaBstat, SigmaPolypWorkflow, SigmaNormdWorkflow, Rand, Seed.
- **Files modified:** hp41-core/tests/stat1_op_test_count.rs
- **Commit:** 07da9d5

### Stat1_op_test_count Meta-Gate Failure (Designed Behavior)

The `each_stat1_op_has_at_least_5_tests` test fails with 16 variants below threshold. This is **by design** per the plan: "If any variant is below threshold, that surfaces at Wave 1 and gets addressed by Wave 2 coverage-gap plans." The gate correctly reports all variants and their counts for Wave 2 action.

## Known Stubs

None — no stub patterns found in created/modified files.

## Threat Flags

None — Phase 37 is a pure-test phase. No new trust boundaries, IPC endpoints, or data persistence changes per the plan's threat model.

## Self-Check: PASSED

Files confirmed to exist:
- hp41-core/tests/stat1_op_test_count.rs ✓
- hp41-core/tests/lint_stat1_assertions.rs ✓
- hp41-core/tests/stat1_rand_determinism.rs ✓ (modified)

Commits confirmed:
- a177ebf (Task 1: LINT-EXEMPT annotations)
- 07da9d5 (Task 2: stat1_op_test_count)
- f7cd452 (Task 3: lint_stat1_assertions)
