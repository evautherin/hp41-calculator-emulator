// ADR-001: Numeric representation — rust_decimal 1.41 with HpNum newtype
//
// Decision date: Phase 1 (2026-05)
// Decision: Use `rust_decimal::Decimal` wrapped in `HpNum` newtype rather than:
//   (a) custom BCD nibble arithmetic, or
//   (b) f64 with manual rounding.
//
// Rationale:
//   - HP-41 hardware stores 10-digit BCD mantissa + 2-digit exponent (56-bit register).
//     Behavioral emulation does not require bit-identical storage — only identical outputs.
//   - `rust_decimal` is decimal-native (no binary float rounding artifacts like 0.1+0.2≠0.3).
//   - `round_sf_with_strategy(10, MidpointAwayFromZero)` enforces HP-41's 10-significant-digit
//     display precision with correct rounding direction (not Bankers rounding).
//   - A custom BCD struct would add ~500 LOC of nibble arithmetic with identical user-visible
//     behavior. The only scenario where this decision must be revisited is if Phase 7 QUAL-06
//     (≥98% numerical agreement, 500-case suite) reveals precision gaps that rust_decimal
//     cannot bridge — at which point a custom BCD struct replaces HpNum's inner type only.
//   - ISG/DSE counter fields (CCCCC.FFFDD) are extracted by string-splitting at the decimal
//     point regardless of representation — never via floor()/fmod() on f64.
//
// Consequences:
//   - All arithmetic in hp41-core flows through HpNum checked_* methods.
//   - Phase 2 adds `features = ["maths"]` to rust_decimal for ln/exp/pow.
//   - No f64 arithmetic on HP-41 register values anywhere in hp41-core.

use crate::num::{HpNum, HpValue};
use crate::ops::time::{alarm::AlarmEntry, ClockDisplayMode, StopwatchMode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Trigonometric angle mode — controls input/output units for SIN/COS/TAN/ASIN/ACOS/ATAN.
/// Default: Deg (HP-41 hardware cold-start default).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AngleMode {
    Deg,
    Rad,
    Grad,
}

/// Number display mode — controls how HpNum values are formatted for display.
/// u8 field = digit count (0–9).
/// Default: Fix(4) (HP-41 hardware cold-start default).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DisplayMode {
    Fix(u8),
    Sci(u8),
    Eng(u8),
}

