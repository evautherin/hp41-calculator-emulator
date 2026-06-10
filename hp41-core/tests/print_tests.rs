//! Integration tests for print operations: PRX, PRA, PRSTK.
//! PRNT-01, PRNT-02, PRNT-03 — hp41-core side.
//!
//! UNC-02 (Phase 66): PRX/PRA/PRSTK are printer-presence-gated on flags 21/55.
//! All tests that exercise the print path must set flag 55 (Printer Existence)
//! in `setup_with_printer()` so print ops succeed. One new test asserts that
//! the default-flags state (no printer) returns `Err(HpError::NonExistent)`.

#![allow(clippy::unwrap_used)]

use hp41_core::error::HpError;
use hp41_core::ops::flags::flag_set;
use hp41_core::ops::{dispatch, Op};
use hp41_core::{CalcState, DisplayMode, HpNum};

/// Return a `CalcState` with flag 55 (Printer Existence) set so PRX/PRA/PRSTK succeed.
fn setup_with_printer() -> CalcState {
    let mut s = CalcState::new();
    s.flags = flag_set(s.flags, 55); // flag 55 = Printer Existence (OM p.54)
    s
}

fn push_val(state: &mut CalcState, n: i32) {
    dispatch(state, Op::PushNum(HpNum::from(n))).unwrap();
}

// ── PRNT-01: PRX ─────────────────────────────────────────────────────────────

/// PRX pushes exactly one line to print_buffer.
#[test]
fn test_prx_pushes_one_line_to_buffer() {
    let mut s = setup_with_printer();
    push_val(&mut s, 42);
    dispatch(&mut s, Op::PRX).unwrap();
    assert_eq!(s.print_buffer.len(), 1, "PRX must push exactly 1 line");
}

/// PRX output line is exactly 24 chars wide (right-aligned).
#[test]
fn test_prx_output_is_24_chars() {
    let mut s = setup_with_printer();
    push_val(&mut s, 42);
    dispatch(&mut s, Op::PRX).unwrap();
    let line = &s.print_buffer[0];
    assert_eq!(
        line.len(),
        24,
        "PRX output must be exactly 24 chars, got: {line:?}"
    );
}

/// PRX output is right-aligned (leading spaces for short values).
#[test]
fn test_prx_output_is_right_aligned() {
    let mut s = setup_with_printer();
    push_val(&mut s, 1); // short value → many leading spaces
    dispatch(&mut s, Op::PRX).unwrap();
    let line = &s.print_buffer[0];
    // Right-aligned: the string ends with the value, not spaces
    assert!(
        line.starts_with(' '),
        "PRX output of small value must have leading spaces: {line:?}"
    );
}

/// PRX respects SCI display mode.
#[test]
fn test_prx_respects_display_mode_sci() {
    let mut s = setup_with_printer();
    s.display_mode = DisplayMode::Sci(4);
    push_val(&mut s, 12345);
    dispatch(&mut s, Op::PRX).unwrap();
    let line = &s.print_buffer[0];
    assert_eq!(line.len(), 24, "PRX SCI output must be 24 chars");
    // SCI 4 of 12345 contains 'E' or 'e' notation
    let trimmed = line.trim();
    assert!(
        trimmed.to_ascii_uppercase().contains('E'),
        "PRX in SCI mode must produce scientific notation: {line:?}"
    );
}

/// PRX has LiftEffect::Neutral — stack unchanged after PRX.
#[test]
fn test_prx_lift_effect_neutral() {
    let mut s = setup_with_printer();
    push_val(&mut s, 7);
    let x_before = s.stack.x.clone();
    dispatch(&mut s, Op::PRX).unwrap();
    assert_eq!(
        s.stack.x, x_before,
        "PRX must not modify the stack (LiftEffect::Neutral)"
    );
}

// ── PRNT-01: PRX no-printer guard (UNC-02) ───────────────────────────────────

/// PRX returns NonExistent when neither flag 21 (Printer Enable) nor flag 55
/// (Printer Existence) is set — OM p.57-58, UNC-02 (Phase 66).
#[test]
fn test_prx_returns_nonexistent_when_no_printer_flag() {
    let mut s = CalcState::new(); // default: flags = 0, no printer flags set
    push_val(&mut s, 42);
    let result = dispatch(&mut s, Op::PRX);
    assert_eq!(
        result,
        Err(HpError::NonExistent),
        "PRX with no printer flags must return Err(NonExistent)"
    );
}

