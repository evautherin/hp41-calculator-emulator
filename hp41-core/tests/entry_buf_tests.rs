//! Integration tests for the number entry buffer (entry_buf) flush semantics.
//!
//! entry_buf holds pending digit characters. On any dispatch() call,
//! flush_entry_buf() parses and pushes the buffered number before the op executes.
//!
//! Also tests backspace_entry() — the shared HP-41 fidelity helper that
//! implements per-digit deletion during number entry.

use hp41_core::ops::{backspace_entry, dispatch, Op};
use hp41_core::{CalcState, HpNum};
use rust_decimal::Decimal;
use std::str::FromStr;

// ── Basic flush on math op ────────────────────────────────────────────────

#[test]
fn test_entry_buf_flushed_before_math_op() {
    // Set entry_buf = "4"; dispatch Sqrt → should compute sqrt(4) = 2
    let mut s = CalcState::new();
    s.entry_buf = "4".to_string();
    dispatch(&mut s, Op::Sqrt).unwrap();
    assert_eq!(
        s.stack.x.inner(),
        Decimal::from(2),
        "entry_buf '4' must flush before Sqrt"
    );
    assert!(
        s.entry_buf.is_empty(),
        "entry_buf must be cleared after flush"
    );
}

#[test]
fn test_entry_buf_flushed_before_add() {
    // Set X=1; entry_buf = "3"; dispatch Add → should compute 1 + 3 = 4
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(1);
    s.stack.lift_enabled = true; // so entry_buf push lifts 1 to Y
    s.entry_buf = "3".to_string();
    dispatch(&mut s, Op::Add).unwrap();
    assert_eq!(s.stack.x.inner(), Decimal::from(4));
    assert!(
        s.entry_buf.is_empty(),
        "entry_buf must be cleared after flush"
    );
}

// ── Flush on Enter ────────────────────────────────────────────────────────

#[test]
fn test_entry_buf_flushed_before_enter() {
    // entry_buf = "7"; dispatch Enter → 7 should be on stack, then Enter duplicates it
    let mut s = CalcState::new();
    s.entry_buf = "7".to_string();
    dispatch(&mut s, Op::Enter).unwrap();
    // After flush, X = 7. Then Enter duplicates X → Y = 7, X = 7.
    assert_eq!(s.stack.x.inner(), Decimal::from(7));
    assert_eq!(s.stack.y.inner(), Decimal::from(7));
    assert!(s.entry_buf.is_empty());
}

// ── Flush on STO ──────────────────────────────────────────────────────────

#[test]
fn test_entry_buf_flushed_before_sto() {
    // entry_buf = "42"; dispatch StoReg(0) → 42 should be stored in R00
    let mut s = CalcState::new();
    s.entry_buf = "42".to_string();
    dispatch(&mut s, Op::StoReg(0)).unwrap();
    assert_eq!(s.regs[0].inner(), Decimal::from(42));
    assert!(s.entry_buf.is_empty());
}

// ── Empty buf is a no-op ──────────────────────────────────────────────────

#[test]
fn test_empty_entry_buf_is_noop() {
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(5);
    s.entry_buf = String::new(); // empty
    dispatch(&mut s, Op::Sq).unwrap(); // 5² = 25
    assert_eq!(s.stack.x.inner(), Decimal::from(25));
}

// ── Decimal number in buf ─────────────────────────────────────────────────

#[test]
fn test_entry_buf_decimal_number() {
    let mut s = CalcState::new();
    s.entry_buf = "3.14".to_string();
    dispatch(&mut s, Op::Clx).unwrap();
    // Flush pushes 3.14, then CLX overwrites X with 0.
    assert!(s.stack.x.is_zero(), "CLX must zero X");
    assert!(s.entry_buf.is_empty());
}

// ── Negative number in buf ────────────────────────────────────────────────

#[test]
fn test_entry_buf_negative_number() {
    let mut s = CalcState::new();
    s.entry_buf = "-9".to_string();
    dispatch(&mut s, Op::Sq).unwrap(); // (-9)² = 81
    assert_eq!(s.stack.x.inner(), Decimal::from(81));
    assert!(s.entry_buf.is_empty());
}

// ── Flush enables lift ────────────────────────────────────────────────────

