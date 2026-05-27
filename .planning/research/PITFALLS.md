# Pitfalls: HP-41 Advantage Pac + Advanced Matrix Pac Emulation (v3.3)

**Milestone:** v3.3 Advantage Pac + Advanced Matrix Pac (fourth XROM module(s))
**Researched:** 2026-05-25
**Scope:** Pitfalls SPECIFIC to adding Advantage Pac and Advanced Matrix Pac behavioral
emulation on top of the shipped v3.2 codebase. Pitfalls 1-42 from v3.0/v3.1/v3.2 that
are already gate-checked in CI are NOT repeated here unless v3.3 introduces a new failure
mode on top of the mitigated pattern. Where a prior pitfall has a v3.3-specific extension,
that extension is called out explicitly with a cross-reference.

**Confidence (overall):** MEDIUM.
The framework mechanics (new XROM module registration, 4-way exhaustive match,
serde backward compat, XROM shadowing) are HIGH confidence because they follow an
established three-milestone pattern. The PROOT algorithm for arbitrary-degree
polynomials is MEDIUM confidence — the hardware uses an iterative algorithm whose
exact specification requires Owner's Manual verification (the OM document 00041-90482
was not accessible during this research session). The Advanced Matrix Pac's exact
function set and XROM ID are MEDIUM confidence because community sources describe
its contents as a superset of the Advantage ROM matrix functions but disagree on
whether it is an official HP product or a community module.

**Already mitigated (do not re-document):**
- P1: `xrom_resolve` fires LAST — `tests/xrom_shadowing.rs` CI gate; v3.3 extends to ADV.ops.
- P11: Long-running op Mutex release + `request_cancel` — v3.0 infrastructure applies.
- P14: Cross-platform f64 drift — `lint_math1_assertions.rs` CI gate.
- P16: Per-Op test count >= 5 — `xrom_op_test_count.rs` meta-gate; v3.3 adds ADV variants.
- P17: `assert_eq!` on iterated HpNum — blocked by CI.
- P19: Free42 GPL contamination guard — 18-token `check-free42-contamination.sh`; v3.3
  extends to the new module directory(ies).
- P20: serde shape for persistent-non-obvious fields — `rand_seed` established the pattern.
- P22: XROM shadowing — disjointness test now covers Math 1 x Stat 1 x Time; v3.3 adds ADV.

---

## Summary

Nine pitfall categories dominate the v3.3 risk surface. They break into five clusters:
module boundary / freeze (3 pitfalls), PROOT numerics (2), complex stack sharing (1),
Romberg vs Simpson coexistence (1), and infrastructure (2):

1. **math1/ freeze boundary violation** (P43) — CRITICAL. Advantage Pac complex ops
   (CABS, CARG, CCHS, CCONJ, CY^X) are semantically related to Math Pac I complex ops
   but must NOT be added to `math1/complex.rs`. They require their own module tree.

2. **PROOT algorithm selection for arbitrary degree** (P44) — CRITICAL. Math Pac I
   POLY handles degrees 2-5 only via Bairstow deflation. Advantage Pac PROOT must handle
   at least degree 6+ (community sources report "limited only by RAM"). Bairstow
   deflation degrades catastrophically for repeated roots at high degree.

3. **Complex stack sharing — two XROM modules using the same overlay** (P45) — HIGH.
   Math Pac I complex ops use `state.complex_mode` and the X/Y/Z/T overlay. Advantage
   Pac complex ops must use the same mechanism, but adding them in a separate module
   directory means they must import `math1::complex` helpers or duplicate them.

4. **XROM ID collision — Advantage Pac occupies XROM 22 and XROM 24** (P46) — HIGH.
   The xrom_modules bitfield is currently `u8` (bits 0-2 used for Math 1/Stat 1/Time).
   Advantage Pac hardware uses two XROM slots: XROM 22 and XROM 24 (117 functions split
   across two ROM pages). A single `u8` bitfield has only 8 bits — XROM IDs 22 and 24
   cannot be encoded as single bits in a u8.

5. **Matrix register layout conflict with Math Pac I MATRIX** (P47) — HIGH.
   Math Pac I uses R14 for order and R15+ for column-major elements. Advanced Matrix Pac
   uses the same HP-41 register file but its own matrix-storage addressing scheme.
   Sharing `state.matrix_dim` / `state.matrix_active_reg` fields across two XROM modules
   risks silent corruption.

6. **Romberg INTG naming conflict with Math Pac I INTG** (P48) — HIGH.
   Both Math Pac I and the Advantage Pac provide an integration function. On real hardware,
   both coexist: Math Pac I XEQ "INTG", Advantage Pac XEQ "INTEG" (note the spelling
   difference). The XROM resolver must not unify them.

