---
phase: 56-app-lifecycle-clock
plan: 01
subsystem: ui
tags: [ios, tauri, react, wkwebview, lifecycle, clock, stopwatch, touch]

# Dependency graph
requires:
  - phase: 55-touch-ui-adaptation
    provides: touch UI, stopwatch keyboard mode, isIos flag, busyRef pattern
  - phase: 54-ios-persistence-layer
    provides: visibilitychange save_state listener (extended here with visible branch)
  - phase: 53-ios-scaffold-spike
    provides: just ios-build, tauri.ios.conf.json merge conventions, device-install loop
provides:
  - tauri.ios.conf.json with backgroundThrottling: throttle (WKWebView not fully suspended)
  - needsTickRef + gated visibilitychange visible branch firing one tick_time on foreground return
  - iOS touch R/S stopwatch keyboard-mode interception in handleClick (touch parity with handleKey)
affects: [57-signing-testflight-pipeline]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "needsTickRef: useRef synced by useEffect([needsTick]) — reads live predicate inside empty-deps listener without re-registering"
    - "tauri.ios.conf.json JSON Merge Patch: override must repeat complete app.windows[0] object because arrays are replaced wholesale (RFC 7396)"
    - "stopwatch keyboard mode touch parity: handleClick mirrors handleKey interception block for R/S/ENTER/any-tap mapping"

key-files:
  created:
    - hp41-gui/src-tauri/tauri.ios.conf.json
  modified:
    - hp41-gui/src/App.tsx

key-decisions:
  - "LIFE-01: backgroundThrottling key is camelCase with no Policy suffix — using backgroundThrottlingPolicy (the Rust TYPE name) would silently no-op; value is throttle (not suspend)"
  - "LIFE-02: resume tick is a single invoke(tick_time) gated on isIos && needsTickRef.current — NOT invoke(get_state) (D-11); fires only when clock_active || stopwatch_keyboard_mode"
  - "Scope addition: touch R/S stopwatch keyboard-mode gap was a Phase 41 pre-existing issue surfaced by device test; fixed inline as Rule 1 auto-fix; touch-only change, D-25.6 parity unaffected"
  - "On-device approval (iPhone 15 Pro, iOS 17+): clock resume shows correct time within one frame, stopwatch refreshes on resume, no spurious IPC when idle, R/S starts/stops stopwatch on touch"

patterns-established:
  - "ref-sync pattern for empty-deps listeners: declare useRef(initialValue) + useEffect([dep]) { ref.current = dep } — reuse wherever a listener closure needs a live value without re-registration"
  - "iOS platform config override: tauri.ios.conf.json MUST repeat all app.windows[0] fields from tauri.conf.json (title/width/height/resizable/decorations/visible) before adding iOS-only settings"

requirements-completed: [LIFE-01, LIFE-02]

# Metrics
duration: ~40min
completed: 2026-06-03
---

# Phase 56 Plan 01: App Lifecycle + Clock Summary

**WKWebView backgroundThrottling configured via tauri.ios.conf.json + needsTickRef-gated resume tick_time on visibilitychange, with iOS touch R/S stopwatch parity fix verified on iPhone 15 Pro**

## Performance

- **Duration:** ~40 min (Tasks 1-2 by prior executor; Task 3 device checkpoint + scope addition by continuation agent)
- **Started:** 2026-06-03
- **Completed:** 2026-06-03
- **Tasks:** 3 (2 auto + 1 checkpoint:human-verify approved) + 1 scope addition
- **Files modified:** 2 (1 created, 1 modified)

## Accomplishments

- LIFE-01: `tauri.ios.conf.json` created with `backgroundThrottling: "throttle"` — prevents WKWebView full suspension on brief backgrounding (iOS 17+); complete base window object repeated to avoid JSON Merge Patch array-replace footgun
- LIFE-02: `needsTickRef` + extended `visibilitychange` handler — fires one `tick_time` on foreground return, gated on `isIos && needsTickRef.current` (i.e., only when clock or stopwatch is active); desktop/macOS fire no spurious IPC
- Scope addition (Rule 1 bug fix): iOS touch R/S stopwatch keyboard-mode interception wired into `handleClick`, closing a Phase 41 touch-parity gap where the stopwatch never started on iOS because the keyboard-mode block existed only in the physical `handleKey` path
- On-device checkpoint APPROVED on iPhone 15 Pro (iOS 17+): clock resume correct within one frame, stopwatch display refreshes, brief-background timekeeping correct, no spurious IPC when idle

## Task Commits

Each task was committed atomically:

1. **Task 1: Add tauri.ios.conf.json with backgroundThrottling (LIFE-01)** - `65da485` (feat)
2. **Task 2: Extend visibilitychange with gated resume tick (LIFE-02)** - `a0f7d74` (feat)
3. **Task 3: On-device checkpoint** - checkpoint:human-verify APPROVED (no code commit)
4. **Scope Addition: iOS touch R/S stopwatch keyboard-mode fix** - `88452f2` (fix)

**Plan metadata:** (docs commit — see final commit)

## Files Created/Modified

