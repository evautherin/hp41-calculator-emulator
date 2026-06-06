# 57-04 SUMMARY — Operational acceptance: signing, TestFlight, device install

**Status:** complete (cloud + device acceptance achieved; CI automation green)
**Requirements:** SHIP-01, SHIP-02 (partial — see manifest note), SHIP-04, SHIP-05

This plan was all human/cloud checkpoints (no executor). Acceptance was reached after a
substantial signing investigation — the planned "ASC API key + `-allowProvisioningUpdates`"
path (D-57.2) proved **unusable on Xcode 26** and was replaced with **manual signing**.

## What was done

1. **ASC API key + 3 GitHub secrets** (Checkpoint 1 ✅) — key `NK2C5F49AS`, issuer
   `bcfd27fd-…`, team **Talent Factory AG `2P4R8QSWT4`**. Validated against the public ASC API
   (HTTP 200). `ASC_KEY_ID` / `ASC_ISSUER_ID` / `ASC_API_KEY_P8` set (later re-set from the
   known-good `.p8` to fix an upload-auth 401).

2. **App registered** in App Store Connect — bundle `ch.talent-factory.hp41` (app id
   `6776616594`, seedId `2P4R8QSWT4`). Without it, export failed "No profiles found".

3. **PrivacyInfo / pbxproj prerequisite** — `just ios-init` regen was attempted to wire
   `PrivacyInfo.xcprivacy` into the bundle but it duplicated `libapp.a` (Externals folder-scan)
   and broke the build; reverted. A stale debug `libapp.a` in Copy-Bundle-Resources was also
   removed (only failed on a clean release build). **PrivacyInfo is NOT yet in the bundle** —
   ITMS-91053 remains a (non-blocking for internal TestFlight) warning. Tracked:
   `.planning/todos/pending/privacy-manifest-bundle-wiring.md`.

4. **Signing pivot (revises D-57.2)** — Xcode 26's `-allowProvisioningUpdates` returns **401**
   on `xcbuild/listTeams.action` even with a valid key (the modern ASC API works fine). Created
   an **Apple Distribution cert** (`N7QPP6P52Q`) + **App Store profile** (`HP-41 App Store
   (ci-ios)`, uuid `af0c5132-…`) directly via the ASC API, then used **manual signing**.

5. **Signed IPA built + uploaded** (Checkpoint 2 ✅) — proven **twice**:
   - **Local:** archive via Tauri → `xcodebuild -exportArchive` (manual ExportOptions) →
     `altool --upload-app`. v4.1 build 1, Delivery UUID `3bbfdd7c-…`, **VALID** in TestFlight.
   - **CI:** `ci-ios.yml` `workflow_dispatch` run **26950247621 green end-to-end** (keychain
     import → archive → manual export → altool upload). SHIP-04 automation complete.

6. **Internal distribution + on-device** (Checkpoint 3 ✅, SHIP-05 / SHIP-01 on-device) — the
   founder confirmed the app installs and runs from TestFlight on a real iPhone (icon, branded
   launch screen, RPN engine all working).

## Carry-forward / follow-ups

- **PrivacyInfo.xcprivacy in bundle** before any App Store (external) submission — todo above.
- **`just ios-init`** would reset the release config from Manual back to Automatic (regen is
  environment-sensitive) — see the `reference_ios_signing_testflight` memory.
- Distribution cert/key + profile live in GitHub secrets (`IOS_DIST_CERT_P12`,
  `IOS_DIST_CERT_PASSWORD`, `IOS_PROVISION_PROFILE_B64`); rotate by re-creating via the ASC API.

## Key artifacts

- `.github/workflows/ci-ios.yml` (manual-signing pipeline), `.github/ios-export-options.plist`
- `Justfile` `ios-build-release` recipe (Tauri archive + self-export)
- `hp41-gui/src-tauri/tauri.conf.json` `version: 4.1.0` (real version source)
