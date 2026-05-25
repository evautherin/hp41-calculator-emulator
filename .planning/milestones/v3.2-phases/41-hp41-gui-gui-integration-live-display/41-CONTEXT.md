# Phase 41: hp41-gui — GUI Integration + Live Display - Context

**Gathered:** 2026-05-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 41 wires all 35 Time Pac ops into the Tauri v2 + React desktop GUI. This phase delivers the 4-way exhaustive-match invariant item 4 (35 new `op_display_name` arms in `hp41-gui/src-tauri/src/prgm_display.rs`), CATALOG 2 "TIME 2C" section via a generic registry-driven refactor (fulfilling D-36.1), the fourth help overlay section "Time Pac (XROM 26)", and — most significantly — the FIRST live-updating display behavior in the GUI: clock and stopwatch time refresh via a new `tick_time` Tauri command with conditional `setInterval`, a documented D-11 exception. Alarm notifications surface through the existing toast overlay.

The mechanical parts (op_display_name arms, help overlay, help_data.ts, modal prompts) follow the Phase 36 (Stat 1 GUI) precedent exactly. The novel parts are the live display timer architecture and the CATALOG 2 refactor from manual if-blocks to a generic loop.

</domain>

<decisions>
## Implementation Decisions

### Live Display Timer Architecture (TIME-GUI-04, TIME-GUI-05)
- **D-41.1:** New `tick_time` Tauri command returns a **full `CalcStateView`** (reusing existing `from_state()` projection). Same data shape as `dispatch_op` — simplifies frontend (one type for all IPC responses). `tick_time` also drains `event_buffer`, so alarm events piggyback for free. Requires new Tauri v2.11 permission TOML (`hp41-gui/src-tauri/permissions/tick-time.toml`).
- **D-41.2:** **Single 100ms `setInterval`** in the frontend for both clock (≥1 Hz per TIME-DSP-05) and stopwatch (≥10 Hz per TIME-SW-08). ~10 IPC calls/sec when active. One interval, one boolean — avoids dual-cadence complexity.
- **D-41.3:** **`clock_active` and `stopwatch_keyboard_mode` booleans added to `CalcStateView`** (projected from CalcState transient fields). Frontend starts `setInterval` when either is `true` in a `dispatch_op` response; clears interval when both are `false`. Detection is response-driven — no new Tauri events, no polling outside of the active live-display modes.
- **D-41.4:** **`tick_time` calls `check_alarms()` before building CalcStateView.** Alarms fire promptly (~100ms latency) during clock/stopwatch display modes. Outside those modes, alarms fire on the next `dispatch_op` call (per-interaction, matching TIME-ALM-08 "on each keypress/dispatch"). No additional alarm-checking mechanism needed.

### CATALOG 2 Refactor (TIME-GUI-02)
- **D-41.5:** **Refactor `op_catalog(state, 2)` from manual if-blocks to a generic loop** over `[(MATH_1, 0b0000_0001), (STAT_1, 0b0000_0010), (TIME_MODULE, 0b0000_0100)]`. Fulfills D-36.1's deferred commitment at the promised decision point (the THIRD XROM module). Existing parallel if-blocks (bit-0 for MATH_1, bit-1 for STAT_1) replaced by a single loop. Fourth module (Advantage Pac) benefits for free.

### Alarm Notification UX (TIME-GUI-06)
- **D-41.6:** Alarm notifications **reuse the existing toast overlay** (single-toast policy, 2s auto-dismiss). Alarm events flow through `event_buffer` → `CalcStateView.event_buffer` → toast queue per Phase 26 D-26.11 infrastructure. The `"alarm:message:{text}"` event string is parsed in App.tsx and surfaced as toast text. Control alarm `"alarm:xeq:{label}"` events trigger dispatch through existing XEQ infrastructure. No new UI components.

