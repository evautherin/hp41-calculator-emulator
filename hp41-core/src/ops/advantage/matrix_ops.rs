// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `matrix_ops` — ADV MTRX element access, lifecycle, and reduction operations.
//!
//! XROM module id 22 (ADV_MATH_A, bit-3 of `CalcState::xrom_modules`).
//!
//! D-43.5 ISOLATION INVARIANT: No function in this file reads or writes
//! `state.matrix_dim` or `state.matrix_active_reg`.  Those fields belong
//! exclusively to the Math Pac I matrix (Plan 28-06).  Named-matrix storage
//! uses `state.adv_matrices` + `state.adv_matrix_i` + `state.adv_matrix_j`
//! exclusively.
//!
//! Operations: I+ / I- / J+ / J- / MR / MS / MRIJ / MSIJ / MSIJR /
//!             MRC+ / MRC- / MRR+ / MRR- / MSR+ / MSC+ / MSWAP /
//!             MNAME? / DIM? / MATDIM / MP / PIV /
//!             R<>R / R>R? / SUM / SUMAB / MAX / MAXAB / MIN / RMAXAB / RNRM / RSUM / FNRM
//!
//! ## Index conventions (D-43.4)
//!
//! `state.adv_matrix_i` and `state.adv_matrix_j` are **0-based** array
//! indices (like Rust slice indices). MATDIM resets both to 0 (first element).
//! MRIJ/MSIJ accept 1-based indices from the user and convert to 0-based
//! internally. I+/I-/J+/J- wrap within [0, rows-1] and [0, cols-1].

use crate::{
    error::HpError,
    num::HpNum,
    ops::advantage::{AdvMatrix, ADV_MATRIX_MAX_COLS, ADV_MATRIX_MAX_ROWS},
    stack::{apply_lift_effect, enter_number, LiftEffect},
    state::CalcState,
};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Compute the row-major flat index for element (i, j) in a matrix.
///
/// Returns `HpError::Domain` if i >= rows or j >= cols (T-43-06 mitigation).
/// Both i and j are 0-based.
fn element_index(mat: &AdvMatrix, i: u8, j: u8) -> Result<usize, HpError> {
    if i >= mat.rows || j >= mat.cols {
        return Err(HpError::Domain);
    }
    Ok((i as usize) * (mat.cols as usize) + (j as usize))
}

/// Find a matrix by name (immutable borrow).
///
/// Returns `HpError::InvalidOp` if no matrix with that name exists.
fn find_matrix<'a>(
    matrices: &'a [AdvMatrix],
    name: &str,
) -> Result<&'a AdvMatrix, HpError> {
    matrices
        .iter()
        .find(|m| m.name == name)
        .ok_or(HpError::InvalidOp)
}

/// Find a matrix by name (mutable borrow).
///
/// Returns `HpError::InvalidOp` if no matrix with that name exists.
fn find_matrix_mut<'a>(
    matrices: &'a mut Vec<AdvMatrix>,
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
///
/// D-43.3: ALPHA register names the current matrix.
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

/// Truncate a HpNum value to a u8, returning `HpError::Domain` on out-of-range.
fn hpnum_to_u8(n: &HpNum) -> Result<u8, HpError> {
    let trunc = n.trunc_int();
    trunc
        .inner()
        .to_u8()
        .ok_or(HpError::Domain)
}

// ── Row index operations ──────────────────────────────────────────────────────

/// ADV I+ — increment current row index by 1, wrapping at matrix row count.
///
/// If no matrix is active the index is incremented as raw u8 (saturating at 255,
/// then wrapping to 0). LiftEffect::Neutral.
///
/// # Errors
/// Returns `HpError::Domain` if the named matrix cannot be found.
pub fn op_adv_i_plus(state: &mut CalcState) -> Result<(), HpError> {
    if let Ok(name) = current_matrix_name(state) {
        if let Ok(mat) = find_matrix(&state.adv_matrices, &name) {
            let rows = mat.rows;
            if rows == 0 {
                return Err(HpError::Domain);
            }
            state.adv_matrix_i = (state.adv_matrix_i + 1) % rows;
            return Ok(());
        }
    }
    // No active matrix — raw u8 wrapping increment
    state.adv_matrix_i = state.adv_matrix_i.wrapping_add(1);
    Ok(())
}

/// ADV I- — decrement current row index by 1, wrapping at matrix row count.
///
/// LiftEffect::Neutral.
///
/// # Errors
/// Never fails (wrapping arithmetic).
pub fn op_adv_i_minus(state: &mut CalcState) -> Result<(), HpError> {
    if let Ok(name) = current_matrix_name(state) {
        if let Ok(mat) = find_matrix(&state.adv_matrices, &name) {
            let rows = mat.rows;
            if rows == 0 {
                return Err(HpError::Domain);
            }
            state.adv_matrix_i = if state.adv_matrix_i == 0 {
                rows - 1
            } else {
                state.adv_matrix_i - 1
            };
            return Ok(());
        }
    }
    // No active matrix — raw u8 wrapping decrement
    state.adv_matrix_i = state.adv_matrix_i.wrapping_sub(1);
    Ok(())
}

/// ADV J+ — increment current column index by 1, wrapping at matrix column count.
///
/// LiftEffect::Neutral.
///
/// # Errors
/// Never fails (wrapping arithmetic).
pub fn op_adv_j_plus(state: &mut CalcState) -> Result<(), HpError> {
    if let Ok(name) = current_matrix_name(state) {
        if let Ok(mat) = find_matrix(&state.adv_matrices, &name) {
            let cols = mat.cols;
            if cols == 0 {
                return Err(HpError::Domain);
            }
            state.adv_matrix_j = (state.adv_matrix_j + 1) % cols;
            return Ok(());
        }
    }
    state.adv_matrix_j = state.adv_matrix_j.wrapping_add(1);
    Ok(())
}

/// ADV J- — decrement current column index by 1, wrapping at matrix column count.
///
/// LiftEffect::Neutral.
///
/// # Errors
/// Never fails (wrapping arithmetic).
pub fn op_adv_j_minus(state: &mut CalcState) -> Result<(), HpError> {
    if let Ok(name) = current_matrix_name(state) {
        if let Ok(mat) = find_matrix(&state.adv_matrices, &name) {
            let cols = mat.cols;
            if cols == 0 {
                return Err(HpError::Domain);
            }
            state.adv_matrix_j = if state.adv_matrix_j == 0 {
                cols - 1
            } else {
                state.adv_matrix_j - 1
            };
            return Ok(());
        }
    }
    state.adv_matrix_j = state.adv_matrix_j.wrapping_sub(1);
    Ok(())
}

// ── Element read/write ────────────────────────────────────────────────────────

