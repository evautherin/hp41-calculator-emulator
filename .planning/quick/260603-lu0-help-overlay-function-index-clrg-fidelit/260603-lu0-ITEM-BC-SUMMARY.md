---
phase: quick-260603-lu0
plan: 01-items-BC
subsystem: gui-help-overlay
tags: [help-overlay, tap-to-run, all-functions, tappability-guard]
dependency_graph:
  requires: [260603-lu0-Item-A-CLRG-fidelity-done]
  provides: [tabbed-help-overlay, allFunctionsEntries, xeqToken, tappability_parity-rust-test]
  affects: [hp41-gui/src/HelpOverlay.tsx, hp41-gui/src/help_data.ts, hp41-cli/tests/tappability_parity.rs]
tech_stack:
  added: []
  patterns: [xeq_-prefix dispatch, NON_TAPPABLE dual-pinning TS+Rust]
key_files:
  created:
    - hp41-gui/src/help_data.test.ts
    - hp41-cli/tests/tappability_parity.rs
  modified:
    - hp41-gui/src/help_data.ts
    - hp41-gui/src/HelpOverlay.tsx
    - hp41-gui/src/App.tsx
    - hp41-gui/src/HelpOverlay.test.tsx
decisions:
  - "Dispatch token === display_name for all 303 tappable entries (no JSON schema change, no just docs-matrix)"
  - "NON_TAPPABLE set (61 op_variants) pinned in BOTH TS and Rust — size-pin assertion guards drift"
  - "Single xeq_<token> id for normal-run and PRGM-insert (backend PRGM gate splits behavior)"
  - "Advantage Pac: single header in All Functions tab combining Adv Conv + Adv Math"
  - "iOS-aware default: All Functions on iOS, Keyboard Shortcuts on Desktop; resets on each open"
metrics:
  duration: "~45 minutes"
  completed: "2026-06-03"
  tasks_completed: 5
  tasks_total: 5
  files_modified: 6
---

# Phase lu0 Items B+C: Tabbed All-Functions Overlay + Tap-to-Run Summary

**One-liner:** Two-tab `?` overlay with All Functions tab listing all 303 implemented HP-41 functions
as tappable buttons (xeq_<display_name> dispatch), 61 NON_TAPPABLE rows rendered non-interactive,
iOS-aware default tab, and dual TS+Rust drift guards preventing resolver divergence.

## Item A Record (DONE, Not Re-Executed)

Item A (CLRG mnemonic fidelity fix) was completed before this run (commits 4a7ea8b + merge c0e2559,
D-lu0-01, on-device verified 2026-06-03). Not re-executed here.

## Tasks Executed

| Task | Commit | Files |
|------|--------|-------|
| B1: allFunctionsEntries() + xeqToken() + NON_TAPPABLE | 119e046 | help_data.ts, help_data.test.ts |
| B2: Two-tab HelpOverlay + tap-to-run + iOS-aware default | 5df7098 | HelpOverlay.tsx, App.tsx |
| B3: HelpOverlay tab-switching + tap-to-run tests | f707f61 | HelpOverlay.test.tsx |
| C1: Vitest completeness guardrail (in help_data.test.ts) | 119e046 | help_data.test.ts |
| C2: Rust tappability_parity drift guard | 2daf789 | hp41-cli/tests/tappability_parity.rs |

## Tap-to-Run Design

**Dispatch token = display_name for all tappable entries.** The resolver chain:
- GUI dispatches `xeq_<display_name>` via `invoke('dispatch_op', { keyId })`
- `key_map.rs:427` strips the `xeq_` prefix → `Op::Xeq(label)` with label = display_name
- `op_xeq()` calls `builtin_card_op(label)` then `xrom_resolve(label, modules)` (exact-match, case-sensitive)
- **303 entries** have display_name === resolvable XEQ-by-name token (confirmed by tappability_parity.rs)
- No JSON schema change, no `just docs-matrix` run, no Op enum change required

**PRGM mode is transparent:** The same `xeq_<token>` id dispatched in PRGM mode hits the PRGM gate
in `dispatch()` (`mod.rs:1612`) which pushes `Op::Xeq(label)` as a program step rendering `XEQ <NAME>`.
No frontend branching on prgm state is needed.

## NON_TAPPABLE Set (61 op_variants)

Three classes of non-tappable entries — none XEQ-runnable on a real HP-41 either:

1. **Parameterized ops** (need an argument): StoReg, RclReg, StoArith, StoArithStack, StoM/N/O,
   RclM/N/O, StoInd, RclInd, StoArithInd, FmtFix, FmtSci, FmtEng, SfFlag, CfFlag, SfFlagInd,
   CfFlagInd, FlagTest, FlagTestInd, View, ViewInd, Tone, Isg, Dse, IsgInd, DseInd, Arcl, ArclInd,
   Asto, AstoInd, Gto, GtoInd, Xeq, XeqInd, Lbl, Clp, Del, Asn, Test
2. **Immediate stack/entry keys**: Add, Sub, Mul, Div, Enter, Clx, Chs, Rdn, XySwap, Lastx, PctChange
3. **Mode/composite placeholders**: AlphaToggle, AlphaAppend, AlphaBackspace, PrgmMode, UserMode,
   Catalog, GetKey, Null

