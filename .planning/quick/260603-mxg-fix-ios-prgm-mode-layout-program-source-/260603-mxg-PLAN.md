---
phase: quick-260603-mxg
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - hp41-gui/src/main.tsx
  - hp41-gui/src/scale.ts
  - hp41-gui/src/scale.test.ts
  - hp41-gui/src/App.tsx
  - hp41-gui/src/App.css
autonomous: false
requirements: [MXG-01, MXG-02, MXG-03]
user_setup: []

must_haves:
  truths:
    - "In PRGM mode on iPhone the 7-segment display is fully below the Dynamic Island / status bar (not clipped)"
    - "In PRGM mode the PRGM key is tappable to exit back to normal view (BottomSheet peek does not occlude it)"
    - "The collapsed BottomSheet peek sits above the home indicator (respects safe-area-inset-bottom)"
    - "Entering/exiting PRGM (and print sheet appearing) re-fits the calculator scale"
    - "Help overlay search field + close X stay reachable (klp goal preserved)"
    - "After closing the help/settings overlay the keypad still re-fits correctly (laz behavior preserved)"
    - "Desktop window mode is pixel-unchanged (safe-area insets resolve to 0)"
  artifacts:
    - path: "hp41-gui/src/main.tsx"
      provides: "Outer un-scaled wrapper carries env(safe-area-inset-*) padding; available-area + peek measurement feeds computeScale"
      contains: "safe-area-inset"
    - path: "hp41-gui/src/scale.ts"
      provides: "computeScale fits into the inset-reduced + peek-reduced available area"
      contains: "computeScale"
    - path: "hp41-gui/src/scale.test.ts"
      provides: "Unit coverage for the reserved/available-area math"
      contains: "computeScale"
    - path: "hp41-gui/src/App.css"
      provides: ".bottom-sheet bottom respects safe-area-inset-bottom; .calculator-safe-area + .help-overlay-header insets removed (centralized)"
      contains: "env(safe-area-inset-bottom"
  key_links:
    - from: "hp41-gui/src/main.tsx (outer wrapper padding)"
      to: "hp41-gui/src/scale.ts computeScale"
      via: "measured content-box (clientW/H minus computed padding) passed as available viewport"
      pattern: "computeScale\\("
    - from: "hp41-gui/src/App.tsx recompute useEffect"
      to: "hp41-gui/src/main.tsx hp41:recompute-scale listener"
      via: "annunciators.prgm + print-sheet visibility added to recompute deps"
      pattern: "hp41:recompute-scale"
---

<objective>
Fix three coupled iOS layout bugs around PRGM mode by centralizing safe-area handling at the
un-scaled outer frame and making the scaler fit content into the inset- and peek-reduced area:

1. **Top clip** — the 7-seg display clips under the Dynamic Island because `.calculator-safe-area`
   padding lives INSIDE the `transform: scale(N)` node, so the gap renders as `scale × inset`.
2. **No recompute on PRGM toggle** — `hp41:recompute-scale` is only keyed on `[helpOpen, settingsOpen]`;
   a stale (too-small) scale persists into PRGM, and the iOS BottomSheet is `position: fixed` so the
   ResizeObserver does not fire on PRGM toggle.
3. **BottomSheet occlusion** — `.bottom-sheet` (`position: fixed; bottom: 0`) overlaps the bottom keypad
   row (incl. the PRGM key) and ignores `safe-area-inset-bottom` (overlaps the home indicator).

Purpose: PRGM mode is usable on iPhone — display unclipped, PRGM key tappable to exit, sheet above the
home indicator — without regressing the klp overlay fix or the laz overlay-close/pinch-lock fix.
Output: Centralized outer-frame safe-area padding, peek/inset-aware computeScale (+ tests), PRGM/print
recompute trigger, BottomSheet bottom inset; redundant per-component insets removed.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@CLAUDE.md
@.planning/quick/260603-klp-fix-help-overlay-search-field-rendering-/260603-klp-SUMMARY.md
@.planning/quick/260603-laz-ios-touch-polish-re-fit-calculator-on-ov/260603-laz-SUMMARY.md
@hp41-gui/src/main.tsx
@hp41-gui/src/scale.ts
@hp41-gui/src/scale.test.ts
@hp41-gui/src/BottomSheet.tsx

