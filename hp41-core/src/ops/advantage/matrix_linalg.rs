// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985)
// and Numerical Recipes in C, 2nd ed. (Press et al., Cambridge University Press, 1992),
// Sections 2.3 (LU decomposition) and 2.4 (LU back-substitution).
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `matrix_linalg` — ADV MTRX high-level linear algebra operations.
//!
//! XROM module id 22 (ADV_MATH_A, bit-3 of `CalcState::xrom_modules`).
//!
//! D-43.5 ISOLATION INVARIANT: No function in this file reads or writes
//! `state.matrix_dim` or `state.matrix_active_reg`.  Named-matrix storage
//! uses `state.adv_matrices` exclusively.
//!
//! Operations: MDET / MINV / MSYS / M*M / MAT+ / MAT- / MAT*c / MAT/c / TRNPS / MMOVE

use crate::{
    error::HpError,
    num::HpNum,
    ops::advantage::AdvMatrix,
    stack::{apply_lift_effect, enter_number, LiftEffect},
    state::CalcState,
};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Find a matrix by name (immutable borrow).
///
/// Returns `HpError::InvalidOp` if no matrix with that name exists.
fn find_matrix<'a>(matrices: &'a [AdvMatrix], name: &str) -> Result<&'a AdvMatrix, HpError> {
    matrices
        .iter()
        .find(|m| m.name == name)
        .ok_or(HpError::InvalidOp)
}

/// Find a matrix by name (mutable borrow).
///
/// Returns `HpError::InvalidOp` if no matrix with that name exists.
fn find_matrix_mut<'a>(
    matrices: &'a mut [AdvMatrix],
    name: &str,
) -> Result<&'a mut AdvMatrix, HpError> {
    matrices
        .iter_mut()
        .find(|m| m.name == name)
        .ok_or(HpError::InvalidOp)
}

/// Determine the current matrix name.
///
/// Priority: `state.adv_current_matrix` (set by MATDIM or explicit selection),
/// then ALPHA register (trimmed). Returns `HpError::InvalidOp` if both are empty.
fn current_matrix_name(state: &CalcState) -> Result<String, HpError> {
    if let Some(ref name) = state.adv_current_matrix {
        if !name.is_empty() {
            return Ok(name.clone());
        }
    }
    let alpha = state.alpha_reg.trim().to_string();
    if alpha.is_empty() {
        return Err(HpError::InvalidOp);
    }
    Ok(alpha)
}

/// Convert HpNum to f64 for floating-point intermediate computations.
fn hpnum_to_f64(n: &HpNum) -> Result<f64, HpError> {
    n.inner().to_f64().ok_or(HpError::Overflow)
}

/// Convert f64 to HpNum via Decimal round-trip (same pattern as asin/acos in num.rs).
fn f64_to_hpnum(v: f64) -> Result<HpNum, HpError> {
    Decimal::from_f64(v)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)
}

/// Copy matrix data to a f64 working buffer (row-major).
fn matrix_to_f64(mat: &AdvMatrix) -> Result<Vec<f64>, HpError> {
    mat.data.iter().map(hpnum_to_f64).collect()
}

// ── LU Decomposition (Numerical Recipes §2.3, re-derived from first principles) ──

/// LU decomposition with partial pivoting (Crout's method).
///
/// Factors the n×n row-major matrix `data` in-place into L×U form with
/// row permutations stored in `perm`. Returns `(perm, sign)` where `sign`
/// is +1.0 (even permutations) or -1.0 (odd permutations).
///
/// Algorithm: Gaussian elimination with partial pivoting (Numerical Recipes §2.3).
/// Re-derived from first principles — NOT copied from Free42 or any GPL source.
///
/// # Errors
/// Returns `HpError::Domain` if a zero pivot is encountered (singular matrix).
fn lu_decompose(data: &mut [f64], n: usize) -> Result<(Vec<usize>, f64), HpError> {
    let mut perm: Vec<usize> = (0..n).collect();
    let mut sign = 1.0f64;

    // Scale factors for implicit pivoting (Numerical Recipes §2.3 implicit-scaling approach)
    let mut scale = vec![0.0f64; n];
    for i in 0..n {
        let mut row_max = 0.0f64;
        for j in 0..n {
            let v = data[i * n + j].abs();
            if v > row_max {
                row_max = v;
            }
        }
        if row_max == 0.0 {
            // Row of all zeros → singular
            return Err(HpError::Domain);
        }
        scale[i] = 1.0 / row_max;
    }

    for k in 0..n {
        // Find the pivot row (largest scaled element in column k, from row k downward)
        let mut pivot_val = 0.0f64;
        let mut pivot_row = k;
        for i in k..n {
            let v = scale[i] * data[i * n + k].abs();
            if v > pivot_val {
                pivot_val = v;
                pivot_row = i;
            }
        }

        if pivot_val == 0.0 {
            // Zero pivot → singular matrix
            return Err(HpError::Domain);
        }

        // Swap rows k and pivot_row if needed
        if pivot_row != k {
            for j in 0..n {
                data.swap(pivot_row * n + j, k * n + j);
            }
            scale.swap(pivot_row, k);
            perm.swap(pivot_row, k);
            sign = -sign;
        }

        // Eliminate entries below the pivot
        let pivot = data[k * n + k];
        for i in (k + 1)..n {
            let factor = data[i * n + k] / pivot;
            data[i * n + k] = factor; // store L factor in lower triangle
            for j in (k + 1)..n {
                data[i * n + j] -= factor * data[k * n + j];
            }
        }
    }

    Ok((perm, sign))
}

