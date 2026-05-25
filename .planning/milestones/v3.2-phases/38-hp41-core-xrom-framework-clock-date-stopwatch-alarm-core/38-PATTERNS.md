# Phase 38: hp41-core XROM Framework + Clock/Date/Stopwatch/Alarm Core - Pattern Map

**Mapped:** 2026-05-24
**Files analyzed:** 12 new/modified files
**Analogs found:** 12 / 12

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `hp41-core/src/ops/time/mod.rs` | module-hub | CRUD | `hp41-core/src/ops/stat1/mod.rs` | exact |
| `hp41-core/src/ops/time/clock.rs` | service | request-response | `hp41-core/src/ops/stat1/rand.rs` | role-match |
| `hp41-core/src/ops/time/date_arith.rs` | utility | transform | `hp41-core/src/ops/program.rs` (`parse_counter`) | partial-match |
| `hp41-core/src/ops/time/alpha_time.rs` | utility | transform | `hp41-core/src/ops/stat1/rand.rs` | role-match |
| `hp41-core/src/ops/time/alarm.rs` | service | event-driven | `hp41-core/src/ops/stat1/normd.rs` | role-match |
| `hp41-core/src/ops/time/stopwatch.rs` | service | CRUD | `hp41-core/src/ops/stat1/rand.rs` | role-match |
| `hp41-core/src/ops/time/modal.rs` | modal-state | request-response | `hp41-core/src/ops/stat1/modal.rs` | exact |
| `hp41-core/src/ops/math1/xrom.rs` (modify) | registry | CRUD | `hp41-core/src/ops/math1/xrom.rs` | self |
| `hp41-core/src/ops/math1/modal.rs` (modify) | modal-carrier | request-response | `hp41-core/src/ops/math1/modal.rs` | self |
| `hp41-core/src/ops/mod.rs` (modify) | dispatch | CRUD | `hp41-core/src/ops/mod.rs` | self |
| `hp41-core/src/ops/program.rs` (modify) | executor | CRUD | `hp41-core/src/ops/program.rs` | self |
| `hp41-core/src/state.rs` (modify) | model | CRUD | `hp41-core/src/state.rs` | self |
| `scripts/check-free42-contamination.sh` (modify) | config | batch | `scripts/check-free42-contamination.sh` | self |

---

## Pattern Assignments

### `hp41-core/src/ops/time/mod.rs` (module-hub)

**Analog:** `hp41-core/src/ops/stat1/mod.rs`

**Disclaim header + module docstring pattern** (lines 1-17):
```rust
// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `time` — HP Time Module (XROM 26) operations.
//!
//! XROM module id: 26 (bit 2 of `CalcState::xrom_modules`).
//! Activated in Phase 38 (v3.2).
```

**Named register consts pattern** (stat1/mod.rs lines 19-90 is the template):
- The `time/mod.rs` header doubles as the OM register-layout source-of-truth: if the Time Module OM documents any storage register layout, declare named consts here (same as `STAT1_AOV_*_REG` in stat1/mod.rs).
- For Phase 38 scope, the Time Module has no named data-register block (alarms are on `CalcState.alarms`, stopwatch on dedicated fields). The `mod.rs` header should still include the OM 82182A "Storage Registers" section note (even if it states "none used").

**Public re-export pattern** (stat1/mod.rs lines 88-130): re-export per-submodule `op_*` functions and types needed by `execute_op` in `ops/program.rs`.

---

### `hp41-core/src/ops/time/clock.rs` (service, request-response)

**Analog:** `hp41-core/src/ops/stat1/rand.rs`

**Disclaim header** (rand.rs lines 1-3):
```rust
// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
```

**Import pattern** (rand.rs lines 40-44):
```rust
use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;
use rust_decimal::Decimal;
```

**Op function signature pattern** (rand.rs lines 67-70 / per-crate convention):
```rust
pub fn op_time(state: &mut CalcState) -> Result<(), HpError> {
    // read SystemTime::now(), apply time_offset_secs, convert to HH.MMSSss
    // push result onto stack X via enter_number + apply_lift_effect
}
```

