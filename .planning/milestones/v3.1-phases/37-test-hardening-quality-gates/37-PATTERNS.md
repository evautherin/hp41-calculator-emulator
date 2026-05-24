# Phase 37: Test Hardening & Quality Gates - Pattern Map

**Mapped:** 2026-05-24
**Files analyzed:** 10 (7 new, 3 modified)
**Analogs found:** 10 / 10

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-core/tests/stat1_op_test_count.rs` | test / meta-gate | transform (scan + assert) | `hp41-core/tests/math1_op_test_count.rs` | exact (sibling) |
| `hp41-core/tests/lint_stat1_assertions.rs` | test / lint | transform (scan + assert) | `hp41-core/tests/lint_math1_assertions.rs` | exact (sibling) |
| `hp41-core/tests/stat1_backward_compat.rs` | test / integration | CRUD (load fixture + assert) | `hp41-core/tests/synthetic_tests.rs` | role-match |
| `hp41-core/tests/stat1_modal_coverage.rs` | test / integration | CRUD (dispatch + assert) | `hp41-core/tests/stat1_rand_determinism.rs` | role-match |
| `hp41-core/tests/stat1_anova_coverage.rs` | test / integration | CRUD (dispatch + assert) | `hp41-core/tests/stat1_rand_determinism.rs` | role-match |
| `hp41-core/tests/fixtures/v30-autosave.json` | fixture / config | file-I/O | `hp41-core/tests/fixtures/v20-autosave.json` | exact (sibling) |
| `hp41-core/tests/numerical_accuracy.rs` | test / accuracy suite | batch (case! macro, pass-rate gate) | self (inline extension) | exact |
| `hp41-gui/e2e/smoke.spec.js` | test / E2E | request-response (invokeBackend) | self (inline extension) | exact |
| `docs/hp41-stat1-divergences.md` | documentation | — | self (append entry) | exact |
| `README.md` | documentation | — | `README.md` v3.0 pattern (v3.0 hard-claim text) | role-match |

---

## Pattern Assignments

### `hp41-core/tests/stat1_op_test_count.rs` (test / meta-gate)

**Analog:** `hp41-core/tests/math1_op_test_count.rs` (229 lines, read in full)

**File header / crate-level allow pattern** (lines 1–48):
```rust
// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Wave-1 meta-test: each Stat 1 Pac `Op` variant registered in `stat1_resolve`
//! must have at least 5 test function mentions across BOTH external
//! `hp41-core/tests/stat1_*.rs` files AND inline `#[cfg(test)]` modules inside
//! `hp41-core/src/ops/stat1/*.rs` (D-37.7 dual-scan scope).
//!
//! ... [doc comment mirroring math1 analog §Plan 32-01 graduation notes]

#![allow(clippy::unwrap_used)]

