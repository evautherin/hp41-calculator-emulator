---
phase: 43
plan: 04
subsystem: hp41-core
tags: [advantage-pac, matrix-linalg, lu-decomposition, xrom-22, mdet, minv, msys, trnps]
depends_on:
  requires: [43-03]
  provides: [43-04-matrix-linalg-implemented]
  affects: [hp41-core/src/ops/advantage/matrix_linalg.rs]
tech_stack:
  added: []
  patterns: [lu-decomposition-partial-pivoting, hpnum-f64-bridge, row-major-matrix-storage, crouts-method, numerical-recipes-algorithm]
key_files:
  created: []
  modified:
    - hp41-core/src/ops/advantage/matrix_linalg.rs
decisions:
  - "Tasks 1 and 2 combined in a single commit — all 10 ops in same file, complete implementation in one pass"
  - "LU decomposition uses implicit scaling (Crout's method, Numerical Recipes §2.3) for numerical stability"
  - "Singular matrix in MDET returns det=0 (not an error) per HP-41 Advantage Pac convention; MINV singular returns HpError::Domain"
  - "MSYS convention: A from adv_current_matrix, b from alpha_reg; result overwrites b"
  - "M*M result stored in named matrix 'ANS' (analogous to Advantage Pac OM convention)"
  - "MAT+/MAT- use zip() iterator pattern to avoid clippy::needless-index warning"
  - "find_matrix_mut uses &mut [AdvMatrix] slice signature per clippy::ptr_arg"
  - "Pre-existing clippy failures in matrix_ops.rs, matrix_complex.rs, modal.rs, tvm.rs are out of scope — deferred"
metrics:
  duration: "~25 minutes"
  completed: "2026-05-25T20:34:00Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 0
  files_modified: 1
---

# Phase 43 Plan 04: ADV MTRX Linear Algebra Ops (LU Decomposition) Summary

## One-liner

