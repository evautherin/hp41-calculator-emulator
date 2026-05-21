# Pitfalls: HP-41 Stat 1 Pac Emulation (v3.1)

**Milestone:** v3.1 Stat 1 Pac Emulation (second XROM module after Math Pac I)
**Researched:** 2026-05-21
**Scope:** Pitfalls SPECIFIC to adding Stat 1 Pac behavioral emulation on top of the
shipped v3.0 codebase. Pitfalls 1–17 from v3.0 Math Pac I are mitigated and
already gate-checked in CI; they are NOT repeated here. Where a v3.0 pitfall has
a Stat-1-specific extension, that extension is called out explicitly with a
cross-reference.

**Already mitigated (do not re-document):**
- P1: xrom_resolve fires LAST — `tests/xrom_shadowing.rs` CI gate already checks every
  `MATH_1.ops` entry; v3.1 adds an identical gate for every `STAT_1.ops` entry.
- P11: Long-running op Mutex release + `request_cancel` — infrastructure already
  ships in v3.0. v3.1 iterative ops (distribution quantile solvers, curve-fit
  iterations) must wire into the same `cancel_requested` channel; see P25 below.
- P14: Cross-platform f64 drift, relative tolerance 1e-7 baseline — already
  documented, `lint_math1_assertions.rs` CI gate already enforces the discipline.
  See P23 for whether stats ops need a TIGHTER baseline than 1e-7.
- P16: Per-Op test count ≥ 5 — `math1_op_test_count.rs` CI gate already enforces.
  v3.1 extends the gate to cover stat1 Op variants.
- P17: `assert_eq!` on iterated HpNum — `lint_math1_assertions.rs` already blocks this.
- Free42 GPL contamination: `scripts/check-free42-contamination.sh` 12-symbol CI gate
  already runs. v3.1 may add stats-domain identifiers to the pattern list; see P31.

**Confidence (overall):** MEDIUM-HIGH.
Variance algorithm behavior (Pitfall 18) is verifiable from first principles and
from the existing `stats.rs` code. Distribution quantile convergence (Pitfall 19)
is documented in the Stat 1 Pac Owner's Manual (HP 00041-15001) and cross-checked
against the pattern established in Math Pac I for SOLVE. RNG pitfalls (Pitfall 20)
are deterministic by design — the constraints are architectural, not empirical.
Sigma-register collision (Pitfall 21) is a direct consequence of the v2.2 stats
layout in `ops/stats.rs` which is in the codebase and readable. Alpha-prompt
shadowing (Pitfall 22) and documentation hygiene (Pitfall 27) are MEDIUM-confidence
process pitfalls. Distribution numerical precision (Pitfall 23) is LOW-MEDIUM:
the exact tolerance floor depends on which distributions Stat 1 includes and how
iterative they are — to be confirmed against the Owner's Manual in Phase 33.

---

## Summary

Eight pitfall categories dominate the v3.1 risk surface. All are NEW relative to
v3.0 — none duplicates the Math Pac I pitfall list:

1. **Variance catastrophic cancellation** (P18) — naive two-pass formula accumulates
   all sums THEN subtracts, triggering catastrophic cancellation on large-N or
   nearly-equal datasets. The existing v2.2 `op_sdev` in `stats.rs` uses this
   formula. Stat 1 Pac extends statistics beyond the six Σ-registers and may add
   functions that compute variance internally — all must use the numerically stable
   formula. This is a CRITICAL silent-wrong-answer risk.

2. **Distribution quantile Newton iteration non-convergence** (P19) — inverse CDF
   functions (e.g. inverse normal, inverse t, inverse chi-squared) use Newton's
   method or bisection on the CDF. Poor initial-guess choices or pathological inputs
   can fail to converge or bracket. The convergence termination convention must match
   the Stat 1 OM spec, not a generic tolerance.

3. **RNG state serialization and SystemRandom contamination** (P20) — if `RAND` is
   implemented, the PRNG seed must be a new `CalcState` field with `#[serde(default)]`
   (not `serde(skip)`) so it survives save/load. Equally, the PRNG algorithm must be
   a deterministic pure-Rust implementation — no `getrandom` / `rand::ThreadRng` /
   `SystemRandom` in `hp41-core` (the crate is I/O-free by invariant).

4. **Sigma-register collision and layout conflicts** (P21) — v2.2's stats block uses
   R01–R06 for Σx², Σx, n, Σy², Σy, Σxy. Stat 1 Pac may use additional Σ-registers
   or a different layout for extended statistics (higher moments, histogram bins).
   Misalignment between v2.2's fixed layout and Stat 1's expected layout produces
   silently wrong results. Also: `SIZE` conflicts — if the user runs `SIZE` to shrink
   the register bank below R06, the v2.2 fail-closed guard in `stats.rs` fires.
   Stat 1's new register requirements extend this minimum SIZE floor.

5. **ALPHA-prompt name shadowing** (P22) — if a Stat 1 Pac workflow prompt uses a
   name that matches a built-in mnemonic (e.g. "MEAN?" or "CORR?"), the XEQ-by-name
   modal may mis-route. Modal prompt names and built-in names occupy different
   namespaces, but the CLI `xeq_by_name_local_resolve` string match is prefix-based;
   a Stat 1 function named exactly "MEAN" would shadow the v2.2 built-in `MEAN`.

6. **Distribution CDF/PDF numerical tolerance baseline** (P23) — distribution
   computations (normal CDF, t-distribution CDF, chi-squared, F, binomial, Poisson)
   are NOT iterative in the same sense as INTG/SOLVE, but they DO involve special
   functions (erf, incomplete gamma, incomplete beta) that accumulate rounding. The
   1e-7 Math Pac I floor may be too loose for distribution results that users expect
   to agree with statistical tables to 4–6 decimal places. Tightening tolerance for
   non-iterative stats ops is possible with `rust_decimal`; the baseline must be
   established before writing accuracy tests.

7. **`xrom_modules` bit allocation for Stat 1** (P24) — v3.0 allocates bit 0 for
   Math 1 and reserves bit 1 for Stat 1 (the comment in `xrom.rs` line 134 says
   `// if modules & 0b0000_0010 != 0 { stat1_resolve(name) }`). The `default_xrom_modules`
   function currently returns `0b0000_0001`. When Stat 1 is added, this default must
   be updated to `0b0000_0011` to pre-load BOTH modules. V3.0 save files carry
   `xrom_modules: 1`; they must deserialize correctly when the new default is 3 — the
   `#[serde(default = "default_xrom_modules")]` annotation means old save files that
   DO serialize the field will load their stored value (1), not the new default (3).
   Old save files that DON'T serialize the field (pre-v3.0) will get 3. This is a
   BEHAVIORAL CHANGE for pre-v3.0 users: Stat 1 becomes enabled on first load.

