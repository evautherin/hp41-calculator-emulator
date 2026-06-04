---
phase: quick-260603-lu0
plan: 01
subsystem: hp41-core/resolver, hp41-cli/prgm_display, hp41-gui/prgm_display, docs/json
tags: [clrg, mnemonic-fidelity, resolver, 4-way-invariant, json-canonical]
dependency_graph:
  requires: []
  provides: [CLRG-FIDELITY]
  affects: [hp41-core/ops/program.rs, docs/hp41cv-functions.json, prgm_display (both), docs/hp41cv-function-matrix.md]
tech_stack:
  added: []
  patterns: [4-way-exhaustive-match, multi-alias resolver arm (CLRG|CLREG)]
key_files:
  created: []
  modified:
    - docs/hp41cv-functions.json
    - hp41-core/src/ops/program.rs
    - hp41-cli/src/prgm_display.rs
    - hp41-gui/src-tauri/src/prgm_display.rs
    - docs/hp41cv-function-matrix.md
    - hp41-cli/tests/xeq_builtin_resolver.rs
decisions:
  - "Option (i): rename JSON display_name CLREG→CLRG (authentic HP-41 mnemonic); keep CLREG as back-compat XEQ alias via multi-arm resolver. op_variant stays Clreg (case-insensitive internal token) — no Op enum change."
metrics:
  duration: ~15min
  completed: 2026-06-03
  tasks_completed: 2
  tasks_total: 2 (Item A only; Items B+C pending)
verification:
  on_device: "PASS (iPhone 15 Pro, 2026-06-03) — PRGM program line renders `XEQ CLRG` (authentic mnemonic, not CLREG); XEQ-by-name resolves. Item A accepted."
  items_b_c: "PENDING — tabbed full-function overlay + coverage guardrail not yet executed"
  follow_ups: "On-device session surfaced two new iOS issues (NOT part of lu0): (1) XEQ-by-name touch entry is opaque (ENTER types ALPHA 'N'; submit = ALPHA key — TOUCH-04 design gap); (2) PRGM mode adds a program-source bar that breaks the calculator layout / clips the top display under the status bar and is hard to exit. Both to be triaged separately."
---

# Quick Task 260603-lu0 Plan 01 Summary

## One-liner

CLRG mnemonic fidelity fix: JSON display_name, 4-way op_display_name mirrors, and multi-alias resolver arm (CLRG|CLREG) — all quality gates green.

## Status

**Item A: COMPLETE** (awaiting on-device iPhone human-verify checkpoint)
**Item B: PENDING** (tabbed overlay — not started)
**Item C: PENDING** (Vitest guardrail — not started)

## What Was Built

### Task A1 — CLREG→CLRG rename + multi-alias resolver

Applied the locked decision (option i):

1. **docs/hp41cv-functions.json** (line 481): `"display_name": "CLREG"` → `"display_name": "CLRG"`. `op_variant: "Clreg"` unchanged (keeps ALL_OP_VARIANT_NAMES forward parity stable).

2. **hp41-core/src/ops/program.rs** (builtin_card_op, ~line 1418): `"CLREG" => Some(Op::Clreg)` → `"CLRG" | "CLREG" => Some(Op::Clreg)`. Mirrors the existing multi-alias pattern for CL SIGMA (line 1427). Both the authentic mnemonic and the legacy alias now resolve.

3. **hp41-cli/src/prgm_display.rs** (line 109): `Op::Clreg => "CLREG".to_string()` → `"CLRG"`. (4-way invariant, half 3/4)

4. **hp41-gui/src-tauri/src/prgm_display.rs** (line 126): same change. (4-way invariant, half 4/4 — duplicated by design)

5. **docs/hp41cv-function-matrix.md**: regenerated via `just docs-matrix`.

### Task A2 — Resolver regression test

**hp41-cli/tests/xeq_builtin_resolver.rs** (stack_clear_resolves test, ~line 76): added `assert_eq!(resolve("CLRG"), Some(Clreg));` next to the existing CLREG assertion. Both asserted and labeled (authentic vs. back-compat alias).

## Quality Gates Passed

| Gate | Result |
|------|--------|
| `just docs-matrix-check` | PASSED (no drift) |
| `cargo test --test xeq_builtin_resolver` | PASSED (17 tests) |
| `cargo test --test function_matrix_parity` | PASSED (26 tests — forward + reverse, including CLRG reverse lookup) |
| `cargo test` (full CLI suite) | PASSED (455 passed, 4 ignored) |
| `cargo check` (hp41-gui/src-tauri) | PASSED (4-way exhaustive match compiles) |

## Commit

`4a7ea8b` — `fix(clrg): rename CLREG→CLRG (authentic HP-41 mnemonic) + add CLRG resolver alias`

Tasks A1 + A2 committed atomically (per plan constraint).

## Pending Human-Verify Checkpoint (Item A)

On-device verification required before Items B+C can proceed:

1. Build iOS app: `cd hp41-gui && npm run tauri ios build`
2. Install/launch via `xcrun devicectl`
3. XEQ "CLRG" — set a register (5 STO 01), XEQ "CLRG", RCL 01 → expect 0.000
4. XEQ "CLREG" (legacy alias) — confirm still clears registers
5. Enter PRGM mode, key the clear-registers op — confirm line displays "CLRG" (not "CLREG")

Resume signal: type "approved" or describe the issue.

## Pending Items

### Item B — Tabbed full-function overlay (NOT STARTED)

Files: `hp41-gui/src/HelpOverlay.tsx`, `hp41-gui/src/App.tsx`, `hp41-gui/src/help_data.ts`, `hp41-gui/src/help_data.test.ts`

Tasks B1 + B2: add `allFunctionsEntries()` builder and two-tab overlay (Keyboard Shortcuts | All Functions) with iOS-aware default tab.

### Item C — Coverage guardrail (NOT STARTED)

File: `hp41-gui/src/help_data.test.ts`

Task C1: Vitest guardrail asserting every `status===implemented` function is present in the All Functions dataset.

## Deviations from Plan

None — plan executed exactly as written for Items A1 and A2.

## Known Stubs

None introduced by Item A.

## Follow-up Candidates (Surface to User — NOT Fixed in This Plan)

Sibling mnemonic divergences noticed during A1:

1. **CL SIGMA / CLΣ** — `hp41-core/src/ops/program.rs:1427` already has the multi-alias resolver arm `"CL SIGMA" | "CLΣ" | "CLSIGMA"`. The JSON `display_name` in `docs/hp41cv-functions.json` should be checked — if it reads `"CL SIGMA"` vs the authentic `"CLΣ"`, a similar rename may be warranted.

2. **CLRALPHA / CLA duplicate** — `program.rs:1419` has `"CLA" => Some(Op::Cla)` and `program.rs:1434` has `"CLRALPHA" => Some(Op::AlphaClear)`. These appear to be distinct ops (CLA = clear alpha register via the Op::Cla variant; CLRALPHA = also clear alpha). Check whether this is intentional or a duplicate resolver pointing to different variants for the same operation.

Neither of these is a bug that blocks functionality — recording for future audit.

## CLI↔GUI Parity Deferral

The CLI TUI overlay (`hp41-cli/src/help_data.rs`, `help_overlay_rows`, ~line 268) still applies the `key_path:null` exclusion — the 74 built-in XEQ-by-name-only functions (SIN, LN, SQRT, AVIEW, CLRG, …) are invisible in the CLI overlay.

Per the GUI-first recommendation in the plan, CLI tab parity is **out of scope** for this quick task. This is a consciously deferred gap, not an oversight. Flag for a future CLI overlay enhancement pass.
