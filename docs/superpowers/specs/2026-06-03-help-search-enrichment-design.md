# Help Search Enrichment — Design

**Date:** 2026-06-03
**Status:** Approved (design phase)
**Topic:** Make the `?` help overlay feel like "free search" (intent-aware), without shipping any ML at runtime.

## Problem

Users increasingly expect ChatBot-style "free search": they type *what they want*
(e.g. *"Zinseszins"*, *"how to clear everything"*) rather than the exact function
mnemonic. Today the `?` overlay only does case-insensitive **substring** matching
over `display_name` / `description` / `category`, so intent-driven queries that
don't contain the literal keyword find nothing.

## Decision summary (from brainstorming)

| Question | Decision |
|----------|----------|
| ChatBot-style generated answers? | **No.** Out of scope — users wanting that should use ChatGPT & co. |
| Search goal | (A) **understand intent**, not (B) generate prose. |
| Generalize to truly unseen vocabulary (runtime embeddings)? | **No.** Not worth breaking the zero-deps invariant or bloating the iOS binary. |
| Approach | **AI at authoring time, lexical at runtime** — an offline LLM enriches each entry with synonyms/paraphrases; runtime stays pure lexical. |
| Languages | **DE + EN** aliases. |
| UX surface | **(A) Live-filter inside the existing `?` overlay** (already the current shell). |
| Typo tolerance | **(B) Yes** — hand-rolled fuzzy distance, no dependency. |

## Non-goals

- No generative LLM, no RAG, no chatbot.
- No embedding model, no vector index, no runtime ML inference.
- No new runtime dependency (honours the *Zero new runtime deps since v3.0* invariant).
- No changes to `hp41-core` (this is a UI/help concern; core stays untouched).
- No save-file impact (this is documentation data, not `CalcState`).

## Existing infrastructure (extend, don't build)

The search shell already exists, mirrored across both frontends (the established
`op_display_name`-style CLI ↔ GUI duplication pattern):

- **CLI:** `hp41-cli/src/help_data.rs`
  - `HelpEntry` struct deserialized from the six `docs/hp41-*-functions.json` pools.
  - `help_entries_all()` chains all six pools.
  - `filter_help_rows(rows, query)` — case-insensitive **substring** match over
    `key` / `op` / `desc`, preserving category headers.
- **GUI:** `hp41-gui/src/HelpOverlay.tsx` + `hp41-gui/src/help_data.ts`
  - `helpEntriesAll()` 6-pool chain (TS mirror of the JSON).
  - Client-side TS filter over `display_name` / `description` / `category`
    (substring `.includes`).

**The live-filter UX (decision A) is therefore already the current shell.** Both
overlays already have a search input that filters as you type. This project does
**not** add a new view or mode — it upgrades the data and the matcher behind the
existing input.

## Architecture

Three changes, no new crate, no `hp41-core` touch, no new dependency:

1. **Data model** — one new field, `search_aliases`, on the help entry (both mirrors).
2. **Authoring pipeline** — an offline, dev-only LLM script that populates
   `search_aliases` with DE+EN synonyms/paraphrases; output is committed static data.
3. **Runtime matcher** — upgrade both mirrored matchers from plain substring to
   alias-aware + fuzzy + relevance-ranked.

```
docs/hp41-*-functions.json  (+ search_aliases: [DE+EN])
        │  include_str! / TS import
        ▼
HelpEntry (Rust)  ───────────  HelpEntry (TS, help_data.ts)
        │                               │
 filter_help_rows()            HelpOverlay.tsx filter
   (CLI matcher)                 (GUI matcher)
        └──────── mirrored fuzzy + scoring ────────┘
                          │
                  ? overlay live filter
```

### Component 1 — Data model

Add a search-only alias field to the help entry, in **both** mirrors:

```rust
// hp41-cli/src/help_data.rs — HelpEntry
#[serde(default)]
pub search_aliases: Vec<String>,
```
```ts
// hp41-gui/src/help_data.ts — HelpEntry type
search_aliases?: string[];
```

- Populated in **all six** `docs/hp41-*-functions.json` pools.
- DE + EN synonyms, paraphrases, and typical natural-language phrasings per entry.
  Example for TVM: `["Zinseszins", "Annuität", "Tilgung", "Kredit", "compound
  interest", "time value of money", "loan payment", "mortgage"]`.
