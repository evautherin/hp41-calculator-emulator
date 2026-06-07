#![allow(clippy::unwrap_used)]
#![allow(clippy::approx_constant)]

//! Phase 65 — supplemental coverage for num.rs and format.rs paths omitted
//! from the initial test suite (phase65_hpnum_large_exp.rs).
//!
//! Targets (by file):
//!
//! **num.rs**
//! - `HpValue::Alpha` delegation methods (lines 31-108): as_numeric / numeric_or_zero
//!   / inner / checked_add/sub/mul/div / is_zero / negate / trunc_int / checked_sq
//!   when the operand is the `Alpha` variant.
//! - `from_sci` borrow-down branch (|m| < 1) and carry-after-round (accessed via
//!   from_f64 which calls it internally with renormalized mantissa).
//! - `from_f64` slow path: non-finite returns None, out-of-range exp returns None.
//! - `checked_add_sci` short-circuits and alignment branches.
//! - `checked_mul` zero-operand short-circuit.
//! - `checked_sqrt` large-exponent path.
//! - `checked_ln` large-exponent path.
//! - `checked_log10` large-exponent path.
//! - `checked_exp` large-exponent path.
//! - `checked_exp10` path via to_f64 bridge.
//! - `checked_powd` large-exponent path.
//! - `trunc_int` large-exponent paths.
//! - `decimal_pow10_small` match arms > 0.
//! - `Display` for exponent != 0.
//!
//! **format.rs**
//! - `format_hpnum` large-exponent branch in FIX / SCI / ENG modes.
//! - `format_sci_large` function body.
//! - `round_to_display_precision` large-exponent branch.
//! - `format_alpha` function.
//! - `decimal_pow10` negative branch (via format_eng with sub-unity value).

use hp41_core::format::{format_alpha, format_hpnum, round_to_display_precision};
use hp41_core::state::DisplayMode;
use hp41_core::{HpNum, HpValue};

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn large(f: f64) -> HpNum {
    HpNum::from_f64(f).expect("test value must be constructable")
}

// ─── HpValue::Alpha delegation methods ──────────────────────────────────────

#[test]
fn hpvalue_alpha_as_numeric_is_err() {
    let alpha = HpValue::Alpha([b'A', b'B', 0, 0, 0, 0]);
    assert!(
        alpha.as_numeric().is_err(),
        "Alpha variant must return Err from as_numeric"
    );
}

#[test]
fn hpvalue_alpha_numeric_or_zero_is_zero() {
    let alpha = HpValue::Alpha([1, 2, 3, 4, 5, 6]);
    assert!(
        alpha.numeric_or_zero().is_zero(),
        "Alpha numeric_or_zero must return zero"
    );
}

#[test]
fn hpvalue_alpha_inner_is_zero() {
    let alpha = HpValue::Alpha([0u8; 6]);
    // inner() on Alpha treats it as zero → returns Decimal::ZERO
    assert_eq!(
        alpha.inner(),
        rust_decimal::Decimal::ZERO,
        "Alpha inner() must return zero"
    );
}

#[test]
fn hpvalue_alpha_is_zero_true() {
    let alpha = HpValue::Alpha([7, 8, 9, 0, 0, 0]);
    assert!(
        alpha.is_zero(),
        "Alpha is_zero must return true (treated as zero)"
    );
}

#[test]
fn hpvalue_alpha_negate_is_zero() {
    let alpha = HpValue::Alpha([0u8; 6]);
    let result = alpha.negate();
    assert!(result.is_zero(), "Alpha negate must return zero");
}

#[test]
fn hpvalue_alpha_trunc_int_is_zero() {
    let alpha = HpValue::Alpha([0u8; 6]);
    let result = alpha.trunc_int();
    assert!(result.is_zero(), "Alpha trunc_int must return zero");
}

#[test]
fn hpvalue_alpha_checked_add_with_num() {
    let alpha = HpValue::Alpha([0u8; 6]);
    let rhs = HpNum::from(42i32);
    let result = alpha.checked_add(&rhs).unwrap();
    // Alpha treated as zero, so result = 0 + 42 = 42
    let v = result.to_f64().unwrap();
    assert!((v - 42.0).abs() < 1e-9, "Alpha + 42 must equal 42, got {v}");
}

