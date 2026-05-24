---
phase: 35-documentation-adrs
reviewed: 2026-05-23T00:00:00Z
depth: standard
files_reviewed: 9
files_reviewed_list:
  - docs/adr/v3.1-001-rng-state-placement.md
  - docs/adr/v3.1-002-distribution-primitives-policy.md
  - docs/adr/v3.1-003-anova-register-layout.md
  - docs/adr/v3.1-004-math1-freeze-second-carve-out.md
  - docs/adr/v3.1-005-modalprogram-stat1-enum-extension.md
  - docs/architecture-history.md
  - docs/hp41-stat1-divergences.md
  - docs/hp41-stat1-function-matrix.md
  - scripts/docs-matrix/src/main.rs
findings:
  critical: 0
  blocker: 0
  warning: 6
  info: 7
  total: 13
status: issues_found
---

# Phase 35: Code Review Report

**Reviewed:** 2026-05-23
**Depth:** standard
**Files Reviewed:** 9 (8 markdown + 1 Rust)
**Status:** issues_found

## Summary

Phase 35 ships the v3.1 documentation deliverables: five long-form ADRs, the
Stat 1 Pac divergence catalog, an updated architecture-history narrative, an
auto-generated function matrix, and a two-line extension to the docs-matrix
renderer. The corpus is well-cross-referenced and the citation discipline
(Pitfall 18) is consistently applied; line numbers cited in the ADRs were
spot-checked against `hp41-core/src/state.rs` and `hp41-core/src/ops/math1/`
and verified accurate.

No BLOCKER findings. The Rust delta in `scripts/docs-matrix/src/main.rs` is
symmetric with the sibling branches and introduces no new failure modes.

WARNING-tier findings center on three classes of defect:

1. **Documentation/code drift** — ADR-v3.1-004 and ADR-v3.1-005 enumerate the
   v3.0 `ModalProgram` enum as "6 v3.0 variants (Matrix / Solve / Poly / Integ
   / Difeq / Four)" but the actual enum carries **7** v3.0 variants — `Trans(TransInputStep)`
   is also there (`hp41-core/src/ops/math1/modal.rs:38`). Every claim of "8-line
   wiring delta" was sized against the wrong baseline.
2. **Misleading provenance claim** in ADR-v3.1-001: the "Mirror pattern:
   `complex_mode: bool` is the IDENTICAL `#[serde(default)]` (no skip) shape
   — the ONLY existing precedent" is technically true for v3.0 `CalcState`
   additions but ignores the v1.0–v2.x persistent fields (`stack`, `program`,
   `registers`, `xrom_modules`, etc.) which are also `#[serde(default)]` without
   `#[serde(skip)]`. The claim's framing exaggerates the uniqueness of the
   pattern.
3. **Internal contradiction in the divergence catalog**: D-35-04 cites
   `33-SPEC-AMENDMENT.md` rows "4 / 5 / 6" but D-35-05 cites
   `33-SPEC-AMENDMENT.md` row "5" — the same row cannot belong to both
   entries. One of the two references is wrong.

INFO-tier findings cover terminology consistency (RAND/SEED status string in
the auto-generated matrix), wording precision (the carve-out file count), and
documentation-history corrections that should land in a follow-up.

## Structural Findings (fallow)

_No `<structural_findings>` block was provided in the prompt; no structural
pre-pass substrate to integrate._

## Narrative Findings (AI reviewer)

## Warnings

### WR-01: ADR-v3.1-004 + ADR-v3.1-005 enumerate the wrong v3.0 `ModalProgram` variant count

**Files:**
- `docs/adr/v3.1-004-math1-freeze-second-carve-out.md:44, 80`
- `docs/adr/v3.1-005-modalprogram-stat1-enum-extension.md:14-25, 66, 86, 192`

**Issue:** Both ADRs describe the pre-v3.1 `ModalProgram` enum as carrying
6 variants:

> ADR-v3.1-004 line 44: "v3.0's `pub enum ModalProgram { Matrix, Solve, Poly, Integ, Difeq, Four }` is the single source of truth"
> ADR-v3.1-004 line 80: "alongside the 6 v3.0 variants"
> ADR-v3.1-005 lines 16–25: identical 6-variant enum reproduction
> ADR-v3.1-005 line 66: "alongside the 6 v3.0 Math 1 variants"
> ADR-v3.1-005 line 86: "Matrix, Solve, Poly, Integ, Difeq, Four"
> ADR-v3.1-005 line 192: "alongside `Matrix`, `Solve`, `Poly`, `Integ`, `Difeq`, `Four`"

