# Feature Research: v4.1 iOS Foundation

**Domain:** Touch-first iPhone port of the HP-41 calculator emulator (Tauri v2 + React GUI adaptation for iOS)
**Researched:** 2026-05-29
**Confidence:** MEDIUM-HIGH — Tauri v2 mobile iOS behavior is HIGH confidence (official docs + community WKWebView gotcha catalog); iOS HIG touch targets / safe areas are HIGH confidence (Apple official); iOS lifecycle autosave patterns are HIGH confidence (Apple UIApplicationDelegate docs); HP-41-specific iOS emulator UI patterns are MEDIUM confidence (App Store observations + competitor analysis).

---

## Overview

v4.1 "iOS Foundation" is a platform port, not a feature addition milestone. The HP-41 engine (`hp41-core`) is feature-complete at v4.0. This milestone's job is to get the existing Tauri v2 + React GUI running touch-first on an iPhone and delivered to TestFlight. No new calculator functions. No App Store submission. No iPad layout. No Android.

The existing desktop GUI (`hp41-gui`) is already architecturally thin: it renders SVG keys via `Keyboard.tsx`, surfaces state through `CalcStateView` returned from Tauri commands, and all calculator logic lives in `hp41-core`. The iPhone build is another adapter on top of the same core.

**Build approach gate:** The choice between Tauri v2 Mobile (WKWebView + existing React) versus native SwiftUI + Rust FFI is an unresolved ADR (to be decided in a separate STACK research file). Many feature items below note which approach they depend on. Where they differ, both paths are described.

---

## 1. Touch Interaction Model

### What desktop affordances must be replaced

The desktop GUI relies on affordances that do not exist on iPhone:

| Desktop Affordance | iPhone Replacement Required |
|--------------------|----------------------------|
| Physical keyboard (letters, digits, ENTER, SHIFT) | On-screen key taps only; hardware keyboard can remain as a bonus |
| Mouse hover (`:hover` CSS state) | Remove or guard with `@media (hover: hover)`; use `:active` for press feedback |
| Right-click / context menu | Long-press gesture OR explicit secondary UI (not needed for v4.1) |
| Window resize / custom window size | Fixed portrait layout; no resize API |
| `?` keyboard-shortcut overlay (opened by pressing `?` on physical keyboard) | Replace with a dedicated tappable button (≥44pt) to open the function reference |
| `p` key for PRGM mode, `s` for SHIFT | All modes must be reachable via tappable on-screen controls |
| Mouse-wheel scroll in print panel / program listing | Touch scroll (native iOS scroll momentum) |

### Touch target sizing

Apple HIG and WCAG 2.5.5 both specify **minimum 44×44 pt** for all interactive controls. On a 2× Retina display this is 88×88 px. The existing SVG keyboard renders within a ~440×1020 desktop window. The HP-41 has 5 rows × 9 columns = 44 keys (plus 4 top mode keys). In a ~390pt-wide portrait layout, each key must be at least 44pt wide; at 9 columns that is ~43pt per key, which is right at the minimum. Keys that are currently smaller in the SVG skin (shifted label overlay regions, narrow side keys) need enlargement or touch-area padding.

**Pattern used by competitor HP-41 emulators (i41CX+, my41CX):** Overlay the HP-41 skin bitmap/SVG with transparent touch-area buttons that are larger than the visible key art, rather than resizing key art to fill the screen. This preserves authentic appearance while meeting touch target minimums.

### Press feedback requirements

iOS users expect **instant visual feedback on tap-down** (not tap-up). Safari/WKWebView does not apply CSS `:active` by default unless `touchstart` is listened to. Two implementation patterns:

1. **Tauri Mobile / WKWebView path:** Add a `touchstart` listener (empty is sufficient to activate `:active` on the body); apply `transform: scale(0.92)` or a background-color change in the `:active` CSS rule on each key button. The `touch-action: manipulation` CSS property eliminates the 350ms tap delay and is required on every tappable element.
2. **SwiftUI path:** SwiftUI buttons provide `.buttonStyle` with `isPressed` state natively; combine with `.sensoryFeedback` (iOS 17+).

