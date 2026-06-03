---
phase: quick-260603-mxg
plan: 01
subsystem: hp41-gui/frontend
tags: [ios, safe-area, prgm-mode, bottom-sheet, scale]
requirements: [MXG-01, MXG-02, MXG-03]
decisions:
  - "Centralize env(safe-area-inset-*) at the un-scaled outer wrapper in main.tsx, not per-component"
  - "computeScale reservedHeight param for BottomSheet peek (32px) keeps scaler concerns separated from inset concerns"
  - "Revert .help-overlay-header to flat padding after outer-frame centralization — no double-count"
  - "DOM query for .bottom-sheet in recompute() avoids prop-drilling peek visibility to main.tsx"
metrics:
  duration: "~20 min"
  completed: "2026-06-03"
  tasks_completed: 3
  tasks_total: 4
  files_changed: 4
key_files:
  modified:
    - hp41-gui/src/scale.ts
    - hp41-gui/src/scale.test.ts
    - hp41-gui/src/main.tsx
    - hp41-gui/src/App.css
    - hp41-gui/src/App.tsx
    - hp41-gui/src/index.css
verification:
  on_device: "PASS (iPhone 15 Pro, 2026-06-03) — top 7-seg display fully below the Dynamic Island in all modes."
  followup_fix: "The initial mxg build still clipped because the outer-frame safe-area padding was a React INLINE env() style, which WKWebView drops (resolves to 0). Fixed by moving it to a .scaled-app-frame stylesheet class in index.css (commit 6fc9d6c) — env() in a stylesheet resolves correctly. Verified PASS after that."
  superseded_part: "The bottom-sheet peek-reservation + occlusion handling (BOTTOM_SHEET_PEEK, .bottom-sheet bottom inset) targeted the PRGM BottomSheet, which 260603-o2e then REMOVED entirely (authentic single-step view). The peek logic still applies to the remaining print sheet. The position:fixed-inside-transform trap of the print sheet remains a known follow-up."
---

# Quick Fix 260603-mxg: iOS PRGM-Mode Layout Fix — Summary

One-liner: Centralize iOS safe-area padding at the un-scaled outer frame so the Dynamic Island gap renders at device pixels; add reservedHeight to computeScale for BottomSheet peek; trigger recompute on PRGM/print toggle.

## What Changed

### Task 1 — computeScale reservedHeight extension (340557e)

`hp41-gui/src/scale.ts`: Added optional 5th parameter `reservedHeight: number = 0` to `computeScale`.
- Computes `effectiveHeight = viewportHeight - reservedHeight` before the ratio.
- Guards return 1 when `effectiveHeight <= 0` (prevents NaN/negative scale from an oversized reserve).
- Documented the caller-side division of responsibility: safe-area inset reduction is the caller's job (passed as smaller viewport); reservedHeight is only for UI chrome sitting on top of the available area (BottomSheet peek).
- All existing 9 test cases pass byte-for-byte; 4 new cases in a new describe block cover: zero-reserve no-op, 100px-reserve consuming extra height, oversized-reserve guard (returns 1), and height-limiting reserve on width-generous viewport.

`hp41-gui/src/scale.test.ts`: New describe block "computeScale — reservedHeight (BottomSheet peek + safe-area)" with 4 `it` cases. Existing describe block untouched.

**Test results:** 14 tests pass (9 existing + 4 new + 1 pre-existing — vitest counts 14). tsc clean.

### Task 2 — Outer frame safe-area + inset-reduced available area (e0f3406)

`hp41-gui/src/main.tsx`:
- Added `outerRef` (`useRef<HTMLDivElement>`) on the outer wrapper div (OUTSIDE the `transform: scale()` node).
- Added `boxSizing: 'border-box'` + four `paddingXxx: 'env(safe-area-inset-Xxx, 0px)'` inline styles to the outer wrapper. Because this node is not transformed, the Dynamic Island / home-indicator gaps render at full device pixels regardless of scale. On desktop `env() = 0` — zero padding — layout unchanged.
- In `recompute()`, replaced `window.innerWidth/innerHeight` with the outer wrapper's content-box measurement: `outerEl.clientWidth - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight)` (and analogously for height). Falls back to `window.inner*` when `outerRef.current` is null (first paint / jsdom).
- Added `BOTTOM_SHEET_PEEK = 32` constant and DOM query `document.querySelector('.bottom-sheet')` inside `recompute()` to detect sheet presence without prop-drilling; passes peek to `computeScale` as 5th arg.
- All laz listeners (visualViewport resize/scroll, `hp41:recompute-scale`, gesture*/touchmove pinch-lock, ResizeObserver) are untouched.

