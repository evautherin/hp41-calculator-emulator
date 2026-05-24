# Phase 38: hp41-core-xrom-framework-clock-date-stopwatch-alarm-core - Research

**Researched:** 2026-05-24
**Domain:** Rust hp41-core — Time Module (XROM 26) core implementation
**Confidence:** HIGH (all locked decisions verified against codebase; algorithms from primary sources)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-38.1:** Direct `std::time::SystemTime::now()` in hp41-core — core calls the system clock directly. SystemTime is a value-returning syscall, not console I/O. The zero-I/O principle applies to console/filesystem/network, not to clock reads.

**D-38.2:** Single `time_offset_secs: i64` persistent CalcState field with `#[serde(default)]`. SETIME computes delta between entered time and current SystemTime, stores as seconds. All time/date reads apply `SystemTime::now() + offset` to derive the "HP-41 time".

**D-38.3:** OS local time, not UTC. Use `libc::localtime_r` (Unix) / `GetLocalTime` (Windows) to convert SystemTime to local. Matches real HP-41CX behavior. Cross-platform via std transitive deps.

**D-38.4:** Interrupting control alarms DEFERRED as a documented divergence. Message alarms + non-interrupting control alarms are fully in scope.

**D-38.5:** Non-interrupting control alarms XEQ the stored program label on acknowledgment. Uses existing XEQ dispatch infrastructure, no re-entrancy hazard.

**D-38.6:** Freeze elapsed time on save. On save: compute `elapsed = Instant::elapsed() + accumulated`, store as persistent `stopwatch_accumulated: f64`. On load: stopwatch is STOPPED with accumulated time preserved. User must RUNSW to resume.

**D-38.7:** Three separate CalcState fields: `stopwatch_mode: StopwatchMode` (enum Idle/Running/Stopped, persistent `#[serde(default)]`), `stopwatch_accumulated: f64` (persistent `#[serde(default)]`), `stopwatch_start: Option<Instant>` (transient `#[serde(default, skip)]`). Split-point time stored as `stopwatch_split: f64` (persistent `#[serde(default)]`).

**D-38.8:** Typed `AlarmType` enum: `Message(String)` carries the ALPHA message, `Control { label: String, interrupting: bool }` carries the program label. Forward-compatible for when interrupting alarms get implemented.

**D-38.9:** `check_alarms()` called by frontend after every dispatch via the drain pattern (same as `print_buffer` and `event_buffer`). Alarm notifications pushed into `event_buffer` for frontend rendering.

**D-38.10:** Direct `alarms: Vec<AlarmEntry>` on CalcState with `#[serde(default)]`. Free functions in `alarm.rs` operate on `&mut Vec<AlarmEntry>`.

**D-38.11:** Repeat interval stored as `i64` seconds. Convert from HH.MMSSss to seconds once at XYZALM entry time. No rounding drift risk over repeated reschedules.

**D-carried.1:** Zero new runtime dependencies — `std::time::{SystemTime, Instant}` + hand-coded Fliegel-Van Flandern JDN (~60 LOC). `chrono`/`time` crates rejected.

**D-carried.2:** Date decimal parsing uses string-split-at-decimal, never float arithmetic. Left-pad fractional part to exactly 6 chars.

**D-carried.3:** Flag 31 = sole DMY/MDY control. `DMY` → `SF 31`, `MDY` → `CF 31`. No separate `date_format` field on CalcState.

**D-carried.4:** `ModalProgram::Time(TimeStep)` variant following the Stat 1 pattern (ADR-v3.1-005). `TimeStep` enum lives in new `time/modal.rs` (outside math1/ freeze boundary).

**D-carried.5:** XROM ID 26 with bit-2 arm in `xrom_resolve`. `default_xrom_modules() → 0b0000_0111`. `migrate_after_load()` upgrades v3.1 saves (`0b0000_0011` → `0b0000_0111`).

**D-carried.6:** Module tree: `hp41-core/src/ops/time/` as sibling to `math1/` and `stat1/`. Free42 disclaim header on all files.

**D-carried.7:** "Pull on redraw" architecture for live display — hp41-core remains thread-free and async-free. Frontend redraw cycle computes time-dependent display state on-demand.

### Claude's Discretion

None recorded — decisions above fully capture implementation direction.

### Deferred Ideas (OUT OF SCOPE)