- `hp41-gui/src-tauri/tauri.ios.conf.json` — iOS platform config override; sets `app.windows[0].backgroundThrottling: "throttle"` with all base window fields (title, width, height, resizable, decorations, visible) repeated to satisfy JSON Merge Patch RFC 7396 array-replace semantics
- `hp41-gui/src/App.tsx` — (1) `needsTickRef` useRef + `useEffect([needsTick])` sync; (2) `visibilitychange` extended with `visible` branch firing `invoke('tick_time')` gated on `isIos && needsTickRef.current && !busyRef.current`; (3) `handleClick` extended with stopwatch-keyboard-mode interception block (R/S→RUNSW/STOPSW, ENTER→STPW, other→sw_exit)

## Decisions Made

- **backgroundThrottling key spelling:** camelCase `backgroundThrottling` (JSON field name) vs `backgroundThrottlingPolicy` (Rust type name) — the Rust type name silently no-ops as Tauri ignores unknown JSON keys. Used `backgroundThrottling: "throttle"` (confirmed by Tauri source).
- **Array replacement guard:** tauri.ios.conf.json repeats every field from `tauri.conf.json` `app.windows[0]` because JSON Merge Patch (RFC 7396) replaces arrays wholesale — a patch with only `backgroundThrottling` would silently drop `title`/`width`/`height`/`visible:false`. Node assertion gates this in Task 1 verification.
- **needsTickRef pattern:** `useRef` initialized to `false` + `useEffect([needsTick])` assigning `needsTickRef.current = needsTick` — mirrors the established `busyRef`/`liveTickRef` pattern; allows the empty-deps `visibilitychange` listener to read the live predicate without re-registering on every 100ms tick.
- **Resume tick is tick_time not get_state:** D-11 prohibits polling `get_state`; `tick_time` is the correct command (advances time state, returns updated view).
- **Touch stopwatch mapping (scope addition):** R/S→RUNSW/STOPSW toggle (HP-41-canonical run/stop key), ENTER→STPW (split), any other tap→sw_exit. Backend clears `stopwatch_keyboard_mode` only on explicit `sw_exit` key_id — without the catch-all, touch users would be stranded in the mode if they tapped a non-R/S key.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] iOS touch R/S did nothing in stopwatch keyboard mode**
- **Found during:** Task 3 on-device checkpoint (iPhone 15 Pro, iOS 17+)
- **Issue:** Stopwatch keyboard-mode key interception existed only in the physical-keyboard path (`handleKey` ~L892); the touch path (`handleClick`) had no equivalent block. An on-screen R/S tap fell through to `invokeForKey` → `run_stop` (wrong op), so the stopwatch never started on iOS. Pre-existing Phase 41 touch-parity gap surfaced by this phase's device test.
- **Fix:** Added a `stopwatch_keyboard_mode` interception block in `handleClick` (37 lines) mirroring `handleKey`'s logic, with touch-appropriate key mapping: R/S→RUNSW/STOPSW toggle, ENTER→STPW split, any other tap→sw_exit (catch-all prevents stranding the user in the mode).
- **Files modified:** `hp41-gui/src/App.tsx`
- **Verification:** On-device (iPhone 15 Pro, iOS 17+): R/S now starts/stops the stopwatch on iOS touch. `npm run build` + `just gui-ci` (271 vitest + cargo) + `just ios-build` all passed.
- **Committed in:** `88452f2` (standalone fix commit)
- **Touch-only scope:** CLI has no touch path; D-25.6 CLI↔GUI parity is unaffected.

---

**Total deviations:** 1 auto-fixed (Rule 1 - Bug)
**Impact on plan:** Essential fix — without it the stopwatch was not operable on iOS touch. Touch-only; no impact to desktop or macOS menu-bar behavior.

## Issues Encountered

- **tauri.ios.conf.json JSON key spelling:** Initial research in CONTEXT.md used the Rust type name `backgroundThrottlingPolicy`; the actual JSON field is `backgroundThrottling` (no Policy suffix). This was captured as a `critical_research_findings` correction in the plan before execution, and the node assertion in Task 1 verification gates the correct value.
- **Stopwatch display under-counts on resume:** Expected and documented (D-56.3) — the stopwatch's `Instant`-based elapsed time does not account for CPU-sleep time during background; foreground-only live updates are accepted as HP-41CX-faithful behavior.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Phase 56 complete: `backgroundThrottling` configured, clock/stopwatch resume correct, touch parity verified on device.
- Phase 57 (Signing + TestFlight Pipeline) is now unblocked: distribution cert + provisioning profile + `PrivacyInfo.xcprivacy` + app icon + `ci-ios.yml`.
- No new blockers. All Phase 56 requirements (LIFE-01, LIFE-02) confirmed on a real iPhone (iOS 17+).

---

## Self-Check: PASSED

- `hp41-gui/src-tauri/tauri.ios.conf.json` exists: FOUND (committed in 65da485)
- `hp41-gui/src/App.tsx` needsTickRef + visible branch: FOUND (committed in a0f7d74)
- Touch fix in App.tsx: FOUND (committed in 88452f2)
- All task commits present: 65da485, a0f7d74, 88452f2 — all verified in git log
- On-device checkpoint: APPROVED (iPhone 15 Pro, iOS 17+) — clock + stopwatch resume correct, no spurious IPC, touch R/S verified

---
*Phase: 56-app-lifecycle-clock*
*Completed: 2026-06-03*
