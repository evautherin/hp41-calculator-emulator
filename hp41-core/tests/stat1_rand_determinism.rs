// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Wave-0 integration test: RAND determinism + SEED round-trip + serde
//! save/load survival of `state.rand_seed` (Plan 33-08 / SPEC.md Req. 35-37).
//!
//! These are integration-level tests rather than unit tests because:
//!   1. they exercise the full `hp41_core` public surface (dispatch path);
//!   2. they round-trip `CalcState` through serde to verify SPEC.md
//!      Req. 37 (the only v3.1 `#[serde(default)]` WITHOUT `skip` field —
//!      `rand_seed` — must survive save/load unchanged).
//!
//! Reference: NPS55-84-003 (Zehna 1984) p. 21 community-LCG (Don Malm /
//! HP-65 User's Library origin) — `r ← FRC(9821·r + 0.211327)`.

use hp41_core::ops::dispatch;
use hp41_core::ops::Op;
use hp41_core::{CalcState, HpNum};
use rust_decimal::Decimal;

/// Helper: construct an `HpNum` from `(mantissa, scale)` exactly without
/// f64 conversion. `(5, 1)` = 0.5; `(711327, 6)` = 0.711327.
fn hp(mantissa: i64, scale: u32) -> HpNum {
    HpNum::from(Decimal::new(mantissa, scale))
}

/// SPEC.md Req. 35 (decimal-exact first iteration):
///   seed = 0.5 → FRC(9821·0.5 + 0.211327) = FRC(4910.711327) = 0.711327.
///
/// HpNum equality (no tolerance) — the LCG body must run on
/// rust_decimal (NOT f64) so the result is byte-for-byte exact.
#[test]
fn rand_lcg_formula_first_iter() {
    let mut state = CalcState::new();
    state.rand_seed = hp(5, 1); // 0.5
    dispatch(&mut state, Op::Rand).expect("RAND must succeed");
    let expected = hp(711_327, 6); // 0.711327
    assert_eq!(
        state.rand_seed, expected,
        "rand_seed mismatch: SPEC Req. 35 decimal-exact LCG"
    );
    // Push lands on stack X.
    assert_eq!(state.stack.x, expected);
}

/// SPEC.md Req. 37: `rand_seed` survives serde save/load AND the
/// subsequent RAND sequence is byte-for-byte identical to running
/// from the same seed without the round-trip.
///
/// This is the empirical proof that the Plan 33-01 serde shape
/// (`#[serde(default)]` WITHOUT `skip`) is correct — the Plan 33-01
/// lib test `rand_seed_serde_round_trip` catches a single-field
/// regression; this test catches the EFFECT of any regression
/// (sequence divergence) end-to-end.
#[test]
fn rand_sequence_deterministic_after_save_load() {
    let mut state_a = CalcState::new();
    state_a.rand_seed = hp(7, 1); // 0.7

    // Capture three RAND outputs from state_a.
    let mut seq_a = Vec::with_capacity(3);
    for _ in 0..3 {
        dispatch(&mut state_a, Op::Rand).expect("RAND must succeed");
        seq_a.push(state_a.rand_seed.clone());
    }

    // Fresh state_b: same seed, round-tripped through serde.
    let mut state_b = CalcState::new();
    state_b.rand_seed = hp(7, 1); // 0.7
    let json = serde_json::to_string(&state_b).expect("CalcState must serialize");
    let mut state_b: CalcState = serde_json::from_str(&json).expect("CalcState must deserialize");
    // Sanity: the round-trip preserved rand_seed.
    assert_eq!(
        state_b.rand_seed,
        hp(7, 1),
        "rand_seed must survive serde round-trip unchanged"
    );

    // Capture three RAND outputs from state_b (post-roundtrip).
    let mut seq_b = Vec::with_capacity(3);
    for _ in 0..3 {
        dispatch(&mut state_b, Op::Rand).expect("RAND must succeed");
        seq_b.push(state_b.rand_seed.clone());
    }

    // SEQUENCES MUST MATCH EXACTLY — no tolerance, decimal-exact HpNum.
    assert_eq!(
        seq_a, seq_b,
        "RAND sequences from identical seeds must be byte-for-byte identical \
         across a serde round-trip (SPEC.md Req. 35 + Req. 37)"
    );
}

/// SPEC.md Req. 36: `XEQ "SEED"` opens the `SEED?` modal; on R/S
/// submit with value 0.5 on X, `state.rand_seed` is set to 0.5.
/// Subsequent RAND produces the canonical SPEC Req. 35 oracle.
#[test]
fn seed_modal_round_trip() {
    let mut state = CalcState::new();
    // 1. XEQ "SEED" opens the modal.
    dispatch(&mut state, Op::Seed).expect("SEED open must succeed");
    assert_eq!(state.modal_prompt, Some("SEED?".to_string()));
    assert!(state.modal_program.is_some());

    // 2. User enters 0.5 on stack X and submits via modal submit_step.
    state.stack.x = hp(5, 1); // 0.5
    hp41_core::ops::stat1::modal::submit_step(
        &mut state,
        hp41_core::ops::stat1::modal::Stat1Step::SeedPrompt,
    )
    .expect("SEED submit must succeed");

    // Modal cleared; rand_seed updated.
    assert!(state.modal_program.is_none());
    assert!(state.modal_prompt.is_none());
    assert_eq!(state.rand_seed, hp(5, 1));

    // 3. RAND produces the Req. 35 oracle: FRC(9821·0.5 + 0.211327) = 0.711327.
    dispatch(&mut state, Op::Rand).expect("RAND must succeed");
    assert_eq!(state.rand_seed, hp(711_327, 6));
}
