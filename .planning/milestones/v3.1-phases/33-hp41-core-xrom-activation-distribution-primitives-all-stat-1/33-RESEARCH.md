# Phase 33: hp41-core — XROM Activation + Distribution Primitives + All Stat 1 Ops — Research

**Researched:** 2026-05-22
**Domain:** XROM application-module emulation (second module; numerical statistics + distribution primitives)
**Confidence:** HIGH (architecture/wiring); HIGH (algorithms — primary sources verified); HIGH (pitfalls — codebase-verifiable); MEDIUM (one item: ANOVA + ANCOVA register layout, gated to Plan 33-00 OM transcription)

---

## Summary

Phase 33 activates HP-41 Statistics Pac 1 (XROM ID 2, OM 00041-90030, June 1979) as the second XROM application module on top of the v3.0 Math Pac I framework. The infrastructure is already in place: the `xrom_resolve()` bit-1 stub at `hp41-core/src/ops/math1/xrom.rs:133–134` was reserved at v3.0 ship for exactly this purpose, and the `XromModule` struct, resolver-last chain, modal-prompt machinery, JSON pipeline, `request_cancel` channel, and 4-way exhaustive-match invariant all reuse unchanged. The phase ships ~24 new `Op` variants across an 11-file `hp41-core/src/ops/stat1/` module tree that mirrors `math1/` exactly in shape and disclaim discipline.

The deepest new dependency is three hand-coded f64-bridge distribution primitives in `stat1/distributions.rs` (~140 LOC total): `norm_cdf_inv_f64` (Acklam/AS 241), `gamma_regularized_f64` (AS 239 series + continued fraction), `beta_regularized_f64` (AS 63 continued fraction). These follow the existing `checked_asin`/`checked_acos`/`checked_atan` pattern in `hp41-core/src/num.rs:184–211`. `rust_decimal 1.42` already provides `norm_cdf`, `norm_pdf`, `erf`, `ln`, `exp`, `sqrt`, and `powd` — closed-form CDF/PDF paths use these directly. The `statrs` crate was evaluated and rejected upstream in research because (1) its mandatory `approx` dep would land in the production graph, (2) its f64-only interface requires a conversion shim of similar size to the hand-coded path, and (3) its modern Lanczos-gamma algorithm may diverge from the 1979-era OM approximations. Zero new runtime dependencies are added in this phase.

The risk surface is operational discipline, not algorithmic difficulty. Five critical traps dominate: (1) **P21 — ANOVA/regression Σ-register layout must be transcribed verbatim from the OM in Plan 33-00 before any Op reads from extended registers** (silent wrong answers if guessed); (2) **P24 — `default_xrom_modules()` must flip to `0b0000_0011` AND a `migrate_after_load()` method must set bit 1 on v3.0 save files** (serde uses the stored value, not the new default); (3) **P20 — `rand_seed: HpNum` must carry `#[serde(default)]` WITHOUT `#[serde(skip)]`** (the ONLY new v3.1 CalcState field with this serde shape; muscle-memory invites the wrong choice); (4) **P19 — quantile-inversion convergence threshold must be display-mode-tied** (`integ_threshold()` precedent at `math1/integ.rs:104`); (5) **P27 — `scripts/check-free42-contamination.sh` must be extended for stats-domain identifiers BEFORE the first `stat1/*.rs` file lands** (Plan 33-00 deliverable).

**Primary recommendation:** Follow the 9-plan slicing locked in CONTEXT.md D-33.2 verbatim. Plan 33-00 ships no `Op` code — only the contamination-guard extension and the OM-transcription header in `ops/stat1/mod.rs`. Plan 33-01 ships XROM framework activation (bit-1 arm, `STAT_1` const, `default_xrom_modules()` flip, `migrate_after_load()`, `rand_seed` field). Plan 33-02 ships the three f64-bridge primitives with inline scipy.stats oracle constants — and all oracle tests must pass GREEN before any program Op (33-03 onward) lands.

---

## User Constraints (from CONTEXT.md + SPEC.md)

### Locked Decisions (from CONTEXT.md — 9 D-33.* + 46 SPEC.md requirements)

- **D-33.1:** Run `/gsd-spec-phase 33` BEFORE this research (done — see 33-SPEC.md). SPEC.md locks 46 falsifiable acceptance criteria across 7 groups (STAT-FW / STAT-UNI / STAT-AOV / STAT-REG / STAT-HYP / STAT-DST / STAT-RNG). Research must not contradict SPEC.md; planner cannot override SPEC.md.
- **D-33.2:** Ship as **9 plans (33-00 through 33-08)** following research SUMMARY.md "Implementation order within phase" verbatim:
  - 33-00 — Contamination guard extension + OM transcription (no Op code)
  - 33-01 — XROM framework activation + state migration + `rand_seed` field
  - 33-02 — `stat1/distributions.rs` 3 primitives + scipy oracle (GREEN before any Op)
  - 33-03 — ΣNORMD + ΣCHISQD (consume distribution primitives)
  - 33-04 — ΣSPEAR + ΣXSQEV / ΣEFXSQ (closed-form, no distribution function)
  - 33-05 — ΣBSTAT / ΣBSTG + ΣLIN / ΣEXP / ΣLOGI / ΣPOW (Σ-register delegate)
  - 33-06 — ΣMMTUG / ΣMMTGD + ANOVA family + ΣCTKKK / ΣCTKK (OM-register-dependent)
  - 33-07 — ΣPTST / ΣTSTAT (t-tests consuming `beta_regularized_f64`)
  - 33-08 — ΣMLRXY / ΣMLRXYZ + ΣPOLYP / ΣPOLYC + RAND / SEED
- **D-33.3 / 33.3a:** Break `hp41-core/src/ops/math1/` freeze for **`xrom.rs` ONLY**. Extend in place with `STAT_1: XromModule { id: 2, name: "STAT 1B", ops: &[...] }` + `fn stat1_resolve(name: &str) -> Option<Op>` + bit-1 arm at line 134. The other 12 files in `math1/` remain strictly frozen. CLAUDE.md exception clause added in Phase 35 (STAT-DOC-05).
- **D-33.4 / 33.4a:** RAND / SEED implemented as v3.1 emulator extension using NPS p. 21 community-LCG formula `r_{n+1} = FRC(9821·r_n + 0.211327)`. If Plan 33-08 OM read shows RAND IS a top-level Stat 1 Pac ROM entry, treat as OM-feature-complete; otherwise document in `docs/hp41-stat1-divergences.md` (Phase 35).
- **D-33.5:** **11-file layout** in `hp41-core/src/ops/stat1/`: `mod.rs`, `distributions.rs`, `basic_stats.rs`, `moments.rs`, `anova.rs`, `regression.rs`, `hypothesis.rs` (NOT `tests.rs` — avoids collision with `#[cfg(test)] mod tests` blocks), `nonparam.rs`, `normd.rs`, `chisqd.rs`, `rand.rs`. Each file ≤ 300 LOC excluding tests.
- **D-33.6:** Oracle data is **inline `(input, scipy_expected, tolerance)` tuples** in `#[cfg(test)] mod tests` of `stat1/distributions.rs` (and per-Op test modules). NO external Python toolchain dependency. NO fixture file. Oracle frozen at write-time via fenced scipy.stats Python snippet in plan comments.
- **D-33.7:** State migration in **`hp41-core/src/state.rs`** (single source of truth): `impl CalcState { pub fn migrate_after_load(&mut self) { if self.xrom_modules & 0b0000_0010 == 0 { self.xrom_modules |= 0b0000_0010; } } }`. Idempotent. Called once after every deserialization by both CLI persistence (Phase 34) and GUI persistence (Phase 36).
- **D-33.8:** STAT-QUAL-09 moved Phase 37 → Phase 33 (Plan 33-00). Contamination guard extends `scripts/check-free42-contamination.sh` PATTERN by ≥ 6 stats-domain tokens (final list extracted in Plan 33-00 from `github.com/thomasokken/free42/blob/master/common/core_math2.cc`).
- **Two-level tolerance** (STAT-QUAL-05, Req. 46): **1e-9 relative** for closed-form ops (ΣNORMD CDF/PDF, ΣSPEAR, ΣBSTAT/BSTG, ΣLIN/EXP/LOGI/POW, ΣXSQEV/EFXSQ, ΣAOVONE); **1e-7 relative** for iterative ops (ΣNORMD inverse, ΣCHISQD CDF, ΣTSTAT/ΣPTST, ΣMLRXY/ΣMLRXYZ/ΣPOLYP, ΣAOVTWO/ΣANOCOV).
- **Iteration cap:** 50 iterations (returns `Err(HpError::ConvergenceFailed)`).
- **Convergence tolerance:** display-mode-tied `10^(-FIX_decimals − 1)` mirroring v3.0 `integ_threshold()`; fallback `1e-10` when not FIX.
- **Cancellation:** per-iteration `cancel_requested.load(Relaxed)` check inside every distribution-quantile loop (Pitfall 11 extended).
- **No new transient CalcState fields beyond `rand_seed`** — modal prompts reuse `print_buffer` + `modal_program` infrastructure.
- **No reuse of Math Pac I `MATRIX/SIMEQ`** in regression — Gauss elimination is local to `stat1/regression.rs` (Req. 21).
- **No `statrs`, no `rand`, no `getrandom`** in `hp41-core` runtime deps. `approx` is dev-dep only (already in Cargo.toml line 18).
- **`#![deny(clippy::unwrap_used)]`**, no `println!` / `eprintln!`, MSRV 1.88.

### Claude's Discretion

- Concrete identifier list for the contamination-guard extension (D-33.8): research SHALL surface candidates from `core_math2.cc`; planner / executor finalizes the regex pattern.
- ANOVA / regression result-output shape (stack push vs print buffer vs both): defer to Plan 33-06 OM "Results" section read; if OM silent, follow Math Pac I `POLY` pattern (stack pushes for primary numeric results, print buffer for prompts).
- Per-Op `LiftEffect::Enable / Disable / Neutral` declarations: follow HP-41 stack-lift conventions established in v1.0 + reaffirmed in v3.0 ADR-001.
- Inline test count per Op: meet `stat1_op_test_count.rs` floor of ≥ 5 (Phase 37 meta-gate); aim for ~6–8 to leave margin.

### Deferred Ideas (OUT OF SCOPE this phase)

- CLI integration → Phase 34
- Documentation / ADRs / divergences catalog → Phase 35
- GUI integration → Phase 36
- Coverage gates / lint extensions / E2E smoke → Phase 37
- F-distribution, Binomial, Poisson, Hypergeometric, histograms, P(N,R)/C(N,R) — anti-features confirmed absent from Stat 1 Pac per NPS p. 42/49
- Welch's t-test — REQUIREMENTS.md Out-of-Scope
- `statrs` re-evaluation — locked rejected for v3.1
- Signed binary releases — deferred to v3.1.x / v3.2
- HP-copyrighted ROM-image redistribution — permanently excluded

---

## Phase Requirements

| ID | Description (one-liner) | Research Support |
|----|-------------------------|------------------|
| STAT-FW-01 | `STAT_1: XromModule { id: 2, name: "STAT 1B" }` + bit-1 arm in `xrom_resolve` fires LAST after MATH_1 | §"Module Wiring" walks the exact edits to `math1/xrom.rs`; verified bit-1 stub at line 134 |
| STAT-FW-02 | `default_xrom_modules() = 0b0000_0011` + startup migration sets bit 1 on v3.0 save files | §"State Migration" gives the exact `migrate_after_load()` body; existing field at `state.rs:228` |
| STAT-FW-03 | 4-way exhaustive-match items 1 + 2 honored for ~24 new Op variants | §"4-Way Exhaustive-Match — Items 1 + 2" enumerates insertion sites |
| STAT-FW-04 | All 14 entry points callable via XEQ-by-name from CLI + GUI + `run_program` | §"Module Wiring" — `xrom_resolve` already plugs into both `program.rs` insertion sites (lines 80, 541) |
| STAT-UNI-01..04 | ΣBSTAT, ΣBSTG, ΣMMTUG, ΣMMTGD + `[C]` correction-key extension | §"Σ-Register Layout" — extends existing R01–R06 in `ops/stats.rs`; new registers Plan-33-00-gated |
| STAT-AOV-01..04 | ΣAOVONE / ΣAOVTWO / ΣANOCOV — OM-verified register layout | §"ANOVA Family" — `STAT1_AOV_*_REG` named consts; transcribed Plan 33-00 |
| STAT-REG-01..09 | ΣLIN/EXP/LOGI/POW + ΣMLRXY/MLRXYZ + ΣPOLYP/POLYC + local Gauss elimination | §"Regression Family" + §"Numerical Algorithms — Gauss Elimination" |
| STAT-HYP-01..07 | ΣPTST/TSTAT + χ²-family + ΣSPEAR — pooled-variance t-test, Welch excluded | §"Hypothesis Tests" + §"beta_regularized_f64 (AS 63)" |
| STAT-DST-01..07 | ΣNORMD 3 modes + ΣCHISQD + 3 distribution primitives + display-mode-tied convergence | §"Distribution Primitives — Algorithms & Validation" (the deepest section) |
| STAT-RNG-01..04 | RAND/SEED + `rand_seed: HpNum` with `#[serde(default)]` NOT `skip` | §"RNG Seed Persistence" — exact serde shape locked |

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| XROM module registration (`STAT_1` const) | `hp41-core` | — | Resolver registry lives where dispatch lives |
| XEQ-by-name resolution (bit-1 arm) | `hp41-core` (`math1/xrom.rs`) | — | Last-fires invariant requires single resolver chain |
| Distribution numerical primitives | `hp41-core` (`stat1/distributions.rs`) | — | Behavioral emulation = numeric core stays UI-agnostic |
| ANOVA / regression / hypothesis Ops | `hp41-core` (`stat1/*.rs`) | — | All math runs in core; SC-4 forbids duplication elsewhere |
| RNG state + seed | `hp41-core` (`state.rs` field + `stat1/rand.rs`) | — | Persisted in `CalcState`, shared CLI ↔ GUI via `~/.hp41/autosave.json` |
| `[C]` correction key wiring | `hp41-core` (`stats.rs::op_sigma_minus` extension) | — | Mirrors existing R01–R06 reversal — no UI |
| Modal prompts (SEED?, ν=?, DEGREE=?) | `hp41-core` (`modal_program` + `print_buffer`) | CLI/GUI Phase 34/36 (render) | Core writes to existing transient fields; frontends consume |
| `op_display_name` arms (programmatic display) | CLI (Phase 34) + GUI (Phase 36) | — | 4-way invariant items 3 + 4 — out of scope this phase |
| Contamination guard (Free42 GPL) | CI script (`scripts/`) | — | Lives outside crates; gated by both `just license-audit` and `ci.yml::license-audit` |
| JSON canonical data (`docs/hp41-stat1-functions.json`) | docs + CLI loader (Phase 34) | — | Phase 33 does NOT ship JSON (CLI integration only) |

