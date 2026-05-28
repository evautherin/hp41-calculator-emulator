---
phase: 50-raw-file-i-o
plan: 02
subsystem: gui-tauri-commands
tags: [rust, tauri, file-dialog, raw-codec, gui]

# Dependency graph
requires:
  - phase: 50-01
    provides: "decode_all_programs, DecodedProgram, picker_label in hp41-core"
provides:
  - "tauri-plugin-dialog 2.x integrated as Tauri plugin in hp41-gui"
  - "import_raw_dialog command: native OS file dialog + single/multi dispatch"
  - "export_raw_dialog command: native OS save dialog + encode_program write"
  - "import_data_dialog command: native OS file dialog + decode_data + load_data_card"
  - "export_data_dialog command: native OS save dialog + capture_data_card + encode_data"
  - "import_selected_programs command: re-decode archive + import by index"
  - "5 permission TOMLs + default.json entries for all new commands"
  - "ImportRawResponse enum for structured single/multi/cancelled/empty responses"
affects: [50-03, 50-04, frontend-picker, raw-file-io-gui]

# Tech tracking
tech-stack:
  added:
    - "tauri-plugin-dialog = \"2\" (hp41-gui/src-tauri Cargo.toml only — zero new hp41-core deps)"
  patterns:
    - "Anti-deadlock dialog-before-lock: dialog + file I/O complete BEFORE AppState Mutex acquired"
    - "Three-phase I/O: (1) brief lock for snapshot, (2) unlock for dialog/I/O, (3) lock for mutation"
    - "ImportRawResponse enum (serde tag = type): Single/Multi/Cancelled/Empty"
    - "T-50-04 path validation: is_absolute() + is_file() before read in import_selected_programs"

key-files:
  created:
    - hp41-gui/src-tauri/permissions/import-raw-dialog.toml
    - hp41-gui/src-tauri/permissions/export-raw-dialog.toml
    - hp41-gui/src-tauri/permissions/import-data-dialog.toml
    - hp41-gui/src-tauri/permissions/export-data-dialog.toml
    - hp41-gui/src-tauri/permissions/import-selected-programs.toml
  modified:
    - hp41-gui/src-tauri/Cargo.toml
    - hp41-gui/src-tauri/src/lib.rs
    - hp41-gui/src-tauri/src/commands.rs
    - hp41-gui/src-tauri/capabilities/default.json

key-decisions:
  - "tauri-plugin-dialog uses major version spec (\"2\") matching Tauri 2.11 version family"
  - "import_raw_dialog returns serde-tagged enum ImportRawResponse; multi-program returns file_path for re-decode by import_selected_programs"
  - "export commands use serde_json::Value return type (cancelled vs message are structurally different)"
  - "import_selected_programs validates absolute path + is_file() (T-50-04 path traversal mitigation)"
  - "dialog:allow-open + dialog:allow-save added to default.json (RESEARCH Pitfall 5 mitigation)"
  - "Plugin registered via .plugin(tauri_plugin_dialog::init()) BEFORE .setup() (RESEARCH Pitfall 4 mitigation)"

requirements-completed: [RAW-01, RAW-02, RAW-04]

# Metrics
duration: 4min
completed: 2026-05-27
---

# Phase 50 Plan 02: Tauri File Dialog Commands Summary

**5 Tauri commands wired to tauri-plugin-dialog for .raw program and .card.json data card import/export, with anti-deadlock dialog-before-lock pattern and structured multi-program picker response**

## Performance

- **Duration:** ~4 min
- **Started:** 2026-05-27T19:27:36Z
- **Completed:** 2026-05-27T19:31:34Z
- **Tasks:** 2 of 2
- **Files modified/created:** 9

## Accomplishments

- `tauri-plugin-dialog = "2"` added to `hp41-gui/src-tauri/Cargo.toml`; plugin registered via `.plugin(tauri_plugin_dialog::init())` in `lib.rs` Builder chain (RESEARCH Pitfall 4 mitigated)
- `import_raw_dialog`: opens native OS file picker, calls `decode_all_programs`, returns `ImportRawResponse::Single` (imports immediately) or `ImportRawResponse::Multi` (returns metadata + file_path for the frontend picker)
- `export_raw_dialog`: snapshots program bytes under brief lock, releases lock, opens OS save dialog, writes `.raw` file
- `import_data_dialog`: opens native OS file picker, decodes `.card.json`, loads data card into registers (D-50.7)
- `export_data_dialog`: snapshots data card under brief lock, releases lock, opens OS save dialog, writes `.card.json` (D-50.7)
- `import_selected_programs`: re-reads file by path, re-decodes archive, inserts selected programs by index (no IPC state cache needed)
- All 5 commands follow the anti-deadlock pattern: dialog/file I/O happen BEFORE any AppState Mutex lock
- `ImportRawResponse` serde-tagged enum provides structured single/multi/cancelled/empty responses for frontend
- `MultiProgramInfo` struct (label, index, byte_len) populates the frontend picker per D-50.4/D-50.5
- 5 permission TOMLs created; `dialog:allow-open` + `dialog:allow-save` added to `default.json`
- `check-tauri-permissions.sh` passes: OK — all 18 commands have permission TOMLs
- `cargo check` passes with zero errors and zero warnings

