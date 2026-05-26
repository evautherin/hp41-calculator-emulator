---
gsd_state_version: 1.0
milestone: v3.3
milestone_name: Advantage Pac Emulation
status: executing
last_updated: "2026-05-26T12:28:10.321Z"
last_activity: 2026-05-26 -- Phase 45 execution started
progress:
  total_phases: 5
  completed_phases: 2
  total_plans: 14
  completed_plans: 12
  percent: 40
---

# Project State: HP-41 Calculator Emulator

---

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-25 after v3.3 roadmap)

**Core value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.

**Current focus:** Phase 45 — documentation-adrs

---

## Current Position

Phase: 45 (documentation-adrs) — EXECUTING
Plan: 1 of 2
Status: Executing Phase 45
Last activity: 2026-05-26 -- Phase 45 execution started

Progress: [████░░░░░░] 40%  (2/5 phases)

---

## Performance Metrics (v3.2 ship baseline)

| Metric | Target | Last measured (v3.2) |
|--------|--------|----------------------|
| Cold-start latency | <= 0.5 s | 2.2 ms (M1) |
| Key-press latency | <= 50 ms | ~65 ns/op |
| `hp41-core` line coverage | >= 95 % | 93.72 % (denominator dilution) |
| `hp41-core` region coverage | >= 93 % | **95.63 %** |
| Numerical accuracy | >= 98 % | 98.86 % (791 cases) |
| Panics in `hp41-core` | 0 | 0 |
| Free42 contamination | 0 | 0 (18-token guard) |
| CI platforms | Win/macOS/Ubuntu | All green |

---

## Accumulated Context

### Decisions (pre-resolved from research)

- XROM IDs: ADV_MATH_A = 22, ADV_MATH_B = 24; bit-3 and bit-4 arms in `xrom_resolve`
- `default_xrom_modules` migrates 0b00111 → 0b11111; `migrate_after_load()` in `state.rs`
- Named-matrix storage: `adv_matrices: Vec<AdvMatrix>` with `#[serde(default)]` — NOT R14/R15+ (Math Pac I incompatibility documented per ADV-DOC-02)
- FROOT uses Laguerre's method (arbitrary degree); coexists with Math Pac I Bairstow (degree 2-5)
- FINTG uses Romberg integration; coexists with Math Pac I Simpson
- Zero new runtime dependencies (ADR-v3.1-002 invariant maintained)
- All Advantage Pac code in `ops/advantage/`; math1/ freeze: only visibility promotions (`complex_atan2` pub(crate), `USER_CALLBACK_MAX_STEPS` pub(crate) if needed)

### Blockers

None.

### Pending Todos

None — all 7 research open questions resolved during Phase 43 execution:

- FROOT: Laguerre's method with quadratic deflation (degree from X, coefficients in R01..R(n+1))
- ADVMTRX: 50 ops across ADV_MATH_A (element access, lifecycle, reductions, linalg, complex matrix)
- CATALOG 2: ADV_MATH_A name "ADV 22A", ADV_MATH_B name "ADV 24B"
- NOT/AND/OR/XOR: 36-bit fixed word size (ADV_WORD_MASK = 0x0000_000F_FFFF_FFFF)
- Complex stack: X+iY convention, delegates to Math Pac I where possible (pub(crate) promotions)
- FSOLVE/FINTG nesting: separate state fields (adv_fsolve_state / adv_fintg_state), D-43.7 cross-nesting
- TVM: Option<TvmState> with #[serde(default)] WITHOUT skip (D-43.11 persistence)

---

## Deferred Items

All quick_tasks from v3.2 milestone close verified as completed (2026-05-26 consistency check):

| Category | Item | Status | Evidence |
|----------|------|--------|----------|
| uat_gap | Phase 41: visual verification scenarios (3 items) | done | covered by v3.2 Phase 41/42 |
| quick_task | 260506-a1g-add-gitignore | done | .gitignore present; multiple gitignore commits |
| quick_task | 260508-06h-fix-sci-eng-digit-input | done | `019009e` fix(format): Mantissa-Carry-Bug |
| quick_task | 260508-y30-eex-chs-exponent-sign-toggle | done | `9cf2104` feat(15-02): eex_chs branch |
| quick_task | 260516-c1p-fix-gui-clp-binding-and-modal-letter-clicks | done | `96c46b8` + `21895da` CLP/LBL + modal letter |
| quick_task | 260522-g7s-add-yellow-keyboard-frame-matching-vorgabe | done | `509344a` + `6094e32` gold trim SVG |
| quick_task | 260522-gud-honor-shift-on-physical-keyboard | done | `aa7e614` fix(gui): honor shiftActive |

---

*State initialized: 2026-05-06*
*Last updated: 2026-05-26 — Phase 44 complete (421 CLI tests, 114-entry JSON pipeline, 5-pool help chain); Phase 45 ready*
