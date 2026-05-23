//! Phase 34 Plan 01 smoke tests — `docs/hp41-stat1-functions.json` is the
//! canonical data source for `hp41-cli/src/help_data.rs::help_entries_stat1()` via
//! include_str! + OnceLock per D-34.6 / D-29.2 (third-pool extension).
//!
//! Mirrors `phase29_help_data_math1.rs` structure exactly; swaps accessor and
//! count target (== 26 unique Op variants — Stat 1 Pac is feature-frozen for v3.1).
//!
//! Test 11 (`help_entries_all_returns_three_pools`) asserts the merged chain
//! length is >= 130 + 45 + 26 = 201 (v2.2 built-ins + Math Pac I + Stat 1 Pac).
//!
//! Test 12 (`stat1_help_entries_no_collision_with_other_pools`) asserts cross-pool
//! op_variant uniqueness — precondition for `function_matrix_parity.rs` 3-pool
//! partition in Plan 34-02.

#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use hp41_cli::help_data::{help_entries, help_entries_all, help_entries_math1, help_entries_stat1};

/// Test 1: hard-build-blocker exercised on success path.
#[test]
fn stat1_help_entries_loads_at_runtime() {
    // Catches: hard-build-blocker not firing on malformed JSON (D-25.17 / D-29.2 third-pool)
    let entries = help_entries_stat1();
    assert!(
        !entries.is_empty(),
        "help_entries_stat1() must return a non-empty slice — \
         docs/hp41-stat1-functions.json may be empty or malformed (D-29.2 / D-34.6)"
    );
}

/// Test 2: exact count — Stat 1 Pac is feature-frozen at 26 entries for v3.1.
#[test]
fn stat1_help_entries_count_meets_26_target() {
    // Exact == 26, not >= 26: Stat 1 Pac is feature-frozen per CONTEXT §"Specific Ideas".
    // If a future Stat 1 Pac entry is added without updating this test, it fails loudly.
    let entries = help_entries_stat1();
    assert_eq!(
        entries.len(),
        26,
        "help_entries_stat1().len() = {} — must be exactly 26 (Stat 1 Pac feature-frozen \
         for v3.1; if this fails after adding an entry, update this test and D-34.1)",
        entries.len()
    );
}

/// Test 3: no duplicate op_variant strings within the Stat 1 pool.
#[test]
fn stat1_help_entries_has_no_duplicate_op_variants() {
    // Catches: duplicate op_variant rows in hp41-stat1-functions.json.
    // The bidirectional parity test (Plan 34-02) assumes unique op_variant per row.
    let entries = help_entries_stat1();
    let mut seen: HashSet<&str> = HashSet::with_capacity(entries.len());
    for entry in entries {
        assert!(
            seen.insert(entry.op_variant.as_str()),
            "duplicate op_variant in docs/hp41-stat1-functions.json: {}",
            entry.op_variant
        );
    }
}

/// Test 4: every entry has non-empty description within 200-char bound.
#[test]
fn stat1_help_entries_all_have_non_empty_description() {
    // Catches: empty or over-long descriptions that break the ? overlay.
    // Description renders in the ? overlay — empty rows render as blank lines.
    for entry in help_entries_stat1() {
        assert!(
            !entry.description.is_empty(),
            "entry '{}' in hp41-stat1-functions.json has empty description",
            entry.op_variant
        );
        assert!(
            entry.description.len() <= 200,
            "entry '{}' description exceeds 200 chars ({} chars)",
            entry.op_variant,
            entry.description.len()
        );
    }
}

/// Test 5: status field is constrained to the three permitted values.
#[test]
fn stat1_help_entries_status_is_closed_enum() {
    // Catches: invalid status string in hp41-stat1-functions.json.
    // All Phase 34 entries have status "implemented" (Phase 33 shipped everything).
    for entry in help_entries_stat1() {
        assert!(
            matches!(entry.status.as_str(), "implemented" | "deferred-v3" | "na"),
            "entry '{}' in hp41-stat1-functions.json has invalid status '{}' — \
             must be implemented | deferred-v3 | na",
            entry.op_variant,
            entry.status
        );
    }
}

/// Test 6: every entry carries xrom.module_id == 2 (C-28.3 invariant).
#[test]
fn stat1_help_entries_all_xrom_module_id_is_2() {
    // Catches: missing or incorrect xrom.module_id (C-28.3 invariant).
    // Every Stat 1 Pac entry MUST carry an xrom block with module_id == 2
    // (HP-41 Stat 1 Pac hardware XROM module ID per calc.fjk.ch "STAT 1B" entry).
    for entry in help_entries_stat1() {
        let xrom = entry.xrom.as_ref().unwrap_or_else(|| {
            panic!(
                "entry '{}' in hp41-stat1-functions.json is missing the xrom block \
                 (C-28.3 invariant: every Stat 1 entry must carry xrom.module_id == 2)",
                entry.op_variant
            )
        });
        assert_eq!(
            xrom.module_id, 2,
            "entry '{}' in hp41-stat1-functions.json has xrom.module_id == {} — must be 2 \
             (HP Stat 1 Pac hardware module ID per D-33.1 / D-34.1)",
            entry.op_variant, xrom.module_id
        );
    }
}

