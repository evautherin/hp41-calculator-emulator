# Phase 43: hp41-core — XROM Framework + All Advantage Pac Ops - Pattern Map

**Mapped:** 2026-05-25
**Files analyzed:** 15 new/modified files
**Analogs found:** 15 / 15

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `hp41-core/src/ops/advantage/mod.rs` | module | CRUD | `hp41-core/src/ops/stat1/mod.rs` | exact |
| `hp41-core/src/ops/advantage/modal.rs` | modal state-machine | request-response | `hp41-core/src/ops/time/modal.rs` | exact |
| `hp41-core/src/ops/advantage/conv.rs` | service | transform | `hp41-core/src/ops/time/clock.rs` | role-match |
| `hp41-core/src/ops/advantage/matrix_ops.rs` | service | CRUD | `hp41-core/src/ops/stat1/basic_stats.rs` | role-match |
| `hp41-core/src/ops/advantage/matrix_linalg.rs` | service | transform | `hp41-core/src/ops/stat1/regression.rs` | role-match |
| `hp41-core/src/ops/advantage/matrix_complex.rs` | service | transform | `hp41-core/src/ops/math1/complex.rs` | role-match |
| `hp41-core/src/ops/advantage/complex_ext.rs` | service | transform | `hp41-core/src/ops/math1/complex.rs` | exact |
| `hp41-core/src/ops/advantage/solvers.rs` | service | event-driven | `hp41-core/src/ops/math1/integ.rs` + `solve.rs` | exact |
| `hp41-core/src/ops/advantage/poly.rs` | service | transform | `hp41-core/src/ops/math1/solve.rs` | role-match |
| `hp41-core/src/ops/advantage/matrix_workflow.rs` | service | request-response | `hp41-core/src/ops/stat1/normd.rs` | role-match |
| `hp41-core/src/ops/advantage/curve_fit.rs` | service | transform | `hp41-core/src/ops/stat1/regression.rs` | role-match |
| `hp41-core/src/ops/advantage/vectors.rs` | service | transform | `hp41-core/src/ops/math1/complex.rs` | role-match |
| `hp41-core/src/ops/advantage/tvm.rs` | service | CRUD | `hp41-core/src/ops/math1/solve.rs` | role-match |
| `hp41-core/src/ops/math1/xrom.rs` | config | request-response | self (freeze carve-out, bits 3+4 extension) | exact |
| `hp41-core/src/ops/math1/modal.rs` | modal state-machine | request-response | self (freeze carve-out, 4th variant) | exact |
| `hp41-core/src/ops/math1/complex.rs` | utility | transform | self (single-line visibility change) | exact |
| `hp41-core/src/state.rs` | model | CRUD | self (additive field extension) | exact |
| `hp41-core/src/ops/mod.rs` | config | request-response | self (additive Op enum extension) | exact |
| `hp41-core/src/ops/program.rs` | config | request-response | self (additive execute_op extension) | exact |
| `scripts/check-free42-contamination.sh` | config | — | self (directory extension) | exact |

---

## Pattern Assignments

### `hp41-core/src/ops/advantage/mod.rs` (module, CRUD)

**Analog:** `hp41-core/src/ops/stat1/mod.rs`

**File header + imports pattern** (lines 1-3 of stat1/mod.rs):
```rust
// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! `advantage` — HP Advantage Pac operations.
//!
//! XROM module ids: 22 (ADV CONV + ADV MTRX, bit 3) and 24 (ADV MATH + ADV TVM, bit 4)
//! of `CalcState::xrom_modules`. Activated in Phase 43 (v3.3).
```

**OM Storage Constants header pattern** (stat1/mod.rs lines 19-65 — transcription doc):
Copy the `//!` block style verbatim: lead with a table showing OM-cited register slots, then named const definitions. For Advantage Pac matrix ops the constants are matrix/row/col index semantics rather than register-layout semantics.

**Sub-module declarations + pub use re-exports pattern** (stat1/mod.rs and time/mod.rs lines 36-55 of time/mod.rs):
```rust
pub mod modal;
pub mod conv;
pub mod matrix_ops;
pub mod matrix_linalg;
pub mod matrix_complex;
pub mod complex_ext;
pub mod solvers;
pub mod poly;
pub mod matrix_workflow;
pub mod curve_fit;
pub mod vectors;
pub mod tvm;

pub use modal::AdvantageStep;
pub use tvm::TvmState;
// ... op function re-exports grouped by sub-module
```

**AdvMatrix struct pattern** (pattern from RESEARCH.md, serde shape from stat1/rand.rs precedent):
```rust
/// Named matrix entry for Advantage Pac X-MEM model (ADV-FW-04 / D-43.1).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdvMatrix {
    pub name: String,
    pub rows: u8,
    pub cols: u8,
    pub is_complex: bool,
    pub data: Vec<HpNum>,   // row-major; complex: len = 2*rows*cols
}

pub const ADV_MATRIX_MAX_ROWS: u8 = 255;
pub const ADV_MATRIX_MAX_COLS: u8 = 255;
pub const ADV_WORD_MASK: u64 = 0x0000_000F_FFFF_FFFF;  // 36-bit mask (D-43.9)
```

