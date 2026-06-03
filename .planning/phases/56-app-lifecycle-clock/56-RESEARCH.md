# Phase 56: App Lifecycle + Clock — Research

**Researched:** 2026-06-03
**Domain:** Tauri v2 iOS platform config, WKWebView lifecycle events, React stable-listener ref patterns
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-56.1** — Extend the EXISTING `visibilitychange` listener (`App.tsx:997–1008`, Phase 54).
  Add a `document.visibilityState === 'visible'` branch alongside the existing `'hidden'` branch.
  Reject a separate `focus` / native become-active listener. Keep empty-deps registration;
  read `needsTick` via a ref (mirror `busyRef`/`liveTickRef` pattern).

- **D-56.2** — `backgroundThrottlingPolicy: "throttle"` in `tauri.ios.conf.json` (iOS 17+).
  Battery-friendly throttle (not `"disabled"`) combined with D-56.3 forced tick on resume.
  iOS ≤16 timers pause in background — accepted as known limitation.

- **D-56.3** — On `visible`, fire ONE gated `tick_time` — only when `needsTick` is true
  (`calcState.clock_active || calcState.stopwatch_keyboard_mode`). Not a full `get_state`
  resync; a single `tick_time` is sufficient. Stopwatch freeze-on-save policy unchanged
  (v3.2 decision).

- **D-56.4** — Verify with inline human checkpoints (Phase 53/54/55 pattern). LIFE-01 and
  LIFE-02 are not faithfully reproducible in the Simulator; authoritative pass is on a real iPhone.

### Claude's Discretion

- Exact placement/shape of the `'visible'` branch and the ref plumbing for reading
  `needsTick`'s live value inside the existing empty-deps listener.
- The precise structure/merge mechanics of `tauri.ios.conf.json` and whether any companion
  Info.plist key is required for `backgroundThrottlingPolicy` — D-56.2 only locks the value.
- Plan decomposition and wave parallelization.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope. Signing/TestFlight → Phase 57.
Interrupting control alarms remain DEFERRED (data model only).

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| LIFE-01 | `backgroundThrottlingPolicy` configured so WKWebView is not fully suspended on brief backgrounding (iOS 17+) | `tauri.ios.conf.json` with `app.windows[0].backgroundThrottling: "throttle"` (§Standard Stack, §Architecture Patterns) |
| LIFE-02 | Clock/stopwatch display refreshes immediately on foreground return (forced `tick_time` on become-active); foreground-only live updates accepted | `visibilitychange` `visible` branch + `needsTickRef` (§Architecture Patterns, §Code Examples) |

</phase_requirements>

---

## Summary

Phase 56 is two small iOS lifecycle polish changes: a config file addition (LIFE-01) and a
four-line React listener extension (LIFE-02). Both deliverables are already well-constrained
by CONTEXT.md D-56.1–D-56.4.

The primary research questions were (a) the exact JSON key name and merge mechanics for the
Tauri iOS platform config, and (b) whether `visibilitychange visible` is reliable on WKWebView
for foreground-resume detection, and (c) whether a single `tick_time` fully corrects the
clock/stopwatch display without any async risk.

Key findings: the JSON key is **`backgroundThrottling`** (camelCase, no `Policy` suffix);
the iOS platform-override file uses JSON Merge Patch (RFC 7396), which means **arrays are
replaced in their entirety** when present in the override — the full `app.windows` entry
must be repeated in `tauri.ios.conf.json`. No companion Info.plist key is required. The
`visibilitychange visible` direction is the symmetric counterpart to the already-working
Phase 54 `hidden` direction and is reliable for app-lifecycle (home button → return)
transitions on WKWebView. A single `tick_time` corrects the clock immediately because it
calls `SystemTime::now()` inside Rust. The stopwatch `Instant`-based elapsed time does NOT
advance during iOS CPU sleep, but this is acceptable: freeze-on-save (D-38.6 / `migrate_after_load()`)
is the v3.2 design; a background-killed app will show the frozen stopwatch time on relaunch,
which is HP-41CX-faithful behavior.