- Interrupting control alarm execution (call-stack re-entrancy not supported; data model supports it but execution deferred).
- Cycle-accurate crystal oscillator simulation — no user value.
- HP-IL alarm wake-up — no OFF state in emulator; HP-IL permanently excluded.
- Accuracy factor correction loop — host OS clock is NTP-synchronized; SETAF/RCLAF store value, CORRECT is documented no-op divergence.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TIME-FW-01 | TIME_MODULE XromModule registered with XROM ID 26 and ~33 ops; xrom_resolve bit-2 arm fires LAST | Pattern: extend `MATH_1`/`STAT_1` const + add `time_resolve()` + bit-2 arm in `xrom_resolve()` in `math1/xrom.rs` |
| TIME-FW-02 | `default_xrom_modules()` returns `0b0000_0111`; `migrate_after_load()` upgrades v3.1 save files | Pattern: existing `migrate_after_load()` at `state.rs:386`; change `default_xrom_modules()` return value |
| TIME-FW-03 | New CalcState fields follow serde invariants | Pattern: all existing fields in `state.rs`; see Serde Shape Decision Matrix in this doc |
| TIME-FW-04 | System clock access via `std::time::SystemTime` in hp41-core; time offset stored as persistent field | D-38.1 + D-38.2 locked; `time_offset_secs: i64` field |
| TIME-FW-05 | Stopwatch elapsed time tracked via `std::time::Instant`; start marker transient, accumulated time persistent | D-38.6 + D-38.7 locked; three-field design |
| TIME-FW-06 | Alarm catalog stored as persistent CalcState field (`Vec<AlarmEntry>`, up to 253) | D-38.10 locked; `alarms: Vec<AlarmEntry>` with `#[serde(default)]` |
| TIME-CLK-01 | TIME returns HH.MMSSss from system clock + offset | `clock.rs::op_time()` — reads `SystemTime::now()`, applies `time_offset_secs`, converts via local time |
| TIME-CLK-02 | DATE returns MM.DDYYYY or DD.MMYYYY per Flag 31 | `clock.rs::op_date()` — reads local date, formats per `state.flags & (1 << 31)` |
| TIME-CLK-03 | ATIME appends time to ALPHA in CLK12/CLK24 format | `alpha_time.rs::op_atime()` — parse HH.MMSSss from X, format string, append to `state.alpha_reg` |
| TIME-CLK-04 | ATIME24 always 24-hour regardless of CLK12/CLK24 | `alpha_time.rs::op_atime24()` — same as ATIME but ignores `clock_12h` field |
| TIME-CLK-05 | ADATE appends current date to ALPHA | `alpha_time.rs::op_adate()` — format date from X per Flag 31 |
| TIME-CLK-06 | T+X adds X register value (seconds) to time accumulator | `clock.rs::op_tplusx()` — adjust `time_offset_secs` by parsed HH.MMSSss, handle midnight rollover |
| TIME-DAT-01 | DATE+ adds X days to date in Y | `date_arith.rs::op_date_plus()` — JDN-based: parse Y as date → JDN, add X, convert back |
| TIME-DAT-02 | DDAYS computes days between X and Y dates | `date_arith.rs::op_ddays()` — JDN(X) - JDN(Y), signed |
| TIME-DAT-03 | DOW returns day-of-week 0=Sun..6=Sat for date in X | `date_arith.rs::op_dow()` — (JDN + 1) % 7 formula |
| TIME-DAT-04 | DMY sets Flag 31 | Trivial: `state.flags |= 1 << 31` |
| TIME-DAT-05 | MDY clears Flag 31 | Trivial: `state.flags &= !(1 << 31)` |
| TIME-DAT-06 | Date decimal format uses string-split-at-decimal parsing | D-carried.2 locked; same as ISG/DSE `parse_counter()` |
| TIME-DSP-01 | CLKT toggles clock display mode (time-only) | Sets `clock_display_mode` field; `clock_active` transient flag (D-carried.7) |
| TIME-DSP-02 | CLKTD toggles time-and-date display within clock mode | Sets `ClockDisplayMode::TimeAndDate` variant |
| TIME-DSP-03 | SETIME prompts for time via modal (HH.MMSSss) | `modal.rs::TimeStep::SetTimePrompt`; on submit: parse, compute offset, store |
| TIME-DSP-04 | SETDATE prompts for date via modal (per Flag 31) | `modal.rs::TimeStep::SetDatePrompt`; on submit: parse, compute date offset |
| TIME-DSP-05 | Clock display updates at ≥1 Hz in CLI and GUI when active | hp41-core scope: exposes `get_clock_display_str()` helper; frontend owns refresh cadence (D-carried.7) |
| TIME-FMT-01 | CLK12 sets 12-hour format | Sets `clock_12h: bool` field |
| TIME-FMT-02 | CLK24 sets 24-hour format | Clears `clock_12h: bool` field |
| TIME-FMT-03 | SETAF stores accuracy factor from X | Sets `accuracy_factor: HpNum` field; no-op divergence |
| TIME-FMT-04 | RCLAF recalls accuracy factor to X | Reads `accuracy_factor` field |
| TIME-FMT-05 | CORRECT adjusts time by accuracy factor (no-op divergence) | Stores factor, documented no-op; same as SETIME logic |
| TIME-SW-01 | SETSW initializes stopwatch with split-point tracking | `stopwatch.rs::op_setsw()` — parse X as HH.MMSSss centiseconds, write to `stopwatch_accumulated` |
| TIME-SW-02 | SW starts interactive stopwatch mode | Sets `stopwatch_keyboard_mode: bool` transient flag; Phase 39 (CLI) and Phase 41 (GUI) wire the key remapping |
| TIME-SW-03 | STPW records a split point | `stopwatch.rs::op_stpw()` — compute current elapsed, store in `stopwatch_split` |
| TIME-SW-04 | RUNSW starts/resumes stopwatch timer | `stopwatch.rs::op_runsw()` — set `stopwatch_mode = Running`, capture `Instant::now()` in transient `stopwatch_start` |
| TIME-SW-05 | STOPSW stops the stopwatch timer | `stopwatch.rs::op_stopsw()` — add elapsed to `stopwatch_accumulated`, clear `stopwatch_start` |
| TIME-SW-06 | RCLSW recalls current elapsed time to X as HH.MMSSss | `stopwatch.rs::op_rclsw()` — compute elapsed = `accumulated + (now - start)` if running, else accumulated |
| TIME-SW-07 | SWPT recalls last split-point time to X | `stopwatch.rs::op_swpt()` — push `stopwatch_split` |
| TIME-SW-08 | Stopwatch display updates at ≥10 Hz when running | hp41-core scope: exposes `get_stopwatch_display_str()` helper; frontend owns cadence |
| TIME-SW-09 | Stopwatch uses monotonic `Instant` | D-38.7 locked; `stopwatch_start: Option<Instant>` is `#[serde(default, skip)]` |
| TIME-ALM-01 | XYZALM sets alarm from X/Y/Z/ALPHA | `alarm.rs::op_xyzalm()` — parse all four stack params; detect alarm type from ALPHA prefix |
| TIME-ALM-02 | RCLALM recalls alarm fields by number from X | `alarm.rs::op_rclalm()` — push fields back to stack, set ALPHA |
| TIME-ALM-03 | ALMCAT enters interactive Alarm Catalog mode | Sets `alarm_catalog_mode: bool` transient flag; Phase 39/41 wire the keyboard remapping |
| TIME-ALM-04 | CLALMA clears alarm by ALPHA match | `alarm.rs::op_clalma()` — find entry where message matches `state.alpha_reg`, remove |
| TIME-ALM-05 | CLALMX clears alarm by number from X | `alarm.rs::op_clalmx()` — 1-indexed removal |
| TIME-ALM-06 | CLRALMS clears all alarms | `alarm.rs::op_clralms()` — `state.alarms.clear()` |
| TIME-ALM-07 | ALMNOW triggers oldest past-due alarm | `alarm.rs::op_almnow()` — find first entry with `past_due = true`, trigger |
| TIME-ALM-08 | Past-due detection fires on each dispatch | `alarm.rs::check_alarms()` — called by frontend drain loop; pushes to `event_buffer` |
| TIME-ALM-09 | Message alarms display message in ALPHA and beep | On trigger: push message to `print_buffer`/`event_buffer`, beep event |
| TIME-ALM-10 | Control alarms trigger program execution (non-interrupting only in v3.2) | On trigger: push XEQ event into `event_buffer` for frontend to dispatch; interrupting deferred |
| TIME-ALM-11 | Repeating alarms reschedule after acknowledgment | `alarm.rs::acknowledge_alarm()` — if `repeat_secs > 0`, add repeat to trigger time |
| TIME-ALM-12 | Alarm catalog persists across save/load | `alarms: Vec<AlarmEntry>` with `#[serde(default)]`; `AlarmEntry` is `Serialize + Deserialize` |
</phase_requirements>

---

## Summary

Phase 38 delivers all ~33 Time Pac `Op` variants in `hp41-core` without touching the CLI or GUI (those are Phases 39 and 41). The implementation mirrors the established Stat 1 Pac pattern from Phase 33: register a new `XromModule` const, add a bit-2 resolver arm, extend `ModalProgram` with a `Time(TimeStep)` variant, add new `CalcState` fields with correct serde annotations, and implement all operations in a 7-file module tree under `hp41-core/src/ops/time/`.

The three technically novel aspects (relative to Phases 33-37) are: (1) system clock reads via `std::time::SystemTime::now()` directly in core — locked as D-38.1; (2) stopwatch timing via `std::time::Instant` with freeze-on-save semantics — locked as D-38.6; and (3) the alarm catalog — a `Vec<AlarmEntry>` with past-due detection triggered by a `check_alarms()` drain function. All are implementable without new runtime dependencies.

The 4-way exhaustive-match invariant applies: all ~33 Op variants must be added to `dispatch()` and `execute_op()` in this phase, with CLI and GUI `prgm_display.rs` arms intentionally left causing a sanctioned CI break until Phases 39 and 41. The Free42 contamination guard script must be extended to cover the new `time/` directory before any algorithm code lands.

