# Phase 51: X-MEM Core - Pattern Map

**Mapped:** 2026-05-28
**Files analyzed:** 8 new/modified files + 3 read-only reference files
**Analogs found:** 8 / 8

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-core/src/ops/xmem/mod.rs` | model + utility | CRUD | `hp41-core/src/ops/advantage/mod.rs` (AdvMatrix struct, lines 100–130) | exact |
| `hp41-core/src/ops/xmem/ops.rs` | service | CRUD + file-I/O | `hp41-core/src/ops/cardreader_ops.rs` + `hp41-core/src/ops/time/alarm.rs:321` | role-match |
| `hp41-core/src/state.rs` | model | CRUD | `hp41-core/src/state.rs` lines 356–413, 500–513 (adv_matrices / adv_tvm_state pattern) | exact |
| `hp41-core/src/ops/mod.rs` | config | request-response | `hp41-core/src/ops/mod.rs` lines 1281–1305 (AdvBinin..AdvBitTest block) | exact |
| `hp41-core/src/ops/program.rs` | config | request-response | `hp41-core/src/ops/program.rs` lines 1122–1139 (AdvBinin..AdvMr block) | exact |
| `hp41-cli/src/prgm_display.rs` | utility | request-response | `hp41-cli/src/prgm_display.rs` lines 364–376 (AdvBinin..AdvBitTest block) | exact |
| `hp41-gui/src-tauri/src/prgm_display.rs` | utility | request-response | `hp41-gui/src-tauri/src/prgm_display.rs` lines 382–394 (AdvBinin..AdvBitTest block) | exact |
| `hp41-core/src/error.rs` | model | request-response | `hp41-core/src/error.rs` (existing HpError variants + thiserror Display pattern) | exact |

---

## Pattern Assignments

### `hp41-core/src/ops/xmem/mod.rs` (model, CRUD)

**Analog:** `hp41-core/src/ops/advantage/mod.rs`

**Imports pattern** (lines 38–39 of analog):
```rust
use crate::num::HpNum;
use serde::{Deserialize, Serialize};
```
For xmem/mod.rs, add `use crate::error::HpError;` and remove the HpNum import (XmemFile.data is `Vec<u8>`, not `Vec<HpNum>`).

**Struct pattern** (analog lines 100–122 — `AdvMatrix`):
```rust
/// Named matrix entry for Advantage Pac X-MEM model (ADV-FW-04 / D-43.1).
///
/// D-43.5 ISOLATION INVARIANT: This struct has no connection to
/// `state.matrix_dim` or `state.matrix_active_reg`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdvMatrix {
    /// ALPHA-register name identifying this matrix.
    pub name: String,
    /// Number of rows (1..=ADV_MATRIX_MAX_ROWS).
    pub rows: u8,
    /// ...
    pub data: Vec<HpNum>,
}
```
XmemFile must replicate this shape exactly — `#[derive(Debug, Clone, Serialize, Deserialize, Default)]`, `pub name: String`, `pub data: Vec<u8>` — and add:
- `pub kind: XmemKind,`
- `#[serde(default)] pub reg_count: usize,` (DATA-file register count stored at save time — avoids decoding on every EMROOM call, Research Pitfall 1)

**Enum + const pattern** (analog lines 124–137 — constants after struct):
```rust
pub const ADV_MATRIX_MAX_ROWS: u8 = 255;
pub const ADV_MATRIX_MAX_COLS: u8 = 255;
pub const ADV_WORD_MASK: u64 = 0x0000_000F_FFFF_FFFF;
```
XmemFile equivalent:
```rust
pub enum XmemKind { Program, Data }
pub const XMEM_CAPACITY: usize = 600; // 124 + 2×238 (two 82181A modules)
```
`XmemKind` must also derive `Debug, Clone, Serialize, Deserialize, Default, PartialEq`.

**register_count() impl pattern** (no analog — new logic; use div_ceil from Research):
```rust
impl XmemFile {
    pub fn register_count(&self) -> usize {
        match self.kind {
            XmemKind::Program => self.data.len().div_ceil(7) + 1,
            XmemKind::Data => self.reg_count + 1,
        }
    }
}
```