The actual enum in `hp41-core/src/ops/math1/modal.rs:24-52` carries **7**
v3.0 variants: `Matrix, Solve, Poly, Integ, Difeq, Four, Trans` (Trans at
line 38, "TRANS workflow (Plan 28-10): coordinate transform setup."). The
delegation arms in `current_prompt()` at lines 65–77 and `requires_alpha_label()`
at lines 91–104 confirm this — `Trans(step)` has its own arm at line 73.

This is a documentation/code drift: every "8-line wiring delta" arithmetic in
the ADRs was sized against an incorrect baseline of 6+1 variants rather than
7+1. The reader who consults these ADRs to understand the math1/ freeze
carve-out perimeter will see a different enum shape in the source.

**Fix:** Update both ADRs to enumerate 7 v3.0 variants. Suggested edit for the
verbatim enum block in ADR-v3.1-005:
```rust
pub enum ModalProgram {
    Matrix,
    Solve,
    Poly,
    Integ,
    Difeq,
    Four,
    Trans,
}
```
And update the "6 v3.0 variants" / "6 v3.0 Math 1 variants" prose to "7 v3.0
variants" across both ADRs. Note: this does NOT change the architectural
decision — the `Stat1(Stat1Step)` variant still lands as variant #8; only the
arithmetic baseline narrative is wrong.

### WR-02: D-35-04 vs D-35-05 conflicting `33-SPEC-AMENDMENT.md` row citations

**File:** `docs/hp41-stat1-divergences.md:323, 363`

**Issue:** Internal contradiction in the divergence catalog's row attribution:

- D-35-04 (ΣEFXSQ χ² Calibration Drift) **See** field at line 323 cites:
  `33-SPEC-AMENDMENT.md rows 4 / 5 / 6 (Plan 35-01 — three rows because three requirements drifted; this catalog entry bundles them since they share root cause)`
- D-35-05 (ΣBSTAT Coefficient-of-Variation Oracle Correction) **See** field
  at line 363 cites: `33-SPEC-AMENDMENT.md row 5 (Plan 35-01)`

Both entries claim ownership of row 5 of the SPEC amendment file. Either:
(a) D-35-04 covers rows 4 and 6 only (and D-35-05 owns row 5), or
(b) D-35-05 actually maps to a different row (perhaps row 5 was renumbered),
or (c) D-35-04's "bundles 4/5/6" wording is correct and D-35-05's "row 5" is
the stale citation.

Reviewers cannot resolve this from the in-scope files alone (33-SPEC-AMENDMENT.md
is outside the review set, but the contradiction is visible from the catalog
itself). The bundling claim in D-35-04 ("bundles them since they share root cause")
plus the body of D-35-04 listing three requirements (Req. 27 / 28 / 29) suggests
D-35-04 should cite rows 4/5/6 covering ΣEFXSQ + ΣCTKKK + ΣCTKK, in which case
D-35-05's "row 5" is the bug — but a row 5 cannot simultaneously be the
ΣCTKKK χ² fix and the ΣBSTAT CV fix.

**Fix:** Verify against `33-SPEC-AMENDMENT.md` and update the wrong reference.
Most likely correction (assuming D-35-05 is the truncated/stale one): change
D-35-05 line 363 from "row 5" to the correct row number for ΣBSTAT Req. 7
(possibly row 7 — note that line 339 in D-35-05 cites "Req. 7"), e.g.:
```markdown
- **See**: `33-SPEC-AMENDMENT.md` row 7 (Plan 35-01); ...
```

### WR-03: ADR-v3.1-001 "ONLY existing precedent" provenance overstatement

**File:** `docs/adr/v3.1-001-rng-state-placement.md:62-64, 104-105`

**Issue:** Lines 62–64 state:

> "The serde shape is unique to this field within v3.1 (the existing v3.0
> sibling that shares the shape — `complex_mode: bool` — predates the
> convention sweep)."

And lines 104–105 in the inline-Rust-comment block:

> "Mirror pattern: `complex_mode: bool` above is the IDENTICAL
> `#[serde(default)]` (no skip) shape — the ONLY existing precedent
> in this struct."

