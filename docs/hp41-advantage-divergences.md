# HP-41C Advantage Pac Emulator Divergences

This document lists known behavioral divergences between this emulator's implementation
of the HP-41C Advantage Pac and the hardware-faithful behavior described in the
HP Advantage Pac Owner's Manual (HP 00041-90482, 1985).

**Status:** Established as comprehensive numbered catalog in Phase 45 / Plan 45-01 (ADV-DOC-02).

**Philosophy:** Where divergences exist, this emulator prioritizes:
1. Hardware-faithful behavior where feasible.
2. User-safety (no silent data corruption without documentation).
3. Clear documentation of known divergences.

---

## How to Use This Document

Each entry carries a stable `D-45-NN` identifier that can be used in cross-references
from source-code comments, ADRs, test files, and issue trackers. The ID encodes the
phase (45 = Phase 45 / ADV-DOC-02) and an ordinal sequence number within this document.

Every entry uses five fixed fields (D-30.5 shape, carried forward as D-45 template):

- **OM citation** — The HP 00041-90482 page-and-example that is the primary source, or
  `"N/A — emulator extension"` when no OM equivalent exists.
- **Our behavior** — What this emulator does.
- **OM behavior** — What the OM says or what real HP-41C Advantage Pac hardware does.
- **Rationale** — Why we made this choice (hardware-fidelity vs. UX trade-off decision).
- **See** — Cross-references: ADR links, CONTEXT.md decision IDs, test file pointers,
  Pitfall references from `research/PITFALLS.md` (carried forward across v3.x).

The citation discipline (Pitfall 18 from `research/PITFALLS.md`, carried forward across
v3.x) requires every entry to carry at least one OM page reference, an explicit
`"N/A — emulator extension"` marker, or a primary-source citation. No uncited assertions
are permitted in this document.

Advantage Pac is the fourth XROM application module in the v3.x line; entry numbering
follows the phase-origin convention established in v3.0 (`D-30-NN` for Math Pac I,
`D-35-NN` for Stat 1 Pac, `D-40-NN` for Time Pac) per the D-35.4 numbering scheme.
The Advantage Pac catalog uses `D-45-NN` identifiers tied to Phase 45 (Documentation
& ADRs).

---

## 1. OM Divergences

*(Numerical / behavioral mismatches with OM-quoted examples or OM-described hardware
behavior. These are cases where the OM specifies or implies a particular outcome and our
emulator either matches or intentionally diverges from that specification.)*

No OM numerical divergences identified in Phase 43-44 implementation. The Advantage Pac
implementation follows HP 00041-90482 (1985) behavioral specification for all callable
functions. No oracle drifts analogous to the Stat 1 Pac scipy-vs-SPEC reconciliations
(D-35-01..D-35-06) were identified during Phase 43/44 verification.

*If future Phase 46 (GUI integration) or Phase 47 (test hardening) surfaces a genuine
OM-quoted-behavior mismatch for Advantage Pac, it will be authored as `D-45-10:` or
later — the numbering reservation `D-45-01..D-45-09` is the canonical bucket-2/3 ID
range per this plan.*

---

## 2. Emulator Extensions

*(Functions or behaviors we added that are not present in HP 00041-90482 (1985). These
are deliberate, documented additions that improve usability without conflicting with OM
behavior for OM-specified inputs. Every extension in this section is marked with
"N/A — emulator extension" in the OM citation field.)*

---

### D-45-01: Unlimited Named-Matrix Count

- **OM citation**: `N/A — emulator extension`. HP 00041-90482 (1985) Section 2 (matrix
  operations) does not specify a maximum number of simultaneously-active named matrices.
  The hardware constraint was the X-MEM (Extended Memory) file slot capacity: each named
  matrix occupied one or more X-MEM file slots, so the practical maximum was bounded by
  available Extended Memory installed in the calculator (typically 64 registers per X-MEM
  module, up to 319 registers total with 5 modules).

- **Our behavior**: `adv_matrices: Vec<AdvMatrix>` grows without artificial cap. Any
  number of named matrices can coexist in the emulator state. There is no maximum count
  enforced at runtime. Each matrix is identified by its name string (matched against the
  ALPHA register at operation call time).

