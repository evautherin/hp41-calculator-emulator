# Phase 59: Runtime Matcher — Research

**Researched:** 2026-06-04
**Domain:** Text matching / relevance scoring — Rust (hp41-cli) and TypeScript (hp41-gui) mirrored pair
**Confidence:** HIGH

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| HSMATCH-01 | Both matchers score a query over `display_name` + `description` + `category` + `search_aliases` | §Standard Stack / §Architecture Patterns — score over HelpEntry directly (D-58.5) |
| HSMATCH-02 | Matches ranked: exact > word-prefix > substring > fuzzy | §Standard Stack / §Architecture Patterns — tiered score table |
| HSMATCH-03 | Hand-rolled fuzzy distance (~30–40 LOC, no new dep) in Rust, mirrored in TS | §Architecture Patterns §Fuzzy Algorithm Choice |
| HSMATCH-04 | Non-empty query → flat ranked list; empty query → existing category-grouped view unchanged | §Empty-Query Preservation — verified exact branch points in both CLIs |
| HSMATCH-05 | DE and EN aliases both resolve (e.g. "Zinseszins", "compound interest" → TVM) | §Architecture Patterns — alias field is Vec<String> on HelpEntry, scored same as other fields |
| HSUX-01 | Existing `?`-overlay live-filter input reused in both CLI and GUI — no new view | §Architecture Patterns — confirmed: no new component, only matcher upgraded |
| HSUX-02 | Typo'd queries surface intended function ("Zineszins" → TVM, "Wurzel" → SQRT) | §Fuzzy Algorithm Choice — bounded Levenshtein with length-relative threshold |
</phase_requirements>

---

## Summary

Phase 58 added `search_aliases: Vec<String>` (Rust) and `search_aliases?: string[]` (TS) to both `HelpEntry` mirrors, with backward-compat serde/optional semantics. Phase 59 upgrades the two existing matchers — `filter_help_rows` in `hp41-cli/src/help_data.rs` and the inline filter `useMemo` blocks in `hp41-gui/src/HelpOverlay.tsx` — to be alias-aware, relevance-ranked, and fuzzy-tolerant.

The key insight from auditing the code is that both matchers currently operate on different data shapes: the CLI matcher works on `HelpRow` (a flat projection with `key`/`op`/`desc` strings, no aliases), while the GUI matcher already operates directly on `HelpEntry` objects. Phase 58's decision D-58.5 established that the Phase 59 matcher must score over `HelpEntry` directly, bypassing `HelpRow` entirely for the scoring logic. This means the CLI upgrade path requires a new scoring function that works over `help_entries_all()` rather than the pre-projected `help_overlay_rows()`, while keeping the empty-query path (which returns the pre-projected `HelpRow` slice) bit-for-bit unchanged.

The fuzzy algorithm must be hand-rolled at ~30–40 LOC with zero new dependencies, implemented identically in Rust and TypeScript. Bounded Levenshtein with a length-relative threshold is the recommended choice: it is trivially portable (pure integer arithmetic, no data structures), naturally handles the German domain (umlauts are just code points), and fits the LOC budget. The mirrored pair discipline follows the same pattern as `op_display_name` duplication — same algorithm, same scoring constants, same tier boundaries, guarded by a parity test fixture in Phase 61.

**Primary recommendation:** Add a new `score_entry(entry: &HelpEntry, query: &str) -> Option<u8>` function in `hp41-cli/src/help_data.rs` that returns a tier score (4=exact, 3=prefix, 2=substring, 1=fuzzy, None=no match), plus a new `ranked_help_entries(query: &str) -> Vec<RankedEntry>` function for the non-empty case. Mirror in `hp41-gui/src/help_data.ts` as `scoreEntry(entry, query): number | null`. Modify `filter_help_rows` (CLI) and the two filter `useMemo` blocks (GUI) to branch on empty/non-empty and delegate to the appropriate path.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Fuzzy distance computation | Frontend Server (CLI) / Browser (GUI) | — | Pure text function, lives in help_data.rs (Rust) and help_data.ts (TS); no server/API tier |
| Relevance scoring per entry | Frontend Server (CLI) / Browser (GUI) | — | Score is computed at filter time; inputs are the static HelpEntry pool and the user's query |
| Empty-query view (category-grouped) | Frontend Server (CLI) / Browser (GUI) | — | Already handled by current filter_help_rows and sectionGroups memo; Phase 59 must NOT touch this path |
| Ranked flat list output | Frontend Server (CLI) / Browser (GUI) | — | New branch in both matchers, triggered by non-empty query; rendering code in ui.rs and HelpOverlay.tsx |
| Data storage (search_aliases) | CDN / Static (JSON pools) | — | Already done in Phase 58; consumed read-only by the matcher |

---

## Existing Code Audit (VERIFIED)

### CLI: `hp41-cli/src/help_data.rs`

**`filter_help_rows` current behavior** [VERIFIED: direct code read]:

```rust
pub fn filter_help_rows<'a>(rows: &'a [HelpRow], query: &str) -> Vec<&'a HelpRow>
```

- Input: `&[HelpRow]` — a pre-projected flat slice that contains both content rows (`key`/`op`/`desc`) and category-header rows (`desc.starts_with("===")`, empty `key`/`op`).
- Empty query: returns `rows.iter().collect()` — every row unfiltered, including headers. This is the empty-query category-grouped view.
- Non-empty query: lowercase the query; match rows by `row.key.to_lowercase().contains(&q) || row.op.to_lowercase().contains(&q) || row.desc.to_lowercase().contains(&q)`. Headers are preserved only when at least one child row in the same category matches (pending-header buffering pattern).
- **Critical gap:** `HelpRow` carries only `key`, `op`, `desc` — no `search_aliases` field. Scoring aliases requires working from `HelpEntry` directly.
- Case handling: `.to_lowercase()` only — ASCII fold. German umlauts (ä, ö, ü) are lowercased correctly by Rust's `char::to_lowercase` since they are multi-byte UTF-8; `.to_lowercase()` on a `String` returns a new `String` with Unicode-aware case folding.
- No ranking: current output preserves the JSON declaration order, with category groups.

