---
gsd_state_version: 1.0
milestone: v3.3
milestone_name: Advantage Pac Emulation
status: planning
last_updated: "2026-05-25T16:02:13.158Z"
last_activity: 2026-05-25
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

See: .planning/PROJECT.md (updated 2026-05-24 after v3.2 milestone start)

**Core value:** Faithful HP-41 RPN fidelity -- four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.

**Shipped milestones:**

- v1.0 CLI (2026-05-08) -- Phases 1-8; foundational RPN engine + TUI
- v1.1 CLI Feature Completeness (2026-05-09) -- Phases 9-12
- v2.0 Tauri GUI (2026-05-10) -- Phases 13-18
- v2.1 Card Reader + Keyboard Authenticity (2026-05-13) -- recorded as quick tasks
- v2.2 HP-41CV Feature Completeness (2026-05-15) -- Phases 20-27, 26/26 plans
- v3.0 Math Pac I Emulation (2026-05-20) -- Phases 28-32, 31/31 plans
- v3.1 Stat 1 Pac Emulation (2026-05-24) -- Phases 33-37, 23/23 plans

**Current focus:** Phase 42 — test-hardening-quality-gates
**Repo:** hp41-calculator-emulator
**Architecture:** Cargo workspace -- `hp41-core` (library) + `hp41-cli` (binary) + `hp41-gui` (nested standalone Tauri workspace); `hp41-core` has zero UI/CLI dependencies enforced at compile time.

---

## Current Position

Phase: Not started (defining requirements)
Plan: —
Status: Defining requirements
Last activity: 2026-05-25 — Milestone v3.3 started

## Performance Metrics (v3.1 ship)

| Metric | Target | Last measured (v3.1) |
|--------|--------|----------------------|
| Cold-start latency | <= 0.5 s | 2.2 ms (M1) -- 228x under gate |
| Key-press latency (median) | <= 50 ms | ~65 ns/op |
| `hp41-core` line coverage | >= 95 % | 93.91 % (denominator dilution from stat1 LOC) |
| `hp41-core` region coverage | >= 93 % | **95.84 %** |
| Numerical accuracy | >= 98 % (791 cases) | **98.86 %** |
| Panics in `hp41-core` | 0 | 0 -- enforced by `#![deny(clippy::unwrap_used)]` |
| Free42 contamination | 0 distinctive symbols | 0 (CI-gated, 18-token grep) |
| CI platforms | Win/macOS/Ubuntu | All green |

---

## Accumulated Context

### Blockers

None.

### Decisions

(None yet for v3.2 -- key decisions to resolve in Phase 38:)

- Clock access pattern in hp41-core: direct `std::time::SystemTime` vs. frontend injection
- Live display architecture: "pull on redraw" pattern -- CLI 16ms poll loop, GUI conditional setInterval
- Interrupting control alarm deferral: document as known divergence per research recommendation

---

## Deferred Items

Items acknowledged and deferred at v3.2 milestone close on 2026-05-25:

| Category | Item | Status |
|----------|------|--------|
| uat_gap | Phase 41: 41-HUMAN-UAT.md — 3 pending visual verification scenarios | partial |
| verification_gap | Phase 41: 41-VERIFICATION.md — human-needed visual confirmation | human_needed |
| quick_task | 260506-a1g-add-gitignore | missing |
| quick_task | 260508-06h-fix-sci-eng-digit-input | missing |
| quick_task | 260508-y30-eex-chs-exponent-sign-toggle | missing |
| quick_task | 260516-c1p-fix-gui-clp-binding-and-modal-letter-clicks | missing |
| quick_task | 260522-g7s-add-yellow-keyboard-frame-matching-vorgabe | missing |
| quick_task | 260522-gud-honor-shift-on-physical-keyboard | missing |

---

*State initialized: 2026-05-06*
*Last updated: 2026-05-25 -- v3.2 milestone close*

## Operator Next Steps

- Start the next milestone with /gsd-new-milestone
