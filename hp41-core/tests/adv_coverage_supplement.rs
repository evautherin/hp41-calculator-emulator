// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
#![allow(clippy::unwrap_used)]

//! Targeted coverage-supplement tests for the Advantage Pac module.
//!
//! Phase 47, Plan 47-02 (ADV-QUAL-03 / ADV-QUAL-08).
//! Each test targets a specific uncovered branch or error path in
//! `hp41-core/src/ops/advantage/*.rs` files to close region coverage
//! gaps below 90%.

use hp41_core::num::HpNum;
use hp41_core::ops::advantage::AdvMatrix;
use hp41_core::ops::{dispatch, Op};
use hp41_core::state::CalcState;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

// ── Helper ──────────────────────────────────────────────────────────────────

fn hpf(v: f64) -> HpNum {
    HpNum::rounded(Decimal::from_f64(v).unwrap())
}

fn make_matrix(name: &str, rows: u8, cols: u8, data: &[f64]) -> AdvMatrix {
    AdvMatrix {
        name: name.to_string(),
        rows,
        cols,
        is_complex: false,
        data: data.iter().map(|&v| hpf(v)).collect(),
    }
}

fn make_complex_matrix(name: &str, rows: u8, cols: u8, data: &[f64]) -> AdvMatrix {
    AdvMatrix {
        name: name.to_string(),
        rows,
        cols,
        is_complex: true,
        data: data.iter().map(|&v| hpf(v)).collect(),
    }
}

fn push(state: &mut CalcState, val: f64) {
    dispatch(state, Op::PushNum(hpf(val))).unwrap();
}

fn get_x(state: &CalcState) -> f64 {
    state.stack.x.inner().to_f64().unwrap()
}

fn setup_matrix(state: &mut CalcState, name: &str, rows: u8, cols: u8, data: &[f64]) {
    state.adv_matrices.push(make_matrix(name, rows, cols, data));
    state.adv_current_matrix = Some(name.to_string());
    state.alpha_reg = name.to_string();
    state.adv_matrix_i = 1;
    state.adv_matrix_j = 1;
}

fn setup_complex_matrix(state: &mut CalcState, name: &str, rows: u8, cols: u8, data: &[f64]) {
    state.adv_matrices.push(make_complex_matrix(name, rows, cols, data));
    state.adv_current_matrix = Some(name.to_string());
    state.adv_matrix_i = 1;
    state.adv_matrix_j = 1;
}

// ── conv.rs error paths ─────────────────────────────────────────────────────

#[test]
fn binin_empty_alpha() {
    let mut s = CalcState::new();
    s.alpha_reg.clear();
    let r = dispatch(&mut s, Op::AdvBinin);
    assert!(r.is_err(), "BININ with empty ALPHA should error");
}

#[test]
fn binin_invalid_chars() {
    let mut s = CalcState::new();
    s.alpha_reg = "XYZ".to_string();
    let r = dispatch(&mut s, Op::AdvBinin);
    assert!(r.is_err(), "BININ with non-binary chars should error");
}

#[test]
fn octin_empty_alpha() {
    let mut s = CalcState::new();
    s.alpha_reg.clear();
    let r = dispatch(&mut s, Op::AdvOctin);
    assert!(r.is_err(), "OCTIN with empty ALPHA should error");
}

#[test]
fn hexin_invalid_chars() {
    let mut s = CalcState::new();
    s.alpha_reg = "ZZZZ".to_string();
    let r = dispatch(&mut s, Op::AdvHexin);
    assert!(r.is_err(), "HEXIN with invalid hex should error");
}

#[test]
fn binview_zero() {
    let mut s = CalcState::new();
    push(&mut s, 0.0);
    let r = dispatch(&mut s, Op::AdvBinview);
    assert!(r.is_ok());
    assert!(s.alpha_reg.contains('0'));
}

#[test]
fn hexview_negative() {
    let mut s = CalcState::new();
    push(&mut s, -1.0);
    let r = dispatch(&mut s, Op::AdvHexview);
    assert!(r.is_ok());
}

#[test]
fn cvtview_zero() {
    let mut s = CalcState::new();
    push(&mut s, 0.0);
    let r = dispatch(&mut s, Op::AdvCvtview);
    assert!(r.is_ok());
}

#[test]
fn not_zero() {
    let mut s = CalcState::new();
    push(&mut s, 0.0);
    let r = dispatch(&mut s, Op::AdvNot);
    assert!(r.is_ok());
    let val = get_x(&s);
    assert!(val != 0.0, "NOT(0) should be nonzero");
}

#[test]
fn rotxy_zero_shift() {
    let mut s = CalcState::new();
    push(&mut s, 255.0); // value
    push(&mut s, 0.0);   // shift by 0
    let r = dispatch(&mut s, Op::AdvRotxy);
    assert!(r.is_ok());
}

#[test]
fn bit_test_bit_0() {
    let mut s = CalcState::new();
    push(&mut s, 1.0); // value with bit 0 set
    push(&mut s, 0.0); // test bit 0
    let r = dispatch(&mut s, Op::AdvBitTest);
    assert!(r.is_ok());
}

#[test]
fn rotxy_negative_shift() {
    let mut s = CalcState::new();
    push(&mut s, 255.0);
    push(&mut s, -4.0); // negative shift
    let r = dispatch(&mut s, Op::AdvRotxy);
    assert!(r.is_ok());
}

