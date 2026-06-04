//! JSON-loaded HP-41CV function help data (D-25.16 canonical pipeline).
//!
//! `docs/hp41cv-functions.json` is the **single source of truth** for both
//! the CLI `?` help overlay AND the generated function matrix
//! (`docs/hp41cv-function-matrix.md` produced by `scripts/docs-matrix/`).
//! This module embeds the JSON at compile time via `include_str!` and
//! lazy-parses it once via `std::sync::OnceLock` per the project precedent
//! in `hp41-cli/src/programs.rs:19,22`.
//!
//! **Hard-build-blocker semantics (D-25.17):** a malformed JSON file fails
//! the OnceLock init with `.expect("hp41cv-functions.json is malformed")`.
//! This is intentional — canonical data files must not be empty / malformed.
//! The smoke test `phase25_help_data::help_entries_count_meets_130_target`
//! also catches empty-file commits at CI time (RESEARCH Pitfall 7).
//!
//! **D-25.18:** Both the right-panel discoverability listing (`KEY_REF_TABLE`
//! consumers) and the `?` overlay derive from the SAME `help_entries()`
//! call — no parallel hand-curated tables permitted.

use std::sync::OnceLock;

use serde::Deserialize;

/// XROM module descriptor embedded in a [`HelpEntry`] row (C-28.3).
///
/// Present for Math Pac I entries; absent (`None`) for v2.2 built-in entries.
/// `#[serde(default)]` on the `xrom` field of `HelpEntry` means v2.2 JSON
/// (no `xrom` key) parses unchanged — schema extension is additive.
///
/// - `module` — human-readable module name (e.g. `"Math 1"`).
/// - `module_id` — HP-41C hardware XROM module ID (`7` for Math Pac I).
/// - `function_id` — 1-indexed position of this entry in `MATH_1.ops`.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct XromEntry {
    pub module: String,
    pub module_id: u8,
    pub function_id: u16,
}

/// One row in the canonical HP-41CV function table.
///
/// Schema per D-25.16:
/// - `op_variant` — hp41-core `Op::` PascalCase name (e.g. `"Pi"`). For the
///   8 XEQ-by-Name-only conditional tests, this is an `_XEQ`-suffixed alias
///   (e.g. `"XNeY_XEQ"`) that resolves to `Op::Test(TestKind::XNeY)` via
///   `keys::xeq_by_name_local_resolve` and `builtin_card_op`. For v3.x-deferred
///   Module-Pac entries, this is a placeholder ID (no real Op variant exists).
/// - `display_name` — HP-41 mnemonic as shown on the display (e.g. `"PI"`).
/// - `category` — one of the 20 enumerated categories (see CONTEXT D-25.16).
/// - `status` — `"implemented"`, `"deferred-v3"`, or `"na"`.
/// - `phase` — GSD phase ID string (e.g. `"21"`) or `None` for v3.x.
/// - `key_path` — CLI keystroke (e.g. `"f-7"`, `"S"`, `"XEQ \"X<>Y?\""`) or
///   `None` for internal / programmatic-only variants.
/// - `description` — <= 80 chars, suitable for the `?` overlay row.
/// - `divergences` — optional free-form notes about HP-41 hardware divergences.
/// - `xrom` — optional XROM descriptor (present for Math Pac I entries; `None`
///   for v2.2 built-in entries). `#[serde(default)]` ensures backward compat.
// `op_variant`, `status`, `phase`, `divergences` are not read inside src/ —
// only by integration tests under `tests/` (cross-crate, opaque to dead-code
// analysis) and by the `scripts/docs-matrix/` bin (deliberate JSON-schema
// duplication per RESEARCH §"Don't Hand-Roll"). They MUST be deserialized so
// the schema remains the single source of truth for both consumers.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct HelpEntry {
    pub op_variant: String,
    pub display_name: String,
    pub category: String,
    pub status: String,
    pub phase: Option<String>,
    pub key_path: Option<String>,
    pub description: String,
    #[serde(default)]
    pub divergences: Vec<String>,
    /// XROM descriptor (C-28.3). `None` for v2.2 built-ins; `Some(_)` for
    /// Math Pac I entries. `#[serde(default)]` keeps v2.2 JSON parsing clean.
    #[serde(default)]
    pub xrom: Option<XromEntry>,
    /// Invisible match surface for Phase 59 search (D-58.3 / HSDATA-01).
    ///
    /// Alternative spellings, abbreviations, and synonyms that broaden
    /// free-text search recall without appearing in the `?` overlay layout.
    /// Populated in Phase 60 (alias content); empty Vec until then.
    /// Mirrors `search_aliases?: string[]` in hp41-gui/src/help_data.ts.
    #[serde(default)]
    pub search_aliases: Vec<String>,
}

