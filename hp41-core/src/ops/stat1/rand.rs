// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `stat1::rand` — RAND / SEED pseudorandom-number Ops (Plan 33-08).
//!
//! Per D-33.4 + SPEC.md Req. 38 these ship as a v3.1 emulator extension
//! (HP-41C Stat 1 Pac OM 00041-90030 / QRC 00041-90061 do NOT enumerate
//! a `RAND` or `SEED` top-level entry — the formula is community-LCG
//! sourced from NPS55-84-003 p. 21 (Zehna 1984), attributed to Don Malm
//! / HP-65 User's Library, and corroborated by HP-41C Standard
//! Applications p. 24). Phase 35 (`docs/hp41-stat1-divergences.md`)
//! records the emulator-extension status.
//!
//! **LCG formula** (SPEC.md Req. 35):
//!
//! ```text
//!   r_{n+1} = FRC(9821 · r_n + 0.211327)
//! ```
//!
//! All arithmetic in `HpNum` (rust_decimal) — `9821` and `0.211327`
//! are exact decimals, so the sequence is deterministic at the
//! 10-significant-digit precision floor. `FRC(x) = x − trunc(x)` uses
//! the existing [`crate::num::HpNum::trunc_int`] helper.
//!
//! **`state.rand_seed`** carries the LCG state. Plan 33-01 declared it
//! with `#[serde(default)]` WITHOUT `#[serde(skip)]` — the SOLE v3.1
//! `CalcState` field with that serde shape (other transient v3.1
//! fields use `skip`). This makes the seed survive save/load
//! round-trips (SPEC.md Req. 37, integration test in
//! `tests/stat1_rand_determinism.rs`).
//!
//! **SEED** opens a modal `SEED?` prompt; on submit, copies the
//! current stack-X value into `state.rand_seed`.

use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;
use rust_decimal::Decimal;

