# Phase 56: App Lifecycle + Clock - Context

**Gathered:** 2026-06-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Two small iOS app-lifecycle polish items so the running app behaves correctly
across brief background → foreground transitions:

1. **LIFE-01** — Configure `backgroundThrottlingPolicy` so the WKWebView is **not
   fully suspended** on *brief* backgrounding (iOS 17+). The setting lives in
   `tauri.ios.conf.json` and the behavior is verified on a real device.
2. **LIFE-02** — On return to foreground, the clock/stopwatch display refreshes
   **immediately** (forced `tick_time` on become-active) so no stale time is
   visible even for one frame. Foreground-only live updates are **accepted as
   HP-41CX-faithful**.

**iOS-gated form-factor/lifecycle work only.** No `hp41-core` changes, no engine
work, no new calculator functions, no new Tauri command (the `tick_time` and
`is_ios` commands already exist). Every change is in `hp41-gui` (one React
lifecycle-listener extension + one platform config file) and must not regress the
desktop or macOS menu-bar apps.

**Out of scope (own phase):** signing / `PrivacyInfo.xcprivacy` / app icon /
launch screen / `ci-ios.yml` / TestFlight → Phase 57. Interrupting control alarms
remain DEFERRED (data model only; needs call-stack re-entrancy — unchanged).

</domain>

<decisions>
## Implementation Decisions

### Foreground-refresh trigger
- **D-56.1:** Fire the forced `tick_time` by **extending the EXISTING
  `visibilitychange` listener** (`App.tsx:984`, added in Phase 54 for the
  `hidden` → `save_state` background-save). Add a `document.visibilityState ===
  'visible'` branch alongside the existing `'hidden'` branch — one lifecycle hook
  handles both directions. **Reject** a separate `focus` / native become-active
  listener: `visibilitychange` is the proven WKWebView-reliable event already in
  the codebase, and a second hook adds surface for double-firing. Keep the
  listener's empty-deps registration (stable, registered once on mount); if the
  `visible` branch needs current state, read it via a ref (mirror the
  `busyRef`/`liveTickRef` pattern) rather than adding deps that re-register the
  listener every tick.

### Background throttling policy
- **D-56.2:** `backgroundThrottlingPolicy: "throttle"` in `tauri.ios.conf.json`
  (iOS 17+). Battery-friendly — reduces rather than fully suspends background
  timer activity. Combined with D-56.3's forced tick on resume, the clock is
  correct within one frame regardless, so the more aggressive `"disabled"` (keep
  the webview fully live in background) is **not** needed and would cost battery.
  iOS ≤16 timers pause in background — **accepted as a known limitation** (matches
  STATE.md pre-research and LIFE-02's "foreground-only live updates accepted").

### Refresh scope / gating on resume
- **D-56.3:** On `visible`, fire **one gated `tick_time`** — only when
  `needsTick` is true (`calcState.clock_active || calcState.stopwatch_keyboard_mode`,
  the same predicate that drives the live-tick interval at `App.tsx:489`). When
  neither is active there is nothing time-dependent on screen, so do nothing — no
  IPC on resume. **Not** a full `get_state` resync (only the clock/stopwatch
  display is time-sensitive). A single `tick_time` is sufficient: the clock and
  stopwatch both derive from `SystemTime::now()` + `time_offset_secs` in
  `hp41-core`, so one forced tick fully catches up all elapsed background time.
  Stopwatch freeze-on-save policy is **unchanged** (v3.2 decision).

### On-device verification
- **D-56.4:** Verify with **inline human checkpoints** (the proven Phase 53/54/55
  pattern): the executor automates all code, then pauses at explicit checkpoint
  tasks with click-by-click on-device steps for the criteria only observable on
  real hardware. LIFE-01 #2 ("not fully suspending on brief backgrounding") and
  LIFE-02 ("clock correct within one frame on resume") are **not** faithfully
  reproducible in the Simulator. Checkpoint script: start the clock, background
  the app (Home button) for ~10s, return → confirm the displayed time jumps to
  the correct current time within one frame (no visible stale value); repeat with
  a running stopwatch. Config/wiring may be smoke-checked in the Simulator first,
  but the authoritative pass is on a real iPhone.

### Claude's Discretion
- Exact placement/shape of the `'visible'` branch inside the existing
  `visibilitychange` handler, and the ref plumbing for reading `needsTick`
  current value, are left to research/planning provided they honor D-56.1
  (single extended listener, no new lifecycle hook, no per-tick re-registration).
- The precise structure/merge mechanics of `tauri.ios.conf.json` (Tauri merges
  `tauri.<platform>.conf.json` over `tauri.conf.json`) and whether any companion
  Info.plist key is required for `backgroundThrottlingPolicy` are research/plan
  questions — D-56.2 only locks the policy **value** (`"throttle"`).
- Plan decomposition and wave parallelization → `gsd-planner` chooses. (For
  reference only: this phase is small enough it may be a single plan with an
  on-device checkpoint task; the planner is free to split or not.)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & state
- `.planning/REQUIREMENTS.md` §"App Lifecycle" — **LIFE-01** (backgroundThrottlingPolicy)
  and **LIFE-02** (forced `tick_time` on foreground; foreground-only live updates
  accepted) — this phase's two requirements.
- `.planning/ROADMAP.md` §"Phase 56: App Lifecycle + Clock" — goal + 2 success criteria.
- `.planning/STATE.md` §"Accumulated Context" → "Decisions (pre-resolved from
  research)": **Background throttling** (`tauri.ios.conf.json`,
  `backgroundThrottlingPolicy: "throttle"`, iOS 17+; iOS ≤16 pause accepted) and
  **Clock on resume** (become-active/focus → immediate `tick_time`; stopwatch
  freeze-on-save unchanged). Also pitfall **P-iOS-27** (background suspension
  freezes clock UI → become-active must force `tick_time`).

