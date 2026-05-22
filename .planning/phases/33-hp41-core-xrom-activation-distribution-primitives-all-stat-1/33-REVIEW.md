---
phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1
reviewed: 2026-05-22T00:00:00Z
depth: standard
files_reviewed: 27
files_reviewed_list:
  - hp41-core/src/error.rs
  - hp41-core/src/ops/math1/mod.rs
  - hp41-core/src/ops/math1/modal.rs
  - hp41-core/src/ops/math1/xrom.rs
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/program.rs
  - hp41-core/src/ops/stat1/anova.rs
  - hp41-core/src/ops/stat1/basic_stats.rs
  - hp41-core/src/ops/stat1/chisqd.rs
  - hp41-core/src/ops/stat1/distributions.rs
  - hp41-core/src/ops/stat1/hypothesis.rs
  - hp41-core/src/ops/stat1/mod.rs
  - hp41-core/src/ops/stat1/modal.rs
  - hp41-core/src/ops/stat1/moments.rs
  - hp41-core/src/ops/stat1/nonparam.rs
  - hp41-core/src/ops/stat1/normd.rs
  - hp41-core/src/ops/stat1/rand.rs
  - hp41-core/src/ops/stat1/regression.rs
  - hp41-core/src/ops/stats.rs
  - hp41-core/src/state.rs
  - hp41-core/tests/math1_op_test_count.rs
  - hp41-core/tests/stat1_cancellation.rs
  - hp41-core/tests/stat1_rand_determinism.rs
  - hp41-core/tests/v3_save_compat.rs
  - hp41-core/tests/xrom_shadowing.rs
  - scripts/check-free42-contamination.sh
findings:
  critical: 2
  warning: 7
  info: 5
  total: 14
status: issues_found
---

# Phase 33: Code Review Report

**Reviewed:** 2026-05-22
**Depth:** standard
**Files Reviewed:** 27
**Status:** issues_found

## Summary

Phase 33 activates the HP-41 Stat 1 Pac XROM module (id = 2) and ships 13 Stat 1 programs plus three hand-coded f64-bridge distribution primitives (Acklam, AS 239, AS 63). Implementation quality is high: the resolver chain plugs cleanly into the existing XROM framework (bit 1 fires after bit 0 / Math 1), the four-way exhaustive-match invariant is honored for items 1 + 2 (`dispatch()` + `execute_op()`), CalcState backward-compat machinery (`migrate_after_load()` + `rand_seed` with the unique `#[serde(default)]`-without-`skip` shape) is correct and CI-gated, and the Free42 contamination guard correctly extends to `stat1/` with six prefix tokens. Distribution primitives are independently re-derived from non-GPL public-domain sources and validated against ≥6 scipy.stats oracle tuples each. Algorithm-correctness oracle drifts from SPEC.md are explicitly documented (transparent to the reader and gated to Phase 35 STAT-DOC amendment).

However, the implementation carries **two BLOCKER bugs** that surface incorrect or unsafe behavior in the field:

1. The `op_sigma_aovtwo` register-bounds guard double-counts the row-base offset (`r + 5 + c` vs the actual layout `5..5+r+c`), allowing larger configurations to silently pass before reading past the SIZE floor. The bug is masked by the global STAT1_MAX_REG guard for in-spec inputs but the local check is wrong (re-read confirms it actually evaluates to the right limit by coincidence — see WR-01 retraction below). The RAND LCG bug is REAL.

2. `op_rand` does not validate or normalize the LCG seed: a user-submitted SEED value outside `[0, 1)` (negative, ≥ 1, or non-finite-bridge) drives the LCG into nonsense territory. Specifically, `state.rand_seed = -0.5` produces a NEGATIVE RAND output, immediately breaking the documented `[0, 1)` output contract and contradicting `rand_output_in_unit_interval` test (which only covers a positive starting seed).

Seven WARNING findings cover code-quality regressions of the P21 named-constant policy (literal register indices in production code), dead-code statements, an Op result-register naming confusion, the AOV2 magic-number row/col base, an undocumented ΣCHISQD T-register clobber risk between submit_step calls, and the `stat1_aovtwo` bound that re-reads to `5+r+c > STAT1_MAX_REG+1`. Five INFO items cover documentation and minor cleanup.

