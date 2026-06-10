// Algorithm independently re-derived from HP Extended Functions / CX Owner's Manual;
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Phase 64 — Interactive GETKEY yield-engine integration tests.
//!
//! Validates PRGM-03: Op::GetKey suspends execution via the Phase 63 yield engine
//! and resumes via `resume_program_with_key(keycode)`.
//!
//! Sub-behaviors covered:
//!   PRGM-03-a: GETKEY mid-run breaks run_loop and sets pending_yield = WaitForKey
//!   PRGM-03-b: resume_program_with_key(code) pushes row×col code to X (LiftEffect::Enable)
//!   PRGM-03-c: resume_program_with_key(0) pushes sentinel 0 (cancel path)
//!   PRGM-03-d: Program continues to the next step after GETKEY + resume
//!   PRGM-03-e: GETKEY does NOT write display_override (D-03: display unchanged)
//!   PRGM-03-f: GETKEY inside alarm handler: pending_interrupt* fields survive resume
//!   PRGM-03-g: GETKEY then PSE: sequential yields work (WaitForKey then Pse)
//!   PRGM-03-i: getkey_captured_code cleared after run_loop error (no leakage)
//!
//! PRGM-03-h (non-program interactive dispatch uses last_key_code) is covered by
//! existing `synthetic_tests.rs::test_getkey_pushes_last_key_code` — no gap.
//!
//! All scenarios use deterministic programs with no real keyboard input.
//! resume_program_with_key is called with an explicit keycode, mirroring
//! the Phase 63 no-wall-clock-sleep approach.
#![allow(clippy::unwrap_used)]

use hp41_core::ops::Op;
use hp41_core::state::{CalcState, YieldKind};
use hp41_core::HpNum;
use hp41_core::{resume_program_with_key, run_program};
use rust_decimal::Decimal;

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Load a program into `state.program` from a flat vec of ops.
fn load_program(state: &mut CalcState, ops: Vec<Op>) {
    state.program = ops;
}

// ── PRGM-03-a: getkey_mid_run_breaks_and_sets_wait_for_key ──────────────────
//
// A program with GETKEY mid-run must:
//   1. Break run_loop (run_program returns, is_running becomes false).
//   2. Set pending_yield with kind=WaitForKey, text="", resume_ms=0.
//   3. NOT write display_override (D-03: display unchanged).
//   4. Leave pc pointing at the step AFTER GETKEY (for resume).
//   5. NOT execute the step after GETKEY.
#[test]
fn getkey_mid_run_breaks_and_sets_wait_for_key() {
    let mut state = CalcState::new();
    load_program(
        &mut state,
        vec![
            Op::Lbl("G".to_string()),
            Op::GetKey,    // <- yield point
            Op::StoReg(0), // must NOT have run yet
            Op::Rtn,
        ],
    );

    run_program(&mut state, "G").unwrap();

    let py = state
        .pending_yield
        .as_ref()
        .expect("pending_yield must be Some after GETKEY mid-run");
    assert!(
        matches!(py.kind, YieldKind::WaitForKey),
        "yield kind must be WaitForKey, got {:?}",
        py.kind
    );
    assert_eq!(
        py.resume_ms, 0,
        "WaitForKey resume_ms must be 0 (event-driven)"
    );
    assert_eq!(
        py.text, "",
        "WaitForKey text must be empty (D-03: no display override)"
    );
    assert!(
        state.display_override.is_none(),
        "display_override must not be written by GETKEY (D-03)"
    );
    assert!(
        !state.is_running,
        "is_running must be false after run_program returns"
    );
    // STO 0 must NOT have run
    assert_eq!(
        state.regs[0].inner(),
        Decimal::from(0),
        "STO 0 must not have executed (program broke at GETKEY)"
    );
}

// ── PRGM-03-b: resume_with_key_pushes_code_to_x ─────────────────────────────
//
// After resume_program_with_key(71):
//   - The keycode 71 is pushed to X (LiftEffect::Enable applied by op_getkey).
//   - STO 1 captures it — proves X contains keycode.
//   - pending_yield is cleared.
//   - is_running is false.
#[test]
fn resume_with_key_pushes_code_to_x() {
    let mut state = CalcState::new();
    load_program(
        &mut state,
        vec![
            Op::Lbl("K".to_string()),
            Op::GetKey,    // yield here
            Op::StoReg(1), // STO 1 runs after resume; captures key code from X
            Op::Rtn,
        ],
    );

    run_program(&mut state, "K").unwrap();
    assert!(
        state.pending_yield.is_some(),
        "must be in WaitForKey yield before resume"
    );

    // Resume with keycode 71 ('1' key row 7, col 1)
    resume_program_with_key(&mut state, 71).unwrap();

    assert!(
        state.pending_yield.is_none(),
        "pending_yield must be None after resume"
    );
    assert!(
        !state.is_running,
        "is_running must be false after program completes"
    );
    // reg[1] must contain keycode 71 (pushed to X by GETKEY, then stored by STO 1)
    assert_eq!(
        state.regs[1].inner(),
        Decimal::from(71),
        "GETKEY must push keycode 71 to X; STO 1 must capture it"
    );
}

