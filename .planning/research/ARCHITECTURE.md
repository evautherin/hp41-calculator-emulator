# Architecture Research: Stat 1 Pac Integration

**Domain:** HP-41 XROM application module emulation (second module, after Math Pac I)
**Researched:** 2026-05-21
**Confidence:** HIGH (existing codebase fully read; XROM ID confirmed from two independent external sources)

---

## Integration Overview

Stat 1 Pac slots directly into the v3.0 XROM framework with four categories of work:

1. A new `STAT_1` const + `stat1_resolve()` in `hp41-core/src/ops/math1/xrom.rs` (bit 1 of `xrom_modules`)
2. New `Op` variants + `hp41-core/src/ops/stat1/` module tree (mirrors `ops/math1/`)
3. A third `OnceLock<Vec<HelpEntry>>` in `hp41-cli/src/help_data.rs` + `docs/hp41-stat1-functions.json`
4. New `ModalProgram` variants and CalcState fields following the same `#[serde(default)]` / `#[serde(skip)]` discipline as v3.0

No new framework infrastructure is needed. The resolver chain, modal-routing layer, user-callback infrastructure, `request_cancel` channel, JSON pipeline, and 4-way exhaustive-match invariant are all fully reusable.

---

## (a) XROM ID: Which Hardware Module Number?

### Authoritative Finding

The HP-41 Statistics Pac 1 carries **hardware XROM ID 2**.

Sources:
- `calc.fjk.ch/db/hp41mod.php` (HP-41 Module Database) lists "Statistics Pac 1B" as **XROM #2, 4k**.
- The HP-41 XROM numbering scheme is `NN,FF` where `NN` is the ROM module number (1-31) and `FF` is the function number within that ROM. Multiple community sources confirm: Math Pac functions = `1,NN`, Statistics Pac functions = `2,NN`. The combined Math/Stat module uses XROM 1 and XROM 2 together.

### Codebase Discrepancy with Math Pac I

The existing `MATH_1.id = 7` in `hp41-core/src/ops/math1/xrom.rs` is inconsistent with the hardware catalog (Math Pac = XROM 1, not 7). The comment "real HP-41C hardware Math Pac I XROM module ID" in the v3.0 code was based on Phase 28 research that reached an incorrect conclusion. Since v3.0 is shipped and the ID is only used for CATALOG 2 display text and bitfield documentation — not for any user-visible address computation — this is a low-impact divergence already locked in.

**Decision for v3.1:** Use `id: 2` for `STAT_1` (the actual hardware XROM ID), and document the Math Pac discrepancy in `docs/hp41-stat1-divergences.md`. Do NOT retroactively change `MATH_1.id` — that is a frozen invariant on shipped code.

### CATALOG 2 Display Name

On real HP-41 hardware, the Stat Pac appears in CATALOG 2 as `"STAT 1B"` (version-dependent; versions 1A and 1B both use XROM 2). Use `name: "STAT 1B"` in the `STAT_1` const, matching the `"MATH 1A"` precedent.

### Bitfield Assignment

`xrom_modules: u8` in `CalcState`:
- Bit 0 = Math 1 loaded (v3.0, `default_xrom_modules() = 0b0000_0001`)
- Bit 1 = Stat 1 loaded (v3.1, extend default to `0b0000_0011`)

The default function changes to `0b0000_0011` so both modules are pre-loaded, matching the Math Pac I pre-loaded pattern. Old v3.0 save files that serialized `"xrom_modules": 1` will load with bit 1 clear (Stat 1 absent). This is acceptable backward-compat behavior — the user gets Math 1 only until the emulator upgrades. Add a `v30_save_loads_with_stat1_off` backward-compat test to document this.

---

## (b) JSON Pipeline: Third Sibling File

### Decision

Create `docs/hp41-stat1-functions.json` as a third sibling file, following D-29.1 precedent (ADR-005) exactly.

