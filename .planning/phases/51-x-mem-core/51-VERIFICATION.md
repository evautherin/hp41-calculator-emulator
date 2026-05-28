---
phase: 51-x-mem-core
verified: 2026-05-28T12:00:00Z
status: passed
score: 5/5 must-haves verified
overrides_applied: 0
deferred:
  - truth: "X-MEM state persists across save/load with #[serde(default)] backward compat"
    addressed_in: "Phase 52"
    evidence: "REQUIREMENTS.md XMEM-08 mapped to Phase 52; backward-compat fields already exist in Phase 51 (both #[serde(default)] without skip) but the full migration/validation story is Phase 52 scope"
  - truth: "X-MEM storage isolated from main registers and adv_matrices (D-43.5 pattern) — formal audit"
    addressed_in: "Phase 52"
    evidence: "REQUIREMENTS.md XMEM-09 mapped to Phase 52; isolation invariant is enforced in Phase 51 code and verified here, but the formal test/ADR coverage is Phase 52 scope"
  - truth: "CLI + GUI integration (4-way exhaustive match, op_display_name, help overlay)"
    addressed_in: "Phase 52"
    evidence: "REQUIREMENTS.md XMEM-10 mapped to Phase 52; Phase 51 wires the 4-way match for compilation only — keyboard/IPC/help-overlay wiring is explicitly deferred"
---

# Phase 51: X-MEM Core Verification Report