/// Test 7: every category begins with "Stat1 " (D-34.1 per-family naming).
#[test]
fn stat1_help_entries_categories_prefix_with_stat1() {
    // Catches: wrong category prefix breaking ? overlay sectioning.
    // Every Stat 1 Pac entry category must start with "Stat1 " so the overlay
    // sections cluster separately from the v2.2 built-in and Math 1 categories.
    for entry in help_entries_stat1() {
        assert!(
            entry.category.starts_with("Stat1 "),
            "entry '{}' in hp41-stat1-functions.json has category '{}' — \
             must start with 'Stat1 ' (per D-34.1 overlay sectioning)",
            entry.op_variant,
            entry.category
        );
    }
}

/// Test 8: function_ids form a dense 1..=26 range with no gaps and no duplicates.
#[test]
fn stat1_help_entries_xrom_function_ids_are_dense() {
    // Catches: duplicate or non-contiguous function_id assignment.
    // function_ids must form a dense 1..=26 range matching STAT_1.ops row order.
    let entries = help_entries_stat1();
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
        "hp41-stat1-functions.json contains duplicate function_id values"
    );

    // Check dense range starting at 1 and ending at 26
    assert_eq!(
        ids.first().copied().unwrap_or(0),
        1,
        "hp41-stat1-functions.json function_ids must start at 1 (HP-41 convention)"
    );
    assert_eq!(
        ids.last().copied().unwrap_or(0),
        26,
        "hp41-stat1-functions.json function_ids must end at 26 (26 Stat 1 Pac entries)"
    );
    assert_eq!(
        ids.len(),
        26,
        "hp41-stat1-functions.json must have exactly 26 unique function_id values (dense 1..=26)"
    );
}

/// Test 9: every entry has key_path in XEQ "<name>" form (D-28.6).
#[test]
fn stat1_help_entries_all_key_path_is_xeq_form() {
    // Catches: wrong key_path format for Stat 1 Pac entries (D-28.6 invariant).
    // Every Stat 1 Pac entry key_path must be Some("XEQ \"<MNEMONIC>\"") because
    // Stat 1 Pac functions are XEQ-by-name only — no dedicated key bindings.
    for entry in help_entries_stat1() {
        let key_path = entry.key_path.as_deref().unwrap_or_else(|| {
            panic!(
                "entry '{}' in hp41-stat1-functions.json has no key_path \
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

/// Test 10: exactly 4 entries carry divergences, and they are the expected 4 (D-34.3).
#[test]
fn stat1_help_entries_divergences_are_surgical() {
    // D-34.3: divergences must be populated ONLY for Rand, Seed, SigmaTstat,
    // SigmaPolypWorkflow. Any addition/removal surfaces here as a CI failure,
    // requiring a matching update to D-34.3 and docs/hp41-stat1-divergences.md (Phase 35).
    let entries = help_entries_stat1();
    let with_div: HashSet<&str> = entries
        .iter()
        .filter(|e| !e.divergences.is_empty())
        .map(|e| e.op_variant.as_str())
        .collect();
    let expected: HashSet<&str> = ["Rand", "Seed", "SigmaTstat", "SigmaPolypWorkflow"]
        .iter()
        .copied()
        .collect();
    assert_eq!(
        with_div, expected,
        "D-34.3: divergences must be populated ONLY for Rand, Seed, SigmaTstat, \
         SigmaPolypWorkflow. Found: {:?}, expected: {:?}. \
         Adding/removing divergences without updating D-34.3 surfaces here.",
        with_div, expected
    );
}

/// Test 11: help_entries_all() chains all three pools (built-ins + Math 1 + Stat 1).
#[test]
fn help_entries_all_returns_three_pools() {
    // Catches: help_entries_all() not chaining all three pools per D-34.6.
    // v2.2 built-in pool: >= 130 entries (phase25_help_data::help_entries_count_meets_130_target)
    // Math Pac I pool: >= 45 entries (phase29_help_data_math1::math1_entries_count_meets_45_target)
    // Stat 1 Pac pool: == 26 entries (stat1_help_entries_count_meets_26_target above)
    // Total minimum: 130 + 45 + 26 = 201
    let all_count = help_entries_all().count();
    assert!(
        all_count >= 201,
        "help_entries_all() returned {} entries but should include >= 130 v2.2 entries \
         + >= 45 Math 1 entries + 26 Stat 1 entries = >= 201 total (D-34.6 three-pool chain)",
        all_count
    );
}

/// Test 12: Stat 1 op_variants do not collide with either prior pool (cross-pool uniqueness).
#[test]
fn stat1_help_entries_no_collision_with_other_pools() {
    // Catches: a Stat 1 mnemonic accidentally shadowing a v2.2 built-in or Math 1 entry.
    // Cross-pool op_variant uniqueness is a precondition for function_matrix_parity.rs
    // 3-pool partition in Plan 34-02 (partition by xrom.module_id).
    let v22: HashSet<&str> = help_entries()
        .iter()
        .map(|e| e.op_variant.as_str())
        .collect();
    let math1: HashSet<&str> = help_entries_math1()
        .iter()
        .map(|e| e.op_variant.as_str())
        .collect();
    for entry in help_entries_stat1() {
        assert!(
            !v22.contains(entry.op_variant.as_str()),
            "Stat 1 op_variant '{}' collides with v2.2 built-in pool — \
             cross-pool op_variant uniqueness is a precondition for \
             function_matrix_parity.rs 3-pool partition (Plan 34-02)",
            entry.op_variant
        );
        assert!(
            !math1.contains(entry.op_variant.as_str()),
            "Stat 1 op_variant '{}' collides with Math Pac I pool — \
             cross-pool op_variant uniqueness is a precondition for \
             function_matrix_parity.rs 3-pool partition (Plan 34-02)",
            entry.op_variant
        );
    }
}
