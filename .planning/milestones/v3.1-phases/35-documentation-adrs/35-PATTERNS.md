# Phase 35: Documentation & ADRs — Pattern Map

**Mapped:** 2026-05-23
**Files analyzed:** 15 (8 NEW, 7 MODIFIED)
**Analogs found:** 15 / 15 (100% — Phase 30 is a near-perfect structural twin)

This pattern map reinforces CONTEXT.md `<code_context>` with concrete code excerpts at file:line granularity, and notes one correction (CLAUDE.md does NOT currently carry the `### v3.0 additions` block — the v3.0 narrative lives in `docs/architecture-history.md`; Phase 35 establishes the CLAUDE.md `### v3.x additions` convention as NEW).

---

## File Classification

### NEW files (8)

| File | Role | Data Flow | Closest Analog | Match Quality |
|------|------|-----------|----------------|---------------|
| `docs/hp41-stat1-function-matrix.md` | docs (generated artifact) | transform (JSON → MD) | `docs/hp41-math1-function-matrix.md` | exact (auto-generated from binary) |
| `docs/hp41-stat1-divergences.md` | docs (catalog) | reference | `docs/hp41-math1-divergences.md` | exact (D-30-NN → D-35-NN, swap "Math Pac I / 00041-90034" → "Stat 1 Pac / 00041-90030") |
| `docs/adr/v3.1-001-rng-state-placement.md` | docs (ADR, numeric/policy lock) | reference | `docs/adr/v3.0-003-inv-epsilon.md` | exact (numeric lock + ready-to-paste Rust constant pattern) |
| `docs/adr/v3.1-002-distribution-primitives-policy.md` | docs (ADR, library policy) | reference | `docs/adr/v3.0-005-json-pipeline.md` + `docs/adr/v3.0-002-user-callback-policy.md` | hybrid (json-pipeline for separate-file alternatives shape; user-callback for Free42 disclaim) |
| `docs/adr/v3.1-003-anova-register-layout.md` | docs (ADR, transcription lock) | reference | `docs/adr/v3.0-003-inv-epsilon.md` | exact (OM-transcription numeric lock; "ready-to-paste Rust constant" → "ready-to-paste register-layout consts") |
| `docs/adr/v3.1-004-math1-freeze-second-carve-out.md` | docs (ADR, frozen-invariant amendment) | reference | `docs/adr/v3.0-001-op-strategy.md` | exact (invariant-affecting structural lock; rejected-alternative quoting pattern) |
| `docs/adr/v3.1-005-modalprogram-stat1-enum-extension.md` | docs (ADR, enum-extension lock) | reference | `docs/adr/v3.0-001-op-strategy.md` | exact (4-way exhaustive-match invariant preserver; rejected ~80-line parallel-enum alternative pattern) |
| `.planning/phases/33-…/33-SPEC-AMENDMENT.md` | planning (history-preserving supplement) | reference | NONE direct — net-new pattern; cousin: `.planning/MILESTONES.md` (audit-table style) and Phase 30/30-VERIFICATION.md (drift-table style) | net-new |

### MODIFIED files (7)

| File | Role | Data Flow | Closest Analog (delta-only) | Match Quality |
|------|------|-----------|------------------------------|---------------|
| `scripts/docs-matrix/src/main.rs:65-76` | utility (matrix renderer) | transform (JSON → MD) | existing math1 branch (lines 72-73) | exact (4-line copy-paste) |
| `justfile` (`docs-matrix` + `docs-matrix-check` recipes, lines 186-202) | tooling (task runner) | invoke | existing math1 invocations (lines 190-191, 200-202) | exact (one-line append × 2) |
| `README.md` (line 51-52 region, `## Features`) | docs (top-level) | reference | existing v3.0 soft-claim (line 51-52) | exact (sibling bullet) |
| `CLAUDE.md` (inserts NEW `### v3.x additions` convention) | docs (project-guide) | reference | `docs/architecture-history.md §"v3.0 additions"` (line 117 region) | role-match — see Note A below |
| `CLAUDE.md` `## Frozen Invariants → Core engine` (line 47-56 region) | docs (invariant ledger) | reference | existing math1 freeze line (line 56) | exact (sentence-suffix amendment) |
| `.planning/PROJECT.md` (lines 5-26 region) | docs (planning-state) | reference | existing v3.0 Shipped lines (lines 68-73) | exact (one-line-per-phase append + Current focus update) |
| `docs/architecture-history.md` (new `## v3.1 additions` section after line 188) | docs (long-form history) | reference | existing `## v3.0 additions` (lines 117-188) | exact (sub-section-per-phase narrative) |
| `.planning/MILESTONES.md` (top-of-file or sibling-to-v3.0 stub) | docs (milestone ledger) | reference | existing v3.0 entry at MILESTONES.md (search by `## v3\.0`) | partial — stub only, not full milestone-summary (full one lands Phase 37 `/gsd-complete-milestone`) |

**Note A:** CONTEXT.md `<canonical_refs>` says "CLAUDE.md `### v3.0 additions` block (lines 117–168 region)". This is INCORRECT — `lines 117-168` in CLAUDE.md is the GUI specifics + Free42 contamination guard sections. The v3.0 additions block actually lives in `docs/architecture-history.md` at lines 117-188. The Phase 30 plan 30-03 `30-03-PLAN.md` (line 27) DID prescribe a CLAUDE.md `### v3.0 additions` block but that plan-instruction was not carried out in the final CLAUDE.md (CLAUDE.md was instead restructured into the "Frozen Invariants" + "Tech Stack" + "Key Files" shape it now has). **Phase 35 has a choice:** (a) follow CONTEXT.md's instruction and add the FIRST-EVER `### v3.x additions` block to CLAUDE.md as new structural convention, OR (b) follow the de-facto pattern by adding the narrative ONLY to `architecture-history.md` and skip CLAUDE.md additions block. **Recommendation for planner:** discuss with user — CONTEXT.md D-30.8 lock implies (a); de-facto evidence implies (b). The frozen-invariant amendment (modal.rs carve-out) is independent of this choice and lands either way.

---

## Pattern Assignments

### `docs/hp41-stat1-function-matrix.md` (NEW; auto-generated)

**Analog:** `docs/hp41-math1-function-matrix.md` (auto-generated by `scripts/docs-matrix/src/main.rs`)

**Zero hand-authored content** — produced by running `just docs-matrix` after the `main.rs` basename dispatch gains its third branch (see next entry).