- **Backward compat:** `#[serde(default)]` means existing/older JSON without the
  field still parses; the change is purely additive.
- **Not displayed** in the overlay — it is an invisible match surface only. No
  layout change.
- **English-only doc convention exception:** the project writes all docs in
  English, but `search_aliases` is *search input*, not documentation, so DE
  aliases are intentional and in-scope. This exception is documented here and in
  the JSON-flow section of `CLAUDE.md`.

### Component 2 — Authoring pipeline (the "AI" part — offline, dev-only)

- New script `scripts/help-aliases/` (alongside the existing `scripts/docs-matrix/`).
- For each JSON entry it reads `display_name`, `description`, `notes`, `category`,
  calls a **large LLM once on the developer's machine**, requests N DE+EN aliases,
  and writes them back into `search_aliases`.
- **The LLM is an authoring tool, never shipped.** No model, no API call, no
  inference at runtime or in the user's binary.
- Output is **committed static data**, reviewed in the PR like any other content.
- **No regenerate-and-diff CI gate.** LLM output is non-deterministic, so a
  "regenerate and assert no diff" check (the `docs-matrix-check` pattern) would
  always report drift. Instead the CI gate is **schema-only**:
  - the `search_aliases` field parses and is typed correctly;
  - optionally: every `status: "implemented"` entry has ≥ 1 alias.
- Re-run only when functions are added or their semantics change.

### Component 3 — Runtime matcher (mirrored CLI Rust ↔ GUI TS)

Upgrade both matchers (kept in sync, like the existing duplicated display logic):

- **Match surface:** `display_name` + `description` + `category` + `search_aliases`.
- **Scoring tiers (highest first):**
  1. exact match
  2. word-prefix match
  3. substring match
  4. **fuzzy match** (typo tolerance — decision B)
- **Fuzzy:** hand-rolled Levenshtein / trigram distance (~30–40 LOC), no
  dependency. Fits the project's "algorithms hand-coded from primary literature,
  zero deps" stance. Implemented in Rust and mirrored in TypeScript.
- **Ranking:** when the query is non-empty, the overlay switches from the
  category-grouped view to a **relevance-ranked flat list** (best matches on top).
  An empty query keeps the existing category-grouped view unchanged.
- `HelpRow` currently drops the alias data; either extend it with a
  non-displayed alias field or score directly over `HelpEntry`.

### Component 4 — UX

- No new view or mode. The existing `?`-overlay live-filter input is reused in
  both CLI and GUI.
- Empty query → unchanged category view. Active query → relevance-ranked results,
  now matching intent (via aliases) and tolerating typos (via fuzzy).

## Testing

- **Unit tests (both frontends):**
  - scoring tiers (exact > prefix > substring > fuzzy);
  - fuzzy hits: *"Zineszins"* (typo) → TVM, *"Wurzel"* → SQRT;
  - DE and EN aliases both resolve;
  - empty-query passthrough unchanged.
- **Parity test:** a fixture of canonical queries asserted to produce identical
  ranked results in CLI and GUI — guards the duplicated matcher against drift
  (same role as the `op_display_name` exhaustive-match invariant).
- **Schema gate:** CI validates the `search_aliases` field across all six JSON
  pools.

## Invariants honoured

- *Zero new runtime deps since v3.0* — ✓ (fuzzy is hand-rolled).
- `hp41-core` UI-agnostic, zero deps — ✓ (untouched).
- *No async, no panics* — ✓ (pure lexical, no model loading).
- *Save-file backward compat* — ✓ (`#[serde(default)]`, documentation data).
- iOS-safe (v4.1 in progress) — ✓ (text-only, negligible binary growth).
- *JSON canonical data flow* — extended with one field; `docs-matrix` pipeline and
  six-pool structure preserved.

## Risks / open points

- **Coverage of truly unseen vocabulary:** with no runtime embeddings, a query
  using vocabulary absent from both the entry text and its aliases finds nothing.
  Accepted — the domain is bounded (~228 module entries plus built-ins); gaps are
  patched by adding aliases. Logged, not silently capped.
- **Alias quality:** generated aliases need a human review pass in the PR to avoid
  misleading matches (e.g. an alias that pulls the wrong function to the top).
- **JSON size growth:** DE+EN aliases roughly multiply per-entry text; total help
  JSON is ~90 KB today, so growth is small and text-only.