/// PRX succeeds with ONLY flag 21 (Printer Enable) set — exercises the `|| flag_get(21)` branch.
///
/// UNC-02 (Phase 66): `require_printer` returns `Ok` if flag 55 OR flag 21 is set.
/// `setup_with_printer()` always sets flag 55; this test deliberately leaves flag 55 CLEAR
/// and sets ONLY flag 21 to ensure the flag-21 half of the disjunction is exercised.
/// Without this test a regression dropping `|| flag_get(21)` from `require_printer`
/// passes the entire suite undetected.
#[test]
fn test_prx_succeeds_with_flag21_only() {
    let mut s = CalcState::new(); // flags = 0 — no printer flags yet
    s.flags = flag_set(s.flags, 21); // set flag 21 (Printer Enable) only; flag 55 stays CLEAR
    push_val(&mut s, 42);
    let result = dispatch(&mut s, Op::PRX);
    assert_eq!(
        result,
        Ok(()),
        "PRX must succeed when only flag 21 (Printer Enable) is set"
    );
    assert_eq!(
        s.print_buffer.len(),
        1,
        "PRX with flag 21 must push exactly one print line"
    );
}

/// PRA returns NonExistent when neither flag 21 (Printer Enable) nor flag 55
/// (Printer Existence) is set — mirrors the existing PRX no-printer test for PRA.
///
/// UNC-02 (Phase 66): per-op wiring of `require_printer` was previously unasserted for PRA.
#[test]
fn test_pra_returns_nonexistent_when_no_printer_flag() {
    let mut s = CalcState::new(); // default: flags = 0
    s.alpha_reg = "HELLO".to_string();
    let result = dispatch(&mut s, Op::PRA);
    assert_eq!(
        result,
        Err(HpError::NonExistent),
        "PRA with no printer flags must return Err(NonExistent)"
    );
}

/// PRSTK returns NonExistent when neither flag 21 (Printer Enable) nor flag 55
/// (Printer Existence) is set — mirrors the existing PRX no-printer test for PRSTK.
///
/// UNC-02 (Phase 66): per-op wiring of `require_printer` was previously unasserted for PRSTK.
#[test]
fn test_prstk_returns_nonexistent_when_no_printer_flag() {
    let mut s = CalcState::new(); // default: flags = 0
    let result = dispatch(&mut s, Op::PRSTK);
    assert_eq!(
        result,
        Err(HpError::NonExistent),
        "PRSTK with no printer flags must return Err(NonExistent)"
    );
}

// ── PRNT-02: PRA ─────────────────────────────────────────────────────────────

/// PRA pushes exactly one line to print_buffer.
#[test]
fn test_pra_pushes_one_line_to_buffer() {
    let mut s = setup_with_printer();
    s.alpha_reg = "HELLO".to_string();
    dispatch(&mut s, Op::PRA).unwrap();
    assert_eq!(s.print_buffer.len(), 1, "PRA must push exactly 1 line");
}

/// PRA output line is exactly 24 chars wide (left-aligned).
#[test]
fn test_pra_output_is_24_chars() {
    let mut s = setup_with_printer();
    s.alpha_reg = "HI".to_string();
    dispatch(&mut s, Op::PRA).unwrap();
    let line = &s.print_buffer[0];
    assert_eq!(
        line.len(),
        24,
        "PRA output must be exactly 24 chars, got: {line:?}"
    );
}

/// PRA output is left-aligned (trailing spaces for short values).
#[test]
fn test_pra_output_is_left_aligned() {
    let mut s = setup_with_printer();
    s.alpha_reg = "HI".to_string();
    dispatch(&mut s, Op::PRA).unwrap();
    let line = &s.print_buffer[0];
    // Left-aligned: starts with "HI", ends with spaces
    assert!(
        line.starts_with("HI"),
        "PRA output must start with alpha content: {line:?}"
    );
    assert!(
        line.ends_with(' '),
        "PRA output of short value must have trailing spaces: {line:?}"
    );
}

/// PRA with empty alpha_reg produces 24 spaces.
#[test]
fn test_pra_empty_alpha_is_24_spaces() {
    let mut s = setup_with_printer();
    s.alpha_reg = String::new();
    dispatch(&mut s, Op::PRA).unwrap();
    let line = &s.print_buffer[0];
    assert_eq!(
        line.as_str(),
        "                        ", // 24 spaces
        "PRA with empty alpha_reg must produce 24 spaces"
    );
}

/// PRA truncates alpha_reg longer than 24 chars to 24 chars.
#[test]
fn test_pra_truncates_long_alpha_to_24_chars() {
    let mut s = setup_with_printer();
    s.alpha_reg = "A".repeat(30);
    dispatch(&mut s, Op::PRA).unwrap();
    let line = &s.print_buffer[0];
    assert_eq!(line.len(), 24, "PRA must truncate to 24 chars");
}

// ── PRNT-03: PRSTK ───────────────────────────────────────────────────────────

