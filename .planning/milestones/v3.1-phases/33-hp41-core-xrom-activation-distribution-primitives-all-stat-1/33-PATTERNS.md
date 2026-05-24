# Phase 33: hp41-core — XROM Activation + Distribution Primitives + All Stat 1 Ops — Pattern Map

**Mapped:** 2026-05-22
**Files to be created or modified:** 23
**Analogs found:** 23 / 23 (every Stat 1 Pac file mirrors a Math Pac I sibling or a sister-self extension in `hp41-core/src/ops/math1/`, `hp41-core/src/ops/`, `hp41-cli/src/`, `hp41-gui/src-tauri/src/`, `docs/`, or `scripts/docs-matrix/`)

**Headline mapping:** the single closest analog for every Stat 1 file is the v3.0 Math Pac I module at `hp41-core/src/ops/math1/`. Stat 1 Pac is a sibling XROM 8 module (id `2`, name `"STAT 1B"`) — same architecture, same disclaim discipline, same JSON-canonical pipeline, same docs-matrix shape. **The one freeze-exception flag (`ModalProgram::Stat1`) is surfaced in §Pitfall 8 below — planner must address before Plan 33-03.**

---

## File Classification

| New / Modified File | Action | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|---|
| `hp41-core/src/ops/stat1/mod.rs` | create | module-hub | request-response | `hp41-core/src/ops/math1/mod.rs` | exact (sibling XROM module hub) |
| `hp41-core/src/ops/stat1/xrom.rs` | n/a (Plan 33-03 — merged into math1/xrom.rs per D-33.3) | — | — | — | — |
| `hp41-core/src/ops/stat1/distributions.rs` | create | numerical-primitive | transform | `hp41-core/src/num.rs:178–211` (f64-bridge trio) + `hp41-core/src/ops/math1/integ.rs` (oracle-constants pattern) | exact (f64-bridge) + exact (inline oracle) |
| `hp41-core/src/ops/stat1/modal.rs` (or extension of `math1/modal.rs`) | create + freeze-exception edit | state-machine | event-driven | `hp41-core/src/ops/math1/modal.rs:23–84` (`ModalProgram` carrier enum) | exact (variant addition) — **FREEZE EXCEPTION FLAGGED** |
| `hp41-core/src/ops/stat1/normd.rs` | create | mode-dispatcher + iterative | request-response | `hp41-core/src/ops/math1/poly.rs:80–90` (modal opener) + `hp41-core/src/ops/math1/integ.rs:104–111` (display-mode-tied tolerance + cancel loop) | role-match + exact |
| `hp41-core/src/ops/stat1/chisqd.rs` | create | mode-dispatcher + iterative | request-response | `hp41-core/src/ops/math1/poly.rs:80–90` (modal opener for `ν=?`) + `hp41-core/src/ops/math1/integ.rs:104–111` | role-match + exact |
| `hp41-core/src/ops/stat1/basic_stats.rs` | create | accumulator-consumer | CRUD | `hp41-core/src/ops/stats.rs:22–53` (`op_sigma_plus`) | exact (Σ-register delegate) |
| `hp41-core/src/ops/stat1/moments.rs` | create | accumulator-extended | CRUD | `hp41-core/src/ops/stats.rs` + `mod.rs::STAT1_MAX_REG` (Plan 33-00 transcription) | role-match (extends R01–R06 → OM-extended block) |
| `hp41-core/src/ops/stat1/anova.rs` | create | group-accumulator | CRUD | `hp41-core/src/ops/stats.rs:22–53` (Σ-register accumulation discipline) | role-match (extends to per-group block; no math1 precedent) |
| `hp41-core/src/ops/stat1/regression.rs` | create | curve-fit + linear-system | CRUD + transform | `hp41-core/src/ops/stats.rs:22–53` (ΣLIN/EXP/LOGI/POW Σ delegate) + **NO math1 precedent for Gauss elimination** (locally re-derived per Req. 21) | role-match (mixed) |
| `hp41-core/src/ops/stat1/hypothesis.rs` (NOT `tests.rs`, per D-33.5) | create | accumulator + distribution-bridge | CRUD + transform | `hp41-core/src/ops/stats.rs` (Σ-register read) + `hp41-core/src/ops/stat1/distributions.rs` (Plan 33-02 primitive callee) | role-match |
| `hp41-core/src/ops/stat1/nonparam.rs` | create | accumulator + closed-form | CRUD | `hp41-core/src/ops/stats.rs:22–53` (ΣSPEAR closed-form on Σ-block) | role-match |
| `hp41-core/src/ops/stat1/rand.rs` | create | state-machine + LCG | event-driven | `hp41-core/src/ops/math.rs::op_frc` (FRC pattern) + `hp41-core/src/state.rs:163–164` (serde-default w/o skip) + `hp41-core/src/ops/math1/poly.rs:80–90` (SEED modal opener) | role-match (no direct LCG precedent — pattern composed from three sources) |
| `hp41-core/src/ops/mod.rs` | modify | enum + dispatch | — | self — math1 arm at `Op::Sinh..Op::Trans3d` (lines 589–662 for enum; lines 1152–1214 for dispatch) | exact (sibling extension) |
| `hp41-core/src/ops/program.rs` | modify | execute_op + resolver insertion | — | self — `builtin_card_op` + `xrom_resolve` at lines 74–82 (`op_xeq` site) and 532–551 (`run_loop` site) | exact (already wired — only execute_op match arms need ~24 new Stat 1 entries) |
| `hp41-core/src/ops/math1/xrom.rs` | modify (freeze exception per D-33.3) | resolver registry | — | self — `MATH_1` const at lines 43–118; `xrom_resolve` at lines 127–136; `math1_resolve` at lines 159–226 | exact (sibling `STAT_1` const + `stat1_resolve` + bit-1 arm at line 134) |
| `hp41-core/src/state.rs` | modify | serde-persisted state | — | self — `xrom_modules` field at lines 159–164 (`#[serde(default = "...")]`); `complex_mode` field at lines 169–170 (`#[serde(default)]`); tests::serde_roundtrip at lines 374–429 | exact (anchor for `rand_seed: HpNum` with `#[serde(default)]` WITHOUT `skip`) |
| `hp41-cli/src/prgm_display.rs` | modify (Phase 34 — out of Phase 33 scope) | display-name string | — | self — math1 arms at lines 239–291 | exact (sibling Σ-prefixed arms) |
| `hp41-cli/src/help_data.rs` | modify (Phase 34) | OnceLock JSON loader | — | self — `MATH1_HELP_ENTRIES` at lines 107–122; `help_entries_all()` at lines 133–135 | exact (third pool + chain) |
| `hp41-cli/src/keys.rs` (key_ref_entries filter) | modify (Phase 34, but filter already universal) | listing filter | — | self — `key_ref_entries` at lines 416–436 | exact (already filters via `entry.xrom.is_some()` — Stat 1 entries inherit policy automatically) |
| `hp41-cli/tests/function_matrix_parity.rs` | modify (Phase 34) | parity test | — | self — `MATH1_OP_VARIANT_NAMES` + 3 math1 parity tests at lines 270–398 | exact (sibling STAT1 inventory + 3 parity tests) |
| `hp41-cli/tests/key_coverage.rs` | modify (Phase 34) | coverage sweep | — | self — math1 sub-loop at lines 251–299 | exact (sibling stat1 sub-loop, filter `xrom.module_id == 2`) |
| `hp41-gui/src-tauri/src/prgm_display.rs` | modify (Phase 36) | display-name string | — | self — math1 arms at lines 260–311 (mirrors CLI prgm_display.rs by design) | exact (4-way invariant item 4) |
| `hp41-gui/src-tauri/src/key_map.rs` | modify (Phase 36) | string-ID → Op resolver | — | self — `resolve()` at lines 25–164 | exact (extends bare-id match with Stat 1 string IDs) |
| `docs/hp41-stat1-functions.json` | create (Phase 34/35) | canonical data | — | `docs/hp41-math1-functions.json` (full schema verbatim) | exact (sibling JSON pool) |
| `docs/hp41-stat1-function-matrix.md` | generated (Phase 35) | docs-matrix output | — | `docs/hp41-math1-function-matrix.md` (generator output) | exact (sibling matrix) |
| `docs/hp41-stat1-divergences.md` | create (Phase 35) | divergence sidecar | — | `docs/hp41-math1-divergences.md:1–35` (header + D-30-NN scheme) | exact (sibling sidecar) |
| `scripts/docs-matrix/src/main.rs` | modify (Phase 35) | docs generator | — | self — `render_markdown` basename-dispatch at lines 65–77 | exact (add third `else if basename.ends_with("hp41-stat1-functions.json")` arm) |
| `scripts/check-free42-contamination.sh` | modify (Plan 33-00 — STAT-QUAL-09) | contamination guard | — | self — full 38-line file | exact (extend `PATTERN` + add `STAT1_DIR`) |
| `hp41-core/tests/stat1_accuracy.rs` | create (Phase 37) | numerical-accuracy suite | — | `hp41-core/tests/numerical_accuracy.rs:1–122` (struct `AccuracyCase`, `case!` macro, `passes_with_tol`) | exact (sibling 768-case suite for Stat 1 Ops) |

---

## Pattern Assignments

Each row below carries one concrete excerpt the planner copies into the corresponding plan's action section. **No paraphrasing** — file paths + line ranges + verbatim fenced code.

---

### `hp41-core/src/ops/stat1/mod.rs` (module-hub, request-response)

**Analog:** `hp41-core/src/ops/math1/mod.rs`

**Disclaim header pattern** (lines 1–3 of every math1 file — Plan 33-00 transcribes byte-for-byte equivalent):
```rust
// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
```

**Stat 1 substitution** (verbatim header for every `ops/stat1/*.rs` file, with OM 00041-90030 swapped in):
```rust
// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
```

**Module-hub shape** (`math1/mod.rs:4–26`):
```rust
//! `math1` — XROM framework and Math Pac I (HP 00041-90034, 1979) operations.
//!
//! Module structure:
//! - `xrom`: XromModule registry, `xrom_resolve()` entry point, `MATH_1` const
//! - `modal`: ModalProgram state-machine enum for prompt-driven workflows
//! - `integ`: IntegState placeholder (Plan 28-07 fills)
//! - `solve`: SolveState placeholder (Plan 28-08 fills)
//! - `difeq`: DifeqState placeholder (Plan 28-09 fills)

pub mod complex;
pub mod difeq;
pub mod four;
pub mod hyperbolics;
pub mod integ;
pub mod matrix;
pub mod modal;
pub mod poly;
pub mod solve;
pub mod trans;
pub mod tri;
pub mod xrom;

pub use modal::ModalProgram;
```