**Primary recommendation:** Implement in plan-wave order: (Wave 0) scaffolding — Op enum + xrom_resolve bit-2 + CalcState fields + migrate_after_load + free42 guard extension; (Wave 1) JDN date arithmetic + date format parsing; (Wave 2) clock operations; (Wave 3) stopwatch state machine; (Wave 4) alarm catalog; (Wave 5) modal prompts for SETIME/SETDATE/XYZALM.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| System clock read (TIME/DATE) | hp41-core | — | D-38.1 locks direct std::time call in core |
| Time offset storage (SETIME/SETDATE) | hp41-core CalcState | — | `time_offset_secs: i64` field |
| Local time conversion (localtime_r/GetLocalTime) | hp41-core | — | D-38.3; already a transitive dep |
| JDN date arithmetic (DATE+/DDAYS/DOW) | hp41-core ops/time/date_arith.rs | — | Pure integer algorithm; no I/O |
| Flag 31 (DMY/MDY) | hp41-core flags system | — | D-carried.3; existing `state.flags` bitfield |
| Stopwatch elapsed time (Instant) | hp41-core | — | D-38.7 transient `stopwatch_start` field |
| Alarm catalog (Vec<AlarmEntry>) | hp41-core CalcState | — | D-38.10; persisted with `#[serde(default)]` |
| Alarm past-due detection (check_alarms) | hp41-core (function) | Frontend (caller) | D-38.9 drain pattern |
| Live clock/stopwatch display refresh | Frontend (CLI/GUI) | — | D-carried.7 pull-on-redraw; hp41-core provides getter |
| SETIME/SETDATE/XYZALM modal prompts | hp41-core modal state machine | — | D-carried.4 ModalProgram::Time(TimeStep) |
| 4-way exhaustive match items 1+2 | hp41-core dispatch/execute_op | — | Phase 38 scope |
| 4-way exhaustive match items 3+4 | hp41-cli (Phase 39) / hp41-gui (Phase 41) | — | Sanctioned CI break in this phase |

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `std::time::SystemTime` | std | Host clock reads (TIME/DATE/SETIME) | Zero-dep, cross-platform, value-returning syscall; D-38.1 |
| `std::time::Instant` | std | Monotonic stopwatch timer | Immune to system clock jumps; already used in hp41-cli auto-save |
| `std::time::Duration` | std | Duration arithmetic (elapsed, offset conversion) | Pairs with Instant and SystemTime |
| `rust_decimal` | 1.42 (existing) | HpNum arithmetic for all time/date value representation | Existing dep; no gaps |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `libc` (transitive) | N/A | `localtime_r` on Unix for local time decomposition | Used in `clock.rs` Unix target; already transitive via std |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-coded JDN (~60 LOC) | `chrono` crate | chrono = 45K LOC + iana-time-zone; no new runtime deps per D-carried.1 |
| `std::time::SystemTime` | Frontend injection | Injection pattern (see P34 in PITFALLS.md) adds coupling; D-38.1 locks direct syscall |
| `std::time::Instant` | `f64` elapsed | Instant is monotonic; f64 accumulation has precision loss over hours |

**Installation:** No new packages. Zero new runtime dependencies confirmed by D-carried.1.

---

## Package Legitimacy Audit

> No new packages. Zero new runtime dependencies are introduced in Phase 38.

| Package | Registry | Age | Downloads | Source Repo | slopcheck | Disposition |
|---------|----------|-----|-----------|-------------|-----------|-------------|
| (none) | — | — | — | — | — | — |

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

---

## Architecture Patterns

### System Architecture Diagram

```
User keypress / programmatic dispatch
           |
           v
    dispatch() in ops/mod.rs
           |
           +-- xrom_resolve(name, xrom_modules) -- bit-2 set? --> time_resolve(name)
           |                                                              |
           |                                                      Op::Time(variant)
           |
    execute_op() in ops/program.rs
           |
    ┌──────┴──────────────────────────────┐
    |                                     |
 clock.rs          date_arith.rs       alarm.rs        stopwatch.rs
 op_time()         op_date_plus()      op_xyzalm()     op_runsw()
 op_date()         op_ddays()          op_almnow()     op_stopsw()
 op_setime()       op_dow()            op_clralms()    op_rclsw()
 op_setdate()      DMY/MDY →           op_clalma()     op_setsw()
 op_tplusx()       state.flags bit 31  op_clalmx()     op_swpt()
 op_setaf()                            op_rclalm()     op_stpw()
 op_rclaf()                            op_almcat()
 op_correct()                          op_almnow()
           |
    alpha_time.rs        modal.rs
    op_adate()           TimeStep enum
    op_atime()           SETIME modal
    op_atime24()         SETDATE modal
                         XYZALM modal
           |
    CalcState fields (new in Phase 38):
    time_offset_secs: i64       [serde(default)]
    clock_12h: bool             [serde(default)]
    clock_display_mode: ClockDisplayMode  [serde(default)]
    accuracy_factor: HpNum      [serde(default)]
    alarms: Vec<AlarmEntry>     [serde(default)]
    stopwatch_mode: StopwatchMode  [serde(default)]
    stopwatch_accumulated: f64  [serde(default)]
    stopwatch_split: f64        [serde(default)]
    stopwatch_start: Option<Instant>  [serde(default, skip)]
    clock_active: bool          [serde(default, skip)]
    stopwatch_keyboard_mode: bool  [serde(default, skip)]
    alarm_catalog_mode: bool    [serde(default, skip)]
           |
    Frontend drain pattern (after each dispatch):
    check_alarms(&mut state) → event_buffer entries
    drain print_buffer (existing)
    drain event_buffer (existing)
```

### Recommended Project Structure
```
hp41-core/src/ops/time/     (new, sibling to math1/ and stat1/)
├── mod.rs                  # module header, OM storage register consts (if any), pub re-exports
├── clock.rs                # TIME, DATE, SETIME, SETDATE, CORRECT, T+X, SETAF, RCLAF, CLK12, CLK24, CLKT, CLKTD
├── date_arith.rs           # JDN algorithms, DATE+, DDAYS, DOW, DMY, MDY, parse_date(), parse_time()
├── alpha_time.rs           # ADATE, ATIME, ATIME24
├── alarm.rs                # AlarmEntry struct, AlarmType enum, XYZALM, RCLALM, ALMCAT, CLALMA,
│                           # CLALMX, CLRALMS, ALMNOW, check_alarms()
├── stopwatch.rs            # StopwatchMode enum, RUNSW, STOPSW, SETSW, RCLSW, SWPT, STPW
└── modal.rs                # TimeStep enum (outside math1/ freeze); SETIME/SETDATE/XYZALM prompt steps
```

### Pattern 1: XROM Module Registration (TIME_MODULE const)

Follow `MATH_1` / `STAT_1` const pattern in `math1/xrom.rs`:

```rust
// Source: hp41-core/src/ops/math1/xrom.rs (MATH_1 const pattern)
// This is the THIRD freeze exception to math1/xrom.rs (after STAT_1 at Phase 33).
// Document in xrom.rs comment header alongside D-33.3 + D-33.3b references.
pub const TIME_MODULE: XromModule = XromModule {
    id: 26,
    name: "TIME 2C",     // HP-41CX CX ROM variant display string
    ops: &[
        ("ADATE",   Op::TimeAdate),
        ("ALMCAT",  Op::TimeAlmcat),
        ("ALMNOW",  Op::TimeAlmnow),
        ("ATIME",   Op::TimeAtime),
        ("ATIME24", Op::TimeAtime24),
        ("CLK12",   Op::TimeClk12),
        ("CLK24",   Op::TimeClk24),
        ("CLKT",    Op::TimeClkt),
        ("CLKTD",   Op::TimeClktd),
        ("CLOCK",   Op::TimeClock),
        ("CORRECT", Op::TimeCorrect),
        ("DATE",    Op::TimeDate),
        ("DATE+",   Op::TimeDatePlus),
        ("DDAYS",   Op::TimeDdays),
        ("DMY",     Op::TimeDmy),
        ("DOW",     Op::TimeDow),
        ("MDY",     Op::TimeMdy),
        ("RCLAF",   Op::TimeRclaf),
        ("RCLALM",  Op::TimeRclalm),
        ("RCLSW",   Op::TimeRclsw),
        ("RUNSW",   Op::TimeRunsw),
        ("SETAF",   Op::TimeSetaf),
        ("SETDATE", Op::TimeSetdate),
        ("SETIME",  Op::TimeSetime),
        ("SETSW",   Op::TimeSetsw),
        ("STOPSW",  Op::TimeStopsw),
        ("SW",      Op::TimeSw),
        ("T+X",     Op::TimeTplusx),
        ("TIME",    Op::TimeTime),
        ("XYZALM",  Op::TimeXyzalm),
        ("CLALMA",  Op::TimeClalma),
        ("CLALMX",  Op::TimeClalmx),
        ("CLRALMS", Op::TimeClralms),
        ("SWPT",    Op::TimeSwpt),
        // STPW (split) — HP-41CX extension
        ("STPW",    Op::TimeStpw),
    ],
};
```