- **OM behavior**: HP-41C hardware with Advantage Pac was limited by physical X-MEM file
  slot capacity. The number of simultaneously-active named matrices depended on installed
  Extended Memory and the sizes of the individual matrices. The hardware never explicitly
  stated a maximum named-matrix count — the limit was an emergent hardware resource
  constraint, not a specified design parameter.

- **Rationale**: The emulator's `Vec`-backed storage has no practical memory constraint
  analogous to X-MEM slots. Imposing an artificial cap would reduce usability without
  emulating a meaningful hardware limit (the OM does not specify a fixed maximum). The
  Vec model supports the OM's intent — unlimited named-matrix coexistence within available
  memory — while using modern host memory instead of X-MEM slots. Rejected alternative:
  cap at 32 matrices (approximate X-MEM worst-case) — rejected because the cap is not
  OM-specified and would break programs that rely on large numbers of small matrices on
  modern hardware with abundant memory.

- **See**: D-43.1 (43-CONTEXT.md — "Unlimited matrix count; `adv_matrices: Vec<AdvMatrix>`
  grows without artificial cap"); ADR-v3.3-001;
  `hp41-core/src/state.rs` (`adv_matrices: Vec<AdvMatrix>` field with `#[serde(default)]`).

---

### D-45-02: 255×255 Per-Dimension Matrix Size Cap

- **OM citation**: `N/A — emulator extension`. HP 00041-90482 (1985) Section 2 does not
  specify a fixed row or column cap for named matrices. The hardware limit on individual
  matrix dimensions came from available X-MEM file slots: a 10×10 matrix required
  100 registers of X-MEM storage plus file-slot overhead, constraining large matrices in
  practice.

- **Our behavior**: `ADV_MATRIX_MAX_ROWS: u8 = 255` and `ADV_MATRIX_MAX_COLS: u8 = 255`
  — the `u8` storage type for matrix dimension fields enforces a 255-element maximum per
  dimension. MATDIM with dimensions exceeding 255 returns `HpError::InvalidData`.
  Individual matrix elements are stored as `Vec<HpNum>` in row-major order; a 255×255
  matrix occupies 65,025 HpNum values in host memory.

- **OM behavior**: Hardware X-MEM slot limits constrained practical matrix sizes (a 14×14
  matrix — 196 registers plus overhead — was near the limit of a typical Extended Memory
  configuration). The OM does not specify a fixed row/column cap as a software parameter;
  the physical limit was a hardware resource constraint.

- **Rationale**: The `u8` type provides a natural upper bound (255) without requiring an
  explicit runtime check — the type system enforces the limit at compile time. The OM does
  not specify a fixed row/column cap; the hardware limit came from available X-MEM file
  slots which do not apply to the emulator's `Vec<AdvMatrix>` model. 255×255 is a
  generous cap relative to real HP-41C hardware practice (most programs used 2×2 to 14×14
  matrices) while remaining trivially representable in host memory. Rejected alternative:
  14×14 cap matching Math Pac I (XROM 7) — rejected because the Advantage Pac OM 00041-90482
  does not specify 14×14 as a limit, and the `u8` type naturally provides a higher and
  equally defensible bound.

- **See**: D-43.2 (43-CONTEXT.md, Claude's Discretion — resolved to u8 type during
  Phase 43 implementation); ADR-v3.3-001;
  `hp41-core/src/ops/advantage/mod.rs` (`ADV_MATRIX_MAX_ROWS`, `ADV_MATRIX_MAX_COLS`
  constants).

---

## 3. Behavioral Policies

*(Cross-cutting rules that are decisions worth documenting — not strictly numerical
divergences, but intentional implementation choices with OM basis or deliberate extension.
These entries document cases where the emulator made a specific policy decision that
affects behavior in ways the OM either specifies explicitly or leaves to the implementation.)*

---

### D-45-03: Named-Matrix vs. Math Pac I Register Model Isolation

- **OM citation**: HP 00041-90482 (1985) Section 2 (matrix operations) — the Advantage
  Pac addresses matrices by name (via the ALPHA register) and stores them in X-MEM file
  slots. HP 00041-90034 (Math Pac I, 1979) addresses matrices by numbered storage
  registers (R14 contains the matrix order; elements start at R15+). The two modules
  used fundamentally different addressing mechanisms on hardware.

- **Our behavior**: `adv_matrices: Vec<AdvMatrix>` (Advantage Pac named-matrix storage)
  is completely isolated from `state.matrix_dim` / `state.matrix_active_reg` (Math Pac I
  R14/R15+ register model). Zero code references to `matrix_dim` or `matrix_active_reg`
  exist in `advantage/` executable code. Named matrices are indexed by ALPHA register
  string at call time; Math Pac I matrices are indexed by register number. The two systems
  cannot share data or inadvertently overwrite each other.

- **OM behavior**: On HP-41C hardware, the Advantage Pac (XROM 22/24) and Math Pac I
  (XROM 7) used XEQ-by-name dispatch to reach their respective entry points. Each module
  maintained its own data conventions — the Advantage Pac used X-MEM file slots for named
  matrices; Math Pac I used numbered storage registers. On hardware, the two matrix systems
  were naturally isolated by their different addressing mechanisms; there was no hardware
  mechanism that could cause one module to corrupt the other's matrix data.

- **Rationale**: Complete isolation prevents silent data corruption between the two matrix
  systems. D-43.5 (43-CONTEXT.md) mandates this: "Named-matrix storage MUST NOT touch
  `state.matrix_dim` or `state.matrix_active_reg`. Complete isolation between the two
  matrix systems." The implementation makes the isolation explicit at the data model level
  rather than relying on callers to avoid cross-system access. Verified by grep in Phase
  43 Plan 10: zero code references to `matrix_dim`/`matrix_active_reg` in `advantage/`
  executable code.

- **See**: D-43.5 (43-CONTEXT.md — isolation mandate); ADR-v3.3-001;
  `hp41-core/src/ops/advantage/matrix.rs` (named-matrix operations);
  `hp41-core/src/state.rs` (`adv_matrices` vs. `matrix_dim`/`matrix_active_reg` fields).

---

### D-45-04: FROOT Laguerre vs. Math Pac I Bairstow Coexistence

- **OM citation**: HP 00041-90482 (1985) Section 3 (FROOT) — the Advantage Pac `FROOT`
  finds all roots of a polynomial of arbitrary degree. The degree is read from register X;
  coefficients are stored in R01..R(n+1). HP 00041-90034 (Math Pac I) §ROOTS — the Math
  Pac I `ROOTS` finds roots of a polynomial of degree 2–5 using Bairstow's method with
  the `DEGREE=?` modal prompt and coefficients in R01..R(n+1).

- **Our behavior**: FROOT uses Laguerre's method with initial guess (0.4, 0.9) to avoid
  convergence failure at the origin (x²+1=0 degenerates if G=0), quadratic deflation for
  complex conjugate pairs, linear deflation for real roots, and root polishing against the
  original polynomial. All intermediate arithmetic is in f64 for numerical stability;
  results are converted to HpNum at final root output. Degree is read from register X
  (not a modal prompt). Both FROOT (Advantage Pac, XROM 24) and ROOTS (Math Pac I, XROM 7)
  coexist — different mnemonics, different register conventions, different algorithms,
  different degree ranges. Arbitrary degree (1–100) for FROOT vs. degree 2–5 for ROOTS.

- **OM behavior**: The Advantage Pac FROOT and Math Pac I ROOTS were independent programs
  on hardware — physically separate ROM chips with distinct XROM IDs. On hardware they
  never conflicted; XEQ "FROOT" dispatched to XROM 24 and XEQ "ROOTS" dispatched to
  XROM 7 via the calculator's built-in XEQ-by-name dispatch mechanism.

- **Rationale**: Two different polynomial root-finders with different capabilities and
  calling conventions that coexist without conflict because they use different Op variants
  and different register layouts. The only user-visible difference is that ROOTS requires a
  `DEGREE=?` modal prompt (Math Pac I style) while FROOT reads degree directly from X
  (Advantage Pac convention per D-43.8 resolution). MATH_1 wins mnemonic priority when
  both XROM modules are loaded (`xrom_modules = 0b0001_1111`) — this is hardware-faithful
  behavior per ADR-v3.3-003.

- **See**: D-43.8 (43-CONTEXT.md, Claude's Discretion — FROOT reads degree from X
  register); ADR-v3.3-002;
  `hp41-core/src/ops/advantage/froot.rs` (Laguerre implementation);
  `hp41-core/src/ops/math1/poly.rs` (Bairstow implementation — frozen).

---

### D-45-05: TVM Register Persistence

- **OM citation**: HP 00041-90482 (1985) Section 4 (TVM — Time Value of Money) — the
  Advantage Pac TVM operations (TVM/N/PV/PMT/FV/*I) read and write named register values.
  On HP-41C hardware the TVM programs stored intermediate values in numbered storage
  registers (R00–R05), which were persistent on the hardware's continuous memory (the
  HP-41C used CMOS RAM with battery backup that preserved all storage registers across
  power cycles).

- **Our behavior**: `adv_tvm_state: Option<TvmState>` is persistent — `#[serde(default)]`
  WITHOUT `#[serde(skip)]`. TVM register values (N, I, PV, PMT, FV, BEGIN/END payment
  mode) survive save/load cycles. `None` state (no TVM values set) is the default; once
  any TVM op is called, `TvmState` is populated and persists across sessions. The
  `adv_tvm_state` field appears in the JSON save file and is restored on load.

- **OM behavior**: On HP-41C hardware, the TVM programs stored values in numbered storage
  registers that always persisted (hardware CMOS RAM). The persistence guarantee was
  absolute on real hardware — there was no mechanism by which storage registers could lose
  their values between sessions (absent battery failure or a deliberate CLREG call).

- **Rationale**: Same persistence guarantee as hardware, implemented via a dedicated
  `TvmState` struct instead of numbered registers. The unique serde shape (`#[serde(default)]`
  WITHOUT `#[serde(skip)]`) follows the `rand_seed` precedent (ADR-v3.1-001 / D-33.4a)
  to avoid the Pitfall 20 muscle-memory trap (every other transient CalcState field uses
  `skip`, making `adv_tvm_state` the deliberate documented exception). The field is at
  `state.rs` lines 376–382 with an explicit warning comment about the non-standard serde
  shape.

- **See**: D-43.11 (43-CONTEXT.md — "TVM state is persistent (`#[serde(default)]`, not
  `skip`)"); ADR-v3.1-001 (rand_seed precedent for non-skip persistent field);
  `hp41-core/src/state.rs` (lines 376–382, `adv_tvm_state` field with serde shape
  warning comment).

---

### D-45-06: `adv_current_matrix` Transient — Clears on Save

- **OM citation**: HP 00041-90482 (1985) Section 2 — matrix operations are invoked via
  XEQ-by-name (e.g., `XEQ "MSTORE"`) with the ALPHA register specifying the target matrix
  name. The HP-41C hardware had no concept of a "current matrix" field — the active matrix
  was always determined by the ALPHA register contents at the time of the XEQ call. There
  was no persistent "current matrix" pointer on hardware.

- **Our behavior**: `adv_current_matrix: Option<String>` is transient (`#[serde(default,
  skip)]`). The active matrix name clears to `None` on save/load. On reload, users
  re-select the active matrix by loading the matrix name into the ALPHA register before
  calling matrix operations. The field exists to cache the last-used matrix name within
  a session for operations that do not re-specify it via ALPHA each call.

- **OM behavior**: On HP-41C hardware, the current matrix was always determined by the
  ALPHA register contents at the time of each XEQ call. There was no hardware "current
  matrix" state that persisted between calls — every matrix operation consumed the ALPHA
  register as its addressing argument. Across power cycles, the ALPHA register itself
  was volatile (the HP-41 did not persist the ALPHA string across power-off cycles in
  normal operation).

- **Rationale**: Matches hardware semantics — the ALPHA register is volatile across power
  cycles, so the "current matrix" pointer (which caches ALPHA register contents) should
  also be transient. Persisting `adv_current_matrix` would create an inconsistency where
  the cached matrix name survived a save/load cycle but the ALPHA register (which it
  caches) did not. D-43.3 (43-CONTEXT.md, Claude's Discretion) resolved this to the
  transient `#[serde(skip)]` shape.

- **See**: D-43.3 (43-CONTEXT.md, Claude's Discretion — current-matrix selection via
  ALPHA register); ADR-v3.3-001;
  `hp41-core/src/state.rs` (`adv_current_matrix: Option<String>` field with
  `#[serde(default, skip)]`).

---

### D-45-07: ADV_WORD_MASK 36-Bit Truncation for Bitwise Operations

- **OM citation**: HP 00041-90482 (1985) Section 1 (Base Conversion) — the Advantage Pac
  provides base-conversion and bitwise logic operations (BININ/BINVIEW/OCTIN/HEXIN/
  HEXVIEW/CVTVIEW for base conversion; NOT/AND/OR/XOR/ROTXY/BIT? for bitwise operations).
  The OM describes operation on integer values in binary, octal, and hexadecimal
  representation. The HP-41C's 10-digit BCD mantissa provides exactly 9,999,999,999
  (approximately 2^33.2) as the maximum positive integer — which fits in 36 bits.

- **Our behavior**: NOT, AND, OR, XOR, ROTXY, and BIT? mask all operands and results to
  the lower 36 bits via `ADV_WORD_MASK = 0x0000_000F_FFFF_FFFF`. Overflow is silently
  truncated — no error or warning is generated when the mask discards high bits. The
  36-bit constraint matches the HP-41C's BCD mantissa capacity as described in
  HP 00041-90482 Section 1.

- **OM behavior**: The HP-41's 10-digit BCD mantissa provides exactly 36 bits of integer
  capacity for bitwise operations. Truncation to 36 bits was implicit in hardware — the
  BCD arithmetic engine could not represent integers larger than 9,999,999,999, so the
  36-bit limit was a natural hardware constraint rather than a deliberate design parameter.

- **Rationale**: Silent truncation matches hardware behavior per D-43.9 and D-43.10
  (43-CONTEXT.md). The `ADV_WORD_MASK` constant makes the truncation explicit and
  documented rather than implicitly relying on HpNum arithmetic overflow behavior.
  Rejected alternative: return an error on overflow — rejected because the OM does not
  describe an error condition for integer overflow in base-conversion operations, and
  real hardware silently truncated via BCD arithmetic limits.

- **See**: D-43.9 (43-CONTEXT.md — "36-bit fixed word size for NOT/AND/OR/XOR/ROTXY/BIT?");
  D-43.10 (43-CONTEXT.md — "Silent truncation (mask to 36 bits) on overflow");
  `hp41-core/src/ops/advantage/conv.rs` (`ADV_WORD_MASK` constant and bitwise operation
  implementations).

---

### D-45-08: Solver Cross-Nesting Allowance

- **OM citation**: HP 00041-90482 (1985) Section 3 (FSOLVE, FINTG) — the Advantage Pac
  provides FSOLVE (numerical root-finding) and FINTG (numerical integration) as higher-
  order solvers that accept a user-program label as their function callback. The OM does
  not explicitly specify whether FINTG inside FSOLVE (or vice versa) is supported on
  hardware. The HP-41CX hardware constrained re-entrancy via its 4-level subroutine return
  stack.

- **Our behavior**: FINTG inside FSOLVE's callback (and vice versa) is allowed — each
  solver guards only its own state field (`adv_fsolve_state`, `adv_fintg_state`). One
  level of cross-nesting is supported. FSOLVE inside FSOLVE (self-nesting) returns
  `HpError::InvalidOp` — the self-nesting guard prevents corruption of an in-progress
  FSOLVE computation. Similarly, FINTG inside FINTG is blocked. Deeper mutual nesting
  (NEST-01) is deferred to post-v3.3.

- **OM behavior**: The OM does not explicitly specify whether solvers can be nested on
  hardware. The HP-41CX hardware's 4-level subroutine return stack was the practical
  constraint — each callback invocation consumed stack levels, limiting how deeply programs
  could nest. In practice, one level of cross-nesting (FINTG inside FSOLVE) was the
  most common real-world use case for the Advantage Pac's mathematical capabilities.

- **Rationale**: One level of cross-nesting is the most common real-world use case
  (D-43.7, 43-CONTEXT.md). Each solver's state field is independent, so cross-nesting
  requires no special infrastructure. Self-nesting would corrupt the in-progress computation
  because there is only one `adv_fsolve_state` field per `CalcState` — a nested FSOLVE
  call would overwrite the outer FSOLVE's iteration state. Deeper mutual nesting (three
  or more levels) is tracked as NEST-01 for post-v3.3 implementation.

- **See**: D-43.7 (43-CONTEXT.md — "One level of solver nesting is REQUIRED; FINTG inside
  FSOLVE must work");
  `hp41-core/src/ops/advantage/fsolve.rs` (self-nesting guard);
  `hp41-core/src/ops/advantage/finteg.rs` (cross-nesting with FSOLVE state);
  `.planning/phases/43-hp41-core-xrom-framework-all-advantage-pac-ops/43-CONTEXT.md`
  (NEST-01 deferred item).

---

### D-45-09: MATH_1 Alias Overlap in ADV_MATH_B (12 Mnemonics)

- **OM citation**: HP 00041-90482 (1985) Section 3 (Complex Extensions) — the Advantage
  Pac (XROM 24, ADV_MATH_B) includes complex number extension operations for the HP-41C.
  HP 00041-90034 (Math Pac I, XROM 7) independently defines complex number operations with
  identical mnemonics. This overlap is a hardware reality: the two ROM chips were developed
  independently and shipped the same function names.

- **Our behavior**: 12 ADV_MATH_B mnemonics in XROM 24 overlap with Math Pac I (XROM 7):
  E^Z, LNZ, LOGZ, Z^N, Z^1/N, Z^W, |Z|, SINZ, COSZ, TANZ, A^Z, CINV. When both modules
  are loaded (`xrom_modules = 0b0001_1111`), MATH_1 (bit-0) wins — the resolver fires
  bit-0 before bit-4. ADV_MATH_B's resolver (bit-4) is only reached when MATH_1 is
  unloaded (`xrom_modules = 0b0001_0000`). The `xrom_shadowing.rs` test uses
  `0b0001_0000` (bit-4 isolation) to verify ADV_MATH_B functions work independently.

- **OM behavior**: On HP-41C hardware, XROM 7 (Math Pac I) and XROM 24 (Advantage Pac
  chip 2) were separate ROM chips that could both be physically present. The XEQ-by-name
  dispatcher resolved to whichever module's entry-point name was found first, with the
  lower-numbered XROM ID winning ties. XROM 7 therefore won over XROM 24 for overlapping
  mnemonics when both chips were present — matching the emulator's bit-0-fires-before-bit-4
  resolver priority.

- **Rationale**: Hardware-faithful resolution order. MATH_1 wins the 12 overlapping
  mnemonics when both modules are loaded because XROM 7 has a lower ID than XROM 24, and
  the hardware XEQ-by-name dispatcher honored this ordering. The XROM shadowing test
  isolates ADV_MATH_B by loading only bit-4 (`xrom_modules = 0b0001_0000`) rather than
  using the full `0b0001_1111` bitmask — this is correct for testing ADV_MATH_B functions
  that overlap with MATH_1 (per Phase 44 discovery documented in 44-02-SUMMARY.md).

- **See**: ADR-v3.3-003;
  Phase 44 discovery (44-02-SUMMARY.md, Deviations section — intentional MATH_1 alias
  overlap in ADV_MATH_B confirmed during xrom_shadowing test);
  `hp41-core/src/ops/math1/xrom.rs` (resolver chain, bit-0 vs. bit-4 arm ordering);
  `hp41-core/tests/xrom_shadowing.rs` (bit-4 isolation test using `0b0001_0000`).

---

### D-65-01: Matrix Elements Assume Decimal Range (exponent-0) — MATH-01 Scope Boundary

- **Our behavior**: The Advantage matrix reduction/norm ops (`MAX`, `MIN`, `MAXAB`,
  `RMAXAB`, `RNRM` compare via `.inner()`; `FNRM` converts via `Decimal::from_f64`) and
  `MP` matrix-element printing (via the `HpNum` `Display` impl) assume matrix elements are
  within the `Decimal` range (`exponent == 0`, |value| ≲ 7.92E28). The MATH-01 range
  extension to ±9.999E±99 (ADR v4.3-005, Phase 65) applies to FACT and scalar `HpNum`
  arithmetic only — it deliberately does **not** extend Advantage matrix element storage.
  Large-exponent matrix elements are not supported by these ops in this milestone; in-range
  matrix data (the supported and tested case) works correctly.

- **Rationale**: Pre-existing unextended limit, not a regression. Extending the matrix
  reductions to exponent-aware comparison (`to_f64()`) and FNRM to the `HpNum::from_f64`
  post-compute wall is scoped out of Phase 65 (standalone-fidelity-fixes) to keep the
  phase focused on its four correctness fixes.

- **See**: ADR v4.3-005 (HpNum range extension); review findings WR-05 / IN-01 / IN-02;
  `hp41-core/src/ops/advantage/matrix_ops.rs` ("Reduction and norm operations" scope note
  + `op_adv_mp` doc).

---

*Last updated: 2026-06-07. Catalog established in Plan 45-01 (Phase 45 / ADV-DOC-02).*
