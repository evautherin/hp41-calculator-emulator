---
phase: quick-260603-laz
plan: inline
subsystem: gui-frontend
tags: [ios, touch-ui, viewport, scaling, pinch-zoom, help-overlay]
dependency_graph:
  requires: [help-overlay-safe-area-insets]
  provides: [overlay-close-refit, webview-pinch-lock]
  affects: [hp41-gui/src/App.tsx, hp41-gui/src/main.tsx, hp41-gui/index.html]
tech_stack:
  added: []
  patterns: [custom-event recompute trigger, visualViewport listener, WebKit gesture* preventDefault]
key_files:
  created: []
  modified:
    - hp41-gui/src/App.tsx
    - hp41-gui/src/main.tsx
    - hp41-gui/index.html
decisions:
  - "Re-fit via a window 'hp41:recompute-scale' custom event keyed on helpOpen/settingsOpen — no prop-drilling helpOpen into ScaledApp, covers every close path (X, Esc, Esc-in-overlay)"
  - "Re-measure across rAF + 350ms after overlay close to outlast the iOS keyboard-dismiss animation (WKWebView does not reliably fire 'resize' on keyboard hide)"
  - "Subscribe to visualViewport resize/scroll as the reliable iOS keyboard show/hide signal alongside window 'resize'"
  - "Lock pinch-zoom via viewport meta (maximum-scale=1, user-scalable=no) PLUS WebKit gesture*/multi-touch touchmove preventDefault belt-and-suspenders — the calculator owns its own CSS-transform fit, user zoom only pans/offsets it"
metrics:
  duration: "~25 min (2 device-verify iterations)"
  completed: "2026-06-03"
  tasks_completed: 2
  tasks_pending: 0
  files_modified: 3
verification:
  on_device: "PASS (iPhone 15 Pro, 2026-06-03) — keypad fits after overlay close; pinch-zoom disabled; layout centered"
---

# Quick Task 260603-laz: iOS Touch Polish — Overlay-Close Re-fit + Pinch-Zoom Lock

**One-liner:** Re-fit the auto-scaled calculator whenever the help/settings overlay
closes, and lock the webview against pinch-zoom — two pre-existing iOS bugs
surfaced while verifying the safe-area fix [[260603-klp]].

## Origin

These bugs were discovered during on-device verification of quick task **260603-klp**
(help-overlay safe-area insets). That fix made the overlay close ✕ reachable on
iPhone for the first time, which exposed two latent issues in the iOS touch UI.

## Fix 1 — Calculator clipped after closing the overlay (`5b4735f`)

**Symptom:** After opening the `?` help overlay and closing it with ✕, the
calculator keypad rendered too wide — the right column (ALPHA / LN / TAN …)
clipped off the screen edge.

**Root cause (pre-existing):** The auto-scaler `ScaledApp` (`main.tsx`) recomputed
the CSS-transform scale only on mount, on window `resize`, and via a
`ResizeObserver` on the content node. The help search input's `autoFocus` raises
the iOS virtual keyboard, shrinking the viewport and the computed scale; closing
the overlay dismisses the keyboard but WKWebView does **not** reliably fire a
window `resize` on keyboard hide, so the scaler stayed stuck at the
keyboard-visible (too-small) scale. No recompute was tied to overlay visibility.

**Fix:**
- `App.tsx`: `useEffect` keyed on `[helpOpen, settingsOpen]` dispatches a
  `window` `'hp41:recompute-scale'` event whenever an overlay opens/closes —
  covers every close path without prop-drilling state into `ScaledApp`.
- `main.tsx`: listens for that event and re-measures across `requestAnimationFrame`
  + a 350ms timeout to outlast the keyboard-dismiss animation. Also subscribes to
  `window.visualViewport` `resize`/`scroll` as the reliable iOS keyboard
  show/hide signal.

## Fix 2 — Pinch-zoom enabled / layout shifted right (`1e485ff`)

**Symptom:** The calculator could be pinch-zoomed with two fingers on iPhone
(unwanted for a fixed-layout calculator), and a residual pinch-pan left the
content visibly shifted to the right.

**Root cause (pre-existing):** The viewport meta allowed user scaling
(`initial-scale=1` only, no `maximum-scale`/`user-scalable`). The calculator
manages its own fit via a CSS transform, so browser pinch-zoom only pans/offsets
the locked layout.

**Fix:**
- `index.html`: viewport meta gains `maximum-scale=1.0, minimum-scale=1.0,
  user-scalable=no` (kept `viewport-fit=cover` for safe-area insets).
- `main.tsx`: `preventDefault` on WebKit `gesturestart`/`gesturechange`/`gestureend`
  and on multi-touch `touchmove` — the WKWebView-reliable belt-and-suspenders.
  Single-finger taps and one-finger scroll are unaffected.

A fresh relaunch resets the webview to scale 1, un-panned — which also resolved
the "shifted right" appearance.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Re-fit calculator scale on help/settings overlay close (iOS keypad clip) | `5b4735f` |
| 2 | Lock webview scale to disable pinch-zoom on iOS | `1e485ff` |

## Build / Test Gate

- `npx tsc --noEmit` — PASS (WebKit gesture* event names resolve via
  `addEventListener`'s string-fallback overload)
- `npm test` (Vitest) — 11 files, 267 tests, all PASS

## On-Device Verification — PASS

iPhone 15 Pro, 2026-06-03, via `npm run tauri ios build` → `xcrun devicectl
install/launch`:
1. ✅ Keypad fits correctly after opening + closing the help overlay (no clip)
2. ✅ Pinch-zoom disabled (two-finger gesture no longer zooms)
3. ✅ Calculator horizontally centered (no right shift)

## Self-Check: PASSED

- [x] `hp41-gui/src/App.tsx`, `hp41-gui/src/main.tsx`, `hp41-gui/index.html` modified
- [x] Commits `5b4735f` + `1e485ff` on `develop`
- [x] 267 Vitest tests pass; `tsc --noEmit` clean
- [x] On-device PASS on iPhone 15 Pro