The "ONLY existing precedent" claim is incorrect when read as a literal
statement about `CalcState`. A grep of `hp41-core/src/state.rs` shows
multiple persistent (non-transient) `#[serde(default)]`-without-`skip` fields:
`xrom_modules` (line 164, `#[serde(default = "default_xrom_modules")]` — also
default-without-skip), and the v1.0+ persistent fields `program` /
`registers` / `flags` / `alpha_reg` etc. that carry `#[serde(default)]` for
backward-compat without `#[serde(skip)]` because they are intentionally
persisted.

The actual claim the ADR wants to make is narrower: "`complex_mode` is the
only OTHER bool/scalar field on `CalcState` that is BOTH new since v2.x AND
persisted across save/load without `#[serde(skip)]`." The current wording
conflates "shares the serde shape" (true of many fields) with "shares the
v3.x-era non-transient-mutable-scalar pattern" (true only of `complex_mode`).

Misleading provenance erodes the doc's value as a future-maintainer reference,
and contradicts a separate claim in the ADR (line 41: "Every other `#[serde(default)]`
field in v3.1's enlarged `CalcState` (`print_buffer`, `modal_program`,
`modal_prompt`, `integ_state`, `solve_state`, `difeq_state`, `cancel_requested`)
carries `#[serde(skip)]`") which is itself accurate but specifies only the
v3.1-additions, not the union of all `CalcState` fields.

**Fix:** Tighten the wording. Suggested edit to line 62–64:
```markdown
The serde shape is unique among **the v3.0/v3.1-era scalar additions** to
`CalcState`: `complex_mode: bool` (v3.0, line 171) is the only other
post-v2.2 scalar field with `#[serde(default)]` without `#[serde(skip)]`.
The v1.0+ persistent collections (`program`, `registers`, `flags`, `alpha_reg`)
also carry `#[serde(default)]` without `#[serde(skip)]` by definition — they
are the save-file payload — but the "scalar mutable state crossing the
session boundary" pattern is new with `complex_mode` and now `rand_seed`.
```
And similarly tighten the inline-Rust-comment block at lines 100–117.

### WR-04: ADR-v3.1-001 contradicts `state.rs:175-200` comment block on field-comment line range

**File:** `docs/adr/v3.1-001-rng-state-placement.md:90, 151`

**Issue:** Line 90 says the field block is at "lines 175–202" and line 151
says "the 25-line documentation comment immediately above the field
declaration (`state.rs:175-200`)." But the actual `rand_seed` comment block
in `hp41-core/src/state.rs` begins at line 173 (not 175) with the section
header `// ── Phase 33 (v3.1): Stat 1 Pac RNG seed ────────────`, with the
doc-comment proper running 174–200 (27 lines, not 25) and the `#[serde(default)]`
annotation at 201, the field declaration at 202.

These are small off-by-two / off-by-two-line miscounts but they directly
contradict the "Mirror pattern: complex_mode above is the IDENTICAL shape"
comment block reproduced inline in the ADR at lines 89–117, which is itself
asserted to be the "Ready-to-paste Rust struct field (consumed by Plan 33-01)".

If the inline Rust block was actually consumed by Plan 33-01, it would have
landed verbatim into `state.rs:175-202`. The current source shows the
comment text is similar but not identical (the source has been edited since
the ADR was written — e.g., the source at lines 186–189 includes the
"silent drift, no error" sentence not present in the ADR's inline block).

**Fix:** Either:
(a) Update the ADR to cite the actual line range (`state.rs:173-202`) and
sync the inline Rust block to the current source, OR
(b) Note explicitly that the inline block is the "as-of-write-time"
snapshot and add a note pointing to the live source for the authoritative
text.

### WR-05: ADR-v3.1-004 references not-yet-existing carve-out line anchors

**File:** `docs/adr/v3.1-004-math1-freeze-second-carve-out.md:241-249`

**Issue:** Footnote `[^3]` claims:
> "`hp41-core/src/ops/math1/xrom.rs` — current `STAT_1` const at line 141;
> bit-1 dispatch arm at line 207 (within `xrom_resolve`); the
> `stat1_resolve` adapter at line 313 delegates into
> `hp41-core/src/ops/stat1/`."

Verified against source: `STAT_1` const IS at line 141, bit-1 arm IS at
line 207, `stat1_resolve` IS at line 313. **These match.**

Footnote `[^4]` claims:
> "`hp41-core/src/ops/math1/modal.rs` — current ~8-line dispatch
> footprint: `Stat1(crate::ops::stat1::modal::Stat1Step)` variant at
> line 51; `current_prompt` arm at line 75; `requires_alpha_label` arm
> at line 101; total under 10 lines per D-33.3b."

Verified: variant at line 51 ✓, `current_prompt` arm at line 75 ✓,
`requires_alpha_label` arm at line 101 ✓. **These match.**

So the line anchors themselves are correct. However, the ADR claims "3-arm
parity with the existing 6 variants" (line 81), "Sibling dispatch arms for
`submit_step()`, `current_prompt()`, and `requires_alpha_label()`" (D-33.3b
verbatim quote at line 178). Verification of `math1/modal.rs` shows only TWO
delegation arms exist for `Stat1` (one in `current_prompt`, one in
`requires_alpha_label`). There is no `submit_step()` impl on `ModalProgram`
itself in the file — submit is dispatched elsewhere.

