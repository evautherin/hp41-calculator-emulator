---
phase: quick-260603-scc
plan: inline
subsystem: hp41-cli/help_data
tags: [cli, tui, help-overlay, cli-gui-parity, discoverability]
dependency_graph:
  requires: [help-overlay-all-functions-index]   # 260603-lu0 (GUI)
  provides: [cli-help-overlay-complete]
  affects: [hp41-cli/src/help_data.rs]
key_files:
  modified:
    - hp41-cli/src/help_data.rs
    - hp41-cli/tests/phase25_help_data.rs
decisions:
  - "CLI `?` overlay already showed all 6 pools — gap was only: keyless built-ins had a blank key column, and deferred-v3 rows leaked in. No new widget / tab needed (single keyboard-driven overlay is the right CLI pattern; the GUI's two-tab split exists only because iOS has no keyboard)."
  - "Keyless built-ins now advertise `XEQ \"NAME\"` in the key column — the CLI analog of the GUI tap-to-run. There is no real tap-to-run in a terminal; discoverability = find the name, then f-N XEQ-by-name modal."
  - "Right-panel key_ref_entries() (keyboard-shortcut reference) left unchanged."
metrics:
  duration: "~20 min (inline)"
  completed: "2026-06-03"
  commit: "b47f921"
verification:
  terminal: "Test-verified (no device needed). help_overlay_rows_cover_all_implemented_functions (C1 analog) + help_overlay_rows_suggest_xeq_for_keyless_builtins green; full hp41-cli suite passes."
---

# Quick Task 260603-scc: CLI `?` Overlay Completeness (CLI↔GUI Parity)

Follow-up to [[260603-lu0]]. Brings the CLI terminal `?` function overlay to parity
with the new GUI All-Functions index.

## What the audit found
The CLI `?` overlay (ratatui Table, search-filtered) ALREADY rendered all 382 entries
from all 6 pools, grouped by category. The only gaps:
1. **Keyless built-ins** (74 cv-pool entries with `key_path:null`, no XROM) rendered
   with a **blank key column** — no hint they run via XEQ-by-name.
2. **Deferred-v3 entries** (18) appeared as if runnable.
3. No CLI analog of the GUI's C1 completeness guardrail.

(The right-panel `key_ref_entries()` keyboard reference is a separate, deliberately
narrow surface — left unchanged.)

## Changes (`help_overlay_rows()` in help_data.rs)
- Filter to `status == "implemented"` (drop deferred-v3), mirroring the GUI's
  `allFunctionsEntries()`.
- Keyless built-ins (`key_path:null && xrom.is_none()`) → key column shows
  `XEQ "NAME"` (e.g. `XEQ "CLST"`, `XEQ "SIN"`). XROM entries already carried
  `key_path = XEQ "NAME"` from the JSON.
- Updated `help_overlay_rows_contain_category_headers` to derive expected categories
  from implemented-only entries (a deferred-only category no longer emits a header).
- Added `help_overlay_rows_cover_all_implemented_functions` (C1 analog) and
  `help_overlay_rows_suggest_xeq_for_keyless_builtins`.

## Gates
- Full `cargo test` (hp41-cli): all binaries green (incl. phase25_help_data 9/9,
  key_ref includes-math1/stat1/time, function_matrix_parity 26, tappability_parity).
- `cargo check` clean.

## Not done / scope
- Optional module section headers (`=== Math 1 Pac ===` parent grouping above the
  flat categories) — deferred; everything is already findable via category + search.
- Verify interactively with `cargo run` / `just` then press `?` (test-covered).
