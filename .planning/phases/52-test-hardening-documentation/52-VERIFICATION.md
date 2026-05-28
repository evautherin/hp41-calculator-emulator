---
phase: 52-test-hardening-documentation
verified: 2026-05-28T00:00:00Z
status: passed
score: 3/3 must-haves verified
overrides_applied: 0
---

# Phase 52: Test Hardening + Documentation Verification Report

**Phase Goal:** Extended Memory is production-ready — backward-compatible, isolated, and fully integrated across CLI, GUI, and test suite
**Verified:** 2026-05-28
**Status:** PASSED
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | v3.3 save file loads without error; `xmem_files` is empty, `xmem_active_file` is None | VERIFIED | `hp41-core/tests/fixtures/v33-autosave.json` has `xrom_modules:31`, no `xmem_files`/`xmem_active_file` fields; `xmem_backward_compat.rs::v33_save_loads_without_error` asserts both; 19 tests pass |
| 2 | X-MEM ops never read from or write to `state.regs` or `adv_matrices` (except SAVED/GETD) | VERIFIED | Three isolation tests in `xmem_backward_compat.rs`: `xmem_ops_never_touch_adv_matrices` (SAVEP/EMDIR/EMROOM leave `adv_matrices` empty), `savep_getp_do_not_touch_state_regs` (SAVEP/GETP do not alter `regs[0]`), `saved_getd_transfer_is_isolated` (SAVED/GETD sanctioned transfer round-trips `regs[5]`); all pass |
| 3 | All 8 X-MEM ops are reachable from CLI + GUI + help overlay; resolved via `builtin_card_op` NOT `xrom_resolve`, NO XROM bit | VERIFIED | 8 arms in `builtin_card_op()` at `program.rs:1447–1455` (observed directly); `function_matrix_parity.rs::test_every_xmem_json_entry_has_builtin_resolver_match` calls `builtin_card_op()` for reverse parity (26/26 pass); `help_entries_xmem()` wired in CLI `help_data.rs:216` and GUI `help_data.ts:234`; `phase52_help_data_xmem.rs::help_overlay_rows_includes_xmem_section` verifies "Extended Memory" header appears; `xeq_builtin_resolver.rs` extended with all 8 mnemonics |