/// The complete, mutable state of the HP-41 calculator.
///
/// All operations take `&mut CalcState`. No global mutable state anywhere.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalcState {
    pub stack: Stack,
    /// Storage registers R00–R99 (0-indexed). All zero on startup.
    pub regs: Vec<HpValue>,
    /// ALPHA register — up to 24 characters.
    pub alpha_reg: String,
    /// true = keyboard routes chars to alpha_reg instead of entry_buf.
    pub alpha_mode: bool,
    /// Trig angle mode (DEG/RAD/GRAD). Default: DEG.
    pub angle_mode: AngleMode,
    /// Number display mode (FIX/SCI/ENG n). Default: FIX 4.
    pub display_mode: DisplayMode,
    /// Pending digit string for number entry. Empty when not in digit-entry state.
    pub entry_buf: String,
    // ── Phase 3: Programming Engine ──────────────────────────────────────────
    /// Keystroke program storage. Flat list — Op::Lbl markers delimit subroutines.
    /// D-01: single contiguous Vec<Op> matching HP-41 flat program memory.
    pub program: Vec<crate::ops::Op>,
    /// PRGM mode: when true dispatch() records ops to program instead of executing.
    /// D-03: gate is checked at the top of dispatch().
    pub prgm_mode: bool,
    /// Program counter — index of the next op to execute in `program`.
    /// D-05: set by run_program(); 0 at startup.
    pub pc: usize,
    /// Subroutine return stack. Max 4 entries (HP-41 hardware limit, D-14).
    pub call_stack: Vec<usize>,
    /// True while run_program() is active; guards against re-entrancy.
    /// D-06: reset to false even on error path.
    pub is_running: bool,
    // ── Phase 5: USER mode & key assignments ─────────────────────────────────
    /// USER mode active: when true, key_assignments are consulted before normal dispatch.
    /// D-25: default false.
    pub user_mode: bool,
    /// User key assignments: maps key char → program label name.
    /// BTreeMap for deterministic JSON serialization order (D-25, D-29).
    pub key_assignments: BTreeMap<char, String>,
    /// HP-41 ASN key assignments: maps hardware key code (row×10+col, 1-indexed)
    /// → assigned target name. Phase 22 (FN-KEY-01). Coexists with key_assignments
    /// (Phase 5, char-keyed) — Phase 25/26 reconciles the two maps.
    /// `#[serde(default)]` keeps v1.0–v2.1 save files loadable (default → empty map).
    #[serde(default)]
    pub assignments: BTreeMap<u8, String>,
    /// HP-41 packed-text register shadows: ASTO writes a 6-char string here,
    /// ARCL reads from here in preference to formatting the numeric `regs[reg]`.
    /// Numeric STO / op_sto_arith / op_clreg CLEAR the matching entry to keep
    /// the two representations from drifting (D-23.4). `op_sto_arith_stack`
    /// targets the Y/Z/T/LastX stack registers (not numbered regs) so it does
    /// not touch this map.
    /// `#[serde(default)]` keeps v1.x–v2.1 save files loadable (default → empty map).
    /// Phase 23 (FN-ALPHA-01, FN-ALPHA-02).
    #[serde(default)]
    pub text_regs: BTreeMap<u8, String>,
    /// Buffer of formatted print lines from PRX/PRA/PRSTK.
    /// Drained by hp41-cli after each dispatch. Never persisted across sessions.
    /// #[serde(default, skip)] — default enables backward-compat deserialization of older
    /// save files that lack this field; skip prevents serialization of transient state.
    #[serde(default, skip)]
    pub print_buffer: Vec<String>,
    /// Last HP-41 row-column key code pressed (row×10+col, 1-indexed). 0 = none since startup.
    /// Updated by hp41-cli `handle_key()` on every Press event. Read by `Op::GetKey`.
    /// Default: 0. Persistent across save/load (#[serde(default)]).
    #[serde(default)]
    pub last_key_code: u8,

    /// Hidden register M — accessible via STO M / RCL M in programs.
    /// Not part of the numbered `regs: Vec<HpNum>`. Default: HpNum::zero().
    #[serde(default)]
    pub reg_m: HpNum,

    /// Hidden register N — accessible via STO N / RCL N in programs.
    #[serde(default)]
    pub reg_n: HpNum,

    /// Hidden register O — accessible via STO O / RCL O in programs.
    #[serde(default)]
    pub reg_o: HpNum,

    // ── Phase 21: Flags ──────────────────────────────────────────────────────
    /// HP-41 flags (user flags 0-29 + system flags 30-55) packed into a single u64.
    /// Bit n = flag n. Default: 0 (all clear). Use `ops::flags` helpers for safe access.
    /// `#[serde(default)]` — a v2.0 autosave.json without this field loads cleanly with flags == 0.
    /// Phase 21 (FN-FLAG-01).
    #[serde(default)]
    pub flags: u64,

    // ── Phase 21: Display Control ────────────────────────────────────────────
    /// HP-41 display override channel: VIEW/AVIEW/PROMPT/CLD write to this.
    /// None = render normal display. Transient — cleared at the top of dispatch
    /// and never persisted (`#[serde(default, skip)]`). Phase 21 (FN-DISP-01..05).
    #[serde(default, skip)]
    pub display_override: Option<String>,

    // ── Phase 21: Sound ──────────────────────────────────────────────────────
    /// HP-41 sound event buffer: BEEP and TONE n push structured event lines here.
    /// Drained by hp41-cli / hp41-gui after each dispatch — frontend plays audio.
    /// Transient — never persisted (`#[serde(default, skip)]`).
    /// Phase 21 (FN-SOUND-01 / FN-SOUND-02).
    #[serde(default, skip)]
    pub event_buffer: Vec<String>,
    /// Pending card I/O request set by `Op::Wdta`/`Op::Rdta`/`Op::Wprgm`/`Op::Rdprgm`.
    /// The frontend (hp41-cli / hp41-gui) drains this after each `dispatch()` and
    /// performs the actual disk I/O — keeps hp41-core UI-agnostic. Mirrors the
    /// `print_buffer` drain pattern.
    #[serde(default, skip)]
    pub pending_card_op: Option<crate::cardreader::CardOpRequest>,

    // ── Phase 28 (v3.0) + Phase 33 (v3.1): XROM framework ───────────────────
    /// Bitfield of loaded XROM modules. Bit 0 = Math 1, bit 1 = Stat 1.
    /// Default: 0b0000_0011 (Math 1 + Stat 1 pre-loaded per v3.1 scope, D-33.2).
    /// Persistent across save/load. `#[serde(default = "default_xrom_modules")]`.
    /// v3.0 save files (bit 1 clear) are upgraded by `migrate_after_load()`.
    #[serde(default = "default_xrom_modules")]
    pub xrom_modules: u8,

    /// Complex stack overlay mode (D-28.1 / D-28.2). When true, X+iY form
    /// the complex number ζ and Z+iT form τ. Auto-on at first complex op;
    /// explicit `XEQ "REAL"` (D-28.3) deactivates. Safe default: false.
    #[serde(default)]
    pub complex_mode: bool,

    // ── Phase 33 (v3.1): Stat 1 Pac RNG seed ────────────────────────────────
    /// HP-41 Stat 1 Pac LCG random-number seed (community-confirmed
    /// emulator extension per D-33.4).
    ///
    /// LCG formula: `r_{n+1} = FRC(9821 · r_n + 0.211327)` (NPS p. 21,
    /// HP-65 User's Library via Don Malm, HP-41C Standard Applications p. 24).
    /// Default initial value: HpNum::zero(); SEED writes via XROM Op::Seed.
    ///
    /// ⚠️ UNIQUE SERDE SHAPE — the SOLE new v3.1 `CalcState` field that
    /// carries `#[serde(default)]` WITHOUT `#[serde(skip)]`. The seed MUST
    /// survive save/load so program-driven SEED commands persist across
    /// sessions and RAND determinism is preserved (STAT-RNG-03).
    ///
    /// Adding `#[serde(skip)]` here would silently break determinism: a
    /// program that calls SEED 0.5 then RAND once would, on the next session
    /// load, observe `rand_seed = 0` instead of the post-RAND value, producing
    /// a different RAND output (P20 trap — silent drift, no error).
    ///
    /// Mirror pattern: `complex_mode: bool` at line above is the IDENTICAL
    /// `#[serde(default)]` (no `skip`) shape — the ONLY existing precedent
    /// in this struct. Every OTHER `#[serde(default)]` field in `CalcState`
    /// (print_buffer / modal_program / modal_prompt / integ_state /
    ///  solve_state / difeq_state / cancel_requested) ALSO carries
    /// `#[serde(skip)]` — DO NOT mimic those for this field.
    ///
    /// Code review block-the-PR rule: any future edit adding `skip` to this
    /// annotation is a STAT-RNG-03 contract violation. The
    /// `rand_seed_serde_round_trip` test in `mod tests` is the CI guard.
    #[serde(default)]
    pub rand_seed: HpNum,

    /// Current matrix dimension (rows, cols) for MATRIX workflow (Plan 28-06).
    /// None = no matrix active. Persistent (matrix shape survives save/load).
    #[serde(default)]
    pub matrix_dim: Option<(u8, u8)>,

    /// Active matrix register index (for MATRIX element-edit mode, Plan 28-06).
    /// None = not editing a matrix register. Persistent.
    #[serde(default)]
    pub matrix_active_reg: Option<u8>,

    /// Active modal program (MATRIX/SOLVE/POLY/INTG/DIFEQ/FOUR/TRANS).
    /// Transient — set on modal-open, cleared on completion or cancel.
    /// Never persisted (`#[serde(default, skip)]`).
    #[serde(default, skip)]
    pub modal_program: Option<crate::ops::math1::modal::ModalProgram>,

    /// Modal prompt text for active workflow step.
    /// CLI renders in `pending_prompt()` (Phase 29 wiring).
    /// GUI renders as overlay banner above LCD (Phase 31 wiring).
    /// R/S key submits numeric input per D-28.5 (CLI/GUI wiring in Phase 29/31).
    /// Transient — never persisted (`#[serde(default, skip)]`).
    #[serde(default, skip)]
    pub modal_prompt: Option<String>,

    /// Mid-iteration state for INTG numerical integration (Plan 28-07).
    /// Transient — never persisted (`#[serde(default, skip)]`).
    /// Placeholder stub; Plan 28-07 fills fields.
    #[serde(default, skip)]
    pub integ_state: Option<crate::ops::math1::integ::IntegState>,

    /// Mid-iteration state for SOLVE root-finding (Plan 28-08).
    /// Transient — never persisted (`#[serde(default, skip)]`).
    /// Placeholder stub; Plan 28-08 fills fields.
    #[serde(default, skip)]
    pub solve_state: Option<crate::ops::math1::solve::SolveState>,

    /// Mid-iteration state for DIFEQ ODE solver (Plan 28-09).
    /// Transient — never persisted (`#[serde(default, skip)]`).
    /// RESEARCH Open Q2 recommendation (a): early commitment.
    /// Placeholder stub; Plan 28-09 fills fields.
    #[serde(default, skip)]
    pub difeq_state: Option<crate::ops::math1::difeq::DifeqState>,

    /// Cancellation flag for long-running solvers (INTG/SOLVE/DIFEQ).
    /// `Arc<AtomicBool>` so Phase 31 `request_cancel` Tauri command can set it
    /// from the GUI thread without locking the `AppState` Mutex (D-28.7).
    /// Per-64-samples check: `cancel_requested.load(Relaxed)` inside solver loops
    /// (D-28.8). Reset to `false` at every op_integ/op_solve/op_difeq entry.
    /// Transient — never persisted (`#[serde(default = "default_cancel_requested", skip)]`).
    #[serde(default = "default_cancel_requested", skip)]
    pub cancel_requested: std::sync::Arc<std::sync::atomic::AtomicBool>,

    /// Transient ν carrier for the ΣCHISQD two-step modal prompt
    /// sequence (REVIEW.md WR-03 / WR-04 mitigation).
    ///
    /// Plan 33-03 originally stashed ν in `state.stack.t` between the
    /// ν-prompt submit and the mode-choice submit (D-33.5 — "no new
    /// transient `CalcState` field"). That design was unsafe: any
    /// stack-lifting Op invoked between the two submits (most
    /// arithmetic, push-lifts from backspace edits, XEQ calls)
    /// silently clobbered T, leaving the mode-choice submit reading
    /// garbage ν data. The user's original T value was also
    /// destructively overwritten with no recovery path.
    ///
    /// This field is the conservative remedy: a transient
    /// `Option<u32>` carrier set by `submit_step(ChisqdNuPrompt)` and
    /// read + cleared by `submit_step(ChisqdModeChoice)`. The stack
    /// is no longer used as a side-channel — `submit_step(ChisqdNu
    /// Prompt)` now performs the standard 4-slot HP-41 stack drop
    /// (`x ← y, y ← z, z ← t, t ← t`) preserving the user's original
    /// T value.
    ///
    /// Transient — never persisted (`#[serde(default, skip)]`).
    /// Cleared on every `op_sigma_chisqd_workflow` interactive open
    /// AND on every `submit_step(ChisqdModeChoice)` exit (success or
    /// error) so a stale carrier never leaks into a subsequent
    /// ΣCHISQD cycle.
    ///
    /// The "no new persistent field" constraint from D-33.5 banned
    /// PERSISTENT additions; transient `#[serde(default, skip)]`
    /// fields are the existing pattern (see `modal_program`,
    /// `modal_prompt`, `integ_state`, etc.) and were always permitted.
    #[serde(default, skip)]
    pub pending_chisqd_nu: Option<u32>,

    // ── Phase 38 (v3.2): Time Module (XROM 26) ──────────────────────────────

    /// Wall-clock offset in seconds (D-38.2).
    /// SETIME computes `delta = entered_unix_secs - SystemTime::now()` and stores here.
    /// Default: 0 (system time unmodified). Persistent — `#[serde(default)]`.
    #[serde(default)]
    pub time_offset_secs: i64,

    /// 12-hour clock display mode flag (D-38.2).
    /// true = 12-hour format (with AM/PM); false = 24-hour format.
    /// Default: false (24-hour). Persistent — `#[serde(default)]`.
    #[serde(default)]
    pub clock_12h: bool,

    /// Continuous clock display mode (D-38.2).
    /// Off = no display; TimeOnly = CLKT; TimeAndDate = CLKTD.
    /// Default: `ClockDisplayMode::Off`. Persistent — `#[serde(default)]`.
    #[serde(default)]
    pub clock_display_mode: ClockDisplayMode,

    /// Clock accuracy correction factor (D-38.2).
    /// Written by CORRECT from stack X. Default: HpNum::zero().
    /// Persistent — `#[serde(default)]`.
    #[serde(default)]
    pub accuracy_factor: HpNum,

    /// Alarm list (D-38.8 / D-38.10).
    /// Default: empty Vec (no alarms). Persistent — `#[serde(default)]`.
    #[serde(default)]
    pub alarms: Vec<AlarmEntry>,

    /// Stopwatch operating mode (D-38.7).
    /// Default: `StopwatchMode::Idle`. Persistent — `#[serde(default)]`.
    /// D-38.6: `migrate_after_load()` transitions Running → Stopped on load.
    #[serde(default)]
    pub stopwatch_mode: StopwatchMode,

    /// Accumulated stopwatch time in seconds not in the current run (D-38.7).
    /// Default: 0.0. Persistent — `#[serde(default)]`.
    #[serde(default)]
    pub stopwatch_accumulated: f64,

    /// Split/lap reference time in seconds (D-38.7).
    /// Written by SWPT. Default: 0.0. Persistent — `#[serde(default)]`.
    #[serde(default)]
    pub stopwatch_split: f64,

    /// Transient: start instant of the current stopwatch run (D-38.6).
    /// `None` when mode ≠ Running. Transient — `#[serde(default, skip)]`.
    #[serde(default, skip)]
    pub stopwatch_start: Option<std::time::Instant>,

    /// Transient: true while CLKT/CLKTD continuous clock display is active (D-38.2).
    /// Transient — `#[serde(default, skip)]`.
    #[serde(default, skip)]
    pub clock_active: bool,

    /// Transient: true while the SW stopwatch keyboard mode is active (D-38.7).
    /// Transient — `#[serde(default, skip)]`.
    #[serde(default, skip)]
    pub stopwatch_keyboard_mode: bool,

    /// Transient: true while ALMCAT alarm catalog browsing is active (D-38.10).
    /// Transient — `#[serde(default, skip)]`.
    #[serde(default, skip)]
    pub alarm_catalog_mode: bool,
}

