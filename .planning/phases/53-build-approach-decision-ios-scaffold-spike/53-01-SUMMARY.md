---
phase: 53-build-approach-decision-ios-scaffold-spike
plan: 01
subsystem: infra
tags: [ios, tauri-mobile, cargo-mobile2, xcodegen, cocoapods, cross-compile, justfile]

# Dependency graph
requires:
  - phase: 52-x-mem (v4.0)
    provides: shipped hp41-gui Tauri v2 desktop app + nested-workspace structure
provides:
  - iOS-linkable hp41-gui lib (crate-type staticlib+cdylib+rlib; desktop rlib preserved)
  - just ios-init / ios-build / ios-sim / ios-dev recipes with explicit triples
  - generated gen/apple/ Xcode scaffold (project.yml, .xcodeproj, Podfile, Sources)
  - decided gen/apple/ gitignore policy (option a — tracked)
  - desktop-only tauri-plugin-autostart gated #[cfg(desktop)] so iOS cross-compiles
affects: [53-03 (spike build), 53-04 (device install), 54, 55, 56, 57]

# Tech tracking
tech-stack:
  added:
    - "rustup targets aarch64-apple-ios + aarch64-apple-ios-sim"
    - "CocoaPods (already present 1.10.1) + libimobiledevice (brew, via tauri ios init)"
    - "cargo-mobile2 generated gen/apple/ scaffold"
  patterns:
    - "iOS recipes mirror gui-* idioms: cd hp41-gui first (P-iOS-03), npm run tauri, [group('ios')], TAB bodies"
    - "desktop-only Tauri plugins gated behind #[cfg(desktop)] for the mobile cross-compile"
    - "gen/apple/ tracked; gen/schemas/ + gen/apple/Pods/ ignored (nested gitignore handles xcuserdata/build/Externals)"

key-files:
  created:
    - "hp41-gui/src-tauri/gen/apple/project.yml (+ 32 scaffold files: .xcodeproj, Podfile, Sources, Info.plist, entitlements, assets)"
  modified:
    - "hp41-gui/src-tauri/Cargo.toml (crate-type line)"
    - "hp41-gui/src-tauri/src/lib.rs (cfg(desktop) gate on autostart plugin)"
    - "Justfile (iOS recipe group)"
    - "hp41-gui/src-tauri/.gitignore (gen/apple tracking policy)"

key-decisions:
  - "gen/apple/ gitignore policy = option (a): track the scaffold (project.yml is XcodeGen source of truth) per Tauri maintainer guidance; keep gen/schemas/ and gen/apple/Pods/ ignored. Rationale: reproducible committed scaffold for CI + Phases 54-57, which will edit it (tauri.ios.conf.json, capabilities/ios.json, PrivacyInfo.xcprivacy); no init step needed before build."
  - "ios-build uses the tauri ios build default target (aarch64 = aarch64-apple-ios device); full triples documented in recipe comments to satisfy P-iOS-07 without hitting the npm `--`/runner-arg forwarding pitfall."
  - "tauri-plugin-autostart gated #[cfg(desktop)] — its init/MacosLauncher symbols do not exist on iOS; desktop behavior unchanged."

patterns-established:
  - "iOS recipe group: cd hp41-gui (P-iOS-03) + npm run tauri + xcrun simctl device discovery (P-iOS-01) + P-iOS-09 PATH comment"
  - "Mobile cross-compile gate: #[cfg(desktop)] around desktop-only plugin registrations"

requirements-completed: [BUILD-02]

# Metrics
duration: ~25 min
completed: 2026-05-31
---

# Phase 53 Plan 01: iOS Build Scaffold Summary

**Stood up the iOS build scaffold inside the nested hp41-gui workspace — crate-type for iOS static linking, a just ios-* recipe group with explicit triples, the cargo-mobile2 gen/apple/ Xcode project, and a tracked-scaffold gitignore policy — with the device-triple cross-compile passing after gating the desktop-only autostart plugin.**

## Performance

- **Duration:** ~25 min
- **Completed:** 2026-05-31
- **Tasks:** 3
- **Files modified:** 4 hand-edited + 33 generated scaffold files

