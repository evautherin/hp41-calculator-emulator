---
phase: 43-hp41-core-xrom-framework-all-advantage-pac-ops
plan: "08"
subsystem: hp41-core/advantage/modal
tags: [advantage-pac, modal-workflow, matrix, medit, cmedit, matrx, mtr]
dependency_graph:
  requires: ["43-01", "43-03"]
  provides: ["43-08"]
  affects: ["hp41-core/src/ops/advantage/modal.rs", "hp41-core/src/ops/advantage/matrix_workflow.rs", "hp41-core/src/state.rs"]
tech_stack:
  added: []
  patterns:
    - "pending_adv_matrix_name / pending_adv_matrix_rows transient carrier fields (following pending_chisqd_nu pattern)"
    - "check_matrix_exists pub(crate) helper for cross-module modal routing"
    - "next_element_1based row-major traversal helper"
    - "get_or_insert_with for TvmState initialization (replaces expect pattern)"
key_files:
  created: []
  modified:
    - hp41-core/src/ops/advantage/matrix_workflow.rs
    - hp41-core/src/ops/advantage/modal.rs
    - hp41-core/src/state.rs
decisions:
  - "CMEDIT reads X=real and Y=imag in one submit step (HP-41 stack convention Y+iX) rather than two separate prompts — avoids adding a second enum variant or complex state tracking"
  - "pending_adv_matrix_name + pending_adv_matrix_rows transient fields added to CalcState following the pending_chisqd_nu precedent (D-33 WR-03/WR-04 pattern)"
  - "MatrixDimColPrompt manually sets stack Y=rows, X=cols before calling op_adv_matdim to avoid double-drop stack corruption"
  - "MEDIT/CMEDIT require adv_current_matrix to be set; return HpError::InvalidOp if not — consistent with op_adv_mr/op_adv_ms behavior"
  - "FDIFEQ workflow: FdifeqOrderPrompt initializes AdvFdifeqState (order + y vector), FdifeqFunctionNamePrompt stores label and clears modal"
  - "VeComponentPrompt stores to R20/R21/R22 (VEC_A_BASE=20) matching vectors.rs register layout"
  - "TVM arms refactored to use get_or_insert_with() instead of expect() pattern, fixing clippy warning"
metrics:
  duration: "~45 minutes"
  completed: "2026-05-25"
  tasks_completed: 1
  tasks_total: 1
  files_changed: 3
  tests_added: 40
  tests_total: 2704
---

# Phase 43 Plan 08: Matrix Workflow Modal Frontends Summary

Matrix workflow modal operations (MATRX, MTR, MEDIT, CMEDIT) and full `submit_step` dispatch for all 15 `AdvantageStep` variants implemented with 40 new tests, zero regressions.

## What Was Built

### matrix_workflow.rs — 4 modal-opening ops

| Op | Step Opened | Initial Prompt |
|----|-------------|----------------|
| `op_adv_matrx` | `MatrixNamePrompt` | `MNAME?` |
| `op_adv_mtr` | `MtrNamePrompt` | `MTR NAME?` |
| `op_adv_medit` | `MeditElementPrompt(1,1)` | `[1,1]=?` |
| `op_adv_cmedit` | `CmeditElementPrompt(1,1)` | `C[1,1]=?` |

Added helpers:
- `next_element_1based(r, c, rows, cols) → Option<(u8, u8)>` — row-major traversal
- `check_matrix_exists(state, name) → bool` — cross-module matrix lookup

### modal.rs — full submit_step dispatch

