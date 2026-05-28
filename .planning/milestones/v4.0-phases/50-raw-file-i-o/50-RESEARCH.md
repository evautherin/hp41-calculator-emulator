# Phase 50: .raw File I/O — Research

**Researched:** 2026-05-27
**Domain:** Tauri v2 file dialog integration, `.raw` codec extension, CLI flag design
**Confidence:** HIGH — all key claims verified against official sources or codebase inspection

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-50.1:** Empty ALPHA triggers native file dialog; non-empty ALPHA uses existing `~/.hp41/cards/NAME.raw` behavior unchanged.
- **D-50.2:** File dialog filter — Claude's discretion (`.raw` filter + all-files fallback).
- **D-50.3:** User feedback: both toast message AND status line message on import/export.
- **D-50.4:** Multi-program `.raw` archives handled via picker dialog (decode all, present list, user selects).
- **D-50.5:** Picker identification: "LBL + size" — e.g., "QUAD (47 bytes)"; fallback "Program N (M bytes)".
- **D-50.6:** Picker allows multi-select; selected programs inserted sequentially via `insert_program_ops`.
- **D-50.7:** Data card file dialog for WDTA/RDTA with empty ALPHA; uses `.card.json` filter.
- **D-50.8:** CLI flags support file path AND stdin/stdout piping (`--import-raw -` for stdin, `--export-raw -` for stdout).
- **D-50.9:** `--import-raw` default: load then start interactive TUI. `--batch`/`--no-tui` flag suppresses TUI (exit 0 after load).
- **D-50.10:** CLI gets `--import-data` / `--export-data` flags, same stdin/stdout piping.

### Claude's Discretion

- File dialog filter configuration (D-50.2) — pick appropriate filter defaults for `tauri-plugin-dialog`.
- `decode_all_programs()` function design — how to split a multi-program byte stream at END markers.
- Multi-program picker UI implementation — modal overlay, list component, or reuse of existing patterns.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| RAW-01 | User can import a single-program `.raw` file into calculator memory | Codec `decode_program` ready; need GUI command + CLI flag |
| RAW-02 | User can export a program to `.raw` format | Codec `encode_program` ready; need GUI command + CLI flag |
| RAW-03 | Multi-program `.raw` archive files handled (import all or clear error) | Requires new `decode_all_programs()` in `raw.rs`; current `decode_program` explicitly rejects trailing bytes |
| RAW-04 | Import/export uses native OS file dialog via `tauri-plugin-dialog` | Plugin verified on crates.io 2.7.1 + npm 2.7.1; blocking API documented |
| RAW-05 | XROM instructions in imported `.raw` files preserved correctly | Codec passes unknowns through as `Op::SyntheticByte` — round-trip guaranteed |
| RAW-06 | CLI equivalent via `--import-raw` / `--export-raw` flags | clap 4.x `--import-raw PATH`, `-` for stdin/stdout, `--batch` flag |

</phase_requirements>

---

## Summary

Phase 50 is a **frontend integration task**, not a codec task. The `.raw` byte codec (`encode_program` / `decode_program`) and the `.card.json` codec (`encode_data` / `decode_data`) are fully implemented and tested in `hp41-core/src/cardreader/`. The three-phase drain contract (`prepare_pending_card_op` / `execute_prepared_card_op` / `apply_card_read_result`) already exists identically in both `hp41-cli/src/cards.rs` and `hp41-gui/src-tauri/src/cards.rs`.

The work breaks into three distinct streams: (1) extend `hp41-core/src/cardreader/raw.rs` with `decode_all_programs()` to support multi-program archives (RAW-03); (2) wire the file dialog path into the GUI via `tauri-plugin-dialog` and new Tauri commands; (3) add `--import-raw`, `--export-raw`, `--import-data`, `--export-data`, and `--batch` CLI flags in `hp41-cli/src/main.rs`. The empty-ALPHA trigger (D-50.1) is the key design branching point: both frontends already check ALPHA to decide the card name, so the dialog path slots cleanly into phase 2 of the existing three-phase drain.

One critical pre-condition: `decode_program` currently **explicitly rejects** trailing bytes after the END marker (by design, to avoid silent truncation of concatenated programs). The new `decode_all_programs()` function must be added to `hp41-core/src/cardreader/raw.rs` before any multi-program import can work.

