---
phase: 34-hp41-cli-cli-integration
plan: "02"
subsystem: hp41-cli
tags: [prgm-display, exhaustive-match, stat1-pac, xeq-by-name, modal-flow, key-ref-exclusion, ci-break-closure]
dependency_graph:
  requires: [34-01-PLAN (help_entries_stat1, docs/hp41-stat1-functions.json), 33-PLAN (Op::Sigma*/Rand/Seed)]
  provides: [op_display_name 26 new arms, function_matrix_parity 3-pool, phase34_modal_flow.rs, phase34_key_ref_includes_stat1.rs]
  affects: [prgm_display.rs, function_matrix_parity.rs, phase25_xeq_by_name.rs, key_coverage.rs]
tech_stack:
  added: []
  patterns: [exhaustive-match closure, 3-pool parity test partition, carrier-enum dispatch test]
key_files:
  created:
    - hp41-cli/tests/phase34_modal_flow.rs
    - hp41-cli/tests/phase34_key_ref_includes_stat1.rs
  modified:
    - hp41-cli/src/prgm_display.rs
    - hp41-cli/tests/function_matrix_parity.rs
    - hp41-cli/tests/phase25_xeq_by_name.rs
    - hp41-cli/tests/key_coverage.rs
decisions:
  - "4-way exhaustive-match invariant item 3 sealed — 26 arms close the Phase 33 non-exhaustive patterns CI break"
  - "key_coverage.rs bitfield 0b0000_0001 → 0b0000_0011 (Rule 1 fix — Stat 1 entries now in help_entries_all())"
  - "phase34_modal_flow.rs uses simple carrier-enum dispatch tests (no CalcState), not full App simulation per CONTEXT discretion"
  - "STAT-CLI-01 is verification-only — no keys.rs code change (Phase 29 wiring already routes Stat 1)"
metrics:
  duration: "~20 minutes"
  completed: "2026-05-23T11:18:44Z"
  tasks_completed: 6
  files_created: 2
  files_modified: 4
---

# Phase 34 Plan 02: CLI Integration — Op Display Arms + Parity + Modal Flow Summary

## One-liner

26 `op_display_name` arms seal the 4-way exhaustive-match invariant item 3, closing the Phase 33 sanctioned CI break; 3-pool bidirectional parity, Stat 1 resolver verification, modal-flow cross-checks, and right-panel exclusion guards complete STAT-CLI-01/03/04/05.

## What Shipped

### Task 1: prgm_display.rs — 26 new op_display_name arms (commit 6addc88)

Added exhaustive match arms in `hp41-cli/src/prgm_display.rs` for all 26 Phase 33 Stat 1 Pac Op variants.

- Arms grouped by family with ASCII section dividers matching Phase 28 style
- Σ encoded as `\u{03A3}` per existing convention (e.g. existing `Op::SigmaPlus => "\u{03A3}+"`)
- Stale "Covers all 35 Op variants" doc-comment replaced with invariant-style comment referencing FN-CLI-04
- No `_ =>` catch-all introduced; exhaustive match is the invariant
- `cargo check -p hp41-cli` exits 0; Plan 34-01's deferred 12 tests also unblocked

**Task 1 verification:**
```
cargo check -p hp41-cli: exit code 0
cargo build -p hp41-cli --tests --no-run 2>&1 | grep -c "non-exhaustive patterns": 0
grep -A 400 "fn op_display_name" hp41-cli/src/prgm_display.rs | grep -c "^\s*_ =>": 0
grep -c "Covers all 35 Op variants" hp41-cli/src/prgm_display.rs: 0
grep -c "FN-CLI-04" hp41-cli/src/prgm_display.rs: 1
All 26 variant arms: 1 occurrence each (verified via grep)
Existing in-file tests: 3 passed (test_display_phase12_op_labels, test_display_phase20_op_labels, test_display_phase24_ind_op_labels)
Plan 34-01 deferred tests: 12 passed (phase34_help_data_stat1.rs now unblocked)
```

### Task 2: function_matrix_parity.rs — 3-pool partition (commit 19a51f6)

Extended `hp41-cli/tests/function_matrix_parity.rs` from 2-pool to 3-pool walk.