[ASSUMED] — Op variant names above follow the `Time` prefix convention for disambiguation. Actual names decided at implementation time. Count: 35 ops (including STPW).

### Pattern 2: xrom_resolve Bit-2 Extension

```rust
// Source: hp41-core/src/ops/math1/xrom.rs xrom_resolve() (existing, extend here)
// D-carried.5: bit-2 arm fires LAST (after bit-0 Math 1, bit-1 Stat 1)
pub fn xrom_resolve(name: &str, modules: u8) -> Option<Op> {
    if modules & 0b0000_0001 != 0 {
        if let Some(op) = math1_resolve(name) { return Some(op); }
    }
    if modules & 0b0000_0010 != 0 {
        if let Some(op) = stat1_resolve(name) { return Some(op); }
    }
    // Phase 38 (v3.2): Time Module bit-2 arm — fires LAST per Pitfall 1.
    if modules & 0b0000_0100 != 0 {
        if let Some(op) = time_resolve(name) { return Some(op); }
    }
    None
}
```

[VERIFIED: codebase] — pattern confirmed from existing `xrom_resolve` in `hp41-core/src/ops/math1/xrom.rs`.

### Pattern 3: CalcState Migration

```rust
// Source: hp41-core/src/state.rs migrate_after_load() (existing, extend)
pub fn migrate_after_load(&mut self) {
    // v3.0 → v3.1: ensure STAT_1 bit (bit 1) is set.
    if self.xrom_modules & 0b0000_0010 == 0 {
        self.xrom_modules |= 0b0000_0010;
    }
    // v3.1 → v3.2: ensure TIME_MODULE bit (bit 2) is set.
    if self.xrom_modules & 0b0000_0100 == 0 {
        self.xrom_modules |= 0b0000_0100;
    }
}
```

[VERIFIED: codebase] — `migrate_after_load()` at `state.rs:386`; idempotent bitwise OR pattern.

### Pattern 4: ModalProgram::Time Extension

```rust
// Source: hp41-core/src/ops/math1/modal.rs ModalProgram enum (existing, extend)
// D-carried.4: follow ADR-v3.1-005 exactly.
// This is the THIRD freeze exception to math1/modal.rs (alongside Stat1 D-33.3b).
pub enum ModalProgram {
    // ... existing variants ...
    Stat1(crate::ops::stat1::modal::Stat1Step),
    // Phase 38 (v3.2): Time Pac modal prompts — SETIME, SETDATE, XYZALM.
    Time(crate::ops::time::modal::TimeStep),
}

impl ModalProgram {
    pub fn current_prompt(&self) -> Option<String> {
        match self {
            // ... existing arms ...
            ModalProgram::Stat1(step) => crate::ops::stat1::modal::current_prompt(step),
            // Phase 38:
            ModalProgram::Time(step) => crate::ops::time::modal::current_prompt(step),
        }
    }

    pub fn requires_alpha_label(&self) -> bool {
        match self {
            // ... existing arms ...
            ModalProgram::Stat1(step) => crate::ops::stat1::modal::requires_alpha_label(step),
            // Phase 38:
            ModalProgram::Time(step) => crate::ops::time::modal::requires_alpha_label(step),
        }
    }
}
```

[VERIFIED: codebase] — `ModalProgram` at `math1/modal.rs:24`; `Stat1` variant at line 51 is the template.

### Pattern 5: JDN Fliegel-Van Flandern Algorithm

The Fliegel-Van Flandern Julian Day Number algorithm from ACM Communications 1968 is a pure integer transformation. It covers the Gregorian calendar from Oct 15, 1582 through Sep 10, 4320. [CITED: Fliegel, H.F. & Van Flandern, T.C. (1968), "A Machine Algorithm for Processing Calendar Dates", Commun. ACM 11(10):657]

```rust
// Source: Fliegel-Van Flandern (1968), ACM Commun. 11(10):657
// Algorithm independently re-derived from primary source; Free42 source
// consulted only as sanity-check oracle, not copied.

/// Convert (year, month, day) to Julian Day Number.
/// Valid for Gregorian dates from Oct 15, 1582.
fn date_to_jdn(year: i32, month: i32, day: i32) -> i64 {
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;
    (day + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045) as i64
}

/// Convert Julian Day Number back to (year, month, day).
fn jdn_to_date(jdn: i64) -> (i32, i32, i32) {
    let l = jdn + 68569;
    let n = 4 * l / 146097;
    let l = l - (146097 * n + 3) / 4;
    let i = 4000 * (l + 1) / 1461001;
    let l = l - 1461 * i / 4 + 31;
    let j = 80 * l / 2447;
    let day = l - 2447 * j / 80;
    let l = j / 11;
    let month = j + 2 - 12 * l;
    let year = 100 * (n - 49) + i + l;
    (year as i32, month as i32, day as i32)
}

/// Day of week: 0=Sunday, 1=Monday, ..., 6=Saturday.
fn jdn_to_dow(jdn: i64) -> i32 {
    ((jdn + 1) % 7) as i32  // JDN 0 was Monday; JDN+1 shifts to Sunday=0
}
```

[CITED: Fliegel & Van Flandern 1968] — algorithm is public domain; standard textbook implementation.

### Pattern 6: Date Decimal Parsing (ISG/DSE precedent)

```rust
// Source: hp41-core/src/ops/program.rs parse_counter() — the established ISG/DSE pattern.
// D-carried.2 / P35: NEVER use float arithmetic for field extraction.

/// Parse HP-41 date decimal (MM.DDYYYY in MDY, DD.MMYYYY in DMY) to (year, month, day).
/// `dmy` = true means Flag 31 is set (Day-Month-Year format).
fn parse_date_hpnum(hpnum: &HpNum, dmy: bool) -> Result<(i32, i32, i32), HpError> {
    let s = hpnum.to_string();
    let (int_part, frac_part) = s.split_once('.').unwrap_or((&s, ""));
    let first = int_part.parse::<u8>().map_err(|_| HpError::Data)?;
    // Left-pad fractional part to exactly 6 digits (preserves leading zeros).
    let padded = format!("{:0>6}", frac_part);
    let second = padded[..2].parse::<u8>().map_err(|_| HpError::Data)?;
    let year = padded[2..6].parse::<u16>().map_err(|_| HpError::Data)? as i32;
    let (month, day) = if dmy {
        (second as i32, first as i32)  // DD.MMYYYY → day=first, month=second
    } else {
        (first as i32, second as i32)  // MM.DDYYYY → month=first, day=second
    };
    Ok((year, month, day))
}
```

[VERIFIED: codebase] — `parse_counter()` string-split pattern confirmed at `program.rs`.

### Pattern 7: AlarmEntry Data Structure

```rust
// D-38.8 + D-38.10 locked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmEntry {
    /// Trigger time as seconds since midnight of the trigger date.
    /// Computed once at XYZALM time from HH.MMSSss + date, stored as i64
    /// Unix timestamp (or relative seconds offset for easier comparison).
    pub trigger_unix: i64,
    /// Repeat interval in seconds (0 = one-shot). D-38.11.
    pub repeat_secs: i64,
    /// Alarm type and payload.
    pub alarm_type: AlarmType,
    /// Whether this alarm has become past-due (trigger time passed without acknowledgment).
    pub past_due: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlarmType {
    /// Message text displayed in ALPHA register on trigger.
    Message(String),
    /// Program or function label to execute on trigger.
    Control {
        label: String,
        /// If true: interrupting (deferred in v3.2). If false: non-interrupting (implemented).
        interrupting: bool,
    },
}
```

