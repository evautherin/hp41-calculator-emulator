//! Phase 39 Plan 03 smoke tests — `docs/hp41-time-functions.json` is the
//! canonical data source for `hp41-cli/src/help_data.rs::help_entries_time()` via
//! include_str! + OnceLock per D-39.12 (fourth-pool extension).
//!
//! Mirrors `phase34_help_data_stat1.rs` structure exactly; swaps accessor and
//! count target (== 35 unique Op variants — Time Pac is feature-frozen for v3.2).
//!
//! Test 11 (`help_entries_all_returns_four_pools`) asserts the merged chain
//! length is >= 130 + 45 + 26 + 35 = 236 (v2.2 built-ins + Math Pac I + Stat 1 Pac + Time Pac).
//!
//! Test 12 (`help_overlay_rows_includes_time_pac_section`) verifies that the
//! 4th pool feeds into help_overlay_rows() and the Time Pac category headers appear.

#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use hp41_cli::help_data::{
    help_entries, help_entries_all, help_entries_math1, help_entries_stat1, help_entries_time,
    help_overlay_rows,
};

/// Test 1: hard-build-blocker exercised on success path.
#[test]
fn time_help_entries_is_not_empty() {
    // Catches: hard-build-blocker not firing on malformed JSON (D-25.17 / D-39.12 fourth-pool)
    let entries = help_entries_time();
    assert!(
        !entries.is_empty(),
        "help_entries_time() must return a non-empty slice — \
         docs/hp41-time-functions.json may be empty or malformed (D-39.12)"
    );
}

/// Test 2: exact count — Time Pac is feature-frozen at 35 entries for v3.2.
#[test]
fn time_help_entries_count_meets_35_target() {
    // Exact == 35, not >= 35: Time Pac is feature-frozen per v3.2 scope.
    // If a future Time Pac entry is added without updating this test, it fails loudly.
    let entries = help_entries_time();
    assert_eq!(
        entries.len(),
        35,
        "help_entries_time().len() = {} — must be exactly 35 (Time Pac feature-frozen \
         for v3.2; if this fails after adding an entry, update this test and D-39.9)",
        entries.len()
    );
}

/// Test 3: no empty display_name in the Time pool.
#[test]
fn time_every_entry_has_display_name() {
    // Catches: entries with empty display_name breaking the ? overlay rendering.
    for entry in help_entries_time() {
        assert!(
            !entry.display_name.is_empty(),
            "entry '{}' in hp41-time-functions.json has empty display_name",
            entry.op_variant
        );
    }
}

/// Test 4: every entry has non-empty description.
#[test]
fn time_every_entry_has_description() {
    // Catches: empty descriptions that break the ? overlay.
    for entry in help_entries_time() {
        assert!(
            !entry.description.is_empty(),
            "entry '{}' in hp41-time-functions.json has empty description",
            entry.op_variant
        );
    }
}

/// Test 5: all Time Pac entries are implemented.
#[test]
fn time_all_entries_are_implemented() {
    // Time Pac is fully implemented in Phase 38 — all 35 entries must be "implemented".
    for entry in help_entries_time() {
        assert_eq!(
            entry.status.as_str(),
            "implemented",
            "entry '{}' in hp41-time-functions.json has status '{}' — \
             all Time Pac entries should be 'implemented' (Phase 38 shipped all 35 ops)",
            entry.op_variant,
            entry.status
        );
    }
}

/// Test 6: every entry carries xrom.module_id == 26 (HP-41CX Time module hardware ID).
#[test]
fn time_every_entry_has_xrom_module_id_26() {
    // Catches: missing or incorrect xrom.module_id (C-28.3 invariant).
    // Every Time Pac entry MUST carry an xrom block with module_id == 26
    // (HP-41CX Time module hardware XROM module ID per D-39.9 / D-33.1 schema).
    for entry in help_entries_time() {
        let xrom = entry.xrom.as_ref().unwrap_or_else(|| {
            panic!(
                "entry '{}' in hp41-time-functions.json is missing the xrom block \
                 (C-28.3 invariant: every Time Pac entry must carry xrom.module_id == 26)",
                entry.op_variant
            )
        });
        assert_eq!(
            xrom.module_id, 26,
            "entry '{}' in hp41-time-functions.json has xrom.module_id == {} — must be 26 \
             (HP-41CX Time module hardware XROM ID per D-39.9 / D-33.1)",
            entry.op_variant, xrom.module_id
        );
    }
}

/// Test 7: every category begins with "Time " (D-39.9 per-family naming).
#[test]
fn time_categories_use_time_prefix() {
    // Catches: wrong category prefix breaking ? overlay sectioning.
    // Every Time Pac entry category must start with "Time " so the overlay
    // sections cluster separately from the v2.2 built-in, Math 1, and Stat 1 categories.
    for entry in help_entries_time() {
        assert!(
            entry.category.starts_with("Time "),
            "entry '{}' in hp41-time-functions.json has category '{}' — \
             must start with 'Time ' (per D-39.9 overlay sectioning; 7 categories expected)",
            entry.op_variant,
            entry.category
        );
    }
}

