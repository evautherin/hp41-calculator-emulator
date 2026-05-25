---
phase: 39
slug: hp41-cli-cli-integration-live-display
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-25
---

# Phase 39 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) + cargo-llvm-cov (coverage) |
| **Config file** | Cargo.toml + justfile |
| **Quick run command** | `just test` |
| **Full suite command** | `just ci` |
| **Estimated runtime** | ~45 seconds |

---

## Sampling Rate

- **After every task commit:** Run `just test`
- **After every plan wave:** Run `just ci`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 45 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 39-01-01 | 01 | 1 | TIME-CLI-01 | — | N/A | integration | `cargo test --package hp41-cli help_data` | ✅ | ⬜ pending |
| 39-01-02 | 01 | 1 | TIME-CLI-02 | — | N/A | integration | `cargo test --package hp41-cli help_data` | ✅ | ⬜ pending |
| 39-01-03 | 01 | 1 | TIME-CLI-03 | — | N/A | compile | `cargo check --package hp41-cli` | ✅ | ⬜ pending |
| 39-01-04 | 01 | 1 | TIME-CLI-04 | — | N/A | integration | `cargo test --package hp41-cli help_overlay` | ✅ | ⬜ pending |
| 39-02-01 | 02 | 1 | TIME-CLI-05 | — | N/A | manual | See manual verifications | ❌ | ⬜ pending |
| 39-02-02 | 02 | 1 | TIME-CLI-06 | — | N/A | manual | See manual verifications | ❌ | ⬜ pending |
| 39-02-03 | 02 | 1 | TIME-CLI-07 | — | N/A | integration | `cargo test --package hp41-cli alarm` | ❌ W0 | ⬜ pending |
| 39-02-04 | 02 | 1 | TIME-CLI-08 | — | N/A | integration | `cargo test --package hp41-cli xrom_shadowing` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- Existing infrastructure covers all phase requirements. cargo test framework is already installed. Integration test files for help_data, prgm_display, xrom_shadowing, and function_matrix_parity already exist.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live clock display updates at ≥1 Hz | TIME-CLI-05 | Requires visual verification of TUI rendering with real-time clock | 1. Run `cargo run --package hp41-cli` 2. Type XEQ "CLKT" 3. Verify LCD updates every second 4. Press any key to exit clock mode |
| Stopwatch display updates at ≥10 Hz | TIME-CLI-06 | Requires visual verification of centisecond-resolution TUI updates | 1. Run `cargo run --package hp41-cli` 2. Type XEQ "SW" 3. Press start key 4. Verify centiseconds are visibly updating 5. Press stop/Esc |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 45s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