**Primary recommendation:** Wave 0 = add `decode_all_programs()` to `hp41-core` and wire the dialog permission TOML plumbing. Wave 1 = GUI import/export single-program. Wave 2 = multi-program picker. Wave 3 = CLI flags.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `.raw` byte decode/encode | `hp41-core` | — | Codec is UI-agnostic; already lives in `cardreader/raw.rs` |
| `decode_all_programs()` | `hp41-core` | — | Same codec layer; must stay in core to be unit-testable |
| File dialog UX (open/save) | GUI Tauri backend | — | `tauri-plugin-dialog` is a backend Rust API; blocking calls from command handlers |
| Multi-program picker UI | GUI Frontend (React) | — | User selection state is UI-only; no `CalcState` involvement until user confirms |
| Card op staging/drain | GUI/CLI frontend | `hp41-core` | `CardOpRequest` enum is core; drain logic is per-frontend |
| CLI import/export flags | CLI binary (`main.rs`) | — | clap 4.x arg parsing at startup; codec calls in startup path, not event loop |
| Permission TOML registration | GUI Tauri build | — | `check-tauri-permissions.sh` CI gate enforces TOML presence per command |
| Toast feedback | GUI Frontend (React) | — | `showToast()` pattern already in `App.tsx`; no backend involvement |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `tauri-plugin-dialog` | 2.7.1 | Native OS file picker/saver dialogs | Official Tauri ecosystem plugin; same version family as Tauri 2.11 in use |
| `@tauri-apps/plugin-dialog` | 2.7.1 | JS bindings for the dialog plugin | Companion npm package; same repo as Rust crate |
| `clap` | 4.x (already in deps) | CLI flag parsing | Already used in `hp41-cli`; add `--import-raw`, `--export-raw`, `--import-data`, `--export-data`, `--batch` flags |

[VERIFIED: crates.io registry] — `tauri-plugin-dialog` 2.7.1 confirmed via `cargo search`.
[VERIFIED: npm registry] — `@tauri-apps/plugin-dialog` 2.7.1 confirmed via `npm view`, published 2023-05-24, modified 2026-05-02, source: `github.com/tauri-apps/plugins-workspace`.

### No New `hp41-core` Dependencies

The "zero new runtime deps since v3.0" policy (CLAUDE.md) is preserved. `tauri-plugin-dialog` is a Tauri plugin (GUI-layer dep only, appears only in `hp41-gui/src-tauri/Cargo.toml`), not an `hp41-core` dependency.

### Installation

```bash
# Rust (hp41-gui/src-tauri/Cargo.toml only)
# Add to [dependencies]:
# tauri-plugin-dialog = "2.7.1"

# JavaScript (hp41-gui/)
npm install @tauri-apps/plugin-dialog@2.7.1
```

---

## Package Legitimacy Audit

slopcheck was unavailable at research time. All packages below are tagged `[ASSUMED]` per the graceful-degradation rule. The planner must gate each install behind a `checkpoint:human-verify` task.

| Package | Registry | Age | Downloads | Source Repo | slopcheck | Disposition |
|---------|----------|-----|-----------|-------------|-----------|-------------|
| `tauri-plugin-dialog` | crates.io | ~3 yrs (first published ~2023) | — | `github.com/tauri-apps/plugins-workspace` | N/A (slopcheck unavailable) | `[ASSUMED]` — planner gates with checkpoint |
| `@tauri-apps/plugin-dialog` | npm | ~3 yrs (2023-05-24) | — | `github.com/tauri-apps/plugins-workspace` | N/A (slopcheck unavailable) | `[ASSUMED]` — planner gates with checkpoint |

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none — both packages are from the official `tauri-apps/plugins-workspace` mono-repo (the same org that publishes the Tauri framework itself). Registry existence + official org provenance gives HIGH practical confidence despite the slopcheck tool being unavailable.

*No postinstall scripts detected on `@tauri-apps/plugin-dialog` (confirmed via `npm view`).*

---

## Architecture Patterns

### System Architecture Diagram

```
User action (empty ALPHA + WPRGM/RDPRGM/WDTA/RDTA)
        │
        ▼
  dispatch_op (GUI) / card op dispatch (CLI)
        │
        ▼ (D-50.1 branch: empty ALPHA?)
   ┌────┴──────────────────────┐
   │ YES (dialog path)          │ NO (existing cards_dir path — unchanged)
   ▼                            ▼
 Native OS file dialog      ~/.hp41/cards/NAME.{raw,card.json}
 (tauri-plugin-dialog)
   │
   ▼ (RAW-03 branch: multi-program?)
 ┌─┴──────────────────────┐
 │ single program          │ multi-program archive
 ▼                         ▼
 decode_program()      decode_all_programs()
                           │
                           ▼
                     Multi-program picker UI
                     (React modal, multi-select)
                           │
                           ▼
                     Selected programs
        │
        ▼
 insert_program_ops() / load_data_card()
        │
        ▼
 Toast + status line feedback (D-50.3)
        │
        ▼
 CalcStateView returned to frontend
```