**Stack push pattern** (all stat1 ops use `enter_number` + `apply_lift_effect`):
```rust
let result = /* ... compute HpNum ... */;
enter_number(state, result);
apply_lift_effect(state, LiftEffect::Enable);
Ok(())
```

---

### `hp41-core/src/ops/time/date_arith.rs` (utility, transform)

**Analog:** `hp41-core/src/ops/program.rs` (`parse_counter` pattern)

**String-split-at-decimal pattern** (program.rs lines 1103-1123) — the ISG/DSE precedent:
```rust
// D-carried.2 / P35: NEVER use float arithmetic for field extraction.
pub fn parse_counter(n: &HpNum) -> Result<(i64, i64, i64, String), HpError> {
    let s = n.inner().to_string();
    let (int_part, frac_part) = if let Some(pos) = s.find('.') {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s.as_str(), "")
    };
    let current: i64 = int_part.parse().map_err(|_| HpError::InvalidOp)?;
    // Pad RIGHT with zeros to exactly 5 chars (trailing-zero normalisation fix)
    let frac_padded = format!("{frac_part:0<5}");
    // ...
}
```

**Date parse adaptation** (from RESEARCH.md Pattern 6 — mirrors the above exactly, LEFT-pads to 6 chars for date):
```rust
fn parse_date_hpnum(hpnum: &HpNum, dmy: bool) -> Result<(i32, i32, i32), HpError> {
    let s = hpnum.inner().to_string();
    let (int_part, frac_part) = if let Some(pos) = s.find('.') {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s.as_str(), "")
    };
    let first = int_part.parse::<u8>().map_err(|_| HpError::Data)?;
    // LEFT-pad fractional part to exactly 6 digits (preserves leading zeros for year).
    // NOTE: parse_counter uses right-pad ("{:0<5}"); dates use LEFT-pad ("{:0>6}").
    let padded = format!("{:0>6}", frac_part);
    // ...
}
```

**Time parse (HH.MMSSss — P44 — do NOT reuse hms.rs):**
```rust
fn parse_time_hpnum(hpnum: &HpNum) -> Result<(u8, u8, u8, u8), HpError> {
    let s = hpnum.inner().to_string();
    let (int_part, frac_part) = if let Some(pos) = s.find('.') {
        (&s[..pos], &s[pos + 1..])
    } else {
        (s.as_str(), "")
    };
    let hours = int_part.parse::<u8>().map_err(|_| HpError::Data)?;
    let padded = format!("{:0>6}", frac_part);  // 6 digits: MMSScc
    // Split: MM = [0..2], SS = [2..4], cc (centisecs) = [4..6]
}
```

**Error type:** Use `HpError::Data` for out-of-range field values (consistent with stat1 data validation pattern). Use `HpError::InvalidOp` for parse failures (consistent with `parse_counter`).

---

### `hp41-core/src/ops/time/alpha_time.rs` (utility, transform)

**Analog:** `hp41-core/src/ops/stat1/rand.rs` (simple op-function file pattern)

**Disclaim header + imports:** same as `clock.rs` above.

**Op function signature:**
```rust
pub fn op_atime(state: &mut CalcState) -> Result<(), HpError> {
    // Reads X register (HH.MMSSss), formats as time string, appends to state.alpha_reg.
    // Does NOT push to stack — ALPHA append is an in-place mutation.
    // Check state.clock_12h for 12/24h formatting.
}

pub fn op_atime24(state: &mut CalcState) -> Result<(), HpError> {
    // Identical to op_atime but ignores state.clock_12h (always 24-hour).
}

pub fn op_adate(state: &mut CalcState) -> Result<(), HpError> {
    // Reads X register (date decimal per Flag 31), formats, appends to state.alpha_reg.
    // Check state.flags & (1 << 31) for DMY/MDY.
}
```

**ALPHA append pattern** (from existing alpha.rs — use `state.alpha_reg.push_str(&formatted)` pattern, truncating to 24 chars per HP-41 ALPHA register limit).

---

### `hp41-core/src/ops/time/alarm.rs` (service, event-driven)