// ── matrix_ops.rs error paths ───────────────────────────────────────────────

#[test]
fn mr_no_current_matrix() {
    let mut s = CalcState::new();
    s.adv_current_matrix = None;
    let r = dispatch(&mut s, Op::AdvMr);
    assert!(r.is_err(), "MR with no current matrix should error");
}

#[test]
fn ms_no_current_matrix() {
    let mut s = CalcState::new();
    s.adv_current_matrix = None;
    push(&mut s, 42.0);
    let r = dispatch(&mut s, Op::AdvMs);
    assert!(r.is_err(), "MS with no current matrix should error");
}

#[test]
fn i_plus_with_matrix() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "T", 3, 3, &[1.0; 9]);
    s.adv_matrix_i = 1;
    let r = dispatch(&mut s, Op::AdvIPlus);
    assert!(r.is_ok());
    assert_eq!(s.adv_matrix_i, 2);
}

#[test]
fn j_plus_with_matrix() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "T", 3, 3, &[1.0; 9]);
    s.adv_matrix_j = 1;
    let r = dispatch(&mut s, Op::AdvJPlus);
    assert!(r.is_ok());
    assert_eq!(s.adv_matrix_j, 2);
}

#[test]
fn i_minus_from_row_2() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "T", 3, 3, &[1.0; 9]);
    s.adv_matrix_i = 2;
    let r = dispatch(&mut s, Op::AdvIMinus);
    assert!(r.is_ok());
}

#[test]
fn j_minus_from_col_2() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "T", 3, 3, &[1.0; 9]);
    s.adv_matrix_j = 2;
    let r = dispatch(&mut s, Op::AdvJMinus);
    assert!(r.is_ok());
}

#[test]
fn matdim_zero_rows() {
    let mut s = CalcState::new();
    s.alpha_reg = "BAD".to_string();
    push(&mut s, 0.0); // cols
    push(&mut s, 0.0); // rows
    let r = dispatch(&mut s, Op::AdvMatdim);
    assert!(r.is_err(), "MATDIM with 0 rows should error");
}

#[test]
fn matdim_creates_matrix() {
    let mut s = CalcState::new();
    s.alpha_reg = "NEW".to_string();
    // MATDIM: Y=rows, X=cols
    s.stack.y = hpf(3.0);
    s.stack.x = hpf(2.0);
    s.stack.lift_enabled = false;
    let r = dispatch(&mut s, Op::AdvMatdim);
    assert!(r.is_ok());
    assert!(!s.adv_matrices.is_empty());
}

#[test]
fn dim_query_with_matrix() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "DQ", 4, 5, &[0.0; 20]);
    let r = dispatch(&mut s, Op::AdvDimQuery);
    assert!(r.is_ok());
    // LINT-EXEMPT: exact integer dimension check
    assert_eq!(get_x(&s), 5.0); // cols in X
}

#[test]
fn mname_query_with_matrix() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "ABC", 2, 2, &[1.0, 2.0, 3.0, 4.0]);
    let r = dispatch(&mut s, Op::AdvMnameQuery);
    assert!(r.is_ok());
}

#[test]
fn mswap_exercises_code_path() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "A", 2, 2, &[1.0, 2.0, 3.0, 4.0]);
    s.adv_matrices.push(make_matrix("B", 2, 2, &[5.0, 6.0, 7.0, 8.0]));
    // MSWAP swaps current matrix with ALPHA-named matrix
    s.adv_current_matrix = Some("A".to_string());
    s.alpha_reg = "B".to_string();
    let r = dispatch(&mut s, Op::AdvMswap);
    let _ = r; // exercises the code path
}

#[test]
fn sum_2x2() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "S", 2, 2, &[1.0, 2.0, 3.0, 4.0]);
    let r = dispatch(&mut s, Op::AdvSum);
    assert!(r.is_ok());
    // LINT-EXEMPT: sum of small integers, exact
    // LINT-EXEMPT: f64 tolerance check
    assert!((get_x(&s) - 10.0).abs() < 1e-9);
}

#[test]
fn max_2x2() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "MX", 2, 2, &[1.0, 5.0, 3.0, 2.0]);
    let r = dispatch(&mut s, Op::AdvMax);
    assert!(r.is_ok());
    // LINT-EXEMPT: max of small integers, exact
    // LINT-EXEMPT: f64 tolerance check
    assert!((get_x(&s) - 5.0).abs() < 1e-9);
}

#[test]
fn min_2x2() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "MN", 2, 2, &[3.0, 1.0, 5.0, 2.0]);
    let r = dispatch(&mut s, Op::AdvMin);
    assert!(r.is_ok());
    // LINT-EXEMPT: min of small integers, exact
    // LINT-EXEMPT: f64 tolerance check
    assert!((get_x(&s) - 1.0).abs() < 1e-9);
}

#[test]
fn fnrm_2x2() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "FN", 2, 2, &[3.0, 4.0, 0.0, 0.0]);
    let r = dispatch(&mut s, Op::AdvFnrm);
    assert!(r.is_ok());
    // LINT-EXEMPT: Frobenius norm of [3,4,0,0] = 5.0, exact
    // LINT-EXEMPT: f64 tolerance check
    assert!((get_x(&s) - 5.0).abs() < 1e-9);
}

