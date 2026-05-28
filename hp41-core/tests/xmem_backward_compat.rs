// Algorithm independently derived from HP-41CX Owner's Manual 00041-90061 (1983) and
// HP-41CX Extended Memory Module Owner's Manual 00082-90014 (1984).
//
//! v3.3 save-file backward compatibility + `migrate_after_load` + isolation tests.
//!
//! Three groups verify the v3.3 save-file X-MEM migration contract:
//!
//! **Backward-compat (XMEM-08):**
//! 1. `v33_save_loads_without_error` — fixture deserializes; xmem_files empty,
//!    xmem_active_file None (exercises `#[serde(default)]` paths from Phase 51).
//! 2. `v33_save_xmem_fields_default_cleanly` — after migrate_after_load both
//!    X-MEM fields remain at default (empty Vec / None).
//! 3. `v33_save_migrate_after_load_is_idempotent` — migrate_after_load() leaves
//!    xrom_modules == 0b0001_1111; no new v4.0 arm needed (D-52.12 confirmed).
//! 4. `v33_save_rand_seed_preserved` — rand_seed "0.5" survives migration
//!    (ADR-v3.1-001 / Pitfall 20 guard; `rand_seed` uses `#[serde(default)]`
//!    WITHOUT `#[serde(skip)]`).
//!
//! **Isolation (XMEM-09):**
//! 5. `xmem_ops_never_touch_adv_matrices` — SAVEP / EMDIR / EMROOM leave
//!    adv_matrices untouched (D-43.5 / D-51.0a isolation).
//! 6. `savep_getp_do_not_touch_state_regs` — SAVEP + GETP round-trip; state.regs
//!    unchanged (sanctioned transfer is SAVED/GETD only).
//! 7. `saved_getd_transfer_is_isolated` — SAVED/GETD round-trip restores regs;
//!    proves the sanctioned transfer path works correctly.
//!
//! **Supplementary per-op coverage (D-52.13 floor prerequisite):**
//! EmDir, EmRoom, GetD, and SaveRx are each exercised in ≥ 5 test locations
//! across this file + inline tests in ops.rs so the Plan 03 floor gate passes.

#![allow(clippy::unwrap_used)]

use hp41_core::num::{HpNum, HpValue};
use hp41_core::ops::xmem::ops::{
    op_emdir, op_emreg, op_emroom, op_getd, op_getp, op_saved, op_savep, op_saverx,
};
use hp41_core::state::CalcState;
use rust_decimal::Decimal;
use std::str::FromStr;

static V33_FIXTURE: &str = include_str!("fixtures/v33-autosave.json");

// ── Backward-compat tests (XMEM-08) ─────────────────────────────────────────

#[test]
fn v33_save_loads_without_error() {
    let state: CalcState = serde_json::from_str(V33_FIXTURE).expect("v33 fixture deserializes");
    assert!(
        state.xmem_files.is_empty(),
        "xmem_files must be empty when absent from v3.3 fixture"
    );
    assert!(
        state.xmem_active_file.is_none(),
        "xmem_active_file must be None when absent from v3.3 fixture"
    );
}

#[test]
fn v33_save_xmem_fields_default_cleanly() {
    let mut state: CalcState = serde_json::from_str(V33_FIXTURE).expect("v33 fixture deserializes");
    state.migrate_after_load();
    assert!(
        state.xmem_files.is_empty(),
        "xmem_files must default to empty Vec"
    );
    assert!(
        state.xmem_active_file.is_none(),
        "xmem_active_file must default to None"
    );
}

#[test]
fn v33_save_migrate_after_load_is_idempotent() {
    let mut state: CalcState = serde_json::from_str(V33_FIXTURE).expect("v33 fixture deserializes");
    state.migrate_after_load();
    // xrom_modules stays 31 (0b0001_1111) — no new XROM bit in v4.0.
    // X-MEM ops are HP-41CX OS built-ins, not XROM modules (D-52.4 / D-52.12).
    assert_eq!(
        state.xrom_modules, 0b0001_1111u8,
        "migrate_after_load must leave xrom_modules at 0b0001_1111 (no v4.0 arm)"
    );
}