**`HelpRow` struct:**
```rust
pub struct HelpRow { pub key: String, pub op: String, pub desc: String }
```
No alias field. D-58.5 confirms the Phase 59 design bypasses `HelpRow` for scoring.

**`help_overlay_rows()` current behavior:**
- Chains all six pools via `help_entries_all()`, filters `status == "implemented"`.
- Groups by category (in first-appearance order from JSON), inserts header rows.
- For keyless built-ins (no `key_path`, no XROM), constructs `XEQ "NAME"` key display.
- Returns `Vec<HelpRow>` — owned, category-header-interleaved.

**Render path in `ui.rs`:**
- Calls `help_overlay_rows()` every render cycle (cheap — `OnceLock` inside).
- Calls `filter_help_rows(&overlay_rows, &app.help_search_query)`.
- Renders returned `Vec<&HelpRow>`: headers get bold style, content rows get three cells.
- Already branches on `app.help_search_query.is_empty()` for the title bar.
- Phase 59: also needs to branch for the body (flat ranked vs category-grouped).

**`help_entries_all()`:**
```rust
pub fn help_entries_all() -> impl Iterator<Item = &'static HelpEntry>
```
Returns all six pools chained in order. The Phase 59 scorer must call this directly (not via `help_overlay_rows`).

### GUI: `hp41-gui/src/HelpOverlay.tsx` + `help_data.ts`

**Current filter behavior** [VERIFIED: direct code read]:

`HelpOverlay.tsx` has TWO independent filter `useMemo` blocks:

1. **Keyboard Shortcuts tab** (`filtered`):
```ts
const filtered = useMemo(() => {
    const q = query.toLowerCase().trim();
    if (q === '') return allEntries;  // allEntries = helpEntriesAll().filter(e => e.key_path !== null)
    return allEntries.filter(e =>
        e.display_name.toLowerCase().includes(q) ||
        e.description.toLowerCase().includes(q) ||
        e.category.toLowerCase().includes(q)
    );
}, [query, allEntries]);
```

2. **All Functions tab** (`filteredAllFn`):
```ts
const filteredAllFn = useMemo(() => {
    const q = query.toLowerCase().trim();
    if (q === '') return allFnEntries;  // allFnEntries = allFunctionsEntries()
    return allFnEntries.filter(e =>
        e.display_name.toLowerCase().includes(q) ||
        e.description.toLowerCase().includes(q) ||
        e.category.toLowerCase().includes(q)
    );
}, [query, allFnEntries]);
```

**Empty-query rendering path (the bit-for-bit invariant):**

Both `filtered` and `filteredAllFn` feed into `sectionGroups` / `allFnSectionGroups` `useMemo` blocks respectively. These group by section (predicate on `xrom` / `category`) and then by `category` within each section, sorting alphabetically. When `q === ''`, the filter returns the full entry arrays — the existing category-grouped rendering path runs unchanged.

**Empty-query branch point (GUI):** `if (q === '') return allEntries` / `if (q === '') return allFnEntries`. Phase 59 must keep these guard clauses intact. The scoring and ranking only applies when `q !== ''`.

**Phase 59 GUI change:** Replace the non-empty filter branch with a scored+ranked result, returned as a flat `HelpEntry[]` in relevance order. The render path must then bypass `sectionGroups` and render a flat list when `query !== ''`.

**`help_data.ts` — `filterHelpEntries` function** (NOT used by HelpOverlay.tsx for the overlay rendering, but used by some tests):
```ts
export function filterHelpEntries(query: string): readonly HelpEntry[]
```
This function also needs to be upgraded to match the new scoring semantics for test consistency.

---

## Standard Stack

### Core (zero new dependencies — VERIFIED constraint from CLAUDE.md)

| Component | Version | Purpose | Why Standard |
|-----------|---------|---------|--------------|
| `hp41-cli/src/help_data.rs` (extended) | current | Rust scorer + ranked accessor | Where HelpEntry lives; existing tests there; no new crate |
| `hp41-gui/src/help_data.ts` (extended) | current | TS mirror scorer | Where HelpEntry TS type lives; existing Vitest tests there |
| `HelpOverlay.tsx` (modified) | current | GUI render path, flat-ranked branch | Existing UI component; no new view |
| `hp41-cli/src/ui.rs` (modified, minimal) | current | CLI render path, flat-ranked branch | Thin: only the body dispatch needs a branch |

**No new dependencies.** The fuzzy distance is ~30–40 lines of pure integer arithmetic in each language.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-rolled Levenshtein | `strsim` crate (Rust) / `fastest-levenshtein` npm | Banned by "Zero new runtime deps since v3.0" invariant |
| Bounded Levenshtein | Trigram (Sørensen-Dice) | Levenshtein is simpler to mirror bit-for-bit; trigram requires a set-intersection step; both fit the LOC budget but Levenshtein is the stated preference in the spec |
| Score directly in HelpOverlay memo | Separate scorer function in help_data.ts | Separate function is unit-testable in isolation; overlay just calls it |

---

## Package Legitimacy Audit

> No new packages are installed in this phase. This section is intentionally omitted — the zero-new-deps invariant is a hard project constraint.

---

## Architecture Patterns

### System Architecture Diagram