## Accomplishments
- Installed both iOS Rust target triples; confirmed `cargo build --target aarch64-apple-ios` compiles hp41-core + rust_decimal + the GUI lib for the device.
- Added `crate-type = ["staticlib", "cdylib", "rlib"]` to the hp41-gui `[lib]` — iOS static link enabled, desktop `rlib` link preserved.
- Added a `just` iOS recipe group (`ios-init`, `ios-build`, `ios-sim`, `ios-dev`) mirroring the `gui-*` idioms, with explicit device/sim triples and the P-iOS-01 simctl discovery + P-iOS-09 PATH guidance.
- Generated `gen/apple/` via `tauri ios init` (cargo-mobile2 + CocoaPods) and committed the tracked scaffold.
- Verified the Frozen Invariant intact: root `Cargo.toml` members unchanged; no `tauri`/`tauri-build` in `hp41-core`.

## Task Commits

1. **Task 1: Toolchain preflight + crate-type edit** - `bb9ae05` (build) — includes the autostart cfg-gate deviation
2. **Task 2: just ios-* recipe group** - `6deef28` (build)
3. **Task 3: tauri ios init + gen/apple gitignore policy** - `e67b305` (build, scaffold + gitignore)

## Files Created/Modified
- `hp41-gui/src-tauri/Cargo.toml` — added the three-element `crate-type`
- `hp41-gui/src-tauri/src/lib.rs` — gated `tauri-plugin-autostart` behind `#[cfg(desktop)]`
- `Justfile` — new `[group('ios')]` recipe section
- `hp41-gui/src-tauri/.gitignore` — narrowed `gen/` to track `gen/apple/`
- `hp41-gui/src-tauri/gen/apple/**` — generated Xcode scaffold (33 files: project.yml, .xcodeproj, Podfile, Sources/main.mm + bindings, Info.plist, entitlements, assets, LaunchScreen)

## Decisions Made
- **gitignore = option (a) tracked** (rationale above in frontmatter) — chosen over option (b) regenerate-only because Phases 54–57 and Phase 57 CI need a reproducible, editable committed scaffold; the heavy transient subdirs stay ignored.
- **ios-build target via CLI default** — `aarch64` (device) is already the `tauri ios build` default; full triples documented in comments rather than passed through the npm `--` boundary (which would misroute `--target` to the xcodebuild runner).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Gated desktop-only autostart plugin for the iOS cross-compile**
- **Found during:** Task 1 (cross-compile smoke check, P-iOS-08)
- **Issue:** `cargo build --target aarch64-apple-ios` failed — `tauri_plugin_autostart::{init, MacosLauncher}` do not exist on the iOS target (the plugin is desktop-only). This blocked Task 1's "cross-compile completes without error" acceptance criterion.
- **Fix:** Restructured the `tauri::Builder` chain to register the autostart plugin behind `#[cfg(desktop)]`. Desktop builds are unchanged (verified via `cargo check`); the iOS device triple now compiles cleanly.
- **Files modified:** hp41-gui/src-tauri/src/lib.rs
- **Verification:** `cargo build --target aarch64-apple-ios` finishes; `cargo check` (desktop) finishes.
- **Committed in:** `bb9ae05` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** Necessary to satisfy Task 1's cross-compile acceptance criterion; minimal scope (one cfg gate), no desktop behavior change. This is a genuine porting fix that Phases 54–57 would have hit regardless.

## Issues Encountered
- The repo's `Justfile` is tracked with a capital `J`; the GSD `commit --files justfile` could not stage it (exact-case). Staged `Justfile` explicitly. No functional impact.

## User Setup Required
None for this plan — toolchain installs were automated. (App ID registration is the separate human-gated Plan 53-02.)

## Next Phase Readiness
- The iOS scaffold is buildable-ready: toolchain installed, crate-type set, `just ios-*` recipes formalized, `gen/apple/` generated and tracked, gitignore policy decided, Frozen Invariant intact.
- **Plan 53-03 can now run the decisive spike** (`just ios-build` → Simulator smoke → ADR v4.1-002).
- Plan 53-02 (Apple Developer App-ID registration, human-only) is independent and can proceed in parallel.

---
*Phase: 53-build-approach-decision-ios-scaffold-spike*
*Completed: 2026-05-31*
