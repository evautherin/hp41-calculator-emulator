# Requirements: HP-41 Calculator Emulator — v4.2 Help Search Enrichment

**Defined:** 2026-06-04
**Core Value:** Faithful HP-41 RPN fidelity — everything else is secondary. This milestone serves *discoverability*: users find the right function by intent (DE or EN, typos tolerated) without learning the exact mnemonic, while honoring the zero-runtime-ML / zero-new-deps / `hp41-core`-untouched invariants.

**Design spec:** `docs/superpowers/specs/2026-06-03-help-search-enrichment-design.md` (approved, design phase).

## v1 Requirements

Requirements for the v4.2 milestone. Each maps to exactly one roadmap phase.

### Data Model (HSDATA)

- [x] **HSDATA-01**: The CLI `HelpEntry` (`hp41-cli/src/help_data.rs`) gains a `search_aliases: Vec<String>` field annotated `#[serde(default)]`, so existing and older `docs/hp41-*-functions.json` without the field still parse (purely additive).
- [x] **HSDATA-02**: The GUI `HelpEntry` TS type (`hp41-gui/src/help_data.ts`) gains an optional `search_aliases?: string[]` field mirroring the Rust field.
- [x] **HSDATA-03**: All six `docs/hp41-*-functions.json` pools populate `search_aliases` with DE + EN synonyms / paraphrases / natural-language phrasings for each `status: "implemented"` entry.
- [x] **HSDATA-04**: `search_aliases` is an invisible match surface only — it is never rendered in the `?` overlay and produces no layout or visual change.

### Authoring Pipeline (HSGEN)

- [x] **HSGEN-01**: A dev-only script under `scripts/help-aliases/` (alongside `scripts/docs-matrix/`) reads each JSON entry's `display_name` / `description` / `notes` / `category` and writes generated DE + EN aliases back into the entry's `search_aliases`.
- [x] **HSGEN-02**: Alias generation calls a large LLM only on the developer's machine; no model, API call, or inference is present in any shipped binary or runs at runtime (the "AI" is authoring-time only).
- [x] **HSGEN-03**: Generated aliases are committed as static data (PR-reviewable like any content); the script is documented as re-run only when functions are added or their semantics change.

### Runtime Matcher (HSMATCH)

- [x] **HSMATCH-01**: Both matchers score a query over `display_name` + `description` + `category` + `search_aliases` — CLI `filter_help_rows` (`help_data.rs`) and the GUI overlay filter (`HelpOverlay.tsx`).
- [x] **HSMATCH-02**: Matches are ranked in tiers, highest first: exact match > word-prefix match > substring match > fuzzy match.
- [x] **HSMATCH-03**: A hand-rolled fuzzy distance (Levenshtein / trigram, ~30–40 LOC, **no new dependency**) provides typo tolerance, implemented in Rust and mirrored bit-for-bit in TypeScript.
- [x] **HSMATCH-04**: A non-empty query renders a relevance-ranked flat list (best matches first); an empty query preserves the existing category-grouped view unchanged.
- [x] **HSMATCH-05**: Both DE and EN aliases resolve a query to the intended entry (e.g. "Zinseszins" and "compound interest" both → TVM).

### UX (HSUX)

- [x] **HSUX-01**: The existing `?`-overlay live-filter input is reused in both CLI and GUI — no new view, mode, or screen is added.
- [x] **HSUX-02**: Typo'd queries surface the intended function via fuzzy matching (e.g. "Zineszins" → TVM, "Wurzel" → SQRT).

### Quality / CI (HSQUAL)

- [x] **HSQUAL-01**: Unit tests in both frontends cover the scoring tiers (exact > prefix > substring > fuzzy), fuzzy hits, DE + EN alias resolution, and empty-query passthrough (unchanged behavior).
- [x] **HSQUAL-02**: A CLI↔GUI parity fixture asserts identical ranked results for a set of canonical queries, guarding the duplicated matcher against drift (same role as the `op_display_name` exhaustive-match invariant).
- [x] **HSQUAL-03**: A schema-only CI gate validates `search_aliases` across all six JSON pools — the field is present and correctly typed, and every `status: "implemented"` entry has ≥ 1 alias. **No** regenerate-and-diff gate (LLM output is non-deterministic).
- [x] **HSQUAL-04**: `CLAUDE.md`'s JSON-canonical-data-flow section documents the new `search_aliases` field, the German-in-search-input convention exception, and the upgraded matcher.

## v2 Requirements

Deferred to a future release. Tracked but not in the v4.2 roadmap.

### Coverage (HSCOV)

- **HSCOV-01**: Log queries that return zero results (a "missed-query" surface) so alias gaps in unseen vocabulary can be discovered and patched by hand. The spec accepts the unseen-vocabulary gap for v4.2 ("logged, not silently capped"); a structured log is a follow-up.

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Generative / ChatBot-style answers, RAG | Users wanting prose answers should use ChatGPT & co. — search goal is *understand intent*, not generate prose |
| Runtime embedding model, vector index, any runtime ML inference | Breaks the zero-deps invariant and bloats the iOS binary; not worth it for a bounded (~228-entry) domain |
| New runtime dependency (e.g. a fuzzy crate) | Honors "Zero new runtime deps since v3.0"; fuzzy is hand-rolled |
| Any change to `hp41-core` | UI/help concern only; the UI-agnostic core stays untouched |
| Save-file / `CalcState` impact | `search_aliases` is documentation data, not state; no migration |
| Regenerate-and-diff CI gate on aliases | LLM output is non-deterministic; a "regenerate, assert no diff" check would always report drift |
| Guaranteed coverage of truly unseen vocabulary | No runtime embeddings → a query absent from entry text *and* aliases finds nothing; accepted, patched by adding aliases (see HSCOV-01) |

## Traceability

Which phases cover which requirements. Populated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| HSDATA-01 | Phase 58 | Complete |
| HSDATA-02 | Phase 58 | Complete |
| HSDATA-03 | Phase 60 | Complete |
| HSDATA-04 | Phase 58 | Complete |
| HSGEN-01 | Phase 60 | Complete |
| HSGEN-02 | Phase 60 | Complete |
| HSGEN-03 | Phase 60 | Complete |
| HSMATCH-01 | Phase 59 | Complete |
| HSMATCH-02 | Phase 59 | Complete |
| HSMATCH-03 | Phase 59 | Complete |
| HSMATCH-04 | Phase 59 | Complete |
| HSMATCH-05 | Phase 59 | Complete |
| HSUX-01 | Phase 59 | Complete |
| HSUX-02 | Phase 59 | Complete |
| HSQUAL-01 | Phase 61 | Complete |
| HSQUAL-02 | Phase 61 | Complete |
| HSQUAL-03 | Phase 61 | Complete |
| HSQUAL-04 | Phase 61 | Complete |

**Coverage:**
- v1 requirements: 18 total
- Mapped to phases: 18 / Unmapped: 0 ✓

---
*Requirements defined: 2026-06-04*
*Last updated: 2026-06-05 — all 18 v1 requirements satisfied at v4.2 milestone close (verified via phase VERIFICATIONs + Phase 61 gates + cross-phase integration audit). HSCOV-01 remains v2/deferred.*
