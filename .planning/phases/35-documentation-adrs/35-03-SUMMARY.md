---
phase: 35
plan: 03
subsystem: docs-adr
tags:
  - docs
  - adr
  - architecture-archaeology
  - free42-disclaim
  - stat1-pac
  - v3.1
requires:
  - .planning/phases/35-documentation-adrs/35-03-PLAN.md
  - .planning/phases/35-documentation-adrs/35-CONTEXT.md
  - .planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-CONTEXT.md
  - docs/adr/v3.0-003-inv-epsilon.md (long-form template anchor)
  - docs/adr/v3.0-004-intg-threshold.md (long-form template anchor)
  - docs/adr/v3.0-002-user-callback-policy.md (Free42 disclaim shape anchor)
provides:
  - docs/adr/v3.1-001-rng-state-placement.md (ADR for rand_seed CalcState field; STAT-RNG-03 lock)
  - docs/adr/v3.1-002-distribution-primitives-policy.md (ADR for statrs rejection + AS 239 / AS 63 / Acklam hand-coded; Free42 disclaim verbatim)
  - docs/adr/v3.1-003-anova-register-layout.md (ADR for OM 00041-90030 transcription + named consts; P21 mitigation)
  - docs/adr/v3.1-004-math1-freeze-second-carve-out.md (ADR for xrom.rs + modal.rs carve-out; CLAUDE.md amendment gate)
  - docs/adr/v3.1-005-modalprogram-stat1-enum-extension.md (ADR for ModalProgram::Stat1(Stat1Step); single-enum lock)
affects:
  - Plan 35-04 narrative-docs (CLAUDE.md Frozen Invariants Core engine amendment is gated by ADR-v3.1-004 Status: Locked 2026-05-22 line)
  - Plan 35-02 divergence-catalog (D-35-12 / sibling cross-references ADR-v3.1-001 for IN-05 first-call-from-zero behaviour)
tech-stack:
  added: []
  patterns:
    - long-form ADR template per D-30.6 (Status / Owner / Requirement refs / Downstream consumer / ADR write-up prose / Context / Decision / Consequences / Alternatives Considered / Footnotes)
    - verbatim 33-CONTEXT.md blockquote pattern per D-30.7 (Alternatives Considered Option B / C / D)
    - Free42 disclaim sentence per Pitfall 19 (grep-detectable; appears in Decision AND Footnotes of ADR-v3.1-002)
    - lock date = decision-lock date (2026-05-22 Phase 33 plan-phase), NOT Phase 35 ship date — CONTEXT.md Claude's Discretion line 143
key-files:
  created:
    - docs/adr/v3.1-001-rng-state-placement.md (264 lines / 13.9 KB)
    - docs/adr/v3.1-002-distribution-primitives-policy.md (279 lines / 16.0 KB)
    - docs/adr/v3.1-003-anova-register-layout.md (269 lines / 14.7 KB)
    - docs/adr/v3.1-004-math1-freeze-second-carve-out.md (277 lines / 15.9 KB)
    - docs/adr/v3.1-005-modalprogram-stat1-enum-extension.md (288 lines / 14.6 KB)
  modified: []
decisions:
  - All 5 ADRs carry Status: Locked 2026-05-22 per CONTEXT.md Claude's Discretion line 143 (Phase 33 plan-phase / discuss-phase session date, NOT the Phase 35 ship date).
  - Owners map per CONTEXT.md Claude's Discretion line 144 — verified inline (-001 Plan 33-01 + Plan 33-08; -002 Plan 33-02; -003 Plan 33-00 + Plan 33-06; -004 + -005 Phase 33 plan-phase D-33.3b).
  - ADR-v3.1-002 carries the verbatim Free42 disclaim sentence "Algorithm independently re-derived from primary sources (Wichura AS 241 / Cody AS 239 / Lentz AS 63); Free42 source consulted only as sanity-check oracle, not copied." in both Decision and Footnotes (grep -c returns 2).
  - ADR-v3.1-004's Status: Locked 2026-05-22 line is the canonical gate for Plan 35-04's CLAUDE.md "Frozen Invariants → Core engine" amendment.