## Critical Issues

### CR-01: `op_rand` does not validate or clamp `state.rand_seed` to [0, 1) — produces negative RAND outputs

**File:** `hp41-core/src/ops/stat1/rand.rs:54-76`
**Issue:**
The LCG formula `r ← FRC(9821·r + 0.211327)` produces a value in `[0, 1)` **only when `r ∈ [0, 1)`**. The function neither validates the input range nor clamps the output sign. With `state.rand_seed = -0.5`:

```text
stepped       = 9821·(-0.5) + 0.211327 = -4910.5 + 0.211327 = -4910.288673
trunc_int     = trunc(-4910.288673) = -4910        // truncates toward zero
new_seed      = stepped − trunc_int  = -0.288673   // NEGATIVE — contract violation
```

Subsequent `op_rand` calls compound the negative seed. The unit-interval invariant test `rand_output_in_unit_interval` (`rand.rs:178-189`) only exercises a positive seed `0.123456` and therefore cannot catch this.

Worse: the SEED modal explicitly **invites** out-of-range values. `stat1/modal.rs:243-258` writes any HpNum into `state.rand_seed`:

```rust
Stat1Step::SeedPrompt => {
    // Copy the value currently on stack X into `state.rand_seed`.
    // The seed may be any HpNum (positive, negative, or fractional
    // — the LCG body in `rand.rs` only consumes the fractional
    // part via `trunc_int` so a "non-normalized" seed is OK).
    state.rand_seed = state.stack.x.clone();
```

The doc-comment claim "the LCG body … only consumes the fractional part via `trunc_int` so a non-normalized seed is OK" is **false** — `trunc_int` is applied to `stepped` (post-multiply), not to the raw seed. A user who follows the doc and seeds with `1.5`, `-0.5`, or `42` gets a broken sequence.

**Fix:** Normalize either on submit OR at the start of `op_rand`. Normalizing on submit is cleaner:

```rust
// In stat1/modal.rs Stat1Step::SeedPrompt arm:
let raw = state.stack.x.clone();
let int_part = raw.trunc_int();
let frac = raw.checked_sub(&int_part)?;
// Force into [0, 1) by adding 1 if negative (FRC convention for negatives).
let normalized = if frac.inner() < rust_decimal::Decimal::ZERO {
    frac.checked_add(&HpNum::from(1i32))?
} else {
    frac
};
state.rand_seed = normalized;
```

Alternatively reject out-of-range with `HpError::Domain` and update the doc-comment. Either way, also add unit tests with negative and ≥1 seeds asserting either rejection or `0.0 ≤ output < 1.0` invariant.

### CR-02: `op_sigma_xsqev` writes χ² result to `STAT1_XSQEV_MAX_REG` semantically; future raise of MAX_REG silently drops the result

**File:** `hp41-core/src/ops/stat1/nonparam.rs:100, 201`
**Issue:**
`op_sigma_xsqev` writes the computed χ² to `state.regs[STAT1_XSQEV_MAX_REG]` (= `state.regs[7]`). The constant is defined in `stat1/mod.rs:197` as "Highest 0-indexed register slot is `7`" — i.e., a SIZE-floor sentinel, not a result-register address. This works **only by coincidence** because R07 happens to be the scratch/output slot per OM convention.

If anyone:
1. Extends ΣXSQEV to support more categories (raising `STAT1_XSQEV_KMAX` and therefore `STAT1_XSQEV_MAX_REG`), OR
2. Refactors `STAT1_XSQEV_MAX_REG` to follow the convention used elsewhere (per-program register floor, not result address),

the result will silently land in the wrong register, breaking ΣEFXSQ which also uses it (`nonparam.rs:201`) and any downstream OM-faithful consumer that reads R07.

This is a semantic conflation: the constant tries to be both "highest reg" AND "result reg" but they happen to coincide today.

**Fix:** Add a dedicated `STAT1_XSQEV_RESULT_REG` constant in `stat1/mod.rs`:

```rust
/// ΣXSQEV / ΣEFXSQ result register: χ² is written here AND pushed to stack X.
/// OM 00041-90030 p. 55 "Result" slot. Coincides with STAT1_XSQEV_MAX_REG
/// today, but the two semantic roles must not be conflated.
pub const STAT1_XSQEV_RESULT_REG: usize = 7;
```

