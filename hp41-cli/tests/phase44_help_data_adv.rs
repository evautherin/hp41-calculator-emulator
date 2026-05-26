//! Phase 44 Plan 02 Task 2 — `docs/hp41-advantage-functions.json` smoke tests (ADV-CLI-05).
//!
//! Mirrors `phase39_help_data_time.rs` structure exactly; swaps accessor and count
//! targets (== 114 unique Op variants split 63 XROM 22 + 51 XROM 24 per Phase 43 ship).
//!
//! Test 11 (`help_entries_all_returns_five_pools`) asserts the merged chain length is
//! >= 130 + 45 + 26 + 35 + 114 = 350 (v2.2 built-ins + Math Pac I + Stat 1 Pac +
//! Time Pac + Advantage Pac). This is the five-pool chain established in Phase 44 Plan 01.
//!
//! Test 12 (`help_overlay_rows_includes_adv_pac_sections`) verifies that the fifth pool
//! feeds into help_overlay_rows() and the Advantage Pac category headers appear.

#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use hp41_cli::help_data::{help_entries_adv, help_entries_all, help_overlay_rows};

/// Test 1: hard-build-blocker exercised on success path.
#[test]
fn adv_help_entries_is_not_empty() {
    // Catches: hard-build-blocker not firing on malformed JSON (D-25.17 / D-44.1 fifth-pool)
    let entries = help_entries_adv();
    assert!(
        !entries.is_empty(),
        "help_entries_adv() must return a non-empty slice — \
         docs/hp41-advantage-functions.json may be empty or malformed (D-44.1)"
    );
}

/// Test 2: exact count — Advantage Pac is feature-frozen at 114 entries for v3.3.
#[test]
fn adv_help_entries_count_meets_114_target() {
    // Exact == 114, not >= 114: Advantage Pac is feature-frozen per v3.3 scope.
    // 63 from ADV_MATH_A (XROM 22) + 51 from ADV_MATH_B (XROM 24) = 114.
    // If a future Advantage Pac entry is added without updating this test, it fails loudly.
    let entries = help_entries_adv();
    assert_eq!(
        entries.len(),
        114,
        "help_entries_adv().len() = {} — must be exactly 114 (Advantage Pac feature-frozen \
         for v3.3; if this fails after adding an entry, update this test and D-44.1)",
        entries.len()
    );
}

/// Test 3: no empty display_name in the Advantage pool.
#[test]
fn adv_every_entry_has_display_name() {
    // Catches: entries with empty display_name breaking the ? overlay rendering.
    for entry in help_entries_adv() {
        assert!(
            !entry.display_name.is_empty(),
            "entry '{}' in hp41-advantage-functions.json has empty display_name",
            entry.op_variant
        );
    }
}

/// Test 4: every entry has non-empty description.
#[test]
fn adv_every_entry_has_description() {
    // Catches: empty descriptions that break the ? overlay.
    for entry in help_entries_adv() {
        assert!(
            !entry.description.is_empty(),
            "entry '{}' in hp41-advantage-functions.json has empty description",
            entry.op_variant
        );
    }
}

/// Test 5: all Advantage Pac entries are implemented.
#[test]
fn adv_all_entries_are_implemented() {
    // Advantage Pac is fully implemented in Phase 43 — all 114 entries must be "implemented".
    for entry in help_entries_adv() {
        assert_eq!(
            entry.status.as_str(),
            "implemented",
            "entry '{}' in hp41-advantage-functions.json has status '{}' — \
             all Advantage Pac entries should be 'implemented' (Phase 43 shipped all 114 ops)",
            entry.op_variant,
            entry.status
        );
    }
}

/// Test 6: every entry carries xrom.module_id in {22, 24} (dual-module check).
#[test]
fn adv_every_entry_has_xrom_module_id_in_22_or_24() {
    // Catches: missing or incorrect xrom.module_id (C-28.3 invariant).
    // Every Advantage Pac entry MUST carry an xrom block with module_id == 22 (ADV_MATH_A)
    // OR module_id == 24 (ADV_MATH_B) per the dual-XROM design.
    // NOT a single value check — both module IDs are valid.
    for entry in help_entries_adv() {
        let xrom = entry.xrom.as_ref().unwrap_or_else(|| {
            panic!(
                "entry '{}' in hp41-advantage-functions.json is missing the xrom block \
                 (C-28.3 invariant: every Advantage Pac entry must carry xrom.module_id in {{22, 24}})",
                entry.op_variant
            )
        });
        assert!(
            xrom.module_id == 22 || xrom.module_id == 24,
            "entry '{}' in hp41-advantage-functions.json has xrom.module_id == {} — \
             must be 22 (ADV_MATH_A / ADV CONV+MTRX) or 24 (ADV_MATH_B / ADV MATH+TVM) \
             per the Advantage Pac dual-XROM design (D-44.1 / D-33.1 schema)",
            entry.op_variant,
            xrom.module_id
        );
    }
}

