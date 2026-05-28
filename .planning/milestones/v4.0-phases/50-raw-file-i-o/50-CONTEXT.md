# Phase 50: .raw File I/O - Context

**Gathered:** 2026-05-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can exchange HP-41 programs and data cards with the wider HP-41 community via standard `.raw` files (programs) and `.card.json` files (data registers). Both GUI (native OS file dialog via `tauri-plugin-dialog`) and CLI (`--import-raw`, `--export-raw`, `--import-data`, `--export-data` flags) paths are in scope. The `.raw`/`.card.json` codec is already implemented in `hp41-core/src/cardreader/`; this phase is a frontend integration task.

</domain>

<decisions>
## Implementation Decisions

### File Dialog UX (GUI)
- **D-50.1:** Import/export triggered by extending existing WPRGM/RDPRGM/WDTA/RDTA card reader ops. When ALPHA register is **empty**, the card reader op opens a native file dialog instead of writing to `~/.hp41/cards/`. When ALPHA has a name, existing `~/.hp41/cards/NAME.raw` behavior is preserved unchanged.
- **D-50.2:** File dialog filter — Claude's discretion (pick `.raw` filter + all-files fallback based on `tauri-plugin-dialog` capabilities).
- **D-50.3:** User feedback on import/export: **both** toast message (e.g. "Imported QUAD (47 steps)") **and** status line message (loaded program name). Matches Phase 49 save_state toast pattern.

### Multi-program .raw Handling (RAW-03)
- **D-50.4:** Multi-program `.raw` archives handled via **picker dialog**. Decode all programs from the archive, present a list to the user, let them choose which to import.
- **D-50.5:** Program identification in picker: **LBL + size** format. Extract first LBL instruction as name (e.g. "QUAD (47 bytes)"), fall back to "Program N (M bytes)" if no LBL found.
- **D-50.6:** Picker allows **multi-select** — user can choose multiple programs to import at once. Selected programs are inserted sequentially via `insert_program_ops`.

### Data Card I/O
- **D-50.7:** Phase 50 includes file dialog import/export for data cards (`.card.json` via WDTA/RDTA). Same empty-ALPHA pattern as programs: WDTA/RDTA with empty ALPHA opens file dialog for `.card.json`.

### CLI Flag Design (RAW-06)
- **D-50.8:** CLI flags support **file path + stdin/stdout piping**. `--import-raw FILE` or `--import-raw -` for stdin. `--export-raw FILE` or `--export-raw -` for stdout. Unix-idiomatic.
- **D-50.9:** `--import-raw` default: load program then start interactive TUI. `--batch` or `--no-tui` flag suppresses interactive mode (exit 0 after load) for scripting pipelines.
- **D-50.10:** CLI also gets `--import-data` / `--export-data` flags for `.card.json`, matching the GUI data card decision (D-50.7). Same stdin/stdout piping support.

### Claude's Discretion
- File dialog filter configuration (D-50.2) — pick appropriate filter defaults for `tauri-plugin-dialog`.
- `decode_all_programs()` function design — how to split a multi-program byte stream at END markers.
- Multi-program picker UI implementation — modal overlay, list component, or reuse of existing patterns.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Core Codec
- `hp41-core/src/cardreader/raw.rs` — `.raw` byte codec (`encode_program`, `decode_program`). Currently rejects multi-program files (trailing bytes after END marker). Must be extended with `decode_all_programs()`.
- `hp41-core/src/cardreader/mod.rs` — Card reader API hub (`CardOpRequest` enum, `insert_program_ops`, `capture_data_card`, `load_data_card`). Three-phase drain contract.
- `hp41-core/src/cardreader/data.rs` — `.card.json` codec (`encode_data`, `decode_data`, `DataCard` struct).

### Frontend Integration (CLI)
- `hp41-cli/src/cards.rs` — CLI card reader frontend (3-phase drain, `sanitize_name`, `cards_dir`). SYNC-NOTE: must stay in step with GUI `cards.rs`.
- `hp41-cli/src/main.rs` — CLI entry point. `--import-raw` / `--export-raw` / `--import-data` / `--export-data` flags to be added here (clap 4.x).

### Frontend Integration (GUI)
- `hp41-gui/src-tauri/src/cards.rs` — GUI card reader frontend (same 3-phase drain contract as CLI). File dialog integration point.
- `hp41-gui/src-tauri/src/commands.rs` — Tauri IPC commands. New commands needed for file dialog + multi-program picker.
- `hp41-gui/src-tauri/src/lib.rs` — `generate_handler!` registration list.

### Existing Patterns
- `hp41-gui/src-tauri/permissions/save-state.toml` — Template for new Tauri v2.11 permission TOMLs.
- `hp41-gui/src-tauri/capabilities/default.json` — Permission registration.

### Project Constraints
- `CLAUDE.md` — Frozen invariants, zero new runtime deps policy, workspace structure.
- `.planning/REQUIREMENTS.md` — RAW-01 through RAW-06 requirement definitions.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Three-phase card op drain** (`prepare_pending_card_op` / `execute_prepared_card_op` / `apply_card_read_result`): Already exists in both CLI and GUI `cards.rs`. File dialog path can hook into phase 2 (filesystem I/O) without restructuring.
- **`CardOpRequest` enum**: Already has `WriteProgram`, `ReadProgram`, `WriteData`, `ReadData` variants with name field. File dialog path can be triggered when name is empty.
- **`encode_program` / `decode_program`**: Codec ready for single-program files. Multi-program needs `decode_all_programs()` extension.
- **Toast pattern**: Phase 49's `showToast()` + `extractErrMessage()` in App.tsx — reuse for import/export feedback.
- **`tauri-plugin-dialog`**: Approved new dep. Provides `FileDialogBuilder` with filter, default path, save/open modes.

### Established Patterns
- **`pending_card_op` staging**: Card reader ops set `state.pending_card_op`; frontend drains it after dispatch. File dialog path must integrate here.
- **CR-01 clone-under-lock**: GUI card ops clone state under Mutex, drop lock, do I/O, re-acquire for apply.
- **Permission TOML + default.json**: Every new Tauri command needs a permission TOML and `default.json` entry.
- **`sanitize_name`**: Rejects path separators, NUL, leading dots. File dialog path bypasses this (user picks the full path via OS dialog).

### Integration Points
- **`state.pending_card_op`**: Modified by card reader op dispatch to signal the frontend. File dialog variant needs a way to signal "open dialog" vs "use cards_dir".
- **`handleKey` / `handleClick` in App.tsx**: Card reader ops (Ctrl+W/R/D/F from Phase 49) trigger `xeq_WPRGM` etc. which eventually sets `pending_card_op`. The frontend drain happens in `commands.rs` dispatch thunk.
- **`hp41-gui/src-tauri/src/persistence.rs`**: Auto-save after state changes. Card reader I/O should trigger auto-save after successful import.

</code_context>

<specifics>
## Specific Ideas

- Multi-program picker shows programs as "QUAD (47 bytes)", "Program 2 (23 bytes)" with checkboxes for multi-select.
- CLI piping enables workflows like `curl https://example.com/program.raw | hp41 --import-raw - --batch` for automated loading.
- Data card dialog uses `.card.json` filter, same empty-ALPHA trigger pattern as programs.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 50-raw-file-i-o*
*Context gathered: 2026-05-27*
