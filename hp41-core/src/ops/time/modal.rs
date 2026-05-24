// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Modal state-machine for Time Module prompt-driven workflows.
//!
//! `TimeStep` is the per-program step enum carried by the
//! `ModalProgram::Time(TimeStep)` variant living in
//! `hp41-core/src/ops/math1/modal.rs` (math1/ freeze exception D-carried.4).
//!
//! ## Phase 38 scope
//!
//! Three prompt variants ship in the scaffold:
//! - `SetTimePrompt` — SETIME opens here; user enters HH.MMSSss in X.
//! - `SetDatePrompt` — SETDATE opens here; user enters date decimal in X.
//! - `XyzalmTimePrompt` — XYZALM opens here; user enters alarm time in X.
//!
//! Full submit logic (SystemTime offset computation, alarm entry construction)
//! lands in Wave 2 plans. The stubs here clear modal state and return Ok(())
//! so the compile invariant and modal-routing infrastructure are valid from day 1.
//!
//! ## Why this file lives in `time/` not `math1/`
//!
//! D-carried.4 authorizes ONE additional math1/ freeze carve-out for
//! `math1/modal.rs` — a single-line variant addition plus three dispatch arms.
//! All Time-Module-specific semantics (this file) stay in `time/` so the
//! math1/ blast radius is the same minimal ~8 lines established by the
//! Stat1 precedent (D-33.3b).

use crate::error::HpError;
use crate::state::CalcState;

/// Per-step modal state for the Time Module prompt-driven workflows.
///
/// Carried by `ModalProgram::Time(TimeStep)` (D-carried.4 freeze-exception
/// variant in `math1/modal.rs`).
#[derive(Debug, Clone, PartialEq)]
pub enum TimeStep {
    /// SETIME — awaiting time entry in X (HH.MMSSss format).
    /// Prompt: "TIME?"
    SetTimePrompt,
    /// SETDATE — awaiting date entry in X (per Flag 31: MDY or DMY).
    /// Prompt: "DATE?"
    SetDatePrompt,
    /// XYZALM — awaiting alarm trigger time in X (HH.MMSSss format).
    /// Prompt: "ALARM TIME?"
    XyzalmTimePrompt,
}

/// Per-step submit dispatch — called by `ModalProgram::Time` dispatch arm.
///
/// Phase 38 stubs clear modal state and return Ok(()). Full implementations
/// (SETIME offset computation, SETDATE, XYZALM alarm-entry construction)
/// land in Wave 2.
pub fn submit_step(state: &mut CalcState, step: TimeStep) -> Result<(), HpError> {
    match step {
        TimeStep::SetTimePrompt => {
            // Phase 38 stub: clear modal state.
            // Wave 2: parse X as HH.MMSSss, compute time_offset_secs.
            state.modal_program = None;
            state.modal_prompt = None;
            Ok(())
        }
        TimeStep::SetDatePrompt => {
            // Phase 38 stub: clear modal state.
            // Wave 2: parse X per Flag 31, adjust time_offset_secs date component.
            state.modal_program = None;
            state.modal_prompt = None;
            Ok(())
        }
        TimeStep::XyzalmTimePrompt => {
            // Phase 38 stub: clear modal state.
            // Wave 2: parse X as alarm time, read ALPHA for alarm type, push AlarmEntry.
            state.modal_program = None;
            state.modal_prompt = None;
            Ok(())
        }
    }
}

/// Per-step prompt accessor — called by `ModalProgram::current_prompt`
/// (the carrier-enum dispatch in `math1/modal.rs`).
pub fn current_prompt(step: &TimeStep) -> Option<String> {
    match step {
        TimeStep::SetTimePrompt => Some("TIME?".to_string()),
        TimeStep::SetDatePrompt => Some("DATE?".to_string()),
        TimeStep::XyzalmTimePrompt => Some("ALARM TIME?".to_string()),
    }
}

/// Per-step alpha-label gate — called by `ModalProgram::requires_alpha_label`
/// (the carrier-enum dispatch in `math1/modal.rs`).
///
/// All Phase 38 Time Module steps accept numeric input (HH.MMSSss or date
/// decimal) — none require an alpha label. Returns false for all variants.
pub fn requires_alpha_label(step: &TimeStep) -> bool {
    match step {
        TimeStep::SetTimePrompt | TimeStep::SetDatePrompt | TimeStep::XyzalmTimePrompt => false,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn set_time_prompt_text() {
        assert_eq!(
            current_prompt(&TimeStep::SetTimePrompt),
            Some("TIME?".to_string())
        );
    }

    #[test]
    fn set_date_prompt_text() {
        assert_eq!(
            current_prompt(&TimeStep::SetDatePrompt),
            Some("DATE?".to_string())
        );
    }

    #[test]
    fn xyzalm_time_prompt_text() {
        assert_eq!(
            current_prompt(&TimeStep::XyzalmTimePrompt),
            Some("ALARM TIME?".to_string())
        );
    }

    #[test]
    fn no_time_step_requires_alpha_label() {
        for step in [
            TimeStep::SetTimePrompt,
            TimeStep::SetDatePrompt,
            TimeStep::XyzalmTimePrompt,
        ] {
            assert!(
                !requires_alpha_label(&step),
                "Time step {step:?} must not require alpha label"
            );
        }
    }

    #[test]
    fn time_step_clone_and_eq() {
        let step = TimeStep::SetTimePrompt;
        assert_eq!(step.clone(), step);
        assert_ne!(TimeStep::SetTimePrompt, TimeStep::SetDatePrompt);
        assert_ne!(TimeStep::SetDatePrompt, TimeStep::XyzalmTimePrompt);
    }

    #[test]
    fn submit_set_time_clears_modal_state() {
        let mut state = CalcState::new();
        state.modal_program = Some(crate::ops::math1::modal::ModalProgram::Time(
            TimeStep::SetTimePrompt,
        ));
        state.modal_prompt = Some("TIME?".to_string());
        submit_step(&mut state, TimeStep::SetTimePrompt).unwrap();
        assert!(state.modal_program.is_none());
        assert!(state.modal_prompt.is_none());
    }

    #[test]
    fn submit_set_date_clears_modal_state() {
        let mut state = CalcState::new();
        state.modal_program = Some(crate::ops::math1::modal::ModalProgram::Time(
            TimeStep::SetDatePrompt,
        ));
        state.modal_prompt = Some("DATE?".to_string());
        submit_step(&mut state, TimeStep::SetDatePrompt).unwrap();
        assert!(state.modal_program.is_none());
        assert!(state.modal_prompt.is_none());
    }

    #[test]
    fn submit_xyzalm_clears_modal_state() {
        let mut state = CalcState::new();
        state.modal_program = Some(crate::ops::math1::modal::ModalProgram::Time(
            TimeStep::XyzalmTimePrompt,
        ));
        state.modal_prompt = Some("ALARM TIME?".to_string());
        submit_step(&mut state, TimeStep::XyzalmTimePrompt).unwrap();
        assert!(state.modal_program.is_none());
        assert!(state.modal_prompt.is_none());
    }
}
