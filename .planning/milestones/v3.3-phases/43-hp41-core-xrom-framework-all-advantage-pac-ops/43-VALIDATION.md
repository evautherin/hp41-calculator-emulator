---
phase: 43
slug: hp41-core-xrom-framework-all-advantage-pac-ops
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-25
---

# Phase 43 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`#[cfg(test)]` + `cargo test`) |
| **Config file** | `hp41-core/Cargo.toml` (test deps: proptest, criterion) |
| **Quick run command** | `just test-core` |
| **Full suite command** | `just ci` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `just test-core`
- **After every plan wave:** Run `just ci`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 43-01-01 | 01 | 1 | ADV-FW-01..03 | — | N/A | unit | `just test-core` | ✅ inline | ⬜ pending |
| 43-01-02 | 01 | 1 | ADV-FW-04..06 | — | N/A | unit | `just test-core` | ✅ inline | ⬜ pending |
| 43-02-01 | 02 | 2 | ADV-CONV-01..12 | — | N/A | unit | `just test-core` | ✅ inline | ⬜ pending |
| 43-03-01 | 03 | 2 | ADV-MTX-01..19, ADV-MTX-30..43 | — | N/A | unit+integration | `just test-core` | ✅ inline | ⬜ pending |
| 43-04-01 | 04 | 3 | ADV-MTX-20..29 | — | N/A | unit+integration | `just test-core` | ✅ inline | ⬜ pending |
| 43-05-01 | 05 | 3 | ADV-MTX-44..48, ADV-MATH-09..26 | — | N/A | unit | `just test-core` | ✅ inline | ⬜ pending |
| 43-06-01 | 06 | 3 | ADV-MATH-27..47 | — | N/A | unit | `just test-core` | ✅ inline | ⬜ pending |
| 43-07-01 | 07 | 4 | ADV-MATH-03..08 | — | N/A | unit+integration | `just test-core` | ✅ inline | ⬜ pending |
| 43-08-01 | 08 | 4 | ADV-MATH-01..02, ADV-MTX-49..50 | — | N/A | unit | `just test-core` | ✅ inline | ⬜ pending |
| 43-09-01 | 09 | 5 | ADV-TVM-01..06 | — | N/A | unit | `just test-core` | ✅ inline | ⬜ pending |
| 43-10-01 | 10 | 5 | ADV-FW-06 | — | N/A | integration | `just test-core` | ✅ inline | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Wave 0 is satisfied by the Rust inline `#[cfg(test)]` convention: tests are created alongside implementation in the same file, same task. Each plan task creates its own test functions as part of the implementation. No separate test-stub creation step is needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Compile-break in hp41-cli/hp41-gui | ADV-FW-06 | Intentional `non-exhaustive patterns` errors | Run `cargo check -p hp41-cli` and verify expected pattern match errors for new Op variants |

*All other phase behaviors have automated verification.*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (inline #[cfg(test)] pattern)
- [x] No watch-mode flags
- [x] Feedback latency < 15s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-05-25 (inline test pattern — tests created alongside implementation per Rust convention)
