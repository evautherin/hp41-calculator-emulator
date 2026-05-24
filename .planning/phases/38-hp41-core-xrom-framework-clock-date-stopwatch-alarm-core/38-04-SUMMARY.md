---
phase: 38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core
plan: "04"
subsystem: hp41-core
tags: [time-module, alpha-append, stopwatch, instant, systemtime, formatting]
dependency_graph:
  requires:
    - 38-01  # Time Module framework scaffolding (stubs + state fields)
  provides:
    - TIME-CLK-03  # ATIME appends current time in 12h or 24h format
    - TIME-CLK-04  # ATIME24 always appends in 24-hour format
    - TIME-CLK-05  # ADATE appends current date in DMY/MDY locale format
    - TIME-SW-01   # RUNSW starts stopwatch with Instant::now()
    - TIME-SW-02   # STOPSW freezes elapsed into accumulated
    - TIME-SW-03   # SETSW presets accumulated from X register
    - TIME-SW-04   # RCLSW returns total elapsed as HH.MMSScc
    - TIME-SW-05   # SWPT recalls split-point time to X
    - TIME-SW-06   # STPW records current elapsed as new split point
    - TIME-SW-07   # SW activates stopwatch keyboard mode
    - TIME-SW-08   # get_stopwatch_display_str() for Phase 39/41 frontend
    - TIME-SW-09   # Stopwatch uses monotonic Instant (immune to NTP adjustments)
  affects:
    - hp41-core/src/ops/time/alpha_time.rs
    - hp41-core/src/ops/time/stopwatch.rs
    - hp41-core/src/ops/time/clock.rs
tech_stack:
  added: []
  patterns:
    - "SystemTime::now() + UNIX_EPOCH for wall-clock reads in alpha_time.rs"
    - "JDN algorithm (Richards 2013) for Unix timestamp → (year, month, day) conversion"
    - "Instant::now() for monotonic stopwatch timing (immune to NTP adjustments)"
    - "String-split at decimal point per P35 invariant for HH.MMSSss parsing in op_setsw"
    - "get_local_time() pub(crate) helper in alpha_time.rs (usable by sibling clock.rs once Plan 03 lands)"
key_files:
  created: []
  modified:
    - hp41-core/src/ops/time/alpha_time.rs
    - hp41-core/src/ops/time/stopwatch.rs
    - hp41-core/src/ops/time/clock.rs
decisions:
  - "Implement get_local_time() directly in alpha_time.rs (pub(crate)) rather than waiting for Plan 03 — Plan 03 runs in parallel so the helper can't be imported from clock.rs"
  - "JDN algorithm (Richards 2013) chosen over chrono dep — zero new runtime dependencies per ADR-v3.1-002 carry-forward"
  - "op_stpw captures split, op_swpt recalls — stub had these semantically inverted; corrected per HP 82182A OM"
  - "ClockDisplayMode derive Default fixed as Rule 1 pre-existing clippy warning (blocked -D warnings CI)"
  - "StopwatchMode derive Default + #[default] Idle replaces manual impl (clippy warning in new code)"
metrics:
  duration_minutes: 25
  completed: "2026-05-24T21:30:14Z"
  tasks_completed: 2
  files_created: 0
  files_modified: 3
---

# Phase 38 Plan 04: ALPHA Time Ops + Stopwatch State Machine Summary

Full implementations of ATIME/ATIME24/ADATE (SystemTime-based alpha-append ops with 12h/24h and DMY/MDY locale support) and the complete 7-op stopwatch state machine using monotonic Instant for NTP-immune elapsed-time measurement.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | ATIME, ATIME24, ADATE alpha-append ops | 1bf2017 | alpha_time.rs, clock.rs |
| 2 | Stopwatch state machine (RUNSW, STOPSW, SETSW, RCLSW, SWPT, STPW, SW) | 177a632 | stopwatch.rs |

## What Was Built

### Task 1: Alpha Time Ops (`alpha_time.rs`)

Full replacement of Wave 1 stubs with real `SystemTime::now()` + time-offset-adjusted clock reads:

- **`get_local_time(offset_secs: i64) -> (i32, u8, u8, u8, u8, u8)`** — `pub(crate)` helper; reads `SystemTime::now()`, adds `time_offset_secs`, decomposes via JDN algorithm (Richards 2013) into `(year, month, day, hour, minute, second)`
- **`op_atime`** — appends `HH:MM:SS` (24h) or ` H:MM:SS AM/PM` (12h) per `state.clock_12h`
- **`op_atime24`** — always appends 24-hour format regardless of `clock_12h`
- **`op_adate`** — appends ` M/DD/YYYY` (MDY, Flag 31 clear) or `DD/ M/YYYY` (DMY, Flag 31 set)
- All three ops APPEND to existing `alpha_reg` content and truncate to 24 chars

12-hour format details (HP 82182A OM §ATIME):
- Hour 0 → "12:MM:SS AM" (midnight)
- Hours 1-11 → " H:MM:SS AM" (single-digit with leading space)
- Hour 12 → "12:MM:SS PM" (noon)
- Hours 13-23 → " H:MM:SS PM" (single-digit with leading space, hours = H-12)

27 tests covering: format_24h/format_12h variants, get_local_time algorithm with known Unix timestamps, op_atime/op_atime24/op_adate append semantics, Flag 31 DMY/MDY check, truncation at 24 chars.

### Task 2: Stopwatch State Machine (`stopwatch.rs`)

