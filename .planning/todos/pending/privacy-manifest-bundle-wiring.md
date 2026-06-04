---
id: privacy-manifest-bundle-wiring
created: 2026-06-04
type: tech-debt
priority: medium
status: pending
source: 57-signing-testflight-pipeline
blocks: app-store-submission
---

# Wire PrivacyInfo.xcprivacy into the shipped iOS bundle (SHIP-02 follow-up)

## What

`hp41-gui/src-tauri/gen/apple/PrivacyInfo.xcprivacy` exists and is declared as a
source in `project.yml` (Phase 57-01), but the **committed `.pbxproj` does not
reference it**, so it is NOT copied into the `.app` bundle. Apple emits **ITMS-91053**
(missing privacy manifest) on upload.

## Why deferred (Phase 57 decision, 2026-06-04)

- v4.1 milestone ships **internal-only TestFlight** (D-57.9). ITMS-91053 is a
  **non-blocking warning** for internal testing — the build still distributes and
  installs. It is only **hard-enforced at App Store (external) review**.
- The obvious fix — `just ios-init` to regenerate the project — is **environment-
  sensitive**: it folder-scans `Externals/`, and when both `debug/` and `release/`
  `libapp.a` exist locally it duplicates the `libapp.a` Resources entry, producing
  `xcodebuild error 65: Multiple commands produce .../libapp.a`. That regression was
  reverted (commit `feed2f8`); the known-good committed project has no PrivacyInfo wiring.

## How to close (before ANY App Store submission)

Pick a regen-independent approach so it survives and doesn't reintroduce the libapp.a dup:

1. **Preferred — fix at the project.yml/source level** so a *clean* `ios-init`
   (run with a pristine `Externals/`, i.e. only the target config's `libapp.a`
   present) regenerates a project that (a) references `PrivacyInfo.xcprivacy` in
   Copy-Bundle-Resources AND (b) keeps a single per-arch `libapp.a` entry. Verify
   `grep -c 'libapp.a in Resources'` stays at 2 and `grep PrivacyInfo project.pbxproj`
   is non-empty, then build on-device via the device-verify loop.
2. **Fallback — surgical `.pbxproj` edit**: add a `PBXFileReference` +
   `PBXBuildFile` for `PrivacyInfo.xcprivacy` into the `hp41-gui_iOS`
   `PBXResourcesBuildPhase` + a group entry, leaving `libapp.a` untouched. Note this
   is overwritten by any future `just ios-init`, so document it loudly.

## Verify

- Build the IPA, unzip it, confirm `PrivacyInfo.xcprivacy` is inside the `.app`.
- A `ci-ios` upload produces **no ITMS-91053** email.

See `.planning/phases/57-signing-testflight-pipeline/57-01-SUMMARY.md` (carry-forward
caveat) and `docs/hp41-xmem-divergences.md` / RESEARCH Pattern 6 for the manifest content.