The "3-arm" claim is part of the verbatim D-33.3b quote so cannot be
edited in this ADR (it would require a CONTEXT.md amendment), but the ADR's
own paraphrase at line 81 ("each delegating to the `stat1::modal` impls (3-arm
parity with the existing 6 variants)") inherits the inaccuracy and should be
softened to "2-arm parity (current_prompt + requires_alpha_label)" or a
footnote should clarify why submit_step is not a sibling-arm site.

**Fix:** Add a clarifying footnote next to either the verbatim D-33.3b quote
or the line 81 paraphrase explaining that `submit_step` is dispatched at a
different site (likely in stat1/modal.rs directly via key-event routing), so
the literal arm count in `math1/modal.rs` is 2, not 3. Without this
clarification a maintainer counting arms in the source will be confused.

### WR-06: `hp41-stat1-function-matrix.md` mis-labels RAND/SEED as "✓ v2.x"

**File:** `docs/hp41-stat1-function-matrix.md:6, 22-23`

**Issue:** The auto-generated matrix's "## Implemented (v2.x)" section
heading and the status column for all 26 entries display "✓ v2.x" — but
this is a Stat 1 Pac matrix (a v3.1 milestone deliverable). Specifically,
RAND (line 22) and SEED (line 23) are explicitly classified as **v3.1
emulator extensions** per ADR-v3.1-001 and D-35-07 in `hp41-stat1-divergences.md:80-123`:

> D-35-07: "RAND / SEED LCG — v3.1 Emulator Extension"

Labelling them "✓ v2.x" inside a section titled "Implemented (v2.x)" is
internally inconsistent with the divergence catalog. The status mapping
arises in `scripts/docs-matrix/src/main.rs:124` where
`"implemented" => "✓ v2.x"` is hard-coded, plus the section header at
`render_markdown` line 89 (`"## Implemented (v2.x)\n\n"`). This is a
documentation-code defect rooted in the renderer (the v3.0 / v3.1 case was
not generalized when the third JSON file was added).

Note: this finding identifies a defect *in the artifact and the renderer* —
not in the JSON itself. The JSON correctly stamps `status: "implemented"`,
but the rendered Markdown surfaces "v2.x" which is misleading for Stat 1
Pac entries.

**Fix (renderer):** Refactor `scripts/docs-matrix/src/main.rs` to derive the
section heading and status label from a per-JSON milestone tag rather than
hard-coding `v2.x`. Suggested approach:

Option A: parameterize via a 4th CLI argument carrying the milestone label,
e.g. `docs-matrix <input.json> <output.md> <milestone-label>`. Drop-in fix.

Option B: hold the milestone tag in the JSON itself as a top-level
`milestone: "v3.1"` field and key the heading/status off it.

Either option breaks the "1-in/1-out" binary signature claim in line 63
(`render_markdown` doc comment "keeps the binary 1-in/1-out per D-30.1"),
so the choice ties back to D-30.1. The fastest non-architectural fix is to
relax the strings from "v2.x"-centric to milestone-agnostic ("✓ Implemented"
and "## Implemented", dropping the version qualifier from auto-generated
content).

## Info

### IN-01: `docs/architecture-history.md` "v3.0 ships" milestone table tag is "pending"

**File:** `docs/architecture-history.md:26`

**Issue:** The milestone status table at line 26 reads:
```
| v3.0 | Math Pac I Emulation | 2026-05-21 | 28–32 + polish batch | pending |
```

A milestone with a "Shipped" date of 2026-05-21 should not show `pending`
under the Tag column. The CLAUDE.md project-instructions block (loaded from
`/Users/daniel/GitRepository/hp41-calculator-emulator/CLAUDE.md`) states:
> "Current: v3.0 Math Pac I (Owner's Manual 00041-90034 feature-complete),
> shipped 2026-05-21."

