# Phase 39: hp41-cli — CLI Integration + Live Display - Context

**Gathered:** 2026-05-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 39 bridges the hp41-core Time Module (Phase 38, XROM 26, 35 Op variants) to the hp41-cli TUI. It delivers: the fourth JSON source-of-truth (`docs/hp41-time-functions.json`), the fourth `OnceLock` help-data pool in `help_data.rs`, exhaustive `op_display_name` arms for all 35 Time Op variants in `prgm_display.rs` (4-way invariant item 3), the `?` help overlay "Time Pac (XROM 26)" section, live-updating clock display in the TUI LCD area, interactive stopwatch keyboard mode with dedicated key bindings, alarm notification surfacing in the status bar, and XROM shadowing test extension covering all three XROM modules.

This phase completes the CLI surface — users can discover, invoke, and observe all Time Pac functions from the terminal. Phase 38 built the engine; Phase 39 exposes it.

</domain>

<decisions>
## Implementation Decisions

### Clock/Stopwatch Display Rendering
- **D-39.1:** Live clock and stopwatch display take HIGHEST priority in `get_display_string()` — above entry_buf, above PRGM mode, above ALPHA mode. When `state.clock_active` is true, format the current time (applying `time_offset_secs` + `clock_12h` + `clock_display_mode`) and return it. When `state.stopwatch_keyboard_mode` is true, format the elapsed stopwatch time as `HH:MM:SS.hh` and return it. This matches real HP-41CX behavior where CLKT and SW override all other display modes.
- **D-39.2:** The existing 16ms poll loop (`event::poll(Duration::from_millis(16))`) already redraws the TUI ~62 times/second without requiring a keypress. Clock display updates at ≥1 Hz (TIME-DSP-05) and stopwatch at ≥10 Hz (TIME-SW-08) are both trivially met — the poll loop calls `terminal.draw()` before polling, so display refreshes even when no input arrives. No new timer or async infrastructure needed.
- **D-39.3:** Clock display mode exits on ANY keypress — insert a check at the top of `handle_key()` that clears `state.clock_active = false` before processing the key. The key itself is still handled normally (not consumed by the clock exit).

### Stopwatch Keyboard Mode
- **D-39.4:** Interactive stopwatch mode (activated by `XEQ "SW"`) uses a top-level check in `handle_key()` before `PendingInput` routing — same architectural pattern as `alpha_mode`. When `state.stopwatch_keyboard_mode` is true, a dedicated key map intercepts R/S (start/stop toggle → RUNSW/STOPSW), a key for split (STPW), a key for reset, and Esc to exit the mode. All other keys are ignored during stopwatch mode.
- **D-39.5:** No new `PendingInput` variant for stopwatch mode. The mode is a CalcState transient flag (`stopwatch_keyboard_mode`), not a multi-key accumulation flow. The key mapping is flat — each key is a single dispatch, no accumulation needed.

### Alarm Event Draining
- **D-39.6:** Extend BOTH `call_dispatch()` and `call_dispatch_and_drain()` to drain `state.event_buffer` after existing `print_buffer` drain. Parse event strings: `alarm:message:*` → set `self.message` (status bar notification). `alarm:xeq:*` → dispatch the referenced label via existing XEQ infrastructure. `BEEP`/`TONE *` events → visual bell or ignored (no terminal audio).
- **D-39.7:** Call `hp41_core::ops::time::alarm::check_alarms(&mut self.state)` in the main `run()` loop body OUTSIDE the `event::poll` conditional — on every 16ms tick, not just on keypress. This satisfies TIME-ALM-08 ("on each keypress/dispatch, check for overdue alarms") extended to the idle case: alarms fire even when the user hasn't pressed a key. Drain event_buffer after the check.
- **D-39.8:** Alarm messages surface as `self.message` (the existing status-bar line below the display). No modal, no popup — the message persists until the next keypress clears it (same pattern as print output and error messages).