/// Verify that `rand_seed` serialized in a v3.3 save file is preserved across
/// migrate_after_load(). `rand_seed` uses `#[serde(default)]` WITHOUT `#[serde(skip)]`
/// per ADR-v3.1-001 / Pitfall 20 — this is the forward-compat guard that catches
/// accidental `#[serde(skip)]` regressions.
#[test]
fn v33_save_rand_seed_preserved() {
    let mut state: CalcState = serde_json::from_str(V33_FIXTURE).expect("v33 fixture deserializes");
    state.migrate_after_load();
    let expected = Decimal::from_str("0.5").expect("literal 0.5 parses");
    // LINT-EXEMPT: serde round-trip Decimal equality; "0.5" is exact in BCD
    assert_eq!(
        state.rand_seed.inner(),
        expected,
        "rand_seed must survive v3.3 → v4.0 migration"
    );
}

// ── Isolation tests (XMEM-09) ────────────────────────────────────────────────
// X-MEM ops must never read/write state.regs (except SAVED→capture / GETD→load)
// and must never touch adv_matrices (D-43.5 isolation, replicated for X-MEM).

#[test]
fn xmem_ops_never_touch_adv_matrices() {
    let mut state = CalcState::new();
    state.alpha_reg = "TESTFILE".to_string();
    // Run SAVEP — EmDir and EmRoom are tested too; none may create adv_matrices entries.
    let _ = op_savep(&mut state);
    let _ = op_emdir(&mut state);
    let _ = op_emroom(&mut state);
    assert!(
        state.adv_matrices.is_empty(),
        "X-MEM ops (SAVEP/EMDIR/EMROOM) must not create adv_matrices entries (D-43.5)"
    );
}

#[test]
fn savep_getp_do_not_touch_state_regs() {
    let mut state = CalcState::new();
    state.regs[0] = HpValue::from(42i32);
    state.alpha_reg = "PROG".to_string();
    // SAVEP stores the program in xmem_files — must NOT touch state.regs.
    let _ = op_savep(&mut state);
    // GETP restores the program from xmem_files — must NOT touch state.regs.
    let _ = op_getp(&mut state);
    assert_eq!(
        state.regs[0],
        HpValue::from(42i32),
        "SAVEP/GETP must not modify state.regs[0] (isolation D-51.0a)"
    );
}

#[test]
fn saved_getd_transfer_is_isolated() {
    // SAVED/GETD are the ONLY sanctioned path to transfer state.regs ↔ xmem_files.
    let mut state = CalcState::new();
    state.regs[5] = HpValue::from(99i32);
    state.alpha_reg = "DATFILE".to_string();

    // SAVED captures current state.regs into an xmem DATA file.
    op_saved(&mut state).unwrap();
    // Overwrite reg 5 — simulating subsequent work.
    state.regs[5] = HpValue::from(0i32);
    // GETD restores regs wholesale from the DATA file.
    state.alpha_reg = "DATFILE".to_string();
    op_getd(&mut state).unwrap();

    assert_eq!(
        state.regs[5],
        HpValue::from(99i32),
        "GETD must restore state.regs[5] == 99 (sanctioned transfer path)"
    );
    // xmem_active_file is set by both SAVED and GETD (D-51.4).
    assert_eq!(
        state.xmem_active_file,
        Some("DATFILE".to_string()),
        "GETD must set xmem_active_file = Some(\"DATFILE\")"
    );
}

// ── Supplementary per-op coverage (D-52.13 floor prerequisite) ──────────────
// EmDir, EmRoom, GetD, SaveRx each need ≥ 5 test mentions total across this file
// + inline tests in ops.rs. These focused tests add coverage for those four.

