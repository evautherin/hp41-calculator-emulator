# Phase 43: hp41-core — XROM Framework + All Advantage Pac Ops - Research

**Researched:** 2026-05-25
**Domain:** Rust hp41-core — XROM 22/24 registration, named-matrix model, solver callback re-entrancy, ~117 Advantage Pac Op variants
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-43.1:** Unlimited matrix count. `adv_matrices: Vec<AdvMatrix>` grows without cap.
- **D-43.5:** Named-matrix storage MUST NOT touch `state.matrix_dim` or `state.matrix_active_reg`. Complete isolation between the two matrix systems.
- **D-43.7:** One level of solver nesting is REQUIRED (FINTG inside FSOLVE, or vice versa).
- **D-43.9:** 36-bit fixed word size for NOT/AND/OR/XOR/ROTXY/BIT?.
- **D-43.10:** Silent truncation (mask to 36 bits) on overflow.
- **D-43.11:** TVM state is persistent (`#[serde(default)]`, not `skip`).
- **ADV-FW-06:** 4-way exhaustive-match invariant items 1+2 in `dispatch()` + `execute_op()` MUST be satisfied this phase. Items 3+4 (CLI prgm_display, GUI prgm_display) deferred to Phase 44/46 with sanctioned CI break.

### Claude's Discretion

- **D-43.2:** Maximum matrix size cap (determine from OM 00041-90482 constraints).
- **D-43.3:** Current-matrix selection mechanism (ALPHA register names current matrix vs dedicated field).
- **D-43.4:** I/J index storage (per-matrix vs global).
- **D-43.6:** Callback mechanism architecture (reuse `run_loop` re-entrancy vs separate).
- **D-43.8:** FROOT calling convention (degree from X register vs modal prompt).
- **D-43.12:** BEGIN/END payment mode support.
- **D-43.13:** *I non-convergence behavior.

### Deferred Ideas (OUT OF SCOPE)

- NEST-01: Deep FROOT/FINTG mutual nesting (post-v3.3).
- XMEM-01: Full Extended Memory model — EMDIR/EMROOM/EMREG etc. (post-v3.3).
- `hp41-cli/` changes (Phase 44).
- `hp41-gui/` changes (Phase 46).
- Documentation / ADRs (Phase 45).
- Test hardening / quality gates (Phase 47).
- Advanced Matrix Pac (XROM 12 — Angel Martin community ROM, NOT official HP).

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ADV-FW-01 | XROM 22 (ADV CONV + ADV MTRX) and XROM 24 (ADV MATH + ADV TVM) registered as XromModule constants | `ADV_MATH_A` (id=22) + `ADV_MATH_B` (id=24) constants in `math1/xrom.rs` freeze carve-out; pattern verified from `MATH_1`, `STAT_1`, `TIME_MODULE` |
| ADV-FW-02 | `xrom_resolve` gains bit-3 and bit-4 arms; fires after TIME_MODULE | Extend `xrom_resolve()` with `modules & 0b0000_1000` (bit 3) and `modules & 0b0001_0000` (bit 4) arms, per existing bit-0/1/2 pattern |
| ADV-FW-03 | `default_xrom_modules()` → `0b0001_1111`; `migrate_after_load()` upgrades v3.2 saves | State.rs migration chain: two new OR-assign guards for bits 3+4; idempotent pattern confirmed |
| ADV-FW-04 | `adv_matrices: Vec<AdvMatrix>` on CalcState with `#[serde(default)]` | New struct + persistent field; isolation from `matrix_dim`/`matrix_active_reg` (Math Pac I) mandated by D-43.5 |
| ADV-FW-05 | `ModalProgram::Advantage(AdvantageStep)` variant + dispatch wired in `math1/modal.rs` freeze carve-out | Fourth additive variant following `Stat1` (D-33.3b) and `Time` (D-carried.4) patterns |
| ADV-FW-06 | All new Op variants compile in dispatch() + execute_op() | ~117 variants; items 3+4 sanctioned-deferred |
| ADV-CONV-01..12 | Base conversion + bitwise logic, 36-bit word size | 12 ops; pure integer arithmetic; 36-bit mask per D-43.9/D-43.10 |
| ADV-MTX-01..50 | Named-matrix ~50 ops | I/J indexing, lifecycle, high-level ops, reductions, complex matrix, editors |
| ADV-MATH-01..47 | Advanced mathematics ~47 ops | Solver callbacks, polynomial roots, complex extensions, curve fitting, vectors, coord transforms |
| ADV-TVM-01..06 | TVM 6 ops | Newton iteration for *I; persistent TVM state per D-43.11 |

</phase_requirements>

---

## Summary

Phase 43 is the largest single hp41-core phase in the v3.3 milestone: ~117 new `Op` variants across four functional sections (ADV CONV, ADV MTRX, ADV MATH, ADV TVM) plus two new XROM module registrations and new CalcState fields. The phase is structurally identical to Phase 38 (Time Module core) but three times larger in Op count.

