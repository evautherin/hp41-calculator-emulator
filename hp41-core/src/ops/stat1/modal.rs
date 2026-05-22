// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Modal state-machine for Stat 1 Pac prompt-driven workflows.
//!
//! `Stat1Step` is the per-program step enum carried by the
//! `ModalProgram::Stat1(Stat1Step)` variant living in
//! `hp41-core/src/ops/math1/modal.rs` (math1/ freeze exception D-33.3b).
//!
//! ## Plan 33-01 scaffolding
//!
//! Plan 33-01 ships ONE placeholder variant (`Stat1Step::Placeholder`) so
//! the three dispatch functions (`submit_step`, `current_prompt`,
//! `requires_alpha_label`) can be exhaustively matched and the
//! `ModalProgram::Stat1` carrier-enum arm compiles. The placeholder is
//! deliberately a no-op:
//!
//! - `submit_step` returns `Ok(())` (does nothing).
//! - `current_prompt` returns `None` (no visible prompt).
//! - `requires_alpha_label` returns `false`.
//!
//! ## Downstream replacement plan
//!
//! - Plan 33-03 replaces `Placeholder` with `NormdModeChoice` (ΣNORMD
//!   mode dispatch: E=CDF / C=PDF / A=inverse) and `ChisqdNuPrompt`
//!   (ΣCHISQD ν=? entry).
//! - Plan 33-08 adds `PolypDegreePrompt` (ΣPOLYP DEGREE=?) and
//!   `SeedPrompt` (SEED label, `requires_alpha_label = true`).
//!
//! Each downstream plan extends the three dispatch functions exhaustively
//! (no `_ =>` catch-all per CLAUDE.md "Core engine — no panics" + the
//! FN-CLI-04 invariant inherited from `math1::modal::ModalProgram::current_prompt`).
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
/// variant in `math1/modal.rs`). Real variants (NormdModeChoice,
/// ChisqdNuPrompt, PolypDegreePrompt, SeedPrompt) are added by Plans
/// 33-03 + 33-08; the `Placeholder` variant lets Plan 33-01 ship the
/// dispatch wiring without forcing those plans to also wrap the carrier
/// enum extension.
#[derive(Debug, Clone, PartialEq)]
pub enum Stat1Step {
    /// Plan-33-01 SCAFFOLDING — to be removed when Plans 33-03 + 33-08
    /// add the real variants. No-op in every dispatch function.
    Placeholder,
}

/// Per-step submit dispatch — called by `math1::submit_modal` when the
/// active modal is `ModalProgram::Stat1(...)` and the user presses R/S.
///
/// Plan 33-01 placeholder body: returns `Ok(())` for every variant.
/// Plan 33-03 + 33-08 replace this body with real per-variant logic.
pub fn submit_step(_state: &mut CalcState, step: Stat1Step) -> Result<(), HpError> {
    match step {
        Stat1Step::Placeholder => Ok(()),
    }
}

/// Per-step prompt accessor — called by `ModalProgram::current_prompt`
/// (the carrier-enum dispatch in `math1/modal.rs`).
///
/// Plan 33-01 placeholder body: returns `None` for every variant
/// (no visible prompt during the scaffolding state). Plan 33-03 + 33-08
/// fill in OM-cited prompt strings (e.g. "ΣNORMD MODE?", "ν=?",
/// "DEGREE=?", "SEED?").
pub fn current_prompt(step: &Stat1Step) -> Option<String> {
    match step {
        Stat1Step::Placeholder => None,
    }
}

/// Per-step alpha-label gate — called by `ModalProgram::requires_alpha_label`
/// (the carrier-enum dispatch in `math1/modal.rs`).
///
/// Plan 33-01 placeholder body: returns `false` for every variant.
/// Plan 33-08 sets `true` for the SEED prompt (alpha-label input flow per
/// D-29.7 / D-29.9 parity with INTG / SOLVE / DIFEQ FunctionNamePrompt).
pub fn requires_alpha_label(step: &Stat1Step) -> bool {
    match step {
        Stat1Step::Placeholder => false,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Catches: Placeholder variant accidentally dropping from dispatch.
    #[test]
    fn placeholder_submit_step_is_noop() {
        let mut state = CalcState::new();
        assert!(submit_step(&mut state, Stat1Step::Placeholder).is_ok());
    }

    // Catches: Placeholder variant returning a non-None prompt.
    #[test]
    fn placeholder_current_prompt_is_none() {
        assert_eq!(current_prompt(&Stat1Step::Placeholder), None);
    }

    // Catches: Placeholder variant erroneously requesting alpha label.
    #[test]
    fn placeholder_requires_alpha_label_is_false() {
        assert!(!requires_alpha_label(&Stat1Step::Placeholder));
    }

    // Catches: Clone + PartialEq derive regression on Stat1Step.
    #[test]
    fn stat1_step_clone_and_eq() {
        let step = Stat1Step::Placeholder;
        assert_eq!(step.clone(), step);
    }
}
