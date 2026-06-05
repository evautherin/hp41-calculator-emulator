//! `help-aliases` — Dev-only alias authoring generator for HP-41 help JSON pools.
//!
//! Usage:
//!   help-aliases <pool.json>                      generate aliases via `claude -p`
//!   help-aliases <pool.json> --apply-cache <f>    re-apply aliases from a cached map (no LLM)
//!
//! Reads the given pool file, generates (or loads) DE+EN search aliases for each
//! `status:"implemented"` entry that has no aliases yet, and writes them back via a
//! byte-preserving text splice (`pool::splice_aliases`) so the only change to the file
//! is the added `search_aliases` arrays (D-60.4 minimal-diff writeback).
//!
//! `--apply-cache` exists so a completed (and paid-for) generation run can be re-applied
//! to pristine pools without calling the LLM again — used when the writeback itself was
//! corrected, and for hand-edit-preserving re-runs. The cache file is JSON shaped
//! `{ "<pool-path>": { "<OpVariant>": ["alias", ...], ... }, ... }`.
//!
//! Invoked via `just help-aliases` — NOT during CI or shipped builds.
//! This is a standalone (non-workspace) dev-only crate. See README.md.

mod batch;
mod claude;
mod merge;
mod pool;

use std::collections::BTreeMap;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // Forms: `<pool.json>` or `<pool.json> --apply-cache <file>`.
    let (path, cache_file): (&str, Option<&str>) = match args.len() {
        2 => (&args[1], None),
        4 if args[2] == "--apply-cache" => (&args[1], Some(&args[3])),
        _ => {
            eprintln!("usage: help-aliases <pool.json> [--apply-cache <cache.json>]");
            std::process::exit(2);
        }
    };

    let clean = match cache_file {
        Some(cache) => run_pool_from_cache(path, cache),
        None => run_pool(path),
    };
    // A degraded run — any entry left in `[needs manual review]`, including a hard
    // `claude` invocation failure — MUST exit non-zero so it can never be mistaken
    // for a clean run. Otherwise a partial alias write lands in committed JSON
    // silently (v4.2 review, CRITICAL).
    if !clean {
        std::process::exit(1);
    }
}

