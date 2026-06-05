---
phase: 59-runtime-matcher
reviewed: 2026-06-04T20:06:37Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - hp41-cli/src/help_data.rs
  - hp41-cli/src/ui.rs
  - hp41-cli/tests/phase59_help_search.rs
  - hp41-gui/src/help_data.ts
  - hp41-gui/src/HelpOverlay.tsx
  - hp41-gui/src/help_data.test.ts
  - hp41-gui/src/HelpOverlay.test.tsx
findings:
  critical: 2
  warning: 4
  info: 3
  total: 9
status: issues_found
---

# Phase 59: Code Review Report

**Reviewed:** 2026-06-04T20:06:37Z
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

Phase 59 ships two mirrored relevance scorers (`hp41-cli/src/help_data.rs` ::
`score_entry` / `ranked_help_entries` / `tier_score` / `levenshtein_bounded`,
and `hp41-gui/src/help_data.ts` :: `scoreEntry` / `rankedEntries` / `tierScore`
/ `levenshteinBounded`) plus the render-branch wiring in `ui.rs` and
`HelpOverlay.tsx`. The stated contract is that the two scorers are
**behaviorally identical** so the CLI and GUI rank help-search results the same
way.

The tier weights, the bounded-Levenshtein DP, the early-exit, the
`q >= 2` fuzzy gate, the four-field aggregation, and the empty-query
short-circuit all match between the two implementations. The empty-query
grouped render path in both `ui.rs` and `HelpOverlay.tsx` is genuinely left
untouched (verified against the pre-existing `help_overlay_rows` /
`sectionGroups` paths). `hp41-core` is not touched. No `unwrap_used`, no
`println!`/`eprintln!` in the new code, no debug artifacts, no empty catch.

However, the **parity invariant is broken in three places** where Rust and
TypeScript primitives differ in ways that change ranking: the tie-break
comparator (`String::cmp` vs `localeCompare`), the "short field" length gate
(`String::len` bytes vs `string.length` UTF-16 units), and the fuzzy-gate /
threshold length (`q.len()` bytes vs `q.length` UTF-16 units). The first
already diverges on shipped data (`Z↑N`, `ΣBSTAT`, `C×`, mixed-case names); the
latter two are latent until Phase 60 lands the German aliases the fixtures
already exercise (`Zinseszins`, `Wurzel`). There is also a CLI↔GUI search-pool
mismatch and an untrimmed-CLI-query gap.

## Critical Issues

### CR-01: Tie-break comparator diverges — `String::cmp` (codepoint) vs `localeCompare` (locale)

**File:** `hp41-cli/src/help_data.rs:492-495` and `hp41-gui/src/help_data.ts:461-464`
**Issue:**
The two scorers sort by `(score DESC, display_name ASC)`. For the secondary
key the Rust side uses codepoint ordering:

```rust
scored.sort_by(|a, b| {
    b.0.cmp(&a.0)
        .then_with(|| a.1.display_name.cmp(&b.1.display_name))
});
```

while the GUI uses locale ordering:

```ts
scored.sort(([sa, ea], [sb, eb]) => {
    if (sb !== sa) return sb - sa;
    return ea.display_name.localeCompare(eb.display_name);
});
```

`String::cmp` compares Unicode scalar values bytewise (uppercase before
lowercase, ASCII before all multi-byte glyphs). `localeCompare` is
case-folding and accent/glyph aware. These disagree on real, currently-shipped
`display_name` values. Verified divergences on actual mnemonics:

- `Z↑N` vs `ZEBRA`-class names: `localeCompare = -1`, codepoint = `+1`
- `C×` / `ΣBSTAT` vs lowercase-leading names: opposite sign
- `SIN` vs `sin`, `ALPHA` vs `alpha`: opposite sign

Because ties on `score` are common (every entry that matches only via a
substring of `display_name` scores 24, every exact-name match scores 40, etc.),
the tie-break runs frequently, so **CLI and GUI will present tied results in a
different order** — a direct violation of the Phase 59 CLI↔GUI parity invariant
("same ranking (score DESC, then display_name ASC)").

