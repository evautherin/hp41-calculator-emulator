//! Integration tests for Phase 22 Plan 01 (program control:
//! STOP / PSE / GTO IND / XEQ IND + resume_program).
//!
//! Covers FN-PROG-01 / FN-PROG-02 / FN-PROG-06 / FN-PROG-07 plus the
//! three RESEARCH.md §2 pitfall sentinels:
//! - Pitfall 1: Op::Stop must NOT write display_override.
//! - Pitfall 2: resume_program must reset is_running on the Err path.
//! - Pitfall 3: Op::Pse's display_override survives subsequent run_loop
//!   iterations and is cleared by the next dispatch.

#![allow(clippy::unwrap_used)]

use hp41_core::ops::program::{resume_program, run_program};
use hp41_core::ops::Op;
use hp41_core::{format_hpnum, CalcState, DisplayMode, HpError, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;

// ── FN-PROG-01: STOP halts; resume_program continues from pc ────────────────

#[test]
fn test_stop_then_resume() {
    let mut state = CalcState::new();
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum::from(42i32)),
        Op::Stop,
        Op::PushNum(HpNum::from(99i32)),
    ];

    run_program(&mut state, "A").unwrap();

    // After STOP: X is 42, is_running is false, pc is within program
    // (advanced past STOP by the top-of-iteration pc += 1).
    assert_eq!(state.stack.x, HpNum::from(42i32));
    assert!(
        !state.is_running,
        "is_running must be false after STOP halt"
    );
    assert!(
        state.pc < state.program.len(),
        "pc ({}) must point at the next step within program (len {})",
        state.pc,
        state.program.len()
    );

    // Resume — continues from saved pc through the final PushNum.
    resume_program(&mut state).unwrap();
    assert_eq!(state.stack.x, HpNum::from(99i32));
    assert!(!state.is_running);
}

// ── Pitfall 1 sentinel: STOP must NOT write display_override ────────────────

#[test]
fn test_stop_does_not_write_display_override() {
    let mut state = CalcState::new();
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum::from(7i32)),
        Op::Stop,
    ];
    // Pre-condition: display_override starts None.
    assert!(state.display_override.is_none());

    run_program(&mut state, "A").unwrap();

    // The PushNum step does not write display_override either, so the
    // contract that STOP "freezes whatever is currently displayed" reduces
    // to: STOP leaves display_override unchanged. With no prior writer in
    // this program, that means display_override stays None.
    assert!(
        state.display_override.is_none(),
        "Op::Stop must NOT write display_override (Pitfall 1); got {:?}",
        state.display_override
    );
}

// ── Pitfall 2 sentinel: resume_program resets is_running on Err path ────────

#[test]
fn test_resume_resets_is_running_on_err() {
    let mut state = CalcState::new();
    // Program designed to fail mid-run: XEQ to non-existent label.
    state.program = vec![Op::Xeq("MISSING".to_string())];
    state.pc = 0;

    let result = resume_program(&mut state);

    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "expected InvalidOp from missing-label XEQ, got {result:?}"
    );
    assert!(
        !state.is_running,
        "is_running MUST be reset to false even on Err path (Pitfall 2)"
    );
}

#[test]
fn test_resume_program_rejects_when_pc_past_end() {
    let mut state = CalcState::new();
    state.program = vec![Op::PushNum(HpNum::from(1i32))];
    state.pc = state.program.len(); // past end

    let result = resume_program(&mut state);
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "resume_program must reject when pc >= program.len()"
    );
    assert!(!state.is_running);
}

#[test]
fn test_resume_program_preserves_call_stack() {
    // resume_program must NOT clear state.call_stack (unlike run_program).
    let mut state = CalcState::new();
    state.program = vec![Op::PushNum(HpNum::from(1i32))]; // any benign program
    state.pc = 0;
    state.call_stack = vec![123usize, 456usize];

    resume_program(&mut state).unwrap();

    // The benign program does not pop the call_stack, so both frames must
    // still be there after a successful resume.
    assert_eq!(state.call_stack, vec![123usize, 456usize]);
}

// ── FN-PROG-02: PSE — Phase 63 yield-and-resume (PRGM-01 / D-04) ───────────
//
// Phase 63 changed Op::Pse mid-run behavior: run_loop now breaks and sets
// `pending_yield` instead of writing `display_override` + pushing "PAUSE 1000".
// The old display_override + event_buffer path survives ONLY for interactive
// (non-program) PSE keystrokes, handled by execute_op in dispatch().