metrics:
  duration: ~22 minutes (worktree-isolated execution; 3 atomic commits)
  completed: 2026-05-23
  tasks: 3
  files: 5
---

# Phase 35 Plan 35-03: Architecture-Decision Records for v3.1 Architectural Locks

Five long-form ADRs documenting the v3.1 architectural locks made during Phase 33 plan-phase / discuss-phase (2026-05-22), authored per the v3.0-003 / v3.0-004 / v3.0-005 template (D-30.6) so future readers (and future v3.2+ pac authors) find the provenance for every lock without re-reading 33-CONTEXT.md.

## Files Created

| ADR | Lines | Bytes | Status | Owner |
| --- | ----- | ----- | ------ | ----- |
| `docs/adr/v3.1-001-rng-state-placement.md` | 264 | 13904 | Locked 2026-05-22 | Plan 33-01 + Plan 33-08 |
| `docs/adr/v3.1-002-distribution-primitives-policy.md` | 279 | 15983 | Locked 2026-05-22 | Plan 33-02 |
| `docs/adr/v3.1-003-anova-register-layout.md` | 269 | 14673 | Locked 2026-05-22 | Plan 33-00 + Plan 33-06 |
| `docs/adr/v3.1-004-math1-freeze-second-carve-out.md` | 277 | 15909 | Locked 2026-05-22 | Phase 33 plan-phase (D-33.3b) |
| `docs/adr/v3.1-005-modalprogram-stat1-enum-extension.md` | 288 | 14645 | Locked 2026-05-22 | Phase 33 plan-phase (D-33.3b) |

All five files comfortably exceed the per-ADR minimums in 35-03-PLAN.md (≥ 140 lines / ≥ 5.5 KB for -001/-003/-004/-005; ≥ 150 lines / ≥ 6.5 KB for -002). Each ADR has at least 5 H2 sections (Context / Decision / Consequences / Alternatives Considered / Footnotes-References); ADR-v3.1-001 has 6 (adds "Ready-to-paste Rust struct field").

## Free42 Disclaim Verification (Pitfall 19)

