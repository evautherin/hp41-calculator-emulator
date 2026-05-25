// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! v3.0 save-file backward compatibility + `migrate_after_load` migration tests
//! (Plan 37-03, STAT-QUAL-10).
//!
//! ## What this asserts
//!
//! Three tests verify the v3.0 → v3.1 save-file migration contract established
//! by D-33.7 (single-source-of-truth migration site at `state.rs:386-391`):
//!
//! 1. `v30_save_loads_with_stat1_migration` — a v3.0 fixture with
//!    `"xrom_modules": 1` (Math 1 only, bit 0) deserializes without error and
//!    `migrate_after_load` sets bit 1 (Stat 1), producing `xrom_modules == 0b11`.
//!
//! 2. `v30_save_rand_seed_defaults_to_zero` — the fixture intentionally OMITS
//!    `rand_seed`; `#[serde(default)]` WITHOUT `#[serde(skip)]` provides
//!    `HpNum::zero()` for the absent field (STAT-RNG-03 / Pitfall 20 contract).
//!
//! 3. `v30_save_stat1_op_resolves_after_migration` — after deserialization +
//!    migration, dispatching `Op::SigmaNormdWorkflow` opens the ΣNORMD modal
//!    (modal_prompt is Some), confirming Stat 1 Pac is operationally available.
//!
//! ## Why an integration test
//!
//! The migration call site (`migrate_after_load`) is `pub` on `CalcState` in
//! `crate::state`, and the deserialization / migration chain spans:
//!
//! 1. `serde_json::from_str::<CalcState>(fixture)` — external deserialization
//! 2. `state.migrate_after_load()` — post-load mutation
//! 3. `dispatch(state, Op::SigmaNormdWorkflow)` — operational smoke test
//!
//! An integration test exercises the full public API surface as would the
//! CLI / GUI persistence layers, mirroring the `test_calcstate_loads_without_new_fields`
//! pattern in `synthetic_tests.rs` (Phase 12 precedent).

#![allow(clippy::unwrap_used)]

use hp41_core::ops::{dispatch, Op};
use hp41_core::state::CalcState;
use rust_decimal::Decimal;

// ── Fixture ───────────────────────────────────────────────────────────────────

/// v3.0 CalcState fixture with `"xrom_modules": 1` (Math 1 only, bit 0).
///
/// Intentionally OMITS `rand_seed` — tests that `#[serde(default)]` WITHOUT
/// `#[serde(skip)]` provides `HpNum::zero()` for the absent field
/// (STAT-RNG-03 / Pitfall 20).
///
/// Also omits all `#[serde(skip)]` transient fields (modal_program, modal_prompt,
/// integ_state, solve_state, cancel_requested, print_buffer, pending_chisqd_nu)
/// since those are never persisted to disk.
static V30_FIXTURE: &str = include_str!("fixtures/v30-autosave.json");

// ── Test 1: Migration activates Stat 1 bit ────────────────────────────────────

/// Verify that a v3.0 save file (`xrom_modules = 1`) migrates correctly to
/// `xrom_modules = 0b0000_0011` (both Math 1 bit 0 and Stat 1 bit 1 set).
///
/// Catches: v3.0 → v3.1 upgrade path failure — if `migrate_after_load` is not
/// called or is incorrectly implemented, Stat 1 ops would return `InvalidOp`
/// on upgraded installations (D-33.7 / STAT-FW-02 regression).
#[test]
fn v30_save_loads_with_stat1_migration() {
    // Deserialize the v3.0 fixture — must succeed without error.
    let mut state: CalcState = serde_json::from_str(V30_FIXTURE).expect("v30 fixture deserializes");

    // Pre-migration: xrom_modules should be 1 (bit 0 = Math 1 only, as in v3.0).
    // LINT-EXEMPT: integer-equality — xrom_modules is u8, not HpNum; literal comparison is exact
    assert_eq!(state.xrom_modules, 1u8);

    // Apply the v3.0 → v3.2 migration (sets bits 1 + 2 on a v3.0 save).
    state.migrate_after_load();

    // Post-migration: bits 1 (Stat 1) + 2 (Time Module) must now be set alongside bit 0 (Math 1).
    // LINT-EXEMPT: integer-equality — xrom_modules is u8, not HpNum; the bitmask is exact
    assert_eq!(state.xrom_modules, 0b0000_0111u8);
}