---

### `hp41-core/src/ops/advantage/modal.rs` (modal state-machine, request-response)

**Analog:** `hp41-core/src/ops/time/modal.rs` (closest) and `hp41-core/src/ops/stat1/modal.rs`

**File header pattern** (time/modal.rs lines 1-28):
```rust
// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Modal state-machine for Advantage Pac prompt-driven workflows.
//!
//! `AdvantageStep` is the per-program step enum carried by the
//! `ModalProgram::Advantage(AdvantageStep)` variant living in
//! `hp41-core/src/ops/math1/modal.rs` (math1/ freeze exception D-43.X).
//!
//! ## Why this file lives in `advantage/` not `math1/`
//!
//! D-43.X authorizes ONE additional math1/ freeze carve-out for
//! `math1/modal.rs` — a single-line variant addition plus three dispatch arms.
//! All Advantage-specific semantics (this file) stay in `advantage/` so the
//! math1/ blast radius stays to ~8 lines of pure dispatch wiring.
```

**Step enum pattern** (time/modal.rs lines 41-52):
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum AdvantageStep {
    /// TVM workflow — awaiting field selection.
    /// Prompt: "TVM?" or per-field prompt.
    TvmPrompt,
    /// MATRX/MTR — awaiting matrix name entry in ALPHA.
    /// Prompt: "MATRIX NAME?"
    MatrixNamePrompt,
    /// MEDIT — awaiting row entry for matrix editor.
    /// Prompt: "ROW?"
    MeditRowPrompt,
    // ... additional variants per OM 00041-90482 modal workflows
}
```

**Three exported functions pattern** (stat1/modal.rs lines 333-361 — the three free functions approach used by stat1 and time, NOT method-on-enum):
```rust
/// Per-step prompt accessor — called by `ModalProgram::current_prompt`
pub fn current_prompt(step: &AdvantageStep) -> Option<String> {
    match step {
        AdvantageStep::TvmPrompt => Some("TVM?".to_string()),
        // ... exhaustive match, NO _ => arm
    }
}

/// Per-step alpha-label gate — called by `ModalProgram::requires_alpha_label`
pub fn requires_alpha_label(step: &AdvantageStep) -> bool {
    match step {
        // ... exhaustive match
    }
}

/// Per-step submit dispatch — called by math1::submit_modal
pub fn submit_step(state: &mut CalcState, step: AdvantageStep) -> Result<(), HpError> {
    match step {
        // clear modal on exit: state.modal_program = None; state.modal_prompt = None;
    }
}
```

---

### `hp41-core/src/ops/math1/xrom.rs` (freeze carve-out — bits 3+4 extension)

**Analog:** self (existing structure at lines 253-278 for `xrom_resolve`, lines 120-244 for existing module constants)

**New XromModule constants pattern** (modeled on STAT_1 at lines 120-186 and TIME_MODULE at lines 188-244):
```rust
/// Advantage Pac Section A: ADV CONV + ADV MTRX (XROM 22) — Phase 43 (v3.3) freeze exception.
/// - `id = 22` — HP hardware Advantage Pac XROM 22 module ID.
/// - `name = "ADV 22A"` — CATALOG 2 display string (verify from OM 00041-90482).
/// math1/ freeze exception: fourth additive constant + two resolver arms,
/// following D-33.3 (STAT_1) and D-carried.5 (TIME_MODULE) precedents.
pub const ADV_MATH_A: XromModule = XromModule {
    id: 22,
    name: "ADV 22A",  // [ASSUMED] — verify against OM 00041-90482 CATALOG 2
    ops: &[
        ("BININ",   Op::AdvBinin),
        ("BINVIEW", Op::AdvBinview),
        // ... all 12 CONV + ~50 MTRX entries; section comments matching STAT_1 style
    ],
};