### JSON Canonical Data
- **D-39.9:** `docs/hp41-time-functions.json` uses 7 categories mirroring REQUIREMENTS.md groupings: "Time Clock" (TIME/DATE/T+X/CLOCK), "Time Date Arithmetic" (DATE+/DDAYS/DOW/DMY/MDY), "Time Display" (CLKT/CLKTD/SETIME/SETDATE), "Time Format" (CLK12/CLK24/CORRECT/SETAF/RCLAF), "Time Alpha" (ATIME/ATIME24/ADATE), "Time Stopwatch" (RUNSW/STOPSW/SETSW/RCLSW/SWPT/STPW/SW), "Time Alarm" (XYZALM/RCLALM/ALMCAT/CLALMA/CLALMX/CLRALMS/ALMNOW).
- **D-39.10:** Each entry carries `"xrom": { "module": "Time", "module_id": 26, "function_id": N }` per the Phase 28 D-28.3 schema. All 35 entries have `key_path: "XEQ \"MNEMONIC\""` since Time Pac functions are XEQ-by-name only (no physical key assignments).
- **D-39.11:** Inline `divergences` field only where OM behavior differs from emulator behavior — SETAF/RCLAF (accuracy factor stored but inert), CORRECT (no-op divergence), SW (centisecond resolution matches OM). Full taxonomy lives in `docs/hp41-time-divergences.md` (Phase 40); JSON inline fields cross-reference the divergences doc per the v3.1 D-34.3 pattern.

### Help Overlay Integration
- **D-39.12:** Fourth `OnceLock<Vec<HelpEntry>>` named `TIME_HELP_ENTRIES` in `help_data.rs` with narrow accessor `help_entries_time()`. `help_entries_all()` extends the chain to 4 pools: cv → math1 → stat1 → time. Fixed insertion order per D-34.6 convention.
- **D-39.13:** `?` help overlay gains "Time Pac (XROM 26)" section auto-derived from the JSON categories. Incremental substring search spans all four JSON pools.

### XROM Shadowing
- **D-39.14:** `xrom_shadowing.rs` extended to verify all Time Pac mnemonics are disjoint from Math 1 + Stat 1 + `BUILTIN_CARD_OP_NAMES` allowlist. Validates Pitfall 22 end-to-end across all three XROM modules.

### Prior Decisions (carried forward)
- **D-carried.1:** Zero new runtime dependencies (Phase 38 established).
- **D-carried.2:** "Pull on redraw" architecture — core remains thread-free and async-free. Frontend redraw cycle computes time-dependent display state on-demand (D-38 carried.7).
- **D-carried.3:** Right-panel `key_ref_entries()` filter: `entry.xrom.is_none()` excludes XROM-module functions from the right panel. Time entries discoverable via `?` overlay only (same as Math 1 + Stat 1).
- **D-carried.4:** 4-way exhaustive-match invariant: Phase 39 closes item 3 (CLI `prgm_display.rs`). Item 4 (GUI) deferred to Phase 41.

### Claude's Discretion
- Exact key bindings for interactive stopwatch mode (which physical keys map to start/stop/split/reset) — choose whatever feels natural and document in help overlay.
- Formatting details for clock display (spacing, AM/PM placement) — follow existing format patterns in `hp41-core/src/ops/time/alpha_time.rs`.
- Order of entries within each JSON category — follow the order from REQUIREMENTS.md or alphabetical by mnemonic.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 38 Context (predecessor)
- `.planning/phases/38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core/38-CONTEXT.md` — All D-38.* decisions on clock access, stopwatch state, alarm catalog, control alarm deferral

### Prior XROM Integration Precedent
- `hp41-cli/src/help_data.rs` — Three existing `OnceLock` pools + `help_entries_all()` merger + `help_overlay_rows()` + `filter_help_rows()` — extend with 4th pool
- `hp41-cli/src/prgm_display.rs` — Exhaustive `op_display_name()` match (4-way invariant item 3) — add 35 Time arms
- `hp41-cli/src/app.rs` — Event loop (`run()`), `handle_key()`, `call_dispatch()`, `call_dispatch_and_drain()`, `get_display_string()` — all Phase 39 integration points
- `hp41-cli/src/ui.rs` — `render_display()`, `get_display_string()` priority chain — extend for clock/stopwatch modes
- `hp41-cli/src/keys.rs` — `xeq_by_name_local_resolve()`, `xrom_resolve()` — Time ops already resolved via XROM bit-2 arm

