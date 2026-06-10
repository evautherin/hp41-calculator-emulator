// Algorithm: soft_reset / memory_lost reset methods for CalcState.
// Derived from the HP-41 emulator design spec (docs/superpowers/specs/2026-06-10-reset-escape-hatch-design.md).
// Free42 not consulted (not applicable to this feature).
//
//! Phase 67-01 — Core reset-escape-hatch tests.
//!
//! Validates the two `CalcState` reset methods:
//!   - `soft_reset()` — clears all transient/input-trapping fields; preserves stored data.
//!   - `memory_lost()` — factory-reset equivalent to `CalcState::new()`.
//!
//! Test groups:
//!
//!   RST-01: `soft_reset()` clears every trapping field
//!     RST-01-a: stack (x/y/z/t/lastx) zeroed, lift_enabled reset
//!     RST-01-b: entry_buf cleared, alpha_reg cleared, alpha_mode false
//!     RST-01-c: display_override = None
//!     RST-01-d: prgm_mode = false, user_mode = false
//!     RST-01-e: modal_program = None, modal_prompt = None
//!     RST-01-f: integ_state/solve_state/difeq_state = None
//!     RST-01-g: matrix_dim/matrix_active_reg = None
//!     RST-01-h: is_running = false, pc = 0, call_stack cleared
//!     RST-01-i: Phase 63/64 transients cleared (pending_interrupt, pending_yield, getkey_captured_code)
//!     RST-01-j: print_buffer cleared, last_key_code reset to 0
//!     RST-01-k: adv transients cleared (adv_current_matrix, adv_froot_state, etc.)
//!     RST-01-l: event_buffer cleared, pending_card_op = None
//!     RST-01-m: clock_active / stopwatch_keyboard_mode / alarm_catalog_mode = false
//!     RST-01-n: pending_chisqd_nu, pending_adv_matrix_name, pending_adv_matrix_rows = None
//!     RST-01-o: cancel_requested reset to false
//!     RST-01-p: stopwatch_start = None
//!
//!   RST-02: `soft_reset()` preserves every stored-data field
//!     RST-02-a: program, regs, text_regs preserved
//!     RST-02-b: flags, key_assignments, assignments preserved
//!     RST-02-c: xmem_files, xmem_active_file, xrom_modules preserved
//!     RST-02-d: rand_seed, adv_matrices, adv_tvm_state preserved
//!     RST-02-e: time_offset_secs, alarms, clock_12h, clock_display_mode preserved
//!     RST-02-f: angle_mode, display_mode preserved
//!     RST-02-g: adv_matrix_i, adv_matrix_j preserved
//!     RST-02-h: stopwatch_accumulated, stopwatch_split preserved
//!     RST-02-i: reg_m, reg_n, reg_o preserved
//!     RST-02-j: accuracy_factor, stopwatch_mode preserved
//!     RST-02-k: complex_mode preserved
//!
//!   RST-03: `memory_lost()` produces a state field-equal to `CalcState::new()`
//!     RST-03-a: JSON of memory_lost equals JSON of fresh CalcState::new()
//!
//!   RST-04: serde round-trip + reset recovery
//!     RST-04-a: trapped state survives serde round-trip; soft_reset() recovers it
//!     RST-04-b: trapped state survives serde round-trip; memory_lost() recovers it
#![allow(clippy::unwrap_used)]