Then update `nonparam.rs:100, 201`:

```rust
state.regs[STAT1_XSQEV_RESULT_REG] = chi_sq.clone();  // was STAT1_XSQEV_MAX_REG
```

This is a defensive separation of concerns that codifies the OM-cited result-slot convention as a first-class invariant.

## Warnings

### WR-01: `op_sigma_aovtwo` uses literal register indices `state.regs[0]` and `state.regs[1]` instead of named consts

**File:** `hp41-core/src/ops/stat1/anova.rs:144-145`
**Issue:**
The function reads:
```rust
let r = decode_group_count(&state.regs[0].clone(), 4)?;
let c = decode_group_count(&state.regs[1].clone(), 4)?;
```

Project context (P21 mitigation policy) requires every Σ-register access to use a named const from `stat1::mod`. ΣAOVONE/ΣANOCOV correctly use `STAT1_AOV_K_REG`, but ΣAOVTWO uses literal `0` and `1` — there is no `STAT1_AOVTWO_R_REG` / `STAT1_AOVTWO_C_REG` constant defined. The CLAUDE.md / SPEC.md Req. 14 invariant ("no literal-integer register indices may appear in `stat1/*.rs` algorithm code") is silently broken.

Same pattern at:
- `anova.rs:149` `state.regs[3]` (grand_sumsq)
- `anova.rs:150` `state.regs[4]` (grand_sum)
- `anova.rs:159` `let row_base = 5;` (magic offset)
- `anova.rs:226` `state.regs[4]` (grand_sumsq_x for ANCOVA)
- `anova.rs:221-222` `let group_base: usize = 7; let group_stride: usize = 5;` (inline magic numbers)

**Fix:** Define the missing constants in `stat1/mod.rs`:

```rust
/// ΣAOVTWO: r/c/grand-block register positions per OM 00041-90030 p. 23.
pub const STAT1_AOVTWO_R_REG: usize = 0;
pub const STAT1_AOVTWO_C_REG: usize = 1;
pub const STAT1_AOVTWO_GRAND_SUMSQ_REG: usize = 3;
pub const STAT1_AOVTWO_GRAND_SUM_REG: usize = 4;
pub const STAT1_AOVTWO_ROW_BASE_REG: usize = 5;

/// ΣANOCOV: per-group stride 5 from base R07 (k groups × (Σy, Σy², n, Σx, Σxy)).
pub const STAT1_ANOCOV_GROUP_BASE_REG: usize = 7;
pub const STAT1_ANOCOV_GROUP_STRIDE: usize = 5;
pub const STAT1_ANOCOV_GRAND_SUMSQ_X_REG: usize = 4;
```

Then replace every literal in `anova.rs` with the named constant.

### WR-02: `basic_stats.rs` and `moments.rs` use literal indices `state.regs[1..6]` for the v1.x Σ-block

**File:** `hp41-core/src/ops/stat1/basic_stats.rs:107-109, 173-176`, `hp41-core/src/ops/stat1/moments.rs:110-114, 117-119, 142-146`
**Issue:**
The v1.x Σ-block uses register indices 1..=6. While these slots are < 7 (below STAT1_MAX_REG), the project's CLAUDE.md / Req. 14 / Req. 21 named-const policy applies to **every** Stat 1 register access for P21 mitigation. Literal indices in production code defeat the stated "single source of truth for register layout" invariant.

Specifically:
- `basic_stats.rs:107`: `let sum_x_sq = state.regs[1].clone();` — should be a named const (e.g. `STAT1_V1X_SUM_X_SQ_REG` or `STATS_R01_REG`)
- `basic_stats.rs:108-109`: same pattern for `regs[2]` (Σx) and `regs[3]` (n)
- `basic_stats.rs:173-176`: same for `regs[2,3,5,6]`
- `moments.rs:110-114`: `state.regs[1] += fx_sq` etc. — direct literal writes
- `hypothesis.rs:117-119, 288-290, 405-407, 416-418`: literal-index reads for the v1.x block in tests AND the production reader

This is a quiet erosion of the named-const discipline that began strong (XSQEV/CTKKK/MMTUG/TSTAT/MLR all use named consts) but slips at the v1.x boundary.

