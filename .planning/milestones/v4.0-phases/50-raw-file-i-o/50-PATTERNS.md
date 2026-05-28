# Phase 50: .raw File I/O — Pattern Map

**Mapped:** 2026-05-27
**Files analyzed:** 12 new/modified files
**Analogs found:** 12 / 12

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-core/src/cardreader/raw.rs` | utility (codec extension) | transform | self (existing `decode_program`) | exact |
| `hp41-gui/src-tauri/src/commands.rs` | controller | request-response | self (existing `dispatch_op`, `save_state`) | exact |
| `hp41-gui/src-tauri/src/lib.rs` | config | - | self (existing `generate_handler!` block) | exact |
| `hp41-gui/src-tauri/permissions/import-raw-dialog.toml` | config | - | `permissions/save-state.toml` | exact |
| `hp41-gui/src-tauri/permissions/export-raw-dialog.toml` | config | - | `permissions/save-state.toml` | exact |
| `hp41-gui/src-tauri/permissions/import-data-dialog.toml` | config | - | `permissions/save-state.toml` | exact |
| `hp41-gui/src-tauri/permissions/export-data-dialog.toml` | config | - | `permissions/save-state.toml` | exact |
| `hp41-gui/package.json` | config | - | existing `package.json` (dep addition) | exact |
| `hp41-gui/src/App.tsx` | component | request-response | self (existing `invokeForKey`, `showToast`, `dispatchKeyId`) | exact |
| `hp41-gui/src/RawPickerOverlay.tsx` | component | request-response | `hp41-gui/src/SettingsPanel.tsx` (modal overlay) | role-match |
| `hp41-cli/src/main.rs` | config (CLI flags) | file-I/O | self (existing `Cli` struct, `state_file` flag) | exact |
| `hp41-core/tests/cardreader_raw_multi.rs` | test | transform | `hp41-core/tests/cardreader_tests.rs` | exact |

---

## Pattern Assignments

### `hp41-core/src/cardreader/raw.rs` — codec extension

**Analog:** self (lines 183–386 of `hp41-core/src/cardreader/raw.rs`)

**Existing public API** (lines 64–75 and 183–195):
```rust
/// Encode a sequence of `Op`s to the bare `.raw` byte stream.
pub fn encode_program(ops: &[Op]) -> Result<Vec<u8>, HpError> { ... }

