// Algorithm independently re-derived from HP module Owner's Manuals;
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Phase 47 Plan 01 / ADV-QUAL-02 — Unified assertion-discipline lint over all
//! five XROM module test surfaces (Math 1, Stat 1, Time, ADV MATH A, ADV MATH B).
//!
//! **Extended in Phase 47 (ADV-QUAL-02):**
//! Extended from Phase 42 / D-42.1 (3 modules) to cover the Advantage Pac
//! test surfaces: external `tests/adv_*.rs` files and inline `#[cfg(test)]`
//! blocks from `src/ops/advantage/*.rs`.
//!
//! **Replaces (D-42.14):**
//! - `lint_math1_assertions.rs` (Phase 32 Plan 01 / T-32-04)
//! - `lint_stat1_assertions.rs` (Phase 37 Wave 1 / STAT-QUAL-06 / D-37.8)
//!
//! **Unification rationale (D-42.1):**
//! With three XROM modules, per-module lint files proliferate. A single unified
//! file enforces Pitfall 14/17 discipline across all module test trees and makes
//! v3.3 Advantage Pac integration free.
//!
//! **Two `#[test]` gates (Pitfall 14/17 T-32-04 rationale):**
//!
//! 1. **`no_decimal_assert_eq_in_xrom_tests`** (Pitfall 17): `assert_eq!`
//!    invocations comparing two BCD `HpNum` / `Decimal` / `f64` values are
//!    forbidden — those flow paths can drift across x86 and ARM FPUs by the
//!    last digit (the BCD layer hides the drift inside `hp41-core` but the
//!    `to_f64()` bridge re-exposes it). Use
//!    `approx::assert_relative_eq!(actual, expected, max_relative = 1e-7)` instead.
//!
//! 2. **`no_manual_tolerance_pattern_in_xrom_tests`** (Pitfall 14): manual
//!    `(actual - expected).abs() < EPSILON` patterns undermine the
//!    single-source-of-truth `max_relative = 1e-7` discipline. Use
//!    `approx::assert_relative_eq!` so every relative-equality check lives
//!    under one canonical tolerance knob (D-27.1).
//!
//! **Scan scope (five-module collection):**
//!
//! - **Math 1:** external `tests/math1_*.rs` files (full content — entire file
//!   is test code). Does NOT include `tests/numerical_accuracy.rs` (the `case!`
//!   macro has its own `AccuracyCase.tol` bookkeeping).
//!
//! - **Stat 1:** external `tests/stat1_*.rs` files (Pass 1) AND inline
//!   `#[cfg(test)]` blocks from `src/ops/stat1/*.rs` (Pass 2). The 6 exact
//!   HpNum `assert_eq!` calls in `stat1_rand_determinism.rs` are pre-annotated
//!   with `// LINT-EXEMPT:` — they use Decimal arithmetic, not f64 bridges.
//!
//! - **Time:** external `tests/time_*.rs` files if any (Pass 1) AND inline
//!   `#[cfg(test)]` blocks from `src/ops/time/*.rs` (Pass 2). Time has 209
//!   inline tests as the primary test surface at Wave 1.
//!
//! - **ADV MATH A + ADV MATH B:** external `tests/adv_*.rs` files (Pass 1) AND
//!   inline `#[cfg(test)]` blocks from `src/ops/advantage/*.rs` (Pass 2). The
//!   Advantage Pac has 322 inline tests as the primary test surface.
//!
//! **LINT-EXEMPT annotations:**
//!
//! A line containing `// LINT-EXEMPT: <reason>` is excluded from both lints.
//! The annotation MUST give a specific rationale (T-32-04: reviewers spot
//! drive-by allowlisting). The `preceding_block_has_lint_exempt` helper uses
//! an adjacent-comment lookback pattern.
//!
//! **Multi-line assert_eq detection (WR-02):**
//! Both single-line and multi-line `assert_eq!(decimal, decimal)` invocations
//! are detected. Lookahead window = current line + 3 following lines.

#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

/// Token that opts a line out of both lint gates. MUST carry a specific
/// rationale per T-32-04 (the threat is silent allowlisting in a future commit).
const LINT_EXEMPT_TOKEN: &str = "LINT-EXEMPT:";