7. **serde field explosion for Advanced Matrix state** (P49) — MEDIUM.
   Advanced Matrix Pac needs matrix state (dimension, base register) fields that may
   conflict with the existing `matrix_dim` / `matrix_active_reg` fields that Math Pac I
   already owns in `CalcState`.

8. **xrom_modules bitfield size** (P50) — MEDIUM.
   Current `xrom_modules: u8` has bits 0, 1, 2 assigned to Math 1, Stat 1, Time.
   Advantage Pac (two XROM IDs: 22 and 24) would need new bit positions. The field type
   must be widened to `u16` or `u32` before the Advantage Pac can be registered.

9. **Free42 contamination extension to new module directories** (P51) — MEDIUM.
   `scripts/check-free42-contamination.sh` currently greps `math1/`, `stat1/`, and `time/`.
   The new `advantage/` (and optionally `adv_matrix/`) directories are not yet covered.

---

## Critical Pitfalls

### Pitfall 43: math1/ Freeze Boundary Violation

**What goes wrong:**
A developer adding Advantage Pac complex operations (CABS, CARG, CCHS, CCONJ, CY^X)
notices that `hp41-core/src/ops/math1/complex.rs` already contains all the Math Pac I
complex infrastructure: `complex_atan2`, `op_c_plus`, `op_c_minus`, complex stack overlay
semantics, and the `complex_mode` flag. The natural instinct is to add the new simple
unary ops (CABS is just sqrt(X^2 + Y^2), CARG is just atan2(Y, X)) as additional
functions in `math1/complex.rs`. This directly modifies frozen code.

