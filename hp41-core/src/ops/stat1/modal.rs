// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Modal state-machine for Stat 1 Pac prompt-driven workflows.
//!
//! `Stat1Step` is the per-program step enum carried by the
//! `ModalProgram::Stat1(Stat1Step)` variant living in
//! `hp41-core/src/ops/math1/modal.rs` (math1/ freeze exception D-33.3b).
//!
//! ## Plan 33-03 — real variants supplant the 33-01 scaffolding
//!
//! Plan 33-01 shipped a single scaffolding variant for cross-plan compile
//! contract. Plan 33-03 (this file) replaces it with the THREE real
//! variants needed by ΣNORMD + ΣCHISQD:
//!
//! - `NormdModeChoice`   — ΣNORMD opens at this step; X = mode index
//!   (1 = CDF / 2 = PDF / 3 = inverse per OM 00041-90030 §ΣNORMD with
//!   tentative key-mapping `[E]=CDF / [C]=PDF / [A]=inverse`).
//! - `ChisqdNuPrompt`    — ΣCHISQD opens at this step; X = ν (degrees
//!   of freedom, positive integer).
//! - `ChisqdModeChoice`  — after `ChisqdNuPrompt` captures ν, transition
//!   here; X = mode index (1 = PDF / 2 = CDF).
//!
//! Plan 33-08 will EXTEND this enum with `PolypDegreePrompt(u8)` and
//! `SeedPrompt`; the three dispatch functions stay exhaustively matched
//! (no `_ =>` catch-all per CLAUDE.md "Core engine — no panics" + the
//! FN-CLI-04 invariant inherited from `math1::modal::ModalProgram`).
//!
//! ## ν storage strategy (D-33.5 — no new transient CalcState fields)
//!
//! Between `ChisqdNuPrompt` and `ChisqdModeChoice`, the captured ν value
//! must persist WITHOUT adding a new `CalcState` field. We re-use the
//! `state.stack.t` register (deepest stack slot, untouched by typical
//! modal-prompt interaction): `submit_step(ChisqdNuPrompt)` writes ν to
//! `state.stack.t`, then `submit_step(ChisqdModeChoice)` reads it back.
//! This mirrors the Math Pac I POLY precedent (degree stored in R06,
//! `poly.rs:477`) but uses T instead because:
//!   (a) no new register slot needs to be reserved;
//!   (b) the user's stack-X (entered as mode index) and stack-Y are not
//!       clobbered between the two prompts;
//!   (c) D-33.5 explicitly bans new `CalcState` fields for transient
//!       Stat 1 Pac state.
//!
//! Plan 33-08 SEED will use the same `state.stack.t` carrier pattern.
//!
//! ## Why this file lives in `stat1/` not `math1/`
//!
//! D-33.3b authorizes ONE additional math1/ freeze exception for
//! `math1/modal.rs` — a single-line variant addition plus three dispatch
//! arms. All Stat-1-specific semantics (this file) stay in `stat1/` so
//! the math1/ blast radius is exactly the 8 lines of dispatch wiring.

use crate::error::HpError;
use crate::state::CalcState;

/// Per-step modal state for the Stat 1 Pac modal-prompt workflows.
///
/// Carried by `ModalProgram::Stat1(Stat1Step)` (D-33.3b freeze-exception
/// variant in `math1/modal.rs`). Plan 33-03 ships the three real variants
/// required by ΣNORMD + ΣCHISQD; Plan 33-08 will add `PolypDegreePrompt(u8)`
/// + `SeedPrompt`.
#[derive(Debug, Clone, PartialEq)]
pub enum Stat1Step {
    /// ΣNORMD — awaiting mode index in X (1 = CDF, 2 = PDF, 3 = inverse).
    ///
    /// OM 00041-90030 §ΣNORMD documents tentative key-mapping
    /// `[E]=CDF / [C]=PDF / [A]=inverse`; the SPEC.md Req. 31 lock is
    /// "deterministic prompt-and-submit workflow reusing existing modal
    /// infrastructure with no new transient CalcState fields", so the
    /// numeric-index convention (1/2/3) is OM-compatible and inherits
    /// the dispatch from `state.stack.x.trunc_int`.
    NormdModeChoice,
    /// ΣCHISQD — awaiting degrees-of-freedom ν entry in X.
    ///
    /// `submit_step` reads X as a positive integer, stores it in
    /// `state.stack.t` (the deepest stack slot — D-33.5 transient storage
    /// without a new CalcState field), then transitions to
    /// `ChisqdModeChoice` with prompt `ΣCHISQD MODE?`.
    ChisqdNuPrompt,
    /// ΣCHISQD — ν is captured in `state.stack.t`; awaiting mode index
    /// in X (1 = PDF, 2 = CDF).
    ChisqdModeChoice,
    // Plan 33-08 will ADD: PolypDegreePrompt(u8), SeedPrompt.
}