use std::path::Path;
```

**Variant collection — brace-depth scan of `stat1_resolve` scope** (adapt from analog lines 57–109):
```rust
fn collect_stat1_variant_names() -> Vec<String> {
    let xrom_src = include_str!("../src/ops/math1/xrom.rs");
    let mut variants = Vec::new();
    let mut in_stat1_resolve = false;
    let mut brace_depth: i32 = 0;

    for line in xrom_src.lines() {
        let trimmed = line.trim();

        if !in_stat1_resolve {
            // KEY ADAPTATION: scope to `fn stat1_resolve`, not `fn math1_resolve`
            if trimmed.contains("fn stat1_resolve") {
                in_stat1_resolve = true;
                brace_depth = line.matches('{').count() as i32 - line.matches('}').count() as i32;
            }
            continue;
        }

        brace_depth += line.matches('{').count() as i32;
        brace_depth -= line.matches('}').count() as i32;

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
            in_stat1_resolve = false;
        }
    }
    variants
}
```

**Word-boundary helper — reuse verbatim** (analog lines 111–134):
```rust
fn line_mentions_variant(line: &str, variant_name: &str) -> bool {
    let token = format!("Op::{variant_name}");
    let mut search_start = 0;
    while let Some(pos) = line[search_start..].find(&token) {
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
```

**Dual-scan count function — KEY ADAPTATION from analog lines 136–186**:
```rust
// D-37.7 KEY ADAPTATION: scan BOTH external tests/stat1_*.rs AND inline
// #[cfg(test)] modules inside src/ops/stat1/*.rs. The analog
// math1_op_test_count.rs scans only external tests/math1_*.rs because
// Math Pac I has predominantly external tests; Stat 1 has 189 inline tests
// distributed across 12 source files, so external-only scan would miss them.
fn count_stat1_test_mentions(variant_name: &str, tests_dir: &Path) -> usize {
    let mut total_fn_count = 0;

    // Pass 1: external tests/stat1_*.rs files (mirrors math1 analog exactly)
    if let Ok(entries) = std::fs::read_dir(tests_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") { continue; }
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !filename.starts_with("stat1_") { continue; }
            if let Ok(content) = std::fs::read_to_string(&path) {
                let test_slices: Vec<&str> = content.split("#[test]").skip(1).collect();
                for slice in test_slices {
                    let found = slice.lines().any(|line| {
                        let trimmed = line.trim();
                        !trimmed.starts_with("//") && line_mentions_variant(line, variant_name)
                    });
                    if found { total_fn_count += 1; }
                }
            }
        }
    }

    // Pass 2: inline #[cfg(test)] modules in src/ops/stat1/*.rs
    // Use CARGO_MANIFEST_DIR to avoid CWD-relative fragility.
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_stat1 = manifest_dir.join("src").join("ops").join("stat1");
    if let Ok(entries) = std::fs::read_dir(&src_stat1) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") { continue; }
            if let Ok(content) = std::fs::read_to_string(&path) {
                // Only scan the content AFTER #[cfg(test)] markers to avoid
                // counting production code that happens to mention an Op variant.
                for test_block in content.split("#[cfg(test)]").skip(1) {
                    let test_slices: Vec<&str> = test_block.split("#[test]").skip(1).collect();
                    for slice in test_slices {
                        let found = slice.lines().any(|line| {
                            let trimmed = line.trim();
                            !trimmed.starts_with("//") && line_mentions_variant(line, variant_name)
                        });
                        if found { total_fn_count += 1; }
                    }
                }
            }
        }
    }

    total_fn_count
}
```

**Test function — adapt from analog lines 188–228**:
```rust
#[test]
fn each_stat1_op_has_at_least_5_tests() {
    let variants = collect_stat1_variant_names();
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tests_dir_buf = manifest_dir.join("tests");
    let tests_dir = tests_dir_buf.as_path();

    let mut failures: Vec<String> = Vec::new();
    for variant_name in &variants {
        let count = count_stat1_test_mentions(variant_name, tests_dir);
        if count < 5 {
            failures.push(format!(
                "Op::{variant_name}: only {count} test mention(s) in stat1 scope (need ≥ 5)"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Pitfall 16 violation — Stat 1 Pac variants with insufficient test coverage:\n{}",
        failures.join("\n")
    );
}
```

---

### `hp41-core/tests/lint_stat1_assertions.rs` (test / lint)

**Analog:** `hp41-core/tests/lint_math1_assertions.rs` (326 lines, read in full)

**File header and constants — reuse nearly verbatim** (analog lines 1–81):
```rust
// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Phase 37 Wave 1 — assertion-discipline lint over `tests/stat1_*.rs` AND
//! inline `#[cfg(test)]` modules in `src/ops/stat1/*.rs` (D-37.8 scope).
//!
//! Two gates: `no_decimal_assert_eq_in_stat1_tests` (Pitfall 17) +
//! `no_manual_tolerance_pattern_in_stat1_tests` (Pitfall 14).
//! LINT-EXEMPT annotation convention unchanged from math1 analog.
//!
//! ## LINT-EXEMPT pre-annotations required before Wave 1 lands:
//! `stat1_rand_determinism.rs` line 39 (`assert_eq!(state.rand_seed, expected)`)
//! and line 43 (`assert_eq!(state.stack.x, expected)`) — exact HpNum equality
//! on Decimal-exact LCG accumulator (no f64 bridge; STAT-RNG-03 semantics).
//! Add: `// LINT-EXEMPT: exact HpNum equality via Decimal::new construction —
//!       no f64 bridge; rand_seed is a Decimal-exact LCG accumulator`

#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

const LINT_EXEMPT_TOKEN: &str = "LINT-EXEMPT:";
```

**File collector — KEY ADAPTATION: dual-scope scan** (adapt from analog lines 83–108):
```rust
// Collect BOTH external tests/stat1_*.rs AND inline #[cfg(test)] content
// from src/ops/stat1/*.rs. Returns (source_path_label, content_to_scan) pairs.
fn collect_stat1_test_content(tests_dir: &Path) -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();

    // Pass 1: external tests/stat1_*.rs
    if let Ok(entries) = std::fs::read_dir(tests_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") { continue; }
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !filename.starts_with("stat1_") { continue; }
            if let Ok(content) = std::fs::read_to_string(&path) {
                files.push((path, content));
            }
        }
    }

    // Pass 2: inline #[cfg(test)] blocks from src/ops/stat1/*.rs
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_stat1 = manifest_dir.join("src").join("ops").join("stat1");
    if let Ok(entries) = std::fs::read_dir(&src_stat1) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") { continue; }
            if let Ok(full_content) = std::fs::read_to_string(&path) {
                // Extract only the #[cfg(test)] portion
                let test_content: String = full_content
                    .split("#[cfg(test)]")
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join("#[cfg(test)]");
                if !test_content.is_empty() {
                    files.push((path, test_content));
                }
            }
        }
    }
    files
}
```

**Lint helpers — reuse verbatim from analog** (analog lines 110–268):

The four helper functions `format_offender`, `preceding_block_has_lint_exempt`,
`line_is_forbidden_assert_eq`, `line_is_forbidden_manual_tolerance` copy verbatim
from `lint_math1_assertions.rs` lines 110–268. No changes needed — the heuristics
are file-content-agnostic.

**Test functions — adapt names only** (analog lines 270–325):
```rust
#[test]
fn no_decimal_assert_eq_in_stat1_tests() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tests_dir = manifest_dir.join("tests");
    let files = collect_stat1_test_content(&tests_dir);
    // ... identical body to analog, using files from the dual-scope collector
    assert!(
        offenders.is_empty(),
        "Pitfall 17 violation — `assert_eq!(decimal, decimal)` on iterated \
         results in stat1 tests (use `approx::assert_relative_eq!` or \
         `// LINT-EXEMPT: <reason>`):\n{}",
        offenders.join("\n")
    );
}