/// Compile-time-embedded canonical data file. The relative path is from this
/// source file (`hp41-cli/src/help_data.rs`) to `docs/hp41cv-functions.json`
/// at the repo root.
const FUNCTIONS_JSON: &str = include_str!("../../docs/hp41cv-functions.json");

static HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

/// Access the parsed help entries (lazily initialized, thread-safe via OnceLock).
///
/// **Panics** on first invocation if `docs/hp41cv-functions.json` is
/// malformed — this is the **intentional** D-25.17 hard-build-blocker
/// behavior. The OnceLock init uses `.expect("hp41cv-functions.json is
/// malformed — fix the JSON")`; subsequent calls return the cached slice.
pub fn help_entries() -> &'static [HelpEntry] {
    HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(FUNCTIONS_JSON)
            .expect("hp41cv-functions.json is malformed — fix the JSON")
    })
}

/// Compile-time-embedded canonical data file for Math Pac I (D-29.1 / D-29.2).
/// The relative path is from `hp41-cli/src/help_data.rs` to
/// `docs/hp41-math1-functions.json` at the repo root.
const MATH1_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-math1-functions.json");

static MATH1_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

/// Access the parsed Math Pac I help entries (lazily initialized, thread-safe via OnceLock).
///
/// **Panics** on first invocation if `docs/hp41-math1-functions.json` is
/// malformed — this is the **intentional** D-25.17 / D-29.2 hard-build-blocker
/// behavior. Subsequent calls return the cached slice.
///
/// Narrow accessor — returns ONLY the Math Pac I pool. Use [`help_entries_all`]
/// for the merged pool (v2.2 + Math Pac I) in UI rendering paths.
pub fn help_entries_math1() -> &'static [HelpEntry] {
    MATH1_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(MATH1_FUNCTIONS_JSON)
            .expect("hp41-math1-functions.json is malformed — fix the JSON")
    })
}

/// Compile-time-embedded canonical data file for Stat 1 Pac (D-34.6 / D-29.2 third-pool extension).
/// The relative path is from `hp41-cli/src/help_data.rs` to
/// `docs/hp41-stat1-functions.json` at the repo root.
const STAT1_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-stat1-functions.json");

static STAT1_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

/// Access the parsed Stat 1 Pac help entries (lazily initialized, thread-safe via OnceLock).
///
/// **Panics** on first invocation if `docs/hp41-stat1-functions.json` is
/// malformed — this is the **intentional** D-25.17 / D-29.2 hard-build-blocker
/// behavior (third-file copy). The per-file panic message `"hp41-stat1-functions.json
/// is malformed — fix the JSON"` routes failures to the correct source-of-truth file.
/// Subsequent calls return the cached slice.
///
/// Narrow accessor — returns ONLY the Stat 1 Pac pool. Use [`help_entries_all`]
/// for the merged pool (v2.2 + Math Pac I + Stat 1 Pac) in UI rendering paths.
/// This narrow accessor exists for the per-file smoke test (`phase34_help_data_stat1.rs`) ONLY.
pub fn help_entries_stat1() -> &'static [HelpEntry] {
    STAT1_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(STAT1_FUNCTIONS_JSON)
            .expect("hp41-stat1-functions.json is malformed — fix the JSON")
    })
}

