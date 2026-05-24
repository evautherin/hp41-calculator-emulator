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
- **v3.2 Time Pac Emulation** — Phases 38–42, third XROM application module (HP 82182A Time Module, XROM 26, ~33 callable functions across clock/date/alarm/stopwatch, first real-time behavior in the emulator)

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

### v3.2 Time Pac Emulation (Phases 38–42)

**Milestone Goal:** Behavioral emulation of the HP-41CX Time Module (HP 82182A, XROM 26, OM 00041-90035) as the third XROM application module -- date/time arithmetic backed by the host system clock, full alarm catalog with past-due detection, and live-updating stopwatch/clock display. This is the FIRST module introducing real-time behavior into the previously event-driven emulator.

- [ ] **Phase 38: hp41-core -- XROM Framework + Clock/Date/Stopwatch/Alarm Core** - All ~33 Op variants in dispatch()/execute_op(); XROM 26 registration; JDN date arithmetic; stopwatch state machine; alarm catalog; clock display mode; live display architecture decision
- [ ] **Phase 39: hp41-cli -- CLI Integration + Live Display** - 4-way invariant item 3; fourth JSON source-of-truth; live-updating clock/stopwatch in TUI; alarm notifications; stopwatch keyboard mode
- [ ] **Phase 40: Documentation & ADRs** - Divergences catalog; function matrix; ADRs for clock access, live display, alarm design; README soft-claim; CLAUDE.md/architecture-history.md updates
- [ ] **Phase 41: hp41-gui -- GUI Integration + Live Display** - 4-way invariant item 4; tick_time Tauri command with conditional setInterval (D-11 exception); alarm toast; CATALOG 2 + help overlay
- [ ] **Phase 42: Test Hardening & Quality Gates** - Coverage gate; date arithmetic accuracy suite; per-Op meta-gate; backward-compat; E2E smoke; Free42 contamination re-verification; README hard-claim graduation

## Phase Details

### Phase 38: hp41-core -- XROM Framework + Clock/Date/Stopwatch/Alarm Core

**Goal**: Users can access all Time Pac functions through XEQ-by-name and programmatic execution -- TIME/DATE recall the host clock, DATE+/DDAYS/DOW perform calendar arithmetic, stopwatch tracks elapsed time, alarm catalog stores/retrieves/detects past-due alarms, and clock display mode flags control live display behavior
**Depends on**: Phase 37 (v3.1 complete)
**Requirements**: TIME-FW-01, TIME-FW-02, TIME-FW-03, TIME-FW-04, TIME-FW-05, TIME-FW-06, TIME-CLK-01, TIME-CLK-02, TIME-CLK-03, TIME-CLK-04, TIME-CLK-05, TIME-CLK-06, TIME-DAT-01, TIME-DAT-02, TIME-DAT-03, TIME-DAT-04, TIME-DAT-05, TIME-DAT-06, TIME-DSP-01, TIME-DSP-02, TIME-DSP-03, TIME-DSP-04, TIME-DSP-05, TIME-FMT-01, TIME-FMT-02, TIME-FMT-03, TIME-FMT-04, TIME-FMT-05, TIME-SW-01, TIME-SW-02, TIME-SW-03, TIME-SW-04, TIME-SW-05, TIME-SW-06, TIME-SW-07, TIME-SW-08, TIME-SW-09, TIME-ALM-01, TIME-ALM-02, TIME-ALM-03, TIME-ALM-04, TIME-ALM-05, TIME-ALM-06, TIME-ALM-07, TIME-ALM-08, TIME-ALM-09, TIME-ALM-10, TIME-ALM-11, TIME-ALM-12
**Success Criteria** (what must be TRUE):

  1. User can execute `XEQ "TIME"` and `XEQ "DATE"` and see the current host system time/date in the X register (formatted as HH.MMSSss and MM.DDYYYY/DD.MMYYYY per Flag 31)
  2. User can execute `XEQ "DATE+"` with a date in Y and days in X and receive the correct future/past date, including across leap year and century boundaries
  3. User can execute `XEQ "RUNSW"` / `XEQ "STOPSW"` / `XEQ "RCLSW"` and observe elapsed time tracked via monotonic Instant, surviving system clock adjustments
  4. User can execute `XEQ "XYZALM"` to store an alarm and `XEQ "RCLALM"` to recall it, with alarm catalog persisting across save/load cycles
  5. All ~33 Op variants compile in dispatch() and execute_op() (4-way invariant items 1+2 complete; intentional CI break in hp41-cli/hp41-gui until Phase 39/41)

**Plans:** 6 plans

Plans:
**Wave 1**

- [ ] 38-01-PLAN.md — Framework scaffolding: Op enum (35 variants), XROM 26 registration, CalcState fields, migration, Free42 guard, time/ module skeleton

**Wave 2** *(blocked on Wave 1 completion)*

- [ ] 38-02-PLAN.md — Date arithmetic: JDN Fliegel-Van Flandern, parse helpers, DATE+/DDAYS/DOW/DMY/MDY
- [ ] 38-03-PLAN.md — Clock + format ops: TIME/DATE/T+X/CORRECT/SETAF/RCLAF/CLK12/CLK24/CLKT/CLKTD/CLOCK/SETIME/SETDATE
- [ ] 38-04-PLAN.md — Alpha time + Stopwatch: ATIME/ATIME24/ADATE + RUNSW/STOPSW/SETSW/RCLSW/SWPT/STPW/SW

**Wave 3** *(blocked on Wave 2 completion)*