**Why it happens:**
The new complex ops share ALL of the Math Pac I complex stack model (D-28.1: ζ = X+iY,
τ = Z+iT; D-28.2: complex_mode auto-on policy). This creates a genuine architectural
temptation: the code is RIGHT THERE. The freeze boundary (Plan 25-01) is documented in
CLAUDE.md and `architecture-history.md`, but the freeze rationale ("algorithm
independence from Free42") feels less relevant for simple unary stack ops like CABS.

**How to avoid:**
Create `hp41-core/src/ops/advantage/complex.rs` (new directory, not inside `math1/`).
Import the shared complex stack helpers from `math1::complex` as `pub(super)` symbols:
`complex_atan2` is already marked `#[allow(dead_code)]` and `pub(super)` in the source —
this was anticipated as a future use case. The `complex_mode: bool` field on `CalcState`
is already public and shared state; no duplication needed. The `advantage/` module reads
`state.complex_mode`, `state.stack.x`, `state.stack.y` just like `math1/complex.rs` does.

The `math1/modal.rs` and `math1/xrom.rs` are the ONLY sanctioned carve-outs
(ADR-v3.1-004, ADR-v3.2 Time variant). Advantage Pac complex ops are NOT a carve-out
candidate — they belong in a peer sibling module. The correct pattern is exactly what
`stat1/` and `time/` demonstrate: a new top-level sibling under `ops/`.

**Warning signs:**
- Any `git diff` showing modifications to `hp41-core/src/ops/math1/complex.rs`,
  `math1/poly.rs`, `math1/matrix.rs`, or any file other than `math1/modal.rs` and
  `math1/xrom.rs`.
- A compiler error mentioning `math1::complex::complex_atan2` from outside `math1`.
- The `check-free42-contamination.sh` script needing to be run against `math1/` for
  new Advantage Pac content.

**Phase to address:** Core implementation phase (first phase of v3.3 — equivalent to
Phase 28 for v3.0, Phase 33 for v3.1, Phase 38 for v3.2).

---

### Pitfall 44: PROOT Algorithm Selection for Arbitrary Degree

**What goes wrong:**
PROOT is treated as "POLY but with more degrees." A developer extends the existing
`math1/poly.rs` Bairstow deflation algorithm to handle degree 6, 7, 8, … by simply
relaxing the `degree > 5 → error` guard. The Bairstow algorithm degrades catastrophically
for repeated roots at higher degrees: for a polynomial like (x-1)^10, Bairstow deflation
produces a near-zero "quadratic factor" whose discriminant is near zero, causing division
by near-zero in the Newton step. The convergence fails silently or diverges.

**Why it happens:**
Math Pac I POLY (degrees 2-5) uses Bairstow deflation because it is exact for degree 2
(quadratic formula) and reliable for degrees 3-5 with the HP-41's 10-digit arithmetic.
At degree 5, a single Bairstow iteration converges in bounded steps. At degree 10+,
accumulated deflation error compounds — each deflated quotient inherits rounding errors
from all previous deflations, and the final linear/quadratic factors are polluted.

Community sources indicate the hardware Advantage Pac PROOT supports degrees "limited
only by available RAM" and can handle 100th-degree polynomials. This is evidence the
hardware uses a fundamentally different algorithm from Bairstow deflation.

**How to avoid:**
Use a distinct algorithm for PROOT in `advantage/poly.rs`. Recommended: Laguerre's
method applied simultaneously to all roots, with forward deflation for root isolation.
Laguerre has guaranteed global convergence for polynomials with real coefficients (proven
1973) and cubic convergence near roots. Alternatively, Durand-Kerner (Weierstrass
simultaneous iteration) is simpler to implement and converges for generic polynomials.

Do NOT extend `math1/poly.rs`. PROOT lives exclusively in `advantage/poly.rs`. PROOT
does NOT replace Math Pac I POLY — both coexist on the real hardware (different XROM
IDs, different mnemonics). Register layout: community consensus is that PROOT stores
coefficients in R00-RNN (degree-indexed) and outputs roots to the same register range
with a defined interleaving of real/imag parts. Verify against OM 00041-90482.

**Numerical stability notes for Laguerre implementation:**
- Use f64 bridge (same pattern as `distributions.rs`) — HpNum checked arithmetic is
  insufficient for complex-number intermediate values in iterative root polishing.
- Guard against zero-derivative case: if |p'(z)| < epsilon, perturb z slightly.
- Multiplicity detection: Advantage Pac presumably returns each root with its multiplicity;
  if the OM specifies multiplicity output, implement distinct root isolation with
  deflation AFTER polishing (not before).
- Convergence cap: cap iterations at degree × 100 and return `HpError::Domain` (displays
  as "DATA ERROR") if not converged. The existing `math1/poly.rs` uses `|imag| > 1e9`
  as a non-convergence sentinel — preserve this philosophy.

**Warning signs:**
- Accuracy test for degree 6+ polynomial with repeated roots fails.
- Test `(x-1)^8` produces wildly wrong roots instead of eight roots near 1.0.
- Compiler warning about dead_code in `math1/poly.rs` being removed (wrong file is being
  modified).

**Phase to address:** Core implementation phase. This is the highest-risk algorithm in v3.3.
The PROOT algorithm deserves its own sub-plan with oracle-derived test cases BEFORE the
implementation. Recommended: derive 5 oracle cases (degree 3, 5, 8, 10, and one with
repeated roots) from scipy.optimize or numpy.roots BEFORE writing any Rust code.

---

## High-Priority Pitfalls

### Pitfall 45: Complex Stack Sharing — Two XROM Modules, One Overlay Model

**What goes wrong:**
The Advantage Pac complex ops (CABS, CARG, CCHS, CCONJ, CY^X) require the exact same
complex stack model as Math Pac I (ζ = X+iY, τ = Z+iT, `complex_mode` flag). If
`complex_atan2` and the complex stack behavior are duplicated in `advantage/complex.rs`
instead of imported from `math1/complex.rs`, the two implementations will diverge at
edge cases (e.g., `complex_atan2(0,0)` must return 0.0 not NaN — Pitfall 6 in the
v3.0 research). The duplication also violates SC-4 spirit.

**Why it happens:**
`math1/complex.rs` marks most helpers as `pub(super)` — visible only within `math1/`.
`complex_atan2` has `pub(super)` visibility. A developer creating `advantage/complex.rs`
cannot call `math1::complex::complex_atan2` directly. The natural fix is to copy the
function, which introduces the drift risk.

**How to avoid:**
Promote `complex_atan2` and any other shared complex helpers to `pub(crate)` in
`math1/complex.rs`. This is a surgical visibility change that does NOT modify the
freeze-protected algorithm. The freeze applies to algorithm logic, not to `pub`
visibility modifiers. Alternative: extract a `ops::complex_utils` module with the
shared helpers (but this requires a v3.3-specific ADR because it touches the module
structure).

The `complex_mode: bool` field on `CalcState` is already `pub` and shared by design.
The `auto-on policy` (D-28.2: every binary complex op sets `state.complex_mode = true`)
must apply identically for Advantage Pac complex ops — do not add a second `adv_complex_mode`
boolean.

**Warning signs:**
- `advantage/complex.rs` contains a local `fn complex_atan2` not imported from math1.
- Divergence between CARG(0,0) in Advantage Pac and MAGZ/CINV behavior for zero input
  in Math Pac I tests.

**Phase to address:** Core implementation phase. Resolve the visibility issue in the
ADR for the Advantage Pac core phase BEFORE writing any `advantage/` code.

---

### Pitfall 46: XROM ID Collision — Advantage Pac Uses Two XROM Slots

**What goes wrong:**
The `xrom_modules: u8` bitfield currently encodes three modules as bits 0, 1, 2:
- Bit 0 = Math 1 (XROM ID 7)
- Bit 1 = Stat 1 (XROM ID 2)
- Bit 2 = Time Module (XROM ID 26)

The hardware HP-41 Advantage Pac spans two XROM slots: XROM 22 and XROM 24 (117
functions, two 4K ROM pages). A `u8` bitfield can hold only 8 bit positions. If the
Advantage Pac is naively assigned bits 3 and 4 (for the two XROM IDs), the bitfield
fills up with only one more module possible before exhaustion.

More critically: the `xrom_modules` default value and `migrate_after_load()` must be
updated. v3.2 save files have `xrom_modules = 0b0000_0111`. The migration to v3.3 must
set the Advantage Pac bits on top of `0b0000_0111`. If the bit layout changes between
v3.2 and v3.3, any `xrom_modules` field in a v3.2 save file will be misinterpreted.

**How to avoid:**
Two options:

Option A (simpler, preferred): Widen `xrom_modules` to `u16` or `u32` before v3.3 ships.
This is a single-field type change with `#[serde(default = "default_xrom_modules")]`
preserved. Old save files will deserialize the `u8` value as a `u16` transparently
(serde numeric widening works for JSON integers). The `migrate_after_load()` migration
for v3.3 sets the two new Advantage Pac bits.

Option B: Keep `u8` and use two bits for the Advantage Pac as a combined "Advantage Pac
loaded" state (both XROM 22 and XROM 24 are always loaded together — the Advantage Pac
is a single physical ROM). This is also acceptable but requires documenting the
deviation from the one-bit-per-module convention.

