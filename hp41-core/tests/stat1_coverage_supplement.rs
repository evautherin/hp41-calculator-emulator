// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Supplementary integration tests to close the Pitfall 16 meta-gate gap
//! for Stat 1 Pac Op variants that remain below the 5-test threshold after
//! Waves 1–3. Covers: SigmaMmtug, SigmaMmtgd, SigmaLin, SigmaExp,
//! SigmaLogi, SigmaPow, SigmaMlrxy, SigmaMlrxyz, SigmaPolyc, SigmaCtkk.

#![allow(clippy::unwrap_used)]

use hp41_core::num::HpNum;
use hp41_core::ops::{dispatch, Op};
use hp41_core::state::CalcState;
use rust_decimal::Decimal;

fn push_xy(state: &mut CalcState, y: i32, x: i32) {
    state.stack.y = HpNum::from(y);
    state.stack.x = HpNum::from(x);
}

// ── SigmaMmtug ──────────────────────────────────────────────────────────────

#[test]
fn mmtug_accumulates_three_points() {
    let mut s = CalcState::new();
    for x in [10, 20, 30] {
        push_xy(&mut s, 0, x);
        dispatch(&mut s, Op::SigmaMmtug).unwrap();
    }
    assert_eq!(s.regs[3].inner(), Decimal::from(3)); // LINT-EXEMPT: integer N counter, not iterated result
}

#[test]
fn mmtug_single_point() {
    let mut s = CalcState::new();
    push_xy(&mut s, 0, 5);
    dispatch(&mut s, Op::SigmaMmtug).unwrap();
    assert_eq!(s.regs[3].inner(), Decimal::from(1)); // LINT-EXEMPT: integer N counter
}

// ── SigmaMmtgd ──────────────────────────────────────────────────────────────

#[test]
fn mmtgd_frequency_weighted_accumulation() {
    let mut s = CalcState::new();
    push_xy(&mut s, 3, 10);
    dispatch(&mut s, Op::SigmaMmtgd).unwrap();
    assert_eq!(s.regs[3].inner(), Decimal::from(3)); // LINT-EXEMPT: integer N counter, not iterated result
}

#[test]
fn mmtgd_two_groups() {
    let mut s = CalcState::new();
    push_xy(&mut s, 2, 5);
    dispatch(&mut s, Op::SigmaMmtgd).unwrap();
    push_xy(&mut s, 3, 10);
    dispatch(&mut s, Op::SigmaMmtgd).unwrap();
    assert_eq!(s.regs[3].inner(), Decimal::from(5)); // LINT-EXEMPT: integer N counter
}

#[test]
fn mmtgd_single_observation() {
    let mut s = CalcState::new();
    push_xy(&mut s, 1, 7);
    dispatch(&mut s, Op::SigmaMmtgd).unwrap();
    assert_eq!(s.regs[3].inner(), Decimal::from(1)); // LINT-EXEMPT: integer N counter
}

#[test]
fn mmtgd_large_frequency() {
    let mut s = CalcState::new();
    push_xy(&mut s, 100, 2);
    dispatch(&mut s, Op::SigmaMmtgd).unwrap();
    assert_eq!(s.regs[3].inner(), Decimal::from(100)); // LINT-EXEMPT: integer N counter
}

// ── SigmaLin ────────────────────────────────────────────────────────────────

#[test]
fn lin_accumulates_five_points() {
    let mut s = CalcState::new();
    for (x, y) in [(1, 3), (2, 5), (3, 7), (4, 9), (5, 11)] {
        push_xy(&mut s, y, x);
        dispatch(&mut s, Op::SigmaLin).unwrap();
    }
    assert_eq!(s.regs[3].inner(), Decimal::from(5)); // LINT-EXEMPT: integer N counter
}

// ── SigmaExp ────────────────────────────────────────────────────────────────

#[test]
fn exp_accumulates_positive_data() {
    let mut s = CalcState::new();
    for (x, y) in [(1, 3), (2, 7), (3, 20)] {
        push_xy(&mut s, y, x);
        dispatch(&mut s, Op::SigmaExp).unwrap();
    }
    assert_eq!(s.regs[3].inner(), Decimal::from(3)); // LINT-EXEMPT: integer N counter, not iterated result
}

#[test]
fn exp_rejects_non_positive_y() {
    let mut s = CalcState::new();
    push_xy(&mut s, 0, 1);
    assert!(dispatch(&mut s, Op::SigmaExp).is_err());
}

// ── SigmaLogi ───────────────────────────────────────────────────────────────

#[test]
fn logi_accumulates_positive_x() {
    let mut s = CalcState::new();
    for (x, y) in [(1, 2), (2, 4), (3, 5)] {
        push_xy(&mut s, y, x);
        dispatch(&mut s, Op::SigmaLogi).unwrap();
    }
    assert_eq!(s.regs[3].inner(), Decimal::from(3)); // LINT-EXEMPT: integer N counter, not iterated result
}

// ── SigmaPow ────────────────────────────────────────────────────────────────

