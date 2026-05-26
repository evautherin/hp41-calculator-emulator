// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `curve_fit` — ADV MATH: curve fitting operations.
//!
//! XROM module id 24 (ADV_MATH_B, bit-4 of `CalcState::xrom_modules`).
//!
//! Operations: CFIT / AS / DS / BFIT / FIT / Y?X / SZ?
//!
//! ## Register layout (ADV curve-fit block, R10–R19)
//!
//! | Register | Constant           | Contents                              |
//! |----------|--------------------|---------------------------------------|
//! | R10      | `CFIT_N_REG`       | n (sample count)                      |
//! | R11      | `CFIT_SUM_X_REG`   | Σx (or Σln(x) for log/power models)  |
//! | R12      | `CFIT_SUM_Y_REG`   | Σy (or Σln(y) for exp/power models)  |
//! | R13      | `CFIT_SUM_X2_REG`  | Σx² (or Σ(ln x)²)                    |
//! | R14      | `CFIT_SUM_XY_REG`  | Σxy (or Σln(x)·ln(y))                |
//! | R15      | `CFIT_SUM_Y2_REG`  | Σy² (or Σ(ln y)²)                    |
//! | R16      | `CFIT_MODEL_REG`   | active model: 0=linear, 1=log, 2=exp, 3=power |
//! | R17      | `CFIT_COEFF_A_REG` | intercept a (after FIT)               |
//! | R18      | `CFIT_COEFF_B_REG` | slope b (after FIT)                   |
//! | R19      | `CFIT_CORR_REG`    | correlation r (after FIT)             |
//!
//! ## Model formulas
//!
//! | Model  | Formula           | Log-linearization                  |
//! |--------|-------------------|------------------------------------|
//! | Linear | ŷ = a + b·x       | identity                           |
//! | Log    | ŷ = a + b·ln(x)  | x' = ln(x)                         |
//! | Exp    | ŷ = a·e^(b·x)    | y' = ln(y); a_final = e^intercept  |
//! | Power  | ŷ = a·x^b         | x' = ln(x), y' = ln(y)             |
//!
//! Source: HP Advantage Pac Owner's Manual 00041-90482 §4 Curve Fitting.

use crate::{
    error::HpError,
    num::HpNum,
    stack::{apply_lift_effect, enter_number, LiftEffect},
    state::CalcState,
};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

// ── Register index constants ───────────────────────────────────────────────

/// Sample count n.
pub const CFIT_N_REG: usize = 10;
/// Σx (or Σln x for log/power models).
pub const CFIT_SUM_X_REG: usize = 11;
/// Σy (or Σln y for exp/power models).
pub const CFIT_SUM_Y_REG: usize = 12;
/// Σx² (or Σ(ln x)²).
pub const CFIT_SUM_X2_REG: usize = 13;
/// Σxy cross-product (transformed as required).
pub const CFIT_SUM_XY_REG: usize = 14;
/// Σy² (or Σ(ln y)²).
pub const CFIT_SUM_Y2_REG: usize = 15;
/// Active curve-fit model: 0=linear, 1=log, 2=exp, 3=power.
pub const CFIT_MODEL_REG: usize = 16;
/// Fit coefficient a (intercept; or scale factor for exp/power).
pub const CFIT_COEFF_A_REG: usize = 17;
/// Fit coefficient b (slope).
pub const CFIT_COEFF_B_REG: usize = 18;
/// Pearson correlation r from the last FIT call.
pub const CFIT_CORR_REG: usize = 19;

/// Highest register index used by the ADV curve-fit block (R19).
/// Callers use this for SIZE-floor checks.
pub const CFIT_MAX_REG: usize = CFIT_CORR_REG;

// ── Model discriminants ────────────────────────────────────────────────────

const MODEL_LINEAR: i32 = 0;
const MODEL_LOG: i32 = 1;
const MODEL_EXP: i32 = 2;
const MODEL_POWER: i32 = 3;

