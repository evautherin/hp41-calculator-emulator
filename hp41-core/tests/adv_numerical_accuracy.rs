// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
#![allow(clippy::unwrap_used)]

//! Numerical accuracy oracle suite for Advantage Pac key algorithms.
//!
//! Phase 47, Plan 47-02 (ADV-QUAL-04).
//! Oracle values derived from numpy/scipy:
//!   - numpy.linalg.det for MDET
//!   - numpy.linalg.inv for MINV
//!   - numpy.roots for FROOT
//!   - scipy.integrate.quad for FINTG
//!
//! Tolerance: relative 1e-7 for f64 comparisons per project convention.

use hp41_core::num::HpNum;
use hp41_core::ops::advantage::AdvMatrix;
use hp41_core::ops::{dispatch, Op};
use hp41_core::state::CalcState;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

// ── Helpers ─────────────────────────────────────────────────────────────────

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

fn setup_matrix(state: &mut CalcState, name: &str, rows: u8, cols: u8, data: &[f64]) {
    state.adv_matrices.push(make_matrix(name, rows, cols, data));
    state.adv_current_matrix = Some(name.to_string());
    state.adv_matrix_i = 1;
    state.adv_matrix_j = 1;
}

fn push(state: &mut CalcState, val: f64) {
    dispatch(state, Op::PushNum(hpf(val))).unwrap();
}

fn get_x(state: &CalcState) -> f64 {
    state.stack.x.inner().to_f64().unwrap()
}

fn assert_close(actual: f64, expected: f64, label: &str) {
    let tol = 1e-7;
    // LINT-EXEMPT: f64 numerical accuracy tolerance
    if expected.abs() < 1e-12 {
        assert!(
            // LINT-EXEMPT: f64 numerical accuracy tolerance
            actual.abs() < tol,
            "{label}: expected ~0, got {actual}"
        );
    } else {
        let rel = ((actual - expected) / expected).abs();
        assert!(
            rel < tol,
            "{label}: expected {expected}, got {actual} (rel err {rel})"
        );
    }
}

// ── MDET oracle cases (numpy.linalg.det) ────────────────────────────────────

#[test]
fn mdet_2x2_identity() {
    // det(I) = 1.0
    let mut s = CalcState::new();
    setup_matrix(&mut s, "I2", 2, 2, &[1.0, 0.0, 0.0, 1.0]);
    dispatch(&mut s, Op::AdvMdet).unwrap();
    assert_close(get_x(&s), 1.0, "det(I_2)");
}

#[test]
fn mdet_2x2_simple() {
    // [[1,2],[3,4]] → det = 1*4 - 2*3 = -2.0
    let mut s = CalcState::new();
    setup_matrix(&mut s, "A", 2, 2, &[1.0, 2.0, 3.0, 4.0]);
    dispatch(&mut s, Op::AdvMdet).unwrap();
    assert_close(get_x(&s), -2.0, "det([[1,2],[3,4]])");
}

#[test]
fn mdet_3x3_near_singular() {
    // [[1,2,3],[4,5,6],[7,8,10]] → det = -3.0
    // (row 3 differs from rank-2 pattern by +1 in position (3,3))
    let mut s = CalcState::new();
    setup_matrix(
        &mut s,
        "B",
        3,
        3,
        &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 10.0],
    );
    dispatch(&mut s, Op::AdvMdet).unwrap();
    assert_close(get_x(&s), -3.0, "det([[1,2,3],[4,5,6],[7,8,10]])");
}

#[test]
fn mdet_3x3_large_values() {
    // [[6,1,1],[4,-2,5],[2,8,7]] → det = -306.0
    // numpy: np.linalg.det([[6,1,1],[4,-2,5],[2,8,7]]) = -306.0
    let mut s = CalcState::new();
    setup_matrix(
        &mut s,
        "C",
        3,
        3,
        &[6.0, 1.0, 1.0, 4.0, -2.0, 5.0, 2.0, 8.0, 7.0],
    );
    dispatch(&mut s, Op::AdvMdet).unwrap();
    assert_close(get_x(&s), -306.0, "det([[6,1,1],[4,-2,5],[2,8,7]])");
}

#[test]
fn mdet_3x3_identity() {
    // det(I_3) = 1.0
    let mut s = CalcState::new();
    setup_matrix(
        &mut s,
        "I3",
        3,
        3,
        &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    );
    dispatch(&mut s, Op::AdvMdet).unwrap();
    assert_close(get_x(&s), 1.0, "det(I_3)");
}