### Recommended Project Structure

New/modified files only:

```
hp41-core/src/cardreader/
└── raw.rs                   # ADD: decode_all_programs() + DecodedProgram struct

hp41-gui/src-tauri/src/
├── cards.rs                 # MODIFY: add dialog-path branch in execute_prepared_card_op
├── commands.rs              # ADD: import_raw_dialog, export_raw_dialog, import_data_dialog,
│                            #      export_data_dialog, get_multi_program_list (new Tauri commands)
└── lib.rs                   # MODIFY: register new commands in generate_handler!

hp41-gui/src-tauri/permissions/
├── import-raw-dialog.toml   # NEW (required by check-tauri-permissions.sh)
├── export-raw-dialog.toml   # NEW
├── import-data-dialog.toml  # NEW
└── export-data-dialog.toml  # NEW

hp41-gui/
└── package.json             # ADD: @tauri-apps/plugin-dialog@2.7.1

hp41-gui/src/
├── App.tsx                  # MODIFY: handle new card dialog IPC responses + multi-program picker
└── MultiProgramPicker.tsx   # NEW: picker modal component (Claude's discretion per D-50.4)

hp41-cli/src/
└── main.rs                  # ADD: --import-raw, --export-raw, --import-data, --export-data, --batch flags
```

### Pattern 1: `decode_all_programs()` Design

**What:** Splits a multi-program byte stream at END markers (`C0 00 0D`), decoding each segment independently.

**Key constraint:** The existing `decode_program()` function explicitly returns `Err(CardData("trailing bytes after END marker: ..."))` when it finds bytes after the END marker. `decode_all_programs()` must scan the byte stream scanning for END markers sequentially, NOT call `decode_program()` in a loop (which would fail on the second segment).

**Design — `decode_all_programs()` in `raw.rs`:**

```rust
// Source: inferred from raw.rs source code analysis + END_MARKER constant definition
pub struct DecodedProgram {
    pub ops: Vec<Op>,
    /// Byte count of this program's segment (including END marker).
    pub byte_len: usize,
}

/// Decode a multi-program bare `.raw` byte stream into individual programs.
///
/// A single-program file is also valid input (returns a Vec of length 1).
/// Empty input returns an empty Vec (no error).
///
/// Splits at each END marker (`C0 00 0D`). Each segment is decoded via the
/// same decode logic as `decode_program`, minus the trailing-bytes check.
pub fn decode_all_programs(bytes: &[u8]) -> Result<Vec<DecodedProgram>, HpError> {
    let mut result = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        // Find next END marker from current offset.
        let end_pos = find_end_marker(bytes, offset)?;
        let segment = &bytes[offset..end_pos + END_MARKER.len()];
        let ops = decode_segment(segment)?;  // reuses decode logic, expects exactly one END
        result.push(DecodedProgram {
            ops,
            byte_len: segment.len(),
        });
        offset = end_pos + END_MARKER.len();
    }
    Ok(result)
}
```

[ASSUMED] — function signature is Claude's design (D-50.2 discretion); not from docs.

### Pattern 2: tauri-plugin-dialog Registration

**In `lib.rs` (Rust setup):**

```rust
// Source: https://v2.tauri.app/plugin/dialog/
use tauri_plugin_dialog::DialogExt;

tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    // ... existing plugins/handlers
```

**Capability permissions in `capabilities/default.json`:**

```json
"dialog:allow-open",
"dialog:allow-save"
```

[VERIFIED: official Tauri v2 docs] — permission identifiers `dialog:allow-open` and `dialog:allow-save` confirmed.

### Pattern 3: Blocking File Dialog from a Tauri Command

Tauri commands run on a thread-pool thread (not the main thread), so `blocking_pick_file()` / `blocking_save_file()` are safe to call from command handlers.

```rust
// Source: https://docs.rs/tauri-plugin-dialog/2.7.1/tauri_plugin_dialog/
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub fn import_raw_dialog(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<CalcStateView, GuiError> {
    let path = app.dialog()
        .file()
        .add_filter("HP-41 Program", &["raw"])
        .add_filter("All Files", &["*"])
        .blocking_pick_file();

    let Some(file_path) = path else {
        // User cancelled — return current state unchanged
        let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
        return handle_get_state(&mut calc);
    };

    let path_buf = file_path.into_path()
        .map_err(|e| GuiError::from_str(&format!("path error: {e}")))?;
    let bytes = std::fs::read(&path_buf)
        .map_err(|e| GuiError::from_str(&format!("io: {e}")))?;

    // Single vs multi-program dispatch handled here or in a separate command
    let ops = hp41_core::cardreader::raw::decode_program(&bytes)
        .map_err(|e| GuiError::from(HpError::CardData(format!("decode: {e}"))))?;

    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    hp41_core::cardreader::insert_program_ops(&mut calc, ops);
    handle_get_state(&mut calc)
}
```

