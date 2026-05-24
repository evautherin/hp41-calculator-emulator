---
phase: 36-hp41-gui-gui-integration
plan: "01"
subsystem: hp41-gui + hp41-core
tags: [gui, prgm-display, xrom, stat1, catalog, 4-way-invariant, D-36.1, D-36.3]
dependency_graph:
  requires:
    - 34-02-SUMMARY.md  # CLI prgm_display.rs 26 arms (source-of-truth for GUI copy)
    - 33-01-SUMMARY.md  # STAT_1 XromModule const + bit-1 arm in xrom_resolve
  provides:
    - hp41-gui/src-tauri/src/prgm_display.rs: 26 Stat 1 Op display-name arms (4-way invariant item 4)
    - hp41-core/src/ops/program.rs: CATALOG 2 bit-1 block for STAT_1
    - hp41-core/tests/op_catalog_xrom.rs: catalog_2_lists_stat1_when_bit1_set test
    - hp41-gui/src-tauri/tests/prgm_display_math1_arms.rs: every_stat1_op_appears_in_prgm_display test
    - .planning/REQUIREMENTS.md: STAT-GUI-05 reassigned Phase 36 → Phase 37
  affects:
    - CATALOG 2 output (CLI + GUI both gain STAT 1B enumeration)
    - GUI program listing display for all Stat 1 Pac programs
    - Phase 37 requirements scope (STAT-GUI-05 added to its requirement list)
tech_stack:
  added: []
  patterns:
    - D-36.1 — parallel if blocks (bit-0 + bit-1) in op_catalog instead of premature abstraction
    - D-25.6 — CLI ↔ GUI parity: 26 arm strings copied verbatim from CLI prgm_display.rs
    - D-36.3 — STAT-GUI-05 reassignment with bounded-iter rationale documented
key_files:
  created: []
  modified:
    - hp41-gui/src-tauri/src/prgm_display.rs
    - hp41-core/src/ops/program.rs
    - hp41-core/tests/op_catalog_xrom.rs
    - hp41-gui/src-tauri/tests/prgm_display_math1_arms.rs
    - .planning/REQUIREMENTS.md
decisions:
  - D-36.1: Mirror bit-0 block for bit-1 in op_catalog — no premature abstraction; parallel if blocks
  - D-36.3: STAT-GUI-05 reassigned to Phase 37 with bounded-iter rationale (Acklam O(1) + gser/gcf ITER_CAP=50)
metrics:
  duration: "7m 24s"
  completed_date: "2026-05-24"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 5
---

# Phase 36 Plan 01: STAT-GUI-05 Reassignment + 4-Way Invariant Seal + CATALOG 2 Bit-1 Summary

**One-liner:** Sealed 4-way exhaustive-match invariant item 4 with 26 Stat 1 `op_display_name` arms in GUI `prgm_display.rs`; extended `op_catalog` with parallel STAT_1 bit-1 block; reassigned STAT-GUI-05 to Phase 37 with bounded-iter rationale.

## What Was Built

### Task 1: STAT-GUI-05 Reassignment (D-36.3 bookkeeping)

- `.planning/REQUIREMENTS.md`: STAT-GUI-05 requirement text updated with bounded-iter rationale (Acklam closed-form / gser+gcf ITER_CAP=50 — microsecond timescale, cancellation not needed per 36-CONTEXT D-36.2)
- `.planning/REQUIREMENTS.md` traceability row: Phase 36 → Phase 37
- `.planning/REQUIREMENTS.md` phase counts: Phase 36 (5→4), Phase 37 (11→12)