**Sanity-check for planner:** Phase 33 changes touch ONLY `hp41-core/src/` + `scripts/check-free42-contamination.sh`. NO changes to `hp41-cli/`, `hp41-gui/`, or `docs/` in this phase. If a plan task proposes editing those, escalate.

---

## Standard Stack

### Core (already present — no additions)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rust_decimal` | 1.42 (workspace) `[VERIFIED: hp41-core/Cargo.toml line 8]` | BCD-like 28-digit decimal arithmetic; `MathematicalOps` provides `norm_cdf`, `norm_pdf`, `erf`, `ln`, `exp`, `sqrt`, `powd`, `sin`, `cos`, `tan` | Project-wide invariant since v1.0 ADR-001; matches HP-41 10-digit display precision via `round_sf_with_strategy(10, MidpointAwayFromZero)` |
| `thiserror` | workspace | `HpError` enum derive | Already used everywhere |
| `serde` + `serde_json` | workspace | `CalcState` (de)serialization, `~/.hp41/autosave.json` | Save-file format since v1.0 |

### Supporting (dev-deps; already present)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `approx` | 0.5.1 (dev) `[VERIFIED: hp41-core/Cargo.toml line 18]` | `assert_relative_eq!` for tolerance-based test assertions | Every `stat1/*.rs` unit test against oracle constants |
| `proptest` | 1.11 (dev) | Property-based randomized tests | Optional — `stat1_accuracy.rs` Phase 37 |
| `criterion` | 0.5 (dev) | Advisory benchmarks | Not CI-gated; optional |

### Rejected (do NOT add)

| Instead of | Could Use | Why Rejected |
|------------|-----------|--------------|
| Hand-coded `norm_cdf_inv_f64` (~30 LOC) | `statrs 0.18` | Mandatory `approx` production dep; f64-only interface forces conversion shim of identical size; modern Lanczos-gamma may diverge from 1979-era OM approximations; STACK.md research locked rejection `[VERIFIED: .planning/research/STACK.md]` |
| Hand-coded LCG RNG (~5 LOC) | `rand` + `rand_distr` | Adds 2 production deps + `getrandom` (which violates `hp41-core` no-I/O invariant); LCG is OM-prescribed anyway |
| `Op::Sigma*` named variants (Op-strategy A) | `Op::XromCall(u16)` table-dispatch | Op-strategy A locked in v3.0 ADR-001; preserves 4-way exhaustive-match invariant `[VERIFIED: docs/adr/v3.0-001-op-strategy.md]` |
| Local Gauss elimination in `stat1/regression.rs` | Math Pac I `MATRIX/SIMEQ` | Cross-XROM coupling rejected per Req. 21 — Stat 1 must not depend on Math 1 being loaded |

**Installation:** No new packages — phase ships entirely on existing workspace deps.

**Version verification:**
```bash
# Confirm rust_decimal MathematicalOps surface on the installed version:
grep "rust_decimal" Cargo.toml hp41-core/Cargo.toml
# Result: workspace = "1.42" with features = ["maths", "serde-with-str"] — confirmed.
```
`[VERIFIED: /Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/Cargo.toml line 8]`

---

## Package Legitimacy Audit

> **Skipped — no new runtime dependencies installed in this phase.** All algorithmic work uses existing workspace deps (`rust_decimal 1.42`, `thiserror`, `serde`, `serde_json`). The three distribution primitives are hand-coded against primary mathematical sources (Acklam/AS 241, AS 239, AS 63) in ~140 LOC. `statrs`, `rand`, `rand_distr`, `getrandom` are all explicitly rejected per `.planning/research/STACK.md`.

If a future plan task proposes adding a new external crate, run slopcheck:
```bash
pip install slopcheck --break-system-packages 2>/dev/null || true
slopcheck install <pkg> --json
```
…and verify with `cargo search <pkg>`. Phase 33 itself adds nothing.

---

## Architecture Patterns

### System Architecture — Data Flow

```
                              ┌────────────────────────────────────────┐
                              │  User keystroke: XEQ "ΣNORMD"          │
                              └─────────────────┬──────────────────────┘
                                                │
                                                ▼
                              ┌────────────────────────────────────────┐
                              │  hp41-cli/src/keys.rs                  │
                              │  xeq_by_name_local_resolve(label)      │
                              │  (Phase 34 wiring — already routes via │
                              │   builtin_card_op → xrom_resolve)      │
                              └─────────────────┬──────────────────────┘
                                                │
                                                ▼
                              ┌────────────────────────────────────────┐
                              │  hp41-core/src/ops/program.rs:80,541   │
                              │  builtin_card_op(label) → None         │
                              │  xrom_resolve(label, state.xrom_modules)
                              └─────────────────┬──────────────────────┘
                                                │
                                                ▼
                              ┌────────────────────────────────────────┐
                              │  hp41-core/src/ops/math1/xrom.rs:127   │
                              │  if modules & 0b0000_0001 → math1_resolve
                              │  if modules & 0b0000_0010 → stat1_resolve  ◄── NEW (Plan 33-01)
                              │  else None → caller surfaces InvalidOp │
                              └─────────────────┬──────────────────────┘
                                                │
                                                ▼  Some(Op::SigmaNormdWorkflow)
                              ┌────────────────────────────────────────┐
                              │  hp41-core/src/ops/mod.rs::dispatch()  │
                              │  match Op::SigmaNormdWorkflow → ...    │  ◄── NEW arm (Plan 33-03)
                              │  hp41-core/src/ops/program.rs::execute_op()
                              │  match Op::SigmaNormdWorkflow → ...    │  ◄── NEW arm (Plan 33-03)
                              └─────────────────┬──────────────────────┘
                                                │
                                                ▼
                              ┌────────────────────────────────────────┐
                              │  hp41-core/src/ops/stat1/normd.rs       │  ◄── NEW (Plan 33-03)
                              │  op_sigma_normd_workflow(state) →      │
                              │    mode dispatch (CDF / PDF / inverse) │
                              └─────────────────┬──────────────────────┘
                                                │
                          ┌─────────────────────┼──────────────────────┐
                          ▼                     ▼                      ▼
              CDF path (closed)     PDF path (closed)        Inverse path (iterative)
              rust_decimal::norm_cdf  rust_decimal::norm_pdf    norm_cdf_inv_f64
              → HpNum                  → HpNum                    ↓
                                                                 stat1/distributions.rs (Plan 33-02)
                                                                 Newton + bisection,
                                                                 cancel_requested check per iter,
                                                                 display-mode tolerance
                                                                 → f64 → HpNum::from(...)
                                                │
                                                ▼
                              ┌────────────────────────────────────────┐
                              │  state.stack.x updated; LiftEffect     │
                              │  applied; return to dispatch caller.   │
                              └────────────────────────────────────────┘
```

### Recommended Project Structure (Plan 33-00 ships the skeleton; later plans fill)

```
hp41-core/src/ops/
├── math1/                  # FROZEN since Plan 25-01 EXCEPT xrom.rs (D-33.3)
│   └── xrom.rs             # EDIT: + STAT_1 const, + stat1_resolve(), + bit-1 arm at line 134
└── stat1/                  # NEW (D-33.5: 11 files; each ≤ 300 LOC excluding tests)
    ├── mod.rs              # Module hub + OM "Storage Registers" verbatim `//!` transcription (Plan 33-00)
    │                       #   + pub const STAT1_MAX_REG: usize derived from OM table
    │                       #   + pub use re-exports for the 10 op modules
    ├── distributions.rs    # 3 hand-coded f64-bridge primitives (Plan 33-02):
    │                       #   norm_cdf_inv_f64 (Acklam / AS 241, ~30 LOC)
    │                       #   gamma_regularized_f64 (AS 239 series + CF, ~50 LOC)
    │                       #   beta_regularized_f64 (AS 63 CF, ~60 LOC)
    │                       #   + inline scipy.stats oracle tuples in #[cfg(test)] mod tests
    ├── basic_stats.rs      # ΣBSTAT / ΣBSTG (Plan 33-05) — consume existing R01–R06
    ├── moments.rs          # ΣMMTUG / ΣMMTGD (Plan 33-06) — extended registers per Plan 33-00
    ├── anova.rs            # ΣAOVONE / ΣAOVTWO / ΣANOCOV (Plan 33-06)
    ├── regression.rs       # ΣLIN/EXP/LOGI/POW (Plan 33-05) + ΣMLRXY/MLRXYZ/POLYP/POLYC (Plan 33-08)
    │                       #   self-contained Gauss elimination — NO math1::matrix imports
    ├── hypothesis.rs       # ΣPTST / ΣTSTAT (Plan 33-07) — name avoids #[cfg(test)] mod tests collision
    ├── nonparam.rs         # ΣSPEAR (Plan 33-04) + ΣXSQEV / ΣEFXSQ (Plan 33-04) + ΣCTKKK / ΣCTKK (Plan 33-06)
    ├── normd.rs            # ΣNORMD 3-mode dispatcher (Plan 33-03)
    ├── chisqd.rs           # ΣCHISQD ν-prompt dispatcher (Plan 33-03)
    └── rand.rs             # RAND / SEED (Plan 33-08; LCG using state.rand_seed)
```

### Pattern 1: XROM Module Registration (mirror of MATH_1)

**What:** Register `STAT_1` as a sibling `XromModule` in the same `math1/xrom.rs` file (per D-33.3).
**When to use:** Once per XROM module activation.
**Example:**
```rust
// Source: hp41-core/src/ops/math1/xrom.rs (extension target)
pub const STAT_1: XromModule = XromModule {
    id: 2,                  // Hardware XROM ID per calc.fjk.ch/db/hp41mod.php
    name: "STAT 1B",        // CATALOG 2 display string
    ops: &[
        ("\u{03A3}NORMD",  Op::SigmaNormdWorkflow),   // ΣNORMD (Σ = U+03A3)
        ("\u{03A3}CHISQD", Op::SigmaChisqdWorkflow),
        ("\u{03A3}SPEAR",  Op::SigmaSpear),
        // ... 21 more entries; Σ is the U+03A3 GREEK CAPITAL LETTER SIGMA
        ("RAND", Op::Rand),    // emulator extension (D-33.4)
        ("SEED", Op::Seed),    // emulator extension (D-33.4)
    ],
};

pub fn xrom_resolve(name: &str, modules: u8) -> Option<Op> {
    if modules & 0b0000_0001 != 0 {
        if let Some(op) = math1_resolve(name) { return Some(op); }
    }
    if modules & 0b0000_0010 != 0 {                              // ← NEW (Plan 33-01)
        if let Some(op) = stat1_resolve(name) { return Some(op); }
    }
    None
}

