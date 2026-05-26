// Algorithm independently re-derived from HP module Owner's Manuals;
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Phase 47 Plan 01 / ADV-QUAL-01 — Unified meta-test: each `Op` variant
//! registered in `math1_resolve`, `stat1_resolve`, `time_resolve`,
//! `adv_a_resolve`, and `adv_b_resolve` must have at least 5 test functions
//! mentioning it (Pitfall 16 guard).
//!
//! **Extended in Phase 47 (ADV-QUAL-01):**
//! Extended from Phase 42 / D-42.1 (3 modules, 106 variants) to cover all 5
//! XROM modules (220 variants total: 45 + 26 + 35 + 63 + 51). ADV_MATH_A and
//! ADV_MATH_B use inline-only test surfaces (322 tests across
//! `src/ops/advantage/*.rs`; 0 dedicated external `tests/adv_*.rs` files
//! beyond `adv_backward_compat.rs`).
//!
//! **Replaces (D-42.14):**
//! - `math1_op_test_count.rs` (Phase 32 Plan 01)
//! - `stat1_op_test_count.rs` (Phase 37 Wave 1)
//!
//! **Unification rationale (D-42.1):**
//! With three XROM modules (Math 1, Stat 1, Time), per-module meta-gate files
//! proliferate. A single unified file fulfills the Phase 37 deferred commitment
//! ("if a third XROM module lands, consider unifying") and makes v3.3 Advantage
//! Pac integration free.
//!
//! **Scan strategy per module:**
//!
//! - **Math 1** — external `tests/math1_*.rs` files (full content).
//! - **Stat 1** — external `tests/stat1_*.rs` + `tests/numerical_accuracy.rs`
//!   (Pass 1) AND inline `#[cfg(test)]` blocks from `src/ops/stat1/*.rs` (Pass 2).
//! - **Time** — external `tests/time_*.rs` (if any; currently 0 files) (Pass 1)
//!   AND inline `#[cfg(test)]` blocks from `src/ops/time/*.rs` (Pass 2).
//!   Time has 209 inline tests and 0 external time_* files at Wave 1, so
//!   inline scanning is essential for accurate counts (D-42.10).
//! - **ADV MATH A** — external `tests/adv_*.rs` (Pass 1; `adv_backward_compat.rs`
//!   picked up) AND inline `#[cfg(test)]` blocks from `src/ops/advantage/*.rs`
//!   (Pass 2; 322 inline tests as the primary surface).
//! - **ADV MATH B** — same dual-scan as ADV MATH A (same `src/ops/advantage/`
//!   directory, same `adv_*` external prefix).
//!
//! **Dual-token matching (D-42.10):**
//! Each variant is matched by EITHER the `Op::<VariantName>` enum token OR the
//! `op_<snake>` function call pattern. For Math 1, only `Op::` matching is needed
//! (external-only tests). For Stat 1, Time, and Advantage, both tokens are checked
//! because inline tests call internal functions directly.
//!
//! **Time variant snake-case convention:**
//! Time `Op` variants carry a `Time` module prefix (e.g. `Op::TimeTime`,
//! `Op::TimeSetdate`). The internal functions strip this prefix: `op_time`,
//! `op_setdate`. `time_variant_to_fn_name()` strips the leading `Time` before
//! PascalCase → snake_case conversion.
//!
//! **Advantage variant snake-case convention:**
//! Advantage `Op` variants do NOT strip the `Adv` prefix (unlike Time). The
//! internal functions keep the prefix: `AdvBinin` → `op_adv_binin`,
//! `AdvExpZ` → `op_adv_exp_z`. `adv_variant_to_fn_name()` delegates directly
//! to `pascal_to_op_snake()` (same as `stat1_variant_to_fn_name`).
//!
//! **Word-boundary matching (WR-03):**
//! `line_mentions_variant` and `line_mentions_variant_or_fn` use word-boundary
//! matching to prevent `Op::Sol` from matching `Op::Solve`, `op_sigma_bstat`
//! from matching `op_sigma_bstg`, etc.
//!
//! **Expected variant counts:**
//! - Math 1 = 45, Stat 1 = 26, Time = 35, ADV MATH A = 63, ADV MATH B = 51
//!   (total 220).
//! - Sanity assertions on per-module counts catch scope leakage between resolver
//!   functions and future additions (T-42-01 mitigation).