**Test pattern** (analog lines 139–173 — `#[cfg(test)] #[allow(clippy::unwrap_used)] mod tests`):
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn adv_matrix_default_is_empty() {
        let m = AdvMatrix::default();
        assert_eq!(m.rows, 0);
        assert!(m.data.is_empty());
        assert!(m.name.is_empty());
    }
}
```
Mirror this for XmemFile: test that `XmemFile::default()` has empty name, Program kind, empty data, reg_count 0; and that `register_count()` returns 1 (0 bytes / 7 rounds up to 0, +1 header) for a default Program file.

---

### `hp41-core/src/ops/xmem/ops.rs` (service, CRUD + file-I/O)

**Analog:** `hp41-core/src/ops/cardreader_ops.rs` (ALPHA-name validation, LiftEffect, error returns) + `hp41-core/src/ops/time/alarm.rs:321` (print_buffer catalog output)

**Imports pattern** (cardreader_ops.rs lines 15–18):
```rust
use crate::cardreader::CardOpRequest;
use crate::error::HpError;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;
```
For ops.rs, replace CardOpRequest with xmem types and add cardreader helpers:
```rust
use crate::cardreader::{
    capture_data_card, decode_data, encode_data, encode_program, decode_program,
    insert_program_ops, load_data_card,
};
use crate::error::HpError;
use crate::stack::{apply_lift_effect, LiftEffect};
use crate::state::CalcState;
use super::{XmemFile, XmemKind, XMEM_CAPACITY};
```

**ALPHA-name validation pattern** (cardreader_ops.rs lines 20–25):
```rust
fn alpha_name(state: &CalcState) -> Result<String, HpError> {
    if state.alpha_reg.is_empty() {
        return Err(HpError::AlphaData);
    }
    Ok(state.alpha_reg.clone())
}
```
Copy verbatim — EMDIR/EMROOM do NOT need alpha_name (they operate on the whole store, not a named file); SAVEP/GETP/SAVED/GETD/EMREG/SAVERX all call it.

**Core op structure pattern** (cardreader_ops.rs lines 36–66):
```rust
pub fn op_wdta(state: &mut CalcState) -> Result<(), HpError> {
    ensure_no_pending(state)?;
    let name = alpha_name(state)?;
    state.pending_card_op = Some(CardOpRequest::WriteData { name });
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```
Every xmem op follows the same skeleton: validate → compute/lookup → mutate state → `apply_lift_effect(state, LiftEffect::XXX)` → `Ok(())`. LiftEffect for each op:
- EMDIR: `Neutral` (catalog, no stack change)
- EMROOM: `Enable` (pushes number to X)
- SAVEP: `Neutral` (store op)
- GETP: `Neutral` (load op — program replaces memory; no X change)
- SAVED: `Neutral` (store op)
- GETD: `Neutral` (load op — regs replace state.regs; no X change)
- EMREG: `Enable` (recall pushes register value to X)
- SAVERX: `Neutral` (store X into register)

**EMDIR print_buffer catalog pattern** (alarm.rs lines 321–333):
```rust
pub fn op_almcat(state: &mut CalcState) -> Result<(), HpError> {
    state.alarm_catalog_mode = true;
    if let Some(first) = state.alarms.first() {
        // ...
        state.print_buffer.push(line);
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```
And the more complete catalog pattern (program.rs lines 303–309):
```rust
pub fn op_catalog(state: &mut CalcState, n: u8) -> Result<(), HpError> {
    state
        .print_buffer
        .push(format!("{:<24}", format!("-- CATALOG {n} --")));
    // ... iterate and push
```
EMDIR iterates `state.xmem_files`, pushes one formatted line per file, then a footer showing registers available. Never use `println!` — only `state.print_buffer.push(...)`.

**Capacity-check-before-mutate anti-pattern guard** (Research Pitfall 3):
```rust
// CORRECT: check BEFORE mutating
let used_after = registers_used(state)
    - existing_file_regs(state, &name)
    + new_file.register_count();
if used_after > XMEM_CAPACITY {
    return Err(HpError::NoRoom);
}
upsert_file(state, new_file);  // mutate only after check passes
```

**File lookup + kind guard pattern** (new — no prior analog; derived from FileType/FileNotFound semantics):
```rust
fn find_file<'a>(state: &'a CalcState, name: &str) -> Option<&'a XmemFile> {
    state.xmem_files.iter().find(|f| f.name == name)
}

fn find_file_mut<'a>(state: &'a mut CalcState, name: &str) -> Option<&'a mut XmemFile> {
    state.xmem_files.iter_mut().find(|f| f.name == name)
}