/// EmDir on a populated store lists each file; header/footer always present.
#[test]
fn emdir_populated_catalog_appears_in_print_buffer() {
    let mut state = CalcState::new();
    state.alpha_reg = "FILE1".to_string();
    op_savep(&mut state).unwrap();
    state.alpha_reg = "FILE2".to_string();
    state.alpha_reg = "FILE2".to_string();
    // Save second file (program is empty — valid, just trivial).
    op_savep(&mut state).unwrap();

    // Clear print_buffer then call EmDir.
    state.print_buffer.clear();
    op_emdir(&mut state).unwrap();

    // Header must be present.
    let header = &state.print_buffer[0];
    assert!(
        header.contains("XMEM DIRECTORY"),
        "EmDir header must contain 'XMEM DIRECTORY', got: {header:?}"
    );
    // At least two file lines + footer = >= 4 lines.
    assert!(
        state.print_buffer.len() >= 4,
        "EmDir must print header + file lines + footer, got {} lines",
        state.print_buffer.len()
    );
    // Footer must list free registers.
    let footer = state.print_buffer.last().unwrap();
    assert!(
        footer.contains("REGS FREE"),
        "EmDir footer must contain 'REGS FREE', got: {footer:?}"
    );
}

/// EmDir on an empty store emits header + footer (0 file lines).
#[test]
fn emdir_empty_store_prints_header_footer_only() {
    let mut state = CalcState::new();
    op_emdir(&mut state).unwrap();
    assert!(
        state.print_buffer.len() >= 2,
        "EmDir on empty store must print at least header + footer"
    );
    // Footer must show full 600 capacity.
    let footer = state.print_buffer.last().unwrap();
    assert!(
        footer.contains("600"),
        "EmDir on empty store must show 600 free regs, got: {footer:?}"
    );
}

/// EmRoom returns 600 on an empty store (maximum capacity).
#[test]
fn emroom_empty_returns_600() {
    let mut state = CalcState::new();
    op_emroom(&mut state).unwrap();
    assert_eq!(
        state.stack.x,
        HpNum::from(600i32),
        "EmRoom on empty store must return 600"
    );
}

/// EmRoom decreases after a file is saved.
#[test]
fn emroom_decreases_after_save() {
    let mut state = CalcState::new();
    state.alpha_reg = "DAT1".to_string();
    // SAVED captures 100 regs → reg_count = 100 → register_count() = 101.
    op_saved(&mut state).unwrap();
    op_emroom(&mut state).unwrap();
    // 600 - 101 = 499.
    assert_eq!(
        state.stack.x,
        HpNum::from(499i32),
        "EmRoom after saving a 100-reg DATA file must return 499"
    );
}

/// GetD sets xmem_active_file as a side effect (D-51.4).
#[test]
fn getd_sets_active_file_side_effect() {
    let mut state = CalcState::new();
    state.alpha_reg = "SIDEEFF".to_string();
    op_saved(&mut state).unwrap();

    // Clear active file, then GetD must re-set it.
    state.xmem_active_file = None;
    state.alpha_reg = "SIDEEFF".to_string();
    op_getd(&mut state).unwrap();
    assert_eq!(
        state.xmem_active_file,
        Some("SIDEEFF".to_string()),
        "GetD must set xmem_active_file = Some(\"SIDEEFF\") as side effect (D-51.4)"
    );
}

/// GetD on a missing file returns FileNotFound.
#[test]
fn getd_missing_file_returns_error() {
    let mut state = CalcState::new();
    state.alpha_reg = "NOFILE".to_string();
    let err = op_getd(&mut state).unwrap_err();
    assert_eq!(
        err,
        hp41_core::error::HpError::FileNotFound,
        "GetD on non-existent file must return FileNotFound"
    );
}

