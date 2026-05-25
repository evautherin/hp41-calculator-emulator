---
phase: 38
slug: hp41-core-xrom-framework-clock-date-stopwatch-alarm-core
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-05-24
---

# Phase 38 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test + cargo-llvm-cov |
| **Config file** | `Cargo.toml` (workspace) |
| **Quick run command** | `cargo test -p hp41-core 2>&1 | tail -5` |
| **Full suite command** | `just test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p hp41-core 2>&1 | tail -5`
- **After every plan wave:** Run `just test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 38-01-01 | 01 | 1 | TIME-FW-01, TIME-FW-02, TIME-FW-03, TIME-FW-04, TIME-FW-05, TIME-FW-06 | unit + integration | `cargo test -p hp41-core xrom_time` | ❌ W0 | ⬜ pending |
| 38-01-02 | 01 | 1 | TIME-FW-01 | unit | `cargo test -p hp41-core time_resolve` | ❌ W0 | ⬜ pending |
| 38-01-03 | 01 | 1 | TIME-FW-03, TIME-FW-04 | integration | `cargo test -p hp41-core time_serde_round_trip` | ❌ W0 | ⬜ pending |
| 38-02-01 | 02 | 2 | TIME-DAT-01, TIME-DAT-02, TIME-DAT-03, TIME-DAT-04, TIME-DAT-05, TIME-DAT-06 | unit | `cargo test -p hp41-core time_date_arith` | ❌ W1 | ⬜ pending |
| 38-02-02 | 02 | 2 | TIME-DAT-06 | unit | `cargo test -p hp41-core parse_date_edge_cases` | ❌ W1 | ⬜ pending |
| 38-03-01 | 03 | 2 | TIME-CLK-01, TIME-CLK-02, TIME-CLK-03, TIME-CLK-04, TIME-CLK-05, TIME-CLK-06 | unit | `cargo test -p hp41-core time_clock_ops` | ❌ W2 | ⬜ pending |
| 38-03-02 | 03 | 2 | TIME-DSP-01, TIME-DSP-02, TIME-DSP-03, TIME-DSP-04, TIME-FMT-01, TIME-FMT-02, TIME-FMT-03, TIME-FMT-04, TIME-FMT-05 | unit | `cargo test -p hp41-core time_format` | ❌ W2 | ⬜ pending |
| 38-04-01 | 04 | 2 | TIME-CLK-03, TIME-CLK-04, TIME-CLK-05 | unit | `cargo test -p hp41-core time_alpha` | ❌ W2 | ⬜ pending |
| 38-04-02 | 04 | 2 | TIME-SW-01, TIME-SW-02, TIME-SW-03, TIME-SW-04, TIME-SW-05, TIME-SW-06, TIME-SW-07, TIME-SW-08, TIME-SW-09 | unit | `cargo test -p hp41-core time_stopwatch` | ❌ W3 | ⬜ pending |
| 38-05-01 | 05 | 3 | TIME-ALM-01, TIME-ALM-02, TIME-ALM-03, TIME-ALM-04, TIME-ALM-05, TIME-ALM-06, TIME-ALM-07, TIME-ALM-08, TIME-ALM-09, TIME-ALM-10, TIME-ALM-11, TIME-ALM-12 | unit | `cargo test -p hp41-core time_alarm` | ❌ W4 | ⬜ pending |
| 38-05-02 | 05 | 3 | TIME-ALM-09, TIME-ALM-10 | unit | `cargo test -p hp41-core check_alarms` | ❌ W4 | ⬜ pending |
| 38-06-01 | 06 | 3 | TIME-DSP-05, TIME-SW-08 | unit | `cargo test -p hp41-core time_modal` | ❌ W4 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-core/tests/time_xrom_registration.rs` — stubs for TIME-FW-01, TIME-FW-02 (xrom_resolve bit-2, TIME_MODULE const fields, default + migration)
- [ ] `hp41-core/tests/time_serde_compat.rs` — stubs for TIME-FW-03, TIME-FW-04 (new field serialization, v3.1 fixture load)
- [ ] `hp41-core/tests/fixtures/v31-autosave.json` — v3.1 save fixture for backward compat test

*Existing infrastructure (cargo test, cargo-llvm-cov, just test) covers all framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Clock display refresh ≥1 Hz | TIME-DSP-05 | Frontend cadence, not testable in core | Phase 39 CLI / Phase 41 GUI — visually confirm tick |
| Stopwatch display refresh ≥10 Hz | TIME-SW-08 | Frontend cadence, not testable in core | Phase 39 CLI / Phase 41 GUI — visually confirm update rate |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 15s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