**Primary recommendation:** Write `tauri.ios.conf.json` with the complete window entry
(not just `backgroundThrottling`), and add a 4-line `visible` branch + a `needsTickRef`
to the existing `visibilitychange` handler. No Rust changes needed.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Background throttling policy | iOS platform config | — | WKWebView behaviour is configured at the Tauri conf layer; no frontend or Rust change |
| Foreground-resume clock refresh | Frontend (React) | — | `tick_time` is already the live-tick mechanism; extending the existing listener keeps all display refresh logic in one place |
| `tick_time` IPC command | API / Backend (Tauri command) | — | Already exists; command is read-only, no state change, returns `CalcStateView` |
| iOS-gating | Frontend (`isIos` flag) | — | `is_ios` command (D-55.1) already wired; the `visible` branch should be gated with `isIos` to avoid spurious ticks on desktop |

---

## Standard Stack

No new packages are installed in this phase. All required pieces already exist.

### Existing Assets Reused

| Asset | Location | Purpose |
|-------|----------|---------|
| `tick_time` Tauri command | `commands.rs:333` | Forced clock/stopwatch refresh on resume |
| `is_ios` Tauri command | `commands.rs:570` | Gate iOS-only behaviour in frontend |
| `allow-tick-time` permission | `permissions/tick-time.toml` | Already granted in capabilities |
| `allow-is-ios` permission | `permissions/is-ios.toml` | Already granted in capabilities |
| `visibilitychange` listener | `App.tsx:997–1008` | Extended (not replaced) for LIFE-02 |
| `busyRef` / `liveTickRef` | `App.tsx:256,259` | Pattern reference for `needsTickRef` |

## Package Legitimacy Audit

No external packages are installed in this phase. Section not applicable.

---

## Architecture Patterns

### System Architecture Diagram

```
iOS Home button press
       │
       ▼
document.visibilitychange fires: state = "hidden"
       ├──► [existing] invoke("save_state")      [Phase 54, unchanged]
       └──► [LIFE-01] iOS throttles WKWebView timers (not full suspend)
                      ← controlled by tauri.ios.conf.json backgroundThrottling: "throttle"

User returns to app
       │
       ▼
document.visibilitychange fires: state = "visible"
       │
       ▼
       [LIFE-02] if (isIos && needsTickRef.current)
                       │
                       ▼
               invoke("tick_time")
                       │
                       ▼
             Rust: SystemTime::now() + time_offset_secs
                       │
                       ▼
                setCalcState(view)   ← display updated in same React render cycle
```

### Recommended Project Structure

Only two files change:

```
hp41-gui/src-tauri/
└── tauri.ios.conf.json          ← NEW: platform override with backgroundThrottling

hp41-gui/src/
└── App.tsx                      ← EXTEND: visibilitychange handler + needsTickRef
```

### Pattern 1: Tauri iOS Platform Override File

**What:** A sibling JSON file to `tauri.conf.json` that Tauri CLI auto-merges using
JSON Merge Patch (RFC 7396) when building for the `ios` platform target.

**When to use:** Whenever a config key should apply to iOS only (and not regress
desktop/macOS). The file lives at `hp41-gui/src-tauri/tauri.ios.conf.json`.

**CRITICAL: JSON Merge Patch array replacement.** RFC 7396 replaces arrays entirely —
it cannot patch individual array elements. Because `tauri.conf.json` declares
`app.windows` as an array, setting any field within `app.windows` in the iOS override
requires repeating the ENTIRE array entry with all existing fields plus the new one.

The base `tauri.conf.json` `app.windows[0]` has:
```json
{
  "title": "HP-41 Calculator",
  "width": 440,
  "height": 1020,
  "resizable": false,
  "decorations": true,
  "visible": false
}
```