/// PRSTK pushes exactly 6 lines to print_buffer.
#[test]
fn test_prstk_produces_six_lines() {
    let mut s = setup_with_printer();
    dispatch(&mut s, Op::PRSTK).unwrap();
    assert_eq!(s.print_buffer.len(), 6, "PRSTK must push exactly 6 lines");
}

/// All 6 PRSTK lines are exactly 24 chars wide.
#[test]
fn test_prstk_all_lines_are_24_chars() {
    let mut s = setup_with_printer();
    push_val(&mut s, 1);
    dispatch(&mut s, Op::PRSTK).unwrap();
    for (i, line) in s.print_buffer.iter().enumerate() {
        assert_eq!(
            line.len(),
            24,
            "PRSTK line {i} must be 24 chars, got {line:?}"
        );
    }
}

/// PRSTK lines appear in T/Z/Y/X/LASTX/ALPHA order.
#[test]
fn test_prstk_line_order_and_labels() {
    let mut s = setup_with_printer();
    dispatch(&mut s, Op::PRSTK).unwrap();
    assert!(
        s.print_buffer[0].starts_with("T:"),
        "Line 0 must start with T:"
    );
    assert!(
        s.print_buffer[1].starts_with("Z:"),
        "Line 1 must start with Z:"
    );
    assert!(
        s.print_buffer[2].starts_with("Y:"),
        "Line 2 must start with Y:"
    );
    assert!(
        s.print_buffer[3].starts_with("X:"),
        "Line 3 must start with X:"
    );
    assert!(
        s.print_buffer[4].starts_with("LASTX:"),
        "Line 4 must start with LASTX:"
    );
    assert!(
        s.print_buffer[5].starts_with("ALPHA:"),
        "Line 5 must start with ALPHA:"
    );
}

/// PRSTK ALPHA line when alpha_reg is empty: "ALPHA:                  " ("ALPHA:" (6) + 18 spaces = 24).
#[test]
fn test_prstk_alpha_empty_line_format() {
    let mut s = setup_with_printer();
    s.alpha_reg = String::new();
    dispatch(&mut s, Op::PRSTK).unwrap();
    let alpha_line = &s.print_buffer[5];
    assert_eq!(
        alpha_line.as_str(),
        "ALPHA:                  ", // "ALPHA:" (6) + 1 pad (to 7-char label field) + 17 spaces (content field) = 24
        "PRSTK ALPHA line with empty alpha_reg must be 'ALPHA:' padded to 24 chars"
    );
}

/// PRSTK ALPHA line with content: left-aligned in 17-char field after 7-char label.
#[test]
fn test_prstk_alpha_nonempty_line_format() {
    let mut s = setup_with_printer();
    s.alpha_reg = "HP41".to_string();
    dispatch(&mut s, Op::PRSTK).unwrap();
    let alpha_line = &s.print_buffer[5];
    assert_eq!(alpha_line.len(), 24, "PRSTK ALPHA line must be 24 chars");
    assert!(
        alpha_line.starts_with("ALPHA:"),
        "PRSTK ALPHA line must start with 'ALPHA:'"
    );
    assert!(
        alpha_line.contains("HP41"),
        "PRSTK ALPHA line must contain alpha content"
    );
}

// ── PRNT-01/02/03: program execution (execute_op arms) ───────────────────────

/// PRX inside a running program fills print_buffer (execute_op arm must exist).
#[test]
fn test_prx_in_program() {
    let mut s = setup_with_printer();
    push_val(&mut s, 99);
    // Build a minimal program: LBL "P", PRX, RTN
    s.program = vec![Op::Lbl("P".to_string()), Op::PRX, Op::Rtn];
    hp41_core::run_program(&mut s, "P").unwrap();
    assert_eq!(
        s.print_buffer.len(),
        1,
        "PRX in a program must push 1 line to print_buffer"
    );
}

/// PRA inside a running program fills print_buffer (execute_op arm must exist).
#[test]
fn test_pra_in_program() {
    let mut s = setup_with_printer();
    s.alpha_reg = "TEST".to_string();
    s.program = vec![Op::Lbl("Q".to_string()), Op::PRA, Op::Rtn];
    hp41_core::run_program(&mut s, "Q").unwrap();
    assert_eq!(
        s.print_buffer.len(),
        1,
        "PRA in a program must push 1 line to print_buffer"
    );
}

/// PRSTK inside a running program fills print_buffer with 6 lines.
#[test]
fn test_prstk_in_program() {
    let mut s = setup_with_printer();
    s.program = vec![Op::Lbl("R".to_string()), Op::PRSTK, Op::Rtn];
    hp41_core::run_program(&mut s, "R").unwrap();
    assert_eq!(
        s.print_buffer.len(),
        6,
        "PRSTK in a program must push 6 lines to print_buffer"
    );
}