fn upsert_file(state: &mut CalcState, file: XmemFile) {
    if let Some(existing) = state.xmem_files.iter_mut().find(|f| f.name == file.name) {
        *existing = file;
    } else {
        state.xmem_files.push(file);
    }
}
```

**Test structure pattern** (cardreader_ops.rs lines 68–144 — unit tests in same file):
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::state::CalcState;

    #[test]
    fn wdta_with_empty_alpha_returns_alpha_data() {
        let mut state = CalcState::new();
        assert_eq!(op_wdta(&mut state), Err(HpError::AlphaData));
    }
}
```
All unit tests for the 8 xmem ops belong in `ops.rs` under the same `#[cfg(test)] #[allow(clippy::unwrap_used)] mod tests` block. Do NOT create a separate test file.

---

### `hp41-core/src/state.rs` (model, CRUD — modification only)

**Analog:** `hp41-core/src/state.rs` lines 356–415 (Phase 43 adv_matrices / adv_tvm_state block) and lines 500–513 (CalcState::new() initialization)

**Field declaration pattern** (state.rs lines 356–388):
```rust
// ── Phase 43 (v3.3): Advantage Pac (XROM 22 + XROM 24) ─────────────────
/// Named matrices for the Advantage Pac ALPHA-register model (D-43.1).
///
/// D-43.5 ISOLATION INVARIANT: This field has NO connection to
/// `state.matrix_dim` or `state.matrix_active_reg` (Math Pac I registers).
/// Persistent — `#[serde(default)]`.
#[serde(default)]
pub adv_matrices: Vec<crate::ops::advantage::AdvMatrix>,

/// Persistent TVM register state (D-43.11).
///
/// ⚠️ UNIQUE SERDE SHAPE — carries `#[serde(default)]` WITHOUT `#[serde(skip)]`
/// so TVM register values survive save/load, ...
#[serde(default)]
pub adv_tvm_state: Option<crate::ops::advantage::TvmState>,

/// Transient: name of the currently active named matrix (D-43.1).
/// None = no matrix active. Transient — `#[serde(default, skip)]`.
#[serde(default, skip)]
pub adv_current_matrix: Option<String>,
```

New fields to add (after the Phase 43 block, in a `// ── Phase 51 (v4.0): X-MEM` comment block):
```rust
// ── Phase 51 (v4.0): X-MEM (Extended Memory) ─────────────────────────────
/// Named extended-memory files (PROGRAM and DATA).
///
/// D-51.0a ISOLATION INVARIANT: This field has NO connection to
/// `state.regs` or `adv_matrices`. X-MEM ops MUST NEVER read/write
/// `state.regs` except via the explicit SAVED/GETD transfer.
/// Persistent — `#[serde(default)]`.
#[serde(default)]
pub xmem_files: Vec<crate::ops::xmem::XmemFile>,

/// Active X-MEM DATA file for EMREG/SAVERX access (D-51.4).
/// Set as side effect of SAVED and GETD. None = no active file.
/// Persistent — `#[serde(default)]` WITHOUT `#[serde(skip)]`
/// (EMREG/SAVERX must survive save/load; analogous to adv_tvm_state).
#[serde(default)]
pub xmem_active_file: Option<String>,
```

**CalcState::new() initialization pattern** (state.rs lines 500–513):
```rust
// Phase 43 (v3.3): Advantage Pac (XROM 22 + XROM 24) fields
adv_matrices: Vec::new(),
adv_matrix_i: 0,
adv_matrix_j: 0,
adv_tvm_state: None,
adv_current_matrix: None,
// ...
```
Add after the Phase 43 block:
```rust
// Phase 51 (v4.0): X-MEM fields
xmem_files: Vec::new(),
xmem_active_file: None,
```

**migrate_after_load pattern** (state.rs lines 547–579): No xmem migration needed in Phase 51 — new fields default cleanly via `#[serde(default)]`. Phase 52 adds the v3.3→v4.0 migration comment only. Do NOT add any `xrom_modules` bit for X-MEM (X-MEM ops are OS built-ins, not XROM modules — confirmed by Research Q5).

