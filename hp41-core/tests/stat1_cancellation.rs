// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Integration test: Stat 1 Pac iterative-quantile cancellation contract
//! (SPEC.md Req. 34 / D-33.5 / Pitfall 11).
//!
//! ## What this asserts
//!
//! ΣNORMD inverse (Plan 33-03 `op_sigma_normd_eval_inverse`) and the
//! display-mode-tied tolerance helper `quantile_threshold` form the
//! `Op`-level contract anchor for SPEC.md Req. 34:
//!
//! - Setting `cancel_requested = true` before invoking ΣNORMD inverse
//!   MUST return `Err(HpError::Canceled)` within < 1 iteration (Test 1).
//! - `quantile_threshold(DisplayMode::Fix(6))` MUST yield `1e-7` —
//!   the canonical "FIX-6 yields 7-decimal precision band" SPEC
//!   acceptance test (Test 2).
//!
//! ## Why an integration test (not just an inline unit test)
//!
//! The cancellation contract spans three crates of abstraction:
//!
//! 1. `HpError::Canceled` is `pub` in `crate::error`.
//! 2. `op_sigma_normd_eval_inverse` is `pub` in `crate::ops::stat1::normd`.
//! 3. `state.cancel_requested` is `pub` on `CalcState` — set via
//!    `AtomicBool::store(true, Ordering::Relaxed)`.
//!
//! An external `tests/` integration test asserts the contract from the
//! perspective of an out-of-crate caller (mirrors the v3.0
//! `cancel_flag_reset_on_open.rs` precedent for Math Pac I cancel
//! invariants — Plan 31-02 Task 3).
//!
//! ## Coverage strategy (D-27.1)
//!
//! Every test carries a `// Catches:` comment naming the regression mode
//! it specifically guards against. See sibling `cancel_flag_reset_on_open.rs`
//! for the v3.0 precedent.

#![allow(clippy::unwrap_used)]

use std::sync::atomic::Ordering;

use hp41_core::error::HpError;
use hp41_core::num::HpNum;
use hp41_core::ops::stat1::distributions::quantile_threshold;
use hp41_core::ops::stat1::normd::op_sigma_normd_eval_inverse;
use hp41_core::state::{CalcState, DisplayMode};

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

// ── Test 1: ΣNORMD inverse halts on cancel_requested within < 1 iter ───────

/// Catches: ΣNORMD inverse failing to check `cancel_requested` BEFORE the
/// first Newton iteration body — would result in the bare Acklam value
/// being pushed even when the GUI request_cancel hook has set the flag.
/// SPEC.md Req. 34 acceptance criterion (Plan 33-03 Task 4 deliverable).
#[test]
fn cancel_requested_kills_normd_inverse_within_one_iter() {
    let mut state = CalcState::new();
    // Push p = 0.025 onto stack X (a probability the inverse would
    // happily compute the Acklam start for and exit in < 1 µs).
    state.stack.x = HpNum::from(Decimal::from_f64(0.025).unwrap());
    // Pre-set cancel BEFORE invoking the inverse. SPEC.md Req. 34:
    // "synthetic test sets cancel_requested = true before invoking
    // ΣNORMD inverse and asserts the call returns Err(HpError::Canceled)
    // within < 1 iteration".
    state.cancel_requested.store(true, Ordering::Relaxed);
    let result = op_sigma_normd_eval_inverse(&mut state);
    assert_eq!(
        result,
        Err(HpError::Canceled),
        "ΣNORMD inverse must surface Canceled when cancel_requested == true"
    );
    // Side-effect invariant: stack X must NOT have been mutated to a
    // pseudo-result during the canceled call — the caller's p stays
    // visible so the user can inspect it after cancellation.
    let x_back = state.stack.x.inner();
    assert!(
        x_back > Decimal::ZERO,
        "stack X must be preserved across a canceled call; got {x_back}"
    );
}

// ── Test 2: quantile_threshold honors Fix(6) ↦ 1e-7 contract ──────────────

/// Catches: `quantile_threshold` regression vs SPEC.md Req. 34:
/// "second test sets FIX 6 display mode and verifies converged-tolerance
/// is `1e-7`". This is the canonical sentinel — if the formula drifts to
/// `5 × 10^(-(decimals+1))` (the integ_threshold factor of 5), this
/// test fires immediately.
#[test]
fn display_mode_fix_6_yields_1e_7_threshold() {
    let tol = quantile_threshold(DisplayMode::Fix(6));
    // Strict relative comparison — the value should be a bit-exact
    // power of 10 (10.0_f64.powi(-7) = 1e-7 with no rounding noise).
    // LINT-EXEMPT: testing the VALUE of the tolerance function itself (f64 power-of-10 exact); not a computed numerical result
    assert!(
        (tol - 1e-7).abs() < 1e-20,
        "quantile_threshold(Fix(6)) must equal 1e-7 exactly; got {tol}"
    );
}

// ── Test 3: quantile_threshold Fix(4) sanity (cross-mode regression) ──────

/// Catches: per-mode drift — Fix(4) must yield 1e-5, NOT 5e-5 (the
/// `integ_threshold` value). Pitfall-2 detection: tests should run the
/// quantile_threshold in BOTH Fix(4) and Fix(6) and assert DIFFERENT
/// precision bands.
#[test]
fn display_mode_fix_4_yields_1e_5_threshold() {
    let tol = quantile_threshold(DisplayMode::Fix(4));
    // LINT-EXEMPT: testing the VALUE of the tolerance function itself (f64 power-of-10 exact); not a computed numerical result
    assert!(
        (tol - 1e-5).abs() < 1e-18,
        "quantile_threshold(Fix(4)) must equal 1e-5 exactly; got {tol}"
    );
}
