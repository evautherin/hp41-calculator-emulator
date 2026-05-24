---
phase: 37
slug: test-hardening-quality-gates
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-24
---

# Phase 37 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + cargo-llvm-cov (coverage) + WebdriverIO 9 (E2E) |
| **Config file** | `justfile` (task runner) / `hp41-gui/wdio.conf.cjs` (E2E) |
| **Quick run command** | `just test` |
| **Full suite command** | `just ci` + `just gui-ci` |
| **Estimated runtime** | ~120 seconds (ci) + ~60 seconds (gui-ci) |

---

## Sampling Rate

- **After every task commit:** Run `just test`
- **After every plan wave:** Run `just ci`
- **Before `/gsd:verify-work`:** `just ci` + `just gui-ci` must be green
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 37-01-01 | 01 | 1 | STAT-QUAL-07 | — | N/A | meta-gate | `cargo test --test stat1_op_test_count` | ❌ W0 | ⬜ pending |
| 37-01-02 | 01 | 1 | STAT-QUAL-06 | — | N/A | lint | `cargo test --test lint_stat1_assertions` | ❌ W0 | ⬜ pending |
| 37-01-03 | 01 | 1 | STAT-QUAL-08 | — | N/A | integration | `cargo test --test xrom_shadowing` | ✅ | ⬜ pending |
| 37-02-01 | 02+ | 2 | STAT-QUAL-01,02,03 | — | N/A | coverage | `just coverage` | ✅ | ⬜ pending |
| 37-03-01 | 03+ | 2 | STAT-QUAL-10 | — | N/A | integration | `cargo test --test stat1_backward_compat` | ❌ W0 | ⬜ pending |
| 37-04-01 | 04 | 3 | STAT-QUAL-04,05 | — | N/A | accuracy | `cargo test --test numerical_accuracy` | ✅ | ⬜ pending |
| 37-05-01 | 05 | 4 | STAT-QUAL-11 | — | N/A | E2E | `npx wdio run wdio.conf.cjs` | ✅ | ⬜ pending |
| 37-05-02 | 05 | 4 | STAT-GUI-05 | — | N/A | doc | VERIFICATION.md attestation | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-core/tests/stat1_op_test_count.rs` — meta-gate for STAT-QUAL-07
- [ ] `hp41-core/tests/lint_stat1_assertions.rs` — assertion lint for STAT-QUAL-06
- [ ] `hp41-core/tests/stat1_backward_compat.rs` — migration test for STAT-QUAL-10
- [ ] `hp41-core/tests/fixtures/v30-autosave.json` — v3.0 save-file fixture

*Existing infrastructure covers the remainder — `numerical_accuracy.rs`, `xrom_shadowing.rs`, `smoke.spec.js` are extended in-place.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Per-file stat1/*.rs coverage ≥ 90% | STAT-QUAL-03 | Coverage is measured locally, not CI-gated | Run `just coverage`, verify per-file output |
| Aggregate hp41-core coverage ≥ 95.39% | STAT-QUAL-01,02 | Coverage is measured locally, not CI-gated | Run `just coverage`, verify aggregate output |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
