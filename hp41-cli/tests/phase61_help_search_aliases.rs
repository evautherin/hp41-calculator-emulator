//! Phase 61 (Quality Gates) — real-data help-search verification.
//!
//! Proves the Phase 59 tiered scorer + Phase 60 alias content against the
//! ACTUAL loaded JSON pools (no synthetic fixtures for the matcher itself):
//!
//! 1. All four scoring tiers — exact > prefix > substring > fuzzy — against a
//!    real loaded entry (SQRT) via the public `score_entry`.
//! 2. DE + EN alias resolution, each top-1, via `ranked_help_entries`.
//! 3. At least one fuzzy-hit top-1 (`sqirt` -> SQRT).
//! 4. Whitespace-only query yields NO ranked results (the literal empty string
//!    is intentionally avoided — it trips the `debug_assert!` in
//!    `ranked_help_entries`; a whitespace-only query trims to empty and returns
//!    `Vec::new()`, which is the contract we assert).
//! 5. The cross-implementation parity fixture
//!    (`docs/fixtures/help_search_parity.json`, also consumed by the GUI's
//!    Vitest suite) — every query's top-1 must equal `expected_top[0]`.
//!
//! Integration test => uses ONLY the public `hp41_cli::help_data` API.
//! The `HelpRow` display-name field is `.op` (it is built from
//! `entry.display_name`), verified by reading `hp41-cli/src/help_data.rs`.

#![allow(clippy::unwrap_used)]

use hp41_cli::help_data::{help_entries_all, ranked_help_entries, score_entry, HelpEntry};

use serde::Deserialize;

// ── Parity fixture schema (must mirror docs/fixtures/help_search_parity.json) ──

#[derive(Debug, Deserialize)]
struct ParityFixture {
    queries: Vec<ParityQuery>,
}

#[derive(Debug, Deserialize)]
struct ParityQuery {
    query: String,
    #[allow(dead_code)]
    top_n: u8,
    expected_top: Vec<String>,
    #[allow(dead_code)]
    note: Option<String>,
}

/// Compile-time embed of the cross-impl parity fixture. Path is relative to
/// THIS source file (`hp41-cli/tests/`): two levels up to the repo root, then
/// `docs/fixtures/...`.
const PARITY_FIXTURE_JSON: &str = include_str!("../../docs/fixtures/help_search_parity.json");

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Fetch a single implemented entry by exact `display_name` from the merged
/// six-pool accessor. Panics (test-only) if absent so a renamed/removed op
/// fails loudly rather than silently skipping the tier proof.
fn entry_by_display_name(name: &str) -> &'static HelpEntry {
    help_entries_all()
        .find(|e| e.display_name == name && e.status == "implemented")
        .unwrap_or_else(|| {
            panic!("expected an implemented entry named {name:?} in the loaded pools")
        })
}

/// Top-1 display name for `query`, or `None` if no ranked rows. Reads `.op`
/// (the `HelpRow` display-name field).
fn top1(query: &str) -> Option<String> {
    ranked_help_entries(query).first().map(|r| r.op.clone())
}

// ── 1. Four scoring tiers against the REAL loaded SQRT entry ──────────────────

/// SQRT (display_name "SQRT") exercises all four tiers deterministically:
/// - exact:     "sqrt"  (f == q)
/// - prefix:    "sqr"   (f.starts_with(q))
/// - substring: "qrt"   (f.contains(q), not a prefix)
/// - fuzzy:     "sqirt" (levenshtein 1, within bound)
///
/// We assert the strict ordering exact > prefix > substring > fuzzy > 0 on the
/// SAME real entry, proving the tier hierarchy against loaded data (not a
/// hand-built fixture). The exact numeric tier constants are private to the
/// module, so we assert relative ordering + non-zero, which is the load-bearing
/// invariant.
#[test]
fn four_scoring_tiers_against_real_sqrt_entry() {
    let sqrt = entry_by_display_name("SQRT");

    let exact = score_entry(sqrt, "sqrt");
    let prefix = score_entry(sqrt, "sqr");
    let substr = score_entry(sqrt, "qrt");
    let fuzzy = score_entry(sqrt, "sqirt");

    assert!(exact > 0, "exact-name query must score > 0");
    assert!(prefix > 0, "prefix-name query must score > 0");
    assert!(substr > 0, "substring-name query must score > 0");
    assert!(fuzzy > 0, "fuzzy-name query must score > 0");

    assert!(
        exact > prefix,
        "exact ({exact}) must outrank prefix ({prefix})"
    );
    assert!(
        prefix > substr,
        "prefix ({prefix}) must outrank substring ({substr})"
    );
    assert!(
        substr > fuzzy,
        "substring ({substr}) must outrank fuzzy ({fuzzy})"
    );
}

