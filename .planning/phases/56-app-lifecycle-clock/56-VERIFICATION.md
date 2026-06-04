---
phase: 56-app-lifecycle-clock
verified: 2026-06-03T00:00:00Z
status: passed
score: 4/4 must-haves verified
overrides_applied: 0
human_verification_resolved: "2026-06-03 — CR-01-fixed build (96f4a7c) reinstalled on iPhone 15 Pro (iOS 17+) via just ios-build + xcrun devicectl; user approved: clock + stopwatch resume correct within one render frame, on-screen R/S drives the stopwatch on touch, no spurious IPC/toast when nothing time-sensitive is active. Both human_verification items below are now satisfied."
human_verification:
  - test: "On a real iPhone (iOS 17+), start the clock (clock_active = true), note the displayed time, press Home to background for ~10 seconds, return to the app."
    expected: "The displayed time immediately shows the correct current wall-clock time within one render frame — no stale value visible even momentarily."
    why_human: "WKWebView backgroundThrottling behavior and one-frame clock-resume timing cannot be verified programmatically; Simulator is not faithful for background-throttling policy; on-device behavior was approved pre-CR-01-fix, and the post-fix build has not been reinstalled on the device."
  - test: "With the fixed build installed on device (just ios-build + xcrun devicectl reinstall), repeat the clock resume check and confirm LIFE-02 still passes."
    expected: "Clock and stopwatch displays show correct time within one frame on foreground return. No spurious error toast when nothing time-sensitive is active."
    why_human: "The CR-01 fix (96f4a7c) changes App.tsx runtime behavior on iOS. The pre-fix build was device-approved but the post-fix build has not yet been reinstalled (device was disconnected). The fix is covered by gui-ci (tsc + 273 vitest), and only strengthens the approved behavior — but the authoritative on-device confirmation with the corrected code is still pending."
---

# Phase 56: App Lifecycle + Clock — Verification Report

**Phase Goal:** The app's WKWebView is not fully suspended on brief backgrounding, and the clock/stopwatch display is immediately correct when the user returns to the app.
**Verified:** 2026-06-03
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | On brief background→foreground (not kill/relaunch), the clock display updates to current wall-clock time within one render frame — no stale value visible even momentarily (LIFE-02) | ✓ VERIFIED (code) / ? HUMAN (device re-confirm) | `isIosRef.current && needsTickRef.current` gate at App.tsx:1078; `invoke('tick_time').then(view => { setCalcState(view); setErrorMessage(null); })` at lines 1082-1083; CR-01 fix confirmed in commit 96f4a7c |
| 2 | The WKWebView is not fully suspended on brief backgrounding on iOS 17+ — backgroundThrottling: throttle is present in tauri.ios.conf.json and verified on a real device (LIFE-01) | ✓ VERIFIED (config) / ? HUMAN (device with fixed build) | `"backgroundThrottling": "throttle"` at tauri.ios.conf.json:12; node assertion confirms all 6 base window fields preserved; pre-fix build approved on iPhone 15 Pro (iOS 17+) |
| 3 | The forced resume tick fires only on iOS and only when needsTick is true (clock_active OR stopwatch_keyboard_mode); desktop and macOS fire no extra IPC on tab/window focus changes | ✓ VERIFIED | `isIosRef.current && needsTickRef.current` double gate; `isIosRef` synced from `useEffect([isIos])` at App.tsx:545-547; on desktop isIosRef.current stays false; visibilitychange deps remain `[]` (no re-registration on every tick) |
| 4 | Desktop and macOS configs and behavior are unchanged (iOS-only override file; gated frontend branch) | ✓ VERIFIED | tauri.ios.conf.json is a new file (no modification to tauri.conf.json); git diff confirms tauri.conf.json unmodified in 65da485; isIos gate prevents any IPC change on macOS/desktop |