#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

// ── Variant collection ─────────────────────────────────────────────────────────

/// Scan `xrom.rs` for Math Pac I variant names inside `fn math1_resolve()`.
/// Uses brace-depth tracking to avoid bleeding into sibling resolvers.
/// Returns a list of variant name strings (e.g. "Sinh", "Cosh").
fn collect_math1_variant_names() -> Vec<String> {
    let xrom_src = include_str!("../src/ops/math1/xrom.rs");
    collect_variants_in_fn(xrom_src, "fn math1_resolve")
}

/// Scan `xrom.rs` for Stat 1 Pac variant names inside `fn stat1_resolve()`.
/// Brace-depth tracking scopes the scan to `stat1_resolve` only.
/// Returns a list of variant name strings (e.g. "SigmaBstat", "Rand").
fn collect_stat1_variant_names() -> Vec<String> {
    let xrom_src = include_str!("../src/ops/math1/xrom.rs");
    collect_variants_in_fn(xrom_src, "fn stat1_resolve")
}

/// Scan `xrom.rs` for Time Pac variant names inside `fn time_resolve()`.
/// Brace-depth tracking scopes the scan to `time_resolve` only.
/// Returns a list of variant name strings (e.g. "TimeTime", "TimeSetdate").
fn collect_time_variant_names() -> Vec<String> {
    let xrom_src = include_str!("../src/ops/math1/xrom.rs");
    collect_variants_in_fn(xrom_src, "fn time_resolve")
}

/// Scan `xrom.rs` for ADV MATH A variant names inside `fn adv_a_resolve()`.
/// Brace-depth tracking scopes the scan to `adv_a_resolve` only.
/// Returns a list of variant name strings (e.g. "AdvBinin", "AdvMdet").
fn collect_adv_a_variant_names() -> Vec<String> {
    let xrom_src = include_str!("../src/ops/math1/xrom.rs");
    collect_variants_in_fn(xrom_src, "fn adv_a_resolve")
}

/// Scan `xrom.rs` for ADV MATH B variant names inside `fn adv_b_resolve()`.
/// Brace-depth tracking scopes the scan to `adv_b_resolve` only.
/// Returns a list of variant name strings (e.g. "AdvExpZ", "AdvTvmStarI").
fn collect_adv_b_variant_names() -> Vec<String> {
    let xrom_src = include_str!("../src/ops/math1/xrom.rs");
    collect_variants_in_fn(xrom_src, "fn adv_b_resolve")
}