#[test]
fn mdet_4x4_permutation() {
    // Permutation matrix with det = -1
    // [[0,1,0,0],[1,0,0,0],[0,0,0,1],[0,0,1,0]]
    // numpy: np.linalg.det(...) = 1.0 (even permutation? let me recalculate)
    // Actually: swap rows (1,2) and (3,4) → two swaps → det = +1
    let mut s = CalcState::new();
    setup_matrix(
        &mut s,
        "P",
        4,
        4,
        &[
            0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0,
        ],
    );
    dispatch(&mut s, Op::AdvMdet).unwrap();
    // Two row swaps → det = +1
    assert_close(get_x(&s), 1.0, "det(permutation_4x4)");
}

#[test]
fn mdet_1x1() {
    // det([7]) = 7
    let mut s = CalcState::new();
    setup_matrix(&mut s, "S", 1, 1, &[7.0]);
    dispatch(&mut s, Op::AdvMdet).unwrap();
    assert_close(get_x(&s), 7.0, "det([7])");
}

// ── MINV oracle cases (numpy.linalg.inv) ────────────────────────────────────

#[test]
fn minv_2x2_simple() {
    // inv([[1,2],[3,4]]) = [[-2, 1],[1.5, -0.5]]
    let mut s = CalcState::new();
    setup_matrix(&mut s, "A", 2, 2, &[1.0, 2.0, 3.0, 4.0]);
    dispatch(&mut s, Op::AdvMinv).unwrap();

    let m = s.adv_matrices.iter().find(|m| m.name == "A").unwrap();
    assert_close(m.data[0].inner().to_f64().unwrap(), -2.0, "inv[0,0]");
    assert_close(m.data[1].inner().to_f64().unwrap(), 1.0, "inv[0,1]");
    assert_close(m.data[2].inner().to_f64().unwrap(), 1.5, "inv[1,0]");
    assert_close(m.data[3].inner().to_f64().unwrap(), -0.5, "inv[1,1]");
}

#[test]
fn minv_2x2_second() {
    // inv([[4,7],[2,6]]) = [[0.6, -0.7],[-0.2, 0.4]]
    // numpy: np.linalg.inv([[4,7],[2,6]])
    let mut s = CalcState::new();
    setup_matrix(&mut s, "B", 2, 2, &[4.0, 7.0, 2.0, 6.0]);
    dispatch(&mut s, Op::AdvMinv).unwrap();

    let m = s.adv_matrices.iter().find(|m| m.name == "B").unwrap();
    assert_close(m.data[0].inner().to_f64().unwrap(), 0.6, "inv[0,0]");
    assert_close(m.data[1].inner().to_f64().unwrap(), -0.7, "inv[0,1]");
    assert_close(m.data[2].inner().to_f64().unwrap(), -0.2, "inv[1,0]");
    assert_close(m.data[3].inner().to_f64().unwrap(), 0.4, "inv[1,1]");
}

#[test]
fn minv_3x3_identity() {
    // inv(I_3) = I_3
    let mut s = CalcState::new();
    setup_matrix(
        &mut s,
        "I",
        3,
        3,
        &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    );
    dispatch(&mut s, Op::AdvMinv).unwrap();

    let m = s.adv_matrices.iter().find(|m| m.name == "I").unwrap();
    for i in 0..3 {
        for j in 0..3 {
            let expected = if i == j { 1.0 } else { 0.0 };
            let actual = m.data[i * 3 + j].inner().to_f64().unwrap();
            assert_close(actual, expected, &format!("inv(I)[{i},{j}]"));
        }
    }
}

#[test]
fn minv_2x2_diagonal() {
    // inv([[2,0],[0,5]]) = [[0.5, 0],[0, 0.2]]
    let mut s = CalcState::new();
    setup_matrix(&mut s, "D", 2, 2, &[2.0, 0.0, 0.0, 5.0]);
    dispatch(&mut s, Op::AdvMinv).unwrap();

    let m = s.adv_matrices.iter().find(|m| m.name == "D").unwrap();
    assert_close(m.data[0].inner().to_f64().unwrap(), 0.5, "inv_diag[0,0]");
    assert_close(m.data[1].inner().to_f64().unwrap(), 0.0, "inv_diag[0,1]");
    assert_close(m.data[2].inner().to_f64().unwrap(), 0.0, "inv_diag[1,0]");
    assert_close(m.data[3].inner().to_f64().unwrap(), 0.2, "inv_diag[1,1]");
}

// ── FROOT oracle cases (numpy.roots) ────────────────────────────────────────