## Task Commits

1. **Task 1: Add tauri-plugin-dialog dep, register plugin, and create permission plumbing** — `bb061f8`
2. **Task 2: Implement 5 Tauri command handlers for file dialog I/O** — `1a7514b`

## Files Created/Modified

- `hp41-gui/src-tauri/Cargo.toml` — added `tauri-plugin-dialog = "2"` to [dependencies]
- `hp41-gui/src-tauri/src/lib.rs` — added `.plugin(tauri_plugin_dialog::init())` + 5 new command registrations
- `hp41-gui/src-tauri/src/commands.rs` — added 340 LOC: 5 new commands + ImportRawResponse + MultiProgramInfo
- `hp41-gui/src-tauri/capabilities/default.json` — 7 new permission entries (5 allow-* + dialog:allow-open + dialog:allow-save)
- `hp41-gui/src-tauri/permissions/import-raw-dialog.toml` — new permission TOML
- `hp41-gui/src-tauri/permissions/export-raw-dialog.toml` — new permission TOML
- `hp41-gui/src-tauri/permissions/import-data-dialog.toml` — new permission TOML
- `hp41-gui/src-tauri/permissions/export-data-dialog.toml` — new permission TOML
- `hp41-gui/src-tauri/permissions/import-selected-programs.toml` — new permission TOML

## Decisions Made

- `tauri-plugin-dialog` uses `"2"` (major version only) matching the project's Tauri 2.11 version family convention
- `import_raw_dialog` returns `ImportRawResponse` (serde-tagged `#[serde(tag = "type")]` enum) because the three response shapes (Single, Multi, Cancelled/Empty) are structurally incompatible
- Multi-program response includes `file_path: String` — `import_selected_programs` re-reads + re-decodes the file; this avoids caching `Vec<Op>` in AppState (which would add a CalcState field) or in IPC (which would be large)
- Export commands return `serde_json::Value` (not CalcStateView) because the export result is a message/cancelled flag, not state
- `import_selected_programs` validates `is_absolute()` + `is_file()` before reading (T-50-04 path traversal mitigation — file_path comes from prior `import_raw_dialog` call but passes through JavaScript before returning)

## Deviations from Plan

None — plan executed exactly as written. Both tasks implemented in a single pass since Task 2 implementation was needed for Task 1 compilation (commands.rs needed to exist before lib.rs could reference the command functions).

## Threat Model Coverage

| Threat ID | Status | Notes |
|-----------|--------|-------|
| T-50-04 | Mitigated | `import_selected_programs` validates is_absolute() + is_file() before read |
| T-50-05 | Mitigated | Dialog opens BEFORE AppState lock; Pitfall 1 anti-deadlock pattern applied to all 5 commands |
| T-50-06 | Accepted | User chose save location via OS dialog; no leakage |
| T-50-07 | Mitigated | decode_all_programs/decode_data return HpError on malformed input; GuiError::from propagates |

## Self-Check: PASSED

| Check | Result |
|-------|--------|
| Cargo.toml contains tauri-plugin-dialog | FOUND |
| lib.rs contains .plugin(tauri_plugin_dialog::init()) before .setup() | FOUND |
| lib.rs generate_handler! includes all 5 new commands | FOUND |
| 5 permission TOMLs exist in permissions/ directory | FOUND |
| default.json includes all 5 allow-* + dialog:allow-open + dialog:allow-save | FOUND |
| commands.rs contains all 5 #[tauri::command] functions | FOUND |
| All 5 commands use dialog/I/O before AppState lock | FOUND |
| import_raw_dialog returns ImportRawResponse with Single/Multi | FOUND |
| import_selected_programs validates path before read | FOUND |
| cargo check passes with zero errors | PASS |
| check-tauri-permissions.sh passes (18/18 commands) | PASS |
| Commit bb061f8 (Task 1) | FOUND |
| Commit 1a7514b (Task 2) | FOUND |

---
*Phase: 50-raw-file-i-o*
*Completed: 2026-05-27*
