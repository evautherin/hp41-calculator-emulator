// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `advantage` — HP Advantage Pac operations.
//!
//! XROM module ids: 22 (ADV CONV + ADV MTRX, bit 3 of `CalcState::xrom_modules`)
//! and 24 (ADV MATH + ADV TVM, bit 4 of `CalcState::xrom_modules`).
//! Activated in Phase 43 (v3.3).
//!
//! ## Named-Matrix Model (D-43.1 / ADV-FW-04)
//!
//! The Advantage Pac uses an ALPHA-register named-matrix model, completely
//! separate from the Math Pac I register-based matrix (R14/R15+).
//!
//! D-43.5 ISOLATION INVARIANT: No code in this module may read or write
//! `state.matrix_dim` or `state.matrix_active_reg`. Those fields belong
//! exclusively to the Math Pac I matrix (Plan 28-06).
//!
//! ## Submodule structure (per RESEARCH.md recommended project structure)
//!
//! - `modal`          — AdvantageStep enum + current_prompt/requires_alpha_label/submit_step
//! - `conv`           — ADV CONV: BININ/BINVIEW/OCTIN/HEXIN/HEXVIEW/CVTVIEW/NOT/AND/OR/XOR/ROTXY/BIT?
//! - `matrix_ops`     — ADV MTRX element access (I+/I-/J+/J-/MR/MS/..) + lifecycle + reductions
//! - `matrix_linalg`  — ADV MTRX high-level: MDET/MINV/MSYS/M*M/MAT+/MAT-.../TRNPS/MMOVE
//! - `matrix_complex` — ADV MTRX complex: C<>C/CMAXAB/CNRM/CSUM/YC+C
//! - `complex_ext`    — ADV MATH complex extensions: e^Z/LNZ/Z^N/.../ADV C+/C-/CINV/C*/C/
//! - `solvers`        — ADV MATH: FSOLVE/FINTG/FDIFEQ/FROOT (solver states + run_loop arms)
//! - `poly`           — ADV MATH: PLY (polynomial eval) + RTS (root output)
//! - `matrix_workflow`— ADV MATH: MATRX/MTR modal frontends + AIP
//! - `curve_fit`      — ADV MATH: CFIT/AS/DS/BFIT/FIT/Y?X/SZ?
//! - `vectors`        — ADV MATH: V+/V-/DOT/CROSS/VC/VS/VR/VE/VXY/UV/V</V*/VD/TR
//! - `tvm`            — ADV TVM: TVM/N/PV/PMT/FV/*I + TvmState struct
//!
//! ## References
//!
//! - HP Advantage Pac Owner's Manual 00041-90482 (HP Portable Computer Division, 1985)

use crate::num::HpNum;
use serde::{Deserialize, Serialize};

pub mod complex_ext;
pub mod conv;
pub mod curve_fit;
pub mod matrix_complex;
pub mod matrix_linalg;
pub mod matrix_ops;
pub mod matrix_workflow;
pub mod modal;
pub mod poly;
pub mod solvers;
pub mod tvm;
pub mod vectors;

pub use modal::AdvantageStep;
pub use solvers::{AdvFdifeqState, AdvFintegState, AdvFsolveState, FrootState};
pub use tvm::TvmState;

