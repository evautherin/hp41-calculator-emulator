//! `help-aliases` — Dev-only alias authoring generator for HP-41 help JSON pools.
//!
//! Usage: `help-aliases <pool.json>`
//!
//! Reads the given pool file in-place, generates DE+EN search aliases for each
//! `status:"implemented"` entry that has no aliases yet, and writes back to
//! the same file (D-60.4 minimal-diff writeback).
//!
//! Invoked via `just help-aliases` — NOT during CI or shipped builds.
//! This is a standalone (non-workspace) dev-only crate. See README.md.

mod batch;
mod claude;
mod merge;
mod pool;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: help-aliases <pool.json>");
        std::process::exit(2);
    }
    let path = &args[1];
    run_pool(path);
}

/// Drive the full alias-generation pipeline for a single pool.
fn run_pool(path: &str) {
    let mut entries = pool::load_pool(path);
    let total = entries.len();

    // Collect indices + snapshot the data we need for batching (no long-lived borrow).
    struct WorkItem {
        idx: usize,
        op_variant: String,
    }
    let worklist: Vec<WorkItem> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            e.get("status").and_then(|v| v.as_str()) == Some("implemented")
                && merge::needs_aliases(e)
        })
        .map(|(i, e)| WorkItem {
            idx: i,
            op_variant: e
                .get("op_variant")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
        .collect();

    let skipped = total - worklist.len();

    if worklist.is_empty() {
        println!(
            "{path}: {total} scanned, {skipped} skipped, 0 populated, 0 aliases added"
        );
        return;
    }

    // Group by category (using cloned Value refs for the batch builder).
    // We clone only the entries we need for prompt building — the main Vec is mutable.
    let worklist_values: Vec<serde_json::Value> =
        worklist.iter().map(|w| entries[w.idx].clone()).collect();
    let worklist_refs: Vec<&serde_json::Value> = worklist_values.iter().collect();
    let batches = batch::group_by_category(&worklist_refs);

    let mut populated = 0usize;
    let mut aliases_added = 0usize;
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
                    let (missing, extra) =
                        batch::validate_batch_response(&expected_variants, map);

                    if !extra.is_empty() {
                        eprintln!(
                            "  [warn] extra keys in response for '{category}': {extra:?}"
                        );
                    }
                    if !missing.is_empty() {
                        eprintln!(
                            "  [warn] missing keys in response for '{category}': {missing:?}"
                        );
                        for m in &missing {
                            needs_review.push(format!("{category}/{m}"));
                        }
                    }

                    // Apply aliases back to the original entries Vec.
                    for (op_variant, aliases_val) in map {
                        if let Some(aliases) = aliases_val.as_array() {
                            let alias_strings: Vec<String> = aliases
                                .iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect();

                            // Find the matching index in our worklist by op_variant name.
                            if let Some(item) = worklist
                                .iter()
                                .find(|w| w.op_variant == op_variant.as_str())
                            {
                                let count = alias_strings.len();
                                merge::insert_aliases(&mut entries[item.idx], alias_strings);
                                populated += 1;
                                aliases_added += count;
                            }
                        }
                    }
                } else {
                    eprintln!(
                        "[error] claude response for '{category}' was not a JSON object"
                    );
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

    // Save the updated pool in-place.
    pool::save_pool(path, &entries);

    // Print per-pool run summary (D-60.5).
    println!(
        "{path}: {total} scanned, {skipped} skipped, {populated} populated, {aliases_added} aliases added"
    );

    if !needs_review.is_empty() {
        println!("  [needs manual review] {}", needs_review.join(", "));
    }
}