**Rationale:**
- D-29.1 established that each module gets its own JSON file ("separate-file" shape, not merged).
- The `docs-matrix` binary (`scripts/docs-matrix/src/main.rs`) is already single-JSON-in / single-markdown-out. Adding a third file requires only: (1) a title-dispatch branch for the new basename in `render_markdown()`, and (2) two new `just docs-stat1` and `just docs-stat1-check` recipe invocations in `justfile`.
- `help_data.rs` needs a third `OnceLock<Vec<HelpEntry>>` (`STAT1_HELP_ENTRIES`) and a third narrow accessor `help_entries_stat1()`. The merged accessor `help_entries_all()` chains all three pools.
- `function_matrix_parity.rs` and `key_coverage.rs` tests sweep `help_entries_all()` — they pick up Stat 1 automatically once chained.

**What NOT to do:** Do not merge into `hp41-math1-functions.json`. That file is the MATH_1 canonical source. Merging would conflate two distinct hardware modules and break the per-module `XromEntry.module_id` discriminator.

### JSON Schema Extension

The existing `HelpEntry` schema (with optional `xrom: Option<XromEntry>`) needs no changes. Stat 1 entries carry `"xrom": { "module": "Stat 1", "module_id": 2, "function_id": N }`. The `#[serde(default)]` on `xrom` is already present.

The `docs-matrix` binary's `render_markdown()` title-dispatch needs one new branch:

```rust
} else if basename.ends_with("hp41-stat1-functions.json") {
    ("# HP-41C Stat Pac I Function Matrix", "`docs/hp41-stat1-functions.json`")
```

---

## (c) New CalcState Fields

### RNG State

The HP-41 Stat Pac provides random number generation. Real HP-41 RNG uses a linear congruential algorithm with a seed stored in a data register. The Stat Pac FOCAL programs read/write a designated register for the seed — the OM specifies which one (typically R00 or a low-numbered register).

**Recommendation:** Do NOT add a dedicated `rng_seed` field to `CalcState`. Use the data register the OM specifies, following real HP-41 convention. This requires zero new CalcState fields and zero serde changes. The Stat Pac `op_rng()` reads/writes `state.regs[seed_reg]` directly. If the OM specifies a hidden/scratch register not in `state.regs` (unlikely — Stat Pac is FOCAL-level user code), then add `#[serde(default)] pub rng_seed: HpNum` as a fallback, but treat this as the last resort.

### Distribution Scratch / Intermediate State

Distribution quantile functions (inverse normal, inverse t, inverse chi-squared, inverse F) require root-finding iteration. Unlike Math Pac I's SOLVE/INTG which use `run_loop` user-callback re-entrancy, Stat Pac distributions are self-contained: they iterate over a private loop with no user-supplied callback program.

**Decision: No user-callback infrastructure needed.** Use a private iteration loop inside each distribution quantile function, exactly like `op_poly()` uses a self-contained eigenvalue iteration. This means:
- No new `*_state: Option<...>` CalcState fields for distribution scratch
- No `#[serde(skip)]` additions for distributions
- The `request_cancel: Arc<AtomicBool>` check IS needed for long-running quantile iterations — reuse the existing field with the same 64-iteration polling interval

**If a function genuinely requires multi-step user prompting** (e.g., a hypothesis test that prompts for sample data entry one value at a time), it uses the existing `modal_program + modal_prompt` machinery with a new `ModalProgram::Stat1(Stat1InputStep)` variant. No new CalcState fields are required for this; `modal_program` is already `#[serde(default, skip)]`.

### Frequency Table / Histogram State

The HP-41 Stat Pac does NOT include histogram/frequency-table operations as ROM-resident programs. The OM-documented programs use the standard Sigma registers (R01–R06) and user-defined registers for data storage. No histogram-specific CalcState fields are needed.

### Summary of New CalcState Fields