#[test]
fn hpvalue_alpha_checked_sub_with_num() {
    let alpha = HpValue::Alpha([0u8; 6]);
    let rhs = HpNum::from(5i32);
    let result = alpha.checked_sub(&rhs).unwrap();
    // Alpha treated as zero: 0 - 5 = -5
    let v = result.to_f64().unwrap();
    assert!(
        (v - (-5.0)).abs() < 1e-9,
        "Alpha - 5 must equal -5, got {v}"
    );
}

#[test]
fn hpvalue_alpha_checked_mul_with_num() {
    let alpha = HpValue::Alpha([0u8; 6]);
    let rhs = HpNum::from(100i32);
    let result = alpha.checked_mul(&rhs).unwrap();
    // Alpha treated as zero: 0 * 100 = 0
    assert!(result.is_zero(), "Alpha * 100 must equal zero");
}

#[test]
fn hpvalue_alpha_checked_div_by_nonzero() {
    let alpha = HpValue::Alpha([0u8; 6]);
    let rhs = HpNum::from(7i32);
    let result = alpha.checked_div(&rhs).unwrap();
    // Alpha treated as zero: 0 / 7 = 0
    assert!(result.is_zero(), "Alpha / 7 must equal zero");
}

#[test]
fn hpvalue_alpha_checked_sq() {
    let alpha = HpValue::Alpha([0u8; 6]);
    let result = alpha.checked_sq().unwrap();
    // Alpha treated as zero: 0^2 = 0
    assert!(result.is_zero(), "Alpha checked_sq must equal zero");
}

// ─── from_sci / borrow-down / carry-up (via from_f64 decomposition) ──────────
// from_sci is pub(crate) so we reach it through from_f64 or deserialization.

#[test]
fn serde_extended_with_small_mantissa_deserializes_via_borrow_down() {
    // Build an extended JSON with m < 1 (e.g. "0.5") and e = 50.
    // The deserializer calls from_sci("0.5", 50) which triggers borrow-down.
    let json = r#"{"m":"0.5","e":50}"#;
    let n: HpNum = serde_json::from_str(json).unwrap();
    let full = n.to_f64().unwrap();
    // 0.5 × 10^50 = 5.0 × 10^49
    assert!(
        (full / 5e49 - 1.0).abs() < 1e-9,
        "0.5e50 borrow-down must equal 5e49, got {full}"
    );
}

#[test]
fn serde_extended_carry_up_large_mantissa() {
    // m = 15.0, e = 50 → carry-up to 1.5 × 10^51.
    let json = r#"{"m":"15","e":50}"#;
    let n: HpNum = serde_json::from_str(json).unwrap();
    let full = n.to_f64().unwrap();
    assert!(
        (full / 1.5e51 - 1.0).abs() < 1e-9,
        "15×10^50 carry-up must equal 1.5e51, got {full}"
    );
}

#[test]
fn serde_extended_carry_up_overflow() {
    // m = 15.0, e = 99 → carry would push to 10^100 → overflow (reject).
    let json = r#"{"m":"15","e":99}"#;
    let result: Result<HpNum, _> = serde_json::from_str(json);
    assert!(
        result.is_err(),
        "15×10^99 carry-up beyond 99 must be rejected"
    );
}

#[test]
fn serde_extended_borrow_down_underflow_to_zero() {
    // m = 0.1, e = -99 → borrow-down moves to 1.0 × 10^(-100) → underflow to zero.
    let json = r#"{"m":"0.1","e":-99}"#;
    let n: HpNum = serde_json::from_str(json).unwrap();
    assert!(
        n.is_zero(),
        "0.1×10^(-99) borrow-down underflow must be zero"
    );
}

