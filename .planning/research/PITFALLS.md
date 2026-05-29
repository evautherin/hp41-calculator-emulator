# Pitfalls: v4.1 iOS Foundation

**Domain:** Adding iPhone/iOS support to a mature Rust-core (hp41-core) + Tauri v2/React desktop app
**Researched:** 2026-05-29
**Approaches covered:** (A) Tauri v2 Mobile reusing React + hp41-core via Tauri commands; (B) Native SwiftUI + hp41-core via Rust FFI (UniFFI or swift-bridge)
**Overall confidence:** HIGH for signing/provisioning/TestFlight patterns; MEDIUM for Tauri iOS maturity specifics (rapidly evolving, multiple open bugs); MEDIUM for FFI threading/memory edge cases

---

## Overview

Seven pitfall clusters dominate the v4.1 risk surface, ordered by severity and likelihood:

1. **Tauri iOS immaturity + Xcode coupling** (P-iOS-01..06) — CRITICAL for Approach A. The mobile bundler has active open bugs (Xcode version hardcoding, physical-device debug failures, nested workspace breakage). Treat `cargo tauri ios` as alpha tooling.
2. **Rust cross-compile target confusion** (P-iOS-07..09) — HIGH for both approaches. `aarch64-apple-ios` (device) vs `aarch64-apple-ios-sim` (simulator on Apple Silicon) is a silent linker failure mode that wastes days.
3. **FFI adapter boundary pollution** (P-iOS-10..13) — HIGH for Approach B. UniFFI/swift-bridge must wrap hp41-core in a NEW crate (`hp41-ffi`); touching hp41-core directly would violate the zero-new-deps and frozen-invariants.
4. **Code signing / provisioning / TestFlight** (P-iOS-14..19) — HIGH for both approaches. `ch.talent-factory.hp41` bundle ID must be registered in App Store Connect before first CI run. Keychain setup on macOS CI runners trips up almost every project the first time.
5. **Touch UX porting** (P-iOS-20..24) — HIGH for both approaches. The desktop SVG keyboard/mouse UI has tap targets ~16–20pt, no touch affordance, and hover-dependent visual feedback. None of this works on iOS without a ground-up redesign.
6. **Audio + background lifecycle** (P-iOS-25..27) — MEDIUM. `BEEP`/`TONE` use Web Audio API (Approach A) or need AVAudioEngine (Approach B); both require explicit user-gesture unlock on iOS. Stopwatch/clock use `SystemTime::now()` which keeps working after app suspend, but the UI timer that redraws it does not.
7. **iOS persistence + sandbox paths** (P-iOS-28..31) — MEDIUM. `~/.hp41/autosave.json` does not exist on iOS. Hard-coding the path or sharing it across CLI/GUI/iOS without abstraction causes a silent data-loss failure on first launch.

---

## Section 1: Tauri v2 iOS Gotchas (Approach A)

### P-iOS-01: Hardcoded Simulator Device Name Breaks After Xcode Upgrade

**What goes wrong:** `cargo tauri ios dev` encodes `"-destination", "platform=iOS Simulator,name=iPhone 13"` inside cargo-mobile2. After upgrading to Xcode 26 (or any version that ships only newer iPhone models as default simulators), xcodebuild cannot find "iPhone 13" and the entire dev loop fails with a cryptic "Unable to find a device matching" error.

**Warning sign:** Build passes on the first attempt then breaks after an Xcode update with no code change.

**Why it happens:** cargo-mobile2 hardcodes the device name rather than querying `xcrun simctl list` at runtime. Issue tauri-apps/tauri#14233 is open as of research date with no merged fix.

**Prevention:**
- Pin the Xcode version used in CI (`macos-14` runner locks Xcode 15.x; do not upgrade mid-milestone without testing).
- Add an `xcrun simctl list devices available` verification step to the `just ios-sim` recipe.
- Accept that the device name may need manual override via `cargo tauri ios dev --device-name "iPhone 16"` or equivalent workaround until the upstream fix lands.

**Phase:** iOS Build Pipeline phase (first phase of v4.1).

**Favors:** Neither approach — but this pitfall exists only for Approach A.

---

### P-iOS-02: Physical Device Debug Fails Despite Successful Build

**What goes wrong:** `cargo tauri ios dev` targeting a connected iPhone (not simulator) produces a successful build but crashes at runtime with no useful error message. The app fails to initialize when run from the Tauri CLI path; it only works when launched directly from Xcode.

**Warning sign:** Simulator runs fine; device shows a crash or blank screen immediately.

**Why it happens:** The Tauri CLI dev path starts a websocket-based configuration server. The physical device must access the dev host over the LAN using the `TAURI_DEV_HOST` environment variable. If the device is not connected via network in Xcode's "Devices and Simulators" window first, or if the LAN address is wrong (VPN, firewall), the handshake fails silently. Issue tauri-apps/tauri#12327 and #14675 document this.

**Prevention:**
- Open Xcode → Window → Devices and Simulators → connect device via network BEFORE running `cargo tauri ios dev`.
- Add `--force-ip-prompt` to manually select the correct LAN address.
- For TestFlight builds, this dev-path issue is irrelevant — use `cargo tauri ios build` → xcodebuild archive pipeline instead.

**Phase:** iOS Build Pipeline phase.

**Favors:** Neither approach — Approach B bypasses this entirely by using a standard Xcode project.

---

### P-iOS-03: Nested Standalone Workspace Breaks Mobile Bundler

**What goes wrong:** `hp41-gui` lives as a nested standalone workspace (`hp41-gui/src-tauri/` has its own `Cargo.toml` without being a member of the root workspace). The Tauri mobile bundler looks for the compiled `.dylib` at a hardcoded path relative to `src-tauri/`, which diverges when the target directory is in the nested workspace's own tree. Build succeeds but the bundler cannot find the output and throws a "missing file" error at the link stage.

**Warning sign:** `cargo tauri ios build` completes Rust compilation but fails during the Xcode project assembly step with a path error.

**Why it happens:** Issue tauri-apps/tauri#5865 — the bundler hardcodes target paths instead of using `cargo metadata` to locate them. The workaround (closed but not fully fixed in v2) is ensuring the Tauri project is in the workspace root or using explicit `--target-dir` flags.