**Analog:** `hp41-core/src/ops/stat1/normd.rs` (multi-function service file)

**Disclaim header + imports:** same as `clock.rs`.

**AlarmEntry + AlarmType struct pattern** (D-38.8 / D-38.10; follows serde derive convention on all CalcState-persisted types):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmEntry {
    pub trigger_unix: i64,
    pub repeat_secs: i64,
    pub alarm_type: AlarmType,
    pub past_due: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlarmType {
    Message(String),
    Control { label: String, interrupting: bool },
}
```

**Free function operating on `&mut Vec<AlarmEntry>` pattern** (D-38.10 — parallel to how stat1 ops operate on `&mut CalcState` registers directly):
```rust
pub fn op_clralms(state: &mut CalcState) -> Result<(), HpError> {
    state.alarms.clear();
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**check_alarms drain pattern** (D-38.9 — mirrors `print_buffer` drain contract):
```rust
// Called by frontend after every dispatch() — same cadence as print_buffer drain.
// Pushes to state.event_buffer (existing transient Vec<String>, #[serde(default, skip)]).
pub fn check_alarms(state: &mut CalcState) {
    // compare alarm.trigger_unix against SystemTime::now() + time_offset_secs
    // push "alarm:message:<text>" or "alarm:xeq:<label>" into state.event_buffer
    // push message text into state.print_buffer for CLI display
}
```

**ALPHA prefix parsing for XYZALM** (check `>>` FIRST — longer match wins):
```rust
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

---

### `hp41-core/src/ops/time/stopwatch.rs` (service, CRUD)

**Analog:** `hp41-core/src/ops/stat1/rand.rs` (simple service with state machine)

**StopwatchMode enum** (D-38.7 — follows CalcState enum convention from `AngleMode`, `DisplayMode` in state.rs):
```rust
// Declared here; CalcState holds `stopwatch_mode: StopwatchMode`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum StopwatchMode {
    Idle,
    Running,
    Stopped,
}

impl Default for StopwatchMode {
    fn default() -> Self { StopwatchMode::Idle }
}
```

**Stopwatch RUNSW pattern** (D-38.7 — `stopwatch_start: Option<Instant>` is `#[serde(default, skip)]`):
```rust
pub fn op_runsw(state: &mut CalcState) -> Result<(), HpError> {
    state.stopwatch_mode = StopwatchMode::Running;
    state.stopwatch_start = Some(std::time::Instant::now());
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

**Stopwatch RCLSW pattern** (compute elapsed including current run if running):
```rust
pub fn op_rclsw(state: &mut CalcState) -> Result<(), HpError> {
    let elapsed_secs = match state.stopwatch_mode {
        StopwatchMode::Running => {
            let start = state.stopwatch_start.unwrap_or_else(std::time::Instant::now);
            state.stopwatch_accumulated + start.elapsed().as_secs_f64()
        }
        _ => state.stopwatch_accumulated,
    };
    // Convert elapsed_secs to HH.MMSScc HpNum, push to X
    let result = secs_to_hpnum_time(elapsed_secs)?;
    enter_number(state, result);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
```

**STOPSW freeze-on-stop pattern** (D-38.6):
```rust
pub fn op_stopsw(state: &mut CalcState) -> Result<(), HpError> {
    if let StopwatchMode::Running = state.stopwatch_mode {
        if let Some(start) = state.stopwatch_start.take() {
            state.stopwatch_accumulated += start.elapsed().as_secs_f64();
        }
        state.stopwatch_mode = StopwatchMode::Stopped;
    }
    apply_lift_effect(state, LiftEffect::Neutral);
    Ok(())
}
```

---

### `hp41-core/src/ops/time/modal.rs` (modal-state, request-response)

**Analog:** `hp41-core/src/ops/stat1/modal.rs` — **exact match**

**Disclaim header + module docstring** (stat1/modal.rs lines 1-17):
```rust
// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Modal state-machine for Time Module prompt-driven workflows.
//!
//! `TimeStep` is the per-program step enum carried by the
//! `ModalProgram::Time(TimeStep)` variant living in
//! `hp41-core/src/ops/math1/modal.rs` (math1/ freeze exception D-carried.4).
```

**Imports** (stat1/modal.rs lines 71-73):
```rust
use crate::error::HpError;
use crate::state::CalcState;
```

**TimeStep enum pattern** (stat1/modal.rs lines 80-137 is the template):
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TimeStep {
    /// SETIME — awaiting time entry in X (HH.MMSSss). Prompt: "TIME?"
    SetTimePrompt,
    /// SETDATE — awaiting date entry in X (per Flag 31). Prompt: "DATE?"
    SetDatePrompt,
    /// XYZALM — step 1: awaiting time in X. Prompt: "ALARM TIME?"
    XyzalmTimePrompt,
    // (additional multi-step XYZALM variants as needed)
}
```

**Three dispatch functions pattern** (stat1/modal.rs lines 153, 333, 353):
```rust
/// Per-step submit dispatch — called by ModalProgram::Time dispatch arm.
pub fn submit_step(state: &mut CalcState, step: TimeStep) -> Result<(), HpError> {
    match step {
        TimeStep::SetTimePrompt => { /* parse X, compute offset, clear modal */ }
        TimeStep::SetDatePrompt => { /* parse X per Flag 31, adjust offset, clear modal */ }
        TimeStep::XyzalmTimePrompt => { /* parse X, advance to next alarm step */ }
    }
}

/// Per-step prompt accessor.
pub fn current_prompt(step: &TimeStep) -> Option<String> {
    match step {
        TimeStep::SetTimePrompt => Some("TIME?".to_string()),
        TimeStep::SetDatePrompt => Some("DATE?".to_string()),
        TimeStep::XyzalmTimePrompt => Some("ALARM TIME?".to_string()),
    }
}

/// Per-step alpha-label gate.
pub fn requires_alpha_label(step: &TimeStep) -> bool {
    match step {
        TimeStep::SetTimePrompt | TimeStep::SetDatePrompt | TimeStep::XyzalmTimePrompt => false,
    }
}
```

**submit_step modal-clear pattern** (stat1/modal.rs lines 168-170 — clear BEFORE dispatch):
```rust
state.modal_program = None;
state.modal_prompt = None;
// then call op_*() or return Ok(())
```

---

### `hp41-core/src/ops/math1/xrom.rs` (modify — THIRD freeze exception)

**Self-analog.** This file is modified in-place following the Phase 33 Stat 1 precedent.

**TIME_MODULE const pattern** (MATH_1 const lines 43-118 + STAT_1 const lines 141-186 as templates):
```rust
/// Time Module registry (D-carried.5 freeze exception — third Stat-related
/// entry in this otherwise-frozen file, alongside the bit-2 arm in
/// `xrom_resolve` below).
///
/// - `id = 26` — HP hardware Time Module XROM module ID (HP 82182A).
/// - `name = "TIME 2C"` — CATALOG 2 display string.
/// - `ops` — 35 entries (RESEARCH.md Pattern 1 / FEATURES.md).
pub const TIME_MODULE: XromModule = XromModule {
    id: 26,
    name: "TIME 2C",
    ops: &[
        ("ADATE",   Op::TimeAdate),
        ("ALMCAT",  Op::TimeAlmcat),
        // ... all ~35 entries ...
    ],
};
```

**xrom_resolve bit-2 extension** (xrom.rs lines 195-212 — extend by appending AFTER the stat1 arm):
```rust
pub fn xrom_resolve(name: &str, modules: u8) -> Option<Op> {
    if modules & 0b0000_0001 != 0 {
        if let Some(op) = math1_resolve(name) { return Some(op); }
    }
    if modules & 0b0000_0010 != 0 {
        if let Some(op) = stat1_resolve(name) { return Some(op); }
    }
    // Phase 38 (v3.2): Time Module bit-2 arm — fires LAST per Pitfall 1 / D-carried.5.
    if modules & 0b0000_0100 != 0 {
        if let Some(op) = time_resolve(name) { return Some(op); }
    }
    None
}

fn time_resolve(name: &str) -> Option<Op> {
    match name {
        "ADATE" => Some(Op::TimeAdate),
        // ... all ~35 entries ...
        _ => None,
    }
}
```

**Test pattern for new module** (xrom.rs lines 484-556 — `stat1_const_id_and_name` + `stat1_ops_has_correct_entry_count` + `resolve_uses_bit_1_for_stat1` as templates):
```rust
#[test]
fn time_module_const_id_and_name() {
    assert_eq!(TIME_MODULE.id, 26, "TIME_MODULE.id must be 26 (HP 82182A)");
    assert_eq!(TIME_MODULE.name, "TIME 2C");
}

#[test]
fn time_module_ops_has_correct_entry_count() {
    assert_eq!(TIME_MODULE.ops.len(), 35, "TIME_MODULE.ops must carry exactly 35 entries");
}

#[test]
fn resolve_uses_bit_2_for_time_module() {
    let with_bit2 = xrom_resolve("TIME", 0b0000_0100);
    assert_eq!(with_bit2, Some(Op::TimeTime));
    let with_bit0_only = xrom_resolve("TIME", 0b0000_0001);
    assert!(with_bit0_only.is_none(), "bit-2 isolation: bit-0 alone must not resolve Time ops");
}
```

---

### `hp41-core/src/ops/math1/modal.rs` (modify — add Time variant)

**Self-analog with Stat1 variant as exact template** (modal.rs lines 39-51 and 74-104).

**Add Time variant to ModalProgram enum** (after line 51, following exact same comment pattern as Stat1):
```rust
/// Time Module workflows (Phase 38 — SETIME / SETDATE / XYZALM prompts).
///
/// math1/ freeze exception per D-carried.4: this single additive variant +
/// three dispatch arms is the THIRD freeze-carve-out for `math1/modal.rs`,
/// alongside `math1/xrom.rs` (D-33.3) and `ModalProgram::Stat1` (D-33.3b).
/// All Time-specific semantics live in `hp41-core/src/ops/time/modal.rs`.
Time(crate::ops::time::modal::TimeStep),
```

**Extend current_prompt match** (modal.rs lines 65-77):
```rust
// D-carried.4: Time Module modal prompts delegate to time::modal.
ModalProgram::Time(step) => crate::ops::time::modal::current_prompt(step),
```

**Extend requires_alpha_label match** (modal.rs lines 97-104):
```rust
ModalProgram::Time(step) => crate::ops::time::modal::requires_alpha_label(step),
```

---

### `hp41-core/src/state.rs` (modify — new CalcState fields + migration)

**Self-analog.** All patterns from existing field declarations in `state.rs`.

**Persistent field pattern** (state.rs lines 92-94 — `#[serde(default)]` WITHOUT `skip`):
```rust
/// Clock time offset in seconds. SETIME computes delta = entered_seconds - current_seconds.
/// All time reads apply SystemTime::now() + Duration::from_secs(time_offset_secs.abs())
/// (adjusting sign). D-38.2.
#[serde(default)]
pub time_offset_secs: i64,

#[serde(default)]
pub clock_12h: bool,

#[serde(default)]
pub clock_display_mode: ClockDisplayMode,   // enum with Default = Off

#[serde(default)]
pub accuracy_factor: HpNum,

#[serde(default)]
pub alarms: Vec<AlarmEntry>,

#[serde(default)]
pub stopwatch_mode: StopwatchMode,          // enum with Default = Idle

#[serde(default)]
pub stopwatch_accumulated: f64,

#[serde(default)]
pub stopwatch_split: f64,
```

**Transient field pattern** (state.rs lines 109-110 — `#[serde(default, skip)]`):
```rust
#[serde(default, skip)]
pub stopwatch_start: Option<std::time::Instant>,   // Instant NOT serializable — P4

#[serde(default, skip)]
pub clock_active: bool,

#[serde(default, skip)]
pub stopwatch_keyboard_mode: bool,

#[serde(default, skip)]
pub alarm_catalog_mode: bool,
```

**default_xrom_modules function update** (state.rs lines 299-301 — change return value):
```rust
// v3.2: Time Module (bit 2) pre-loaded alongside Math 1 (bit 0) + Stat 1 (bit 1).
fn default_xrom_modules() -> u8 {
    0b0000_0111
}
```

**migrate_after_load extension** (state.rs lines 386-391 — append new arm after existing v3.0→v3.1 migration):
```rust
pub fn migrate_after_load(&mut self) {
    // v3.0 → v3.1: ensure STAT_1 bit (bit 1) is set.
    if self.xrom_modules & 0b0000_0010 == 0 {
        self.xrom_modules |= 0b0000_0010;
    }
    // v3.1 → v3.2: ensure TIME_MODULE bit (bit 2) is set.
    if self.xrom_modules & 0b0000_0100 == 0 {
        self.xrom_modules |= 0b0000_0100;
    }
    // v3.2: Running stopwatch cannot resume after load (Instant not serializable — D-38.6).
    // Freeze: transition Running → Stopped, preserve stopwatch_accumulated.
    if self.stopwatch_mode == StopwatchMode::Running {
        self.stopwatch_mode = StopwatchMode::Stopped;
        self.stopwatch_start = None;
    }
}
```

**CalcState::new() additions** (state.rs lines 336-351 — add Phase 38 comment block):
```rust
// Phase 38 (v3.2): Time Module fields
time_offset_secs: 0,
clock_12h: false,
clock_display_mode: ClockDisplayMode::default(),
accuracy_factor: HpNum::zero(),
alarms: Vec::new(),
stopwatch_mode: StopwatchMode::default(),
stopwatch_accumulated: 0.0,
stopwatch_split: 0.0,
stopwatch_start: None,
clock_active: false,
stopwatch_keyboard_mode: false,
alarm_catalog_mode: false,
```

---

### `hp41-core/src/ops/mod.rs` (modify — new Op variants)

**Self-analog.** Pattern: add ~33 new variants to the `Op` enum, mirroring the Phase 33 Stat 1 additions.

**Op variant naming convention** (from STAT_1.ops in xrom.rs): prefix with `Time` for disambiguation:
```rust
// ── Phase 38 (v3.2): Time Module (XROM 26) ─────────────────────────────────
TimeAdate,
TimeAlmcat,
TimeAlmnow,
TimeAtime,
TimeAtime24,
TimeClk12,
TimeClk24,
TimeClkt,
TimeClktd,
TimeClock,
TimeCorrect,
TimeDate,
TimeDatePlus,
TimeDdays,
TimeDmy,
TimeDow,
TimeMdy,
TimeRclaf,
TimeRclalm,
TimeRclsw,
TimeRunsw,
TimeSetaf,
TimeSetdate,
TimeSetime,
TimeSetsw,
TimeStopsw,
TimeSw,
TimeTplusx,
TimeTime,
TimeXyzalm,
TimeClalma,
TimeClalmx,
TimeClralms,
TimeSwpt,
TimeStpw,
```

---

### `hp41-core/src/ops/program.rs` (modify — execute_op routing)

**Self-analog.** Pattern: add ~33 new arms to `execute_op`, mirroring the Phase 33 Stat 1 additions.

**execute_op routing pattern** (program.rs lines 709-800 — each arm delegates to a module function):
```rust
// ── Phase 38 (v3.2): Time Module (XROM 26) ─────────────────────────────────
// Phase 38 sanctioned CI break — Phase 39 closes item 3, Phase 41 closes item 4.
Op::TimeTime   => crate::ops::time::clock::op_time(state),
Op::TimeDate   => crate::ops::time::clock::op_date(state),
Op::TimeSetime => crate::ops::time::clock::op_setime(state),
Op::TimeSetdate => crate::ops::time::clock::op_setdate(state),
Op::TimeTplusx => crate::ops::time::clock::op_tplusx(state),
Op::TimeSetaf  => crate::ops::time::clock::op_setaf(state),
Op::TimeRclaf  => crate::ops::time::clock::op_rclaf(state),
Op::TimeCorrect => crate::ops::time::clock::op_correct(state),
Op::TimeClk12  => crate::ops::time::clock::op_clk12(state),
Op::TimeClk24  => crate::ops::time::clock::op_clk24(state),
Op::TimeClkt   => crate::ops::time::clock::op_clkt(state),
Op::TimeClktd  => crate::ops::time::clock::op_clktd(state),
Op::TimeClock  => crate::ops::time::clock::op_clock(state),
Op::TimeDatePlus => crate::ops::time::date_arith::op_date_plus(state),
Op::TimeDdays  => crate::ops::time::date_arith::op_ddays(state),
Op::TimeDow    => crate::ops::time::date_arith::op_dow(state),
Op::TimeDmy    => crate::ops::time::date_arith::op_dmy(state),
Op::TimeMdy    => crate::ops::time::date_arith::op_mdy(state),
Op::TimeAdate  => crate::ops::time::alpha_time::op_adate(state),
Op::TimeAtime  => crate::ops::time::alpha_time::op_atime(state),
Op::TimeAtime24 => crate::ops::time::alpha_time::op_atime24(state),
Op::TimeRunsw  => crate::ops::time::stopwatch::op_runsw(state),
Op::TimeStopsw => crate::ops::time::stopwatch::op_stopsw(state),
Op::TimeSetsw  => crate::ops::time::stopwatch::op_setsw(state),
Op::TimeRclsw  => crate::ops::time::stopwatch::op_rclsw(state),
Op::TimeSwpt   => crate::ops::time::stopwatch::op_swpt(state),
Op::TimeStpw   => crate::ops::time::stopwatch::op_stpw(state),
Op::TimeSw     => crate::ops::time::stopwatch::op_sw(state),
Op::TimeXyzalm => crate::ops::time::alarm::op_xyzalm(state),
Op::TimeRclalm => crate::ops::time::alarm::op_rclalm(state),
Op::TimeAlmcat => crate::ops::time::alarm::op_almcat(state),
Op::TimeClalma => crate::ops::time::alarm::op_clalma(state),
Op::TimeClalmx => crate::ops::time::alarm::op_clalmx(state),
Op::TimeClralms => crate::ops::time::alarm::op_clralms(state),
Op::TimeAlmnow => crate::ops::time::alarm::op_almnow(state),
```

---

### `scripts/check-free42-contamination.sh` (modify)

**Self-analog.** Pattern: extend script following the Phase 33 Stat 1 addition (lines 23-38 and 57-63).

**Directory variable addition** (after `STAT1_DIR` at line 24):
```bash
TIME_DIR="hp41-core/src/ops/time"
```

**Directory existence loop extension** (lines 33-38 — extend `for dir in` loop):
```bash
for dir in "$MATH1_DIR" "$STAT1_DIR" "$TIME_DIR"; do
    if [[ ! -d "$dir" ]]; then
        echo "FAIL: $dir does not exist — license guard cannot run." >&2
        exit 2
    fi
done
```

**PATTERN extension** (line 55 — add Time Module Free42 identifiers; per RESEARCH.md Pattern 9):
```bash
PATTERN='phloat|Phloat|bid128_|decNumber|decContext|vartype|arg_struct|prgm_lines|bcd_t|Thomas Okken|AGPL|GNU General Public License|math_normal_|math_chi2_|math_t_dist_|math_F_dist_|math_gamma_|math_beta_inc|core_commands7|date2j|j2date'
```

**Scan loop extension** (lines 57-63 — extend `for dir in` loop):
```bash
for dir in "$MATH1_DIR" "$STAT1_DIR" "$TIME_DIR"; do
    if matches=$(grep -rn -E "$PATTERN" "$dir" | grep -v "$DISCLAIM_LINE"); then
        ...
    fi
done
```

---

## Shared Patterns

### Disclaim Header (ALL files in `hp41-core/src/ops/time/`)
**Source:** `hp41-core/src/ops/math1/xrom.rs` lines 1-2 and `hp41-core/src/ops/stat1/mod.rs` lines 1-2
**Apply to:** Every `.rs` file in `hp41-core/src/ops/time/`
```rust
// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
```

### Op Function Signature
**Source:** `hp41-core/src/ops/stat1/rand.rs` lines 67, 91 (general convention across all op modules)
**Apply to:** All `pub fn op_*` in `time/*.rs`
```rust
pub fn op_<name>(state: &mut CalcState) -> Result<(), HpError>
```

### Stack Push (unary result)
**Source:** `hp41-core/src/ops/stat1/rand.rs` lines 91-100 (enter_number + apply_lift_effect)
**Apply to:** All ops that push a result onto the X register
```rust
enter_number(state, result_hpnum);
apply_lift_effect(state, LiftEffect::Enable);
Ok(())
```

### Neutral Lift (no stack change)
**Source:** Throughout `stat1/*.rs` for ops like `DMY`/`MDY`/`CLRALMS` that modify state without touching the stack
```rust
apply_lift_effect(state, LiftEffect::Neutral);
Ok(())
```

### Modal Clear Before Dispatch
**Source:** `hp41-core/src/ops/stat1/modal.rs` lines 168-170
**Apply to:** All `time/modal.rs` `submit_step` arms
```rust
state.modal_program = None;
state.modal_prompt = None;
// then call op or return Ok(())
```

### Serde Annotation for New CalcState Fields
**Source:** `hp41-core/src/state.rs` lines 92-203 (the canonical annotation matrix)
**Apply to:** All Phase 38 additions to `CalcState`

| Field type | Annotation |
|---|---|
| Persistent primitive/struct | `#[serde(default)]` |
| Persistent enum (needs `Default` impl) | `#[serde(default)]` |
| Transient (`Option<Instant>`, `bool` flags) | `#[serde(default, skip)]` |

**NEVER add `#[serde(skip)]` to `time_offset_secs`** — the clock offset must survive save/load (same discipline as `rand_seed`).

### Error Handling in Op Functions
**Source:** `hp41-core/src/ops/stat1/rand.rs` lines 67-88 (general pattern)
**Apply to:** All Time Module op functions
```rust
// Use ? propagation for HpNum arithmetic errors (HpError::Overflow, etc.)
// Use HpError::Data for out-of-range date/time field values
// Use HpError::InvalidOp for parse failures and unsupported operations
// Use HpError::Domain for mathematically undefined inputs
// NEVER use .unwrap() in production code — #![deny(clippy::unwrap_used)]
```

### Test File Structure
**Source:** `hp41-core/src/ops/stat1/rand.rs` test module + `hp41-core/src/ops/math1/xrom.rs` test module
**Apply to:** All `time/*.rs` test modules
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::state::CalcState;

    // Each test catches ONE specific regression path (per STAT-QUAL test discipline)
    #[test]
    fn <descriptive_name>() {
        let mut state = CalcState::new();
        // ... arrange, act, assert pattern ...
    }
}
```

---

## No Analog Found

All Phase 38 files have close analogs. The three genuinely novel implementation aspects (not covered by existing code patterns) are:

| Novel aspect | File | Resolution |
|---|---|---|
| `std::time::SystemTime::now()` call in hp41-core | `time/clock.rs` | RESEARCH.md D-38.1 locked; use `SystemTime::now().duration_since(UNIX_EPOCH)` + `time_offset_secs` |
| `std::time::Instant` for monotonic stopwatch | `time/stopwatch.rs` | RESEARCH.md D-38.7 locked; `Option<Instant>` field is `#[serde(default, skip)]` — see Serde matrix |
| `libc::localtime_r` (Unix) / `GetLocalTime` (Windows) for local-time decomposition | `time/clock.rs` | RESEARCH.md D-38.3; transitive dep; use `#[cfg(unix)]` / `#[cfg(windows)]` |

For the JDN Fliegel-Van Flandern algorithm (`date_arith.rs`), use RESEARCH.md Pattern 5 directly — no codebase analog exists, but the algorithm is fully cited and locked.

---

## Metadata

**Analog search scope:** `hp41-core/src/ops/math1/`, `hp41-core/src/ops/stat1/`, `hp41-core/src/state.rs`, `hp41-core/src/ops/program.rs`, `scripts/`
**Files scanned:** 14
**Pattern extraction date:** 2026-05-24
