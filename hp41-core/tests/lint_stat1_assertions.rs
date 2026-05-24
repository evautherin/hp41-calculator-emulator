// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Wave-1 meta-test: assertion-discipline lint over all Stat 1 Pac test files
//! (Phase 37 Wave 1 / STAT-QUAL-06 / D-37.8).
//!
//! Two `#[test]` gates enforce the Pitfall 14/17 tolerance discipline over the
//! Stat 1 Pac test surface, mirroring `lint_math1_assertions.rs` (Plan 32-01)
//! with one KEY adaptation: **TWO-PASS file collection** spanning:
//!   - Pass 1 — external `tests/stat1_*.rs` integration test files
//!   - Pass 2 — inline `#[cfg(test)]` blocks from `src/ops/stat1/*.rs`
//!
//! **LINT-EXEMPT pre-annotations (D-37.8):**
//! `stat1_rand_determinism.rs` contains 6 exact HpNum `assert_eq!` calls on
//! LCG accumulator values — these are semantically correct (the LCG runs on
//! `rust_decimal`, not `f64`, so exact equality is the intended invariant).
//! Task 1 of this plan pre-annotated each call with `// LINT-EXEMPT: exact
//! HpNum equality via Decimal::new construction — no f64 bridge; ...`.
//! These annotations prevent false positives in this lint gate.
//!
//! ## Gate 1: `no_decimal_assert_eq_in_stat1_tests` (Pitfall 17)
//!
//! `assert_eq!` invocations comparing two BCD `HpNum` / `Decimal` / `f64`
//! values are forbidden — those flow paths can drift across x86 and ARM FPUs.
//! Use `approx::assert_relative_eq!(actual, expected, max_relative = 1e-7)`
//! instead. LINT-EXEMPT annotations opt out specific calls.
//!
//! ## Gate 2: `no_manual_tolerance_pattern_in_stat1_tests` (Pitfall 14)
//!
//! Manual `(actual - expected).abs() < EPSILON` patterns undermine the single-
//! source-of-truth `max_relative` discipline. Use
//! `approx::assert_relative_eq!` so every relative-equality check lives under
//! one canonical tolerance knob per D-27.1.
//!
//! ## Scope
//!
//! - External: `hp41-core/tests/stat1_*.rs`
//! - Inline: `#[cfg(test)]` blocks in `hp41-core/src/ops/stat1/*.rs`
//!
//! Does NOT scan `tests/numerical_accuracy.rs` (the `case!` macro has its own
//! internal `AccuracyCase.tol` bookkeeping). Does NOT scan `tests/math1_*.rs`
//! (covered by `lint_math1_assertions.rs`).
//!
//! ## LINT-EXEMPT annotations
//!
//! A line containing `// LINT-EXEMPT: <reason>` is excluded from both lints.
//! The annotation MUST give a specific rationale (T-32-04: reviewers spot
//! drive-by allowlisting). The `preceding_block_has_lint_exempt` helper uses
//! a 3-line lookback window as established by D-32.1.

#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

/// Token that opts a line out of both lint gates. MUST carry a specific
/// rationale per T-32-04 (the threat is silent allowlisting in a future commit).
const LINT_EXEMPT_TOKEN: &str = "LINT-EXEMPT:";

/// Collect all stat1 test content for linting via TWO-PASS strategy.
///
/// **Pass 1 — external `tests/stat1_*.rs` files:**
/// Full file content (not just `#[cfg(test)]` sections — the whole file is
/// test code).
///
/// **Pass 2 — inline `#[cfg(test)]` blocks in `src/ops/stat1/*.rs`:**
/// For each source file, split on `#[cfg(test)]`, skip the non-test prefix,
/// collect the test content. This captures inline unit tests that call
/// internal functions directly (e.g. `op_sigma_bstat()`).
///
/// Returns `Vec<(PathBuf, String)>` of (path, scanned-content) pairs.
fn collect_stat1_test_content(tests_dir: &Path, src_stat1_dir: &Path) -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();

    // ── Pass 1: external tests/stat1_*.rs ────────────────────────────────────
    let entries = match std::fs::read_dir(tests_dir) {
        Ok(e) => e,
        Err(_) => return files,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !filename.starts_with("stat1_") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        files.push((path, content));
    }

    // ── Pass 2: inline #[cfg(test)] blocks in src/ops/stat1/*.rs ─────────────
    let src_entries = match std::fs::read_dir(src_stat1_dir) {
        Ok(e) => e,
        Err(_) => return files,
    };
    for entry in src_entries.flatten() {
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

    files
}

