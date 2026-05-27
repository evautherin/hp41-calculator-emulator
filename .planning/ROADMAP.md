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
- ✅ **v3.3 Advantage Pac Emulation** — Phases 43–47, fourth XROM application module (XROM 22 + XROM 24, 114 XEQ entry points: bitwise/base conversion, named-matrix operations, advanced math/complex/solver/curve-fit, TVM) — SHIPPED 2026-05-26 · [Archive](milestones/v3.3-ROADMAP.md)
- [ ] **v4.0 Platform Maturity** — Phases 48–52, visual themes, onboarding, GUI keyboard parity, `.raw` file I/O, Extended Memory

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

See [milestones/v3.0-ROADMAP.md](milestones/v3.0-ROADMAP.md) for full phase detail. Phase plans archived at `milestones/v3.0-phases/`.

</details>

<details>
<summary>✅ v3.1 Stat 1 Pac Emulation (Phases 33–37) — SHIPPED 2026-05-24</summary>

See [milestones/v3.1-ROADMAP.md](milestones/v3.1-ROADMAP.md) for full phase detail. Phase plans archived at `milestones/v3.1-phases/`.

</details>

<details>
<summary>✅ v3.2 Time Pac Emulation (Phases 38–42) — SHIPPED 2026-05-25</summary>

See [milestones/v3.2-ROADMAP.md](milestones/v3.2-ROADMAP.md) for full phase detail. Phase plans archived at `milestones/v3.2-phases/`.

</details>

<details>
<summary>✅ v3.3 Advantage Pac Emulation (Phases 43–47) — SHIPPED 2026-05-26</summary>

See [milestones/v3.3-ROADMAP.md](milestones/v3.3-ROADMAP.md) for full phase detail. Phase plans archived at `milestones/v3.3-phases/`.

</details>

### v4.0 Platform Maturity (Phases 48–52)

- [x] **Phase 48: GUI Infrastructure + Theming** — prefs.rs backend, 4 skin themes, CSS custom properties (completed 2026-05-27)
- [x] **Phase 49: Onboarding + GUI Keyboard Parity** — first-run guide, searchable reference, keyboard shortcuts (completed 2026-05-27)
- [ ] **Phase 50: .raw File I/O** — import/export via Tauri file dialog and CLI flags
- [ ] **Phase 51: X-MEM Core** — EMDIR/EMROOM/SAVEP/GETP/SAVED/GETD/EMREG ops in hp41-core
- [ ] **Phase 52: Test Hardening + Documentation** — backward compat, full CLI+GUI integration, ADRs, coverage

---

## Phase Details

### Phase 48: GUI Infrastructure + Theming

**Goal**: Users can personalize the calculator's appearance and preferences persist across restarts
**Depends on**: Nothing (first v4.0 phase; zero hp41-core changes)
**Requirements**: INFRA-01, INFRA-02, THEME-01, THEME-02, THEME-03, THEME-04, THEME-05
**Success Criteria** (what must be TRUE):

  1. User can open a theme selector and switch between dark, light, classic beige, and high-contrast themes without restarting
  2. Selected theme is still active the next time the user launches the app
  3. Key press animations work correctly in all four themes (press animation visible, not broken)
  4. High-contrast theme passes WCAG AA contrast ratio for all key labels and display text
  5. Theme preference is stored in `~/.hp41/prefs.json` — it never appears in `autosave.json`

**Plans**: 3 plans
Plans:
**Wave 1**

- [x] 48-01-PLAN.md — Tauri backend: prefs.rs, get_prefs/set_pref commands, permission TOMLs
- [x] 48-02-PLAN.md — CSS theme system: themes.css with 4 data-theme blocks, App.css color migration

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 48-03-PLAN.md — Frontend integration: SettingsPanel, Keyboard gradient props, App.tsx wiring, tests

**UI hint**: yes

### Phase 49: Onboarding + GUI Keyboard Parity

**Goal**: New users can get started immediately and power users can discover every keyboard shortcut
**Depends on**: Phase 48 (prefs.rs backend required for seen-flag and future preference storage)
**Requirements**: ONBOARD-01, ONBOARD-02, ONBOARD-03, ONBOARD-04, ONBOARD-05, KBD-01, KBD-02, KBD-03, KBD-04
**Success Criteria** (what must be TRUE):

  1. First-run shows a quick-start overlay covering RPN entry, stack, and how to XEQ a function; it does not appear again on subsequent launches
  2. User can reopen the quick-start guide from the `?` overlay or help menu at any time
  3. Searchable in-app reference covers all ~350 functions (built-in + all 5 XROM modules) with usage examples
  4. Card reader shortcuts (Ctrl+W/R/D/F) and F5 manual save work in the GUI physical keyboard
  5. The `?` overlay lists all GUI physical keyboard shortcuts alongside function names