8. **Cancellation channel wiring for iterative Stat 1 ops** (P25) — quantile solvers
   and curve-fitting iterations must periodically check `state.cancel_requested`
   and release the Mutex, exactly as Math Pac I's INTG/SOLVE/DIFEQ does. The
   infrastructure is already shipped; the risk is forgetting to wire new Stat 1
   iterative ops into the channel, causing GUI freezes on edge-case inputs.

Two supporting pitfalls concern documentation and licensing:
- P26: Stat 1 divergence catalog file naming and phase addressing (analogue of D-29.1)
- P27: Free42 stats-domain identifier additions to the contamination guard

---

## Critical Pitfalls

These mistakes cause rewrites, silent wrong answers, or backward-compat breaks.

---

### Pitfall 18: Variance Catastrophic Cancellation — Naive Formula in Extended Stats

**What goes wrong:**
The existing v2.2 `op_sdev` in `hp41-core/src/ops/stats.rs` computes:

```rust
// σx = sqrt((n·Σx² − (Σx)²) / (n·(n−1)))
let denom_x = n.checked_mul(sum_x2)?.checked_sub(&sum_x.checked_sq()?)?;
```

This is the textbook one-pass formula using pre-accumulated Σx² and Σx registers.
With `rust_decimal`'s 28-digit internal precision, this is generally fine for the
six-register built-in stats (the Σ-registers store exact decimal values, not
floating-point approximations). However, when Stat 1 Pac extends statistics to higher
moments or provides summary stats functions that compute variance INLINE from X/Y
stack values or from a register block — rather than from pre-accumulated Σ-registers —
the naive two-pass formula `(Σx² − (Σx)²/N)/(N−1)` is dangerous:

- With N=1000 values all near 1,000,000, `Σx²` ≈ 10¹⁵ and `(Σx)²/N` ≈ 10¹⁵. Their
  difference is at most 10⁸, but both operands use all 28 significant digits. The
  subtraction cancels ~7 leading digits → result has only ~21 significant digits
  before `rust_decimal`'s 28-digit representation helps you. That is enough in most
  cases, but for HP-41's 10-digit display target the margin is thin.
- If Stat 1 adds a "population variance" function that iterates over a register block
  using `STO IND` / `RCL IND` patterns (common in HP-41 PAC programs), intermediate
  accumulations may not have the benefit of 10-digit HP BCD rounding at each step.

The real pitfall is this: `op_sdev` in v2.2 is CORRECT because it uses pre-accumulated
Σ-registers (each step adds one `HpNum`-exact value). Any NEW Stat 1 function that
computes variance from a raw data block (not via Σ+/Σ− accumulation) must use
Welford's online algorithm or the compensated two-pass form — NOT the naive formula.

**Why it happens:**
The naive formula is what statistics textbooks and the HP Owner's Manual naturally
express. The behavioral emulation mandate ("reproduce what the OM says") creates
tension: the OM formula may be the naive one, but the OM is describing behavior on
10-digit BCD hardware where intermediate results are rounded at each step — behavior
that `rust_decimal` does NOT replicate for inline computations.

**How to avoid:**
- Check the Stat 1 Pac OM for each function that computes variance or standard
  deviation. If the function uses the Σ-registers (R01–R06) as intermediate storage,
  the existing accumulated-sum path is safe. If it iterates over a raw register block,
  use Welford's online algorithm:

  ```rust
  // Welford's: O(n) single-pass, numerically stable
  let mut mean = HpNum::zero();
  let mut m2   = HpNum::zero();
  for k in 1..=count {
      let x = state.regs[start + k].clone();
      let delta = x.checked_sub(&mean)?;
      mean = mean.checked_add(&delta.checked_div(&HpNum::from(k as i32))?)?;
      let delta2 = x.checked_sub(&mean)?;
      m2 = m2.checked_add(&delta.checked_mul(&delta2)?)?;
  }
  let variance = m2.checked_div(&HpNum::from(count - 1))?;
  ```

- Document in `docs/hp41-stat1-divergences.md` if the emulator's Welford output
  differs from the OM's example by more than 1 ULP at 10-digit precision. On HP BCD
  hardware, the OM example was computed with 10-digit intermediate rounding; Welford
  without intermediate rounding may give a slightly different last digit.
- The EXISTING `op_sdev` in `stats.rs` does NOT need to change — it is
  Σ-register-based and correct. Only new functions that work on raw register blocks
  are at risk.

**Warning signs:**
- A Stat 1 function for "variance of a data set in R20..R29" produces a result that
  differs from the OM example by more than 1 ULP for well-conditioned data (data
  values spread over a wide range). This is NOT catastrophic cancellation — it is just
  normal floating-point divergence.
- A Stat 1 function produces a NEGATIVE variance (before sqrt) when all values are
  nearly identical. This IS catastrophic cancellation in the two-pass formula.
  `checked_sqrt()` will catch it (returns `HpError::Domain`) but the error is not
  a meaningful calculator result.

**Phase to address:** Phase 33 (Stat 1 core ops). Any Stat 1 function that operates
on a raw register block must use Welford's algorithm. The Σ-register-based ops (MEAN,
SDEV, L.R. extensions) remain on the existing accumulated path.

**CI gate:** Add a `tests/stat1_variance_stability.rs` test with N=100 values all
near 10^6 that differ only in the last digit. Assert that variance ≠ 0 and relative
error vs. known analytical value < 1e-7. This catches the negative-variance trap.

---

### Pitfall 19: Distribution Quantile Inverse — Newton Iteration Non-Convergence

**What goes wrong:**
Stat 1 Pac provides inverse distribution functions — given a probability p, return
the quantile x such that CDF(x) = p. The inverse normal (e.g. "NORMI" or "INVNORM"),
inverse t, inverse chi-squared, and inverse F are all computed by iterative root-
finding on the CDF function.

Newton's method applied to `CDF(x) = p` is:
```
x_{n+1} = x_n − (CDF(x_n) − p) / PDF(x_n)
```

Three failure modes not covered by the existing SOLVE/secant infrastructure:

1. **Initial-guess failure near the tails.** For p near 0 or 1 (e.g. p = 0.9999),
   a poor starting guess produces an iteration that immediately jumps to x → −∞ or
   +∞ because `CDF(x_0) − p` is nearly zero and `PDF(x_0)` is also nearly zero near
   the tails. The emulator must use the Rational Approximation (Beasley-Springer-Moro
   or similar) for the initial guess, not x_0 = 0.

