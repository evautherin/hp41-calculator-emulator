# Phase 50: .raw File I/O - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-27
**Phase:** 50-raw-file-i-o
**Areas discussed:** File dialog UX, Multi-program .raw, Data card I/O, CLI flag design

---

## File Dialog UX

| Option | Description | Selected |
|--------|-------------|----------|
| Extend card reader ops | WPRGM/RDPRGM with file dialog when ALPHA empty | ✓ |
| Dedicated menu/buttons | Import/Export buttons in SettingsPanel or File menu | |
| Both paths | File dialog on card ops + dedicated buttons | |

**User's choice:** Extend card reader ops
**Notes:** Clean extension of existing WPRGM/RDPRGM/WDTA/RDTA flow.

| Option | Description | Selected |
|--------|-------------|----------|
| Cards dir when named | ALPHA has name → cards dir; ALPHA empty → dialog | ✓ |
| Always dialog | File dialog always opens, pre-filled when ALPHA has name | |
| You decide | Claude picks | |

**User's choice:** Cards dir when named

| Option | Description | Selected |
|--------|-------------|----------|
| .raw filter + All | Default *.raw filter with all-files secondary | |
| .raw only | Strict filter | |
| You decide | Claude picks based on tauri-plugin-dialog | ✓ |

**User's choice:** You decide (Claude's discretion)

| Option | Description | Selected |
|--------|-------------|----------|
| Toast message | Toast like "Imported TEST (12 steps)" | |
| Status line | Display in calculator status line | |
| Both | Toast for success/error + status line for program name | ✓ |

**User's choice:** Both toast and status line feedback

---

## Multi-program .raw

| Option | Description | Selected |
|--------|-------------|----------|
| Import all sequentially | Decode all, concatenate, insert all | |
| Import first + warn | Import first only with warning toast | |
| Picker dialog | Show list, let user choose which to import | ✓ |

**User's choice:** Picker dialog

| Option | Description | Selected |
|--------|-------------|----------|
| LBL-based names | First LBL instruction as name | |
| Index + size | "Program 1 (47 bytes)" | |
| LBL + size | "QUAD (47 bytes)" with fallback | ✓ |

**User's choice:** LBL + size

| Option | Description | Selected |
|--------|-------------|----------|
| Single selection | One program per import | |
| Multi-select | Checkboxes for multiple programs | ✓ |

**User's choice:** Multi-select

---

## Data Card I/O

| Option | Description | Selected |
|--------|-------------|----------|
| Include data cards | Same empty-ALPHA pattern for WDTA/RDTA → .card.json dialog | ✓ |
| Programs only | Defer data card file dialog | |
| You decide | Claude assesses effort | |

**User's choice:** Include data cards

---

## CLI Flag Design

| Option | Description | Selected |
|--------|-------------|----------|
| File path only | --import-raw FILE / --export-raw FILE | |
| File + stdin/stdout | FILE or - for stdin/stdout piping | ✓ |
| You decide | Claude picks based on clap patterns | |

**User's choice:** File + stdin/stdout

| Option | Description | Selected |
|--------|-------------|----------|
| Load + TUI | Import then start interactive TUI | |
| Load + exit | Import and exit silently | |
| Both via flag | TUI default, --batch/--no-tui for silent | ✓ |

**User's choice:** Both via flag

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, include | --import-data / --export-data for .card.json | ✓ |
| Programs only | Defer data card CLI flags | |

**User's choice:** Yes, include data card CLI flags

---

## Claude's Discretion

- File dialog filter configuration (D-50.2)
- `decode_all_programs()` function design
- Multi-program picker UI implementation

## Deferred Ideas

None — discussion stayed within phase scope.