<interfaces>
<!-- Extracted contracts the executor needs — no codebase exploration required. -->

scale.ts (current):
```typescript
export const DESIGN_WIDTH = 392;
export const DESIGN_HEIGHT = 900;
export const MAX_SCALE = 2;
export function computeScale(
  viewportWidth: number, viewportHeight: number,
  designWidth?: number, designHeight?: number,
): number; // Math.min(MAX_SCALE, w/designW, h/designH); returns 1 for non-positive inputs
```

main.tsx ScaledApp DOM shape:
- OUTER `<div>`: width/height 100%, overflow hidden, flex, justify center, align flex-start. This node is OUTSIDE the transform → device pixels.
- INNER `<div ref={contentRef}>`: `transform: scale(${scale})`, transformOrigin 'top center', display inline-block. The `.calculator` root (and `.calculator-safe-area` on iOS) lives INSIDE this.
- `recompute()` reads `window.innerWidth/innerHeight` + `contentRef.offsetWidth/offsetHeight`, calls `computeScale(...)`.
- Existing listeners (DO NOT REMOVE — laz): window 'resize'→recompute; 'hp41:recompute-scale'→recomputeSoon (rAF + 350ms); visualViewport resize/scroll→recompute; gesture*/multi-touch touchmove preventDefault (pinch lock); ResizeObserver on contentRef.

App.tsx:
- Recompute trigger (laz): `useEffect(() => { window.dispatchEvent(new Event('hp41:recompute-scale')); }, [helpOpen, settingsOpen]);` (~L547)
- `.calculator` root: `className={\`calculator${isIos ? ' calculator-safe-area' : ''}\`}` (~L1170)
- PRGM BottomSheet: `visible={calcState.annunciators.prgm}` (~L1313)
- Print BottomSheet: `visible={printLog.length > 0}` (~L1354)

App.css anchors:
- `.calculator-safe-area` (~L142): padding-top/bottom/left/right = env(safe-area-inset-*, 0px)
- `.help-overlay-header` (~L453, klp fix): padding-top/left/right = max(10|16px, env(safe-area-inset-*,0px)); padding-bottom:10px
- `.bottom-sheet` (~L188): position fixed; bottom:0; left:0; right:0; max-height:32px (peek); z-index:50; transition max-height 300ms
- `.bottom-sheet.expanded` (~L203): max-height:40vh