10 high-level matrix linear algebra operations implemented with hand-rolled LU decomposition (Crout's method with partial pivoting) — MDET, MINV, MSYS, M*M, MAT+, MAT-, MAT*c, MAT/c, TRNPS, MMOVE (~500 LOC core algorithms + 26 tests).

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | LU decomposition + MDET/MINV/MSYS | 87e4901 | hp41-core/src/ops/advantage/matrix_linalg.rs |
| 2 | M*M, MAT+, MAT-, MAT*c, MAT/c, TRNPS, MMOVE | 87e4901 | hp41-core/src/ops/advantage/matrix_linalg.rs (same commit) |

## What Was Built

### Core LU Decomposition Infrastructure

- **`lu_decompose(data, n)`** — Crout's method with partial pivoting (Numerical Recipes §2.3, re-derived from first principles). Returns `(perm, sign)` where perm is the row permutation vector and sign is +1.0 or -1.0. Returns `HpError::Domain` if matrix is singular (zero row or zero pivot after pivoting). Uses implicit scaling for numerical stability (~80 LOC).
- **`lu_solve(lu_data, perm, b, n)`** — Forward substitution (L unit lower triangular) followed by back substitution (U upper triangular) to solve LUx = Pb. L factors stored in lower triangle of lu_data, U in upper triangle including diagonal (~30 LOC).

### Flagship Linear Algebra Operations

- **MDET** (ADV-MTX-20): Copy matrix to f64 buffer → LU decompose → product of U diagonal × sign. Singular matrix returns det=0 (not an error). LiftEffect::Enable (result pushed to X).
- **MINV** (ADV-MTX-21): LU decompose copy → solve n identity columns → write back n×n inverse in-place. Singular matrix returns `HpError::Domain`. LiftEffect::Neutral.
- **MSYS** (ADV-MTX-22): LU decompose A → solve each column of b → overwrite b with solution x. Convention: A from `adv_current_matrix`, b from `alpha_reg`. LiftEffect::Neutral.

### Matrix Arithmetic Operations

- **M*M** (ADV-MTX-23): Standard O(n·m·k) matrix multiply. A from `adv_current_matrix`, B from `alpha_reg`. Result stored in named matrix "ANS". Validates A.cols == B.rows. LiftEffect::Neutral.
- **MAT+** (ADV-MTX-24): Element-wise addition of B into A (validates same dimensions). zip() iterator pattern avoids clippy warnings.
- **MAT-** (ADV-MTX-25): Element-wise subtraction of B from A (validates same dimensions).
- **MAT*c** (ADV-MTX-26): Scalar multiply all elements by X register in-place. LiftEffect::Neutral.
- **MAT/c** (ADV-MTX-27): Scalar divide all elements by X register in-place. Rejects X=0 with `HpError::Domain`. LiftEffect::Neutral.
- **TRNPS** (ADV-MTX-28): Transpose matrix in-place. Swaps rows↔cols dimensions; rewrites data as `new[j*old_rows+i] = old[i*old_cols+j]`. LiftEffect::Neutral.
- **MMOVE** (ADV-MTX-29): Copy all elements from src (alpha_reg) to dst (adv_current_matrix). Validates identical dimensions. LiftEffect::Neutral.

### Internal Helpers

- **`find_matrix(matrices, name)`** — Immutable lookup by name; `HpError::InvalidOp` if not found.
- **`find_matrix_mut(matrices, name)`** — Mutable lookup using `&mut [AdvMatrix]` slice signature (per clippy::ptr_arg).
- **`current_matrix_name(state)`** — Priority: `adv_current_matrix` > trimmed `alpha_reg`.
- **`hpnum_to_f64(n)`** / **`f64_to_hpnum(v)`** — f64 round-trip bridge (same pattern as asin/acos in num.rs).
- **`matrix_to_f64(mat)`** — Copy matrix data to f64 working buffer.

## Test Coverage

- 26 tests in `ops::advantage::matrix_linalg::tests`
- Full hp41-core suite: 2698 passed, 2 ignored (up from 2483 in plan 43-01, 2483 → 2698 total)
- All behavior-specified tests pass:
  - MDET 2x2 [[1,2],[3,4]] = -2 ✓
  - MDET 3x3 [[1,2,3],[4,5,6],[7,8,10]] = -3 ✓
  - MDET singular [[1,2],[2,4]] → det ≈ 0 ✓
  - MDET non-square → HpError::Domain ✓
  - MDET lift effect: Y=99 preserved when lift_enabled ✓
  - MINV 2x2 [[1,2],[3,4]] → [[-2,1],[1.5,-0.5]] ✓
  - MINV singular → HpError::Domain ✓
  - MINV non-square → HpError::Domain ✓
  - MINV 3x3 round-trip: inv[0,0] = 5/8 ✓
  - MSYS 2x2 A=[[2,1],[5,3]], b=[4,7] → x=[5,-6] ✓
  - MSYS dimension mismatch → HpError::Domain ✓
  - MSYS no A name → HpError::InvalidOp ✓
  - M*M [[1,2],[3,4]] × [[5,6],[7,8]] = [[19,22],[43,50]] ✓
  - M*M dimension mismatch → HpError::Domain ✓
  - MAT+ [[1,2],[3,4]] + [[5,6],[7,8]] = [[6,8],[10,12]] ✓
  - MAT+ dimension mismatch → HpError::Domain ✓
  - MAT- [[5,6],[7,8]] - [[1,2],[3,4]] = [[4,4],[4,4]] ✓
  - MAT*c ×2 on [[1,2],[3,4]] = [[2,4],[6,8]] ✓
  - MAT/c ÷2 on [[2,4],[6,8]] = [[1,2],[3,4]] ✓
  - MAT/c zero scalar → HpError::Domain ✓
  - TRNPS 2×3 → 3×2, data transposed correctly ✓
  - TRNPS 2×2 square transpose ✓
  - MMOVE copies 4 elements SRC → DST ✓
  - MMOVE dimension mismatch → HpError::Domain ✓
  - LU decompose 2×2 det = -2 ✓
  - LU decompose singular → HpError::Domain ✓

## Verification

- `cargo test -p hp41-core -- matrix_linalg`: 26 passed
- `cargo test -p hp41-core`: 2698 passed, 2 ignored
- D-43.5 isolation: no production code reads/writes `state.matrix_dim` or `state.matrix_active_reg`
- Free42 contamination: algorithms independently derived from Numerical Recipes primary sources; verbatim disclaim header present

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Design] Tasks 1 and 2 combined in a single commit**
- **Found during:** Task 1 implementation
- **Issue:** Both tasks implement functions in the same file (matrix_linalg.rs). Splitting would require an intermediate commit with 3/10 ops as stubs — same pattern as plan 43-03.
- **Fix:** Implemented all 10 ops in a single commit with complete test coverage for all 10.
- **Files modified:** `hp41-core/src/ops/advantage/matrix_linalg.rs`
- **Commit:** 87e4901

### Out-of-Scope Issues (Deferred)

Pre-existing clippy `-D warnings` failures in files from other plans (not introduced by this plan):
- `matrix_ops.rs`: unused imports `ADV_MATRIX_MAX_COLS`/`ADV_MATRIX_MAX_ROWS`, `&mut Vec` should be `&mut [_]`
- `matrix_complex.rs`: needless explicit lifetime `'a`
- `modal.rs`: two `unnecessary_unwrap` calls on `adv_tvm_state`
- `tvm.rs`: `manual_clamp` pattern

These were logged to `deferred-items.md` per deviation scope rules; they predate this plan.

## Known Stubs

None — all 10 matrix_linalg.rs operations are fully implemented. The remaining stub ops in other advantage/ sub-modules (matrix_complex.rs, etc.) are out of scope for this plan.

## Threat Flags

None — no new network endpoints, auth paths, or trust-boundary changes.

T-43-09 mitigation implemented: LU decomposition detects near-zero pivots during both `lu_decompose()` calls (implicit scaling, zero-pivot check) and returns `HpError::Domain` for singular matrices in MINV/MSYS. MDET returns det=0 (not an error) for singular matrices per HP-41 Advantage Pac hardware behavior.

## Self-Check: PASSED

- [x] `hp41-core/src/ops/advantage/matrix_linalg.rs` — 26 tests implemented, all pass
- [x] Commit: 87e4901
- [x] `cargo test -p hp41-core -- matrix_linalg`: 26 passed
- [x] `cargo test -p hp41-core`: 2698 passed, 2 ignored
- [x] D-43.5 isolation: no production code references to `matrix_dim` or `matrix_active_reg`
- [x] All 10 required functions (ADV-MTX-20 through ADV-MTX-29) implemented
