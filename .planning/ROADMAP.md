# Roadmap: HP-41 Calculator Emulator

**Project:** HP-41 Calculator Emulator

---

## Milestones

- ✅ **v1.0 CLI** — Phases 1–8, foundational RPN engine + TUI — SHIPPED 2026-05-08 · [Archive](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 CLI Feature Completeness** — Phases 9–12, EEX fix / STO modals / print / synthetic — SHIPPED 2026-05-09 · [Archive](milestones/v1.1-ROADMAP.md)
- ✅ **v2.0 Tauri GUI** — Phases 13–18, pixel-perfect HP-41C desktop app — SHIPPED 2026-05-10 · [Archive](milestones/v2.0-ROADMAP.md)
- ✅ **v2.1 Card Reader + Keyboard Authenticity** — quick-task entries (no Phase 19 GSD directory) — SHIPPED 2026-05-13 · see MILESTONES.md
- ✅ **v2.2 HP-41CV Feature Completeness** — Phases 20–27, full ROM built-in set + JSON pipeline + GUI integration + coverage gate raise — SHIPPED 2026-05-15 · [Archive](milestones/v2.2-ROADMAP.md)
- ✅ **v3.0 Math Pac I Emulation** — Phases 28–32, first XROM application module (10 prompt-driven programs, ~55 XEQ entry points, 95.39 % line coverage, 99.3 % numerical accuracy) — SHIPPED 2026-05-20 · [Archive](milestones/v3.0-ROADMAP.md)
- ✅ **v3.1 Stat 1 Pac Emulation** — Phases 33–37, second XROM application module (13 programs, 26 XEQ entry points, RAND/SEED extension, 98.86 % numerical accuracy) — SHIPPED 2026-05-24 · [Archive](milestones/v3.1-ROADMAP.md)
- ✅ **v3.2 Time Pac Emulation** — Phases 38–42, third XROM application module (HP 82182A Time Module, XROM 26, 35 XEQ entry points, first real-time behavior, 96.01% region coverage) — SHIPPED 2026-05-25 · [Archive](milestones/v3.2-ROADMAP.md)
- ✅ **v3.3 Advantage Pac Emulation** — Phases 43–47, fourth XROM application module (XROM 22 + XROM 24, 114 XEQ entry points: bitwise/base conversion, named-matrix operations, advanced math/complex/solver/curve-fit, TVM) — SHIPPED 2026-05-26 · [Archive](milestones/v3.3-ROADMAP.md)
- ✅ **v4.0 Platform Maturity** — Phases 48–52, visual themes, onboarding, GUI keyboard parity, `.raw` file I/O, Extended Memory — SHIPPED 2026-05-28 · [Archive](milestones/v4.0-ROADMAP.md)
- 🚧 **v4.1 iOS Foundation** — Phases 53–57, touch-first iPhone build to TestFlight — IN PROGRESS

---

## Phases

<details>
<summary>✅ v1.0 CLI (Phases 1–8) — SHIPPED 2026-05-08</summary>

See [milestones/v1.0-ROADMAP.md](milestones/v1.0-ROADMAP.md) for full phase detail.

</details>

<details>
<summary>✅ v1.1 CLI Feature Completeness (Phases 9–12) — SHIPPED 2026-05-09</summary>

See [milestones/v1.1-ROADMAP.md](milestones/v1.1-ROADMAP.md) for full phase detail. Phase plans archived at `milestones/v1.1-phases/`.

</details>

<details>
<summary>✅ v2.0 Tauri GUI (Phases 13–18) — SHIPPED 2026-05-10</summary>

See [milestones/v2.0-ROADMAP.md](milestones/v2.0-ROADMAP.md) for full phase detail. Phase plans archived at `milestones/v2.0-phases/`.

</details>

<details>
<summary>✅ v2.1 Card Reader + Keyboard Authenticity — SHIPPED 2026-05-13</summary>

Recorded as quick-task entries in MILESTONES.md (no Phase 19 GSD directory; scope evolved out-of-band).

</details>

<details>
<summary>✅ v2.2 HP-41CV Feature Completeness (Phases 20–27) — SHIPPED 2026-05-15</summary>

See [milestones/v2.2-ROADMAP.md](milestones/v2.2-ROADMAP.md) for full phase detail. Phase plans archived at `milestones/v2.2-phases/`.

</details>

<details>
<summary>✅ v3.0 Math Pac I Emulation (Phases 28–32) — SHIPPED 2026-05-20</summary>

See [milestones/v3.0-ROADMAP.md](milestones/v3.0-ROADMAP.md) for full phase detail. Phase plans archived at `milestones/v3.0-phases/`.

</details>

<details>
<summary>✅ v3.1 Stat 1 Pac Emulation (Phases 33–37) — SHIPPED 2026-05-24</summary>

See [milestones/v3.1-ROADMAP.md](milestones/v3.1-ROADMAP.md) for full phase detail. Phase plans archived at `milestones/v3.1-phases/`.

</details>

<details>
<summary>✅ v3.2 Time Pac Emulation (Phases 38–42) — SHIPPED 2026-05-25</summary>

See [milestones/v3.2-ROADMAP.md](milestones/v3.2-ROADMAP.md) for full phase detail. Phase plans archived at `milestones/v3.2-phases/`.

</details>

<details>
<summary>✅ v3.3 Advantage Pac Emulation (Phases 43–47) — SHIPPED 2026-05-26</summary>

See [milestones/v3.3-ROADMAP.md](milestones/v3.3-ROADMAP.md) for full phase detail. Phase plans archived at `milestones/v3.3-phases/`.

</details>

<details>
<summary>✅ v4.0 Platform Maturity (Phases 48–52) — SHIPPED 2026-05-28</summary>

See [milestones/v4.0-ROADMAP.md](milestones/v4.0-ROADMAP.md) for full phase detail. Phase plans archived at `milestones/v4.0-phases/`.

</details>

### v4.1 iOS Foundation (Phases 53–57)

- [x] **Phase 53: Build-Approach Decision + iOS Scaffold Spike** — Hands-on `cargo tauri ios build` spike resolves nested-workspace bug (#5865); ADR `v4.1-002-build-approach.md` locked; bundle ID registered; app runs in Simulator with one key dispatching through the engine (completed 2026-06-02)
- [x] **Phase 54: iOS Persistence Layer** — iOS sandbox path replaces `~/.hp41`; autosave fires on backgrounding; state survives kill/relaunch; desktop path unchanged; serde backward-compat preserved (completed 2026-06-02)
- [x] **Phase 55: Touch UI Adaptation** — All 44 keys at ≥44pt; portrait layout with safe-area insets; press feedback + tap-delay elimination; ALPHA touch entry; haptics; audio resume; SHIFT/stack visibility; bottom sheets; collapsible stack; overscroll suppression (completed 2026-06-03)
- [ ] **Phase 56: App Lifecycle + Clock** — `backgroundThrottlingPolicy` configured; clock display refreshes immediately on foreground return via forced `tick_time`
- [ ] **Phase 57: Signing + TestFlight Pipeline** — Distribution cert + provisioning profile; `PrivacyInfo.xcprivacy`; app icon + launch screen; `ci-ios.yml` GitHub Actions workflow; build uploaded and available in TestFlight

---

## Phase Details

### Phase 53: Build-Approach Decision + iOS Scaffold Spike
**Goal**: The iOS build approach is decided and the app runs on both the Simulator and a physical device with one key dispatching through the engine
**Depends on**: Nothing (first phase of v4.1; gates all subsequent phases)
**Requirements**: BUILD-01, BUILD-02, BUILD-03, BUILD-04
**Success Criteria** (what must be TRUE):
  1. Running `cargo tauri ios build` inside `hp41-gui/` either succeeds (Approach A confirmed) or fails at the Xcode assembly step with the nested-workspace path error (Approach B fallback triggered), and ADR `docs/adr/v4.1-002-build-approach.md` records which approach was chosen and why
  2. The app launches in the iOS Simulator and tapping one calculator key (e.g., SIN) dispatches through the existing `hp41-core` engine and updates the display
  3. The same signed debug build installs and runs on a physical iPhone connected via Xcode
  4. Bundle ID `ch.talent-factory.hp41` is registered as an explicit App ID in App Store Connect (Identifiers) and as an app record, unblocking any future provisioning profile creation
**Plans**: 4 plans
- [x] 53-01-PLAN.md — Toolchain preflight, crate-type edit, `just ios-*` recipes, `tauri ios init`, gen/apple gitignore policy
- [x] 53-02-PLAN.md — Register bundle ID `ch.talent-factory.hp41` in App Store Connect (App ID + app record) [checkpoint]
- [x] 53-03-PLAN.md — Decisive `cargo tauri ios build` spike, Simulator RPN smoke, write ADR v4.1-002-build-approach.md
- [x] 53-04-PLAN.md — Physical-iPhone signing-team selection + on-device RPN smoke [checkpoint]
**UI hint**: yes

**Note**: The spike outcome in this phase may require altering the scope or approach of Phases 54–57 (particularly Phase 55 if Approach B is chosen, which requires a new SwiftUI keyboard instead of CSS adaptation). ADR `v4.1-002-build-approach.md` must be written before planning subsequent phases in detail.

---

### Phase 54: iOS Persistence Layer
**Goal**: Calculator state is saved and restored correctly inside the iOS app sandbox, with no regression on the desktop path
**Depends on**: Phase 53
**Requirements**: PERSIST-01, PERSIST-02, PERSIST-03
**Success Criteria** (what must be TRUE):
  1. After a session on the iPhone (Simulator or device), killing and relaunching the app restores the exact calculator state (X/Y/Z/T registers, program memory, XROM module state) that existed before the kill
  2. Moving the app to the background (pressing the Home button or switching apps) triggers an immediate autosave; the 30-second periodic timer alone is not relied upon for iOS
  3. The save file is written to the iOS app-sandbox container path (not `~/.hp41/`), and the desktop app continues reading from `~/.hp41/autosave.json` unaffected
  4. A v4.0 `autosave.json` from the desktop loads without error in the iOS build (serde backward-compat preserved; `#[serde(default)]` policy verified)
**Plans**: 3 plans
- [x] 54-01-PLAN.md — AppHandle-aware path resolvers (`state_path_for_app` / `prefs_path_for_app`) + all five call-site rewrites + v4.0 backward-compat fixture/test + P-iOS-29 audit (PERSIST-01, PERSIST-03)
- [x] 54-02-PLAN.md — `visibilitychange` → `invoke('save_state')` background-save listener in App.tsx (PERSIST-02)
- [x] 54-03-PLAN.md — On-device build + background→kill→relaunch round-trip + #12552 / AppDataWrite resolution [checkpoints] (PERSIST-01/02/03)

---

### Phase 55: Touch UI Adaptation
**Goal**: The calculator is fully operable by touch on an iPhone, with all 44 keys reachable, native iOS feedback, and the layout respecting the device form factor
**Depends on**: Phase 53 (approach confirmed); Phase 54 can overlap (different files)
**Requirements**: TOUCH-01, TOUCH-02, TOUCH-03, TOUCH-04, TOUCH-05, TOUCH-06, TOUCH-07, TOUCH-08, TOUCH-09, TOUCH-10, TOUCH-11
**Success Criteria** (what must be TRUE):
  1. Every one of the 44 calculator keys can be tapped accurately on an iPhone SE (smallest iPhone) without accidental adjacent key activation; hit targets are ≥44×44pt with the layout respecting safe-area insets (no keys hidden behind the notch, Dynamic Island, or home indicator)
  2. Tapping a key produces immediate visual press feedback (`:active` state), no tap-highlight flash, and no perceptible 300ms delay; the first tap and rapid successive taps all feel equally responsive
  3. ALPHA-mode character entry works without a hardware keyboard: an on-screen text input or character grid is presented, stays visible above the iOS software keyboard, and characters are correctly committed into the ALPHA register
  4. Every key tap triggers haptic feedback; feedback intensity varies by key type (standard vs. function vs. ENTER); a distinct haptic pattern fires when the display shows DATA ERROR or NO ROOM
  5. TONE n / BEEP produce audio on the first use after launch (AudioContext is resumed inside the first user-gesture handler; the silent-switch behavior is documented and accepted)
  6. The SHIFT and ALPHA annunciators and the X/Y/Z/T stack panel are visible and legible without zooming; the print panel and PRGM-mode listing are accessible via pull-up bottom sheets; the stack panel can be collapsed to give the keypad more vertical room; rubber-band overscroll is suppressed on the calculator body
**Plans**: 6 plans
- [x] 55-01-PLAN.md — iOS shared infra: is_ios command + haptics plugin (mobile-gated) + isIos flag + viewport-fit=cover + portrait lock
- [x] 55-02-PLAN.md — 44pt hit-target overlays + tap feedback (pointerdown, no delay/flash) + safe-area padding
- [x] 55-03-PLAN.md — Per-key haptic tiers + error haptic + audio resume (verified impactFeedback string API)
- [x] 55-04-PLAN.md — ALPHA + modal-label touch text entry (AlphaTouchInput, D-55.2)
- [x] 55-05-PLAN.md — Pull-up bottom sheets (print/PRGM) + collapsible stack + overscroll
- [x] 55-06-PLAN.md — On-device iPhone SE verification checkpoints (D-55.4) [checkpoints]
**UI hint**: yes

---

### Phase 56: App Lifecycle + Clock
**Goal**: The app's WKWebView is not fully suspended on brief backgrounding, and the clock/stopwatch display is immediately correct when the user returns to the app
**Depends on**: Phase 55 (touch UI in place; lifecycle polish layered on top)
**Requirements**: LIFE-01, LIFE-02
**Success Criteria** (what must be TRUE):
  1. After briefly backgrounding and returning to the app (not a kill/relaunch), the clock display updates to the current time within one rendering frame — no stale time is visible even momentarily
  2. The `backgroundThrottlingPolicy` configuration prevents the WKWebView from fully suspending during brief backgrounding on iOS 17+; the setting is present in `tauri.ios.conf.json` and the behavior is verified on a real device
**Plans**: TBD
**UI hint**: yes

---

### Phase 57: Signing + TestFlight Pipeline
**Goal**: A signed IPA is produced by CI and at least one build is distributed to testers via TestFlight internal distribution
**Depends on**: Phase 56 (full app ready for distribution); Phase 53 (bundle ID registered, unblocks signing)
**Requirements**: SHIP-01, SHIP-02, SHIP-03, SHIP-04, SHIP-05
**Success Criteria** (what must be TRUE):
  1. A signed IPA is produced by `cargo tauri ios build --export-method app-store-connect` using a distribution certificate and provisioning profile for `ch.talent-factory.hp41`; the IPA installs and runs on a real iPhone
  2. `gen/apple/PrivacyInfo.xcprivacy` declares `NSPrivacyAccessedAPICategoryFileTimestamp` with reason `C617.1`; an upload to TestFlight does not trigger ITMS-91053
  3. The app has a custom app icon (all required sizes) and a launch screen (not a blank white screen)
  4. `ci-ios.yml` runs on a macOS GitHub Actions runner, builds the signed IPA, and uploads it via `xcrun altool` on every push to `main`
  5. At least one internal TestFlight build is distributed to testers and the app can be installed from TestFlight on a real iPhone
**Plans**: TBD

---

## Progress Table

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 53. Build-Approach Decision + iOS Scaffold Spike | 4/4 | Complete   | 2026-06-02 |
| 54. iOS Persistence Layer | 3/3 | Complete   | 2026-06-02 |
| 55. Touch UI Adaptation | 6/6 | Complete    | 2026-06-03 |
| 56. App Lifecycle + Clock | 0/? | Not started | - |
| 57. Signing + TestFlight Pipeline | 0/? | Not started | - |

---

*Last updated: 2026-06-03 — Phase 55 planned (6 plans, 6 waves). Next: `/gsd-execute-phase 55`.*
