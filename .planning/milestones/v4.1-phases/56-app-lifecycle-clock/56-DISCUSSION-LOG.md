# Phase 56: App Lifecycle + Clock - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-03
**Phase:** 56-App Lifecycle + Clock
**Areas discussed:** Foreground trigger, Throttling policy value, Refresh scope/gating, Device verification

---

## Foreground trigger

| Option | Description | Selected |
|--------|-------------|----------|
| Extend visibilitychange | Add a `'visible'` branch to the existing Phase-54 listener (App.tsx:984). One listener handles hidden→save and visible→tick. Minimal surface, reuses WKWebView-reliable event. | ✓ |
| Separate focus listener | Distinct window `focus` / native become-active listener dedicated to clock refresh; second lifecycle hook, focus less reliable in WKWebView. | |
| Both events | visibilitychange('visible') AND focus with a double-fire guard. Most robust, more code. | |

**User's choice:** Extend visibilitychange (D-56.1)
**Notes:** Reuses the proven, already-registered empty-deps listener; avoids a second hook and double-fire risk.

---

## Throttling policy value

| Option | Description | Selected |
|--------|-------------|----------|
| throttle | STATE.md pre-research default. Reduces (not fully suspends) background timer activity — battery-friendly; forced tick on resume guarantees correctness anyway. | ✓ |
| disabled | Throttling off — webview stays fully live in background. More battery, overkill given LIFE-02 only needs correctness on resume. | |

**User's choice:** throttle (D-56.2)
**Notes:** iOS ≤16 background timer pause accepted as a known limitation (LIFE-02 accepts foreground-only live updates).

---

## Refresh scope/gating

| Option | Description | Selected |
|--------|-------------|----------|
| Gated single tick | Fire one `tick_time` only when `needsTick` (clock/stopwatch active). Matches LIFE-02 scope, zero waste. | ✓ |
| Always single tick | Fire `tick_time` unconditionally on resume; harmless but an unnecessary IPC each foreground. | |
| Full get_state resync | Call `get_state` for full resync; broader than needed — only clock/stopwatch is time-sensitive. | |

**User's choice:** Gated single tick (D-56.3)
**Notes:** Clock + stopwatch derive from `SystemTime::now()` + `time_offset_secs`, so one forced tick fully catches up elapsed background time.

---

## Device verification

| Option | Description | Selected |
|--------|-------------|----------|
| Inline checkpoints | Reuse Phase 53/54/55 pattern: automate code, pause at explicit on-device checkpoint tasks (background ~10s, return, confirm time correct within one frame; repeat with stopwatch). | ✓ |
| Simulator + light device | Smoke config in Simulator (can't reproduce real throttling) + single lighter manual device pass. Weaker evidence for LIFE-01. | |

**User's choice:** Inline checkpoints (D-56.4)
**Notes:** LIFE-01 #2 and LIFE-02 are only faithfully observable on real hardware.

## Claude's Discretion

- Exact placement/shape of the `'visible'` branch and ref plumbing for reading `needsTick`'s live value inside the stable listener.
- `tauri.ios.conf.json` merge mechanics and any companion Info.plist key (only the policy *value* `"throttle"` is locked).
- Plan decomposition + wave parallelization → `gsd-planner`.

## Deferred Ideas

None — discussion stayed within phase scope. (Signing/TestFlight → Phase 57; interrupting control alarms remain DEFERRED per existing project decision.)