#[test]
fn no_manual_tolerance_pattern_in_stat1_tests() {
    // ... identical body to analog
}
```

---

### `hp41-core/tests/stat1_backward_compat.rs` (test / integration)

**Analog:** `hp41-core/tests/synthetic_tests.rs` lines 263–297 (`test_calcstate_loads_without_new_fields`)

**File header pattern** (from `stat1_rand_determinism.rs` lines 1–15):
```rust
// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Backward-compat migration test for STAT-QUAL-10:
//! a v3.0 save file (xrom_modules: 1) must load and have bit 1 set
//! after `migrate_after_load` (STAT-FW-02 / D-33.7).

#![allow(clippy::unwrap_used)]

use hp41_core::CalcState;
```

**Core test pattern** (adapt from `synthetic_tests.rs` lines 263–297):
```rust
/// STAT-QUAL-10: v3.0 save file with `"xrom_modules": 1` loads into v3.1
/// without error; `migrate_after_load` activates bit 1 (Stat 1 Pac).
///
/// The fixture uses `include_str!` (compile-time embedding) matching the
/// `v20-autosave.json` pattern. The `rand_seed` field is ABSENT to exercise
/// `#[serde(default)]` WITHOUT `#[serde(skip)]` (Pitfall 20 / STAT-RNG-03).
///
/// Catches: migrate_after_load not setting bit 1; serde failing on
/// absent rand_seed; xrom_modules field not carrying #[serde(default)].
#[test]
fn v30_save_loads_with_stat1_migration() {
    let json = include_str!("fixtures/v30-autosave.json");
    let mut state: CalcState = serde_json::from_str(json)
        .expect("v30-autosave.json must deserialize without error");
    assert_eq!(
        state.xrom_modules, 1,
        "pre-migration: v3.0 fixture must have xrom_modules=1 (only Math 1)"
    );
    hp41_core::state::migrate_after_load(&mut state);
    assert_eq!(
        state.xrom_modules, 0b0000_0011,
        "post-migration: bit 1 must be set (Stat 1 Pac auto-enabled per STAT-FW-02)"
    );
}