[ASSUMED] — D-38.8 specifies the shape; exact field names chosen here follow codebase style. `trigger_unix` as seconds avoids timezone confusion for the comparison in `check_alarms()`.

### Pattern 8: Time Format HH.MMSSss Parsing (P44 — new, not reuse hms.rs)

```rust
// P44 mitigation: HH.MMSSss has 6 fractional digits (MM, SS, ss = centiseconds).
// hp41-core/src/ops/hms.rs parse_hms() handles H.MMSS (4 fractional digits).
// Do NOT reuse parse_hms() — centiseconds would be silently truncated.

/// Parse HH.MMSSss to (hours, minutes, seconds, centiseconds).
fn parse_time_hpnum(hpnum: &HpNum) -> Result<(u8, u8, u8, u8), HpError> {
    let s = hpnum.to_string();
    let (int_part, frac_part) = s.split_once('.').unwrap_or((&s, ""));
    let hours = int_part.parse::<u8>().map_err(|_| HpError::Data)?;
    let padded = format!("{:0>6}", frac_part);  // 6 digits: MMSS cc
    let minutes = padded[..2].parse::<u8>().map_err(|_| HpError::Data)?;
    let seconds = padded[2..4].parse::<u8>().map_err(|_| HpError::Data)?;
    let centis = padded[4..6].parse::<u8>().map_err(|_| HpError::Data)?;
    if hours > 23 || minutes > 59 || seconds > 59 || centis > 99 {
        return Err(HpError::Data);
    }
    Ok((hours, minutes, seconds, centis))
}
```

[ASSUMED] — implementation shape; mirrors ISG/DSE string-split pattern.

### Pattern 9: check_alarms() Drain Pattern

```rust
// D-38.9: called by frontend after every dispatch (same pattern as print_buffer drain).
// Returns nothing; pushes alarm events into state.event_buffer.

pub fn check_alarms(state: &mut CalcState) {
    let now_unix = current_unix_timestamp(); // std::time::SystemTime + time_offset_secs
    let mut triggered = Vec::new();
    for (idx, alarm) in state.alarms.iter_mut().enumerate() {
        if !alarm.past_due && alarm.trigger_unix <= now_unix {
            alarm.past_due = true;
            triggered.push(idx);
        }
    }
    for idx in triggered {
        let alarm = &state.alarms[idx];
        match &alarm.alarm_type {
            AlarmType::Message(msg) => {
                state.event_buffer.push(format!("alarm:message:{}", msg));
                state.print_buffer.push(msg.clone());
            }
            AlarmType::Control { label, interrupting: false } => {
                state.event_buffer.push(format!("alarm:xeq:{}", label));
            }
            AlarmType::Control { interrupting: true, .. } => {
                // Deferred per D-38.4; treat as past-due but do not execute.
                state.event_buffer.push("alarm:interrupting:deferred".to_string());
            }
        }
    }
}
```

[ASSUMED] — event format strings are implementation detail; the pattern is confirmed from D-38.9 and the existing drain infrastructure.

### Anti-Patterns to Avoid

- **Reusing `hms.rs::parse_hms()` for time values:** `hms.rs` handles `H.MMSS` (4 fractional digits); Time Module needs `HH.MMSSss` (6 fractional digits). Centiseconds would be silently truncated. Write a new `parse_time_hpnum()`. (P44)
- **Adding `date_format: DateFormat` field to CalcState:** Flag 31 IS the date format. All date functions check `state.flags & (1 << 31)` directly. (P41 / D-carried.3)
- **Using float arithmetic for date parsing:** `MM.DDYYYY` as f64 loses trailing zeros (year 2000 → `1.012` instead of `1.012000`). Always string-split. (P35)
- **Calling `chrono` or `time` crates:** zero new runtime deps. (D-carried.1)
- **Adding any Time Module code to `math1/` files beyond the three sanctioned exceptions:** `xrom.rs` (TIME_MODULE const + time_resolve + bit-2 arm), `modal.rs` (Time variant + 3 dispatch arms). All semantics live in `time/`.
- **Serializing `stopwatch_start: Option<Instant>`:** `Instant` is not serializable. Field MUST be `#[serde(default, skip)]`. Use accumulated centiseconds for persistence. (P33 / D-38.7)
- **Persisting `clock_active`, `stopwatch_keyboard_mode`, `alarm_catalog_mode`:** all three are transient UI state; all must be `#[serde(default, skip)]`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Gregorian calendar arithmetic | Custom date math | Hand-coded JDN (Fliegel-Van Flandern 1968, ~60 LOC) | Textbook algorithm; proven correct; no dependency |
| Monotonic elapsed time | `f64` accumulation | `std::time::Instant` | Immune to clock adjustments; nanosecond precision |
| Local time decomposition | Manual timezone offset | `libc::localtime_r` (Unix) / `GetLocalTime` (Windows) | Already transitive dep; handles DST correctly |
| Serde for alarm catalog | Custom binary format | Standard `#[derive(Serialize, Deserialize)]` | Existing infrastructure; autosave.json pattern |

**Key insight:** The ONLY genuinely new algorithm in Phase 38 is the ~60-LOC Fliegel-Van Flandern JDN. Everything else uses existing infrastructure.

---

## Common Pitfalls

### Pitfall 1: Reusing `parse_hms()` for Time Values (P44)
**What goes wrong:** `hms.rs::parse_hms()` expects `H.MMSS` (4 fractional digits). Time Module uses `HH.MMSSss` (6 fractional digits). Calling `parse_hms()` silently drops centiseconds.
**Why it happens:** The field names look similar; H.MMSS and HH.MMSSss are visually close.
**How to avoid:** Write a dedicated `parse_time_hpnum()` in `date_arith.rs` or `clock.rs`. Never reference `hms.rs` from `time/`.
**Warning signs:** `RCLSW` returns 0 centiseconds even when stopwatch was running for fractional seconds.

### Pitfall 2: Wrong Flag 31 Semantics (P41)
**What goes wrong:** A separate `date_format: DateFormat` field is added to CalcState, and `DMY`/`MDY` ops write to it. Then `SF 31` doesn't affect date formatting and `DMY` doesn't set the flag. The two systems drift.
**Why it happens:** It feels cleaner to have an enum; the flag 31 connection is non-obvious.
**How to avoid:** D-carried.3 is locked. Implement `Op::TimeDmy` as `state.flags |= 1 << 31`, `Op::TimeMdy` as `state.flags &= !(1 << 31)`. All date format checks read `state.flags & (1 << 31)` directly.
**Warning signs:** `SF 31` + `DATE` still returns MDY format.

### Pitfall 3: Date Decimal Trailing Zero Loss (P35)
**What goes wrong:** `HH.MMYYYY` where YYYY ends in `0` (e.g., year 2000: `1.012000`) loses trailing zeros when stored as `rust_decimal`. Parsing as f64 and extracting digits gives wrong year.
**Why it happens:** Float representation drops trailing zeros.
**How to avoid:** Use `hpnum.to_string()` then string-split. Left-pad fractional part to exactly 6 chars with `format!("{:0>6}", frac)`.
**Warning signs:** `DATE` returns wrong year for dates in year 2000, 2010, 2020 (any year with trailing zeros).