2. **Non-monotone numerics near the distribution mean.** For the t-distribution with
   few degrees of freedom, the PDF has heavy tails. A Newton step near the peak of
   the PDF can overshoot by an amount large enough to flip the bracket. Without
   bisection fallback, the iteration diverges.

3. **Termination criterion mismatch.** The Stat 1 OM specifies a convergence
   criterion analogous to the INTG display-mode-tied threshold. If the emulator
   terminates on `|x_{n+1} − x_n| < 1e-10` (fixed tolerance) instead of the
   OM-specified criterion, results will agree for p in [0.01, 0.99] but diverge
   for extreme quantiles where the OM criterion allows earlier termination.

**Why it happens:**
Distribution quantile inversion is a well-studied numerical problem and most
references recommend Newton + bisection hybrid. The HP-41 implementation is
constrained by available math: it runs on user-code, uses the calculator's 10-digit
BCD arithmetic, and is bounded by iteration count. The temptation is to use a modern
more-precise algorithm — but the goal is behavioral emulation of the OM-specified
behavior, not a numerically superior answer.

**How to avoid:**
- Read the Stat 1 Pac Owner's Manual convergence specification before writing any
  quantile inversion function. If the OM specifies "convergence to display precision"
  (as Math Pac I INTG does), tie the threshold to `state.display_mode` exactly as
  `integ_threshold(mode)` does:

  ```rust
  fn quantile_threshold(mode: DisplayMode) -> HpNum {
      // matches Math Pac I D-30-03 / ADR-004 convention:
      // threshold = 5 × 10^(-(decimals + 1))
      let d = match mode { DisplayMode::Fix(d) | DisplayMode::Sci(d) | DisplayMode::Eng(d) => d };
      HpNum::from(5) * HpNum::from(10i64).checked_pow(-(d as i32 + 1))
  }
  ```

- Use bisection as primary fallback when the Newton step would leave the valid domain
  (e.g. x < 0 for chi-squared, |p − CDF| increasing after the step). Do not allow
  more than 50 Newton + 50 bisection steps before returning `DATA ERROR`.
- For initial guess: use a simple rational approximation for the normal quantile
  (the 3-coefficient Abramowitz & Stegun formula is public domain and gives ≤ 0.45%
  relative error for all p in (0,1), which is an excellent Newton starting point).
  Document the formula with the A&S reference (section and page number) in the
  doc-comment — per the citation-discipline pattern from Pitfall 18 in PITFALLS.md v3.0.

**Warning signs:**
- `NORMI(0.9999)` returns DATA ERROR when the OM example shows a valid result.
  Root cause: initial guess was too far from the tail quantile.
- `TINV(0.5, 1)` (median of t with 1 degree of freedom = 0) returns a non-zero
  value. Root cause: Newton overshot symmetrically around 0 due to flat PDF at 0.
- Result for p=0.95 differs from a reference table in the 4th decimal place but
  p=0.5 is exact. Root cause: convergence threshold is too loose for extreme quantiles.

**Phase to address:** Phase 33 (Stat 1 core ops). Establish the convergence
specification from the OM BEFORE implementing any quantile function. Document in
`docs/hp41-stat1-divergences.md` if the emulator's convergence behavior differs from
the OM's described behavior (e.g. if the OM is silent on tail behavior).

**CI gate:** Extend `tests/numerical_accuracy.rs` with ≥ 10 distribution quantile
cases covering p in {0.001, 0.01, 0.1, 0.5, 0.9, 0.99, 0.999} for the normal
distribution, using relative tolerance 1e-6 (tighter than Math Pac I 1e-7 because
quantile tables expect 6 significant digits). Add a specific "tail convergence"
test for p = 0.9999 and p = 0.0001.

---

### Pitfall 20: RNG State Serialization and SystemRandom Contamination

**What goes wrong:**
If Stat 1 Pac includes a `RAND` (random number generator) function, two separate
pitfalls merge into one:

**A. RNG state must survive save/load (serde non-skip).**
The PRNG seed/state is NOT transient — it is part of the reproducible calculator
state. If a user saves mid-simulation, reloads, and continues, the next `RAND` call
must produce the same number it would have produced without the save/load cycle.
This means the RNG state field on `CalcState` must use `#[serde(default)]` but
NOT `#[serde(skip)]`. The pattern for ALL other transient fields in `CalcState` is
`#[serde(default, skip)]`; the RNG field is the EXCEPTION. Forgetting this and
using `skip` means the RNG silently re-seeds from a default on every load — users
running reproducible simulations get different results after saving.

Concrete implementation: add a new `CalcState` field:
```rust
/// PRNG state for RAND (Stat 1 Pac). LCG 32-bit: seed 0 = uninitialized default.
/// Persistent: survives save/load. NOT serde(skip). D-33-?? to be assigned.
#[serde(default)]
pub rand_seed: u32,
```

The field name and type must be locked in Phase 33 before the first Stat 1 commit
that adds `RAND` — late addition would silently change the default for loaded v3.0
save files.

**B. hp41-core must remain SystemRandom-free.**
The `hp41-core` invariant (`CLAUDE.md` "No async, no panics") extends to I/O:
`hp41-core` must NOT call `getrandom::getrandom()`, `rand::thread_rng()`, or any
syscall to `/dev/urandom` or Windows `CryptGenRandom`. These would violate the
I/O-free / no-external-calls constraint AND make RAND non-deterministic (same seed
→ same sequence is required for reproducibility). Use a simple deterministic LCG or
Xorshift32 seeded from `rand_seed`. The HP-41 hardware used a linear congruential
generator; using the same LCG constants produces hardware-faithful random sequences.

**Why it happens:**
The `#[serde(default, skip)]` pattern is so pervasive in `CalcState` (8 fields use it)
that a developer adding RNG state reaches for the same annotation by muscle memory.
The distinction between "transient state" (skip = correct) and "reproducible persistent
state" (default without skip = correct) is non-obvious.

The SystemRandom trap: Rust's `rand` crate is convenient and its default RNG is
excellent — but the crate pulls in `getrandom` as a dependency, which adds platform-
specific syscall code and breaks the I/O-free property of `hp41-core`. Even a dev-only
use of `rand::random::<f64>()` in a test inside `hp41-core` would add the dependency.

**How to avoid:**
- Before Phase 33: add a `// RAND state rule: #[serde(default)] NOT skip` comment in
  `state.rs` near the Phase 28 block, so future developers see the exception before
  adding fields.