`hp41-gui/src/App.css`:
- `.calculator-safe-area`: removed all four `padding-*: env(safe-area-inset-*)` declarations. Selector and class kept; empty rule body with comment "safe-area now handled by ScaledApp outer wrapper (mxg)". The `.calculator-safe-area` class is still applied in App.tsx harmlessly.
- `.help-overlay-header`: reverted from klp per-header insets back to `padding: 10px 16px` (flat). Reasoning: `.help-overlay` is `position: absolute; top: 0` inside `.calculator`, which is now pushed below the Dynamic Island by the outer-frame padding. A per-header inset would double-count and push the search field too low. The klp GOAL (search field reachable on iPhone) is preserved via the outer-frame inset.

**Test results:** 271 tests pass (11 files). tsc clean.

### Task 3 — PRGM/print recompute trigger + BottomSheet bottom inset (5a4a240)

`hp41-gui/src/App.tsx`:
- Extended the laz recompute `useEffect` dependency array from `[helpOpen, settingsOpen]` to `[helpOpen, settingsOpen, calcState?.annunciators.prgm, printLog.length > 0]`.
- Comment updated to explain: the iOS BottomSheet is `position: fixed` so the `ResizeObserver` in main.tsx does NOT fire on its mount/unmount; adding prgm + print-sheet visibility here ensures the stale-scale-into-PRGM bug is cleared on every toggle and Task 2's peek reserve is recomputed.
- Effect body unchanged: `window.dispatchEvent(new Event('hp41:recompute-scale'))`.

`hp41-gui/src/App.css`:
- `.bottom-sheet`: changed `bottom: 0` to `bottom: env(safe-area-inset-bottom, 0px)` so the collapsed 32px peek sits above the iPhone home indicator. On desktop `env() = 0` → `bottom: 0` unchanged.

**Test results:** 271 tests pass. tsc clean. No Rust / src-tauri / IPC changes.

## How laz and klp Fixes Were Preserved

### laz (260603-laz) — overlay-close refit + pinch-lock

- The `hp41:recompute-scale` custom-event listener in main.tsx is untouched.
- The `recomputeSoon` function (rAF + 350ms re-measure) is untouched.
- The `visualViewport` resize/scroll listeners are untouched.
- The WebKit `gesture*` / multi-touch `touchmove` preventDefault pinch-lock is untouched.
- The `recompute` function now reads the outer wrapper's content-box instead of `window.innerWidth/innerHeight`, but on desktop these are equivalent (outer wrapper = 100% with 0 padding). The laz behavior of re-measuring over 350ms after overlay-close still fires via `recomputeSoon`.

### klp (260603-klp) — help overlay search field + close X reachable on iPhone

The `.help-overlay-header` safe-area longhand was reverted to `padding: 10px 16px`. This is safe because:

**DOM reasoning:** `.help-overlay` has `position: absolute; top: 0; left: 0; right: 0; bottom: 0` inside `.calculator` (which is a `position: relative` flex child). The outer wrapper now has `paddingTop: env(safe-area-inset-top)` — so the `.calculator` box starts below the Dynamic Island. The `.help-overlay`'s `top: 0` resolves relative to `.calculator`, which is already below the notch. Therefore `padding-top: 10px` in the header is sufficient clearance without any `env()`. A per-header inset would add to an already-safe position and push the search field too low.

**Conclusion:** klp GOAL (search input + close X reachable below the Dynamic Island) is fully preserved by the outer-frame inset moving the entire `.calculator` down. No per-header inset needed or correct.

## Pending Checkpoint

**Task 4 (checkpoint:human-verify)** requires on-device verification on a physical iPhone with Dynamic Island (iPhone 15 Pro or similar). The 8 verification steps are listed in the plan.

Gate command: `cd hp41-gui && npm run tauri ios build`

## Self-Check

- [x] hp41-gui/src/scale.ts modified
- [x] hp41-gui/src/scale.test.ts modified  
- [x] hp41-gui/src/main.tsx modified
- [x] hp41-gui/src/App.css modified
- [x] hp41-gui/src/App.tsx modified
- [x] Commits 340557e, e0f3406, 5a4a240 exist on worktree-agent branch
- [x] tsc clean (3/3 runs)
- [x] 271 vitest tests pass (3/3 runs — Task 1: 14, Task 2: 271, Task 3: 271)
- [x] No hp41-core / src-tauri / IPC changes
- [x] laz listeners untouched in main.tsx
- [x] index.html viewport meta (viewport-fit=cover) untouched

## Self-Check: PASSED
