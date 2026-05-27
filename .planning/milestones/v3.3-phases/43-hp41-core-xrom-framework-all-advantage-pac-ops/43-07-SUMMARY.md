---
phase: 43-hp41-core-xrom-framework-all-advantage-pac-ops
plan: 07
subsystem: hp41-core/ops/advantage
tags: [solver, polynomial, fsolve, fintg, fdifeq, froot, ply, rts, laguerre, horner, rk4, secant, simpson]
requires:
  - 43-01  # CalcState solver state fields + Op variants
provides:
  - ADV-MATH-03  # FSOLVE root finding
  - ADV-MATH-04  # FINTG numerical integration
  - ADV-MATH-05  # FDIFEQ ODE solver
  - ADV-MATH-06  # FROOT polynomial roots via Laguerre
  - ADV-MATH-07  # PLY polynomial evaluation
  - ADV-MATH-08  # RTS root output
affects:
  - hp41-core/src/ops/mod.rs        # dispatch arms
  - hp41-core/src/ops/program.rs    # run_loop arms
tech-stack:
  added: []
  patterns:
    - run_loop re-entrancy (user-program callback) for solver ops
    - Laguerre's method with quadratic deflation for conjugate complex root pairs
    - Modified secant method for FSOLVE (same pattern as math1/solve.rs)
    - Composite Simpson rule for FINTG (same pattern as math1/integ.rs)
    - Classical RK4 for FDIFEQ (same pattern as math1/difeq.rs)
    - D-43.7 cross-nesting: each solver guard checks only its own state field
key-files:
  created: []
  modified:
    - hp41-core/src/ops/advantage/solvers.rs
    - hp41-core/src/ops/advantage/poly.rs
    - hp41-core/src/ops/advantage/tvm.rs  # derivable_impls clippy fix
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
decisions:
  - "D-43.7 cross-nesting: FSOLVE guard checks only adv_fsolve_state.is_some(); FINTG guard checks only adv_fintg_state.is_some() — this enables FINTG inside FSOLVE and vice versa while blocking self-nesting"
  - "Laguerre initial guess (0.4, 0.9) rather than (0, 0) to avoid symmetry collapse for purely imaginary roots (x^2+1)"
  - "Quadratic deflation (deflate by x^2+bx+c) for complex conjugate root pairs to keep remaining polynomial real — avoids accumulated rounding error from sequential real-only linear deflation"
  - "dispatch arms in mod.rs return InvalidOp for AdvFsolveRunLoop/AdvFintgRunLoop/AdvFdifeqRunLoop; run_loop match arms in program.rs call the two-arg op_adv_*_run_loop(state, program) variants"
  - "All Laguerre/FROOT arithmetic in f64 (not HpNum) for numerical stability; converted to HpNum at final root output only"
metrics:
  duration: "~45 minutes (including debugging quadratic deflation bug)"
  completed: "2026-05-25T20:02:10Z"
  tasks_completed: 2
  files_modified: 5
---

# Phase 43 Plan 07: Solver and Polynomial Operations Summary

FSOLVE/FINTG/FDIFEQ solver ops and FROOT/PLY/RTS polynomial ops implemented for the HP-41 Advantage Pac using run_loop re-entrancy for user-program callbacks and Laguerre's method with quadratic deflation for polynomial root finding.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| T1 | Wire run_loop arms for Adv solver ops | 85c576b | ops/mod.rs, ops/program.rs |
| T2 | Implement FSOLVE/FINTG/FDIFEQ/FROOT/PLY/RTS | d91c617 | advantage/solvers.rs, advantage/poly.rs, advantage/tvm.rs |

## Implementation Details

### FSOLVE (ADV-MATH-03) — Modified secant root finding

Entry: `state.stack.y` = x1, `state.stack.x` = x2, `state.regs[25]` = function label (R25).
Algorithm: secant method with bisection fallback. Max 100 iterations. Cancel check every 64 evaluations.
Guard: `if state.adv_fsolve_state.is_some()` only — does NOT check `adv_fintg_state` (D-43.7).
Run-loop arm in `program.rs` calls `op_adv_fsolve_run_loop(state, program)`.

### FINTG (ADV-MATH-04) — Composite Simpson integration

Entry: `state.stack.y` = lower bound, `state.stack.x` = upper bound, `state.regs[24]` = function label (R24).
Algorithm: composite Simpson's rule with adaptive subdivision. 8 quadrature points minimum.
Guard: `if state.adv_fintg_state.is_some()` only — does NOT check `adv_fsolve_state` (D-43.7).
One-level nesting: FINTG inside FSOLVE works; FINTG inside FINTG returns InvalidOp.

### FDIFEQ (ADV-MATH-05) — RK4 ODE solver

