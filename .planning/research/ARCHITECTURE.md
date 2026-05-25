# Architecture Patterns: HP-41CX Time Module Emulation (v3.2)

**Domain:** HP-41CX Time Module (HP 82182A, XROM 26) -- real-time clock, alarm catalog, live stopwatch
**Researched:** 2026-05-24
**Confidence:** HIGH (codebase fully read; XROM 26 confirmed from calc.fjk.ch + qrg41.fjk.ch)

## The Central Challenge

The existing emulator is **purely event-driven**: `hp41-core` owns `CalcState` (no threads, no timers, no async), the CLI polls `crossterm::event::poll(16ms)` and redraws, and the GUI holds `CalcState` behind a single `Mutex<CalcState>` with IPC via Tauri commands. The Time Pac introduces three categories of behavior that do NOT fit the existing dispatch-and-return model:

1. **System clock reads** (TIME, DATE) -- simple one-shot reads; fits existing model perfectly.
2. **Live clock display** (CLKT, CLKTD, CLOCK) -- requires periodic LCD refresh WITHOUT user input.
3. **Stopwatch with sub-second display** (SW, RUNSW, STOPSW, RCLSW, SETSW, SWPT) -- running timer with live LCD updates and lap timing.
4. **Alarm scheduling and triggering** (XYZALM, ALMCAT, RCLAF, ALMNOW, CLALMA, CLALMX, CLRALMS, RCLALM) -- periodic alarm-check to fire interrupts/programs.

Category 1 is trivial. Categories 2-4 require architectural decisions.

---

## Recommended Architecture

### Strategy: "Pull on redraw" -- NO new background threads in hp41-core

**Decision:** hp41-core remains thread-free, timer-free, and async-free. Time-dependent state is computed on-demand when the frontend asks for display state, not maintained by a background ticker.

**Rationale:** The emulator does NOT need cycle-accurate HP-41CX hardware timing. It needs _behavioral fidelity_ -- the user sees a running clock, the stopwatch counts correctly, alarms fire when due. The existing 16ms poll loop (CLI) and IPC-on-demand model (GUI) both provide sufficient refresh frequency. We leverage these existing refresh cycles rather than adding concurrency.

### Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| `hp41-core/src/ops/time/` | Time Pac ops; pure functions reading `std::time::SystemTime` / `std::time::Instant`; alarm state machine; stopwatch arithmetic | `CalcState` (owned), `std::time` (read-only) |
| `hp41-core/src/state.rs` | New persistent + transient `CalcState` fields for clock config, alarms, stopwatch | All ops via `&mut CalcState` |
| `hp41-core/src/ops/math1/xrom.rs` | `TIME_1` module registry (XROM 26, bit 2); `time_resolve()` function | `xrom_resolve()` caller chain |
| `hp41-core/src/ops/math1/modal.rs` | `ModalProgram::Time(TimeStep)` variant for SETTIME, SETDATE, SETSW, XYZALM prompts | `modal_prompt` / `modal_program` channel |
| `hp41-cli/src/app.rs` | Clock-display redraw on every poll tick; stopwatch LCD live-update; alarm-check on every poll tick | `CalcState` directly |
| `hp41-gui/src-tauri/src/commands.rs` | `tick_time` command for periodic frontend refresh when clock/stopwatch active | `AppState` Mutex |
| `hp41-gui/src/App.tsx` | Conditional `setInterval(tick_time, 500)` when clock-display or stopwatch is active | Tauri IPC |

### Data Flow

