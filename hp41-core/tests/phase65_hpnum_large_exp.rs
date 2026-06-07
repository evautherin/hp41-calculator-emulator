#![allow(clippy::unwrap_used)]
#![allow(clippy::approx_constant)]

//! Phase 65 — large-exponent `HpNum` arithmetic regression suite (CR-02).
//!
//! Covers the behaviorally-novel MATH-01 (ADR v4.3-005) paths that the existing
//! property tests do NOT exercise (their generator clamps the exponent to ±18,
//! so every operand has `exponent == 0` and only the Decimal fast path runs).
//!
//! These tests are the regression guard for CR-01: before the fix,
//! `checked_mul` / `checked_div` computed the result exponent as `i32` then
//! narrowed it with `as i8` BEFORE the range check, so a true exponent above 99
//! wrapped (e.g. `198 as i8 == -58`) and returned a wildly wrong in-range value
//! instead of `Err(HpError::Overflow)`. The mul/div overflow cases below FAIL on
//! the pre-fix code (they returned a wrong `Ok` value) and PASS after.
//!
//! Also covers WR-02 (`to_sci` exponent extraction at exact powers of ten) and
//! the underflow-to-zero / in-range sanity semantics.

use hp41_core::error::HpError;
use hp41_core::HpNum;

// ─── CR-01 / CR-02: mul/div exponent overflow MUST be detected ──────────────────

#[test]
fn mul_1e80_times_1e80_overflows() {
    // True value 1e160 — exponent 160 > 99 → must Overflow.
    // Pre-CR-01: 80 + 80 = 160; `160 as i8 == -96` → returned ~1e-96 (garbage).
    let a = HpNum::from_f64(1e80).unwrap();
    let b = HpNum::from_f64(1e80).unwrap();
    assert_eq!(
        a.checked_mul(&b),
        Err(HpError::Overflow),
        "1e80 * 1e80 (true 1e160) must overflow, not wrap to an in-range value"
    );
}

#[test]
fn mul_9e99_times_9e99_overflows() {
    // 9e99 * 9e99 = 8.1e199 — exponent ~199 > 99 → must Overflow.
    // Pre-CR-01: 99 + 99 = 198; `198 as i8 == -58` → returned ~1e-58 (garbage).
    let a = HpNum::from_f64(9e99).unwrap();
    let b = HpNum::from_f64(9e99).unwrap();
    assert_eq!(
        a.checked_mul(&b),
        Err(HpError::Overflow),
        "9e99 * 9e99 (true ~8.1e199) must overflow"
    );
}

#[test]
fn div_at_ceiling_overflows() {
    // 9.999999999e99 / 0.1 = ~1e101 — exponent 100 > 99 → must Overflow.
    // Numerator has exponent 99, divisor 0.1 has sci-exponent -1, so the result
    // exponent is 99 - (-1) = 100. This exercises the checked_div large-exponent
    // path and the from_sci > 99 guard via the div call site (CR-01).
    let a = HpNum::from_f64(9.999_999_999e99).unwrap();
    let b = HpNum::from_f64(0.1).unwrap();
    assert_eq!(
        a.checked_div(&b),
        Err(HpError::Overflow),
        "9.999999999e99 / 0.1 (true ~1e101) must overflow"
    );
}

#[test]
fn div_large_by_small_exponent_overflows() {
    // 1e80 (exp 80) / 1e-40 — build the small divisor via a reciprocal-style
    // large-exponent value so the divisor genuinely carries a negative exponent
    // (not collapsed to Decimal-zero). 1e80 / 1e-40 = 1e120 → exponent 120 > 99.
    // Pre-CR-01: 80 - (-40) = 120; `120 as i8 == -120 + 256 = ...` wrapped to an
    // in-range value instead of overflowing.
    let numerator = HpNum::from_f64(1e80).unwrap();
    // 1e-40 IS representable as a large-exponent HpNum (mantissa 1.0, exp -40):
    // Decimal::from_f64(1e-40) collapses to 0, so construct it as 1 / 1e40.
    let divisor = HpNum::from_f64(1.0)
        .unwrap()
        .checked_div(&HpNum::from_f64(1e40).unwrap())
        .expect("1 / 1e40 = 1e-40 must succeed");
    assert_eq!(
        numerator.checked_div(&divisor),
        Err(HpError::Overflow),
        "1e80 / 1e-40 (true 1e120) must overflow"
    );
}

// ─── Underflow-to-zero (hardware-faithful) ──────────────────────────────────────

#[test]
fn div_underflow_to_zero() {
    // 1e-99 / 1e80 = 1e-179 — exponent -179 < -99 → underflow to zero.
    let a = HpNum::from_f64(1e-99).unwrap();
    let b = HpNum::from_f64(1e80).unwrap();
    let result = a.checked_div(&b).expect("underflow must be Ok(zero), not Err");
    assert!(
        result.is_zero(),
        "1e-99 / 1e80 (true 1e-179) must underflow to zero, got {result}"
    );
}

