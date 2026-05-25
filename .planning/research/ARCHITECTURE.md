# Architecture Patterns: HP-41 Advantage Pac + Advanced Matrix Pac Emulation (v3.3)

**Domain:** HP-41 Advantage Pac (HP 00041-90546, XROM 22+24) and community "Advanced Matrix Pac" extension
**Researched:** 2026-05-25
**Confidence:** MEDIUM-HIGH (codebase read in full; Advantage Pac XROM IDs confirmed from calc.fjk.ch; function categories confirmed from multiple community sources; exact per-function XROM number sub-assignments MEDIUM confidence — OM PDF not fully parsed)

---

## Critical Context: What These Modules Actually Are

### HP Advantage Pac (official HP product, HP 00041-90546)

The Advantage Pac spans TWO hardware ROM pages: XROM 22 and XROM 24. It contains ~117 functions under four section headers:

| Section Header | Count | Contents |
|---|---|---|
| `-ADV CONV` | 12 | Base conversion (BIN/OCT/HEX input/view/convert) + Boolean (NOT, AND, OR, XOR, ROTXY, BIT?) |
| `-ADV MTRX` | 52 | Matrix operations (M-code, based on CCD ROM): M*M, MAT*, MAT+, MAT-, MAT/, MATDIM, MDET, MINV, MMOVE, MNAME?, MR, MRC+, MRC-, MRIJ, MRR+, MRR-, MS, MSC+, MSIJ, MSR+, MSWAP, MSYS, and ~30 more row/col/vector ops including I+, I-, J+, J-, V+, VDOT, IDN, FNRM, CNRM, CSUM, DIM?, CMAXAB, MAX, MAXAB, MIN, C<>C |
| `-ADV MATH` | 47 | Complex ops (CABS, CARG, CCHS, CCONJ, CY^X, + complex stack from HP-15C heritage), FROOT (arbitrary-degree polynomial roots, Romberg-based), FINTG (enhanced numerical integration), SOLVE, INTEG plus complex function stack overlay similar to Math Pac I |
| `-ADV TVM` | 6 | Time Value of Money (N, I%YR, PV, PMT, FV, CMPD) |

