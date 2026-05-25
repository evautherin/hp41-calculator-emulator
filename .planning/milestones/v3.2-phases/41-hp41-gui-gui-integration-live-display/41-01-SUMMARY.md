---
phase: 41-hp41-gui-gui-integration-live-display
plan: "01"
subsystem: hp41-gui + hp41-core
tags: [gui, tauri, time-pac, live-display, catalog, xrom]
dependency_graph:
  requires: []
  provides:
    - tick_time Tauri command for live clock/stopwatch display
    - CalcStateView.clock_active + CalcStateView.stopwatch_keyboard_mode projections
    - Generic 3-module CATALOG 2 loop in hp41-core
    - TIME-GUI-01 confirmed (all 35 Time arms in prgm_display.rs)
  affects:
    - hp41-gui/src-tauri/src/types.rs (new fields in CalcStateView)
    - hp41-gui/src-tauri/src/commands.rs (new tick_time command)
    - hp41-core/src/ops/program.rs (op_catalog refactored to generic loop)
tech_stack:
  added: []
  patterns:
    - tick_time Tauri command mirrors handle_get_state pattern + check_alarms call
    - CalcStateView field projection for transient CalcState booleans
    - Generic xrom_registry array loop over (XromModule, u8) tuples
    - Tauri v2.11 permission TOML + capabilities/default.json registration
key_files:
  created:
    - hp41-gui/src-tauri/permissions/tick-time.toml
  modified:
    - hp41-core/src/ops/program.rs
    - hp41-core/tests/op_catalog_xrom.rs
    - hp41-gui/src-tauri/src/commands.rs
    - hp41-gui/src-tauri/src/types.rs
    - hp41-gui/src-tauri/src/lib.rs
    - hp41-gui/src-tauri/capabilities/default.json
    - hp41-gui/src-tauri/src/prgm_display.rs
decisions:
  - D-41.5 generic loop replaces parallel if-blocks in op_catalog CATALOG 2
  - D-41.1/D-41.4 tick_time mirrors get_state + calls check_alarms before draining
  - D-41.3 clock_active + stopwatch_keyboard_mode projected into CalcStateView
  - TIME-GUI-01 already satisfied — 35 Time arms confirmed by cargo check exit 0
metrics:
  duration: "20 minutes"
  completed: "2026-05-25"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 7
  files_created: 1
---

# Phase 41 Plan 01: Rust Backend Infrastructure (prgm_display + op_catalog + tick_time) Summary

Generic CATALOG 2 loop over 3 XROM modules + tick_time Tauri command with clock_active/stopwatch_keyboard_mode CalcStateView projections, closing the sanctioned GUI CI break (TIME-GUI-01) and enabling live display polling.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Verify prgm_display.rs + op_catalog generic loop refactor + catalog test | ab55a52 | program.rs, op_catalog_xrom.rs, prgm_display.rs |
| 2 | CalcStateView projections + tick_time command + Tauri permission + handler registration | ad727d0 | types.rs, commands.rs, lib.rs, tick-time.toml, default.json |

## What Was Built

### Task 1: prgm_display.rs + op_catalog refactor

**TIME-GUI-01 confirmed satisfied:** `cargo check -p hp41-gui` exits 0 with zero `non-exhaustive patterns` warnings. All 35 Time Module Op arms were already present in `hp41-gui/src-tauri/src/prgm_display.rs` lines 346-382 (written during Phase 38/39 work). The stale doc comment at line 46 was updated to include "+ Time Module".

**op_catalog generic loop (D-41.5):** Replaced the parallel if-blocks in `hp41-core/src/ops/program.rs` CATALOG 2 block with a generic `xrom_registry: &[(&XromModule, u8)]` array loop containing MATH_1 (bit 0), STAT_1 (bit 1), and TIME_MODULE (bit 2). The latent else-if bug (line 364 in the old code: `else if state.xrom_modules & 0b0000_0001 == 0` that printed "NO XROM" incorrectly when only bit-2 was set) is now eliminated. The fourth module (Advantage Pac) will extend this array automatically in a future phase.