```
User types query
        │
        ▼
query === '' ?
  ├── YES → existing path (unchanged)
  │         CLI: filter_help_rows(help_overlay_rows(), "")  → Vec<&HelpRow> (header-interleaved)
  │         GUI: filtered/filteredAllFn = allEntries / allFnEntries  → sectionGroups grouping
  │
  └── NO  → new ranking path
            CLI: ranked_help_entries(query) → Vec<HelpRow> (flat, sorted by score desc, no headers)
            GUI: rankedEntries(entries, query) → HelpEntry[] (sorted by score desc)
                    │
                    ▼
            score_entry(entry, query) → Option<u8>   [Rust]
            scoreEntry(entry, query)  → number|null  [TS]
                    │
                    ├── check display_name  (tier 4=exact, 3=word-prefix, 2=substring)
                    ├── check description   (same tiers, demoted by 1 vs name match)
                    ├── check category      (same tiers, demoted by 2 vs name match)
                    └── check search_aliases[] (best alias tier)
                                │
                                ▼ tier < 2 on all fields?
                            levenshtein_distance(query, field_token)
                            threshold: distance <= max(1, query.len()/4)
                            if within threshold → tier 1 (fuzzy)
                    │
                    ▼
            entries sorted by (score desc), then display_name asc (tie-break)
```

### Recommended Project Structure

```
hp41-cli/src/
└── help_data.rs          # extended: add score_entry(), ranked_help_entries()
                          # filter_help_rows() updated to call ranked path when non-empty
hp41-cli/src/
└── ui.rs                 # minimal: add flat-list render branch when query non-empty
hp41-cli/tests/
└── phase59_help_search.rs  # new: Rust unit/integration tests for scorer

hp41-gui/src/
├── help_data.ts          # extended: add scoreEntry(), rankedEntries()
├── help_data.test.ts     # extended: Phase 59 scorer tests
└── HelpOverlay.tsx       # updated: flat-ranked render when query !== ''
```

### Pattern 1: Tiered Scoring Function

The score is a `u8` (Rust) / `number` (TS). Higher is better. The final entry score is the **maximum field-tier score** across all four searched fields, with a **field-weight multiplier** that demotes description/category/alias hits relative to display_name hits.

**Recommended field weights and tier encoding:**

| Tier | Display Name score | Description score | Category score | Alias score |
|------|-------------------|--------------------|----------------|-------------|
| exact (`query == field_token`) | 40 | 30 | 20 | 35 |
| word-prefix (field starts with query, or a space-delimited word in field starts with query) | 32 | 24 | 16 | 28 |
| substring (field contains query) | 24 | 18 | 12 | 21 |
| fuzzy (Levenshtein within threshold) | 8 | 6 | 4 | 7 |
| no match | 0 | 0 | 0 | 0 |

**Entry score = max(all field scores for that entry).**

Tie-breaking: stable sort by `(score DESC, display_name ASC)`. This produces identical ordering in CLI and GUI when applied to the same entry pool in the same iteration order — which is guaranteed because both pull from `help_entries_all()` / `helpEntriesAll()` using the same fixed six-pool chain order.

**Threshold for including a fuzzy match:**
An entry is included in results only if its best field score is > 0. A fuzzy match is only counted when `levenshtein(query, token) <= max(1, query.len() / 4)`. For `query.len() < 4`, this means distance <= 1 (one typo). For longer queries (e.g. "Zinseszins" = 10 chars), distance <= 2. This prevents "x" from fuzzy-matching everything.

**Important:** fuzzy is applied to the **entire field string** for short fields (display_name, category), but for longer description/alias strings, apply against each **space-split word** and take the minimum distance. This keeps fuzzy meaningful for descriptions without false-positives.

**Example walkthrough:**
- query = "Zineszins" (typo of "Zinseszins")
- TVM entry has alias "Zinseszins": levenshtein("zineszins", "zinseszins") = 1, threshold = max(1, 9/4) = 2 → within threshold → alias fuzzy score = 7
- Entry score = 7 → included
- Other TVM aliases ("compound interest") won't fuzzy-match "zineszins" at distance <= 2 → score from those = 0
- "Wurzel" query against SQRT entry: levenshtein("wurzel", "wurzel") = 0 (exact match on alias when alias "Wurzel" is present) → alias exact score = 35

### Pattern 2: Bounded Levenshtein (~30 LOC)

The algorithm: classic DP matrix, bounded by the threshold to short-circuit early. The implementation below is the exact algorithm to mirror bit-for-bit in both languages.

**Rust (hp41-cli/src/help_data.rs):**
```rust
// Source: [ASSUMED] — standard algorithm from algorithms literature
/// Returns the edit distance between a and b, capped at `max_dist + 1`.
/// If the true distance exceeds `max_dist`, returns `max_dist + 1`.
fn levenshtein_bounded(a: &str, b: &str, max_dist: usize) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let n = a.len();
    let m = b.len();
    if n.abs_diff(m) > max_dist { return max_dist + 1; }
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr = vec![0usize; m + 1];
    for i in 1..=n {
        curr[0] = i;
        let row_min = curr[0];
        for j in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (curr[j - 1] + 1)
                .min(prev[j] + 1)
                .min(prev[j - 1] + cost);
        }
        // Early-exit: if all values in curr exceed max_dist, no point continuing.
        if curr.iter().min().copied().unwrap_or(0) > max_dist { return max_dist + 1; }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}
```

**TypeScript mirror (hp41-gui/src/help_data.ts):**
```ts
// Source: [ASSUMED] — mirror of the Rust implementation above
function levenshteinBounded(a: string, b: string, maxDist: number): number {
    const n = [...a].length;  // Unicode-aware split
    const m = [...b].length;
    if (Math.abs(n - m) > maxDist) return maxDist + 1;
    const aChars = [...a];
    const bChars = [...b];
    let prev: number[] = Array.from({ length: m + 1 }, (_, i) => i);
    let curr = new Array<number>(m + 1).fill(0);
    for (let i = 1; i <= n; i++) {
        curr[0] = i;
        for (let j = 1; j <= m; j++) {
            const cost = aChars[i - 1] === bChars[j - 1] ? 0 : 1;
            curr[j] = Math.min(curr[j - 1] + 1, prev[j] + 1, prev[j - 1] + cost);
        }
        if (Math.min(...curr) > maxDist) return maxDist + 1;
        [prev, curr] = [curr, prev];
    }
    return prev[m];
}
```

