---
phase: 51-x-mem-core
plan: "02"
subsystem: hp41-core
tags: [x-mem, ops, 4-way-match, cardreader, serde]
dependency_graph:
  requires: [51-01]
  provides: [op_emdir, op_emroom, op_savep, op_getp, op_saved, op_getd, op_emreg, op_saverx, Op::EmDir..SaveRx]
  affects:
    - hp41-core/src/ops/xmem/ops.rs
    - hp41-core/src/ops/xmem/mod.rs
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-cli/src/prgm_display.rs
    - hp41-gui/src-tauri/src/prgm_display.rs
tech_stack:
  added: []
  patterns:
    - cardreader codec delegation (encode/decode_program, capture/load_data_card, encode/decode_data)
    - upsert_file overwrite-in-place (D-51.6)
    - capacity-check-before-mutate (Pitfall 3 / D-51.1)
    - trunc_int index extraction via Decimal::trunc (no floor/fmod)
    - alpha_name() validation (HpError::AlphaData on empty)
    - 4-way exhaustive match (8 variants × 4 arms)
key_files:
  created:
    - hp41-core/src/ops/xmem/ops.rs
  modified:
    - hp41-core/src/ops/xmem/mod.rs
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-cli/src/prgm_display.rs
    - hp41-gui/src-tauri/src/prgm_display.rs
decisions:
  - index_from_x() uses trunc_int().inner().to_usize() (rust_decimal ToPrimitive trait) — matches hpnum_to_u8 pattern from advantage/matrix_ops.rs
  - op_saverx uses a two-step find+decode then find_file_mut write-back to avoid borrow-checker conflict with immutable and mutable borrows coexisting
  - emdir pushes header + per-file lines + footer; footer always shows "N REGS FREE" matching HP-41CX QRG p.39 catalog output style
  - EMREG uses numeric_or_zero() for the register recall value (same as other recall ops)
metrics:
  duration_minutes: 20
  tasks_completed: 2
  files_created: 1
  files_modified: 5
  tests_added: 15
  completed_date: "2026-05-28"
---

# Phase 51 Plan 02: X-MEM Op Implementation Summary

**One-liner:** Eight X-MEM ops (EMDIR/EMROOM/SAVEP/GETP/SAVED/GETD/EMREG/SAVERX) implemented with full error coverage, delegating to the Phase 50 cardreader codec, wired through the 4-way exhaustive match so the workspace compiles.

## What Was Built

### Task 1: Implement the 8 X-MEM ops in xmem/ops.rs

Created `hp41-core/src/ops/xmem/ops.rs` with the 8 public op functions plus all private helpers and unit tests:

**Private helpers:**
- `alpha_name(state)` — validates ALPHA register (copied from cardreader_ops.rs pattern)
- `find_file(state, name)` / `find_file_mut(state, name)` — immutable/mutable file lookup
- `upsert_file(state, file)` — overwrite-in-place per D-51.6 (find + replace, else push)
- `registers_used(state)` — sum of `register_count()` across all xmem_files
- `existing_file_regs(state, name)` — lookup for capacity-check-before-mutate
- `index_from_x(state)` — extract register index via `trunc_int().inner().to_usize()` (NEVER floor/fmod)

**Op implementations:**
- `op_emdir`: catalog to `state.print_buffer` (header + per-file lines + footer). NEVER uses `println!`. LiftEffect: Neutral.
- `op_emroom`: `enter_number(state, 600 - used)`. LiftEffect: Enable.
- `op_savep`: `alpha_name` → `encode_program` → capacity check → `upsert_file`. LiftEffect: Neutral.
- `op_getp`: `alpha_name` → `find_file` → kind check → `decode_program` → `insert_program_ops`. LiftEffect: Neutral.
- `op_saved`: `alpha_name` → `capture_data_card` → `encode_data` → capacity check → `upsert_file` → set `xmem_active_file`. LiftEffect: Neutral.
- `op_getd`: `alpha_name` → `find_file` → kind check → `decode_data` → `load_data_card` → set `xmem_active_file`. LiftEffect: Neutral.
- `op_emreg`: `index_from_x` → `xmem_active_file` → `find_file` → `decode_data` → `card.registers.get(n)` → `enter_number`. LiftEffect: Enable.
- `op_saverx`: `index_from_x` → active file → `decode_data` → `registers.get_mut(n)` → write Y → `encode_data` → `find_file_mut` write-back. LiftEffect: Neutral.

Added `pub mod ops;` declaration in `hp41-core/src/ops/xmem/mod.rs`.

### Task 2: Wire 8 Op variants through the 4-way exhaustive match