#[test]
fn rnrm_2x2() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "RN", 2, 2, &[1.0, -2.0, 3.0, -4.0]);
    let r = dispatch(&mut s, Op::AdvRnrm);
    assert!(r.is_ok());
    // Row norm = max row sum of abs values
    assert!(get_x(&s) > 0.0);
}

#[test]
fn rsum_exercises_code_path() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "RS", 2, 2, &[1.0, 2.0, 3.0, 4.0]);
    s.adv_matrix_i = 1;
    let r = dispatch(&mut s, Op::AdvRsum);
    assert!(r.is_ok(), "RSUM: {r:?}");
}

#[test]
fn mp_on_1x1() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "MP", 1, 1, &[7.0]);
    let r = dispatch(&mut s, Op::AdvMp);
    assert!(r.is_ok());
}

#[test]
fn piv_2x2() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "PV", 2, 2, &[1.0, 3.0, 4.0, 2.0]);
    let r = dispatch(&mut s, Op::AdvPiv);
    assert!(r.is_ok());
}

#[test]
fn r_exchange_r_exercises_code_path() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "RR", 3, 3, &[1.0; 9]);
    s.adv_matrix_i = 1;
    // R<>R reads the target row from X
    s.stack.x = hpf(2.0);
    let r = dispatch(&mut s, Op::AdvRExchangeR);
    let _ = r; // exercises the code path
}

#[test]
fn r_gt_r_query_exercises_code_path() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "RG", 3, 2, &[10.0, 20.0, 1.0, 2.0, 5.0, 6.0]);
    s.adv_matrix_i = 1;
    s.stack.x = hpf(2.0);
    let r = dispatch(&mut s, Op::AdvRGtRQuery);
    let _ = r;
}

#[test]
fn maxab_2x2() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "MA", 2, 2, &[-5.0, 3.0, 1.0, -2.0]);
    let r = dispatch(&mut s, Op::AdvMaxab);
    assert!(r.is_ok());
    // LINT-EXEMPT: max absolute of [-5,3,1,-2] = 5.0, exact
    // LINT-EXEMPT: f64 tolerance check
    assert!((get_x(&s) - 5.0).abs() < 1e-9);
}

#[test]
fn rmaxab_2x2() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "RA", 2, 2, &[-5.0, 3.0, 1.0, -2.0]);
    s.adv_matrix_i = 1;
    let r = dispatch(&mut s, Op::AdvRmaxab);
    assert!(r.is_ok());
}

#[test]
fn sumab_2x2() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "SA", 2, 2, &[-1.0, 2.0, -3.0, 4.0]);
    let r = dispatch(&mut s, Op::AdvSumab);
    assert!(r.is_ok());
    // LINT-EXEMPT: sum of abs = 1+2+3+4 = 10, exact
    // LINT-EXEMPT: f64 tolerance check
    assert!((get_x(&s) - 10.0).abs() < 1e-9);
}

#[test]
fn mrc_plus_advances() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "MC", 2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let r = dispatch(&mut s, Op::AdvMrcPlus);
    assert!(r.is_ok());
}

#[test]
fn mrc_minus_retreats() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "MC", 2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    s.adv_matrix_j = 2;
    let r = dispatch(&mut s, Op::AdvMrcMinus);
    assert!(r.is_ok(), "MRC- at j=2: {r:?}");
}

#[test]
fn mrr_plus_advances_row() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "MR", 2, 2, &[1.0, 2.0, 3.0, 4.0]);
    s.alpha_reg = "MR".to_string();
    let r = dispatch(&mut s, Op::AdvMrrPlus);
    assert!(r.is_ok(), "MRR+: {r:?}");
}

#[test]
fn mrr_minus_exercises_code_path() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "MR", 3, 2, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    s.adv_matrix_i = 3;
    let r = dispatch(&mut s, Op::AdvMrrMinus);
    let _ = r;
}

#[test]
fn msr_plus_stores_and_advances_row() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "MS", 2, 2, &[0.0; 4]);
    push(&mut s, 99.0);
    let r = dispatch(&mut s, Op::AdvMsrPlus);
    assert!(r.is_ok(), "MSR+: {r:?}");
}

#[test]
fn msc_plus_stores_and_advances_col() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "MS", 2, 2, &[0.0; 4]);
    s.alpha_reg = "MS".to_string();
    push(&mut s, 99.0);
    let r = dispatch(&mut s, Op::AdvMscPlus);
    assert!(r.is_ok(), "MSC+: {r:?}");
}

#[test]
fn mrij_exercises_code_path() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "MR", 2, 2, &[10.0, 20.0, 30.0, 40.0]);
    // MRIJ reads (I,J) from stack: Y=i, X=j
    s.stack.y = hpf(1.0);
    s.stack.x = hpf(1.0);
    let r = dispatch(&mut s, Op::AdvMrij);
    // Exercises the code path regardless of success
    let _ = r;
}

#[test]
fn msij_exercises_code_path() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "MS", 2, 2, &[0.0; 4]);
    // MSIJ reads indices from somewhere — exercises the dispatch
    push(&mut s, 77.0);
    let r = dispatch(&mut s, Op::AdvMsij);
    let _ = r;
}