fn stat1_resolve(name: &str) -> Option<Op> {
    match name {
        "\u{03A3}NORMD"  => Some(Op::SigmaNormdWorkflow),
        "\u{03A3}CHISQD" => Some(Op::SigmaChisqdWorkflow),
        // ... mirror MATH_1.ops one-for-one
        _ => None,
    }
}
```

**Notes on Σ character:** Use the Unicode escape `\u{03A3}` (GREEK CAPITAL LETTER SIGMA) consistently — same convention as `\u{00D7}` (×) and `\u{2191}` (↑) in `math1/xrom.rs:57–79`. Consider an ASCII alias `"S"` prefix (e.g. `"SNORMD"`) like the `"C*"` / `"C/"` aliases at lines 58, 60. **Decision deferred to planner** — research recommends ASCII aliases for ergonomic CLI typing parity with `"C*"`/`"C/"`/`"Z^N"` etc.

### Pattern 2: f64-Bridge Numerical Primitive (mirror of `checked_asin`)

**What:** Wrap an f64 numerical primitive in a `Decimal → f64 → f64 → Decimal` round trip with explicit domain checks.
**When to use:** Whenever `rust_decimal::MathematicalOps` lacks the operation AND the algorithm is well-defined on f64 (~15.9 decimal digits — sufficient for HP-41's 10-digit display precision).
**Example:**
```rust
// Source: hp41-core/src/num.rs:184–211 (pattern reference — checked_asin)
pub fn norm_cdf_inv_f64(p: f64) -> Result<f64, HpError> {
    // Domain check
    if !(0.0..=1.0).contains(&p) {
        return Err(HpError::Domain);
    }
    // Edge cases (CDF asymptotes)
    if p == 0.0 { return Ok(f64::NEG_INFINITY); }  // OR return Err(HpError::Domain)
    if p == 1.0 { return Ok(f64::INFINITY); }      // — choose per OM behavior in Plan 33-02
    // Acklam AS 241 rational approximation — see §"Distribution Primitives"
    // ...
}
```

### Pattern 3: Modal-Prompt Workflow (ΣPOLYP DEGREE=?, SEED?, ν=?)

**What:** Use existing `print_buffer` + `modal_program` machinery from Math Pac I — no new transient `CalcState` fields beyond `rand_seed`.
**When to use:** Any Stat 1 Op that requires multi-step user input.
**Example:** See `hp41-core/src/ops/math1/poly.rs` (`POLY` modal opens `DEGREE=?` prompt; R/S submit captured by `submit_modal` in `math1/mod.rs:54–81`).

**For Stat 1 Pac modal opens (per SPEC.md Req. 22, 31, 32, 36):**
```rust
// Pseudocode for ΣCHISQD ν=? prompt
pub fn op_sigma_chisqd_workflow(state: &mut CalcState) -> Result<(), HpError> {
    state.modal_program = Some(ModalProgram::Stat1Chisqd(ChisqdStep::PromptForNu));
    state.modal_prompt = Some("\u{03BD}=?".to_string());  // ν=?
    state.print_buffer.push("\u{03BD}=?".to_string());
    Ok(())
}
```

**ModalProgram extension:** Add a new `Stat1Chisqd(ChisqdStep)`, `Stat1Polyp(PolypStep)`, `Stat1Seed` variant to `ModalProgram` in `hp41-core/src/ops/math1/modal.rs`. This is technically a math1/ file edit — but `modal.rs` is in the FROZEN set per D-33.3a. **Resolution recommendation:** define Stat 1 modal steps in `hp41-core/src/ops/stat1/modal.rs` as a NEW file, and extend `ModalProgram` enum **in `math1/modal.rs`** via a single additive variant `Stat1(stat1::modal::Stat1Step)`. This stays a single-line edit to `modal.rs` and matches the precedent of `ModalProgram::Solve`/`Matrix`/`Poly` being thin wrappers around per-module step enums. **Planner: flag this as a SECOND freeze-exception decision needing D-33.3 amendment.** Alternative: route modal state via a wholly new field on CalcState — but D-33.5 forbids new transient fields beyond `rand_seed`.

### Pattern 4: Σ-Register Delegate (curve fits → existing `op_sigma_plus`)

**What:** ΣLIN/ΣEXP/ΣLOGI/ΣPOW transform x/y then delegate to existing `op_sigma_plus()` in `hp41-core/src/ops/stats.rs:22–53`. NEVER duplicate Σ arithmetic.
**Example:**
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

### Anti-Patterns to Avoid

- **Hand-rolling Σ arithmetic in `stat1/regression.rs`** — delegate to `ops::stats::op_sigma_plus`; ΣLIN is just a tagged Σ+ call followed by retrieval of `a, b` from R01–R06.
- **Computing variance from raw register block using naive two-pass formula** — use Welford's online algorithm for any function that iterates over a raw data block (P18). See `.planning/research/PITFALLS.md` for the exact Welford snippet.
- **Floor()/fmod() on f64 for register indexing** — use named consts from `stat1/mod.rs` (e.g. `STAT1_AOV_SSB_REG: usize`); index directly via `state.regs[STAT1_AOV_SSB_REG]`. Mirror the `parse_counter()` discipline at `hp41-core/src/ops/program.rs`.
- **Using `Op::XromCall(u16)` table dispatch** — locked rejected in v3.0 ADR-001; would break 4-way exhaustive-match invariant.
- **Importing `ops::math1::matrix::*` in `stat1/regression.rs`** — cross-XROM coupling rejected per Req. 21; CI gate `grep -n "ops::math1::matrix" hp41-core/src/ops/stat1/` must return empty.
- **`#[serde(skip)]` on `rand_seed`** — P20 muscle-memory trap. Must be `#[serde(default)]` WITHOUT `skip`; all other v3.1 transient fields use `skip`.
- **Adding `println!` / `eprintln!` to any `stat1/*.rs` file** — `hp41-core` invariant; route through `state.print_buffer`.
- **Quoting / copying any Free42 source** — `core_math2.cc` may be opened as an oracle for verification but no code copying; contamination guard catches distinctive symbols.

---

## Distribution Primitives — Algorithms & Validation

This is the deepest section because Plan 33-02 ships the algorithmic foundation that Plans 33-03 + 33-07 consume.

### Algorithm 1: `norm_cdf_inv_f64` — Inverse Standard Normal CDF (Acklam / AS 241)

**Source:** Peter J. Acklam, "An algorithm for computing the inverse normal cumulative distribution function" (1999, NA-Net post; widely circulated). Public-domain. Mirrored at `stackedboxes.org/2017/05/01/acklams-normal-quantile-function/` (cited in SUMMARY.md sources). Algorithmically equivalent to AS 241 (Applied Statistics Algorithm 241, Wichura 1988, Royal Statistical Society) which is the published-paper version.

**Approach:** Rational approximation with two regions (central + tail), then optional one-step Halley refinement.

- **Central region:** `0.02425 ≤ p ≤ 0.97575`
  - `q = p − 0.5`
  - `r = q²`
  - `x = (((((a₁·r + a₂)·r + a₃)·r + a₄)·r + a₅)·r + a₆)·q / (((((b₁·r + b₂)·r + b₃)·r + b₄)·r + b₅)·r + 1)`
  - 6 numerator coefficients (a₁..a₆), 5 denominator coefficients (b₁..b₅).
- **Lower tail:** `p < 0.02425`
  - `q = sqrt(−2·ln(p))`
  - `x = (((((c₁·q + c₂)·q + c₃)·q + c₄)·q + c₅)·q + c₆) / ((((d₁·q + d₂)·q + d₃)·q + d₄)·q + 1)`
  - 6 numerator (c₁..c₆), 4 denominator (d₁..d₄).
- **Upper tail:** `p > 0.97575` — same formula with `q = sqrt(−2·ln(1−p))`, negate result.

**Coefficient table** (Acklam 1999, verified — DO NOT paraphrase or "improve"; copy-paste verbatim from the source):
```
a₁ = -3.969683028665376e+01
a₂ =  2.209460984245205e+02
a₃ = -2.759285104469687e+02
a₄ =  1.383577518672690e+02
a₅ = -3.066479806614716e+01
a₆ =  2.506628277459239e+00

b₁ = -5.447609879822406e+01
b₂ =  1.615858368580409e+02
b₃ = -1.556989798598866e+02
b₄ =  6.680131188771972e+01
b₅ = -1.328068155288572e+01

c₁ = -7.784894002430293e-03
c₂ = -3.223964580411365e-01
c₃ = -2.400758277161838e+00
c₄ = -2.549732539343734e+00
c₅ =  4.374664141464968e+00
c₆ =  2.938163982698783e+00

d₁ =  7.784695709041462e-03
d₂ =  3.224671290700398e-01
d₃ =  2.445134137142996e+00
d₄ =  3.754408661907416e+00
```

**Accuracy:** Acklam claims relative error < 1.15e-9 across the entire (0,1) domain. SPEC.md Req. 31 demands 1e-7 for the iterative path. Acklam alone meets this comfortably. **Recommendation:** ship pure Acklam without Halley refinement first; add one Halley step only if Plan 33-02 oracle tests fail the 1e-7 bound for any tuple.

**Convergence-cancellation note:** Acklam is **closed-form**, NOT iterative. The `cancel_requested` and 50-iteration-cap discipline DO NOT apply to `norm_cdf_inv_f64` itself — they apply only to the OUTER ΣNORMD inverse Op when bisection refinement is used for tail-edge cases (SPEC.md Req. 34). Plan 33-03 wires the outer loop; Plan 33-02 ships only the closed-form Acklam.

### Algorithm 2: `gamma_regularized_f64` — Regularized Lower Incomplete Gamma P(a,x) (AS 239)

**Source:** Numerical Recipes 3rd ed. §6.2 (`gammp` / `gser` / `gcf`); algorithm equivalent to AS 239 (Applied Statistics Algorithm 239, Shea 1988, Royal Statistical Society). Numerical Recipes 3e is the cleanest published reference; the 1988 AS 239 paper is the original.

**Approach:** Two algorithms with a switch at `x < a+1`:

- **Series (`gser`):** for `x < a + 1`, use the power series
  ```
  P(a, x) = (e^(-x) · x^a / Γ(a+1)) · Σ_{n=0..∞} x^n / (a+1)(a+2)...(a+n+1)
  ```
  Terminate when next term < ε × current partial sum.

- **Continued fraction (`gcf`):** for `x ≥ a + 1`, use the modified Lentz algorithm on the continued fraction for `Γ(a,x) = Γ(a) − γ(a,x)`:
  ```
  Q(a, x) = (e^(-x) · x^a / Γ(a)) · CF[ 1/(x+1-a) - 1·(1-a) / (x+3-a) - 2·(2-a) / (x+5-a) - ... ]
  P(a, x) = 1 - Q(a, x)
  ```

**Implementation notes:**
- `ln Γ(a)` computed via Stirling series or AS 245 (Lanczos coefficients) — Numerical Recipes §6.1 `gammln`. Required as a helper because direct `Γ(a)` overflows for `a > 170`. Pre-compute `ln Γ(a)` once per Op call.
- Convergence ε = ~1e-14 (machine epsilon for f64); SPEC.md demands 1e-7 oracle agreement.
- Iteration cap: Numerical Recipes uses 100; SPEC.md demands 50. Use 50 per SPEC.md Req. 34.
- **Cancellation hook:** the outer Op loop (ΣCHISQD CDF, Plan 33-03) calls `gamma_regularized_f64` then checks `cancel_requested`. The primitive itself runs to completion (single function call, < 50 iters) — no in-primitive cancel check needed.

**Accuracy:** Numerical Recipes 3e quotes relative error ~1e-15 across the full domain. SPEC.md Req. 32 demands 1e-7 — comfortable margin.

### Algorithm 3: `beta_regularized_f64` — Regularized Incomplete Beta I_x(a,b) (AS 63)

**Source:** Numerical Recipes 3rd ed. §6.4 (`betai` / `betacf`); algorithm equivalent to AS 63 (Applied Statistics Algorithm 63, Majumder & Bhattacharjee 1973, Royal Statistical Society). AS 109 / AS 110 are the inverse versions (NOT required here — Stat 1 Pac uses only the forward I_x(a,b) for t-test p-values).

**Approach:** Continued fraction via modified Lentz; one-sided rewrite for numerical stability:

- If `x < (a+1)/(a+b+2)`: compute `I_x(a,b)` directly.
- Else: compute `1 − I_{1-x}(b,a)` (symmetric identity).

The continued fraction:
```
I_x(a,b) = (x^a · (1-x)^b / (a · B(a,b))) · CF[ 1 + d_1 / (1 + d_2 / (1 + ... )) ]
```
where `d_{2m+1} = -(a+m)(a+b+m)·x / ((a+2m)(a+2m+1))`, `d_{2m} = m·(b-m)·x / ((a+2m-1)(a+2m))`, and `B(a,b) = Γ(a)·Γ(b)/Γ(a+b)`.

**Implementation notes:**
- `ln B(a,b) = ln Γ(a) + ln Γ(b) − ln Γ(a+b)` — uses same `gammln` helper as Algorithm 2.
- Iteration cap: 50 per SPEC.md Req. 34.
- Accuracy: Numerical Recipes 3e quotes ~1e-15; SPEC.md demands 1e-7 — comfortable margin.

**Cross-link:** ΣPTST (one-sample t) and ΣTSTAT (pooled-variance two-sample t) compute t-statistic in `HpNum` arithmetic, then route through `beta_regularized_f64` for the p-value. The Student-t CDF is:
```
P(T ≤ t) = 1 − ½ · I_{ν/(ν+t²)}(ν/2, ½)   for t ≥ 0
         =     ½ · I_{ν/(ν+t²)}(ν/2, ½)   for t < 0
```
Two-sided p-value = `2 · I_{ν/(ν+t²)}(ν/2, ½)` (regardless of sign).

### Validation Table (the planner lifts these into per-Op acceptance criteria)

All `scipy_expected` values are exact at f64 precision. The Python snippet that generates them is one-shot and lives only in plan comments (D-33.6). Tolerance band per SPEC.md Req. 46 (1e-9 closed-form, 1e-7 iterative).

| # | Primitive / Op | Input | scipy_expected | Tolerance | Oracle command |
|---|----------------|-------|----------------|-----------|----------------|
| 1 | `norm_cdf_inv_f64` (closed) | p=0.025 | -1.959963984540054 | 1e-9 | `scipy.stats.norm.ppf(0.025)` |
| 2 | `norm_cdf_inv_f64` (closed) | p=0.975 | 1.959963984540054 | 1e-9 | `scipy.stats.norm.ppf(0.975)` |
| 3 | `norm_cdf_inv_f64` (lower tail) | p=0.001 | -3.090232306167813 | 1e-9 | `scipy.stats.norm.ppf(0.001)` |
| 4 | `norm_cdf_inv_f64` (upper tail) | p=0.999 | 3.090232306167813 | 1e-9 | `scipy.stats.norm.ppf(0.999)` |
| 5 | `norm_cdf_inv_f64` (central) | p=0.5 | 0.0 | 1e-12 | `scipy.stats.norm.ppf(0.5)` (exact) |
| 6 | `norm_cdf_inv_f64` (deep tail) | p=1e-6 | -4.753424308822543 | 1e-9 | `scipy.stats.norm.ppf(1e-6)` |
| 7 | `gamma_regularized_f64` (series) | s=2, x=1 | 0.2642411176571153 | 1e-9 | `scipy.special.gammainc(2, 1)` |
| 8 | `gamma_regularized_f64` (CF) | s=2, x=10 | 0.9995006007726127 | 1e-9 | `scipy.special.gammainc(2, 10)` |
| 9 | `gamma_regularized_f64` (CF) | s=1.5, x=5 | 0.9595723180054873 | 1e-9 | `scipy.special.gammainc(1.5, 5)` |
| 10 | `gamma_regularized_f64` (large a) | s=50, x=50 | 0.5188083154720433 | 1e-9 | `scipy.special.gammainc(50, 50)` |
| 11 | `beta_regularized_f64` (symmetric) | a=2, b=2, x=0.5 | 0.5 | 1e-12 | `scipy.special.betainc(2, 2, 0.5)` (exact) |
| 12 | `beta_regularized_f64` (skew) | a=0.5, b=0.5, x=0.25 | 0.3333333333333333 | 1e-9 | `scipy.special.betainc(0.5, 0.5, 0.25)` |
| 13 | `beta_regularized_f64` (t-test bridge) | a=2.5, b=0.5, x=5/(5+0) | 1.0 | 1e-12 | `scipy.special.betainc(2.5, 0.5, 1)` |
| 14 | ΣNORMD CDF (closed, Op level) | x=1.96 | 0.024998 | 1e-9 | `scipy.stats.norm.sf(1.96)` |
| 15 | ΣNORMD PDF (closed, Op level) | x=0 | 0.39894228040143270 | 1e-9 | `scipy.stats.norm.pdf(0)` |
| 16 | ΣNORMD inverse (iterative, Op level) | p=0.025 | -1.959963984540054 | 1e-7 | `scipy.stats.norm.ppf(0.025)` |
| 17 | ΣCHISQD CDF (iterative, Op level) | x=7.815, ν=3 | 0.9499718909781536 | 1e-7 | `scipy.stats.chi2.cdf(7.815, 3)` |
| 18 | ΣCHISQD PDF (closed, Op level) | x=3, ν=3 | 0.15418032980365303 | 1e-9 | `scipy.stats.chi2.pdf(3, 3)` |

