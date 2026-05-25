---
gsd_state_version: 1.0
milestone: v3.3
milestone_name: Advantage Pac Emulation
status: planning
last_updated: "2026-05-25T17:29:03.966Z"
last_activity: 2026-05-25 — v3.3 roadmap created; ready to begin Phase 43
progress:
  total_phases: 5
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State: HP-41 Calculator Emulator

---

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-25 after v3.3 roadmap)

**Core value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.

**Current focus:** Phase 43 — hp41-core XROM Framework + All Advantage Pac Ops

---

## Current Position

Phase: 43 of 47 (hp41-core — XROM Framework + All Advantage Pac Ops)
Plan: — (not yet planned)
Status: Ready to plan
Last activity: 2026-05-25 — v3.3 roadmap created; ready to begin Phase 43

Progress: [░░░░░░░░░░] 0%  (0/5 phases)

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

- Resolve 7 open questions from research (Phase 43 pre-work): FROOT calling convention, complete ADVMTRX 52-op sub-numbers, CATALOG 2 display strings, NOT/AND/OR/XOR word size, complex stack convention, FSOLVE/FINTG nesting architecture, TVM register persistence model

---

## Deferred Items

Carried forward from v3.2 milestone close (2026-05-25):

| Category | Item | Status |
|----------|------|--------|
| uat_gap | Phase 41: visual verification scenarios (3 items) | partial |
| quick_task | 260506-a1g-add-gitignore | missing |
| quick_task | 260508-06h-fix-sci-eng-digit-input | missing |
| quick_task | 260508-y30-eex-chs-exponent-sign-toggle | missing |
| quick_task | 260516-c1p-fix-gui-clp-binding-and-modal-letter-clicks | missing |
| quick_task | 260522-g7s-add-yellow-keyboard-frame-matching-vorgabe | missing |
| quick_task | 260522-gud-honor-shift-on-physical-keyboard | missing |

---

*State initialized: 2026-05-06*
*Last updated: 2026-05-25 — v3.3 roadmap created; Phase 43 ready to plan*