```
                                 hp41-core (pure, no threads)
                                 +--------------------------+
                                 | CalcState                |
                                 |   .clock_display_mode    |  <-- CLKT/CLKTD sets (transient)
                                 |   .date_format           |  <-- DMY/MDY sets (persistent)
                                 |   .clock_format_24h      |  <-- CLK12/CLK24 sets (persistent)
                                 |   .accuracy_factor       |  <-- SETAF/RCLAF (persistent)
                                 |   .stopwatch_state       |  <-- SETSW/RUNSW/STOPSW (transient)
                                 |   .alarm_catalog         |  <-- XYZALM/CLALMA/etc. (persistent)
                                 +--------------------------+

  CLI (app.rs event loop):                   GUI (React + Tauri):
  +---------------------------+              +----------------------------------+
  | poll(16ms) {              |              | useEffect(() => {                |
  |   if clock_display_active |              |   if (clockActive || swRunning)  |
  |     -> format live clock  |              |     interval = setInterval(      |
  |     -> render in LCD area |              |       invoke("tick_time"), 500)  |
  |   if sw_running           |              | }, [clockActive, swRunning])     |
  |     -> elapsed from start |              |                                 |
  |     -> render in LCD area |              | tick_time command:               |
  |   check_alarms(&mut state)|              |   lock state, check_alarms(),   |
  |     -> fire if due        |              |   return CalcStateView with      |
  |   handle_key(...)         |              |   live clock/sw display strings  |
  | }                         |              +----------------------------------+
  +---------------------------+
```

---

## New CalcState Fields

### Persistent Fields (`#[serde(default)]` without `#[serde(skip)]`)

| Field | Type | Default | Purpose |
|-------|------|---------|---------|
| `date_format` | `DateFormat` (enum: Mdy, Dmy) | `Mdy` | Date display order per DMY/MDY ops |
| `clock_format_24h` | `bool` | `false` | 12h vs 24h time display per CLK12/CLK24 |
| `accuracy_factor` | `HpNum` | `zero()` | Clock accuracy correction factor (-99.9..99.9) |
| `alarm_catalog` | `Vec<AlarmEntry>` | `vec![]` | Ordered list of pending + past-due alarms |
| `next_alarm_id` | `u64` | `0` | Monotonic ID counter for CLALMX (clear by ID) |

### Transient Fields (`#[serde(default, skip)]`)

| Field | Type | Default | Purpose |
|-------|------|---------|---------|
| `clock_display_mode` | `Option<ClockDisplayMode>` | `None` | TimeOnly / TimeAndDate -- volatile display mode |
| `stopwatch_state` | `Option<StopwatchState>` | `None` | Running/stopped/lap state |
| `pending_alarm_event` | `Option<AlarmEvent>` | `None` | Fired alarm waiting for frontend drain |

**Rationale for transient `clock_display_mode`:** On real HP-41CX hardware, clock display is a volatile mode that turns off when power is lost. Skipping serialization is hardware-faithful.

**Rationale for transient `stopwatch_state`:** `StopwatchState` contains `std::time::Instant` which CANNOT be serialized (no epoch reference). The real hardware also loses stopwatch state on power loss. `#[serde(skip)]` is both technically required and hardware-faithful.

**Total new CalcState fields: 8** (5 persistent, 3 transient). This is more than Stat 1 (2 fields: `rand_seed` persistent, `pending_chisqd_nu` transient) because Time Pac introduces genuinely new state categories (alarms, stopwatch) that have no analog in the existing engine.

---

## Key Data Structures

### ClockDisplayMode

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ClockDisplayMode {
    TimeOnly,     // CLKT -- show HH:MM:SS or HH:MM:SS AM/PM
    TimeAndDate,  // CLKTD -- alternating time and date display
}
```

### DateFormat

```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DateFormat {
    Mdy,  // Month.Day Year -- HP-41 cold-start default
    Dmy,  // Day.Month Year -- European convention
}
impl Default for DateFormat { fn default() -> Self { DateFormat::Mdy } }
```

### StopwatchState

```rust
#[derive(Debug, Clone)]
pub struct StopwatchState {
    pub running: bool,
    pub accumulated: std::time::Duration,     // elapsed from prior run segments
    pub last_start: Option<std::time::Instant>, // monotonic start point
    pub split_time: Option<std::time::Duration>, // SWPT lap display
}

