// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `matrix_complex` — ADV MTRX complex-matrix operations.
//!
//! XROM module id 22 (ADV_MATH_A, bit-3 of `CalcState::xrom_modules`).
//!
//! D-43.5 ISOLATION INVARIANT: No function in this file reads or writes
//! `state.matrix_dim` or `state.matrix_active_reg`.  Named-matrix storage
//! uses `state.adv_matrices` exclusively.
//!
//! ## Complex element interleaving (D-43.5 / T-43-10 mitigated)
//!
//! For a complex matrix with `rows` rows and `cols` columns, element (i, j)
//! (0-based) is stored as:
//!   - `data[2 * (i * cols as usize + j)]`     — real part
//!   - `data[2 * (i * cols as usize + j) + 1]` — imaginary part
//!
//! The helper `complex_elem_indices(mat, i, j)` validates bounds and returns
//! `Ok((re_idx, im_idx))` or `Err(HpError::Domain)`.  All element accesses in
//! this module MUST go through this helper (T-43-10 mitigation).
//!
//! Operations: C<>C / CMAXAB / CNRM / CSUM / YC+C

use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

use crate::{
    error::HpError,
    num::HpNum,
    ops::advantage::AdvMatrix,
    stack::{apply_lift_effect, LiftEffect},
    state::CalcState,
};

// ── Helper: complex element index validation (T-43-10 mitigation) ─────────────

/// Compute and validate the real/imaginary data indices for complex element (i, j).
///
/// Both `i` and `j` are **0-based**. Returns `(re_idx, im_idx)` or
/// `Err(HpError::Domain)` if out of range.
///
/// # Errors
/// Returns `HpError::Domain` if (i, j) is outside the matrix dimensions or the
/// computed index would exceed `data.len()`.
fn complex_elem_indices(mat: &AdvMatrix, i: usize, j: usize) -> Result<(usize, usize), HpError> {
    if i >= mat.rows as usize || j >= mat.cols as usize {
        return Err(HpError::Domain);
    }
    let re_idx = 2 * (i * mat.cols as usize + j);
    let im_idx = re_idx + 1;
    if im_idx >= mat.data.len() {
        return Err(HpError::Domain);
    }
    Ok((re_idx, im_idx))
}

/// Find the index of a matrix by name in `state.adv_matrices`.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix with the given name exists.
fn find_matrix_by_name<'a>(matrices: &'a [AdvMatrix], name: &str) -> Result<usize, HpError> {
    matrices
        .iter()
        .position(|m| m.name == name)
        .ok_or(HpError::InvalidOp)
}

// ── Op implementations ────────────────────────────────────────────────────────