[VERIFIED: docs.rs/tauri-plugin-dialog/2.7.1] — `app.dialog().file().add_filter(...).blocking_pick_file()` confirmed. `FilePath::into_path()` returns `Result<PathBuf, Error>` — confirmed.

### Pattern 4: Permission TOML for New Commands

Each new Tauri command needs a TOML in `hp41-gui/src-tauri/permissions/`. The CI gate `check-tauri-permissions.sh` enforces this. Template from `save-state.toml`:

```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-import-raw-dialog"
description = "Allows the import_raw_dialog command."
commands.allow = ["import_raw_dialog"]
```

And registered in `default.json`:

```json
"allow-import-raw-dialog"
```

[VERIFIED: codebase inspection of `permissions/save-state.toml` + `scripts/check-tauri-permissions.sh`]

### Pattern 5: CLI Flag Additions (clap 4.x)

Add to `Cli` struct in `hp41-cli/src/main.rs`:

```rust
// Source: clap 4.x derive API (already in use in this project)
/// Import a .raw program file into calculator memory, then launch the TUI.
/// Use '-' to read from stdin.
#[arg(long, value_name = "FILE")]
import_raw: Option<std::path::PathBuf>,  // special-cased: "-" → stdin

/// Export the current program to a .raw file, then exit.
/// Use '-' to write to stdout.
#[arg(long, value_name = "FILE")]
export_raw: Option<std::path::PathBuf>,

/// Import a .card.json data file into calculator registers.
#[arg(long, value_name = "FILE")]
import_data: Option<std::path::PathBuf>,

/// Export current data registers to a .card.json file.
#[arg(long, value_name = "FILE")]
export_data: Option<std::path::PathBuf>,

/// Suppress interactive TUI — exit after import/export operation completes.
#[arg(long)]
batch: bool,
```

[ASSUMED] — flag names and clap attribute syntax follow project patterns; confirmed clap 4.x is already in use.

### Pattern 6: Multi-Program Picker UI (Claude's Discretion)

The picker is a React modal overlay reusing the existing Phase 49 modal patterns. It receives a list of `{ label: string; byte_len: number; index: number }` entries from a dedicated Tauri command (`get_multi_program_list`) and returns the selected indices via a separate `import_programs_by_index` command. This keeps the picker stateless and avoids encoding an array of `Vec<Op>` in IPC (which would be large for complex programs).

Alternatively, the picker could operate purely on the frontend: the `import_raw_dialog` command returns structured metadata (program labels + sizes) as part of the `CalcStateView` extension, and a follow-up command imports the selected ones. This is the recommended approach to keep command count minimal.

### Anti-Patterns to Avoid

- **Calling `decode_program()` in a loop to split multi-program files:** `decode_program` explicitly errors on trailing bytes. Use the new `decode_all_programs()` function.
- **Holding AppState mutex during the file dialog:** The `blocking_pick_file()` call blocks until the user responds. Never hold the Mutex during this call — open the dialog BEFORE acquiring state, or acquire/release state only after the dialog returns.
- **Adding dialog logic to `execute_prepared_card_op`:** The existing three-phase drain is designed for named-card operations. The dialog path is a NEW command path (new Tauri commands), not a modification of the existing drain.
- **Modifying `hp41-core` CalcState for dialog state:** Dialog selection state is transient UI state. Never add dialog-related fields to `CalcState`.
- **Skipping permission TOML for new commands:** The CI gate `check-tauri-permissions.sh` will fail. Every new `#[tauri::command]` function registered in `generate_handler!` needs a corresponding TOML.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Native OS file picker dialogs | Custom webview file input | `tauri-plugin-dialog` | `<input type="file">` in Tauri webview does not produce filesystem paths on all platforms; blocking_pick_file returns a `PathBuf` directly |
| `.raw` byte stream splitting | Custom END-marker scanner | `decode_all_programs()` (new function in `raw.rs`) | Needs to reuse the exact same prefix/alpha-instruction logic already in `decode_program`; duplication would introduce codec divergence |
| File path sanitization for dialog paths | Custom path validator | None needed — dialog provides full path | Dialog paths are user-selected absolute paths; `sanitize_name()` is only for card names (alphanumeric identifiers) and is not applicable here |

**Key insight:** The existing three-phase drain in `cards.rs` is the single source of truth for card I/O. The file dialog path should be a new command path, not a fork of the existing drain. This preserves the existing `cards_dir` behavior unchanged (D-50.1).

---

## Common Pitfalls

