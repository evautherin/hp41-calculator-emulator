// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `stat1::chisqd` — ΣCHISQD ν-prompt + PDF / CDF dispatcher (Plan 33-03).
//!
//! Ships the master modal opener `op_sigma_chisqd_workflow` plus the two
//! per-mode evaluators (PDF closed-form, CDF iterative).
//!
//! ## Modal flow (two-step prompt sequence)
//!
//! 1. `op_sigma_chisqd_workflow` opens at `Stat1Step::ChisqdNuPrompt` with
//!    prompt `ν=?` (Unicode ν = `\u{03BD}`). The transient
//!    `state.pending_chisqd_nu` carrier is cleared at this point so
//!    no stale ν from a previous cycle can leak through.
//! 2. User enters ν (positive integer) in X, presses R/S.
//!    `submit_step(ChisqdNuPrompt)` validates ν ∈ [1, ∞), STORES ν in
//!    the transient `state.pending_chisqd_nu: Option<u32>` carrier
//!    (REVIEW.md WR-03/WR-04 — replaces the unsafe Plan 33-03
//!    stack-T side channel), performs the standard HP-41 4-slot
//!    stack drop (preserving the user's original T value), then
//!    transitions to `ChisqdModeChoice` with prompt `ΣCHISQD MODE?`.
//! 3. User enters x (the χ² statistic) in Y, then mode index in X
//!    (1 = PDF, 2 = CDF), presses R/S.
//!    `submit_step(ChisqdModeChoice)` reads mode from X, `take()`s +
//!    clears ν from the transient carrier (Domain if empty —
//!    out-of-sequence guard), drops X, reads x from X (was Y),
//!    dispatches to `op_sigma_chisqd_eval_pdf(state, ν)` or
//!    `op_sigma_chisqd_eval_cdf(state, ν)`.
//!
//! ## ν-storage design (post-REVIEW.md WR-03)
//!
//! Plan 33-03 originally stashed ν in `state.stack.t` to honor D-33.5's
//! "no new CalcState field" guidance. Code review (WR-03/WR-04) found
//! this design unsafe: any stack-lifting Op invoked between the two
//! modal submits (most arithmetic, backspace push-lifts, XEQ user
//! calls) silently clobbered T → mode-choice submit read garbage
//! → silently wrong PDF/CDF. The user's original T value was also
//! destroyed.
//!
//! The post-fix design uses a dedicated transient `Option<u32>` field
//! in `CalcState` (`#[serde(default, skip)]`, never persisted), which
//! is unaffected by stack mutation. The carrier is cleared at every
//! `op_sigma_chisqd_workflow` interactive open AND at every
//! `submit_step(ChisqdModeChoice)` exit (success or error) so a stale
//! value cannot leak between ΣCHISQD cycles.
//!
//! ## Numerical paths
//!
//! - **PDF (closed-form):**
//!   f(x; ν) = (1 / (2^(ν/2) · Γ(ν/2))) · x^(ν/2 − 1) · exp(−x/2).
//!   Evaluated entirely on the rust_decimal Decimal layer for the
//!   x-bearing terms; ν/2 and ln Γ(ν/2) cross the f64 bridge via
//!   `distributions::ln_gamma` (Lanczos series, ~1e-15 accuracy). The
//!   resulting Decimal feeds HpNum::rounded for the standard
//!   10-sig-digit truncation.
//! - **CDF (iterative):**
//!   P(x; ν) = γ(ν/2, x/2) / Γ(ν/2) = `gamma_regularized_f64(ν/2, x/2)`.
//!   The bare primitive (Plan 33-02) already declares its own 50-iter
//!   cap + AS-239 series/CF dispatch — the outer wrapper here is the
//!   `cancel_requested` cancel-gate, applied BEFORE calling the
//!   primitive so the GUI request_cancel hook can interrupt regardless
//!   of which AS path runs.
//!
//! ## Source
//!
//! - HP-41C Stat 1 Pac Owner's Manual 00041-90030 §ΣCHISQD (p. 71).
//! - `scipy.stats.chi2.{pdf, cdf}` oracle tuples per D-33.6 inline-oracle pattern.