/// Compile-time-embedded canonical data file for Time Pac (D-39.12 / D-29.2 fourth-pool extension).
/// The relative path is from `hp41-cli/src/help_data.rs` to
/// `docs/hp41-time-functions.json` at the repo root.
const TIME_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-time-functions.json");

static TIME_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

/// Access the parsed Time Pac help entries (lazily initialized, thread-safe via OnceLock).
///
/// **Panics** on first invocation if `docs/hp41-time-functions.json` is
/// malformed — this is the **intentional** D-25.17 / D-29.2 hard-build-blocker
/// behavior (fourth-file copy). The per-file panic message `"hp41-time-functions.json
/// is malformed — fix the JSON"` routes failures to the correct source-of-truth file.
/// Subsequent calls return the cached slice.
///
/// Narrow accessor — returns ONLY the Time Pac pool. Use [`help_entries_all`]
/// for the merged pool (v2.2 + Math Pac I + Stat 1 Pac + Time Pac) in UI rendering paths.
/// This narrow accessor exists for per-pool surgical tests ONLY.
pub fn help_entries_time() -> &'static [HelpEntry] {
    TIME_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(TIME_FUNCTIONS_JSON)
            .expect("hp41-time-functions.json is malformed — fix the JSON")
    })
}

/// Compile-time-embedded canonical data file for Advantage Pac (D-44.1 / D-29.2 fifth-pool extension).
/// The relative path is from `hp41-cli/src/help_data.rs` to
/// `docs/hp41-advantage-functions.json` at the repo root.
const ADV_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-advantage-functions.json");

static ADV_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

/// Access the parsed Advantage Pac help entries (lazily initialized, thread-safe via OnceLock).
///
/// **Panics** on first invocation if `docs/hp41-advantage-functions.json` is
/// malformed — this is the **intentional** D-25.17 / D-29.2 hard-build-blocker
/// behavior (fifth-file copy). The per-file panic message `"hp41-advantage-functions.json
/// is malformed — fix the JSON"` routes failures to the correct source-of-truth file.
/// Subsequent calls return the cached slice.
///
/// Narrow accessor — returns ONLY the Advantage Pac pool. Use [`help_entries_all`]
/// for the merged pool (v2.2 + Math Pac I + Stat 1 Pac + Time Pac + Advantage Pac)
/// in UI rendering paths. This narrow accessor exists for per-pool surgical tests ONLY.
pub fn help_entries_adv() -> &'static [HelpEntry] {
    ADV_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(ADV_FUNCTIONS_JSON)
            .expect("hp41-advantage-functions.json is malformed — fix the JSON")
    })
}

/// Compile-time-embedded canonical data file for X-MEM built-ins (D-52.1 sixth-pool extension).
/// The relative path is from `hp41-cli/src/help_data.rs` to
/// `docs/hp41-xmem-functions.json` at the repo root.
const XMEM_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-xmem-functions.json");

static XMEM_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

/// Access the parsed X-MEM built-in help entries (lazily initialized, thread-safe via OnceLock).
///
/// **Panics** on first invocation if `docs/hp41-xmem-functions.json` is
/// malformed — this is the **intentional** D-25.17 / D-29.2 hard-build-blocker
/// behavior (sixth-file copy). The per-file panic message routes failures to
/// the correct source-of-truth file. Subsequent calls return the cached slice.
///
/// Narrow accessor — returns ONLY the X-MEM pool. Use [`help_entries_all`]
/// for the merged pool in UI rendering paths. This narrow accessor exists
/// for per-pool surgical tests (`phase52_help_data_xmem.rs`) ONLY.
pub fn help_entries_xmem() -> &'static [HelpEntry] {
    XMEM_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(XMEM_FUNCTIONS_JSON)
            .expect("hp41-xmem-functions.json is malformed — fix the JSON")
    })
}