- **`hp41-core/src/ops/mod.rs`**: Added 8 Op variants (`EmDir`..`SaveRx`) in a Phase 51 block after `AdvTvmStarI`. Added 8 `dispatch()` arms routing to `crate::ops::xmem::ops::op_*`.
- **`hp41-core/src/ops/program.rs`**: Added 8 `execute_op()` arms delegating to `crate::ops::dispatch(state, Op::*)` (Phase 43 pattern).
- **`hp41-cli/src/prgm_display.rs`**: Added 8 `op_display_name` arms with mnemonic strings.
- **`hp41-gui/src-tauri/src/prgm_display.rs`**: Added identical 8 arms (intentional duplication per CLAUDE.md).

## Test Coverage

| Test | Verifies |
|------|---------|
| `emdir` | print_buffer catalog with header + footer; empty store shows 600 free |
| `emroom` | 600 on empty store; decrements after DATA file saved |
| `savep_round_trip` | SAVEP creates PROGRAM file in xmem_files |
| `getp_round_trip` | GETP restores program Op vector intact |
| `saved_round_trip` | SAVED creates DATA file + sets xmem_active_file |
| `getd_round_trip` | GETD restores regs (len >= 100); sets xmem_active_file |
| `emreg_saverx` | EMREG recall + SAVERX store with index from X; xmem_files.len() unchanged |
| `savep_no_room` | NoRoom when capacity exceeded; xmem_files not mutated |
| `getp_not_found` | FileNotFound on missing name |
| `getp_type_mismatch` | FileType when GETP on DATA file |
| `getd_type_mismatch` | FileType when GETD on PROGRAM file |
| `emreg_no_active` | FileNotFound when xmem_active_file is None |
| `emreg_out_of_range` | OutOfRange when N >= reg count |
| `overwrite_in_place` | D-51.6: xmem_files.len() unchanged on duplicate save; new content wins |
| `emroom_capacity_boundary` | Exactly 600 succeeds (EMROOM = 0); one over → NoRoom |

All 15 op tests + 8 pre-existing xmem/mod tests = 23 total pass. Full core suite: 2991 tests passed.

## Verification

- `cargo test -p hp41-core -- xmem`: 23 passed (15 ops + 8 model)
- `just build`: Finished with no warnings — workspace compiles, 4-way exhaustive match complete
- `just lint`: Finished clean, no clippy::unwrap_used violations in production code
- No `println!`/`eprintln!` in hp41-core/src/ops/xmem/ops.rs (2 instances are comments only)
- No `floor`/`fmod` calls — index via `trunc_int().inner().to_usize()`
- No `adv_matrices` access in ops.rs (isolation invariant preserved)
- No direct `state.regs[` access in ops.rs except via load_data_card/capture_data_card

## Deviations from Plan

**1. [Rule 1 - Bug] Fixed emreg_saverx test logic error**
- **Found during:** Task 1 verification
- **Issue:** The initial test had a confusing comment block with a wrong assertion (`assert_eq!(state.stack.x, HpNum::from(0i32))`) where the correct value was 30 (regs[2] = 30 in the test setup)
- **Fix:** Rewrote the test to use a clean state and correct assertions
- **Files modified:** `hp41-core/src/ops/xmem/ops.rs`
- **Commit:** d162a99 (test was part of the original implementation commit; rewrite within same commit after test failure)

## Known Stubs

None — all 8 ops are fully implemented and connect to the Phase 50 cardreader codec. The `op_display_name` arms in both prgm_display.rs files return minimal mnemonic strings; full keyboard/IPC/help-overlay wiring is Phase 52 scope.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes. All T-51-03..T-51-07 mitigations from the threat register are implemented:
- T-51-03: `alpha_name()` returns AlphaData on empty ALPHA
- T-51-04: `index_from_x()` uses trunc_int + to_usize; `registers.get(n).ok_or(OutOfRange)`
- T-51-05: kind check before decode in GETP/GETD/EMREG/SAVERX
- T-51-06: capacity check before mutate in SAVEP/SAVED
- T-51-07: `xmem_active_file.ok_or(FileNotFound)` in EMREG/SAVERX

## Self-Check

### Created files exist:
- `hp41-core/src/ops/xmem/ops.rs` — FOUND
- Contains `pub fn op_emroom` — FOUND
- Contains `pub fn op_emdir` — FOUND

### Commits exist:
- `d162a99` — Task 1 (8 X-MEM ops + tests in xmem/ops.rs)
- `ee35b6c` — Task 2 (8 Op variants wired through 4-way exhaustive match)

## Self-Check: PASSED
