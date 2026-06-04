---
phase: 54-ios-persistence-layer
plan: "02"
subsystem: ui
tags: [react, typescript, tauri, ios, persistence, visibilitychange]

# Dependency graph
requires:
  - phase: 54-ios-persistence-layer
    provides: "save_state Tauri command (plan 54-01)"
provides:
  - "visibilitychange useEffect in App.tsx that invokes save_state when document becomes hidden"
  - "PERSIST-02 frontend trigger: iOS backgrounding fires background save"
affects: [54-03-on-device-verification]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "document.addEventListener('visibilitychange') + StrictMode-safe cleanup in useEffect with empty deps"
    - "fire-and-forget void invoke<void>(...).catch() for non-fatal background operations"

key-files:
  created: []
  modified:
    - hp41-gui/src/App.tsx

key-decisions:
  - "Used empty [] dependency array: handler has no React-state dependency; always saves current backend state"
  - "fire-and-forget invoke: background-save failure logged via console.warn only; 30s thread is the safety net (D-54.2a)"
  - "Placed immediately after keydown useEffect to mirror the same addEventListener+cleanup pattern (D-12)"
  - "No pagehide or beforeunload listeners added per D-54.2c"

patterns-established:
  - "Pattern: visibilitychange useEffect with empty deps and StrictMode-safe cleanup — document.addEventListener / removeEventListener analog to the keydown useEffect at lines 933-936"

requirements-completed: [PERSIST-02]

# Metrics
duration: 8min
completed: 2026-06-02
---

# Phase 54 Plan 02: iOS Background Save Trigger Summary

**React visibilitychange useEffect that fires invoke('save_state') on document hidden, with StrictMode-safe cleanup and fire-and-forget error handling**

## Performance

- **Duration:** 8 min
- **Started:** 2026-06-02T00:00:00Z
- **Completed:** 2026-06-02T00:08:00Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- Added `visibilitychange` useEffect in App.tsx immediately after the existing keydown listener (lines 938-956)
- Handler guards on `document.visibilityState === 'hidden'` and calls `void invoke<void>('save_state').catch(...)` — fire-and-forget
- StrictMode-safe: `return () => document.removeEventListener('visibilitychange', handleVisibilityChange)` cleanup prevents double-registration
- TypeScript type-check (`npx tsc --noEmit`) passes cleanly
- All acceptance criteria met: visibilitychange count == 2, invoke count >= 1, pagehide/beforeunload count == 0

## Task Commits

Each task was committed atomically:

1. **Task 1: Add visibilitychange → save_state listener in App.tsx** - `e4cee0a` (feat)

## Files Created/Modified

- `hp41-gui/src/App.tsx` — Added visibilitychange useEffect (18 insertions), no deletions

## Decisions Made

Followed plan exactly as specified. The plan's acceptance criteria specified `grep -c "visibilitychange" == 2` — the two functional occurrences are `document.addEventListener('visibilitychange', ...)` and `document.removeEventListener('visibilitychange', ...)`. Comments were carefully worded to avoid adding extra occurrences of the exact string being tested, while still documenting the constraint (D-54.2c) without using the forbidden keywords.

## Deviations from Plan

None — plan executed exactly as written. The minor wording adjustment to comments (to avoid extra grep matches against acceptance-criteria strings) was cosmetic, not a behavioral deviation.

## Issues Encountered

None. TypeScript type-check passed on the first attempt. The `invoke` import and `extractErrMessage` helper were already available at module scope as specified in the plan's `<interfaces>` section.

## Known Stubs

None — the new useEffect is fully wired. The `save_state` Tauri command already exists (Phase 49 KBD-02, rewired to iOS path by plan 54-01).

## Threat Flags

None. The `visibilitychange` handler fires `save_state` which only persists the current in-memory `CalcState` snapshot — no attacker-controlled data path. This is covered by the plan's T-54-07 (accepted) disposition.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- PERSIST-02 frontend trigger is implemented; TypeScript passes
- Wave-merge gate `just gui-ci` should be run to confirm Vitest suite green
- On-device verification of the visibilitychange firing in WKWebView is the remaining gate (plan 54-03 D-54.4a)
- The 30s auto-save thread remains the safety net if `visibilitychange` does not fire on iOS (D-54.2a, RESEARCH Pitfall 4)

## Self-Check: PASSED

- `hp41-gui/src/App.tsx` modified: FOUND (git log e4cee0a — 1 file changed, 18 insertions)
- Commit e4cee0a: FOUND (`git log --oneline -3` confirms)
- `grep -c "visibilitychange" hp41-gui/src/App.tsx` == 2: VERIFIED
- `grep -c "invoke<void>('save_state')" hp41-gui/src/App.tsx` >= 1: VERIFIED (== 2)
- `grep -c "pagehide\|beforeunload" hp41-gui/src/App.tsx` == 0: VERIFIED
- `npx tsc --noEmit` exits 0: VERIFIED

---
*Phase: 54-ios-persistence-layer*
*Completed: 2026-06-02*