- Implement the PRNG as a standalone function in `hp41-core/src/ops/stats1/rand.rs`
  with no external crate dependencies. Use the HP-41 hardware LCG constants if
  documented in the Stat 1 OM; otherwise use Xorshift32 (4 lines, no dependencies,
  period 2³²−1, deterministic).
- Add `rand` and `getrandom` to the Free42-contamination script's BLOCKED imports:

  ```bash
  # In scripts/check-free42-contamination.sh, add:
  grep -rn "extern crate rand\|use rand::\|getrandom" hp41-core/src/ && exit 1
  ```

- Add a `tests/stat1_rand_determinism.rs` test: seed PRNG to 42, call RAND 100
  times, serialize CalcState, deserialize into a fresh state, call RAND 1 more time,
  assert it equals the 101st expected value from a pre-computed reference sequence.

**Warning signs:**
- `Cargo.toml` for `hp41-core` gains a `rand` or `getrandom` dependency in any
  non-dev-dependencies block.
- Two identical save files loaded into two program runs produce DIFFERENT sequences
  after the first RAND call (would only happen if SystemRandom was used instead of
  the deterministic LCG).
- `hp41-core` test run produces different results when run twice with the same seed
  (determinism violation).

**Phase to address:** Phase 33 (core ops: lock RNG type and serde annotation
before writing the RAND Op). Phase 36 (test hardening: add the round-trip
determinism test to the CI suite).

---

### Pitfall 21: Sigma-Register Collision and Layout Conflicts with v2.2 SCI-01

**What goes wrong:**
The existing v2.2 statistics layout (locked in `ops/stats.rs` header comment) is:
```
R01 = Σx²
R02 = Σx
R03 = n (count)
R04 = Σy²
R05 = Σy
R06 = Σxy
```

Stat 1 Pac programs on real HP-41 hardware use these SAME registers (R01–R06) as
their sigma accumulator. This is intentional — Stat 1 extends the built-in Σ+/Σ−/MEAN/
SDEV foundation. However, several collision risks exist:

**A. Extended-statistics register extension.** If Stat 1 Pac adds functions for
higher moments (skewness, kurtosis) or histogram operations, it may need R07–R12 or
another block. If the emulator hard-codes a "Stat 1 uses R07–R12" assumption without
verifying against the Stat 1 OM, any user program that stores values in R07–R12 will
corrupt the Stat 1 bookkeeping silently.

**B. SIZE floor conflict.** The v2.2 fail-closed guard `if state.regs.len() < 7` in
every `stats.rs` function fires when the user shrinks the register bank below R06.
If Stat 1 uses registers up to R12 (hypothetically), the SIZE floor for Stat 1-using
programs rises to 13. Currently no guard enforces a minimum SIZE for XROM module ops.
Without the guard, Stat 1 functions accessing R07–R12 will panic-on-out-of-bounds
(caught by `#[deny(clippy::unwrap_used)]` at compile time but manifesting as a
checked-index-out-of-range HpError at runtime that surfaces as DATA ERROR — confusing
but not a panic).

**C. CLΣSTAT interaction.** The existing `op_cl_sigma_stat` zeroes R01–R06. If Stat 1
adds an "extended CLΣSTAT" that also zeroes R07–R12, the two ops must share
consistent semantics. If `CLΣSTAT` (v2.2) zeroes only R01–R06 but a Stat 1 function
reads R07 expecting it to be initialized by Stat 1's own clear function, a user who
runs `CLΣSTAT` after some Stat 1 accumulation gets a partially-cleared register block
and wrong higher-moment results.

**Why it happens:**
The v2.2 Σ-register layout was locked in Phase 6 for the built-in functions and is
correct for those functions. Stat 1 Pac was designed on the same hardware and uses the
same register layout — but the specific register allocation for Stat 1 extensions MAY
differ from what the emulator assumes based on the v2.2 layout.

**How to avoid:**
- Before Phase 33: read the Stat 1 Pac OM's "Storage Registers" section (every HP
  PAC manual has one). Transcribe the exact register layout into a comment at the top
  of `hp41-core/src/ops/stats1/mod.rs` (the new module, analogous to math1/xrom.rs).
  Verify it matches or extends R01–R06 exactly.
- If Stat 1 uses additional registers (R07–R12), add a second fail-closed guard:

  ```rust
  // In every Stat 1 function that accesses registers beyond R06:
  if state.regs.len() < STAT1_MAX_REG + 1 {
      return Err(HpError::InvalidOp);
  }
  ```

  where `STAT1_MAX_REG` is a `pub const` derived from the OM's register table.
- Document the register layout in `docs/hp41-stat1-divergences.md` analogous to
  the `D-30-NN` divergence catalog pattern from Math Pac I.
- If Stat 1's CLΣSTAT clears MORE registers than v2.2's, extend `op_cl_sigma_stat`
  conditionally: clear R01–R06 always; clear R07–R12 only if `state.xrom_modules`
  has the Stat 1 bit set. Document this as a divergence entry.

**Warning signs:**
- Stat 1 "skewness" function returns a wrong value on a known distribution (e.g.
  skewness of a normal sample should be near 0) when the user has not run `CLΣSTAT`
  first. Root cause: R07 was non-zero from a previous computation.
- A user program that stores intermediate results in R07 and then calls a Stat 1
  function produces wrong results. Root cause: Stat 1 overwrote R07 without
  documentation.

**Phase to address:** Phase 33 (core ops: register layout locked from OM BEFORE
any Stat 1 Op is implemented). Phase 36 (test hardening: add a `SIZE` floor test —
set SIZE to 10, call a Stat 1 function that needs R12, assert clean InvalidOp error).

---

## Moderate Pitfalls

---

### Pitfall 22: ALPHA-Prompt Name Shadowing with Built-In Mnemonics

**What goes wrong:**
Stat 1 Pac workflows use ALPHA-driven modal prompts (e.g. "MEAN?", "SDEV?", "N=?",
"CORR?"). The v3.0 modal infrastructure routes ALPHA input through the `modal_program`
state machine — the prompt name is NOT resolved as a function call. However, the
CLI's `xeq_by_name_local_resolve` fast-path resolver does check a static set of
built-in names. If a Stat 1 XROM function is registered under the SAME mnemonic as
a v2.2 built-in (e.g. a Stat 1 Pac "extended MEAN" also named `"MEAN"`), the built-in
wins per the Pitfall 1 (v3.0) convention — and the Stat 1 version is unreachable via
`XEQ "MEAN"`.

