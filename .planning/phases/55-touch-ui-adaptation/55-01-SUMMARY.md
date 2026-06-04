---
phase: 55-touch-ui-adaptation
plan: "01"
subsystem: hp41-gui
tags: [ios, tauri, haptics, permissions, orientation, viewport]
dependency_graph:
  requires: []
  provides: [is_ios-command, isIos-frontend-flag, haptics-plugin-mobile-gated, viewport-fit-cover, portrait-lock]
  affects: [hp41-gui/src-tauri/src/commands.rs, hp41-gui/src-tauri/src/lib.rs, hp41-gui/src/App.tsx]
tech_stack:
  added: [tauri-plugin-haptics 2.3.2 (mobile-only Cargo dep), @tauri-apps/plugin-haptics ^2.3.2 (npm)]
  patterns: [cfg(mobile) mobile gate, platform-restricted capability file, data-isios CSS attribute]
key_files:
  created:
    - hp41-gui/src-tauri/permissions/is-ios.toml
    - hp41-gui/src-tauri/capabilities/mobile.json
  modified:
    - hp41-gui/src-tauri/src/commands.rs
    - hp41-gui/src-tauri/src/lib.rs
    - hp41-gui/src-tauri/Cargo.toml
    - hp41-gui/src-tauri/capabilities/default.json
    - hp41-gui/src/App.tsx
    - hp41-gui/index.html
    - hp41-gui/src-tauri/gen/apple/project.yml
    - hp41-gui/src-tauri/gen/apple/hp41-gui_iOS/Info.plist
    - hp41-gui/package.json
    - hp41-gui/package-lock.json
decisions:
  - "mobile.json platform-restricted capability: haptics permissions live in capabilities/mobile.json (platforms: [iOS, android]) rather than default.json — Tauri v2.11 build-time permission validation rejects unknown permissions even when the plugin is mobile-target-only"
  - "data-isios attribute: isIos exposed via data-isios on root .calculator element — enables CSS [data-isios] selectors for downstream plans + satisfies TypeScript strict unused-variable check"
  - "portrait lock applied to both project.yml AND Info.plist (Pitfall 6)"
metrics:
  duration: "~5 minutes"
  completed_date: "2026-06-03"
  tasks: 3
  files_modified: 10
---

# Phase 55 Plan 01: iOS Infrastructure Foundation Summary

**One-liner:** is_ios Tauri command + mobile-gated haptics plugin + isIos frontend state + viewport-fit=cover + portrait orientation lock for iOS touch-UI gate infrastructure.

## What Was Built

iOS shared infrastructure that every subsequent Phase-55 plan depends on:

- **Task 1 (794a991):** `is_ios()` Tauri command (mirrors `is_macos`, D-55.1), `permissions/is-ios.toml`, and `"allow-is-ios"` in `capabilities/default.json`. Cargo check run first to regenerate the permission registry (Tauri v2.11 ordering rule).

- **Task 2 (d3706af):** `tauri-plugin-haptics 2.3.2` added under `[target.'cfg(any(target_os = "android", target_os = "ios"))'.dependencies]` in Cargo.toml. `#[cfg(mobile)]` registration block in lib.rs (mirrors the `#[cfg(desktop)]` autostart pattern). Four haptics permissions in a new `capabilities/mobile.json` with `"platforms": ["iOS", "android"]`. npm: `@tauri-apps/plugin-haptics ^2.3.2` was already present in package.json (pre-committed). Both `just gui-ci` (desktop gate) and `cargo check --target aarch64-apple-ios` (cfg(mobile) blind-spot gate) pass.

- **Task 3 (77822c5):** `isIos` state + useEffect in App.tsx (mirrors isMacos pattern), exposed via `data-isios` attribute on root `.calculator` element. `viewport-fit=cover` added to index.html viewport meta (TOUCH-02 — required for `env(safe-area-inset-*)` in WKWebView). Portrait-only orientation lock in both `gen/apple/project.yml` and `gen/apple/hp41-gui_iOS/Info.plist` (Pitfall 6 — must edit both).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] haptics permissions moved to mobile.json instead of default.json**
- **Found during:** Task 2
- **Issue:** The plan specified adding `haptics:allow-*` permissions to `capabilities/default.json`, but Tauri v2.11's build-time permission validation rejects unknown permissions even when the plugin is a mobile-target-only Cargo dependency. The `haptics:*` identifiers are not registered in the desktop permission schema because the plugin isn't compiled for desktop.
- **Fix:** Created `capabilities/mobile.json` with `"platforms": ["iOS", "android"]` restriction containing all four haptics permissions. This is the correct Tauri v2.11 pattern for platform-restricted permissions.
- **Files modified:** `hp41-gui/src-tauri/capabilities/mobile.json` (new), `hp41-gui/src-tauri/capabilities/default.json` (haptics permissions removed)
- **Commit:** d3706af

**2. [Rule 1 - Bug] TypeScript strict mode: isIos declared but never used**
- **Found during:** Task 3
- **Issue:** TypeScript strict mode (`TS6133`) flagged `isIos` as declared-but-never-read because Plan 01 doesn't add any isIos-gated JSX rendering (those land in Plans 02–05).
- **Fix:** Exposed `isIos` via `data-isios={isIos || undefined}` on the root `.calculator` element. This is semantically correct: downstream plans can use CSS `[data-isios]` selectors for iOS-specific styles, and the attribute satisfies TypeScript's usage check. `|| undefined` ensures the attribute is omitted on desktop (false → omitted, true → present).
- **Files modified:** `hp41-gui/src/App.tsx`
- **Commit:** 77822c5

## Verification Results

| Check | Result |
|-------|--------|
| `grep 'fn is_ios' commands.rs` | ✓ line 548 |
| `grep 'commands::is_ios' lib.rs` | ✓ line 181 |
| `grep 'allow-is-ios' permissions/is-ios.toml` | ✓ |
| `grep 'allow-is-ios' capabilities/default.json` | ✓ |
| `grep 'tauri-plugin-haptics' Cargo.toml` | ✓ mobile-target section |
| `grep cfg(mobile) lib.rs` | ✓ 2 occurrences (comment + attribute) |
| `grep 'haptics:allow-impact-feedback' capabilities/mobile.json` | ✓ |
| `grep 'viewport-fit=cover' index.html` | ✓ |
| `grep -c 'invoke<boolean>' App.tsx (is_ios)` | ✓ 1 match |
| project.yml portrait-only | ✓ LandscapeLeft/Right removed |
| Info.plist portrait-only | ✓ 1 `<string>` in UISupportedInterfaceOrientations |
| `just gui-ci` | ✓ 224 tests, release build clean |
| `cargo check --target aarch64-apple-ios` | ✓ 0 errors |

## Known Stubs

None. Plan 01 is pure infrastructure — no UI rendering that could be a stub.

## Threat Flags

No new high-severity threat surface. `is_ios()` returns only a compile-time platform boolean (identical pattern to shipped `is_macos()`). `tauri-plugin-haptics 2.3.2` is an official `tauri-apps` org package, version-pinned. See plan threat model for full STRIDE analysis.

## Self-Check: PASSED

- `hp41-gui/src-tauri/permissions/is-ios.toml` — EXISTS ✓
- `hp41-gui/src-tauri/capabilities/mobile.json` — EXISTS ✓
- Commit 794a991 — EXISTS ✓ (Task 1)
- Commit d3706af — EXISTS ✓ (Task 2)
- Commit 77822c5 — EXISTS ✓ (Task 3)