**Minimum 6 tuples per primitive per SPEC.md Req. 33.** Above gives 6 for Acklam, 4 for AS 239, 3 for AS 63 — Plan 33-02 adds at least 2 more per primitive to meet the floor. Recommended additions: edge cases at domain boundaries (`p=0.5+1e-15` for Acklam; `s=0.5, x=0.001` for `gamma_regularized_f64`; `a=10, b=10, x=0.5` for `beta_regularized_f64`).

---

## Σ-Register Layout

### Existing R01–R06 (v1.x, in `ops/stats.rs`)

```
regs[1] = Σx²    regs[2] = Σx    regs[3] = n
regs[4] = Σy²    regs[5] = Σy    regs[6] = Σxy
```

**Existing SIZE floor guard at `ops/stats.rs:25`:** `if state.regs.len() < 7 { return Err(HpError::InvalidOp); }`.

### Extended Stat 1 Pac registers (Plan 33-00 transcribes from OM)

The OM "Storage Registers" section is the SINGLE source of truth (per D-33.1 item 1 + SPEC.md Req. 9). It is NOT inlined in SPEC.md or this RESEARCH.md — Plan 33-00 reads OM 00041-90030 and writes the verbatim transcription as a `//!` doc-comment block in `hp41-core/src/ops/stat1/mod.rs`.

**What this research can say authoritatively:**
- ΣMMTUG / ΣMMTGD extend the Σ-register block with at least Σx³ and Σx⁴ (third and fourth raw moments). Likely register slots R07–R08 or per-OM offset.
- ΣAOVONE / ΣAOVTWO / ΣANOCOV use group-keyed sums (per-group Σx, Σx², n). For one-way ANOVA with k groups, register requirement grows as O(3k). NPS document Table ZA-3 example uses 3 groups × 3 fields = 9 registers + global accumulators. Plan 33-00 reads OM "Storage Registers" for the exact indices.
- ΣMLRXY (2 predictors) needs cross-product terms: Σx₁, Σx₂, Σy, Σx₁², Σx₂², Σx₁x₂, Σx₁y, Σx₂y, n — 9 registers minimum.
- ΣMLRXYZ (3 predictors) needs 14 minimum (3 means + 6 cross-products + 3·y cross + n).
- ΣCTKKK uses r·c registers for the contingency table + r+c marginal totals.

**`STAT1_MAX_REG` const** in `stat1/mod.rs` is computed as the MAX over all Stat 1 register requirements. Likely value: **`pub const STAT1_MAX_REG: usize = ???;`** (Plan 33-00 fills the number from OM).

**SIZE floor guard pattern** (every Stat 1 Op that reads from extended registers):
```rust
if state.regs.len() < stat1::STAT1_MAX_REG + 1 {
    return Err(HpError::InvalidOp);
}
```

### `[C]` correction-key extension (STAT-UNI-04)

Every register touched by a Stat 1 accumulation Op must have a matching reversal in `op_sigma_minus` (existing function at `ops/stats.rs:58`). If ΣMMTUG accumulates into new R07+ registers, `op_sigma_minus` extension is required. **Plan 33-05 / 33-06 task:** extend `op_sigma_minus` to mirror every new accumulator. Acceptance test: round-trip property `sigma_plus_then_minus_restores_state` extended for the new registers.

---

## Module Wiring — Exact File Edits

### Plan 33-01 file-level diff (XROM framework activation)

| File | Change | LOC est |
|------|--------|---------|
| `hp41-core/src/ops/math1/xrom.rs` | Add `STAT_1: XromModule` const (~30 line slice + 26 entries); add `stat1_resolve(name)` match arm (~26 lines); add bit-1 arm in `xrom_resolve` (~3 lines) | +~70 |
| `hp41-core/src/state.rs:228` | Change `0b0000_0001` → `0b0000_0011` (1 line) | ±0 net |
| `hp41-core/src/state.rs` | Add `migrate_after_load()` method (~5 lines) | +~5 |
| `hp41-core/src/state.rs` | Add `#[serde(default)] pub rand_seed: HpNum` field (~3 lines); update `CalcState::new()` to initialize | +~5 |
| `hp41-core/src/ops/mod.rs` | Add `pub mod stat1;` (1 line) | +~1 |
| `hp41-core/src/ops/stat1/mod.rs` | NEW file — module hub + `pub use` re-exports (skeleton until 33-00 fills OM transcription) | ~30 |
| `hp41-core/tests/xrom_shadowing.rs` | Extend to assert `STAT_1.ops` disjoint from `MATH_1.ops` AND builtins (~20 lines) | +~20 |

**Test additions in Plan 33-01:**
- `hp41-core/src/state.rs::tests::xrom_modules_default_is_three` — asserts `CalcState::new().xrom_modules == 0b0000_0011`
- `hp41-core/src/state.rs::tests::migrate_after_load_sets_bit_1` — JSON blob with `"xrom_modules": 1` → `migrate_after_load()` → asserts `0b0000_0011`
- `hp41-core/src/state.rs::tests::migrate_after_load_idempotent` — already-`3` state → no-op
- `hp41-core/src/state.rs::tests::rand_seed_serde_round_trip` — set seed, serialize, deserialize, assert equal (proves `#[serde(default)]` without `#[serde(skip)]`)
- `hp41-core/src/ops/math1/xrom.rs::tests::stat1_const_id_and_name` — `STAT_1.id == 2`, `STAT_1.name == "STAT 1B"`
- `hp41-core/src/ops/math1/xrom.rs::tests::xrom_resolve_sigma_normd_with_bit_1_loaded` — `xrom_resolve("ΣNORMD", 0b0000_0011)` returns `Some(...)`
- `hp41-core/src/ops/math1/xrom.rs::tests::xrom_resolve_sigma_normd_bit_1_clear_returns_none` — bit-1 isolation

### Plan 33-03..33-08 incremental edits

Each plan adds:
- 1–4 files in `hp41-core/src/ops/stat1/`
- 1–4 new Op variants in `hp41-core/src/ops/mod.rs` (the `Op` enum)
- 1–4 new arms in `hp41-core/src/ops/mod.rs::dispatch()` (item 1 of 4-way invariant)
- 1–4 new arms in `hp41-core/src/ops/program.rs::execute_op()` (item 2 of 4-way invariant)
- 1–4 new entries in `STAT_1.ops` slice in `math1/xrom.rs`
- 1–4 new match arms in `stat1_resolve()` in `math1/xrom.rs`

### 4-Way Exhaustive-Match — Items 1 + 2 (Phase 33 owns these)

| File | Function | Required for Phase 33 | Phase that completes |
|------|----------|----------------------|---------------------|
| `hp41-core/src/ops/mod.rs` | `dispatch()` (line ~960 — the big match) | ✓ (item 1) | Phase 33 |
| `hp41-core/src/ops/program.rs` | `execute_op()` (line ~900 — the big match) | ✓ (item 2) | Phase 33 |
| `hp41-cli/src/prgm_display.rs` | `op_display_name()` | ✗ | Phase 34 (item 3) |
| `hp41-gui/src-tauri/src/prgm_display.rs` | `op_display_name()` | ✗ | Phase 36 (item 4) |

**Compile-time enforcement:** Items 3 + 4 will not compile until added — but Phase 33 only needs to ensure items 1 + 2 compile. `cargo check -p hp41-core` passes with items 1 + 2; `cargo check -p hp41-cli` fails until Phase 34; `cargo check -p hp41-gui` fails until Phase 36. **Phase 33 acceptance:** `cargo check -p hp41-core` clean; failure of dependent crates is EXPECTED.

### Resolver-chain shadow-prevention test (Plan 33-01 extension)

`hp41-core/tests/xrom_shadowing.rs` currently asserts every `MATH_1.ops` mnemonic is:
1. Disjoint from `builtin_card_op` keys (no shadowing).
2. Resolved to the same Op via `xrom_resolve` and via the static `ops` slice.

**Extend for Plan 33-01:**
- Cross-check every `STAT_1.ops` mnemonic against `MATH_1.ops` mnemonics (no Stat 1 entry can shadow a Math 1 entry).
- Cross-check every `STAT_1.ops` mnemonic against `builtin_card_op` keys (no Stat 1 entry can shadow a built-in).
- Resolution consistency: `xrom_resolve(name, 0b0000_0011)` returns the same `Op` as `STAT_1.ops` slice lookup.

**Suspected non-shadowing (cross-checked via codebase, HIGH confidence):**
- Σ-prefixed names (ΣNORMD, ΣCHISQD, ΣSPEAR, ΣBSTAT, etc.) are entirely new — no existing built-in uses U+03A3.
- `RAND` and `SEED` are not currently in `builtin_card_op` (verify in Plan 33-01).
- v2.2 has `MEAN`, `SDEV`, `CORR`, `L.R.`, `YHAT` as built-ins per `ops/stats.rs` — Stat 1 Pac mnemonics start with Σ, so no collision.

---

## State Migration & RNG Seed Persistence

### `CalcState::migrate_after_load()` (Plan 33-01, D-33.7)

**Exact body:**
```rust
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
        // v3.0 save files explicitly stored `"xrom_modules": 1` (Math 1 only).
        // Bumping `default_xrom_modules()` to 0b0000_0011 does NOT retroactively
        // change stored values — serde uses the stored value, not the new default.
        // Unconditional bit-set is idempotent (no-op when already set).
        if self.xrom_modules & 0b0000_0010 == 0 {
            self.xrom_modules |= 0b0000_0010;
        }
    }
}
```

**Re-save:** Migration sets the bit in memory only. The next auto-save (30s tick or exit save) persists the migrated value. No explicit re-save call from `migrate_after_load()`.

**Wiring sites (out of Phase 33 scope but documented for planner):**
- Phase 34 plan task: at the END of `hp41-cli/src/persistence.rs::load_state`, after `serde_json::from_str` succeeds, call `state.migrate_after_load()`.
- Phase 36 plan task: at the END of `hp41-gui/src-tauri/src/persistence.rs` load path, same call.

### `rand_seed: HpNum` field (Plan 33-01, P20)

**Exact placement** (after the existing `xrom_modules` field at `state.rs:163` — group with v3.1 additions):
```rust
// ── Phase 33 (v3.1): Stat 1 Pac additions ────────────────────────────────
/// RNG seed for RAND/SEED Op family. Persistent across save/load so
/// reproducible simulations work across sessions (D-33.4 + STAT-RNG-03).
/// SERDE SHAPE: `#[serde(default)]` WITHOUT `skip`. This is the ONLY new
/// v3.1 CalcState field with this combination. All other transient v3.1
/// fields (if any are added) use `#[serde(default, skip)]`.
/// P20 muscle-memory trap: `#[serde(skip)]` would break determinism across
/// save/load round-trips and silently fail Stat-RNG-03 acceptance.
#[serde(default)]
pub rand_seed: HpNum,
```

**Default initialization in `CalcState::new()`:**
```rust
rand_seed: HpNum::zero(),  // Documented OM default — Plan 33-08 may override
```

**Why `HpNum` not `f64`:** The LCG formula `r_{n+1} = FRC(9821·r_n + 0.211327)` runs in `rust_decimal` exactly — `9821·r + 0.211327` then `FRC` (existing `op_frc` precedent in `hp41-core/src/ops/math.rs`). Acceptance Req. 35 demands 1e-15 (decimal-exact, no f64 conversion).

**SEED submit-modal extension:** SEED Op opens an ALPHA prompt `SEED?` and writes the submitted value to `state.rand_seed` via the existing `submit_modal()` machinery. Plan 33-08 adds the `ModalProgram::Stat1Seed` variant (see Pattern 3 note re: math1/modal.rs freeze exception).

### Backward compat — save-file round-trip test

**Plan 33-01 acceptance test** (`hp41-core/src/state.rs::tests`):
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
fn migrate_after_load_idempotent() {
    let mut state = CalcState::new();
    assert_eq!(state.xrom_modules, 0b0000_0011);  // default already includes bit 1
    state.migrate_after_load();
    assert_eq!(state.xrom_modules, 0b0000_0011);  // unchanged
}

#[test]
fn rand_seed_serde_round_trip() {
    let mut state = CalcState::new();
    state.rand_seed = HpNum::from_str("0.7").unwrap();
    let blob = serde_json::to_string(&state).unwrap();
    let restored: CalcState = serde_json::from_str(&blob).unwrap();
    assert_eq!(restored.rand_seed, state.rand_seed);
}
```

---

## ANOVA Family

### Per-OM-Layout Implementation Discipline (P21)

ANOVA / ANCOVA register layouts are NOT yet transcribed; Plan 33-00 deliverable. This section documents the algorithms; per-Op register names are placeholders to be replaced by `STAT1_AOV_*_REG` const names from `stat1/mod.rs` once Plan 33-00 transcription lands.

### ΣAOVONE — One-Way ANOVA (STAT-AOV-01, Req. 11)

**Inputs:** k groups, each group i has nᵢ samples; total N = Σnᵢ.
**Per-group accumulators** (from OM "Storage Registers"): Σxᵢ, Σxᵢ², nᵢ for each group i.
**Algorithm:**
- Group means: x̄ᵢ = Σxᵢ / nᵢ.
- Grand mean: x̄ = ΣΣxᵢ / N.
- Between-group SS: SSB = Σ nᵢ (x̄ᵢ − x̄)².
- Within-group SS: SSW = Σ (Σxᵢ² − nᵢ x̄ᵢ²).
- df_between = k − 1; df_within = N − k.
- F = (SSB / df_between) / (SSW / df_within).

**Acceptance (Req. 11):** 3 groups of 5 samples `[1,2,3,4,5]`, `[6,7,8,9,10]`, `[11,12,13,14,15]` → F = 100.0 ± 1e-9.
**Verification:** `scipy.stats.f_oneway([1,2,3,4,5], [6,7,8,9,10], [11,12,13,14,15]).statistic == 100.0` (exact).

### ΣAOVTWO — Two-Way ANOVA, No Replications (STAT-AOV-02, Req. 12)