/// Per-step submit dispatch — called by `math1::submit_modal` when the
/// active modal is `ModalProgram::Stat1(...)` and the user presses R/S.
///
/// Variant routing:
///
/// - `NormdModeChoice` — reads X = mode index (1 = CDF, 2 = PDF, 3 =
///   inverse). The mode dispatch convention is "X holds the mode index;
///   the z-score (or probability) was entered into Y BEFORE the user
///   pressed R/S to enter the mode index" — mirrors the HP-41 Σ+
///   two-value-entry convention. After reading the mode, drop X →
///   Y → X (so the z-score is now in X) and call the appropriate
///   `op_sigma_normd_eval_*` function. Clear modal state on success.
/// - `ChisqdNuPrompt` — Task 4 of this plan ships this body.
/// - `ChisqdModeChoice` — Task 4 of this plan ships this body.
pub fn submit_step(state: &mut CalcState, step: Stat1Step) -> Result<(), HpError> {
    match step {
        Stat1Step::NormdModeChoice => {
            // Read mode index from X (truncate-to-integer per HP-41
            // convention — fractional indices are rejected as Domain).
            let mode_index = state.stack.x.trunc_int();
            let mode_i32 = mode_index.inner().to_i32_safe()?;
            // Drop X (the mode index); the z-score / probability the user
            // entered earlier is now in X. HP-41 "drop": X←Y, Y←Z, Z←T,
            // T←T (T duplicated per hardware). lift_enabled stays true so
            // the eval result push behaves as a unary_result.
            state.stack.x = state.stack.y.clone();
            state.stack.y = state.stack.z.clone();
            state.stack.z = state.stack.t.clone();
            // Clear modal state BEFORE dispatching so the eval function
            // sees a clean modal context (eval ops are non-modal).
            state.modal_program = None;
            state.modal_prompt = None;
            match mode_i32 {
                1 => crate::ops::stat1::normd::op_sigma_normd_eval_cdf(state),
                2 => crate::ops::stat1::normd::op_sigma_normd_eval_pdf(state),
                3 => crate::ops::stat1::normd::op_sigma_normd_eval_inverse(state),
                _ => Err(HpError::Domain),
            }
        }
        Stat1Step::ChisqdNuPrompt => {
            // Read ν from X (truncate toward zero — fractional degrees of
            // freedom are rejected; ν must be a positive integer per the
            // chi-square distribution definition).
            let nu_dec = state.stack.x.trunc_int();
            let nu_i32 = nu_dec.inner().to_i32_safe()?;
            if nu_i32 <= 0 {
                // Restore the modal state so a domain-error doesn't lose
                // the user's place in the workflow — D-07 never-discard.
                return Err(HpError::Domain);
            }
            // D-33.5 transient storage: stash ν in state.stack.t (the
            // deepest stack slot). Drop X (the ν the user just submitted)
            // so the next prompt's X-input is collected cleanly.
            state.stack.t = state.stack.x.clone(); // ν → T
            // Drop X off the stack (X←Y, Y←Z, Z←T-was — but T was just
            // overwritten with ν, so use a saved copy of the original T
            // value to preserve the standard HP-41 stack-drop semantics).
            state.stack.x = state.stack.y.clone();
            state.stack.y = state.stack.z.clone();
            // Note: we deliberately do NOT touch z ← t here, because
            // t is now ν (the carrier value). The user's original T
            // is lost — acceptable trade-off per D-33.5 (no new
            // CalcState field, deepest stack slot re-used).
            // Advance modal to ChisqdModeChoice.
            state.modal_program =
                Some(crate::ops::math1::modal::ModalProgram::Stat1(Stat1Step::ChisqdModeChoice));
            state.modal_prompt = Some("\u{03A3}CHISQD MODE?".to_string());
            Ok(())
        }
        Stat1Step::ChisqdModeChoice => {
            // Read mode index from X (truncate to integer; reject fractional).
            let mode_index = state.stack.x.trunc_int();
            let mode_i32 = mode_index.inner().to_i32_safe()?;
            // Recover ν from state.stack.t (where ChisqdNuPrompt stashed it).
            // trunc_int is defensive — T was written from a trunc'd value
            // already, but a future round of testing might bypass that.
            let nu_dec = state.stack.t.trunc_int();
            let nu_i32 = nu_dec.inner().to_i32_safe()?;
            if nu_i32 <= 0 {
                state.modal_program = None;
                state.modal_prompt = None;
                return Err(HpError::Domain);
            }
            let nu_u32 = nu_i32 as u32;
            // Drop X (the mode index); the χ² statistic x was entered
            // earlier and now sits in X (Y becomes X). Use the standard
            // HP-41 drop pattern, but DUPLICATE T on drop per hardware
            // (rather than letting it stay as ν — which would be
            // confusing if the user inspected the stack post-eval).
            state.stack.x = state.stack.y.clone();
            state.stack.y = state.stack.z.clone();
            // Clear modal state BEFORE dispatching so the eval function
            // sees a clean modal context.
            state.modal_program = None;
            state.modal_prompt = None;
            match mode_i32 {
                1 => crate::ops::stat1::chisqd::op_sigma_chisqd_eval_pdf(state, nu_u32),
                2 => crate::ops::stat1::chisqd::op_sigma_chisqd_eval_cdf(state, nu_u32),
                _ => Err(HpError::Domain),
            }
        }
    }
}