The ROADMAP.md already reflected the correct state from planning (Phase 36 success criterion #4 already stated the reassignment; Phase 37 requirements already listed STAT-GUI-05).

### Task 2: 4-Way Invariant Item 4 + CATALOG 2 + Tests

**Part A — GUI prgm_display.rs (STAT-GUI-01):**
- 26 new `Op::Sigma*/Rand/Seed` match arms added to `hp41-gui/src-tauri/src/prgm_display.rs`
- Strings copied verbatim from `hp41-cli/src/prgm_display.rs` per D-25.6 parity invariant
- File-header comment updated from stale count to description-based (avoids future staleness)
- `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` now exits 0 with zero `non-exhaustive patterns` warnings
- 4-way exhaustive-match invariant item 4 (GUI) sealed for the first time since Phase 33 ship

**Part B — hp41-core op_catalog bit-1 block (STAT-GUI-02):**
- `hp41-core/src/ops/program.rs`: `STAT_1` added to xrom import
- CATALOG 2 arm restructured: parallel `if state.xrom_modules & 0b0000_0010 != 0` block (D-36.1 — mirrors bit-0 block, no abstraction)
- NO XROM fallback now fires only when BOTH bits are clear (`else if state.xrom_modules & 0b0000_0001 == 0`) — defensive path post-`migrate_after_load`
- CLI and GUI both inherit STAT_1 enumeration from this shared core code

**Part C — Tests:**
- `hp41-core/tests/op_catalog_xrom.rs`: 
  - New `catalog_2_lists_stat1_when_bit1_set` test: asserts both "MATH 1A" and "STAT 1B" module headers appear when `xrom_modules = 0b0000_0011`; spot-checks "POLY" (Math 1) and "ΣBSTAT" (Stat 1); validates exact line count
  - Fixed `catalog_2_with_math1_loaded_lists_header_and_functions`: changed from `CalcState::default()` (which is now `0b0000_0011` per D-33.7) to explicit `xrom_modules: 0b0000_0001` (Math 1 only isolation)
- `hp41-gui/src-tauri/tests/prgm_display_math1_arms.rs`:
  - New `STAT1_VARIANT_IDS` const array (26 identifiers)
  - New `every_stat1_op_appears_in_prgm_display` file-text-scan test (mirrors `every_math1_op_appears_in_prgm_display`)

## Verification Results

| Check | Result |
|-------|--------|
| `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` — zero non-exhaustive patterns | PASS |
| `cargo test -p hp41-core --test op_catalog_xrom` — 4 tests pass | PASS |
| `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml --test prgm_display_math1_arms` — 2 tests pass | PASS |
| SC-4 grep: `grep -rn "fn op_*(add\|sub\|...")` in hp41-gui/src-tauri/src/ | PASS (empty) |
| REQUIREMENTS.md STAT-GUI-05 shows Phase 37 | PASS |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed existing catalog test broken by v3.1 default xrom_modules**

- **Found during:** Task 2, running `cargo test -p hp41-core catalog_2`
- **Issue:** `catalog_2_with_math1_loaded_lists_header_and_functions` used `CalcState::default()` which has `xrom_modules = 0b0000_0011` (v3.1 default per D-33.7 `default_xrom_modules`). The new bit-1 block now adds STAT_1 entries, causing the test's line-count assertion to fail (expected 52 MATH_1 ops, got 52+27 = 79 lines).
- **Fix:** Changed test to use explicit `xrom_modules: 0b0000_0001` (Math 1 only) and updated comment from "(default v3.0 config)" to reflect the isolation intent. This keeps the test focused on verifying the bit-0 Math 1 path in isolation.
- **Files modified:** `hp41-core/tests/op_catalog_xrom.rs`
- **Commit:** e91872b

**2. [Rule 1 - Bug] Fixed Y^X spot-check in new catalog test**

- **Found during:** Task 2, first run of `catalog_2_lists_stat1_when_bit1_set` test
- **Issue:** The plan specified spot-checking "Y^X" as a Math 1 entry, but Y^X (`Op::YPow`) is NOT in `MATH_1.ops`. `YPow` is a built-in v2.2 Op (not a XROM entry).
- **Fix:** Changed spot-check from "Y^X" to "POLY" (which IS in `MATH_1.ops` as the POLY workflow entry).
- **Files modified:** `hp41-core/tests/op_catalog_xrom.rs`
- **Commit:** e91872b

## Known Stubs

None — all 26 Op display-name arms return real HP-41 mnemonic strings. CATALOG 2 enumerates both XROM modules when both bits are set. No placeholder text or empty returns.

## Threat Flags

None — this plan adds pure string lookup table arms and a CATALOG enumeration display path. No new network endpoints, auth paths, file access patterns, or schema changes introduced.

## Self-Check: PASSED

- `hp41-gui/src-tauri/src/prgm_display.rs` modified: FOUND
- `hp41-core/src/ops/program.rs` modified: FOUND
- `hp41-core/tests/op_catalog_xrom.rs` modified: FOUND
- `hp41-gui/src-tauri/tests/prgm_display_math1_arms.rs` modified: FOUND
- `.planning/REQUIREMENTS.md` modified: FOUND
- Task 1 commit `14d6e3a`: FOUND
- Task 2 commit `e91872b`: FOUND