---

### `hp41-core/src/ops/mod.rs` (config, request-response — Op enum + dispatch)

**Analog:** `hp41-core/src/ops/mod.rs` lines 1281–1305 (AdvBinin..AdvBitTest Op variant block) and lines 2018–2032 (dispatch arms for same)

**Op enum variant pattern** (lines 1281–1305):
```rust
// ADV CONV (XROM 22) — binary/octal/hex I/O and bitwise operations
/// BININ — enter binary integer from ALPHA register into X.
AdvBinin,
/// BINVIEW — display X as 36-bit binary in ALPHA register.
AdvBinview,
// ...
```
X-MEM equivalent — add a new section after the last Phase 43 variant block:
```rust
// ── Phase 51 (v4.0): X-MEM (Extended Memory) built-in ops ───────────────
/// EMDIR — list all X-MEM files (name/type/size) to print_buffer. LiftEffect: Neutral.
EmDir,
/// EMROOM — push available register count (600 − used) onto X. LiftEffect: Enable.
EmRoom,
/// SAVEP — save current program to named X-MEM PROGRAM file. LiftEffect: Neutral.
SaveP,
/// GETP — retrieve named X-MEM PROGRAM file and insert via RDPRGM semantics. LiftEffect: Neutral.
GetP,
/// SAVED — save data registers R00..R(SIZE-1) to named X-MEM DATA file. LiftEffect: Neutral.
SaveD,
/// GETD — retrieve named X-MEM DATA file and replace state.regs wholesale. LiftEffect: Neutral.
GetD,
/// EMREG — recall register N (from X) of the active X-MEM DATA file → push to stack. LiftEffect: Enable.
EmReg,
/// SAVERX — store Y into register N (from X) of the active X-MEM DATA file. LiftEffect: Neutral.
SaveRx,
```

**dispatch() arm pattern** (lines 2018–2032):
```rust
// ── Phase 43 (v3.3): Advantage Pac (XROM 22 + XROM 24) ─────────────
Op::AdvBinin => crate::ops::advantage::conv::op_adv_binin(state),
Op::AdvAnd => crate::ops::advantage::conv::op_adv_and(state),
// ...
```
X-MEM equivalent dispatch section:
```rust
// ── Phase 51 (v4.0): X-MEM built-in ops ─────────────────────────────
Op::EmDir => crate::ops::xmem::ops::op_emdir(state),
Op::EmRoom => crate::ops::xmem::ops::op_emroom(state),
Op::SaveP => crate::ops::xmem::ops::op_savep(state),
Op::GetP => crate::ops::xmem::ops::op_getp(state),
Op::SaveD => crate::ops::xmem::ops::op_saved(state),
Op::GetD => crate::ops::xmem::ops::op_getd(state),
Op::EmReg => crate::ops::xmem::ops::op_emreg(state),
Op::SaveRx => crate::ops::xmem::ops::op_saverx(state),
```

---

### `hp41-core/src/ops/program.rs` (config, request-response — execute_op)

**Analog:** `hp41-core/src/ops/program.rs` lines 1122–1139 (Advantage Pac execute_op arms)

**execute_op arm pattern** (lines 1122–1139):
```rust
// ── Phase 43 (v3.3): Advantage Pac (XROM 22 + XROM 24) ─────────────
Op::AdvBinin => crate::ops::dispatch(state, Op::AdvBinin),
Op::AdvBinview => crate::ops::dispatch(state, Op::AdvBinview),
Op::AdvAnd => crate::ops::dispatch(state, Op::AdvAnd),
// ...
```
All Phase 43 ops delegate back to `crate::ops::dispatch()`. X-MEM follows the same pattern:
```rust
// ── Phase 51 (v4.0): X-MEM built-in ops ─────────────────────────────
Op::EmDir => crate::ops::dispatch(state, Op::EmDir),
Op::EmRoom => crate::ops::dispatch(state, Op::EmRoom),
Op::SaveP => crate::ops::dispatch(state, Op::SaveP),
Op::GetP => crate::ops::dispatch(state, Op::GetP),
Op::SaveD => crate::ops::dispatch(state, Op::SaveD),
Op::GetD => crate::ops::dispatch(state, Op::GetD),
Op::EmReg => crate::ops::dispatch(state, Op::EmReg),
Op::SaveRx => crate::ops::dispatch(state, Op::SaveRx),
```

