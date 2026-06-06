// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Phase 63 — PSE / VIEW / AVIEW yield-engine integration tests.
//!
//! Validates that Op::Pse, Op::View, and Op::AView set `pending_yield` (kind,
//! text, resume_ms) and break `run_loop` without writing `display_override`.
//!
//! All scenarios use deterministic programs; `resume_ms` is asserted as DATA
//! (never slept in core tests — the frontend owns the wait).
//!
//! Tests marked "// GREEN after 63-02" require the run_loop PSE/VIEW/AVIEW
//! yield arms in program.rs (plan 63-02) and will fail RED until that plan
//! executes.
#![allow(clippy::unwrap_used)]

use hp41_core::num::HpValue;
use hp41_core::ops::Op;
use hp41_core::state::{CalcState, YieldKind, PSE_RESUME_MS};
use hp41_core::HpNum;
use rust_decimal::Decimal;
use std::str::FromStr;

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Load a program into `state.program` from a flat slice of ops.
fn load_program(state: &mut CalcState, ops: Vec<Op>) {
    state.program = ops;
}

// ── Scenario: pse_mid_run_breaks_and_records_resume_ms ───────────────────────
//
// GREEN after 63-02 — requires run_loop PSE yield arm.
//
// A program with PSE mid-run must:
//   1. Break run_loop (run_program returns, is_running becomes false).
//   2. Set pending_yield with kind=Pse, text=format_hpnum(X), resume_ms=PSE_RESUME_MS.
//   3. NOT write display_override (D-04: DISP-01 stays deferred).
//   4. Leave pc pointing at the step AFTER PSE (for resume).
#[test]
fn pse_mid_run_breaks_and_records_resume_ms() {
    let mut state = CalcState::new();

    // Push a known value on X: 3.14
    let val = Decimal::from_str("3.14").unwrap();
    state.stack.x = HpNum::rounded(val);
    state.stack.lift_enabled = true;

    load_program(
        &mut state,
        vec![
            Op::Lbl("P".to_string()),
            Op::Pse, // <- yield point
            Op::Sto(0), // <- step AFTER PSE; should NOT have run yet
            Op::Rtn,
        ],
    );

    // run_program should return after PSE breaks the loop
    hp41_core::ops::program::run_program(&mut state, "P").unwrap();

    // pending_yield must be set
    let py = state
        .pending_yield
        .as_ref()
        .expect("pending_yield must be Some after PSE mid-run");

    assert_eq!(
        py.resume_ms, PSE_RESUME_MS,
        "resume_ms must be PSE_RESUME_MS ({PSE_RESUME_MS})"
    );
    assert!(
        matches!(py.kind, YieldKind::Pse),
        "yield kind must be Pse"
    );
    // text must be the formatted X value (format_hpnum(3.14, Fix(4)))
    assert!(
        !py.text.is_empty(),
        "pending_yield.text must not be empty"
    );
    assert!(
        py.text.contains("3.14") || py.text.contains("3.1400"),
        "pending_yield.text must contain the formatted X value; got: {:?}",
        py.text
    );

    // display_override must NOT be written (D-04)
    assert!(
        state.display_override.is_none(),
        "display_override must remain None after PSE yield (D-04)"
    );

    // STO 0 must NOT have run (program broke at PSE)
    assert_eq!(
        state.regs[0].inner(),
        Decimal::from(0),
        "STO 0 must not have executed (program broke at PSE)"
    );
}

// ── Scenario: pse_resume_continues_to_next_step ──────────────────────────────
//
// GREEN after 63-02 — requires run_loop PSE yield arm + resume_program clear.
//
// After resume_program:
//   - pending_yield is cleared.
//   - Execution continues at the step after PSE.
//   - The program completes normally.
#[test]
fn pse_resume_continues_to_next_step() {
    let mut state = CalcState::new();

    state.stack.x = HpNum::rounded(Decimal::from(7));
    state.stack.lift_enabled = true;

    load_program(
        &mut state,
        vec![
            Op::Lbl("R".to_string()),
            Op::Pse, // yield here
            Op::PushNum(HpNum::rounded(Decimal::from(42))),
            Op::Sto(3), // proves continuation ran
            Op::Rtn,
        ],
    );

    // First run — breaks at PSE
    hp41_core::ops::program::run_program(&mut state, "R").unwrap();
    assert!(state.pending_yield.is_some(), "must have pending_yield after first run");

    // Resume — must continue from step after PSE
    hp41_core::ops::program::resume_program(&mut state).unwrap();

    // pending_yield must be cleared by resume_program
    assert!(
        state.pending_yield.is_none(),
        "pending_yield must be None after resume_program"
    );

    // Step after PSE (STO 3 = 42) must have run
    assert_eq!(
        state.regs[3].inner(),
        Decimal::from(42),
        "STO 3 must have run after resume (continuation from after-PSE step)"
    );
    assert!(!state.is_running, "is_running must be false after completion");
}