index.html viewport meta (laz — keep): `viewport-fit=cover` present (required for env() insets to resolve non-zero on iOS).
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Make computeScale fit into the inset- and peek-reduced available area</name>
  <files>hp41-gui/src/scale.ts, hp41-gui/src/scale.test.ts</files>
  <behavior>
    - Add an optional `reservedHeight` parameter (default 0) to computeScale, subtracted from viewportHeight BEFORE the ratio: effective height = max(0, viewportHeight - reservedHeight). This reserves the BottomSheet peek so the keypad scales to sit ABOVE the sheet.
    - Existing 4-arg signature and all current scale.test.ts cases keep passing byte-for-byte (reservedHeight defaults to 0 = no-op).
    - Test: reservedHeight=0 → identical to current behavior (e.g. computeScale(W,H)===1 at design size).
    - Test: reservedHeight>0 shrinks the result (e.g. computeScale(DESIGN_WIDTH, DESIGN_HEIGHT+100, DESIGN_WIDTH, DESIGN_HEIGHT, 100) === 1, i.e. the 100px reserve is consumed before fitting).
    - Test: when reservedHeight >= viewportHeight, effective height is 0 → guard returns 1 (no negative/NaN scale).
    - Test: reservedHeight makes height the limiting ratio (width-generous viewport, tall reserve → height-limited result).
  </behavior>
  <action>Extend `computeScale` in scale.ts with a 5th optional param `reservedHeight: number = 0`. Compute `effectiveHeight = viewportHeight - reservedHeight`; keep the existing `if (viewportWidth <= 0 || viewportHeight <= 0) return 1` guard and ALSO return 1 when `effectiveHeight <= 0` (so a too-large reserve never yields a negative or zero-division scale). Use `effectiveHeight` in the `Math.min(MAX_SCALE, viewportWidth / designWidth, effectiveHeight / designHeight)` expression. Keep DESIGN_WIDTH/DESIGN_HEIGHT/MAX_SCALE exports unchanged. Add the four new test cases described in <behavior> to scale.test.ts in a new describe block ("computeScale — reservedHeight (BottomSheet peek + safe-area)"); do not modify existing cases. The available-area reduction for safe-area insets is applied by the CALLER (Task 2) by passing a smaller viewport width/height — computeScale itself only needs the reservedHeight knob; document this division of responsibility in a comment above the function.</action>
  <verify>
    <automated>cd hp41-gui && npx tsc --noEmit && npm test -- scale.test.ts</automated>
  </verify>
  <done>computeScale accepts reservedHeight (default 0, fully backward-compatible); all existing scale.test.ts cases still pass; 4 new reservedHeight cases pass; tsc clean.</done>
</task>

<task type="auto">
  <name>Task 2: Centralize safe-area padding on the outer frame + feed available area & peek to computeScale</name>
  <files>hp41-gui/src/main.tsx, hp41-gui/src/App.css</files>
  <action>
Three coordinated edits implementing the solution architecture (safe area handled ONCE, outside the transform):

(a) **main.tsx — outer wrapper padding (real device pixels).** Add to the OUTER wrapper div's inline style (the flex container, NOT the `transform: scale` contentRef node): `boxSizing: 'border-box'`, `paddingTop: 'env(safe-area-inset-top, 0px)'`, `paddingRight: 'env(safe-area-inset-right, 0px)'`, `paddingBottom: 'env(safe-area-inset-bottom, 0px)'`, `paddingLeft: 'env(safe-area-inset-left, 0px)'`. Because this node is OUTSIDE the transform, the Dynamic Island / home-indicator gaps render at full device pixels regardless of scale. On desktop env() = 0 → zero padding → unchanged. Add a `ref` to this outer div (e.g. `outerRef`) so recompute can measure its content box.

(b) **main.tsx — measure available content box + reserve the peek in `recompute`.** Replace the `window.innerWidth/innerHeight` reads in `recompute()` with the outer wrapper's CONTENT box (the area inside the safe-area padding): read `outerRef.current` and compute availableW/H. Robust method: `const cs = getComputedStyle(outerEl); const availW = outerEl.clientWidth - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight); const availH = outerEl.clientHeight - parseFloat(cs.paddingTop) - parseFloat(cs.paddingBottom);` (clientWidth/Height already exclude scrollbars; subtracting computed padding yields the inset-reduced area). Fall back to `window.innerWidth/innerHeight` when `outerRef.current` is null (first paint / jsdom). Document in a comment WHY the content-box measurement is chosen over a :root env() probe: it reflects the padding actually applied and needs no separate hidden probe element. Then determine the reserved peek height: when a BottomSheet is visible the collapsed peek occupies 32px at the bottom; read it from a small module constant `BOTTOM_SHEET_PEEK = 32` and reserve it only when a sheet is present. Detect sheet presence WITHOUT prop-drilling by querying the DOM inside recompute: `const peek = document.querySelector('.bottom-sheet') ? BOTTOM_SHEET_PEEK : 0;` (the sheet mounts/unmounts with visibility; recompute is already re-triggered on PRGM toggle by Task 3). Call `setScale(computeScale(availW, availH, measuredW, measuredH, peek))`. Keep the `measuredW/measuredH` content-node measurement (offsetWidth/offsetHeight of contentRef) exactly as-is.