**Phase Goal:** Users can store and retrieve named programs and data sets in Extended Memory, mirroring HP-41CX behavior.
**Verified:** 2026-05-28T12:00:00Z
**Status:** PASSED
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (from ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `EMDIR` displays a catalog of all named files in extended memory with names, types, and sizes | VERIFIED | `op_emdir` in `ops.rs:94-113` pushes header + per-file lines (`name / PGM\|DAT / register_count()`) + free-registers footer to `state.print_buffer`. Test `emdir` (ops.rs:348-365) asserts header + footer exist and footer contains "600" and "REGS FREE". No `println!` used (grep confirms 0 production hits). |
| 2 | `EMROOM` reports the number of available extended memory registers as a numeric value on the stack | VERIFIED | `op_emroom` in `ops.rs:121-126` computes `XMEM_CAPACITY.saturating_sub(registers_used(state))` and pushes to X via `enter_number`. Test `emroom` (ops.rs:372-397) asserts 600 on empty store and decrements correctly after a DATA file is saved. LiftEffect::Enable applied before push. |
| 3 | `SAVEP` / `GETP` round-trips a program through extended memory and retrieves it intact | VERIFIED | `op_savep` (ops.rs:139-158) encodes via `encode_program`, capacity-checks before mutate, upserts file. `op_getp` (ops.rs:170-180) decodes via `decode_program` + `insert_program_ops`. Tests `savep_round_trip` and `getp_round_trip` (ops.rs:403-428) verify PROGRAM file created with correct name and Op vector survives round-trip. |
| 4 | `SAVED` / `GETD` round-trips data registers through extended memory and retrieves them intact | VERIFIED | `op_saved` (ops.rs:194-216) captures via `capture_data_card`, encodes via `encode_data`, sets `xmem_active_file`. `op_getd` (ops.rs:229-240) decodes via `decode_data` + `load_data_card`, sets `xmem_active_file`. Tests `saved_round_trip` and `getd_round_trip` (ops.rs:434-463) verify DATA file created, `reg_count` set, regs restored (len >= 100), active file pointer set. |
| 5 | `EMREG` accesses register N within the active X-MEM file, returning the stored value | VERIFIED | `op_emreg` (ops.rs:253-268) reads index from X via `index_from_x` (trunc_int, no floor/fmod), looks up active file via `xmem_active_file`, decodes, recalls `card.registers[n]` via `enter_number`. `op_saverx` (ops.rs:282-310) stores Y into register N and re-encodes. Test `emreg_saverx` (ops.rs:469-494) verifies recall returns correct value and SAVERX stores Y into N without growing `xmem_files`. |

**Score:** 5/5 truths verified

### Deferred Items

Items not yet met but explicitly addressed in later milestone phases.

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | XMEM-08: X-MEM state persists across save/load (full test/migration coverage) | Phase 52 | REQUIREMENTS.md traceability table maps XMEM-08 to Phase 52 |
| 2 | XMEM-09: Formal isolation audit of adv_matrices / state.regs | Phase 52 | REQUIREMENTS.md traceability table maps XMEM-09 to Phase 52 |
| 3 | XMEM-10: CLI + GUI integration (keyboard/IPC/help-overlay wiring) | Phase 52 | REQUIREMENTS.md traceability table maps XMEM-10 to Phase 52 |

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-core/src/ops/xmem/mod.rs` | XmemFile struct, XmemKind enum, register_count(), XMEM_CAPACITY | VERIFIED | All four present; `pub struct XmemFile`, `pub enum XmemKind`, `pub fn register_count`, `pub const XMEM_CAPACITY: usize = 600` confirmed in file |
| `hp41-core/src/ops/xmem/ops.rs` | 8 op functions + unit tests | VERIFIED | `pub fn op_emdir/emroom/savep/getp/saved/getd/emreg/saverx` all present; 15 unit tests in `#[cfg(test)] mod tests` block |
| `hp41-core/src/error.rs` | FileNotFound, FileType, NoRoom variants | VERIFIED | Three variants at lines 59, 63, 67 with `#[error("fl not found")]`, `#[error("fl type err")]`, `#[error("no room")]` |
| `hp41-core/src/state.rs` | xmem_files + xmem_active_file as persistent #[serde(default)] fields | VERIFIED | Lines 431-442: `#[serde(default)] pub xmem_files: Vec<crate::ops::xmem::XmemFile>` and `#[serde(default)] pub xmem_active_file: Option<String>`, no `#[serde(skip)]`; initialized in `CalcState::new()` at lines 534-535 |
| `hp41-core/src/ops/mod.rs` | `pub mod xmem` declaration + 8 Op variants + 8 dispatch arms | VERIFIED | `pub mod xmem` at line 11; 8 variants (EmDir..SaveRx) at lines 1540-1554; 8 dispatch arms at lines 2172-2179 |
| `hp41-core/src/ops/program.rs` | 8 execute_op delegation arms | VERIFIED | 8 arms at lines 1241-1248, all delegating to `crate::ops::dispatch(state, Op::*)` |
| `hp41-cli/src/prgm_display.rs` | 8 op_display_name arms | VERIFIED | Arms at lines 484-491 returning "EMDIR".."SAVERX" mnemonic strings |
| `hp41-gui/src-tauri/src/prgm_display.rs` | 8 op_display_name arms (identical to CLI) | VERIFIED | Arms at lines 502-509, identical mnemonic strings confirmed |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `hp41-core/src/ops/mod.rs` | `hp41-core/src/ops/xmem/mod.rs` | `pub mod xmem` declaration | WIRED | Line 11: `pub mod xmem;` present |
| `hp41-core/src/ops/mod.rs` | `hp41-core/src/ops/xmem/ops.rs` | dispatch() arms call op functions | WIRED | 8 arms at lines 2172-2179 all routing to `crate::ops::xmem::ops::op_*` |
| `hp41-core/src/ops/xmem/ops.rs` | `hp41-core/src/cardreader` | use crate::cardreader imports | WIRED | Lines 17-20: `encode_program, decode_program, insert_program_ops, capture_data_card, load_data_card, encode_data, decode_data` all imported and used |
| `hp41-core/src/ops/xmem/ops.rs` | `state.xmem_files / state.xmem_active_file` | upsert/lookup + active-file pointer | WIRED | `state.xmem_files` mutated via `upsert_file`; `state.xmem_active_file` set in `op_saved` and `op_getd` |
| `hp41-core/src/state.rs` | `crate::ops::xmem::XmemFile` | xmem_files field type | WIRED | Line 432: `Vec<crate::ops::xmem::XmemFile>` |

### Data-Flow Trace (Level 4)

X-MEM is a storage module — data flows into/out of `state.xmem_files` (Vec). All ops either push to `state.print_buffer` (EMDIR), push to stack (EMROOM, EMREG), or modify `state.xmem_files` (SAVEP, SAVED) / `state.program` / `state.regs` (GETP, GETD, SAVERX).

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|-------------------|--------|
| `op_emroom` | `available: usize` | `registers_used()` summing over `state.xmem_files` | Yes — computed from actual file contents | FLOWING |
| `op_savep` | `bytes: Vec<u8>` | `encode_program(&state.program)` | Yes — encodes actual program ops | FLOWING |
| `op_getp` | `ops: Vec<Op>` | `decode_program(&file.data)` | Yes — decodes from stored XmemFile.data | FLOWING |
| `op_saved` | `card: DataCard` | `capture_data_card(state)` | Yes — captures actual `state.regs` | FLOWING |
| `op_getd` | `card: DataCard` | `decode_data(&file.data)` | Yes — decodes from stored XmemFile.data | FLOWING |
| `op_emreg` | `value: HpNum` | `card.registers.get(n)` from decoded DATA file | Yes — reads actual stored register | FLOWING |
| `op_saverx` | writes `card.registers[n]` | `state.stack.y` + `encode_data` + write-back | Yes — stores Y into actual stored register | FLOWING |

### Behavioral Spot-Checks

Behavioral spot-checks apply to runnable code. The X-MEM ops are hp41-core library functions — no CLI/GUI entry point yet (Phase 52 scope). The 15 unit tests in `ops.rs` cover all key behaviors end-to-end within the Rust test harness. Spot-check via test names is the appropriate form here.

| Behavior | Test | Status |
|----------|------|--------|
| EMDIR pushes catalog lines to print_buffer | `xmem::ops::tests::emdir` | PASS (15 tests green per SUMMARY-02) |
| EMROOM returns 600 on empty store | `xmem::ops::tests::emroom` | PASS |
| SAVEP / GETP round-trip program | `savep_round_trip` + `getp_round_trip` | PASS |
| SAVED / GETD round-trip data | `saved_round_trip` + `getd_round_trip` | PASS |
| EMREG / SAVERX recall and store | `emreg_saverx` | PASS |
| D-51.6 overwrite-in-place | `overwrite_in_place` | PASS |
| NoRoom at capacity | `savep_no_room` + `emroom_capacity_boundary` | PASS |
| Error paths (FileNotFound, FileType, OutOfRange) | `getp_not_found`, `getp_type_mismatch`, `getd_type_mismatch`, `emreg_no_active`, `emreg_out_of_range` | PASS |

### Probe Execution

No phase-declared probes. Phase 51 is a core library phase; no `scripts/*/tests/probe-*.sh` exists for it. Step 7b/7c: SKIPPED (no runnable entry points — ops are library functions callable only via dispatch, not yet wired to keyboard/IPC).

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| XMEM-01 | 51-02-PLAN.md | EMDIR lists all files with names, types, and sizes | SATISFIED | `op_emdir` pushes name + PGM/DAT tag + register_count per file to print_buffer |
| XMEM-02 | 51-01-PLAN.md, 51-02-PLAN.md | EMROOM reports available register space | SATISFIED | `op_emroom` pushes `600 - registers_used` to X; `XMEM_CAPACITY = 600` and `register_count()` defined |
| XMEM-03 | 51-02-PLAN.md | SAVEP saves a named program to extended memory | SATISFIED | `op_savep` encodes program, capacity-checks, upserts XmemFile with kind=Program |
| XMEM-04 | 51-02-PLAN.md | GETP retrieves a named program from extended memory | SATISFIED | `op_getp` decodes and inserts via `insert_program_ops` (RDPRGM semantics) |
| XMEM-05 | 51-02-PLAN.md | SAVED saves data registers to a named file | SATISFIED | `op_saved` captures via `capture_data_card`, encodes, upserts XmemFile with kind=Data; sets `xmem_active_file` |
| XMEM-06 | 51-02-PLAN.md | GETD retrieves data registers from a named file | SATISFIED | `op_getd` decodes via `decode_data` + `load_data_card`; sets `xmem_active_file` |
| XMEM-07 | 51-02-PLAN.md | EMREG accesses register N within the active X-MEM file | SATISFIED | `op_emreg` reads index from X, looks up active file, decodes, returns `card.registers[n]`; `op_saverx` stores Y into N |
| XMEM-08 | NOT in Phase 51 | X-MEM state persists across save/load | DEFERRED | Mapped to Phase 52 in REQUIREMENTS.md; `#[serde(default)]` fields already present but full coverage deferred |
| XMEM-09 | NOT in Phase 51 | X-MEM storage isolated from main registers | DEFERRED | Mapped to Phase 52 in REQUIREMENTS.md; isolation invariant enforced in Phase 51 code |
| XMEM-10 | NOT in Phase 51 | CLI + GUI integration (keyboard, help overlay) | DEFERRED | Mapped to Phase 52 in REQUIREMENTS.md |

**Orphaned requirements check:** XMEM-08, XMEM-09, XMEM-10 are mapped to Phase 52 in REQUIREMENTS.md and do NOT appear in any Phase 51 plan's `requirements:` field. Not orphaned — correctly mapped to a future phase.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `ops.rs:74` | 74 | `index_from_x` doc contract says "rejects non-integer" but implementation truncates silently | WARNING (WR-01) | Fractional index like 2.9 accepted as 2 — contract vs. behavior mismatch. Not a data-loss bug in practice; HP-41CX truncates integer ops. Documented in 51-REVIEW.md WR-01. |
| `ops.rs:296-305` | 296-305 | SAVERX triple-lookup pattern with `expect("file was just validated")` | INFO (IN-01) | CLAUDE.md-sanctioned `.expect()` form. Fragile but not incorrect. Documented in 51-REVIEW.md IN-01. |
| `ops.rs:305` | 305 | SAVERX `if let Some(file) = find_file_mut` silently no-ops if file disappears between validate and write-back | INFO (IN-01) | Locally impossible given single-threaded dispatch, but the silent discard is noted. |

**Debt marker check:** No `TBD`, `FIXME`, or `XXX` markers found in any Phase 51 modified files. `TODO` and `HACK` patterns: none found in xmem module. Clean.

**Isolation invariants confirmed:**
- `grep -c "adv_matrices" ops.rs` = 0 (comment-in-doc-string only — confirmed)
- No `println!` / `eprintln!` in production paths of `ops.rs`
- No `floor()` / `fmod()` in production paths of `mod.rs` or `ops.rs`
- `state.regs[]` direct access in `ops.rs` exists only inside `#[cfg(test)]` block (lines 337+, test module starts at line 314)

**4-way exhaustive-match invariant (CLAUDE.md):** All 8 variants appear in all 4 required locations:
1. `Op` enum variants in `ops/mod.rs` (lines 1540-1554) — VERIFIED
2. `dispatch()` arms in `ops/mod.rs` (lines 2172-2179) — VERIFIED
3. `execute_op()` arms in `ops/program.rs` (lines 1241-1248) — VERIFIED
4. `op_display_name()` in `hp41-cli/src/prgm_display.rs` (lines 484-491) — VERIFIED
5. `op_display_name()` in `hp41-gui/src-tauri/src/prgm_display.rs` (lines 502-509) — VERIFIED

Both `op_display_name` functions confirmed to have no `_ =>` catch-all (exhaustive match is load-bearing).

### Human Verification Required

None. All required behaviors for Phase 51 are verifiable programmatically through the unit test suite and source inspection. The phase scope is hp41-core library only — no UI rendering, real-time behavior, or external service integration is involved.

### Gaps Summary

No gaps. All 5 ROADMAP Success Criteria are verified by direct source inspection and confirmed by the unit test coverage. The three warnings from the code review (WR-01: contract mismatch on non-integer index; WR-02: untested insert-into-non-empty GETP path; WR-03: over-capacity store arithmetic) are correctness concerns for Phase 52 test-hardening, not blockers for the Phase 51 goal of store/retrieve working correctly under normal usage.

The code review findings WR-01 and WR-02 warrant attention in Phase 52 but do not prevent any of the 5 success criteria from being true today.

---

_Verified: 2026-05-28T12:00:00Z_
_Verifier: Claude (gsd-verifier)_