In `xrom_resolve`, the Advantage Pac needs a new arm that fires after Time (bit 2) and
before `Err(InvalidOp)`. The `xrom_resolve` function must check the new Advantage bit(s)
and call `advantage_resolve()`.

**Warning signs:**
- Compiler error on `xrom_modules | 0b0001_1000` (no room in u8 without overflow).
- v3.2 save-file test (`time_backward_compat.rs`) fails after widening — check JSON
  integer deserialization.

**Phase to address:** Core implementation phase, first task. Widening `xrom_modules`
must happen BEFORE any Advantage Pac Op variants are added.

---

### Pitfall 47: Matrix Register Layout Conflict Between Math Pac I and Advanced Matrix Pac

**What goes wrong:**
Math Pac I MATRIX uses `state.matrix_dim: Option<(u8, u8)>` and
`state.matrix_active_reg: Option<u8>` on `CalcState` to track the active matrix. The
column-major element storage at R15+ and the ORDER in R14 are a shared global assumption.

The Advanced Matrix Pac (if it uses different register conventions, or if it uses X-Memory
for matrix storage as community sources suggest the Advantage Pac can) may silently corrupt
the matrix state fields that Math Pac I MATRIX writes and reads.

If both Math Pac I MATRIX and Advanced Matrix Pac are simultaneously loaded and the user
runs a Math Pac I MATRIX operation followed by an Advanced Matrix Pac operation, the
shared `matrix_dim` and `matrix_active_reg` fields may hold stale state from the previous
module.

**Why it happens:**
In the real hardware, the programmer is responsible for not interleaving operations from
two matrix modules that use incompatible register conventions. The emulator doesn't have
this constraint documented as a "user responsibility" divergence. A developer may assume
`matrix_dim` and `matrix_active_reg` are safe to reuse across modules.

**How to avoid:**
Verify the Advanced Matrix Pac register conventions against its Owner's Manual before
implementing any ops. Two scenarios:

If Advanced Matrix Pac uses the same R14/R15+ convention: the existing `matrix_dim`
and `matrix_active_reg` fields are shared by design. Document this as "modules share the
same matrix register convention per HP-41C hardware ground truth" in the v3.3 divergence
catalog.

If Advanced Matrix Pac uses a different convention (e.g., named matrices in X-Memory):
add separate `adv_matrix_*` fields to `CalcState` with `#[serde(default)]` and document
the separation explicitly. Do NOT rename or repurpose the existing `matrix_dim` field
(backward compat).

**Warning signs:**
- An Advanced Matrix Pac op reads `state.matrix_dim` and gets dimensions from a
  Math Pac I MATRIX call made earlier in the session.
- Test: run Math Pac I MATRIX (3x3), then Advanced Matrix Pac op — does the Advanced
  op interpret ORDER=3 correctly or does it use its own register layout?

**Phase to address:** Core implementation phase. Must be resolved with Owner's Manual
verification BEFORE writing any Advanced Matrix Pac operations that touch CalcState
matrix fields.

---

### Pitfall 48: Romberg INTG Naming Conflict with Math Pac I INTG

