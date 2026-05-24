// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Wave-1 meta-test: each Stat 1 Pac `Op` variant registered in `stat1_resolve`
//! must have at least 5 test functions mentioning it (Pitfall 16 guard /
//! STAT-QUAL-07 / D-37.7).
//!
//! Mirrors the v3.0 `math1_op_test_count.rs` pattern (Plan 32-01) with two KEY
//! adaptations:
//!
//! 1. **TWO-PASS scan** over:
//!    - Pass 1 — external `tests/stat1_*.rs` integration test files
//!    - Pass 2 — inline `#[cfg(test)]` modules in `src/ops/stat1/*.rs`
//!
//! 2. **DUAL token matching** — stat1 inline tests call internal functions
//!    directly (e.g. `op_sigma_bstat()`) rather than going through
//!    `dispatch(&mut state, Op::SigmaBstat)`. Each variant is matched by EITHER
//!    the `Op::<VariantName>` enum token (external tests) OR the `op_<snake>`
//!    function call pattern (inline tests). The snake-case conversion is:
//!    PascalCase → snake_case prepended with `op_` (e.g., `SigmaBstat` →
//!    `op_sigma_bstat`, `SigmaPolypWorkflow` → `op_sigma_polyp_workflow`).
//!
//! This two-pass / dual-token strategy captures the 189 inline unit tests
//! distributed across the 12 source files in `src/ops/stat1/`, which do NOT
//! appear in external `tests/` files and do NOT use `Op::` enum references.
//!
//! **Scan scope (Pass 1):** `hp41-core/tests/stat1_*.rs`
//! **Scan scope (Pass 2):** inline `#[cfg(test)]` blocks from `hp41-core/src/ops/stat1/*.rs`
//! **Variant source:** `fn stat1_resolve` scope in `hp41-core/src/ops/math1/xrom.rs`
//!
//! **Word-boundary matching (WR-03):** `line_mentions_variant_or_fn` uses
//! word-boundary matching to prevent `Op::SigmaBstat` from matching
//! `Op::SigmaBstg`, and `op_sigma_bstat` from matching `op_sigma_bstg`.
//!
//! **Expected outcome:** 26 variants detected. If any variant falls below 5
//! mentions, the gate reports the specific variant(s) — surfacing coverage gaps
//! for Wave 2 plans to address.

#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

/// Scan `hp41-core/src/ops/math1/xrom.rs` for Stat 1 Pac variant names
/// registered in `stat1_resolve()`. Returns a list of variant name strings
/// (e.g. "SigmaBstat", "SigmaBstg", "Rand", "Seed").
///
/// Detection: lines matching `Some(Op::` inside the `fn stat1_resolve` scope.
/// Brace-depth tracking ensures the scan is scoped to `stat1_resolve` only —
/// it does NOT bleed into `math1_resolve` or any other fn (which would
/// produce 45 Math Pac I variants instead of 26 Stat 1 Pac variants).
fn collect_stat1_variant_names() -> Vec<String> {
    let xrom_src = include_str!("../src/ops/math1/xrom.rs");
    let mut variants = Vec::new();

    let mut in_stat1_resolve = false;
    let mut brace_depth: i32 = 0;

    for line in xrom_src.lines() {
        let trimmed = line.trim();

        if !in_stat1_resolve {
            if trimmed.contains("fn stat1_resolve") {
                in_stat1_resolve = true;
                brace_depth =
                    line.matches('{').count() as i32 - line.matches('}').count() as i32;
            }
            continue;
        }

        // Inside stat1_resolve: maintain brace depth.
        brace_depth += line.matches('{').count() as i32;
        brace_depth -= line.matches('}').count() as i32;

        // Detect lines like: `"\u{03A3}BSTAT" => Some(Op::SigmaBstat),`
        if trimmed.contains("=> Some(Op::") && !trimmed.starts_with("//") {
            if let Some(after_op) = trimmed.split("Some(Op::").nth(1) {
                // Extract variant name (up to ')' or ',')
                let variant_name: String = after_op
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !variant_name.is_empty() {
                    variants.push(variant_name);
                }
            }
        }

        if brace_depth <= 0 {
            in_stat1_resolve = false;
        }
    }
    variants
}

