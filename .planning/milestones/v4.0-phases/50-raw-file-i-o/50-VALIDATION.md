---
phase: 50
slug: raw-file-i-o
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-27
---

# Phase 50 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework (Rust core)** | `cargo test` (built-in) |
| **Framework (GUI Rust)** | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` |
| **Framework (GUI frontend)** | Vitest (`cd hp41-gui && npm test`) |
| **Config file** | `hp41-gui/vitest.config.ts` (or `vite.config.ts` with test block) |
| **Quick run command** | `cargo test -p hp41-core --test cardreader_tests` |
| **Full suite command** | `just test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p hp41-core --test cardreader_tests`
- **After every plan wave:** Run `just test`
- **Before `/gsd:verify-work`:** Full suite must be green (`just test && cd hp41-gui && npm test`)
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 50-01-01 | 01 | 0 | RAW-03 | T-50-03 | Malformed archives rejected with clear error | unit | `cargo test -p hp41-core decode_all_programs` | ❌ W0 | ⬜ pending |
| 50-01-02 | 01 | 0 | RAW-05 | — | N/A | unit | `cargo test -p hp41-core xrom_round_trip` | ❌ W0 | ⬜ pending |
| 50-02-01 | 02 | 1 | RAW-01 | T-50-01 | Malformed .raw bytes rejected without panic | unit | `cargo test -p hp41-core --test cardreader_tests` | ✅ partial | ⬜ pending |
| 50-02-02 | 02 | 1 | RAW-02 | — | N/A | unit | `cargo test -p hp41-core --test cardreader_tests` | ✅ partial | ⬜ pending |
| 50-02-03 | 02 | 1 | RAW-04 | T-50-02 | Dialog path never panics on cancel | manual | — | ❌ | ⬜ pending |
| 50-03-01 | 03 | 2 | RAW-03 | T-50-03 | Multi-program picker caps at 256 entries | integration | `cargo test -p hp41-core decode_all_programs` | ❌ W0 | ⬜ pending |
| 50-04-01 | 04 | 3 | RAW-06 | T-50-04 | stdin `-` handled; no path traversal | integration | `cargo test -p hp41-cli cli_import_export` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-core/tests/cardreader_raw_multi.rs` — covers RAW-03 (`decode_all_programs`, multi-program archives, false-split guard for alpha payloads containing END bytes)
- [ ] `hp41-core/tests/cardreader_xrom_roundtrip.rs` — covers RAW-05 (XROM op round-trip via SyntheticByte)
- [ ] `hp41-cli/tests/cli_import_export.rs` — covers RAW-06 (CLI flag integration test with tempfile)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| File dialog opens on GUI with correct filters | RAW-04 | Native OS dialog cannot be unit-tested | 1. Launch `just gui-dev` 2. With ALPHA empty, trigger WPRGM 3. Verify OS file dialog opens with `.raw` filter |
| Multi-program picker shows checkboxes | RAW-03 | UI interaction cannot be unit-tested | 1. Import a multi-program `.raw` archive 2. Verify picker modal appears with program list 3. Select multiple programs 4. Verify all selected programs loaded |
| Toast + status line feedback on import | D-50.3 | Visual verification | 1. Import a program via dialog 2. Verify toast "Imported QUAD (47 steps)" appears 3. Verify status line shows program name |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
