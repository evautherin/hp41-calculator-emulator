# Roadmap: HP-41 Calculator Emulator

**Project:** HP-41 Calculator Emulator

---

## Milestones

- ✅ **v1.0 CLI** — Phases 1–8, foundational RPN engine + TUI — SHIPPED 2026-05-08 · [Archive](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 CLI Feature Completeness** — Phases 9–12, EEX fix / STO modals / print / synthetic — SHIPPED 2026-05-09 · [Archive](milestones/v1.1-ROADMAP.md)
- ✅ **v2.0 Tauri GUI** — Phases 13–18, pixel-perfect HP-41C desktop app — SHIPPED 2026-05-10 · [Archive](milestones/v2.0-ROADMAP.md)
- ✅ **v2.1 Card Reader + Keyboard Authenticity** — quick-task entries (no Phase 19 GSD directory) — SHIPPED 2026-05-13 · see MILESTONES.md
- ✅ **v2.2 HP-41CV Feature Completeness** — Phases 20–27, full ROM built-in set + JSON pipeline + GUI integration + coverage gate raise — SHIPPED 2026-05-15 · [Archive](milestones/v2.2-ROADMAP.md)
- ✅ **v3.0 Math Pac I Emulation** — Phases 28–32, first XROM application module (10 prompt-driven programs, ~55 XEQ entry points, 95.39 % line coverage, 99.3 % numerical accuracy) — SHIPPED 2026-05-20 · [Archive](milestones/v3.0-ROADMAP.md)
- ✅ **v3.1 Stat 1 Pac Emulation** — Phases 33–37, second XROM application module (13 programs, 26 XEQ entry points, RAND/SEED extension, 98.86 % numerical accuracy) — SHIPPED 2026-05-24 · [Archive](milestones/v3.1-ROADMAP.md)
- ✅ **v3.2 Time Pac Emulation** — Phases 38–42, third XROM application module (HP 82182A Time Module, XROM 26, 35 XEQ entry points, first real-time behavior, 96.01% region coverage) — SHIPPED 2026-05-25 · [Archive](milestones/v3.2-ROADMAP.md)

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
- [x] **Phase 29: CLI Integration** — `hp41-cli` only; 3 plans; `xeq_by_name_local_resolve` -> `xrom_resolve`; second `OnceLock` for Math Pac I JSON (DOC-01 absorbed per D-29.1); ~40 new `op_display_name` arms; modal-prompt routing — completed 2026-05-17
- [x] **Phase 30: Documentation & ADRs** — `docs/` + tooling only; 3 plans; `scripts/docs-matrix` two-input extension; 3 new ADRs (v3.0-001/002/005); divergence catalog three-bucket expansion; README v3.0 soft-claim + CLAUDE.md `### v3.0 additions` block — completed 2026-05-17
- [x] **Phase 31: GUI Integration** — `hp41-gui` only; 5 plans; CATALOG 2 + Math Pac I help overlay parallel-load + LCD-alternation modal prompts + R/S 3-way + Esc cascade + `request_cancel` cancellation channel (Pitfall 11) — completed 2026-05-18
- [x] **Phase 32: Test Hardening & Quality Gates** — `tests/` + `scripts/` + `.github/`; 10 plans (3 original + 7 gap-closure); coverage 91.74 % -> 95.39 % lines / 92.14 % -> 94.26 % regions; `numerical_accuracy.rs` 566 -> 763 cases (99.3 % pass); E2E smoke extended (`sinh(1)` + `MATRIX DET`); `scripts/check-free42-contamination.sh` 12-symbol guard wired into `just ci` + `ci.yml::license-audit`; README v3.0 hard-claim graduated per D-32.5 — completed 2026-05-18 -> 2026-05-20

See [milestones/v3.0-ROADMAP.md](milestones/v3.0-ROADMAP.md) for full phase detail. Phase plans archived at `milestones/v3.0-phases/`.

</details>

<details>
<summary>✅ v3.1 Stat 1 Pac Emulation (Phases 33–37) — SHIPPED 2026-05-24</summary>

- [x] **Phase 33: hp41-core — XROM Activation + Distribution Primitives + All Stat 1 Ops** — 9 plans; 26 new Op variants; 3 hand-coded distribution primitives (Acklam/AS 241 + Cody AS 239 + Lentz AS 63); STAT_1 XROM registration; `default_xrom_modules` migration; `rand_seed` CalcState field — completed 2026-05-22
- [x] **Phase 34: hp41-cli — CLI Integration** — 2 plans; `docs/hp41-stat1-functions.json` (26 entries); third `OnceLock`; 26 `op_display_name` arms; `?` overlay "Stat 1 Pac (XROM 2)" section — completed 2026-05-23
- [x] **Phase 35: Documentation & ADRs** — 4 plans; `docs/hp41-stat1-divergences.md` (12 D-35-NN entries); `docs/hp41-stat1-function-matrix.md`; 5 ADRs (v3.1-001..005); `33-SPEC-AMENDMENT.md`; README v3.1 soft-claim; CLAUDE.md `### v3.1 additions` block — completed 2026-05-23
- [x] **Phase 36: hp41-gui — GUI Integration** — 3 plans; 26 GUI `op_display_name` arms (4-way invariant sealed); CATALOG 2 "STAT 1B"; help overlay third section; LCD-alternation modal prompts — completed 2026-05-24
- [x] **Phase 37: Test Hardening & Quality Gates** — 5 plans; meta-gate infrastructure; coverage gap closure; 791 accuracy cases (98.86%); backward-compat test; E2E smoke NORMD; README hard-claim graduated — completed 2026-05-24

See [milestones/v3.1-ROADMAP.md](milestones/v3.1-ROADMAP.md) for full phase detail. Phase plans archived at `milestones/v3.1-phases/`.

</details>

<details>
<summary>✅ v3.2 Time Pac Emulation (Phases 38–42) — SHIPPED 2026-05-25</summary>

- [x] **Phase 38: hp41-core — XROM Framework + Clock/Date/Stopwatch/Alarm Core** — 6 plans; 35 new Op variants; XROM 26 registration; JDN date arithmetic; stopwatch state machine; alarm catalog; clock display mode — completed 2026-05-24
- [x] **Phase 39: hp41-cli — CLI Integration + Live Display** — 3 plans; fourth JSON pool; live clock/stopwatch display; alarm event draining; stopwatch keyboard mode — completed 2026-05-25
- [x] **Phase 40: Documentation & ADRs** — 3 plans; divergences catalog; function matrix; 3 ADRs; README soft-claim — completed 2026-05-25
- [x] **Phase 41: hp41-gui — GUI Integration + Live Display** — 3 plans; tick_time conditional setInterval; alarm toast; CATALOG 2 + help overlay — completed 2026-05-25
- [x] **Phase 42: Test Hardening & Quality Gates** — 4 plans; unified meta-gates; 96.01% region coverage; date accuracy + stopwatch timing + alarm latency suites; backward compat; E2E DDAYS smoke; README hard-claim graduated — completed 2026-05-25

See [milestones/v3.2-ROADMAP.md](milestones/v3.2-ROADMAP.md) for full phase detail.

</details>

## Phase Details

No active phases — all milestones shipped. Start next milestone with `/gsd-new-milestone`.

---

*Last updated: 2026-05-25 — v3.2 Time Pac Emulation milestone shipped and archived.*