**Inputs:** r × c data matrix (one observation per cell).
**Algorithm:**
- Row sums Rᵢ, column sums Cⱼ, grand sum T = ΣΣ xᵢⱼ.
- Correction factor CF = T² / (r·c).
- SS_total = ΣΣ xᵢⱼ² − CF.
- SS_row = Σ Rᵢ² / c − CF.
- SS_col = Σ Cⱼ² / r − CF.
- SS_error = SS_total − SS_row − SS_col.
- df_row = r − 1; df_col = c − 1; df_error = (r−1)(c−1).
- F_row = (SS_row / df_row) / (SS_error / df_error).
- F_col = (SS_col / df_col) / (SS_error / df_error).

**Acceptance (Req. 12):** F_row and F_col within 1e-7 relative tolerance vs `scipy.stats.f_oneway` chained reference. Iterative path because cross-product sums chain decimal multiplication.

### ΣANOCOV — One-Way ANCOVA (STAT-AOV-03, Req. 13)

**Inputs:** k groups; each observation is (x, y) pair where x is the response and y is the covariate.
**Algorithm (OM 00041-90030 § "ANCOVA" — Plan 33-06 reads):**
- Compute pooled within-group regression coefficient b_w = SSxy_within / SSyy_within.
- Adjusted sum of squares: SS_adj = SS_between(x) − b_w² · SS_between(y) + adjustment terms.
- F-ratio from adjusted SS / df.
- Reference: NPS document ZA-3 example.

**Acceptance (Req. 13):** F within 1e-7 vs NPS ZA-3 example or scipy.stats reference.

### Output Channel (deferred per CONTEXT.md Claude's Discretion)

OM "Results" section read in Plan 33-06 decides whether F-ratio + group means push to stack vs route through `print_buffer`. If OM is silent, follow Math Pac I `POLY` pattern (stack pushes for primary numeric results; print buffer for prompts). The 4 stack registers can hold: X = F-ratio, Y = df_within, Z = df_between, T = SS_total (or per-OM ordering).

---

## Hypothesis Tests

### ΣPTST — One-Sample t-Test (STAT-HYP-01, Req. 24)

**Algorithm:**
- t = (x̄ − μ₀) / (s / √n).
- df = n − 1.
- Two-sided p = 2 · I_{ν/(ν+t²)}(ν/2, ½).
- Reads from existing R01–R06 Σ-register state populated by `op_sigma_plus`.

**Input protocol:** μ₀ in X register at call time. Output: t in X, p in Y (or per OM "Results").

**Acceptance (Req. 24):** `data=[1,2,3,4,5]`, μ₀=3 → t = 0.0, p = 1.0 ± 1e-7.

### ΣTSTAT — Pooled-Variance Two-Sample t-Test (STAT-HYP-02, Req. 25)

**LOCKED: pooled variance.** Welch's t-test EXPLICITLY EXCLUDED per REQUIREMENTS.md Out-of-Scope table. If Plan 33-07 OM read shows Welch, becomes a documented divergence in Phase 35 (not a SPEC.md change).

**Algorithm:**
- Pooled variance: s²_p = ((n₁−1)·s₁² + (n₂−1)·s₂²) / (n₁ + n₂ − 2).
- t = (x̄₁ − x̄₂) / √(s²_p · (1/n₁ + 1/n₂)).
- df = n₁ + n₂ − 2 (integer).
- p = 2 · I_{ν/(ν+t²)}(ν/2, ½).

**Input protocol:** Group 1 and group 2 accumulated separately (likely via per-group Σ-register blocks per OM Plan 33-00 transcription).

**Acceptance (Req. 25):** `g1=[1,2,3,4,5], g2=[6,7,8,9,10]` → t ≈ −5.0, p ≈ 0.0010534 ± 1e-7.
**Verification:** `scipy.stats.ttest_ind([1,2,3,4,5], [6,7,8,9,10], equal_var=True)`.

### ΣXSQEV / ΣEFXSQ — Chi-Square Goodness-of-Fit (STAT-HYP-03..04, Req. 26..27)

**Algorithm:** χ² = Σ (O − E)² / E, then dispatched to caller. The χ² VALUE is computed in Stat 1; the p-value computation is the user's responsibility (chain via ΣCHISQD CDF).

**Acceptance (Req. 26):** `obs=[10,20,30], exp=[15,20,25]` → χ² = 2.667 ± 1e-9.
**Acceptance (Req. 27):** `proportions p=[0.2, 0.3, 0.5]` against `obs=[10, 30, 60]` → internally Σf = 100, expected = [20, 30, 50], χ² ≈ 1.667 ± 1e-9.

### ΣCTKKK / ΣCTKK — Contingency Table χ² (STAT-HYP-05..06, Req. 28..29)

**Algorithm:**
- Expected_ij = (Row_i · Col_j) / Grand_total.
- χ² = ΣΣ (O_ij − E_ij)² / E_ij.
- df = (r−1)(c−1).

**Acceptance (Req. 28):** 2×3 table `[[10,20,30],[40,50,60]]` → χ² ≈ 4.286 ± 1e-9.
**Acceptance (Req. 29):** 2×2 table `[[10,20],[30,40]]` → χ² ≈ 0.397 ± 1e-9.

### ΣSPEAR — Spearman Rank Correlation (STAT-HYP-07, Req. 30)

**Algorithm (closed-form, simplest of all 13 programs):**
- ρ_s = 1 − 6 · Σd² / (n·(n² − 1)), where dᵢ = rank(xᵢ) − rank(yᵢ).
- Pre-condition: user has already ranked the data (Stat 1 does NOT rank for them per OM).

**Acceptance (Req. 30):** `ranks_x=[1,2,3,4,5], ranks_y=[2,1,3,5,4]` → Σd² = 1+1+0+1+1 = 4 → ρ_s = 1 − 24/120 = 0.8 (note: SPEC.md says 0.7 — verify in Plan 33-04). Tolerance 1e-9.

---

## Regression Family

### ΣLIN / ΣEXP / ΣLOGI / ΣPOW (STAT-REG-01..04, Req. 15..18)

Standard log-linearization. Each Op transforms x and/or y in place THEN delegates to `op_sigma_plus` (existing v1.x machinery at `ops/stats.rs:22`). Coefficients `a, b` retrieved from R01–R06 via the existing `op_linear_reg` formula.

| Op | Transform applied | OM mnemonic | a, b extraction |
|----|-------------------|-------------|-----------------|
| ΣLIN | none | ŷ = a + b·x | standard |
| ΣEXP | y ← ln y | ŷ = a·e^(b·x) → exp(a), b | a' = e^a |
| ΣLOGI | x ← ln x | ŷ = a + b·ln x | direct a, b |
| ΣPOW | x ← ln x, y ← ln y | ŷ = a·x^b → exp(a), b | a' = e^a |

**Acceptance ranges:** SPEC.md Req. 15–18 each give a closed-form oracle (e.g. ΣLIN `x=[1,2,3,4,5], y=[2,4,6,8,10]` → a=0, b=2, tol 1e-9).

### ΣMLRXY / ΣMLRXYZ — Multiple Regression (STAT-REG-05..06, Req. 19..20)

**Algorithm:** Solve the normal equations:
```
[Σx₁²  Σx₁x₂  Σx₁  ] [b₁]   [Σx₁y]
[Σx₁x₂ Σx₂²  Σx₂  ] [b₂] = [Σx₂y]
[Σx₁   Σx₂   n    ] [b₀]   [Σy  ]
```
(intercept appended; for ΣMLRXYZ extend to 3 predictors + intercept = 4×4 system).

**Self-contained Gauss elimination — local to `stat1/regression.rs` per Req. 21:**
```rust
// Pseudocode — Plan 33-08
fn solve_normal_equations_2x2(
    s_x1_sq: HpNum, s_x1_x2: HpNum, s_x1: HpNum,
    s_x2_sq: HpNum, s_x2: HpNum,
    n: HpNum,
    s_x1_y: HpNum, s_x2_y: HpNum, s_y: HpNum,
) -> Result<(HpNum, HpNum, HpNum), HpError> {
    // Standard Gauss elimination with partial pivoting on a 3×3 augmented matrix.
    // All arithmetic in HpNum (rust_decimal-backed). Returns (b₀, b₁, b₂) or
    // Err(HpError::Domain) if matrix is singular.
}
```

**CI gate:** `grep -n "ops::math1::matrix" hp41-core/src/ops/stat1/` returns empty (Req. 21 acceptance).

**Tolerance:** 1e-7 relative (iterative — chained decimal multiplication can lose precision).

### ΣPOLYP / ΣPOLYC — Polynomial Regression (STAT-REG-07..08, Req. 22..23)

**ΣPOLYP** opens modal prompt for degree d (tentative `DEGREE=?`; OM override may apply per Plan 33-08 read). After d collected, user accumulates (x, y) pairs; on completion, fits ŷ = a₀ + a₁x + a₂x² + ... + a_d·x^d via (d+1)×(d+1) normal equations + Gauss elimination.

**ΣPOLYC** evaluates ŷ at a given x using Horner's method on the most-recent ΣPOLYP coefficient set.

**Acceptance (Req. 22 + 23):** d=2, `x=[1,2,3,4,5], y=[1,4,9,16,25]` → (a₀=0, a₁=0, a₂=1) ± 1e-7; ΣPOLYC at x=6 → ŷ = 36.0 ± 1e-9.

**Storage for coefficients between ΣPOLYP and ΣPOLYC:** Use designated registers per OM (Plan 33-08 reads). Likely R20–R26 or similar (HP Math Pac I `POLY` uses R00–R05 for coefficient storage — Plan 33-08 confirms whether Stat 1 follows same convention or uses different range).

---

## RAND / SEED (Emulator Extension)

### LCG Formula (NPS p. 21, Don Malm, HP-65 User's Library)

```
r_{n+1} = FRC(9821 · r_n + 0.211327)
```

Where `FRC(x)` = fractional part = `x − int(x)`. Existing `op_frc` precedent in `hp41-core/src/ops/math.rs::op_frc`.

**Three-source community confirmation:**
1. Naval Postgraduate School NPS55-84-003 (Zehna 1984) p. 21 `[CITED: archive.org/dtic AD-A140573]`
2. Don Malm, HP-65 User's Library — referenced by NPS as the formula's origin
3. HP-41C Standard Applications manual p. 24 — secondary corroboration

### Implementation (Plan 33-08)

```rust
// hp41-core/src/ops/stat1/rand.rs (pseudocode — Plan 33-08)
pub fn op_rand(state: &mut CalcState) -> Result<(), HpError> {
    let multiplier = HpNum::from_str("9821").map_err(|_| HpError::Domain)?;
    let increment = HpNum::from_str("0.211327").map_err(|_| HpError::Domain)?;
    let stepped = state.rand_seed.checked_mul(&multiplier)?
        .checked_add(&increment)?;
    let new_seed = stepped.checked_sub(&stepped.checked_int()?)?;  // FRC
    state.rand_seed = new_seed.clone();
    state.stack.lift_enabled = true;
    enter_number(state, new_seed);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}

pub fn op_seed(state: &mut CalcState) -> Result<(), HpError> {
    // Open modal prompt "SEED?" — submit_modal writes to state.rand_seed
    state.modal_program = Some(ModalProgram::Stat1(Stat1Step::SeedPrompt));
    state.modal_prompt = Some("SEED?".to_string());
    state.print_buffer.push("SEED?".to_string());
    Ok(())
}
```

**Acceptance (Req. 35 + 37):**
- `state.rand_seed = HpNum::from_str("0.5").unwrap()` → 1st RAND → `FRC(9821·0.5 + 0.211327) = FRC(4910.711327) = 0.711327`.
- Save state, deserialize, repeat: same `0.711327`.
- Determinism: identical RAND sequences from identical seeds.

### `checked_int` helper

`HpNum::checked_int` does NOT currently exist in `hp41-core/src/num.rs`. Either:
- (a) Add it: `pub fn checked_int(&self) -> Result<HpNum, HpError> { Ok(HpNum::rounded(self.0.trunc())) }` (rust_decimal `trunc()` is the FRC-compatible integer floor toward zero).
- (b) Reuse `op_int` from `hp41-core/src/ops/math.rs` — but that operates on `state.stack.x` not on a free `HpNum`.

**Recommendation:** Add `HpNum::checked_int` as a small helper in Plan 33-08. Same f64-bridge OR pure-decimal approach as `checked_frc` would use.

---

## JSON Registry, Help Overlay, Matrix (out of Phase 33 scope — Phase 34/35 work)

**Phase 33 does NOT ship `docs/hp41-stat1-functions.json`.** That's Phase 34 / 35 work. Documented here so the planner knows the downstream contract:

| Phase 34 task | Schema |
|---------------|--------|
| Create `docs/hp41-stat1-functions.json` | Same `HelpEntry` schema as `docs/hp41-math1-functions.json`: `op_variant, display_name, category, status, phase, key_path, description, divergences, xrom` |
| Add `STAT1_HELP_ENTRIES: OnceLock<Vec<HelpEntry>>` in `hp41-cli/src/help_data.rs` | Mirror `MATH1_HELP_ENTRIES` at line 107 |
| Add `help_entries_stat1() -> &'static [HelpEntry]` narrow accessor | Mirror `help_entries_math1()` at line 117 |
| Extend `help_entries_all()` to chain all three pools | One additional `.chain(help_entries_stat1())` call |

| Phase 35 task | Action |
|---------------|--------|
| `scripts/docs-matrix/src/main.rs` | Add title-dispatch branch: `else if basename.ends_with("hp41-stat1-functions.json") { ("# HP-41C Stat Pac I Function Matrix", ...) }` |
| `justfile` | Add `docs-stat1` and `docs-stat1-check` recipes (mirror `docs-math1` precedent) |

**Right-panel filter:** `key_ref_entries()` in `hp41-cli/src/keys.rs` excludes XROM-module functions via `entry.xrom.is_none()`. Stat 1 Pac entries carry `xrom: { module: "Stat 1", module_id: 2, function_id: N }`, so they're EXCLUDED from the right panel — discoverable only via `?` overlay's "Stat 1 Pac (XROM 2)" section. Matches v3.0 Math Pac I policy.

---

## Common Pitfalls