**Generated header pattern** (`scripts/docs-matrix/src/main.rs:78-82`):
```rust
out.push_str(title);
out.push_str("\n\n");
out.push_str(&format!("> Generated from {} via `just docs-matrix`.\n", src));
out.push_str("> Edit the JSON, regenerate this file, commit both.\n\n");
```

**Generated table column pattern** (`scripts/docs-matrix/src/main.rs:105-111`) — `has_xrom` evaluates `true` for Stat 1 (every entry in `hp41-stat1-functions.json` carries an `xrom: { module: "Stat 1", module_id: 2, function_id: N }` block per D-34.1), so the 8-column header renders:
```
| Op | Display | XROM | Category | Status | Phase | Key Path | Description |
|----|---------|------|----------|--------|-------|----------|-------------|
```

**XROM cell renderer** (`scripts/docs-matrix/src/main.rs:133-135`):
```rust
let xrom_cell = e.xrom.as_ref()
    .map(|x| format!("{} / {}-{}", x.module, x.module_id, x.function_id))
    .unwrap_or_else(|| "\u{2014}".to_string());
```
→ Stat 1 rows render as `"Stat 1 / 2-N"`.

**Status filter / Implemented vs Deferred** (`scripts/docs-matrix/src/main.rs:84-95`) — applies unchanged.

---

### `scripts/docs-matrix/src/main.rs:65-76` (MODIFIED, basename dispatch)

**Analog:** the existing math1 branch in the same function (lines 72-73).

**Current code** (lines 70-76):
```rust
let (title, src) = if basename.ends_with("hp41cv-functions.json") {
    ("# HP-41CV ROM Function Matrix", "`docs/hp41cv-functions.json`")
} else if basename.ends_with("hp41-math1-functions.json") {
    ("# HP-41C Math Pac I Function Matrix", "`docs/hp41-math1-functions.json`")
} else {
    ("# Function Matrix", "`{json_path}`")
};
```

**Required edit** — insert one `else if` branch BEFORE the fallthrough:
```rust
} else if basename.ends_with("hp41-stat1-functions.json") {
    ("# HP-41C Stat 1 Pac Function Matrix", "`docs/hp41-stat1-functions.json`")
```

**Invariant preservation** (cited from D-30.1 / file header lines 9-14): the binary's 1-in/1-out CLI signature is unchanged. No `Entry` struct widening. No `args.len() != 3` check change. The `has_xrom` conditional at line 103 already does the right thing for Stat 1 because of D-34.1 schema.

---

### `justfile` `docs-matrix` + `docs-matrix-check` recipes (MODIFIED)

**Analog:** the existing math1 invocation in each recipe.

**Current `docs-matrix` recipe** (lines 186-191):
```just
[group('docs')]
docs-matrix:
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41cv-functions.json docs/hp41cv-function-matrix.md
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41-math1-functions.json docs/hp41-math1-function-matrix.md
```

**Required edit** — append one invocation line after the math1 line:
```just
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41-stat1-functions.json docs/hp41-stat1-function-matrix.md
```

**Current `docs-matrix-check` recipe** (lines 195-202):
```just
[group('docs')]
docs-matrix-check:
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41cv-functions.json /tmp/hp41cv-function-matrix-check.md
	diff -u docs/hp41cv-function-matrix.md /tmp/hp41cv-function-matrix-check.md
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41-math1-functions.json /tmp/hp41-math1-function-matrix-check.md
	diff -u docs/hp41-math1-function-matrix.md /tmp/hp41-math1-function-matrix-check.md
```

**Required edit** — append cargo-run + diff pair:
```just
	cargo run --quiet --manifest-path scripts/docs-matrix/Cargo.toml -- \
		docs/hp41-stat1-functions.json /tmp/hp41-stat1-function-matrix-check.md
	diff -u docs/hp41-stat1-function-matrix.md /tmp/hp41-stat1-function-matrix-check.md
```

**Tmp-file naming** — `/tmp/hp41-stat1-function-matrix-check.md` does not collide with the existing two tmp files (cv + math1). Verified by inspecting current recipe.

**CI gate continuity** — `.github/workflows/ci.yml` already calls `just docs-matrix-check`; no workflow file edits needed.

---

### `docs/hp41-stat1-divergences.md` (NEW)

**Analog:** `docs/hp41-math1-divergences.md`

**Preamble pattern** (math1-divergences.md lines 1-35) — copy verbatim with substitutions:
- `Math Pac I` → `Stat 1 Pac`
- `HP 00041-90034, 1979` → `HP 00041-90030, 1979`
- `D-30-NN` → `D-35-NN`
- `Phase 30 / Plan 30-02 (DOC-04)` → `Phase 35 / Plan 35-02 (STAT-DOC-02)`
- `Pitfall 18 from 28-RESEARCH.md` → `Pitfall 18 from 33-RESEARCH.md`

Per D-35.4 + CONTEXT.md Claude's Discretion, add one v3.1-specific clarifying note in the preamble (~1 sentence): "Stat 1 Pac is the second XROM module in v3.x; numbering follows the phase-origin convention established in v3.0 (D-30-NN for math1; D-35-NN for stat1) per Phase 35 CONTEXT D-35.4."

**Three-bucket section headers** (math1-divergences.md lines 38, 188, 230):
```markdown
## 1. OM Divergences

*(Numerical / behavioral mismatches with OM-quoted examples or OM-described hardware
behavior. ...)*

## 2. Emulator Extensions

*(Functions or behaviors we added that are not present in HP 00041-90030 (1979). ...)*

## 3. Behavioral Policies

*(Cross-cutting rules that are decisions worth documenting — not strictly numerical
divergences ...)*
```
Copy the parenthetical italic descriptions verbatim, swapping `00041-90034` → `00041-90030`.

**Entry shape — 5-field per D-30.5** (math1-divergences.md lines 44-79, the D-30-01 entry):
```markdown
### D-35-NN: <Title>

- **OM citation**: HP 00041-90030 (1979), Chapter N "<chapter title>", page N — "<verbatim quote>".
  (Or `N/A — emulator extension` for bucket-2 entries.)

- **Our behavior**: <what this emulator does>.

- **OM behavior**: <what the OM says or what real hardware does>.

- **Rationale**: <why we made this choice>. <Rejected alternatives, if any, with brief explanation>.

- **See**: <cross-references: ADR links, CONTEXT.md decision IDs, test file pointers,
  Pitfall references from 33-RESEARCH.md>.
```

**Footer trailer** (math1-divergences.md lines 322-329) — copy substituting phase + author:
```markdown
*Last updated: 2026-05-23. Catalog established in Plan 35-02 (Phase 35 / STAT-DOC-02).*

*Next planned update: Phase 37 may add entries for cross-platform numerical-drift
documentation discovered during the STAT-QUAL-04 coverage push, and additional behavioral
specifics cataloged once Phase 37 numerical accuracy testing completes.*
```