#[test]
fn serde_extended_carry_after_round_pushes_exponent() {
    // 9.99999999995 × 10^50: round_sf(10) → 10.0 → carry to 1.0 × 10^51.
    let json = r#"{"m":"9.99999999995","e":50}"#;
    let n: HpNum = serde_json::from_str(json).unwrap();
    let full = n.to_f64().unwrap();
    assert!(
        (full / 1e51 - 1.0).abs() < 1e-9,
        "9.99999999995e50 carry-after-round must produce 1e51, got {full}"
    );
}

// ─── from_f64: non-finite / out-of-range returns None ────────────────────────

#[test]
fn from_f64_nan_returns_none() {
    assert!(
        HpNum::from_f64(f64::NAN).is_none(),
        "from_f64(NaN) must return None"
    );
}

#[test]
fn from_f64_infinity_returns_none() {
    assert!(
        HpNum::from_f64(f64::INFINITY).is_none(),
        "from_f64(+Inf) must return None"
    );
    assert!(
        HpNum::from_f64(f64::NEG_INFINITY).is_none(),
        "from_f64(-Inf) must return None"
    );
}

#[test]
fn from_f64_above_exp99_returns_none() {
    // 1e100 has exponent 100 > 99 → must return None
    assert!(
        HpNum::from_f64(1e100).is_none(),
        "from_f64(1e100) must return None (exp 100 > 99)"
    );
}

#[test]
fn from_f64_large_value_slow_path() {
    // 1e50 exceeds Decimal range (~7.92E28), exercising the slow path.
    let n = large(1e50);
    let full = n.to_f64().unwrap();
    assert!(
        (full / 1e50 - 1.0).abs() < 1e-9,
        "from_f64(1e50) slow path must reconstruct 1e50, got {full}"
    );
}

#[test]
fn from_f64_negative_large_value_slow_path() {
    // Negative large value: -1.23456789e60
    let n = large(-1.23456789e60);
    let full = n.to_f64().unwrap();
    assert!(
        (full / -1.23456789e60 - 1.0).abs() < 1e-8,
        "from_f64(-1.23456789e60) slow path, got {full}"
    );
}

// ─── checked_add_sci: short-circuits and alignment branches ──────────────────

#[test]
fn checked_add_sci_self_zero_short_circuit() {
    // Zero + large = large (self is zero short-circuit in checked_add_sci).
    // Force the large-exp path: rhs has exponent != 0.
    let zero = HpNum::zero();
    let large_val = large(5e50);
    let result = zero.checked_add(&large_val).unwrap();
    let full = result.to_f64().unwrap();
    assert!(
        (full / 5e50 - 1.0).abs() < 1e-9,
        "zero + 5e50 must equal 5e50, got {full}"
    );
}

#[test]
fn checked_add_sci_rhs_zero_short_circuit() {
    // Large + zero = large (rhs is zero short-circuit in checked_add_sci).
    let large_val = large(3e60);
    let zero = HpNum::zero();
    let result = large_val.checked_add(&zero).unwrap();
    let full = result.to_f64().unwrap();
    assert!(
        (full / 3e60 - 1.0).abs() < 1e-9,
        "3e60 + zero must equal 3e60, got {full}"
    );
}

#[test]
fn checked_add_sci_exp_diff_ge_10_uses_larger() {
    // exp_diff >= 10: smaller value is rounded away (hardware-faithful).
    // 1e99 + 1e85: diff = 14 → result ≈ 1e99
    let a = large(1e99);
    let b = large(1e85);
    let result = a.checked_add(&b).unwrap();
    let full = result.to_f64().unwrap();
    assert!(
        (full / 1e99 - 1.0).abs() < 1e-6,
        "1e99 + 1e85 must return ~1e99 (smaller lost), got {full}"
    );
}

#[test]
fn checked_add_sci_exp_diff_le_neg10_uses_rhs() {
    // exp_diff <= -10: self is the smaller, rhs is the larger.
    // 1e85 + 1e99: diff = 85-99 = -14 → result ≈ 1e99
    let a = large(1e85);
    let b = large(1e99);
    let result = a.checked_add(&b).unwrap();
    let full = result.to_f64().unwrap();
    assert!(
        (full / 1e99 - 1.0).abs() < 1e-6,
        "1e85 + 1e99 must return ~1e99, got {full}"
    );
}

