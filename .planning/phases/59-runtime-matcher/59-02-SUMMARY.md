---
phase: 59-runtime-matcher
plan: "02"
subsystem: help-search-cli
tags: [tdd, green-wave-2, scorer, rust, cli]
dependency_graph:
  requires: [59-01, 58-data-model]
  provides: [cli-scorer, ranked-help-entries, flat-ranked-render]
  affects: [hp41-cli/src/help_data.rs, hp41-cli/src/ui.rs]
tech_stack:
  added: []
  patterns: [tiered-scoring, bounded-levenshtein, flat-ranked-render-branch, cli-gui-mirror-discipline]
key_files:
  modified:
    - hp41-cli/src/help_data.rs
    - hp41-cli/src/ui.rs
decisions:
  - "Levenshtein bounded at max_dist+1 with early-exit on row-min; unwrap_or(usize::MAX) for clippy compliance"
  - "tier_score applies fuzzy only when q.len() >= 2 to guard against P-HS-02 false positives on single-char queries"
  - "ranked_help_entries returns owned Vec<HelpRow> (no headers); ui.rs branches on is_empty() to share the downstream render loop"
  - "match_count in the non-empty path equals display_rows.len() (all rows are data rows, no header filtering needed)"
metrics:
  duration: "~10 minutes"
  completed: "2026-06-04"
  tasks_completed: 2
  tasks_total: 2
  files_created: 0
  files_modified: 2
---

# Phase 59 Plan 02: CLI Scorer GREEN Summary

Wave-2 GREEN plan: implements `levenshtein_bounded`, `tier_score`, `score_entry`, and `ranked_help_entries` in `hp41-cli/src/help_data.rs`, then wires `ui.rs` `render_help_overlay` to branch on the query being empty vs non-empty.

---

## One-Liner

Hand-rolled tiered scorer (40/32/24/8 name, 35/28/21/7 alias tiers) + bounded Levenshtein in help_data.rs; flat-ranked render branch in ui.rs — all 7 RED Rust integration tests from plan 59-01 are now GREEN.

---

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Hand-rolled scorer (levenshtein_bounded, tier_score, score_entry, ranked_help_entries) | 1a25292 | hp41-cli/src/help_data.rs (modified) |
| 2 | Flat-ranked render branch in ui.rs render_help_overlay | 6e764c1 | hp41-cli/src/ui.rs (modified) |

---

## Verification

- `cargo test -p hp41-cli --test phase59_help_search`: **7/7 PASS** (all RED tests GREEN)
- `just test`: full Rust workspace — **0 failures, 0 regressions**
- `cargo build -p hp41-cli`: clean, no unwrap warnings, no clippy issues

---

## Implementation Details

### Task 1 — help_data.rs scorer

Added ~197 lines immediately before `filter_help_rows`:

1. **Tier constants** — 16 `u8` consts (SCORE_EXACT_NAME=40 … SCORE_FUZZY_CAT=4), with a comment pointing to 59-RESEARCH.md §Open Questions Q1 for tuning.

2. **`levenshtein_bounded(a, b, max_dist) -> usize`** — classic DP with `chars().collect::<Vec<char>>()` for Unicode-correct umlaut handling. Early-exit: if all values in `curr` row exceed `max_dist`, returns `max_dist + 1` immediately. Uses `.unwrap_or(usize::MAX)` on the `.min()` call (no bare `.unwrap()` — clippy compliant).

3. **`tier_score(field, q, exact, prefix, substr, fuzzy) -> u8`** (private) — lowercases field; checks exact == / word-prefix (starts_with or any split_whitespace token) / substring (contains) / fuzzy (only when q.len() >= 2, threshold = max(1, q.len()/4), whole-field for len <= 20 else min over words).

4. **`pub fn score_entry(entry: &HelpEntry, q: &str) -> u8`** — max over all four field scores. Alias score uses `.max().unwrap_or(0)` on the iterator.

5. **`pub fn ranked_help_entries(query: &str) -> Vec<HelpRow>`** — lowercases query once, filters `status == "implemented"`, scores via score_entry, keeps score > 0, stable-sorts by (score DESC, display_name ASC), projects to `HelpRow` reusing the exact `key_path else XEQ "NAME"` logic from `help_overlay_rows`.

6. **Provenance comment** — `// Mirrored scorer — keep byte-equivalent in behavior with hp41-gui/src/help_data.ts per Phase 59 (CLI<->GUI parity). Parity fixture lands in Phase 61.`

`filter_help_rows` empty-guard, `HelpRow` struct, and all existing tests are **unchanged** (verified by diff).

### Task 2 — ui.rs render branch

Replaced the single `overlay_rows`/`filtered` block in `render_help_overlay` with:
- `if app.help_search_query.is_empty()` → existing `help_overlay_rows()` + `filter_help_rows("", …)` path, header-interleaved, unchanged output (HSMATCH-04 / SC-4)
- `else` → `help_data::ranked_help_entries(&app.help_search_query)` returns flat `Vec<HelpRow>`, no headers
- Downstream render loop (`===` arm + data arm) is shared and unchanged
- `match_count` computed correctly for both paths (non-header filter in empty path; all rows in ranked path)

---

## Deviations from Plan

None — plan executed exactly as written.

---

## Known Stubs

None. All four fields are scored; alias scoring is live. JSON aliases are populated in Phase 60; until then `search_aliases` is empty on all entries, so alias-tier scores will be 0 — correct behavior, not a stub.

---

## Threat Flags

None. The bounded Levenshtein has the mitigate from T-59-02-DOS: early-exit cap at `max_dist + 1` bounds worst-case work; fuzzy gated by `q.len() >= 2`. No new network/auth/file surface.

---

## Self-Check: PASSED

- hp41-cli/src/help_data.rs: EXISTS, contains `fn levenshtein_bounded`, `fn score_entry`, `fn ranked_help_entries`, provenance comment `help_data.ts`
- hp41-cli/src/ui.rs: EXISTS, contains `ranked_help_entries`, `help_search_query.is_empty()`
- Commits 1a25292 and 6e764c1: both exist on develop
- `cargo test -p hp41-cli --test phase59_help_search`: 7 passed
- `just test`: 0 failures
- No hp41-core files touched; no hp41-gui files touched; test file unchanged
- No new `[dependencies]` in any Cargo.toml