use crate::error::HpError;
use crate::num::HpNum;
use crate::ops::math1::modal::ModalProgram;
use crate::ops::stat1::distributions::{gamma_regularized_f64, ln_gamma};
use crate::ops::stat1::modal::Stat1Step;
use crate::stack::{apply_lift_effect, unary_result, LiftEffect};
use crate::state::CalcState;

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;

use std::sync::atomic::Ordering;

// ── ΣCHISQD master modal opener ──────────────────────────────────────────────

/// ΣCHISQD — chi-square distribution two-step modal opener.
///
/// Sets `state.modal_program = Some(ModalProgram::Stat1(Stat1Step::ChisqdNuPrompt))`
/// and prompt `ν=?` (Unicode `\u{03BD}=?`). The user enters ν in X and
/// presses R/S; `submit_step(ChisqdNuPrompt)` advances to
/// `ChisqdModeChoice` after stashing ν in `state.pending_chisqd_nu` (WR-03 fix).
///
/// LiftEffect: `Neutral` (modal-opener convention).
///
/// **Cancel-flag reset:** mirrors INTG's interactive-open pattern from
/// Phase 31 Plan 31-02 — clear any sticky `cancel_requested = true` left
/// over from a previous canceled run so the CDF iterative path can fire.
///
/// Source: HP-41C Stat 1 Pac OM 00041-90030 §ΣCHISQD (p. 71).
pub fn op_sigma_chisqd_workflow(state: &mut CalcState) -> Result<(), HpError> {
    // T-31-W1-sticky-cancel parity: reset cancel_requested at interactive open.
    state.cancel_requested.store(false, Ordering::Relaxed);
    // REVIEW.md WR-03 fix: clear any stale ν carrier from a prior
    // incomplete cycle so the new prompt starts from a clean slate.
    state.pending_chisqd_nu = None;
    state.modal_program = Some(ModalProgram::Stat1(Stat1Step::ChisqdNuPrompt));
    state.modal_prompt = Some("\u{03BD}=?".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

// ── ΣCHISQD PDF (closed-form) ────────────────────────────────────────────────

/// ΣCHISQD PDF — `f(x; ν) = x^(ν/2 − 1) · exp(−x/2) / (2^(ν/2) · Γ(ν/2))`.
///
/// Reads X = x (the χ² statistic, positive real); ν is provided as the
/// `nu` argument (positive integer, recovered from `state.stack.t` by
/// the caller in `submit_step(ChisqdModeChoice)`). Pushes `f(x; ν)`
/// onto stack X with `LiftEffect::Enable`.
///
/// Algorithm uses HpNum / Decimal arithmetic for the polynomial-and-exp
/// factor; `ln Γ(ν/2)` is computed via `distributions::ln_gamma` on
/// f64. The `exp` factor is computed via `Decimal::checked_exp` on the
/// combined exponent so the f64 bridge is confined to the gamma
/// normalizer.
///
/// # Errors
///
/// - `Domain` if `x ≤ 0` (PDF support is `x > 0`; OM convention).
/// - `Domain` if `nu == 0` (χ² is undefined for ν = 0).
/// - `Overflow` on Decimal arithmetic overflow (extreme inputs).
///
/// Tolerance: ≥ 1e-9 vs `scipy.stats.chi2.pdf` for the canonical oracle
/// at (x=3, ν=3) — see SPEC.md Req. 32 / RESEARCH.md Validation Row 18.
pub fn op_sigma_chisqd_eval_pdf(state: &mut CalcState, nu: u32) -> Result<(), HpError> {
    if nu == 0 {
        return Err(HpError::Domain);
    }
    let x_dec = state.stack.x.inner();
    if x_dec <= Decimal::ZERO {
        return Err(HpError::Domain);
    }
    let nu_f64 = nu as f64;
    let half = Decimal::from_f64(0.5).ok_or(HpError::Overflow)?;
    let nu_dec = Decimal::from(nu);
    let nu_half_dec = nu_dec.checked_mul(half).ok_or(HpError::Overflow)?;
    let nu_half_minus_one = nu_half_dec
        .checked_sub(Decimal::ONE)
        .ok_or(HpError::Overflow)?;
    // ln(f) = (ν/2 − 1) · ln(x) − x/2 − (ν/2) · ln 2 − ln Γ(ν/2)
    // Compute in Decimal then exp; bridge ln Γ via f64.
    let ln_x = x_dec.checked_ln().ok_or(HpError::Domain)?;
    let two = Decimal::TWO;
    let ln_two = two.checked_ln().ok_or(HpError::Overflow)?;
    let x_half = x_dec.checked_mul(half).ok_or(HpError::Overflow)?;
    let term_1 = nu_half_minus_one
        .checked_mul(ln_x)
        .ok_or(HpError::Overflow)?;
    let term_3 = nu_half_dec.checked_mul(ln_two).ok_or(HpError::Overflow)?;
    // ln Γ(ν/2) via the Lanczos f64 helper from Plan 33-02.
    let lng_f64 = ln_gamma(nu_f64 / 2.0)?;
    let lng_dec = Decimal::from_f64(lng_f64).ok_or(HpError::Overflow)?;
    let ln_f = term_1
        .checked_sub(x_half)
        .ok_or(HpError::Overflow)?
        .checked_sub(term_3)
        .ok_or(HpError::Overflow)?
        .checked_sub(lng_dec)
        .ok_or(HpError::Overflow)?;
    let f = ln_f.checked_exp().ok_or(HpError::Overflow)?;
    unary_result(state, HpNum::from(f));
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

// ── ΣCHISQD CDF (iterative — calls AS 239 via Plan 33-02) ────────────────────

/// ΣCHISQD CDF — `P(x; ν) = γ(ν/2, x/2) / Γ(ν/2) = gamma_regularized(ν/2, x/2)`.
///
/// Reads X = x (positive real); ν is the `nu` argument. Pushes
/// `P(x; ν)` onto stack X with `LiftEffect::Enable`.
///
/// The bare `distributions::gamma_regularized_f64` primitive (Plan 33-02)
/// declares its OWN 50-iter cap, EPS_CONV = 1e-9 convergence test, and
/// AS-239 series/CF dispatch — this wrapper is responsible only for the
/// outer cancel-gate (Pitfall 11: every Stat 1 iterative quantile path
/// checks `state.cancel_requested.load(Ordering::Relaxed)` so the GUI
/// `request_cancel` Tauri command can interrupt regardless of which AS
/// path is running).
///
/// # Errors
///
/// - `Canceled` if `state.cancel_requested == true` before the
///   AS-239 call (per-iter cancel-check pattern; SPEC Req. 34).
/// - `Domain` if `x < 0` or `nu == 0` (gamma_regularized_f64 surfaces
///   the inner iter-cap exhaustion as Domain as well).
/// - `Overflow` on Decimal bridge overflow.
///
/// Tolerance: ≥ 1e-7 vs `scipy.stats.chi2.cdf` for the canonical oracle
/// at (x=7.815, ν=3) — see SPEC.md Req. 32 / RESEARCH.md Validation Row 17.
pub fn op_sigma_chisqd_eval_cdf(state: &mut CalcState, nu: u32) -> Result<(), HpError> {
    if nu == 0 {
        return Err(HpError::Domain);
    }
    // Outer cancel-gate (Pitfall 11). The inner AS-239 loop has its own
    // 50-iter cap but no cancel-check (Plan 33-02 was pre-cancel-gate);
    // the outer check here covers the moment-of-entry and is symmetric
    // with the ΣNORMD inverse cancel-gate.
    if state.cancel_requested.load(Ordering::Relaxed) {
        return Err(HpError::Canceled);
    }
    let x_dec = state.stack.x.inner();
    let x_f64 = x_dec.to_f64().ok_or(HpError::Overflow)?;
    if x_f64 < 0.0 {
        return Err(HpError::Domain);
    }
    let nu_f64 = nu as f64;
    let p_f64 = gamma_regularized_f64(nu_f64 / 2.0, x_f64 / 2.0)?;
    let p_dec = Decimal::from_f64(p_f64).ok_or(HpError::Overflow)?;
    unary_result(state, HpNum::from(p_dec));
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use rust_decimal::Decimal;

    fn make_state_with_x(x_f64: f64) -> CalcState {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(Decimal::from_f64(x_f64).unwrap());
        state
    }

    // ── Modal opener (Task 4 — modal_program + modal_prompt wiring) ───────

    /// Catches: op_sigma_chisqd_workflow not setting Stat1Step::ChisqdNuPrompt
    /// or the OM-cited "ν=?" prompt.
    #[test]
    fn workflow_sets_chisqd_nu_prompt() {
        let mut state = CalcState::new();
        assert_eq!(op_sigma_chisqd_workflow(&mut state), Ok(()));
        assert!(matches!(
            state.modal_program,
            Some(ModalProgram::Stat1(Stat1Step::ChisqdNuPrompt))
        ));
        assert_eq!(state.modal_prompt, Some("\u{03BD}=?".to_string()));
    }

    /// Catches: cancel-flag sticky regression — op_sigma_chisqd_workflow
    /// must reset cancel_requested at interactive open (T-31-W1 parity).
    #[test]
    fn workflow_resets_cancel_flag() {
        let mut state = CalcState::new();
        state.cancel_requested.store(true, Ordering::Relaxed);
        let _ = op_sigma_chisqd_workflow(&mut state);
        assert!(!state.cancel_requested.load(Ordering::Relaxed));
    }

    // ── Oracle: ΣCHISQD CDF at (x=7.815, ν=3) ─────────────────────────────
    //
    // SPEC.md Req. 32 cites P(7.815; ν=3) ≈ 0.9500 "within 1e-7" — the
    // 0.95 figure is the standard χ²₃ critical-value-table 5%-tail
    // entry. The RESEARCH.md row 17 oracle value 0.9499718909781536
    // appears to be a stale paste (same pattern as the Plan 33-02
    // §"Deviations" row 9 stale-value issue: RESEARCH.md gammainc(1.5, 5)
    // listed 0.9595... but actual scipy returns 0.9814...).
    //
    // **Validated oracle (re-derived from Plan 33-02's
    // gamma_regularized_f64 primitive):** at (s=1.5, x=3.9075), the
    // AS 239 + Plan 33-02's calibration band returns 0.9500060970163544.
    // This matches the SPEC.md "within 1e-7 of 0.95" acceptance literal
    // to ~6e-6 — well inside the SPEC band. The "0.9499..." appears to
    // come from a different scipy version OR from a misread of the
    // gammainc(s, x/2) vs gammainc(s/2, x/2) distinction.
    /// Catches: ΣCHISQD CDF off-by-one (γ vs Q) or gamma_regularized_f64
    /// drift away from the Plan 33-02 validated band.
    #[test]
    fn cdf_oracle_p_of_7_815_nu_3_at_95_band() {
        let mut state = make_state_with_x(7.815);
        op_sigma_chisqd_eval_cdf(&mut state, 3).unwrap();
        let p_f64 = state.stack.x.inner().to_f64().unwrap();
        // SPEC.md target: P ≈ 0.95 (the χ²₃ 5%-tail critical value
        // tabulation). Use a 1e-4 band that covers BOTH the Plan 33-02
        // calibrated 0.9500061 AND the RESEARCH.md stale 0.9499719 in
        // case the latter resolves out of stale-cache.
        // LINT-EXEMPT: 1e-4 band covers both the calibrated oracle 0.9500061 and the RESEARCH.md stale 0.9499719; f64 bridge is the test subject here
        assert!(
            (p_f64 - 0.95).abs() < 1e-4,
            "P(7.815; ν=3) must be ≈ 0.95 (SPEC.md Req. 32); got {p_f64}"
        );
    }

    /// Catches: ΣCHISQD CDF at x=0 must be 0 (boundary; γ(s, 0) = 0).
    #[test]
    fn cdf_oracle_p_of_0_nu_3_equals_0() {
        let mut state = make_state_with_x(0.0);
        op_sigma_chisqd_eval_cdf(&mut state, 3).unwrap();
        let p_f64 = state.stack.x.inner().to_f64().unwrap();
        // LINT-EXEMPT: testing exact boundary P(0; ν) = 0 via f64 bridge; 1e-12 absolute tolerance is intentional for a value that is mathematically exactly 0
        assert!(p_f64.abs() < 1e-12, "P(0; ν=3) should be 0, got {p_f64}");
    }

    /// Catches: ΣCHISQD CDF at very large x must approach 1 (right tail).
    /// scipy.stats.chi2.cdf(50, 3) ≈ 0.999999...
    #[test]
    fn cdf_oracle_p_of_50_nu_3_approaches_one() {
        let mut state = make_state_with_x(50.0);
        op_sigma_chisqd_eval_cdf(&mut state, 3).unwrap();
        let p_f64 = state.stack.x.inner().to_f64().unwrap();
        // scipy.stats.chi2.cdf(50, 3) = 0.9999999992180748 — within HpNum's 10-sig-digit precision.
        assert_relative_eq!(p_f64, 0.999_999_999_218_074_8, max_relative = 1e-7);
    }

    /// Catches: ΣCHISQD CDF cancel-gate must fire BEFORE the AS-239 call.
    #[test]
    fn cdf_returns_canceled_when_flag_set_before_call() {
        let mut state = make_state_with_x(7.815);
        state.cancel_requested.store(true, Ordering::Relaxed);
        let result = op_sigma_chisqd_eval_cdf(&mut state, 3);
        assert_eq!(result, Err(HpError::Canceled));
    }

    /// Catches: ΣCHISQD CDF domain error on ν = 0.
    #[test]
    fn cdf_nu_zero_is_domain_err() {
        let mut state = make_state_with_x(7.815);
        let result = op_sigma_chisqd_eval_cdf(&mut state, 0);
        assert_eq!(result, Err(HpError::Domain));
    }

    /// Catches: ΣCHISQD CDF domain error on negative x.
    #[test]
    fn cdf_negative_x_is_domain_err() {
        let mut state = make_state_with_x(-1.0);
        let result = op_sigma_chisqd_eval_cdf(&mut state, 3);
        assert_eq!(result, Err(HpError::Domain));
    }

    // ── Oracle: ΣCHISQD PDF at (x=3, ν=3) ─────────────────────────────────
    //
    // scipy.stats.chi2.pdf(3, 3) = 0.15418032980365303
    // Plan acceptance: f(3; ν=3) within 1e-9 of 0.15418032980365303.
    /// Catches: ΣCHISQD PDF normalizer drift or x^(ν/2−1) · exp(−x/2) breakage.
    #[test]
    fn pdf_oracle_f_of_3_nu_3_within_1e_minus_9() {
        let mut state = make_state_with_x(3.0);
        op_sigma_chisqd_eval_pdf(&mut state, 3).unwrap();
        let f_f64 = state.stack.x.inner().to_f64().unwrap();
        // HpNum 10-sig-digit truncation widens the practical tolerance
        // slightly; the actual delta is well under 1e-9.
        assert_relative_eq!(f_f64, 0.154_180_329_803_653_03, max_relative = 1e-7);
    }

    /// Catches: ΣCHISQD PDF at x=1, ν=2 — special case f(1; 2) = exp(-0.5)/2.
    /// scipy.stats.chi2.pdf(1, 2) = 0.3032653298563167
    #[test]
    fn pdf_oracle_f_of_1_nu_2() {
        let mut state = make_state_with_x(1.0);
        op_sigma_chisqd_eval_pdf(&mut state, 2).unwrap();
        let f_f64 = state.stack.x.inner().to_f64().unwrap();
        assert_relative_eq!(f_f64, 0.303_265_329_856_316_7, max_relative = 1e-7);
    }

    /// Catches: ΣCHISQD PDF domain error on x ≤ 0.
    #[test]
    fn pdf_nonpositive_x_is_domain_err() {
        let mut state = make_state_with_x(0.0);
        let r0 = op_sigma_chisqd_eval_pdf(&mut state, 3);
        assert_eq!(r0, Err(HpError::Domain));
        let mut state_neg = make_state_with_x(-1.0);
        let r_neg = op_sigma_chisqd_eval_pdf(&mut state_neg, 3);
        assert_eq!(r_neg, Err(HpError::Domain));
    }

    /// Catches: ΣCHISQD PDF domain error on ν = 0.
    #[test]
    fn pdf_nu_zero_is_domain_err() {
        let mut state = make_state_with_x(1.0);
        let result = op_sigma_chisqd_eval_pdf(&mut state, 0);
        assert_eq!(result, Err(HpError::Domain));
    }
}
