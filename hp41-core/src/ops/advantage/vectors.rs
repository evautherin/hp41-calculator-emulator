// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `vectors` — ADV MATH: vector operations.
//!
//! XROM module id 24 (ADV_MATH_B, bit-4 of `CalcState::xrom_modules`).
//!
//! Operations: V+ / V- / DOT / CROSS / VC / VS / VR / VE / VXY / UV / V< / V* / VD / TR
//!
//! ## Register layout (ADV vector block, R20–R28)
//!
//! Vectors are stored in three contiguous registers per component (x, y, z):
//!
//! | Registers   | Constant       | Contents                     |
//! |-------------|----------------|------------------------------|
//! | R20, R21, R22 | `VEC_A_BASE`  | Vector A (components 1,2,3)  |
//! | R23, R24, R25 | `VEC_B_BASE`  | Vector B (components 1,2,3)  |
//! | R26, R27, R28 | `VEC_RESULT_BASE` | Result vector             |
//!
//! Helper functions `read_vec` / `write_vec` encapsulate register access.
//!
//! ## Operations
//!
//! | Op    | ADV-MATH | Description                                        |
//! |-------|----------|----------------------------------------------------|
//! | VE    | ADV-MATH-41 | Enter vector A components via modal prompt      |
//! | V+    | ADV-MATH-34 | Element-wise sum: result = A + B               |
//! | V-    | ADV-MATH-35 | Element-wise difference: result = A − B        |
//! | DOT   | ADV-MATH-36 | Dot product: scalar = A · B                    |
//! | CROSS | ADV-MATH-37 | Cross product: result = A × B                  |
//! | VC    | ADV-MATH-38 | Alias for CROSS                                |
//! | VS    | ADV-MATH-39 | Scale: result = scalar(X) × A                  |
//! | VR    | ADV-MATH-40 | Recall: push result components to X/Y/Z        |
//! | VXY   | ADV-MATH-42 | Project to X-Y plane (zero Z component of A)   |
//! | UV    | ADV-MATH-43 | Unit vector: result = A / |A|                  |
//! | V<    | ADV-MATH-44 | Magnitude: scalar = |A|                        |
//! | VD    | ADV-MATH-45 | Alias for DOT                                  |
//! | V*    | ADV-MATH-46 | Alias for VS (scalar multiply)                  |
//! | TR    | ADV-MATH-47 | Coordinate transform: rotate A by angle X (deg) |
//!
//! Source: HP Advantage Pac Owner's Manual 00041-90482 §3 Vector Operations.

use crate::ops::math1::ModalProgram;
use crate::{
    error::HpError,
    num::HpNum,
    ops::advantage::modal::AdvantageStep,
    stack::{apply_lift_effect, enter_number, LiftEffect},
    state::CalcState,
};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

// ── Register index constants ───────────────────────────────────────────────

/// Base register index for vector A (components at VEC_A_BASE, +1, +2).
pub const VEC_A_BASE: usize = 20;
/// Base register index for vector B (components at VEC_B_BASE, +1, +2).
pub const VEC_B_BASE: usize = 23;
/// Base register index for the result vector.
pub const VEC_RESULT_BASE: usize = 26;

/// Highest register index used by the vector block (R28).
pub const VEC_MAX_REG: usize = VEC_RESULT_BASE + 2;

// ── Internal helpers ───────────────────────────────────────────────────────

/// SIZE-floor guard: fails if regs cannot address VEC_MAX_REG.
#[inline]
fn require_vec_size(state: &CalcState) -> Result<(), HpError> {
    if state.regs.len() <= VEC_MAX_REG {
        return Err(HpError::InvalidOp);
    }
    Ok(())
}