/// Build the `(scanned, skipped, worklist op_variants)` for a pool: only
/// `status:"implemented"` entries that still need aliases are in the worklist.
fn scan_worklist(entries: &[serde_json::Value]) -> (usize, usize, Vec<String>) {
    let total = entries.len();
    let worklist: Vec<String> = entries
        .iter()
        .filter(|e| {
            e.get("status").and_then(|v| v.as_str()) == Some("implemented")
                && merge::needs_aliases(e)
        })
        .filter_map(|e| {
            e.get("op_variant")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .collect();
    let skipped = total - worklist.len();
    (total, skipped, worklist)
}

/// Splice `by_op` into the pool file in place and print the run summary.
fn write_and_report(
    path: &str,
    total: usize,
    skipped: usize,
    by_op: &BTreeMap<String, Vec<String>>,
) {
    let original = pool::read_text(path);
    let updated =
        pool::splice_aliases(&original, by_op).unwrap_or_else(|e| panic!("splice {path}: {e}"));
    pool::write_text(path, &updated);

    let populated = by_op.len();
    let aliases_added: usize = by_op.values().map(|v| v.len()).sum();
    println!("{path}: {total} scanned, {skipped} skipped, {populated} populated, {aliases_added} aliases added");
}

/// Drive the full alias-generation pipeline for a single pool (live `claude -p`).
///
/// Returns `true` if the run was clean (every worklist entry got aliases), `false`
/// if any entry was left for manual review — the caller exits non-zero on `false`.
fn run_pool(path: &str) -> bool {
    let entries = pool::load_pool(path);
    let (total, skipped, worklist) = scan_worklist(&entries);

    if worklist.is_empty() {
        println!("{path}: {total} scanned, {skipped} skipped, 0 populated, 0 aliases added");
        return true;
    }

    // Group the work entries by category for batched prompts.
    let worklist_values: Vec<serde_json::Value> = entries
        .iter()
        .filter(|e| {
            e.get("op_variant")
                .and_then(|v| v.as_str())
                .map(|op| worklist.iter().any(|w| w == op))
                .unwrap_or(false)
        })
        .cloned()
        .collect();
    let worklist_refs: Vec<&serde_json::Value> = worklist_values.iter().collect();
    let batches = batch::group_by_category(&worklist_refs);

    let mut by_op: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut needs_review: Vec<String> = Vec::new();

    for (category, batch_entries) in &batches {
        let prompt = batch::build_prompt(category, batch_entries);
        let expected_variants: Vec<&str> = batch_entries
            .iter()
            .filter_map(|e| e.get("op_variant").and_then(|v| v.as_str()))
            .collect();

        match claude::invoke_claude(&prompt) {
            Ok(response) => {
                if let Some(map) = response.as_object() {
                    let (missing, extra) = batch::validate_batch_response(&expected_variants, map);
                    if !extra.is_empty() {
                        eprintln!("  [warn] extra keys in response for '{category}': {extra:?}");
                    }
                    if !missing.is_empty() {
                        eprintln!(
                            "  [warn] missing keys in response for '{category}': {missing:?}"
                        );
                        for m in &missing {
                            needs_review.push(format!("{category}/{m}"));
                        }
                    }
                    // Collect aliases for worklist op_variants only (fill-only is enforced by the
                    // worklist scan and again by splice_aliases skipping already-populated entries).
                    for (op_variant, aliases_val) in map {
                        if !worklist.iter().any(|w| w == op_variant) {
                            continue;
                        }
                        if let Some(arr) = aliases_val.as_array() {
                            let aliases: Vec<String> = arr
                                .iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect();
                            if !aliases.is_empty() {
                                by_op.insert(op_variant.clone(), aliases);
                            }
                        }
                    }
                } else {
                    eprintln!("[error] claude response for '{category}' was not a JSON object");
                    for v in &expected_variants {
                        needs_review.push(format!("{category}/{v}"));
                    }
                }
            }
            Err(e) => {
                eprintln!("[error] claude call failed for '{category}': {e}");
                for v in &expected_variants {
                    needs_review.push(format!("{category}/{v}"));
                }
            }
        }
    }

    write_and_report(path, total, skipped, &by_op);

    if !needs_review.is_empty() {
        println!("  [needs manual review] {}", needs_review.join(", "));
    }
    needs_review.is_empty()
}

/// Re-apply aliases from a cached `{ pool-path: { op_variant: [aliases] } }` map (no LLM call).
/// Only worklist (implemented + currently-unaliased) op_variants are applied (fill-only).
fn run_pool_from_cache(path: &str, cache_file: &str) -> bool {
    let entries = pool::load_pool(path);
    let (total, skipped, worklist) = scan_worklist(&entries);

    let cache_text = std::fs::read_to_string(cache_file)
        .unwrap_or_else(|e| panic!("read cache {cache_file}: {e}"));
    let cache: serde_json::Value = serde_json::from_str(&cache_text)
        .unwrap_or_else(|e| panic!("parse cache {cache_file}: {e}"));

    // Match the pool by exact path, falling back to a UNIQUE basename match. An
    // ambiguous basename (>1 cache key ending with it) is refused rather than
    // silently binding the wrong pool's aliases (v4.2 review).
    let pool_map = cache.get(path).or_else(|| {
        let base = std::path::Path::new(path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(path);
        let m = cache.as_object()?;
        let mut hits = m.iter().filter(|(k, _)| k.ends_with(base));
        let first = hits.next();
        if hits.next().is_some() {
            eprintln!(
                "  [error] cache: basename '{base}' matches multiple keys; refusing to guess (use the exact pool path as the cache key)"
            );
            return None;
        }
        if let Some((k, _)) = first {
            eprintln!("  [info] cache: no exact-path key '{path}'; using basename match '{k}'");
        }
        first.map(|(_, v)| v)
    });

    let mut by_op: BTreeMap<String, Vec<String>> = BTreeMap::new();
    if let Some(map) = pool_map.and_then(|v| v.as_object()) {
        for op in &worklist {
            if let Some(arr) = map.get(op).and_then(|v| v.as_array()) {
                let aliases: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                if !aliases.is_empty() {
                    by_op.insert(op.clone(), aliases);
                }
            }
        }
    }

    write_and_report(path, total, skipped, &by_op);

    // Surface any worklist entry the cache did not cover.
    let uncovered: Vec<&String> = worklist
        .iter()
        .filter(|w| !by_op.contains_key(*w))
        .collect();
    if !uncovered.is_empty() {
        println!("  [needs manual review] {} not in cache", uncovered.len());
    }
    uncovered.is_empty()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::scan_worklist;
    use serde_json::json;

    /// `scan_worklist` gates the LLM worklist on BOTH `status == "implemented"`
    /// AND `needs_aliases` (D-60.3 / D-60.2): non-implemented entries (deferred /
    /// na) and already-populated entries are excluded. Preserves the status-gate
    /// coverage after the merge.rs dead-code deletion (v4.2 review).
    #[test]
    fn scan_worklist_selects_only_implemented_unpopulated_entries() {
        let entries = vec![
            json!({ "op_variant": "NeedsIt", "status": "implemented" }),
            json!({ "op_variant": "Populated", "status": "implemented", "search_aliases": ["x"] }),
            json!({ "op_variant": "Deferred", "status": "deferred-v3" }),
            json!({ "op_variant": "Na", "status": "na" }),
            json!({ "op_variant": "EmptyAliases", "status": "implemented", "search_aliases": [] }),
        ];
        let (total, skipped, worklist) = scan_worklist(&entries);
        assert_eq!(total, 5);
        assert_eq!(
            worklist,
            vec!["NeedsIt".to_string(), "EmptyAliases".to_string()],
            "only implemented + unpopulated entries, preserving order"
        );
        assert_eq!(skipped, total - worklist.len());
    }
}