// ── Internal helpers ───────────────────────────────────────────────────────

/// SIZE-floor guard: fails if regs cannot address CFIT_MAX_REG.
#[inline]
fn require_cfit_size(state: &CalcState) -> Result<(), HpError> {
    if state.regs.len() <= CFIT_MAX_REG {
        return Err(HpError::InvalidOp);
    }
    Ok(())
}

/// Read a register as f64 (numeric_or_zero for Alpha values).
#[inline]
fn reg_f64(state: &CalcState, idx: usize) -> f64 {
    state.regs[idx]
        .numeric_or_zero()
        .inner()
        .to_f64()
        .unwrap_or(0.0)
}

/// Write an f64 value to a register.
fn set_reg_f64(state: &mut CalcState, idx: usize, v: f64) -> Result<(), HpError> {
    let dec = Decimal::from_f64(v).ok_or(HpError::Overflow)?;
    state.regs[idx] = HpNum::rounded(dec).into();
    Ok(())
}

/// Convert HpNum to f64 via Decimal inner.
#[inline]
fn hpnum_to_f64(n: &HpNum) -> Result<f64, HpError> {
    n.inner().to_f64().ok_or(HpError::Overflow)
}

/// Build an HpNum from f64. Returns Overflow if value is NaN/inf.
#[inline]
fn f64_to_hpnum(v: f64) -> Result<HpNum, HpError> {
    Decimal::from_f64(v)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)
}

/// Apply log-linearization transforms to raw (x, y) for the current model.
///
/// Returns `(x', y')` after any required transforms, or an error if the
/// transforms are not defined for the given input (e.g. ln(0) or ln(negative)).
fn transform_xy(model: i32, x: f64, y: f64) -> Result<(f64, f64), HpError> {
    match model {
        m if m == MODEL_LINEAR => Ok((x, y)),
        m if m == MODEL_LOG => {
            if x <= 0.0 {
                return Err(HpError::Domain);
            }
            Ok((x.ln(), y))
        }
        m if m == MODEL_EXP => {
            if y <= 0.0 {
                return Err(HpError::Domain);
            }
            Ok((x, y.ln()))
        }
        m if m == MODEL_POWER => {
            if x <= 0.0 || y <= 0.0 {
                return Err(HpError::Domain);
            }
            Ok((x.ln(), y.ln()))
        }
        _ => Err(HpError::InvalidOp),
    }
}

/// Compute the Pearson correlation and OLS slope/intercept from the
/// accumulated sums (n, Σx, Σy, Σx², Σxy, Σy²).
///
/// Returns `(slope_b, intercept_a, correlation_r)`.
///
/// # Errors
///
/// - `HpError::InvalidOp` if n < 2.
/// - `HpError::DivideByZero` if denominator is zero (all x values identical).
fn compute_fit(
    n: f64,
    sum_x: f64,
    sum_y: f64,
    sum_x2: f64,
    sum_xy: f64,
    sum_y2: f64,
) -> Result<(f64, f64, f64), HpError> {
    if n < 2.0 {
        return Err(HpError::InvalidOp);
    }

    // SS_xx = Σx² − (Σx)²/n
    let ss_xx = sum_x2 - sum_x * sum_x / n;
    // SS_yy = Σy² − (Σy)²/n
    let ss_yy = sum_y2 - sum_y * sum_y / n;
    // SS_xy = Σxy − (Σx)(Σy)/n
    let ss_xy = sum_xy - sum_x * sum_y / n;

    if ss_xx.abs() < 1e-15 {
        return Err(HpError::DivideByZero);
    }

    let b = ss_xy / ss_xx;
    let a = (sum_y - b * sum_x) / n;

    // Pearson r = SS_xy / sqrt(SS_xx * SS_yy)
    let r = if ss_xx.abs() < 1e-15 || ss_yy.abs() < 1e-15 {
        0.0
    } else {
        let denom = (ss_xx * ss_yy).abs().sqrt();
        if denom < 1e-15 {
            0.0
        } else {
            ss_xy / denom
        }
    };

    Ok((b, a, r))
}