### JSON Schema Precedent
- `docs/hp41-stat1-functions.json` — 26-entry Stat 1 JSON (most recent XROM module JSON — follow exact schema)
- `docs/hp41-math1-functions.json` — 45-entry Math 1 JSON (first XROM module JSON)

### Core Time Module Implementation
- `hp41-core/src/ops/time/` — All Time Pac implementation files (clock.rs, date_arith.rs, alpha_time.rs, stopwatch.rs, alarm.rs, modal.rs)
- `hp41-core/src/ops/time/alarm.rs` — `check_alarms()` function + `event_buffer` drain protocol
- `hp41-core/src/state.rs:290-354` — Phase 38 CalcState fields (clock_display_mode, stopwatch_mode, stopwatch_keyboard_mode, alarm_catalog_mode, clock_active, etc.)
- `hp41-core/src/ops/mod.rs` — 35 `Time*` Op variants in dispatch()

### XROM Shadowing Test
- `hp41-cli/tests/xrom_shadowing.rs` — Existing shadowing test for Math 1 + Stat 1 — extend to Time Module

### Requirements & Roadmap
- `.planning/REQUIREMENTS.md` — TIME-CLI-01..08 (8 requirements mapped to Phase 39)
- `.planning/ROADMAP.md` — Phase 39 goal, success criteria, depends-on

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `OnceLock<Vec<HelpEntry>>` pattern in `help_data.rs` — copy the Stat 1 block verbatim, change file name and accessor name
- `help_entries_all()` chain — append `.chain(help_entries_time().iter())` as 4th arm
- `call_dispatch_and_drain()` — extend with `event_buffer` drain block after `print_buffer` drain
- `get_display_string()` priority chain — insert clock/stopwatch checks at top
- `handle_key()` mode interception — alpha_mode pattern at line ~296 for stopwatch keyboard mode
- `check_autosave()` pattern — called in poll loop outside event conditional; same location for `check_alarms()`
- `self.message` status bar — reuse for alarm notifications (no new UI element needed)

### Established Patterns
- Exhaustive match in `prgm_display.rs` — NO `_ =>` catch-all; compile error forces coverage
- XROM functions are XEQ-by-name only — `key_path: "XEQ \"MNEMONIC\""` for all Time entries
- Right-panel filter: `entry.xrom.is_none()` — Time entries excluded from right panel (same as Math 1 + Stat 1)
- Sanctioned CI break in GUI `prgm_display.rs` — Phase 39 adds CLI arms only; GUI deferred to Phase 41

### Integration Points
- `hp41-cli/src/app.rs:270` — poll loop body: insert `check_alarms` + event drain
- `hp41-cli/src/app.rs:296` — `handle_key()` entry: insert clock-exit and stopwatch-mode checks
- `hp41-cli/src/app.rs:1783` — `call_dispatch_and_drain()`: add event_buffer drain block
- `hp41-cli/src/ui.rs:131` — `get_display_string()`: add clock/stopwatch priority above entry_buf
- `hp41-cli/src/help_data.rs:164` — `help_entries_all()`: add 4th chain arm
- `hp41-cli/tests/` — new `xrom_shadowing.rs` extension + `function_matrix_parity.rs` 4-pool test

</code_context>

<specifics>
## Specific Ideas

No specific requirements beyond the established v3.0/v3.1 integration pattern. Phase 39 is a direct mechanical extension of the pattern used in Phase 34 (Stat 1 CLI integration) — same JSON schema, same help_data extension, same prgm_display exhaustive match, same XROM shadowing test.

The novel elements are the live display modes (clock/stopwatch) and alarm draining — these have no direct precedent in prior CLI integration phases but the architectural decisions above (D-39.1 through D-39.8) fully specify the approach.

</specifics>

<deferred>
## Deferred Ideas

- **Alarm sound/bell** — Terminal bell (`\x07`) for alarm notifications. Could enhance UX but not in the OM spec for the emulator. Consider for a future polish phase.
- **Clock display format customization** — The OM supports only the fixed format; any additional formatting options are out of scope.

None of the above are scope creep from this discussion — they are pre-existing Phase 40/42 boundaries.

</deferred>

---

*Phase: 39-hp41-cli-cli-integration-live-display*
*Context gathered: 2026-05-25*
