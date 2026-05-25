---
phase: 39-hp41-cli-cli-integration-live-display
plan: "02"
subsystem: hp41-cli
tags: [live-display, clock, stopwatch, alarm, tui, real-time]
dependency_graph:
  requires: [39-01]
  provides: [TIME-CLI-05, TIME-CLI-06, TIME-CLI-07]
  affects: [hp41-cli/src/ui.rs, hp41-cli/src/app.rs]
tech_stack:
  added: []
  patterns:
    - "pull-on-redraw: get_clock_display_str / get_stopwatch_display_str called every frame"
    - "drain_event_buffer: alarm event routing from CalcState.event_buffer"
    - "handle_stopwatch_mode_key: routing block pattern (like handle_alpha_mode_key)"
key_files:
  modified:
    - hp41-cli/src/ui.rs
    - hp41-cli/src/app.rs
decisions:
  - "D-39.1: clock/stopwatch display at highest priority in get_display_string() — before entry_buf"
  - "D-39.3: clock-exit is mutation-only, not a routing block — key falls through for normal processing"
  - "D-39.4: stopwatch-mode is a routing block — keys are consumed by handle_stopwatch_mode_key"
  - "D-39.6: drain_event_buffer() appended to both call_dispatch and call_dispatch_and_drain"
  - "D-39.7: check_alarms + drain_event_buffer called on every 16ms tick in run() loop"
metrics:
  duration: "~15 minutes"
  completed: "2026-05-25"
  tasks_completed: 3
  tasks_total: 3
  files_modified: 2
---

# Phase 39 Plan 02: CLI Live Display and Alarm Drain Summary

Live-updating clock and stopwatch display wired into the TUI, interactive stopwatch keyboard mode with dedicated key bindings, and alarm event draining integrated into the CLI dispatch and main loop.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Display priority + clock exit behavior | a7d54cf | hp41-cli/src/ui.rs, hp41-cli/src/app.rs |
| 2 | Stopwatch keyboard mode + alarm drain | a7d54cf | hp41-cli/src/app.rs |
| 3 | Verify live display (checkpoint:human-verify) | — | Auto-approved (non-blocking; post-merge UAT) |

## What Was Built

**ui.rs — `get_display_string()` priority chain (D-39.1):**

The existing priority chain `entry_buf > prgm > alpha > formatted X` was extended to:
`clock_active > stopwatch_keyboard_mode > entry_buf > prgm > alpha > formatted X`.

Two new use statements pull `get_clock_display_str` from `hp41_core::ops::time::clock` and
`get_stopwatch_display_str` from `hp41_core::ops::time::stopwatch`. Both return `Option<String>`
and are called at the top of `get_display_string()` (pull-on-redraw pattern, D-carried.7).

**app.rs — clock-exit intercept (D-39.3):**

Inserted after the `alpha_mode` routing block, before the f-prefix state machine. If
`state.clock_active` is true, the flag is cleared. The key falls through for normal processing —
this is mutation-only, not a routing block. Any keypress exits clock display without consuming
the key.

**app.rs — stopwatch-mode intercept (D-39.4):**

Immediately after the clock-exit block. If `state.stopwatch_keyboard_mode` is true,
`handle_stopwatch_mode_key(key)` is called and the method returns — keys are consumed by
the stopwatch handler (like `handle_alpha_mode_key`).

**app.rs — `handle_stopwatch_mode_key()` (D-39.4, D-39.5):**

New method following the `handle_alpha_mode_key` pattern:
- `Esc` → set `stopwatch_keyboard_mode = false` (exit mode)
- `Space` or `Enter` → toggle start/stop via `Op::TimeRunsw` / `Op::TimeStopsw`
- `'s'` → split/lap via `Op::TimeSwpt`
- `'r'` → reset via `Op::TimeStpw`
- `_` → all other keys consumed silently

Uses `hp41_core::ops::time::StopwatchMode` for the start/stop toggle comparison.

**app.rs — `drain_event_buffer()` (D-39.6):**

New method that drains `state.event_buffer` and routes:
- `"alarm:message:{text}"` → `self.message = Some(text)`
- `"alarm:xeq:{label}"` → `hp41_core::run_program(state, label)`; on error, sets `self.message`
- `"alarm:interrupting:..."` and all other events → silently ignored

**app.rs — alarm check in `run()` loop (D-39.7):**

After `self.check_autosave()`, two new calls on every 16ms tick:
1. `hp41_core::ops::time::alarm::check_alarms(&mut self.state)` — pushes events
2. `self.drain_event_buffer()` — processes them

This makes alarms fire in real-time without user interaction.

**app.rs — `drain_event_buffer()` at dispatch tails (D-39.6):**

Appended to both `call_dispatch()` and `call_dispatch_and_drain()` after `maybe_auto_open_collect_for_modal()`, so alarm events triggered by program dispatch are also processed immediately.

## Deviations from Plan

None — plan executed exactly as written.

## Verification

- `cargo build -p hp41-cli` — passes (no errors)
- `cargo clippy -p hp41-cli -- -D warnings` — passes (no warnings)
- Task 3 (manual TUI verification) — auto-approved per `<auto_mode>` directive; post-merge UAT required

## Self-Check: PASSED

- `hp41-cli/src/ui.rs` — modified (exists)
- `hp41-cli/src/app.rs` — modified (exists)
- Commit `a7d54cf` — exists in git log

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced.
The `drain_event_buffer` event routing operates on emulator-internal strings from
`CalcState.event_buffer` — no external input path. This matches the T-39-03 / T-39-04
threat register entries in the plan (both `accept` disposition, no mitigation required).