**Unicode / German umlaut handling:**
- Rust: `a.chars().collect::<Vec<char>>()` correctly treats ä, ö, ü as single `char` values (Unicode scalar values). The `==` comparison is exact code-point equality, so "ä" != "a". This is correct: we do NOT fold umlauts (e.g. "ä" → "a"), because German aliases will be spelled with actual umlauts ("Annuität"), and users typing on a German keyboard will also type umlauts. The fuzzy distance naturally handles slight spelling variations.
- TypeScript: `[...str]` (spread of string into array) correctly splits by Unicode code point, handling surrogate pairs and composed characters. Same exact-equality comparison.
- **No umlaut folding needed.** The DE aliases in the JSON will be spelled with proper umlauts; a German-keyboard user typing "Annuität" will match exactly. A user who can't type umlauts (e.g. on an English keyboard) would type "Annuitat" — levenshtein("annuitat", "annuitat") = 0 for an alias "Annuitat", or distance 1–2 for "Annuität" (each umlaut is one character, not two). Whether this cross-folding is needed is an alias-authoring concern (Phase 60), not a matcher concern. The matcher does not need to fold.
- **Case folding:** both Rust (`.to_lowercase()` on `String`) and TypeScript (`.toLowerCase()`) correctly lowercase German umlauts: "Ä" → "ä", "Ö" → "ö", "Ü" → "ü". The scoring function should lowercase both query and field before all comparisons.

### Pattern 3: CLI Rendering — New Flat-List Branch

`ui.rs` already branches on `app.help_search_query.is_empty()` for the title. Phase 59 adds a body branch:

```rust
// Pseudocode for ui.rs render_help_overlay update
let overlay_rows: Vec<HelpRow>;
if app.help_search_query.is_empty() {
    // UNCHANGED PATH: category-grouped, header-interleaved
    let all = help_data::help_overlay_rows();
    overlay_rows = help_data::filter_help_rows(&all, "")
        .into_iter()
        .cloned()
        .collect();
} else {
    // NEW PATH: flat ranked list from Phase 59
    overlay_rows = help_data::ranked_help_entries(&app.help_search_query);
}
// render overlay_rows — same render loop works for both (headers absent in ranked path,
// so the "starts_with('===')" branch is never hit for ranked results).
```

`ranked_help_entries(query: &str) -> Vec<HelpRow>` returns owned `HelpRow` values (no headers, no leading `===`). The existing render loop in `ui.rs` handles this naturally — it already skips the header-style rendering when `!row.desc.starts_with("===")`.

### Pattern 4: GUI Rendering — Flat-List Branch in HelpOverlay

The current `sectionGroups` and `allFnSectionGroups` memo only runs the grouping logic. Phase 59 adds a ranked path:

```ts
// In HelpOverlay.tsx, replace the current two filter memos with scorer-aware versions:

const filteredOrRanked = useMemo(() => {
    const q = query.toLowerCase().trim();
    if (q === '') return allEntries;  // UNCHANGED EMPTY PATH
    // NEW: score and rank
    return rankedEntries(allEntries, q);  // from help_data.ts
}, [query, allEntries]);

// rankedEntries returns HelpEntry[] sorted by score desc, ties broken by display_name asc.
// Then in the render JSX:
{query !== '' ? (
    <FlatRankedList entries={filteredOrRanked} ... />
) : (
    <SectionGroupedList sectionGroups={sectionGroups} ... />
)}
```

Or, more minimally, keep the `sectionGroups` memo but add a flat render path that bypasses section/category grouping. The exact React component structure is a planner decision; the key invariant is: `if (query === '') → existing grouped render, unchanged`.

### Anti-Patterns to Avoid

- **Modifying `HelpRow` to add aliases:** D-58.5 explicitly forbids this. Score over `HelpEntry`.
- **Placing scorer logic in `hp41-core`:** `hp41-core` must not gain UI deps. The scorer is a help-UI concern; it lives in `hp41-cli` and `hp41-gui` only.
- **Applying fuzzy to every query:** A minimum query length guard (e.g. `query.len() >= 2` before fuzzy) prevents false positives on single-char queries. P-HS-02 from STATE.md.
- **Float scores:** Use `u8` / `number` integer scores for deterministic tie-breaking across Rust and TS.
- **Stable sort assumption:** Rust's `sort_by` is stable (preserves original order for equal elements); TypeScript's `Array.sort` is stable in all modern engines (ECMAScript 2019+). Both are safe for the tie-break pattern.
- **Mutable captured reference in `filter_help_rows` return:** The current function returns `Vec<&'a HelpRow>` (references into the passed slice). The new `ranked_help_entries` should return `Vec<HelpRow>` (owned), avoiding lifetime complexity for the new code path.

---

## Empty-Query Preservation (HSMATCH-04)

This is verified from the code [VERIFIED: direct code read]:

### CLI empty-query branch point:

In `filter_help_rows`:
```rust
if query.is_empty() {
    return rows.iter().collect();  // returns ALL rows including headers
}
```
In `ui.rs`:
```rust
let overlay_rows = help_data::help_overlay_rows();
let filtered = help_data::filter_help_rows(&overlay_rows, &app.help_search_query);
```
When `app.help_search_query` is empty, `filter_help_rows` short-circuits and returns all rows. The render loop renders them identically to today. **Phase 59 must not touch this path at all.**

The safe approach: add a new function `ranked_help_entries(query: &str) -> Vec<HelpRow>` that is ONLY called when `query` is non-empty. `filter_help_rows` itself is preserved unchanged (the planner may choose to call it for the empty case or call `help_overlay_rows()` + the new function based on empty/non-empty). The render in `ui.rs` branches explicitly:

