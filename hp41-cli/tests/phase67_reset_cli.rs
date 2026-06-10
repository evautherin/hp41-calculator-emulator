//! Phase 67-02 — CLI Ctrl+R reset escape-hatch integration tests.
//!
//! Covers the two-tier status-bar state machine:
//!   RST-CLI-01: Ctrl+R opens the tier-selection prompt even over an active pending_input.
//!   RST-CLI-02: `s` in AwaitingTier calls soft_reset, clears CLI transient state, persists.
//!   RST-CLI-03: `f` + `y` calls memory_lost, clears CLI transient state, persists.
//!   RST-CLI-04: `f` + `n`/Esc/other cancels; no change to state or disk.
//!   RST-CLI-05: Esc in AwaitingTier cancels; any other key also cancels.
//!   RST-CLI-06: shift_armed is cleared on both soft and full reset.
//!   RST-CLI-07: Ctrl+R no longer dispatches RDPRGM (conflict resolved to Ctrl+E).

#![allow(clippy::unwrap_used)]

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use hp41_cli::app::{App, PendingInput, ResetPrompt};
use hp41_core::state::CalcState;

// ── Test scaffolding ──────────────────────────────────────────────────────────

/// Build a `KeyEvent::Press` for a plain character key (no modifiers).
fn key(c: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(c),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

/// Build a Ctrl+<char> `KeyEvent::Press`.
fn ctrl_key(c: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(c),
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

/// Build an Esc `KeyEvent::Press`.
fn esc_key() -> KeyEvent {
    KeyEvent {
        code: KeyCode::Esc,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

/// Build a fresh App backed by a tempdir so persistence side effects are isolated.
/// The state file path is inside the tempdir — saves are real but contained.
fn make_app() -> (App, tempfile::TempDir) {
    let tmp = tempfile::tempdir().expect("tempdir creation must succeed");
    let state_path = tmp.path().join("phase67-cli-test-state.json");
    let app = App::new(CalcState::new(), state_path, None);
    (app, tmp)
}

// ── RST-CLI-01: Ctrl+R opens the reset prompt even over active pending_input ──

/// Ctrl+R opens the tier-selection prompt from normal state.
#[test]
fn test_ctrl_r_opens_tier_prompt_from_normal() {
    let (mut app, _tmp) = make_app();
    assert_eq!(app.reset_prompt, ResetPrompt::None, "starts in None");

    app.handle_key(ctrl_key('r'));

    assert_eq!(
        app.reset_prompt,
        ResetPrompt::AwaitingTier,
        "Ctrl+R must advance reset_prompt to AwaitingTier"
    );
    assert!(!app.exit, "Ctrl+R must not exit the app");
}

/// Ctrl+R opens the reset prompt even when a pending_input modal is active (D-07 exception).
#[test]
fn test_ctrl_r_over_active_pending_input_opens_prompt() {
    let (mut app, _tmp) = make_app();
    // Simulate a "stuck" state: an active modal that would normally swallow keys.
    app.pending_input = Some(PendingInput::TonePrompt);

    app.handle_key(ctrl_key('r'));

    // The reset prompt must be open regardless of the trapped pending_input.
    assert_eq!(
        app.reset_prompt,
        ResetPrompt::AwaitingTier,
        "Ctrl+R must open the reset prompt even with an active pending_input"
    );
    // The pending_input is NOT cleared by merely opening the prompt — it will be
    // cleared when the user actually performs a reset (s or f+y).
    // This preserves the cancel path: if the user presses Esc, the modal continues.
}

// ── RST-CLI-02: `s` → soft_reset + persist ───────────────────────────────────

/// Pressing `s` in AwaitingTier calls soft_reset and clears transient state.
#[test]
fn test_s_in_awaiting_tier_soft_resets() {
    let (mut app, _tmp) = make_app();

    // Set up some trapped / transient state.
    app.state.entry_buf = "3.14".to_string();
    app.state.prgm_mode = true;
    app.state.is_running = false; // already stopped but prgm_mode is stuck
    app.shift_armed = true;
    app.pending_input = Some(PendingInput::TonePrompt);

    // Preserved data: store a known value in a register.
    app.state.regs[0] = hp41_core::HpNum::from(42).into();

    // Open reset prompt, then press 's'.
    app.handle_key(ctrl_key('r'));
    assert_eq!(app.reset_prompt, ResetPrompt::AwaitingTier);
    app.handle_key(key('s'));

    // Reset prompt must be cleared.
    assert_eq!(
        app.reset_prompt,
        ResetPrompt::None,
        "reset_prompt must return to None after soft reset"
    );
    // CLI transient state cleared.
    assert!(
        !app.shift_armed,
        "shift_armed must be cleared by soft reset"
    );
    assert!(
        app.pending_input.is_none(),
        "pending_input must be cleared by soft reset"
    );
    // Core trapping state cleared.
    assert!(
        !app.state.prgm_mode,
        "prgm_mode must be cleared by soft reset"
    );
    assert!(
        app.state.entry_buf.is_empty(),
        "entry_buf must be cleared by soft reset"
    );
    // Stored data preserved.
    assert_eq!(
        app.state.regs[0],
        hp41_core::HpNum::from(42).into(),
        "register data must be preserved after soft reset"
    );
    // A success message is set.
    let msg = app.message.as_deref().unwrap_or("");
    assert!(
        msg.contains("Soft reset") || msg.contains("soft reset"),
        "message must confirm the soft reset; got {msg:?}"
    );
}

// ── RST-CLI-03: `f` + `y` → memory_lost + persist ───────────────────────────

/// Pressing `f` then `y` calls memory_lost and wipes all data.
#[test]
fn test_f_then_y_performs_full_reset() {
    let (mut app, _tmp) = make_app();

    // Set up some stored data that should be wiped.
    app.state.regs[5] = hp41_core::HpNum::from(99).into();
    app.state.flags = 0b1111;
    app.shift_armed = true;
    app.pending_input = Some(PendingInput::TonePrompt);

    // Open reset prompt → advance to confirm tier → confirm.
    app.handle_key(ctrl_key('r'));
    assert_eq!(app.reset_prompt, ResetPrompt::AwaitingTier);
    app.handle_key(key('f'));
    assert_eq!(
        app.reset_prompt,
        ResetPrompt::AwaitingFullConfirm,
        "'f' must advance to AwaitingFullConfirm"
    );
    app.handle_key(key('y'));

    // Reset prompt must be cleared.
    assert_eq!(
        app.reset_prompt,
        ResetPrompt::None,
        "reset_prompt must return to None after full reset"
    );
    // CLI transient state cleared.
    assert!(
        !app.shift_armed,
        "shift_armed must be cleared by full reset"
    );
    assert!(
        app.pending_input.is_none(),
        "pending_input must be cleared by full reset"
    );
    // Stored data wiped.
    assert!(
        app.state.regs[5].is_zero(),
        "register data must be wiped by full reset (MEMORY LOST)"
    );
    assert_eq!(app.state.flags, 0, "flags must be wiped by full reset");
    // A MEMORY LOST confirmation message is set.
    let msg = app.message.as_deref().unwrap_or("");
    assert!(
        msg.contains("MEMORY LOST"),
        "message must confirm MEMORY LOST; got {msg:?}"
    );
}

/// Uppercase 'Y' also confirms full reset (case-insensitive confirm).
#[test]
fn test_full_reset_confirmed_by_uppercase_y() {
    let (mut app, _tmp) = make_app();
    app.state.regs[0] = hp41_core::HpNum::from(7).into();

    app.handle_key(ctrl_key('r'));
    app.handle_key(key('f'));
    app.handle_key(key('Y')); // uppercase Y

    assert_eq!(app.reset_prompt, ResetPrompt::None);
    assert!(
        app.state.regs[0].is_zero(),
        "uppercase Y must also confirm full reset"
    );
}

// ── RST-CLI-04: `f` + `n`/Esc/other → cancel ────────────────────────────────

/// Pressing `n` in AwaitingFullConfirm cancels without any state change.
#[test]
fn test_full_confirm_n_cancels() {
    let (mut app, _tmp) = make_app();
    app.state.regs[3] = hp41_core::HpNum::from(55).into();

    app.handle_key(ctrl_key('r'));
    app.handle_key(key('f'));
    assert_eq!(app.reset_prompt, ResetPrompt::AwaitingFullConfirm);
    app.handle_key(key('n'));

    assert_eq!(
        app.reset_prompt,
        ResetPrompt::None,
        "'n' must cancel and return to None"
    );
    // Data must be unchanged.
    assert_eq!(
        app.state.regs[3],
        hp41_core::HpNum::from(55).into(),
        "'n' must leave register data untouched"
    );
    let msg = app.message.as_deref().unwrap_or("");
    assert!(
        msg.contains("cancelled") || msg.contains("cancel"),
        "cancellation message expected; got {msg:?}"
    );
}

/// Pressing Esc in AwaitingFullConfirm cancels.
#[test]
fn test_full_confirm_esc_cancels() {
    let (mut app, _tmp) = make_app();
    app.state.regs[0] = hp41_core::HpNum::from(12).into();

    app.handle_key(ctrl_key('r'));
    app.handle_key(key('f'));
    app.handle_key(esc_key());

    assert_eq!(app.reset_prompt, ResetPrompt::None);
    assert_eq!(
        app.state.regs[0],
        hp41_core::HpNum::from(12).into(),
        "Esc in AwaitingFullConfirm must not wipe data"
    );
}

/// Any non-y/n key in AwaitingFullConfirm cancels.
#[test]
fn test_full_confirm_other_key_cancels() {
    let (mut app, _tmp) = make_app();
    app.state.regs[1] = hp41_core::HpNum::from(8).into();

    app.handle_key(ctrl_key('r'));
    app.handle_key(key('f'));
    app.handle_key(key('x')); // random key

    assert_eq!(app.reset_prompt, ResetPrompt::None);
    assert_eq!(
        app.state.regs[1],
        hp41_core::HpNum::from(8).into(),
        "unrecognised key must not wipe data"
    );
}

// ── RST-CLI-05: Esc and other keys in AwaitingTier cancel ────────────────────

/// Pressing Esc in AwaitingTier cancels without performing any reset.
#[test]
fn test_tier_esc_cancels() {
    let (mut app, _tmp) = make_app();
    app.state.regs[2] = hp41_core::HpNum::from(100).into();

    app.handle_key(ctrl_key('r'));
    assert_eq!(app.reset_prompt, ResetPrompt::AwaitingTier);
    app.handle_key(esc_key());

    assert_eq!(
        app.reset_prompt,
        ResetPrompt::None,
        "Esc in AwaitingTier must cancel"
    );
    assert_eq!(
        app.state.regs[2],
        hp41_core::HpNum::from(100).into(),
        "Esc must not change stored data"
    );
}

/// Any unrecognised key in AwaitingTier cancels.
#[test]
fn test_tier_other_key_cancels() {
    let (mut app, _tmp) = make_app();

    app.handle_key(ctrl_key('r'));
    app.handle_key(key('z')); // not 's' or 'f'

    assert_eq!(
        app.reset_prompt,
        ResetPrompt::None,
        "unrecognised key in AwaitingTier must cancel"
    );
}

// ── RST-CLI-06: shift_armed cleared on both soft and full reset ───────────────

/// shift_armed is always cleared as part of a soft reset.
#[test]
fn test_soft_reset_clears_shift_armed() {
    let (mut app, _tmp) = make_app();
    app.shift_armed = true;

    app.handle_key(ctrl_key('r'));
    app.handle_key(key('s'));

    assert!(
        !app.shift_armed,
        "soft reset must clear shift_armed (CLI-only transient state)"
    );
}

/// shift_armed is always cleared as part of a full reset.
#[test]
fn test_full_reset_clears_shift_armed() {
    let (mut app, _tmp) = make_app();
    app.shift_armed = true;

    app.handle_key(ctrl_key('r'));
    app.handle_key(key('f'));
    app.handle_key(key('y'));

    assert!(
        !app.shift_armed,
        "full reset must clear shift_armed (CLI-only transient state)"
    );
}

// ── RST-CLI-07: Ctrl+R no longer dispatches RDPRGM ──────────────────────────

/// After Phase 67-02, Ctrl+R opens the reset prompt, NOT RDPRGM.
/// The user must use Ctrl+E for RDPRGM now.
#[test]
fn test_ctrl_r_no_longer_dispatches_rdprgm() {
    let tmp = tempfile::tempdir().unwrap();
    let mut app = App::new(CalcState::new(), tmp.path().join("state.json"), None);
    // Set up cards_dir to isolate card I/O.
    // If Ctrl+R still dispatched Rdprgm it would set an error message about missing card.
    app.state.alpha_reg = "MISSING".to_string();

    app.handle_key(ctrl_key('r'));

    // Must open the reset prompt, not dispatch Rdprgm.
    assert_eq!(
        app.reset_prompt,
        ResetPrompt::AwaitingTier,
        "Ctrl+R must open the reset prompt, not dispatch RDPRGM"
    );
    // The message should NOT contain card-data error — that would indicate Rdprgm ran.
    let msg = app.message.as_deref().unwrap_or("");
    assert!(
        !msg.to_lowercase().contains("card"),
        "Ctrl+R must not dispatch RDPRGM; got message: {msg:?}"
    );
}