### Pitfall 4: Stopwatch `Instant` Serialization (P33 / D-38.7)
**What goes wrong:** `stopwatch_start: Option<std::time::Instant>` is added to CalcState without `#[serde(skip)]`. `Instant` does not implement `Serialize`. Compilation fails.
**Why it happens:** Forgetting that `Instant` is not serializable (unlike `SystemTime` which has OS-specific serialization support).
**How to avoid:** `stopwatch_start: Option<Instant>` MUST be `#[serde(default, skip)]`. Accumulated centiseconds go in `stopwatch_accumulated: f64` which IS persistent.
**Warning signs:** Compile error "the trait `Serialize` is not implemented for `std::time::Instant`".

### Pitfall 5: Free42 Contamination Guard Not Extended Before Code Lands (P40)
**What goes wrong:** Time Module algorithm code (JDN, clock parsing) is written before the contamination guard script is extended to scan `time/`. A contamination slip would go undetected until the script is updated (too late).
**Why it happens:** Easy to start coding before updating CI scripts.
**How to avoid:** Extend `scripts/check-free42-contamination.sh` to include `TIME_DIR="hp41-core/src/ops/time"` in Wave 0 (scaffolding), before any `time/*.rs` algorithm file lands. Add it to the directory existence check AND the grep loop. Add time-specific tokens (e.g., `date2j`, `j2date`, `core_commands7`) if Free42 uses those as identifiers.
**Warning signs:** CI passes on a `time/` file that contains contaminated code.

### Pitfall 6: 4-Way Match Items 3+4 Causing Unintentional Build Failure (P43)
**What goes wrong:** All ~33 Op variants are added to `dispatch()` and `execute_op()` but the developer also tries to update `prgm_display.rs` in the same phase, or forgets to create a placeholder arm.
**Why it happens:** The established pattern (sanctioned CI break for items 3+4) is well-documented but easy to forget under pressure.
**How to avoid:** Phase 38 scope is items 1+2 ONLY. Items 3+4 are the Phases 39+41 scope. The CI break in `hp41-cli` and `hp41-gui` is intentional and expected. Document it in the plan with a `// Phase 38 sanctioned CI break — Phase 39 closes item 3` comment in the match blocks.
**Warning signs:** Trying to add arms to `prgm_display.rs` in Phase 38 tasks.

### Pitfall 7: SETIME/SETDATE Offset Model Confusion (D-38.2)
**What goes wrong:** Instead of storing a delta between the entered time and the current system time, the entered time is stored directly. Then calling `TIME` returns the user-set value instead of the current system time + offset. The clock doesn't advance in real time.
**Why it happens:** "Store what the user typed" feels natural.
**How to avoid:** D-38.2 is locked. `SETIME` implementation: parse X as HH.MMSSss → convert to seconds-since-midnight → compute delta = `entered_seconds - current_local_seconds` → store delta as `time_offset_secs`. `TIME` reads `SystemTime::now() + offset_duration` every call.
**Warning signs:** After `SETIME`, calling `TIME` twice 5 seconds apart returns the same value.

### Pitfall 8: T+X Midnight Rollover (P35 adjacent)
**What goes wrong:** `T+X` adds a time value to the clock. If the addition crosses midnight (e.g., 23:30 + 1 hour = 00:30 next day), the date must also advance. Forgetting the date rollover produces a clock that shows 24:30:00.
**Why it happens:** Midnight rollover is a cross-subsystem concern (time + date).
**How to avoid:** After computing new time seconds: if `new_secs >= 86400` (24h), add `new_secs / 86400` days to the date offset and take `new_secs % 86400` as the time. Store both adjustments in `time_offset_secs` (which encodes both time AND date offset as a total seconds offset from Unix epoch). This is the natural consequence of D-38.2: `time_offset_secs` is a single value that shifts the entire clock.
**Warning signs:** After T+X crossing midnight, DATE returns yesterday's date.

### Pitfall 9: XYZALM ALPHA Prefix Parsing
**What goes wrong:** The `>>` (interrupting) and `>` (non-interrupting) prefixes in ALPHA must be parsed correctly. If `>` is treated as a single-char prefix that also matches `>>`, message alarms with `>` at the start would be misclassified as control alarms.
**Why it happens:** String prefix matching without checking both chars.
**How to avoid:** Check `>>`-prefix FIRST (longer match wins). Only if ALPHA starts with `>>` is it an interrupting control alarm. If ALPHA starts with `>` (and not `>>`) it is a non-interrupting control alarm. Any other content is a message alarm.
**Warning signs:** ALPHA = `>>MYPROG` is treated as a non-interrupting alarm instead of interrupting.

### Pitfall 10: migrate_after_load Not Updated (D-carried.5)
**What goes wrong:** `default_xrom_modules()` is updated to `0b0000_0111` but `migrate_after_load()` is not extended with the bit-2 upgrade. v3.1 save files that have `xrom_modules = 0b0000_0011` load without bit 2 set. `XEQ "TIME"` returns InvalidOp.
**Why it happens:** Two-location update (default function + migration function) that can get out of sync.
**How to avoid:** Always update both `default_xrom_modules()` (returns `0b0000_0111`) AND `migrate_after_load()` (adds `| 0b0000_0100` arm) in the same Wave 0 plan task.
**Warning signs:** v3.1 save file backward compat test fails; TIME returns InvalidOp on loaded save.

---

## Serde Shape Decision Matrix (P33)

| Field | Persistent? | `#[serde(default)]` | `#[serde(skip)]` | Rationale |
|-------|-------------|---------------------|------------------|-----------|
| `time_offset_secs: i64` | YES | YES | NO | Clock offset persists across restarts |
| `clock_12h: bool` | YES | YES | NO | User format preference persists |
| `clock_display_mode: ClockDisplayMode` | YES | YES | NO | User display preference persists |
| `accuracy_factor: HpNum` | YES | YES | NO | Calibration data; store-only |
| `alarms: Vec<AlarmEntry>` | YES | YES | NO | Alarm catalog persists |
| `stopwatch_mode: StopwatchMode` | YES | YES | NO | Running/stopped state persists |
| `stopwatch_accumulated: f64` | YES | YES | NO | Accumulated centiseconds persist |
| `stopwatch_split: f64` | YES | YES | NO | Split-point time persists |
| `stopwatch_start: Option<Instant>` | NO | YES | YES | Instant not serializable; transient |
| `clock_active: bool` | NO | YES | YES | Transient display mode flag |
| `stopwatch_keyboard_mode: bool` | NO | YES | YES | Transient keyboard takeover flag |
| `alarm_catalog_mode: bool` | NO | YES | YES | Transient keyboard takeover flag |

**IMPORTANT:** `StopwatchMode::Running` persists, but `stopwatch_start` (the Instant) does not. On load, if `stopwatch_mode = Running`, the implementation should transition to `Stopped` and keep `stopwatch_accumulated` as-is (per D-38.6: freeze elapsed time on save). The `migrate_after_load()` function should include this transition.

---

## Code Examples

### Complete XROM ID 26 Function Reference (from QRC research)

[CITED: HP 82182A Time Module Quick Reference Card (82182-90002, November 1981)]
[CITED: HP-41CX Quick Reference Guide (00041-90475, August 1983)]