/// Test 7: every category begins with "Adv " (D-44.1 per-family naming).
#[test]
fn adv_categories_use_adv_prefix() {
    // Catches: wrong category prefix breaking ? overlay sectioning.
    // Every Advantage Pac entry category must start with "Adv " so the overlay
    // sections cluster separately from the v2.2 built-in, Math 1, Stat 1, and Time categories.
    for entry in help_entries_adv() {
        assert!(
            entry.category.starts_with("Adv "),
            "entry '{}' in hp41-advantage-functions.json has category '{}' — \
             must start with 'Adv ' (per D-44.1 overlay sectioning convention)",
            entry.op_variant,
            entry.category
        );
    }
}

/// Test 8: function_ids form dense 1-based ranges per module (XROM 22 and XROM 24 separately).
#[test]
fn adv_function_ids_dense_per_module() {
    // Catches: duplicate or non-contiguous function_id assignment within each module.
    // Pitfall 1: IDs are 1-based PER MODULE — do NOT check them as a single combined range.
    // ADV_MATH_A (module_id=22): function_ids 1..=63
    // ADV_MATH_B (module_id=24): function_ids 1..=51
    let entries = help_entries_adv();

    // ADV_MATH_A (XROM 22) check
    let mut adv_a_ids: Vec<u16> = entries
        .iter()
        .filter(|e| e.xrom.as_ref().map(|x| x.module_id) == Some(22))
        .map(|e| {
            e.xrom
                .as_ref()
                .unwrap_or_else(|| panic!("entry '{}' missing xrom block", e.op_variant))
                .function_id
        })
        .collect();
    adv_a_ids.sort_unstable();
    let adv_a_unique: Vec<u16> = {
        let mut v = adv_a_ids.clone();
        v.dedup();
        v
    };
    assert_eq!(
        adv_a_ids.len(),
        adv_a_unique.len(),
        "hp41-advantage-functions.json contains duplicate function_id values for ADV_MATH_A (module_id=22)"
    );
    assert_eq!(
        adv_a_ids.first().copied().unwrap_or(0),
        1,
        "hp41-advantage-functions.json ADV_MATH_A function_ids must start at 1"
    );
    assert_eq!(
        adv_a_ids.last().copied().unwrap_or(0),
        63,
        "hp41-advantage-functions.json ADV_MATH_A function_ids must end at 63 (63 ADV CONV+MTRX entries)"
    );
    assert_eq!(
        adv_a_ids.len(),
        63,
        "hp41-advantage-functions.json must have exactly 63 ADV_MATH_A entries (dense 1..=63)"
    );

    // ADV_MATH_B (XROM 24) check
    let mut adv_b_ids: Vec<u16> = entries
        .iter()
        .filter(|e| e.xrom.as_ref().map(|x| x.module_id) == Some(24))
        .map(|e| {
            e.xrom
                .as_ref()
                .unwrap_or_else(|| panic!("entry '{}' missing xrom block", e.op_variant))
                .function_id
        })
        .collect();
    adv_b_ids.sort_unstable();
    let adv_b_unique: Vec<u16> = {
        let mut v = adv_b_ids.clone();
        v.dedup();
        v
    };
    assert_eq!(
        adv_b_ids.len(),
        adv_b_unique.len(),
        "hp41-advantage-functions.json contains duplicate function_id values for ADV_MATH_B (module_id=24)"
    );
    assert_eq!(
        adv_b_ids.first().copied().unwrap_or(0),
        1,
        "hp41-advantage-functions.json ADV_MATH_B function_ids must start at 1"
    );
    assert_eq!(
        adv_b_ids.last().copied().unwrap_or(0),
        51,
        "hp41-advantage-functions.json ADV_MATH_B function_ids must end at 51 (51 ADV MATH+TVM entries)"
    );
    assert_eq!(
        adv_b_ids.len(),
        51,
        "hp41-advantage-functions.json must have exactly 51 ADV_MATH_B entries (dense 1..=51)"
    );
}

