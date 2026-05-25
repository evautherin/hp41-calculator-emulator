// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! v3.1 save-file backward compatibility + `migrate_after_load` migration tests
//! (Plan 42-04, TIME-QUAL-04).
//!
//! ## What this asserts
//!
//! Three tests verify the v3.1 → v3.2 save-file migration contract established
//! by D-33.7 (single-source-of-truth migration site at `state.rs::migrate_after_load`):
//!
//! 1. `v31_save_loads_with_time_migration` — a v3.1 fixture with
//!    `"xrom_modules": 3` (Math 1 + Stat 1, bits 0+1) deserializes without error
//!    and `migrate_after_load` sets bit 2 (Time Module), producing
//!    `xrom_modules == 0b0000_0111`.
//!
//! 2. `v31_save_time_fields_default_cleanly` — after deserialization + migration,
//!    all Time Module `CalcState` fields (which are absent from the v3.1 fixture)
//!    default via `#[serde(default)]` to their zero/false/empty values.
//!
//! 3. `v31_save_rand_seed_preserved` — the fixture includes `"rand_seed": "0.5"`;
//!    this test confirms rand_seed survives the v3.2 migration unchanged
//!    (STAT-RNG-03 / Pitfall 20 regression guard).
//!
//! ## Why an integration test
//!
//! The migration call site (`migrate_after_load`) is `pub` on `CalcState` in
//! `crate::state`, and the deserialization / migration chain spans:
//!
//! 1. `serde_json::from_str::<CalcState>(fixture)` — external deserialization
//! 2. `state.migrate_after_load()` — post-load mutation
//! 3. field-level assertions — operational smoke test
//!
//! An integration test exercises the full public API surface as would the
//! CLI / GUI persistence layers, mirroring the `stat1_backward_compat.rs`
//! pattern (Phase 37, STAT-QUAL-10).

#![allow(clippy::unwrap_used)]

use hp41_core::ops::time::StopwatchMode;
use hp41_core::state::CalcState;
use rust_decimal::Decimal;
use std::str::FromStr;

// ── Fixture ───────────────────────────────────────────────────────────────────

/// v3.1 CalcState fixture with `"xrom_modules": 3` (Math 1 + Stat 1, bits 0+1).
///
/// Intentionally OMITS all Time-related fields (time_offset_secs, alarms,
/// stopwatch_*, clock_*, accuracy_factor) — tests that `#[serde(default)]`
/// provides clean zero/false/empty values for all absent Time fields.
///
/// Intentionally INCLUDES `"rand_seed": "0.5"` — tests that the v3.1 rand_seed
/// field survives the v3.2 migration without being reset to zero
/// (STAT-RNG-03 / Pitfall 20).
///
/// Also omits all `#[serde(skip)]` transient fields (modal_program, modal_prompt,
/// integ_state, solve_state, cancel_requested, print_buffer, etc.) since those
/// are never persisted to disk.
static V31_FIXTURE: &str = include_str!("fixtures/v31-autosave.json");

// ── Test 1: Migration activates Time Module bit ───────────────────────────────

/// Verify that a v3.1 save file (`xrom_modules = 3`) migrates correctly to
/// `xrom_modules = 0b0000_0111` (Math 1 + Stat 1 + Time Module, all three bits set).
///
/// Catches: v3.1 → v3.2 upgrade path failure — if `migrate_after_load` is not
/// called or is incorrectly implemented, Time Module ops would return `InvalidOp`
/// on upgraded installations (TIME-FW-01 regression).
#[test]
fn v31_save_loads_with_time_migration() {
    // Deserialize the v3.1 fixture — must succeed without error.
    let mut state: CalcState = serde_json::from_str(V31_FIXTURE).expect("v31 fixture deserializes");

    // Pre-migration: xrom_modules should be 3 (bits 0+1 = Math 1 + Stat 1, as in v3.1).
    // LINT-EXEMPT: integer-equality -- xrom_modules is u8, not HpNum; literal comparison is exact
    assert_eq!(state.xrom_modules, 3u8);

    // Apply the v3.1 → v3.2 migration (sets bit 2 = Time Module).
    state.migrate_after_load();

    // Post-migration: bits 0 (Math 1) + 1 (Stat 1) + 2 (Time Module) must all be set.
    // LINT-EXEMPT: integer-equality -- xrom_modules is u8, not HpNum; the bitmask is exact
    assert_eq!(state.xrom_modules, 0b0000_0111u8);
}