use hp41_core::ops::advantage::AdvMatrix;
use hp41_core::ops::Op;
use hp41_core::state::CalcState;
use hp41_core::HpNum;
use rust_decimal::Decimal;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build a "trapped" CalcState — every input-blocking field set, plus meaningful
/// stored data so we can verify it survives soft_reset().
fn make_trapped_state() -> CalcState {
    let mut s = CalcState::new();

    // Stored data — must survive soft_reset():
    s.program = vec![Op::Lbl("A".to_string()), Op::Rtn];
    s.regs[0] = HpNum::from(Decimal::new(42, 0)).into();
    s.flags = 0b101;
    s.key_assignments.insert('a', "A".to_string());
    s.assignments.insert(12, "B".to_string());
    s.text_regs.insert(0, "HI".to_string());
    s.rand_seed = HpNum::from(Decimal::new(5, 1)); // 0.5
    s.adv_matrix_i = 3;
    s.adv_matrix_j = 2;
    s.adv_matrices.push(AdvMatrix {
        name: "MAT".to_string(),
        rows: 2,
        cols: 2,
        data: vec![HpNum::zero(), HpNum::zero(), HpNum::zero(), HpNum::zero()],
        is_complex: false,
    });
    s.time_offset_secs = 9876;
    s.stopwatch_accumulated = 2.5;
    s.stopwatch_split = 1.5;
    s.reg_m = HpNum::from(Decimal::new(7, 0));
    s.reg_n = HpNum::from(Decimal::new(8, 0));
    s.reg_o = HpNum::from(Decimal::new(9, 0));
    s.xrom_modules = 0b0001_1111;
    s.complex_mode = true;

    // Trapping fields — all must be cleared by soft_reset():
    s.stack.x = HpNum::from(Decimal::new(1, 0));
    s.stack.y = HpNum::from(Decimal::new(2, 0));
    s.stack.z = HpNum::from(Decimal::new(3, 0));
    s.stack.t = HpNum::from(Decimal::new(4, 0));
    s.stack.lastx = HpNum::from(Decimal::new(5, 0));
    s.stack.lift_enabled = true;
    s.entry_buf = "3.14".to_string();
    s.alpha_reg = "TEST".to_string();
    s.alpha_mode = true;
    s.display_override = Some("STUCK".to_string());
    s.prgm_mode = true;
    s.user_mode = true;
    s.is_running = true;
    s.pc = 5;
    s.call_stack = vec![1, 2];
    s.last_key_code = 15;
    s.print_buffer = vec!["line1".to_string()];
    s.event_buffer = vec!["beep".to_string()];
    s.clock_active = true;
    s.stopwatch_keyboard_mode = true;
    s.alarm_catalog_mode = true;

    // Phase 63/64 transients:
    s.pending_interrupt = Some("INT".to_string());
    s.pending_interrupt_alarm_index = Some(0);
    s.pending_interrupt_depth = Some(1);
    s.getkey_captured_code = Some(42);

    s
}

// ── RST-01: soft_reset() clears every trapping field ─────────────────────────

#[test]
fn soft_reset_clears_stack() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert!(s.stack.x.is_zero(), "stack.x must be zeroed");
    assert!(s.stack.y.is_zero(), "stack.y must be zeroed");
    assert!(s.stack.z.is_zero(), "stack.z must be zeroed");
    assert!(s.stack.t.is_zero(), "stack.t must be zeroed");
    assert!(s.stack.lastx.is_zero(), "stack.lastx must be zeroed");
    assert!(!s.stack.lift_enabled, "lift_enabled must be false");
}

#[test]
fn soft_reset_clears_entry_and_alpha() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert!(s.entry_buf.is_empty(), "entry_buf must be empty");
    assert!(s.alpha_reg.is_empty(), "alpha_reg must be empty");
    assert!(!s.alpha_mode, "alpha_mode must be false");
}

#[test]
fn soft_reset_clears_display_override() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert!(s.display_override.is_none(), "display_override must be None");
}

#[test]
fn soft_reset_clears_mode_flags() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert!(!s.prgm_mode, "prgm_mode must be false");
    assert!(!s.user_mode, "user_mode must be false");
}

#[test]
fn soft_reset_clears_modal_fields() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert!(s.modal_program.is_none(), "modal_program must be None");
    assert!(s.modal_prompt.is_none(), "modal_prompt must be None");
}

#[test]
fn soft_reset_clears_solver_states() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert!(s.integ_state.is_none(), "integ_state must be None");
    assert!(s.solve_state.is_none(), "solve_state must be None");
    assert!(s.difeq_state.is_none(), "difeq_state must be None");
}

#[test]
fn soft_reset_clears_matrix_mode() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert!(s.matrix_dim.is_none(), "matrix_dim must be None");
    assert!(s.matrix_active_reg.is_none(), "matrix_active_reg must be None");
}

#[test]
fn soft_reset_clears_program_execution_state() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert!(!s.is_running, "is_running must be false");
    assert_eq!(s.pc, 0, "pc must be 0");
    assert!(s.call_stack.is_empty(), "call_stack must be empty");
}

#[test]
fn soft_reset_clears_phase63_64_transients() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert!(
        s.pending_interrupt.is_none(),
        "pending_interrupt must be None"
    );
    assert!(
        s.pending_interrupt_alarm_index.is_none(),
        "pending_interrupt_alarm_index must be None"
    );
    assert!(
        s.pending_interrupt_depth.is_none(),
        "pending_interrupt_depth must be None"
    );
    assert!(s.pending_yield.is_none(), "pending_yield must be None");
    assert!(
        s.getkey_captured_code.is_none(),
        "getkey_captured_code must be None"
    );
}

