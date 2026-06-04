# Phase 55: Touch UI Adaptation - Research

**Researched:** 2026-06-03
**Domain:** iOS touch adaptation — Tauri v2 Mobile / WKWebView / React / CSS
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-55.1:** Gate all iOS-only behaviors behind a new `is_ios` Tauri command (mirrors `is_macos`). Compile-time `cfg(target_os = "ios")`. Requires `permissions/is-ios.toml` + `capabilities/default.json` entry. Run `cargo check` first to generate permission registry. Frontend `isIos` state gates bottom sheets, collapsible stack, ALPHA bar, and touch text entry.
- **D-55.2:** Touch text entry must cover BOTH (a) ALPHA-register mode (`annunciators.alpha === true`) AND (b) modal name/label prompts (`modal_requires_alpha_label === true`). The `AlphaTouchInput` component's trigger condition expands from `annunciators.alpha` to also fire when `modal_requires_alpha_label === true`, routing characters through the existing modal-label dispatch path.
- **D-55.3:** On-screen ALPHA character grid deferred to v4.2. Native iOS keyboard `<input type="text">` is the chosen mechanism.
- **D-55.4:** Verify with inline human checkpoints (Phase 53/54 pattern). Criteria observable only on real hardware require explicit checkpoint tasks.

### Claude's Discretion

- **D-55.5:** No decomposition preference — `gsd-planner` chooses plan boundaries and wave parallelization.
- Exact permission-file naming, `isIos` state plumbing shape, and how `AlphaTouchInput` trigger is refactored to cover both ALPHA-register and modal-label cases are left to planning, provided they honor the UI-SPEC's visual contract and the iOS-gating / no-desktop-regression rule.

### Deferred Ideas (OUT OF SCOPE)

- On-screen ALPHA character grid — v4.2
- Annunciator bump to 13px — non-blocking follow-up
- Bottom-sheet swipe gesture — v4.2
- Landscape orientation, iPad, Android — v4.2+
- App lifecycle background-throttling + clock-on-resume — Phase 56
- Signing / PrivacyInfo.xcprivacy / app icon / ci-ios.yml / TestFlight — Phase 57
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TOUCH-01 | All 44 keys tappable with hit targets ≥ 44×44 pt | `.key-touch-target` overlay architecture; SVG key geometry analysis; keyPosition() function |
| TOUCH-02 | Portrait-locked layout respects safe-area insets | `viewport-fit=cover` in index.html; CSS `env(safe-area-inset-*)`; project.yml portrait-lock edit |
| TOUCH-03 | Immediate press feedback, 350ms delay eliminated, no tap flash | `touch-action: manipulation`; `pointerdown`/`pointerup` trigger; `-webkit-tap-highlight-color: transparent` |
| TOUCH-04 | ALPHA-mode character entry by touch, stays visible above keyboard | `AlphaTouchInput` component; `visualViewport` resize listener; 16px input font-size constraint |
| TOUCH-05 | Haptic feedback on every key tap | `@tauri-apps/plugin-haptics` 2.3.2; `impactFeedback('light'/'medium'/'heavy')` |
| TOUCH-06 | Audio resumes correctly after first user gesture | `ensureAudioResumed` pattern; `audioCtx.resume()` inside `pointerdown` handler |
| TOUCH-07 | Annunciators + stack panel visible and legible on iPhone | Existing 11px / 0.35 opacity — no change needed; collapsible stack reduces vertical crowding |
| TOUCH-08 | Haptic intensity varies by key type; distinct haptic on DATA ERROR / NO ROOM | Three tier classification (Light/Medium/Heavy) + `notificationFeedback('error')`; error detection post-IPC |
| TOUCH-09 | Pull-up bottom sheets for print and PRGM listing | CSS `.bottom-sheet` + `.expanded` pattern; iOS-gated via `isIos` |
| TOUCH-10 | Stack panel collapsible | `.stack-panel-collapsible` + `.collapsed` CSS; chevron toggle; local React state |
| TOUCH-11 | Overscroll suppressed on calculator body | Already partially done in index.css; bottom-sheet inner scroll uses `overscroll-behavior-y: contain` |
</phase_requirements>

---

## Summary

Phase 55 is a pure front-end adaptation phase — every change lives in `hp41-gui` (React + CSS) plus one new Tauri command (`is_ios`) and one new Tauri plugin (`tauri-plugin-haptics`). The `hp41-core` engine is untouched. The design is fully locked by 55-UI-SPEC.md.

The research confirmed seven areas of technical correctness that are critical for the planner. The most significant finding is an **API signature discrepancy**: the UI-SPEC specifies `impactFeedback({ style: 'Light' })` with an object argument and PascalCase values, but the actual `@tauri-apps/plugin-haptics` 2.3.2 API (verified from installed type definitions) is `impactFeedback('light')` — a bare string with lowercase values. The planner must use the correct API, not the UI-SPEC pseudo-code. Every other design decision in the UI-SPEC is technically correct.

A second important finding: the haptics plugin **requires** `#[cfg(mobile)]` gating in `lib.rs` (unconditional registration compiles on desktop but the plugin will not link on desktop builds without the gate). The Cargo dependency should be added with a target specifier.

