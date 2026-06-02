---
phase: 54
slug: ios-persistence-layer
status: approved
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-02
---

# Phase 54 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` (cargo test via `just`) |
| **Config file** | none — workspace `Cargo.toml` |
| **Quick run command** | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` (no `just gui-test` recipe exists) |
| **Full suite command** | `just gui-ci` |
| **Estimated runtime** | ~30–60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml`
- **After every plan wave:** Run `just gui-ci`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 54-01-01 | 01 | 1 | PERSIST-01 | — | N/A | unit | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Note: device-lifecycle behaviors (PERSIST-02 background trigger, PERSIST-03 kill→relaunch round-trip) are exercised on-device and recorded under Manual-Only Verifications; the planner fills the automated rows for path resolution and the v4.0 backward-compat fixture test.*

---

## Wave 0 Requirements

- [ ] v4.0 `autosave.json` fixture committed under `hp41-gui/src-tauri/` test data — for PERSIST-03 backward-compat assertion
- [ ] backward-compat test augmentation in `persistence.rs` (existing tests at ~line 153)

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Background → kill → relaunch restores X/Y/Z/T + program memory + XROM state | PERSIST-03 | Requires real iOS lifecycle (Simulator/device); WKWebView suspend/resume cannot be faithfully simulated host-side | Run RPN session on iPhone, press Home, kill from app switcher, relaunch, confirm full state restored |
| Backgrounding triggers immediate autosave via `visibilitychange` | PERSIST-02 | WKWebView visibilitychange firing on Home/app-switch is device-observable only (MEDIUM confidence per RESEARCH) | Observe save file mtime updates on Home press before the 30 s timer would fire |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 60s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-06-02
