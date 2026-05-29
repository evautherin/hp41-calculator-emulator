# Architecture: iOS Integration Research

**Project:** HP-41 Calculator Emulator — v4.1 iOS Foundation
**Researched:** 2026-05-29
**Confidence:** HIGH (Approach A structural details), MEDIUM (Approach B FFI surface), MEDIUM (iOS lifecycle/persistence)

---

## Overview

This document answers how an iPhone target integrates with the existing `hp41-core` + Tauri/React architecture for both candidate approaches. The frozen invariant — `hp41-core` never depends on UI crates; `tauri`/`tauri-build` appear only in `hp41-gui/src-tauri/Cargo.toml`; root workspace members stay `["hp41-core", "hp41-cli"]` — is the binding constraint on every structural decision described here.

Two approaches are compared:
- **Approach A:** Tauri v2 Mobile — add iOS target to the existing `hp41-gui` Tauri app, reusing the React frontend and `hp41-core` as-is.
- **Approach B:** Native SwiftUI + Rust FFI — new SwiftUI UI, bind `hp41-core` via UniFFI through a thin adapter crate.

---

## Engine Reuse (hp41-core Unchanged)

`hp41-core` is a pure Rust library crate with zero UI or platform dependencies. Its `CalcState`, `Op` enum, `dispatch()`, and all XROM modules are entirely platform-agnostic. Both approaches reuse it identically:

- `hp41-core` compiles to `aarch64-apple-ios` without modification. The crate uses only `std`, `serde`, `serde_json`, `rust_decimal`, and `thiserror` — all of which cross-compile to this tier-2 Rust target without issues.
- `SystemTime::now()` (used by the Time Pac for real-time clock) maps correctly to iOS; no platform stubs are needed.
- The `dirs` crate (used in `hp41-gui` for `~/.hp41/autosave.json` path resolution) is NOT in `hp41-core` — it lives only in `hp41-gui`'s persistence layer. `hp41-core` has no filesystem knowledge.
- `MSRV 1.88` is compatible with `aarch64-apple-ios` (tier-2 target, standard library available).

**FROZEN INVARIANT STATUS:** The engine is unchanged regardless of approach chosen.

---

## Approach A: Tauri v2 Mobile Integration

### How It Works

Tauri v2 compiles the React frontend into a WKWebView-hosted web app on iOS. The Rust backend (`hp41-gui/src-tauri`) becomes a static library (`staticlib`) that the generated Xcode project links. The existing Tauri command IPC (`dispatch_op`, `get_state`, etc.) and the React frontend are reused unchanged.

### What Changes vs. What Carries Over

**Carries over unchanged:**
- All 10 Tauri commands in `commands.rs` (`dispatch_op`, `get_state`, `sst_step`, `bst_step`, `run_stop`, `request_cancel`, `tick_time`, `submit_modal`, `cancel_modal`, `submit_modal_with_label`)
- `CalcStateView` / `GuiError` IPC types in `types.rs`
- `key_map::resolve()` string-ID-to-Op translation
- The entire React + TypeScript frontend (Keyboard.tsx, Display, help overlay, etc.)
- `hp41-core` dependency path `../../hp41-core` — unchanged

**Changes required in `hp41-gui/src-tauri/Cargo.toml`:**

The `[lib]` section currently lacks `crate-type`. iOS requires `staticlib` for the Xcode link step:

```toml
[lib]
name = "hp41_gui_lib"
path = "src/lib.rs"
crate-type = ["staticlib", "cdylib", "rlib"]
```

This is the single structural change needed to `src-tauri/Cargo.toml`. The `tauri` and `tauri-build` deps remain confined to this file — the frozen invariant is preserved.

**`lib.rs` is already partially prepared:** The `#[cfg_attr(mobile, tauri::mobile_entry_point)]` attribute is already on the `run()` function. No further changes to `lib.rs` are needed for basic iOS function.

**`main.rs` delegates to `lib.rs`:** The existing `main.rs` calls `hp41_gui_lib::run()` — this is the correct desktop entry point. On iOS the `mobile_entry_point` macro kicks in instead. No changes needed.

**New files created by `tauri ios init`:**

```
hp41-gui/src-tauri/
  gen/
    apple/
      project.yml          <- XcodeGen source of truth (commit this)
      hp41-calculator/     <- generated Xcode project (regeneratable, commit per Tauri maintainer guidance)
      Podfile              <- CocoaPods deps (generated)
```

The `gen/apple/Pods/`, `gen/apple/Externals/`, and `gen/apple/build/` subdirectories are auto-gitignored by the internal `.gitignore` Tauri generates. The `project.yml` is the authoritative source; running `tauri ios init` again is idempotent (regenerates from `project.yml`).

