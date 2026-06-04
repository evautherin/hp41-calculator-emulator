---
phase: 55-touch-ui-adaptation
plan: "03"
subsystem: hp41-gui
tags: [ios, haptics, audio, touch, tap-feedback, error-haptic]
dependency_graph:
  requires: [55-01-is_ios-command, 55-01-isIos-frontend-flag, 55-02-onPointerDown-hook]
  provides: [haptics-ts-module, per-key-haptic-tier, error-haptic-guard, audio-resume]
  affects: [hp41-gui/src/haptics.ts, hp41-gui/src/haptics.test.ts, hp41-gui/src/App.tsx]
tech_stack:
  added: []
  patterns: [pure-classifier, ios-gated-haptic, one-shot-ref-guard, lazy-audiocontext-init]
key_files:
  created:
    - hp41-gui/src/haptics.ts
    - hp41-gui/src/haptics.test.ts
  modified:
    - hp41-gui/src/App.tsx
decisions:
  - "haptics.ts as a separate module: pure getHapticTier (testable without mocking), iOS-gated triggerHaptic, guarded maybeFireErrorHaptic — all using verified bare-string API impactFeedback('light'/'medium'/'heavy') and notificationFeedback('error')"
  - "Lazy AudioContext creation: new AudioContext() constructed inside the first onPointerDown gesture handler (audioCtxRef) so the constructor itself is inside a user-gesture context; audioResumedRef one-shot guard prevents redundant resume calls"
  - "maybeFireErrorHaptic at all three post-IPC setCalcState sites: dispatchKeyId, applyModalResult, handleClick — covers all code paths that can return a CalcStateView with a DATA ERROR or NO ROOM display"
  - "void prefix for fire-and-forget haptic calls: all haptic calls use void to satisfy TypeScript no-floating-promise; silent .catch() inside each helper guards desktop non-availability"
metrics:
  duration: "~8 minutes"
  completed_date: "2026-06-03"
  tasks: 2
  files_modified: 3
---

# Phase 55 Plan 03: Per-Key Haptics + Audio Resume Summary

**One-liner:** haptics.ts module with pure tier classifier + iOS-gated triggerHaptic + guarded error haptic, wired into App.tsx onPointerDown and all post-IPC setCalcState sites.

## What Was Built

### Task 1 RED (fc7de4a): Failing tests for haptics tier classification + error guard

Created `haptics.test.ts` covering 4 behavior groups:
- `getHapticTier` tier classification: shift→heavy, enter/prompts→medium, digit→light
- `triggerHaptic`: iOS-gated call, correct tier forwarded to impactFeedback
- `maybeFireErrorHaptic`: fires once on DATA ERROR / NO ROOM, does not re-fire on re-renders, resets when display clears

Tests failed as expected (haptics.ts did not exist).

### Task 1 GREEN (d62248f): haptics.ts module with verified API

Created `hp41-gui/src/haptics.ts` exporting:

- **`getHapticTier(key: KeyDef): 'light' | 'medium' | 'heavy'`** — pure classifier, no side effects. Rules: `variant === 'shift'` → `'heavy'`; `variant === 'enter'` OR `id ∈ {sto_prompt, rcl_prompt, xeq_prompt, gto_prompt, r_s, rtn, sst, bst}` → `'medium'`; otherwise → `'light'`.
- **`triggerHaptic(key, isIos)`** — iOS-gated `impactFeedback(tier).catch()` using the VERIFIED bare-string API (`'light'`, not `{ style: 'Light' }`).
- **`maybeFireErrorHaptic(displayStr, isIos, firedRef)`** — fires `notificationFeedback('error').catch()` once on transition into DATA ERROR / NO ROOM; resets `firedRef` when display clears (T-55-06 guard).
- **`ensureAudioResumed(audioCtx, audioResumedRef)`** — one-shot `audioCtx.resume()` inside gesture handler (TOUCH-06).

All 23 haptics.test.ts tests pass.

### Task 2 (275f0ee): Wire haptics + audio resume + error haptic into App.tsx

Modified `hp41-gui/src/App.tsx`:

- **Import:** `triggerHaptic`, `maybeFireErrorHaptic`, `ensureAudioResumed` from `./haptics`
- **New refs** (near `toastSeqRef`): `audioResumedRef`, `errorHapticFiredRef`, `audioCtxRef`
- **onPointerDown stub filled:** Lazily constructs `AudioContext` on first iOS touch → `ensureAudioResumed` → `triggerHaptic`. Both calls wrapped in `if (isIos)` guard; fire-and-forget with `void`.
- **Error haptic after each IPC response:** `maybeFireErrorHaptic` added after `setCalcState(view)` in three places: `dispatchKeyId`, `applyModalResult`, `handleClick`. Prevents haptic buzz loop on clock ticks (Pitfall 7).
- **useCallback deps updated:** `isIos` added to `dispatchKeyId`, `applyModalResult`, `handleClick` dependency arrays.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing functionality] No existing AudioContext — created lazily in onPointerDown**
- **Found during:** Task 2
- **Issue:** The plan specified "reuse the existing AudioContext used by TONE/BEEP." However, TONE/BEEP in App.tsx are routed entirely through Rust/Tauri dispatch — there is no JavaScript AudioContext in the codebase. Line 980 of App.tsx says "A future v3.x Web Audio API replacement plugs in here" indicating this is intentional.
- **Fix:** Added `audioCtxRef = useRef<AudioContext | null>(null)` and lazily construct `new AudioContext()` inside the first `onPointerDown` gesture handler (per Pattern 4 from RESEARCH.md, which explicitly shows creating the context inside a gesture). This satisfies TOUCH-06 and unblocks future Web Audio adoption.
- **Files modified:** `hp41-gui/src/App.tsx`
- **Commit:** 275f0ee

## Verification Results

| Check | Result |
|-------|--------|
| `grep 'impactFeedback' haptics.ts` (bare string, not object) | ✓ `impactFeedback(tier)` with lowercase string |
| `grep '{ style:' haptics.ts` (WRONG form absent) | ✓ 0 matches |
| `grep "notificationFeedback" haptics.ts` | ✓ `notificationFeedback('error')` present |
| `grep 'getHapticTier' haptics.ts + haptics.test.ts` | ✓ both match |
| `cd hp41-gui && npm test -- haptics` (4 behavior groups) | ✓ 23/23 tests pass |
| `grep 'triggerHaptic' App.tsx` | ✓ 2 occurrences (import + call) |
| `grep 'errorHapticFiredRef' App.tsx` | ✓ 5 occurrences |
| `grep 'maybeFireErrorHaptic' App.tsx` | ✓ 4 occurrences (import + 3 call sites) |
| `grep 'audioResumedRef' App.tsx` | ✓ 3 occurrences |
| `grep 'ensureAudioResumed' App.tsx` | ✓ 2 occurrences (import + call) |
| `grep '{ style:' App.tsx` (wrong form absent) | ✓ 0 haptic-related matches |
| `cd hp41-gui && npm test` (full suite) | ✓ 250/250 tests pass |
| `cd hp41-gui && just gui-ci` | ✓ release build clean, 250 tests pass |

## Known Stubs

None. All haptic and audio-resume hooks are fully wired. Device-level verification (haptics felt, correct tier, error haptic on DATA ERROR, audio audible on BEEP/TONE after first tap) is deferred to Plan 06 per D-55.4 (manual device checkpoint).

## Threat Flags

No new threat surface beyond the plan's `<threat_model>`:
- T-55-05 (accepted): Haptic tier args are fixed string literals from `getHapticTier`; no user-controlled value reaches the plugin.
- T-55-06 (mitigated): `errorHapticFiredRef` guard confirmed firing once per error transition and resetting on clear; haptic buzz loop on clock ticks prevented.

## TDD Gate Compliance

- RED gate: commit `fc7de4a` (`test(55-03): add failing tests...`) — tests failed as expected
- GREEN gate: commit `d62248f` (`feat(55-03): implement haptics.ts...`) — all 23 tests pass

## Self-Check: PASSED

- `hp41-gui/src/haptics.ts` — EXISTS ✓
- `hp41-gui/src/haptics.test.ts` — EXISTS ✓
- `hp41-gui/src/App.tsx` (triggerHaptic + errorHapticFiredRef + ensureAudioResumed) — EXISTS ✓
- Commit fc7de4a (Task 1 RED) — EXISTS ✓
- Commit d62248f (Task 1 GREEN) — EXISTS ✓
- Commit 275f0ee (Task 2) — EXISTS ✓