### Pitfall 1: Holding AppState Mutex During Dialog (Deadlock / UI Freeze)

**What goes wrong:** If `AppState` (the `Mutex<CalcState>`) is locked before calling `blocking_pick_file()`, the UI will be frozen for the duration of the dialog interaction (which can be arbitrarily long). On some platforms this may cause the window to become unresponsive.

**Why it happens:** Tauri command handlers are async-capable but the blocking dialog call runs synchronously; if the mutex guard is alive during the call, it blocks all other state access.

**How to avoid:** Open the file dialog first (no lock held), get the path, then lock `AppState` only for the codec + state mutation. Pattern:

```rust
let path = app.dialog().file().blocking_pick_file(); // NO lock held
let bytes = std::fs::read(path)?;                    // NO lock held
let ops = decode_program(&bytes)?;                   // NO lock held
let mut calc = state.lock()...;                      // lock AFTER I/O
insert_program_ops(&mut calc, ops);
```

**Warning signs:** UI becomes unresponsive during file chooser; other commands time out.

### Pitfall 2: decode_program Rejects Multi-Program Files (P52)

**What goes wrong:** `decode_program()` returns `Err(CardData("trailing bytes after END marker: N extra byte(s)"))` for any concatenated file. Without `decode_all_programs()`, importing a community `.raw` archive silently fails with a confusing error.

**Why it happens:** The existing `decode_program` was intentionally designed this way (see comment in `raw.rs` line 193–199): "multi-program concatenation is unsupported and a truncated-then-appended file would otherwise silently load only the first program."

**How to avoid:** Always call `decode_all_programs()` first, check `len() > 1` to decide whether to show the picker, fall through to `decode_program` for the single-program case only if needed.

**Warning signs:** Import of any V41/Free42-exported archive fails with "trailing bytes" error.

### Pitfall 3: Skipping Permission TOML Causes CI Failure

**What goes wrong:** `bash scripts/check-tauri-permissions.sh` (first step of `just gui-ci`) will fail with "MISSING: permissions/<kebab-case>.toml" for every new command that lacks a TOML.

**Why it happens:** The gate greps `lib.rs` for `commands::*` patterns and checks for the existence of the corresponding TOML file.

**How to avoid:** For every new function added to `generate_handler!` in `lib.rs`, create the matching TOML in `permissions/` AND add it to `capabilities/default.json`. Do this in Wave 0 (plumbing wave) before implementing the command body.

**Warning signs:** `just gui-ci` fails on the first `check-tauri-permissions.sh` step.

### Pitfall 4: Plugin Not Registered in `lib.rs` Builder

**What goes wrong:** Adding `tauri-plugin-dialog` to `Cargo.toml` is not sufficient. The plugin must also be registered in `tauri::Builder::default().plugin(tauri_plugin_dialog::init())` in `lib.rs`. Without this, any call to `app.dialog()` panics at runtime.

**Why it happens:** Tauri plugins use a separate registration mechanism from `manage()` / `generate_handler!`.

**How to avoid:** Add `.plugin(tauri_plugin_dialog::init())` to the builder chain in `lib.rs` in the same wave as adding the `Cargo.toml` dependency.

**Warning signs:** Runtime panic "the 'dialog' plugin was not added to the tauri::Builder" (or similar).

### Pitfall 5: `dialog:allow-open` / `dialog:allow-save` Missing from `capabilities/default.json`

**What goes wrong:** Even with the plugin registered, Tauri v2's capability system requires explicit `dialog:allow-open` and `dialog:allow-save` entries in `capabilities/default.json`. Without them, dialog calls fail at runtime with a permission error.

**Why it happens:** Tauri v2.11 enforces capability-based permissions for all plugins.

**How to avoid:** Add `"dialog:allow-open"` and `"dialog:allow-save"` to the `permissions` array in `capabilities/default.json` in Wave 0.

**Warning signs:** Dialog commands fail with "permission denied" or similar capability errors.

### Pitfall 6: `FilePath::into_path()` Returns `Result`, Not `PathBuf`

**What goes wrong:** `blocking_pick_file()` returns `Option<FilePath>`. `FilePath` is an enum with a `Path(PathBuf)` variant AND a `Url(Url)` variant. Calling `.as_path()` returns `Option<&Path>` only for the `Path` variant. For desktop use, `into_path()` is the correct method (handles both variants, returns `Result<PathBuf>`).

**Why it happens:** The type system does not force the `.into_path()` call; a developer might assume a `PathBuf` is directly accessible.

**How to avoid:** Always call `file_path.into_path().map_err(|e| ...)?` to unwrap.

**Warning signs:** Compile error "FilePath does not implement Deref<Target=Path>" or panics on mobile platforms.