**`tauri.ios.conf.json`** (new platform-override file at `hp41-gui/src-tauri/`):

```json
{
  "app": {
    "backgroundThrottlingPolicy": "throttle"
  }
}
```

This prevents the WKWebView from fully suspending when the app moves to background (iOS 17+). Needed because `tick_time` setInterval runs on a 100ms cadence for the live clock/stopwatch display (D-11 / D-41.1).

**`capabilities/ios.json`** (new capability file):

The existing `capabilities/default.json` applies to all platforms. An iOS-specific capability file scopes mobile-only permissions and targets with `"platforms": ["iOS"]`.

**iOS build commands (added to `justfile`):**

```just
ios-init:
    cd hp41-gui && cargo tauri ios init

ios-dev device="":
    cd hp41-gui && cargo tauri ios dev {{device}}

ios-build:
    cd hp41-gui && cargo tauri ios build --export-method release-testing

ios-testflight:
    cd hp41-gui && cargo tauri ios build --export-method app-store-connect
    xcrun altool --upload-app --type ios \
      --file "hp41-gui/src-tauri/gen/apple/build/arm64/hp41-calculator.ipa" \
      --apiKey $APPLE_API_KEY_ID --apiIssuer $APPLE_API_ISSUER
```

### Workspace Structure — Is the Frozen Invariant Preserved?

Yes, completely. `tauri ios init` operates entirely within `hp41-gui/src-tauri/`. It:
- Does NOT touch the root `Cargo.toml` or root workspace members
- Does NOT add `tauri` to `hp41-core`
- Does NOT create a new top-level workspace member
- Creates `gen/apple/` inside `hp41-gui/src-tauri/` only

The iOS target is an add-on to the **existing** nested standalone `hp41-gui` workspace, not a new workspace member at any level.

### iOS App Type

The iOS app is NOT a new separate product — it IS the same `hp41-gui` app with iOS as an additional build target. There is no separate nested workspace, no new `Cargo.toml`, no new crate. The iOS build shares the same `src-tauri/` Rust code, the same `src/` React code, and the same Tauri commands.

### Data Flow in Approach A (iOS)

```
[Touch event: key tap]
       |
[React: key_map.resolve(keyId) -> string ID]
       |
[Tauri IPC: invoke("dispatch_op", {keyId})]
       |
[WKWebView JS bridge -> Rust tauri command]
       |
[commands::dispatch_op() -> handle_op_prepare(&mut CalcState, key_id)]
       | (same dispatch() / xrom_resolve() / CalcState mutation as desktop)
[CalcStateView serialized to JSON -> IPC return]
       |
[React: re-renders display, stack, annunciators]
```

This data flow is identical to the desktop Tauri app. The IPC bridge changes from a native WebKit message-passing mechanism on macOS to WKWebView's `window.webkit.messageHandlers` mechanism on iOS — but this is transparent to both the React frontend and the Rust command handlers.

---

## Approach B: SwiftUI + UniFFI Integration

### How It Works

A thin Rust adapter crate (`hp41-ios-bridge`) sits between `hp41-core` and the SwiftUI app. UniFFI generates Swift bindings from this adapter. SwiftUI calls into Rust synchronously for ops (since `hp41-core` has no async); state is returned as a Swift-friendly `StateView` value type after each call.

### FFI Boundary Shape

**What the adapter crate exposes (surface design):**

The adapter crate wraps `hp41-core`'s `CalcState` in an `Arc<Mutex<CalcState>>` (required by UniFFI's `Sync + Send` constraint for object types) and exposes a Swift-callable class `Calculator`:

```rust
// hp41-ios-bridge/src/lib.rs

uniffi::setup_scaffolding!();

#[derive(uniffi::Object)]
pub struct Calculator {
    state: std::sync::Mutex<hp41_core::CalcState>,
}

#[uniffi::export]
impl Calculator {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self { state: std::sync::Mutex::new(hp41_core::CalcState::new()) })
    }

    pub fn dispatch_op(&self, key_id: String) -> StateView {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        // resolve key_id -> Op (reuse key_map logic from hp41-gui)
        // call hp41_core::ops::dispatch(&op, &mut state)
        StateView::from_state(&state)
    }

    pub fn load_state_json(&self, json: String) -> bool { /* serde_json parse + replace */ true }
    pub fn save_state_json(&self) -> String { /* serde_json::to_string(state) */ String::new() }
}

#[derive(uniffi::Record)]
pub struct StateView {
    pub display_str: String,
    pub x_str: String,
    pub y_str: String,
    pub z_str: String,
    pub t_str: String,
    pub lastx_str: String,
    pub print_lines: Vec<String>,
    pub program_steps: Vec<String>,
    pub pc: u64,
    pub is_running: bool,
    pub modal_prompt: Option<String>,
    pub modal_requires_alpha_label: bool,
    pub clock_active: bool,
    pub stopwatch_running: bool,
    // mirrors CalcStateView fields, using only UniFFI-compatible primitives
}
```

