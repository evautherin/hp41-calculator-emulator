---
phase: 36
slug: hp41-gui-gui-integration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-24
---

# Phase 36 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + vitest (React) + WebdriverIO (E2E) |
| **Config file** | `Cargo.toml` / `hp41-gui/vitest.config.ts` / `hp41-gui/wdio.conf.cjs` |
| **Quick run command** | `just check` |
| **Full suite command** | `just ci && just gui-ci` |
| **Estimated runtime** | ~45 seconds (CLI) + ~30 seconds (GUI) |

---

## Sampling Rate

- **After every task commit:** Run `just check`
- **After every plan wave:** Run `just ci && just gui-ci`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 75 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 36-01-01 | 01 | 1 | STAT-GUI-01 | — | N/A | compile | `cargo check -p hp41-gui` | ✅ | ⬜ pending |
| 36-02-01 | 02 | 1 | STAT-GUI-02 | — | N/A | unit | `cargo test -p hp41-gui catalog` | ❌ W0 | ⬜ pending |
| 36-02-02 | 02 | 1 | STAT-GUI-03 | — | N/A | unit + vitest | `cd hp41-gui && npx vitest run` | ✅ | ⬜ pending |
| 36-03-01 | 03 | 2 | STAT-GUI-04 | — | N/A | integration | `cargo test -p hp41-gui modal` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements. No new test frameworks needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| CATALOG 2 visual display | STAT-GUI-02 | Visual layout verification | Open GUI, press CATALOG, verify "STAT 1B" appears alongside "MATH 1A" |
| Help overlay search | STAT-GUI-03 | Interactive search UX | Open `?` overlay, type "NORMD", verify Stat 1 results appear |