/// RAND — step the LCG and push the new fractional seed to stack X.
///
/// One iteration: `r ← FRC(9821 · r + 0.211327)`. Updates
/// `state.rand_seed` in place (so subsequent RAND calls continue the
/// sequence) AND pushes the new value to X. LiftEffect::Enable.
///
/// Errors: propagated arithmetic overflow (theoretically impossible
/// for the canonical LCG; defensive `?`-propagation only).
///
/// SPEC.md Req. 35 oracle: with `state.rand_seed = 0.5`, returns
/// `FRC(9821·0.5 + 0.211327) = FRC(4910.711327) = 0.711327` exactly.
///
/// Source: NPS55-84-003 (Zehna 1984) p. 21 community-LCG; not in OM.
pub fn op_rand(state: &mut CalcState) -> Result<(), HpError> {
    // LCG constants as exact decimals (NO f64 conversion — SPEC.md
    // Req. 35 acceptance asserts decimal-exact HpNum equality).
    let multiplier = HpNum::from(9821i32);
    let increment = HpNum::from(Decimal::new(211327, 6)); // 0.211327
    let stepped = state
        .rand_seed
        .checked_mul(&multiplier)?
        .checked_add(&increment)?;
    // FRC(x) = x − trunc(x). HpNum::trunc_int truncates toward zero
    // per rust_decimal::Decimal::trunc.
    let int_part = stepped.trunc_int();
    let new_seed = stepped.checked_sub(&int_part)?;
    // Write-back BEFORE the push so a failed enter_number/apply_lift
    // path still updates the LCG state (defensive; both calls are
    // infallible in practice but the ordering keeps the semantics
    // clear at the cost of zero perf).
    state.rand_seed = new_seed.clone();
    state.stack.lift_enabled = true;
    enter_number(state, new_seed);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

/// SEED — open the `SEED?` modal; on submit, copies stack-X into
/// `state.rand_seed`. LiftEffect::Neutral on open. The actual seed
/// write happens in `stat1::modal::submit_step(SeedPrompt)`.
///
/// SPEC.md Req. 36 oracle: `XEQ "SEED"` opens prompt; user enters
/// 0.5 and presses R/S; `state.rand_seed` is now 0.5.
///
/// Source: emulator extension (D-33.4); not in OM.
pub fn op_seed(state: &mut CalcState) -> Result<(), HpError> {
    state.modal_program = Some(crate::ops::math1::modal::ModalProgram::Stat1(
        crate::ops::stat1::modal::Stat1Step::SeedPrompt,
    ));
    state.modal_prompt = Some("SEED?".to_string());
    crate::stack::apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::num::HpNum;
    use rust_decimal::prelude::ToPrimitive;
    use rust_decimal::Decimal;

    /// Helper: convert HpNum to f64 for relative comparisons.
    fn as_f64(n: &HpNum) -> f64 {
        n.inner().to_f64().expect("HpNum→f64 must succeed")
    }

    /// SPEC.md Req. 35 first-iteration oracle (decimal-exact):
    /// seed=0.5 → FRC(9821·0.5 + 0.211327) = FRC(4910.711327) = 0.711327.
    #[test]
    fn rand_lcg_formula_first_iter() {
        let mut state = CalcState::new();
        state.rand_seed = HpNum::from(Decimal::new(5, 1)); // 0.5
        op_rand(&mut state).expect("RAND must succeed");
        let expected = HpNum::from(Decimal::new(711327, 6)); // 0.711327
        assert_eq!(
            state.rand_seed, expected,
            "rand_seed mismatch: got {:?}, expected {:?}",
            state.rand_seed, expected
        );
        // Push lands on X.
        assert_eq!(state.stack.x, expected);
    }

    /// Determinism: identical seeds → identical sequences.
    #[test]
    fn rand_deterministic_sequence() {
        let seed = HpNum::from(Decimal::new(7, 1)); // 0.7
        let mut s1 = CalcState::new();
        let mut s2 = CalcState::new();
        s1.rand_seed = seed.clone();
        s2.rand_seed = seed;
        let mut seq1 = Vec::new();
        let mut seq2 = Vec::new();
        for _ in 0..5 {
            op_rand(&mut s1).unwrap();
            op_rand(&mut s2).unwrap();
            seq1.push(s1.rand_seed.clone());
            seq2.push(s2.rand_seed.clone());
        }
        assert_eq!(seq1, seq2, "identical seeds must yield identical RAND sequences");
    }

    /// SEED modal opener sets the carrier-enum variant and prompt.
    #[test]
    fn seed_opens_modal() {
        let mut state = CalcState::new();
        op_seed(&mut state).expect("SEED open must succeed");
        assert!(matches!(
            state.modal_program,
            Some(crate::ops::math1::modal::ModalProgram::Stat1(
                crate::ops::stat1::modal::Stat1Step::SeedPrompt
            ))
        ));
        assert_eq!(state.modal_prompt, Some("SEED?".to_string()));
    }

    /// SEED submit (via submit_step) copies stack-X into state.rand_seed.
    #[test]
    fn seed_submit_writes_rand_seed() {
        let mut state = CalcState::new();
        op_seed(&mut state).unwrap();
        let seed_val = HpNum::from(Decimal::new(3, 1)); // 0.3
        state.stack.x = seed_val.clone();
        crate::ops::stat1::modal::submit_step(
            &mut state,
            crate::ops::stat1::modal::Stat1Step::SeedPrompt,
        )
        .expect("SEED submit must succeed");
        assert_eq!(state.rand_seed, seed_val);
        assert!(state.modal_program.is_none());
        assert!(state.modal_prompt.is_none());
    }

    /// Output range: each RAND output is in [0, 1) — fractional part
    /// of an LCG step by construction.
    #[test]
    fn rand_output_in_unit_interval() {
        let mut state = CalcState::new();
        state.rand_seed = HpNum::from(Decimal::new(123_456, 6)); // 0.123456
        for _ in 0..20 {
            op_rand(&mut state).unwrap();
            let v = as_f64(&state.rand_seed);
            assert!(
                (0.0..1.0).contains(&v),
                "rand_seed must stay in [0, 1), got {v}"
            );
        }
    }
}
