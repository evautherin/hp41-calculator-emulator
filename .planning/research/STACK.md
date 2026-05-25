# Technology Stack -- v3.2 Time Pac

**Project:** HP-41 Calculator Emulator
**Researched:** 2026-05-24
**Scope:** NEW dependencies and architectural additions needed to add HP-41CX Time Module (XROM 26, HP 82182A, OM 00041-90035) behavioral emulation on top of the validated v3.1 stack.

---

## Executive Summary

**Recommendation: ZERO new runtime dependencies in `hp41-core` or `hp41-cli` for v3.2.**

The Time Pac introduces four capability domains not present in the v3.0/v3.1 surface:

1. **System clock access** (TIME, DATE, SETIME, SETDATE, CORRECT, T+X) -- reads the host OS real-time clock via `std::time::SystemTime`, which is part of the Rust standard library and already available without any dependency addition. Not an external crate.

2. **Date arithmetic** (DATE+, DDAYS, DOW, DMY/MDY) -- pure integer algorithms over the Gregorian calendar. The Fliegel-Van Flandern Julian Day Number conversion (1968) is a well-known ~15-line integer-only algorithm. Days-between-dates = `JDN(date2) - JDN(date1)`. Day-of-week = `JDN mod 7`. Date+N = `JDN_to_date(date_to_JDN(date) + N)`. No external date crate needed.

3. **Stopwatch** (SW, RUNSW, STOPSW, SETSW, RCLSW, SWPT) -- elapsed-time tracking via `std::time::Instant` (monotonic clock, standard library). The stopwatch needs an `Option<Instant>` start-marker and an accumulated `Duration` on `CalcState`. Live-updating display is a frontend concern (CLI event loop / GUI timer), not a core library concern.

4. **Alarm system** (XYZALM, ALMCAT, ALMNOW, RCLALM, CLALMA, CLALMX, CLRALMS, SETAF, RCLAF) -- a `Vec<Alarm>` catalog on `CalcState` with date/time/message/repeat-interval fields. Alarm triggering is a frontend polling concern (check alarms against current time in the event loop). No external scheduler or async runtime needed.

All four domains are implementable with `std::time` (standard library) + hand-coded Gregorian calendar arithmetic + existing `HpNum` / `rust_decimal` infrastructure. The `chrono` crate (0.4.x) and `time` crate (0.3.x) were evaluated and rejected -- they are 10x-50x the code surface needed for the 5 date functions the Time Pac requires, and they introduce transitive dependencies that violate the project's zero-new-runtime-deps discipline.

**Confidence: HIGH** -- `std::time::SystemTime` and `std::time::Instant` are core Rust standard library types documented at doc.rust-lang.org; the Fliegel-Van Flandern algorithm is a textbook reference published in Communications of the ACM (1968) and used by the US Naval Observatory.

---

## Verified Source Material

**HP 82182A Time Module Quick Reference Card (82182-90002, November 1981) -- read directly.**

Complete function catalog: 28 functions in the original HP 82182A module (XROM 26), plus 5 CX-only additions (CLALMA, CLALMX, CLRALMS, RCLALM, SWPT) for a total of 33 time-related operations. The XROM ID is confirmed as **26** per the HP-41 Module Database at calc.fjk.ch (entries: "Time Module 1A/1B/1C" for the plug-in, "Time Module 2C" for the CX-internal variant).

**Date/time representation formats (from QRC):**

| Format | Setting | Input/Output | Display (from keyboard) |
|--------|---------|-------------|------------------------|
| MDY | `MDY` function | MM.DDYYYY | MM/DD/YY day |
| DMY | `DMY` function (sets flag 31) | DD.MMYYYY | DD.MM.YY day |

- **TIME format:** HH.MMSSss (same as existing H.MS format in `hms.rs`; 24-hour internal, CLK12/CLK24 affects display only)
- **SETIME range:** 0.000000-11.595999 = AM; 12.000000-23.595999 = PM; negative values -1.000000 to -11.595999 = PM
- **T+X format:** +/-HHHHH.MMSShh (hours can exceed 24; crosses date boundary)
- **Date range:** October 15, 1582 (first Gregorian date) through September 10, 4320 (999,999 days later) per Free42 documentation; confirmed by QRC's "all trailing digits after the year must be zero" constraint
- **XYZALM stack:** Z=repeat interval, Y=date, X=time; ALPHA=message/label/function