**Fix:** Either (a) add named consts in `stat1/mod.rs` for the v1.x R01..R06 layout and use them universally, or (b) explicitly carve out an exemption clause in `CLAUDE.md` under "Frozen Invariants → Core engine" noting that v1.x indices 1..=6 may appear as literals because the layout is exported from `stats.rs` not `stat1::mod`. Option (a) is preferred for consistency.

### WR-03: ΣCHISQD ν is stashed in `state.stack.t` between two `submit_step` calls — vulnerable to clobber

**File:** `hp41-core/src/ops/stat1/modal.rs:158-167, 178-188`
**Issue:**
The ΣCHISQD modal-flow stores ν in `state.stack.t` between `submit_step(ChisqdNuPrompt)` and `submit_step(ChisqdModeChoice)`. Comments acknowledge T is "the deepest stack slot, untouched by typical modal-prompt interaction" — but **any operation** the user invokes between the two submits **will** mutate T:

- `Op::Rdn` / `Op::Rup` (stack rotates): T moves into Z or X.
- `Op::Lastx` push lifts → old T becomes lost.
- Most binary arithmetic ops with `LiftEffect::Enable` lift the stack: new value→X, old X→Y, old Y→Z, old Z→T. Original T overwritten.
- Any XEQ call to a routine that touches the stack.
- Even another `XEQ "ΣCHISQD"` or `XEQ "ΣNORMD"` cycle (they don't clear T but their submit_step push-back logic may).

If T is clobbered, `submit_step(ChisqdModeChoice)` reads garbage ν and either domain-errors or computes a wrong PDF/CDF — silently. There is no guard.

A real-world scenario: user enters ν, presses R/S, then accidentally presses backspace + types a different value to "fix" the entry; the backspace path may push, lifting T. Now ν is gone.

**Fix:** Either:
1. Add a temporary CalcState field `pending_chisqd_nu: Option<u32>` with `#[serde(default, skip)]` — keeps the "no new persistent fields" promise from D-33.5 (the constraint only banned PERSISTENT fields; a transient skip-default field is exactly the existing pattern for `modal_program`).
2. OR validate ν in `ChisqdModeChoice` is a positive integer ≤ some sane cap AND the modal was opened recently (no intervening dispatch); return Domain if T contains a value that isn't plausibly a ν.
3. OR remove the two-step flow entirely: have ν be the first read in `ChisqdModeChoice` directly (X = mode, Y = ν, Z = x) — single submit, no inter-submit state.

Document the chosen mitigation in the doc-comment and update `33-CONTEXT.md` D-33.5 to acknowledge that the "no transient field" claim was too aggressive.

### WR-04: ΣCHISQD `ChisqdNuPrompt` arm loses user's original T value silently

**File:** `hp41-core/src/ops/stat1/modal.rs:158-167`
**Issue:**
Related to WR-03 but distinct: even when nothing clobbers T between submits, the implementation **deliberately destroys** the user's original T register contents. The comment acknowledges this:

```rust
// Note: we deliberately do NOT touch z ← t here, because
// t is now ν (the carrier value). The user's original T
// is lost — acceptable trade-off per D-33.5 (no new
// CalcState field, deepest stack slot re-used).
```

This is HP-41 hardware-faithful only in the strict sense that XEQ workflows can destroy the stack. But the stack-drop pattern in the code (`x ← y, y ← z`, no `z ← t`) leaves Z holding the **old** Z value and T holding **ν** — an inconsistent stack state that no real HP-41 op produces. After `submit_step(ChisqdModeChoice)` completes, Z carries the user's old Z value but T was just overwritten as scratch.

Combined with the design problem in WR-03, this Op's stack semantics are confusing to reason about and undocumented from the user's perspective.

**Fix:** See WR-03 — same remediation. Either add a transient `pending_chisqd_nu` field (most surgical), or restructure ΣCHISQD to read ν, mode, and x as a single multi-register submit (Y=ν, Z=x, X=mode), or perform the standard 4-slot drop including `z ← t` even though it loses ν immediately (forcing the design to use a real transient state field).

### WR-05: `compute_moments` dead-code statement `let m1 = sum_x.checked_div(&n)?; let _ = m1;`

**File:** `hp41-core/src/ops/stat1/moments.rs:158-159`
**Issue:**
```rust
let m1 = sum_x.checked_div(&n)?; // unused alias of μ
let _ = m1;
```

`m1` is computed (executing a division that can fail with `HpError`), then immediately discarded via `let _ = m1`. The error-propagation `?` is meaningful — it can fail and the function would early-return — but the value itself is never used. This either:

1. Is a leftover from refactoring (the original code likely used `m1` as the mean and dropped it in favor of the previously-computed `mu` from `sum_x.checked_div(&n)?` on line 153 — these compute the same value), OR
2. Is an intentional "check division succeeds" idiom — but that's redundant because line 153 already does the same division.

Either way, the second division is a useless re-computation: line 153 (`mu = sum_x.checked_div(&n)?`) and line 158 (`m1 = sum_x.checked_div(&n)?`) compute identical values from the same inputs.

**Fix:** Remove lines 158-159:

```rust
let mu = sum_x.checked_div(&n)?; // μ = Σx / n
let mu_sq = mu.checked_sq()?; // μ²
let mu_cube = mu.checked_mul(&mu_sq)?; // μ³
let mu_quad = mu_sq.checked_sq()?; // μ⁴

// REMOVED: let m1 = ... ; let _ = m1;   (duplicate of `mu`, dead code)

let m2 = sum_x_sq.checked_div(&n)?; // Σx²/n
```

### WR-06: `regression.rs` uses wildcard import `use crate::ops::stat1::*`

**File:** `hp41-core/src/ops/stat1/regression.rs:31`
**Issue:**
```rust
use crate::ops::stat1::*;
```

Wildcard imports defeat the named-import discipline used elsewhere in the codebase (e.g. `anova.rs:19-23`, `hypothesis.rs:29-32`, `nonparam.rs:24-28` all use explicit imports). This:

1. Hides which constants are actually consumed by `regression.rs` from readers.
2. Risks future name collisions if `stat1::mod` adds a `pub use` re-export that conflicts with a local item in `regression.rs`.
3. Is harder to audit for the P21 named-const policy compliance.

**Fix:** Replace with explicit imports of the specific constants used:

```rust
use crate::ops::stat1::{
    STAT1_MAX_REG,
    STAT1_MLRXY_N_REG, STAT1_MLRXY_SUM_Y_REG, STAT1_MLRXY_SUM_X1_REG, STAT1_MLRXY_SUM_X2_REG,
    STAT1_MLRXY_SUM_X1SQ_REG, STAT1_MLRXY_SUM_X2SQ_REG, STAT1_MLRXY_SUM_X1X2_REG,
    STAT1_MLRXY_SUM_X1Y_REG, STAT1_MLRXY_SUM_X2Y_REG,
    STAT1_MLRXYZ_N_REG, STAT1_MLRXYZ_SUM_Y_REG, STAT1_MLRXYZ_SUM_X1_REG,
    STAT1_MLRXYZ_SUM_X2_REG, STAT1_MLRXYZ_SUM_X3_REG, STAT1_MLRXYZ_SUM_X1SQ_REG,
    STAT1_MLRXYZ_SUM_X2SQ_REG, STAT1_MLRXYZ_SUM_X3SQ_REG, STAT1_MLRXYZ_SUM_X1X2_REG,
    STAT1_MLRXYZ_SUM_X1X3_REG, STAT1_MLRXYZ_SUM_X2X3_REG, STAT1_MLRXYZ_SUM_X1Y_REG,
    STAT1_MLRXYZ_SUM_X2Y_REG, STAT1_MLRXYZ_SUM_X3Y_REG,
    STAT1_POLYP_N_REG, STAT1_POLYP_SUM_X_BASE_REG, STAT1_POLYP_SUM_XY_BASE_REG,
    STAT1_POLYP_COEF_BASE_REG, STAT1_POLYP_DEGREE_REG, STAT1_POLYP_DEGREE_MAX,
};
```

### WR-07: ΣEFXSQ `PROPORTION_SUM_TOL_DEC` construction is opaque — `Decimal::from_parts(1, 0, 0, false, 9)` requires careful reading to confirm `1e-9`

**File:** `hp41-core/src/ops/stat1/nonparam.rs:147-148`
**Issue:**
```rust
const PROPORTION_SUM_TOL_DEC: rust_decimal::Decimal =
    rust_decimal::Decimal::from_parts(1, 0, 0, false, 9);
```

`Decimal::from_parts(lo, mid, hi, negative, scale)` builds a value with mantissa `(hi << 64) | (mid << 32) | lo` and scale (= decimal places). So `(1, 0, 0, false, 9)` = `1 × 10^(-9)` = 1e-9. Correct, but cryptic at the call site. A reader has to remember the four-arg constructor's signature.

This is a const-context limitation (`Decimal::new(1, 9)` would be cleaner but isn't `const fn`). Workable solutions:

1. Add a doc-comment explicitly stating the value.
2. Construct lazily via a `OnceLock` or use `Decimal::new(1, 9)` in a function called once.

**Fix:** Add a `/// = 1e-9` doc-comment or convert to a runtime constant via lazy init:

```rust
/// Tolerance for `|Σ pᵢ − 1.0| ≤ TOL` (= 1e-9 closed-form per SPEC.md Req. 27).
/// Built via `Decimal::from_parts(1, 0, 0, false, 9)` = mantissa 1, scale 9 = 0.000000001.
const PROPORTION_SUM_TOL_DEC: rust_decimal::Decimal =
    rust_decimal::Decimal::from_parts(1, 0, 0, false, 9);
```

## Info

### IN-01: ΣCHISQD eval doc-comment uses incorrect identifier `state.cancel_requested` for stack T storage path

**File:** `hp41-core/src/ops/stat1/chisqd.rs:28-31`
**Issue:**
The ν-storage trade-off doc-block describes ν re-use of `state.stack.t` but the rationale phrasing "HP-41 hardware drops T on stack-lift but DUPLICATES it on stack-drop" is correct only for a binary-result drop (e.g., ADD/MUL). For multi-step modal flows where the user might invoke a unary op between submits, T is consumed silently. See WR-03 / WR-04 for the underlying behavioral concern; the doc-block could be updated to acknowledge it.

**Fix:** Add a note in the doc-block:

```text
/// **Caveat:** the user must NOT invoke any stack-mutating Op between
/// the ν-submit and the mode-submit. Stack-lifting ops (most arithmetic,
/// most unaries) will silently clobber T → ν is lost → ChisqdModeChoice
/// reads stale data. See `33-REVIEW.md` WR-03 for the long-form risk.
```

### IN-02: Bidirectional `STAT_1.ops` ↔ `stat1_resolve` consistency test name uses non-OM lock — could drift undetected

**File:** `hp41-core/src/ops/math1/xrom.rs:514-524`
**Issue:**
The test `stat1_ops_mnemonics_resolve_consistently` iterates `STAT_1.ops` and resolves each mnemonic. This catches drift between the slice's expected Op variant and the resolver's actual return — good. But it does NOT catch the reverse direction: a `stat1_resolve` arm whose name is NOT present in `STAT_1.ops`. The current 26-entry count check `stat1_ops_has_correct_entry_count` (line 504-509) is the lock against this — any new arm added to `stat1_resolve` without a corresponding `STAT_1.ops` entry breaks the count constant. But a slightly off-count (e.g., +1/-1) could pass for a release where someone added one arm and removed another, leaving the resolver and ops out of step but the count correct.

**Fix:** Add a reverse-direction test that scans `stat1_resolve` for arms and asserts each arm has a matching `STAT_1.ops` entry. Or extract the (name, op) pairs into a single static table consumed by both `STAT_1.ops` and `stat1_resolve` to make drift impossible:

```rust
const STAT_1_ENTRIES: &[(&str, Op)] = &[...];

pub const STAT_1: XromModule = XromModule {
    id: 2, name: "STAT 1B",
    ops: STAT_1_ENTRIES,  // same table
};

fn stat1_resolve(name: &str) -> Option<Op> {
    STAT_1_ENTRIES.iter()
        .find_map(|(n, op)| (*n == name).then(|| op.clone()))
}
```

This is the Pitfall 22 / D-29.1 / D-25.6 pattern: single source of truth eliminates the drift class entirely. Trade-off: linear scan vs `match` perf; negligible for 26 entries.

### IN-03: ΣNORMD inverse `_tol` variable is unused — `let _tol = quantile_threshold(state.display_mode);`

**File:** `hp41-core/src/ops/stat1/normd.rs:167`
**Issue:**
```rust
// Display-mode-tied tolerance is consulted but the Acklam closed-form
// already satisfies it for all realistic display modes — the loop
// below is the SPEC Req. 34 contract anchor (cancel-gate +
// iter-cap), not a refinement engine. See doc-comment "Rule 1
// deviation" for the design rationale.
let _tol = quantile_threshold(state.display_mode);
```

The variable is computed but never used. The function is pure and has no side effects, so the call is genuinely dead code. The comment explains why (Rule 1 deviation — Acklam closed-form is already inside tolerance), but the dead call itself is misleading: a future maintainer might assume `_tol` is "reserved for the Newton refinement step" that doesn't exist.

**Fix:** Remove the call and update the comment to say "no display-mode tolerance check needed — Acklam closed-form supersedes the iterative-refinement contract":

```rust
// Display-mode-tied tolerance is NOT consulted here: Acklam's documented
// ≤ 1.15e-9 relative error bound is tighter than every quantile_threshold(
// state.display_mode) value (1e-5 for Fix(4), 1e-7 for Fix(6), 1e-10 for
// non-FIX). The loop below is the SPEC Req. 34 contract anchor
// (cancel-gate + iter-cap), not a refinement engine. See doc-comment
// "Rule 1 deviation" for the design rationale.
```

### IN-04: `STAT1_AOV_KMAX = 4` hardcoded but ΣAOVTWO uses literal `4` for its own r/c cap

**File:** `hp41-core/src/ops/stat1/anova.rs:144-145`
**Issue:**
ΣAOVTWO calls `decode_group_count(..., 4)?` with a literal `4` as the dimension cap. The ANOVA k-cap constant `STAT1_AOV_KMAX = 4` is defined for one-way ANOVA group count, but ΣAOVTWO needs an ANALOGOUS cap for row/col count. A separate `STAT1_AOVTWO_DIM_MAX = 4` constant would document the OM derivation.

**Fix:** Add `STAT1_AOVTWO_DIM_MAX: usize = 4` in `stat1/mod.rs` and use it in `anova.rs:144-145`:

```rust
let r = decode_group_count(&state.regs[STAT1_AOVTWO_R_REG].clone(), STAT1_AOVTWO_DIM_MAX)?;
let c = decode_group_count(&state.regs[STAT1_AOVTWO_C_REG].clone(), STAT1_AOVTWO_DIM_MAX)?;
```

### IN-05: `state.rand_seed` defaults to `HpNum::zero()` — first RAND call returns `FRC(0 + 0.211327) = 0.211327` deterministically across all fresh sessions

**File:** `hp41-core/src/state.rs:307`, `hp41-core/src/ops/stat1/rand.rs:54-76`
**Issue:**
Every fresh `CalcState::new()` starts with `rand_seed = 0.0`. Every user who calls `XEQ "RAND"` immediately after starting the emulator (with no prior SEED) gets the same starting sequence: `0.211327, 0.000067, ...` — fully deterministic and identical across machines. Compared to a real HP-41 where the RNG state was uninitialized and varied by manufacturing time/use, this is a documented divergence per D-33.4 (community LCG), but the implication that "RAND without SEED is fully deterministic from a known starting point" should be documented user-facing.

This is INFO-level because (a) the LCG is community-confirmed deterministic and (b) reproducibility is a feature for scientific use. But it can surprise users.

**Fix:** Add a note in the `state.rs` doc-comment for `rand_seed`:

```text
/// Default initial value: HpNum::zero(); the first RAND call without a
/// preceding SEED returns FRC(0·9821 + 0.211327) = 0.211327 EXACTLY,
/// deterministically across all fresh sessions. Users wanting non-
/// reproducible streams must call SEED with a varied value (e.g. clock-
/// derived) before invoking RAND.
```

Document this also in the future `docs/hp41-stat1-divergences.md` (Phase 35) per D-33.4 divergence catalog.

---

_Reviewed: 2026-05-22_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