`grep -c "consulted only as sanity-check oracle, not copied" docs/adr/v3.1-002-distribution-primitives-policy.md` returns **2** — one in the Decision section (immediately following the locked policy statement), one in the Footnotes [^10] block (the dedicated Free42 reference with the project's standard CONSULT-ONLY oracle policy text). The sentence is structured on a single line per source so the verbatim grep matches regardless of line-wrap.

Two additional lighter mentions of Free42 appear in ADR-v3.1-002 — `27` total `free42` matches across the file (Footnotes URLs, contamination-guard script reference, etc.) — but only the two grep-detectable disclaim sentences are required by Pitfall 19.

## Citation Provenance Verification (Pitfall 18)

Every ADR carries at least one HP 00041-90030 OM reference OR a non-OM citation pair (NPS document / scipy / 33-CONTEXT.md decision ID + Pitfall reference):

| ADR | OM 00041-90030 refs | Non-OM citations |
| --- | ------------------- | --------------- |
| v3.1-001 | 5 | NPS55-84-003 p. 21–22; HP-65 User's Library (Don Malm); HP-41C Standard Applications p. 24; 33-CONTEXT.md D-33.4 / D-33.4a / D-33.7; 33-REVIEW.md IN-05; PITFALLS.md P20; ADR-v3.0-001 |
| v3.1-002 | 2 | Wichura AS 241; Cody AS 239; Lentz AS 63; Acklam 1996/1999 (stackedboxes.org public-domain mirror); NR 3e §6.2 + §6.4; SUMMARY.md statrs rejection paragraph; STACK.md MathematicalOps coverage; PITFALLS.md P19 + P27; Free42 URL pair (consult-only oracle); scripts/check-free42-contamination.sh 18-token policy; scipy.stats oracle citation discipline; 33-CONTEXT.md D-33.5 / D-33.6 / D-33.8 |
| v3.1-003 | 4 | NPS55-84-003 (secondary register-layout cross-check); PITFALLS.md P21 + P18; hp41-core/src/ops/stat1/mod.rs; 33-REVIEW.md WR-01 (commit b29aa03); 33-CONTEXT.md D-33.1 + D-33.5 |
| v3.1-004 | 1 | 33-CONTEXT.md D-33.3 + D-33.3a + D-33.3b; CLAUDE.md Frozen Invariants Core engine; hp41-core/src/ops/math1/xrom.rs + modal.rs file:line refs; hp41-core/src/ops/stat1/modal.rs; calc.fjk.ch/db/hp41mod.php (Statistics Pac 1B hardware-XROM ID 2); ADR-v3.0-001 (Op-strategy A); ADR-v3.1-005 (sister ADR); PITFALLS.md P22 |
| v3.1-005 | 1 | 33-CONTEXT.md D-33.3b + D-33.5; ADR-v3.1-004 (sister); ADR-v3.0-001 (single-Op-enum precedent preserved by analogy); hp41-core/src/ops/math1/modal.rs file:line refs; hp41-core/src/ops/stat1/modal.rs; CLAUDE.md 4-way exhaustive-match invariant; PITFALLS.md P22 |

ADR-v3.1-002, -003, -004, -005 all carry an OM 00041-90030 mention even where the lock itself is OS-architecture-only (-004, -005 cite the OM as the source for the Stat 1 Pac module-id provenance via calc.fjk.ch and the per-program modal user flows).

## Verbatim 33-CONTEXT.md Quoting (D-30.7)

| ADR | 33-CONTEXT.md decision quoted verbatim | Location in ADR |
| --- | -------------------------------------- | --------------- |
| v3.1-001 | D-33.4 (RAND/SEED policy + LCG community-source rationale) | Alternatives Considered → Option B blockquote |
| v3.1-002 | (No 33-CONTEXT.md verbatim — uses .planning/research/SUMMARY.md statrs-rejection paragraph instead, per D-30.7 sourcing latitude when the originating source is a research artifact rather than a CONTEXT.md decision; the paragraph appears in Alternatives Considered → Option A blockquote) | Alternatives Considered → Option A blockquote |
| v3.1-003 | (No 33-CONTEXT.md verbatim — Pitfall 21 paraphrased + 33-CONTEXT.md D-33.1 / D-33.5 cited in Footnotes; the rejected alternatives are paraphrased per Pitfall 18 sourcing discipline because no discrete D-33.x decision rejected the inline-literal-indices option — it was rejected at the spec-phase Plan 33-00 gate by structural design) | Alternatives Considered narrative |
| v3.1-004 | D-33.3b (full multi-line verbatim — modal.rs second carve-out + parallel-enum rejection rationale) | Alternatives Considered → Option B blockquote |
| v3.1-005 | D-33.3b (same verbatim text as ADR-v3.1-004; reused per CONTEXT.md D-30.7 — the same decision viewed from the enum-design angle) | Alternatives Considered → Option B blockquote |

## URLs and Community Citations Included

- `<https://stackedboxes.org/2017/05/01/acklams-normal-quantile-function/>` — ADR-v3.1-002 Footnote [^5] (Acklam public-domain mirror).
- `<https://thomasokken.com/free42/>` + `<https://github.com/thomasokken/free42>` — ADR-v3.1-002 Footnote [^10] (Free42 consult-only oracle pair, paralleling ADR-v3.0-002 line 238 pattern).
- `calc.fjk.ch/db/hp41mod.php` — ADR-v3.1-004 Footnote [^6] (Statistics Pac 1B hardware-XROM ID 2 provenance; cited as inline plain-text URL).

No MoHPC / Mike Sebastian forensic URLs were added — the listed Footnote URLs above plus the citation-tuple of (OM page + NPS55-84-003 + 33-CONTEXT.md D-33.x + relevant Pitfall) suffice for each ADR per the Pitfall 18 sourcing-discipline minimum.

## ADR-v3.1-004 Status: Locked Line Gate

The CLAUDE.md amendment task in Plan 35-04 depends on ADR-v3.1-004's Status: Locked line being grep-detectable in the canonical location:

```
$ grep -E "^\*\*Status:\*\* Locked 2026-05-22" docs/adr/v3.1-004-math1-freeze-second-carve-out.md
**Status:** Locked 2026-05-22
```

This grep returns the expected single match on line 3 of the ADR. Plan 35-04 may rely on this as the policy gate for the CLAUDE.md "Frozen Invariants → Core engine" amendment that lists `xrom.rs` + `modal.rs` as the two carved-out files.

## Deviations from Plan

None — plan executed exactly as written:

- 3 tasks executed in sequence; each task committed individually.
- Each ADR follows the long-form template from `v3.0-003-inv-epsilon.md` / `v3.0-004-intg-threshold.md` (with `v3.0-002-user-callback-policy.md` informing the Free42 disclaim shape for ADR-v3.1-002).
- All line counts, byte counts, H2 section counts, lock dates, owners, verbatim quotes, and citation provenance pass the per-ADR acceptance criteria in 35-03-PLAN.md.
- Single minor mid-task fix-up (Rule 3): the Free42 disclaim sentence in ADR-v3.1-002 was initially written across two source lines, which caused `grep -E "consulted only as sanity-check oracle, not copied"` to return zero matches. Reflowed both occurrences (Decision section and Footnote [^10]) onto single source lines so the verbatim grep matches as required by the plan's automated verification. The fix did not change visible markdown rendering.

## Self-Check: PASSED

| Check | Result |
| ----- | ------ |
| `docs/adr/v3.1-001-rng-state-placement.md` exists | FOUND |
| `docs/adr/v3.1-002-distribution-primitives-policy.md` exists | FOUND |
| `docs/adr/v3.1-003-anova-register-layout.md` exists | FOUND |
| `docs/adr/v3.1-004-math1-freeze-second-carve-out.md` exists | FOUND |
| `docs/adr/v3.1-005-modalprogram-stat1-enum-extension.md` exists | FOUND |
| Commit `8c1d2b1` (ADR-001 + ADR-002) | FOUND in git log |
| Commit `0b6a514` (ADR-003 + ADR-004) | FOUND in git log |
| Commit `1cbb281` (ADR-005) | FOUND in git log |

All artifacts present; no missing items.

## Commits

| Hash | Tasks | Files |
| ---- | ----- | ----- |
| `8c1d2b1` | Task 1 | `docs/adr/v3.1-001-rng-state-placement.md`, `docs/adr/v3.1-002-distribution-primitives-policy.md` |
| `0b6a514` | Task 2 | `docs/adr/v3.1-003-anova-register-layout.md`, `docs/adr/v3.1-004-math1-freeze-second-carve-out.md` |
| `1cbb281` | Task 3 | `docs/adr/v3.1-005-modalprogram-stat1-enum-extension.md` |

## Downstream Consumer Notes

- **Plan 35-04 (Wave 2):** the CLAUDE.md "Frozen Invariants → Core engine" amendment task is now unblocked — ADR-v3.1-004's Status: Locked 2026-05-22 line is the canonical gate. Suggested amendment wording is reproduced in ADR-v3.1-004's Decision section (the substance is fixed; Plan 35-04 picks exact phrasing).
- **Plan 35-02 (Wave 1 sibling, in flight):** ADR-v3.1-001's Negative Consequences subsection forward-references D-35-12 (or sibling) in `docs/hp41-stat1-divergences.md` for the IN-05 first-call-from-zero behavioural note. Plan 35-02 is the natural home; if Plan 35-02 elects to route IN-05 to ADR-v3.1-001's Footnotes instead, no cross-reference update in ADR-v3.1-001 is needed (the Footnote [^6] already cites IN-05 directly).
- **Phase 36 GUI integration:** ADR-v3.1-005 documents the `ModalProgram::Stat1(Stat1Step)` extension that the GUI `pending_prompt` exhaustive match (CLAUDE.md GUI key files) must enumerate; the 4-way invariant adds one new arm per match site. STAT-GUI-04 explicitly calls out the reuse of existing modal infrastructure.

## Known Stubs

None — no placeholder text, no `TODO` / `FIXME` markers, no empty-data UI surfaces. All ADRs are complete narrative documents ready for downstream consumers.

---

*Plan 35-03 complete. 5 ADRs / 3 commits / ~75 KB markdown across all artefacts.*
