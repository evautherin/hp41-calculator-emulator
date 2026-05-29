# Technology Stack — v4.1 iOS Foundation

**Project:** HP-41 Calculator Emulator — iOS build approach decision
**Researched:** 2026-05-29
**Scope:** iOS-only additions; existing desktop stack (Tauri v2.11 + React 18 + Rust MSRV 1.88) is unchanged.

---

## Overview

The central decision for v4.1 is the build approach:

- **Approach A** — Tauri v2 Mobile: extend the existing `hp41-gui` Tauri project to target iOS, reusing the React frontend and all existing Tauri commands verbatim.
- **Approach B** — Native SwiftUI + Rust FFI: write a new SwiftUI UI, bind `hp41-core` via UniFFI (Mozilla) as a static library / xcframework.

Both approaches compile `hp41-core` unchanged to `aarch64-apple-ios`. Neither requires any changes to `hp41-core` itself. The critical difference is what wraps the core on the mobile side.

**Recommendation up front (see rationale below): Approach A — Tauri v2 Mobile.**

---

## 1. Rust iOS Toolchain

### Targets

| Target | Use | Tier | Install |
|--------|-----|------|---------|
| `aarch64-apple-ios` | Physical iPhone device | Tier 2 | `rustup target add aarch64-apple-ios` |
| `aarch64-apple-ios-sim` | iOS Simulator on Apple Silicon Mac | Tier 2 | `rustup target add aarch64-apple-ios-sim` |
| `x86_64-apple-ios` | iOS Simulator on Intel Mac (legacy) | Tier 2 | `rustup target add x86_64-apple-ios` |

All three are Tier 2 targets (no host tools, but guaranteed to build via `rustup`). They have been stable since Xcode 12. Source: official Rust platform support docs.

### Cross-compilation Requirements

- Must compile on macOS — Apple's toolchain (`ld`, `lipo`, iOS SDK) is required. Linux CI cannot link iOS binaries.
- Xcode 14+ recommended (Xcode 12 is the documented minimum; current GitHub-hosted macOS runners ship Xcode 16.x). Full Xcode app install required — Command Line Tools alone do not ship the iOS SDK.
- `SDKROOT` / `IPHONEOS_DEPLOYMENT_TARGET` env vars are respected by `rustc`.
- Rust's own floor for iOS is iOS 10.0. Tauri v2 sets its floor to **iOS 14.0** (`bundle.iOS.minimumSystemVersion` default, confirmed from `v2.tauri.app/reference/config/`). iOS 14.0 is the correct deployment target for this project — it covers 98%+ of active devices and is required by WKWebView features Tauri depends on.

### CocoaPods

Both Approach A and the xcframework tooling for Approach B require CocoaPods on the build Mac: `brew install cocoapods`. Tauri's iOS scaffold uses it for native dependency management inside the Xcode project.

### `rust_decimal` 1.42 on iOS

`rust_decimal` 1.42 is a pure-Rust, `std`-using crate with no platform-specific code. It compiles cleanly to all three iOS targets — the Rust std is available via rustup for `aarch64-apple-ios`, `aarch64-apple-ios-sim`, and `x86_64-apple-ios`. No `no_std` complications arise because `hp41-core` already uses `std` (required by `serde`, `SystemTime::now()`, etc.). **No issues found; confidence HIGH.**

---

## 2. Approach A — Tauri v2 Mobile

### Maturity Status (as of 2026-05-29)

**Tauri v2.11.2 is the current stable version.** iOS support shipped in the Tauri 2.0 stable release (2024-10-02) and has been iteratively improved across 2025 and into 2026. The current project already uses Tauri v2.11, so no version upgrade is required.

