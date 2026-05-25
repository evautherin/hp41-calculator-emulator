// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `solvers` — ADV MATH solver states + stub ops: FSOLVE / FINTG / FDIFEQ / FROOT.
//!
//! Each solver has its own transient state struct (D-43.7 re-entrancy design):
//! one active level per solver type, allowing cross-solver nesting (e.g. FINTG
//! calling FSOLVE internally).  All fields carry `#[serde(default, skip)]` on
//! `CalcState` — solver state is inherently transient.
//!
//! Full numerical implementations live in Plans 43-07 (FSOLVE/FROOT) and
//! 43-09 (FINTG/FDIFEQ).

use crate::{
    error::HpError,
    num::HpNum,
    state::CalcState,
};

// ---------------------------------------------------------------------------
// Solver state structs (transient — skipped by serde on CalcState)
// ---------------------------------------------------------------------------

/// Transient state for FROOT (root-finding with initial guess from stack).
#[derive(Debug, Clone, Default)]
pub struct FrootState {
    /// Current root estimate (X register value at entry).
    pub x_estimate: HpNum,
    /// Number of iterations consumed so far.
    pub iterations: u32,
    /// Label/name of the function being solved.
    pub function_label: String,
}

/// Transient state for FINTG (numerical integration).
#[derive(Debug, Clone, Default)]
pub struct AdvFintegState {
    /// Lower bound of integration (Y on entry).
    pub lower: HpNum,
    /// Upper bound of integration (X on entry).
    pub upper: HpNum,
    /// Current accumulated integral estimate.
    pub accumulator: HpNum,
    /// Label/name of integrand function.
    pub function_label: String,
    /// Number of sub-intervals used so far.
    pub intervals: u32,
}

/// Transient state for FSOLVE (equation solver / root-finding with two initial guesses).
#[derive(Debug, Clone, Default)]
pub struct AdvFsolveState {
    /// First initial guess (Y on entry).
    pub x0: HpNum,
    /// Second initial guess (X on entry).
    pub x1: HpNum,
    /// Current best estimate.
    pub current: HpNum,
    /// Label/name of function to solve.
    pub function_label: String,
    /// Iteration count.
    pub iterations: u32,
}

/// Transient state for FDIFEQ (differential equation integrator).
#[derive(Debug, Clone, Default)]
pub struct AdvFdifeqState {
    /// Order of the ODE system.
    pub order: u8,
    /// Current independent variable (t).
    pub t: HpNum,
    /// Current dependent-variable values y_0..y_(order-1).
    pub y: Vec<HpNum>,
    /// Step size.
    pub h: HpNum,
    /// Label/name of derivative function.
    pub function_label: String,
    /// Steps completed.
    pub steps: u32,
}

// ---------------------------------------------------------------------------
// Stub op functions — Plans 43-07 and 43-09 implement full solvers
// ---------------------------------------------------------------------------

/// ADV FSOLVE — solve f(x)=0 using Brent's method with two initial guesses (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_fsolve(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV FINTG — numerically integrate f(x) from Y to X (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_fintg(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV FDIFEQ — integrate ODE system dy/dt = f(t, y) (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_fdifeq(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV FROOT — find a root near X using secant/Newton iteration (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_froot(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV FSOLVE run-loop arm — called on each re-entry from program execution (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_fsolve_run_loop(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV FINTG run-loop arm — called on each re-entry from program execution (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_fintg_run_loop(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV FDIFEQ run-loop arm — called on each re-entry from program execution (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_fdifeq_run_loop(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Catches: FrootState default initializes cleanly
    #[test]
    fn froot_state_default() {
        let s = FrootState::default();
        assert_eq!(s.iterations, 0);
        assert!(s.function_label.is_empty());
    }

    // Catches: AdvFintegState default initializes cleanly
    #[test]
    fn finteg_state_default() {
        let s = AdvFintegState::default();
        assert_eq!(s.intervals, 0);
        assert!(s.function_label.is_empty());
    }

    // Catches: AdvFsolveState default initializes cleanly
    #[test]
    fn fsolve_state_default() {
        let s = AdvFsolveState::default();
        assert_eq!(s.iterations, 0);
        assert!(s.function_label.is_empty());
    }

    // Catches: AdvFdifeqState default initializes cleanly
    #[test]
    fn fdifeq_state_default() {
        let s = AdvFdifeqState::default();
        assert_eq!(s.order, 0);
        assert_eq!(s.steps, 0);
        assert!(s.y.is_empty());
        assert!(s.function_label.is_empty());
    }

    // Catches: solver stubs return InvalidOp (skeleton phase invariant)
    #[test]
    fn solver_stubs_return_invalid_op() {
        let mut state = CalcState::new();
        assert!(matches!(op_adv_fsolve(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_fintg(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_fdifeq(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_froot(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_fsolve_run_loop(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_fintg_run_loop(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_fdifeq_run_loop(&mut state), Err(HpError::InvalidOp)));
    }

    // Catches: solver states are Clone (required for save/restore around re-entrancy)
    #[test]
    fn solver_states_are_clone() {
        let _r = FrootState::default().clone();
        let _i = AdvFintegState::default().clone();
        let _s = AdvFsolveState::default().clone();
        let _d = AdvFdifeqState::default().clone();
    }
}
