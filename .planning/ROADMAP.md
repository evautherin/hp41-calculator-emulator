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
- 🚧 **v3.3 Advantage Pac Emulation** — Phases 43–47, fourth XROM application module (XROM 22 + XROM 24, ~117 ops: bitwise/base conversion, named-matrix operations, advanced math/complex/solver/curve-fit, TVM)

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

### v3.3 Advantage Pac Emulation (In Progress)

**Milestone Goal:** Behavioral emulation of the HP-41 Advantage Pac (OM 00041-90482) as the fourth XROM application module, completing all remaining HP-41 module emulation. Two XROM IDs (22 + 24): ~117 ops across bitwise/base conversion (ADV CONV), named-matrix operations (ADV MTRX), advanced math/complex/solver/curve-fit (ADV MATH), and time-value-of-money (ADV TVM).

## Phases (v3.3)

- [x] **Phase 43: hp41-core — XROM Framework + All Advantage Pac Ops** - XROM 22 + XROM 24 registration, named-matrix CalcState model, AdvantageStep modal variant, and all ~117 Op variants (ADV CONV/MTRX/MATH/TVM) implemented in `ops/advantage/` (completed 2026-05-25)
- [x] **Phase 44: hp41-cli — CLI Integration** - Fifth JSON canonical source, op_display_name arms, help overlay sections, xrom_shadowing extension, modal-prompt routing (completed 2026-05-26)
- [ ] **Phase 45: Documentation & ADRs** - Divergences catalog, function matrix, ADRs for named-matrix model / FROOT algorithm / dual-XROM design / math1 visibility promotions, CLAUDE.md v3.3 block
- [ ] **Phase 46: hp41-gui — GUI Integration** - GUI op_display_name arms (4-way invariant item 4), HelpOverlay sections, CATALOG 2 entries for XROM 22+24, modal LCD rendering
- [ ] **Phase 47: Test Hardening & Quality Gates** - Unified meta-gates extended to Advantage Pac, FROOT/FINTG/MDET/MINV accuracy oracles, backward compat v3.2→v3.3, E2E smoke, README hard-claim graduated

## Phase Details

### Phase 43: hp41-core — XROM Framework + All Advantage Pac Ops

**Goal**: All Advantage Pac operations are callable via XEQ in `hp41-core`, with XROM 22 + XROM 24 registered, named-matrix storage on CalcState, and the full ~117-op ADV CONV / ADV MTRX / ADV MATH / ADV TVM behavioral implementations complete
**Depends on**: Phase 42 (v3.2 shipped baseline)
**Requirements**: ADV-FW-01, ADV-FW-02, ADV-FW-03, ADV-FW-04, ADV-FW-05, ADV-FW-06, ADV-CONV-01, ADV-CONV-02, ADV-CONV-03, ADV-CONV-04, ADV-CONV-05, ADV-CONV-06, ADV-CONV-07, ADV-CONV-08, ADV-CONV-09, ADV-CONV-10, ADV-CONV-11, ADV-CONV-12, ADV-MTX-01, ADV-MTX-02, ADV-MTX-03, ADV-MTX-04, ADV-MTX-05, ADV-MTX-06, ADV-MTX-07, ADV-MTX-08, ADV-MTX-09, ADV-MTX-10, ADV-MTX-11, ADV-MTX-12, ADV-MTX-13, ADV-MTX-14, ADV-MTX-15, ADV-MTX-16, ADV-MTX-17, ADV-MTX-18, ADV-MTX-19, ADV-MTX-20, ADV-MTX-21, ADV-MTX-22, ADV-MTX-23, ADV-MTX-24, ADV-MTX-25, ADV-MTX-26, ADV-MTX-27, ADV-MTX-28, ADV-MTX-29, ADV-MTX-30, ADV-MTX-31, ADV-MTX-32, ADV-MTX-33, ADV-MTX-34, ADV-MTX-35, ADV-MTX-36, ADV-MTX-37, ADV-MTX-38, ADV-MTX-39, ADV-MTX-40, ADV-MTX-41, ADV-MTX-42, ADV-MTX-43, ADV-MTX-44, ADV-MTX-45, ADV-MTX-46, ADV-MTX-47, ADV-MTX-48, ADV-MTX-49, ADV-MTX-50, ADV-MATH-01, ADV-MATH-02, ADV-MATH-03, ADV-MATH-04, ADV-MATH-05, ADV-MATH-06, ADV-MATH-07, ADV-MATH-08, ADV-MATH-09, ADV-MATH-10, ADV-MATH-11, ADV-MATH-12, ADV-MATH-13, ADV-MATH-14, ADV-MATH-15, ADV-MATH-16, ADV-MATH-17, ADV-MATH-18, ADV-MATH-19, ADV-MATH-20, ADV-MATH-21, ADV-MATH-22, ADV-MATH-23, ADV-MATH-24, ADV-MATH-25, ADV-MATH-26, ADV-MATH-27, ADV-MATH-28, ADV-MATH-29, ADV-MATH-30, ADV-MATH-31, ADV-MATH-32, ADV-MATH-33, ADV-MATH-34, ADV-MATH-35, ADV-MATH-36, ADV-MATH-37, ADV-MATH-38, ADV-MATH-39, ADV-MATH-40, ADV-MATH-41, ADV-MATH-42, ADV-MATH-43, ADV-MATH-44, ADV-MATH-45, ADV-MATH-46, ADV-MATH-47, ADV-TVM-01, ADV-TVM-02, ADV-TVM-03, ADV-TVM-04, ADV-TVM-05, ADV-TVM-06
**Success Criteria** (what must be TRUE):

  1. `XEQ "BININ"` in hp41-core converts a binary string in ALPHA to an integer in X without panicking
  2. `XEQ "MATDIM"` creates a named matrix in `adv_matrices`, and `XEQ "MR"` / `XEQ "MS"` read and write elements by current index
  3. `XEQ "FROOT"` computes roots of a polynomial using Laguerre's method; `XEQ "FINTG"` integrates a user callback via Romberg; both fire the user-program callback via existing `run_loop` re-entrancy
  4. `XEQ "TVM"` modal prompts for N, I, PV, PMT, FV; `XEQ "*I"` converges via Newton iteration; v3.2 save files load without error (xrom_modules migrated 0b0111 → 0b11111)
  5. All new Op variants compile without errors in `dispatch()` and `execute_op()` (4-way invariant items 1+2 satisfied); `hp41-cli` and `hp41-gui` emit only expected `non-exhaustive patterns` compile breaks (items 3+4 deferred to Phase 44/46)