**Stat 1 substitution** (Plan 33-00 deliverable — 10 submodules per D-33.5; `mod.rs` carries the OM "Storage Registers" verbatim transcription as the `//!` block and `pub const STAT1_MAX_REG: usize`):
```rust
//! `stat1` — HP Statistics Pac 1 (HP 00041-90030, 1979) operations.
//!
//! ## Σ Storage Register Layout (verbatim from OM 00041-90030 §"Storage Registers")
//!
//! <-- Plan 33-00 transcribes the OM table verbatim here -->
//!
//! Submodule structure:
//! - `distributions`: 3 hand-coded f64-bridge primitives (Acklam, AS 239, AS 63)
//! - `normd`:         ΣNORMD 3-mode dispatcher (CDF, PDF, inverse)
//! - `chisqd`:        ΣCHISQD ν-prompt + PDF/CDF
//! - `basic_stats`:   ΣBSTAT / ΣBSTG (univariate extended)
//! - `moments`:       ΣMMTUG / ΣMMTGD (3rd/4th moments)
//! - `anova`:         ΣAOVONE / ΣAOVTWO / ΣANOCOV
//! - `regression`:    ΣLIN/EXP/LOGI/POW + ΣMLRXY/ΣMLRXYZ + ΣPOLYP/ΣPOLYC
//! - `hypothesis`:    ΣPTST / ΣTSTAT (renamed from tests.rs per D-33.5)
//! - `nonparam`:      ΣSPEAR + ΣXSQEV/ΣEFXSQ + ΣCTKKK/ΣCTKK
//! - `rand`:          RAND / SEED (LCG; emulator extension per D-33.4)

pub mod anova;
pub mod basic_stats;
pub mod chisqd;
pub mod distributions;
pub mod hypothesis;
pub mod moments;
pub mod nonparam;
pub mod normd;
pub mod rand;
pub mod regression;

/// Highest 0-indexed Σ-register slot touched by any Stat 1 Pac accumulator.
/// Value derived from OM 00041-90030 §"Storage Registers" — see `//!` block above.
pub const STAT1_MAX_REG: usize = /* Plan 33-00 fills from OM table */;
```

**What changes for Stat 1 Pac:** sibling submodule names (no `xrom` here — `xrom.rs` STAY in `math1/`; freeze exception D-33.3 places STAT_1 const next to MATH_1 const). `STAT1_MAX_REG` const replaces the implicit R06 floor in `ops/stats.rs:25`. No `pub use modal::ModalProgram` re-export — `ModalProgram` continues to live in `math1::modal` (single carrier enum; see Pitfall 8 below).

---

### `hp41-core/src/ops/math1/xrom.rs` (FREEZE EXCEPTION per D-33.3 — extend in place)

**Analog:** self — same file. `MATH_1` const at lines 43–118 + `xrom_resolve` at lines 127–136 + `math1_resolve` at lines 159–226 are the verbatim templates.

**Bit-1 stub to replace** (lines 127–136):
```rust
pub fn xrom_resolve(name: &str, modules: u8) -> Option<Op> {
    if modules & 0b0000_0001 != 0 {
        if let Some(op) = math1_resolve(name) {
            return Some(op);
        }
    }
    // Future v3.1+ modules go here:
    // if modules & 0b0000_0010 != 0 { stat1_resolve(name) }
    None
}
```

**Stat 1 substitution** (Plan 33-01 — replace the comment with the real arm; resolver-LAST + bit-1-isolation invariants preserved):
```rust
pub fn xrom_resolve(name: &str, modules: u8) -> Option<Op> {
    if modules & 0b0000_0001 != 0 {
        if let Some(op) = math1_resolve(name) {
            return Some(op);
        }
    }
    if modules & 0b0000_0010 != 0 {
        if let Some(op) = stat1_resolve(name) {
            return Some(op);
        }
    }
    None
}
```

**`MATH_1` const template** (lines 43–46 + entry-table pattern):
```rust
pub const MATH_1: XromModule = XromModule {
    id: 7,
    name: "MATH 1A",
    ops: &[
        // ── Plan 28-02: Hyperbolics ────────────────────────────────────────────
        ("SINH", Op::Sinh),
        ("COSH", Op::Cosh),
        // ... 50 more entries; Unicode primary + ASCII alias per line 57–60 pattern ...
    ],
};
```

**Stat 1 substitution** (Plan 33-01 — sibling `STAT_1` const lives in the SAME file per D-33.3):
```rust
pub const STAT_1: XromModule = XromModule {
    id: 2,                    // HP hardware Statistics Pac 1 XROM ID per calc.fjk.ch/db/hp41mod.php
    name: "STAT 1B",          // CATALOG 2 display string per OM 00041-90030
    ops: &[
        ("\u{03A3}NORMD",  Op::SigmaNormdWorkflow),    // ΣNORMD (Σ = U+03A3)
        ("\u{03A3}CHISQD", Op::SigmaChisqdWorkflow),
        ("\u{03A3}SPEAR",  Op::SigmaSpear),
        // ... 21 more Stat 1 entries per SPEC.md "Stat 1 Pac Mnemonics" table ...
        ("RAND", Op::Rand),   // emulator extension (D-33.4)
        ("SEED", Op::Seed),   // emulator extension (D-33.4)
    ],
};
```

**`math1_resolve` match-arm pattern** (lines 159–226 — exhaustive `_ => None`):
```rust
fn math1_resolve(name: &str) -> Option<Op> {
    match name {
        "SINH" => Some(Op::Sinh),
        "COSH" => Some(Op::Cosh),
        // ... 50 more arms; Unicode + ASCII alias via | pattern (line 171 example):
        "C\u{00D7}" | "C*" => Some(Op::CTimes),
        // ...
        _ => None,
    }
}
```

**Stat 1 substitution** (Plan 33-01 — sibling `stat1_resolve` in same file):
```rust
fn stat1_resolve(name: &str) -> Option<Op> {
    match name {
        "\u{03A3}NORMD"  => Some(Op::SigmaNormdWorkflow),
        "\u{03A3}CHISQD" => Some(Op::SigmaChisqdWorkflow),
        "\u{03A3}SPEAR"  => Some(Op::SigmaSpear),
        // ... 21 more arms ...
        "RAND" => Some(Op::Rand),
        "SEED" => Some(Op::Seed),
        _ => None,
    }
}
```

**Tests to mirror** (lines 270–348 — `math1_const_id_and_name`, `math1_ops_has_correct_entry_count`, `math1_ops_mnemonics_resolve_consistently`, `resolve_uses_bit_0_only_for_math1`):
```rust
#[test]
fn math1_const_id_and_name() {
    assert_eq!(MATH_1.id, 7);
    assert_eq!(MATH_1.name, "MATH 1A");
}

#[test]
fn math1_ops_mnemonics_resolve_consistently() {
    for (name, expected_op) in MATH_1.ops {
        let resolved = xrom_resolve(name, 0b0000_0001);
        assert_eq!(resolved.as_ref(), Some(expected_op));
    }
}
```

**What changes for Stat 1 Pac:** sibling `stat1_const_id_and_name` (id 2, name "STAT 1B"), `stat1_ops_mnemonics_resolve_consistently` (loop over `STAT_1.ops` with bitmask `0b0000_0010`), and the bit-isolation triple from `resolve_uses_bit_0_only_for_math1`: assert `xrom_resolve("ΣNORMD", 0b0000_0010) == Some(...)` AND `xrom_resolve("ΣNORMD", 0b0000_0001) == None` (proves bit-1 isolation per SPEC.md Req. 2). **No structural deviation.**

---

### `hp41-core/src/ops/stat1/distributions.rs` (numerical-primitive, transform)

**Analog 1 — f64-bridge pattern:** `hp41-core/src/num.rs:184–211` (`checked_asin`, `checked_acos`, `checked_atan`):
```rust
/// asin(x) — returns result in radians. Domain error if |x| > 1.
pub fn checked_asin(&self) -> Result<HpNum, HpError> {
    let v = self.0.to_f64().ok_or(HpError::Overflow)?;
    if !(-1.0..=1.0).contains(&v) {
        return Err(HpError::Domain);
    }
    Decimal::from_f64(v.asin())
        .map(HpNum::rounded)
        .ok_or(HpError::Overflow)
}
```

**Stat 1 substitution** (Plan 33-02 — three primitives follow this exact bridge shape, but operate on bare `f64` (not `&HpNum`) and stay private to `stat1::distributions` for the inner-loop hot path; outer Op layer does the `HpNum → f64 → HpNum` round-trip):
```rust
/// Inverse standard-normal CDF (Acklam / AS 241, ~30 LOC rational approximation).
/// Returns `Ok(x)` such that Φ(x) = p, for p ∈ (0, 1).
pub fn norm_cdf_inv_f64(p: f64) -> Result<f64, HpError> {
    if !(0.0..=1.0).contains(&p) {
        return Err(HpError::Domain);
    }
    if p == 0.0 { return Err(HpError::Domain); }  // asymptote — see Plan 33-02 OM read for final behavior
    if p == 1.0 { return Err(HpError::Domain); }
    // ... Acklam rational approximation, 21 coefficients verbatim from
    //     stackedboxes.org/2017/05/01/acklams-normal-quantile-function/
    Ok(/* computed */)
}
```

**Analog 2 — inline scipy oracle constants:** RESEARCH.md §"Distribution Primitives" Table (Q.7) confirms the math1/poly.rs and math1/integ.rs precedent — `(input, scipy_expected, tolerance)` tuples in `#[cfg(test)] mod tests`. **Pattern shape** (D-33.6 — every oracle tuple line carries the verbatim Python comment above it):
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn norm_cdf_inv_central() {
        // scipy.stats.norm.ppf(0.025) = -1.959963984540054
        assert_relative_eq!(norm_cdf_inv_f64(0.025).unwrap(), -1.959963984540054, max_relative = 1e-9);
        // scipy.stats.norm.ppf(0.975) = 1.959963984540054
        assert_relative_eq!(norm_cdf_inv_f64(0.975).unwrap(), 1.959963984540054, max_relative = 1e-9);
        // scipy.stats.norm.ppf(0.5) = 0.0
        assert_relative_eq!(norm_cdf_inv_f64(0.5).unwrap(), 0.0, epsilon = 1e-12);
    }
    // ... ≥ 6 tuples per primitive per SPEC.md Req. 33 ...
}
```

**What changes for Stat 1 Pac:** primitives return `Result<f64, HpError>` not `Result<HpNum, HpError>` (the outer Op layer in `normd.rs` / `chisqd.rs` / `hypothesis.rs` wraps the bridge). All three primitives must pass GREEN before Plan 33-03 lands the first Op (SPEC.md Acceptance criterion).

---

### `hp41-core/src/ops/stat1/normd.rs` (mode-dispatcher + iterative, request-response)

**Analog 1 — modal opener:** `hp41-core/src/ops/math1/poly.rs:80–90` (`op_poly_workflow`):
```rust
/// POLY — polynomial root-finder master entry.
///
/// Opens the modal workflow: sets `state.modal_program = Some(ModalProgram::Poly(DegreePrompt))`
/// and `state.modal_prompt = Some("DEGREE=?")`.
pub fn op_poly_workflow(state: &mut CalcState) -> Result<(), HpError> {
    state.modal_program = Some(ModalProgram::Poly(PolyInputStep::DegreePrompt));
    state.modal_prompt = Some("DEGREE=?".to_string());
    state.print_buffer.push("DEGREE=?".to_string());
    Ok(())
}
```

**Stat 1 substitution** (Plan 33-03 — ΣNORMD mode dispatcher; tentative `MODE?` prompt to be confirmed in plan):
```rust
pub fn op_sigma_normd_workflow(state: &mut CalcState) -> Result<(), HpError> {
    state.modal_program = Some(ModalProgram::Stat1(Stat1Step::NormdModeChoice));
    state.modal_prompt = Some("\u{03A3}NORMD MODE?".to_string());
    state.print_buffer.push("\u{03A3}NORMD MODE?".to_string());
    Ok(())
}
```

**Analog 2 — display-mode-tied tolerance:** `hp41-core/src/ops/math1/integ.rs:104–111` (`integ_threshold`):
```rust
pub fn integ_threshold(mode: DisplayMode) -> f64 {
    let decimals = match mode {
        DisplayMode::Fix(n) | DisplayMode::Sci(n) | DisplayMode::Eng(n) => n as i32,
    };
    // threshold = 5 × 10^(-(decimals + 1))
    5.0_f64 * 10.0_f64.powi(-(decimals + 1))
}
```

**Stat 1 substitution** (Plan 33-03 — `quantile_threshold` differs by factor 5 per SPEC.md Req. 34: `10^(-FIX_decimals − 1)`, fallback `1e-10`):
```rust
pub fn quantile_threshold(mode: DisplayMode) -> f64 {
    let decimals = match mode {
        DisplayMode::Fix(n) | DisplayMode::Sci(n) | DisplayMode::Eng(n) => n as i32,
    };
    // SPEC Req. 34: 10^(-decimals - 1); falls back to 1e-10 when display is not FIX
    10.0_f64.powi(-(decimals + 1))
}

