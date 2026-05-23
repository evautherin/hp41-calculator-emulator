//! Phase 34 / Plan 34-02 Task 5 — Stat 1 Pac right-panel exclusion guards
//! (STAT-CLI-04 continuity — post-v3.0 right-panel UX revert).
//!
//! ## Why this file inverts CONTEXT.md's literal wording
//!
//! CONTEXT.md §"In scope" item 10 says "Stat 1 entries with non-null
//! `key_path` appear in `keys::key_ref_entries()`". That wording was
//! authored before the post-v3.0 right-panel UX revert was discovered (see
//! `phase29_key_ref_includes_math1.rs` header comment for the full
//! revert rationale: ~45 XEQ rows crowded out the keyboard reference).
//!
//! The actual production behavior — confirmed by reading
//! `hp41-cli/src/keys.rs:437` (`if entry.xrom.is_some() { continue; }`) —
//! EXCLUDES every XROM-module function from the right-panel. Phase 34's
//! correct continuation of Phase 29's inverted Math 1 tests is to assert
//! the SAME exclusion for Stat 1. Stat 1 functions remain fully
//! discoverable via the `?` overlay's `Stat 1 Pac (XROM 2)` section per
//! D-34.5 — the discoverability story is intact, just routed through the
//! overlay rather than the right-panel.
//!
//! ## Test inversion rationale
//!
//! Without the `entry.xrom.is_none()` filter, the right-panel would balloon
//! by an additional 26 rows on top of the ~45 from Math 1 (= 71 XEQ rows
//! crowding a ~15–25-row keyboard reference). The filter is a load-bearing
//! UX guard, and these tests are its regression sentinel.

#![allow(clippy::unwrap_used)]

use hp41_cli::keys::key_ref_entries;

/// Asserts that `key_ref_entries()` EXCLUDES the Stat 1 Pac `XEQ "ΣNORMD"` entry.
///
/// ΣNORMD is the canonical Stat 1 sentinel (ROADMAP Phase 33 SC §2 uses
/// `XEQ "ΣNORMD"` as the integration smoke target; Phase 37 STAT-QUAL-11
/// uses ΣNORMD for the WebdriverIO E2E smoke). A regression that adds
/// Σ-prefixed entries back to the right-panel surfaces here AND in the
/// future E2E surface — defense in depth.
#[test]
fn key_ref_entries_excludes_stat1_sigma_normd() {
    let entries = key_ref_entries();
    let leaked = entries.iter().any(|(key_path, display)| {
        key_path == "XEQ \"\u{03A3}NORMD\"" && display == "\u{03A3}NORMD"
    });
    assert!(
        !leaked,
        "key_ref_entries() must NOT include (\"XEQ \\\"\u{03A3}NORMD\\\"\", \"\u{03A3}NORMD\") — \
         the post-v3.0 `entry.xrom.is_none()` filter in keys.rs:437 excludes \
         XROM-module functions from the right-panel. Stat 1 functions live in \
         the `?` overlay's \"Stat 1 Pac (XROM 2)\" section per D-34.5, NOT in \
         the keyboard reference."
    );
}

/// Asserts that `key_ref_entries()` EXCLUDES the Stat 1 Pac `XEQ "RAND"` entry.
///
/// RAND is an ASCII-named Stat 1 entry (no Σ prefix), confirming the
/// exclusion is xrom-presence-based and NOT display-name-pattern-based.
/// If a future bug introduced a heuristic like `display_name.starts_with("Σ")`
/// in the filter, RAND would leak — this test would catch that drift.
#[test]
fn key_ref_entries_excludes_stat1_rand() {
    let entries = key_ref_entries();
    let leaked = entries
        .iter()
        .any(|(key_path, display)| key_path == "XEQ \"RAND\"" && display == "RAND");
    assert!(
        !leaked,
        "key_ref_entries() must NOT include (\"XEQ \\\"RAND\\\"\", \"RAND\") — \
         the post-v3.0 `entry.xrom.is_none()` filter must apply uniformly to \
         every XROM-module function regardless of display_name shape (Σ-prefixed \
         or ASCII). Stat 1 RAND lives in the `?` overlay, NOT the keyboard reference."
    );
}

/// Asserts that `key_ref_entries()` still contains v2.2 built-in entries after
/// the Plan 34-01 third-pool extension.
///
/// Negative-regression test: the merged accessor's three-pool chain must not
/// accidentally drop v2.2 entries (whose `xrom` field is `None`). The v2.2 Add
/// op (`("+", "+")`) is the canonical sentinel from `phase29_key_ref_includes_math1.rs`.
#[test]
fn key_ref_entries_preserves_v22_entries() {
    let entries = key_ref_entries();
    let found = entries
        .iter()
        .any(|(key_path, display)| key_path == "+" && display == "+");
    assert!(
        found,
        "key_ref_entries() must still include (\"+\", \"+\") after the \
         Plan 34-01 three-pool chain extension — negative-regression guard \
         ensuring v2.2 built-in entries (with `xrom: None`) are NOT accidentally \
         caught by the filter. The filter drops ONLY entries where `xrom.is_some()`."
    );
}