#[test]
fn checked_add_sci_equal_exponents() {
    // Both have equal exponents: exp_diff = 0 (positive alignment branch, scale = 1).
    // 2e50 + 3e50 = 5e50
    let a = large(2e50);
    let b = large(3e50);
    let result = a.checked_add(&b).unwrap();
    let full = result.to_f64().unwrap();
    assert!(
        (full / 5e50 - 1.0).abs() < 1e-9,
        "2e50 + 3e50 must equal 5e50, got {full}"
    );
}

#[test]
fn checked_add_sci_rhs_larger_exponent() {
    // rhs has larger exponent: self is scaled up (negative alignment branch in the code).
    // 1e49 + 1e50 = 1.1e50
    let a = large(1e49);
    let b = large(1e50);
    let result = a.checked_add(&b).unwrap();
    let full = result.to_f64().unwrap();
    assert!(
        (full / 1.1e50 - 1.0).abs() < 1e-9,
        "1e49 + 1e50 must equal 1.1e50, got {full}"
    );
}

#[test]
fn checked_add_sci_self_larger_exponent() {
    // self has larger exponent: rhs is scaled up.
    // 1e50 + 1e49 = 1.1e50
    let a = large(1e50);
    let b = large(1e49);
    let result = a.checked_add(&b).unwrap();
    let full = result.to_f64().unwrap();
    assert!(
        (full / 1.1e50 - 1.0).abs() < 1e-9,
        "1e50 + 1e49 must equal 1.1e50, got {full}"
    );
}

// ─── checked_mul: zero operand short-circuit ─────────────────────────────────

#[test]
fn checked_mul_zero_times_large_is_zero() {
    // When one operand is zero and the other has exponent != 0,
    // exercise the is_zero short-circuit in the large-exponent mul path.
    // Force large-exp path: create zero (exponent=0) * large (exponent!=0)
    let zero = HpNum::zero();
    let large_val = large(5e60);
    // self.exponent==0, rhs.exponent!=0 → large-exp path → is_zero check
    let result = zero.checked_mul(&large_val).unwrap();
    assert!(result.is_zero(), "0 * 5e60 must be zero");
}

#[test]
fn checked_mul_large_times_zero_is_zero() {
    let large_val = large(5e60);
    let zero = HpNum::zero();
    // self.exponent!=0 (large_val), rhs.is_zero() → is_zero short-circuit
    let result = large_val.checked_mul(&zero).unwrap();
    assert!(result.is_zero(), "5e60 * 0 must be zero");
}

// ─── checked_sqrt large-exponent path ────────────────────────────────────────

#[test]
fn checked_sqrt_large_exponent_1e80() {
    // sqrt(1e80) = 1e40
    let a = large(1e80);
    let result = a.checked_sqrt().unwrap();
    let full = result.to_f64().unwrap();
    assert!(
        (full / 1e40 - 1.0).abs() < 1e-9,
        "sqrt(1e80) must equal 1e40, got {full}"
    );
}

#[test]
fn checked_sqrt_large_exponent_4e98() {
    // sqrt(4e98) = 2e49
    let a = large(4e98);
    let result = a.checked_sqrt().unwrap();
    let full = result.to_f64().unwrap();
    assert!(
        (full / 2e49 - 1.0).abs() < 1e-9,
        "sqrt(4e98) must equal 2e49, got {full}"
    );
}

// ─── checked_ln large-exponent path ─────────────────────────────────────────

#[test]
fn checked_ln_1e98() {
    // ln(1e98) = 98 * ln(10) ≈ 225.6533191
    let a = large(1e98);
    let result = a.checked_ln().unwrap();
    let full = result.to_f64().unwrap();
    let expected = (1e98f64).ln();
    assert!(
        (full - expected).abs() < 1e-4,
        "ln(1e98) must be ~{expected:.6}, got {full}"
    );
}

