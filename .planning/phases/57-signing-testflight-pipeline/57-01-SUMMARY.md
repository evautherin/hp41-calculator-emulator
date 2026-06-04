---
phase: 57-signing-testflight-pipeline
plan: "01"
subsystem: ios-signing-config
tags: [ios, signing, privacy-manifest, testflight, justfile]
dependency_graph:
  requires: []
  provides:
    - PrivacyInfo.xcprivacy (SHIP-02 — file-timestamp privacy manifest)
    - ExportOptions.plist with method=app-store-connect (SHIP-01 config)
    - marketing version 4.1 in Info.plist + project.yml (D-57.6)
    - ios-build-release just recipe (SHIP-01 build entry point)
  affects:
    - hp41-gui iOS Xcode project (project.yml source list + version properties)
    - CI invocation surface (justfile ios group)
tech_stack:
  added: []
  patterns:
    - Apple privacy manifest plist format (NSPrivacyAccessedAPICategoryFileTimestamp / C617.1)
    - XcodeGen bare source path for .xcprivacy (not under buildPhase: resources — avoids issue #1459)
    - ExportOptions.plist method=app-store-connect (Xcode 26 canonical, app-store deprecated)
    - just recipe with npm ci + npm run tauri ios build -- flags (mirrors gui-build pattern)
key_files:
  created:
    - hp41-gui/src-tauri/gen/apple/PrivacyInfo.xcprivacy
  modified:
    - hp41-gui/src-tauri/gen/apple/project.yml
    - hp41-gui/src-tauri/gen/apple/ExportOptions.plist
    - hp41-gui/src-tauri/gen/apple/hp41-gui_iOS/Info.plist
    - Justfile
decisions:
  - "PrivacyInfo.xcprivacy wired via project.yml bare sources entry (not buildPhase: resources) to avoid XcodeGen #1459 double-add"
  - "ExportOptions.plist method set to app-store-connect (not deprecated app-store); Tauri CLI overwrites at build time anyway"
  - "CFBundleVersion baseline kept at 1.0.0 — CI PlistBuddy is the single writer (D-57.5)"
  - "ios-build-release placed between ios-build and ios-sim in [group(ios)]"
metrics:
  duration_minutes: 10
  completed_date: "2026-06-04"
  tasks_completed: 3
  tasks_total: 3
  files_changed: 5
---

# Phase 57 Plan 01: Signing Config + Privacy Manifest Foundation Summary

Laid the signing-config and privacy-manifest foundation for the Phase 57 TestFlight pipeline: created `PrivacyInfo.xcprivacy` (SHIP-02), wired it into the Xcode project via `project.yml`, switched `ExportOptions.plist` to `method=app-store-connect`, bumped marketing version to 4.1 in both `Info.plist` and `project.yml`, and added the `ios-build-release` just recipe (SHIP-01 build entry point).

## Tasks Completed

| # | Name | Commit | Files |
|---|------|--------|-------|
| 1 | Create PrivacyInfo.xcprivacy + wire into project.yml | 590d508 | PrivacyInfo.xcprivacy (new), project.yml |
| 2 | Switch ExportOptions to app-store-connect + bump marketing version to 4.1 | 1f720ad | ExportOptions.plist, Info.plist, project.yml |
| 3 | Add just ios-build-release recipe | 17c7b3b | Justfile |

## Key Notes (Required by Plan Output Spec)

### 1. .pbxproj Does NOT Yet Reference PrivacyInfo.xcprivacy — Regen Owed Before Plan 04 Upload

`grep 'PrivacyInfo' hp41-gui/src-tauri/gen/apple/hp41-gui.xcodeproj/project.pbxproj` returns 0 matches. The `project.yml` source entry is the source-of-truth, but the `.pbxproj` (which Xcode actually reads) has not been regenerated yet. XcodeGen regeneration happens via `npm run tauri ios init` (or `just ios-init`).

**Action required before Plan 04 (TestFlight upload):** Run `just ios-init` to regenerate the `.pbxproj` from the updated `project.yml`, then verify `PrivacyInfo.xcprivacy` appears in the Xcode file tree and is included in the Copy Bundle Resources phase. Alternatively, edit `project.pbxproj` directly to add the file reference and resource build phase entry. Without this step, the privacy manifest will NOT be included in the IPA bundle, causing ITMS-91053 from Apple.

### 2. Tauri CLI Overwrites ExportOptions.plist at Build Time

Per 57-RESEARCH.md Pattern 5: when `--export-method app-store-connect` is passed to `tauri ios build`, Tauri CLI regenerates `ExportOptions.plist` before invoking `xcodebuild -exportArchive`. The committed file therefore serves as:
- **Documentation** of the intended export configuration
- **Fallback** for direct `xcodebuild` invocations that bypass Tauri CLI
- **Reference** for `signingStyle=automatic` and `teamID=2P4R8QSWT4`

The `--export-method` flag passed to the recipe is the authoritative source for CI builds.

### 3. ios-build-release Recipe — 3 Required Env Vars

The `ios-build-release` recipe requires these environment variables at invocation time (consumed by Tauri CLI, forwarded to `xcodebuild -authenticationKeyPath ... -allowProvisioningUpdates`):

| Env Var | Content | Source in CI |
|---------|---------|--------------|
| `APPLE_API_KEY` | Key ID string (e.g. `ABC123DEF456`) | `secrets.ASC_KEY_ID` |
| `APPLE_API_ISSUER` | Issuer ID UUID | `secrets.ASC_ISSUER_ID` |
| `APPLE_API_KEY_PATH` | Absolute path to `.p8` file (e.g. `~/.appstoreconnect/private_keys/AuthKey_<ID>.p8`) | Constructed from `runner.home` + `ASC_KEY_ID` in CI step env block |

Without all three, `xcodebuild` cannot authenticate for automatic provisioning and will fail with "No signing certificate" or "Unable to find authentication key."

## Decisions Made

1. **PrivacyInfo.xcprivacy bare sources entry (not buildPhase: resources):** XcodeGen has a known bug (#1459, fixed in PR #1464) where `.xcprivacy` files added to `buildPhase: resources` get double-added. A bare `path:` entry lets XcodeGen handle the `.xcprivacy` type natively.

2. **ExportOptions.plist method=app-store-connect:** `app-store` is deprecated in Xcode 26 per `xcodebuild -help`. `app-store-connect` is the canonical Xcode 26+ value. The file is overwritten by Tauri CLI at build time; the committed value is documentation.

3. **CFBundleVersion stays at 1.0.0 baseline (D-57.5):** CI PlistBuddy writes `github.run_number` to the tracked `Info.plist` before archiving (ephemeral mutation on the runner, never committed). This preserves the monotonically-increasing TestFlight build number requirement without any bookkeeping.

4. **ios-build-release recipe placement:** Inserted between `ios-build` and `ios-sim` in the `[group('ios')]` section, following the `gui-build` pattern of `npm ci` first (lockfile-strict) then the build command.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — all config changes are complete and functional. The .pbxproj regen is a follow-up action (documented above), not a stub.

## Verification Results

All acceptance criteria passed:

```
plutil -lint PrivacyInfo.xcprivacy          → OK
grep NSPrivacyAccessedAPICategoryFileTimestamp → exit 0
grep C617.1                                 → exit 0
grep PrivacyInfo.xcprivacy project.yml      → exit 0 (not under buildPhase: resources)
plutil -lint ExportOptions.plist            → OK
grep app-store-connect ExportOptions.plist  → exit 0
grep -v debugging ExportOptions.plist       → OK (not present)
PlistBuddy CFBundleShortVersionString       → 4.1
PlistBuddy CFBundleVersion                 → 1.0.0
grep CFBundleShortVersionString: 4.1 project.yml → exit 0
just --list | grep ios-build-release        → found
grep --export-method app-store-connect Justfile → exit 0
grep --ci Justfile                          → exit 0
grep [group('ios')] Justfile               → exit 0
grep -A4 ios-build-release: | grep cargo|xcodebuild → no match (OK)
```

## Self-Check

### Commits verified:
- `590d508`: feat(57-01): add PrivacyInfo.xcprivacy and wire into iOS Xcode project (SHIP-02)
- `1f720ad`: feat(57-01): switch ExportOptions to app-store-connect and bump marketing version to 4.1 (SHIP-01)
- `17c7b3b`: feat(57-01): add ios-build-release just recipe for signed App Store Connect export (SHIP-01)

### Files verified to exist:
- `hp41-gui/src-tauri/gen/apple/PrivacyInfo.xcprivacy` ✓
- `hp41-gui/src-tauri/gen/apple/ExportOptions.plist` ✓ (app-store-connect)
- `hp41-gui/src-tauri/gen/apple/hp41-gui_iOS/Info.plist` ✓ (4.1)
- `hp41-gui/src-tauri/gen/apple/project.yml` ✓ (4.1 + PrivacyInfo source)
- `Justfile` ✓ (ios-build-release recipe present)

## Self-Check: PASSED