/// Read 3 vector components from registers at `base`, `base+1`, `base+2`.
#[inline]
fn read_vec(state: &CalcState, base: usize) -> (f64, f64, f64) {
    let v1 = state.regs[base]
        .numeric_or_zero()
        .inner()
        .to_f64()
        .unwrap_or(0.0);
    let v2 = state.regs[base + 1]
        .numeric_or_zero()
        .inner()
        .to_f64()
        .unwrap_or(0.0);
    let v3 = state.regs[base + 2]
        .numeric_or_zero()
        .inner()
        .to_f64()
        .unwrap_or(0.0);
    (v1, v2, v3)
}

/// Write 3 vector components to registers at `base`, `base+1`, `base+2`.
fn write_vec(state: &mut CalcState, base: usize, v: (f64, f64, f64)) -> Result<(), HpError> {
    let mk = |val: f64| -> Result<crate::num::HpValue, HpError> {
        Ok(HpNum::rounded(Decimal::from_f64(val).ok_or(HpError::Overflow)?).into())
    };
    state.regs[base] = mk(v.0)?;
    state.regs[base + 1] = mk(v.1)?;
    state.regs[base + 2] = mk(v.2)?;
    Ok(())
}

/// Convert HpNum to f64.
#[inline]
fn hpnum_to_f64(n: &HpNum) -> Result<f64, HpError> {
    n.inner().to_f64().ok_or(HpError::Overflow)
}

/// Build an HpNum from f64.
#[inline]
fn f64_to_hpnum(v: f64) -> Result<HpNum, HpError> {
    Decimal::from_f64(v)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)
}

// ── Public op implementations ──────────────────────────────────────────────