| Field | Type | Serde | Needed? | Reason |
|-------|------|-------|---------|--------|
| `rng_seed` | `HpNum` | `#[serde(default)]` | Only if OM specifies non-register seed | Use data register by default |
| `stat1_scratch` | `Option<Stat1State>` | `#[serde(default, skip)]` | No | Private iteration loops, no user callback |
| Any modal state | embedded in `ModalProgram::Stat1(...)` | already `#[serde(skip)]` | Only if multi-step prompt needed | Reuse existing `modal_program` field |

**Likely net-new CalcState fields: 0.** All state lives in existing registers and the existing transient fields.

---

## (d) Stat 1 Pac Data Storage — Where Live Data Lives

### HP-41 Sigma Register Convention (already in codebase)

The existing `ops/stats.rs` Sigma infrastructure uses R01–R06 (0-indexed: `regs[1]..regs[6]`):
- `regs[1]` = Sx2 (sum of x-squared)
- `regs[2]` = Sx (sum of x)
- `regs[3]` = n (count)
- `regs[4]` = Sy2 (sum of y-squared)
- `regs[5]` = Sy (sum of y)
- `regs[6]` = Sxy (sum of x*y)

The `SIGMAREG` command (present in v2.2) allows repositioning this block. The Stat Pac programs call Sigma+/Sigma- using the standard HP-41 Sigma register set. **The Stat Pac does not invent new register semantics.**

### Stat Pac Register Usage

- **Univariate statistics** (WSTAT, CSTAT): read directly from Sigma registers R01-R06 after user has accumulated data via Sigma+
- **Bivariate / curve fitting**: same Sigma register set; Stat Pac adds logarithmic, exponential, power curve models on top of v2.2 `op_lr()` by transforming x/y before Sigma accumulation
- **Distributions** (NOR, CHI-SQ, T-DIST, F-DIST, BINOMIAL, POISSON): parameters come from the stack (X, Y registers) — no separate register block
- **RNG**: reads/writes one data register as seed — no new block beyond what the OM specifies
- **Permutations/Combinations**: computed from X and Y on the stack — pure stack ops, no register state

### Comparison to Math Pac I Complex/Matrix Data

Math Pac I needed register blocks beyond Sigma: matrix elements live at R15+, matrix order in R14, complex stack overlays X/Y/Z/T. These required new `CalcState` fields (`matrix_dim`, `matrix_active_reg`, `complex_mode`).

Stat 1 Pac is more stack-and-sigma-oriented:
- No matrix-like element block claiming a separate register range
- No stack-overlay mode (no equivalent of `complex_mode`)
- No `matrix_dim` / `matrix_active_reg` analog needed

The `ops/stats.rs` Sigma operations are already a solid foundation. The Stat Pac wraps and extends them.

### Curve Fitting Register Convention

HP-41 Stat Pac curve-fitting programs (linear, logarithmic, exponential, power) use the same Sigma registers as `op_lr()`. The FOCAL programs compute transformations (e.g., `ln(y)` for exponential fit) before accumulating into Sigma registers. No new register block needed in `CalcState` for this. Emulation: the Stat Pac `op_expfit_plus()` transforms data and then calls `op_sigma_plus()`, delegating to the existing Sigma infrastructure.

---

## (e) Build Order and Framework Dependencies

### What Is Genuinely NEW vs REUSED from v3.0

