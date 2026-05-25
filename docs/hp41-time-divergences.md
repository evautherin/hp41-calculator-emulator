# HP-41C Time Pac Emulator Divergences

This document lists known behavioral divergences between this emulator's implementation
of the HP-41C Time Module and the hardware-faithful behavior described in the
HP Time Module Owner's Manual (HP 00041-90035, 1982).

**Status:** Established as comprehensive numbered catalog in Phase 40 / Plan 40-01 (TIME-DOC-02).

**Philosophy:** Where divergences exist, this emulator prioritizes:
1. Hardware-faithful behavior where feasible.
2. User-safety (no silent data corruption without documentation).
3. Clear documentation of known divergences.

---

## How to Use This Document

Each entry carries a stable `D-40-NN` identifier that can be used in cross-references
from source-code comments, ADRs, test files, and issue trackers. The ID encodes the
phase (40 = Phase 40 / TIME-DOC-02) and an ordinal sequence number within this document.

Every entry uses five fixed fields (D-30.5 shape, carried forward as D-40 template):

- **OM citation** — The HP 00041-90035 page-and-example that is the primary source, or
  `"N/A — emulator extension"` when no OM equivalent exists.
- **Our behavior** — What this emulator does.
- **OM behavior** — What the OM says or what real HP-41CX Time Module hardware does.
- **Rationale** — Why we made this choice (hardware-fidelity vs. UX trade-off decision).
- **See** — Cross-references: ADR links, CONTEXT.md decision IDs, test file pointers,
  Pitfall references from `research/PITFALLS.md` (carried forward across v3.x).

The citation discipline (Pitfall 18 from `research/PITFALLS.md`, carried forward across
v3.x) requires every entry to carry at least one OM page reference, an explicit
`"N/A — emulator extension"` marker, or a primary-source citation (HP 82182A QRC, MoHPC
URL, HP technical documentation). No uncited assertions are permitted in this document.

Time Pac is the third XROM application module in the v3.x line; entry numbering
follows the phase-origin convention established in v3.0 (`D-30-NN` for Math Pac I,
`D-35-NN` for Stat 1 Pac) per the D-35.4 numbering scheme. The Time Pac catalog uses
`D-40-NN` identifiers tied to Phase 40 (Documentation & ADRs).

---

## 1. OM Divergences

*(Numerical / behavioral mismatches with OM-quoted examples or OM-described hardware
behavior. These are cases where the OM specifies or implies a particular outcome and our
emulator either matches or intentionally diverges from that specification.)*

No OM numerical divergences identified in Phase 38-39 implementation. The Time Pac
implementation follows HP 00041-90035 (1982) behavioral specification for all 35
callable functions. Hardware-observable date arithmetic (DATE+, DDAYS, DOW) uses the
Fliegel-Van Flandern JDN algorithm (Communications of the ACM, 1968) which is exact
over the HP-41CX supported date range. No oracle drifts analogous to the Stat 1 Pac
scipy-vs-SPEC reconciliations (D-35-01..D-35-06) were identified during Phase 38/39
verification.

*If future Phase 41 (GUI integration) or Phase 42 (test hardening) surfaces a genuine
OM-quoted-behavior mismatch for Time Pac, it will be authored as `D-40-07:` or later —
the numbering reservation `D-40-01..D-40-06` is the canonical bucket-2/3 ID range per
this plan.*

---

## 2. Emulator Extensions

*(Functions or behaviors we added that are not present in HP 00041-90035 (1982). These
are deliberate, documented additions that improve usability without conflicting with OM
behavior for OM-specified inputs. Every extension in this section is marked with
"N/A — emulator extension" in the OM citation field.)*

---

### D-40-06: SW Interactive Stopwatch Keyboard Mode — v3.2 Emulator Extension

- **OM citation**: `N/A — emulator extension`. The HP Time Module Owner's Manual
  HP 00041-90035 (1982) does not describe an `SW` top-level XEQ entry point with
  interactive keyboard mode. The `SW` command in the OM is the abbreviation for
  "stopwatch" as a noun (referring to the stopwatch subsystem as a whole), not a
  callable program entry point. The emulator adds `XEQ "SW"` as a dedicated
  interactive stopwatch display mode that binds the keyboard to stopwatch control keys
  (R/S = RUNSW/STOPSW toggle, ENTER = STPW split, Esc = exit) during the session.
  This is a v3.2 emulator extension per `docs/hp41-time-functions.json` SW entry
  inline `divergences` field: "Extended interactive stopwatch keyboard mode -- emulator
  extension".