---

## Recommended Stack

### Core Framework (unchanged)

| Technology | Version | Purpose | Status |
|------------|---------|---------|--------|
| Rust stable | MSRV 1.88 | Core language | Already in use |
| rust_decimal | 1.42 | BCD-accurate arithmetic via HpNum | Already in use |
| serde / serde_json | 1.x | State persistence | Already in use |
| thiserror | 2.0 | Error types | Already in use |

### New Standard Library Usage (no crate additions)

| Std Module | Purpose | Why Safe |
|------------|---------|----------|
| `std::time::SystemTime` | TIME/DATE: read host OS clock | Part of Rust std; cross-platform (POSIX `clock_gettime` on macOS/Linux, `GetSystemTimePreciseAsFileTime` on Windows); returns seconds-since-UNIX-epoch |
| `std::time::Instant` | Stopwatch: monotonic elapsed-time tracking | Already used in `hp41-cli/src/app.rs` for auto-save timer; guaranteed monotonic; nanosecond precision |
| `std::time::Duration` | Stopwatch: accumulated time storage | Already used in `hp41-cli/src/app.rs` |
| `std::time::UNIX_EPOCH` | Anchor for SystemTime-to-calendar conversion | Standard constant; `SystemTime::now().duration_since(UNIX_EPOCH)` gives seconds since 1970-01-01T00:00:00Z |

### Hand-Coded Algorithms (no external crate)

| Algorithm | Lines | Purpose | Source |
|-----------|-------|---------|--------|
| Fliegel-Van Flandern JDN | ~15 | Gregorian (Y,M,D) <-> Julian Day Number | Communications of the ACM, 1968; US Naval Observatory reference |
| Unix-seconds-to-calendar | ~25 | Convert `UNIX_EPOCH` offset to (Y,M,D,H,M,S) | Standard algorithm using JDN + modular arithmetic |
| Calendar-to-unix-seconds | ~15 | Convert (Y,M,D,H,M,S) to seconds-since-epoch | Inverse of above |
| Day-of-week from JDN | 1 | `DOW = (JDN + 1) % 7` (0=Sunday) | Standard property of Julian Day Numbers |
| Date+N days | 1 | `JDN_to_date(date_to_JDN(base) + N)` | Trivial composition |
| Days between dates | 1 | `date_to_JDN(d2) - date_to_JDN(d1)` | Trivial composition |
| HMS parse/format | 0 | Already implemented in `hp41-core/src/ops/hms.rs` | Reuse `parse_hms()` pattern |

**Total hand-coded surface: ~60 lines** for all date/time conversion logic, plus the operational implementations that consume them.

### TUI (`hp41-cli`) Additions

| Technology | Version | Purpose | Status |
|------------|---------|---------|--------|
| ratatui | 0.30 | Live stopwatch display (repaint on 16ms tick) | Already in use; event loop already polls at 16ms |
| crossterm | 0.29 | Terminal event handling | Already in use |

No new CLI dependencies. The existing 16ms poll interval in `app.rs` (`event::poll(Duration::from_millis(16))`) already provides ~60fps repaint cadence, sufficient for stopwatch display updates. The clock display (CLKT/CLKTD/CLOCK) reuses the same tick.

### GUI (`hp41-gui`) Additions

| Technology | Version | Purpose | Status |
|------------|---------|---------|--------|
| Tauri v2 | 2.11 | Desktop app framework | Already in use |
| React 18 | 18.x | Frontend rendering | Already in use |

No new GUI dependencies. Live stopwatch/clock display uses `setInterval()` in React (already a standard browser API) to poll `get_state()` at ~100ms intervals when stopwatch/clock-display mode is active. This is a **controlled exception** to the no-polling rule (D-11) -- the stopwatch/clock literally requires periodic display refresh; the poll is gated behind a `clock_display_active || stopwatch_running` flag and stops when neither mode is active.