**Plans**: 4 plans
Plans:
**Wave 1**

- [x] 49-01-PLAN.md — Rust backend: onboarding_done pref, save_state command, keyboard-shortcuts.json
- [x] 49-02-PLAN.md — Data files: enrich ~50 function entries with example/notes across 5 JSONs

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 49-03-PLAN.md — Components: OnboardingWizard, HelpOverlay KBD section + expandable entries, CSS

**Wave 3** *(blocked on Wave 1 + Wave 2 completion)*

- [x] 49-04-PLAN.md — Wiring: App.tsx onboarding state, Ctrl+key bindings, SettingsPanel Quick Start, visual checkpoint

**UI hint**: yes

### Phase 50: .raw File I/O

**Goal**: Users can exchange HP-41 programs with the HP-41 community via standard `.raw` files
**Depends on**: Nothing (codec already in hp41-core; independent of Phases 48–49)
**Requirements**: RAW-01, RAW-02, RAW-03, RAW-04, RAW-05, RAW-06
**Success Criteria** (what must be TRUE):

  1. User can import a `.raw` file via a native OS file dialog and the program appears in calculator memory
  2. User can export a program to a `.raw` file via a native OS file dialog
  3. A multi-program `.raw` archive either imports all programs or shows a clear error — never silently truncates
  4. XROM instructions survive a round-trip import/export without corruption
  5. CLI users can import and export `.raw` files via `--import-raw` and `--export-raw` flags

**Plans**: 4 plans
Plans:
**Wave 1**

- [ ] 50-01-PLAN.md — Core codec: decode_all_programs, DecodedProgram, picker_label + multi-program tests

**Wave 2** *(blocked on Wave 1 completion)*

- [ ] 50-02-PLAN.md — GUI backend: tauri-plugin-dialog, 5 Tauri commands, permission TOMLs, default.json
- [ ] 50-04-PLAN.md — CLI flags: --import-raw, --export-raw, --import-data, --export-data, --batch + integration tests

**Wave 3** *(blocked on Wave 2 completion)*

- [ ] 50-03-PLAN.md — GUI frontend: App.tsx dialog wiring, RawPickerOverlay component, CSS, visual checkpoint

**UI hint**: yes

### Phase 51: X-MEM Core

**Goal**: Users can store and retrieve named programs and data sets in Extended Memory, mirroring HP-41CX behavior
**Depends on**: Phase 50 (SAVEP/GETP delegate to the `.raw` codec for serialization)
**Requirements**: XMEM-01, XMEM-02, XMEM-03, XMEM-04, XMEM-05, XMEM-06, XMEM-07
**Success Criteria** (what must be TRUE):

  1. `EMDIR` displays a catalog of all named files in extended memory with names, types (program/data), and sizes
  2. `EMROOM` reports the number of available extended memory registers as a numeric value on the stack
  3. `SAVEP` / `GETP` round-trips a program through extended memory and retrieves it intact
  4. `SAVED` / `GETD` round-trips data registers through extended memory and retrieves them intact
  5. `EMREG` accesses register N within the active X-MEM file, returning the stored value

**Plans**: TBD

### Phase 52: Test Hardening + Documentation

**Goal**: Extended Memory is production-ready — backward-compatible, isolated, and fully integrated across CLI, GUI, and test suite
**Depends on**: Phase 51 (X-MEM core ops required before integration and testing)
**Requirements**: XMEM-08, XMEM-09, XMEM-10
**Success Criteria** (what must be TRUE):

  1. A v3.3 save file loads without error in v4.0 and `xmem_files` is empty (not missing/erroring)
  2. X-MEM operations never read from or write to `state.regs` or `adv_matrices` — verified by targeted isolation tests
  3. XEQ "EMDIR", XEQ "SAVEP", and all X-MEM functions are reachable from both CLI and GUI and appear in the `?` help overlay

**Plans**: TBD

---

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 48. GUI Infrastructure + Theming | 3/3 | Complete   | 2026-05-27 |
| 49. Onboarding + GUI Keyboard Parity | 4/4 | Complete   | 2026-05-27 |
| 50. .raw File I/O | 0/4 | Planned | - |
| 51. X-MEM Core | 0/? | Not started | - |
| 52. Test Hardening + Documentation | 0/? | Not started | - |

---

*Last updated: 2026-05-27 — Phase 50 planned (4 plans, 3 waves)*