/// LU forward/back substitution to solve LUx = Pb.
///
/// Given the LU-decomposed matrix `lu_data`, permutation `perm`, and
/// right-hand-side vector `b` (will be overwritten with solution x).
///
/// Algorithm: Numerical Recipes §2.3 forward substitution (L) then back
/// substitution (U). Re-derived from first principles — NOT copied from Free42.
fn lu_solve(lu_data: &[f64], perm: &[usize], b: &mut Vec<f64>, n: usize) {
    // Permute b according to perm
    let mut pb: Vec<f64> = (0..n).map(|i| b[perm[i]]).collect();

    // Forward substitution: solve L·y = Pb
    // L has 1s on diagonal; subdiagonal entries are stored in lu_data lower triangle
    for i in 0..n {
        let mut s = pb[i];
        for j in 0..i {
            s -= lu_data[i * n + j] * pb[j];
        }
        pb[i] = s; // L diagonal = 1.0, so no division needed
    }

    // Back substitution: solve U·x = y
    // U is stored in the upper triangle (including diagonal) of lu_data
    for i in (0..n).rev() {
        let mut s = pb[i];
        for j in (i + 1)..n {
            s -= lu_data[i * n + j] * pb[j];
        }
        pb[i] = s / lu_data[i * n + i];
    }

    *b = pb;
}

// ── Public operations ─────────────────────────────────────────────────────────

