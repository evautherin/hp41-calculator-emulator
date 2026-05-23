# Phase 35: Documentation & ADRs - Context

**Gathered:** 2026-05-23
**Status:** Ready for planning

<domain>
## Phase Boundary

`docs/` + `scripts/docs-matrix/` (justfile-only) + repo-root briefing files (`README.md`, `PROJECT.md`, `CLAUDE.md`) + `.planning/` SPEC reconciliation. Phase 35 is the v3.1 documentation lock — Stat 1 Pac narrative is sealed across every surface a downstream reader, archaeologist, or future-pac author may land on. Six concrete deliverables (mirroring v3.0 Phase 30 structure verbatim, with two v3.1-specific additions):

1. **`scripts/docs-matrix` three-input extension.** Binary signature stays 1-in/1-out per D-30.1 precedent — justfile recipes `docs-matrix` and `docs-matrix-check` gain a **third** `cargo run` invocation for `hp41-stat1-functions.json → hp41-stat1-function-matrix.md`. Existing two invocations (cv + math1) stay bit-for-bit unchanged. The renderer's `has_xrom` conditional (already present at `scripts/docs-matrix/src/main.rs:103`) automatically emits the XROM column for the new file since every Stat 1 JSON entry carries `xrom: { module: "Stat 1", module_id: 2, function_id: N }` (Phase 34 D-34.1).

2. **`docs/hp41-stat1-function-matrix.md` (new file).** Generated from `docs/hp41-stat1-functions.json` (26 entries, authored Phase 34) via `just docs-matrix`. Schema mirrors `hp41-math1-function-matrix.md` (8-column with XROM); category sort order mirrors D-34.1 (Stat1 Univariate / ANOVA / Regression / Hypothesis / Nonparam / Distributions / RNG). README v3.1 soft-claim links specifically to this file (D-35.3).

