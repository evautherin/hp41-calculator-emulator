//! Phase 14 IPC Layer — Tauri command handlers.
//!
//! Two commands registered in lib.rs:
//! - `dispatch_op(key_id, state)` — resolves key_id to Op, dispatches against CalcState,
//!   drains print_buffer, returns CalcStateView.
//! - `get_state(state)` — drains print_buffer and returns current CalcStateView (no dispatch).
//!
//! Decisions: D-08 (stateless IPC — no PendingModal), D-10 (Result<View, Error>),
//! and Claude's Discretion on poisoned-lock recovery (.unwrap_or_else(|e| e.into_inner())).
//!
//! Test strategy (RESEARCH.md Validation Architecture):
//! Tauri's `State<'_, AppState>` extractor cannot be constructed in unit tests without
//! the WebView mock harness. Therefore the IPC LOGIC is factored into two private helpers
//! `handle_op(&mut CalcState, &str)` and `handle_get_state(&mut CalcState)`. The
//! `#[tauri::command]` thunks are 2-line glue (lock + call helper). Tests target the
//! helpers and cover SC-2 (unknown key → GuiError) and SC-3 (print_buffer drain).

use crate::cards;
use crate::key_map;
use crate::persistence;
use crate::prefs::{default_prefs_path, save_prefs, GuiPrefs, VALID_THEMES};
use crate::types::{CalcStateView, GuiError};
use crate::{AppState, CancelFlag, PrefsState};
use hp41_core::cardreader::{
    capture_data_card, decode_all_programs, decode_data, encode_data, encode_program,
    insert_program_ops, load_data_card, picker_label,
};
use hp41_core::ops::dispatch;
use hp41_core::CalcState;
use serde::Serialize;
use tauri::AppHandle;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

#[cfg(test)]
use std::path::Path;

/// Tauri command: dispatch an op identified by a string key ID.
///
/// Uses the three-phase card-op drain so the AppState mutex is released
/// during disk I/O — without this split, a WPRGM on a large program or a
/// network-mounted `~/.hp41/cards/` would block every concurrent
/// `get_state` poll and freeze the UI.
///
/// Phase 1 (locked): dispatch, take any pending card request, encode
/// outgoing bytes for writes.
/// Phase 2 (unlocked): perform the actual filesystem I/O.
/// Phase 3 (locked): apply any read result, drain print_buffer, build view.
///
/// Concurrency note: between phase 1 (lock drop) and phase 3 (lock
/// re-acquire), another `dispatch_op` thunk could observe and mutate
/// `CalcState`. For writes this is fine — phase 1 captured an immutable
/// `PreparedCardOp` snapshot of the bytes to write. For reads, phase 3
/// overwrites whatever the interim thunk did to `program` (RDPRGM) or
/// `regs` (RDTA). This is acceptable given Tauri's single-channel command
/// queue and the single-user UI: a card-read overwriting concurrent state
/// matches user expectations ("the read landed").
#[tauri::command]
pub fn dispatch_op(key_id: &str, state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    // Phase 1: in-lock dispatch + prepare card op (no I/O).
    let prepared = {
        let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
        handle_op_prepare(&mut calc, key_id)?
    };

    // Phase 2: lock-free I/O. cards_dir_required surfaces "no $HOME" as a
    // CardData error instead of silently dropping the user's write.
    let read_result = match prepared {
        Some(p) => {
            let dir = cards::cards_dir_required().map_err(GuiError::from)?;
            cards::execute_prepared_card_op(p, &dir).map_err(GuiError::from)?
        }
        None => None,
    };

    // Phase 3: in-lock apply + view assembly.
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_op_finalize(&mut calc, read_result)
}

/// Tauri command: get current state without executing any op.
///
/// Locks AppState (with poisoned-lock recovery) and delegates to `handle_get_state`.
/// Does NOT touch `pending_card_op` — that field is `#[serde(skip)]` and therefore
/// cannot survive a process restart, so there is no "stale request" case to defend
/// against here. The drain belongs in `dispatch_op` (the only path that stages a
/// request) and nowhere else.
#[tauri::command]
pub fn get_state(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_get_state(&mut calc)
}

/// Single-phase test helper composing prepare + execute + apply + view.
///
/// `cards_dir` is injected so unit tests can drive the full card-op path
/// through a `tempfile::tempdir()`. Production code uses the three-phase
/// `dispatch_op` thunk instead, which releases the AppState mutex during I/O.
#[cfg(test)]
pub fn handle_op_with_cards_dir(
    calc: &mut CalcState,
    key_id: &str,
    cards_dir: &Path,
) -> Result<CalcStateView, GuiError> {
    let prepared = handle_op_prepare(calc, key_id)?;
    let read_result = match prepared {
        Some(p) => cards::execute_prepared_card_op(p, cards_dir).map_err(GuiError::from)?,
        None => None,
    };
    handle_op_finalize(calc, read_result)
}

/// Backwards-compatible single-phase entry point for tests and callers that
/// do not exercise the card-op path. Equivalent to `handle_op_with_cards_dir`
/// with a junk directory — any pending card op would error out at phase 2,
/// but the existing handle_op tests do not stage one.
#[cfg(test)]
pub fn handle_op(calc: &mut CalcState, key_id: &str) -> Result<CalcStateView, GuiError> {
    // Use a never-created tempdir path — tests using this helper never stage
    // a card op, so phase 2 is never reached. Staging one would panic the
    // test, which is the correct signal that the test should switch to
    // `handle_op_with_cards_dir`.
    let never = Path::new("/this/path/should/never/be/created/by/handle_op/tests");
    handle_op_with_cards_dir(calc, key_id, never)
}