---

## New CalcState Fields

Based on the Time Pac's feature surface, the following new fields are needed on `CalcState`:

### Persistent Fields (`#[serde(default)]`)

| Field | Type | Purpose | Serde |
|-------|------|---------|-------|
| `date_format` | `DateFormat` enum (Mdy/Dmy) | MDY vs DMY mode (flag 31 on real hardware) | `#[serde(default)]` -- default Mdy |
| `clock_format` | `ClockFormat` enum (Clk12/Clk24) | 12-hour vs 24-hour display | `#[serde(default)]` -- default Clk24 |
| `accuracy_factor` | `HpNum` | Clock accuracy correction factor (-99.9 to 99.9) | `#[serde(default)]` |
| `alarms` | `Vec<Alarm>` | Alarm catalog (up to 253 per OM) | `#[serde(default)]` |
| `stopwatch_accumulated` | `HpNum` | Accumulated stopwatch time in HH.MMSSss format | `#[serde(default)]` |

### Transient Fields (`#[serde(default, skip)]`)

| Field | Type | Purpose | Serde |
|-------|------|---------|-------|
| `stopwatch_running` | `bool` | Whether the stopwatch is currently counting | `#[serde(default, skip)]` |
| `stopwatch_start` | `Option<std::time::Instant>` | Monotonic start time for current run segment | `#[serde(default, skip)]` |
| `clock_display_mode` | `Option<ClockDisplayMode>` | Active clock display (None/TimeOnly/TimeAndDate) | `#[serde(default, skip)]` |
| `stopwatch_mode` | `bool` | Whether SW mode is active (keyboard remapping) | `#[serde(default, skip)]` |
| `sw_register_pointer` | `Option<u8>` | SWPT target register for split recording | `#[serde(default, skip)]` |
| `alarm_pending` | `Option<usize>` | Index of oldest overdue alarm awaiting acknowledgment | `#[serde(default, skip)]` |

### New Supporting Types

```rust
/// Alarm entry in the catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alarm {
    pub time: HpNum,           // HH.MMSSss
    pub date: HpNum,           // MM.DDYYYY or DD.MMYYYY
    pub repeat_interval: HpNum, // 0 = no repeat; HH.MMSSss interval
    pub message: String,       // ALPHA content (message, label, or function name)
    pub is_control_alarm: bool, // true = >>label (interrupting); false = >label (non-interrupting)
}

/// Date input/output format.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DateFormat { Mdy, Dmy }

/// Clock display format.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ClockFormat { Clk12, Clk24 }

/// Active clock display mode (transient).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClockDisplayMode { TimeOnly, TimeAndDate }
```

---

## Architectural Integration Points

### 1. System Clock Access in hp41-core

`std::time::SystemTime::now()` is called inside `hp41-core` for TIME/DATE operations. This is **not** an I/O violation -- the CLAUDE.md constraint is "println!/eprintln! are forbidden in hp41-core" (console I/O). Reading the system clock is a pure value-returning syscall, analogous to reading the `cancel_requested` `AtomicBool` (which hp41-core already does). No callback, no trait injection needed.

**Calendar conversion pipeline:**
```
SystemTime::now()
  -> duration_since(UNIX_EPOCH) -> total_seconds: u64
  -> unix_to_gregorian(total_seconds) -> (year, month, day, hour, min, sec)
  -> encode as HpNum in MM.DDYYYY or DD.MMYYYY per state.date_format
```

### 2. Stopwatch State Machine

The stopwatch is a three-state machine:
- **Stopped** (`stopwatch_running: false`, `stopwatch_start: None`) -- RCLSW returns `stopwatch_accumulated`
- **Running** (`stopwatch_running: true`, `stopwatch_start: Some(instant)`) -- RCLSW returns `accumulated + elapsed_since_start`
- **Split** (SW mode keyboard action; records current time to register via SWPT pointer)