---

### `hp41-cli/src/prgm_display.rs` (utility, request-response — op_display_name)

**Analog:** `hp41-cli/src/prgm_display.rs` lines 364–376

**op_display_name arm pattern** (lines 364–376):
```rust
// ── Phase 43 (v3.3): Advantage Pac XROM 22 (ADV CONV + ADV MTRX) ──
Op::AdvBinin => "BININ".to_string(),
Op::AdvBinview => "BINVIEW".to_string(),
Op::AdvNot => "NOT".to_string(),
Op::AdvAnd => "AND".to_string(),
// ...
```
X-MEM equivalent (Phase 51 — stub arms that compile; Phase 52 wires real display):
```rust
// ── Phase 51 (v4.0): X-MEM built-in ops ─────────────────────────────
Op::EmDir => "EMDIR".to_string(),
Op::EmRoom => "EMROOM".to_string(),
Op::SaveP => "SAVEP".to_string(),
Op::GetP => "GETP".to_string(),
Op::SaveD => "SAVED".to_string(),
Op::GetD => "GETD".to_string(),
Op::EmReg => "EMREG".to_string(),
Op::SaveRx => "SAVERX".to_string(),
```
Note: `op_display_name` has no `_ =>` catch-all (CLAUDE.md 4-way exhaustive-match invariant item 3 — adding these arms is required for compilation).

---

### `hp41-gui/src-tauri/src/prgm_display.rs` (utility, request-response — op_display_name)

**Analog:** `hp41-gui/src-tauri/src/prgm_display.rs` lines 382–394

Identical arm block to CLI — the two files are intentionally duplicated (CLAUDE.md: "Items 3 + 4 are duplicated by design"):
```rust
// ── Phase 51 (v4.0): X-MEM built-in ops ─────────────────────────────
Op::EmDir => "EMDIR".to_string(),
Op::EmRoom => "EMROOM".to_string(),
Op::SaveP => "SAVEP".to_string(),
Op::GetP => "GETP".to_string(),
Op::SaveD => "SAVED".to_string(),
Op::GetD => "GETD".to_string(),
Op::EmReg => "EMREG".to_string(),
Op::SaveRx => "SAVERX".to_string(),
```

---

### `hp41-core/src/error.rs` (model, request-response — modification only)

**Analog:** `hp41-core/src/error.rs` (existing HpError variants, lines 1–56)

**thiserror Display pattern** (lines 1–56):
```rust
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum HpError {
    #[error("overflow")]
    Overflow,
    #[error("out of range")]
    OutOfRange,
    /// Card Reader: WDTA/RDTA/WPRGM/RDPRGM with an empty ALPHA register.
    /// Matches the hardware-faithful "ALPHA DATA" message on real HP-41 card readers.
    #[error("alpha data")]
    AlphaData,
    /// Hard iteration-cap exhaustion ...
    #[error("convergence failed")]
    ConvergenceFailed,
```

New variants to add (after `NoRoot`, before the closing `}`):
```rust
/// X-MEM: named file not found (GETP/GETD/EMREG on missing name, or EMREG
/// with no active file set). Matches HP-41CX QRG p.39 "FL NOT FOUND".
#[error("fl not found")]
FileNotFound,
/// X-MEM: file type mismatch (GETP on a DATA file, GETD/EMREG/SAVERX on a
/// PROGRAM file). Matches HP-41CX QRG p.39 "FL TYPE ERR".
#[error("fl type err")]
FileType,
/// X-MEM: insufficient extended memory (SAVEP/SAVED when file would not fit
/// in 600-register capacity). Matches HP-41CX QRG p.39 "NO ROOM".
#[error("no room")]
NoRoom,
```