- **Our behavior**: `XEQ "SW"` sets `state.stopwatch_keyboard_mode = true` and returns.
  The frontend (CLI Phase 39 / GUI Phase 41) intercepts this flag and enters an
  interactive stopwatch display loop where the LCD continuously shows the running
  stopwatch time (centisecond resolution via `Instant::elapsed()`). Keys are rebound
  per Phase 39's stopwatch keyboard mode: R/S toggles RUNSW/STOPSW, ENTER records a
  split (STPW), and Esc or any unbound key exits keyboard mode and restores normal
  operation.

- **OM behavior**: The OM does not specify `SW` as a callable XEQ entry point. Real
  HP-41CX with the Time Module does not have a keyboard-rebound interactive stopwatch
  mode — the user operates the stopwatch by calling RUNSW, STOPSW, RCLSW individually
  via XEQ-by-name or program execution.

- **Rationale**: An interactive stopwatch display is the natural UX equivalent of the
  HP-41CX's real-time clock display mode (CLOCK / CLKT) applied to the stopwatch
  subsystem. The HP Time Module OM describes the stopwatch as a live, continuously-
  updating subsystem on the hardware LCD; emulating that experience requires a
  dedicated keyboard mode since there is no background thread updating the TUI display.
  The emulator-extension classification preserves the "feature-complete per OM
  00041-90035" hard-claim discipline — `SW` is additive and does not interfere with
  any OM-specified behavior. Rejected alternative: no interactive mode (user calls
  RUNSW / STOPSW / RCLSW manually) — rejected because the hardware LCD showed a
  running stopwatch continuously, and the TUI/GUI should provide an equivalent.

- **See**: `hp41-core/src/ops/time/stopwatch.rs::op_sw` (sets stopwatch_keyboard_mode);
  `hp41-core/src/state.rs` (`stopwatch_keyboard_mode: bool` field);
  `docs/hp41-time-functions.json` SW entry inline `divergences` field (cross-reference
  per D-34.3 surgical-inline convention); Phase 39 CLI implementation (interactive
  keyboard loop); D-38.7 (38-CONTEXT.md — stopwatch state machine three-field design).

---

## 3. Behavioral Policies

*(Cross-cutting rules that are decisions worth documenting — not strictly numerical
divergences, but intentional implementation choices with OM basis or deliberate extension.
These entries document cases where the emulator made a specific policy decision that
affects behavior in ways the OM either specifies explicitly or leaves to the implementation.)*

---

### D-40-01: CORRECT / SETAF Accuracy Factor — Documented No-Op

- **OM citation**: HP 00041-90035 (1982), §CORRECT and §SETAF — the OM describes
  `CORRECT` as a command that applies a user-stored crystal accuracy correction factor
  (stored via `SETAF`, recalled via `RCLAF`) to the Time Module's internal crystal
  oscillator. The correction factor compensates for crystal drift measured over a
  calibration period. The OM's §CORRECT example measures drift over a fixed interval and
  uses the stored factor to keep the crystal accurate.

- **Our behavior**: `XEQ "CORRECT"` is a documented no-op: it applies
  `LiftEffect::Neutral` and returns `Ok(())` without modifying any state. `XEQ "SETAF"`
  stores stack X into `state.accuracy_factor: HpNum` (persistent, `#[serde(default)]`).
  `XEQ "RCLAF"` recalls `state.accuracy_factor` to stack X. The stored factor value
  persists across save/load cycles but has no effect on any clock computation.
  `state.time_offset_secs` is the sole mechanism that adjusts the displayed clock time;
  `accuracy_factor` is decoupled from it entirely (no CORRECT-driven offset update).

- **OM behavior**: On real HP-41CX hardware with the Time Module, `CORRECT` reads the
  stored accuracy factor and applies a micro-correction to the internal crystal oscillator
  circuit, effectively re-calibrating the hardware clock to eliminate accumulated drift.
  The correction is a physical hardware operation against an analog crystal circuit —
  it cannot be emulated without access to hardware timing primitives.

- **Rationale**: The host system clock (backed by NTP on modern systems) is authoritative
  and does not drift in the way a 1982 crystal oscillator does. CORRECT's crystal-
  correction semantics are physically meaningless in an emulator context — there is no
  analog oscillator to correct. Storing and recalling the accuracy factor via SETAF/RCLAF
  preserves OM-faithful data flow (users can run CORRECT-based calibration programs
  without crashes or errors) while not silently corrupting the time offset with a
  correction that has no physical basis. Rejected alternative: apply `accuracy_factor`
  as a seconds-per-day drift correction to `time_offset_secs` — rejected because NTP
  makes such correction meaningless and the arithmetic would require knowing elapsed
  "calibration time" which the emulator does not track. Documented as
  TIME-FMT-05 requirement in REQUIREMENTS.md.

