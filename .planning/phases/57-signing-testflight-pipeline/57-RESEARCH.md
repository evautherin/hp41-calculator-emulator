# Phase 57: Signing + TestFlight Pipeline - Research

**Researched:** 2026-06-04
**Domain:** iOS CI/CD — Apple code signing, TestFlight distribution, app assets, GitHub Actions macOS runners
**Confidence:** HIGH (primary tool verification + official docs + live system checks on local Xcode 26.5 + Tauri 2.11.1)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-57.1:** App Store Connect API key (.p8 + issuer ID + key ID) for BOTH signing provisioning (`CODE_SIGN_STYLE: Automatic` + `xcodebuild -allowProvisioningUpdates`) AND the `xcrun altool` upload. Same key for both.
- **D-57.2:** No fastlane match, no certificate git repo, no raw P12/`.mobileprovision` keychain import — avoids P-iOS-16.
- **D-57.3:** Three ASC API-key components stored as GitHub repo secrets. `.p8` written to disk on runner and cleaned up after.
- **D-57.4:** `ci-ios.yml` runs on `workflow_dispatch` ONLY (manual trigger). NOTE: ROADMAP.md Success Criterion #4 says "on every push to main" — this CONFLICTS with D-57.4. CONTEXT.md is the more recent authority; the planner must reconcile before writing the plan (recommend verifying with user, or updating ROADMAP criterion to match D-57.4).
- **D-57.5:** `CFBundleVersion` = `github.run_number`, injected at build time via PlistBuddy/agvtool, never committed.
- **D-57.6:** `CFBundleShortVersionString` stays manual (e.g. `"4.1"`), not auto-derived.
- **D-57.7:** Real HP-41-styled app icon this phase (1024×1024 master → regenerate full `AppIcon.appiconset`) + branded `LaunchScreen.storyboard`. Placeholder not acceptable.
- **D-57.8:** `PrivacyInfo.xcprivacy` in `gen/apple/` declaring `NSPrivacyAccessedAPICategoryFileTimestamp` / reason `C617.1`.
- **D-57.9:** TestFlight INTERNAL distribution only.

### Claude's Discretion
- Exact GitHub secret variable names, precise `xcodebuild`/`altool` invocation
- Where in the `just` recipe layer CI hooks in
- Whether `ExportOptions.plist` is edited in place or generated in CI

### Deferred Ideas (OUT OF SCOPE)
- `v*`-tag-triggered TestFlight uploads
- External/public TestFlight + App Store submission (STORE-01/02)
- Desktop macOS notarization / signed GUI binaries
- `.raw` file picker on iOS
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SHIP-01 | Distribution cert + provisioning profile → signed IPA | ASC API key + automatic signing; `tauri ios build --export-method app-store-connect` |
| SHIP-02 | `PrivacyInfo.xcprivacy` in `gen/apple/` with `NSPrivacyAccessedAPICategoryFileTimestamp` / `C617.1` | Exact XML structure verified; XcodeGen sources entry confirmed |
| SHIP-03 | Custom app icon (all required sizes) + launch screen | `cargo tauri icon` + manual slot verification; `LaunchScreen.storyboard` branding |
| SHIP-04 | `ci-ios.yml` GitHub Actions workflow builds signed IPA + uploads via `xcrun altool` | Full invocation syntax verified via local `xcrun altool --help`; exact flags documented |
| SHIP-05 | Build distributed to testers via TestFlight internal | Upload + ASC TestFlight internal group assignment |
</phase_requirements>

---

## Summary

Phase 57 delivers a signed IPA produced by a GitHub Actions macOS CI pipeline and distributed to internal TestFlight testers. The entire pipeline rides on an App Store Connect (ASC) API key, eliminating all interactive keychain operations that fail on headless runners (P-iOS-16). Tauri CLI 2.11.1 supports this workflow natively via three environment variables (`APPLE_API_KEY`, `APPLE_API_ISSUER`, `APPLE_API_KEY_PATH`) that are forwarded to `xcodebuild -allowProvisioningUpdates -authenticationKeyPath ...` automatically during the archive step.