/// Decode a bare `.raw` byte stream back into a sequence of `Op`s.
///
/// Requires an END marker (`C0 00 0D`) — input that runs out without one is
/// considered truncated and returns `HpError::CardData`.
pub fn decode_program(bytes: &[u8]) -> Result<Vec<Op>, HpError> {
    let mut ops = Vec::new();
    let mut i = 0;
    let mut saw_end = false;
    while i < bytes.len() {
        // Stop at END marker. Any bytes after the marker are an error —
        // multi-program concatenation is unsupported ...
        if bytes[i..].starts_with(&END_MARKER) {
            saw_end = true;
            if i + END_MARKER.len() < bytes.len() {
                return Err(HpError::CardData(format!(
                    "trailing bytes after END marker: {} extra byte(s)",
                    ...
```

**END_MARKER constant** (line 38):
```rust
const END_MARKER: [u8; 3] = [0xC0, 0x00, 0x0D];
```

**New `decode_all_programs()` pattern** — add after `decode_program`:
```rust
/// Decoded program from a multi-program byte archive.
pub struct DecodedProgram {
    pub ops: Vec<Op>,
    /// Byte count of this program's segment (including END marker).
    pub byte_len: usize,
}

/// Decode a multi-program bare `.raw` byte stream.
///
/// A single-program file is valid input (returns Vec of length 1).
/// Empty input returns an empty Vec (not an error).
/// Splits at each END marker (`C0 00 0D`); each segment is fed to
/// `decode_program`, which handles full decode logic minus trailing-bytes check.
/// Caps at 256 programs to prevent memory-exhaustion from crafted inputs.
pub fn decode_all_programs(bytes: &[u8]) -> Result<Vec<DecodedProgram>, HpError> {
    let mut result = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if result.len() >= 256 {
            return Err(HpError::CardData(
                "archive contains more than 256 programs; refusing to decode".into(),
            ));
        }
        let end_pos = bytes[offset..]
            .windows(END_MARKER.len())
            .position(|w| w == END_MARKER)
            .map(|p| offset + p)
            .ok_or_else(|| HpError::CardData(
                "truncated input: stream ended without END marker (C0 00 0D)".into(),
            ))?;
        let seg_end = end_pos + END_MARKER.len();
        let segment = &bytes[offset..seg_end];
        // decode_program expects exactly one END marker with no trailing bytes —
        // segment is sliced to [ops... END_MARKER], so the call is correct.
        let ops = decode_program(segment)?;
        result.push(DecodedProgram { ops, byte_len: segment.len() });
        offset = seg_end;
    }
    Ok(result)
}
```

**LBL name extractor for picker label (D-50.5)** — add as free function in `raw.rs` or in `commands.rs`:
```rust
// Op::Lbl variant confirmed at raw.rs line 118: Op::Lbl(name)
pub fn picker_label(program_index: usize, ops: &[Op], byte_len: usize) -> String {
    let name = ops.iter().find_map(|op| {
        if let Op::Lbl(name) = op { Some(name.as_str()) } else { None }
    });
    match name {
        Some(n) => format!("{} ({} bytes)", n, byte_len),
        None => format!("Program {} ({} bytes)", program_index + 1, byte_len),
    }
}
```

**Error handling pattern** (line 194–197):
```rust
return Err(HpError::CardData(format!(
    "trailing bytes after END marker: {} extra byte(s)",
    bytes.len() - (i + END_MARKER.len())
)));
```

**Test pattern** (lines 417–419 and 428–430):
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    fn append_end(mut bytes: Vec<u8>) -> Vec<u8> {
        bytes.extend_from_slice(&END_MARKER);
        bytes
    }
    #[test]
    fn encode_appends_end_marker() {
        let bytes = encode_program(&[Op::Add]).unwrap();
        assert_eq!(bytes, vec![0x40, 0xC0, 0x00, 0x0D]);
    }
```

---

### `hp41-gui/src-tauri/src/commands.rs` — new Tauri commands

**Analog:** self (existing commands, especially `save_state` lines 474–479 and `dispatch_op` lines 52–72)

**Imports pattern** (lines 18–27):
```rust
use crate::cards;
use crate::key_map;
use crate::persistence;
use crate::types::{CalcStateView, GuiError};
use crate::{AppState, CancelFlag, PrefsState};
use hp41_core::ops::dispatch;
use hp41_core::CalcState;
use tauri::State;
```

**New imports to add for dialog commands:**
```rust
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use hp41_core::cardreader::{decode_all_programs, decode_program, encode_program,
                             capture_data_card, encode_data, decode_data,
                             insert_program_ops, load_data_card};
```

**CR-01 clone-under-lock pattern** (`save_state`, lines 474–479):
```rust
#[tauri::command]
pub fn save_state(state: State<'_, AppState>) -> Result<(), String> {
    // CR-01: clone under lock, then release lock before disk I/O.
    let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let path = persistence::default_state_path();
    persistence::save_state(&path, &snapshot).map_err(|e| e.to_string())
}
```

**Mutex-BEFORE-dialog pattern** (anti-deadlock — from RESEARCH Pitfall 1):
```rust
// Dialog FIRST (no lock held), file read SECOND (no lock held),
// lock ONLY for state mutation.
#[tauri::command]
pub fn import_raw_dialog(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CalcStateView, GuiError> {
    // Phase 1: open dialog — NO lock held (Pitfall 1 anti-deadlock).
    let file_path_opt = app.dialog()
        .file()
        .set_title("Import HP-41 Program")
        .add_filter("HP-41 Program", &["raw"])
        .add_filter("All Files", &["*"])
        .blocking_pick_file();

    let Some(file_path) = file_path_opt else {
        // User cancelled — return current state unchanged (same pattern as no-op key).
        let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
        return handle_get_state(&mut calc);
    };

    // Phase 2: file I/O — NO lock held.
    let path_buf = file_path.into_path()
        .map_err(|e| GuiError::from_str(&format!("path error: {e}")))?;
    let bytes = std::fs::read(&path_buf)
        .map_err(|e| GuiError::from_str(&format!("io: read {}: {e}", path_buf.display())))?;

    // Decode: try multi-program first; single-program is a subset (len==1).
    let programs = hp41_core::cardreader::raw::decode_all_programs(&bytes)
        .map_err(GuiError::from)?;

    // Phase 3: state mutation — lock acquired AFTER all I/O.
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());

    if programs.len() == 1 {
        insert_program_ops(&mut calc, programs.into_iter().next().unwrap().ops);
        handle_get_state(&mut calc)
    } else {
        // Multi-program: return list to frontend for picker selection.
        // (Store decoded programs in a temporary command-scoped location
        //  or return structured metadata — see RawPickerOverlay pattern below.)
        // ...
    }
}
```

**Error handling pattern** — `GuiError::from_str` (existing pattern, `types.rs`):
```rust
GuiError::from_str(&format!("io: read {}: {e}", path_buf.display()))
```

**State lock + finalize pattern** (lines 82–85, `get_state`):
```rust
#[tauri::command]
pub fn get_state(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_get_state(&mut calc)
}
```

**Export pattern** (mirrors import; uses `blocking_save_file`):
```rust
#[tauri::command]
pub fn export_raw_dialog(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), GuiError> {
    // Phase 1: snapshot under lock, then drop lock before dialog.
    let bytes = {
        let calc = state.lock().unwrap_or_else(|e| e.into_inner());
        encode_program(&calc.program).map_err(GuiError::from)?
    };
    // Phase 2: dialog + write — NO lock held.
    let save_path_opt = app.dialog()
        .file()
        .set_title("Export HP-41 Program")
        .set_file_name("program.raw")
        .add_filter("HP-41 Program", &["raw"])
        .add_filter("All Files", &["*"])
        .blocking_save_file();

    let Some(save_path) = save_path_opt else { return Ok(()); };
    let path_buf = save_path.into_path()
        .map_err(|e| GuiError::from_str(&format!("path error: {e}")))?;
    std::fs::write(&path_buf, &bytes)
        .map_err(|e| GuiError::from_str(&format!("io: write {}: {e}", path_buf.display())))?;
    Ok(())
}
```

---

### `hp41-gui/src-tauri/src/lib.rs` — register new commands

**Analog:** self (lines 100–114, existing `invoke_handler!`)

**Core pattern — add new commands to `generate_handler!`** (lines 100–114):
```rust
.invoke_handler(tauri::generate_handler![
    commands::dispatch_op,
    commands::get_state,
    // ... existing commands ...
    commands::save_state,              // Phase 49
    commands::import_raw_dialog,       // Phase 50 RAW-01/RAW-03
    commands::export_raw_dialog,       // Phase 50 RAW-02
    commands::import_data_dialog,      // Phase 50 D-50.7
    commands::export_data_dialog,      // Phase 50 D-50.7
    commands::get_multi_program_list,  // Phase 50 D-50.4 (if multi-picker uses separate command)
])
```

**Plugin registration — add to `tauri::Builder::default()` chain** (after line 36):
```rust
tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())  // Phase 50 — must be before setup()
    .setup(|app| {
        // ... existing setup ...
    })
```

---

### `hp41-gui/src-tauri/permissions/import-raw-dialog.toml` (and export/data variants)

**Analog:** `hp41-gui/src-tauri/permissions/save-state.toml` (all 7 lines)

**Exact template to copy and adapt:**
```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-import-raw-dialog"
description = "Allows the import_raw_dialog command."
commands.allow = ["import_raw_dialog"]
```

Repeat for:
- `export-raw-dialog.toml` — identifier `"allow-export-raw-dialog"`, command `"export_raw_dialog"`
- `import-data-dialog.toml` — identifier `"allow-import-data-dialog"`, command `"import_data_dialog"`
- `export-data-dialog.toml` — identifier `"allow-export-data-dialog"`, command `"export_data_dialog"`
- If `get_multi_program_list` is a separate command: `get-multi-program-list.toml`

---

### `hp41-gui/src-tauri/capabilities/default.json` — register permissions

**Analog:** self (lines 1–21)

**Add to `"permissions"` array** (after `"allow-save-state"`, lines 5–19):
```json
"allow-save-state",
"allow-import-raw-dialog",
"allow-export-raw-dialog",
"allow-import-data-dialog",
"allow-export-data-dialog",
"dialog:allow-open",
"dialog:allow-save"
```

`"dialog:allow-open"` and `"dialog:allow-save"` are built-in Tauri plugin identifiers (not backed by TOML files — they come from the plugin's own capability manifest). The custom `allow-*` entries correspond to the new TOML files.

---

### `hp41-gui/src/App.tsx` — frontend integration

**Analog:** self (existing `invokeForKey` lines 81–108, `dispatchKeyId` lines 380–387, `showToast` lines 274–277)

**Toast pattern** (lines 274–277):
```typescript
const showToast = useCallback((msg: string) => {
  toastSeqRef.current += 1;
  setToast({ msg, seq: toastSeqRef.current });
}, []);
```

**invoke + setCalcState pattern** (lines 380–387):
```typescript
const dispatchKeyId = useCallback((keyId: string) => {
  if (busyRef.current) return;
  busyRef.current = true;
  invokeForKey(keyId, calcState)
    .then(view => { setCalcState(view); setErrorMessage(null); })
    .catch(err => showToast(extractErrMessage(err)))
    .finally(() => { busyRef.current = false; });
}, [calcState, showToast]);
```

**New import/export dialog invocation** — copy this pattern:
```typescript
async function importRawDialog() {
  if (busyRef.current) return;
  busyRef.current = true;
  try {
    const view = await invoke<CalcStateView>('import_raw_dialog');
    setCalcState(view);
    setErrorMessage(null);
    // D-50.3: toast uses metadata from view (if CalcStateView carries it)
    // or a generic confirmation message.
    showToast('Program imported');
  } catch (err) {
    showToast(extractErrMessage(err));
  } finally {
    busyRef.current = false;
  }
}
```

**extractErrMessage pattern** (lines 61–71) — already in file, reuse unchanged:
```typescript
function extractErrMessage(err: unknown): string {
  if (typeof err === 'object' && err !== null) {
    if ('message' in err) return String((err as { message: unknown }).message);
    try { return JSON.stringify(err); } catch { }
  }
  return String(err);
}
```

**Physical keyboard wiring** (lines 115–120 — Ctrl+key bindings pattern):
```typescript
if (e.ctrlKey || e.metaKey) {
  switch (e.key.toLowerCase()) {
    case 'w': return 'xeq_WPRGM';   // existing card writer key
    // Phase 50: empty-ALPHA path is handled in backend; same key IDs trigger
    // the dialog branch when ALPHA is empty (D-50.1).
```

---

### `hp41-gui/src/RawPickerOverlay.tsx` — new modal component

**Analog:** `hp41-gui/src/SettingsPanel.tsx` (closest role-match for a modal overlay with action buttons)

Let me check the SettingsPanel for the overlay pattern:
```
Analog: SettingsPanel.tsx — modal overlay component pattern
```
The key structural pattern to copy is:
- Props interface with `onClose` / `onConfirm` callbacks
- Overlay `<div>` with CSS class for backdrop
- List of selectable items with checkboxes (for multi-select per D-50.6)
- Confirm / Cancel buttons

Since the existing modal system uses `ModalProgram` for calculator-modal prompts, `RawPickerOverlay` is a pure React UI component (no Tauri command needed for display state — only for final confirm). Pattern:

```typescript
interface ProgramEntry {
  label: string;   // e.g. "QUAD (47 bytes)"
  index: number;   // index in the decoded programs array
}

interface RawPickerOverlayProps {
  programs: ProgramEntry[];
  onConfirm: (selectedIndices: number[]) => void;
  onClose: () => void;
}

export function RawPickerOverlay({ programs, onConfirm, onClose }: RawPickerOverlayProps) {
  const [selected, setSelected] = useState<Set<number>>(new Set());
  // ... checkbox list + Confirm/Cancel buttons
}
```

State: `programs: ProgramEntry[] | null` in App.tsx state (null = picker closed). When `import_raw_dialog` returns a multi-program response, App.tsx populates this list and renders `<RawPickerOverlay>`. On confirm, App.tsx calls a second command (`import_programs_by_index`) with selected indices.

---

### `hp41-cli/src/main.rs` — new CLI flags

**Analog:** self (existing `Cli` struct, lines 27–41)

**Existing flag pattern** (lines 30–32):
```rust
/// Path to the state file (JSON). Loaded on startup, saved on exit and every 30s.
/// Default: ~/.hp41/autosave.json
#[arg(long, value_name = "FILE")]
state_file: Option<std::path::PathBuf>,
```

**New flags to add** — use `Option<String>` not `Option<PathBuf>` to support `-` for stdin/stdout (RESEARCH Pitfall 7):
```rust
/// Import a .raw program file into calculator memory, then launch the TUI.
/// Use '-' to read from stdin.
#[arg(long, value_name = "FILE")]
import_raw: Option<String>,

/// Export the current program to a .raw file (does not launch TUI by default
/// when combined with --batch). Use '-' to write to stdout.
#[arg(long, value_name = "FILE")]
export_raw: Option<String>,

/// Import a .card.json data file into calculator data registers.
/// Use '-' to read from stdin.
#[arg(long, value_name = "FILE")]
import_data: Option<String>,

/// Export current data registers to a .card.json file.
/// Use '-' to write to stdout.
#[arg(long, value_name = "FILE")]
export_data: Option<String>,

/// Suppress interactive TUI — exit after import/export completes (exit 0).
/// Enables scripting pipelines. Example:
///   curl https://example.com/prog.raw | hp41 --import-raw - --batch
#[arg(long)]
batch: bool,
```

**Stdin/stdout piping pattern** (from RESEARCH Pitfall 7):
```rust
// In main(), after Cli::parse():
fn read_file_or_stdin(path: &str) -> std::io::Result<Vec<u8>> {
    if path == "-" {
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut std::io::stdin(), &mut buf)?;
        Ok(buf)
    } else {
        std::fs::read(path)
    }
}

fn write_file_or_stdout(path: &str, bytes: &[u8]) -> std::io::Result<()> {
    if path == "-" {
        std::io::Write::write_all(&mut std::io::stdout(), bytes)
    } else {
        std::fs::write(path, bytes)
    }
}
```

**Startup load/error pattern** (lines 52–63 — mirror for import):
```rust
if let Some(ref raw_path) = cli.import_raw {
    let bytes = read_file_or_stdin(raw_path)
        .map_err(|e| std::io::Error::new(e.kind(), format!("--import-raw: {e}")))?;
    let programs = hp41_core::cardreader::raw::decode_all_programs(&bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData,
                                          format!("decode: {e}")))?;
    // For single program: insert directly. For multi: pick first or surface error.
    let ops = programs.into_iter().next()
        .map(|p| p.ops)
        .unwrap_or_default();
    hp41_core::cardreader::insert_program_ops(&mut initial_state, ops);
}
if cli.batch { return Ok(()); }
```

---

### `hp41-core/tests/cardreader_raw_multi.rs` — new test file

**Analog:** `hp41-core/tests/cardreader_tests.rs` (lines 1–56)

**File header pattern** (lines 1–18 of `cardreader_tests.rs`):
```rust
//! Integration tests for decode_all_programs().
//!
//! Covers:
//! - Single-program file treated as multi-program (Vec of length 1)
//! - Two-program concatenation split at END marker
//! - Truncated file (no END marker) → HpError::CardData
//! - False-split guard: alpha payload containing `C0 00 0D` bytes must not
//!   split at the embedded bytes
//! - Archive capped at 256 programs

use hp41_core::cardreader::raw::{decode_all_programs, encode_program, END_MARKER};
use hp41_core::error::HpError;
use hp41_core::ops::Op;
```

**Test structure pattern** (lines 20–56 of `cardreader_tests.rs`):
```rust
#[test]
fn decode_all_single_program_returns_vec_of_one() {
    let bytes = encode_program(&[Op::Add, Op::Sub]).unwrap();
    let programs = decode_all_programs(&bytes).unwrap();
    assert_eq!(programs.len(), 1);
    assert_eq!(programs[0].ops, vec![Op::Add, Op::Sub]);
    assert_eq!(programs[0].byte_len, bytes.len());
}

#[test]
fn decode_all_two_programs() {
    let prog1 = encode_program(&[Op::Add]).unwrap();
    let prog2 = encode_program(&[Op::Sub, Op::Mul]).unwrap();
    let combined = [prog1.as_slice(), prog2.as_slice()].concat();
    let programs = decode_all_programs(&combined).unwrap();
    assert_eq!(programs.len(), 2);
    assert_eq!(programs[0].ops, vec![Op::Add]);
    assert_eq!(programs[1].ops, vec![Op::Sub, Op::Mul]);
}

#[test]
fn decode_all_empty_input_returns_empty_vec() {
    let programs = decode_all_programs(&[]).unwrap();
    assert!(programs.is_empty());
}

#[test]
fn decode_all_truncated_no_end_marker_returns_error() {
    let err = decode_all_programs(&[0x40, 0x41]).unwrap_err();
    assert!(matches!(err, HpError::CardData(msg) if msg.contains("END")));
}
```

---

## Shared Patterns

### Poisoned-lock recovery
**Source:** `hp41-gui/src-tauri/src/commands.rs` line 55
**Apply to:** All new `#[tauri::command]` functions in `commands.rs`
```rust
let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
```

### Dialog-before-lock (CR-01 extended)
**Source:** RESEARCH.md Pitfall 1 + `save_state` pattern (commands.rs line 476)
**Apply to:** All import/export dialog commands
- For imports: open dialog and do file I/O BEFORE acquiring AppState lock
- For exports: snapshot bytes under lock, RELEASE lock, then open dialog + write

### Error wrapping to `GuiError`
**Source:** `hp41-gui/src-tauri/src/commands.rs` line 64
**Apply to:** All new commands
```rust
cards::execute_prepared_card_op(p, &dir).map_err(GuiError::from)?
```
Note: `GuiError::from_str(&format!(...))` for constructing ad-hoc error messages.

### Permission TOML + `default.json` registration
**Source:** `permissions/save-state.toml` + `capabilities/default.json` lines 19
**Apply to:** Every new `#[tauri::command]` function registered in `lib.rs`
The CI gate `scripts/check-tauri-permissions.sh` enforces one TOML per command.

### `HpError::CardData(format!(...))` error construction
**Source:** `hp41-core/src/cardreader/raw.rs` lines 141–144
**Apply to:** `decode_all_programs()` and any new codec-layer functions
```rust
return Err(HpError::CardData(format!(
    "op cannot be encoded in the .raw subset: {other:?}"
)));
```

### `#[cfg(test)] #[allow(clippy::unwrap_used)]` test module
**Source:** `hp41-core/src/cardreader/raw.rs` lines 417–419
**Apply to:** All new test files and inline test modules
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
```

---

## No Analog Found

All files have close matches. No entries in this section.

---

## Metadata

**Analog search scope:** `hp41-core/src/cardreader/`, `hp41-gui/src-tauri/src/`, `hp41-gui/src/`, `hp41-cli/src/`, `hp41-core/tests/`
**Files read:** 12
**Pattern extraction date:** 2026-05-27