The specific conflict candidates for Stat 1 Pac are:
- `"MEAN"` — v2.2 built-in `Op::Mean` in `stats.rs`. A Stat 1 "bivariate MEAN" or
  "vector MEAN" function must be named differently (e.g. "VMEAN" or "MEAN2").
- `"SDEV"` — v2.2 built-in `Op::Sdev`. Same issue.
- `"CORR"` — v2.2 built-in `Op::Corr`.
- `"LR"` — v2.2 built-in `Op::Lr`.
- `"YHAT"` — v2.2 built-in `Op::Yhat`.

On real HP-41 hardware, Stat 1 Pac adds EXTENDED versions of these functions that
operate on a larger register set or accept different parameters. If the OM names them
identically to the built-ins, the emulator must implement the Stat 1 version as the
XROM variant (accessible via `XEQ "MEAN"` when Stat 1 module is loaded) and the v2.2
built-in as the fallback (accessible via direct keyboard Σ/stats keys). This is the
same disambiguation decision that Pitfall 1 (v3.0) locked for Math Pac I — but Stat 1
has MORE potential collisions because it extends existing stats functions.

**Why it happens:**
The v3.0 XROM resolver chain resolves built-ins FIRST, XROM LAST. If the Stat 1 OM
names its extended MEAN function `"MEAN"`, the XROM-registered `Op::Stat1Mean` is
unreachable because `xeq_by_name_local_resolve` returns the built-in first. The
developer must either (a) accept that `XEQ "MEAN"` always calls the v2.2 built-in,
or (b) switch the resolver priority so XROM wins when the module is loaded, or (c)
give Stat 1 functions disambiguated names.

**How to avoid:**
- Before Phase 33: enumerate every Stat 1 Pac OM function name against the v2.2
  built-in mnemonic list in `docs/hp41cv-functions.json`. Flag each collision.
- For each collision: decide between (a) different mnemonic in Stat 1 (preferred —
  no resolver change), (b) module-loaded-priority override (requires Pitfall 1
  resolver-chain redesign — probably too expensive for v3.1).
- Add `STAT_1.ops` to the `tests/xrom_shadowing.rs` CI gate — this gate already
  checks Math Pac I; extending it to Stat 1 is a 5-line addition. The gate asserts
  that no STAT_1 mnemonic is ALSO a v2.2 built-in mnemonic with the same string.
  Any collision that slips through testing surfaces immediately.
- Document each Stat 1 function that has a v2.2 built-in namesake in
  `docs/hp41-stat1-divergences.md` with an OM citation for the naming decision.

**Warning signs:**
- `xrom_shadowing.rs` CI gate fails with a new entry in the collision list.
- A user types `XEQ "CORR"` expecting the Stat 1 extended correlation function and
  gets the v2.2 built-in result (which requires Σ-registers R01–R06 to be pre-loaded
  via Σ+, not a raw data block).

**Phase to address:** Phase 33 (core ops: confirm mnemonic list from OM before
registering any STAT_1 entry in `xrom.rs`). Phase 34 (CLI integration: extend
`xrom_shadowing.rs` gate).

---

### Pitfall 23: Distribution CDF Tolerance Baseline — Tighter Than 1e-7 May Be Needed

**What goes wrong:**
The Math Pac I relative-tolerance baseline is 1e-7 (6 of 10 HP-41 digits
guaranteed, last 4 platform-dependent). This is appropriate for INTG/SOLVE/DIFEQ
because those methods iterate and accumulate rounding error. Distribution CDFs and
PDFs are NOT iterative in the same sense — they evaluate a closed-form expression
(possibly via a polynomial approximation of erf, incomplete gamma, or incomplete beta)
in a fixed number of operations. For a well-implemented distribution function,
`rust_decimal`'s 28-digit precision should yield results that match 10-digit HP BCD
tables to all 10 displayed digits.

Using a 1e-7 relative tolerance for distribution tests would allow a normal CDF
implementation to be wrong by 0.0001% and still pass. Statistical tables are tabulated
to 4–8 significant digits; users comparing the emulator's output against tables
expect ≤ 1 ULP agreement at 4-digit display.

