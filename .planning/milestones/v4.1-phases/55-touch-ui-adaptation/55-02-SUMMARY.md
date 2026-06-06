---
phase: 55-touch-ui-adaptation
plan: "02"
subsystem: hp41-gui
tags: [ios, touch, css, keyboard, hit-targets, safe-area, tap-feedback]
dependency_graph:
  requires: [55-01-is_ios-command, 55-01-isIos-frontend-flag]
  provides: [key-touch-target-overlay, calculator-safe-area-padding, onPointerDown-hook]
  affects: [hp41-gui/src/App.css, hp41-gui/src/Keyboard.tsx, hp41-gui/src/App.tsx, hp41-gui/src/App.test.tsx, hp41-gui/src/Keyboard.test.tsx]
tech_stack:
  added: []
  patterns: [transparent-hit-area-overlay, percentage-based-positioning, pointer-events-discipline, touch-action-manipulation]
key_files:
  created: []
  modified:
    - hp41-gui/src/App.css
    - hp41-gui/src/Keyboard.tsx
    - hp41-gui/src/App.tsx
    - hp41-gui/src/App.test.tsx
    - hp41-gui/src/Keyboard.test.tsx
decisions:
  - "Transparent overlay architecture: one .key-touch-target div per wired key (Pattern 6 Option A), positioned as % of SVG container — no scale factor needed, works with existing computeScale/ResizeObserver"
  - "SVG <g> onClick suppressed when isIos && key.id — overlay owns dispatch to prevent double-fire (Pitfall 3 / T-55-04 DoS mitigation)"
  - "onPointerDown stub in App.tsx for Plan 03 hookpoint — synchronous in user-gesture handler, required for iOS AudioContext.resume() and haptics"
  - "App.test.tsx beforeEach + H1/H2 tests: is_ios and is_macos must return boolean false (not CalcStateView) to keep isIos=false in tests and preserve SVG onClick path"
metrics:
  duration: "~8 minutes"
  completed_date: "2026-06-03"
  tasks: 3
  files_modified: 5
---

# Phase 55 Plan 02: Touch Hit-Targets + Safe-Area + Tap Feedback Summary

**One-liner:** 44pt transparent hit-target overlays per key (isIos-gated), .calculator-safe-area padding for notch/home-indicator clearance, and immediate onPointerDown hook for Plan 03 haptics/audio.

## What Was Built

### Task 1 (f3dfde8): Add .key-touch-target and .calculator-safe-area CSS classes

Appended the Phase-55 touch layer block to `App.css` after the `.key-bevel` rule (P51 invariant preserved — `transform-box: fill-box` stays in App.css, never moved to themes.css):

- `.key-touch-target`: `position: absolute`, `min-width/height: 44px`, `touch-action: manipulation`, `-webkit-tap-highlight-color: transparent`, `transform: translate(-50%, -50%)`, `background: transparent`, `border: none` (TOUCH-01 + TOUCH-03)
- `.calculator-safe-area`: `env(safe-area-inset-top/bottom/left/right, 0px)` padding on all four sides (TOUCH-02)

### Task 2 (6e0081f + 8955c49): iOS-gated overlays + onPointerDown in Keyboard.tsx (TDD)

**RED (6e0081f):** Added 3 behavior tests to `Keyboard.test.tsx`:
- Test 1: `isIos=true` → one `.key-touch-target` per wired key (44 overlays)
- Test 2: `isIos=false` → zero overlays (desktop regression guard)
- Test 3: clicking overlay dispatches `onKey` exactly once (no double-fire)

**GREEN (8955c49):**
- Added `isIos?: boolean` and `onPointerDown?: (key: KeyDef) => void` to `KeyboardProps` interface
- Wrapped `<svg>` in a `position: relative` `<div>` to anchor absolute overlay divs
- Rendered one `.key-touch-target` div per wired key (`isIos && key.id` gate), positioned at `${(pos.x + pos.w/2)/KEYBOARD_W*100}%` / `${(pos.y + pos.h/2)/KEYBOARD_H*100}%` (Pattern 6 Option A — percentage of SVG container, no scale factor)
- SVG `<g>` `onClick` conditionally suppressed when `isIos && key.id` (overlay owns dispatch, preventing double-fire — T-55-04 mitigation)
- `onPointerDown` fires immediately on touch for Plan 03 haptic/audio hookpoint

### Task 3 (d710682): Apply .calculator-safe-area to wrapper (iOS-gated)

