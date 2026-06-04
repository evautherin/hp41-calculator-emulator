---
phase: 57
slug: signing-testflight-pipeline
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-04
---

# Phase 57 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `just` recipes + `cargo test` (hp41-core/cli, unchanged); CI = GitHub Actions macOS runner; manual = TestFlight install on a real iPhone |
| **Config file** | `.github/workflows/ci-ios.yml` (new), `hp41-gui/src-tauri/gen/apple/*` (tracked) |
| **Quick run command** | `just gui-ci` (host-target check — cannot see iOS-only build errors, see P cfg(mobile) blind spot) |
| **Full suite command** | `just ios-build` (release export) on macOS; then `ci-ios.yml` workflow_dispatch run |
| **Estimated runtime** | ~local: minutes; CI macOS runner: ~15–25 min |

---

## Sampling Rate

- **After every task commit:** Run `just gui-ci` (fast host check) + `plutil -lint` on any edited plist/xcprivacy
- **After every plan wave:** Run `just ios-build` locally (signing/export) where the wave touches the build path
- **Before `/gsd:verify-work`:** A `ci-ios.yml` `workflow_dispatch` run produces a signed IPA AND an internal TestFlight build is installable
- **Max feedback latency:** local host check < 120s; full iOS export/CI run is the slow gate (minutes)

---

## Per-Task Verification Map

> Filled by the planner/executor. Many SHIP criteria are device/cloud-bound (TestFlight, App Store Connect) and therefore live in **Manual-Only Verifications** below; CI-side checks (plist lint, file presence, workflow YAML validity, build success) are automatable.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 57-XX-XX | XX | X | SHIP-02 | — | `PrivacyInfo.xcprivacy` valid + in bundle | unit | `plutil -lint hp41-gui/src-tauri/gen/apple/PrivacyInfo.xcprivacy` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] No new test framework needed — `hp41-core`/`hp41-cli` suites unchanged (engine frozen)
- [ ] CI-side automatable checks: `plutil -lint` for plist/xcprivacy, `actionlint`/`yamllint` for `ci-ios.yml`, presence + slot-count check for `AppIcon.appiconset`

*Existing infrastructure covers all phase requirements; new checks are lint/presence assertions, not a new framework.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Signed IPA installs + runs on a real iPhone | SHIP-01 | Requires physical device + distribution provisioning | `just ios-build` → install IPA via `xcrun devicectl` (device unlocked); launch and confirm app runs |
| Upload does not trigger ITMS-91053 | SHIP-02 | Only observable on real App Store Connect upload | Run `ci-ios.yml` → confirm `altool` upload succeeds with no ITMS-91053 |
| Custom icon (all sizes) + branded launch screen | SHIP-03 | Visual judgement | Inspect installed app icon on Home screen + launch screen on cold start; founder approves art direction |
| `ci-ios.yml` builds signed IPA + uploads via `altool` | SHIP-04 | Needs live ASC secrets + runner | Trigger workflow_dispatch; confirm green run + build appears in App Store Connect |
| Internal TestFlight build installable on a real iPhone | SHIP-05 | TestFlight cloud + device | Add internal tester; install from TestFlight app; launch on device |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify OR a Manual-Only row with explicit instructions
- [ ] Sampling continuity: CI-side checks run after each commit
- [ ] Wave 0 covers all lint/presence checks
- [ ] No watch-mode flags
- [ ] Feedback latency target documented (host check < 120s)
- [ ] `nyquist_compliant: true` set in frontmatter once map is filled

**Approval:** pending