**Prevention:**
- Before enabling iOS support, verify `cargo metadata --manifest-path hp41-gui/src-tauri/Cargo.toml` produces the correct `target_directory`.
- If the bundler fails, add `build.withGlobalTauri = true` and verify `tauri.conf.json` `build.distDir` is an absolute path, not relative.
- Keep `hp41-gui/src-tauri/` as the `tauri build` working directory; do not invoke Tauri CLI from the repository root.

**Phase:** iOS Build Pipeline phase — resolve before any other iOS work.

**Favors:** Neither approach — Approach B bypasses this entirely.

---

### P-iOS-04: WKWebView Service Workers Disabled by Default

**What goes wrong:** Any caching strategy, background sync, or Push API code that relies on service workers silently fails. WKWebView embedded in third-party apps does not run service workers unless the app declares `WKAppBoundDomains` in `Info.plist` AND sets `limitsNavigationsToAppBoundDomains = true` in the WKWebViewConfiguration. Tauri does not configure App-Bound Domains automatically (wry issue #1587).

**Warning sign:** Vite's PWA plugin builds without error, but the service worker is never registered in production.

**Current state of the hp41-gui:** The existing desktop GUI does not use service workers — this pitfall is a non-issue for HP-41's current feature set. However, if any future caching is added, be aware.

**Prevention:** Do not add service-worker-based caching to the iOS build. Use standard HTTP caching headers and WKWebView's built-in cache instead.

**Phase:** Not blocking for v4.1 MVP, but document in ADR for future reference.

---

### P-iOS-05: WKWebView Audio Requires User-Gesture Unlock

**What goes wrong:** The HP-41 GUI uses Web Audio API (`BEEP`/`TONE` via `AudioContext`). On iOS, `AudioContext` starts in `suspended` state. A call to `audioCtx.resume()` must happen synchronously inside a user gesture handler (tap or click). If the context is created at module load time (or resumed by a programmatic event), iOS silently refuses to play audio. The first `TONE` call produces no sound and no error.

**Warning sign:** `BEEP` works in the browser/desktop build; on an iOS device, it is silent with no console error.

**Prevention:**
- In the iOS build, call `audioCtx.resume()` inside the first key-tap handler, not at app initialization.
- Guard all `BEEP`/`TONE` dispatch paths with `if (audioCtx.state === 'suspended') { await audioCtx.resume(); }`.
- Add a one-time "tap to enable sound" interaction on first launch if needed.
- Verify by running the iOS build on a real device (not the simulator — the simulator does not enforce this policy).

**Phase:** Touch UI + Audio phase.

**Favors:** Both approaches need this fix; Approach B (native AVAudioEngine) avoids WKWebView audio restrictions entirely.

---

### P-iOS-06: WKWebView Layout Pitfalls (Safe Area, Keyboard, Fixed Positioning)

**What goes wrong (three sub-pitfalls):**

a. **Safe area not applied:** Without `<meta name="viewport" content="viewport-fit=cover">` and `padding: env(safe-area-inset-bottom)` on the calculator skin container, the bottom row of keys is hidden behind the iPhone Home indicator. On iPhone 14 Pro and later (Dynamic Island), the top is also affected.

b. **Keyboard viewport not shrinking:** When a text input receives focus (e.g., XEQ modal, ALPHA entry), the iOS virtual keyboard overlaps the bottom of the screen. WKWebView does NOT resize the viewport height — instead, content is hidden behind the keyboard. The existing modal design must use `window.visualViewport.height` to position itself above the keyboard.

c. **Fixed-position flicker:** The calculator skin is likely positioned with `position: fixed` or `position: absolute` at 100vh. During inertial scroll or after keyboard dismiss, fixed elements can flicker or jump. Use `transform: translateZ(0)` to force GPU compositing.

**Prevention:**
- Add `viewport-fit=cover` to the HTML meta tag in the iOS Tauri config.
- Add `padding-bottom: env(safe-area-inset-bottom)` to the outermost skin container.
- Use `window.visualViewport` event listener to detect keyboard height on iOS.
- Test on a real device with Face ID (not just the simulator) because safe-area insets differ.

**Phase:** Touch UI phase.

---

## Section 2: Rust Cross-Compile Pitfalls (Both Approaches)

### P-iOS-07: Using aarch64-apple-ios for the Simulator on Apple Silicon

**What goes wrong:** On an Apple Silicon Mac (M1/M2/M3), the iOS Simulator also runs on ARM64. The WRONG target to use for the simulator is `aarch64-apple-ios` (that's the device target). The CORRECT target is `aarch64-apple-ios-sim`. If you compile `hp41-core` with `aarch64-apple-ios` and link it into a simulator build, the linker produces: `building for iOS Simulator, but linking in object file built for iOS`. The error message is clear but easily missed on first attempt.

**Warning sign:** Linker error mentioning "building for iOS Simulator, but linking in object file built for iOS".

**Three targets to be explicit about:**
- `aarch64-apple-ios` — real iPhone (ARM64 device)
- `aarch64-apple-ios-sim` — iOS Simulator on Apple Silicon Mac
- `x86_64-apple-ios` — iOS Simulator on Intel Mac (rarely needed; CI runs on `macos-14` = Apple Silicon)

**Prevention:**
- Document the three targets in the `justfile`. Add explicit `ios-sim` and `ios-device` recipes using the correct triple.
- CI: `macos-14` GitHub Actions runner is Apple Silicon — use `aarch64-apple-ios-sim` for simulator tests, `aarch64-apple-ios` for device archive.
- Tauri CLI handles target selection automatically via `cargo tauri ios build` — do not manually invoke `cargo build --target` for Approach A.

**Phase:** iOS Build Pipeline phase — must be correct from day one.

---

### P-iOS-08: rust_decimal and No-std Considerations on iOS

**What goes wrong:** `rust_decimal` 1.42 is used for `HpNum`. On iOS, the crate compiles as `std` (iOS supports std). This is not the pitfall. The pitfall is assuming that any future move to `no_std` for `hp41-core` would be straightforward — `rust_decimal` with `std` features is fine, but `no_std` would require `alloc` and may behave differently. For v4.1, `rust_decimal` on `aarch64-apple-ios` compiles without modification.

**Actual risk:** Low for v4.1. The zero-new-deps policy means no new crates can pull in C libraries via `build.rs` that might lack iOS support. Any crate with a C build dependency (e.g., `ring`, `openssl`, `aws-lc`) would be blocked by this policy anyway.

**Prevention:**
- Run `cargo build --target aarch64-apple-ios` for `hp41-core` early in the first phase as a smoke test.
- If any crate in the dependency tree emits a `build.rs` that calls a C compiler, it must be audited for iOS SDK compatibility before proceeding.

**Phase:** iOS Build Pipeline phase — verify in Phase 1.

---

### P-iOS-09: Xcode Build Phase Cannot Find cargo / just

**What goes wrong:** Tauri generates an Xcode project with build phases that invoke `cargo`. Xcode's process environment does not inherit the user's shell PATH. If `cargo` or `just` is installed via `rustup` (typically at `~/.cargo/bin`) or via Homebrew, Xcode cannot find them unless the PATH is explicitly set in the build phase script or via `.xcode.env`.

**Warning sign:** `cargo: command not found` in the Xcode build log, even though `cargo` works fine in the terminal.

**Prevention:**
- Add `export PATH="$HOME/.cargo/bin:$PATH"` to the Xcode build phase script or create `hp41-gui/src-tauri/gen/apple/.xcode.env.local` with the correct PATH.
- In CI, use `echo "$HOME/.cargo/bin" >> $GITHUB_PATH` before the iOS build step.
- Document this in the `justfile` iOS recipe.

**Phase:** iOS Build Pipeline phase.

---

## Section 3: FFI Pitfalls (Approach B — SwiftUI + Rust FFI)

### P-iOS-10: FFI Adapter Must Be a New Crate, Not hp41-core Modification

**What goes wrong:** The most tempting mistake is adding UniFFI attributes or `cbindgen` annotations directly to `hp41-core`. This would add `uniffi` or `swift-bridge` as a runtime dependency of `hp41-core`, violating the "zero new runtime deps" invariant and the "hp41-core must never depend on CLI/GUI" constraint. It would also pull cbindgen/uniffi into the core's build, increasing compile time and adding GPL-licensed tooling to the dependency graph (triggering the Free42 contamination guard if the scanner sees unexpected license strings).

**Prevention:**
- Create a new crate `hp41-ffi` (outside the root workspace members, similar to how `hp41-gui` is a nested workspace) that depends on `hp41-core` and declares the UniFFI/swift-bridge interface.
- `hp41-core/Cargo.toml` must remain unchanged — no new dependencies, no `[lib] crate-type = ["cdylib"]`.
- The 4-way exhaustive-match invariant does NOT apply to `hp41-ffi` — it's a thin adapter. But every `Op` dispatch must still go through `hp41-core`'s `dispatch()`.

**Phase:** Build Approach Decision ADR phase — establish this boundary before any FFI code is written.

**Favors:** Approach B requires this discipline; Approach A avoids it entirely.

---

### P-iOS-11: UniFFI Breaking Changes Across Versions

**What goes wrong:** UniFFI is explicitly "not 1.0" and warns that advanced features may break between minor versions. If `hp41-ffi` is pinned at UniFFI 0.28.x but `uniffi-bindgen` (the code generator run at build time) is updated to 0.29.x, the generated Swift bindings are incompatible with the Rust runtime library. The build succeeds but the app crashes at the FFI call site with an ABI mismatch.

**Warning sign:** App works when built from scratch, crashes after `cargo update` without source changes.

**Prevention:**
- Pin `uniffi` and `uniffi-bindgen` to the same exact version in `Cargo.toml` and in the Gemfile/Fastfile/build script.
- Use a `Cargo.lock` (checked in) for `hp41-ffi` to lock transitive UniFFI versions.
- Document the version pair in the crate's README.

**Phase:** FFI Layer phase (if Approach B chosen).

---

### P-iOS-12: Memory / Ownership: Rust Strings Leaked to Swift

**What goes wrong:** When `hp41-ffi` returns a Rust-allocated string (e.g., the display string, ALPHA buffer, error message) to Swift via raw FFI (not UniFFI's auto-managed types), the CString is allocated on the Rust heap with `CString::into_raw()`. Swift does not own this memory and will not free it. Repeated calls to `get_display_text()` leak memory — not noticeable in testing but fatal in long emulator sessions.

**Warning sign:** Memory usage grows steadily during normal emulator operation with no obvious allocation source.

**Prevention:**
- Use UniFFI's managed `String` type — it handles the allocation round-trip automatically.
- If using raw `extern "C"` functions, provide a paired `free_hp41_string(ptr: *mut c_char)` exported function and call it from Swift immediately after copying the value.
- Run the app under Instruments / Leaks for 5 minutes of typical use before shipping TestFlight.

**Phase:** FFI Layer phase.

---

### P-iOS-13: Threading: hp41-core Is Single-Threaded; Swift May Call from Any Thread

**What goes wrong:** `CalcState` is not `Send + Sync`. `hp41-core` has no async and no panics, but it expects single-threaded access. SwiftUI's view update cycle may invoke the FFI bindings from a background thread (especially if wrapped in a `Task` or `DispatchQueue.global()`). If two Swift threads call `dispatch_op()` concurrently, the shared `CalcState` behind the FFI is corrupted — or the Mutex in the Rust side panics with a poisoned-lock error.

**Warning sign:** Occasional crash in `dispatch_op` under rapid tap sequences or when the UI renders during a long program run.

**Prevention:**
- Wrap `CalcState` in a `Mutex<CalcState>` on the Rust side of `hp41-ffi`, mirroring the pattern already used in `hp41-gui/src-tauri/src/commands.rs`.
- In Swift, serialize all FFI calls through a single `DispatchQueue` (serial) or `actor`.
- Document this threading contract in `hp41-ffi`'s README.

**Phase:** FFI Layer phase.

---

## Section 4: Code Signing / Provisioning / TestFlight

### P-iOS-14: Bundle ID Not Registered in App Store Connect Before First Build

**What goes wrong:** The project's bundle ID is `ch.talent-factory.hp41`. An iOS app cannot be signed with a distribution provisioning profile until this App ID exists in App Store Connect (Certificates, Identifiers & Profiles → Identifiers). If the ID is not registered, `fastlane match appstore` fails with a 403-level error and an opaque "No app with bundle identifier" message. People often diagnose this as a certificate problem and spend hours re-generating certs.

**Warning sign:** `fastlane match appstore` exits with a non-certificate error early in the run.

**Prevention:**
- Register `ch.talent-factory.hp41` as an explicit App ID in App Store Connect → Identifiers before writing any CI pipeline.
- Also register it as an app in App Store Connect (even with no binary) so TestFlight upload works later.
- Do this in Phase 1 of the Signing phase — it takes 5 minutes but blocks everything else.

**Phase:** Signing + TestFlight phase — first task.

---

### P-iOS-15: Development vs. Distribution Profile Type Mismatch

**What goes wrong:** A distribution (appstore) provisioning profile is required for TestFlight, but developers often set up only a development profile (used for running directly on a device). `fastlane match` must be run with `type("appstore")` to generate the distribution profile. Using a development profile for an archive build produces a signed IPA that TestFlight rejects with "Invalid Signature".

**Warning sign:** `xcodebuild -exportArchive` succeeds but `altool` or `xcrun altool` returns "The bundle's signature...is invalid".

**Prevention:**
- Run `fastlane match development` AND `fastlane match appstore` separately — both are needed (development for on-device debugging, appstore for TestFlight).
- In the CI Fastfile, always call `match(type: "appstore", readonly: true)` before the archive step.
- Store the appstore certificate in the same match repo as the development certificate.

**Phase:** Signing + TestFlight phase.

---

### P-iOS-16: Keychain Access Failures on macOS CI Runners

**What goes wrong:** GitHub Actions macOS runners start with a locked default keychain. If `fastlane match` tries to install a certificate into the default keychain without unlocking it first, or if the keychain times out during a long build, code signing fails with "No identities found". This is the single most common CI signing failure.

**Warning sign:** Local machine signs and builds perfectly; CI fails at the code signing step with a keychain error.

**Prevention:**
- Call `setup_ci` in the Fastfile at the start of every CI lane — it creates a temporary keychain, imports the Match certs into it, and cleans up after the run.
- Set `MATCH_KEYCHAIN_NAME` and `MATCH_KEYCHAIN_PASSWORD` as GitHub Secrets.
- Set `FASTLANE_XCODEBUILD_SETTINGS_TIMEOUT` to `120` (seconds) to prevent timeouts on cold runners.
- Test the CI lane end-to-end at least once before the milestone demo.

**Phase:** Signing + TestFlight phase.

---

### P-iOS-17: Three App Store Connect API Credentials, Not One

**What goes wrong:** Developers generate an App Store Connect API key and copy the `.p8` key file content, but forget that three values are required: Issuer ID, Key ID, and the key content. If any one of the three is wrong or stored under the wrong GitHub Secret name, `xcrun altool` or `fastlane pilot upload` silently fails or gives an authentication error that looks unrelated to the key.

**Prevention:**
- Store all three as separate GitHub Secrets: `ASC_ISSUER_ID`, `ASC_KEY_ID`, `ASC_KEY` (the `.p8` file content, including the BEGIN/END lines).
- Verify locally with `fastlane pilot list` (which requires the same three credentials) before trusting CI.
- The API key must have "Developer" role or higher — a "Marketing" role key cannot upload builds.

**Phase:** Signing + TestFlight phase.

---

### P-iOS-18: "Works in Simulator, Fails on Device or TestFlight" Checklist

This is a cluster of failures that all share the same symptom (simulator green, device/TF fails) but have different root causes:

| Root cause | Detection | Fix |
|---|---|---|
| Wrong provisioning profile type (development vs. appstore) | "Invalid Signature" in TestFlight | Use `match(type: "appstore")` |
| Entitlements in `.entitlements` file don't match the App ID's capabilities | "Invalid entitlements" on device | Match entitlements to App ID capabilities in App Store Connect |
| Missing `NSPrivacyAccessedAPITypes` in `PrivacyInfo.xcprivacy` | Email from Apple after upload | Add `PrivacyInfo.xcprivacy` with `NSPrivacyAccessedAPICategoryFileTimestamp` entry (C617.1 reason) |
| `DEVELOPMENT_TEAM` not set in Xcode project | "No signing certificate" | Set team ID in `tauri.conf.json` iOS signing section or in the generated `project.yml` |
| Architecture mismatch (wrong slice in IPA) | Crash on launch on physical device | Ensure release archive targets `aarch64-apple-ios` not `aarch64-apple-ios-sim` |

**Prevention:** Walk through this checklist when diagnosing device/TF failures rather than randomly regenerating certificates.

**Phase:** Signing + TestFlight phase.

---

### P-iOS-19: Privacy Manifest Is Required Since May 2024

**What goes wrong:** Starting May 1, 2024, Apple rejects uploads to App Store Connect (including TestFlight) if the app uses "required reason" APIs without declaring them in a `PrivacyInfo.xcprivacy` file. `tauri-plugin-fs` accesses file timestamps (`NSPrivacyAccessedAPICategoryFileTimestamp`). Without the privacy manifest, every upload triggers an ITMS-91053 warning email, and eventually a hard rejection.

**Warning sign:** Apple sends a warning email after the first TestFlight upload: "ITMS-91053: Missing API declaration — NSPrivacyAccessedAPITypes".

**Prevention:**
- The Tauri docs explicitly state: create `hp41-gui/src-tauri/gen/apple/PrivacyInfo.xcprivacy` (which maps to the generated Xcode project) with the `NSPrivacyAccessedAPICategoryFileTimestamp` key and reason `C617.1`.
- Do this BEFORE the first TestFlight upload — fix it immediately rather than after the first warning.
- For Approach B, create the same `PrivacyInfo.xcprivacy` in the Xcode project root.

**Phase:** Signing + TestFlight phase — include as a checklist item before first upload.

---

## Section 5: Touch UX Porting

### P-iOS-20: Desktop Key Targets Are Too Small to Tap Accurately

**What goes wrong:** The HP-41C hardware has 45 keys in a compact layout. The desktop SVG skin renders these at roughly 40×16px per key at typical window sizes. Apple HIG requires minimum 44×44pt touch targets. At standard iPhone resolution, the HP-41 key grid would need to be roughly 2.5–3x larger than the desktop layout to be tappable, OR the layout must be rethought (scroll, zoom, or two-view design). Attempting to use the exact desktop SVG at a scaled-down mobile size is a guaranteed usability failure.

**Warning sign:** User misses keys frequently on a real device, especially the top row (f/g shift, ON).

**Prevention:**
- Design the iPhone layout from scratch for portrait orientation, targeting 44×44pt minimum per key with at least 8pt spacing.
- Consider splitting the keyboard into sections with a scroll, or making the skin configurable (compact/standard).
- Test on a physical iPhone SE (smallest screen) as the worst case.
- Do NOT ship the desktop SVG unchanged — schedule a dedicated Touch UI Design phase.

**Phase:** Touch UI Design phase (dedicated phase, not a sub-task of another).

---

### P-iOS-21: No Hover State — Active/Press Feedback Only

**What goes wrong:** The desktop key CSS uses `:hover` for visual feedback (key highlight on mouse-over). On iOS, hover events do not exist or are triggered only after a long tap, not during normal use. More critically, after a touch ends, `:hover` state can remain "stuck" on the last-touched element on some iOS versions. The visual press animation (currently CSS `transform: scale(0.95)` triggered on `:hover` or `.pressed`) may never fire, making taps feel unresponsive.

**Prevention:**
- Replace all `:hover` triggers with `:active` in the iOS-specific stylesheet (or use touch-event-based JavaScript to add/remove a CSS class).
- Ensure `touchstart` fires the same visual feedback as `mousedown`.
- Add `touch-action: manipulation` to all key elements to eliminate the 300ms tap delay without needing FastClick.

**Phase:** Touch UI Design phase.

---

### P-iOS-22: Gesture Conflicts — Tap vs. Scroll vs. Long-Press vs. System Gestures

**What goes wrong:** iOS system gestures (swipe-up from bottom for Control Center, swipe-down for Notification Center, swipe-from-edge for back navigation) can intercept taps near the screen edges. On a dense calculator key grid that extends to the screen edges, the bottom row of keys will conflict with the home gesture area. Additionally, if the calculator skin is inside a scrollable container, horizontal swipes on keys may trigger scroll instead of the key press.

**Prevention:**
- Set `overscroll-behavior: none` on the skin container to disable browser-level pull-to-refresh and bounce.
- Do not place interactive keys in the bottom ~34pt Safe Area — leave that clear.
- Disable the WKWebView's native scroll gesture on the skin element using `touch-action: none` (or equivalent native WKWebView configuration) if the skin should not scroll.
- For Approach B (SwiftUI), use `UIGestureRecognizer` with `cancelsTouchesInView = false` to avoid consuming touches that should reach subviews.

**Phase:** Touch UI Design phase.

---

### P-iOS-23: ALPHA Entry Modal Has No Physical Keyboard on iPhone

**What goes wrong:** The HP-41's ALPHA mode accepts character-by-character input. On desktop/CLI, this works via keyboard. On iPhone, there is no hardware keyboard (most users). The existing GUI ALPHA input mechanism — which was designed around key clicks — must be adapted to either use the iOS software keyboard or provide a scrollable character picker. The XEQ-by-name modal has the same problem.

**Warning sign:** ALPHA mode activates but the user cannot input text.

**Prevention:**
- For ALPHA/XEQ modals, show a standard `<input type="text">` (Approach A) or a `UITextField` (Approach B) positioned above the safe area, triggering the iOS software keyboard.
- The `window.visualViewport` listener (see P-iOS-06b) is essential to keep the input visible when the keyboard appears.
- This is a significant UX design decision — plan it early in the Touch UI phase.

**Phase:** Touch UI Design phase — design decision required before implementation.

---

### P-iOS-24: Print Buffer / Right Panel Layout Does Not Translate to Portrait Mobile

**What goes wrong:** The desktop GUI has a side panel (right panel) showing the program listing, stack registers, and print buffer. On a 390pt-wide iPhone screen in portrait orientation, there is no room for a side-by-side layout. Attempting to render the desktop layout on mobile produces overlapping or unreadably small panels.

**Prevention:**
- Design the mobile layout as a bottom-sheet or modal overlay for the stack/print panels, or use tabs.
- The v4.1 milestone can defer the full right-panel to v4.2 — provide only the essentials (X register display, stack X/Y/Z/T).
- The Tauri config can conditionally load a different top-level React component for iOS vs. desktop via platform detection.

**Phase:** Touch UI Design phase — scope the v4.1 panel carefully to avoid scope creep.

---

## Section 6: Audio and Background Timers

### P-iOS-25: Web Audio Context Suspended at App Start (Approach A)

Already covered in P-iOS-05. Summary: `AudioContext` must be resumed inside a user-gesture handler. The first `BEEP`/`TONE n` call after app launch will fail silently if this is not handled. **Target phase:** Touch UI + Audio phase.

---

### P-iOS-26: AVAudioSession Category Needed for Background Audio (Both Approaches)

**What goes wrong:** On iOS, if `BEEP`/`TONE` plays and the user has their iPhone on silent mode or the audio session category is not configured, audio is routed to the "ringer" output — which is muted when the user's ring/silent switch is in the silent position. For a calculator emulator this is acceptable behavior, but if the intent is that `TONE` always audible, the `AVAudioSession` category must be set to `playback` (which overrides silent switch) rather than the default `ambient` (which respects it).

**Recommendation:** Accept the silent-switch-muted behavior for v4.1. The HP-41 hardware's beeper is also silenceable — this is consistent. Document it rather than fighting it.

**Prevention:** Explicitly choose and document the AVAudioSession category in the ADR. For Approach A (Tauri), this requires a native Swift plugin; defer to v4.2 unless audio is a TestFlight requirement.

**Phase:** Document in ADR during Signing/Polish phase; implement in v4.2.

---

### P-iOS-27: Background Suspension Breaks the Live Clock Redraw (Both Approaches)

**What goes wrong:** The HP-41 Time Module (`TIME`, `CLKT`) uses `SystemTime::now()` in `hp41-core` plus a `time_offset_secs: i64` delta. The Rust computation is correct even after app suspension — `SystemTime::now()` will return the correct wall-clock time when the app resumes. However, the UI timer that triggers a `tick_time` IPC call (Approach A: `setInterval`) or a SwiftUI `Timer.publish` (Approach B) is paused by iOS when the app goes to the background. The LCD clock display will be stale when the user returns to the app until the next tick fires.

**What does NOT go wrong:** The time stored in `hp41-core` is correct on resume because `SystemTime::now()` is computed fresh on each tick. There is no cumulative drift or data loss — only a brief display stale period.

**Prevention:**
- Handle the `UIApplication.didBecomeActiveNotification` (Approach B) or `appActive` web event (Approach A via a Tauri plugin) to immediately call `tick_time` / `get_state` on resume, forcing a redraw.
- For Approach A, `tauri-plugin-app-events` or the built-in `window.addEventListener('focus', ...)` can serve as the resume trigger.
- The stopwatch is already frozen on save (by design per v3.2 decision) — this behavior is correct on iOS and needs no change.

**Phase:** Touch UI + Audio + Lifecycle phase.

---

## Section 7: iOS Persistence Pitfalls

### P-iOS-28: ~/.hp41/autosave.json Does Not Exist on iOS

**What goes wrong:** The desktop persistence uses `~/.hp41/autosave.json` — a path that assumes a Unix home directory. iOS apps run inside a strict app sandbox; the home directory equivalent is `/var/mobile/Containers/Data/Application/<UUID>/` and is only accessible via the iOS file system APIs (`FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)`). Hard-coding `~/.hp41` in any iOS code path (even by accident via the shared `hp41-core` persistence path) will fail silently — the file is never written, the state is never saved, and the user loses their session on every app restart.

**Warning sign:** App launches fresh every time with default state; no autosave persists.

**Prevention:**
- In `hp41-core`, the persistence path is already abstracted: the caller (`hp41-cli`, `hp41-gui`) supplies the path. For iOS, supply `FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0].appendingPathComponent("hp41/autosave.json")`.
- For Approach A (Tauri), use `tauri-plugin-fs` with the `$APPDATA` base directory, which Tauri resolves to the correct app container directory on iOS.
- The CLI and desktop GUI continue using `~/.hp41/` — the path is a caller concern, not a `hp41-core` concern. This is already architecturally correct; the pitfall is forgetting to wire the iOS-specific path.

**Phase:** iOS Persistence phase — first task of that phase.

---

### P-iOS-29: Absolute Paths Must Never Be Persisted

**What goes wrong:** iOS app container UUIDs are stable across normal updates but can change in edge cases (device restore, app reinstall). If `autosave.json` contains an absolute file path to another file (e.g., a stored card-reader file path, an imported `.raw` program path, an X-MEM file path), that absolute path becomes invalid after a container UUID change. The next app launch will silently fail to load the referenced file.

**Prevention:**
- All paths stored in `CalcState` or `autosave.json` must be relative to the app container root, not absolute.
- The existing card-reader and `.raw` import features use in-memory data (the file content, not the path) — this design is correct and should be preserved for iOS.
- Audit `CalcState` fields for any stored file path strings before adding iOS persistence.

**Phase:** iOS Persistence phase.

---

### P-iOS-30: iCloud Backup of Autosave File

**What goes wrong:** Files in `Documents/` are backed up to iCloud by default. The `autosave.json` is small and user-meaningful (they would want it restored), so backing it up is correct behavior. However, if large cache or temporary files are placed in `Documents/` (e.g., exported `.raw` buffers), iCloud backup size grows unnecessarily, and Apple may flag the app during review for storing non-user-data in `Documents/`.

**Prevention:**
- `autosave.json` → `Documents/hp41/` (backed up — correct; user data).
- Temporary export buffers, log files, pre-rendered textures → `Library/Caches/` (not backed up, purged by OS under storage pressure).
- Set `isExcludedFromBackupKey = true` on any directory that should not be backed up.
- For v4.1, only `autosave.json` exists — this pitfall mainly matters when additional file types are added.

**Phase:** iOS Persistence phase — establish the directory structure correctly from the start.

---

### P-iOS-31: Data Loss on App Update If Migration Is Not Handled

**What goes wrong:** When the app is updated via TestFlight or the App Store, the app container UUID persists and files are preserved. However, if v4.1 changes the `autosave.json` schema without the existing `migrate_after_load()` covering the iOS-specific fields, a user who updates from an older TestFlight build may load a JSON that is missing new fields. The existing `#[serde(default)]` policy handles this correctly — but only if it is consistently applied.

**Prevention:**
- The existing invariant (`#[serde(default)]` on every new `CalcState` field) already covers this.
- Add any new iOS-specific `CalcState` fields (e.g., iOS preferences, UI state) with `#[serde(default)]` following the existing pattern.
- Run a backward-compat test using a v4.0 `autosave.json` loaded in the v4.1 build before first TestFlight upload.

**Phase:** iOS Persistence phase.

---

## Section 8: Apple Review Landmines (Now vs. Later)

### P-iOS-32: Privacy Manifest (NOW — Affects TestFlight Upload)

Already covered in P-iOS-19. **Matters now (TestFlight):** Yes — Apple sends ITMS-91053 warnings starting with the first upload. These are warnings now but have been escalating toward hard rejections. Fix before first upload.

---

### P-iOS-33: "HP-41" Naming and HP Trademark (LATER — App Store Submission)

**What goes wrong:** Hewlett-Packard's calculator trademarks ("HP-41", "HP41C", "HP-41CX", etc.) are registered trademarks. App Store search results show multiple existing apps with names like "HP41CV", "i41CX+", "my41CX" — so HP (now HP Inc.) has historically tolerated emulator apps using the model name for clarity. However:

- Using "HP" prominently in the app name without HP's explicit permission is technically a trademark violation.
- HP Inc. could file a takedown after submission. The existing apps may have grandfathered status.
- Apple's review team does not proactively scan for third-party trademark violations during TestFlight, but HP Inc. can file a claim at any time after App Store publication.

**Current legal posture (from PROJECT.md):** "Behavioral emulation, not cycle-accurate Nut CPU" — no ROM bytes, no HP-copyrighted code.

**Risk assessment:** MEDIUM-LOW for TestFlight; MEDIUM for App Store. The project explicitly avoids ROM reproduction, which is the stronger legal risk. The name "HP-41" in the app title/description is the remaining exposure.

**Prevention:**
- For TestFlight: use any name — TestFlight is not publicly indexed; HP is unlikely to act on a beta.
- For App Store submission (later milestone): consult a trademark attorney before submission. Consider names like "RPN-41 Emulator", "Voyager 41", or "Folio 41" that describe the product without using HP's mark. The app DESCRIPTION can reference "HP-41C compatible" behavior.
- Do not use HP logos, the HP shield, or the HP wordmark as app icons or screenshots.

**Phase:** TestFlight = low risk, no action needed. App Store submission (v4.2+) = research and decide on naming strategy before submission.

---

### P-iOS-34: Guideline 4.2 Minimum Functionality (LATER — App Store; INFORMATIONAL for TestFlight)

**What goes wrong:** Apple's App Review Guideline 4.2 rejects "apps that provide minimum functionality" — commonly invoked against thin webview wrappers of websites. A WKWebView-wrapped web app (Approach A) that has no native integration, no offline capability, and no features beyond a website is at risk.

**Risk for this project:** LOW. The HP-41 emulator is a fully self-contained, offline application with a complex domain (RPN programming, XROM modules, keystroke programs, extended memory) that has no equivalent website. Existing HP-41 emulators have shipped on the App Store without 4.2 rejections.

**Prevention:**
- For Approach A, ensure the app works fully offline (no network calls — already true by design) and has substantive native functionality (file I/O, persistence, audio) beyond what a website provides.
- For Approach B (SwiftUI), native apps are rarely rejected for 4.2.
- Matters for App Store submission (later milestone), not TestFlight.

**Phase:** App Store submission milestone (v4.2+).

---

### P-iOS-35: App Store Screenshot / Metadata Requirements (LATER)

**What goes wrong:** App Store requires screenshots for all supported iPhone sizes, an app description that does not mention competitor products, and age rating metadata. None of this is required for TestFlight. Preparing screenshots after the fact is a known time sink.

**Prevention:** Out of scope for v4.1. Flag for the App Store submission milestone.

**Phase:** App Store submission milestone (v4.2+).

---

## Prevention Matrix

| Pitfall | Severity | Prevention | Phase | Now/Later |
|---------|----------|------------|-------|-----------|
| P-iOS-01: Simulator hardcoded device name | HIGH | Pin Xcode, add xcrun verification | iOS Build Pipeline | Now |
| P-iOS-02: Physical device debug fails | HIGH | Xcode Devices pre-connect, `--force-ip-prompt` | iOS Build Pipeline | Now |
| P-iOS-03: Nested workspace breaks bundler | CRITICAL | Verify target dir via `cargo metadata` | iOS Build Pipeline | Now |
| P-iOS-04: Service workers disabled | LOW | Don't add service workers; document | Not blocking v4.1 | — |
| P-iOS-05: Web Audio suspended on tap | HIGH | Resume AudioContext inside touchstart handler | Touch UI + Audio | Now |
| P-iOS-06: WKWebView safe area / keyboard / fixed | HIGH | viewport-fit=cover, safe-area CSS, visualViewport | Touch UI | Now |
| P-iOS-07: Wrong target triple sim vs device | HIGH | Three explicit targets in justfile; document | iOS Build Pipeline | Now |
| P-iOS-08: rust_decimal on iOS | LOW | Smoke test `cargo build --target aarch64-apple-ios` | iOS Build Pipeline | Now |
| P-iOS-09: Xcode build phase can't find cargo | HIGH | Export PATH in Xcode build phase / .xcode.env.local | iOS Build Pipeline | Now |
| P-iOS-10: FFI adapter must be new crate | CRITICAL (B) | Create `hp41-ffi` crate; never modify hp41-core | ADR + FFI Layer | Now (if B) |
| P-iOS-11: UniFFI breaking changes | HIGH (B) | Pin exact UniFFI version; check-in Cargo.lock | FFI Layer | Now (if B) |
| P-iOS-12: Rust string leaks to Swift | HIGH (B) | Use UniFFI managed types; Instruments check | FFI Layer | Now (if B) |
| P-iOS-13: CalcState threading via FFI | HIGH (B) | Mutex on Rust side; serial DispatchQueue on Swift | FFI Layer | Now (if B) |
| P-iOS-14: Bundle ID not registered | CRITICAL | Register App ID in App Store Connect first | Signing — first task | Now |
| P-iOS-15: Development vs. appstore profile | HIGH | Run `match development` AND `match appstore` | Signing | Now |
| P-iOS-16: Keychain fails on CI | HIGH | `setup_ci`, temporary keychain, 5 secrets | Signing | Now |
| P-iOS-17: Three ASC credentials needed | HIGH | ASC_ISSUER_ID + ASC_KEY_ID + ASC_KEY in Secrets | Signing | Now |
| P-iOS-18: Works sim, fails device/TF | HIGH | Use checklist table; audit before upload | Signing | Now |
| P-iOS-19: Privacy manifest required | HIGH | Create PrivacyInfo.xcprivacy before first upload | Signing | Now |
| P-iOS-20: Desktop keys too small | CRITICAL | Ground-up touch layout design, 44pt targets | Touch UI Design | Now |
| P-iOS-21: No hover state on iOS | HIGH | Replace :hover with :active + touchstart | Touch UI Design | Now |
| P-iOS-22: Gesture conflicts | MEDIUM | overscroll-behavior:none, safe-area margin | Touch UI Design | Now |
| P-iOS-23: No hardware keyboard for ALPHA | HIGH | iOS UITextField/input for ALPHA/XEQ modals | Touch UI Design | Now |
| P-iOS-24: Right panel layout | MEDIUM | Bottom-sheet or defer to v4.2 | Touch UI Design | Now (decide) |
| P-iOS-25: Audio suspended (same as 05) | — | See P-iOS-05 | — | — |
| P-iOS-26: AVAudioSession silent switch | LOW | Accept muted behavior; document ADR | Polish | Later |
| P-iOS-27: Background suspend freezes clock UI | MEDIUM | didBecomeActive → tick_time | Lifecycle | Now |
| P-iOS-28: ~/.hp41 path doesn't exist | CRITICAL | Use app container DocumentDirectory | Persistence | Now |
| P-iOS-29: Absolute paths in CalcState | HIGH | Audit CalcState; use relative paths | Persistence | Now |
| P-iOS-30: iCloud backup strategy | LOW | autosave → Documents; caches → Library/Caches | Persistence | Now (establish) |
| P-iOS-31: Schema migration on update | MEDIUM | `#[serde(default)]` invariant (already in place) | Persistence | Now |
| P-iOS-32: Privacy manifest (TestFlight) | HIGH | See P-iOS-19 | Signing | Now |
| P-iOS-33: HP trademark "HP-41" name | MEDIUM | TestFlight: no action; App Store: legal review | — | Later (App Store) |
| P-iOS-34: Guideline 4.2 minimum functionality | LOW | Fully offline + complex domain = low risk | — | Later (App Store) |
| P-iOS-35: App Store screenshots/metadata | LOW | Out of scope for v4.1 | — | Later (App Store) |

---

## Approach Comparison Summary

| Pitfall Category | Approach A (Tauri iOS) | Approach B (SwiftUI + FFI) |
|---|---|---|
| Build toolchain stability | WORSE — active bugs in cargo tauri ios | BETTER — standard Xcode workflow |
| WKWebView audio restrictions | Must handle explicitly | N/A — AVAudioEngine is straightforward |
| Touch UI redesign | Needed regardless | Needed regardless |
| FFI layer complexity | None — uses existing Tauri IPC | New crate required; threading discipline |
| Code reuse (React UI) | HIGH — existing components usable | LOW — full Swift/SwiftUI UI |
| Signing / TestFlight | Identical for both | Identical for both |
| Persistence path | tauri-plugin-fs handles it | FileManager API call |
| Guideline 4.2 risk | Slightly higher (WKWebView wrapper) | Lower (native app) |
| Debug experience | Harder — opaque Xcode integration | Easier — standard Xcode debugging |
| Freeze invariant safety | SAFER — hp41-core untouched | Requires FFI crate discipline |

**Overall:** Approach A is faster to prototype (existing React UI is reusable) but requires tolerating immature tooling and active bugs. Approach B has a higher initial investment (new FFI crate + SwiftUI UI) but yields a more debuggable, Apple-review-safe product with no build-system fragility.

---

## Sources

- Tauri iOS GitHub Discussions and Issues: [tauri-apps/tauri#10197](https://github.com/tauri-apps/tauri/discussions/10197), [#14233](https://github.com/tauri-apps/tauri/issues/14233), [#12327](https://github.com/tauri-apps/tauri/issues/12327), [#12172](https://github.com/tauri-apps/tauri/issues/12172), [#5865](https://github.com/tauri-apps/tauri/issues/5865)
- WKWebView iOS Gotchas (Tauri community): [takazudomodular.com/pj/zudo-tauri/docs/mobile/wkwebview-gotchas/](https://takazudomodular.com/pj/zudo-tauri/docs/mobile/wkwebview-gotchas/)
- Tauri v2 Webview Versions Reference: [v2.tauri.app/reference/webview-versions/](https://v2.tauri.app/reference/webview-versions/)
- Rust iOS target documentation: [doc.rust-lang.org/beta/rustc/platform-support/apple-ios.html](https://doc.rust-lang.org/beta/rustc/platform-support/apple-ios.html), [aarch64-apple-ios-sim](https://dev-doc.rust-lang.org/beta/rustc/platform-support/aarch64-apple-ios-sim.html)
- aarch64-apple-ios-sim Tier 2 promotion: [rust-lang/compiler-team#428](https://github.com/rust-lang/compiler-team/issues/428)
- UniFFI Swift bindings: [mozilla.github.io/uniffi-rs/latest/swift/overview.html](https://mozilla.github.io/uniffi-rs/latest/swift/overview.html)
- swift-bridge repository: [github.com/chinedufn/swift-bridge](https://github.com/chinedufn/swift-bridge)
- iOS TestFlight + GitHub Actions + fastlane match: [brightinventions.pl/blog/ios-testflight-github-actions-fastlane-match/](https://brightinventions.pl/blog/ios-testflight-github-actions-fastlane-match/)
- Apple Privacy Manifest requirements: [developer.apple.com/documentation/technotes/tn3183-adding-required-reason-api-entries-to-your-privacy-manifest](https://developer.apple.com/documentation/technotes/tn3183-adding-required-reason-api-entries-to-your-privacy-manifest)
- tauri-plugin-fs PrivacyInfo requirement: [v2.tauri.app/plugin/file-system/](https://v2.tauri.app/plugin/file-system/)
- iOS File System guide: [tanaschita.com/ios-file-system-overview/](https://tanaschita.com/ios-file-system-overview/)
- iOS touch target size: [docs.deque.com/devtools-mobile/2025.7.2/en/ios-touch-target-size/](https://docs.deque.com/devtools-mobile/2025.7.2/en/ios-touch-target-size/)
- Tauri iOS dev guide (2025): [tasukehub.com/articles/tauri-v2-mobile-guide-2025](https://tasukehub.com/articles/tauri-v2-mobile-guide-2025?lang=en)
- XCFramework and lipo workflow: [rhonabwy.com/2023/02/10/creating-an-xcframework/](https://rhonabwy.com/2023/02/10/creating-an-xcframework/)
- Ferrostar: Rust on iOS packaging guide: [stadiamaps.com/news/ferrostar-building-a-cross-platform-navigation-sdk-in-rust-part-2/](https://stadiamaps.com/news/ferrostar-building-a-cross-platform-navigation-sdk-in-rust-part-2/)
- App Store existing HP-41 emulators: [apps.apple.com/us/app/my41cx/id979041950](https://apps.apple.com/us/app/my41cx/id979041950), [apps.apple.com/us/app/i41cx/id289068865](https://apps.apple.com/us/app/i41cx/id289068865)
- Apple Guideline 4.2 Minimum Functionality: [iossubmissionguide.com/guideline-4-2-minimum-functionality/](https://iossubmissionguide.com/guideline-4-2-minimum-functionality/)
