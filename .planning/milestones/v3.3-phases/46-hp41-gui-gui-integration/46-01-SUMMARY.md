---
phase: 46-hp41-gui-gui-integration
plan: "01"
subsystem: hp41-gui
tags:
  - gui
  - advantage-pac
  - xrom
  - 4-way-invariant
dependency_graph:
  requires:
    - "45-02 (ADRs + docs; 4-way invariant item 3 complete)"
    - "44-01 (CLI prgm_display.rs with 117 Advantage Pac arms)"
    - "43-01 (117 Op::Adv* variants in dispatch + execute_op)"
  provides:
    - "4-way exhaustive-match invariant item 4 complete"
    - "hp41-gui compiles without non-exhaustive patterns warnings"
    - "CATALOG 2 shows XROM 22 (ADV 22A) and XROM 24 (ADV 24B)"
  affects:
    - "hp41-core/src/ops/program.rs (xrom_registry)"
    - "hp41-gui/src-tauri/src/prgm_display.rs (op_display_name)"
tech_stack:
  added: []
  patterns:
    - "SC-4 duplication: byte-for-byte identical op_display_name arms in CLI and GUI"
    - "Exhaustive match without catch-all: compile-time enforcement of 4-way invariant"
key_files:
  created: []
  modified:
    - "hp41-core/src/ops/program.rs"
    - "hp41-gui/src-tauri/src/prgm_display.rs"
decisions:
  - "ADV_MATH_A (bit 3 = 0b0000_1000) and ADV_MATH_B (bit 4 = 0b0001_0000) added to xrom_registry array in program.rs matching CalcState::xrom_modules encoding"
  - "117 Advantage Pac arms added byte-for-byte identical to CLI copy per SC-4 invariant; no catch-all arm to preserve compile-time exhaustive-match enforcement"
metrics:
  duration: "~4 minutes"
  completed: "2026-05-26T15:51:27Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 2
---

# Phase 46 Plan 01: GUI Integration (Advantage Pac) Summary

GUI integration for Advantage Pac: sealed the 4-way exhaustive-match invariant by adding 117 Op::Adv* arms to hp41-gui prgm_display.rs and extending CATALOG 2 xrom_registry with ADV_MATH_A (XROM 22) + ADV_MATH_B (XROM 24).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Extend CATALOG 2 xrom_registry with Advantage Pac modules | a4d8fa0 | hp41-core/src/ops/program.rs |
| 2 | Add 117 Advantage Pac op_display_name arms to GUI prgm_display.rs | 695418c | hp41-gui/src-tauri/src/prgm_display.rs |

## What Was Built

### Task 1 — CATALOG 2 xrom_registry Extension

Extended `hp41-core/src/ops/program.rs` to:
- Import `ADV_MATH_A` and `ADV_MATH_B` from `crate::ops::math1::xrom`
- Append two entries to the `xrom_registry` array:
  - `(&ADV_MATH_A, 0b0000_1000)` — bit 3, Advantage Pac ADV CONV+MTRX (XROM 22)
  - `(&ADV_MATH_B, 0b0001_0000)` — bit 4, Advantage Pac ADV MATH+TVM (XROM 24)
- Updated comment from "Fourth module (Advantage Pac) will extend this array automatically" to "5 XROM modules in registry (Advantage Pac ADV_MATH_A + ADV_MATH_B added)"

The xrom_registry array now has all 5 entries: MATH_1 (bit 0), STAT_1 (bit 1), TIME_MODULE (bit 2), ADV_MATH_A (bit 3), ADV_MATH_B (bit 4). The existing generic loop in CATALOG 2 execution iterates all 5 entries automatically.

### Task 2 — GUI prgm_display.rs Advantage Pac Arms

Added 117 `Op::Adv*` match arms to `hp41-gui/src-tauri/src/prgm_display.rs`:
- 64 arms in the XROM 22 section (ADV CONV + ADV MTRX family), starting with `Op::AdvBinin => "BININ"`
- 53 arms in the XROM 24 section (ADV MATH + ADV TVM family), ending with `Op::AdvTvmStarI => "*I"`
- Arms are byte-for-byte identical to `hp41-cli/src/prgm_display.rs` per the SC-4 duplication invariant
- No `_ =>` catch-all arm; exhaustive match enforces the 4-way invariant at compile time
- Doc comment updated to mention "Advantage Pac" in the coverage list

## Requirements Satisfied

- **ADV-GUI-01**: hp41-gui compiles with zero non-exhaustive patterns warnings — SATISFIED
- **ADV-GUI-03**: CATALOG 2 lists XROM 22 and XROM 24 entries — SATISFIED
- **ADV-GUI-04**: Modal LCD alternation works for Advantage Pac workflows via existing CalcStateView::from_state infrastructure (ModalProgram::Advantage(step).current_prompt() wired at modal.rs line 96) — SATISFIED (no new code needed, verified by existing types.rs priority chain)

## Frozen Invariants Preserved

- **4-way exhaustive-match invariant**: item 4 (GUI prgm_display.rs) is now complete. All 4 sites are sealed: dispatch() + execute_op() (Phase 43), CLI prgm_display.rs (Phase 44), GUI prgm_display.rs (Phase 46 this plan).
- **SC-4 invariant**: No calculator logic added to hp41-gui. The Advantage Pac math remains in hp41-core/src/ops/advantage/. op_display_name is the documented SC-4 exception (display helpers duplicated CLI ↔ GUI by design).
- **Zero new runtime deps**: No new crates added.
- **MSRV 1.88**: Unchanged.

## Verification Results

1. `cd hp41-gui/src-tauri && cargo check` — PASS (compiled cleanly, no warnings)
2. `grep -c "Op::Adv" hp41-gui/src-tauri/src/prgm_display.rs` — 117 (PASS)
3. `cargo test -p hp41-core -- op_catalog` — 1 passed (CATALOG 2 with 5 modules, PASS)
4. `grep -c "ADV_MATH" hp41-core/src/ops/program.rs` — 4 matches (import + comment + 2 registry entries, >= 2, PASS)
5. No `_ =>` catch-all arm in GUI prgm_display.rs — confirmed via grep (PASS)

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — this plan adds display-name strings and CATALOG 2 registry entries. All 117 Op::Adv* variants have behavioral implementations from Phase 43. Modal LCD alternation uses existing infrastructure (no stubs in the code paths touched by this plan).

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. The two files modified contain compile-time constants (xrom_registry) and display-name strings (prgm_display.rs) — no new trust boundaries.

## Self-Check: PASSED

- `hp41-core/src/ops/program.rs` — modified, exists: confirmed
- `hp41-gui/src-tauri/src/prgm_display.rs` — modified, exists: confirmed
- Commit a4d8fa0 (Task 1): confirmed via `git log`
- Commit 695418c (Task 2): confirmed via `git log`