- [ ] 38-05-PLAN.md — Alarm system: XYZALM/RCLALM/ALMCAT/CLALMA/CLALMX/CLRALMS/ALMNOW/check_alarms
- [ ] 38-06-PLAN.md — Modal prompts: TimeStep submit logic for SETIME/SETDATE + ModalProgram::Time wiring

### Phase 39: hp41-cli -- CLI Integration + Live Display

**Goal**: Users can discover and use all Time Pac functions from the CLI with live-updating clock/stopwatch display, searchable help, and alarm notifications surfaced in the status bar
**Depends on**: Phase 38
**Requirements**: TIME-CLI-01, TIME-CLI-02, TIME-CLI-03, TIME-CLI-04, TIME-CLI-05, TIME-CLI-06, TIME-CLI-07, TIME-CLI-08
**Success Criteria** (what must be TRUE):

  1. User can press `?` and see a "Time Pac (XROM 26)" section listing all ~33 Time functions with keybindings, searchable via the incremental substring filter
  2. User can activate clock display mode (XEQ "CLKT") and see the TUI LCD area updating the current time at least once per second without pressing any key
  3. User can enter interactive stopwatch mode (XEQ "SW") and see centisecond-resolution elapsed time updating in the TUI at 10+ Hz, with dedicated key bindings for start/stop/split/reset
  4. When an alarm comes due, the user sees a status-bar notification without having to execute any command

**Plans**: TBD
**UI hint**: yes

### Phase 40: Documentation & ADRs

**Goal**: All Time Pac architectural decisions, emulator divergences from the OM, and function reference material are documented following the v3.0/v3.1 established pattern
**Depends on**: Phase 39
**Requirements**: TIME-DOC-01, TIME-DOC-02, TIME-DOC-03, TIME-DOC-04, TIME-DOC-05, TIME-DOC-06
**Success Criteria** (what must be TRUE):

  1. `just docs-matrix` generates `docs/hp41-time-function-matrix.md` alongside the existing cv/math1/stat1 matrices, and `just docs-matrix-check` passes in CI
  2. `docs/hp41-time-divergences.md` documents the accuracy-factor no-op, host-clock-vs-crystal backing, stopwatch persistence behavior, and interrupting-alarm deferral with rationale
  3. README carries a v3.2 soft-claim for Time Pac emulation (hard-claim deferred to Phase 42 quality gate graduation)

**Plans**: TBD

### Phase 41: hp41-gui -- GUI Integration + Live Display

**Goal**: Users can discover and use all Time Pac functions from the GUI with live-updating clock/stopwatch in the LCD, alarm toast notifications, and full CATALOG 2 + help overlay coverage
**Depends on**: Phase 40
**Requirements**: TIME-GUI-01, TIME-GUI-02, TIME-GUI-03, TIME-GUI-04, TIME-GUI-05, TIME-GUI-06, TIME-GUI-07
**Success Criteria** (what must be TRUE):

  1. User can activate clock display mode and see the GUI LCD updating the current time at least once per second (via conditional setInterval -- documented D-11 exception)
  2. User can enter stopwatch mode and see centisecond-resolution elapsed time updating in the GUI LCD
  3. When an alarm fires, the user sees a toast notification in the GUI with the alarm message
  4. User can browse CATALOG 2 and see the "TIME 2C" section listing all Time Pac functions; help overlay shows "Time Pac (XROM 26)" section

**Plans**: TBD
**UI hint**: yes

### Phase 42: Test Hardening & Quality Gates

**Goal**: All quality gates are maintained or exceeded after Time Pac additions -- comprehensive date arithmetic edge cases, stopwatch timing accuracy, alarm state-machine coverage, backward compatibility with v3.1 save files, and E2E smoke for at least one Time Pac workflow
**Depends on**: Phase 41
**Requirements**: TIME-QUAL-01, TIME-QUAL-02, TIME-QUAL-03, TIME-QUAL-04, TIME-QUAL-05, TIME-QUAL-06, TIME-QUAL-07, TIME-QUAL-08, TIME-QUAL-09, TIME-QUAL-10, TIME-QUAL-11
**Success Criteria** (what must be TRUE):

  1. `hp41-core` region coverage remains >= 93% after all Time Pac additions (denominator dilution from ~33 new ops acknowledged; region coverage is the primary gate per v3.1 precedent)
  2. Date arithmetic accuracy suite covers leap years (Feb 29 2000, 2100, 2400), century boundaries, Gregorian calendar start (Oct 15, 1582), and the DATE+/DDAYS/DOW trio with oracle-verified expected values
  3. v3.1 save file (`xrom_modules: 3`) loads successfully in v3.2 with automatic migration to `xrom_modules: 7`, clock/alarm state defaulting cleanly
  4. E2E smoke test executes at least one Time Pac workflow (e.g., DATE+ or DDAYS) end-to-end in the GUI on Ubuntu
  5. README v3.2 line graduated to OM-cited hard-claim "feature-complete per Owner's Manual 00041-90035" after all quality gates pass

## Progress

**Execution Order:**
Phases execute in numeric order: 38 -> 39 -> 40 -> 41 -> 42

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 38. hp41-core -- XROM + Clock/Date/Stopwatch/Alarm | 0/6 | Planning complete | - |
| 39. hp41-cli -- CLI Integration + Live Display | 0/TBD | Not started | - |
| 40. Documentation & ADRs | 0/TBD | Not started | - |
| 41. hp41-gui -- GUI Integration + Live Display | 0/TBD | Not started | - |
| 42. Test Hardening & Quality Gates | 0/TBD | Not started | - |

---

*Last updated: 2026-05-24 -- Phase 38 planned (6 plans, 3 waves).*
