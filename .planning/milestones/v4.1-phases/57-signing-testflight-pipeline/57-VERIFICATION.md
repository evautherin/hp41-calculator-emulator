---
phase: 57-signing-testflight-pipeline
verified: 2026-06-04T12:30:00Z
status: passed
score: 5/5
overrides_applied: 0
re_verification: false
method: "Verified by live evidence rather than a host verifier subagent: a green ci-ios workflow_dispatch run (26950247621) built + signed + uploaded the IPA, and the founder confirmed install + run on a physical iPhone via TestFlight. Cross-phase wiring confirmed by the v4.1 milestone integration check."
note: "SHIP-02 satisfied as worded (PrivacyInfo.xcprivacy present in gen/apple/ with C617.1); the stricter bundle-wiring (manifest inside the shipped .app) is DEFERRED tech debt — ITMS-91053 is a non-blocking warning for internal TestFlight. Tracked: .planning/todos/pending/privacy-manifest-bundle-wiring.md."
---

## Requirements

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| SHIP-01 | 57-01, 57-04 | Distribution cert + provisioning profile → signed IPA | passed | Apple Distribution cert `N7QPP6P52Q` + App Store profile `HP-41 App Store (ci-ios)` (created via ASC API); signed IPA produced locally and by green CI run 26950247621 (`codesign` Authority = Apple Distribution: Talent Factory AG `2P4R8QSWT4`). |
| SHIP-02 | 57-01 | `PrivacyInfo.xcprivacy` present in `gen/apple/` with `NSPrivacyAccessedAPICategoryFileTimestamp` / `C617.1` | passed (bundle-wiring deferred) | File present + declared in `project.yml`; NOT yet referenced in committed `.pbxproj` → ITMS-91053 warning (non-blocking for internal TestFlight). Tracked tech debt before App Store submission. |
| SHIP-03 | 57-02 | App icon + launch screen | passed | HP-41 stylized-faceplate icon (1024 master → 18 opaque slots incl. marketing) + dark-branded `LaunchScreen.storyboard`; founder confirmed both on-device. |
| SHIP-04 | 57-03 | `ci-ios.yml` (macOS) builds signed IPA + uploads via `xcrun altool` | passed | `workflow_dispatch`-only workflow, build via `just ios-build-release` (manual signing), upload via `altool --upload-app`. **Green end-to-end run 26950247621.** |
| SHIP-05 | 57-04 | Build distributed to testers via TestFlight (internal) | passed | Build uploaded (local Delivery UUID `3bbfdd7c-…` + CI run); founder confirmed install + run (icon, branded launch screen, RPN engine) on a physical iPhone. |

## must_haves

- Signed IPA builds + uploads to TestFlight — ✅ (local + CI)
- No ITMS-91053 (SHIP-02 in cloud) — ⚠ deferred: manifest not in bundle; warning accepted for internal TestFlight, tracked todo
- Internal distribution + on-device install/run — ✅ (founder-confirmed)
- CI automation green — ✅ (`workflow_dispatch` run 26950247621)

## Notes

- Planned auto-provisioning (D-57.2) was unusable on Xcode 26 (`-allowProvisioningUpdates` → 401);
  pivoted to manual signing (cert + profile via ASC API). See `reference_ios_signing_testflight`.
- Build-correctness fixes during execution: `runner.home` → `$HOME`; removed a stale debug
  `libapp.a` from Copy-Bundle-Resources; version source set in `tauri.conf.json` (`4.1.0`).