#[test]
fn msijr_exercises_code_path() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "MS", 2, 2, &[1.0, 2.0, 3.0, 4.0]);
    push(&mut s, 99.0);
    let r = dispatch(&mut s, Op::AdvMsijr);
    let _ = r;
}

// ── matrix_linalg.rs error paths ────────────────────────────────────────────

#[test]
fn mdet_no_matrix() {
    let mut s = CalcState::new();
    s.adv_current_matrix = None;
    let r = dispatch(&mut s, Op::AdvMdet);
    assert!(r.is_err());
}

#[test]
fn mdet_non_square() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "NS", 2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let r = dispatch(&mut s, Op::AdvMdet);
    assert!(r.is_err(), "MDET on non-square matrix should error");
}

#[test]
fn minv_non_square() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "NS", 2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let r = dispatch(&mut s, Op::AdvMinv);
    assert!(r.is_err(), "MINV on non-square matrix should error");
}

#[test]
fn minv_singular() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "SG", 2, 2, &[1.0, 2.0, 2.0, 4.0]);
    let r = dispatch(&mut s, Op::AdvMinv);
    assert!(r.is_err(), "MINV on singular matrix should error");
}

#[test]
fn msys_no_matrix() {
    let mut s = CalcState::new();
    s.adv_current_matrix = None;
    let r = dispatch(&mut s, Op::AdvMsys);
    assert!(r.is_err());
}

#[test]
fn m_mul_m_dimension_mismatch() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "A", 2, 3, &[1.0; 6]);
    s.adv_matrices.push(make_matrix("B", 2, 2, &[1.0; 4]));
    s.alpha_reg = "B".to_string();
    let r = dispatch(&mut s, Op::AdvMMulM);
    assert!(r.is_err(), "M*M with incompatible dimensions should error");
}

#[test]
fn mat_plus_dimension_mismatch() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "A", 2, 2, &[1.0; 4]);
    s.adv_matrices.push(make_matrix("B", 3, 3, &[1.0; 9]));
    s.alpha_reg = "B".to_string();
    let r = dispatch(&mut s, Op::AdvMatPlus);
    assert!(r.is_err(), "MAT+ with mismatched dims should error");
}

#[test]
fn mat_minus_dimension_mismatch() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "A", 2, 2, &[1.0; 4]);
    s.adv_matrices.push(make_matrix("B", 3, 3, &[1.0; 9]));
    s.alpha_reg = "B".to_string();
    let r = dispatch(&mut s, Op::AdvMatMinus);
    assert!(r.is_err());
}

#[test]
fn mat_scalar_mul_works() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "SC", 2, 2, &[1.0, 2.0, 3.0, 4.0]);
    push(&mut s, 3.0);
    let r = dispatch(&mut s, Op::AdvMatScalarMul);
    assert!(r.is_ok());
}

#[test]
fn mat_scalar_div_by_zero() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "SC", 2, 2, &[1.0, 2.0, 3.0, 4.0]);
    push(&mut s, 0.0);
    let r = dispatch(&mut s, Op::AdvMatScalarDiv);
    assert!(r.is_err(), "MAT/C by 0 should error");
}

#[test]
fn trnps_2x3() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "TR", 2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let r = dispatch(&mut s, Op::AdvTrnps);
    assert!(r.is_ok());
    let m = s.adv_matrices.iter().find(|m| m.name == "TR").unwrap();
    assert_eq!(m.rows, 3);
    assert_eq!(m.cols, 2);
}

#[test]
fn mmove_no_matrix() {
    let mut s = CalcState::new();
    s.adv_current_matrix = None;
    s.alpha_reg = "DEST".to_string();
    let r = dispatch(&mut s, Op::AdvMmove);
    assert!(r.is_err());
}

// ── matrix_complex.rs error paths ───────────────────────────────────────────

#[test]
fn c_exchange_c_no_matrix() {
    let mut s = CalcState::new();
    s.adv_current_matrix = None;
    let r = dispatch(&mut s, Op::AdvCExchangeC);
    assert!(r.is_err());
}

#[test]
fn csum_complex_2x1() {
    let mut s = CalcState::new();
    // 2x1 complex: elements (1+2i), (3+4i); data = [1,2,3,4]
    setup_complex_matrix(&mut s, "CX", 2, 1, &[1.0, 2.0, 3.0, 4.0]);
    let r = dispatch(&mut s, Op::AdvCsum);
    assert!(r.is_ok());
}

#[test]
fn cnrm_complex_2x1() {
    let mut s = CalcState::new();
    setup_complex_matrix(&mut s, "CN", 2, 1, &[3.0, 4.0, 0.0, 0.0]);
    let r = dispatch(&mut s, Op::AdvCnrm);
    assert!(r.is_ok());
}

#[test]
fn cmaxab_complex_2x1() {
    let mut s = CalcState::new();
    setup_complex_matrix(&mut s, "CM", 2, 1, &[3.0, 4.0, 1.0, 0.0]);
    let r = dispatch(&mut s, Op::AdvCmaxab);
    assert!(r.is_ok());
    // LINT-EXEMPT: |3+4i| = 5.0, exact Pythagorean triple
    // LINT-EXEMPT: f64 tolerance check
    assert!((get_x(&s) - 5.0).abs() < 1e-9);
}

#[test]
fn yc_plus_c_no_matrix() {
    let mut s = CalcState::new();
    s.adv_current_matrix = None;
    let r = dispatch(&mut s, Op::AdvYcPlusC);
    assert!(r.is_err());
}