pub const QUANTILE_MAX_ITERS: u32 = 50;  // SPEC Req. 34 iteration cap
```

**Analog 3 — cancellation check inside iterative loop:** `hp41-core/src/ops/math1/integ.rs` uses `std::sync::atomic::Ordering` at line 28; RESEARCH.md §"Code Examples" gives the verbatim pattern:
```rust
use std::sync::atomic::Ordering;
// ... inside iteration loop:
if state.cancel_requested.load(Ordering::Relaxed) {
    return Err(HpError::Cancelled);
}
```

**Stat 1 substitution** (Plan 33-03 — every iteration of the Newton-bisection inverse loop):
```rust
for _ in 0..QUANTILE_MAX_ITERS {
    if state.cancel_requested.load(Ordering::Relaxed) {
        return Err(HpError::Cancelled);
    }
    // Newton/bisection step using norm_cdf_inv_f64 + rust_decimal::norm_cdf for residual
    // ...
}
return Err(HpError::ConvergenceFailed);  // hard cap per SPEC Req. 34
```

**What changes for Stat 1 Pac:** distinct mode dispatch (E=CDF, C=PDF, A=inverse per Acceptance Req. 31 tentative); inverse path uses `norm_cdf_inv_f64` from Plan 33-02 + bisection refinement for tail-edge cases. PDF + CDF use `rust_decimal::MathematicalOps::norm_cdf` / `norm_pdf` (closed-form, no iteration, no cancellation hook).

---

### `hp41-core/src/ops/stat1/chisqd.rs` (mode-dispatcher + iterative, request-response)

**Analog 1 — modal opener:** same as `normd.rs` above, `hp41-core/src/ops/math1/poly.rs:80–90` (`op_poly_workflow` template).
**Analog 2 — iterative tolerance + cancellation:** same as `normd.rs`, `hp41-core/src/ops/math1/integ.rs:104–111` + `Ordering::Relaxed` loop.

**Stat 1 substitution** (Plan 33-03 — ΣCHISQD opens `ν=?` ALPHA prompt; mode selection happens after):
```rust
pub fn op_sigma_chisqd_workflow(state: &mut CalcState) -> Result<(), HpError> {
    state.modal_program = Some(ModalProgram::Stat1(Stat1Step::ChisqdNuPrompt));
    state.modal_prompt = Some("\u{03BD}=?".to_string());  // ν=?
    state.print_buffer.push("\u{03BD}=?".to_string());
    Ok(())
}
```

**Numerical path:** CDF P(x; ν) = `gamma_regularized_f64(ν/2.0, x/2.0)` from Plan 33-02. PDF closed-form via `rust_decimal::MathematicalOps::exp` + `ln Γ` (Stirling). Acceptance Req. 32: PDF f(7.815; ν=3) within 1e-9; CDF P(7.815; ν=3) ≈ 0.95 within 1e-7.

---

### `hp41-core/src/ops/stat1/basic_stats.rs` (accumulator-consumer, CRUD)

**Analog:** `hp41-core/src/ops/stats.rs:22–53` (`op_sigma_plus` — Σ-register R01–R06 accumulation discipline):
```rust
pub fn op_sigma_plus(state: &mut CalcState) -> Result<(), HpError> {
    // Phase 22 D-22.11.1 / Pitfall 5: fail-closed when Σ block R01..R06
    // unaddressable under SIZE shrink.
    if state.regs.len() < 7 {
        return Err(HpError::InvalidOp);
    }
    let x = state.stack.x.clone();
    let y = state.stack.y.clone();

    // Accumulate — compute each term atomically before writing (Pitfall guard)
    let new_r1 = state.regs[1].checked_add(&x.checked_sq()?)?; // Σx² += x²
    // ... 5 more accumulations ...
    state.regs[1] = new_r1;
    // ... writes atomic after all computations succeed ...

    state.stack.lift_enabled = true;
    enter_number(state, new_r3);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
```

**Stat 1 substitution** (Plan 33-05 — ΣBSTAT consumes the SAME R01–R06 block; never accumulates, only reads):
```rust
pub fn op_sigma_bstat(state: &mut CalcState) -> Result<(), HpError> {
    // Same SIZE-floor guard as op_sigma_plus — consumes R01–R06 read-only.
    if state.regs.len() < 7 {
        return Err(HpError::InvalidOp);
    }
    let n         = state.regs[3].clone();
    let sum_x     = state.regs[2].clone();
    let sum_x_sq  = state.regs[1].clone();
    let sum_y     = state.regs[5].clone();
    // ... compute weighted mean + CV per OM 00041-90030 §"Results" ...
    // Push results per OM-specified output channel (stack vs print_buffer per Plan 33-05 OM read)
    Ok(())
}
```

**What changes for Stat 1 Pac:** ΣBSTAT / ΣBSTG never write to R01–R06 (read-only over existing accumulator state). For ΣLIN/EXP/LOGI/POW, the **`op_sigma_exp_accumulate` delegate pattern** from RESEARCH.md §"Code Examples" applies — apply ln transform on Y/X then DELEGATE to `crate::ops::stats::op_sigma_plus`. NEVER duplicate Σ arithmetic.

---

### `hp41-core/src/ops/stat1/moments.rs` (accumulator-extended, CRUD)

**Analog:** `hp41-core/src/ops/stats.rs:22–53` (Σ-register accumulation discipline) + `hp41-core/src/ops/stat1/mod.rs::STAT1_MAX_REG` (Plan 33-00 transcription).

**Stat 1 substitution** (Plan 33-06 — ΣMMTUG accumulates `Σx³`, `Σx⁴` into extended OM register block; index names from Plan 33-00 transcription):
```rust
pub fn op_sigma_mmtug_accumulate(state: &mut CalcState) -> Result<(), HpError> {
    // SIZE-floor guard against the Stat 1 max-reg const, NOT the v1.x R01–R06 floor.
    if state.regs.len() < crate::ops::stat1::STAT1_MAX_REG + 1 {
        return Err(HpError::InvalidOp);
    }
    let x = state.stack.x.clone();
    // Compute Σx³, Σx⁴ atomically before write — same atomic discipline as op_sigma_plus.
    let new_r_cube = state.regs[STAT1_MMTUG_CUBE_REG]
        .checked_add(&x.checked_mul(&x.checked_sq()?)?)?;
    let new_r_quad = state.regs[STAT1_MMTUG_QUAD_REG]
        .checked_add(&x.checked_sq()?.checked_sq()?)?;
    state.regs[STAT1_MMTUG_CUBE_REG] = new_r_cube;
    state.regs[STAT1_MMTUG_QUAD_REG] = new_r_quad;
    // Delegate Σx, Σx², n updates to op_sigma_plus to avoid duplication.
    crate::ops::stats::op_sigma_plus(state)
}
```

**`[C]` correction-key extension** (STAT-UNI-04, Req. 10): `hp41-core/src/ops/stats.rs:58–85` (`op_sigma_minus`) MUST be extended in Plan 33-05/06 to mirror every new register touched by ΣMMTUG accumulation. Plan 33-06 acceptance test extends the round-trip `sigma_plus_then_minus_restores_state` from `hp41-core/tests/`.

**What changes for Stat 1 Pac:** all register indices MUST be named consts derived from Plan 33-00's `STAT1_*_REG` block — Pitfall 1/P21 mitigation. No literal-integer register indices in `stat1/anova.rs`, `stat1/moments.rs`, `stat1/regression.rs`, `stat1/nonparam.rs` (code-review acceptance gate, RESEARCH.md Pitfall 1).

---

### `hp41-core/src/ops/stat1/anova.rs` (group-accumulator, CRUD)

**Analog:** `hp41-core/src/ops/stats.rs:22–53` (Σ-register accumulation discipline) — no direct math1 precedent for grouped accumulators; this is the closest pattern.

**Stat 1 substitution** (Plan 33-06 — ANOVA Ops read group-keyed Σ blocks per OM transcription):
```rust
pub fn op_sigma_aovone(state: &mut CalcState) -> Result<(), HpError> {
    if state.regs.len() < crate::ops::stat1::STAT1_MAX_REG + 1 {
        return Err(HpError::InvalidOp);
    }
    // Read per-group accumulators from OM-transcribed register layout (Plan 33-00).
    // Compute SSB, SSW, F = (SSB/dfB) / (SSW/dfW).
    // Push F per OM "Results" — Plan 33-06 OM read decides stack vs print_buffer.
    // ...
    Ok(())
}
```

**What changes for Stat 1 Pac:** F-ratio output channel deferred to Plan 33-06 OM read (per CONTEXT.md Claude's Discretion). All register indices use named consts from `stat1::mod::STAT1_*_REG`. Acceptance Req. 11: 3-groups × 5-samples oracle yields F = 100.0 within 1e-9.

---

### `hp41-core/src/ops/stat1/regression.rs` (curve-fit + linear-system, CRUD + transform)

**Analog 1 — Σ-block delegate for ΣLIN/EXP/LOGI/POW:** `hp41-core/src/ops/stats.rs:22–53` (`op_sigma_plus` delegate). RESEARCH.md §"Pattern 4":
```rust
// Source: pattern derived from hp41-core/src/ops/stats.rs:22 (op_sigma_plus)
pub fn op_sigma_exp_accumulate(state: &mut CalcState) -> Result<(), HpError> {
    // ΣEXP: ŷ = a·e^(b·x) → accumulate (x, ln y)
    let y = state.stack.y.clone();
    let ln_y = y.checked_ln()?;  // existing rust_decimal-backed checked_ln on HpNum
    state.stack.y = ln_y;        // transform Y in place
    crate::ops::stats::op_sigma_plus(state)  // delegate to existing v1.x Σ+
}
```

**Analog 2 — self-contained Gauss elimination:** **NO direct math1 precedent.** Math Pac I `MATRIX/SIMEQ` is explicitly REJECTED for reuse per SPEC.md Req. 21. Pattern composed inline per RESEARCH.md §"Regression Family":
```rust
// Pseudocode — Plan 33-08
fn solve_normal_equations_2x2(
    s_x1_sq: HpNum, s_x1_x2: HpNum, s_x1: HpNum,
    s_x2_sq: HpNum, s_x2: HpNum, n: HpNum,
    s_x1_y: HpNum, s_x2_y: HpNum, s_y: HpNum,
) -> Result<(HpNum, HpNum, HpNum), HpError> {
    // Standard Gauss elimination with partial pivoting on 3×3 augmented matrix.
    // All arithmetic in HpNum (rust_decimal-backed). Returns (b₀, b₁, b₂) or
    // Err(HpError::Domain) if matrix is singular.
}
```

**CI gate** (Req. 21 acceptance — Plan 33-08 task and Phase 37 retest):
```bash
grep -n "ops::math1::matrix" hp41-core/src/ops/stat1/   # MUST be empty
```

**What changes for Stat 1 Pac:** ΣMLRXY (2-predictor → 3×3 system) and ΣMLRXYZ (3-predictor → 4×4 system) and ΣPOLYP (d-degree → (d+1)×(d+1) system) all use LOCAL Gauss elimination. ΣPOLYP additionally opens a `DEGREE=?` modal — pattern from `math1/poly.rs:80–90` (modal opener) + SEE Pitfall 8 about `ModalProgram::Stat1` extension.

---

### `hp41-core/src/ops/stat1/hypothesis.rs` (NOT `tests.rs`, per D-33.5) (accumulator + distribution-bridge, CRUD + transform)

**Analog:** `hp41-core/src/ops/stats.rs:22–53` (Σ-register read) + `hp41-core/src/ops/stat1/distributions.rs::beta_regularized_f64` (Plan 33-02 callee).

**Stat 1 substitution** (Plan 33-07 — ΣPTST one-sample t-test):
```rust
pub fn op_sigma_ptst(state: &mut CalcState) -> Result<(), HpError> {
    if state.regs.len() < 7 {
        return Err(HpError::InvalidOp);
    }
    let n = state.regs[3].clone();
    let sum_x = state.regs[2].clone();
    let sum_x_sq = state.regs[1].clone();
    let mu_0 = state.stack.x.clone();  // hypothesized mean from stack
    // Compute t = (x̄ − μ₀) / (s / √n), df = n − 1
    // Bridge through distributions::beta_regularized_f64 for two-sided p-value
    // ...
    Ok(())
}
```

**What changes for Stat 1 Pac:** distinct filename per D-33.5 (avoids `#[cfg(test)] mod tests` collision with the per-file unit-test blocks). ΣTSTAT is pooled-variance only (SPEC.md Req. 25) — Welch EXCLUDED.

---

### `hp41-core/src/ops/stat1/nonparam.rs` (accumulator + closed-form, CRUD)

**Analog:** `hp41-core/src/ops/stats.rs:22–53` (Σ-register read pattern, no distribution function — pure arithmetic on existing R01–R06).

**Stat 1 substitution** (Plan 33-04 — ΣSPEAR closed-form, simplest of all 13 programs):
```rust
pub fn op_sigma_spear(state: &mut CalcState) -> Result<(), HpError> {
    if state.regs.len() < 7 {
        return Err(HpError::InvalidOp);
    }
    // ρ_s = 1 − 6·Σd² / (n·(n²−1))
    // d² accumulated by user via Σ+ on (rank_x − rank_y)² prior to ΣSPEAR call.
    // ...
    Ok(())
}
```

**Open question carried forward (RESEARCH.md Open Q 4 / A8):** SPEC.md says ρ_s = 0.7 for `ranks_x=[1,2,3,4,5], ranks_y=[2,1,3,5,4]` but scipy yields 0.8. Plan 33-04 verifies and corrects SPEC.md if needed.

---

### `hp41-core/src/ops/stat1/rand.rs` (state-machine + LCG, event-driven)

**Analog 1 — FRC primitive:** `hp41-core/src/ops/math.rs::op_frc` (precedent for `FRC(x) = x − int(x)`); RESEARCH.md §"RAND / SEED" gives the exact LCG body.

**Analog 2 — serde-default-without-skip field:** `hp41-core/src/state.rs:169–170` (`complex_mode`):
```rust
/// Complex stack overlay mode (D-28.1 / D-28.2). When true, X+iY form
/// the complex number ζ and Z+iT form τ. Auto-on at first complex op;
/// explicit `XEQ "REAL"` (D-28.3) deactivates. Safe default: false.
#[serde(default)]
pub complex_mode: bool,
```

**Stat 1 substitution** (Plan 33-01 — `rand_seed: HpNum` lands in `state.rs` with the same serde shape; doc-comment SHOUTS the unique shape per P20 trap mitigation):
```rust
/// RNG seed for RAND/SEED Op family. Persistent across save/load so
/// reproducible simulations work across sessions (D-33.4 + STAT-RNG-03).
/// SERDE SHAPE: `#[serde(default)]` WITHOUT `skip`. This is the ONLY new
/// v3.1 CalcState field with this combination. All other transient v3.1
/// fields (if any are added) use `#[serde(default, skip)]`.
/// P20 muscle-memory trap: `#[serde(skip)]` would break determinism across
/// save/load round-trips and silently fail STAT-RNG-03 acceptance.
#[serde(default)]
pub rand_seed: HpNum,
```

**Analog 3 — SEED modal opener:** same as `math1/poly.rs:80–90` (`op_poly_workflow` template).

**Stat 1 substitution** (Plan 33-08 — SEED opens `SEED?` ALPHA prompt; RAND uses `state.rand_seed` directly):
```rust
pub fn op_rand(state: &mut CalcState) -> Result<(), HpError> {
    let multiplier = HpNum::from_str("9821").map_err(|_| HpError::Domain)?;
    let increment  = HpNum::from_str("0.211327").map_err(|_| HpError::Domain)?;
    let stepped    = state.rand_seed.checked_mul(&multiplier)?
                                    .checked_add(&increment)?;
    // FRC: x − trunc(x). Use HpNum::trunc_int from num.rs:222 (or add checked_int helper).
    let new_seed = stepped.checked_sub(&stepped.trunc_int())?;
    state.rand_seed = new_seed.clone();
    state.stack.lift_enabled = true;
    enter_number(state, new_seed);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
```

**What changes for Stat 1 Pac:** RAND/SEED ship even if OM does not list them as ROM entries (D-33.4 emulator-extension policy). `rand_seed` is the SOLE new v3.1 field with `#[serde(default)]` WITHOUT `#[serde(skip)]` — see §Shared Patterns "Serde discipline" below.

---

### `hp41-core/src/state.rs` (modify — `rand_seed` field + `migrate_after_load()`)

**Analog:** self — `xrom_modules` field at lines 163–164 + `default_xrom_modules` helper at lines 228–230 + serde_roundtrip test at lines 374–429.

**`xrom_modules` field pattern with default helper** (lines 159–164 + 228–230):
```rust
// ── Phase 28 (v3.0): XROM framework + Math Pac I ────────────────────────
/// Bitfield of loaded XROM modules. Bit 0 = Math 1 loaded.
/// Default: 0b0000_0001 (Math 1 pre-loaded per v3.0 scope).
/// Persistent across save/load. `#[serde(default = "default_xrom_modules")]`.
#[serde(default = "default_xrom_modules")]
pub xrom_modules: u8,
// ...
/// Default value for `xrom_modules`: bit 0 = Math 1 pre-loaded.
fn default_xrom_modules() -> u8 {
    0b0000_0001
}
```

**Stat 1 substitution** (Plan 33-01 — flip to `0b0000_0011`; add migration method; add `rand_seed`):
```rust
/// Default value for `xrom_modules`: bit 0 = Math 1 + bit 1 = Stat 1, both pre-loaded.
fn default_xrom_modules() -> u8 {
    0b0000_0011
}

impl CalcState {
    /// Apply post-deserialization migrations.
    ///
    /// Called once after every deserialize() by:
    /// - `hp41-cli/src/persistence.rs::load_state` (Phase 34 wiring)
    /// - `hp41-gui/src-tauri/src/persistence.rs` (Phase 36 wiring)
    ///
    /// Idempotent — safe to call multiple times.
    pub fn migrate_after_load(&mut self) {
        // v3.0 → v3.1: set STAT_1 bit (bit 1) if previously absent.
        if self.xrom_modules & 0b0000_0010 == 0 {
            self.xrom_modules |= 0b0000_0010;
        }
    }
}
```

**serde-roundtrip test pattern** (lines 374–429 — already asserts `xrom_modules` round-trips through serde):
```rust
#[test]
fn serde_roundtrip() {
    let mut state = CalcState::new();
    state.xrom_modules = 0b0000_0011;
    // ...
    let json = serde_json::to_string(&state).unwrap();
    assert!(json.contains("xrom_modules"), "xrom_modules must be serialized");
    let restored: CalcState = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.xrom_modules, 0b0000_0011);
}
```

**Stat 1 substitution** (Plan 33-01 — sibling tests per RESEARCH.md §"State Migration & RNG Seed Persistence"):
```rust
#[test]
fn v3_0_save_loads_with_stat_1_after_migration() {
    let v30_blob = r#"{"xrom_modules": 1, /* ... other fields ... */}"#;
    let mut state: CalcState = serde_json::from_str(v30_blob).unwrap();
    assert_eq!(state.xrom_modules, 0b0000_0001);  // pre-migration
    state.migrate_after_load();
    assert_eq!(state.xrom_modules, 0b0000_0011);  // post-migration
}

#[test]
fn rand_seed_serde_round_trip() {
    let mut state = CalcState::new();
    state.rand_seed = HpNum::from_str("0.7").unwrap();
    let blob = serde_json::to_string(&state).unwrap();
    let restored: CalcState = serde_json::from_str(&blob).unwrap();
    assert_eq!(restored.rand_seed, state.rand_seed);  // proves NOT skipped
}
```

**What changes for Stat 1 Pac:** the `default_xrom_modules` value flips `0b0000_0001 → 0b0000_0011`; `migrate_after_load()` is NEW; `rand_seed: HpNum` is a sibling persistent field. CLI + GUI wiring sites for `migrate_after_load()` are out of Phase 33 (Phase 34 + 36).

---

### `hp41-core/src/ops/mod.rs` (modify — Op enum + dispatch)

**Analog:** self — math1 Op variants at lines 589–662 (enum) + dispatch arms at lines 1152–1214.

**Enum pattern** (lines 589–597, math1 hyperbolics):
```rust
// ── Phase 28: Hyperbolics (Plan 28-02) ────────────────────────────────────
/// SINH — hyperbolic sine. Angle-mode-independent. LiftEffect: Enable.
/// XROM Math Pac I (HP 00041-90034). No domain restriction; Overflow for extreme magnitudes.
Sinh,
/// COSH — hyperbolic cosine. Angle-mode-independent. LiftEffect: Enable.
/// XROM Math Pac I (HP 00041-90034). No domain restriction; Overflow for extreme magnitudes.
Cosh,
```

**Stat 1 substitution** (Plans 33-03..33-08 incrementally append; each variant carries OM citation in doc-comment per RESEARCH.md §"Module Wiring"):
```rust
// ── Phase 33: Stat 1 Pac (Plan 33-03 onward) ────────────────────────────
/// ΣNORMD master entry — opens 3-mode dispatcher (CDF/PDF/inverse).
/// XROM Stat 1 Pac (HP 00041-90030). LiftEffect: Neutral (modal opener).
SigmaNormdWorkflow,
/// ΣCHISQD master entry — opens ν-prompt then PDF/CDF dispatcher.
/// XROM Stat 1 Pac (HP 00041-90030). LiftEffect: Neutral (modal opener).
SigmaChisqdWorkflow,
// ... 22 more variants per SPEC.md "Stat 1 Pac Mnemonics" table ...
```

**Dispatch arm pattern** (lines 1152–1158, math1 hyperbolics):
```rust
// ── Phase 28: Hyperbolics (Plan 28-02) ────────────────────────────────────
Op::Sinh => op_sinh(state),
Op::Cosh => op_cosh(state),
Op::Tanh => op_tanh(state),
Op::Asinh => op_asinh(state),
Op::Acosh => op_acosh(state),
Op::Atanh => op_atanh(state),
```

**Stat 1 substitution** (Plans 33-03..33-08 — each new variant lands an exhaustive arm; NO `_ =>` catch-all permitted, SPEC.md Req. 5):
```rust
// ── Phase 33: Stat 1 Pac (Plan 33-03 onward) ────────────────────────────
Op::SigmaNormdWorkflow => stat1::normd::op_sigma_normd_workflow(state),
Op::SigmaChisqdWorkflow => stat1::chisqd::op_sigma_chisqd_workflow(state),
// ... 22 more arms ...
```

**`pub mod stat1` insertion** (Plan 33-01 — single line added to `hp41-core/src/ops/mod.rs` near the existing `pub mod math1;` declaration; mirrors the math1 mounting site).

**What changes for Stat 1 Pac:** items 1 + 2 of the 4-way exhaustive-match invariant — see §Shared Patterns "4-way invariant" for the full enforcement chain.

---

### `hp41-core/src/ops/program.rs` (modify — execute_op arms; resolver already wired)

**Analog:** self — `xrom_resolve` already plugged in at both insertion sites; only execute_op match arms need ~24 new entries.

**Insertion site 1 — `op_xeq`** (lines 74–82, ALREADY WIRED for Stat 1 via `state.xrom_modules`):
```rust
if let Some(card_op) = builtin_card_op(label) {
    return crate::ops::dispatch(state, card_op);
}
// Phase 28 (v3.0) XROM resolver — fires LAST (C-28.4 / Pitfall 1).
// Checked after builtin_card_op so built-in names always win.
if let Some(xrom_op) = crate::ops::math1::xrom::xrom_resolve(label, state.xrom_modules) {
    return crate::ops::dispatch(state, xrom_op);
}
return Err(HpError::InvalidOp);
```

**Insertion site 2 — `run_loop` (Op::Xeq runtime path)** (lines 532–551, ALREADY WIRED):
```rust
if let Some(card_op) = builtin_card_op(&label) {
    crate::ops::dispatch(state, card_op)?;
} else if let Some(xrom_op) =
    crate::ops::math1::xrom::xrom_resolve(&label, state.xrom_modules)
{
    crate::ops::dispatch(state, xrom_op)?;
} else {
    return Err(HpError::InvalidOp);
}
```

**execute_op arms to add:** mirror the dispatch arms above (Plans 33-03..33-08). Pattern from RESEARCH.md §"4-Way Exhaustive-Match — Items 1 + 2":
```rust
// In execute_op() match block — same shape as math1 arms ~line 900 in program.rs:
Op::SigmaNormdWorkflow => stat1::normd::op_sigma_normd_workflow(state),
Op::SigmaChisqdWorkflow => stat1::chisqd::op_sigma_chisqd_workflow(state),
// ... 22 more arms ...
```

**What changes for Stat 1 Pac:** NO changes to `op_xeq` or `run_loop` resolver insertion sites — both fire `xrom_resolve` with `state.xrom_modules` which now defaults to `0b0000_0011`. ONLY execute_op match arms grow.

---

### `hp41-cli/src/prgm_display.rs` (PHASE 34 — not Phase 33)

**Analog:** self — math1 arms at lines 239–291.

**Pattern** (lines 264–266, MATRIX block as example — Σ-name strings analog):
```rust
// ── Phase 28: POLY / ROOTS (Plan 28-05) ────────────────────────────────────
Op::PolyWorkflow => "POLY".to_string(),
Op::Roots => "ROOTS".to_string(),
// ── Phase 28: MATRIX (Plan 28-06) ────────────────────────────────────────
Op::MatrixWorkflow => "MATRIX".to_string(),
Op::MatSize => "SIZE".to_string(),
```

**Stat 1 substitution** (Phase 34 — item 3 of 4-way invariant; mirrors GUI prgm_display.rs):
```rust
// ── Phase 33: Stat 1 Pac (XROM 2) ────────────────────────────────────────
Op::SigmaNormdWorkflow => "\u{03A3}NORMD".to_string(),
Op::SigmaChisqdWorkflow => "\u{03A3}CHISQD".to_string(),
// ... 22 more arms; Σ = U+03A3 ...
```

**What changes for Stat 1 Pac:** Phase 34 work (compile-time enforcement guarantees Phase 33 `cargo check -p hp41-cli` FAILS until item 3 lands — expected per SPEC.md Acceptance Req. 5 + RESEARCH.md §"4-Way Exhaustive-Match" table).

---

### `hp41-cli/src/help_data.rs` (PHASE 34)

**Analog:** self — `MATH1_HELP_ENTRIES` at lines 102–122 + `help_entries_all` chain at lines 133–135.

**OnceLock pattern** (lines 102–122):
```rust
/// Compile-time-embedded canonical data file for Math Pac I (D-29.1 / D-29.2).
/// The relative path is from `hp41-cli/src/help_data.rs` to
/// `docs/hp41-math1-functions.json` at the repo root.
const MATH1_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-math1-functions.json");

static MATH1_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

pub fn help_entries_math1() -> &'static [HelpEntry] {
    MATH1_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(MATH1_FUNCTIONS_JSON)
            .expect("hp41-math1-functions.json is malformed — fix the JSON")
    })
}
```

**Chain pattern** (lines 133–135):
```rust
pub fn help_entries_all() -> impl Iterator<Item = &'static HelpEntry> {
    help_entries().iter().chain(help_entries_math1().iter())
}
```

**Stat 1 substitution** (Phase 34 — third pool + extended chain):
```rust
const STAT1_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-stat1-functions.json");
static STAT1_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