// ─── In-range sanity (must NOT overflow) ────────────────────────────────────────

#[test]
fn mul_1e40_times_1e40_is_1e80() {
    let a = HpNum::from_f64(1e40).unwrap();
    let b = HpNum::from_f64(1e40).unwrap();
    let result = a.checked_mul(&b).expect("1e40 * 1e40 must succeed (in range)");
    let full = result.to_f64().expect("result must be f64-representable");
    assert!(
        (full / 1e80 - 1.0).abs() < 1e-9,
        "1e40 * 1e40 must equal 1e80, got {full}"
    );
}

#[test]
fn mul_at_exact_exp_99_succeeds() {
    // 1e50 * 1e49 = 1e99 — exactly at the ceiling, must succeed.
    let a = HpNum::from_f64(1e50).unwrap();
    let b = HpNum::from_f64(1e49).unwrap();
    let result = a.checked_mul(&b).expect("1e50 * 1e49 = 1e99 must succeed at the ceiling");
    let full = result.to_f64().expect("result must be f64-representable");
    assert!(
        (full / 1e99 - 1.0).abs() < 1e-9,
        "1e50 * 1e49 must equal 1e99, got {full}"
    );
}

// ─── WR-02: to_sci exponent extraction at exact powers of ten ───────────────────
//
// to_sci() is private, but it is reached transitively: an exponent-0 operand
// multiplied by a large-exponent operand forces the large-exponent path, which
// calls to_sci() on BOTH. We assert the round-trip magnitude is exact at powers
// of ten — a wrong (off-by-one) exponent from f64 log10 would show up as a 10×
// error in the product.

fn assert_pow10_roundtrip(in_range: f64, large: HpNum, expected_ratio_to: f64) {
    // (in_range as exponent-0 HpNum) * (large-exponent HpNum) forces to_sci on
    // the in-range operand. Compare the full product to the expected magnitude.
    let a = HpNum::from_f64(in_range).unwrap();
    let result = a.checked_mul(&large).expect("product must succeed");
    let full = result.to_f64().expect("result must be f64-representable");
    assert!(
        (full / expected_ratio_to - 1.0).abs() < 1e-9,
        "to_sci power-of-ten extraction wrong: {in_range} * large = {full}, expected {expected_ratio_to}"
    );
}

#[test]
fn to_sci_exact_powers_of_ten() {
    let large = HpNum::from_f64(1e50).unwrap(); // exponent != 0 forces the sci path
    // 1e-1 * 1e50 = 1e49
    assert_pow10_roundtrip(1e-1, large.clone(), 1e49);
    // 1 * 1e50 = 1e50
    assert_pow10_roundtrip(1.0, large.clone(), 1e50);
    // 1e1 * 1e50 = 1e51
    assert_pow10_roundtrip(1e1, large.clone(), 1e51);
    // 1e15 * 1e50 = 1e65 (within range)
    assert_pow10_roundtrip(1e15, large.clone(), 1e65);
    // 1e-3 * 1e50 = 1e47
    assert_pow10_roundtrip(1e-3, large, 1e47);
}

// ─── WR-01: Extended serde arm validates / normalizes (untrusted boundary) ──────

#[test]
fn deserialize_extended_out_of_range_exponent_rejected() {
    // A hand-edited / corrupt save with |e| > 99 must be REJECTED, not stored.
    let corrupt = r#"{"m":"1.5","e":120}"#;
    let parsed: Result<HpNum, _> = serde_json::from_str(corrupt);
    assert!(
        parsed.is_err(),
        "Extended save with e=120 (out of -99..=99) must be rejected"
    );
}

#[test]
fn deserialize_extended_normalized_roundtrips() {
    // A legitimate large-exponent value round-trips losslessly.
    let n = HpNum::from_f64(1.711224524e98).unwrap();
    let json = serde_json::to_string(&n).unwrap();
    let back: HpNum = serde_json::from_str(&json).unwrap();
    assert_eq!(n, back, "normalized large-exponent save must round-trip");
}

#[test]
fn deserialize_legacy_bare_string_still_roundtrips() {
    // Save-file backward compat: legacy bare Decimal string (v1.0–v4.2) must
    // still load (the Legacy serde arm is unchanged by WR-01).
    let legacy = r#""3.1415926536""#;
    let back: HpNum = serde_json::from_str(legacy).unwrap();
    let full = back.to_f64().unwrap();
    assert!(
        (full - 3.141592654).abs() < 1e-8,
        "legacy bare-string save must still deserialize, got {full}"
    );
}
