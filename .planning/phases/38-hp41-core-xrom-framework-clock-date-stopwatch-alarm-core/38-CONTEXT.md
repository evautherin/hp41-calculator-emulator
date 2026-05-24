# Phase 38: hp41-core — XROM Framework + Clock/Date/Stopwatch/Alarm Core - Context

**Gathered:** 2026-05-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 38 delivers all ~33 Time Pac `Op` variants in `hp41-core` — XROM 26 registration, system-clock-backed TIME/DATE, JDN date arithmetic (DATE+/DDAYS/DOW), stopwatch state machine (RUNSW/STOPSW/SETSW/RCLSW/SWPT/SW), alarm catalog (XYZALM/RCLALM/ALMCAT/CLALMA/CLALMX/CLRALMS/ALMNOW), clock display mode flags (CLKT/CLKTD/CLK12/CLK24), time/date formatting in ALPHA (ADATE/ATIME/ATIME24), clock adjustment (SETIME/SETDATE/CORRECT/T+X/SETAF/RCLAF), and date format control (DMY/MDY). This phase completes 4-way invariant items 1+2 (dispatch + execute_op); items 3+4 (CLI + GUI `op_display_name`) are intentionally deferred to Phases 39/41 with a sanctioned CI break.

</domain>

<decisions>
## Implementation Decisions

### Clock Access Pattern
- **D-38.1:** Direct `std::time::SystemTime::now()` in hp41-core — core calls the system clock directly. Precedent: core already uses `Arc<AtomicBool>` (cancel_requested) and `Instant` is a transitive dep. SystemTime is a value-returning syscall, not console I/O. The zero-I/O principle applies to console/filesystem/network, not to clock reads.
- **D-38.2:** Single `time_offset_secs: i64` persistent CalcState field with `#[serde(default)]`. SETIME computes delta between entered time and current SystemTime, stores as seconds. All time/date reads apply `SystemTime::now() + offset` to derive the "HP-41 time".
- **D-38.3:** OS local time, not UTC. Use `libc::localtime_r` (Unix) / `GetLocalTime` (Windows) to convert SystemTime to local. Matches real HP-41CX behavior (no timezone concept). Cross-platform via std transitive deps.

### Control Alarm Scope
- **D-38.4:** Interrupting control alarms DEFERRED as a documented divergence. Message alarms + non-interrupting control alarms are fully in scope. Interrupting control alarm execution requires re-entrancy against the 4-level call stack, which is not supported. Document in `docs/hp41-time-divergences.md`.
- **D-38.5:** Non-interrupting control alarms XEQ the stored program label on acknowledgment. Uses existing XEQ dispatch infrastructure, no re-entrancy hazard.

### Stopwatch Save/Load
- **D-38.6:** Freeze elapsed time on save. On save: compute `elapsed = Instant::elapsed() + accumulated`, store as persistent `stopwatch_accumulated: f64`. On load: stopwatch is STOPPED with accumulated time preserved. User must RUNSW to resume. Rationale: `Instant` cannot serialize, and pretending to track time across sessions is misleading.
- **D-38.7:** Stopwatch state uses three separate CalcState fields: `stopwatch_mode: StopwatchMode` (enum Idle/Running/Stopped, persistent with `#[serde(default)]`), `stopwatch_accumulated: f64` (persistent with `#[serde(default)]`), `stopwatch_start: Option<Instant>` (transient with `#[serde(default, skip)]`). Split-point time stored as `stopwatch_split: f64` (persistent with `#[serde(default)]`).

### Alarm Catalog Shape
- **D-38.8:** Typed `AlarmType` enum: `Message(String)` carries the ALPHA message, `Control { label: String, interrupting: bool }` carries the program label. Forward-compatible for when interrupting alarms get implemented — the data model supports storing them even though execution is deferred.
- **D-38.9:** `check_alarms()` called by frontend after every dispatch via the drain pattern (same as `print_buffer` and `event_buffer`). Alarm notifications pushed into `event_buffer` for frontend rendering. Matches TIME-ALM-08 requirement: "on each keypress/dispatch".
- **D-38.10:** Direct `alarms: Vec<AlarmEntry>` on CalcState with `#[serde(default)]`. Free functions in `alarm.rs` operate on `&mut Vec<AlarmEntry>`. Matches existing patterns: `regs` is `Vec<HpNum>`, `programs` is `Vec<Program>`.
- **D-38.11:** Repeat interval stored as `i64` seconds. Convert from HH.MMSSss to seconds once at XYZALM entry time. No rounding drift risk over repeated reschedules.

### Prior Decisions (carried forward)
- **D-carried.1:** Zero new runtime dependencies — `std::time::{SystemTime, Instant}` + hand-coded Fliegel-Van Flandern JDN (~60 LOC). `chrono`/`time` crates rejected per ADR-v3.1-002 discipline.
- **D-carried.2:** Date decimal parsing uses string-split-at-decimal (ISG/DSE precedent), never float arithmetic. Left-pad fractional part to exactly 6 chars, split into DD[2] + YYYY[4] for MDY, or MM[2] + YYYY[4] for DMY.
- **D-carried.3:** Flag 31 = sole DMY/MDY control. `DMY` → `SF 31`, `MDY` → `CF 31`. No separate `date_format` field on CalcState.
- **D-carried.4:** `ModalProgram::Time(TimeStep)` variant following the Stat 1 pattern (ADR-v3.1-005). `TimeStep` enum lives in new `time/modal.rs` (outside math1/ freeze boundary).
- **D-carried.5:** XROM ID 26 with bit-2 arm in `xrom_resolve`. `default_xrom_modules() → 0b0000_0111`. `migrate_after_load()` upgrades v3.1 saves (`0b0000_0011` → `0b0000_0111`).
- **D-carried.6:** Module tree: `hp41-core/src/ops/time/` as sibling to `math1/` and `stat1/`. Free42 disclaim header on all files.
- **D-carried.7:** "Pull on redraw" architecture for live display — hp41-core remains thread-free and async-free. Frontend redraw cycle computes time-dependent display state on-demand.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Owner's Manual & Hardware References
- `docs/adr/` — existing ADRs; v3.2 ADRs will be added in Phase 40
- `docs/architecture-history.md` — full phase-by-phase narrative + decision rationale

