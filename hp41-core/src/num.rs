use crate::error::HpError;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;
use rust_decimal::RoundingStrategy;
use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Tagged union for HP-41 register values.
///
/// `HpValue` wraps `HpNum` (numeric) and adds an `Alpha` variant for
/// packed-text data stored by ASTO, replacing the `text_regs` shadow
/// mechanism with first-class type tagging.
///
/// Serde uses `#[serde(untagged)]`: old save files with bare Decimal
/// strings deserialize as `Numeric`; new `[u8; 6]` arrays as `Alpha`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HpValue {
    /// A numeric value (the common case).
    Numeric(HpNum),
    /// Packed-text data -- up to 6 bytes of ALPHA data (ASTO).
    Alpha([u8; 6]),
}

impl HpValue {
    /// Extract the numeric `HpNum`, returning `AlphaData` if Alpha.
    pub fn as_numeric(&self) -> Result<HpNum, HpError> {
        match self {
            HpValue::Numeric(n) => Ok(n.clone()),
            HpValue::Alpha(_) => Err(HpError::AlphaData),
        }
    }

    /// Unwrap to `HpNum` or return `HpNum::zero()` (used in production
    /// code where Alpha registers should be treated as zero).
    pub fn numeric_or_zero(&self) -> HpNum {
        match self {
            HpValue::Numeric(n) => n.clone(),
            HpValue::Alpha(_) => HpNum::zero(),
        }
    }

    /// Returns `true` if Alpha.
    pub fn is_alpha(&self) -> bool {
        matches!(self, HpValue::Alpha(_))
    }

    /// Returns `true` if Numeric.
    pub fn is_numeric(&self) -> bool {
        matches!(self, HpValue::Numeric(_))
    }

    // ── Delegation methods (treat Alpha as zero) ──────────────────────────────
    // These allow production code that formerly called HpNum methods on
    // register values to work without explicit numeric_or_zero() calls.

    /// Inner Decimal value (Alpha -> ZERO).
    ///
    /// For values with exponent == 0 (the common case, all values ≤ ~7.92E28)
    /// this returns the full value exactly as before ADR v4.3-005.
    /// For large-exponent values (exponent != 0) this returns the mantissa in [1, 10).
    pub fn inner(&self) -> Decimal {
        self.numeric_or_zero().inner()
    }

    /// Checked add (Alpha treated as zero).
    pub fn checked_add(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
        self.numeric_or_zero().checked_add(rhs)
    }

    /// Checked sub (Alpha treated as zero).
    pub fn checked_sub(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
        self.numeric_or_zero().checked_sub(rhs)
    }

    /// Checked mul (Alpha treated as zero).
    pub fn checked_mul(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
        self.numeric_or_zero().checked_mul(rhs)
    }

    /// Checked div (Alpha treated as zero).
    pub fn checked_div(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
        self.numeric_or_zero().checked_div(rhs)
    }

    /// Is this value numerically zero? (Alpha -> true)
    pub fn is_zero(&self) -> bool {
        self.numeric_or_zero().is_zero()
    }

    /// Negate (Alpha -> zero).
    pub fn negate(&self) -> HpNum {
        self.numeric_or_zero().negate()
    }

    /// Truncate to integer part (Alpha -> zero).
    pub fn trunc_int(&self) -> HpNum {
        self.numeric_or_zero().trunc_int()
    }

    /// Checked square (Alpha treated as zero).
    pub fn checked_sq(&self) -> Result<HpNum, HpError> {
        self.numeric_or_zero().checked_sq()
    }
}

impl Default for HpValue {
    fn default() -> Self {
        HpValue::Numeric(HpNum::zero())
    }
}

impl From<HpNum> for HpValue {
    fn from(n: HpNum) -> Self {
        HpValue::Numeric(n)
    }
}

impl From<i32> for HpValue {
    fn from(n: i32) -> Self {
        HpValue::Numeric(HpNum::from(n))
    }
}