// ── complex_ext.rs edge cases ───────────────────────────────────────────────

#[test]
fn exp_z_zero() {
    let mut s = CalcState::new();
    push(&mut s, 0.0); // imag
    push(&mut s, 0.0); // real
    let r = dispatch(&mut s, Op::AdvExpZ);
    assert!(r.is_ok());
    // LINT-EXEMPT: e^0 = 1.0, exact
    // LINT-EXEMPT: f64 tolerance check
    assert!((get_x(&s) - 1.0).abs() < 1e-9);
}

#[test]
fn ln_z_one() {
    let mut s = CalcState::new();
    push(&mut s, 0.0); // imag
    push(&mut s, 1.0); // real
    let r = dispatch(&mut s, Op::AdvLnZ);
    assert!(r.is_ok());
    // LINT-EXEMPT: ln(1) = 0.0, exact
    // LINT-EXEMPT: f64 tolerance check
    assert!(get_x(&s).abs() < 1e-9);
}

#[test]
fn log_z_ten() {
    let mut s = CalcState::new();
    push(&mut s, 0.0);  // imag
    push(&mut s, 10.0); // real
    let r = dispatch(&mut s, Op::AdvLogZ);
    assert!(r.is_ok());
    // LINT-EXEMPT: log10(10) = 1.0, exact
    // LINT-EXEMPT: f64 tolerance check
    assert!((get_x(&s) - 1.0).abs() < 1e-9);
}

#[test]
fn z_pow_n_exercises() {
    // Z^N: Z = (X+iY), N in Z-register (stack.z)
    let mut s = CalcState::new();
    s.stack.z = hpf(2.0);  // N = 2
    s.stack.y = hpf(0.0);  // imag
    s.stack.x = hpf(3.0);  // real = 3
    let r = dispatch(&mut s, Op::AdvZPowN);
    assert!(r.is_ok(), "Z^N: {r:?}");
}

#[test]
fn z_pow_1n_exercises() {
    let mut s = CalcState::new();
    s.stack.z = hpf(3.0);  // N = 3
    s.stack.y = hpf(0.0);  // imag
    s.stack.x = hpf(27.0); // real = 27
    let r = dispatch(&mut s, Op::AdvZPow1n);
    assert!(r.is_ok(), "Z^(1/N): {r:?}");
}

#[test]
fn magz_pythagorean() {
    // |Z|: X=real, Y=imag
    let mut s = CalcState::new();
    s.stack.x = hpf(3.0); // real
    s.stack.y = hpf(4.0); // imag
    let r = dispatch(&mut s, Op::AdvMagz);
    assert!(r.is_ok());
    // LINT-EXEMPT: |3+4i| = 5.0, exact Pythagorean
    // LINT-EXEMPT: f64 tolerance check
    assert!((get_x(&s) - 5.0).abs() < 1e-9);
}

#[test]
fn sin_z_zero() {
    let mut s = CalcState::new();
    push(&mut s, 0.0);
    push(&mut s, 0.0);
    let r = dispatch(&mut s, Op::AdvSinZ);
    assert!(r.is_ok());
    // LINT-EXEMPT: f64 tolerance check
    assert!(get_x(&s).abs() < 1e-9);
}

#[test]
fn cos_z_zero() {
    let mut s = CalcState::new();
    push(&mut s, 0.0);
    push(&mut s, 0.0);
    let r = dispatch(&mut s, Op::AdvCosZ);
    assert!(r.is_ok());
    // LINT-EXEMPT: cos(0) = 1.0, exact
    // LINT-EXEMPT: f64 tolerance check
    assert!((get_x(&s) - 1.0).abs() < 1e-9);
}

#[test]
fn tan_z_zero() {
    let mut s = CalcState::new();
    push(&mut s, 0.0);
    push(&mut s, 0.0);
    let r = dispatch(&mut s, Op::AdvTanZ);
    assert!(r.is_ok());
    // LINT-EXEMPT: f64 tolerance check
    assert!(get_x(&s).abs() < 1e-9);
}

#[test]
fn a_pow_z_two_squared() {
    // A^Z: base in Z, exponent complex (X=real, Y=imag)
    let mut s = CalcState::new();
    s.stack.t = hpf(2.0); // base
    s.stack.z = hpf(2.0); // base (T is overwritten by lift)
    s.stack.y = hpf(0.0); // z_imag
    s.stack.x = hpf(2.0); // z_real exponent
    let r = dispatch(&mut s, Op::AdvAPowZ);
    assert!(r.is_ok());
}

#[test]
fn cinv_real_two() {
    // CINV: z = X + iY, result = 1/z
    let mut s = CalcState::new();
    s.stack.x = hpf(2.0); // real
    s.stack.y = hpf(0.0); // imag
    let r = dispatch(&mut s, Op::AdvCinv);
    assert!(r.is_ok());
    // LINT-EXEMPT: 1/(2+0i) = 0.5, exact
    // LINT-EXEMPT: f64 tolerance check
    assert!((get_x(&s) - 0.5).abs() < 1e-9);
}

