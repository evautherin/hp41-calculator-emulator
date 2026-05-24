---
gsd_state_version: 1.0
milestone: v3.1
milestone_name: — Stat 1 Pac Emulation
status: shipped
last_updated: "2026-05-24"
last_activity: 2026-05-24 -- v3.1 milestone archived via /gsd-complete-milestone
progress:
  total_phases: 5
  completed_phases: 5
  total_plans: 23
  completed_plans: 23
  percent: 100
---

# Project State: HP-41 Calculator Emulator

---

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-24 after v3.1 milestone archive)

**Core value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.

**Shipped milestones:**

- v1.0 CLI (2026-05-08) — Phases 1–8; foundational RPN engine + TUI
- v1.1 CLI Feature Completeness (2026-05-09) — Phases 9–12
- v2.0 Tauri GUI (2026-05-10) — Phases 13–18
- v2.1 Card Reader + Keyboard Authenticity (2026-05-13) — recorded as quick tasks (no Phase 19 GSD directory)
- v2.2 HP-41CV Feature Completeness (2026-05-15) — Phases 20–27, 26/26 plans
- v3.0 Math Pac I Emulation (2026-05-20) — Phases 28–32, 31/31 plans
- **v3.1 Stat 1 Pac Emulation (2026-05-24) — Phases 33–37, 23/23 plans; 95.84% region coverage on `hp41-core`; 98.86% numerical accuracy (791 cases); CI green across `ci.yml` + `ci-gui.yml`**

**Current focus:** Planning next milestone
**Repo:** hp41-calculator-emulator
**Architecture:** Cargo workspace — `hp41-core` (library) + `hp41-cli` (binary) + `hp41-gui` (nested standalone Tauri workspace); `hp41-core` has zero UI/CLI dependencies enforced at compile time.

---

## Current Position

Phase: — (between milestones)
Status: v3.1 shipped and archived
Last activity: 2026-05-24 — v3.1 milestone archived

## Performance Metrics (v3.1 ship)

| Metric | Target | Last measured (v3.1) |
|--------|--------|----------------------|
| Cold-start latency | ≤ 0.5 s | 2.2 ms (M1) — 228× under gate |
| Key-press latency (median) | ≤ 50 ms | ~65 ns/op |
| `hp41-core` line coverage | ≥ 95 % | 93.91 % (denominator dilution from stat1 LOC) |
| `hp41-core` region coverage | ≥ 93 % | **95.84 %** |
| Per-file `ops/stat1/*.rs` floor | ≥ 90 % | 11/12 ≥ 90 % (anova.rs 86.64 %) |
| Numerical accuracy | ≥ 98 % (791 cases) | **98.86 %**; v1.x 503-case floor + v3.0 768-case floor preserved |
| Panics in `hp41-core` | 0 | 0 — enforced by `#![deny(clippy::unwrap_used)]` |
| Free42 contamination | 0 distinctive symbols | 0 (CI-gated, 18-token grep) |
| CI platforms | Win/macOS/Ubuntu | All green (`ci.yml` + `ci-gui.yml` + `e2e-linux` + `license-audit`) |

---

## Accumulated Context

### Blockers

None — milestone shipped.

---

*State initialized: 2026-05-06*
*Last updated: 2026-05-24 — v3.1 Stat 1 Pac Emulation shipped and archived via /gsd-complete-milestone*
