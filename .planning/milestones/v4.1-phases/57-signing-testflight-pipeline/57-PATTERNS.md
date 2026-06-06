# Phase 57: Signing + TestFlight Pipeline - Pattern Map

**Mapped:** 2026-06-04
**Files analyzed:** 8 new/modified files
**Analogs found:** 7 / 8 (1 no analog — PrivacyInfo.xcprivacy is XML with no prior example)

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `.github/workflows/ci-ios.yml` | workflow | request-response (CI trigger → signed artifact) | `.github/workflows/ci-gui.yml` | role-match |
| `justfile` (add `ios-build-release`) | config/utility | batch | `justfile` existing `ios-build`, `gui-build` recipes | exact |
| `hp41-gui/src-tauri/gen/apple/ExportOptions.plist` | config | — | `hp41-gui/src-tauri/gen/apple/ExportOptions.plist` (self — edit in place) | exact |
| `hp41-gui/src-tauri/gen/apple/project.yml` | config | — | `hp41-gui/src-tauri/gen/apple/project.yml` (self — add source entry) | exact |
| `hp41-gui/src-tauri/gen/apple/hp41-gui_iOS/Info.plist` | config | — | `hp41-gui/src-tauri/gen/apple/hp41-gui_iOS/Info.plist` (self — edit in place) | exact |
| `hp41-gui/src-tauri/gen/apple/PrivacyInfo.xcprivacy` | config | — | none | no analog |
| `hp41-gui/src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset/` | asset | — | existing appiconset (self — regenerate) | exact |
| `hp41-gui/src-tauri/gen/apple/LaunchScreen.storyboard` | asset | — | `hp41-gui/src-tauri/gen/apple/LaunchScreen.storyboard` (self — edit in place) | exact |

---

## Pattern Assignments

### `.github/workflows/ci-ios.yml` (workflow, request-response)

**Analog:** `.github/workflows/ci-gui.yml`

**Trigger pattern** — ci-gui.yml uses `push`/`pull_request`; ci-ios.yml uses `workflow_dispatch` only (D-57.4). No matrix — single `macos-26` job:

```yaml
name: ci-ios

on:
  workflow_dispatch:

jobs:
  build-and-upload:
    runs-on: macos-26
```

**Env block pattern** (ci-gui.yml lines 15-18) — mirror the global env vars:

```yaml
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
```

**Toolchain setup pattern** (ci-gui.yml lines 29-48) — the exact sequence used in every workflow: checkout → dtolnay/rust-toolchain → Swatinem/rust-cache → `rustup default stable` (cargo-proxy rebind) → setup-node → install-action just. The `rustup default stable` step after the cache restore is mandatory (see comment in ci-gui.yml lines 38-40). For iOS, add `targets: aarch64-apple-ios` to the toolchain action:

```yaml
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-apple-ios

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: hp41-gui/src-tauri -> hp41-gui/src-tauri/target

      # See ci-gui.yml: macOS runner occasionally leaves cargo proxying to
      # rustup-init AFTER Swatinem/rust-cache restores ~/.cargo/bin/.
      # Running rustup default stable POST cache-restore rebinds the cargo proxy.
      - run: rustup default stable

      - uses: actions/setup-node@v4
        with:
          node-version: 'lts/*'

      - uses: taiki-e/install-action@v2
        with:
          tool: just
```

**Rust cache workspace key pattern** (ci-gui.yml line 34-35) — `workspaces: hp41-gui/src-tauri -> hp41-gui/src-tauri/target` (NOT `-> target`; the nested workspace puts target inside src-tauri). The `release-gui-binaries.yml` uses the shorter form `-> target` — the ci-gui.yml form is more precise and preferred:

```yaml
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: hp41-gui/src-tauri -> hp41-gui/src-tauri/target
```

**Secret injection + cleanup pattern** (from RESEARCH.md Pattern 2) — write .p8 to the canonical search path, cleanup with `if: always()`:

```yaml
      - name: Write ASC API key
        run: |
          mkdir -p "$HOME/.appstoreconnect/private_keys"
          echo "${{ secrets.ASC_API_KEY_P8 }}" | base64 --decode \
            > "$HOME/.appstoreconnect/private_keys/AuthKey_${{ secrets.ASC_KEY_ID }}.p8"
          chmod 600 \
            "$HOME/.appstoreconnect/private_keys/AuthKey_${{ secrets.ASC_KEY_ID }}.p8"

      - name: Cleanup API key
        if: always()
        run: rm -rf "$HOME/.appstoreconnect/private_keys/"
```

**Env-var injection for signing** (release-gui-binaries.yml lines 93-118 pattern adapted) — use step-level `env:` (not global) so secrets are scoped to only the build step that needs them. Note `APPLE_API_KEY_PATH` uses `format()` to avoid shell-injection via secret values:

```yaml
      - name: Build signed IPA
        env:
          APPLE_API_KEY: ${{ secrets.ASC_KEY_ID }}
          APPLE_API_ISSUER: ${{ secrets.ASC_ISSUER_ID }}
          APPLE_API_KEY_PATH: ${{ format('{0}/.appstoreconnect/private_keys/AuthKey_{1}.p8', runner.home, secrets.ASC_KEY_ID) }}
        run: just ios-build-release
```

**Warning annotation pattern** (release-gui-binaries.yml line 116) — use `::warning::` for non-fatal advisory messages if secrets are absent. Not needed for ios.yml (secrets are required, not optional) but the syntax is available if partial-config runs are desired later.

**IPA path — always quoted** (RESEARCH.md Pitfall 7). IPA lives at `gen/apple/build/arm64/HP-41 Calculator.ipa` (space in app name from `PRODUCT_NAME: HP-41 Calculator` in project.yml):

```yaml
      - name: Upload to TestFlight
        run: |
          IPA="hp41-gui/src-tauri/gen/apple/build/arm64/HP-41 Calculator.ipa"
          xcrun altool \
            --upload-package "$IPA" \
            --api-key "${{ secrets.ASC_KEY_ID }}" \
            --api-issuer "${{ secrets.ASC_ISSUER_ID }}"
```

**No `permissions:` block needed** — unlike release-gui-binaries.yml (which needs `contents: write` to attach assets to a GitHub Release), ci-ios.yml only uploads to TestFlight via altool; no GitHub Release is created. Omit `permissions:` entirely.

**Build-number injection** (RESEARCH.md Pattern 3) — must run before the build step, after checkout:

```yaml
      - name: Inject build number
        run: |
          /usr/libexec/PlistBuddy \
            -c "Set :CFBundleVersion ${{ github.run_number }}" \
            hp41-gui/src-tauri/gen/apple/hp41-gui_iOS/Info.plist
```

---

### `justfile` — add `ios-build-release` recipe

**Analog:** `justfile` existing `ios-build` (line 265-267) and `gui-build` (lines 143-145)

**`ios-build` recipe pattern** (justfile lines 258-267) — the base recipe this extends. Always `cd hp41-gui` first (P-iOS-03 comment at line 248), drive via `npm run tauri`:

```makefile
# iOS: release build + IPA for the physical-device triple aarch64-apple-ios
# (the `tauri ios build` default target `aarch64` — D-53.9/P-iOS-07)
[group('ios')]
ios-build:
	cd hp41-gui && npm run tauri ios build
```

**`gui-build` recipe pattern** (justfile lines 143-145) — uses `npm ci` first (lockfile-strict) then build. Mirror this: `npm ci` before `npm run tauri ios build`:

```makefile
[group('gui')]
gui-build:
	cd hp41-gui && npm ci
	cd hp41-gui && npm run tauri build
```

**New `ios-build-release` recipe to add** (follows both patterns — group, cd, npm ci, build with dist flags):

```makefile
# iOS: signed release IPA for App Store Connect (distribution) distribution.
# Passes --export-method app-store-connect and --ci (no interactive prompts).
# Requires APPLE_API_KEY, APPLE_API_ISSUER, APPLE_API_KEY_PATH env vars for signing.
[group('ios')]
ios-build-release:
	cd hp41-gui && npm ci
	cd hp41-gui && npm run tauri ios build -- --export-method app-store-connect --ci
```

