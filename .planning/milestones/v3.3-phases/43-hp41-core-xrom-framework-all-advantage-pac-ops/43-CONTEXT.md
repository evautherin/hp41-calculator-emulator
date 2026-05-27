# Phase 43: hp41-core — XROM Framework + All Advantage Pac Ops - Context

**Gathered:** 2026-05-25
**Status:** Ready for planning

<domain>
## Phase Boundary

All ~117 Advantage Pac operations implemented in `hp41-core/src/ops/advantage/`, with XROM 22 (ADV CONV + ADV MTRX) and XROM 24 (ADV MATH + ADV TVM) registered as XromModule constants, named-matrix storage on CalcState, `ModalProgram::Advantage(AdvantageStep)` variant wired, and all new Op variants compiling in `dispatch()` + `execute_op()` (4-way invariant items 1+2 satisfied; items 3+4 deferred to Phase 44/46).

**What this phase delivers:**

1. **XROM registration (ADV-FW-01..03):** Two new `XromModule` constants (ADV_MATH_A id=22, ADV_MATH_B id=24); `xrom_resolve` gains bit-3 and bit-4 arms; `default_xrom_modules()` updated from `0b0000_0111` to `0b0001_1111`; `migrate_after_load()` auto-upgrades v3.2 save files.

2. **Named-matrix storage (ADV-FW-04):** `adv_matrices: Vec<AdvMatrix>` on CalcState with `#[serde(default)]` — X-MEM named-matrix model incompatible with Math Pac I's R14/R15+ register layout.

3. **Modal variant (ADV-FW-05):** `ModalProgram::Advantage(AdvantageStep)` per ADR-v3.1-005 pattern; dispatch wired in `math1/modal.rs` freeze carve-out (fourth additive variant).

4. **ADV CONV (12 ops, ADV-CONV-01..12):** Base conversion (BININ/BINVIEW/OCTIN/HEXIN/HEXVIEW/CVTVIEW) + bitwise logic (NOT/AND/OR/XOR/ROTXY/BIT?) with 36-bit fixed word size.

5. **ADV MTRX (~50 ops, ADV-MTX-01..50):** Element access with I/J indexing, matrix lifecycle (MATDIM/DIM?/MNAME?), high-level operations (MDET/MINV/MSYS/M*M/MAT+/MAT-/TRNPS), reductions/norms, complex matrix ops, simplified matrix editors.

6. **ADV MATH (~47 ops, ADV-MATH-01..47):** Matrix workflow frontends (MATRX/MTR), FSOLVE/FINTG/FDIFEQ/FROOT solvers with user-program callback re-entrancy, complex number extensions, curve fitting, vector operations, coordinate transforms.