**Entry-order per D-35.4 + CONTEXT.md `<specifics>`:**
- Bucket 1 (OM Divergences): D-35-01..06 — the 6 SPEC oracle drifts in SPEC.md original-line-number order (ΣNORMD, ΣAOVONE, ΣSPEAR, ΣEFXSQ, ΣBSTAT, ΣTSTAT); each cross-references `33-SPEC-AMENDMENT.md` row N and the test file:line that asserts the corrected value.
- Bucket 2 (Emulator Extensions): D-35-07..NN — RAND/SEED entry first (cross-references ADR-v3.1-001), then ΣPOLYP `DEGREE=?` UX entry, then any LCG-formula provenance entry.
- Bucket 3 (Behavioral Policies): D-35-(NN+1)..MM — XROM-7 vs XROM-2 prefix convention, ΣTSTAT pooled-only (cites SPEC Req. 25), math1/ freeze second carve-out cross-ref to ADR-v3.1-004, any IN-01..IN-05 follow-ups that fit.

---

### `docs/adr/v3.1-001-rng-state-placement.md` (NEW)

**Analog:** `docs/adr/v3.0-003-inv-epsilon.md` (numeric/policy lock template, ~6 KB)

**Header pattern** (v3.0-003 lines 1-7):
```markdown
# ADR-NNN: <SHORT-NAME> — <Long descriptive title>

**Status:** Locked YYYY-MM-DD
**Owner:** Plan NN-NN Task N
**Requirement refs:** <REQ-IDs>, Pitfall <N> (NN-RESEARCH.md §"<section>")
**Downstream consumer:** `<source-file>` (Plan NN-NN)
**ADR write-up prose:** Phase 35 / STAT-DOC-04 (this is the narrative ADR; the lock happened in Plan 33-XX)
```

