---
gsd_state_version: 1.0
milestone: v4.1
milestone_name: iOS Foundation
status: executing
last_updated: "2026-06-03T17:50:00.000Z"
last_activity: 2026-06-03 -- Phase 56 VERIFIED passed (LIFE-01, LIFE-02 + touch R/S parity + CR-01 stale-isIos fix; device re-approved on CR-01-fixed build)
progress:
  total_phases: 5
  completed_phases: 4
  total_plans: 14
  completed_plans: 14
  percent: 80
---

# Project State: HP-41 Calculator Emulator

---

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-29 for v4.1 iOS Foundation)

**Core value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.

**Current focus:** Phase 56 — app-lifecycle-clock

---

## Current Position

Phase: 56 (app-lifecycle-clock) — COMPLETE
Plan: 1/1 complete
Status: Phase 56 complete; Phase 57 is next
Resume file: .planning/phases/56-app-lifecycle-clock/56-01-SUMMARY.md
Last activity: 2026-06-03 -- Phase 56 plan 01 complete (LIFE-01, LIFE-02 + touch R/S fix, device approved)

## Progress Bar

```
v4.1 iOS Foundation
Phase 53 ██████████ 100%  Phase 54 ██████████ 100%  Phase 55 ██████████ 100%
Phase 56 ██████████ 100%  Phase 57 ░░░░░░░░░░  0%   Overall  ████████░░ 80%
```

| Phase | Goal | Status |
|-------|------|--------|
| 53 | Build-Approach Decision + iOS Scaffold Spike | ✅ Complete (4/4; Approach A confirmed, runs on device) |
| 54 | iOS Persistence Layer | ✅ Complete (3/3; iOS sandbox path, autosave on background, kill/relaunch round-trip) |
| 55 | Touch UI Adaptation | ✅ Complete (6/6; all 44 keys ≥44pt, haptics, audio, ALPHA touch, bottom sheets, stack)
| 56 | App Lifecycle + Clock | ✅ Complete (1/1; backgroundThrottling config, resume tick_time, touch R/S fix, device-approved) |
| 57 | Signing + TestFlight Pipeline | Not started |

## Quick Tasks Completed

| # | Description | Date | Commit | Status | Directory |
|---|-------------|------|--------|--------|-----------|
| 260602-kw4 | Eliminate whitespace around the calculator — scale GUI to fill viewport (macOS + iPhone) | 2026-06-02 | dd71fb2 | Complete ✓ | [260602-kw4-eliminate-whitespace-around-the-calculat](./quick/260602-kw4-eliminate-whitespace-around-the-calculat/) |
| 260603-e4e | Allow entering '.1' as '0.1' (leading-zero number entry, real HP-41CV behavior) | 2026-06-03 | d127e97 | Complete ✓ | [260603-e4e-allow-entering-1-as-0-1-leading-zero-num](./quick/260603-e4e-allow-entering-1-as-0-1-leading-zero-num/) |
| 260603-klp | Fix help-overlay search field rendering off-screen at iPhone top edge (safe-area inset) so input and close X are reachable | 2026-06-03 | 8d0fd00 | Complete ✓ | [260603-klp-fix-help-overlay-search-field-rendering-](./quick/260603-klp-fix-help-overlay-search-field-rendering-/) |
| 260603-laz | iOS touch polish — re-fit calculator scale on help/settings overlay close + lock pinch-zoom (surfaced verifying 260603-klp) | 2026-06-03 | 1e485ff | Complete ✓ | [260603-laz-ios-touch-polish-re-fit-calculator-on-ov](./quick/260603-laz-ios-touch-polish-re-fit-calculator-on-ov/) |
| 260603-mxg | iOS PRGM-mode layout — safe-area handled OUTSIDE the CSS-transform (top display no longer clipped under Dynamic Island), via stylesheet class not inline env() (WKWebView drops inline env). Bottom-sheet occlusion later mooted by 260603-o2e | 2026-06-03 | 6fc9d6c | Complete ✓ | [260603-mxg-fix-ios-prgm-mode-layout-program-source-](./quick/260603-mxg-fix-ios-prgm-mode-layout-program-source-/) |
| 260603-o2e | Authentic single-step PRGM view — main display shows current step (SST/BST navigate); removed inauthentic program listing (iOS sheet + desktop panel); restored CLI↔GUI parity D-25.6 | 2026-06-03 | 1eeba7e | Complete ✓ | [260603-o2e-authentic-hp-41-prgm-view-single-program](./quick/260603-o2e-authentic-hp-41-prgm-view-single-program/) |
| 260603-lu0 | Help-overlay function index — A: CLREG→CLRG fidelity; B: tabbed overlay ("Keyboard Shortcuts" \| "All Functions") exposing all 74 keyless built-ins, tap-to-run (XEQ-by-name / insert-step in PRGM), HP-41-styled; C: Vitest+Rust coverage guardrails | 2026-06-03 | 652b4b1 | Complete ✓ | [260603-lu0-help-overlay-function-index-clrg-fidelit](./quick/260603-lu0-help-overlay-function-index-clrg-fidelit/) |
| 260603-s17 | Mnemonic fidelity — CL SIGMA→CLΣ (glyph; data-only, resolver/mirrors already had it); hide CLRALPHA legacy alias from All Functions index via OVERLAY_HIDDEN_ALIASES (Op kept for v1.0 save compat, Pitfall 8) | 2026-06-03 | 8a8e6de | Complete ✓ | [260603-s17-mnemonic-fidelity-cl-sigma-clsigma-glyph](./quick/260603-s17-mnemonic-fidelity-cl-sigma-clsigma-glyph/) |