7. **ADV TVM (6 ops, ADV-TVM-01..06):** Time Value of Money solver (TVM/N/PV/PMT/FV/*I) with persistent state and Newton iteration for *I.

**In scope:**
- `hp41-core/src/ops/advantage/` — all Advantage Pac implementation files
- `hp41-core/src/ops/math1/xrom.rs` — bit-3/bit-4 arms + ADV_MATH_A/ADV_MATH_B constants (freeze carve-out)
- `hp41-core/src/ops/math1/modal.rs` — `ModalProgram::Advantage(AdvantageStep)` variant (freeze carve-out)
- `hp41-core/src/ops/math1/complex.rs` — `complex_atan2` visibility promotion `pub(super)` → `pub(crate)` (freeze carve-out)
- `hp41-core/src/state.rs` — `adv_matrices`, `adv_tvm_state`, `default_xrom_modules()`, `migrate_after_load()`
- `hp41-core/src/ops/mod.rs` — `Op` enum + `dispatch()` (~117 new variants)
- `hp41-core/src/ops/program.rs` — `execute_op()` (~117 new arms)
- `scripts/check-free42-contamination.sh` — extend to `advantage/` directory

**Out of scope (explicit):**
- `hp41-cli/src/` changes (Phase 44)
- `hp41-gui/` changes (Phase 46)
- Documentation / ADRs (Phase 45)
- Test hardening / quality gates (Phase 47)
- Full X-MEM Extended Memory model (EMDIR/EMROOM/EMREG — post-v3.3)
- Advanced Matrix Pac (XROM 12 — Angel Martin community ROM, NOT official HP)
- FROOT/FINTG mutual nesting beyond one level (NEST-01 — post-v3.3 for deeper nesting)

</domain>

<decisions>
## Implementation Decisions

### Named-Matrix Model
- **D-43.1:** **Unlimited matrix count.** `adv_matrices: Vec<AdvMatrix>` grows without artificial cap. No maximum number of named matrices. Document as emulator extension in divergences (hardware had X-MEM file slot limits).
- **D-43.2:** **Maximum matrix size is Claude's discretion.** Determine from OM 00041-90482 constraints and practical memory considerations. Document the chosen cap as behavioral policy in `docs/hp41-advantage-divergences.md`.
- **D-43.3:** **Current-matrix selection mechanism is Claude's discretion.** Determine from OM 00041-90482 conventions whether ALPHA register names the current matrix or a dedicated field tracks it.
- **D-43.4:** **I/J index storage location is Claude's discretion.** Determine from OM 00041-90482 behavior whether indices are per-matrix or global. The choice affects MR/MS/I+/J+ semantics.
- **D-43.5:** **Named-matrix storage MUST NOT touch `state.matrix_dim` or `state.matrix_active_reg`.** These belong to Math Pac I's R14/R15+ register-based matrix model. Complete isolation between the two matrix systems.

### FROOT/FINTG/FSOLVE Callback Architecture
- **D-43.6:** **Callback mechanism is Claude's discretion.** Whether to reuse the existing `run_loop` re-entrancy infrastructure (with new transient CalcState fields like `adv_integ_state`/`adv_solve_state`/`froot_state`) or build a separate mechanism — determined by the math1/ freeze constraint and code cleanliness.
- **D-43.7:** **One level of solver nesting is REQUIRED.** FINTG inside FSOLVE's callback (and vice versa) must work. This is the most common real-world use case. Deeper mutual nesting (NEST-01) is deferred to post-v3.3.
- **D-43.8:** **FROOT calling convention is Claude's discretion.** Whether polynomial degree comes from X register or a modal prompt — determine from OM 00041-90482 Section 3 conventions.

### ADV CONV Word Size & Overflow
- **D-43.9:** **36-bit fixed word size** for NOT/AND/OR/XOR/ROTXY/BIT?. Matches HP-41's 10-digit BCD mantissa capacity per OM 00041-90482 Section 1.
- **D-43.10:** **Silent truncation (mask to 36 bits)** on overflow. Results are masked to lower 36 bits with no error. Consistent with hardware behavior.

### TVM Register Model
- **D-43.11:** **TVM state is persistent** (`#[serde(default)]`). N/I/PV/PMT/FV survive save/load. Matches financial calculator UX expectations and OM 00041-90482 Section 4 behavior.
- **D-43.12:** **BEGIN/END payment mode is Claude's discretion.** Determine from OM 00041-90482 TVM section whether annuity-due mode is specified.
- **D-43.13:** **\*I non-convergence handling is Claude's discretion.** Determine from OM behavior and existing solver patterns (SOLVE/INTG error handling precedent).

### Claude's Discretion (summary)
The following decisions are explicitly delegated to Claude based on OM 00041-90482 research:
- D-43.2: Maximum matrix size cap
- D-43.3: Current-matrix selection mechanism (ALPHA vs dedicated field)
- D-43.4: I/J index storage (per-matrix vs global)
- D-43.6: Callback mechanism architecture (reuse run_loop vs separate)
- D-43.8: FROOT calling convention (X register vs modal prompt)
- D-43.12: BEGIN/END payment mode support
- D-43.13: *I non-convergence behavior

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### XROM Framework Patterns (prior art)
- `hp41-core/src/ops/math1/xrom.rs` — XROM registration pattern (`XromModule` struct, `xrom_resolve()` resolver chain, `math1_resolve`/`stat1_resolve`/`time_resolve` per-module resolvers)
- `hp41-core/src/ops/math1/modal.rs` — `ModalProgram` enum + freeze carve-out pattern (Stat1, Time variants as precedent for Advantage)
- `hp41-core/src/state.rs` — `migrate_after_load()` XROM migration chain, `default_xrom_modules()`, all CalcState field serde patterns

### Prior Module Implementations (structural templates)
- `hp41-core/src/ops/stat1/` — Stat 1 Pac structure (closest organizational precedent: `mod.rs` constants + `modal.rs` step enum + per-feature files)
- `hp41-core/src/ops/time/` — Time Module structure (second structural template)
- `hp41-core/src/ops/math1/` — Math Pac I (frozen; reference for complex ops, SOLVE/INTG/DIFEQ callback patterns, BUT DO NOT MODIFY beyond sanctioned carve-outs)

### Solver Callback Infrastructure
- `hp41-core/src/ops/math1/integ.rs` — `IntegState` + Romberg integration pattern (INTG)
- `hp41-core/src/ops/math1/solve.rs` — `SolveState` + secant method pattern (SOLVE)
- `hp41-core/src/ops/math1/difeq.rs` — `DifeqState` + RK4 pattern (DIFEQ)
- `hp41-core/src/ops/program.rs` — `run_loop` re-entrancy, `execute_op()` exhaustive match

### Project Decisions & Architecture
- `docs/adr/v3.1-002-distribution-primitives-policy.md` — Zero-new-runtime-deps invariant (applies to v3.3)
- `docs/adr/v3.1-004-math1-freeze-second-carve-out.md` — math1/ freeze policy (sanctioned exception pattern)
- `docs/adr/v3.1-005-modalprogram-stat1-enum-extension.md` — `ModalProgram` additive-variant pattern
- `.planning/research/SUMMARY.md` — v3.3 research synthesis (open questions, build order, watch-outs)
- `scripts/check-free42-contamination.sh` — GPL contamination guard (extend to `advantage/`)

### Quality Infrastructure
- `hp41-core/tests/xrom_op_test_count.rs` — Unified meta-gate (extend to Advantage Pac variants)
- `hp41-core/tests/xrom_shadowing.rs` — XROM mnemonic disjointness CI gate (extend to ADV_MATH_A + ADV_MATH_B)
- `hp41-core/tests/function_matrix_parity.rs` — Op ↔ JSON bidirectional parity (Phase 44 extends to 5th pool)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `math1/xrom.rs::XromModule` struct: reuse for `ADV_MATH_A` and `ADV_MATH_B` constants
- `math1/xrom.rs::xrom_resolve()`: add bit-3/bit-4 arms following existing bit-0/1/2 pattern
- `math1/modal.rs::ModalProgram`: add `Advantage(AdvantageStep)` variant following `Stat1`/`Time` precedent
- `math1/complex.rs::complex_atan2`: promote to `pub(crate)` for reuse in `advantage/complex.rs`
- `state.rs::migrate_after_load()`: extend with bit-3/bit-4 migration following existing pattern
- `math1/integ.rs` + `solve.rs` + `difeq.rs`: callback re-entrancy pattern via `run_loop` + transient state fields

### Established Patterns
- **Free42 disclaim header**: every file in `advantage/` must carry the verbatim header (Pitfall 19)
- **`#[serde(default)]` on new CalcState fields**: persistent fields get `default` only; transient fields get `default, skip`
- **`#![deny(clippy::unwrap_used)]`**: no unwrap in production code; tests carry `#[allow]`
- **String-split-at-decimal for date parsing**: ISG/DSE precedent — never `floor()`/`fmod()` (extends to any decimal-encoded data)
- **OM register transcription constants**: named consts in `mod.rs` doc-comment header (stat1 pattern per ADR-v3.1-003)

### Integration Points
- `hp41-core/src/ops/mod.rs`: ~117 new `Op` enum variants + `dispatch()` arms
- `hp41-core/src/ops/program.rs`: ~117 new `execute_op()` arms + `xrom_resolve` path in `xeq_by_name`
- `hp41-core/src/state.rs`: `adv_matrices: Vec<AdvMatrix>` + `adv_tvm_state: Option<TvmState>` + migration
- `hp41-core/src/ops/math1/modal.rs`: fourth freeze carve-out variant
- `hp41-core/src/ops/math1/xrom.rs`: bit-3/bit-4 arms + two new `XromModule` constants + two new resolve functions

</code_context>

<specifics>
## Specific Ideas

- **One-level solver nesting**: FINTG inside FSOLVE (or vice versa) must work — this is a core use case for the Advantage Pac's mathematical power. Deeper nesting deferred.
- **36-bit word size + silent truncation**: user explicitly chose these for ADV CONV, matching OM Section 1.
- **Persistent TVM state**: user explicitly chose `#[serde(default)]` (not `skip`) so financial calculations survive across sessions.
- **Unlimited matrix count**: user explicitly chose unbounded `Vec<AdvMatrix>` — no artificial cap on simultaneous named matrices.

</specifics>

<deferred>
## Deferred Ideas

- **NEST-01: Deep FROOT/FINTG mutual nesting** — re-entrant X-MEM buffer stack for arbitrary nesting depth. Tracked in REQUIREMENTS.md `## Future Requirements`. One level of nesting is supported in v3.3; deeper nesting is post-v3.3.
- **XMEM-01: Full Extended Memory model** — EMDIR/EMROOM/EMREG etc. Only named-matrix storage is implemented for v3.3. Tracked in REQUIREMENTS.md `## Future Requirements`.

None from discussion — all topics stayed within phase scope.

</deferred>

---

*Phase: 43-hp41-core — XROM Framework + All Advantage Pac Ops*
*Context gathered: 2026-05-25*