The correct `tauri.ios.conf.json` therefore must be: [VERIFIED: v2.tauri.app/reference/config, v2.tauri.app/develop/configuration-files]

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "app": {
    "windows": [
      {
        "title": "HP-41 Calculator",
        "width": 440,
        "height": 1020,
        "resizable": false,
        "decorations": true,
        "visible": false,
        "backgroundThrottling": "throttle"
      }
    ]
  }
}
```

**Key names (verified):**
- JSON key: `backgroundThrottling` (camelCase, NOT `backgroundThrottlingPolicy`) [VERIFIED: v2.tauri.app/reference/config]
- Valid values: `"disabled"` | `"throttle"` | `"suspend"` (default is `"suspend"` = full suspension) [VERIFIED: Context7 /websites/v2_tauri_app]
- Platform support: iOS 17.0+, macOS 14.0+; unsupported on Linux / Windows / Android [VERIFIED: Context7 /websites/v2_tauri_app]
- Introduced in Tauri v2.3.0; project uses v2.11 — compatible [VERIFIED: Context7 /websites/v2_tauri_app]

**Info.plist requirement:** None. `backgroundThrottling` maps to `WKPreferences.inactiveSchedulingPolicy`
— a runtime API call, no Info.plist key or UIBackgroundModes entry is required. [MEDIUM confidence —
no official source explicitly states "no Info.plist required", but (a) Tauri documentation lists no
Info.plist companion, (b) the commit that introduced the feature shows only `WindowConfig` changes
with no Info.plist modifications, and (c) the WKPreferences API is a pure code-path setting.]

### Pattern 2: Ref-for-Current-Value in a Stable Empty-Deps Listener

**What:** To read `needsTick`'s live value inside the existing `visibilitychange` handler
(which has `[]` deps and is registered once), add a `needsTickRef` that mirrors `needsTick`
and is kept in sync via a dedicated `useEffect`. The closure captures the ref object (stable
identity) and reads `.current` at call time.

**When to use:** Any React stable listener that needs current state without re-registering.
Pattern is already established in this codebase (`busyRef`, `liveTickRef`).

**Example:** [ASSUMED — pattern matches established codebase convention; exact placement is Claude's Discretion]

```typescript
// Near the other useRef declarations (~App.tsx:259)
const needsTickRef = useRef(false);

// Keep needsTickRef in sync with needsTick (near the needsTick declaration ~App.tsx:487)
useEffect(() => {
  needsTickRef.current = needsTick;
}, [needsTick]);