```rust
if app.help_search_query.is_empty() {
    // Existing code path — UNCHANGED
    let overlay_rows = help_data::help_overlay_rows();
    let filtered_refs = help_data::filter_help_rows(&overlay_rows, "");
    // ... render filtered_refs
} else {
    let ranked = help_data::ranked_help_entries(&app.help_search_query);
    // ... render ranked (no headers)
}
```

### GUI empty-query branch point:

In `HelpOverlay.tsx`, both filter memos have:
```ts
if (q === '') return allEntries;  // or allFnEntries
```
These guard clauses are the empty-query branch points. Phase 59 must keep them intact. The non-empty branch below them is what gets replaced.

The render currently pipes `filtered` through `sectionGroups` (a second `useMemo` that groups). Both memos must be touched: the filter memo (to add scoring in the non-empty branch) and the render path (to use flat display when `query !== ''`).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Edit distance | Custom ad-hoc string metric | Standard Levenshtein DP matrix | Levenshtein is well-understood, trivially portable, handles all Unicode; ad-hoc metrics produce inconsistent false positives |
| Relevance ranking | Custom multi-pass logic | Simple tiered integer score as described | Additive scoring with field weights is already industry standard for small-vocabulary search; more complex IR approaches (BM25, TF-IDF) are overkill at 364 entries |
| Case folding | Manual char replacement | `.to_lowercase()` (Rust) / `.toLowerCase()` (TS) | Both handle Unicode case-folding including German umlauts correctly |

**Key insight:** At 364 entries total, brute-force O(n) scan is completely acceptable. No index, no trie, no BK-tree needed.

---

## Common Pitfalls

### Pitfall 1: Scoring Over HelpRow Instead of HelpEntry (P-HS-01)

**What goes wrong:** `HelpRow` has no `search_aliases` field. Scoring over `HelpRow` means aliases are silently ignored.
**Why it happens:** `filter_help_rows` currently takes `&[HelpRow]`. A naive upgrade adds alias scoring there.
**How to avoid:** Use `help_entries_all()` (Rust iterator) and `helpEntriesAll()` (TS function) as the base for scoring. D-58.5 is explicit: "Phase 59 matcher reads search_aliases from HelpEntry without any HelpRow involvement."
**Warning signs:** Tests for alias resolution failing even when JSON has aliases.

### Pitfall 2: Fuzzy False Positives on Short Queries (P-HS-02)

**What goes wrong:** Query "x" (1 char) fuzzy-matches dozens of entries (every entry has a word starting with or near "x").
**Why it happens:** Levenshtein(1-char, 1-char) = 0 or 1 for many pairs.
**How to avoid:** Apply fuzzy ONLY when `query.len() >= 2`. With threshold `max(1, query.len()/4)`, a 2-char query has threshold 1 — only exact and 1-typo matches pass. A 1-char query should not use fuzzy at all (substring is enough).
**Warning signs:** Single-letter search returning unexpectedly large ranked lists.

### Pitfall 3: TS Sort Stability Assumption

**What goes wrong:** `Array.sort` was unstable in old engines; ties could appear in different order than Rust's stable `sort_by`.
**Why it happens:** Pre-ES2019 JS engines had unstable sort.
**How to avoid:** The project targets modern iOS (WKWebView) and modern desktop browsers, all of which implement stable sort per ES2019. The Vitest test environment (Node.js 20+) also has stable sort. No workaround needed, but document this assumption.
**Warning signs:** Parity test (Phase 61) failing due to ordering differences.

### Pitfall 4: GUI sectionGroups Memo Renders Grouped Even When Ranked

**What goes wrong:** The new `filteredOrRanked` array (sorted by score) is piped through the existing `sectionGroups` useMemo that re-groups by section + category. The grouping erases the score ordering.
**Why it happens:** `sectionGroups` is downstream of `filtered`, so upgrading `filtered` is not enough — the render path must also bypass grouping.
**How to avoid:** The render JSX must branch on `query !== ''` to pick between the flat ranked list render and the existing section-grouped render. The `sectionGroups` memo can remain but is simply not used when `query !== ''`.
**Warning signs:** Ranked results displayed in category groups, not score order.

### Pitfall 5: Two Tabs in GUI Both Need Upgrading

**What goes wrong:** Only one of the two filter memos (`filtered` for Keyboard Shortcuts tab, or `filteredAllFn` for All Functions tab) is upgraded to use ranking.
**Why it happens:** The two tabs have parallel filter logic. HSMATCH-01 requires BOTH to search aliases.
**How to avoid:** Both `filtered` and `filteredAllFn` memos and their respective render branches must be upgraded. Consider a shared `scoredAndRanked(entries, query)` helper so the logic is DRY even within the TS side.
**Warning signs:** Aliases visible in one tab but not the other; parity test failing.

### Pitfall 6: `filterHelpEntries` in help_data.ts Not Upgraded

**What goes wrong:** `filterHelpEntries` (exported from `help_data.ts`) is used by test fixtures. If it is not upgraded, tests that import it will test the old behavior.
**Why it happens:** `filterHelpEntries` is a separate export alongside the overlay-specific functions; it can be overlooked.
**How to avoid:** Either upgrade `filterHelpEntries` to delegate to the new scorer, or replace its callers with the new function. Tests in `help_data.test.ts` import `filterHelpEntries` directly; those tests will need to be updated or supplemented.
**Warning signs:** Tests pass against old `filterHelpEntries` but `HelpOverlay` uses new scorer — coverage gap.

### Pitfall 7: Rust `unwrap()` in Production Scoring Code

**What goes wrong:** The score function uses `Vec::iter().min().copied().unwrap()` — `unwrap()` is banned in production code by `#![deny(clippy::unwrap_used)]`.
**Why it happens:** The early-exit Levenshtein implementation above has a `.unwrap_or(0)` — that's acceptable. But other patterns (e.g. `chars().next().unwrap()`) are not.
**How to avoid:** Use `.unwrap_or(...)` or `?` (where applicable). For the `min()` call on a non-empty vector, use `.unwrap_or(usize::MAX)` or check length first.
**Warning signs:** `cargo clippy` failing on the new code.