- Added `STAT1_OP_VARIANT_NAMES` const (26 entries in STAT_1.ops row order)
- `test_stat1_op_inventory_count`: inventory drift sentinel frozen at 26
- `test_every_stat1_rom_op_has_stat1_json_entry`: forward parity (Op → JSON)
- `test_every_stat1_json_entry_has_xrom_resolver_match`: reverse parity using `0b0000_0011` (v3.1 default)
- `test_pool_partition_is_exhaustive`: rogue module_id guard — fires for any future v3.2+ XROM module
- Updated `use` import to add `help_entries_stat1`
- All 11 tests green (4 v2.2 + 3 Math 1 + 4 new Stat 1)

**Task 2 verification:**
```
cargo test -p hp41-cli --test function_matrix_parity: 11 passed; 0 failed
```

### Task 3: phase25_xeq_by_name.rs — Stat 1 resolver extension (commit 4cb7bc5)

Extended `cli_resolver_matches_core_resolver` with Stat 1 verification block.

- 5 canonical Stat 1 mnemonics (ΣNORMD, ΣSPEAR, ΣBSTAT, RAND, SEED)
- Tested at 3 bitfield states: 0b0000_0011 (positive), 0b0000_0010 (bit-1 only, positive), 0b0000_0000 (negative)
- No keys.rs code change — STAT-CLI-01 is verification-only

**Task 3 verification:**
```
cargo test -p hp41-cli --test phase25_xeq_by_name -- cli_resolver_matches_core_resolver: 1 passed; 0 failed
Op::SigmaNormdWorkflow references: 1
Op::SigmaSpear references: 1
Op::SigmaBstat references: 1
Op::Rand references: 1
Op::Seed references: 1
```

### Task 4: phase34_modal_flow.rs — 5 Stat1Step modal-prompt tests (commit 8ba0519)

Created `hp41-cli/tests/phase34_modal_flow.rs` with 5 unit tests.

- Tests at carrier-enum level (`ModalProgram::Stat1(_)`), not inner function directly
- All 5 Stat1Step variants: OM-cited prompt verified + requires_alpha_label=false
- `PolypDegreePrompt` tested with degrees [0, 1, 3, 5, 99] — inner u8 is opaque to prompt
- Deviation: plan specified `usize` for PolypDegreePrompt inner value; actual type is `u8` (Rule 1 auto-fix)

**Task 4 verification:**
```
cargo test -p hp41-cli --test phase34_modal_flow: 5 passed; 0 failed
grep -c "^fn stat1_" hp41-cli/tests/phase34_modal_flow.rs: 5
grep -c "ModalProgram::Stat1" hp41-cli/tests/phase34_modal_flow.rs: 10 (at least 5)
grep -c "stat1::modal::current_prompt" hp41-cli/tests/phase34_modal_flow.rs: 0
grep -c "CalcState" hp41-cli/tests/phase34_modal_flow.rs: 0
grep -c "requires_alpha_label" hp41-cli/tests/phase34_modal_flow.rs: 5
```

### Task 5: phase34_key_ref_includes_stat1.rs — right-panel exclusion guards (commit 50d9b8c)

Created `hp41-cli/tests/phase34_key_ref_includes_stat1.rs` with 3 regression guards.

- Asserts Stat 1 entries EXCLUDED from key_ref_entries() (mirrors post-v3.0 Math 1 inversion)
- ΣNORMD sentinel (Σ-prefixed) excluded
- RAND sentinel (ASCII-named) excluded — confirms xrom-based filter, not display-name-pattern-based
- v2.2 Add op (+, +) still present as negative-regression guard
- Header comment explains CONTEXT.md wording inversion (post-v3.0 right-panel UX revert)

**Task 5 verification:**
```
cargo test -p hp41-cli --test phase34_key_ref_includes_stat1: 3 passed; 0 failed
cargo test -p hp41-cli --test phase29_key_ref_includes_math1: 3 passed; 0 failed (sibling still green)
```

### Task 6: Full CI verification gate (commit c2f42d4 — also includes Rule 1 fix for key_coverage.rs)