/// STAT-QUAL-10 secondary: rand_seed defaults to zero (HpNum::zero) when absent
/// from the v3.0 fixture (the field did not exist in v3.0 CalcState).
///
/// Catches: rand_seed accidentally carrying #[serde(skip)] instead of
/// #[serde(default)] — would cause deserialization failure on old saves.
#[test]
fn v30_save_rand_seed_defaults_to_zero() {
    let json = include_str!("fixtures/v30-autosave.json");
    let mut state: CalcState = serde_json::from_str(json)
        .expect("v30-autosave.json must deserialize");
    hp41_core::state::migrate_after_load(&mut state);
    // rand_seed absent in fixture → serde(default) → HpNum::zero()
    use rust_decimal::Decimal;
    assert_eq!(
        state.rand_seed.inner(),
        Decimal::ZERO,
        "rand_seed must default to zero when absent from v3.0 fixture"
    );
}
```

**Import pattern** (mirrors `stat1_rand_determinism.rs` lines 16–25):
```rust
use hp41_core::CalcState;
use rust_decimal::Decimal;
```

---

### `hp41-core/tests/stat1_modal_coverage.rs` (test / coverage-gap)

**Analog:** `hp41-core/tests/stat1_rand_determinism.rs` (file header + dispatch pattern)
**Also reference:** `hp41-core/tests/stat1_cancellation.rs` (modal submit pattern)

**File header pattern** (mirror `stat1_rand_determinism.rs` lines 1–3, 16–25):
```rust
// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Wave 2 coverage-gap closure for `stat1/modal.rs` (74.21% → ≥ 90%).
//! Targets: error paths in submit_step per Stat1Step variant, cancel_step paths,
//! edge cases in modal initialization, multi-step modal sequences.

#![allow(clippy::unwrap_used)]

use hp41_core::ops::dispatch;
use hp41_core::ops::Op;
use hp41_core::ops::stat1::modal::{submit_step, Stat1Step};
use hp41_core::{CalcState, HpNum};
use rust_decimal::Decimal;
```

**Dispatch + modal submit pattern** (from `stat1_rand_determinism.rs` lines 98–121):
```rust
// Pattern: dispatch Op to open modal, then call submit_step directly
// (same as seed_modal_round_trip in stat1_rand_determinism.rs lines 98-121)
#[test]
fn normd_mode_choice_cdf_path() {
    let mut state = CalcState::new();
    // Open ΣNORMD → NormdModeChoice modal
    dispatch(&mut state, Op::SigmaNormdWorkflow).expect("ΣNORMD must open modal");
    assert_eq!(state.modal_prompt, Some("ΣNORMD MODE?".to_string()));

    // Submit mode 1 (CDF) with x = 1.0
    state.stack.x = HpNum::from(Decimal::from(1i32)); // mode 1 = CDF
    push(&mut state, "1.0");  // x input
    submit_step(&mut state, Stat1Step::NormdModeChoice)
        .expect("NormdModeChoice CDF submit must succeed");
    // Catches: NormdModeChoice branch not covered in submit_step
}
```

**Error-path pattern** (showing how to test error returns):
```rust
#[test]
fn chisqd_nu_prompt_invalid_nu_returns_error() {
    let mut state = CalcState::new();
    dispatch(&mut state, Op::SigmaChisqdWorkflow).expect("ΣCHISQD must open modal");
    // ν must be a positive integer; x=0 is invalid
    state.stack.x = HpNum::from(Decimal::ZERO);
    let result = submit_step(&mut state, Stat1Step::ChisqdNuPrompt);
    assert!(result.is_err(), "ν=0 must return error from ChisqdNuPrompt submit");
    // Catches: error path in ChisqdNuPrompt not covered (missed branch in 74% modal)
}
```

---

### `hp41-core/tests/stat1_anova_coverage.rs` (test / coverage-gap)

**Analog:** `hp41-core/tests/stat1_rand_determinism.rs` + `hp41-core/tests/stat1_cancellation.rs`

**File header pattern** (mirror `stat1_rand_determinism.rs` lines 1–3):
```rust
// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Wave 2 coverage-gap closure for `stat1/anova.rs` (86.50% → ≥ 90%).
//! Targets: ΣAOVTWO 0-cell error path, ΣANOCOV covariate edge case,
//! SIZE-guard paths (fewer Σ-registers than needed).