## Performance Metrics (v4.0 ship baseline)

| Metric | Target | Last measured (v4.0) |
|--------|--------|----------------------|
| Cold-start latency | <= 0.5 s | 2.2 ms (M1) |
| Key-press latency | <= 50 ms | ~65 ns/op |
| `hp41-core` line coverage | >= 95 % | ~93 % (denominator dilution) |
| `hp41-core` region coverage | >= 93 % | ~95 % |
| Numerical accuracy | >= 98 % | 98.86 % (843 cases) |
| Panics in `hp41-core` | 0 | 0 |
| Free42 contamination | 0 | 0 (18-token guard) |
| CI platforms | Win/macOS/Ubuntu | All green |
| Tests passing | — | 3371 (v4.0 baseline) |

---
| Phase 53 P04 | 45 min | 2 tasks | 5 files |
| Phase 55 P01 | 5 | 3 tasks | 10 files |
| Phase 55 P05 | 10 | 2 tasks | 4 files |

## Execution Metrics (Phase 53)

| Plan | Duration | Tasks | Files |
|------|----------|-------|-------|
| Phase 53 P01 | 25 min | 3 tasks | 4 files |
| Phase 53 P02 | 5 min  | 1 task  | 0 files (App ID portal) |
| Phase 53 P03 | 35 min | 3 tasks | 1 files |
| Phase 53 P04 | 45 min | 2 tasks | 5 files |
| Phase 56 P01 | ~40 min | 3 tasks + 1 scope addition | 2 files |

## Accumulated Context

### Decisions (pre-resolved from research)

