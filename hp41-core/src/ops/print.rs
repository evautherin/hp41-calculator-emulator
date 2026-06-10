//! Phase 11 print operations: PRX, PRA, PRSTK.
//!
//! All three ops have LiftEffect::Neutral — they read state but do not modify the stack.
//! Output is buffered into state.print_buffer; the CLI drains the buffer after each dispatch.
//!
//! UNC-02 (Phase 66): Per HP-41C OM p.53-54 and p.57-58, print functions error with
//! NONEXISTENT when no printer is connected. Flag 55 = "Printer Existence" (set
//! automatically when a printer is in the system); flag 21 = "Printer Enable" (user-
//! settable: "calculator assumes printer is present in system"). If neither flag is set,
//! op_prx/op_pra/op_prstk return Err(HpError::NonExistent).
//! Note: flag 25 = "Error Ignore" — NOT a printer flag; use flags 21 and 55 only.

use crate::error::HpError;
use crate::format::format_hpnum;
use crate::ops::flags::flag_get;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;

/// Return `Err(HpError::NonExistent)` if neither flag 55 (Printer Existence) nor flag 21
/// (Printer Enable) is set. Matches HP-41C OM p.57-58: "An attempt was made to execute a
/// specific print function when the printer was not connected to the system."
#[inline]
fn require_printer(state: &CalcState) -> Result<(), HpError> {
    if !flag_get(state.flags, 55) && !flag_get(state.flags, 21) {
        return Err(HpError::NonExistent);
    }
    Ok(())
}

/// PRX — print X register in current display format, right-aligned to 24 chars.
/// Pushes exactly one line to state.print_buffer. LiftEffect: Neutral.
/// Returns Err(NonExistent) when no printer is connected (OM p.57-58, UNC-02).
pub fn op_prx(state: &mut CalcState) -> Result<(), HpError> {
    require_printer(state)?;
    let line = format!("{:>24}", format_hpnum(&state.stack.x, &state.display_mode));
    state.print_buffer.push(line);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// PRA — print ALPHA register, left-aligned to 24 chars.
/// Pushes exactly one line to state.print_buffer. LiftEffect: Neutral.
/// Returns Err(NonExistent) when no printer is connected (OM p.57-58, UNC-02).
/// NOTE: Does NOT use format_alpha() which truncates to 12 chars. Uses 24-char width directly.
pub fn op_pra(state: &mut CalcState) -> Result<(), HpError> {
    require_printer(state)?;
    // Take at most 24 chars from alpha_reg (HP-41 ALPHA is max 24 chars but guard regardless).
    let alpha = state.alpha_reg.chars().take(24).collect::<String>();
    let line = format!("{alpha:<24}");
    state.print_buffer.push(line);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// PRSTK — print full stack T/Z/Y/X/LASTX/ALPHA, 6 lines of 24 chars each.
/// Pushes 6 lines to state.print_buffer. Line format: left-aligned 7-char label + 17-char value.
/// Returns Err(NonExistent) when no printer is connected (OM p.57-58, UNC-02).
/// LiftEffect: Neutral.
pub fn op_prstk(state: &mut CalcState) -> Result<(), HpError> {
    require_printer(state)?;
    let mode = &state.display_mode.clone();
    // Numeric lines: 7-char label (left) + 17-char formatted value (right) = 24 chars total.
    // format_hpnum output for SCI 9 widest case is "-1.234567890E-99" = 16 chars → fits in :>17.
    let lines: [String; 6] = [
        format!("{:<7}{:>17}", "T:", format_hpnum(&state.stack.t, mode)),
        format!("{:<7}{:>17}", "Z:", format_hpnum(&state.stack.z, mode)),
        format!("{:<7}{:>17}", "Y:", format_hpnum(&state.stack.y, mode)),
        format!("{:<7}{:>17}", "X:", format_hpnum(&state.stack.x, mode)),
        format!(
            "{:<7}{:>17}",
            "LASTX:",
            format_hpnum(&state.stack.lastx, mode)
        ),
        {
            // ALPHA line is left-aligned value (not right-aligned), max 17 chars.
            let alpha = state.alpha_reg.chars().take(17).collect::<String>();
            format!("{:<7}{:<17}", "ALPHA:", alpha)
        },
    ];
    for line in lines {
        state.print_buffer.push(line);
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