**Score:** 3/3 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `docs/hp41-xmem-functions.json` | 8-entry X-MEM help pool, no `xrom` field | VERIFIED | 8 entries, `xrom` absent on all, `category:"Extended Memory"`, all have `key_path` XEQ-form, all have `example`+`notes` |
| `hp41-core/tests/fixtures/v33-autosave.json` | v3.3 fixture, `xrom_modules:31`, no xmem fields | VERIFIED | `xrom_modules:31`, no `xmem_files`, no `xmem_active_file`, `rand_seed:"0.5"`, 100 regs |
| `hp41-core/tests/xmem_backward_compat.rs` | Backward-compat + isolation + supplementary per-op tests | VERIFIED | 19 tests in 3 groups: 4 backward-compat (XMEM-08), 3 isolation (XMEM-09), 12 supplementary (D-52.13 floor) |
| `hp41-core/src/ops/program.rs` | 8 X-MEM arms in `builtin_card_op()` | VERIFIED | Lines 1447–1455: `"EMDIR"=>Some(Op::EmDir)` through `"SAVERX"=>Some(Op::SaveRx)`, comment `Phase 52 (v4.0): X-MEM built-in ops` |
| `hp41-cli/src/help_data.rs` | Sixth OnceLock pool `help_entries_xmem()` + chain extension | VERIFIED | `XMEM_FUNCTIONS_JSON` const + `XMEM_HELP_ENTRIES` OnceLock at lines 202–221; chained sixth in `help_entries_all()` at line 249 |
| `hp41-gui/src/help_data.ts` | `helpEntriesXmem()` + `helpEntriesAll()` spread | VERIFIED | `import xmemFunctions` at line 33, `helpEntriesXmem()` at line 234, spread sixth in `helpEntriesAll()` at line 247 |
| `hp41-gui/src/HelpOverlay.test.tsx` | Six-pool length assertion + X-MEM loop + import | VERIFIED | `helpEntriesXmem` added to import (line 37); "all 6 pools" test (line 68) sums all 6 lengths; Advantage loop bounded to `advStart+advCount`; X-MEM loop (lines 99–103) asserts `xrom===undefined` + `category==='Extended Memory'` |
| `hp41-cli/tests/phase52_help_data_xmem.rs` | Pool smoke test: 8-count + six-pool chain + overlay section | VERIFIED | 6 tests, all pass: count==8, chain>=358, overlay header, category+key_path invariants, op_variant drift-catch |
| `hp41-cli/tests/function_matrix_parity.rs` | `XMEM_OP_VARIANT_NAMES` + 3 X-MEM parity tests | VERIFIED | Lines 1053–1143: inventory sentinel (len==8), forward parity (every variant in JSON), reverse parity (every JSON entry resolves via `builtin_card_op`); 26/26 tests pass |
| `hp41-core/tests/xrom_op_test_count.rs` | `collect_xmem_variant_names()` + per-op >=5 floor | VERIFIED | Lines 116–135: hardcoded 8-name list; `count_xmem_test_mentions` dual-scan (external `xmem_*.rs` + inline `ops.rs`); floor block inside `each_xrom_op_has_at_least_5_tests` at lines 713–741; 21/21 tests pass |
| `docs/hp41-xmem-function-matrix.md` | Generated matrix doc (not hand-authored) | VERIFIED | File exists; `just docs-matrix-check` passes (no diff) |
| `docs/adr/v4.0-001-xmem-os-builtin.md` | ADR: X-MEM as OS built-ins, no XROM bit | VERIFIED | Exists (9.9K), contains `ADR-v4.0-001`, proper Status/Owner/Requirement refs/Downstream consumer header |
| `docs/adr/v4.0-002-xmem-capacity.md` | ADR: fixed 600-register capacity | VERIFIED | Exists (8.3K), documents 600 = 124 + 2×238, EMROOM semantics, alternatives considered |
| `docs/adr/v4.0-003-xmem-register-transfer.md` | ADR: full-register-set SAVED/GETD | VERIFIED | Exists (10.0K), documents `SAVED`/`GETD` behavior, bbb.eee deferral, upgrade path |
| `docs/hp41-xmem-divergences.md` | Two divergences: overwrite-on-duplicate + full-reg-set | VERIFIED | Exists (9.5K), contains "DUP FL" (divergence D-52-01) and "bbb.eee"/"block control" (divergence D-52-02) |
| `README.md` | Scoped/honest X-MEM claim, no "feature-complete" for X-MEM | VERIFIED | Line 64: "v4.0 adds Extended Memory: named PROGRAM + DATA file storage (HP-41CX X-Functions)"; all "feature-complete" occurrences reference prior modules only |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `hp41-cli/src/help_data.rs` | `docs/hp41-xmem-functions.json` | `include_str!` + OnceLock | WIRED | Line 202: `const XMEM_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-xmem-functions.json")` |
| `hp41-gui/src/help_data.ts` | `docs/hp41-xmem-functions.json` | Vite static JSON import | WIRED | Line 33: `import xmemFunctions from '../../docs/hp41-xmem-functions.json'` |
| `hp41-gui/src/HelpOverlay.test.tsx` | `hp41-gui/src/help_data.ts` | `helpEntriesXmem()` import + assertion | WIRED | Line 37: import includes `helpEntriesXmem`; line 71 sums it in length assertion |
| `hp41-cli/tests/function_matrix_parity.rs` | `hp41_core::ops::program::builtin_card_op` | Reverse parity test (NOT xrom_resolve) | WIRED | Line 1131: `hp41_core::ops::program::builtin_card_op(entry.display_name.as_str())` |
| `hp41-cli/tests/function_matrix_parity.rs` | `help_entries_xmem` | Forward parity test | WIRED | Line 1102: `help_entries_xmem().iter()...` |
| `hp41-core/tests/xmem_backward_compat.rs` | `hp41-core/tests/fixtures/v33-autosave.json` | `include_str!` fixture | WIRED | Line 41: `static V33_FIXTURE: &str = include_str!("fixtures/v33-autosave.json")` |
| `hp41-core/src/ops/program.rs` | `Op::EmDir..Op::SaveRx` | `builtin_card_op` match arms | WIRED | Lines 1447–1455: 8 arms returning `Some(Op::EmDir)` through `Some(Op::SaveRx)` |

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `builtin_card_op` resolves all 8 X-MEM mnemonics | `cargo test -p hp41-cli --test function_matrix_parity` | 26/26 pass | PASS |
| v3.3 fixture backward-compat + isolation | `cargo test -p hp41-core --test xmem_backward_compat` | 19/19 pass | PASS |
| Per-op >=5 mention floor for all 8 X-MEM ops | `cargo test -p hp41-core --test xrom_op_test_count` | 21/21 pass | PASS |
| Six-pool chain + overlay section | `cargo test -p hp41-cli --test phase52_help_data_xmem` | 6/6 pass | PASS |
| JSON<->matrix doc no drift | `just docs-matrix-check` | 0 diff output | PASS |
| Core suite green | `just test-core` | 0 failures | PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|---------|
| XMEM-08 | Plan 02 | X-MEM state persists across save/load with `#[serde(default)]` backward compat | SATISFIED | v33 fixture has no xmem fields; `v33_save_loads_without_error` + `v33_save_xmem_fields_default_cleanly` pass; `migrate_after_load()` idempotency confirmed |
| XMEM-09 | Plan 02 | X-MEM storage isolated from main registers and `adv_matrices` | SATISFIED | Three isolation tests pass: no adv_matrices mutation, no regs mutation outside SAVED/GETD, sanctioned SAVED/GETD round-trip |
| XMEM-10 | Plans 01, 02, 03 | CLI + GUI integration (4-way exhaustive match, op_display_name, help overlay) | SATISFIED | 8 builtin_card_op arms; CLI+GUI help pools wired; overlay section present; bidirectional parity gate + >=5 floor gate both pass |

