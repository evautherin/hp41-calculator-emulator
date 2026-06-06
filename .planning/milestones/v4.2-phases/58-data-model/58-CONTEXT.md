# Phase 58: Data Model - Context

**Gathered:** 2026-06-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Add the additive `search_aliases` match-surface field to **both** help-entry mirrors so the later phases have a field to read (59, matcher) and write (60, alias content). This phase delivers the data-model plumbing only — no matcher behavior, no alias content, no UX change.

**In scope (HSDATA-01, HSDATA-02, HSDATA-04):**
- Rust: `HelpEntry` in `hp41-cli/src/help_data.rs` gains `search_aliases: Vec<String>` with `#[serde(default)]`.
- TS: `HelpEntry` interface in `hp41-gui/src/help_data.ts` gains optional `search_aliases?: string[]`.
- The field is an **invisible match surface** — never rendered in the `?` overlay, no layout/visual change.
- A minimal backward-compat regression test in each frontend (additive guarantee).

**Out of scope (other phases — do NOT pull in):**
- Populating `search_aliases` with DE+EN content (HSDATA-03 → **Phase 60**).
- Any matcher / scoring / fuzzy / ranking change (HSMATCH-* → **Phase 59**).
- The `scripts/help-aliases/` authoring pipeline (HSGEN-* → **Phase 60**).
- Schema CI gate, parity fixture, scoring unit tests, CLAUDE.md doc update (HSQUAL-* → **Phase 61**).
- Any change to `hp41-core/`, `CalcState`, or save-file format (milestone invariant).

</domain>

<decisions>
## Implementation Decisions

### JSON seeding scope
- **D-58.1:** Phase 58 is **type definitions + tests only**. The six `docs/hp41-*-functions.json` pools are **left untouched** — no empty `search_aliases: []` seeding now. Rationale: `#[serde(default)]` (Rust) / optional `?` (TS) already make the absent field parse cleanly, and Phase 60's authoring pipeline writes `search_aliases` when it generates content. Seeding empty arrays now would be ~228+ entries of churn that Phase 60 immediately overwrites, for no parse benefit.

### Verification scope
- **D-58.2:** Ship a **minimal backward-compat regression test in each frontend** proving the field is additive — old JSON without `search_aliases` resolves to an empty alias list. Richer scoring/fuzzy/DE-EN/parity tests stay in Phase 61 (HSQUAL-01/02).
  - **Rust:** a genuine serde round-trip — `serde_json::from_str::<HelpEntry>(<json-without-search_aliases>)` deserializes with `search_aliases == []` (the `#[serde(default)]` guarantee). This is the load-bearing test.
  - **TS asymmetry (flag for planner):** the GUI imports the JSON as a static Vite import baked at build time, so there is no runtime parse to assert against. The TS "backward-compat test" is therefore type-level / consumer-handling, not a serde round-trip. A reasonable assertion: `JSON.parse(<object-without-search_aliases>)` typed as `HelpEntry` yields `search_aliases === undefined` and any (future) consumer treats it as empty. Keep the TS assertion thin — full undefined-handling coverage naturally belongs with the Phase 59 matcher that actually reads the field.

### Rust field shape
- **D-58.3:** Use `search_aliases: Vec<String>` with `#[serde(default)]`, mirroring the existing `divergences` field (NOT `Option<Vec<String>>` like `xrom`). An empty-by-default list is the cleaner match surface: Phase 59's matcher iterates it with no `None`-handling. Aliases are conceptually a populated-or-empty list, not a presence flag.

### TS field shape
- **D-58.4:** Use optional `search_aliases?: string[]` (mirrors the existing optional enrichment fields `divergences?`, `xrom?`, `example?`, `notes?`). Optional is required so the current JSON imports (no field yet) typecheck until Phase 60 populates them.

### Invisible-surface guarantee
- **D-58.5:** No render-path change. Verified during scout that the field is invisible by construction — neither `help_overlay_rows()`/`HelpRow` (Rust) nor `helpOverlayRows()`/`filterHelpEntries()` (TS) read it; they project only `display_name`/`description`/`category`/`key_path`. The executor must NOT add `search_aliases` to `HelpRow` or any overlay row struct in this phase (that decision belongs to Phase 59, see Pitfall P-HS-01).

### Claude's Discretion
- Exact doc-comment wording on the new fields, and where in the struct/interface field-order the field is inserted (after `notes` / alongside the other optional enrichment fields is natural).
- Test-file placement and fixture-string contents for the backward-compat test, following existing `#[cfg(test)]` conventions in `help_data.rs` and the `*.test.ts` conventions in `hp41-gui/src/`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design contract (read first)
- `docs/superpowers/specs/2026-06-03-help-search-enrichment-design.md` — approved milestone design. §"Component 1 — Data model" (lines 78–103) specifies the exact field signatures, the `#[serde(default)]` backward-compat rationale, the not-displayed/invisible-surface rule, and the English-only-doc exception for DE search input.

