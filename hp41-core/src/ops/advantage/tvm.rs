// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `tvm` — ADV TVM: Time Value of Money operations.
//!
//! Provides `TvmState` (persistent per D-43.11) and stub implementations of
//! TVM/N/PV/PMT/FV/*I.  Full numerical solver implemented in Plan 43-06.

use crate::{
    error::HpError,
    num::HpNum,
    state::CalcState,
};

/// Maximum Newton iterations for TVM interest-rate solve (D-43.6).
pub const TVM_MAX_ITERATIONS: u8 = 100;

/// Convergence threshold for TVM *I interest-rate solve (D-43.6).
pub const TVM_CONVERGENCE_THRESHOLD: f64 = 1e-9;

/// Persistent TVM register state (D-43.11 / ADV-FW-05).
///
/// Stored on `CalcState` with `#[serde(default)]` but WITHOUT `#[serde(skip)]`
/// so TVM register values survive save/load (same pattern as `rand_seed` per ADR-v3.1-001).
/// All fields default to `HpNum::zero()` / `false` per `Default`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TvmState {
    /// Number of periods (N).
    pub n: HpNum,
    /// Periodic interest rate in % (I).
    pub i: HpNum,
    /// Present value (PV).
    pub pv: HpNum,
    /// Payment per period (PMT).
    pub pmt: HpNum,
    /// Future value (FV).
    pub fv: HpNum,
    /// Payment timing: `true` = BEGIN (annuity due), `false` = END (ordinary annuity).
    pub begin_mode: bool,
}

impl Default for TvmState {
    fn default() -> Self {
        Self {
            n: HpNum::zero(),
            i: HpNum::zero(),
            pv: HpNum::zero(),
            pmt: HpNum::zero(),
            fv: HpNum::zero(),
            begin_mode: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Stub op functions — Plan 43-06 implements full TVM solver
// ---------------------------------------------------------------------------

/// ADV TVM — open the TVM register-entry workflow (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_tvm(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV TVM N — store X into TVM N register (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_tvm_n(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV TVM PV — store X into TVM PV register (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_tvm_pv(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV TVM PMT — store X into TVM PMT register (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_tvm_pmt(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV TVM FV — store X into TVM FV register (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_tvm_fv(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

/// ADV TVM *I — solve for periodic interest rate; iterative Newton (stub).
///
/// # Errors
/// Stub: always returns `Err(HpError::InvalidOp)`.
pub fn op_adv_tvm_star_i(_state: &mut CalcState) -> Result<(), HpError> {
    Err(HpError::InvalidOp)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Catches: TvmState default initializes all fields to zero / false
    #[test]
    fn tvm_state_default() {
        let t = TvmState::default();
        assert_eq!(t.n, HpNum::zero());
        assert_eq!(t.i, HpNum::zero());
        assert_eq!(t.pv, HpNum::zero());
        assert_eq!(t.pmt, HpNum::zero());
        assert_eq!(t.fv, HpNum::zero());
        assert!(!t.begin_mode);
    }

    // Catches: TvmState can be cloned (required for CalcState::clone())
    #[test]
    fn tvm_state_clone() {
        let mut t = TvmState::default();
        t.begin_mode = true;
        let t2 = t.clone();
        assert!(t2.begin_mode);
    }

    // Catches: TvmState round-trips through JSON (D-43.11 persistence requirement)
    #[test]
    fn tvm_state_serde_round_trip() {
        let mut t = TvmState::default();
        t.n = HpNum::from(12_i32);
        t.begin_mode = true;
        let json = serde_json::to_string(&t).unwrap();
        let t2: TvmState = serde_json::from_str(&json).unwrap();
        assert_eq!(t2.n, t.n);
        assert!(t2.begin_mode);
    }

    // Catches: op stubs return InvalidOp (skeleton phase invariant)
    #[test]
    fn tvm_stubs_return_invalid_op() {
        let mut state = CalcState::new();
        assert!(matches!(op_adv_tvm(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_tvm_n(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_tvm_pv(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_tvm_pmt(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_tvm_fv(&mut state), Err(HpError::InvalidOp)));
        assert!(matches!(op_adv_tvm_star_i(&mut state), Err(HpError::InvalidOp)));
    }

    // Catches: TVM_MAX_ITERATIONS and TVM_CONVERGENCE_THRESHOLD constants present
    #[test]
    fn tvm_constants_sane() {
        assert_eq!(TVM_MAX_ITERATIONS, 100);
        assert!(TVM_CONVERGENCE_THRESHOLD < 1e-8);
    }
}
