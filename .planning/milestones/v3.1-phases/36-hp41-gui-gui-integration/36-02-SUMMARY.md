---
phase: 36-hp41-gui-gui-integration
plan: "02"
subsystem: hp41-gui (TypeScript + Rust)
tags: [gui, help-overlay, stat1-pac, lcd-alternation, modal-prompt, prgm_display]
dependency_graph:
  requires:
    - 34-01 (docs/hp41-stat1-functions.json authored — third JSON source of truth)
    - 33-03 (Stat1Step enum + modal.rs dispatch)
    - 31-04 (HelpOverlay two-section foundation)
  provides:
    - HelpOverlay three-section layout (hp41cv + math1 + stat1)
    - helpEntriesStat1() 3-pool accessor in help_data.ts
    - 5 LCD-alternation modal prompt integration tests for Stat1Step
    - GUI prgm_display.rs 26 Stat 1 Op arms (closes 4-way invariant item 4)
  affects:
    - hp41-gui/src/help_data.ts
    - hp41-gui/src/HelpOverlay.tsx
    - hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt_stat1.rs
    - hp41-gui/src-tauri/src/prgm_display.rs
tech_stack:
  added: []
  patterns:
    - Third Vite static JSON import (parallel to CLI third OnceLock pattern per D-34.2)
    - 3-pool helpEntriesAll() in-place update (Pitfall 5 avoidance)
    - SectionDef union type widening pattern for collapsible overlay sections
    - Stat1Step LCD-alternation routing via CalcStateView::from_state
key_files:
  created:
    - hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt_stat1.rs
  modified:
    - hp41-gui/src/help_data.ts
    - hp41-gui/src/HelpOverlay.tsx
    - hp41-gui/src-tauri/src/prgm_display.rs
decisions:
  - 3-pool helpEntriesAll() updated in-place per Pitfall 5 (no parallel helpEntriesAll3())
  - prgm_display.rs 26 Stat 1 arms added as Rule 3 auto-fix to unblock test compilation
metrics:
  duration_minutes: ~8
  completed_date: "2026-05-24"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 4
  files_created: 1
---

# Phase 36 Plan 02: Stat 1 Pac GUI Help Overlay + LCD Modal Prompt Tests Summary

Third Vite JSON import + 3-pool helpEntriesAll() + Stat 1 Pac HelpOverlay section + 5 LCD-alternation modal prompt integration tests for all Stat1Step variants (STAT-GUI-03 + STAT-GUI-04).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Extend help_data.ts with third Vite import + 3-pool helpEntriesAll() | `6b789bc` | hp41-gui/src/help_data.ts |
| 2 | Extend HelpOverlay.tsx with Stat 1 Pac section + LCD modal prompt tests | `aabf865` | hp41-gui/src/HelpOverlay.tsx, hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt_stat1.rs, hp41-gui/src-tauri/src/prgm_display.rs |

## What Was Built

**Task 1 — help_data.ts 3-pool extension:**
- Added `import stat1Functions from '../../docs/hp41-stat1-functions.json'` (third Vite static import, parallel to CLI third OnceLock per D-34.2)
- Added `helpEntriesStat1()` accessor returning `stat1Functions as readonly HelpEntry[]`
- Updated `helpEntriesAll()` in-place to return `[...helpEntries(), ...helpEntriesMath1(), ...helpEntriesStat1()]` (3-pool chain, Pitfall 5: no parallel function created)
- TypeScript compiles clean

**Task 2A — HelpOverlay.tsx Stat 1 Pac section:**
- Widened `SectionDef.id` union from `'hp41cv' | 'math1'` to `'hp41cv' | 'math1' | 'stat1'`
- Added third SECTIONS entry: `{ id: 'stat1', heading: 'Stat 1 Pac (XROM 2)', predicate: (e) => e.xrom?.module === 'Stat 1' }`
- Widened `expanded` state type to `{hp41cv: boolean; math1: boolean; stat1: boolean}`; initial state + useEffect reset both include `stat1: true`
- Widened `toggleSection` id parameter to `'hp41cv' | 'math1' | 'stat1'`
- No JSX changes needed — existing `sectionGroups.map()` render loop handles the third section automatically