**Test pattern** (error.rs lines 58–81 — test the new variants):
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::HpError;

    #[test]
    fn file_not_found_display() {
        assert_eq!(HpError::FileNotFound.to_string(), "fl not found");
    }

    #[test]
    fn file_type_display() {
        assert_eq!(HpError::FileType.to_string(), "fl type err");
    }

    #[test]
    fn no_room_display() {
        assert_eq!(HpError::NoRoom.to_string(), "no room");
    }
}
```

---

## Read-Only Reference: Cardreader Public Signatures

These are the concrete signatures `xmem/ops.rs` will call (do NOT re-implement):

**`hp41-core/src/cardreader/raw.rs`** (lines 63–75):
```rust
pub fn encode_program(ops: &[Op]) -> Result<Vec<u8>, HpError>
```

**`hp41-core/src/cardreader/raw.rs`** (decode_program — search for `pub fn decode_program`):
```rust
pub fn decode_program(bytes: &[u8]) -> Result<Vec<Op>, HpError>
// (or the multi-program variant — use the single-program form)
```

**`hp41-core/src/cardreader/mod.rs`** (lines 44–85):
```rust
pub fn insert_program_ops(state: &mut CalcState, ops: Vec<Op>)
pub fn capture_data_card(state: &CalcState) -> DataCard
pub fn load_data_card(state: &mut CalcState, card: DataCard)
const MIN_REGS_AFTER_LOAD: usize = 100;  // pub(crate) — use via load_data_card
```

**`hp41-core/src/cardreader/data.rs`** (lines 25–54):
```rust
pub struct DataCard {
    pub format: String,
    pub version: u32,
    pub registers: Vec<crate::num::HpValue>,
}
pub fn encode_data(card: &DataCard) -> Result<Vec<u8>, HpError>
pub fn decode_data(bytes: &[u8]) -> Result<DataCard, HpError>
pub const FORMAT_TAG: &str = "hp41-data-v1";
pub const FORMAT_VERSION: u32 = 1;
```

---

## Shared Patterns

### LiftEffect on Every Op
**Source:** `hp41-core/src/ops/cardreader_ops.rs` lines 40, 48, 56, 64
**Apply to:** All 8 xmem op functions in `xmem/ops.rs`
```rust
apply_lift_effect(state, LiftEffect::Neutral);   // store/catalog ops
apply_lift_effect(state, LiftEffect::Enable);    // recall ops that push to X
```
Import path: `use crate::stack::{apply_lift_effect, LiftEffect};`

### No println! — Use print_buffer
**Source:** `hp41-core/src/ops/program.rs` lines 307–310, `hp41-core/src/ops/time/alarm.rs` line 330
**Apply to:** `op_emdir` in `xmem/ops.rs`
```rust
state.print_buffer.push(format!("..."));  // CORRECT
// println!("...");  // FORBIDDEN in hp41-core
```

### #[allow(clippy::unwrap_used)] in Tests
**Source:** `hp41-core/src/ops/advantage/mod.rs` line 140, `hp41-core/src/ops/cardreader_ops.rs` line 69
**Apply to:** Every `#[cfg(test)]` block in Phase 51 files
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests { ... }
```

### #[serde(default)] WITHOUT #[serde(skip)] for Persistent New Fields
**Source:** `hp41-core/src/state.rs` lines 362, 381 (`adv_matrices`, `adv_tvm_state`)
**Apply to:** `xmem_files` and `xmem_active_file` on CalcState
- `xmem_files` and `xmem_active_file` are NOT transient — they must survive save/load.
- Add `#[serde(default)]` only. Do NOT add `#[serde(skip)]`.
- Transient fields carry `#[serde(default, skip)]` — do not confuse the two patterns.

### ALPHA-Register Name Extraction
**Source:** `hp41-core/src/ops/cardreader_ops.rs` lines 20–25
**Apply to:** All xmem ops that take a file name (SAVEP, GETP, SAVED, GETD, EMREG, SAVERX)
```rust
fn alpha_name(state: &CalcState) -> Result<String, HpError> {
    if state.alpha_reg.is_empty() {
        return Err(HpError::AlphaData);
    }
    Ok(state.alpha_reg.clone())
}
```

---

## No Analog Found

None — all Phase 51 files have strong analogs in the codebase. The xmem module structure itself (new directory) has no exact file precedent, but `hp41-core/src/ops/advantage/` provides the structural template.

---

## Metadata

**Analog search scope:** `hp41-core/src/`, `hp41-cli/src/`, `hp41-gui/src-tauri/src/`
**Files scanned:** 12 files read directly; ~8 additional via grep line-targeting
**Pattern extraction date:** 2026-05-28