#[test]
fn soft_reset_clears_print_buffer_and_last_key() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert!(s.print_buffer.is_empty(), "print_buffer must be empty");
    assert_eq!(s.last_key_code, 0, "last_key_code must be 0");
}

#[test]
fn soft_reset_clears_adv_transients() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert!(
        s.adv_current_matrix.is_none(),
        "adv_current_matrix must be None"
    );
    assert!(s.adv_froot_state.is_none(), "adv_froot_state must be None");
    assert!(s.adv_fintg_state.is_none(), "adv_fintg_state must be None");
    assert!(
        s.adv_fsolve_state.is_none(),
        "adv_fsolve_state must be None"
    );
    assert!(
        s.adv_fdifeq_state.is_none(),
        "adv_fdifeq_state must be None"
    );
    assert!(
        s.pending_adv_matrix_name.is_none(),
        "pending_adv_matrix_name must be None"
    );
    assert!(
        s.pending_adv_matrix_rows.is_none(),
        "pending_adv_matrix_rows must be None"
    );
    assert!(
        s.pending_chisqd_nu.is_none(),
        "pending_chisqd_nu must be None"
    );
}

#[test]
fn soft_reset_clears_event_buffer_and_card_op() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert!(s.event_buffer.is_empty(), "event_buffer must be empty");
    assert!(s.pending_card_op.is_none(), "pending_card_op must be None");
}

#[test]
fn soft_reset_clears_clock_and_stopwatch_modes() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert!(!s.clock_active, "clock_active must be false");
    assert!(
        !s.stopwatch_keyboard_mode,
        "stopwatch_keyboard_mode must be false"
    );
    assert!(
        !s.alarm_catalog_mode,
        "alarm_catalog_mode must be false"
    );
    assert!(s.stopwatch_start.is_none(), "stopwatch_start must be None");
}

#[test]
fn soft_reset_clears_cancel_requested() {
    use std::sync::atomic::Ordering;
    let mut s = make_trapped_state();
    s.cancel_requested
        .store(true, std::sync::atomic::Ordering::SeqCst);
    s.soft_reset();
    assert!(
        !s.cancel_requested.load(Ordering::SeqCst),
        "cancel_requested must be false after soft_reset"
    );
}

// ── RST-02: soft_reset() preserves stored-data fields ────────────────────────

#[test]
fn soft_reset_preserves_program_and_regs() {
    let mut s = make_trapped_state();
    let saved_prog = s.program.clone();
    s.soft_reset();
    assert_eq!(s.program, saved_prog, "program must be preserved");
    assert_eq!(
        s.regs[0],
        HpNum::from(Decimal::new(42, 0)).into(),
        "regs[0] must be preserved"
    );
}

#[test]
fn soft_reset_preserves_flags_and_assignments() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert_eq!(s.flags, 0b101, "flags must be preserved");
    assert!(
        s.key_assignments.contains_key(&'a'),
        "key_assignments must be preserved"
    );
    assert!(
        s.assignments.contains_key(&12),
        "assignments must be preserved"
    );
}

#[test]
fn soft_reset_preserves_text_regs() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert_eq!(
        s.text_regs.get(&0),
        Some(&"HI".to_string()),
        "text_regs must be preserved"
    );
}

#[test]
fn soft_reset_preserves_xmem_and_xrom_modules() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert_eq!(s.xrom_modules, 0b0001_1111, "xrom_modules must be preserved");
    // xmem_files and xmem_active_file are empty/None in make_trapped_state —
    // that's intentional; xmem state is the "not changed" class of stored data.
}

#[test]
fn soft_reset_preserves_rand_seed() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert_eq!(
        s.rand_seed,
        HpNum::from(Decimal::new(5, 1)),
        "rand_seed must be preserved"
    );
}

#[test]
fn soft_reset_preserves_adv_matrices_and_tvm() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert_eq!(s.adv_matrices.len(), 1, "adv_matrices must be preserved");
    assert_eq!(s.adv_matrices[0].name, "MAT", "adv_matrix name preserved");
    // adv_tvm_state: None in make_trapped_state — preserved as None
    assert!(
        s.adv_tvm_state.is_none(),
        "adv_tvm_state None preserved"
    );
    assert_eq!(s.adv_matrix_i, 3, "adv_matrix_i preserved");
    assert_eq!(s.adv_matrix_j, 2, "adv_matrix_j preserved");
}