SETSW sets `stopwatch_accumulated` from X-register. RUNSW captures `Instant::now()` into `stopwatch_start`. STOPSW adds elapsed to `stopwatch_accumulated` and clears `stopwatch_start`. This is a standard monotonic-clock stopwatch pattern.

**`Instant` is not serializable** -- the `stopwatch_start` field carries `#[serde(skip)]`. On save/load, a running stopwatch becomes stopped with accumulated time preserved. This is an acceptable behavioral divergence from real hardware (which has a battery-backed clock chip).

### 3. Alarm Triggering (Frontend Concern)

hp41-core stores and manages the alarm catalog (`Vec<Alarm>`). The **triggering** logic -- checking whether any alarm's time has passed -- is a frontend responsibility:

- **CLI:** Check in the 16ms event-loop tick: `if current_time >= next_alarm_time { set alarm_pending }`
- **GUI:** Check in the auto-save thread or a dedicated alarm-check interval

When an alarm triggers, the frontend calls a new `acknowledge_alarm()` core function that handles the ALPHA display and optional program execution.

### 4. XROM Module Registration

Following the established pattern in `hp41-core/src/ops/math1/xrom.rs`:

```rust
pub const TIME: XromModule = XromModule {
    id: 26,
    name: "TIME 2C",  // CATALOG 2 display string per CX convention
    ops: &[
        ("ADATE", Op::Adate),
        ("ALMCAT", Op::Almcat),
        // ... 31 more entries
    ],
};
```

`xrom_modules` bitfield: bit 0 = Math 1 (XROM 7), bit 1 = Stat 1 (XROM 2), **bit 2 = Time (XROM 26)**. Default becomes `0b0000_0111`. `migrate_after_load()` upgrades v3.1 save files (`0b11` -> `0b111`).

### 5. ModalProgram Extension

The `ModalProgram` enum gains a `Time(TimeStep)` variant for:
- SETIME / SETDATE interactive prompts (if entered from keyboard)
- ALMCAT interactive catalog browsing
- SW stopwatch mode (keyboard remapping)

This follows the exact pattern established by `ModalProgram::Stat1(Stat1Step)` in v3.1.

### 6. HMS Reuse

The Time Pac's HH.MMSSss format is identical to the existing H.MS format in `hms.rs`. The `parse_hms()` and related functions can be reused directly (or extracted to a shared utility). The date format MM.DDYYYY uses the same string-split-at-decimal-point pattern as `parse_counter()` in `program.rs`.

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Date arithmetic | Hand-coded Fliegel-Van Flandern (~60 LOC) | `chrono 0.4.44` | 45K LOC crate for 5 date functions; pulls `iana-time-zone`; violates zero-new-deps discipline |
| Date arithmetic | Hand-coded | `time 0.3.47` | 25K LOC crate; `local-offset` feature has known soundness issues on multithreaded programs; same overdependency concern |
| Date arithmetic | Hand-coded | `julian 0.5` | Small crate but still an unnecessary dep; our needs are simpler than its API |
| System clock | `std::time::SystemTime` | Trait-injection callback from frontend | Over-engineered; SystemTime is no more "I/O" than AtomicBool::load; adds interface complexity for no benefit |
| Stopwatch | `std::time::Instant` (std) | `quanta` / `coarsetime` | Specialized high-perf monotonic clocks; Instant is already nanosecond-precise and sufficient for a calculator stopwatch |
| Alarm scheduling | Frontend polling in event loop | `tokio` / `async-std` / `timer` crate | Massive dep for a simple "check if alarm time passed" comparison; violates no-async invariant |
| Date range | Gregorian 1582-10-15 to 4320-09-10 | Proleptic Gregorian (extend before 1582) | HP OM explicitly states dates start at Oct 15, 1582; following the spec |

---

## What NOT to Add

1. **Do NOT add `chrono` or `time` crate** -- the 5 date functions (DATE+, DDAYS, DOW, DMY, MDY) need ~60 lines of integer arithmetic, not a 25K-45K LOC date library.

2. **Do NOT add any async runtime** -- alarm checking is a simple comparison in the existing synchronous event loop. The CLI already polls at 16ms; the GUI already has a 30s auto-save thread.

