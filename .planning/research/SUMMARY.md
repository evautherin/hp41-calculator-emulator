# Research Summary: v4.1 iOS Foundation

**Synthesized:** 2026-05-29
**Sources:** STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md
**Confidence:** MEDIUM-HIGH overall

## Headline — Build Approach Recommendation

**Recommended: Approach A — Tauri v2 Mobile, contingent on one spike.**

Run `cargo tauri ios build` inside `hp41-gui/` on a macOS machine before committing to Approach A. If Tauri's iOS bundler **nested-workspace bug (GitHub #5865)** is unresolved in v2.11 and the build fails at the Xcode assembly step (path error *after* successful Rust compilation), fall back to **Approach B — native SwiftUI + Rust FFI via UniFFI 0.31.1**. This spike is the single gating task of the milestone; capture the outcome in ADR `docs/adr/v4.1-001-build-approach.md`.

## Executive Summary

v4.1 brings the HP-41 emulator to iPhone with a touch-first UI delivered to **TestFlight** (App Store deferred). `hp41-core` is feature-complete at v4.0 — **no new calculator functions**; the iOS build is another adapter on an unchanged core. The Frozen Invariant (root members `["hp41-core", "hp41-cli"]`; `tauri`/`tauri-build` confined to `hp41-gui/src-tauri/`) is preserved under **both** approaches.

Approach A is far cheaper: existing React UI, all 10 Tauri commands, and the full IPC contract carry over unchanged (~500–800 LOC of CSS/lifecycle/persistence). `lib.rs` already has `#[cfg_attr(mobile, tauri::mobile_entry_point)]`; the only Rust structural change is adding `crate-type = ["staticlib", "cdylib", "rlib"]` to `[lib]`. Approach B bypasses every Tauri-iOS rough edge via a standard Xcode workflow but costs 3–5× more (~2,500–3,500 LOC: a new `hp41-ios-bridge` nested workspace + SwiftUI keyboard). Not justified for a TestFlight foundation, but the correct escalation if the spike fails.

**Three hard prerequisites that block everything regardless of approach:**
1. **iOS persistence path** — `~/.hp41/autosave.json` does not exist in the iOS sandbox; replace with the app-container path before the app can save state.
2. **Bundle ID `ch.talent-factory.hp41`** must be registered in App Store Connect before any signing attempt (5-min task; missing → opaque 403s).
3. **`PrivacyInfo.xcprivacy`** (with `NSPrivacyAccessedAPICategoryFileTimestamp`, reason `C617.1`) must exist in `gen/apple/` before the first TestFlight upload, or Apple rejects with ITMS-91053.

## Stack Additions

No new runtime deps for `hp41-core`. All additions live in the iOS adapter layer only.

**Approach A (inside `hp41-gui/`):**

| Tool / Library | Version | Role |
|----------------|---------|------|
| `tauri-cli` | 2.11.x (installed) | `tauri ios init` / `tauri ios build` |
| `cocoapods` | system (brew) | Required by Tauri's iOS Podfile scaffold |
| `tauri-plugin-haptics` | 2.3.2 | Per-key haptic feedback (table stake) |
| `tauri-plugin-fs` | 2.x | iOS-sandbox-aware file paths |
| Rust iOS targets | MSRV 1.88 OK | `aarch64-apple-ios` (device), `aarch64-apple-ios-sim` (Apple-Silicon sim), `x86_64-apple-ios` (Intel sim) |

**Approach B (only if spike fails):** `uniffi` 0.31.1 + `uniffi_build` 0.31.1 in a new `hp41-ios-bridge` nested standalone workspace. `hp41-core/Cargo.toml` untouched in both cases.

**Signing (v4.1):** GitHub Secrets `IOS_CERTIFICATE`, `IOS_CERTIFICATE_PASSWORD`, `IOS_MOBILE_PROVISION`, `APPLE_API_KEY_ID`, `APPLE_API_ISSUER` + `xcrun altool` for upload. No fastlane needed until App Store. `tauri-action` does **not** support iOS — CI needs a custom `ci-ios.yml` (macOS runner) alongside `ci.yml` + `ci-gui.yml`. Deployment target iOS 14.0 (Tauri v2 default; ~98%+ of active devices). `rust_decimal` 1.42 compiles cleanly on iOS (pure Rust, full std).