// In the existing visibilitychange handler (~App.tsx:997–1008) — extend only:
const handleVisibilityChange = () => {
  if (document.visibilityState === 'hidden') {
    void invoke<void>('save_state').catch((err: unknown) => {
      console.warn('background save failed:', extractErrMessage(err));
    });
  } else if (document.visibilityState === 'visible' && isIos && needsTickRef.current) {
    // D-56.1/D-56.3: force a single tick_time on foreground return so the
    // clock/stopwatch display is correct within one frame. Only when needsTick
    // is true (clock_active || stopwatch_keyboard_mode) to avoid IPC on resume
    // when nothing time-sensitive is on screen.
    if (!busyRef.current) {
      invoke<CalcStateView>('tick_time')
        .then(view => { setCalcState(view); setErrorMessage(null); })
        .catch(err => showToast(extractErrMessage(err)));
    }
  }
};
```

**Note on `isIos` gating:** `isIos` is React state (not a ref), but it is set once on mount
and never changes during the app's lifetime. It is safe to close over in an empty-deps
handler because it will always equal its mount-time value. This is consistent with how `isMacos`
is used elsewhere in the codebase. If the linter flags the stale closure, promote `isIos`
to an `isIosRef` following the same pattern.

### Anti-Patterns to Avoid

- **Putting `backgroundThrottling` at the top level of `tauri.ios.conf.json`** instead of under
  `app.windows[0]`: the key lives in `WindowConfig`, not `AppConfig` or the root config.

- **Repeating only `backgroundThrottling` in `app.windows[0]` without other fields:** JSON Merge
  Patch replaces the entire `windows` array. An override entry that only sets `backgroundThrottling`
  would produce a window with that key and no other properties — the window would lose its title,
  size, and visibility settings.

- **Adding `needsTick` to the `visibilitychange` listener's deps array:** This would re-register
  the listener on every tick (every 100 ms when clock is active), creating churn and a
  registration/deregistration race. Use `needsTickRef` instead.

- **Using `document.visibilityState` for the `visible` branch without the `isIos` gate:** The
  `visibilitychange` listener fires on desktop too (tab switching, window minimize). Without the
  `isIos` gate, the forced tick would fire unnecessarily on every desktop tab switch. This does
  not regress the display (one extra tick is harmless), but it violates the "no IPC unless needed"
  discipline of D-11 and the phase boundary ("iOS-only").

- **Calling `invoke('get_state')` instead of `invoke('tick_time')` on resume:** A full
  `get_state` resync is not needed — only the time fields change during a background interval.
  `tick_time` is the correct mechanism per D-56.3 and D-11.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| iOS platform-specific config | A runtime `cfg!(target_os = "ios")` check to set WKWebView preferences from Rust | `tauri.ios.conf.json` platform override | Tauri has a first-class platform config file; runtime code for a static configuration is unnecessary indirection |
| Reading live React state in a stable listener | Passing `needsTick` directly or adding it to deps | `needsTickRef` with a sync effect | The ref pattern is already the codebase standard (`busyRef`, `liveTickRef`); deps modification causes listener re-registration |
| Foreground-resume time synchronisation | Polling `get_state` on an interval on resume | Single `invoke('tick_time')` | `tick_time` calls `SystemTime::now()` inside Rust — one call is sufficient; polling violates D-11 |

**Key insight:** Both deliverables fit entirely within existing extension points. No new Tauri
commands, no new plugins, no new Rust code.

---

## Runtime State Inventory

Phase 56 is a greenfield-style addition (new config file + listener extension). No rename or
migration is involved. Section not applicable.

---

## Common Pitfalls

### Pitfall 1: Wrong JSON key name — `backgroundThrottlingPolicy` vs `backgroundThrottling`

**What goes wrong:** The config key that appears in CONTEXT.md, STATE.md, and REQUIREMENTS.md
is called `backgroundThrottlingPolicy`, but the actual Tauri v2 `WindowConfig` JSON key is
`backgroundThrottling` (no `Policy` suffix). Using the wrong name silently produces a config
with no effect (Tauri ignores unknown keys).

**Why it happens:** The JavaScript API namespace uses `BackgroundThrottlingPolicy` as the
_type name_; the JSON config field is `backgroundThrottling`. The distinction is easy to miss.

**How to avoid:** Copy the key from the verified schema reference:
`app.windows[0].backgroundThrottling` = `"throttle"`.

**Warning signs:** `just ios-build` succeeds but background behaviour is unchanged on device.

### Pitfall 2: JSON Merge Patch replaces the entire `app.windows` array

**What goes wrong:** A `tauri.ios.conf.json` that contains `"app": { "windows": [{ "backgroundThrottling": "throttle" }] }` produces an iOS window that has only `backgroundThrottling` set and all other window properties missing. The window may open with default dimensions (not the 440×1020 portrait size) or fail to start.

**Why it happens:** JSON Merge Patch (RFC 7396) is not a deep merge — arrays are replaced
in their entirety, not merged element-by-element. This is documented Tauri behaviour.

**How to avoid:** Copy the full `app.windows[0]` entry from `tauri.conf.json` verbatim and
add `"backgroundThrottling": "throttle"` as a new field.

**Warning signs:** The iOS build starts but the window shows at wrong dimensions, or `visible:false` is lost causing a flash of blank window on launch.

### Pitfall 3: `visibilitychange visible` may fire during desktop tab-switch / window minimise

**What goes wrong:** Without the `isIos` gate, the forced `tick_time` fires on every desktop
tab-switch or window minimise → restore (macOS menu-bar also triggers `visibilitychange`).
This is not a display bug (one extra tick is imperceptible), but it fires unnecessary IPC on
desktop and violates phase boundary ("iOS-only change").

**How to avoid:** Gate the `visible` branch with `isIos && needsTickRef.current`.

### Pitfall 4: Stopwatch `Instant` does not advance during iOS CPU sleep

**What goes wrong:** `std::time::Instant` on iOS is backed by `mach_absolute_time`, which
stops ticking while the CPU is sleeping (i.e., during app suspension). A Running stopwatch
backgrounded for 10 real-world seconds may show only 0–2 seconds of elapsed time on resume
because `start.elapsed()` only counted the active CPU time.

**Why it happens:** This is a known Rust platform behavior on Apple platforms. The `suspend-time`
crate exists specifically to work around it, but it is not in this project's dependency set.

**Impact for Phase 56:** The phase requirement (LIFE-02) is "clock/stopwatch display refreshes
immediately on return to foreground." For the **clock**, this is fully satisfied — `tick_time`
calls `SystemTime::now()` which is wall-clock time and always correct. For the **stopwatch**,
the forced tick will display the Instant-measured elapsed time, which underestimates real
elapsed time if the app was suspended mid-run. This is **accepted** per D-56.3 ("Stopwatch
freeze-on-save policy is unchanged — v3.2 decision"). The HP-41CX hardware also stopped
counting when unpowered; "foreground-only live updates accepted as HP-41CX-faithful" is
the explicit REQUIREMENTS.md language for LIFE-02. No fix is required.

**Warning signs (expected, not a bug):** After a 30-second background, a running stopwatch
may advance by less than 30 seconds. The clock will show the correct wall-clock time.

### Pitfall 5: `isIos` ref stale closure concern

**What goes wrong:** `isIos` is React state, not a ref, and the `visibilitychange` handler
has empty deps (registered once on mount). In React's strict-mode double-invocation, the
closure captures the `isIos` value from the first render (before the `is_ios` IPC response
arrives). If the `is_ios` response has not yet arrived when `visibilitychange` fires (very
unlikely — the first `visibilitychange` fires only when the user backgrounds the app, well
after mount), the `isIos` gate would be `false` and the branch skipped.

**Impact:** Low risk — the `is_ios` command is a trivial compile-time constant, resolves
within one microtask, and `visibilitychange` never fires during the first render cycle.

**How to avoid (if desired):** Promote `isIos` to `isIosRef` mirroring the `needsTickRef`
pattern. This is Claude's Discretion per the plan.

---

## Code Examples

### tauri.ios.conf.json (complete file)

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "app": {
    "windows": [
      {
        "title": "HP-41 Calculator",
        "width": 440,
        "height": 1020,
        "resizable": false,
        "decorations": true,
        "visible": false,
        "backgroundThrottling": "throttle"
      }
    ]
  }
}
```

