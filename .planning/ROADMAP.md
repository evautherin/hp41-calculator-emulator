# Roadmap: HP-41 Calculator Emulator

**Project:** HP-41 Calculator Emulator
**Current milestone:** v3.1 — Stat 1 Pac Emulation (Phases 33–37)

---

## Milestones

- ✅ **v1.0 CLI** — Phases 1–8, foundational RPN engine + TUI — SHIPPED 2026-05-08 · [Archive](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 CLI Feature Completeness** — Phases 9–12, EEX fix / STO modals / print / synthetic — SHIPPED 2026-05-09 · [Archive](milestones/v1.1-ROADMAP.md)
- ✅ **v2.0 Tauri GUI** — Phases 13–18, pixel-perfect HP-41C desktop app — SHIPPED 2026-05-10 · [Archive](milestones/v2.0-ROADMAP.md)
- ✅ **v2.1 Card Reader + Keyboard Authenticity** — quick-task entries (no Phase 19 GSD directory) — SHIPPED 2026-05-13 · see MILESTONES.md
- ✅ **v2.2 HP-41CV Feature Completeness** — Phases 20–27, full ROM built-in set + JSON pipeline + GUI integration + coverage gate raise — SHIPPED 2026-05-15 · [Archive](milestones/v2.2-ROADMAP.md)
- ✅ **v3.0 Math Pac I Emulation** — Phases 28–32, first XROM application module (10 prompt-driven programs, ~55 XEQ entry points, 95.39 % line coverage, 99.3 % numerical accuracy) — SHIPPED 2026-05-20 · [Archive](milestones/v3.0-ROADMAP.md)
- ⏳ **v3.1 Stat 1 Pac Emulation** — Phases 33–37, second XROM application module (13 programs, ~24 Op variants, distribution primitives, ANOVA family, multiple + polynomial regression, hypothesis tests, RNG bonus)

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

### ⏳ v3.1 — Stat 1 Pac Emulation (Phases 33–37)

- [x] **Phase 33: hp41-core — XROM Activation + Distribution Primitives + All Stat 1 Ops** — Spec-phase OM layout transcription; contamination-guard extension (Pitfall 27, first); STAT_1 XROM registration (bit 1); `default_xrom_modules` migration (0b0000_0001 → 0b0000_0011); `rand_seed` CalcState field; `stat1/distributions.rs` (3 numerical primitives); all ~24 Op variants across `stat1/` module tree; 4-way invariant items 1 + 2 complete. Estimate 9–11 plans. (completed 2026-05-22)
- [x] **Phase 34: hp41-cli — CLI Integration** — `docs/hp41-stat1-functions.json` canonical source; third `OnceLock<Vec<HelpEntry>>`; ~24 new `op_display_name` arms; "Stat 1 Pac (XROM 2)" help-overlay section; `xrom_shadowing.rs` extended to STAT_1.ops; modal-prompt routing for multi-step Stat 1 workflows. Estimate 2–3 plans. (completed 2026-05-23)
- [x] **Phase 35: Documentation & ADRs** — `docs/hp41-stat1-divergences.md` three-bucket catalog; `scripts/docs-matrix` three-input extension; `docs/hp41-stat1-function-matrix.md` generated; new ADRs for v3.1 architectural decisions (RNG-state serde, register layout, distribution-primitive policy); README v3.1 section + CLAUDE.md `### v3.1 additions` block; `docs/architecture-history.md` v3.1 narrative. Estimate 3–4 plans. (completed 2026-05-23)
- [ ] **Phase 36: hp41-gui — GUI Integration** — ~24 new `op_display_name` arms in GUI `prgm_display.rs`; CATALOG 2 gains "STAT 1B" XROM entry; help overlay "Stat 1 Pac (XROM 2)" parallel-load section; modal-prompt routing for Stat 1 multi-step workflows; `request_cancel` reuse for iterative-quantile paths (ΣNORMD inverse + ΣCHISQD CDF). Estimate 3–5 plans.
- [ ] **Phase 37: Test Hardening & Quality Gates** — `hp41-core` coverage hold ≥ 95.39 % lines / ≥ 94.26 % regions; per-file `stat1/*.rs` floor ≥ 90 %; two-level tolerance discipline (1e-9 closed-form, 1e-7 iterative); `stat1_op_test_count.rs` meta-gate; `xrom_shadowing.rs` STAT_1 extension; backward-compat test for v3.0 save migration; `numerical_accuracy.rs` extended with ~30 Stat 1 oracle cases; `lint_stat1_assertions.rs`; `stat1_rand_determinism.rs`; E2E smoke extended with one Stat 1 Pac workflow on Ubuntu. Estimate 6–10 plans.

---

## Phase Details

### Phase 33: hp41-core — XROM Activation + Distribution Primitives + All Stat 1 Ops

**Goal**: Users can call all 13 Stat 1 Pac programs via `XEQ "ΣNORMD"`, `XEQ "ΣSPEAR"`, etc. from within `hp41-core` — distributions return correct values, ANOVA family respects OM register layout, RAND/SEED persist across save/load
**Depends on**: Phase 32 (v3.0 shipped; all Math Pac I infrastructure in place)
**Requirements**: STAT-FW-01, STAT-FW-02, STAT-FW-03, STAT-FW-04, STAT-UNI-01, STAT-UNI-02, STAT-UNI-03, STAT-UNI-04, STAT-AOV-01, STAT-AOV-02, STAT-AOV-03, STAT-AOV-04, STAT-REG-01, STAT-REG-02, STAT-REG-03, STAT-REG-04, STAT-REG-05, STAT-REG-06, STAT-REG-07, STAT-REG-08, STAT-REG-09, STAT-HYP-01, STAT-HYP-02, STAT-HYP-03, STAT-HYP-04, STAT-HYP-05, STAT-HYP-06, STAT-HYP-07, STAT-DST-01, STAT-DST-02, STAT-DST-03, STAT-DST-04, STAT-DST-05, STAT-DST-06, STAT-DST-07, STAT-RNG-01, STAT-RNG-02, STAT-RNG-03, STAT-RNG-04
**Success Criteria** (what must be TRUE):

  1. `norm_cdf_inv_f64(0.025)` returns a value within 1e-9 of 1.959964 (Acklam/AS 241 rational approximation validated against scipy.stats oracle before any Op is implemented)
  2. `XEQ "ΣNORMD"` with x = 1.96 returns Q ≈ 0.0250 (upper-tail CDF); with x = 0 returns PDF = 0.3989; with p = 0.025 via inverse path returns x ≈ 1.9600
  3. `XEQ "ΣCHISQD"` with ν = 3, x = 7.815 returns P ≈ 0.9500 (regularized incomplete gamma validated to 1e-7)
  4. `XEQ "ΣAOVONE"` accumulates group data across multiple groups and returns a correct F-ratio — register layout transcribed from OM "Storage Registers" section before any ANOVA Op line is written
  5. `XEQ "RAND"` called twice with a fixed seed returns the same deterministic sequence after a save/load round-trip (RNG seed survives serde with `#[serde(default)]` NOT `skip`)

**Plans**: 9 plans
**UI hint**: no
Plans:

- [x] 33-00-PLAN.md — Free42 contamination guard extension (stats-domain identifiers) + OM Storage Registers transcription in ops/stat1/mod.rs (Wave 0, no Op code)
- [x] 33-01-PLAN.md — XROM framework activation: STAT_1 const + bit-1 arm + stat1_resolve + default_xrom_modules→0b0000_0011 + migrate_after_load + rand_seed field + Stat1Step skeleton + Op::Stat1Stub scaffolding (Wave 1)
- [x] 33-02-PLAN.md — Three hand-coded distribution primitives (Acklam/AS 241 + AS 239 + AS 63) with ≥18 inline scipy.stats oracle tuples; all GREEN before any Op (Wave 2)
- [x] 33-03-PLAN.md — ΣNORMD (CDF/PDF/inverse) + ΣCHISQD (ν-prompt+PDF/CDF) + HpError::Cancelled/ConvergenceFailed + quantile_threshold + stat1_cancellation.rs (Wave 2)
- [x] 33-04-PLAN.md — ΣSPEAR + ΣXSQEV + ΣEFXSQ (closed-form nonparametrics; SPEC.md Req. 30 oracle 0.7→0.8 discrepancy resolution) (Wave 2)
- [x] 33-05-PLAN.md — ΣBSTAT/BSTG (univariate) + ΣLIN/EXP/LOGI/POW (log-linearization with op_sigma_plus delegate) (Wave 2)
- [x] 33-06-PLAN.md — ΣMMTUG/MMTGD + ΣAOVONE/AOVTWO/ANOCOV + ΣCTKKK/CTKK (OM-register-dependent set, P21 mitigation, op_sigma_minus extension for [C] correction-key) (Wave 3)
- [x] 33-07-PLAN.md — ΣPTST + ΣTSTAT (pooled-variance two-sample t-test, Welch excluded; consumes beta_regularized_f64) (Wave 3)
- [x] 33-08-PLAN.md — ΣMLRXY/MLRXYZ + ΣPOLYP/POLYC + RAND/SEED + Op::Stat1Stub deletion + stat1_rand_determinism.rs (Wave 3, final)

### Phase 34: hp41-cli — CLI Integration

**Goal**: Users can discover and invoke all Stat 1 Pac programs from the CLI — `XEQ "ΣSPEAR"` works from the keyboard, the `?` overlay shows a "Stat 1 Pac (XROM 2)" section, and program listings display all ~24 new Op variant names
**Depends on**: Phase 33
**Requirements**: STAT-CLI-01, STAT-CLI-02, STAT-CLI-03, STAT-CLI-04, STAT-CLI-05
**Success Criteria** (what must be TRUE):

  1. Typing `XEQ "ΣNORMD"` at the CLI keyboard invokes the ΣNORMD Op via the existing `xeq_by_name_local_resolve` → `xrom_resolve` chain — no CLI code change required beyond confirming fall-through still works
  2. Pressing `?` in the CLI shows a "Stat 1 Pac (XROM 2)" section listing all 14 entry-point mnemonics; search across all three JSON pools (`hp41cv-functions.json` + `hp41-math1-functions.json` + `hp41-stat1-functions.json`) returns Stat 1 results
  3. A user program containing `XEQ "ΣSPEAR"` displays `XEQ "ΣSPEAR"` in the program listing panel (not a raw Op ID) — `op_display_name` arms are exhaustive with no `_ =>` catch-all
  4. Multi-step Stat 1 workflows (e.g. ΣPOLYP degree prompt, SEED prompt) appear on the CLI status bar via the existing `print_buffer` + `modal_program` infrastructure with no new transient `CalcState` fields

**Plans**: 2 plans
**UI hint**: no
Plans:
**Wave 1**

- [x] 34-01-PLAN.md — Author docs/hp41-stat1-functions.json (26 entries, D-34.1/D-34.3/C-28.3) + third OnceLock pool + help_entries_stat1() + help_entries_all() 3-pool chain + phase34_help_data_stat1.rs smoke test (STAT-CLI-02)

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 34-02-PLAN.md — 26 op_display_name arms (closes CI break, 4-way invariant item 3) + function_matrix_parity 3-pool partition + phase25_xeq_by_name Stat 1 extension + phase34_modal_flow.rs + phase34_key_ref_includes_stat1.rs + just ci verification (STAT-CLI-01, 03, 04, 05)

### Phase 35: Documentation & ADRs

**Goal**: Stat 1 Pac integration is fully documented — OM divergences cataloged, docs-matrix generates a third function matrix, all v3.1 architectural decisions are recorded as ADRs, and README + CLAUDE.md are updated for v3.1
**Depends on**: Phase 34
**Requirements**: STAT-DOC-01, STAT-DOC-02, STAT-DOC-03, STAT-DOC-04, STAT-DOC-05, STAT-DOC-06
**Success Criteria** (what must be TRUE):

  1. `just docs-matrix` regenerates three function matrices (`docs/hp41cv-function-matrix.md`, `docs/hp41-math1-function-matrix.md`, `docs/hp41-stat1-function-matrix.md`) from their respective JSON source files without error; `just docs-matrix-check` exits 0 in CI (no drift)
  2. `docs/hp41-stat1-divergences.md` contains a numbered three-bucket catalog covering at minimum: (a) OM divergences (XROM ID 2 vs hardware numbering, `MATH_1.id = 7` discrepancy lock), (b) emulator extensions (RAND/SEED as bonus utility, `rand_seed` serde shape), (c) behavioral policies (distribution convergence criteria, mnemonic shadowing guard)
  3. At least two new ADRs exist in `docs/adr/` covering: (a) RNG-state placement (`rand_seed` on CalcState with non-skip serde — the only v3.1 exception to the transient-field pattern), (b) distribution-primitive policy (AS 239/63/241 hand-coded, `statrs` rejected)
  4. `docs/architecture-history.md` contains a v3.1 phase narrative parallel to the v3.0 additions; CLAUDE.md `### v3.1 additions` block describes the STAT_1 XROM module, new Op count, and `rand_seed` serde exception

**Plans**: 4 plans
**UI hint**: no
Plans:

**Wave 1**

- [x] 35-01-PLAN.md — Tooling + matrix regeneration + 33-SPEC-AMENDMENT: scripts/docs-matrix basename dispatch 4-line extension + justfile third invocation + docs/hp41-stat1-function-matrix.md generation + .planning/phases/33-.../33-SPEC-AMENDMENT.md 6-row drift table (STAT-DOC-01, STAT-DOC-02, STAT-DOC-03)
- [x] 35-02-PLAN.md — docs/hp41-stat1-divergences.md three-bucket catalog with ≥ 11 D-35-NN entries (6 oracle drifts in bucket 3 cross-referencing 33-SPEC-AMENDMENT.md + RAND/SEED + ΣPOLYP DEGREE=? in bucket 2 + ΣTSTAT pooled-only + XROM-7 vs XROM-2 prefix + math1/ freeze second carve-out in bucket 3) (STAT-DOC-03)
- [x] 35-03-PLAN.md — 5 new ADRs in docs/adr/v3.1-001..005-*.md (RNG state placement, distribution primitives policy + Free42 disclaim, ANOVA register layout, math1/ freeze second carve-out, ModalProgram::Stat1 enum extension) — long-form D-30.6 template, lock date 2026-05-22 (STAT-DOC-04)

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 35-04-PLAN.md — Narrative docs (Wave 2, depends on 35-03): README v3.1 soft-claim + matrix link, CLAUDE.md FIRST-EVER ### v3.1 additions block + math1/ freeze carve-out amendment gated by ADR-v3.1-004, PROJECT.md Shipped/Current focus update, docs/architecture-history.md v3.1 narrative section, .planning/MILESTONES.md v3.1 stub (STAT-DOC-05, STAT-DOC-06)

### Phase 36: hp41-gui — GUI Integration

**Goal**: Users can invoke all Stat 1 Pac programs from the GUI — CATALOG 2 shows "STAT 1B", the `?` overlay has a "Stat 1 Pac (XROM 2)" section, program listings display correct names, and iterative-quantile paths are cancellable via R/S
**Depends on**: Phase 35
**Requirements**: STAT-GUI-01, STAT-GUI-02, STAT-GUI-03, STAT-GUI-04, STAT-GUI-05
**Success Criteria** (what must be TRUE):

  1. `XEQ "ΣNORMD"` entered from the GUI keyboard (via the XEQ modal) returns Q(1.96) ≈ 0.0250 displayed on the LCD — the GUI prgm_display.rs Op variant arms are exhaustive and compile without `_ =>` catch-all
  2. CATALOG 2 displays "STAT 1B" alongside "MATH 1A" — both XROM module entries are visible in the GUI
  3. The GUI `?` overlay shows a "Stat 1 Pac (XROM 2)" section with all 14 Stat 1 entry-point mnemonics; search returns Stat 1 results alongside built-in and Math Pac I results
  4. A running ΣNORMD inverse computation (iterative quantile path) can be cancelled by pressing R/S — the `request_cancel` `Arc<AtomicBool>` check fires within the iterative loop (per-loop check reusing the v3.0 Pitfall 11 pattern)

**Plans**: TBD
**UI hint**: yes

### Phase 37: Test Hardening & Quality Gates

**Goal**: All v3.1 quality gates hold — `hp41-core` coverage does not regress below the v3.0 baseline, Stat 1 Pac numerical accuracy meets the two-level tolerance discipline, backward-compat migration is verified, and no Free42 contamination is detectable
**Depends on**: Phase 36
**Requirements**: STAT-QUAL-01, STAT-QUAL-02, STAT-QUAL-03, STAT-QUAL-04, STAT-QUAL-05, STAT-QUAL-06, STAT-QUAL-07, STAT-QUAL-08, STAT-QUAL-09, STAT-QUAL-10, STAT-QUAL-11
**Success Criteria** (what must be TRUE):

  1. `just ci` reports `hp41-core` line coverage ≥ 95.39 % and region coverage ≥ 94.26 % (v3.0 baseline preserved); all `hp41-core/src/ops/stat1/*.rs` files individually report ≥ 90 % coverage
  2. `numerical_accuracy.rs` passes at ≥ 98 % with Stat 1 oracle cases added (scipy.stats reference values); v1.x 503-case floor 498/503 and v3.0 768-case floor 763/768 both preserved
  3. A v3.0 save file with `"xrom_modules": 1` loads into v3.1 without error and the emulator correctly activates Stat 1 (bit 1 set via startup migration); `v30_save_loads_with_stat1_off` round-trip test asserts this
  4. `stat1_rand_determinism.rs` verifies that the RNG sequence is identical after a serde save/load cycle — seed survives persistence
  5. E2E smoke (`ci-gui.yml::e2e-linux`) includes at least one Stat 1 Pac workflow: `XEQ "ΣNORMD"` with x = 1.96 returns Q ≈ 0.0250 on the GUI LCD (Ubuntu, WebdriverIO)

**Plans**: TBD
**UI hint**: no

---

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1–8 | v1.0 | 45/45 | Complete | 2026-05-08 |
| 9–12 | v1.1 | 14/14 | Complete | 2026-05-09 |
| 13–18 | v2.0 | 19/19 | Complete | 2026-05-10 |
| — | v2.1 | quick tasks | Complete | 2026-05-13 |
| 20–27 | v2.2 | 26/26 | Complete | 2026-05-15 |
| 28–32 | v3.0 | 31/31 | Complete | 2026-05-20 |
| 33 | v3.1 | 9/9 | Complete    | 2026-05-22 |
| 34 | v3.1 | 2/2 | Complete   | 2026-05-23 |
| 35 | v3.1 | 4/4 | Complete    | 2026-05-23 |
| 36 | v3.1 | 0/0 | Not started | - |
| 37 | v3.1 | 0/0 | Not started | - |

---

*Last updated: 2026-05-23 — Phase 34 planned (2 plans authored per D-34.2; STAT-CLI-01..05 all addressed).*