/// Merged accessor: chains all six JSON pools in order per D-52.1.
///
/// Fixed insertion order: v2.2 built-ins → Math Pac I → Stat 1 Pac → Time Pac
/// → Advantage Pac → X-MEM built-ins.
///
/// This is the **single source of truth** for:
///
/// - The `?` help overlay (`ui::render_help_overlay` via `help_overlay_rows`)
/// - The right-panel discoverability listing (`keys::key_ref_entries`)
/// - The `function_matrix_parity.rs` full-pool sweep
///
/// **D-34.6 / D-39.12 / D-44.1 / D-52.1 ordering rationale:** the chain order is
/// the natural render order for the `?` overlay sections. The order is fixed by
/// convention, NOT alphabetical, to preserve users' learned mental model.
///
/// The narrow accessors [`help_entries`], [`help_entries_math1`], [`help_entries_stat1`],
/// [`help_entries_time`], [`help_entries_adv`], and [`help_entries_xmem`] are retained for
/// per-pool surgical tests (130-target, 45-target, 26-target, 35-target, 114-target, 8-target
/// smoke tests) and MUST NOT be removed.
pub fn help_entries_all() -> impl Iterator<Item = &'static HelpEntry> {
    help_entries()
        .iter()
        .chain(help_entries_math1().iter())
        .chain(help_entries_stat1().iter())
        .chain(help_entries_time().iter())
        .chain(help_entries_adv().iter())
        .chain(help_entries_xmem().iter()) // Phase 52 (D-52.1)
}

/// Render a list of `(key, op, desc)` 3-tuples in the legacy `HELP_DATA`
/// shape with category-header rows interleaved. Used by
/// `ui::render_help_overlay` so the existing table-rendering code keeps
/// working over the new JSON-derived data source.
///
/// Entries are sorted by category (in their first-appearance order in the
/// JSON) so the overlay groups them naturally; within a category, entries
/// keep the JSON's declared order.
///
/// **Static-lifetime trade-off:** the legacy `HELP_DATA` const used
/// `&'static str` directly. The JSON-loaded entries are `String`-owned, so
/// the consumer (`ui::render_help_overlay`) now receives a borrowed
/// `&HelpEntry` slice via [`help_entries`] and reads `&str` slices directly
/// from those strings. Callers that want the legacy 3-tuple shape should use
/// the [`help_overlay_rows`] helper which returns owned `String`s grouped by
/// category with synthetic `=== {category} ===` header rows interleaved.
pub fn help_overlay_rows() -> Vec<HelpRow> {
    // D-29.2: migrate to merged accessor so Math Pac I categories appear in
    // the `?` overlay alongside the v2.2 built-in categories (CLI-04).
    // CLI-parity (260603-scc): only implemented entries — drop deferred-v3, which
    // cannot be run, mirroring the GUI's allFunctionsEntries() filter.
    let entries: Vec<&HelpEntry> = help_entries_all()
        .filter(|e| e.status == "implemented")
        .collect();
    let mut categories: Vec<&str> = Vec::new();
    for entry in &entries {
        if !categories.iter().any(|c| *c == entry.category) {
            categories.push(&entry.category);
        }
    }

    let mut rows: Vec<HelpRow> = Vec::with_capacity(entries.len() + categories.len());
    for cat in categories {
        rows.push(HelpRow {
            key: String::new(),
            op: String::new(),
            desc: format!("=== {cat} ==="),
        });
        for entry in entries.iter().filter(|e| e.category == cat) {
            // CLI-parity (260603-scc): keyless built-ins (key_path:null, no XROM)
            // are run via XEQ-by-name — show `XEQ "NAME"` in the key column so the
            // `?` overlay is self-documenting about how to run them (the CLI analog
            // of the GUI's tap-to-run). XROM module entries already carry
            // key_path = `XEQ "NAME"` from the JSON, so they are unaffected.
            let key = entry.key_path.clone().unwrap_or_else(|| {
                if entry.xrom.is_none() {
                    format!("XEQ \"{}\"", entry.display_name)
                } else {
                    String::new()
                }
            });
            rows.push(HelpRow {
                key,
                op: entry.display_name.clone(),
                desc: entry.description.clone(),
            });
        }
    }
    rows
}