Source: `tauri.conf.json` (copy-then-extend); key name [VERIFIED: v2.tauri.app/reference/config]

### needsTickRef sync effect (placement near existing needsTick at App.tsx:487)

```typescript
// Keep needsTickRef current for the stable visibilitychange listener (D-56.1).
const needsTickRef = useRef(false);
useEffect(() => {
  needsTickRef.current = needsTick;
}, [needsTick]);
```

Source: established `busyRef` / `liveTickRef` pattern in this codebase [ASSUMED — code shape]

### visibilitychange handler extension (App.tsx:998, extend existing closure)

```typescript
const handleVisibilityChange = () => {
  if (document.visibilityState === 'hidden') {
    void invoke<void>('save_state').catch((err: unknown) => {
      console.warn('background save failed:', extractErrMessage(err));
    });
  } else if (document.visibilityState === 'visible' && isIos && needsTickRef.current) {
    // D-56.1/D-56.3: force one tick_time on foreground return so clock/stopwatch
    // display is correct within one frame. Gated: isIos (desktop must not fire
    // extra IPC) + needsTickRef (no IPC when nothing time-sensitive is on screen).
    if (!busyRef.current) {
      invoke<CalcStateView>('tick_time')
        .then(view => { setCalcState(view); setErrorMessage(null); })
        .catch(err => showToast(extractErrMessage(err)));
    }
  }
};
```