- **Build approach (RESOLVED 2026-05-31):** Approach A (Tauri v2 Mobile) **confirmed** by the Phase-53 spike — `just ios-build` passed Rust compile + Xcode assembly with no #5865 nested-workspace path error (stopped only at the expected signing gate); the simulator build ran the RPN smoke through `hp41-core`. Approach B (SwiftUI + UniFFI 0.31.1) not pursued. Outcome captured in ADR `docs/adr/v4.1-002-build-approach.md` (note: **-002**; v4.1-001 is the macOS menu-bar ADR).
- **Frozen Invariant:** Both approaches preserve it — root `Cargo.toml` members stay `["hp41-core", "hp41-cli"]`; `tauri`/`tauri-build` confined to `hp41-gui/src-tauri/Cargo.toml` only; `hp41-core` unchanged.
- **No new calculator functions:** Engine is feature-complete at v4.0; this milestone is form-factor only.
- **Persistence path:** iOS uses `app_local_data_dir()` (Tauri) resolving to `Library/Application Support/ch.talent-factory.hp41/autosave.json`; desktop keeps `~/.hp41/autosave.json` unchanged. Workaround for Tauri bug #12552: fall back to `dirs::home_dir()` if `app_local_data_dir()` returns Permission Denied.
- **Autosave on resign-active (iOS):** `document.addEventListener("visibilitychange", ...)` in React `App.tsx` → `invoke("save_state")` on `hidden`. This fires reliably in WKWebView when the user backgrounds the app; the 30s periodic save is supplementary.
- **Touch targets:** Apple HIG minimum 44×44pt. Desktop keys are currently ~40×16px — a ground-up touch layout pass is required. Use transparent hit-area overlays (i41CX+ / Free42 pattern).
- **Haptics:** `tauri-plugin-haptics` 2.3.2 — per-key feedback is a table stake (competitor analysis: Free42, i41CX+, my41CX all include it).
- **Audio:** `AudioContext` must be resumed inside the first user-gesture handler; guard all BEEP/TONE paths with `if (audioCtx.state === 'suspended') await audioCtx.resume()`. Silent-switch muted behavior is accepted as HP-41-faithful.
- **Background throttling (RESOLVED 56-01):** `tauri.ios.conf.json` with `backgroundThrottling: "throttle"` (camelCase, no Policy suffix — the Rust type name `backgroundThrottlingPolicy` would silently no-op). On iOS 16 and below, timers pause in background — accepted. Array-replace footgun: override must repeat all `app.windows[0]` fields from `tauri.conf.json`.
- **Clock on resume (RESOLVED 56-01):** `needsTickRef` + extended `visibilitychange` `visible` branch fires one `invoke('tick_time')` gated on `isIos && needsTickRef.current && !busyRef.current`. NOT `get_state` (D-11). Resume tick is iOS-only and needsTick-gated; desktop/macOS fire no spurious IPC. Verified on-device (iPhone 15 Pro, iOS 17+).
- **Touch R/S stopwatch keyboard mode (RESOLVED 56-01, scope addition):** Phase 41 gap — stopwatch keyboard-mode interception existed only in physical `handleKey` path; `handleClick` had no block. On-screen R/S fell through to `run_stop` (wrong op). Fixed by mirroring the block in `handleClick`: R/S→RUNSW/STOPSW toggle, ENTER→STPW, other→sw_exit. Touch-only; D-25.6 CLI↔GUI parity unaffected.
- **Bundle ID:** `ch.talent-factory.hp41` — already in `tauri.conf.json`; must be registered as App ID in App Store Connect before any signing (Phase 53 task).
- **PrivacyInfo.xcprivacy:** Required before first TestFlight upload (ITMS-91053). Create in `gen/apple/` with `NSPrivacyAccessedAPICategoryFileTimestamp` + reason `C617.1` (Phase 57).
- **ALPHA touch entry:** Design spike at start of Phase 55. Options: `<input type="text">` + `window.visualViewport` listener to stay above the iOS software keyboard; or an on-screen character grid that avoids the system keyboard entirely.
- **BottomSheet pattern (D-55.5, Plan 05):** early-return-null when visible=false; App.tsx gates mounting with isIos — desktop never sees the component. Desktop inline panels preserved byte-for-byte via isIos ternary.
- **Collapsible stack (D-55.5, Plan 05):** X row always visible on iOS; Y/Z/T/L in .stack-panel-collapsible.collapsed; stackExpanded is local state only (not persisted to GuiPrefs; resets to collapsed on each launch).
- **Phase dependency order:** 53 → (54, 55 can overlap — different files) → 56 → 57.
- **iOS build env (P-iOS-09, RESOLVED 53-04):** the Xcode "Build Rust Code" phase sources `gen/apple/.xcode.env` (committed, `$HOME/.cargo/bin`) + `.xcode.env.local` (gitignored, nvm node) so GUI-launched Xcode finds cargo/node. Reuse this pattern in Phase 57 CI.
- **iOS signing (53-04):** `DEVELOPMENT_TEAM 2P4R8QSWT4` (Talent Factory AG) + automatic signing committed in `gen/apple/project.yml`. App ID `ch.talent-factory.hp41` registered (53-02).
- **Device-install caveat (53-04):** Xcode debug ⌘R panics on the missing Tauri dev-server addr file (expects `tauri ios dev`). For standalone installs use release `just ios-build` IPA + `xcrun devicectl install/launch`.