## Feature Scope

The desktop window (440×1020) is already near phone-shaped (iPhone 15 Pro ≈ 393×852pt) — the SVG layout needs **adaptation, not redesign** — but key hit targets are currently ~40×16px, far below Apple's 44×44pt minimum, so a touch-layout pass is required.

**Table stakes (must-have for a credible TestFlight build):** `hp41-core` → `aarch64-apple-ios`; signed IPA + TestFlight upload; app icon + launch screen; portrait lock; safe-area insets; ≥44pt touch targets on all 44 keys; CSS `:active` feedback + `touch-action: manipulation` (kills 350ms tap delay); `-webkit-tap-highlight-color: transparent`; **per-key haptic feedback** (competitor analysis — Free42, i41CX+, my41CX — confirms this is a table stake, not optional); iOS sandbox persistence path; autosave on `applicationWillResignActive`; modal inputs at `font-size: 16px`; ALPHA-mode touch entry; visible stack panel + SHIFT indicator.

**Differentiators (optional this milestone):** per-key haptic intensity variation; error haptic on DATA ERROR / NO ROOM; print panel + PRGM listing as bottom sheets; collapsible stack panel; `overscroll-behavior: none`; force `tick_time` on foreground return.

**Deferred / anti-features:** App Store submission; iPad layout, landscape, Android; `.raw` file picker on iOS (`tauri-plugin-dialog` has no native iOS picker — non-blocking for TestFlight); iCloud sync, Widgets, Siri, Watch.

## Architecture / Integration (Approach A)

The iOS target is the same `hp41-gui` with iOS as an additional build target. `tauri ios init` runs entirely inside `hp41-gui/src-tauri/`, generates `gen/apple/`, and never touches the root workspace. Data flow is identical to desktop: touch → React `key_map.resolve()` → `invoke("dispatch_op")` → WKWebView bridge → Rust command → `CalcState` → `CalcStateView` → React re-render.

**Concrete changes:**
1. `hp41-gui/src-tauri/Cargo.toml`: add `crate-type = ["staticlib", "cdylib", "rlib"]` to `[lib]` (the single Rust change).
2. `persistence.rs`: add `state_path_for_app(handle: &AppHandle)` via `app_local_data_dir()`; keep `default_state_path()` for desktop. `CalcState` serde format unchanged; `#[serde(default)]` covers TestFlight-update forward-compat.
3. `tauri.ios.conf.json` (new): `backgroundThrottlingPolicy: "throttle"` (iOS 17+, prevents full WKWebView suspension).
4. `capabilities/ios.json` (new): iOS-scoped permissions incl. `haptics:allow-*`.
5. `hp41-gui/src/` (React/TS): safe-area CSS, touch sizing, portrait layout, `visibilitychange` autosave, ALPHA input, haptics wiring.
6. `ci-ios.yml` (new): third CI layer, macOS-only.

**Clock/lifecycle:** `SystemTime::now()` keeps advancing during suspension; the clock display is stale until the next `tick_time` on resume — HP-41CX-faithful (hardware also stopped when unpowered). Add a become-active handler to force one `tick_time` on resume. Stopwatch already frozen-on-save (v3.2); no change. Foreground-only live updates are the correct v4.1 scope.

## Watch Out For (top pitfalls)