#[test]
fn test_pse_writes_both_channels() {
    // Phase 63: run_loop intercepts Op::Pse and breaks with pending_yield.
    // display_override is NOT written (D-04: DISP-01 deferred to v4.4).
    // "PAUSE 1000" event marker is NOT pushed (replaced by typed yield channel).
    let mut state = CalcState::new();
    state.display_mode = DisplayMode::Fix(4);
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum::rounded(Decimal::from_str("1.23").unwrap())),
        Op::Pse,
    ];

    run_program(&mut state, "A").unwrap();

    // Phase 63: display_override must NOT be written by the program-path PSE (D-04).
    assert!(
        state.display_override.is_none(),
        "Phase 63: display_override must NOT be written by program-path PSE (D-04)"
    );
    // Phase 63: no legacy "PAUSE 1000" marker — typed pending_yield channel instead.
    assert!(
        !state.event_buffer.iter().any(|e| e == "PAUSE 1000"),
        "Phase 63: 'PAUSE 1000' must NOT be in event_buffer (replaced by pending_yield)"
    );
    // Phase 63: pending_yield must be set with kind=Pse.
    let py = state.pending_yield.as_ref().expect("pending_yield must be Some after PSE");
    assert!(
        matches!(py.kind, hp41_core::state::YieldKind::Pse),
        "pending_yield kind must be Pse"
    );
    // text must contain the formatted X value.
    let expected = format_hpnum(
        &HpNum::rounded(Decimal::from_str("1.23").unwrap()),
        &DisplayMode::Fix(4),
    );
    assert_eq!(py.text, expected, "pending_yield.text must be format_hpnum(X)");
    assert_eq!(py.resume_ms, hp41_core::state::PSE_RESUME_MS);
}

// ── Pitfall 3 sentinel: PSE breaks run_loop — steps after PSE do NOT run ────

#[test]
fn test_pse_display_override_survives_next_program_step() {
    // Phase 63: PSE breaks run_loop. Steps AFTER PSE do not execute until
    // resume_program is called. This test verifies the break (not display survival).
    let mut state = CalcState::new();
    state.display_mode = DisplayMode::Fix(4);
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum::rounded(Decimal::from_str("1.23").unwrap())),
        Op::Pse,
        Op::PushNum(HpNum::from(5i32)), // must NOT execute before resume
    ];

    run_program(&mut state, "A").unwrap();

    // Phase 63: PSE breaks run_loop, so the subsequent PushNum(5) did NOT run.
    // X must still be 1.23 (the push that happened before PSE).
    let expected_pse_x = HpNum::rounded(Decimal::from_str("1.23").unwrap());
    assert_eq!(
        state.stack.x, expected_pse_x,
        "Phase 63: PSE breaks run_loop; subsequent PushNum(5) must not have run yet"
    );
    // pending_yield must be set (program is paused at PSE).
    assert!(
        state.pending_yield.is_some(),
        "Phase 63: pending_yield must be Some (program paused at PSE)"
    );
    // display_override must NOT be written.
    assert!(
        state.display_override.is_none(),
        "Phase 63: display_override must NOT be written by program-path PSE (D-04)"
    );
}

// ── Pitfall 3 sentinel: next interactive dispatch clears display_override ───
//
// Phase 63: PSE no longer writes display_override during program execution.
// This test verifies resume_program clears pending_yield and continues execution.

#[test]
fn test_pse_display_override_cleared_by_next_dispatch() {
    // Phase 63: PSE sets pending_yield and breaks. resume_program clears pending_yield.
    let mut state = CalcState::new();
    state.display_mode = DisplayMode::Fix(4);
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::PushNum(HpNum::from(7i32)),
        Op::Pse,
        Op::PushNum(HpNum::from(42i32)), // runs after resume
    ];

    run_program(&mut state, "A").unwrap();
    assert!(
        state.pending_yield.is_some(),
        "Phase 63: PSE must set pending_yield (program paused)"
    );
    assert!(
        state.display_override.is_none(),
        "Phase 63: display_override must NOT be written by program-path PSE (D-04)"
    );

    // Resume — pending_yield is cleared and execution continues.
    resume_program(&mut state).unwrap();
    assert!(
        state.pending_yield.is_none(),
        "pending_yield must be cleared after resume_program"
    );
    // PushNum(42) ran after resume — X = 42.
    assert_eq!(
        state.stack.x,
        HpNum::from(42i32),
        "PushNum(42) must have run after resume_program"
    );
}