| XROM ID | Mnemonic | Category |
|---------|----------|----------|
| 26,1 | ADATE | Alpha Date/Time |
| 26,2 | ALMCAT | Alarm |
| 26,3 | ALMNOW | Alarm |
| 26,4 | ATIME | Alpha Date/Time |
| 26,5 | ATIME24 | Alpha Date/Time |
| 26,6 | CLK12 | Clock Display |
| 26,7 | CLK24 | Clock Display |
| 26,8 | CLKT | Clock Display |
| 26,9 | CLKTD | Clock Display |
| 26,10 | CLOCK | Clock Display |
| 26,11 | CORRECT | Clock |
| 26,12 | DATE | Date |
| 26,13 | DATE+ | Date Arithmetic |
| 26,14 | DDAYS | Date Arithmetic |
| 26,15 | DMY | Date Format |
| 26,16 | DOW | Date Arithmetic |
| 26,17 | MDY | Date Format |
| 26,18 | RCLAF | Clock |
| 26,19 | RCLSW | Stopwatch |
| 26,20 | RUNSW | Stopwatch |
| 26,21 | SETAF | Clock |
| 26,22 | SETDATE | Clock |
| 26,23 | SETIME | Clock |
| 26,24 | SETSW | Stopwatch |
| 26,25 | STOPSW | Stopwatch |
| 26,26 | SW | Stopwatch |
| 26,27 | T+X | Clock |
| 26,28 | TIME | Clock |
| 26,29 | XYZALM | Alarm |
| 26,31 | CLALMA | Alarm (CX-only) |
| 26,32 | CLALMX | Alarm (CX-only) |
| 26,33 | CLRALMS | Alarm (CX-only) |
| 26,34 | RCLALM | Alarm (CX-only) |
| 26,35 | SWPT | Stopwatch (CX-only) |

Note: STPW (split recording during SW mode) is a keyboard-mode sub-operation; it may or may not be a distinct XROM function. [ASSUMED] Treat as a distinct Op if the QRC confirms it; otherwise fold into the SW keyboard mode implementation.

### XYZALM Stack Layout (from QRC)

[CITED: HP 82182A Time Module Quick Reference Card (82182-90002, November 1981)]

| Stack Register | Content | Format |
|----------------|---------|--------|
| T | (unused) | — |
| Z | Repeat interval | HHHH.MMSSss or 0 (no repeat) |
| Y | Date | MM.DDYYYY or DD.MMYYYY (per Flag 31); 0 = today |
| X | Time | HH.MMSSss |
| ALPHA | Alarm type/content | Empty/text = Message; `>label` = Non-interrupting control; `>>label` = Interrupting control |

### Alarm Type Detection from ALPHA

```rust
// Source: HP 82182A QRC (82182-90002) + FEATURES.md §"Category 6: Alarm System"
fn parse_alarm_type(alpha: &str) -> AlarmType {
    if let Some(label) = alpha.strip_prefix(">>") {
        AlarmType::Control { label: label.to_string(), interrupting: true }
    } else if let Some(label) = alpha.strip_prefix('>') {
        AlarmType::Control { label: label.to_string(), interrupting: false }
    } else {
        AlarmType::Message(alpha.to_string())
    }
}
```

[CITED: HP 82182A Time Module Quick Reference Card (82182-90002)] — `>>` and `>` prefix convention confirmed.

### check-free42-contamination.sh Extension Pattern

```bash
# Add to scripts/check-free42-contamination.sh (Wave 0 task):
TIME_DIR="hp41-core/src/ops/time"

for dir in "$MATH1_DIR" "$STAT1_DIR" "$TIME_DIR"; do
    if [[ ! -d "$dir" ]]; then
        echo "FAIL: $dir does not exist — license guard cannot run." >&2
        exit 2
    fi
done

# Extend PATTERN with time-specific Free42 identifiers:
# core_commands7 = Free42's time module source file; date2j/j2date = Free42's JDN functions.
PATTERN='phloat|Phloat|bid128_|decNumber|decContext|vartype|arg_struct|prgm_lines|bcd_t|Thomas Okken|AGPL|GNU General Public License|math_normal_|math_chi2_|math_t_dist_|math_F_dist_|math_gamma_|math_beta_inc|core_commands7|date2j|j2date'

for dir in "$MATH1_DIR" "$STAT1_DIR" "$TIME_DIR"; do
    if matches=$(grep -rn -E "$PATTERN" "$dir" | grep -v "$DISCLAIM_LINE"); then
        ...
    fi
done
```

[VERIFIED: codebase] — `check-free42-contamination.sh` lines 23-63 are the template; TIME_DIR extension follows the existing STAT1_DIR pattern exactly.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Math 1 only (v3.0) | Math 1 + Stat 1 (v3.1) | Phase 33, 2026-05-22 | Template for Time Module XROM registration |
| No real-time display | "Pull on redraw" (D-carried.7) | Phase 38 (this phase) | Frontend redraws compute time-dependent display on-demand |
| No system clock in core | Direct std::time::SystemTime (D-38.1) | Phase 38 (this phase) | Value-returning syscall; not console I/O |
| H.MMSS format (hms.rs) | HH.MMSSss for Time Module (P44) | Phase 38 (this phase) | New dedicated parser; hms.rs unchanged |

**Deprecated/outdated:**
- `default_xrom_modules()` returning `0b0000_0011`: will return `0b0000_0111` after Wave 0.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | TIME_MODULE `name = "TIME 2C"` (HP-41CX CX ROM variant) | Standard Stack / Pattern 1 | Display string in CATALOG 2 would be wrong; low user impact |
| A2 | Op variant naming prefix `Time` (e.g., `Op::TimeTime`, `Op::TimeDate`) | Pattern 1 | Compile errors if naming conflicts exist; easy to rename at implementation time |
| A3 | STPW is a distinct XROM function (not just a keyboard sub-key in SW mode) | Function reference table | If STPW is keyboard-only, no separate Op variant needed; reduces count by 1 |
| A4 | `trigger_unix: i64` stores the alarm trigger as a Unix timestamp | Pattern 7 | If a different time representation is cleaner (e.g., days + seconds-since-midnight), data model changes; functionally equivalent |
| A5 | `event_buffer` format strings for alarms (e.g., `"alarm:message:..."`) | Pattern 9 | Frontend must parse this format; agreed upon at implementation time between core and Phase 39 |
| A6 | `AlarmEntry` uses `trigger_unix` and `repeat_secs` as `i64` | Pattern 7 | If OM specifies maximum values that fit in smaller types, no correctness risk |
| A7 | CLOCK and SW are programmable (store as Op variants in programs) | Phase Requirements table | If non-programmable, `execute_op()` arms should return `Err(InvalidOp)` instead of executing |

**If this table is empty after implementation:** All claims should be verified against the HP 82182A Owner's Manual (00041-90035) before Phase 38 is completed.

---

## Open Questions (RESOLVED)

1. **Are CLOCK and SW non-programmable?**
   - What we know: PITFALLS.md P38 notes this needs Owner's Manual verification. The QRC confirms their interactive nature. Some HP-41 functions are keyboard-only (like PRGM and ON).
   - What's unclear: Whether `Op::TimeClock` and `Op::TimeSw` in a program should execute or return an error.
   - RESOLVED: Implement as functional in programs (they just set flags); document as a potential divergence in `docs/hp41-time-divergences.md`. The safe default is to execute rather than error — a program that sets clock mode is plausibly useful.

2. **Exact DOW formula for day-of-week numbering**
   - What we know: OM documents 0=Sunday, 1=Monday, ..., 6=Saturday. JDN 0 was a Monday.
   - What's unclear: The exact modulo formula (`(JDN + 1) % 7` gives Sunday=0 for modern dates, but needs verification against a known date).
   - RESOLVED: Use `(JDN + 1) % 7` which yields 0=Sunday for modern dates. Verify in test suite against 2026-05-24 (Saturday=6) and 2026-05-25 (Sunday=0).

3. **SETIME negative values (PM shorthand)**
   - What we know: FEATURES.md confirms SETIME accepts `-1` through `-11` as PM shorthand (1 PM through 11 PM).
   - What's unclear: Whether the clock display shows negative values or converts them to 24h format.
   - RESOLVED: Normalize on input in the modal submit logic: if X is negative and in range [-1, -11], convert to 24h equivalent (`abs(X) + 12`). Display always shows 24h-normalized values. Document in divergence file.