Disable the default iOS tap highlight: `-webkit-tap-highlight-color: transparent` on key buttons to prevent the grey flash.

### Haptic feedback

`tauri-plugin-haptics` (v2.3.2) exposes four functions on iOS:
- `selectionFeedback()` — maps to `UISelectionFeedbackGenerator`; ideal for individual key presses
- `impactFeedback('light' | 'medium' | 'heavy' | 'rigid' | 'soft')` — maps to `UIImpactFeedbackGenerator`; use `'light'` for most keys, `'medium'` for ENTER
- `notificationFeedback('success' | 'warning' | 'error')` — use for error conditions (DATA ERROR, NO ROOM)
- `vibrate(duration)` — raw vibration; not preferred for per-key feedback

**Required capability permissions** in `capabilities/default.json`:
- `haptics:allow-selection-feedback`
- `haptics:allow-impact-feedback`
- `haptics:allow-notification-feedback`

On the SwiftUI path, haptics are called directly via `UIFeedbackGenerator` in Swift without a plugin.

Haptic feedback per key press is a user expectation for calculator apps on iPhone (competitor apps i41CX+, Free42, iHP48 all implement it). Its absence makes the app feel unfinished.

### What NOT to implement in touch interaction for this milestone

- Multi-touch gestures (pinch-to-zoom, swipe-to-dismiss, two-finger scroll) — defer
- Long-press context menu on keys — defer
- Drag-and-drop between registers — defer

---

## 2. iPhone Layout and Safe Areas

### Existing geometry

The desktop GUI runs in a 440×1020 window (`hp41-gui` enlarged to this size in 2026-05-28 commit). This aspect ratio is already phone-shaped — the desktop window is essentially a tall rectangle. iPhone 15 Pro is 393pt × 852pt logical. This is close to the existing 440×1020 ratio, which is a significant advantage: the SVG keyboard skin and CSS layout need adaptation, not a full redesign.

### Safe area insets

Modern iPhones (Face ID models with Dynamic Island or notch, Plus/Max models with home indicator) use safe area insets that must be respected:

| Region | Inset (approx.) | Affected component |
|--------|----------------|--------------------|
| Status bar / Dynamic Island | ~59pt top | LCD / display area |
| Home indicator | ~34pt bottom | Bottom key row |
| Side rails | 0pt (portrait) | Key columns |

Implementation (Tauri Mobile / WKWebView path):
```css
/* viewport-fit=cover is required in <meta name="viewport"> */
padding-top: env(safe-area-inset-top);
padding-bottom: env(safe-area-inset-bottom);
```

**Known WKWebView bug:** `env(safe-area-inset-*)` values may not be set until a frame after first paint, causing a brief layout flash. Mitigation: use `max(env(safe-area-inset-top), 20px)` as a fallback, or apply padding on `DOMContentLoaded` via a `requestAnimationFrame` callback.

**Rotation issues:** Safe area values may not update correctly after device rotation. Since this milestone targets portrait-only, lock orientation in `Info.plist` with `UISupportedInterfaceOrientations = [UIInterfaceOrientationPortrait]`. This eliminates the rotation bug entirely.

### How much of the SVG skin survives

The SVG skin in `Keyboard.tsx` is a programmatic SVG (generated by React, not an image file). It renders within a flex container. Expected survival rate:

- **LCD display area** — survives unchanged; position needs safe-area top padding
- **Annunciators** — survive; check they remain visible above Dynamic Island
- **Key grid (5×9 main + 4 top mode keys)** — survives; needs CSS `width: 100%` and `height: auto` responsive scaling rather than fixed pixel dimensions
- **Key label text** — survives; may need `font-size` increase for legibility at smaller physical sizes
- **ENTER double-width key** — survives; double-width already in SVG structure
- **Stack panel (X/Y/Z/T/LASTX)** — survives; relocate above keyboard in a scrollable zone or make it collapsible
- **Print panel** — needs adaptation (see section 4)
- **PRGM listing** — needs adaptation (see section 4)