**Pinned in both TS and Rust** — the Rust `tappability_parity.rs` test asserts `NON_TAPPABLE.len() == 61`
preventing silent drift between the two lists.

## All Functions Tab Module Order (Locked)

1. HP-41CV Built-in (predicate: `!e.xrom && e.category !== 'Extended Memory'`)
2. Math Pac (MATH 1) (predicate: `e.xrom?.module === 'Math 1'`)
3. Stat Pac (STAT 1) (predicate: `e.xrom?.module === 'Stat 1'`)
4. Time Pac (TIME 2C) (predicate: `e.xrom?.module === 'Time'`)
5. Advantage Pac — SINGLE header combining Adv Conv (XROM 22) + Adv Math (XROM 24)
6. Extended Memory (predicate: `e.xrom === undefined && e.category === 'Extended Memory'`)

## Gate Results

| Gate | Result |
|------|--------|
| `cd hp41-gui && npx tsc --noEmit` | CLEAN |
| `cd hp41-gui && npx vitest run` | 293 tests PASS (12 test files) |
| `cargo test --test tappability_parity` | 2 tests PASS |
| `cd hp41-gui/src-tauri && cargo check` | CLEAN |

**Vitest breakdown (293 total):**
- 43 existing HelpOverlay tests: all pass (existing behaviors preserved)
- 11 new HelpOverlay lu0 tests (B3): tabs, iOS default, All Functions completeness, tap-to-run dispatch
- 9 new help_data tests (B1 + C1): allFunctionsEntries, xeqToken, completeness guardrail
- 230 other existing tests: all pass (no regressions)

## Preserved Behaviors (Not Regressed)

- **o2e (authentic single-step PRGM view):** No changes to program step rendering or PRGM dispatch
- **laz (overlay-close recompute):** HelpOverlay reset-on-open useEffect preserved; tab resets on open
- **mxg (safe-area handling):** No changes to layout/CSS outside the overlay
- **Keyboard Shortcuts tab:** Existing `key_path !== null` filter unchanged; KEYBOARD SHORTCUTS
  collapsible section, sectionGroups render, and per-entry expand all preserved
- **D-26.8 exclusion:** filterHelpEntries, helpOverlayRows, and Keyboard Shortcuts tab still
  exclude key_path:null entries — only the All Functions tab includes them

## CLI↔GUI Parity Deferral

The CLI TUI overlay (`hp41-cli/src/help_data.rs::help_overlay_rows()`) still excludes key_path:null
entries and has no tap-to-run (TUI has no touch). This is intentional — GUI-first per D-lu0-02.
A CLI All-Functions listing is a flagged follow-up, NOT in scope for this plan.

## Sibling-Mnemonic Follow-Ups (Not Fixed Here)

Surfaced during Item A (CLRG): CL SIGMA / CLΣ JSON display_name check; CLRALPHA vs CLA duplicate.
These are NOT fixed here and remain as flagged follow-ups.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — all runnable entries are wired to live dispatch; non-tappable rows are correctly
marked non-interactive. No placeholder data.

## Threat Flags

None — no new network endpoints, auth paths, or trust-boundary crossings introduced.
Tap dispatch reuses the existing `dispatch_op` IPC command + `xeq_` key_map prefix.

## Self-Check: PASSED

- hp41-gui/src/help_data.ts — FOUND (allFunctionsEntries, xeqToken, NON_TAPPABLE exported)
- hp41-gui/src/help_data.test.ts — FOUND (9 tests)
- hp41-gui/src/HelpOverlay.tsx — FOUND (two-tab component, min_lines > 350 satisfied)
- hp41-gui/src/HelpOverlay.test.tsx — FOUND (54 tests)
- hp41-gui/src/App.tsx — FOUND (handleOverlayRun callback + isIos/onRun props wired)
- hp41-cli/tests/tappability_parity.rs — FOUND (2 Rust tests)
- Commits: 119e046, 5df7098, f707f61, 2daf789 — all verified in git log

## Pending: On-Device Checkpoint

Task `checkpoint:human-verify` (gate="blocking") requires on-device (iPhone 15 Pro) verification.
See 260603-lu0-PLAN.md §checkpoint for exact steps. Cannot be satisfied by automated testing.

## On-Device Verification — PASS

Verified on iPhone 15 Pro (2026-06-03): "All Functions" tab is the iOS default and lists all previously-keyless built-ins (CLST/CLRG/SIN/AVIEW...) grouped Module→Category; search finds CLRG/SIN; tapping a function runs it (overlay closes) and inserts `XEQ <NAME>` as a program step in PRGM mode; non-runnable rows are inert.

A follow-up visual pass (commit 652b4b1) made the tab match the HP-41 look (opaque overlay, two-row header with segmented tabs, chrome-less tappable rows with LCD-green op + ▸, dimmed non-runnable rows with ⌨) after on-device feedback that the first cut rendered as default browser pills. Re-approved on device. **Item B+C complete; lu0 fully done.**