| Concern | v3.0 Status | v3.1 Action |
|---------|-------------|-------------|
| XROM framework (`XromModule`, `xrom_resolve`, `MATH_1`) | Shipped, frozen | Add `STAT_1` const, extend `xrom_resolve` with `stat1_resolve()` behind bit 1 check |
| `xrom_modules` bitfield in `CalcState` | Bit 0 used | Extend: bit 1 = Stat 1; change `default_xrom_modules()` to `0b0000_0011`; add serde backward-compat test |
| `ModalProgram` enum | 7 variants | Add `Stat1(Stat1InputStep)` variant if any Stat Pac program needs multi-step prompting; otherwise leave untouched |
| `submit_modal` / `cancel_modal` / `submit_modal_with_label` | Public API | Extend `submit_modal` with new `ModalProgram::Stat1` arm if added |
| `modal_prompt`, `modal_program` CalcState fields | Already transient + `#[serde(skip)]` | Reused as-is |
| `cancel_requested: Arc<AtomicBool>` | Already in CalcState | Reused as-is for distribution quantile iteration cancellation |
| `print_buffer` drain pattern | Already present | Reused for any Stat Pac print output (e.g., WSTAT result lines) |
| `help_data.rs` OnceLock chain | 2 pools | Add third pool: `STAT1_HELP_ENTRIES` + `help_entries_stat1()` accessor; extend `help_entries_all()` chain |
| `docs-matrix` binary | 1-in/1-out, 2 title branches | Add third title branch; add two `justfile` recipes |
| `prgm_display.rs` (CLI + GUI) | ~40 arms each | Extend both with new Op variant arms (4-way invariant items 3+4) |
| `xrom_shadowing.rs` CI gate | Covers MATH_1.ops | Extend allowlist to cover STAT_1.ops non-collision with v2.2 builtins AND MATH_1 names |
| `numerical_accuracy.rs` | 763 cases | Extend with Stat 1 numerical cases (distributions, curve fitting, P&C, RNG smoke) |
| Free42 contamination guard | 12-symbol grep | Extend only if new Stat Pac-distinctive identifiers appear (LOW risk: stat algorithms are standard public-domain methods) |
| 4-way exhaustive-match invariant | dispatch + execute_op + 2x prgm_display | Every new Op variant lands in all four before any caller compiles — no exceptions |

### Recommended Build Order (5 Phases, starting at Phase 33)

**Phase 33: hp41-core**
- New `src/ops/stat1/` module tree (mirrors `ops/math1/` structure: xrom.rs, modal.rs if needed, stats_ext.rs, curvfit.rs, distributions.rs, rng.rs)
- `STAT_1` const + `stat1_resolve()` — extend `ops/math1/xrom.rs` (minimal surgical edit)
- All new `Op` variants (Sigma-stat extensions, curve fitting, distributions, RNG, P&C)
- `ModalProgram::Stat1` variant in `ops/math1/modal.rs` if any workflow needs multi-step prompting
- CalcState: change `default_xrom_modules()` to `0b0000_0011`; add `rng_seed: HpNum` only if OM requires it
- `dispatch()` + `execute_op()` arms for all new Op variants (4-way invariant items 1+2)
- `xrom_modules` bit 1 wiring in `xrom_resolve()`
- Compile gate: zero new panics, 4-way match completeness enforced at build time

**Phase 34: hp41-cli**
- `prgm_display.rs` new arms (4-way invariant item 3)
- `help_data.rs` third `OnceLock` + `help_entries_stat1()` + extend `help_entries_all()`
- `docs/hp41-stat1-functions.json` authored (D-29.1 precedent: DOC-01 absorbed)
- `xeq_by_name_local_resolve` already routes through `xrom_resolve` — no changes needed
- Modal-prompt routing for any new `Stat1` modal programs in `app.rs`

**Phase 35: Docs and ADRs**
- `docs-matrix` binary: third title branch
- `justfile`: `just docs-stat1` + `just docs-stat1-check` recipes
- `docs/hp41-stat1-function-matrix.md` generated
- `docs/hp41-stat1-divergences.md` (OM divergences + XROM ID 7 vs hardware clarification + algorithm sources)
- ADRs for any new architectural decisions (RNG register choice, distribution algorithm selection)
- README v3.1 soft-claim; CLAUDE.md v3.1 section