/// Convert a PascalCase `Op` variant name to the `op_<snake_case>` function
/// name used by stat1 inline tests.
///
/// Examples:
/// - `SigmaBstat`        → `op_sigma_bstat`
/// - `SigmaPolypWorkflow` → `op_sigma_polyp_workflow`
/// - `SigmaNormdWorkflow` → `op_sigma_normd_workflow`
/// - `Rand`               → `op_rand`
/// - `Seed`               → `op_seed`
///
/// Algorithm: insert `_` before each uppercase letter that follows a lowercase
/// letter or digit, then lowercaseall, then prepend `op_`.
fn variant_to_fn_name(variant: &str) -> String {
    let mut snake = String::from("op_");
    let chars: Vec<char> = variant.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() && i > 0 {
            // Insert underscore before uppercase letter, unless previous was also uppercase
            // and this isn't a transition from upper to lower (e.g. MLR stays mlr, not m_l_r)
            let prev = chars[i - 1];
            // Transition from lower/digit to upper: always insert _
            if prev.is_lowercase() || prev.is_ascii_digit() {
                snake.push('_');
            } else {
                // Transition from upper to upper: check if NEXT char is lower
                // (e.g. "MLRxy" → "mlr_xy" would split MLR→mlr; but we treat
                //  a run of uppercase as one word unit). Insert _ if next is lowercase.
                if let Some(&next) = chars.get(i + 1) {
                    if next.is_lowercase() {
                        snake.push('_');
                    }
                }
            }
        }
        snake.push(c.to_ascii_lowercase());
    }
    snake
}

/// Return true if `line` mentions the `Op` variant by EITHER its enum token
/// `Op::<VariantName>` OR its internal function name `op_<snake>`.
///
/// Uses word-boundary checking: the character after the token must NOT be
/// alphanumeric or `_` (WR-03 — prevents `op_sigma_bstat` from matching
/// `op_sigma_bstg`, and `Op::Sigma` from matching `Op::SigmaBstat`).
///
/// Comment lines are pre-filtered before calling this helper.
fn line_mentions_variant_or_fn(line: &str, variant_name: &str, fn_name: &str) -> bool {
    // Check for Op::VariantName token (used by external tests that go through dispatch)
    let op_token = format!("Op::{variant_name}");
    if check_word_boundary(line, &op_token) {
        return true;
    }
    // Check for op_<snake_case> token (used by inline tests that call directly)
    if check_word_boundary(line, fn_name) {
        return true;
    }
    false
}

/// Return true if `text` appears in `line` as a whole-word token (i.e., the
/// character immediately after `text` is NOT alphanumeric or `_`).
fn check_word_boundary(line: &str, token: &str) -> bool {
    let mut search_start = 0;
    while let Some(pos) = line[search_start..].find(token) {
        let abs_pos = search_start + pos;
        let after_pos = abs_pos + token.len();
        let boundary_ok = match line[after_pos..].chars().next() {
            None => true,
            Some(c) => !c.is_alphanumeric() && c != '_',
        };
        if boundary_ok {
            return true;
        }
        search_start = abs_pos + 1;
    }
    false
}

/// Count how many distinct `#[test]` function blocks across BOTH passes contain
/// at least one mention of the variant (by enum token or by function name).
///
/// **Pass 1 — external integration tests:**
/// Glob `hp41-core/tests/stat1_*.rs` files, split each on `#[test]` markers,
/// count function blocks that mention the variant.
///
/// **Pass 2 — inline unit tests in source files:**
/// For each `hp41-core/src/ops/stat1/*.rs` file, extract `#[cfg(test)]` block
/// content by splitting on `#[cfg(test)]` (taking the tail), then split that
/// content on `#[test]` and count mentioning blocks. Inline tests call internal
/// functions directly (e.g. `op_sigma_bstat()`), not `dispatch(Op::SigmaBstat)`.
///
/// This two-pass / dual-token strategy is required because Stat 1 Pac places
/// 189 tests inline alongside implementation, using direct fn calls.
fn count_stat1_test_mentions(
    variant_name: &str,
    fn_name: &str,
    tests_dir: &Path,
    src_stat1_dir: &Path,
) -> usize {
    let mut total_fn_count = 0;

    // ── Pass 1: external tests/stat1_*.rs ────────────────────────────────────
    let entries = match std::fs::read_dir(tests_dir) {
        Ok(e) => e,
        Err(_) => return 0,
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
        let test_slices: Vec<&str> = content.split("#[test]").skip(1).collect();
        for slice in test_slices {
            let found = slice.lines().any(|line| {
                let trimmed = line.trim();
                !trimmed.starts_with("//")
                    && line_mentions_variant_or_fn(line, variant_name, fn_name)
            });
            if found {
                total_fn_count += 1;
            }
        }
    }

    // ── Pass 2: inline #[cfg(test)] blocks in src/ops/stat1/*.rs ─────────────
    let src_entries = match std::fs::read_dir(src_stat1_dir) {
        Ok(e) => e,
        Err(_) => return total_fn_count,
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
        // Extract the content of the #[cfg(test)] block(s). Each split on
        // `#[cfg(test)]` gives one non-test prefix + one-or-more test sections.
        let cfg_test_sections: Vec<&str> = content.split("#[cfg(test)]").skip(1).collect();
        for section in cfg_test_sections {
            // Within the cfg(test) section, split by #[test] to get per-fn slices.
            let test_slices: Vec<&str> = section.split("#[test]").skip(1).collect();
            for slice in test_slices {
                let found = slice.lines().any(|line| {
                    let trimmed = line.trim();
                    !trimmed.starts_with("//")
                        && line_mentions_variant_or_fn(line, variant_name, fn_name)
                });
                if found {
                    total_fn_count += 1;
                }
            }
        }
    }

    total_fn_count
}