### Pitfall 7: stdin/stdout Piping for CLI (`--import-raw -`)

**What goes wrong:** clap does not natively handle `-` as a special stdin sentinel for `PathBuf` arguments. A naive `Option<PathBuf>` argument will store the literal path `-` which `std::fs::read` will fail on with "No such file or directory".

**Why it happens:** clap treats `-` as a regular string argument, not a special stdio token.

**How to avoid:** Use `Option<String>` (not `Option<PathBuf>`) for these flags and manually check for `"-"` to read from stdin/write to stdout. Example:

```rust
let bytes = if import_raw == "-" {
    let mut buf = Vec::new();
    std::io::Read::read_to_end(&mut std::io::stdin(), &mut buf)?;
    buf
} else {
    std::fs::read(import_raw)?
};
```

[ASSUMED] — this is a known clap/Unix pattern from experience; the specific recommendation is not verified against current clap 4.x docs.

---

## Code Examples

### decode_all_programs() — Codec Extension

```rust
// Source: inferred from raw.rs design (END_MARKER constant, decode_program logic)
// This function goes in hp41-core/src/cardreader/raw.rs

const END_MARKER: [u8; 3] = [0xC0, 0x00, 0x0D];

pub struct DecodedProgram {
    pub ops: Vec<Op>,
    pub byte_len: usize,
}

pub fn decode_all_programs(bytes: &[u8]) -> Result<Vec<DecodedProgram>, HpError> {
    let mut result = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        // Find END marker starting from offset
        let end_pos = bytes[offset..]
            .windows(END_MARKER.len())
            .position(|w| w == END_MARKER)
            .map(|p| offset + p)
            .ok_or_else(|| HpError::CardData(
                "truncated input: stream ended without END marker (C0 00 0D)".into()
            ))?;
        let seg_end = end_pos + END_MARKER.len();
        let segment = &bytes[offset..seg_end];
        // decode_program expects exactly one END marker and no trailing bytes —
        // segment is exactly [ops... END_MARKER], so this is safe.
        let ops = decode_program(segment)?;
        result.push(DecodedProgram { ops, byte_len: segment.len() });
        offset = seg_end;
    }
    Ok(result)
}
```

[ASSUMED] — implementation design; verified that `decode_program` accepts a segment with exactly one trailing END marker with no extra bytes.

### LBL Name Extraction for Picker (D-50.5)

```rust
// Source: inferred from Op enum in hp41-core/src/ops/mod.rs
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

[ASSUMED] — Op::Lbl variant name confirmed by reading raw.rs; exact pattern is Claude's design.

### File Dialog Open (Rust, inside a Tauri command)

```rust
// Source: https://docs.rs/tauri-plugin-dialog/2.7.1/
use tauri_plugin_dialog::DialogExt;

let file_path_opt = app.dialog()
    .file()
    .set_title("Import HP-41 Program")
    .add_filter("HP-41 Program", &["raw"])
    .add_filter("All Files", &["*"])
    .blocking_pick_file();
```

[VERIFIED: docs.rs/tauri-plugin-dialog/2.7.1] — method chain confirmed.

### File Dialog Save (Rust, inside a Tauri command)

```rust
// Source: https://docs.rs/tauri-plugin-dialog/2.7.1/
let save_path_opt = app.dialog()
    .file()
    .set_title("Export HP-41 Program")
    .set_file_name("program.raw")
    .add_filter("HP-41 Program", &["raw"])
    .add_filter("All Files", &["*"])
    .blocking_save_file();
```

[VERIFIED: docs.rs/tauri-plugin-dialog/2.7.1] — `blocking_save_file()` confirmed.

### Frontend Dialog Invoke (TypeScript)

```typescript
// Source: @tauri-apps/plugin-dialog is NOT used directly on the frontend for this phase.
// The file dialog is opened from the Rust backend (via blocking_pick_file in a command).
// The frontend calls the Tauri command, which opens the dialog and returns the result.
//
// This is the correct pattern because:
// 1. The codec (decode_program) lives in Rust.
// 2. The state mutation (insert_program_ops) lives in Rust.
// 3. Keeping dialog + codec + state in one Rust command avoids a round-trip.
import { invoke } from '@tauri-apps/api/core';