/// Pure-Rust phase-1 helper: entry-buf branches return `Ok(None)` directly
/// because there is no dispatched op (and hence no card request). For named
/// ops, dispatches against `CalcState` and then takes any staged
/// `pending_card_op` out so the caller can release its state lock before
/// performing disk I/O.
///
/// Routing logic mirrors hp41-cli/src/app.rs digit-entry and dispatch
/// helpers. Digit keys append to entry_buf and bypass `dispatch`;
/// named/parameterized keys flow through key_map → dispatch → prepare.
pub fn handle_op_prepare(
    calc: &mut CalcState,
    key_id: &str,
) -> Result<Option<cards::PreparedCardOp>, GuiError> {
    // D-39.3 mirror: any dispatch exits clock display mode (CLI: app.rs:464).
    if calc.clock_active {
        calc.clock_active = false;
    }

    // D-39.4 mirror: "sw_exit" is a GUI-only key_id that clears stopwatch keyboard
    // mode (Esc in the CLI's handle_stopwatch_mode_key). No Op is dispatched.
    if key_id == "sw_exit" {
        calc.stopwatch_keyboard_mode = false;
        return Ok(None);
    }

    // ── Digit keys 0..=9 — bypass dispatch, append to entry_buf ───────────────
    if matches!(
        key_id,
        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
    ) {
        // INPUT-02 guard (Phase 9): cap exponent entry at 2 digits.
        if let Some(e_pos) = calc.entry_buf.find('e') {
            let after_e = &calc.entry_buf[e_pos + 1..];
            let exp_digits = after_e.chars().filter(|ch| ch.is_ascii_digit()).count();
            if exp_digits >= 2 {
                // Silently block third exponent digit — caller's finalize will
                // drain print_buffer and rebuild the view unchanged.
                return Ok(None);
            }
        }
        // key_id is exactly one ASCII char — safe to take .next().
        let ch = key_id
            .chars()
            .next()
            .expect("digit key_id has at least one char");
        calc.entry_buf.push(ch);
        return Ok(None);
    }

    // ── '.' — block duplicate '.' and '.' after 'e' ──────────────────────────
    if key_id == "." {
        if !calc.entry_buf.contains('.') && !calc.entry_buf.contains('e') {
            calc.entry_buf.push('.');
        }
        return Ok(None);
    }

    // ── 'e' (EEX) — implicit "1" mantissa on empty entry_buf (Phase 9 D-07) ──
    if key_id == "e" {
        if !calc.entry_buf.contains('e') {
            if calc.entry_buf.is_empty() {
                calc.entry_buf.push_str("1e");
            } else {
                calc.entry_buf.push('e');
            }
        }
        return Ok(None);
    }

    // ── "eex_chs" — toggle exponent sign in entry_buf (Phase 15 D-06) ────────────
    // Source: hp41-cli/src/app.rs (entry_buf direct mutation, no dispatch).
    // MUST come before key_map::resolve() — no Op::EexChs variant exists.
    if key_id == "eex_chs" {
        if let Some(e_pos) = calc.entry_buf.find('e') {
            let after_e = &calc.entry_buf[e_pos + 1..];
            if after_e.starts_with('-') {
                // Remove minus: "1e-2" → "1e2", "1e-" → "1e"
                calc.entry_buf.remove(e_pos + 1);
            } else {
                // Insert minus: "1e2" → "1e-2", "1e" → "1e-"
                calc.entry_buf.insert(e_pos + 1, '-');
            }
        }
        // No-op if entry_buf has no 'e' — React guards this but Rust is defensive.
        return Ok(None);
    }

    // ── Named / parameterized op — resolve and dispatch ──────────────────────
    let op = key_map::resolve(key_id)?;
    dispatch(calc, op).map_err(GuiError::from)?;

    // Take any staged Card Reader request out so the caller can release the
    // AppState mutex before performing disk I/O. encode-for-write happens
    // here too (against the just-dispatched state) so phase 2 sees a frozen
    // snapshot even if a concurrent thunk later mutates `calc`.
    cards::prepare_pending_card_op(calc).map_err(GuiError::from)
}

/// Phase 3 of `dispatch_op`: apply any read result back to state, drain
/// `print_buffer`, and build the `CalcStateView` the frontend renders.
pub fn handle_op_finalize(
    calc: &mut CalcState,
    read_result: Option<cards::CardReadResult>,
) -> Result<CalcStateView, GuiError> {
    if let Some(r) = read_result {
        cards::apply_card_read_result(calc, r);
    }
    // Phase 26 D-26.11: drain print_buffer AND event_buffer before from_state
    // (both are &mut, then from_state takes &). Mirror of the v2.0 print_buffer
    // drain — preserves the no-IPC-leakage pattern across both transient buffers.
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines, event_lines))
}

/// Pure-Rust helper for get_state — drains print_buffer and builds a CalcStateView.
///
/// Intentionally does NOT touch `pending_card_op`: that field is
/// `#[serde(skip)]` and therefore cannot survive a process restart, so the
/// "stale-request-from-crash" hazard the prior version defended against
/// does not exist. Draining here also triggered `fs::create_dir_all` on
/// every React poll — a syscall in the redraw path.
pub fn handle_get_state(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines, event_lines))
}

