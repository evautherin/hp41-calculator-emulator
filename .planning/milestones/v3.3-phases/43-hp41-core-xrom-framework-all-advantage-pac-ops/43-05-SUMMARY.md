---
phase: 43
plan: 05
subsystem: hp41-core
tags: [advantage-pac, complex-math, matrix-complex, xrom-24, xrom-22]
depends_on:
  requires: [43-01]
  provides: [43-05-complex-ext, 43-05-matrix-complex]
  affects:
    - hp41-core/src/ops/advantage/complex_ext.rs
    - hp41-core/src/ops/advantage/matrix_complex.rs
    - hp41-core/src/ops/math1/complex.rs
tech_stack:
  added: []
  patterns:
    - delegation-pattern (ADV ops delegate to Math Pac I implementations)
    - complex-matrix-interleaving (2*(i*cols+j) indexing with validation helper)
    - pub(crate) visibility promotion (minimal, non-algorithm change in frozen math1/)
key_files:
  created: []
  modified:
    - hp41-core/src/ops/advantage/complex_ext.rs
    - hp41-core/src/ops/advantage/matrix_complex.rs
    - hp41-core/src/ops/math1/complex.rs
decisions:
  - "Delegate all 17 complex ext ops to Math Pac I implementations (same algorithm, different XROM)"
  - "Promote complex_atan2 from pub(super) to pub(crate) — visibility-only change, algorithm frozen"
  - "Z^(1/W) implemented as exp(ln(z)/w) using f64 bridge — not in Math Pac I"
  - "AIP reads X as Unicode codepoint, appends char to alpha_reg; negative X returns Domain"
  - "HpError::Domain used for out-of-bounds matrix index (no Bounds variant in HpError)"
  - "complex_elem_indices() helper enforces T-43-10 validation at all element access sites"
  - "C<>C uses split_at_mut for borrow-safe in-place data swap"
metrics:
  duration: "~30 minutes"
  completed: "2026-05-25T19:53:02Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 0
  files_modified: 3
---

# Phase 43 Plan 05: Complex Extensions + Complex Matrix Operations Summary

## One-liner

23 operations implemented: 18 complex extensions (MAGZ/E^Z/LNZ/LOGZ/Z^N/Z^1N/SINZ/COSZ/TANZ/A^Z/Z^W/Z^1W/AIP + 5 arithmetic variants C+/C-/CINV/C*/C/) delegating to Math Pac I where identical, plus 5 complex matrix ops (CSUM/CNRM/CMAXAB/C<>C/YC+C) with validated interleaved indexing.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Implement 18 complex number extension operations | 673809e | complex_ext.rs, math1/complex.rs |
| 2 | Implement 5 complex matrix operations | 8372020 | matrix_complex.rs |

## What Was Built

### Task 1: 18 Complex Number Extension Operations

**File:** `hp41-core/src/ops/advantage/complex_ext.rs`

Replaced all 18 stubs with full implementations:

**Delegation wrappers (17 ops):** The following ADV ops are functionally identical to Math Pac I and delegate directly:
- `op_adv_exp_z` → `op_exp_z` (e^Z)
- `op_adv_ln_z` → `op_ln_z` (LNZ, Domain guard at 0+0i)
- `op_adv_log_z` → `op_log_z` (LOGZ, Domain guard at 0+0i)
- `op_adv_z_pow_n` → `op_z_pow_n` (Z^N, N from X, base Y+iZ)
- `op_adv_z_pow_1n` → `op_z_pow_1_n` (Z^1/N)
- `op_adv_z_pow_w` → `op_z_pow_w` (Z^W, binary, T-replicate)
- `op_adv_magz` → `op_magz` (|Z|, Pythagorean magnitude)
- `op_adv_sin_z` → `op_sin_z` (complex sine)
- `op_adv_cos_z` → `op_cos_z` (complex cosine)
- `op_adv_tan_z` → `op_tan_z` (complex tangent)
- `op_adv_a_pow_z` → `op_a_pow_z` (A^Z, binary, T-replicate)
- `op_adv_c_plus` → `op_c_plus` (C+)
- `op_adv_c_minus` → `op_c_minus` (C-)
- `op_adv_cinv` → `op_cinv` (CINV, DivideByZero guard)
- `op_adv_c_mul` → `op_c_times` (C*)
- `op_adv_c_div` → `op_c_div` (C/, DivideByZero guard)

**New implementations (2 ops):**
- `op_adv_z_pow_1w`: Z^(1/W) = exp(ln(z)/w) — complex power using f64 bridge; guards for w=0 (DivideByZero) and z=0+Re(w)≤0 (Domain); T-replicate semantics; LiftEffect::Enable.
- `op_adv_aip`: AIP — reads X (truncated integer), converts to Unicode scalar, appends char to `state.alpha_reg`; negative X → Domain; LiftEffect::Neutral.