// ── serde-default helpers ────────────────────────────────────────────────────

/// Default value for `xrom_modules`: bit 0 = Math 1, bit 1 = Stat 1, bit 2 = Time Module,
/// all pre-loaded per v3.2 scope (D-carried.5).
///
/// v3.0 shipped with `0b0000_0001` (Math 1 only); v3.1 flipped bit 1 on (D-33.2).
/// v3.2 flips bit 2 on because Time Module (XROM 26) is part of this milestone
/// (TIME-FW-01). Migration of v3.1 save files lacking bit 2 happens in
/// `CalcState::migrate_after_load()`.
fn default_xrom_modules() -> u8 {
    0b0000_0111
}

/// Default value for `cancel_requested`: a new Arc<AtomicBool> initialized to false.
fn default_cancel_requested() -> std::sync::Arc<std::sync::atomic::AtomicBool> {
    std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false))
}

impl CalcState {
    pub fn new() -> Self {
        CalcState {
            stack: Stack::new(),
            regs: vec![HpValue::default(); 100],
            alpha_reg: String::new(),
            alpha_mode: false,
            angle_mode: AngleMode::Deg,
            display_mode: DisplayMode::Fix(4),
            entry_buf: String::new(),
            program: Vec::new(),
            prgm_mode: false,
            pc: 0,
            call_stack: Vec::new(),
            is_running: false,
            user_mode: false,
            key_assignments: BTreeMap::new(),
            assignments: BTreeMap::new(),
            text_regs: BTreeMap::new(),
            print_buffer: Vec::new(),
            last_key_code: 0,
            reg_m: HpNum::zero(),
            reg_n: HpNum::zero(),
            reg_o: HpNum::zero(),
            flags: 0,
            display_override: None,
            event_buffer: Vec::new(),
            pending_card_op: None,
            // Phase 28 (v3.0) fields
            xrom_modules: default_xrom_modules(),
            complex_mode: false,
            // Phase 33 (v3.1): Stat 1 Pac RNG seed (D-33.4 emulator extension)
            rand_seed: HpNum::zero(),
            matrix_dim: None,
            matrix_active_reg: None,
            modal_program: None,
            modal_prompt: None,
            integ_state: None,
            solve_state: None,
            difeq_state: None,
            cancel_requested: default_cancel_requested(),
            // Phase 33 (v3.1) review-fix: transient ΣCHISQD ν carrier
            // (REVIEW.md WR-03/WR-04 — replaces the stack-T side channel).
            pending_chisqd_nu: None,
            // Phase 38 (v3.2): Time Module (XROM 26) fields
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
        }
    }
}