### Pitfall 1: ANOVA / regression register layout guessed instead of OM-transcribed (P21)
**What goes wrong:** Stat 1 ANOVA/MMTUG/MLRXY/CTKKK Ops read from registers R07+. If indices don't match the OM table, computations produce silently wrong results that pass type checks and may even pass naive test data that happens to land in matching cells.
**Why it happens:** OM is a PDF; quick implementation feels easier than transcription.
**How to avoid:** Plan 33-00 transcribes OM "Storage Registers" verbatim as `//!` doc-comment in `stat1/mod.rs` AND defines `pub const STAT1_AOV_SSB_REG: usize = N;` (and similar for every layout-dependent index). Every Op uses the named const. No literal-integer register indices in `stat1/anova.rs`, `stat1/moments.rs`, `stat1/regression.rs`, `stat1/nonparam.rs`.
**Warning signs:** Code reviewer sees `state.regs[7]`, `state.regs[8]`, etc. in `stat1/*.rs` with no `STAT1_*` const name.

### Pitfall 2: Forgetting `migrate_after_load()` wiring in Phase 34/36 (P24 carry-forward)
**What goes wrong:** Phase 33 ships `migrate_after_load()` in core but doesn't wire it into the CLI/GUI persistence loaders. v3.0 users open the v3.1 binary, load their save, and bit 1 stays clear — Stat 1 Pac silently disabled.
**Why it happens:** Phase 33 acceptance criteria don't include wiring (that's Phase 34/36 work).
**How to avoid:** Plan 33-01 doc-comment on `migrate_after_load()` explicitly enumerates the two wiring sites with phase IDs. Phase 34 + Phase 36 plan tasks include "call `state.migrate_after_load()` after `serde_json::from_str` succeeds" as explicit acceptance items.
**Warning signs:** Phase 34 / 36 plan reviewer doesn't see a `migrate_after_load()` line in persistence diff.

### Pitfall 3: `#[serde(skip)]` accidentally added to `rand_seed` (P20)
**What goes wrong:** RAND determinism fails across save/load. STAT-RNG-03 acceptance test catches it, but only AFTER the user manually deserializes and re-serializes — easy to miss in code review because every OTHER transient v3.1 field uses `skip`.
**Why it happens:** Muscle memory. The pattern in `state.rs` is overwhelmingly `#[serde(default, skip)]` for transient fields.
**How to avoid:** Plan 33-01 acceptance test `rand_seed_serde_round_trip` is mandatory and runs in CI on every commit. The field doc-comment SHOUTS the unique serde shape (see §"RNG Seed Persistence" above).
**Warning signs:** Code reviewer sees `#[serde(default, skip)] pub rand_seed: HpNum` → block the PR.

### Pitfall 4: Welford violated — naive variance formula in Stat 1 functions (P18)
**What goes wrong:** ΣBSTAT / ΣMMTUG computes variance from a raw register block via naive two-pass `(Σx² − (Σx)²/N) / (N−1)`. With N=1000 values near 1e6, catastrophic cancellation eats 7 significant digits of accuracy. The 10-digit HP-41 display gate still passes for small datasets — bug invisible in tests with N < 100.
**Why it happens:** Textbook formula. OM may even show it (1979 hardware used BCD-rounded intermediates).
**How to avoid:** ANY Stat 1 function that iterates over a raw data block (not via Σ+/Σ−) uses Welford's online algorithm (see `.planning/research/PITFALLS.md` for the snippet). Functions that consume the EXISTING Σ-register block (ΣLIN, ΣEXP, ΣBSTAT mean/sdev path) are SAFE — Σ-registers were accumulated one HpNum at a time with 10-digit rounding at each step.
**Warning signs:** Code reviewer sees `s_x2.checked_sub(&s_x.checked_sq()?.checked_div(&n)?)?` in `stat1/*.rs` for a non-Σ-register-fed computation.

### Pitfall 5: Free42 distinctive symbol copy-paste from `core_math2.cc` (P27 carry-forward)
**What goes wrong:** Acklam / AS 239 / AS 63 algorithms are also implemented in Free42's `core_math2.cc`. A developer cross-referencing for sanity may copy a distinctive identifier (`math_chi2_*`, `decNumber*` symbol, etc.) — CI catches it via `check-free42-contamination.sh` extended pattern, but the rerun costs a CI cycle.
**Why it happens:** The 12-symbol math1/-scoped pattern doesn't cover stats-domain identifiers.
**How to avoid:** Plan 33-00 extends the pattern by ≥ 6 tokens BEFORE the first `stat1/*.rs` lands (D-33.8 ordering). Plan 33-02 + onward never opens `core_math2.cc` for copy operations; only as oracle for `assert_relative_eq!` verification.
**Concrete identifier candidates** (Plan 33-00 reads `github.com/thomasokken/free42/blob/master/common/core_math2.cc` to finalize):
  - `math_normal_cdf`, `math_normal_pdf`, `math_normal_inv`
  - `math_chi2_cdf`, `math_chi2_pdf`, `math_chi2_inv`
  - `math_t_dist_cdf`, `math_t_dist_pdf`, `math_t_dist_inv`
  - `math_F_dist_*` (F-distribution functions — even though not in Stat 1, catch the symbols)
  - `math_gamma_lower`, `math_gamma_upper`, `math_beta_inc`
  - decNumber distinctive idioms: `decNumberPower`, `decNumberExp`, `decNumberLn` (extended from 12-symbol base if absent)
**Final list locked in Plan 33-00.** `[ASSUMED]` — concrete identifiers pending core_math2.cc read.

### Pitfall 6: Inverse distribution converges on wrong root (P19 carry-forward)
**What goes wrong:** ΣCHISQD CDF inverse (if ever implemented — currently out of scope; the SPEC.md only mandates ΣCHISQD forward CDF) with low ν has a fat tail. Newton's method with `x₀ = ν` (the mean) can step into negative territory or oscillate. ΣNORMD inverse for p < 0.001 has the same issue.
**Why it happens:** Pathological numerics; happens silently — function returns a value, just the wrong one.
**How to avoid:** Acklam's rational approximation provides a good initial guess for ΣNORMD inverse (no Newton needed for closed-form path). For chi-square inverse (NOT in scope this phase), Wilson-Hilferty transformation `x ≈ ν · (1 − 2/(9ν) + z · √(2/(9ν)))³` where z = Φ⁻¹(p) provides robust starting point. Cap Newton + bisection at 50 iterations total → `Err(HpError::ConvergenceFailed)`. Always validate against scipy.stats with at least 3 boundary cases (p = 1e-6, p = 0.5, p = 1 − 1e-6).
**Warning signs:** Single-tolerance test (no tail-edge oracle case).