#[test]
fn checked_ln_1e50() {
    // ln(1e50) = 50 * ln(10) ≈ 115.1292546
    let a = large(1e50);
    let result = a.checked_ln().unwrap();
    let full = result.to_f64().unwrap();
    let expected = (1e50f64).ln();
    assert!(
        (full - expected).abs() < 1e-5,
        "ln(1e50) must be ~{expected:.6}, got {full}"
    );
}

// ─── checked_log10 large-exponent path ───────────────────────────────────────

#[test]
fn checked_log10_1e50() {
    // log10(1e50) = 50 (mantissa = 1.0, exponent = 50 → log10 = 0 + 50 = 50)
    let a = large(1e50);
    let result = a.checked_log10().unwrap();
    let full = result.to_f64().unwrap();
    assert!(
        (full - 50.0).abs() < 1e-9,
        "log10(1e50) must equal 50.0, got {full}"
    );
}

#[test]
fn checked_log10_2e98() {
    // log10(2e98) = 98 + log10(2) ≈ 98.30103
    let a = large(2e98);
    let result = a.checked_log10().unwrap();
    let full = result.to_f64().unwrap();
    let expected = (2e98f64).log10();
    assert!(
        (full - expected).abs() < 1e-6,
        "log10(2e98) must be ~{expected:.6}, got {full}"
    );
}

// ─── checked_exp large-exponent path ─────────────────────────────────────────
// The large-exp branch is: `if self.exponent != 0 { let v = self.to_f64()... }`
// We need a value with exponent != 0. from_f64 for values > ~7.92e28 gives exponent != 0.

#[test]
fn checked_exp_large_exponent_overflows() {
    // exp(1e30) is astronomically large → overflow.
    // 1e30 has exponent != 0, triggering the large-exp path.
    let x = large(1e30);
    let result = x.checked_exp();
    assert!(
        result.is_err(),
        "exp(1e30) must overflow (large-exp path), got {result:?}"
    );
}

#[test]
fn checked_exp_large_negative_exponent_underflows() {
    // exp(-1e30) ≈ 0 → underflows to zero.
    let neg = large(1e30).negate();
    let result = neg.checked_exp().unwrap();
    assert!(result.is_zero(), "exp(-1e30) must underflow to zero");
}

// ─── checked_exp10 large-exponent path ────────────────────────────────────────

#[test]
fn checked_exp10_large_exponent_overflows() {
    // 10^(1e30) is beyond exp=99 → must return overflow.
    let x = large(1e30);
    let result = x.checked_exp10();
    assert!(result.is_err(), "10^(1e30) must overflow, got {result:?}");
}

// ─── checked_powd large-exponent path ────────────────────────────────────────

#[test]
fn checked_powd_large_base_overflows() {
    // (1e50)^2 = 1e100 → out of range. The f64 bridge path returns Err (Domain or Overflow).
    let base = large(1e50);
    let exp = HpNum::from(2i32);
    let result = base.checked_powd(&exp);
    assert!(
        result.is_err(),
        "(1e50)^2 must fail (1e100 > max range), got {result:?}"
    );
}

#[test]
fn checked_powd_large_base_power_1() {
    // (1e40)^1 = 1e40 — stays in range via the f64 bridge.
    let base = large(1e40);
    let exp = HpNum::from(1i32);
    let result = base.checked_powd(&exp).unwrap();
    let full = result.to_f64().unwrap();
    assert!(
        (full / 1e40 - 1.0).abs() < 1e-9,
        "(1e40)^1 must equal 1e40, got {full}"
    );
}

#[test]
fn checked_powd_large_exp_operand_overflows() {
    // 2^(1e30): exp operand has exponent != 0 → f64 bridge → overflow.
    let base = HpNum::from(2i32);
    let exp_val = large(1e30);
    let result = base.checked_powd(&exp_val);
    assert!(result.is_err(), "2^(1e30) must overflow");
}

// ─── trunc_int large-exponent paths ─────────────────────────────────────────