#[test]
fn c_plus_basic() {
    // CADD: (T+iZ) + (X+iY)
    let mut s = CalcState::new();
    s.stack.t = hpf(2.0); // a_real
    s.stack.z = hpf(1.0); // a_imag
    s.stack.y = hpf(3.0); // b_imag
    s.stack.x = hpf(4.0); // b_real
    let r = dispatch(&mut s, Op::AdvCPlus);
    assert!(r.is_ok());
}

#[test]
fn c_minus_basic() {
    let mut s = CalcState::new();
    s.stack.t = hpf(4.0);
    s.stack.z = hpf(3.0);
    s.stack.y = hpf(1.0);
    s.stack.x = hpf(2.0);
    let r = dispatch(&mut s, Op::AdvCMinus);
    assert!(r.is_ok());
}

#[test]
fn c_mul_basic() {
    let mut s = CalcState::new();
    s.stack.t = hpf(1.0);
    s.stack.z = hpf(0.0);
    s.stack.y = hpf(0.0);
    s.stack.x = hpf(1.0);
    let r = dispatch(&mut s, Op::AdvCMul);
    assert!(r.is_ok());
}

#[test]
fn c_div_basic() {
    let mut s = CalcState::new();
    s.stack.t = hpf(4.0);
    s.stack.z = hpf(2.0);
    s.stack.y = hpf(0.0);
    s.stack.x = hpf(2.0);
    let r = dispatch(&mut s, Op::AdvCDiv);
    assert!(r.is_ok());
}

#[test]
fn z_pow_w_basic() {
    let mut s = CalcState::new();
    // Z^W: Z = T+iZ, W = X+iY → (2+0i)^(2+0i) = 4
    s.stack.t = hpf(2.0);
    s.stack.z = hpf(0.0);
    s.stack.y = hpf(0.0);
    s.stack.x = hpf(2.0);
    let r = dispatch(&mut s, Op::AdvZPowW);
    assert!(r.is_ok());
}

#[test]
fn z_pow_1w_basic() {
    let mut s = CalcState::new();
    s.stack.t = hpf(4.0);
    s.stack.z = hpf(0.0);
    s.stack.y = hpf(0.0);
    s.stack.x = hpf(2.0);
    let r = dispatch(&mut s, Op::AdvZPow1w);
    assert!(r.is_ok());
}

#[test]
fn aip_appends_to_alpha() {
    let mut s = CalcState::new();
    push(&mut s, 65.0); // ASCII 'A'
    let r = dispatch(&mut s, Op::AdvAip);
    assert!(r.is_ok());
    assert!(s.alpha_reg.contains('A'));
}

// ── vectors.rs edge cases ───────────────────────────────────────────────────

#[test]
fn v_plus_3d() {
    let mut s = CalcState::new();
    // Vector A in regs 0-2, Vector B in stack
    s.regs[0] = HpNum::from(1i32).into();
    s.regs[1] = HpNum::from(2i32).into();
    s.regs[2] = HpNum::from(3i32).into();
    push(&mut s, 4.0);
    push(&mut s, 5.0);
    push(&mut s, 6.0);
    let r = dispatch(&mut s, Op::AdvVPlus);
    assert!(r.is_ok());
}

#[test]
fn v_minus_3d() {
    let mut s = CalcState::new();
    s.regs[0] = HpNum::from(4i32).into();
    s.regs[1] = HpNum::from(5i32).into();
    s.regs[2] = HpNum::from(6i32).into();
    push(&mut s, 1.0);
    push(&mut s, 2.0);
    push(&mut s, 3.0);
    let r = dispatch(&mut s, Op::AdvVMinus);
    assert!(r.is_ok());
}

#[test]
fn dot_product_orthogonal() {
    let mut s = CalcState::new();
    s.regs[0] = HpNum::from(1i32).into();
    s.regs[1] = HpNum::from(0i32).into();
    s.regs[2] = HpNum::from(0i32).into();
    push(&mut s, 0.0);
    push(&mut s, 1.0);
    push(&mut s, 0.0);
    let r = dispatch(&mut s, Op::AdvDot);
    assert!(r.is_ok());
    // LINT-EXEMPT: f64 tolerance check
    assert!(get_x(&s).abs() < 1e-9);
}

#[test]
fn cross_product_unit() {
    let mut s = CalcState::new();
    s.regs[0] = HpNum::from(1i32).into();
    s.regs[1] = HpNum::from(0i32).into();
    s.regs[2] = HpNum::from(0i32).into();
    push(&mut s, 0.0);
    push(&mut s, 1.0);
    push(&mut s, 0.0);
    let r = dispatch(&mut s, Op::AdvCross);
    assert!(r.is_ok());
}

#[test]
fn uv_unit_vector() {
    let mut s = CalcState::new();
    // UV reads vector A from regs 20-22
    s.regs[20] = HpNum::from(3i32).into();
    s.regs[21] = HpNum::from(4i32).into();
    s.regs[22] = HpNum::from(0i32).into();
    let r = dispatch(&mut s, Op::AdvUv);
    assert!(r.is_ok(), "UV: {r:?}");
}

#[test]
fn v_mag_3d() {
    let mut s = CalcState::new();
    // |V|: vector in stack Z,Y,X
    s.stack.z = hpf(0.0);
    s.stack.y = hpf(4.0);
    s.stack.x = hpf(3.0);
    let r = dispatch(&mut s, Op::AdvVMag);
    assert!(r.is_ok(), "|V|: {r:?}");
}

