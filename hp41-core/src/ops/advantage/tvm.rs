// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `tvm` — ADV TVM: Time Value of Money operations.
//!
//! Provides `TvmState` (persistent per D-43.11) and full implementations of
//! TVM/N/PV/PMT/FV/*I with Newton-Raphson interest-rate solver.

use crate::{
    error::HpError,
    num::HpNum,
    ops::advantage::modal::AdvantageStep,
    ops::math1::modal::ModalProgram,
    stack::{apply_lift_effect, enter_number, LiftEffect},
    state::CalcState,
};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

/// Maximum Newton iterations for TVM interest-rate solve (D-43.6).
pub const TVM_MAX_ITERATIONS: u8 = 100;

/// Convergence threshold for TVM *I interest-rate solve (D-43.6).
pub const TVM_CONVERGENCE_THRESHOLD: f64 = 1e-9;

/// Persistent TVM register state (D-43.11 / ADV-FW-05).
///
/// Stored on `CalcState` with `#[serde(default)]` but WITHOUT `#[serde(skip)]`
/// so TVM register values survive save/load (same pattern as `rand_seed` per ADR-v3.1-001).
/// All fields default to `HpNum::zero()` / `false` per `Default`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
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

// ---------------------------------------------------------------------------
// TVM helper: get_or_init TvmState
// ---------------------------------------------------------------------------

/// Return a reference to the mutable TvmState, initializing it if needed.
#[inline]
fn tvm_state(state: &mut CalcState) -> &mut TvmState {
    state.adv_tvm_state.get_or_insert_with(TvmState::default)
}

// ---------------------------------------------------------------------------
// TVM equation helpers (END mode, annuity-immediate)
// ---------------------------------------------------------------------------
//
// TVM equation (END mode):
//   f(i) = PV + PMT * (1 - (1+i)^(-N)) / i + FV * (1+i)^(-N) = 0
//
// Special form at i = 0 (zero-interest limit):
//   f(0) = PV + PMT * N + FV
//
// For BEGIN mode (annuity-due), PMT term is multiplied by (1+i).
//
// Derivative (END mode):
//   f'(i) = PMT * [(1-(1+i)^(-N))/i^2 - N*(1+i)^(-N-1)/i] - N*FV*(1+i)^(-N-1)
//
// All arithmetic done in f64 via ToPrimitive bridge (same pattern as stat1/distributions.rs).

/// Evaluate the TVM function f(i) and its derivative f'(i) at periodic rate i.
/// Returns `(f, f_prime)`.
///
/// Arguments are all in their natural units:
/// - `n`: number of periods
/// - `pv`: present value
/// - `pmt`: payment per period
/// - `fv`: future value
/// - `i`: periodic interest rate (fractional, e.g. 0.005 for 0.5%)
/// - `begin`: true for BEGIN (annuity-due) mode
fn tvm_f_and_fprime(n: f64, pv: f64, pmt: f64, fv: f64, i: f64, begin: bool) -> (f64, f64) {
    if i.abs() < 1e-15 {
        // Zero-rate limit: f(0) = PV + PMT*N + FV
        // f'(0) using L'Hopital: d/di [PMT*(1-(1+i)^-N)/i] at i→0 = PMT*N*(N+1)/2
        // f'(0) = PMT * N*(N+1)/2 - N*FV
        let f = pv + pmt * n + fv;
        let fprime = pmt * n * (n + 1.0) / 2.0 - n * fv;
        return (f, fprime);
    }

    let v = (1.0 + i).powf(-n); // (1+i)^(-N)
    let annuity_factor = (1.0 - v) / i; // (1-(1+i)^-N) / i

    // PMT adjustment for BEGIN vs END mode
    let pmt_eff = if begin { pmt * (1.0 + i) } else { pmt };

    let f = pv + pmt_eff * annuity_factor + fv * v;

    // Derivative
    // d/di[(1-(1+i)^-N)/i] = [-N*(1+i)^(-N-1) * i - (1-(1+i)^-N)] / i^2
    //                       = [-N*v/(1+i) * i - (1-v)] / i^2
    let v_over_1pi = v / (1.0 + i); // (1+i)^(-N-1)
    let d_annuity = (-n * v_over_1pi * i - (1.0 - v)) / (i * i);

    let fprime = if begin {
        // f = PV + PMT*(1+i)*annuity + FV*v
        // f' = PMT * annuity + PMT*(1+i)*d_annuity - N*FV*v_over_1pi
        pmt * annuity_factor + pmt * (1.0 + i) * d_annuity - n * fv * v_over_1pi
    } else {
        // f = PV + PMT*annuity + FV*v
        // f' = PMT*d_annuity - N*FV*v_over_1pi
        pmt * d_annuity - n * fv * v_over_1pi
    };

    (f, fprime)
}

