---
phase: 51
slug: x-mem-core
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-28
---

# Phase 51 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Seeded from `51-RESEARCH.md` § Validation Architecture. Task IDs are assigned by the planner; wire each test below into the matching task's `<automated>` verify.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `proptest` (already in workspace — zero new deps) |
| **Config file** | none — `hp41-core/Cargo.toml` (workspace test config) |
| **Quick run command** | `just test-core xmem` |
| **Full suite command** | `just test-core` |
| **Estimated runtime** | ~15 seconds (core suite) |

---

## Sampling Rate

- **After every task commit:** Run `just test-core xmem`
- **After every plan wave:** Run `just test-core`
- **Before `/gsd:verify-work`:** Full suite (`just test`) must be green
- **Max feedback latency:** ~15 seconds

---

## Per-Task Verification Map

> Task IDs are TBD until the planner emits PLAN.md files. Each behavior below MUST be wired into the matching task's `<automated>` verify. Filter form: `just test-core <filter>`.

| Behavior | Requirement | Test Type | Automated Command | File Exists | Status |
|----------|-------------|-----------|-------------------|-------------|--------|
| EMDIR prints file list (name/type/size) to print_buffer | XMEM-01 | unit | `just test-core xmem::ops::tests::emdir` | ❌ W0 | ⬜ pending |
| EMROOM returns 600 for empty store, decreases as files accumulate | XMEM-02 | unit | `just test-core xmem::ops::tests::emroom` | ❌ W0 | ⬜ pending |
| SAVEP encodes program, stores in xmem_files, register_count correct | XMEM-03 | unit | `just test-core xmem::ops::tests::savep_round_trip` | ❌ W0 | ⬜ pending |
| GETP retrieves + inserts program via insert_program_ops | XMEM-04 | unit | `just test-core xmem::ops::tests::getp_round_trip` | ❌ W0 | ⬜ pending |
| SAVED captures regs, stores file, register_count = N+1 | XMEM-05 | unit | `just test-core xmem::ops::tests::saved_round_trip` | ❌ W0 | ⬜ pending |
| GETD restores regs via load_data_card, sets active file | XMEM-06 | unit | `just test-core xmem::ops::tests::getd_round_trip` | ❌ W0 | ⬜ pending |
| EMREG recalls register N from active DATA file; SAVERX stores Y into N | XMEM-07 | unit | `just test-core xmem::ops::tests::emreg_saverx` | ❌ W0 | ⬜ pending |
| SAVEP returns NoRoom when capacity exceeded | XMEM-03 | unit | `just test-core xmem::ops::tests::savep_no_room` | ❌ W0 | ⬜ pending |
| GETP returns FileNotFound for missing name | XMEM-04 | unit | `just test-core xmem::ops::tests::getp_not_found` | ❌ W0 | ⬜ pending |
| GETP returns FileType when name is a DATA file | XMEM-04 | unit | `just test-core xmem::ops::tests::getp_type_mismatch` | ❌ W0 | ⬜ pending |
| GETD returns FileType when name is a PROGRAM file | XMEM-06 | unit | `just test-core xmem::ops::tests::getd_type_mismatch` | ❌ W0 | ⬜ pending |
| EMREG returns FileNotFound when no active file set | XMEM-07 | unit | `just test-core xmem::ops::tests::emreg_no_active` | ❌ W0 | ⬜ pending |
| EMREG returns OutOfRange for N >= reg_count | XMEM-07 | unit | `just test-core xmem::ops::tests::emreg_out_of_range` | ❌ W0 | ⬜ pending |
| Duplicate SAVEP/SAVED overwrites in place; xmem_files.len() unchanged (D-51.6) | XMEM-03/05 | unit | `just test-core xmem::ops::tests::overwrite_in_place` | ❌ W0 | ⬜ pending |
| EMROOM at exactly capacity = 0; one register over = NoRoom (boundary) | XMEM-02 | unit | `just test-core xmem::ops::tests::emroom_capacity_boundary` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-core/src/ops/xmem/mod.rs` — `XmemFile` struct, `XmemKind` enum, `register_count()`, `XMEM_CAPACITY` (600), stubs for XMEM-01..07
- [ ] `hp41-core/src/ops/xmem/ops.rs` (or equivalent) — the 8 op functions + unit-test module
- [ ] `hp41-core/src/error.rs` — new `HpError` variants: `FileNotFound`, `FileType`, `NoRoom`
- [ ] `hp41-core/src/state.rs` — `xmem_files` + `xmem_active_file` fields (`#[serde(default)]`, NO `#[serde(skip)]`) + `CalcState::new()` init

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| — | — | — | All Phase 51 behaviors have automated unit verification (hp41-core-only phase; no UI/IPC until Phase 52). |

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