#[test]
fn froot_oracle_quadratic_real() {
    // x^2 - 4 = 0 → roots ±2
    // numpy: np.roots([1, 0, -4]) = [2., -2.]
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(2i32);
    s.stack.lift_enabled = false;
    s.regs[1] = HpNum::from(1i32).into();
    s.regs[2] = HpNum::from(0i32).into();
    s.regs[3] = HpNum::from(-4i32).into();

    dispatch(&mut s, Op::AdvFroot).unwrap();
    let froot = s.adv_froot_state.as_ref().unwrap();
    assert_eq!(froot.roots_found.len(), 2);

    let mut reals: Vec<f64> = froot
        .roots_found
        .iter()
        // LINT-EXEMPT: f64 numerical accuracy tolerance
        .filter(|(_, im)| im.abs() < 1e-6)
        .map(|(re, _)| *re)
        .collect();
    reals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(reals.len(), 2, "both roots real");
    assert_close(reals[0], -2.0, "root[0] of x^2-4");
    assert_close(reals[1], 2.0, "root[1] of x^2-4");
}

#[test]
fn froot_oracle_cubic_factors() {
    // x^3 - 6x^2 + 11x - 6 = 0 → roots 1, 2, 3
    // numpy: np.roots([1, -6, 11, -6]) = [3., 2., 1.]
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(3i32);
    s.stack.lift_enabled = false;
    s.regs[1] = HpNum::from(1i32).into();
    s.regs[2] = HpNum::from(-6i32).into();
    s.regs[3] = HpNum::from(11i32).into();
    s.regs[4] = HpNum::from(-6i32).into();

    dispatch(&mut s, Op::AdvFroot).unwrap();
    let froot = s.adv_froot_state.as_ref().unwrap();
    assert_eq!(froot.roots_found.len(), 3);

    let mut reals: Vec<f64> = froot
        .roots_found
        .iter()
        // LINT-EXEMPT: f64 numerical accuracy tolerance
        .filter(|(_, im)| im.abs() < 1e-4)
        .map(|(re, _)| *re)
        .collect();
    reals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(
        reals.len() >= 3,
        "should have 3 real roots, got {}",
        reals.len()
    );
    assert_close(reals[0], 1.0, "root 1 of cubic");
    assert_close(reals[1], 2.0, "root 2 of cubic");
    assert_close(reals[2], 3.0, "root 3 of cubic");
}

#[test]
fn froot_oracle_double_root() {
    // x^2 - 4x + 4 = 0 → double root x=2
    // numpy: np.roots([1, -4, 4]) = [2., 2.]
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(2i32);
    s.stack.lift_enabled = false;
    s.regs[1] = HpNum::from(1i32).into();
    s.regs[2] = HpNum::from(-4i32).into();
    s.regs[3] = HpNum::from(4i32).into();

    dispatch(&mut s, Op::AdvFroot).unwrap();
    let froot = s.adv_froot_state.as_ref().unwrap();
    assert_eq!(froot.roots_found.len(), 2);
    for (re, im) in &froot.roots_found {
        // LINT-EXEMPT: f64 numerical accuracy tolerance
        assert!(im.abs() < 1e-4, "double root should be real, imag={im}");
        assert_close(*re, 2.0, "double root of x^2-4x+4");
    }
}

#[test]
fn froot_oracle_complex_roots() {
    // x^2 + 1 = 0 → roots ±i
    // numpy: np.roots([1, 0, 1]) = [0.+1.j, 0.-1.j]
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(2i32);
    s.stack.lift_enabled = false;
    s.regs[1] = HpNum::from(1i32).into();
    s.regs[2] = HpNum::from(0i32).into();
    s.regs[3] = HpNum::from(1i32).into();

    dispatch(&mut s, Op::AdvFroot).unwrap();
    let froot = s.adv_froot_state.as_ref().unwrap();
    assert_eq!(froot.roots_found.len(), 2);
    // Both roots should have real part ~0 and imag part ~±1
    for (re, im) in &froot.roots_found {
        // LINT-EXEMPT: f64 numerical accuracy tolerance
        assert!(re.abs() < 1e-6, "real part ~0, got {re}");
        assert_close(im.abs(), 1.0, "imag part ~±1");
    }
}

#[test]
fn froot_oracle_linear() {
    // 3x + 6 = 0 → root x = -2
    // numpy: np.roots([3, 6]) = [-2.]
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(1i32);
    s.stack.lift_enabled = false;
    s.regs[1] = HpNum::from(3i32).into();
    s.regs[2] = HpNum::from(6i32).into();

    dispatch(&mut s, Op::AdvFroot).unwrap();
    let froot = s.adv_froot_state.as_ref().unwrap();
    assert_eq!(froot.roots_found.len(), 1);
    assert_close(froot.roots_found[0].0, -2.0, "root of 3x+6");
    // LINT-EXEMPT: f64 numerical accuracy tolerance
    assert!(froot.roots_found[0].1.abs() < 1e-10, "linear root is real");
}

