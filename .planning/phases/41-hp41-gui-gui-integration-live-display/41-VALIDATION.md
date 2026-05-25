---
phase: 41
slug: hp41-gui-gui-integration-live-display
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-25
---

# Phase 41 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + vitest (React) + WebdriverIO (E2E) |
| **Config file** | `hp41-gui/vitest.config.ts`, `hp41-gui/wdio.conf.cjs` |
| **Quick run command** | `cargo check -p hp41-gui && cd hp41-gui && npx vitest run` |
| **Full suite command** | `just gui-ci` |
| **Estimated runtime** | ~45 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check -p hp41-gui`
- **After every plan wave:** Run `just gui-ci`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 45 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 41-01-01 | 01 | 1 | TIME-GUI-01 | — | N/A | compile | `cargo check -p hp41-gui` | ✅ | ⬜ pending |
| 41-01-02 | 01 | 1 | TIME-GUI-05 | — | N/A | compile | `cargo check -p hp41-gui` | ✅ | ⬜ pending |
| 41-02-01 | 02 | 1 | TIME-GUI-02 | — | N/A | unit | `cargo test -p hp41-gui` | ❌ W0 | ⬜ pending |
| 41-02-02 | 02 | 1 | TIME-GUI-03 | — | N/A | unit | `cargo test -p hp41-gui` | ❌ W0 | ⬜ pending |
| 41-03-01 | 03 | 2 | TIME-GUI-04 | — | N/A | vitest | `cd hp41-gui && npx vitest run` | ❌ W0 | ⬜ pending |
| 41-03-02 | 03 | 2 | TIME-GUI-06 | — | N/A | vitest | `cd hp41-gui && npx vitest run` | ❌ W0 | ⬜ pending |
| 41-04-01 | 04 | 2 | TIME-GUI-07 | — | N/A | e2e | `cd hp41-gui && npx wdio run wdio.conf.cjs` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements — vitest and WebdriverIO already configured from v2.0/v3.1.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Clock display updates ≥1 Hz | TIME-GUI-04 | Visual timing verification | Activate clock mode → observe LCD updates for 5s → verify at least 5 transitions |
| Stopwatch centisecond display | TIME-GUI-04 | Visual timing verification | Start stopwatch → observe LCD → verify centisecond digits change rapidly |
| Alarm toast appears on fire | TIME-GUI-06 | Requires real-time alarm scheduling | Set alarm 10s in future → wait → observe toast notification |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 45s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
