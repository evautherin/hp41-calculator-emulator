# Phase 41: hp41-gui — GUI Integration + Live Display - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-25
**Phase:** 41-hp41-gui-gui-integration-live-display
**Areas discussed:** Live display timer, CATALOG 2 refactor, Alarm notification UX, Stopwatch mode in GUI

---

## Live Display Timer

### tick_time return type

| Option | Description | Selected |
|--------|-------------|----------|
| Full CalcStateView | Reuse existing from_state() projection. ~300 bytes JSON. Same data shape as dispatch_op. Also drains event_buffer (alarm events piggyback). | ✓ |
| Lightweight time-only struct | New struct with just { display_str, clock_active, stopwatch_mode, event_buffer }. ~80 bytes. Needs second type on both sides. | |
| You decide | Let Claude pick. | |

**User's choice:** Full CalcStateView
**Notes:** Simplicity wins — one type for all IPC responses.

### setInterval cadence

| Option | Description | Selected |
|--------|-------------|----------|
| Single 100ms interval | Covers both clock (≥1 Hz) and stopwatch (≥10 Hz). Simple: one interval, one boolean. ~10 IPC calls/sec. | ✓ |
| Dual cadence (1s clock, 100ms stopwatch) | More efficient for clock-only. Slightly more complex interval management. | |
| You decide | Let Claude pick. | |

**User's choice:** Single 100ms interval
**Notes:** None.

### Timer start/stop signaling

| Option | Description | Selected |
|--------|-------------|----------|
| CalcStateView fields | Add clock_active + stopwatch_keyboard_mode booleans. Check after every dispatch_op response. Zero new IPC commands for detection. | ✓ |
| Dedicated Tauri event | Core emits Tauri event when clock/stopwatch activates. More "push" style but adds event plumbing. | |

**User's choice:** CalcStateView fields
**Notes:** Response-driven detection — no new event infrastructure.

### Alarm checking in tick_time

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, piggyback on tick_time | tick_time calls check_alarms() before CalcStateView. ~100ms alarm latency during display modes. Dispatch-only outside. | ✓ |
| Separate alarm mechanism | Independent alarm checking. Would need its own periodic check or remain dispatch-only. | |

**User's choice:** Piggyback on tick_time
**Notes:** None.

---

## CATALOG 2 Refactor

| Option | Description | Selected |
|--------|-------------|----------|
| Refactor to loop now | D-36.1 scheduled this for the 3rd module. Loop over [(MATH_1, bit0), (STAT_1, bit1), (TIME, bit2)]. Fourth module benefits for free. | ✓ |
| Keep manual if-blocks | Add 3rd if-block. Defer loop to v3.3+. Simpler delta but accumulates debt. | |
| You decide | Let Claude pick. | |

**User's choice:** Refactor to loop now
**Notes:** Fulfills D-36.1's deferred commitment at the promised decision point.

---

## Alarm Notification UX

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse existing toast | Alarm events flow through event_buffer → toast. Keep 2s auto-dismiss. Zero new UI. | ✓ |
| Persistent alarm toast | Persist until acknowledged. More faithful to real HP-41. Needs new toast variant. | |
| Dedicated alarm banner | New UI element. Most distinction, most work. | |

**User's choice:** Reuse existing toast
**Notes:** None.

---

## Stopwatch Mode in GUI

### Keyboard mode

| Option | Description | Selected |
|--------|-------------|----------|
| XEQ buttons only | No special keyboard intercepts. User clicks XEQ buttons. Consistent with other XROM functions. | ✓ |
| Physical keyboard shortcuts | Intercept Space/S/R/Esc when stopwatch active. Mirrors CLI. Needs keydown handler. | |
| You decide | Let Claude pick. | |

**User's choice:** XEQ buttons only
**Notes:** GUI doesn't need CLI's interactive keyboard mode — on-screen keyboard provides the same functionality.

### Timer start signal

| Option | Description | Selected |
|--------|-------------|----------|
| Only clock_active or stopwatch_keyboard_mode | Matches display priority chain. Programmatic RUNSW without SW runs in background. Faithful to HP-41CX. | ✓ |
| Include stopwatch_mode == Running too | Start interval whenever stopwatch running. More forgiving UX but deviates from OM. | |

**User's choice:** Only clock_active || stopwatch_keyboard_mode
**Notes:** Background stopwatch (RUNSW without SW) is silent — check via RCLSW.

---

## Claude's Discretion

- tick_time Rust function signature and body shape
- Frontend setInterval management (useRef pattern, cleanup)
- Mutex acquisition pattern in tick_time (lock vs try_lock)
- Plan slicing (recommended 3-4 plans)
- op_catalog loop body shape
- HelpOverlay expanded state widening approach

## Deferred Ideas

- **Persistent alarm acknowledgment UI** — beep-until-acknowledged fidelity. Future polish phase.
- **Stopwatch physical keyboard shortcuts in GUI** — Space/S/R/Esc bindings. Decided against for simplicity.
- **Dual-cadence timer optimization** — 1s clock / 100ms stopwatch. Decided against for simplicity.