**Two new catalog tests:**
- `catalog_2_lists_time_when_bit2_set`: xrom_modules = 0b0000_0111, all three module headers present, spot-checks POLY/ΣBSTAT/TIME, no "NO XROM", correct line count
- `catalog_2_time_only`: xrom_modules = 0b0000_0100, only Time Module enumerated, regression guard against the old else-if bug

### Task 2: CalcStateView + tick_time

**CalcStateView new fields (D-41.3):** Added `pub clock_active: bool` and `pub stopwatch_keyboard_mode: bool` to the CalcStateView struct in `types.rs`, projected from `state.clock_active` and `state.stopwatch_keyboard_mode` in `from_state()`. JSON size budget comments updated to reflect ~52 bytes additional for two booleans; both ≤500 and ≤600 byte budget assertions continue to pass.

**tick_time command (D-41.1/D-41.4):**
- `pub fn tick_time` Tauri command thunk: acquires AppState Mutex with `.unwrap_or_else(|e| e.into_inner())`, calls `handle_tick_time`
- `pub fn handle_tick_time` helper: calls `hp41_core::ops::time::alarm::check_alarms(calc)` FIRST (D-41.4), then drains `print_buffer` and `event_buffer`, returns `CalcStateView::from_state(...)`
- Two unit tests added: `handle_tick_time_drains_event_buffer_and_returns_view` and `handle_tick_time_returns_clock_active_field`

**Registration and permissions:**
- `commands::tick_time` added to `generate_handler!` macro in `lib.rs`
- `hp41-gui/src-tauri/permissions/tick-time.toml` created with `identifier = "allow-tick-time"`
- `"allow-tick-time"` added to `capabilities/default.json` permissions array

## Verification Results

```
cargo check -p hp41-gui: exits 0 (0 warnings)
cargo test -p hp41-core --test op_catalog_xrom: 6 passed (4 existing + 2 new)
cargo test -p hp41-gui -- handle_tick_time: 2 passed
cargo test -p hp41-gui (full suite): 87 passed
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Test substring false positive in catalog_2_time_only**
- **Found during:** Task 1 verification
- **Issue:** Initial `catalog_2_time_only` test used `!all_text.contains("XROM 2")` to verify Stat 1 was absent, but "XROM 2" is a substring of "XROM 26" (the Time Module header). The test failed incorrectly.
- **Fix:** Changed to `!all_text.contains(STAT_1.name)` (matching "STAT 1B" exact string), which is unambiguous and more semantically correct
- **Files modified:** `hp41-core/tests/op_catalog_xrom.rs`
- **Commit:** ab55a52 (same commit after fix)

## Known Stubs

None. All new functions are fully implemented. The `tick_time` command is complete and calls the live `check_alarms()` function from `hp41-core`.

## Threat Flags

No new threat surface beyond what was captured in the plan's `<threat_model>`. The `tick_time` endpoint is covered by T-41-01 (DoS via Mutex contention, mitigated by frontend busyRef guard in Plan 41-03). The new `CalcStateView` fields expose only boolean flags — no sensitive data (T-41-02 accepted).

## Self-Check: PASSED

Files exist:
- FOUND: hp41-gui/src-tauri/permissions/tick-time.toml
- FOUND: tick_time in lib.rs generate_handler!
- FOUND: allow-tick-time in capabilities/default.json
- FOUND: clock_active in types.rs CalcStateView
- FOUND: stopwatch_keyboard_mode in types.rs CalcStateView
- FOUND: check_alarms in commands.rs handle_tick_time
- FOUND: xrom_registry loop in program.rs
- FOUND: TIME_MODULE imported in program.rs

Commits exist:
- ab55a52: feat(41-01): refactor op_catalog to generic 3-module loop + Time Module tests
- ad727d0: feat(41-01): add tick_time command + CalcStateView live-display fields
