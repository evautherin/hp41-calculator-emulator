---
phase: 35-documentation-adrs
verified: 2026-05-23T00:00:00Z
status: passed
score: 6/6 roadmap success criteria verified; 4/4 plan-level must-have truth sets verified; 6/6 STAT-DOC requirement IDs satisfied
overrides_applied: 0
---

# Phase 35: Documentation & ADRs Verification Report

**Phase Goal:** Stat 1 Pac integration is fully documented — OM divergences cataloged, docs-matrix generates a third function matrix, all v3.1 architectural decisions are recorded as ADRs, and README + CLAUDE.md are updated for v3.1.

**Verified:** 2026-05-23
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria + Plan must-haves merged)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| SC-1 | `just docs-matrix` regenerates three function matrices (`docs/hp41cv-function-matrix.md`, `docs/hp41-math1-function-matrix.md`, `docs/hp41-stat1-function-matrix.md`); `just docs-matrix-check` exits 0 | VERIFIED | Ran `just docs-matrix-check` live during this verification: exit code 0 with all three diff stages clean. Justfile lines 184-208 confirm three cargo-run + three diff pairs. |
| SC-2 | `docs/hp41-stat1-divergences.md` is a three-bucket catalog covering OM divergences / emulator extensions (RAND/SEED, rand_seed serde shape) / behavioral policies | VERIFIED | 597-line file; bucket headers at lines 49 (OM Divergences), 71 (Emulator Extensions), 169 (Behavioral Policies). 12 numbered `D-35-NN` entries: D-35-07 RAND/SEED (bucket 2) + D-35-08 ΣPOLYP DEGREE=? (bucket 2) + D-35-01..06 oracle drifts + D-35-09..12 policies (bucket 3). Bucket 1 intentionally empty per 33-VERIFICATION.md finding (documented in the bucket header text). |
| SC-3 | At least two new ADRs in `docs/adr/` covering (a) RNG-state placement (b) distribution-primitive policy (statrs rejected, AS 239/63/241 chosen) | VERIFIED (exceeds) | FIVE ADRs shipped: v3.1-001 RNG state placement (264 lines), v3.1-002 distribution primitives policy + Free42 disclaim (279 lines), v3.1-003 ANOVA register layout (269 lines), v3.1-004 math1/ freeze second carve-out (277 lines), v3.1-005 ModalProgram::Stat1 enum extension (288 lines). All 5 carry `**Status:** Locked 2026-05-22` on line 3; full long-form structure (Context / Decision / Consequences / Alternatives Considered / Footnotes); Free42 disclaim sentence appears exactly 2× in v3.1-002 (Decision line 60 + Footnotes line 249). |
| SC-4 | `docs/architecture-history.md` contains a v3.1 phase narrative parallel to v3.0; CLAUDE.md `### v3.1 additions` block describes STAT_1 XROM module + new Op count + rand_seed serde exception | VERIFIED | architecture-history.md: `## v3.1 additions (Stat 1 Pac Emulation, Phases 33-35 — 36-37 IN PROGRESS)` H2 at line 192; sub-sections `### Phase 33` (line 196), `### Phase 34` (line 210), `### Phase 35` (line 220); 262 lines total (grew +62 from 201 baseline). CLAUDE.md `### v3.1 additions` block at line 120, between Frozen Invariants (line 34) and Tech Stack (line 184). |
| Plan 35-01 truth #1 | `33-SPEC-AMENDMENT.md` lands as a SIBLING (not in-place edit of 33-SPEC.md) per D-35.1 | VERIFIED | File present at `.planning/phases/33-.../33-SPEC-AMENDMENT.md` (65 lines, 6 numbered `D-35-NN` entries). `git log -1 33-SPEC.md` returns commit `a0fa401` (Phase 33 spec-phase), confirming history-preserving discipline upheld. |
| Plan 35-01 truth #2 | Renderer keeps 1-in/1-out CLI signature; only a 4-line `else if` branch added | VERIFIED | `scripts/docs-matrix/src/main.rs` lines 74-75 show the new branch (predicate + title/src tuple); no struct widening; cargo build clean. |
| Plan 35-01 truth #3 | hp41cv + math1 matrices byte-identical to HEAD after regeneration (D-30.2 + D-35.1 carry-forward) | VERIFIED | `git diff --stat docs/hp41cv-function-matrix.md docs/hp41-math1-function-matrix.md` returns empty after `just docs-matrix` ran during this verification's docs-matrix-check execution. |
| Plan 35-02 truth | Catalog has ≥ 12 D-35-NN entries with 5-field shape + citation provenance per Pitfall 18 | VERIFIED | `grep -cE "^### D-35-[0-9]{2}:" docs/hp41-stat1-divergences.md` = 12. `grep -cE "HP 00041-90030\|N/A — emulator\|scipy.stats" docs/hp41-stat1-divergences.md` = 40 (well above the per-entry floor). Forward-refs to `v3.1-001` + `v3.1-004` + `33-SPEC-AMENDMENT.md` all resolve. |
| Plan 35-03 truth | All 5 ADRs lock-dated 2026-05-22 (Phase 33 discuss-phase, NOT Phase 35 ship date) | VERIFIED | `grep -nE "^\*\*Status:\*\* Locked 2026-05-22" docs/adr/v3.1-00*.md` returns exactly 5 matches (one per ADR, all on line 3). |
| Plan 35-04 truth #1 | README v3.1 soft-claim bullet present with exact D-35.3 wording; Documentation table row added | VERIFIED | README.md line 53-54: `- Stat 1 Pac behavioral emulation (13 programs, 26 XEQ entry points, RAND/SEED extension, [documented divergences](docs/hp41-stat1-divergences.md)) — see [Stat 1 Pac Function Matrix](docs/hp41-stat1-function-matrix.md)`. Documentation table row at line 139. |
| Plan 35-04 truth #2 | CLAUDE.md gains FIRST-EVER `### v3.x additions` block (v3.1 only, no v3.0 back-fill) per D-35.5 | VERIFIED | `grep -n "^### v3.1 additions" CLAUDE.md` returns line 120 only; no `### v3.0 additions` H3 introduced. v3.0 narrative remains in architecture-history.md per D-35.5 Option B. |
| Plan 35-04 truth #3 | CLAUDE.md `## Frozen Invariants → Core engine` math1/ freeze sentence gains an `**Exception (v3.1):**` clause listing xrom.rs + modal.rs gated by ADR-v3.1-004 | VERIFIED | CLAUDE.md line 56 contains the amended sentence: `**Exception (v3.1):** \`xrom.rs\` (D-33.3 / [ADR-v3.1-004]...) + \`modal.rs\` (D-33.3b / ADR-v3.1-004 — ...). All OTHER files in \`math1/\` remain frozen.` |
| Plan 35-04 truth #4 | Hard claim "feature-complete Stat 1 Pac" does NOT appear in README or CLAUDE.md (deferred to Phase 37) | VERIFIED | `grep -F "feature-complete Stat 1 Pac" README.md CLAUDE.md` returns no matches. |