**Phase 36: hp41-gui**
- `src-tauri/src/prgm_display.rs` new arms (4-way invariant item 4)
- CATALOG 2 XROM enumeration extended (`STAT_1` entry after `MATH_1`)
- Help overlay: `"Stat 1 Pac (XROM 2)"` section parallel to `"Math 1 Pac (XROM 7)"` section
- `key_map.rs`: no new keys needed (all Stat Pac via XEQ-by-name, same as Math Pac I)
- `xrom_modules` GUI rendering updated if module status is displayed anywhere

**Phase 37: Test Hardening and Quality Gates**
- `hp41-core` coverage >= 95% maintained (likely needs targeted error-branch tests)
- `numerical_accuracy.rs` extended with Stat 1 cases (distributions, curve fitting, P&C)
- `xrom_shadowing.rs`: extend to cover `STAT_1.ops` vs v2.2 builtins + `MATH_1` names
- `stat1_coverage` meta-gate (mirrors `math1_op_test_count` pattern from Phase 32)
- E2E smoke: one Stat Pac XEQ workflow (e.g., `XEQ "WSTAT"` or `XEQ "NOR"`)
- Free42 contamination check: verify no Stat Pac-specific GPL symbols introduced
- `v30_save_loads_with_stat1_off` backward-compat regression test

### Critical Dependency: 4-Way Match Completeness

The 4-way invariant means `prgm_display.rs` in both CLI and GUI must have arms for all new Op variants before those crates compile. In v3.0, the CLI arm landed in Phase 29 and the GUI arm in Phase 31 — the invariant held because Phase 33 (hp41-core) compiles independently from hp41-cli and hp41-gui. The two `prgm_display.rs` files are only required when those crates build.

The safe sequence within each phase is: add Op variant to `Op` enum AND add its arm to `dispatch()`, `execute_op()`, and the respective `prgm_display.rs` in a single commit batch. Never leave a new `Op` variant without all four arms in the same compilable commit state.

### No Framework Upgrades Needed

Unlike v3.0 which required building the XROM framework from scratch (Plan 28-01 was entirely new infrastructure), v3.1 needs zero framework changes:
- `xrom_resolve()` already has the comment stub: `// Future v3.1+ modules go here: if modules & 0b0000_0010 != 0 { stat1_resolve(name) }`
- The resolver chain fires LAST (Pitfall 1) — adding `stat1_resolve` inside the existing `xrom_resolve` function maintains this invariant automatically
- `XromModule` struct is generic — `STAT_1` is a new const of the existing type
- Modal infrastructure, user-callback infrastructure, and cancel channel all reused unchanged

---

## Component Boundary Map

```
hp41-core/src/ops/
├── math1/           (frozen since Phase 28-01..32; touch xrom.rs only)
│   ├── xrom.rs      (ADD: STAT_1 const, stat1_resolve(), extend xrom_resolve())
│   ├── modal.rs     (ADD: ModalProgram::Stat1 + Stat1InputStep if multi-step needed)
│   └── mod.rs       (ADD: extend submit_modal if Stat1 modal variant added)
│
├── stat1/           (NEW — mirrors math1/ structure)
│   ├── mod.rs       (module pub re-exports)
│   ├── stats_ext.rs (extended statistics: weighted mean, grouped sigma)
│   ├── curvfit.rs   (curve fitting: log/exp/power on Sigma registers)
│   ├── distributions.rs (normal, t, chi-sq, F, binomial, Poisson CDF + quantile)
│   └── rng.rs       (LCG random number generator reading/writing seed register)
│
└── mod.rs           (ADD: Op variants for all Stat 1 ops; dispatch() arms; imports)

hp41-core/src/ops/program.rs    (ADD: execute_op() arms for all Stat 1 Op variants)
hp41-core/src/state.rs          (MODIFY: default_xrom_modules() -> 0b0000_0011)

hp41-cli/src/
├── help_data.rs     (ADD: STAT1_HELP_ENTRIES, help_entries_stat1(), extend help_entries_all())
└── prgm_display.rs  (ADD: op_display_name arms for all Stat 1 Op variants)

hp41-gui/src-tauri/src/
└── prgm_display.rs  (ADD: op_display_name arms for all Stat 1 Op variants)

docs/
├── hp41-stat1-functions.json      (NEW: canonical Stat 1 function table)
├── hp41-stat1-function-matrix.md  (NEW: generated by docs-matrix)
└── hp41-stat1-divergences.md      (NEW: OM divergences + XROM ID 7 clarification)

scripts/docs-matrix/src/main.rs   (ADD: third title-dispatch branch)
justfile                           (ADD: just docs-stat1, just docs-stat1-check)
```