// ── Unified test-content collector ────────────────────────────────────────────

/// Collect all XROM module test content for linting via five-module strategy.
///
/// **Module 1 — Math 1:** external `tests/math1_*.rs` files (full file content).
///
/// **Module 2 — Stat 1:** external `tests/stat1_*.rs` files (Pass 1) AND inline
/// `#[cfg(test)]` blocks from `src/ops/stat1/*.rs` (Pass 2).
///
/// **Module 3 — Time:** external `tests/time_*.rs` files if any (Pass 1) AND
/// inline `#[cfg(test)]` blocks from `src/ops/time/*.rs` (Pass 2).
///
/// **Module 4+5 — ADV MATH A + ADV MATH B:** external `tests/adv_*.rs` files
/// (Pass 1) AND inline `#[cfg(test)]` blocks from `src/ops/advantage/*.rs`
/// (Pass 2). Both ADV modules share the same source directory.
///
/// Returns `Vec<(PathBuf, String)>` of (path, scanned-content) pairs for all
/// collected test surfaces.
fn collect_xrom_test_content(tests_dir: &Path, manifest_dir: &Path) -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    let src_ops_dir = manifest_dir.join("src").join("ops");

    // ── Math 1: external tests/math1_*.rs (Pass 1 only) ─────────────────────
    collect_external_files(tests_dir, "math1_", None, &mut files);

    // ── Stat 1: external tests/stat1_*.rs (Pass 1) + inline src/ops/stat1/*.rs (Pass 2)
    collect_external_files(tests_dir, "stat1_", None, &mut files);
    collect_inline_cfg_test_blocks(&src_ops_dir.join("stat1"), &mut files);

    // ── Time: external tests/time_*.rs (Pass 1) + inline src/ops/time/*.rs (Pass 2)
    collect_external_files(tests_dir, "time_", None, &mut files);
    collect_inline_cfg_test_blocks(&src_ops_dir.join("time"), &mut files);

    // ── Advantage (ADV MATH A + ADV MATH B): external tests/adv_*.rs (Pass 1)
    //    + inline src/ops/advantage/*.rs (Pass 2).
    //    Both XROM modules share the same source directory and test prefix.
    collect_external_files(tests_dir, "adv_", None, &mut files);
    collect_inline_cfg_test_blocks(&src_ops_dir.join("advantage"), &mut files);

    files
}

