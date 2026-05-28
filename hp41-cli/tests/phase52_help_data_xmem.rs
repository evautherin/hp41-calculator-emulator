//! Phase 52 Plan 01 — `docs/hp41-xmem-functions.json` smoke tests (XMEM-10 / D-52.1).
//!
//! Mirrors `phase44_help_data_adv.rs` structure exactly; swaps accessor and count
//! targets (== 8 unique Op variants, all "Extended Memory" category).
//!
//! Test 3 (`help_entries_all_returns_six_pools`) asserts the merged chain length is
//! `>= 130 + 45 + 26 + 35 + 114 + 8 = 358` (v2.2 + Math 1 + Stat 1 + Time + Adv + X-MEM).
//!
//! Test 4 (`help_overlay_rows_includes_xmem_section`) verifies that the sixth pool
//! feeds into help_overlay_rows() and the "Extended Memory" category header appears.

#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use hp41_cli::help_data::{help_entries_all, help_entries_xmem, help_overlay_rows};

/// Test 1: hard-build-blocker exercised on success path.
#[test]
fn xmem_help_entries_is_not_empty() {
    // Catches: hard-build-blocker not firing on malformed JSON (D-25.17 / D-52.1 sixth-pool)
    let entries = help_entries_xmem();
    assert!(
        !entries.is_empty(),
        "help_entries_xmem() must return a non-empty slice — \
         docs/hp41-xmem-functions.json may be empty or malformed (D-52.1)"
    );
}

/// Test 2: exact count — X-MEM pool is feature-frozen at 8 entries for v4.0.
#[test]
fn xmem_help_entries_count_meets_8_target() {
    // Exact == 8, not >= 8: X-MEM is feature-frozen per v4.0 / Phase 52 scope.
    // 8 HP-41CX Extended Memory OS built-in ops (EMDIR, EMROOM, SAVEP, GETP, SAVED, GETD, EMREG, SAVERX).
    // If a future X-MEM op is added without updating this test, it fails loudly.
    let entries = help_entries_xmem();
    assert_eq!(
        entries.len(),
        8,
        "help_entries_xmem().len() = {} — must be exactly 8 (X-MEM Phase 52 feature-frozen)",
        entries.len()
    );
}

/// Test 3: help_entries_all() chains all six pools (built-ins + Math 1 + Stat 1 + Time + Advantage + X-MEM).
#[test]
fn help_entries_all_returns_six_pools() {
    // Catches: help_entries_all() not chaining all six pools per D-52.1.
    // v2.2 built-in pool: >= 130 entries
    // Math Pac I pool: >= 45 entries
    // Stat 1 Pac pool: == 26 entries
    // Time Pac pool: == 35 entries
    // Advantage Pac pool: == 114 entries
    // X-MEM pool: == 8 entries
    // Total minimum: 130 + 45 + 26 + 35 + 114 + 8 = 358
    let all: Vec<_> = help_entries_all().collect();
    assert!(
        all.len() >= 358,
        "help_entries_all() has {} entries — expected >= 358 \
         (v2.2 + Math1 + Stat1 + Time + Adv + X-MEM)",
        all.len()
    );
}

/// Test 4: help_overlay_rows() contains an "Extended Memory" category header row.
#[test]
fn help_overlay_rows_includes_xmem_section() {
    // Catches: the 6th pool not feeding into help_overlay_rows().
    // help_overlay_rows() generates "=== {category} ===" header rows for each
    // unique category in help_entries_all(). Since all X-MEM entries have
    // category "Extended Memory", at least one such header must appear.
    let rows = help_overlay_rows();
    let has_xmem_header = rows.iter().any(|r| r.desc.contains("Extended Memory"));
    assert!(
        has_xmem_header,
        "help_overlay_rows() must include 'Extended Memory' category header from X-MEM pool"
    );
}

/// Test 5: every X-MEM entry has category "Extended Memory" and a non-null key_path.
#[test]
fn xmem_every_entry_has_category_and_key_path() {
    // Catches: wrong category (breaks overlay sectioning D-52.1) or missing key_path
    // (breaks GUI helpOverlayRows() filter per key_path !== null convention).
    let entries = help_entries_xmem();
    for entry in entries {
        assert_eq!(
            entry.category.as_str(),
            "Extended Memory",
            "entry '{}' in docs/hp41-xmem-functions.json has category '{}' — \
             must be 'Extended Memory' (D-52.1 overlay sectioning)",
            entry.op_variant,
            entry.category
        );
        assert!(
            entry.key_path.is_some(),
            "entry '{}' in docs/hp41-xmem-functions.json has no key_path — \
             X-MEM ops must have XEQ-by-name key_path (D-52.4)",
            entry.op_variant
        );
    }
}

/// Bonus: op_variant set matches the 8 expected names exactly (op_variant drift-catch).
#[test]
fn xmem_op_variants_match_expected_set() {
    let expected: HashSet<&str> = [
        "EmDir", "EmRoom", "SaveP", "GetP", "SaveD", "GetD", "EmReg", "SaveRx",
    ]
    .iter()
    .copied()
    .collect();

    let actual: HashSet<&str> = help_entries_xmem()
        .iter()
        .map(|e| e.op_variant.as_str())
        .collect();

    assert_eq!(
        actual,
        expected,
        "docs/hp41-xmem-functions.json op_variant set mismatch.\n\
         Missing: {:?}\n\
         Extra: {:?}",
        expected.difference(&actual).collect::<Vec<_>>(),
        actual.difference(&expected).collect::<Vec<_>>()
    );
}