### XROM Framework Precedent
- `hp41-core/src/ops/math1/xrom.rs` — `XromModule` struct, `MATH_1` const, `xrom_resolve()` — the pattern to follow for `TIME_MODULE` registration
- `hp41-core/src/ops/stat1/mod.rs` — Stat 1 XROM registration + named register consts — second XROM module precedent
- `hp41-core/src/ops/math1/modal.rs` — `ModalProgram` enum with `Stat1(Stat1Step)` variant — extend with `Time(TimeStep)`
- `hp41-core/src/ops/stat1/modal.rs` — `Stat1Step` enum + dispatch — pattern for `TimeStep`
- `docs/adr/v3.1-004-math1-freeze-second-carve-out.md` — sanctioned carve-outs in math1/modal.rs
- `docs/adr/v3.1-005-modalprogram-stat1-enum-extension.md` — how ModalProgram was extended for Stat 1

### CalcState Patterns
- `hp41-core/src/state.rs` — CalcState struct, `migrate_after_load()`, serde invariants, `default_xrom_modules()`
- `hp41-core/src/ops/program.rs` — `parse_counter()` for string-split-at-decimal precedent (ISG/DSE)

### Research
- `.planning/research/SUMMARY.md` — v3.2 research summary; recommended stack, architecture approach, critical pitfalls
- `.planning/research/FEATURES.md` — 33 callable functions confirmed from HP 82182A QRC
- `.planning/research/PITFALLS.md` — P32 (live display), P34 (clock in core), P35 (date parsing), P37 (alarm state machine), P41 (flag 31)

### Requirements & Roadmap
- `.planning/REQUIREMENTS.md` — 48 requirements mapped to Phase 38 (TIME-FW-01..06, TIME-CLK-01..06, TIME-DAT-01..06, TIME-DSP-01..05, TIME-FMT-01..05, TIME-SW-01..09, TIME-ALM-01..12)
- `.planning/ROADMAP.md` — Phase 38 goal, success criteria, depends-on

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `XromModule` struct in `math1/xrom.rs` — reuse directly for `TIME_MODULE` const
- `xrom_resolve()` in `math1/xrom.rs` — extend with bit-2 arm for Time Module
- `ModalProgram` enum in `math1/modal.rs` — extend with `Time(TimeStep)` variant
- `migrate_after_load()` in `state.rs` — extend with v3.1→v3.2 xrom_modules migration
- `parse_counter()` in `program.rs` — string-split-at-decimal pattern for date format parsing
- `event_buffer: Vec<String>` drain pattern — reuse for alarm notifications
- `print_buffer: Vec<String>` drain pattern — reuse for alarm message display

### Established Patterns
- Serde invariants: `#[serde(default)]` for persistent new fields, `#[serde(default, skip)]` for transient
- Free42 disclaim header on every source file in XROM module directories
- XROM bit numbering: bit 0 = Math 1 (XROM 7), bit 1 = Stat 1 (XROM 2), bit 2 = Time (XROM 26)
- 4-way exhaustive-match invariant: dispatch() + execute_op() + CLI prgm_display + GUI prgm_display
- Sanctioned CI break: items 3+4 deferred to Phases 39/41 (matches v3.0/v3.1 cadence)

### Integration Points
- `hp41-core/src/ops/mod.rs` — `Op` enum (new Time variants), `dispatch()` routing
- `hp41-core/src/ops/program.rs` — `execute_op()` routing
- `hp41-core/src/state.rs` — new CalcState fields, migration, default functions
- `hp41-cli/src/keys.rs:464` — `xrom_resolve(name, xrom_modules)` call site (Phase 39 wires bit-2)
- `scripts/check-free42-contamination.sh` — extend to cover `time/` directory

</code_context>

<specifics>
## Specific Ideas

No specific requirements — decisions above fully capture implementation direction. The research SUMMARY.md recommends a 7-file module tree: `mod.rs`, `clock.rs`, `date_arith.rs`, `alpha_time.rs`, `alarm.rs`, `stopwatch.rs`, `modal.rs`.

</specifics>

<deferred>
## Deferred Ideas

- **Interrupting control alarm execution** — requires call-stack re-entrancy not currently supported. Stored in alarm catalog data model (D-38.8 forward-compatible) but execution deferred as documented divergence.
- **Cycle-accurate crystal oscillator simulation** — no user value; system clock is superior.
- **HP-IL alarm wake-up** — no OFF state in emulator; HP-IL permanently excluded.
- **Accuracy factor correction loop** — host OS clock is NTP-synchronized; SETAF/RCLAF store value, CORRECT is documented no-op divergence.

None — discussion stayed within phase scope (deferred items above are from research, not scope creep).

</deferred>

---

*Phase: 38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core*
*Context gathered: 2026-05-24*
