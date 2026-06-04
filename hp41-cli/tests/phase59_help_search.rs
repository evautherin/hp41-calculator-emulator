//! Phase 59 Wave 0 — RED scorer contract; implementation lands in plan 59-02.
//!
//! These tests describe the full scoring API before the scorer is written.
//! They will fail to compile until `score_entry` and `ranked_help_entries` are
//! added to `hp41-cli/src/help_data.rs` in plan 59-02. That is the intended
//! Wave-0 RED state — not a defect.
//!
//! Tier constants asserted here (from 59-RESEARCH.md §Tiered Scoring):
//!   name:  exact 40 > prefix 32 > substr 24 > fuzzy 8
//!   alias: exact 35 > prefix 28 > substr 21 > fuzzy 7
//!   desc:  exact 30 > prefix 24 > substr 18 > fuzzy 6
//!   cat:   exact 20 > prefix 16 > substr 12 > fuzzy 4
//!
//! Fuzzy threshold: max(1, query.len() / 4); fuzzy only when query.len() >= 2.
//! score_entry expects a PRE-LOWERCASED query `q` (caller lowercases once).
//! ranked_help_entries returns flat Vec<HelpRow>, score DESC then display_name ASC.

#![allow(clippy::unwrap_used)]

use hp41_cli::help_data::{filter_help_rows, ranked_help_entries, score_entry, HelpEntry, HelpRow};

// ── Fixture builder ───────────────────────────────────────────────────────────

/// Build a synthetic `HelpEntry` for scorer unit tests. All fields are
/// populated so `score_entry` can find matches in every field.
/// Uses the same literal pattern as `hp41-cli/src/help_data.rs` `HelpEntry`
/// struct definition (all fields, `#[serde(default)]` absent from test context).
fn make_entry(display_name: &str, description: &str, category: &str, aliases: &[&str]) -> HelpEntry {
    HelpEntry {
        op_variant: display_name.to_string(),
        display_name: display_name.to_string(),
        category: category.to_string(),
        status: "implemented".to_string(),
        phase: None,
        key_path: None,
        description: description.to_string(),
        divergences: vec![],
        xrom: None,
        search_aliases: aliases.iter().map(|s| s.to_string()).collect(),
    }
}

// ── Tier order (HSMATCH-02) ────────────────────────────────────────────────────

/// Verifies the four scoring tiers are strictly ordered:
///   exact (40) > word-prefix (32) > substring (24) > fuzzy (8)
/// on the display_name field, using isolated one-field-match entries.
#[test]
fn tier_order_exact_prefix_substring_fuzzy() {
    // Exact name match: query "tvm" == display_name "tvm" (lowercased)
    let exact_entry = make_entry("tvm", "Time Value of Money", "Finance", &[]);
    let exact_score = score_entry(&exact_entry, "tvm");
    assert_eq!(exact_score, 40, "exact name match must score 40, got {}", exact_score);

    // Word-prefix name match: query "tv" is a prefix of "tvm"
    let prefix_entry = make_entry("tvm", "Time Value of Money", "Finance", &[]);
    let prefix_score = score_entry(&prefix_entry, "tv");
    assert_eq!(prefix_score, 32, "word-prefix name match must score 32, got {}", prefix_score);

    // Substring name match: query "vm" is a substring (not prefix) of "tvm"
    let substr_entry = make_entry("tvm", "Time Value of Money", "Finance", &[]);
    let substr_score = score_entry(&substr_entry, "vm");
    assert_eq!(substr_score, 24, "substring name match must score 24, got {}", substr_score);

    // Fuzzy name match: query "tvm" vs display_name "tvm" is exact; use a 1-typo case:
    // query "tvm" vs display_name "tvn" — 1 edit, threshold max(1, 3/4)=1 → fuzzy match score 8
    let fuzzy_entry = make_entry("tvn", "Time Value of Money", "Finance", &[]);
    let fuzzy_score = score_entry(&fuzzy_entry, "tvm");
    assert_eq!(fuzzy_score, 8, "fuzzy name match (1 typo) must score 8, got {}", fuzzy_score);

    // Tier order is strictly descending
    assert!(exact_score > prefix_score, "exact({}) must beat prefix({})", exact_score, prefix_score);
    assert!(prefix_score > substr_score, "prefix({}) must beat substr({})", prefix_score, substr_score);
    assert!(substr_score > fuzzy_score, "substr({}) must beat fuzzy({})", substr_score, fuzzy_score);
    assert!(fuzzy_score > 0, "fuzzy match must be non-zero");
}