---

## Data Flow: Stat 1 Pac Op Invocation

**Non-modal (e.g., NOR, PERM, COMB, RNG):**

```
User types XEQ "NOR" (CLI) or presses XEQ key (GUI)
    |
    v
xeq_by_name_local_resolve() -- checks v2.2 built-ins (no match)
    |
    v
builtin_card_op() -- not a card reader op (no match)
    |
    v
xrom_resolve("NOR", state.xrom_modules)
  |-- bit 0 set? -> math1_resolve("NOR") -> None (not a Math Pac I function)
  +-- bit 1 set? -> stat1_resolve("NOR") -> Some(Op::Nor)
    |
    v
dispatch(Op::Nor, &mut state) -> op_nor(state)
  |-- reads X = argument (probability p)
  |-- private iteration (rational approximation, no user callback)
  |-- per-64-iterations: check state.cancel_requested.load(Relaxed)
  +-- writes result via unary_result(state, result)
```

**Modal (e.g., hypothesis test prompting for sample count):**

```
XEQ "TTEST"
    |
    v
dispatch(Op::Ttest, ..) -> sets state.modal_program = Some(ModalProgram::Stat1(
                               Stat1InputStep::SampleSizePrompt))
    | (same as Math Pac I modal flow)
    v
R/S submit -> submit_modal(state) -> stat1::submit_step(state, step)
    |
    v
Modal completes -> result on stack, modal_program = None
```

**Sigma-based statistics (WSTAT, CSTAT, curve fitting accumulation):**

```
XEQ "WSTAT"
    |
    v
dispatch(Op::Wstat, ..) -> op_wstat(state)
  |-- reads Sigma registers regs[1]..regs[6]
  |-- computes weighted statistics from accumulator state
  +-- writes lines to state.print_buffer (drained by CLI/GUI)
```

---

## Integration Points: Each CLAUDE.md Invariant Considered