| Pitfall | Severity | Prevention / phase |
|---------|----------|--------------------|
| Nested-workspace Tauri iOS bundler bug (#5865) — `hp41-gui` is a nested workspace | CRITICAL (Approach A) | Phase 1 spike; fall back to Approach B on Xcode-stage path error |
| `~/.hp41` does not exist on iOS — autosave silently fails | CRITICAL | Phase 2: `state_path_for_app(handle)`; `app_local_data_dir()` may hit Tauri #12552 → `dirs::home_dir()` workaround |
| Bundle ID not registered before first CI run | CRITICAL | Register `ch.talent-factory.hp41` in App Store Connect in Phase 1 |
| Desktop SVG keys ~40×16px < 44pt minimum | CRITICAL | Phase 3 touch-layout pass (transparent hit-area overlay, as i41CX+/Free42) |
| `PrivacyInfo.xcprivacy` required for upload | HIGH | Create in `gen/apple/` before Phase 5 first upload |
| ALPHA-mode touch entry has no hardware-keyboard fallback | HIGH | Phase 3 design spike: `<input>` + `visualViewport` to stay above iOS keyboard |
| Simulator vs device target triple (`-sim` on Apple Silicon) | HIGH | Be explicit in `justfile`; Tauri CLI handles it, manual `cargo build --target` is the trap |
| Web Audio (TONE/BEEP) suspended until user gesture | MED | Resume AudioContext on first touch (Phase 3) |

## Suggested Build Order (5 phases)

1. **Build-Approach ADR + iOS Scaffold Spike** — `crate-type` line, `tauri ios init`, simulator boot + one-key dispatch, register bundle ID, write ADR. Gates everything; **NEEDS SPIKE** before later phases are planned in detail. (Pitfalls: #5865, target triples, bundle-ID.)
2. **iOS Persistence Layer** — sandbox path in `persistence.rs`, `visibilitychange` autosave, round-trip verified in sim, desktop unaffected. Must precede UI work so testing actually persists. (Risk: Tauri #12552 workaround.)
3. **Touch UI Adaptation** — largest phase: ≥44pt targets on all keys, safe-area insets, portrait lock, SHIFT/ALPHA indicators, ALPHA text input, haptics, AudioContext resume. (ALPHA UX is a design spike at phase start.)
4. **App Lifecycle + Clock** — `tauri.ios.conf.json`, become-active `tick_time`; clock correct after background/foreground. Short, standard patterns.
5. **Signing + TestFlight Pipeline** — distribution cert + provisioning profile, `PrivacyInfo.xcprivacy`, app icon + launch screen, `ci-ios.yml`, `altool` upload → TestFlight internal distribution.

Phases 2 and 3 touch different files (`persistence.rs` vs CSS/React) and can overlap. Dependency order: 1 → (2, 3) → 4 → 5.

## Open Questions

| Question | Blocks | Resolution |
|----------|--------|------------|
| Does `cargo tauri ios build` succeed in the `hp41-gui/` nested workspace on v2.11? (#5865) | Phase 1 — approach decision | Run the spike; Xcode-stage path error → Approach B |
| Does `app_local_data_dir()` work on iOS in v2.11? (#12552) | Phase 2 persistence | Try first; `dirs::home_dir()` fallback on Permission Denied |
| ALPHA touch-entry UX — HTML `<input>` + `visualViewport`, or suppress iOS keyboard? | Phase 3 | Design spike at phase start |
| Does `Instant` (stopwatch) drift after iOS deep-sleep on real devices? | Phase 4 | Test on device; document divergence if any |
| Does `tauri ios init --ci` run headless in GitHub Actions? | Phase 5 CI | `--ci` flag exists; verify in practice |

## Confidence

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Tauri v2.11 iOS officially supported; `rust_decimal` iOS compile confirmed; signing toolchain documented |
| Features | HIGH | Apple HIG touch targets definitive; lifecycle Apple-documented; haptics-as-table-stake from competitor analysis |
| Architecture (Approach A) | HIGH / MED on persistence | `crate-type` + pre-existing `mobile_entry_point` code-confirmed; `app_local_data_dir` bug adds uncertainty |
| Pitfalls | HIGH (signing/TestFlight) / MED (Tauri iOS maturity) | Signing well-established; Tauri iOS has multiple open bugs in active development — needs hands-on confirmation |

**Overall: MEDIUM-HIGH.** Primary gaps: nested-workspace bug #5865 status (resolve via spike), `app_local_data_dir` #12552 (workaround documented), ALPHA touch-entry design (short spike).

---
*Synthesized from STACK.md + FEATURES.md + ARCHITECTURE.md + PITFALLS.md on 2026-05-29.*
