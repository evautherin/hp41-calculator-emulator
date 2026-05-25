// Algorithm independently re-derived from HP Time Module Owner's Manual (HP 82182A);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `time` — HP Time Module (XROM 26) operations.
//!
//! XROM module id: 26 (bit 2 of `CalcState::xrom_modules`).
//! Activated in Phase 38 (v3.2).
//!
//! ## Storage Registers — HP 82182A Time Module
//!
//! The HP Time Module (HP 82182A) does not maintain a contiguous user-register
//! data block in the way that Stat 1 Pac programs do. Instead, time-module state
//! is held in dedicated `CalcState` fields (D-38.2 / D-38.7 / D-38.10):
//!
//! - `time_offset_secs: i64` — Wall-clock adjustment: `SystemTime::now() +
//!   Duration::from_secs(|time_offset_secs|)` (with sign adjustment) gives
//!   the HP-41 clock time. SETIME computes the delta and stores it here.
//! - `clock_12h: bool` — True = 12-hour display; false = 24-hour.
//! - `clock_display_mode: ClockDisplayMode` — Off / TimeOnly / TimeAndDate.
//! - `accuracy_factor: HpNum` — Correction factor used by CORRECT.
//! - `alarms: Vec<AlarmEntry>` — Persistent alarm list; AlarmEntry carries
//!   trigger time, repeat interval, type (Message / Control), and past-due flag.
//! - `stopwatch_mode: StopwatchMode` — Idle / Running / Stopped.
//! - `stopwatch_accumulated: f64` — Elapsed seconds not in the current run.
//! - `stopwatch_split: f64` — Split-lap reference time (SWPT stores here).
//! - `stopwatch_start: Option<Instant>` — Transient; reset on load (D-38.6).
//! - `clock_active: bool` — Transient; true while CLKT/CLKTD is displaying.
//! - `stopwatch_keyboard_mode: bool` — Transient; true while SW is active.
//! - `alarm_catalog_mode: bool` — Transient; true while ALMCAT is browsing.
//!
//! All persistent fields carry `#[serde(default)]` on `CalcState` so v3.1
//! save files load cleanly (backward-compat invariant per D-38.2).
//! All transient fields carry `#[serde(default, skip)]` per the established
//! pattern (print_buffer / modal_program / etc.).

pub mod alarm;
pub mod alpha_time;
pub mod clock;
pub mod date_arith;
pub mod modal;
pub mod stopwatch;

// Re-export types needed by ops/mod.rs, ops/program.rs, and state.rs.
pub use alarm::{op_clalma, op_clalmx, op_clralms, op_rclalm, op_xyzalm, AlarmEntry, AlarmType};
pub use alpha_time::{op_adate, op_atime, op_atime24};
pub use clock::{
    op_clk12, op_clk24, op_clkt, op_clktd, op_clock, op_correct, op_date, op_rclaf, op_setaf,
    op_setdate, op_setime, op_time, op_tplusx, ClockDisplayMode,
};
pub use date_arith::{op_date_plus, op_ddays, op_dmy, op_dow, op_mdy};
pub use modal::TimeStep;
pub use stopwatch::{
    op_rclsw, op_runsw, op_setsw, op_stopsw, op_stpw, op_sw, op_swpt, secs_to_hpnum_time,
    StopwatchMode,
};