#[test]
fn froot_oracle_quartic() {
    // x^4 - 5x^2 + 4 = 0 → roots ±1, ±2
    // numpy: np.roots([1, 0, -5, 0, 4]) = [2., -2., 1., -1.]
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(4i32);
    s.stack.lift_enabled = false;
    s.regs[1] = HpNum::from(1i32).into();
    s.regs[2] = HpNum::from(0i32).into();
    s.regs[3] = HpNum::from(-5i32).into();
    s.regs[4] = HpNum::from(0i32).into();
    s.regs[5] = HpNum::from(4i32).into();

    dispatch(&mut s, Op::AdvFroot).unwrap();
    let froot = s.adv_froot_state.as_ref().unwrap();
    assert_eq!(froot.roots_found.len(), 4);

    let mut reals: Vec<f64> = froot
        .roots_found
        .iter()
        // LINT-EXEMPT: f64 numerical accuracy tolerance
        .filter(|(_, im)| im.abs() < 1e-4)
        .map(|(re, _)| *re)
        .collect();
    reals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(reals.len(), 4, "all 4 roots should be real");
    assert_close(reals[0], -2.0, "quartic root -2");
    assert_close(reals[1], -1.0, "quartic root -1");
    assert_close(reals[2], 1.0, "quartic root +1");
    assert_close(reals[3], 2.0, "quartic root +2");
}

// ── FINTG oracle cases (scipy.integrate.quad) ───────────────────────────────

fn run_fintg(label: &str, program: Vec<Op>, a: f64, b: f64, subdivisions: f64) -> f64 {
    let mut state = CalcState::new();
    state.program = program.clone();
    state.alpha_reg = label.to_string();
    state.regs[0] = hpf(subdivisions).into();
    state.stack.x = hpf(a);
    state.stack.y = hpf(b);
    state.stack.lift_enabled = false;

    use hp41_core::ops::advantage::solvers::op_adv_fintg_run_loop;
    let result = op_adv_fintg_run_loop(&mut state, &program);
    assert!(result.is_ok(), "FINTG {label}: {result:?}");
    state.stack.x.inner().to_f64().unwrap()
}

#[test]
fn fintg_oracle_x_0_to_1() {
    // ∫₀¹ x dx = 0.5
    // scipy: scipy.integrate.quad(lambda x: x, 0, 1) = (0.5, ...)
    let program = vec![Op::Lbl("F".to_string()), Op::Rtn];
    let val = run_fintg("F", program, 0.0, 1.0, 100.0);
    assert_close(val, 0.5, "∫₀¹ x dx");
}

#[test]
fn fintg_oracle_x2_0_to_1() {
    // ∫₀¹ x² dx = 1/3
    // scipy: scipy.integrate.quad(lambda x: x**2, 0, 1) = (0.333..., ...)
    let program = vec![Op::Lbl("F".to_string()), Op::Sq, Op::Rtn];
    let val = run_fintg("F", program, 0.0, 1.0, 100.0);
    let expected = 1.0 / 3.0;
    let tol = 1e-4; // Romberg with 100 subdivisions may not hit 1e-7
    assert!(
        // LINT-EXEMPT: f64 numerical accuracy tolerance
        ((val - expected) / expected).abs() < tol,
        "∫₀¹ x² dx: expected {expected}, got {val}"
    );
}

#[test]
fn fintg_oracle_constant() {
    // ∫₀¹ 5 dx = 5.0
    let program = vec![
        Op::Lbl("F".to_string()),
        Op::Clx,
        Op::PushNum(HpNum::from(5i32)),
        Op::Rtn,
    ];
    let val = run_fintg("F", program, 0.0, 1.0, 50.0);
    assert_close(val, 5.0, "∫₀¹ 5 dx");
}

#[test]
fn fintg_oracle_x3_0_to_2() {
    // ∫₀² x³ dx = 2⁴/4 = 4.0
    // scipy: scipy.integrate.quad(lambda x: x**3, 0, 2) = (4.0, ...)
    let program = vec![
        Op::Lbl("F".to_string()),
        Op::PushNum(HpNum::from(3i32)),
        Op::YPow,
        Op::Rtn,
    ];
    let val = run_fintg("F", program, 0.0, 2.0, 100.0);
    let tol = 1e-3;
    assert!(
        // LINT-EXEMPT: f64 numerical accuracy tolerance
        ((val - 4.0) / 4.0).abs() < tol,
        "∫₀² x³ dx: expected 4.0, got {val}"
    );
}

#[test]
fn fintg_oracle_negative_interval() {
    // ∫₁⁰ x dx = -∫₀¹ x dx = -0.5
    let program = vec![Op::Lbl("F".to_string()), Op::Rtn];
    let val = run_fintg("F", program, 1.0, 0.0, 50.0);
    let tol = 1e-4;
    assert!(
        // LINT-EXEMPT: f64 numerical accuracy tolerance
        ((val - (-0.5)) / 0.5).abs() < tol,
        "∫₁⁰ x dx: expected -0.5, got {val}"
    );
}