/// ADV C<>C — exchange two named complex matrices.
///
/// Swaps the data of the currently active complex matrix (from
/// `state.adv_current_matrix`) with the matrix named in `state.alpha_reg`.
/// Both matrices must be complex (`is_complex == true`); a real matrix returns
/// `HpError::Domain`. LiftEffect: Neutral.
///
/// # Errors
/// - `HpError::InvalidOp` — no active matrix or named matrix not found.
/// - `HpError::Domain` — either matrix is not complex.
pub fn op_adv_c_exchange_c(state: &mut CalcState) -> Result<(), HpError> {
    let active_name = state.adv_current_matrix.clone().ok_or(HpError::InvalidOp)?;
    let alpha_name = state.alpha_reg.trim().to_string();
    if alpha_name.is_empty() {
        return Err(HpError::InvalidOp);
    }

    // Validate both exist and are complex before mutating
    let active_idx = find_matrix_by_name(&state.adv_matrices, &active_name)?;
    let alpha_idx = find_matrix_by_name(&state.adv_matrices, &alpha_name)?;

    if !state.adv_matrices[active_idx].is_complex {
        return Err(HpError::Domain);
    }
    if !state.adv_matrices[alpha_idx].is_complex {
        return Err(HpError::Domain);
    }

    // Swap their data vectors
    // To avoid borrow-checker issues, take the data out and swap
    if active_idx == alpha_idx {
        // Same matrix — no-op
        apply_lift_effect(state, LiftEffect::Neutral);
        return Ok(());
    }

    // Swap data
    let (lo, hi) = if active_idx < alpha_idx {
        (active_idx, alpha_idx)
    } else {
        (alpha_idx, active_idx)
    };
    let (left, right) = state.adv_matrices.split_at_mut(hi);
    std::mem::swap(&mut left[lo].data, &mut right[0].data);

    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV CMAXAB — maximum |z| across all elements of the current complex matrix.
///
/// Computes |z_ij| = sqrt(re² + im²) for every element and pushes the maximum
/// to X. LiftEffect: Enable. Sets `complex_mode = true`.
///
/// # Errors
/// - `HpError::InvalidOp` — no active matrix or matrix not found.
/// - `HpError::Domain` — matrix is not complex.
/// - `HpError::Overflow` — conversion to f64 fails.
pub fn op_adv_cmaxab(state: &mut CalcState) -> Result<(), HpError> {
    let name = state.adv_current_matrix.clone().ok_or(HpError::InvalidOp)?;
    let idx = find_matrix_by_name(&state.adv_matrices, &name)?;
    if !state.adv_matrices[idx].is_complex {
        return Err(HpError::Domain);
    }

    let mat = &state.adv_matrices[idx];
    let rows = mat.rows as usize;
    let cols = mat.cols as usize;

    let mut max_abs: f64 = 0.0;
    for i in 0..rows {
        for j in 0..cols {
            let (re_idx, im_idx) = complex_elem_indices(mat, i, j)?;
            let re = mat.data[re_idx].inner().to_f64().ok_or(HpError::Overflow)?;
            let im = mat.data[im_idx].inner().to_f64().ok_or(HpError::Overflow)?;
            let abs_val = (re * re + im * im).sqrt();
            if abs_val > max_abs {
                max_abs = abs_val;
            }
        }
    }

    let result = Decimal::from_f64(max_abs)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;

    state.complex_mode = true;
    state.stack.x = result;
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV CNRM — complex Frobenius norm of the current complex matrix.
///
/// Frobenius norm = sqrt(Σ |z_ij|²) = sqrt(Σ (re² + im²)).
/// Pushes the norm to X. LiftEffect: Enable. Sets `complex_mode = true`.
///
/// # Errors
/// - `HpError::InvalidOp` — no active matrix or matrix not found.
/// - `HpError::Domain` — matrix is not complex.
/// - `HpError::Overflow` — conversion to f64 fails.
pub fn op_adv_cnrm(state: &mut CalcState) -> Result<(), HpError> {
    let name = state.adv_current_matrix.clone().ok_or(HpError::InvalidOp)?;
    let idx = find_matrix_by_name(&state.adv_matrices, &name)?;
    if !state.adv_matrices[idx].is_complex {
        return Err(HpError::Domain);
    }

    let mat = &state.adv_matrices[idx];
    let rows = mat.rows as usize;
    let cols = mat.cols as usize;

    let mut sum_sq: f64 = 0.0;
    for i in 0..rows {
        for j in 0..cols {
            let (re_idx, im_idx) = complex_elem_indices(mat, i, j)?;
            let re = mat.data[re_idx].inner().to_f64().ok_or(HpError::Overflow)?;
            let im = mat.data[im_idx].inner().to_f64().ok_or(HpError::Overflow)?;
            sum_sq += re * re + im * im;
        }
    }

    let norm = sum_sq.sqrt();
    let result = Decimal::from_f64(norm)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;

    state.complex_mode = true;
    state.stack.x = result;
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV CSUM — sum all complex elements of the current complex matrix.
///
/// Pushes real sum to X, imaginary sum to Y. LiftEffect: Enable.
/// Sets `complex_mode = true`.
///
/// # Errors
/// - `HpError::InvalidOp` — no active matrix or matrix not found.
/// - `HpError::Domain` — matrix is not complex.
/// - `HpError::Overflow` — f64 conversion or HpNum arithmetic fails.
pub fn op_adv_csum(state: &mut CalcState) -> Result<(), HpError> {
    let name = state.adv_current_matrix.clone().ok_or(HpError::InvalidOp)?;
    let idx = find_matrix_by_name(&state.adv_matrices, &name)?;
    if !state.adv_matrices[idx].is_complex {
        return Err(HpError::Domain);
    }

    let mat = &state.adv_matrices[idx];
    let rows = mat.rows as usize;
    let cols = mat.cols as usize;

    let mut sum_re: f64 = 0.0;
    let mut sum_im: f64 = 0.0;
    for i in 0..rows {
        for j in 0..cols {
            let (re_idx, im_idx) = complex_elem_indices(mat, i, j)?;
            sum_re += mat.data[re_idx].inner().to_f64().ok_or(HpError::Overflow)?;
            sum_im += mat.data[im_idx].inner().to_f64().ok_or(HpError::Overflow)?;
        }
    }

    let new_x = Decimal::from_f64(sum_re)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    let new_y = Decimal::from_f64(sum_im)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;

    state.complex_mode = true;
    state.stack.x = new_x;
    state.stack.y = new_y;
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV YC+C — add complex scalar (X+iY from stack) to element (I,J) of current complex matrix.
///
/// `state.adv_matrix_i` and `state.adv_matrix_j` hold the 1-based row/col indices.
/// The element is modified in-place. LiftEffect: Neutral (consumes scalar but does not
/// push to stack; scalar is preserved in X+iY per HP convention — Neutral keeps lift state).
///
/// # Errors
/// - `HpError::InvalidOp` — no active matrix or matrix not found.
/// - `HpError::Domain` — matrix is not complex.
/// - `HpError::Domain` — (I, J) is out of range for the matrix.
pub fn op_adv_yc_plus_c(state: &mut CalcState) -> Result<(), HpError> {
    let name = state.adv_current_matrix.clone().ok_or(HpError::InvalidOp)?;
    let idx = find_matrix_by_name(&state.adv_matrices, &name)?;
    if !state.adv_matrices[idx].is_complex {
        return Err(HpError::Domain);
    }

    // 1-based I, J from state
    let i = state.adv_matrix_i.saturating_sub(1) as usize; // convert to 0-based
    let j = state.adv_matrix_j.saturating_sub(1) as usize;

    let (re_idx, im_idx) = complex_elem_indices(&state.adv_matrices[idx], i, j)?;

    // Read scalar from stack X+iY
    let scalar_re = state.stack.x.clone();
    let scalar_im = state.stack.y.clone();

    // Modify element in-place
    let new_re = state.adv_matrices[idx].data[re_idx].checked_add(&scalar_re)?;
    let new_im = state.adv_matrices[idx].data[im_idx].checked_add(&scalar_im)?;

    state.adv_matrices[idx].data[re_idx] = new_re;
    state.adv_matrices[idx].data[im_idx] = new_im;

    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

// ── Helper for tests: create a complex matrix ─────────────────────────────────

#[cfg(test)]
fn make_complex_matrix(name: &str, rows: u8, cols: u8, data: Vec<f64>) -> AdvMatrix {
    use rust_decimal::Decimal;
    AdvMatrix {
        name: name.to_string(),
        rows,
        cols,
        is_complex: true,
        data: data
            .into_iter()
            .map(|v| HpNum::rounded(Decimal::from_f64(v).unwrap_or(Decimal::ZERO)))
            .collect(),
    }
}

#[cfg(test)]
fn make_real_matrix(name: &str, rows: u8, cols: u8, data: Vec<f64>) -> AdvMatrix {
    use rust_decimal::Decimal;
    AdvMatrix {
        name: name.to_string(),
        rows,
        cols,
        is_complex: false,
        data: data
            .into_iter()
            .map(|v| HpNum::rounded(Decimal::from_f64(v).unwrap_or(Decimal::ZERO)))
            .collect(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use rust_decimal::prelude::ToPrimitive;

    // Helper: set up state with an active complex matrix
    fn state_with_complex_matrix(name: &str, rows: u8, cols: u8, data: Vec<f64>) -> CalcState {
        let mut state = CalcState::new();
        let mat = make_complex_matrix(name, rows, cols, data);
        state.adv_matrices.push(mat);
        state.adv_current_matrix = Some(name.to_string());
        state.adv_matrix_i = 1;
        state.adv_matrix_j = 1;
        state
    }

    fn get_x_f64(state: &CalcState) -> f64 {
        state.stack.x.inner().to_f64().unwrap()
    }

    fn get_y_f64(state: &CalcState) -> f64 {
        state.stack.y.inner().to_f64().unwrap()
    }

    // ── ADV CSUM tests ───────────────────────────────────────────────────────

    /// Catches: CSUM of 2x1 complex matrix [(1+2i),(3+4i)] = 4+6i.
    /// Source: Plan 43-05 behavior spec.
    #[test]
    fn adv_csum_basic() {
        // 2x1 complex matrix: data = [re(0,0), im(0,0), re(1,0), im(1,0)]
        // = [1, 2, 3, 4]  → element (0,0) = 1+2i, element (1,0) = 3+4i
        let mut state = state_with_complex_matrix("A", 2, 1, vec![1.0, 2.0, 3.0, 4.0]);
        op_adv_csum(&mut state).unwrap();
        assert_relative_eq!(get_x_f64(&state), 4.0, max_relative = 1e-7);
        assert_relative_eq!(get_y_f64(&state), 6.0, max_relative = 1e-7);
    }

    /// Catches: CSUM sets complex_mode.
    #[test]
    fn adv_csum_sets_complex_mode() {
        let mut state = state_with_complex_matrix("A", 1, 1, vec![1.0, 0.0]);
        op_adv_csum(&mut state).unwrap();
        assert!(state.complex_mode);
    }

    /// Catches: CSUM enables lift.
    #[test]
    fn adv_csum_enables_lift() {
        let mut state = state_with_complex_matrix("A", 1, 1, vec![1.0, 0.0]);
        state.stack.lift_enabled = false;
        op_adv_csum(&mut state).unwrap();
        assert!(state.stack.lift_enabled);
    }

    /// Catches: CSUM of zero-element matrix (1x1 with 0+0i) = 0+0i.
    #[test]
    fn adv_csum_zero_matrix() {
        let mut state = state_with_complex_matrix("A", 1, 1, vec![0.0, 0.0]);
        op_adv_csum(&mut state).unwrap();
        assert!(state.stack.x.is_zero());
        assert!(state.stack.y.is_zero());
    }

    /// Catches: CSUM on real matrix → Domain.
    #[test]
    fn adv_csum_real_matrix_returns_domain() {
        let mut state = CalcState::new();
        let mat = make_real_matrix("R", 2, 1, vec![1.0, 2.0]);
        state.adv_matrices.push(mat);
        state.adv_current_matrix = Some("R".to_string());
        assert!(matches!(op_adv_csum(&mut state), Err(HpError::Domain)));
    }

    /// Catches: CSUM with no active matrix → InvalidOp.
    #[test]
    fn adv_csum_no_active_matrix() {
        let mut state = CalcState::new();
        assert!(matches!(op_adv_csum(&mut state), Err(HpError::InvalidOp)));
    }

    // ── ADV CNRM tests ───────────────────────────────────────────────────────

    /// Catches: CNRM of 1x2 complex matrix [(3+4i),(0+0i)] = sqrt(25) = 5.
    /// |3+4i|² = 9+16 = 25; |0+0i|² = 0; norm = sqrt(25) = 5.
    #[test]
    fn adv_cnrm_3_4i_norm_is_5() {
        // 1x2 complex: data = [3, 4, 0, 0]
        let mut state = state_with_complex_matrix("A", 1, 2, vec![3.0, 4.0, 0.0, 0.0]);
        op_adv_cnrm(&mut state).unwrap();
        assert_relative_eq!(get_x_f64(&state), 5.0, max_relative = 1e-7);
    }

    /// Catches: CNRM of identity-like complex matrix.
    /// 2x2 complex matrix with (1+0i) on diagonal, (0+0i) off-diagonal.
    /// norm = sqrt(1² + 0² + 0² + 1²) = sqrt(2)
    #[test]
    fn adv_cnrm_identity_2x2() {
        // data = [1, 0, 0, 0, 0, 0, 1, 0] (row-major, complex interleaved)
        let data = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let mut state = state_with_complex_matrix("A", 2, 2, data);
        op_adv_cnrm(&mut state).unwrap();
        assert_relative_eq!(get_x_f64(&state), 2.0_f64.sqrt(), max_relative = 1e-7);
    }

    /// Catches: CNRM sets complex_mode.
    #[test]
    fn adv_cnrm_sets_complex_mode() {
        let mut state = state_with_complex_matrix("A", 1, 1, vec![1.0, 0.0]);
        op_adv_cnrm(&mut state).unwrap();
        assert!(state.complex_mode);
    }

    /// Catches: CNRM enables lift.
    #[test]
    fn adv_cnrm_enables_lift() {
        let mut state = state_with_complex_matrix("A", 1, 1, vec![1.0, 0.0]);
        state.stack.lift_enabled = false;
        op_adv_cnrm(&mut state).unwrap();
        assert!(state.stack.lift_enabled);
    }

    /// Catches: CNRM on real matrix → Domain.
    #[test]
    fn adv_cnrm_real_matrix_is_domain() {
        let mut state = CalcState::new();
        let mat = make_real_matrix("R", 1, 1, vec![1.0]);
        state.adv_matrices.push(mat);
        state.adv_current_matrix = Some("R".to_string());
        assert!(matches!(op_adv_cnrm(&mut state), Err(HpError::Domain)));
    }

    // ── ADV CMAXAB tests ─────────────────────────────────────────────────────

    /// Catches: CMAXAB returns max |z| across all elements.
    /// Elements: (3+4i) → |z|=5, (1+0i) → |z|=1. Max = 5.
    #[test]
    fn adv_cmaxab_max_is_5() {
        // 1x2: data = [3, 4, 1, 0] → (3+4i), (1+0i)
        let mut state = state_with_complex_matrix("A", 1, 2, vec![3.0, 4.0, 1.0, 0.0]);
        op_adv_cmaxab(&mut state).unwrap();
        assert_relative_eq!(get_x_f64(&state), 5.0, max_relative = 1e-7);
    }

    /// Catches: CMAXAB of zero matrix = 0.
    #[test]
    fn adv_cmaxab_zero_matrix_is_zero() {
        let mut state = state_with_complex_matrix("A", 1, 1, vec![0.0, 0.0]);
        op_adv_cmaxab(&mut state).unwrap();
        assert_relative_eq!(get_x_f64(&state), 0.0, epsilon = 1e-10);
    }

    /// Catches: CMAXAB sets complex_mode.
    #[test]
    fn adv_cmaxab_sets_complex_mode() {
        let mut state = state_with_complex_matrix("A", 1, 1, vec![1.0, 0.0]);
        op_adv_cmaxab(&mut state).unwrap();
        assert!(state.complex_mode);
    }

    /// Catches: CMAXAB on real matrix → Domain.
    #[test]
    fn adv_cmaxab_real_matrix_is_domain() {
        let mut state = CalcState::new();
        let mat = make_real_matrix("R", 1, 1, vec![1.0]);
        state.adv_matrices.push(mat);
        state.adv_current_matrix = Some("R".to_string());
        assert!(matches!(op_adv_cmaxab(&mut state), Err(HpError::Domain)));
    }

    /// Catches: CMAXAB enables lift.
    #[test]
    fn adv_cmaxab_enables_lift() {
        let mut state = state_with_complex_matrix("A", 1, 1, vec![1.0, 0.0]);
        state.stack.lift_enabled = false;
        op_adv_cmaxab(&mut state).unwrap();
        assert!(state.stack.lift_enabled);
    }

    // ── ADV C<>C tests ───────────────────────────────────────────────────────

    /// Catches: C<>C swaps data of two complex matrices.
    #[test]
    fn adv_c_exchange_c_swaps_data() {
        let mut state = CalcState::new();
        let mat_a = make_complex_matrix("A", 1, 1, vec![1.0, 2.0]);
        let mat_b = make_complex_matrix("B", 1, 1, vec![3.0, 4.0]);
        state.adv_matrices.push(mat_a);
        state.adv_matrices.push(mat_b);
        state.adv_current_matrix = Some("A".to_string());
        state.alpha_reg = "B".to_string();

        op_adv_c_exchange_c(&mut state).unwrap();

        let a_re = state.adv_matrices[0].data[0].inner().to_f64().unwrap();
        let a_im = state.adv_matrices[0].data[1].inner().to_f64().unwrap();
        let b_re = state.adv_matrices[1].data[0].inner().to_f64().unwrap();
        let b_im = state.adv_matrices[1].data[1].inner().to_f64().unwrap();

        // After swap: A has B's data, B has A's data
        assert_relative_eq!(a_re, 3.0, max_relative = 1e-7);
        assert_relative_eq!(a_im, 4.0, max_relative = 1e-7);
        assert_relative_eq!(b_re, 1.0, max_relative = 1e-7);
        assert_relative_eq!(b_im, 2.0, max_relative = 1e-7);
    }

    /// Catches: C<>C on real matrix → Domain.
    #[test]
    fn adv_c_exchange_c_real_matrix_is_domain() {
        let mut state = CalcState::new();
        let mat_a = make_real_matrix("A", 1, 1, vec![1.0]);
        let mat_b = make_complex_matrix("B", 1, 1, vec![3.0, 4.0]);
        state.adv_matrices.push(mat_a);
        state.adv_matrices.push(mat_b);
        state.adv_current_matrix = Some("A".to_string());
        state.alpha_reg = "B".to_string();
        assert!(matches!(
            op_adv_c_exchange_c(&mut state),
            Err(HpError::Domain)
        ));
    }

    /// Catches: C<>C with no active matrix → InvalidOp.
    #[test]
    fn adv_c_exchange_c_no_active_matrix() {
        let mut state = CalcState::new();
        state.alpha_reg = "B".to_string();
        assert!(matches!(
            op_adv_c_exchange_c(&mut state),
            Err(HpError::InvalidOp)
        ));
    }

    /// Catches: C<>C with non-existent alpha matrix → InvalidOp.
    #[test]
    fn adv_c_exchange_c_missing_alpha_matrix() {
        let mut state = CalcState::new();
        let mat_a = make_complex_matrix("A", 1, 1, vec![1.0, 2.0]);
        state.adv_matrices.push(mat_a);
        state.adv_current_matrix = Some("A".to_string());
        state.alpha_reg = "MISSING".to_string();
        assert!(matches!(
            op_adv_c_exchange_c(&mut state),
            Err(HpError::InvalidOp)
        ));
    }

    /// Catches: C<>C is LiftEffect::Neutral.
    #[test]
    fn adv_c_exchange_c_neutral_lift() {
        let mut state = CalcState::new();
        let mat_a = make_complex_matrix("A", 1, 1, vec![1.0, 2.0]);
        let mat_b = make_complex_matrix("B", 1, 1, vec![3.0, 4.0]);
        state.adv_matrices.push(mat_a);
        state.adv_matrices.push(mat_b);
        state.adv_current_matrix = Some("A".to_string());
        state.alpha_reg = "B".to_string();
        state.stack.lift_enabled = true;
        op_adv_c_exchange_c(&mut state).unwrap();
        assert!(state.stack.lift_enabled, "C<>C must be LiftEffect::Neutral");
    }

    // ── ADV YC+C tests ───────────────────────────────────────────────────────

    /// Catches: YC+C adds (X+iY) to element (I,J) in-place.
    /// Element (0,0) = (1+2i), add (3+4i) → (4+6i).
    #[test]
    fn adv_yc_plus_c_basic() {
        let mut state = state_with_complex_matrix(
            "A",
            2,
            2,
            vec![
                1.0, 2.0, // (0,0)
                0.0, 0.0, // (0,1)
                0.0, 0.0, // (1,0)
                0.0, 0.0, // (1,1)
            ],
        );
        use rust_decimal::Decimal;
        state.stack.x = HpNum::rounded(Decimal::from_f64(3.0).unwrap());
        state.stack.y = HpNum::rounded(Decimal::from_f64(4.0).unwrap());
        state.adv_matrix_i = 1; // row 1 (0-based: 0)
        state.adv_matrix_j = 1; // col 1 (0-based: 0)

        op_adv_yc_plus_c(&mut state).unwrap();

        let re = state.adv_matrices[0].data[0].inner().to_f64().unwrap();
        let im = state.adv_matrices[0].data[1].inner().to_f64().unwrap();
        assert_relative_eq!(re, 4.0, max_relative = 1e-7);
        assert_relative_eq!(im, 6.0, max_relative = 1e-7);
    }

    /// Catches: YC+C on real matrix → Domain.
    #[test]
    fn adv_yc_plus_c_real_matrix_is_domain() {
        let mut state = CalcState::new();
        let mat = make_real_matrix("R", 1, 1, vec![1.0]);
        state.adv_matrices.push(mat);
        state.adv_current_matrix = Some("R".to_string());
        state.adv_matrix_i = 1;
        state.adv_matrix_j = 1;
        assert!(matches!(op_adv_yc_plus_c(&mut state), Err(HpError::Domain)));
    }

    /// Catches: YC+C with no active matrix → InvalidOp.
    #[test]
    fn adv_yc_plus_c_no_active_matrix() {
        let mut state = CalcState::new();
        assert!(matches!(
            op_adv_yc_plus_c(&mut state),
            Err(HpError::InvalidOp)
        ));
    }

    /// Catches: YC+C with out-of-bounds I,J → Bounds.
    #[test]
    fn adv_yc_plus_c_out_of_bounds() {
        let mut state = state_with_complex_matrix("A", 1, 1, vec![0.0, 0.0]);
        state.adv_matrix_i = 5; // out of range for 1x1
        state.adv_matrix_j = 1;
        assert!(matches!(op_adv_yc_plus_c(&mut state), Err(HpError::Domain)));
    }

    /// Catches: YC+C is LiftEffect::Neutral.
    #[test]
    fn adv_yc_plus_c_neutral_lift() {
        let mut state = state_with_complex_matrix("A", 1, 1, vec![0.0, 0.0]);
        use rust_decimal::Decimal;
        state.stack.x = HpNum::rounded(Decimal::from_f64(1.0).unwrap());
        state.stack.y = HpNum::zero();
        state.adv_matrix_i = 1;
        state.adv_matrix_j = 1;
        state.stack.lift_enabled = true;
        op_adv_yc_plus_c(&mut state).unwrap();
        assert!(state.stack.lift_enabled, "YC+C must be LiftEffect::Neutral");
    }
}