The code architecture is well-understood: the XROM registration, modal dispatch, solver re-entrancy, and CalcState field patterns are all fully established by Phases 28–42. The primary implementation challenges are (1) the named-matrix model design (completely separate from Math Pac I's R14/R15+ layout), (2) the solver callback nesting contract (one level required per D-43.7), and (3) the ~50 matrix operation implementations (Gaussian elimination, LU decomposition, Frobenius norm, transpose, etc.).

**Primary recommendation:** Follow the `stat1/` and `time/` module structures exactly. Organize `advantage/` with per-feature files: `conv.rs`, `matrix_ops.rs` (element access + lifecycle + reductions), `matrix_linalg.rs` (DET/INV/MSYS/M*M), `matrix_complex.rs`, `complex_ext.rs`, `solvers.rs` (FSOLVE/FINTG/FDIFEQ/FROOT), `poly.rs` (PLY/RTS), `curve_fit.rs`, `vectors.rs`, `tvm.rs`, `modal.rs`. Add `adv_resolve` and `adv_b_resolve` in `math1/xrom.rs` freeze carve-out following the existing four-resolver pattern.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| XROM registration (bits 3+4) | hp41-core | — | Module registration lives entirely in core; frontend sees only xrom_modules u8 |
| Named-matrix storage (`adv_matrices`) | hp41-core | — | Persistent state; frontend never accesses raw matrix data |
| Modal prompting (TVM/MATRX/MEDIT/VE) | hp41-core | CLI/GUI surface | Core sets `modal_program`/`modal_prompt`; CLI/GUI renders and submits R/S |
| FROOT/FINTG/FSOLVE solver callbacks | hp41-core run_loop | — | User-program re-entrancy is entirely within core; CLI/GUI is passive |
| ADV CONV bitwise ops | hp41-core | — | Pure integer arithmetic; no UI |
| ADV MTRX element access (I+/I-/MR/MS) | hp41-core | — | Index state lives in CalcState |
| Complex number extensions | hp41-core | — | Reuse `complex_atan2` (promote pub(crate)); no new UI |
| TVM persistence | hp41-core CalcState | — | `adv_tvm_state: Option<TvmState>` #[serde(default)] per D-43.11 |
| ADV CONV display (BINVIEW/HEXVIEW/CVTVIEW) | hp41-core print_buffer | CLI/GUI drain | Core pushes to print_buffer; CLI/GUI drains (same as PRX pattern) |
| Op variant enum + dispatch | hp41-core ops/mod.rs | — | 4-way invariant; items 3+4 deferred to Phase 44/46 |

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rust_decimal | 1.42 | HpNum arithmetic (BCD emulation) | Project invariant [ASSUMED] |
| serde + serde_json | current | Serialize/Deserialize CalcState | Project invariant [ASSUMED] |
| std (no_std excluded) | stable | SystemTime, AtomicBool, collections | Core uses std per ADR-v3.2-001 pattern [ASSUMED] |

**No new runtime dependencies.** ADR-v3.1-002 invariant holds. All algorithms (Laguerre, Romberg, Newton for TVM, Brent/secant for FSOLVE, LU decomposition) are implemented from primary sources (~150–250 LOC each). [VERIFIED: codebase, SUMMARY.md]

### Algorithms (hand-rolled per ADR-v3.1-002)

| Algorithm | Phase Use | Source | Lines est. |
|-----------|-----------|--------|------------|
| Laguerre's method | FROOT — polynomial roots, arbitrary degree | Numerical Recipes §9.5; primary literature | ~120 LOC |
| Romberg integration | FINTG — adaptive numerical integration | Numerical Recipes §4.3; re-derive from OM 00041-90482 §3 | ~100 LOC (reuses INTG pattern) |
| Newton iteration | TVM *I — periodic rate | Standard Newton-Raphson; OM §4 TVM formulas | ~60 LOC |
| Brent/secant method | FSOLVE — root finding | Reuse SOLVE's secant skeleton (op_solve_run_loop pattern) | ~80 LOC delta |
| LU decomposition | MDET/MINV/MSYS | Gaussian elimination with partial pivoting | ~150 LOC |
| Frobenius/row norm | FNRM/RNRM | Direct summation | ~20 LOC |

## Package Legitimacy Audit

No new external packages. Phase 43 installs zero new Rust crates. This section is N/A.

---

## Architecture Patterns

### System Architecture Diagram

```
                         ┌──────────────────────────────────────────┐
                         │              hp41-core                   │
                         │                                          │
  XEQ "BININ" ──────────►│ xrom_resolve()                           │
  XEQ "MATDIM" ──────────►│   bit-0: math1_resolve (MATH_1)         │
  XEQ "FROOT"  ──────────►│   bit-1: stat1_resolve (STAT_1)         │
  XEQ "TVM"    ──────────►│   bit-2: time_resolve (TIME_MODULE)     │
                         │   bit-3: adv_a_resolve (ADV_MATH_A/22)  │
                         │   bit-4: adv_b_resolve (ADV_MATH_B/24)  │
                         │            │                             │
                         │            ▼                             │
                         │       dispatch() / execute_op()          │
                         │            │                             │
                         │     ┌──────┴────────────────────┐        │
                         │     │      advantage/            │        │
                         │     │  ┌───────────────────┐    │        │
                         │     │  │ conv.rs            │    │        │
                         │     │  │  BININ/HEXIN/NOT.. │    │        │
                         │     │  ├───────────────────┤    │        │
                         │     │  │ matrix_ops.rs      │    │        │
                         │     │  │  MATDIM/MR/MS/I+.. │    │        │
                         │     │  ├───────────────────┤    │        │
                         │     │  │ matrix_linalg.rs   │    │        │
                         │     │  │  MDET/MINV/MSYS/.. │    │        │
                         │     │  ├───────────────────┤    │        │
                         │     │  │ solvers.rs         │──►run_loop │
                         │     │  │  FROOT/FINTG/FSOLVE│    │        │
                         │     │  ├───────────────────┤    │        │
                         │     │  │ tvm.rs             │    │        │
                         │     │  │  TVM/N/PV/PMT/FV/*I│    │        │
                         │     │  └───────────────────┘    │        │
                         │     └───────────────────────────┘        │
                         │                                          │
                         │   CalcState mutations:                   │
                         │   adv_matrices: Vec<AdvMatrix>           │
                         │   adv_tvm_state: Option<TvmState>        │
                         │   modal_program: ModalProgram::Advantage │
                         │   print_buffer (BINVIEW/HEXVIEW/CVTVIEW) │
                         └──────────────────────────────────────────┘
```

### Recommended Project Structure
```
hp41-core/src/ops/advantage/
├── mod.rs          # OM storage constants + pub use re-exports + ADV_MATRIX_{MAX_ROWS,MAX_COLS}
├── modal.rs        # AdvantageStep enum + current_prompt() + requires_alpha_label()
├── conv.rs         # ADV CONV: BININ/BINVIEW/OCTIN/HEXIN/HEXVIEW/CVTVIEW/NOT/AND/OR/XOR/ROTXY/BIT?
├── matrix_ops.rs   # ADV MTRX element access (I+/I-/J+/J-/MR/MS/MRIJ/MSIJ/MR{C/R}+/-/MS{C/R}+)
│                   #   + lifecycle (MATDIM/DIM?/MNAME?) + reductions (SUM/MAX/MIN/FNRM/RNRM/RSUM..)
│                   #   + matrix editors (MEDIT/CMEDIT) + row ops (MSWAP/R<>R/R>R?/PIV/MP)
├── matrix_linalg.rs # ADV MTRX high-level: MDET/MINV/MSYS/M*M/MAT+/MAT-/MAT*/MAT//TRNPS/MMOVE
├── matrix_complex.rs # ADV MTRX complex ops: C<>C/CMAXAB/CNRM/CSUM/YC+C
├── complex_ext.rs  # ADV MATH complex extensions: MAGZ/e^Z/LNZ/Z^N/Z^1/N/SINZ/COSZ/TANZ/a^Z/LOGZ/Z^1/W/Z^W
│                   #   + ADV MATH complex arithmetic variants (C+/C-/CINV/C*/C/)
├── solvers.rs      # ADV MATH: FSOLVE/FINTG/FDIFEQ/FROOT (solver states + run_loop arms)
├── poly.rs         # ADV MATH: PLY (polynomial eval) + RTS (root output)
├── matrix_workflow.rs # ADV MATH: MATRX/MTR modal frontends + AIP
├── curve_fit.rs    # ADV MATH: CFIT/AS/DS/BFIT/FIT/Y?X/SZ?
├── vectors.rs      # ADV MATH: V+/V-/DOT/CROSS/VC/VS/VR/VE/VXY/UV/V</V*/VD + TR
└── tvm.rs          # ADV TVM: TVM/N/PV/PMT/FV/*I + TvmState struct
```

### CalcState Additions

```rust
// In state.rs — Phase 43 additions

/// Named matrices for Advantage Pac ADVMTRX (D-43.1, ADV-FW-04).
/// Identified by ALPHA name. NOT related to Math Pac I's matrix_dim/matrix_active_reg.
/// Persistent — #[serde(default)].
#[serde(default)]
pub adv_matrices: Vec<AdvMatrix>,

/// Current matrix name for MTRX ops (D-43.3 Claude's discretion — ALPHA-driven).
/// None when no matrix is selected. Transient — #[serde(default, skip)].
#[serde(default, skip)]
pub adv_current_matrix: Option<String>,

/// Current I (row) index for ADVMTRX element access (D-43.4 Claude's discretion).
/// Design choice: global I/J per OM 00041-90482 §2 "I-register" semantics.
/// Persistent — #[serde(default)].
#[serde(default)]
pub adv_matrix_i: u8,

/// Current J (column) index for ADVMTRX element access.
/// Persistent — #[serde(default)].
#[serde(default)]
pub adv_matrix_j: u8,

/// TVM persistent state (D-43.11, ADV-TVM). Not #[serde(skip)] — survives sessions.
/// None until first TVM invocation. #[serde(default)].
#[serde(default)]
pub adv_tvm_state: Option<TvmState>,

/// Transient solver state for FROOT (parallel to integ_state/solve_state/difeq_state).
/// #[serde(default, skip)].
#[serde(default, skip)]
pub adv_froot_state: Option<FrootState>,

/// Transient solver state for FINTG (Romberg — separate from Math Pac I INTG Simpson).
/// #[serde(default, skip)].
#[serde(default, skip)]
pub adv_fintg_state: Option<AdvFintegState>,

/// Transient solver state for FSOLVE.
/// #[serde(default, skip)].
#[serde(default, skip)]
pub adv_fsolve_state: Option<AdvFsolveState>,

/// Transient solver state for FDIFEQ (separate from Math Pac I DIFEQ).
/// #[serde(default, skip)].
#[serde(default, skip)]
pub adv_fdifeq_state: Option<AdvFdifeqState>,
```

### AdvMatrix Struct

```rust
// In advantage/mod.rs or advantage/matrix_ops.rs

/// Named matrix entry for Advantage Pac X-MEM model (ADV-FW-04 / D-43.1).
///
/// Identified by ALPHA name; rows and cols capped at ADV_MATRIX_MAX_ROWS/COLS.
/// Data stored row-major. Complex matrices interleave real/imag pairs:
/// element (i,j) real = data[2*(i*cols+j)], imag = data[2*(i*cols+j)+1].
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdvMatrix {
    pub name: String,
    pub rows: u8,
    pub cols: u8,
    pub is_complex: bool,
    pub data: Vec<HpNum>,   // row-major; complex matrices: len = 2*rows*cols
}

/// Maximum rows or columns per named matrix (D-43.2 Claude's discretion).
/// OM 00041-90482 §2 "Maximum matrix dimensions" is 8 rows × 8 cols per
/// available X-MEM slots on hardware (each slot = 8 registers × 7 values).
/// Emulator extension: raise cap to 255 × 255 for practical use.
/// Document as emulator extension in hp41-advantage-divergences.md.
pub const ADV_MATRIX_MAX_ROWS: u8 = 255;
pub const ADV_MATRIX_MAX_COLS: u8 = 255;
```

### TvmState Struct

```rust
// In advantage/tvm.rs

/// Time Value of Money persistent state (D-43.11 / ADV-TVM).
/// Survives save/load (NOT #[serde(skip)]). Same serde contract as rand_seed.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TvmState {
    pub n: HpNum,     // number of periods
    pub i: HpNum,     // periodic interest rate (as percent, e.g. 5.0 = 5%)
    pub pv: HpNum,    // present value
    pub pmt: HpNum,   // payment per period
    pub fv: HpNum,    // future value
    pub begin_mode: bool, // false = END (annuity-immediate), true = BEGIN (annuity-due)
}
```

### XROM Registration Pattern (freeze carve-out)

```rust
// In math1/xrom.rs — bit-3 and bit-4 freeze carve-outs

/// Advantage Pac Section A: ADV CONV + ADV MTRX (XROM 22).
/// id = 22 per HP-41 Advantage Pac hardware module ID.
/// name = "ADV 22A" (CATALOG 2 display — D-43 Claude's discretion; verify from OM).
pub const ADV_MATH_A: XromModule = XromModule {
    id: 22,
    name: "ADV 22A",  // [ASSUMED] — verify against OM 00041-90482 CATALOG 2
    ops: &[
        // ADV CONV (12 ops) + ADV MTRX (~50 ops) — populated with real Op variants
        ("BININ", Op::AdvBinin),
        ("BINVIEW", Op::AdvBinview),
        // ... all 12 CONV + ~50 MTRX entries
    ],
};

/// Advantage Pac Section B: ADV MATH + ADV TVM (XROM 24).
pub const ADV_MATH_B: XromModule = XromModule {
    id: 24,
    name: "ADV 24B",  // [ASSUMED] — verify against OM 00041-90482 CATALOG 2
    ops: &[
        // ADV MATH (~47 ops) + ADV TVM (6 ops)
        ("FSOLVE", Op::AdvFsolve),
        // ... all ~47 MATH + 6 TVM entries
    ],
};

// In xrom_resolve() — bit-3 and bit-4 arms:
if modules & 0b0000_1000 != 0 {
    if let Some(op) = adv_a_resolve(name) {
        return Some(op);
    }
}
if modules & 0b0001_0000 != 0 {
    if let Some(op) = adv_b_resolve(name) {
        return Some(op);
    }
}
```

### ModalProgram Freeze Carve-Out Pattern

```rust
// In math1/modal.rs — fourth additive variant (freeze carve-out)

pub enum ModalProgram {
    // ... existing variants ...
    Stat1(crate::ops::stat1::modal::Stat1Step),
    Time(crate::ops::time::modal::TimeStep),
    /// Advantage Pac modal workflows (Phase 43 — TVM/MATRX/MTR/MEDIT/CMEDIT/VE prompts).
    /// math1/ freeze exception per D-43.X: fourth additive variant following D-33.3b + D-carried.4.
    Advantage(crate::ops::advantage::modal::AdvantageStep),
}

// current_prompt dispatch arm:
ModalProgram::Advantage(step) => crate::ops::advantage::modal::current_prompt(step),
// requires_alpha_label dispatch arm:
ModalProgram::Advantage(step) => crate::ops::advantage::modal::requires_alpha_label(step),
```

### Solver Nesting Architecture (D-43.6 / D-43.7)

The existing `run_loop` re-entrancy infrastructure in `math1/integ.rs` and `math1/solve.rs` provides the template. For one-level nesting (FINTG inside FSOLVE or vice versa), the guard in each solver's run_loop arm checks ONLY its own state field being non-None, not the sibling's:

```rust
// FINTG run_loop arm guard — allows nesting inside FSOLVE:
if state.adv_fintg_state.is_some() {
    return Err(HpError::InvalidOp); // already integrating
}
// Does NOT reject when adv_fsolve_state.is_some() — this is the one-level nesting path

// FSOLVE run_loop arm guard — allows nesting inside FINTG:
if state.adv_fsolve_state.is_some() {
    return Err(HpError::InvalidOp); // already solving
}
```

This is a deliberate deviation from the Math Pac I strict mutual-exclusion guard:
```rust
// Math Pac I INTG run_loop guard (strict — rejects ALL nesting):
if state.integ_state.is_some()
    || state.solve_state.is_some()
    || state.difeq_state.is_some()
{
    return Err(HpError::InvalidOp);
}
```

The Advantage Pac solvers use separate state fields (`adv_fintg_state`, `adv_fsolve_state`, etc.) so they can coexist during one-level nesting without touching Math Pac I state. The call_stack depth limit (4 levels) naturally prevents deeper nesting from compiling successfully.

### Op Variant Naming Convention

Based on existing patterns:
- `Op::TimeTime`, `Op::TimeRunsw` — `Time` prefix for all Time Module variants
- `Op::SigmaBstat`, `Op::SigmaNormdWorkflow` — module-prefix for Stat 1

For Advantage Pac, use `Adv` prefix: `Op::AdvBinin`, `Op::AdvMatdim`, `Op::AdvFroot`, `Op::AdvTvm`, `Op::AdvDot`, etc. This is consistent and prevents shadowing with any future plain-named variants. [ASSUMED — no existing Advantage Pac variants to confirm against; follows established prefix convention]

### Free42 Contamination Guard Extension

The `scripts/check-free42-contamination.sh` script must be extended to scan `hp41-core/src/ops/advantage/`. The existing `for dir in "$MATH1_DIR" "$STAT1_DIR" "$TIME_DIR"` loop expands to include `ADV_DIR="hp41-core/src/ops/advantage"`. The `PATTERN` (21 tokens) does not need new tokens for the Advantage Pac — the existing 21 tokens cover all Free42 identifiers. [VERIFIED: codebase check-free42-contamination.sh]

Every file in `advantage/` MUST carry the verbatim Free42 disclaim header:
```
// Algorithm independently re-derived from HP Advantage Pac Owner's Manual 00041-90482 (1985);
// Free42 source consulted only as sanity-check oracle, not copied.
```

### Anti-Patterns to Avoid

- **Touching `state.matrix_dim` or `state.matrix_active_reg` from advantage/ code:** These are exclusively Math Pac I matrix fields (D-43.5). Any line in `advantage/*.rs` that references either is a bug.
- **Using `floor()`/`fmod()` for decimal-encoded data:** The ISG/DSE and date-parsing precedent applies — use string-split at the decimal point (CLAUDE.md "Core engine" invariant).
- **Adding `#[serde(skip)]` to `adv_tvm_state`:** TVM persistence is D-43.11. The `rand_seed` precedent (`#[serde(default)]` WITHOUT `#[serde(skip)]`) applies here too.
- **Nested Math Pac I solver guards blocking Advantage Pac solvers:** The new `adv_fintg_state`/`adv_fsolve_state` fields are separate from `integ_state`/`solve_state`. Do NOT add Advantage Pac state checks to the Math Pac I solver guards.
- **`_ =>` catch-all in `adv_a_resolve`/`adv_b_resolve`:** Match must be exhaustive — `_ => None` is the terminal arm by Rust default but the XromModule `ops` slice is the authoritative list. Do not add variants to the resolver without adding to the slice.
- **Duplicating `complex_atan2` logic in `advantage/complex_ext.rs`:** Promote `complex_atan2` from `pub(super)` to `pub(crate)` in `math1/complex.rs` (single-line carve-out change) and call it from `advantage/complex_ext.rs`. This is the sanctioned path per CONTEXT.md.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Complex atan2 | Duplicate implementation | `math1::complex::complex_atan2` (promote to `pub(crate)`) | Already solves (0,0) → 0 Pitfall 6; tested |
| Solver cancel flag | Per-solver AtomicBool | Existing `state.cancel_requested` Arc<AtomicBool> | Shared across all solvers; GUI `request_cancel` Tauri command already wires to it |
| Run_loop re-entrancy | New callback mechanism | Existing `run_loop` + transient state field pattern | Used by INTG/SOLVE/DIFEQ; call_stack depth enforces nesting limit |
| HpNum precision | f64 arithmetic on register values | `HpNum` + `rust_decimal` throughout | CLAUDE.md "BCD/f64" invariant; ADR-001 |
| Stack-lift | Manual lift logic | `apply_lift_effect()` + `LiftEffect` enum | Every op declares its lift effect; must use this path |

**Key insight:** Phase 43 builds on seven phases of XROM infrastructure (Phases 28, 33, 34, 38, 39, 41, 42). The plumbing is complete — the work is implementing the algorithms correctly inside the established patterns.

---

## Claude's Discretion Resolutions

The following D-43.X items are Claude's discretion. This section documents the recommended resolution based on codebase patterns and OM 00041-90482 behavioral analysis:

### D-43.2: Maximum Matrix Size Cap
**Recommendation:** `ADV_MATRIX_MAX_ROWS = 255`, `ADV_MATRIX_MAX_COLS = 255`. HP-41 hardware was limited by X-MEM file slots (~8×8 practical), but the emulator extension is unlimited matrices with a per-matrix cap at 255 per dimension. This fits `u8` index fields, keeps `Vec<HpNum>` size bounded at 255×255×8 bytes = ~520 KB max per matrix, and is a documented emulator extension in `hp41-advantage-divergences.md`. [ASSUMED — OM hardware cap was ~8×8; emulator extension value chosen for practical use]

### D-43.3: Current-Matrix Selection Mechanism
**Recommendation:** ALPHA register names the current matrix. When MATDIM/MR/MS/MEDIT ops fire, they read `state.alpha_reg` as the matrix name. This matches how the HP-41 Advantage Pac actually works — the ALPHA register is the "matrix name pointer" per OM §2. A dedicated `adv_current_matrix: Option<String>` transient field caches the last-activated name for ops like MNAME? and I+/J+ (which operate on the "current matrix"). [ASSUMED — OM 00041-90482 §2 "Matrix Functions" describes ALPHA-register naming; verify before commit]

### D-43.4: I/J Index Storage
**Recommendation:** Global I/J indices on CalcState (`adv_matrix_i: u8`, `adv_matrix_j: u8`, both `#[serde(default)]`). The OM describes I and J as single global "matrix index registers" — not per-matrix. This is consistent with how the HP-41 hardware stores I/J as shared registers. All matrix element access ops (MR, MS, MRC+, etc.) read/write the global I/J. [ASSUMED — OM 00041-90482 §2 describes I/J as single registers; verify]

### D-43.6: Callback Mechanism Architecture
**Recommendation:** Reuse existing `run_loop` re-entrancy with new transient state fields. Create `adv_froot_state: Option<FrootState>`, `adv_fintg_state: Option<AdvFintegState>`, `adv_fsolve_state: Option<AdvFsolveState>`, `adv_fdifeq_state: Option<AdvFdifeqState>` on CalcState (all `#[serde(default, skip)]`). The nesting architecture (see Solver Nesting section) uses separate state fields to allow one-level cross-nesting. [VERIFIED: codebase — this is exactly how INTG/SOLVE/DIFEQ work]

### D-43.8: FROOT Calling Convention
**Recommendation:** Degree from X register (integer, popped off stack), coefficients in registers R01..Rn (highest-degree first). FROOT reads degree from X, then reads coefficients from R01 through R(degree+1) using the same register-convention approach as Math Pac I POLY. This avoids a degree-prompt modal which would conflict with the typical usage pattern (compute roots, retrieve via RTS). Display results via print_buffer. [ASSUMED — OM 00041-90482 §3 "FROOT" likely describes degree-in-X + coeff-in-registers; must verify against OM before implementing]

### D-43.12: BEGIN/END Payment Mode
**Recommendation:** Implement both modes. `TvmState.begin_mode: bool` (false = END, default). END is the standard annuity-immediate mode; BEGIN (annuity-due) shifts all payments by one period. Document whether the OM supports both modes — if OM supports only END mode, document BEGIN as an emulator extension. [ASSUMED — standard TVM implementations support both; verify OM §4]

### D-43.13: *I Non-Convergence Handling
**Recommendation:** After 100 Newton iterations without convergence, return `HpError::NoRoot` (reuse existing error) and push the last iterate into X. Print "NO SOLUTION" via print_buffer. Mirror SOLVE's "NO ROOT FOUND" pattern. [ASSUMED — consistent with existing solver error handling precedents in the codebase]

---

## Common Pitfalls

### Pitfall 1: Matrix Register Layout Confusion
**What goes wrong:** Advantage Pac matrix code accidentally reads or writes `state.matrix_dim` or `state.matrix_active_reg` — the Math Pac I register-indexed matrix fields.
**Why it happens:** Both systems are called "matrix"; the `matrix_dim`/`matrix_active_reg` fields are visible everywhere in `state.rs`.
**How to avoid:** D-43.5 mandates complete isolation. Never reference `matrix_dim` or `matrix_active_reg` from `advantage/`.
**Warning signs:** `grep -rn "matrix_dim\|matrix_active_reg" hp41-core/src/ops/advantage/` returns any results.

### Pitfall 2: Solver State Mutual Exclusion Over-Restriction
**What goes wrong:** Copying Math Pac I's strict guard (`integ_state.is_some() || solve_state.is_some() || difeq_state.is_some()`) into Advantage Pac solver guards, blocking FINTG-inside-FSOLVE nesting.
**Why it happens:** The Math Pac I guards are correct for Math Pac I (no nesting specified). D-43.7 requires one level of Advantage cross-nesting.
**How to avoid:** Each Advantage solver only guards its OWN state field being non-None. See Solver Nesting Architecture section.
**Warning signs:** `XEQ "FSOLVE"` + callback that runs `XEQ "FINTG"` returns `InvalidOp`.

### Pitfall 3: complex_atan2 Duplication
**What goes wrong:** Implementing a new `complex_atan2` in `advantage/complex_ext.rs` instead of promoting the existing one.
**Why it happens:** `math1/complex::complex_atan2` is currently `pub(super)` — not visible outside math1/.
**How to avoid:** Change `pub(super)` to `pub(crate)` in `math1/complex.rs` (single-line freeze carve-out). Then call `crate::ops::math1::complex::complex_atan2()` from advantage code.
**Warning signs:** Duplicate `fn complex_atan2` in `advantage/`.

### Pitfall 4: serde(skip) on adv_tvm_state
**What goes wrong:** Adding `#[serde(default, skip)]` to `adv_tvm_state` — losing TVM state across sessions.
**Why it happens:** All other solver state fields (`integ_state`, `solve_state`, etc.) use `skip`. D-43.11 makes TVM state the exception (persistent).
**How to avoid:** `#[serde(default)]` ONLY, no `skip`. Same contract as `rand_seed` (D-33.4). Document in the field comment with the same "mirror pattern" warning.
**Warning signs:** TVM state disappears after save/reload.

### Pitfall 5: INTG mnemonic vs FINTG shadowing
**What goes wrong:** Registering "INTG" in `ADV_MATH_B.ops` (shadowing Math Pac I's INTG).
**Why it happens:** The Advantage Pac integrator is a different function (Romberg vs Simpson) but might be called "INTG" in some references.
**How to avoid:** Advantage Pac uses "FINTG" (not "INTG"). `xrom_shadowing.rs` CI gate will catch any collision with `MATH_1.ops`. Register "FINTG", "FSOLVE", "FROOT", "FDIFEQ" — all have the "F" prefix in the Advantage Pac OM.
**Warning signs:** `xrom_shadowing.rs` CI failure.

### Pitfall 6: Omitting Free42 Disclaim Header
**What goes wrong:** A `advantage/*.rs` file lacks the verbatim Free42 disclaim header — the contamination guard cannot distinguish it.
**Why it happens:** The header is a project convention, not enforced by the compiler.
**How to avoid:** Every file under `advantage/` must start with the two-line disclaim. Add the ADV_DIR check to `check-free42-contamination.sh`.
**Warning signs:** `just license-audit` would fail if any distinctive token appears without the disclaim.

### Pitfall 7: 36-bit mask value error
**What goes wrong:** Using `0xFFFFFFFFF` (9 hex F = 36 bits) or `2^36 - 1` computed wrong.
**Why it happens:** Off-by-one in hex digit counting.
**How to avoid:** `const ADV_WORD_MASK: u64 = 0x0000_000F_FFFF_FFFF;` — that is 1+8 hex digits = 36 bits. Verify: `u64::MAX & 0x0000_000F_FFFF_FFFF == 68_719_476_735`. Write a unit test: `assert_eq!(ADV_WORD_MASK.count_ones(), 36)`.
**Warning signs:** ROTXY or NOT producing values > 2^36.

### Pitfall 8: xrom_resolve bit-3/bit-4 mask confusion
**What goes wrong:** Using `0b0001_0000` for bit 3 (which is actually bit 4).
**Why it happens:** Bit numbering: bit 0 = 0b0000_0001, bit 1 = 0b0000_0010, bit 2 = 0b0000_0100, **bit 3 = 0b0000_1000**, **bit 4 = 0b0001_0000**.
**How to avoid:** `ADV_MATH_A` = bit 3 = `0b0000_1000`; `ADV_MATH_B` = bit 4 = `0b0001_0000`. `default_xrom_modules() = 0b0001_1111` (bits 0–4 all set).
**Warning signs:** `xrom_chain_order.rs` test failures; Advantage ops resolve with wrong module bits set.

---

## Code Examples

### xrom_resolve Extension Pattern
```rust
// Source: hp41-core/src/ops/math1/xrom.rs (existing code, verified)

pub fn xrom_resolve(name: &str, modules: u8) -> Option<Op> {
    // ... existing bit-0, bit-1, bit-2 arms ...
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
[VERIFIED: codebase — exact pattern from existing bit-0/1/2 arms]

### migrate_after_load Extension Pattern
```rust
// Source: hp41-core/src/state.rs (existing code, verified)
pub fn migrate_after_load(&mut self) {
    // v3.0 → v3.1: bit-1
    if self.xrom_modules & 0b0000_0010 == 0 { self.xrom_modules |= 0b0000_0010; }
    // v3.1 → v3.2: bit-2
    if self.xrom_modules & 0b0000_0100 == 0 { self.xrom_modules |= 0b0000_0100; }
    // Phase 43 v3.2 → v3.3: bits 3+4 (ADV_MATH_A + ADV_MATH_B)
    if self.xrom_modules & 0b0000_1000 == 0 { self.xrom_modules |= 0b0000_1000; }
    if self.xrom_modules & 0b0001_0000 == 0 { self.xrom_modules |= 0b0001_0000; }
    // ... existing stopwatch freeze-on-load logic ...
}
```
[VERIFIED: codebase — exact idempotent migration pattern]

### ADV CONV: BININ Implementation Pattern
```rust
// Source: advantage/conv.rs (new file, pattern derived from codebase conventions)
// Free42 source consulted only as sanity-check oracle, not copied.

pub fn op_adv_binin(state: &mut CalcState) -> Result<(), HpError> {
    // Read binary string from ALPHA register, convert to integer
    let s = state.alpha_reg.trim().to_string();
    if s.is_empty() {
        return Err(HpError::Domain);
    }
    let value = u64::from_str_radix(&s, 2).map_err(|_| HpError::Domain)?;
    let masked = value & ADV_WORD_MASK;  // 36-bit silent truncation (D-43.10)
    let result = HpNum::from_u64(masked).ok_or(HpError::Domain)?;
    apply_lift_effect(state, LiftEffect::Enable);
    state.stack.x = result;
    Ok(())
}
```

### Laguerre Polynomial Root Algorithm Pattern
```rust
// Source: advantage/solvers.rs (new file, algorithm from Numerical Recipes §9.5)
// Laguerre's method for polynomial root p(x) = 0
// Input: coefficients c[0..=n] (c[0] = leading, c[n] = constant)
// Returns: one complex root or error

fn laguerre_step(
    coeffs: &[f64],   // polynomial coefficients, degree n = coeffs.len()-1
    x: (f64, f64),    // complex initial guess (re, im)
) -> Result<(f64, f64), HpError> {
    let n = (coeffs.len() - 1) as f64;
    // Evaluate P(x), P'(x), P''(x) using Horner's method
    // G = P'(x) / P(x), H = G^2 - P''(x)/P(x)
    // a = n / (G ± sqrt((n-1)(nH - G^2)))
    // Choose sign to maximize |denominator|
    // ...
}
```
[ASSUMED — Laguerre algorithm from Numerical Recipes §9.5, primary literature]

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Math Pac I POLY/ROOTS (Bairstow, degree 2-5) | ADV FROOT (Laguerre, arbitrary degree) | Phase 43 | Both coexist; different XROM modules |
| Math Pac I INTG (Simpson, R00-R07 scratch) | ADV FINTG (Romberg, separate state) | Phase 43 | Both coexist; different op names |
| Math Pac I SOLVE (secant method, R00-R01 scratch) | ADV FSOLVE (Brent/secant, separate state) | Phase 43 | Both coexist; different op names |
| No named matrices (only R14/R15+ register blocks) | `adv_matrices: Vec<AdvMatrix>` with ALPHA names | Phase 43 | Two matrix systems coexist; complete isolation |

**Deprecated/outdated:**
- Nothing deprecated — the Advantage Pac is additive. Math Pac I matrix/solver ops remain fully functional alongside new ADV variants.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | ADV_MATH_A CATALOG 2 name is "ADV 22A" | XROM Registration Pattern | Wrong display in CATALOG 2; cosmetic only |
| A2 | ADV_MATH_B CATALOG 2 name is "ADV 24B" | XROM Registration Pattern | Wrong display in CATALOG 2; cosmetic only |
| A3 | FROOT reads degree from X register (not modal prompt) | D-43.8 resolution | Wrong calling convention; behavior mismatch |
| A4 | I/J are global registers (not per-matrix) | D-43.4 resolution | All element access ops read wrong index |
| A5 | ALPHA register names the current matrix | D-43.3 resolution | All ADVMTRX ops use wrong selection mechanism |
| A6 | ADV_MATRIX_MAX_ROWS/COLS = 255 | D-43.2 resolution | Cap too high (memory) or too low (incompatible) |
| A7 | TVM BEGIN mode stored as bool on TvmState | D-43.12 resolution | Missing mode support if OM requires it |
| A8 | *I non-convergence → HpError::NoRoot + "NO SOLUTION" in print_buffer | D-43.13 resolution | Wrong error signal or missing user feedback |
| A9 | `Op::Adv` prefix for all 117 Advantage Pac variant names | Op Naming Convention | Name collisions; shadowing in xrom_shadowing |
| A10 | Advantage Pac XROM 22 contains ADV CONV + ADV MTRX (id=22) | ADV-FW-01 | Wrong ops in wrong XROM module |
| A11 | Advantage Pac XROM 24 contains ADV MATH + ADV TVM (id=24) | ADV-FW-01 | Wrong ops in wrong XROM module |
| A12 | FINTG uses Romberg (not repeated Simpson like INTG) | solvers.rs | Wrong algorithm choice; accuracy difference |
| A13 | Complex matrix storage interleaves real/imag per element (not separate arrays) | AdvMatrix struct | All complex matrix ops compute wrong indices |

**Items A10/A11 are medium-risk:** The XROM module assignment is fixed hardware and should be verifiable from OM 00041-90482 or the HP-41 Module Database. Confirm before writing the XromModule constants. If the split is wrong (e.g., all ~117 ops in one XROM), the CATALOG 2 display strings and function-ID numbering will be off.

---

## Open Questions

1. **CATALOG 2 display strings for XROM 22 and XROM 24**
   - What we know: XROM IDs 22 and 24 confirmed from research SUMMARY.md; TIME_MODULE uses "TIME 2C"
   - What's unclear: Exact strings HP displayed — "ADV 22A"/"ADV 24B" is plausible but unverified
   - Recommendation: Check OM 00041-90482 cover or HP-41 Module Database (calc.fjk.ch/db/hp41mod.php); document as ASSUMED until verified

2. **Complete FROOT entry convention**
   - What we know: OM §3 describes FROOT for polynomial roots; Laguerre algorithm is the correct choice (D-43.8 mentions "OM 00041-90482 Section 3 conventions")
   - What's unclear: Whether degree comes from X register, a modal prompt, or is implicit in the coefficient register count
   - Recommendation: Implement X-register convention (A3) as default; if OM says otherwise, change before Phase 44

3. **ADV MTRX op count: 50 or 52?**
   - What we know: REQUIREMENTS.md lists ADV-MTX-01..50 (50 ops); CONTEXT.md says "~52"; SUMMARY.md says "~52"
   - What's unclear: Whether 50 or 52 is the canonical count; 2 ops may be undocumented
   - Recommendation: Implement all 50 ops from REQUIREMENTS.md; if OM lists additional ops, add them

4. **FDIFEQ algorithm — 1st order RK4 or otherwise?**
   - What we know: REQUIREMENTS.md ADV-MATH-05 says "1st/2nd order RK4"; DIFEQ (Math Pac I) also uses RK4
   - What's unclear: Whether FDIFEQ requires the same multi-step interface as DIFEQ (ORDER=? prompt) or differs
   - Recommendation: Mirror DIFEQ architecture but use separate state field; confirm from OM §3

5. **TVM *I uniqueness requirement**
   - What we know: Newton iteration on TVM equation can find multiple interest rates (negative rates, etc.)
   - What's unclear: Whether OM specifies which root to return when multiple exist
   - Recommendation: Return the first convergent root > -100% and document in divergences file

---

## Environment Availability

Step 2.6: No new external tool dependencies. Phase 43 is pure Rust code changes. The existing build infrastructure (`just`, `cargo`, `cargo-llvm-cov`) is unchanged.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable / MSRV 1.88 | All Rust compilation | [ASSUMED: yes] | 1.88+ | — |
| just | Task runner | [ASSUMED: yes] | current | — |
| cargo-llvm-cov | Coverage gate | [ASSUMED: yes] | current | — |

---

## Validation Architecture

**Nyquist validation enabled** (`workflow.nyquist_validation: true` in `.planning/config.json`).

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) + cargo-llvm-cov |
| Config file | Cargo.toml workspace |
| Quick run command | `just test` (or `cargo test -p hp41-core`) |
| Full suite command | `just coverage` (cargo-llvm-cov with ≥95% line / ≥93% region gate) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ADV-FW-01 | ADV_MATH_A.id==22, ADV_MATH_B.id==24 | unit | `cargo test -p hp41-core adv_module_const_id_and_name` | ❌ Wave 0 |
| ADV-FW-02 | bit-3/bit-4 resolve correctly | unit | `cargo test -p hp41-core resolve_uses_bit_3_for_adv_a` | ❌ Wave 0 |
| ADV-FW-03 | migrate_after_load sets bits 3+4 | unit | `cargo test -p hp41-core migrate_sets_adv_bits` | ❌ Wave 0 |
| ADV-FW-04 | adv_matrices default is empty Vec | unit | `cargo test -p hp41-core adv_matrices_default_empty` | ❌ Wave 0 |
| ADV-FW-05 | ModalProgram::Advantage dispatches current_prompt | unit | `cargo test -p hp41-core advantage_modal_dispatch` | ❌ Wave 0 |
| ADV-FW-06 | All Op variants compile (implicit: `cargo build`) | compile | `cargo build -p hp41-core` | implicit |
| ADV-CONV-01 | BININ converts binary string | unit | `cargo test -p hp41-core adv_binin_basic` | ❌ Wave 0 |
| ADV-CONV-07 | NOT masks to 36 bits | unit | `cargo test -p hp41-core adv_not_36bit_mask` | ❌ Wave 0 |
| ADV-MTX-17 | MATDIM creates named matrix | unit | `cargo test -p hp41-core adv_matdim_creates_matrix` | ❌ Wave 0 |
| ADV-MTX-05/06 | MR/MS read and write matrix elements | unit | `cargo test -p hp41-core adv_mr_ms_roundtrip` | ❌ Wave 0 |
| ADV-MTX-20 | MDET computes determinant via LU | unit | `cargo test -p hp41-core adv_mdet_basic` | ❌ Wave 0 |
| ADV-MATH-03 | FSOLVE finds root via run_loop callback | integration | `cargo test -p hp41-core adv_fsolve_callback` | ❌ Wave 0 |
| ADV-MATH-04 | FINTG integrates via run_loop callback | integration | `cargo test -p hp41-core adv_fintg_callback` | ❌ Wave 0 |
| ADV-MATH-04+03 | FINTG inside FSOLVE (one-level nesting) | integration | `cargo test -p hp41-core adv_fintg_inside_fsolve` | ❌ Wave 0 |
| ADV-MATH-06 | FROOT finds roots of polynomial | unit | `cargo test -p hp41-core adv_froot_quadratic` | ❌ Wave 0 |
| ADV-TVM-06 | *I converges via Newton | unit | `cargo test -p hp41-core adv_tvm_star_i_convergence` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p hp41-core` (full unit suite, ~30s)
- **Per wave merge:** `just coverage` (llvm-cov ≥95% line / ≥93% region gate)
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `hp41-core/src/ops/advantage/mod.rs` — module file + AdvMatrix struct + ADV_MATRIX_MAX constants
- [ ] `hp41-core/src/ops/advantage/modal.rs` — AdvantageStep enum + current_prompt() + requires_alpha_label()
- [ ] `hp41-core/src/ops/advantage/conv.rs` — 12 ADV CONV ops
- [ ] `hp41-core/src/ops/advantage/matrix_ops.rs` — element access + lifecycle + reductions
- [ ] `hp41-core/src/ops/advantage/matrix_linalg.rs` — high-level matrix ops (LU-based)
- [ ] `hp41-core/src/ops/advantage/matrix_complex.rs` — complex matrix ops
- [ ] `hp41-core/src/ops/advantage/complex_ext.rs` — complex number extensions
- [ ] `hp41-core/src/ops/advantage/solvers.rs` — FSOLVE/FINTG/FDIFEQ/FROOT + solver states
- [ ] `hp41-core/src/ops/advantage/poly.rs` — PLY/RTS
- [ ] `hp41-core/src/ops/advantage/matrix_workflow.rs` — MATRX/MTR frontends + AIP
- [ ] `hp41-core/src/ops/advantage/curve_fit.rs` — CFIT/AS/DS/BFIT/FIT/Y?X/SZ?
- [ ] `hp41-core/src/ops/advantage/vectors.rs` — vector ops + TR
- [ ] `hp41-core/src/ops/advantage/tvm.rs` — TVM/N/PV/PMT/FV/*I + TvmState

---

## Security Domain

`security_enforcement` is not set to `false`. This is a desktop calculator emulator with no network access, no authentication, no user-facing input validation beyond HP-41 register semantics. ASVS categories are not applicable — no user authentication, no sessions, no external network, no cryptography, no SQL.

| ASVS Category | Applies | Rationale |
|---------------|---------|-----------|
| V2 Authentication | no | Standalone desktop app; no user accounts |
| V3 Session Management | no | No sessions |
| V4 Access Control | no | No multi-user access |
| V5 Input Validation | partial | Validate matrix dimension inputs (u8 bounds), register indices; return `HpError::Domain` on invalid |
| V6 Cryptography | no | No crypto anywhere in project |

**Relevant security consideration for Phase 43:** Matrix dimension inputs from MATDIM (Y=rows, X=cols) must be validated against `ADV_MATRIX_MAX_ROWS/COLS` before allocation to prevent oversized Vec allocation. Use `HpError::Domain` consistent with existing register-range validation patterns.

---

## Sources

### Primary (HIGH confidence)
- `hp41-core/src/ops/math1/xrom.rs` — XromModule struct, xrom_resolve pattern, existing module constants [VERIFIED: codebase]
- `hp41-core/src/ops/math1/modal.rs` — ModalProgram enum, freeze carve-out pattern, Stat1/Time precedents [VERIFIED: codebase]
- `hp41-core/src/state.rs` — CalcState field serde shapes, migrate_after_load pattern, default_xrom_modules() [VERIFIED: codebase]
- `hp41-core/src/ops/math1/integ.rs` — IntegState, run_loop re-entrancy, user callback pattern [VERIFIED: codebase]
- `hp41-core/src/ops/math1/solve.rs` — SolveState, secant method, non-convergence handling [VERIFIED: codebase]
- `.planning/phases/43-hp41-core-xrom-framework-all-advantage-pac-ops/43-CONTEXT.md` — locked decisions [VERIFIED: codebase]
- `.planning/REQUIREMENTS.md` — ADV-FW-01..06, ADV-CONV-01..12, ADV-MTX-01..50, ADV-MATH-01..47, ADV-TVM-01..06 [VERIFIED: codebase]
- `.planning/research/SUMMARY.md` — XROM IDs confirmed (22+24), algorithm choices, zero-new-runtime-deps [VERIFIED: codebase]
- `scripts/check-free42-contamination.sh` — 21-token contamination guard, directory extension pattern [VERIFIED: codebase]
- `hp41-core/tests/xrom_op_test_count.rs` — unified meta-gate extension pattern for v3.3 [VERIFIED: codebase]
- `CLAUDE.md` — all frozen invariants (4-way exhaustive match, serde patterns, MSRV, no-async/no-panic) [VERIFIED: codebase]

### Secondary (MEDIUM confidence)
- `.planning/STATE.md` — pre-resolved decisions (XROM IDs, algorithm choices, migration path) [VERIFIED: codebase]
- `hp41-core/src/ops/stat1/` — organizational template for `advantage/` module structure [VERIFIED: codebase]
- `hp41-core/src/ops/time/` — second organizational template (simpler: less modal complexity) [VERIFIED: codebase]

### Tertiary (LOW confidence / ASSUMED)
- CATALOG 2 display strings for XROM 22 + 24 — not found in codebase; assumed from naming convention
- FROOT calling convention — assumed X-register for degree; verify against OM 00041-90482 §3
- OM 00041-90482 §2 I/J global register semantics — assumed global; verify from manual
- BEGIN/END TVM mode from OM §4 — assumed supported; verify from manual

---

## Metadata

**Confidence breakdown:**
- XROM framework registration: HIGH — exact pattern established by three prior modules
- Modal freeze carve-out: HIGH — fourth additive variant, exact code path verified
- CalcState serde patterns: HIGH — locked invariants in CLAUDE.md; every case verified in state.rs
- Solver nesting architecture: HIGH — design derived from existing INTG/SOLVE patterns; single-level nesting clearly separable via distinct state fields
- ADV CONV implementation: HIGH — pure integer arithmetic; 36-bit mask is unambiguous
- ADV MTRX element access: HIGH — I/J index model is clear once D-43.4 is resolved
- ADV MTRX high-level (LU decomp, etc.): MEDIUM — well-known algorithms but ~150 LOC each; correctness requires oracle test cases
- ADV MATH complex extensions: HIGH — pattern established by Math Pac I complex ops; complex_atan2 already exists
- ADV MATH solvers (FROOT/FINTG/FSOLVE): MEDIUM — algorithms known; OM calling conventions for FROOT need verification (A3)
- ADV TVM: MEDIUM-HIGH — Newton for *I is standard; BEGIN/END mode (D-43.12) needs OM confirmation
- CATALOG 2 display strings: LOW — assumed; must verify from OM or HP-41 module database

**Research date:** 2026-05-25
**Valid until:** 2026-06-25 (stable domain — no external library changes; only OM verification gaps)
