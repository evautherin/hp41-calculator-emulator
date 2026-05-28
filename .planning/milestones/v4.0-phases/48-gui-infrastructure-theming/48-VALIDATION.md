---
phase: 48
slug: gui-infrastructure-theming
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-27
---

# Phase 48 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust) + manual browser QA (Tauri GUI) |
| **Config file** | `hp41-gui/src-tauri/Cargo.toml` |
| **Quick run command** | `just gui-check` |
| **Full suite command** | `just gui-ci` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `just gui-check`
- **After every plan wave:** Run `just gui-ci`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 48-01-01 | 01 | 1 | INFRA-01 | — | N/A | unit | `cargo test -p hp41-gui prefs` | ❌ W0 | ⬜ pending |
| 48-01-02 | 01 | 1 | INFRA-02 | — | N/A | unit | `cargo test -p hp41-gui commands` | ❌ W0 | ⬜ pending |
| 48-02-01 | 02 | 2 | THEME-01 | — | N/A | manual | Browser QA: switch all 4 themes | ❌ W0 | ⬜ pending |
| 48-02-02 | 02 | 2 | THEME-02 | — | N/A | manual | Restart app, verify theme persists | ❌ W0 | ⬜ pending |
| 48-02-03 | 02 | 2 | THEME-03 | — | N/A | manual | Key press animation in all themes | ❌ W0 | ⬜ pending |
| 48-02-04 | 02 | 2 | THEME-04 | — | N/A | manual | WCAG AA contrast check | ❌ W0 | ⬜ pending |
| 48-02-05 | 02 | 2 | THEME-05 | — | N/A | unit | `grep -L theme autosave.json` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Existing `just gui-check` and `just gui-ci` infrastructure covers Rust compilation and type checks
- [ ] Manual browser QA covers visual theme verification

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Theme switch is instant with no flicker | THEME-01 | Visual assessment | Switch themes rapidly; verify no flash-of-wrong-theme |
| Key press animations work in all themes | THEME-03 | Visual assessment | Press keys in each theme; verify press animation visible |
| High-contrast meets WCAG AA | THEME-04 | Contrast ratio measurement | Use browser dev tools or axe to check contrast ratios |
| Classic Beige evokes real HP-41C | THEME-01 | Aesthetic judgment | Compare against HP-41C reference photos |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