// ── Phase 59 scorer ──────────────────────────────────────────────────────────
//
// Mirrored scorer — keep byte-equivalent in behavior with
// hp41-gui/src/help_data.ts per Phase 59 (CLI<->GUI parity).
// Parity fixture lands in Phase 61.
//
// Tier constants are tunable — see 59-RESEARCH.md §Open Questions Q1.

const SCORE_EXACT_NAME: u8 = 40;
const SCORE_PREFIX_NAME: u8 = 32;
const SCORE_SUBSTR_NAME: u8 = 24;
const SCORE_FUZZY_NAME: u8 = 8;
const SCORE_EXACT_ALIAS: u8 = 35;
const SCORE_PREFIX_ALIAS: u8 = 28;
const SCORE_SUBSTR_ALIAS: u8 = 21;
const SCORE_FUZZY_ALIAS: u8 = 7;
const SCORE_EXACT_DESC: u8 = 30;
const SCORE_PREFIX_DESC: u8 = 24;
const SCORE_SUBSTR_DESC: u8 = 18;
const SCORE_FUZZY_DESC: u8 = 6;
const SCORE_EXACT_CAT: u8 = 20;
const SCORE_PREFIX_CAT: u8 = 16;
const SCORE_SUBSTR_CAT: u8 = 12;
const SCORE_FUZZY_CAT: u8 = 4;

/// Bounded Levenshtein edit distance between `a` and `b`.
///
/// Returns the true edit distance, capped at `max_dist + 1`. If the true
/// distance would exceed `max_dist`, returns `max_dist + 1` immediately
/// (early-exit to avoid O(n·m) work for clearly non-fuzzy pairs).
///
/// Uses `chars().collect::<Vec<char>>()` for Unicode-correct handling of
/// multi-byte characters such as German umlauts (ä, ö, ü).
fn levenshtein_bounded(a: &str, b: &str, max_dist: usize) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let n = a.len();
    let m = b.len();
    if n.abs_diff(m) > max_dist {
        return max_dist + 1;
    }
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr = vec![0usize; m + 1];
    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (curr[j - 1] + 1).min(prev[j] + 1).min(prev[j - 1] + cost);
        }
        // Early-exit: if no value in `curr` can improve on max_dist, stop.
        if curr.iter().min().copied().unwrap_or(usize::MAX) > max_dist {
            return max_dist + 1;
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}

/// Compute the tier score for a single field against the query `q`.
///
/// `q` must already be lowercased by the caller (`score_entry` does this once).
/// Tier order (highest wins): exact > word-prefix > substring > fuzzy.
/// Fuzzy is only attempted when `q.len() >= 2` to prevent false-positives on
/// single-character queries (P-HS-02).
fn tier_score(field: &str, q: &str, exact: u8, prefix: u8, substr: u8, fuzzy: u8) -> u8 {
    let f = field.to_lowercase();
    if f == q {
        return exact;
    }
    if f.starts_with(q) || f.split_whitespace().any(|w| w.starts_with(q)) {
        return prefix;
    }
    if f.contains(q) {
        return substr;
    }
    if q.len() >= 2 {
        let max_dist = (q.len() / 4).max(1);
        // Short fields: compare whole field; long fields: compare each word.
        let min_dist = if f.len() <= 20 {
            levenshtein_bounded(&f, q, max_dist)
        } else {
            f.split_whitespace()
                .map(|w| levenshtein_bounded(w, q, max_dist))
                .min()
                .unwrap_or(usize::MAX)
        };
        if min_dist <= max_dist {
            return fuzzy;
        }
    }
    0
}

