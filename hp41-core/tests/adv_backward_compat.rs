// Algorithm independently re-derived from HP Advantage Pac Owner's Manual;
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! v3.2 save-file backward compatibility + `migrate_after_load` migration tests.
//!
//! Four tests verify the v3.2 → v3.3 save-file migration contract:
//!
//! 1. `v32_save_loads_with_advantage_migration` — a v3.2 fixture with
//!    `"xrom_modules": 7` (Math 1 + Stat 1 + Time, bits 0+1+2) deserializes
//!    and `migrate_after_load` sets bits 3+4 (Advantage Pac modules).
//!
//! 2. `v32_save_advantage_fields_default_cleanly` — all Advantage Pac CalcState
//!    fields absent from the v3.2 fixture default via `#[serde(default)]`.
//!
//! 3. `v32_save_time_fields_preserved` — Time fields present in the v3.2 fixture
//!    survive the v3.3 migration unchanged.
//!
//! 4. `v32_save_rand_seed_preserved` — `rand_seed` present in the v3.2 fixture
//!    ("0.5") survives the v3.3 migration unchanged (ADR-v3.1-001 / Pitfall 20:
//!    `rand_seed` uses `#[serde(default)]` WITHOUT `#[serde(skip)]`).

#![allow(clippy::unwrap_used)]

use hp41_core::state::CalcState;
use rust_decimal::Decimal;
use std::str::FromStr;

static V32_FIXTURE: &str = include_str!("fixtures/v32-autosave.json");

#[test]
fn v32_save_loads_with_advantage_migration() {
    let mut state: CalcState = serde_json::from_str(V32_FIXTURE).expect("v32 fixture deserializes");

    assert_eq!(state.xrom_modules, 7u8);

    state.migrate_after_load();

    assert_eq!(state.xrom_modules, 0b0001_1111u8);
}

#[test]
fn v32_save_advantage_fields_default_cleanly() {
    let mut state: CalcState = serde_json::from_str(V32_FIXTURE).expect("v32 fixture deserializes");
    state.migrate_after_load();

    assert!(state.adv_matrices.is_empty());
    assert_eq!(state.adv_matrix_i, 0u8);
    assert_eq!(state.adv_matrix_j, 0u8);
    assert!(state.adv_tvm_state.is_none());
}

#[test]
fn v32_save_time_fields_preserved() {
    let mut state: CalcState = serde_json::from_str(V32_FIXTURE).expect("v32 fixture deserializes");
    state.migrate_after_load();

    assert_eq!(state.time_offset_secs, 3600i64);
    assert!(state.clock_12h);
}

/// Verify that `rand_seed` serialized in a v3.2 save file is preserved across
/// the v3.3 migration. `rand_seed` uses `#[serde(default)]` WITHOUT
/// `#[serde(skip)]` per ADR-v3.1-001 / Pitfall 20 — this test is the
/// forward-compat guard that catches accidental `#[serde(skip)]` regressions.
///
/// The v3.2 fixture contains `"rand_seed": "0.5"` — must survive migration
/// unchanged.
#[test]
fn v32_save_rand_seed_preserved() {
    let mut state: CalcState = serde_json::from_str(V32_FIXTURE).expect("v32 fixture deserializes");
    state.migrate_after_load();

    let expected = Decimal::from_str("0.5").expect("literal 0.5 parses");
    // LINT-EXEMPT: serde round-trip Decimal equality; "0.5" is exact in BCD
    assert_eq!(state.rand_seed.inner(), expected, "rand_seed must survive v3.2 → v3.3 migration");
}
