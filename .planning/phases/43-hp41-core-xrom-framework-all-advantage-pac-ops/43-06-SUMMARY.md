---
phase: 43-hp41-core-xrom-framework-all-advantage-pac-ops
plan: "06"
subsystem: hp41-core/ops/advantage
tags: [curve-fitting, vectors, ols-regression, 3d-math, advantage-pac]
dependency_graph:
  requires: ["43-01"]
  provides: [curve_fit_ops, vector_ops]
  affects: [hp41-core]
tech_stack:
  added: []
  patterns:
    - OLS curve fitting via log-linearization (linear/log/exp/power models)
    - f64 bridge via Decimal::inner().to_f64() / Decimal::from_f64()
    - Named register block constants for all indexed accesses (T-43-11 mitigation)
    - 3D vector arithmetic with read_vec/write_vec helpers
key_files:
  created: []
  modified:
    - hp41-core/src/ops/advantage/curve_fit.rs
    - hp41-core/src/ops/advantage/vectors.rs
decisions:
  - "Curve fitting uses R10-R19 register block (avoids R01-R06 Σ-block conflict with Stat 1 Pac)"
  - "f64 bridge for all curve fitting/vector math (OLS + trigonometry require f64 precision)"
  - "BFIT reports |r| of stored accumulator sums (single-block limitation — OM §4.5 convention)"
  - "TR implements X-Y plane rotation (most common HP-41 coordinate transform use case)"
  - "VE sets modal prompt via ModalProgram::Advantage(VeComponentPrompt(1))"
metrics:
  duration: "~18 minutes"
  completed: "2026-05-25T19:54:17Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 2
  tests_added: 27
---

# Phase 43 Plan 06: Curve Fitting + Vector Operations Summary

**One-liner:** OLS curve fitting (4-model log-linearization) and 3D vector algebra (14 ops) using named register blocks R10–R28.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Implement 7 curve fitting operations | 6f5293d | `hp41-core/src/ops/advantage/curve_fit.rs` |
| 2 | Implement 14 vector and coordinate transform operations | eecbe62 | `hp41-core/src/ops/advantage/vectors.rs` |

## What Was Built

### Task 1: Curve Fitting (curve_fit.rs)

Replaced all 7 stubs with full implementations using an OLS (ordinary least squares) regression engine:

**Register block R10–R19** (named constants defined at file top):
- `CFIT_N_REG` (R10) — sample count n
- `CFIT_SUM_X_REG` (R11) — Σx (transformed for log/power models)
- `CFIT_SUM_Y_REG` (R12) — Σy (transformed for exp/power models)
- `CFIT_SUM_X2_REG` (R13) — Σx²
- `CFIT_SUM_XY_REG` (R14) — Σxy
- `CFIT_SUM_Y2_REG` (R15) — Σy²
- `CFIT_MODEL_REG` (R16) — active model: 0=linear, 1=log, 2=exp, 3=power
- `CFIT_COEFF_A_REG` (R17) — intercept a (scale factor for exp/power)
- `CFIT_COEFF_B_REG` (R18) — slope b
- `CFIT_CORR_REG` (R19) — Pearson correlation r

**Operations implemented:**
- `CFIT` (ADV-MATH-27): clears R10–R19, resets model to linear
- `AS` (ADV-MATH-28): accumulates (X,Y) data point with log-linearization per active model
- `DS` (ADV-MATH-29): removes (X,Y) data point (inverse of AS)
- `BFIT` (ADV-MATH-30): reports |r| for current stored accumulator
- `FIT` (ADV-MATH-31): computes OLS a, b, r; pushes b to Y, a to X
- `Y?X` (ADV-MATH-32): predicts Y from X using stored coefficients and model
- `SZ?` (ADV-MATH-33): pushes 1 if n ≥ 3, else 0

**10 tests added** covering all acceptance criteria.

### Task 2: Vector Operations (vectors.rs)

Replaced all 14 stubs with 3D vector arithmetic:

**Register block R20–R28** (named constants defined at file top):
- `VEC_A_BASE` = 20 (vector A: R20, R21, R22)
- `VEC_B_BASE` = 23 (vector B: R23, R24, R25)
- `VEC_RESULT_BASE` = 26 (result vector: R26, R27, R28)

**Helper functions:**
- `read_vec(state, base)` → `(f64, f64, f64)`
- `write_vec(state, base, v)`

**Operations implemented:**
- `V+` (ADV-MATH-34): element-wise A+B → result registers
- `V-` (ADV-MATH-35): element-wise A−B → result registers
- `DOT` (ADV-MATH-36): A·B → X (scalar)
- `CROSS` (ADV-MATH-37): A×B → result registers + magnitude to X
- `VC` (ADV-MATH-38): alias for CROSS
- `VS` (ADV-MATH-39): scalar(X) × A → result registers
- `VR` (ADV-MATH-40): recall result → push to X/Y/Z stack
- `VE` (ADV-MATH-41): vector entry modal (VeComponentPrompt(1))
- `VXY` (ADV-MATH-42): project A to X-Y plane (zero Z component of A)
- `UV` (ADV-MATH-43): normalize A to unit vector → result registers
- `V<` (ADV-MATH-44): |A| → X (scalar)
- `VD` (ADV-MATH-45): alias for DOT
- `V*` (ADV-MATH-46): alias for VS
- `TR` (ADV-MATH-47): rotate A by angle-in-degrees from X in X-Y plane → result registers

**17 tests added** covering all acceptance criteria.

## Test Results

- Task 1: 10 tests pass (`cargo test -p hp41-core -- adv_cfit adv_as adv_fit adv_y_query adv_sz adv_bfit adv_ds cfit_register`)
- Task 2: 17 tests pass (`cargo test -p hp41-core -- adv_dot adv_cross adv_v_plus adv_v_minus adv_vs adv_uv adv_v_mag adv_vr adv_ve adv_vxy adv_tr adv_vc adv_vd adv_v_star vec_register`)
- Full suite: 2463 tests pass (0 failures, 2 ignored)
- Clippy: clean on both files (`cargo clippy -p hp41-core -- -D warnings` reports 0 errors on our files)

## Decisions Made

1. **R10–R19 register block for curve fitting:** Avoids conflict with Stat 1 Pac R01–R06 Σ-block and Math Pac I matrix registers. Named constants enumerate all 10 slots.

2. **f64 bridge for all math:** OLS computation (SS_xx, SS_xy, Pearson r) and trigonometry (TR rotation) require f64. Pattern follows existing `hp41-core/src/ops/math.rs` convention using `Decimal::inner().to_f64()` / `Decimal::from_f64()`.

3. **BFIT single-block limitation:** With one accumulator block, BFIT computes |r| from the stored sums (data was linearized under the active model at AS time). This matches HP Advantage Pac OM §4.5 convention where the user runs AS under each model separately.

4. **TR as X-Y rotation:** The OM §3 coordinate transformation is implemented as a 2D rotation in the X-Y plane, which is the most common use case. Z component is preserved unchanged.

5. **VE modal entry:** Sets `state.modal_program = Some(ModalProgram::Advantage(VeComponentPrompt(1)))` and `state.modal_prompt = Some("V[1]=?")`. The modal dispatch infrastructure from Plan 43-01 handles the R/S submission sequence.

## Deviations from Plan

None — plan executed exactly as written. All 21 operations implemented and tested.

## Known Stubs

None in this plan. Both files are fully implemented.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. All register accesses use named constants — T-43-11 (register index out of bounds) mitigated by named constants in valid range R10–R28 and SIZE-floor guards.

## Self-Check: PASSED

- Files exist: `hp41-core/src/ops/advantage/curve_fit.rs` ✓, `hp41-core/src/ops/advantage/vectors.rs` ✓
- Task 1 commit `6f5293d` exists ✓
- Task 2 commit `eecbe62` exists ✓
- 2463 tests pass, 0 failures ✓