Source: Phase 54 code at App.tsx:998–1004 extended with new branch [ASSUMED — code shape]

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `backgroundThrottlingPolicy` (wrong name) | `backgroundThrottling` (correct Tauri v2 key) | Tauri v2.3.0 (introduced) | Implementors must use the field name from the schema, not the JS type name |
| Manual Swift plugin for foreground-resume notification | `visibilitychange visible` branch | Phase 54 established WKWebView-reliable pattern | No new Tauri plugin or Swift code needed |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | No companion Info.plist key is required for `backgroundThrottling: "throttle"` to take effect on iOS 17+ | §Architecture Patterns, Pitfall section | If a key IS required (e.g., UIBackgroundModes), the config change would be silently ineffective; device verify in D-56.4 catches this |
| A2 | `visibilitychange` with `visibilityState === 'visible'` fires reliably on WKWebView for the app-background → foreground transition (home button → return) | §Architecture Patterns | If it doesn't fire, the clock would not refresh on resume; device verify in D-56.4 catches this; fallback would be a native Tauri plugin emitting a JS event on `applicationWillEnterForeground` |
| A3 | `isIos` React state (set once on mount from `invoke('is_ios')`) is stable enough to close over in an empty-deps listener without promotion to ref | §Architecture Patterns, Pitfall 5 | If `is_ios` IPC response arrives after the first `visibilitychange` fires, the gate would skip the tick; consequence is one missed refresh (display corrects on next 100ms interval tick) |
| A4 | Exact placement and import shape of `needsTickRef` sync effect (code example above) | §Code Examples | Linter/compiler error if wrong; trivially corrected during implementation |

---

## Open Questions (RESOLVED)

1. **Does `backgroundThrottling: "throttle"` require a companion Info.plist key?**
   - What we know: Tauri docs list no companion key; the commit adds it as a pure `WKPreferences` API call; `WKPreferences.inactiveSchedulingPolicy` is documented as needing no special entitlement.
   - What's unclear: Apple sometimes adds background-mode requirements that aren't obvious from the API documentation alone.
   - Recommendation: Tagged A1 (ASSUMED). The on-device checkpoint (D-56.4) is the authoritative test. If backgrounding still causes full suspension, check WKPreferences entitlement requirements in Apple documentation.

2. **Should the `visible` branch use `isIosRef` instead of `isIos` state?**
   - What we know: `isIos` resolves in one microtask; `visibilitychange` cannot fire before user interaction (always after mount); practical risk is negligible.
   - What's unclear: Whether the linter treats `isIos` in an empty-deps handler as a stale-closure warning.
   - Recommendation: Start with `isIos` state (simpler). If the linter warns, promote to `isIosRef`. Claude's Discretion per D-56.1.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Tauri CLI | `just ios-build` | ✓ | v2.11 (from project) | — |
| Xcode / iOS toolchain | `just ios-build` | ✓ (established Phase 53) | — | — |
| Physical iPhone | D-56.4 on-device checkpoint | ✓ (used in Phases 53–55) | iPhone 15 Pro confirmed | — |
| `just ios-build` recipe | Build + install | ✓ (established Phase 53) | — | — |

No new environment dependencies.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) + on-device manual checkpoint |
| Config file | None (existing `cargo test` in `hp41-gui/src-tauri/`) |
| Quick run command | `cargo test -p hp41-gui` (Rust unit tests) |
| Full suite command | `just ci` (includes CLI + GUI) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | Notes |
|--------|----------|-----------|-------------------|-------|
| LIFE-01 | WKWebView not fully suspended on brief backgrounding | device-only | — | `just ios-build` smoke (config valid), then human checkpoint D-56.4 |
| LIFE-01 | `tauri.ios.conf.json` is valid JSON and merges cleanly | build check | `just ios-build` | Build failure = invalid config |
| LIFE-02 | `tick_time` IPC exists and returns `CalcStateView` | unit | `cargo test -p hp41-gui -- handle_tick_time` | Already tested at `commands.rs:1275,1297` |
| LIFE-02 | Clock/stopwatch correct on foreground return | device-only | — | Human checkpoint D-56.4 (10s background, verify jump) |
| LIFE-02 | `visible` branch does not fire on desktop | manual | `npm run dev` + tab-switch | Verify no extra IPC; or trust `isIos: false` on desktop |

### Sampling Rate

- **Per task commit:** `cargo test -p hp41-gui` (unit tests for tick_time)
- **Phase gate:** `just ios-build` succeeds + human checkpoint on real device (D-56.4)