(c) **App.css — remove now-redundant per-component insets (centralized at outer frame).** In `.calculator-safe-area` (~L142) DELETE the four `padding-*: env(safe-area-inset-*)` declarations (leave the selector empty or remove it; the `.calculator-safe-area` class may stay applied harmlessly — prefer emptying the rule body and leaving a comment "safe-area now handled by ScaledApp outer wrapper (mxg)"). In `.help-overlay-header` (~L453, the klp fix) REVERT the safe-area longhand back to flat `padding: 10px 16px`. Reasoning to VERIFY against the DOM before editing: with `.calculator` now rendered BELOW the Dynamic Island (outer-frame padding pushes the whole transformed box down), the absolutely-positioned `.help-overlay` `top:0` resolves inside the already-safe area, so a per-header inset would double-count and push the search field too low. The klp GOAL (search field reachable) is preserved by the central inset — only its implementation moves. Do NOT touch index.html viewport meta or any gesture/visualViewport/pinch code (laz).
  </action>
  <verify>
    <automated>cd hp41-gui && npx tsc --noEmit && npm test && grep -q "safe-area-inset-top" hp41-gui/src/main.tsx && ! grep -q "env(safe-area-inset" hp41-gui/src/App.css || grep -c "env(safe-area-inset-bottom" hp41-gui/src/App.css</automated>
  </verify>
  <done>Outer wrapper carries env() safe-area padding with box-sizing:border-box; recompute measures the inset-reduced content box and passes the 32px peek reserve when a .bottom-sheet is mounted; `.calculator-safe-area` and `.help-overlay-header` no longer carry env() insets; index.html + laz pinch/visualViewport code untouched; tsc + full vitest pass.</done>
</task>

<task type="auto">
  <name>Task 3: Recompute scale on PRGM/print sheet toggle + BottomSheet bottom safe-area inset</name>
  <files>hp41-gui/src/App.tsx, hp41-gui/src/App.css</files>
  <action>
(a) **App.tsx — add sheet visibility to the recompute trigger.** Extend the existing laz recompute `useEffect` (~L547) dependency array from `[helpOpen, settingsOpen]` to also include the BottomSheet visibility signals: `calcState.annunciators.prgm` and the print-sheet condition `printLog.length > 0` (use a stable boolean, e.g. add `printLog.length > 0` directly or a derived `const printSheetVisible = printLog.length > 0`). The effect body stays `window.dispatchEvent(new Event('hp41:recompute-scale'))` — entering/exiting PRGM (or the print sheet appearing/disappearing) now re-fits scale so the stale-scale-into-PRGM bug is gone and Task 2's peek reserve is recomputed when the sheet mounts/unmounts. Update the effect's comment to note PRGM/print-sheet toggles are covered (the iOS BottomSheet is position:fixed so the ResizeObserver does NOT fire on its own). Plain recompute is sufficient (no virtual keyboard on PRGM toggle) but the existing `recomputeSoon` listener handles it fine — no listener change needed.