pub fn help_entries_stat1() -> &'static [HelpEntry] {
    STAT1_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(STAT1_FUNCTIONS_JSON)
            .expect("hp41-stat1-functions.json is malformed — fix the JSON")
    })
}

pub fn help_entries_all() -> impl Iterator<Item = &'static HelpEntry> {
    help_entries().iter()
        .chain(help_entries_math1().iter())
        .chain(help_entries_stat1().iter())
}
```

**What changes for Stat 1 Pac:** Phase 34 work. **The right-panel filter `key_ref_entries` at `hp41-cli/src/keys.rs:416–436` (lines below) is UNIVERSAL — it filters `entry.xrom.is_some()` regardless of module, so Stat 1 entries are AUTOMATICALLY excluded from the right panel and only discoverable via `?` overlay. No code change to `key_ref_entries` is needed.**

---

### `hp41-cli/src/keys.rs` — `key_ref_entries` filter (no change needed)

**Analog:** self — `key_ref_entries` at lines 416–436.

**Pattern** (the line 422 filter is universal):
```rust
pub fn key_ref_entries() -> Vec<(String, String)> {
    let mut seen: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    for entry in crate::help_data::help_entries_all() {
        if entry.status != "implemented" {
            continue;
        }
        if entry.xrom.is_some() {
            // Skip XROM-module functions — they live in the `?` overlay,
            // not in the right-panel keyboard reference.
            continue;
        }
        // ...
    }
    seen.into_iter().collect()
}
```

**What changes for Stat 1 Pac:** **NOTHING.** Stat 1 entries in `docs/hp41-stat1-functions.json` will carry `xrom: { module: "Stat 1", module_id: 2, function_id: N }` — the existing `entry.xrom.is_some()` check excludes them automatically. Documented in RESEARCH.md §"JSON Registry" line 924: "post-v3.0 UX revert — module functions stay discoverable via `?` overlay's 'Stat 1 Pac (XROM 2)' section". Plan-phase note: only the `help_entries_all()` chain must be extended (Phase 34 work).

---

### `hp41-cli/tests/function_matrix_parity.rs` (PHASE 34)

**Analog:** self — `MATH1_OP_VARIANT_NAMES` at lines 285–340 + 3 math1 parity tests at lines 342–398.

**Inventory const pattern** (lines 285–340):
```rust
const MATH1_OP_VARIANT_NAMES: &[&str] = &[
    // Phase 28-02: Hyperbolics (6)
    "Sinh", "Cosh", "Tanh", "Asinh", "Acosh", "Atanh",
    // Phase 28-03: Complex Stack Arithmetic (5)
    "CPlus", "CMinus", "CTimes", "CDiv", "Real",
    // ... 45 total entries ...
];

