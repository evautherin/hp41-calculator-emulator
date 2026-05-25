//! Phase 39 Plan 03 Task 2 — Time Pac right-panel exclusion guards
//! (TIME-CLI-04 continuity — post-v3.0 right-panel UX revert).
//!
//! ## Why Time Pac entries must be excluded from the right panel
//!
//! The production behavior in `hp41-cli/src/keys.rs` (`if entry.xrom.is_some() { continue; }`)
//! EXCLUDES every XROM-module function from the right-panel. Time Pac is the
//! third XROM module (after Math 1 / Stat 1), and all its entries carry an
//! `xrom` block with `module_id == 26`. This means all 35 Time Pac functions
//! are automatically excluded from `key_ref_entries()`.
//!
//! Time Pac functions remain fully discoverable via the `?` overlay's
//! "Time Clock", "Time Date Arithmetic", "Time Display", etc. section headers
//! (auto-generated from the fourth JSON pool). The right-panel would balloon
//! by 35 rows if the filter were bypassed — these tests are the regression
//! sentinel for that UX guard.
//!
//! ## Mirrors phase34_key_ref_includes_stat1.rs
//!
//! Two Time-specific sentinels (TIME and CLKT) confirm the xrom-presence filter
//! applies to Time Pac. A third test confirms v2.2 built-in entries are still
//! present after the fourth-pool chain extension (negative regression guard).

#![allow(clippy::unwrap_used)]

use hp41_cli::keys::key_ref_entries;

/// Asserts that `key_ref_entries()` EXCLUDES the Time Pac `XEQ "TIME"` entry.
///
/// TIME is the canonical Time Pac sentinel — it is the most fundamental
/// Time Pac function and has the simplest ASCII mnemonic. A regression that
/// accidentally adds Time Pac entries to the right-panel would surface here.
#[test]
fn key_ref_entries_excludes_time_pac_time() {
    let entries = key_ref_entries();
    let leaked = entries
        .iter()
        .any(|(key_path, display)| key_path == "XEQ \"TIME\"" && display == "TIME");
    assert!(
        !leaked,
        "key_ref_entries() must NOT include (\"XEQ \\\"TIME\\\"\", \"TIME\") — \
         the post-v3.0 `entry.xrom.is_some()` filter in keys.rs excludes \
         XROM-module functions from the right-panel. Time Pac TIME lives in \
         the `?` overlay's \"Time Clock\" section per D-39.12, NOT in \
         the keyboard reference."
    );
}

/// Asserts that `key_ref_entries()` EXCLUDES the Time Pac `XEQ "CLKT"` entry.
///
/// CLKT is the clock-display toggle — a distinct function from TIME that
/// confirms the exclusion applies to all Time Pac entries, not just TIME.
/// This provides defense-in-depth against a heuristic filter based on display
/// name patterns rather than the `xrom.is_some()` field check.
#[test]
fn key_ref_entries_excludes_time_pac_clkt() {
    let entries = key_ref_entries();
    let leaked = entries
        .iter()
        .any(|(key_path, display)| key_path == "XEQ \"CLKT\"" && display == "CLKT");
    assert!(
        !leaked,
        "key_ref_entries() must NOT include (\"XEQ \\\"CLKT\\\"\", \"CLKT\") — \
         the `entry.xrom.is_none()` filter must apply uniformly to every \
         XROM-module function regardless of display_name shape. \
         Time Pac CLKT lives in the `?` overlay, NOT the keyboard reference."
    );
}

/// Asserts that `key_ref_entries()` still contains v2.2 built-in entries after
/// the Plan 39-01 fourth-pool extension.
///
/// Negative-regression test: the merged accessor's four-pool chain must not
/// accidentally drop v2.2 entries (whose `xrom` field is `None`). The v2.2 Add
/// op `("+", "+")` is the canonical sentinel from `phase29_key_ref_includes_math1.rs`
/// and `phase34_key_ref_includes_stat1.rs` — consistent across all three XROM
/// module regression test files for easy auditing.
#[test]
fn key_ref_entries_preserves_v22_entries() {
    let entries = key_ref_entries();
    let found = entries
        .iter()
        .any(|(key_path, display)| key_path == "+" && display == "+");
    assert!(
        found,
        "key_ref_entries() must still include (\"+\", \"+\") after the \
         Plan 39-01 four-pool chain extension — negative-regression guard \
         ensuring v2.2 built-in entries (with `xrom: None`) are NOT accidentally \
         caught by the filter. The filter drops ONLY entries where `xrom.is_some()`."
    );
}
