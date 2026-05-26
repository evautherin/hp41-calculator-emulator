---
phase: 43
plan: 01
subsystem: hp41-core
tags: [advantage-pac, xrom, skeleton, framework, stub]
depends_on:
  requires: [phase-42]
  provides: [43-01-advantage-skeleton, 43-01-xrom-framework]
  affects: [hp41-core, math1/xrom.rs, math1/modal.rs, state.rs, ops/mod.rs, ops/program.rs]
tech_stack:
  added: []
  patterns: [xrom-module-registry, stub-op-pattern, serde-default-skip, modal-dispatch-carve-out]
key_files:
  created:
    - hp41-core/src/ops/advantage/mod.rs
    - hp41-core/src/ops/advantage/modal.rs
    - hp41-core/src/ops/advantage/tvm.rs
    - hp41-core/src/ops/advantage/solvers.rs
    - hp41-core/src/ops/advantage/conv.rs
    - hp41-core/src/ops/advantage/matrix_ops.rs
    - hp41-core/src/ops/advantage/matrix_linalg.rs
    - hp41-core/src/ops/advantage/matrix_complex.rs
    - hp41-core/src/ops/advantage/complex_ext.rs
    - hp41-core/src/ops/advantage/poly.rs
    - hp41-core/src/ops/advantage/matrix_workflow.rs
    - hp41-core/src/ops/advantage/curve_fit.rs
    - hp41-core/src/ops/advantage/vectors.rs
  modified:
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-core/src/ops/math1/xrom.rs
    - hp41-core/src/ops/math1/modal.rs
    - hp41-core/src/ops/math1/mod.rs
    - hp41-core/src/state.rs
    - hp41-core/tests/stat1_backward_compat.rs
    - hp41-core/tests/time_backward_compat.rs
    - hp41-core/tests/v3_save_compat.rs
    - scripts/check-free42-contamination.sh
decisions:
  - "HpNum::ZERO not defined — used HpNum::zero() instead (Rule 1 auto-fix)"
  - "TvmState default() hand-written instead of derived because HpNum doesn't impl Default via zero"
  - "ADV_MATH_A.ops count is 63 (23 element-access ops, not 22 as estimated)"
  - "xrom_modules tests in stat1_backward_compat, time_backward_compat, v3_save_compat updated to 0b0001_1111 (Rule 1 test fixes)"
  - "ModalProgram::Advantage arm added to math1/mod.rs submit_step (Rule 2 — missing exhaustive match)"
metrics:
  duration: "~90 minutes"
  completed: "2026-05-25T19:34:57Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 13
  files_modified: 10
---

# Phase 43 Plan 01: Advantage Pac XROM Framework Skeleton Summary

## One-liner

HP Advantage Pac skeleton — 13 sub-modules with ~117 stub ops, XROM 22+24 registration, new CalcState fields, and ModalProgram::Advantage variant wired into hp41-core.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Create advantage/ module skeleton | 7d3ebae | 13 new files in hp41-core/src/ops/advantage/ |
| 2 | Wire XROM framework | 60b6a83 | ops/mod.rs, math1/xrom.rs, math1/modal.rs, state.rs, program.rs, contamination script |

## What Was Built

### Task 1: advantage/ Module Skeleton (13 files)

Created the complete HP Advantage Pac sub-module skeleton under `hp41-core/src/ops/advantage/`:

- **mod.rs** — Module hub with `AdvMatrix` struct (ALPHA-register named-matrix model), `ADV_WORD_MASK` (36-bit, 0x0000_000F_FFFF_FFFF), `ADV_MATRIX_MAX_ROWS/COLS` constants, and `pub use` re-exports for all ~117 op functions.
- **modal.rs** — `AdvantageStep` enum (16 variants covering TVM/matrix/solver/vector modal workflows), `current_prompt()`, `requires_alpha_label()`, `submit_step()` (all stubs return `Err(HpError::InvalidOp)`).
- **tvm.rs** — `TvmState` struct (persistent, no `#[serde(skip)]` per D-43.11), 6 TVM stub ops.
- **solvers.rs** — `FrootState`, `AdvFintegState`, `AdvFsolveState`, `AdvFdifeqState` (all transient Default structs), 7 solver stub ops.
- **conv.rs** — 12 ADV CONV stubs (BININ/BINVIEW/OCTIN/HEXIN/HEXVIEW/CVTVIEW/NOT/AND/OR/XOR/ROTXY/BIT?).
- **matrix_ops.rs** — 33 ADV MTRX element-access/lifecycle/reduction stubs with D-43.5 isolation invariant comments.
- **matrix_linalg.rs** — 10 ADV MTRX linalg stubs (MDET/MINV/MSYS/M*M/MAT+/MAT-/MAT*c/MAT/c/TRNPS/MMOVE) with D-43.5 comments.
- **matrix_complex.rs** — 5 ADV MTRX complex stubs (C<>C/CMAXAB/CNRM/CSUM/YC+C) with D-43.5 comments.
- **complex_ext.rs** — 18 ADV MATH complex extension stubs (e^Z/LNZ/LOG Z/Z^N/Z^(1/N)/Z^W/Z^(1/W)/|Z|/SIN Z/COS Z/TAN Z/A^Z/ADV C+/C-/CINV/C*/C//AIP).
- **poly.rs** — 2 ADV MATH polynomial stubs (PLY/RTS).
- **matrix_workflow.rs** — 4 ADV MTRX workflow modal stubs (MATRX/MTR/MEDIT/CMEDIT) with D-43.5 comments.
- **curve_fit.rs** — 7 ADV MATH curve-fit stubs (CFIT/AS/DS/BFIT/FIT/Y?X/SZ?).
- **vectors.rs** — 14 ADV MATH vector stubs (V+/V-/DOT/CROSS/VC/VS/VR/VE/VXY/UV/|V|/V*/VD/TR).

Every file carries the verbatim Free42 disclaim header per contamination guard policy.

### Task 2: XROM Framework Wiring

- **ops/mod.rs** — Added `pub mod advantage`, 117 `Op::Adv*` variants (two commented sections for XROM 22 + XROM 24), and 117 dispatch arms routing to the stub functions.
- **ops/program.rs** — Added 117 `execute_op()` arms (pattern: `Op::AdvX => crate::ops::dispatch(state, Op::AdvX)`).
- **math1/xrom.rs** — Added `ADV_MATH_A` (id=22, "ADV CONV", 63 ops) and `ADV_MATH_B` (id=24, "ADV MATH", 51 ops) `XromModule` constants; `adv_a_resolve()` and `adv_b_resolve()` resolver functions; bit-3 and bit-4 arms in `xrom_resolve()`; 12 new tests including isolation and bidirectional consistency.
- **math1/modal.rs** — Added `ModalProgram::Advantage(crate::ops::advantage::modal::AdvantageStep)` variant (4th carve-out per D-43.6); updated `current_prompt()`, `requires_alpha_label()` match arms.
- **math1/mod.rs** — Added `ModalProgram::Advantage(step)` arm to `submit_step` match.
- **state.rs** — Updated `default_xrom_modules()` from `0b0000_0111` to `0b0001_1111`; added 9 new CalcState fields (4 persistent: `adv_matrices`, `adv_matrix_i`, `adv_matrix_j`, `adv_tvm_state`; 5 transient: `adv_current_matrix`, `adv_froot_state`, `adv_fintg_state`, `adv_fsolve_state`, `adv_fdifeq_state`); added bit-3 and bit-4 migration guards in `migrate_after_load()`; updated `CalcState::new()` initializers.
- **scripts/check-free42-contamination.sh** — Added `ADV_DIR="hp41-core/src/ops/advantage"` as 4th scanned directory in both the existence check loop and the pattern scan loop.
- **Backward-compat tests** — Updated `stat1_backward_compat.rs`, `time_backward_compat.rs`, `v3_save_compat.rs` expected migration target from `0b0000_0111` to `0b0001_1111`.

## Verification

- `cargo check -p hp41-core`: PASSED
- `cargo test -p hp41-core`: 2439 passed, 2 ignored
- `bash scripts/check-free42-contamination.sh`: OK, exits 0
- `cargo check -p hp41-cli`: Expected sanctioned CI break (117 Adv* variants not in CLI `op_display_name()` — Phase 44 closes item 3)
- `cargo check -p hp41-gui`: Not checked (same sanctioned break, Phase 46 closes item 4)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] HpNum::ZERO does not exist**
- **Found during:** Task 1, tvm.rs Default impl
- **Issue:** `HpNum::ZERO` is not an associated constant on the HpNum type; `HpNum::zero()` is the correct method.
- **Fix:** Replaced all `HpNum::ZERO` with `HpNum::zero()` in tvm.rs.
- **Files modified:** `hp41-core/src/ops/advantage/tvm.rs`
- **Commit:** 7d3ebae (included in skeleton commit)