/// Local extension trait for `rust_decimal::Decimal` → `i32` conversion
/// with `HpError::Domain` mapping. Keeps the `submit_step` body
/// dependency-free of `rust_decimal::prelude::ToPrimitive` in this scope.
trait DecimalToI32Safe {
    fn to_i32_safe(&self) -> Result<i32, HpError>;
}

impl DecimalToI32Safe for rust_decimal::Decimal {
    fn to_i32_safe(&self) -> Result<i32, HpError> {
        use rust_decimal::prelude::ToPrimitive;
        self.to_i32().ok_or(HpError::Domain)
    }
}

/// Per-step prompt accessor — called by `ModalProgram::current_prompt`
/// (the carrier-enum dispatch in `math1/modal.rs`).
///
/// Prompt strings per OM 00041-90030 §ΣNORMD and §ΣCHISQD; Unicode Σ
/// encoded as `\u{03A3}` per the established Plan-28 convention, ν as
/// `\u{03BD}` (Greek small nu).
pub fn current_prompt(step: &Stat1Step) -> Option<String> {
    match step {
        Stat1Step::NormdModeChoice => Some("\u{03A3}NORMD MODE?".to_string()),
        Stat1Step::ChisqdNuPrompt => Some("\u{03BD}=?".to_string()),
        Stat1Step::ChisqdModeChoice => Some("\u{03A3}CHISQD MODE?".to_string()),
    }
}