**Task 6 verification:**
```
just ci: exit code 0
cargo test -p hp41-cli --tests: 345 passed (16 suites)
cargo check -p hp41-cli: exit code 0
cargo build -p hp41-cli --tests --no-run 2>&1 | grep -c "non-exhaustive patterns": 0
cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml 2>&1 | grep -c "non-exhaustive patterns": 1 (Phase 36 scope)
Coverage: 95.42% lines / 97.27% regions (above 95% gate)
Free42 contamination: OK: no contamination detected
```

## Phase 34 Sealed

| Requirement | Status | Evidence |
|-------------|--------|---------|
| STAT-CLI-01: xeq_by_name_local_resolve routes Stat 1 names | SEALED | Task 3 verification block; 3 bitfield states tested |
| STAT-CLI-02: help_entries_stat1() + three-pool chain | SEALED | Shipped in Plan 34-01; deferred tests now green |
| STAT-CLI-03: 26 op_display_name arms — non-exhaustive break closed | SEALED | Task 1; cargo check exits 0; 26 arms present |
| STAT-CLI-04 (a): function_matrix_parity 3-pool | SEALED | Task 2; 11 tests green |
| STAT-CLI-04 (b): right-panel exclusion | SEALED | Task 5; 3 guards green |
| STAT-CLI-05: modal-prompt routing | SEALED | Task 4; 5 Stat1Step tests green |
| 4-way invariant item 3 | SEALED | prgm_display.rs exhaustive; item 4 (GUI) deferred to Phase 36 |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed key_coverage.rs Stat 1 resolver failure**

- **Found during:** Task 6 CI verification
- **Issue:** `key_coverage_implemented_entries_dispatch` panicked when the main loop's `XeqByName` branch called `xeq_by_name_local_resolve(&name, 0b0000_0001)` for Stat 1 entries added to `help_entries_all()` by Plan 34-01. Stat 1 needs bit 1 (not bit 0) set in `xrom_modules`.
- **Fix:** Changed the main loop XeqByName bitfield from `0b0000_0001` to `0b0000_0011` (v3.1 default — both Math 1 + Stat 1 loaded). Added a Stat 1 sub-loop (parallel to BL-04 Math1 sub-loop) with `xrom.module_id == 2` filter. Updated floor assertion from 95 to 120 entries.
- **Files modified:** `hp41-cli/tests/key_coverage.rs`
- **Commit:** c2f42d4

**2. [Rule 1 - Bug] Fixed PolypDegreePrompt inner type in phase34_modal_flow.rs**

- **Found during:** Task 4 — compile error
- **Issue:** Plan specified `usize` for `Stat1Step::PolypDegreePrompt` inner value; actual type in `hp41-core/src/ops/stat1/modal.rs` is `u8`.
- **Fix:** Changed `[0usize, 1, 3, 5, 99]` to `[0u8, 1, 3, 5, 99]` and updated the doc-comment.
- **Files modified:** `hp41-cli/tests/phase34_modal_flow.rs`
- **Commit:** 8ba0519 (in-task fix, same commit)

## Threat Flags

No new security-relevant surface introduced. All changes are test-only or `op_display_name` display-rendering arms (no dispatch logic, no trust-boundary crossings).

## Self-Check

### Created files exist:
- `hp41-cli/tests/phase34_modal_flow.rs`: FOUND
- `hp41-cli/tests/phase34_key_ref_includes_stat1.rs`: FOUND

### Modified files exist:
- `hp41-cli/src/prgm_display.rs`: FOUND (37 lines added)
- `hp41-cli/tests/function_matrix_parity.rs`: FOUND (145 lines added)
- `hp41-cli/tests/phase25_xeq_by_name.rs`: FOUND (42 lines added)
- `hp41-cli/tests/key_coverage.rs`: FOUND (60 lines added)

### Commits exist:
- `6addc88`: feat(34-02): add 26 op_display_name arms — FOUND
- `19a51f6`: test(34-02): extend function_matrix_parity to 3-pool partition — FOUND
- `4cb7bc5`: test(34-02): extend cli_resolver_matches_core_resolver with Stat 1 cases — FOUND
- `8ba0519`: test(34-02): add phase34_modal_flow.rs — FOUND
- `50d9b8c`: test(34-02): add phase34_key_ref_includes_stat1.rs — FOUND
- `c2f42d4`: fix(34-02): extend key_coverage.rs for Stat 1 Pac — FOUND

## Self-Check: PASSED
