//! Phase 3 programming engine: LBL, GTO, XEQ, RTN, PRGM, Test, ISG, DSE, run_program.
//!
//! All programming ops have LiftEffect: Neutral (they do not modify lift_enabled).
//! run_program() is the public interpreter entry point exported via lib.rs.
//!
//! Key design constraints:
//!   - run_program() clones state.program to avoid a borrow conflict: dispatch() needs &mut CalcState
//!     while iterating over state.program (D-06)
//!   - execute_op() is a private helper that does NOT call flush_entry_buf — the entry buffer
//!     must not be reset mid-execution
//!   - ISG/DSE parse counters by string-split, never floor()/fmod() (ADR-001, D-10)

use rust_decimal::Decimal;
use std::str::FromStr;

use crate::error::HpError;
use crate::num::HpNum;
use crate::ops::math1::xrom::{XromModule, ADV_MATH_A, ADV_MATH_B, MATH_1, STAT_1, TIME_MODULE};
use crate::ops::{Op, TestKind};
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;

// ── Public op dispatch functions ─────────────────────────────────────────────
// Called from dispatch() match arms (added in plan 03-06).

/// LBL: no-op during interactive execution — a label is a marker only.
/// LiftEffect: Neutral.
pub fn op_lbl(_state: &mut CalcState) -> Result<(), HpError> {
    // LBL is a recording marker; executing it interactively is a no-op.
    // Inside run_loop, Lbl arms are handled directly (also no-op).
    Ok(())
}

