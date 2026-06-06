//! Alias-eligibility predicate (D-60.2 / D-60.3).
//!
//! This module now contains ONLY `needs_aliases` — the fill-only / `implemented`
//! eligibility check. The actual on-disk writeback is `pool::splice_aliases`, a
//! minimal text splice that preserves the byte-exact alias-only diff (D-60.4).
//!
//! The earlier `serde_json`-re-serialization helpers (`insert_aliases` /
//! `fill_aliases`) were removed in the v4.2 review: they were dead code (no
//! production caller) AND embodied the exact `PrettyFormatter` churn that broke
//! the alias-only-diff guarantee in 60-01 (the Wave-2 defect). Do NOT reintroduce
//! a re-serialization writeback — splice the text via `pool::splice_aliases`.

use serde_json::Value;

/// Returns `true` if this entry needs aliases populated.
///
/// `true` when `search_aliases` is absent, an empty array, or malformed (not an
/// array). Used by `scan_worklist` to build the LLM worklist; the same fill-only
/// guard is re-checked in `pool::splice_aliases` as a safety net.
pub fn needs_aliases(entry: &Value) -> bool {
    match entry.get("search_aliases") {
        None => true,
        Some(Value::Array(arr)) => arr.is_empty(),
        _ => true, // malformed — treat as needing population
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;

    /// `needs_aliases` is the eligibility predicate gating the LLM worklist (and
    /// the `pool::splice_aliases` safety net): absent / empty / malformed
    /// `search_aliases` → needs population; a non-empty array → already populated
    /// and must be left byte-for-byte untouched. (The `status == "implemented"`
    /// gate lives in `main::scan_worklist`, tested there.)
    #[test]
    fn needs_aliases_classifies_the_alias_field() {
        let populated = json!({
            "op_variant": "TestOp", "status": "implemented",
            "search_aliases": ["alias1", "alias2"]
        });
        assert!(
            !needs_aliases(&populated),
            "non-empty array → already populated"
        );

        let absent = json!({ "op_variant": "AbsentOp", "status": "implemented" });
        assert!(
            needs_aliases(&absent),
            "absent search_aliases → needs population"
        );

        let empty = json!({
            "op_variant": "EmptyOp", "status": "implemented", "search_aliases": []
        });
        assert!(needs_aliases(&empty), "empty array → needs population");

        let malformed = json!({
            "op_variant": "BadOp", "status": "implemented", "search_aliases": "nope"
        });
        assert!(
            needs_aliases(&malformed),
            "malformed (non-array) → needs population"
        );
    }
}