#![allow(clippy::unwrap_used)]

use hp41_core::error::HpError;
use hp41_core::ops::dispatch;
use hp41_core::ops::Op;
use hp41_core::CalcState;
```

**Core dispatch + error-path pattern**:
```rust
/// Catches: ΣAOVONE SIZE-guard path (R16..R22 must contain valid ANOVA data;
/// calling with default-zero state triggers the guard).
#[test]
fn aovone_with_insufficient_data_returns_error() {
    let mut state = CalcState::new();
    // Default state has all registers = 0; ΣAOVONE needs populated Σ-registers
    let result = dispatch(&mut state, Op::SigmaAovone);
    assert!(
        result.is_err(),
        "ΣAOVONE with zero data must return error (SIZE-guard path)"
    );
}
```

---

### `hp41-core/tests/fixtures/v30-autosave.json` (fixture)

**Analog:** `hp41-core/tests/fixtures/v20-autosave.json` (131 lines, read in full)

**Structure:** Minimal `CalcState` JSON with `"xrom_modules": 1` and WITHOUT `rand_seed`
(to exercise `#[serde(default)]` without `#[serde(skip)]`).

**Pattern from `v20-autosave.json`** — same shape, adding v3.0-era fields:
```json
{
  "stack": {
    "x": "0", "y": "0", "z": "0", "t": "0",
    "lastx": "0", "lift_enabled": true
  },
  "regs": [ <64 "0" entries> ],
  "alpha_reg": "", "alpha_mode": false, "angle_mode": "Deg",
  "display_mode": {"Fix": 4},
  "entry_buf": "", "program": [], "prgm_mode": false, "pc": 0,
  "call_stack": [], "is_running": false, "user_mode": false,
  "key_assignments": {}, "last_key_code": 0,
  "reg_m": "0", "reg_n": "0", "reg_o": "0",
  "xrom_modules": 1
}
```

**Key difference from `v20-autosave.json`:** adds `"xrom_modules": 1` and explicitly
OMITS `rand_seed` (the field did not exist in v3.0 saves). All `#[serde(skip)]` fields
(`modal_program`, `modal_prompt`, `integ_state`, `solve_state`, `cancel_requested`,
`print_buffer`, `pending_chisqd_nu`) are also absent — they all carry `#[serde(default, skip)]`
so deserialization must succeed. Use `serde_json::to_string_pretty(&CalcState::new())`
as generation reference, then manually set `xrom_modules: 1` and remove `rand_seed`.

---

### `hp41-core/tests/numerical_accuracy.rs` (MODIFY — inline extension)

**Analog:** self (lines 100–123 for `case!` macro, lines 6934–7013 for gate logic)