**Fix:** Make the GUI tie-break byte/codepoint-identical to Rust `str::cmp`.
`localeCompare` cannot reproduce `String::cmp`; use a plain scalar comparison:

```ts
scored.sort(([sa, ea], [sb, eb]) => {
    if (sb !== sa) return sb - sa;
    // Match Rust String::cmp (Unicode-scalar / codepoint order), NOT locale order.
    return ea.display_name < eb.display_name ? -1
         : ea.display_name > eb.display_name ? 1 : 0;
});
```

(JS `<`/`>` on strings compares by UTF-16 code unit; for the BMP glyphs used in
the mnemonics this equals Rust's codepoint order. If any astral-plane glyph is
ever added, compare via `[...a]` codepoint arrays to stay exact.) Add a
CLI↔GUI parity fixture asserting identical order for a pool containing
`Z↑N`, `ΣBSTAT`, `C×`.

---

### CR-02: Fuzzy "short field" gate uses byte length in Rust, UTF-16 length in TS

**File:** `hp41-cli/src/help_data.rs:399` and `hp41-gui/src/help_data.ts:420`
**Issue:**
The strategy switch between "fuzzy the whole field" and "fuzzy each word" is
gated on a length of 20:

```rust
let min_dist = if f.len() <= 20 {                 // f.len() = BYTE length
    levenshtein_bounded(&f, q, max_dist)
} else {
    f.split_whitespace().map(|w| levenshtein_bounded(w, q, max_dist)).min()...
};
```

```ts
const minDist = f.length <= 20                     // f.length = UTF-16 units
    ? levenshteinBounded(f, q, maxDist)
    : Math.min(...f.split(/\s+/).map(w => levenshteinBounded(w, q, maxDist)));
```

`String::len()` in Rust is the UTF-8 **byte** count; `string.length` in JS is
the UTF-16 **code-unit** count. A field containing umlauts/glyphs crosses the
`<= 20` boundary at different content lengths in the two engines. Example: a
field of 19 visible chars containing two umlauts is 21 bytes (Rust takes the
**word-split** branch) but `length === 19` in JS (takes the **whole-field**
branch). The whole-field vs per-word Levenshtein can yield different
`min_dist`, hence a different fuzzy hit/miss → different score → different
ranking.

This is exactly the German-term path the fixtures target (`Zineszins` →
`Zinseszins`, `Wurzel`). No **currently shipped** field straddles the boundary
(empirically 0 today), but Phase 60 adds `search_aliases` with these very terms,
and `tier_score`/`tierScore` run over alias fields too. The defect is latent but
guaranteed to activate with the data this phase was built for.

**Fix:** Make both sides measure the same unit. Simplest is to compare Unicode
scalar count on both sides (the Rust `levenshtein_bounded` already works on
`Vec<char>`, so use char count for the gate too):

```rust
let f_len = f.chars().count();
let min_dist = if f_len <= 20 { ... } else { ... };
```

```ts
const fLen = [...f].length;
const minDist = fLen <= 20 ? ... : ...;
```

Pick one canonical unit (code-point count is the natural choice given the
Levenshtein already iterates code points) and document it as the parity
contract. Add a parity fixture with a 19–21 "char" field containing umlauts.

## Warnings

### WR-01: Fuzzy gate and threshold use `q.len()` (bytes) vs `q.length` (UTF-16)

**File:** `hp41-cli/src/help_data.rs:396-397` and `hp41-gui/src/help_data.ts:418-419`
**Issue:**
Same byte-vs-UTF-16 mismatch as CR-02, applied to the **query**:

```rust
if q.len() >= 2 {                       // bytes
    let max_dist = (q.len() / 4).max(1);
```
```ts
if (q.length >= 2) {                    // UTF-16 units
    const maxDist = Math.max(1, Math.floor(q.length / 4));
```