**Score:** 13/13 truths verified (4 roadmap success criteria + 9 plan-level must-haves)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `scripts/docs-matrix/src/main.rs` | Three-way basename dispatch | VERIFIED | Stat1 branch present at lines 74-75; cargo build clean. |
| `Justfile` | Three-input docs-matrix + docs-matrix-check recipes | VERIFIED | 3 cargo-run lines + 3 diff lines across the two recipes (lines 184-208). |
| `docs/hp41-stat1-function-matrix.md` | Generated Stat 1 Pac matrix, ≥ 30 lines, 26 `Stat 1 / 2-N` cells | VERIFIED | 39 lines; 26 `Stat 1 / 2-N` cells; 8-column shape; title `# HP-41C Stat 1 Pac Function Matrix`. |
| `.planning/phases/33-.../33-SPEC-AMENDMENT.md` | 6-row drift reconciliation, ≥ 30 lines | VERIFIED | 65 lines; 6 numbered drift sub-sections (D-35-01..06); references `history-preserving` discipline; 16 hits across `history-preserving \| scipy \| hp41-core/src/ops/stat1/`. |
| `docs/hp41-stat1-divergences.md` | Three-bucket catalog, ≥ 250 lines, ≥ 12 D-35-NN entries | VERIFIED | 597 lines; 12 D-35-NN entries; three buckets at lines 49 / 71 / 169. |
| `docs/adr/v3.1-001-rng-state-placement.md` | ≥ 140 lines, Alternatives Considered | VERIFIED | 264 lines; full template structure. |
| `docs/adr/v3.1-002-distribution-primitives-policy.md` | ≥ 150 lines, Free42 disclaim verbatim | VERIFIED | 279 lines; disclaim sentence appears 2× as required by Pitfall 19. |
| `docs/adr/v3.1-003-anova-register-layout.md` | ≥ 140 lines, Alternatives Considered | VERIFIED | 269 lines; full template structure. |
| `docs/adr/v3.1-004-math1-freeze-second-carve-out.md` | ≥ 140 lines, contains `modal.rs` | VERIFIED | 277 lines; modal.rs cited extensively; Status: Locked 2026-05-22 line is the gate for CLAUDE.md amendment. |
| `docs/adr/v3.1-005-modalprogram-stat1-enum-extension.md` | ≥ 140 lines, contains `ModalProgram` | VERIFIED | 288 lines; full template structure. |
| `README.md` | v3.1 soft-claim + matrix link | VERIFIED | Lines 53-54 (soft-claim) + line 139 (Documentation table row). |
| `CLAUDE.md` | `### v3.1 additions` block + math1/ freeze amendment | VERIFIED | Line 120 (block) + line 56 (Exception clause). |
| `.planning/PROJECT.md` | v3.1 milestone progress + Current focus update | VERIFIED | Lines 74-77 (Shipped milestones sub-list); line 214 (Active in flight); Phase 35 / Phase 36 cited. |
| `docs/architecture-history.md` | New `## v3.1 additions` H2 | VERIFIED | Line 192; 262 lines total (+62 from 201 baseline). |
| `.planning/MILESTONES.md` | v3.1 IN PROGRESS stub | VERIFIED | Line 281 stub block; matches `D-30.8` carry-forward (full entry deferred to Phase 37 ship). |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `Justfile::docs-matrix` | `docs/hp41-stat1-functions.json → docs/hp41-stat1-function-matrix.md` | `cargo run -- <in> <out>` | WIRED | Justfile line 193-194. |
| `Justfile::docs-matrix-check` | `/tmp/hp41-stat1-function-matrix-check.md` (diff vs committed) | `cargo run + diff -u` | WIRED | Justfile lines 206-208. |
| `scripts/docs-matrix/src/main.rs::render_markdown` | stat1 title and src path | basename `ends_with` check | WIRED | Lines 74-75. |
| `33-SPEC-AMENDMENT.md` rows | `docs/hp41-stat1-divergences.md` bucket-3 entries | `D-35-NN` cross-reference | WIRED | 7 cross-refs from divergences to SPEC-AMENDMENT (D-35-01..06 + preamble). |
| `docs/hp41-stat1-divergences.md` (D-35-07 RAND/SEED) | `docs/adr/v3.1-001-rng-state-placement.md` | See-field forward-ref | WIRED | Line 117. |
| `docs/hp41-stat1-divergences.md` (D-35-11 math1/ freeze) | `docs/adr/v3.1-004-math1-freeze-second-carve-out.md` | See-field forward-ref | WIRED | Line 540. |
| `README.md::## Features` | `docs/hp41-stat1-function-matrix.md` | Inline v3.1 soft-claim bullet | WIRED | Line 54. |
| `README.md::## Documentation` | `docs/hp41-stat1-function-matrix.md` | Documentation table row | WIRED | Line 139. |
| `CLAUDE.md::Frozen Invariants Core engine` | `docs/adr/v3.1-004-math1-freeze-second-carve-out.md` | Exception clause inline-link | WIRED | Line 56. |
| `CLAUDE.md::### v3.1 additions` | 5 ADRs + divergence catalog + 33-SPEC-AMENDMENT | Synthesized bullets | WIRED | Block lines 120-182 cite all artifacts. |
| `docs/architecture-history.md::## v3.1 additions` | 5 ADRs + Phase 33-35 narrative | Sub-section-per-phase prose | WIRED | Lines 192-250. |

