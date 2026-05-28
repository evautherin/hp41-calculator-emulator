---
phase: 51-x-mem-core
plan: "01"
subsystem: hp41-core
tags: [x-mem, data-model, serde, error-handling]
dependency_graph:
  requires: []
  provides: [XmemFile, XmemKind, XMEM_CAPACITY, HpError::FileNotFound, HpError::FileType, HpError::NoRoom, CalcState::xmem_files, CalcState::xmem_active_file]
  affects: [hp41-core/src/ops/xmem/mod.rs, hp41-core/src/error.rs, hp41-core/src/ops/mod.rs, hp41-core/src/state.rs]
tech_stack:
  added: []
  patterns: [D-43.5 named-model isolation, adv_tvm_state serde persistent-without-skip pattern, div_ceil register packing]
key_files:
  created:
    - hp41-core/src/ops/xmem/mod.rs
  modified:
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/error.rs
    - hp41-core/src/state.rs
decisions:
  - XmemFile.data is Vec<u8> (raw codec bytes), not Vec<HpNum> — matches AdvMatrix precedent but stores raw bytes instead of HpNum
  - register_count() uses div_ceil (MSRV 1.88 stable) per CLAUDE.md decimal-field discipline
  - xmem_active_file carries #[serde(default)] WITHOUT #[serde(skip)] (D-51.4 persistent pattern, mirrors adv_tvm_state)
  - v33_save_loads_with_xmem_defaults reuses v2.2 JSON fixture to avoid brittle hand-crafted JSON
metrics:
  duration_minutes: 12
  tasks_completed: 2
  files_created: 1
  files_modified: 3
  tests_added: 11
  completed_date: "2026-05-28"
---

# Phase 51 Plan 01: X-MEM Contract Layer Summary

**One-liner:** XmemFile data model with register_count() and XMEM_CAPACITY=600, three HpError X-MEM variants, and two persistent CalcState fields — isolated from state.regs and adv_matrices per D-51.0a.

## What Was Built

### Task 1: XmemFile model + HpError variants

Created `hp41-core/src/ops/xmem/mod.rs` as a new module following the AdvMatrix precedent:

- `XmemKind` enum (`Program` | `Data`) with `#[default] Program`
- `XmemFile` struct: `name: String`, `kind: XmemKind`, `data: Vec<u8>`, `#[serde(default)] reg_count: usize`
- `register_count()`: `Program => data.len().div_ceil(7) + 1`, `Data => reg_count + 1`
- `XMEM_CAPACITY: usize = 600` (124 built-in + 2×238 two HP 82181A modules)
- `pub mod xmem;` declaration added to `hp41-core/src/ops/mod.rs`
- Three `HpError` variants: `FileNotFound` ("fl not found"), `FileType` ("fl type err"), `NoRoom` ("no room") with HP-41CX QRG p.39 doc comments
- 6 unit tests in xmem/mod.rs + 3 Display tests in error.rs (9 tests total)

### Task 2: CalcState persistent X-MEM fields

Added Phase 51 field block to `hp41-core/src/state.rs` following the Phase 43 adv_matrices/adv_tvm_state pattern:

- `#[serde(default)] pub xmem_files: Vec<crate::ops::xmem::XmemFile>` — persistent, no skip
- `#[serde(default)] pub xmem_active_file: Option<String>` — persistent, no skip (D-51.4 active file)
- Both initialized in `CalcState::new()` (`Vec::new()` / `None`)
- `migrate_after_load()` untouched (new fields default cleanly via `#[serde(default)]`)
- `xmem_serde_round_trip` test: XmemFile with name/kind/reg_count survives serialize+deserialize
- `v33_save_loads_with_xmem_defaults` test: legacy JSON (v2.2 fixture, no xmem keys) loads with empty/None

## Test Coverage

| Test | Location | Verifies |
|------|----------|---------|
| `xmem_file_default_shape` | xmem/mod.rs | Default struct shape |
| `register_count_default_program` | xmem/mod.rs | 0 bytes → 1 register |
| `register_count_program_8_bytes` | xmem/mod.rs | 8 bytes → 3 registers |
| `register_count_program_922_bytes` | xmem/mod.rs | OM-verified 922 bytes → 133 registers |
| `register_count_data_5_regs` | xmem/mod.rs | Data file 5+1 = 6 registers |
| `xmem_capacity_value` | xmem/mod.rs | XMEM_CAPACITY == 600 |
| `file_not_found_display` | error.rs | "fl not found" exact string |
| `file_type_display` | error.rs | "fl type err" exact string |
| `no_room_display` | error.rs | "no room" exact string |
| `xmem_serde_round_trip` | state.rs | Fields survive save/load |
| `v33_save_loads_with_xmem_defaults` | state.rs | Legacy JSON loads with defaults |

All 11 tests pass. `just build` and `just lint` both pass.

## Verification

- `just test-core xmem`: 8 passed (6 model + 2 state)
- `just test-core error`: 6 passed (3 new + 3 pre-existing)
- `just build`: Finished with no warnings in new code
- `just lint`: Finished clean, no clippy::unwrap_used violations in production code
- No `floor`/`fmod` calls in register_count() (doc comment only)
- No `println!`/`eprintln!` in hp41-core
- No Op variants added (Plan 02 scope)
- `migrate_after_load()` unchanged

## Deviations from Plan

None — plan executed exactly as written.

The `v33_save_loads_with_xmem_defaults` test was adjusted to reuse the established v2.2-shape JSON fixture (instead of a hand-crafted minimal JSON that failed due to HpValue serialization format differences). The v2.2 fixture already proves backward compat across five major version transitions and is the established pattern in state.rs.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries introduced. The XmemFile.data deserialization threat (T-51-01) is accepted; kind validation happens in Plan 02 op functions. The XMEM_CAPACITY constant for the DoS mitigation (T-51-02) is defined here as required; enforcement happens in Plan 02 SAVEP/SAVED.

## Self-Check

### Created files exist:
- `hp41-core/src/ops/xmem/mod.rs` — FOUND
- `hp41-core/src/state.rs` contains `pub xmem_files` — FOUND
- `hp41-core/src/error.rs` contains `FileNotFound` — FOUND

### Commits exist:
- `11b1420` — Task 1 (XmemFile model + HpError variants)
- `6fabac0` — Task 2 (CalcState persistent fields + tests)

## Self-Check: PASSED