The wrong direction: tightening to 1e-10 (matching rust_decimal's native precision)
is also wrong because HP-41 hardware uses 10-digit BCD and rounds intermediate
results. A result that matches the HP-41 hardware output exactly may differ from the
"true" mathematical value at the 1e-10 level.

**Why it happens:**
Re-using the Math Pac I `1e-7` floor as a universal stats tolerance is expedient —
the `lint_math1_assertions.rs` CI gate enforces it for math1 tests. If the same gate
is naively applied to stat1 tests without distinguishing iterative vs. closed-form,
distribution tests become too loose.

**How to avoid:**
- Establish a TWO-LEVEL tolerance policy for Stat 1:
  - **Closed-form / polynomial-approximation ops** (CDF, PDF, erf-based functions):
    tolerance = 1e-9 (9 of 10 digits). These should agree with HP BCD tables to
    this level. Use `assert_hp_close!(actual, expected, 1e-9)` in test files.
  - **Iterative ops** (quantile inversion, curve-fitting): tolerance = 1e-7 (matches
    Math Pac I floor). Use `assert_hp_close!(actual, expected, 1e-7)` in test files.
- Create a `tests/stat1_accuracy.rs` file distinct from the existing
  `tests/numerical_accuracy.rs`. The new file uses the two-level tolerance policy
  from day one — do not mix Stat 1 and Math Pac I accuracy cases in the same file
  (the lint gate filenames are `math1_*`; keeping `stat1_*` separate allows
  distinct lint rules if needed).
- When the Stat 1 OM quotes example values (e.g. "P(Z < 1.645) ≈ 0.9500"), use
  those as the expected values in accuracy tests — not a modern high-precision
  reference. The goal is to match what HP-41 hardware produces, not to be more
  accurate than the OM.

**Warning signs:**
- A normal CDF test passes at 1e-7 tolerance but fails at 1e-9 — the implementation
  is losing 2+ significant digits somewhere in the polynomial approximation.
- A normal CDF test that was passing at 1e-9 starts failing on a new platform after
  a `rust_decimal` upgrade — the intermediate precision changed. Check whether the
  polynomial coefficients use `f64` literals (which round differently on x86 vs ARM).

**Phase to address:** Phase 33 (core ops: establish tolerance baseline per function
category BEFORE writing tests). Phase 36 (test hardening: `stat1_accuracy.rs` with
the two-level tolerance policy enforced).

---

### Pitfall 24: `xrom_modules` Bit Allocation and Default Value Update

**What goes wrong:**
The `default_xrom_modules()` function in `hp41-core/src/state.rs` currently returns
`0b0000_0001` (Math 1 loaded, Stat 1 not loaded). The comment in `xrom.rs` line 134
reserves bit 1 for Stat 1: `// if modules & 0b0000_0010 != 0 { stat1_resolve(name) }`.

When Stat 1 is added, `default_xrom_modules()` must change to `0b0000_0011` so both
modules are pre-loaded by default. This creates a BACKWARD-COMPAT wrinkle:

**Case A: v3.0 save file that serializes `xrom_modules`.**
The `#[serde(default = "default_xrom_modules")]` annotation means: if the JSON field
is PRESENT, use the stored value. If ABSENT, use the default function. A v3.0 save
file where `xrom_modules: 1` is present will deserialize with `xrom_modules = 1` —
even after the default function returns 3. The user will not have Stat 1 loaded on
their first launch after upgrade. They will need to manually enable Stat 1 (or the
emulator provides a migration path).

**Case B: Pre-v3.0 save file (no `xrom_modules` field).**
The field was introduced in v3.0 with `#[serde(default = "default_xrom_modules")]`.
A v2.2 save file has no `xrom_modules` key — serde uses the default function, which
after v3.1 returns 3. Pre-v3.0 users get BOTH Math 1 AND Stat 1 on their first launch
after upgrading to v3.1. This is probably the desired behavior.

**The migration decision for v3.0 save files:**
Option A: Accept that v3.0 users need to re-enable Stat 1 manually. Document in
release notes. Low implementation cost.
Option B: On load, if `xrom_modules == 1` (exactly the v3.0 default), set it to 3
automatically. This is a one-way migration that requires a version field in the save
file (currently absent). Implementation cost: moderate.
Option C: Change the default function to `0b0000_0011` and accept that v3.0 save
files that explicitly stored `1` will silently lose Stat 1 enablement on first load,
but a "first launch after upgrade" check (or just documentation) covers this.

The recommended approach (locks in Phase 33): use Option A with a release note, plus
a startup migration: if `xrom_modules & 0b0000_0010 == 0` on the first dispatch after
load, set the bit and re-save. This is the least invasive change.

**Why it happens:**
The serde default-function pattern works correctly for NEW fields that did not exist
before. It does NOT auto-upgrade EXISTING fields whose stored value is now stale. The
`xrom_modules` field was introduced with value 1; upgrading the default to 3 does not
retroactively change stored-1 saves.

**How to avoid:**
- In Phase 33 Phase 0 (framework extension): update `default_xrom_modules` to return
  `0b0000_0011`. Add the startup-migration one-liner in `state.rs::CalcState::new()`
  or in the serde post-deserialization hook (implement `serde_with::DeserializeAs`
  or a manual `impl<'de> Deserialize<'de> for CalcState` — the latter is heavy; the
  simpler option is to add a `fn migrate_xrom_modules` called once from the
  persistence layer after deserialization).
- Lock the v3.1 release notes to mention: "Stat 1 Pac auto-enabled on first launch;
  v3.0 save files upgraded automatically."
- Add a test: deserialize a v3.0-era JSON fixture with `xrom_modules: 1`, run the
  migration, assert `xrom_modules == 3`.

**Warning signs:**
- After installing v3.1, `XEQ "MEAN2"` (hypothetical Stat 1 MEAN extension) returns
  `"MEAN2 is planned for a future phase"` stub-error instead of running. Root cause:
  the loaded save file has `xrom_modules: 1`, bit 1 is clear, `xrom_resolve` skips
  Stat 1 entirely.
- The v3.1 `xrom_shadowing.rs` test for `STAT_1` fails because `xrom_resolve("MEAN2",
  0b0000_0011)` returns None — the stat1_resolve function was not wired in.

**Phase to address:** Phase 33 (framework extension: first commit that adds `STAT_1`
must update `default_xrom_modules` and the `xrom_resolve` bit-1 branch in `xrom.rs`).

---

### Pitfall 25: Cancellation Channel Omission in Iterative Stat 1 Ops

**What goes wrong:**
The v3.0 `request_cancel` infrastructure (Pitfall 11 mitigation) is already shipped:
- `state.cancel_requested: Arc<AtomicBool>` field on `CalcState`
- `request_cancel` Tauri command flips the flag
- INTG/SOLVE/DIFEQ check the flag every N sample points and release the Mutex

The risk for Stat 1 is not building the infrastructure (it exists) — it is FORGETTING
to wire new iterative ops into the channel.

Stat 1 Pac functions that are likely iterative:
- Quantile inversion functions (Newton/bisection iterations, potentially 50+ steps)
- Curve-fitting routines (iterative least-squares, potentially 100+ steps)
- Any function that iterates over a user-supplied register block (N potentially large)

For single-distribution-evaluation functions (PDF, CDF) — NOT iterative — the
`cancel_requested` check is unnecessary overhead. The distinction matters for Phase 33
implementation: add the check ONLY in functions that iterate.

**Why it happens:**
The developer implementing a "small" quantile inversion function (50 Newton steps
with fast arithmetic) thinks "this won't block the GUI for more than a few ms" and
skips the `cancel_requested` check. On a 10-degree-of-freedom t-distribution with
p=0.9999, the Newton iteration diverges near the tail and runs the full 50 steps
with expensive incomplete-beta evaluations — wall time is now 2–5 ms on modern
hardware but could be 200 ms on slower machines. Multiply by a user program that
calls `TINV` in a loop (curve-fitting outer iteration) and the GUI freezes.

**How to avoid:**
- Any Stat 1 function with a loop that runs > 10 iterations MUST check
  `state.cancel_requested.load(Ordering::Relaxed)` at the top of each iteration.
  This is a compile-time-unenforceable rule; enforce it via code review checklist.
- Add a `// Cancellation: check cancel_requested at iteration start` comment in the
  function template used for new stat1 iterative ops (analogous to the v3.0 INTG
  implementation's pattern).
- For the Mutex release interval: release every 64 iterations for distribution
  quantile functions (fast per-step) and every 16 iterations for curve-fitting
  (slower per-step). Document the interval choice in the function's doc-comment.
- CI gate: add a test in `tests/stat1_cancellation.rs` that sets `cancel_requested`
  BEFORE calling a quantile inversion function, asserts the function returns
  `Err(HpError::Interrupted)` without computing the answer.

**Warning signs:**
- A GUI E2E smoke test that clicks "TINV" with p=0.0001 and immediately clicks R/S
  hangs for > 500 ms before returning control. Root cause: `cancel_requested` not
  checked in the Newton iteration.
- A Stat 1 test that calls a quantile function with a deliberately non-convergent
  input (e.g. p=2.0, outside [0,1]) loops to max iterations instead of returning
  `DATA ERROR` after 50 steps.

**Phase to address:** Phase 33 (core ops: add `cancel_requested` check to every
iterative Stat 1 function as part of the initial implementation — do not defer to
a "cleanup phase"). Phase 36 (test hardening: `stat1_cancellation.rs`).

---

## Minor Pitfalls

---

### Pitfall 26: Stat 1 Divergence Catalog File Naming and Phase Addressing

**What goes wrong:**
The Math Pac I divergence catalog lives in `docs/hp41-math1-divergences.md` per the
D-29.1 precedent (Phase 29, Plan 30-02 DOC-04 expanded format). The catalog uses
`D-30-NN` identifiers tied to the PHASE where the entry was written (Phase 30).

For Stat 1, the analogous file must be `docs/hp41-stat1-divergences.md`. The risk is:
(a) re-using the same `D-30-NN` numbering (collision), or (b) calling the file
`hp41-math1-divergences.md` by mistake (conflation), or (c) not creating the file
until Phase 36 test hardening when the entries should have been written in Phase 33.

Stat 1-specific divergences that should go into this file from the start of Phase 33:
- The register layout confirmation or divergence (Pitfall 21 outcome)
- The CLΣSTAT extended-clear policy (if Stat 1 changes it)
- The RNG algorithm (Pitfall 20: LCG constants vs. hardware-faithful sequence)
- The convergence termination criterion for each iterative function (Pitfall 19)
- The `xrom_modules` default migration (Pitfall 24)

**How to avoid:**
- Create `docs/hp41-stat1-divergences.md` in Phase 33 Phase 0 (framework), with the
  same five-field format as `hp41-math1-divergences.md`:
  OM citation / our behavior / OM behavior / rationale / see.
- Use `D-33-NN` identifiers (tied to Phase 33, the creation phase) for the first
  batch of entries. If additional entries are written in Phase 34 or later, use
  `D-34-NN` etc. — the same pattern as Math Pac I's `D-30-NN` entries.
- Never add Stat 1 entries to the Math Pac I divergence file — they are separate
  modules and separate OM documents.

**Phase to address:** Phase 33 (framework: create the file stub before the first
Stat 1 Op is committed). Phase 36 (test hardening: complete all entries before the
v3.1 accuracy test suite is finalized).

---

### Pitfall 27: Free42 Stats-Domain Identifiers Not in Contamination Guard

**What goes wrong:**
The `scripts/check-free42-contamination.sh` CI gate greps for 12 distinctive
identifiers that appear in Free42's GPL source code for Math Pac I operations
(`core_math1.cc`, decNumber, Intel BID). Free42 also has `core_math2.cc` and
`core_sto_rcl.cc` with statistics-related code. The statistics domain uses different
identifiers: `free42_stats`, `do_normal_cdf`, `NSTAT`, `do_linear_regression`,
`do_chi_square_cdf`, etc.

If v3.1 Stat 1 ops are accidentally contaminated with Free42 statistics code, the
existing 12-symbol grep will NOT catch it because the stats-domain identifiers differ
from the math-domain identifiers already in the guard.

**How to avoid:**
- Before Phase 33: inspect Free42's `core_math2.cc` (the statistics implementation
  file) and identify 6–8 distinctive function/variable names that appear in Free42
  but should NOT appear in `hp41-core`. Add them to the grep pattern in
  `scripts/check-free42-contamination.sh`.
  Candidates (to verify against Free42 source): `do_xroot_of_y`, `stats_x`,
  `stat1_n`, `reg_names_n`, `do_normcdf`, `t_cdf_helper`.
- Update the header comment in each `hp41-core/src/ops/stats1/*.rs` file to use the
  Stat 1 OM citation variant:
  `// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-15001;`
  `// Free42 source consulted only as sanity-check oracle, not copied.`
  (The existing Math Pac I files carry the analogous math1 citation per ADR-002.)
- The CI script `check-free42-contamination.sh` must list both the math1 AND stats1
  disclaimer sentences as REQUIRED headers (present in every math1/stats1 source file)
  and both math1 AND stats1 distinctive identifiers as BLOCKED patterns.

**Phase to address:** Phase 33 (framework: update the contamination script before
any stats1 source file is written). Phase 36 (test hardening: verify all new
stats1 source files carry the correct per-file header).

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Re-use `op_sdev`'s two-pass formula for new stats ops | 0 new code | Silent wrong answers on nearly-equal data sets | Never — use Welford for raw-block ops |
| Skip `cancel_requested` check in "small" quantile inversion | Simpler code | GUI freeze on tail inputs in user-program loops | Never — add the check even for small N |
| `#[serde(default, skip)]` on `rand_seed` | Consistent with other transient fields | RNG state lost on save/load; simulations not reproducible | Never — `rand_seed` must NOT be skip |
| Reuse Math Pac I 1e-7 tolerance for distribution CDFs | No new tolerance logic | Distribution results off by 10 ULP pass tests silently | Only acceptable for iterative distribution ops (quantile inversion) |
| Add Stat 1 Op variants to existing math1 source files | Fewer new files | math1 files are frozen per CLAUDE.md invariant | Never — new `stats1/` subdirectory required |

---

## Integration Gotchas

| Integration point | Common Mistake | Correct Approach |
|-------------------|----------------|------------------|
| `xrom_resolve` bit-1 branch | Forgetting to uncomment the `stat1_resolve` stub in `xrom.rs` line 134 | Uncomment and implement `stat1_resolve` in Phase 33 Phase 0 |
| `default_xrom_modules` | Returning `0b0000_0001` after adding Stat 1 | Update to `0b0000_0011`; add startup migration for v3.0 save files |
| `STAT_1.ops` entry count | Not updating the `math1_ops_has_correct_entry_count` test | Add a `stat1_ops_has_correct_entry_count` test with the expected Stat 1 Op count |
| `op_display_name()` in both `prgm_display.rs` files | Adding Stat 1 Op variants to only cli's copy | Four-way exhaustive-match invariant: both CLI and GUI copies must be updated simultaneously |
| `docs/hp41-stat1-functions.json` | Not creating this file before Phase 34 CLI integration | Create in Phase 33 alongside the Op variants; Phase 34 OnceLock wires it in |
| `scripts/docs-matrix` | Not extending to three-input mode for the Stat 1 JSON source | `docs-matrix` already handles two JSON sources; add a third input mode |

---

## "Looks Done But Isn't" Checklist

- [ ] **RAND implementation:** `rand_seed` field added to `CalcState` with `#[serde(default)]` (NOT skip) — verify by checking serde annotations
- [ ] **Stat 1 Σ-register layout:** confirmed from Stat 1 OM "Storage Registers" section — verify by checking `stats1/mod.rs` header comment against OM
- [ ] **`xrom_modules` default migration:** v3.0 save file with `xrom_modules: 1` loads correctly and gets bit 1 set — verify via serde round-trip test
- [ ] **Cancellation wiring:** every iterative Stat 1 function checks `cancel_requested` at loop top — verify via `stat1_cancellation.rs` test
- [ ] **Contamination guard extended:** `check-free42-contamination.sh` includes stats-domain identifiers — verify by running script against a file containing a known Free42 stats identifier
- [ ] **Divergence catalog created:** `docs/hp41-stat1-divergences.md` exists with entries for each known divergence before v3.1 ships — verify file exists and has ≥ 3 entries
- [ ] **Four-way exhaustive match:** all Stat 1 Op variants present in dispatch, execute_op, cli/prgm_display, gui/prgm_display — compile-time enforced
- [ ] **Per-Op test count ≥ 5:** `math1_op_test_count.rs` CI gate extended to cover stat1 Op variants — verify by checking gate configuration

---

## Pitfall-to-Phase Mapping

| Pitfall | Name | Prevention Phase | Verification |
|---------|------|-----------------|--------------|
| P18 | Variance catastrophic cancellation | Phase 33 (core ops) | `tests/stat1_variance_stability.rs` with nearly-equal values |
| P19 | Quantile inversion non-convergence | Phase 33 (core ops) | `tests/numerical_accuracy.rs` extended; ≥ 10 tail quantile cases |
| P20 | RNG serde and SystemRandom contamination | Phase 33 (core ops) | `tests/stat1_rand_determinism.rs` round-trip test; `Cargo.toml` review |
| P21 | Sigma-register layout collision | Phase 33 (core ops) | OM register table transcription; SIZE floor test |
| P22 | ALPHA-prompt mnemonic shadowing | Phase 33 (core ops + xrom.rs) | `xrom_shadowing.rs` extended to STAT_1.ops |
| P23 | Distribution tolerance baseline | Phase 33 (core ops) | `stat1_accuracy.rs` two-level tolerance policy |
| P24 | `xrom_modules` bit allocation | Phase 33 (framework) | Serde migration test; startup bit-set verification |
| P25 | Cancellation channel omission | Phase 33 (core ops) | `stat1_cancellation.rs`; GUI E2E smoke for R/S-during-TINV |
| P26 | Divergence catalog file naming | Phase 33 (framework) | File exists and uses `D-33-NN` identifiers before Phase 36 |
| P27 | Free42 stats contamination gap | Phase 33 (framework) | `check-free42-contamination.sh` updated; per-file headers on all stats1 sources |

---

## Sources

**Confidence: HIGH** (project codebase — directly verifiable)
- `hp41-core/src/ops/stats.rs` — v2.2 Σ-register layout, existing variance formula,
  fail-closed `regs.len() < 7` guard pattern.
- `hp41-core/src/state.rs` — `CalcState` field patterns: `#[serde(default)]` vs
  `#[serde(default, skip)]`; `default_xrom_modules()` current return value `0b0000_0001`;
  `cancel_requested: Arc<AtomicBool>` infrastructure from v3.0.
- `hp41-core/src/ops/math1/xrom.rs` — bit-1 Stat 1 stub comment at line 134;
  bit-0 Math 1 check pattern for replication.
- `docs/hp41-math1-divergences.md` — D-29.1 divergence catalog format and naming
  convention as template for `hp41-stat1-divergences.md`.
- `docs/adr/v3.0-002-user-callback-policy.md` — ADR-002 strict-reject policy as
  precedent for Stat 1 iterative op policy decisions.
- `CLAUDE.md` frozen invariants — `#[serde(default, skip)]` discipline; 4-way
  exhaustive match; `hp41-core/src/ops/math1/` is frozen; no `println!` in core.

**Confidence: MEDIUM** (HP documentation — public-domain, requires verification)
- HP Stat 1 Pac Owner's Manual (HP 00041-15001) — the authoritative spec for Stat 1
  behavioral emulation. Register layout, convergence criteria, distribution function
  algorithms, and worked examples are all in this document. Must be verified against
  a physical or scanned copy before Phase 33 implementation begins.
  Available at: https://www.hpmuseum.org/software/41/41stat.htm (scan may be available;
  verify access during Phase 33 research setup).