impl StopwatchState {
    pub fn elapsed(&self) -> Duration {
        let running_delta = match (self.running, self.last_start) {
            (true, Some(start)) => start.elapsed(),
            _ => Duration::ZERO,
        };
        self.accumulated + running_delta
    }
}
```

**Critical design:** `Instant` for stopwatch (monotonic, never goes backward). `SystemTime` for wall-clock date/time ops (TIME, DATE, alarm comparison). Never mix them.

### AlarmEntry

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlarmEntry {
    pub time: HpNum,       // HH.MMSSss format
    pub date: HpNum,       // MM.DDYYYY or DD.MMYYYY format
    pub message: String,   // from ALPHA at XYZALM time
    pub alarm_type: u8,    // 0 = message, 1-4 = execute program
    pub past_due: bool,    // already fired
    pub id: u64,           // monotonic ID for CLALMX
}
```

### AlarmEvent (drain payload)

```rust
#[derive(Debug, Clone)]
pub struct AlarmEvent {
    pub message: String,
    pub alarm_type: u8,
    pub program_label: Option<String>, // for type 1-4 alarm
}
```

---

## Integration Strategy: Clock Display (CLKT / CLKTD / CLOCK)

### Problem
The existing `display_str` in `CalcStateView` is computed once per dispatch. Clock display needs to update every second WITHOUT user interaction.

### Solution: Extend the existing poll-based redraw

**CLI:** The `App::run()` loop already redraws every ~16ms via `event::poll(Duration::from_millis(16))`. When `state.clock_display_mode.is_some()`, the display-string computation in `ui::render_ui()` calls a new `format_clock_display()` function that reads `SystemTime::now()` and formats the current time. No new thread needed -- the existing 16ms tick is 60 FPS, more than enough for a 1-second clock update.

**GUI:** Add a new Tauri command `tick_time` that the React frontend calls via `setInterval` ONLY when `clock_display_mode` is active or stopwatch is running. This is a purpose-built time-tick command, NOT a general poll of `get_state()`.

**D-11 compliance:** The "no polling" invariant (D-11) prohibits polling `get_state()` in a loop. `tick_time` is a separate, purpose-built command that only runs when the user has explicitly entered a time-display mode (CLKT/CLKTD/CLOCK/SW). When no time display is active, no timer fires, no polling occurs.

### Display Priority Chain Extension

The existing `CalcStateView::from_state()` display_str priority chain (types.rs lines 130-146) gains a new top-priority branch:

```
Priority 0 (NEW): clock_display_mode active AND entry_buf empty -> format_clock_display()
Priority 0b (NEW): stopwatch running AND entry_buf empty -> format_stopwatch_display()
Priority 1 (existing): modal_prompt active AND entry_buf empty -> truncate(modal_prompt)
Priority 2 (existing): entry_buf non-empty -> entry_buf verbatim
Priority 3 (existing): alpha_mode -> format_alpha()
Priority 4 (existing): format_hpnum(stack.x)
```

Clock and stopwatch display take highest priority because they represent an active hardware display mode that overrides everything else (matching HP-41CX behavior). Once the user starts typing (entry_buf non-empty), the typed digits take precedence (live feedback), and the clock resumes when entry_buf is flushed.

---

## Integration Strategy: Stopwatch (SW / RUNSW / STOPSW / RCLSW / SETSW / SWPT)

### Live Display
Elapsed time is computed on-demand using `Instant::elapsed()`:

```rust
// Called in the display path -- every 16ms CLI, every 500ms GUI
fn format_stopwatch_display(sw: &StopwatchState) -> String {
    let elapsed = sw.elapsed(); // no state mutation
    let secs = elapsed.as_secs();
    let tenths = elapsed.subsec_millis() / 100;
    format!("{:02}:{:02}:{:02}.{}", secs / 3600, (secs % 3600) / 60, secs % 60, tenths)
}
```

### SW Interactive Mode
On real HP-41CX hardware, the `SW` command enters a special keyboard mode where specific keys map to START/STOP/SPLIT/RESET. This maps to a new `PendingInput::StopwatchMode` variant in the CLI and a new modal state in the GUI. The existing `PendingInput` / modal infrastructure handles this.

**Key mapping in SW mode:**
- R/S -> toggle RUNSW/STOPSW
- ENTER -> SWPT (split/lap time)
- CLX -> SETSW 0 (reset)
- BST/SST -> RCLSW
- Any other key -> exit SW mode