**New `iter` arm addition** (after existing two arms, before first Stat 1 case):
```rust
// New constant and macro arm for STAT-QUAL-05 iterative-path tolerance tier
const ITER_TOL: f64 = 1e-7; // Two-level discipline: 1e-9 closed-form, 1e-7 iterative

// Add to case! macro (append as third arm AFTER the existing `wide` arm):
($domain:expr, $desc:expr, $expected:expr, $actual:expr, iter) => {{
    id += 1;
    cases.push(AccuracyCase {
        id,
        domain: $domain,
        description: $desc.to_string(),
        expected: $expected,
        actual: $actual,
        tol: ITER_TOL,
    });
}};
```

**Case comment discipline** (from existing accuracy cases, carried forward):
```rust
// Each new Stat 1 case carries the three required comment lines:
// Source: scipy.stats.<fn>(<args>) = <value>  [D-37.1: fresh derivation]
// Free42: N/A — Stat 1 Pac oracle; scipy.stats ground truth per D-37.1
// Catches: <regression class per D-27.1 risk-weighted discipline>
{
    let mut s = CalcState::new();
    push(&mut s, "1.96");
    dispatch(&mut s, Op::SigmaNormdWorkflow).unwrap();
    // ... modal mode selection + eval
    // Source: scipy.stats.norm.sf(1.96) = 0.024997895...  (upper-tail Q)
    // Free42: N/A — Stat 1 Pac oracle; scipy.stats ground truth per D-37.1
    // Catches: ΣNORMD upper-tail CDF accuracy regression
    case!("stat1_normd", "Q(1.96) upper-tail CDF", 0.024997895450, get_x(&s));
}
```

**Gate logic — no change required** (lines 6934–7013 unchanged). The existing
`div_ceil(100)` combined ≥ 98% gate automatically covers the extended ~798-case suite.
The `EXPECTED_BASELINE_FAILURES` constant and `c.id < 504` v1.x sub-gate are
range-bounded and unaffected by appending new cases with `id > 768`.

**Case allocation pattern** (D-37.4 / D-37.5):
- `default` arm (tol = TOLERANCE = 1e-9): closed-form ops — ΣNORMD CDF/PDF, ΣSPEAR, ΣBSTAT/BSTG, ΣLIN/EXP/LOGI/POW, ΣXSQEV/EFXSQ, ΣAOVONE F-ratio
- `iter` arm (tol = ITER_TOL = 1e-7): iterative paths — ΣNORMD inverse/probit, ΣCHISQD CDF, ΣTSTAT p-value, ΣMLRXY regression coefficient, ΣPOLYP polynomial

---

### `hp41-gui/e2e/smoke.spec.js` (MODIFY — new `it()` block)

**Analog:** self (lines 214–244 for `xeq_SINH` invokeBackend pattern, lines 101–129 for `invokeBackend` helper)

**New `it()` block placement:** append after the existing MATRIX DET block (line 341),
inside the same `describe(...)` block. Mirror the SINH test structure.

**Core invokeBackend pattern** (from lines 101–129, 214–244 — reuse verbatim):
```javascript
// ΣNORMD test — mirrors the XEQ "SINH" pattern exactly:
// direct dispatch_op via invokeBackend (XROM modal bypass pattern).
//
// ΣNORMD modal flow per stat1/modal.rs:
//   Step 1: dispatch Op::SigmaNormdWorkflow → modal opens (NormdModeChoice)
//   Step 2: dispatch mode key → mode selected (e.g., mode 1 = upper-tail CDF)
//   Step 3: dispatch x value (push 1.96 to stack before xeq_ΣNORMD)
//   Step 4: assert view.display_str ≈ "0.0250"
//
// CRITICAL: Wave 4 plan must include reconnaissance of stat1/modal.rs
// NormdModeChoice submit_step to confirm exact key IDs for mode selection.
// The xeq_ΣNORMD magic-prefix dispatches Op::Xeq("ΣNORMD") → xrom_resolve
// → Op::SigmaNormdWorkflow, consistent with xeq_SINH precedent.
it('XEQ "ΣNORMD" x=1.96 upper-tail Q ≈ 0.0250 (Stat 1 Pac via xrom_resolve)', async () => {
    const display = await $('[data-testid="lcd-display"]');
    await display.waitForExist({ timeout: 10000 });

    // Push X = 1.96 via digit clicks (confirms keyboard path for digit entry)
    await clickKey('1');
    await clickKey('decimal');
    await clickKey('9');
    await clickKey('6');
    await clickKey('enter');

    // Open ΣNORMD modal via xeq magic-prefix (bypasses alpha-modal Unicode problem)
    await invokeBackend('dispatch_op', { keyId: 'xeq_ΣNORMD' });

    // Select mode 1 (upper-tail CDF). Key ID TBD from stat1/modal.rs reconnaissance.
    // Fallback: dispatch numeric mode directly as pending_num = 1 then R/S.
    // [PLANNER: read stat1/modal.rs NormdModeChoice to find correct key sequence]
    await clickKey('1');
    const view = await invokeBackend('dispatch_op', { keyId: 'r_s' });

    // Assert on view.display_str (D-11 no-polling invariant, same as SINH test)
    if (!view.display_str.startsWith('0.0250')) {
        throw new Error(
            `expected ΣNORMD(1.96) display_str to start with '0.0250', got '${view.display_str}'`
        );
    }
});
```