**Placement:** insert after the existing `ios-build` recipe (justfile line 267), before `ios-sim`. The `# ─── iOS (Tauri v2 Mobile) ───` section boundary is at line 238.

---

### `hp41-gui/src-tauri/gen/apple/ExportOptions.plist` (config, edit in place)

**Analog:** self (current state read above, lines 1-8)

**Current state** (full file, 8 lines):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>method</key>
    <string>debugging</string>
</dict>
</plist>
```

**Target state** — add `signingStyle` and `teamID`; change `method` to `app-store-connect`. The `method` value `app-store` is deprecated in Xcode 26 (xcodebuild -help confirmed); `app-store-connect` is canonical. Note: Tauri CLI overwrites this plist at build time with `--export-method app-store-connect`, so the committed file is documentation + fallback for direct xcodebuild invocations:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>method</key>
    <string>app-store-connect</string>
    <key>signingStyle</key>
    <string>automatic</string>
    <key>teamID</key>
    <string>2P4R8QSWT4</string>
</dict>
</plist>
```

---

### `hp41-gui/src-tauri/gen/apple/project.yml` (config, add source entry)

**Analog:** self (current state read above, lines 31-44)

**Current sources list** for `hp41-gui_iOS` target (project.yml lines 35-44):

```yaml
    sources:
      - path: Sources
      - path: Assets.xcassets
      - path: Externals
      - path: hp41-gui_iOS
      - path: assets
        buildPhase: resources
        type: folder
      - path: LaunchScreen.storyboard
```

