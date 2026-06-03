---
phase: 56
slug: app-lifecycle-clock
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-03
---

# Phase 56 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) + on-device manual checkpoint |
| **Config file** | None — existing `cargo test` in `hp41-gui/src-tauri/` |
| **Quick run command** | `cargo test -p hp41-gui` |
| **Full suite command** | `just gui-ci` (GUI) / `just ci` (CLI + GUI) |
| **Estimated runtime** | ~30–60 seconds (Rust GUI tests); `just ios-build` adds minutes |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p hp41-gui`
- **After every plan wave:** Run `just gui-ci`
- **Before `/gsd:verify-work`:** `just ios-build` must succeed AND the device checkpoint (D-56.4) must pass on a real iPhone
- **Max feedback latency:** ~60 seconds (unit); device checkpoint is manual and out-of-band

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 56-01-xx | 01 | 1 | LIFE-01 | — | N/A | build check | `just ios-build` (valid JSON + clean merge) | ✅ | ⬜ pending |
| 56-01-xx | 01 | 1 | LIFE-02 | — | N/A | unit | `cargo test -p hp41-gui` (existing `tick_time` IPC tests, commands.rs) | ✅ | ⬜ pending |
| 56-01-xx | 01 | 1 | LIFE-01/02 | — | desktop/macOS not regressed | build check | `just gui-ci` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all automatable phase requirements.* No new test framework or fixtures needed — `cargo test -p hp41-gui` already exercises the `tick_time` IPC command (commands.rs), and `just ios-build` validates the new `tauri.ios.conf.json` (build fails on invalid JSON or a broken merge patch).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| WKWebView not fully suspended on brief backgrounding (iOS 17+) | LIFE-01 | Background-throttling behavior is not faithfully reproducible in the Simulator; authoritative only on real hardware (D-56.4) | On a real iPhone: start the clock, press Home for ~10 s, return → confirm timekeeping behaves per `backgroundThrottling: "throttle"` (no full suspend). |
| Clock/stopwatch display correct within one frame on foreground return | LIFE-02 | "No stale time visible even for one frame" is observable only on device, not in Simulator (D-56.4) | Start the clock, background the app (Home) ~10 s, return → displayed time must jump to current wall-clock time within one render frame (no visible stale value). Repeat with a running stopwatch. |
| `visible` branch does not fire spurious IPC on desktop / macOS menu-bar | LIFE-02 (no-regression) | Cross-platform regression guard; `isIos: false` gates the iOS-only path | `just gui-dev` (or `npm run dev`), switch browser tab away and back → confirm no extra `tick_time` IPC fires when `needsTick` is false; desktop + macOS menu-bar behavior unchanged. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or are listed as Manual-Only (device-only per D-56.4)
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (none required)
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s (unit)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