#[test]
fn trunc_int_large_exponent_ge10_returns_self() {
    // exponent >= 10: value is an integer at that magnitude → returns clone.
    // 1e50: exponent = 50 → already an integer.
    let a = large(1e50);
    let result = a.trunc_int();
    let full = result.to_f64().unwrap();
    assert!(
        (full / 1e50 - 1.0).abs() < 1e-9,
        "trunc_int(1e50) must return 1e50, got {full}"
    );
}

#[test]
fn trunc_int_large_exponent_1_truncates() {
    // exponent = 1: mantissa × 10^1 is a two-digit number, truncate fractional part.
    // We deserialize 3.7 × 10^1 = 37.0 via the extended serde arm.
    // from_f64(37.0) will have exponent == 0 (fits in Decimal). We need
    // a value where from_f64 gives exponent == 1. from_f64(1e30) has exponent=30.
    // For exponent in [1..9], we construct via serde Extended form with m=3.7, e=1.
    let json = r#"{"m":"3.7","e":1}"#;
    let n: HpNum = serde_json::from_str(json).unwrap();
    let result = n.trunc_int();
    let full = result.to_f64().unwrap();
    // 3.7 × 10^1 = 37.0; trunc = 37
    assert!(
        (full - 37.0).abs() < 1e-9,
        "trunc_int(3.7e1=37.0) must return 37.0, got {full}"
    );
}

#[test]
fn trunc_int_large_exponent_5_truncates() {
    // 1.9 × 10^5 = 190000; trunc → 190000 (no fractional part to cut).
    let json = r#"{"m":"1.9","e":5}"#;
    let n: HpNum = serde_json::from_str(json).unwrap();
    let result = n.trunc_int();
    let full = result.to_f64().unwrap();
    assert!(
        (full - 190_000.0).abs() < 1.0,
        "trunc_int(1.9e5=190000) must return 190000, got {full}"
    );
}

// ─── decimal_pow10_small match arms ──────────────────────────────────────────
// The match arms 0..=9 in decimal_pow10_small are exercised by checked_add_sci
// alignment with specific exponent differences 1..9.

#[test]
fn checked_add_sci_exercises_all_pow10_small_arms() {
    // For exp_diff = k (1..=9), the alignment branch calls decimal_pow10_small(k).
    // Use: a = 10^(50+k), b = 10^50 → exp_diff = k, scale = 10^k.
    for k in 1usize..=9 {
        let exp_a = 50 + k as i32;
        let a = large(10f64.powi(exp_a));
        let b = large(1e50);
        let result = a.checked_add(&b).unwrap();
        let full = result.to_f64().unwrap();
        let expected = 10f64.powi(exp_a) + 1e50;
        assert!(
            (full / expected - 1.0).abs() < 1e-7,
            "10^{exp_a} + 1e50 (k={k}): expected ~{expected:.3e}, got {full:.3e}"
        );
    }
}

// ─── Display for exponent != 0 ────────────────────────────────────────────────

#[test]
fn display_large_exponent_contains_e() {
    let n = large(1.711224524e98);
    let s = format!("{n}");
    assert!(
        s.contains('e'),
        "Display of large-exponent HpNum must contain 'e', got {s:?}"
    );
}

#[test]
fn display_1e50_format() {
    // 1e50: mantissa = 1.0 (stored as Decimal with trailing zeros), exponent = 50.
    // The Display impl writes "{mantissa}e{exponent}" — mantissa is the raw Decimal.
    let n = large(1e50);
    let s = format!("{n}");
    // The format is "Xe50" for some representation of 1.0 — just verify e50 is present
    assert!(
        s.ends_with("e50"),
        "Display of 1e50 must end with 'e50', got {s:?}"
    );
    assert!(
        s.contains('e'),
        "Display of large-exponent value must contain 'e', got {s:?}"
    );
}

#[test]
fn display_large_exponent_negative_value() {
    // Negative large-exp value should show the negative mantissa.
    let n = large(-2.5e75);
    let s = format!("{n}");
    assert!(
        s.contains('e'),
        "Display of negative large-exp must contain 'e', got {s:?}"
    );
    assert!(
        s.contains('-'),
        "Display of negative large-exp must contain '-', got {s:?}"
    );
}

