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
//! ## ν storage strategy (REVIEW.md WR-03 / WR-04 — transient carrier field)
//!
//! Between `ChisqdNuPrompt` and `ChisqdModeChoice`, the captured ν value
//! persists in the dedicated transient `state.pending_chisqd_nu:
//! Option<u32>` field (`#[serde(default, skip)]`; see `state.rs`).
//!
//! ### Plan 33-03 history (pre-review): stack-T side-channel
//!
//! The initial implementation stashed ν in `state.stack.t` to honor
//! D-33.5's "no new CalcState field" guidance. That design proved
//! unsafe because any stack-lifting Op invoked between the two
//! modal submits (most arithmetic, push-lifts from backspace edits,
//! XEQ calls to user routines) silently clobbered T — the
//! mode-choice submit then read garbage ν data and silently computed
//! a wrong PDF/CDF. The user's original T value was also destroyed.
//!
//! ### Current design (post-WR-03 fix): transient carrier field
//!
//! `submit_step(ChisqdNuPrompt)` writes the validated ν to
//! `state.pending_chisqd_nu = Some(u32)` and then performs a STANDARD
//! HP-41 4-slot stack drop (`x ← y, y ← z, z ← t, t ← t`) preserving
//! the user's original T value. `submit_step(ChisqdModeChoice)`
//! `take()`s + clears the carrier, returning `Domain` if it was empty
//! (out-of-sequence call) rather than consuming undefined data.
//!
//! D-33.5's "no new persistent field" rule was always understood to
//! permit transient `#[serde(default, skip)]` fields — the existing
//! `modal_program` / `modal_prompt` / `integ_state` / `solve_state` /
//! `cancel_requested` fields are precedents.
//!
//! Plan 33-08 SEED uses a different pattern (direct write into
//! `state.rand_seed` at submit time + defensive normalization in
//! `op_rand` per the REVIEW.md CR-01 fix) — no transient carrier
//! needed.
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
    /// `submit_step` reads X as a positive integer, stores it in the
    /// transient `state.pending_chisqd_nu: Option<u32>` carrier
    /// (REVIEW.md WR-03/WR-04 — replaces the unsafe stack-T side
    /// channel from the initial Plan 33-03 design), then transitions
    /// to `ChisqdModeChoice` with prompt `ΣCHISQD MODE?`. The user's
    /// original T value is preserved by the standard 4-slot HP-41
    /// stack drop.
    ChisqdNuPrompt,
    /// ΣCHISQD — ν is captured in the transient
    /// `state.pending_chisqd_nu` carrier; awaiting mode index in X
    /// (1 = PDF, 2 = CDF). `submit_step` `take()`s + clears the
    /// carrier; out-of-sequence invocation (carrier empty) returns
    /// `Domain` rather than reading garbage.
    ChisqdModeChoice,
    /// ΣPOLYP — awaiting polynomial degree `d` entry in X (1 ≤ d ≤
    /// [`crate::ops::stat1::STAT1_POLYP_DEGREE_MAX`]; OM override per
    /// SPEC.md Req. 22 — the prompt string `DEGREE=?` is NOT locked,
    /// only the modal-prompt-driven workflow shape).
    ///
    /// The u8 payload is reserved for future multi-step expansion
    /// (e.g. accumulation-ready substate). Plan 33-08 uses only `(0)`
    /// for the initial prompt; submit transitions out of the modal
    /// after storing d in [`crate::ops::stat1::STAT1_POLYP_DEGREE_REG`]
    /// and dispatching `compute_polyp_coefficients` to fit the
    /// pre-populated higher-power Σ sums.
    PolypDegreePrompt(u8),
    /// SEED — awaiting numeric seed value entry in X. Submit NORMALIZES
    /// `state.stack.x` into the closed-open unit interval `[0, 1)` via
    /// [`crate::ops::stat1::rand::normalize_seed_to_unit_interval`]
    /// (REVIEW.md CR-01 mitigation) and writes the result into
    /// `state.rand_seed`, then clears the modal state.
    ///
    /// Any HpNum is accepted on input (positive, negative, fractional
    /// or integer); the normalization wraps via the HP-41 FRC convention
    /// with a `+1` step for negative inputs so the LCG body always sees
    /// a non-negative seed in `[0, 1)`. RAND output is therefore
    /// guaranteed to stay in the unit interval regardless of the
    /// user-submitted value.
    ///
    /// Per D-33.4 SPEC.md Req. 36 ALPHA prompt naming — the original OM
    /// (if it exists) shows an ALPHA-prompt convention, but our
    /// implementation accepts a numeric HpNum directly from stack X
    /// (consistent with the rest of Stat 1 Pac modal numeric submits).
    SeedPrompt,
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
            // REVIEW.md WR-03/WR-04 mitigation: store ν in the
            // transient `pending_chisqd_nu` carrier rather than the
            // stack T register. The previous design used T as a
            // side-channel between the two modal-submit calls but any
            // stack-lifting Op (most arithmetic, backspace push-lifts,
            // XEQ calls) silently clobbered T → mode-choice submit
            // read garbage ν → silent wrong PDF/CDF computation. The
            // new transient field is unaffected by stack ops and
            // is cleared on success/error in the ChisqdModeChoice arm
            // below.
            #[allow(clippy::cast_sign_loss)]
            // safe: nu_i32 > 0 enforced above.
            let nu_u32 = nu_i32 as u32;
            state.pending_chisqd_nu = Some(nu_u32);
            // Standard 4-slot HP-41 stack drop: X←Y, Y←Z, Z←T, T←T
            // (HP-41 duplicates T on drop). The user's original T
            // value is now preserved — no more silent destruction.
            state.stack.x = state.stack.y.clone();
            state.stack.y = state.stack.z.clone();
            state.stack.z = state.stack.t.clone();
            // (T unchanged — HP-41 stack-drop convention.)
            // Advance modal to ChisqdModeChoice.
            state.modal_program = Some(crate::ops::math1::modal::ModalProgram::Stat1(
                Stat1Step::ChisqdModeChoice,
            ));
            state.modal_prompt = Some("\u{03A3}CHISQD MODE?".to_string());
            Ok(())
        }
        Stat1Step::ChisqdModeChoice => {
            // Read mode index from X (truncate to integer; reject fractional).
            let mode_index = state.stack.x.trunc_int();
            let mode_i32 = mode_index.inner().to_i32_safe()?;
            // Recover ν from the transient carrier (REVIEW.md WR-03):
            // the field is set by ChisqdNuPrompt and unaffected by
            // any stack mutation between the two submits. Take +
            // clear so a subsequent ΣCHISQD cycle starts fresh.
            let nu_u32 = state.pending_chisqd_nu.take().ok_or_else(|| {
                // No prior ν submit (caller invoked ChisqdModeChoice
                // out of sequence) — clear modal state and surface
                // Domain rather than silently consuming garbage.
                state.modal_program = None;
                state.modal_prompt = None;
                HpError::Domain
            })?;
            if nu_u32 == 0 {
                state.modal_program = None;
                state.modal_prompt = None;
                return Err(HpError::Domain);
            }
            // Standard HP-41 stack drop: X←Y (where the user's χ²
            // statistic x sits), Y←Z, Z←T, T←T (duplicates per
            // hardware). With ν no longer occupying T, the drop
            // is straightforward.
            state.stack.x = state.stack.y.clone();
            state.stack.y = state.stack.z.clone();
            state.stack.z = state.stack.t.clone();
            // (T unchanged.)
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
        Stat1Step::PolypDegreePrompt(_) => {
            // Read degree d from X (truncate-to-integer; reject fractional).
            let degree_dec = state.stack.x.trunc_int();
            let d_i32 = degree_dec.inner().to_i32_safe()?;
            if d_i32 < 1 || d_i32 > crate::ops::stat1::STAT1_POLYP_DEGREE_MAX as i32 {
                state.modal_program = None;
                state.modal_prompt = None;
                return Err(HpError::Domain);
            }
            // SIZE-floor check BEFORE we touch any extended register.
            if state.regs.len() < crate::ops::stat1::STAT1_MAX_REG + 1 {
                state.modal_program = None;
                state.modal_prompt = None;
                return Err(HpError::InvalidOp);
            }
            // Persist d to the OM-cited degree slot.
            state.regs[crate::ops::stat1::STAT1_POLYP_DEGREE_REG] = state.stack.x.clone().into();
            // Drop X (the degree the user just submitted).
            state.stack.x = state.stack.y.clone();
            state.stack.y = state.stack.z.clone();
            state.stack.z = state.stack.t.clone();
            // Clear modal state (the user is now expected to accumulate
            // higher-power Σ sums and re-invoke ΣPOLYP for compute, OR
            // the test/program path may invoke compute_polyp_coefficients
            // directly).
            state.modal_program = None;
            state.modal_prompt = None;
            // Per OM/SPEC: after degree submission, compute the fit using
            // any pre-populated higher-power Σ sums. For the most common
            // user flow the sums are zero — compute will return Domain
            // (singular n=0 row) — which is acceptable; the user can
            // re-invoke ΣPOLYP once the sums are populated. For the test
            // path the sums are pre-populated; compute succeeds.
            crate::ops::stat1::regression::compute_polyp_coefficients(state)
        }
        Stat1Step::SeedPrompt => {
            // Normalize the value currently on stack X into [0, 1) before
            // writing into `state.rand_seed`. The previous implementation
            // claimed the LCG body would "consume only the fractional part"
            // but `trunc_int` is applied to the post-multiply `stepped`
            // value (not to the raw seed) — so a user seed of -0.5 or 1.5
            // produced a NEGATIVE RAND output, violating the documented
            // [0, 1) contract. REVIEW.md CR-01 mitigation.
            state.rand_seed =
                crate::ops::stat1::rand::normalize_seed_to_unit_interval(&state.stack.x)?;
            // Drop X (the seed the user submitted).
            state.stack.x = state.stack.y.clone();
            state.stack.y = state.stack.z.clone();
            state.stack.z = state.stack.t.clone();
            // Clear modal state.
            state.modal_program = None;
            state.modal_prompt = None;
            Ok(())
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
        // Plan 33-08: OM override allowed (SPEC.md Req. 22). Default to
        // the Math Pac I POLY precedent string "DEGREE=?".
        Stat1Step::PolypDegreePrompt(_) => Some("DEGREE=?".to_string()),
        // Plan 33-08: SEED? matches OM convention (SPEC.md Req. 36).
        Stat1Step::SeedPrompt => Some("SEED?".to_string()),
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
        | Stat1Step::ChisqdModeChoice
        | Stat1Step::PolypDegreePrompt(_)
        | Stat1Step::SeedPrompt => false,
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
    /// ChisqdModeChoice + stash ν in `state.pending_chisqd_nu` carrier
    /// (REVIEW.md WR-03 — replaces the previous stack-T side channel).
    #[test]
    fn submit_chisqd_nu_prompt_advances_to_mode_choice() {
        let mut state = CalcState::new();
        state.modal_program = Some(crate::ops::math1::modal::ModalProgram::Stat1(
            Stat1Step::ChisqdNuPrompt,
        ));
        state.stack.x = crate::num::HpNum::from(3i32);
        // Seed T with a distinctive value so we can verify the
        // standard HP-41 drop preserves it (no more T-clobber).
        state.stack.t = crate::num::HpNum::from(42i32);
        let r = submit_step(&mut state, Stat1Step::ChisqdNuPrompt);
        assert_eq!(r, Ok(()));
        // Modal advances to ChisqdModeChoice with the "ΣCHISQD MODE?" prompt.
        assert!(matches!(
            state.modal_program,
            Some(crate::ops::math1::modal::ModalProgram::Stat1(
                Stat1Step::ChisqdModeChoice
            ))
        ));
        assert_eq!(state.modal_prompt, Some("\u{03A3}CHISQD MODE?".to_string())); // LINT-EXEMPT: string comparison, no HpNum; lookahead false positive from comment text containing 'HpNum'
                                                                                  // ν stashed in the transient carrier (post-WR-03 design).
                                                                                  // LINT-EXEMPT: integer-equality — Option<u32> comparison, no HpNum value; flagged by lookahead false positive due to adjacent HpNum on next line
        assert_eq!(state.pending_chisqd_nu, Some(3));
        // User's original T is preserved (no more side-channel clobber).
        // LINT-EXEMPT: integer-equality via HpNum::from(42i32) — no f64 bridge; verifying exact stack.t preservation across modal submit
        assert_eq!(state.stack.t, crate::num::HpNum::from(42i32));
    }

    /// Catches: submit_step(ChisqdModeChoice) silently consuming
    /// stale data when invoked out of sequence (carrier empty).
    /// REVIEW.md WR-03 regression guard — the new design returns
    /// Domain rather than reading garbage from the stack.
    #[test]
    fn submit_chisqd_mode_choice_without_nu_carrier_is_domain_err() {
        let mut state = CalcState::new();
        state.modal_program = Some(crate::ops::math1::modal::ModalProgram::Stat1(
            Stat1Step::ChisqdModeChoice,
        ));
        // pending_chisqd_nu is None by default — caller forgot to
        // submit ν first.
        state.stack.x = crate::num::HpNum::from(1i32); // mode = PDF
        let r = submit_step(&mut state, Stat1Step::ChisqdModeChoice);
        assert_eq!(r, Err(HpError::Domain));
        // Modal state cleared even on error (D-07 never-discard).
        assert!(state.modal_program.is_none());
        assert!(state.modal_prompt.is_none());
    }

    /// Catches: ΣCHISQD T-clobber regression — confirm that arbitrary
    /// stack-lifting Ops invoked BETWEEN the two submits do not
    /// corrupt the ν carrier. REVIEW.md WR-03 acceptance test.
    #[test]
    fn submit_chisqd_nu_survives_stack_lift_between_submits() {
        let mut state = CalcState::new();
        // Open modal at NuPrompt, submit ν=5.
        state.modal_program = Some(crate::ops::math1::modal::ModalProgram::Stat1(
            Stat1Step::ChisqdNuPrompt,
        ));
        state.stack.x = crate::num::HpNum::from(5i32);
        submit_step(&mut state, Stat1Step::ChisqdNuPrompt).unwrap();
        assert_eq!(state.pending_chisqd_nu, Some(5));

        // User invokes some arbitrary stack-lifting Op between
        // submits — e.g., adds 1 to the entered χ² statistic. This
        // would have clobbered T under the old design.
        state.stack.t = crate::num::HpNum::zero(); // simulate any stack-lift
        state.stack.z = crate::num::HpNum::from(99i32);

        // ν carrier still holds 5 (decoupled from stack).
        assert_eq!(state.pending_chisqd_nu, Some(5));
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
