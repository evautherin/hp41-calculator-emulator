---
phase: 44-hp41-cli-cli-integration
plan: 02
subsystem: hp41-cli + hp41-core (test suite)
tags: [advantage-pac, xrom, function-matrix-parity, shadowing, modal-routing, test-hardening]
dependency_graph:
  requires: ["44-01 (fifth JSON pool + help_entries_adv())"]
  provides: ["per-module ADV_A/ADV_B inventory constants + parity tests", "8 xrom_shadowing tests for ADV_MATH_A/B", "phase44_help_data_adv.rs (13 smoke tests)", "phase44_modal_flow.rs (16 modal routing tests)"]
  affects: ["hp41-cli CI test gate (function_matrix_parity, xrom_shadowing, key_coverage, phase44_help_data_adv, phase44_modal_flow)"]
tech_stack:
  added: []
  patterns: ["per-module inventory constants (ADV_A/ADV_B split)", "bit-4 isolation for ADV_MATH_B resolver test", "carrier-enum modal routing verification"]
key_files:
  created: ["hp41-cli/tests/phase44_help_data_adv.rs", "hp41-cli/tests/phase44_modal_flow.rs"]
  modified: ["hp41-cli/tests/function_matrix_parity.rs", "hp41-core/tests/xrom_shadowing.rs"]
decisions:
  - "ADV_MATH_B disjointness test excludes MATH_1 — intentional hardware-faithful ASCII alias overlap (E^Z, LNZ, LOGZ, Z^N, Z^1/N, Z^W, |Z|, SINZ, COSZ, TANZ, A^Z, CINV) per HP Advantage Pac OM 00041-90482"
  - "adv_b_ops_resolve_via_xrom_resolve uses 0b0001_0000 (bit-4 isolation) rather than 0b0001_1111 — when all modules loaded, MATH_1 (bit-0) fires first and wins shared aliases before ADV_MATH_B arm is reached"
  - "phase44_help_data_adv.rs test 6 checks module_id in {22, 24} NOT a single value — dual-module architecture requires disjoint per-module check"
  - "phase44_help_data_adv.rs test 8 verifies function_id density independently per module (XROM 22: 1..=63, XROM 24: 1..=51)"
metrics:
  duration: "12 minutes"
  completed: "2026-05-26"
  tasks_completed: 2
  files_modified: 4
---

# Phase 44 Plan 02: CI Test Gate Extensions for Advantage Pac Summary

Per-module ADV_A/ADV_B inventory constants + parity tests, 8 new xrom_shadowing tests, and two new test files (phase44_help_data_adv.rs, phase44_modal_flow.rs) extending the CI test gate to cover the Advantage Pac's dual-XROM structure end-to-end.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Extend function_matrix_parity.rs + xrom_shadowing.rs | 8b61a2e | hp41-cli/tests/function_matrix_parity.rs, hp41-core/tests/xrom_shadowing.rs |
| 2 | Create phase44_help_data_adv.rs + phase44_modal_flow.rs | 53ce923 | hp41-cli/tests/phase44_help_data_adv.rs, hp41-cli/tests/phase44_modal_flow.rs |

## Success Criteria Verification

- [x] function_matrix_parity: partition guard accepts module_ids 22+24; ADV_A inventory==63, ADV_B inventory==51; forward+reverse parity per module pass
- [x] xrom_shadowing: 8 new tests pass; ADV_MATH_A disjoint from all prior modules + builtins; ADV_MATH_B disjoint from STAT_1, TIME, ADV_MATH_A + builtins (MATH_1 overlap is intentional hardware behavior); resolver round-trips clean
- [x] phase44_help_data_adv: 13 tests pass; 114 entries; dual-module function_ids dense independently; 5-pool >= 350; overlay sections with "=== Adv " prefix
- [x] phase44_modal_flow: all 16 AdvantageStep variants route correctly via ModalProgram::Advantage carrier enum; alpha-label steps return true; TVM prompts correct; parameterized prompts contain coordinates
- [x] key_coverage: probes >= 270 entries (already done in Plan 01); Advantage sub-loop probes >= 100 entries (already done in Plan 01)
- [x] `cargo test --package hp41-cli` exits 0 (421 tests pass, 23 suites)
- [x] `cargo test --package hp41-core --test xrom_shadowing` exits 0 (18 tests pass)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ADV_MATH_B disjointness test incorrectly checked MATH_1 overlap**
- **Found during:** Task 1 (running `cargo test --package hp41-core --test xrom_shadowing`)
- **Issue:** `adv_b_ops_disjoint_from_prior_modules_and_adv_a` failed: `ADV_MATH_B mnemonic "E^Z" also appears in MATH_1.ops`. 12 ADV_MATH_B mnemonics (E^Z, LNZ, LOGZ, Z^N, Z^1/N, Z^W, |Z|, SINZ, COSZ, TANZ, A^Z, CINV) are intentional ASCII alias overlaps with MATH_1 per the hardware Advantage Pac design — when both modules loaded, MATH_1 wins, routing to the standard complex-number ops.
- **Fix:** Changed the disjointness test to check only against STAT_1, TIME_MODULE, and ADV_MATH_A (not MATH_1). Added documentation comment explaining the intentional overlap.
- **Files modified:** hp41-core/tests/xrom_shadowing.rs
- **Commit:** 8b61a2e

**2. [Rule 1 - Bug] ADV_MATH_B resolver test used wrong bitfield (0b0001_1111 instead of 0b0001_0000)**
- **Found during:** Task 1 (same test run)
- **Issue:** `adv_b_ops_resolve_via_xrom_resolve` failed for shared ASCII aliases: `"E^Z" must resolve to AdvExpZ via xrom_resolve(name, 0b0001_1111) — left: Some(ExpZ) right: Some(AdvExpZ)`. With all 5 modules loaded, MATH_1 (bit-0) fires first and returns `Some(ExpZ)` before the ADV_MATH_B arm is reached.
- **Fix:** Changed bitfield from `0b0001_1111` to `0b0001_0000` (bit-4 isolation only) so only ADV_MATH_B is active, letting `adv_b_resolve` handle all 51 ops without MATH_1 interference.
- **Files modified:** hp41-core/tests/xrom_shadowing.rs
- **Commit:** 8b61a2e

Both bugs reflect the same root cause: the hardware Advantage Pac intentionally reuses MATH_1 ASCII aliases for XROM 24's complex-number operations, with MATH_1 winning when both are loaded. This is hardware-faithful behavior; the test expectations needed to match the architecture.

## Threat Flags

None. All changes are test files — no production code surface added, no new network endpoints, auth paths, or schema changes.

## Known Stubs

None.

## Self-Check: PASSED

- FOUND: hp41-cli/tests/phase44_help_data_adv.rs
- FOUND: hp41-cli/tests/phase44_modal_flow.rs
- FOUND: hp41-cli/tests/function_matrix_parity.rs (modified)
- FOUND: hp41-core/tests/xrom_shadowing.rs (modified)
- FOUND commit 8b61a2e: test(44-02): extend parity + shadowing tests for ADV_MATH_A/B modules
- FOUND commit 53ce923: test(44-02): create phase44_help_data_adv + phase44_modal_flow tests
- cargo test --package hp41-cli: 421 passed
- cargo test --package hp41-core --test xrom_shadowing: 18 passed