The three-label key design (primary / shifted / alphaChar) means key labels are already dense. On a phone, the shifted and alpha labels may become illegible. Acceptable mitigation for the foundation milestone: keep them as-is and document that label readability is a differentiator for a later polishing pass.

### Layout zones (portrait iPhone)

Recommended portrait layout from top to bottom:

```
┌────────────────────────────┐
│  [safe area top padding]   │  ~59pt
│  Annunciators              │  ~18pt
│  LCD display (12-char)     │  ~40pt
│  Stack panel (X/Y/Z/T/LSX) │  ~60pt  (collapsible)
│  ─────────────────────────  │
│  Keyboard skin (SVG)       │  ~560pt (majority of screen)
│  ─────────────────────────  │
│  [safe area bottom padding] │  ~34pt
└────────────────────────────┘
```

The print panel and program listing are secondary panels that should open as sheets or be accessed via a navigation button — not always visible. This is a departure from the desktop layout where panels are always-visible sidebars, but it is the correct mobile pattern.

### Dynamic Type / Accessibility

iOS users expect text to respect their Dynamic Type size setting. For the foundation milestone:
- Calculator LCD display: fixed-size (it simulates hardware; Dynamic Type should NOT apply here)
- Stack panel register values: fixed-size (same rationale)
- Overlay text (function reference, onboarding): SHOULD respect Dynamic Type; use `clamp()` or relative `em` sizing
- Key labels: fixed-size (hardware-faithful)

The minimum font size iOS auto-zooms inputs for is 16px. Any `<input>` used for modal prompts (XEQ-by-name entry, label entry) must use `font-size: 16px` or larger to prevent iOS auto-zoom on focus.

---

## 3. App Lifecycle and State Restoration

### iOS lifecycle states relevant to the emulator

| Lifecycle Event | iOS API | Tauri Event / Hook | Required Action |
|-----------------|---------|-------------------|-----------------|
| App enters background | `applicationDidEnterBackground` | `tauri-plugin-app-events` pause handler | Trigger autosave |
| App will resign active | `applicationWillResignActive` | pause handler (fires first) | Same — ensure save completes before suspension |
| App returns to foreground | `applicationDidBecomeActive` | resume handler | Resume clock tick, stopwatch if running |
| App terminated by OS | No hook (process is killed) | N/A | Autosave on resign-active covers this |

iOS gives ~5 seconds in the background handler before suspension. The existing autosave writes a JSON file asynchronously. This is safe because the current autosave thread releases the Mutex BEFORE disk I/O (existing invariant from CLAUDE.md). The same pattern works on iOS.

### Persistence path change: critical requirement

**The desktop GUI writes to `~/.hp41/autosave.json`. This path does not exist in the iOS app sandbox.** The iOS equivalent is the app's private container, accessible via Tauri's `AppLocalData` base directory or the iOS Documents directory. The persistence layer in `hp41-gui/src-tauri/src/persistence.rs` must be updated to use a platform-aware path:

- Desktop: `~/.hp41/autosave.json` (unchanged)
- iOS: `<AppLocalDataDir>/autosave.json` (iOS app container sandbox)

The `tauri-plugin-store` or the `tauri::api::path::app_local_data_dir()` API provides the correct sandbox path on iOS. **This is not optional** — without this change the app will fail to save/load state on device.

The iOS sandbox path structure is something like:
```
/var/mobile/Containers/Data/Application/<UUID>/Library/Application Support/ch.talent-factory.hp41/
```

Users cannot access this from the Files app directly (it is a private container). For the foundation milestone this is acceptable. Exposing saves to Files.app (via `UIFileSharingEnabled` + `LSSupportsOpeningDocumentsInPlace`) is a differentiator, not a table stake.

### State restoration vs. data persistence

On iOS, the OS may kill the app while it is backgrounded to reclaim memory. When the user returns, iOS re-launches the app. The autosave-on-background-entry approach means the last saved state is at most a few seconds stale. This is the correct pattern for a calculator — the HP-41 hardware had a similar "memory retained as long as batteries are installed" model. No additional UIKit state preservation API is needed beyond autosave-on-resign-active.

