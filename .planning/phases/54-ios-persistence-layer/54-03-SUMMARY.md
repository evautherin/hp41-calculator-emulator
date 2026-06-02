---
phase: 54-ios-persistence-layer
plan: "03"
subsystem: ios
tags: [ios, tauri, persistence, on-device, devicectl, verification, tauri-12552]

# Dependency graph
requires:
  - phase: 54-ios-persistence-layer
    provides: "AppHandle-aware path resolvers (plan 54-01)"
  - phase: 54-ios-persistence-layer
    provides: "visibilitychange background-save trigger (plan 54-02)"
provides:
  - "On-device confirmation that the iOS save path resolves inside the sandbox Application Support container (no #12552 fallback)"
  - "On-device confirmation that Home-press background save + kill/relaunch round-trip restores full calculator state"
  - "Resolution of the #12552 / tauri-plugin-fs / AppDataWrite open question: Outcome A (happy-path, no capability change)"
affects: [55-touch-ui-adaptation, 56-app-lifecycle-clock, 57-signing-testflight-pipeline]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "On-device verification via devicectl install/launch --console; Rust eprintln! fallback log captured over the launch stream"
    - "iOS-target compile guard: cargo check --target aarch64-apple-ios catches #[cfg(mobile)] errors the host gui-ci gate cannot"

key-files:
  created: []
  modified:
    - hp41-gui/src-tauri/src/persistence.rs
    - hp41-gui/src-tauri/src/prefs.rs

key-decisions:
  - "Task 3 = Outcome A (happy-path): app_local_data_dir() succeeded on the iPhone 15 Pro — no 'app_local_data_dir failed' log over the devicectl console stream — so #12552 did NOT bite and NO AppDataWrite capability change is needed"
  - "PERSIST-02/03 proven behaviorally: Home-press + fast kill (well under the 30s timer) + relaunch restored X/Y/Z/T + program memory + XROM state exactly (user-approved round-trip), confirming the visibilitychange save fired ahead of the periodic timer"
  - "Build-fix deviation: the #[cfg(mobile)] resolver branches called handle.path() (a tauri::Manager trait method) without the trait in scope; host gui-ci cfg-compiled it out and stayed green, but just ios-build failed (E0599). Added a mobile-scoped `use tauri::Manager;` to both resolvers"

patterns-established:
  - "iOS persistence wiring (54-01 resolver + 54-02 trigger) verified end-to-end on physical hardware; this is the device-observation contract future iOS phases inherit"

requirements-completed: [PERSIST-01, PERSIST-02, PERSIST-03]

# Metrics
duration: 25min
completed: 2026-06-02
---

# Phase 54 Plan 03: iOS Persistence On-Device Verification Summary

**End-to-end on-device proof that the iOS persistence layer (AppHandle-aware path resolver + visibilitychange background save) saves to the sandbox Application Support container and survives a background → kill → relaunch round-trip on a physical iPhone 15 Pro.**

## Performance

- **Duration:** ~25 min (incl. one build-fix + rebuild cycle)
- **Tasks:** 3 (Task 1 auto-build/install/launch; Task 2 human-verify; Task 3 decision)
- **Files modified:** 2 (build-fix only; no happy-path source changes, as planned)

## Accomplishments

- **Task 1 — Build & install:** `just ios-build` produced a signed release IPA (`gen/apple/build/arm64/HP-41 Calculator.ipa`); installed on the paired iPhone 15 Pro (`ch.talent-factory.hp41`) via `xcrun devicectl device install app`; launched with `devicectl ... process launch --console`.
- **Task 2 — Human-verify (PERSIST-01/02/03):**
  - **PATH (PERSIST-01):** No `hp41: app_local_data_dir failed` line on the console stream → `app_local_data_dir()` succeeded → save lands at `Library/Application Support/ch.talent-factory.hp41/autosave.json`, the architecturally-correct D-54.1 path. #12552 did not manifest on this device/OS.
  - **BACKGROUND TRIGGER (PERSIST-02):** Home-press fired `visibilitychange → invoke('save_state')`; no background-save warning surfaced; proven by the successful restore after a fast kill (well inside the 30s timer window).
  - **ROUND-TRIP (PERSIST-03):** Distinctive stack (X=5678 / Y=1234) + non-trivial program/XROM state set, Home, kill from app switcher, relaunch → X/Y/Z/T + program memory + XROM/assignment state restored exactly. **User-approved.**
- **Task 3 — Decision:** Resolved to **Outcome A (happy-path)** — `app_local_data_dir()` works and `std::fs` writes succeed; no `AppDataWrite` / `tauri-plugin-fs` capability added.

## Task Commits

- **Build-fix (deviation, see below):** `98985e9` — `🐛 fix(54-01): import tauri::Manager in cfg(mobile) path resolvers`
- Tasks 2 and 3 produced no source changes (device-observation + happy-path decision); this SUMMARY records their outcomes.

## Files Created/Modified

- `hp41-gui/src-tauri/src/persistence.rs` — added mobile-scoped `use tauri::Manager;` (1 insertion)
- `hp41-gui/src-tauri/src/prefs.rs` — added mobile-scoped `use tauri::Manager;` (1 insertion)

## Deviations from Plan

**One unplanned build-fix (in-scope for Task 1's "build must succeed" gate).** The first `just ios-build` failed with `E0599: no method named 'path' found for &AppHandle` at `persistence.rs:51` and `prefs.rs:90`. Root cause: the `#[cfg(mobile)]` branches of the 54-01 resolvers call `handle.path()` — a `tauri::Manager` trait method — without the trait in scope. The host gate (`just gui-ci`, macOS target) compiles those branches out under `#[cfg(not(mobile))]`, so it stayed green; only the iOS (`mobile`) target exercises them. Fix: a `use tauri::Manager;` scoped **inside** each `#[cfg(mobile)]` block (so the desktop build gains no unused-import warning). Verified with `cargo check --target aarch64-apple-ios` (green) and a clean re-run of `just ios-build` (signed IPA exported). This is a genuine 54-01 defect surfaced by the device build — recorded as a project learning (host gui-ci is blind to `#[cfg(mobile)]` compile errors).

## Issues Encountered

- First `devicectl ... process launch` was denied (`FBSOpenApplicationServiceErrorDomain error 1`, device Locked) — resolved by unlocking the iPhone and relaunching. Operational, not a code issue.

## Known Stubs

None. The iOS persistence path is fully wired and proven on hardware.

## Threat Flags

- **T-54-09 (DoS, sandbox EPERM on write):** mitigated — `std::fs` writes succeeded on device; the Outcome-C capability path was not needed.
- **T-54-10 (Info disclosure, wrong container path):** mitigated — the absent fallback log + successful round-trip confirm the correct Application Support path.
- **T-54-11 (Tampering, absolute path across container-UUID change):** already audited clean in 54-01 (P-iOS-29); device round-trip after a fresh install showed no path-string breakage.
- **T-54-05 (iCloud backup of Application Support):** accepted per D-54.1 (no PII/secrets).

## User Setup Required

None. Build/sign/install used the existing Phase 53 toolchain (DEVELOPMENT_TEAM `2P4R8QSWT4`, automatic signing).

## Next Phase Readiness

iOS persistence (PERSIST-01/02/03) is complete and device-verified. Phase 55 (Touch UI Adaptation) can build on a calculator that reliably saves/restores inside the sandbox. The `#12552` question is closed (Outcome A) — no capability surface was added, so the fs/permission posture for Phases 56–57 is unchanged.