- **See**: `hp41-core/src/ops/time/clock.rs::op_correct` (no-op implementation);
  `hp41-core/src/ops/time/clock.rs::op_setaf` (stores to accuracy_factor);
  `hp41-core/src/ops/time/clock.rs::op_rclaf` (recalls accuracy_factor);
  `hp41-core/src/state.rs` (`accuracy_factor: HpNum` field with `#[serde(default)]`);
  `docs/hp41-time-functions.json` CORRECT and SETAF/RCLAF entries inline `divergences`
  fields; D-38.1 (38-CONTEXT.md — SystemTime::now() host-clock decision);
  TIME-FMT-05 (REQUIREMENTS.md).

---

### D-40-02: Host System Clock Backing — SystemTime::now() vs. Crystal Oscillator

- **OM citation**: HP 00041-90035 (1982), §TIME, §DATE, §ATIME, §ADATE, §SETIME,
  §SETDATE — the OM describes a real-time clock module backed by an internal crystal
  oscillator that runs independently of the HP-41C calculator CPU. Time persists across
  calculator off/on cycles. SETIME and SETDATE set the hardware clock registers directly.

- **Our behavior**: `XEQ "TIME"`, `XEQ "DATE"`, `XEQ "ATIME"`, `XEQ "ATIME24"`,
  `XEQ "ADATE"` all call `SystemTime::now()` (via `adjusted_epoch_secs(state.time_offset_secs)`)
  to obtain the current time. The `time_offset_secs: i64` CalcState field (persistent,
  `#[serde(default)]`) stores a seconds delta computed by SETIME/SETDATE. All
  time-reading operations return `system_clock_secs + time_offset_secs`, converted
  through the Fliegel-Van Flandern JDN algorithm for date decomposition. CLKT / CLKTD
  display modes use the same pull-on-redraw pattern: `get_clock_display_str()` calls
  `SystemTime::now()` on each frontend redraw cycle (D-carried.7). There is no
  background thread maintaining clock state — the clock is computed fresh on each read.

- **OM behavior**: Real HP-41CX hardware maintains a continuously-running crystal
  oscillator that counts time independently of the calculator CPU. The hardware clock
  keeps accurate time even when the calculator is powered off (via a backup battery on
  the Time Module). The clock is read from hardware registers in the Time Module IC.

- **Rationale**: `std::time::SystemTime::now()` is the portable, standard-library
  mechanism for reading the host OS clock. On modern systems this is backed by
  high-resolution hardware timers synchronized to NTP; it is far more accurate than
  the 1982 crystal oscillator it replaces. The `time_offset_secs` field provides the
  same user-control semantics as SETIME/SETDATE on real hardware — the user can set
  the HP-41 clock to any desired time independently of the host OS time zone or clock
  settings. Zero new runtime dependencies: `chrono` and `time` crates were rejected per
  D-carried.1 (zero new runtime deps discipline from ADR-v3.1-002). Precedent:
  `hp41-core` already uses `std::time::Instant` (via stopwatch) and `Arc<AtomicBool>`
  (cancel_requested); `SystemTime` follows the same "value-returning syscall" pattern.

- **See**: `hp41-core/src/ops/time/clock.rs::adjusted_epoch_secs` (SystemTime::now()
  + offset call site); `hp41-core/src/ops/time/clock.rs::get_clock_display_str`
  (pull-on-redraw clock display); `hp41-core/src/state.rs` (`time_offset_secs: i64`
  field); D-38.1 (38-CONTEXT.md — SystemTime decision);
  D-38.2 (38-CONTEXT.md — time_offset_secs design);
  D-38.3 (38-CONTEXT.md — OS local time, hand-coded Gregorian arithmetic);
  D-carried.1 (38-CONTEXT.md — zero new runtime deps);
  D-carried.7 (38-CONTEXT.md — pull-on-redraw architecture).

---

### D-40-03: Stopwatch Freeze-on-Save — Running State Not Serializable

- **OM citation**: HP 00041-90035 (1982), §RUNSW, §STOPSW, §RCLSW — the OM describes
  the stopwatch as a continuously-running hardware timer that accumulates time
  independently of calculator CPU cycles. The hardware stopwatch runs during calculator
  off/on cycles and across power interruptions (via backup battery), transparent to the
  user. Saving and restoring a running stopwatch across sessions is implicitly supported
  by the hardware design.