/// Generic variant collector: scan `src` for `Some(Op::...)` patterns inside
/// the function whose signature contains `fn_sig`. Brace-depth tracking
/// ensures the scan is scoped to that function body only.
fn collect_variants_in_fn(src: &str, fn_sig: &str) -> Vec<String> {
    let mut variants = Vec::new();
    let mut in_fn = false;
    let mut brace_depth: i32 = 0;

    for line in src.lines() {
        let trimmed = line.trim();

        if !in_fn {
            if trimmed.contains(fn_sig) {
                in_fn = true;
                brace_depth = line.matches('{').count() as i32 - line.matches('}').count() as i32;
            }
            continue;
        }

        // Inside the function: maintain brace depth.
        brace_depth += line.matches('{').count() as i32;
        brace_depth -= line.matches('}').count() as i32;

        // Detect lines like: `"SINH" => Some(Op::Sinh),`
        if trimmed.contains("=> Some(Op::") && !trimmed.starts_with("//") {
            if let Some(after_op) = trimmed.split("Some(Op::").nth(1) {
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
            in_fn = false;
        }
    }
    variants
}

// ── Snake-case converters ──────────────────────────────────────────────────────

/// Convert a PascalCase `Op` variant name to the `op_<snake_case>` function
/// name used by Stat 1 inline tests.
///
/// Examples:
/// - `SigmaBstat`         → `op_sigma_bstat`
/// - `SigmaPolypWorkflow` → `op_sigma_polyp_workflow`
/// - `Rand`               → `op_rand`
/// - `Seed`               → `op_seed`
///
/// Algorithm: insert `_` before each uppercase letter that follows a lowercase
/// letter or digit. For uppercase-to-uppercase transitions, only insert `_`
/// if the NEXT character is lowercase (handles acronyms like "MLR").
fn stat1_variant_to_fn_name(variant: &str) -> String {
    pascal_to_op_snake(variant)
}

/// Convert a Time `Op` variant name to the `op_<snake_case>` internal function
/// name used by Time Pac inline tests.
///
/// Time variants carry a `Time` module prefix (e.g. `TimeTime`, `TimeSetdate`).
/// The internal functions drop this prefix: `op_time`, `op_setdate`.
///
/// Examples:
/// - `TimeTime`      → `op_time`
/// - `TimeSetdate`   → `op_setdate`
/// - `TimeDatePlus`  → `op_date_plus`
/// - `TimeClk12`     → `op_clk12`
/// - `TimeRunsw`     → `op_runsw`
/// - `TimeXyzalm`    → `op_xyzalm`
/// - `TimeAlmcat`    → `op_almcat`
/// - `TimeTplusx`    → `op_tplusx`
fn time_variant_to_fn_name(variant: &str) -> String {
    // Strip the leading "Time" module prefix
    let stripped = if let Some(s) = variant.strip_prefix("Time") {
        s
    } else {
        variant
    };
    pascal_to_op_snake(stripped)
}

/// Convert an Advantage Pac `Op` variant name to the `op_<snake_case>` internal
/// function name used by Advantage Pac inline tests.
///
/// Unlike Time, Advantage variants do NOT strip a module prefix. The "Adv" prefix
/// is kept as part of the snake-case function name.
///
/// Examples:
/// - `AdvBinin`      → `op_adv_binin`
/// - `AdvExpZ`       → `op_adv_exp_z`
/// - `AdvZPowN`      → `op_adv_z_pow_n`
/// - `AdvMMulM`      → `op_adv_m_mul_m`
/// - `AdvTvmStarI`   → `op_adv_tvm_star_i`
fn adv_variant_to_fn_name(variant: &str) -> String {
    pascal_to_op_snake(variant)
}

/// Core PascalCase → `op_<snake_case>` converter (shared by stat1, time, adv).
fn pascal_to_op_snake(pascal: &str) -> String {
    let mut snake = String::from("op_");
    let chars: Vec<char> = pascal.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() && i > 0 {
            let prev = chars[i - 1];
            if prev.is_lowercase() || prev.is_ascii_digit() {
                snake.push('_');
            } else {
                // Upper-to-upper: insert _ only if next char is lowercase
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

// ── Word-boundary matching helpers ─────────────────────────────────────────────

/// Return true if `line` contains `Op::<variant_name>` as a whole token — i.e.,
/// the character immediately after `variant_name` is NOT alphanumeric or `_`.
/// Prevents `Op::Sol` from matching `Op::Solve`, etc. (WR-03).
/// Comment lines are pre-filtered before calling this helper.
fn line_mentions_variant(line: &str, variant_name: &str) -> bool {
    let token = format!("Op::{variant_name}");
    check_word_boundary(line, &token)
}

/// Return true if `line` mentions the Op variant by EITHER its enum token
/// `Op::<VariantName>` OR its internal function name `op_<snake>`.
/// Uses word-boundary checking for both tokens (WR-03).
fn line_mentions_variant_or_fn(line: &str, variant_name: &str, fn_name: &str) -> bool {
    let op_token = format!("Op::{variant_name}");
    if check_word_boundary(line, &op_token) {
        return true;
    }
    check_word_boundary(line, fn_name)
}

/// Return true if `token` appears in `line` as a whole-word token — i.e., the
/// character immediately after `token` is NOT alphanumeric or `_`.
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

// ── Test-mention counters ──────────────────────────────────────────────────────

/// Count how many distinct `#[test]` function blocks in `tests/math1_*.rs`
/// contain at least one word-boundary mention of `Op::<variant_name>`.
/// Math 1 uses external-only test files (no inline tests with direct fn calls).
fn count_math1_test_mentions(variant_name: &str, tests_dir: &Path) -> usize {
    let entries = match std::fs::read_dir(tests_dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    let mut total = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !filename.starts_with("math1_") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        // Split on `#[test]` markers to get per-function body slices.
        let test_slices: Vec<&str> = content.split("#[test]").skip(1).collect();
        for slice in test_slices {
            let found = slice.lines().any(|line| {
                let trimmed = line.trim();
                !trimmed.starts_with("//") && line_mentions_variant(line, variant_name)
            });
            if found {
                total += 1;
            }
        }
    }
    total
}

/// Count how many distinct `#[test]` function blocks across BOTH passes contain
/// at least one mention of the Stat 1 variant (by enum token OR function name).
///
/// Pass 1: external `tests/stat1_*.rs` + `tests/numerical_accuracy.rs`.
/// Pass 2: inline `#[cfg(test)]` blocks from `src/ops/stat1/*.rs`.
fn count_stat1_test_mentions(
    variant_name: &str,
    fn_name: &str,
    tests_dir: &Path,
    src_stat1_dir: &Path,
) -> usize {
    count_xrom_test_mentions_dual(
        variant_name,
        fn_name,
        tests_dir,
        "stat1_",
        Some("numerical_accuracy.rs"),
        src_stat1_dir,
    )
}

/// Count how many distinct `#[test]` function blocks across BOTH passes contain
/// at least one mention of the Time variant (by enum token OR function name).
///
/// Pass 1: external `tests/time_*.rs` (if any — currently 0 at Wave 1).
/// Pass 2: inline `#[cfg(test)]` blocks from `src/ops/time/*.rs`.
/// Time has 209 inline tests which are the primary test surface at Wave 1.
fn count_time_test_mentions(
    variant_name: &str,
    fn_name: &str,
    tests_dir: &Path,
    src_time_dir: &Path,
) -> usize {
    count_xrom_test_mentions_dual(
        variant_name,
        fn_name,
        tests_dir,
        "time_",
        None,
        src_time_dir,
    )
}

/// Count how many distinct `#[test]` function blocks across BOTH passes contain
/// at least one mention of the ADV MATH A variant (by enum token OR function name).
///
/// Pass 1: external `tests/adv_*.rs` (picks up `adv_backward_compat.rs`).
/// Pass 2: inline `#[cfg(test)]` blocks from `src/ops/advantage/*.rs`.
/// Advantage Pac has 322 inline tests as the primary test surface.
fn count_adv_a_test_mentions(
    variant_name: &str,
    fn_name: &str,
    tests_dir: &Path,
    src_adv_dir: &Path,
) -> usize {
    count_xrom_test_mentions_dual(
        variant_name,
        fn_name,
        tests_dir,
        "adv_",
        None,
        src_adv_dir,
    )
}

/// Count how many distinct `#[test]` function blocks across BOTH passes contain
/// at least one mention of the ADV MATH B variant (by enum token OR function name).
///
/// Pass 1: external `tests/adv_*.rs` (picks up `adv_backward_compat.rs`).
/// Pass 2: inline `#[cfg(test)]` blocks from `src/ops/advantage/*.rs`.
fn count_adv_b_test_mentions(
    variant_name: &str,
    fn_name: &str,
    tests_dir: &Path,
    src_adv_dir: &Path,
) -> usize {
    count_xrom_test_mentions_dual(
        variant_name,
        fn_name,
        tests_dir,
        "adv_",
        None,
        src_adv_dir,
    )
}

/// Generic dual-scan (external files + inline cfg(test)) for XROM modules.
///
/// - `tests_dir`: directory containing external `tests/*.rs` files
/// - `prefix`: filename prefix filter for external test files (e.g. "stat1_")
/// - `extra_file`: optional extra external file to include (e.g. "numerical_accuracy.rs")
/// - `src_module_dir`: directory of module source files for inline test extraction
fn count_xrom_test_mentions_dual(
    variant_name: &str,
    fn_name: &str,
    tests_dir: &Path,
    prefix: &str,
    extra_file: Option<&str>,
    src_module_dir: &Path,
) -> usize {
    let mut total = 0;

    // ── Pass 1: external test files ───────────────────────────────────────────
    if let Ok(entries) = std::fs::read_dir(tests_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let include = filename.starts_with(prefix) || extra_file == Some(filename);
            if !include {
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
                    total += 1;
                }
            }
        }
    }

    // ── Pass 2: inline #[cfg(test)] blocks ────────────────────────────────────
    if let Ok(entries) = std::fs::read_dir(src_module_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            // Each split on `#[cfg(test)]` yields one non-test prefix + test sections.
            let cfg_test_sections: Vec<&str> = content.split("#[cfg(test)]").skip(1).collect();
            for section in cfg_test_sections {
                let test_slices: Vec<&str> = section.split("#[test]").skip(1).collect();
                for slice in test_slices {
                    let found = slice.lines().any(|line| {
                        let trimmed = line.trim();
                        !trimmed.starts_with("//")
                            && line_mentions_variant_or_fn(line, variant_name, fn_name)
                    });
                    if found {
                        total += 1;
                    }
                }
            }
        }
    }

    total
}

// ── Unified meta-test ─────────────────────────────────────────────────────────

/// Meta-test (Phase 47 / ADV-QUAL-01 — extended from Phase 42 / D-42.1):
/// every Op variant registered in any of the five XROM module resolvers
/// (math1_resolve, stat1_resolve, time_resolve, adv_a_resolve, adv_b_resolve)
/// must have ≥ 5 test function references.
///
/// Scans per module:
/// - Math 1: `tests/math1_*.rs` (external-only; 14 files at graduation)
/// - Stat 1: `tests/stat1_*.rs` + `tests/numerical_accuracy.rs` + inline
///   `src/ops/stat1/*.rs` cfg(test) blocks (dual-scan per D-42.10)
/// - Time:   `tests/time_*.rs` (0 at Wave 1) + inline `src/ops/time/*.rs`
///   cfg(test) blocks (inline-primary at Wave 1; dual-scan per D-42.10)
/// - ADV MATH A: `tests/adv_*.rs` + inline `src/ops/advantage/*.rs` cfg(test)
///   blocks (322 inline tests as primary surface at v3.3 graduation)
/// - ADV MATH B: same dual-scan as ADV MATH A (same source directory)
///
/// Expected variant counts (T-42-01 sanity assertions):
/// - Math 1 = 45, Stat 1 = 26, Time = 35, ADV MATH A = 63, ADV MATH B = 51
///   (total = 220)
///
/// Catches: Pitfall 16 — Op variants with fewer than 5 tests risk missing edge
/// cases that the coverage gate would otherwise catch.
#[test]
fn each_xrom_op_has_at_least_5_tests() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tests_dir = manifest_dir.join("tests");
    let src_ops_dir = manifest_dir.join("src").join("ops");

    // ── Math 1 ────────────────────────────────────────────────────────────────
    let math1_variants = collect_math1_variant_names();
    assert_eq!(
        math1_variants.len(),
        45,
        "Math 1: expected 45 variants, got {}; \
         check fn math1_resolve scope in xrom.rs (T-42-01 scope-leakage guard)",
        math1_variants.len()
    );

    let mut failures: Vec<String> = Vec::new();
    for variant_name in &math1_variants {
        let count = count_math1_test_mentions(variant_name, &tests_dir);
        if count < 5 {
            failures.push(format!(
                "[Math1] Op::{variant_name}: only {count} test mention(s) in \
                 math1_*.rs (need ≥ 5)"
            ));
        }
    }

    // ── Stat 1 ────────────────────────────────────────────────────────────────
    let stat1_variants = collect_stat1_variant_names();
    assert_eq!(
        stat1_variants.len(),
        26,
        "Stat 1: expected 26 variants, got {}; \
         check fn stat1_resolve scope in xrom.rs (T-42-01 scope-leakage guard)",
        stat1_variants.len()
    );

    let src_stat1_dir = src_ops_dir.join("stat1");
    for variant_name in &stat1_variants {
        let fn_name = stat1_variant_to_fn_name(variant_name);
        let count = count_stat1_test_mentions(variant_name, &fn_name, &tests_dir, &src_stat1_dir);
        if count < 5 {
            failures.push(format!(
                "[Stat1] Op::{variant_name} (fn {fn_name}): only {count} test mention(s) \
                 across stat1_*.rs + src/ops/stat1/*.rs inline tests (need ≥ 5)"
            ));
        }
    }

    // ── Time ─────────────────────────────────────────────────────────────────
    let time_variants = collect_time_variant_names();
    assert_eq!(
        time_variants.len(),
        35,
        "Time: expected 35 variants, got {}; \
         check fn time_resolve scope in xrom.rs (T-42-01 scope-leakage guard)",
        time_variants.len()
    );

    let src_time_dir = src_ops_dir.join("time");
    for variant_name in &time_variants {
        let fn_name = time_variant_to_fn_name(variant_name);
        let count = count_time_test_mentions(variant_name, &fn_name, &tests_dir, &src_time_dir);
        if count < 5 {
            failures.push(format!(
                "[Time] Op::{variant_name} (fn {fn_name}): only {count} test mention(s) \
                 across time_*.rs + src/ops/time/*.rs inline tests (need ≥ 5)"
            ));
        }
    }

    // ── ADV MATH A (XROM 22, bit-3) ──────────────────────────────────────────
    let adv_a_variants = collect_adv_a_variant_names();
    assert_eq!(
        adv_a_variants.len(),
        63,
        "ADV MATH A: expected 63 variants, got {}; \
         check fn adv_a_resolve scope in xrom.rs (ADV-QUAL-01 scope-leakage guard)",
        adv_a_variants.len()
    );

    let src_adv_dir = src_ops_dir.join("advantage");
    for variant_name in &adv_a_variants {
        let fn_name = adv_variant_to_fn_name(variant_name);
        let count =
            count_adv_a_test_mentions(variant_name, &fn_name, &tests_dir, &src_adv_dir);
        if count < 5 {
            failures.push(format!(
                "[AdvA] Op::{variant_name} (fn {fn_name}): only {count} test mention(s) \
                 across adv_*.rs + src/ops/advantage/*.rs inline tests (need ≥ 5)"
            ));
        }
    }

    // ── ADV MATH B (XROM 24, bit-4) ──────────────────────────────────────────
    let adv_b_variants = collect_adv_b_variant_names();
    assert_eq!(
        adv_b_variants.len(),
        51,
        "ADV MATH B: expected 51 variants, got {}; \
         check fn adv_b_resolve scope in xrom.rs (ADV-QUAL-01 scope-leakage guard)",
        adv_b_variants.len()
    );

    for variant_name in &adv_b_variants {
        let fn_name = adv_variant_to_fn_name(variant_name);
        let count =
            count_adv_b_test_mentions(variant_name, &fn_name, &tests_dir, &src_adv_dir);
        if count < 5 {
            failures.push(format!(
                "[AdvB] Op::{variant_name} (fn {fn_name}): only {count} test mention(s) \
                 across adv_*.rs + src/ops/advantage/*.rs inline tests (need ≥ 5)"
            ));
        }
    }

    // ── Report ────────────────────────────────────────────────────────────────
    assert!(
        failures.is_empty(),
        "Pitfall 16 violation — XROM module variants with insufficient test coverage \
         across all 5 modules (Math 1 + Stat 1 + Time + ADV MATH A + ADV MATH B):\n{}",
        failures.join("\n")
    );
}