#[test]
fn test_math1_op_inventory_count() {
    assert_eq!(MATH1_OP_VARIANT_NAMES.len(), 45, "...");
}

#[test]
fn test_every_math1_rom_op_has_math1_json_entry() {
    let json_variants: HashSet<&str> = help_entries_math1()
        .iter().map(|e| e.op_variant.as_str()).collect();
    // ... assert every name in MATH1_OP_VARIANT_NAMES is in json_variants ...
}

#[test]
fn test_every_math1_json_entry_has_xrom_resolver_match() {
    for entry in help_entries_math1() {
        let resolved = hp41_core::ops::math1::xrom::xrom_resolve(
            entry.display_name.as_str(), 0b0000_0001);
        assert!(resolved.is_some(), "...");
    }
}
```

**Stat 1 substitution** (Phase 34 — sibling STAT1 inventory const + 3 parity tests with bitmask `0b0000_0010`):
```rust
const STAT1_OP_VARIANT_NAMES: &[&str] = &[
    "SigmaNormdWorkflow", "SigmaChisqdWorkflow", "SigmaSpear",
    // ... ~24 entries ...
    "Rand", "Seed",
];

#[test]
fn test_stat1_op_inventory_count() {
    assert_eq!(STAT1_OP_VARIANT_NAMES.len(), 24 /* or final count */);
}

