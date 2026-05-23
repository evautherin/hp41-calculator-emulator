# Phase 35: Documentation & ADRs - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-23
**Phase:** 35-documentation-adrs
**Areas discussed:** SPEC oracle drift reconciliation, ADR scope, README v3.1 claim strength, Divergence catalog numbering scheme

---

## Area Selection (multiSelect)

| Option | Description | Selected |
|--------|-------------|----------|
| SPEC oracle drift reconciliation | Where do the 5–6 SPEC.md oracle drifts live? Supplement / revision / divergences-only / test-comment-only? | ✓ |
| ADR scope: minimum 3 vs +math1-freeze-exception/ModalProgram::Stat1 | STAT-DOC-04 mandates ≥3; should the math1/ freeze second carve-out and the ModalProgram::Stat1 extension each get their own ADR? | ✓ |
| README v3.1 claim strength + graduation gating | Soft-claim now (mirror v3.0) vs hard-claim now vs skip until Phase 37 vs hybrid | ✓ |
| Divergence catalog numbering scheme (D-30-NN vs D-35-NN vs module-prefix) | Phase-origin parallelism vs pac-prefix that scales vs hybrid retro-rename | ✓ |

**User's choice:** All four gray areas selected for discussion.
**Notes:** Phase 35 is the v3.1 documentation lock — every selected area materially shapes how the milestone narrative gets sealed.

---

## SPEC Oracle Drift Reconciliation

Phase 33 verification (`33-VERIFICATION.md`) explicitly flagged 5–6 oracle drifts where the planning-phase SPEC.md numbers differed from scipy ground truth: ΣNORMD CDF tolerance band 1e-9 → 1e-5, ΣAOVONE F-ratio 100.0 → 50.0, ΣSPEAR ρ_s 0.7 → 0.8, ΣEFXSQ χ² calibration, ΣBSTAT CV 0.4083 → 0.5270, ΣTSTAT deep-tail Student-t p (AS 63 precision limit). Tests assert scipy-correct values and pass; SPEC.md still carries the original planning-phase numbers.

| Option | Description | Selected |
|--------|-------------|----------|
| SPEC.md SUPPLEMENT + stat1-divergences.md entries | History-preserving: write `33-SPEC-AMENDMENT.md` (original SPEC stays frozen) + 1 divergence entry per drift in `stat1-divergences.md` | ✓ |
| SPEC.md in-place REVISION (revisionism) | Edit `33-SPEC.md` directly with scipy-correct values + changelog block at top | |
| stat1-divergences.md entries ONLY | Document drifts only in divergences catalog; SPEC.md stays unchanged | |
| Test-comment-only (lowest visibility) | Inline test-file comments only; no docs surface | |

