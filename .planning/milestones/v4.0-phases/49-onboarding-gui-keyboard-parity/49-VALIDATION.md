---
phase: 49
slug: onboarding-gui-keyboard-parity
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-27
---

# Phase 49 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest (GUI) + cargo test (Tauri backend) |
| **Config file** | `hp41-gui/vitest.config.ts` / `hp41-gui/src-tauri/Cargo.toml` |
| **Quick run command** | `cd hp41-gui && npm test -- --run` |
| **Full suite command** | `just gui-ci` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd hp41-gui && npm test -- --run`
- **After every plan wave:** Run `just gui-ci`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 49-01-01 | 01 | 1 | ONBOARD-01 | — | N/A | manual | Visual check: wizard shows on first launch | — | ⬜ pending |
| 49-01-02 | 01 | 1 | ONBOARD-02 | — | N/A | manual | Visual check: 5 panels with Next/Back nav | — | ⬜ pending |
| 49-01-03 | 01 | 1 | ONBOARD-04 | — | N/A | unit | `cargo test --package hp41-gui prefs` | ❌ W0 | ⬜ pending |
| 49-02-01 | 02 | 1 | ONBOARD-05 | — | N/A | manual | Visual check: expandable entries in ? overlay | — | ⬜ pending |
| 49-02-02 | 02 | 1 | KBD-03 | — | N/A | manual | Visual check: Keyboard Shortcuts section in ? overlay | — | ⬜ pending |
| 49-03-01 | 03 | 2 | KBD-01 | — | N/A | unit | `cd hp41-gui && npm test -- --run` | ❌ W0 | ⬜ pending |
| 49-03-02 | 03 | 2 | KBD-02 | — | N/A | unit | `cd hp41-gui && npm test -- --run` | ❌ W0 | ⬜ pending |
| 49-03-03 | 03 | 2 | KBD-04 | — | N/A | integration | `just gui-ci` | ❌ W0 | ⬜ pending |
| 49-04-01 | 04 | 2 | ONBOARD-03 | — | N/A | manual | Visual check: Show Guide button in SettingsPanel | — | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Existing vitest + cargo test infrastructure covers all automated test needs
- [ ] No new test framework installation required

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Wizard shows on first launch only | ONBOARD-01 | Requires app state reset + visual UI verification | 1. Delete `~/.hp41/prefs.json` 2. Launch GUI 3. Verify wizard appears 4. Close wizard 5. Relaunch 6. Verify wizard does NOT appear |
| 5-panel wizard with RPN/Stack/SHIFT/KBD/PRGM | ONBOARD-02 | Visual content + navigation UX verification | 1. Trigger wizard 2. Verify 5 panels with Next/Back 3. Check panel counter "N of 5" |
| Re-open wizard from SettingsPanel | ONBOARD-03 | Visual interaction across two overlays | 1. Open settings 2. Click "Show Guide" 3. Verify wizard opens from panel 1 |
| Expandable function entries | ONBOARD-05 | Visual UI interaction | 1. Open ? overlay 2. Click enriched entry 3. Verify example/notes expand 4. Click again to collapse |
| Keyboard Shortcuts section | KBD-03 | Visual layout verification | 1. Open ? overlay 2. Verify Keyboard Shortcuts at top 3. Check all mappings present |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