### Stopwatch Mode in GUI (TIME-GUI-05)
- **D-41.7:** Stopwatch mode in GUI has **NO dedicated keyboard intercepts** (unlike CLI's D-39.4-5 interactive mode). User clicks XEQ "RUNSW"/"STOPSW"/"SWPT"/"STPW" via the GUI on-screen keyboard. Live display updates via `tick_time` + `setInterval` when `stopwatch_keyboard_mode` is true. Simpler UX — consistent with how all other XROM functions work in the GUI.
- **D-41.8:** Frontend `setInterval` starts when `clock_active || stopwatch_keyboard_mode` in the CalcStateView response. Programmatic `RUNSW` without `SW` interactive mode runs the stopwatch in background — no interval, no live display, check via `RCLSW`. Faithful to HP-41CX behavior.

### Prior Decisions (carried forward)
- **D-carried.1:** Zero new runtime dependencies in hp41-core, hp41-cli, or hp41-gui.
- **D-carried.2:** "Pull on redraw" architecture — hp41-core remains thread-free and async-free. `tick_time` reads time-dependent state on-demand (D-38 carried.7).
- **D-carried.3:** Right-panel `key_ref_entries()` filter: `entry.xrom.is_none()` excludes XROM-module functions from the right panel. Time entries discoverable via `?` overlay only (same as Math 1 + Stat 1).
- **D-carried.4:** 4-way exhaustive-match invariant: Phase 41 closes item 4 (GUI `prgm_display.rs`). Sanctioned `non-exhaustive patterns` CI break in `hp41-gui` (open since Phase 38) closes when 35 arms land.
- **D-carried.5:** D-25.6 CLI ↔ GUI parity — the 35 display-name arm strings are deliberately duplicated from Phase 39's CLI work. The compile-time exhaustive match in both files is what makes this invariant load-bearing.
- **D-carried.6:** No `println!`/`eprintln!` in hp41-core; `op_catalog` loop uses `state.print_buffer.push(format!(...))`.
- **D-carried.7:** `CalcState::migrate_after_load()` in `state.rs` is the single source of truth for XROM module migration — both CLI and GUI persistence paths call it (D-33.7 inherited). Phase 41 inherits Time Module bit-2 activation for free.
- **D-carried.8:** JSON canonical data flow — `docs/hp41-time-functions.json` consumed read-only via Vite static import (TypeScript compile error on malformed JSON is the GUI's hard-build-blocker equivalent of CLI's `OnceLock` panic).
- **D-carried.9:** Modal prompts for SETIME/SETDATE already plumbed end-to-end through `CalcStateView::modal_prompt` (Phase 31 infrastructure). Phase 41 does NOT touch the modal routing — GUI inherits ModalProgram::Time dispatch via shared hp41-core code.

### Claude's Discretion
- Exact `tick_time` Rust function signature and body shape — mirror existing `get_state` thunk pattern in `commands.rs`, add `check_alarms()` call
- Frontend `useRef<ReturnType<typeof setInterval>>` management and cleanup in `App.tsx` — standard React pattern
- Whether `tick_time` acquires the AppState Mutex with `lock()` or `try_lock()` (recommend `lock()` with `.unwrap_or_else(|e| e.into_inner())` per existing poisoned-lock pattern)
- Plan slicing (recommend 3-4 plans per Phase 36 precedent: arms+catalog refactor, help overlay+help_data, tick_time+live display, vitest extensions)
- `op_catalog` loop body shape — array of `(&XromModule, u8)` tuples with `for (module, bit) in XROM_REGISTRY { if state.xrom_modules & bit != 0 { ... } }`
- HelpOverlay `expanded` state widening to 4 keys — extend the discriminated record per Phase 36 pattern or refactor to `Record<SectionId, boolean>` (recommend hold the discriminated record for grep-friendliness)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 38/39 Context (source material for implementation decisions)
- `.planning/phases/38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core/38-CONTEXT.md` — All D-38.* decisions (clock access, stopwatch state, alarm catalog, control alarm deferral, date parsing)
- `.planning/phases/39-hp41-cli-cli-integration-live-display/39-CONTEXT.md` — All D-39.* decisions (live display rendering, stopwatch keyboard mode, alarm draining, JSON canonical data, help overlay)

### Phase 36 (direct STRUCTURAL analog — v3.1 GUI integration)
- `.planning/milestones/v3.1-phases/36-hp41-gui-gui-integration/36-CONTEXT.md` — Phase 36 decisions D-36.1 through D-36.4; 1:1 structural template for the mechanical parts of Phase 41 (op_display_name arms, help overlay extension, CATALOG 2 core touch, vitest extensions)
- `.planning/milestones/v3.0-phases/31-hp41-gui-integration/31-CONTEXT.md` — Phase 31 decisions (Math Pac I GUI integration — help overlay, cancellation channel, LCD-alternation modal prompts)

### hp41-gui Existing Pattern Reservoir
- `hp41-gui/src-tauri/src/prgm_display.rs` — Exhaustive `op_display_name()` match (currently sanctioned `non-exhaustive patterns` CI break since Phase 38); extend with 35 Time arms
- `hp41-gui/src-tauri/src/commands.rs` — `dispatch_op`, `get_state`, `run_stop`, `request_cancel` thunks + event_buffer drain pattern; add `tick_time` command following same pattern
- `hp41-gui/src-tauri/src/types.rs` — `CalcStateView` struct + `from_state()` constructor; add `clock_active` + `stopwatch_keyboard_mode` boolean fields
- `hp41-gui/src-tauri/src/key_map.rs` — String ID → Op resolver; NOT touched (XEQ-by-name resolves Time ops via shared `xrom_resolve`)
- `hp41-gui/src-tauri/src/lib.rs` — `setup()`, `generate_handler!` registration; add `tick_time` to handler macro
- `hp41-gui/src-tauri/permissions/*.toml` — Tauri v2.11 inline-command permission registry; add `tick-time.toml`
- `hp41-gui/src/App.tsx` — React root, toast overlay, event_buffer consumption; add setInterval management for tick_time
- `hp41-gui/src/HelpOverlay.tsx` — `SECTIONS` array (currently 3 entries); extend to 4 with "Time Pac (XROM 26)"
- `hp41-gui/src/help_data.ts` — 3 existing `OnceLock`-analog Vite imports; add 4th for `docs/hp41-time-functions.json`

### hp41-core Public Surface (consumed read-only except op_catalog)
- `hp41-core/src/ops/time/alarm.rs` — `check_alarms()` function; called by `tick_time`
- `hp41-core/src/ops/mod.rs` — 35 `Time*` Op variants in `dispatch()`
- `hp41-core/src/ops/program.rs` — `op_catalog(state, 2)` — refactor from manual if-blocks to generic loop (D-41.5)
- `hp41-core/src/ops/math1/xrom.rs` — `MATH_1`, `STAT_1`, `TIME_MODULE` consts + `xrom_resolve()`
- `hp41-core/src/state.rs` — CalcState with `clock_active`, `stopwatch_keyboard_mode` transient fields

### JSON Pipeline (read-only consumer)
- `docs/hp41-time-functions.json` — 35-entry Time Pac JSON (Phase 39 D-39.9); consumed via Vite static import
- `docs/hp41-stat1-functions.json` — 26-entry Stat 1 JSON (import pattern template)
- `docs/hp41-math1-functions.json` — 45-entry Math 1 JSON (first import pattern)

### Requirements & Roadmap
- `.planning/REQUIREMENTS.md` — TIME-GUI-01..07 (7 requirements mapped to Phase 41)
- `.planning/ROADMAP.md` — Phase 41 goal, success criteria (4 items), depends-on

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`hp41-gui/src-tauri/src/commands.rs::get_state`** — thunk pattern for `tick_time`: acquire Mutex, build CalcStateView from state. Add `check_alarms()` call before `from_state()`.
- **`hp41-gui/src-tauri/src/commands.rs::dispatch_op` event_buffer drain** — exact pattern `tick_time` reuses: `let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();`
- **`hp41-gui/src/help_data.ts`** — 3 existing Vite static imports; copy stat1 import pattern for time
- **`hp41-gui/src/HelpOverlay.tsx:SECTIONS`** — 3-entry array; add 4th entry `{ id: 'time', heading: 'Time Pac (XROM 26)', predicate: (e) => e.xrom?.module === 'Time' }`
- **Phase 39 CLI `prgm_display.rs`** — 35 Time arm strings to copy verbatim per D-25.6 parity
- **Existing toast overlay in `App.tsx`** — reused for alarm notifications; `"alarm:message:{text}"` event parsing

### Established Patterns
- Exhaustive match in `prgm_display.rs` — NO `_ =>` catch-all; compile error forces coverage
- Tauri v2.11 inline-command permissions: TOML in `hp41-gui/src-tauri/permissions/`, referenced in `capabilities/default.json`
- `busyRef` two-layer debounce in App.tsx — `tick_time` does NOT need this (no user-initiated overlap risk, but consider guard against concurrent `tick_time` + `dispatch_op` Mutex contention)
- JSON canonical data flow + Vite static-import build-time gate
- `CalcStateView::from_state` is the SINGLE projection point — all new fields added here

### Integration Points
- **4-way invariant item 4 closure:** 35 Time arms in `hp41-gui/src-tauri/src/prgm_display.rs` close the sanctioned CI break; `cargo check -p hp41-gui` clean
- **`generate_handler!` in lib.rs:** add `tick_time` to the macro invocation alongside existing commands
- **`capabilities/default.json`:** reference new `tick-time` permission identifier
- **`CalcStateView` in types.rs:** add `clock_active: bool` + `stopwatch_keyboard_mode: bool` fields; set in `from_state()` from CalcState transient fields
- **App.tsx event_buffer consumption:** extend existing `"alarm:*"` event parsing (currently handles `event_buffer` lines as toasts) to recognize `"alarm:message:{text}"` and `"alarm:xeq:{label}"` patterns
- **`op_catalog` refactor in `hp41-core/src/ops/program.rs`:** replace 2 manual if-blocks with a loop over 3 `(XromModule, u8)` tuples; both CLI and GUI inherit

</code_context>

<specifics>
## Specific Ideas

No specific requirements beyond the decisions captured above. The mechanical parts follow Phase 36 precedent exactly. The live display timer is the only novel element and is fully specified by D-41.1 through D-41.4 + D-41.8.

</specifics>

<deferred>
## Deferred Ideas

- **Persistent alarm acknowledgment UI** — Real HP-41CX beeps until user acknowledges. A persistent toast or modal for alarm acknowledgment could enhance fidelity. Belongs in a future polish phase, not Phase 41.
- **Stopwatch physical keyboard shortcuts in GUI** — Space/S/R/Esc bindings like the CLI. Decided against for Phase 41 (D-41.7) but could be added as a UX enhancement.
- **Dual-cadence timer optimization** — 1s for clock-only, 100ms for stopwatch. Decided against for simplicity (D-41.2) but could optimize IPC load if profiling shows need.

None are scope creep from this discussion — all are potential enhancements deferred to future phases.

</deferred>

---

*Phase: 41-hp41-gui-gui-integration-live-display*
*Context gathered: 2026-05-25*