4. **ALMCAT keyboard mode in hp41-core scope**
   - What we know: ALMCAT enters an interactive catalog mode with keyboard redefinition. The keyboard redefinition is frontend scope (Phases 39/41).
   - What's unclear: Whether `op_almcat()` in hp41-core should just set the `alarm_catalog_mode: bool` flag (and return), or also produce some output.
   - RESOLVED: `op_almcat()` sets `alarm_catalog_mode = true` and writes the first alarm's display string to `print_buffer`. Frontend handles the keyboard mode in Phase 39/41. This is the minimal hp41-core scope.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` / Rust toolchain | All compilation | ✓ | 1.95.0 | — |
| `just` | `just test`, `just check` | ✓ | 1.49.0 | `cargo test` directly |
| `std::time::SystemTime` | TIME/DATE ops | ✓ | std | — |
| `std::time::Instant` | Stopwatch | ✓ | std | — |
| `libc::localtime_r` | Local time conversion (Unix) | ✓ (transitive) | N/A | `GetLocalTime` on Windows (cfg) |

**Missing dependencies with no fallback:** none

**Missing dependencies with fallback:** none

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test + existing cargo-llvm-cov |
| Config file | `Cargo.toml` (workspace) |
| Quick run command | `cargo test -p hp41-core 2>&1 \| tail -5` |
| Full suite command | `just test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TIME-FW-01 | TIME_MODULE xrom_resolve bit-2 | unit | `cargo test -p hp41-core xrom_time` | ❌ Wave 0 |
| TIME-FW-02 | default_xrom_modules returns 0b111, migrate_after_load sets bit 2 | unit | `cargo test -p hp41-core xrom_modules_default` | ❌ Wave 0 |
| TIME-FW-03 | CalcState serializes/deserializes new fields with defaults | integration | `cargo test -p hp41-core time_serde_round_trip` | ❌ Wave 0 |
| TIME-DAT-01/02/03 | DATE+, DDAYS, DOW with edge cases | unit | `cargo test -p hp41-core time_date_arith` | ❌ Wave 1 |
| TIME-DAT-06 | Date parsing edge cases (trailing zeros, leap years) | unit | `cargo test -p hp41-core parse_date_edge_cases` | ❌ Wave 1 |
| TIME-CLK-01..06 | Clock operations return correct values | unit | `cargo test -p hp41-core time_clock_ops` | ❌ Wave 2 |
| TIME-SW-01..09 | Stopwatch state machine transitions | unit | `cargo test -p hp41-core time_stopwatch` | ❌ Wave 3 |
| TIME-ALM-01..12 | Alarm catalog CRUD + check_alarms | unit | `cargo test -p hp41-core time_alarm` | ❌ Wave 4 |
| TIME-FW-04 | v3.1 save file loads with migrated xrom_modules | integration | `cargo test -p hp41-core time_backward_compat` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p hp41-core 2>&1 | tail -10`
- **Per wave merge:** `just test`
- **Phase gate:** Full suite green + coverage check before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `hp41-core/tests/time_xrom_registration.rs` — covers TIME-FW-01, TIME-FW-02 (xrom_resolve bit-2, TIME_MODULE const fields, default + migration)
- [ ] `hp41-core/tests/time_serde_compat.rs` — covers TIME-FW-03, TIME-FW-04 (new field serialization, v3.1 fixture load)
- [ ] `hp41-core/tests/fixtures/v31-autosave.json` — v3.1 save fixture for backward compat test

*(All other time test files are Wave 1-4 gaps, detailed above)*

---

## Security Domain

> `security_enforcement` not explicitly set to false in config.json; treated as enabled.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Parse date/time/alarm inputs with explicit range validation; `HpError::Data` on out-of-range |
| V6 Cryptography | no | — |

### Known Threat Patterns for {stack}

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Integer overflow in JDN arithmetic | Tampering | Use `i64` (not `i32`) for JDN; valid Gregorian range fits in i64 |
| Alarm catalog unbounded growth | Denial of Service | Cap at 253 entries (OM hardware limit); enforce in `op_xyzalm()` |
| String parsing of ALPHA for alarm type | Tampering | `strip_prefix` on ALPHA content; no eval or exec of arbitrary strings in core |

---

## Sources

### Primary (HIGH confidence)
- HP 82182A Time Module Quick Reference Card (82182-90002, November 1981) — all 29 base functions, alarm formats, XYZALM parameters, stopwatch keyboard layout [CITED]
- HP-41CX Quick Reference Guide (00041-90475, August 1983) — 6 CX-only functions (CLALMA, CLALMX, CLRALMS, RCLALM, SWPT + CLOCK), keyboard layouts, Flag 31 table [CITED]
- Fliegel, H.F. & Van Flandern, T.C. (1968), ACM Commun. 11(10):657 — JDN algorithm [CITED]
- Codebase: `hp41-core/src/ops/math1/xrom.rs` (MATH_1 + STAT_1 + xrom_resolve patterns) [VERIFIED: codebase]
- Codebase: `hp41-core/src/state.rs` (CalcState fields, migrate_after_load, default_xrom_modules) [VERIFIED: codebase]
- Codebase: `hp41-core/src/ops/math1/modal.rs` (ModalProgram enum + dispatch pattern) [VERIFIED: codebase]
- Codebase: `hp41-core/src/ops/stat1/modal.rs` (Stat1Step pattern for TimeStep) [VERIFIED: codebase]
- Codebase: `scripts/check-free42-contamination.sh` (extension template) [VERIFIED: codebase]
- Codebase: `hp41-core/tests/xrom_shadowing.rs` (CI gate extension template) [VERIFIED: codebase]
- Codebase: `hp41-core/src/ops/hms.rs` (H.MMSS parse pattern; confirmed different from HH.MMSSss) [VERIFIED: codebase]
- `.planning/research/SUMMARY.md` — v3.2 research summary [CITED]
- `.planning/research/PITFALLS.md` — 12 pitfall categories P32-P45 [CITED]
- `.planning/research/FEATURES.md` — 35 callable functions confirmed from QRC [CITED]
- `38-CONTEXT.md` — all locked decisions D-38.1 through D-carried.7 [VERIFIED: project]

### Secondary (MEDIUM confidence)
- HP Museum XROM Numbers list (hpmuseum.org/software/xroms.htm) — XROM ID 26 confirmed [CITED]
- HP-41 Module Database (calc.fjk.ch/db/hp41mod.php) — Time Module 1A/1B/1C/2C all XROM 26 [CITED]

### Tertiary (LOW confidence)
- HP-41CX Owner's Manual 00041-90035 — alarm type encoding details, CLOCK/SW programmability. Full text not read; referenced via excerpt summary in FEATURES.md. Treat CLOCK/SW programmability as [ASSUMED].

---

## Metadata

**Confidence breakdown:**
- XROM framework extension: HIGH — exact pattern from codebase; Pattern 1-3 verified line-by-line
- JDN date arithmetic: HIGH — textbook algorithm; Fliegel-Van Flandern 1968 primary source
- Date/time format parsing: HIGH — ISG/DSE precedent pattern in codebase; edge cases documented
- Alarm system structure: HIGH (data model) / MEDIUM (XYZALM encoding) — D-38.8-D-38.11 locked; encoding verified from QRC
- Stopwatch state machine: HIGH — D-38.6/D-38.7 locked; Instant/Duration patterns standard
- System clock integration: HIGH — D-38.1/D-38.2 locked; std::time well-documented
- CLOCK/SW programmability: LOW — needs OM verification before execute_op() arms finalized

**Research date:** 2026-05-24
**Valid until:** 2026-06-24 (30 days; stable domain — no fast-moving external APIs)