// ── Test 2: Time fields default cleanly when absent ───────────────────────────

/// Verify that all Time Module `CalcState` fields absent from the v3.1 fixture
/// default to their expected zero/false/empty values after deserialization + migration.
///
/// `#[serde(default)]` on each field provides the zero-value when the JSON key
/// is missing; this test confirms the contract is met for each Time field.
///
/// Catches: a future edit accidentally removing `#[serde(default)]` from any
/// Time Module CalcState field — would cause v3.1 save file deserialization to
/// fail with a "missing field" error.
#[test]
fn v31_save_time_fields_default_cleanly() {
    let mut state: CalcState = serde_json::from_str(V31_FIXTURE).expect("v31 fixture deserializes");
    state.migrate_after_load();

    // time_offset_secs defaults to 0 (no clock offset applied).
    // LINT-EXEMPT: integer-equality -- time_offset_secs is i64, not HpNum; exact zero comparison
    assert_eq!(state.time_offset_secs, 0i64);

    // alarms defaults to empty Vec (no alarms set).
    // LINT-EXEMPT: is_empty() check -- no HpNum involved; Vec<AlarmEntry> empty-check
    assert!(state.alarms.is_empty());

    // stopwatch_mode defaults to StopwatchMode::Idle.
    assert_eq!(state.stopwatch_mode, StopwatchMode::Idle);

    // stopwatch_accumulated defaults to 0.0 (no accumulated time).
    // LINT-EXEMPT: float equality -- stopwatch_accumulated is f64 defaulted to 0.0 (exact);
    // no arithmetic involved, just verifying serde default provides a clean zero.
    #[allow(clippy::float_cmp)]
    {
        assert_eq!(state.stopwatch_accumulated, 0.0f64);
    }

    // clock_12h defaults to false (24-hour format).
    assert!(!state.clock_12h);
}

// ── Test 3: rand_seed preserved through migration ─────────────────────────────

/// Verify that `rand_seed = "0.5"` from the v3.1 fixture survives the v3.2 migration.
///
/// The v3.1 `rand_seed` field carries `#[serde(default)]` WITHOUT `#[serde(skip)]` —
/// the unique serde shape that permits it to survive save/load cycles (STAT-RNG-03).
/// This test guards against:
/// 1. `migrate_after_load()` accidentally resetting rand_seed to zero.
/// 2. A future edit adding `#[serde(skip)]` which would silently discard the loaded seed.
///
/// Catches: Pitfall 20 regression on the v3.1→v3.2 migration path — if rand_seed
/// were zeroed by migrate_after_load, a program that called SEED 0.5 in v3.1 would
/// produce different RAND outputs in v3.2 (silent determinism break).
#[test]
fn v31_save_rand_seed_preserved() {
    let mut state: CalcState = serde_json::from_str(V31_FIXTURE).expect("v31 fixture deserializes");
    state.migrate_after_load();

    // rand_seed from fixture ("0.5") must survive migration unchanged.
    // LINT-EXEMPT: exact Decimal equality -- rand_seed is constructed via Decimal::from_str,
    // not f64; Decimal::from_str("0.5") is an exact representation with no float drift.
    assert_eq!(
        state.rand_seed.inner(),
        Decimal::from_str("0.5").unwrap(),
        "rand_seed must survive v3.1 → v3.2 migration unchanged (STAT-RNG-03 / Pitfall 20)"
    );
}