For a single multi-byte query char (e.g. `"ä"`: 2 bytes, `length === 1`), Rust
passes the `>= 2` fuzzy gate while JS skips fuzzy entirely → divergent results.
For longer multi-byte queries, `max_dist` also diverges (`q.len()/4` over bytes
vs UTF-16 units), changing the fuzzy threshold. Lower-risk than CR-02 because
real queries are short ASCII tokens, but it is the same class of bug and should
be fixed in the same pass: switch both to code-point count
(`q.chars().count()` / `[...q].length`).
**Fix:** Use code-point count on both sides, consistently with the CR-02 fix.

### WR-02: CLI search pool ≠ GUI Keyboard-Shortcuts search pool

**File:** `hp41-cli/src/help_data.rs:484-485` and `hp41-gui/src/HelpOverlay.tsx:214-228`
**Issue:**
The CLI `ranked_help_entries` scores `help_entries_all().filter(status ==
"implemented")` — the full 6-pool set, including `key_path: null` ops (SIN, LN,
CLRG, …). The GUI **Keyboard Shortcuts** tab ranks `allEntries`, which is
`helpEntriesAll().filter(e => e.key_path !== null)` — `key_path: null` ops are
excluded. Only the GUI **All Functions** tab (`allFunctionsEntries()`) uses the
same full implemented pool as the CLI.

Net effect: typing the same query into the CLI overlay and into the GUI
*Keyboard Shortcuts* tab yields **different result sets** (the CLI surfaces
`key_path: null` built-ins; that GUI tab does not). There is also a within-CLI
inconsistency: the CLI empty path is `key_path`-filtered (grouped), but the
non-empty path is not, so typing a query suddenly reveals ops that were
invisible a keystroke earlier. If the intended parity counterpart is the
All-Functions tab, document that explicitly; if it is the Shortcuts tab, the
pools must be reconciled.
**Fix:** Decide the canonical CLI counterpart and align pools. Either filter the
CLI ranked pool to `key_path.is_some()` to match the Shortcuts tab, or document
(in the scorer header and the phase notes) that the CLI single overlay mirrors
the GUI **All Functions** tab pool, and add a parity test pinning that choice.

### WR-03: CLI query is neither trimmed nor pre-lowercased before scoring

**File:** `hp41-cli/src/app.rs:594-595`, `hp41-cli/src/help_data.rs:483`, `hp41-gui/src/HelpOverlay.tsx:225,234`
**Issue:**
The GUI lowercases **and trims** before scoring (`query.toLowerCase().trim()`).
The CLI accumulates raw `KeyCode::Char(c)` (including spaces) into
`help_search_query` and `ranked_help_entries` only does `query.to_lowercase()`
— **no trim**. `tier_score` lowercases the field but compares against the
untrimmed `q`. A leading/trailing space the GUI would strip causes the CLI to
miss matches that the GUI finds: `" sin"` fails `f == q`, fails
`f.starts_with(q)`, fails the word-prefix check (words never carry a leading
space) and fails `f.contains(q)` for prefix-position matches. Result: divergent
match sets for whitespace-bearing queries.
**Fix:** Trim in the CLI before scoring to mirror the GUI contract:
`let q = query.trim().to_lowercase();` (and keep the `debug_assert!(!q.is_empty())`
checking the post-trim value, or guard the empty-after-trim case in `ui.rs`).

### WR-04: `ui.rs` non-empty path applies the empty-query title/count semantics to a different match concept

**File:** `hp41-cli/src/ui.rs:453-467, 489-497`
**Issue:**
In the empty path `match_count` counts non-header rows of the grouped
(`key_path`-filtered) view; in the non-empty path `match_count =
display_rows.len()` counts the full implemented ranked set. Combined with WR-02,
the displayed `[N matches]` count for a query is computed over a *different
population* than the rows shown a keystroke earlier in the empty view, which can
read as a jump (e.g. empty view shows ~30 shortcut rows, first character shows a
much larger ranked count). Not incorrect arithmetic, but a UX/consistency
defect tied to the WR-02 pool decision.
**Fix:** Resolve WR-02 first; once the pools are reconciled the count becomes
consistent. No separate code change needed beyond the WR-02 fix.

## Info

### IN-01: `f.split(/\s+/)` yields a leading empty word on leading-whitespace fields (benign today)