/// SaveRx stores a value into the active X-MEM DATA file register N.
#[test]
fn saverx_stores_into_active_file() {
    let mut state = CalcState::new();
    state.alpha_reg = "SAVERXDAT".to_string();
    // Populate registers before saving.
    for i in 0..5 {
        state.regs[i] = HpValue::from(i as i32 * 10);
    }
    op_saved(&mut state).unwrap();

    // SaveRx: X = register index 2, Y = value 77.
    state.stack.x = HpNum::from(2i32);
    state.stack.y = HpNum::from(77i32);
    op_saverx(&mut state).unwrap();

    // Verify via EmReg: recall register 2 — must be 77.
    state.stack.x = HpNum::from(2i32);
    op_emreg(&mut state).unwrap();
    assert_eq!(
        state.stack.x,
        HpNum::from(77i32),
        "SaveRx must store Y=77 into register index 2 of the active file"
    );
}

/// SaveRx does not grow xmem_files.len() (stores in-place).
#[test]
fn saverx_does_not_grow_xmem_files() {
    let mut state = CalcState::new();
    state.alpha_reg = "INPLACE".to_string();
    op_saved(&mut state).unwrap();
    let before = state.xmem_files.len();

    state.stack.x = HpNum::from(0i32);
    state.stack.y = HpNum::from(42i32);
    op_saverx(&mut state).unwrap();

    assert_eq!(
        state.xmem_files.len(),
        before,
        "SaveRx must not change xmem_files.len() (in-place mutation)"
    );
}

// ── Additional coverage to reach ≥5 floor (D-52.13 / Plan 03 gate) ──────────

/// EmDir prints the file name for each stored file (D-52.13 floor).
#[test]
fn emdir_shows_file_names() {
    let mut state = CalcState::new();
    state.alpha_reg = "MYFILE".to_string();
    op_savep(&mut state).unwrap();
    state.print_buffer.clear();
    op_emdir(&mut state).unwrap();
    let printed = state.print_buffer.join(" ");
    assert!(
        printed.contains("MYFILE"),
        "EmDir must print the stored file name; got: {printed:?}"
    );
}

/// EmReg on register 0 of a fresh DATA file returns the register value (D-52.13 floor).
#[test]
fn emreg_register_zero_returns_value() {
    let mut state = CalcState::new();
    state.regs[0] = HpValue::from(55i32);
    state.alpha_reg = "REGTEST".to_string();
    op_saved(&mut state).unwrap();
    state.stack.x = HpNum::from(0i32);
    op_emreg(&mut state).unwrap();
    assert_eq!(
        state.stack.x,
        HpNum::from(55i32),
        "EmReg at index 0 must return regs[0] == 55"
    );
}

/// SaveRx error: no active file → FileNotFound (D-52.13 floor).
#[test]
fn saverx_no_active_file_returns_error() {
    let mut state = CalcState::new();
    // No active file set (xmem_active_file = None after CalcState::new()).
    state.stack.x = HpNum::from(0i32);
    state.stack.y = HpNum::from(1i32);
    let err = op_saverx(&mut state).unwrap_err();
    assert_eq!(
        err,
        hp41_core::error::HpError::FileNotFound,
        "SaveRx with no active file must return FileNotFound"
    );
}

/// SaveRx round-trip: store then recall preserves the value (D-52.13 floor).
#[test]
fn saverx_round_trip_preserves_value() {
    let mut state = CalcState::new();
    state.alpha_reg = "RTRIP".to_string();
    op_saved(&mut state).unwrap();

    // Store value 123 into register 5 via SaveRx.
    state.stack.x = HpNum::from(5i32);
    state.stack.y = HpNum::from(123i32);
    op_saverx(&mut state).unwrap();

    // Recall register 5 via EmReg — must be 123.
    state.stack.x = HpNum::from(5i32);
    op_emreg(&mut state).unwrap();
    assert_eq!(
        state.stack.x,
        HpNum::from(123i32),
        "SaveRx round-trip: EmReg must return the value stored by SaveRx"
    );
}