### Pitfalls to watch (iOS-specific)

| ID | Description | Phase |
|----|-------------|-------|
| P-iOS-01 | Simulator device name hardcoded in cargo-mobile2 — breaks after Xcode upgrade | 53 |
| P-iOS-02 | Physical device debug via Tauri CLI requires LAN + Xcode Devices pre-setup | 53 |
| P-iOS-03 | Nested workspace breaks Tauri iOS bundler (#5865) — the spike gating task | 53 |
| P-iOS-05 | Web Audio suspended until first user gesture; BEEP/TONE silent | 55 |
| P-iOS-06 | WKWebView: safe area not applied; keyboard overlaps viewport; fixed-position flicker | 55 |
| P-iOS-07 | Wrong Rust target triple: `aarch64-apple-ios` vs `aarch64-apple-ios-sim` | 53 |
| P-iOS-09 | Xcode build phase cannot find `cargo` (PATH not inherited from shell) | 53 |
| P-iOS-14 | Bundle ID not registered in App Store Connect before first CI run | 53 |
| P-iOS-16 | Keychain access fails on macOS CI runners (GitHub Actions) | 57 |
| P-iOS-18 | Works in Simulator, fails on device or TestFlight — checklist of 5 root causes | 57 |
| P-iOS-19 | `PrivacyInfo.xcprivacy` required; ITMS-91053 on first upload without it | 57 |
| P-iOS-20 | Desktop key targets ~40×16px — far below Apple HIG 44×44pt minimum | 55 |
| P-iOS-23 | ALPHA entry has no physical keyboard on iPhone — design spike required | 55 |
| P-iOS-27 | Background suspension freezes clock UI; need become-active → tick_time | 56 |
| P-iOS-28 | `~/.hp41/autosave.json` does not exist on iOS — silent data loss on first launch | 54 |
| P-iOS-29 | Absolute paths must never be persisted in CalcState | 54 |
| P-iOS-31 | Schema migration on TestFlight update — `#[serde(default)]` invariant must hold | 54 |

### Blockers

None.

### Pending Todos

- Run `/gsd-plan-phase 53` to plan Phase 53: Build-Approach Decision + iOS Scaffold Spike
- After Phase 53 spike outcome: review whether Phase 55 approach changes (Approach B → SwiftUI keyboard instead of CSS adaptation)

---

## Deferred Items

| Category | Item | Status |
|----------|------|--------|
| Deferred | Interrupting control alarm execution | Still deferred; data model ready (D-38.4); requires call-stack re-entrancy |
| Deferred | Signed binary releases (cargo-dist + tauri-action) | Deferred post-v4.1 |
| Deferred | App Store submission (STORE-01, STORE-02) | v4.2+ milestone |
| Deferred | iPad universal layout (IPAD-01) | v4.2+ milestone |
| Deferred | Landscape orientation (LAND-01) | v4.2+ milestone |
| Deferred | Android (ANDROID-01) | v4.2+ milestone |
| Deferred | .raw file picker on iOS (RAW-IOS-01) | v4.2+ (no native iOS picker in tauri-plugin-dialog) |

---

*State initialized: 2026-05-06*
*Last updated: 2026-05-29 — v4.1 iOS Foundation roadmap created (Phases 53–57, 25 requirements, 5 phases). Next: `/gsd-plan-phase 53`.*