**Score:** 4/4 truths verified (code-level); 2 require device re-confirmation with post-CR-01 build

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-gui/src-tauri/tauri.ios.conf.json` | iOS platform config override carrying `app.windows[0].backgroundThrottling: "throttle"` with COMPLETE base window object | ✓ VERIFIED | File exists, 16 lines, valid JSON. Node assertion: "OK: all base fields preserved + backgroundThrottling=throttle". All 6 base fields (title, width, height, resizable, decorations, visible) present with matching values. Committed in 65da485. |
| `hp41-gui/src/App.tsx` | `needsTickRef` + `isIosRef` + extended visibilitychange handler with gated 'visible' branch firing `tick_time` | ✓ VERIFIED | `needsTickRef = useRef(false)` at line 264; `isIosRef = useRef(false)` at line 270 (CR-01 fix); `needsTickRef.current = needsTick` synced at lines 522-524; `isIosRef.current = isIos` synced at lines 545-547; visible branch at line 1078 reads `isIosRef.current && needsTickRef.current`. Committed in a0f7d74 + 96f4a7c. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| App.tsx visibilitychange 'visible' branch | `tick_time` IPC command | `invoke('tick_time')` gated on `isIosRef.current && needsTickRef.current` | ✓ WIRED | Line 1078: gate check; line 1082: `invoke<CalcStateView>('tick_time')`; line 1083: `.then(view => { setCalcState(view); setErrorMessage(null); })`. No `get_state` (D-11 preserved). |
| `hp41-gui/src-tauri/tauri.ios.conf.json` | WKWebView `inactiveSchedulingPolicy` | Tauri iOS platform config merge (JSON Merge Patch RFC 7396) | ✓ WIRED | `"backgroundThrottling": "throttle"` inside `app.windows[0]`. Array replaced wholesale (RFC 7396) — mitigated by repeating all 6 base fields. Node assertion verified. |
| `isIos` useState → `isIosRef.current` | visibilitychange listener (empty deps) | `useEffect([isIos]) { isIosRef.current = isIos; }` | ✓ WIRED | Lines 543-547. CR-01 fix: the async-resolved `isIos` value now flows into the ref before any foreground-return event. Stale-closure trap eliminated. |
| `needsTick` boolean → `needsTickRef.current` | visibilitychange listener (empty deps) | `useEffect([needsTick]) { needsTickRef.current = needsTick; }` | ✓ WIRED | Lines 519-524. Mirrors the established busyRef/liveTickRef pattern. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| App.tsx visibilitychange visible branch | `view` (CalcStateView) | `invoke<CalcStateView>('tick_time')` — existing Tauri command, returns real calculator state with updated time | Yes — tick_time is an established IPC command that advances time state in hp41-core and returns the live CalcStateView | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `isIosRef` declared and synced | `grep -n "isIosRef" hp41-gui/src/App.tsx` | Lines 264, 270, 540, 546, 1065, 1078, 1089 — declaration, sync, and usage all present | ✓ PASS |
| Visibilitychange deps remain empty | Line 1089 of App.tsx | `}, []); // empty deps: listener reads live values via refs (needsTickRef, isIosRef, busyRef)` | ✓ PASS |
| No `get_state` in visibility handler | lines 1071-1089 | `tick_time` only; no `get_state` invocation | ✓ PASS |
| tauri.ios.conf.json node assertion | `node -e "..."` | "OK: all base fields preserved + backgroundThrottling=throttle" | ✓ PASS |
| No stale `isIos` in visible branch | line 1078 | `isIosRef.current` (not `isIos`) | ✓ PASS |
| `setErrorMessage(null)` in resume branch | line 1083 | `.then(view => { setCalcState(view); setErrorMessage(null); })` — WR-01 also fixed | ✓ PASS |
| Touch stopwatch R/S block in handleClick | lines 697-718 | `stopwatch_keyboard_mode` interception with R/S→RUNSW/STOPSW, ENTER→STPW, other→sw_exit | ✓ PASS (bonus, not a LIFE gate) |
| On-device device verification (post-CR-01 build) | reinstall + manual test | Build not reinstalled after 96f4a7c; device disconnected | ? SKIP → routes to human verification |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| LIFE-01 | 56-01-PLAN.md | `backgroundThrottlingPolicy` configured so WKWebView is not fully suspended on brief backgrounding (iOS 17+) | ✓ SATISFIED (config level) | `backgroundThrottling: "throttle"` in tauri.ios.conf.json; JSON Merge Patch footgun mitigated; pre-fix on-device approved |
| LIFE-02 | 56-01-PLAN.md | Clock/stopwatch display refreshes immediately on return to foreground (forced `tick_time` on become-active) | ✓ SATISFIED (code level) | `isIosRef.current && needsTickRef.current` gate; `invoke('tick_time')` in visible branch; CR-01 stale-closure fix applied |