---

## Code Examples

### Scorer skeleton — Rust

```rust
// Source: [ASSUMED] — design derived from spec + code audit; pattern matches project conventions

/// Tier score constants. Higher = better match.
const SCORE_EXACT_NAME: u8 = 40;
const SCORE_PREFIX_NAME: u8 = 32;
const SCORE_SUBSTR_NAME: u8 = 24;
const SCORE_FUZZY_NAME:  u8 = 8;
const SCORE_EXACT_ALIAS: u8 = 35;
const SCORE_PREFIX_ALIAS: u8 = 28;
const SCORE_SUBSTR_ALIAS: u8 = 21;
const SCORE_FUZZY_ALIAS:  u8 = 7;
const SCORE_EXACT_DESC:  u8 = 30;
const SCORE_PREFIX_DESC: u8 = 24;
const SCORE_SUBSTR_DESC: u8 = 18;
const SCORE_FUZZY_DESC:  u8 = 6;
const SCORE_EXACT_CAT:   u8 = 20;
const SCORE_PREFIX_CAT:  u8 = 16;
const SCORE_SUBSTR_CAT:  u8 = 12;
const SCORE_FUZZY_CAT:   u8 = 4;

fn tier_score(field: &str, q: &str, exact: u8, prefix: u8, substr: u8, fuzzy: u8) -> u8 {
    let f = field.to_lowercase();
    if f == q { return exact; }
    // Word-prefix: any space-delimited token in f starts with q
    if f.starts_with(q) || f.split_whitespace().any(|w| w.starts_with(q)) { return prefix; }
    if f.contains(q) { return substr; }
    if q.len() >= 2 {
        let max_dist = (q.len() / 4).max(1);
        // For short fields: compare whole field; for long fields: compare each word
        let min_dist = if f.len() <= 20 {
            levenshtein_bounded(&f, q, max_dist)
        } else {
            f.split_whitespace()
                .map(|w| levenshtein_bounded(w, q, max_dist))
                .min()
                .unwrap_or(max_dist + 1)
        };
        if min_dist <= max_dist { return fuzzy; }
    }
    0
}

/// Returns the best score for a query against a HelpEntry (0 = no match).
pub fn score_entry(entry: &HelpEntry, q: &str) -> u8 {
    let name_score = tier_score(&entry.display_name, q,
        SCORE_EXACT_NAME, SCORE_PREFIX_NAME, SCORE_SUBSTR_NAME, SCORE_FUZZY_NAME);
    let desc_score = tier_score(&entry.description, q,
        SCORE_EXACT_DESC, SCORE_PREFIX_DESC, SCORE_SUBSTR_DESC, SCORE_FUZZY_DESC);
    let cat_score  = tier_score(&entry.category, q,
        SCORE_EXACT_CAT, SCORE_PREFIX_CAT, SCORE_SUBSTR_CAT, SCORE_FUZZY_CAT);
    let alias_score = entry.search_aliases.iter()
        .map(|a| tier_score(a, q, SCORE_EXACT_ALIAS, SCORE_PREFIX_ALIAS, SCORE_SUBSTR_ALIAS, SCORE_FUZZY_ALIAS))
        .max()
        .unwrap_or(0);
    name_score.max(desc_score).max(cat_score).max(alias_score)
}

/// Returns ranked HelpRow entries for a non-empty query, highest score first.
/// Tie-break: display_name ascending. No category headers (flat list).
pub fn ranked_help_entries(query: &str) -> Vec<HelpRow> {
    debug_assert!(!query.is_empty(), "ranked_help_entries called with empty query");
    let q = query.to_lowercase();
    let mut scored: Vec<(u8, &'static HelpEntry)> = help_entries_all()
        .filter(|e| e.status == "implemented")
        .filter_map(|e| {
            let s = score_entry(e, &q);
            if s > 0 { Some((s, e)) } else { None }
        })
        .collect();
    // Stable sort: score DESC, then display_name ASC for ties.
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.display_name.cmp(&b.1.display_name)));
    scored.into_iter().map(|(_, e)| HelpRow {
        key: e.key_path.clone().unwrap_or_else(|| {
            if e.xrom.is_none() { format!("XEQ \"{}\"", e.display_name) } else { String::new() }
        }),
        op: e.display_name.clone(),
        desc: e.description.clone(),
    }).collect()
}
```

### Scorer skeleton — TypeScript