**beforeEach state-reset** (reuse existing pattern lines 145–156 — unchanged):
The existing `invokeBackend('dispatch_op', { keyId: 'clx' })` reset covers the new test
automatically. No changes to `beforeEach`.

---

### `docs/hp41-stat1-divergences.md` (MODIFY — append D-35-13 entry)

**Analog:** self (existing D-35-07 through D-35-12 entries — bucket 3 Behavioral Policies shape)

**Entry location:** Append to bucket 3 (Behavioral Policies) as the next numbered entry
after the highest existing `D-35-NN` number. From the file, D-35-07 is visible in bucket 2
at line 80. The CONTEXT.md specifics block provides the complete entry text.

**Five-field entry pattern** (from existing D-35-07 structure):
```markdown
### D-35-13: Bounded-iteration distribution primitives do not wire cancel_requested

**OM citation:** N/A (emulator-internal design choice)

**Our behavior:** Distribution primitives (`norm_cdf_inv_f64`, `gamma_regularized_f64`,
`beta_regularized_f64`) are bounded at `ITER_CAP=50` and complete in microseconds on
all supported platforms. No per-loop `cancel_requested` check is wired.

**OM behavior:** N/A — HP-41C hardware has no cancellation mechanism for built-in math
operations.

**Rationale:** The Phase 31 `cancel_requested` channel (`Arc<AtomicBool>`) was designed
for user-driven open-ended iterative paths (INTG/SOLVE/DIFEQ). Bounded 50-iteration
f64 primitives complete faster than a user can press R/S — wiring cancel would add
per-iteration Arc overhead to a path where cancellation is physically impossible to
trigger. See D-36.2 formal assessment, ADR-v3.1-002 §Iteration Bound.

**See:** `hp41-core/src/ops/stat1/distributions.rs` (`ITER_CAP` constant),
`.planning/phases/36-hp41-gui-gui-integration/36-CONTEXT.md` §D-36.2,
`docs/adr/v3.1-002-distribution-primitives-policy.md`
```

---

### `README.md` (MODIFY — hard-claim graduation)

**Analog:** README.md line 51 (v3.0 Math Pac I hard-claim, "feature-complete per Owner's Manual 00041-90034")

**Soft-claim (current, line 53–54):**
```markdown
- Stat 1 Pac behavioral emulation (13 programs, 26 XEQ entry points,
  RAND/SEED extension, [documented divergences](docs/hp41-stat1-divergences.md)) — see [Stat 1 Pac Function Matrix](docs/hp41-stat1-function-matrix.md)
```

**Hard-claim graduation pattern** (mirror v3.0 Math Pac I claim at line 51):
```markdown
- Stat 1 Pac behavioral emulation, feature-complete per Owner's Manual HP 00041-90030
  (13 programs, 26 XEQ entry points, RAND/SEED extension,
  [documented divergences](docs/hp41-stat1-divergences.md)) — see [Stat 1 Pac Function Matrix](docs/hp41-stat1-function-matrix.md)
```