// ── Unit tests for snake-case converters ──────────────────────────────────────

#[cfg(test)]
mod unit_tests {
    use super::*;

    // ── stat1_variant_to_fn_name ──────────────────────────────────────────────

    #[test]
    fn stat1_variant_to_fn_name_sigma_bstat() {
        assert_eq!(stat1_variant_to_fn_name("SigmaBstat"), "op_sigma_bstat");
    }

    #[test]
    fn stat1_variant_to_fn_name_sigma_polyp_workflow() {
        assert_eq!(
            stat1_variant_to_fn_name("SigmaPolypWorkflow"),
            "op_sigma_polyp_workflow"
        );
    }

    #[test]
    fn stat1_variant_to_fn_name_sigma_normd_workflow() {
        assert_eq!(
            stat1_variant_to_fn_name("SigmaNormdWorkflow"),
            "op_sigma_normd_workflow"
        );
    }

    #[test]
    fn stat1_variant_to_fn_name_rand() {
        assert_eq!(stat1_variant_to_fn_name("Rand"), "op_rand");
    }

    #[test]
    fn stat1_variant_to_fn_name_seed() {
        assert_eq!(stat1_variant_to_fn_name("Seed"), "op_seed");
    }

    // ── time_variant_to_fn_name ───────────────────────────────────────────────