/// Collect external test files with a given filename prefix.
/// Full file content is collected (the whole file is test code in external tests/).
fn collect_external_files(
    tests_dir: &Path,
    prefix: &str,
    _extra_file: Option<&str>,
    files: &mut Vec<(PathBuf, String)>,
) {
    let entries = match std::fs::read_dir(tests_dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !filename.starts_with(prefix) {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        files.push((path, content));
    }
}

/// Collect inline `#[cfg(test)]` blocks from all `*.rs` files in a source directory.
/// Only the test section content is collected (non-test prefix is skipped).
fn collect_inline_cfg_test_blocks(src_dir: &Path, files: &mut Vec<(PathBuf, String)>) {
    let entries = match std::fs::read_dir(src_dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        // Extract only the #[cfg(test)] sections (skip the non-test prefix).
        // Multiple #[cfg(test)] blocks in a single file are each captured.
        let test_sections: Vec<&str> = content.split("#[cfg(test)]").skip(1).collect();
        if !test_sections.is_empty() {
            // Rejoin sections — each is a test module; combined content is the
            // full test surface for linting purposes.
            let test_content = test_sections.join("\n#[cfg(test)]");
            files.push((path, test_content));
        }
    }
}

// ── Lint helper functions (verbatim from lint_math1_assertions.rs, T-32-04) ───

/// Format a single offender line as `"{filename}:{line_no}: {trimmed_source_line}"`.
fn format_offender(path: &Path, line_no: usize, line: &str) -> String {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("<unknown>");
    format!("{}:{}: {}", name, line_no, line.trim())
}

/// Check the preceding lines for a `LINT-EXEMPT:` annotation that applies
/// to the assertion at `lines[idx]`. Walks upward, skipping continuation
/// lines of the same `assert!(...)` macro (lines that look like part of a
/// multi-line macro invocation). Once a comment block is reached, the block
/// is scanned for `LINT-EXEMPT:`.
///
/// The annotation MUST be adjacent to the offender's enclosing item (no
/// blank line OR semicolon-terminated statement between), so future
/// drive-by allowlisting is visible in diff review per T-32-04.
fn preceding_block_has_lint_exempt(lines: &[&str], idx: usize) -> bool {
    let mut i = idx;
    // Phase 1: walk upward past continuation lines of the same macro/expression.
    // Stop when we hit a comment, blank line, or terminating ';' / '}'.
    while i > 0 {
        i -= 1;
        let trimmed = lines[i].trim_start();
        if trimmed.is_empty() {
            return false;
        }
        if trimmed.starts_with("//") {
            // Reached the comment block — fall through to Phase 2.
            break;
        }
        let trimmed_end = lines[i].trim_end();
        if trimmed_end.ends_with(';') || trimmed_end.ends_with('}') {
            // Reached the previous statement; LINT-EXEMPT must precede it.
            return false;
        }
        // Otherwise: continuation line of the offender's enclosing
        // `assert!(...)` macro — keep walking upward.
    }
    // Phase 2: scan the contiguous comment block for LINT-EXEMPT.
    loop {
        let trimmed = lines[i].trim_start();
        if !trimmed.starts_with("//") {
            return false;
        }
        if trimmed.contains(LINT_EXEMPT_TOKEN) {
            return true;
        }
        if i == 0 {
            return false;
        }
        i -= 1;
        if lines[i].trim().is_empty() {
            return false;
        }
    }
}

/// Heuristic for forbidden `assert_eq!(decimal, decimal)` lines per Pitfall 17.
///
/// A line is forbidden iff it contains `assert_eq!` AND the line itself OR up
/// to 3 following lines (multi-line invocation lookahead) contain any of:
/// `.to_f64()`, `HpNum`, `Decimal`, `.inner()`.
///
/// **Multi-line detection (WR-02):** both single-line and multi-line
/// `assert_eq!(decimal, decimal)` invocations are detected. The lookahead
/// window is: current line + 3 following lines joined with `\n`.
///
/// Lines bearing `LINT-EXEMPT:` (inline) OR carrying a `LINT-EXEMPT:` in the
/// preceding contiguous comment block are exempted. Comment lines are exempted.
fn line_is_forbidden_assert_eq(line: &str, lines: &[&str], idx: usize) -> bool {
    if line.contains(LINT_EXEMPT_TOKEN) {
        return false;
    }
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return false;
    }
    if !line.contains("assert_eq!") {
        return false;
    }
    // Check the line itself AND up to 3 following lines (multi-line invocation
    // lookahead). take(4) = current line + 3 following.
    let next_3: String = lines
        .iter()
        .skip(idx)
        .take(4)
        .copied()
        .collect::<Vec<_>>()
        .join("\n");
    let is_decimal = next_3.contains(".to_f64()")
        || next_3.contains("HpNum")
        || next_3.contains("Decimal")
        || next_3.contains(".inner()");
    if !is_decimal {
        return false;
    }
    if preceding_block_has_lint_exempt(lines, idx) {
        return false;
    }
    true
}

/// Heuristic for forbidden manual-tolerance `(a - b).abs() < EPSILON` lines
/// per Pitfall 14.
///
/// A line is forbidden iff it matches the textual pattern `).abs() <` AND
/// contains a ` - ` inside parentheses on the same line.
///
/// Lines bearing `LINT-EXEMPT:` (inline) OR carrying a `LINT-EXEMPT:` in the
/// preceding contiguous comment block are exempted. Comment lines are exempted.
fn line_is_forbidden_manual_tolerance(line: &str, lines: &[&str], idx: usize) -> bool {
    if line.contains(LINT_EXEMPT_TOKEN) {
        return false;
    }
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return false;
    }
    // Look for ").abs() <" anchor.
    let Some(abs_idx) = line.find(").abs() <") else {
        return false;
    };
    // Look for a "-" inside parentheses before the anchor (i.e., the
    // `(a - b)` pattern, not `(x).abs() < ...` which uses single-arg abs).
    let prefix = &line[..abs_idx];
    let Some(open_idx) = prefix.rfind('(') else {
        return false;
    };
    let inner = &prefix[open_idx + 1..];
    // Require a ' - ' (with surrounding spaces) so we don't match negative
    // literals like `(-1.0).abs()`.
    if !inner.contains(" - ") {
        return false;
    }
    if preceding_block_has_lint_exempt(lines, idx) {
        return false;
    }
    true
}