// ── Fuzzy typo: Zineszins → TVM (HSMATCH-03, HSUX-02) ─────────────────────────

/// "Zineszins" (1-character typo of "Zinseszins") must resolve the TVM entry
/// via alias fuzzy match (threshold = max(1, 9/4) = 2 edits).
/// A control entry without the alias must score 0.
#[test]
fn fuzzy_typo_zineszins_resolves_tvm() {
    // TVM with alias "Zinseszins" (German: compound interest)
    let tvm = make_entry("TVM", "Time Value of Money solver", "Finance", &["Zinseszins", "compound interest"]);
    // "zineszins" is 1 edit from "zinseszins" (dropped the second 's' after 'z')
    // threshold = max(1, 9/4) = 2, so distance 1 is within threshold
    let score = score_entry(&tvm, "zineszins");
    assert!(
        score > 0,
        "TVM with alias 'Zinseszins' must score > 0 for query 'zineszins' (1-typo fuzzy); got {}",
        score
    );

    // Control: entry without the alias must score 0
    let control = make_entry("SIN", "Sine of X", "Trigonometry", &[]);
    let control_score = score_entry(&control, "zineszins");
    assert_eq!(
        control_score, 0,
        "SIN without alias must score 0 for 'zineszins', got {}",
        control_score
    );
}

// ── Fuzzy: Wurzel → SQRT (HSUX-02) ────────────────────────────────────────────

/// "wurzel" is an exact alias match (alias "Wurzel", lowercased = "wurzel").
/// Score must be 35 (alias-exact band) and SQRT must rank highest in a small pool.
#[test]
fn fuzzy_wurzel_resolves_sqrt() {
    let sqrt = make_entry("SQRT", "Square root of X", "Math", &["Wurzel"]);
    let score = score_entry(&sqrt, "wurzel");
    assert_eq!(
        score, 35,
        "SQRT with alias 'Wurzel' must score 35 for exact alias query 'wurzel', got {}",
        score
    );

    // In a pool, SQRT must be the max-scoring entry for "wurzel"
    let pool = vec![
        make_entry("SIN", "Sine of X", "Trigonometry", &[]),
        make_entry("COS", "Cosine of X", "Trigonometry", &[]),
        sqrt.clone(),
        make_entry("TAN", "Tangent of X", "Trigonometry", &[]),
    ];
    let max_score = pool.iter().map(|e| score_entry(e, "wurzel")).max().unwrap_or(0);
    assert_eq!(
        score, max_score,
        "SQRT must be the highest-scoring entry for 'wurzel' in the pool; pool max = {}, sqrt score = {}",
        max_score, score
    );
}

// ── DE + EN alias resolution (HSMATCH-05, HSMATCH-01) ─────────────────────────

/// Both German and English aliases on TVM must resolve (score > 0).
/// An alias-less entry must score 0 for both queries.
#[test]
fn alias_de_en_both_resolve() {
    let tvm = make_entry(
        "TVM",
        "Time Value of Money solver",
        "Finance",
        &["Zinseszins", "compound interest"],
    );

    let de_score = score_entry(&tvm, "zinseszins");
    assert!(
        de_score > 0,
        "TVM with DE alias 'Zinseszins' must score > 0 for 'zinseszins', got {}",
        de_score
    );

    let en_score = score_entry(&tvm, "compound interest");
    assert!(
        en_score > 0,
        "TVM with EN alias 'compound interest' must score > 0 for 'compound interest', got {}",
        en_score
    );

    // Entry without any alias scores 0 for both queries
    let aliasless = make_entry("SIN", "Sine of X", "Trigonometry", &[]);
    assert_eq!(
        score_entry(&aliasless, "zinseszins"),
        0,
        "alias-less SIN must score 0 for 'zinseszins'"
    );
    assert_eq!(
        score_entry(&aliasless, "compound interest"),
        0,
        "alias-less SIN must score 0 for 'compound interest'"
    );
}