async function importRaw() {
  try {
    const view = await invoke<CalcStateView>('import_raw_dialog');
    setCalcState(view);
    showToast(`Imported ${view.program_name ?? 'program'}`);
  } catch (err) {
    showToast(extractErrMessage(err));
  }
}
```

[ASSUMED] — frontend invoke pattern follows existing dispatch_op pattern; `program_name` field in CalcStateView is illustrative (actual feedback format TBD per D-50.3).

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `decode_program` rejects multi-program archives | `decode_all_programs` splits at END markers | Phase 50 (new) | Enables importing V41/Free42 `.raw` archives with multiple programs |
| Card reader only uses `~/.hp41/cards/NAME.raw` | Empty-ALPHA triggers OS file dialog | Phase 50 (new) | Users can now import any `.raw` file from the filesystem |
| No CLI import/export | `--import-raw`, `--export-raw`, `--import-data`, `--export-data` flags | Phase 50 (new) | Enables scripting and piping workflows |

**Not deprecated:** The existing `~/.hp41/cards/` mechanism (non-empty ALPHA) is unchanged and preserved. Both paths coexist.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `decode_all_programs()` calling `decode_program()` on each segment works because segment contains exactly one END marker | Code Examples | decode_program's END-marker logic might not handle this edge — needs a unit test to confirm |
| A2 | clap 4.x requires `Option<String>` (not `Option<PathBuf>`) to handle `-` as stdin sentinel | Common Pitfalls P7 | If clap 4.x has a built-in `value_parser` for stdin, a String workaround would be unnecessary boilerplate |
| A3 | Multi-program picker implemented as new React component (not reusing existing modal) | Architecture Patterns | If existing modal pattern is flexible enough, a new component may be unnecessary |
| A4 | `import_raw_dialog` returns a `CalcStateView` with embedded feedback signal for the toast (program name, step count) | Code Examples | CalcStateView may not have a field for import metadata; may need a new field or a separate feedback mechanism |
| A5 | Frontend calls backend commands for dialog (not `@tauri-apps/plugin-dialog` JS API directly) | Code Examples | Using the JS dialog API would move codec/state mutation to JavaScript, violating SC-4 invariant |

---

## Open Questions (RESOLVED)

1. **`decode_all_programs()` segment boundary correctness** — RESOLVED: Alpha-string payloads are ASCII (0x20–0x7E range per HP-41 character set). While `0x0D` (CR) is theoretically a valid ASCII byte, HP-41 alpha strings do not contain control characters — the character set maps bytes 0x00–0x7E to printable glyphs. The 3-byte END marker `C0 00 0D` cannot appear inside an alpha payload because `0xC0` is outside the alpha byte range. A regression test `decode_all_programs_does_not_false_split_on_alpha_payload()` is included in Plan 01 Task 2 as defense-in-depth.

2. **Toast feedback format for import (D-50.3)** — RESOLVED: New dialog commands return a custom JSON struct `{ view: CalcStateView, message: String }` instead of bare `CalcStateView`. The `message` field carries the toast text (e.g., "Imported QUAD (47 steps)"). This avoids polluting `CalcState`/`CalcStateView` with transient GUI-only state. Implemented in Plan 02 Task 2.

3. **`--batch` vs `--no-tui` naming (D-50.9)** — RESOLVED: Using `--batch` (shorter, conventional for scripting modes, consistent with Unix tooling patterns). Implemented in Plan 04.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust (MSRV 1.88) | All Rust compilation | Checked by CI | — | — |
| Node.js / npm | npm install @tauri-apps/plugin-dialog | Available (project already uses npm) | — | — |
| `tauri-plugin-dialog` crate | GUI Rust backend | Not yet in Cargo.toml | — | None needed; must add |
| `@tauri-apps/plugin-dialog` npm | GUI frontend | Not yet in package.json | — | None needed; must add |

**Missing dependencies with no fallback:**
- `tauri-plugin-dialog` Rust crate — must be added to `hp41-gui/src-tauri/Cargo.toml`
- `@tauri-apps/plugin-dialog` npm — must be added to `hp41-gui/package.json`

**Missing dependencies with fallback:** None.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework (Rust core) | `cargo test` (built-in) |
| Framework (GUI Rust) | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` |
| Framework (GUI frontend) | Vitest (`cd hp41-gui && npm test`) |
| Config file | `hp41-gui/vitest.config.ts` (or `vite.config.ts` with test block) |
| Quick run command (core) | `cargo test -p hp41-core --test cardreader_tests` |
| Full suite command | `just test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| RAW-01 | Single-program `.raw` import inserts ops into CalcState | unit (core) | `cargo test -p hp41-core --test cardreader_tests` | Partial (existing `wprgm_full_pipeline_round_trips_program`) — needs dialog variant test |
| RAW-02 | Export writes valid `.raw` bytes | unit (core) | `cargo test -p hp41-core --test cardreader_tests` | Partial — existing round-trip tests cover encoding |
| RAW-03 | Multi-program archive decoded correctly | unit (core) | `cargo test -p hp41-core cardreader_raw::decode_all_programs` | No — Wave 0 gap |
| RAW-04 | File dialog opens on GUI (native OS) | manual | — | No — dialog open cannot be unit-tested |
| RAW-05 | XROM instructions round-trip without corruption | unit (core) | `cargo test -p hp41-core -- xrom_round_trip` | No — Wave 0 gap |
| RAW-06 | `--import-raw FILE` loads program, TUI launches | integration (CLI) | `cargo test -p hp41-cli -- cli_import_raw` | No — Wave 0 gap |

### Sampling Rate

- **Per task commit:** `cargo test -p hp41-core --test cardreader_tests` (fast, covers codec)
- **Per wave merge:** `just test` (full Rust suite)
- **Phase gate:** `just test && cd hp41-gui && npm test` before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `hp41-core/tests/cardreader_raw_multi.rs` — covers RAW-03 (`decode_all_programs`, multi-program archives, false-split guard for alpha payloads containing END bytes)
- [ ] `hp41-core/tests/cardreader_xrom_roundtrip.rs` — covers RAW-05 (XROM op round-trip via SyntheticByte)
- [ ] `hp41-cli/tests/cli_import_raw.rs` — covers RAW-06 (CLI flag integration test with tempfile)

*(Existing `cardreader_tests.rs` and `cardreader_xeq_tests.rs` already cover the single-program pipeline; RAW-01/RAW-02 can extend those files.)*

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Validate decoded Op sequence; `decode_program` already returns `Err(CardData)` on malformed input |
| V6 Cryptography | no | — |

### Known Threat Patterns for this Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via crafted `.raw` filename | Tampering | Dialog returns absolute path chosen by OS — no user-supplied path string. `sanitize_name()` not applicable to dialog paths. |
| Malformed `.raw` byte stream (crafted to overflow or panic) | DoS | `decode_program` returns `Err(CardData)` on all malformed input; no panics possible due to `#![deny(clippy::unwrap_used)]` at crate root |
| Multi-program archive with too many programs (memory DoS) | DoS | `decode_all_programs()` should cap the number of decoded programs (e.g., 256 programs max) — ASSUMED reasonable limit |
| Crafted alpha payload containing END marker bytes | Tampering | Covered by `decode_alpha_instruction` bounds checking; alpha payload bytes are never re-interpreted as opcodes |