/// HP-41 numeric value with full hardware range ±9.999999999E±99.
///
/// Represents `mantissa × 10^exponent` where:
///
/// **Common case (exponent == 0, all values ≤ ~7.92E28):**
/// `mantissa` holds the full decimal value with up to 10 significant digits.
/// This is backward-compatible with the pre-ADR v4.3-005 `HpNum(Decimal)` representation.
/// `inner()` returns the full value exactly as before.
///
/// **Large-exponent case (exponent != 0, values above ~7.92E28 or below ~1E-99):**
/// `mantissa` is normalized to `[1, 10)` for nonzero values (10 significant digits).
/// The full value is `mantissa × 10^exponent`.
/// Use `to_f64()` for the full numeric value.
///
/// This two-tier design ensures all existing code using `inner()` continues to work
/// correctly for values that fit in the Decimal range (the 99%+ case).
///
/// ADR v4.3-005 documents the representation choice and amends the Frozen Invariant.
#[derive(Clone, Debug, PartialEq)]
pub struct HpNum {
    /// For exponent == 0: the full decimal value (up to 10 sig digits).
    /// For exponent != 0: normalized mantissa in [1, 10) (10 sig digits).
    pub(crate) mantissa: Decimal,
    /// Exponent: -99..=99. For exponent == 0 (common case), the full value is in mantissa.
    /// The full value is `mantissa × 10^exponent`.
    pub(crate) exponent: i8,
}

// ─── Private serde helper ──────────────────────────────────────────────────────

/// Wire representation for `HpNum` deserialization.
///
/// `#[serde(untagged)]` with `Extended` arm FIRST (serde tries arms in order).
/// This allows backward-compatible loading of legacy v1.0–v4.2 save files
/// that stored `HpNum` as a bare Decimal string (e.g. `"3.1415926536"`).
///
/// New saves use the `Extended { m, e }` object form.
#[derive(Deserialize)]
#[serde(untagged)]
enum HpNumWire {
    /// New format (arm FIRST per Pitfall 2): `{ "m": "1.234567890", "e": 45 }`
    Extended { m: String, e: i8 },
    /// Legacy format (arm SECOND): bare decimal string `"1.234567890"`
    Legacy(String),
}

impl Serialize for HpNum {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("m", &self.mantissa.to_string())?;
        map.serialize_entry("e", &self.exponent)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for HpNum {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        match HpNumWire::deserialize(de)? {
            HpNumWire::Extended { m, e } => {
                let mantissa =
                    Decimal::from_str(&m).map_err(serde::de::Error::custom)?;
                Ok(HpNum { mantissa, exponent: e })
            }
            HpNumWire::Legacy(s) => {
                let d = Decimal::from_str(&s).map_err(serde::de::Error::custom)?;
                // Legacy format: always produces exponent=0 (in-range Decimal).
                Ok(HpNum::from_decimal(d))
            }
        }
    }
}

// ─── Core impl ────────────────────────────────────────────────────────────────

impl HpNum {
    /// Enforce HP-41 10-significant-digit precision with round-half-away-from-zero.
    ///
    /// **Common path (exponent == 0):** rounds the Decimal mantissa to 10 sig digits.
    /// This is the canonical constructor for in-range Decimal values and matches the
    /// behavior of the pre-ADR v4.3-005 `HpNum::rounded(d: Decimal)` method.
    ///
    /// This matches HP-41 hardware display rounding (NOT Bankers/MidpointNearestEven).
    pub fn rounded(d: Decimal) -> Self {
        HpNum::from_decimal(d)
    }

    /// Construct an `HpNum` from a `Decimal` value (for in-range values, exponent == 0).
    ///
    /// Rounds the Decimal to 10 significant digits and stores with `exponent = 0`.
    /// Trailing zeros are normalized away so `inner().to_string()` is compact
    /// (e.g., `-3` not `-3.000000000`).
    ///
    /// Used for legacy serde deserialization, `From<Decimal>` / `From<i32>`, and all
    /// arithmetic paths where the result is within the Decimal representable range.
    ///
    /// For the common case, `inner()` on the result returns the full value (unchanged
    /// from pre-ADR v4.3-005 behavior).
    pub fn from_decimal(d: Decimal) -> Self {
        if d.is_zero() {
            return HpNum::zero();
        }
        let rounded = d
            .round_sf_with_strategy(10, RoundingStrategy::MidpointAwayFromZero)
            .expect("round_sf_with_strategy(10) must succeed for valid finite Decimal");
        HpNum {
            mantissa: rounded,
            exponent: 0,
        }
    }