### Clock / stopwatch behavior on background

The Time Pac (`TIME_MODULE`) uses `SystemTime::now()` for the clock and a monotonic `Instant` for the stopwatch. On iOS:
- **System clock** (`SystemTime::now()`) continues to advance while backgrounded — correct behavior on return
- **Stopwatch `Instant`:** If the stopwatch was running when the app backgrounded, the elapsed time since backgrounding will be missing from the stopwatch accumulation unless the app records the background entry timestamp and compensates on resume. For the foundation milestone, document this as a known divergence (stopwatch may show incorrect elapsed time after background/resume). This matches the behavior of the original HP-41 stopwatch (it also stops when power is removed).

---

## 4. Desktop-to-iPhone Feature Mapping

### Direct maps (no adaptation needed)

| Feature | Notes |
|---------|-------|
| LCD 12-char display + annunciators | Already React component; safe-area top padding required |
| Clickable key grid → `dispatch_op` | CSS touch-action + `:active` feedback needed |
| 4 skin themes (dark/light/beige/hi-contrast) | CSS custom properties work on WKWebView |
| Function reference / help pool | HelpOverlay.tsx opens as sheet; works on mobile |
| Keystroke programming (`run_stop`, `sst_step`, `bst_step`) | IPC commands unchanged |
| XROM modules (Math/Stat/Time/Advantage) | Core unchanged; XEQ-by-name prompts need modal adaptation |
| `.raw` file import/export | File picker works on iOS via `tauri-plugin-dialog` (needs `NSPhotoLibraryUsageDescription` plist entry if accessing photos; Documents works without) |
| 4-way exhaustive-match invariant | No new Op variants = no change required |

### Need adaptation

| Feature | Adaptation Required | Complexity |
|---------|--------------------|----|
| SHIFT key (one-shot) | Already frontend-only `shiftActive` state, no IPC change; need visible SHIFT indicator and large enough SHIFT key tap target | Low |
| ALPHA mode | Already frontend-only; iOS keyboard auto-show on ALPHA entry must be suppressed (use custom on-screen alpha keys or native keyboard with `inputmode="text"` and dismiss button) | Medium |
| Modal prompts (XEQ, LBL, register number, etc.) | Desktop modals use CSS overlay; on iOS the system keyboard will appear for text input. Must set `font-size: 16px` on inputs to prevent auto-zoom. The `visualViewport` API is needed to reposition the modal above the keyboard. | Medium |
| Print panel | Desktop: always-visible scrollable panel on the right. iPhone: hide by default, open as a bottom sheet or navigation push. | Medium |
| Program listing (PRGM mode) | Desktop: always-visible right panel. iPhone: same as print panel — bottom sheet or full-screen push. | Medium |
| `?` function reference overlay | Currently triggered by `?` key on physical keyboard. iPhone: replace with a tappable `[?]` button in the display area. | Low |
| USER mode key assignment display | Currently shown in the right panel. iPhone: accessible via a dedicated secondary screen. | Low-Medium |
| Stack panel (X/Y/Z/T/LASTX) | Desktop: always-visible side panel. iPhone: show in a horizontal strip above the keyboard; make collapsible to save vertical space. | Low |
| Physical keyboard shortcuts | Continue to work when a hardware Bluetooth keyboard is connected; not required for touch-only use. | Low (no change needed) |
| Onboarding wizard | Multi-step overlay; needs touch-friendly step navigation (large tap targets, swipe or button navigation). | Low |

### Desktop affordances that do NOT translate and must be removed/hidden

| Feature | Action |
|---------|--------|
| Window title bar / resize handle | Not present in mobile Tauri fullscreen mode |
| `p` keyboard shortcut for PRGM mode | Keyboard shortcut remains; touch access must also exist |
| CLI persistence path `~/.hp41/` | Replace with iOS sandbox path (see section 3) |
| Desktop window size (440×1020 fixed) | Replace with responsive CSS that fills safe-area-adjusted viewport |