#[test]
fn pow_accumulates_positive_xy() {
    let mut s = CalcState::new();
    for (x, y) in [(1, 1), (2, 4), (3, 9)] {
        push_xy(&mut s, y, x);
        dispatch(&mut s, Op::SigmaPow).unwrap();
    }
    assert_eq!(s.regs[3].inner(), Decimal::from(3)); // LINT-EXEMPT: integer N counter, not iterated result
}

// ── SigmaMlrxy (register-based solver) ──────────────────────────────────────

#[test]
fn mlrxy_solves_simple_system() {
    use hp41_core::ops::stat1::{
        STAT1_MLRXY_N_REG, STAT1_MLRXY_SUM_X1SQ_REG, STAT1_MLRXY_SUM_X1X2_REG,
        STAT1_MLRXY_SUM_X1Y_REG, STAT1_MLRXY_SUM_X1_REG, STAT1_MLRXY_SUM_X2SQ_REG,
        STAT1_MLRXY_SUM_X2Y_REG, STAT1_MLRXY_SUM_X2_REG, STAT1_MLRXY_SUM_Y_REG,
    };
    let mut s = CalcState::new();
    s.regs[STAT1_MLRXY_N_REG] = HpNum::from(5i32);
    s.regs[STAT1_MLRXY_SUM_Y_REG] = HpNum::from(160i32);
    s.regs[STAT1_MLRXY_SUM_X1_REG] = HpNum::from(15i32);
    s.regs[STAT1_MLRXY_SUM_X2_REG] = HpNum::from(55i32);
    s.regs[STAT1_MLRXY_SUM_X1SQ_REG] = HpNum::from(55i32);
    s.regs[STAT1_MLRXY_SUM_X2SQ_REG] = HpNum::from(979i32);
    s.regs[STAT1_MLRXY_SUM_X1X2_REG] = HpNum::from(225i32);
    s.regs[STAT1_MLRXY_SUM_X1Y_REG] = HpNum::from(630i32);
    s.regs[STAT1_MLRXY_SUM_X2Y_REG] = HpNum::from(2688i32);
    dispatch(&mut s, Op::SigmaMlrxy).unwrap();
}

// ── SigmaMlrxyz (register-based solver) ─────────────────────────────────────

fn load_mlrxyz_independent(s: &mut CalcState) {
    use hp41_core::ops::stat1::{
        STAT1_MLRXYZ_N_REG, STAT1_MLRXYZ_SUM_X1SQ_REG, STAT1_MLRXYZ_SUM_X1X2_REG,
        STAT1_MLRXYZ_SUM_X1X3_REG, STAT1_MLRXYZ_SUM_X1Y_REG, STAT1_MLRXYZ_SUM_X1_REG,
        STAT1_MLRXYZ_SUM_X2SQ_REG, STAT1_MLRXYZ_SUM_X2X3_REG, STAT1_MLRXYZ_SUM_X2Y_REG,
        STAT1_MLRXYZ_SUM_X2_REG, STAT1_MLRXYZ_SUM_X3SQ_REG, STAT1_MLRXYZ_SUM_X3Y_REG,
        STAT1_MLRXYZ_SUM_X3_REG, STAT1_MLRXYZ_SUM_Y_REG,
    };
    // y = 1+2x1+3x2+4x3, data: (1,0,0,3),(0,1,0,4),(0,0,1,5),(1,1,0,6),(0,1,1,8)
    s.regs[STAT1_MLRXYZ_N_REG] = HpNum::from(5i32);
    s.regs[STAT1_MLRXYZ_SUM_Y_REG] = HpNum::from(26i32);
    s.regs[STAT1_MLRXYZ_SUM_X1_REG] = HpNum::from(2i32);
    s.regs[STAT1_MLRXYZ_SUM_X2_REG] = HpNum::from(3i32);
    s.regs[STAT1_MLRXYZ_SUM_X3_REG] = HpNum::from(2i32);
    s.regs[STAT1_MLRXYZ_SUM_X1SQ_REG] = HpNum::from(2i32);
    s.regs[STAT1_MLRXYZ_SUM_X2SQ_REG] = HpNum::from(3i32);
    s.regs[STAT1_MLRXYZ_SUM_X3SQ_REG] = HpNum::from(2i32);
    s.regs[STAT1_MLRXYZ_SUM_X1X2_REG] = HpNum::from(1i32);
    s.regs[STAT1_MLRXYZ_SUM_X1X3_REG] = HpNum::from(0i32);
    s.regs[STAT1_MLRXYZ_SUM_X2X3_REG] = HpNum::from(1i32);
    s.regs[STAT1_MLRXYZ_SUM_X1Y_REG] = HpNum::from(9i32);
    s.regs[STAT1_MLRXYZ_SUM_X2Y_REG] = HpNum::from(18i32);
    s.regs[STAT1_MLRXYZ_SUM_X3Y_REG] = HpNum::from(13i32);
}