// ---------------------------------------------------------------------------
// Full op functions
// ---------------------------------------------------------------------------

/// ADV TVM — open the TVM register-entry workflow.
///
/// Sets `state.modal_program = Some(ModalProgram::Advantage(AdvantageStep::TvmN))`
/// and `state.modal_prompt = Some("N=?")` to begin the interactive TVM data-entry
/// workflow. The user cycles through N→I→PV→PMT→FV prompts; each R/S submission
/// is handled by `modal::submit_step`.
///
/// # Errors
/// Returns `Err(HpError::InvalidOp)` if the modal framework is unavailable.
pub fn op_adv_tvm(state: &mut CalcState) -> Result<(), HpError> {
    state.modal_program = Some(ModalProgram::Advantage(AdvantageStep::TvmN));
    state.modal_prompt = Some("N=?".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV TVM N — store X into TVM N register (ADV-TVM-02).
///
/// Reads the X register, stores it as `tvm.n`. Initializes TvmState
/// if not already present. LiftEffect::Neutral (HP-41 register-store convention).
///
/// # Errors
/// Returns `Err(HpError::Overflow)` if the X value cannot be converted.
pub fn op_adv_tvm_n(state: &mut CalcState) -> Result<(), HpError> {
    let x = state.stack.x.clone();
    tvm_state(state).n = x;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV TVM PV — store X into TVM PV register (ADV-TVM-03).
///
/// Reads the X register, stores it as `tvm.pv`. LiftEffect::Neutral.
///
/// # Errors
/// Returns `Err(HpError::Overflow)` if the X value cannot be converted.
pub fn op_adv_tvm_pv(state: &mut CalcState) -> Result<(), HpError> {
    let x = state.stack.x.clone();
    tvm_state(state).pv = x;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV TVM PMT — store X into TVM PMT register (ADV-TVM-04).
///
/// Reads the X register, stores it as `tvm.pmt`. LiftEffect::Neutral.
///
/// # Errors
/// Returns `Err(HpError::Overflow)` if the X value cannot be converted.
pub fn op_adv_tvm_pmt(state: &mut CalcState) -> Result<(), HpError> {
    let x = state.stack.x.clone();
    tvm_state(state).pmt = x;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV TVM FV — store X into TVM FV register (ADV-TVM-05).
///
/// Reads the X register, stores it as `tvm.fv`. LiftEffect::Neutral.
///
/// # Errors
/// Returns `Err(HpError::Overflow)` if the X value cannot be converted.
pub fn op_adv_tvm_fv(state: &mut CalcState) -> Result<(), HpError> {
    let x = state.stack.x.clone();
    tvm_state(state).fv = x;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV TVM *I — solve for periodic interest rate via Newton-Raphson iteration (ADV-TVM-06).
///
/// Uses Newton-Raphson iteration on the TVM equation:
/// `f(i) = PV + PMT * (1 - (1+i)^(-N)) / i + FV * (1+i)^(-N) = 0`
///
/// All arithmetic is done in f64 via the `ToPrimitive` bridge (same pattern as
/// `stat1/distributions.rs`). The result `i` (periodic rate) is stored as `tvm.i`
/// and pushed to X as `i * 100` (percent).
///
/// On convergence: stores `i` as `tvm.i`, pushes `i * 100` (percent) to X.
///   LiftEffect::Enable.
///
/// On non-convergence (D-43.13): pushes the last iterate to X, pushes "NO SOLUTION"
///   to `print_buffer`, returns `HpError::NoRoot`.
///
/// Special cases:
/// - N == 0: returns `HpError::Domain`.
/// - Zero initial guess leads to zero-rate special form.
///
/// # Errors
/// - `HpError::Domain` if N == 0 or TvmState is not initialized.
/// - `HpError::NoRoot` if Newton iteration does not converge within TVM_MAX_ITERATIONS.
pub fn op_adv_tvm_star_i(state: &mut CalcState) -> Result<(), HpError> {
    // Read TVM registers from state (or default if not yet initialized)
    let (n_hp, pv_hp, pmt_hp, fv_hp, begin) = {
        let tvm = tvm_state(state);
        (
            tvm.n.clone(),
            tvm.pv.clone(),
            tvm.pmt.clone(),
            tvm.fv.clone(),
            tvm.begin_mode,
        )
    };

    // Convert to f64 for Newton iteration
    let n = n_hp.inner().to_f64().ok_or(HpError::Overflow)?;
    let pv = pv_hp.inner().to_f64().ok_or(HpError::Overflow)?;
    let pmt = pmt_hp.inner().to_f64().ok_or(HpError::Overflow)?;
    let fv = fv_hp.inner().to_f64().ok_or(HpError::Overflow)?;

    // N == 0 is a domain error
    if n == 0.0 {
        return Err(HpError::Domain);
    }

    // Initial guess: derive a sensible per-period rate estimate.
    // For typical financial scenarios (PV positive, PMT negative, FV near 0),
    // the zero-rate approximation gives: PV + PMT*N + FV ≈ 0
    // → PMT ≈ -(PV + FV)/N → i starts near |PMT*N - PV - FV| / (PV * N * 0.5)
    //
    // Simple heuristic: i = -(PV + FV + PMT*N) / (N * (PV + FV) * 0.5)
    // Fallback to 0.1 if that produces invalid values.
    let initial_guess = {
        let approx = -(pv + fv + pmt * n) / (n * (pv.abs() + fv.abs()) * 0.5 + 1e-30);
        if approx.is_finite() && approx > -0.9 && approx < 10.0 {
            approx.max(0.0001) // clamp to small positive to avoid zero-rate edge case
        } else {
            0.1_f64
        }
    };

    let mut i = initial_guess;
    let mut converged = false;

    for _ in 0..TVM_MAX_ITERATIONS {
        let (f, fprime) = tvm_f_and_fprime(n, pv, pmt, fv, i, begin);

        // Guard against non-finite values or zero derivative
        if !f.is_finite() || !fprime.is_finite() || fprime.abs() < 1e-20 {
            break;
        }

        let step = f / fprime;
        let i_new = i - step;

        // Clamp to avoid extreme values that cause (1+i)^(-N) to overflow/underflow.
        // Keep periodic rate in (-0.9999, 10.0) = (-99.99%, +1000%) per period.
        // Damp large steps by limiting to 50% change from current i.
        let i_damped = if (i_new - i).abs() > i.abs() * 2.0 + 0.5 {
            i - (if step > 0.0 { 1.0_f64 } else { -1.0_f64 }) * (i.abs() * 0.5 + 0.05)
        } else {
            i_new
        };
        let i_clamped = i_damped.clamp(-0.9999, 10.0);

        if (i_clamped - i).abs() < TVM_CONVERGENCE_THRESHOLD {
            i = i_clamped;
            // Verify actual function residual — damp/clamp can cause false convergence
            // at a boundary; the function value must also be near zero.
            let (f_check, _) = tvm_f_and_fprime(n, pv, pmt, fv, i, begin);
            if f_check.abs() < 1e-4 * (pv.abs() + fv.abs() + 1.0) {
                converged = true;
            }
            break;
        }
        i = i_clamped;
    }

    if !converged {
        // Push last iterate to X as percent, push "NO SOLUTION" message (D-43.13)
        let i_pct = Decimal::from_f64(i * 100.0).ok_or(HpError::Overflow)?;
        let result_hp = HpNum::from(i_pct);
        enter_number(state, result_hp);
        apply_lift_effect(state, LiftEffect::Enable);
        state.print_buffer.push("NO SOLUTION".to_string());
        return Err(HpError::NoRoot);
    }

    // Store result as periodic rate in TvmState
    let i_decimal = Decimal::from_f64(i).ok_or(HpError::Overflow)?;
    let i_hp = HpNum::from(i_decimal);
    tvm_state(state).i = i_hp;

    // Push i * 100 (percent) to X
    let i_pct = Decimal::from_f64(i * 100.0).ok_or(HpError::Overflow)?;
    let result_hp = HpNum::from(i_pct);
    enter_number(state, result_hp);
    apply_lift_effect(state, LiftEffect::Enable);

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
    use rust_decimal::Decimal;

    // Helper: get X register as f64
    fn x_as_f64(state: &CalcState) -> f64 {
        state.stack.x.inner().to_f64().unwrap()
    }

    // Helper: push a value onto X
    fn push_x(state: &mut CalcState, v: f64) {
        state.stack.x = HpNum::from(Decimal::from_f64(v).unwrap());
    }

    // Catches: TvmState default initializes all fields to zero / false
    #[test]
    fn tvm_state_default() {
        let t = TvmState::default();
        // LINT-EXEMPT: Decimal-exact integer equality — HpNum::zero() is Decimal(0), no f64 bridge
        assert_eq!(t.n, HpNum::zero());
        // LINT-EXEMPT: Decimal-exact integer equality — HpNum::zero() is Decimal(0), no f64 bridge
        assert_eq!(t.i, HpNum::zero());
        // LINT-EXEMPT: Decimal-exact integer equality — HpNum::zero() is Decimal(0), no f64 bridge
        assert_eq!(t.pv, HpNum::zero());
        // LINT-EXEMPT: Decimal-exact integer equality — HpNum::zero() is Decimal(0), no f64 bridge
        assert_eq!(t.pmt, HpNum::zero());
        // LINT-EXEMPT: Decimal-exact integer equality — HpNum::zero() is Decimal(0), no f64 bridge
        assert_eq!(t.fv, HpNum::zero());
        assert!(!t.begin_mode);
    }

    // Catches: TvmState.begin_mode defaults to false (END mode per plan)
    #[test]
    fn tvm_state_default_end_mode() {
        let t = TvmState::default();
        assert!(!t.begin_mode, "begin_mode must default to false (END mode)");
    }

    // Catches: TvmState can be cloned (required for CalcState::clone())
    #[test]
    fn tvm_state_clone() {
        let t = TvmState {
            begin_mode: true,
            ..Default::default()
        };
        let t2 = t.clone();
        assert!(t2.begin_mode);
    }

    // Catches: TvmState round-trips through JSON (D-43.11 persistence requirement)
    // Also verifies adv_tvm_state has #[serde(default)] without skip
    #[test]
    fn tvm_state_serde_round_trip() {
        let t = TvmState {
            n: HpNum::from(12_i32),
            pv: HpNum::from(Decimal::from_f64(200_000.0).unwrap()),
            pmt: HpNum::from(Decimal::from_f64(-1199.1).unwrap()),
            fv: HpNum::zero(),
            begin_mode: true,
            ..Default::default()
        };
        let json = serde_json::to_string(&t).unwrap();
        let t2: TvmState = serde_json::from_str(&json).unwrap();
        // LINT-EXEMPT: serde round-trip equality — HpNum fields constructed from exact Decimal literals, no f64 bridge
        assert_eq!(t2.n, t.n);
        // LINT-EXEMPT: serde round-trip equality — HpNum fields constructed from exact Decimal literals, no f64 bridge
        assert_eq!(t2.pv, t.pv);
        // LINT-EXEMPT: serde round-trip equality — HpNum fields constructed from exact Decimal literals, no f64 bridge
        assert_eq!(t2.pmt, t.pmt);
        // LINT-EXEMPT: serde round-trip equality — HpNum fields constructed from exact Decimal literals, no f64 bridge
        assert_eq!(t2.fv, t.fv);
        assert!(t2.begin_mode);
    }

    // Catches: adv_tvm_state on CalcState round-trips (D-43.11 — persistent field)
    #[test]
    fn calc_state_adv_tvm_state_serde_round_trip() {
        let mut state = CalcState::new();
        state.adv_tvm_state = Some(TvmState {
            n: HpNum::from(360_i32),
            i: HpNum::from(Decimal::from_f64(0.5).unwrap()),
            pv: HpNum::from(Decimal::from_f64(200_000.0).unwrap()),
            pmt: HpNum::from(Decimal::from_f64(-1199.1).unwrap()),
            fv: HpNum::zero(),
            begin_mode: false,
        });
        let json = serde_json::to_string(&state).unwrap();
        // Confirm adv_tvm_state appears in the JSON (not skipped)
        assert!(
            json.contains("adv_tvm_state"),
            "adv_tvm_state must be in JSON (not serde(skip))"
        );
        let state2: CalcState = serde_json::from_str(&json).unwrap();
        let tvm2 = state2.adv_tvm_state.unwrap();
        // LINT-EXEMPT: Decimal-exact integer equality — HpNum::from(360_i32) is exact, no f64 bridge
        assert_eq!(tvm2.n, HpNum::from(360_i32));
        assert!(!tvm2.begin_mode);
    }

    // Catches: TVM_MAX_ITERATIONS and TVM_CONVERGENCE_THRESHOLD constants present
    #[test]
    fn tvm_constants_sane() {
        assert_eq!(TVM_MAX_ITERATIONS, 100);
        const { assert!(TVM_CONVERGENCE_THRESHOLD < 1e-8) };
    }

    // Catches: op_adv_tvm opens the modal workflow (not InvalidOp)
    #[test]
    fn adv_tvm_opens_modal() {
        let mut state = CalcState::new();
        let result = op_adv_tvm(&mut state);
        assert!(result.is_ok(), "op_adv_tvm must succeed");
        assert!(
            state.modal_program.is_some(),
            "modal_program must be set after op_adv_tvm"
        );
        assert_eq!(
            state.modal_prompt,
            Some("N=?".to_string()),
            "first TVM prompt must be N=?"
        );
    }

    // Catches: TVM N stores X into tvm.n (ADV-TVM-02)
    #[test]
    fn adv_tvm_n_stores_x() {
        let mut state = CalcState::new();
        push_x(&mut state, 360.0);
        op_adv_tvm_n(&mut state).unwrap();
        let n = state
            .adv_tvm_state
            .as_ref()
            .unwrap()
            .n
            .inner()
            .to_f64()
            .unwrap();
        // LINT-EXEMPT: pure-f64 tolerance check on a value extracted via to_f64() — n is 360.0 (exact int), 1e-9 threshold appropriate
        assert!(
            (n - 360.0).abs() < 1e-9,
            "tvm.n must equal 360 after op_adv_tvm_n with X=360"
        );
    }

    // Catches: TVM PV stores X into tvm.pv (ADV-TVM-03)
    #[test]
    fn adv_tvm_pv_stores_x() {
        let mut state = CalcState::new();
        push_x(&mut state, 200_000.0);
        op_adv_tvm_pv(&mut state).unwrap();
        let pv = state
            .adv_tvm_state
            .as_ref()
            .unwrap()
            .pv
            .inner()
            .to_f64()
            .unwrap();
        // LINT-EXEMPT: pure-f64 tolerance on to_f64() extraction of exact Decimal(200000), no iterative computation
        assert!((pv - 200_000.0).abs() < 1e-9, "tvm.pv must equal 200000");
    }

    // Catches: TVM PMT stores X into tvm.pmt (ADV-TVM-04)
    #[test]
    fn adv_tvm_pmt_stores_x() {
        let mut state = CalcState::new();
        push_x(&mut state, -1000.0);
        op_adv_tvm_pmt(&mut state).unwrap();
        let pmt = state
            .adv_tvm_state
            .as_ref()
            .unwrap()
            .pmt
            .inner()
            .to_f64()
            .unwrap();
        assert!((pmt - (-1000.0)).abs() < 1e-9, "tvm.pmt must equal -1000");
    }

    // Catches: TVM FV stores X into tvm.fv (ADV-TVM-05)
    #[test]
    fn adv_tvm_fv_stores_x() {
        let mut state = CalcState::new();
        push_x(&mut state, 0.0);
        op_adv_tvm_fv(&mut state).unwrap();
        let fv = state
            .adv_tvm_state
            .as_ref()
            .unwrap()
            .fv
            .inner()
            .to_f64()
            .unwrap();
        assert!(fv.abs() < 1e-9, "tvm.fv must equal 0");
    }

    // Catches: N/PV/PMT/FV ops use LiftEffect::Neutral (don't modify lift_enabled)
    #[test]
    fn adv_tvm_register_ops_are_lift_neutral() {
        let mut state = CalcState::new();
        state.stack.lift_enabled = true;
        push_x(&mut state, 1.0);
        op_adv_tvm_n(&mut state).unwrap();
        assert!(
            state.stack.lift_enabled,
            "N op must be lift-neutral (leave lift_enabled=true)"
        );

        state.stack.lift_enabled = false;
        push_x(&mut state, 1.0);
        op_adv_tvm_pv(&mut state).unwrap();
        assert!(
            !state.stack.lift_enabled,
            "PV op must be lift-neutral (leave lift_enabled=false)"
        );
    }

    // Catches: *I converges on standard 30-year mortgage case
    // N=360, PV=200000, PMT=-1199.10, FV=0 → monthly rate ≈ 0.5% (6% APR / 12)
    #[test]
    fn adv_tvm_star_i_mortgage_convergence() {
        let mut state = CalcState::new();
        state.adv_tvm_state = Some(TvmState {
            n: HpNum::from(360_i32),
            i: HpNum::zero(),
            pv: HpNum::from(Decimal::from_f64(200_000.0).unwrap()),
            pmt: HpNum::from(Decimal::from_f64(-1199.10).unwrap()),
            fv: HpNum::zero(),
            begin_mode: false,
        });

        let result = op_adv_tvm_star_i(&mut state);
        assert!(
            result.is_ok(),
            "*I must converge for standard mortgage: {result:?}"
        );

        // X should contain monthly rate in percent ≈ 0.5% (6%/12)
        let i_pct = x_as_f64(&state);
        // LINT-EXEMPT: iterative *I convergence result — 1% tolerance is intentionally coarse for Newton root-find
        assert!(
            (i_pct - 0.5).abs() < 0.01,
            "*I result {i_pct:.6}% must be approximately 0.5% monthly (6% APR/12)"
        );

        // tvm.i must store the periodic rate (not percent)
        let tvm = state.adv_tvm_state.as_ref().unwrap();
        let i_stored = tvm.i.inner().to_f64().unwrap();
        // LINT-EXEMPT: iterative *I convergence result — 1e-4 tolerance covers TVM_CONVERGENCE_THRESHOLD (1e-9) + f64 extraction
        assert!(
            (i_stored - 0.005).abs() < 0.0001,
            "tvm.i must store periodic rate ~0.005, got {i_stored:.8}"
        );
    }

    // Catches: *I converges on trivial 1-period case
    // N=1, PV=-100, PMT=0, FV=110 → I = 10%
    #[test]
    fn adv_tvm_star_i_trivial_one_period() {
        let mut state = CalcState::new();
        state.adv_tvm_state = Some(TvmState {
            n: HpNum::from(1_i32),
            i: HpNum::zero(),
            pv: HpNum::from(Decimal::from_f64(-100.0).unwrap()),
            pmt: HpNum::zero(),
            fv: HpNum::from(Decimal::from_f64(110.0).unwrap()),
            begin_mode: false,
        });

        let result = op_adv_tvm_star_i(&mut state);
        assert!(
            result.is_ok(),
            "*I must converge for N=1,PV=-100,FV=110: {result:?}"
        );

        let i_pct = x_as_f64(&state);
        // LINT-EXEMPT: iterative *I convergence result for trivial 1-period case — 1e-3 is tight given TVM_CONVERGENCE_THRESHOLD
        assert!(
            (i_pct - 10.0).abs() < 0.001,
            "*I result {i_pct:.6}% must equal 10% for trivial 1-period case"
        );
    }

    // Catches: *I returns HpError::Domain when N=0
    #[test]
    fn adv_tvm_star_i_domain_error_when_n_zero() {
        let mut state = CalcState::new();
        state.adv_tvm_state = Some(TvmState {
            n: HpNum::zero(), // N=0
            i: HpNum::zero(),
            pv: HpNum::from(Decimal::from_f64(100.0).unwrap()),
            pmt: HpNum::from(Decimal::from_f64(-10.0).unwrap()),
            fv: HpNum::zero(),
            begin_mode: false,
        });
        let result = op_adv_tvm_star_i(&mut state);
        assert!(
            matches!(result, Err(HpError::Domain)),
            "*I with N=0 must return HpError::Domain, got {result:?}"
        );
    }

    // Catches: *I non-convergence returns HpError::NoRoot and pushes "NO SOLUTION"
    // Force non-convergence by using impossible parameters (all-positive flows, no root)
    #[test]
    fn adv_tvm_star_i_no_root_on_impossible_params() {
        let mut state = CalcState::new();
        // PV>0, PMT>0, FV>0 — can't have all positive flows sum to zero at any positive rate
        // This is financially impossible — Newton diverges
        state.adv_tvm_state = Some(TvmState {
            n: HpNum::from(10_i32),
            i: HpNum::zero(),
            pv: HpNum::from(Decimal::from_f64(100.0).unwrap()),
            pmt: HpNum::from(Decimal::from_f64(100.0).unwrap()),
            fv: HpNum::from(Decimal::from_f64(100.0).unwrap()),
            begin_mode: false,
        });
        let result = op_adv_tvm_star_i(&mut state);
        assert!(
            matches!(result, Err(HpError::NoRoot)),
            "*I with impossible parameters must return HpError::NoRoot, got {result:?}"
        );
        assert!(
            state.print_buffer.contains(&"NO SOLUTION".to_string()),
            "print_buffer must contain 'NO SOLUTION' on non-convergence"
        );
    }

    // Catches: *I result is pushed to X with LiftEffect::Enable
    #[test]
    fn adv_tvm_star_i_lifts_on_result() {
        let mut state = CalcState::new();
        state.stack.lift_enabled = false; // ensure it gets set to true
        state.adv_tvm_state = Some(TvmState {
            n: HpNum::from(1_i32),
            i: HpNum::zero(),
            pv: HpNum::from(Decimal::from_f64(-100.0).unwrap()),
            pmt: HpNum::zero(),
            fv: HpNum::from(Decimal::from_f64(110.0).unwrap()),
            begin_mode: false,
        });
        op_adv_tvm_star_i(&mut state).unwrap();
        assert!(
            state.stack.lift_enabled,
            "*I must enable lift after successful convergence"
        );
    }

    // Catches: tvm_f_and_fprime zero-rate limit (i→0)
    #[test]
    fn tvm_zero_rate_limit() {
        // At i=0: f = PV + PMT*N + FV
        // Example: PV=100, PMT=-10, N=10, FV=0 → f = 100 - 100 + 0 = 0 ✓
        let (f, _) = tvm_f_and_fprime(10.0, 100.0, -10.0, 0.0, 0.0, false);
        assert!(f.abs() < 1e-9, "TVM at i=0 must satisfy zero-rate limit");
    }

    // Catches: *I initializes TvmState if None before reading N (no panic)
    #[test]
    fn adv_tvm_star_i_with_no_tvm_state_returns_domain() {
        let mut state = CalcState::new();
        // adv_tvm_state is None → after tvm_state(state) it's initialized to default
        // default N=0 → returns Domain
        assert!(state.adv_tvm_state.is_none());
        let result = op_adv_tvm_star_i(&mut state);
        // default N=0 → Domain (not panic)
        assert!(
            matches!(result, Err(HpError::Domain)),
            "*I on uninitialized state (N=0 default) must return Domain, got {result:?}"
        );
    }
}