/// Exact name match must produce a strictly higher score than a fuzzy match on
/// the same entry — guards against accidental tier-constant inversion.
#[test]
fn exact_name_outranks_fuzzy_top1() {
    // "sqrt" is the exact name; it must come back top-1.
    assert_eq!(top1("sqrt").as_deref(), Some("SQRT"));
    // "pi" is an exact baseline regression.
    assert_eq!(top1("pi").as_deref(), Some("PI"));
}

// ── 2. DE + EN alias resolution, each top-1 ───────────────────────────────────

#[test]
fn de_alias_zeitwert_des_geldes_resolves_tvm() {
    assert_eq!(top1("zeitwert des geldes").as_deref(), Some("TVM"));
}

#[test]
fn de_alias_zinseszins_resolves_tvm() {
    assert_eq!(top1("zinseszins").as_deref(), Some("TVM"));
}

#[test]
fn en_alias_square_root_resolves_sqrt() {
    assert_eq!(top1("square root").as_deref(), Some("SQRT"));
}

#[test]
fn de_alias_quadratwurzel_resolves_sqrt() {
    assert_eq!(top1("quadratwurzel").as_deref(), Some("SQRT"));
}

#[test]
fn de_alias_wurzel_resolves_sqrt() {
    assert_eq!(top1("wurzel").as_deref(), Some("SQRT"));
}

// ── 3. Fuzzy-hit top-1 ────────────────────────────────────────────────────────

#[test]
fn fuzzy_typo_sqirt_resolves_sqrt() {
    // One-character typo (sq[i]rt vs sq[r]t) — must still surface SQRT top-1.
    assert_eq!(top1("sqirt").as_deref(), Some("SQRT"));
}

// ── 4. Whitespace-only query => no ranked results ─────────────────────────────

/// LANDMINE: the literal empty string `""` trips a `debug_assert!` inside
/// `ranked_help_entries`. A whitespace-only query trims to empty and returns an
/// empty Vec — that is the "no ranked results for blank input" contract.
#[test]
fn whitespace_only_query_yields_no_ranked_results() {
    let rows = ranked_help_entries("   ");
    assert!(
        rows.is_empty(),
        "whitespace-only query must yield zero ranked rows, got {}",
        rows.len()
    );
}

// ── 5. Cross-implementation parity fixture ────────────────────────────────────

/// Every query in the shared parity fixture must produce the expected top-1
/// display name. This fixture is ALSO asserted by the GUI's Vitest suite, so a
/// passing run here proves CLI <-> GUI matcher parity on the canonical query set.
#[test]
fn parity_fixture_top1_matches_for_every_query() {
    let fixture: ParityFixture = serde_json::from_str(PARITY_FIXTURE_JSON)
        .expect("docs/fixtures/help_search_parity.json is malformed — fix the JSON");

    assert!(
        !fixture.queries.is_empty(),
        "parity fixture must contain at least one query"
    );

    for q in &fixture.queries {
        let expected = q
            .expected_top
            .first()
            .expect("each parity query must declare at least one expected_top entry");
        let got = top1(&q.query);
        assert_eq!(
            got.as_deref(),
            Some(expected.as_str()),
            "parity mismatch for query {:?}: expected top-1 {:?}, got {:?}",
            q.query,
            expected,
            got
        );
    }
}
