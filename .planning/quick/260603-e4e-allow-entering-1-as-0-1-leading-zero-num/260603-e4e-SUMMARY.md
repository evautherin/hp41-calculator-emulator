---
phase: quick-260603-e4e
plan: "01"
subsystem: number-entry
tags: [hp41cv-parity, entry-buf, cli-gui-parity, tdd]
dependency_graph:
  requires: []
  provides: [leading-zero-decimal-entry]
  affects: [hp41-cli/src/app.rs, hp41-gui/src-tauri/src/commands.rs]
tech_stack:
  added: []
  patterns: [entry_buf seeding, CLI-GUI parity D-25.6]
key_files:
  modified:
    - hp41-cli/src/app.rs
    - hp41-gui/src-tauri/src/commands.rs
decisions:
  - "Seed '0.' on empty entry_buf in both CLI and GUI decimal handlers — frontend-only, no hp41-core change needed"
  - "Duplicate-dot guard runs before empty-check so '0.' seeds do not re-trigger (correct order maintained)"
metrics:
  duration: "8 min"
  completed_date: "2026-06-03"
---

# Quick Task 260603-e4e: Leading-Zero Number Entry Summary

**One-liner:** HP-41CV-faithful leading-zero decimal entry — pressing '.' on an empty buffer seeds "0." instead of "." in both CLI and GUI, making '.1' display and flush as '0.1'.

## What Was Done

### Task 1 — Production code (feat commit d127e97)

**hp41-cli/src/app.rs** (`if c == '.'` block, ~line 650):
After the existing duplicate-'.'/'e' guard, added an `is_empty()` branch: empty buffer → `push_str("0.")`, non-empty → existing `push('.')`. Same guard order preserved.

**hp41-gui/src-tauri/src/commands.rs** (`if key_id == "."` block, ~line 226):
Inside the existing `if !contains('.') && !contains('e')` body, same `is_empty()` branch mirrors the CLI change bit-for-bit (D-25.6 parity).

### Task 2 — Tests (test commit 8b98d3b)

**CLI** — 4 new tests in `mod tests` (alongside existing decimal tests):
- `test_decimal_on_empty_buf_seeds_leading_zero` — '.' on empty → "0."
- `test_decimal_then_digit_yields_zero_point_one` — '.' '1' → "0.1"; Enter → stack X == 0.1000
- `test_zero_decimal_digit_no_double_zero_regression` — '0' '.' '1' → "0.1" (not "00.1")
- `test_second_decimal_after_zero_point_one_is_blocked` — second '.' silently ignored

**GUI** — 4 new tests in `mod tests` (alongside existing eex_chs tests):
- `test_gui_decimal_on_empty_buf_seeds_leading_zero`
- `test_gui_decimal_then_digit_yields_zero_point_one`
- `test_gui_zero_decimal_digit_no_double_zero_regression`
- `test_gui_second_decimal_after_zero_point_one_is_blocked`

## Test Results

| Suite | Before | After |
|-------|--------|-------|
| hp41-cli (all) | 444 | 452 passed, 4 ignored |
| hp41-gui (all) | 113 | 117 passed |

All new tests pass. All existing tests continue to pass.

## Commits

| # | Hash | Type | Description |
|---|------|------|-------------|
| 1 | d127e97 | feat | Seed "0." on empty-buffer decimal in CLI and GUI |
| 2 | 8b98d3b | test | Add CLI + GUI tests for leading-zero decimal entry |

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

None — change is purely within existing entry_buf string manipulation; no new network endpoints, auth paths, file access patterns, or schema changes.

## Self-Check: PASSED

- hp41-cli/src/app.rs: modified (production + tests) — confirmed
- hp41-gui/src-tauri/src/commands.rs: modified (production + tests) — confirmed
- Commit d127e97 exists: confirmed
- Commit 8b98d3b exists: confirmed
- All 452 CLI tests pass, 117 GUI tests pass — confirmed