    /// Construct a large-exponent `HpNum` from a scientific-notation decomposition.
    ///
    /// `mantissa` should be in [1, 10) for nonzero values.
    /// This is for large values (above the Decimal ceiling ~7.92E28) only.
    ///
    /// - mantissa.is_zero() → `HpNum::zero()` (Pitfall 1: guard before round_sf).
    /// - Rounds mantissa to 10 significant digits.
    /// - exponent > 99  → `Err(HpError::Overflow)`.
    /// - exponent < -99 → `Ok(HpNum::zero())` (underflow to zero — hardware-faithful).
    pub(crate) fn from_sci(mantissa: Decimal, exponent: i8) -> Result<HpNum, HpError> {
        if mantissa.is_zero() {
            return Ok(HpNum::zero());
        }
        if exponent > 99 {
            return Err(HpError::Overflow);
        }
        if exponent < -99 {
            return Ok(HpNum::zero()); // underflow
        }

        // Round to 10 significant digits.
        let rounded = mantissa
            .round_sf_with_strategy(10, RoundingStrategy::MidpointAwayFromZero)
            .expect("round_sf_with_strategy(10) must succeed for valid finite Decimal");

        if rounded.is_zero() {
            return Ok(HpNum::zero());
        }

        // Handle carry: if rounding pushed mantissa to 10.0 or higher, increment exponent.
        let abs_rounded = rounded.abs();
        if abs_rounded >= Decimal::from(10) {
            let new_exp = (exponent as i32) + 1;
            if new_exp > 99 {
                return Err(HpError::Overflow);
            }
            let carried = rounded
                .checked_div(Decimal::from(10))
                .expect("division by 10 cannot fail");
            let carried_final = carried
                .round_sf_with_strategy(10, RoundingStrategy::MidpointAwayFromZero)
                .expect("round_sf(10) after carry must succeed");
            return Ok(HpNum {
                mantissa: carried_final,
                exponent: new_exp as i8,
            });
        }

        Ok(HpNum { mantissa: rounded, exponent })
    }

    /// Construct an `HpNum` from an `f64` value (factorial / transcendental bridge).
    ///
    /// Mirrors the `checked_asin` f64-bridge pattern: extracts the base-10 exponent
    /// via `log10`, constructs the mantissa as a Decimal, then uses `from_sci`.
    /// Returns `None` if `acc` is NaN, infinite, or the exponent exceeds 99.
    ///
    /// Values where |acc| ≤ ~7.92E28 (within Decimal range) will have mantissa = full
    /// value and exponent = 0 via the from_decimal path, preserving inner() compat.
    pub fn from_f64(acc: f64) -> Option<HpNum> {
        if acc == 0.0 {
            return Some(HpNum::zero());
        }
        if !acc.is_finite() {
            return None;
        }

        // Fast path: if the value fits in Decimal range (~7.92E28), use from_decimal.
        // This preserves exponent == 0 for in-range values and backward compatibility
        // with inner() returning the full value.
        if let Some(d) = Decimal::from_f64(acc) {
            return Some(HpNum::from_decimal(d));
        }

        // Large value (above ~7.92E28): use scientific notation decomposition.
        let abs_acc = acc.abs();
        let exp_f = abs_acc.log10().floor();
        if exp_f > 99.0 || exp_f < -99.0 {
            return None;
        }
        let exp = exp_f as i32;
        let mantissa_f = acc / 10f64.powi(exp);
        let mantissa = Decimal::from_f64(mantissa_f)?;
        HpNum::from_sci(mantissa, exp as i8).ok()
    }