**Note on REQUIREMENTS.md status column:** LIFE-01 and LIFE-02 still show "Pending" in the traceability table (last updated 2026-05-29). This is a documentation-only gap; the implementation is complete. REQUIREMENTS.md checkboxes at lines 43-44 are also unchecked. These should be updated post-verification — not a blocker.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None | — | No TBD/FIXME/XXX/placeholder markers found in either modified file | — | — |

### Human Verification Required

#### 1. Post-CR-01 On-Device Clock Resume (LIFE-02 final confirmation)

**Test:** Reinstall the post-CR-01 build on iPhone 15 Pro (iOS 17+): run `just ios-build`, then install + launch via `xcrun devicectl --device <id>` (device unlocked). Start the clock (`clock_active = true`), note the displayed time, press Home to background for ~10 seconds, return to the app.

**Expected:** The displayed time immediately jumps to the correct current wall-clock time within one render frame. No stale time visible even momentarily.

**Why human:** The CR-01 fix (commit 96f4a7c, `isIosRef` replacing stale `isIos` closure) changes the runtime behavior of the LIFE-02 resume branch on iOS. The pre-fix build received on-device approval on iPhone 15 Pro (iOS 17+), but the post-fix build has not been reinstalled (device was disconnected after the initial checkpoint). The fix is mechanically correct and covered by gui-ci (tsc + 273 vitest), but the authoritative one-frame-resume behavioral check must be re-confirmed on device with the corrected code.

#### 2. Post-CR-01 Stopwatch + Idle Spot-Check

**Test:** Same build. Start a running stopwatch (stopwatch keyboard mode), background ~10 seconds, return. Then, with neither clock nor stopwatch active, background and return.

**Expected:** Stopwatch display refreshes on resume (under-count of CPU-sleep time accepted as HP-41CX-faithful). With nothing time-sensitive active: no error toast, no spurious IPC.

**Why human:** Behavioral — requires observing the actual display refresh and absence of spurious IPC on device.

---

## Gaps Summary

No blocking gaps. All code-level must-haves are VERIFIED:

- LIFE-01: `tauri.ios.conf.json` is present, valid, and carries the correct `backgroundThrottling: "throttle"` with all base window fields preserved (node assertion green).
- LIFE-02: The stale-closure CR-01 bug is fixed — the visible branch now correctly reads `isIosRef.current` (not a stale `isIos` closure). The `needsTickRef` and `isIosRef` sync pattern is properly wired. The `busyRef` guard is present. No `get_state` call. `setErrorMessage(null)` is included (WR-01 from code review also resolved). Empty deps confirmed.
- Desktop/macOS isolation: confirmed by `isIosRef.current` gate and absence of any modification to `tauri.conf.json`.

The two human_verification items are behavioral re-confirmations of already-approved behavior with the post-CR-01 build. They do not represent unknown risks — the fix is a mechanical ref-sync that only _enables_ LIFE-02 to fire correctly on iOS; it cannot regress LIFE-01 or desktop behavior.

---

_Verified: 2026-06-03_
_Verifier: Claude (gsd-verifier)_