| Invariant | v3.1 Impact |
|-----------|-------------|
| **4-way exhaustive match** | Every new Op variant must land in `dispatch()` + `execute_op()` + both `prgm_display.rs` before any caller compiles. Same as v3.0. No exceptions. |
| **Resolver chain (D-07)** | `stat1_resolve` fires inside `xrom_resolve` which fires LAST. The comment stub in `xrom.rs` already marks the exact insertion point. No resolver chain restructuring needed. |
| **Save-file backward compat** | `xrom_modules` already `#[serde(default)]`; changing `default_xrom_modules()` to `0b0000_0011` means v3.0 saves with `"xrom_modules": 1` load with Stat 1 disabled. Add `v30_save_loads_with_stat1_off` test to document this. |
| **JSON canonical pipeline** | Third sibling file `docs/hp41-stat1-functions.json`. Existing schema (HelpEntry + XromEntry) covers Stat 1 without changes. `help_entries_all()` must chain all three pools. |
| **No core duplication (SC-4)** | `op_display_name()` in GUI `prgm_display.rs` is the ONLY stat-related code in `hp41-gui/src-tauri/src/`. All computation in `hp41-core`. |
| **Free42 contamination guard** | Stat algorithms (normal distribution, t-distribution, etc.) use public-domain methods (e.g., Abramowitz and Stegun approximations, Wichura AS241 for inverse normal if used). Document sources in op comments. Extend the grep pattern only if Stat Pac-specific Free42 identifiers appear (LOW risk). |
| **Print emulation** | `println!`/`eprintln!` forbidden in `hp41-core`. Stat Pac print output (WSTAT/CSTAT result display) uses `state.print_buffer` drain pattern — same as Math Pac I DIFEQ step-output. |
| **No async, no panics** | All Stat 1 ops follow `#![deny(clippy::unwrap_used)]`. Distribution quantile iterations return `Err(HpError::InvalidOp)` on non-convergence rather than panicking. |
| **Stack-lift declarations** | Every new Op variant declares `LiftEffect::Enable / Disable / Neutral`. Distribution result ops use `unary_result()` (LiftEffect::Enable). Sigma accumulation ops follow the Sigma+ pattern (manual lift handling, not `binary_result()`). |
| **`modal_prompt` routing** | If any Stat Pac program needs multi-step prompting, it follows the Phase 29/31 wiring: `modal_prompt` written by `current_prompt()`, rendered by CLI's `pending_prompt()` and GUI's overlay banner. No new routing infrastructure. |
| **`pending_input` routing block above modal interceptors** | The D-07 resolver chain constraint applies equally to Stat 1 ops called via XEQ-by-name. No new key bindings means no new modal-interceptor edge cases. |

---

## Architectural Patterns to Follow

### Pattern 1: XROM Module Registration (from Math Pac I)

Each module is a `pub const XromModule` in its own file, added to `xrom_resolve()` behind a bit check. The `ops` slice drives both the resolver and the `xrom_shadowing.rs` test. Stat 1 follows this exactly:

```rust
// In hp41-core/src/ops/stat1/xrom.rs (or added to ops/math1/xrom.rs):
pub const STAT_1: XromModule = XromModule {
    id: 2,            // HP-41C hardware Statistics Pac XROM module ID
    name: "STAT 1B",  // CATALOG 2 display name
    ops: &[
        ("WSTAT",  Op::Wstat),
        ("CSTAT",  Op::Cstat),
        // ... all stat1 mnemonics
    ],
};

// Minimal extension to xrom_resolve() in ops/math1/xrom.rs:
pub fn xrom_resolve(name: &str, modules: u8) -> Option<Op> {
    if modules & 0b0000_0001 != 0 {
        if let Some(op) = math1_resolve(name) { return Some(op); }
    }
    if modules & 0b0000_0010 != 0 {
        if let Some(op) = stat1_resolve(name) { return Some(op); }
    }
    None
}
```

### Pattern 2: Self-Contained Iteration (from POLY/ROOTS — NOT from SOLVE/INTG)

Distribution quantile functions use private iteration with no user callback. This is the POLY pattern, not the SOLVE pattern:

```rust
pub fn op_inv_normal(state: &mut CalcState) -> Result<(), HpError> {
    let p = state.stack.x.clone();
    // Rational approximation -- O(1) for central region, iteration for tails
    let mut iter = 0u32;
    let result = loop {
        iter += 1;
        if iter % 64 == 0 && state.cancel_requested.load(Ordering::Relaxed) {
            return Err(HpError::InvalidOp); // cancelled
        }
        // ... iteration body returning result
        break candidate;
    };
    crate::stack::unary_result(state, result);
    Ok(())
}
```

### Pattern 3: Sigma Register Reuse

Stat Pac curve fitting accumulates into the existing Sigma register block (R01-R06 by default). The Stat Pac transforms x/y data before calling `op_sigma_plus()`:

```rust
pub fn op_log_fit_plus(state: &mut CalcState) -> Result<(), HpError> {
    // Logarithmic curve fit: accumulate (ln(x), y) into Sigma registers
    let x_transformed = state.stack.x.checked_ln()?;
    state.stack.x = x_transformed;
    crate::ops::stats::op_sigma_plus(state)
}
```

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Touching the Frozen math1/ Directory Beyond xrom.rs

