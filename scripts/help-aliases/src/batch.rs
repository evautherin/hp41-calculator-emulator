//! Per-category batching: group entries, build prompts, validate responses.
//!
//! Strategy: one `claude -p` call per category (or sub-batch for large categories).
//! - `group_by_category`: groups entries by their `category` field.
//! - `build_prompt`: constructs the DE+EN alias generation prompt for a batch.
//! - `validate_batch_response`: detects missing and extra `op_variant` keys.
//!
//! Large categories (Adv Mtrx ~51, Adv Math ~45) are split into ~17/~15-entry
//! sub-batches to stay within practical context limits (RESEARCH Open Question 3/A4).

use serde_json::{Map, Value};

/// Maximum batch size per `claude -p` call.
/// Large categories are split into sub-batches of at most this size.
const MAX_BATCH_SIZE: usize = 20;

/// Group a pool worklist by the `category` field of each entry.
///
/// Large categories (> `MAX_BATCH_SIZE`) are automatically split into sub-batches
/// named `"Category (1/N)"`, `"Category (2/N)"`, etc.
///
/// Returns a `Vec` of `(display_name, entries)` pairs in a stable order.
pub fn group_by_category<'a>(entries: &[&'a Value]) -> Vec<(String, Vec<&'a Value>)> {
    use std::collections::BTreeMap;

    // Group by category, preserving order within each group.
    let mut by_category: BTreeMap<String, Vec<&'a Value>> = BTreeMap::new();
    for entry in entries {
        let cat = entry
            .get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        by_category.entry(cat).or_default().push(entry);
    }

    // Split large categories into sub-batches.
    let mut result: Vec<(String, Vec<&'a Value>)> = Vec::new();
    for (category, cat_entries) in by_category {
        if cat_entries.len() <= MAX_BATCH_SIZE {
            result.push((category, cat_entries));
        } else {
            // Split into chunks of MAX_BATCH_SIZE.
            let chunks: Vec<_> = cat_entries.chunks(MAX_BATCH_SIZE).collect();
            let total = chunks.len();
            for (i, chunk) in chunks.into_iter().enumerate() {
                let label = format!("{category} ({}/{})", i + 1, total);
                result.push((label, chunk.to_vec()));
            }
        }
    }

    result
}

/// Build the DE+EN alias generation prompt for a batch of entries.
///
/// Prompt design (RESEARCH §Prompt Design):
/// - 4–8 aliases per entry, DE+EN combined
/// - Keep German umlauts as real characters
/// - Output: JSON object mapping op_variant → [aliases]
pub fn build_prompt(category: &str, entries: &[&Value]) -> String {
    let mut prompt = String::new();

    prompt.push_str(
        "Generate DE+EN search aliases for these HP-41 calculator functions.\n\n\
        For each function, produce 4-8 short search terms (2-5 words each) that a user \
        might type when looking for this function. Include both German and English terms.\n\
        These are search keywords, not documentation prose. Include common synonyms, \
        abbreviations, and natural-language phrasings.\n\n\
        Keep German umlauts as real characters (ä, ö, ü, Ä, Ö, Ü, ß), not escaped.\n\n\
        Output ONLY a JSON object where each key is the op_variant and the value is an array \
        of alias strings — no prose, no markdown, no code blocks:\n\
        {\"OpVariant1\": [\"alias1\", \"alias2\", ...], \"OpVariant2\": [...], ...}\n\n",
    );

    prompt.push_str(&format!("Functions (category: {category}):\n\n"));

    for entry in entries {
        let op_variant = entry
            .get("op_variant")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let display_name = entry
            .get("display_name")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let description = entry
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let notes = entry.get("notes").and_then(|v| v.as_str());

        if let Some(notes_text) = notes {
            prompt.push_str(&format!(
                "{op_variant}: display={display_name}, description={description}, notes={notes_text}\n"
            ));
        } else {
            prompt.push_str(&format!(
                "{op_variant}: display={display_name}, description={description}\n"
            ));
        }
    }

    prompt
}

/// Validate that the batch response contains exactly the expected `op_variant` keys.
///
/// Returns `(missing, extra)`:
/// - `missing`: keys expected but not present in the response (Pitfall 5).
/// - `extra`: keys present in the response but not expected (hallucinated entries).
///
/// Both are reported (never silently dropped) so the run summary can surface them.
pub fn validate_batch_response(
    expected_variants: &[&str],
    response: &Map<String, Value>,
) -> (Vec<String>, Vec<String>) {
    let missing: Vec<String> = expected_variants
        .iter()
        .filter(|v| !response.contains_key(**v))
        .map(|v| v.to_string())
        .collect();

    let extra: Vec<String> = response
        .keys()
        .filter(|k| !expected_variants.contains(&k.as_str()))
        .cloned()
        .collect();

    (missing, extra)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;

    /// validate_batch_response must detect missing AND extra op_variant keys.
    #[test]
    fn validate_key_mismatch() {
        let expected = vec!["OpA", "OpB"];
        let mut response_map = Map::new();
        response_map.insert("OpA".to_string(), json!(["alias1"]));
        response_map.insert("OpC".to_string(), json!(["alias2"])); // Extra: OpC

        let (missing, extra) = validate_batch_response(&expected, &response_map);

        assert_eq!(missing, vec!["OpB".to_string()], "OpB must be reported as missing");
        assert_eq!(extra, vec!["OpC".to_string()], "OpC must be reported as extra");
    }

    /// validate_batch_response must report empty missing and extra when keys match exactly.
    #[test]
    fn validate_no_mismatch() {
        let expected = vec!["OpA", "OpB", "OpC"];
        let mut response_map = Map::new();
        response_map.insert("OpA".to_string(), json!(["alias1"]));
        response_map.insert("OpB".to_string(), json!(["alias2"]));
        response_map.insert("OpC".to_string(), json!(["alias3"]));

        let (missing, extra) = validate_batch_response(&expected, &response_map);

        assert!(missing.is_empty(), "no keys must be missing when all expected are present");
        assert!(extra.is_empty(), "no extra keys when response matches expected exactly");
    }

    /// group_by_category must split a large category into sub-batches.
    #[test]
    fn large_category_split() {
        // Create 25 entries in the same category (exceeds MAX_BATCH_SIZE=20).
        let entries_owned: Vec<Value> = (0..25)
            .map(|i| {
                json!({
                    "op_variant": format!("Op{i}"),
                    "display_name": format!("OP{i}"),
                    "category": "Big Category",
                    "status": "implemented",
                    "description": format!("Operation {i}")
                })
            })
            .collect();
        let entries_refs: Vec<&Value> = entries_owned.iter().collect();

        let batches = group_by_category(&entries_refs);

        // Should be split into 2 sub-batches: one of 20, one of 5.
        assert_eq!(batches.len(), 2, "25 entries must split into 2 sub-batches");
        assert!(batches[0].0.contains("1/2"), "first sub-batch must be labelled 1/2");
        assert!(batches[1].0.contains("2/2"), "second sub-batch must be labelled 2/2");
        assert_eq!(batches[0].1.len(), 20, "first sub-batch must have 20 entries");
        assert_eq!(batches[1].1.len(), 5, "second sub-batch must have 5 entries");
    }
}