// ── FN-PROG-06: GTO IND happy + reject paths ────────────────────────────────

#[test]
fn test_gto_ind_happy() {
    // R05 = 42 (integer pointer); program has LBL "42" target.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(42i32).into();
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::GtoInd(5),
        Op::PushNum(HpNum::from(111i32)), // would-be unreachable after GTO
        Op::Lbl("42".to_string()),
        Op::PushNum(HpNum::from(7i32)),
    ];

    run_program(&mut state, "A").unwrap();
    assert_eq!(
        state.stack.x,
        HpNum::from(7i32),
        "GTO IND R05 (= 42) must branch to LBL 42; got X = {:?}",
        state.stack.x
    );
}

#[test]
fn test_gto_ind_non_integer_rejects() {
    let mut state = CalcState::new();
    state.regs[5] = HpNum::rounded(Decimal::from_str("12.345").unwrap()).into();
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::GtoInd(5),
        Op::Lbl("12".to_string()), // would-be target if truncation were allowed
    ];

    let result = run_program(&mut state, "A");
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "GTO IND with non-integer pointer must return InvalidOp (FN-IND-02); got {result:?}"
    );
}

#[test]
fn test_gto_ind_reg_out_of_range_rejects() {
    // CalcState::new() ships 100 registers (R00..R99). Asking for R200 must
    // return InvalidOp via .get().ok_or(InvalidOp) — and crucially NOT panic.
    let mut state = CalcState::new();
    state.program = vec![Op::Lbl("A".to_string()), Op::GtoInd(200)];

    let result = run_program(&mut state, "A");
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "GTO IND with reg >= regs.len() must return InvalidOp, not panic; got {result:?}"
    );
}

// ── FN-PROG-07: XEQ IND happy + reject paths + call-stack guard ─────────────

#[test]
fn test_xeq_ind_happy() {
    let mut state = CalcState::new();
    state.regs[3] = HpNum::from(10i32).into();
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::XeqInd(3),
        Op::PushNum(HpNum::from(2i32)),
        Op::Rtn,
        Op::Lbl("10".to_string()),
        Op::PushNum(HpNum::from(99i32)),
        Op::Rtn,
    ];

    run_program(&mut state, "A").unwrap();
    // After: A pushes 99 (via the subroutine), RTN returns to step after
    // XeqInd, pushes 2, RTN at the top level breaks.
    assert_eq!(
        state.stack.x,
        HpNum::from(2i32),
        "X after XEQ IND subroutine + return must be 2; got {:?}",
        state.stack.x
    );
}

#[test]
fn test_xeq_ind_4_deep_call_stack_rejects() {
    // Drive via resume_program (NOT run_program) so the pre-set call_stack
    // is NOT cleared at entry — run_program would wipe it (line 162).
    let mut state = CalcState::new();
    state.regs[3] = HpNum::from(10i32).into();
    state.program = vec![Op::XeqInd(3), Op::Lbl("10".to_string())];
    state.pc = 0;
    state.call_stack = vec![999usize; 4]; // pre-fill to 4 frames

    let result = resume_program(&mut state);

    assert!(
        matches!(result, Err(HpError::CallDepth)),
        "XEQ IND at 4-deep call_stack must return CallDepth; got {result:?}"
    );
    // Pre-mutation atomicity: the push did NOT happen — still exactly 4.
    assert_eq!(
        state.call_stack.len(),
        4,
        "call_stack must not be mutated on CallDepth (pre-mutation guard); got {:?}",
        state.call_stack
    );
}

#[test]
fn test_xeq_ind_reg_out_of_range_rejects() {
    let mut state = CalcState::new();
    state.program = vec![Op::Lbl("A".to_string()), Op::XeqInd(200)];

    let result = run_program(&mut state, "A");
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "XEQ IND with reg >= regs.len() must return InvalidOp, not panic; got {result:?}"
    );
}

#[test]
fn test_xeq_ind_non_integer_rejects() {
    let mut state = CalcState::new();
    state.regs[3] = HpNum::rounded(Decimal::from_str("10.5").unwrap()).into();
    state.program = vec![Op::Lbl("A".to_string()), Op::XeqInd(3)];

    let result = run_program(&mut state, "A");
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "XEQ IND with non-integer pointer must return InvalidOp (FN-IND-02); got {result:?}"
    );
}