    #[test]
    fn time_variant_to_fn_name_time_time() {
        // TimeTime → strip "Time" → "Time" → snake → "time" → "op_time"
        assert_eq!(time_variant_to_fn_name("TimeTime"), "op_time");
    }

    #[test]
    fn time_variant_to_fn_name_time_setdate() {
        assert_eq!(time_variant_to_fn_name("TimeSetdate"), "op_setdate");
    }

    #[test]
    fn time_variant_to_fn_name_time_date_plus() {
        assert_eq!(time_variant_to_fn_name("TimeDatePlus"), "op_date_plus");
    }

    #[test]
    fn time_variant_to_fn_name_time_clk12() {
        assert_eq!(time_variant_to_fn_name("TimeClk12"), "op_clk12");
    }

    #[test]
    fn time_variant_to_fn_name_time_runsw() {
        assert_eq!(time_variant_to_fn_name("TimeRunsw"), "op_runsw");
    }

    #[test]
    fn time_variant_to_fn_name_time_xyzalm() {
        assert_eq!(time_variant_to_fn_name("TimeXyzalm"), "op_xyzalm");
    }

    #[test]
    fn time_variant_to_fn_name_time_almcat() {
        assert_eq!(time_variant_to_fn_name("TimeAlmcat"), "op_almcat");
    }