#[test]
fn test_every_stat1_rom_op_has_stat1_json_entry() { /* mirror */ }

#[test]
fn test_every_stat1_json_entry_has_xrom_resolver_match() {
    for entry in help_entries_stat1() {
        let resolved = hp41_core::ops::math1::xrom::xrom_resolve(
            entry.display_name.as_str(), 0b0000_0010);  // bit 1 for STAT_1
        assert!(resolved.is_some());
    }
}
```

**What changes for Stat 1 Pac:** Phase 34 work; identical shape to math1 tests but bitmask `0b0000_0010`.

---

### `hp41-cli/tests/key_coverage.rs` (PHASE 34)

**Analog:** self — math1 sub-loop at lines 251–299.

**Pattern** (lines 256–299):
```rust
let mut math1_probed = 0usize;
for entry in entries.iter() {
    if entry.status != "implemented" { continue; }
    let Some(xrom) = entry.xrom.as_ref() else { continue; };
    // xrom.module_id is the HARDWARE module ID (7 for HP Math Pac I)
    if xrom.module_id != 7 { continue; }
    let Some(key_path) = entry.key_path.as_deref() else { continue; };
    let Some(rest) = key_path.strip_prefix("XEQ \"") else { continue; };
    let Some(name) = rest.strip_suffix('"') else { continue; };
    math1_probed += 1;
    let resolved = xeq_by_name_local_resolve(name, 0b0000_0001);
    assert!(resolved.is_some(), "...");
}
assert!(math1_probed >= 40, "...");
```

**Stat 1 substitution** (Phase 34 — sibling sub-loop, filter `xrom.module_id == 2`, bitmask `0b0000_0010`):
```rust
let mut stat1_probed = 0usize;
for entry in entries.iter() {
    if entry.status != "implemented" { continue; }
    let Some(xrom) = entry.xrom.as_ref() else { continue; };
    if xrom.module_id != 2 { continue; }  // Stat 1 Pac
    // ... same shape as math1 sub-loop ...
    let resolved = xeq_by_name_local_resolve(name, 0b0000_0010);  // bit 1
    assert!(resolved.is_some());
    stat1_probed += 1;
}
assert!(stat1_probed >= 20, "...");
```

---

### `hp41-gui/src-tauri/src/prgm_display.rs` (PHASE 36 — 4-way invariant item 4)

**Analog:** self — math1 arms at lines 260–311; mirrors CLI prgm_display.rs by design (CLAUDE.md "duplicated CLI ↔ GUI by design" exemption to SC-4).

**Pattern** (lines 260–266): IDENTICAL to `hp41-cli/src/prgm_display.rs:239–245` — same strings, same arms.

**Stat 1 substitution** (Phase 36 — same arms as CLI, byte-for-byte). NO deviations permitted (SC-4 exempt per CLAUDE.md "Workspace structure").

---

### `hp41-gui/src-tauri/src/key_map.rs` (PHASE 36)

**Analog:** self — `resolve()` at lines 25–164 (bare IDs in big match block) + `resolve_parameterized` at line 173 (compound IDs).

**Pattern** (lines 25–164 — every bare string ID maps to an `Op`):
```rust
pub fn resolve(key_id: &str) -> Result<Op, GuiError> {
    match key_id {
        "enter" => Ok(Op::Enter),
        // ... ~150 named ops ...
        // ── Stub-error arm for prompt IDs (defense-in-depth, D-26.5) ──
        "asn" | "catalog" | "view" | "xeq_prompt" | /* ... */ | "tone" => Err(GuiError {
            message: format!("'{key_id}' is planned for a future phase"),
        }),
        // ── Parameterized & unknown ──
        _ => resolve_parameterized(key_id),
    }
}
```

**Stat 1 substitution** (Phase 36 — new bare IDs added to the match):
```rust
// ── Phase 33: Stat 1 Pac (XROM 2) — Phase 36 GUI wiring ──
"sigma_normd" => Ok(Op::SigmaNormdWorkflow),
"sigma_chisqd" => Ok(Op::SigmaChisqdWorkflow),
"sigma_spear" => Ok(Op::SigmaSpear),
// ... 21 more bare IDs; RAND/SEED added per Plan 36 ...
"rand" => Ok(Op::Rand),
"seed" => Ok(Op::Seed),
```

**What changes for Stat 1 Pac:** Phase 36 work; the key IDs follow snake_case convention from `key_map.rs:6` ("Named ops use snake_case strings mirroring `hp41_cli::keys::key_to_op` semantically"). Σ-prefix replaced with `sigma_` prefix per snake_case discipline.

---

### `docs/hp41-stat1-functions.json` (PHASE 34/35)

**Analog:** `docs/hp41-math1-functions.json` lines 1–60 (full schema).

**Pattern** (entry shape):
```json
{
    "op_variant": "Sinh",
    "display_name": "SINH",
    "category": "Math1 Hyperbolics",
    "status": "implemented",
    "phase": "28",
    "key_path": "XEQ \"SINH\"",
    "description": "Hyperbolic sine: X <- sinh(X)",
    "xrom": { "module": "Math 1", "module_id": 7, "function_id": 1 }
}
```

**Stat 1 substitution** (Phase 34 — sibling JSON pool, mod 2):
```json
{
    "op_variant": "SigmaNormdWorkflow",
    "display_name": "ΣNORMD",
    "category": "Stat1 Distributions",
    "status": "implemented",
    "phase": "33",
    "key_path": "XEQ \"ΣNORMD\"",
    "description": "Standard normal distribution: 3-mode (CDF/PDF/inverse)",
    "xrom": { "module": "Stat 1", "module_id": 2, "function_id": 1 }
}
```

**What changes for Stat 1 Pac:** `module_id: 2` (Stat 1 hardware ID per `calc.fjk.ch`), `category` prefixed `Stat1`, `phase: "33"` (or per-plan phase). Hard-build-blocker: malformed JSON panics at `OnceLock` init per CLAUDE.md "JSON canonical data flow".

---

### `docs/hp41-stat1-function-matrix.md` (generated, PHASE 35)

**Analog:** `docs/hp41-math1-function-matrix.md` — generator output from `scripts/docs-matrix/`.

**Generation command:** `just docs-stat1` (Phase 35 — new recipe mirroring `docs-math1`). CI drift gate via `just docs-stat1-check`. The generator (`scripts/docs-matrix/src/main.rs`) is the next entry.

---

### `docs/hp41-stat1-divergences.md` (create, PHASE 35)

**Analog:** `docs/hp41-math1-divergences.md` lines 1–35 (header + D-30-NN scheme).

**Pattern** (header):
```markdown
# HP-41C Math Pac I Emulator Divergences

This document lists known behavioral divergences between this emulator's implementation
of the HP-41C Math Pac I module and the hardware-faithful behavior described in the
HP-41C Math Pac I Owner's Manual (HP 00041-90034, 1979).

**Status:** Expanded to comprehensive numbered catalog in Phase 30 / Plan 30-02 (DOC-04).

## How to Use This Document

Each entry carries a stable `D-30-NN` identifier ...

Every entry uses five fixed fields (D-30.5 shape):
- **OM citation** — The HP 00041-90034 page-and-example ...
- **Our behavior** — What this emulator does.
- **OM behavior** — What the OM says ...
- **Rationale** — Why we made this choice ...
- **See** — Cross-references ...
```

**Stat 1 substitution** (Phase 35 — sibling sidecar with `D-33-NN` numbering scheme per CONTEXT.md Deferred Ideas):
```markdown
# HP-41C Stat 1 Pac Emulator Divergences

This document lists known behavioral divergences between this emulator's implementation
of the HP-41C Statistics Pac 1 module and the hardware-faithful behavior described in the
HP-41C Stat 1 Pac Owner's Manual (HP 00041-90030, 1979).

## How to Use This Document

Each entry carries a stable `D-33-NN` identifier ...

Every entry uses five fixed fields:
- **OM citation** — The HP 00041-90030 page-and-example ...
- **Our behavior** ...
- **OM behavior** ...
- **Rationale** ...
- **See** ...

## 1. OM Divergences
## 2. Emulator Extensions
   (RAND / SEED per D-33.4 if OM does not list them; ν=? prompt vs OM convention; etc.)
## 3. Documented Carve-Outs
   (math1/ freeze exception per D-33.3a; ModalProgram::Stat1 freeze exception per D-33.3b if applicable)
