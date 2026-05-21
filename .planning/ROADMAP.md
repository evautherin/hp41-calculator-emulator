# Roadmap: HP-41 Calculator Emulator

**Project:** HP-41 Calculator Emulator
**Current milestone:** awaiting v3.1 — Stat 1 Pac per scope lock 2026-05-13 (run `/gsd-new-milestone` to start)

---

## Milestones

- ✅ **v1.0 CLI** — Phases 1–8, foundational RPN engine + TUI — SHIPPED 2026-05-08 · [Archive](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 CLI Feature Completeness** — Phases 9–12, EEX fix / STO modals / print / synthetic — SHIPPED 2026-05-09 · [Archive](milestones/v1.1-ROADMAP.md)
- ✅ **v2.0 Tauri GUI** — Phases 13–18, pixel-perfect HP-41C desktop app — SHIPPED 2026-05-10 · [Archive](milestones/v2.0-ROADMAP.md)
- ✅ **v2.1 Card Reader + Keyboard Authenticity** — quick-task entries (no Phase 19 GSD directory) — SHIPPED 2026-05-13 · see MILESTONES.md
- ✅ **v2.2 HP-41CV Feature Completeness** — Phases 20–27, full ROM built-in set + JSON pipeline + GUI integration + coverage gate raise — SHIPPED 2026-05-15 · [Archive](milestones/v2.2-ROADMAP.md)
- ✅ **v3.0 Math Pac I Emulation** — Phases 28–32, first XROM application module (10 prompt-driven programs, ~55 XEQ entry points, 95.39 % line coverage, 99.3 % numerical accuracy) — SHIPPED 2026-05-20 · [Archive](milestones/v3.0-ROADMAP.md)
- ⏳ **v3.1 — TBD** (Stat 1 Pac per scope lock 2026-05-13) — run `/gsd-new-milestone` to define

---

## Phases

<details>
<summary>✅ v1.0 CLI (Phases 1–8) — SHIPPED 2026-05-08</summary>

See [milestones/v1.0-ROADMAP.md](milestones/v1.0-ROADMAP.md) for full phase detail.

</details>

<details>
<summary>✅ v1.1 CLI Feature Completeness (Phases 9–12) — SHIPPED 2026-05-09</summary>

See [milestones/v1.1-ROADMAP.md](milestones/v1.1-ROADMAP.md) for full phase detail. Phase plans archived at `milestones/v1.1-phases/`.

</details>

<details>
<summary>✅ v2.0 Tauri GUI (Phases 13–18) — SHIPPED 2026-05-10</summary>

See [milestones/v2.0-ROADMAP.md](milestones/v2.0-ROADMAP.md) for full phase detail. Phase plans archived at `milestones/v2.0-phases/`.

</details>

<details>
<summary>✅ v2.1 Card Reader + Keyboard Authenticity — SHIPPED 2026-05-13</summary>

Recorded as quick-task entries in MILESTONES.md (no Phase 19 GSD directory; scope evolved out-of-band).

</details>

<details>
<summary>✅ v2.2 HP-41CV Feature Completeness (Phases 20–27) — SHIPPED 2026-05-15</summary>

See [milestones/v2.2-ROADMAP.md](milestones/v2.2-ROADMAP.md) for full phase detail. Phase plans archived at `milestones/v2.2-phases/`.

</details>

<details>
<summary>✅ v3.0 Math Pac I Emulation (Phases 28–32) — SHIPPED 2026-05-20</summary>

- [x] **Phase 28: XROM Framework + Math Pac I Core Ops** — `hp41-core` only; 10 plans; ~40 new `Op` variants; XROM resolver chain fires LAST; modal-workflow state machine; user-callback re-entrancy (ADR-001..005) — completed 2026-05-16
- [x] **Phase 29: CLI Integration** — `hp41-cli` only; 3 plans; `xeq_by_name_local_resolve` → `xrom_resolve`; second `OnceLock` for Math Pac I JSON (DOC-01 absorbed per D-29.1); ~40 new `op_display_name` arms; modal-prompt routing — completed 2026-05-17
- [x] **Phase 30: Documentation & ADRs** — `docs/` + tooling only; 3 plans; `scripts/docs-matrix` two-input extension; 3 new ADRs (v3.0-001/002/005); divergence catalog three-bucket expansion; README v3.0 soft-claim + CLAUDE.md `### v3.0 additions` block — completed 2026-05-17
- [x] **Phase 31: GUI Integration** — `hp41-gui` only; 5 plans; CATALOG 2 + Math Pac I help overlay parallel-load + LCD-alternation modal prompts + R/S 3-way + Esc cascade + `request_cancel` cancellation channel (Pitfall 11) — completed 2026-05-18
- [x] **Phase 32: Test Hardening & Quality Gates** — `tests/` + `scripts/` + `.github/`; 10 plans (3 original + 7 gap-closure); coverage 91.74 % → 95.39 % lines / 92.14 % → 94.26 % regions; `numerical_accuracy.rs` 566 → 763 cases (99.3 % pass); E2E smoke extended (`sinh(1)` + `MATRIX DET`); `scripts/check-free42-contamination.sh` 12-symbol guard wired into `just ci` + `ci.yml::license-audit`; README v3.0 hard-claim graduated per D-32.5 — completed 2026-05-18 → 2026-05-20

See [milestones/v3.0-ROADMAP.md](milestones/v3.0-ROADMAP.md) for full phase detail and decision rationale. Phase plans archived at `milestones/v3.0-phases/`.

</details>

### ⏳ v3.1 — TBD

Run `/gsd-new-milestone` to define v3.1 scope. Per scope lock 2026-05-13: **Stat 1 Pac** is the planned next module-emulation milestone.

---

## Progress

| Phase | Milestone | Plans | Status | Completed |
|-------|-----------|-------|--------|-----------|
| 1–8 | v1.0 | 45 | Complete | 2026-05-08 |
| 9–12 | v1.1 | 14 | Complete | 2026-05-09 |
| 13–18 | v2.0 | 19 | Complete | 2026-05-10 |
| — | v2.1 | quick tasks | Complete | 2026-05-13 |
| 20–27 | v2.2 | 26 | Complete | 2026-05-15 |
| 28–32 | v3.0 | 31 | Complete | 2026-05-20 |

---

*Last updated: 2026-05-21 — v3.0 archived via `/gsd-complete-milestone`. Phase plans 28–32 archived to `milestones/v3.0-phases/`. Orphaned 09–18 retro-archived during the same operation.*