/// Format a single offender line as `"{filename}:{line_no}: {trimmed_source_line}"`.
/// Pattern matches `lint_math1_assertions.rs::format_offender` per D-37.8.
fn format_offender(path: &Path, line_no: usize, line: &str) -> String {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("<unknown>");
    format!("{}:{}: {}", name, line_no, line.trim())
}

/// Check the preceding lines for a `LINT-EXEMPT:` annotation that applies
/// to the assertion at `lines[idx]`. Walks upward, skipping continuation
/// lines of the same `assert!(...)` macro. Once a comment block is reached,
/// the block is scanned for `LINT-EXEMPT:`.
///
/// The annotation MUST be adjacent to the offender's enclosing item (no
/// blank line OR semicolon-terminated statement between), so future
/// drive-by allowlisting is visible in diff review per T-32-04.
///
/// Copied verbatim from `lint_math1_assertions.rs` — the logic is
/// content-agnostic (D-37.8 inheritance).
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
/// Lines bearing `LINT-EXEMPT:` (inline) OR carrying a `LINT-EXEMPT:` in the
/// preceding contiguous comment block are exempted.
/// Comment lines (`//` / `///` / `//!`) are exempted.
///
/// Copied verbatim from `lint_math1_assertions.rs` — the heuristic is
/// content-agnostic (D-37.8 inheritance).
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
/// preceding contiguous comment block are exempted. Comment lines exempted.
///
/// Copied verbatim from `lint_math1_assertions.rs` — the heuristic is
/// content-agnostic (D-37.8 inheritance).
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

/// Catches: Pitfall 17 — `assert_eq!(decimal, decimal)` on iterated results
/// can drift across x86 and ARM FPUs by the last digit. The BCD layer hides
/// the drift inside `hp41-core`, but `to_f64()` bridges re-expose it. T-32-04:
/// the offender list is reported in full so a reviewer can spot weakening.
///
/// Scans ALL Stat 1 Pac test content: `tests/stat1_*.rs` (Pass 1) AND inline
/// `#[cfg(test)]` blocks in `src/ops/stat1/*.rs` (Pass 2 — per D-37.8).
///
/// The 6 exact HpNum `assert_eq!` calls in `stat1_rand_determinism.rs` are
/// pre-annotated with `// LINT-EXEMPT: exact HpNum equality via Decimal::new
/// construction — no f64 bridge; ...` (Task 1 of this plan) and are exempt.
#[test]
fn no_decimal_assert_eq_in_stat1_tests() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tests_dir = manifest_dir.join("tests");
    let src_stat1_dir = manifest_dir.join("src").join("ops").join("stat1");
    let files = collect_stat1_test_content(&tests_dir, &src_stat1_dir);

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
/// Scans ALL Stat 1 Pac test content: `tests/stat1_*.rs` (Pass 1) AND inline
/// `#[cfg(test)]` blocks in `src/ops/stat1/*.rs` (Pass 2 — per D-37.8).
#[test]
fn no_manual_tolerance_pattern_in_stat1_tests() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tests_dir = manifest_dir.join("tests");
    let src_stat1_dir = manifest_dir.join("src").join("ops").join("stat1");
    let files = collect_stat1_test_content(&tests_dir, &src_stat1_dir);

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
