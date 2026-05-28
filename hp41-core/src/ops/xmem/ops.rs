// Algorithm independently derived from HP-41CX Owner's Manual 00041-90061 (1983) and
// HP-41CX Extended Memory Module Owner's Manual 00082-90014 (1984).
//
//! `ops` — X-MEM (Extended Memory) built-in op handlers.
//!
//! Eight operations:
//!   EMDIR / EMROOM / SAVEP / GETP / SAVED / GETD / EMREG / SAVERX
//!
//! D-51.0a ISOLATION INVARIANT: These ops MUST NEVER read or write
//! `state.regs` or `state.adv_matrices` directly.  The ONLY sanctioned
//! transfer paths to/from `state.regs` are:
//!   - SAVED → `capture_data_card(state)` (read-only snapshot)
//!   - GETD  → `load_data_card(state, card)` (wholesale replace)
//!
//! No async, no panics, no println!/eprintln!.

use super::{XmemFile, XmemKind, XMEM_CAPACITY};
use crate::cardreader::{
    capture_data_card, decode_data, decode_program, encode_data, encode_program,
    insert_program_ops, load_data_card,
};
use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;
use rust_decimal::prelude::ToPrimitive;

// ── Private helpers ────────────────────────────────────────────────────────────

/// Validate the ALPHA register is non-empty and return its value as a file name.
/// Returns `Err(HpError::AlphaData)` on empty alpha_reg.
fn alpha_name(state: &CalcState) -> Result<String, HpError> {
    if state.alpha_reg.is_empty() {
        return Err(HpError::AlphaData);
    }
    Ok(state.alpha_reg.clone())
}

/// Find a file by name (immutable).
fn find_file<'a>(state: &'a CalcState, name: &str) -> Option<&'a XmemFile> {
    state.xmem_files.iter().find(|f| f.name == name)
}

/// Find a file by name (mutable).
fn find_file_mut<'a>(state: &'a mut CalcState, name: &str) -> Option<&'a mut XmemFile> {
    state.xmem_files.iter_mut().find(|f| f.name == name)
}

/// Insert or overwrite a file by name (D-51.6 overwrite-in-place semantics).
/// If a file with the same name exists, replaces it; otherwise appends.
/// xmem_files.len() is unchanged on overwrite.
fn upsert_file(state: &mut CalcState, file: XmemFile) {
    if let Some(existing) = state.xmem_files.iter_mut().find(|f| f.name == file.name) {
        *existing = file;
    } else {
        state.xmem_files.push(file);
    }
}

/// Return the total X-MEM registers currently used across all files.
fn registers_used(state: &CalcState) -> usize {
    state.xmem_files.iter().map(|f| f.register_count()).sum()
}

/// Return the register count of an existing file (0 if not found).
/// Used in the capacity-check-before-mutate guard for overwrites.
fn existing_file_regs(state: &CalcState, name: &str) -> usize {
    find_file(state, name)
        .map(|f| f.register_count())
        .unwrap_or(0)
}

/// Extract a zero-based register index from `state.stack.x`.
/// Truncates to the integer part — HP-41 register-addressing convention, so
/// `2.7` resolves to register 2. Returns `Err(HpError::OutOfRange)` only when
/// the value is negative (no valid `usize`); the caller bounds-checks the upper end.
/// NEVER uses floor/fmod — integer part via Decimal::trunc (CLAUDE.md invariant).
fn index_from_x(state: &CalcState) -> Result<usize, HpError> {
    let truncated = state.stack.x.trunc_int();
    truncated.inner().to_usize().ok_or(HpError::OutOfRange)
}

// ── X-MEM Ops ─────────────────────────────────────────────────────────────────