All three requirements declared by Phase 52 plans are SATISFIED. No orphaned requirements found.

---

### Anti-Patterns Found

No TBD, FIXME, or XXX markers found in any file modified by this phase.

One documentation accuracy concern (WARNING, not blocker):

| File | Field | Issue | Severity |
|------|-------|-------|---------|
| `docs/hp41-xmem-functions.json` — EMREG entry | `description`, `example`, `notes` | Description says "Return number of registers used by X-MEM file named by ALPHA" and example shows a register-count result. Actual `op_emreg` recalls register N (N from X) from the **active** file — not from ALPHA-named file, not a count. `EMROOM` returns a count; `EMREG` returns a register value. The parity tests, resolver, and implementation are all correct; only the human-readable description in the JSON pool is inaccurate. | WARNING — wrong description shown to users in `?` overlay; no functional or test impact |

---

### Human Verification Required

1. **EMREG help-overlay description accuracy**

   **Test:** Open the `?` overlay in CLI or GUI and view the "Extended Memory" section. Compare the EMREG description against the HP-41CX OM behavior.
   **Expected:** EMREG description should read approximately "Recall register N (from X) of the active X-MEM DATA file → push to X" — not a register-count function.
   **Why human:** The description in `docs/hp41-xmem-functions.json` (the `description` and `example` fields for `op_variant: "EmReg"`) incorrectly describes EMREG as a register-count function (like EMROOM). The implementation `op_emreg` is correct. Only the user-visible documentation is wrong. Fixing requires updating the JSON entry — a one-line change to `description`, `example`, and `notes`. The verifier flags this as a documentation accuracy issue but cannot determine user-visible severity without seeing the rendered overlay.

---

### Gaps Summary

No gaps blocking the phase goal. The EMREG description inaccuracy is a documentation quality issue: the implementation, all tests, the resolver wiring, the parity gates, and the overlay sectioning are correct. Only the human-readable description fields (`description`, `example`, `notes`) for the EMREG JSON entry misrepresent what the function does. This does not prevent Extended Memory from being production-ready per the phase goal definition.

---

## Summary

All three success criteria are met:

1. **Backward-compat (XMEM-08):** The v33-autosave.json fixture genuinely lacks `xmem_files`/`xmem_active_file`; `#[serde(default)]` is the migration path; `xmem_backward_compat.rs` proves it with 4 dedicated tests including idempotency + rand_seed guard.

2. **Isolation (XMEM-09):** Three isolation tests verify that SAVEP/EMDIR/EMROOM never touch `adv_matrices`, SAVEP/GETP never mutate `state.regs`, and SAVED/GETD is the only sanctioned register-transfer path.

3. **CLI + GUI integration (XMEM-10):** 8 arms in `builtin_card_op()` (not `xrom_resolve`, no XROM bit); sixth help pool in CLI `OnceLock` and GUI Vite import; "Extended Memory" section in `?` overlay; bidirectional op↔JSON parity gate via `builtin_card_op`; per-op >=5 mention floor gate; 26-test parity suite + 21-test floor suite both green; `just docs-matrix-check` passes.

The one open item (EMREG description inaccuracy in the JSON documentation fields) is flagged for human review and does not block the phase goal.

---

_Verified: 2026-05-28_
_Verifier: Claude (gsd-verifier)_