impl Default for CalcState {
    fn default() -> Self {
        Self::new()
    }
}

// ── Phase 33 (v3.1): post-deserialization migration helpers ─────────────────

impl CalcState {
    /// Apply post-deserialization migrations to a freshly-loaded `CalcState`.
    ///
    /// Called once after every `serde_json::from_str::<CalcState>(...)` by the
    /// two persistence wiring sites (single source of truth per D-33.7):
    ///
    /// - `hp41-cli/src/persistence.rs::load_state` — Phase 34 wiring
    /// - `hp41-gui/src-tauri/src/persistence.rs` — Phase 36 wiring
    ///
    /// Idempotent — safe to call multiple times on already-migrated state
    /// (the bitwise OR is a no-op when bit 1 is already set).
    ///
    /// ## v3.0 → v3.1 migration: Stat 1 XROM bit
    ///
    /// v3.0 save files persist `"xrom_modules": 1` (bit 0 = Math 1 only).
    /// Without this migration, v3.0 users opening a v3.1 binary would see
    /// Stat 1 silently disabled — `XEQ "ΣNORMD"` would return InvalidOp.
    /// The migration unconditionally sets bit 1 so Stat 1 is loaded for
    /// every save file (P24 trap mitigation).
    ///
    /// Re-save happens on the next 30 s auto-save tick or exit-save; no
    /// explicit re-save call from this method (D-33.7).
    pub fn migrate_after_load(&mut self) {
        // v3.0 → v3.1: ensure STAT_1 bit (bit 1) is set.
        if self.xrom_modules & 0b0000_0010 == 0 {
            self.xrom_modules |= 0b0000_0010;
        }
        // v3.1 → v3.2: ensure TIME_MODULE bit (bit 2) is set.
        // v3.1 save files persist `"xrom_modules": 3` (bits 0+1 = Math 1 + Stat 1).
        // Without this migration, v3.1 users opening a v3.2 binary would see
        // Time Module silently disabled — `XEQ "TIME"` would return InvalidOp.
        if self.xrom_modules & 0b0000_0100 == 0 {
            self.xrom_modules |= 0b0000_0100;
        }
        // D-38.6: freeze a Running stopwatch on load.
        // `stopwatch_start: Option<Instant>` is `#[serde(skip)]` and thus always
        // `None` after deserialization. A Running stopwatch with no Instant is
        // indeterminate — freeze to Stopped so RCLSW/STOPSW sees a stable state.
        if self.stopwatch_mode == StopwatchMode::Running {
            self.stopwatch_mode = StopwatchMode::Stopped;
            self.stopwatch_start = None;
        }
    }
}