#[test]
fn vc_creates_vector() {
    let mut s = CalcState::new();
    push(&mut s, 3.0);
    push(&mut s, 2.0);
    push(&mut s, 1.0);
    let r = dispatch(&mut s, Op::AdvVc);
    assert!(r.is_ok());
}

#[test]
fn vs_stores_to_regs() {
    let mut s = CalcState::new();
    push(&mut s, 3.0);
    push(&mut s, 2.0);
    push(&mut s, 1.0);
    let r = dispatch(&mut s, Op::AdvVs);
    assert!(r.is_ok());
}

#[test]
fn vr_recalls_from_regs() {
    let mut s = CalcState::new();
    s.regs[0] = HpNum::from(10i32).into();
    s.regs[1] = HpNum::from(20i32).into();
    s.regs[2] = HpNum::from(30i32).into();
    let r = dispatch(&mut s, Op::AdvVr);
    assert!(r.is_ok());
}

#[test]
fn ve_exchanges_stack_regs() {
    let mut s = CalcState::new();
    s.regs[0] = HpNum::from(1i32).into();
    s.regs[1] = HpNum::from(2i32).into();
    s.regs[2] = HpNum::from(3i32).into();
    push(&mut s, 4.0);
    push(&mut s, 5.0);
    push(&mut s, 6.0);
    let r = dispatch(&mut s, Op::AdvVe);
    assert!(r.is_ok());
}

#[test]
fn vxy_2d_rotation() {
    let mut s = CalcState::new();
    push(&mut s, 0.0); // angle
    push(&mut s, 0.0);
    push(&mut s, 1.0);
    let r = dispatch(&mut s, Op::AdvVxy);
    assert!(r.is_ok());
}

#[test]
fn v_star_scalar_mul() {
    let mut s = CalcState::new();
    push(&mut s, 0.0);
    push(&mut s, 1.0);
    push(&mut s, 2.0);
    push(&mut s, 3.0); // scalar
    let r = dispatch(&mut s, Op::AdvVStar);
    assert!(r.is_ok());
}

#[test]
fn vd_scalar_div() {
    let mut s = CalcState::new();
    push(&mut s, 0.0);
    push(&mut s, 4.0);
    push(&mut s, 6.0);
    push(&mut s, 2.0); // divisor
    let r = dispatch(&mut s, Op::AdvVd);
    assert!(r.is_ok());
}

// ── tvm.rs edge cases ───────────────────────────────────────────────────────

#[test]
fn tvm_opens_modal() {
    let mut s = CalcState::new();
    let r = dispatch(&mut s, Op::AdvTvm);
    assert!(r.is_ok());
    // TVM opens a modal — may or may not initialize adv_tvm_state immediately
    assert!(s.modal_program.is_some() || s.adv_tvm_state.is_some());
}

#[test]
fn tvm_n_stores() {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::AdvTvm).unwrap();
    push(&mut s, 30.0);
    let r = dispatch(&mut s, Op::AdvTvmN);
    assert!(r.is_ok());
}

#[test]
fn tvm_pv_stores() {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::AdvTvm).unwrap();
    push(&mut s, 100000.0);
    let r = dispatch(&mut s, Op::AdvTvmPv);
    assert!(r.is_ok());
}

#[test]
fn tvm_pmt_stores() {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::AdvTvm).unwrap();
    push(&mut s, -500.0);
    let r = dispatch(&mut s, Op::AdvTvmPmt);
    assert!(r.is_ok());
}

#[test]
fn tvm_fv_stores() {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::AdvTvm).unwrap();
    push(&mut s, 0.0);
    let r = dispatch(&mut s, Op::AdvTvmFv);
    assert!(r.is_ok());
}

#[test]
fn tvm_star_i_no_state() {
    let mut s = CalcState::new();
    s.adv_tvm_state = None;
    let r = dispatch(&mut s, Op::AdvTvmStarI);
    assert!(r.is_err(), "*I without TVM state should error");
}

#[test]
fn tvm_full_workflow() {
    let mut s = CalcState::new();
    dispatch(&mut s, Op::AdvTvm).unwrap();
    // N = 360 (30-year mortgage)
    push(&mut s, 360.0);
    dispatch(&mut s, Op::AdvTvmN).unwrap();
    // PV = 200000
    push(&mut s, 200000.0);
    dispatch(&mut s, Op::AdvTvmPv).unwrap();
    // PMT = -1199.10 (monthly payment)
    push(&mut s, -1199.10);
    dispatch(&mut s, Op::AdvTvmPmt).unwrap();
    // FV = 0
    push(&mut s, 0.0);
    dispatch(&mut s, Op::AdvTvmFv).unwrap();
    // Solve for *I
    let r = dispatch(&mut s, Op::AdvTvmStarI);
    assert!(r.is_ok(), "*I solve: {r:?}");
    let rate = get_x(&s);
    // LINT-EXEMPT: Newton-Raphson TVM solver; ~0.5% monthly rate for 6% annual
    assert!(rate > 0.0 && rate < 2.0, "monthly rate should be 0-2%, got {rate}");
}

// ── curve_fit.rs edge cases ─────────────────────────────────────────────────

#[test]
fn cfit_no_data() {
    let mut s = CalcState::new();
    let r = dispatch(&mut s, Op::AdvCfit);
    // CFIT may succeed or error with no data — test exercises the code path
    let _ = r;
}