3. **`docs/hp41-stat1-divergences.md` (new file).** Three-bucket catalog per D-30.4 precedent: OM Divergences / Emulator Extensions / Behavioral Policies. 5-field entry shape per D-30.5 precedent (OM citation / Our behavior / OM behavior / Rationale / See). Entries numbered `D-35-NN` per D-35.4 (phase-origin scheme, parallel to math1's D-30-NN). Initial entries include the 5–6 SPEC oracle drifts per D-35.1, the RAND/SEED emulator extension claim (D-33.4), the XROM-7 vs XROM-2 prefix-convention lock, the ΣTSTAT pooled-only policy (Phase 33 SPEC Req. 25), the math1/ freeze second carve-out as a behavioral-policy entry cross-referencing ADR-v3.1-004 (D-35.2), and the ΣPOLYP DEGREE=? emulator-convention transcription.

4. **Five ADRs in `docs/adr/`** per D-35.2 — long-form template per D-30.6 (`# ADR-NNN: Title` / `## Status` / `## Context` / `## Decision` / `## Consequences` / `## Alternatives Considered` / `## Footnotes / References`):
   - `docs/adr/v3.1-001-rng-state-placement.md` — `rand_seed: HpNum` on CalcState with `#[serde(default)]` WITHOUT `skip` (the only v3.1 exception to the transient-field pattern; STAT-RNG-03 lock; D-33.4 / D-33.4a)
   - `docs/adr/v3.1-002-distribution-primitives-policy.md` — `statrs` rejection + Acklam/AS 241 + AS 239 + AS 63 hand-coded ~140 LOC in `stat1/distributions.rs`; rationale cites SUMMARY.md research finding + last-digit-OM-divergence risk of f64 conversion shims; ~6 oracle tuples per primitive validated against scipy.stats inline-constant
   - `docs/adr/v3.1-003-anova-register-layout.md` — OM 00041-90030 "Storage Registers" transcribed into `stat1/mod.rs` `//!` header + named consts (P21 mitigation, locked Phase 33 Plan 33-00). Cites NPS document for ΣMMTUG / ΣAOVONE / ΣMLRXY / ΣCTKKK indices.
   - `docs/adr/v3.1-004-math1-freeze-second-carve-out.md` — Frozen-invariant amendment: `hp41-core/src/ops/math1/` carved out a SECOND time for `modal.rs` only (D-33.3b user-confirmed during `/gsd-plan-phase 33`). Documents the 3-arm dispatch wiring + `Stat1Step` lives in new `hp41-core/src/ops/stat1/modal.rs` so the math1/modal.rs delta stays ~8 lines of pure dispatch. CLAUDE.md "Frozen Invariants → Core engine" updated in lock-step per STAT-DOC-05 / SPEC.md Req. 42.
   - `docs/adr/v3.1-005-modalprogram-stat1-enum-extension.md` — Architectural lock: `ModalProgram::Stat1(Stat1Step)` enum variant added alongside the 6 v3.0 Math 1 variants (Matrix / Solve / Poly / Integ / Difeq / Four). Quotes the parallel `Stat1ModalProgram` rejection from 33-CONTEXT.md D-33.3b verbatim — 8-line freeze amendment beats ~80-line cross-frontend duplication.

5. **`33-SPEC-AMENDMENT.md` + per-drift `D-35-NN` entries** per D-35.1 (history-preserving reconciliation strategy). Document name: `.planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-SPEC-AMENDMENT.md`. Frontmatter cites the original SPEC.md line + the scipy-correct value + the test file:line that asserts the corrected value. Six drifts queued:
   - ΣNORMD CDF tolerance band 1e-9 → 1e-5 (rust_decimal A&S 6 limit)
   - ΣAOVONE F-ratio oracle 100.0 → 50.0 (scipy-correct)
   - ΣSPEAR rank correlation ρ_s 0.7 → 0.8 (scipy-correct)
   - ΣEFXSQ χ² calibration drift
   - ΣBSTAT CV 0.4083 → 0.5270 (scipy-correct)
   - ΣTSTAT deep-tail Student-t p (AS 63 precision limit)
   Each drift gets a matching `D-35-NN` entry in `docs/hp41-stat1-divergences.md` so users browsing the catalog see the reconciliation alongside the OM divergences and policies.

6. **README v3.1 soft-claim + `### v3.1 additions` block in CLAUDE.md + v3.1 narrative in `docs/architecture-history.md` + `### v3.1 additions` block in PROJECT.md** per D-35.3 + D-30.8 incremental-population precedent. README soft-claim format: `"Stat 1 Pac behavioral emulation (13 programs, 26 XEQ entry points, RAND/SEED extension, documented divergences)"` + link to `docs/hp41-stat1-function-matrix.md`. Hard-claim graduation deferred to Phase 37 conditional on STAT-QUAL-04 (numerical_accuracy.rs ≥ 98 % with Stat 1 cases) + STAT-QUAL-11 (E2E smoke extended) — identical discipline to v3.0 D-30.9 → D-32.5 graduation pattern. CLAUDE.md `### v3.1 additions (Stat 1 Pac Emulation, Phases 33–35 — 36–37 IN PROGRESS)` block fully populates Phase 33 / 34 / 35 subsections; Phase 36 / 37 stub-headers with `(in progress)` markers. `architecture-history.md` v3.1 narrative parallels the v3.0 §"v3.0 additions" structure (one subsection per phase, OM-citation discipline).

**Plan 33-09 follow-up (deferred Info findings from Phase 33 code review):** the 5 Info findings (IN-01..IN-05) deferred per `/gsd:code-review --fix` default policy land in stat1-divergences.md as `D-35-NN` Behavioral Policies entries OR in the matching ADR's Footnotes section, as fits.

**In scope:**
- `scripts/docs-matrix` extension via justfile third invocation (binary unchanged)
- `docs/hp41-stat1-function-matrix.md` (new; ~26 entries; 8-col with XROM column)
- `docs/hp41-stat1-divergences.md` (new; 3-bucket catalog; `D-35-NN` numbering; 5-field entries)
- `docs/adr/v3.1-{001,002,003,004,005}-*.md` (5 new ADRs, long-form template)
- `.planning/phases/33-…/33-SPEC-AMENDMENT.md` (history-preserving SPEC supplement; 6 oracle-drift reconciliations)
- `README.md` v3.1 soft-claim + matrix link
- `CLAUDE.md` `### v3.1 additions (Stat 1 Pac Emulation, Phases 33–35 — 36–37 IN PROGRESS)` block
- `CLAUDE.md` `## Frozen Invariants → Core engine` amendment listing `modal.rs` alongside `xrom.rs` as carved-out files (gating per ADR-v3.1-004)
- `PROJECT.md` "Shipped" / "Current focus" line updates + `### v3.1 additions` block (D-30.8 incremental pattern)
- `docs/architecture-history.md` v3.1 phase narrative (Phases 33 / 34 / 35 populated; 36 / 37 stub)
- `.planning/MILESTONES.md` v3.1 milestone-summary one-liner stub (full entry after Phase 37 ship via `/gsd-complete-milestone`)

**Out of scope (explicit):**
- Any `hp41-core/src/`, `hp41-cli/src/`, or `hp41-gui/src-tauri/src/` source changes — Phase 35 is documentation-only
- `docs/hp41-stat1-functions.json` modification — Phase 34 authored it (26 entries, D-34.1 categories); Phase 35 consumes it read-only as the matrix-regeneration input and as the source-of-truth for stat1-divergences entry order
- `docs/hp41cv-function-matrix.md` or `docs/hp41-math1-function-matrix.md` regeneration — both files are current; Phase 35 does NOT re-render them (the v2.2 + v3.0 paths are unchanged and the `just docs-matrix-check` CI gate confirms this)
- New XROM-column work in the hp41cv matrix — XROM column is XROM-pac-only (Math 1 Pac matrix has it; v2.2 baseline matrix doesn't, by D-30.3 lock)
- ADR-v3.0-001..005 modification — Phase 30 / Phase 28 shipped them; Phase 35 references them in ADR-v3.1-004 (freeze carve-out) and ADR-v3.1-005 (ModalProgram::Stat1) Footnotes only
- `33-SPEC.md` in-place editing per D-35.1 (history-preserving choice — supplement, not revisionism)
- Hard-claim "feature-complete Stat 1 Pac behavioral emulation per OM 00041-90030" in README — deferred to Phase 37 per D-35.3
- Phase 36 GUI subsection content in the v3.1 additions block — only the stub header `(in progress)` lands in Phase 35
- Phase 37 metrics in the v3.1 additions block — populated at Phase 37 ship-time only
- `hp41-gui/` files of any kind (Phase 36)
- New test files of any kind (Phase 37 test-hardening territory)
- `numerical_accuracy.rs` extension with Stat 1 cases — STAT-QUAL-04 / Phase 37
- E2E smoke extension — STAT-QUAL-11 / Phase 37
- `lint_stat1_assertions.rs` / `stat1_op_test_count.rs` / `xrom_shadowing.rs` STAT_1 extension — STAT-QUAL-06..08 / Phase 37
- `scripts/check-free42-contamination.sh` — already extended in Phase 33 Plan 33-00 to 18 tokens
- Signed binary releases — deferred to v3.1.x / v3.2 per PROJECT.md lock 2026-05-21

**Mandated by ROADMAP cross-cutting constraints + CLAUDE.md frozen invariants:**
- **SC-4 invariant** trivially preserved — Phase 35 touches `docs/` + `scripts/docs-matrix/` (justfile only) + repo-root markdown files; no `hp41-gui/src-tauri/src/op_*` functions added.
- **`#![deny(clippy::unwrap_used)]`** continues to apply in `hp41-core`; Phase 35 does not touch core, trivially preserved.
- **MSRV 1.88** unchanged. Zero new runtime or dev-dependencies.
- **Citation provenance (Pitfall 18)**: every divergence-doc entry and ADR write-up MUST cite OM page-and-example, NPS document section, MoHPC URL, or Mike Sebastian forensic page; no uncited assertions. Tighter than v3.0 Phase 30 because oracle drifts add scipy-citation discipline alongside OM citations.
- **Free42 GPL contamination guard (Pitfall 19)**: ADR-v3.1-002 (distribution primitives policy) explicitly disclaims Free42 as a source for AS 239 / AS 63 / Acklam algorithms — independently re-derived from primary sources (Cody, Wichura, NPS). The CI guard already lands (extended Phase 33 Plan 33-00); Phase 35 documents the policy.
- **JSON canonical data flow** (CLAUDE.md): the matrix regeneration extension preserves the third source-of-truth pattern (`hp41cv-functions.json` + `hp41-math1-functions.json` + `hp41-stat1-functions.json`); each JSON's matrix lives at the parallel path-shape.

</domain>

<decisions>
## Implementation Decisions

### Already locked in PROJECT.md / REQUIREMENTS.md / 33-CONTEXT.md / 33-SPEC.md / 34-CONTEXT.md (carried forward — NOT re-decided here)

- **D-30.1 (carried forward as structural template):** `scripts/docs-matrix` binary stays 1-in/1-out. Phase 35 adds a THIRD justfile invocation (cv + math1 + stat1); binary's existing `has_xrom` conditional (line 103) handles the new column automatically.
- **D-30.2 (carried forward):** Three separate matrix files. `hp41cv-function-matrix.md` (130 entries, no XROM column, unchanged), `hp41-math1-function-matrix.md` (~55 entries, XROM column, unchanged), `hp41-stat1-function-matrix.md` (~26 entries, XROM column, NEW).
- **D-30.3 (carried forward):** XROM column emitted conditionally per `has_xrom` check; Stat 1 JSON entries all carry `xrom` block per D-34.1, so column renders for every row.
- **D-30.4 (carried forward as structural template):** Three-bucket divergence catalog — OM Divergences / Emulator Extensions / Behavioral Policies.
- **D-30.5 (carried forward as structural template):** 5-field entry shape — OM citation / Our behavior / OM behavior / Rationale / See.
- **D-30.6 (carried forward as ADR template):** Long-form template — Status / Context / Decision / Consequences / Alternatives Considered / Footnotes-References.
- **D-30.7 (carried forward):** Rejected alternatives quote CONTEXT.md verbatim + add OM/community citations; no rewrites of original deliberation.
- **D-30.8 (carried forward):** Add v3.1 additions block to CLAUDE.md NOW with Phase 33 / 34 / 35 sub-sections fully populated; Phase 36 / 37 stub-headers with `(in progress)` markers; future ships fill their own subsection.
- **D-30.9 (carried forward as soft-claim-then-graduate pattern):** README soft-claim now, hard-claim graduation gates on Phase 37 (mirrors v3.0 Phase 30 → Phase 32 cadence). NEW lock D-35.3 confirms this is the v3.1 choice.
- **D-33.1 / 33-SPEC.md:** 7 LOCKED Stat 1 Pac requirements (OM register layouts for ΣMMTUG / ΣAOVONE / ΣMLRXY / ΣCTKKK; ΣPOLYP `DEGREE=?` prompt; ΣCHISQD ν entry convention; ΣTSTAT pooled-only; RAND ROM-presence; quantile convergence criteria; Free42 stats-domain identifiers) are downstream ground truth. Phase 35 ADRs cite these locks.
- **D-33.3 / D-33.3a:** math1/ freeze first carve-out (xrom.rs only); Phase 33 lock. Phase 35 ADR-v3.1-004 covers this.
- **D-33.3b (user-confirmed Phase 33 plan-phase):** math1/ freeze SECOND carve-out for modal.rs only — 8-line dispatch wiring; `Stat1Step` lives in `stat1/modal.rs`. Phase 35 ADR-v3.1-004 + CLAUDE.md amendment lock this.
- **D-33.4 / D-33.4a:** RAND / SEED as v3.1 emulator extension (NOT part of "feature-complete per OM 00041-90030" claim); LCG formula `r_{n+1} = FRC(9821·r_n + 0.211327)` per NPS p. 21 / Don Malm community convention. Phase 35 stat1-divergences entry + ADR-v3.1-001.
- **D-33.7:** `CalcState::migrate_after_load()` in `hp41-core/src/state.rs` — single source of truth, called by both CLI and GUI persistence layers. Phase 35 references this in ADR-v3.1-001 (RNG state placement context).
- **D-33.8:** STAT-QUAL-09 reassigned Phase 37 → Phase 33 (Plan 33-00). Phase 35 architecture-history.md narrative cites this reassignment under the Phase 33 sub-section.
- **D-34.1 / D-34.3:** stat1-functions.json category convention (7 per-family categories) + surgical inline `divergences` field (RAND/SEED, ΣTSTAT, ΣPOLYP only); full taxonomy lives in stat1-divergences.md per D-35.4. Phase 35 stat1-divergences entries DO NOT duplicate the JSON inline divergences — they cite them with cross-reference.

### Discussed and decided in this session (D-35.1 — D-35.4)

#### SPEC oracle drift reconciliation strategy

- **D-35.1: History-preserving reconciliation — `33-SPEC-AMENDMENT.md` supplement + per-drift `D-35-NN` entries in `docs/hp41-stat1-divergences.md`.** Both surfaces ship. The original `33-SPEC.md` stays frozen (planning-phase archaeology preserved verbatim — future readers see the original oracle assumptions). The amendment document is a sibling file in the Phase 33 directory with a row-per-drift table: `SPEC.md line# | Original oracle | Scipy-correct value | Test file:line asserting corrected value | Why drifted | D-35-NN cross-ref`. Each drift ALSO gets a numbered divergence entry in `stat1-divergences.md` (bucket-3 Behavioral Policies, since these are mathematical-ground-truth corrections, not OM divergences) so users browsing the catalog see the reconciliation alongside the OM divergences.
  - **Why:** The two-surface choice resolves the central tension between "SPEC.md should be authoritative" and "planning archaeology has educational value". Supplement preserves both. Tests are ground truth either way; the docs follow. Rejected in-place SPEC.md revision because it rewrites history (future archaeologists wouldn't see that the original planning-phase oracle estimates can drift from scipy — that's a real lesson worth preserving). Rejected stat1-divergences.md-only because SPEC.md would then be structurally false-on-its-face for those 6 numbers and a reader who lands on SPEC.md without checking divergences would walk away with the wrong reference. Rejected test-comment-only because it defeats the Phase 35 docs purpose.

#### ADR scope and count

- **D-35.2: 5 ADRs — 3 STAT-DOC-04-mandated + 2 architecture-locking (freeze carve-out + ModalProgram::Stat1).** Full list:
  - `docs/adr/v3.1-001-rng-state-placement.md` — `rand_seed: HpNum` on CalcState; `#[serde(default)]` WITHOUT `skip`; STAT-RNG-03 lock; rationale cites D-33.4 / D-33.7.
  - `docs/adr/v3.1-002-distribution-primitives-policy.md` — `statrs` rejection + Acklam / AS 239 / AS 63 hand-coded; rationale cites runtime-dep cost + last-digit-OM-divergence risk; ~140 LOC `stat1/distributions.rs`; ≥ 6 oracle tuples per primitive validated against scipy.stats. Explicitly disclaims Free42 as a source per Pitfall 19.
  - `docs/adr/v3.1-003-anova-register-layout.md` — OM 00041-90030 "Storage Registers" transcription policy + per-Op named consts in `stat1/mod.rs`; P21 (silent-wrong-answer trap) mitigation; cites NPS document and OM page.
  - `docs/adr/v3.1-004-math1-freeze-second-carve-out.md` — Frozen-invariant amendment; `hp41-core/src/ops/math1/` carved out twice (xrom.rs per D-33.3, modal.rs per D-33.3b); rationale cites the rejected ~80-line cross-frontend-duplication parallel-enum option. CLAUDE.md "Frozen Invariants → Core engine" amendment is gated by this ADR's Status: Locked line.
  - `docs/adr/v3.1-005-modalprogram-stat1-enum-extension.md` — `ModalProgram::Stat1(Stat1Step)` enum extension lock; cites 33-CONTEXT.md D-33.3b rejected alternative verbatim per D-30.7; rationale: 8-line freeze amendment beats infrastructure duplication.
  - **Why 5 vs 3:** STAT-DOC-04 mandates a minimum 3, but D-33.3b (modal.rs freeze amendment) and ModalProgram::Stat1 were BOTH real architectural locks made during Phase 33 plan-phase that survive v3.1 unchanged. Not ADR-recording them loses provenance for future v3.2+ pacs (Time, Advantage) that will face the same "extend the enum vs spawn a parallel enum" choice. Mirrors v3.0's 5-ADR scope (003+004 in Phase 28, 001+002+005 in Phase 30); 5 is the right scale for a major XROM module. Rejected 4 (skip ModalProgram) because the ModalProgram extension is the load-bearing infrastructure choice that makes v3.2+ pacs cheap. Rejected 6+ (XROM-prefix convention) because that's a behavioral policy, not an architectural lock — belongs in stat1-divergences.md as bucket-3.

#### README v3.1 claim strength + graduation gating

- **D-35.3: Mirror v3.0 — soft-claim now (Phase 35), hard-claim graduation gates on Phase 37 (after STAT-QUAL-04 + STAT-QUAL-11 close).** README soft-claim wording:
  ```
  - Stat 1 Pac behavioral emulation (13 programs, 26 XEQ entry points,
    RAND/SEED extension, documented divergences)
  ```
  Plus link `docs/hp41-stat1-function-matrix.md` under "See" or equivalent navigation. Hard-claim graduation pattern mirrors v3.0 D-30.9 → D-32.5 exactly: Phase 37's `gsd-ship` rewrites the line to `"Stat 1 Pac feature-complete per Owner's Manual 00041-90030"` conditional on STAT-QUAL-04 (numerical_accuracy.rs Stat 1 cases ≥ 98 % pass) + STAT-QUAL-11 (E2E smoke extended with a Stat 1 workflow). Phase 33 already preserves the 95.39 % coverage gate (STAT-QUAL-01) but the user-facing accuracy + workflow assertions live in Phase 37; that's the right gate.
  - **Why:** identical discipline to v3.0; reader learning curve stays consistent across milestones; conservative claim is accurate at this point + clear upgrade path at Phase 37. Rejected hard-claim-now because E2E smoke + Stat 1 numerical-accuracy cases haven't landed yet; overclaim risk if Phase 37 surfaces a gap. Rejected skip-soft-claim because user-facing v3.1 invisibility between Phase 35 and Phase 37 would lose discoverability the soft-claim provides. Rejected dependencies-pending-block because v3.0 didn't do it and it diverges from the established cadence.

#### Divergence catalog numbering scheme

- **D-35.4: Stat 1 divergence entries use `D-35-NN` (phase-origin scheme, parallel to math1's `D-30-NN`).** First entries are `D-35-01`, `D-35-02`, …; ordering follows D-30.5 thematic-by-bucket convention (OM Divergences first, then Emulator Extensions, then Behavioral Policies; within each bucket, chronological by Phase 33 / 34 decision sequence + the 6 oracle drifts ordered by SPEC.md original-line-number).
  - **Why:** parallelism with math1-divergences.md = zero cognitive load for readers; phase number always recoverable from ID; cross-doc cites stay short (`see D-35-07`). Rejected module-prefix (D-S-NN / D-STAT1-NN) because it breaks parallelism — would force a retro-rename of math1's D-30-NN to D-M-NN for consistency, or accept divergent schemes between docs (worst of both). Rejected hybrid (ship D-35-NN now, retro-rename if a third pac lands) because it defers a rename that may never happen + if it does happen it's a doc-wide find-replace cascading into external citations. v3.0 Phase 30 set the phase-origin precedent; it has held; Phase 35 extends it. Future v3.2 Time Pac will use `D-XX-NN` where XX is whatever phase locks that pac's documentation (same pattern).

### Claude's Discretion

- **`33-SPEC-AMENDMENT.md` exact location and frontmatter shape:** lives at `.planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-SPEC-AMENDMENT.md` (sibling to `33-SPEC.md`). Frontmatter: title + date + amendment-author + cross-reference to Phase 33 VERIFICATION.md's drift listings. Body: markdown table with columns `SPEC.md row | Original oracle | Scipy-correct value | Test file:line | Drift root cause | D-35-NN cross-ref`. Planner picks table column order; the 6 rows + cross-refs are fixed.
- **ADR-v3.1-001..005 `**Status:** Locked YYYY-MM-DD`** dates: pick the date the underlying decision was actually locked (NOT the Phase 35 ship date), matching v3.0 Phase 30 D-30 discretion. For -001 (RNG-state placement): 2026-05-22 (Phase 33 discuss-phase D-33.4). For -002 (distribution primitives): 2026-05-22 (Phase 33 discuss-phase, but cite SUMMARY.md research date 2026-05-21 as the original source). For -003 (ANOVA register layout): 2026-05-22 (Phase 33 Plan 33-00 + SPEC.md Req. 30..35 lock). For -004 (math1/ freeze second carve-out): 2026-05-22 (Phase 33 plan-phase user-confirmed). For -005 (ModalProgram::Stat1): 2026-05-22 (same plan-phase session).
- **`**Owner:**` lines for ADRs:** use Phase + Plan numbers where the decision was operationally locked: ADR-v3.1-001 owner "Plan 33-01 + Plan 33-08"; ADR-v3.1-002 owner "Plan 33-02"; ADR-v3.1-003 owner "Plan 33-00 + Plan 33-06"; ADR-v3.1-004 owner "Phase 33 plan-phase (D-33.3b)"; ADR-v3.1-005 owner "Phase 33 plan-phase (D-33.3b)".
- **Plan slicing (3 vs 4):** ROADMAP estimates 3–4 plans. Recommendation: 4 plans for clean blast-radius separation.
  - Plan 35-01 — Tooling + matrix regeneration + SPEC amendment: justfile third invocation + `hp41-stat1-function-matrix.md` regeneration + `33-SPEC-AMENDMENT.md` + `just docs-matrix-check` CI gate confirmation. Tight, low-risk, ships green.
  - Plan 35-02 — `docs/hp41-stat1-divergences.md` full 3-bucket catalog with all `D-35-NN` entries (including 6 oracle-drift entries cross-referencing the amendment + RAND/SEED + XROM-prefix convention + ΣTSTAT pooled-only + ΣPOLYP UX + math1/ freeze carve-out cross-ref to ADR-v3.1-004 + any IN-01..IN-05 follow-ups that fit).
  - Plan 35-03 — 5 ADRs in `docs/adr/v3.1-001..005-*.md`, long-form template per D-30.6, Pitfall-18 citations throughout, Pitfall-19 disclaim in ADR-v3.1-002.
  - Plan 35-04 — Narrative docs: README soft-claim + matrix link, CLAUDE.md `### v3.1 additions` block + `## Frozen Invariants → Core engine` amendment gated by ADR-v3.1-004, PROJECT.md additions block + Shipped/Current-focus updates, `docs/architecture-history.md` v3.1 narrative (Phases 33 / 34 / 35 populated + 36 / 37 stub), `.planning/MILESTONES.md` v3.1 stub.
  - Rejected 3 plans because Plan 35-04 would otherwise mix the architecture-history narrative (which references all 5 ADRs) with the ADR authoring itself, creating an ordering constraint mid-plan. Rejected 2 plans because the tooling work (Plan 35-01) is a different blast-radius from the narrative work and should ship separately.
  - Planner has discretion to fold 35-01 + 35-02 if the amendment is short, or to split 35-04 into "repo-root markdown" + "architecture-history.md" if either grows beyond reasonable plan size. 4-plan shape is the recommended starting point.
- **`docs/hp41-stat1-divergences.md` preamble content:** copy the math1-divergences.md "How to Use This Document" preamble verbatim (or near-verbatim) for consistency; planner picks any v3.1-specific clarifying note (e.g., "Stat 1 Pac is the second XROM module; numbering follows phase-origin convention D-35-NN per Phase 35 CONTEXT D-35.4").
- **`### v3.1 additions` block insertion point in CLAUDE.md:** immediately after the `### v3.0 additions (Math Pac I Emulation, Phases 28–32)` block (line 117 region in current CLAUDE.md). Planner picks the exact line; structure mirrors v3.0 block (one sub-section per phase + tail note about preserved invariants).
- **`### v3.1 additions` block in PROJECT.md:** D-30.8 Claude's Discretion suggested PROJECT.md gets only Shipped/Current-focus line updates while CLAUDE.md carries the full additions block. Recommendation: hold that pattern — PROJECT.md gets concise milestone lines (Phase 33 / 34 / 35 ship dates + the Stat 1 Pac soft-claim wording) but the v3.1 architecture-decisions detail lives in CLAUDE.md. Planner picks final shape.
- **`docs/architecture-history.md` v3.1 narrative sub-section depth:** mirror v3.0 §"v3.0 additions" depth (one sub-section per phase with 2–4 paragraph narrative each; cite OM pages where relevant; explicitly call out cross-phase trap mitigations like P21 / P22 / P27). Phase 36 / 37 sub-sections appear as stub headers + `(in progress)` markers; their narrative fills in at ship time.
- **CLAUDE.md `## Frozen Invariants → Core engine` amendment exact wording:** the math1/ freeze section currently locks the entire directory frozen since Plan 25-01. The amendment must add: "Exceptions: `xrom.rs` (D-33.3 / ADR-v3.1-004 — XROM registry for v3.1+ extension; bit-1 stub was always intended for v3.1+) and `modal.rs` (D-33.3b / ADR-v3.1-004 — ~8-line dispatch wiring for `ModalProgram::Stat1` variant; `Stat1Step` semantics live in the new `stat1/modal.rs` so no Stat 1 Pac code leaks into the frozen module)." Planner picks exact wording; the substance is fixed.
- **Whether the 5 deferred Info findings (IN-01..IN-05) from Phase 33 code review land in stat1-divergences.md or in ADR Footnotes:** depends on the content of each finding (if it's a behavioral policy, divergences.md bucket-3; if it's a rationale/historical-context note, the matching ADR's Footnotes). Planner reads the Phase 33 `33-REVIEW.md` deferred section and routes each.
- **README "See" / navigation section structure:** match the existing v2.2 / v3.0 soft-claim link layout. Planner picks; the matrix link itself is fixed (`docs/hp41-stat1-function-matrix.md`).
- **CI gate continuity for `just docs-matrix-check`:** the existing CI step calls `just docs-matrix-check` which after D-35.1 lands runs all three invocations and diffs all three matrix files. No CI workflow file changes required (parallel to D-30 Claude's Discretion).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-level (always-on)

- `.planning/PROJECT.md` — v3.1 milestone scope locked 2026-05-21; "Shipped milestones" line updates land in Phase 35; v3.0 archived block; key decisions ledger
- `.planning/REQUIREMENTS.md` — 66 v3.1 requirements; Phase 35 maps to STAT-DOC-01..06 (rows 223–228 in the traceability table)
- `.planning/ROADMAP.md` — Phase 35 section (lines 132–146) — 4 success criteria + 6 STAT-DOC requirements; cross-cutting constraints carried forward from v3.0 ROADMAP archive
- `.planning/STATE.md` — v3.1 phase overview; "Current focus: Phase 35 — docs + tooling"; carried-forward decisions; critical implementation traps
- `CLAUDE.md` (repo root) — Frozen Invariants section (Core engine math1/ freeze amendment target per ADR-v3.1-004); Tech Stack section; Key Files table; JSON canonical data flow lines 122–129; v3.0 additions block (lines 117 region — Phase 35 inserts `### v3.1 additions` block immediately after)

### Phase 33 (the contract Phase 35 documents + reconciles)

- `.planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-CONTEXT.md` — primary source for ADR-v3.1-001 (D-33.4 RAND/SEED), ADR-v3.1-002 (D-33.5 file layout + research SUMMARY rejection of statrs), ADR-v3.1-003 (D-33.5 stat1/mod.rs OM transcription + Plan 33-00 + Plan 33-06 register-layout-dependent set), ADR-v3.1-004 (D-33.3 + D-33.3a math1/ freeze first carve-out for xrom.rs), ADR-v3.1-005 (D-33.3b math1/ freeze SECOND carve-out + ModalProgram::Stat1 extension lock + rejected ~80-line parallel-enum alternative)
- `.planning/phases/33-…/33-SPEC.md` — 39 LOCKED Stat 1 Pac requirements; Phase 35's 33-SPEC-AMENDMENT.md is a SIBLING file preserving the original (no in-place editing per D-35.1)
- `.planning/phases/33-…/33-VERIFICATION.md` — primary source for the 6 oracle drifts queued for Phase 35 STAT-DOC: ΣNORMD 1e-9→1e-5, ΣAOVONE F=100→50, ΣSPEAR 0.7→0.8, ΣEFXSQ χ², ΣBSTAT CV 0.4083→0.5270, ΣTSTAT deep-tail t. Lines 13–22 + lines 186–187 enumerate them.
- `.planning/phases/33-…/33-REVIEW.md` — 5 deferred Info findings (IN-01..IN-05) candidates for Phase 35 stat1-divergences entries or ADR Footnotes
- `.planning/phases/33-…/33-RESEARCH.md` — Stat 1 Pac behavioral inventory; OM page references for ADR-v3.1-003 citations
- `.planning/phases/33-…/33-PATTERNS.md` — closest-analog file mapping
- `.planning/research/SUMMARY.md` — `statrs` rejection rationale (primary source for ADR-v3.1-002 "Alternatives Considered" section)
- `.planning/research/STACK.md` — additional `statrs` rejection detail + `rust_decimal 1.42` `MathematicalOps` coverage
- `.planning/research/PITFALLS.md` — P18 (citation provenance, applies to every ADR + divergence entry), P19 (Free42 GPL contamination, applies to ADR-v3.1-002), P21 (OM register layout, applies to ADR-v3.1-003), P27 (Free42 stats contamination guard, cross-referenced from ADR-v3.1-002 Footnotes)

### Phase 34 (the canonical JSON Phase 35 consumes read-only)

- `.planning/phases/34-hp41-cli-cli-integration/34-CONTEXT.md` — D-34.1 (7-category convention used in stat1-divergences entry-order chronology), D-34.3 (surgical inline JSON divergences for RAND/SEED + ΣTSTAT + ΣPOLYP — Phase 35 divergence entries CROSS-REFERENCE these, do NOT duplicate the text), D-34.5 (`?` overlay section header convention parallels README link wording)
- `docs/hp41-stat1-functions.json` — 26 entries (Phase 34 Plan 34-01); read-only input to `just docs-matrix` Stat 1 invocation; entry order + categories + per-entry `divergences` field are the source of truth for `hp41-stat1-function-matrix.md` rendering and for the entry-order of Phase 35 stat1-divergences

### v3.0 Phase 30 (the DIRECT analog — STRUCTURE TEMPLATE)

- `.planning/milestones/v3.0-phases/30-documentation-adrs/30-CONTEXT.md` — Phase 30 decisions D-30.1 through D-30.9; structurally identical phase one milestone earlier; Phase 35 reuses ALL nine decisions verbatim (D-30.1 docs-matrix shape; D-30.2 separate matrix files; D-30.3 XROM column conditional; D-30.4 three-bucket catalog; D-30.5 5-field entries; D-30.6 long-form ADR template; D-30.7 verbatim alternative quoting; D-30.8 incremental additions block; D-30.9 soft-claim-then-graduate)
- `.planning/milestones/v3.0-phases/30-documentation-adrs/30-01-PLAN.md` — docs-matrix two-input extension plan (Phase 35 Plan 35-01 mirrors this, swapping two-input → three-input)
- `.planning/milestones/v3.0-phases/30-documentation-adrs/30-02-PLAN.md` — divergences + 3 ADRs plan (Phase 35 Plan 35-02 + 35-03 split this, scaling to 5 ADRs)
- `.planning/milestones/v3.0-phases/30-documentation-adrs/30-03-PLAN.md` — narrative-docs plan (README + CLAUDE.md + PROJECT.md + architecture-history) — Phase 35 Plan 35-04 mirrors this
- `.planning/milestones/v3.0-phases/30-documentation-adrs/30-VERIFICATION.md` — Phase 30 ship-verification template; Phase 35 mirrors the verification dimensions

### Existing docs/ artifacts Phase 35 extends or references

- `docs/hp41-math1-divergences.md` — the structural template Phase 35 mirrors for `hp41-stat1-divergences.md`; preamble + 3-bucket structure + `D-30-NN` numbering precedent
- `docs/hp41-math1-function-matrix.md` — the structural template Phase 35 mirrors for `hp41-stat1-function-matrix.md` (XROM column conditional emission, 8-column header, category-then-op-variant sort)
- `docs/hp41cv-function-matrix.md` — v2.2 baseline; Phase 35 does NOT modify; `just docs-matrix-check` keeps gating
- `docs/hp41-math1-functions.json` — v3.0 baseline; Phase 35 does NOT modify
- `docs/hp41cv-functions.json` — v2.2 baseline; Phase 35 does NOT modify
- `docs/adr/v3.0-001-op-strategy.md` — Op-strategy A precedent referenced from ADR-v3.1-001 (RNG state as Op-variant alternative considered) + ADR-v3.1-005 (ModalProgram extension preserves invariant)
- `docs/adr/v3.0-002-user-callback-policy.md` — strict-reject pattern; referenced from ADR-v3.1-002 (self-contained iteration for distribution quantile loops is the natural extension, NOT user-callback)
- `docs/adr/v3.0-003-inv-epsilon.md` — long-form template structural template (D-30.6 reference) — ADR-v3.1-001..005 mirror this shape
- `docs/adr/v3.0-004-intg-threshold.md` — long-form template structural template; convergence-threshold discipline parallels D-33.1 item 6 (quantile convergence criteria)
- `docs/adr/v3.0-005-json-pipeline.md` — separate-JSON-per-XROM-module pattern; ADR-v3.1-001..005 reference this as the canonical JSON-pipeline lock (D-29.1 / D-34.1 inherit this)
- `docs/architecture-history.md` — v3.0 additions section (lines 117–168 region) is the structural template for Phase 35's new `## v3.1 additions` section; phase-by-phase narrative with OM-citation discipline
- `docs/hp41-overview.md`, `docs/operations-reference.md`, `docs/programming-guide.md`, `docs/keyboard-layout.md` — user-facing docs that may need a one-line "see also Stat 1 Pac" pointer; planner reviews each at write time

### Tooling (the build-data-driven infrastructure Phase 35 extends)

- `scripts/docs-matrix/Cargo.toml` — standalone non-workspace crate (verified empty `[workspace]` stanza in v3.0 Phase 30); Phase 35 does NOT touch this file
- `scripts/docs-matrix/src/main.rs` — single binary, 1-in/1-out signature; Phase 35 does NOT touch this file. The existing `has_xrom` conditional at line 103 + the `basename` title dispatch at lines 70–76 handle the new file automatically once the third entry is added to the dispatch table at lines 72–76.
  - **Discretion note for Plan 35-01:** the basename dispatch at lines 70–76 currently handles `hp41cv-functions.json` and `hp41-math1-functions.json`; planner adds a third branch for `hp41-stat1-functions.json` → `"# HP-41C Stat 1 Pac Function Matrix"`. This is the ONLY binary-side change (and it's a 4-line edit, not a signature change — keeps D-30.1 1-in/1-out intact).
- `justfile` — `docs-matrix` and `docs-matrix-check` recipes; Phase 35 adds a third invocation under each recipe (cv + math1 + stat1)
- `Cargo.lock` (under `scripts/docs-matrix/`) — unchanged

### HP Stat 1 Pac primary sources (HP-copyrighted — DO NOT redistribute; OM-style citations only)

- HP-41C Stat 1 Pac Owner's Manual (HP 00041-90030, 1979) — primary source for every ADR and every OM-divergence entry; cited via OM page numbers in ADR-v3.1-003 (register layout), ADR-v3.1-001 (RAND/SEED if OM-listed), and every divergence entry in bucket-1 (OM Divergences)
- HP-41C Stat 1 Pac Quick Reference Card (HP 00041-90061, 1979) — entry-point catalog; referenced from stat1-divergences XROM-prefix convention entry (mnemonics derive from QRC)
- Naval Postgraduate School NPS55-84-003 (Zehna, 1984, DTIC AD-A140573) — secondary source; cited from ADR-v3.1-001 (RAND/SEED community-convention provenance, p. 21–22), ADR-v3.1-002 (anti-features F/Binomial/Poisson Out-of-Scope justification, p. 42, 49), stat1-divergences ΣTSTAT pooled-only entry (ZS-4/5 evidence)
- `calc.fjk.ch/db/hp41mod.php` — "Statistics Pac 1B" XROM #2 confirmation; referenced from ADR-v3.1-004 / -005 Footnotes for hardware-XROM-ID-2 provenance
- HP-65 User's Library via Don Malm — historical LCG-formula community provenance; cited from ADR-v3.1-001 Footnotes for RAND/SEED community-convention sourcing
- HP-41C Standard Applications (1979) — secondary RAND/SEED LCG confirmation source (p. 24); cited from ADR-v3.1-001 Footnotes alongside NPS and HP-65

### Reference oracles (NOT sources for copying)

- Free42 — `https://thomasokken.com/free42/` and `https://github.com/thomasokken/free42` — PUBLIC GPL source. ADR-v3.1-002 explicitly disclaims Free42 as a source per Pitfall 19 (independently re-derived from primary sources). The contamination-guard CI script (`scripts/check-free42-contamination.sh`) is already extended to 18 tokens per Phase 33 Plan 33-00.
- scipy.stats — used as oracle for Phase 33 numerical_accuracy validation + the 6 oracle-drift corrections (D-35.1). Cited in `33-SPEC-AMENDMENT.md` per-drift rows and in stat1-divergences.md bucket-3 entries for the drifts.
- MoHPC (Museum of HP Calculators) — `https://www.hpmuseum.org/forum/` — community discussion threads cited where they strengthen ADR arguments (Pitfall 18 discretion; no fabricated URLs)
- Mike Sebastian forensic page — `https://www.rskey.org/~mwsebastian/miscprj/forensic.htm` — referenced where it bears on Stat 1 Pac accuracy claims; not all ADRs need it

### Project-local CLAUDE.md guidance

- CLAUDE.md `## Git Workflow` — commits use `/git-workflow:commit --with-skills` only; English-only subject + body
- CLAUDE.md `## Frozen Invariants → Core engine` — math1/ freeze amendment target (Phase 35 Plan 35-04 lands the carve-out clause gated by ADR-v3.1-004 Status: Locked)
- CLAUDE.md `## Frozen Invariants → JSON canonical data flow` — third JSON file extends the pattern; matrix regeneration extension preserves the bidirectional invariant
- CLAUDE.md `### v3.0 additions` block (lines 117–168 region) — structural template for the new `### v3.1 additions` block (Phase 35 Plan 35-04)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`scripts/docs-matrix/src/main.rs:65-76` `render_markdown` basename dispatch** — currently handles cv + math1; Plan 35-01 adds a third `else if basename.ends_with("hp41-stat1-functions.json")` branch with title `"# HP-41C Stat 1 Pac Function Matrix"` and src `` "`docs/hp41-stat1-functions.json`" ``. 4-line edit; signature stays 1-in/1-out per D-30.1.
- **`scripts/docs-matrix/src/main.rs:103-104` `has_xrom` conditional** — already correct for Stat 1: every entry in `hp41-stat1-functions.json` carries `xrom: { module: "Stat 1", module_id: 2, function_id: N }` per D-34.1, so the conditional emits the XROM column automatically. Zero code change.
- **`scripts/docs-matrix/src/main.rs:132-135` `xrom_cell` formatter** — `format!("{} / {}-{}", x.module, x.module_id, x.function_id)` renders as `"Stat 1 / 2-N"` for Stat 1 rows. Zero code change.
- **`justfile` `docs-matrix` + `docs-matrix-check` recipes** — Plan 35-01 adds a third `cargo run --bin docs-matrix --manifest-path scripts/docs-matrix/Cargo.toml -- docs/hp41-stat1-functions.json docs/hp41-stat1-function-matrix.md` line under each recipe (mirrors the existing math1 line that mirrored the cv line).
- **`docs/hp41-math1-divergences.md` preamble + 3-bucket structure** — direct structural template; Plan 35-02 copies the preamble shape, swapping "Math Pac I / OM 00041-90034" → "Stat 1 Pac / OM 00041-90030" and "D-30-NN" → "D-35-NN" in section markers.
- **`docs/adr/v3.0-003-inv-epsilon.md` + `docs/adr/v3.0-004-intg-threshold.md`** — long-form ADR template for ADR-v3.1-001..005 (Plan 35-03). Header metadata block (`# ADR-NNN: Title` / `**Status:** Locked YYYY-MM-DD` / `**Owner:** Plan NN-NN Task X` / `**Requirement refs:**` / `**Downstream consumer:**`) + section sequence (`## Context` / `## Decision` / `## Consequences` / `## Alternatives Considered` / `## Footnotes / References`) — structural template per D-30.6.
- **`docs/adr/v3.0-005-json-pipeline.md`** — direct structural template for ADR-v3.1-002 (distribution primitives policy) since both lock a tooling/library decision with rejected alternatives that need OM/community provenance.
- **CLAUDE.md `### v3.0 additions (Math Pac I Emulation, Phases 28–32)` block (lines 117–168 region)** — structural template for the new `### v3.1 additions (Stat 1 Pac Emulation, Phases 33–35 — 36–37 IN PROGRESS)` block; sub-section-per-phase shape mirrors directly.
- **`docs/architecture-history.md` `## v3.0 additions` section (line 117 region)** — structural template for the new `## v3.1 additions` section; per-phase narrative depth mirrors directly.
- **README.md v3.0 soft-claim line (under `## Features`)** — Plan 35-04 inserts v3.1 soft-claim immediately below it, mirroring the same wording shape ("X behavioral emulation (M programs, N XEQ entry points, …, documented divergences)" + matrix-link).

### Established Patterns

- **JSON-canonical pipeline + hard-build-blocker on malformed JSON** (D-25.16/17/18): Phase 35 preserves this via the third-JSON extension; ADR-v3.1-002 implicitly references this as a precedent for the matrix-regen path.
- **Justfile as task runner, never `cargo` directly** (CLAUDE.md "Never call `cargo` directly in CI or docs"): D-30.1 → D-35.1 enforces this — Plan 35-01 justfile change is the entire matrix-regen delivery surface (binary unchanged).
- **`just docs-matrix-check` CI drift-catch pattern** (v2.2 Pitfall 8 mitigation): Phase 35 extension means ALL THREE matrix files are diffed; if any drifts, CI fails. Planner verifies the existing tmp-file naming doesn't collide across the three invocations (likely solved by using JSON-derived basenames per main.rs line 70).
- **OM citation + Pitfall 18 provenance discipline**: every ADR + divergence entry carries a page-and-example citation. D-35.1 oracle-drift entries additionally carry scipy citations.
- **Phase-only-doc cross-cutting touch** (CLAUDE.md "v2.2 ROADMAP cross-cutting"): Phase 35 follows the v3.0 Phase 30 documentation precedent — touches `docs/`, `scripts/` (justfile only), repo-root markdown files, `.planning/phases/33-…/33-SPEC-AMENDMENT.md`. No Rust source.
- **History-preserving SPEC amendment pattern (NEW with D-35.1)**: when SPEC.md oracle values drift from ground-truth post-implementation, ship a `NN-SPEC-AMENDMENT.md` sibling file (don't edit SPEC.md in place). This is a v3.1 convention; future milestones inherit if/when their SPEC.md oracle estimates similarly drift.

### Integration Points

- **`scripts/docs-matrix/src/main.rs:70-76` basename dispatch:** Plan 35-01 inserts the third `else if` branch. The else-fallthrough (`{json_path}` literal placeholder) is preserved for hypothetical fourth-pac defensiveness.
- **`justfile` `docs-matrix` recipe:** Plan 35-01 appends one `cargo run --bin docs-matrix … docs/hp41-stat1-functions.json docs/hp41-stat1-function-matrix.md` line below the existing math1 invocation. Mirror change for `docs-matrix-check` recipe with the matching diff target.
- **`docs/hp41-stat1-function-matrix.md` (new file):** appears at the same path-shape as `docs/hp41-math1-function-matrix.md`. The first `just docs-matrix` run after Plan 35-01 lands creates it. README v3.1 soft-claim (D-35.3) links to it.
- **`docs/hp41-stat1-divergences.md` (new file):** appears at the same path-shape as `docs/hp41-math1-divergences.md`. Plan 35-02 lands it. Cross-references back to `33-SPEC-AMENDMENT.md` for the 6 oracle-drift entries.
- **`docs/adr/v3.1-001..005-*.md` (5 new files):** appear at the same path-shape as `docs/adr/v3.0-001..005-*.md`. Plan 35-03 lands all 5 in one commit (or split if any individual ADR exceeds ~12K; planner decides).
- **`.planning/phases/33-…/33-SPEC-AMENDMENT.md` (new file):** sibling to `33-SPEC.md`. Plan 35-01 lands it alongside the matrix regeneration so the 6 oracle drifts are documented in the same commit as the matrix that depends on (indirectly) the corrected oracle values.
- **`CLAUDE.md` insertion points (Plan 35-04):**
  - `### v3.1 additions` block: immediately after `### v3.0 additions (Math Pac I Emulation, Phases 28–32)` block, before `## Tech Stack` section
  - `## Frozen Invariants → Core engine` amendment: extend the "frozen since Plan 25-01" sentence to list xrom.rs + modal.rs as carve-outs gated by ADR-v3.1-004
- **`PROJECT.md` insertion points (Plan 35-04):** "Shipped" lines update (add Phase 33 / 34 / 35 ship dates); "Current focus" update (Phase 36 — GUI integration); optional `### v3.1 additions` block (planner discretion per D-30.8 Claude's Discretion analog)
- **`README.md` insertion point (Plan 35-04):** under `## Features`, v3.1 soft-claim line immediately below the v3.0 soft-claim; link to `docs/hp41-stat1-function-matrix.md` under "See" or equivalent navigation
- **`docs/architecture-history.md` insertion point (Plan 35-04):** new `## v3.1 additions (Stat 1 Pac Emulation, Phases 33–35 — 36–37 IN PROGRESS)` section after the v3.0 section; sub-sections per phase
- **`.planning/MILESTONES.md` v3.1 stub (Plan 35-04):** one-liner placeholder; full milestone entry lands at Phase 37 ship via `/gsd-complete-milestone`
- **CI gate continuity:** existing CI step that calls `just docs-matrix-check` continues to gate against drift; after Plan 35-01 lands, the same step runs all three invocations and diffs all three matrix files. No CI workflow file changes required.

</code_context>

<specifics>
## Specific Ideas

- **Stat 1 soft-claim wording (locked in D-35.3):**
  ```
  - Stat 1 Pac behavioral emulation (13 programs, 26 XEQ entry points,
    RAND/SEED extension, documented divergences)
  ```
  Plus link `docs/hp41-stat1-function-matrix.md`.

- **`### v3.1 additions` block heading (locked in D-35.3 / D-30.8 carry-forward):**
  ```
  ### v3.1 additions (Stat 1 Pac Emulation, Phases 33–35 — 36–37 IN PROGRESS)
  ```
  Phases 36 + 37 sub-sections carry `(in progress)` suffixes until their own ships fill them in.

- **`33-SPEC-AMENDMENT.md` 6-row table shape (D-35.1):**
  | # | SPEC.md row | Original oracle | Scipy-correct value | Test file:line | Drift root cause | D-35-NN |
  |---|-------------|-----------------|---------------------|----------------|------------------|---------|
  | 1 | Req. NN | 1e-9 tolerance | 1e-5 band | `stat1/normd.rs:NNN` | rust_decimal A&S 6 precision limit | D-35-NN |
  | … | …         | …               | …                   | …              | …                | …       |

- **ADR-v3.1-002 Free42 disclaim wording (Pitfall 19):** the `## Decision` or `## Consequences` section of ADR-v3.1-002 explicitly states something like "Algorithm independently re-derived from Wichura (AS 241) / Cody (AS 239) / Lentz (AS 63) primary sources; Free42 source consulted only as sanity-check oracle, not copied." Matches the per-file header policy already enforced in `hp41-core/src/ops/stat1/*.rs` via the Phase 33 Plan 33-00 contamination-guard extension.

- **Stat 1 divergence-catalog `D-35-NN` entry-order convention (D-35.4):**
  - Bucket 1 (OM Divergences): D-35-01..NN — start with the 6 oracle drifts (in SPEC.md original-line-number order); then any OM-numerical-mismatch entries discovered during Plan 35-02 authoring
  - Bucket 2 (Emulator Extensions): D-35-(NN+1)..MM — RAND/SEED entry first (links to ADR-v3.1-001), then any other extensions (e.g., ΣPOLYP `DEGREE=?` UX-convention transcription, RAND/SEED LCG formula provenance detail)
  - Bucket 3 (Behavioral Policies): D-35-(MM+1)..ZZ — ΣTSTAT pooled-only (cites SPEC Req. 25), XROM-7 vs XROM-2 prefix convention, math1/ freeze second carve-out cross-ref to ADR-v3.1-004, any IN-01..IN-05 follow-ups that fit

- **Footnote/citation style:** ADRs use markdown footnote syntax (`[^1]`) for OM page citations and inline links for community URLs — matches v3.0-003/004 precedent.

- **`docs/hp41-stat1-function-matrix.md` filename:** matches the path-shape `docs/hp41-<pac-slug>-function-matrix.md`; planner does NOT pick `hp41-stat1b-function-matrix.md` (mirroring `STAT_1.name = "STAT 1B"` CATALOG 2 display) because the JSON file uses `hp41-stat1-` and the matrix-renderer dispatch keys off the basename.

- **`### v3.1 additions` block sub-section headings (Plan 35-04 input):**
  ```
  #### Phase 33 — XROM Activation + Distribution Primitives + All Stat 1 Ops (shipped 2026-05-22)
  #### Phase 34 — CLI Integration (shipped 2026-05-23)
  #### Phase 35 — Documentation & ADRs (shipped YYYY-MM-DD)
  #### Phase 36 — GUI Integration (in progress)
  #### Phase 37 — Test Hardening & Quality Gates (in progress)
  ```

</specifics>

<deferred>
## Deferred Ideas

- **GUI mirroring of all v3.1 documentation (CATALOG 2 STAT 1B display, GUI `?` overlay "Stat 1 Pac (XROM 2)" section)** — Phase 36 work. Phase 35 documents the CLI surface fully; the GUI surface lands when Phase 36 mirrors it via the same shared core code (Stat 1 enum extension + ModalProgram::Stat1 dispatch).
- **Hard-claim "feature-complete Stat 1 Pac behavioral emulation per OM 00041-90030" in README and CLAUDE.md** — Phase 37 conditional on STAT-QUAL-04 + STAT-QUAL-11. Phase 35 ships only the soft-claim (D-35.3).
- **`numerical_accuracy.rs` extension with Stat 1 oracle cases** — Phase 37 / STAT-QUAL-04. Phase 35 documents the 6 oracle drifts that already exist; Phase 37 expands the oracle test surface.
- **E2E smoke extension with one Stat 1 workflow** — Phase 37 / STAT-QUAL-11.
- **`lint_stat1_assertions.rs` / `stat1_op_test_count.rs` / `xrom_shadowing.rs` STAT_1 extension** — Phase 37 / STAT-QUAL-06..08.
- **`/gsd-complete-milestone` v3.1 ship** — Phase 37 post-ship. Phase 35 lands a `.planning/MILESTONES.md` stub; full milestone-summary one-liner + ROADMAP archive moves at Phase 37 ship.
- **Future v3.2+ pac documentation pattern** (Time Pac, Advanced Matrix, Advantage) — when the THIRD XROM module lands, the matrix-renderer XROM column convention + the divergence-catalog three-bucket pattern + the soft-claim-then-graduate README cadence + the phase-origin `D-XX-NN` numbering all serve as the template. Captured implicitly via the Phase 30 → Phase 35 cross-pac consistency.
- **`docs/hp41-stat1-function-matrix.md` schema documentation** — the matrix file's column convention (XROM column on Stat 1 + Math 1 + future pacs, no XROM column on hp41cv) isn't documented anywhere yet beyond the v3.0 Phase 30 CONTEXT and this CONTEXT. Future doc-debt candidate if/when readers stumble on it.
- **Cross-pac divergence cross-linking (math1 ↔ stat1)** — some policies span both pacs (e.g., the strict-reject nested-user-callback policy from D-30-06 carries to stat1 as "self-contained iteration only" for the quantile loops). Phase 35 cites the math1 entry; doesn't replicate. If a third pac lands and the pattern is shared, capture as a cross-pac index page (`docs/divergences-index.md`) at that point.
- **Signed binary releases (cargo-dist CLI + tauri-action GUI)** — deferred to v3.1.x / v3.2 per PROJECT.md lock 2026-05-21.
- **Module-prefix divergence-ID rename retro** — D-35.4 deferred for now. If a third pac (v3.2 Time) lands and the phase-origin convention gets unwieldy, retro-rename math1 + stat1 + Time docs to module-prefix in one sweep.

</deferred>

---

*Phase: 35-documentation-adrs*
*Context gathered: 2026-05-23*