### Wave 0 Gaps

None — existing test infrastructure covers all automatable requirements. The two device-only
checkpoints (LIFE-01 background throttling, LIFE-02 clock correctness) are deliberately not
automated per D-56.4 (Simulator is not authoritative for these behaviors).

---

## Security Domain

Phase 56 introduces no new IPC endpoints, no new capabilities, no data-handling changes, and
no network or storage changes. The `tick_time` and `is_ios` commands are read-only with no
user-supplied inputs. No ASVS categories are newly activated by this phase.

The `tauri.ios.conf.json` platform override file does not grant new capabilities; it only
modifies a WKWebView scheduling policy.

---

## Sources

### Primary (HIGH confidence)

- [v2.tauri.app/reference/config](https://v2.tauri.app/reference/config/) — WindowConfig.backgroundThrottling field name and type; BackgroundThrottlingPolicy enum values (`disabled`, `throttle`, `suspend`); iOS 17.0+ / macOS 14.0+ platform support; Tauri v2.3.0 introduction
- [v2.tauri.app/develop/configuration-files](https://v2.tauri.app/develop/configuration-files/) — Platform-specific config files (`tauri.ios.conf.json`); JSON Merge Patch (RFC 7396) merge semantics
- [schema.tauri.app/config/2](https://schema.tauri.app/config/2) — `app.windows` array type definition; `backgroundThrottling` field placement within `WindowConfig`
- Context7 `/websites/v2_tauri_app` — BackgroundThrottlingPolicy enum members (Throttle/Disabled); Since 2.3.0 annotation; iOS 17.0+ platform note
- `hp41-gui/src/App.tsx:997–1008` — Phase 54 `visibilitychange` listener (extends this); `busyRef`/`liveTickRef` ref pattern
- `hp41-gui/src-tauri/src/commands.rs:333,570` — `tick_time` and `is_ios` commands confirmed existing
- `hp41-core/src/ops/time/clock.rs:94` — `SystemTime::now()` in clock path → single tick always current
- `hp41-core/src/ops/time/stopwatch.rs:65` — `start.elapsed().as_secs_f64()` via `Instant` → does not advance during CPU sleep on iOS

### Secondary (MEDIUM confidence)

- [github.com/tauri-apps/tauri commit a2d36b8](https://github.com/tauri-apps/tauri/commit/a2d36b8c34a8dcfc6736797ca5cd4665faf75e7e) — backgroundThrottling feature commit; confirms no Info.plist changes
- [github.com/rust-lang/rust issues/87906](https://github.com/rust-lang/rust/issues/87906) — Instant/CLOCK_MONOTONIC suspension behavior discussion; Apple platforms use mach_absolute_time
- WebSearch: `mach_absolute_time` does not advance during iOS CPU sleep; `Instant` on iOS is suspend-unaware

### Tertiary (LOW confidence / ASSUMED)

- A1: No Info.plist companion key — inferred from Tauri commit + WKPreferences API pattern; not explicitly confirmed in official docs
- A2: `visibilitychange visible` fires for app-lifecycle transitions — inferred from symmetry with Phase 54 `hidden` direction; Apple Forums issue about unreliability concerns back/forward navigation, not app lifecycle

---

## Metadata

**Confidence breakdown:**

- `backgroundThrottling` JSON key name and location: HIGH — confirmed via Context7 + official config reference + JSON schema
- JSON Merge Patch array-replacement behavior: HIGH — confirmed via official Tauri docs + RFC 7396 spec
- No Info.plist companion key required: MEDIUM — inferred from implementation commit; no explicit "not required" statement
- `visibilitychange visible` WKWebView reliability for app lifecycle: MEDIUM — inferred from symmetry with working Phase 54 `hidden` direction; not separately tested in this research
- `Instant` suspend-unawareness on iOS: MEDIUM — cross-referenced from multiple Rust/Apple sources

**Research date:** 2026-06-03
**Valid until:** 2026-07-03 (stable API; Tauri v2 platform config format is stable)

---

## RESEARCH COMPLETE