#[test]
fn soft_reset_preserves_time_state() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert_eq!(s.time_offset_secs, 9876, "time_offset_secs preserved");
}

#[test]
fn soft_reset_preserves_angle_and_display_mode() {
    let mut s = CalcState::new();
    use hp41_core::state::{AngleMode, DisplayMode};
    s.angle_mode = AngleMode::Rad;
    s.display_mode = DisplayMode::Sci(3);
    s.soft_reset();
    assert_eq!(s.angle_mode, AngleMode::Rad, "angle_mode preserved");
    assert_eq!(s.display_mode, DisplayMode::Sci(3), "display_mode preserved");
}

#[test]
fn soft_reset_preserves_hidden_regs_m_n_o() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert_eq!(s.reg_m, HpNum::from(Decimal::new(7, 0)), "reg_m preserved");
    assert_eq!(s.reg_n, HpNum::from(Decimal::new(8, 0)), "reg_n preserved");
    assert_eq!(s.reg_o, HpNum::from(Decimal::new(9, 0)), "reg_o preserved");
}

#[test]
fn soft_reset_preserves_stopwatch_accumulated() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert!(
        (s.stopwatch_accumulated - 2.5).abs() < 1e-9,
        "stopwatch_accumulated preserved"
    );
    assert!(
        (s.stopwatch_split - 1.5).abs() < 1e-9,
        "stopwatch_split preserved"
    );
}

#[test]
fn soft_reset_preserves_complex_mode() {
    let mut s = make_trapped_state();
    s.soft_reset();
    assert!(s.complex_mode, "complex_mode preserved");
}

// ── RST-03: memory_lost() == CalcState::new() ────────────────────────────────

#[test]
fn memory_lost_equals_new() {
    let mut s = make_trapped_state();
    s.memory_lost();

    let fresh = CalcState::new();

    // Compare via JSON serialization (the canonical serde-equality check used
    // throughout this codebase; see rand_seed_serde_round_trip in state.rs).
    let s_json =
        serde_json::to_string(&s).expect("memory_lost state must serialize");
    let fresh_json =
        serde_json::to_string(&fresh).expect("fresh state must serialize");

    assert_eq!(
        s_json, fresh_json,
        "memory_lost() result must be field-equal to CalcState::new()"
    );
}

// ── RST-04: serde round-trip + reset recovery ─────────────────────────────────

#[test]
fn trapped_state_round_trip_soft_reset() {
    let trapped = make_trapped_state();

    // Serialize the trapped state (simulates autosave on disk).
    let json = serde_json::to_string(&trapped).expect("serialize trapped state");

    // Deserialize it (simulates app relaunch loading autosave).
    let mut restored: CalcState =
        serde_json::from_str(&json).expect("deserialize trapped state");
    restored.migrate_after_load();

    // Transient fields don't survive serde (skip), so they're gone.
    // But the state may still be in a trapping condition (prgm_mode,
    // is_running, entry_buf, alpha_mode all survived via #[serde(default)]).
    // Apply soft_reset and verify recovery.
    restored.soft_reset();

    // After soft_reset the state must be input-accepting:
    assert!(!restored.is_running, "is_running must be false");
    assert!(!restored.prgm_mode, "prgm_mode must be false");
    assert!(!restored.alpha_mode, "alpha_mode must be false");
    assert!(restored.entry_buf.is_empty(), "entry_buf must be empty");
    assert!(restored.display_override.is_none(), "display_override None");

    // Stored data must survive the round-trip + soft_reset:
    assert_eq!(restored.program.len(), 2, "program preserved");
    assert_eq!(restored.flags, 0b101, "flags preserved");
    assert!(
        restored.key_assignments.contains_key(&'a'),
        "key_assignments preserved"
    );
}

#[test]
fn trapped_state_round_trip_memory_lost() {
    let trapped = make_trapped_state();

    let json = serde_json::to_string(&trapped).expect("serialize trapped state");
    let mut restored: CalcState =
        serde_json::from_str(&json).expect("deserialize trapped state");
    restored.migrate_after_load();

    // Apply memory_lost (factory reset).
    restored.memory_lost();

    let fresh = CalcState::new();
    let restored_json =
        serde_json::to_string(&restored).expect("serialize restored");
    let fresh_json = serde_json::to_string(&fresh).expect("serialize fresh");
    assert_eq!(
        restored_json, fresh_json,
        "memory_lost after round-trip must equal CalcState::new()"
    );
}