/// Tauri command: periodic tick for live clock/stopwatch display (D-41.1 / D-41.4).
///
/// Called by the frontend `setInterval` at 100ms intervals ONLY when
/// `clock_active || stopwatch_keyboard_mode` is true in the CalcStateView
/// (D-41.3 / D-41.8). When neither mode is active, the interval is cleared
/// by the frontend and this command is not called.
///
/// Calls `check_alarms()` first so alarm events fire promptly (~100ms latency)
/// during clock/stopwatch display modes (D-41.4). Outside those modes, alarms
/// fire on the next `dispatch_op` call per TIME-ALM-08.
///
/// This is a READ-ONLY query — it does NOT set `busyRef.current = true`.
/// A single skipped tick (100ms) due to Mutex contention is imperceptible.
#[tauri::command]
pub fn tick_time(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_tick_time(&mut calc)
}

/// Pure-Rust helper for tick_time — calls check_alarms, drains both buffers,
/// returns a CalcStateView with live-display trigger booleans projected.
///
/// Factored out for unit-testability (mirrors `handle_get_state` pattern).
pub fn handle_tick_time(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    // D-41.4: call check_alarms BEFORE draining event_buffer so that any newly
    // triggered alarms are captured in the drain and returned in event_lines.
    hp41_core::ops::time::alarm::check_alarms(calc);
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines, event_lines))
}

/// Tauri command: step the program counter forward by 1 (SST — Single Step).
/// Mirrors HP-41 hardware: pc stays at program.len(), no wrap-around.
#[tauri::command]
pub fn sst_step(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_sst(&mut calc)
}

/// Tauri command: step the program counter backward by 1 (BST — Back Step).
/// Mirrors HP-41 hardware: pc stays at 0, no underflow.
#[tauri::command]
pub fn bst_step(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_bst(&mut calc)
}

/// Tauri command: toggle program run/stop state (R/S key).
///
/// Mirrors sst_step/bst_step shape — never goes through dispatch_op because
/// R/S is not a single Op variant (it toggles CalcState.is_running).
/// Toggles the flag only; no run loop is spawned here (the IPC thread cannot
/// block on a run loop, so actual stepping requires a separate tick thread).
#[tauri::command]
pub fn run_stop(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_run_stop(&mut calc)
}

/// Tauri command: flip the cancellation flag for long-running Math Pac I ops.
///
/// ## CRITICAL — no AppState lock (Pitfall 1 / deadlock avoidance)
///
/// `request_cancel` takes `State<'_, CancelFlag>` (a separate managed state),
/// NOT `State<'_, AppState>`. This is intentional and MUST NOT be changed:
/// `dispatch_op` holds the AppState Mutex for the entire duration of
/// `op_integ` / `op_solve` / `op_difeq`. If this thunk tried to lock AppState,
/// it would deadlock (RESEARCH.md §"AppState Mutex + AtomicBool interleaving",
/// lines 927-1029).
///
/// The `CancelFlag` Arc is the SAME `Arc<AtomicBool>` as `CalcState.cancel_requested`
/// — cloned out at setup() time (lib.rs) before the CalcState was wrapped in the Mutex.
/// The solver loops read it via Relaxed loads every 64 samples (D-28.7 / D-28.8).
///
/// Idempotent: safe to call when no long op is running — the next INTG/SOLVE/DIFEQ
/// entry resets the flag to false (Plan 31-02 Task 3 surgical hp41-core exception).
#[tauri::command]
pub fn request_cancel(cancel: State<'_, CancelFlag>) -> Result<(), GuiError> {
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

/// Tauri command: submit a numeric R/S input to the currently active modal workflow.
///
/// Calls `hp41_core::ops::math1::submit_modal` which flushes `entry_buf` then
/// advances the modal state machine. Returns `GuiError` if no modal is active
/// (InvalidOp) or on numerical errors during the modal step.
///
/// Phase 31 Plan 03 / D-25.6: 4-line glue around shared hp41-core function.
/// SC-4 invariant: no calculator logic in hp41-gui/src-tauri/src/.
#[tauri::command]
pub fn submit_modal(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    hp41_core::ops::math1::submit_modal(&mut calc).map_err(GuiError::from)?;
    handle_get_state(&mut calc)
}

/// Tauri command: cancel the currently active modal workflow.
///
/// Calls `hp41_core::ops::math1::cancel_modal` which clears modal_program,
/// modal_prompt, and entry_buf. Always succeeds (no Result from core fn).
///
/// Phase 31 Plan 03 / D-25.6: 4-line glue around shared hp41-core function.
/// SC-4 invariant: no calculator logic in hp41-gui/src-tauri/src/.
#[tauri::command]
pub fn cancel_modal(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    hp41_core::ops::math1::cancel_modal(&mut calc); // no Result — always succeeds
    handle_get_state(&mut calc)
}

/// Tauri command: submit an alpha label to the currently active modal workflow.
///
/// Used for the FUNCTION NAME? prompt step in INTG/SOLVE/DIFEQ workflows (D-29.7).
/// The `label` parameter contains the XEQ-by-name function name the user typed
/// (CollectForModal mode in pending_input.ts — D-29.8 / D-29.9 mirror).
///
/// ## Parameter ordering: `label` precedes `state`
/// Tauri v2 convention puts custom params first, State extractor last.
/// `label` takes owned `String` (not `&str`) to avoid lifetime complications
/// in command-macro expansion (RESEARCH Assumption A8).
///
/// Phase 31 Plan 03 / D-25.6: 4-line glue around shared hp41-core function.
/// SC-4 invariant: no calculator logic in hp41-gui/src-tauri/src/.
#[tauri::command]
pub fn submit_modal_with_label(
    label: String,
    state: State<'_, AppState>,
) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    hp41_core::ops::math1::submit_modal_with_label(&mut calc, &label)
        .map_err(GuiError::from)?;
    handle_get_state(&mut calc)
}