**User's choice:** SPEC.md SUPPLEMENT + stat1-divergences.md entries → **D-35.1**.
**Notes:** Both surfaces ship. The original `33-SPEC.md` stays frozen (planning archaeology preserved); the amendment is a sibling file documenting the 6 drifts with scipy citations + test-file:line cross-references. Each drift ALSO gets a numbered divergence entry in `stat1-divergences.md` (bucket-3 Behavioral Policies — mathematical-ground-truth corrections aren't OM divergences but are policy locks worth catalog-citing). Resolves the central tension between "SPEC.md should be authoritative" and "planning archaeology has educational value". Tests are ground truth either way; docs follow. Revisionism rejected because it rewrites history; divergences-only rejected because SPEC.md would then be structurally false-on-its-face; test-comment-only rejected because it defeats the Phase 35 docs purpose.

---

## ADR Scope and Count

STAT-DOC-04 mandates a minimum 3 ADRs (RNG-state placement; ANOVA register layout; hand-coded distribution primitives policy). Phase 33's plan-phase user-confirmed a SECOND math1/ freeze carve-out for `modal.rs` (D-33.3b) alongside `xrom.rs` (D-33.3a) — and chose `ModalProgram::Stat1` enum extension over a parallel `Stat1ModalProgram`. Both decisions are architectural locks that survive v3.1.

| Option | Description | Selected |
|--------|-------------|----------|
| 5 ADRs (3 mandated + freeze-exception + ModalProgram::Stat1) | v3.1-001 RNG; -002 dist primitives; -003 ANOVA register layout; -004 math1/ freeze second carve-out; -005 ModalProgram::Stat1 enum extension | ✓ |
| 3 ADRs (minimum mandated) | Only the three STAT-DOC-04 mandates; document freeze + ModalProgram inline in CLAUDE.md / divergences / CONTEXT.md | |
| 4 ADRs (3 + freeze-exception, skip ModalProgram) | Promote the freeze carve-out (changes a CLAUDE.md Frozen Invariant) but skip the ModalProgram architectural lock | |
| 6+ ADRs (max coverage) | Add ADR for XROM-prefix mnemonic-convention lock on top of the 5 above | |

**User's choice:** 5 ADRs → **D-35.2**.
**Notes:** Mirrors v3.0's 5-ADR scope (003+004 in Phase 28, 001+002+005 in Phase 30). Both D-33.3b (modal.rs freeze amendment) and ModalProgram::Stat1 were real architectural locks made during Phase 33 plan-phase; not ADR-recording them would lose provenance for future v3.2+ pacs (Time, Advantage) that will face the same "extend the enum vs spawn a parallel enum" choice. 4-option rejected because the ModalProgram extension is the load-bearing infrastructure choice that makes v3.2+ pacs cheap. 6+ rejected because the XROM-prefix convention is a behavioral policy, not an architectural lock — belongs in stat1-divergences.md bucket-3.

---

## README v3.1 Claim Strength + Graduation Gating

v3.0 shipped soft-claim in Phase 30 (D-30.9) and graduated to hard-claim in Phase 32 (D-32.5) after coverage hit 95.39%. Phase 33 already preserves coverage ≥ 95.39%, but Phase 37 still gates STAT-QUAL-04 (numerical_accuracy.rs Stat 1 cases) + STAT-QUAL-11 (E2E smoke Stat 1 workflow).

| Option | Description | Selected |
|--------|-------------|----------|
| Mirror v3.0: soft-claim now, graduate at Phase 37 | "Stat 1 Pac behavioral emulation (13 programs, 26 XEQ entry points, RAND/SEED extension, documented divergences)" + matrix link | ✓ |
| Hard-claim now | "Feature-complete Stat 1 Pac behavioral emulation per OM 00041-90030" now in Phase 35 | |
| Skip soft-claim; hold until Phase 37 | No README change in Phase 35; full claim lands at Phase 37 ship | |
| Soft-claim + explicit dependencies-still-pending block | Soft-claim PLUS visible paragraph listing what Phase 37 still gates on | |

**User's choice:** Mirror v3.0 → **D-35.3**.
**Notes:** Identical discipline to v3.0 D-30.9 → D-32.5; reader learning curve stays consistent across milestones; conservative claim is accurate at Phase 35 + clear upgrade path at Phase 37. Hard-claim-now rejected because Stat 1 numerical-accuracy cases + E2E smoke haven't landed yet → overclaim risk. Skip-soft-claim rejected because user-facing v3.1 invisibility between Phase 35 and Phase 37 would lose discoverability. Dependencies-pending-block rejected because v3.0 didn't do it and diverges from the established cadence.

---

## Divergence Catalog Numbering Scheme

`docs/hp41-math1-divergences.md` uses `D-30-NN` (phase-origin scheme — Phase 30 locked them). Stat 1 needs its own scheme.

| Option | Description | Selected |
|--------|-------------|----------|
| Continue D-35-NN (phase-origin) | Parallel to D-30-NN; phase number always recoverable from ID | ✓ |
| Module-prefix D-S-NN (or D-STAT1-NN) | ID self-describes the pac; survives phase renumbering | |
| Hybrid: D-35-NN now, retro-rename to module-prefix later | Ship D-35-NN now; if v3.2 Time Pac arrives and pattern outgrows, retro-rename in one sweep | |

**User's choice:** D-35-NN (phase-origin) → **D-35.4**.
**Notes:** Parallelism with math1-divergences.md = zero cognitive load for readers; cross-doc cites stay short (`see D-35-07`). Module-prefix rejected because it breaks parallelism — would force a retro-rename of math1's D-30-NN entries for consistency, or accept divergent schemes between docs. Hybrid rejected because it defers a rename that may never happen + if it does happen it's a doc-wide find-replace cascading into external citations. v3.0 Phase 30 set the phase-origin precedent; it has held; Phase 35 extends it. Future v3.2 Time Pac will use `D-XX-NN` where XX is whatever phase locks that pac's documentation (same pattern).

---

## Claude's Discretion

The following items were captured in CONTEXT.md as planner-discretion (not load-bearing for downstream agents but documented to prevent re-deliberation):

- `33-SPEC-AMENDMENT.md` exact frontmatter shape + table column order (table rows + cross-refs are fixed; presentation is planner's call)
- ADR-v3.1-001..005 `**Status:** Locked YYYY-MM-DD` dates (use the date the underlying decision was actually locked, NOT Phase 35 ship date; all map to 2026-05-22 Phase 33 plan-phase session)
- ADR `**Owner:**` lines (planner picks per-ADR Plan number ownership)
- Plan slicing (recommended 4 plans: tooling+matrix+amendment / divergences / ADRs / narrative-docs — planner has discretion to fold or split)
- `docs/hp41-stat1-divergences.md` preamble content (mirror math1-divergences.md with v3.1-specific clarifying note)
- `### v3.1 additions` block insertion point in CLAUDE.md (immediately after v3.0 block; planner picks exact line)
- `### v3.1 additions` block shape in PROJECT.md (recommend: PROJECT.md gets concise milestone lines; CLAUDE.md carries the full block — per D-30.8 Claude's Discretion analog)
- `docs/architecture-history.md` v3.1 narrative sub-section depth (mirror v3.0 depth: 2–4 paragraph per phase)
- CLAUDE.md `## Frozen Invariants → Core engine` amendment exact wording (substance fixed; wording is planner's)
- Routing of Phase 33's deferred IN-01..IN-05 Info findings (stat1-divergences.md bucket-3 vs ADR Footnotes per finding-content)
- README "See" / navigation section structure (match existing v2.2 / v3.0 soft-claim link layout)
- CI gate continuity (no CI workflow file changes required; existing `just docs-matrix-check` step picks up the third invocation automatically)

---

## Deferred Ideas

Surfaced during Phase 35 discussion but explicitly outside Phase 35 scope:

- GUI mirroring of all v3.1 documentation (CATALOG 2 "STAT 1B", GUI `?` overlay "Stat 1 Pac (XROM 2)" section) — Phase 36
- Hard-claim "feature-complete Stat 1 Pac behavioral emulation per OM 00041-90030" — Phase 37 conditional on STAT-QUAL-04 + STAT-QUAL-11
- `numerical_accuracy.rs` Stat 1 oracle cases — Phase 37 / STAT-QUAL-04
- E2E smoke extension with Stat 1 workflow — Phase 37 / STAT-QUAL-11
- `lint_stat1_assertions.rs` / `stat1_op_test_count.rs` / `xrom_shadowing.rs` STAT_1 extension — Phase 37 / STAT-QUAL-06..08
- `/gsd-complete-milestone` v3.1 ship — Phase 37 post-ship
- Future v3.2+ pac documentation pattern (Time Pac, Advanced Matrix, Advantage) — captured implicitly via Phase 30 → Phase 35 cross-pac consistency
- `docs/hp41-stat1-function-matrix.md` schema documentation as standalone doc — future doc-debt candidate
- Cross-pac divergence cross-linking (math1 ↔ stat1) via `docs/divergences-index.md` — capture if a third pac lands and the pattern is shared
- Signed binary releases — deferred to v3.1.x / v3.2 per PROJECT.md lock 2026-05-21
- Module-prefix divergence-ID rename retro — D-35.4 deferred for now; revisit only if a third pac surfaces the rename pressure