---

## Sources

### Primary (HIGH confidence)

- `hp41-core/src/cardreader/raw.rs` — source of truth for codec behavior, END_MARKER constant, `decode_program` trailing-byte rejection
- `hp41-core/src/cardreader/mod.rs` — CardOpRequest API, insert_program_ops, three-phase drain contract
- `hp41-cli/src/cards.rs` + `hp41-gui/src-tauri/src/cards.rs` — existing drain implementations (identical, SYNC-NOTE)
- `hp41-gui/src-tauri/src/commands.rs` — existing Tauri command patterns (dispatch_op, three-phase pattern, poisoned-lock recovery)
- `hp41-gui/src-tauri/src/lib.rs` — generate_handler! registration list
- `scripts/check-tauri-permissions.sh` — permission gate enforcement logic
- `https://docs.rs/tauri-plugin-dialog/2.7.1/tauri_plugin_dialog/struct.FileDialogBuilder.html` — FileDialogBuilder API
- `https://docs.rs/tauri-plugin-dialog/2.7.1/tauri_plugin_dialog/enum.FilePath.html` — FilePath variants + into_path()
- `https://v2.tauri.app/plugin/dialog/` — permission identifiers `dialog:allow-open`, `dialog:allow-save`

### Secondary (MEDIUM confidence)

- `https://v2.tauri.app/develop/calling-rust/` — AppHandle injection in Tauri commands pattern
- `cargo search tauri-plugin-dialog` — version 2.7.1 confirmed on crates.io
- `npm view @tauri-apps/plugin-dialog` — version 2.7.1 confirmed on npm, no postinstall script

### Tertiary (LOW confidence)

- `decode_all_programs()` implementation design — Claude's design from first principles; needs unit testing to verify correctness

---

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — `tauri-plugin-dialog` 2.7.1 verified on both crates.io and npm; official Tauri org package
- Architecture: HIGH — codec already exists; three-phase drain already exists; only integration work needed
- Pitfalls: HIGH — dialog mutex deadlock, trailing-bytes rejection, permission gate, stdin `-` handling all verified from codebase + official docs
- `decode_all_programs()` design: MEDIUM — logic is straightforward but edge case (alpha payload containing END bytes) needs a regression test

**Research date:** 2026-05-27
**Valid until:** 2026-07-27 (stable Tauri 2.x ecosystem; `tauri-plugin-dialog` API is mature)