// ── Test 2: rand_seed defaults to Decimal::ZERO when absent ──────────────────

/// Verify that `rand_seed` defaults to `HpNum::zero()` when absent from the
/// v3.0 fixture.
///
/// `rand_seed` is declared `#[serde(default)]` WITHOUT `#[serde(skip)]` — the
/// ONLY v3.1 CalcState field with this serde shape (STAT-RNG-03 / Pitfall 20).
/// The `#[serde(default)]` attribute provides `HpNum::zero()` for fields
/// absent in the serialized input. This test verifies that contract.
///
/// Catches: Pitfall 20 regression — if a future edit mistakenly adds `#[serde(skip)]`
/// to `rand_seed`, the field would NOT be deserialized from the fixture (it would
/// always default to zero even when a non-zero seed is present in the JSON).
/// The `rand_seed_serde_round_trip` test in `state.rs` guards the round-trip side;
/// this test guards the "absent-field defaults to zero" side.
#[test]
fn v30_save_rand_seed_defaults_to_zero() {
    let mut state: CalcState = serde_json::from_str(V30_FIXTURE).expect("v30 fixture deserializes");
    state.migrate_after_load();

    // rand_seed absent from fixture → #[serde(default)] provides HpNum::zero().
    // LINT-EXEMPT: integer-equality — comparing Decimal::ZERO via .inner(); the LCG seed
    // is an exact Decimal value (not f64-derived); rust_decimal integer construction is exact
    assert_eq!(state.rand_seed.inner(), Decimal::ZERO);
}

// ── Test 3: ΣNORMD op resolves after migration ────────────────────────────────

/// Verify that `Op::SigmaNormdWorkflow` is functionally available after the
/// v3.0 → v3.1 migration.
///
/// `op_sigma_normd_workflow` opens the ΣNORMD modal prompt (sets `modal_prompt`)
/// and returns `Ok(())`. A `None` modal prompt indicates the op failed silently,
/// which would happen if the Stat 1 XROM bit were not set after migration.
///
/// Note: `dispatch(Op::SigmaNormdWorkflow)` routes directly to the implementation
/// function regardless of `xrom_modules` — the xrom_modules bit controls the
/// *resolver chain* (`xeq_by_name_local_resolve` / `xrom_resolve`) but not the
/// direct `dispatch()` call. The test therefore validates the modal machinery,
/// not the XROM bit itself (bit-gate validation is in Test 1).
///
/// Catches: ΣNORMD modal initialization regression — if `op_sigma_normd_workflow`
/// fails to set `state.modal_prompt`, the CLI/GUI modal flow would silently no-op
/// with no user feedback.
#[test]
fn v30_save_stat1_op_resolves_after_migration() {
    let mut state: CalcState = serde_json::from_str(V30_FIXTURE).expect("v30 fixture deserializes");
    state.migrate_after_load();

    // Dispatch ΣNORMD workflow op — must open the modal prompt without error.
    let result = dispatch(&mut state, Op::SigmaNormdWorkflow);
    // LINT-EXEMPT: error-propagation check — no HpNum involved; Ok(()) vs Err path
    assert!(result.is_ok(), "SigmaNormdWorkflow failed: {result:?}");

    // The modal prompt must be set (non-None) — confirms modal initialization.
    // LINT-EXEMPT: Option::is_some() check — no HpNum involved; modal_prompt is Option<String>
    assert!(
        state.modal_prompt.is_some(),
        "expected modal_prompt to be Some after SigmaNormdWorkflow, got None"
    );
}