**What goes wrong:**
Both Math Pac I and Advantage Pac provide numerical integration. On the real hardware:
- Math Pac I: `XEQ "INTG"` (uses Simpson's rule, registers R00-R07 as scratch)
- Advantage Pac: `XEQ "INTEG"` (different spelling, uses a Romberg/HP-15C-style algorithm)

Note the ONE LETTER difference: "INTG" (Math Pac I, XROM 7) vs "INTEG" (Advantage Pac,
XROM 22/24). If a developer adds the Advantage Pac integration function to
`advantage/xrom.rs` with the mnemonic `"INTG"` instead of `"INTEG"`, the XROM resolver
will have two modules claiming the same mnemonic. `xrom_resolve` will call the FIRST
matching resolver arm — whichever module's bit is checked first wins, and the other is
silently unreachable.

**Why it happens:**
The milestne context description calls it "Romberg-INTG" and mentions "INTG" as the
function name. Community sources and hardware records indicate the Advantage Pac uses
"INTEG" (not "INTG"). Without direct Owner's Manual verification, the mnemonic is
uncertain. Defaulting to "INTG" because "that's what INTG does" would silently shadow
Math Pac I.

**How to avoid:**
Use the hardware-faithful mnemonic: `"INTEG"` for Advantage Pac (per hardware XROM 24
database entries), `"INTG"` for Math Pac I (already in `math1/xrom.rs` line 96). The
`xrom_shadowing.rs` CI test will catch a collision at compile time: if `ADVANTAGE.ops`
contains `"INTG"`, the shadowing test fails because `MATH_1.ops` already claims it.

Similarly verify: Advantage Pac likely has `"SOLVE"` or `"FROOT"` for root-finding
(different from Math Pac I `"SOLVE"`). Community sources suggest `"FROOT"` and `"FINTG"`
as alternate names. Verify EVERY Advantage Pac function mnemonic against hardware records
before registering it in `advantage/xrom.rs` — the `xrom_shadowing.rs` test is a CI gate,
not a pre-registration guard.

**Warning signs:**
- `xrom_shadowing.rs` test fails with "collision: INTG claimed by both MATH_1 and ADVANTAGE."
- Integration accuracy tests pass for Advantage Pac but silently skip Math Pac I INTG tests.
- `just ci` passes with a `_` catch-all arm in xrom_resolve (masks the collision).

**Phase to address:** Core implementation phase, Advantage Pac `xrom.rs` registration.
Must be verified against hardware documentation before the shadowing test is extended.

---

## Medium-Priority Pitfalls

### Pitfall 49: CalcState Field Ownership Confusion for Matrix State

**What goes wrong:**
The existing `CalcState` fields `matrix_dim: Option<(u8, u8)>` and
`matrix_active_reg: Option<u8>` were added in Phase 28 specifically for Math Pac I MATRIX.
A developer implementing Advanced Matrix Pac assumes these fields are "generic matrix state"
shared by all matrix modules, and writes Advanced Matrix Pac operations that read and write
these fields directly. This creates an implicit coupling: Math Pac I MATRIX state and
Advanced Matrix Pac state are now entangled in the same two CalcState fields.

The consequence: running Math Pac I MATRIX workflow (which sets `matrix_dim` to (3,3))
followed by Advanced Matrix Pac ops that expect a freshly initialized matrix will read
stale Math Pac I dimensions.

**How to avoid:**
Treat `matrix_dim` and `matrix_active_reg` as OWNED by Math Pac I, not as shared
infrastructure. The Advanced Matrix Pac gets its own fields if it needs matrix tracking
state that differs from Math Pac I. If the Advanced Matrix Pac is designed to EXTEND Math
Pac I MATRIX (operating on the same matrix in R14/R15+), then reading these fields is
correct but must be documented explicitly as an intentional design choice.

Concretely: add a code comment on `matrix_dim` and `matrix_active_reg` stating which
module owns them. If Advanced Matrix Pac uses the same R14/R15+ layout, note
"shared by Math Pac I MATRIX and Advanced Matrix Pac per hardware ground truth" — this
makes the shared-ownership intentional rather than accidental.

**Phase to address:** Core implementation phase. Resolve ownership BEFORE any Advanced
Matrix Pac ops are written that touch CalcState.

---

### Pitfall 50: xrom_modules Bitfield Type Width

**What goes wrong:**
`xrom_modules: u8` is currently defined with:
```rust
#[serde(default = "default_xrom_modules")]
pub xrom_modules: u8,
```
and `default_xrom_modules() -> u8 { 0b0000_0111 }`.

If Advantage Pac requires bits 3 and 4 (the two XROM IDs encoded as two bits), the
default becomes `0b0001_1111`. After the v3.3 migration, v3.2 save files with
`"xrom_modules": 7` must be migrated to `"xrom_modules": 31` (decimal). This is handled
by `migrate_after_load()`.

The risk: if a developer forgets to add `migrate_after_load()` for v3.3, OR forgets to
update `default_xrom_modules()`, one of two bad things happens:
(a) New sessions default to Advantage Pac NOT loaded (bits 3+4 = 0), so XEQ "PROOT"
    returns InvalidOp silently.
(b) Old v3.2 save files do not get the Advantage Pac bits set on load, same silent failure.

**How to avoid:**
Follow the exact same migration pattern used in every prior milestone:
1. Update `default_xrom_modules()` to return `0b0001_1111` (or the equivalent for u16/u32).
2. Add a migration arm in `migrate_after_load()`:
   `if self.xrom_modules & 0b0000_0111 == 0b0000_0111 && self.xrom_modules & 0b0001_1000 == 0 { self.xrom_modules |= 0b0001_1000; }` (example for two new bits).
3. Add a backward-compat test: `v32-autosave.json` fixture with `xrom_modules: 7` should
   deserialize to `xrom_modules: 31` after `migrate_after_load()`.

The `time_backward_compat.rs` test in Phase 42 is the canonical pattern.

**Warning signs:**
- `XEQ "PROOT"` returns `InvalidOp` on a fresh session (bits not set in default).
- `XEQ "PROOT"` returns `InvalidOp` after loading a v3.2 save file (migration not written).

**Phase to address:** Core implementation phase, first task (before Op variants are added).

---

### Pitfall 51: Free42 GPL Contamination Guard — New Directory Not Scanned

**What goes wrong:**
`scripts/check-free42-contamination.sh` currently greps three directories:
`math1/`, `stat1/`, `time/`. If a developer creates `advantage/` (and optionally
`adv_matrix/`) without extending the contamination check, the CI guard misses new files.

The risk is higher for the Advantage Pac than for previous modules because:
- Romberg integration is directly inherited from the HP-15C and HP-34C, which are also
  implemented in Free42. The temptation to look at Free42's Romberg source and verify
  against it is higher than for statistics or clock arithmetic.
- PROOT for arbitrary degree may be verified against Free42's polynomial solver
  (if Free42 includes one), creating a contamination risk.

**How to avoid:**
Extend `check-free42-contamination.sh` in the SAME commit that creates the `advantage/`
directory. The script currently uses a `find -path` pattern — add `advantage/` and
`adv_matrix/` (if applicable) to the grep target list. The token count (currently 18) does
not change — the file-scope extension only adds new directories to scan, not new tokens.

Verify the extension is working: run `just license-audit` after the script change and
confirm it passes (no false positives on valid algorithmic commentary referencing Romberg).

**Warning signs:**
- `just license-audit` passes on the new `advantage/` directory even if Free42 token
  strings are present (grep is not scanning the new directory).
- A file in `advantage/` contains "Free42" as a string without the verbal disclaim pattern
  ("consulted only as sanity-check oracle, not copied").

**Phase to address:** Core implementation phase, SAME commit as `advantage/` directory creation.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Extending Bairstow deflation to degree 6+ | No new algorithm to implement | Catastrophic failures for repeated roots; silently wrong answers | Never — use Laguerre or Durand-Kerner for PROOT |
| Adding CABS/CARG to `math1/complex.rs` | No new file to create | Modifies frozen code; must be justified as a new ADR carve-out | Never without explicit ADR |
| Using `"INTG"` as mnemonic for Advantage integration | Familiar name | Silently shadows Math Pac I INTG; xrom_shadowing test catches this | Never — use hardware-faithful `"INTEG"` |
| Reusing `matrix_dim` / `matrix_active_reg` without documentation | No new CalcState fields | Implicit coupling between two XROM modules; hard to debug | Acceptable only if explicitly documented as "shared per hardware ground truth" |
| Keeping `xrom_modules: u8` and using one bit for Advantage Pac | Minimal field change | Conflates two distinct XROM IDs (22 and 24) into one bit | Acceptable only if documented: Advantage is always loaded as a unit |
| Copying `complex_atan2` from `math1/complex.rs` | No visibility change needed | Edge-case drift between two implementations; Pitfall 6 can re-emerge | Never — promote visibility instead |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Advantage Pac XROM registration | Claiming both XROM 22 and XROM 24 as separate module entries in `xrom.rs` | Register as a single `XromModule` with a combined `ops` slice; the hardware XROM IDs are internal addresses, the emulator's dispatch uses the mnemonic string |
| Math Pac I + Advantage Pac coexistence | Assuming XEQ "INTG" and XEQ "INTEG" can share the same `Op` variant | They must be distinct `Op` variants; two separate solver implementations in `math1/integ.rs` and `advantage/integ.rs` |
| `ModalProgram` extension for Advantage Pac | Adding a new `ModalProgram::Advantage(AdvantageStep)` variant in `math1/modal.rs` | Follow ADR-v3.1-004 pattern: add a single `ModalProgram::Advantage(crate::ops::advantage::modal::AdvantageStep)` variant in `math1/modal.rs` (fourth freeze carve-out), keep all semantics in `advantage/modal.rs` |
| Complex ops auto-on policy | Forgetting to set `state.complex_mode = true` in new Advantage Pac complex ops | Follow D-28.2: EVERY binary/unary complex op sets `complex_mode = true` BEFORE computation; `Op::Real` is the sole setter of `complex_mode = false` |
| PROOT register output layout | Interleaving real/imaginary parts in an undocumented order | Verify against OM 00041-90482 table of register assignments before implementing output; add a `D-NN` entry to the Advantage divergence catalog |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Laguerre iteration without per-root cap | CPU spin for degree 20+ polynomials with near-zero discriminant | Cap iterations at `degree × 100` per root; return `HpError::Domain` if not converged | Polynomials with clusters of near-equal roots at degree > 8 |
| Simultaneous Durand-Kerner without initial separation | All initial guesses converge to the same root | Use evenly-spaced complex circle initial guesses: `z_k = r × exp(2πi × k/n)` where r = Cauchy bound | Any polynomial with roots that are nearly equal |
| Advanced Matrix Pac on 14×14 matrices without register count check | Panic or register-out-of-bounds on small SIZE values | Check `state.regs.len()` before any matrix op; Math Pac I already has this check in `matrix.rs::matrix_get/set` | SIZE set smaller than matrix dimension |

---

## "Looks Done But Isn't" Checklist

- [ ] **PROOT degree range:** Verify test cases include degrees 2, 3, 5, 8, 10, and one
  polynomial with a repeated root (e.g., (x-2)^4 · (x+1)^3). If degree 6+ cases all
  pass but degree 2 regresses, PROOT is calling the wrong algorithm path.
- [ ] **Advantage Pac integration mnemonic:** Verify `ADVANTAGE.ops` contains `"INTEG"` not
  `"INTG"`. Run `xrom_shadowing.rs` and confirm zero collision with `MATH_1.ops`.
- [ ] **complex_mode auto-on:** Verify every Advantage Pac complex op (CABS, CARG, CCHS,
  CCONJ, CY^X) sets `state.complex_mode = true` before computation. Regression test:
  `complex_mode` is false before call, true after.
- [ ] **xrom_modules migration:** Load a v3.2 `autosave.json` (xrom_modules=7) and verify
  Advantage Pac functions are reachable after `migrate_after_load()`.
- [ ] **4-way exhaustive match:** Verify ALL new `Op` variants appear in BOTH `dispatch()` AND
  `execute_op()` AND CLI `prgm_display.rs` AND GUI `prgm_display.rs` before the
  first phase ships. Compiler will catch items 1+2; items 3+4 only catch on CI
  after the respective CLI/GUI phases.
- [ ] **Free42 contamination scan:** Verify `check-free42-contamination.sh` exits 0 after
  scanning `advantage/` directory. If Romberg integration algorithm is re-derived from
  the Romberg ACM paper (not from Free42), the disclaim comment must still be present.
- [ ] **XROM shadowing extended:** Verify `xrom_shadowing.rs` includes `ADVANTAGE.ops` in
  the disjointness check against `MATH_1.ops`, `STAT_1.ops`, `TIME_MODULE.ops`, and
  `BUILTIN_CARD_OP_NAMES`.
- [ ] **JSON canonical pipeline (5th pool):** Verify `docs/hp41-advantage-functions.json`
  is loaded as the 5th `OnceLock<Vec<HelpEntry>>` in `help_data.rs` and included in
  `help_entries_all()`. Malformed JSON must panic at first access per D-25.17.
- [ ] **`docs-matrix` 5th invocation:** Verify `justfile` and `scripts/docs-matrix/src/main.rs`
  have a 5th branch for `hp41-advantage-functions.json` → `hp41-advantage-function-matrix.md`.

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| math1/ freeze violation discovered post-implementation | HIGH — all new code must be moved to `advantage/` | Create `advantage/complex.rs`, move functions, fix imports, re-run CI. The move is mechanical but touching the frozen file requires a new ADR regardless. |
| Wrong PROOT algorithm produces wrong answers for high degree | MEDIUM — algorithm swap, not architecture change | Replace Bairstow extension with Laguerre in `advantage/poly.rs`. Existing Op variants and CalcState fields unchanged. Only the internal algorithm and test oracles change. |
| INTG/INTEG mnemonic collision | LOW — rename in `advantage/xrom.rs` | Change `"INTG"` to `"INTEG"` in `ADVANTAGE.ops`. Update JSON, `op_display_name`, help text. `xrom_shadowing.rs` failure would have caught this before it ships. |
| xrom_modules type wrong | MEDIUM — type change and migration | Widen to `u16`, update `default_xrom_modules()` return type, update `migrate_after_load()` migration arm. Backward-compat test confirms v3.2 save files still load. |
| Missing Free42 contamination scan on new directory | LOW | Edit `check-free42-contamination.sh` to add new directory. Run `just license-audit`. |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| P43: math1/ freeze violation | Core implementation phase (Phase A) | Diff shows no changes to `math1/` except `modal.rs` and `xrom.rs` |
| P44: PROOT algorithm | Core implementation phase (Phase A) | Oracle test cases degree 6, 10, repeated roots all pass |
| P45: Complex stack sharing | Core implementation phase (Phase A) | `advantage/complex.rs` imports from `math1::complex`, not copy |
| P46: XROM ID collision — two slots | Core implementation phase (Phase A), first task | `xrom_modules` type widened; `migrate_after_load()` updated |
| P47: Matrix register layout | Core implementation phase (Phase A) | Owner's Manual verified; `matrix_dim` ownership documented |
| P48: INTG vs INTEG mnemonic | Core implementation phase (Phase A) | `xrom_shadowing.rs` passes with `ADVANTAGE.ops` included |
| P49: CalcState field ownership | Core implementation phase (Phase A) | Code comment on `matrix_dim` specifying ownership |
| P50: xrom_modules bit width | Core implementation phase (Phase A), first task | Backward-compat test loads v3.2 fixture and finds Advantage Pac reachable |
| P51: Free42 contamination | Core implementation phase (Phase A), same commit as directory creation | `just license-audit` passes on `advantage/` directory |

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Core — XROM registration | P46 (bitfield width) + P48 (mnemonic collision) | Widen `xrom_modules`, verify mnemonics in hardware docs FIRST |
| Core — PROOT implementation | P44 (wrong algorithm for high degree) | Choose Laguerre/Durand-Kerner; derive oracle cases BEFORE coding |
| Core — complex ops | P43 (freeze violation) + P45 (shared helpers visibility) | New `advantage/` sibling; promote `complex_atan2` to `pub(crate)` |
| Core — matrix ops | P47 (register layout) + P49 (field ownership) | Verify OM; document field ownership explicitly |
| CLI phase | P48 (display name for INTEG vs INTG) | `op_display_name` must not reuse Math Pac I arms |
| Documentation phase | P51 (contamination guard) | Extend `check-free42-contamination.sh`; new divergence catalog |
| Quality gates phase | P44 (PROOT accuracy) | All degree ranges tested; numerical accuracy >= 98% on PROOT oracle suite |

---

## Sources

- HP-41C codebase: `hp41-core/src/ops/math1/` (freeze boundary reference)
- `hp41-core/src/state.rs` (CalcState field inventory, `xrom_modules: u8` type)
- `hp41-core/src/ops/math1/xrom.rs` (MATH_1, STAT_1, TIME_MODULE registry pattern)
- `hp41-core/src/ops/math1/complex.rs` (complex stack overlay model, `complex_atan2` visibility)
- `hp41-core/src/ops/math1/poly.rs` (Bairstow deflation, degree 2-5 only)
- `hp41-core/src/ops/math1/matrix.rs` (R14/R15+ register layout, `matrix_dim` field)
- `docs/architecture-history.md` (freeze rationale, phase-by-phase decisions)
- `docs/adr/v3.1-004-math1-freeze-second-carve-out.md` (precedent for modal.rs carve-out)
- `docs/adr/v3.0-002-user-callback-policy.md` (re-entrancy constraints for INTG/SOLVE)
- HP Museum XROM database: `calc.fjk.ch/db/hp41mod.php` (Advantage Pac XROM 22 + 24 confirmed)
- HP Museum community (MEDIUM confidence): Advantage XROM 22 functions include MAT*, MDET, MINV, etc.;
  XROM 24 includes INTEG, SOLVE (not INTG/SOLVE), complex functions C+, Z^N, etc.
- Community reports (LOW confidence): Advantage Pac PROOT handles "degree limited only by RAM"
  (implies non-Bairstow algorithm); FROOT and FINTG are alternative names for the root/integration
  functions in some sources — verify against OM 00041-90482.
- `docs/hp41-math1-divergences.md` (scratch register clobber during INTG — same risk applies to
  Advantage Pac INTEG if it uses R00-R07 scratch)
- ADR v3.1-002 (distribution primitives policy — algorithm independence from Free42 template
  for Advantage Pac Romberg implementation)

---
*Pitfalls research for: HP-41 Advantage Pac + Advanced Matrix Pac emulation (v3.3)*
*Researched: 2026-05-25*