- **Our behavior**: When the calculator state is saved with the stopwatch in Running
  mode, the `CalcState::migrate_after_load()` function transitions the stopwatch from
  Running to Stopped. The accumulated time up to the moment of save (`stopwatch_accumulated: f64`,
  persistent via `#[serde(default)]`) is preserved. The partial lap since the last
  `RUNSW` is lost (the `stopwatch_start: Option<Instant>` field carries `#[serde(skip)]`
  — `Instant` is not serializable). After loading a previously-running stopwatch,
  `RCLSW` returns the frozen accumulated time; the user must call `RUNSW` explicitly
  to resume timing.

- **OM behavior**: The real HP-41CX Time Module's stopwatch hardware accumulates time
  continuously, across save/load cycles and power cycles, without user intervention.
  A running stopwatch on real hardware would continue counting through a memory save
  and would show the correct (larger) elapsed time when recalled after the save — there
  is no "freeze on save" on real hardware.

- **Rationale**: `std::time::Instant` cannot be serialized — it is an opaque monotonic
  clock value with no cross-session or cross-platform meaning. Pretending to track time
  across sessions by saving a wall-clock timestamp and reconstructing elapsed time on
  load would be silently wrong (monotonic clock discontinuities, sleep/hibernate gaps,
  time zone issues). The freeze-on-save policy is honest: the user sees the correctly-
  accumulated time up to the save point, and a clear implicit signal that timing has
  stopped (they must RUNSW to resume). Rejected alternative: save `stopwatch_running_since_unix: i64`
  and reconstruct elapsed as `(now - saved_unix) + accumulated` — rejected because
  `SystemTime` is not monotonic (NTP jumps, DST changes, VM migration can cause
  negative deltas), which would produce a corrupt stopwatch reading. Rejected
  alternative: mark the stopwatch as running in the save file and silently claim no
  time passed during the gap — rejected as misleading (silent data inaccuracy).
  D-38.6 documents this as the canonical disposition.

- **See**: `hp41-core/src/ops/time/stopwatch.rs` (`StopwatchMode`, `stopwatch_start`,
  `stopwatch_accumulated`); `hp41-core/src/state.rs::migrate_after_load` (Running →
  Stopped transition on load); `hp41-core/src/state.rs` (`stopwatch_start: Option<Instant>`
  field with `#[serde(default, skip)]`); `hp41-core/src/state.rs`
  (`stopwatch_accumulated: f64` field with `#[serde(default)]`);
  D-38.6 (38-CONTEXT.md — freeze-on-save canonical decision);
  D-38.7 (38-CONTEXT.md — three-field stopwatch state machine design).

---

### D-40-04: Interrupting Control Alarm Deferral — Re-Entrancy Not Supported

- **OM citation**: HP 00041-90035 (1982), §XYZALM — the OM describes three alarm types:
  message alarms (display text on trigger), non-interrupting control alarms (`>label` prefix —
  execute the named program label on acknowledgment), and interrupting control alarms
  (`>>label` prefix — execute the named program label immediately upon trigger, interrupting
  any running program). The `>>` prefix convention for interrupting control alarms is
  documented in the OM's XYZALM section.

- **Our behavior**: Message alarms and non-interrupting control alarms (`>label`) are
  fully implemented. When a past-due non-interrupting control alarm is acknowledged
  (via the `check_alarms` drain), the label is queued in `event_buffer` for the frontend
  to XEQ on next user interaction — no re-entrancy hazard because execution happens at
  a user-interaction boundary, not during a running program. Interrupting control alarms
  (`>>label`) are stored in the alarm catalog with full fidelity (`AlarmType::Control {
  label, interrupting: true }` per D-38.8) but the `interrupting: true` flag is not
  acted upon — the alarm fires as a non-interrupting message alarm instead, without
  executing the stored label mid-program. No program re-entrancy against the 4-level
  call stack is attempted.

- **OM behavior**: On real HP-41CX hardware, an interrupting control alarm (`>>label`)
  halts the currently-executing program at the next instruction boundary, saves the
  execution state, XEQs the alarm's stored label program, and resumes the interrupted
  program when the alarm program completes (subject to the 4-level call stack limit).
  This is analogous to a hardware interrupt request handled in firmware.