// ── Lint gate tests ───────────────────────────────────────────────────────────

/// Catches: Pitfall 17 — `assert_eq!(decimal, decimal)` on iterated results
/// can drift across x86 and ARM FPUs by the last digit. The BCD layer hides
/// the drift inside `hp41-core`, but `to_f64()` bridges re-expose it. T-32-04:
/// the offender list is reported in full so a reviewer can spot weakening.
///
/// Scans ALL five XROM module test surfaces (D-42.1 unification + Phase 47
/// ADV-QUAL-02 extension):
/// - Math 1:     `tests/math1_*.rs` (external)
/// - Stat 1:     `tests/stat1_*.rs` (external) + `src/ops/stat1/*.rs` inline cfg(test)
/// - Time:       `tests/time_*.rs` (external) + `src/ops/time/*.rs` inline cfg(test)
/// - ADV MATH A+B: `tests/adv_*.rs` (external) + `src/ops/advantage/*.rs` inline cfg(test)
#[test]
fn no_decimal_assert_eq_in_xrom_tests() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tests_dir = manifest_dir.join("tests");
    let files = collect_xrom_test_content(&tests_dir, &manifest_dir);

    let mut offenders: Vec<String> = Vec::new();
    for (path, content) in &files {
        let lines: Vec<&str> = content.lines().collect();
        for (idx, line) in lines.iter().enumerate() {
            if line_is_forbidden_assert_eq(line, &lines, idx) {
                offenders.push(format_offender(path, idx + 1, line));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "Pitfall 17 violation — `assert_eq!(decimal, decimal)` on iterated \
         results (use `approx::assert_relative_eq!(actual, expected, \
         max_relative = 1e-7)` or add a `// LINT-EXEMPT: <reason>` annotation):\n{}",
        offenders.join("\n")
    );
}

/// Catches: Pitfall 14 — manual `(a - b).abs() < EPSILON` patterns undermine
/// the single-source-of-truth `max_relative = 1e-7` discipline. T-32-04: the
/// offender list is reported in full so a reviewer can spot weakening.
///
/// Scans ALL five XROM module test surfaces (D-42.1 unification + Phase 47
/// ADV-QUAL-02 extension):
/// - Math 1:     `tests/math1_*.rs` (external)
/// - Stat 1:     `tests/stat1_*.rs` (external) + `src/ops/stat1/*.rs` inline cfg(test)
/// - Time:       `tests/time_*.rs` (external) + `src/ops/time/*.rs` inline cfg(test)
/// - ADV MATH A+B: `tests/adv_*.rs` (external) + `src/ops/advantage/*.rs` inline cfg(test)
#[test]
fn no_manual_tolerance_pattern_in_xrom_tests() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tests_dir = manifest_dir.join("tests");
    let files = collect_xrom_test_content(&tests_dir, &manifest_dir);

    let mut offenders: Vec<String> = Vec::new();
    for (path, content) in &files {
        let lines: Vec<&str> = content.lines().collect();
        for (idx, line) in lines.iter().enumerate() {
            if line_is_forbidden_manual_tolerance(line, &lines, idx) {
                offenders.push(format_offender(path, idx + 1, line));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "Pitfall 14 violation — manual `(a - b).abs() < EPSILON` (use \
         `approx::assert_relative_eq!(actual, expected, max_relative = 1e-7)` \
         or add a `// LINT-EXEMPT: <reason>` annotation):\n{}",
        offenders.join("\n")
    );
}
