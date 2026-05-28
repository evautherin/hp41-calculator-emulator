---
phase: 52-test-hardening-documentation
plan: "01"
subsystem: help-data
tags: [help-data, x-mem, json-pool, docs-matrix, vitest, smoke-test]
dependency_graph:
  requires: []
  provides: [docs/hp41-xmem-functions.json, hp41-cli/src/help_data.rs::help_entries_xmem, hp41-gui/src/help_data.ts::helpEntriesXmem]
  affects: [help-overlay-cli, help-overlay-gui, docs-matrix-ci]
tech_stack:
  added: []
  patterns: [OnceLock-JSON-pool-Rust, Vite-static-JSON-import-TS, docs-matrix-title-dispatch]
key_files:
  created:
    - docs/hp41-xmem-functions.json
    - docs/hp41-xmem-function-matrix.md
    - hp41-cli/tests/phase52_help_data_xmem.rs
  modified:
    - scripts/docs-matrix/src/main.rs
    - justfile
    - hp41-cli/src/help_data.rs
    - hp41-gui/src/help_data.ts
    - hp41-gui/src/HelpOverlay.test.tsx
decisions:
  - "X-MEM entries have no xrom field — they are HP-41CX OS built-ins, not XROM ops (D-52.4)"
  - "help_entries_all() six-pool chain order: built-ins → Math1 → Stat1 → Time → Adv → X-MEM (D-52.1)"
  - "Advantage xrom-truthy loop in HelpOverlay.test.tsx bounded to advStart+advCount to avoid false-failing on X-MEM entries"
metrics:
  duration_seconds: 373
  completed_date: "2026-05-28"
  tasks_completed: 3
  tasks_total: 3
  files_created: 3
  files_modified: 5
---

# Phase 52 Plan 01: X-MEM Help Pool + Docs-Matrix Wiring Summary

**One-liner:** 8-entry X-MEM JSON help pool wired into CLI OnceLock + GUI Vite import with dedicated "Extended Memory" overlay section and generated matrix doc.

## What Was Built

### Task 1: docs/hp41-xmem-functions.json + docs-matrix wiring
Created `docs/hp41-xmem-functions.json` with exactly 8 entries covering all HP-41CX Extended Memory OS built-in operations (EMDIR, EMROOM, SAVEP, GETP, SAVED, GETD, EMREG, SAVERX). Each entry has `category: "Extended Memory"`, a `XEQ "<MNEMONIC>"` key_path, a <=80-char description, and full `example` + `notes` fields per D-52.3. No `xrom` field on any entry (X-MEM are OS built-ins, not XROM).

Added a title-dispatch arm to `scripts/docs-matrix/src/main.rs` for `hp41-xmem-functions.json` → title "# HP-41CX Extended Memory Function Matrix". Extended `docs-matrix` and `docs-matrix-check` justfile recipes to generate and drift-check `docs/hp41-xmem-function-matrix.md` (D-52.2).

### Task 2: CLI + GUI help pool wiring + HelpOverlay.test.tsx + matrix doc
Added `XMEM_FUNCTIONS_JSON` constant + `XMEM_HELP_ENTRIES` OnceLock + `pub fn help_entries_xmem()` to `hp41-cli/src/help_data.rs`. Extended `help_entries_all()` to chain X-MEM sixth (D-52.1). Updated doc comment with proper list indentation to pass clippy.

Added `import xmemFunctions` + `export function helpEntriesXmem()` to `hp41-gui/src/help_data.ts`. Extended `helpEntriesAll()` to spread X-MEM sixth. No changes to HelpOverlay.tsx — `helpOverlayRows()` automatically generates the "Extended Memory" section because X-MEM entries have non-null `key_path` and `category: "Extended Memory"`.

Updated `HelpOverlay.test.tsx`:
- Added `helpEntriesXmem` to import destructuring
- Renamed "all 5 pools" test to "all 6 pools"; added `+ helpEntriesXmem().length` to exact-sum assertion
- Bounded Advantage xrom-truthy loop to `advStart + advCount` (not `all.length`)
- Added X-MEM loop asserting `xrom === undefined` and `category === 'Extended Memory'`

Generated `docs/hp41-xmem-function-matrix.md` via `just docs-matrix`. `just docs-matrix-check` passes. `just gui-ci` passes (210 tests green).

### Task 3: phase52_help_data_xmem.rs smoke tests (TDD)
Created `hp41-cli/tests/phase52_help_data_xmem.rs` with 6 tests mirroring `phase44_help_data_adv.rs`:
1. `xmem_help_entries_is_not_empty` — hard-build-blocker exercised on success path
2. `xmem_help_entries_count_meets_8_target` — exact == 8 (feature-frozen)
3. `help_entries_all_returns_six_pools` — >= 358 total entries
4. `help_overlay_rows_includes_xmem_section` — "Extended Memory" header present
5. `xmem_every_entry_has_category_and_key_path` — category + key_path invariants
6. `xmem_op_variants_match_expected_set` — op_variant drift-catch (bonus)

All 6 tests pass (`cargo test -p hp41-cli --test phase52_help_data_xmem`).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed clippy doc_lazy_continuation in help_entries_all() doc comment**
- **Found during:** Task 3 lint verification
- **Issue:** New multi-line doc comment used continuation text after a bullet list without blank line, triggering `clippy::doc_lazy_continuation`
- **Fix:** Added blank line before the list and restructured the opening sentence to put the list in its own paragraph block
- **Files modified:** `hp41-cli/src/help_data.rs`
- **Commit:** 323f12e

## Known Stubs

None — all 8 X-MEM entries are implemented ops, not placeholders.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries. All JSON is compile-time-embedded (`include_str!` / Vite import).

## TDD Gate Compliance

Task 3 has `tdd="true"`. The plan's task ordering deliberately places the implementation (Task 2) before the smoke-test authoring (Task 3), so the tests passed on first run. Per task spec: "the implementation is already done in Task 2 — the tests confirm the implementation is correct."

Gate sequence:
- RED: test file created at `hp41-cli/tests/phase52_help_data_xmem.rs` (commit 323f12e test(...) prefix)
- GREEN: all 6 tests pass immediately because implementation was in Task 2

## Self-Check

Files created/modified:
- `docs/hp41-xmem-functions.json` ✓
- `docs/hp41-xmem-function-matrix.md` ✓
- `scripts/docs-matrix/src/main.rs` ✓
- `justfile` ✓
- `hp41-cli/src/help_data.rs` ✓
- `hp41-gui/src/help_data.ts` ✓
- `hp41-gui/src/HelpOverlay.test.tsx` ✓
- `hp41-cli/tests/phase52_help_data_xmem.rs` ✓

Commits:
- 6558c5a: feat(52-01): add X-MEM JSON pool + docs-matrix generator arm + justfile wiring
- 77982ec: feat(52-01): wire X-MEM sixth help pool CLI+GUI; update HelpOverlay tests; generate matrix doc
- 323f12e: test(52-01): add phase52_help_data_xmem.rs smoke tests for X-MEM help pool (XMEM-10)

## Self-Check: PASSED