// ── PRGM-03-c: resume_with_key_zero_pushes_sentinel ─────────────────────────
//
// resume_program_with_key(0) delivers the no-key sentinel.
// HP-41 cancel/escape path (D-02) pushes 0 to X.
#[test]
fn resume_with_key_zero_pushes_sentinel() {
    let mut state = CalcState::new();
    load_program(
        &mut state,
        vec![
            Op::Lbl("C".to_string()),
            Op::GetKey,
            Op::StoReg(2), // captures sentinel 0
            Op::Rtn,
        ],
    );

    run_program(&mut state, "C").unwrap();
    assert!(state.pending_yield.is_some());

    // Cancel path: keycode 0 = no-key sentinel (D-02)
    resume_program_with_key(&mut state, 0).unwrap();

    assert_eq!(
        state.regs[2].inner(),
        Decimal::from(0),
        "cancel path must push sentinel 0 to X and store it in reg 2"
    );
}

// ── PRGM-03-d: getkey_resume_continues_to_next_step ─────────────────────────
//
// After resume, execution continues past GETKEY to the following steps.
#[test]
fn getkey_resume_continues_to_next_step() {
    let mut state = CalcState::new();
    // Push a known value into reg[4] before GETKEY to confirm continuation
    // steps run after resume.
    load_program(
        &mut state,
        vec![
            Op::Lbl("D".to_string()),
            Op::GetKey,
            Op::StoReg(4),                                  // step 1 after GETKEY
            Op::PushNum(HpNum::rounded(Decimal::from(99))), // step 2
            Op::StoReg(5),                                  // step 3 — proves full continuation
            Op::Rtn,
        ],
    );

    run_program(&mut state, "D").unwrap();
    assert!(state.pending_yield.is_some(), "must yield at GETKEY");

    resume_program_with_key(&mut state, 55).unwrap();

    // reg[4] must hold keycode 55 (GETKEY pushed it; STO 4 stored it)
    assert_eq!(
        state.regs[4].inner(),
        Decimal::from(55),
        "STO 4 after GETKEY must store the key code"
    );
    // reg[5] must hold 99 (the step after STO 4 ran to completion)
    assert_eq!(
        state.regs[5].inner(),
        Decimal::from(99),
        "continuation must reach STO 5 = 99"
    );
    assert!(!state.is_running);
}

// ── PRGM-03-e: getkey_does_not_write_display_override ───────────────────────
//
// D-03: GETKEY must NOT write display_override.
// Verify both before and after resume.
#[test]
fn getkey_does_not_write_display_override() {
    let mut state = CalcState::new();
    // Pre-set display_override to a known value to verify it is not cleared
    // or overwritten by GETKEY.
    state.display_override = Some("BEFORE".to_string());

    load_program(
        &mut state,
        vec![Op::Lbl("E".to_string()), Op::GetKey, Op::Rtn],
    );

    run_program(&mut state, "E").unwrap();
    assert!(state.pending_yield.is_some());
    // display_override must be unchanged (GETKEY must not touch it)
    assert_eq!(
        state.display_override.as_deref(),
        Some("BEFORE"),
        "GETKEY must not write display_override (D-03)"
    );

    resume_program_with_key(&mut state, 31).unwrap();
    // After resume, display_override still untouched
    assert_eq!(
        state.display_override.as_deref(),
        Some("BEFORE"),
        "resume_program_with_key must not write display_override (D-03)"
    );
}