/// Test 9: every entry has key_path in XEQ "<name>" form (D-28.6).
#[test]
fn adv_all_key_paths_are_xeq_form() {
    // Catches: wrong key_path format for Advantage Pac entries (D-28.6 invariant).
    // Every Advantage Pac entry key_path must be Some("XEQ \"<MNEMONIC>\"") because
    // Advantage Pac functions are XEQ-by-name only — no dedicated key bindings.
    for entry in help_entries_adv() {
        let key_path = entry.key_path.as_deref().unwrap_or_else(|| {
            panic!(
                "entry '{}' in hp41-advantage-functions.json has no key_path \
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

/// Test 10: no divergence entries — no documented Advantage Pac divergences in Phase 44.
#[test]
fn adv_no_divergence_entries() {
    // Catches: unexpected divergence entries being added without a corresponding
    // docs/hp41-advantage-divergences.md update. As of Phase 44 there are no
    // documented Advantage Pac divergences.
    let with_div: Vec<&str> = help_entries_adv()
        .iter()
        .filter(|e| !e.divergences.is_empty())
        .map(|e| e.display_name.as_str())
        .collect();
    assert!(
        with_div.is_empty(),
        "Expected zero Advantage Pac entries with divergences (Phase 44 has no documented \
         Advantage Pac divergences); found: {with_div:?}"
    );
}

/// Test 11: help_entries_all() chains all five pools (built-ins + Math 1 + Stat 1 + Time + Advantage).
#[test]
fn help_entries_all_returns_five_pools() {
    // Catches: help_entries_all() not chaining all five pools per D-44.1.
    // v2.2 built-in pool: >= 130 entries
    // Math Pac I pool: >= 45 entries
    // Stat 1 Pac pool: == 26 entries
    // Time Pac pool: == 35 entries
    // Advantage Pac pool: == 114 entries
    // Total minimum: 130 + 45 + 26 + 35 + 114 = 350
    let all_count = help_entries_all().count();
    assert!(
        all_count >= 350,
        "help_entries_all() returned {all_count} entries but should include >= 130 v2.2 entries \
         + >= 45 Math 1 entries + 26 Stat 1 entries + 35 Time entries + 114 Advantage entries = \
         >= 350 total (D-44.1 five-pool chain)",
    );
}

/// Test 12: help_overlay_rows() contains at least one Advantage Pac category header.
#[test]
fn help_overlay_rows_includes_adv_pac_sections() {
    // Catches: the 5th pool not feeding into help_overlay_rows().
    // help_overlay_rows() generates "=== {category} ===" header rows for each
    // unique category in help_entries_all(). Since all Advantage Pac entries have
    // categories starting with "Adv ", at least one such header must appear.
    let rows = help_overlay_rows();
    let has_adv_header = rows.iter().any(|r| {
        // Category headers have the form "=== <category> ==="
        r.desc.starts_with("=== Adv ") && r.desc.ends_with(" ===")
    });
    assert!(
        has_adv_header,
        "help_overlay_rows() must contain at least one '=== Adv * ===' category header \
         from the Advantage Pac entries — the 5th pool does not appear to feed into \
         help_overlay_rows(). Check that help_entries_all() chains help_entries_adv() \
         per D-44.1."
    );
}

/// Bonus: Advantage pool op_variants do not collide with prior pools (cross-pool uniqueness).
#[test]
fn adv_help_entries_no_collision_with_other_pools() {
    // Catches: an Advantage mnemonic accidentally shadowing a v2.2 built-in, Math 1,
    // Stat 1, or Time entry in the op_variant space.
    use hp41_cli::help_data::{
        help_entries, help_entries_math1, help_entries_stat1, help_entries_time,
    };
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
    let time: HashSet<&str> = help_entries_time()
        .iter()
        .map(|e| e.op_variant.as_str())
        .collect();
    for entry in help_entries_adv() {
        assert!(
            !v22.contains(entry.op_variant.as_str()),
            "Advantage op_variant '{}' collides with v2.2 built-in pool",
            entry.op_variant
        );
        assert!(
            !math1.contains(entry.op_variant.as_str()),
            "Advantage op_variant '{}' collides with Math Pac I pool",
            entry.op_variant
        );
        assert!(
            !stat1.contains(entry.op_variant.as_str()),
            "Advantage op_variant '{}' collides with Stat 1 Pac pool",
            entry.op_variant
        );
        assert!(
            !time.contains(entry.op_variant.as_str()),
            "Advantage op_variant '{}' collides with Time Pac pool",
            entry.op_variant
        );
    }
}