#[test]
fn test_flush_enables_lift() {
    let mut s = CalcState::new();
    s.entry_buf = "5".to_string();
    s.stack.lift_enabled = false;
    // Dispatch a Neutral op (SetDeg) to trigger flush without changing stack
    dispatch(&mut s, Op::SetDeg).unwrap();
    assert!(
        s.stack.lift_enabled,
        "flush must enable lift after pushing a number"
    );
    assert_eq!(s.stack.x.inner(), Decimal::from(5));
}

// ── Multi-digit integer in buf ────────────────────────────────────────────

#[test]
fn test_entry_buf_multi_digit_integer() {
    // "150" in entry_buf; dispatch Sq → 150² = 22500
    let mut s = CalcState::new();
    s.entry_buf = "150".to_string();
    dispatch(&mut s, Op::Sq).unwrap();
    let expected = Decimal::from_str("22500").unwrap();
    assert_eq!(s.stack.x.inner(), expected);
    assert!(s.entry_buf.is_empty());
}

// ── Invalid entry_buf returns error ──────────────────────────────────────

#[test]
fn test_entry_buf_invalid_content_returns_error() {
    use hp41_core::HpError;
    let mut s = CalcState::new();
    s.entry_buf = "not_a_number".to_string();
    let result = dispatch(&mut s, Op::Sqrt);
    assert!(
        result.is_err(),
        "malformed entry_buf should yield InvalidOp error"
    );
    assert_eq!(result.unwrap_err(), HpError::InvalidOp);
    // WR-02: entry_buf is preserved on parse error so the user can see what went wrong.
    // Previously it was cleared unconditionally; now it is only cleared on success.
    assert_eq!(
        s.entry_buf, "not_a_number",
        "entry_buf must be preserved on parse error (WR-02: no silent data loss)"
    );
}

// ── backspace_entry — HP-41 fidelity: per-digit deletion during number entry ─

/// Backspace on a multi-digit entry removes only the last digit.
/// "300" ← → "30" (entry_buf still non-empty; stack unchanged).
#[test]
fn test_backspace_entry_multi_digit_removes_last() {
    let mut s = CalcState::new();
    s.entry_buf = "300".to_string();
    backspace_entry(&mut s);
    assert_eq!(s.entry_buf, "30", "backspace on '300' must leave '30'");
    // Stack is not touched — entry_buf still active
    assert!(
        s.stack.x.is_zero(),
        "stack X must not change during entry_buf edit"
    );
}

/// Backspace on a decimal entry removes the last character (the digit after dot).
/// "3.5" ← → "3." (entry_buf still non-empty).
#[test]
fn test_backspace_entry_decimal_removes_last_char() {
    let mut s = CalcState::new();
    s.entry_buf = "3.5".to_string();
    backspace_entry(&mut s);
    assert_eq!(s.entry_buf, "3.", "backspace on '3.5' must leave '3.'");
}

/// Backspace on a single-digit entry fully empties the buf and calls clx,
/// leaving X=0 with lift disabled.
/// "3" ← → entry_buf="", X=0.
#[test]
fn test_backspace_entry_single_digit_clears_to_zero() {
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(99); // pre-existing X (should be overwritten by implicit clx)
    s.stack.lift_enabled = true;
    s.entry_buf = "3".to_string();
    backspace_entry(&mut s);
    assert!(
        s.entry_buf.is_empty(),
        "entry_buf must be empty after full backspace"
    );
    assert!(
        s.stack.x.is_zero(),
        "X must be 0 (clx) after full backspace"
    );
    assert!(
        !s.stack.lift_enabled,
        "lift must be disabled after full backspace (clx semantics)"
    );
}

/// Backspace when entry_buf is already empty (number complete / no active entry)
/// behaves exactly like CLX: clears X to 0 and disables lift.
#[test]
fn test_backspace_entry_empty_buf_acts_as_clx() {
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(42);
    s.stack.lift_enabled = true;
    s.entry_buf = String::new();
    backspace_entry(&mut s);
    assert!(s.entry_buf.is_empty(), "entry_buf stays empty");
    assert!(
        s.stack.x.is_zero(),
        "X must be 0 (clx) when buf was already empty"
    );
    assert!(!s.stack.lift_enabled, "lift must be disabled after clx");
}

/// Backspace removes the decimal point itself when it is the last character.
/// "3." ← → "3" (entry_buf still non-empty).
#[test]
fn test_backspace_entry_removes_trailing_dot() {
    let mut s = CalcState::new();
    s.entry_buf = "3.".to_string();
    backspace_entry(&mut s);
    assert_eq!(s.entry_buf, "3", "backspace on '3.' must leave '3'");
}