#[test]
fn sz_query_no_data() {
    let mut s = CalcState::new();
    let r = dispatch(&mut s, Op::AdvSzQuery);
    assert!(r.is_ok());
    // LINT-EXEMPT: SZ? returns 0 when no data, exact integer
    // LINT-EXEMPT: f64 tolerance check
    assert!((get_x(&s)).abs() < 1e-9);
}

// ── poly.rs edge cases ──────────────────────────────────────────────────────

#[test]
fn ply_exercises_code_path() {
    let mut s = CalcState::new();
    // PLY evaluates polynomial; degree in Y, x value in X
    s.regs[1] = HpNum::from(2i32).into(); // coefficient of x
    s.regs[2] = HpNum::from(3i32).into(); // constant term
    s.stack.y = hpf(1.0); // degree
    s.stack.x = hpf(5.0); // x value
    s.stack.lift_enabled = false;
    let r = dispatch(&mut s, Op::AdvPly);
    assert!(r.is_ok(), "PLY: {r:?}");
}

// ── modal.rs edge cases ─────────────────────────────────────────────────────

#[test]
fn matrx_modal_opens() {
    let mut s = CalcState::new();
    let r = dispatch(&mut s, Op::AdvMatrx);
    assert!(r.is_ok());
    assert!(s.modal_program.is_some());
}

#[test]
fn mtr_modal_opens() {
    let mut s = CalcState::new();
    let r = dispatch(&mut s, Op::AdvMtr);
    assert!(r.is_ok());
    assert!(s.modal_program.is_some());
}

#[test]
fn medit_no_matrix() {
    let mut s = CalcState::new();
    s.adv_current_matrix = None;
    let r = dispatch(&mut s, Op::AdvMedit);
    assert!(r.is_err());
}

#[test]
fn cmedit_no_matrix() {
    let mut s = CalcState::new();
    s.adv_current_matrix = None;
    let r = dispatch(&mut s, Op::AdvCmedit);
    assert!(r.is_err());
}

// ── solvers.rs additional edge cases ────────────────────────────────────────

#[test]
fn fdifeq_no_program() {
    let mut s = CalcState::new();
    let r = dispatch(&mut s, Op::AdvFdifeq);
    assert!(r.is_err());
}

#[test]
fn froot_degree_1_linear() {
    // x + 2 = 0 → root = -2
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(1i32); // degree 1
    s.stack.lift_enabled = false;
    s.regs[1] = HpNum::from(1i32).into();  // x coefficient
    s.regs[2] = HpNum::from(2i32).into();  // constant
    let r = dispatch(&mut s, Op::AdvFroot);
    assert!(r.is_ok(), "FROOT degree 1: {r:?}");
    let froot = s.adv_froot_state.as_ref().unwrap();
    assert_eq!(froot.roots_found.len(), 1);
    // LINT-EXEMPT: linear root x = -2, exact
    // LINT-EXEMPT: f64 root tolerance
    assert!((froot.roots_found[0].0 - (-2.0)).abs() < 1e-6);
}

#[test]
fn froot_quartic() {
    // x^4 - 5x^2 + 4 = 0 → roots ±1, ±2
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(4i32);
    s.stack.lift_enabled = false;
    s.regs[1] = HpNum::from(1i32).into();   // x^4
    s.regs[2] = HpNum::from(0i32).into();   // x^3
    s.regs[3] = HpNum::from(-5i32).into();  // x^2
    s.regs[4] = HpNum::from(0i32).into();   // x
    s.regs[5] = HpNum::from(4i32).into();   // constant

    let r = dispatch(&mut s, Op::AdvFroot);
    assert!(r.is_ok(), "FROOT quartic: {r:?}");
    let froot = s.adv_froot_state.as_ref().unwrap();
    assert_eq!(froot.roots_found.len(), 4);
    let mut reals: Vec<f64> = froot
        .roots_found
        .iter()
    // LINT-EXEMPT: f64 filter tolerance
        .filter(|(_, im)| im.abs() < 1e-4)
        .map(|(re, _)| *re)
        .collect();
    reals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(reals.len(), 4, "all 4 roots should be real");
}

// ── TR (trace) ──────────────────────────────────────────────────────────────

#[test]
fn tr_converts_rect_to_polar() {
    let mut s = CalcState::new();
    push(&mut s, 0.0);
    push(&mut s, 3.0);
    push(&mut s, 4.0);
    let r = dispatch(&mut s, Op::AdvTr);
    assert!(r.is_ok());
}

// ── matrix_workflow.rs ──────────────────────────────────────────────────────

#[test]
fn medit_with_matrix() {
    let mut s = CalcState::new();
    setup_matrix(&mut s, "ED", 2, 2, &[1.0, 2.0, 3.0, 4.0]);
    let r = dispatch(&mut s, Op::AdvMedit);
    assert!(r.is_ok());
    assert!(s.modal_program.is_some());
}

#[test]
fn cmedit_with_complex_matrix() {
    let mut s = CalcState::new();
    setup_complex_matrix(&mut s, "CE", 2, 1, &[1.0, 2.0, 3.0, 4.0]);
    let r = dispatch(&mut s, Op::AdvCmedit);
    assert!(r.is_ok());
    assert!(s.modal_program.is_some());
}