**File:** `hp41-gui/src/help_data.ts:416,422`
**Issue:** JS `"  foo".split(/\s+/)` returns `["", "foo"]`; Rust
`split_whitespace()` never yields empty tokens. In the word-prefix branch
`"".startsWith(q)` is false for the non-empty `q` used here, and in the fuzzy
long-field branch `levenshteinBounded("", q)` returns `q.length`
(> `maxDist` for `q.length >= 2`), so the extra empty word cannot change the
result. Latent only if a field begins with whitespace AND the gate logic later
changes.
**Fix:** Use `f.split(/\s+/).filter(Boolean)` (or `f.trim().split(/\s+/)`) to
match `split_whitespace()` semantics exactly and remove the latent asymmetry.

### IN-02: `ranked_help_entries` empty-query contract is a `debug_assert!` only

**File:** `hp41-cli/src/help_data.rs:482`
**Issue:** The empty-query guard is `debug_assert!(!query.is_empty(), ...)`,
which is compiled out in release. The sole caller (`ui.rs:453`) correctly
branches on `help_search_query.is_empty()` first, so this is fine today, but a
future caller passing an empty string in release would silently score every
entry against `""` ( `"".contains("")`/`starts_with("")` are true → every entry
scores its highest tier). The GUI counterpart (`rankedEntries`) instead returns
the pool unchanged for `q === ''`, so the two functions are not interchangeable
on empty input.
**Fix:** Make the Rust function defensively return early (mirroring the GUI):
`if query.is_empty() { return Vec::new(); }` or have it return the unranked
pool, matching whichever empty-semantics the GUI exposes — and keep them
parity-equal.

### IN-03: Stale `filter_help_rows` reference in the `render_help_overlay` doc comment

**File:** `hp41-cli/src/ui.rs:409`
**Issue:** The function doc still says the table is "driven by
`help_data::filter_help_rows(...)` over `app.help_search_query`", but Phase 59
routes the non-empty case through `ranked_help_entries`. The comment is now half
right (empty path still uses `filter_help_rows("")`). Minor doc drift.
**Fix:** Update the doc comment to describe the two-branch behavior (empty →
`help_overlay_rows` + `filter_help_rows`; non-empty → `ranked_help_entries`).

---

## Resolution (orchestrator, post-review — commit `5db690c`)

Parity-critical findings fixed in the same pass and re-verified GREEN
(`cargo test --workspace`, `tsc --noEmit`, vitest 302/302):

| Finding | Status | Fix |
|---------|--------|-----|
| CR-01 (tie-break comparator) | ✅ fixed | GUI `localeCompare` → scalar `<`/`>` to match Rust `String::cmp` |
| CR-02 (fuzzy short-field gate byte vs UTF-16) | ✅ fixed | Both sides gate on code-point count (`chars().count()` / `[...f].length`) |
| WR-01 (query length byte vs UTF-16) | ✅ fixed | Query length via code-point count on both sides |
| WR-03 (CLI query not trimmed) | ✅ fixed | `ranked_help_entries` now `query.trim().to_lowercase()` + empty-after-trim guard |
| IN-02 (release-mode empty-query guard) | ✅ fixed | Empty-after-trim guard returns `Vec::new()`, mirroring GUI passthrough intent |

All fixes are behavior-preserving for the current ASCII test fixtures; they
correct the active CR-01 divergence and the Phase-60-latent CR-02/WR-01 ones.

**Deferred (need a design decision, not a mechanical fix):**

- **WR-02** — CLI ranked pool (full implemented 6-pool) vs GUI **Keyboard
  Shortcuts** tab pool (`key_path !== null`). Requires deciding whether the CLI
  single overlay mirrors the GUI **All Functions** tab (full pool — likely
  intended) or the Shortcuts tab. Left for phase verification / user.
- **WR-04** — `[N matches]` count consistency; resolves once WR-02 is decided.
- **IN-01, IN-03** — cosmetic (leading-empty-word split; stale `ui.rs` doc
  comment). Non-functional.

---

_Reviewed: 2026-06-04T20:06:37Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