**Task 2B — lcd_alternation_modal_prompt_stat1.rs (5 tests):**
- `normd_mode_choice_prompt_renders_verbatim` — ΣNORMD MODE? (12 chars, no truncation)
- `chisqd_nu_prompt_renders_verbatim` — ν=? (3 chars, no truncation)
- `chisqd_mode_choice_prompt_truncates` — ΣCHISQD MODE? (13 chars → "\u{03A3}CHISQD MOD\u{2261}", 12 chars total, LCD truncation verified)
- `polyp_degree_prompt_renders_verbatim` — DEGREE=? (8 chars, no truncation)
- `seed_prompt_renders_verbatim` — SEED? (5 chars, no truncation)
- All 5 pass; ΣCHISQD MODE? test asserts both the truncated string and chars().count() == 12

## Verification Results

- `cd hp41-gui && npx tsc --noEmit` — exits 0 (TypeScript clean)
- `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml --test lcd_alternation_modal_prompt_stat1` — 5/5 tests pass

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added 26 Stat 1 Op arms to prgm_display.rs to unblock test compilation**
- **Found during:** Task 2 — cargo test compile step for lcd_alternation_modal_prompt_stat1.rs
- **Issue:** `hp41-gui/src-tauri/src/prgm_display.rs` had a `non-exhaustive patterns` compile error (the sanctioned Phase 33 CI break) — `Op::SigmaSpear` and 25 more Stat 1 variants missing from the match. Since `lcd_alternation_modal_prompt_stat1.rs` imports `hp41_gui_lib::types::CalcStateView`, the entire `hp41_gui_lib` crate must compile for the test to link. This is the same work that Plan 36-01 (parallel wave-1 agent) performs, but without it my test could not compile.
- **Fix:** Added all 26 Stat 1 Op display-name arms to `prgm_display.rs` using CLI `prgm_display.rs` as the reference (identical display strings per 4-way invariant). This closes 4-way invariant item 4.
- **Files modified:** `hp41-gui/src-tauri/src/prgm_display.rs`
- **Commit:** `aabf865` (same commit as Task 2)
- **Note:** Plan 36-01 was supposed to handle prgm_display.rs. Both plans are wave-1 parallel. If Plan 36-01 also adds the same arms, the merge will show a conflict that the orchestrator must resolve. The arms are identical to Plan 36-01's intent, so merge resolution should be trivial.

## Requirements Coverage

- STAT-GUI-03: HelpOverlay shows 3 sections including "Stat 1 Pac (XROM 2)" — verified by SECTIONS array + TypeScript compile
- STAT-GUI-04: All 5 Stat1Step modal prompts route through CalcStateView correctly including ΣCHISQD MODE? truncation — verified by 5 passing integration tests

## Known Stubs

None — all wiring is real data from docs/hp41-stat1-functions.json (26 entries) and real Stat1Step dispatch.

## Threat Flags

None. The Vite JSON import consumes `docs/hp41-stat1-functions.json` at build time (T-36-03 accept per plan threat model). No new network endpoints, auth paths, or trust boundaries introduced.

## Self-Check: PASSED

- `hp41-gui/src/help_data.ts` — modified (import + helpEntriesStat1 + helpEntriesAll update): FOUND
- `hp41-gui/src/HelpOverlay.tsx` — modified (SectionDef, SECTIONS, expanded, toggleSection): FOUND
- `hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt_stat1.rs` — created (5 tests): FOUND
- `hp41-gui/src-tauri/src/prgm_display.rs` — modified (26 arms): FOUND
- Commit `6b789bc` (Task 1): FOUND
- Commit `aabf865` (Task 2): FOUND
- All 5 tests pass: VERIFIED
- TypeScript compiles clean: VERIFIED