---

## Integration Strategy: Alarms (XYZALM / ALMCAT / ALMNOW / etc.)

### Alarm Check Scheduling
Alarms are checked on every event-loop iteration. The check is a simple O(n) scan:

```rust
pub fn check_alarms(state: &mut CalcState) -> Option<AlarmEvent> {
    let now = SystemTime::now();
    let (now_date, now_time) = system_time_to_hp41(&now, &state.date_format);
    for alarm in &mut state.alarm_catalog {
        if !alarm.past_due && alarm_is_due(alarm, now_date, now_time) {
            alarm.past_due = true;
            return Some(AlarmEvent {
                message: alarm.message.clone(),
                alarm_type: alarm.alarm_type,
                program_label: if alarm.alarm_type > 0 {
                    Some(alarm.message.clone())
                } else { None },
            });
        }
    }
    None
}
```

### Alarm Triggering
When an alarm fires:
- **Message alarm (type 0):** Write message to `display_override` + push "BEEP" to `event_buffer`. Both channels are already drained by CLI and GUI.
- **Program alarm (type 1-4):** Push `AlarmEvent` into `pending_alarm_event` field. Frontend drains and executes `XEQ <label>`. This mirrors the `pending_card_op` drain pattern.

### ALMCAT
Outputs to `print_buffer` (same channel as CATALOG 1/2, PRX, PRSTK). Each pending alarm produces one formatted line.

---

## Integration Strategy: Date Arithmetic (DATE+ / DDAYS / DOW)

Pure functions -- no system clock, no timers. Operate on HP-41 date format (MM.DDYYYY or DD.MMYYYY).

**Implementation:** Hand-coded Julian Day Number (JDN) conversion, ~40 LOC total. NO `chrono` dependency.