### Requirements & roadmap
- `.planning/REQUIREMENTS.md` — HSDATA-01 / HSDATA-02 / HSDATA-04 are this phase; HSDATA-03 is explicitly Phase 60 (traceability table).
- `.planning/ROADMAP.md` — Phase 58 goal + dependency order (58 field → 59 matcher → 60 content → 61 gates).
- `.planning/STATE.md` §"Accumulated Context" — pre-resolved milestone decisions + the P-HS-* pitfall table (P-HS-01 is the one that constrains this phase: do NOT widen `HelpRow` here).

### Code to modify (both mirrors — the `op_display_name`-style CLI↔GUI duplication pattern)
- `hp41-cli/src/help_data.rs` — `HelpEntry` struct (~lines 64–82); existing `#[serde(default)]` precedent on `divergences` (`Vec<String>`) and `xrom` (`Option<_>`); `HelpRow` (~line 353) and `help_overlay_rows()` are the render path that must stay alias-free.
- `hp41-gui/src/help_data.ts` — `HelpEntry` interface (lines 59–86); existing optional enrichment fields `divergences?`/`xrom?`/`example?`/`notes?`; `helpOverlayRows()` / `filterHelpEntries()` render path that must stay alias-free.

### Project invariant doc
- `CLAUDE.md` §"JSON canonical data flow" — the six-pool `include_str!` + `OnceLock` (Rust) / Vite static-import (TS) structure this field rides on. (The doc UPDATE itself is Phase 61 / HSQUAL-04, not this phase — listed for the field's structural context only.)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`divergences` field precedent (Rust):** `pub divergences: Vec<String>` with `#[serde(default)]` is the exact shape D-58.3 adopts — copy the annotation/idiom.
- **Optional enrichment fields precedent (TS):** `divergences?`, `xrom?`, `example?`, `notes?` on the TS `HelpEntry` are the exact shape D-58.4 adopts.
- **Existing `#[cfg(test)]` block in `help_data.rs`** already has a `HelpRow` fixture + `filter_help_rows` tests — the new serde-default test slots in alongside.

### Established Patterns
- **CLI↔GUI mirror discipline:** the two `HelpEntry` definitions are kept field-for-field in sync (TS doc-comment literally cites the Rust line range). Add the field to both in lockstep; the parity guard for the matcher comes in Phase 61.
- **Additive serde / optional TS = backward compat:** every prior pool extension (math1/stat1/time/advantage/xmem, plus `example`/`notes`) used `#[serde(default)]` / optional `?` so older JSON keeps parsing. This field is the same move — purely additive, no migration.
- **Hard-build-blocker on malformed JSON** (`.expect("…malformed")` Rust / Vite build fail TS) is unaffected — adding an optional field does not change malformed-detection.

### Integration Points
- The field is **defined** here and **consumed** later: Phase 59 reads it in the upgraded matcher (and decides whether to widen `HelpRow` or score over `HelpEntry` directly — P-HS-01), Phase 60 writes it via the authoring script. Phase 58 leaves both the matcher and the render path untouched.
- Expect a transient "field never read in `src/`" situation in Phase 58 — the Rust `HelpEntry` already carries `#[allow(dead_code)]` for exactly this (fields deserialized for tooling/tests but not read in `src/`), so no new lint suppression is needed.

</code_context>

<specifics>
## Specific Ideas

- Field name is fixed: `search_aliases` (from the design spec — do not rename).
- Spec's worked example for later content (Phase 60, not 58): TVM → `["Zinseszins", "Annuität", "Tilgung", "Kredit", "compound interest", "time value of money", "loan payment", "mortgage"]`. Captured here only to anchor the field's intended shape (mixed DE+EN strings).

</specifics>

<deferred>
## Deferred Ideas

- **Empty-`[]` JSON seeding** — considered (D-58.1) and rejected for Phase 58; the field first appears in JSON when Phase 60 populates it.
- **Matcher reads aliases / `HelpRow` widening** — Phase 59 (HSMATCH + P-HS-01).
- **DE+EN alias content + `scripts/help-aliases/` pipeline** — Phase 60 (HSDATA-03, HSGEN-*).
- **Schema CI gate, parity fixture, scoring/fuzzy tests, `CLAUDE.md` doc update** — Phase 61 (HSQUAL-*).

### Reviewed Todos (not folded)
- `privacy-manifest-bundle-wiring.md` (wire `PrivacyInfo.xcprivacy` into the iOS bundle) — surfaced by todo-match on weak keyword overlap ("type", "shipped"), score 0.4. **Not folded** — it is a v4.1 iOS App-Store-prep task with no relationship to the help-search data model. Stays in the backlog for a future iOS milestone.

</deferred>

---

*Phase: 58-data-model*
*Context gathered: 2026-06-04*