    pub fn zero() -> Self {
        HpNum {
            mantissa: Decimal::ZERO,
            exponent: 0,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.mantissa.is_zero()
    }

    /// Returns the full value as `f64`.
    ///
    /// For values with exponent == 0, equivalent to `inner().to_f64()`.
    /// For large-exponent values, reconstructs the full value: `mantissa × 10^exponent`.
    pub fn to_f64(&self) -> Option<f64> {
        if self.exponent == 0 {
            return self.mantissa.to_f64();
        }
        let m = self.mantissa.to_f64()?;
        Some(m * 10f64.powi(self.exponent as i32))
    }

    // ── Arithmetic ─────────────────────────────────────────────────────────────

    pub fn checked_add(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
        // Common case: both exponents are 0 → delegate to Decimal arithmetic.
        if self.exponent == 0 && rhs.exponent == 0 {
            return self.mantissa
                .checked_add(rhs.mantissa)
                .map(HpNum::rounded)
                .ok_or(HpError::Overflow);
        }
        // Large-exponent path: work in scientific notation.
        self.checked_add_sci(rhs)
    }

    /// Add for large-exponent operands (at least one has exponent != 0).
    fn checked_add_sci(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
        if self.is_zero() {
            return Ok(rhs.clone());
        }
        if rhs.is_zero() {
            return Ok(self.clone());
        }

        // Normalize both operands to scientific notation form.
        let (m_l, e_l) = self.to_sci();
        let (m_r, e_r) = rhs.to_sci();

        let exp_diff = e_l - e_r;

        // If exponent difference ≥ 10, the smaller value is beyond 10 sig digits —
        // hardware-faithful underflow (sum ≈ the larger operand).
        if exp_diff >= 10 {
            let (lm, le) = if e_l >= e_r { (m_l, e_l) } else { (m_r, e_r) };
            return HpNum::from_sci(lm, le as i8);
        }
        if exp_diff <= -10 {
            let (lm, le) = if e_r >= e_l { (m_r, e_r) } else { (m_l, e_l) };
            return HpNum::from_sci(lm, le as i8);
        }

        // Align to the same exponent for addition.
        let base_exp = e_l.min(e_r);
        let (aligned_l, aligned_r) = if exp_diff >= 0 {
            // self has larger exponent: scale rhs up by 10^exp_diff
            let scale = decimal_pow10_small(exp_diff as u32);
            let scaled = m_r.checked_mul(scale).ok_or(HpError::Overflow)?;
            (m_l, scaled)
        } else {
            // rhs has larger exponent: scale self up by 10^(-exp_diff)
            let scale = decimal_pow10_small((-exp_diff) as u32);
            let scaled = m_l.checked_mul(scale).ok_or(HpError::Overflow)?;
            (scaled, m_r)
        };

        let sum = aligned_l.checked_add(aligned_r).ok_or(HpError::Overflow)?;
        HpNum::from_sci(sum, base_exp as i8)
    }

    /// Convert self to (mantissa, exponent) in proper [1,10) scientific form.
    /// For exponent==0 values, compute the mantissa normalization.
    fn to_sci(&self) -> (Decimal, i32) {
        if self.exponent != 0 {
            return (self.mantissa, self.exponent as i32);
        }
        if self.is_zero() {
            return (Decimal::ZERO, 0);
        }
        // exponent == 0: mantissa IS the full value. Normalize to [1,10) form.
        let abs_m = self.mantissa.abs();
        let f = abs_m.to_f64().expect("exponent-0 mantissa must be representable as f64");
        let exp = if f >= 1.0 { f.log10().floor() as i32 } else if f > 0.0 { -((-f.log10()).ceil() as i32) } else { 0 };
        let scale = decimal_pow10_f64(exp);
        let normalized = self.mantissa.checked_div(scale)
            .unwrap_or(self.mantissa); // safe: scale is always nonzero
        (normalized, exp)
    }

    pub fn checked_sub(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
        self.checked_add(&rhs.negate())
    }

    pub fn checked_mul(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
        // Common case: both exponents are 0 → delegate to Decimal arithmetic.
        if self.exponent == 0 && rhs.exponent == 0 {
            return self.mantissa
                .checked_mul(rhs.mantissa)
                .map(HpNum::rounded)
                .ok_or(HpError::Overflow);
        }
        // Large-exponent path.
        if self.is_zero() || rhs.is_zero() {
            return Ok(HpNum::zero());
        }
        let (m_l, e_l) = self.to_sci();
        let (m_r, e_r) = rhs.to_sci();
        // Mantissas in [1, 10): product in [1, 100) — always Decimal-representable.
        let product = m_l.checked_mul(m_r).ok_or(HpError::Overflow)?;
        let new_exp = e_l + e_r;
        HpNum::from_sci(product, new_exp as i8)
    }

    pub fn checked_div(&self, rhs: &HpNum) -> Result<HpNum, HpError> {
        if rhs.is_zero() {
            return Err(HpError::DivideByZero);
        }
        // Common case: both exponents are 0 → delegate to Decimal arithmetic.
        if self.exponent == 0 && rhs.exponent == 0 {
            return self.mantissa
                .checked_div(rhs.mantissa)
                .map(HpNum::rounded)
                .ok_or(HpError::Overflow);
        }
        if self.is_zero() {
            return Ok(HpNum::zero());
        }
        let (m_l, e_l) = self.to_sci();
        let (m_r, e_r) = rhs.to_sci();
        let quotient = m_l.checked_div(m_r).ok_or(HpError::Overflow)?;
        let new_exp = e_l - e_r;
        HpNum::from_sci(quotient, new_exp as i8)
    }

    // ── Scalar math methods ───────────────────────────────────────────────────

    /// 1/x — reciprocal of self.
    /// Returns DivideByZero if self is zero.
    /// LiftEffect declared by caller (Enable for op_recip).
    pub fn checked_recip(&self) -> Result<HpNum, HpError> {
        HpNum::from(1).checked_div(self) // reuses existing DivideByZero guard in checked_div
    }

    /// √x — square root of self.
    /// Returns Domain if self < 0.
    pub fn checked_sqrt(&self) -> Result<HpNum, HpError> {
        if self.mantissa < Decimal::ZERO {
            return Err(HpError::Domain);
        }
        if self.is_zero() {
            return Ok(HpNum::zero());
        }
        if self.exponent == 0 {
            // rust_decimal MathematicalOps provides sqrt() returning Option<Decimal>
            return self.mantissa.sqrt().map(HpNum::rounded).ok_or(HpError::Overflow);
        }
        // Large-exponent path: use f64 bridge.
        let full_val = self.to_f64().ok_or(HpError::Overflow)?;
        HpNum::from_f64(full_val.sqrt()).ok_or(HpError::Domain)
    }

    /// x² — self multiplied by self.
    /// Uses checked_mul — no maths feature dependency.
    pub fn checked_sq(&self) -> Result<HpNum, HpError> {
        self.checked_mul(self)
    }

    /// LN — natural logarithm of self.
    /// Returns Domain if self ≤ 0.
    pub fn checked_ln(&self) -> Result<HpNum, HpError> {
        if self.mantissa <= Decimal::ZERO {
            return Err(HpError::Domain);
        }
        if self.exponent == 0 {
            return self.mantissa
                .checked_ln()
                .map(HpNum::rounded)
                .ok_or(HpError::Overflow);
        }
        // LN(m × 10^e) = LN(m) + e × LN(10).
        let ln_m = self.mantissa.checked_ln().ok_or(HpError::Overflow)?;
        // ln(10) = 2.302585093 (10 sig digits)
        let ln_10 = Decimal::from_str("2.302585093")
            .expect("ln(10) literal must parse");
        let e_term = Decimal::from(self.exponent)
            .checked_mul(ln_10)
            .ok_or(HpError::Overflow)?;
        let total = ln_m.checked_add(e_term).ok_or(HpError::Overflow)?;
        Ok(HpNum::rounded(total))
    }

    /// LOG — log base 10 of self.
    /// Returns Domain if self ≤ 0.
    pub fn checked_log10(&self) -> Result<HpNum, HpError> {
        if self.mantissa <= Decimal::ZERO {
            return Err(HpError::Domain);
        }
        if self.exponent == 0 {
            return self.mantissa
                .checked_log10()
                .map(HpNum::rounded)
                .ok_or(HpError::Overflow);
        }
        // LOG10(m × 10^e) = LOG10(m) + e.
        let log_m = self.mantissa.checked_log10().ok_or(HpError::Overflow)?;
        let e_dec = Decimal::from(self.exponent);
        let total = log_m.checked_add(e_dec).ok_or(HpError::Overflow)?;
        Ok(HpNum::rounded(total))
    }

    /// e^x — natural exponential of self.
    pub fn checked_exp(&self) -> Result<HpNum, HpError> {
        if self.exponent != 0 {
            let v = self.to_f64().ok_or(HpError::Overflow)?;
            let result = v.exp();
            return HpNum::from_f64(result).ok_or(HpError::Overflow);
        }
        self.mantissa
            .checked_exp()
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    /// 10^x — base-10 exponential of self.
    pub fn checked_exp10(&self) -> Result<HpNum, HpError> {
        if self.exponent != 0 {
            let v = self.to_f64().ok_or(HpError::Overflow)?;
            let result = 10f64.powf(v);
            return HpNum::from_f64(result).ok_or(HpError::Overflow);
        }
        Decimal::from(10)
            .checked_powd(self.mantissa)
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    /// Y^X — self raised to the power of exp.
    /// Returns Domain if self < 0 and exp has a non-zero fractional part
    /// (complex result — HP-41 returns INVALID DATA in this case).
    pub fn checked_powd(&self, exp: &HpNum) -> Result<HpNum, HpError> {
        if self.mantissa.is_sign_negative() && !exp.mantissa.fract().is_zero() {
            return Err(HpError::Domain);
        }
        // For in-range values (both exponents == 0), delegate to Decimal MathematicalOps.
        if self.exponent == 0 && exp.exponent == 0 {
            return self.mantissa
                .checked_powd(exp.mantissa)
                .map(HpNum::rounded)
                .ok_or(HpError::Domain);
        }
        // For large-exponent values, use f64 bridge.
        let base_f = self.to_f64().ok_or(HpError::Overflow)?;
        let exp_f = exp.to_f64().ok_or(HpError::Overflow)?;
        HpNum::from_f64(base_f.powf(exp_f)).ok_or(HpError::Domain)
    }

    /// %CH — percent change from self (base, Y) to new_val (the new value, X).
    /// Computes `((new_val − self) / self) × 100`.
    /// Returns `DivideByZero` if self is zero; `Overflow` on intermediate or final overflow.
    pub fn checked_pct_change(&self, new_val: &HpNum) -> Result<HpNum, HpError> {
        let delta = new_val.checked_sub(self)?;
        let ratio = delta.checked_div(self)?; // DivideByZero if self == 0
        ratio.checked_mul(&HpNum::from(100i32))
    }

    // ── Trigonometric methods (angle in RADIANS) ──────────────────────────────
    // All trig methods expect/return values in radians.
    // Angle mode conversion (DEG/GRAD ↔ RAD) is the caller's responsibility.
    //
    // Note: for trig, the input is always in-range (angles are in radians/degrees/grads
    // far from the Decimal ceiling). These methods use self.mantissa directly (same as
    // before ADR v4.3-005 since all trig inputs have exponent == 0).

    /// sin(x) — x must be in radians. Uses rust_decimal MathematicalOps (Maclaurin series).
    pub fn checked_sin(&self) -> Result<HpNum, HpError> {
        self.mantissa
            .checked_sin()
            .map(HpNum::rounded)
            .ok_or(HpError::Domain)
    }

    /// cos(x) — x must be in radians. Uses rust_decimal MathematicalOps.
    pub fn checked_cos(&self) -> Result<HpNum, HpError> {
        self.mantissa
            .checked_cos()
            .map(HpNum::rounded)
            .ok_or(HpError::Domain)
    }

    /// tan(x) — x must be in radians. Returns Domain at tan(π/2) etc.
    pub fn checked_tan(&self) -> Result<HpNum, HpError> {
        self.mantissa
            .checked_tan()
            .map(HpNum::rounded)
            .ok_or(HpError::Domain)
    }

    // ── Inverse trig — f64 round-trip bridge ─────────────────────────────────
    // rust_decimal MathematicalOps does not provide asin/acos/atan.
    // f64 has ~15.9 decimal digits of precision; rounding to 10 via HpNum::rounded()
    // is sufficient to meet QUAL-06 (≥98% accuracy at 10 sig digits).

    /// asin(x) — returns result in radians. Domain error if |x| > 1.
    pub fn checked_asin(&self) -> Result<HpNum, HpError> {
        let v = self.mantissa.to_f64().ok_or(HpError::Overflow)?;
        if !(-1.0..=1.0).contains(&v) {
            return Err(HpError::Domain);
        }
        Decimal::from_f64(v.asin())
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    /// acos(x) — returns result in radians. Domain error if |x| > 1.
    pub fn checked_acos(&self) -> Result<HpNum, HpError> {
        let v = self.mantissa.to_f64().ok_or(HpError::Overflow)?;
        if !(-1.0..=1.0).contains(&v) {
            return Err(HpError::Domain);
        }
        Decimal::from_f64(v.acos())
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    /// atan(x) — returns result in radians. No domain restriction.
    pub fn checked_atan(&self) -> Result<HpNum, HpError> {
        let v = self.mantissa.to_f64().ok_or(HpError::Overflow)?;
        Decimal::from_f64(v.atan())
            .map(HpNum::rounded)
            .ok_or(HpError::Overflow)
    }

    pub fn negate(&self) -> Self {
        if self.is_zero() {
            return HpNum::zero();
        }
        HpNum {
            mantissa: -self.mantissa,
            exponent: self.exponent,
        }
    }

    /// Returns the mantissa Decimal.
    ///
    /// For values with exponent == 0 (the common case, all values ≤ ~7.92E28):
    /// this returns the full value — identical to pre-ADR v4.3-005 behavior.
    ///
    /// For large-exponent values (exponent != 0): returns only the normalized
    /// mantissa in [1, 10). Use `to_f64()` for the full numeric value.
    pub fn inner(&self) -> Decimal {
        self.mantissa
    }

    /// INT — truncate toward zero (integer part, HP-41 INT function).
    /// Equivalent to Decimal::trunc() which truncates toward zero.
    /// No domain restriction; always succeeds.
    pub fn trunc_int(&self) -> HpNum {
        if self.is_zero() {
            return HpNum::zero();
        }
        if self.exponent == 0 {
            return HpNum::from_decimal(self.mantissa.trunc());
        }
        // Large-exponent value: the integer part via f64 bridge.
        // For exponent >= 10, all values of this magnitude are integers already.
        if self.exponent >= 10 {
            return self.clone();
        }
        // For exponent in [1,9]: reconstruct, truncate, re-normalize.
        // Value = mantissa × 10^exp where mantissa in [1,10), exp in [1,9].
        // Full value < 10^10 — within Decimal range.
        let v = self.to_f64().expect("large-exponent value in [1..10^10] must be f64-representable");
        HpNum::from_decimal(Decimal::from_f64(v.trunc()).unwrap_or(Decimal::ZERO))
    }
}

/// Compute `10^exp` as a Decimal, for small non-negative exp (0..=9).
/// Used for exponent alignment in large-exponent arithmetic.
fn decimal_pow10_small(exp: u32) -> Decimal {
    match exp {
        0 => Decimal::ONE,
        1 => Decimal::from(10u32),
        2 => Decimal::from(100u32),
        3 => Decimal::from(1_000u32),
        4 => Decimal::from(10_000u32),
        5 => Decimal::from(100_000u32),
        6 => Decimal::from(1_000_000u32),
        7 => Decimal::from(10_000_000u32),
        8 => Decimal::from(100_000_000u32),
        9 => Decimal::from(1_000_000_000u32),
        _ => {
            let mut result = Decimal::ONE;
            let ten = Decimal::from(10u32);
            for _ in 0..exp {
                result = result.checked_mul(ten).expect("decimal_pow10_small: bounded");
            }
            result
        }
    }
}

/// Compute `10^exp` as a Decimal via f64, for arbitrary integer exp.
/// Used for to_sci() normalization (log10-based).
fn decimal_pow10_f64(exp: i32) -> Decimal {
    if exp == 0 {
        return Decimal::ONE;
    }
    if exp > 0 {
        decimal_pow10_small(exp as u32)
    } else {
        // 10^(-|exp|): build as string for precision.
        let abs_exp = (-exp) as usize;
        let s = "0.".to_string() + &"0".repeat(abs_exp - 1) + "1";
        Decimal::from_str(&s).expect("decimal_pow10_f64: valid negative-exp string")
    }
}

impl From<i32> for HpNum {
    fn from(n: i32) -> Self {
        HpNum::from_decimal(Decimal::from(n))
    }
}

impl From<Decimal> for HpNum {
    fn from(d: Decimal) -> Self {
        HpNum::rounded(d)
    }
}

impl std::fmt::Display for HpNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.exponent == 0 {
            // Normalize for display: remove trailing zeros added by round_sf(10).
            // E.g., "7.000000000" → "7". The internal mantissa may carry trailing
            // zeros (needed for HMS/date string parsing), but Display shows compact form.
            write!(f, "{}", self.mantissa.normalize())
        } else {
            write!(f, "{}e{}", self.mantissa, self.exponent)
        }
    }
}

impl Default for HpNum {
    fn default() -> Self {
        HpNum::zero()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ─── Wave-0 unit tests (TDD RED → GREEN for MATH-01) ────────────────────

    #[test]
    fn test_hpnum_serde_is_string() {
        // New format: serializes as {"m":"3","e":0}; round-trips losslessly.
        let n = HpNum::from(3i32);
        let json = serde_json::to_string(&n).unwrap();
        let back: HpNum = serde_json::from_str(&json).unwrap();
        assert_eq!(back, n, "round-trip of small integer must be lossless");

        // Legacy backward-compat: a bare Decimal string from v1.0–v4.2 loads correctly.
        let legacy_json = r#""3.1415926536""#;
        let from_legacy: HpNum = serde_json::from_str(legacy_json).unwrap();
        let expected = HpNum::from(
            rust_decimal::Decimal::from_str("3.1415926536").unwrap(),
        );
        assert_eq!(
            from_legacy, expected,
            "legacy bare Decimal string must deserialize correctly"
        );
    }

    #[test]
    fn test_hpnum_serde_decimal_precision() {
        let d = Decimal::from_str("3.1415926536").unwrap();
        let n = HpNum::from(d);
        let json = serde_json::to_string(&n).unwrap();
        let back: HpNum = serde_json::from_str(&json).unwrap();
        assert_eq!(n, back, "10-digit decimal must round-trip exactly");
    }

    #[test]
    fn test_hpnum_large_exponent_serde_roundtrip() {
        // FACT(69) ≈ 1.711224524e98
        let n = HpNum::from_f64(1.711224524e98).unwrap();
        let json = serde_json::to_string(&n).unwrap();
        let back: HpNum = serde_json::from_str(&json).unwrap();
        assert_eq!(n, back, "large-exponent round-trip must be lossless");
    }

    #[test]
    fn test_hpnum_arithmetic_large_exponent() {
        // 1e40 * 1e40 = 1e80 (well within exp ≤ 99 range)
        let a = HpNum::from_f64(1e40).unwrap();
        let b = HpNum::from_f64(1e40).unwrap();
        let result = a.checked_mul(&b).unwrap();
        assert_eq!(result.exponent, 80, "1e40 * 1e40 should have exponent 80");
        assert!(
            (result.mantissa.to_f64().unwrap() - 1.0).abs() < 1e-9,
            "mantissa should be ~1.0"
        );

        // 9e98 + 9e98 = 1.8e99 (still in range)
        let c = HpNum::from_f64(9e98).unwrap();
        let d = HpNum::from_f64(9e98).unwrap();
        let sum = c.checked_add(&d).unwrap();
        assert_eq!(sum.exponent, 99, "9e98 + 9e98 should have exponent 99");
    }

    #[test]
    fn test_hpnum_overflow_at_exp_99() {
        // 9.999999999e99 * 2 → exponent would be 100 → Overflow
        let a = HpNum::from_f64(9.999999999e99).unwrap();
        let b = HpNum::from(2i32);
        assert_eq!(
            a.checked_mul(&b),
            Err(HpError::Overflow),
            "product exceeding exp=99 must return Overflow"
        );
    }

    #[test]
    fn test_hpnum_underflow_to_zero() {
        // Underflow: exponent < -99 should produce zero.
        let result = HpNum::from_sci(Decimal::from_str("1.0").unwrap(), -100i8);
        assert_eq!(
            result.unwrap(),
            HpNum::zero(),
            "exponent < -99 must underflow to zero"
        );
    }

    #[test]
    fn test_hpnum_zero_normalize_no_panic() {
        // Pitfall 1: from_sci(Decimal::ZERO, anything) must return zero, never panic.
        let result = HpNum::from_sci(Decimal::ZERO, 50);
        assert_eq!(result.unwrap(), HpNum::zero(), "zero mantissa must normalize to zero");

        let result2 = HpNum::from_sci(Decimal::ZERO, -50);
        assert_eq!(result2.unwrap(), HpNum::zero(), "zero mantissa with negative exp must also be zero");
    }

    #[test]
    fn test_hpnum_inner_compat_small_values() {
        // For values with exponent==0, inner() must return the full value — compat test.
        let n = HpNum::from(42i32);
        assert_eq!(n.exponent, 0, "integer 42 must have exponent 0");
        assert_eq!(n.inner(), Decimal::from(42), "inner() of 42 must be 42");

        let n2 = HpNum::from(Decimal::from_str("3.1415926536").unwrap());
        assert_eq!(n2.exponent, 0, "pi must have exponent 0");
        // 3.1415926536 has 11 sig digits; rounds to 10 sig digits = 3.141592654
        assert_eq!(n2.inner(), Decimal::from_str("3.141592654").unwrap());
    }
}