**Gating condition (D-35.3):** Replace only after STAT-QUAL-04 (numerical accuracy ≥ 98%
with Stat 1 cases) AND STAT-QUAL-11 (E2E smoke extended) are confirmed passing in the
final Wave 4 plan. Mirrors v3.0 D-30.9 → D-32.5 graduation pattern where the same
phase (Phase 32) that verified quality gates also graduated the README claim.

---

## Shared Patterns

### Free42 Disclaim Header
**Source:** `hp41-core/tests/stat1_rand_determinism.rs` lines 1–3
**Apply to:** ALL new stat1 test files (stat1_op_test_count.rs, lint_stat1_assertions.rs, stat1_backward_compat.rs, stat1_modal_coverage.rs, stat1_anova_coverage.rs)
```rust
// Algorithm independently re-derived from HP Stat 1 Pac Owner's Manual 00041-90030 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
```

### `#![allow(clippy::unwrap_used)]` File-Level Allow
**Source:** Every file in `hp41-core/tests/` (analog line 48 in math1_op_test_count.rs, line 75 in lint_math1_assertions.rs, line 9 in synthetic_tests.rs, line 39 in stat1_cancellation.rs)
**Apply to:** ALL new test files — this is the v3.0/v3.1 established pattern for test modules where `expect()` panic-on-failure is intentional.
```rust
#![allow(clippy::unwrap_used)]
```

### `CARGO_MANIFEST_DIR` Path Resolution
**Source:** `hp41-core/tests/math1_op_test_count.rs` lines 203–205
**Apply to:** stat1_op_test_count.rs, lint_stat1_assertions.rs (any test needing filesystem access)
```rust
let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
let tests_dir = manifest_dir.join("tests");
```
This is robust across `cargo test -p hp41-core` vs workspace-root CWD.

### `include_str!` Fixture Loading
**Source:** `hp41-core/tests/stat1_rand_determinism.rs` (inline); `hp41-core/tests/synthetic_tests.rs` lines 270–278 (serde_json pattern)
**Apply to:** stat1_backward_compat.rs
```rust
let json = include_str!("fixtures/v30-autosave.json");
let mut state: CalcState = serde_json::from_str(json)
    .expect("v30-autosave.json must deserialize without error");
```

### E2E `invokeBackend` + `view.display_str` Assertion
**Source:** `hp41-gui/e2e/smoke.spec.js` lines 101–129 (helper), lines 237–243 (assertion pattern)
**Apply to:** New Stat 1 `it()` block in smoke.spec.js
```javascript
const view = await invokeBackend('dispatch_op', { keyId: 'xeq_ΣNORMD' });
if (view.display_str !== '0.0250') {
    throw new Error(
        `expected dispatch_op('xeq_ΣNORMD').display_str='0.0250', got '${view.display_str}'`
    );
}
```
Use `display_str` from the backend response (NOT `data-text` on the LCD element) per D-11
no-polling invariant — `invokeBackend` calls do not propagate into the React tree.

### `#[test] fn` + `// Catches:` Documentation
**Source:** `hp41-core/tests/math1_op_test_count.rs` lines 196–228, `stat1_cancellation.rs` (every test)
**Apply to:** All new test functions in all new test files
```rust
/// <description of what the test asserts>
///
/// Catches: <specific regression class / Pitfall reference>
#[test]
fn <test_name>() {
    // ...
}
```

---

## No Analog Found

All files have close analogs. No entries in this table.

---

## Metadata

**Analog search scope:** `hp41-core/tests/`, `hp41-gui/e2e/`, `docs/`, `hp41-core/tests/fixtures/`, `README.md`
**Files scanned:** 10 analog files read in full; `numerical_accuracy.rs` read in 2 non-overlapping passes (lines 1–150, lines 6925–7014)
**Pattern extraction date:** 2026-05-24