// ── Phase 24 D-24.5 sentinel: refactored GtoInd/XeqInd onto shared helper ────
//
// These tests exercise the same code paths as the original Phase-22 tests
// above, but assert specific invariants of the shared-helper refactor:
// (1) call-depth guard still runs FIRST (pre-mutation atomicity)
// (2) Decimal::to_string preserves sign for label-lookup callers
//
// If any of these tests fail, the refactor regressed Phase-22 behavior.

#[test]
fn phase24_gto_ind_uses_shared_helper() {
    // Sanity — proves the refactored arm still routes via
    // crate::ops::indirect::resolve_indirect_decimal to find_in_program.
    // Identical inputs/outputs to test_gto_ind_happy.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(42i32).into();
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::GtoInd(5),
        Op::PushNum(HpNum::from(111i32)), // would-be unreachable after GTO
        Op::Lbl("42".to_string()),
        Op::PushNum(HpNum::from(7i32)),
    ];

    run_program(&mut state, "A").unwrap();
    assert_eq!(
        state.stack.x,
        HpNum::from(7i32),
        "GTO IND R05 (= 42) must branch to LBL 42 via shared helper; got X = {:?}",
        state.stack.x
    );
}

#[test]
fn phase24_xeq_ind_uses_shared_helper() {
    // Sanity for XeqInd refactor — same flow as test_xeq_ind_happy.
    let mut state = CalcState::new();
    state.regs[3] = HpNum::from(10i32).into();
    state.program = vec![
        Op::Lbl("A".to_string()),
        Op::XeqInd(3),
        Op::PushNum(HpNum::from(2i32)),
        Op::Rtn,
        Op::Lbl("10".to_string()),
        Op::PushNum(HpNum::from(99i32)),
        Op::Rtn,
    ];

    run_program(&mut state, "A").unwrap();
    assert_eq!(
        state.stack.x,
        HpNum::from(2i32),
        "X after XEQ IND subroutine + return must be 2 via shared helper; got {:?}",
        state.stack.x
    );
}

#[test]
fn phase24_xeq_ind_call_depth_guard_runs_before_pointer_read() {
    // CRITICAL D-24.5 sentinel: drive via resume_program (NOT run_program —
    // the latter wipes call_stack at entry per the inline comment at
    // test_xeq_ind_4_deep_call_stack_rejects above). With a 4-deep call_stack
    // AND a non-integer pointer, the pre-mutation atomicity guard must fire
    // FIRST and return CallDepth — NOT InvalidOp from a downstream pointer
    // read. If a future planner accidentally moves the call_stack.len() >= 4
    // check below the pointer read, this test catches it.
    let mut state = CalcState::new();
    state.regs[3] = HpNum::rounded(Decimal::from_str("12.345").unwrap()).into();
    state.program = vec![Op::XeqInd(3), Op::Lbl("12".to_string())];
    state.pc = 0;
    state.call_stack = vec![999usize; 4]; // pre-fill to 4 frames

    let result = resume_program(&mut state);

    assert!(
        matches!(result, Err(HpError::CallDepth)),
        "XEQ IND at 4-deep call_stack with non-integer pointer must return \
         CallDepth (pre-mutation guard fires FIRST), NOT InvalidOp; got {result:?}"
    );
    // Pre-mutation atomicity: the push did NOT happen — still exactly 4.
    assert_eq!(
        state.call_stack.len(),
        4,
        "call_stack must not be mutated on CallDepth (pre-mutation guard); got {:?}",
        state.call_stack
    );
}

#[test]
fn phase24_gto_ind_negative_pointer_stringifies_with_sign() {
    // Pitfall 2 sentinel: -3 IS an integer (passes the inner helper without
    // rejection), so the failure path is "find_in_program('-3') failed", NOT
    // "non-integer pointer". This confirms Decimal::to_string preserves the
    // sign exactly as Phase 22 did pre-refactor.
    let mut state = CalcState::new();
    state.regs[5] = HpNum::from(-3i32).into();
    state.program = vec![Op::Lbl("A".to_string()), Op::GtoInd(5)]; // no LBL "-3"

    let result = run_program(&mut state, "A");
    assert!(
        matches!(result, Err(HpError::InvalidOp)),
        "GTO IND R05 (= -3) must InvalidOp via find_in_program (label '-3' \
         not found), NOT via non-integer rejection; got {result:?}"
    );
}