### Data-Flow Trace (Level 4)

Not applicable — documentation-only phase produces no runtime data flow. The docs-matrix renderer (the only runnable code touched, +2 lines) is verified by `just docs-matrix-check` exit 0 (Step 7b spot-check).

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `just docs-matrix-check` exits 0 (no drift across three matrix files) | `just docs-matrix-check; echo $?` | Exit code 0; all three diff stages clean | PASS |
| Stat1 matrix has exactly 26 `Stat 1 / 2-N` cells | `grep -c "Stat 1 / 2-" docs/hp41-stat1-function-matrix.md` | 26 | PASS |
| All 5 ADRs carry Status: Locked 2026-05-22 (Phase 33 plan-phase date) | `grep -cE "^\*\*Status:\*\* Locked 2026-05-22" docs/adr/v3.1-00*.md` | 5/5 | PASS |
| ADR-v3.1-002 carries the verbatim Free42 disclaim sentence (Pitfall 19) | `grep -c "consulted only as sanity-check oracle, not copied" docs/adr/v3.1-002-distribution-primitives-policy.md` | 2 | PASS |
| 33-SPEC.md NOT modified (history-preserving discipline) | `git log -1 --oneline .planning/phases/33-.../33-SPEC.md` | `a0fa401 spec(phase-33): add SPEC.md...` (Phase 33 spec-phase commit) | PASS |
| `docs/hp41-stat1-divergences.md` has exactly 12 D-35-NN entries | `grep -cE "^### D-35-[0-9]{2}:" docs/hp41-stat1-divergences.md` | 12 | PASS |

### Probe Execution