pub const ADV_MATH_B: XromModule = XromModule {
    id: 24,
    name: "ADV 24B",  // [ASSUMED] — verify against OM 00041-90482 CATALOG 2
    ops: &[
        ("FSOLVE", Op::AdvFsolve),
        // ... all ~47 MATH + 6 TVM entries
    ],
};
```

**xrom_resolve extension pattern** (lines 253-278, extending the existing if-chain):
```rust
pub fn xrom_resolve(name: &str, modules: u8) -> Option<Op> {
    // ... existing bit-0 (math1), bit-1 (stat1), bit-2 (time) arms ...
    // Phase 43 (v3.3): ADV_MATH_A bit-3 arm
    if modules & 0b0000_1000 != 0 {
        if let Some(op) = adv_a_resolve(name) {
            return Some(op);
        }
    }
    // Phase 43 (v3.3): ADV_MATH_B bit-4 arm
    if modules & 0b0001_0000 != 0 {
        if let Some(op) = adv_b_resolve(name) {
            return Some(op);
        }
    }
    None
}
```

**New resolver functions pattern** (modeled on `stat1_resolve` lines 379-425 and `time_resolve` lines 432-476):
```rust
/// Advantage Pac Section A (bit 3) mnemonic resolver — Phase 43 freeze exception.
///
/// Bidirectional consistency with `ADV_MATH_A.ops`: the
/// `adv_a_ops_mnemonics_resolve_consistently` test iterates the
/// slice and asserts every mnemonic round-trips through this match.
fn adv_a_resolve(name: &str) -> Option<Op> {
    match name {
        // ── ADV CONV ────────────────────────────────────────────────────────────
        "BININ"   => Some(Op::AdvBinin),
        "BINVIEW" => Some(Op::AdvBinview),
        // ... exhaustive; _ => None terminal
        _ => None,
    }
}

fn adv_b_resolve(name: &str) -> Option<Op> {
    match name {
        // ── ADV MATH ────────────────────────────────────────────────────────────
        "FSOLVE" => Some(Op::AdvFsolve),
        // ... exhaustive; _ => None terminal
        _ => None,
    }
}
```

**In-file test pattern** (lines 479-728 of xrom.rs — add new test block following the TIME_MODULE block at lines 676-728):
```rust
// ── Phase 43 (v3.3): ADV_MATH_A + ADV_MATH_B const + bit-3/bit-4 arm tests ──

#[test]
fn adv_math_a_const_id_and_name() {
    assert_eq!(ADV_MATH_A.id, 22, "ADV_MATH_A.id must be 22");
    assert_eq!(ADV_MATH_A.name, "ADV 22A", "...");
}