`hp41-core/src/ops/math1/` is frozen since Phase 28-32 (CLAUDE.md explicit invariant). The exception is `xrom.rs` (one new `if modules & 0b0000_0010` arm in `xrom_resolve()`) and `modal.rs` (one new `ModalProgram::Stat1` variant if needed). All other Stat 1 computation lives in the new `ops/stat1/` tree.

### Anti-Pattern 2: Using run_program Re-Entrancy for Distribution Quantiles

Math Pac I's SOLVE/INTG/DIFEQ use `run_loop` user-callback re-entrancy because they call user-supplied LBL programs. Distribution quantile root-finders have NO user callback. Using the user-callback infrastructure here would add unnecessary complexity and violate ADR-002's strict-reject policy for nested calls.

Use simple iteration loops with `cancel_requested` polling instead.

### Anti-Pattern 3: Duplicating Sigma Register Logic

`op_sigma_plus()`, `op_sigma_minus()`, `op_mean()`, `op_sdev()`, `op_lr()` in `ops/stats.rs` are authoritative. Stat Pac extensions that build on these (WSTAT, CSTAT, curve fitting) must call these functions rather than duplicating the Sigma arithmetic.

### Anti-Pattern 4: Adding a `complex_mode`-Style Flag for Stat 1

Math Pac I needed `complex_mode: bool` because the complex stack overlays X/Y/Z/T. Stat 1 Pac has no equivalent mode. Statistics data lives in registers, not in a stack-overlay mode. No new boolean mode flag is needed in CalcState.

### Anti-Pattern 5: Merging Stat 1 JSON Into math1-functions.json

Each module gets its own JSON file (ADR-005). Merging would conflate two distinct XROM modules, break `XromEntry.module_id` filtering in `key_ref_entries()`, and make the `xrom` field mandatory even for v2.2 built-in entries. Keep three separate files.

---

## Sources

- HP-41 Module Database (calc.fjk.ch/db/hp41mod.php): Statistics Pac 1B = XROM #2, 4k. HIGH confidence.
- HP-41 XROM numbering scheme confirmation: XROM NN,FF where NN is module number 1-31; web search confirms Math Pac functions = 1,NN and Statistics Pac functions = 2,NN. MEDIUM confidence (HP Museum xroms.htm was inaccessible; confirmed via multiple community sources).
- Existing codebase `hp41-core/src/ops/math1/xrom.rs`: MATH_1 const structure, XromModule fields, `xrom_resolve()` stub comment for v3.1+ (`if modules & 0b0000_0010 != 0`). HIGH confidence — direct code read.
- Existing codebase `hp41-core/src/state.rs`: CalcState fields, `xrom_modules` bitfield, `default_xrom_modules()`, serde annotations. HIGH confidence — direct code read.
- Existing codebase `hp41-cli/src/help_data.rs`: OnceLock pattern, `help_entries_all()` chain, HelpEntry/XromEntry schema. HIGH confidence — direct code read.
- Existing codebase `scripts/docs-matrix/src/main.rs`: title-dispatch pattern showing where to add the third branch. HIGH confidence — direct code read.
- Existing codebase `hp41-core/src/ops/stats.rs`: Sigma register layout R01-R06, op_sigma_plus/minus pattern. HIGH confidence — direct code read.
- Existing codebase `hp41-core/src/ops/math1/modal.rs`: ModalProgram enum structure, submit_modal/cancel_modal API. HIGH confidence — direct code read.
- PROJECT.md v3.0 shipping record: confirmed XROM framework as reusable infrastructure, resolver chain invariants. HIGH confidence — direct read.

---

*Architecture research for: HP-41 Stat 1 Pac XROM module integration into v3.0 framework*
*Researched: 2026-05-21*