**Rationale for no chrono:** Zero new runtime deps policy (ADR-v3.1-002). The algorithms needed are well-known:
1. `gregorian_to_jdn(y, m, d) -> i32` -- standard formula
2. `jdn_to_gregorian(jdn) -> (i32, u8, u8)` -- inverse
3. `day_of_week(y, m, d) -> u8` -- JDN mod 7 (or Zeller's congruence)
4. `days_between(d1, d2) -> i32` -- JDN difference

---

## XROM Registration

### Bitfield Extension

```rust
fn default_xrom_modules() -> u8 {
    0b0000_0111 // bit 0 = Math 1, bit 1 = Stat 1, bit 2 = Time
}
```

Migration in `migrate_after_load()`:
```rust
// v3.1 -> v3.2: set bit 2 (Time module)
if self.xrom_modules & 0b0000_0100 == 0 {
    self.xrom_modules |= 0b0000_0100;
}
```

### TIME_1 Module Registry

```rust
pub const TIME_1: XromModule = XromModule {
    id: 26,
    name: "TIME 2C", // HP-41CX internal Time Module version
    ops: &[
        // Clock (15 ops)
        ("TIME", Op::Time), ("DATE", Op::Date),
        ("CLK12", Op::Clk12), ("CLK24", Op::Clk24),
        ("CLKT", Op::Clkt), ("CLKTD", Op::Clktd), ("CLOCK", Op::Clock),
        ("CORRECT", Op::Correct), ("SETIME", Op::SetTime), ("SETDATE", Op::SetDate),
        ("T+X", Op::TplusX), ("SETAF", Op::SetAf), ("RCLAF", Op::RclAf),
        ("DMY", Op::Dmy), ("MDY", Op::Mdy),
        // Date arithmetic (3 ops)
        ("DATE+", Op::DatePlus), ("DDAYS", Op::Ddays), ("DOW", Op::Dow),
        // ALPHA display (3 ops)
        ("ADATE", Op::Adate), ("ATIME", Op::Atime), ("ATIME24", Op::Atime24),
        // Alarm (7 ops)
        ("XYZALM", Op::Xyzalm), ("ALMCAT", Op::Almcat),
        ("ALMNOW", Op::AlmNow), ("RCLALM", Op::RclAlm),
        ("CLALMA", Op::ClAlmA), ("CLALMX", Op::ClAlmX), ("CLRALMS", Op::ClrAlms),
        // Stopwatch (6 ops)
        ("SW", Op::Sw), ("SETSW", Op::SetSw), ("RUNSW", Op::RunSw),
        ("STOPSW", Op::StopSw), ("RCLSW", Op::RclSw), ("SWPT", Op::Swpt),
    ],
};
```

**Total: ~34 Op variants.** Plus `time_resolve()` match block and bit-2 arm in `xrom_resolve()`.

---

## Patterns to Follow

### Pattern 1: Event Buffer Drain (established v2.0+)
**What:** hp41-core pushes structured event strings; frontend drains after each dispatch.
**When:** Alarm BEEP events, clock-mode-change notifications.
**Precedent:** `Op::Beep` pushes "BEEP" to `event_buffer`; `Op::Pse` pushes "PAUSE 1000".

### Pattern 2: Modal Program State Machine (established v3.0)
**What:** `ModalProgram` enum with per-program step enums.
**When:** SETTIME, SETDATE, SETSW prompt sequences; XYZALM multi-field entry.
**Extension:** `ModalProgram::Time(TimeStep)` parallels `ModalProgram::Stat1(Stat1Step)`.

### Pattern 3: XROM Module Registry (established v3.0, extended v3.1)
**What:** `XromModule` struct + bitfield.
**When:** TIME_1 at bit 2.

### Pattern 4: Pull-on-Redraw for Time-Dependent Display (NEW)
**What:** Display path reads `SystemTime::now()` / `Instant::elapsed()` on every redraw.
**When:** Clock display (CLKT/CLKTD) and running stopwatch.
**Why new:** No existing op needs periodic display refresh. Additive -- runs in the existing poll/redraw path without new threads.

### Pattern 5: Separate Clock Sources
**What:** `SystemTime::now()` for wall-clock; `Instant::now()` for stopwatch.
**When:** TIME/DATE/alarm-check vs RUNSW/STOPSW/RCLSW.
**Why:** `SystemTime` gives wall-clock correctness; `Instant` gives monotonic guarantees.

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Background Timer Thread in hp41-core
**Why bad:** Violates "no async, no threads in hp41-core" invariant. Non-deterministic tests.
**Instead:** Pull on redraw.

### Anti-Pattern 2: Storing Wall-Clock Timestamps as Instant
**Why bad:** `Instant` has no epoch, cannot be serialized, cannot represent alarm times.
**Instead:** Alarms stored as HP-41 formatted `HpNum` values; compared against `SystemTime::now()`.

### Anti-Pattern 3: Polling get_state() from GUI for Live Display
**Why bad:** Violates D-11 "no polling" invariant.
**Instead:** Purpose-built `tick_time` command, conditional on `clockActive || swRunning`.

### Anti-Pattern 4: Persisting StopwatchState
**Why bad:** `Instant` cannot be serialized; elapsed gap across save/load is unaccountable.
**Instead:** `#[serde(default, skip)]`. User re-initializes via SETSW.

### Anti-Pattern 5: Using chrono for Date Arithmetic
**Why bad:** Violates zero-new-runtime-deps policy (ADR-v3.1-002). 35k LOC for 40 LOC of use.
**Instead:** Hand-coded Julian Day Number functions.

---

## File Tree (New + Modified)

### New Files

| File | Purpose |
|------|---------|
| `hp41-core/src/ops/time/mod.rs` | Module root; AlarmEntry, StopwatchState, ClockDisplayMode, DateFormat, named consts |
| `hp41-core/src/ops/time/clock.rs` | TIME, DATE, CLK12, CLK24, CLKT, CLKTD, CLOCK, CORRECT, SETIME, SETDATE, T+X, SETAF, RCLAF, DMY, MDY |
| `hp41-core/src/ops/time/date_arith.rs` | DATE+, DDAYS, DOW -- Julian Day Number algorithms (~40 LOC) |
| `hp41-core/src/ops/time/alpha_time.rs` | ADATE, ATIME, ATIME24 -- append formatted date/time to ALPHA |
| `hp41-core/src/ops/time/alarm.rs` | XYZALM, ALMCAT, ALMNOW, RCLALM, CLALMA, CLALMX, CLRALMS, check_alarms() |
| `hp41-core/src/ops/time/stopwatch.rs` | SW, SETSW, RUNSW, STOPSW, RCLSW, SWPT, format_stopwatch_display() |
| `hp41-core/src/ops/time/modal.rs` | TimeStep enum + submit_step dispatch for SETTIME, SETDATE, SETSW, XYZALM |
| `docs/hp41-time-functions.json` | Fourth JSON source-of-truth (~34 entries) |
| `docs/hp41-time-function-matrix.md` | Generated via `just docs-matrix` (fourth invocation) |
| `docs/hp41-time-divergences.md` | Divergence catalog |

### Modified Files

| File | Change |
|------|--------|
| `hp41-core/src/state.rs` | 8 new CalcState fields; `migrate_after_load()` v3.1->v3.2; new type definitions |
| `hp41-core/src/ops/mod.rs` | `pub mod time;` + ~34 new Op variants + dispatch arms |
| `hp41-core/src/ops/math1/xrom.rs` | `TIME_1` const + `time_resolve()` + bit-2 arm in `xrom_resolve()` |
| `hp41-core/src/ops/math1/modal.rs` | `ModalProgram::Time(TimeStep)` variant |
| `hp41-core/src/ops/program.rs` | `execute_op()` arms for ~34 Time ops |
| `hp41-cli/src/app.rs` | `check_alarms()` + clock/SW display in draw; `PendingInput::StopwatchMode`; alarm drain |
| `hp41-cli/src/prgm_display.rs` | ~34 new `op_display_name` arms |
| `hp41-cli/src/help_data.rs` | Fourth `OnceLock` + 4-pool `help_entries_all()` |
| `hp41-gui/src-tauri/src/commands.rs` | `tick_time` command; alarm-check in finalize |
| `hp41-gui/src-tauri/src/types.rs` | CalcStateView gains `clock_display_str`, `sw_running`, `clock_active` |
| `hp41-gui/src-tauri/src/prgm_display.rs` | ~34 new `op_display_name` arms |
| `hp41-gui/src/App.tsx` | Conditional `setInterval` for `tick_time` |

---

## Suggested Build Order

1. **Phase N: hp41-core** -- XROM framework + all ~34 Time ops + date arithmetic + alarm catalog + stopwatch state + clock display mode. Intentional CI break in hp41-cli/hp41-gui.
2. **Phase N+1: hp41-cli** -- JSON help, `?` overlay, op_display_name, clock/SW display rendering, alarm-check in poll loop, SW interactive mode.
3. **Phase N+2: Documentation** -- Function matrix, divergence catalog, ADRs, README.
4. **Phase N+3: hp41-gui** -- tick_time command, CalcStateView extensions, alarm toast, help overlay, CATALOG 2.
5. **Phase N+4: Test Hardening** -- Coverage, date-arithmetic accuracy, backward-compat, E2E smoke.

**Phase ordering rationale:** Core first (4-way match). CLI before GUI (simpler validation). Docs after CLI (needs complete Op set). GUI after docs (mechanical wiring). Tests last (end-to-end required).

---

## Sources

- [HP 82182A Time Module QREF](https://qrg41.fjk.ch/hp82182a.html) -- complete function listing with descriptions (HIGH confidence)
- [HP-41CX function listing](https://www.finseth.com/hpdata/hp41cx.php) -- function descriptions including CLALMA/CLALMX/CLRALMS/RCLALM/SWPT (HIGH confidence)
- [HP-41 Module Database](https://calc.fjk.ch/db/hp41mod.php) -- XROM 26 = Time Module (HIGH confidence)
- [HP-41C XROM Numbers](https://www.hpmuseum.org/software/xroms.htm) -- XROM numbering reference
- Existing codebase: `state.rs`, `xrom.rs`, `modal.rs`, `app.rs`, `commands.rs`, `lib.rs`, `types.rs` (all directly read)