3. **Do NOT add a timezone library** -- the HP-41CX Time Module operates in local time with no timezone concept. `SystemTime` gives UTC; we apply a fixed UTC offset (determined once at startup from the OS) or simply use local time via `libc::localtime_r` / Windows `GetLocalTime`. No IANA timezone database needed.

4. **Do NOT inject system-clock access via a trait/callback** -- this was considered and rejected. The `hp41-core` "zero I/O" constraint means no stdout/stderr output, not "no reading the system clock." `std::time::SystemTime::now()` is a pure function that returns a value. The crate already uses `std::sync::Arc<AtomicBool>` which involves cross-thread atomic loads -- a more complex system interaction than reading a clock.

5. **Do NOT persist `Instant`** -- `std::time::Instant` has no serialization and no meaningful value across process restarts. The stopwatch accumulated time is stored as `HpNum` (serializable); the `Instant` start-marker is transient.

6. **Do NOT use `f64` for date/time arithmetic** -- follow the existing ISG/DSE and HMS pattern: string-split at the decimal point for field extraction, HpNum arithmetic for computation. The Fliegel-Van Flandern algorithm uses integer arithmetic exclusively.

---

## Installation

```bash
# No new dependencies to install. The v3.2 Time Pac uses only:
# - std::time (SystemTime, Instant, Duration, UNIX_EPOCH) -- standard library
# - Existing rust_decimal 1.42 for HpNum arithmetic
# - Existing serde/serde_json for persistence
```

---

## Complete XROM 26 Function Catalog

33 functions total (28 from HP 82182A + 5 CX additions):

| # | Mnemonic | Category | Stack/Input | Output | Notes |
|---|----------|----------|-------------|--------|-------|
| 1 | ADATE | Alpha/Date | X=date | ALPHA appended | Format per display setting |
| 2 | ALMCAT | Alarm | -- | Display | Interactive catalog; keyboard remapped |
| 3 | ALMNOW | Alarm | -- | Execute | Activates oldest overdue alarm |
| 4 | ATIME | Alpha/Time | X=time | ALPHA appended | CLK12/CLK24 format |
| 5 | ATIME24 | Alpha/Time | X=time | ALPHA appended | Always 24-hour format |
| 6 | CLK12 | Clock | -- | -- | Sets 12-hour display mode |
| 7 | CLK24 | Clock | -- | -- | Sets 24-hour display mode |
| 8 | CLKT | Clock | -- | Display | Time-only clock display |
| 9 | CLKTD | Clock | -- | Display | Time + date clock display |
| 10 | CLOCK | Clock | -- | Display | Running clock display (alias: ON) |
| 11 | CORRECT | Clock | X=time | -- | Sets time + auto-adjusts accuracy |
| 12 | DATE | Date | -- | X=date | Current date to X-register |
| 13 | DATE+ | Date | Y=date, X=days | X=new date | Date + N days |
| 14 | DDAYS | Date | Y=date1, X=date2 | X=days | Days between two dates |
| 15 | DMY | Date | -- | -- | Sets Day-Month-Year format (flag 31) |
| 16 | DOW | Date | X=date | X=dow (0-6) | 0=Sunday, 6=Saturday |
| 17 | MDY | Date | -- | -- | Sets Month-Day-Year format |
| 18 | RCLAF | Clock | -- | X=factor | Recall accuracy factor |
| 19 | RCLSW | Stopwatch | -- | X=time | Current stopwatch time to X |
| 20 | RUNSW | Stopwatch | -- | -- | Start stopwatch |
| 21 | SETAF | Clock | X=factor | -- | Set accuracy factor (-99.9 to 99.9) |
| 22 | SETDATE | Date | X=date | -- | Set clock date |
| 23 | SETSW | Stopwatch | X=time | -- | Set stopwatch starting time |
| 24 | STOPSW | Stopwatch | -- | -- | Halt stopwatch |
| 25 | SW | Stopwatch | -- | -- | Enter stopwatch mode (keyboard remap) |
| 26 | T+X | Clock | X=delta | -- | Adjust clock by +/-HHHHH.MMSShh |
| 27 | TIME | Clock | -- | X=time | Current time to X-register (HH.MMSSss) |
| 28 | XYZALM | Alarm | Z=repeat, Y=date, X=time, ALPHA=msg | -- | Create alarm |
| 29 | SETIME | Clock | X=time | -- | Set clock time |
| 30 | CLALMA | Alarm (CX) | ALPHA=msg | -- | Clear alarm by ALPHA match |
| 31 | CLALMX | Alarm (CX) | X=alarm# | -- | Clear alarm by index |
| 32 | CLRALMS | Alarm (CX) | -- | -- | Clear all alarms |
| 33 | RCLALM | Alarm (CX) | X=alarm# | Z=repeat, Y=date, X=time, ALPHA=msg | Recall alarm parameters |
| 34 | SWPT | Stopwatch (CX) | X=register# | -- | Set stopwatch split register pointer |