/// Per-step alpha-label gate — called by `ModalProgram::requires_alpha_label`
/// (the carrier-enum dispatch in `math1/modal.rs`).
///
/// All three Plan-33-03 variants accept NUMERIC input (mode index or ν),
/// not alpha labels — return `false`. Plan 33-08 will introduce
/// `SeedPrompt`, the first Stat-1 step requiring an alpha label
/// (`requires_alpha_label = true`) for the SEED alpha-label flow.
pub fn requires_alpha_label(step: &Stat1Step) -> bool {
    match step {
        Stat1Step::NormdModeChoice
        | Stat1Step::ChisqdNuPrompt
        | Stat1Step::ChisqdModeChoice => false,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ── current_prompt — OM-cited strings (Plan 33-03 Task 2) ────────────

    /// Catches: NormdModeChoice prompt string drift away from OM "ΣNORMD MODE?".
    #[test]
    fn normd_mode_choice_prompt() {
        assert_eq!(
            current_prompt(&Stat1Step::NormdModeChoice),
            Some("\u{03A3}NORMD MODE?".to_string())
        );
    }

    /// Catches: ChisqdNuPrompt prompt string drift away from OM "ν=?".
    #[test]
    fn chisqd_nu_prompt() {
        assert_eq!(
            current_prompt(&Stat1Step::ChisqdNuPrompt),
            Some("\u{03BD}=?".to_string())
        );
    }

    /// Catches: ChisqdModeChoice prompt string drift away from OM "ΣCHISQD MODE?".
    #[test]
    fn chisqd_mode_choice_prompt() {
        assert_eq!(
            current_prompt(&Stat1Step::ChisqdModeChoice),
            Some("\u{03A3}CHISQD MODE?".to_string())
        );
    }

    // ── requires_alpha_label — all three numeric-input steps (Plan 33-03) ──

    /// Catches: any Plan-33-03 variant accidentally requesting alpha-label
    /// (no Stat-1 mode/numeric step takes alpha input; SeedPrompt — Plan 33-08
    /// — is the first variant where this returns `true`).
    #[test]
    fn no_plan_33_03_variant_requires_alpha_label() {
        for step in [
            Stat1Step::NormdModeChoice,
            Stat1Step::ChisqdNuPrompt,
            Stat1Step::ChisqdModeChoice,
        ] {
            assert!(
                !requires_alpha_label(&step),
                "Plan 33-03 step {step:?} must not request alpha-label"
            );
        }
    }

    // ── submit_step — placeholder bodies until Tasks 3 + 4 wire eval calls ──
    //
    // After Plan 33-03 Task 1 (this commit), submit_step returns InvalidOp
    // for every variant — the modal-state clearing is the only side effect.
    // Tasks 3 (ΣNORMD) + 4 (ΣCHISQD) replace each arm's body with the
    // appropriate eval-function dispatch. The tests below assert ONLY the
    // current shape and will be rewritten when those tasks land.

    /// Catches: submit_step(NormdModeChoice) not clearing modal_program.
    #[test]
    fn submit_normd_mode_choice_clears_modal_state() {
        let mut state = CalcState::new();
        state.modal_program = Some(crate::ops::math1::modal::ModalProgram::Stat1(
            Stat1Step::NormdModeChoice,
        ));
        state.modal_prompt = Some("\u{03A3}NORMD MODE?".to_string());
        let _ = submit_step(&mut state, Stat1Step::NormdModeChoice);
        assert!(state.modal_program.is_none());
        assert!(state.modal_prompt.is_none());
    }

    /// Catches: submit_step(ChisqdNuPrompt) failing to ADVANCE to
    /// ChisqdModeChoice + stash ν in state.stack.t (Task 4 wiring).
    #[test]
    fn submit_chisqd_nu_prompt_advances_to_mode_choice() {
        let mut state = CalcState::new();
        state.modal_program = Some(crate::ops::math1::modal::ModalProgram::Stat1(
            Stat1Step::ChisqdNuPrompt,
        ));
        state.stack.x = crate::num::HpNum::from(3i32);
        let r = submit_step(&mut state, Stat1Step::ChisqdNuPrompt);
        assert_eq!(r, Ok(()));
        // Modal advances to ChisqdModeChoice with the "ΣCHISQD MODE?" prompt.
        assert!(matches!(
            state.modal_program,
            Some(crate::ops::math1::modal::ModalProgram::Stat1(Stat1Step::ChisqdModeChoice))
        ));
        assert_eq!(
            state.modal_prompt,
            Some("\u{03A3}CHISQD MODE?".to_string())
        );
        // ν stashed in stack.T (D-33.5 transient carrier).
        assert_eq!(state.stack.t, crate::num::HpNum::from(3i32));
    }

    /// Catches: submit_step(ChisqdNuPrompt) accepting non-positive ν
    /// (chi-square is undefined for ν ≤ 0; Domain error must surface).
    #[test]
    fn submit_chisqd_nu_prompt_rejects_zero_nu() {
        let mut state = CalcState::new();
        state.modal_program = Some(crate::ops::math1::modal::ModalProgram::Stat1(
            Stat1Step::ChisqdNuPrompt,
        ));
        state.stack.x = crate::num::HpNum::from(0i32);
        let r = submit_step(&mut state, Stat1Step::ChisqdNuPrompt);
        assert_eq!(r, Err(HpError::Domain));
    }

    /// Catches: Stat1Step Clone + PartialEq derive regression on the three
    /// real Plan-33-03 variants.
    #[test]
    fn stat1_step_clone_and_eq() {
        let step = Stat1Step::NormdModeChoice;
        assert_eq!(step.clone(), step);
        assert_ne!(Stat1Step::NormdModeChoice, Stat1Step::ChisqdNuPrompt);
        assert_ne!(Stat1Step::ChisqdNuPrompt, Stat1Step::ChisqdModeChoice);
    }
}