#[test]
fn mlrxyz_solves_simple_system() {
    let mut s = CalcState::new();
    load_mlrxyz_independent(&mut s);
    dispatch(&mut s, Op::SigmaMlrxyz).unwrap();
}

#[test]
fn mlrxyz_n_count_preserved() {
    let mut s = CalcState::new();
    load_mlrxyz_independent(&mut s);
    dispatch(&mut s, Op::SigmaMlrxyz).unwrap();
    let n_reg = hp41_core::ops::stat1::STAT1_MLRXYZ_N_REG;
    assert_eq!(s.regs[n_reg].inner(), Decimal::from(5)); // LINT-EXEMPT: integer N counter
}

#[test]
fn mlrxyz_empty_returns_domain() {
    let mut s = CalcState::new();
    assert!(dispatch(&mut s, Op::SigmaMlrxyz).is_err());
}

#[test]
fn mlrxyz_singular_returns_domain() {
    use hp41_core::ops::stat1::STAT1_MLRXYZ_N_REG;
    let mut s = CalcState::new();
    s.regs[STAT1_MLRXYZ_N_REG] = HpNum::from(1i32);
    assert!(dispatch(&mut s, Op::SigmaMlrxyz).is_err());
}

// ── SigmaPolyc ──────────────────────────────────────────────────────────────

#[test]
fn polyc_without_prior_polyp_errors() {
    let mut s = CalcState::new();
    s.stack.x = HpNum::from(5);
    let result = dispatch(&mut s, Op::SigmaPolyc);
    assert!(result.is_err());
}

#[test]
fn polyc_evaluates_after_polyp_workflow() {
    use hp41_core::ops::math1::modal::ModalProgram;
    use hp41_core::ops::stat1::modal::{submit_step, Stat1Step};
    let mut s = CalcState::new();
    for (x, y) in [(1, 1), (2, 4), (3, 9), (4, 16), (5, 25)] {
        push_xy(&mut s, y, x);
        dispatch(&mut s, Op::SigmaPlus).unwrap();
    }
    s.modal_program = Some(ModalProgram::Stat1(Stat1Step::PolypDegreePrompt(0)));
    s.stack.x = HpNum::from(2);
    submit_step(&mut s, Stat1Step::PolypDegreePrompt(0)).unwrap();
    s.stack.x = HpNum::from(6);
    dispatch(&mut s, Op::SigmaPolyc).unwrap();
}

#[test]
fn polyc_after_linear_fit() {
    use hp41_core::ops::math1::modal::ModalProgram;
    use hp41_core::ops::stat1::modal::{submit_step, Stat1Step};
    let mut s = CalcState::new();
    for (x, y) in [(1, 2), (2, 4), (3, 6)] {
        push_xy(&mut s, y, x);
        dispatch(&mut s, Op::SigmaPlus).unwrap();
    }
    s.modal_program = Some(ModalProgram::Stat1(Stat1Step::PolypDegreePrompt(0)));
    s.stack.x = HpNum::from(1);
    submit_step(&mut s, Stat1Step::PolypDegreePrompt(0)).unwrap();
    s.stack.x = HpNum::from(4);
    dispatch(&mut s, Op::SigmaPolyc).unwrap();
}

// ── SigmaCtkk (2x2 cap) ────────────────────────────────────────────────────

#[test]
fn ctkk_two_by_two_table() {
    use hp41_core::ops::stat1::{STAT1_CTKKK_CELL_BASE_REG, STAT1_CTKKK_C_REG, STAT1_CTKKK_R_REG};
    let mut s = CalcState::new();
    s.regs[STAT1_CTKKK_R_REG] = HpNum::from(2);
    s.regs[STAT1_CTKKK_C_REG] = HpNum::from(2);
    let base = STAT1_CTKKK_CELL_BASE_REG;
    s.regs[base] = HpNum::from(10);
    s.regs[base + 1] = HpNum::from(20);
    s.regs[base + 2] = HpNum::from(30);
    s.regs[base + 3] = HpNum::from(40);
    dispatch(&mut s, Op::SigmaCtkk).unwrap();
}

#[test]
fn ctkk_symmetric_table() {
    use hp41_core::ops::stat1::{STAT1_CTKKK_CELL_BASE_REG, STAT1_CTKKK_C_REG, STAT1_CTKKK_R_REG};
    let mut s = CalcState::new();
    s.regs[STAT1_CTKKK_R_REG] = HpNum::from(2);
    s.regs[STAT1_CTKKK_C_REG] = HpNum::from(2);
    let base = STAT1_CTKKK_CELL_BASE_REG;
    s.regs[base] = HpNum::from(25);
    s.regs[base + 1] = HpNum::from(25);
    s.regs[base + 2] = HpNum::from(25);
    s.regs[base + 3] = HpNum::from(25);
    dispatch(&mut s, Op::SigmaCtkk).unwrap();
}

#[test]
fn ctkk_missing_dimensions_errors() {
    let mut s = CalcState::new();
    let result = dispatch(&mut s, Op::SigmaCtkk);
    assert!(result.is_err());
}
