# Requirements: HP-41 Calculator Emulator — v4.1 iOS Foundation

**Defined:** 2026-05-29
**Core Value:** Faithful HP-41 RPN fidelity — the four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to the original hardware; everything else is secondary.

**Milestone goal:** Bring the emulator to iPhone with a touch-first UI and a working signed build distributed via TestFlight — foundation for a later App Store release. `hp41-core` is feature-complete at v4.0; **no new calculator functions** are added. The iOS build is another adapter on the unchanged core; the workspace Frozen Invariant is preserved.

**Build approach:** Approach A (Tauri v2 Mobile — reuse React UI + `hp41-core`) is recommended, **contingent on a Phase-1 spike** verifying the Tauri iOS nested-workspace bundler (#5865) works in v2.11. Fallback: Approach B (native SwiftUI + Rust FFI via UniFFI). The decision is recorded in `docs/adr/v4.1-001-build-approach.md`. See `.planning/research/SUMMARY.md`.

## v4.1 Requirements

Requirements for this milestone. Each maps to a roadmap phase.

### Build Pipeline

- [x] **BUILD-01**: Build approach (Tauri v2 Mobile vs. native SwiftUI + Rust FFI) is decided via a hands-on `cargo tauri ios build` spike and captured in ADR `v4.1-001-build-approach.md`
- [x] **BUILD-02**: `hp41-core` compiles to the iOS targets and the app builds and launches in the iOS Simulator (one key dispatches through the existing engine)
- [x] **BUILD-03**: The app builds, installs, and runs on a physical iPhone
- [x] **BUILD-04**: Bundle ID `ch.talent-factory.hp41` is registered in App Store Connect (App ID + app record)

### Persistence

- [ ] **PERSIST-01**: Autosave/load uses the iOS app-sandbox container path instead of `~/.hp41/` (desktop path behavior unchanged)
- [ ] **PERSIST-02**: State is autosaved when the app is backgrounded (resign-active / `visibilitychange`)
- [ ] **PERSIST-03**: State survives a background → kill → relaunch cycle on device, and existing desktop save files still load (serde backward-compat preserved)

### Touch UI

- [ ] **TOUCH-01**: All 44 calculator keys are tappable with hit targets ≥ 44×44 pt
- [x] **TOUCH-02**: Portrait-locked layout respects safe-area insets (notch / Dynamic Island / home indicator)
- [ ] **TOUCH-03**: Keys give immediate press feedback (`:active`), the 350 ms tap delay is eliminated (`touch-action: manipulation`), and there is no tap-highlight flash
- [ ] **TOUCH-04**: ALPHA-mode character entry works by touch (iOS software keyboard or on-screen grid), with the input staying visible above the keyboard
- [x] **TOUCH-05**: Key presses produce haptic feedback
- [ ] **TOUCH-06**: Audio (TONE/BEEP) resumes correctly after the first user gesture (iOS AudioContext rule)
- [ ] **TOUCH-07**: SHIFT/ALPHA indicators and the X/Y/Z/T stack panel are visible and legible on iPhone
- [ ] **TOUCH-08**: Haptic intensity varies by key type, and a distinct haptic fires on DATA ERROR / NO ROOM *(differentiator)*
- [ ] **TOUCH-09**: The print panel and PRGM-mode program listing are presented as pull-up bottom sheets *(differentiator)*
- [ ] **TOUCH-10**: The stack panel can be collapsed to give the keypad more room *(differentiator)*
- [ ] **TOUCH-11**: iOS rubber-band overscroll is suppressed on the calculator body *(differentiator)*

### Lifecycle & Clock

- [ ] **LIFE-01**: `backgroundThrottlingPolicy` is configured so the WKWebView is not fully suspended on brief backgrounding (iOS 17+)
- [ ] **LIFE-02**: The clock/stopwatch display refreshes immediately on return to foreground (forced `tick_time` on become-active); foreground-only live updates are accepted as HP-41CX-faithful

### Signing & TestFlight

- [ ] **SHIP-01**: A distribution certificate + provisioning profile produce a signed IPA
- [ ] **SHIP-02**: `PrivacyInfo.xcprivacy` is present in `gen/apple/` with the file-timestamp accessed-API reason (`NSPrivacyAccessedAPICategoryFileTimestamp` / `C617.1`)
- [ ] **SHIP-03**: The app has an app icon and a launch screen
- [ ] **SHIP-04**: A `ci-ios.yml` GitHub Actions workflow (macOS runner) builds the signed IPA and uploads it via `xcrun altool`
- [ ] **SHIP-05**: A build is distributed to testers via TestFlight (internal distribution)

## Future Requirements

Deferred to a follow-up milestone. Tracked but not in this roadmap.

### App Store Release

- **STORE-01**: App Store submission (store assets, screenshots, privacy policy, Apple review)
- **STORE-02**: "HP-41" trademark / app-naming decision for public listing

### Platform Breadth

- **IPAD-01**: iPad universal layout
- **LAND-01**: Landscape orientation support
- **ANDROID-01**: Android target (Tauri v2 can also target Android)

### File Exchange on iOS

- **RAW-IOS-01**: `.raw` program import/export on iOS via the Files app (`tauri-plugin-dialog` has no native iOS picker today)

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| App Store submission | Foundation milestone targets TestFlight only; submission + trademark navigation deferred to a follow-up |
| iPad layout / landscape | iPhone-only foundation; universal layout is separate work |
| Android | iPhone-only this milestone; Tauri can revisit Android later |
| `.raw` file picker on iOS | No native iOS picker in `tauri-plugin-dialog`; non-blocking for TestFlight |
| New calculator functions | Engine is feature-complete at v4.0; this milestone is form-factor only |
| iCloud sync, Widgets, Siri, Watch companion | Out of foundation scope; privacy/effort |
| Background calculator execution | iOS suspends background apps; HP-41CX hardware also stopped when unpowered |

## Traceability

Which phases cover which requirements. Filled in during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| BUILD-01 | Phase 53 | Complete |
| BUILD-02 | Phase 53 | Complete |
| BUILD-03 | Phase 53 | Complete |
| BUILD-04 | Phase 53 | Complete |
| PERSIST-01 | Phase 54 | Pending |
| PERSIST-02 | Phase 54 | Pending |
| PERSIST-03 | Phase 54 | Pending |
| TOUCH-01 | Phase 55 | Pending |
| TOUCH-02 | Phase 55 | Complete |
| TOUCH-03 | Phase 55 | Pending |
| TOUCH-04 | Phase 55 | Pending |
| TOUCH-05 | Phase 55 | Complete |
| TOUCH-06 | Phase 55 | Pending |
| TOUCH-07 | Phase 55 | Pending |
| TOUCH-08 | Phase 55 | Pending |
| TOUCH-09 | Phase 55 | Pending |
| TOUCH-10 | Phase 55 | Pending |
| TOUCH-11 | Phase 55 | Pending |
| LIFE-01 | Phase 56 | Pending |
| LIFE-02 | Phase 56 | Pending |
| SHIP-01 | Phase 57 | Pending |
| SHIP-02 | Phase 57 | Pending |
| SHIP-03 | Phase 57 | Pending |
| SHIP-04 | Phase 57 | Pending |
| SHIP-05 | Phase 57 | Pending |

**Coverage:**
- v4.1 requirements: 25 total
- Mapped to phases: 25
- Unmapped: 0 ✓

---
*Requirements defined: 2026-05-29*
*Last updated: 2026-05-29 — Traceability table filled; all 25 requirements mapped to Phases 53–57*