```

**What changes for Stat 1 Pac:** sibling sidecar with `D-33-NN` scheme. First D-33-NN entries come from CONTEXT.md Deferred Ideas: items from D-33.1 (whichever resolve as divergences), D-33.3a (math1/ freeze exception), D-33.4 (RAND emulator-extension claim), D-33.8 (STAT-QUAL-09 phase reassignment).

---

### `scripts/docs-matrix/src/main.rs` (PHASE 35)

**Analog:** self — `render_markdown` basename-dispatch at lines 65–77.

**Pattern** (lines 65–77 — two-input title dispatch already in place):
```rust
fn render_markdown(entries: &[Entry], json_path: &str) -> String {
    let basename = std::path::Path::new(json_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(json_path);
    let (title, src) = if basename.ends_with("hp41cv-functions.json") {
        ("# HP-41CV ROM Function Matrix", "`docs/hp41cv-functions.json`")
    } else if basename.ends_with("hp41-math1-functions.json") {
        ("# HP-41C Math Pac I Function Matrix", "`docs/hp41-math1-functions.json`")
    } else {
        ("# Function Matrix", "`{json_path}`")
    };
    // ...
}
```

**Stat 1 substitution** (Phase 35 — add third arm before the catch-all else):
```rust
} else if basename.ends_with("hp41-stat1-functions.json") {
    ("# HP-41C Stat 1 Pac Function Matrix", "`docs/hp41-stat1-functions.json`")
}
```

**What changes for Stat 1 Pac:** ONE arm added; the XROM-column rendering (lines 100–158) is universal — `entry.xrom.is_some()` triggers the wider table format automatically for Stat 1 entries.

---

### `scripts/check-free42-contamination.sh` (PLAN 33-00 — STAT-QUAL-09 reassigned per D-33.8)

**Analog:** self — full 38-line file.

**Pattern** (lines 11–34 — `MATH1_DIR` + 12-symbol `PATTERN` + grep gate):
```bash
MATH1_DIR="hp41-core/src/ops/math1"
DISCLAIM_LINE='Free42 source consulted only as sanity-check oracle'

# WR-01: explicit directory existence check ...
if [[ ! -d "$MATH1_DIR" ]]; then
    echo "FAIL: $MATH1_DIR does not exist — license guard cannot run." >&2
    exit 2
fi

# D-32.7: 12 distinctive symbols verified zero false-positives against current source.
# The bare string "Free42" is deliberately NOT in this pattern — 122 legitimate
# "Free42 v3.0.5: <value>" cross-check references exist across the codebase
# (per RESEARCH.md). The 12 symbols below are tight enough to never match those.
PATTERN='phloat|Phloat|bid128_|decNumber|decContext|vartype|arg_struct|prgm_lines|bcd_t|Thomas Okken|AGPL|GNU General Public License'

if matches=$(grep -rn -E "$PATTERN" "$MATH1_DIR" | grep -v "$DISCLAIM_LINE"); then
    echo "FAIL: Free42 contamination detected in $MATH1_DIR:"
    echo "$matches"
    exit 1
fi
```

**The 12 distinctive identifiers (verbatim — per cross-cutting §"Free42 contamination guard" below):**
`phloat`, `Phloat`, `bid128_`, `decNumber`, `decContext`, `vartype`, `arg_struct`, `prgm_lines`, `bcd_t`, `Thomas Okken`, `AGPL`, `GNU General Public License`.

**Stat 1 substitution** (Plan 33-00 — extend `PATTERN` by ≥ 6 stats-domain tokens AND add `STAT1_DIR` scanned directory per D-33.8):
```bash
MATH1_DIR="hp41-core/src/ops/math1"
STAT1_DIR="hp41-core/src/ops/stat1"  # NEW
DISCLAIM_LINE='Free42 source consulted only as sanity-check oracle'

# WR-01: directory existence checks
for dir in "$MATH1_DIR" "$STAT1_DIR"; do
    if [[ ! -d "$dir" ]]; then
        echo "FAIL: $dir does not exist — license guard cannot run." >&2
        exit 2
    fi
done

# Extended pattern: 12 existing tokens + ≥ 6 new stats-domain tokens (Plan 33-00 finalizes the list
# by reading github.com/thomasokken/free42/blob/master/common/core_math2.cc). Candidate list per
# RESEARCH.md §"Pitfall 5":
#   math_normal_cdf, math_normal_pdf, math_normal_inv,
#   math_chi2_cdf, math_chi2_pdf, math_chi2_inv,
#   math_t_dist_cdf, math_t_dist_pdf, math_t_dist_inv,
#   math_F_dist_*,
#   math_gamma_lower, math_gamma_upper, math_beta_inc,
#   decNumberPower, decNumberExp, decNumberLn
PATTERN='phloat|Phloat|bid128_|decNumber|decContext|vartype|arg_struct|prgm_lines|bcd_t|Thomas Okken|AGPL|GNU General Public License|math_normal_|math_chi2_|math_t_dist_|math_F_dist_|math_gamma_|math_beta_inc'

for dir in "$MATH1_DIR" "$STAT1_DIR"; do
    if matches=$(grep -rn -E "$PATTERN" "$dir" | grep -v "$DISCLAIM_LINE"); then
        echo "FAIL: Free42 contamination detected in $dir:"
        echo "$matches"
        exit 1
    fi
done
```

**Acceptance probes** (SPEC.md Req. 39):
- Script exits 0 on Plan-33-00 skeleton (`stat1/mod.rs` containing only `//!` OM transcription).
- Script exits 1 when a probe-token (e.g. `math_normal_cdf_xxxxprobe`) is inserted into a temp file inside `stat1/`.
- Pattern contains ≥ 18 total tokens (12 existing + ≥ 6 new).

**What changes for Stat 1 Pac:** PATTERN regex grows; `STAT1_DIR` scanned. CI gates `just license-audit` + `.github/workflows/ci.yml::license-audit` continue to drive the script — no CI yaml changes needed.

---

### `hp41-core/tests/stat1_accuracy.rs` (PHASE 37 — create from numerical_accuracy.rs template)

**Analog:** `hp41-core/tests/numerical_accuracy.rs:1–122` (struct + macro + helpers).

**Test-table structure** (lines 27–122):
```rust
const TOLERANCE: f64 = 1e-9;
const WIDE_TOL: f64 = 1e-6;

struct AccuracyCase {
    id: usize,
    domain: &'static str,
    description: String,
    expected: f64,
    actual: f64,
    tol: f64,
}

fn passes_with_tol(actual: f64, expected: f64, tol: f64) -> bool {
    if expected == 0.0 { actual.abs() <= tol }
    else { ((actual - expected) / expected).abs() <= tol }
}

#[test]
fn test_numerical_accuracy_suite() {
    let mut cases: Vec<AccuracyCase> = Vec::with_capacity(500);
    let mut id = 0usize;

    macro_rules! case {
        ($domain:expr, $desc:expr, $expected:expr, $actual:expr) => {{
            id += 1;
            cases.push(AccuracyCase { id, domain: $domain, description: $desc.to_string(),
                expected: $expected, actual: $actual, tol: TOLERANCE });
        }};
        // wide-tolerance variant for cases where BCD rounding compounds
    }
    // ... thousands of case!(...) lines ...
}
```

**Stat 1 substitution** (Phase 37 — sibling suite with two-level tolerance per SPEC.md Req. 46):
```rust
const TOL_CLOSED: f64 = 1e-9;   // closed-form ops
const TOL_ITERATIVE: f64 = 1e-7; // iterative ops (SPEC Req. 46)

struct Stat1AccuracyCase { /* same shape */ }

#[test]
fn test_stat1_accuracy_suite() {
    macro_rules! case_closed {
        // 1e-9 tolerance for ΣNORMD CDF/PDF, ΣSPEAR, ΣBSTAT/BSTG, ΣLIN/EXP/LOGI/POW, ΣXSQEV/EFXSQ, ΣAOVONE
    }
    macro_rules! case_iterative {
        // 1e-7 tolerance for ΣNORMD inverse, ΣCHISQD CDF, ΣTSTAT/ΣPTST, ΣMLRXY/MLRXYZ/POLYP, ΣAOVTWO/ΣANOCOV
    }
    // ... case_closed!("ΣNORMD CDF", "Q(1.96)", 0.024998, ...);
    // ... case_iterative!("ΣNORMD inv", "Φ⁻¹(0.025)", -1.959963984540054, ...);
}
```

**What changes for Stat 1 Pac:** two-level tolerance per SPEC.md Req. 46 (1e-9 closed-form vs 1e-7 iterative). Phase 37 work (Phase 33 produces only per-Op unit tests inside `#[cfg(test)] mod tests` blocks).

---

## Shared Patterns (Cross-Cutting Concerns)

### 4-way exhaustive-match invariant (apply to EVERY new Op variant in Phase 33)

**Source:** CLAUDE.md §"4-way exhaustive-match invariant" + RESEARCH.md §"4-Way Exhaustive-Match — Items 1 + 2".

**The exact 4 call sites** (every new `Op::*` variant must land in all four before any caller compiles):

| # | File | Function / Match Block | Phase that owns it for Stat 1 |
|---|---|---|---|
| 1 | `hp41-core/src/ops/mod.rs` | `dispatch()` (large match ~line 1152+) | **Phase 33 (Plan 33-03..33-08)** |
| 2 | `hp41-core/src/ops/program.rs` | `execute_op()` (large match ~line 900) | **Phase 33 (Plan 33-03..33-08)** |
| 3 | `hp41-cli/src/prgm_display.rs` | `op_display_name()` (lines 239–291 for math1) | Phase 34 (CLI integration) |
| 4 | `hp41-gui/src-tauri/src/prgm_display.rs` | `op_display_name()` (lines 260–311 for math1) | Phase 36 (GUI integration) |

**Compile-time enforcement:** items 3 + 4 will not compile until added; Phase 33 acceptance is `cargo check -p hp41-core` GREEN; `cargo check -p hp41-cli` / `-p hp41-gui` FAILURES are EXPECTED post-Phase-33 and resolved in Phase 34 / 36 (RESEARCH.md §"4-Way Exhaustive-Match" line 575).

---

### Free42 contamination guard — the 12 distinctive identifiers verbatim

**Source:** `scripts/check-free42-contamination.sh:28` (the `PATTERN` variable).

**Verbatim list** (planner can grep their own plan content against these):

1. `phloat`
2. `Phloat`
3. `bid128_`
4. `decNumber`
5. `decContext`
6. `vartype`
7. `arg_struct`
8. `prgm_lines`
9. `bcd_t`
10. `Thomas Okken`
11. `AGPL`
12. `GNU General Public License`

**Note** (`scripts/check-free42-contamination.sh:25–27`): bare `Free42` is DELIBERATELY EXCLUDED because 122 legitimate cross-check references exist in the codebase (form: `"Free42 v3.0.5: <value>"`).

**Stat 1 extension (Plan 33-00, STAT-QUAL-09 reassigned per D-33.8):** ≥ 6 new stats-domain tokens added to PATTERN; final list extracted from `github.com/thomasokken/free42/blob/master/common/core_math2.cc` during Plan 33-00. Candidates (RESEARCH.md §"Pitfall 5"):

- `math_normal_cdf`, `math_normal_pdf`, `math_normal_inv`
- `math_chi2_cdf`, `math_chi2_pdf`, `math_chi2_inv`
- `math_t_dist_cdf`, `math_t_dist_pdf`, `math_t_dist_inv`
- `math_F_dist_*` (F-distribution functions, even though out of Phase 33 scope — catch the symbols)
- `math_gamma_lower`, `math_gamma_upper`, `math_beta_inc`
- decNumber distinctive idioms: `decNumberPower`, `decNumberExp`, `decNumberLn` (extended from 12-symbol base if absent)