/// Meta-test: every Stat 1 Pac Op variant registered in `stat1_resolve` must
/// have ≥ 5 test function references across external `tests/stat1_*.rs` AND
/// inline `src/ops/stat1/*.rs` `#[cfg(test)]` modules.
///
/// **Expected at Wave 1 install:** 26 variants detected. Most should meet the
/// threshold given 189 inline tests + 2 external test files. Variants below
/// threshold surface here for Wave 2 coverage-gap plans to address
/// (STAT-QUAL-07 design — report, don't mask).
///
/// Catches: Pitfall 16 — Op variants with fewer than 5 tests risk missing edge
/// cases that the Phase 37 coverage gate would otherwise catch.
#[test]
fn each_stat1_op_has_at_least_5_tests() {
    let variants = collect_stat1_variant_names();

    // CARGO_MANIFEST_DIR is set at compile time to the hp41-core package dir.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tests_dir_buf = manifest_dir.join("tests");
    let tests_dir = tests_dir_buf.as_path();
    let src_stat1_dir_buf = manifest_dir.join("src").join("ops").join("stat1");
    let src_stat1_dir = src_stat1_dir_buf.as_path();

    // Sanity: must detect exactly 26 variants. A count of 0 means stat1_resolve
    // was not found; 45 means the scan leaked into math1_resolve scope.
    assert_eq!(
        variants.len(),
        26,
        "collect_stat1_variant_names() returned {} variants (expected 26); \
         check that fn stat1_resolve scope is correctly detected in xrom.rs",
        variants.len()
    );

    let mut failures: Vec<String> = Vec::new();
    for variant_name in &variants {
        let fn_name = variant_to_fn_name(variant_name);
        let count = count_stat1_test_mentions(variant_name, &fn_name, tests_dir, src_stat1_dir);
        if count < 5 {
            failures.push(format!(
                "Op::{variant_name} (fn {fn_name}): only {count} test mention(s) across \
                 stat1_*.rs + src/ops/stat1/*.rs inline tests (need ≥ 5)"
            ));
        }
    }

    // Catches: Pitfall 16 — Op variants with insufficient test coverage risk
    // missing edge cases that the Phase 37 coverage gate would otherwise catch.
    // T-37.7: per-Op count baseline; drops below floor are visible in diff review.
    assert!(
        failures.is_empty(),
        "Pitfall 16 violation — Stat 1 Pac variants with insufficient test coverage:\n{}",
        failures.join("\n")
    );
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn variant_to_fn_name_sigma_bstat() {
        assert_eq!(variant_to_fn_name("SigmaBstat"), "op_sigma_bstat");
    }

    #[test]
    fn variant_to_fn_name_sigma_polyp_workflow() {
        assert_eq!(
            variant_to_fn_name("SigmaPolypWorkflow"),
            "op_sigma_polyp_workflow"
        );
    }

    #[test]
    fn variant_to_fn_name_sigma_normd_workflow() {
        assert_eq!(
            variant_to_fn_name("SigmaNormdWorkflow"),
            "op_sigma_normd_workflow"
        );
    }

    #[test]
    fn variant_to_fn_name_rand() {
        assert_eq!(variant_to_fn_name("Rand"), "op_rand");
    }

    #[test]
    fn variant_to_fn_name_seed() {
        assert_eq!(variant_to_fn_name("Seed"), "op_seed");
    }
}