**Key UniFFI constraints affecting the design:**

1. Objects exposed via `#[derive(uniffi::Object)]` cannot use `&mut self` — interior mutability (`Mutex`) is required. This aligns with the existing `AppState = Mutex<CalcState>` pattern in `hp41-gui`.
2. `#[derive(uniffi::Record)]` is used for value types (passed by copy). `StateView` is a flat record with only primitive-compatible types — strings, booleans, u64, `Vec<String>`, `Option<String>`. This avoids the complexity of crossing `HpNum` (a `rust_decimal::Decimal` struct) across the boundary.
3. `uniffi::setup_scaffolding!()` in the adapter crate's `lib.rs` — NOT in `hp41-core`.
4. Generic functions are not supported by `#[uniffi::export]` — the `StateView::from_state` helper is a plain function in the adapter, not generic.

**What crosses the boundary:**
- `String` — key IDs, display strings, JSON for persistence
- `Vec<String>` — print lines, program step listings
- `bool`, `u64`, `Option<String>` — annunciators, PC, modal state
- `StateView` record — the complete view snapshot returned by every dispatch call
- `Calculator` object — held by SwiftUI as `@StateObject var calc: Calculator`

**What does NOT cross the boundary:**
- `CalcState` itself (stays in Rust, behind the Mutex)
- `HpNum` / `rust_decimal::Decimal` (not UniFFI-annotatable without a wrapper)
- `Op` enum (not exposed; the adapter resolves string key IDs to Op internally)
- `ModalProgram` enum (not exposed; only `modal_prompt: Option<String>` and flags cross)

### Adapter Crate Location

The adapter crate lives as a new nested standalone workspace — NOT added to the root `Cargo.toml` members list:

```
hp41-ios-bridge/          <- new nested standalone workspace
  Cargo.toml              <- [workspace] + [package], depends on hp41-core via path ../hp41-core
  src/
    lib.rs
  build.rs                <- uniffi build script for binding generation
  uniffi-bindgen/
    main.rs               <- uniffi-bindgen binary entry point
  ios-app/                <- SwiftUI Xcode project
    hp41-ios.xcodeproj/
    Sources/
    Tests/
  scripts/
    build-xcframework.sh
```

This strictly preserves the frozen invariant: `hp41-ios-bridge/Cargo.toml` has `[workspace]` with `resolver = "2"`, is completely isolated from the root workspace, and `tauri`/`tauri-build` remain confined to `hp41-gui/src-tauri/Cargo.toml` only.

**FROZEN INVARIANT CHECK for Approach B:**
- Root `Cargo.toml` `members` stays `["hp41-core", "hp41-cli"]` — NOT modified.
- `hp41-core` is a dependency of the adapter, not the reverse. `hp41-core` gains zero new deps.
- `tauri`/`tauri-build` remain confined to `hp41-gui/src-tauri/Cargo.toml`.

**Tension:** If the adapter crate is accidentally added to root `members`, the invariant is violated. Prevention: the adapter Cargo.toml has its own `[workspace]` header, which prevents Cargo from rolling it up into the parent workspace even if placed in a subdirectory.

### Data Flow in Approach B (SwiftUI)

```
[SwiftUI touch: key tap]
       |
[SwiftUI: calc.dispatch_op(keyId: "sin")]  <- generated Swift proxy
       |
[UniFFI FFI layer: lowers String to C-compatible, calls Rust]
       |
[adapter: key_id -> Op, dispatch(&mut CalcState), StateView::from_state()]
       |
[UniFFI: lifts StateView record fields to Swift struct]
       |
[SwiftUI: @Published stateView updated -> View re-renders]
```

### Key Op Resolver for Approach B

The adapter needs its own key resolver (equivalent to `key_map::resolve()` in `hp41-gui`). This logic can be extracted from `hp41-gui/src-tauri/src/key_map.rs` into the adapter crate. The resolver is pure Rust logic with no Tauri dependency — extraction is straightforward and does not violate any invariant.

### Build Pipeline for Approach B