All 15 `AdvantageStep` variants now fully implemented (no `InvalidOp` stubs except when delegating to stub linalg ops that are Plan 43-05's responsibility):

**TVM workflow** (preserved from prior plan, refactored to clippy-clean `get_or_insert_with`):
- `TvmN → TvmI → TvmPv → TvmPmt → TvmFv → TvmBeginEnd → done`

**MATRX workflow** (new):
- `MatrixNamePrompt` — reads ALPHA; existing matrix → `MatrxOperationChoice`; new → `MatrixDimRowPrompt`
- `MatrixDimRowPrompt` — validates rows > 0 ≤ 255, stores to `pending_adv_matrix_rows`
- `MatrixDimColPrompt` — validates cols, calls `op_adv_matdim` with stored name/rows, → `MatrxOperationChoice`
- `MatrxOperationChoice` — choice 1/2/3 → `op_adv_mdet`/`op_adv_minv`/`op_adv_msys`

**MTR workflow** (new): Parallel to MATRX — `MtrNamePrompt` branches to either `MatrxOperationChoice` (existing) or `MatrixDimRowPrompt` (new)

**MEDIT element loop** (new): `MeditElementPrompt(r, c)` stores X via `op_adv_ms`, advances row-major, clears modal at last element

**CMEDIT complex loop** (new): `CmeditElementPrompt(r, c)` reads Y=imag, X=real, writes interleaved to complex matrix data; advances row-major

**VE component entry** (new): `VeComponentPrompt(k)` stores X to R(19+k) for k∈{1,2,3}; clears after component 3

**FDIFEQ config** (new): `FdifeqOrderPrompt` validates order ∈ {1,2}, inits `AdvFdifeqState`; `FdifeqFunctionNamePrompt` stores ALPHA label and clears modal

### state.rs — 2 new transient fields

```rust
#[serde(default, skip)]
pub pending_adv_matrix_name: Option<String>,

#[serde(default, skip)]
pub pending_adv_matrix_rows: Option<u8>,
```

Both follow the `pending_chisqd_nu` pattern (D-43 / T-43-14 threat mitigation).

## Acceptance Criteria Status

- [x] MATRX opens modal with "MNAME?" prompt via `AdvantageStep::MatrixNamePrompt`
- [x] `submit_step` for `MatrixNamePrompt` reads ALPHA and advances to `DimRowPrompt` (new matrix) or `MatrxOperationChoice` (existing)
- [x] `submit_step` for `DimRowPrompt` stores rows and advances to `DimColPrompt`
- [x] MEDIT iterates through all matrix elements with correct prompts
- [x] `modal_program` and `modal_prompt` cleared after workflow completion
- [x] All 15 `AdvantageStep` variants handled in `submit_step` — exhaustive match, no `_ =>` catch-all
- [x] Tests exercise MATRX name→dim→operation workflow end-to-end

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] TvmI/TvmBeginEnd used expect() pattern**
- **Found during:** Clippy run after implementation
- **Issue:** Original `if is_none { ... expect() } else { ... expect() }` pattern triggers clippy warning
- **Fix:** Replaced with `get_or_insert_with(TvmState::default)` per clippy recommendation
- **Files modified:** `hp41-core/src/ops/advantage/modal.rs`
- **Commit:** a70e401

**2. [Rule 2 - Missing critical functionality] validate_dim + MAX_DIM helper removed as unused**
- **Found during:** Clippy warning (dead_code)
- **Issue:** validate_dim was defined but the actual validation is done inline via x_to_dim() in modal.rs
- **Fix:** Removed unused constant and function from matrix_workflow.rs
- **Commit:** a70e401

### Pre-existing Issues (Not Fixed — Out of Scope)

- `matrix_ops.rs:29` unused import warnings (`ADV_MATRIX_MAX_COLS`, `ADV_MATRIX_MAX_ROWS`) — pre-existing, not in this plan's files
- `matrix_complex.rs:62` explicit lifetime could be elided — pre-existing
- `tvm.rs:283` clamp-like pattern — pre-existing
- `hp41-cli` non-exhaustive match compilation error — pre-existing Phase 43 deferred CLI integration

## Test Results

- 40 new tests added across `ops::advantage::matrix_workflow::tests` and `ops::advantage::modal::tests`
- 2704 total `hp41-core` tests pass (0 failures, 2 ignored pre-existing)
- Clippy clean on hp41-core (4 pre-existing warnings in unmodified files)

## Known Stubs

The following stubs remain intentional — delegated to their respective plans:
- `op_adv_mdet`, `op_adv_minv`, `op_adv_msys` — Plan 43-05 (linear algebra)
- The `MatrxOperationChoice → op_adv_mdet/minv/msys` workflow reaches these stubs and returns `InvalidOp`; this is expected until Plan 43-05 ships

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced.
T-43-14 (modal input validation) mitigated: dimension entries validated via `x_to_dim()` (rejects 0, validates ≤ 255); operation choice validated as 1/2/3 with `HpError::Domain` on invalid input.

## Self-Check: PASSED

Files verified:
- `hp41-core/src/ops/advantage/matrix_workflow.rs` — FOUND
- `hp41-core/src/ops/advantage/modal.rs` — FOUND
- `hp41-core/src/state.rs` (pending_adv_matrix_name, pending_adv_matrix_rows) — FOUND

Commit `a70e401` verified in git log.