**2. [Rule 1 - Bug] HpNum does not implement TryFrom<f64>**
- **Found during:** Task 1, tvm.rs test
- **Issue:** Test used `HpNum::try_from(12.0_f64)` which fails as HpNum only implements `From<i32>` and `From<Decimal>`.
- **Fix:** Changed to `HpNum::from(12_i32)` in the serde round-trip test.
- **Files modified:** `hp41-core/src/ops/advantage/tvm.rs`
- **Commit:** 7d3ebae

**3. [Rule 2 - Missing Critical Functionality] ModalProgram::Advantage arm missing in submit_step**
- **Found during:** Task 2, cargo check
- **Issue:** `math1/mod.rs` submit_step match was non-exhaustive — `ModalProgram::Advantage(_)` not handled.
- **Fix:** Added `ModalProgram::Advantage(step) => crate::ops::advantage::modal::submit_step(state, step)` arm.
- **Files modified:** `hp41-core/src/ops/math1/mod.rs`
- **Commit:** 60b6a83

**4. [Rule 1 - Bug] Backward-compat tests hardcoded v3.2 xrom_modules value**
- **Found during:** Task 2, cargo test
- **Issue:** `stat1_backward_compat.rs`, `time_backward_compat.rs`, `v3_save_compat.rs` all asserted `state.xrom_modules == 0b0000_0111` after migration, but the v3.3 migration now sets bits 3+4, making the expected value `0b0001_1111`.
- **Fix:** Updated all three test files to expect `0b0001_1111` post-migration.
- **Files modified:** `hp41-core/tests/stat1_backward_compat.rs`, `hp41-core/tests/time_backward_compat.rs`, `hp41-core/tests/v3_save_compat.rs`
- **Commit:** 60b6a83

**5. [Rule 1 - Bug] ADV_MATH_A.ops entry count miscounted (62 vs 63)**
- **Found during:** Task 2, cargo test
- **Issue:** Test asserted `ADV_MATH_A.ops.len() == 62` but actual count is 63 (23 element-access ops, not 22 as estimated).
- **Fix:** Updated test assertion to 63.
- **Files modified:** `hp41-core/src/ops/math1/xrom.rs`
- **Commit:** 60b6a83

## Sanctioned CI Break

Items 3+4 of the 4-way exhaustive-match invariant are intentionally deferred:
- Item 3 (CLI `prgm_display.rs` op_display_name): Phase 44 closes this.
- Item 4 (GUI `prgm_display.rs` op_display_name): Phase 46 closes this.

`cargo check -p hp41-cli` fails with "non-exhaustive patterns: AdvBinin, AdvBinview, AdvOctin and 114 more not covered" — this is the correct and expected failure documented in the plan.

## Known Stubs

All ~117 `Op::Adv*` ops return `Err(HpError::InvalidOp)` — this is intentional. Each subsequent plan in Phase 43 (43-02 through 43-10) implements specific subsets:
- 43-02: ADV CONV binary/octal I/O (BININ/BINVIEW/OCTIN)
- 43-03: ADV CONV hex I/O and bitwise (HEXIN/HEXVIEW/CVTVIEW/NOT/AND/OR/XOR/ROTXY/BIT?)
- 43-04: ADV MTRX element access and lifecycle
- 43-05: ADV MTRX reductions, linalg, complex ops
- 43-06: ADV TVM solver
- 43-07: ADV MATH complex extensions, PLY/RTS, FSOLVE/FROOT
- 43-08: ADV MATH matrix workflow modals (MEDIT/CMEDIT/MATRX/MTR)
- 43-09: ADV MATH FINTG/FDIFEQ/curve-fit/vectors
- 43-10: CLI integration (closes item 3 of 4-way invariant)

## Threat Flags

None — no new network endpoints, auth paths, or trust-boundary changes. The advantage/ module is pure computational state with standard serde persistence.

## Self-Check: PASSED

- [x] hp41-core/src/ops/advantage/ — 13 files created
- [x] hp41-core/src/ops/mod.rs — pub mod advantage + 117 Op variants + 117 dispatch arms
- [x] hp41-core/src/ops/program.rs — 117 execute_op arms
- [x] hp41-core/src/ops/math1/xrom.rs — ADV_MATH_A + ADV_MATH_B + resolvers + bit-3/4 arms
- [x] hp41-core/src/ops/math1/modal.rs — ModalProgram::Advantage variant
- [x] hp41-core/src/state.rs — default_xrom_modules=0b0001_1111 + 9 new fields + migration guards
- [x] scripts/check-free42-contamination.sh — ADV_DIR added
- [x] Commits: 7d3ebae (Task 1) + 60b6a83 (Task 2)
- [x] cargo test -p hp41-core: 2439 passed
- [x] contamination guard: exits 0