Entry: `state.stack.x` = x0, `state.stack.y` = y0 (1st order) or [y0, y'0] (2nd order), `state.stack.z` = x_end.
Modal prompt for ORDER (1 or 2). Classical RK4 steps with user-program callback evaluating f(x, y).
State cleared after convergence; result pushed to stack X.

### FROOT (ADV-MATH-06) — Laguerre's method polynomial roots

Entry: `state.stack.x` = degree (1..=100), `state.regs[1..=degree+1]` = coefficients (highest-degree first).
Algorithm: Laguerre's iteration with initial guess (0.4, 0.9) to avoid symmetry collapse.
- Complex conjugate pairs: quadratic deflation (divide by x^2+bx+c) to keep polynomial real.
- Real roots: linear deflation (divide by x-r).
- Roots polished against original polynomial after deflation sequence.
Stores roots as `Vec<(f64, f64)>` (re, im) pairs in `FrootState`; `root_index` cursor for RTS.

### PLY (ADV-MATH-07) — Horner polynomial evaluation

Entry: `state.regs[0]` (R00) = degree, `state.stack.x` = evaluation point, `state.regs[1..=degree+1]` = coefficients.
Exact Horner evaluation: `result = coeffs[0]; for c: result = result * x + c`.
Domain guard: degree > 100 returns HpError::Domain.

### RTS (ADV-MATH-08) — Sequential root recall

Reads from `state.adv_froot_state`. Advances `root_index` cursor on each call; wraps at end.
Real root (|im| < 1e-10): pushes real part to X only.
Complex root: pushes imaginary part to Y, then real part to X.
Returns InvalidOp if no prior FROOT state.

## Test Coverage

| Scope | Count | Key coverage |
|-------|-------|-------------|
| FSOLVE tests | ~12 | Convergence, no-root, self-nesting guard, cross-nesting |
| FINTG tests | ~12 | Integration accuracy, zero-width, self-nesting guard, cross-nesting |
| FDIFEQ tests | ~10 | RK4 accuracy, order modal, cancel |
| FROOT tests | ~18 | Quadratic, cubic, complex roots, domain guard, cursor |
| PLY tests | ~8 | Degree 0-3, domain rejection, x=0 |
| RTS tests | ~8 | Sequential, wrap-around, no-state guard |
| Laguerre unit | ~7 | Complex arithmetic, degenerate cases |
| **Total** | **91** | All solver/poly tests pass |

Total hp41-core tests: **2472 passed, 0 failed**.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] dispatch arms in mod.rs could not call 2-argument run_loop functions**

- **Found during:** Initial implementation
- **Issue:** `mod.rs` dispatch arms received `state: &mut CalcState` only; `op_adv_fsolve_run_loop` requires `(state, program: &[Op])`. The plan implied wiring run_loop from the dispatch arm.
- **Fix:** Dispatch arms in `mod.rs` return `Err(HpError::InvalidOp)`; specific `Op::Adv*RunLoop` match arms added to `execute_op` in `program.rs` that call the two-arg variants with the program slice.
- **Files modified:** `hp41-core/src/ops/mod.rs`, `hp41-core/src/ops/program.rs`
- **Commit:** 85c576b

**2. [Rule 1 - Bug] deflate_quadratic returned wrong-length quotient polynomial**

- **Found during:** Task 2 test `froot_cubic` — polynomial x^3-1 found 4 roots instead of 3.
- **Issue:** `deflate_quadratic` used `let m = n - 2; vec![0.0; m + 1]` which produced `n - 1` elements. For length-L input divided by a degree-2 factor, the quotient has `L - 2` elements (not `L - 1`).
- **Fix:** Changed to `let q_len = len - 2; vec![0.0; q_len]` — for degree-n polynomial (len=n+1), quotient after quadratic division has n-1 coefficients.
- **Files modified:** `hp41-core/src/ops/advantage/solvers.rs`
- **Commit:** d91c617 (included in final implementation commit)

**3. [Rule 1 - Bug] Laguerre initial guess (0, 0) caused convergence failure for x^2+1**

- **Found during:** Task 2 test for complex roots (x^2+1 = 0).
- **Issue:** Starting at (0, 0): P(0)=1, P'(0)=0, so G=0. Laguerre denominator = 0 ± sqrt(n*(n*H-0)) which is real-only, causing the iteration to find only real-valued steps and fail for purely imaginary roots.
- **Fix:** Changed initial Laguerre guess to (0.4, 0.9) — a non-symmetric starting point that avoids the G=0 degeneracy.
- **Files modified:** `hp41-core/src/ops/advantage/solvers.rs`
- **Commit:** d91c617

**4. [Rule 2 - Clippy] Three clippy warnings fixed**

- `map_err` over `inspect_err` in `eval_user_fn_inner` (unnecessary mutation through map_err closure)
- `ok_or_else(|| HpError::InvalidOp)` over `ok_or(HpError::InvalidOp)` (unnecessary closure)
- Index loop `for i in 1..=n` → iterator `for &c in coeffs.iter().skip(1)` in `horner_complex`
- Also: `TvmState` manual `Default` impl replaced with `#[derive(Default)]`
- **Files modified:** `hp41-core/src/ops/advantage/solvers.rs`, `hp41-core/src/ops/advantage/tvm.rs`

## Stub Scan

No stubs remain in the implemented files. All solver operations execute full algorithms:
- `op_adv_fsolve` / `op_adv_fintg` / `op_adv_fdifeq`: dispatch stubs return `InvalidOp` intentionally (solver requires program context; run_loop arms handle the actual computation).
- `op_adv_fsolve_run_loop`, `op_adv_fintg_run_loop`, `op_adv_fdifeq_run_loop`, `op_adv_froot`, `op_adv_ply`, `op_adv_rts`: all fully implemented.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. All mitigations from the plan's threat model applied:

| Threat ID | Mitigation |
|-----------|------------|
| T-43-12 | FSOLVE: 100-iteration cap + cancel_requested check every 64 evaluations. FINTG: adaptive subdivision with convergence tolerance. FROOT: 50-iteration cap per root. |
| T-43-13 | PLY and FROOT both validate degree 1..=100 (PLY: 0..=100) with HpError::Domain. |

## Self-Check

### Created Files
- `.planning/phases/43-hp41-core-xrom-framework-all-advantage-pac-ops/43-07-SUMMARY.md` — this file

### Commits
- 85c576b: `feat(43-07): wire AdvFsolveRunLoop / AdvFintgRunLoop / AdvFdifeqRunLoop into program.rs run_loop`
- d91c617: `feat(43-07): implement FSOLVE / FINTG / FDIFEQ / FROOT / PLY / RTS for Advantage Pac`

## Self-Check: PASSED

All 2472 hp41-core tests pass. Clippy `-D warnings` clean. Both commits verified in git log.