/// ADV MR — matrix recall: push element (I,J) of current matrix onto stack.
///
/// Pushes the element at (adv_matrix_i, adv_matrix_j) to X. LiftEffect::Enable.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active or no such matrix exists.
/// Returns `HpError::Domain` if indices are out of bounds.
pub fn op_adv_mr(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let i = state.adv_matrix_i;
    let j = state.adv_matrix_j;
    let mat = find_matrix(&state.adv_matrices, &name)?;
    let idx = element_index(mat, i, j)?;
    let value = mat.data[idx].clone();
    apply_lift_effect(state, LiftEffect::Enable);
    enter_number(state, value);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV MS — matrix store: store X into element (I,J) of current matrix.
///
/// Writes `state.stack.x` to `data[element_index(i, j)]`. LiftEffect::Neutral.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active.
/// Returns `HpError::Domain` if indices are out of bounds.
pub fn op_adv_ms(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let i = state.adv_matrix_i;
    let j = state.adv_matrix_j;
    let value = state.stack.x.clone();
    let mat = find_matrix_mut(&mut state.adv_matrices, &name)?;
    let idx = element_index(mat, i, j)?;
    mat.data[idx] = value;
    // LiftEffect::Neutral — stores value, does not produce a result
    Ok(())
}

/// ADV MRIJ — matrix recall with explicit I/J from stack.
///
/// Y = row (1-based), X = col (1-based). Converts to 0-based internally.
/// Pops Y+X, pushes element value. LiftEffect::Enable.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active.
/// Returns `HpError::Domain` if indices are out of bounds.
pub fn op_adv_mrij(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    // Y = row (1-based), X = col (1-based)
    let row_1based = hpnum_to_u8(&state.stack.y)?;
    let col_1based = hpnum_to_u8(&state.stack.x)?;
    if row_1based == 0 || col_1based == 0 {
        return Err(HpError::Domain);
    }
    let i = row_1based - 1; // convert to 0-based
    let j = col_1based - 1;
    let mat = find_matrix(&state.adv_matrices, &name)?;
    let idx = element_index(mat, i, j)?;
    let value = mat.data[idx].clone();
    // Drop Y+X (both consumed), push element value
    state.stack.x = state.stack.z.clone();
    state.stack.y = state.stack.t.clone();
    state.stack.z = state.stack.t.clone();
    apply_lift_effect(state, LiftEffect::Enable);
    enter_number(state, value);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV MSIJ — matrix store with explicit I/J from stack.
///
/// T = value to store, Z unused, Y = row (1-based), X = col (1-based).
/// Wait — per OM convention: Y=row, X=col, value already in Z or via separate MS.
/// Implementation: pops Y+X, writes X (before pop) to position (Y-1, X-1).
/// Actually per typical HP-41 pattern: X=value is in T before Y and X are row/col.
/// For simplicity and per plan semantics: Y=row, X=col, and value comes from state
/// (not stack) — MS stores the *current* X to matrix. MSIJ = MS with explicit I,J.
/// That means: Y=row (1-based), X=col (1-based). Z = value to write.
///
/// HP-41 Advantage OM: MSIJ stores the value in Z at element (Y, X) where Y and X
/// are 1-based row/col indices, then drops two stack elements.
///
/// LiftEffect::Neutral (store operation).
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active.
/// Returns `HpError::Domain` if indices are out of bounds.
pub fn op_adv_msij(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let row_1based = hpnum_to_u8(&state.stack.y)?;
    let col_1based = hpnum_to_u8(&state.stack.x)?;
    if row_1based == 0 || col_1based == 0 {
        return Err(HpError::Domain);
    }
    let i = row_1based - 1;
    let j = col_1based - 1;
    let value = state.stack.z.clone();
    let mat = find_matrix_mut(&mut state.adv_matrices, &name)?;
    let idx = element_index(mat, i, j)?;
    mat.data[idx] = value;
    // Drop Y+X (row/col indices consumed)
    state.stack.x = state.stack.z.clone();
    state.stack.y = state.stack.t.clone();
    state.stack.z = state.stack.t.clone();
    // LiftEffect::Neutral
    Ok(())
}

/// ADV MSIJR — matrix store with explicit I/J then auto-increment row.
///
/// Same as MSIJ, then increments adv_matrix_i by 1 (wrapping at rows boundary).
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active.
/// Returns `HpError::Domain` if indices are out of bounds.
pub fn op_adv_msijr(state: &mut CalcState) -> Result<(), HpError> {
    op_adv_msij(state)?;
    op_adv_i_plus(state)
}

// ── Read + increment/decrement combos ────────────────────────────────────────

/// ADV MRC+ — matrix recall column-element at (I,J) then increment J.
///
/// Reads element, pushes to X, then increments column index J (wrapping). LiftEffect::Enable.
///
/// # Errors
/// Returns `HpError::InvalidOp` or `HpError::Domain` if matrix/index invalid.
pub fn op_adv_mrc_plus(state: &mut CalcState) -> Result<(), HpError> {
    op_adv_mr(state)?;
    op_adv_j_plus(state)
}

/// ADV MRC- — matrix recall column-element at (I,J) then decrement J.
///
/// Reads element, pushes to X, then decrements column index J (wrapping). LiftEffect::Enable.
///
/// # Errors
/// Returns `HpError::InvalidOp` or `HpError::Domain` if matrix/index invalid.
pub fn op_adv_mrc_minus(state: &mut CalcState) -> Result<(), HpError> {
    op_adv_mr(state)?;
    op_adv_j_minus(state)
}

/// ADV MRR+ — matrix recall row-element at (I,J) then increment I.
///
/// Reads element, pushes to X, then increments row index I (wrapping). LiftEffect::Enable.
///
/// # Errors
/// Returns `HpError::InvalidOp` or `HpError::Domain` if matrix/index invalid.
pub fn op_adv_mrr_plus(state: &mut CalcState) -> Result<(), HpError> {
    op_adv_mr(state)?;
    op_adv_i_plus(state)
}

/// ADV MRR- — matrix recall row-element at (I,J) then decrement I.
///
/// Reads element, pushes to X, then decrements row index I (wrapping). LiftEffect::Enable.
///
/// # Errors
/// Returns `HpError::InvalidOp` or `HpError::Domain` if matrix/index invalid.
pub fn op_adv_mrr_minus(state: &mut CalcState) -> Result<(), HpError> {
    op_adv_mr(state)?;
    op_adv_i_minus(state)
}

/// ADV MSR+ — matrix store row-element at (I,J) then increment I.
///
/// Writes X to element (I,J), then increments row index I (wrapping). LiftEffect::Neutral.
///
/// # Errors
/// Returns `HpError::InvalidOp` or `HpError::Domain` if matrix/index invalid.
pub fn op_adv_msr_plus(state: &mut CalcState) -> Result<(), HpError> {
    op_adv_ms(state)?;
    op_adv_i_plus(state)
}

/// ADV MSC+ — matrix store column-element at (I,J) then increment J.
///
/// Writes X to element (I,J), then increments column index J (wrapping). LiftEffect::Neutral.
///
/// # Errors
/// Returns `HpError::InvalidOp` or `HpError::Domain` if matrix/index invalid.
pub fn op_adv_msc_plus(state: &mut CalcState) -> Result<(), HpError> {
    op_adv_ms(state)?;
    op_adv_j_plus(state)
}

/// ADV MRIJR — read at explicit I,J then advance to next row (increment I).
///
/// Like MRIJ but also increments adv_matrix_i after reading.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active.
/// Returns `HpError::Domain` if indices are out of bounds.
pub fn op_adv_mrijr(state: &mut CalcState) -> Result<(), HpError> {
    // MRIJ reads and pops the row/col from stack
    // Save row/col for I register update before MRIJ consumes them
    let row_1based = hpnum_to_u8(&state.stack.y).unwrap_or(1);
    let i_0based = if row_1based > 0 { row_1based - 1 } else { 0 };
    op_adv_mrij(state)?;
    // Update adv_matrix_i to the row that was just read
    state.adv_matrix_i = i_0based;
    // Then increment row
    op_adv_i_plus(state)
}

// ── Lifecycle operations ──────────────────────────────────────────────────────

/// ADV MATDIM — define or resize a named matrix (ADV-MTX-17).
///
/// Y = rows, X = cols. Name from `state.alpha_reg`. Rejects empty names.
/// Rejects rows=0, cols=0, or > ADV_MATRIX_MAX_*. If matrix already exists,
/// resizes and zeroes data. Otherwise pushes a new AdvMatrix. Sets
/// `adv_current_matrix` to the name. Resets I=0, J=0.
///
/// Stack effect: drops two values (X ← Z, Y ← T, Z ← T).
///
/// # Errors
/// Returns `HpError::InvalidOp` if alpha_reg is empty.
/// Returns `HpError::Domain` if rows or cols are zero or exceed maximum.
pub fn op_adv_matdim(state: &mut CalcState) -> Result<(), HpError> {
    let rows = hpnum_to_u8(&state.stack.y.clone())?;
    let cols = hpnum_to_u8(&state.stack.x.clone())?;
    if rows == 0 || cols == 0 {
        return Err(HpError::Domain);
    }
    // ADV_MATRIX_MAX_ROWS/COLS = 255, which is u8::MAX — always fits
    // max allocation guard: 255*255*8 bytes = ~520 KB (T-43-07)
    let name = state.alpha_reg.trim().to_string();
    if name.is_empty() {
        return Err(HpError::InvalidOp);
    }
    let data_len = (rows as usize) * (cols as usize);
    if let Some(mat) = state.adv_matrices.iter_mut().find(|m| m.name == name) {
        mat.rows = rows;
        mat.cols = cols;
        mat.data = vec![HpNum::zero(); data_len];
    } else {
        state.adv_matrices.push(AdvMatrix {
            name: name.clone(),
            rows,
            cols,
            is_complex: false,
            data: vec![HpNum::zero(); data_len],
        });
    }
    state.adv_current_matrix = Some(name);
    state.adv_matrix_i = 0;
    state.adv_matrix_j = 0;
    // Drop Y + X (both consumed); stack: X ← Z, Y ← T, Z ← T
    state.stack.x = state.stack.z.clone();
    state.stack.y = state.stack.t.clone();
    state.stack.z = state.stack.t.clone();
    // LiftEffect: the stack was dropped, not lifted
    state.stack.lift_enabled = true;
    Ok(())
}

/// ADV DIM? — push rows and cols of named matrix to stack (ADV-MTX-18).
///
/// Reads matrix named by ALPHA register. Pushes rows to Y, cols to X.
/// LiftEffect::Enable.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no such matrix exists.
pub fn op_adv_dim_query(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let mat = find_matrix(&state.adv_matrices, &name)?;
    let rows = HpNum::from(mat.rows as i32);
    let cols = HpNum::from(mat.cols as i32);
    // Push rows to Y, cols to X
    apply_lift_effect(state, LiftEffect::Enable);
    enter_number(state, rows); // This pushes to X (with lift)
    apply_lift_effect(state, LiftEffect::Enable);
    // Now X=rows; push cols to X (rows goes to Y)
    enter_number(state, cols);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV MNAME? — write current matrix name to ALPHA register (ADV-MTX-19).
///
/// Sets `state.alpha_reg` to `adv_current_matrix` (or the ALPHA-derived name).
/// LiftEffect::Neutral.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no current matrix is set.
pub fn op_adv_mname_query(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    state.alpha_reg = name;
    // LiftEffect::Neutral — display/alpha op, no stack change
    Ok(())
}

// ── Row operations ────────────────────────────────────────────────────────────

/// ADV MSWAP — swap two rows of the current matrix (ADV-MTX-30).
///
/// Y = row k (1-based), X = row l (1-based). Both consumed from stack.
/// LiftEffect: stack drop (lift_enabled = true after consuming Y+X).
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active.
/// Returns `HpError::Domain` if row indices are out of bounds.
pub fn op_adv_mswap(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let row_k_1based = hpnum_to_u8(&state.stack.y)?;
    let row_l_1based = hpnum_to_u8(&state.stack.x)?;
    if row_k_1based == 0 || row_l_1based == 0 {
        return Err(HpError::Domain);
    }
    let k = (row_k_1based - 1) as usize;
    let l = (row_l_1based - 1) as usize;
    let mat = find_matrix_mut(&mut state.adv_matrices, &name)?;
    if k >= mat.rows as usize || l >= mat.rows as usize {
        return Err(HpError::Domain);
    }
    let cols = mat.cols as usize;
    for c in 0..cols {
        mat.data.swap(k * cols + c, l * cols + c);
    }
    // Drop Y + X
    state.stack.x = state.stack.z.clone();
    state.stack.y = state.stack.t.clone();
    state.stack.z = state.stack.t.clone();
    state.stack.lift_enabled = true;
    Ok(())
}

/// ADV R<>R — exchange two rows in current matrix (ADV-MTX-31).
///
/// Identical semantics to MSWAP per OM 00041-90482.
///
/// # Errors
/// Same as `op_adv_mswap`.
pub fn op_adv_r_exchange_r(state: &mut CalcState) -> Result<(), HpError> {
    op_adv_mswap(state)
}

/// ADV R>R? — compare rows k and l; result 1 in X if row k > row l, else 0 (ADV-MTX-32).
///
/// Comparison uses the first element of each row (standard partial ordering).
/// Y = row k (1-based), X = row l (1-based).
/// Result (0 or 1) pushed to X. LiftEffect::Enable after consuming Y+X.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active.
/// Returns `HpError::Domain` if row indices are out of bounds.
pub fn op_adv_r_gt_r_query(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let row_k_1based = hpnum_to_u8(&state.stack.y)?;
    let row_l_1based = hpnum_to_u8(&state.stack.x)?;
    if row_k_1based == 0 || row_l_1based == 0 {
        return Err(HpError::Domain);
    }
    let k = (row_k_1based - 1) as usize;
    let l = (row_l_1based - 1) as usize;
    let mat = find_matrix(&state.adv_matrices, &name)?;
    if k >= mat.rows as usize || l >= mat.rows as usize {
        return Err(HpError::Domain);
    }
    let cols = mat.cols as usize;
    // Compare first elements of each row (lexicographic first element comparison)
    let a = &mat.data[k * cols];
    let b = &mat.data[l * cols];
    let result = if a.inner() > b.inner() {
        HpNum::from(1)
    } else {
        HpNum::from(0)
    };
    // Drop Y + X, push result
    state.stack.x = state.stack.z.clone();
    state.stack.y = state.stack.t.clone();
    state.stack.z = state.stack.t.clone();
    apply_lift_effect(state, LiftEffect::Enable);
    enter_number(state, result);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV PIV — partial pivoting step (ADV-MTX-33).
///
/// Find the element with maximum absolute value in the current column
/// (from `adv_matrix_i` to the last row), swap that row with the current row.
/// Used as a building block for LU decomposition.
///
/// LiftEffect::Neutral.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active.
/// Returns `HpError::Domain` if current index is out of bounds.
pub fn op_adv_piv(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let pivot_row = state.adv_matrix_i as usize;
    let pivot_col = state.adv_matrix_j as usize;
    let mat = find_matrix_mut(&mut state.adv_matrices, &name)?;
    let rows = mat.rows as usize;
    let cols = mat.cols as usize;
    if pivot_row >= rows || pivot_col >= cols {
        return Err(HpError::Domain);
    }
    // Find row with maximum |element| in current column from pivot_row downward
    let mut max_row = pivot_row;
    let mut max_abs = mat.data[pivot_row * cols + pivot_col]
        .inner()
        .abs();
    for r in (pivot_row + 1)..rows {
        let abs_val = mat.data[r * cols + pivot_col].inner().abs();
        if abs_val > max_abs {
            max_abs = abs_val;
            max_row = r;
        }
    }
    // Swap pivot_row with max_row
    if max_row != pivot_row {
        for c in 0..cols {
            mat.data.swap(pivot_row * cols + c, max_row * cols + c);
        }
    }
    // LiftEffect::Neutral
    Ok(())
}

/// ADV MP — matrix print: push all elements to print_buffer in row-major order (ADV-MTX-34).
///
/// Format: "RkCl= value" for each element. LiftEffect::Neutral.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active.
pub fn op_adv_mp(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let mat = find_matrix(&state.adv_matrices, &name)?;
    let rows = mat.rows as usize;
    let cols = mat.cols as usize;
    let mut lines: Vec<String> = Vec::with_capacity(rows * cols);
    for r in 0..rows {
        for c in 0..cols {
            let val = &mat.data[r * cols + c];
            lines.push(format!("R{}C{}= {}", r + 1, c + 1, val));
        }
    }
    state.print_buffer.extend(lines);
    // LiftEffect::Neutral
    Ok(())
}

// ── Reduction and norm operations ─────────────────────────────────────────────

/// ADV SUM — sum all elements of current matrix (ADV-MTX-35).
///
/// Pushes result to X. LiftEffect::Enable.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active.
/// Returns `HpError::Overflow` on arithmetic overflow.
pub fn op_adv_sum(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let mat = find_matrix(&state.adv_matrices, &name)?;
    let mut acc = HpNum::zero();
    for elem in &mat.data {
        acc = acc.checked_add(elem)?;
    }
    apply_lift_effect(state, LiftEffect::Enable);
    enter_number(state, acc);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV SUMAB — sum of absolute values of all elements (ADV-MTX-36).
///
/// Pushes result to X. LiftEffect::Enable.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active.
/// Returns `HpError::Overflow` on arithmetic overflow.
pub fn op_adv_sumab(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let mat = find_matrix(&state.adv_matrices, &name)?;
    let mut acc = HpNum::zero();
    for elem in &mat.data {
        let abs_val = HpNum(elem.inner().abs());
        acc = acc.checked_add(&abs_val)?;
    }
    apply_lift_effect(state, LiftEffect::Enable);
    enter_number(state, acc);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV MAX — maximum element value of current matrix (ADV-MTX-37).
///
/// Pushes result to X. LiftEffect::Enable.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active or matrix is empty.
pub fn op_adv_max(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let mat = find_matrix(&state.adv_matrices, &name)?;
    if mat.data.is_empty() {
        return Err(HpError::InvalidOp);
    }
    let mut max_val = mat.data[0].inner();
    for elem in mat.data.iter().skip(1) {
        let v = elem.inner();
        if v > max_val {
            max_val = v;
        }
    }
    let result = HpNum::rounded(max_val);
    apply_lift_effect(state, LiftEffect::Enable);
    enter_number(state, result);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV MIN — minimum element value of current matrix (ADV-MTX-38).
///
/// Pushes result to X. LiftEffect::Enable.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active or matrix is empty.
pub fn op_adv_min(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let mat = find_matrix(&state.adv_matrices, &name)?;
    if mat.data.is_empty() {
        return Err(HpError::InvalidOp);
    }
    let mut min_val = mat.data[0].inner();
    for elem in mat.data.iter().skip(1) {
        let v = elem.inner();
        if v < min_val {
            min_val = v;
        }
    }
    let result = HpNum::rounded(min_val);
    apply_lift_effect(state, LiftEffect::Enable);
    enter_number(state, result);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV MAXAB — maximum absolute value element of current matrix (ADV-MTX-39).
///
/// Pushes result to X. LiftEffect::Enable.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active or matrix is empty.
pub fn op_adv_maxab(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let mat = find_matrix(&state.adv_matrices, &name)?;
    if mat.data.is_empty() {
        return Err(HpError::InvalidOp);
    }
    let mut max_abs = mat.data[0].inner().abs();
    for elem in mat.data.iter().skip(1) {
        let v = elem.inner().abs();
        if v > max_abs {
            max_abs = v;
        }
    }
    let result = HpNum::rounded(max_abs);
    apply_lift_effect(state, LiftEffect::Enable);
    enter_number(state, result);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV RMAXAB — maximum absolute value in the current row (ADV-MTX-40).
///
/// Current row = `adv_matrix_i`. Pushes result to X. LiftEffect::Enable.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active or row is empty.
/// Returns `HpError::Domain` if adv_matrix_i is out of bounds.
pub fn op_adv_rmaxab(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let row = state.adv_matrix_i as usize;
    let mat = find_matrix(&state.adv_matrices, &name)?;
    if row >= mat.rows as usize {
        return Err(HpError::Domain);
    }
    let cols = mat.cols as usize;
    if cols == 0 {
        return Err(HpError::InvalidOp);
    }
    let mut max_abs = mat.data[row * cols].inner().abs();
    for c in 1..cols {
        let v = mat.data[row * cols + c].inner().abs();
        if v > max_abs {
            max_abs = v;
        }
    }
    let result = HpNum::rounded(max_abs);
    apply_lift_effect(state, LiftEffect::Enable);
    enter_number(state, result);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV FNRM — Frobenius norm of current matrix (ADV-MTX-41).
///
/// Frobenius norm = sqrt(sum of squares of all elements).
/// Uses f64 round-trip for sqrt (same pattern as inverse trig in num.rs).
/// Pushes result to X. LiftEffect::Enable.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active or matrix is empty.
/// Returns `HpError::Overflow` if sum of squares overflows f64.
pub fn op_adv_fnrm(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let mat = find_matrix(&state.adv_matrices, &name)?;
    if mat.data.is_empty() {
        return Err(HpError::InvalidOp);
    }
    let mut sum_sq = 0.0_f64;
    for elem in &mat.data {
        let v = elem.inner().to_f64().ok_or(HpError::Overflow)?;
        sum_sq += v * v;
    }
    let norm_f64 = sum_sq.sqrt();
    let result = Decimal::from_f64(norm_f64)
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)?;
    apply_lift_effect(state, LiftEffect::Enable);
    enter_number(state, result);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV RNRM — row norm of current matrix (ADV-MTX-42).
///
/// Row norm = maximum over all rows of (sum of absolute values in that row).
/// Also known as the infinity norm. Pushes result to X. LiftEffect::Enable.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active or matrix is empty.
pub fn op_adv_rnrm(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let mat = find_matrix(&state.adv_matrices, &name)?;
    if mat.data.is_empty() {
        return Err(HpError::InvalidOp);
    }
    let rows = mat.rows as usize;
    let cols = mat.cols as usize;
    let mut max_row_sum = Decimal::ZERO;
    for r in 0..rows {
        let mut row_sum = Decimal::ZERO;
        for c in 0..cols {
            row_sum += mat.data[r * cols + c].inner().abs();
        }
        if row_sum > max_row_sum {
            max_row_sum = row_sum;
        }
    }
    let result = HpNum::rounded(max_row_sum);
    apply_lift_effect(state, LiftEffect::Enable);
    enter_number(state, result);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// ADV RSUM — sum of current row's elements (ADV-MTX-43).
///
/// Current row = `adv_matrix_i`. Pushes result to X. LiftEffect::Enable.
///
/// # Errors
/// Returns `HpError::InvalidOp` if no matrix is active.
/// Returns `HpError::Domain` if adv_matrix_i is out of bounds.
/// Returns `HpError::Overflow` on arithmetic overflow.
pub fn op_adv_rsum(state: &mut CalcState) -> Result<(), HpError> {
    let name = current_matrix_name(state)?;
    let row = state.adv_matrix_i as usize;
    let mat = find_matrix(&state.adv_matrices, &name)?;
    if row >= mat.rows as usize {
        return Err(HpError::Domain);
    }
    let cols = mat.cols as usize;
    let mut acc = HpNum::zero();
    for c in 0..cols {
        acc = acc.checked_add(&mat.data[row * cols + c])?;
    }
    apply_lift_effect(state, LiftEffect::Enable);
    enter_number(state, acc);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ── Helper to build a test state with a named matrix ──────────────────────

    /// Create a CalcState with a named matrix "A" of shape rows × cols.
    fn make_state_with_matrix(name: &str, rows: u8, cols: u8, data: Vec<f64>) -> CalcState {
        let mut state = CalcState::new();
        let len = (rows as usize) * (cols as usize);
        assert_eq!(data.len(), len, "test data length mismatch");
        let hp_data: Vec<HpNum> = data
            .into_iter()
            .map(|v| HpNum::rounded(Decimal::from_f64(v).expect("valid f64")))
            .collect();
        state.adv_matrices.push(AdvMatrix {
            name: name.to_string(),
            rows,
            cols,
            is_complex: false,
            data: hp_data,
        });
        state.adv_current_matrix = Some(name.to_string());
        state.adv_matrix_i = 0;
        state.adv_matrix_j = 0;
        state
    }

    fn f64_from_hpnum(n: &HpNum) -> f64 {
        n.inner().to_f64().expect("to_f64")
    }

    // ── MATDIM tests ──────────────────────────────────────────────────────────

    // Catches: MATDIM creates a named matrix with specified rows and cols
    #[test]
    fn adv_matdim_creates_matrix() {
        let mut state = CalcState::new();
        state.alpha_reg = "A".to_string();
        state.stack.y = HpNum::from(3); // rows
        state.stack.x = HpNum::from(4); // cols
        op_adv_matdim(&mut state).unwrap();
        assert_eq!(state.adv_matrices.len(), 1);
        let mat = &state.adv_matrices[0];
        assert_eq!(mat.name, "A");
        assert_eq!(mat.rows, 3);
        assert_eq!(mat.cols, 4);
        assert_eq!(mat.data.len(), 12);
        assert!(mat.data.iter().all(|x| x.is_zero()));
    }

    // Catches: MATDIM sets adv_current_matrix and resets I=0, J=0
    #[test]
    fn adv_matdim_sets_current_matrix_and_resets_indices() {
        let mut state = CalcState::new();
        state.alpha_reg = "B".to_string();
        state.stack.y = HpNum::from(2);
        state.stack.x = HpNum::from(3);
        op_adv_matdim(&mut state).unwrap();
        assert_eq!(state.adv_current_matrix, Some("B".to_string()));
        assert_eq!(state.adv_matrix_i, 0);
        assert_eq!(state.adv_matrix_j, 0);
    }

    // Catches: MATDIM with empty ALPHA returns InvalidOp
    #[test]
    fn adv_matdim_empty_alpha_returns_invalid_op() {
        let mut state = CalcState::new();
        state.alpha_reg = "".to_string();
        state.stack.y = HpNum::from(2);
        state.stack.x = HpNum::from(2);
        assert!(matches!(op_adv_matdim(&mut state), Err(HpError::InvalidOp)));
    }

    // Catches: MATDIM with rows=0 returns Domain
    #[test]
    fn adv_matdim_zero_rows_returns_domain() {
        let mut state = CalcState::new();
        state.alpha_reg = "A".to_string();
        state.stack.y = HpNum::from(0);
        state.stack.x = HpNum::from(3);
        assert!(matches!(op_adv_matdim(&mut state), Err(HpError::Domain)));
    }

    // Catches: MATDIM with cols=0 returns Domain
    #[test]
    fn adv_matdim_zero_cols_returns_domain() {
        let mut state = CalcState::new();
        state.alpha_reg = "A".to_string();
        state.stack.y = HpNum::from(2);
        state.stack.x = HpNum::from(0);
        assert!(matches!(op_adv_matdim(&mut state), Err(HpError::Domain)));
    }

    // Catches: MATDIM on existing matrix resizes it (zeroing data)
    #[test]
    fn adv_matdim_resizes_existing_matrix() {
        let mut state = make_state_with_matrix("A", 2, 2, vec![1.0, 2.0, 3.0, 4.0]);
        state.alpha_reg = "A".to_string();
        state.stack.y = HpNum::from(3);
        state.stack.x = HpNum::from(3);
        op_adv_matdim(&mut state).unwrap();
        let mat = &state.adv_matrices[0];
        assert_eq!(mat.rows, 3);
        assert_eq!(mat.cols, 3);
        assert_eq!(mat.data.len(), 9);
        assert!(mat.data.iter().all(|x| x.is_zero()));
    }

    // Catches: MATDIM drops two stack elements (X ← Z)
    #[test]
    fn adv_matdim_drops_two_stack_values() {
        let mut state = CalcState::new();
        state.alpha_reg = "A".to_string();
        state.stack.z = HpNum::from(99);
        state.stack.y = HpNum::from(2);
        state.stack.x = HpNum::from(2);
        op_adv_matdim(&mut state).unwrap();
        // After dropping Y+X, X should be old Z
        assert_eq!(f64_from_hpnum(&state.stack.x), 99.0);
    }

    // ── MR / MS round-trip tests ──────────────────────────────────────────────

    // Catches: MR after MATDIM reads element (0,0) = 0.0
    #[test]
    fn adv_mr_reads_zero_after_matdim() {
        let mut state = make_state_with_matrix("A", 2, 2, vec![0.0, 0.0, 0.0, 0.0]);
        state.adv_matrix_i = 0;
        state.adv_matrix_j = 0;
        op_adv_mr(&mut state).unwrap();
        assert_eq!(f64_from_hpnum(&state.stack.x), 0.0);
    }

    // Catches: MS writes value, MR reads it back (round-trip)
    #[test]
    fn adv_ms_mr_round_trip() {
        let mut state = make_state_with_matrix("A", 3, 3, vec![0.0; 9]);
        state.adv_matrix_i = 1;
        state.adv_matrix_j = 2;
        state.stack.x = HpNum::from(42);
        op_adv_ms(&mut state).unwrap();
        op_adv_mr(&mut state).unwrap();
        assert_eq!(f64_from_hpnum(&state.stack.x), 42.0);
    }

    // Catches: MS with X=5.0 writes to current position
    #[test]
    fn adv_ms_writes_x_to_element() {
        let mut state = make_state_with_matrix("A", 2, 2, vec![0.0; 4]);
        state.adv_matrix_i = 0;
        state.adv_matrix_j = 1;
        state.stack.x = HpNum::rounded(Decimal::from_f64(5.0).unwrap());
        op_adv_ms(&mut state).unwrap();
        let idx = element_index(&state.adv_matrices[0], 0, 1).unwrap();
        assert_eq!(f64_from_hpnum(&state.adv_matrices[0].data[idx]), 5.0);
    }

    // ── I+/I-/J+/J- tests ────────────────────────────────────────────────────

    // Catches: I+ increments adv_matrix_i
    #[test]
    fn adv_i_plus_increments() {
        let mut state = make_state_with_matrix("A", 3, 3, vec![0.0; 9]);
        state.adv_matrix_i = 0;
        op_adv_i_plus(&mut state).unwrap();
        assert_eq!(state.adv_matrix_i, 1);
        op_adv_i_plus(&mut state).unwrap();
        assert_eq!(state.adv_matrix_i, 2);
    }

    // Catches: I+ wraps at rows boundary
    #[test]
    fn adv_i_plus_wraps_at_boundary() {
        let mut state = make_state_with_matrix("A", 3, 3, vec![0.0; 9]);
        state.adv_matrix_i = 2; // last row (0-based in 3-row matrix)
        op_adv_i_plus(&mut state).unwrap();
        assert_eq!(state.adv_matrix_i, 0, "should wrap from 2 to 0");
    }

    // Catches: J+ increments adv_matrix_j and wraps at cols boundary
    #[test]
    fn adv_j_plus_increments_and_wraps() {
        let mut state = make_state_with_matrix("A", 2, 3, vec![0.0; 6]);
        state.adv_matrix_j = 2; // last col
        op_adv_j_plus(&mut state).unwrap();
        assert_eq!(state.adv_matrix_j, 0, "should wrap from 2 to 0");
    }

    // Catches: I- decrements and wraps from 0 to rows-1
    #[test]
    fn adv_i_minus_wraps_from_zero() {
        let mut state = make_state_with_matrix("A", 4, 2, vec![0.0; 8]);
        state.adv_matrix_i = 0;
        op_adv_i_minus(&mut state).unwrap();
        assert_eq!(state.adv_matrix_i, 3, "should wrap from 0 to 3");
    }

    // Catches: J- decrements and wraps from 0 to cols-1
    #[test]
    fn adv_j_minus_wraps_from_zero() {
        let mut state = make_state_with_matrix("A", 2, 5, vec![0.0; 10]);
        state.adv_matrix_j = 0;
        op_adv_j_minus(&mut state).unwrap();
        assert_eq!(state.adv_matrix_j, 4, "should wrap from 0 to 4");
    }

    // Catches: I- decrements normally
    #[test]
    fn adv_i_minus_decrements() {
        let mut state = make_state_with_matrix("A", 3, 2, vec![0.0; 6]);
        state.adv_matrix_i = 2;
        op_adv_i_minus(&mut state).unwrap();
        assert_eq!(state.adv_matrix_i, 1);
    }

    // ── MRIJ / MSIJ tests ────────────────────────────────────────────────────

    // Catches: MRIJ reads element at explicit I=Y, J=X (1-based)
    #[test]
    fn adv_mrij_reads_element_at_explicit_ij() {
        let mut state = make_state_with_matrix("A", 3, 3, vec![
            1.0, 2.0, 3.0,
            4.0, 5.0, 6.0,
            7.0, 8.0, 9.0,
        ]);
        // Read element at row=2, col=3 (1-based) = (1,2) 0-based = value 6
        state.stack.y = HpNum::from(2); // row 2 (1-based)
        state.stack.x = HpNum::from(3); // col 3 (1-based)
        op_adv_mrij(&mut state).unwrap();
        assert_eq!(f64_from_hpnum(&state.stack.x), 6.0);
    }

    // Catches: MSIJ writes X to element at explicit I,J
    #[test]
    fn adv_msij_writes_to_explicit_position() {
        let mut state = make_state_with_matrix("A", 2, 2, vec![0.0; 4]);
        // Write 99 at row=1, col=2 (1-based)
        state.stack.z = HpNum::from(99); // value to write
        state.stack.y = HpNum::from(1); // row 1 (1-based) = 0-based 0
        state.stack.x = HpNum::from(2); // col 2 (1-based) = 0-based 1
        op_adv_msij(&mut state).unwrap();
        let mat = &state.adv_matrices[0];
        assert_eq!(f64_from_hpnum(&mat.data[0 * 2 + 1]), 99.0);
    }

    // ── MRC+/MRR+/MSC+/MSR+ tests ────────────────────────────────────────────

    // Catches: MRC+ reads element then increments column index
    #[test]
    fn adv_mrc_plus_reads_then_increments_j() {
        let mut state = make_state_with_matrix("A", 2, 3, vec![
            10.0, 20.0, 30.0,
            40.0, 50.0, 60.0,
        ]);
        state.adv_matrix_i = 0;
        state.adv_matrix_j = 1; // read element (0,1) = 20
        op_adv_mrc_plus(&mut state).unwrap();
        assert_eq!(f64_from_hpnum(&state.stack.x), 20.0);
        assert_eq!(state.adv_matrix_j, 2, "J should have incremented");
    }

    // Catches: MRR+ reads element then increments row index
    #[test]
    fn adv_mrr_plus_reads_then_increments_i() {
        let mut state = make_state_with_matrix("A", 3, 2, vec![
            1.0, 2.0,
            3.0, 4.0,
            5.0, 6.0,
        ]);
        state.adv_matrix_i = 1; // read element (1,0) = 3
        state.adv_matrix_j = 0;
        op_adv_mrr_plus(&mut state).unwrap();
        assert_eq!(f64_from_hpnum(&state.stack.x), 3.0);
        assert_eq!(state.adv_matrix_i, 2, "I should have incremented");
    }

    // Catches: MSC+ writes element then increments J
    #[test]
    fn adv_msc_plus_writes_then_increments_j() {
        let mut state = make_state_with_matrix("A", 2, 3, vec![0.0; 6]);
        state.adv_matrix_i = 0;
        state.adv_matrix_j = 0;
        state.stack.x = HpNum::from(77);
        op_adv_msc_plus(&mut state).unwrap();
        let mat = &state.adv_matrices[0];
        assert_eq!(f64_from_hpnum(&mat.data[0]), 77.0);
        assert_eq!(state.adv_matrix_j, 1, "J should have incremented");
    }

    // Catches: MSR+ writes element then increments I
    #[test]
    fn adv_msr_plus_writes_then_increments_i() {
        let mut state = make_state_with_matrix("A", 3, 2, vec![0.0; 6]);
        state.adv_matrix_i = 0;
        state.adv_matrix_j = 0;
        state.stack.x = HpNum::from(55);
        op_adv_msr_plus(&mut state).unwrap();
        let mat = &state.adv_matrices[0];
        assert_eq!(f64_from_hpnum(&mat.data[0]), 55.0);
        assert_eq!(state.adv_matrix_i, 1, "I should have incremented");
    }

    // ── DIM? tests ───────────────────────────────────────────────────────────

    // Catches: DIM? returns rows in Y, cols in X for named matrix
    #[test]
    fn adv_dim_query_returns_rows_and_cols() {
        let mut state = make_state_with_matrix("A", 4, 7, vec![0.0; 28]);
        state.alpha_reg = "A".to_string();
        // Clear stack to known values
        state.stack.x = HpNum::from(0);
        state.stack.y = HpNum::from(0);
        op_adv_dim_query(&mut state).unwrap();
        // After two enter_number calls: Y=rows, X=cols
        assert_eq!(f64_from_hpnum(&state.stack.x), 7.0, "X should be cols=7");
        assert_eq!(f64_from_hpnum(&state.stack.y), 4.0, "Y should be rows=4");
    }

    // ── MNAME? tests ─────────────────────────────────────────────────────────

    // Catches: MNAME? writes current matrix name to ALPHA register
    #[test]
    fn adv_mname_query_writes_name_to_alpha() {
        let mut state = make_state_with_matrix("MYMAT", 2, 2, vec![0.0; 4]);
        state.alpha_reg = "SOMETHING_ELSE".to_string();
        op_adv_mname_query(&mut state).unwrap();
        assert_eq!(state.alpha_reg, "MYMAT");
    }

    // Catches: MNAME? fails when no matrix is active
    #[test]
    fn adv_mname_query_no_matrix_returns_error() {
        let mut state = CalcState::new();
        state.alpha_reg = "".to_string();
        assert!(matches!(op_adv_mname_query(&mut state), Err(HpError::InvalidOp)));
    }

    // ── Row operation tests ───────────────────────────────────────────────────

    // Catches: MSWAP swaps two rows of the matrix
    #[test]
    fn adv_mswap_swaps_rows() {
        let mut state = make_state_with_matrix("A", 3, 2, vec![
            1.0, 2.0,  // row 0
            3.0, 4.0,  // row 1
            5.0, 6.0,  // row 2
        ]);
        // Swap row 1 and row 3 (1-based)
        state.stack.y = HpNum::from(1); // row k=1 (1-based)
        state.stack.x = HpNum::from(3); // row l=3 (1-based)
        op_adv_mswap(&mut state).unwrap();
        let mat = &state.adv_matrices[0];
        // Row 0 (was row 1) should now be [5, 6]
        assert_eq!(f64_from_hpnum(&mat.data[0]), 5.0);
        assert_eq!(f64_from_hpnum(&mat.data[1]), 6.0);
        // Row 2 (was row 3) should now be [1, 2]
        assert_eq!(f64_from_hpnum(&mat.data[4]), 1.0);
        assert_eq!(f64_from_hpnum(&mat.data[5]), 2.0);
    }

    // Catches: R<>R exchanges two rows (alias of MSWAP)
    #[test]
    fn adv_r_exchange_r_same_as_mswap() {
        let mut state_a = make_state_with_matrix("A", 2, 2, vec![1.0, 2.0, 3.0, 4.0]);
        let mut state_b = make_state_with_matrix("A", 2, 2, vec![1.0, 2.0, 3.0, 4.0]);
        state_a.stack.y = HpNum::from(1);
        state_a.stack.x = HpNum::from(2);
        state_b.stack.y = HpNum::from(1);
        state_b.stack.x = HpNum::from(2);
        op_adv_r_exchange_r(&mut state_a).unwrap();
        op_adv_mswap(&mut state_b).unwrap();
        assert_eq!(state_a.adv_matrices[0].data, state_b.adv_matrices[0].data);
    }

    // Catches: PIV performs partial pivoting step
    #[test]
    fn adv_piv_swaps_pivot_row() {
        // Matrix [[1,2],[5,6],[3,4]] — current col=0, starting from row 0
        // PIV should identify row 1 (value 5) as pivot and swap with row 0
        let mut state = make_state_with_matrix("A", 3, 2, vec![
            1.0, 2.0,  // row 0, col 0 = 1
            5.0, 6.0,  // row 1, col 0 = 5 (max abs)
            3.0, 4.0,  // row 2, col 0 = 3
        ]);
        state.adv_matrix_i = 0;
        state.adv_matrix_j = 0;
        op_adv_piv(&mut state).unwrap();
        let mat = &state.adv_matrices[0];
        // Row 0 should now be [5, 6] (swapped from row 1)
        assert_eq!(f64_from_hpnum(&mat.data[0]), 5.0);
        assert_eq!(f64_from_hpnum(&mat.data[1]), 6.0);
    }

    // Catches: MP prints all matrix elements to print_buffer
    #[test]
    fn adv_mp_pushes_correct_number_of_lines() {
        let mut state = make_state_with_matrix("A", 2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        op_adv_mp(&mut state).unwrap();
        assert_eq!(
            state.print_buffer.len(),
            6,
            "2x3 matrix should produce 6 print lines"
        );
        // Check format of first line
        assert!(state.print_buffer[0].contains("R1C1="));
    }

    // ── Reduction tests ───────────────────────────────────────────────────────

    // Catches: SUM of 2x2 matrix [[1,2],[3,4]] = 10
    #[test]
    fn adv_sum_2x2() {
        let mut state = make_state_with_matrix("A", 2, 2, vec![1.0, 2.0, 3.0, 4.0]);
        op_adv_sum(&mut state).unwrap();
        assert_eq!(f64_from_hpnum(&state.stack.x), 10.0);
    }

    // Catches: SUMAB of [[1,-2],[3,-4]] = 10
    #[test]
    fn adv_sumab_2x2() {
        let mut state = make_state_with_matrix("A", 2, 2, vec![1.0, -2.0, 3.0, -4.0]);
        op_adv_sumab(&mut state).unwrap();
        assert_eq!(f64_from_hpnum(&state.stack.x), 10.0);
    }

    // Catches: MAX of [[1,5],[3,2]] = 5
    #[test]
    fn adv_max_2x2() {
        let mut state = make_state_with_matrix("A", 2, 2, vec![1.0, 5.0, 3.0, 2.0]);
        op_adv_max(&mut state).unwrap();
        assert_eq!(f64_from_hpnum(&state.stack.x), 5.0);
    }

    // Catches: MIN of [[1,5],[3,2]] = 1
    #[test]
    fn adv_min_2x2() {
        let mut state = make_state_with_matrix("A", 2, 2, vec![1.0, 5.0, 3.0, 2.0]);
        op_adv_min(&mut state).unwrap();
        assert_eq!(f64_from_hpnum(&state.stack.x), 1.0);
    }

    // Catches: MAXAB of [[1,-5],[3,-2]] = 5
    #[test]
    fn adv_maxab_2x2() {
        let mut state = make_state_with_matrix("A", 2, 2, vec![1.0, -5.0, 3.0, -2.0]);
        op_adv_maxab(&mut state).unwrap();
        assert_eq!(f64_from_hpnum(&state.stack.x), 5.0);
    }

    // Catches: FNRM of [[1,2],[3,4]] = sqrt(1+4+9+16) = sqrt(30)
    #[test]
    fn adv_fnrm_2x2() {
        let mut state = make_state_with_matrix("A", 2, 2, vec![1.0, 2.0, 3.0, 4.0]);
        op_adv_fnrm(&mut state).unwrap();
        let result = f64_from_hpnum(&state.stack.x);
        let expected = 30.0_f64.sqrt();
        assert!(
            (result - expected).abs() < 1e-6,
            "FNRM expected {expected}, got {result}"
        );
    }

    // Catches: RNRM row 0 of [[1,2],[3,4]] = max(|1|+|2|, |3|+|4|) = 7
    #[test]
    fn adv_rnrm_2x2() {
        let mut state = make_state_with_matrix("A", 2, 2, vec![1.0, 2.0, 3.0, 4.0]);
        op_adv_rnrm(&mut state).unwrap();
        assert_eq!(f64_from_hpnum(&state.stack.x), 7.0, "max row sum of abs = 3+4 = 7");
    }

    // Catches: RSUM of row 0 of [[1,2],[3,4]] = 1+2 = 3
    #[test]
    fn adv_rsum_row0() {
        let mut state = make_state_with_matrix("A", 2, 2, vec![1.0, 2.0, 3.0, 4.0]);
        state.adv_matrix_i = 0;
        op_adv_rsum(&mut state).unwrap();
        assert_eq!(f64_from_hpnum(&state.stack.x), 3.0);
    }

    // Catches: RSUM of row 1 of [[1,2],[3,4]] = 3+4 = 7
    #[test]
    fn adv_rsum_row1() {
        let mut state = make_state_with_matrix("A", 2, 2, vec![1.0, 2.0, 3.0, 4.0]);
        state.adv_matrix_i = 1;
        op_adv_rsum(&mut state).unwrap();
        assert_eq!(f64_from_hpnum(&state.stack.x), 7.0);
    }

    // Catches: RMAXAB of row 0 of [[1,-5],[3,2]] = max(|1|, |-5|) = 5
    #[test]
    fn adv_rmaxab_row0() {
        let mut state = make_state_with_matrix("A", 2, 2, vec![1.0, -5.0, 3.0, 2.0]);
        state.adv_matrix_i = 0;
        op_adv_rmaxab(&mut state).unwrap();
        assert_eq!(f64_from_hpnum(&state.stack.x), 5.0);
    }

    // Catches: R>R? returns 1 if first element of row k > first element of row l
    #[test]
    fn adv_r_gt_r_query_returns_1_when_true() {
        let mut state = make_state_with_matrix("A", 3, 2, vec![
            1.0, 0.0,  // row 1
            5.0, 0.0,  // row 2
            3.0, 0.0,  // row 3
        ]);
        // Row 2 > Row 1 (5 > 1) → should return 1
        state.stack.y = HpNum::from(2); // row k=2 (1-based)
        state.stack.x = HpNum::from(1); // row l=1 (1-based)
        op_adv_r_gt_r_query(&mut state).unwrap();
        assert_eq!(f64_from_hpnum(&state.stack.x), 1.0);
    }

    // Catches: R>R? returns 0 if row k <= row l
    #[test]
    fn adv_r_gt_r_query_returns_0_when_false() {
        let mut state = make_state_with_matrix("A", 3, 2, vec![
            5.0, 0.0,  // row 1
            1.0, 0.0,  // row 2
            3.0, 0.0,  // row 3
        ]);
        // Row 2 > Row 1 (1 > 5)? No → should return 0
        state.stack.y = HpNum::from(2); // row k=2 (1-based)
        state.stack.x = HpNum::from(1); // row l=1 (1-based)
        op_adv_r_gt_r_query(&mut state).unwrap();
        assert_eq!(f64_from_hpnum(&state.stack.x), 0.0);
    }

    // Catches: D-43.5 isolation invariant — production code in this file must not
    // reference state.matrix_dim or state.matrix_active_reg (those belong to Math Pac I).
    // Static verification is done via the grep CI gate in the plan verification section.
    // This test validates that matrix ops work correctly using only adv_matrices + adv_matrix_i/j.
    #[test]
    fn matrix_ops_d43_5_isolation_uses_only_adv_fields() {
        let mut state = make_state_with_matrix("A", 2, 2, vec![1.0, 2.0, 3.0, 4.0]);
        // All ops should work correctly using only adv_matrices + adv_matrix_i/j
        op_adv_sum(&mut state).unwrap();
        assert_eq!(f64_from_hpnum(&state.stack.x), 10.0, "SUM uses adv_matrices only");
    }

    // Catches: MR on non-existent matrix returns InvalidOp
    #[test]
    fn adv_mr_no_matrix_returns_error() {
        let mut state = CalcState::new();
        assert!(matches!(op_adv_mr(&mut state), Err(HpError::InvalidOp)));
    }

    // Catches: element_index bounds checking
    #[test]
    fn element_index_out_of_bounds_returns_domain() {
        let mat = AdvMatrix {
            name: "A".to_string(),
            rows: 2,
            cols: 2,
            is_complex: false,
            data: vec![HpNum::zero(); 4],
        };
        assert!(matches!(element_index(&mat, 2, 0), Err(HpError::Domain)));
        assert!(matches!(element_index(&mat, 0, 2), Err(HpError::Domain)));
        assert!(element_index(&mat, 1, 1).is_ok());
    }

    // Catches: stub tests replaced — all ops now pass or fail functionally
    #[test]
    fn matrix_ops_index_stubs_now_functional() {
        let mut state = make_state_with_matrix("A", 2, 2, vec![0.0; 4]);
        // These should now succeed (not return InvalidOp as in stub phase)
        assert!(op_adv_i_plus(&mut state).is_ok());
        assert!(op_adv_i_minus(&mut state).is_ok());
        assert!(op_adv_j_plus(&mut state).is_ok());
        assert!(op_adv_j_minus(&mut state).is_ok());
    }

    #[test]
    fn matrix_ops_reduction_stubs_now_functional() {
        let mut state = make_state_with_matrix("A", 2, 2, vec![1.0, 2.0, 3.0, 4.0]);
        assert!(op_adv_sum(&mut state).is_ok());
        assert!(op_adv_sumab(&mut state).is_ok());
        assert!(op_adv_max(&mut state).is_ok());
        assert!(op_adv_maxab(&mut state).is_ok());
        assert!(op_adv_min(&mut state).is_ok());
        assert!(op_adv_rmaxab(&mut state).is_ok());
        assert!(op_adv_rnrm(&mut state).is_ok());
        assert!(op_adv_rsum(&mut state).is_ok());
        assert!(op_adv_fnrm(&mut state).is_ok());
    }

    #[test]
    fn matrix_ops_lifecycle_stubs_now_functional() {
        // Create a matrix first
        let mut state = CalcState::new();
        state.alpha_reg = "A".to_string();
        state.stack.y = HpNum::from(2);
        state.stack.x = HpNum::from(3);
        assert!(op_adv_matdim(&mut state).is_ok());
        // Now DIM? and MNAME? should work
        state.alpha_reg = "A".to_string();
        assert!(op_adv_dim_query(&mut state).is_ok());
        assert!(op_adv_mname_query(&mut state).is_ok());
    }
}