#[test]
fn resolve_uses_bit_3_for_adv_a() {
    // bit 3 set, bits 0+1+2 clear
    let result = xrom_resolve("BININ", 0b0000_1000);
    assert_eq!(result, Some(Op::AdvBinin), "...");
    // bits 0+1+2 set, bit 3 clear → None
    let without_bit3 = xrom_resolve("BININ", 0b0000_0111);
    assert!(without_bit3.is_none(), "...");
}
```

---

### `hp41-core/src/ops/math1/modal.rs` (freeze carve-out — 4th variant)

**Analog:** self (existing structure; extend ModalProgram enum at line 24, current_prompt at line 72, requires_alpha_label at line 100)

**Fourth variant addition pattern** (following Time variant at lines 52-58):
```rust
pub enum ModalProgram {
    // ... existing variants (Matrix, Solve, Poly, Integ, Difeq, Four, Trans, Stat1, Time) ...
    /// Advantage Pac workflows (Phase 43 — TVM/MATRX/MTR/MEDIT/CMEDIT/VE prompts).
    ///
    /// math1/ freeze exception per D-43.X: fourth additive variant following
    /// D-33.3b (Stat1) + D-carried.4 (Time).
    /// All Advantage-specific semantics live in `hp41-core/src/ops/advantage/modal.rs`.
    Advantage(crate::ops::advantage::modal::AdvantageStep),
}
```

**current_prompt dispatch arm** (lines 82-84 as model):
```rust
// D-43.X: Advantage Pac modal prompts delegate to advantage::modal.
ModalProgram::Advantage(step) => crate::ops::advantage::modal::current_prompt(step),
```

**requires_alpha_label dispatch arm** (lines 110-112 as model):
```rust
// D-43.X: Advantage Pac alpha-label gate delegates to advantage::modal.
ModalProgram::Advantage(step) => crate::ops::advantage::modal::requires_alpha_label(step),
```

---

### `hp41-core/src/ops/math1/complex.rs` (utility — single-line visibility change)

**Change:** Line 52 — change `pub(super)` to `pub(crate)` on `complex_atan2`:
```rust
// BEFORE (line 52):
pub(super) fn complex_atan2(im: HpNum, re: HpNum) -> HpNum {

// AFTER:
pub(crate) fn complex_atan2(im: HpNum, re: HpNum) -> HpNum {
```

Also remove `#[allow(dead_code)]` from line 51 since it will now be used from `advantage/complex_ext.rs`.

---

### `hp41-core/src/state.rs` (model — additive field extension)

**Analog:** self (existing `migrate_after_load` pattern at lines 466-486; `rand_seed` field pattern at lines 182-203)

**New persistent fields pattern** (insert after Phase 38 fields, modeled on lines 293-302 for `time_offset_secs` / `clock_12h`):
```rust
// ── Phase 43 (v3.3): Advantage Pac (XROM 22 + 24) ──────────────────────
/// Named matrices for Advantage Pac ADVMTRX (D-43.1, ADV-FW-04).
/// Isolated from Math Pac I's matrix_dim/matrix_active_reg (D-43.5).
/// Persistent — #[serde(default)].
#[serde(default)]
pub adv_matrices: Vec<crate::ops::advantage::AdvMatrix>,

/// Current I (row) index for ADVMTRX element access (D-43.4).
/// Global I/J per OM 00041-90482 §2 "I-register" semantics.
/// Persistent — #[serde(default)].
#[serde(default)]
pub adv_matrix_i: u8,

/// Current J (column) index for ADVMTRX element access (D-43.4).
/// Persistent — #[serde(default)].
#[serde(default)]
pub adv_matrix_j: u8,

/// TVM persistent state (D-43.11, ADV-TVM-01..06).
/// ⚠️ PERSISTENT — #[serde(default)] WITHOUT #[serde(skip)].
/// Mirrors rand_seed exception (line ~203): TVM must survive save/load.
#[serde(default)]
pub adv_tvm_state: Option<crate::ops::advantage::TvmState>,
```

**New transient fields pattern** (insert after adv_tvm_state; modeled on integ_state/solve_state at lines 229-246):
```rust
/// Current matrix name cached from ALPHA (D-43.3 — ALPHA-driven selection).
/// Transient — #[serde(default, skip)].
#[serde(default, skip)]
pub adv_current_matrix: Option<String>,

/// Transient solver state for FROOT (D-43.6, ADV-MATH-06).
/// #[serde(default, skip)].
#[serde(default, skip)]
pub adv_froot_state: Option<crate::ops::advantage::solvers::FrootState>,

/// Transient solver state for FINTG (D-43.6, ADV-MATH-04).
/// #[serde(default, skip)].
#[serde(default, skip)]
pub adv_fintg_state: Option<crate::ops::advantage::solvers::AdvFintegState>,

/// Transient solver state for FSOLVE (D-43.6, ADV-MATH-03).
/// #[serde(default, skip)].
#[serde(default, skip)]
pub adv_fsolve_state: Option<crate::ops::advantage::solvers::AdvFsolveState>,

/// Transient solver state for FDIFEQ (D-43.6, ADV-MATH-05).
/// #[serde(default, skip)].
#[serde(default, skip)]
pub adv_fdifeq_state: Option<crate::ops::advantage::solvers::AdvFdifeqState>,
```

**migrate_after_load extension pattern** (lines 466-486 — extend the idempotent OR-assign chain):
```rust
pub fn migrate_after_load(&mut self) {
    // ... existing v3.0→v3.1 (bit 1) and v3.1→v3.2 (bit 2) guards ...
    // v3.2 → v3.3: ensure ADV_MATH_A bit (bit 3) and ADV_MATH_B bit (bit 4) are set.
    if self.xrom_modules & 0b0000_1000 == 0 { self.xrom_modules |= 0b0000_1000; }
    if self.xrom_modules & 0b0001_0000 == 0 { self.xrom_modules |= 0b0001_0000; }
    // ... existing stopwatch freeze logic ...
}
```

**default_xrom_modules update** (currently returns `0b0000_0111`):
```rust
fn default_xrom_modules() -> u8 {
    0b0001_1111  // bits 0-4: Math1 + Stat1 + Time + AdvA + AdvB
}
```

**CalcState::new() additions** (insert after Phase 38 entries at lines 419-432):
```rust
// Phase 43 (v3.3): Advantage Pac fields
adv_matrices: Vec::new(),
adv_matrix_i: 0,
adv_matrix_j: 0,
adv_tvm_state: None,
adv_current_matrix: None,
adv_froot_state: None,
adv_fintg_state: None,
adv_fsolve_state: None,
adv_fdifeq_state: None,
```

---

### `hp41-core/src/ops/advantage/conv.rs` (service, transform)

**Analog:** `hp41-core/src/ops/time/clock.rs` (pure computation from register/alpha input)

**File header pattern** (clock.rs lines 1-18 as structural template):
```rust
// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! ADV CONV — base conversion and bitwise logic (ADV-CONV-01..12).
//!
//! All ops use 36-bit fixed word size (D-43.9). Overflow is silently truncated
//! to lower 36 bits (D-43.10): `result & ADV_WORD_MASK`.
```

**Imports pattern** (clock.rs lines 19-29):
```rust
use crate::error::HpError;
use crate::num::HpNum;
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
use crate::state::CalcState;
use super::ADV_WORD_MASK;
```

**Op function pattern** (clock.rs pattern — simple pure-computation ops):
```rust
/// BININ — convert binary string in ALPHA to integer (ADV-CONV-01).
///
/// Source: HP Advantage Pac OM 00041-90482 §1 "Base Conversions".
pub fn op_adv_binin(state: &mut CalcState) -> Result<(), HpError> {
    let s = state.alpha_reg.trim().to_string();
    if s.is_empty() {
        return Err(HpError::Domain);
    }
    let value = u64::from_str_radix(&s, 2).map_err(|_| HpError::Domain)?;
    let masked = value & ADV_WORD_MASK;
    let result = HpNum::from_u64(masked).ok_or(HpError::Domain)?;
    apply_lift_effect(state, LiftEffect::Enable);
    state.stack.x = result;
    Ok(())
}
```

**BINVIEW/HEXVIEW display pattern** (PRX pattern from print.rs — push to print_buffer):
```rust
/// BINVIEW — display X as binary string to print_buffer (ADV-CONV-02).
pub fn op_adv_binview(state: &mut CalcState) -> Result<(), HpError> {
    let x_u64 = x_to_u64_masked(&state.stack.x)?;
    let s = format!("{x_u64:b}");
    state.print_buffer.push(s);
    // LiftEffect::Neutral — display-only, does not consume X
    Ok(())
}
```

---

### `hp41-core/src/ops/advantage/solvers.rs` (service, event-driven)

**Analog:** `hp41-core/src/ops/math1/integ.rs` + `hp41-core/src/ops/math1/solve.rs` (run_loop re-entrancy pattern)

**State struct pattern** (integ.rs lines 58-83 and solve.rs lines 71-102):
```rust
/// Mid-iteration state for FINTG (Romberg integration).
/// CalcState.adv_fintg_state holds this as Option<AdvFintegState>.
/// #[serde(skip)] on CalcState field — purely transient.
#[derive(Debug, Clone, Default)]
pub struct AdvFintegState {
    pub user_label: String,
    pub a: HpNum,
    pub b: HpNum,
    pub tolerance: f64,
    pub iteration: u8,
    pub result: HpNum,
}

/// Mid-iteration state for FSOLVE (Brent/secant root finding).
#[derive(Debug, Clone, Default)]
pub struct AdvFsolveState {
    pub user_label: String,
    pub x1: HpNum,
    pub x2: HpNum,
    pub fx1: HpNum,
    pub fx2: HpNum,
    pub iteration: u8,
}

/// Mid-iteration state for FROOT (Laguerre polynomial root finding).
#[derive(Debug, Clone, Default)]
pub struct FrootState {
    pub degree: usize,
    pub coeffs: Vec<f64>,
    pub roots_found: Vec<(f64, f64)>,  // (re, im) pairs
    pub current_index: usize,
}

/// Mid-iteration state for FDIFEQ (RK4 ODE solver).
#[derive(Debug, Clone, Default)]
pub struct AdvFdifeqState {
    pub user_label: String,
    pub x: HpNum,
    pub y: HpNum,
    pub y_prime: HpNum,   // for 2nd order; ignored for 1st order
    pub order: u8,
    pub step_size: HpNum,
    pub iteration: u32,
}
```

**Dispatch arm stub pattern** (solve.rs lines 117-120 for the dispatch arm that rejects non-run_loop calls):
```rust
/// Dispatch arm for Op::AdvFsolve (interactive / dispatch() call).
/// Returns Err(HpError::InvalidOp) when not running in a program.
/// Real implementation is op_adv_fsolve_run_loop.
pub fn op_adv_fsolve(state: &mut CalcState) -> Result<(), HpError> {
    if !state.is_running {
        return Err(HpError::InvalidOp);
    }
    // Set up solver state, then return — run_loop handles iteration.
    // ...
    Ok(())
}
```

**Solver nesting guard pattern** (D-43.7 — deliberately RELAXED from Math Pac I strict guard in integ.rs):
```rust
// FINTG run_loop arm — allows nesting inside FSOLVE (D-43.7):
// Guard: only rejects when adv_fintg_state is ALREADY Some.
// Does NOT reject when adv_fsolve_state.is_some() — this IS the one-level nesting path.
if state.adv_fintg_state.is_some() {
    return Err(HpError::InvalidOp); // already integrating (recursion guard)
}
// Compare with Math Pac I STRICT guard (integ.rs pattern — DO NOT copy for Advantage solvers):
// if state.integ_state.is_some() || state.solve_state.is_some() || state.difeq_state.is_some() { ... }
```

**Cancel flag pattern** (integ.rs uses `state.cancel_requested.load(Ordering::Relaxed)` every N iterations — same approach for Advantage solvers):
```rust
use std::sync::atomic::Ordering;
// Inside iteration loop:
if state.cancel_requested.load(Ordering::Relaxed) {
    state.adv_fintg_state = None;
    return Err(HpError::Cancelled);
}
```

**Non-convergence / print_buffer pattern** (solve.rs "NO ROOT FOUND" pattern):
```rust
// *I non-convergence (D-43.13):
state.print_buffer.push("NO SOLUTION".to_string());
state.adv_tvm_state = None;
return Err(HpError::NoRoot);
```

---

### `hp41-core/src/ops/advantage/tvm.rs` (service, CRUD)

**Analog:** `hp41-core/src/ops/math1/solve.rs` (Newton iteration pattern) + `hp41-core/src/ops/stat1/rand.rs` (persistent state pattern)

**TvmState struct pattern** (from RESEARCH.md, serde shape mirrors rand_seed exception):
```rust
/// Time Value of Money persistent state (D-43.11 / ADV-TVM).
/// ⚠️ NOT #[serde(skip)] — must survive save/load (same exception as rand_seed).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TvmState {
    pub n: HpNum,
    pub i: HpNum,     // periodic interest rate (percent, e.g. 5.0 = 5%)
    pub pv: HpNum,
    pub pmt: HpNum,
    pub fv: HpNum,
    pub begin_mode: bool,  // false = END, true = BEGIN (D-43.12)
}
```

**Op function pattern** (set one field from X register, returning after store):
```rust
/// N — store X as number of periods into TVM state (ADV-TVM-02).
pub fn op_adv_tvm_n(state: &mut CalcState) -> Result<(), HpError> {
    let tvm = state.adv_tvm_state.get_or_insert_with(TvmState::default);
    tvm.n = state.stack.x.clone();
    // LiftEffect::Neutral — stores value, does not consume or produce stack result
    Ok(())
}
```

**Newton iteration constants** (solve.rs lines 55-68 as model):
```rust
const TVM_MAX_ITERATIONS: u8 = 100;
const TVM_CONVERGENCE_THRESHOLD: f64 = 1e-9;
```

---

### `hp41-core/src/ops/advantage/matrix_ops.rs` (service, CRUD)

**Analog:** `hp41-core/src/ops/stat1/basic_stats.rs` (register-access, validation pattern)

**Isolation guard pattern** (D-43.5 — add as a comment at the top of the file):
```rust
// D-43.5 ISOLATION INVARIANT: This file MUST NOT reference state.matrix_dim
// or state.matrix_active_reg. Those belong exclusively to Math Pac I's
// R14/R15+ register-based matrix model. CI gate:
// grep -rn "matrix_dim\|matrix_active_reg" hp41-core/src/ops/advantage/
// must return empty.
```

**Matrix lookup helper pattern** (consistent with how stat1 ops do SIZE-floor checks):
```rust
fn find_matrix_by_name<'a>(
    matrices: &'a [crate::ops::advantage::AdvMatrix],
    name: &str,
) -> Result<(usize, &'a crate::ops::advantage::AdvMatrix), HpError> {
    matrices
        .iter()
        .enumerate()
        .find(|(_, m)| m.name == name)
        .ok_or(HpError::InvalidOp)
}
```

**MATDIM op pattern** (creates or resizes named matrix — allocation with bounds check):
```rust
/// MATDIM — define or resize a named matrix (ADV-MTX-17).
/// Y = rows, X = cols. Name from state.alpha_reg. D-43.5: no matrix_dim touch.
pub fn op_adv_matdim(state: &mut CalcState) -> Result<(), HpError> {
    let rows = state.stack.y.trunc_int().inner().to_u8_safe()?;
    let cols = state.stack.x.trunc_int().inner().to_u8_safe()?;
    if rows == 0 || cols == 0
        || rows > crate::ops::advantage::ADV_MATRIX_MAX_ROWS
        || cols > crate::ops::advantage::ADV_MATRIX_MAX_COLS
    {
        return Err(HpError::Domain);
    }
    let name = state.alpha_reg.trim().to_string();
    if name.is_empty() {
        return Err(HpError::InvalidOp);
    }
    let data_len = (rows as usize) * (cols as usize);
    if let Some(m) = state.adv_matrices.iter_mut().find(|m| m.name == name) {
        m.rows = rows;
        m.cols = cols;
        m.data.resize(data_len, HpNum::zero());
    } else {
        state.adv_matrices.push(crate::ops::advantage::AdvMatrix {
            name,
            rows,
            cols,
            is_complex: false,
            data: vec![HpNum::zero(); data_len],
        });
    }
    // Drop Y+X (both consumed). Stack: X←Z, Y←T, Z←T, T←T.
    state.stack.x = state.stack.z.clone();
    state.stack.y = state.stack.t.clone();
    state.stack.z = state.stack.t.clone();
    Ok(())
}
```

---

### `hp41-core/src/ops/advantage/matrix_linalg.rs` (service, transform)

**Analog:** `hp41-core/src/ops/stat1/regression.rs` (multi-step computation returning results to stack)

**LU decomposition helper** (standard pattern — hand-rolled per ADR-v3.1-002):
```rust
/// LU decomposition with partial pivoting.
/// Returns (L*U factored matrix in-place, permutation vec) or Err(Domain) for singular.
/// Gaussian elimination with partial pivoting (~150 LOC).
/// Source: Numerical Recipes §2.3 (primary literature, re-derived).
fn lu_decompose(
    mat: &mut [f64],
    n: usize,
) -> Result<Vec<usize>, HpError> {
    // ... no free42 code, no external crate (ADR-v3.1-002)
}
```

**MDET pattern** (read matrix by ALPHA name, compute, push to X):
```rust
/// MDET — matrix determinant via LU decomposition (ADV-MTX-20).
pub fn op_adv_mdet(state: &mut CalcState) -> Result<(), HpError> {
    let name = state.alpha_reg.trim().to_string();
    let (_, mat) = find_matrix_by_name(&state.adv_matrices, &name)?;
    if mat.rows != mat.cols {
        return Err(HpError::Domain);  // non-square
    }
    // ... LU decompose, compute det from diagonal product + permutation sign
    apply_lift_effect(state, LiftEffect::Enable);
    state.stack.x = det_result;
    Ok(())
}
```

---

### `hp41-core/src/ops/advantage/complex_ext.rs` (service, transform)

**Analog:** `hp41-core/src/ops/math1/complex.rs` (complex operations following the same stack convention)

**Import pattern** (promote complex_atan2 via `pub(crate)` change in math1/complex.rs):
```rust
use crate::ops::math1::complex::complex_atan2;  // pub(crate) after single-line carve-out
```

**Op function pattern** (matches math1/complex.rs style for complex ops):
```rust
/// MAGZ — complex magnitude (ADV-MATH complex extension).
///
/// Stack effect: X' = sqrt(X^2 + Y^2) (magnitude of complex X+iY).
/// Calls complex_atan2 for arg(ζ) computation (Pitfall 3 — reuse existing, do not duplicate).
pub fn op_adv_magz_ext(state: &mut CalcState) -> Result<(), HpError> {
    // Note: Advantage MAGZ is the same as Math Pac I MAGZ — this file handles
    // any ADDITIONAL complex extensions not in math1/complex.rs.
    todo!()
}
```

---

### `hp41-core/src/ops/mod.rs` (additive Op enum extension)

**Analog:** self (existing Op enum; Time Module block at lines 1263+ as structural template)

**New variant block pattern** (insert after Time Module variants; comment style matches existing blocks):
```rust
// ── Phase 43 (v3.3): Advantage Pac XROM 22 (ADV CONV + ADV MTRX) ────────────
// 4-way invariant items 1+2 complete here. Items 3+4 (CLI/GUI prgm_display)
// deferred to Phase 44/46 with sanctioned CI break.
/// BININ — binary string in ALPHA to integer (ADV-CONV-01, ADV-FW-06).
AdvBinin,
/// BINVIEW — display X as binary string (ADV-CONV-02).
AdvBinview,
// ... all 12 CONV entries, then ~50 MTRX entries ...

// ── Phase 43 (v3.3): Advantage Pac XROM 24 (ADV MATH + ADV TVM) ─────────────
/// FSOLVE — root-finding via Brent/secant method (ADV-MATH-03).
AdvFsolve,
// ... all ~47 MATH + 6 TVM entries ...
```

**dispatch() arm pattern** (lines 1723-1749 as model — `Op::AdvXxx => crate::ops::advantage::sub::op_adv_xxx(state)`):
```rust
// ── Phase 43 (v3.3): Advantage Pac (ADV-FW-06) ──────────────────────────────
Op::AdvBinin => crate::ops::advantage::conv::op_adv_binin(state),
Op::AdvBinview => crate::ops::advantage::conv::op_adv_binview(state),
// ... exhaustive; sanctioned CI break for prgm_display items 3+4
```

---

### `hp41-core/src/ops/program.rs` (additive execute_op extension)

**Analog:** self (existing execute_op exhaustive match; Time Module additions as model)

**execute_op arm pattern** (mirrors dispatch() but with run_loop routing for solver ops):
```rust
// ── Phase 43 (v3.3): Advantage Pac solver ops route through run_loop arms ────
Op::AdvFsolve => crate::ops::advantage::solvers::op_adv_fsolve(state),
Op::AdvFintg  => crate::ops::advantage::solvers::op_adv_fintg(state),
// Non-solver ops use same crate::ops::advantage::... path as dispatch()
```

---

### `scripts/check-free42-contamination.sh` (config — directory extension)

**Analog:** self (existing structure; extend the three-directory loop and add ADV_DIR check)

**Extension pattern** (lines 23-38 and 63-71 as model):
```bash
ADV_DIR="hp41-core/src/ops/advantage"

# Extend directory existence check:
for dir in "$MATH1_DIR" "$STAT1_DIR" "$TIME_DIR" "$ADV_DIR"; do
    if [[ ! -d "$dir" ]]; then
        echo "FAIL: $dir does not exist — license guard cannot run." >&2
        exit 2
    fi
done

# Extend scan loop (PATTERN unchanged — 21 tokens cover advantage/ as-is):
for dir in "$MATH1_DIR" "$STAT1_DIR" "$TIME_DIR" "$ADV_DIR"; do
    if matches=$(grep -rn -E "$PATTERN" "$dir" | grep -v "$DISCLAIM_LINE"); then
        ...
    fi
done

echo "OK: no Free42 contamination detected in $MATH1_DIR/ or $STAT1_DIR/ or $TIME_DIR/ or $ADV_DIR/"
```

---

## Shared Patterns

### Free42 Disclaim Header
**Source:** `hp41-core/src/ops/stat1/rand.rs` line 1-2 (stat1 wording); `hp41-core/src/ops/time/clock.rs` line 1-2 (time wording)
**Apply to:** Every file under `hp41-core/src/ops/advantage/`
```rust
// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
```

### Serde Pattern — Persistent Fields
**Source:** `hp41-core/src/state.rs` lines 293-302 (time_offset_secs, clock_12h pattern)
**Apply to:** `adv_matrices`, `adv_matrix_i`, `adv_matrix_j`, `adv_tvm_state`
```rust
#[serde(default)]
pub field_name: FieldType,
```

### Serde Pattern — Transient Fields
**Source:** `hp41-core/src/state.rs` lines 229-246 (integ_state, solve_state, difeq_state)
**Apply to:** `adv_current_matrix`, `adv_froot_state`, `adv_fintg_state`, `adv_fsolve_state`, `adv_fdifeq_state`
```rust
#[serde(default, skip)]
pub field_name: Option<StateType>,
```

### Serde Exception — Persistent Optional State (TVM)
**Source:** `hp41-core/src/state.rs` lines 182-203 (`rand_seed` — sole `#[serde(default)]` WITHOUT `#[serde(skip)]` on an otherwise-transient-shaped field)
**Apply to:** `adv_tvm_state` ONLY
```rust
// #[serde(default)] — NO skip. TVM state must survive save/load (D-43.11).
#[serde(default)]
pub adv_tvm_state: Option<crate::ops::advantage::TvmState>,
```

### No-unwrap Pattern
**Source:** `hp41-core/src/ops/stat1/modal.rs` line 199-203 (`#[allow(clippy::unwrap_used)]` on test module)
**Apply to:** All `advantage/*.rs` files — production code: no `.unwrap()`; test modules carry `#[allow(clippy::unwrap_used)]` at file or module scope.

### Stack LiftEffect Pattern
**Source:** `hp41-core/src/ops/time/clock.rs` (uses `apply_lift_effect` + `enter_number` from `crate::stack`)
**Apply to:** All `advantage/*.rs` op functions
```rust
use crate::stack::{apply_lift_effect, enter_number, LiftEffect};
// Producing a result:
apply_lift_effect(state, LiftEffect::Enable);
state.stack.x = result;
// Neutral (display/store only):
// LiftEffect::Neutral — no apply_lift_effect call needed
```

### Error Handling Pattern
**Source:** `hp41-core/src/ops/stat1/modal.rs` lines 153-310 (clear modal state before returning error; clear modal before dispatching)
**Apply to:** All modal `submit_step` arms in `advantage/modal.rs`
```rust
// Clear modal state BEFORE dispatching (modal ops see clean context):
state.modal_program = None;
state.modal_prompt = None;
// On domain error from validation — optionally clear or keep modal (D-07 never-discard principle):
return Err(HpError::Domain);
```

### print_buffer Display Pattern
**Source:** `hp41-core/src/ops/math1/solve.rs` lines (push strings to print_buffer for results like "ROOT IS X", "NO ROOT FOUND")
**Apply to:** `advantage/conv.rs` (BINVIEW/HEXVIEW/CVTVIEW), `advantage/solvers.rs` (*I non-convergence "NO SOLUTION"), `advantage/tvm.rs`
```rust
state.print_buffer.push(format!("RESULT: {}", value));
```

### XROM Module Bidirectional Test Pattern
**Source:** `hp41-core/src/ops/math1/xrom.rs` lines 586-641 (`stat1_ops_mnemonics_resolve_consistently` test) and lines 694-727 (`time_module_ops_mnemonics_resolve_consistently` test)
**Apply to:** In-file tests in `math1/xrom.rs` for `ADV_MATH_A` and `ADV_MATH_B`
```rust
#[test]
fn adv_a_ops_mnemonics_resolve_consistently() {
    for (name, expected_op) in ADV_MATH_A.ops {
        let resolved = xrom_resolve(name, 0b0000_1000);
        assert_eq!(resolved.as_ref(), Some(expected_op), "ADV_MATH_A mnemonic {name:?} must resolve");
    }
}
```

---

## No Analog Found

All files in Phase 43 have strong analogs from prior phases. No files are entirely without precedent.

The `advantage/matrix_linalg.rs` algorithms (LU decomposition, Frobenius norm, matrix multiply) are hand-rolled per ADR-v3.1-002 — no analog in the codebase for the algorithm internals, but the file structure and op-function shape follow `stat1/regression.rs` closely.

---

## Metadata

**Analog search scope:** `hp41-core/src/ops/math1/`, `hp41-core/src/ops/stat1/`, `hp41-core/src/ops/time/`, `hp41-core/src/state.rs`, `hp41-core/src/ops/mod.rs`, `scripts/`
**Files scanned:** 20 files read in full or targeted sections
**Pattern extraction date:** 2026-05-25