Full replacement of Wave 1 stubs with complete monotonic-Instant state machine:

- **`StopwatchMode`** — `derive Default + #[default] Idle` (cleaner than manual impl)
- **`current_elapsed(state)`** — private helper; returns `accumulated + start.elapsed()` when Running, else `accumulated`
- **`op_runsw`** — sets mode to Running, sets `stopwatch_start = Some(Instant::now())`
- **`op_stopsw`** — if Running: adds `start.elapsed()` to accumulated, clears start, sets Stopped; else no-op
- **`op_setsw`** — parses X as HH.MMSSss via string-split (P35 invariant), writes to accumulated, mode → Stopped
- **`op_rclsw`** — computes total elapsed via `current_elapsed`, converts via `secs_to_hpnum_time`, pushes to X
- **`op_stpw`** — records `current_elapsed` into `stopwatch_split` without stopping (split capture)
- **`op_swpt`** — recalls `stopwatch_split` as HH.MMSScc to X (split recall)
- **`op_sw`** — sets `stopwatch_keyboard_mode = true`
- **`get_stopwatch_display_str`** — returns `"HH:MM:SS.cc"` when in keyboard mode (for Phase 39/41 frontend)
- **`secs_to_hpnum_time`** — unchanged conversion helper; now uses `!(0.0..360_000.0).contains()` per clippy

34 tests covering: all state transitions (Idle→Running, Running→Stopped, resume), SETSW parsing, RCLSW accuracy, STPW/SWPT round-trip, split capture semantics, display string format, serde skip verification for `stopwatch_start`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `get_local_time` not available from clock.rs (Plan 03 runs in parallel)**
- **Found during:** Task 1 implementation
- **Issue:** Plan 03 (which implements clock ops + `get_local_time` helper) runs in Wave 2 in parallel with Plan 04; the helper cannot be imported from `super::clock` at compile time
- **Fix:** Implemented `get_local_time()` directly in `alpha_time.rs` as `pub(crate)` — uses `SystemTime::now()` + JDN algorithm independently. When Plan 03 lands, its clock.rs can optionally use `alpha_time::get_local_time` (or implement its own equivalent via `super::alpha_time::get_local_time`)
- **Files modified:** `hp41-core/src/ops/time/alpha_time.rs`
- **Commit:** 1bf2017

**2. [Rule 1 - Bug] op_stpw and op_swpt semantically inverted in stub**
- **Found during:** Task 2 plan review
- **Issue:** The Wave 1 stub had `op_stpw` doing a full reset (Idle, clear accumulated/split) and `op_swpt` storing elapsed into split — exactly backwards from HP 82182A OM semantics. Per OM: STPW = "STop with lap (split capture)", SWPT = "SWitch with lap Point (recall)"
- **Fix:** Correct implementation per OM: `op_stpw` captures current elapsed into `stopwatch_split` (continues running); `op_swpt` recalls `stopwatch_split` to X register
- **Files modified:** `hp41-core/src/ops/time/stopwatch.rs`
- **Commit:** 177a632

**3. [Rule 1 - Bug] Pre-existing `ClockDisplayMode` impl Default clippy warning in clock.rs**
- **Found during:** Task 1 clippy clean check (justfile uses `-D warnings`)
- **Issue:** `ClockDisplayMode` in `clock.rs` (Plan 01 stub) had `impl Default for ClockDisplayMode { fn default() -> Self { ClockDisplayMode::Off } }` — clippy warns `this impl can be derived` and `-D warnings` would fail CI
- **Fix:** Changed to `#[derive(Default)]` with `#[default]` on `Off` variant
- **Files modified:** `hp41-core/src/ops/time/clock.rs`
- **Commit:** 1bf2017 (included with Task 1)

**4. [Rule 1 - Bug] StopwatchMode impl Default clippy warning (new code)**
- **Found during:** Task 2 initial implementation
- **Issue:** New code had manual `impl Default for StopwatchMode` — clippy warns `this impl can be derived`
- **Fix:** Changed to `#[derive(Default)]` with `#[default]` on `Idle` variant
- **Files modified:** `hp41-core/src/ops/time/stopwatch.rs`
- **Commit:** 177a632

## Known Stubs

None — all planned functionality fully implemented:
- `op_atime`, `op_atime24`, `op_adate`: real SystemTime + formatting (not placeholders)
- All 7 stopwatch ops: real state machine (not stubs)
- `get_stopwatch_display_str`: real formatter (not placeholder)

## Threat Surface Scan

No new network endpoints, auth paths, or file access patterns. `SystemTime::now()` is a trusted OS monotonic source (T-38-12 accepted per plan). `op_setsw` input validation implemented: `parse_time_hpnum`-style string-split with range checks (minutes < 60, seconds < 60, centiseconds < 100) mitigates T-38-10.

## Self-Check: PASSED

Files exist:
- `hp41-core/src/ops/time/alpha_time.rs` — FOUND (16.3K)
- `hp41-core/src/ops/time/stopwatch.rs` — FOUND (new complete implementation)
- `hp41-core/src/ops/time/clock.rs` — FOUND (clippy fix applied)

Commits exist:
- `1bf2017` — feat(38-04): implement ATIME, ATIME24, ADATE alpha-append ops
- `177a632` — feat(38-04): implement complete stopwatch state machine

Test suite: 2139 passed, 1 ignored (up from 2094 in Plan 01 baseline — 45 new tests added)
Clippy `-D warnings`: exits OK (0 issues)