```ts
// Source: [ASSUMED] — mirror of Rust implementation; to be added to hp41-gui/src/help_data.ts

const SCORE = {
    exactName: 40, prefixName: 32, substrName: 24, fuzzyName: 8,
    exactAlias: 35, prefixAlias: 28, substrAlias: 21, fuzzyAlias: 7,
    exactDesc: 30, prefixDesc: 24, substrDesc: 18, fuzzyDesc: 6,
    exactCat: 20, prefixCat: 16, substrCat: 12, fuzzyCat: 4,
} as const;

function tierScore(field: string, q: string,
    exact: number, prefix: number, substr: number, fuzzy: number): number {
    const f = field.toLowerCase();
    if (f === q) return exact;
    if (f.startsWith(q) || f.split(/\s+/).some(w => w.startsWith(q))) return prefix;
    if (f.includes(q)) return substr;
    if (q.length >= 2) {
        const maxDist = Math.max(1, Math.floor(q.length / 4));
        const minDist = f.length <= 20
            ? levenshteinBounded(f, q, maxDist)
            : Math.min(...f.split(/\s+/).map(w => levenshteinBounded(w, q, maxDist)));
        if (minDist <= maxDist) return fuzzy;
    }
    return 0;
}

export function scoreEntry(entry: HelpEntry, q: string): number {
    const nameScore = tierScore(entry.display_name, q,
        SCORE.exactName, SCORE.prefixName, SCORE.substrName, SCORE.fuzzyName);
    const descScore = tierScore(entry.description, q,
        SCORE.exactDesc, SCORE.prefixDesc, SCORE.substrDesc, SCORE.fuzzyDesc);
    const catScore  = tierScore(entry.category, q,
        SCORE.exactCat, SCORE.prefixCat, SCORE.substrCat, SCORE.fuzzyCat);
    const aliasScore = (entry.search_aliases ?? [])
        .map(a => tierScore(a, q, SCORE.exactAlias, SCORE.prefixAlias, SCORE.substrAlias, SCORE.fuzzyAlias))
        .reduce((a, b) => Math.max(a, b), 0);
    return Math.max(nameScore, descScore, catScore, aliasScore);
}

export function rankedEntries(entries: readonly HelpEntry[], q: string): HelpEntry[] {
    return entries
        .map(e => ({ e, s: scoreEntry(e, q) }))
        .filter(({ s }) => s > 0)
        .sort((a, b) => b.s - a.s || a.e.display_name.localeCompare(b.e.display_name))
        .map(({ e }) => e);
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Phase 25: static `HELP_DATA` const array | JSON-loaded `HelpEntry` via `OnceLock` + `include_str!` | Phase 25/26 | HelpEntry is the scoring base |
| Simple substring filter | Tiered scoring + fuzzy (Phase 59) | Phase 59 | Aliases and typos now match |
| Category-grouped view always | Flat ranked when query non-empty, grouped when empty | Phase 59 | Best matches surface first |

**Deprecated/outdated:**
- `filterHelpEntries` in `help_data.ts`: still present but will be superseded by `rankedEntries`; old behavior (no aliases, no ranking) should be updated or clearly documented as the pre-Phase-59 path.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Bounded Levenshtein with threshold `max(1, query.len()/4)` strikes the right balance for the ~364-entry domain | Fuzzy Algorithm Choice | Could be too tight (misses near-typos) or too loose (false positives); threshold is a constant, easy to adjust in tests |
| A2 | Score constants (40/32/24/8 for name tiers, etc.) produce correct relative ordering across real entries | Architecture Patterns §Tiered Scoring | If scoring is inverted (wrong function wins), the spec's example queries would fail in Phase 61 parity tests |
| A3 | German umlauts in aliases will be entered with actual umlaut characters by German users; no fold needed | Architecture Patterns §Fuzzy Algorithm | If users on non-German keyboards type "Annuitat" (no umlaut) and no fold alias is provided, fuzzy will bridge it via distance 1–2 |
| A4 | TypeScript `Array.sort` is stable in the project's target environments (iOS WKWebView, desktop browsers, Node.js) | Common Pitfalls §Pitfall 3 | If unstable sort is encountered, parity with Rust would require explicit secondary key enforcement |
| A5 | `levenshtein_bounded` early-exit `.min().unwrap_or(usize::MAX)` does not trigger clippy denial; alternative is to use `unwrap_or` | Code Examples §Rust | If clippy rejects, use `unwrap_or(usize::MAX)` — already shown in code skeleton |

---

## Open Questions

1. **Score constant calibration**
   - What we know: the tiers (exact > prefix > substring > fuzzy) are fixed by spec; the exact numeric values are researcher-chosen
   - What's unclear: whether the proposed constants (40/32/24/8 etc.) produce correct ordering when an entry has a strong alias hit vs. a weak display_name hit (e.g. SQRT: display_name="SQRT", alias="Wurzel" — query "sqrt" should be tier 40 on display_name, query "wurzel" should be tier 35 on alias; both should surface SQRT as result #1)
   - Recommendation: write the scoring tests first (Phase 61 HSQUAL-01), using synthetic entry fixtures — tune constants in the scoring function until all spec examples pass. Constants are localized to the score function, not structural.

2. **Keyboard Shortcuts tab filter behavior when query is active**
   - What we know: the Keyboard Shortcuts tab shows only entries with `key_path !== null` (D-26.8)
   - What's unclear: when a query matches an alias of an entry that has `key_path === null`, should the Keyboard Shortcuts tab show it? Or should aliases only affect the All Functions tab?
   - Recommendation: HSMATCH-01 says "Both matchers score a query over... search_aliases". The All Functions tab should show alias hits; the Keyboard Shortcuts tab only shows entries that have a `key_path`. This is the natural outcome of the existing `key_path !== null` filter applied before scoring. No change to the tab semantics needed.

3. **`ranked_help_entries` module placement**
   - What we know: D-58.5 says "Phase 59 matcher reads search_aliases from HelpEntry without any HelpRow involvement"; CLAUDE.md says `hp41-core` must not gain CLI deps
   - What's unclear: whether scorer goes in `hp41-cli/src/help_data.rs` (inline with HelpEntry) or a new `hp41-cli/src/search.rs` module
   - Recommendation: inline in `help_data.rs` — it is 50–80 LOC, the existing test infrastructure is already there, and the precedent is that all help-data functions live in that file. A separate `search.rs` is warranted only if the file grows beyond ~600 LOC.

---

## Environment Availability

> This phase is purely code/config — no external tools, services, or CLIs required beyond the project's own build stack.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable | Rust scorer | ✓ | (project MSRV 1.88) | — |
| Node.js / npm | Vitest tests | ✓ | (project standard) | — |
| `just` | Build/test runner | ✓ | (project standard) | — |

---

## Validation Architecture

> `nyquist_validation` is `true` in `.planning/config.json`. This section is required.

### Test Framework

| Property | Value |
|----------|-------|
| Framework (Rust) | Cargo test (`just test`) |
| Framework (TS) | Vitest (`cd hp41-gui && npm test`) |
| Config file (TS) | `hp41-gui/vite.config.ts` (`test:` block, jsdom, `globals: false`) |
| Quick run (Rust) | `just test-core` (hp41-core only, fast); or `cargo test -p hp41-cli --test phase59_help_search` |
| Quick run (TS) | `cd hp41-gui && npm test -- --reporter=verbose help_data` |
| Full suite | `just test` (Rust workspace) + `cd hp41-gui && npm test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| HSMATCH-01 | Score over display_name + description + category + search_aliases | unit (Rust + TS) | `cargo test -p hp41-cli --test phase59_help_search` | ❌ Wave 0 |
| HSMATCH-02 | exact > prefix > substring > fuzzy ordering | unit (Rust + TS) | `cargo test -p hp41-cli --test phase59_help_search -- tier_order` | ❌ Wave 0 |
| HSMATCH-03 | Hand-rolled fuzzy (no dep), typo "Zineszins" → TVM | unit (Rust + TS) | `cargo test -p hp41-cli --test phase59_help_search -- fuzzy_typo` | ❌ Wave 0 |
| HSMATCH-04 | Empty query = unchanged grouped view, non-empty = flat ranked | unit (Rust + TS) | `cargo test -p hp41-cli -- empty_query_returns_all` (existing) + new empty-query invariant test | Partial (existing `empty_query_returns_all_rows` in `help_data.rs`) |
| HSMATCH-05 | DE alias "Zinseszins" + EN alias "compound interest" both → TVM | unit (Rust + TS) | `cargo test -p hp41-cli --test phase59_help_search -- alias_de_en` | ❌ Wave 0 |
| HSUX-01 | Overlay input reused (no new view) | smoke / render | Existing `HelpOverlay.test.tsx` input test + new ranked-render test | Partial (existing) |
| HSUX-02 | Typo "Wurzel" → SQRT surfaces as top result | unit (Rust + TS) | `cargo test -p hp41-cli --test phase59_help_search -- fuzzy_wurzel` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `just test` (Rust workspace, ~seconds) + `cd hp41-gui && npm test` (Vitest, ~seconds)
- **Per wave merge:** `just test` + `cd hp41-gui && npm test`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `hp41-cli/tests/phase59_help_search.rs` — covers HSMATCH-01/02/03/04/05, HSUX-02: scoring tiers, fuzzy typo hits (Zineszins→TVM, Wurzel→SQRT), DE/EN alias resolution, empty-query invariance. Uses synthetic HelpEntry fixtures (no real JSON needed — tests the scoring logic, not the data).
- [ ] `hp41-gui/src/help_data.test.ts` (extended) — Phase 59 scorer tests: `scoreEntry` tier ordering, fuzzy, alias, empty-passthrough.
- [ ] `hp41-gui/src/HelpOverlay.test.tsx` (extended) — flat-ranked render branch: query non-empty → results appear without section headers; query empty → section groups appear (regression guard for HSMATCH-04).