/// ADV V+ — compute element-wise sum of vector A and vector B.
///
/// Reads A from R20–R22 and B from R23–R25, stores A+B in R26–R28.
/// LiftEffect: Neutral (result in registers, not pushed to stack).
///
/// # Errors
///
/// Returns `HpError::InvalidOp` on SIZE-floor.
pub fn op_adv_v_plus(state: &mut CalcState) -> Result<(), HpError> {
    require_vec_size(state)?;
    let (a1, a2, a3) = read_vec(state, VEC_A_BASE);
    let (b1, b2, b3) = read_vec(state, VEC_B_BASE);
    write_vec(state, VEC_RESULT_BASE, (a1 + b1, a2 + b2, a3 + b3))?;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV V- — compute element-wise difference A − B.
///
/// Reads A from R20–R22 and B from R23–R25, stores A−B in R26–R28.
/// LiftEffect: Neutral.
///
/// # Errors
///
/// Returns `HpError::InvalidOp` on SIZE-floor.
pub fn op_adv_v_minus(state: &mut CalcState) -> Result<(), HpError> {
    require_vec_size(state)?;
    let (a1, a2, a3) = read_vec(state, VEC_A_BASE);
    let (b1, b2, b3) = read_vec(state, VEC_B_BASE);
    write_vec(state, VEC_RESULT_BASE, (a1 - b1, a2 - b2, a3 - b3))?;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV DOT — compute dot product of vector A and vector B.
///
/// Reads A from R20–R22 and B from R23–R25. Pushes scalar result to X.
/// LiftEffect: Enable.
///
/// # Errors
///
/// Returns `HpError::InvalidOp` on SIZE-floor.
pub fn op_adv_dot(state: &mut CalcState) -> Result<(), HpError> {
    require_vec_size(state)?;
    let (a1, a2, a3) = read_vec(state, VEC_A_BASE);
    let (b1, b2, b3) = read_vec(state, VEC_B_BASE);
    let dot = a1 * b1 + a2 * b2 + a3 * b3;
    let result = f64_to_hpnum(dot)?;
    enter_number(state, result);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV CROSS — compute cross product of vector A × vector B.
///
/// Reads A from R20–R22 and B from R23–R25. Stores result in R26–R28.
/// Also pushes the magnitude of the result to X.
/// LiftEffect: Enable.
///
/// # Errors
///
/// Returns `HpError::InvalidOp` on SIZE-floor.
pub fn op_adv_cross(state: &mut CalcState) -> Result<(), HpError> {
    require_vec_size(state)?;
    let (a1, a2, a3) = read_vec(state, VEC_A_BASE);
    let (b1, b2, b3) = read_vec(state, VEC_B_BASE);
    // Standard cross product formula
    let r1 = a2 * b3 - a3 * b2;
    let r2 = a3 * b1 - a1 * b3;
    let r3 = a1 * b2 - a2 * b1;
    write_vec(state, VEC_RESULT_BASE, (r1, r2, r3))?;
    let mag = (r1 * r1 + r2 * r2 + r3 * r3).sqrt();
    let result = f64_to_hpnum(mag)?;
    enter_number(state, result);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV VC — alias for CROSS (alternate entry point per OM §3.4).
///
/// # Errors
///
/// Returns `HpError::InvalidOp` on SIZE-floor.
pub fn op_adv_vc(state: &mut CalcState) -> Result<(), HpError> {
    op_adv_cross(state)
}

/// ADV VS — scale vector A by scalar X: result = X × A.
///
/// Reads scalar from X and vector A from R20–R22.
/// Stores result in R26–R28.
/// LiftEffect: Neutral (scalar consumed; result in registers, not stack).
///
/// # Errors
///
/// Returns `HpError::InvalidOp` on SIZE-floor.
pub fn op_adv_vs(state: &mut CalcState) -> Result<(), HpError> {
    require_vec_size(state)?;
    let scalar = hpnum_to_f64(&state.stack.x)?;
    let (a1, a2, a3) = read_vec(state, VEC_A_BASE);
    write_vec(
        state,
        VEC_RESULT_BASE,
        (scalar * a1, scalar * a2, scalar * a3),
    )?;
    // Drop X (scalar consumed)
    state.stack.x = state.stack.y.clone();
    state.stack.y = state.stack.z.clone();
    state.stack.z = state.stack.t.clone();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV VR — recall result vector: push components to X, Y, Z stack registers.
///
/// Pushes result[3] to Z, result[2] to Y, result[1] to X (so component 1
/// is in X after the operation).
/// LiftEffect: Enable.
///
/// # Errors
///
/// Returns `HpError::InvalidOp` on SIZE-floor.
pub fn op_adv_vr(state: &mut CalcState) -> Result<(), HpError> {
    require_vec_size(state)?;
    let (r1, r2, r3) = read_vec(state, VEC_RESULT_BASE);

    let r1_hp = f64_to_hpnum(r1)?;
    let r2_hp = f64_to_hpnum(r2)?;
    let r3_hp = f64_to_hpnum(r3)?;

    // Triple-push: r3 → Z, r2 → Y, r1 → X
    // First push r3
    state.stack.t = state.stack.z.clone();
    state.stack.z = state.stack.y.clone();
    state.stack.y = state.stack.x.clone();
    state.stack.x = r3_hp;
    // Push r2
    state.stack.t = state.stack.z.clone();
    state.stack.z = state.stack.y.clone();
    state.stack.y = state.stack.x.clone();
    state.stack.x = r2_hp;
    // Push r1
    state.stack.t = state.stack.z.clone();
    state.stack.z = state.stack.y.clone();
    state.stack.y = state.stack.x.clone();
    state.stack.x = r1_hp;

    state.stack.lift_enabled = true;
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV VE — enter vector A components via modal prompt.
///
/// Sets up a `ModalProgram::Advantage(AdvantageStep::VeComponentPrompt(1))`
/// with the initial prompt "V[1]=?". Each R/S submit stores one component
/// into R20, R21, R22 respectively.
///
/// LiftEffect: Neutral (no stack change; modal interaction follows).
///
/// # Errors
///
/// Returns `HpError::InvalidOp` on SIZE-floor.
pub fn op_adv_ve(state: &mut CalcState) -> Result<(), HpError> {
    require_vec_size(state)?;
    // Clear vector A registers before entry
    write_vec(state, VEC_A_BASE, (0.0, 0.0, 0.0))?;
    // Enter modal workflow for component 1
    state.modal_program = Some(ModalProgram::Advantage(AdvantageStep::VeComponentPrompt(1)));
    state.modal_prompt = Some("V[1]=?".to_string());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV VXY — project vector A onto X-Y plane (zero the Z component).
///
/// Reads A from R20–R22, sets component 3 to 0, stores result back in R20–R22.
/// LiftEffect: Neutral.
///
/// # Errors
///
/// Returns `HpError::InvalidOp` on SIZE-floor.
pub fn op_adv_vxy(state: &mut CalcState) -> Result<(), HpError> {
    require_vec_size(state)?;
    let (a1, a2, _) = read_vec(state, VEC_A_BASE);
    write_vec(state, VEC_A_BASE, (a1, a2, 0.0))?;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV UV — normalize vector A to unit vector: result = A / |A|.
///
/// Stores the unit vector in R26–R28.
/// LiftEffect: Neutral.
///
/// # Errors
///
/// - `HpError::InvalidOp` on SIZE-floor.
/// - `HpError::DivideByZero` if |A| = 0.
pub fn op_adv_uv(state: &mut CalcState) -> Result<(), HpError> {
    require_vec_size(state)?;
    let (a1, a2, a3) = read_vec(state, VEC_A_BASE);
    let mag = (a1 * a1 + a2 * a2 + a3 * a3).sqrt();
    if mag < 1e-15 {
        return Err(HpError::DivideByZero);
    }
    write_vec(state, VEC_RESULT_BASE, (a1 / mag, a2 / mag, a3 / mag))?;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV V< — compute Euclidean magnitude of vector A: |A| = sqrt(a1²+a2²+a3²).
///
/// Pushes magnitude to X.
/// LiftEffect: Enable.
///
/// # Errors
///
/// Returns `HpError::InvalidOp` on SIZE-floor.
pub fn op_adv_v_mag(state: &mut CalcState) -> Result<(), HpError> {
    require_vec_size(state)?;
    let (a1, a2, a3) = read_vec(state, VEC_A_BASE);
    let mag = (a1 * a1 + a2 * a2 + a3 * a3).sqrt();
    let result = f64_to_hpnum(mag)?;
    enter_number(state, result);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV VD — alias for DOT (alternate entry point per OM §3.2).
///
/// # Errors
///
/// Returns `HpError::InvalidOp` on SIZE-floor.
pub fn op_adv_vd(state: &mut CalcState) -> Result<(), HpError> {
    op_adv_dot(state)
}

/// ADV V* — alias for VS (scalar multiply, alternate entry point per OM §3.3).
///
/// # Errors
///
/// Returns `HpError::InvalidOp` on SIZE-floor.
pub fn op_adv_v_star(state: &mut CalcState) -> Result<(), HpError> {
    op_adv_vs(state)
}

/// ADV TR — coordinate transformation: rotate vector A by angle θ (degrees from X).
///
/// Reads scalar angle θ from X (in degrees), rotates vector A in the X-Y plane.
/// The rotation matrix is:
///   [cos θ  -sin θ  0]
///   [sin θ   cos θ  0]
///   [0       0      1]
///
/// Stores rotated result in R26–R28.
/// LiftEffect: Neutral.
///
/// # Errors
///
/// - `HpError::InvalidOp` on SIZE-floor.
/// - `HpError::Overflow` if angle conversion fails.
pub fn op_adv_tr(state: &mut CalcState) -> Result<(), HpError> {
    require_vec_size(state)?;
    let angle_deg = hpnum_to_f64(&state.stack.x)?;
    let angle_rad = angle_deg.to_radians();
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();

    let (a1, a2, a3) = read_vec(state, VEC_A_BASE);

    // Rotation in X-Y plane
    let r1 = cos_a * a1 - sin_a * a2;
    let r2 = sin_a * a1 + cos_a * a2;
    let r3 = a3;

    write_vec(state, VEC_RESULT_BASE, (r1, r2, r3))?;

    // Drop X (angle consumed)
    state.stack.x = state.stack.y.clone();
    state.stack.y = state.stack.z.clone();
    state.stack.z = state.stack.t.clone();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Helper: create a fresh CalcState (100 registers)
    fn new_state() -> CalcState {
        CalcState::new()
    }

    // Helper: set vector A registers
    fn set_vec_a(state: &mut CalcState, v: (f64, f64, f64)) {
        write_vec(state, VEC_A_BASE, v).unwrap();
    }

    // Helper: set vector B registers
    fn set_vec_b(state: &mut CalcState, v: (f64, f64, f64)) {
        write_vec(state, VEC_B_BASE, v).unwrap();
    }

    // Helper: read result vector registers
    fn get_result(state: &CalcState) -> (f64, f64, f64) {
        read_vec(state, VEC_RESULT_BASE)
    }

    // Helper: read stack X as f64
    fn x_f64(state: &CalcState) -> f64 {
        hpnum_to_f64(&state.stack.x).unwrap()
    }

    // Catches: DOT([1,2,3],[4,5,6]) = 32
    #[test]
    fn adv_dot_basic() {
        let mut state = new_state();
        set_vec_a(&mut state, (1.0, 2.0, 3.0));
        set_vec_b(&mut state, (4.0, 5.0, 6.0));
        op_adv_dot(&mut state).unwrap();
        let result = x_f64(&state);
        assert!(
            (result - 32.0).abs() < 1e-9,
            "DOT([1,2,3],[4,5,6]) should be 32.0, got {result}"
        );
    }

    // Catches: DOT is commutative
    #[test]
    fn adv_dot_commutative() {
        let mut state = new_state();
        set_vec_a(&mut state, (1.0, 0.0, 0.0));
        set_vec_b(&mut state, (0.0, 1.0, 0.0));
        op_adv_dot(&mut state).unwrap();
        let result = x_f64(&state);
        assert!(
            result.abs() < 1e-9,
            "DOT of orthogonal unit vectors should be 0"
        );
    }

    // Catches: CROSS([1,0,0],[0,1,0]) = [0,0,1]
    #[test]
    fn adv_cross_unit_vectors() {
        let mut state = new_state();
        set_vec_a(&mut state, (1.0, 0.0, 0.0));
        set_vec_b(&mut state, (0.0, 1.0, 0.0));
        op_adv_cross(&mut state).unwrap();
        let (r1, r2, r3) = get_result(&state);
        assert!(r1.abs() < 1e-9, "cross result[1] should be 0, got {r1}");
        assert!(r2.abs() < 1e-9, "cross result[2] should be 0, got {r2}");
        assert!(
            (r3 - 1.0).abs() < 1e-9,
            "cross result[3] should be 1, got {r3}"
        );
    }

    // Catches: V+([1,2,3],[4,5,6]) = [5,7,9]
    #[test]
    fn adv_v_plus_basic() {
        let mut state = new_state();
        set_vec_a(&mut state, (1.0, 2.0, 3.0));
        set_vec_b(&mut state, (4.0, 5.0, 6.0));
        op_adv_v_plus(&mut state).unwrap();
        let (r1, r2, r3) = get_result(&state);
        assert!((r1 - 5.0).abs() < 1e-9, "V+[1] should be 5, got {r1}");
        assert!((r2 - 7.0).abs() < 1e-9, "V+[2] should be 7, got {r2}");
        assert!((r3 - 9.0).abs() < 1e-9, "V+[3] should be 9, got {r3}");
    }

    // Catches: V-([4,5,6],[1,2,3]) = [3,3,3]
    #[test]
    fn adv_v_minus_basic() {
        let mut state = new_state();
        set_vec_a(&mut state, (4.0, 5.0, 6.0));
        set_vec_b(&mut state, (1.0, 2.0, 3.0));
        op_adv_v_minus(&mut state).unwrap();
        let (r1, r2, r3) = get_result(&state);
        assert!((r1 - 3.0).abs() < 1e-9, "V-[1] should be 3, got {r1}");
        assert!((r2 - 3.0).abs() < 1e-9, "V-[2] should be 3, got {r2}");
        assert!((r3 - 3.0).abs() < 1e-9, "V-[3] should be 3, got {r3}");
    }

    // Catches: VS scalar * [1,2,3] with scalar=2 = [2,4,6]
    #[test]
    fn adv_vs_scalar_multiply() {
        let mut state = new_state();
        set_vec_a(&mut state, (1.0, 2.0, 3.0));
        state.stack.x = HpNum::rounded(Decimal::from_f64(2.0).unwrap());
        op_adv_vs(&mut state).unwrap();
        let (r1, r2, r3) = get_result(&state);
        assert!((r1 - 2.0).abs() < 1e-9, "VS[1] should be 2, got {r1}");
        assert!((r2 - 4.0).abs() < 1e-9, "VS[2] should be 4, got {r2}");
        assert!((r3 - 6.0).abs() < 1e-9, "VS[3] should be 6, got {r3}");
    }

    // Catches: UV normalizes [3,4,0] to [0.6, 0.8, 0]
    #[test]
    fn adv_uv_unit_vector() {
        let mut state = new_state();
        set_vec_a(&mut state, (3.0, 4.0, 0.0));
        op_adv_uv(&mut state).unwrap();
        let (r1, r2, r3) = get_result(&state);
        assert!((r1 - 0.6).abs() < 1e-9, "UV[1] should be 0.6, got {r1}");
        assert!((r2 - 0.8).abs() < 1e-9, "UV[2] should be 0.8, got {r2}");
        assert!(r3.abs() < 1e-9, "UV[3] should be 0, got {r3}");
    }

    // Catches: UV of zero vector returns DivideByZero
    #[test]
    fn adv_uv_zero_vector_error() {
        let mut state = new_state();
        set_vec_a(&mut state, (0.0, 0.0, 0.0));
        let result = op_adv_uv(&mut state);
        assert!(matches!(result, Err(HpError::DivideByZero)));
    }

    // Catches: V< magnitude of [3,4,0] = 5
    #[test]
    fn adv_v_mag_basic() {
        let mut state = new_state();
        set_vec_a(&mut state, (3.0, 4.0, 0.0));
        op_adv_v_mag(&mut state).unwrap();
        let mag = x_f64(&state);
        assert!(
            (mag - 5.0).abs() < 1e-9,
            "V< of [3,4,0] should be 5.0, got {mag}"
        );
    }

    // Catches: VE sets up modal prompt for vector entry
    #[test]
    fn adv_ve_sets_modal_prompt() {
        let mut state = new_state();
        op_adv_ve(&mut state).unwrap();
        assert!(state.modal_program.is_some(), "VE should set modal_program");
        assert_eq!(state.modal_prompt.as_deref(), Some("V[1]=?"));
    }

    // Catches: VR recalls result vector to X/Y/Z
    #[test]
    fn adv_vr_recalls_result() {
        let mut state = new_state();
        // Put known result vector in registers
        write_vec(&mut state, VEC_RESULT_BASE, (10.0, 20.0, 30.0)).unwrap();
        op_adv_vr(&mut state).unwrap();
        // After VR: X=component1, Y=component2, Z=component3
        let x = x_f64(&state);
        let y = hpnum_to_f64(&state.stack.y).unwrap();
        let z = hpnum_to_f64(&state.stack.z).unwrap();
        assert!(
            (x - 10.0).abs() < 1e-9,
            "X should be component 1 = 10, got {x}"
        );
        assert!(
            (y - 20.0).abs() < 1e-9,
            "Y should be component 2 = 20, got {y}"
        );
        assert!(
            (z - 30.0).abs() < 1e-9,
            "Z should be component 3 = 30, got {z}"
        );
    }

    // Catches: VXY zeros the Z component
    #[test]
    fn adv_vxy_projects_to_xy_plane() {
        let mut state = new_state();
        set_vec_a(&mut state, (3.0, 4.0, 5.0));
        op_adv_vxy(&mut state).unwrap();
        let (a1, a2, a3) = read_vec(&state, VEC_A_BASE);
        assert!((a1 - 3.0).abs() < 1e-9);
        assert!((a2 - 4.0).abs() < 1e-9);
        assert!(
            a3.abs() < 1e-9,
            "Z component should be 0 after VXY, got {a3}"
        );
    }

    // Catches: TR rotates vector A by 90 degrees
    #[test]
    fn adv_tr_rotation_90deg() {
        let mut state = new_state();
        set_vec_a(&mut state, (1.0, 0.0, 0.0));
        state.stack.x = HpNum::rounded(Decimal::from_f64(90.0).unwrap());
        op_adv_tr(&mut state).unwrap();
        let (r1, r2, r3) = get_result(&state);
        // Rotating [1,0,0] by 90° → [0,1,0]
        assert!(r1.abs() < 1e-9, "TR[1] should be ~0, got {r1}");
        assert!((r2 - 1.0).abs() < 1e-9, "TR[2] should be ~1, got {r2}");
        assert!(r3.abs() < 1e-9, "TR[3] should be 0, got {r3}");
    }

    // Catches: VC is alias for CROSS (same result)
    #[test]
    fn adv_vc_same_as_cross() {
        let mut state = new_state();
        set_vec_a(&mut state, (1.0, 0.0, 0.0));
        set_vec_b(&mut state, (0.0, 1.0, 0.0));
        op_adv_vc(&mut state).unwrap();
        let (r1, r2, r3) = get_result(&state);
        assert!(r1.abs() < 1e-9);
        assert!(r2.abs() < 1e-9);
        assert!((r3 - 1.0).abs() < 1e-9);
    }

    // Catches: VD is alias for DOT
    #[test]
    fn adv_vd_same_as_dot() {
        let mut state = new_state();
        set_vec_a(&mut state, (2.0, 3.0, 4.0));
        set_vec_b(&mut state, (1.0, 1.0, 1.0));
        op_adv_vd(&mut state).unwrap();
        let result = x_f64(&state);
        assert!(
            (result - 9.0).abs() < 1e-9,
            "VD([2,3,4],[1,1,1]) should be 9, got {result}"
        );
    }

    // Catches: V* is alias for VS
    #[test]
    fn adv_v_star_same_as_vs() {
        let mut state = new_state();
        set_vec_a(&mut state, (1.0, 2.0, 3.0));
        state.stack.x = HpNum::rounded(Decimal::from_f64(3.0).unwrap());
        op_adv_v_star(&mut state).unwrap();
        let (r1, r2, r3) = get_result(&state);
        assert!((r1 - 3.0).abs() < 1e-9);
        assert!((r2 - 6.0).abs() < 1e-9);
        assert!((r3 - 9.0).abs() < 1e-9);
    }

    // Catches: named constants are in valid register range (T-43-11 mitigation)
    #[test]
    fn vec_register_constants_in_bounds() {
        let state = new_state();
        // CalcState::new() provides 100 registers (0..99)
        assert!(
            VEC_MAX_REG < state.regs.len(),
            "VEC_MAX_REG must be addressable"
        );
        const {
            assert!(
                VEC_A_BASE >= 20,
                "vector A block must start at R20 or later"
            )
        };
        const { assert!(VEC_MAX_REG <= 28, "vector block must end at R28 or earlier") };
    }
}