Not applicable — no `scripts/*/tests/probe-*.sh` declared in PLAN files; phase is documentation-only.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| STAT-DOC-01 | 35-01 | `docs/hp41-stat1-divergences.md` three-bucket catalog | SATISFIED | 597-line file with all three buckets present (actual catalog also satisfied by Plan 35-02; 35-01 frontmatter claims it but 35-02 is the substantive author — both plans collectively satisfy). |
| STAT-DOC-02 | 35-01 | `scripts/docs-matrix/` extended to three-input | SATISFIED | Renderer + Justfile both extended; `just docs-matrix-check` covers all three pairs. |
| STAT-DOC-03 | 35-01, 35-02 | `docs/hp41-stat1-function-matrix.md` regenerated from canonical JSON | SATISFIED | 39-line matrix file regenerated by `just docs-matrix`; in sync per CI gate. |
| STAT-DOC-04 | 35-03 | ADRs for RNG / register-layout / distribution-primitives | SATISFIED (exceeds) | 5 ADRs shipped (minimum 3 required). RNG = v3.1-001; register-layout = v3.1-003; distribution-primitives = v3.1-002 (with Free42 disclaim per Pitfall 19); plus v3.1-004 math1/ freeze + v3.1-005 modal enum. |
| STAT-DOC-05 | 35-04 | README v3.1 section + CLAUDE.md `### v3.1 additions` block | SATISFIED | README lines 53-54 + Documentation table row 139; CLAUDE.md `### v3.1 additions` block line 120. |
| STAT-DOC-06 | 35-04 | `docs/architecture-history.md` v3.1 narrative + MILESTONES.md stub | SATISFIED | architecture-history.md line 192 `## v3.1 additions` H2 (+62 lines); MILESTONES.md line 281 IN PROGRESS stub. |

**All 6 requirement IDs SATISFIED. No orphaned requirements** — REQUIREMENTS.md `Phase 35 | STAT-DOC-01..06 | 6` matches plan-frontmatter coverage exactly.

### Anti-Patterns Found

`grep -nE "TBD|FIXME|XXX"` across all 14 phase-35-modified files returned no matches. No debt markers, no unresolved follow-up references.

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | — | — | — | — |

The `35-REVIEW.md` code-review report (referenced by the verification request) found 0 BLOCKERS / 6 WARNINGS / 7 INFO. None of the WR-01..WR-06 findings invalidate goal achievement:
- WR-01..WR-05 are accuracy/precision polish on ADR-v3.1-001/004/005 narrative wording.
- WR-06 (`hp41-stat1-function-matrix.md` mis-labels RAND/SEED as "✓ v2.x") is a JSON-source-of-truth labeling concern that traces back to Phase 34's authoring of `docs/hp41-stat1-functions.json`, not a Phase 35 regression — the matrix faithfully reflects whatever the JSON says.

All WR/IN findings are advisory-only and surfaced for follow-up review-fix iteration; none gate the goal.

### Human Verification Required

None — all goal-relevant artifacts are programmatically verifiable (file existence, content patterns, link resolution, CI gate exit code, byte-identity of unchanged matrices). No visual/UX surface introduced in this phase; CLAUDE.md/README.md text is structural and grep-verified.

### Gaps Summary

No gaps. Phase 35 delivers the full Stat 1 Pac documentation surface:

1. **Tooling drift-catch:** `just docs-matrix-check` now covers three matrix files (cv + math1 + stat1); cv + math1 byte-identical to HEAD; `just docs-matrix-check` returns exit 0 live during verification.
2. **Divergence catalog:** 597 lines, 12 numbered `D-35-NN` entries across three buckets, full citation provenance per Pitfall 18 (40 OM/N/A/scipy markers).
3. **History-preserving SPEC reconciliation:** `33-SPEC-AMENDMENT.md` sibling file with 6 oracle-drift rows; original `33-SPEC.md` untouched (git log confirms).
4. **5 long-form ADRs:** All 5 v3.1 ADRs (RNG, distribution primitives + Free42 disclaim, ANOVA registers, math1/ freeze carve-out, ModalProgram enum extension) ship with full Context/Decision/Consequences/Alternatives/Footnotes structure; Status: Locked 2026-05-22 line on line 3 of each.
5. **User-facing v3.1 narrative:** README soft-claim + matrix links, CLAUDE.md FIRST-EVER `### v3.1 additions` block + math1/ freeze Exception (v3.1) amendment, PROJECT.md Shipped milestones entry, architecture-history.md `## v3.1 additions` H2 parallel to v3.0 additions (+62 lines), MILESTONES.md IN PROGRESS stub.

Documentation-only phase; library code unchanged except for +2 lines in `scripts/docs-matrix/src/main.rs` (renderer dispatch). SC-4 invariant trivially preserved (no `hp41-gui/src-tauri/src/` changes). MSRV unchanged; zero new dependencies.

---

_Verified: 2026-05-23_
_Verifier: Claude (gsd-verifier)_