// ── Scenario: view_mid_run_captures_formatted_register_into_yield ────────────
//
// GREEN after 63-02 — requires run_loop VIEW yield arm.
//
// VIEW reg mid-program must:
//   1. Break run_loop.
//   2. Set pending_yield with kind=View, text=format_hpnum(regs[reg]), resume_ms=PSE_RESUME_MS.
//   3. NOT write display_override (D-04).
#[test]
fn view_mid_run_captures_formatted_register_into_yield() {
    let mut state = CalcState::new();

    // Set reg[5] = 12.5 directly
    let val = Decimal::from_str("12.5").unwrap();
    state.regs[5] = hp41_core::num::HpValue::Numeric(HpNum::rounded(val));

    load_program(
        &mut state,
        vec![
            Op::Lbl("V".to_string()),
            Op::View(5), // VIEW reg 5 — yield point
            Op::Sto(0),  // must NOT run
            Op::Rtn,
        ],
    );

    hp41_core::ops::program::run_program(&mut state, "V").unwrap();

    let py = state
        .pending_yield
        .as_ref()
        .expect("pending_yield must be Some after VIEW mid-run");

    assert!(
        matches!(py.kind, YieldKind::View),
        "yield kind must be View"
    );
    assert_eq!(
        py.resume_ms, PSE_RESUME_MS,
        "VIEW resume_ms must equal PSE_RESUME_MS"
    );
    // text must be the formatted reg[5] value
    assert!(
        py.text.contains("12.5") || py.text.contains("12.5000"),
        "pending_yield.text must contain formatted reg[5] value; got: {:?}",
        py.text
    );

    // display_override must NOT be written (D-04)
    assert!(
        state.display_override.is_none(),
        "display_override must remain None after VIEW yield (D-04)"
    );

    // Step after VIEW (STO 0) must NOT have run
    assert_eq!(
        state.regs[0].inner(),
        Decimal::from(0),
        "STO 0 must not have executed (program broke at VIEW)"
    );
}

// ── Scenario: aview_mid_run_captures_alpha_into_yield ────────────────────────
//
// GREEN after 63-02 — requires run_loop AVIEW yield arm.
//
// AVIEW mid-program must:
//   1. Break run_loop.
//   2. Set pending_yield with kind=Aview, text=alpha_reg[..24], resume_ms=PSE_RESUME_MS.
//   3. NOT write display_override (D-04).
#[test]
fn aview_mid_run_captures_alpha_into_yield() {
    let mut state = CalcState::new();

    // Set ALPHA register content
    state.alpha_reg = "HELLO WORLD".to_string();

    load_program(
        &mut state,
        vec![
            Op::Lbl("AV".to_string()),
            Op::AView, // yield point
            Op::Sto(2), // must NOT run
            Op::Rtn,
        ],
    );

    hp41_core::ops::program::run_program(&mut state, "AV").unwrap();

    let py = state
        .pending_yield
        .as_ref()
        .expect("pending_yield must be Some after AVIEW mid-run");

    assert!(
        matches!(py.kind, YieldKind::Aview),
        "yield kind must be Aview"
    );
    assert_eq!(
        py.resume_ms, PSE_RESUME_MS,
        "AVIEW resume_ms must equal PSE_RESUME_MS"
    );
    assert_eq!(
        py.text, "HELLO WORLD",
        "pending_yield.text must be alpha_reg content; got: {:?}",
        py.text
    );

    // display_override must NOT be written (D-04)
    assert!(
        state.display_override.is_none(),
        "display_override must remain None after AVIEW yield (D-04)"
    );

    // Step after AVIEW (STO 2) must NOT have run
    assert_eq!(
        state.regs[2].inner(),
        Decimal::from(0),
        "STO 2 must not have executed (program broke at AVIEW)"
    );
}