// ── Public op implementations ──────────────────────────────────────────────

/// ADV CFIT — clear curve-fit accumulation registers R10–R19 and reset
/// to linear model (model=0).
///
/// LiftEffect: Neutral.
///
/// # Errors
///
/// Returns `HpError::InvalidOp` if `state.regs.len() <= CFIT_MAX_REG`.
pub fn op_adv_cfit(state: &mut CalcState) -> Result<(), HpError> {
    require_cfit_size(state)?;
    for idx in CFIT_N_REG..=CFIT_MAX_REG {
        state.regs[idx] = HpNum::zero().into();
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV AS — add a (X, Y) data point to curve-fit accumulators.
///
/// Reads X (x-value) and Y (y-value) from the stack. Applies
/// log-linearization transforms for the current model, then accumulates
/// into R10–R15. Stack: Y is consumed (stack drops), count n pushed to X.
///
/// # Errors
///
/// - `HpError::InvalidOp` on SIZE-floor or unknown model.
/// - `HpError::Domain` if transform is undefined for the given input.
pub fn op_adv_as(state: &mut CalcState) -> Result<(), HpError> {
    require_cfit_size(state)?;

    let x_raw = hpnum_to_f64(&state.stack.x)?;
    let y_raw = hpnum_to_f64(&state.stack.y)?;

    let model = reg_f64(state, CFIT_MODEL_REG) as i32;
    let (x, y) = transform_xy(model, x_raw, y_raw)?;

    // Accumulate all terms atomically (read before write)
    let n = reg_f64(state, CFIT_N_REG) + 1.0;
    let sum_x = reg_f64(state, CFIT_SUM_X_REG) + x;
    let sum_y = reg_f64(state, CFIT_SUM_Y_REG) + y;
    let sum_x2 = reg_f64(state, CFIT_SUM_X2_REG) + x * x;
    let sum_xy = reg_f64(state, CFIT_SUM_XY_REG) + x * y;
    let sum_y2 = reg_f64(state, CFIT_SUM_Y2_REG) + y * y;

    set_reg_f64(state, CFIT_N_REG, n)?;
    set_reg_f64(state, CFIT_SUM_X_REG, sum_x)?;
    set_reg_f64(state, CFIT_SUM_Y_REG, sum_y)?;
    set_reg_f64(state, CFIT_SUM_X2_REG, sum_x2)?;
    set_reg_f64(state, CFIT_SUM_XY_REG, sum_xy)?;
    set_reg_f64(state, CFIT_SUM_Y2_REG, sum_y2)?;

    // Push n to X (stack drops Y — binary-style stack drop)
    let n_hp = f64_to_hpnum(n)?;
    state.stack.y = state.stack.z.clone();
    state.stack.z = state.stack.t.clone();
    state.stack.lift_enabled = true;
    state.stack.x = n_hp;
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV DS — remove a (X, Y) data point from curve-fit accumulators (inverse of AS).
///
/// # Errors
///
/// - `HpError::InvalidOp` on SIZE-floor or unknown model.
/// - `HpError::Domain` if transform is undefined for the given input.
pub fn op_adv_ds(state: &mut CalcState) -> Result<(), HpError> {
    require_cfit_size(state)?;

    let x_raw = hpnum_to_f64(&state.stack.x)?;
    let y_raw = hpnum_to_f64(&state.stack.y)?;

    let model = reg_f64(state, CFIT_MODEL_REG) as i32;
    let (x, y) = transform_xy(model, x_raw, y_raw)?;

    let n = reg_f64(state, CFIT_N_REG) - 1.0;
    let sum_x = reg_f64(state, CFIT_SUM_X_REG) - x;
    let sum_y = reg_f64(state, CFIT_SUM_Y_REG) - y;
    let sum_x2 = reg_f64(state, CFIT_SUM_X2_REG) - x * x;
    let sum_xy = reg_f64(state, CFIT_SUM_XY_REG) - x * y;
    let sum_y2 = reg_f64(state, CFIT_SUM_Y2_REG) - y * y;

    set_reg_f64(state, CFIT_N_REG, n.max(0.0))?;
    set_reg_f64(state, CFIT_SUM_X_REG, sum_x)?;
    set_reg_f64(state, CFIT_SUM_Y_REG, sum_y)?;
    set_reg_f64(state, CFIT_SUM_X2_REG, sum_x2)?;
    set_reg_f64(state, CFIT_SUM_XY_REG, sum_xy)?;
    set_reg_f64(state, CFIT_SUM_Y2_REG, sum_y2)?;

    // Push n to X (stack drops Y)
    let n_hp = f64_to_hpnum(n.max(0.0))?;
    state.stack.y = state.stack.z.clone();
    state.stack.z = state.stack.t.clone();
    state.stack.lift_enabled = true;
    state.stack.x = n_hp;
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV BFIT — select the best-fitting model by correlation |r|.
///
/// Computes the correlation for the currently-accumulated sums (sums stored
/// in R10–R15 under the active model's log-linearization). Pushes |r| to X.
///
/// Note: with a single accumulator block, BFIT evaluates r on the stored
/// linearized data. The user is expected to have accumulated data under each
/// model separately before calling BFIT for comparison (per OM §4.5 convention).
/// In this implementation, BFIT reports |r| of the current stored accumulator
/// and retains the current model index in R16.
///
/// LiftEffect: Enable.
///
/// # Errors
///
/// - `HpError::InvalidOp` on SIZE-floor or if n < 2.
pub fn op_adv_bfit(state: &mut CalcState) -> Result<(), HpError> {
    require_cfit_size(state)?;

    let n = reg_f64(state, CFIT_N_REG);
    if n < 2.0 {
        return Err(HpError::InvalidOp);
    }

    let sum_x = reg_f64(state, CFIT_SUM_X_REG);
    let sum_y = reg_f64(state, CFIT_SUM_Y_REG);
    let sum_x2 = reg_f64(state, CFIT_SUM_X2_REG);
    let sum_xy = reg_f64(state, CFIT_SUM_XY_REG);
    let sum_y2 = reg_f64(state, CFIT_SUM_Y2_REG);

    let (_, _, r) = compute_fit(n, sum_x, sum_y, sum_x2, sum_xy, sum_y2)?;

    let abs_r = r.abs();
    let result = f64_to_hpnum(abs_r)?;
    enter_number(state, result);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV FIT — compute OLS fit coefficients for the current model.
///
/// Reads accumulated sums from R10–R15, computes slope b and intercept a
/// using OLS. Stores a in R17, b in R18, r in R19. Pushes b to Y, a to X.
///
/// For exp/power models, `a_raw` is the log-space intercept; the
/// back-transformed scale factor `a_final = e^a_raw` is stored in R17.
///
/// LiftEffect: Enable.
///
/// # Errors
///
/// - `HpError::InvalidOp` on SIZE-floor or n < 2.
/// - `HpError::DivideByZero` if all x values are identical.
pub fn op_adv_fit(state: &mut CalcState) -> Result<(), HpError> {
    require_cfit_size(state)?;

    let n = reg_f64(state, CFIT_N_REG);
    let sum_x = reg_f64(state, CFIT_SUM_X_REG);
    let sum_y = reg_f64(state, CFIT_SUM_Y_REG);
    let sum_x2 = reg_f64(state, CFIT_SUM_X2_REG);
    let sum_xy = reg_f64(state, CFIT_SUM_XY_REG);
    let sum_y2 = reg_f64(state, CFIT_SUM_Y2_REG);
    let model = reg_f64(state, CFIT_MODEL_REG) as i32;

    let (b, a_raw, r) = compute_fit(n, sum_x, sum_y, sum_x2, sum_xy, sum_y2)?;

    // For exp and power models, the log-space intercept a_raw = ln(a_final).
    // Back-transform to get the physical scale factor.
    let a_final = match model {
        m if m == MODEL_EXP || m == MODEL_POWER => a_raw.exp(),
        _ => a_raw,
    };

    set_reg_f64(state, CFIT_COEFF_A_REG, a_final)?;
    set_reg_f64(state, CFIT_COEFF_B_REG, b)?;
    set_reg_f64(state, CFIT_CORR_REG, r)?;

    // Push b to Y and a to X using double-lift
    let b_hp = f64_to_hpnum(b)?;
    let a_hp = f64_to_hpnum(a_final)?;

    // First lift: push b (will become Y after second lift)
    state.stack.t = state.stack.z.clone();
    state.stack.z = state.stack.y.clone();
    state.stack.y = state.stack.x.clone();
    state.stack.x = b_hp;
    // Second lift: push a to X (b moves to Y)
    state.stack.t = state.stack.z.clone();
    state.stack.z = state.stack.y.clone();
    state.stack.y = state.stack.x.clone();
    state.stack.x = a_hp;

    state.stack.lift_enabled = true;
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV Y?X — predict Y from X using current fit coefficients and model.
///
/// Reads X from the stack, computes the predicted y using a (R17), b (R18),
/// and the current model (R16). Writes result to X (saves old X to LASTX).
///
/// LiftEffect: Enable.
///
/// # Errors
///
/// - `HpError::InvalidOp` on SIZE-floor or unknown model.
/// - `HpError::Domain` on domain error (e.g. log of non-positive x).
pub fn op_adv_y_query_x(state: &mut CalcState) -> Result<(), HpError> {
    require_cfit_size(state)?;

    let x = hpnum_to_f64(&state.stack.x)?;
    let a = reg_f64(state, CFIT_COEFF_A_REG);
    let b = reg_f64(state, CFIT_COEFF_B_REG);
    let model = reg_f64(state, CFIT_MODEL_REG) as i32;

    let y_pred = match model {
        m if m == MODEL_LINEAR => a + b * x,
        m if m == MODEL_LOG => {
            if x <= 0.0 {
                return Err(HpError::Domain);
            }
            a + b * x.ln()
        }
        m if m == MODEL_EXP => a * (b * x).exp(),
        m if m == MODEL_POWER => {
            if x <= 0.0 {
                return Err(HpError::Domain);
            }
            a * x.powf(b)
        }
        _ => return Err(HpError::InvalidOp),
    };

    let result = f64_to_hpnum(y_pred)?;
    state.stack.lastx = state.stack.x.clone();
    state.stack.x = result;
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV SZ? — test if the current sample count n >= 3 (minimum for
/// meaningful curve fitting). Pushes 1 to X if n >= 3, 0 otherwise.
///
/// LiftEffect: Enable.
///
/// # Errors
///
/// Returns `HpError::InvalidOp` on SIZE-floor.
pub fn op_adv_sz_query(state: &mut CalcState) -> Result<(), HpError> {
    require_cfit_size(state)?;
    let n = reg_f64(state, CFIT_N_REG);
    let result = if n >= 3.0 {
        HpNum::from(1i32)
    } else {
        HpNum::zero()
    };
    enter_number(state, result);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Helper: create a fresh CalcState (100 registers → covers R0–R99)
    fn new_state() -> CalcState {
        CalcState::new()
    }

    // Helper: set X and Y on the stack
    fn set_xy(state: &mut CalcState, x: f64, y: f64) {
        state.stack.x = HpNum::rounded(Decimal::from_f64(x).unwrap());
        state.stack.y = HpNum::rounded(Decimal::from_f64(y).unwrap());
    }

    // Helper: read stack X as f64
    fn x_f64(state: &CalcState) -> f64 {
        hpnum_to_f64(&state.stack.x).unwrap()
    }

    // Helper: read stack Y as f64
    fn y_f64(state: &CalcState) -> f64 {
        hpnum_to_f64(&state.stack.y).unwrap()
    }

    // Catches: CFIT clears all accumulation registers (R10–R19)
    #[test]
    fn adv_cfit_clears_registers() {
        let mut state = new_state();
        // Pre-dirty some registers
        state.regs[CFIT_N_REG] = HpNum::from(5i32).into();
        state.regs[CFIT_SUM_X_REG] = HpNum::from(42i32).into();
        state.regs[CFIT_COEFF_A_REG] = HpNum::from(3i32).into();

        op_adv_cfit(&mut state).unwrap();

        for idx in CFIT_N_REG..=CFIT_MAX_REG {
            assert!(
                state.regs[idx].numeric_or_zero().is_zero(),
                "register R{idx} should be zero after CFIT"
            );
        }
    }

    // Catches: AS accumulates count correctly
    #[test]
    fn adv_as_accumulates_count() {
        let mut state = new_state();
        op_adv_cfit(&mut state).unwrap();

        set_xy(&mut state, 1.0, 2.0);
        op_adv_as(&mut state).unwrap();
        // X should now be n=1
        let n = x_f64(&state);
        assert!(
            (n - 1.0).abs() < 1e-9,
            "n should be 1 after first AS, got {n}"
        );

        set_xy(&mut state, 3.0, 6.0);
        op_adv_as(&mut state).unwrap();
        let n = x_f64(&state);
        assert!(
            (n - 2.0).abs() < 1e-9,
            "n should be 2 after second AS, got {n}"
        );
    }

    // Catches: linear AS + FIT with collinear data returns correct coefficients
    #[test]
    fn adv_fit_linear_collinear() {
        let mut state = new_state();
        op_adv_cfit(&mut state).unwrap();

        // 3 collinear points: y = 2x + 1 → slope=2, intercept=1, r=1
        let points = [(1.0, 3.0), (2.0, 5.0), (3.0, 7.0)];
        for (x, y) in points {
            set_xy(&mut state, x, y);
            op_adv_as(&mut state).unwrap();
        }

        op_adv_fit(&mut state).unwrap();

        // X = a (intercept), Y = b (slope)
        let a = x_f64(&state);
        let b = y_f64(&state);
        let corr = reg_f64(&state, CFIT_CORR_REG);

        assert!((a - 1.0).abs() < 1e-6, "intercept should be 1.0, got {a}");
        assert!((b - 2.0).abs() < 1e-6, "slope should be 2.0, got {b}");
        assert!(
            (corr - 1.0).abs() < 1e-6,
            "correlation should be 1.0, got {corr}"
        );
    }

    // Catches: Y?X predicts correctly after a linear fit
    #[test]
    fn adv_y_query_x_linear_prediction() {
        let mut state = new_state();
        op_adv_cfit(&mut state).unwrap();

        // y = 2x + 1
        for (x, y) in [(1.0, 3.0), (2.0, 5.0), (3.0, 7.0)] {
            set_xy(&mut state, x, y);
            op_adv_as(&mut state).unwrap();
        }
        op_adv_fit(&mut state).unwrap();

        // Predict y at x=4 → expected: 9.0
        state.stack.x = HpNum::rounded(Decimal::from_f64(4.0).unwrap());
        op_adv_y_query_x(&mut state).unwrap();

        let pred = x_f64(&state);
        assert!(
            (pred - 9.0).abs() < 1e-6,
            "predicted y at x=4 should be 9.0, got {pred}"
        );
    }

    // Catches: SZ? returns 0 when n < 3
    #[test]
    fn adv_sz_query_false_when_n_lt_3() {
        let mut state = new_state();
        op_adv_cfit(&mut state).unwrap();

        set_xy(&mut state, 1.0, 2.0);
        op_adv_as(&mut state).unwrap();
        set_xy(&mut state, 2.0, 4.0);
        op_adv_as(&mut state).unwrap();
        // n == 2 → SZ? should return 0
        op_adv_sz_query(&mut state).unwrap();
        let result = x_f64(&state);
        assert!(
            result.abs() < 1e-9,
            "SZ? should return 0 when n=2, got {result}"
        );
    }

    // Catches: SZ? returns 1 when n >= 3
    #[test]
    fn adv_sz_query_true_when_n_gte_3() {
        let mut state = new_state();
        op_adv_cfit(&mut state).unwrap();

        for (x, y) in [(1.0, 3.0), (2.0, 5.0), (3.0, 7.0)] {
            set_xy(&mut state, x, y);
            op_adv_as(&mut state).unwrap();
        }
        op_adv_sz_query(&mut state).unwrap();
        let result = x_f64(&state);
        assert!(
            (result - 1.0).abs() < 1e-9,
            "SZ? should return 1 when n=3, got {result}"
        );
    }

    // Catches: DS reverses AS correctly
    #[test]
    fn adv_ds_reverses_as() {
        let mut state = new_state();
        op_adv_cfit(&mut state).unwrap();

        // Add 3 points, then remove 1
        for (x, y) in [(1.0, 3.0), (2.0, 5.0), (3.0, 7.0)] {
            set_xy(&mut state, x, y);
            op_adv_as(&mut state).unwrap();
        }
        // Remove last point (3, 7)
        set_xy(&mut state, 3.0, 7.0);
        op_adv_ds(&mut state).unwrap();

        let n = reg_f64(&state, CFIT_N_REG);
        assert!((n - 2.0).abs() < 1e-9, "n should be 2 after DS, got {n}");
    }

    // Catches: end-to-end linear fit workflow
    #[test]
    fn adv_cfit_full_workflow() {
        let mut state = new_state();
        op_adv_cfit(&mut state).unwrap();

        // Data: y = 3x + 2
        for (x, y) in [(1.0, 5.0), (2.0, 8.0), (3.0, 11.0), (4.0, 14.0)] {
            set_xy(&mut state, x, y);
            op_adv_as(&mut state).unwrap();
        }

        // SZ? should indicate sufficient data
        op_adv_sz_query(&mut state).unwrap();
        assert!((x_f64(&state) - 1.0).abs() < 1e-9);

        op_adv_fit(&mut state).unwrap();

        // Predict at x=5: expected y = 3*5+2 = 17
        state.stack.x = HpNum::rounded(Decimal::from_f64(5.0).unwrap());
        op_adv_y_query_x(&mut state).unwrap();
        let pred = x_f64(&state);
        assert!(
            (pred - 17.0).abs() < 1e-4,
            "predicted y at x=5 should be 17.0, got {pred}"
        );
    }

    // Catches: BFIT returns |r| for the stored data
    #[test]
    fn adv_bfit_returns_abs_r() {
        let mut state = new_state();
        op_adv_cfit(&mut state).unwrap();

        // Perfect linear data → |r| = 1
        for (x, y) in [(1.0, 3.0), (2.0, 5.0), (3.0, 7.0)] {
            set_xy(&mut state, x, y);
            op_adv_as(&mut state).unwrap();
        }
        op_adv_bfit(&mut state).unwrap();
        let r_abs = x_f64(&state);
        assert!(
            (r_abs - 1.0).abs() < 1e-6,
            "|r| should be 1.0 for perfect linear data, got {r_abs}"
        );
    }

    // Catches: named constants are in valid register range (T-43-11 mitigation)
    #[test]
    fn cfit_register_constants_in_bounds() {
        let state = new_state();
        // CalcState::new() provides 100 registers (0..99)
        assert!(
            CFIT_MAX_REG < state.regs.len(),
            "CFIT_MAX_REG must be addressable"
        );
        const { assert!(CFIT_N_REG >= 10, "CFIT block must start at R10 or later") };
        const { assert!(CFIT_MAX_REG <= 19, "CFIT block must end at R19 or earlier") };
    }
}