**For ADR-v3.1-001 specifically** (per CONTEXT.md Claude's Discretion lines 135-136):
- **Status:** `Locked 2026-05-22` (Phase 33 discuss-phase D-33.4 lock date)
- **Owner:** `Plan 33-01 + Plan 33-08`
- **Requirement refs:** `STAT-RNG-03, Pitfall 18 (33-RESEARCH.md §"RAND/SEED community provenance")` plus the v2.2 serde-default invariant ref
- **Downstream consumer:** `hp41-core/src/state.rs` (`rand_seed: HpNum` field), `hp41-core/src/ops/stat1/random.rs` (LCG impl), `hp41-core/src/state.rs::migrate_after_load()` (D-33.7)
- **ADR write-up prose:** Phase 35 / Plan 35-03

**Section sequence per D-30.6** (mirrors v3.0-003):
1. `## Context` — describe HP-65/Math1 Pac LCG community provenance, RAND/SEED OM absence, serde-skip convention
2. `## Decision` — `rand_seed: HpNum` on `CalcState` with `#[serde(default)]` WITHOUT `#[serde(skip)]` — the only v3.1 exception to the transient-field pattern (D-33.4a)
3. `## Ready-to-paste Rust struct field` (parallels v3.0-003's "Ready-to-paste Rust constant" section, lines 77-100) — show the field declaration + 5–8 line provenance comment block
4. `## Consequences`
5. `## Alternatives Considered`
6. `## Footnotes / References`

**Ready-to-paste comment-block pattern** (v3.0-003 lines 79-94, copy structure swapping `INV_EPSILON` for `rand_seed`):
```rust
// hp41-core/src/state.rs
//
// rand_seed: HpNum — Stat 1 Pac RAND/SEED LCG state (D-33.4 / ADR-v3.1-001).
//
// Source: NPS55-84-003 (Zehna, 1984) p. 21–22 describes the HP Stat 1 Pac LCG
// formula `r_{n+1} = FRC(9821·r_n + 0.211327)`. RAND/SEED are NOT present in the
// Stat 1 Pac Owner's Manual (HP 00041-90030, 1979); they are an emulator extension
// per D-33.4 / D-33.4a documented in D-35-07 (docs/hp41-stat1-divergences.md).
//
// Persistence: #[serde(default)] WITHOUT #[serde(skip)] — this is the only v3.1
// exception to the v1.0+ transient-field convention. Rationale: the LCG state
// is user-visible state (SEED is a deliberate save-point for reproducible
// random sequences), so save-file round-trip must preserve it.
//
// ADR-v3.1-001 (Plan 33-01 / Plan 33-08) locks this placement.
#[serde(default)]
pub rand_seed: HpNum,
```

**Free42 contamination disclaim** — NOT required in ADR-v3.1-001 (RAND/SEED algorithm sourced from NPS / HP-65 community, not Free42). The disclaim is concentrated in ADR-v3.1-002.

---

### `docs/adr/v3.1-002-distribution-primitives-policy.md` (NEW)

**Analog (primary):** `docs/adr/v3.0-005-json-pipeline.md` (tooling/library-choice ADR template, ~11 KB)
**Analog (Free42 disclaim section):** `docs/adr/v3.0-002-user-callback-policy.md` (Free42 disclaim wording, lines 66-70 + 100-110 footnote)

**Header per D-30.6** (mirror v3.0-005 lines 1-13):
- **Status:** `Locked 2026-05-22` (Phase 33 discuss-phase) — CONTEXT.md notes original research SUMMARY.md date `2026-05-21` may be cited in Footnotes
- **Owner:** `Plan 33-02`
- **Requirement refs:** `STAT-CORE-04, STAT-CORE-05, Pitfall 19 (Free42 GPL contamination, 33-RESEARCH.md §"Distribution Primitives")`, plus `.planning/research/SUMMARY.md §"statrs rejection"`
- **Downstream consumer:** `hp41-core/src/ops/stat1/distributions.rs` (~140 LOC), `hp41-core/Cargo.toml` (zero new runtime deps — `rust_decimal 1.42` only)

**Section sequence per D-30.6** (mirrors v3.0-005):
1. `## Context` — describe distribution-quantile need (ΣNORMD, ΣTSTAT, ΣCHISQD, ΣFTEST inverse-CDF paths); cite `.planning/research/SUMMARY.md`'s `statrs` rejection; cite scipy.stats as oracle (NOT source); cite Free42 as oracle (NOT source) per Pitfall 19
2. `## Decision` — hand-coded Acklam quantile + AS 241 inverse-normal + AS 239 chi² inverse + AS 63 Student-t — ~140 LOC in `hp41-core/src/ops/stat1/distributions.rs`; ≥ 6 oracle tuples per primitive validated against scipy.stats inline-constant
3. `## Consequences` — positive (no runtime dep churn, last-digit OM precision controlled in-house) / negative (~140 LOC maintenance) / neutral (future v3.2 Time Pac inherits this pattern if it needs distributions)
4. `## Alternatives Considered` — Option A (chosen): hand-coded primary-source-derived; Option B (rejected): `statrs` runtime dep
5. `## Footnotes / References` — Wichura (AS 241), Cody (AS 239), Lentz (AS 63), NPS55-84-003, scipy.stats, Free42 disclaim citation

**Free42 disclaim sentence per Pitfall 19** (copy from v3.0-002 line 66 verbatim, swapping the algorithm names):
```markdown
Algorithm independently re-derived from Wichura (AS 241), Cody (AS 239), and Lentz (AS 63)
primary sources; Free42 source consulted only as sanity-check oracle, not copied.
```

This sentence must appear at least once in `## Decision` or `## Consequences` so `scripts/check-free42-contamination.sh`'s grep-detectability is satisfied (the 12-symbol guard expects the disclaim header; the ADR is reference-only material, but the sentence-as-grep-target convention from v3.0-002 is preserved).

**Rejected-alternative quoting per D-30.7** (parallel to v3.0-005 lines 152-162):
- Quote `.planning/research/SUMMARY.md`'s `statrs` rejection paragraph verbatim in the blockquote
- Add scipy.stats citation as the oracle-but-not-source pattern (parallel to v3.0-005's hp41cv-JSON-was-not-extended pattern)

---

### `docs/adr/v3.1-003-anova-register-layout.md` (NEW)

**Analog:** `docs/adr/v3.0-003-inv-epsilon.md` (OM-transcription numeric lock, ~6 KB)

**Header per D-30.6:**
- **Status:** `Locked 2026-05-22` (Phase 33 Plan 33-00 + SPEC.md Req. 30..35 lock date)
- **Owner:** `Plan 33-00 + Plan 33-06`
- **Requirement refs:** `STAT-CORE-08 (ANOVA register layout), STAT-CORE-09, Pitfall 21 (silent-wrong-answer trap on register-layout-dependent set), Pitfall 18 (OM citation discipline)`
- **Downstream consumer:** `hp41-core/src/ops/stat1/mod.rs` (`//!` doc header + per-Op named consts), `hp41-core/src/ops/stat1/anova.rs`, `hp41-core/src/ops/stat1/regression.rs`

**Section sequence per D-30.6** (mirrors v3.0-003 structure):
1. `## Context` — describe OM Storage Registers section for ΣMMTUG / ΣAOVONE / ΣMLRXY / ΣCTKKK; explain P21 trap (silent wrong answers when register indices drift); cite NPS55-84-003 cross-reference where it strengthens the OM citation
2. `## Decision` — OM 00041-90030 "Storage Registers" transcribed into `stat1/mod.rs` `//!` header + per-Op named consts (e.g., `const ANOVA_GROUP_COUNT_REG: usize = 17;`); P21 mitigation via named-const indirection
3. `## Ready-to-paste Rust constants` (parallel to v3.0-003 lines 77-100) — show the `//!` header and the 4 register-layout const blocks (one per ΣMMTUG / ΣAOVONE / ΣMLRXY / ΣCTKKK)
4. `## Consequences`
5. `## Alternatives Considered`
6. `## Footnotes / References` — OM 00041-90030 page + NPS55-84-003 §

---

### `docs/adr/v3.1-004-math1-freeze-second-carve-out.md` (NEW)

**Analog:** `docs/adr/v3.0-001-op-strategy.md` (frozen-invariant-affecting structural lock, ~11 KB)

**Header per D-30.6:**
- **Status:** `Locked 2026-05-22` (Phase 33 plan-phase user-confirmed D-33.3b)
- **Owner:** `Phase 33 plan-phase (D-33.3b)`
- **Requirement refs:** `STAT-DOC-05 (frozen-invariant amendment), CLAUDE.md "Frozen Invariants → Core engine" amendment target`
- **Downstream consumer:** `CLAUDE.md` (Frozen Invariants amendment), `hp41-core/src/ops/math1/modal.rs` (~8-line dispatch wiring), `hp41-core/src/ops/stat1/modal.rs` (`Stat1Step` semantics)

**Section sequence per D-30.6** (mirror v3.0-001):
1. `## Context` — recap the math1/ freeze invariant (frozen since Plan 25-01); describe Phase 33 the SECOND carve-out need (first was xrom.rs per D-33.3 / ADR-v3.0-001 indirect lineage; second is modal.rs per D-33.3b)
2. `## Decision` — `hp41-core/src/ops/math1/modal.rs` carved out for an 8-line `ModalProgram::Stat1(_) => stat1::modal::dispatch(...)` arm; `Stat1Step` enum and its dispatch logic live entirely in NEW `hp41-core/src/ops/stat1/modal.rs` so no Stat 1 Pac code leaks into frozen math1/
3. `## Consequences` — CLAUDE.md `## Frozen Invariants → Core engine` amendment listing `xrom.rs` AND `modal.rs` as documented carve-outs; future v3.2+ pacs (Time, Advantage) follow this precedent for their own modal-flow infrastructure additions
4. `## Alternatives Considered` — quote 33-CONTEXT.md D-33.3b rejected-alternative verbatim per D-30.7: the rejected "parallel `Stat1ModalProgram` enum" approach that would have duplicated ~80 lines of cross-frontend modal infrastructure
5. `## Footnotes / References` — ADR-v3.0-001 (Op-strategy precedent), `calc.fjk.ch/db/hp41mod.php` (Stat 1B XROM #2 confirmation), 33-CONTEXT.md D-33.3a / D-33.3b lines

**Rejected-alternative blockquote pattern** (v3.0-001 lines 149-180, structure):
```markdown
## Alternatives Considered

### Option B: Parallel `Stat1ModalProgram` Enum

The rejected alternative is described verbatim in 33-CONTEXT.md D-33.3b (locked 2026-05-22):

> **D-33.3b (rejected alternative):** Add a parallel `Stat1ModalProgram` enum
> alongside `ModalProgram` to avoid carving out math1/modal.rs a second time.
> Each frontend (CLI + GUI) would dispatch on EITHER variant.
>   - **Why rejected:** ~80 lines of duplicated dispatch logic across both
>     frontends, plus the 4-way exhaustive-match invariant would lose its
>     load-bearing strength because `Op` variants targeting Stat1 modal flows
>     would only appear in `Stat1ModalProgram` arms, not in the canonical
>     `ModalProgram` arms.

**Additional context on why Option B was rejected:**
[Planner adds 2-3 paragraphs of additional rationale beyond the verbatim quote]
```

---

### `docs/adr/v3.1-005-modalprogram-stat1-enum-extension.md` (NEW)

**Analog:** `docs/adr/v3.0-001-op-strategy.md` (4-way exhaustive-match invariant preserver lock)

**Header per D-30.6:**
- **Status:** `Locked 2026-05-22` (Phase 33 plan-phase user-confirmed D-33.3b, same session as ADR-v3.1-004)
- **Owner:** `Phase 33 plan-phase (D-33.3b)`
- **Requirement refs:** `STAT-CORE-06 (modal-workflow extensibility), CLAUDE.md "4-way exhaustive-match invariant"`
- **Downstream consumer:** `hp41-core/src/state.rs` (`ModalProgram` enum gets one new variant), `hp41-core/src/ops/stat1/modal.rs` (`Stat1Step` enum + dispatch), CLI + GUI `pending_prompt()` (one new arm)

**Section sequence per D-30.6** (mirror v3.0-001):
1. `## Context` — describe `ModalProgram` enum's 6 v3.0 variants (Matrix / Solve / Poly / Integ / Difeq / Four); STAT 1 Pac needs at least 4 modal flows (ANOVA / Regression / Distribution / Random) but unifying them under a single `Stat1(Stat1Step)` variant minimizes the dispatch surface
2. `## Decision` — extend `ModalProgram` with ONE new variant: `Stat1(Stat1Step)`. `Stat1Step` is a separate enum in `hp41-core/src/ops/stat1/modal.rs` carrying step-level semantics. The 4-way exhaustive-match invariant is preserved: every match-arm on `ModalProgram` adds one new `Stat1(_) => ...` line
3. `## Consequences` — minimal blast radius (one new arm per match site); future v3.2 Time Pac follows the same extension pattern (`ModalProgram::Time(TimeStep)`); CLAUDE.md "4-way exhaustive-match invariant" section unaffected
4. `## Alternatives Considered` — quote 33-CONTEXT.md D-33.3b rejected alternative verbatim per D-30.7 (parallel `Stat1ModalProgram` enum — same rejection as ADR-v3.1-004 but viewed from the enum-design angle rather than the math1/-freeze angle)
5. `## Footnotes / References` — ADR-v3.0-001 (Op-strategy precedent), ADR-v3.1-004 (math1/ freeze cross-reference), CLAUDE.md 4-way invariant section

---

### `.planning/phases/33-…/33-SPEC-AMENDMENT.md` (NEW)

**Analog:** NONE direct — this is a net-new history-preserving pattern established by D-35.1.
**Closest cousin (table style):** Phase 30/30-VERIFICATION.md drift-listing rows OR `.planning/MILESTONES.md` audit-table rows.

**Frontmatter pattern** (CONTEXT.md Claude's Discretion):
```markdown
---
title: 33-SPEC Amendment — Oracle Drift Reconciliation
date: 2026-05-23
amendment_author: Phase 35 / Plan 35-01
amends: .planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-SPEC.md
cross_reference: .planning/phases/33-…/33-VERIFICATION.md (lines 13-22, 186-187 — original drift listing)
preserves_archaeology: yes
---
```

**Body — 6-row table per CONTEXT.md `<specifics>` shape:**
```markdown
# 33-SPEC Amendment — Oracle Drift Reconciliation

This sibling file supplements (does NOT modify) `33-SPEC.md`. The original SPEC.md
oracle values stay frozen — planning-phase archaeology preserved per D-35.1. The
test-asserted ground truth is now the scipy-correct value documented below; each row
cross-references a D-35-NN entry in `docs/hp41-stat1-divergences.md` (bucket 3 —
Behavioral Policies, since these are mathematical-ground-truth corrections rather than
OM divergences).

| # | SPEC.md row | Original oracle | Scipy-correct value | Test file:line | Drift root cause | D-35-NN |
|---|-------------|-----------------|---------------------|----------------|------------------|---------|
| 1 | Req. NN — ΣNORMD CDF tolerance | `1e-9` tolerance band | `1e-5` tolerance band | `hp41-core/tests/stat1_normd.rs:NNN` | rust_decimal A&S 6 precision limit | D-35-01 |
| 2 | Req. NN — ΣAOVONE F-ratio | `100.0` | `50.0` | `hp41-core/tests/stat1_anova.rs:NNN` | F-ratio formula was misread from OM worked example; scipy.stats.f_oneway oracle | D-35-02 |
| 3 | Req. NN — ΣSPEAR rank ρ_s | `0.7` | `0.8` | `hp41-core/tests/stat1_spear.rs:NNN` | Rank-correlation formula tie-handling drift; scipy.stats.spearmanr oracle | D-35-03 |
| 4 | Req. NN — ΣEFXSQ χ² calibration | <orig value> | <scipy value> | `hp41-core/tests/stat1_efxsq.rs:NNN` | χ² calibration drift | D-35-04 |
| 5 | Req. NN — ΣBSTAT CV | `0.4083` | `0.5270` | `hp41-core/tests/stat1_bstat.rs:NNN` | CV formula off-by-one in summation; scipy.stats.variation oracle | D-35-05 |
| 6 | Req. NN — ΣTSTAT deep-tail | <orig precision> | <AS 63 limit> | `hp41-core/tests/stat1_tstat.rs:NNN` | AS 63 precision limit for deep-tail Student-t p | D-35-06 |
```

**Plan 35-01 fills in the exact `Req. NN`, `<orig value>`, `<scipy value>`, and `file:line`** by reading 33-SPEC.md and the relevant test files.

---

### `README.md` (MODIFIED)

**Analog:** the existing v3.0 soft-claim under `## Features` (line 51-52).

**Current code** (lines 51-52):
```markdown
- v3.0 ships Math Pac I behavioral emulation, feature-complete per Owner's Manual 00041-90034
  ([documented divergences](docs/hp41-math1-divergences.md)) — see [Math Pac I Function Matrix](docs/hp41-math1-function-matrix.md)
```

**Required edit** — insert immediately below as a sibling bullet (D-35.3 soft-claim wording locked):
```markdown
- Stat 1 Pac behavioral emulation (13 programs, 26 XEQ entry points, RAND/SEED extension,
  [documented divergences](docs/hp41-stat1-divergences.md)) — see
  [Stat 1 Pac Function Matrix](docs/hp41-stat1-function-matrix.md)
```

**Hard-claim deferred** per D-35.3 → Phase 37 / `/gsd-ship` rewrites the soft-claim to hard-claim conditional on STAT-QUAL-04 + STAT-QUAL-11.

**Optional: `## Releases` table entry** (README lines 27-36) — Plan 35-04 can add a stub `v3.1` row at the top of the table OR defer that to Phase 37 / `/gsd-complete-milestone`. Recommendation: defer (matches v3.0 pattern — README Releases row only lands at the milestone-ship `gsd-ship`, not at the docs phase).

---

### `CLAUDE.md` `### v3.1 additions` block (NEW SECTION — see Note A above)

**Analog (structural):** `docs/architecture-history.md §"v3.0 additions"` (lines 117-188) — the per-phase narrative.
**Analog (intended-by-30-03-PLAN-but-never-shipped):** the prescriptive shape in 30-03-PLAN.md line 27.

**Sub-section heading pattern** (architecture-history.md lines 119, 135, 146, 155, 159):
```markdown
### v3.1 additions (Stat 1 Pac Emulation, Phases 33–35 — 36–37 IN PROGRESS)

#### Phase 33 — XROM Activation + Distribution Primitives + All Stat 1 Ops (shipped 2026-05-22)

[2–4 paragraphs of narrative; cite ADR-v3.1-001/002/003/004/005; cite D-33.x decisions]

#### Phase 34 — CLI Integration (shipped 2026-05-23)

[2–4 paragraphs of narrative; cite D-34.1/3 etc.]

#### Phase 35 — Documentation & ADRs (shipped YYYY-MM-DD)

[2–4 paragraphs of narrative; cite this Phase's deliverables]

#### Phase 36 — GUI Integration (in progress)

[stub paragraph]

#### Phase 37 — Test Hardening & Quality Gates (in progress)

[stub paragraph]
```

**Per-bullet pattern per phase** (architecture-history.md line 121 example):
```markdown
- **<lock-name> locked (ADR-v3.1-NNN / D-NN.x):** <one-sentence description of what was locked> — <one-sentence rationale>. Full write-up: `docs/adr/v3.1-NNN-<slug>.md`.
```

**Tail note pattern** (architecture-history.md lines 181-187) — "Frozen invariants preserved across v3.1: ..." sub-paragraph at the end of the block. Mirrors the v3.0 tail note verbatim, swapping invariant descriptions to match v3.1 (`math1/modal.rs` is now a documented second carve-out per ADR-v3.1-004).

**Insertion point in CLAUDE.md** — CONTEXT.md says "immediately after the `### v3.0 additions` block (line 117 region in current CLAUDE.md)". Since the v3.0 additions block does NOT exist in CLAUDE.md (it lives in `architecture-history.md`), the planner must choose:
- (a) Add a NEW pair of blocks (`### v3.0 additions` summary + `### v3.1 additions` summary) immediately after `## Frozen Invariants` and before `## Tech Stack` (line 119-120 region). This creates a precedent that goes forward but back-fills v3.0.
- (b) Add only `### v3.1 additions` and leave the v3.0 reference structure unchanged. Cleaner but inconsistent.

**Recommendation:** discuss with user during planning. CONTEXT.md D-35.3 lock implies (a) is the locked choice; the de-facto state implies user may not have wanted CLAUDE.md to carry duplicate narrative. **Plan 35-04 should surface this decision explicitly before writing.**

---

### `CLAUDE.md` `## Frozen Invariants → Core engine` amendment (MODIFIED)

**Analog:** existing math1 freeze line (line 56).

**Current text** (line 56):
```markdown
- **`hp41-core/src/ops/math1/` is frozen** since Plan 25-01. Math Pac I algorithms re-derived from HP OM 00041-90034 (1979); Free42 consulted as sanity-check oracle only, **not** copied. Every file in this directory carries the verbatim disclaim header.
```

**Required amendment** (per CONTEXT.md Claude's Discretion line 148):
Append after the existing sentence (gated by ADR-v3.1-004 Status: Locked):
```markdown
**Exceptions:** `xrom.rs` (D-33.3 / ADR-v3.1-004 — XROM registry for v3.1+ extension; bit-1 stub was always intended for v3.1+) and `modal.rs` (D-33.3b / ADR-v3.1-004 — ~8-line dispatch wiring for `ModalProgram::Stat1` variant; `Stat1Step` semantics live in the new `stat1/modal.rs` so no Stat 1 Pac code leaks into the frozen module).
```

**Substance is fixed; wording is planner discretion** per CONTEXT.md.

---

### `.planning/PROJECT.md` (MODIFIED)

**Analog:** existing v3.0 Shipped lines (lines 68-73) + Current Milestone block (lines 9-25).

**Current "Shipped milestones" structure** (lines 62-73) — flat bullet list, one bullet per milestone with sub-bullets per phase.

**Required edits:**
1. **Current State + Current Milestone block (lines 5-25)** — update once Phase 35 ships:
   - Line 5 `**Last shipped:** v3.0 ...` → after Phase 37 milestone ship this becomes `**Last shipped:** v3.1 ...`. Phase 35 alone does NOT update the milestone Shipped — Phase 35 is an in-flight milestone phase.
   - Line 7 `**Status:** v3.1 Stat 1 Pac Emulation — planning phase ...` → update to `Phase 35 — Documentation & ADRs` or to `Phase 36 — GUI Integration` after Phase 35 ships.
2. **Per-phase one-liner additions in the Current Milestone block** (parallel to v3.0 sub-bullets at lines 69-73) — append once Phase 35 ships:
   ```markdown
   - Phase 33 XROM Activation + Distribution Primitives + All Stat 1 Ops (shipped 2026-05-22) — `hp41-core` only; N plans; …
   - Phase 34 CLI Integration (shipped 2026-05-23) — `hp41-cli` only; 2 plans; …
   - Phase 35 Documentation & ADRs (shipped YYYY-MM-DD) — `docs/` + tooling only; 4 plans; …
   ```
3. **(Optional per D-30.8 Claude's Discretion)** `### v3.1 additions` block in PROJECT.md — CONTEXT.md recommends HOLDING the v3.0 pattern (PROJECT.md gets only Shipped/Current-focus updates; full additions block lives in CLAUDE.md and architecture-history.md). **Recommendation: hold; planner picks final shape.**

---

### `docs/architecture-history.md` (MODIFIED — new `## v3.1 additions` section)

**Analog:** `## v3.0 additions (Math Pac I Emulation, Phases 28–32)` section (lines 117-188).

**Section structure to mirror** (architecture-history.md):
- Line 117: `### v3.0 additions (Math Pac I Emulation, Phases 28–32)`
- Lines 119-133: `#### Phase 28 — XROM Framework + Math Pac I Core Ops (shipped 2026-05-16)` with ~14 bullet points
- Lines 135-144: `#### Phase 29 — CLI Integration (shipped 2026-05-17)` with ~6 bullets
- Lines 146-153: `#### Phase 30 — Documentation & ADRs (shipped 2026-05-17)` with ~6 bullets
- Lines 155-157: `#### Phase 31 — GUI Integration (shipped 2026-05-18)` with single narrative paragraph
- Lines 159-166: `#### Phase 32 — Test Hardening & Quality Gates (shipped 2026-05-18)` with ~6 bullets
- Lines 168-179: `#### Post-graduation polish before v3.0 ship (2026-05-20, quick-task batch on develop)` with 6 numbered post-ship items
- Lines 181-187: `**Frozen invariants preserved across v3.0:**` tail note

**Insertion point** — new section immediately AFTER line 188 (the `---` separator before `## Quality Gate History`), BEFORE the `## Quality Gate History` heading at line 190.

**Phase 35-specific shape** (parallels v3.0 but for Phases 33–37):
```markdown
### v3.1 additions (Stat 1 Pac Emulation, Phases 33–35 — 36–37 IN PROGRESS)

#### Phase 33 — XROM Activation + Distribution Primitives + All Stat 1 Ops (shipped 2026-05-22)

[2–4 paragraphs / ~10–14 bullets]

#### Phase 34 — CLI Integration (shipped 2026-05-23)

[~6 bullets]

#### Phase 35 — Documentation & ADRs (shipped YYYY-MM-DD)

[~6 bullets covering the matrix extension, divergence catalog, 5 ADRs, SPEC amendment, narrative updates]

#### Phase 36 — GUI Integration (in progress)

[stub paragraph: narrative fills in at Phase 36 ship-time]

#### Phase 37 — Test Hardening & Quality Gates (in progress)

[stub paragraph: narrative fills in at Phase 37 ship-time]

**Frozen invariants preserved across v3.1:**
- SC-4 invariant: …
- 4-exhaustive-match invariant: every new `Op` variant (~26 Stat 1) landed in `dispatch()` + `execute_op()` + both `prgm_display.rs` copies …
- math1/ freeze: per ADR-v3.1-004, `xrom.rs` + `modal.rs` are documented carve-outs; the rest of `hp41-core/src/ops/math1/` is bit-identical to v3.0 …
- Save-file backward compat: `rand_seed: HpNum` field carries `#[serde(default)]` per ADR-v3.1-001 (note: NOT `serde(skip)` — the v3.1-specific exception) …
- MSRV 1.88 unchanged through Phases 33–35. Zero new runtime deps (no `statrs` per ADR-v3.1-002).
```

**OM-citation discipline** (per CONTEXT.md `<canonical_refs>` line 217) — every narrative paragraph that cites a behavior cites HP 00041-90030 page + example OR the NPS document section OR explicit emulator-extension marker.

---

### `.planning/MILESTONES.md` (MODIFIED — v3.1 stub)

**Analog:** existing v1.0 / v1.1 / v2.0 / v2.2 / v3.0 milestone-summary entries.

**Current pattern** (MILESTONES.md lines 1-48 show v1.0 milestone-summary structure: `## vN.N — <Name>` + `**Status:**` + `**Phases:**` + `### Delivered` + `### Key Accomplishments` + `### Quality at Ship` + `### Archives` + `### Known Deferred Items`).

**Phase 35 lands a one-liner stub** (CONTEXT.md "Out of scope: `/gsd-complete-milestone` v3.1 ship — Phase 37 post-ship. Phase 35 lands a `.planning/MILESTONES.md` stub; full milestone-summary one-liner + ROADMAP archive moves at Phase 37 ship.").

**Suggested stub shape** (one paragraph between v3.0 and the start of any in-progress section):
```markdown
## v3.1 — HP-41 Calculator Emulator Stat 1 Pac Emulation

**Status:** IN PROGRESS (Phases 33–35 shipped 2026-05-22..23; Phases 36–37 pending)
**Phases:** Phases 33–37 (5 phases planned)
**Plans (so far):** 33-00 through 35-04
**Source-of-truth:** [.planning/PROJECT.md](PROJECT.md) — full milestone scope + decisions ledger; full milestone-summary lands at Phase 37 ship via `/gsd-complete-milestone`.

(Full delivery summary, quality table, and known-deferred items will be added at Phase 37 milestone-ship time.)
```

Full milestone-summary one-liner + ROADMAP archive moves at Phase 37 ship via `/gsd-complete-milestone`.

---

## Shared Patterns

### ADR header convention (D-30.6)

**Source:** `docs/adr/v3.0-003-inv-epsilon.md` lines 1-7 (the most compact / canonical header).

**Apply to:** all 5 ADRs (v3.1-001 through v3.1-005)

```markdown
# ADR-NNN: <SHORT-NAME> — <Long descriptive title>

**Status:** Locked YYYY-MM-DD
**Owner:** Plan NN-NN Task N  (OR  Phase NN plan-phase (D-NN.x))
**Requirement refs:** <REQ-IDs>, Pitfall <N> (NN-RESEARCH.md §"<section>")
**Downstream consumer:** `<source-file>` (Plan NN-NN), `<another>` (Plan NN-NN)
**ADR write-up prose:** Phase 35 / Plan 35-03 (this is the narrative ADR; the lock happened in <where>)

---

## Context

[2–4 paragraphs setting up the problem, citing OM page, citing community evidence,
naming any rejected positions]

---

## Decision

**<Statement of locked choice>.**

[2–4 paragraphs explaining the decision, with sub-bullets for sub-choices.]

---

## (Optional) Ready-to-paste Rust <constant|struct|consts>

[For numeric or struct-shape locks only — show the actual Rust code the
downstream consumer can copy-paste verbatim. Mirror v3.0-003 lines 77-100.]

---

## Consequences

### Positive
- ...

### Negative
- ...

### Neutral
- ...

---

## Alternatives Considered

### Option B: <Rejected alternative name>

The rejected alternative is described verbatim in NN-CONTEXT.md D-NN.NN (locked YYYY-MM-DD):

> [verbatim blockquote from CONTEXT.md]

**Additional context on why Option B was rejected:**
[2–3 paragraphs of additional rationale]

---

## Footnotes / References

[^1]: <citation>
[^2]: <citation>
...

---

*ADR-NNN locked: YYYY-MM-DD. Plan NN-NN Task N.*
*OM reference: HP 00041-90030, 1979 — <chapter/page>.*
```

### Citation provenance discipline (Pitfall 18)

**Source:** `docs/adr/v3.0-003-inv-epsilon.md` lines 28-56 (Context section's OM transcription + community cross-verification pattern).

**Apply to:** all 5 ADRs + every entry in `docs/hp41-stat1-divergences.md`

Every assertion must cite at least one of:
1. **OM page** — `HP 00041-90030 (1979), Chapter N, page N — "verbatim quote"`
2. **NPS document** — `Naval Postgraduate School NPS55-84-003 (Zehna, 1984), §N, p. N`
3. **MoHPC URL** — `https://www.hpmuseum.org/forum/thread-NNNNN.html` (real URLs only)
4. **Mike Sebastian forensic** — `https://www.rskey.org/~mwsebastian/miscprj/forensic.htm`
5. **Free42 disclaim marker** — `"N/A — independently re-derived; Free42 consulted as sanity-check oracle only"` (NOT a citation; a disclaim)
6. **Scipy oracle citation** — `scipy.stats.<function>` inline-constant (validated against version N.N.N) — for D-35.1 oracle-drift entries
7. **Emulator-extension marker** — `"N/A — emulator extension"` (explicit non-citation for bucket-2 entries)

No uncited assertions permitted.

### Free42 GPL contamination disclaim (Pitfall 19)

**Source:** `docs/adr/v3.0-002-user-callback-policy.md` line 66.

**Apply to:** ADR-v3.1-002 ONLY (distribution primitives policy — the only v3.1 ADR where Free42 was an oracle).

**Verbatim sentence** (modify the algorithm names but keep the sentence shape):
```
Algorithm independently re-derived from Wichura (AS 241), Cody (AS 239), and Lentz (AS 63)
primary sources; Free42 source consulted only as sanity-check oracle, not copied.
```

Must appear in either `## Decision` or `## Consequences` so the policy is grep-detectable when a future contamination-audit scan covers `docs/adr/`.

### 5-field divergence entry shape (D-30.5)

**Source:** `docs/hp41-math1-divergences.md` lines 44-79 (the D-30-01 entry).

**Apply to:** every entry in `docs/hp41-stat1-divergences.md` (all three buckets).

See the per-file pattern assignment for `docs/hp41-stat1-divergences.md` above for the entry template.

### Verbatim rejected-alternative quoting (D-30.7)

**Source:** `docs/adr/v3.0-005-json-pipeline.md` lines 150-162 (the Option B verbatim blockquote pattern).

**Apply to:** all 5 ADRs (each must quote one Phase 33 CONTEXT.md D-33.x rejected alternative verbatim).

**Pattern:**
```markdown
### Option B: <Rejected option name>

The rejected alternative is described verbatim in 33-CONTEXT.md D-33.NN (locked YYYY-MM-DD):

> [verbatim quote of the rejected-alternative paragraph from CONTEXT.md]

**Additional context on why Option B was rejected:**
[2–3 paragraphs of additional rationale beyond the verbatim quote]
```

### Save-file backward-compat invariant (CLAUDE.md "Frozen Invariants → Save-file backward compat")

**Source:** CLAUDE.md lines 79-83.

**Apply to:** ADR-v3.1-001 (the only v3.1 lock that affects this invariant; `rand_seed: HpNum` carries `#[serde(default)]` WITHOUT `skip` — explicit deviation requiring ADR documentation).

The ADR-v3.1-001 `## Consequences` section must explicitly call out that v1.0–v3.0 save files load without migration AND that v3.1+ save files round-trip `rand_seed` (RAND/SEED reproducibility requirement).

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `.planning/phases/33-…/33-SPEC-AMENDMENT.md` | planning supplement | reference | **Net-new pattern established by D-35.1**. No prior Phase has shipped a SPEC-AMENDMENT (Phase 30's drift listings landed in 30-VERIFICATION.md as ship-time findings, not as a separate supplement file). This is a new convention for v3.1+ to inherit. Closest cousin in form: the audit-table rows in `.planning/MILESTONES.md` v1.0 archives. Closest cousin in purpose: 30-VERIFICATION.md drift-listing rows. |

This file's lack of direct analog is the strongest signal in the pattern map: D-35.1 is the load-bearing new convention this Phase establishes. Plan 35-01 has discretion on the exact frontmatter shape and table-column order, but the 6 drift rows + cross-references to `D-35-NN` divergence entries are fixed.

---

## Metadata

**Analog search scope:**
- `docs/adr/` (5 v3.0 ADRs read)
- `docs/hp41-math1-divergences.md` (preamble + 3 bucket headers + 2 sample entries read)
- `docs/hp41-math1-function-matrix.md` (auto-generated; no hand-authored content)
- `docs/architecture-history.md` (lines 117-188 — `### v3.0 additions` block + tail note)
- `CLAUDE.md` (lines 1-95 — Frozen Invariants section)
- `scripts/docs-matrix/src/main.rs` (full file — 167 lines)
- `justfile` (lines 181-202 — docs section)
- `.planning/PROJECT.md` (lines 1-80 — Current State + History)
- `.planning/MILESTONES.md` (lines 1-60 — v1.0 entry as canonical milestone-summary shape)
- `.planning/milestones/v3.0-phases/30-documentation-adrs/30-01-PLAN.md` (lines 1-80)
- `.planning/milestones/v3.0-phases/30-documentation-adrs/30-02-PLAN.md` (lines 1-60)
- `.planning/milestones/v3.0-phases/30-documentation-adrs/30-03-PLAN.md` (lines 1-80)
- `README.md` (lines 1-60)

**Files scanned:** 13 analog files + 1 CONTEXT.md + 1 ROADMAP.md + 1 REQUIREMENTS.md.

**Key correction noted vs CONTEXT.md `<code_context>`:**
- CONTEXT.md `<canonical_refs>` says `CLAUDE.md ### v3.0 additions block (lines 117-168 region)`. This is INCORRECT — the v3.0 additions narrative lives in `docs/architecture-history.md` lines 117-188, NOT in CLAUDE.md. CLAUDE.md lines 117-168 cover GUI specifics + Free42 contamination guard sections. Phase 35 planner must decide explicitly whether to introduce the FIRST-EVER `### v3.x additions` block in CLAUDE.md as a new structural convention (Recommendation: discuss with user — CONTEXT.md D-30.8 lock implies yes; de-facto state implies the narrative belongs in architecture-history.md). Either choice is implementable from the patterns mapped here; the recommended Plan 35-04 should surface the decision.

**Pattern extraction date:** 2026-05-23