- `App.tsx`: `className={`calculator${isIos ? ' calculator-safe-area' : ''}`}` applied conditionally on `isIos`
- `App.tsx`: `isIos={isIos}` and `onPointerDown={(_key) => { /* stub — Plan 03 */ }}` wired through to `<Keyboard>`
- `scale.ts` and `main.tsx` unchanged — existing `ResizeObserver` + `computeScale` recompute automatically when safe-area padding shrinks the content node
- `App.test.tsx`: Fixed `beforeEach` mock to return `false` for `is_ios` and `is_macos` (previously returned `makeEmptyView()` object, which made `isIos` truthy and suppressed SVG onClick in tests). Fixed H1/H2 tests from `mockResolvedValue` to `mockImplementation` for the same reason.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] App.test.tsx: is_ios mock returned truthy CalcStateView object**
- **Found during:** Task 3 (after wiring `isIos` prop to `<Keyboard>`)
- **Issue:** The `beforeEach` `mockImplementation` returned `makeEmptyView()` (a CalcStateView object, truthy) for any unrecognized `invoke` command, including `is_ios`. When `isIos={isIos}` started flowing through to `<Keyboard>`, `isIos=truthy` caused the SVG `<g>` `onClick` to be suppressed for all keys, breaking 21 of 27 App.test.tsx tests.
- **Fix:** Added explicit `if (cmd === 'is_ios') return Promise.resolve(false)` and `if (cmd === 'is_macos') return Promise.resolve(false)` branches to `beforeEach` mock. Also converted H1 and H2 from `mockResolvedValue` to `mockImplementation` (they replaced the entire mock with a CalcStateView-returning function).
- **Files modified:** `hp41-gui/src/App.test.tsx`
- **Commit:** d710682

## Verification Results

| Check | Result |
|-------|--------|
| `grep 'touch-action: manipulation' App.css` | ✓ line 132 (inside .key-touch-target) |
| `grep '-webkit-tap-highlight-color: transparent' App.css` | ✓ |
| `grep 'min-width: 44px' App.css` | ✓ |
| `grep 'min-height: 44px' App.css` | ✓ |
| `grep 'env(safe-area-inset' App.css` | ✓ 4 lines in .calculator-safe-area |
| P51 guard: `grep -c 'transform-box: fill-box' themes.css` | ✓ 0 matches |
| `grep 'key-touch-target' Keyboard.tsx` | ✓ class + comment |
| `grep 'onPointerDown' Keyboard.tsx` | ✓ prop + render |
| `grep 'calculator-safe-area' App.tsx` | ✓ conditional className |
| `cd hp41-gui && npm test -- Keyboard` | ✓ 44/44 tests (incl. 3 new) |
| `cd hp41-gui && just gui-ci` | ✓ 227/227 tests, release build clean |
| scale.ts + main.tsx in `git diff` | ✓ not listed (unchanged) |

## Known Stubs

**onPointerDown handler in App.tsx** (intentional, Plan 03 fills it):
- File: `hp41-gui/src/App.tsx`, near line 1150
- Content: `onPointerDown={(_key) => { /* stub — Plan 03 */ }}`
- Reason: Plan 02 establishes the prop contract; Plan 03 wires haptics (`impactFeedback`) + audio (`ensureAudioResumed`) inside this handler. The stub ensures the onPointerDown prop is exercised by the overlay divs without effect on this plan.

## Threat Flags

No new threat surface beyond the plan's `<threat_model>`:
- T-55-03 (accepted): Overlay dispatches through the same `onKey → dispatch_op` path; Rust `key_map::resolve` validates identically.
- T-55-04 (mitigated): SVG `<g>` onClick suppressed when `isIos && key.id`; `busyRef` dedup retained as safety net.

## Self-Check: PASSED

- `hp41-gui/src/App.css` (`.key-touch-target` + `.calculator-safe-area`) — EXISTS ✓
- `hp41-gui/src/Keyboard.tsx` (`key-touch-target` class + `onPointerDown`) — EXISTS ✓
- `hp41-gui/src/App.tsx` (`calculator-safe-area` + `isIos` prop) — EXISTS ✓
- Commit f3dfde8 (Task 1) — EXISTS ✓
- Commit 6e0081f (Task 2 RED) — EXISTS ✓
- Commit 8955c49 (Task 2 GREEN) — EXISTS ✓
- Commit d710682 (Task 3) — EXISTS ✓