/// Test 8: function_ids form a dense 1..=35 range with no gaps and no duplicates.
#[test]
fn time_function_ids_dense_and_sequential() {
    // Catches: duplicate or non-contiguous function_id assignment.
    // function_ids must form a dense 1..=35 range matching TIME_MODULE.ops row order.
    let entries = help_entries_time();
    let mut ids: Vec<u16> = entries
        .iter()
        .map(|e| {
            e.xrom
                .as_ref()
                .unwrap_or_else(|| panic!("entry '{}' missing xrom block", e.op_variant))
                .function_id
        })
        .collect();
    ids.sort_unstable();

    // Check no duplicates
    let unique_count = {
        let mut deduped = ids.clone();
        deduped.dedup();
        deduped.len()
    };
    assert_eq!(
        ids.len(),
        unique_count,
        "hp41-time-functions.json contains duplicate function_id values"
    );

    // Check dense range starting at 1 and ending at 35
    assert_eq!(
        ids.first().copied().unwrap_or(0),
        1,
        "hp41-time-functions.json function_ids must start at 1 (HP-41 convention)"
    );
    assert_eq!(
        ids.last().copied().unwrap_or(0),
        35,
        "hp41-time-functions.json function_ids must end at 35 (35 Time Pac entries)"
    );
    assert_eq!(
        ids.len(),
        35,
        "hp41-time-functions.json must have exactly 35 unique function_id values (dense 1..=35)"
    );
}

/// Test 9: every entry has key_path in XEQ "<name>" form (D-28.6).
#[test]
fn time_no_key_path_is_empty() {
    // Catches: wrong key_path format for Time Pac entries (D-28.6 invariant).
    // Every Time Pac entry key_path must be Some("XEQ \"<MNEMONIC>\"") because
    // Time Pac functions are XEQ-by-name only — no dedicated key bindings.
    for entry in help_entries_time() {
        let key_path = entry.key_path.as_deref().unwrap_or_else(|| {
            panic!(
                "entry '{}' in hp41-time-functions.json has no key_path \
                 (D-28.6: XEQ-by-name only, key_path must be Some(...))",
                entry.op_variant
            )
        });
        assert!(
            key_path.starts_with("XEQ \"") && key_path.ends_with('"'),
            "entry '{}' key_path '{}' must be in XEQ \"<MNEMONIC>\" form (D-28.6)",
            entry.op_variant,
            key_path
        );
    }
}

/// Test 10: exactly 4 entries carry divergences, and they are the expected 4 (D-39.11).
#[test]
fn time_divergences_are_surgical() {
    // D-39.11: divergences must be populated ONLY for SETAF, RCLAF, CORRECT, SW.
    // Any addition/removal surfaces here as a CI failure, requiring a matching
    // update to D-39.11 and docs/hp41-time-divergences.md (future Phase 40 or similar).
    let entries = help_entries_time();
    let with_div: HashSet<&str> = entries
        .iter()
        .filter(|e| !e.divergences.is_empty())
        .map(|e| e.display_name.as_str())
        .collect();
    let expected: HashSet<&str> = ["SETAF", "RCLAF", "CORRECT", "SW"]
        .iter()
        .copied()
        .collect();
    assert_eq!(
        with_div, expected,
        "D-39.11: divergences must be populated ONLY for SETAF, RCLAF, CORRECT, SW. \
         Found: {with_div:?}, expected: {expected:?}. \
         Adding/removing divergences without updating D-39.11 surfaces here.",
    );
}

/// Test 11: help_entries_all() chains all four pools (built-ins + Math 1 + Stat 1 + Time).
#[test]
fn help_entries_all_returns_four_pools() {
    // Catches: help_entries_all() not chaining all four pools per D-39.12.
    // v2.2 built-in pool: >= 130 entries
    // Math Pac I pool: >= 45 entries
    // Stat 1 Pac pool: == 26 entries
    // Time Pac pool: == 35 entries
    // Total minimum: 130 + 45 + 26 + 35 = 236
    let all_count = help_entries_all().count();
    assert!(
        all_count >= 236,
        "help_entries_all() returned {all_count} entries but should include >= 130 v2.2 entries \
         + >= 45 Math 1 entries + 26 Stat 1 entries + 35 Time entries = >= 236 total (D-39.12 four-pool chain)",
    );
}

/// Test 12: help_overlay_rows() contains at least one Time Pac category header.
#[test]
fn help_overlay_rows_includes_time_pac_section() {
    // Catches: the 4th pool not feeding into help_overlay_rows().
    // help_overlay_rows() generates "=== {category} ===" header rows for each
    // unique category in help_entries_all(). Since all Time Pac entries have
    // categories starting with "Time ", at least one such header must appear.
    let rows = help_overlay_rows();
    let has_time_header = rows.iter().any(|r| {
        // Category headers have the form "=== <category> ==="
        r.desc.starts_with("=== Time ") && r.desc.ends_with(" ===")
    });
    assert!(
        has_time_header,
        "help_overlay_rows() must contain at least one '=== Time * ===' category header \
         from the Time Pac entries — the 4th pool does not appear to feed into help_overlay_rows(). \
         Check that help_entries_all() chains help_entries_time() per D-39.12."
    );
}

/// Bonus: Time pool op_variants do not collide with prior pools (cross-pool uniqueness).
#[test]
fn time_help_entries_no_collision_with_other_pools() {
    // Catches: a Time mnemonic accidentally shadowing a v2.2 built-in, Math 1, or Stat 1 entry.
    let v22: HashSet<&str> = help_entries()
        .iter()
        .map(|e| e.op_variant.as_str())
        .collect();
    let math1: HashSet<&str> = help_entries_math1()
        .iter()
        .map(|e| e.op_variant.as_str())
        .collect();
    let stat1: HashSet<&str> = help_entries_stat1()
        .iter()
        .map(|e| e.op_variant.as_str())
        .collect();
    for entry in help_entries_time() {
        assert!(
            !v22.contains(entry.op_variant.as_str()),
            "Time op_variant '{}' collides with v2.2 built-in pool",
            entry.op_variant
        );
        assert!(
            !math1.contains(entry.op_variant.as_str()),
            "Time op_variant '{}' collides with Math Pac I pool",
            entry.op_variant
        );
        assert!(
            !stat1.contains(entry.op_variant.as_str()),
            "Time op_variant '{}' collides with Stat 1 Pac pool",
            entry.op_variant
        );
    }
}