/// EMDIR — list all X-MEM files to print_buffer.
///
/// Pushes one header line, one line per file (name / type / register_count),
/// and one footer line showing registers available.
/// LiftEffect: Neutral (catalog only; no stack change).
///
/// # Errors
/// This op always succeeds — returns `Ok(())`.
pub fn op_emdir(state: &mut CalcState) -> Result<(), HpError> {
    let available = XMEM_CAPACITY.saturating_sub(registers_used(state));
    state
        .print_buffer
        .push(format!("{:<24}", "-- XMEM DIRECTORY --"));
    for file in &state.xmem_files {
        let kind_tag = match file.kind {
            XmemKind::Program => "PGM",
            XmemKind::Data => "DAT",
        };
        state.print_buffer.push(format!(
            "{:<10} {:>3} {:>5}",
            file.name,
            kind_tag,
            file.register_count()
        ));
    }
    state.print_buffer.push(format!("{} REGS FREE", available));
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// EMROOM — push available X-MEM register count (XMEM_CAPACITY − used) onto X.
///
/// LiftEffect: Enable (recall-style push to X).
///
/// # Errors
/// Always succeeds.
pub fn op_emroom(state: &mut CalcState) -> Result<(), HpError> {
    let available = XMEM_CAPACITY.saturating_sub(registers_used(state));
    apply_lift_effect(state, LiftEffect::Enable);
    enter_number(state, HpNum::from(available as i32));
    Ok(())
}

/// SAVEP — save the current program to a named X-MEM PROGRAM file.
///
/// File name is taken from `state.alpha_reg`. Encodes via `encode_program`.
/// D-51.6: overwrites an existing file with the same name in place.
/// Capacity is checked BEFORE mutation (Pitfall 3).
/// LiftEffect: Neutral.
///
/// # Errors
/// - `HpError::AlphaData` — empty ALPHA register
/// - `HpError::NoRoom` — file would push total over XMEM_CAPACITY
/// - Propagates `HpError` from `encode_program`
pub fn op_savep(state: &mut CalcState) -> Result<(), HpError> {
    let name = alpha_name(state)?;
    let bytes = encode_program(&state.program)?;
    let new_file = XmemFile {
        name: name.clone(),
        kind: XmemKind::Program,
        data: bytes,
        reg_count: 0,
    };
    // Capacity check BEFORE mutation (Research Pitfall 3 / D-51.1).
    let used_after = registers_used(state)
        .saturating_sub(existing_file_regs(state, &name))
        .saturating_add(new_file.register_count());
    if used_after > XMEM_CAPACITY {
        return Err(HpError::NoRoom);
    }
    upsert_file(state, new_file);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// GETP — retrieve a named X-MEM PROGRAM file and insert it (RDPRGM semantics).
///
/// Decodes via `decode_program`; inserts via `insert_program_ops`.
/// LiftEffect: Neutral.
///
/// # Errors
/// - `HpError::AlphaData` — empty ALPHA register
/// - `HpError::FileNotFound` — no file with the given name
/// - `HpError::FileType` — file exists but is not a PROGRAM file
/// - Propagates `HpError` from `decode_program`
pub fn op_getp(state: &mut CalcState) -> Result<(), HpError> {
    let name = alpha_name(state)?;
    let file = find_file(state, &name).ok_or(HpError::FileNotFound)?;
    if file.kind != XmemKind::Program {
        return Err(HpError::FileType);
    }
    let ops = decode_program(&file.data.clone())?;
    insert_program_ops(state, ops);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// SAVED — save data registers R00..R(SIZE-1) to a named X-MEM DATA file.
///
/// Captures via `capture_data_card`, encodes via `encode_data`.
/// Sets `state.xmem_active_file = Some(name)` (D-51.4).
/// D-51.6: overwrites an existing file with the same name in place.
/// Capacity is checked BEFORE mutation (Pitfall 3).
/// LiftEffect: Neutral.
///
/// # Errors
/// - `HpError::AlphaData` — empty ALPHA register
/// - `HpError::NoRoom` — file would push total over XMEM_CAPACITY
/// - Propagates `HpError` from `encode_data`
pub fn op_saved(state: &mut CalcState) -> Result<(), HpError> {
    let name = alpha_name(state)?;
    let card = capture_data_card(state);
    let reg_count = card.registers.len();
    let bytes = encode_data(&card)?;
    let new_file = XmemFile {
        name: name.clone(),
        kind: XmemKind::Data,
        data: bytes,
        reg_count,
    };
    // Capacity check BEFORE mutation (Research Pitfall 3 / D-51.1).
    let used_after = registers_used(state)
        .saturating_sub(existing_file_regs(state, &name))
        .saturating_add(new_file.register_count());
    if used_after > XMEM_CAPACITY {
        return Err(HpError::NoRoom);
    }
    upsert_file(state, new_file);
    state.xmem_active_file = Some(name);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// GETD — retrieve a named X-MEM DATA file and replace state.regs wholesale.
///
/// Decodes via `decode_data`; loads via `load_data_card`.
/// Sets `state.xmem_active_file = Some(name)` (D-51.4).
/// LiftEffect: Neutral.
///
/// # Errors
/// - `HpError::AlphaData` — empty ALPHA register
/// - `HpError::FileNotFound` — no file with the given name
/// - `HpError::FileType` — file exists but is not a DATA file
/// - Propagates `HpError` from `decode_data`
pub fn op_getd(state: &mut CalcState) -> Result<(), HpError> {
    let name = alpha_name(state)?;
    let file = find_file(state, &name).ok_or(HpError::FileNotFound)?;
    if file.kind != XmemKind::Data {
        return Err(HpError::FileType);
    }
    let card = decode_data(&file.data.clone())?;
    load_data_card(state, card);
    state.xmem_active_file = Some(name);
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

/// EMREG — recall register N (from X) of the active X-MEM DATA file → push to X.
///
/// N is taken from `state.stack.x` via `trunc_int` (never floor/fmod).
/// The active file is identified by `state.xmem_active_file`.
/// LiftEffect: Enable (pushes recall value onto X).
///
/// # Errors
/// - `HpError::FileNotFound` — no active file set, or active file not found
/// - `HpError::FileType` — active file is a PROGRAM file
/// - `HpError::OutOfRange` — N is negative or >= reg count (fractional N truncates)
/// - Propagates `HpError` from `decode_data`
pub fn op_emreg(state: &mut CalcState) -> Result<(), HpError> {
    let n = index_from_x(state)?;
    let name = state
        .xmem_active_file
        .clone()
        .ok_or(HpError::FileNotFound)?;
    let file = find_file(state, &name).ok_or(HpError::FileNotFound)?;
    if file.kind != XmemKind::Data {
        return Err(HpError::FileType);
    }
    let card = decode_data(&file.data.clone())?;
    let reg = card.registers.get(n).ok_or(HpError::OutOfRange)?;
    let value = reg.numeric_or_zero();
    apply_lift_effect(state, LiftEffect::Enable);
    enter_number(state, value);
    Ok(())
}

/// SAVERX — store Y into register N (from X) of the active X-MEM DATA file.
///
/// N is taken from `state.stack.x` (X = index, Y = value to store).
/// Decodes the active DATA file, mutates register N, re-encodes and writes back.
/// LiftEffect: Neutral.
///
/// # Errors
/// - `HpError::FileNotFound` — no active file set, or active file not found
/// - `HpError::FileType` — active file is a PROGRAM file
/// - `HpError::OutOfRange` — N is negative or >= reg count (fractional N truncates)
/// - Propagates `HpError` from `decode_data` / `encode_data`
pub fn op_saverx(state: &mut CalcState) -> Result<(), HpError> {
    let n = index_from_x(state)?;
    let name = state
        .xmem_active_file
        .clone()
        .ok_or(HpError::FileNotFound)?;
    // Validate existence and kind before decoding
    {
        let file = find_file(state, &name).ok_or(HpError::FileNotFound)?;
        if file.kind != XmemKind::Data {
            return Err(HpError::FileType);
        }
    }
    // Decode, mutate, re-encode, write back
    let data = find_file(state, &name)
        .expect("file was just validated")
        .data
        .clone();
    let mut card = decode_data(&data)?;
    let reg = card.registers.get_mut(n).ok_or(HpError::OutOfRange)?;
    *reg = state.stack.y.clone().into();
    let new_bytes = encode_data(&card)?;
    // Write back via find_file_mut (upsert semantics preserved)
    if let Some(file) = find_file_mut(state, &name) {
        file.data = new_bytes;
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::cardreader::DataCard;
    use crate::num::{HpNum, HpValue};
    use crate::ops::Op;
    use crate::state::CalcState;

    // Helper: create a CalcState with alpha_reg set to the given name.
    fn state_with_alpha(name: &str) -> CalcState {
        let mut state = CalcState::new();
        state.alpha_reg = name.to_string();
        state
    }

    // Helper: save a trivial DATA file ("DAT1") with N registers set to
    // the given value. Returns the state with the file in xmem_files and
    // xmem_active_file = Some("DAT1").
    fn state_with_data_file(reg_values: &[i32]) -> CalcState {
        let mut state = state_with_alpha("DAT1");
        for (i, &v) in reg_values.iter().enumerate() {
            if i < state.regs.len() {
                state.regs[i] = HpValue::from(v);
            }
        }
        op_saved(&mut state).unwrap();
        state
    }

    // ── EMDIR tests ────────────────────────────────────────────────────────────

    /// EMDIR on an empty store: pushes header + footer only; no file lines.
    #[test]
    fn emdir() {
        let mut state = CalcState::new();
        op_emdir(&mut state).unwrap();
        // Must have at least header + footer
        assert!(state.print_buffer.len() >= 2, "expected header + footer");
        // Footer must mention free registers
        let footer = state.print_buffer.last().unwrap();
        assert!(
            footer.contains("REGS FREE"),
            "footer must show free registers, got: {footer:?}"
        );
        // Footer must show 600 (empty store)
        assert!(
            footer.contains("600"),
            "empty store must show 600 free, got: {footer:?}"
        );
        // No println! guard — print_buffer used, not stdout
    }

    // ── EMROOM tests ───────────────────────────────────────────────────────────

    /// EMROOM on an empty store pushes 600 onto X.
    /// After saving a 5-register DATA file (6 registers incl. header), pushes 594.
    #[test]
    fn emroom() {
        let mut state = CalcState::new();
        op_emroom(&mut state).unwrap();
        assert_eq!(
            state.stack.x,
            HpNum::from(600i32),
            "empty store must report 600"
        );

        // Now save a DATA file with exactly 5 registers
        state.alpha_reg = "DAT1".to_string();
        // Set R00..R04 to non-zero values
        for i in 0..5 {
            state.regs[i] = HpValue::from(i as i32 + 1);
        }
        op_saved(&mut state).unwrap();
        // The DATA file has reg_count == state.regs.len() (100 regs — all captured).
        // register_count() = 100 + 1 = 101; available = 600 - 101 = 499
        let file_regs = state.xmem_files[0].register_count();
        let expected = 600u32.saturating_sub(file_regs as u32) as i32;

        // Reset X so we can verify EMROOM result
        state.stack.x = HpNum::from(0i32);
        op_emroom(&mut state).unwrap();
        assert_eq!(
            state.stack.x,
            HpNum::from(expected),
            "emroom after data file must report {expected}"
        );
    }

    // ── SAVEP / GETP round-trip ────────────────────────────────────────────────

    /// Save a program and retrieve it — the Op vector survives the round-trip.
    #[test]
    fn savep_round_trip() {
        let mut state = state_with_alpha("PRG1");
        state.program = vec![Op::Add, Op::Sub, Op::Mul];
        op_savep(&mut state).unwrap();
        assert_eq!(state.xmem_files.len(), 1);
        assert_eq!(state.xmem_files[0].name, "PRG1");
        assert_eq!(state.xmem_files[0].kind, XmemKind::Program);
    }

    /// Retrieve a stored program — Op vector is identical after encode→decode.
    #[test]
    fn getp_round_trip() {
        let mut state = state_with_alpha("PRG1");
        let original = vec![Op::Add, Op::Sub, Op::Mul];
        state.program = original.clone();
        op_savep(&mut state).unwrap();

        // Clear program, then GETP should restore it
        state.program = vec![];
        state.alpha_reg = "PRG1".to_string();
        op_getp(&mut state).unwrap();
        assert_eq!(
            state.program, original,
            "GETP must restore the original program"
        );
    }

    // ── SAVED / GETD round-trip ────────────────────────────────────────────────

    /// Save data registers and verify xmem_files gain a DATA file + active_file set.
    #[test]
    fn saved_round_trip() {
        let mut state = state_with_alpha("DAT1");
        state.regs[0] = HpValue::from(42i32);
        state.regs[5] = HpValue::from(-7i32);
        op_saved(&mut state).unwrap();

        assert_eq!(state.xmem_files.len(), 1);
        assert_eq!(state.xmem_files[0].name, "DAT1");
        assert_eq!(state.xmem_files[0].kind, XmemKind::Data);
        assert_eq!(
            state.xmem_files[0].reg_count,
            state.regs.len(),
            "reg_count must equal captured reg count"
        );
        assert_eq!(state.xmem_active_file, Some("DAT1".to_string()));
    }

    /// GETD replaces state.regs wholesale (len >= 100), sets xmem_active_file.
    #[test]
    fn getd_round_trip() {
        let original_val = HpValue::from(99i32);
        let mut state = state_with_alpha("DAT1");
        state.regs[3] = original_val.clone();
        op_saved(&mut state).unwrap();

        // Overwrite reg 3 then GETD to verify restore
        state.regs[3] = HpValue::from(0i32);
        state.alpha_reg = "DAT1".to_string();
        op_getd(&mut state).unwrap();

        assert_eq!(state.regs[3], original_val, "GETD must restore reg 3");
        assert!(
            state.regs.len() >= 100,
            "regs.len() must be >= 100 after GETD"
        );
        assert_eq!(state.xmem_active_file, Some("DAT1".to_string()));
    }

    // ── EMREG / SAVERX ────────────────────────────────────────────────────────

    /// EMREG recalls register N onto X; SAVERX stores Y into register N.
    #[test]
    fn emreg_saverx() {
        let mut state = CalcState::new();
        state.alpha_reg = "DAT1".to_string();
        for (i, &v) in [10i32, 20, 30, 40, 50].iter().enumerate() {
            state.regs[i] = HpValue::from(v);
        }
        op_saved(&mut state).unwrap();

        // EMREG register 2 → should be 30 (regs[2] = 30)
        state.stack.x = HpNum::from(2i32);
        op_emreg(&mut state).unwrap();
        assert_eq!(
            state.stack.x,
            HpNum::from(30i32),
            "EMREG[2] must return regs[2] = 30"
        );

        // SAVERX: store 99 into register 1 (X = index 1, Y = 99)
        state.stack.x = HpNum::from(1i32);
        state.stack.y = HpNum::from(99i32);
        op_saverx(&mut state).unwrap();

        // Verify register 1 is now 99 via EMREG
        state.stack.x = HpNum::from(1i32);
        op_emreg(&mut state).unwrap();
        assert_eq!(
            state.stack.x,
            HpNum::from(99i32),
            "SAVERX must store Y into register N"
        );

        // xmem_files.len() must remain 1 after SAVERX
        assert_eq!(
            state.xmem_files.len(),
            1,
            "SAVERX must not change xmem_files.len()"
        );
    }

    // ── Error: SAVEP / SAVED no room ──────────────────────────────────────────

    /// Saving a file that would exceed XMEM_CAPACITY returns NoRoom; xmem_files unchanged.
    #[test]
    fn savep_no_room() {
        let mut state = CalcState::new();

        // Pack xmem_files to just below capacity with fake DATA files.
        // Each DATA file with reg_count = 98 occupies 99 registers (98 + 1 header).
        // 6 × 99 = 594 registers used; 600 - 594 = 6 free.
        // A trivial program encodes to at least 1 byte → register_count() = 1 + 1 = 2 regs.
        // So 6 - 2 = 4 free — still fits if we use exactly 6 files.
        // Instead, fill to 598 used (leaving 2 free), then try to save a program
        // that needs >= 3 registers.
        //
        // Build two fake DATA files: reg_count = 298 → 299 registers each → 598 total.
        for i in 0..2 {
            let card = DataCard {
                format: crate::cardreader::data::FORMAT_TAG.to_string(),
                version: crate::cardreader::data::FORMAT_VERSION,
                registers: vec![HpValue::default(); 298],
            };
            let bytes = encode_data(&card).unwrap();
            state.xmem_files.push(XmemFile {
                name: format!("F{i}"),
                kind: XmemKind::Data,
                data: bytes,
                reg_count: 298,
            });
        }
        // 598 registers used; 2 free. Now a program of 15 bytes needs
        // div_ceil(15, 7) + 1 = 3 registers — doesn't fit.
        state.program = vec![Op::Add; 15]; // enough ops to exceed 2-register budget
        state.alpha_reg = "P1".to_string();

        let err = op_savep(&mut state).unwrap_err();
        assert_eq!(
            err,
            HpError::NoRoom,
            "savep must return NoRoom when capacity exceeded"
        );
        // xmem_files must NOT have grown
        assert_eq!(
            state.xmem_files.len(),
            2,
            "xmem_files must not grow on NoRoom"
        );
    }

    // ── Error: GETP file not found ─────────────────────────────────────────────

    #[test]
    fn getp_not_found() {
        let mut state = state_with_alpha("NOEXIST");
        let err = op_getp(&mut state).unwrap_err();
        assert_eq!(err, HpError::FileNotFound);
    }

    // ── Error: GETP type mismatch ──────────────────────────────────────────────

    /// GETP on a DATA file returns FileType.
    #[test]
    fn getp_type_mismatch() {
        let mut state = state_with_data_file(&[1, 2, 3]);
        state.alpha_reg = "DAT1".to_string();
        let err = op_getp(&mut state).unwrap_err();
        assert_eq!(err, HpError::FileType);
    }

    // ── Error: GETD type mismatch ──────────────────────────────────────────────

    /// GETD on a PROGRAM file returns FileType.
    #[test]
    fn getd_type_mismatch() {
        let mut state = state_with_alpha("PRG1");
        state.program = vec![Op::Add, Op::Sub];
        op_savep(&mut state).unwrap();

        state.alpha_reg = "PRG1".to_string();
        let err = op_getd(&mut state).unwrap_err();
        assert_eq!(err, HpError::FileType);
    }

    // ── Error: EMREG no active file ───────────────────────────────────────────

    #[test]
    fn emreg_no_active() {
        let mut state = CalcState::new();
        state.stack.x = HpNum::from(0i32);
        let err = op_emreg(&mut state).unwrap_err();
        assert_eq!(err, HpError::FileNotFound);
    }

    // ── Error: EMREG out of range ─────────────────────────────────────────────

    /// EMREG with N >= reg count returns OutOfRange.
    #[test]
    fn emreg_out_of_range() {
        let mut state = state_with_data_file(&[1, 2, 3]);
        // Active file "DAT1" has reg_count = 100 (CalcState::new() default)
        // Request index 9999 — out of range
        state.stack.x = HpNum::from(9999i32);
        let err = op_emreg(&mut state).unwrap_err();
        assert_eq!(err, HpError::OutOfRange);
    }

    // ── Error: SAVERX out of range ────────────────────────────────────────────

    /// SAVERX with N >= reg count returns OutOfRange (own guard, separate from EMREG).
    #[test]
    fn saverx_out_of_range() {
        let mut state = state_with_data_file(&[1, 2, 3]);
        state.stack.x = HpNum::from(9999i32); // index out of range
        state.stack.y = HpNum::from(42i32);
        let err = op_saverx(&mut state).unwrap_err();
        assert_eq!(err, HpError::OutOfRange);
    }

    // ── Error: negative index (EMREG / SAVERX) ────────────────────────────────

    /// EMREG with a negative index returns OutOfRange (no valid usize).
    #[test]
    fn emreg_negative_index() {
        let mut state = state_with_data_file(&[1, 2, 3]);
        state.stack.x = HpNum::from(-1i32);
        let err = op_emreg(&mut state).unwrap_err();
        assert_eq!(err, HpError::OutOfRange);
    }

    /// SAVERX with a negative index returns OutOfRange.
    #[test]
    fn saverx_negative_index() {
        let mut state = state_with_data_file(&[1, 2, 3]);
        state.stack.x = HpNum::from(-5i32);
        state.stack.y = HpNum::from(7i32);
        let err = op_saverx(&mut state).unwrap_err();
        assert_eq!(err, HpError::OutOfRange);
    }

    // ── Fractional index truncates (HP-41 register-addressing convention) ──────

    /// EMREG truncates a fractional index to its integer part: X = 2.7 recalls
    /// register 2, not an error and not register 3.
    #[test]
    fn emreg_fractional_index_truncates() {
        use rust_decimal::Decimal;
        let mut state = CalcState::new();
        state.alpha_reg = "DAT1".to_string();
        for (i, &v) in [10i32, 20, 30, 40, 50].iter().enumerate() {
            state.regs[i] = HpValue::from(v);
        }
        op_saved(&mut state).unwrap();

        state.stack.x = HpNum::rounded(Decimal::new(27, 1)); // 2.7
        op_emreg(&mut state).unwrap();
        assert_eq!(
            state.stack.x,
            HpNum::from(30i32),
            "EMREG must truncate 2.7 → register 2 (= 30)"
        );
    }

    // ── Error: SAVED no room ──────────────────────────────────────────────────

    /// SAVED that would exceed XMEM_CAPACITY returns NoRoom; xmem_files and the
    /// active-file pointer are left unchanged (own capacity guard, separate from SAVEP).
    #[test]
    fn saved_no_room() {
        let mut state = CalcState::new();
        // Two fake DATA files of 298 regs each → 299 registers each (incl. header)
        // = 598 used, 2 free.
        for i in 0..2 {
            let card = DataCard {
                format: crate::cardreader::data::FORMAT_TAG.to_string(),
                version: crate::cardreader::data::FORMAT_VERSION,
                registers: vec![HpValue::default(); 298],
            };
            let bytes = encode_data(&card).unwrap();
            state.xmem_files.push(XmemFile {
                name: format!("F{i}"),
                kind: XmemKind::Data,
                data: bytes,
                reg_count: 298,
            });
        }
        // SAVED captures the full register set (100 regs by default) — far more
        // than the 2 free registers remaining.
        state.alpha_reg = "DNEW".to_string();
        let err = op_saved(&mut state).unwrap_err();
        assert_eq!(
            err,
            HpError::NoRoom,
            "saved must return NoRoom when capacity exceeded"
        );
        assert_eq!(
            state.xmem_files.len(),
            2,
            "xmem_files must not grow on NoRoom"
        );
        assert_ne!(
            state.xmem_active_file,
            Some("DNEW".to_string()),
            "active file must not be set when SAVED fails"
        );
    }

    // ── D-51.6: Overwrite in place ────────────────────────────────────────────

    /// Saving a file with a duplicate name overwrites in place; xmem_files.len() unchanged.
    #[test]
    fn overwrite_in_place() {
        let mut state = state_with_alpha("PRG1");
        state.program = vec![Op::Add];
        op_savep(&mut state).unwrap();
        assert_eq!(state.xmem_files.len(), 1);

        // Save again with same name but different program
        state.alpha_reg = "PRG1".to_string();
        state.program = vec![Op::Sub, Op::Mul];
        op_savep(&mut state).unwrap();
        assert_eq!(
            state.xmem_files.len(),
            1,
            "overwrite must not grow xmem_files"
        );

        // Verify the file was updated (retrieve and check)
        state.program = vec![];
        state.alpha_reg = "PRG1".to_string();
        op_getp(&mut state).unwrap();
        assert_eq!(
            state.program,
            vec![Op::Sub, Op::Mul],
            "overwritten program must be the new one"
        );
    }

    // ── Boundary: capacity exactly 600 succeeds; one over → NoRoom ───────────

    /// A store that fills exactly to 600 registers succeeds; EMROOM then == 0.
    /// The next store (however small) returns NoRoom.
    #[test]
    fn emroom_capacity_boundary() {
        let mut state = CalcState::new();

        // Fill to exactly 600 with two DATA files: reg_count = 299 each → 300 each → 600 total.
        for i in 0..2 {
            let card = DataCard {
                format: crate::cardreader::data::FORMAT_TAG.to_string(),
                version: crate::cardreader::data::FORMAT_VERSION,
                registers: vec![HpValue::default(); 299],
            };
            let bytes = encode_data(&card).unwrap();
            state.xmem_files.push(XmemFile {
                name: format!("G{i}"),
                kind: XmemKind::Data,
                data: bytes,
                reg_count: 299,
            });
        }
        // Verify total = 600
        let total: usize = state.xmem_files.iter().map(|f| f.register_count()).sum();
        assert_eq!(total, 600, "setup must fill exactly 600 registers");

        // EMROOM must return 0
        op_emroom(&mut state).unwrap();
        assert_eq!(
            state.stack.x,
            HpNum::from(0i32),
            "EMROOM at capacity must return 0"
        );

        // Saving any program now must fail with NoRoom
        state.alpha_reg = "OVERFLOW".to_string();
        state.program = vec![Op::Add]; // even a 1-op program needs 2 registers
        let err = op_savep(&mut state).unwrap_err();
        assert_eq!(
            err,
            HpError::NoRoom,
            "save past capacity must return NoRoom"
        );
    }
}