### Established iOS platform precedent (Phase 53–55)
- `.planning/phases/55-touch-ui-adaptation/55-CONTEXT.md` — **D-55.1** establishes
  the `is_ios` Tauri command + `isIos` frontend flag for gating iOS-only behavior
  (reuse for any iOS gating here); **D-55.4** establishes the inline on-device
  checkpoint verification pattern reused by D-56.4.
- `.planning/phases/54-ios-persistence-layer/54-CONTEXT.md` — Phase 54
  `visibilitychange` background-save listener (D-54.2*); D-56.1 extends this exact
  listener.

### Code touchpoints (read before editing)
- `hp41-gui/src/App.tsx:984` — existing `visibilitychange` listener (the file to
  extend per D-56.1).
- `hp41-gui/src/App.tsx:487-507` — `needsTick` predicate + live-tick `setInterval`
  loop calling `invoke('tick_time')` (the gating predicate reused by D-56.3).
- `hp41-gui/src-tauri/tauri.conf.json` — base Tauri config; `tauri.ios.conf.json`
  is the new sibling platform-override file for D-56.2 (does not exist yet).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`visibilitychange` listener** (`App.tsx:984`): already registered, empty-deps,
  WKWebView-reliable. D-56.1 adds a `'visible'` branch — no new listener.
- **`needsTick` boolean** (`App.tsx:487`): `clock_active || stopwatch_keyboard_mode`.
  D-56.3 reuses it as the gate for the forced resume tick.
- **`tick_time` IPC command** (`hp41-gui/src-tauri/src/commands.rs`, permission
  `permissions/tick-time.toml`): the forced refresh call. No new command needed.
- **`is_ios` command + `isIos` flag** (D-55.1): available for iOS gating.

### Established Patterns
- **No-polling (D-11):** the live display advances via `tick_time` on an interval,
  never `get_state` polling. The resume refresh is a single `tick_time`, consistent
  with this rule (D-56.3 — not a `get_state` resync).
- **Ref-for-current-value in stable listeners:** `busyRef`/`liveTickRef` show the
  pattern for reading mutable current state inside an empty-deps listener without
  re-registering — apply if the `'visible'` branch needs `needsTick`'s live value.
- **Platform config override:** macOS menu-bar mode is applied at runtime in
  `tray.rs` (NOT `tauri.conf.json`) so other platforms aren't affected; the iOS
  throttling setting instead uses the `tauri.ios.conf.json` platform-override file
  so desktop/macOS configs are untouched.
- **Inline on-device checkpoint verification (D-53/54/55):** automate code, pause
  at explicit device-only checkpoint tasks.

### Integration Points
- Frontend: one new branch in the `App.tsx` `visibilitychange` handler.
- Config: one new `hp41-gui/src-tauri/tauri.ios.conf.json` carrying
  `backgroundThrottlingPolicy`.
- No `hp41-core` changes; no IPC contract changes.

</code_context>

<specifics>
## Specific Ideas

- Resume-correctness bar is strict: **"no stale time visible even for one frame"**
  (ROADMAP Phase 56 criterion #1). The forced tick must run on the `visible`
  transition itself, not on a subsequent interval cycle.
- Stopwatch behavior on resume: a single forced `tick_time` must visibly catch the
  running stopwatch up to true elapsed wall-clock time (it derives from
  `SystemTime::now()`), not just resume ticking from where it paused.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope. (Signing/TestFlight → Phase 57;
interrupting control alarms remain DEFERRED per existing project decision.)

</deferred>

---

*Phase: 56-App Lifecycle + Clock*
*Context gathered: 2026-06-03*