**Visibility promotion:** `complex_atan2` in `math1/complex.rs` promoted from `pub(super)` to `pub(crate)` to satisfy the plan's must_have (single-token change; algorithm untouched; frozen invariant preserved).

**Tests:** 51 tests covering all 18 operations with correctness, error paths, complex_mode, and lift effect validation.

### Task 2: 5 Complex Matrix Operations

**File:** `hp41-core/src/ops/advantage/matrix_complex.rs`

Replaced all 5 stubs with full implementations:

**Helper:** `complex_elem_indices(mat, i, j)` — validates 0-based row/col bounds and returns `(re_idx, im_idx)` for the interleaved data layout `data[2*(i*cols+j)]` / `data[2*(i*cols+j)+1]`. T-43-10 mitigation: all element accesses go through this helper.

**Implementations:**
- `op_adv_csum`: Sums all complex elements; pushes real sum to X, imaginary sum to Y. LiftEffect::Enable.
- `op_adv_cnrm`: Frobenius norm = sqrt(Σ(re² + im²)); pushes result to X. LiftEffect::Enable.
- `op_adv_cmaxab`: Maximum |z| across all elements; pushes max to X. LiftEffect::Enable.
- `op_adv_c_exchange_c`: Swaps data of active matrix (from `adv_current_matrix`) with matrix named in `alpha_reg` using `split_at_mut` for borrow-safe swap. LiftEffect::Neutral.
- `op_adv_yc_plus_c`: Adds complex scalar (X+iY) to element at (adv_matrix_i, adv_matrix_j) 1-based indices converted to 0-based. LiftEffect::Neutral.

All 5 ops validate `is_complex == true`; return `HpError::Domain` for real matrices.

**Tests:** 26 tests covering correctness, error paths (no active matrix, real matrix, out-of-bounds), complex_mode, and lift effects.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing visibility] Promote complex_atan2 to pub(crate)**
- **Found during:** Task 1 implementation
- **Issue:** `complex_atan2` was `pub(super)` — only accessible from within `math1/`; plan requires `pub(crate)` for reuse
- **Fix:** Changed `pub(super)` to `pub(crate)` in math1/complex.rs (single-token, no algorithm change)
- **Files modified:** `hp41-core/src/ops/math1/complex.rs`
- **Commit:** 673809e

**2. [Rule 1 - Missing error variant] HpError::Bounds does not exist**
- **Found during:** Task 2 implementation
- **Issue:** Plan specified "bounds check" for matrix index errors but `HpError::Bounds` is not a variant in the enum
- **Fix:** Used `HpError::Domain` (closest semantic match for invalid matrix index) consistently; tests updated accordingly
- **Files modified:** `hp41-core/src/ops/advantage/matrix_complex.rs`

## Acceptance Criteria Verification

- [x] MAGZ(3+4i) = 5.0 — test `adv_magz_3_4_is_5` passes
- [x] e^Z(0+πi) = -1+0i — test `adv_exp_z_euler_formula` passes (within 1e-6)
- [x] Advantage C+(1+2i, 3+4i) = 4+6i — test `adv_c_plus_basic` passes
- [x] AIP with X=65 appends 'A' — test `adv_aip_65_is_a` passes
- [x] No duplicate complex_atan2 — delegates to math1 via import, Pitfall 3 satisfied
- [x] All 18 complex_ext tests pass (51 total including edge cases)
- [x] CSUM of known complex matrix returns correct sum — test `adv_csum_basic` passes
- [x] CNRM produces correct Frobenius norm — test `adv_cnrm_3_4i_norm_is_5` passes
- [x] C<>C swaps two matrices' data — test `adv_c_exchange_c_swaps_data` passes
- [x] YC+C modifies the correct complex element — test `adv_yc_plus_c_basic` passes
- [x] Non-complex matrix input returns HpError::Domain — multiple tests verify
- [x] All 5 complex matrix tests pass (26 total including edge cases)

## Known Stubs

None — all 23 operations are fully implemented with tests.

## Threat Flags

None — no new network endpoints, auth paths, or schema changes introduced.

## Self-Check: PASSED

Files verified:
- `hp41-core/src/ops/advantage/complex_ext.rs` — FOUND (673809e)
- `hp41-core/src/ops/advantage/matrix_complex.rs` — FOUND (8372020)
- `hp41-core/src/ops/math1/complex.rs` — FOUND (673809e, pub(crate) promotion)

Commits verified:
- `673809e` — feat(43-05): implement 18 complex number extension operations
- `8372020` — feat(43-05): implement 5 complex matrix operations

Test count: 77 tests pass (2513 total in hp41-core — 0 failures)