**Note:** The QRC shows SETIME (not SETTIME) as the canonical mnemonic. The CX additions (CLALMA, CLALMX, CLRALMS, RCLALM, SWPT) are at XROM 26 function IDs 31-35.

---

## Local Time Without Timezone Crate

The HP-41CX Time Module operates in local time. Converting `SystemTime` (UTC) to local time without a timezone crate:

**macOS / Linux:** `libc::localtime_r(&timestamp, &mut tm)` -- one function call, returns broken-down local time. The `libc` crate is already a transitive dependency of Rust's standard library on Unix platforms.

**Windows:** `GetLocalTime(&mut SYSTEMTIME)` via `windows-sys` -- already a transitive dependency of `std` on Windows.

**Cross-platform wrapper (~20 lines):**
```rust
fn local_now() -> (i32, u8, u8, u8, u8, u8) {
    // Returns (year, month, day, hour, minute, second) in local time
    // Platform-specific implementation behind cfg(unix) / cfg(windows)
}
```

This avoids any external timezone crate. The HP-41 has no concept of timezone names or DST rules -- it just displays "the current time" as the OS reports it.

**Alternative approach (simpler, recommended):** Use `SystemTime::now()` + `duration_since(UNIX_EPOCH)` to get UTC seconds, then apply a fixed offset computed once at startup:
```rust
// At startup:
let utc_offset_seconds = compute_local_utc_offset(); // via libc or platform API
// At each TIME/DATE call:
let local_secs = utc_secs + utc_offset_seconds;
```

This avoids DST transitions mid-session from producing surprising jumps, which matches the HP-41CX behavior (the hardware clock ran continuously in one timezone).

---

## Sources

- HP 82182A Time Module Quick Reference Card (82182-90002, November 1981) -- read directly from PDF [HIGH confidence]
- HP-41 Module Database, calc.fjk.ch -- XROM 26 confirmed for Time Module [HIGH confidence]
- HP Museum XROM Numbers list, hpmuseum.org/software/xroms.htm -- XROM 25 (X Functions) / XROM 26 (Time) confirmed [HIGH confidence]
- Free42 documentation (thomasokken.com/free42/) -- date range Oct 15, 1582 to Sep 10, 4320; date format MDY/DMY/YMD; function subset confirmed [MEDIUM confidence]
- Fliegel, H.F. & Van Flandern, T.C. (1968), "A Machine Algorithm for Processing Calendar Dates", Communications of the ACM, 11(10):657 [HIGH confidence]
- US Naval Observatory, "Converting Between Julian Dates and Gregorian Calendar Dates", aa.usno.navy.mil/faq/JD_formula [HIGH confidence]
- Rust std::time::SystemTime documentation, doc.rust-lang.org [HIGH confidence]
- Rust std::time::Instant documentation, doc.rust-lang.org [HIGH confidence]
- chrono 0.4.44 documentation, docs.rs/chrono [HIGH confidence -- evaluated and rejected]
- time 0.3.47 documentation, docs.rs/time [HIGH confidence -- evaluated and rejected]
- HP-41CX finseth.com/hpdata/hp41cx.php -- CX-specific time functions confirmed [MEDIUM confidence]