// Re-export all op functions for use in ops/mod.rs dispatch() and ops/program.rs execute_op()
pub use complex_ext::{
    op_adv_a_pow_z, op_adv_aip, op_adv_c_div, op_adv_c_minus, op_adv_c_mul, op_adv_c_plus,
    op_adv_cinv, op_adv_cos_z, op_adv_exp_z, op_adv_ln_z, op_adv_log_z, op_adv_magz, op_adv_sin_z,
    op_adv_tan_z, op_adv_z_pow_1n, op_adv_z_pow_1w, op_adv_z_pow_n, op_adv_z_pow_w,
};
pub use conv::{
    op_adv_and, op_adv_binin, op_adv_binview, op_adv_bit_test, op_adv_cvtview, op_adv_hexin,
    op_adv_hexview, op_adv_not, op_adv_octin, op_adv_or, op_adv_rotxy, op_adv_xor,
};
pub use curve_fit::{
    op_adv_as, op_adv_bfit, op_adv_cfit, op_adv_ds, op_adv_fit, op_adv_sz_query, op_adv_y_query_x,
};
pub use matrix_complex::{
    op_adv_c_exchange_c, op_adv_cmaxab, op_adv_cnrm, op_adv_csum, op_adv_yc_plus_c,
};
pub use matrix_linalg::{
    op_adv_m_mul_m, op_adv_mat_minus, op_adv_mat_plus, op_adv_mat_scalar_div,
    op_adv_mat_scalar_mul, op_adv_mdet, op_adv_minv, op_adv_mmove, op_adv_msys, op_adv_trnps,
};
pub use matrix_ops::{
    op_adv_dim_query, op_adv_fnrm, op_adv_i_minus, op_adv_i_plus, op_adv_j_minus, op_adv_j_plus,
    op_adv_matdim, op_adv_max, op_adv_maxab, op_adv_min, op_adv_mname_query, op_adv_mp, op_adv_mr,
    op_adv_mrc_minus, op_adv_mrc_plus, op_adv_mrij, op_adv_mrr_minus, op_adv_mrr_plus, op_adv_ms,
    op_adv_msc_plus, op_adv_msij, op_adv_msijr, op_adv_msr_plus, op_adv_mswap, op_adv_piv,
    op_adv_r_exchange_r, op_adv_r_gt_r_query, op_adv_rmaxab, op_adv_rnrm, op_adv_rsum, op_adv_sum,
    op_adv_sumab,
};
pub use matrix_workflow::{op_adv_cmedit, op_adv_matrx, op_adv_medit, op_adv_mtr};
pub use poly::{op_adv_ply, op_adv_rts};
pub use solvers::{
    op_adv_fdifeq, op_adv_fdifeq_run_loop, op_adv_fintg, op_adv_fintg_run_loop, op_adv_froot,
    op_adv_fsolve, op_adv_fsolve_run_loop,
};
pub use tvm::{
    op_adv_tvm, op_adv_tvm_fv, op_adv_tvm_n, op_adv_tvm_pmt, op_adv_tvm_pv, op_adv_tvm_star_i,
};
pub use vectors::{
    op_adv_cross, op_adv_dot, op_adv_tr, op_adv_uv, op_adv_v_mag, op_adv_v_minus, op_adv_v_plus,
    op_adv_v_star, op_adv_vc, op_adv_vd, op_adv_ve, op_adv_vr, op_adv_vs, op_adv_vxy,
};

/// Named matrix entry for Advantage Pac X-MEM model (ADV-FW-04 / D-43.1).
///
/// Identified by ALPHA register name; rows and cols capped at
/// `ADV_MATRIX_MAX_ROWS` / `ADV_MATRIX_MAX_COLS`.
/// Data stored row-major. Complex matrices interleave real/imag pairs:
/// element (i,j) real  = data[2*(i * cols as usize + j)]
/// element (i,j) imag  = data[2*(i * cols as usize + j) + 1]
///
/// D-43.5 ISOLATION INVARIANT: This struct has no connection to
/// `state.matrix_dim` or `state.matrix_active_reg`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdvMatrix {
    /// ALPHA-register name identifying this matrix.
    pub name: String,
    /// Number of rows (1..=ADV_MATRIX_MAX_ROWS).
    pub rows: u8,
    /// Number of columns (1..=ADV_MATRIX_MAX_COLS).
    pub cols: u8,
    /// Whether this is a complex matrix (interleaved real/imag data).
    pub is_complex: bool,
    /// Row-major element data; complex: len = 2 * rows * cols; real: len = rows * cols.
    pub data: Vec<HpNum>,
}

/// Maximum rows per named matrix (D-43.2 emulator extension — hardware was ~8).
/// Fits in a `u8` index. Document as emulator extension in hp41-advantage-divergences.md.
pub const ADV_MATRIX_MAX_ROWS: u8 = 255;

/// Maximum columns per named matrix (D-43.2 emulator extension).
pub const ADV_MATRIX_MAX_COLS: u8 = 255;

/// 36-bit fixed word size mask for ADV CONV operations (D-43.9 / D-43.10).
///
/// The HP-41's 10-digit BCD mantissa can hold integers up to ~9.999999999e9,
/// which corresponds to a 33-bit range. The Advantage Pac OM 00041-90482
/// Section 1 specifies 36-bit fixed word size for bitwise operations.
/// Verify: `ADV_WORD_MASK.count_ones() == 36` (see tests).
pub const ADV_WORD_MASK: u64 = 0x0000_000F_FFFF_FFFF;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Catches: ADV_WORD_MASK bit-count regression (Pitfall 7 from RESEARCH.md)
    #[test]
    fn adv_word_mask_is_36_bits() {
        assert_eq!(
            ADV_WORD_MASK.count_ones(),
            36,
            "ADV_WORD_MASK must have exactly 36 set bits (D-43.9)"
        );
    }

    // Catches: ADV_WORD_MASK value regression
    #[test]
    fn adv_word_mask_value() {
        assert_eq!(
            ADV_WORD_MASK, 68_719_476_735u64,
            "ADV_WORD_MASK == 2^36 - 1 == 68_719_476_735"
        );
    }

    // Catches: AdvMatrix default is empty
    #[test]
    fn adv_matrix_default_is_empty() {
        let m = AdvMatrix::default();
        assert_eq!(m.rows, 0);
        assert_eq!(m.cols, 0);
        assert!(m.data.is_empty());
        assert!(m.name.is_empty());
        assert!(!m.is_complex);
    }
}
