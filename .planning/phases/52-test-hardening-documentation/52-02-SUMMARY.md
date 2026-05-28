---
phase: 52-test-hardening-documentation
plan: "02"
subsystem: hp41-core / hp41-cli
tags: [xmem, builtin_card_op, resolver, backward-compat, isolation, test]
dependency_graph:
  requires: [51-05]
  provides: [XMEM-08, XMEM-09, XMEM-10-resolution]
  affects: [hp41-core/src/ops/program.rs, hp41-core/tests/, hp41-cli/tests/]
tech_stack:
  added: []
  patterns:
    - "builtin_card_op match arm extension (single-point X-MEM resolution)"
    - "#[serde(default)] backward-compat fixture test pattern"
    - "Isolation test pattern (D-43.5 replicated for X-MEM)"
key_files:
  created:
    - hp41-core/tests/fixtures/v33-autosave.json
    - hp41-core/tests/xmem_backward_compat.rs
  modified:
    - hp41-core/src/ops/program.rs
    - hp41-cli/tests/xeq_builtin_resolver.rs
decisions:
  - "8 X-MEM arms added to builtin_card_op() only — zero changes to keys.rs or key_map.rs (D-52.4 single-point principle)"
  - "HpValue (not HpNum) for state.regs indexing in tests — type fix via Rule 1 auto-fix"
  - "CalcState::new() used instead of CalcState::default() — clippy field_reassign_with_default compliance"
metrics:
  duration: "~15 minutes"
  completed: "2026-05-28"
  tasks_completed: 3
  tasks_total: 3
  files_created: 3
  files_modified: 2
---

# Phase 52 Plan 02: X-MEM Resolver Wiring + Backward-Compat Tests Summary

**One-liner:** 8 X-MEM builtin_card_op arms unlock CLI/GUI/programmatic XEQ; v33 fixture + isolation test suite prove backward-compat and D-51.0a invariants.

---

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Add 8 X-MEM arms to builtin_card_op + resolver test | db08bf6 | hp41-core/src/ops/program.rs, hp41-cli/tests/xeq_builtin_resolver.rs |
| 2 | Create v33-autosave.json backward-compat fixture | 93db565 | hp41-core/tests/fixtures/v33-autosave.json |
| 3 | Create xmem_backward_compat.rs — backward-compat + isolation + supplementary tests | 0d7139d | hp41-core/tests/xmem_backward_compat.rs |

---

## What Was Built

### Task 1: builtin_card_op resolver extension (D-52.4)

Added 8 match arms to `hp41-core/src/ops/program.rs::builtin_card_op()` immediately before `_ => None`:

```rust
// Phase 52 (v4.0): X-MEM built-in ops (HP-41CX Extended Functions / D-52.4)
"EMDIR"  => Some(Op::EmDir),
"EMROOM" => Some(Op::EmRoom),
"SAVEP"  => Some(Op::SaveP),
"GETP"   => Some(Op::GetP),
"SAVED"  => Some(Op::SaveD),
"GETD"   => Some(Op::GetD),
"EMREG"  => Some(Op::EmReg),
"SAVERX" => Some(Op::SaveRx),
```

This single-point change simultaneously unlocks:
- CLI XEQ-by-name via `xeq_by_name_local_resolve` (keys.rs:369 — unchanged)
- GUI XEQ-by-name via the existing `xeq_<name>` strip_prefix arm (key_map.rs:427 — unchanged)
- Programmatic XEQ from `run_loop`

Extended `xeq_builtin_resolver.rs` with `xmem_builtins_resolve` and `xmem_unknown_still_none` tests. Both pass. The pre-existing `key_coverage` test that was already asserting X-MEM resolution now also passes.

### Task 2: v33-autosave.json fixture (D-52.10)

Created `hp41-core/tests/fixtures/v33-autosave.json` — exact copy of `v32-autosave.json` with one change: `xrom_modules: 7` → `31` (0b0001_1111 = v3.3 value after Advantage Pac migration). Intentionally omits `xmem_files` and `xmem_active_file` to exercise the `#[serde(default)]` paths (XMEM-08 requirement).

### Task 3: xmem_backward_compat.rs (D-52.10 / D-52.12 / XMEM-08 / XMEM-09)

Created 15-test file with three groups:

**Backward-compat (4 tests):**
- `v33_save_loads_without_error` — fixture deserializes; xmem_files empty, xmem_active_file None
- `v33_save_xmem_fields_default_cleanly` — fields remain at default post-migrate
- `v33_save_migrate_after_load_is_idempotent` — xrom_modules stays 0b0001_1111, no v4.0 arm (D-52.12 confirmed)
- `v33_save_rand_seed_preserved` — 0.5 survives migration (Pitfall 20 guard)

**Isolation (3 tests):**
- `xmem_ops_never_touch_adv_matrices` — SAVEP/EMDIR/EMROOM leave adv_matrices empty
- `savep_getp_do_not_touch_state_regs` — SAVEP/GETP round-trip; state.regs[0] unchanged
- `saved_getd_transfer_is_isolated` — sanctioned SAVED/GETD transfer round-trips reg[5]=99

**Supplementary per-op coverage (8 tests):**
- EmDir: 2 tests (empty store + populated catalog)
- EmRoom: 2 tests (empty returns 600 + decreases after save)
- GetD: 2 tests (active-file side effect + missing file error)
- SaveRx: 2 tests (stores into active file + does not grow xmem_files)

Combined with ops.rs inline tests, all four ops reach ≥5 mentions (D-52.13 floor prerequisite satisfied for Plan 03).

---

## Verification Results

| Check | Result |
|-------|--------|
| `just test` | All suites pass (0 failures) |
| `just lint` | Clean (no warnings or errors) |
| `cargo test -p hp41-core --test xmem_backward_compat` | 15/15 pass |
| `builtin_card_op("EMDIR")` == `Some(Op::EmDir)` | Verified |
| All 8 mnemonics resolve | Verified (xeq_builtin_resolver.rs + key_coverage.rs) |
| keys.rs / key_map.rs untouched | Verified (git diff empty) |
| v33 fixture: xrom_modules==31, no xmem fields | Verified (python3 assertion) |

---

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] HpValue vs HpNum type mismatch in test file**
- **Found during:** Task 3 first compile
- **Issue:** `state.regs[i]` is `Vec<HpValue>` but tests initially used `HpNum::from(i32)` for assignment and comparison
- **Fix:** Added `HpValue` to imports; replaced `HpNum::from(42i32)` with `HpValue::from(42i32)` for regs assignments and comparisons
- **Files modified:** hp41-core/tests/xmem_backward_compat.rs
- **Commit:** Inline fix before Task 3 commit

**2. [Rule 1 - Bug] clippy field_reassign_with_default triggered by CalcState::default()**
- **Found during:** Task 3 lint run
- **Issue:** `let mut state = CalcState::default(); state.alpha_reg = "..."` triggers `field_reassign_with_default` clippy lint (-D warnings)
- **Fix:** Replaced all `CalcState::default()` with `CalcState::new()` in xmem_backward_compat.rs — the existing ops.rs inline tests already use `CalcState::new()` for this reason
- **Files modified:** hp41-core/tests/xmem_backward_compat.rs
- **Commit:** Inline fix before Task 3 commit

---

## Success Criteria Verification

| Criterion | Status |
|-----------|--------|
| All 8 X-MEM mnemonics resolve via builtin_card_op | PASS |
| CLI + GUI + programmatic XEQ reachable (D-52.4) | PASS |
| v3.3 save loads; xmem_files empty; xmem_active_file None (XMEM-08) | PASS |
| Isolation proven: no adv_matrices / state.regs mutation outside SAVED/GETD (XMEM-09) | PASS |
| migrate_after_load needs no new arm (D-52.12) | PASS (idempotency test confirms) |
| EmDir/EmRoom/GetD/SaveRx reach ≥5 mentions (D-52.13 floor prerequisite) | PASS |

---

## Threat Flags

None. This plan adds Rust match arms, a JSON fixture, and a Rust test file — no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries.

## Known Stubs

None — all behavior is fully implemented. No placeholder or TODO values in created/modified files.

## Self-Check: PASSED

- hp41-core/src/ops/program.rs: FOUND (8 X-MEM arms present)
- hp41-core/tests/fixtures/v33-autosave.json: FOUND
- hp41-core/tests/xmem_backward_compat.rs: FOUND (15 tests)
- hp41-cli/tests/xeq_builtin_resolver.rs: FOUND (extended with X-MEM tests)
- Commits db08bf6, 93db565, 0d7139d: all confirmed in git log