**Honest maturity caveat:** The Tauri team acknowledged at v2.0 launch that they are "not completely happy about the developer experience" on mobile and are actively improving it. Community feedback (GitHub Discussion #10197) describes the iOS DX as having significant rough edges beyond basic templates — complex circular Tauri↔Xcode build invocations, underdocumented configuration, and insufficient support for native iOS extensions. However, the core functionality — WKWebView hosting a React app with Tauri command IPC — works and ships real apps.

**Known gaps/bugs to be aware of:**

| Issue | Status | Impact for hp41 |
|-------|--------|-----------------|
| Entitlements not always applied to IPA (issue #11089) | Closed — resolution unclear from GitHub content | Low: hp41 needs no special capabilities (no NFC, no biometrics, no push notifications) |
| Bundle identifier not read from `tauri.conf.json` in older versions (issue #9851) | Fixed in 2.x patch | None — using 2.11.2+ and `ch.talent-factory.hp41` has no hyphens |
| Native iOS extensions (share sheets, etc.) are hard to integrate | Known architectural limitation | None: hp41 does not use native extensions |
| `tauri-action` GitHub Action does not cover iOS builds | Confirmed gap | Medium: CI pipeline needs a separate custom workflow for iOS |
| Not all official Tauri plugins support iOS | Confirmed | See plugin assessment below |

### Workflow: `tauri ios init` / `tauri ios build`

```bash
# Prerequisites (once per dev Mac)
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim
brew install cocoapods

# In hp41-gui/ (the nested workspace root)
npm run tauri ios init          # generates src-tauri/gen/apple/ Xcode project
npm run tauri ios dev           # simulator hot-reload development loop
npm run tauri ios build -- --export-method app-store-connect   # release IPA
```

`tauri ios init` runs `cargo-mobile2` under the hood to scaffold an Xcode project at `src-tauri/gen/apple/`. The scaffold includes:
- An Xcode project (`.xcodeproj`) with a build phase that invokes `cargo build --target aarch64-apple-ios` as a shell script
- CocoaPods integration (`Podfile`, `Pods/`)
- Generated Swift glue that loads the compiled Rust library at runtime
- `src-tauri/gen/apple/` is a generated artifact — not manually edited except for specific capability overrides

The `identifier` field in `tauri.conf.json` becomes the iOS bundle ID. The current project bundle ID `ch.talent-factory.hp41` uses only dots and alphanumeric characters — **no hyphens** — so the known identifier bug does not apply.

### WKWebView Runtime

Tauri iOS uses **WKWebView** (WebKit / Safari engine) exclusively. Key implications:

- The Nitro JavaScript engine (used by WKWebView) is fast — performance of the React frontend is comparable to Safari.
- WKWebView does not support Service Workers by default on iOS (requires App-Bound Domains in `Info.plist`). The current hp41-gui React app does not use Service Workers. No impact.
- WebKit CSS/JS compatibility is the same as Safari. The existing React + TypeScript + Vite frontend is standard and will render identically.

### IPC Carry-Over

The existing Tauri commands carry over **without change** to iOS. Tauri v2's IPC contract is identical on iOS and desktop. The complete set of Tauri commands from `commands.rs` (`dispatch_op`, `get_state`, `sst_step`, `bst_step`, `run_stop`, `request_cancel`, `tick_time`, `submit_modal`, `cancel_modal`, `submit_modal_with_label`) are available on iOS.

The React frontend's `invoke()` calls from `@tauri-apps/api/core` work unchanged.

**What needs adaptation (not a core change):**

- **Persistence path**: `~/.hp41/autosave.json` must become the iOS app-sandbox Documents path. This is an iOS adapter concern — the iOS Tauri app passes the correct sandboxed path; `hp41-core` itself does not hard-code the path.
- **Touch UI**: The existing keyboard-grid React UI is pointer/click based. Touch targets need enlargement (minimum 44×44 pt per Apple HIG), safe-area inset handling (`env(safe-area-inset-*)`), and portrait layout optimization. This is CSS/React work, not an IPC change.

### Plugin Availability on iOS

| Plugin | iOS Support | hp41 uses it? | Notes |
|--------|-------------|---------------|-------|
| `tauri-plugin-fs` | Yes (sandboxed paths) | Yes — needed for save-file path on iOS | Must use `BaseDirectory::Document` for iOS container |
| `tauri-plugin-dialog` | Limited — no native file picker | Used for `.raw` import/export | File picker is a known gap on iOS; workaround needed for v4.1+ |
| `tauri-plugin-notification` | Yes | Not used | |
| `tauri-plugin-shell` | No iOS support | Not used in hp41-gui | |
| Web Audio API | Via WKWebView (same as Safari) | Yes — BEEP/TONE | Works unchanged |

The `.raw` file import/export (v4.0 Phase 50) uses `tauri-plugin-dialog` for a native file picker. On iOS, file access is sandboxed — the iOS document picker is not directly exposed via the Tauri dialog plugin. This is a **known gap that is non-blocking** for the v4.1 TestFlight foundation goal; `.raw` file I/O on iOS can be deferred to a follow-up.

### Signing Integration

Signing is configured via environment variables. See Section 4 for the full TestFlight workflow.

---

## 3. Approach B — Native SwiftUI + Rust FFI

### UniFFI (Recommended FFI Tool for Approach B)

**UniFFI** (Mozilla) is the industry-standard Rust→multi-language binding generator. Used in production by Firefox for Android/iOS, Proton Pass, the Ferrostar navigation SDK, and many others.

| Property | Value |
|----------|-------|
| Version | 0.31.1 (released 2026-04-13) |
| MSRV | Rust stable |
| Languages supported | Kotlin, Swift, Python, Ruby |
| Swift 6 support | Partial (actively improving) |
| Maturity | Production-used by Mozilla at scale; pre-1.0 API |
| License | MPL-2.0 |

**Why UniFFI over alternatives:**

- **vs. swift-bridge 0.1.59** (released 2026-01-06, 1.1k stars, 91 open issues): swift-bridge uses a `#[swift_bridge::bridge]` macro-bridge model that requires annotating Rust types and functions in the source files. Using it would require modifying `hp41-core` — violating the frozen invariant. It also has significant missing type support: `Box<T>`, `Arc<T>`, `&[T]` slices are unimplemented. **Rejected: requires source modification to hp41-core.**
- **vs. cbindgen + manual Swift wrappers**: Generates a C header; Swift wrappers must be hand-written. No type safety, enormous boilerplate for a ~325-variant `Op` enum. Rejected.
- **UniFFI proc-macro approach** (available since 0.25, no UDL file required): Annotate the public API of a thin adapter crate with `#[uniffi::export]` macros. The adapter wraps `hp41-core`; the core is never modified.

### Architecture for Approach B

```
hp41-core  (unchanged, frozen invariant preserved)
    |
    | depended on by
    v
hp41-ios-adapter  (new thin Rust crate, ~200–400 LOC of UniFFI exports)
    |
    | generates at build time
    v
libhp41ios.a (staticlib) + hp41ios.swift + hp41iosFFI.h
    |
    | assembled into
    v
hp41ios.xcframework
    |
    | imported by
    v
hp41-ios/  (new Xcode project, ~800–1500 LOC SwiftUI)
```

The adapter crate is where all iOS-specific deps live. `hp41-core` gets zero new deps.

**Build process (Approach B) — automated via `just ios-xcframework`:**

```bash
# 1. Cross-compile adapter + core for all three iOS targets
cargo build --manifest-path hp41-ios-adapter/Cargo.toml \
  --target aarch64-apple-ios --release
cargo build --manifest-path hp41-ios-adapter/Cargo.toml \
  --target aarch64-apple-ios-sim --release
cargo build --manifest-path hp41-ios-adapter/Cargo.toml \
  --target x86_64-apple-ios --release

# 2. Create simulator fat binary (Apple Silicon + Intel sim)
lipo -create \
  target/aarch64-apple-ios-sim/release/libhp41ios.a \
  target/x86_64-apple-ios/release/libhp41ios.a \
  -output target/universal-sim/libhp41ios.a

# 3. Generate Swift bindings (driven from the device build)
cargo run -p uniffi-bindgen -- \
  generate --library target/aarch64-apple-ios/release/libhp41ios.a \
  --language swift --out-dir generated/

# 4. Assemble xcframework
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libhp41ios.a \
  -headers generated/ \
  -library target/universal-sim/libhp41ios.a \
  -headers generated/ \
  -output hp41ios.xcframework
```

**cargo-swift** (from lib.rs) automates these steps interactively. It uses UniFFI under the hood and is macOS-only. Convenient for initial setup; the `just` recipe should script the steps directly for CI reproducibility.

**Effort profile for Approach B:**

- New Rust adapter crate (`hp41-ios-adapter`): ~200–400 LOC of UniFFI export wrappers. The public API surface is small — `dispatch_op(key_id: String)`, `get_state()`, `run_stop()`, `tick_time()` — mirroring the existing Tauri IPC contract.
- New SwiftUI app (`hp41-ios/`): ~800–1500 LOC for the touch keyboard, HP-41 display, help overlay, persistence, and modal handling.
- Total new code estimate: **~2,500–3,500 LOC** vs. Approach A's ~500–800 LOC.

---

## 4. Signing & TestFlight Tooling

### Prerequisites (Both Approaches)

1. **Apple Developer Program membership** — confirmed ($99/yr). Required for signing, device testing, and TestFlight.
2. **App ID** — register `ch.talent-factory.hp41` in App Store Connect (or a new iOS-specific ID if differentiating from the macOS app). The bundle ID must match `identifier` in `tauri.conf.json`.
3. **iOS Distribution Certificate** — "Apple Distribution" cert. Export as `.p12` with password.
4. **Provisioning Profile** — "App Store" distribution profile linked to the App ID.
5. **App Store Connect API Key** — recommended for CI (avoids 2FA friction). Developer role is sufficient.

### Approach A: Tauri-Native Signing (Recommended for v4.1)

```bash
# Environment variables drive signing in tauri ios build
export IOS_CERTIFICATE="$(base64 -i dist.p12)"
export IOS_CERTIFICATE_PASSWORD="..."
export IOS_MOBILE_PROVISION="$(base64 -i hp41.mobileprovision)"
npm run tauri ios build -- --export-method app-store-connect
# IPA written to: src-tauri/gen/apple/build/arm64/hp41.ipa
xcrun altool --upload-app --type ios \
  --file "src-tauri/gen/apple/build/arm64/hp41.ipa" \
  --apiKey "$APPLE_API_KEY_ID" --apiIssuer "$APPLE_API_ISSUER"
```

This is fully automatable in GitHub Actions with macOS runners. No fastlane required for v4.1.

### Alternative: fastlane + fastlane Match (Recommended for v5.x App Store)

Fastlane Match stores certificates and profiles in an encrypted private Git repo, synced automatically in CI. Robust for long-term multi-developer use. Overkill for v4.1 internal TestFlight.

```ruby
# Minimal Fastfile for TestFlight (Approach A)
lane :beta do
  setup_ci
  app_store_connect_api_key(
    key_id: ENV["ASC_KEY_ID"],
    issuer_id: ENV["ASC_ISSUER_ID"],
    key_content: ENV["ASC_KEY"]
  )
  match(type: "appstore", readonly: is_ci)
  sh("npm run tauri ios build -- --export-method app-store-connect")
  upload_to_testflight(ipa: "src-tauri/gen/apple/build/arm64/hp41.ipa")
end
```

### GitHub Actions CI

**Runner:** `macos-latest` (currently macOS 15 on GitHub-hosted runners, ships Xcode 16.x). iOS builds require macOS runners — Linux/Windows cannot sign iOS apps. The existing `ci-gui.yml` already uses macOS runners, so this fits naturally as a new job or workflow file.

**`tauri-action` does NOT support iOS builds.** The official `tauri-apps/tauri-action` only covers desktop (macOS, Windows, Linux). The iOS build workflow must be custom.

**Required GitHub Secrets:**

Without fastlane (minimal, recommended for v4.1):

| Secret | Value |
|--------|-------|
| `IOS_CERTIFICATE` | base64-encoded .p12 Apple Distribution certificate |
| `IOS_CERTIFICATE_PASSWORD` | .p12 password |
| `IOS_MOBILE_PROVISION` | base64-encoded App Store provisioning profile |
| `APPLE_API_KEY_ID` | App Store Connect API key ID |
| `APPLE_API_ISSUER` | App Store Connect issuer ID |

With fastlane Match (for v5.x):

| Secret | Value |
|--------|-------|
| `ASC_KEY_ID` | App Store Connect key ID |
| `ASC_ISSUER_ID` | App Store Connect issuer ID |
| `ASC_KEY` | API private key (.p8) content |
| `MATCH_PASSWORD` | Passphrase for encrypted certs repo |
| `MATCH_GIT_PRIVATE_KEY` | SSH key to access the certs repo |

**TestFlight upload:** After `xcrun altool --upload-app`, Apple validates the binary (typically 5–15 minutes). The build then appears in TestFlight for internal testers — no App Store review required for internal distribution.

---

## 5. Approach A vs. Approach B — Decision Matrix

| Criterion | Approach A (Tauri v2 Mobile) | Approach B (SwiftUI + UniFFI) |
|-----------|------------------------------|-------------------------------|
| **Reuse of existing code** | High — React UI, all Tauri commands, IPC contract carry over | Low — new SwiftUI app; adapter is new |
| **New code volume** | ~500–800 LOC (touch CSS + iOS path adapter) | ~2,500–3,500 LOC (SwiftUI UI + Rust adapter) |
| **hp41-core changes** | Zero | Zero (adapter wraps it) |
| **UI language** | TypeScript/React (existing) | Swift/SwiftUI (new to project) |
| **IPC latency** | JSON serialization per Tauri command call | Direct Swift→Rust FFI (near-zero overhead) |
| **Touch UX quality ceiling** | Good (CSS touch targets, safe areas); native controls impossible | Excellent (native iOS scroll, haptics, safe areas, UIKit) |
| **Feature parity with desktop** | Automatic — same React codebase | Manual — each feature re-implemented in SwiftUI |
| **Estimated milestone effort** | 3–4 phases | 5–7 phases |
| **Tauri iOS maturity** | Stable; rough DX; known plugin gaps | N/A |
| **TestFlight path** | Supported via `tauri ios build` + altool | Standard Xcode / fastlane path |
| **GitHub Actions CI** | macOS runner required; custom workflow (tauri-action does NOT cover iOS) | macOS runner required; standard fastlane (well-documented) |
| **App Store path (future)** | Supported via `--export-method app-store-connect` | Standard Xcode path |
| **Long-term native iOS features** | Limited (WKWebView-based) | Full access to UIKit/SwiftUI APIs |
| **Frozen invariant (hp41-core dep-free)** | Preserved | Preserved |

---

## 6. Recommendation: Approach A — Tauri v2 Mobile

**Use Tauri v2 Mobile for v4.1 iOS Foundation.**

**Rationale:**

1. **The existing React UI is almost right.** The HP-41 skin is already SVG/HTML/CSS. Touch adaptation is CSS work — enlarging key hit targets to 44×44 pt, adding `touch-action: manipulation`, handling safe-area insets with `env(safe-area-inset-*)`. This is a few hundred lines of CSS changes, not a rewrite.

2. **The Tauri IPC contract carries over unchanged.** All 10 Tauri commands work identically on iOS. Zero risk of re-introducing bugs fixed during v2.0–v4.0.

3. **The v4.1 goal is TestFlight distribution, not App Store polish.** Tauri v2 Mobile is stable enough for this. The rough edges (DX friction, entitlements bug, plugin gaps) do not affect the hp41 use case — hp41 needs no special capabilities (no NFC, biometrics, push notifications), and the `.raw` file dialog gap is non-blocking for TestFlight.

4. **Approach B is significantly more work for the same outcome.** A SwiftUI rewrite produces a nicer long-term iOS app, but v4.1 is explicitly foundation-first. The extra ~2,000 LOC of SwiftUI + UniFFI adapter provides no additional v4.1 functionality. App Store distribution is explicitly out of scope for v4.1.

5. **The Tauri ecosystem is already in the project.** `hp41-gui` is already Tauri v2.11. iOS is just another Tauri target. No new ecosystem to learn, no new toolchain to maintain for v4.1.

**When to reconsider Approach B (defer to v5.x or later):** If native iOS features are needed (share sheet, Siri shortcuts, Widgets, Live Activities, extensive haptics), or if users report that the WKWebView-based UI feels unacceptably non-native. The hp41-core + adapter architecture works for either approach — switching later remains feasible because hp41-core is UI-agnostic.

---

## 7. New Dependencies for v4.1 (iOS Adapter Layer Only)

All new deps live in the iOS adapter layer. `hp41-core` gets zero new dependencies — frozen invariant preserved.

### Approach A: Additions to `hp41-gui/`

No new Rust runtime crates expected. The existing Tauri v2.11 framework handles the iOS bridge. What is added:

| Tool/Library | Version | Role | Why |
|-------------|---------|------|-----|
| `tauri-cli` | 2.11.x (already installed) | `tauri ios init/build` commands | Drives cargo-mobile2 iOS scaffold and Xcode integration |
| `cocoapods` | system (brew) | Native iOS dependency manager | Required by Tauri iOS scaffold's Podfile |
| `tauri-plugin-fs` | 2.x | iOS-sandbox-aware file paths | iOS does not allow `~/.hp41/`; plugin exposes `BaseDirectory::Document` |

### Approach B: New Crates (hypothetical)

| Crate | Version | Role | Why |
|-------|---------|------|-----|
| `uniffi` | 0.31.1 | Generates Swift bindings from Rust | Industry-standard, Mozilla-backed, production-tested |
| `uniffi_build` | 0.31.1 | Build script integration | Drives binding generation in `build.rs` |
| `cargo-swift` (dev tool) | current | xcframework packaging helper | Wraps multi-target compile + `lipo` + `xcodebuild -create-xcframework` |

---

## 8. What NOT to Add

Explicitly excluded — keep `hp41-core` dependency-free:

- **Do NOT add `uniffi` to `hp41-core/Cargo.toml`** — UniFFI belongs in the adapter crate only. hp41-core has zero new runtime deps since v3.0.
- **Do NOT add `libc`, `objc`, or any Apple SDK bindings to hp41-core** — iOS-only; would break cross-platform compilation.
- **Do NOT add `tokio` or any async runtime** — hp41-core is deliberately synchronous; WKWebView's JavaScript bridge handles async at the UI layer.
- **Do NOT add `chrono` or `time` crate to hp41-core** — `SystemTime::now()` is sufficient and was already chosen over chrono (v3.2 decision D-38.2).
- **Do NOT modify `[workspace]` members in root `Cargo.toml`** — `hp41-gui` is a nested standalone workspace; iOS changes stay inside `hp41-gui/src-tauri/`. Root members remain `["hp41-core", "hp41-cli"]`.
- **Do NOT add fastlane as a v4.1 CI dependency** — `xcrun altool` (bundled with Xcode) is sufficient for TestFlight-only internal distribution. Fastlane Match is appropriate for v5.x App Store submission.
- **Do NOT use `tauri-plugin-shell` on iOS** — not supported on iOS; invoking shell commands is impossible in the iOS sandbox.
- **Do NOT use `tauri-action` for the iOS CI job** — it does not support iOS builds. A custom GitHub Actions workflow using environment-variable signing is required.

---

## 9. Open Questions / Risks

| Question | Risk | Mitigation |
|----------|------|------------|
| Does `ch.talent-factory.hp41` work as an iOS bundle ID with Tauri 2.11? | Low | Identifier uses only dots and alphanumeric chars; the known hyphen/underscore bug does not apply. |
| Will the SVG-based HP-41 key layout fit acceptably on a 6.1" iPhone (390×844 pt)? | Medium | SVG scales cleanly; need to verify key hit targets are 44×44 pt minimum (Apple HIG). A CSS media query pass required in Phase 1. |
| Does `tauri-plugin-fs` expose the iOS Documents container path correctly? | Medium | Plugin has iOS support; `BaseDirectory::Document` should resolve to the sandboxed container. Verify in Phase 1 implementation. |
| Does `tauri ios build` run headless (non-interactive) in GitHub Actions? | Medium | Yes — `xcrun altool` and environment-variable signing work headless. Tauri's build shell script invokes `xcodebuild` non-interactively. macOS runner required. |
| Will the auto-save thread (Mutex-protected, releases before I/O) work in Tauri iOS process model? | Low | Tauri on iOS runs Rust in the same process as the WKWebView host. The threading model is unchanged from desktop. |
| Tauri's Xcode build phase may not use cargo's incremental build cache, causing slow iteration. | Low | Known DX friction; accept for v4.1. `tauri ios dev` with simulator avoids full rebuilds for frontend changes. |
| The `tauri-plugin-dialog` native file picker is not available on iOS — `.raw` import/export needs a workaround. | Medium | Non-blocking for v4.1 TestFlight goal. Defer `.raw` iOS support to a follow-up phase. |

---

## Sources

| Source | URL | Confidence |
|--------|-----|-----------|
| Tauri v2 stable release announcement | https://v2.tauri.app/blog/tauri-20/ | HIGH |
| Tauri App Store / iOS distribution guide | https://v2.tauri.app/distribute/app-store/ | HIGH |
| Tauri iOS code signing guide | https://tauri.app/distribute/sign/ios/ | HIGH |
| Tauri v2 prerequisites (iOS toolchain) | https://v2.tauri.app/start/prerequisites/ | HIGH |
| Tauri v2 configuration reference (minimumSystemVersion default 14.0) | https://v2.tauri.app/reference/config/ | HIGH |
| Tauri iOS DX community feedback | https://github.com/tauri-apps/tauri/discussions/10197 | MEDIUM |
| Tauri iOS entitlements bug | https://github.com/tauri-apps/tauri/issues/11089 | MEDIUM |
| Tauri iOS bundle ID bug | https://github.com/tauri-apps/tauri/issues/9851 | MEDIUM |
| Tauri iOS build & development (DeepWiki) | https://deepwiki.com/tauri-apps/tauri/8.2-ios-development-and-build | MEDIUM |
| Rust iOS platform support (Tier 2, minimum iOS 10.0, Xcode 12+) | https://doc.rust-lang.org/beta/rustc/platform-support/apple-ios.html | HIGH |
| UniFFI crates.io (v0.31.1, released 2026-04-13) | https://crates.io/crates/uniffi | HIGH |
| UniFFI Swift/Xcode integration guide | https://mozilla.github.io/uniffi-rs/latest/swift/xcode.html | HIGH |
| swift-bridge GitHub (v0.1.59, released 2026-01-06) | https://github.com/chinedufn/swift-bridge | HIGH |
| Ferrostar iOS/Rust xcframework production case study | https://stadiamaps.com/news/ferrostar-building-a-cross-platform-navigation-sdk-in-rust-part-2/ | MEDIUM |
| Fastlane TestFlight + GitHub Actions tutorial | https://brightinventions.pl/blog/ios-testflight-github-actions-fastlane-match/ | MEDIUM |
| cargo-swift tool | https://lib.rs/crates/cargo-swift | MEDIUM |