**Target state** — add `PrivacyInfo.xcprivacy` as the last sources entry. XcodeGen handles `.xcprivacy` files natively (fixed in XcodeGen PR #1464). The file lives at `gen/apple/PrivacyInfo.xcprivacy` which is the same directory as `Sources/`, `Assets.xcassets/` etc., so a bare `path:` entry works:

```yaml
    sources:
      - path: Sources
      - path: Assets.xcassets
      - path: Externals
      - path: hp41-gui_iOS
      - path: assets
        buildPhase: resources
        type: folder
      - path: LaunchScreen.storyboard
      - path: PrivacyInfo.xcprivacy   # SHIP-02: privacy manifest (NSPrivacyAccessedAPICategoryFileTimestamp / C617.1)
```

**Also update** `CFBundleShortVersionString` in the `info.properties` block (project.yml line 57) from `1.0.0` to `4.1` (D-57.6). The `CFBundleVersion` is NOT updated here — it stays `"1.0.0"` as the committed baseline; CI injects the actual build number via PlistBuddy at run time (D-57.5).

---

### `hp41-gui/src-tauri/gen/apple/hp41-gui_iOS/Info.plist` (config, edit in place)

**Analog:** self (current state read above, lines 1-42)

**Current state** (relevant lines 17-20):

```xml
	<key>CFBundleShortVersionString</key>
	<string>1.0.0</string>
	<key>CFBundleVersion</key>
	<string>1.0.0</string>
```

**Target state** — update `CFBundleShortVersionString` to `4.1` (D-57.6). Leave `CFBundleVersion` at `1.0.0` — the CI PlistBuddy step overwrites it at build time with `github.run_number`; the committed value is just the development baseline:

```xml
	<key>CFBundleShortVersionString</key>
	<string>4.1</string>
	<key>CFBundleVersion</key>
	<string>1.0.0</string>
```

**Note:** project.yml `info.properties` block also declares these keys (lines 57-58 of project.yml). Both must be updated to `4.1` / `"1.0.0"` respectively to stay in sync; XcodeGen merges them into the generated `Info.plist`. The committed `Info.plist` is the authoritative file Xcode reads (the `info.path` reference in project.yml points to it).

---

### `hp41-gui/src-tauri/gen/apple/PrivacyInfo.xcprivacy` (NEW — no analog)

**No analog exists in the codebase.** This is a new XML plist file. The exact structure is defined by Apple's privacy manifest format and is fully specified in RESEARCH.md Pattern 6:

```xml
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

**Placement:** `hp41-gui/src-tauri/gen/apple/PrivacyInfo.xcprivacy` (same directory level as `Sources/`, `Assets.xcassets/`, `LaunchScreen.storyboard`).

**Reason justification for C617.1:** "Access the timestamps, size, or other metadata of files inside the app container." This matches autosave I/O (`Library/Application Support/ch.talent-factory.hp41/autosave.json`).

---

### `hp41-gui/src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset/` (asset — regenerate)

**Analog:** self (current state is 18 PNG files + Contents.json — see file listing above)

**Current Contents.json** defines 18 image entries (lines 1-115 read above). All 18 PNG slots exist on disk. After `cargo tauri icon` runs with the new 1024×1024 master, all 18 slots will be overwritten. The `Contents.json` structure does NOT need manual editing — `cargo tauri icon` regenerates it.

**Required command pattern** (RESEARCH.md Pattern 9 — run from `hp41-gui/` directory):

```bash
cd hp41-gui
npm run tauri icon -- ../app-icon-1024.png --ios-color "#1A1A1A"
```

**Post-run verification** — due to known Tauri issue #11578, manually verify all 18 slots exist after running the command:

```bash
ls hp41-gui/src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset/*.png | wc -l
# Expect: 18 (or more if tauri adds new slots)
```

**The 18 existing filename-to-slot mapping** (from Contents.json):
- iPhone 2x/3x: `AppIcon-20x20@2x.png`, `AppIcon-20x20@3x.png`, `AppIcon-29x29@2x-1.png`, `AppIcon-29x29@3x.png`, `AppIcon-40x40@2x.png`, `AppIcon-40x40@3x.png`, `AppIcon-60x60@2x.png`, `AppIcon-60x60@3x.png`
- iPad 1x/2x: `AppIcon-20x20@1x.png`, `AppIcon-20x20@2x-1.png`, `AppIcon-29x29@1x.png`, `AppIcon-29x29@2x.png`, `AppIcon-40x40@1x.png`, `AppIcon-40x40@2x-1.png`, `AppIcon-76x76@1x.png`, `AppIcon-76x76@2x.png`, `AppIcon-83.5x83.5@2x.png`
- Marketing (1024×1024): `AppIcon-512@2x.png` (idiom: ios-marketing, scale: 1x)

---

### `hp41-gui/src-tauri/gen/apple/LaunchScreen.storyboard` (asset, edit in place)

**Analog:** self (current state read above, 30 lines)

**Current key elements** (LaunchScreen.storyboard lines 12-19):

```xml
                <viewController id="Y6W-OH-hqX" sceneMemberID="viewController">
                    <view key="view" contentMode="scaleToFill" id="5EZ-qb-Rvc">
                        <rect key="frame" x="0.0" y="0.0" width="414" height="896"/>
                        <autoresizingMask key="autoresizingMask" widthSizable="YES" heightSizable="YES"/>
                        <viewLayoutGuide key="safeArea" id="vDu-zF-Fre"/>
                        <color key="backgroundColor" systemColor="systemBackgroundColor"/>
                    </view>
                </viewController>
```

**Target state** — replace `systemColor="systemBackgroundColor"` with an explicit dark background matching HP-41 faceplate aesthetic (D-57.7). Remove the `<resources><systemColor>` block at lines 25-29 since it won't be needed once systemColor is replaced:

```xml
                        <color key="backgroundColor" red="0.102" green="0.102" blue="0.102" alpha="1" colorSpace="custom" customColorSpace="sRGB"/>
```

**Art direction note (D-57.7):** the exact visual elements (centered label, icon image view, font) are to be confirmed when the 1024×1024 master is produced. The background color change above is the minimum viable branding change. Additional `<subviews>` (UILabel, UIImageView) can be added as XML elements following standard storyboard patterns — no Xcode Interface Builder required for simple additions.

---

## Shared Patterns

### PATH shim for CI (mirrors `.xcode.env`)

**Source:** `hp41-gui/src-tauri/gen/apple/.xcode.env` (line 7)
**Apply to:** `ci-ios.yml` — the runner must have `~/.cargo/bin` on PATH for any step that shells through to cargo.

`.xcode.env` current content:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

On CI the GitHub Actions `dtolnay/rust-toolchain` + `Swatinem/rust-cache` sequence places `cargo` in `~/.cargo/bin` on the runner. The `rustup default stable` step (inherited from ci-gui.yml pattern) rebinds the cargo proxy. No separate PATH export step is needed in the workflow — the toolchain actions cover it. The `.xcode.env` pattern is relevant only for local Xcode GUI builds and is already committed.

### `just` as sole task runner (CLAUDE.md invariant)

**Source:** `justfile` (entire file — never call `cargo` or `npm run tauri` directly in CI `run:` steps)
**Apply to:** `ci-ios.yml` build step

The `run: just ios-build-release` pattern is the only correct CI invocation. Raw `cd hp41-gui && npm run tauri ios build` is NOT allowed in a `run:` step per CLAUDE.md §"Tech Stack". The recipe layer is where direct tool invocations live.

### Secrets never echoed to logs

**Source:** `release-gui-binaries.yml` lines 102-117 (the base64/conditional pattern)
**Apply to:** `ci-ios.yml` — never `cat` or `echo` the `.p8` contents. Always pipe `base64 --decode` directly to the file. Use `${{ secrets.X }}` only in `env:` or masked contexts.

### `if: always()` cleanup

**Source:** pattern from release-gui-binaries.yml step structure
**Apply to:** `ci-ios.yml` — the key file cleanup step must run even if earlier steps fail. The `if: always()` condition ensures the `.p8` is removed from the ephemeral runner regardless of build/upload outcome.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `hp41-gui/src-tauri/gen/apple/PrivacyInfo.xcprivacy` | config | — | No privacy manifest exists in the codebase. XML plist format defined entirely by Apple spec. Use RESEARCH.md Pattern 6 as the template. |

---

## Metadata

**Analog search scope:** `.github/workflows/`, `justfile`, `hp41-gui/src-tauri/gen/apple/`
**Files scanned:** 9 (4 workflows, 1 justfile, 4 gen/apple files + Contents.json)
**Pattern extraction date:** 2026-06-04

---

## PATTERN MAPPING COMPLETE

**Phase:** 57 - Signing + TestFlight Pipeline
**Files classified:** 8
**Analogs found:** 7 / 8

### Coverage
- Files with exact analog (self-edit): 5 (`ExportOptions.plist`, `project.yml`, `Info.plist`, `AppIcon.appiconset/`, `LaunchScreen.storyboard`)
- Files with role-match analog: 2 (`ci-ios.yml` ← `ci-gui.yml`; `justfile` recipe ← existing `ios-build`/`gui-build`)
- Files with no analog: 1 (`PrivacyInfo.xcprivacy` — use RESEARCH.md Pattern 6)

### Key Patterns Identified

1. **All CI workflows use identical toolchain setup sequence:** `actions/checkout@v4` → `dtolnay/rust-toolchain@stable` (with `targets:` for cross-compilation) → `Swatinem/rust-cache@v2` (workspace: `hp41-gui/src-tauri -> hp41-gui/src-tauri/target`) → `rustup default stable` (cargo-proxy rebind, mandatory) → `actions/setup-node@v4` → `taiki-e/install-action@v2 just`.

2. **`just` is the sole CI task runner** — the build step is `run: just ios-build-release`, never a raw `npm run tauri` or `cargo` invocation in a workflow `run:` step.

3. **Secrets are scoped to step-level `env:`, never global** — signing vars (`APPLE_API_KEY`, `APPLE_API_ISSUER`, `APPLE_API_KEY_PATH`) appear only on the build step that needs them; cleanup uses `if: always()`.

4. **IPA path has a space** — `"hp41-gui/src-tauri/gen/apple/build/arm64/HP-41 Calculator.ipa"` must always be double-quoted; unquoted shell splits on the space in "HP-41 Calculator".

5. **`rustup default stable` after cache restore** — mandatory step copied verbatim from ci-gui.yml lines 38-41; omitting it causes cargo proxy failures on macOS runners after the Swatinem cache action restores `~/.cargo/bin/`.