    #[test]
    fn time_variant_to_fn_name_time_tplusx() {
        assert_eq!(time_variant_to_fn_name("TimeTplusx"), "op_tplusx");
    }

    #[test]
    fn time_variant_to_fn_name_time_ddays() {
        assert_eq!(time_variant_to_fn_name("TimeDdays"), "op_ddays");
    }

    #[test]
    fn time_variant_to_fn_name_time_atime24() {
        assert_eq!(time_variant_to_fn_name("TimeAtime24"), "op_atime24");
    }

    // ── adv_variant_to_fn_name ────────────────────────────────────────────────

    #[test]
    fn adv_variant_to_fn_name_adv_binin() {
        assert_eq!(adv_variant_to_fn_name("AdvBinin"), "op_adv_binin");
    }

    #[test]
    fn adv_variant_to_fn_name_adv_exp_z() {
        assert_eq!(adv_variant_to_fn_name("AdvExpZ"), "op_adv_exp_z");
    }

    #[test]
    fn adv_variant_to_fn_name_adv_z_pow_n() {
        assert_eq!(adv_variant_to_fn_name("AdvZPowN"), "op_adv_z_pow_n");
    }

    #[test]
    fn adv_variant_to_fn_name_adv_m_mul_m() {
        assert_eq!(adv_variant_to_fn_name("AdvMMulM"), "op_adv_m_mul_m");
    }

    #[test]
    fn adv_variant_to_fn_name_adv_tvm_star_i() {
        assert_eq!(adv_variant_to_fn_name("AdvTvmStarI"), "op_adv_tvm_star_i");
    }
}