// ── Empty-query regression guard (HSMATCH-04) ─────────────────────────────────

/// `filter_help_rows` with an empty query must return all rows unmodified
/// (regression guard: plan 59-02 must not remove the empty-query guard).
/// Mirrors the existing `empty_query_returns_all_rows` test in the inline
/// `mod tests` block of `help_data.rs`.
#[test]
fn empty_query_returns_all_grouped() {
    let rows = vec![
        HelpRow { key: "f-1".to_string(), op: "SIN".to_string(), desc: "Sine of X".to_string() },
        HelpRow {
            key: "".to_string(),
            op: "".to_string(),
            desc: "=== Trigonometry ===".to_string(),
        },
        HelpRow { key: "f-2".to_string(), op: "COS".to_string(), desc: "Cosine of X".to_string() },
        HelpRow { key: "g-7".to_string(), op: "SQRT".to_string(), desc: "Square root".to_string() },
    ];

    let filtered = filter_help_rows(&rows, "");
    assert_eq!(
        filtered.len(),
        rows.len(),
        "empty query must return all {} rows unfiltered; got {}",
        rows.len(),
        filtered.len()
    );

    // Order must be preserved (first row of output matches first row of input)
    assert_eq!(
        filtered[0].op, rows[0].op,
        "row order must be preserved for empty query"
    );
}

// ── All four fields scored (HSMATCH-01) ──────────────────────────────────────

/// Verifies that the scorer searches all four fields:
/// display_name, description, category, and search_aliases.
/// One entry per field — each must score > 0 when the query matches only that field.
#[test]
fn score_over_all_four_fields() {
    // Match only via display_name (query == display_name, nothing else matches)
    let name_entry = make_entry("SIN", "No-match description", "No-match-cat", &[]);
    assert!(
        score_entry(&name_entry, "sin") > 0,
        "entry matching only via display_name 'SIN' must score > 0 for query 'sin'"
    );

    // Match only via description (query is a word in the description, not in name/cat/alias)
    let desc_entry = make_entry("NOOP", "trigonometry function", "No-match-cat", &[]);
    assert!(
        score_entry(&desc_entry, "trigonometry") > 0,
        "entry matching only via description 'trigonometry function' must score > 0"
    );

    // Match only via category (query == category, nothing else matches)
    let cat_entry = make_entry("NOOP2", "no match description here", "Finance", &[]);
    assert!(
        score_entry(&cat_entry, "finance") > 0,
        "entry matching only via category 'Finance' must score > 0 for query 'finance'"
    );

    // Match only via alias (query == alias, nothing else matches)
    let alias_entry = make_entry("NOOP3", "no match description here", "No-match-cat", &["Zinseszins"]);
    assert!(
        score_entry(&alias_entry, "zinseszins") > 0,
        "entry matching only via alias 'Zinseszins' must score > 0 for query 'zinseszins'"
    );

    // Sanity: entry matching nothing scores 0
    let nomatch_entry = make_entry("NOOP4", "no match here", "Other", &[]);
    assert_eq!(
        score_entry(&nomatch_entry, "zinseszins"),
        0,
        "entry with no matching field must score 0"
    );
}

// ── ranked_help_entries output ordering ──────────────────────────────────────

/// `ranked_help_entries` returns a flat Vec<HelpRow> sorted by score DESC,
/// then display_name ASC. Zero-score entries are excluded.
/// This test uses a controlled query against the real JSON pools so it is
/// integration-level (not purely synthetic). The key assertion is ordering.
#[test]
fn ranked_entries_sorted_desc_no_headers() {
    // Use the real loaded pools via ranked_help_entries.
    // Query "sin" should return at least one entry; none should be a category header.
    let results = ranked_help_entries("sin");
    assert!(
        !results.is_empty(),
        "ranked_help_entries('sin') must return at least one result"
    );
    // No category headers in ranked output (desc must not start with "===")
    for row in &results {
        assert!(
            !row.desc.starts_with("==="),
            "ranked output must not contain category headers; found: {}",
            row.desc
        );
    }
    // First result must be 'SIN' itself (exact name match → score 40, highest)
    assert_eq!(
        results[0].op, "SIN",
        "first result for query 'sin' must be 'SIN' (exact name match); got '{}'",
        results[0].op
    );
}