```bash
# From hp41-ios-bridge/:
cargo build --release --target aarch64-apple-ios        # device
cargo build --release --target aarch64-apple-ios-sim    # ARM simulator
cargo build --release --target x86_64-apple-ios         # Intel simulator

# Combine simulators with lipo:
lipo -create \
  target/aarch64-apple-ios-sim/release/libhp41_ios_bridge.a \
  target/x86_64-apple-ios/release/libhp41_ios_bridge.a \
  -o target/iOS-sim/release/libhp41_ios_bridge.a

# Generate Swift bindings:
cargo run --bin uniffi-bindgen generate \
  --library target/aarch64-apple-ios/release/libhp41_ios_bridge.a \
  --language swift --out-dir bindings/

# Create XCFramework:
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libhp41_ios_bridge.a \
  -headers bindings/ \
  -library target/iOS-sim/release/libhp41_ios_bridge.a \
  -headers bindings/ \
  -output ios-app/HP41Bridge.xcframework
```

This build script must run after every Rust change that affects the public API surface.

---

## iOS Persistence and App Lifecycle

### The Problem: `~/.hp41/autosave.json` Does Not Exist on iOS

The desktop app uses `dirs::home_dir().join(".hp41").join("autosave.json")`. On iOS:
- The iOS sandbox provides no traditional home directory accessible to app code.
- `dirs::home_dir()` may return `None` or an unstable UUID-based path inside the app container under `/var/mobile/Containers/`.
- The existing fallback `PathBuf::from(".")` in `persistence.rs` would resolve to an undefined relative path — unsuitable for iOS.
- There is an open Tauri bug (#12552) where `app_handle.path().app_data_dir()` throws "Permission Denied" on iOS in some configurations.

### The Correct iOS Paths

**Approach A (Tauri):**
Use `AppHandle::path().app_local_data_dir()` which resolves to `Library/Application Support/<bundle_id>` inside the app container. This must be threaded into the persistence layer through the `AppHandle`, replacing the direct `dirs::home_dir()` call in the auto-save setup. The file name stays `autosave.json` but under the iOS container path.

A proven workaround for the `app_data_dir` bug: `dirs::home_dir()` reportedly works on iOS (returns a path inside the container). However, this is a low-confidence workaround — the `app_local_data_dir()` via `AppHandle` is the architecturally correct path and should be attempted first.

**Approach B (SwiftUI):**
Obtain the Application Support path entirely on the Swift side:
```swift
let appSupport = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
let savePath = appSupport.appendingPathComponent("autosave.json")
```
Pass this as a `String` to the adapter's `load_state_json` / `save_state_json` methods. The adapter crate itself has no filesystem knowledge — the SwiftUI layer provides the path. The Application Support directory must be created before first use (`FileManager.createDirectory` with `withIntermediateDirectories: true`).

### Concrete Changes to `persistence.rs` (Approach A)

1. Add `pub fn state_path_for_app(handle: &tauri::AppHandle) -> PathBuf` that calls `handle.path().app_local_data_dir().expect("app_local_data_dir unavailable").join("autosave.json")`.
2. Keep `default_state_path()` for the desktop fallback and existing unit tests (unchanged behavior).
3. In `lib.rs` setup, call `state_path_for_app(&app.handle())` when building for mobile (`#[cfg(mobile)]`) or always (passing `AppHandle` to the setup path).
4. Thread the resolved path into the auto-save thread via `move` capture.

**The desktop `~/.hp41/autosave.json` path and the iOS sandbox path are different files on different platforms.** The "shared with CLI" property of the desktop path does not apply on iOS — iOS never runs the CLI. This is correct and expected.

### Autosave on Resign-Active (iOS-specific)

iOS suspends apps within seconds of moving to background. The 30-second periodic auto-save thread is insufficient for iOS — the app may be suspended before the next save fires.

**Approach A mitigation (simplest for v4.1 foundation):**
Add a `document.addEventListener("visibilitychange", ...)` handler in React's `App.tsx`. When `document.visibilityState === "hidden"`, call `invoke("save_state")`. This fires reliably in WKWebView when the user presses the Home button or switches apps, and requires no Swift plugin. This is well-tested in WKWebView environments.

**Approach B mitigation:**
Add a `NotificationCenter` observer for `UIApplication.willResignActiveNotification` in the SwiftUI app's `@main` entry point. On notification, call `calc.saveStateJson()` and write to the Application Support path. This is native iOS and entirely within the SwiftUI layer.

### State Restoration on Relaunch

`CalcState`'s serde format is stable (`#[serde(default)]` on all fields, `migrate_after_load()` auto-upgrades). No changes to the CalcState serde format are needed for iOS. The same `StateFile { version: u32, state: CalcState }` JSON wrapper works on all platforms. The iOS save file is simply a different path, not a different format.

---

## Real-Time Clock and Stopwatch Under iOS

### `tick_time` and `setInterval` on iOS

The existing `tick_time` Tauri command is called every 100ms via `setInterval` in the React frontend when `clock_active || stopwatch_keyboard_mode` is true (D-41.8). On iOS, WKWebView's `setInterval` behavior when the app is backgrounded is the critical concern.

**Key findings:**
- By default, iOS throttles and eventually suspends WKWebView tasks when backgrounded.
- The `backgroundThrottlingPolicy: "throttle"` configuration (Tauri v2, iOS 17+) prevents full suspension while allowing some CPU throttling. For iOS 16 and below, timers will pause in background regardless.
- `SystemTime::now()` in `hp41-core` always returns wall-clock time regardless of background pauses — the displayed time will "jump" correctly after backgrounding, which matches accurate behavior.
- The stopwatch uses a monotonic `Instant` internally — iOS may affect `Instant` baselines after deep sleep, causing stopwatch drift. This is already a known limitation (stopwatch is frozen on save per CLAUDE.md).

**Required behavior for v4.1 foundation:** Accept foreground-only live updates. Timers run normally in foreground; they pause in background; on return to foreground, the next `tick_time` call updates the display to current wall-clock time. This is correct HP-41CX emulation behavior (the hardware had no background running).

**Approach B (SwiftUI):** Use a Swift `Timer.scheduledTimer(withTimeInterval: 0.1, repeats: true)` that calls an adapter method (e.g., `calc.tickTime()` wrapping `hp41-core`'s clock ops). Invalidate the timer on `resign-active`, restart it on `become-active`. This is more controllable than the WKWebView `setInterval` approach and avoids the background throttling concern entirely.

---

## Build Pipeline Structure

### Approach A: iOS CI alongside existing two-layer CI

The existing CI is:
- `ci.yml`: CLI + `hp41-core` tests + license-audit + MSRV check
- `ci-gui.yml`: 3-OS Tauri GUI matrix + E2E smoke

**New: `ci-ios.yml`** — macOS-only, iPhone simulator + device + TestFlight:

```yaml
name: iOS CI
on:
  push:
    branches: [develop, main]
  pull_request:
    branches: [develop]

jobs:
  ios-simulator:
    runs-on: macos-15
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-apple-ios,aarch64-apple-ios-sim,x86_64-apple-ios
      - name: Install cargo-tauri
        run: cargo install tauri-cli --version "^2"
      - name: Install CocoaPods
        run: gem install cocoapods
      - name: iOS init
        run: cd hp41-gui && cargo tauri ios init --ci
      - name: iOS simulator build
        run: cd hp41-gui && cargo tauri ios build --target aarch64-sim
      - name: iOS device build (debug)
        run: cd hp41-gui && cargo tauri ios build --target aarch64

  testflight:
    runs-on: macos-15
    if: github.ref == 'refs/heads/main'
    environment: testflight
    needs: ios-simulator
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-apple-ios
      - uses: apple-actions/import-codesign-certs@v2
        with:
          p12-file-base64: ${{ secrets.DIST_CERT_P12 }}
          p12-password: ${{ secrets.DIST_CERT_PASSWORD }}
      - name: Install deps
        run: |
          cargo install tauri-cli --version "^2"
          gem install cocoapods
      - name: iOS init
        run: cd hp41-gui && cargo tauri ios init --ci
      - name: Build release IPA
        run: cd hp41-gui && cargo tauri ios build --export-method app-store-connect
      - name: Upload to TestFlight
        run: |
          xcrun altool --upload-app --type ios \
            --file "hp41-gui/src-tauri/gen/apple/build/arm64/HP-41-Calculator.ipa" \
            --apiKey ${{ secrets.APPLE_API_KEY_ID }} \
            --apiIssuer ${{ secrets.APPLE_API_ISSUER }}
```

**CI notes:**
- iOS builds are macOS-only; they slot alongside (not inside) the existing `ci-gui.yml`.
- `tauri ios init --ci` skips interactive prompts and rustup target installation (targets pre-installed by the toolchain step).
- The IPA output path uses the app name from `tauri.conf.json` `productName`.
- Signing secrets are stored in a GitHub Actions `environment: testflight` for approval gates.
- `PrivacyInfo.xcprivacy` must be added to `gen/apple/` before App Store submission (foundation milestone: TestFlight only, so this is not yet blocking).

### Approach B: iOS CI for SwiftUI + UniFFI

```yaml
name: iOS CI (SwiftUI/UniFFI)
jobs:
  ios-build:
    runs-on: macos-15
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-apple-ios,aarch64-apple-ios-sim,x86_64-apple-ios
      - name: Build XCFramework
        run: cd hp41-ios-bridge && ./scripts/build-xcframework.sh
      - name: Build Xcode project (simulator)
        run: |
          xcodebuild -project hp41-ios-bridge/ios-app/hp41-ios.xcodeproj \
            -scheme HP41 \
            -destination "generic/platform=iOS Simulator" \
            build
```

---

## Component Inventory

| Component | Status | Approach A | Approach B |
|-----------|--------|-----------|-----------|
| `hp41-core` (entire crate) | REUSED UNCHANGED | Compiled to `aarch64-apple-ios` | Compiled to `aarch64-apple-ios` |
| `hp41-cli` | REUSED UNCHANGED | Not involved | Not involved |
| Root `Cargo.toml` members | REUSED UNCHANGED | `["hp41-core", "hp41-cli"]` — not modified | `["hp41-core", "hp41-cli"]` — not modified |
| `hp41-gui/src-tauri/Cargo.toml` | MODIFIED | Add `crate-type` to existing `[lib]` section | Not involved |
| `hp41-gui/src-tauri/src/lib.rs` | REUSED UNCHANGED | `mobile_entry_point` already present | Not involved |
| `hp41-gui/src-tauri/src/main.rs` | REUSED UNCHANGED | Delegates to `lib.rs` unchanged | Not involved |
| `hp41-gui/src-tauri/src/commands.rs` | REUSED UNCHANGED | All 10 commands work as-is | Not involved |
| `hp41-gui/src-tauri/src/types.rs` | REUSED UNCHANGED | `CalcStateView` unchanged | Not involved |
| `hp41-gui/src-tauri/src/key_map.rs` | REUSED UNCHANGED (A) / EXTRACTED (B) | iOS uses same resolver | Logic extracted/copied to adapter crate |
| `hp41-gui/src-tauri/src/persistence.rs` | MODIFIED | `state_path_for_app(handle)` function added; `default_state_path()` kept | Not involved |
| `hp41-gui/src-tauri/src/lib.rs` (setup block) | MODIFIED | Auto-save thread uses `state_path_for_app` on iOS | Not involved |
| `hp41-gui/src/` (React/TS) | MODIFIED (touch UI) | Touch targets, safe-area CSS, portrait layout, `visibilitychange` autosave | Not involved |
| `hp41-gui/src-tauri/tauri.conf.json` | REUSED UNCHANGED | Bundle ID `ch.talent-factory.hp41` already correct | Not involved |
| `hp41-gui/src-tauri/tauri.ios.conf.json` | NEW | `backgroundThrottlingPolicy`, iOS window config | Not involved |
| `hp41-gui/src-tauri/capabilities/ios.json` | NEW | iOS-scoped permissions capability | Not involved |
| `hp41-gui/src-tauri/gen/apple/` | NEW | Generated by `tauri ios init` | Not involved |
| `hp41-gui/src-tauri/gen/apple/project.yml` | NEW | XcodeGen source of truth (commit this) | Not involved |
| `hp41-gui/src-tauri/gen/schemas/` | EXISTS (no iOS yet) | Schemas only; `gen/apple/` still missing | Not involved |
| `hp41-ios-bridge/` | NOT INVOLVED | — | NEW nested standalone workspace |
| `hp41-ios-bridge/src/lib.rs` | NOT INVOLVED | — | NEW adapter crate |
| `hp41-ios-bridge/ios-app/` | NOT INVOLVED | — | NEW SwiftUI Xcode project |
| `ci-ios.yml` | NEW | New CI file (3rd layer alongside ci.yml + ci-gui.yml) | New CI file (different build steps) |
| `justfile` | MODIFIED | New `ios-init`, `ios-dev`, `ios-build`, `ios-testflight` recipes | New `ios-bridge-build`, `ios-bridge-xcframework` recipes |

---

## Suggested Build Order

### Foundation Milestone Build Order (Approach A — Recommended Path)

The recommended sequence verifies the build pipeline before investing in UI adaptation.

**Phase A1: Core iOS Build Scaffold (unlock simulator run)**
1. Add `crate-type = ["staticlib", "cdylib", "rlib"]` to `hp41-gui/src-tauri/Cargo.toml [lib]`.
2. Run `cargo check --target aarch64-apple-ios` inside `hp41-gui/src-tauri/` to verify `hp41-core` and all deps cross-compile.
3. Run `cargo tauri ios init` — generates `gen/apple/`, Podfile, `project.yml`.
4. Run `cargo tauri ios build --target aarch64-sim` — first simulator IPA.
5. Verify app launches in iOS Simulator (smoke test: display renders, one key tap dispatches).
6. Write ADR `v4.1-001-build-approach-tauri-mobile.md`.

**Phase A2: Persistence Layer (iOS Path)**
1. Add `state_path_for_app(handle: &AppHandle) -> PathBuf` to `persistence.rs` using `handle.path().app_local_data_dir()`.
2. Update `lib.rs` auto-save thread to use `state_path_for_app`.
3. Add `visibilitychange` listener in React (`App.tsx`) — `invoke("save_state")` on hidden.
4. Verify save/load round-trip in simulator (container path, not `~/.hp41/`).
5. Verify desktop behavior is unaffected (existing `default_state_path()` still used for desktop).

**Phase A3: Touch UI Adaptation**
1. Add `viewport-fit=cover` meta tag and `env(safe-area-inset-*)` CSS.
2. Resize touch targets (HP-41 keys: minimum 44×44pt per Apple HIG).
3. Portrait-only orientation lock in `tauri.ios.conf.json`.
4. Remove or guard hardware-keyboard-specific behavior (Ctrl shortcuts) in mobile path.
5. Visual validation on device or simulator with iPhone 15 Pro skin.

**Phase A4: Clock/Stopwatch iOS Behavior**
1. Add `tauri.ios.conf.json` with `backgroundThrottlingPolicy: "throttle"`.
2. Verify `tick_time` setInterval survives brief backgrounding (notification check → return).
3. Verify clock display resumes correctly after background/foreground cycle.

**Phase A5: Signing + TestFlight**
1. Configure Apple Developer provisioning profile for `ch.talent-factory.hp41`.
2. Add `PrivacyInfo.xcprivacy` to `gen/apple/` (Apple requires this for file access APIs).
3. Run `cargo tauri ios build --export-method app-store-connect`.
4. Upload IPA to TestFlight via `altool`.
5. Add `ci-ios.yml` GitHub Actions workflow.

### Foundation Milestone Build Order (Approach B — Alternative Path)

**Phase B1: Core iOS Build Scaffold**
1. Create `hp41-ios-bridge/` as a standalone nested workspace with `[workspace]` header.
2. Add `[lib]` with `crate-type = ["staticlib", "cdylib"]`, `uniffi` dep, `hp41-core` path dep.
3. Expose minimal `Calculator` object with `new()` + `dispatch_op()` + `save_state_json()`.
4. Write `build-xcframework.sh` script; verify it produces a valid XCFramework.
5. Create minimal SwiftUI app linking the XCFramework.
6. Verify `Calculator().dispatch_op("enter")` works in Swift. Write ADR `v4.1-001-build-approach-swiftui-uniffi.md`.

**Phase B2: Full Op Surface + StateView**
1. Define `StateView` record mirroring `CalcStateView` fields (primitives only).
2. Expose all ops as adapter methods (sst_step, bst_step, run_stop, etc.).
3. Implement `load_state_json` / `save_state_json` for persistence.
4. Extract and adapt `key_map::resolve()` into the adapter crate.

**Phase B3: SwiftUI UI + Persistence**
1. Build SwiftUI calculator keyboard (44 touch targets, safe-area aware).
2. Wire persistence to Application Support path via `FileManager`.
3. Add `willResignActiveNotification` observer for autosave.
4. Add Swift `Timer` for `tick_time` equivalent.

**Phase B4: Signing + TestFlight** (same as Approach A Phase A5)

---

## Frozen Invariant Check

| Invariant | Approach A | Approach B |
|-----------|-----------|-----------|
| Root `Cargo.toml` members stay `["hp41-core", "hp41-cli"]` | PRESERVED — `tauri ios init` does not modify root Cargo.toml | PRESERVED — adapter is a separate nested standalone workspace |
| `hp41-core` never depends on `hp41-cli` or `hp41-gui` | PRESERVED — `hp41-core` gains no new deps | PRESERVED — adapter depends on core, not the reverse |
| `tauri`/`tauri-build` appear ONLY in `hp41-gui/src-tauri/Cargo.toml` | PRESERVED — iOS builds inside existing `hp41-gui/src-tauri/` | PRESERVED — `hp41-ios-bridge` uses `uniffi`, no `tauri` |
| Bundle ID `ch.talent-factory.hp41` | PRESERVED — `tauri.conf.json` identifier unchanged | ADAPTED — SwiftUI app uses same bundle ID; no conflict |
| MSRV 1.88 | PRESERVED — `aarch64-apple-ios` tier-2 target, MSRV 1.88 compiles | PRESERVED — same MSRV declared in adapter crate |
| SC-4 (no core logic duplication) | PRESERVED — no new `op_*` functions in any iOS-specific file | PRESERVED — adapter is a thin wrapper only |
| 4-way exhaustive-match invariant | PRESERVED — no new `Op` variants in iOS foundation milestone | PRESERVED — same |
| `#[serde(default)]` on new CalcState fields | PRESERVED — no new CalcState fields in this milestone | PRESERVED — same |
| No polling (D-11) | PRESERVED — setInterval only when `clock_active` (same rule) | ADAPTED — Swift Timer equivalent replaces setInterval |

**Tension for Approach A — persistence layer:** The `default_state_path()` function uses `dirs::home_dir()`, which is desktop-only. If `state_path_for_app(handle)` is not correctly conditioned, both paths might be used on desktop. Mitigation: `state_path_for_app` replaces the path construction in `lib.rs` setup unconditionally (AppHandle is always available there); `default_state_path()` is retained only for unit tests.

---

## Confidence Assessment

| Area | Confidence | Source |
|------|-----------|--------|
| Tauri v2 iOS `crate-type = ["staticlib", ...]` requirement | HIGH | Tauri docs + community confirmed |
| `mobile_entry_point` already in `lib.rs` | HIGH | Direct code inspection |
| `gen/apple/` structure from `tauri ios init` | HIGH | Tauri maintainer guidance + community docs |
| Approach A carries IPC/React unchanged to iOS | HIGH | Tauri architecture docs + WKWebView evidence |
| iOS path via `app_local_data_dir` (Tauri) | MEDIUM | Open bug #12552 — may need `dirs` workaround |
| `backgroundThrottlingPolicy` for iOS 17+ | HIGH | Tauri commit a2d36b8 + config schema |
| Timer behavior on iOS 16 and below | MEDIUM | iOS platform documentation |
| UniFFI `Arc<Mutex>` pattern for stateful objects | HIGH | UniFFI user guide (interface docs) |
| Approach B `StateView` UniFFI Record | HIGH | UniFFI Record type documentation |
| Approach B binary size (~25MB static lib) | MEDIUM | One real-world measurement; hp41 core is smaller |
| TestFlight `altool` upload command | HIGH | Tauri App Store distribution docs |
| `visibilitychange` autosave on iOS WKWebView | MEDIUM | General WKWebView behavior; needs verification |

---

## Sources

- [Tauri v2 iOS Prerequisites](https://v2.tauri.app/start/prerequisites/) — rustup targets, Xcode requirements
- [Tauri v2 iOS CLI Reference](https://v2.tauri.app/reference/cli/) — `tauri ios init/dev/build` commands
- [Tauri v2 App Store Distribution](https://v2.tauri.app/distribute/app-store/) — TestFlight upload, signing
- [Tauri v2 File System Plugin](https://v2.tauri.app/plugin/file-system/) — iOS access restrictions
- [Tauri GitHub: gen/ folder discussion #8323](https://github.com/tauri-apps/tauri/discussions/8323) — commit gen/ recommendation
- [Tauri GitHub: iOS app data dir bug #12552](https://github.com/tauri-apps/tauri/issues/12552) — Permission Denied on iOS
- [Tauri GitHub: iOS/Android path resolution bug #12276](https://github.com/tauri-apps/tauri/issues/12276) — path inconsistency
- [Tauri background throttling commit](https://github.com/tauri-apps/tauri/commit/a2d36b8c34a8dcfc6736797ca5cd4665faf75e7e) — `backgroundThrottlingPolicy` iOS 17+
- [WKWebView Gotchas on iOS](https://takazudomodular.com/pj/zudo-tauri/docs/mobile/wkwebview-gotchas/) — safe area, viewport, service workers
- [iOS Project Structure (Tauri)](https://takazudomodular.com/pj/zudo-tauri/docs/mobile/ios-project-structure/) — gen/apple/ layout, project.yml
- [UniFFI Interfaces/Objects Guide](https://mozilla.github.io/uniffi-rs/0.27/udl/interfaces.html) — `Arc<T>`, `Sync+Send`, no `&mut self`
- [Rust aarch64-apple-ios Target](https://doc.rust-lang.org/beta/rustc/platform-support/apple-ios.html) — Tier 2, iOS 10+ minimum
- [Building iOS App with Rust using UniFFI](https://dev.to/almaju/building-an-ios-app-with-rust-using-uniffi-200a) — build workflow, XCFramework
- [Setting up UniFFI for iOS Simulators](https://codethoughts.io/posts/2024-06-24-setting-up-uniffi-for-ios-simulators-and-watchos/) — lipo, XCFramework creation
- [State replication across Rust-Swift barriers](https://www.tantaluspath.com/tech/rust_to_swift_state_syncing/) — diff-based state sync pattern with UniFFI
- [UniFFI Starter project](https://github.com/ianthetechie/uniffi-starter) — workspace structure, build script pattern
- [Multiplatform with Rust on iOS](https://mobilesystemdesign.substack.com/p/multiplatform-with-rust-on-ios-2c4) — adapter crate pattern