The `visualViewport` resize listener approach for keyboard tracking is correct and verified, but Tauri has a known open issue (#10631) where `visualViewport.height` may not correctly account for the keyboard on some configurations — the fallback `window.innerHeight - vv.height - vv.offsetTop` calculation is the correct workaround.

**Primary recommendation:** Implement in four natural waves: (1) hit-targets + safe-area + tap-feedback + portrait-lock, (2) haptics plugin + audio unlock, (3) ALPHA/modal text entry, (4) bottom sheets + collapsible stack + overscroll refinement. Each wave is independently deployable and iOS-gated.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Hit-target overlays (.key-touch-target) | Frontend / React CSS | — | Pure client-side presentation; no IPC change |
| Safe-area insets | Frontend / CSS | iOS system | `env()` CSS variables resolved by WKWebView |
| Portrait lock | iOS native (project.yml) | — | `UISupportedInterfaceOrientations` is an OS-level plist key |
| Tap-delay elimination | Frontend / CSS | — | `touch-action: manipulation` on CSS class |
| Press feedback (pointerdown/up) | Frontend / React | — | Existing `.key-pressed` CSS; only trigger event changes |
| Haptics | Frontend / JS | Rust plugin (mobile only) | JS calls Tauri IPC to native `UIFeedbackGenerator` |
| Audio resume | Frontend / JS | — | `AudioContext.resume()` inside user gesture handler |
| AlphaTouchInput bar | Frontend / React | Existing Tauri IPC | New React component; dispatches via existing `dispatch_op` |
| ALPHA/modal label dispatch | Frontend / React | Existing Tauri IPC | Reuses `alpha_<X>` routing + `submit_modal_with_label` |
| Bottom sheets (print/PRGM) | Frontend / React | — | iOS-gated CSS + React state; reads existing `CalcStateView` fields |
| Collapsible stack panel | Frontend / React | — | Local React state; reads `x_str`/`y_str`/`z_str`/`t_str`/`lastx_str` |
| is_ios detection | Rust backend | Frontend consumer | `cfg(target_os = "ios")` — same pattern as `is_macos` |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `@tauri-apps/plugin-haptics` | 2.3.2 | iOS/Android haptic feedback | Official Tauri plugin from tauri-apps/plugins-workspace [VERIFIED: npm registry + official docs] |
| `tauri-plugin-haptics` | 2.3.2 | Rust backend for haptics | Same package, Rust crate [VERIFIED: crates.io] |
| `touch-action: manipulation` | CSS | Tap-delay elimination | W3C Pointer Events standard; supported iOS 13+ [CITED: webkit.org/blog/5610] |
| `env(safe-area-inset-*)` | CSS | Safe-area insets | W3C standard; WKWebView iOS 11+ with `viewport-fit=cover` [CITED: developer.apple.com] |
| `window.visualViewport` | Web API | Keyboard height tracking | Supported iOS 13+; WKWebView [CITED: MDN Web Docs] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `AudioContext` (Web Audio API) | WKWebView built-in | TONE/BEEP audio | Already present; needs resume guard on iOS |
| `ResizeObserver` | Already in main.tsx | Content size tracking | Already present; safe-area changes flow through it for free |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `tauri-plugin-haptics` | `navigator.vibrate()` | `vibrate()` is Web API, not iOS haptics (iPhone has no simple vibrate API); plugin is the only iOS haptic path |
| `visualViewport` | `window.innerHeight` | `window.innerHeight` does NOT shrink on iOS keyboard; `visualViewport` is the only correct approach |
| `pointerdown` dispatch | `touchstart` dispatch | `pointerdown` unifies mouse and touch; touchstart + onClick = double-fire risk |

**Installation:**

```bash
# In hp41-gui/ directory
npm install @tauri-apps/plugin-haptics@2.3.2

# Cargo.toml — add with mobile target gate (REQUIRED — do not add unconditionally):
# [target.'cfg(any(target_os = "android", target_os = "ios"))'.dependencies]
# tauri-plugin-haptics = "2.3.2"
```

---

## Package Legitimacy Audit

> slopcheck was not installable in this environment. All packages are tagged [ASSUMED] per graceful-degradation policy. However, `@tauri-apps/plugin-haptics` is the official Tauri plugin from the `tauri-apps/plugins-workspace` monorepo — the same monorepo that ships Tauri itself — which is strong evidence of legitimacy beyond registry existence.

| Package | Registry | Age | Downloads | Source Repo | slopcheck | Disposition |
|---------|----------|-----|-----------|-------------|-----------|-------------|
| `@tauri-apps/plugin-haptics` | npm | ~18 months (v2.0.0-rc.2 first) | 27,977 total (crates.io) | github.com/tauri-apps/plugins-workspace | [ASSUMED] | Approved — official Tauri org package |
| `tauri-plugin-haptics` | crates.io | ~18 months | 27,977 | Same repo | [ASSUMED] | Approved — official Tauri org crate |

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none — both packages are from the official `tauri-apps` organization.

*slopcheck was unavailable at research time; packages above are tagged [ASSUMED]. The planner must add a `checkpoint:human-verify` task before the npm/cargo install step. Given the official Tauri org authorship this is a low-risk gate.*

---

## Architecture Patterns

### System Architecture Diagram

```
iPhone Touch Input
  │
  ▼
.key-touch-target overlay (position: absolute, 44×44pt, pointer-events: auto)
  │
  ├── onPointerDown ──► ensureAudioResumed(audioCtx)  ─► AudioContext.resume() [once]
  │                 ──► impactFeedback('light'|'medium'|'heavy')  [iOS only, #cfg(mobile)]
  │                 ──► setPressedKey(key.id)  ─► .key-pressed CSS (scale 0.92)
  │
  └── onPointerUp / onClick ──► handleClick(key: KeyDef)
                                  │
                                  ├── SHIFT toggle (setShiftActive)
                                  ├── ALPHA-mode: effectiveId = alpha_<char>
                                  ├── Modal-open route (MODAL_OPENERS)
                                  └── invokeForKey(effectiveId, calcState)
                                        │
                                        ▼
                                      Tauri IPC (dispatch_op / sst_step / etc.)
                                        │
                                        ▼
                                      CalcStateView (Rust → JSON → React)
                                        │
                                        ├── Error check: display_str contains 'DATA ERROR'/'NO ROOM'?
                                        │     └── notificationFeedback('error')  [iOS only]
                                        │
                                        └── setCalcState(view) → React re-render

AlphaTouchInput bar (fixed, above iOS keyboard)
  │  trigger: isIos && (annunciators.alpha || modal_requires_alpha_label)
  │
  ├── visualViewport resize ──► set bottom position dynamically
  ├── <input type="text"> (16px, controlled) ──► input event ──► alpha_<char> dispatch
  ├── Backspace ──► clx dispatch
  ├── "Done" button ──► alpha_toggle dispatch (ALPHA mode)
  └── "Done" button ──► submit_modal (modal label mode — D-55.2)

Bottom Sheets (iOS-gated via isIos)
  │
  ├── Print Sheet: visible when print_lines.length > 0
  └── PRGM Sheet: visible when annunciators.prgm === true
        toggle: tap drag handle → .bottom-sheet.expanded class

Collapsible Stack Panel (iOS-gated)
  │
  └── chevron button → local stackExpanded state → max-height transition
```

### Recommended Project Structure

No new directories. All new files live within the existing `hp41-gui/src/` and `hp41-gui/src-tauri/` structure:

```
hp41-gui/
├── index.html                          # ADD: viewport-fit=cover to meta viewport
├── src/
│   ├── App.tsx                         # ADD: isIos state, AlphaTouchInput trigger, error haptic
│   ├── App.css                         # ADD: 10 new CSS classes per UI-SPEC
│   ├── AlphaTouchInput.tsx             # NEW: ALPHA + modal text entry component
│   ├── BottomSheet.tsx                 # NEW: print + PRGM pull-up sheet component
│   └── Keyboard.tsx                    # MODIFY: add .key-touch-target overlay elements
└── src-tauri/
    ├── src/
    │   ├── commands.rs                  # ADD: is_ios() command (clone of is_macos)
    │   └── lib.rs                       # ADD: #[cfg(mobile)] haptics plugin registration
    ├── Cargo.toml                       # ADD: tauri-plugin-haptics (mobile target gate)
    ├── capabilities/
    │   └── default.json                 # ADD: is_ios + haptics permissions
    └── permissions/
        └── is-ios.toml                  # NEW: permission for is_ios command
```

Additionally: `gen/apple/project.yml` — remove landscape orientation entries.

### Pattern 1: is_ios Tauri Command (Clone of is_macos)

**What:** Compile-time platform detection via `cfg(target_os = "ios")` returned as a Tauri command.
**When to use:** Any front-end code that must behave differently on iOS vs desktop.

```rust
// commands.rs — clone of the existing is_macos pattern (line 544)
/// Tauri command: report whether the backend was compiled for iOS.
///
/// The frontend uses this to gate touch-specific behaviors (bottom sheets,
/// AlphaTouchInput bar, .key-touch-target overlays). Authoritative via
/// compile-time cfg — no JS platform sniffing.
#[tauri::command]
pub fn is_ios() -> bool {
    cfg!(target_os = "ios")
}
```

```toml
# permissions/is-ios.toml (exact clone of permissions/is-macos.toml)
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-is-ios"
description = "Allows the is_ios command."
commands.allow = ["is_ios"]
```

```json
// capabilities/default.json — add alongside "allow-is-macos"
"allow-is-ios"
```

**Ordering discipline:** Run `cargo check` from `hp41-gui/src-tauri/` BEFORE editing `default.json` to regenerate the permission schema. [CITED: CLAUDE.md Tauri v2.11 permission flow]

### Pattern 2: Haptics Plugin Registration (mobile-gated)

**CRITICAL:** The Tauri haptics plugin must be registered with `#[cfg(mobile)]`. Unconditional registration fails the desktop compile because the plugin's native iOS symbol `UIImpactFeedbackGenerator` does not exist on desktop targets.

```toml
# hp41-gui/src-tauri/Cargo.toml — add with target specifier
[target.'cfg(any(target_os = "android", target_os = "ios"))'.dependencies]
tauri-plugin-haptics = "2.3.2"
```

```rust
// lib.rs — add inside the .setup() closure, gated on #[cfg(mobile)]
#[cfg(mobile)]
app.handle().plugin(tauri_plugin_haptics::init())?;
```

```json
// capabilities/default.json — add all 4 haptics permissions
"haptics:allow-impact-feedback",
"haptics:allow-notification-feedback",
"haptics:allow-selection-feedback",
"haptics:allow-vibrate"
```

[VERIFIED: npm registry — installed from @tauri-apps/plugin-haptics@2.3.2 dist-js/bindings.d.ts]
[CITED: v2.tauri.app/plugin/haptics/]

### Pattern 3: Correct Haptics API (FIXES UI-SPEC ERROR)

**CRITICAL DISCREPANCY:** The UI-SPEC specifies `impactFeedback({ style: 'Light' })` with an object argument and PascalCase values. The **actual API** (verified from the installed package's TypeScript types) is:

```typescript
// WRONG (UI-SPEC pseudo-code):
impactFeedback({ style: 'Light' })        // ← incorrect
notificationFeedback({ type: 'Error' })   // ← incorrect

// CORRECT (verified from bindings.d.ts):
import { impactFeedback, notificationFeedback } from '@tauri-apps/plugin-haptics';

// Impact tiers (exact lowercase string literals):
await impactFeedback('light');    // ImpactFeedbackStyle: 'light' | 'medium' | 'heavy' | 'soft' | 'rigid'
await impactFeedback('medium');
await impactFeedback('heavy');

// Error haptic (exact lowercase string literal):
await notificationFeedback('error');  // NotificationFeedbackType: 'success' | 'warning' | 'error'
```

The type is: `type ImpactFeedbackStyle = 'light' | 'medium' | 'heavy' | 'soft' | 'rigid'`
The type is: `type NotificationFeedbackType = 'success' | 'warning' | 'error'`

[VERIFIED: npm registry — inspected /hp41-gui/node_modules/@tauri-apps/plugin-haptics/dist-js/bindings.d.ts]

**Desktop behavior:** On desktop builds, `#[cfg(mobile)]` excludes the plugin entirely. The JS `impactFeedback()` call will reject with a Tauri "command not found" error on desktop. Guard all haptic calls with the `isIos` flag:

```typescript
// Correct iOS-gated haptic call pattern:
if (isIos) {
  await impactFeedback(hapticTier).catch(() => {/* silent — desktop or unsupported device */});
}
```

### Pattern 4: Audio Resume (ensureAudioResumed)

**What:** iOS WKWebView starts `AudioContext` in `'suspended'` state. `resume()` must be called synchronously inside a user gesture event handler. [CITED: developer.apple.com/forums/thread/658375]

```typescript
// Source: UI-SPEC Audio Contract (technically correct)
const audioResumedRef = useRef(false);

async function ensureAudioResumed(audioCtx: AudioContext): Promise<void> {
  if (audioResumedRef.current) return;
  if (audioCtx.state === 'suspended') {
    await audioCtx.resume();
  }
  audioResumedRef.current = true;
}

// Called at top of pointerdown handler, before IPC:
const handlePointerDown = async (key: KeyDef) => {
  if (audioCtx && isIos) await ensureAudioResumed(audioCtx);
  // ... haptic trigger ...
  // ... press visual ...
};
```

**webkitAudioContext:** As of iOS 14+, `AudioContext` (unprefixed) is supported in WKWebView. `webkitAudioContext` is NOT needed for iOS 14.0+ (the project's deployment target). [ASSUMED — not independently verified for WKWebView specifically; evidence from MDN and browser compat tables]

**Silent switch:** `AudioContext` audio is muted by the iOS hardware silent switch. This is correct HP-41-faithful behavior. Document in Phase 57 release notes, not in-app UI (per UI-SPEC). [CITED: 55-UI-SPEC.md Audio Contract]

### Pattern 5: visualViewport Keyboard Tracking

**What:** WKWebView does NOT resize `window.innerHeight` when the iOS keyboard appears. `window.visualViewport.height` does shrink (in most Tauri versions). [CITED: WKWebView iOS community documentation]

**Known risk:** Tauri GitHub issue #10631 documents that `visualViewport.height` may not account for keyboard height correctly in some Tauri mobile configurations. The recommended pattern uses `window.innerHeight - vv.height - vv.offsetTop` as the keyboard inset, which is more robust than relying on `vv.height` alone.

```typescript
// AlphaTouchInput component — keyboard tracking
useEffect(() => {
  const vv = window.visualViewport;
  if (!vv) return;
  
  const updatePosition = () => {
    // Keyboard inset = difference between window height and visible viewport
    const keyboardHeight = window.innerHeight - vv.height - vv.offsetTop;
    setBottomOffset(Math.max(keyboardHeight, 0));
  };
  
  vv.addEventListener('resize', updatePosition);
  vv.addEventListener('scroll', updatePosition);
  return () => {
    vv.removeEventListener('resize', updatePosition);
    vv.removeEventListener('scroll', updatePosition);
  };
}, []);
```

CSS for positioning:

```css
.alpha-touch-input-bar {
  position: fixed;
  bottom: calc(var(--keyboard-inset, 0px) + env(safe-area-inset-bottom, 0px));
  /* or: bottom set inline via style={} from the JS state */
}
```

**16px font-size is non-negotiable:** The `<input>` inside `AlphaTouchInput` must be `font-size: 16px` or larger. iOS auto-zooms the viewport when a focused input has a font-size below 16px, fighting `computeScale`. [CITED: multiple WebKit / Apple HIG sources — long-standing rule]

### Pattern 6: Hit-Target Overlay Architecture

**What:** The SVG keyboard renders key caps at KEY_W=68, KEY_H=44 design units. At iPhone SE scale (×0.957), the caps render at ~65×42pt. The hit-area must extend to 44×44pt minimum.

The `keyPosition()` function in Keyboard.tsx already computes pixel positions for each key in SVG space. The overlay approach places absolutely positioned `<div>` elements over the SVG, one per key, sized to 44pt minimum.

**Architecture consideration:** The SVG is inside a `transform: scale(N)` wrapper (ScaledApp). Overlay `div` elements must be positioned relative to the calculator's coordinate system AFTER scaling, not in SVG design units. Two approaches:

**Option A — Overlay `div` inside the SVG's React parent `div`:** Place overlays as siblings of the `<svg>` element, both inside a `position: relative` container. Convert key positions from SVG design units to CSS pixels using the scale factor.

**Option B — Add invisible React `<foreignObject>` elements inside the SVG at each key position:** Each `<foreignObject>` contains a `<div>` with `touch-action: manipulation` and `onPointerDown`/`onPointerUp` handlers.

**Recommendation:** Option A (overlay divs outside SVG). `<foreignObject>` inside SVG has historied cross-browser rendering quirks in WKWebView. The `position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%)` pattern from the UI-SPEC positions overlays centered on each key's center point.

**Key center coordinates** (from `keyPosition()`):
- `cx = x + w/2` (design units)
- `cy = y + h/2` (design units)
- In CSS pixels: `cx_px = cx * scale`, `cy_px = cy * scale`

The `Keyboard` component will need to accept `scale` as a prop OR the overlay `div` container will use CSS to match the SVG's rendered size.

**Simpler approach:** Use `position: absolute; left: <cx_pct>%; top: <cy_pct>%` as percentages of the SVG's rendered size (which is `width: 100%` in its container). This avoids needing the scale factor.

```css
.key-touch-target {
  position: absolute;
  min-width: 44px;
  min-height: 44px;
  touch-action: manipulation;
  -webkit-tap-highlight-color: transparent;
  cursor: pointer;
  transform: translate(-50%, -50%);
  /* top and left set per key via inline style as percentage of SVG container */
}
```

### Pattern 7: pointerdown/pointerup vs onClick Event Model

**What:** The existing `Keyboard.tsx` uses `onClick` on SVG `<g>` elements for key dispatch. Phase 55 adds `onPointerDown` for the haptic trigger + press visual, and keeps the dispatch in `onClick`/`onPointerUp`.

**Double-fire risk with onClick:** Using BOTH `onTouchStart` and `onClick` on the same element causes double-fire (touchStart fires, then the synthetic click fires ~350ms later). With `touch-action: manipulation`, the click fires without the delay but still fires after pointerup. The pattern: use `onPointerDown` for immediate feedback (haptic + press visual) and `onClick` or `onPointerUp` for dispatch (fires once). This is the correct split. [CITED: developer.chrome.com/blog/300ms-tap-delay-gone-away + SitePoint]

**On the overlay `div`:** The `.key-touch-target` div has `onPointerDown` and `onClick`. The underlying SVG `onClick` handler should be disabled (remove the SVG `onClick` on iOS, or keep it and deduplicate via `busyRef`). The cleanest approach: move key dispatch from SVG `onClick` to overlay `div` `onPointerUp`, with `busyRef.current` as the deduplication guard (already in place).

### Anti-Patterns to Avoid

- **Using `impactFeedback({ style: 'Light' })` (object form):** Incorrect API; will cause a runtime type error. Use `impactFeedback('light')`.
- **Registering `tauri_plugin_haptics::init()` unconditionally:** Fails desktop compile. Must be `#[cfg(mobile)]`.
- **Adding haptics as a regular `[dependencies]` entry in Cargo.toml:** Compiles but includes dead code on desktop. Use `[target.'cfg(any(target_os = "android", target_os = "ios"))'.dependencies]`.
- **Relying on `window.innerHeight` for keyboard height:** Incorrect on iOS WKWebView; use `visualViewport`.
- **Input font-size below 16px in AlphaTouchInput:** Triggers iOS viewport auto-zoom.
- **Adding `.key-touch-target` divs on desktop (non-iOS):** Must be iOS-gated via `isIos` to avoid changing desktop click behavior.
- **Calling `audioCtx.resume()` outside a user gesture handler:** iOS silently refuses the resume; no error, just silence.
- **Moving SVG `.key` CSS from App.css to themes.css:** P51 frozen invariant — `transform-box: fill-box` must remain in App.css.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Per-key haptic feedback | Custom `navigator.vibrate()` wrapper | `@tauri-apps/plugin-haptics` | iPhone has no standard vibrate API; plugin calls `UIImpactFeedbackGenerator` via native iOS SDK |
| iOS platform detection | `window.__TAURI_INTERNALS__?.platform` sniff | `is_ios` Tauri command | D-55.1 explicitly rejected the JS sniff in favor of compile-time `cfg()` |
| 300ms delay elimination | FastClick.js or custom touch handler | `touch-action: manipulation` CSS property | FastClick is deprecated; CSS property is the W3C standard approach |
| Safe-area insets | Reading UIKit safe area values via Tauri plugin | `env(safe-area-inset-*)` CSS | Browser-level CSS standard; WKWebView exposes it correctly with `viewport-fit=cover` |

---

## D-55.2 Modal-Label Touch Entry: Trace

The planner needs to understand exactly which App.tsx state and functions AlphaTouchInput must integrate with.

### Existing Desktop Path for Modal Label Entry

1. **Trigger:** User presses XEQ (opens `xeq_name` modal), or GTO/LBL/CLP/ASN-label prompts open a `pendingInput` with `kind === 'xeq_name' | 'clp' | 'assign_label'`.
2. **Detection:** `modal_requires_alpha_label: boolean` field in `CalcStateView` (App.tsx line ~60) — `true` when the Rust backend has an active "FUNCTION NAME?" prompt step.
3. **Desktop character routing** (App.tsx lines ~630-660): In `handleClick`, when `pendingInput !== null` AND `pendingInput.kind === 'xeq_name' | 'clp' | 'assign_label'` AND the key has `alphaChar`, the `routedKey` becomes `key.alphaChar` — routed through `handleModalKey`.
4. **Physical keyboard path** (App.tsx lines ~149-156): `resolveKeyId` — when `annunciators.alpha === true` AND key.length === 1, routes to `alpha_<X>`.
5. **Submit:** When Enter is pressed in a modal, `handleModalKey` returns a `dispatchId` of `SUBMIT_MODAL_WITH_LABEL_PREFIX + label` → `invokeForKey` routes to `invoke('submit_modal_with_label', { label })`.

### AlphaTouchInput Integration Points (D-55.2)

The `AlphaTouchInput` component must:

1. **Trigger condition:** `isIos && (calcState.annunciators.alpha || calcState.modal_requires_alpha_label)` — covers both ALPHA-register mode AND modal-label prompts.

2. **Header label:** Show "ALPHA REGISTER" when `annunciators.alpha`, show modal prompt string (`calcState.modal_prompt`) when `modal_requires_alpha_label`.

3. **Character routing:**
   - When `annunciators.alpha`: dispatch `alpha_<CHAR>` via `invoke('dispatch_op', { keyId: 'alpha_' + ch })` for each char diff.
   - When `modal_requires_alpha_label`: build the label string and dispatch `SUBMIT_MODAL_WITH_LABEL_PREFIX + currentLabel` OR route individual chars through the existing `pendingInput` machinery. **Simpler approach:** accumulate the string in local state, and on "Done" dispatch `submit_modal_with_label({ label: accumulatedString })` directly — bypassing `pendingInput` state in App.tsx entirely for the touch path. This avoids duplicating the `pendingInput` state machine in the mobile component.

4. **"Done" button behavior:**
   - When `annunciators.alpha`: dispatch `alpha_toggle` to exit ALPHA mode.
   - When `modal_requires_alpha_label`: dispatch `submit_modal_with_label({ label: currentInputValue })`.

5. **Backspace:** `clx` op (per UI-SPEC) when `annunciators.alpha`. When `modal_requires_alpha_label`, update local string state (remove last char) without dispatching.

6. **State to expose via App.tsx:** `isIos` (already planned per D-55.1), `calcState.annunciators.alpha`, `calcState.modal_requires_alpha_label`, `calcState.modal_prompt`, and access to `dispatchKeyId` / `invokeForKey`.

**Recommended implementation structure:**

```tsx
// AlphaTouchInput.tsx
interface AlphaTouchInputProps {
  isAlphaMode: boolean;           // calcState.annunciators.alpha
  isModalLabelMode: boolean;      // calcState.modal_requires_alpha_label
  modalPrompt: string | null;     // calcState.modal_prompt
  onDispatch: (keyId: string) => void;   // App.tsx dispatchKeyId
  onSubmitLabel: (label: string) => void; // invoke('submit_modal_with_label')
}
```

---

## Common Pitfalls

### Pitfall 1: Wrong Haptics API Signature (BLOCKS iOS BUILD)

**What goes wrong:** Using the UI-SPEC's pseudo-code `impactFeedback({ style: 'Light' })` or `notificationFeedback({ type: 'Error' })` (object arguments, PascalCase values). The actual API takes a bare string with lowercase values.
**Why it happens:** The UI-SPEC was authored with illustrative pseudo-code, not verified API calls.
**How to avoid:** Use `impactFeedback('light')`, `impactFeedback('medium')`, `impactFeedback('heavy')`, `notificationFeedback('error')`. Import from `@tauri-apps/plugin-haptics`.
**Warning signs:** TypeScript compile error "Argument of type '{ style: string }' is not assignable to parameter of type 'ImpactFeedbackStyle'".

### Pitfall 2: Unconditional Haptics Plugin Registration (BREAKS DESKTOP BUILD)

**What goes wrong:** Adding `tauri_plugin_haptics::init()` unconditionally in `lib.rs` breaks desktop compilation because the native iOS symbols don't link on macOS/Windows/Linux.
**Why it happens:** The README example shows unconditional registration; the target-gated form requires knowing to use `#[cfg(mobile)]`.
**How to avoid:** Always gate with `#[cfg(mobile)]` in `lib.rs`, and use `[target.'cfg(any(target_os = "android", target_os = "ios"))'.dependencies]` in Cargo.toml.
**Warning signs:** `error[E0433]: failed to resolve: use of undeclared crate or module` on desktop `cargo build`.

### Pitfall 3: Desktop regression from .key-touch-target overlays

**What goes wrong:** If `.key-touch-target` overlay divs are rendered on desktop, they sit on top of the SVG's existing `onClick` handlers and may intercept or duplicate clicks, changing desktop behavior.
**Why it happens:** Forgetting to iOS-gate the overlay rendering.
**How to avoid:** Render the overlay `div` elements only when `isIos === true`. On desktop, keep the existing SVG `onClick` handlers unchanged.
**Warning signs:** Desktop key clicks feel sluggish or double-fire.

### Pitfall 4: visualViewport Not Tracking Keyboard (P-iOS-06b)

**What goes wrong:** In some Tauri iOS versions (see issue #10631), `visualViewport.height` does not correctly reflect the keyboard inset.
**Why it happens:** WKWebView implementation quirk; the Tauri issue is open as of research date.
**How to avoid:** Use `window.innerHeight - vv.height - vv.offsetTop` (not just `vv.height`) as the keyboard inset calculation. Test on a real iPhone (not Simulator — Simulator does not show a software keyboard for the host Mac keyboard).
**Warning signs:** `AlphaTouchInput` bar is hidden behind the iOS keyboard on device but correct in Simulator.

### Pitfall 5: Auto-zoom from AlphaTouchInput font-size < 16px

**What goes wrong:** Setting the `<input>` font-size below 16px causes iOS to auto-zoom the viewport on focus, which fights `computeScale` and produces a jarring visual.
**Why it happens:** iOS auto-zoom behavior for small inputs is hardcoded in WebKit.
**How to avoid:** The UI-SPEC specifies 16px — do not change it. The 16px is specifically to prevent auto-zoom.
**Warning signs:** Calculator layout zooms/reflows when ALPHA mode is activated.

### Pitfall 6: portrait lock not editing Info.plist (only project.yml)

**What goes wrong:** `project.yml` is the source for XcodeGen, but the **generated** `gen/apple/hp41-gui_iOS/Info.plist` also has `UISupportedInterfaceOrientations`. Editing only `project.yml` without regenerating the Xcode project leaves the old plist in place.
**Why it happens:** The plist is a generated artifact from `project.yml` via XcodeGen/tauri ios init.
**How to avoid:** After editing `project.yml`, run `just ios-build` (or the equivalent xcodegen/tauri ios init regeneration) to sync the plist. Alternatively, edit BOTH `project.yml` AND `Info.plist` directly for immediate effect.
**Warning signs:** iPhone still rotates to landscape after editing only `project.yml`.

### Pitfall 7: Error haptic fires on every re-render

**What goes wrong:** The error haptic check after IPC fires `notificationFeedback('error')` every time `setCalcState` is called while `display_str` contains 'DATA ERROR', causing the error haptic to repeat on every subsequent state refresh (e.g., clock tick).
**Why it happens:** No guard against re-triggering on the same error state.
**How to avoid:** Use a `useRef` boolean `errorHapticFiredRef` that is set to `true` when the error haptic fires and reset when `display_str` no longer contains an error string. Only fire the haptic when transitioning from non-error to error state.
**Warning signs:** Rapid haptic buzzing while an error message is displayed.

### Pitfall 8: cfg(mobile) blind spot (MEMORY.md entry)

**What goes wrong:** `just gui-ci` (desktop CI) cannot see compile errors in `#[cfg(mobile)]` code blocks. A bug in the haptics registration code or `is_ios` command under `#[cfg(mobile)]` compiles fine on desktop but fails the iOS build.
**Why it happens:** `#[cfg(mobile)]` code is excluded from the desktop compile (the blind spot documented in MEMORY.md: `project_cfg_mobile_gate_blindspot.md`).
**How to avoid:** After writing any `#[cfg(mobile)]` code, run `cargo check --target aarch64-apple-ios` from `hp41-gui/src-tauri/` to smoke-check the mobile compile path. Add this to the implementation checklist.
**Warning signs:** Code in `#[cfg(mobile)]` blocks that silently compiles on desktop but fails the `just ios-build` step.

---

## Code Examples

### is_ios Rust command (complete, copy-ready)

```rust
// hp41-gui/src-tauri/src/commands.rs
// Add after the existing is_macos() function (line ~546)

/// Tauri command: report whether the backend was compiled for iOS.
///
/// The frontend uses this to gate touch-specific behaviors (bottom sheets,
/// collapsible stack panel, AlphaTouchInput bar, .key-touch-target overlays,
/// haptic calls). Authoritative via compile-time `cfg(target_os = "ios")` —
/// consistent with the existing `is_macos()` precedent (D-55.1).
#[tauri::command]
pub fn is_ios() -> bool {
    cfg!(target_os = "ios")
}
```

### is_ios permission TOML (complete, copy-ready)

```toml
# hp41-gui/src-tauri/permissions/is-ios.toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-is-ios"
description = "Allows the is_ios command."
commands.allow = ["is_ios"]
```

### is_ios App.tsx consumer (complete, copy-ready)

```typescript
// App.tsx — after the existing is_macos useEffect (~line 495)
const [isIos, setIsIos] = useState(false);
useEffect(() => {
  invoke<boolean>('is_ios').then(setIsIos).catch(() => setIsIos(false));
}, []);
```

### Haptics Cargo.toml entry (complete, copy-ready)

```toml
# hp41-gui/src-tauri/Cargo.toml — add new section
[target.'cfg(any(target_os = "android", target_os = "ios"))'.dependencies]
tauri-plugin-haptics = "2.3.2"
```

### Haptics lib.rs registration (complete, copy-ready)

```rust
// hp41-gui/src-tauri/src/lib.rs — inside the .setup() closure
// Add after the existing #[cfg(desktop)] autostart block:
#[cfg(mobile)]
app.handle().plugin(tauri_plugin_haptics::init())?;
```

### Haptic tier classification helper

```typescript
// Add to App.tsx or a new haptics.ts module
import { impactFeedback, notificationFeedback } from '@tauri-apps/plugin-haptics';

type HapticTier = 'light' | 'medium' | 'heavy';

// Per UI-SPEC haptic tier table (with correct lowercase values)
function getHapticTier(key: KeyDef): HapticTier {
  if (key.variant === 'shift') return 'heavy';
  if (
    key.variant === 'enter' ||
    ['sto_prompt', 'rcl_prompt', 'xeq_prompt', 'gto_prompt', 'r_s', 'rtn', 'sst', 'bst'].includes(key.id)
  ) return 'medium';
  return 'light';
}

// Usage in pointerdown handler (iOS-gated):
async function triggerHaptic(key: KeyDef): Promise<void> {
  const tier = getHapticTier(key);
  await impactFeedback(tier).catch(() => {/* desktop no-op */});
}
```

### Error haptic guard

```typescript
// In App.tsx — after setCalcState(view) in any IPC completion handler:
const errorHapticFiredRef = useRef(false);

// In the IPC .then() callback, after setCalcState:
const isError = view.display_str.includes('DATA ERROR') || view.display_str.includes('NO ROOM');
if (isIos && isError && !errorHapticFiredRef.current) {
  errorHapticFiredRef.current = true;
  notificationFeedback('error').catch(() => {});
} else if (!isError) {
  errorHapticFiredRef.current = false;
}
```

### viewport-fit=cover index.html edit

```html
<!-- index.html — replace current viewport meta -->
<meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover">
```

### Portrait lock in project.yml

```yaml
# gen/apple/project.yml — UISupportedInterfaceOrientations in hp41-gui_iOS target
# Replace the existing entry (which includes Landscape) with portrait-only:
UISupportedInterfaceOrientations:
  - UIInterfaceOrientationPortrait
# Remove UIInterfaceOrientationLandscapeLeft and UIInterfaceOrientationLandscapeRight
# Keep the ~ipad key for now (deferred to v4.2, leave as-is or also restrict)
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| FastClick.js for tap delay | `touch-action: manipulation` CSS | ~2016 (WebKit) | FastClick deprecated; CSS property is the standard |
| `document.addEventListener('touchstart')` for audio unlock | `audioCtx.resume()` inside `pointerdown` | iOS 13+ | Pointer Events API unifies mouse + touch |
| `webkitAudioContext` prefix | `AudioContext` (unprefixed) | iOS 14+ | Unprefixed works on the project's minimum deployment target |
| Fixed-position layout for modals | `visualViewport` resize listener | iOS 13+ | `innerHeight` is unreliable; `visualViewport` is the standard |

**Deprecated/outdated:**
- `webkitAudioContext`: Deprecated; iOS 14.0+ (our deployment target) supports `AudioContext` unprefixed. Do not add a prefix fallback.
- FastClick: Fully deprecated; `touch-action: manipulation` replaces it.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `webkitAudioContext` is NOT needed for iOS 14.0+ in WKWebView | Code Examples → Audio Resume | Minor code addition if wrong; `window.AudioContext || window.webkitAudioContext` fallback is safe to add defensively |
| A2 | `#[cfg(mobile)]` gate on `tauri_plugin_haptics::init()` prevents desktop link failure | Standard Stack → Pattern 2 | Build breaks on desktop if wrong — verify with `cargo build` after adding |
| A3 | Haptics JS calls with `isIos` guard gracefully no-op on desktop (catch is sufficient) | Don't Hand-Roll | If desktop build exposes the Tauri haptics commands, the call may silently succeed or return an error; the `.catch()` guard handles both |
| A4 | `visualViewport.height` correctly tracks keyboard height in Tauri v2.11 on iOS | Pattern 5 | AlphaTouchInput bar hidden behind keyboard — fallback: use `window.innerHeight - vv.height - vv.offsetTop` |
| A5 | Percentage-based positioning (left/top as % of SVG width/height) is sufficient for .key-touch-target overlay centering without needing the explicit scale factor | Pattern 6 | Overlays misaligned — fallback: pass scale factor as prop to Keyboard component |

---

## Open Questions

1. **Does `visualViewport` work correctly in Tauri v2.11 iOS builds?**
   - What we know: Tauri issue #10631 documents a problem; issue is open as of research date.
   - What's unclear: Whether the v2.11 release resolved this; the issue was filed against an older version.
   - Recommendation: The `window.innerHeight - vv.height - vv.offsetTop` calculation is the safest approach. If both vv.height and vv.offsetTop are wrong, a native Tauri plugin providing keyboard height would be needed (deferred).

2. **Should `.key-touch-target` overlays be positioned as siblings to the SVG or via SVG `<foreignObject>`?**
   - What we know: Both approaches work; `<foreignObject>` has historical WKWebView quirks.
   - What's unclear: Whether the SVG's 100% width rendering makes percentage-based positioning of sibling divs straightforward without extra measurement.
   - Recommendation: Use sibling divs with percentage positioning. Add a prop `scale` to Keyboard if needed for pixel-exact positioning.

3. **`modal_requires_alpha_label` vs accumulating label in AlphaTouchInput local state**
   - What we know: The desktop path routes individual keypresses through `pendingInput` state machine; for touch, dispatching full labels via `submit_modal_with_label` directly may be cleaner.
   - What's unclear: Whether the Rust backend expects characters submitted one-by-one (like desktop alpha_<X>) or accepts a full label string from `submit_modal_with_label`.
   - Recommendation: Use `submit_modal_with_label` for the touch path (the existing `invokeForKey` SUBMIT_MODAL_WITH_LABEL_PREFIX path already handles this). The "Done" button dispatches the full accumulated input value.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| iOS Simulator (Xcode) | TOUCH-01..11 smoke testing | ✓ (Phase 53 confirmed) | Xcode on macOS | — |
| Physical iPhone | TOUCH-01,03,04,05,06,07,08 authoritative verify | ✓ (Phase 53 confirmed on device) | iPhone SE or newer | Simulator for layout only |
| `just ios-build` recipe | iOS build pipeline | ✓ (Phase 53 established) | From Phase 53 | — |
| `@tauri-apps/plugin-haptics` 2.3.2 | TOUCH-05, TOUCH-08 | Installed in this session | 2.3.2 | — |
| `tauri-plugin-haptics` 2.3.2 Rust | TOUCH-05, TOUCH-08 | Available on crates.io | 2.3.2 | — |

**Missing dependencies with no fallback:** none — all required dependencies are available.

---

## Validation Architecture

> `workflow.nyquist_validation` key is absent from `.planning/config.json` — treated as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Vitest 4.1.6 (existing) |
| Config file | `hp41-gui/` — vitest runs from package.json `test` script |
| Quick run command | `cd hp41-gui && npm test` |
| Full suite command | `cd hp41-gui && npm test` (no separate full suite) |
| iOS cargo check | `cargo check --target aarch64-apple-ios` from `hp41-gui/src-tauri/` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | Notes |
|--------|----------|-----------|-------------------|-------|
| TOUCH-01 | 44pt hit targets present on iOS | Manual (device) | — | D-55.4 human checkpoint: iPhone SE |
| TOUCH-01 | `.key-touch-target` CSS class renders for each key | Unit (Vitest) | `npm test` | Vitest render test for Keyboard |
| TOUCH-02 | viewport-fit=cover in index.html | Automated (grep/CI) | `grep 'viewport-fit=cover' hp41-gui/index.html` | Simple file check |
| TOUCH-02 | Portrait lock in project.yml | Automated (grep) | `grep -v LandscapeLeft gen/apple/project.yml` | File check |
| TOUCH-03 | No tap delay | Manual (device) | — | D-55.4 human checkpoint: perceptible delay? |
| TOUCH-03 | `.key-touch-target` has `touch-action: manipulation` | Automated (grep/CSS) | `grep 'touch-action: manipulation' hp41-gui/src/App.css` | CSS check |
| TOUCH-04 | AlphaTouchInput renders when alpha active + isIos | Unit (Vitest) | `npm test` | Mock `isIos=true`, set `annunciators.alpha=true` |
| TOUCH-04 | AlphaTouchInput renders when modal_requires_alpha_label + isIos | Unit (Vitest) | `npm test` | Mock `isIos=true`, set `modal_requires_alpha_label=true` |
| TOUCH-04 | Input field font-size is 16px | Automated (CSS grep) | `grep 'font-size: 16px' hp41-gui/src/App.css` | Non-negotiable iOS zoom prevention |
| TOUCH-04 | AlphaTouchInput stays above keyboard | Manual (device) | — | D-55.4 human checkpoint |
| TOUCH-05 | Haptic fires on key press | Manual (device) | — | D-55.4 human checkpoint: haptics felt |
| TOUCH-05 | Haptic tier classification logic | Unit (Vitest) | `npm test` | Unit test: `getHapticTier(key)` pure function |
| TOUCH-05 | Haptics compile on iOS target | Automated | `cargo check --target aarch64-apple-ios` from hp41-gui/src-tauri/ | cfg(mobile) blind spot guard |
| TOUCH-06 | Audio audible on first tap | Manual (device) | — | D-55.4 human checkpoint: TONE audible |
| TOUCH-07 | Annunciators visible | Manual (device) | — | D-55.4 human checkpoint: SE legibility |
| TOUCH-08 | Different haptic tiers per key type | Manual (device) | — | D-55.4: SHIFT heavier than digit |
| TOUCH-08 | Error haptic fires on DATA ERROR | Manual (device) | — | D-55.4: trigger SQRT(-1), feel error haptic |
| TOUCH-08 | Error haptic does not double-fire | Unit (Vitest) | `npm test` | Test `errorHapticFiredRef` guard logic |
| TOUCH-09 | Bottom sheet components render on isIos | Unit (Vitest) | `npm test` | Mock `isIos=true`, verify sheet renders |
| TOUCH-09 | Bottom sheets do not render on desktop | Unit (Vitest) | `npm test` | Mock `isIos=false`, verify sheet absent |
| TOUCH-10 | Stack panel collapses on iOS | Unit (Vitest) | `npm test` | Mock `isIos=true`, click chevron |
| TOUCH-11 | Overscroll suppressed | Manual (device) | — | D-55.4 human checkpoint: rubber-band absent |
| TOUCH-11 | index.css has overscroll-behavior: none | Automated (grep) | `grep 'overscroll-behavior: none' hp41-gui/src/index.css` | Already present — verify not removed |

### Simulator vs Device Decision Criteria

| Criterion | Simulator OK? | Device Required? | Reason |
|-----------|--------------|-----------------|--------|
| Layout / safe-area insets | Yes (approximate) | Verify (authoritative) | Simulator safe-area can differ from real device |
| Overscroll suppression | Yes | Verify | Simulator approximates rubber-band behavior |
| Hit-target accuracy (SE) | No | Yes (D-55.4) | Simulator uses mouse; no fat-finger test |
| Tap delay | No | Yes (D-55.4) | Simulator uses mouse; 350ms delay not present |
| Tap highlight flash | No | Yes (D-55.4) | Simulator does not simulate touch highlight |
| Haptics (felt) | No | Yes (D-55.4) | Simulator has no Taptic Engine |
| Audio (TONE/BEEP) | No | Yes (D-55.4) | AudioContext suspend behavior differs |
| AlphaTouchInput keyboard tracking | No | Yes (D-55.4) | Simulator keyboard behavior differs |

### cfg(mobile) Blind-Spot Check

Per MEMORY.md (`project_cfg_mobile_gate_blindspot.md`): after any changes to code inside `#[cfg(mobile)]` blocks, run:

```bash
cargo check --target aarch64-apple-ios
```

from `hp41-gui/src-tauri/`. This catches compile errors invisible to `just gui-ci`.

### Wave 0 Gaps

- [ ] `hp41-gui/src/AlphaTouchInput.test.tsx` — covers TOUCH-04 (component renders correctly, triggers and routes correctly)
- [ ] `hp41-gui/src/BottomSheet.test.tsx` — covers TOUCH-09 (sheet renders on iOS, absent on desktop)
- [ ] Haptic tier classification unit test in existing `Keyboard.test.tsx` or new file — covers TOUCH-05 (pure function test)
- [ ] Error haptic guard unit test — covers TOUCH-08 double-fire prevention

*(Existing test infrastructure covers all other phase requirements with grep/cargo-check automation or manual checkpoints.)*

---

## Security Domain

> No user-input processing, no network calls, no authentication changes. ASVS categories V2/V3/V4/V6 do not apply to this phase.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | partial | AlphaTouchInput: characters routed through existing `alpha_<X>` IPC path; Rust validates on receipt. Backspace → `clx` (safe). No new attack surface. |
| V6 Cryptography | no | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| AlphaTouchInput injects malformed keyId | Tampering | All keyIds routed through existing `dispatch_op` which validates via `key_map::resolve()` in Rust — same path as desktop |
| Label input to submit_modal_with_label | Tampering | Existing Rust command validates label length (HP-41 limit); frontend sends verbatim string |

---

## Sources

### Primary (HIGH confidence)

- `@tauri-apps/plugin-haptics` installed dist-js/bindings.d.ts — exact TypeScript type definitions [VERIFIED: npm registry]
- `@tauri-apps/plugin-haptics` README.md — installation, registration, permissions [VERIFIED: npm registry]
- [v2.tauri.app/plugin/haptics/](https://v2.tauri.app/plugin/haptics/) — official Tauri plugin documentation [CITED]
- [tauri.app/fr/reference/javascript/haptics/](https://tauri.app/fr/reference/javascript/haptics/) — complete JavaScript API reference with type aliases [CITED]
- `hp41-gui/src/App.tsx` — modal dispatch path, is_macos pattern, existing state [VERIFIED: codebase]
- `hp41-gui/src/Keyboard.tsx` — KEY_W/KEY_H/GAP/PAD constants, keyPosition(), 44-key KeyDef inventory, onClick handler [VERIFIED: codebase]
- `hp41-gui/src-tauri/src/commands.rs` line 544 — is_macos template [VERIFIED: codebase]
- `hp41-gui/src-tauri/src/lib.rs` — plugin registration pattern, #[cfg(desktop)] gating [VERIFIED: codebase]
- `hp41-gui/src-tauri/permissions/is-macos.toml` — permission TOML template [VERIFIED: codebase]
- `hp41-gui/src-tauri/capabilities/default.json` — current capabilities structure [VERIFIED: codebase]
- `hp41-gui/src/scale.ts` — DESIGN_WIDTH=392, DESIGN_HEIGHT=900, computeScale [VERIFIED: codebase]
- `hp41-gui/src/main.tsx` — ScaledApp, ResizeObserver pattern [VERIFIED: codebase]
- `hp41-gui/src/index.css` — existing overscroll suppression (already present) [VERIFIED: codebase]
- `gen/apple/project.yml` — current orientation list (includes landscape) [VERIFIED: codebase]
- `gen/apple/hp41-gui_iOS/Info.plist` — generated plist with duplicate orientation list [VERIFIED: codebase]

### Secondary (MEDIUM confidence)

- [takazudomodular.com/pj/zudo-tauri/docs/mobile/wkwebview-gotchas/](https://takazudomodular.com/pj/zudo-tauri/docs/mobile/wkwebview-gotchas/) — visualViewport + keyboard pattern [CITED]
- [github.com/tauri-apps/tauri/issues/10631](https://github.com/tauri-apps/tauri/issues/10631) — visualViewport bug in Tauri iOS [CITED]
- [webkit.org/blog/5610/more-responsive-tapping-on-ios/](https://webkit.org/blog/5610/more-responsive-tapping-on-ios/) — touch-action: manipulation official WebKit documentation [CITED]
- [developer.apple.com/forums/thread/658375](https://developer.apple.com/forums/thread/658375) — AudioContext in WKWebView user gesture requirement [CITED]
- [developer.chrome.com/blog/300ms-tap-delay-gone-away](https://developer.chrome.com/blog/300ms-tap-delay-gone-away) — tap delay elimination [CITED]

### Tertiary (LOW confidence)

- WebSearch results for iOS WKWebView visualViewport keyboard height — multiple community sources confirm `window.innerHeight - vv.height - vv.offsetTop` pattern [noted as LOW, use with care]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — haptics API verified from installed TypeScript type definitions; Tauri command pattern verified from codebase
- Architecture: HIGH — directly derived from existing codebase analysis and locked UI-SPEC
- Pitfalls: HIGH — API discrepancy verified; cfg(mobile) requirement confirmed from official docs and README

**Research date:** 2026-06-03
**Valid until:** 2026-07-03 (30 days; Tauri plugin versions evolve; re-verify if using haptics > 2.3.2)