/// The HP-41 4-level RPN stack with LASTX and stack-lift flag.
///
/// Registers: X (visible), Y, Z, T (bottom). T is dropped (overwritten by Z) on lift;
/// T is duplicated (not consumed) on stack drop (binary result).
/// lift_enabled: true means the next number entry will lift the stack before writing X.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack {
    /// X register — the currently visible value
    pub x: HpNum,
    /// Y register
    pub y: HpNum,
    /// Z register
    pub z: HpNum,
    /// T register — dropped (overwritten by Z) on lift; duplicated (not consumed) on stack drop
    pub t: HpNum,
    /// LASTX — captures X before it is consumed by a binary operation
    pub lastx: HpNum,
    /// Stack-lift flag: true = next number entry lifts; false = overwrites X
    pub lift_enabled: bool,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            x: HpNum::zero(),
            y: HpNum::zero(),
            z: HpNum::zero(),
            t: HpNum::zero(),
            lastx: HpNum::zero(),
            lift_enabled: false,
        }
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    // Catches: default initializer regression for Phase 28 + Phase 33 + Phase 38 fields.
    // Phase 33 (v3.1) flipped the default from `0b0000_0001` to `0b0000_0011`
    // (D-33.2 / STAT-FW-02). Phase 38 (v3.2) flips bit 2 on (TIME-FW-01):
    // Math 1 (bit 0) + Stat 1 (bit 1) + Time Module (bit 2) all pre-loaded.
    #[test]
    fn default_construction_phase28_fields() {
        let state = CalcState::default();
        assert_eq!(
            state.xrom_modules, 0b0000_0111,
            "Math 1 + Stat 1 + Time Module must be pre-loaded by default (v3.2 scope)"
        );
        assert!(!state.complex_mode, "complex_mode must default to false");
        assert_eq!(state.matrix_dim, None, "matrix_dim must default to None");
        assert_eq!(
            state.matrix_active_reg, None,
            "matrix_active_reg must default to None"
        );
        assert!(
            state.modal_program.is_none(),
            "modal_program must default to None"
        );
        assert!(
            state.modal_prompt.is_none(),
            "modal_prompt must default to None"
        );
        assert!(
            state.integ_state.is_none(),
            "integ_state must default to None"
        );
        assert!(
            state.solve_state.is_none(),
            "solve_state must default to None"
        );
        assert!(
            state.difeq_state.is_none(),
            "difeq_state must default to None"
        );
        assert!(
            !state.cancel_requested.load(Ordering::Relaxed),
            "cancel_requested must default to false"
        );
    }

    // Catches: serde(skip) on transient fields — they must NOT appear in serialized output;
    //          serde(default) on persistent fields — they must survive round-trip.
    #[test]
    fn serde_roundtrip() {
        let mut state = CalcState::new();
        // Set some transient fields to non-default values
        state.modal_prompt = Some("ORDER=?".to_string());
        state.integ_state = Some(crate::ops::math1::integ::IntegState::default());
        // Set persistent fields
        state.xrom_modules = 0b0000_0011; // Math 1 + hypothetical module 2
        state.complex_mode = true;
        state.matrix_dim = Some((3, 3));
        state.matrix_active_reg = Some(5);

        let json = serde_json::to_string(&state).unwrap();

        // Transient fields must NOT appear in serialized output
        assert!(
            !json.contains("modal_prompt"),
            "modal_prompt must be serde(skip)"
        );
        assert!(
            !json.contains("integ_state"),
            "integ_state must be serde(skip)"
        );
        assert!(
            !json.contains("cancel_requested"),
            "cancel_requested must be serde(skip)"
        );

        // Persistent fields must appear in serialized output
        assert!(
            json.contains("xrom_modules"),
            "xrom_modules must be serialized"
        );
        assert!(
            json.contains("complex_mode"),
            "complex_mode must be serialized"
        );
        assert!(json.contains("matrix_dim"), "matrix_dim must be serialized");

        let restored: CalcState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.xrom_modules, 0b0000_0011);
        assert!(restored.complex_mode);
        assert_eq!(restored.matrix_dim, Some((3, 3)));
        assert_eq!(restored.matrix_active_reg, Some(5));

        // Transient fields reset to defaults after round-trip
        assert!(
            restored.modal_prompt.is_none(),
            "modal_prompt must reset to None after deserialization"
        );
        assert!(
            restored.integ_state.is_none(),
            "integ_state must reset to None after deserialization"
        );
        assert!(
            !restored.cancel_requested.load(Ordering::Relaxed),
            "cancel_requested must reset to false after deserialization"
        );
    }

    // Catches: v2.2 save-file backward-compat regression (Pitfall 12 mitigation)
    #[test]
    fn v22_save_loads_with_defaults() {
        // A minimal v2.2-shape JSON without any v3.0 fields
        let v22_json = r#"{
            "stack": {"x": "0", "y": "0", "z": "0", "t": "0", "lastx": "0", "lift_enabled": false},
            "regs": ["0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0"],
            "alpha_reg": "",
            "alpha_mode": false,
            "angle_mode": "Deg",
            "display_mode": {"Fix": 4},
            "entry_buf": "",
            "program": [],
            "prgm_mode": false,
            "pc": 0,
            "call_stack": [],
            "is_running": false,
            "user_mode": false,
            "key_assignments": {},
            "assignments": {},
            "text_regs": {},
            "last_key_code": 0,
            "reg_m": "0",
            "reg_n": "0",
            "reg_o": "0",
            "flags": 0,
            "pending_card_op": null
        }"#;

        let state: CalcState = serde_json::from_str(v22_json).unwrap();

        // Phase 28 + Phase 33 + Phase 38 fields must default cleanly.
        // Note: v3.2 flipped default_xrom_modules() to 0b0000_0111 (TIME-FW-01);
        // a v2.2 save lacking the field therefore loads with all three bits set.
        assert_eq!(
            state.xrom_modules, 0b0000_0111,
            "v2.2 save must get v3.2 default xrom_modules = 0b0000_0111"
        );
        assert!(
            !state.complex_mode,
            "v2.2 save must get default complex_mode"
        );
        assert_eq!(
            state.matrix_dim, None,
            "v2.2 save must get default matrix_dim"
        );
        assert_eq!(
            state.matrix_active_reg, None,
            "v2.2 save must get default matrix_active_reg"
        );
    }

    // Catches: cancel_requested not being a real Arc (copy instead of shared reference)
    #[test]
    fn cancel_field_present() {
        let state = CalcState::new();
        let cloned_arc = std::sync::Arc::clone(&state.cancel_requested);
        // Set via the clone — must be visible through the original
        cloned_arc.store(true, Ordering::Relaxed);
        assert!(
            state.cancel_requested.load(Ordering::Relaxed),
            "cancel_requested must be a real Arc<AtomicBool> (not a copy)"
        );
    }

    // ── Phase 33 (v3.1): default flip + migration + rand_seed serde ─────────

    // Catches: default_xrom_modules() regression — must be 0b0000_0111 after Phase 38 (TIME-FW-01).
    #[test]
    fn xrom_modules_default_is_seven() {
        let state = CalcState::new();
        assert_eq!(
            state.xrom_modules, 0b0000_0111,
            "default_xrom_modules() must return 0b0000_0111 in v3.2 (Math 1 + Stat 1 + Time pre-loaded per TIME-FW-01)"
        );
    }

    // Catches: P24 trap — v3.0 save files load with bit 1 clear; migrate_after_load
    // must unconditionally upgrade them so Stat 1 functionality is reachable.
    #[test]
    fn v3_0_save_loads_with_stat_1_after_migration() {
        // Synthetic v3.0 save: contains xrom_modules: 1 (the v3.0 default) plus
        // all other v3.0 fields. Mirrors the v22 blob shape from
        // `loads_synthetic_v22_save_without_v3_fields` extended with v3.0 keys.
        let v30_json = r#"{
            "stack": {"x": "0", "y": "0", "z": "0", "t": "0", "lastx": "0", "lift_enabled": false},
            "regs": ["0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0"],
            "alpha_reg": "",
            "alpha_mode": false,
            "angle_mode": "Deg",
            "display_mode": {"Fix": 4},
            "entry_buf": "",
            "program": [],
            "prgm_mode": false,
            "pc": 0,
            "call_stack": [],
            "is_running": false,
            "user_mode": false,
            "key_assignments": {},
            "assignments": {},
            "text_regs": {},
            "last_key_code": 0,
            "reg_m": "0",
            "reg_n": "0",
            "reg_o": "0",
            "flags": 0,
            "pending_card_op": null,
            "xrom_modules": 1,
            "complex_mode": false,
            "matrix_dim": null,
            "matrix_active_reg": null
        }"#;

        let mut state: CalcState = serde_json::from_str(v30_json).unwrap();
        // Pre-migration: a v3.0 save preserved its bit-0-only value.
        assert_eq!(
            state.xrom_modules, 0b0000_0001,
            "v3.0 save with explicit xrom_modules:1 must deserialize as 0b0000_0001 BEFORE migration"
        );

        state.migrate_after_load();

        // Post-migration: bits 1+2 are now set (Stat 1 + Time Module reachable).
        assert_eq!(
            state.xrom_modules, 0b0000_0111,
            "migrate_after_load() must set bits 1+2 on a v3.0 save (P24 trap + TIME-FW-02)"
        );
    }

    // Catches: migrate_after_load not being idempotent — repeated calls must
    // not corrupt already-migrated state (CLI/GUI both call it on every load,
    // and a re-saved v3.2 file gets migrated again on the next session).
    #[test]
    fn migrate_after_load_idempotent() {
        let mut state = CalcState::new();
        assert_eq!(
            state.xrom_modules, 0b0000_0111,
            "fresh CalcState::new() already has bits 0+1+2 set (v3.2 default)"
        );

        state.migrate_after_load();
        assert_eq!(
            state.xrom_modules, 0b0000_0111,
            "first migrate_after_load() on already-migrated state must be a no-op"
        );

        state.migrate_after_load();
        assert_eq!(
            state.xrom_modules, 0b0000_0111,
            "repeated migrate_after_load() must remain idempotent (no bit drift)"
        );
    }

    // Catches: P20 trap — `rand_seed` carries the UNIQUE serde shape
    // (`#[serde(default)]` WITHOUT `skip`). If anyone accidentally adds
    // `skip`, this round-trip test fails and the PR is blocked. The test
    // proves the seed survives serialize → deserialize unchanged so
    // STAT-RNG-03 determinism holds across save/load.
    #[test]
    fn rand_seed_serde_round_trip() {
        use rust_decimal::Decimal;
        use std::str::FromStr;

        let mut state = CalcState::new();
        let seed_value = HpNum::from(Decimal::from_str("0.7").unwrap());
        state.rand_seed = seed_value.clone();

        let json = serde_json::to_string(&state).unwrap();
        // The field MUST appear in the serialized output (proves NOT skipped).
        assert!(
            json.contains("rand_seed"),
            "rand_seed must be serialized (no #[serde(skip)] — STAT-RNG-03 contract)"
        );

        let restored: CalcState = serde_json::from_str(&json).unwrap();
        assert_eq!(
            restored.rand_seed, seed_value,
            "rand_seed must round-trip through serde unchanged (P20 trap mitigation)"
        );
    }

    // ── Phase 38 (v3.2): Time Module CalcState serde + migration tests ────────

    // Catches: persistent Time fields not surviving serde round-trip; transient
    // fields (stopwatch_start, clock_active, etc.) leaking into/out of JSON.
    #[test]
    fn time_fields_serde_round_trip() {
        use crate::ops::time::{ClockDisplayMode, StopwatchMode, alarm::{AlarmEntry, AlarmType}};
        use rust_decimal::Decimal;
        use std::str::FromStr;

        let mut state = CalcState::new();

        // Set persistent Time fields.
        state.time_offset_secs = 3600;
        state.clock_12h = true;
        state.clock_display_mode = ClockDisplayMode::TimeAndDate;
        state.accuracy_factor = HpNum::from(Decimal::from_str("1.5").unwrap());
        state.alarms.push(AlarmEntry {
            trigger_unix: 1_700_000_000,
            repeat_secs: 86400,
            alarm_type: AlarmType::Message("Test alarm".to_string()),
            past_due: false,
        });
        state.stopwatch_mode = StopwatchMode::Stopped;
        state.stopwatch_accumulated = 123.45;
        state.stopwatch_split = 10.5;

        // Set transient fields (should NOT survive deserialization).
        state.clock_active = true;
        state.stopwatch_keyboard_mode = true;
        state.alarm_catalog_mode = true;
        state.stopwatch_start = Some(std::time::Instant::now());

        let json = serde_json::to_string(&state).unwrap();

        // Persistent fields must appear in JSON.
        assert!(json.contains("time_offset_secs"), "time_offset_secs must be serialized");
        assert!(json.contains("clock_12h"), "clock_12h must be serialized");
        assert!(json.contains("stopwatch_accumulated"), "stopwatch_accumulated must be serialized");

        let restored: CalcState = serde_json::from_str(&json).unwrap();

        // Persistent fields must survive round-trip.
        assert_eq!(restored.time_offset_secs, 3600);
        assert!(restored.clock_12h);
        assert_eq!(restored.clock_display_mode, ClockDisplayMode::TimeAndDate);
        assert_eq!(restored.stopwatch_mode, StopwatchMode::Stopped);
        assert!((restored.stopwatch_accumulated - 123.45).abs() < 1e-9);
        assert!((restored.stopwatch_split - 10.5).abs() < 1e-9);
        assert_eq!(restored.alarms.len(), 1);

        // Transient fields must reset to defaults after deserialization.
        assert!(!restored.clock_active, "clock_active is transient — must reset to false");
        assert!(!restored.stopwatch_keyboard_mode, "stopwatch_keyboard_mode is transient — must reset to false");
        assert!(!restored.alarm_catalog_mode, "alarm_catalog_mode is transient — must reset to false");
        assert!(restored.stopwatch_start.is_none(), "stopwatch_start is transient — must reset to None");
    }

    // Catches: v3.1 save files (xrom_modules=3) not being migrated to 7 on load.
    #[test]
    fn time_v31_save_migration() {
        // Synthetic v3.1 save: xrom_modules=3 (Math 1 + Stat 1, no Time), no time fields.
        let v31_json = r#"{
            "stack": {"x": "0", "y": "0", "z": "0", "t": "0", "lastx": "0", "lift_enabled": false},
            "regs": ["0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0",
                     "0","0","0","0","0","0","0","0","0","0"],
            "alpha_reg": "",
            "alpha_mode": false,
            "angle_mode": "Deg",
            "display_mode": {"Fix": 4},
            "entry_buf": "",
            "program": [],
            "prgm_mode": false,
            "pc": 0,
            "call_stack": [],
            "is_running": false,
            "user_mode": false,
            "key_assignments": {},
            "assignments": {},
            "text_regs": {},
            "last_key_code": 0,
            "reg_m": "0",
            "reg_n": "0",
            "reg_o": "0",
            "flags": 0,
            "pending_card_op": null,
            "xrom_modules": 3,
            "complex_mode": false,
            "matrix_dim": null,
            "matrix_active_reg": null,
            "rand_seed": "0"
        }"#;

        let mut state: CalcState = serde_json::from_str(v31_json).unwrap();

        // Before migration: v3.1 value preserved exactly.
        assert_eq!(
            state.xrom_modules, 3,
            "v3.1 save with xrom_modules:3 must deserialize as 3 BEFORE migration"
        );

        // Time fields should be at serde defaults (no time fields in the JSON).
        assert_eq!(state.time_offset_secs, 0, "time_offset_secs must default to 0");
        assert!(!state.clock_12h, "clock_12h must default to false");
        assert_eq!(
            state.stopwatch_mode,
            crate::ops::time::StopwatchMode::Idle,
            "stopwatch_mode must default to Idle"
        );

        state.migrate_after_load();

        // After migration: bit 2 must be set (Time Module loaded).
        assert_eq!(
            state.xrom_modules, 0b0000_0111,
            "migrate_after_load() must set bit 2 on a v3.1 save (TIME-FW-02)"
        );
    }

    // Catches: D-38.6 — Running stopwatch must freeze to Stopped after load
    // (stopwatch_start is #[serde(skip)] and thus None after deserialization).
    #[test]
    fn time_stopwatch_running_freeze_on_load() {
        use crate::ops::time::StopwatchMode;

        let mut state = CalcState::new();
        state.stopwatch_mode = StopwatchMode::Running;
        // stopwatch_start is transient — not serialized.

        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("\"Running\""), "StopwatchMode::Running must serialize as 'Running'");

        let mut restored: CalcState = serde_json::from_str(&json).unwrap();

        // Before migration: Running mode was preserved in JSON.
        assert_eq!(
            restored.stopwatch_mode, StopwatchMode::Running,
            "StopwatchMode::Running must survive deserialization"
        );
        assert!(
            restored.stopwatch_start.is_none(),
            "stopwatch_start must be None after deserialization (transient field)"
        );

        restored.migrate_after_load();

        // After migration: D-38.6 freeze semantics applied.
        assert_eq!(
            restored.stopwatch_mode, StopwatchMode::Stopped,
            "migrate_after_load() must freeze Running → Stopped (D-38.6)"
        );
        assert!(
            restored.stopwatch_start.is_none(),
            "stopwatch_start must remain None after D-38.6 freeze"
        );
    }

    // Catches: default_xrom_modules() regression after Phase 38 default flip.
    // Complements the updated `xrom_modules_default_is_seven` test above.
    #[test]
    fn default_xrom_modules_returns_0b111() {
        use crate::ops::time::StopwatchMode;
        use crate::ops::time::ClockDisplayMode;

        let state = CalcState::new();

        // xrom_modules must be 7 (all 3 module bits set).
        assert_eq!(
            state.xrom_modules, 0b0000_0111,
            "CalcState::new() must have xrom_modules = 0b0000_0111 (Math 1 + Stat 1 + Time)"
        );

        // Time fields at correct defaults.
        assert_eq!(state.time_offset_secs, 0);
        assert!(!state.clock_12h);
        assert_eq!(state.clock_display_mode, ClockDisplayMode::Off);
        assert_eq!(state.stopwatch_mode, StopwatchMode::Idle);
        assert_eq!(state.stopwatch_accumulated, 0.0);
        assert_eq!(state.stopwatch_split, 0.0);
        assert!(state.stopwatch_start.is_none());
        assert!(!state.clock_active);
        assert!(!state.stopwatch_keyboard_mode);
        assert!(!state.alarm_catalog_mode);
        assert!(state.alarms.is_empty());
    }
}
