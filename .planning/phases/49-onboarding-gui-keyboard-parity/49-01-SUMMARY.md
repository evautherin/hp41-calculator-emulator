---
phase: 49-onboarding-gui-keyboard-parity
plan: "01"
subsystem: hp41-gui
tags: [tauri, preferences, persistence, keyboard-shortcuts, ipc]
dependency_graph:
  requires: []
  provides:
    - GuiPrefs.onboarding_done field (prefs.rs)
    - save_state Tauri command (commands.rs)
    - docs/keyboard-shortcuts.json single source of truth
  affects:
    - hp41-gui/src-tauri/src/prefs.rs
    - hp41-gui/src-tauri/src/commands.rs
    - hp41-gui/src-tauri/src/lib.rs
    - hp41-gui/src-tauri/capabilities/default.json
tech_stack:
  added: []
  patterns:
    - CR-01 clone-under-lock for save_state
    - serde(default) for forward/backward compat on new prefs field
    - module-qualified persistence::save_state to avoid name shadowing
key_files:
  created:
    - hp41-gui/src-tauri/permissions/save-state.toml
    - docs/keyboard-shortcuts.json
  modified:
    - hp41-gui/src-tauri/src/prefs.rs
    - hp41-gui/src-tauri/src/commands.rs
    - hp41-gui/src-tauri/src/lib.rs
    - hp41-gui/src-tauri/capabilities/default.json
decisions:
  - "onboarding_done stored in prefs.json not autosave.json (P59 / D-49.4)"
  - "save_state uses module-qualified persistence::save_state to avoid name shadowing (RESEARCH Pitfall 1)"
  - "F5=SAVE in GUI is deliberate divergence from CLI F5=run_program (KBD-02 / D-49.13)"
  - "keyboard-shortcuts.json includes 61 entries covering all existing + Phase 49 new bindings"
metrics:
  duration: "~10 minutes"
  completed: "2026-05-27T16:30:21Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 2
  files_modified: 4
---

# Phase 49 Plan 01: Rust Backend Extensions + Keyboard Shortcuts Data File Summary

**One-liner:** GuiPrefs extended with `onboarding_done` bool, `save_state` Tauri command wired with permission TOML, and `keyboard-shortcuts.json` created as 61-entry single source of truth for all GUI physical keyboard bindings.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Extend GuiPrefs + set_pref + save_state command + permission TOML | 2c99199 | prefs.rs, commands.rs, lib.rs, save-state.toml, default.json |
| 2 | Create keyboard-shortcuts.json data file | 6fd1764 | docs/keyboard-shortcuts.json |

## What Was Built

### Task 1: Rust Backend Extensions

**prefs.rs changes:**
- Added `onboarding_done: bool` with `#[serde(default)]` to `GuiPrefs` struct
- Updated `impl Default for GuiPrefs` to initialize `onboarding_done: false`
- Added `test_onboarding_done_roundtrip` — verifies `true` survives save/load cycle
- Added `test_onboarding_done_serde_default` — verifies old `{"theme":"dark"}` format loads with `onboarding_done == false` (backward compat)
- Fixed two existing tests that constructed `GuiPrefs` structs without the new field

**commands.rs changes:**
- Added `use crate::persistence;` import for module-qualified save_state call
- Extended `set_pref` match block with `"onboarding_done"` arm: parses `value == "true"` as bool (T-49-01 mitigation — safe coercion)
- Added `save_state` Tauri command: CR-01 clone-under-lock pattern, calls `persistence::save_state` to avoid name shadowing, returns `Result<(), String>`

**lib.rs changes:**
- Added `commands::save_state` to `tauri::generate_handler!` list

**save-state.toml (new):**
- Tauri v2.11 permission TOML with `identifier = "allow-save-state"` and `commands.allow = ["save_state"]`

**default.json changes:**
- Added `"allow-save-state"` to permissions array

### Task 2: keyboard-shortcuts.json

Created `docs/keyboard-shortcuts.json` with 61 entries covering:
- Digits 0-9, decimal point, EEX, CHS (n key)
- Navigation: Enter (ENTER), Backspace (CLX), Tab (SHIFT)
- Arithmetic: +, -, *, /, s (SQRT), % (PCT CHG)
- Stack: r (R↓), x (X<>Y), l (LASTX)
- Trig: q (SIN), a (ASIN), C (COS), c (ACOS), T (TAN), k (ATAN)
- Transcendental: L (LN), G (LOG), E (EXP), H (10^X), I (1/X), W (X^2), Y (Y^X)
- Statistics: z (SIGMA+), Z (SIGMA-), m (MEAN), D (SDEV), y (Y-HAT), b (LR), O (CORR), V (CL SIGMA)
- Time: h (HMS→H), j (HMS+), J (HMS-)
- Mode: u (USER), p (PRGM), P (PRX / SHIFT+P)
- Misc: g (CLREG)
- Program/debug: F7 (SST), F8 (BST)
- Card reader (Phase 49): Ctrl+W (WPRGM), Ctrl+R (RDPRGM), Ctrl+D (WDTA), Ctrl+F (RDTA)
- Save (Phase 49): Ctrl+S (SAVE), F5 (SAVE — GUI-only divergence from CLI)

## Verification

- `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- prefs::` → 8 tests passed
- `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml -- commands::` → 16 tests passed
- `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml` → clean compile
- Python JSON validation: 61 entries, all fields present

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed GuiPrefs struct-literal compilation errors in existing tests**
- **Found during:** Task 1 — first test run
- **Issue:** Adding `onboarding_done` field to `GuiPrefs` struct caused two existing tests (`test_roundtrip` and `test_unknown_theme_falls_back_to_default`) to fail to compile because they used struct literal syntax without the new field
- **Fix:** Added `onboarding_done: false` to both struct literals
- **Files modified:** hp41-gui/src-tauri/src/prefs.rs
- **Commit:** 2c99199 (fixed in the same commit as the feature addition)

## Known Stubs

None — all plan artifacts are fully wired. The `keyboard-shortcuts.json` documents Phase 49 Ctrl+key bindings that are implemented in Plan 49-02 (App.tsx keyboard handler extensions), which is by design: the data file is created as the single source of truth before the frontend consumes it.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries beyond what was planned and covered by the threat model.

## Self-Check: PASSED

Files exist:
- FOUND: hp41-gui/src-tauri/src/prefs.rs
- FOUND: hp41-gui/src-tauri/src/commands.rs
- FOUND: hp41-gui/src-tauri/src/lib.rs
- FOUND: hp41-gui/src-tauri/permissions/save-state.toml
- FOUND: hp41-gui/src-tauri/capabilities/default.json
- FOUND: docs/keyboard-shortcuts.json

Commits exist:
- FOUND: 2c99199 (feat(49-01): extend GuiPrefs with onboarding_done + add save_state command)
- FOUND: 6fd1764 (feat(49-01): create keyboard-shortcuts.json single source of truth)