// ── PRGM-03-f: getkey_inside_alarm_handler_preserves_interrupt_state ─────────
//
// If GETKEY fires inside an alarm-handler frame, pending_interrupt_alarm_index
// and pending_interrupt_depth must NOT be cleared by resume_program_with_key
// before run_loop runs (unlike resume_program which clears all interrupt state
// via D-09 at function entry).
//
// Strategy: we need to distinguish two observable states:
//   A) resume_program_with_key clears alarm_index BEFORE run_loop (wrong)
//   B) resume_program_with_key passes alarm_index INTO run_loop unchanged (correct)
//
// We do this by using a two-step program: after the GETKEY yield we inject
// alarm-handler context, then resume. In the resume we check that the GETKEY
// result was captured correctly (proves run_loop ran) AND that the RTN in the
// test program did NOT trigger an unexpected ack-after-RTN clear that would
// mask the regression.
//
// Specifically: the ack-after-RTN gate in run_loop fires only when
// `pending_interrupt_alarm_index.is_some()` AND the alarm at that index exists.
// In this test the alarm catalog is EMPTY (no alarm pushed), so ack-after-RTN
// cannot fire even if alarm_index reaches run_loop. This means:
//   - If alarm_index was cleared by resume_program_with_key BEFORE run_loop:
//     it will be None after the call.
//   - If alarm_index was correctly passed into run_loop unchanged: run_loop
//     finds no matching alarm (catalog empty) and leaves alarm_index untouched;
//     it will still be Some(2) after the call.
// The direct assertion `assert_eq!(pending_interrupt_alarm_index, Some(2))` below
// is therefore load-bearing: it fails if and only if resume_program_with_key
// prematurely clears the field.
#[test]
fn getkey_inside_alarm_handler_preserves_interrupt_state() {
    let mut state = CalcState::new();
    // Program: LBL H → GETKEY (yield) → STO 9 (captures keycode after resume) → RTN.
    // No alarm in catalog: ack-after-RTN cannot fire, so alarm_index is not consumed.
    load_program(
        &mut state,
        vec![
            Op::Lbl("H".to_string()),
            Op::GetKey,    // yield point
            Op::StoReg(9), // captures keycode — proves run_loop ran after resume
            Op::Rtn,
        ],
    );

    run_program(&mut state, "H").unwrap();
    assert!(
        state.pending_yield.is_some(),
        "must be in WaitForKey yield before resume"
    );

    // Simulate being inside an alarm handler frame by manually injecting the
    // alarm-handler context fields. In a real interrupt scenario these are set
    // by run_loop's Phase-B injection boundary (Phase 63). Here we inject them
    // AFTER the GETKEY yield to test that resume_program_with_key does NOT clear
    // them at function entry (before run_loop runs).
    state.pending_interrupt_alarm_index = Some(2); // index 2 — catalog is empty so ack-after-RTN cannot fire
    state.pending_interrupt_depth = Some(1);

    resume_program_with_key(&mut state, 44).unwrap();

    // Primary assertion: alarm_index must still be Some(2).
    // If resume_program_with_key had cleared it before run_loop, this would be None.
    // The empty alarm catalog ensures ack-after-RTN did NOT consume it.
    assert_eq!(
        state.pending_interrupt_alarm_index,
        Some(2),
        "pending_interrupt_alarm_index must survive resume_program_with_key unchanged \
         (must not be cleared at function entry before run_loop)"
    );
    assert_eq!(
        state.pending_interrupt_depth,
        Some(1),
        "pending_interrupt_depth must survive resume_program_with_key unchanged"
    );

    // Secondary assertion: the keycode reached STO 9 — proves run_loop actually ran.
    assert_eq!(
        state.regs[9].inner(),
        rust_decimal::Decimal::from(44),
        "STO 9 must have stored keycode 44 — proves run_loop ran after resume"
    );

    assert!(
        !state.is_running,
        "is_running must be reset after resume completes"
    );
}

// ── PRGM-03-g: getkey_then_pse_sequential_yields ─────────────────────────────
//
// Sequential yields: GETKEY (WaitForKey) followed by PSE (Pse).
// Verifies the yield engine handles two different yield kinds in sequence
// without corruption.
#[test]
fn getkey_then_pse_sequential_yields() {
    let mut state = CalcState::new();
    state.stack.x = HpNum::rounded(Decimal::from(42));
    state.stack.lift_enabled = true;

    load_program(
        &mut state,
        vec![
            Op::Lbl("S".to_string()),
            Op::GetKey,    // yield 1: WaitForKey
            Op::Pse,       // yield 2: Pse (after resume from GETKEY)
            Op::StoReg(6), // must run after second resume
            Op::Rtn,
        ],
    );

    // First run: breaks at GETKEY
    run_program(&mut state, "S").unwrap();
    {
        let py = state
            .pending_yield
            .as_ref()
            .expect("must have WaitForKey yield");
        assert!(
            matches!(py.kind, YieldKind::WaitForKey),
            "first yield must be WaitForKey"
        );
    }

    // Resume from GETKEY: run_loop continues to PSE, then breaks again
    resume_program_with_key(&mut state, 44).unwrap();
    {
        let py = state
            .pending_yield
            .as_ref()
            .expect("must have Pse yield after GETKEY resume");
        assert!(
            matches!(py.kind, YieldKind::Pse),
            "second yield must be Pse"
        );
    }

    // Resume from PSE: run_loop continues to STO 6 and completes
    hp41_core::resume_program(&mut state).unwrap();
    assert!(
        state.pending_yield.is_none(),
        "no more yields after PSE resume"
    );
    assert!(!state.is_running);
}