/// Score a `HelpEntry` against a pre-lowercased query `q`.
///
/// Returns the best tier score across all four searched fields:
/// `display_name`, `description`, `category`, and `search_aliases`.
/// A score of `0` means no field matched.
///
/// The caller must lowercase the query once before passing it here:
/// ```text
/// let q = query.to_lowercase();
/// let s = score_entry(entry, &q);
/// ```
pub fn score_entry(entry: &HelpEntry, q: &str) -> u8 {
    let name_score = tier_score(
        &entry.display_name,
        q,
        SCORE_EXACT_NAME,
        SCORE_PREFIX_NAME,
        SCORE_SUBSTR_NAME,
        SCORE_FUZZY_NAME,
    );
    let desc_score = tier_score(
        &entry.description,
        q,
        SCORE_EXACT_DESC,
        SCORE_PREFIX_DESC,
        SCORE_SUBSTR_DESC,
        SCORE_FUZZY_DESC,
    );
    let cat_score = tier_score(
        &entry.category,
        q,
        SCORE_EXACT_CAT,
        SCORE_PREFIX_CAT,
        SCORE_SUBSTR_CAT,
        SCORE_FUZZY_CAT,
    );
    let alias_score = entry
        .search_aliases
        .iter()
        .map(|a| {
            tier_score(
                a,
                q,
                SCORE_EXACT_ALIAS,
                SCORE_PREFIX_ALIAS,
                SCORE_SUBSTR_ALIAS,
                SCORE_FUZZY_ALIAS,
            )
        })
        .max()
        .unwrap_or(0);
    name_score.max(desc_score).max(cat_score).max(alias_score)
}

/// Return a flat, relevance-ranked list of help rows for a non-empty query.
///
/// Scores every `status == "implemented"` entry from all six JSON pools via
/// [`score_entry`], discards zero-score entries, then stable-sorts by
/// `(score DESC, display_name ASC)`. Returns owned [`HelpRow`] values with no
/// category-header rows — the caller can render the result directly with the
/// same table loop used by the empty-query grouped path.
///
/// # Panics
///
/// Never in production. A `debug_assert!` fires in debug builds if `query` is
/// empty — callers should use the [`filter_help_rows`] / [`help_overlay_rows`]
/// path for the empty-query case.
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
    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| a.1.display_name.cmp(&b.1.display_name))
    });
    scored
        .into_iter()
        .map(|(_, e)| {
            // Reuse the same key-display logic as help_overlay_rows.
            let key = e.key_path.clone().unwrap_or_else(|| {
                if e.xrom.is_none() {
                    format!("XEQ \"{}\"", e.display_name)
                } else {
                    String::new()
                }
            });
            HelpRow {
                key,
                op: e.display_name.clone(),
                desc: e.description.clone(),
            }
        })
        .collect()
}

// ── End Phase 59 scorer ───────────────────────────────────────────────────────

/// Filter the help-overlay rows by a case-insensitive substring match against
/// the row's key, op, or description. Category headers (`=== <name> ===`) are
/// preserved only when at least one child row in that category matches the
/// query, so the filtered output is never a category header with no entries.
///
/// An empty query returns every row unfiltered, mirroring the closed-search
/// state in the TUI. This is the data-side counterpart of the GUI's
/// `?`-overlay search input (v3.0 Phase 31's `HelpOverlay.tsx`); putting the
/// filter here keeps `ui::render_help_overlay` thin and makes the filter
/// unit-testable without spinning up a `Frame`.
pub fn filter_help_rows<'a>(rows: &'a [HelpRow], query: &str) -> Vec<&'a HelpRow> {
    if query.is_empty() {
        return rows.iter().collect();
    }
    let q = query.to_lowercase();
    let matches = |row: &HelpRow| {
        row.key.to_lowercase().contains(&q)
            || row.op.to_lowercase().contains(&q)
            || row.desc.to_lowercase().contains(&q)
    };
    let mut out: Vec<&HelpRow> = Vec::new();
    let mut pending_header: Option<&HelpRow> = None;
    for row in rows {
        let is_header = row.desc.starts_with("===");
        if is_header {
            // Buffer the header — only emit it when (and if) a child matches.
            pending_header = Some(row);
        } else if matches(row) {
            if let Some(h) = pending_header.take() {
                out.push(h);
            }
            out.push(row);
        }
    }
    out
}