(b) **App.css — BottomSheet above the home indicator.** Change `.bottom-sheet` (~L188) `bottom: 0;` to `bottom: env(safe-area-inset-bottom, 0px);` so the collapsed peek sits above the iPhone home indicator. On desktop env() = 0 → bottom:0 unchanged. Combined with Task 2's peek reserve (which scales the keypad to sit above the 32px peek), the PRGM key in the bottom keypad row is no longer occluded and is tappable to exit PRGM. Leave `max-height: 32px` (peek) and `.bottom-sheet.expanded { max-height: 40vh }` unchanged. Note: the peek reserve in Task 2 uses the 32px peek height; the added bottom inset shifts the sheet up but the 32px reserve remains a safe lower bound for keypad clearance — acceptable for this fix (an exact `32 + inset` reserve is not required because the keypad already clears with the 32px reserve and the inset only adds margin below the sheet).
  </action>
  <verify>
    <automated>cd hp41-gui && npx tsc --noEmit && npm test && grep -q "env(safe-area-inset-bottom" hp41-gui/src/App.css && grep -q "annunciators.prgm" hp41-gui/src/App.tsx</automated>
  </verify>
  <done>Recompute useEffect depends on prgm annunciator + print-sheet visibility; `.bottom-sheet` bottom respects safe-area-inset-bottom; tsc + full vitest pass; no Rust/IPC changes.</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <what-built>
    Centralized iOS safe-area handling at the un-scaled outer frame (main.tsx), made computeScale
    fit into the inset- and peek-reduced area (scale.ts + tests), added a PRGM/print recompute
    trigger (App.tsx), gave the BottomSheet a bottom safe-area inset, and removed the now-redundant
    per-component insets (`.calculator-safe-area`, `.help-overlay-header`).
  </what-built>
  <how-to-verify>
    Build and run on a physical iPhone (Dynamic Island device, e.g. iPhone 15 Pro) via the established
    loop: `cd hp41-gui && npm run tauri ios build` → `xcrun devicectl device install/launch`.

    1. Enter PRGM mode (tap PRGM / via GTO etc.). Confirm the 7-segment display is FULLY below the
       Dynamic Island / status bar — NOT clipped.
    2. In PRGM mode, confirm the PRGM key in the bottom keypad row is TAPPABLE — tap it to exit back
       to normal view ("can get back to normal view"). The collapsed PRGM bottom sheet must NOT
       intercept that tap.
    3. Confirm the collapsed bottom sheet peek sits ABOVE the home indicator (not under it).
    4. Normal (non-PRGM) mode: calculator unclipped top and bottom, centered.
    5. Open the `?` help overlay: search input + close X reachable below the Dynamic Island (klp goal).
    6. Close the help overlay: keypad re-fits correctly, right column not clipped (laz behavior).
    7. Trigger a print (e.g. PRX) so the print sheet appears, then dismiss — scale refits, no clip.
    8. Desktop (macOS window mode): launch with `HP41_SHOW_ON_START=1` (or normal window) and confirm
       layout is pixel-unchanged from before (insets resolve to 0).
  </how-to-verify>
  <resume-signal>Type "approved" or describe any clipping / untappable / regression issue observed.</resume-signal>
</task>

</tasks>

<verification>
- `cd hp41-gui && npx tsc --noEmit` clean.
- `cd hp41-gui && npm test` — all vitest files pass (no scale/App/HelpOverlay/BottomSheet regressions).
- `grep -n "safe-area-inset" hp41-gui/src/main.tsx` shows the four outer-wrapper insets.
- `grep -n "env(safe-area-inset" hp41-gui/src/App.css` shows ONLY `.bottom-sheet { bottom: ... }` (no `.calculator-safe-area`, no `.help-overlay-header` insets).
- index.html viewport meta + main.tsx gesture/visualViewport/pinch listeners unchanged (laz preserved).
- No changes under hp41-core/, src-tauri/, or any IPC command surface (frontend-only).
</verification>

<success_criteria>
- PRGM-mode 7-seg display clears the Dynamic Island on device (Task 4 step 1).
- PRGM key tappable to exit; bottom sheet above home indicator (Task 4 steps 2-3).
- klp (overlay search reachable) and laz (overlay-close refit + pinch lock) both still pass (Task 4 steps 5-6).
- Desktop pixel-unchanged (Task 4 step 8).
- computeScale reservedHeight math unit-tested; all existing tests green.
</success_criteria>

<output>
Create `.planning/quick/260603-mxg-fix-ios-prgm-mode-layout-program-source-/260603-mxg-SUMMARY.md` when done
</output>