A critical finding confirmed by local Xcode 26.5 build artifacts (`Packaging.log`, `DistributionSummary.plist`): **the lswiftCompatibility56 linker bug (Tauri issue #15066) does NOT affect this project under Tauri 2.11.1 + Xcode 26.5**. The local development signing build completed to a valid IPA (104 MB, arm64). This means the CI pipeline can use the macOS-26 runner image's default Xcode 26.x without a version-lock workaround.

The ExportOptions.plist `method` must be `app-store-connect` (not `app-store`, which is deprecated per `xcodebuild -help`). The correct invocation uses `--export-method app-store-connect` passed to `tauri ios build`. The `PrivacyInfo.xcprivacy` file needs to be added to both the filesystem at `gen/apple/PrivacyInfo.xcprivacy` AND referenced in `project.yml` as a source entry so XcodeGen includes it in the Xcode project's Copy Bundle Resources phase.

**Primary recommendation:** Wire the CI pipeline through `APPLE_API_KEY` / `APPLE_API_ISSUER` / `APPLE_API_KEY_PATH` env vars consumed by Tauri CLI; inject `CFBundleVersion` via PlistBuddy on the tracked `Info.plist` before calling `tauri ios build`; upload with `xcrun altool --upload-package` using the same key.

---

## ROADMAP vs CONTEXT Conflict (Requires Planner/User Resolution)

| Source | Statement |
|--------|-----------|
| ROADMAP.md Phase 57 Success Criterion #4 | "`ci-ios.yml` runs on a macOS GitHub Actions runner … on every push to `main`" |
| CONTEXT.md D-57.4 | "`ci-ios.yml` runs on `workflow_dispatch` ONLY" |

CONTEXT.md was authored after ROADMAP.md and is the more recent authority. The planner must either: (a) write the plan for `workflow_dispatch` only and flag that ROADMAP SC #4 needs updating; or (b) surface the conflict for the user to resolve before locking the plan. Research recommends option (a) — the `workflow_dispatch`-only design is correct for a first proven pipeline, and `verify-phase` should check the CONTEXT criterion rather than the stale ROADMAP text.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Code signing (cert + profile) | Apple Developer Portal + CI | — | Handled by ASC API key + xcodebuild automatic signing on CI runner |
| IPA export | CI (xcodebuild / tauri ios build) | — | Happens on macOS runner inside the archive → export pipeline |
| Build number injection | CI runner (PlistBuddy) | — | Modifies tracked `Info.plist` before archiving; not committed |
| TestFlight upload | CI (xcrun altool) | — | Same runner, same ASC API key credential |
| PrivacyInfo.xcprivacy | gen/apple filesystem + project.yml | — | File committed to repo; XcodeGen includes it in bundle |
| App icon generation | Developer workstation (design tool + `cargo tauri icon`) | — | 1024×1024 master → all slots; committed to gen/apple |
| LaunchScreen branding | gen/apple/LaunchScreen.storyboard | — | XML edit; committed |
| Secrets management | GitHub repo secrets | CI runner env | .p8 written to disk, removed after upload |

---

## Standard Stack

### Core (CI pipeline tools — no new dependencies)

| Tool | Version | Purpose | Source |
|------|---------|---------|--------|
| `npm run tauri ios build` | Tauri CLI 2.11.1 (pinned in package-lock.json) | Archive + IPA export | [VERIFIED: local `npm view @tauri-apps/cli@2.11.1 version`] |
| `xcrun altool` | 26.40.1 (174001) bundled with Xcode 26.5 | Upload IPA to TestFlight | [VERIFIED: `xcrun altool --help` on Xcode 26.5] |
| `/usr/libexec/PlistBuddy` | macOS built-in | Inject CFBundleVersion before archive | [VERIFIED: present on macOS-26 runner (bundled with macOS)] |
| `xcodebuild` | Xcode 26.x (runner default) | Archive + exportArchive (called by Tauri CLI) | [VERIFIED: `xcodebuild -version` = Xcode 26.5 locally] |
| `actions/checkout@v4` | v4 | CI checkout | [ASSUMED] |
| `dtolnay/rust-toolchain@stable` | latest stable | Rust toolchain setup | [ASSUMED] |
| `Swatinem/rust-cache@v2` | v2 | Cargo cache | [ASSUMED] |
| `actions/setup-node@v4` | v4 | Node.js setup | [ASSUMED] |
| `taiki-e/install-action@v2` | v2 | `just` install | [ASSUMED] |

### Supporting (asset generation — no CI dependency)

| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| `cargo tauri icon` | 2.11.1 | Generate iOS icon sizes from 1024×1024 master | Run locally when creating app icon |
| ImageMagick / Affinity Designer / etc. | any | Create the 1024×1024 HP-41-styled master PNG | Before running `cargo tauri icon` |

**No new npm or Cargo dependencies are introduced by this phase.** All tooling is either macOS-bundled (Xcode, PlistBuddy) or already present in the project (Tauri CLI via package-lock, `just`).

### Package Legitimacy Audit

This phase installs no external packages. No package legitimacy audit is required.

---

## Architecture Patterns

### System Architecture Diagram

```
GitHub Actions trigger (workflow_dispatch)
        |
        v
[macos-26 runner]
  checkout → toolchain setup → npm ci
        |
        v
  PlistBuddy: inject CFBundleVersion = github.run_number
  into gen/apple/hp41-gui_iOS/Info.plist
        |
        v
  Write .p8 → ~/.appstoreconnect/private_keys/AuthKey_<KEY_ID>.p8
  (from base64-decoded ASC_API_KEY_P8 secret)
        |
        v
  [just ios-build-release]
    = cd hp41-gui && npm run tauri ios build --
        --export-method app-store-connect --ci
    env: APPLE_API_KEY, APPLE_API_ISSUER, APPLE_API_KEY_PATH
        |
        v
  Tauri CLI → npm run build (Vite frontend)
           → cargo build (aarch64-apple-ios)
           → xcodebuild archive
               -authenticationKeyPath ~/.appstoreconnect/.../AuthKey_<ID>.p8
               -authenticationKeyID <KEY_ID>
               -authenticationKeyIssuerID <ISSUER_ID>
               -allowProvisioningUpdates
               (creates + downloads distribution profile automatically)
           → xcodebuild -exportArchive
               ExportOptions.plist: method=app-store-connect
        |
        v
  IPA at: gen/apple/build/arm64/HP-41 Calculator.ipa
        |
        v
  xcrun altool --upload-package <IPA>
    --api-key <KEY_ID> --api-issuer <ISSUER_ID>
    (reads .p8 from ~/.appstoreconnect/private_keys/ automatically)
        |
        v
  Secure cleanup: rm -rf ~/.appstoreconnect/private_keys/
        |
        v
  [App Store Connect] → TestFlight internal build available
```

### Recommended Project File Changes

```
hp41-gui/src-tauri/gen/apple/
├── PrivacyInfo.xcprivacy          [NEW — SHIP-02]
├── project.yml                    [EDIT — add PrivacyInfo source entry]
├── hp41-gui_iOS/Info.plist        [EDIT — update CFBundleShortVersionString to "4.1"]
├── ExportOptions.plist            [EDIT — method: app-store-connect]
├── LaunchScreen.storyboard        [EDIT — brand with HP-41 identity — SHIP-03]
├── Assets.xcassets/AppIcon.appiconset/
│   └── [all PNG slots]            [REPLACE — new HP-41-styled icons — SHIP-03]
.github/workflows/
└── ci-ios.yml                     [NEW — SHIP-04]
justfile                           [EDIT — add ios-build-release recipe]
```

### Pattern 1: ASC API Key — Tauri CLI Environment Variables

Tauri CLI 2.11.x reads three env vars for automatic signing on CI:

```bash
# Source: https://v2.tauri.app/distribute/sign/ios/ [CITED]
export APPLE_API_KEY="ABC123DEF456"           # Key ID (not the .p8 content)
export APPLE_API_ISSUER="57246542-96fe-1a63-e053-0824d011072a"
export APPLE_API_KEY_PATH="$HOME/.appstoreconnect/private_keys/AuthKey_${APPLE_API_KEY}.p8"

cd hp41-gui
npm run tauri ios build -- --export-method app-store-connect --ci
```

These are forwarded by Tauri CLI to `xcodebuild` as:
```
-authenticationKeyID <APPLE_API_KEY>
-authenticationKeyIssuerID <APPLE_API_ISSUER>
-authenticationKeyPath <APPLE_API_KEY_PATH>
-allowProvisioningUpdates
```

[CITED: v2.tauri.app/distribute/sign/ios/]

### Pattern 2: .p8 Key File Placement and Cleanup

```bash
# Write .p8 from base64 GitHub secret
mkdir -p "$HOME/.appstoreconnect/private_keys"
echo "${{ secrets.ASC_API_KEY_P8 }}" | base64 --decode \
  > "$HOME/.appstoreconnect/private_keys/AuthKey_${{ secrets.ASC_KEY_ID }}.p8"
chmod 600 "$HOME/.appstoreconnect/private_keys/AuthKey_${{ secrets.ASC_KEY_ID }}.p8"

# ... build + upload ...

# Cleanup — always runs (use `if: always()` or trap)
rm -rf "$HOME/.appstoreconnect/private_keys/"
```

`altool` automatically searches `~/.appstoreconnect/private_keys/AuthKey_<KEYID>.p8` when given `--api-key <KEYID>`. No explicit `--p8-file-path` flag needed if the file is placed there. [VERIFIED: `xcrun altool --help` output confirms search paths]

### Pattern 3: Build Number Injection via PlistBuddy

```bash
# Run BEFORE tauri ios build (before the archive step reads Info.plist)
# Target: the committed Info.plist that Xcode reads via project.yml info.path
BUILD_NUM="${{ github.run_number }}"
/usr/libexec/PlistBuddy \
  -c "Set :CFBundleVersion $BUILD_NUM" \
  hp41-gui/src-tauri/gen/apple/hp41-gui_iOS/Info.plist
# Never commit this change back (CI runner is ephemeral)
```

**Important:** The `tauri ios build --build-number <N>` flag exists but is described as "append to the app version" — behavior is ambiguous for standalone integer build numbers. PlistBuddy on the tracked `Info.plist` provides deterministic control with no ambiguity. [ASSUMED: `--build-number` behavior undocumented precisely; PlistBuddy approach is conventional and verified available on macOS runners]

### Pattern 4: TestFlight Upload via altool

```bash
# Verified syntax from `xcrun altool --help` on Xcode 26.5 [VERIFIED]
IPA="hp41-gui/src-tauri/gen/apple/build/arm64/HP-41 Calculator.ipa"

xcrun altool \
  --upload-package "$IPA" \
  --api-key "${{ secrets.ASC_KEY_ID }}" \
  --api-issuer "${{ secrets.ASC_ISSUER_ID }}"
# .p8 is read automatically from ~/.appstoreconnect/private_keys/AuthKey_<KEY_ID>.p8
```

**Note on `--upload-app` vs `--upload-package`:** Both forms exist in Xcode 26.5 altool. `--upload-package` is the current/preferred form per altool help examples. `--upload-app -f <file>` is the legacy form. The Tauri docs show `--upload-app --type ios --file <ipa>` — this also works but `--upload-package` is cleaner and the `--type ios` flag is not required for `.ipa` files. [VERIFIED: `xcrun altool --help`]

**Note on altool deprecation:** altool is deprecated for macOS *notarization* only (replaced by `notarytool`). For iOS TestFlight uploads, `altool` remains the correct tool and is NOT deprecated for this use case. [CITED: Apple TN3147, fastlane discussion #21347]

### Pattern 5: ExportOptions.plist for Distribution

```xml
<!-- gen/apple/ExportOptions.plist — edit in place, committed -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" ...>
<plist version="1.0">
<dict>
    <key>method</key>
    <string>app-store-connect</string>   <!-- was: debugging -->
    <key>signingStyle</key>
    <string>automatic</string>
    <key>teamID</key>
    <string>2P4R8QSWT4</string>
</dict>
</plist>
```

**`method` value:** `app-store-connect` is the current non-deprecated value in Xcode 26. `app-store` is deprecated (per `xcodebuild -help`: "deprecated: use app-store-connect"). [VERIFIED: `xcodebuild -help` output]

**Important:** `tauri ios build --export-method app-store-connect` writes this plist before calling xcodebuild, so the committed `ExportOptions.plist` is overwritten by Tauri CLI at build time. Editing the committed file mainly serves as documentation / fallback for direct xcodebuild runs. The `--export-method` flag is the authoritative source for CI.

### Pattern 6: PrivacyInfo.xcprivacy

```xml
<!-- gen/apple/PrivacyInfo.xcprivacy — new file -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
          "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>NSPrivacyAccessedAPITypes</key>
    <array>
        <dict>
            <key>NSPrivacyAccessedAPIType</key>
            <string>NSPrivacyAccessedAPICategoryFileTimestamp</string>
            <key>NSPrivacyAccessedAPITypeReasons</key>
            <array>
                <string>C617.1</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
```

[CITED: developer.apple.com (Apple privacy manifest) + multiple verified sources]

**C617.1 reason justification:** "Access the timestamps, size, or other metadata of files inside the app container, app group container, or the app's CloudKit container." This matches the autosave file I/O in `Library/Application Support/ch.talent-factory.hp41/autosave.json`. [CITED: Apple developer docs]

### Pattern 7: Adding PrivacyInfo.xcprivacy to Xcode Project via project.yml

XcodeGen (which Tauri uses for the iOS Xcode project) requires the privacy manifest to be listed as a source. However, there is a known XcodeGen issue (#1459) where `.xcprivacy` files get incorrectly added to Copy Bundle Resources, causing duplicate errors. PR #1464 fixed this. The correct `project.yml` addition:

```yaml
# In the hp41-gui_iOS target's sources list, add:
targets:
  hp41-gui_iOS:
    sources:
      - path: Sources
      - path: Assets.xcassets
      - path: Externals
      - path: hp41-gui_iOS
      - path: assets
        buildPhase: resources
        type: folder
      - path: LaunchScreen.storyboard
      - path: PrivacyInfo.xcprivacy   # ADD THIS — XcodeGen handles .xcprivacy natively
```

**Alternative if XcodeGen version has the bug:** Add the file directly to `project.pbxproj` as a resource reference. Since `gen/apple/` is tracked, this edit is durable. The safer option is to add it to `project.yml` sources and regenerate, or edit the `.pbxproj` directly. [MEDIUM confidence — depends on XcodeGen version bundled with Tauri 2.11.1]

### Pattern 8: GitHub Actions Workflow Structure (`ci-ios.yml`)

```yaml
name: ci-ios

on:
  workflow_dispatch:

jobs:
  build-and-upload:
    runs-on: macos-26    # ARM64 runner; has Rust 1.95, Xcode 26.x, Node 22/24
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-apple-ios

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: hp41-gui/src-tauri -> hp41-gui/src-tauri/target

      - run: rustup default stable   # Rebind cargo proxy post-cache (pattern from ci-gui.yml)

      - uses: actions/setup-node@v4
        with:
          node-version: 'lts/*'

      - uses: taiki-e/install-action@v2
        with:
          tool: just

      - name: Inject build number
        run: |
          /usr/libexec/PlistBuddy \
            -c "Set :CFBundleVersion ${{ github.run_number }}" \
            hp41-gui/src-tauri/gen/apple/hp41-gui_iOS/Info.plist

      - name: Write ASC API key
        run: |
          mkdir -p "$HOME/.appstoreconnect/private_keys"
          echo "${{ secrets.ASC_API_KEY_P8 }}" | base64 --decode \
            > "$HOME/.appstoreconnect/private_keys/AuthKey_${{ secrets.ASC_KEY_ID }}.p8"
          chmod 600 \
            "$HOME/.appstoreconnect/private_keys/AuthKey_${{ secrets.ASC_KEY_ID }}.p8"

      - name: Build signed IPA
        env:
          APPLE_API_KEY: ${{ secrets.ASC_KEY_ID }}
          APPLE_API_ISSUER: ${{ secrets.ASC_ISSUER_ID }}
          APPLE_API_KEY_PATH: ${{ format('{0}/.appstoreconnect/private_keys/AuthKey_{1}.p8', runner.home, secrets.ASC_KEY_ID) }}
        run: just ios-build-release

      - name: Upload to TestFlight
        run: |
          IPA="hp41-gui/src-tauri/gen/apple/build/arm64/HP-41 Calculator.ipa"
          xcrun altool \
            --upload-package "$IPA" \
            --api-key "${{ secrets.ASC_KEY_ID }}" \
            --api-issuer "${{ secrets.ASC_ISSUER_ID }}"

      - name: Cleanup API key
        if: always()
        run: rm -rf "$HOME/.appstoreconnect/private_keys/"
```

**GitHub Secrets required (3 total):**
| Secret Name | Content |
|-------------|---------|
| `ASC_KEY_ID` | Key ID string (e.g. `ABC123DEF456`) |
| `ASC_ISSUER_ID` | Issuer ID UUID (e.g. `57246542-96fe-1a63-e053-0824d011072a`) |
| `ASC_API_KEY_P8` | Base64-encoded contents of `AuthKey_<ID>.p8` (`base64 -i AuthKey_XXX.p8`) |

**New `just` recipe (`ios-build-release`):**
```makefile
# iOS: signed release IPA for App Store Connect (distribution) distribution
# Passes --export-method app-store-connect and --ci (no interactive prompts)
# Requires APPLE_API_KEY, APPLE_API_ISSUER, APPLE_API_KEY_PATH env vars for signing.
[group('ios')]
ios-build-release:
    cd hp41-gui && npm ci
    cd hp41-gui && npm run tauri ios build -- --export-method app-store-connect --ci
```

### Pattern 9: App Icon — `cargo tauri icon`

```bash
# From hp41-gui/ working directory
# Source: https://v2.tauri.app/develop/icons/ [CITED]
# Input: a squared PNG (1024×1024, no transparency recommended for iOS)
# Output: updates gen/apple/Assets.xcassets/AppIcon.appiconset/ automatically

cd hp41-gui
npm run tauri icon -- ../app-icon-1024.png --ios-color "#1A1A1A"
# Tauri places mobile icons into the Xcode project directly
```

**Known bug:** Tauri issue #11578 ("tauri-cli does not update all ios icons") was closed as "not planned." Some slots may be missed. After running `cargo tauri icon`, manually verify all slots in `Contents.json` have corresponding PNG files. [CITED: github.com/tauri-apps/tauri/issues/11578]

**Required slots** (from existing `Contents.json` — 19 entries):
```
iPhone: 20@2x, 20@3x, 29@2x, 29@3x, 40@2x, 40@3x, 60@2x, 60@3x
iPad:   20@1x, 20@2x, 29@1x, 29@2x, 40@1x, 40@2x, 76@1x, 76@2x, 83.5@2x
Marketing: 1024×1024 (stored as AppIcon-512@2x.png, idiom: ios-marketing, scale: 1x)
```

**iOS icon must NOT have transparency** (App Store Connect rejects transparent icons). Use `--ios-color` to set a solid background.

### Pattern 10: LaunchScreen.storyboard Branding

The current `LaunchScreen.storyboard` has a plain white background with `systemBackgroundColor`. For HP-41-styled branding:

```xml
<!-- Change the background color to HP-41 dark tone -->
<color key="backgroundColor" red="0.102" green="0.102" blue="0.102" alpha="1" colorSpace="custom" customColorSpace="sRGB"/>
<!-- Add a centered UIImageView with the app icon or calculator silhouette -->
<!-- Add a centered UILabel with "HP-41" text in calculator font -->
```

The storyboard can be edited as XML directly (committed file, no Xcode needed for simple changes) or via Xcode Interface Builder. [ASSUMED: specific HP-41 art direction TBD per D-57.7]

### Anti-Patterns to Avoid

- **Do not use `--upload-app --type ios`** as the primary form — use `--upload-package` (newer altool form per current examples). Both work but `--upload-package` is the current example form.
- **Do not commit the PlistBuddy-modified Info.plist** — the CI step is ephemeral; the committed file has `CFBundleVersion: "1.0.0"` which is the development baseline.
- **Do not base64-encode the .p8 file with line breaks** — use `base64 -i <file>` (no `-w 0` needed on macOS; macOS `base64` outputs no line breaks by default). On Linux `base64 --wrap=0` is needed.
- **Do not write the .p8 to the working directory** — use `~/.appstoreconnect/private_keys/` so both xcodebuild and altool find it automatically.
- **Do not use `macos-latest`** if it has reverted to macOS 15 — use `macos-26` explicitly for Xcode 26.x availability. The default Xcode on `macos-26` is 26.4.1 as of 2026-05-18; 26.5 is also available.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Certificate/profile fetching on CI | Custom keychain import script | ASC API key + `-allowProvisioningUpdates` | Avoids P-iOS-16; Xcode manages cert/profile lifecycle automatically |
| App icon resizing | Shell script with ImageMagick | `cargo tauri icon` + manual slot verification | Handles 19 required slots + `Contents.json` correctly |
| Privacy manifest inclusion | Manually edit `.pbxproj` entries | XcodeGen `project.yml` source entry | Keeps the project.yml as single source of truth |
| Build number tracking | Manual commit + increment | `github.run_number` + PlistBuddy | Monotonic, zero bookkeeping, never resets |
| TestFlight upload | Custom HTTPS calls to ASC API | `xcrun altool --upload-package` | Handles validation, chunking, retries |
| IPA signing | Manual `codesign` invocations | Xcode automatic signing + API key | Automatic signing creates Distribution cert if missing |

---

## Runtime State Inventory

> Skipped — this is a greenfield CI pipeline + new asset phase. No renaming/refactoring. No runtime state to audit.

None — verified by phase description (adds new files only: `ci-ios.yml`, `PrivacyInfo.xcprivacy`, icon assets, updated storyboard).

---

## Common Pitfalls

### Pitfall 1: `method: app-store-connect` rejected by older Xcode
**What goes wrong:** Tauri issue #13818 — older Xcode versions reject `app-store-connect` in `ExportOptions.plist` (expected `app-store`). The roles are now reversed: `app-store` is the deprecated value.
**Why it happens:** The allowed set changed across Xcode major versions.
**How to avoid:** Xcode 26 (used on macos-26 runner) accepts `app-store-connect` per `xcodebuild -help`. If the workflow runs on an older runner image, the error message will clearly state the allowed values. [VERIFIED: Xcode 26.5 `xcodebuild -help`]
**Warning signs:** `exportOptionsPlist error for key "method": expected one of {…}`

### Pitfall 2: lswiftCompatibility56 linker error (Tauri + Xcode 26)
**What goes wrong:** Tauri issue #15066 — building iOS apps with Xcode 26 failed with `ld: library not found for -lswiftCompatibility56` in Tauri 2.9.5.
**Why it happens:** Xcode 26 removed Swift backward-compatibility libraries.
**How to avoid:** **This issue does NOT affect Tauri 2.11.1 + Xcode 26.5** — confirmed by `Packaging.log` from the local development build (2026-06-03). The CI pipeline can safely use `macos-26` with default Xcode. [VERIFIED: local Packaging.log shows successful archive]
**Warning signs:** If the linker error appears on CI, pin to `Xcode 26.4` via `xcode-select` as a fallback while filing a Tauri bug.

### Pitfall 3: P-iOS-16 — Keychain Access on Headless macOS CI Runners
**What goes wrong:** Importing a P12 certificate into a temporary keychain on GitHub Actions triggers interactive unlock prompts that hang the job. The `security import` / `security set-key-partition-list` dance is brittle and Xcode occasionally cannot access the keychain even when set up correctly.
**Why it happens:** GitHub Actions runners run non-interactively; keychain ACLs require UI confirmation in some paths.
**How to avoid:** D-57.2 locks this out entirely. The ASC API key + `-allowProvisioningUpdates` approach bypasses the keychain for the archive step. No P12, no `.mobileprovision`, no keychain gymnastics.
**Warning signs:** Any step calling `security create-keychain` or `security import`.

### Pitfall 4: .p8 Key File Search Path Mismatch
**What goes wrong:** altool (and xcodebuild) searches for the key file at `~/.appstoreconnect/private_keys/AuthKey_<KEYID>.p8`. If the key ID in the filename doesn't match `ASC_KEY_ID`, the tool cannot find the key.
**Why it happens:** Typos in the key ID when constructing the filename.
**How to avoid:** Use `echo "${{ secrets.ASC_KEY_ID }}"` in the path construction:
  ```bash
  KEY_FILE="$HOME/.appstoreconnect/private_keys/AuthKey_${ASC_KEY_ID}.p8"
  ```
  Verify the key ID (not the issuer ID) is exactly what appears in App Store Connect Integrations.
**Warning signs:** `altool: Error: Unable to find authentication key.`

### Pitfall 5: TestFlight Build Number Must Be Strictly Increasing
**What goes wrong:** TestFlight rejects builds where `CFBundleVersion` is not strictly greater than the previous build's version for the same `CFBundleShortVersionString`.
**Why it happens:** Apple enforces a monotonic build number per marketing version.
**How to avoid:** `github.run_number` never resets and always increases within the repository lifetime. D-57.5 is correctly designed.
**Warning signs:** `ERROR ITMS-90189: Redundant Binary Upload`

### Pitfall 6: `PrivacyInfo.xcprivacy` Not Included in Bundle
**What goes wrong:** The file exists in `gen/apple/` but is not referenced in the Xcode project, so it does not ship inside the `.app` bundle. Apple sends ITMS-91053 email after upload.
**Why it happens:** XcodeGen requires explicit source listing in `project.yml`; placing a file on disk is not enough.
**How to avoid:** Add `path: PrivacyInfo.xcprivacy` to the `hp41-gui_iOS` target sources in `project.yml`. Then regenerate the Xcode project (`npm run tauri ios init`) OR edit `project.pbxproj` directly to add the file reference and resource build phase entry.
**Warning signs:** ITMS-91053 email from Apple after first TestFlight upload.

### Pitfall 7: IPA Path Contains Spaces
**What goes wrong:** The IPA is at `gen/apple/build/arm64/HP-41 Calculator.ipa` (space in app name). Unquoted shell variables cause the path to be split.
**Why it happens:** `PRODUCT_NAME: HP-41 Calculator` in project.yml generates a space-containing app name.
**How to avoid:** Always quote the IPA path in shell scripts:
  ```bash
  IPA="hp41-gui/src-tauri/gen/apple/build/arm64/HP-41 Calculator.ipa"
  xcrun altool --upload-package "$IPA" ...
  ```
**Warning signs:** `File does not exist at path` error despite file existing.

### Pitfall 8: `APPLE_API_KEY_PATH` env var required for iOS (not optional)
**What goes wrong:** On macOS desktop builds, `APPLE_API_KEY_PATH` is optional (altool searches standard dirs). But the Tauri iOS signing documentation explicitly states: "For iOS this variable is required."
**Why it happens:** The xcodebuild `-authenticationKeyPath` flag must be explicit for iOS archive jobs.
**How to avoid:** Always set `APPLE_API_KEY_PATH` explicitly in the CI env, pointing to the written `.p8` file. [CITED: v2.tauri.app/reference/environment-variables/]

### Pitfall 9: P-iOS-18 — Works in Simulator, Fails on Device/TestFlight (5 Root Causes)
Pre-flight checklist before first upload:

| Root cause | Detection | Fix |
|---|---|---|
| Wrong provisioning profile type (development vs. distribution) | "Invalid Signature" in TestFlight | Ensure `--export-method app-store-connect` creates a distribution IPA; check DistributionSummary.plist shows "Apple Distribution" certificate |
| Entitlements mismatch | "Invalid entitlements" on device | Match `hp41-gui_iOS.entitlements` to App ID capabilities in App Store Connect |
| Missing `PrivacyInfo.xcprivacy` | ITMS-91053 email from Apple | Add manifest with `NSPrivacyAccessedAPICategoryFileTimestamp` / `C617.1` (SHIP-02) |
| `DEVELOPMENT_TEAM` not set | "No signing certificate" | Already set to `2P4R8QSWT4` in `project.yml`; verify survives `tauri ios init` re-run |
| Architecture mismatch | Crash on launch | Ensure `aarch64-apple-ios` (device) not `aarch64-apple-ios-sim`; `tauri ios build` defaults to `aarch64` (correct) |

### Pitfall 10: altool Selecting Wrong App (Multiple Similar Bundle IDs)
**What goes wrong:** Xcode 26 altool may select the wrong app if the App Store Connect account has multiple apps with similar bundle IDs (fastlane issue #29743, #29698).
**Why it happens:** Altered app resolution logic in Xcode 26's avtool.
**How to avoid:** This project has only one app (`ch.talent-factory.hp41`) — not an issue for the current account. If the error occurs anyway, add `--apple-id <numeric-app-id>` to the altool command.
**Warning signs:** Upload appears to succeed but appears under the wrong app in App Store Connect.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Xcode 26.x | IPA archive + export | ✓ (macos-26 runner) | 26.4.1 default, 26.5 available | Pin to 26.4.1 via `xcode-select` if needed |
| `xcrun altool` | TestFlight upload | ✓ (Xcode-bundled) | 26.40.1 on local Xcode 26.5 | — |
| `/usr/libexec/PlistBuddy` | CFBundleVersion injection | ✓ (macOS built-in) | macOS-bundled | — |
| `aarch64-apple-ios` Rust target | Rust iOS compile | ✓ (installed locally; must add on runner) | Rust 1.95 on macos-26 runner | `rustup target add aarch64-apple-ios` in workflow |
| `just` | Wrapping build recipes | ✓ | 1.49.0 locally; `taiki-e/install-action@v2` on CI | — |
| Node.js LTS | npm ci + Vite build | ✓ | v22 or v24 on macos-26 | — |
| ASC API key (.p8) | Signing + upload | Must be created by user | — | Prerequisite; blocks SHIP-01/04/05 |
| App Store Connect App Record | TestFlight upload target | ✓ (registered in Phase 53-02) | — | — |

**Missing dependencies with no fallback:**
- ASC API key (`.p8` + Key ID + Issuer ID) — must be created by the user in App Store Connect → Users and Access → Integrations before running the CI pipeline. This is an operational prerequisite, not a code task.

**Missing dependencies with fallback:**
- None — all CI tooling is available on `macos-26`.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| P12 + mobileprovision keychain import | ASC API key + `-allowProvisioningUpdates` | ~Xcode 13 (2021) | Eliminates P-iOS-16; Apple manages certs automatically |
| `xcrun altool --upload-app --type ios` | `xcrun altool --upload-package` | Xcode 26 (preferred form) | Both work; `--upload-package` is the current example form |
| `method: app-store` in ExportOptions.plist | `method: app-store-connect` | Xcode 15+ | `app-store` is now deprecated; `app-store-connect` is canonical |
| `xcrun notarytool` | Only replaces altool for macOS *notarization* | Xcode 13 (2021) | For iOS TestFlight uploads, altool is still correct |
| fastlane `match` | ASC API key automatic signing | — | match adds complexity; API key + automatic signing is simpler for small teams |

**Deprecated/outdated:**
- `method: app-store` in ExportOptions.plist: deprecated, use `app-store-connect`
- `method: ad-hoc`: deprecated, use `release-testing`
- `method: development`: deprecated, use `debugging`
- `xcrun notarytool` for iOS uploads: not applicable (notarytool is macOS notarization only)

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `cargo tauri icon` updates all 19 AppIcon slots automatically | Standard Stack / Pattern 9 | Some slots may be missed (known issue #11578); manual slot verification is the mitigation |
| A2 | `tauri ios build --build-number` behavior for plain integer injection | Pattern 3 | If `--build-number 42` produces `1.0.0.42` instead of `42`, PlistBuddy approach is required (recommended anyway) |
| A3 | `macos-26` runner's default Xcode (26.4.1) is compatible with Tauri 2.11.1 | Environment / Pitfall 2 | If lswiftCompatibility56 re-emerges on 26.4.1 vs 26.5, pin Xcode version in CI |
| A4 | Adding `path: PrivacyInfo.xcprivacy` to project.yml sources works cleanly in the XcodeGen version bundled with Tauri 2.11.1 | Pattern 7 | If XcodeGen has bug #1459, direct `project.pbxproj` edit is the fallback |
| A5 | `APPLE_API_KEY_PATH` env var must be set explicitly (not relying on altool's directory search) | Pattern 8 | If omitted, xcodebuild may fail to authenticate; explicit path is safer |
| A6 | `github.run_number` satisfies Apple's strictly-increasing requirement per marketing version | Pattern 3 / D-57.5 | If the team previously submitted builds manually with higher build numbers, `run_number` might not be strictly greater; reset risk on repo fork |

---

## Open Questions

1. **ROADMAP vs CONTEXT trigger conflict (D-57.4)**
   - What we know: CONTEXT.md says `workflow_dispatch` only; ROADMAP SC #4 says "on every push to main"
   - What's unclear: Which should be the verification criterion for `/gsd:verify-phase`
   - Recommendation: Write plan for `workflow_dispatch`, document that ROADMAP SC #4 needs updating, let user confirm

2. **PrivacyInfo.xcprivacy XcodeGen inclusion method**
   - What we know: XcodeGen has a known bug with `.xcprivacy` in some versions (fixed in PR #1464)
   - What's unclear: Which XcodeGen version is bundled with Tauri 2.11.1
   - Recommendation: Plan should include a fallback: if XcodeGen project.yml approach fails, add the file reference directly to `project.pbxproj`

3. **App Store Connect API key permissions level**
   - What we know: Tauri docs say "Admin access"; Apple docs say "Developer" is sufficient for uploads
   - What's unclear: Whether Developer-level key can also create/update provisioning profiles for `-allowProvisioningUpdates`
   - Recommendation: Create the key with App Manager or Admin access to be safe; least-privilege can be revisited

4. **HP-41 icon art direction**
   - What we know: D-57.7 says "real HP-41-styled icon, 1024×1024 master, gold-on-dark aesthetic"
   - What's unclear: Specific visual elements (is it the calculator face? the logo? a stylized "41"?)
   - Recommendation: Planner should add a checkpoint task for the user to approve icon design before it's committed

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Manual verification + CI pipeline execution |
| Config file | `.github/workflows/ci-ios.yml` (new) |
| Quick run command | `workflow_dispatch` in GitHub Actions UI |
| Full suite command | Same (single workflow) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SHIP-01 | Signed IPA produced with distribution cert | manual | Inspect `DistributionSummary.plist` — cert type must be "Apple Distribution" | ❌ Wave 0 |
| SHIP-01 | IPA installs on real iPhone | manual | `xcrun devicectl device install app --device <id> <ipa>` | ❌ (manual checkpoint) |
| SHIP-02 | `PrivacyInfo.xcprivacy` present in bundle | manual/smoke | `unzip -p "HP-41 Calculator.ipa" "Payload/HP-41 Calculator.app/PrivacyInfo.xcprivacy" | head` | ❌ Wave 0 |
| SHIP-02 | No ITMS-91053 after upload | manual | Upload to TestFlight; check App Store Connect for emails | — (upload-time) |
| SHIP-03 | App icon in all required slots | manual | `ls gen/apple/Assets.xcassets/AppIcon.appiconset/*.png \| wc -l` → expect 19 | ❌ Wave 0 |
| SHIP-03 | Launch screen is branded (not blank white) | manual | Install IPA on device; observe launch | — (visual) |
| SHIP-04 | `ci-ios.yml` workflow runs and produces IPA | manual | Trigger `workflow_dispatch`; verify artifact in Actions run | ❌ Wave 0 |
| SHIP-04 | Upload step exits 0 | smoke | CI step exit code | ❌ Wave 0 |
| SHIP-05 | Build visible in TestFlight internal group | manual | Open App Store Connect → TestFlight → Internal Testing | — (portal) |
| SHIP-05 | App installs from TestFlight on real iPhone | manual | Download from TestFlight; launch | — (device) |

### IPA Signature Verification (SHIP-01 smoke)

```bash
# Verify IPA is signed for distribution (not development):
unzip -p "hp41-gui/src-tauri/gen/apple/build/arm64/HP-41 Calculator.ipa" \
  "Payload/HP-41 Calculator.app/embedded.mobileprovision" \
  | security cms -D -i /dev/stdin \
  | grep -A2 "ProvisionsAllDevices\|ProvisionedDevices\|ProvisionsAllDevices"
# Distribution profile: ProvisionsAllDevices = true (app-store) OR no ProvisionedDevices key
```

### Wave 0 Gaps

- [ ] `ci-ios.yml` — the workflow file itself (the main deliverable)
- [ ] `gen/apple/PrivacyInfo.xcprivacy` — SHIP-02
- [ ] Updated `gen/apple/project.yml` — PrivacyInfo source entry
- [ ] Updated `gen/apple/ExportOptions.plist` — method: app-store-connect
- [ ] Updated `gen/apple/hp41-gui_iOS/Info.plist` — CFBundleShortVersionString: "4.1"
- [ ] App icon master (1024×1024 PNG) — design artifact; prerequisite for SHIP-03
- [ ] All 19 AppIcon slots regenerated — SHIP-03
- [ ] Branded `LaunchScreen.storyboard` — SHIP-03
- [ ] New `ios-build-release` just recipe

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes (CI secrets) | GitHub repo secrets (encrypted at rest); never echo in logs |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | no | — |
| V6 Cryptography | yes (API key .p8) | RSA private key stored as base64 secret; written to disk with mode 600; deleted after use |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| .p8 key exposed in CI logs | Information Disclosure | Never `cat` the .p8 or echo it; use `base64 --decode` directly to file |
| Workflow injection via run_number | Tampering | `github.run_number` is a numeric integer from GitHub Actions context; not user-controlled |
| .p8 persists on runner after failure | Information Disclosure | `if: always()` cleanup step; GitHub Actions hosted runners are ephemeral (destroyed after run) |
| Secret exfiltration via malicious workflow trigger | Spoofing | `workflow_dispatch` only; no third-party PR triggers; no external contributors can trigger |

---

## Sources

### Primary (HIGH confidence)

- `xcrun altool --help` on Xcode 26.5 — verified flags: `--upload-package`, `--api-key`, `--api-issuer`, `--p8-file-path`, search paths for `AuthKey_<ID>.p8`
- `xcodebuild -help` on Xcode 26.5 — verified: method `app-store-connect` (current), `app-store` (deprecated); `-authenticationKeyPath`, `-authenticationKeyID`, `-authenticationKeyIssuerID`, `-allowProvisioningUpdates` flags
- `hp41-gui/src-tauri/gen/apple/build/Packaging.log` (2026-06-03) — local Tauri 2.11.1 + Xcode 26.5 build completed to valid IPA (confirms no lswiftCompatibility56 issue)
- `hp41-gui/src-tauri/gen/apple/build/DistributionSummary.plist` — confirms IPA at `gen/apple/build/arm64/HP-41 Calculator.ipa`
- [Tauri iOS Code Signing docs](https://v2.tauri.app/distribute/sign/ios/) — env vars `APPLE_API_KEY`, `APPLE_API_ISSUER`, `APPLE_API_KEY_PATH`
- [Tauri iOS App Store docs](https://v2.tauri.app/distribute/app-store/) — upload command `xcrun altool --upload-app --type ios --file ...`
- [Tauri CLI reference](https://v2.tauri.app/reference/cli/) — `--export-method app-store-connect`, `--build-number`, `--ci` flags
- [Tauri Environment Variables](https://v2.tauri.app/reference/environment-variables/) — `APPLE_API_KEY_PATH` required for iOS
- [Tauri App Icons](https://v2.tauri.app/develop/icons/) — `cargo tauri icon` iOS slot placement
- [GitHub Actions macos-26 arm64 README](https://github.com/actions/runner-images/blob/main/images/macos/macos-26-arm64-Readme.md) — Rust 1.95, Xcode 26.4.1 default, Node.js 22/24

### Secondary (MEDIUM confidence)

- [Tauri iOS issue #13818](https://github.com/tauri-apps/tauri/issues/13818) — `app-store-connect` method compatibility issue in older Xcode versions; not applicable to Xcode 26
- [Tauri iOS issue #15066](https://github.com/tauri-apps/tauri/issues/15066) — lswiftCompatibility56 issue in Tauri 2.9.5; confirmed not present in local Tauri 2.11.1 + Xcode 26.5 build
- [Tauri iOS issue #11578](https://github.com/tauri-apps/tauri/issues/11578) — `cargo tauri icon` may miss some iOS slots; closed "not planned"
- [XcodeGen issue #1459](https://github.com/yonaskolb/XcodeGen/issues/1459) — PrivacyInfo.xcprivacy handling; fixed in PR #1464
- [Apple privacy manifest format](https://dev.to/aishanipach/include-nsprivacyaccessedapicategorydiskspace-nsprivacyaccessedapicategoryfiletimestamp-information-59hi) — NSPrivacyAccessedAPICategoryFileTimestamp / C617.1 XML structure
- [fastlane discussion #21347](https://github.com/fastlane/fastlane/discussions/21347) — altool deprecation scope (notarization only, not TestFlight uploads)
- [fastlane issue #29743](https://github.com/fastlane/fastlane/issues/29743) — altool app selection bug on Xcode 26; mitigated by single-app account
- [macos-26 GA announcement](https://github.blog/changelog/2026-02-26-macos-26-is-now-generally-available-for-github-hosted-runners/) — runner availability
- [GitHub Actions runner image #14001](https://github.com/actions/runner-images/issues/14001) — Xcode 26.4.1 as new default on macos-26

### Tertiary (LOW confidence)

- [VPSMAC iOS CI blog](https://vpsmac.com/en/blog/mac-cloud-ios-ci-signing-keychain-headless-xcodebuild-2026.html) — general patterns; limited actionable detail

---

## Project Constraints (from CLAUDE.md)

- **`just` is the sole task runner** — CI must invoke `just ios-build-release`, not raw `cargo tauri` or `npm run tauri` directly in CI `run:` steps (or wrap in a just recipe).
- **Never call `cargo` directly in CI** — all cargo invocations happen inside `just` recipes.
- **`gen/apple/` is git-tracked** — all edits (ExportOptions.plist, project.yml, Info.plist, PrivacyInfo.xcprivacy, icon assets, LaunchScreen.storyboard) are committed and durable; CI does not regenerate the scaffold.
- **Frozen Invariant** — root `Cargo.toml` members stay `["hp41-core", "hp41-cli"]`; `tauri` confined to `hp41-gui/src-tauri/Cargo.toml`; `hp41-core` is frozen (no engine changes).
- **4-way exhaustive-match invariant** — not applicable (no new `Op` variants this phase).
- **Commit language: English** — all commit messages in English.
- **Milestone PR merge: always `--merge`** (relevant when closing v4.1 after this phase).

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all tooling verified via local `xcrun altool --help`, `xcodebuild -help`, Packaging.log
- Architecture: HIGH — complete flow derived from verified tool invocations and Tauri official docs
- Pitfalls: HIGH — P-iOS-16/18/19 from accumulated STATE.md context + tool verification
- App icon: MEDIUM — `cargo tauri icon` behavior verified from docs but #11578 bug is "not planned"
- PrivacyInfo XcodeGen inclusion: MEDIUM — depends on XcodeGen version; fallback documented

**Research date:** 2026-06-04
**Valid until:** 2026-09-01 (Tauri and Xcode APIs are stable; review if either major-versions)

---

## RESEARCH COMPLETE

**Phase:** 57 - Signing + TestFlight Pipeline
**Confidence:** HIGH

### Key Findings

1. **lswiftCompatibility56 is NOT an issue** — Tauri 2.11.1 + Xcode 26.5 builds to a valid IPA locally (Packaging.log confirmed). The CI pipeline can use `macos-26` with default Xcode 26.4.1+ without a version-lock workaround.

2. **Exact CI invocation is clear** — `APPLE_API_KEY` / `APPLE_API_ISSUER` / `APPLE_API_KEY_PATH` env vars are read by Tauri CLI and forwarded to `xcodebuild -authenticationKeyPath ... -allowProvisioningUpdates`. No keychain import needed.

3. **IPA path has a space** — `gen/apple/build/arm64/HP-41 Calculator.ipa` must always be quoted in shell scripts. This is easy to miss and will cause cryptic "file not found" errors.

4. **`method: app-store-connect` is correct for Xcode 26** — `app-store` is the deprecated value. This is the opposite of what issue #13818 (older Xcode) documented.

5. **ROADMAP vs CONTEXT conflict (D-57.4)** — ROADMAP Success Criterion #4 says "on every push to main" but D-57.4 locks to `workflow_dispatch` only. CONTEXT.md is authoritative; ROADMAP SC #4 needs correction. Planner must surface this before writing the plan.

### File Created

`.planning/phases/57-signing-testflight-pipeline/57-RESEARCH.md`

### Confidence Assessment

| Area | Level | Reason |
|------|-------|--------|
| Standard Stack | HIGH | All tools verified via local Xcode 26.5 and Tauri 2.11.1 |
| Architecture | HIGH | Complete flow derived from verified tool flags + official Tauri docs |
| Pitfalls | HIGH | P-iOS-16/18/19 from STATE.md + tool verification |
| App icon | MEDIUM | `cargo tauri icon` docs verified; slot bug (#11578) is known |
| PrivacyInfo XcodeGen | MEDIUM | XcodeGen version-dependent; fallback documented |

### Open Questions

- Trigger conflict (D-57.4 vs ROADMAP SC #4) — needs user resolution before plan is locked
- ASC API key permissions level (Admin vs App Manager vs Developer)
- HP-41 icon visual art direction (blocking SHIP-03 design work)

### Ready for Planning

Research complete. Planner can now create PLAN.md files for Phase 57.
