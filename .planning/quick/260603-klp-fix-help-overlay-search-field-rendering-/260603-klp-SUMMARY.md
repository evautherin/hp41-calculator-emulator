---
phase: quick-260603-klp
plan: 01
subsystem: gui-frontend
tags: [ios, safe-area, css, help-overlay, touch-ui]
dependency_graph:
  requires: []
  provides: [help-overlay-safe-area-insets]
  affects: [hp41-gui/src/App.css]
tech_stack:
  added: []
  patterns: [env(safe-area-inset-*) + max() via Phase 55 TOUCH-02]
key_files:
  created: []
  modified:
    - hp41-gui/src/App.css
decisions:
  - "Reused Phase 55 TOUCH-02 env(safe-area-inset-*) + max(base, inset) pattern rather than an iOS-only class; desktop padding unchanged"
  - "Ran frontend-only gate (npm ci + tsc --noEmit + npm test) instead of full just gui-ci to avoid slow Rust release build for a pure CSS change"
metrics:
  duration: "~5 min"
  completed: "2026-06-03T12:55:49Z"
  tasks_completed: 2
  tasks_pending: 1
  files_modified: 1
---

# Quick Task 260603-klp: Fix Help Overlay Search Field Rendering (iOS Safe Area)

**One-liner:** Added `env(safe-area-inset-top/left/right)` via `max()` to `.help-overlay-header` so the search input and close X clear the iOS status bar / Dynamic Island.

## What Was Done

The help overlay (`?` panel) rendered its header — containing the search input and close X button — pinned to the very top of the iPhone screen, underneath the iOS status bar / Dynamic Island. Both UI elements were untappable on device.

**Root cause (from plan):** `.help-overlay` is `position: absolute; top:0` inside the `.calculator` wrapper. The wrapper gained `.calculator-safe-area` padding in Phase 55 (TOUCH-02), but CSS absolute positioning resolves `top:0` to the padding-box edge — wrapper padding does NOT push absolute children inward. The overlay header had to consume the safe-area insets itself.

**Fix:** Replaced flat `padding: 10px 16px` in `.help-overlay-header` (App.css L453) with safe-area-aware longhand padding using the Phase 55 TOUCH-02 established pattern:

```css
padding-top:    max(10px, env(safe-area-inset-top,   0px));
padding-left:   max(16px, env(safe-area-inset-left,  0px));
padding-right:  max(16px, env(safe-area-inset-right, 0px));
padding-bottom: 10px;
```

On desktop, `env(safe-area-inset-*)` resolves to 0, so effective padding stays 10px/16px — pixel-identical to before. No new class, no JS gating.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| Task 1 | Add safe-area insets to `.help-overlay-header` in App.css | 8d0fd00 |
| Task 2 | GUI frontend test + build gate (tsc + vitest) | — (part of Task 1 commit) |

## Build / Test Gate (Task 2)

**Recipe chosen:** Frontend-only sub-steps of `just gui-ci` (skipping the slow `cargo build --release` Rust step — irrelevant to a pure CSS change):

1. `cd hp41-gui && npm ci` — dependencies installed
2. `cd hp41-gui && npx tsc --noEmit` — TypeScript compilation: PASS
3. `cd hp41-gui && npm test` — Vitest run: 11 files, 267 tests — all PASS (2.87 s)

No HelpOverlay test regressions. Vite/lightningcss accepted `max()` + `env()` CSS without error.

## Deviations from Plan

None — plan executed exactly as written.

## Pending: Task 3 (On-Device iPhone Verification)

Task 3 is a `type="checkpoint:human-verify" gate="blocking"` requiring physical iPhone confirmation. It CANNOT be automated. The automated tasks are complete and committed; human on-device verification is required before this quick task is marked fully done.

**To verify:**
1. Build and run the GUI on iPhone (`just gui-ios-dev` or Tauri iOS dev command)
2. Press `?` to open the help overlay
3. Confirm the search input ("Search functions...") is FULLY visible below the status bar / Dynamic Island and accepts taps
4. Confirm the close X button (top-right) is FULLY visible and tappable
5. (Landscape, optional) Confirm no clipping at Dynamic Island left/right edges
6. On macOS desktop, confirm the overlay header looks unchanged (10px/16px padding, no extra top gap)

Resume signal: Type "approved" once both elements are reachable on iPhone and desktop is unchanged.

## Self-Check

- [x] `hp41-gui/src/App.css` modified (8 insertions, 1 deletion) — confirmed
- [x] Commit `8d0fd00` exists on `worktree-agent-ac6d3037ccd1479f7`
- [x] Grep confirms all three inset references present in `.help-overlay-header`:
  - `env(safe-area-inset-top` — FOUND
  - `env(safe-area-inset-left` — FOUND
  - `env(safe-area-inset-right` — FOUND
- [x] 267 Vitest tests pass, 0 failures

## Self-Check: PASSED