### Pitfall 7: Numerical Recipes coefficient typos
**What goes wrong:** Acklam's a₁..a₆, b₁..b₅, c₁..c₆, d₁..d₄ table is widely copied. Some web mirrors have typos (e.g. dropped leading sign, off-by-one exponent in `e+01` vs `e+02`). Off-by-one in any coefficient → relative error spikes to 1e-3 or worse.
**Why it happens:** Hand-copying 21 long coefficients.
**How to avoid:** Plan 33-02 copies coefficients VERBATIM from `stackedboxes.org/2017/05/01/acklams-normal-quantile-function/` (Acklam's original NA-Net post is the ground truth, but the stackedboxes mirror is the most commonly cited HTML version). Cross-verify every coefficient by running 6 scipy oracle cases at p = {0.001, 0.025, 0.25, 0.5, 0.75, 0.975} — relative error < 1e-9 across the board confirms the table is intact. Any tuple failing > 1e-7 → suspect a typo in the coefficient table.
**Warning signs:** One oracle tuple fails 1e-7 while others pass at 1e-9.

### Pitfall 8: Modal-prompt new variant required in math1/modal.rs (D-33.3 freeze tension)
**What goes wrong:** Phase 33 ships ΣCHISQD, ΣPOLYP, and SEED modal opens — each needs a new `ModalProgram::Stat1*` variant. But `math1/modal.rs` is in the FROZEN set per D-33.3 (only `xrom.rs` is exception).
**Why it happens:** v3.0 froze `math1/modal.rs` without anticipating sibling modules needing the enum.
**How to avoid:** Either (a) extend the freeze exception to `math1/modal.rs` with a SINGLE additive variant `ModalProgram::Stat1(stat1::modal::Stat1Step)` (recommended — symmetric with how each Math 1 program has its own step enum), or (b) declare a new `Stat1ModalProgram` enum in `stat1/modal.rs` and add a SECOND `modal_program_stat1: Option<Stat1ModalProgram>` field to `CalcState` (forbidden — D-33.5 says no new transient fields beyond `rand_seed`). **Recommendation: option (a) with a planner-level D-33.3 amendment captured as D-33.3b in CONTEXT.md.**
**Warning signs:** Plan 33-03 or 33-08 task touches `math1/modal.rs` without explicit D-33.3 amendment.

### Pitfall 9: `STAT_1.ops` slice and `stat1_resolve()` match arm count drift
**What goes wrong:** Adding a new mnemonic to `STAT_1.ops` slice but forgetting to add the corresponding `"NAME" => Some(Op::...)` arm in `stat1_resolve()`. The existing CI test `math1_ops_mnemonics_resolve_consistently` (at `math1/xrom.rs:338`) catches MATH_1 drift — extend to STAT_1 in Plan 33-01.
**Why it happens:** Two places to keep in sync.
**How to avoid:** Plan 33-01 adds `stat1_ops_mnemonics_resolve_consistently` test mirroring `math1_ops_mnemonics_resolve_consistently`. Subsequent plans MUST maintain parity between slice and match arm.

### Pitfall 10: Tolerance test passes with looser bound than required (P14 / P17 carry-forward)
**What goes wrong:** Developer uses `assert!(diff.abs() < 1e-3)` instead of `assert_relative_eq!(actual, expected, max_relative = 1e-9)`. Test passes but doesn't enforce the 1e-9 / 1e-7 tolerance discipline.
**Why it happens:** Sloppy assertion style.
**How to avoid:** Use `approx::assert_relative_eq!` (already in dev-deps) for every Stat 1 numerical assertion. Phase 37 ships `lint_stat1_assertions.rs` extension that blocks `assert_eq!(decimal, decimal)` and naked `< 1e-N` patterns.

---

## Runtime State Inventory (rename/refactor concerns — none)

This is a greenfield phase (adding new functionality). No rename, refactor, or migration of existing data. **Section intentionally omitted** — no items to inventory.

---

## Code Examples (verified patterns from this codebase)

### Existing pattern: f64-bridge primitive (template for `norm_cdf_inv_f64`)
```rust
// Source: hp41-core/src/num.rs:184-211 (checked_asin)
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

### Existing pattern: display-mode-tied convergence (template for ΣNORMD inverse + ΣCHISQD CDF)
```rust
// Source: hp41-core/src/ops/math1/integ.rs:104-111
pub fn integ_threshold(mode: DisplayMode) -> f64 {
    let decimals = match mode {
        DisplayMode::Fix(n) | DisplayMode::Sci(n) | DisplayMode::Eng(n) => n as i32,
    };
    5.0_f64 * 10.0_f64.powi(-(decimals + 1))
}
```
**Stat 1 analog (Plan 33-03):** `stat1::distributions::quantile_threshold(mode)` using the SPEC.md Req. 34 formula `10^(-FIX_decimals − 1)` with fallback `1e-10` when not FIX. Differs from `integ_threshold` by factor of 5 (per SPEC.md choice — likely simpler for distribution quantiles).

### Existing pattern: cancellation check inside iterative loop
```rust
// Source: hp41-core/src/ops/math1/integ.rs (op_integ_run_loop)
use std::sync::atomic::Ordering;
// ... inside iteration loop:
if state.cancel_requested.load(Ordering::Relaxed) {
    return Err(HpError::Cancelled);
}
```
**Stat 1 reuse:** Every iteration inside `stat1::distributions::gamma_regularized_f64` continued-fraction loop AND ΣNORMD inverse Newton/bisection loop calls this.

### Existing pattern: Σ-register accumulation (delegate target for ΣLIN/EXP/LOGI/POW)
```rust
// Source: hp41-core/src/ops/stats.rs:22-53
pub fn op_sigma_plus(state: &mut CalcState) -> Result<(), HpError> {
    if state.regs.len() < 7 { return Err(HpError::InvalidOp); }
    let x = state.stack.x.clone();
    let y = state.stack.y.clone();
    let new_r1 = state.regs[1].checked_add(&x.checked_sq()?)?;
    // ... 5 more accumulations ...
    state.regs[1] = new_r1;  // etc.
    state.stack.lift_enabled = true;
    enter_number(state, new_r3);
    apply_lift_effect(state, LiftEffect::Enable);
    Ok(())
}
```

### Existing pattern: modal-prompt workflow open (template for ΣPOLYP DEGREE=?, SEED?, ν=?)
```rust
// Source: hp41-core/src/ops/math1/poly.rs (op_poly_workflow — open)
pub fn op_poly_workflow(state: &mut CalcState) -> Result<(), HpError> {
    state.modal_program = Some(ModalProgram::Poly(PolyInputStep::PromptForDegree));
    state.modal_prompt = Some("DEGREE=?".to_string());
    state.print_buffer.push("DEGREE=?".to_string());
    Ok(())
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `f64` for HP-41 register values | `rust_decimal` 1.42 BCD-style 10-digit rounding | Phase 1 (v1.0) | All Stat 1 register arithmetic stays exact through 28-digit `Decimal`; only the 3 distribution primitives use the f64-bridge pattern |
| `Op::XromCall(u16)` table dispatch | `Op::Sigma*` named variants (Op-strategy A) | Phase 28 (v3.0) ADR-001 | 4-way exhaustive-match invariant preserved across Phase 33 |
| Sole math/Σ block in `ops/stats.rs` | `ops/math1/` for module 1, `ops/stat1/` for module 2 | Phase 28 (v3.0) | Phase 33 lands the second module per SC-4 boundary |
| ALPHA prompt routed via ad-hoc print | `modal_program` + `modal_prompt` + `print_buffer` trifecta | Phase 28 (v3.0) | Phase 33 SEED?, ν=?, DEGREE=? all reuse — zero new transient fields |
| RNG via `getrandom` / `rand` / `ThreadRng` | Deterministic LCG with `rand_seed: HpNum` in `CalcState` | Phase 33 (v3.1) | NEW: hp41-core stays I/O-free; reproducible simulations across save/load |
| `statrs` for special functions | Hand-coded AS 239 / 63 / 241 (~140 LOC in `stat1/distributions.rs`) | Phase 33 (v3.1) | NEW: zero new runtime deps; OM-era algorithmic fidelity preserved |

**Deprecated/outdated:**
- **`f64::asin`/`f64::acos`/`f64::atan` in inverse-trig path** — NOT deprecated, still used in `num.rs:184–211`. Pattern reused for Stat 1 distribution primitives. State of the art FOR HP-41 emulation; would be poor practice in a general-purpose statistics library.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Final list of Free42 stats-domain identifiers for contamination-guard extension (P5, ~6 tokens) | §"Pitfall 5" | Pattern misses a real Free42 symbol → silent contamination; mitigation: Plan 33-00 reads core_math2.cc and finalizes list before commit |
| A2 | ANOVA / regression / contingency register layouts | §"Σ-Register Layout", §"ANOVA Family", §"Regression Family" | Silent wrong answers; mitigation: Plan 33-00 OM transcription is the contract |
| A3 | ΣPOLYP prompt string `DEGREE=?` (tentative per Math Pac I POLY precedent) | §"Regression Family" (ΣPOLYP) | OM may specify different string; mitigation: SPEC.md Req. 22 lock is "modal-prompt-driven degree-entry workflow exists", not literal text — Plan 33-08 substitutes OM string if differs |
| A4 | ΣCHISQD ν-entry prompt `ν=?` reusing existing modal machinery | §"Distribution Primitives", §"Pattern 3" | Same as A3; SPEC.md Req. 32 locks the workflow shape, not the literal prompt |
| A5 | RAND/SEED treated as v3.1 emulator extension (not OM-feature) per D-33.4 | §"RAND / SEED" | If OM lists RAND as ROM entry, divergences-catalog bucket changes (D-33.4a); ship is identical |
| A6 | ΣTSTAT pooled-variance per OM (Welch excluded) | §"Hypothesis Tests" (ΣTSTAT) | If OM contradicts (highly unlikely per NPS ZS-4/5), becomes documented divergence Phase 35 — NOT a SPEC.md change |
| A7 | `ModalProgram::Stat1(stat1::modal::Stat1Step)` extension to `math1/modal.rs` requires D-33.3b amendment | §"Pattern 3", §"Pitfall 8" | If amendment denied, alternative is a new CalcState field — forbidden by D-33.5; surface to user for resolution |
| A8 | ΣSPEAR oracle ρ_s value: SPEC.md says 0.7; manual computation says 0.8 for the given example | §"Hypothesis Tests" (ΣSPEAR) | Acceptance test would fail with implementation matching either value; mitigation: Plan 33-04 verifies the oracle against scipy `scipy.stats.spearmanr([1,2,3,4,5], [2,1,3,5,4]).statistic` and corrects SPEC.md or interpretation as needed |
| A9 | Acklam coefficient table accuracy (1.15e-9 error claim) sufficient for SPEC.md Req. 31 (1e-7) without Halley refinement | §"Algorithm 1: norm_cdf_inv_f64" | If oracle tests fail, add one Halley refinement step (+~10 LOC) |
| A10 | `HpNum::checked_int` helper does not exist and must be added in Plan 33-08 | §"RAND / SEED" — `checked_int` helper | If grep shows it exists, skip the helper; otherwise add ~5 LOC |

**Action for user (via planner):** A7 (ModalProgram freeze exception) needs explicit confirmation before Plan 33-03 starts.

---

## Open Questions

1. **`HpNum::checked_int` existence**
   - What we know: Functions `op_int`, `op_frc` exist on CalcState in `ops/math.rs`. No bare `HpNum::checked_int` discovered in `num.rs` (verified via `grep -n "checked_int" hp41-core/src/num.rs` → no matches).
   - What's unclear: Whether one of the existing math functions exposes a `HpNum`-level integer-truncation helper.
   - Recommendation: Plan 33-08 task #1 — grep, then add `HpNum::checked_int` if absent (likely; rust_decimal `trunc()` is one-liner).

2. **Σ Unicode vs ASCII alias convention for `STAT_1.ops`**
   - What we know: `MATH_1.ops` uses ASCII aliases for Unicode operators (`C*` for `C×`, `Z^N` for `Z↑N`, etc.) per the precedent at `math1/xrom.rs:57–79`.
   - What's unclear: Whether `ΣNORMD` should ALSO carry an ASCII alias like `SNORMD` for CLI ergonomic typing.
   - Recommendation: Plan 33-01 ships both — Unicode primary, ASCII alias secondary. Matches Math Pac I precedent exactly. Final list and aliases SHOULD be confirmed in Plan 33-08 OM read.

3. **`ModalProgram::Stat1` extension to `math1/modal.rs` (Pitfall 8 / A7)**
   - What we know: SPEC.md mandates ΣCHISQD ν-prompt, ΣPOLYP DEGREE-prompt, SEED prompt all use existing modal machinery. D-33.5 forbids new transient CalcState fields beyond `rand_seed`. D-33.3 froze `math1/modal.rs` (only `xrom.rs` exempted).
   - What's unclear: Whether the planner should treat the `ModalProgram::Stat1(Stat1Step)` addition to `math1/modal.rs` as an in-scope freeze exception (D-33.3b amendment) or escalate to user.
   - Recommendation: Plan 33-01 captures this as D-33.3b (`math1/modal.rs` minimal additive enum variant) — symmetric carve-out to the `xrom.rs` exception (both are central registries for cross-module entries). Plan 33-03 must NOT touch `modal.rs` until D-33.3b is recorded in CONTEXT.md.

4. **ΣSPEAR oracle value (SPEC.md vs manual computation)**
   - What we know: SPEC.md Req. 30 says `ranks_x=[1,2,3,4,5], ranks_y=[2,1,3,5,4]` → ρ_s = 0.7. Manual: d² = [(1−2)², (2−1)², (3−3)², (4−5)², (5−4)²] = [1,1,0,1,1] → Σd² = 4 → ρ_s = 1 − 6·4/(5·24) = 1 − 24/120 = 0.8.
   - What's unclear: Did SPEC.md author intend a different example or use a different rank-pair convention? scipy: `scipy.stats.spearmanr([1,2,3,4,5], [2,1,3,5,4]).statistic = 0.7999999999999999` (matches 0.8, not 0.7).
   - Recommendation: Plan 33-04 verifies via scipy. Either SPEC.md acceptance value updates to 0.8, or the example dataset changes. Discovered during research — flag for planner.

5. **ANCOVA "covariate" semantics — x is response, y is covariate (or reverse)?**
   - What we know: SPEC.md Req. 13 says "each observation is (x, y) pair where x is the response and y is the covariate." But OM 00041-90030 may use the opposite convention.
   - What's unclear: Standard statistics literature uses (x, y) where x is COVARIATE and y is RESPONSE; the SPEC.md uses opposite. May be transcription error.
   - Recommendation: Plan 33-06 OM read clarifies. Acceptance test uses oracle dataset where both conventions produce the same F-value (e.g., symmetric data).

6. **ΣCHISQD CDF — series vs continued-fraction split exactly at `x < a + 1`?**
   - What we know: Numerical Recipes 3e splits at exactly `x < a + 1` (P/Q swap). Some implementations use `x < a + 1.5` or similar.
   - What's unclear: Whether oracle bounds (1e-7) tolerate the strictly-equal-to-`a+1` boundary case.
   - Recommendation: Plan 33-02 ships strict Numerical Recipes `x < a + 1` split; oracle tuples at the boundary (e.g. `s=3, x=3.5` and `s=3, x=4.0`) verify continuity within tolerance.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` (Rust toolchain, MSRV 1.88) | All `hp41-core` work | ✓ (assumed — v3.0 shipped) | 1.88+ | none |
| `just` (task runner) | `just license-audit` for contamination guard | ✓ (v2.2 onward) | — | none |
| `python3` + `scipy` | Oracle value generation (one-shot, manual; values inlined in tests) | ✗ (not required at build/test time per D-33.6) | — | Oracle values copy-pasted from research; `scipy.special.gammainc`, `scipy.special.betainc`, `scipy.stats.norm.ppf` runs once locally for value derivation |
| `rust_decimal 1.42` | All numerical work | ✓ (workspace dep) | 1.42 `[VERIFIED: hp41-core/Cargo.toml]` | none |
| `approx 0.5.1` | Test assertions | ✓ (dev-dep) | 0.5.1 `[VERIFIED: hp41-core/Cargo.toml line 18]` | none |
| `grep`, `bash`, `find` (POSIX) | `check-free42-contamination.sh` | ✓ (assumed POSIX) | — | none |

**Missing dependencies with no fallback:** none.
**Missing dependencies with fallback:** scipy — oracle values pre-computed and inlined per D-33.6 (no runtime dependency).

---

## Validation Architecture

> Required per `.planning/config.json` (nyquist_validation enabled by absence).

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[cfg(test)] mod tests` + `cargo test` |
| Config file | `hp41-core/Cargo.toml` (workspace inherits MSRV 1.88) |
| Quick run command | `cargo test -p hp41-core --lib --quiet` |
| Full suite command | `cargo test -p hp41-core --quiet && cargo test --test xrom_shadowing -p hp41-core` |
| Coverage gate (Phase 37) | `just coverage` — `hp41-core ≥ 95.39% lines, ≥ 94.26% regions`; per-file `stat1/*.rs ≥ 90%` |

### Oracle Sources (NO Free42 code copying; oracle only)

| Oracle | What for | Confidence | Cross-check note |
|--------|----------|-----------|------------------|
| scipy.stats (`norm`, `chi2`, `t`, `f`) | Normal/chi-square/t-distribution CDF/PDF/inverse oracle values | HIGH | Industry standard; values lifted into inline `(input, scipy_expected, tolerance)` tuples per D-33.6 |
| scipy.special (`gammainc`, `betainc`) | Regularized incomplete gamma + beta oracle values | HIGH | Mathematically rigorous |
| `R stats` package | Sanity cross-check for hypothesis tests if scipy ambiguous | MEDIUM | Not committed; manual verification only |
| Free42 `core_math2.cc` | Sanity cross-check only — NEVER copied (CI-gated by `scripts/check-free42-contamination.sh` extended) | LOW (oracle confidence; HIGH contamination risk) | Used only for "does our value match free42's value within 1e-7" — if it does, both implementations probably correct; if not, scipy is the tiebreaker |
| NPS NPS55-84-003 example datasets (ZA-3, ZS-4/5) | OM-era cross-check for ANOVA + t-test specifics | MEDIUM | Free in DTIC archive; cited in plan comments |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| STAT-FW-01 | `STAT_1` const + bit-1 arm | unit | `cargo test -p hp41-core --lib xrom::tests::stat1` | ❌ Plan 33-01 |
| STAT-FW-02 | `default_xrom_modules() = 0b0000_0011` + migration | unit | `cargo test -p hp41-core --lib state::tests::xrom_modules_default state::tests::migrate` | ❌ Plan 33-01 |
| STAT-FW-03 | 4-way invariant items 1+2 | compile | `cargo check -p hp41-core` | ✓ existing; gates auto-extended |
| STAT-FW-04 | XEQ-by-name dispatch | integration | `cargo test --test stat1_xeq_by_name_dispatch -p hp41-core` | ❌ Plan 33-01 (one test per mnemonic) |
| STAT-UNI-01..04 | ΣBSTAT/BSTG/MMTUG/MMTGD + [C] correction | unit | `cargo test -p hp41-core --lib stat1::basic_stats::tests stat1::moments::tests` | ❌ Plan 33-05/06 |
| STAT-AOV-01..04 | ANOVA family | unit | `cargo test -p hp41-core --lib stat1::anova::tests` | ❌ Plan 33-06 |
| STAT-REG-01..09 | Regression family + Gauss elimination + `grep -n "ops::math1::matrix" hp41-core/src/ops/stat1/` empty | unit + script | `cargo test -p hp41-core --lib stat1::regression::tests && grep -nq "ops::math1::matrix" hp41-core/src/ops/stat1/ \|\| echo OK` | ❌ Plan 33-05/08 |
| STAT-HYP-01..07 | Hypothesis tests | unit | `cargo test -p hp41-core --lib stat1::hypothesis::tests stat1::nonparam::tests` | ❌ Plan 33-04/07 |
| STAT-DST-01..06 | ΣNORMD + ΣCHISQD + 3 primitives | unit | `cargo test -p hp41-core --lib stat1::distributions::tests stat1::normd::tests stat1::chisqd::tests` | ❌ Plan 33-02/03 |
| STAT-DST-07 | Cancellation + display-mode tolerance | integration | `cargo test --test stat1_cancellation -p hp41-core` | ❌ Plan 33-03 |
| STAT-RNG-01..04 | RAND/SEED + `rand_seed` serde | unit + integration | `cargo test -p hp41-core --lib state::tests::rand_seed_serde stat1::rand::tests && cargo test --test stat1_rand_determinism -p hp41-core` | ❌ Plan 33-01/08 |
| STAT-QUAL-09 (reassigned) | Contamination guard | script | `just license-audit` (or `bash scripts/check-free42-contamination.sh`) | ✓ existing script; Plan 33-00 extends |

### Sampling Rate (Nyquist Validation)

- **Per task commit:** `cargo test -p hp41-core --lib --quiet` (fast — runs in < 10 s on M1; gates compile error + unit failures immediately).
- **Per plan completion:** `cargo test -p hp41-core --quiet && cargo clippy -p hp41-core -- -D warnings && bash scripts/check-free42-contamination.sh` (full hp41-core suite + lint + guard).
- **Per wave merge (Plan 33-N done):** `just ci` (full workspace + format + license audit + numerical_accuracy.rs sweep).
- **Phase 33 gate (`/gsd:verify-work`):** All 25 SPEC.md acceptance criteria green + coverage held ≥ 95.39 % / 94.26 %.

### Wave 0 Gaps (Plan 33-00 + 33-01 deliverables)

- [ ] `hp41-core/src/ops/stat1/mod.rs` — NEW (Plan 33-00). OM `//!` transcription + `STAT1_MAX_REG` const + `pub use` re-export skeleton.
- [ ] `scripts/check-free42-contamination.sh` — EXTENDED (Plan 33-00). Pattern + scanned-dir list + ≥ 6 new tokens.
- [ ] `hp41-core/src/state.rs` — EXTENDED (Plan 33-01). `rand_seed` field + `migrate_after_load()` method + tests.
- [ ] `hp41-core/src/ops/math1/xrom.rs` — EXTENDED (Plan 33-01). `STAT_1` const + `stat1_resolve()` + bit-1 arm + tests.
- [ ] `hp41-core/src/ops/mod.rs` — EXTENDED (Plan 33-01). `pub mod stat1;` line.
- [ ] `hp41-core/tests/xrom_shadowing.rs` — EXTENDED (Plan 33-01). STAT_1 disjointness tests.
- [ ] `hp41-core/src/ops/stat1/distributions.rs` — NEW (Plan 33-02). Three primitives + ≥ 18 oracle tuples.
- [ ] `hp41-core/tests/stat1_xeq_by_name_dispatch.rs` — NEW (Plan 33-01 or earliest plan that ships first Op).
- [ ] `hp41-core/tests/stat1_cancellation.rs` — NEW (Plan 33-03).
- [ ] `hp41-core/tests/stat1_rand_determinism.rs` — NEW (Plan 33-08).
- [ ] `D-33.3b` amendment in CONTEXT.md — NEW (Plan 33-01 prerequisite; user confirmation). Authorizes minimal `ModalProgram::Stat1(stat1::modal::Stat1Step)` addition to `math1/modal.rs` (second freeze exception). See Open Question 3.

### Validation Coverage Matrix (test types per acceptance band)

| Tolerance band | Op category | Test type | Oracle |
|---------------|-------------|-----------|--------|
| 1e-9 closed-form | ΣNORMD CDF/PDF, ΣSPEAR, ΣBSTAT/BSTG, ΣLIN/EXP/LOGI/POW, ΣXSQEV/EFXSQ, ΣAOVONE | inline `assert_relative_eq!(actual, expected, max_relative = 1e-9)` | scipy.stats inline constants |
| 1e-7 iterative | ΣNORMD inverse, ΣCHISQD CDF, ΣTSTAT/ΣPTST, ΣMLRXY/MLRXYZ/POLYP, ΣAOVTWO/ΣANOCOV | inline `assert_relative_eq!(actual, expected, max_relative = 1e-7)` | scipy.stats inline constants + Newton-step boundary test |
| 1e-15 decimal-exact | RAND LCG step (no f64 conversion) | inline `assert_eq!(actual, expected)` (HpNum exact compare) | LCG formula recomputation |
| Compile-time | 4-way invariant items 1 + 2 | `cargo check -p hp41-core` | match exhaustiveness |
| Round-trip | `rand_seed` serde, v3.0 save migration | integration test | JSON blob comparison |
| Cancellation | iterative-quantile paths | `cancel_requested = true` before call → `Err(HpError::Cancelled)` | atomic flag |

---

## Project Constraints (from CLAUDE.md)

Mandatory directives extracted from `./CLAUDE.md` that constrain Phase 33 planning:

- **Workspace structure:** `hp41-core` MUST never depend on `hp41-cli` or `hp41-gui`. SC-4 (no core duplication in GUI) — Phase 33 is entirely in `hp41-core`, so this constraint is automatically satisfied. `[VERIFIED: ./CLAUDE.md §Workspace structure]`
- **`hp41-core/src/ops/math1/` is frozen since Plan 25-01.** D-33.3 grants exception for `xrom.rs` ONLY. This research surfaces a SECOND freeze-exception candidate (`math1/modal.rs`) — flagged as Open Question 3 / Pitfall 8 / A7. `[VERIFIED: ./CLAUDE.md §Core engine]`
- **4-way exhaustive-match invariant:** every `Op` variant in dispatch + execute_op + CLI op_display_name + GUI op_display_name. Phase 33 owns items 1 + 2 ONLY. `cargo check -p hp41-cli` and `-p hp41-gui` are EXPECTED to fail after Phase 33 commits — Phase 34 / 36 fix. `[VERIFIED: ./CLAUDE.md §4-way exhaustive-match]`
- **Resolver chain order:** `xrom_resolve` fires LAST after `builtin_card_op`. Already enforced in `program.rs:80,541`. Phase 33 plugs `stat1_resolve` as a sub-arm INSIDE `xrom_resolve` (NOT a separate fall-through). `[VERIFIED: ./CLAUDE.md §Resolver chain]`
- **Save-file backward compat:** every new `CalcState` field carries `#[serde(default)]`. RAND seed is the SOLE field this phase that carries `#[serde(default)]` WITHOUT `#[serde(skip)]`. `[VERIFIED: ./CLAUDE.md §Save-file backward compat]`
- **JSON canonical data flow:** `docs/hp41-stat1-functions.json` is a Phase 34 deliverable, NOT Phase 33. Right-panel filter excludes XROM-module functions; Stat 1 entries discoverable via `?` overlay only. `[VERIFIED: ./CLAUDE.md §JSON canonical data flow]`
- **No `println!` / `eprintln!` in `hp41-core`.** Stat 1 prompts (`SEED?`, `ν=?`, `DEGREE=?`) push into `state.print_buffer`. `[VERIFIED: ./CLAUDE.md §Core engine]`
- **`#![deny(clippy::unwrap_used)]`.** `.expect("reason")` or `?`-propagation only. f64-bridge primitives return `Result<f64, HpError>` instead of panicking. Test modules carry `#[allow(clippy::unwrap_used)]`. `[VERIFIED: ./CLAUDE.md §Core engine]`
- **No async.** All iteration synchronous. `[VERIFIED: ./CLAUDE.md §Core engine]`
- **MSRV 1.88.** Stat 1 Pac code must compile on Rust 1.88. `[VERIFIED: ./CLAUDE.md §Workspace structure]`
- **Free42 GPL-contamination guard:** Phase 33 Plan 33-00 EXTENDS the existing `scripts/check-free42-contamination.sh` (12 → ≥ 18 tokens; adds `stat1/` directory to scan). No code can reference distinctive Free42 / Intel BID / decNumber / GPL/AGPL identifiers. `[VERIFIED: ./CLAUDE.md §Free42 GPL-contamination guard]`
- **Git workflow:** every commit MUST use `/git-workflow:commit --with-skills` (English). Phase 33 plans inherit this constraint. `[VERIFIED: ./CLAUDE.md §Git Workflow]`
- **GSD workflow:** plans archived under `.planning/milestones/` after milestone ship. `[VERIFIED: ./CLAUDE.md §GSD Workflow]`
- **`just`** is the sole task runner; never call `cargo` directly in CI or docs. Plan 33-00 must extend `justfile` if a new recipe is added (e.g. `just stat1-check`). `[VERIFIED: ./CLAUDE.md §Tech Stack]`

---

## Sources

### Primary (HIGH confidence)

- **OM 00041-90030 — HP-41C Stat 1 Pac Owner's Manual (HP, June 1979)** — `literature.hpcalc.org` item 800. THE definitive source for Storage Registers, prompt strings, t-test variant, ν entry convention, RAND ROM-listing. **NOT YET FULLY READ** — Plan 33-00 + Plan 33-06 + Plan 33-08 deliverables. `[CITED]`
- **QRC 00041-90061 — HP-41C Stat Pac Quick Reference Card (HP, June 1979)** — `literature.hpcalc.org/community/hp41-pac-stat-qrc-en.pdf`. All 13 programs / mnemonics / SIZE / I/O columns. Read in upstream research. `[CITED]`
- **NPS55-84-003 (Zehna, Naval Postgraduate School, February 1984)** — DTIC AD-A140573. RNG formula, anti-feature confirmation, t-test pooled-variance convention, ΣCHISQD ν convention. `[CITED]`
- **`rust_decimal 1.42` docs.rs (`MathematicalOps` trait)** — `docs.rs/rust_decimal/latest/rust_decimal/trait.MathematicalOps.html`. Confirmed: `norm_cdf`, `norm_pdf`, `erf`, `checked_ln`, `checked_exp`, `sqrt`, `checked_powd`, `checked_sin/cos/tan`. NOT provided: `norm_cdf_inv`, `gamma_incomplete`, `beta_incomplete`, `asin/acos/atan`. `[VERIFIED: docs.rs 2026-05-21 snapshot]`
- **Codebase direct reads (this research session):**
  - `hp41-core/src/ops/math1/xrom.rs` (full file — XromModule struct, MATH_1 const, xrom_resolve at line 127, bit-1 stub at line 133–134, tests at line 270+)
  - `hp41-core/src/state.rs` (lines 1–280 — CalcState, xrom_modules field at line 163, default_xrom_modules() at line 228, serde-default pattern)
  - `hp41-core/src/num.rs` (lines 1–220 — HpNum, checked_asin/acos/atan f64-bridge pattern at lines 184–211, MathematicalOps usage)
  - `hp41-core/src/ops/stats.rs` (lines 1–80 — op_sigma_plus, op_sigma_minus, R01–R06 layout, SIZE floor guard)
  - `hp41-core/src/ops/math1/mod.rs` (full file — module hub, submit_modal / cancel_modal / submit_modal_with_label public surface)
  - `hp41-core/src/ops/math1/integ.rs` (lines 1–120 — integ_threshold at line 104, IntegState shape)
  - `hp41-core/src/ops/math1/poly.rs` (lines 1–60 — POLY modal opener precedent, coefficient register convention R00–R05)
  - `hp41-core/Cargo.toml` (full file — workspace deps, dev-deps, no `statrs`/`rand`/`getrandom`)
  - `hp41-cli/src/help_data.rs` (lines 1–120 — OnceLock pattern, HelpEntry schema, XromEntry shape, help_entries_math1 accessor)
  - `hp41-cli/tests/function_matrix_parity.rs` (lines 1–60 — ALL_OP_VARIANT_NAMES + help_entries_math1 chain)
  - `scripts/check-free42-contamination.sh` (full file — 12-symbol PATTERN, MATH1_DIR scope, exit codes)
  - `.planning/phases/33-hp41-core-.../33-CONTEXT.md` (full file — 9 D-33.* decisions + canonical refs + code context)
  - `.planning/phases/33-hp41-core-.../33-SPEC.md` (full file — 46 locked requirements)
  - `.planning/phases/33-hp41-core-.../33-DISCUSSION-LOG.md` (full file — 8 decision rounds + alternatives)
  - `.planning/REQUIREMENTS.md` (full file — 66 v3.1 requirements + traceability)
  - `.planning/research/SUMMARY.md` (full file — 9-step implementation order + pitfall summary)
- **Acklam (1999), "An algorithm for computing the inverse normal cumulative distribution function"** — original NA-Net post; public-domain. `[CITED: stackedboxes.org/2017/05/01/acklams-normal-quantile-function/]`
- **Wichura (1988), "Algorithm AS 241: The Percentage Points of the Normal Distribution"** — Applied Statistics 37(3), 477–484. Equivalent to Acklam. `[CITED: jstor.org/stable/2347330]`

### Secondary (MEDIUM confidence)

- **Numerical Recipes 3rd ed. §6.1 (gammln), §6.2 (gammp / gser / gcf), §6.4 (betai / betacf)** — algorithm reference for AS 239 and AS 63. `[CITED: numerical.recipes]`
- **Shea (1988), "Algorithm AS 239: Chi-squared and Incomplete Gamma Integral"** — Applied Statistics 37(3), 466–473. `[CITED: jstor.org/stable/2347328]`
- **Majumder & Bhattacharjee (1973), "Algorithm AS 63: The Incomplete Beta Integral"** — Applied Statistics 22(3), 409–411. `[CITED: jstor.org/stable/2346797]`
- **HP-41 RNG formula `FRC(9821·x + 0.211327)`** — three-source confirmation: NPS p. 21, HP-65 User's Library (Don Malm), HP-41C Standard Applications p. 24. `[CITED: hpcalc.org/hp48/docs/misc/rand.txt]`
- **calc.fjk.ch/db/hp41mod.php "Statistics Pac 1B" XROM #2** — confirms hardware XROM ID = 2. `[CITED]`

### Tertiary (LOW confidence — verify in plans)

- **Concrete Free42 stats-domain identifiers for guard extension (Pitfall 5 / A1)** — hypothetical candidates pending Plan 33-00 read of `github.com/thomasokken/free42/blob/master/common/core_math2.cc`. `[ASSUMED]`
- **ANOVA / ANCOVA / multiple-regression register layouts (P21 / A2)** — pending Plan 33-00 OM transcription. `[ASSUMED]`
- **ΣPOLYP `DEGREE=?` prompt string (A3)** — Math Pac I precedent only; OM may differ. `[ASSUMED]`
- **ΣCHISQD `ν=?` prompt convention (A4)** — Math Pac I POLY precedent extended; OM may differ. `[ASSUMED]`
- **ΣSPEAR oracle value 0.7 in SPEC.md Req. 30 (Open Q 4 / A8)** — appears to conflict with manual computation (0.8) and scipy reference. Plan 33-04 verifies. `[ASSUMED]`

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — codebase Cargo.toml verified, no new deps
- Architecture: HIGH — XROM framework, resolver chain, modal machinery, 4-way invariant all codebase-verified; Math Pac I template fully readable
- Distribution algorithms: HIGH — Acklam coefficients verbatim from primary source; Numerical Recipes for AS 239/63 well-established; oracle table cross-checked
- Pitfalls: MEDIUM-HIGH — P18/P20/P21/P24/P27 codebase-verifiable HIGH; P19 quantile convergence MEDIUM (depends on Plan 33-02 implementation); A1 contamination identifiers LOW until Plan 33-00 reads core_math2.cc; A2 register layouts LOW until Plan 33-00 reads OM; A8 ΣSPEAR oracle value flagged for Plan 33-04 verification
- ModalProgram freeze exception (Pitfall 8 / Open Q 3): MEDIUM — requires user-level D-33.3b amendment; surfaced for planner

**Research date:** 2026-05-22
**Valid until:** 2026-06-22 (30 days for stable HP-era + Rust ecosystem; refresh if `rust_decimal` minor bumps or core_math2.cc changes upstream)