But CLAUDE.md doesn't actually list `v3.0` as a git tag — only `v2.2`, `v2.0`,
`v1.1`, `v1.0`. So if v3.0 is not tagged, `pending` is accurate. The table
just looks inconsistent at a glance. Same observation for v3.1 row which is
correctly marked "IN PROGRESS".

**Fix:** Cosmetic — either tag v3.0 in git and update the table, or add a
column-footer note clarifying that the Tag column reflects git-tag presence
and "pending" is the expected state for a milestone awaiting its post-ship
tag bump.

### IN-02: ADR-v3.1-002 footnote [^10] disclaim sentence count

**File:** `docs/adr/v3.1-002-distribution-primitives-policy.md:249-253`

**Issue:** Footnote [^10] (line 249) reads:
> "Algorithm independently re-derived from primary sources (Wichura AS 241 /
> Cody AS 239 / Lentz AS 63); Free42 source consulted only as sanity-check
> oracle, not copied."
>
> "This disclaim sentence appears in this Footnotes section, in the
> Decision section above, **and** in the per-file header of
> `hp41-core/src/ops/stat1/distributions.rs`."

The ADR claims three appearances: Footnotes + Decision section + per-file
source header. The reviewer cannot verify the per-file source header without
reading `distributions.rs` (out of scope for this review), but the count is
worth flagging because the `scripts/check-free42-contamination.sh` allowlist
relies on disclaim header text being grep-detectable in `stat1/*.rs` files
(claimed in ADR line 64–66).

The exact sentence appears at line 60 (Decision section) and line 249
(Footnotes). The ADR also redundantly repeats the assertion at line 249's
trailing text "appears in this Footnotes section, in the Decision section
above, and in the per-file header" — a minor structural quirk because the
self-referential count includes its own location.

**Fix:** None required if the per-file header text matches verbatim. As
a quality improvement, consider extracting the disclaim sentence into a
single canonical location (e.g., a snippet constant) and have the ADR
reference it once with the count derived programmatically by the CI grep
script that already scans for it.

### IN-03: ADR-v3.1-003 OM page citations slight format inconsistency

**File:** `docs/adr/v3.1-003-anova-register-layout.md:80-97, 232-236`

**Issue:** Footnote [^2] at line 232–236 enumerates per-program OM pages:
> "page 11 ΣBSTAT, page 15 ΣMMTUG, page 20 ΣAOVONE, page 23 ΣAOVTWO,
> page 28 ΣANOCOV, page 35 ΣLIN/EXP/LOGI/POW, pages 40–44 ΣMLR / ΣPOLYP,
> page 52 ΣPTST / ΣTSTAT, page 73–74 Appendix A summary"

The inline Decision-section code block at line 80 marks ΣAOVONE as
"(OM p. 20, SIZE 020)" — consistent. But the Appendix A table at line 55–69
gives row counts like "Analysis of Variance (One Way) | 29 | 00 ~ 19" while
the Decision-section commentary at line 74 says "(`STAT1_MAX_REG = 44` from
ΣMLR / ΣPOLYP)" referencing R00–R44, which is the correct range for
SIZE 045 (45 registers, indexed 0–44).

The OM page citation discipline (Pitfall 18) is honored everywhere, but
several entries use different punctuation/format ("page 11" vs "p. 11" vs
"pages 40–44") within the same ADR. Suggested style consolidation, not a
correctness issue.

**Fix:** Cosmetic style sweep — standardize to "p. N" or "OM p. N" form
throughout ADR-v3.1-003.

### IN-04: `hp41-stat1-divergences.md` "How to Use" gap between numbering convention and bucket-1 ordering

**File:** `docs/hp41-stat1-divergences.md:38-46, 54-67`

**Issue:** The "How to Use This Document" section (lines 38–46) describes
the numbering convention as `D-35-NN` (Phase 35 / STAT-DOC-03) and notes
"A subset of `D-35-NN` entries (D-35-01..D-35-06) document the 6 oracle
drifts." Bucket 1 (lines 54–67) then explains why bucket 1 is empty.

However, the catalog body actually starts at D-35-07 in bucket 2
(Emulator Extensions). A reader scanning the catalog top-to-bottom sees:
bucket 1 header → "No entries — Phase 35 / Plan 35-02 did not surface..."
→ bucket 2 → D-35-07 → D-35-08 → bucket 3 → D-35-01 → D-35-02 ...