**Planner CI gate to copy into every Plan 33-N acceptance section:**
```bash
bash scripts/check-free42-contamination.sh    # exit 0 = clean; exit 1 = contamination; exit 2 = missing dir
just license-audit                            # same gate via the just recipe
```

---

### JSON canonical data flow — Stat 1 mirrors Math Pac I exactly

**Source:** CLAUDE.md §"JSON canonical data flow" + RESEARCH.md §"JSON Registry".

`docs/hp41-stat1-functions.json` (created in Phase 34) is the **single source of truth** for:

1. **Help overlay** (`hp41-cli/src/help_data.rs::help_entries_all()` chain — Phase 34 extension)
2. **Right-panel filter** (`hp41-cli/src/keys.rs::key_ref_entries` — already universal, NO code change needed; `entry.xrom.is_some()` excludes Stat 1 entries automatically)
3. **Docs matrix** (`scripts/docs-matrix/src/main.rs::render_markdown` basename-dispatch — Phase 35 extension)
4. **Bidirectional parity test** (`hp41-cli/tests/function_matrix_parity.rs` — Phase 34 extension with 3 sibling tests at `0b0000_0010` bitmask)
5. **Coverage probe** (`hp41-cli/tests/key_coverage.rs::stat1_probed` sub-loop — Phase 34 extension)

**Phase 33 does NOT ship the JSON file** — Phase 34 work. Phase 33 only ships the Ops + framework that the JSON later references.

---

### Serde discipline for `rand_seed: HpNum` — the unique pattern (P20 trap)

**Source:** SPEC.md Req. 37, RESEARCH.md §"Pitfall 3", CLAUDE.md §"Save-file backward compat".

**The ONLY new v3.1 CalcState field with this serde shape:**
```rust
#[serde(default)]                  // ← yes
pub rand_seed: HpNum,
// NO #[serde(skip)] — would break STAT-RNG-03 determinism across save/load
```

**Existing field that uses the IDENTICAL shape** (`state.rs:169–170`):
```rust
#[serde(default)]
pub complex_mode: bool,
```

**Existing fields that use the WRONG (skip) shape** — DO NOT mimic for `rand_seed`:
- `print_buffer` (line ~140): `#[serde(default, skip)]`
- `modal_program` (line 185): `#[serde(default, skip)]`
- `modal_prompt` (line 193): `#[serde(default, skip)]`
- `integ_state` (line 199): `#[serde(default, skip)]`
- `solve_state` (line 205): `#[serde(default, skip)]`
- `difeq_state` (line 212): `#[serde(default, skip)]`
- `cancel_requested` (line 221): `#[serde(default = "...", skip)]`

**Code review block-the-PR rule:** if reviewer sees `#[serde(default, skip)] pub rand_seed: HpNum` → BLOCK. The doc-comment on the field MUST SHOUT the unique shape (RESEARCH.md §"RNG Seed Persistence" gives the exact doc-comment template).

**Plan 33-01 CI test gate:** `rand_seed_serde_round_trip` test in `hp41-core/src/state.rs::tests` is mandatory (RESEARCH.md §"State Migration & RNG Seed Persistence"):
```rust
#[test]
fn rand_seed_serde_round_trip() {
    let mut state = CalcState::new();
    state.rand_seed = HpNum::from_str("0.7").unwrap();
    let blob = serde_json::to_string(&state).unwrap();
    let restored: CalcState = serde_json::from_str(&blob).unwrap();
    assert_eq!(restored.rand_seed, state.rand_seed);  // proves NOT skipped
}
```

---

### OM disclaim header — byte-for-byte template

**Source:** every file in `hp41-core/src/ops/math1/*.rs` lines 1–3.

**Math Pac I header** (verbatim):
```rust
// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
```

**Stat 1 Pac substitution** (every file in `hp41-core/src/ops/stat1/*.rs`):
```rust
// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
```

The contamination-guard script (`check-free42-contamination.sh:12`) uses the substring `'Free42 source consulted only as sanity-check oracle'` as the DISCLAIM_LINE allow-list — Stat 1 substitution preserves this string byte-for-byte, so the existing allow-list logic continues to work.

---

## Patterns That DO NOT Translate (Flagged for Planner Action)

### Pitfall 8: `ModalProgram::Stat1` extension to `math1/modal.rs` (FREEZE-EXCEPTION CANDIDATE)

**Source:** RESEARCH.md §"Pitfall 8" + RESEARCH.md §"Open Question 3" + RESEARCH.md §"Pattern 3 / ModalProgram extension".

**The tension:**
- Phase 33 ships ΣCHISQD `ν=?`, ΣPOLYP `DEGREE=?`, and SEED `SEED?` modal opens.
- Each needs a new `ModalProgram::Stat1*` variant on the carrier enum.
- The carrier enum `ModalProgram` lives in `hp41-core/src/ops/math1/modal.rs:23–39` — in the FROZEN set per D-33.3 (only `xrom.rs` is the documented freeze exception).
- D-33.5 forbids new transient `CalcState` fields beyond `rand_seed` — so a separate `modal_program_stat1: Option<Stat1ModalProgram>` field is OUT.

**RESEARCH.md recommendation (option a, treated as A7 assumption):**
- Add a SINGLE additive variant `ModalProgram::Stat1(stat1::modal::Stat1Step)` to `math1/modal.rs`.
- Symmetric carve-out to the `xrom.rs` exception (both are central registries for cross-module entries).
- Plan 33-01 captures this as a SECOND freeze exception per D-33.3b in CONTEXT.md AMENDMENT before Plan 33-03 starts.

**Pattern from `math1/modal.rs:23–39`** (the enum to extend):
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ModalProgram {
    Matrix(MatrixInputStep),
    Solve(SolveInputStep),
    Poly(PolyInputStep),
    Integ(IntegInputStep),
    Difeq(DifeqInputStep),
    Four(FourInputStep),
    Trans(TransInputStep),
    // ← ADD: Stat1(Stat1Step),  — Plan 33-03 (requires D-33.3b amendment first)
}
```

**Action for planner before Plan 33-03 starts:** explicitly write D-33.3b in CONTEXT.md authorizing the single additive variant in `math1/modal.rs`. RESEARCH.md A7 calls out this as "needs explicit confirmation".

**Submit_step dispatch table** (`math1/mod.rs:72–80`) MUST gain one extra arm:
```rust
match modal {
    ModalProgram::Matrix(step) => matrix::submit_step(state, step),
    ModalProgram::Solve(step) => solve::submit_step(state, step),
    // ... 5 more existing arms ...
    ModalProgram::Stat1(step) => crate::ops::stat1::modal::submit_step(state, step),  // ← NEW
}
```

Same single-line extension required for `current_prompt()` dispatch (`math1/modal.rs:52–62`) and `requires_alpha_label()` dispatch (`math1/modal.rs:76–83`).

---

### No direct math1 precedent for grouped accumulators or Gauss elimination

ANOVA (`stat1/anova.rs`) and 2/3-predictor regression (`stat1/regression.rs`) have **no direct math1 precedent**. Patterns composed from:
- Σ-register accumulation discipline (`hp41-core/src/ops/stats.rs:22–53`) — for accumulator atomicity + SIZE-floor guard.
- Local Gauss elimination — re-derived per SPEC.md Req. 21 (NO `ops::math1::matrix::*` imports; CI gate: `grep -n "ops::math1::matrix" hp41-core/src/ops/stat1/` returns empty).

---

### LCG RNG has no math1 precedent

`stat1/rand.rs` LCG body (`r_{n+1} = FRC(9821·r_n + 0.211327)`) is composed from THREE existing patterns, not a single analog:
1. `op_frc` in `hp41-core/src/ops/math.rs` — FRC primitive (likely needs a `HpNum::checked_int` / `trunc_int` helper added per RESEARCH.md Open Q1).
2. `state.rs:169–170` `complex_mode` — the `#[serde(default)]` WITHOUT `skip` serde shape.
3. `math1/poly.rs:80–90` `op_poly_workflow` — modal opener for SEED `SEED?` prompt.

---

## Metadata

**Analog search scope:**
- `hp41-core/src/` (full read of `ops/math1/*.rs`, `ops/stats.rs`, `ops/mod.rs`, `ops/program.rs`, `state.rs`, `num.rs`)
- `hp41-cli/src/` (`help_data.rs`, `keys.rs`, `prgm_display.rs`)
- `hp41-cli/tests/` (`function_matrix_parity.rs`, `key_coverage.rs`)
- `hp41-gui/src-tauri/src/` (`prgm_display.rs`, `key_map.rs`)
- `docs/` (`hp41-math1-functions.json`, `hp41-math1-divergences.md`)
- `scripts/` (`check-free42-contamination.sh`, `docs-matrix/src/main.rs`)
- `hp41-core/tests/numerical_accuracy.rs` (test-table template)

**Files scanned:** 19 source files + 4 docs/scripts files + 3 GSD planning artifacts (33-CONTEXT.md, 33-SPEC.md, 33-RESEARCH.md)

**Pattern extraction date:** 2026-05-22

**Planner consumption guidance:**
- Plans 33-00 → use the "Free42 contamination guard" cross-cutting block + the `mod.rs` module-hub pattern.
- Plan 33-01 → use the `state.rs` xrom_modules + complex_mode + serde-roundtrip patterns + `math1/xrom.rs` MATH_1 const template.
- Plan 33-02 → use the `num.rs` f64-bridge `checked_asin` template + the inline scipy oracle pattern from `math1/poly.rs` / `math1/integ.rs` (RESEARCH.md §"Distribution Primitives" Table for the verbatim oracle tuples).
- Plan 33-03 → use the `math1/poly.rs:80–90` modal-opener template + the `math1/integ.rs:104–111` display-mode-tied tolerance + the `Ordering::Relaxed` cancellation loop pattern. **MUST FIRST resolve D-33.3b ModalProgram freeze exception.**
- Plan 33-04 → use the `ops/stats.rs:22–53` Σ-block-read pattern.
- Plan 33-05 → use the `op_sigma_plus` delegate pattern (transform Y then call existing stats).
- Plan 33-06 → use the `ops/stats.rs:58–85` `op_sigma_minus` extension + Plan 33-00 STAT1_*_REG named consts.
- Plan 33-07 → use the `ops/stats.rs` Σ-block-read + the `distributions::beta_regularized_f64` bridge.
- Plan 33-08 → use the `op_frc` LCG pattern + `complex_mode` serde shape + `op_poly_workflow` modal opener + local Gauss elimination (NO math1::matrix imports).

---

*Phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1*
*Pattern map produced: 2026-05-22*
*Next: `/gsd-plan-phase 33` consumes this PATTERNS.md alongside CONTEXT.md + SPEC.md + RESEARCH.md to produce the 9 plans per D-33.2.*