**Plans:** 10/10 plans complete

Plans:
**Wave 1**

- [x] 43-01-PLAN.md — XROM framework + advantage/ module skeleton + all Op stubs + dispatch wiring

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 43-02-PLAN.md — ADV CONV: 12 base conversion + bitwise logic ops (36-bit word size)
- [x] 43-03-PLAN.md — ADV MTRX element access + lifecycle + reductions (33 ops)

**Wave 3** *(blocked on Wave 2 completion)*

- [x] 43-04-PLAN.md — ADV MTRX linear algebra: LU decomposition + MDET/MINV/MSYS/M*M (10 ops)
- [x] 43-05-PLAN.md — Complex extensions (18 ops) + complex matrix ops (5 ops)
- [x] 43-06-PLAN.md — Curve fitting (7 ops) + vectors/coordinate transform (14 ops)

**Wave 4** *(blocked on Wave 3 completion)*

- [x] 43-07-PLAN.md — Solvers: FSOLVE/FINTG/FDIFEQ/FROOT + PLY/RTS (6 ops, run_loop re-entrancy)
- [x] 43-08-PLAN.md — Matrix modal workflows: MATRX/MTR/MEDIT/CMEDIT + submit_step wiring

**Wave 5** *(blocked on Wave 4 completion)*

- [x] 43-09-PLAN.md — TVM: TVM/N/PV/PMT/FV/*I (6 ops, Newton iteration)
- [x] 43-10-PLAN.md — Integration verification: stub audit, test suite, contamination guard

### Phase 44: hp41-cli — CLI Integration

**Goal**: All Advantage Pac functions are discoverable and executable from the CLI: `XEQ "FROOT"` etc. work end-to-end, the `?` overlay shows "Advantage Pac (XROM 22)" and "Advantage Pac (XROM 24)" sections, and the JSON canonical pipeline is extended to the fifth pool
**Depends on**: Phase 43
**Requirements**: ADV-CLI-01, ADV-CLI-02, ADV-CLI-03, ADV-CLI-04, ADV-CLI-05, ADV-CLI-06, ADV-CLI-07, ADV-CLI-08
**Success Criteria** (what must be TRUE):

  1. `docs/hp41-advantage-functions.json` exists with entries for all Advantage Pac ops; `function_matrix_parity.rs` 5-pool partition test passes
  2. `?` overlay in the CLI displays four Advantage Pac category sections ("Adv Conv", "Adv Mtrx", "Adv Math", "Adv TVM"); substring search spans all five JSON pools
  3. `XEQ "MATRX"`, `XEQ "TVM"`, `XEQ "MEDIT"`, and `XEQ "VE"` route through modal-prompt infrastructure with correct prompts appearing in the status bar
  4. `xrom_shadowing.rs` CI gate passes: all ADV MTRX + ADV MATH + ADV TVM + ADV CONV mnemonics are confirmed disjoint from all other XROM modules and built-in card ops (Pitfall 22 across all 5 XROM modules)
  5. `hp41-cli` compiles with no `non-exhaustive patterns` warnings — 4-way invariant item 3 is fully satisfied

**Plans:** 2/2 plans complete

Plans:
**Wave 1**

- [x] 44-01-PLAN.md — JSON authoring (114 entries) + fifth OnceLock pool + docs-matrix extension

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 44-02-PLAN.md — Partition parity + xrom shadowing + modal flow + help data smoke + key coverage tests

### Phase 45: Documentation & ADRs

**Goal**: The Advantage Pac emulation is fully documented: divergence catalog, function matrix, ADRs capturing the four key architectural decisions, and CLAUDE.md + architecture-history.md updated for v3.3
**Depends on**: Phase 44
**Requirements**: ADV-DOC-01, ADV-DOC-02, ADV-DOC-03, ADV-DOC-04, ADV-DOC-05, ADV-DOC-06
**Success Criteria** (what must be TRUE):

  1. `docs/hp41-advantage-function-matrix.md` is generated by `just docs-matrix` (fifth invocation) and `just docs-matrix-check` passes without drift
  2. `docs/hp41-advantage-divergences.md` exists with a three-bucket numbered catalog covering at minimum: named-matrix vs R14/R15+ incompatibility, FROOT Laguerre vs Math Pac I Bairstow coexistence, TVM register persistence policy, and matrix size cap
  3. At least four ADRs exist for v3.3: named-matrix storage model, FROOT algorithm choice (Laguerre), dual-XROM-ID design (XROM 22 + 24), and math1/ visibility-promotion policy; each follows the D-30.6 long-form template
  4. `docs/architecture-history.md` contains a `## v3.3 additions` section and CLAUDE.md contains a `### v3.3 additions` block
  5. README contains the v3.3 soft-claim bullet under `## Features`

**Plans:** 2 plans

Plans:
**Wave 1**

- [ ] 45-01-PLAN.md — Divergence catalog (9 D-45-NN entries, 3 buckets) + 4 ADRs (v3.3-001 through v3.3-004)

**Wave 2** *(blocked on Wave 1 completion)*

- [ ] 45-02-PLAN.md — architecture-history.md v3.3 narrative + CLAUDE.md v3.3 block + README soft-claim + ADV-DOC-01 verification

### Phase 46: hp41-gui — GUI Integration

**Goal**: All Advantage Pac functions are accessible from the Tauri GUI: op_display_name arms complete the 4-way invariant, CATALOG 2 lists XROM 22 + XROM 24, and modal workflows (MATRX, MTR, TVM, MEDIT/CMEDIT) render prompts correctly in the LCD
**Depends on**: Phase 45
**Requirements**: ADV-GUI-01, ADV-GUI-02, ADV-GUI-03, ADV-GUI-04
**Success Criteria** (what must be TRUE):

  1. `hp41-gui` compiles with no `non-exhaustive patterns` warnings — 4-way invariant item 4 is fully satisfied
  2. CATALOG 2 in the GUI lists entries for XROM 22 (ADV MTRX / ADV CONV) and XROM 24 (ADV MATH / ADV TVM) with correct HP hardware display strings
  3. `XEQ "TVM"` from the GUI keyboard launches the TVM modal workflow; LCD alternates between the prompt label and current X value on each step, matching the Math Pac I modal pattern
  4. HelpOverlay.tsx displays "Advantage Pac (XROM 22)" and "Advantage Pac (XROM 24)" sections; search across all five JSON pools returns Advantage Pac entries

**UI hint**: yes
**Plans**: TBD

### Phase 47: Test Hardening & Quality Gates

**Goal**: The Advantage Pac emulation satisfies all quality gates: per-Op test coverage, numerical accuracy oracles for key algorithms, backward compatibility from v3.2 save files, E2E smoke in the GUI, Free42 contamination guard extended, and README hard-claim graduated
**Depends on**: Phase 46
**Requirements**: ADV-QUAL-01, ADV-QUAL-02, ADV-QUAL-03, ADV-QUAL-04, ADV-QUAL-05, ADV-QUAL-06, ADV-QUAL-07, ADV-QUAL-08, ADV-QUAL-09
**Success Criteria** (what must be TRUE):

  1. `xrom_op_test_count.rs` passes for all Advantage Pac Op variants (>= 5 tests each); `lint_xrom_assertions.rs` covers all Advantage Pac test files
  2. All `ops/advantage/*.rs` source files achieve >= 90% region coverage; aggregate `hp41-core` region coverage is >= 93%
  3. Numerical accuracy oracle suite includes FROOT (polynomial roots, >= 5 cases), FINTG (Romberg integration, >= 5 cases), MDET (matrix determinant), and MINV (matrix inverse); all oracle cases pass within tolerance
  4. `adv_backward_compat.rs` test confirms v3.2 save files (`xrom_modules = 0b0111`) load and migrate to `0b11111` without data loss; `adv_matrices` defaults to empty; existing `rand_seed` and `time_offset_secs` are preserved
  5. E2E smoke test in `hp41-gui/e2e/smoke.spec.js` exercises an Advantage Pac workflow (e.g., FROOT or MDET); Free42 contamination guard covers `advantage/` directory and exits 0; README hard-claim reads "feature-complete per Owner's Manual 00041-90482"

**Plans**: TBD

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 43. hp41-core — XROM Framework + All Advantage Pac Ops | 10/10 | Complete   | 2026-05-25 |
| 44. hp41-cli — CLI Integration | 2/2 | Complete   | 2026-05-26 |
| 45. Documentation & ADRs | 0/2 | Not started | - |
| 46. hp41-gui — GUI Integration | 0/TBD | Not started | - |
| 47. Test Hardening & Quality Gates | 0/TBD | Not started | - |

---

*Last updated: 2026-05-26 — Phase 45 planned (2 plans in 2 waves).*