/// PrgmMode: enter PRGM recording mode (toggle enter path).
/// The exit path (prgm_mode=true + Op::PrgmMode) is handled in dispatch() gate directly.
/// LiftEffect: Neutral.
pub fn op_prgm_mode(state: &mut CalcState) -> Result<(), HpError> {
    state.prgm_mode = true;
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// GTO: unconditional branch to label. Only meaningful when is_running.
/// Interactive GTO outside a running program → InvalidOp; HP-41 supports interactive GTO,
/// but this emulator keeps the interactive dispatch path simple by not implementing it.
/// LiftEffect: Neutral.
pub fn op_gto(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    if !state.is_running {
        return Err(HpError::InvalidOp);
    }
    let target = find_label_in_state(state, label)?;
    state.pc = target + 1; // execute step AFTER the Lbl marker (Pitfall 4)
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// XEQ: subroutine call. Enforces 4-level call stack limit (D-14).
///
/// Interactive XEQ (not running): tries the four-entry Card Reader
/// XEQ-by-name fallback before erroring. This is the
/// path the GUI uses — `dispatch(Op::Xeq("WPRGM"))` with `is_running=false`.
///
/// Programmatic XEQ inside `run_loop` is handled there directly (with
/// call-depth check + user-label scan + same builtin fallback) — this
/// function is never reached during program execution.
///
/// LiftEffect: Neutral.
pub fn op_xeq(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    if !state.is_running {
        // Built-in XEQ-by-name fallback for Card Reader ops + conditional tests.
        // No user-label scan here: user-program XEQ goes through run_loop,
        // not op_xeq. If a user wants to call their own LBL interactively
        // they use run_program(state, label) directly, not dispatch.
        if let Some(card_op) = builtin_card_op(label) {
            return crate::ops::dispatch(state, card_op);
        }
        // Phase 28 (v3.0) XROM resolver — fires LAST (C-28.4 / Pitfall 1).
        // Checked after builtin_card_op so built-in names always win.
        // xeq_by_name_local_resolve (hp41-cli) is the third call site — Phase 29 / CLI-01.
        if let Some(xrom_op) = crate::ops::math1::xrom::xrom_resolve(label, state.xrom_modules) {
            return crate::ops::dispatch(state, xrom_op);
        }
        return Err(HpError::InvalidOp);
    }
    // run_loop handles Op::Xeq directly. Reaching here while running is a
    // logic bug elsewhere — return InvalidOp as a safe guard.
    Err(HpError::InvalidOp)
}

/// RTN: return from subroutine. If call_stack is empty, terminates run (top-level RTN).
/// Interactive RTN when not running: no-op (call_stack is empty, nothing to pop).
/// LiftEffect: Neutral.
pub fn op_rtn(state: &mut CalcState) -> Result<(), HpError> {
    if let Some(return_pc) = state.call_stack.pop() {
        state.pc = return_pc;
    }
    // Empty call_stack = top-level RTN — run_loop breaks on next iteration.
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// Test: interactive dispatch arm — no-op (read-only conditional; result only meaningful inside run_loop).
/// Inside run_loop, Test is handled directly using evaluate_test().
/// LiftEffect: Neutral.
pub fn op_test(state: &mut CalcState, _kind: TestKind) -> Result<(), HpError> {
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// ISG n: increment register n by step; return true if loop should skip (new_current > final).
/// Uses string-split parsing per ADR-001 — never floor()/fmod() on f64.
/// LiftEffect: Neutral.
pub fn op_isg(state: &mut CalcState, reg: u8) -> Result<bool, HpError> {
    if reg as usize >= state.regs.len() {
        return Err(HpError::InvalidOp);
    }
    let (current, final_val, step, frac_padded) =
        parse_counter(&state.regs[reg as usize].numeric_or_zero())?;
    let new_current = current + step;
    state.regs[reg as usize] = build_counter(new_current, &frac_padded)?.into();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(new_current > final_val) // true = skip next (loop exits, D-11)
}

/// DSE n: decrement register n by step; return true if loop should skip (new_current <= final).
/// LiftEffect: Neutral.
pub fn op_dse(state: &mut CalcState, reg: u8) -> Result<bool, HpError> {
    if reg as usize >= state.regs.len() {
        return Err(HpError::InvalidOp);
    }
    let (current, final_val, step, frac_padded) =
        parse_counter(&state.regs[reg as usize].numeric_or_zero())?;
    let new_current = current - step;
    state.regs[reg as usize] = build_counter(new_current, &frac_padded)?.into();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(new_current <= final_val) // true = skip next (loop exits, D-11)
}

// ── Phase 22: Program editing primitives (D-22.7, D-22.8, D-22.9, D-22.10) ──
// CLP / DEL / INS are PRGM-mode editing primitives — they mutate state.program
// directly and NEVER get recorded into the program buffer. The dispatch gate
// in mod.rs special-cases them so they execute immediately while prgm_mode == true.
//
// Stubs land here in task 22-02-01 so the workspace compiles after the variants
// are added. Real bodies fill in over the next two tasks (22-02-02 / 22-02-03).

/// Phase 22 D-22.7 (FN-PROG-03). Clear program from `Op::Lbl("label")` to the
/// next `Op::Lbl(_)` (or end-of-Vec if no further LBL).
///
/// Cursor reposition (Pitfall 6): after the drain, `state.pc` is set to `start`
/// (clamped to the post-drain `state.program.len()`) so the cursor lands at the
/// start of whatever block was deleted. Missing label → `HpError::InvalidOp`.
/// PRGM-mode only (D-22.10) — interactive dispatch with `prgm_mode == false`
/// returns InvalidOp via the defense-in-depth guard.
///
/// Documented divergence: HP-41 hardware uses END/.END. markers; we use
/// next-LBL boundaries because the flat-Vec program model has no explicit
/// END marker. (RESEARCH §1 D-22.7 row, OQ resolved as Option B.)
pub fn op_clp(state: &mut CalcState, label: &str) -> Result<(), HpError> {
    if !state.prgm_mode {
        return Err(HpError::InvalidOp);
    }
    let start = state
        .program
        .iter()
        .position(|op| matches!(op, Op::Lbl(n) if n == label))
        .ok_or(HpError::InvalidOp)?;
    let end = state
        .program
        .iter()
        .skip(start + 1)
        .position(|op| matches!(op, Op::Lbl(_)))
        .map(|i| start + 1 + i)
        .unwrap_or(state.program.len());
    state.program.drain(start..end);
    // Pitfall 6: cursor lands at start of deleted block, clamped to new len
    // (protects against the rare case where start == post-drain program.len()).
    state.pc = start.min(state.program.len());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// Phase 22 D-22.9 (FN-PROG-04). Delete `nnn` program steps starting at
/// `state.pc`. `nnn` silently clamps to `min(nnn, program.len() - pc)`;
/// `nnn == 0` OR `pc == program.len()` → no-op. PRGM-mode only (D-22.10).
///
/// `state.pc` is UNCHANGED — drain shifts the trailing tail down to fill the
/// gap, so the cursor naturally points at the next surviving step.
pub fn op_del(state: &mut CalcState, nnn: u8) -> Result<(), HpError> {
    if !state.prgm_mode {
        return Err(HpError::InvalidOp);
    }
    // D-22.9 clamping: saturating_sub guards against the pathological pc > len
    // (shouldn't happen, but keeps the helper bounds-safe under any state).
    let n = (nnn as usize).min(state.program.len().saturating_sub(state.pc));
    if n == 0 {
        // No-op for nnn == 0 OR pc == program.len()
        apply_lift_effect(state, LiftEffect::Neutral);
        return Ok(());
    }
    state.program.drain(state.pc..state.pc + n);
    // state.pc deliberately unchanged: drain shifts the tail down so pc
    // naturally falls at the same index (which is the post-drain position).
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// Phase 22 D-22.8 (FN-PROG-05). Insert `Op::Null` (no-op placeholder from
/// Phase 12) at `state.pc`. PRGM-mode only (D-22.10).
///
/// `state.pc` is UNCHANGED — the cursor still points at the freshly inserted
/// Null. This matches HP-41 hardware "INS lands a blank step at cursor" behavior.
pub fn op_ins(state: &mut CalcState) -> Result<(), HpError> {
    if !state.prgm_mode {
        return Err(HpError::InvalidOp);
    }
    // Defense-in-depth (D-22.23 zero-panic): clamp `state.pc` to
    // `state.program.len()` before `Vec::insert`, which would otherwise
    // panic when `index > len()`. Under normal control flow `state.pc <=
    // state.program.len()` always holds (CLP/DEL/run_loop maintain it),
    // but a corrupted `~/.hp41/autosave.json` could deserialize a
    // `CalcState` with `pc > program.len()`. Silent clamp mirrors the
    // existing `op_del` `saturating_sub` / `.min(...)` neutralization.
    let idx = state.pc.min(state.program.len());
    state.program.insert(idx, Op::Null);
    // state.pc deliberately unchanged — cursor still on the new Null.
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

// ── Phase 22: Memory management (D-22.11) ────────────────────────────────────

/// Phase 22 D-22.14 / FN-MEM-03. Zero stack X/Y/Z/T.
///
/// PRESERVES `state.stack.lastx` AND `state.stack.lift_enabled` — verified by
/// the absence of any assignment to these fields in this helper body.
/// `Neutral` `apply_lift_effect` is a no-op for `lift_enabled` (see
/// `stack.rs::apply_lift_effect`), so the trailing apply_lift_effect call
/// also preserves the flag.
///
/// Sentinel test `test_clst_preserves_lastx_and_lift_enabled` in
/// `hp41-core/tests/phase22_memory_ops.rs` enforces both invariants.
///
/// LiftEffect: Neutral.
pub fn op_clst(state: &mut CalcState) -> Result<(), HpError> {
    state.stack.x = crate::num::HpNum::zero();
    state.stack.y = crate::num::HpNum::zero();
    state.stack.z = crate::num::HpNum::zero();
    state.stack.t = crate::num::HpNum::zero();
    // lastx UNTOUCHED (D-22.14)
    // lift_enabled UNTOUCHED (Neutral lift does not modify it — stack.rs)
    crate::stack::apply_lift_effect(state, crate::stack::LiftEffect::Neutral);
    Ok(())
}

/// Phase 22 D-22.11 / FN-MEM-01. Resize `state.regs` to `nnn` slots.
///
/// OQ-2 (AMENDED 2026-05-14): `nnn == 0` silently clamps to 1 (documented
/// divergence from real HP-41 which accepts `SIZE 000`). `nnn > 319`
/// returns `HpError::InvalidOp`. Otherwise `state.regs.resize(target,
/// crate::num::HpValue::default())`: shrinking truncates the tail (hardware-faithful
/// "MEM LOST"); growing zero-fills the new slots. Preserves values where
/// the old and new ranges overlap.
///
/// SAFETY: every legacy register access (op_sto/op_rcl/op_sto_arith/op_view/
/// op_clreg/Σ-family) was audited in 22-03-01..03 to honor `state.regs.len()`
/// dynamically, so shrinking via SIZE will NOT panic. The Σ-family
/// additionally fails closed when `state.regs.len() < 7` (Pitfall 5).
///
/// LiftEffect: Neutral.
pub fn op_size(state: &mut CalcState, nnn: u16) -> Result<(), HpError> {
    if nnn > 319 {
        return Err(HpError::InvalidOp);
    }
    let target = nnn.max(1) as usize; // OQ-2: SIZE 0 → silently clamp to 1
    state.regs.resize(target, crate::num::HpValue::default());
    // Phase 23 D-23.4 (WR-01): drop text_regs entries that now point past
    // end-of-regs so a shrink-then-grow cycle cannot resurrect a stale text
    // shadow. CONTEXT.md D-23.4's audit inventory listed STO/STO-arith/CLREG
    // but missed op_size; this retain step closes the gap and keeps the
    // sidecar from leaking across SIZE cycles or autosave round-trips.
    state.text_regs.retain(|&k, _| (k as usize) < target);
    crate::stack::apply_lift_effect(state, crate::stack::LiftEffect::Neutral);
    Ok(())
}

/// Phase 22 D-22.16 (AMENDED OQ-1 Option B, FN-MEM-05). Hardware-faithful CATALOG.
///
/// `n == 0` OR `n >= 5` → `HpError::InvalidOp`. Output writes to
/// `state.print_buffer` (Phase 11 drain channel) with 24-char width:
///   - Header: `-- CATALOG n --` (left-padded to 24).
///   - Payload:
///     * CAT 1 (programs): one `LBL <name>  <steps>` line per `Op::Lbl(_)` in
///       `state.program`. Step count = distance to next LBL or program.len()
///       for the last labelled block. Long names truncated to 9 chars so the
///       full line stays within 24 chars (4 + 9 + 2 + 5 = 20, padded to 24).
///       Empty program → zero payload lines (header + footer only).
///     * CAT 2 (XROM modules), CAT 3 (HP-IL), CAT 4 (peripherals) — none in
///       this emulator → single 24-char "NOT AVAILABLE" line.
///   - Footer: `-- END --` (left-padded to 24).
///
/// LiftEffect: Neutral.
pub fn op_catalog(state: &mut CalcState, n: u8) -> Result<(), HpError> {
    if n == 0 || n >= 5 {
        return Err(HpError::InvalidOp);
    }
    state
        .print_buffer
        .push(format!("{:<24}", format!("-- CATALOG {n} --")));
    match n {
        1 => {
            // CATALOG 1: programs (hardware-faithful, OQ-1 Option B).
            // Collect (position, name) per Op::Lbl entry.
            let labels: Vec<(usize, String)> = state
                .program
                .iter()
                .enumerate()
                .filter_map(|(i, op)| match op {
                    Op::Lbl(nm) => Some((i, nm.clone())),
                    _ => None,
                })
                .collect();
            for (idx, (pos, name)) in labels.iter().enumerate() {
                let end = labels
                    .get(idx + 1)
                    .map(|(p, _)| *p)
                    .unwrap_or(state.program.len());
                let steps = end - pos;
                // Truncate long names to 9 chars so the full LBL line stays
                // within 24 chars: "LBL " (4) + name :9 + "  " (2) + steps :5 = 20.
                let display_name: String = name.chars().take(9).collect();
                state.print_buffer.push(format!(
                    "{:<24}",
                    format!("LBL {display_name:9}  {steps:5}")
                ));
            }
        }
        2 => {
            // CATALOG 2: XROM modules loaded in this emulator.
            // Phase 41 D-41.5: generic 3-module loop replaces the previous
            // parallel if-blocks (D-36.1 sibling block pattern). The loop
            // correctly handles any combination of modules and eliminates the
            // latent else-if bug from the 2-module era. Phase 46: 5 XROM modules
            // in registry (Advantage Pac ADV_MATH_A + ADV_MATH_B added).
            // Instant-scroll per W1 fix: single-pass synchronous push into
            // print_buffer — NO PSE-step, NO per-line yield (v2.2 CAT 1 shape;
            // D-31.12/D-31.14 PSE-step deferred per RESEARCH Open Q2).
            let xrom_registry: &[(&XromModule, u8)] = &[
                (&MATH_1, 0b0000_0001),      // bit 0 — Math Pac I (XROM 7)
                (&STAT_1, 0b0000_0010),      // bit 1 — Stat 1 Pac (XROM 2)
                (&TIME_MODULE, 0b0000_0100), // bit 2 — Time Module (XROM 26)
                (&ADV_MATH_A, 0b0000_1000),  // bit 3 — Advantage Pac ADV CONV+MTRX (XROM 22)
                (&ADV_MATH_B, 0b0001_0000),  // bit 4 — Advantage Pac ADV MATH+TVM (XROM 24)
            ];
            let any_module = xrom_registry
                .iter()
                .any(|(_, bit)| state.xrom_modules & bit != 0);
            if !any_module {
                // NO XROM: no modules loaded (defensive; post-migrate_after_load
                // the default is 0b0000_0111 but save files may have xrom_modules=0).
                state.print_buffer.push(format!("{:<24}", "NO XROM"));
            } else {
                for (module, bit) in xrom_registry {
                    if state.xrom_modules & bit != 0 {
                        state.print_buffer.push(format!(
                            "{:<24}",
                            format!("XROM {} {}", module.id, module.name)
                        ));
                        for (name, _op) in module.ops {
                            state.print_buffer.push(format!("{name:<24}"));
                        }
                    }
                }
            }
        }
        3..=4 => {
            // CATALOG 3 (HP-IL) / 4 (peripherals) — not emulated.
            state.print_buffer.push(format!("{:<24}", "NOT AVAILABLE"));
        }
        _ => return Err(HpError::InvalidOp), // defensive; guarded above
    }
    state.print_buffer.push(format!("{:<24}", "-- END --"));
    crate::stack::apply_lift_effect(state, crate::stack::LiftEffect::Neutral);
    Ok(())
}

/// Phase 22 D-22.18 (AMENDED OQ-3 Option A, FN-KEY-01). ASN key assignment.
///
/// If `name` is empty: removes assignment for `key_code` via
/// `state.assignments.remove(&key_code)` — silent no-op if absent.
/// Otherwise: `state.assignments.insert(key_code, name)`.
///
/// `key_code` uses HP-41 row×10+col encoding (1-indexed; same as
/// `last_key_code` and `keycode_to_hp41_code`).
///
/// Late-binding resolution (parse-as-Op vs LBL search) happens at USER-mode
/// dispatch in Phase 25/26 — hp41-core just stores the assignment String.
///
/// LiftEffect: Neutral.
pub fn op_asn(state: &mut CalcState, name: String, key_code: u8) -> Result<(), HpError> {
    if name.is_empty() {
        // OQ-3 Option A: empty name removes the assignment (silent no-op if
        // key_code not present in the map).
        state.assignments.remove(&key_code);
    } else {
        // Insert or overwrite.
        state.assignments.insert(key_code, name);
    }
    crate::stack::apply_lift_effect(state, crate::stack::LiftEffect::Neutral);
    Ok(())
}

// ── Public interpreter entry point ───────────────────────────────────────────

/// Execute a recorded program starting at the given label.
///
/// Clones state.program to avoid Rust borrow conflict: cannot hold &program[pc]
/// and &mut state simultaneously (standard Rust ownership constraint).
/// HP-41 programs are at most 999 steps; the clone is negligible. (RESEARCH Pitfall 1)
///
/// D-06: sets is_running = true, resets to false even on error path.
pub fn run_program(state: &mut CalcState, entry_label: &str) -> Result<(), HpError> {
    // Clone program — borrow conflict guard (D-06, RESEARCH Pitfall 1)
    let program = state.program.clone();

    // Linear scan for entry label (D-02). On miss, try the XEQ-by-name
    // fallback for the four Card Reader ops. User labels always take
    // precedence — fallback only fires on a true miss.
    let start = match program
        .iter()
        .position(|op| matches!(op, Op::Lbl(l) if l == entry_label))
    {
        Some(idx) => idx,
        None => {
            if let Some(op) = builtin_card_op(entry_label) {
                // Dispatch the built-in once and return — no program to run.
                // is_running stays false; we never enter run_loop.
                return crate::ops::dispatch(state, op);
            }
            return Err(HpError::InvalidOp);
        }
    };

    state.pc = start + 1; // execute step AFTER the Lbl marker (Pitfall 4)
    state.call_stack.clear();
    state.is_running = true;

    let result = run_loop(state, &program);

    state.is_running = false; // always reset, even on error (is_running safety reset pattern)
    result
}

/// Resume a halted program from `state.pc`.
///
/// Mirror of [`run_program`] but skips the entry-label search — `state.pc` is
/// the resume point. Used after `Op::Stop` (D-22.1) breaks `run_loop`, when the
/// user hits R/S to continue. Does NOT clear `state.call_stack`: pending XEQ
/// frames must survive a STOP/resume cycle so `RTN` behaves correctly
/// (D-22.2; planner PATTERNS.md §"resume_program()").
///
/// CRITICAL — Pitfall 2: do NOT use `?` to propagate the `run_loop` error.
/// Capture into `let result`, reset `is_running = false`, then return `result`.
/// The naive `run_loop(...)?` short-circuits before the cleanup and leaves
/// `state.is_running == true`. (RESEARCH §2 Pitfall 2.)
///
/// Phase 22 (FN-PROG-01).
pub fn resume_program(state: &mut CalcState) -> Result<(), HpError> {
    if state.pc >= state.program.len() {
        return Err(HpError::InvalidOp); // nothing to resume
    }
    let program = state.program.clone();
    state.is_running = true;
    let result = run_loop(state, &program);
    state.is_running = false; // ALWAYS reset, even on Err (Pitfall 2)
    result
}

// ── Private interpreter loop ──────────────────────────────────────────────────

/// Maximum steps per run_program execution — guards against infinite loops.
/// HP-41 programs are at most 999 steps; 1 000 000 allows generous loop counts.
const MAX_STEPS: u64 = 1_000_000;

fn run_loop(state: &mut CalcState, program: &[Op]) -> Result<(), HpError> {
    let mut steps: u64 = 0;
    loop {
        if steps >= MAX_STEPS {
            return Err(HpError::Overflow); // infinite-loop guard (CR-01)
        }
        steps += 1;
        if state.pc >= program.len() {
            // Ran off end of program = implicit top-level RTN
            break;
        }
        let op = program[state.pc].clone();
        state.pc += 1;

        match op {
            Op::Rtn => {
                match state.call_stack.pop() {
                    Some(return_pc) => state.pc = return_pc,
                    None => break, // top-level RTN = normal termination
                }
            }
            Op::Lbl(_) => {
                // No-op during execution — LBL is only a search target
            }
            Op::Gto(label) => {
                let target = find_in_program(program, &label)?;
                state.pc = target + 1;
            }
            Op::GtoInd(reg) => {
                // Phase 24 (D-24.5): pointer-validation logic now lives in the shared
                // `resolve_indirect_decimal` helper — single source of truth across all
                // ~14 indirect-resolving callers (this + Op::XeqInd + 12 Op::*Ind variants).
                let i = crate::ops::indirect::resolve_indirect_decimal(state, reg)?;
                let label_str = i.to_string();
                let target = find_in_program(program, &label_str)?;
                state.pc = target + 1; // mirrors Op::Gto: pc → step AFTER LBL marker
            }
            Op::XeqInd(reg) => {
                // Pre-mutation atomicity guard (D-22.15) — UNCHANGED, must run before
                // any pointer read.
                if state.call_stack.len() >= 4 {
                    return Err(HpError::CallDepth);
                }
                // Phase 24 (D-24.5): shared pointer-validation helper. The label-lookup
                // path (find_in_program + call_stack.push + pc advance) is NOT modified —
                // GTO/XEQ-IND resolve to LABELS, not register addresses, so the inner
                // Decimal-returning helper (not the u8 wrapper) is the right consumer.
                let i = crate::ops::indirect::resolve_indirect_decimal(state, reg)?;
                let label_str = i.to_string();
                let target = find_in_program(program, &label_str)?;
                state.call_stack.push(state.pc);
                state.pc = target + 1;
            }
            Op::Xeq(label) => {
                if state.call_stack.len() >= 4 {
                    return Err(HpError::CallDepth); // D-13/D-14: error before mutation
                }
                // User-label lookup first; on miss fall back to the four
                // Card Reader built-ins. Built-in dispatch does NOT push
                // the call stack — it's a single op, not a subroutine
                // call, so pc just advances.
                match find_in_program(program, &label) {
                    Ok(target) => {
                        state.call_stack.push(state.pc);
                        state.pc = target + 1;
                    }
                    Err(_) => {
                        if let Some(card_op) = builtin_card_op(&label) {
                            // No pc adjustment — the main run_loop advance at the top of
                            // this iteration already moved pc past the XEQ. A card op is a
                            // single instruction (not a control-flow change), so pc resumes
                            // at the step that follows the XEQ.
                            crate::ops::dispatch(state, card_op)?;
                        } else if let Some(xrom_op) =
                            // Phase 28 (v3.0) XROM resolver — fires LAST (C-28.4 / Pitfall 1).
                            // Checked after builtin_card_op so built-in names always win.
                            crate::ops::math1::xrom::xrom_resolve(
                                &label,
                                state.xrom_modules,
                            )
                        {
                            // No pc adjustment — same rationale as card_op above: XROM ops
                            // are single instructions dispatched inline, not subroutine calls.
                            crate::ops::dispatch(state, xrom_op)?;
                        } else {
                            return Err(HpError::InvalidOp);
                        }
                    }
                }
            }
            Op::Test(kind) => {
                if !evaluate_test(state, &kind) {
                    state.pc += 1; // skip next step (D-09: skip-if-false)
                }
            }
            Op::Isg(reg) => {
                if op_isg(state, reg)? {
                    state.pc += 1; // loop exit: skip next
                }
            }
            Op::Dse(reg) => {
                if op_dse(state, reg)? {
                    state.pc += 1; // loop exit: skip next
                }
            }
            // -- Phase 24: Indirect ISG/DSE arms (FN-IND-01, Pitfall 1) ----
            // Skip-next-step semantic preserved exactly as in Op::Isg/Op::Dse.
            // The shim returns Result<bool, _>; the bool signals counter exit.
            Op::IsgInd(reg) => {
                if crate::ops::indirect::op_isg_ind(state, reg)? {
                    state.pc += 1; // loop exit: skip next (mirrors Op::Isg)
                }
            }
            Op::DseInd(reg) => {
                if crate::ops::indirect::op_dse_ind(state, reg)? {
                    state.pc += 1; // loop exit: skip next (mirrors Op::Dse)
                }
            }
            // ── Phase 21: Flag tests (skip next step pattern, mirrors Op::Test) ──
            // FS?/FC? skip the next step when the flag is in the "false" state.
            // FS?C/FC?C ALWAYS clear the flag as a side effect (RESEARCH A4), THEN
            // decide the skip based on the PRE-clear state.
            Op::FlagTest { kind, flag } => {
                use crate::ops::flags::{flag_clear, flag_get};
                use crate::ops::FlagTestKind;
                let is_set = flag_get(state.flags, flag);
                let should_skip = match kind {
                    FlagTestKind::IsSet => !is_set,
                    FlagTestKind::IsClear => is_set,
                    FlagTestKind::IsSetThenClear => {
                        state.flags = flag_clear(state.flags, flag);
                        !is_set
                    }
                    FlagTestKind::IsClearThenClear => {
                        state.flags = flag_clear(state.flags, flag);
                        is_set
                    }
                };
                if should_skip {
                    state.pc += 1;
                }
            }
            // -- Phase 24: Indirect FlagTest arm (FN-IND-01, D-24.6) -------
            // Resolves the flag number via resolve_indirect, then reuses the
            // exact kind-match block from Op::FlagTest verbatim (always-clear
            // semantics for FS?C/FC?C, skip-if-false for FS?/FC?). Pitfall 1
            // mitigation: skip semantics live HERE in run_loop, NOT dispatch.
            Op::FlagTestInd { kind, ind_reg } => {
                use crate::ops::flags::{flag_clear, flag_get};
                use crate::ops::FlagTestKind;
                let flag = crate::ops::indirect::resolve_indirect(state, ind_reg)?;
                let is_set = flag_get(state.flags, flag);
                let should_skip = match kind {
                    FlagTestKind::IsSet => !is_set,
                    FlagTestKind::IsClear => is_set,
                    FlagTestKind::IsSetThenClear => {
                        state.flags = flag_clear(state.flags, flag);
                        !is_set
                    }
                    FlagTestKind::IsClearThenClear => {
                        state.flags = flag_clear(state.flags, flag);
                        is_set
                    }
                };
                if should_skip {
                    state.pc += 1;
                }
            }
            // ── Phase 22 D-22.1 / Pitfall 1: STOP breaks run_loop only — NO display_override write
            // (unlike Op::Prompt below). The previous step's display persists.
            // state.pc is already advanced past the STOP step by the top-of-iteration
            // `state.pc += 1` (line 189). FN-PROG-01.
            Op::Stop => break,
            // ── Phase 21: PROMPT — write ALPHA to display_override + break run_loop.
            // Full STOP/resume semantics deferred to Phase 22 (RESEARCH A5).
            Op::Prompt => {
                state.display_override = Some(state.alpha_reg.chars().take(24).collect::<String>());
                break;
            }
            // ── Phase 28: DIFEQ (Plan 28-09) ─────────────────────────────────
            // Op::Difeq must run inside run_loop (not dispatch) to allow re-entrant
            // run_loop calls for each RK4 sub-step (C-28.5 / Pitfall 4).
            // The real implementation is in op_difeq_run_loop; the dispatch arm
            // (execute_op and dispatch()) returns InvalidOp per the Op::Integ / Op::Solve precedents.
            Op::Difeq => {
                crate::ops::math1::difeq::op_difeq_run_loop(state, program)?;
            }
            // ── Phase 28: INTG (Plan 28-07) ──────────────────────────────────
            // Op::Integ must run inside run_loop (not dispatch) to allow re-entrant
            // run_loop calls for each sample point (C-28.5 / Pitfall 4).
            // The real implementation is in op_integ_run_loop; the dispatch arm
            // (execute_op and dispatch()) returns InvalidOp per the XeqInd precedent.
            Op::Integ => {
                crate::ops::math1::integ::op_integ_run_loop(state, program)?;
            }
            // ── Phase 28: SOLVE / SOL (Plan 28-08) ───────────────────────────
            // Op::Solve and Op::Sol must run inside run_loop (not dispatch) to allow
            // re-entrant run_loop calls for each secant iteration (C-28.5 / Pitfall 4).
            // The real implementations are in op_solve_run_loop / op_sol_run_loop;
            // the dispatch arm (execute_op and dispatch()) returns InvalidOp per the
            // Op::Integ precedent from Plan 28-07.
            Op::Solve => {
                crate::ops::math1::solve::op_solve_run_loop(state, program)?;
            }
            Op::Sol => {
                crate::ops::math1::solve::op_sol_run_loop(state, program)?;
            }
            // ── Phase 43: ADV FSOLVE / FINTG / FDIFEQ run_loop arms ──────────
            // These solver ops must run inside run_loop (not dispatch) to allow
            // re-entrant user-program callbacks (D-43.7 cross-solver nesting).
            // The dispatch arms return InvalidOp; the run_loop arms call the real
            // implementations with the program slice (same pattern as Op::Integ /
            // Op::Solve / Op::Difeq from Plans 28-07/08/09).
            Op::AdvFsolveRunLoop => {
                crate::ops::advantage::solvers::op_adv_fsolve_run_loop(state, program)?;
            }
            Op::AdvFintgRunLoop => {
                crate::ops::advantage::solvers::op_adv_fintg_run_loop(state, program)?;
            }
            Op::AdvFdifeqRunLoop => {
                crate::ops::advantage::solvers::op_adv_fdifeq_run_loop(state, program)?;
            }
            other => {
                // All other ops execute without flush_entry_buf (no digit entry mid-program)
                // and without prgm_mode check (RESEARCH Pitfall 2)
                execute_op(state, other)?;
            }
        }
    }
    Ok(())
}

// ── Private execute_op (no flush, no prgm_mode) ──────────────────────────────

/// Execute a non-programming op inside the interpreter loop.
///
/// MUST NOT call flush_entry_buf (no digit entry mid-program, RESEARCH Pitfall 2).
/// MUST NOT check prgm_mode (always false when is_running = true).
///
/// `pub(crate)` visibility so `ops::math1::integ::run_user_function` can call it
/// for user-callback re-entry (Plan 28-07 / C-28.5). Do NOT call from outside hp41-core.
pub(crate) fn execute_op_pub(state: &mut CalcState, op: Op) -> Result<(), HpError> {
    execute_op(state, op)
}

fn execute_op(state: &mut CalcState, op: Op) -> Result<(), HpError> {
    use crate::ops::alpha::{op_alpha_append, op_alpha_backspace, op_alpha_clear, op_alpha_toggle};
    use crate::ops::arithmetic::{op_add, op_div, op_mul, op_sub};
    use crate::ops::math::{
        op_abs, op_acos, op_asin, op_atan, op_cos, op_exp, op_fact, op_frc, op_int, op_ln, op_log,
        op_mod, op_pct_change, op_pi, op_polar_to_rect, op_recip, op_rect_to_polar, op_rnd,
        op_set_deg, op_set_grad, op_set_rad, op_sign, op_sin, op_sq, op_sqrt, op_tan, op_tenpow,
        op_ypow,
    };
    use crate::ops::registers::{op_clreg, op_rcl, op_sto, op_sto_arith, op_sto_arith_stack};
    use crate::ops::stack_ops::{op_chs, op_clx, op_enter, op_lastx, op_r_up, op_rdn, op_xy_swap};
    use crate::state::DisplayMode;

    match op {
        // Phase 1 arithmetic
        Op::Add => op_add(state),
        Op::Sub => op_sub(state),
        Op::Mul => op_mul(state),
        Op::Div => op_div(state),
        // Phase 1 stack ops
        Op::Enter => op_enter(state),
        Op::Clx => op_clx(state),
        Op::Chs => op_chs(state),
        Op::Rdn => op_rdn(state),
        Op::Rup => op_r_up(state),
        Op::XySwap => op_xy_swap(state),
        Op::Lastx => op_lastx(state),
        Op::Pi => op_pi(state),
        Op::PushNum(v) => {
            enter_number(state, v);
            // PushNum inside a program enables lift so subsequent PushNums lift the stack
            // (mirrors flush_entry_buf execute-mode behavior: enter_number + LiftEffect::Enable)
            apply_lift_effect(state, LiftEffect::Enable);
            Ok(())
        }
        // Phase 2 math/trig/angle
        Op::Int => op_int(state),
        // ── Phase 20 additions ──────────────────────────────────────────────
        Op::Rnd => op_rnd(state),
        Op::Frc => op_frc(state),
        Op::Abs => op_abs(state),
        Op::Sign => op_sign(state),
        Op::Fact => op_fact(state),
        Op::Recip => op_recip(state),
        Op::Sqrt => op_sqrt(state),
        Op::Sq => op_sq(state),
        Op::YPow => op_ypow(state),
        Op::Mod => op_mod(state),
        Op::PctChange => op_pct_change(state),
        Op::Ln => op_ln(state),
        Op::Log => op_log(state),
        Op::Exp => op_exp(state),
        Op::TenPow => op_tenpow(state),
        Op::Sin => op_sin(state),
        Op::Cos => op_cos(state),
        Op::Tan => op_tan(state),
        Op::Asin => op_asin(state),
        Op::Acos => op_acos(state),
        Op::Atan => op_atan(state),
        Op::PolarToRect => op_polar_to_rect(state),
        Op::RectToPolar => op_rect_to_polar(state),
        Op::SetDeg => op_set_deg(state),
        Op::SetRad => op_set_rad(state),
        Op::SetGrad => op_set_grad(state),
        Op::FmtFix(n) => {
            if n > 9 {
                return Err(HpError::InvalidOp);
            }
            state.display_mode = DisplayMode::Fix(n);
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        Op::FmtSci(n) => {
            if n > 9 {
                return Err(HpError::InvalidOp);
            }
            state.display_mode = DisplayMode::Sci(n);
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        Op::FmtEng(n) => {
            if n > 9 {
                return Err(HpError::InvalidOp);
            }
            state.display_mode = DisplayMode::Eng(n);
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        Op::StoReg(r) => op_sto(state, r),
        Op::RclReg(r) => op_rcl(state, r),
        Op::StoArith { reg, kind } => op_sto_arith(state, reg, kind),
        Op::StoArithStack { kind, stack_reg } => op_sto_arith_stack(state, stack_reg, kind),
        Op::Clreg => op_clreg(state),
        Op::AlphaToggle => op_alpha_toggle(state),
        Op::AlphaAppend(ch) => op_alpha_append(state, ch),
        Op::AlphaClear => op_alpha_clear(state),
        Op::AlphaBackspace => op_alpha_backspace(state),
        Op::UserMode => {
            state.user_mode = !state.user_mode;
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        // ── Phase 6: Science & Engineering ───────────────────────────────────────
        Op::SigmaPlus => super::stats::op_sigma_plus(state),
        Op::SigmaMinus => super::stats::op_sigma_minus(state),
        Op::Mean => super::stats::op_mean(state),
        Op::Sdev => super::stats::op_sdev(state),
        Op::LR => super::stats::op_lr(state),
        Op::Yhat => super::stats::op_yhat(state),
        Op::Corr => super::stats::op_corr(state),
        Op::ClSigmaStat => super::stats::op_cl_sigma_stat(state),
        Op::HmsToH => super::hms::op_hms_to_h(state),
        Op::HToHms => super::hms::op_h_to_hms(state),
        Op::HmsAdd => super::hms::op_hms_add(state),
        Op::HmsSub => super::hms::op_hms_sub(state),
        // ── Phase 11: Print operations ───────────────────────────────────────────────
        Op::PRX => super::print::op_prx(state),
        Op::PRA => super::print::op_pra(state),
        Op::PRSTK => super::print::op_prstk(state),
        // ── Phase 12: Synthetic Programming ─────────────────────────────────
        Op::GetKey => super::registers::op_getkey(state),
        Op::Null => {
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        Op::StoM => super::registers::op_sto_m(state),
        Op::StoN => super::registers::op_sto_n(state),
        Op::StoO => super::registers::op_sto_o(state),
        Op::RclM => super::registers::op_rcl_m(state),
        Op::RclN => super::registers::op_rcl_n(state),
        Op::RclO => super::registers::op_rcl_o(state),
        Op::SyntheticByte(b) => {
            if let Some(op) = super::synthetic_byte_to_op(b) {
                // Recursive — safe per the same invariant as in dispatch().
                execute_op(state, op)
            } else {
                Err(HpError::InvalidOp)
            }
        }
        // Inside a running program, card ops stage a request just like in
        // interactive dispatch. Back-to-back card ops without a frontend
        // drain in between surface as `HpError::CardData` rather than
        // silently dropping the prior request.
        Op::Wdta => super::cardreader_ops::op_wdta(state),
        Op::Rdta => super::cardreader_ops::op_rdta(state),
        Op::Wprgm => super::cardreader_ops::op_wprgm(state),
        Op::Rdprgm => super::cardreader_ops::op_rdprgm(state),
        // ── Phase 21: Flag operations ──────────────────────────────────────────
        Op::SfFlag(n) => super::flags::op_sf(state, n),
        Op::CfFlag(n) => super::flags::op_cf(state, n),
        // ── Phase 21: Display Control ─────────────────────────────────────────
        // Op::Prompt is intentionally omitted — it is handled by run_loop directly
        // (with the `break` side effect) and listed in the catch-all below so any
        // accidental reach into execute_op returns InvalidOp.
        Op::View(r) => super::display_ops::op_view(state, r),
        Op::AView => super::display_ops::op_aview(state),
        Op::Aon => super::display_ops::op_aon(state),
        Op::Aoff => super::display_ops::op_aoff(state),
        Op::Cld => super::display_ops::op_cld(state),
        // ── Phase 21: Sound ───────────────────────────────────────────────────
        Op::Beep => super::sound::op_beep(state),
        Op::Tone(n) => super::sound::op_tone(state, n),
        // ── Phase 22: PSE — pause display (D-22.4, FN-PROG-02, Pitfall 3) ────
        // Writes both channels: display_override (visible value) + event_buffer
        // ("PAUSE 1000" marker for frontend timing). run_loop does NOT break;
        // execution continues to the next step. display_override survives
        // subsequent run_loop iterations because run_loop calls execute_op
        // directly (NOT dispatch), so the dispatch-top clear at mod.rs:410
        // does not fire between iterations. The NEXT interactive dispatch
        // clears it — matches HP-41 "value visible until next key" semantic.
        // Pitfall 10: do NOT add flush_entry_buf here — dispatch already
        // called it; execute_op inside run_loop never sees stale entry_buf.
        Op::Pse => {
            let formatted = crate::format::format_hpnum(&state.stack.x, &state.display_mode);
            state.display_override = Some(formatted);
            state.event_buffer.push("PAUSE 1000".to_string());
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        // ── Phase 22: Memory management (D-22.11..13, FN-MEM-01..02) ──────────
        // SIZE executes fine inside run_loop — it is a regular dispatch op,
        // not a control-flow primitive. Does NOT join the programming-ops
        // catch-all below.
        Op::Size(n) => op_size(state, n),
        // D-22.13: Op::Cla delegates to op_alpha_clear (hardware-faithful
        // "CLA" listing). Op::AlphaClear (legacy v1.0) stays separate.
        Op::Cla => super::alpha::op_alpha_clear(state),
        // D-22.14: CLST zeros X/Y/Z/T while PRESERVING lastx + lift_enabled.
        Op::Clst => op_clst(state),
        // D-22.12: PACK is a documented no-op on the flat-Vec program
        // model (no gaps to compact). Neutral lift.
        Op::Pack => {
            apply_lift_effect(state, LiftEffect::Neutral);
            Ok(())
        }
        // D-22.16 (AMENDED OQ-1 Option B): CATALOG n — hardware-faithful.
        // Executes fine inside run_loop AND interactively; output drained
        // from state.print_buffer by the frontend.
        Op::Catalog(n) => op_catalog(state, n),
        // D-22.18 (AMENDED OQ-3 Option A): ASN — empty `name` removes;
        // non-empty inserts. Executes fine inside run_loop AND interactively.
        Op::Asn { name, key_code } => op_asn(state, name, key_code),
        // ── Phase 23: ALPHA-register operations (D-23.12) ─────────────────
        // ARCL/ASTO are regular ops — they execute fine inside run_loop AND
        // interactively; no control-flow concerns. Neutral lift both.
        Op::Arcl(reg) => super::alpha::op_arcl(state, reg),
        Op::Asto(reg) => super::alpha::op_asto(state, reg),
        // Phase 23 plan 02 (FN-ALPHA-03..06): ATOX Enable, XTOA Neutral,
        // AROT Neutral, POSA Disable. None are control-flow primitives —
        // they execute fine in both run_loop and interactive contexts.
        Op::Atox => super::alpha::op_atox(state),
        Op::Xtoa => super::alpha::op_xtoa(state),
        Op::Arot => super::alpha::op_arot(state),
        Op::Posa => super::alpha::op_posa(state),
        // -- Phase 24: Indirect Addressing execute_op arms (FN-IND-01) -----
        // Mirrors the dispatch() arms; each delegates to the corresponding
        // op_*_ind shim. IsgInd/DseInd discard the bool (defense-in-depth;
        // primary skip-semantic path is run_loop). FlagTestInd is NOT here
        // -- it joins the catch-all `|`-pattern below (mirrors Op::FlagTest).
        Op::StoInd(reg) => crate::ops::indirect::op_sto_ind(state, reg),
        Op::RclInd(reg) => crate::ops::indirect::op_rcl_ind(state, reg),
        Op::StoArithInd(reg, kind) => crate::ops::indirect::op_sto_arith_ind(state, reg, kind),
        Op::SfFlagInd(reg) => crate::ops::indirect::op_sf_flag_ind(state, reg),
        Op::CfFlagInd(reg) => crate::ops::indirect::op_cf_flag_ind(state, reg),
        Op::ArclInd(reg) => crate::ops::indirect::op_arcl_ind(state, reg),
        Op::AstoInd(reg) => crate::ops::indirect::op_asto_ind(state, reg),
        Op::ViewInd(reg) => crate::ops::indirect::op_view_ind(state, reg),
        Op::IsgInd(reg) => crate::ops::indirect::op_isg_ind(state, reg).map(|_| ()),
        Op::DseInd(reg) => crate::ops::indirect::op_dse_ind(state, reg).map(|_| ()),
        // ── Phase 28: Hyperbolics (Plan 28-02) ────────────────────────────────────
        Op::Sinh => crate::ops::math1::hyperbolics::op_sinh(state),
        Op::Cosh => crate::ops::math1::hyperbolics::op_cosh(state),
        Op::Tanh => crate::ops::math1::hyperbolics::op_tanh(state),
        Op::Asinh => crate::ops::math1::hyperbolics::op_asinh(state),
        Op::Acosh => crate::ops::math1::hyperbolics::op_acosh(state),
        Op::Atanh => crate::ops::math1::hyperbolics::op_atanh(state),
        // ── Phase 28: Complex Stack Arithmetic (Plan 28-03) ──────────────────────
        Op::CPlus => crate::ops::math1::complex::op_c_plus(state),
        Op::CMinus => crate::ops::math1::complex::op_c_minus(state),
        Op::CTimes => crate::ops::math1::complex::op_c_times(state),
        Op::CDiv => crate::ops::math1::complex::op_c_div(state),
        Op::Real => crate::ops::math1::complex::op_real(state),
        // ── Phase 28: Complex Functions (Plan 28-04) ─────────────────────────────
        Op::Magz => crate::ops::math1::complex::op_magz(state),
        Op::Cinv => crate::ops::math1::complex::op_cinv(state),
        Op::ZpowN => crate::ops::math1::complex::op_z_pow_n(state),
        Op::Zpow1N => crate::ops::math1::complex::op_z_pow_1_n(state),
        Op::ExpZ => crate::ops::math1::complex::op_exp_z(state),
        Op::LnZ => crate::ops::math1::complex::op_ln_z(state),
        Op::SinZ => crate::ops::math1::complex::op_sin_z(state),
        Op::CosZ => crate::ops::math1::complex::op_cos_z(state),
        Op::TanZ => crate::ops::math1::complex::op_tan_z(state),
        Op::ApowZ => crate::ops::math1::complex::op_a_pow_z(state),
        Op::LogZ => crate::ops::math1::complex::op_log_z(state),
        Op::ZpowW => crate::ops::math1::complex::op_z_pow_w(state),
        // ── Phase 28: POLY / ROOTS (Plan 28-05) ─────────────────────────────
        Op::PolyWorkflow => crate::ops::math1::poly::op_poly_workflow(state),
        Op::Roots => crate::ops::math1::poly::op_roots(state),
        // ── Phase 28: MATRIX (Plan 28-06) ────────────────────────────────────
        Op::MatrixWorkflow => crate::ops::math1::matrix::op_matrix_workflow(state),
        Op::MatSize => crate::ops::math1::matrix::op_mat_size(state),
        Op::MatVmat => crate::ops::math1::matrix::op_mat_vmat(state),
        Op::MatEdit => crate::ops::math1::matrix::op_mat_edit(state),
        Op::MatDet => crate::ops::math1::matrix::op_mat_det(state),
        Op::MatInv => crate::ops::math1::matrix::op_mat_inv(state),
        Op::MatSimeq => crate::ops::math1::matrix::op_mat_simeq(state),
        Op::MatVcol => crate::ops::math1::matrix::op_mat_vcol(state),
        // Programming ops handled by run_loop directly — must not reach here
        Op::Lbl(_)
        | Op::Gto(_)
        | Op::Xeq(_)
        | Op::Rtn
        | Op::PrgmMode
        | Op::Test(_)
        | Op::Isg(_)
        | Op::Dse(_)
        | Op::FlagTest { .. }
        | Op::Prompt
        | Op::Stop                              // Phase 22: STOP handled by run_loop break
        | Op::GtoInd(_)                         // Phase 22: GTO IND has run_loop arm
        | Op::XeqInd(_)                         // Phase 22: XEQ IND has run_loop arm
        | Op::Clp(_)                            // Phase 22: CLP is a PRGM-mode editing primitive
        | Op::Del(_)                            // Phase 22: DEL is a PRGM-mode editing primitive
        | Op::FlagTestInd { .. }              // Phase 24: FlagTestInd has run_loop arm (no execute_op delegate)
        | Op::Ins                               // Phase 22: INS is a PRGM-mode editing primitive
        | Op::Difeq                             // Phase 28: DIFEQ has run_loop arm; dispatch returns InvalidOp (Plan 28-09)
        | Op::Integ                             // Phase 28: INTG has run_loop arm; dispatch returns InvalidOp
        | Op::Solve                             // Phase 28: SOLVE has run_loop arm; dispatch returns InvalidOp
        | Op::Sol => Err(HpError::InvalidOp),   // Phase 28: SOL has run_loop arm; dispatch returns InvalidOp
        // ── Phase 28: FOUR / Triangle Solvers / TRANS (Plan 28-10) ──────────
        // Pure-data ops: execute fully from dispatch context (no run_loop re-entry needed).
        // Mirrors hyperbolics Plan 28-02 pattern (NOT the user-callback Plan 28-07/08 pattern).
        Op::Four => crate::ops::dispatch(state, Op::Four),
        Op::TriSss => crate::ops::dispatch(state, Op::TriSss),
        Op::TriAsa => crate::ops::dispatch(state, Op::TriAsa),
        Op::TriSaa => crate::ops::dispatch(state, Op::TriSaa),
        Op::TriSas => crate::ops::dispatch(state, Op::TriSas),
        Op::TriSsa => crate::ops::dispatch(state, Op::TriSsa),
        Op::Trans2d => crate::ops::dispatch(state, Op::Trans2d),
        Op::Trans3d => crate::ops::dispatch(state, Op::Trans3d),
        // ── Phase 33 Plan 33-04: Closed-form non-parametric Ops ─────────────
        // Pure-data ops (closed-form, no iteration / no user callback) routed
        // through dispatch; mirrors hyperbolics + Plan 28-10 Triangle/TRANS
        // pattern (NOT the user-callback Plan 28-07/08 pattern).
        Op::SigmaSpear => crate::ops::dispatch(state, Op::SigmaSpear),
        Op::SigmaXsqev => crate::ops::dispatch(state, Op::SigmaXsqev),
        Op::SigmaEfxsq => crate::ops::dispatch(state, Op::SigmaEfxsq),
        // ── Phase 33 Plan 33-05: Univariate / weighted summary Ops ──────────
        // Pure-data ops (closed-form on R01–R06); same dispatch routing.
        Op::SigmaBstat => crate::ops::dispatch(state, Op::SigmaBstat),
        Op::SigmaBstg => crate::ops::dispatch(state, Op::SigmaBstg),
        // ── Phase 33 Plan 33-05: Curve-fit accumulators (delegate pattern) ──
        // Per-point accumulators (delegate to op_sigma_plus after transform);
        // pure-data routing through dispatch.
        Op::SigmaLin => crate::ops::dispatch(state, Op::SigmaLin),
        Op::SigmaExp => crate::ops::dispatch(state, Op::SigmaExp),
        Op::SigmaLogi => crate::ops::dispatch(state, Op::SigmaLogi),
        Op::SigmaPow => crate::ops::dispatch(state, Op::SigmaPow),
        // ── Phase 33 Plan 33-06: ΣMMTUG / ΣMMTGD third + fourth moments ─────
        // Per-point accumulators; pure-data routing through dispatch.
        Op::SigmaMmtug => crate::ops::dispatch(state, Op::SigmaMmtug),
        Op::SigmaMmtgd => crate::ops::dispatch(state, Op::SigmaMmtgd),
        // ── Phase 33 Plan 33-06: ANOVA family (one-way / two-way / ANCOVA) ──
        // Closed-form (one-way) and iterative (two-way / ANCOVA) Ops; pure
        // dispatch routing — same pattern as Plan 33-04 / 33-05 Σ-family.
        Op::SigmaAovone => crate::ops::dispatch(state, Op::SigmaAovone),
        Op::SigmaAovtwo => crate::ops::dispatch(state, Op::SigmaAovtwo),
        Op::SigmaAnocov => crate::ops::dispatch(state, Op::SigmaAnocov),
        // ── Phase 33 Plan 33-06: Contingency-table χ² Ops ───────────────────
        Op::SigmaCtkkk => crate::ops::dispatch(state, Op::SigmaCtkkk),
        Op::SigmaCtkk => crate::ops::dispatch(state, Op::SigmaCtkk),
        // ── Phase 33 Plan 33-07: ΣPTST one-sample t-test ────────────────────
        // Pure-data op (closed-form + iterative-primitive bridge); pure
        // dispatch routing — same pattern as Plan 33-04/05/06 Σ-family.
        Op::SigmaPtst => crate::ops::dispatch(state, Op::SigmaPtst),
        // ── Phase 33 Plan 33-07: ΣTSTAT pooled-variance two-sample t-test ───
        Op::SigmaTstat => crate::ops::dispatch(state, Op::SigmaTstat),
        // ── Phase 33 Plan 33-08: Multiple + polynomial regression ───────────
        // Pure-data ops (closed-form Gauss elimination + Horner eval);
        // modal opener (SigmaPolypWorkflow) follows the PolyWorkflow
        // pattern. All four route through dispatch().
        Op::SigmaMlrxy => crate::ops::dispatch(state, Op::SigmaMlrxy),
        Op::SigmaMlrxyz => crate::ops::dispatch(state, Op::SigmaMlrxyz),
        Op::SigmaPolypWorkflow => crate::ops::dispatch(state, Op::SigmaPolypWorkflow),
        Op::SigmaPolyc => crate::ops::dispatch(state, Op::SigmaPolyc),
        // ── Phase 33 Plan 33-03: ΣNORMD modal opener ────────────────────────
        // Modal opener (mirrors PolyWorkflow / MatrixWorkflow / etc. pattern);
        // pure-dispatch routing through dispatch().
        Op::SigmaNormdWorkflow => crate::ops::dispatch(state, Op::SigmaNormdWorkflow),
        // ── Phase 33 Plan 33-03: ΣCHISQD modal opener ───────────────────────
        Op::SigmaChisqdWorkflow => crate::ops::dispatch(state, Op::SigmaChisqdWorkflow),
        // ── Phase 33 Plan 33-08: RAND / SEED — emulator extension (D-33.4) ──
        Op::Rand => crate::ops::dispatch(state, Op::Rand),
        Op::Seed => crate::ops::dispatch(state, Op::Seed),
        // ── Phase 38 (v3.2): Time Module (XROM 26) ──────────────────────────
        // Phase 38 sanctioned CI break — Phase 39 closes item 3 (CLI), Phase 41 closes item 4 (GUI).
        // All Time Module ops route through dispatch() (same pattern as Stat 1 ops).
        Op::TimeTime => crate::ops::dispatch(state, Op::TimeTime),
        Op::TimeDate => crate::ops::dispatch(state, Op::TimeDate),
        Op::TimeSetime => crate::ops::dispatch(state, Op::TimeSetime),
        Op::TimeSetdate => crate::ops::dispatch(state, Op::TimeSetdate),
        Op::TimeClk12 => crate::ops::dispatch(state, Op::TimeClk12),
        Op::TimeClk24 => crate::ops::dispatch(state, Op::TimeClk24),
        Op::TimeClkt => crate::ops::dispatch(state, Op::TimeClkt),
        Op::TimeClktd => crate::ops::dispatch(state, Op::TimeClktd),
        Op::TimeClock => crate::ops::dispatch(state, Op::TimeClock),
        Op::TimeCorrect => crate::ops::dispatch(state, Op::TimeCorrect),
        Op::TimeTplusx => crate::ops::dispatch(state, Op::TimeTplusx),
        Op::TimeDatePlus => crate::ops::dispatch(state, Op::TimeDatePlus),
        Op::TimeDdays => crate::ops::dispatch(state, Op::TimeDdays),
        Op::TimeDow => crate::ops::dispatch(state, Op::TimeDow),
        Op::TimeDmy => crate::ops::dispatch(state, Op::TimeDmy),
        Op::TimeMdy => crate::ops::dispatch(state, Op::TimeMdy),
        Op::TimeAtime => crate::ops::dispatch(state, Op::TimeAtime),
        Op::TimeAtime24 => crate::ops::dispatch(state, Op::TimeAtime24),
        Op::TimeAdate => crate::ops::dispatch(state, Op::TimeAdate),
        Op::TimeRunsw => crate::ops::dispatch(state, Op::TimeRunsw),
        Op::TimeStopsw => crate::ops::dispatch(state, Op::TimeStopsw),
        Op::TimeRclsw => crate::ops::dispatch(state, Op::TimeRclsw),
        Op::TimeSetsw => crate::ops::dispatch(state, Op::TimeSetsw),
        Op::TimeSw => crate::ops::dispatch(state, Op::TimeSw),
        Op::TimeSwpt => crate::ops::dispatch(state, Op::TimeSwpt),
        Op::TimeStpw => crate::ops::dispatch(state, Op::TimeStpw),
        Op::TimeXyzalm => crate::ops::dispatch(state, Op::TimeXyzalm),
        Op::TimeAlmcat => crate::ops::dispatch(state, Op::TimeAlmcat),
        Op::TimeAlmnow => crate::ops::dispatch(state, Op::TimeAlmnow),
        Op::TimeRclalm => crate::ops::dispatch(state, Op::TimeRclalm),
        Op::TimeRclaf => crate::ops::dispatch(state, Op::TimeRclaf),
        Op::TimeSetaf => crate::ops::dispatch(state, Op::TimeSetaf),
        Op::TimeClalma => crate::ops::dispatch(state, Op::TimeClalma),
        Op::TimeClalmx => crate::ops::dispatch(state, Op::TimeClalmx),
        Op::TimeClralms => crate::ops::dispatch(state, Op::TimeClralms),
        // ── Phase 43 (v3.3): Advantage Pac (XROM 22 + XROM 24) ─────────────
        Op::AdvBinin => crate::ops::dispatch(state, Op::AdvBinin),
        Op::AdvBinview => crate::ops::dispatch(state, Op::AdvBinview),
        Op::AdvOctin => crate::ops::dispatch(state, Op::AdvOctin),
        Op::AdvHexin => crate::ops::dispatch(state, Op::AdvHexin),
        Op::AdvHexview => crate::ops::dispatch(state, Op::AdvHexview),
        Op::AdvCvtview => crate::ops::dispatch(state, Op::AdvCvtview),
        Op::AdvNot => crate::ops::dispatch(state, Op::AdvNot),
        Op::AdvAnd => crate::ops::dispatch(state, Op::AdvAnd),
        Op::AdvOr => crate::ops::dispatch(state, Op::AdvOr),
        Op::AdvXor => crate::ops::dispatch(state, Op::AdvXor),
        Op::AdvRotxy => crate::ops::dispatch(state, Op::AdvRotxy),
        Op::AdvBitTest => crate::ops::dispatch(state, Op::AdvBitTest),
        Op::AdvIPlus => crate::ops::dispatch(state, Op::AdvIPlus),
        Op::AdvIMinus => crate::ops::dispatch(state, Op::AdvIMinus),
        Op::AdvJPlus => crate::ops::dispatch(state, Op::AdvJPlus),
        Op::AdvJMinus => crate::ops::dispatch(state, Op::AdvJMinus),
        Op::AdvMr => crate::ops::dispatch(state, Op::AdvMr),
        Op::AdvMs => crate::ops::dispatch(state, Op::AdvMs),
        Op::AdvMrij => crate::ops::dispatch(state, Op::AdvMrij),
        Op::AdvMsij => crate::ops::dispatch(state, Op::AdvMsij),
        Op::AdvMsijr => crate::ops::dispatch(state, Op::AdvMsijr),
        Op::AdvMrcPlus => crate::ops::dispatch(state, Op::AdvMrcPlus),
        Op::AdvMrcMinus => crate::ops::dispatch(state, Op::AdvMrcMinus),
        Op::AdvMrrPlus => crate::ops::dispatch(state, Op::AdvMrrPlus),
        Op::AdvMrrMinus => crate::ops::dispatch(state, Op::AdvMrrMinus),
        Op::AdvMsrPlus => crate::ops::dispatch(state, Op::AdvMsrPlus),
        Op::AdvMscPlus => crate::ops::dispatch(state, Op::AdvMscPlus),
        Op::AdvMswap => crate::ops::dispatch(state, Op::AdvMswap),
        Op::AdvMnameQuery => crate::ops::dispatch(state, Op::AdvMnameQuery),
        Op::AdvDimQuery => crate::ops::dispatch(state, Op::AdvDimQuery),
        Op::AdvMatdim => crate::ops::dispatch(state, Op::AdvMatdim),
        Op::AdvMp => crate::ops::dispatch(state, Op::AdvMp),
        Op::AdvPiv => crate::ops::dispatch(state, Op::AdvPiv),
        Op::AdvRExchangeR => crate::ops::dispatch(state, Op::AdvRExchangeR),
        Op::AdvRGtRQuery => crate::ops::dispatch(state, Op::AdvRGtRQuery),
        Op::AdvSum => crate::ops::dispatch(state, Op::AdvSum),
        Op::AdvSumab => crate::ops::dispatch(state, Op::AdvSumab),
        Op::AdvMax => crate::ops::dispatch(state, Op::AdvMax),
        Op::AdvMaxab => crate::ops::dispatch(state, Op::AdvMaxab),
        Op::AdvMin => crate::ops::dispatch(state, Op::AdvMin),
        Op::AdvRmaxab => crate::ops::dispatch(state, Op::AdvRmaxab),
        Op::AdvRnrm => crate::ops::dispatch(state, Op::AdvRnrm),
        Op::AdvRsum => crate::ops::dispatch(state, Op::AdvRsum),
        Op::AdvFnrm => crate::ops::dispatch(state, Op::AdvFnrm),
        Op::AdvMdet => crate::ops::dispatch(state, Op::AdvMdet),
        Op::AdvMinv => crate::ops::dispatch(state, Op::AdvMinv),
        Op::AdvMsys => crate::ops::dispatch(state, Op::AdvMsys),
        Op::AdvMMulM => crate::ops::dispatch(state, Op::AdvMMulM),
        Op::AdvMatPlus => crate::ops::dispatch(state, Op::AdvMatPlus),
        Op::AdvMatMinus => crate::ops::dispatch(state, Op::AdvMatMinus),
        Op::AdvMatScalarMul => crate::ops::dispatch(state, Op::AdvMatScalarMul),
        Op::AdvMatScalarDiv => crate::ops::dispatch(state, Op::AdvMatScalarDiv),
        Op::AdvTrnps => crate::ops::dispatch(state, Op::AdvTrnps),
        Op::AdvMmove => crate::ops::dispatch(state, Op::AdvMmove),
        Op::AdvCExchangeC => crate::ops::dispatch(state, Op::AdvCExchangeC),
        Op::AdvCmaxab => crate::ops::dispatch(state, Op::AdvCmaxab),
        Op::AdvCnrm => crate::ops::dispatch(state, Op::AdvCnrm),
        Op::AdvCsum => crate::ops::dispatch(state, Op::AdvCsum),
        Op::AdvYcPlusC => crate::ops::dispatch(state, Op::AdvYcPlusC),
        Op::AdvMatrx => crate::ops::dispatch(state, Op::AdvMatrx),
        Op::AdvMtr => crate::ops::dispatch(state, Op::AdvMtr),
        Op::AdvMedit => crate::ops::dispatch(state, Op::AdvMedit),
        Op::AdvCmedit => crate::ops::dispatch(state, Op::AdvCmedit),
        Op::AdvExpZ => crate::ops::dispatch(state, Op::AdvExpZ),
        Op::AdvLnZ => crate::ops::dispatch(state, Op::AdvLnZ),
        Op::AdvLogZ => crate::ops::dispatch(state, Op::AdvLogZ),
        Op::AdvZPowN => crate::ops::dispatch(state, Op::AdvZPowN),
        Op::AdvZPow1n => crate::ops::dispatch(state, Op::AdvZPow1n),
        Op::AdvZPowW => crate::ops::dispatch(state, Op::AdvZPowW),
        Op::AdvZPow1w => crate::ops::dispatch(state, Op::AdvZPow1w),
        Op::AdvMagz => crate::ops::dispatch(state, Op::AdvMagz),
        Op::AdvSinZ => crate::ops::dispatch(state, Op::AdvSinZ),
        Op::AdvCosZ => crate::ops::dispatch(state, Op::AdvCosZ),
        Op::AdvTanZ => crate::ops::dispatch(state, Op::AdvTanZ),
        Op::AdvAPowZ => crate::ops::dispatch(state, Op::AdvAPowZ),
        Op::AdvCPlus => crate::ops::dispatch(state, Op::AdvCPlus),
        Op::AdvCMinus => crate::ops::dispatch(state, Op::AdvCMinus),
        Op::AdvCinv => crate::ops::dispatch(state, Op::AdvCinv),
        Op::AdvCMul => crate::ops::dispatch(state, Op::AdvCMul),
        Op::AdvCDiv => crate::ops::dispatch(state, Op::AdvCDiv),
        Op::AdvAip => crate::ops::dispatch(state, Op::AdvAip),
        Op::AdvPly => crate::ops::dispatch(state, Op::AdvPly),
        Op::AdvRts => crate::ops::dispatch(state, Op::AdvRts),
        Op::AdvFsolve => crate::ops::dispatch(state, Op::AdvFsolve),
        Op::AdvFintg => crate::ops::dispatch(state, Op::AdvFintg),
        Op::AdvFdifeq => crate::ops::dispatch(state, Op::AdvFdifeq),
        Op::AdvFroot => crate::ops::dispatch(state, Op::AdvFroot),
        Op::AdvFsolveRunLoop => crate::ops::dispatch(state, Op::AdvFsolveRunLoop),
        Op::AdvFintgRunLoop => crate::ops::dispatch(state, Op::AdvFintgRunLoop),
        Op::AdvFdifeqRunLoop => crate::ops::dispatch(state, Op::AdvFdifeqRunLoop),
        Op::AdvCfit => crate::ops::dispatch(state, Op::AdvCfit),
        Op::AdvAs => crate::ops::dispatch(state, Op::AdvAs),
        Op::AdvDs => crate::ops::dispatch(state, Op::AdvDs),
        Op::AdvBfit => crate::ops::dispatch(state, Op::AdvBfit),
        Op::AdvFit => crate::ops::dispatch(state, Op::AdvFit),
        Op::AdvYQueryX => crate::ops::dispatch(state, Op::AdvYQueryX),
        Op::AdvSzQuery => crate::ops::dispatch(state, Op::AdvSzQuery),
        Op::AdvVPlus => crate::ops::dispatch(state, Op::AdvVPlus),
        Op::AdvVMinus => crate::ops::dispatch(state, Op::AdvVMinus),
        Op::AdvDot => crate::ops::dispatch(state, Op::AdvDot),
        Op::AdvCross => crate::ops::dispatch(state, Op::AdvCross),
        Op::AdvVc => crate::ops::dispatch(state, Op::AdvVc),
        Op::AdvVs => crate::ops::dispatch(state, Op::AdvVs),
        Op::AdvVr => crate::ops::dispatch(state, Op::AdvVr),
        Op::AdvVe => crate::ops::dispatch(state, Op::AdvVe),
        Op::AdvVxy => crate::ops::dispatch(state, Op::AdvVxy),
        Op::AdvUv => crate::ops::dispatch(state, Op::AdvUv),
        Op::AdvVMag => crate::ops::dispatch(state, Op::AdvVMag),
        Op::AdvVStar => crate::ops::dispatch(state, Op::AdvVStar),
        Op::AdvVd => crate::ops::dispatch(state, Op::AdvVd),
        Op::AdvTr => crate::ops::dispatch(state, Op::AdvTr),
        Op::AdvTvm => crate::ops::dispatch(state, Op::AdvTvm),
        Op::AdvTvmN => crate::ops::dispatch(state, Op::AdvTvmN),
        Op::AdvTvmPv => crate::ops::dispatch(state, Op::AdvTvmPv),
        Op::AdvTvmPmt => crate::ops::dispatch(state, Op::AdvTvmPmt),
        Op::AdvTvmFv => crate::ops::dispatch(state, Op::AdvTvmFv),
        Op::AdvTvmStarI => crate::ops::dispatch(state, Op::AdvTvmStarI),
        // ── Phase 51 (v4.0): X-MEM built-in ops ─────────────────────────────
        Op::EmDir => crate::ops::dispatch(state, Op::EmDir),
        Op::EmRoom => crate::ops::dispatch(state, Op::EmRoom),
        Op::SaveP => crate::ops::dispatch(state, Op::SaveP),
        Op::GetP => crate::ops::dispatch(state, Op::GetP),
        Op::SaveD => crate::ops::dispatch(state, Op::SaveD),
        Op::GetD => crate::ops::dispatch(state, Op::GetD),
        Op::EmReg => crate::ops::dispatch(state, Op::EmReg),
        Op::SaveRx => crate::ops::dispatch(state, Op::SaveRx),
    }
}

// ── Public conditional test evaluator ────────────────────────────────────────

/// Evaluate a conditional test against the current stack.
/// Returns true if condition is TRUE (execute next step).
/// Returns false if condition is FALSE (skip next step, D-09).
/// Stack is NOT modified (LiftEffect: Neutral — read-only access to X and Y).
pub fn evaluate_test(state: &CalcState, kind: &TestKind) -> bool {
    let x = state.stack.x.inner();
    let y = state.stack.y.inner();
    let zero = Decimal::ZERO;
    match kind {
        TestKind::XEqZero => x == zero,
        TestKind::XNeZero => x != zero,
        TestKind::XLtZero => x < zero,
        TestKind::XGtZero => x > zero,
        TestKind::XLeZero => x <= zero,
        TestKind::XGeZero => x >= zero,
        TestKind::XEqY => x == y,
        TestKind::XNeY => x != y,
        TestKind::XLtY => x < y,
        TestKind::XGtY => x > y,
        TestKind::XLeY => x <= y,
        TestKind::XGeY => x >= y,
    }
}

// ── Private helpers ──────────────────────────────────────────────────────────

/// Parse CCCCC.FFFDD counter format by string-splitting at '.'.
/// Returns (current, final, step, frac_padded_5_chars).
///
/// ADR-001: never use floor()/fmod() on f64.
/// D-10: left of decimal = current (i64); right padded to 5 chars;
///       first 3 = final count, last 2 = step (00 → 1).
///
/// CRITICAL: format!("{:0<5}", frac_part) pads RIGHT with zeros (left-align).
/// Do NOT use "{:0>5}" (pads LEFT = wrong field extraction).
pub fn parse_counter(n: &HpNum) -> Result<(i64, i64, i64, String), HpError> {
    let s = n.inner().to_string(); // rust_decimal normalises trailing zeros (e.g. 1.00500 → "1.005")
    let (int_part, frac_part) = if let Some(pos) = s.find('.') {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s.as_str(), "")
    };
    let current: i64 = int_part.parse().map_err(|_| HpError::InvalidOp)?;
    // Pad RIGHT with zeros to exactly 5 chars (trailing-zero normalisation fix, RESEARCH Pitfall 3)
    let frac_padded = format!("{frac_part:0<5}");
    // Truncate if somehow longer than 5 (defensive; should not occur with valid HP-41 counters)
    let frac_padded = if frac_padded.len() > 5 {
        frac_padded[..5].to_string()
    } else {
        frac_padded
    };
    let final_val: i64 = frac_padded[..3].parse().map_err(|_| HpError::InvalidOp)?;
    let step_raw: i64 = frac_padded[3..5].parse().map_err(|_| HpError::InvalidOp)?;
    let step = if step_raw == 0 { 1 } else { step_raw }; // step 00 → 1 (D-10)
    Ok((current, final_val, step, frac_padded))
}

/// Reconstruct counter HpNum from updated current and the preserved frac_padded string.
/// Preserves the FFFDD fields exactly (only CCCCC changes, D-12).
fn build_counter(current: i64, frac_padded: &str) -> Result<HpNum, HpError> {
    let s = format!("{current}.{frac_padded}");
    let d = Decimal::from_str(&s).map_err(|_| HpError::InvalidOp)?;
    Ok(HpNum::rounded(d))
}

/// Linear scan for a label in the cloned program slice (run_loop helper).
fn find_in_program(program: &[Op], label: &str) -> Result<usize, HpError> {
    program
        .iter()
        .position(|op| matches!(op, Op::Lbl(l) if l == label))
        .ok_or(HpError::InvalidOp)
}

/// Linear scan for a label in state.program (interactive dispatch helpers op_gto/op_xeq).
fn find_label_in_state(state: &CalcState, label: &str) -> Result<usize, HpError> {
    state
        .program
        .iter()
        .position(|op| matches!(op, Op::Lbl(l) if l == label))
        .ok_or(HpError::InvalidOp)
}

/// XEQ-by-name fallback: resolves 12 hardware ROM names to their `Op`
/// variants — the four v2.1 Card Reader op names plus the eight non-keyboard
/// HP-41CV conditional-test mnemonics (Phase 25 / Plan 03 / D-25.8).
///
/// Resolved names:
/// - v2.1 card-reader: `WPRGM`, `RDPRGM`, `WDTA`, `RDTA`
/// - Phase 25 conditional tests (both ASCII-pure and Unicode-symbol spellings
///   accepted — RESEARCH §"Conditional tests"):
///   - `X<>Y? | X≠Y? | X#Y?`   → `Op::Test(TestKind::XNeY)`
///   - `X<Y?`                   → `Op::Test(TestKind::XLtY)`
///   - `X>=Y? | X≥Y?`           → `Op::Test(TestKind::XGeY)`
///   - `X#0? | X≠0?`            → `Op::Test(TestKind::XNeZero)`
///   - `X<0?`                   → `Op::Test(TestKind::XLtZero)`
///   - `X>0?`                   → `Op::Test(TestKind::XGtZero)`
///   - `X<=0? | X≤0?`           → `Op::Test(TestKind::XLeZero)`
///   - `X>=0? | X≥0?`           → `Op::Test(TestKind::XGeZero)`
///
/// Returns `None` for anything else — including unknown names, lowercase
/// variants, and any built-in not in this 12-name table. Case-sensitive.
///
/// The 4 keyboard-reachable conditional tests (X=Y, X≤Y, X>Y, X=0) are
/// intentionally NOT registered here (W4 asymmetry per D-25.9) — they are
/// reachable only via the f-shifted arithmetic keys per Plan 01 / D-25.7.
/// A user who types `XEQ "X=Y?"` gets `HpError::InvalidOp` by design.
///
/// Used as the label-miss fallback in `op_xeq`, `run_program`, and the
/// `Op::Xeq` arm of `run_loop`. User `LBL "name"` matches take precedence,
/// matching real HP-41 `XEQ "name"` resolution order.
///
/// Covers ROM built-in ops reachable via XEQ-by-name on a real HP-41.
/// Shared by CLI (`xeq_by_name_local_resolve`) and GUI (`op_xeq`) to
/// ensure D-25.6 CLI↔GUI parity for name resolution.
pub fn builtin_card_op(name: &str) -> Option<Op> {
    match name {
        // v2.1 Card Reader op names (regression preserved unchanged).
        "WPRGM" => Some(Op::Wprgm),
        "RDPRGM" => Some(Op::Rdprgm),
        "WDTA" => Some(Op::Wdta),
        "RDTA" => Some(Op::Rdta),
        // Phase 25 / D-25.8: 8 non-keyboard conditional-test mnemonics.
        "X<>Y?" | "X\u{2260}Y?" | "X#Y?" => Some(Op::Test(TestKind::XNeY)),
        "X<Y?" => Some(Op::Test(TestKind::XLtY)),
        "X>=Y?" | "X\u{2265}Y?" => Some(Op::Test(TestKind::XGeY)),
        "X#0?" | "X\u{2260}0?" => Some(Op::Test(TestKind::XNeZero)),
        "X<0?" => Some(Op::Test(TestKind::XLtZero)),
        "X>0?" => Some(Op::Test(TestKind::XGtZero)),
        "X<=0?" | "X\u{2264}0?" => Some(Op::Test(TestKind::XLeZero)),
        "X>=0?" | "X\u{2265}0?" => Some(Op::Test(TestKind::XGeZero)),
        // ROM built-in ops (canonical HP-41 display names).
        "SIN" => Some(Op::Sin),
        "COS" => Some(Op::Cos),
        "TAN" => Some(Op::Tan),
        "ASIN" => Some(Op::Asin),
        "ACOS" => Some(Op::Acos),
        "ATAN" => Some(Op::Atan),
        "LN" => Some(Op::Ln),
        "LOG" => Some(Op::Log),
        "E^X" => Some(Op::Exp),
        "10^X" => Some(Op::TenPow),
        "SQRT" => Some(Op::Sqrt),
        "X^2" | "XSQ" => Some(Op::Sq),
        "Y^X" => Some(Op::YPow),
        "1/X" | "RECIP" => Some(Op::Recip),
        "PI" => Some(Op::Pi),
        "ABS" => Some(Op::Abs),
        "INT" => Some(Op::Int),
        "FRC" => Some(Op::Frc),
        "SIGN" => Some(Op::Sign),
        "N!" | "FACT" => Some(Op::Fact),
        "MOD" => Some(Op::Mod),
        "RND" => Some(Op::Rnd),
        "P->R" | "P\u{2192}R" => Some(Op::PolarToRect),
        "R->P" | "R\u{2192}P" => Some(Op::RectToPolar),
        "HMS->H" | "HMS\u{2192}H" => Some(Op::HmsToH),
        "H->HMS" | "H\u{2192}HMS" => Some(Op::HToHms),
        "HMS+" => Some(Op::HmsAdd),
        "HMS-" => Some(Op::HmsSub),
        "DEG" => Some(Op::SetDeg),
        "RAD" => Some(Op::SetRad),
        "GRAD" => Some(Op::SetGrad),
        "R^" | "R\u{2191}" | "RUP" => Some(Op::Rup),
        "CLST" => Some(Op::Clst),
        "CLREG" => Some(Op::Clreg),
        "CLA" => Some(Op::Cla),
        "SIGMA+" | "\u{03A3}+" => Some(Op::SigmaPlus),
        "SIGMA-" | "\u{03A3}-" => Some(Op::SigmaMinus),
        "MEAN" => Some(Op::Mean),
        "SDEV" => Some(Op::Sdev),
        "L.R." | "LR" => Some(Op::LR),
        "YHAT" => Some(Op::Yhat),
        "CORR" => Some(Op::Corr),
        "CL SIGMA" | "CL\u{03A3}" | "CLSIGMA" => Some(Op::ClSigmaStat),
        "AVIEW" => Some(Op::AView),
        "PROMPT" => Some(Op::Prompt),
        "AON" => Some(Op::Aon),
        "AOFF" => Some(Op::Aoff),
        "CLD" => Some(Op::Cld),
        "BEEP" => Some(Op::Beep),
        "CLRALPHA" => Some(Op::AlphaClear),
        "ATOX" => Some(Op::Atox),
        "XTOA" => Some(Op::Xtoa),
        "AROT" => Some(Op::Arot),
        "POSA" => Some(Op::Posa),
        "RTN" => Some(Op::Rtn),
        "STOP" => Some(Op::Stop),
        "PSE" => Some(Op::Pse),
        "PACK" => Some(Op::Pack),
        "INS" => Some(Op::Ins),
        "PRX" => Some(Op::PRX),
        "PRA" => Some(Op::PRA),
        "PRSTK" => Some(Op::PRSTK),
        _ => None,
    }
}

/// Legacy test-only alias — kept for backward compatibility with existing test call sites.
#[cfg(test)]
pub fn __test_builtin_card_op(name: &str) -> Option<Op> {
    builtin_card_op(name)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod program_tests {
    use crate::error::HpError;
    use crate::num::HpNum;
    use crate::ops::program::{
        evaluate_test, op_dse, op_gto, op_isg, op_lbl, op_prgm_mode, op_rtn, op_test, op_xeq,
        parse_counter,
    };
    use crate::ops::{Op, TestKind};
    use crate::state::CalcState;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn state_with_program(ops: Vec<Op>) -> CalcState {
        CalcState {
            program: ops,
            ..Default::default()
        }
    }

    // ── Original 12 targeted tests for error paths ────────────────────────────

    #[test]
    fn test_run_program_label_not_found() {
        let mut state = CalcState::default();
        let result = crate::ops::program::run_program(&mut state, "A");
        assert_eq!(result, Err(HpError::InvalidOp));
    }

    #[test]
    fn test_run_program_is_running_reset_on_error() {
        let mut state = CalcState::default();
        // Error result intentionally discarded — this test only checks the is_running side-effect.
        let _ = crate::ops::program::run_program(&mut state, "A");
        assert!(
            !state.is_running,
            "is_running must be false after run_program error"
        );
    }

    #[test]
    fn test_call_depth_limit() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::Xeq("B".to_string()),
            Op::Rtn,
            Op::Lbl("B".to_string()),
            Op::Xeq("C".to_string()),
            Op::Rtn,
            Op::Lbl("C".to_string()),
            Op::Xeq("D".to_string()),
            Op::Rtn,
            Op::Lbl("D".to_string()),
            Op::Xeq("E".to_string()),
            Op::Rtn,
            Op::Lbl("E".to_string()),
            Op::Xeq("F".to_string()),
            Op::Rtn,
            Op::Lbl("F".to_string()),
            Op::Rtn,
        ];
        let mut state = state_with_program(program);
        let result = crate::ops::program::run_program(&mut state, "A");
        assert_eq!(
            result,
            Err(HpError::CallDepth),
            "5th XEQ must exceed 4-level limit"
        );
    }

    #[test]
    fn test_max_steps_infinite_loop_guard() {
        let program = vec![Op::Lbl("A".to_string()), Op::Gto("A".to_string())];
        let mut state = state_with_program(program);
        let result = crate::ops::program::run_program(&mut state, "A");
        assert_eq!(
            result,
            Err(HpError::Overflow),
            "Infinite loop must be caught by MAX_STEPS"
        );
    }

    #[test]
    fn test_op_isg_reg_out_of_bounds() {
        let mut state = CalcState::default();
        let result = op_isg(&mut state, 100);
        assert_eq!(result, Err(HpError::InvalidOp));
    }

    #[test]
    fn test_op_dse_reg_out_of_bounds() {
        let mut state = CalcState::default();
        let result = op_dse(&mut state, 100);
        assert_eq!(result, Err(HpError::InvalidOp));
    }

    #[test]
    fn test_op_gto_interactive_invalid() {
        let mut state = CalcState::default();
        let result = op_gto(&mut state, "A");
        assert_eq!(result, Err(HpError::InvalidOp));
    }

    #[test]
    fn test_op_xeq_interactive_invalid() {
        let mut state = CalcState::default();
        let result = op_xeq(&mut state, "A");
        assert_eq!(result, Err(HpError::InvalidOp));
    }

    #[test]
    fn test_gto_label_not_found_during_run() {
        let program = vec![Op::Lbl("A".to_string()), Op::Gto("MISSING".to_string())];
        let mut state = state_with_program(program);
        let result = crate::ops::program::run_program(&mut state, "A");
        assert_eq!(result, Err(HpError::InvalidOp));
    }

    #[test]
    fn test_parse_counter_canonical_phase3_example() {
        let n = HpNum(Decimal::from_str("1.005").unwrap());
        let (current, final_val, step, frac_padded) = parse_counter(&n).unwrap();
        assert_eq!(current, 1);
        assert_eq!(final_val, 5);
        assert_eq!(step, 1);
        assert_eq!(&frac_padded, "00500");
    }

    #[test]
    fn test_parse_counter_integer_only_register() {
        // A register with no decimal part (e.g. initialised to 5 without ISG setup):
        // frac = "" → padded = "00000" → final=0, step 00 → 1
        let n = HpNum(Decimal::from_str("5").unwrap());
        let (current, final_val, step, frac_padded) = parse_counter(&n).unwrap();
        assert_eq!(current, 5);
        assert_eq!(final_val, 0, "no decimal → final=0");
        assert_eq!(step, 1, "no decimal → step 00 → 1");
        assert_eq!(&frac_padded, "00000");
    }

    #[test]
    fn test_parse_counter_step_99_max_step() {
        // counter = 1.00099 → current=1, final=000=0, step=99
        let n = HpNum(Decimal::from_str("1.00099").unwrap());
        let (current, final_val, step, frac_padded) = parse_counter(&n).unwrap();
        assert_eq!(current, 1);
        assert_eq!(final_val, 0);
        assert_eq!(step, 99, "step field '99' must parse as 99");
        assert_eq!(&frac_padded, "00099");
    }

    #[test]
    fn test_isg_increments_and_then_skips() {
        let mut state = CalcState::default();
        state.regs[0] = HpNum(Decimal::from_str("4.005").unwrap()).into();
        let result1 = op_isg(&mut state, 0).unwrap();
        assert!(
            !result1,
            "isg at current=4 (new=5 not > final=5): must NOT skip"
        );
        let result2 = op_isg(&mut state, 0).unwrap();
        assert!(result2, "isg at current=5 (new=6 > final=5): must skip");
    }

    #[test]
    fn test_rtn_interactive_noop() {
        let mut state = CalcState::default();
        assert!(state.call_stack.is_empty());
        let result = op_rtn(&mut state);
        assert!(result.is_ok());
        assert!(state.call_stack.is_empty());
    }

    // ── execute_op and run_loop coverage tests ────────────────────────────────

    #[test]
    fn test_program_arithmetic_add() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("3").unwrap())),
            Op::PushNum(HpNum(Decimal::from_str("4").unwrap())),
            Op::Add,
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("7").unwrap()));
    }

    #[test]
    fn test_program_sub_mul_div() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("10").unwrap())),
            Op::PushNum(HpNum(Decimal::from_str("2").unwrap())),
            Op::Sub,
            Op::PushNum(HpNum(Decimal::from_str("3").unwrap())),
            Op::Mul,
            Op::PushNum(HpNum(Decimal::from_str("4").unwrap())),
            Op::Div,
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("6").unwrap()));
    }

    #[test]
    fn test_program_stack_ops() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("5").unwrap())),
            Op::Enter,
            Op::Clx,
            Op::PushNum(HpNum(Decimal::from_str("3").unwrap())),
            Op::Chs,
            Op::PushNum(HpNum(Decimal::from_str("7").unwrap())),
            Op::XySwap,
            Op::Rdn,
            Op::Lastx,
        ];
        let mut state = state_with_program(program);
        assert!(crate::ops::program::run_program(&mut state, "A").is_ok());
    }

    #[test]
    fn test_program_sto_rcl_clreg() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("42").unwrap())),
            Op::StoReg(5),
            Op::Clreg,
            Op::RclReg(5),
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum::zero());
    }

    #[test]
    fn test_program_fmt_ops() {
        use crate::state::DisplayMode;
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::FmtFix(2),
            Op::FmtSci(3),
            Op::FmtEng(4),
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.display_mode, DisplayMode::Eng(4));
    }

    #[test]
    fn test_program_alpha_ops() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::AlphaToggle,
            Op::AlphaAppend('H'),
            Op::AlphaAppend('I'),
            Op::AlphaBackspace,
            Op::AlphaClear,
            Op::AlphaToggle,
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert!(state.alpha_reg.is_empty());
    }

    #[test]
    fn test_program_math_ops() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("4").unwrap())),
            Op::Sqrt,
            Op::Sq,
            Op::Int,
            Op::Recip,
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("0.25").unwrap()));
    }

    #[test]
    fn test_program_runs_off_end() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("1").unwrap())),
        ];
        let mut state = state_with_program(program);
        let result = crate::ops::program::run_program(&mut state, "A");
        assert!(result.is_ok());
        assert!(!state.is_running);
    }

    #[test]
    fn test_program_lbl_noop_in_execution() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::Lbl("B".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("9").unwrap())),
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("9").unwrap()));
    }

    #[test]
    fn test_program_test_op_skip() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("0").unwrap())),
            Op::Test(TestKind::XNeZero),
            Op::PushNum(HpNum(Decimal::from_str("99").unwrap())),
            Op::PushNum(HpNum(Decimal::from_str("7").unwrap())),
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("7").unwrap()));
    }

    #[test]
    fn test_program_test_op_no_skip() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("0").unwrap())),
            Op::Test(TestKind::XEqZero),
            Op::PushNum(HpNum(Decimal::from_str("42").unwrap())),
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("42").unwrap()));
    }

    #[test]
    fn test_program_user_mode_toggle() {
        let program = vec![Op::Lbl("A".to_string()), Op::UserMode];
        let mut state = state_with_program(program);
        assert!(!state.user_mode);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert!(state.user_mode);
    }

    #[test]
    fn test_program_isg_inside_program() {
        // counter 0.00103 → current=0, final=1, step=3; 0+3=3 > 1 → skip
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("0.00103").unwrap())),
            Op::StoReg(0),
            Op::Isg(0),
            Op::Gto("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("5").unwrap())),
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("5").unwrap()));
    }

    #[test]
    fn test_program_dse_inside_program() {
        // counter 3.00103 → current=3, final=1, step=3; 3-3=0 <= 1 → skip
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("3.00103").unwrap())),
            Op::StoReg(0),
            Op::Dse(0),
            Op::Gto("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("8").unwrap())),
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("8").unwrap()));
    }

    #[test]
    fn test_program_xeq_subroutine_returns() {
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("1").unwrap())),
            Op::Xeq("B".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("2").unwrap())),
            Op::Rtn,
            Op::Lbl("B".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("10").unwrap())),
            Op::Rtn,
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("2").unwrap()));
    }

    #[test]
    fn test_evaluate_test_relational_variants() {
        let mut state = CalcState::default();
        state.stack.x = HpNum(Decimal::from_str("-3").unwrap());
        state.stack.y = HpNum(Decimal::from_str("5").unwrap());

        assert!(evaluate_test(&state, &TestKind::XLtZero));
        assert!(!evaluate_test(&state, &TestKind::XGtZero));
        assert!(evaluate_test(&state, &TestKind::XLeZero));
        assert!(!evaluate_test(&state, &TestKind::XGeZero));
        assert!(!evaluate_test(&state, &TestKind::XEqY));
        assert!(evaluate_test(&state, &TestKind::XNeY));
        assert!(evaluate_test(&state, &TestKind::XLtY));
        assert!(!evaluate_test(&state, &TestKind::XGtY));
        assert!(evaluate_test(&state, &TestKind::XLeY));
        assert!(!evaluate_test(&state, &TestKind::XGeY));
    }

    #[test]
    fn test_op_prgm_mode_sets_flag() {
        let mut state = CalcState::default();
        assert!(!state.prgm_mode);
        op_prgm_mode(&mut state).unwrap();
        assert!(state.prgm_mode);
    }

    #[test]
    fn test_op_lbl_interactive_noop() {
        let mut state = CalcState::default();
        assert!(op_lbl(&mut state).is_ok());
    }

    #[test]
    fn test_op_test_interactive_noop() {
        let mut state = CalcState::default();
        assert!(op_test(&mut state, TestKind::XEqZero).is_ok());
    }

    #[test]
    fn test_program_trig_and_exp_ops() {
        // Cover Op::Ln, Op::Log, Op::Exp, Op::TenPow, Op::YPow,
        //       Op::SetDeg, Op::SetRad, Op::SetGrad
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("1").unwrap())),
            Op::Exp,
            Op::Ln,
            Op::SetRad,
            Op::SetGrad,
            Op::SetDeg,
            Op::PushNum(HpNum(Decimal::from_str("100").unwrap())),
            Op::Log,
            Op::TenPow,
            Op::PushNum(HpNum(Decimal::from_str("2").unwrap())),
            Op::YPow,
        ];
        let mut state = state_with_program(program);
        assert!(crate::ops::program::run_program(&mut state, "A").is_ok());
    }

    #[test]
    fn test_program_trig_sin_cos_tan() {
        // Cover Op::Sin, Op::Cos, Op::Tan, Op::Asin, Op::Acos, Op::Atan
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("30").unwrap())),
            Op::Sin,
            Op::Asin,
            Op::PushNum(HpNum(Decimal::from_str("60").unwrap())),
            Op::Cos,
            Op::Acos,
            Op::PushNum(HpNum(Decimal::from_str("45").unwrap())),
            Op::Tan,
            Op::Atan,
        ];
        let mut state = state_with_program(program);
        assert!(crate::ops::program::run_program(&mut state, "A").is_ok());
    }

    #[test]
    fn test_program_fmt_invalid_n_errors() {
        // Cover FmtFix/FmtSci/FmtEng > 9 error paths inside execute_op
        let program_fix = vec![
            Op::Lbl("A".to_string()),
            Op::FmtFix(10), // n > 9 → InvalidOp
        ];
        let mut state = state_with_program(program_fix);
        assert_eq!(
            crate::ops::program::run_program(&mut state, "A"),
            Err(HpError::InvalidOp)
        );

        let program_sci = vec![Op::Lbl("A".to_string()), Op::FmtSci(10)];
        let mut state2 = state_with_program(program_sci);
        assert_eq!(
            crate::ops::program::run_program(&mut state2, "A"),
            Err(HpError::InvalidOp)
        );

        let program_eng = vec![Op::Lbl("A".to_string()), Op::FmtEng(10)];
        let mut state3 = state_with_program(program_eng);
        assert_eq!(
            crate::ops::program::run_program(&mut state3, "A"),
            Err(HpError::InvalidOp)
        );
    }

    #[test]
    fn test_program_sto_arith() {
        // Cover Op::StoArith inside execute_op
        use crate::ops::StoArithKind;
        let program = vec![
            Op::Lbl("A".to_string()),
            Op::PushNum(HpNum(Decimal::from_str("10").unwrap())),
            Op::StoReg(0),
            Op::PushNum(HpNum(Decimal::from_str("5").unwrap())),
            Op::StoArith {
                reg: 0,
                kind: StoArithKind::Add,
            },
            Op::RclReg(0),
        ];
        let mut state = state_with_program(program);
        crate::ops::program::run_program(&mut state, "A").unwrap();
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("15").unwrap()));
    }

    #[test]
    fn builtin_card_op_resolves_four_names() {
        use crate::ops::program::builtin_card_op;
        use crate::ops::Op;
        assert_eq!(builtin_card_op("WPRGM"), Some(Op::Wprgm));
        assert_eq!(builtin_card_op("RDPRGM"), Some(Op::Rdprgm));
        assert_eq!(builtin_card_op("WDTA"), Some(Op::Wdta));
        assert_eq!(builtin_card_op("RDTA"), Some(Op::Rdta));
        assert_eq!(
            builtin_card_op("wprgm"),
            None,
            "case-sensitive — HP-41 names are uppercase"
        );
        assert_eq!(builtin_card_op("UNKNOWN"), None);
        assert_eq!(builtin_card_op(""), None);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 25 / Plan 03 — builtin_card_op 4 → 12 extension tests.
//
// These tests live INSIDE program.rs (next to the existing `program_tests`
// module) so `builtin_card_op` is reachable via `super::*` without widening
// its `pub(super)` visibility (W1 fix from the 2026-05-14 plan revision).
// Coverage:
//   - 8 mnemonic resolutions, each in BOTH ASCII and Unicode spellings.
//   - 4 v2.1 card-reader names regression (independent of the original
//     `builtin_card_op_resolves_four_names` test above).
//   - Unknown-name returns None.
//   - Case-sensitivity (lowercase rejected).
//   - Programmatic XEQ symmetry: run_program with `Op::Xeq("X<>Y?")` resolves
//     through builtin_card_op → Op::Test dispatch end-to-end without error.
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod phase25_builtin_card_op_tests {
    use super::builtin_card_op;
    use crate::num::HpNum;
    use crate::ops::{Op, TestKind};
    use crate::state::CalcState;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn state_with_program(ops: Vec<Op>) -> CalcState {
        CalcState {
            program: ops,
            ..Default::default()
        }
    }

    /// 8 conditional-test mnemonics × 2 spellings (ASCII + Unicode) = 16
    /// assertions. Every spelling listed in the mnemonic table (RESEARCH §
    /// "Conditional tests") MUST resolve to the documented `TestKind` variant.
    #[test]
    fn resolves_8_conditional_test_mnemonics() {
        // X ≠ Y — three accepted spellings.
        assert_eq!(builtin_card_op("X<>Y?"), Some(Op::Test(TestKind::XNeY)));
        assert_eq!(
            builtin_card_op("X\u{2260}Y?"),
            Some(Op::Test(TestKind::XNeY))
        );
        assert_eq!(builtin_card_op("X#Y?"), Some(Op::Test(TestKind::XNeY)));

        // X < Y — single spelling.
        assert_eq!(builtin_card_op("X<Y?"), Some(Op::Test(TestKind::XLtY)));

        // X ≥ Y — two spellings.
        assert_eq!(builtin_card_op("X>=Y?"), Some(Op::Test(TestKind::XGeY)));
        assert_eq!(
            builtin_card_op("X\u{2265}Y?"),
            Some(Op::Test(TestKind::XGeY))
        );

        // X ≠ 0 — two spellings.
        assert_eq!(builtin_card_op("X#0?"), Some(Op::Test(TestKind::XNeZero)));
        assert_eq!(
            builtin_card_op("X\u{2260}0?"),
            Some(Op::Test(TestKind::XNeZero))
        );

        // X < 0 — single spelling.
        assert_eq!(builtin_card_op("X<0?"), Some(Op::Test(TestKind::XLtZero)));

        // X > 0 — single spelling.
        assert_eq!(builtin_card_op("X>0?"), Some(Op::Test(TestKind::XGtZero)));

        // X ≤ 0 — two spellings.
        assert_eq!(builtin_card_op("X<=0?"), Some(Op::Test(TestKind::XLeZero)));
        assert_eq!(
            builtin_card_op("X\u{2264}0?"),
            Some(Op::Test(TestKind::XLeZero))
        );

        // X ≥ 0 — two spellings.
        assert_eq!(builtin_card_op("X>=0?"), Some(Op::Test(TestKind::XGeZero)));
        assert_eq!(
            builtin_card_op("X\u{2265}0?"),
            Some(Op::Test(TestKind::XGeZero))
        );
    }

    /// Independent regression: the four v2.1 card-reader names still resolve.
    /// Mirrors `builtin_card_op_resolves_four_names` from the sibling
    /// `program_tests` module but is bound to this new module so a Plan-04
    /// (or later) refactor that splits the modules cannot orphan the check.
    #[test]
    fn preserves_4_card_reader_names() {
        assert_eq!(builtin_card_op("WPRGM"), Some(Op::Wprgm));
        assert_eq!(builtin_card_op("RDPRGM"), Some(Op::Rdprgm));
        assert_eq!(builtin_card_op("WDTA"), Some(Op::Wdta));
        assert_eq!(builtin_card_op("RDTA"), Some(Op::Rdta));
    }

    /// Unknown / empty names return None — Pitfall 9 (the caller surfaces
    /// `HpError::InvalidOp`; no "did you mean…?" hint until Phase 26).
    #[test]
    fn unknown_name_returns_none() {
        assert_eq!(builtin_card_op("foobar"), None);
        assert_eq!(builtin_card_op(""), None);
        assert_eq!(builtin_card_op("FOOBAR"), None);
        // Spelling-typo guards — these are NOT in the 12-name table.
        assert_eq!(builtin_card_op("X=Y?"), None, "X=Y? is keyboard-only (W4)");
        assert_eq!(builtin_card_op("X<>Y"), None, "missing trailing '?'");
        assert_eq!(builtin_card_op(" X<>Y? "), None, "whitespace not stripped");
    }

    /// HP-41 ROM names are uppercase — case-sensitive match enforced.
    #[test]
    fn case_sensitive_lowercase_rejected() {
        assert_eq!(builtin_card_op("wprgm"), None);
        assert_eq!(builtin_card_op("x<>y?"), None);
        assert_eq!(builtin_card_op("X<>y?"), None, "mixed case rejected");
    }

    /// Programmatic XEQ symmetry (one of the success criteria from <objective>):
    /// `Op::Xeq("X<>Y?")` inside a running program resolves through
    /// `builtin_card_op` → `dispatch(state, Op::Test(TestKind::XNeY))` without
    /// error. The XNeY test for 5 ≠ 7 is TRUE → no skip in `run_loop`;
    /// `run_program` returns Ok(()).
    #[test]
    fn programmatic_xeq_dispatches_x_ne_y() {
        let program = vec![
            Op::Lbl("TEST".to_string()),
            Op::Xeq("X<>Y?".to_string()),
            Op::Rtn,
        ];
        let mut state = state_with_program(program);
        state.stack.y = HpNum(Decimal::from_str("5").unwrap());
        state.stack.x = HpNum(Decimal::from_str("7").unwrap());

        let result = super::run_program(&mut state, "TEST");
        assert!(
            result.is_ok(),
            "Op::Xeq(\"X<>Y?\") inside a program must resolve via builtin_card_op → Op::Test dispatch without error; got {result:?}"
        );
        // is_running is reset on the success path (D-06).
        assert!(!state.is_running);
        // Stack is read-only for Op::Test (LiftEffect::Neutral) — values
        // preserved.
        assert_eq!(state.stack.x, HpNum(Decimal::from_str("7").unwrap()));
        assert_eq!(state.stack.y, HpNum(Decimal::from_str("5").unwrap()));
    }

    // ── Phase 28 / Task 6: resolver chain extension tests ──────────────────────

    // Catches: unknown XEQ name must still return InvalidOp (regression from v2.2)
    #[test]
    fn xeq_unknown_returns_invalid_op() {
        use crate::error::HpError;
        use crate::ops::program::op_xeq;
        let mut state = CalcState::new();
        let result = op_xeq(&mut state, "COMPLETELY_UNKNOWN_NAME_XYZZY");
        assert_eq!(
            result,
            Err(HpError::InvalidOp),
            "XEQ of an unknown name must return HpError::InvalidOp"
        );
    }

    // Catches: builtin_card_op must continue to take precedence over xrom (C-28.4)
    #[test]
    fn xeq_wprgm_built_in() {
        use crate::error::HpError;
        use crate::ops::program::op_xeq;
        // WPRGM is a builtin_card_op entry — must resolve to Op::Wprgm, not fall through
        // to the xrom resolver. This ensures built-ins win over XROM (C-28.4 / Pitfall 1).
        let mut state = CalcState::new();
        // Dispatch WPRGM — it writes a "WPRGM" line to pending_card_op or errors if
        // alpha_reg is empty. Either way, it must NOT return InvalidOp.
        let result = op_xeq(&mut state, "WPRGM");
        // WPRGM with empty alpha_reg returns HpError::AlphaData — that's fine.
        // The important thing is it did NOT return InvalidOp (which would mean
        // builtin_card_op was bypassed).
        assert_ne!(
            result,
            Err(HpError::InvalidOp),
            "XEQ 'WPRGM' must resolve via builtin_card_op, not fall through to InvalidOp"
        );
    }
}