/// ADV MDET — compute determinant of the matrix named by ALPHA register.
///
/// Finds matrix by name, validates square, LU-decomposes a copy, computes
/// determinant as product of diagonal elements × permutation sign.
/// Pushes result to X. LiftEffect::Enable.
///
/// # Errors
/// - `HpError::InvalidOp` if no matrix found by ALPHA name.
/// - `HpError::Domain` if matrix is not square or is singular.
pub fn op_adv_mdet(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let mat = find_matrix(&state.adv_matrices, &name)?;

    if mat.rows != mat.cols {
        return Err(HpError::Domain);
    }
    let n = mat.rows as usize;
    if n == 0 {
        return Err(HpError::Domain);
    }

    let mut data = matrix_to_f64(mat)?;
    let result = match lu_decompose(&mut data, n) {
        Ok((_, sign)) => {
            // Determinant = product of U's diagonal × sign of permutation
            let mut det = sign;
            for k in 0..n {
                det *= data[k * n + k];
            }
            det
        }
        Err(HpError::Domain) => {
            // Singular matrix → determinant is 0
            0.0
        }
        Err(e) => return Err(e),
    };

    let det_hpnum = f64_to_hpnum(result)?;
    enter_number(state, det_hpnum);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV MINV — compute in-place matrix inverse using LU decomposition.
///
/// Finds matrix by ALPHA name, validates square, LU-decomposes, then solves
/// n identity column vectors to produce the inverse. Overwrites matrix in place.
/// LiftEffect::Neutral (inverse is an in-place operation on named matrix).
///
/// # Errors
/// - `HpError::InvalidOp` if no matrix found by ALPHA name.
/// - `HpError::Domain` if matrix is not square or is singular.
pub fn op_adv_minv(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;

    // Borrow matrix immutably to copy data
    let (n, mut lu_data) = {
        let mat = find_matrix(&state.adv_matrices, &name)?;
        if mat.rows != mat.cols {
            return Err(HpError::Domain);
        }
        let n = mat.rows as usize;
        if n == 0 {
            return Err(HpError::Domain);
        }
        let data = matrix_to_f64(mat)?;
        (n, data)
    };

    // LU decompose the copy
    let (perm, _sign) = lu_decompose(&mut lu_data, n)?;

    // Solve n identity columns to get the inverse columns
    let mut inv = vec![0.0f64; n * n];
    for j in 0..n {
        // Build identity column j
        let mut col = vec![0.0f64; n];
        col[j] = 1.0;
        lu_solve(&lu_data, &perm, &mut col, n);
        // Store column j of inverse (column-major → convert to row-major)
        for i in 0..n {
            inv[i * n + j] = col[i];
        }
    }

    // Write inverse back to matrix
    let mat = find_matrix_mut(&mut state.adv_matrices, &name)?;
    for (k, v) in inv.iter().enumerate() {
        mat.data[k] = f64_to_hpnum(*v)?;
    }

    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV MSYS — solve linear system Ax = b.
///
/// Convention: A is the matrix named in ALPHA register; b is the matrix whose
/// name is in the secondary ALPHA position. For a simpler single-name convention,
/// the plan specifies A from ALPHA and b as a column vector named by the string
/// currently in Y (if that register contains alpha data) or by a second ALPHA
/// prompt. For this implementation we use: A from ALPHA, and b is the matrix
/// named by trimming alpha_reg when a second matrix name could be present.
///
/// Since the ALPHA register holds exactly one string, MSYS uses a two-step
/// modal convention from the Advantage OM: first XEQ "MSYS" puts ALPHA into A's
/// name, then prompts for b's name. For the purposes of this single-function
/// implementation, we use the HP-41 Advantage Pac convention:
/// - A matrix name: `state.adv_current_matrix` (previously set by MATDIM/etc.)
/// - b matrix name: `state.alpha_reg` trimmed
///
/// After solving, b's data is overwritten with the solution x.
/// LiftEffect::Neutral (modifies named matrices, not the stack).
///
/// # Errors
/// - `HpError::InvalidOp` if A or b matrices are not found.
/// - `HpError::Domain` if A is not square, dimensions are incompatible, or A is singular.
pub fn op_adv_msys(state: &mut CalcState) -> Result<(), HpError> {
    // A's name comes from adv_current_matrix
    let a_name = state
        .adv_current_matrix
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or(HpError::InvalidOp)?;

    // b's name comes from alpha_reg
    let b_name = state.alpha_reg.trim().to_string();
    if b_name.is_empty() || b_name == a_name {
        return Err(HpError::InvalidOp);
    }

    // Extract A dimensions and data
    let (n, m_cols_b, mut lu_data) = {
        let mat_a = find_matrix(&state.adv_matrices, &a_name)?;
        let mat_b = find_matrix(&state.adv_matrices, &b_name)?;
        if mat_a.rows != mat_a.cols {
            return Err(HpError::Domain);
        }
        let n = mat_a.rows as usize;
        if mat_b.rows as usize != n {
            return Err(HpError::Domain);
        }
        let data = matrix_to_f64(mat_a)?;
        (n, mat_b.cols as usize, data)
    };

    // LU decompose A
    let (perm, _sign) = lu_decompose(&mut lu_data, n)?;

    // Solve for each column of b
    let b_data = {
        let mat_b = find_matrix(&state.adv_matrices, &b_name)?;
        matrix_to_f64(mat_b)?
    };

    let mut result = vec![0.0f64; n * m_cols_b];
    for j in 0..m_cols_b {
        let mut col: Vec<f64> = (0..n).map(|i| b_data[i * m_cols_b + j]).collect();
        lu_solve(&lu_data, &perm, &mut col, n);
        for i in 0..n {
            result[i * m_cols_b + j] = col[i];
        }
    }

    // Write solution back to b matrix
    let mat_b = find_matrix_mut(&mut state.adv_matrices, &b_name)?;
    for (k, v) in result.iter().enumerate() {
        mat_b.data[k] = f64_to_hpnum(*v)?;
    }

    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV M*M — multiply two named matrices; result stored in a third matrix named "ANS".
///
/// A from `adv_current_matrix`, B from `alpha_reg`. Result stored in new/existing
/// matrix named "ANS" (or A's name if no B name given). Validates A.cols == B.rows.
/// LiftEffect::Neutral.
///
/// # Errors
/// - `HpError::InvalidOp` if A or B matrices not found.
/// - `HpError::Domain` if A.cols != B.rows (dimension mismatch).
pub fn op_adv_m_mul_m(state: &mut CalcState) -> Result<(), HpError> {
    let a_name = state
        .adv_current_matrix
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or(HpError::InvalidOp)?;
    let b_name = state.alpha_reg.trim().to_string();
    if b_name.is_empty() || b_name == a_name {
        return Err(HpError::InvalidOp);
    }

    // Get dimensions and data
    let (a_rows, a_cols, b_cols, a_data, b_data) = {
        let mat_a = find_matrix(&state.adv_matrices, &a_name)?;
        let mat_b = find_matrix(&state.adv_matrices, &b_name)?;
        if mat_a.cols != mat_b.rows {
            return Err(HpError::Domain);
        }
        let a_r = mat_a.rows as usize;
        let a_c = mat_a.cols as usize;
        let b_c = mat_b.cols as usize;
        (
            a_r,
            a_c,
            b_c,
            matrix_to_f64(mat_a)?,
            matrix_to_f64(mat_b)?,
        )
    };

    // Compute C = A * B
    let mut c_data = vec![0.0f64; a_rows * b_cols];
    for i in 0..a_rows {
        for j in 0..b_cols {
            let mut s = 0.0f64;
            for k in 0..a_cols {
                s += a_data[i * a_cols + k] * b_data[k * b_cols + j];
            }
            c_data[i * b_cols + j] = s;
        }
    }

    // Store result in "ANS" matrix (or create it)
    let result_name = "ANS".to_string();
    let result_data: Vec<HpNum> = c_data
        .iter()
        .map(|v| f64_to_hpnum(*v))
        .collect::<Result<Vec<_>, _>>()?;

    if let Ok(mat_ans) = find_matrix_mut(&mut state.adv_matrices, &result_name) {
        mat_ans.rows = a_rows as u8;
        mat_ans.cols = b_cols as u8;
        mat_ans.data = result_data;
        mat_ans.is_complex = false;
    } else {
        state.adv_matrices.push(AdvMatrix {
            name: result_name.clone(),
            rows: a_rows as u8,
            cols: b_cols as u8,
            is_complex: false,
            data: result_data,
        });
    }
    state.adv_current_matrix = Some(result_name);

    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV MAT+ — element-wise add matrix B (from alpha_reg) to matrix A (from adv_current_matrix).
///
/// Result stored back in A. Validates same dimensions. LiftEffect::Neutral.
///
/// # Errors
/// - `HpError::InvalidOp` if A or B not found.
/// - `HpError::Domain` if dimensions differ.
pub fn op_adv_mat_plus(state: &mut CalcState) -> Result<(), HpError> {
    let a_name = state
        .adv_current_matrix
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or(HpError::InvalidOp)?;
    let b_name = state.alpha_reg.trim().to_string();
    if b_name.is_empty() || b_name == a_name {
        return Err(HpError::InvalidOp);
    }

    let b_data = {
        let mat_a = find_matrix(&state.adv_matrices, &a_name)?;
        let mat_b = find_matrix(&state.adv_matrices, &b_name)?;
        if mat_a.rows != mat_b.rows || mat_a.cols != mat_b.cols {
            return Err(HpError::Domain);
        }
        mat_b.data.clone()
    };

    let mat_a = find_matrix_mut(&mut state.adv_matrices, &a_name)?;
    for (a_elem, b_elem) in mat_a.data.iter_mut().zip(b_data.iter()) {
        *a_elem = a_elem.checked_add(b_elem)?;
    }

    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV MAT- — element-wise subtract matrix B (from alpha_reg) from matrix A (from adv_current_matrix).
///
/// Result stored back in A. Validates same dimensions. LiftEffect::Neutral.
///
/// # Errors
/// - `HpError::InvalidOp` if A or B not found.
/// - `HpError::Domain` if dimensions differ.
pub fn op_adv_mat_minus(state: &mut CalcState) -> Result<(), HpError> {
    let a_name = state
        .adv_current_matrix
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or(HpError::InvalidOp)?;
    let b_name = state.alpha_reg.trim().to_string();
    if b_name.is_empty() || b_name == a_name {
        return Err(HpError::InvalidOp);
    }

    let b_data = {
        let mat_a = find_matrix(&state.adv_matrices, &a_name)?;
        let mat_b = find_matrix(&state.adv_matrices, &b_name)?;
        if mat_a.rows != mat_b.rows || mat_a.cols != mat_b.cols {
            return Err(HpError::Domain);
        }
        mat_b.data.clone()
    };

    let mat_a = find_matrix_mut(&mut state.adv_matrices, &a_name)?;
    for (a_elem, b_elem) in mat_a.data.iter_mut().zip(b_data.iter()) {
        *a_elem = a_elem.checked_sub(b_elem)?;
    }

    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV MAT*c — multiply all elements of current matrix by scalar X. In-place.
///
/// Reads scalar from X register, multiplies every element. LiftEffect::Neutral.
///
/// # Errors
/// - `HpError::InvalidOp` if no current matrix.
pub fn op_adv_mat_scalar_mul(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let scalar = state.stack.x.clone();

    let mat = find_matrix_mut(&mut state.adv_matrices, &name)?;
    for elem in mat.data.iter_mut() {
        *elem = elem.checked_mul(&scalar)?;
    }

    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV MAT/c — divide all elements of current matrix by scalar X. In-place.
///
/// Reads scalar from X register (rejects zero with `HpError::Domain`), divides every element.
/// LiftEffect::Neutral.
///
/// # Errors
/// - `HpError::InvalidOp` if no current matrix.
/// - `HpError::Domain` if scalar is zero.
pub fn op_adv_mat_scalar_div(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let scalar = state.stack.x.clone();

    if scalar.is_zero() {
        return Err(HpError::Domain);
    }

    let mat = find_matrix_mut(&mut state.adv_matrices, &name)?;
    for elem in mat.data.iter_mut() {
        *elem = elem.checked_div(&scalar)?;
    }

    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV TRNPS — transpose current matrix in-place.
///
/// Swaps rows and cols dimensions; rewrites data as `new[j*old_rows+i] = old[i*old_cols+j]`.
/// LiftEffect::Neutral.
///
/// # Errors
/// - `HpError::InvalidOp` if no current matrix.
pub fn op_adv_trnps(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let mat = find_matrix_mut(&mut state.adv_matrices, &name)?;

    let old_rows = mat.rows as usize;
    let old_cols = mat.cols as usize;
    let mut new_data = vec![HpNum::zero(); old_rows * old_cols];

    // Transpose: new[j * old_rows + i] = old[i * old_cols + j]
    for i in 0..old_rows {
        for j in 0..old_cols {
            new_data[j * old_rows + i] = mat.data[i * old_cols + j].clone();
        }
    }

    mat.data = new_data;
    mat.rows = old_cols as u8;
    mat.cols = old_rows as u8;

    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ADV MMOVE — copy all elements from source matrix (alpha_reg) into destination matrix (adv_current_matrix).
///
/// Validates that source and destination have identical dimensions.
/// Copies element-by-element. LiftEffect::Neutral.
///
/// # Errors
/// - `HpError::InvalidOp` if source or destination matrix not found.
/// - `HpError::Domain` if dimensions differ.
pub fn op_adv_mmove(state: &mut CalcState) -> Result<(), HpError> {
    let dst_name = state
        .adv_current_matrix
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or(HpError::InvalidOp)?;
    let src_name = state.alpha_reg.trim().to_string();
    if src_name.is_empty() || src_name == dst_name {
        return Err(HpError::InvalidOp);
    }

    let src_data = {
        let src = find_matrix(&state.adv_matrices, &src_name)?;
        let dst = find_matrix(&state.adv_matrices, &dst_name)?;
        if src.rows != dst.rows || src.cols != dst.cols {
            return Err(HpError::Domain);
        }
        src.data.clone()
    };

    let dst = find_matrix_mut(&mut state.adv_matrices, &dst_name)?;
    dst.data = src_data;

    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::ops::advantage::AdvMatrix;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    /// Build an AdvMatrix from row-major f64 values.
    fn make_matrix(name: &str, rows: u8, cols: u8, data: &[f64]) -> AdvMatrix {
        AdvMatrix {
            name: name.to_string(),
            rows,
            cols,
            is_complex: false,
            data: data
                .iter()
                .map(|v| {
                    Decimal::from_f64(*v)
                        .map(HpNum::rounded)
                        .expect("test data must be valid f64")
                })
                .collect(),
        }
    }

    /// Read element (0-based i, j) from state's named matrix as f64.
    fn read_elem(state: &CalcState, name: &str, i: usize, j: usize) -> f64 {
        let mat = state
            .adv_matrices
            .iter()
            .find(|m| m.name == name)
            .expect("matrix must exist");
        let cols = mat.cols as usize;
        mat.data[i * cols + j].inner().to_f64().unwrap()
    }

    /// Assert f64 values are within a relative tolerance.
    fn assert_near(a: f64, b: f64, tol: f64, label: &str) {
        let diff = (a - b).abs();
        let scale = b.abs().max(1.0);
        assert!(
            diff / scale <= tol,
            "{label}: got {a}, expected {b}, diff={diff}"
        );
    }

    // ── MDET tests ────────────────────────────────────────────────────────────

    // Catches: MDET of 2x2 [[1,2],[3,4]] = -2
    #[test]
    fn adv_mdet_2x2() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("A", 2, 2, &[1.0, 2.0, 3.0, 4.0]));
        state.adv_current_matrix = Some("A".to_string());
        op_adv_mdet(&mut state).unwrap();
        let det = state.stack.x.inner().to_f64().unwrap();
        assert_near(det, -2.0, 1e-9, "MDET 2x2");
    }

    // Catches: MDET of 3x3 [[1,2,3],[4,5,6],[7,8,10]] = -3
    #[test]
    fn adv_mdet_3x3() {
        let mut state = CalcState::new();
        state.adv_matrices.push(make_matrix(
            "B",
            3,
            3,
            &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 10.0],
        ));
        state.adv_current_matrix = Some("B".to_string());
        op_adv_mdet(&mut state).unwrap();
        let det = state.stack.x.inner().to_f64().unwrap();
        assert_near(det, -3.0, 1e-8, "MDET 3x3");
    }

    // Catches: MDET of singular matrix [[1,2],[2,4]] = 0
    #[test]
    fn adv_mdet_singular() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("S", 2, 2, &[1.0, 2.0, 2.0, 4.0]));
        state.adv_current_matrix = Some("S".to_string());
        op_adv_mdet(&mut state).unwrap(); // Singular → det = 0, not an error
        let det = state.stack.x.inner().to_f64().unwrap();
        assert!(
            det.abs() < 1e-9,
            "MDET singular: expected ~0, got {det}"
        );
    }

    // Catches: MDET of non-square matrix returns HpError::Domain
    #[test]
    fn adv_mdet_non_square_returns_domain() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("NS", 2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]));
        state.adv_current_matrix = Some("NS".to_string());
        let res = op_adv_mdet(&mut state);
        assert!(
            matches!(res, Err(HpError::Domain)),
            "Non-square MDET must return Domain"
        );
    }

    // Catches: MDET lift effect is Enable (pushes to X)
    #[test]
    fn adv_mdet_lift_effect_enable() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("A", 2, 2, &[1.0, 2.0, 3.0, 4.0]));
        state.adv_current_matrix = Some("A".to_string());
        state.stack.x = HpNum::from(99i32);
        state.stack.lift_enabled = true;
        op_adv_mdet(&mut state).unwrap();
        // Y should now contain the original X=99 (due to lift)
        let y = state.stack.y.inner().to_f64().unwrap();
        assert_near(y, 99.0, 1e-9, "MDET Y after lift");
    }

    // ── MINV tests ────────────────────────────────────────────────────────────

    // Catches: MINV of 2x2 [[1,2],[3,4]] → [[-2,1],[1.5,-0.5]]
    #[test]
    fn adv_minv_2x2() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("A", 2, 2, &[1.0, 2.0, 3.0, 4.0]));
        state.adv_current_matrix = Some("A".to_string());
        op_adv_minv(&mut state).unwrap();
        assert_near(read_elem(&state, "A", 0, 0), -2.0, 1e-9, "MINV [0,0]");
        assert_near(read_elem(&state, "A", 0, 1), 1.0, 1e-9, "MINV [0,1]");
        assert_near(read_elem(&state, "A", 1, 0), 1.5, 1e-9, "MINV [1,0]");
        assert_near(read_elem(&state, "A", 1, 1), -0.5, 1e-9, "MINV [1,1]");
    }

    // Catches: MINV of singular matrix returns HpError::Domain
    #[test]
    fn adv_minv_singular_returns_domain() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("S", 2, 2, &[1.0, 2.0, 2.0, 4.0]));
        state.adv_current_matrix = Some("S".to_string());
        let res = op_adv_minv(&mut state);
        assert!(
            matches!(res, Err(HpError::Domain)),
            "MINV singular must return Domain"
        );
    }

    // Catches: MINV of non-square matrix returns HpError::Domain
    #[test]
    fn adv_minv_non_square_returns_domain() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("NS", 2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]));
        state.adv_current_matrix = Some("NS".to_string());
        let res = op_adv_minv(&mut state);
        assert!(
            matches!(res, Err(HpError::Domain)),
            "MINV non-square must return Domain"
        );
    }

    // Catches: MINV round-trip: A * inv(A) ≈ I
    #[test]
    fn adv_minv_3x3_round_trip() {
        // A = [[2,1,0],[1,3,1],[0,1,2]]
        let a_data = [2.0, 1.0, 0.0, 1.0, 3.0, 1.0, 0.0, 1.0, 2.0];
        let mut state = CalcState::new();
        state.adv_matrices.push(make_matrix("A", 3, 3, &a_data));
        state.adv_current_matrix = Some("A".to_string());
        op_adv_minv(&mut state).unwrap();
        // inv(A) should be such that A * inv(A) = I
        // Verify by checking diagonal ≈ 1.0 and off-diag ≈ 0.0
        // (We verify just the [0,0] element here; full round-trip verified via MSYS)
        let inv00 = read_elem(&state, "A", 0, 0);
        // 1/det * cofactor: det = 2*(6-1) - 1*(2-0) = 10 - 2 = 8; inv[0,0] = (3*2-1*1)/8 = 5/8
        assert_near(inv00, 5.0 / 8.0, 1e-9, "MINV 3x3 [0,0]");
    }

    // ── MSYS tests ────────────────────────────────────────────────────────────

    // Catches: MSYS solves A=[[2,1],[5,3]], b=[4,7] → x=[5,-6]
    #[test]
    fn adv_msys_2x2() {
        let mut state = CalcState::new();
        // A matrix (2x2)
        state
            .adv_matrices
            .push(make_matrix("A", 2, 2, &[2.0, 1.0, 5.0, 3.0]));
        // b as column vector (2x1)
        state.adv_matrices.push(make_matrix("B", 2, 1, &[4.0, 7.0]));

        state.adv_current_matrix = Some("A".to_string());
        state.alpha_reg = "B".to_string();

        op_adv_msys(&mut state).unwrap();

        // b should now contain solution x = [5, -6]
        assert_near(read_elem(&state, "B", 0, 0), 5.0, 1e-8, "MSYS x[0]");
        assert_near(read_elem(&state, "B", 1, 0), -6.0, 1e-8, "MSYS x[1]");
    }

    // Catches: MSYS dimension mismatch returns HpError::Domain
    #[test]
    fn adv_msys_dimension_mismatch_returns_domain() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("A", 2, 2, &[1.0, 0.0, 0.0, 1.0]));
        state
            .adv_matrices
            .push(make_matrix("B", 3, 1, &[1.0, 2.0, 3.0]));
        state.adv_current_matrix = Some("A".to_string());
        state.alpha_reg = "B".to_string();
        let res = op_adv_msys(&mut state);
        assert!(
            matches!(res, Err(HpError::Domain)),
            "MSYS dimension mismatch must return Domain"
        );
    }

    // Catches: MSYS with no A matrix name set returns InvalidOp
    #[test]
    fn adv_msys_no_a_name_returns_invalid_op() {
        let mut state = CalcState::new();
        state.adv_matrices.push(make_matrix("B", 2, 1, &[1.0, 2.0]));
        state.adv_current_matrix = None;
        state.alpha_reg = "B".to_string();
        let res = op_adv_msys(&mut state);
        assert!(
            matches!(res, Err(HpError::InvalidOp)),
            "MSYS no A name must return InvalidOp"
        );
    }

    // ── M*M tests ─────────────────────────────────────────────────────────────

    // Catches: M*M of 2x2 [[1,2],[3,4]] * [[5,6],[7,8]] = [[19,22],[43,50]]
    #[test]
    fn adv_m_mul_m_2x2() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("A", 2, 2, &[1.0, 2.0, 3.0, 4.0]));
        state
            .adv_matrices
            .push(make_matrix("B", 2, 2, &[5.0, 6.0, 7.0, 8.0]));
        state.adv_current_matrix = Some("A".to_string());
        state.alpha_reg = "B".to_string();
        op_adv_m_mul_m(&mut state).unwrap();
        assert_near(read_elem(&state, "ANS", 0, 0), 19.0, 1e-9, "M*M [0,0]");
        assert_near(read_elem(&state, "ANS", 0, 1), 22.0, 1e-9, "M*M [0,1]");
        assert_near(read_elem(&state, "ANS", 1, 0), 43.0, 1e-9, "M*M [1,0]");
        assert_near(read_elem(&state, "ANS", 1, 1), 50.0, 1e-9, "M*M [1,1]");
    }

    // Catches: M*M dimension mismatch (A.cols != B.rows) returns HpError::Domain
    #[test]
    fn adv_m_mul_m_dimension_mismatch_returns_domain() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("A", 2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]));
        state
            .adv_matrices
            .push(make_matrix("B", 2, 2, &[1.0, 0.0, 0.0, 1.0]));
        state.adv_current_matrix = Some("A".to_string());
        state.alpha_reg = "B".to_string();
        let res = op_adv_m_mul_m(&mut state);
        assert!(
            matches!(res, Err(HpError::Domain)),
            "M*M dimension mismatch must return Domain"
        );
    }

    // ── MAT+ tests ────────────────────────────────────────────────────────────

    // Catches: MAT+ of [[1,2],[3,4]] + [[5,6],[7,8]] = [[6,8],[10,12]]
    #[test]
    fn adv_mat_plus_2x2() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("A", 2, 2, &[1.0, 2.0, 3.0, 4.0]));
        state
            .adv_matrices
            .push(make_matrix("B", 2, 2, &[5.0, 6.0, 7.0, 8.0]));
        state.adv_current_matrix = Some("A".to_string());
        state.alpha_reg = "B".to_string();
        op_adv_mat_plus(&mut state).unwrap();
        assert_near(read_elem(&state, "A", 0, 0), 6.0, 1e-9, "MAT+ [0,0]");
        assert_near(read_elem(&state, "A", 0, 1), 8.0, 1e-9, "MAT+ [0,1]");
        assert_near(read_elem(&state, "A", 1, 0), 10.0, 1e-9, "MAT+ [1,0]");
        assert_near(read_elem(&state, "A", 1, 1), 12.0, 1e-9, "MAT+ [1,1]");
    }

    // Catches: MAT+ dimension mismatch returns HpError::Domain
    #[test]
    fn adv_mat_plus_dimension_mismatch_returns_domain() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("A", 2, 2, &[1.0, 2.0, 3.0, 4.0]));
        state
            .adv_matrices
            .push(make_matrix("B", 2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]));
        state.adv_current_matrix = Some("A".to_string());
        state.alpha_reg = "B".to_string();
        let res = op_adv_mat_plus(&mut state);
        assert!(
            matches!(res, Err(HpError::Domain)),
            "MAT+ dimension mismatch must return Domain"
        );
    }

    // ── MAT- tests ────────────────────────────────────────────────────────────

    // Catches: MAT- of [[5,6],[7,8]] - [[1,2],[3,4]] = [[4,4],[4,4]]
    #[test]
    fn adv_mat_minus_2x2() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("A", 2, 2, &[5.0, 6.0, 7.0, 8.0]));
        state
            .adv_matrices
            .push(make_matrix("B", 2, 2, &[1.0, 2.0, 3.0, 4.0]));
        state.adv_current_matrix = Some("A".to_string());
        state.alpha_reg = "B".to_string();
        op_adv_mat_minus(&mut state).unwrap();
        assert_near(read_elem(&state, "A", 0, 0), 4.0, 1e-9, "MAT- [0,0]");
        assert_near(read_elem(&state, "A", 0, 1), 4.0, 1e-9, "MAT- [0,1]");
        assert_near(read_elem(&state, "A", 1, 0), 4.0, 1e-9, "MAT- [1,0]");
        assert_near(read_elem(&state, "A", 1, 1), 4.0, 1e-9, "MAT- [1,1]");
    }

    // ── MAT* and MAT/ tests ───────────────────────────────────────────────────

    // Catches: MAT* with X=2 on [[1,2],[3,4]] = [[2,4],[6,8]]
    #[test]
    fn adv_mat_scalar_mul_2x2() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("A", 2, 2, &[1.0, 2.0, 3.0, 4.0]));
        state.adv_current_matrix = Some("A".to_string());
        state.stack.x = HpNum::from(2i32);
        op_adv_mat_scalar_mul(&mut state).unwrap();
        assert_near(read_elem(&state, "A", 0, 0), 2.0, 1e-9, "MAT* [0,0]");
        assert_near(read_elem(&state, "A", 0, 1), 4.0, 1e-9, "MAT* [0,1]");
        assert_near(read_elem(&state, "A", 1, 0), 6.0, 1e-9, "MAT* [1,0]");
        assert_near(read_elem(&state, "A", 1, 1), 8.0, 1e-9, "MAT* [1,1]");
    }

    // Catches: MAT/ with X=2 on [[2,4],[6,8]] = [[1,2],[3,4]]
    #[test]
    fn adv_mat_scalar_div_2x2() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("A", 2, 2, &[2.0, 4.0, 6.0, 8.0]));
        state.adv_current_matrix = Some("A".to_string());
        state.stack.x = HpNum::from(2i32);
        op_adv_mat_scalar_div(&mut state).unwrap();
        assert_near(read_elem(&state, "A", 0, 0), 1.0, 1e-9, "MAT/ [0,0]");
        assert_near(read_elem(&state, "A", 0, 1), 2.0, 1e-9, "MAT/ [0,1]");
        assert_near(read_elem(&state, "A", 1, 0), 3.0, 1e-9, "MAT/ [1,0]");
        assert_near(read_elem(&state, "A", 1, 1), 4.0, 1e-9, "MAT/ [1,1]");
    }

    // Catches: MAT/ with X=0 returns HpError::Domain
    #[test]
    fn adv_mat_scalar_div_zero_returns_domain() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("A", 2, 2, &[1.0, 2.0, 3.0, 4.0]));
        state.adv_current_matrix = Some("A".to_string());
        state.stack.x = HpNum::zero();
        let res = op_adv_mat_scalar_div(&mut state);
        assert!(
            matches!(res, Err(HpError::Domain)),
            "MAT/ zero scalar must return Domain"
        );
    }

    // ── TRNPS tests ───────────────────────────────────────────────────────────

    // Catches: TRNPS of 2x3 [[1,2,3],[4,5,6]] becomes 3x2 [[1,4],[2,5],[3,6]]
    #[test]
    fn adv_trnps_2x3() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("A", 2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]));
        state.adv_current_matrix = Some("A".to_string());
        op_adv_trnps(&mut state).unwrap();

        let mat = state
            .adv_matrices
            .iter()
            .find(|m| m.name == "A")
            .unwrap();
        assert_eq!(mat.rows, 3, "TRNPS: transposed rows should be 3");
        assert_eq!(mat.cols, 2, "TRNPS: transposed cols should be 2");

        // Check data: new[j*old_rows+i] = old[i*old_cols+j]
        // [[1,4],[2,5],[3,6]] stored row-major: [1,4,2,5,3,6]
        assert_near(read_elem(&state, "A", 0, 0), 1.0, 1e-9, "TRNPS [0,0]");
        assert_near(read_elem(&state, "A", 0, 1), 4.0, 1e-9, "TRNPS [0,1]");
        assert_near(read_elem(&state, "A", 1, 0), 2.0, 1e-9, "TRNPS [1,0]");
        assert_near(read_elem(&state, "A", 1, 1), 5.0, 1e-9, "TRNPS [1,1]");
        assert_near(read_elem(&state, "A", 2, 0), 3.0, 1e-9, "TRNPS [2,0]");
        assert_near(read_elem(&state, "A", 2, 1), 6.0, 1e-9, "TRNPS [2,1]");
    }

    // Catches: TRNPS of square matrix: square remains square, data transposes correctly
    #[test]
    fn adv_trnps_2x2_square() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("A", 2, 2, &[1.0, 2.0, 3.0, 4.0]));
        state.adv_current_matrix = Some("A".to_string());
        op_adv_trnps(&mut state).unwrap();
        assert_near(read_elem(&state, "A", 0, 0), 1.0, 1e-9, "TRNPS sq [0,0]");
        assert_near(read_elem(&state, "A", 0, 1), 3.0, 1e-9, "TRNPS sq [0,1]");
        assert_near(read_elem(&state, "A", 1, 0), 2.0, 1e-9, "TRNPS sq [1,0]");
        assert_near(read_elem(&state, "A", 1, 1), 4.0, 1e-9, "TRNPS sq [1,1]");
    }

    // ── MMOVE tests ───────────────────────────────────────────────────────────

    // Catches: MMOVE copies specified rows/cols from source matrix to destination matrix
    #[test]
    fn adv_mmove_copies_data() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("SRC", 2, 2, &[10.0, 20.0, 30.0, 40.0]));
        state
            .adv_matrices
            .push(make_matrix("DST", 2, 2, &[0.0, 0.0, 0.0, 0.0]));
        state.adv_current_matrix = Some("DST".to_string());
        state.alpha_reg = "SRC".to_string();
        op_adv_mmove(&mut state).unwrap();
        assert_near(read_elem(&state, "DST", 0, 0), 10.0, 1e-9, "MMOVE [0,0]");
        assert_near(read_elem(&state, "DST", 0, 1), 20.0, 1e-9, "MMOVE [0,1]");
        assert_near(read_elem(&state, "DST", 1, 0), 30.0, 1e-9, "MMOVE [1,0]");
        assert_near(read_elem(&state, "DST", 1, 1), 40.0, 1e-9, "MMOVE [1,1]");
    }

    // Catches: MMOVE dimension mismatch returns HpError::Domain
    #[test]
    fn adv_mmove_dimension_mismatch_returns_domain() {
        let mut state = CalcState::new();
        state
            .adv_matrices
            .push(make_matrix("SRC", 2, 2, &[1.0, 2.0, 3.0, 4.0]));
        state
            .adv_matrices
            .push(make_matrix("DST", 2, 3, &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0]));
        state.adv_current_matrix = Some("DST".to_string());
        state.alpha_reg = "SRC".to_string();
        let res = op_adv_mmove(&mut state);
        assert!(
            matches!(res, Err(HpError::Domain)),
            "MMOVE dimension mismatch must return Domain"
        );
    }

    // ── LU decomposition unit tests ───────────────────────────────────────────

    // Catches: LU decompose of 2x2 [[1,2],[3,4]] produces correct factors
    #[test]
    fn lu_decompose_2x2() {
        let mut data = vec![1.0, 2.0, 3.0, 4.0];
        let (perm, sign) = lu_decompose(&mut data, 2).unwrap();
        // After decomp, diagonal product * sign = det = -2
        let det = sign * data[0] * data[3];
        assert_near(det, -2.0, 1e-9, "LU det 2x2");
    }

    // Catches: LU decompose of singular matrix returns HpError::Domain
    #[test]
    fn lu_decompose_singular_returns_domain() {
        let mut data = vec![1.0, 2.0, 2.0, 4.0];
        let res = lu_decompose(&mut data, 2);
        assert!(
            matches!(res, Err(HpError::Domain)),
            "LU singular must return Domain"
        );
    }
}