/// One row of the help overlay table, produced by [`help_overlay_rows`].
/// Category headers carry `desc == "=== <name> ==="` with empty `key`/`op`.
#[derive(Debug, Clone)]
pub struct HelpRow {
    pub key: String,
    pub op: String,
    pub desc: String,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn fixture() -> Vec<HelpRow> {
        vec![
            HelpRow {
                key: String::new(),
                op: String::new(),
                desc: "=== Arithmetic ===".into(),
            },
            HelpRow {
                key: "+".into(),
                op: "+".into(),
                desc: "Add: X <- Y + X, drop stack".into(),
            },
            HelpRow {
                key: "*".into(),
                op: "*".into(),
                desc: "Multiply: X <- Y * X, drop stack".into(),
            },
            HelpRow {
                key: String::new(),
                op: String::new(),
                desc: "=== Math ===".into(),
            },
            HelpRow {
                key: "PI".into(),
                op: "PI".into(),
                desc: "Push pi onto X".into(),
            },
            HelpRow {
                key: String::new(),
                op: String::new(),
                desc: "=== Empty ===".into(),
            },
        ]
    }

    #[test]
    fn empty_query_returns_all_rows() {
        let rows = fixture();
        let filtered = filter_help_rows(&rows, "");
        assert_eq!(filtered.len(), rows.len());
    }

    #[test]
    fn substring_match_on_desc_keeps_category_header() {
        let rows = fixture();
        let filtered = filter_help_rows(&rows, "stack");
        // 2 matching rows + 1 header = 3.
        assert_eq!(filtered.len(), 3);
        assert!(filtered[0].desc.contains("Arithmetic"));
        assert_eq!(filtered[1].op, "+");
        assert_eq!(filtered[2].op, "*");
    }

    #[test]
    fn substring_match_is_case_insensitive() {
        let rows = fixture();
        let lower = filter_help_rows(&rows, "multiply");
        let upper = filter_help_rows(&rows, "MULTIPLY");
        let mixed = filter_help_rows(&rows, "MultiPly");
        assert_eq!(lower.len(), upper.len());
        assert_eq!(lower.len(), mixed.len());
        assert_eq!(lower.len(), 2); // 1 header + 1 row
    }

    #[test]
    fn substring_match_on_op_field() {
        let rows = fixture();
        let filtered = filter_help_rows(&rows, "PI");
        assert_eq!(filtered.len(), 2); // Math header + PI row
        assert!(filtered[0].desc.contains("Math"));
        assert_eq!(filtered[1].op, "PI");
    }

    #[test]
    fn category_header_only_appears_when_child_matches() {
        // "Empty" category has no children, so its header must never appear
        // in filter output regardless of query. The "Math" header should not
        // appear in this query either (no "stack" match in Math category).
        let rows = fixture();
        let filtered = filter_help_rows(&rows, "stack");
        assert!(
            !filtered
                .iter()
                .any(|r| r.desc.contains("Math") || r.desc.contains("Empty")),
            "category headers without matching children must be filtered out"
        );
    }

    #[test]
    fn no_match_yields_empty_output() {
        let rows = fixture();
        let filtered = filter_help_rows(&rows, "this-string-matches-nothing-xyzzy");
        assert!(filtered.is_empty());
    }

    /// D-58.2 backward-compat guarantee: a JSON object that omits `search_aliases`
    /// (i.e. all existing JSON pool entries) deserializes cleanly into a `HelpEntry`
    /// whose `search_aliases` is an empty Vec — no missing-field error.
    #[test]
    fn search_aliases_defaults_to_empty_vec_when_field_absent() {
        let json = r#"{
            "op_variant": "Pi",
            "display_name": "PI",
            "category": "Math",
            "status": "implemented",
            "phase": "21",
            "key_path": "f-7",
            "description": "Push pi (3.14159265358979) onto X"
        }"#;
        let entry: HelpEntry = serde_json::from_str(json).unwrap();
        assert!(
            entry.search_aliases.is_empty(),
            "search_aliases must default to an empty Vec when the JSON key is absent"
        );
    }
}