/// Pure-Rust helper for sst_step — unit-testable without a Tauri runtime.
/// Advances pc by 1, capped at program.len() (no wrap-around, matching HP-41 hardware behavior).
pub fn handle_sst(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    if calc.pc < calc.program.len() {
        calc.pc += 1;
    }
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines, event_lines))
}

/// Pure-Rust helper for bst_step — decrements pc via saturating_sub, clamped at 0.
pub fn handle_bst(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    calc.pc = calc.pc.saturating_sub(1);
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines, event_lines))
}

/// Pure-Rust helper for run_stop — toggles `is_running`. Unit-testable
/// without a Tauri runtime. Flag-toggle only; no run loop spawned here.
pub fn handle_run_stop(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    calc.is_running = !calc.is_running;
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines, event_lines))
}

/// Tauri command: return the current GUI preferences (Phase 48 / INFRA-01).
///
/// Locks `PrefsState` (separate from `AppState`) with poisoned-lock recovery,
/// clones and returns the `GuiPrefs`. The frontend calls this once on startup
/// to initialize theme state (48-03 plan).
///
/// No CalcState involved — preference reads never touch calculator state (P59/THEME-05).
#[tauri::command]
pub fn get_prefs(prefs: State<'_, PrefsState>) -> GuiPrefs {
    prefs.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// Tauri command: validate and persist a single GUI preference by key/value.
///
/// Parameter ordering follows Tauri v2 convention: custom params (`key`, `value`)
/// precede the State extractor (`prefs`).
///
/// # Supported keys
/// - `"theme"`: one of `"dark"` | `"light"` | `"classic-beige"` | `"high-contrast"`.
///
/// Returns `Err(String)` for unknown keys or invalid theme values (T-48-01 threat mitigation).
/// Persists immediately to `~/.hp41/prefs.json` via `save_prefs` after updating in-memory state.
///
/// P59/THEME-05: never touches `CalcState` or `autosave.json`.
#[tauri::command]
pub fn set_pref(
    key: String,
    value: String,
    prefs: State<'_, PrefsState>,
) -> Result<(), String> {
    let mut p = prefs.lock().unwrap_or_else(|e| e.into_inner());
    match key.as_str() {
        "theme" => {
            if !VALID_THEMES.contains(&value.as_str()) {
                return Err(format!("unknown theme: {value}"));
            }
            p.theme = value;
        }
        // T-49-01: parse value as bool via string comparison — any non-"true" value is false
        // (safe coercion; unknown boolean strings silently become false rather than erroring,
        // which matches the HP-41 philosophy of safe fallbacks).
        "onboarding_done" => {
            p.onboarding_done = value == "true";
        }
        _ => return Err(format!("unknown pref key: {key}")),
    }
    save_prefs(&default_prefs_path(), &*p).map_err(|e| e.to_string())
}

/// Tauri command: persist the current CalcState to disk on demand (Ctrl+S / F5 in GUI).
///
/// Mirrors the CLI's Ctrl+S handler (`hp41-cli/src/app.rs` lines 324-330). Follows
/// CR-01 clone-under-lock pattern: clones the CalcState while holding the Mutex lock,
/// then releases the lock before performing disk I/O so the UI is never blocked longer
/// than necessary.
///
/// Uses `persistence::save_state` (module-qualified) to avoid name shadowing with
/// this function. Returns `Err(String)` on I/O failure (serialized to GuiError by Tauri).
///
/// GUI divergence (KBD-02 / D-49.13): F5 triggers save in the GUI; CLI uses F5 for
/// `run_program("A")`. This is intentional and documented — desktop keyboard idiom.
#[tauri::command]
pub fn save_state(state: State<'_, AppState>) -> Result<(), String> {
    // CR-01: clone under lock, then release lock before disk I/O.
    let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let path = persistence::default_state_path();
    persistence::save_state(&path, &snapshot).map_err(|e| e.to_string())
}

// ── Phase 50: .raw / .card.json file dialog I/O ──────────────────────────────

/// Structured info for a single program in a multi-program `.raw` archive.
/// Returned as part of `ImportRawResponse::Multi` so the frontend can render
/// a picker (RawPickerOverlay). Per D-50.4 / D-50.5.
#[derive(Debug, Serialize)]
pub struct MultiProgramInfo {
    /// Human-readable label: "LBL NAME (N bytes)" or "Program N (N bytes)".
    pub label: String,
    /// Zero-based index in the decoded archive (passed back to import_selected_programs).
    pub index: usize,
    /// Byte count of this program's segment (informational; also in label).
    pub byte_len: usize,
}

/// Response variants for `import_raw_dialog`.
///
/// - `Single`: exactly one program found — imported immediately, view + message returned.
/// - `Multi`: 2+ programs found — NOT imported; returns metadata for the frontend picker.
/// - `Cancelled`: user dismissed the dialog — current state returned unchanged.
/// - `Empty`: file contains no programs (e.g., empty file or only END markers).
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ImportRawResponse {
    Single {
        view: CalcStateView,
        message: String,
    },
    Multi {
        programs: Vec<MultiProgramInfo>,
        file_path: String,
    },
    Cancelled {
        view: CalcStateView,
    },
    Empty {
        view: CalcStateView,
        message: String,
    },
}

/// Tauri command: open a native OS file dialog, decode the selected `.raw` file,
/// and import the program(s) into calculator memory.
///
/// Anti-deadlock pattern (RESEARCH Pitfall 1): dialog and file I/O happen BEFORE
/// any AppState lock is acquired. AppState is only locked for state mutation.
///
/// Returns:
/// - `Single` — one program decoded + inserted; ready for frontend to display.
/// - `Multi` — 2+ programs; frontend must show picker; call import_selected_programs next.
/// - `Cancelled` — user dismissed dialog; current state returned.
/// - `Empty` — file had no programs.
#[tauri::command]
pub fn import_raw_dialog(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ImportRawResponse, GuiError> {
    // Phase 1 (no lock): open file dialog
    let file_result = app
        .dialog()
        .file()
        .set_title("Import HP-41 Program")
        .add_filter("HP-41 Program", &["raw"])
        .add_filter("All Files", &["*"])
        .blocking_pick_file();

    let Some(file_path) = file_result else {
        // User cancelled
        let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
        let view = handle_get_state(&mut calc)?;
        return Ok(ImportRawResponse::Cancelled { view });
    };

    // Phase 2 (no lock): file I/O + decode
    let path_buf = file_path
        .into_path()
        .map_err(|e| GuiError { message: format!("path error: {e}") })?;

    // T-50-04: validate file_path is a regular file before reading
    if !path_buf.is_file() {
        return Err(GuiError {
            message: format!(
                "not a regular file: {}",
                path_buf.display()
            ),
        });
    }

    let bytes = std::fs::read(&path_buf)
        .map_err(|e| GuiError { message: format!("io: read failed: {e}") })?;

    // T-50-07: decode_all_programs returns HpError::CardData on malformed input
    let programs = decode_all_programs(&bytes).map_err(GuiError::from)?;

    match programs.len() {
        0 => {
            // Empty file or only whitespace — no programs to import
            let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
            let view = handle_get_state(&mut calc)?;
            Ok(ImportRawResponse::Empty {
                view,
                message: "No programs found in file".to_string(),
            })
        }
        1 => {
            // Phase 3 (lock for state mutation): single program — import immediately
            let decoded = programs.into_iter().next().expect("len == 1");
            let ops_count = decoded.ops.len();
            let label = picker_label(0, &decoded.ops, decoded.byte_len);
            let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
            insert_program_ops(&mut calc, decoded.ops);
            let view = handle_get_state(&mut calc)?;
            Ok(ImportRawResponse::Single {
                view,
                message: format!("Imported {} ({} steps)", label, ops_count),
            })
        }
        _ => {
            // Multi-program: return metadata for the frontend picker; DO NOT import yet
            let programs_info: Vec<MultiProgramInfo> = programs
                .iter()
                .enumerate()
                .map(|(idx, p)| MultiProgramInfo {
                    label: picker_label(idx, &p.ops, p.byte_len),
                    index: idx,
                    byte_len: p.byte_len,
                })
                .collect();
            let file_path_str = path_buf.to_string_lossy().to_string();
            Ok(ImportRawResponse::Multi {
                programs: programs_info,
                file_path: file_path_str,
            })
        }
    }
}

/// Tauri command: import user-selected programs from a previously decoded `.raw` archive.
///
/// Called by the frontend picker after `import_raw_dialog` returns `Multi`.
/// Re-reads and re-decodes the file (fast; avoids caching `Vec<Op>` in IPC).
/// Inserts each selected program sequentially via `insert_program_ops`.
///
/// Parameter ordering: `file_path` and `indices` precede `state` (Tauri v2 convention).
///
/// T-50-04 mitigation: validates that `file_path` is an absolute path to a regular file.
#[tauri::command]
pub fn import_selected_programs(
    file_path: String,
    indices: Vec<usize>,
    state: State<'_, AppState>,
) -> Result<CalcStateView, GuiError> {
    // Phase 1 (no lock): validate + file I/O + decode
    let path_buf = std::path::Path::new(&file_path);

    // T-50-04: reject non-absolute or non-file paths to prevent path traversal
    if !path_buf.is_absolute() {
        return Err(GuiError {
            message: format!("expected an absolute path, got: {file_path}"),
        });
    }
    if !path_buf.is_file() {
        return Err(GuiError {
            message: format!("not a regular file: {file_path}"),
        });
    }

    let bytes = std::fs::read(path_buf)
        .map_err(|e| GuiError { message: format!("io: read failed: {e}") })?;
    let programs = decode_all_programs(&bytes).map_err(GuiError::from)?;

    // Validate all selected indices BEFORE mutating, so a stale/out-of-range
    // index fails the whole import instead of silently dropping a selection
    // (the frontend toast would otherwise claim success for a partial import).
    if let Some(&bad) = indices.iter().find(|&&idx| idx >= programs.len()) {
        return Err(GuiError {
            message: format!(
                "import index out of range: {bad} (file has {} programs)",
                programs.len()
            ),
        });
    }

    // Phase 2 (lock for state mutation): insert selected programs sequentially
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    for idx in &indices {
        if let Some(decoded) = programs.get(*idx) {
            insert_program_ops(&mut calc, decoded.ops.clone());
        }
    }
    drop(programs); // explicit: allow early free before view build

    handle_get_state(&mut calc)
}

/// Tauri command: export the current program to a user-chosen `.raw` file via native
/// OS save dialog.
///
/// Anti-deadlock pattern (RESEARCH Pitfall 1): snapshot program under lock, then
/// release lock BEFORE opening the save dialog (dialog is blocking + can take long).
/// File write happens after dialog returns, with no lock held.
#[tauri::command]
pub fn export_raw_dialog(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, GuiError> {
    // Phase 1 (brief lock): snapshot program bytes — released before dialog
    let encoded = {
        let calc = state.lock().unwrap_or_else(|e| e.into_inner());
        encode_program(&calc.program).map_err(GuiError::from)?
    };

    // Phase 2 (no lock): open save dialog
    let save_result = app
        .dialog()
        .file()
        .set_title("Export HP-41 Program")
        .set_file_name("program.raw")
        .add_filter("HP-41 Program", &["raw"])
        .add_filter("All Files", &["*"])
        .blocking_save_file();

    let Some(save_path) = save_result else {
        return Ok(serde_json::json!({"cancelled": true}));
    };

    // Phase 3 (no lock): write to disk
    let path_buf = save_path
        .into_path()
        .map_err(|e| GuiError { message: format!("path error: {e}") })?;

    std::fs::write(&path_buf, &encoded)
        .map_err(|e| GuiError { message: format!("io: write failed: {e}") })?;

    let filename = path_buf
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "program.raw".to_string());

    Ok(serde_json::json!({ "message": format!("Saved {}", filename) }))
}

/// Tauri command: open a native OS file dialog, decode the selected `.card.json` file,
/// and load the data card into calculator registers.
///
/// Handles `.card.json` files per D-50.7. Anti-deadlock pattern: dialog + decode
/// happen BEFORE AppState lock.
#[tauri::command]
pub fn import_data_dialog(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, GuiError> {
    // Phase 1 (no lock): open file dialog
    let file_result = app
        .dialog()
        .file()
        .set_title("Import HP-41 Data Card")
        .add_filter("HP-41 Data Card", &["json"])
        .add_filter("All Files", &["*"])
        .blocking_pick_file();

    let Some(file_path) = file_result else {
        let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
        let view = handle_get_state(&mut calc)?;
        return Ok(serde_json::json!({ "view": view, "cancelled": true }));
    };

    // Phase 2 (no lock): file I/O + decode
    let path_buf = file_path
        .into_path()
        .map_err(|e| GuiError { message: format!("path error: {e}") })?;

    if !path_buf.is_file() {
        return Err(GuiError {
            message: format!("not a regular file: {}", path_buf.display()),
        });
    }

    let bytes = std::fs::read(&path_buf)
        .map_err(|e| GuiError { message: format!("io: read failed: {e}") })?;

    let card = decode_data(&bytes).map_err(GuiError::from)?;

    // Phase 3 (lock for state mutation): load data card
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    load_data_card(&mut calc, card);
    let view = handle_get_state(&mut calc)?;

    Ok(serde_json::json!({
        "view": view,
        "message": "Loaded data card"
    }))
}

/// Tauri command: export current data registers to a user-chosen `.card.json` file via
/// native OS save dialog.
///
/// Per D-50.7. Anti-deadlock pattern: snapshot data under lock, release, then open dialog.
#[tauri::command]
pub fn export_data_dialog(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, GuiError> {
    // Phase 1 (brief lock): snapshot data card — released before dialog
    let encoded = {
        let calc = state.lock().unwrap_or_else(|e| e.into_inner());
        let card = capture_data_card(&calc);
        encode_data(&card).map_err(GuiError::from)?
    };

    // Phase 2 (no lock): open save dialog
    let save_result = app
        .dialog()
        .file()
        .set_title("Export HP-41 Data Card")
        .set_file_name("data.card.json")
        .add_filter("HP-41 Data Card", &["json"])
        .add_filter("All Files", &["*"])
        .blocking_save_file();

    let Some(save_path) = save_result else {
        return Ok(serde_json::json!({"cancelled": true}));
    };

    // Phase 3 (no lock): write to disk
    let path_buf = save_path
        .into_path()
        .map_err(|e| GuiError { message: format!("path error: {e}") })?;

    std::fs::write(&path_buf, &encoded)
        .map_err(|e| GuiError { message: format!("io: write failed: {e}") })?;

    let filename = path_buf
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "data.card.json".to_string());

    Ok(serde_json::json!({ "message": format!("Saved {}", filename) }))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use hp41_core::ops::{dispatch, Op};
    use hp41_core::HpNum;

    #[test]
    fn test_dispatch_op_unknown_key() {
        // SC-2: unknown key_id returns GuiError, NEVER panics, NEVER silently discards.
        let mut calc = CalcState::new();
        let result = handle_op(&mut calc, "totally_unknown_xyz");
        let err = result.expect_err("unknown key must produce Err(GuiError)");
        assert!(
            err.message.contains("unknown key"),
            "expected 'unknown key' in error message, got: {}",
            err.message
        );
    }

    #[test]
    fn test_print_buffer_drained() {
        // SC-3: After a command that produces print output, the print_buffer is empty
        // AND the returned view.print_lines contains the produced lines.
        let mut calc = CalcState::new();
        calc.stack.x = HpNum::from(42);
        // Ensure the buffer has at least one line via direct dispatch (sanity).
        dispatch(&mut calc, Op::PRX).unwrap();
        assert!(
            !calc.print_buffer.is_empty(),
            "PRX should populate print_buffer"
        );
        // Now exercise handle_op via the "prx" key — Wave 1 will produce another line
        // (or we'd have to reset buffer; we instead test the drain via handle_get_state).
        calc.print_buffer.clear();
        calc.stack.x = HpNum::from(7);
        dispatch(&mut calc, Op::PRX).unwrap();
        let pre_drain_lines = calc.print_buffer.len();
        assert_eq!(pre_drain_lines, 1, "second PRX should add 1 line");

        // Wave 1: handle_get_state must drain the buffer and put lines in the view.
        let view = handle_get_state(&mut calc).unwrap();
        assert!(
            calc.print_buffer.is_empty(),
            "print_buffer must be empty after handle_get_state drain"
        );
        assert_eq!(
            view.print_lines.len(),
            1,
            "view.print_lines should contain the drained line"
        );
    }

    /// PR #5 review (pr-test-analyzer) — the existing test_print_buffer_drained
    /// only exercises handle_get_state. The handle_op path (the most-trafficked
    /// dispatch_op IPC route) was never directly asserted to drain print_buffer
    /// AND populate view.print_lines. A regression that skipped the drain in
    /// handle_op would silently break the entire GUI print panel without any
    /// test signal.
    #[test]
    fn test_handle_op_drains_print_buffer_via_dispatch() {
        // PRX
        let mut calc = CalcState::new();
        calc.stack.x = HpNum::from(7);
        let view = handle_op(&mut calc, "prx").expect("handle_op prx must succeed");
        assert!(
            calc.print_buffer.is_empty(),
            "handle_op('prx') must drain print_buffer (was: {} lines)",
            calc.print_buffer.len()
        );
        assert_eq!(
            view.print_lines.len(),
            1,
            "view.print_lines must contain the PRX line after dispatch"
        );

        // PRA — exercises a different op_* helper through the same drain path
        let mut calc2 = CalcState::new();
        calc2.alpha_reg = "HELLO".to_string();
        let view2 = handle_op(&mut calc2, "pra").expect("handle_op pra must succeed");
        assert!(calc2.print_buffer.is_empty());
        assert_eq!(view2.print_lines.len(), 1);

        // PRSTK — drains 6 lines in one call
        let mut calc3 = CalcState::new();
        let view3 = handle_op(&mut calc3, "prstk").expect("handle_op prstk must succeed");
        assert!(calc3.print_buffer.is_empty());
        assert_eq!(
            view3.print_lines.len(),
            6,
            "PRSTK must drain all 6 lines (T/Z/Y/X/LASTX/ALPHA) into the view"
        );
    }

    #[test]
    fn test_eex_chs_toggles_exponent_sign() {
        // Wave 0 RED test: "eex_chs" key_id must toggle the exponent sign in entry_buf.
        // entry_buf "1e2" → "1e-2" on first call → "1e2" on second call.
        let mut calc = CalcState::new();
        calc.entry_buf = "1e2".to_string();
        handle_op(&mut calc, "eex_chs").unwrap();
        assert_eq!(
            calc.entry_buf, "1e-2",
            "first eex_chs must insert minus sign"
        );
        handle_op(&mut calc, "eex_chs").unwrap();
        assert_eq!(
            calc.entry_buf, "1e2",
            "second eex_chs must remove minus sign"
        );
    }

    #[test]
    fn test_eex_chs_noop_without_e() {
        // Defensive: if no 'e' in entry_buf, eex_chs must return Ok without panic or error.
        let mut calc = CalcState::new();
        calc.entry_buf = "42".to_string();
        let result = handle_op(&mut calc, "eex_chs");
        assert!(
            result.is_ok(),
            "eex_chs with no 'e' must not error or panic"
        );
        assert_eq!(
            calc.entry_buf, "42",
            "entry_buf must be unchanged when no 'e' present"
        );
    }

    #[test]
    fn test_handle_sst_advances_pc() {
        use hp41_core::ops::Op;
        let mut calc = CalcState::new();
        calc.program = vec![Op::Add, Op::Enter];
        calc.pc = 0;
        handle_sst(&mut calc).unwrap();
        assert_eq!(calc.pc, 1, "SST must advance pc from 0 to 1");
    }

    #[test]
    fn test_handle_sst_clamps_at_end() {
        use hp41_core::ops::Op;
        let mut calc = CalcState::new();
        calc.program = vec![Op::Add];
        calc.pc = 1; // already at end (program.len() == 1)
        handle_sst(&mut calc).unwrap();
        assert_eq!(calc.pc, 1, "SST must not advance past program.len()");
    }

    #[test]
    fn test_handle_bst_decrements_pc() {
        use hp41_core::ops::Op;
        let mut calc = CalcState::new();
        calc.program = vec![Op::Add];
        calc.pc = 1;
        handle_bst(&mut calc).unwrap();
        assert_eq!(calc.pc, 0, "BST must decrement pc from 1 to 0");
    }

    #[test]
    fn test_handle_bst_clamps_at_zero() {
        let mut calc = CalcState::new();
        calc.pc = 0;
        handle_bst(&mut calc).unwrap();
        assert_eq!(calc.pc, 0, "BST must not underflow below 0");
    }

    #[test]
    fn test_handle_run_stop_toggles_is_running() {
        let mut calc = CalcState::new();
        assert!(
            !calc.is_running,
            "fresh CalcState must start with is_running == false"
        );
        handle_run_stop(&mut calc).unwrap();
        assert!(
            calc.is_running,
            "first run_stop must flip is_running to true"
        );
        handle_run_stop(&mut calc).unwrap();
        assert!(
            !calc.is_running,
            "second run_stop must flip is_running back to false"
        );
    }

    /// I5 regression: dispatching a card op through `handle_op_with_cards_dir`
    /// (the test-injected mirror of the production `dispatch_op` thunk) must
    /// drive the full three-phase path: dispatch → prepare → execute (real
    /// fs::write) → finalize. A regression that skipped phase 2 or any of
    /// the new helpers would leave `pending_card_op` populated AND no file
    /// on disk. Uses `xeq_WPRGM` — the canonical GUI key_id form per
    /// `key_map.rs` (card ops are XEQ-by-name, not bare named keys).
    #[test]
    fn test_handle_op_drives_full_card_op_via_thunk_path() {
        let tmp = tempfile::tempdir().unwrap();
        let mut calc = CalcState::new();
        calc.alpha_reg = "GUI_TEST".to_string();
        calc.program = vec![Op::Lbl("X".to_string()), Op::Rtn];

        let view = handle_op_with_cards_dir(&mut calc, "xeq_WPRGM", tmp.path())
            .expect("WPRGM via thunk path must succeed");

        assert!(
            calc.pending_card_op.is_none(),
            "thunk path must drain pending_card_op via prepare()"
        );
        assert!(
            tmp.path().join("GUI_TEST.raw").exists(),
            "thunk path must perform the actual fs::write"
        );
        // View still gets built (print_buffer drained, even when empty).
        assert_eq!(view.print_lines.len(), 0);
    }

    /// I5: a missing card file routed through the thunk path must surface
    /// `CardData` to the caller (Tauri serialises this as GuiError to the
    /// frontend) instead of silently succeeding.
    #[test]
    fn test_handle_op_card_read_error_routed_to_gui_error() {
        let tmp = tempfile::tempdir().unwrap();
        let mut calc = CalcState::new();
        calc.alpha_reg = "DOES_NOT_EXIST".to_string();

        let result = handle_op_with_cards_dir(&mut calc, "xeq_RDPRGM", tmp.path());
        let err = result.expect_err("missing card must produce GuiError");
        assert!(
            err.message.contains("card data") || err.message.contains("io: read"),
            "expected card-data diagnostic, got: {}",
            err.message
        );
        assert!(
            calc.pending_card_op.is_none(),
            "request must be cleared even on error so user is not locked out"
        );
    }

    /// I1 regression guard: handle_get_state must NOT touch pending_card_op
    /// and must NOT perform any disk I/O. If a future change re-introduces
    /// a defensive drain here, the assertion that the staged request is
    /// preserved verbatim will fail.
    #[test]
    fn test_handle_get_state_does_not_drain_pending_card_op() {
        use hp41_core::cardreader::CardOpRequest;
        let mut calc = CalcState::new();
        calc.pending_card_op = Some(CardOpRequest::WriteProgram {
            name: "STAGED".to_string(),
        });

        let _ = handle_get_state(&mut calc).unwrap();

        assert!(
            calc.pending_card_op.is_some(),
            "handle_get_state must leave pending_card_op untouched — drain belongs in dispatch_op"
        );
    }

    /// Smoke test for Op::PctChange through the Tauri command path.
    /// Y=100 (base), X=125 (new value) → %CH = (125-100)/100 * 100 = 25; Y preserved at 100.
    /// Seeds chosen to yield an exact integer result. Parses x_str/y_str back to Decimal
    /// so the comparison is independent of rust_decimal's trailing-zero scale.
    #[test]
    fn handle_op_pct_change_produces_expected_view() {
        use rust_decimal::Decimal;
        let mut state = CalcState::new();
        state.stack.y = HpNum::from(100i32);
        state.stack.x = HpNum::from(125i32);
        let view =
            handle_op(&mut state, "pct_change").expect("pct_change must dispatch successfully");

        let x_val: Decimal = view.x_str.parse().expect("x_str must parse as Decimal");
        let y_val: Decimal = view.y_str.parse().expect("y_str must parse as Decimal");
        assert_eq!(x_val, Decimal::from(25), "%CH(100→125) must be 25");
        assert_eq!(y_val, Decimal::from(100), "Y must be preserved at 100");
    }

    /// Phase 41 D-41.4: handle_tick_time must drain event_buffer and return it in the view.
    /// Verifies the alarm event flow: push event → tick → view contains event + buffer empty.
    #[test]
    fn handle_tick_time_drains_event_buffer_and_returns_view() {
        let mut calc = CalcState::new();
        calc.event_buffer
            .push("alarm:message:TEST ALARM".to_string());

        let view = handle_tick_time(&mut calc).expect("handle_tick_time must succeed");

        assert!(
            calc.event_buffer.is_empty(),
            "event_buffer must be empty after handle_tick_time drains it; got: {:?}",
            calc.event_buffer
        );
        assert_eq!(
            view.event_buffer,
            vec!["alarm:message:TEST ALARM"],
            "view.event_buffer must contain the drained alarm event"
        );
    }

    /// Phase 41 D-41.3: handle_tick_time must project clock_active from CalcState.
    /// Verifies the live-display trigger field flows through correctly.
    #[test]
    fn handle_tick_time_returns_clock_active_field() {
        let mut calc = CalcState::new();
        calc.clock_active = true;

        let view = handle_tick_time(&mut calc).expect("handle_tick_time must succeed");

        assert!(
            view.clock_active,
            "view.clock_active must mirror CalcState.clock_active (true)"
        );
    }
}