Note: the parity fixture (HSQUAL-02) is scoped to Phase 61; Phase 59 only needs per-frontend unit tests.

---

## Security Domain

> This phase adds no new network endpoints, auth paths, file access, or trust boundaries. The scorer is pure text processing over static compile-time data. No ASVS categories apply. `security_enforcement` is not disabled, but there are no applicable threat patterns for a pure text-scoring function over in-memory static data.

---

## Sources

### Primary (HIGH confidence)
- `hp41-cli/src/help_data.rs` (lines 66–88, 276–365) — HelpEntry struct, HelpRow struct, filter_help_rows, help_overlay_rows, existing tests [VERIFIED: direct read]
- `hp41-gui/src/HelpOverlay.tsx` (lines 221–291) — Both filter useMemo blocks, empty-query branch points [VERIFIED: direct read]
- `hp41-gui/src/help_data.ts` (lines 59–94, 154–166) — HelpEntry TS interface, filterHelpEntries, helpEntriesAll [VERIFIED: direct read]
- `hp41-cli/src/ui.rs` (lines 447–496) — Render path for help overlay, existing empty/non-empty title branch [VERIFIED: direct read]
- `.planning/phases/58-data-model/58-01-SUMMARY.md` — D-58.5: "Phase 59 matcher reads search_aliases from HelpEntry without any HelpRow involvement" [VERIFIED: direct read]
- `.planning/STATE.md` — Pitfalls P-HS-01/02, locked decisions, architecture notes [VERIFIED: direct read]
- `.planning/REQUIREMENTS.md` — HSMATCH-01 through HSUX-02 [VERIFIED: direct read]
- `docs/superpowers/specs/2026-06-03-help-search-enrichment-design.md` — Approved design spec [VERIFIED: direct read]
- `CLAUDE.md` — Frozen invariants, dependency rules, CLI↔GUI parity discipline [VERIFIED: direct read]

### Secondary (MEDIUM confidence)
- Entry count analysis: 6 JSON pools × `grep -c "implemented"` → 364 total implemented entries [VERIFIED: bash grep]
- Vitest config: `hp41-gui/vite.config.ts` confirms `globals: false`, jsdom environment, `npm test` = `vitest run` [VERIFIED: direct read]
- `just` recipe audit: `just test` = `cargo test --workspace`, `just gui-ci` includes `npm test` [VERIFIED: direct read]

### Tertiary (LOW confidence — assumptions)
- Score constants (40/32/24/8 etc.) — [ASSUMED] chosen to preserve tier ordering; calibration by unit tests
- Levenshtein threshold `max(1, len/4)` — [ASSUMED] common heuristic; needs validation on spec examples

---

## Metadata

**Confidence breakdown:**
- Code audit (HelpEntry, HelpRow, filter paths): HIGH — read directly
- Architecture approach (score over HelpEntry, flat-ranked new path): HIGH — validated against D-58.5 and spec
- Fuzzy algorithm (bounded Levenshtein): HIGH — algorithm is trivially portable; implementation sketch needs unit-test validation
- Score constants: LOW — assumed; must be validated against the test cases from the spec
- Unicode/umlaut handling: HIGH — Rust `.to_lowercase()` and TS `[...str]` are well-specified

**Research date:** 2026-06-04
**Valid until:** 2026-07-04 (stable codebase, no moving deps)