---

## 5. Table Stakes

Features that MUST be present for a credible iPhone foundation build (TestFlight distribution).

| Feature | Why Required | Complexity | Build-Approach Dependency |
|---------|-------------|------------|--------------------------|
| `hp41-core` compiles to `aarch64-apple-ios` | Without this nothing works | Low (target is straightforward Rust) | Both paths |
| iOS build pipeline in CI / justfile | Reproducible builds | Medium (Xcode project, provisioning) | Tauri: `tauri ios build`; SwiftUI: `xcodebuild` |
| Signed `.ipa` + TestFlight upload | Delivery goal of milestone | Medium (Apple Developer cert, provisioning) | Both paths; cert already owned ($99/yr) |
| App icon (1024×1024 source → all sizes via Xcode) | TestFlight rejects missing icons | Low | Both paths; `cargo tauri icon` (Tauri) or Xcode asset catalog (SwiftUI) |
| Launch screen / splash | TestFlight requirement; missing = rejection | Low | Tauri: `Default.storyboard`; SwiftUI: `LaunchScreen.storyboard` |
| Basic app metadata (bundle ID, display name, version) | Required for TestFlight | Low | `tauri.conf.json` / `Info.plist` |
| Portrait-only lock (`UISupportedInterfaceOrientations`) | Prevents broken landscape layout | Low | Both paths; `Info.plist` entry |
| Safe-area insets applied | Prevents content hidden under Dynamic Island / home indicator | Low-Medium | Tauri: `env(safe-area-inset-*)` CSS + `viewport-fit=cover`; SwiftUI: `.safeAreaInset()` |
| Touch targets ≥44pt on all keys | Usability gate; Apple HIG requirement | Medium (SVG key sizing review + transparent hit-area overlay) | Both paths |
| CSS `:active` feedback + `touch-action: manipulation` | Without this, tap feels broken (350ms delay, no press state) | Low | Tauri path only |
| `-webkit-tap-highlight-color: transparent` | Remove grey flash on key tap | Low | Tauri path only |
| Haptic feedback on key press (`selectionFeedback`) | Expected by iOS calculator users; absence feels broken | Low | Tauri: `tauri-plugin-haptics`; SwiftUI: `UISelectionFeedbackGenerator` |
| iOS sandbox persistence path | Without this, autosave silently fails | Medium (path detection + `persistence.rs` update) | Both paths |
| Autosave on `applicationWillResignActive` | Data loss prevention; table stakes on mobile | Medium (lifecycle hook wiring) | Tauri: `tauri-plugin-app-events` pause handler; SwiftUI: `SceneDelegate.sceneWillResignActive()` |
| Modal prompt inputs at `font-size: 16px` | Prevents iOS auto-zoom on input focus | Low | Tauri path only |
| ALPHA mode touch entry | Without this, ALPHA register entry is impossible touch-only | Medium | Both paths |
| All 44 keys tappable and correctly dispatching ops | Core functionality | Low (IPC unchanged, key map unchanged) | Both paths |
| Stack panel visible (X/Y/Z/T/LASTX) | Users cannot operate calculator without seeing the stack | Low | Both paths |
| SHIFT one-shot indicator visible | Users cannot know shift state without it | Low | Both paths |

---

## 6. Differentiators

Features that improve the iPhone experience but are not blockers for a TestFlight foundation build.