**Confidence: MEDIUM** (numerical analysis — well-established, context-dependent)
- Welford's online algorithm: Welford, B. P. (1962), "Note on a method for calculating
  corrected sums of squares and products", Technometrics 4(3): 419–420. Public domain,
  widely reproduced. Used in the variance stability recommendation (Pitfall 18).
- Abramowitz and Stegun §26.2.17 rational approximation for normal quantile — initial
  guess recommendation (Pitfall 19). P.D in public domain.
- Beasley-Springer-Moro algorithm for normal quantile initial guess — alternative to
  A&S, higher accuracy. Springer (1977) "Algorithm AS 111". MEDIUM confidence that
  this is what the HP Stat 1 OM specifies; verify against the OM.

**Confidence: LOW** (unverified — flag for Phase 33 research)
- Free42 `core_math2.cc` statistics identifiers — the specific identifier names
  listed in Pitfall 27 are HYPOTHETICAL based on Free42's code organization pattern.
  Verify against actual Free42 source before adding to the contamination script.
  URL: https://github.com/thomasokken/free42/blob/master/common/core_math2.cc
- HP Stat 1 Pac XROM ID number — reported as a different module ID from Math Pac I
  (XROM 7 = "MATH 1A"). Stat 1's hardware XROM ID must be confirmed from the Stat 1
  OM or from community sources (e.g. HP-41 MCODE FAQ, MoHPC forum) before assigning
  a module bit position in `xrom_modules`.
  Risk: if Stat 1 is XROM N for some N not equal to what we assume, the CATALOG 2
  enumeration will display the wrong module name.

---
*Pitfalls research for: HP-41 Stat 1 Pac behavioral emulation (v3.1)*
*Researched: 2026-05-21*
*Supersedes: The previous PITFALLS.md in this directory (v3.0 Math Pac I, Pitfalls 1–22).*
*Math Pac I pitfalls 1–22 remain valid as mitigated invariants; see*
*`.planning/milestones/v3.0-ROADMAP.md` and `CLAUDE.md` for their current status.*