XROM 22 holds the ADV CONV + ADV MTRX functions. XROM 24 holds ADV MATH + ADV TVM functions. The Advantage Pac "inherits" the Math Pac I complex stack approach but adds more complex number ops (CABS, CARG, CCHS, CCONJ, CY^X) and uses a Romberg algorithm for FROOT (arbitrary-degree polynomial roots, unlike Math Pac I's POLY which is degree 2-5 only).

### "Advanced Matrix Pac" (community extension, NOT an official HP product)

Based on research: the "Advanced Matrix Pac" referenced in PROJECT.md is a community-created ROM extension (associated with Ángel Martin and the hp41.org community) that extends the Advantage Pac's matrix capabilities. It is NOT a separate official HP module with its own XROM ID. The community description states it includes "all the ADV MATRIX functions from the Advantage Pac, all Matrix Functions and Programs from the ALGEBRA module, a new Matrix Input mode for fast data entry, all Binary Conversion Functions, and keeps SOLVE and INTEG."

**Architectural implication:** "Advanced Matrix Pac" functions most likely share XROM 22/24 space (same module IDs as Advantage Pac) or represent a superset of the Advantage Pac's matrix section. For the emulator, both are best treated as ONE module milestone with XROM 22 + XROM 24 as the canonical IDs.

---

## The Central Architectural Challenge

The Advantage Pac creates THREE distinct integration concerns relative to the existing codebase:

1. **Complex ops overlap with Math Pac I** — CABS, CARG, CCHS, CCONJ, CY^X operate on the same complex stack overlay (`complex_mode: bool`, X+iY = ζ, Z+iT = τ) that Math Pac I already owns. These are ADDITIVE functions on the existing complex stack, not a new complex system.

2. **FROOT vs POLY: different algorithms, same output convention** — FROOT in the Advantage Pac supports arbitrary-degree polynomial roots (not just degree 2-5 like Math Pac I's POLY) using a Romberg-based algorithm. It uses the same `U=u / V=v` print-buffer output convention. The existing `math1/poly.rs` Bairstow implementation is FROZEN — FROOT goes in the new `advantage/` directory, NOT in `math1/`.

3. **ADV MTRX matrix ops are M-code (machine code), one-shot stack ops** — unlike Math Pac I's MATRIX which is a modal multi-step workflow. The ADV MTRX ops (MAT*, MSYS, etc.) are one-shot: arguments on stack/registers, result returned immediately. No new `ModalProgram` variant needed for these. FROOT/FINTG still need modals for their prompt sequences.

4. **ADV TVM requires user-callback-style iteration** — FINTG's enhanced integration and FROOT's Romberg solver both invoke user-defined function labels (same pattern as Math Pac I's INTG/SOLVE/DIFEQ re-entrancy infrastructure).

---

## Recommended Architecture

### New Directory Structure

```
hp41-core/src/ops/
├── math1/          # FROZEN (XROM 7) — except xrom.rs + modal.rs carve-outs
├── stat1/          # (XROM 2, v3.1)
├── time/           # (XROM 26, v3.2)
└── advantage/      # NEW (XROM 22 + XROM 24, v3.3)
    ├── mod.rs      # Module root + named consts (register layout, output format)
    ├── xrom.rs     # ADV_CONV + ADV_MTRX + ADV_MATH + ADV_TVM module registries
    ├── modal.rs    # AdvantageStep enum for FROOT/FINTG prompt sequences
    ├── conv.rs     # ADV CONV: base conversion + boolean (12 ops)
    ├── matrix.rs   # ADV MTRX: one-shot matrix ops (52 ops)
    ├── complex.rs  # ADV MATH complex ops: CABS, CARG, CCHS, CCONJ, CY^X (5 ops)
    ├── froot.rs    # FROOT: Romberg arbitrary-degree polynomial root finder
    ├── fintg.rs    # FINTG: enhanced numerical integration (re-uses user-callback infra)
    ├── tvm.rs      # ADV TVM: N, I%YR, PV, PMT, FV, CMPD (6 ops)
    └── solve_intg.rs # ADV SOLVE + ADV INTEG stubs (re-use math1 infra if possible)
```

**Why separate `advantage/` directory, not extending `math1/`:**
- `math1/` is FROZEN since Plan 25-01 (with two sanctioned carve-outs: `xrom.rs` + `modal.rs`)
- Advantage Pac is a distinct HP product with separate XROM IDs (22/24 vs 7)
- Following the established pattern: `stat1/` is separate from `math1/`, `time/` is separate from both
- The `advantage/` directory carries the same Free42 disclaim header verbatim on every file

### Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| `hp41-core/src/ops/advantage/` | All ~117 Advantage Pac + Advanced Matrix Pac ops; pure functions | `CalcState` (owned), user-callback re-entrancy via `run_loop` |
| `hp41-core/src/ops/math1/xrom.rs` | `ADV_MATH_A` (XROM 22) + `ADV_MATH_B` (XROM 24) module registries; `advantage_resolve()` + bits 3+4 in `xrom_resolve()` | `xrom_resolve()` caller chain |
| `hp41-core/src/ops/math1/modal.rs` | `ModalProgram::Advantage(AdvantageStep)` variant — fourth carve-out of frozen modal.rs | `modal_prompt` / `modal_program` channel |
| `hp41-core/src/state.rs` | 2-3 new persistent CalcState fields (xrom_modules bits 3+4, TVM solver state) | All ops via `&mut CalcState` |

---

## XROM Registration: Bits 3 and 4

### Bitmask Extension

```rust
// In state.rs:
fn default_xrom_modules() -> u8 {
    0b0001_1111  // bits 0-4: Math1(0) + Stat1(1) + Time(2) + AdvConv/Mtrx(3) + AdvMath/Tvm(4)
}
```

The Advantage Pac spans two hardware ROM pages (XROM 22 and XROM 24). These map to two bitmask bits:
- **Bit 3** = XROM 22 (`ADV_MATH_A`): ADV CONV + ADV MTRX section (12 + 52 ops)
- **Bit 4** = XROM 24 (`ADV_MATH_B`): ADV MATH + ADV TVM section (47 + 6 ops)

In practice, both bits are always set or always clear together (the Advantage Pac is one physical module). The split mirrors the hardware reality that XROM 22 and XROM 24 are two ROM pages of the same physical module. On the HP-41, plugging the module loads both pages simultaneously.

### Migration in `migrate_after_load()`

```rust
// v3.2 → v3.3: set bits 3+4 (Advantage Pac pages A+B)
if self.xrom_modules & 0b0000_1000 == 0 {
    self.xrom_modules |= 0b0000_1000;
}
if self.xrom_modules & 0b0001_0000 == 0 {
    self.xrom_modules |= 0b0001_0000;
}
```

### XromModule Registry Additions in `math1/xrom.rs`

Following the established carve-out pattern (ADR-v3.1-004, D-33.3, D-38.X), two new module constants go in `math1/xrom.rs` and two new resolver arms go in `xrom_resolve()`:

```rust
// Fourth freeze exception — Advantage Pac page A (XROM 22)
pub const ADV_MATH_A: XromModule = XromModule {
    id: 22,
    name: "ADV CONV A",  // CATALOG 2 display string — to verify against OM
    ops: &[
        // ADV CONV (12 ops): base conversion + boolean
        ("BININ", Op::AdvBinin), ("BINVIEW", Op::AdvBinview),
        ("OCTIN", Op::AdvOctin), ("HEXIN", Op::AdvHexin),
        ("HEXVIEW", Op::AdvHexview), ("CVTVIEW", Op::AdvCvtview),
        ("NOT", Op::AdvNot), ("AND", Op::AdvAnd),
        ("OR", Op::AdvOr), ("XOR", Op::AdvXor),
        ("ROTXY", Op::AdvRotxy), ("BIT?", Op::AdvBitQ),
        // ADV MTRX (52 ops): matrix + vector ops
        ("M*M", Op::AdvMtimesM), ("MAT*", Op::AdvMatTimes),
        ("MAT+", Op::AdvMatPlus), ("MAT-", Op::AdvMatMinus),
        ("MAT/", Op::AdvMatDiv), ("MATDIM", Op::AdvMatdim),
        ("MDET", Op::AdvMdet), ("MINV", Op::AdvMinv),
        ("MMOVE", Op::AdvMmove), ("MNAME?", Op::AdvMnameQ),
        ("MSYS", Op::AdvMsys), ("IDN", Op::AdvIdn),
        ("V+", Op::AdvVplus), ("VDOT", Op::AdvVdot),
        ("FNRM", Op::AdvFnrm), ("CNRM", Op::AdvCnrm),
        ("CSUM", Op::AdvCsum), ("DIM?", Op::AdvDimQ),
        ("CMAXAB", Op::AdvCmaxab), ("MAX", Op::AdvMax),
        ("MAXAB", Op::AdvMaxab), ("MIN", Op::AdvMin),
        ("C<>C", Op::AdvCswapC), ("I+", Op::AdvIplus),
        ("I-", Op::AdvIminus), ("J+", Op::AdvJplus),
        ("J-", Op::AdvJminus),
        // ... remaining ~23 matrix row/col ops
    ],
};

// Fifth freeze exception — Advantage Pac page B (XROM 24)
pub const ADV_MATH_B: XromModule = XromModule {
    id: 24,
    name: "ADV MATH B",  // CATALOG 2 display string — to verify against OM
    ops: &[
        // ADV MATH complex ops (5 new complex ops)
        ("CABS", Op::AdvCabs), ("CARG", Op::AdvCarg),
        ("CCHS", Op::AdvCchs), ("CCONJ", Op::AdvCconj),
        ("CY^X", Op::AdvCpowYX),
        // ADV MATH numeric solvers
        ("FROOT", Op::AdvFrootWorkflow),
        ("FINTG", Op::AdvFintgWorkflow),
        ("SOLVE", Op::AdvSolve),   // possibly reuse Math1 SOLVE or separate
        ("INTEG", Op::AdvInteg),   // possibly reuse Math1 INTG or separate
        // ADV TVM (6 ops)
        ("N", Op::AdvTvmN), ("I%YR", Op::AdvTvmI),
        ("PV", Op::AdvTvmPV), ("PMT", Op::AdvTvmPMT),
        ("FV", Op::AdvTvmFV), ("CMPD", Op::AdvTvmCmpd),
    ],
};
```

**Op naming convention:** `Adv` prefix on all new Op variants prevents shadowing with existing builtins and with the math1/stat1/time variants. Example: `Op::AdvCabs` not `Op::Cabs` (which would conflict if Math Pac I ever added CABS).

### `xrom_resolve()` Extension

```rust
// Phase 43 (v3.3): ADV_MATH_A bit-3 arm
if modules & 0b0000_1000 != 0 {
    if let Some(op) = adv_math_a_resolve(name) {
        return Some(op);
    }
}
// Phase 43 (v3.3): ADV_MATH_B bit-4 arm
if modules & 0b0001_0000 != 0 {
    if let Some(op) = adv_math_b_resolve(name) {
        return Some(op);
    }
}
```

---

## Complex Ops Integration: CABS, CARG, CCHS, CCONJ, CY^X

These five functions operate on the EXISTING complex stack overlay (`state.complex_mode`, X+iY = ζ).

### Integration with Existing `math1/complex.rs` Infrastructure

The Advantage Pac complex ops are NOT in `math1/` (which is frozen). They live in `advantage/complex.rs`. However they depend on the same stack model:

- All five read from `state.stack.x` (real part of ζ) and `state.stack.y` (imag part of ζ)
- `CABS`: pushes `sqrt(x^2 + y^2)` to X, exits complex mode (returns real scalar)
- `CARG`: pushes `atan2(y, x)` to X in current angle mode, exits complex mode
- `CCHS`: negates both X and Y (`ζ' = -ζ`), stays in complex mode
- `CCONJ`: negates Y only (`ζ' = conj(ζ) = X - iY`), stays in complex mode
- `CY^X`: computes complex power `τ^ζ` using the existing complex arithmetic infrastructure

The `complex_atan2()` helper in `math1/complex.rs` is declared `pub(super)` — it needs to be promoted to `pub(crate)` so `advantage/complex.rs` can reuse it without duplication.

**CalcState fields consumed:** Only the existing `complex_mode: bool` and the standard stack. No new CalcState fields needed for these five ops.

### Stack-Lift Semantics

- CABS/CARG: `LiftEffect::Disable` (consume ζ from X+Y, push one real result to X; stack drops)
- CCHS/CCONJ: `LiftEffect::Neutral` (modify in place)
- CY^X: `LiftEffect::Disable` (consume both ζ and τ, push result to ζ; T-replicate)

---

## FROOT Integration: Romberg Polynomial Root Finder

FROOT in the Advantage Pac supports arbitrary-degree polynomials (not just degree 2-5). The algorithm is Romberg-based (Laguerre's method or similar iterative deflation extended beyond degree 5).

### How FROOT Differs from Existing `math1/poly.rs`

| Aspect | Math Pac I POLY (frozen) | Advantage FROOT (new) |
|--------|--------------------------|----------------------|
| Degree limit | 2–5 only | Arbitrary (hardware: up to register space) |
| Algorithm | Bairstow iterative deflation | Romberg / Laguerre method |
| Coefficient storage | R00–R05 (A–F prompt modal) | Coefficients pre-loaded in registers by user |
| Entry point | `XEQ "POLY"` opens modal workflow | `XEQ "FROOT"` — degree in X, coeff in registers |
| Output | U=u/V=v to print_buffer | Same U=u/V=v convention (MEDIUM confidence) |
| Mutual exclusion | Separate Op variants | Separate Op variants — `Op::AdvFrootWorkflow` |

**The POLY modal workflow in `math1/poly.rs` is NOT reused.** FROOT has different calling conventions and a different algorithm. The print-buffer output convention (`U=u / V=v`) is likely the same (hardware-faithful), but this must be confirmed against the Advantage Pac OM.

### ModalProgram Extension for FROOT

FROOT needs a prompt sequence: degree in X, then the root computation. If FROOT takes its degree from X directly (no modal prompt), it may not need a `ModalProgram` variant at all — confirm against OM. If it prompts interactively, `AdvantageStep::FrootDegreePrompt` follows the same pattern as `PolyInputStep::DegreePrompt`.

### User-Callback Re-entrancy

FROOT does NOT call a user-provided function label — it operates entirely on pre-loaded register data. FINTG does call a user function (the integrand). FINTG's integration uses the existing `run_loop` re-entrancy infrastructure already built for Math Pac I's INTG/SOLVE/DIFEQ. The `USER_CALLBACK_MAX_STEPS` constant in `math1/mod.rs` is already exported `pub(super)` and should be promoted to `pub(crate)` for reuse.

---

## ADV MTRX Integration: One-Shot Matrix Ops

The 52 ADV MTRX ops are M-code (machine code) one-shot operations. They are NOT modal workflows. They read matrix data from registers and return results immediately. This is a fundamentally different model than Math Pac I's MATRIX (which prompts for dimensions and elements interactively).

### Interaction with Existing `math1/matrix.rs` (frozen)

The existing `math1/matrix.rs` implements `MATRIX` workflow with Gauss-Jordan inversion. ADV MTRX adds:
- `MSYS`: solve Ax=b (uses MATRIX-like Gauss elimination, but one-shot — reads pre-loaded matrix from registers)
- `M*M` / `MAT*`: matrix multiplication (no equivalent in Math Pac I)
- `MINV`: matrix inversion — same operation as Math Pac I's `MatInv` but different calling convention (one-shot vs modal)
- `IDN`: identity matrix generation
- `V+`, `VDOT`: vector addition and dot product (not in Math Pac I at all)

**Name shadowing check required:** `MINV` and `INV` — Math Pac I claims `INV` → `Op::MatInv`. The Advantage Pac uses `MINV` (different mnemonic). The `xrom_shadowing.rs` CI gate must be extended to include both `ADV_MATH_A.ops` and `ADV_MATH_B.ops` against the MATH_1/STAT_1/TIME_MODULE allowlists.

**Critical: `NOT`, `AND`, `OR`, `XOR` shadowing** — These are common mnemonics. The existing `builtin_card_op` must be audited to ensure none of them appear there. If they do, the Advantage Pac mnemonic wins in `xrom_resolve` (fires last, after `builtin_card_op`), but this must be explicitly verified and documented.

### Register Layout

ADV MTRX operates on HP-41 matrix registers. Matrices are stored in named "matrix files" in HP-41 extended memory (XM), not in the numbered registers R00–R99. The matrix file format has:
- Header register: stores dimensions (rows, cols) and data-file pointer
- Data registers: column-major element storage

**CalcState impact:** The existing `matrix_dim: Option<(u8, u8)>` and `matrix_active_reg: Option<u8>` fields in CalcState (from Math Pac I) support a DIFFERENT matrix model (numbered registers R15...). ADV MTRX uses named matrix files. This likely requires new CalcState fields for the "current matrix" pointer/name.

---

## New CalcState Fields

### Required for Advantage Pac

| Field | Type | Serde | Purpose |
|-------|------|-------|---------|
| None for complex ops | — | — | Reuses `complex_mode: bool` |
| `adv_matrix_name: String` | `String` | `#[serde(default)]` | Name of currently active matrix file for ADV MTRX ops (`DIM?`, `MNAME?`, etc.) |
| `adv_tvm_state: Option<TvmState>` | `Option<TvmState>` | `#[serde(default)]` | TVM solver iteration state (persistent — user sets N, I, PV, PMT, FV and solves for missing) |

### Potentially Required (MEDIUM confidence — OM research needed)

| Field | Type | Serde | Trigger |
|-------|------|-------|---------|
| `froot_degree: Option<u8>` | `Option<u8>` | `#[serde(default, skip)]` | Transient degree context for FROOT if multi-step |
| `adv_fintg_state: Option<AdvFintgState>` | `Option<...>` | `#[serde(default, skip)]` | Enhanced integration state (separate from `integ_state`) |

**Minimize new fields** — the pattern from v3.1 (stat1) and v3.2 (time) shows that most ops can reuse the print_buffer/modal_program channels without new CalcState additions.

---

## ModalProgram Extension: Fourth Carve-Out

Following ADR-v3.1-004 and ADR-v3.1-005 pattern:

```rust
// In math1/modal.rs — fourth freeze carve-out
/// Advantage Pac workflows (Phase 43/44 — FROOT / FINTG prompt sequences)
///
/// math1/ freeze exception: this single additive variant + dispatch arms
/// is the FOURTH freeze carve-out for `math1/modal.rs`. All Advantage Pac
/// semantics live in `hp41-core/src/ops/advantage/modal.rs`.
Advantage(crate::ops::advantage::modal::AdvantageStep),
```

`AdvantageStep` enum lives in `advantage/modal.rs` (outside freeze boundary), parallel to `Stat1Step` in `stat1/modal.rs` and `TimeStep` in `time/modal.rs`.

### Required AdvantageStep Variants (MEDIUM confidence — depends on OM prompt sequences)

```rust
pub enum AdvantageStep {
    // FROOT prompt sequence (if interactive — confirm vs OM)
    FrootDegreePrompt,   // "DEGREE=?" — may not exist if degree comes from X
    FrootReady,
    // FINTG prompt sequence
    FintgFunctionNamePrompt,  // "FUNCTION NAME?"
    FintgIntervalPrompt,      // "(A,B)=?"
    FintgReady,
    // TVM — may not need modal (could be all immediate-solve style)
}
```

---

## 4-Way Exhaustive Match Invariant

Every new `Op::Adv*` variant must land in all four sites before any caller compiles:

1. `dispatch()` in `hp41-core/src/ops/mod.rs`
2. `execute_op()` in `hp41-core/src/ops/program.rs`
3. `op_display_name()` in `hp41-cli/src/prgm_display.rs`
4. `op_display_name()` in `hp41-gui/src-tauri/src/prgm_display.rs`

Items 3+4 are sanctioned-deferred to the CLI/GUI phases (same pattern as every previous XROM module). The intentional `non-exhaustive patterns` CI break in hp41-cli/hp41-gui during the core phase is expected and documented.

---

## JSON-Canonical Help Pipeline Extension

A fifth JSON file `docs/hp41-advantage-functions.json` becomes the source of truth for:
- `?` overlay section "Advantage Pac (XROM 22+24)"
- Right-panel exclusion via `entry.xrom.is_none()` (already excludes all XROM functions)
- `just docs-matrix` fifth invocation → `docs/hp41-advantage-function-matrix.md`

In `hp41-cli/src/help_data.rs`: fifth `OnceLock<Vec<HelpEntry>>` (`ADV_HELP_ENTRIES`) + 5-pool `help_entries_all()` chain.

---

## Suggested Build Order (Dependency-Aware)

The dependency chain determines phase ordering:

```
Phase 43: hp41-core (advantage/) — XROM framework + all ~117 Op variants
           ↓ (intentional CI break in cli/gui)
Phase 44: hp41-cli — JSON, op_display_name, ? overlay
           ↓
Phase 45: Documentation — divergences, function matrix, ADRs, README
           ↓
Phase 46: hp41-gui — CATALOG 2 extension, help overlay, modal routing
           ↓
Phase 47: Test Hardening — coverage, accuracy, backward compat, E2E
```

### Phase 43: hp41-core

**Prerequisites met:** `xrom.rs` carve-out for `ADV_MATH_A`/`ADV_MATH_B` constants + resolver arms; `modal.rs` carve-out for `ModalProgram::Advantage(AdvantageStep)`; promote `complex_atan2()` and `USER_CALLBACK_MAX_STEPS` to `pub(crate)`.

**Execution order within phase:**
1. `advantage/xrom.rs` — module registries (empty ops slices initially)
2. `advantage/conv.rs` — ADV CONV 12 ops (simple, no dependencies)
3. `advantage/complex.rs` — CABS/CARG/CCHS/CCONJ/CY^X (depends on complex_atan2 promotion)
4. `advantage/matrix.rs` — ADV MTRX 52 ops (largest, most complex; one-shot ops)
5. `advantage/froot.rs` — FROOT Romberg solver (depends on user-callback infra)
6. `advantage/fintg.rs` — FINTG enhanced integration (depends on INTG re-entrancy pattern)
7. `advantage/tvm.rs` — ADV TVM 6 ops (financial math, self-contained)
8. Wire all Op variants to `dispatch()` + `execute_op()` + `xrom.rs` ops slices

**Estimated new Op variants:** ~117 total across XROM 22+24 (subject to exact OM verification)

### Phase 44: hp41-cli

Wire `docs/hp41-advantage-functions.json` → fifth OnceLock; `op_display_name()` ~117 arms; `?` overlay "Advantage Pac (XROM 22+24)" section; `xrom_shadowing.rs` extended to cover ADV_MATH_A.ops + ADV_MATH_B.ops.

### Phase 45: Documentation

`docs/hp41-advantage-function-matrix.md` (generated); `docs/hp41-advantage-divergences.md` (three-bucket catalog); 3-5 new ADRs; README v3.3 soft-claim.

### Phase 46: hp41-gui

`prgm_display.rs` ~117 arms; CATALOG 2 extension (two new XROM entries: 22 + 24); help overlay 5th section; modal prompt routing for FROOT/FINTG.

### Phase 47: Test Hardening

Meta-gates (per-Op test count ≥ 5); backward-compat (`v32-autosave.json` fixture: bits 3+4 migration); numerical accuracy (complex ops, matrix ops, polynomial roots); E2E smoke (FROOT or MSYS workflow); README hard-claim graduation.

---

## Key Integration Points: New vs Modified Files

### New Files

| File | Purpose |
|------|---------|
| `hp41-core/src/ops/advantage/mod.rs` | Module root; `AdvantageStep` from modal.rs; named consts for register layout |
| `hp41-core/src/ops/advantage/xrom.rs` | `ADV_MATH_A` + `ADV_MATH_B` + resolver fns |
| `hp41-core/src/ops/advantage/modal.rs` | `AdvantageStep` enum + `submit_step()` + `current_prompt()` |
| `hp41-core/src/ops/advantage/conv.rs` | 12 ADV CONV ops |
| `hp41-core/src/ops/advantage/complex.rs` | 5 ADV MATH complex ops |
| `hp41-core/src/ops/advantage/matrix.rs` | 52 ADV MTRX ops |
| `hp41-core/src/ops/advantage/froot.rs` | FROOT Romberg root finder |
| `hp41-core/src/ops/advantage/fintg.rs` | FINTG enhanced integration |
| `hp41-core/src/ops/advantage/tvm.rs` | 6 ADV TVM ops |
| `docs/hp41-advantage-functions.json` | Fifth JSON source-of-truth |
| `docs/hp41-advantage-function-matrix.md` | Generated via `just docs-matrix` |
| `docs/hp41-advantage-divergences.md` | Divergence catalog (three-bucket, D-NN numbering) |

### Modified Files (Minimal Surface)

| File | Change | Invariant |
|------|--------|-----------|
| `hp41-core/src/ops/math1/xrom.rs` | Add `ADV_MATH_A` + `ADV_MATH_B` consts + two resolver fns + bits 3+4 in `xrom_resolve()` | 5th/6th freeze carve-out, doc comment required |
| `hp41-core/src/ops/math1/modal.rs` | Add `ModalProgram::Advantage(AdvantageStep)` variant + dispatch arms | 4th freeze carve-out, doc comment required |
| `hp41-core/src/ops/math1/complex.rs` | Promote `complex_atan2()` from `pub(super)` to `pub(crate)` | Additive only; frozen file otherwise unchanged |
| `hp41-core/src/ops/math1/mod.rs` | Promote `USER_CALLBACK_MAX_STEPS` to `pub(crate)` | Additive only |
| `hp41-core/src/state.rs` | 2-3 new CalcState fields; `default_xrom_modules() → 0b0001_1111`; `migrate_after_load()` bits 3+4 | Pattern: `#[serde(default)]` |
| `hp41-core/src/ops/mod.rs` | `pub mod advantage;` + ~117 new Op variants + dispatch arms | 4-way invariant item 1 |
| `hp41-core/src/ops/program.rs` | `execute_op()` arms for ~117 Advantage ops | 4-way invariant item 2 |
| `hp41-cli/src/prgm_display.rs` | ~117 new `op_display_name` arms | 4-way invariant item 3 |
| `hp41-cli/src/help_data.rs` | Fifth OnceLock + 5-pool `help_entries_all()` | JSON-canonical pipeline |
| `hp41-gui/src-tauri/src/prgm_display.rs` | ~117 new `op_display_name` arms | 4-way invariant item 4 |
| `hp41-cli/tests/xrom_shadowing.rs` | Extend to cover ADV_MATH_A.ops + ADV_MATH_B.ops | Pitfall 22 CI gate |
| `scripts/docs-matrix/src/main.rs` | 5th `else if` branch for `hp41-advantage-functions.json` | D-30.1 1-in/1-out pattern |
| `justfile` | Fifth `docs-matrix` + `docs-matrix-check` invocation | |
| `scripts/check-free42-contamination.sh` | Extend scan to `advantage/` directory | Free42 contamination guard |

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Putting Advantage Pac Code in `math1/`
**What:** Adding `advantage/complex.rs` functions or `FROOT` into the frozen `math1/` directory.
**Why bad:** `math1/` freeze invariant. Even though these ops use the complex stack (which originated in `math1/`), they belong in `advantage/` because they come from a different HP product with different XROM IDs.
**Instead:** `advantage/complex.rs` imports `complex_atan2` via `pub(crate)` from `math1/complex.rs`.

### Anti-Pattern 2: Creating Separate `ModalProgram` Enum for Advantage
**What:** Defining a new `AdvantageModalProgram` enum separate from the existing `ModalProgram`.
**Why bad:** The submit_modal / cancel_modal / submit_modal_with_label functions in `math1/mod.rs` all dispatch on `ModalProgram`. A parallel enum requires duplicating all dispatch infrastructure.
**Instead:** Add `ModalProgram::Advantage(AdvantageStep)` as the fourth variant in the existing `ModalProgram` enum (fourth carve-out of `modal.rs`, consistent with v3.1 and v3.2 patterns).

### Anti-Pattern 3: Reusing `Op::MatInv` for Advantage MINV
**What:** Pointing the `"MINV"` mnemonic in ADV_MATH_A to the existing `Op::MatInv`.
**Why bad:** `Op::MatInv` triggers the Math Pac I modal matrix workflow (requires pre-entered matrix via ORDER=? prompt). ADV MTRX's `MINV` is a one-shot op reading from named matrix files.
**Instead:** New `Op::AdvMinv` with separate implementation in `advantage/matrix.rs`.

### Anti-Pattern 4: Persisting FROOT/FINTG Iteration State
**What:** Adding `froot_state: Option<FrootState>` with `#[serde(default)]` (no skip).
**Why bad:** Iteration state is transient by nature (mid-computation). Persisting partial iteration state across save/load creates unsound continuation semantics.
**Instead:** `#[serde(default, skip)]` for all solver mid-iteration state — same as `integ_state`, `solve_state`, `difeq_state`.

### Anti-Pattern 5: Shadowing `NOT`, `AND`, `OR`, `XOR` Without Audit
**What:** Adding these boolean ops to ADV_MATH_A.ops without first verifying they don't appear in `builtin_card_op`.
**Why bad:** The resolver chain fires `builtin_card_op` BEFORE `xrom_resolve`. If `builtin_card_op` claims "NOT", `xrom_resolve` never sees it.
**Instead:** Run `grep -n '"NOT"\|"AND"\|"OR"\|"XOR"' hp41-core/src/ops/program.rs` before Phase 43. If any of them appear in `builtin_card_op`, document the conflict in the divergences file.

### Anti-Pattern 6: Treating "Advanced Matrix Pac" as a Separate XROM Module
**What:** Assigning a fifth/sixth XROM ID and separate bitmask bits for "Advanced Matrix Pac" as if it were a distinct physical module.
**Why bad:** Research confirms "Advanced Matrix Pac" is a community extension that reuses the Advantage Pac's XROM 22+24 function space. It is not a separate HP-published module with a distinct XROM ID.
**Instead:** Treat all Advantage Pac + Advanced Matrix Pac functions as one combined module under XROM 22+24. If the community matrix extension adds functions beyond the HP Advantage Pac OM, document them as emulator extensions in the divergences file (same discipline as RAND/SEED in v3.1).

---

## Scalability Considerations

This is a local calculator emulator — scalability means "number of registered Op variants" not users:

| Concern | Current (v3.2) | After v3.3 |
|---------|---------------|-------------|
| Op enum variants | ~200+ | +~117 = ~317+ |
| xrom_modules bitmask | 3 bits used (0b0000_0111) | 5 bits used (0b0001_1111) |
| help_entries_all() pools | 4 | 5 |
| docs-matrix invocations | 4 | 5 |
| JSON source files | 4 | 5 |
| Free42 scan directories | math1/, stat1/, time/ | + advantage/ |

The `u8` bitmask has 8 bits — 5 used after v3.3, 3 remain free for hypothetical future modules.

---

## Open Questions Requiring Phase-Specific Research

These MUST be resolved before or during Phase 43, not assumed:

1. **FROOT calling convention:** Does FROOT read degree from X register directly, or does it open an interactive modal prompt? This determines whether `AdvantageStep::FrootDegreePrompt` is needed.

2. **ADV MATH SOLVE/INTEG vs Math Pac I SOLVE/INTEG:** Are the Advantage Pac's SOLVE and INTEG the same algorithms as Math Pac I with the same mnemonics, or different algorithms with identical names? If same, mnemonic collision with XROM 7 must be resolved (XROM 24 fires AFTER XROM 7 in the resolver chain, so XROM 7 wins — this may be wrong for users who only have Advantage Pac loaded).

3. **`"NOT"`, `"AND"`, `"OR"`, `"XOR"` in `builtin_card_op`:** Must audit before adding to ADV_MATH_A.ops.

4. **CATALOG 2 display strings for XROM 22+24:** The `name` field in `ADV_MATH_A` and `ADV_MATH_B` should match what the real HP-41 CATALOG 2 displays. "ADV CONV A" and "ADV MATH B" are placeholders — verify against OM.

5. **`ADV_MATH_A.ops` completeness:** The 52 ADV MTRX ops are only partially known from community sources. The full list must be verified against the OM PDF before implementing. Do not stub-implement ops whose names are uncertain.

6. **TVM solver semantics:** Does CMPD trigger an iterative solve (like INTG/SOLVE user-callbacks), or is it a closed-form expression? If iterative, it needs the same `cancel_requested` arc and `run_loop` guard as INTG/SOLVE.

---

## Sources

- [HP-41 Module Database](https://calc.fjk.ch/db/hp41mod.php) — XROM 22+24 = Advantage Pac 1A/1B (HIGH confidence)
- [HP-41C XROM Numbers database](https://www.hpmuseum.org/software/xroms.htm) — XROM 22 ADV MTRX partial function list including M*M, MAT*, MSYS, IDN, V+, VDOT, C<>C, CMAXAB, CNRM (MEDIUM confidence — list from community parsing, not OM)
- [HP-41 Modules list PDF](https://lastin.dti.supsi.ch/VET/sys/HPXX/HP41CV/HP-41C-CV-CX_Modules.pdf) — Advantage Pac 1A/1B XROM 22+24 confirmed (HIGH confidence)
- [HP Museum forum: Advantage module functions](https://www.hpmuseum.org/cgi-bin/archv016.cgi?read=100494) — 117 functions, four headers, ADV MATH includes HP-15C complex ops (MEDIUM confidence)
- [HP Museum forum: Advanced Matrix Pac](https://www.hpmuseum.org/cgi-bin/archv020.cgi?read=184196) — community extension, not official HP (research note: 403 on fetch, confirmed community origin from search snippets)
- [Advantage Math ROM Manual (Ángel Martin, 2020)](https://www.systemyde.com/pdf/Advantage_Math_Manual.pdf) — community XROM 12 extension, NOT the official HP Advantage Pac; useful as reference for matrix op semantics (ADV MTRX functions referenced in application examples)
- [HP-41 Advantage Pac manual](https://literature.hpcalc.org/community/hp41-pac-advantage-en.pdf) — official HP OM, 156pp; PDF too large to parse fully in research; function table pages not retrieved (LOW confidence on per-function details — requires Phase 43 pre-work)
- Existing codebase: `state.rs`, `ops/math1/xrom.rs`, `ops/math1/modal.rs`, `ops/math1/complex.rs`, `ops/math1/mod.rs`, `ops/math1/poly.rs` (all directly read — HIGH confidence on integration points)