- **Rationale**: Implementing hardware-interrupt semantics against the emulator's
  synchronous `run_program` / `run_loop` dispatch model would require either (a) a
  pre-emption point in every instruction's dispatch loop (adding latency to every op),
  or (b) a parallel execution thread with shared mutable state on `CalcState` (requiring
  `Arc<Mutex<CalcState>>` or `Arc<RwLock<CalcState>>` throughout, a major architectural
  change). Both alternatives introduce complexity that outweighs the benefit — the real
  HP-41CX's 4-level call stack makes interrupting control alarm programs very limited in
  scope anyway. The data model (`interrupting: bool`) is forward-compatible: a future
  phase can implement the semantics without a schema migration. D-38.4 documents this
  as the canonical disposition; the divergence is documented here per TIME-DOC-02.

- **See**: `hp41-core/src/ops/time/alarm.rs::AlarmType` (`Control { label, interrupting }`
  enum variant); `hp41-core/src/ops/time/alarm.rs::check_alarms` (alarm drain — fires
  non-interrupting alarms only); `hp41-core/src/ops/time/alarm.rs::parse_alarm_type`
  (`>>` prefix parsing into `interrupting: true`); `hp41-core/src/state.rs`
  (`alarms: Vec<AlarmEntry>` field with `#[serde(default)]`);
  D-38.4 (38-CONTEXT.md — interrupting control alarm deferral decision);
  D-38.5 (38-CONTEXT.md — non-interrupting control alarm execution via acknowledgment);
  D-38.8 (38-CONTEXT.md — AlarmType enum design with forward-compat interrupting field).

---

### D-40-05: Centisecond Stopwatch Resolution — Instant::elapsed() vs. Hardware Timer

- **OM citation**: HP 00041-90035 (1982), §RCLSW, §SETSW — the OM specifies that the
  stopwatch displays and returns time in HH.MMSScc format (hours, minutes, seconds,
  centiseconds). The OM-quoted resolution is centiseconds (0.01 second). The HP-41CX
  Time Module's hardware stopwatch timer has a 10ms (centisecond) resolution per the
  OM's display specification.

- **Our behavior**: The stopwatch tracks elapsed time internally as `f64` seconds via
  `std::time::Instant::elapsed().as_secs_f64()`. The resolution of `Instant::elapsed()`
  is platform-dependent but is typically sub-microsecond on modern hardware (macOS M1:
  ~1 ns; Ubuntu 22.04: ~1 µs; Windows 10: ~100 ns). The elapsed `f64` is converted to
  HH.MMSScc HpNum via integer arithmetic that truncates to centiseconds (by multiplying
  the fractional-second component by 100 and taking the floor). RCLSW and SWPT both
  return values with centisecond precision matching the OM format. SETSW accepts HH.MMSScc
  input with centisecond granularity per the HP-41 decimal format convention.

- **OM behavior**: The real HP-41CX Time Module stopwatch has a hardware centisecond
  counter that increments every 10ms. RCLSW returns the accumulated centiseconds as
  HH.MMSScc. The hardware resolution is exactly 0.01 seconds (centiseconds) — no
  sub-centisecond measurement.

- **Rationale**: Using `Instant::elapsed().as_secs_f64()` provides significantly higher
  resolution than the OM's centisecond hardware — the emulator is MORE precise, not
  less. The centisecond truncation in the HH.MMSScc conversion is OM-faithful (matches
  the hardware display format exactly). Users cannot perceive sub-centisecond differences
  in stopwatch readings; the higher underlying precision makes the emulator immune to
  rounding accumulation over long stopwatch sessions. The `f64` storage format provides
  ~15 significant decimal digits, which at centisecond output precision is effectively
  lossless for any session duration a user would reasonably time. Zero additional
  complexity: `Instant` is already a transitive dependency of `hp41-core` via the
  cancel_requested infrastructure and the stopwatch implementation.

- **See**: `hp41-core/src/ops/time/stopwatch.rs::current_elapsed` (`Instant::elapsed().as_secs_f64()`);
  `hp41-core/src/ops/time/stopwatch.rs::secs_to_hpnum_time` (f64-to-HH.MMSScc
  centisecond truncation); `hp41-core/src/state.rs`
  (`stopwatch_accumulated: f64`, `stopwatch_start: Option<Instant>`);
  D-38.7 (38-CONTEXT.md — stopwatch state machine design with f64 accumulated secs);
  HP 00041-90035 (1982) §RCLSW (centisecond resolution specification).

---

*Last updated: 2026-05-25. Catalog established in Plan 40-01 (Phase 40 / TIME-DOC-02).*