// ── PRGM-03-i: getkey_captured_code_cleared_on_error ─────────────────────────
//
// Even if run_loop returns Err, getkey_captured_code must be cleaned up (None).
// Simulated by a program that causes an error after GETKEY (invalid op path).
// Uses a program that tries to RCL beyond available registers to force an error.
//
// Note: We verify cleanup by checking the field is None after a resume that
// results in an Err return from resume_program_with_key.
#[test]
fn getkey_captured_code_cleared_on_error() {
    let mut state = CalcState::new();
    // RcpReg(255) will fail (out of bounds) — forces run_loop error after GETKEY
    load_program(
        &mut state,
        vec![
            Op::Lbl("I".to_string()),
            Op::GetKey,
            Op::RclReg(255), // out-of-bounds RCL — causes InvalidOp in run_loop
            Op::Rtn,
        ],
    );

    run_program(&mut state, "I").unwrap();
    assert!(state.pending_yield.is_some());

    // Resume will encounter an error on RclReg(255)
    let result = resume_program_with_key(&mut state, 22);
    assert!(result.is_err(), "resume must Err on out-of-bounds RCL");

    // getkey_captured_code must be None even on error (Pitfall 6 cleanup)
    assert!(
        state.getkey_captured_code.is_none(),
        "getkey_captured_code must be cleared even on error (Pitfall 6)"
    );
    // is_running must be false even on error (Pitfall 2)
    assert!(
        !state.is_running,
        "is_running must be reset even when resume_program_with_key returns Err"
    );
}

// ── CR-01 (Phase 64 code review): resume_with_key rejects spurious calls ──────
//
// resume_program_with_key must act ONLY when the program is suspended on a
// WaitForKey yield. A call when not waiting (no yield, or an unrelated PSE/VIEW
// yield) must return Err(InvalidOp) WITHOUT mutating the stack, the captured-code
// field, or clobbering the unrelated pending yield. Guards against a GUI
// double-tap race re-entering run_loop at the wrong pc (D-07 never-discard).
#[test]
fn resume_with_key_rejected_when_not_waiting_for_key() {
    // Case A — no pending yield at all.
    let mut state = CalcState::new();
    state.stack.x = HpNum::rounded(Decimal::from(7));
    let x_before = state.stack.x.clone();
    assert!(state.pending_yield.is_none());

    let r = resume_program_with_key(&mut state, 31);
    assert!(r.is_err(), "resume with no pending yield must Err");
    assert_eq!(
        state.stack.x, x_before,
        "stack X must be unchanged on reject"
    );
    assert!(
        state.getkey_captured_code.is_none(),
        "captured code must not be set on reject"
    );

    // Case B — suspended on a PSE yield (not WaitForKey): must NOT be consumed,
    // and the live PSE yield must survive untouched.
    let mut state = CalcState::new();
    state.stack.x = HpNum::rounded(Decimal::from(7));
    load_program(&mut state, vec![Op::Lbl("P".to_string()), Op::Pse, Op::Rtn]);
    run_program(&mut state, "P").unwrap();
    assert!(
        matches!(
            state.pending_yield.as_ref().map(|y| &y.kind),
            Some(YieldKind::Pse)
        ),
        "program should be suspended on a PSE yield"
    );
    let x_before = state.stack.x.clone();

    let r = resume_program_with_key(&mut state, 31);
    assert!(r.is_err(), "resume_with_key on a PSE yield must Err");
    assert!(
        matches!(
            state.pending_yield.as_ref().map(|y| &y.kind),
            Some(YieldKind::Pse)
        ),
        "PSE yield must be preserved after a rejected resume_with_key"
    );
    assert_eq!(
        state.stack.x, x_before,
        "stack X must be unchanged on reject"
    );
    assert!(
        state.getkey_captured_code.is_none(),
        "captured code must not be set on reject"
    );
}
