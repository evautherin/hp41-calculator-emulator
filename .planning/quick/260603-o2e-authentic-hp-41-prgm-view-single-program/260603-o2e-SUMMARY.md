---
phase: quick-260603-o2e
plan: 01
subsystem: gui
tags: [parity, display, prgm-mode, cli-gui-parity, cleanup]
dependency_graph:
  requires: []
  provides: [PRGM-AUTHENTIC-01]
  affects: [hp41-gui/src-tauri/src/types.rs, hp41-gui/src-tauri/src/prgm_display.rs, hp41-gui/src/App.tsx, hp41-gui/src/App.css, hp41-gui/src/themes.css, hp41-gui/src/App.test.tsx]
tech_stack:
  added: []
  patterns: [display-priority-chain, cli-gui-parity-D-25.6]
key_files:
  modified:
    - hp41-gui/src-tauri/src/types.rs
    - hp41-gui/src-tauri/src/prgm_display.rs
    - hp41-gui/src/App.tsx
    - hp41-gui/src/App.css
    - hp41-gui/src/themes.css
    - hp41-gui/src/App.test.tsx
decisions:
  - "Inserted prgm_mode branch between entry_buf and alpha_mode in CalcStateView::from_state to match CLI priority order exactly (clock -> stopwatch -> [modal_prompt] -> entry_buf -> prgm_mode -> alpha -> X)"
  - "program_steps/pc retained in CalcStateView IPC contract (not removed) — follow-up optimization deferred per plan followups"
  - "modal_prompt branch (GUI-only) left at its existing position; it is gated disjointly (modal_program.is_some() && entry_buf.is_empty() && modal_prompt.is_some()) so it cannot conflict with prgm_mode view"
metrics:
  duration_secs: 199
  completed_date: "2026-06-03T15:32:11Z"
  tasks_completed: 3
  tasks_total: 4
  files_changed: 6
---

# Phase quick-260603-o2e Plan 01: Authentic HP-41 PRGM View (Single-Step Display) Summary

**One-liner:** Restored hardware-faithful PRGM display by wiring `prgm_display::format_step` into the GUI display_str priority chain and removing the inauthentic multi-line program listing UI (CLI<->GUI parity D-25.6 restored).

## What Changed

### Task 1 — Backend: prgm_mode branch in CalcStateView::from_state (1eeba7e)

The GUI display_str priority chain in `hp41-gui/src-tauri/src/types.rs` was missing a `prgm_mode` branch. The CLI (`hp41-cli/src/ui.rs:get_display_string`) already showed the current program step via `prgm_display::format_step`; the GUI silently diverged by falling through to the X-register display.

Fix: inserted `} else if state.prgm_mode { prgm_display::format_step(state) }` between the `entry_buf` and `alpha_mode` arms, exactly mirroring the CLI priority. The `#[allow(dead_code)]` attribute on `format_step` was removed (it now has a real caller). Priority-chain comment updated to note D-25.6 parity.

New Rust tests in `types.rs`:
- `prgm_mode=true` + empty `entry_buf` → `display_str == "000 END"`
- `prgm_mode=true` + non-empty `entry_buf` → entry_buf wins (program entry in progress)
- `prgm_mode=false` → normal X-register display

### Task 2 — Frontend: deleted inauthentic PRGM listing UI (810a15f)

Removed from `App.tsx`:
- `const activeStepRef = useRef<HTMLDivElement>(null)` (L268) — no remaining consumer
- The `useEffect` that scrolled `activeStepRef` on `[calcState?.pc]` changes
- The entire PRGM ternary block (iOS `<BottomSheet id="prgm-sheet">` + desktop `<div className="prgm-panel">`) including its "Phase 55 Plan 05 — PRGM panel" comment
- `calcState?.annunciators.prgm` removed from the recompute-scale `useEffect` deps array; comment updated to reflect print-sheet-only responsibility

Removed from `App.css`:
- The entire "Program Listing Panel (Phase 18)" CSS block: `.prgm-panel`, `.prgm-panel-header`, `.prgm-panel-content`, `.step-row`, `.step-active`

Removed from `themes.css`:
- `--step-active-bg` and `--step-active-text` custom-property lines from all four `[data-theme]` blocks (dark, light, classic-beige, high-contrast) — no consumer after `.step-active` was deleted

**Untouched (verified):**
- `<BottomSheet id="print-sheet">` and its content (print log) — byte-for-byte unchanged
- Generic `BottomSheet` component and `BottomSheet.test.tsx`
- `printLog`, `printEndRef`, the print scroll `useEffect`
- `CalcStateView` interface: `program_steps: string[]` and `pc: number` retained in IPC contract

### Task 3 — Frontend regression tests (519b0f2)

Added `describe('quick-task 260603-o2e — authentic PRGM step in main display', ...)` to `App.test.tsx`:
- **O2E-1**: `display_str="000 END"` + `annunciators.prgm=true` → `getDisplayText(container)` returns `"000 END"` (not `"0.0000"`)
- **O2E-2**: `display_str="001 XEQ CLRG"` + `prgm=true` → display shows `"001 XEQ CLRG"` (guards against accidental hardcoded "END")

Full test suite: **273 tests across 11 files — all passed**.

## Gate Results

| Gate | Result |
|------|--------|
| `cargo check` (src-tauri) | PASS — no errors, no dead_code warnings |
| `cargo test prgm_display` | PASS — 7 tests |
| `cargo test test_prgm_mode_display_str` | PASS — 1 test |
| `tsc --noEmit` (hp41-gui) | PASS — clean |
| `npm test -- --run` (vitest) | PASS — 273/273 tests |
| Listing-UI grep gate | PASS — 0 references remain in App.tsx/App.css/themes.css |
| hp41-core files changed | NONE — only hp41-gui/* |

## CLI<->GUI Parity (D-25.6)

After this change, both frontends implement the same display priority order:

```
clock → stopwatch → [modal_prompt (GUI-only, disjoint gate)] → entry_buf → prgm_mode → alpha → X
```

The CLI reference is `hp41-cli/src/ui.rs:get_display_string` (L133-161). The GUI implementation is `hp41-gui/src-tauri/src/types.rs:CalcStateView::from_state` (display_str chain, L130+). They are now equivalent.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. `program_steps` and `pc` remain in `CalcStateView` IPC but are unused by the frontend (intentional follow-up per plan `<followups>` section — field removal deferred to avoid unrelated churn in tests/IPC consumers).

## Task 4 (On-Device Human Verify) — PASS

Verified on iPhone 15 Pro (2026-06-03): in PRGM mode the main display shows the
current step (e.g. `000 END` / `001 XEQ CLRG`), SST/BST navigate steps in the main
display, no PROGRAM bottom sheet appears (overlap gone), and the print sheet still
works. **Approved.**

## Self-Check: PASSED

Verified:
- `hp41-gui/src-tauri/src/types.rs` — prgm_mode branch present between entry_buf and alpha_mode
- `hp41-gui/src-tauri/src/prgm_display.rs` — `#[allow(dead_code)]` removed from format_step
- `hp41-gui/src/App.tsx` — no `activeStepRef`, no `prgm-sheet`, no `prgm-panel`, no `scrollIntoView` on pc, `calcState?.annunciators.prgm` removed from scale-effect deps
- `hp41-gui/src/App.css` — no `.prgm-panel*` or `.step-row` or `.step-active` rules
- `hp41-gui/src/themes.css` — no `--step-active-bg` or `--step-active-text` in any theme block
- `hp41-gui/src/App.test.tsx` — two new O2E tests present
- Commits: `1eeba7e`, `810a15f`, `519b0f2` all present in git log
