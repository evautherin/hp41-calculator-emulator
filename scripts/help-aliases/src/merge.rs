//! Fill-only alias merge (D-60.2): populates `search_aliases` only when absent/empty.
//!
//! Entries that already have aliases are left byte-for-byte untouched.
//! Only `status:"implemented"` entries are eligible (D-60.3).

use serde_json::Value;

/// Returns `true` if this entry needs aliases populated.
///
/// `true` when `search_aliases` is:
/// - absent (`None`)
/// - an empty array
/// - malformed (not an array)
pub fn needs_aliases(entry: &Value) -> bool {
    match entry.get("search_aliases") {
        None => true,
        Some(Value::Array(arr)) => arr.is_empty(),
        _ => true, // malformed — treat as needing population
    }
}

/// Insert aliases as the last key in the object.
///
/// With `preserve_order` (IndexMap), inserting a new key appends it at the end —
/// which is the correct position for `search_aliases` (D-60.4 key ordering).
pub fn insert_aliases(entry: &mut Value, aliases: Vec<String>) {
    let arr: Vec<Value> = aliases.into_iter().map(Value::String).collect();
    if let Value::Object(map) = entry {
        map.insert("search_aliases".to_string(), Value::Array(arr));
    }
}

/// Walk a pool, apply the given `op_variant -> aliases` map (fill-only), and return counts.
///
/// Only entries with `status == "implemented"` AND `needs_aliases == true` are modified.
/// Returns `(scanned, skipped, populated, aliases_added)`.
///
/// Used by the test suite and available as a utility for callers that build
/// an alias map separately (e.g. from a cached claude response).
#[allow(dead_code)]
pub fn fill_aliases(
    entries: &mut Vec<Value>,
    alias_map: &std::collections::HashMap<String, Vec<String>>,
) -> (usize, usize, usize, usize) {
    let scanned = entries.len();
    let mut populated = 0usize;
    let mut aliases_added = 0usize;

    for entry in entries.iter_mut() {
        let is_implemented =
            entry.get("status").and_then(|v| v.as_str()) == Some("implemented");

        if !is_implemented || !needs_aliases(entry) {
            continue;
        }

        let op_variant = entry
            .get("op_variant")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if let Some(key) = op_variant {
            if let Some(aliases) = alias_map.get(&key) {
                let count = aliases.len();
                insert_aliases(entry, aliases.clone());
                populated += 1;
                aliases_added += count;
            }
        }
    }

    let skipped = scanned - populated;
    (scanned, skipped, populated, aliases_added)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;

    /// An entry with a non-empty `search_aliases` is NOT selected for population.
    #[test]
    fn fill_only_skips_populated() {
        let entry = json!({
            "op_variant": "TestOp",
            "display_name": "TEST",
            "category": "Test",
            "status": "implemented",
            "phase": "1",
            "key_path": null,
            "description": "A test operation",
            "search_aliases": ["alias1", "alias2"]
        });

        // needs_aliases must return false for a populated entry.
        assert!(!needs_aliases(&entry), "populated entry must NOT need aliases");

        // fill_aliases must leave the entry untouched.
        let mut entries = vec![entry.clone()];
        let mut alias_map = std::collections::HashMap::new();
        alias_map.insert("TestOp".to_string(), vec!["new alias".to_string()]);

        let (scanned, skipped, populated, added) = fill_aliases(&mut entries, &alias_map);

        assert_eq!(scanned, 1);
        assert_eq!(skipped, 1, "entry with existing aliases must be skipped");
        assert_eq!(populated, 0);
        assert_eq!(added, 0);

        // The entry's search_aliases must be byte-for-byte identical to the original.
        let aliases = entries[0]["search_aliases"].as_array().unwrap();
        assert_eq!(aliases.len(), 2);
        assert_eq!(aliases[0].as_str().unwrap(), "alias1");
        assert_eq!(aliases[1].as_str().unwrap(), "alias2");
    }

    /// An entry with absent OR empty `search_aliases` gets populated.
    #[test]
    fn fill_only_populates_empty() {
        // Case 1: absent search_aliases.
        let entry_absent = json!({
            "op_variant": "AbsentOp",
            "display_name": "ABSENT",
            "category": "Test",
            "status": "implemented",
            "phase": "1",
            "key_path": null,
            "description": "No aliases field"
        });

        // Case 2: empty search_aliases array.
        let entry_empty = json!({
            "op_variant": "EmptyOp",
            "display_name": "EMPTY",
            "category": "Test",
            "status": "implemented",
            "phase": "1",
            "key_path": null,
            "description": "Empty aliases array",
            "search_aliases": []
        });

        assert!(needs_aliases(&entry_absent), "absent search_aliases must need population");
        assert!(needs_aliases(&entry_empty), "empty search_aliases must need population");

        // Test insert_aliases appends the key as the LAST key.
        let mut entry = entry_absent.clone();
        insert_aliases(&mut entry, vec!["a".to_string(), "b".to_string()]);

        let aliases = entry["search_aliases"].as_array().unwrap();
        assert_eq!(aliases.len(), 2);
        assert_eq!(aliases[0].as_str().unwrap(), "a");
        assert_eq!(aliases[1].as_str().unwrap(), "b");

        // With preserve_order, search_aliases must be the last key.
        if let Value::Object(map) = &entry {
            let keys: Vec<&str> = map.keys().map(|k| k.as_str()).collect();
            assert_eq!(
                keys.last().copied(),
                Some("search_aliases"),
                "search_aliases must be the last key in the object"
            );
        }

        // Test via fill_aliases.
        let mut entries = vec![entry_absent, entry_empty];
        let mut alias_map = std::collections::HashMap::new();
        alias_map.insert("AbsentOp".to_string(), vec!["alias_a".to_string(), "alias_b".to_string()]);
        alias_map.insert("EmptyOp".to_string(), vec!["alias_c".to_string()]);

        let (scanned, skipped, populated, added) = fill_aliases(&mut entries, &alias_map);
        assert_eq!(scanned, 2);
        assert_eq!(skipped, 0);
        assert_eq!(populated, 2);
        assert_eq!(added, 3);
    }

    /// Entries with `status != "implemented"` must NOT be selected for population (D-60.3).
    #[test]
    fn skips_non_implemented() {
        let entry_deferred = json!({
            "op_variant": "DeferredOp",
            "display_name": "DEFERRED",
            "category": "Test",
            "status": "deferred-v3",
            "phase": "1",
            "key_path": null,
            "description": "A deferred operation"
        });

        let entry_na = json!({
            "op_variant": "NaOp",
            "display_name": "NA",
            "category": "Test",
            "status": "na",
            "phase": "1",
            "key_path": null,
            "description": "Not applicable"
        });

        let mut entries = vec![entry_deferred, entry_na];
        let mut alias_map = std::collections::HashMap::new();
        alias_map.insert("DeferredOp".to_string(), vec!["alias1".to_string()]);
        alias_map.insert("NaOp".to_string(), vec!["alias2".to_string()]);

        let (scanned, skipped, populated, added) = fill_aliases(&mut entries, &alias_map);

        assert_eq!(scanned, 2);
        assert_eq!(skipped, 2, "non-implemented entries must all be skipped");
        assert_eq!(populated, 0);
        assert_eq!(added, 0);

        // search_aliases must NOT have been added to either entry.
        assert!(entries[0].get("search_aliases").is_none());
        assert!(entries[1].get("search_aliases").is_none());
    }
}