// ─── format_hpnum large-exponent branch (FIX / SCI / ENG) ───────────────────

#[test]
fn format_hpnum_large_exp_fix_overflows_to_sci9() {
    // FIX mode for a large-exponent value always overflows to SCI 9.
    let n = large(1.711224524e98);
    let s = format_hpnum(&n, &DisplayMode::Fix(4));
    assert!(
        s.contains('E'),
        "FIX mode for large-exp must produce SCI output, got {s:?}"
    );
    assert!(
        s.contains("98"),
        "FIX overflow output must contain exponent 98, got {s:?}"
    );
}

#[test]
fn format_hpnum_large_exp_sci4() {
    // SCI(4) mode for FACT(69) ≈ 1.711224524e98.
    let n = large(1.711224524e98);
    let s = format_hpnum(&n, &DisplayMode::Sci(4));
    // Mantissa 1.711224524 at 4 decimal places → "1.7112"; exponent 98
    assert!(
        s.contains("E 98"),
        "SCI(4) for 1.711e98 must contain 'E 98', got {s:?}"
    );
    assert!(
        s.starts_with("1.7112"),
        "SCI(4) for 1.711e98 must start with '1.7112', got {s:?}"
    );
}

#[test]
fn format_hpnum_large_exp_sci0() {
    // SCI(0) mode: only 1 digit before decimal, decimal point must be present.
    let n = large(5.5e60);
    let s = format_hpnum(&n, &DisplayMode::Sci(0));
    assert!(
        s.contains('.'),
        "SCI(0) large-exp output must contain decimal point, got {s:?}"
    );
    assert!(
        s.contains('E'),
        "SCI(0) large-exp output must contain 'E', got {s:?}"
    );
}

#[test]
fn format_hpnum_large_exp_eng_mode() {
    // ENG mode for a large-exponent value falls through to the `digits` branch.
    let n = large(1.711224524e98);
    let s = format_hpnum(&n, &DisplayMode::Eng(3));
    assert!(
        s.contains('E'),
        "ENG mode for large-exp must produce SCI output (overflow), got {s:?}"
    );
}

#[test]
fn format_hpnum_large_exp_negative() {
    // Negative large-exponent value: sign must appear.
    let n = large(-5.5e75);
    let s = format_hpnum(&n, &DisplayMode::Sci(4));
    assert!(
        s.starts_with('-'),
        "Negative large-exp SCI output must start with '-', got {s:?}"
    );
    assert!(
        s.contains("75"),
        "Output must contain exponent 75, got {s:?}"
    );
}

#[test]
fn format_sci_large_carry_bumps_exponent() {
    // 9.99995 × 10^50 in SCI(4): mantissa rounds to 10.0000 → carry to 1.0000E 51.
    // Construct via serde: m="9.99995", e=50
    let json = r#"{"m":"9.99995","e":50}"#;
    let n: HpNum = serde_json::from_str(json).unwrap();
    let s = format_hpnum(&n, &DisplayMode::Sci(4));
    assert!(
        s.contains("51"),
        "format_sci_large carry must bump exponent to 51, got {s:?}"
    );
}

// ─── round_to_display_precision large-exponent branch ────────────────────────

#[test]
fn round_to_display_precision_large_exp_fix() {
    // FIX mode for large-exp uses SCI 9 precision internally (9 sig digits).
    let n = large(1.711224524e98);
    let result = round_to_display_precision(&n, &DisplayMode::Fix(4));
    let full = result.to_f64().unwrap();
    assert!(
        (full / 1.711224524e98 - 1.0).abs() < 1e-8,
        "round_to_display_precision Fix large-exp must preserve 9 sig digits, got {full}"
    );
}

#[test]
fn round_to_display_precision_large_exp_sci4() {
    // SCI(4): round to 5 sig digits.
    let n = large(1.711224524e98);
    let result = round_to_display_precision(&n, &DisplayMode::Sci(4));
    let full = result.to_f64().unwrap();
    // 1.711224524e98 to 5 sig digits → 1.7112e98
    assert!(
        (full / 1.7112e98 - 1.0).abs() < 1e-4,
        "round_to_display_precision Sci(4) large-exp must round to 5 sig digits, got {full}"
    );
}