This is not strictly wrong (the IDs are stable identifiers, not positional
ordinals), but a first-time reader sees IDs 07, 08 in bucket 2 and IDs
01–06, 09, 10, 11, 12 in bucket 3. The "How to Use" section should explain
this non-monotonic-but-stable numbering convention explicitly so a reader
isn't surprised by the apparent gap.

**Fix:** Add one sentence after line 46 clarifying: "Entry IDs are assigned
in lock-step with the Plan 35-02 work order, not in section order; readers
should treat the `D-35-NN` IDs as stable cross-reference handles, not as
section-ordinal markers."

### IN-05: `scripts/docs-matrix/src/main.rs` fallback branch uses literal `{json_path}` not interpolation

**File:** `scripts/docs-matrix/src/main.rs:76-77`

**Issue:** The fallback branch at line 76–77:
```rust
} else {
    ("# Function Matrix", "`{json_path}`")
};
```

The src cell `"\`{json_path}\`"` is a **literal string** containing the
characters `{json_path}` — it is NOT a `format!` template. If this branch
ever fires (i.e., a JSON file with an unrecognized basename), the rendered
output will literally read:
```
> Generated from `{json_path}` via `just docs-matrix`.
```

…showing the placeholder text instead of the actual path. This is a
pre-existing defect that predates Phase 35 (lines 70–78 are an `if/else if`
chain; Phase 35 only added the `hp41-stat1-functions.json` arm at lines
74–75). However, since Phase 35 extended this dispatch chain, a defensive
reviewer would note that the fallback branch's bug is now one branch
"closer" to firing if a fourth JSON sibling is added later.

The branch only fires if someone runs the binary with an unrecognized JSON
basename. In normal `just docs-matrix` usage this can't happen — the
recipe hard-codes the three valid JSON paths. But the binary is reachable
directly, and the failure mode is silent (no warning printed, just wrong
output).

**Fix:** Convert the literal to a `format!` call or use a runtime interpolation:
```rust
} else {
    let src = format!("`{json_path}`");
    ("# Function Matrix", src.as_str())  // or restructure to defer the format!
};
```
This requires returning an owned `String` for `src` instead of `&'static str`,
so the signature/types of the tuple change. Alternatively, log a warning and
exit non-zero on unrecognized basename — more aligned with the existing
`std::process::exit(2)` pattern at line 50.

### IN-06: `scripts/docs-matrix/src/main.rs` 3-way `if/else if` not exhaustive

**File:** `scripts/docs-matrix/src/main.rs:70-78`

**Issue:** The Phase 35 +2 line addition extends the basename dispatch to a
3-way `else if` chain. As the count of supported JSON files grows
(v3.2 Time Pac, v3.3 Advantage Pac), this chain will reach 4–5 arms.
Rust idiom for exhaustive string dispatch typically prefers a `match`
expression over a chain of `if/else if`, e.g.:
```rust
let (title, src) = match basename {
    b if b.ends_with("hp41cv-functions.json") => (...),
    b if b.ends_with("hp41-math1-functions.json") => (...),
    b if b.ends_with("hp41-stat1-functions.json") => (...),
    _ => (...),
};
```
…or extract the dispatch into a lookup table keyed by basename suffix.

For 3 arms the current form is readable; flagging this only to acknowledge
the trajectory.

**Fix:** Defer. Refactor when the chain grows past 4 arms.

### IN-07: ADR-v3.1-005 example code block diverges from inline comment policy

**File:** `docs/adr/v3.1-005-modalprogram-stat1-enum-extension.md:69-78`

**Issue:** The example `Stat1Step` enum reproduction at lines 69–78 carries
inline `// ΣNORMD opens here; X = mode index` style comments. The actual
file `hp41-core/src/ops/stat1/modal.rs` (cited as the canonical home) was
not read during this review, so divergence between the ADR's reproduction
and the source cannot be confirmed.

In ADR-v3.1-001 (line 89–117) the inline Rust block diverged slightly from
the live source (see WR-04). The same pattern may apply here. A standard
mitigation: add a "// Snapshot as of 2026-05-22 — see source for live text"
header to each inline-Rust block in the ADRs so future readers know the
ADR is not the source-of-truth.

**Fix:** Add a snapshot disclaimer to each inline-Rust block in ADR-v3.1-001
and ADR-v3.1-005 (and any other ADR that reproduces source code). Suggested:
```markdown
> Snapshot as of ADR lock date (2026-05-22). For live text, see the cited
> source file — the ADR is not the source of truth.
```

---

_Reviewed: 2026-05-23_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
