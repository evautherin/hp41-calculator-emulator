---
gsd_state_version: 1.0
milestone: v4.0
milestone_name: Platform Maturity
status: planning
last_updated: "2026-05-27T09:15:25.257Z"
last_activity: 2026-05-27
progress:
  total_phases: 0
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State: HP-41 Calculator Emulator

---

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-27 after v3.3 shipped)

**Core value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.

**Current focus:** None — v3.3 milestone complete. Run `/gsd-new-milestone` to start the next milestone.

---

## Current Position

Phase: Not started (defining requirements)
Plan: —
Status: Defining requirements
Last activity: 2026-05-27 — Milestone v4.0 started

## Performance Metrics (v3.3 ship baseline)

| Metric | Target | Last measured (v3.3) |
|--------|--------|----------------------|
| Cold-start latency | <= 0.5 s | 2.2 ms (M1) |
| Key-press latency | <= 50 ms | ~65 ns/op |
| `hp41-core` line coverage | >= 95 % | ~93 % (denominator dilution from ~32K new LOC) |
| `hp41-core` region coverage | >= 93 % | ~95 % |
| Numerical accuracy | >= 98 % | 98.86 % (791+30+22 = 843 cases) |
| Panics in `hp41-core` | 0 | 0 |
| Free42 contamination | 0 | 0 (18-token guard) |
| CI platforms | Win/macOS/Ubuntu | All green |
| Tests passing | — | 3262 (up from 3161 at v3.2) |

---

## Accumulated Context

### Decisions (pre-resolved from research)

- XROM IDs: ADV_MATH_A = 22, ADV_MATH_B = 24; bit-3 and bit-4 arms in `xrom_resolve`
- `default_xrom_modules` migrates 0b00111 → 0b11111; `migrate_after_load()` in `state.rs`
- Named-matrix storage: `adv_matrices: Vec<AdvMatrix>` with `#[serde(default)]` — NOT R14/R15+ (Math Pac I incompatibility documented per ADV-DOC-02)
- FROOT uses Laguerre's method (arbitrary degree); coexists with Math Pac I Bairstow (degree 2-5)
- FINTG uses Romberg integration; coexists with Math Pac I Simpson
- Zero new runtime dependencies (ADR-v3.1-002 invariant maintained)
- All Advantage Pac code in `ops/advantage/`; math1/ freeze: only visibility promotions (`complex_atan2` pub(crate))

### Blockers

None.

### Pending Todos

None — all v3.3 work complete.

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
*Last updated: 2026-05-26 — v3.3 Advantage Pac Emulation shipped (5 phases, 18 plans, 47 total project phases)*