| Feature | Value | Complexity | Notes |
|---------|-------|------------|-------|
| Per-key haptic intensity variation (`impactFeedback('medium')` for ENTER, `'light'` for digits) | More authentic feel | Low | |
| Error haptic (`notificationFeedback('error')`) on DATA ERROR / NO ROOM | Communicates errors without eyes on screen | Low | |
| Print panel as native iOS bottom sheet with momentum scroll | Native-feeling UX | Medium | |
| Program listing as full-screen push navigation | Native iOS pattern | Medium | |
| `font-size` adaptive key labels (larger labels on Plus/Max models) | Legibility improvement | Low | |
| Visible SHIFT / ALPHA mode buttons styled to match real HP-41 shift key color (orange/blue) | Authentic + immediately recognizable | Low | Already done on desktop |
| `?` function reference button always visible in display area | Quick access without hardware keyboard | Low | |
| Stack panel collapsible with animation | Saves vertical space for users who know the stack | Low-Medium | |
| PRGM listing animated slide-up panel | Native iOS feel | Medium | |
| Disable rubber-band scroll on the main calculator view (`overscroll-behavior: none`) | Prevents jarring bounce on the calculator body | Low | Tauri path |
| iOS Files.app access (`UIFileSharingEnabled` + `LSSupportsOpeningDocumentsInPlace`) | Allows `.raw` file exchange via Files.app | Low-Medium | |
| Localization / VoiceOver accessibility labels on keys | Accessibility | Medium | |
| Restore system clock offset on resume (compensate stopwatch for background time) | Faithful Time Pac behavior | Low-Medium | |

---

## 7. Anti-Features / Explicitly Deferred

These items are out of scope for v4.1 iOS Foundation. They must not be started in this milestone.

| Item | Why Deferred | When |
|------|-------------|------|
| App Store submission | Out of scope per PROJECT.md; requires store assets, screenshots, review readiness | v4.2 or later |
| App Store review navigation ("HP-41" trademark) | Legal research required separately | v4.2 |
| iPad layout | Different safe areas, different key sizing, split-screen; separate design work | v4.2+ |
| Landscape orientation | Requires entirely different key layout; portrait-only locked for v4.1 | v4.2+ |
| Android support | Separate platform, different build pipeline, different signing | Post-v4.x |
| iCloud sync of `autosave.json` | Privacy/infrastructure concern; app is local-only by design | Post-v4.x |
| Apple Pencil input | iPad-only; not relevant to iPhone | v4.2+ |
| Widgets (WidgetKit) | Complex, iOS-specific extension; not relevant to foundation | Post-v4.x |
| Shortcuts / Siri integration | Complex, low priority | Post-v4.x |
| App Store screenshots / marketing assets | Required only for App Store, not TestFlight | v4.2 |
| New calculator math functions | Core is frozen and feature-complete | Never (no new ops planned) |
| Custom keyboard extension | Not relevant for a self-contained app | Post-v4.x |
| Apple Watch companion | Requires separate target; very small screen for HP-41 | Indefinitely deferred |
| Binary releases of desktop app via cargo-dist | Separate from iOS; defer per existing plan | Post-v4.x |
| Interrupting control alarm execution | Already deferred at v3.2 level; iOS doesn't change this | Post-v4.x |

---

## 8. Competitor Feature Reference

Existing HP-41 iOS emulators confirm the above feature set is correct for the domain:

**i41CX+ (App Store, active as of 2025):**
- 43 keyboard overlay skins with dynamic switching
- Haptic feedback per key press
- Virtual thermal printer/plotter panel
- Full-screen text editor (ALPHA mode)
- iCloud support (differentiator; not a table stake for foundation)
- 43 overlays suggests they solve touch targets via transparent overlay buttons over skin artwork, not by resizing the SVG

**my41CX / MY41CX (App Store, active as of 2025):**
- Microcode-accurate HP-41CX emulator (vs. behavioral emulation here)
- Clean portrait layout with HP-41 aesthetic
- Available on both iOS and macOS
- Supports `.MOD` files (HP-41 module format)

**Free42 (App Store, Thomas Okken):**
- Open source HP-42S emulator; actively maintained
- Haptic feedback for key presses (user-requested for years, eventually added)
- Portrait layout; skin-based UI
- Confirms haptic feedback is a "must-have" expectation for HP calculator emulators on iPhone

---

## 9. Build-Approach Dependencies Summary

Several table-stakes items have different implementation paths depending on the ADR outcome (Tauri v2 Mobile vs. SwiftUI + Rust FFI). The roadmapper should note:

| Concern | Tauri v2 Mobile | SwiftUI + Rust FFI |
|---------|----------------|-------------------|
| UI reuse from desktop | HIGH — existing React components reused directly | LOW — UI must be rewritten in SwiftUI |
| Safe area CSS | `env(safe-area-inset-*)` CSS variables | SwiftUI `.safeAreaInset()` modifier |
| Haptics | `tauri-plugin-haptics` npm package | `UIFeedbackGenerator` directly in Swift |
| Lifecycle hooks | `tauri-plugin-app-events` Rust plugin | `SceneDelegate` / `AppDelegate` in Swift |
| Persistence path | Tauri `app_local_data_dir()` API | `FileManager.default.urls(.applicationSupportDirectory)` |
| Touch feedback (`:active`) | CSS `:active` + `touchstart` workaround required | Native SwiftUI `.buttonStyle` with `isPressed` |
| Modal prompt UX | HTML `<input>` with `visualViewport` keyboard detection | SwiftUI `.sheet` + `TextField` |
| WKWebView gotchas | Must mitigate all items in section 1 | N/A (no WebView) |
| Phase count impact | Fewer phases (existing React UI) | More phases (UI rewrite) |
| Risk | Medium — Tauri mobile is "not first-class citizen" per docs | Medium — Rust FFI setup complexity |

The research files STACK.md and ARCHITECTURE.md should contain the full build-approach ADR analysis. This FEATURES.md treats both paths as valid and notes differences per feature.

---

## Sources

- [Apple Human Interface Guidelines — Touch Targets](https://developer.apple.com/design/human-interface-guidelines/buttons) — HIGH confidence
- [Tauri v2 Haptics Plugin](https://v2.tauri.app/plugin/haptics/) — HIGH confidence
- [Tauri v2 App Store Distribution](https://v2.tauri.app/distribute/app-store/) — HIGH confidence
- [Tauri v2 iOS Code Signing](https://v2.tauri.app/distribute/sign/ios/) — HIGH confidence
- [Tauri v2 Prerequisites (iOS)](https://v2.tauri.app/start/prerequisites/) — HIGH confidence
- [WKWebView Gotchas for Tauri iOS — zudo-tauri-wisdom](https://takazudomodular.com/pj/zudo-tauri/docs/mobile/wkwebview-gotchas/) — HIGH confidence (primary source for WKWebView behavior)
- [tauri-plugin-app-events (lifecycle hooks)](https://github.com/wtto00/tauri-plugin-app-events) — MEDIUM confidence (third-party plugin, not official Tauri)
- [Safe Area Insets in SwiftUI](https://www.swiftuifieldguide.com/layout/safe-area/) — HIGH confidence
- [iOS App Lifecycle — UIApplicationDelegate](https://developer.apple.com/documentation/uikit/uiapplicationdelegate/applicationwillresignactive(_:)) — HIGH confidence
- [More Responsive Tapping on iOS — WebKit Blog](https://webkit.org/blog/5610/more-responsive-tapping-on-ios/) — HIGH confidence (official WebKit)
- [i41CX+ App Store listing](https://apps.apple.com/us/app/i41cx/id289068865) — MEDIUM confidence (feature list from App Store description)
- [my41CX App Store listing](https://apps.apple.com/us/app/my41cx/id979041950) — MEDIUM confidence
- [Free42 App Store listing](https://apps.apple.com/us/app/free42/id337692629) — HIGH confidence (long-running reference project)
- [Deque — iOS Touch Target Size (WCAG 2.5.5)](https://docs.deque.com/devtools-mobile/2025.7.2/en/ios-touch-target-size/) — HIGH confidence
- [safe-area-inset-bottom keyboard bug](https://webventures.rejh.nl/blog/2025/safe-area-inset-bottom-does-not-update/) — MEDIUM confidence (community blog, but well-documented bug)
- [iOS App Icon Sizes 2026 guide](https://appilot.ai/blog/ios-app-icon-sizes-2026) — MEDIUM confidence (verified: Xcode 15+ generates all sizes from 1024×1024 source)
- [Tauri mobile for iOS — DEV.to](https://dev.to/adimac93/tauri-mobile-for-ios-4dp6) — MEDIUM confidence (community walkthrough)