#[test]
fn round_to_display_precision_large_exp_eng3() {
    // ENG(3) mode for large-exp.
    let n = large(3.141592654e80);
    let result = round_to_display_precision(&n, &DisplayMode::Eng(3));
    let full = result.to_f64().unwrap();
    // Rounded to 4 sig digits.
    assert!(
        (full / 3.142e80 - 1.0).abs() < 1e-3,
        "round_to_display_precision Eng(3) large-exp, got {full}"
    );
}

#[test]
fn round_to_display_precision_large_exp_carry() {
    // If rounding pushes mantissa to 10, exponent bumps.
    // 9.99995 × 10^50 at Sci(4): 9.99995 rounded to 5 sig → 10.000 → carry to 1.0×10^51.
    let json = r#"{"m":"9.99995","e":50}"#;
    let n: HpNum = serde_json::from_str(json).unwrap();
    let result = round_to_display_precision(&n, &DisplayMode::Sci(4));
    let full = result.to_f64().unwrap();
    assert!(
        (full / 1e51 - 1.0).abs() < 1e-9,
        "round_to_display_precision carry must produce 1e51, got {full}"
    );
}

// ─── format_alpha ─────────────────────────────────────────────────────────────

#[test]
fn format_alpha_short_string_unchanged() {
    assert_eq!(format_alpha("HELLO"), "HELLO");
}

#[test]
fn format_alpha_exactly_12_chars_unchanged() {
    assert_eq!(format_alpha("ABCDEFGHIJKL"), "ABCDEFGHIJKL");
}

#[test]
fn format_alpha_truncates_beyond_12() {
    assert_eq!(format_alpha("ABCDEFGHIJKLMNOP"), "ABCDEFGHIJKL");
}

#[test]
fn format_alpha_empty_string() {
    assert_eq!(format_alpha(""), "");
}

// ─── decimal_pow10 negative branch in format.rs ───────────────────────────────
// scale_decimal calls decimal_pow10 with positive shift (from -sci_exp where sci_exp<0,
// giving a positive shift). The negative branch is reached when scale_decimal is called
// with a negative shift, which happens in format_eng's "Re-scale mantissa back to absolute
// magnitude" step: scale_decimal(mantissa_rounded, eng_exp) where eng_exp is negative.

#[test]
fn format_eng_sub_unity_exercises_decimal_pow10_negative() {
    // 0.001 in ENG(3): eng_exp = -3, scale_decimal(1.000, -3) = 0.001
    use rust_decimal::Decimal;
    use std::str::FromStr;
    let n = HpNum::from(Decimal::from_str("0.001").unwrap());
    let s = format_hpnum(&n, &DisplayMode::Eng(3));
    // 0.001 = 1.000 × 10^-3 → "1.000E-03"
    assert_eq!(
        s, "1.000E-03",
        "0.001 in ENG(3) must be '1.000E-03', got {s:?}"
    );
}

#[test]
fn format_eng_small_negative_exponent_value() {
    // 0.00042 in ENG(2): 420.00 × 10^-6 → "420.E-06" or "420.00E-06"
    use rust_decimal::Decimal;
    use std::str::FromStr;
    let n = HpNum::from(Decimal::from_str("0.00042").unwrap());
    let s = format_hpnum(&n, &DisplayMode::Eng(2));
    assert!(
        s.contains("E-"),
        "0.00042 ENG(2) must have negative exponent, got {s:?}"
    );
}

#[test]
fn format_eng_tiny_value_exercises_negative_exp_scale() {
    // 1e-6 in ENG(3) → "1.000E-06"
    use rust_decimal::Decimal;
    use std::str::FromStr;
    let n = HpNum::from(Decimal::from_str("0.000001").unwrap());
    let s = format_hpnum(&n, &DisplayMode::Eng(3));
    assert!(
        s.contains("E-06"),
        "1e-6 in ENG(3) must produce 'E-06', got {s:?}"
    );
}
