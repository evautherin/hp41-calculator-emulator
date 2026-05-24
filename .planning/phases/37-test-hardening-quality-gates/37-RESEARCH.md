# Phase 37: Test Hardening & Quality Gates — Research

**Researched:** 2026-05-24
**Domain:** Test infrastructure for HP-41 Stat 1 Pac (XROM 2) surface; coverage enforcement, accuracy oracle extension, meta-gates, E2E smoke, backward-compat migration
**Confidence:** HIGH

## Summary

Phase 37 is a pure-test phase — no implementation changes to `hp41-core/src/`, `hp41-cli/src/`, or `hp41-gui/src-tauri/src/`. It closes the v3.1 milestone by installing permanent CI gates over the Stat 1 Pac surface shipped in Phases 33–36, verifying coverage holds at the v3.0 baseline, extending the numerical accuracy suite with ~30 independently-derived scipy oracle cases, and extending the E2E smoke with one Stat 1 Pac workflow on Ubuntu.

The direct structural template is v3.0 Phase 32. Every artifact in Phase 37 has a named Phase 32 counterpart: `stat1_op_test_count.rs` mirrors `math1_op_test_count.rs`; `lint_stat1_assertions.rs` mirrors `lint_math1_assertions.rs`; the accuracy suite inline extension mirrors the Phase 32 ~134-case block; the E2E `it()` block mirrors the `XEQ "SINH"` and `MATRIX DET` blocks; the Free42 re-verification mirrors Plan 32-03 §license-audit.

**Critical discovery:** Per-file `stat1/*.rs` coverage has already been measured — `stat1/modal.rs` is at **74.21% lines / 79.01% regions**, well below the 90% floor. `stat1/anova.rs` is at **86.50% lines**. `stat1/moments.rs` is at **90.60% lines** (at floor boundary). `stat1/hypothesis.rs` is at **91.68%**. Six of twelve files already meet the 90% floor with headroom. The coverage gap work is not uniformly distributed — `modal.rs` is the primary gap file and needs targeted test additions before `stat1_op_test_count.rs` is added.

**Primary recommendation:** Follow the Phase 32 wave structure exactly. Wave 1: meta-gates (`stat1_op_test_count.rs` + `lint_stat1_assertions.rs` + xrom_shadowing STAT_1 extension). Wave 2: coverage gap closure (targeted for `modal.rs` 74% and `anova.rs` 86%) + backward-compat + STAT-GUI-05 resolution + Free42 re-verification. Wave 3: numerical accuracy ~30 scipy cases. Wave 4: E2E smoke + quality-gate graduation + README hard-claim.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Coverage enforcement | `hp41-core` tests | CI (`just ci`) | Coverage measurement and per-file assertion belong in the test layer; CI runs the gate |
| Per-Op test count meta-gate | `hp41-core` tests | — | `stat1_op_test_count.rs` is a test that reads `xrom.rs` at compile time and scans test files at runtime |
| Assertion-discipline lint | `hp41-core` tests | — | `lint_stat1_assertions.rs` scans `tests/stat1_*.rs` + inline `#[cfg(test)]` modules at test time |
| XROM shadowing gate | `hp41-core` tests | — | Already extended in Phase 33 Plan 33-01; STAT_1 disjointness tests already live in `xrom_shadowing.rs` |
| Numerical accuracy oracle | `hp41-core` tests | — | Inline extension to `numerical_accuracy.rs` |
| Backward-compat migration test | `hp41-core` tests | `hp41-core/tests/fixtures/` | Load fixture JSON, call `migrate_after_load`, assert `xrom_modules` |
| Free42 contamination guard | `scripts/` + CI | — | Already scans both `math1/` and `stat1/`; re-verification is documentation |
| E2E smoke extension | `hp41-gui/e2e/` | `ci-gui.yml::e2e-linux` | Ubuntu-only per D-37.2; WebdriverIO + tauri-driver |
| STAT-GUI-05 resolution | Documentation | `docs/hp41-stat1-divergences.md` | Documented N/A waiver per D-37.9 |
| README hard-claim graduation | `README.md` | `CLAUDE.md` | Final plan; mirrors v3.0 D-32.5 graduation |

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Carried forward from prior phases:**
- **D-36.2 / D-36.3 (STAT-GUI-05 reassignment):** bounded 50-iter distribution primitives (`norm_cdf_inv_f64` closed-form Acklam, `gamma_regularized_f64` gser/gcf ITER_CAP=50, `beta_regularized_f64` betacf ITER_CAP=50) complete in microseconds; cancellation via `request_cancel` `Arc<AtomicBool>` is not applicable. Requirement reassigned Phase 36 → Phase 37 for formal resolution.
- **D-33.8 / STAT-QUAL-09:** Free42 contamination guard already extended from 12→18 tokens in Phase 33 Plan 33-00 (D-32.7 reassignment). Both `math1/` and `stat1/` trees scanned. Phase 37 re-verifies but does not add new tokens.
- **D-32.1 / math1_op_test_count.rs pattern:** scan `xrom.rs` for variant names registered in module-level `_resolve()` function, then count test-function-body mentions across module-prefixed test files. Word-boundary matching to avoid substring inflation (WR-03). ≥ 5 mentions per variant.
- **D-32.1 / lint_math1_assertions.rs pattern:** two lints — `no_decimal_assert_eq` (Pitfall 17) and `no_manual_tolerance_pattern` (Pitfall 14). Line-level grep with LINT-EXEMPT annotations. Multi-line assert_eq detection (WR-02). Scans module-prefixed `tests/math1_*.rs` files.
- **D-27.13 / E2E smoke shape:** WebdriverIO + Mocha + tauri-driver on `ci-gui.yml::e2e-linux`. Plain `.js` spec. `data-key-id` for keyboard clicks, `data-text` attribute on LCD for assertions (14-segment SVG has no getText()).
- **STAT-QUAL-05 (two-level tolerance):** 1e-9 relative for closed-form ops (ΣNORMD CDF/PDF, ΣSPEAR, ΣBSTAT/BSTG, ΣLIN/EXP/LOGI/POW, ΣXSQEV/EFXSQ); 1e-7 relative for iterative paths (probit Φ⁻¹, regularized incomplete gamma/beta, t-CDF, multiple-regression normal-equation solve). Locked in REQUIREMENTS.md.
- **D-35.3 (README soft-claim graduation gating):** hard-claim graduation conditioned on STAT-QUAL-04 (numerical_accuracy ≥ 98 % with Stat 1 cases) + STAT-QUAL-11 (E2E smoke extended). Mirrors v3.0 D-30.9 → D-32.5 graduation pattern.

**Discussed and decided in this session (D-37.1 — D-37.11):**
- **D-37.1:** Fresh scipy derivation for all ~30 Stat 1 accuracy cases — independent validation layer from Phase 33 inline tests.
- **D-37.2:** Ubuntu-only for the Stat 1 E2E smoke extension. `ci-gui.yml::e2e-linux` only.
- **D-37.10:** Wave structure: Wave 1 (meta-gates), Wave 2 (coverage gaps + backward-compat + STAT-GUI-05), Wave 3 (numerical accuracy), Wave 4 (E2E + graduation).
- **D-37.11:** README hard-claim graduation in Phase 37's final plan.

### Claude's Discretion

- **D-37.3 (accuracy case file shape):** Inline extension to `numerical_accuracy.rs` (not separate file).
- **D-37.4 (accuracy case allocation):** ~12–15 iterative-path cases (ΣNORMD inverse/CDF, ΣCHISQD CDF, ΣTSTAT p-value, ΣMLRXY, ΣPOLYP), ~15–18 closed-form cases.
- **D-37.5 (per-case tolerance):** Per-case `tol` field via existing `case!` macro variants (`TOLERANCE` default or `wide` arm mapped to `WIDE_TOL`). For iterative paths needing exactly 1e-7, add a new `case!` arm variant.
- **D-37.6 (coverage gap closure strategy):** Measure first via `cargo llvm-cov` per-file, then write targeted tests only for files below 90%. Primary gap: `stat1/modal.rs` at 74.21%. Secondary gap: `stat1/anova.rs` at 86.50%.
- **D-37.7 (stat1_op_test_count.rs scan scope):** Scan BOTH inline tests inside `stat1/*.rs` source files AND external `tests/stat1_*.rs` files.
- **D-37.8 (lint_stat1_assertions.rs):** New file, scanning `stat1_*.rs` files (both inline and external).
- **D-37.9 (STAT-GUI-05 resolution):** Documented N/A waiver in VERIFICATION.md + behavioral-policy entry in `docs/hp41-stat1-divergences.md`. No code changes.

### Deferred Ideas (OUT OF SCOPE)

- Signed binary releases (cargo-dist CLI + tauri-action GUI) — deferred to v3.1.x / v3.2
- CI-automated coverage gate — currently measured locally per established workflow
- Cross-platform E2E smoke — Ubuntu-only per D-37.2
- `lint_xrom_assertions.rs` unification — defer until third XROM module
- `stat1_op_test_count.rs` → `xrom_op_test_count.rs` unification — defer
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| STAT-QUAL-01 | `hp41-core` line coverage ≥ 95.39% (no regression vs v3.0 baseline) | Current aggregate: 93.53% lines — coverage gap closure in Wave 2 (particularly `stat1/modal.rs` at 74.21%) needed to recover; see Coverage Gap Analysis |
| STAT-QUAL-02 | `hp41-core` region coverage ≥ 94.26% (no regression vs v3.0 baseline) | Current aggregate: 97.27% regions — well above floor; aggregate line coverage is the binding constraint |
| STAT-QUAL-03 | Per-file `stat1/*.rs` coverage floor ≥ 90% | `modal.rs` at 74.21% and `anova.rs` at 86.50% need gap-closure tests; 8 of 12 files already at floor or above |
| STAT-QUAL-04 | `numerical_accuracy.rs` extended with Stat 1 Pac cases at ≥ 98% pass rate; v1.x floor 498/503 and v3.0 floor 763/768 preserved | ~30 independently scipy-derived cases per D-37.1; inline extension per D-37.3; `case!` macro infrastructure already exists |
| STAT-QUAL-05 | Two-level tolerance discipline: 1e-9 for closed-form, 1e-7 for iterative | Enforced via `case!` macro tol parameter; Phase 37 adds `iter` arm to macro; `lint_stat1_assertions.rs` enforces this in external test files |
| STAT-QUAL-06 | `lint_stat1_assertions.rs` blocks `assert_eq!(decimal, decimal)` on iterated results | New file, mirrors `lint_math1_assertions.rs` exactly; two-pass lint (no_decimal_assert_eq + no_manual_tolerance_pattern); LINT-EXEMPT annotation convention preserved |
| STAT-QUAL-07 | `stat1_op_test_count.rs` enforces ≥ 5 tests per Stat 1 Op variant | New file, mirrors `math1_op_test_count.rs`; scans `stat1_resolve` fn body in `xrom.rs`; dual-scan (inline + external) per D-37.7 |
| STAT-QUAL-08 | `xrom_shadowing.rs` STAT_1 extension verifies no Math Pac I / builtin mnemonic shadowed | Already done in Phase 33 Plan 33-01 — `stat1_names_do_not_shadow_builtins`, `stat1_ops_disjoint_from_math1_ops`, `stat1_ops_resolve_via_xrom_resolve` and `stat1_const_fields` tests already exist. Phase 37 formally attests these pass as STAT-QUAL-08 closure. |
| STAT-QUAL-09 | `check-free42-contamination.sh` re-verified over final stat1/*.rs tree | Script already extended to 18 tokens (Phase 33 Plan 33-00); Phase 37 re-runs and documents final token count |
| STAT-QUAL-10 | Backward-compat: v3.0 save file with `"xrom_modules": 1` loads and migrates to bit 1 set | New `stat1_backward_compat.rs` + `tests/fixtures/v30-autosave.json` fixture; `migrate_after_load` asserts bit 1 |
| STAT-QUAL-11 | E2E smoke extended with Stat 1 Pac workflow on Ubuntu | New `it()` block in `smoke.spec.js`: `XEQ "ΣNORMD"` x=1.96 → Q≈0.0250 via `invokeBackend` fallback pattern |
| STAT-GUI-05 | `request_cancel` not needed for bounded 50-iter primitives — formal resolution | Documented N/A waiver in VERIFICATION.md + D-35-13 behavioral-policy entry in `docs/hp41-stat1-divergences.md` |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust built-in `#[test]` | — | Test runner | Zero deps; `cargo test` runs all `tests/` files as integration tests |
| `cargo-llvm-cov` | 0.6+ | Coverage measurement | Already in workflow; `just coverage` recipe |
| `approx` | 0.5.1 [VERIFIED: Cargo.toml] | `assert_relative_eq!` for f64 tolerance assertions in test files | Already in `hp41-core` `[dev-dependencies]`; `max_relative = 1e-7` pattern established |
| `rust_decimal` | 1.42 [VERIFIED: Cargo.toml] | `Decimal::new(mantissa, scale)` for exact HpNum construction in tests | Runtime dep; test files use it for fixture construction |
| WebdriverIO | 9.x [VERIFIED: package.json] | E2E smoke runner | Already configured via `wdio.conf.cjs`; `tauri-driver` speaks WebDriver classic |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde_json` | 1.x [VERIFIED: Cargo.toml] | Serialize/deserialize `CalcState` fixtures | Backward-compat fixture load test |
| `std::fs::read_dir` + `include_str!` | stdlib | File discovery in meta-gate tests | `stat1_op_test_count.rs` and `lint_stat1_assertions.rs` |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Inline `numerical_accuracy.rs` extension | Separate `stat1_numerical_accuracy.rs` | Inline: single unified pass-rate gate; separate: cleaner file but splits the denominator. Per D-37.3: inline wins. |
| New `lint_xrom_assertions.rs` unifying math1+stat1 | Two sibling files | Unification deferred to v3.2 when a third XROM module lands (D-37.8) |

**Installation:** No new dependencies — zero new runtime or dev-dependencies per Phase 37 CONTEXT.md locked invariant. [VERIFIED: CONTEXT.md]

## Package Legitimacy Audit

Phase 37 installs **no new packages**. All tooling is already in the project's dependency graph.

| Package | Registry | Age | Downloads | Source Repo | slopcheck | Disposition |
|---------|----------|-----|-----------|-------------|-----------|-------------|
| (no new packages) | — | — | — | — | — | N/A |

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     Phase 37 Deliverables                   │
│                                                             │
│  Wave 1 (Meta-gates)                                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ stat1_op_test_count.rs                               │  │
│  │   reads xrom.rs ──► stat1_resolve scope              │  │
│  │   ──► extract 26 variant names                       │  │
│  │   ──► scan tests/stat1_*.rs + src/ops/stat1/*.rs     │  │
│  │   ──► assert ≥ 5 test-function mentions each         │  │
│  │                                                      │  │
│  │ lint_stat1_assertions.rs                             │  │
│  │   scans stat1_*.rs (external + inline #[cfg(test)])  │  │
│  │   ──► no assert_eq!(decimal, decimal) (Pitfall 17)   │  │
│  │   ──► no (a-b).abs() < eps (Pitfall 14)              │  │
│  │                                                      │  │
│  │ xrom_shadowing.rs [already done in Phase 33]         │  │
│  │   ──► STAT-QUAL-08 attestation                       │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  Wave 2 (Coverage + compat + waiver)                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ cargo llvm-cov ──► stat1/modal.rs: 74.21% (gap!)    │  │
│  │                    stat1/anova.rs: 86.50% (gap!)    │  │
│  │                                                      │  │
│  │ tests/stat1_modal_coverage.rs (new — gap closure)   │  │
│  │ tests/stat1_anova_coverage.rs (new — gap closure)   │  │
│  │                                                      │  │
│  │ tests/stat1_backward_compat.rs (new)                │  │
│  │   loads tests/fixtures/v30-autosave.json            │  │
│  │   ──► migrate_after_load ──► xrom_modules == 0b11   │  │
│  │                                                      │  │
│  │ STAT-GUI-05: docs/hp41-stat1-divergences.md D-35-13 │  │
│  │ scripts/check-free42-contamination.sh re-verify     │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  Wave 3 (Numerical accuracy)                                │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ numerical_accuracy.rs inline extension (~30 cases)  │  │
│  │   scipy-derived ──► case! macro entries             │  │
│  │   tol=TOLERANCE (1e-9) for closed-form              │  │
│  │   tol=WIDE_TOL (1e-6) or new ITER_TOL (1e-7) arm   │  │
│  │   for iterative paths                               │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  Wave 4 (E2E + graduation)                                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ hp41-gui/e2e/smoke.spec.js: new it() block          │  │
│  │   invokeBackend('dispatch_op', {keyId:'xeq_ΣNORMD'})│  │
│  │   ──► (modal flow) ──► Q(1.96) = 0.0250            │  │
│  │                                                      │  │
│  │ just ci + just gui-ci ──► quality-gate graduation   │  │
│  │ README.md hard-claim + CLAUDE.md Phase 37 entry     │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure
```
hp41-core/tests/
├── stat1_op_test_count.rs       # NEW — Wave 1
├── lint_stat1_assertions.rs     # NEW — Wave 1
├── stat1_backward_compat.rs     # NEW — Wave 2
├── stat1_modal_coverage.rs      # NEW — Wave 2 (gap: 74%)
├── stat1_anova_coverage.rs      # NEW — Wave 2 (gap: 86%)
├── stat1_rand_determinism.rs    # EXISTING — unchanged
├── stat1_cancellation.rs        # EXISTING — unchanged
├── numerical_accuracy.rs        # EXISTING — inline extension Wave 3
└── fixtures/
    ├── v20-autosave.json        # EXISTING
    └── v30-autosave.json        # NEW — Wave 2

hp41-gui/e2e/
└── smoke.spec.js                # EXISTING — new it() block Wave 4
```

### Pattern 1: stat1_op_test_count.rs — Dual-Scan Meta-Gate

**What:** Scans `stat1_resolve` function body in `src/ops/math1/xrom.rs` (the STAT_1 block) for variant names, then counts `#[test]` function mentions across both (a) external `tests/stat1_*.rs` files AND (b) inline `#[cfg(test)]` modules inside `src/ops/stat1/*.rs` files. Asserts ≥ 5 mentions per variant.

**When to use:** Phase 37 Wave 1. Mirrors `math1_op_test_count.rs` but with dual-scan scope (D-37.7) because Stat 1 has 189 inline tests vs. Math Pac I's mostly external tests.

**Key adaptation from math1 template:** [VERIFIED: math1_op_test_count.rs]
- Scope the variant scan to `fn stat1_resolve` (not `fn math1_resolve`) — brace-depth tracking unchanged
- Glob `stat1_*.rs` test files from `tests/` dir (same as math1 analog)
- ADDITIONALLY scan inline `#[cfg(test)]` modules in `src/ops/stat1/*.rs` (new — D-37.7)
- Word-boundary matching unchanged (WR-03 fix)

```rust
// Source: math1_op_test_count.rs structural template [VERIFIED: codebase]
// Key adaptation: stat1_resolve scope + dual-scan (external + inline)
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
                if !variant_name.is_empty() { variants.push(variant_name); }
            }
        }
        if brace_depth <= 0 { in_stat1_resolve = false; }
    }
    variants
}
```

### Pattern 2: lint_stat1_assertions.rs — Two-Pass Scope-Extended Lint

**What:** Scans `stat1_*.rs` external test files AND inline `#[cfg(test)]` modules in `src/ops/stat1/*.rs` for (1) `assert_eq!(decimal, decimal)` on float/HpNum (Pitfall 17) and (2) `(a - b).abs() < eps` manual-tolerance (Pitfall 14). LINT-EXEMPT annotation convention preserved.

**When to use:** Phase 37 Wave 1. Mirrors `lint_math1_assertions.rs` exactly with scope extended to include inline stat1 source modules.

**Key adaptation from math1 template:** [VERIFIED: lint_math1_assertions.rs]
- Replace `filename.starts_with("math1_")` with `filename.starts_with("stat1_")` for external files
- Add a second scan pass over `src/ops/stat1/*.rs` extracting inline `#[cfg(test)] mod tests { ... }` blocks
- All lint logic (`line_is_forbidden_assert_eq`, `line_is_forbidden_manual_tolerance`, `preceding_block_has_lint_exempt`) reused verbatim

### Pattern 3: Backward-Compat Fixture Test

**What:** Load `tests/fixtures/v30-autosave.json` (a serialized `CalcState` with `"xrom_modules": 1`), deserialize, call `migrate_after_load()`, assert `state.xrom_modules == 0b0000_0011`.

**When to use:** Phase 37 Wave 2.

**v30-autosave.json shape** (minimal — mirrors `v20-autosave.json` pattern): [VERIFIED: v20-autosave.json fixture]
```json
{
  "stack": {"x":"0","y":"0","z":"0","t":"0","lastx":"0","lift_enabled":true},
  "regs": [<64 zeros>],
  "alpha_reg": "", "alpha_mode": false, "angle_mode": "Deg",
  "display_mode": {"Fix": 4},
  "entry_buf": "", "program": [], "prgm_mode": false, "pc": 0,
  "call_stack": [], "is_running": false, "user_mode": false,
  "key_assignments": {}, "last_key_code": 0,
  "reg_m": "0", "reg_n": "0", "reg_o": "0",
  "xrom_modules": 1
}
```

The `rand_seed` field is intentionally ABSENT — testing that `#[serde(default)]` provides `HpNum::zero()` and `migrate_after_load` sets `xrom_modules` bit 1 regardless.

```rust
// Source: synthetic_tests.rs structural template [VERIFIED: codebase]
#[test]
fn v30_save_loads_with_stat1_migration() {
    let json = include_str!("fixtures/v30-autosave.json");
    let mut state: CalcState = serde_json::from_str(json)
        .expect("v30-autosave.json must deserialize");
    assert_eq!(state.xrom_modules, 1, "pre-migration: v3.0 default has only bit 0 set");
    hp41_core::state::migrate_after_load(&mut state);
    assert_eq!(
        state.xrom_modules, 0b0000_0011,
        "post-migration: bit 1 must be set (Stat 1 Pac auto-enabled per STAT-FW-02)"
    );
}
```

### Pattern 4: numerical_accuracy.rs — Stat 1 Inline Extension

**What:** Append ~30 `case!()` entries after the existing 768 cases. Each case carries:
- `// Source: HP Stat 1 Pac OM 00041-90030 (1979), p.<n> — <description>` OR `// Source: scipy.stats.<fn>(<args>) = <value>`
- `// Free42: N/A — Stat 1 Pac oracle derivation; scipy.stats used as ground truth`
- `// Catches: <bug class per D-27.1 risk-weighted discipline>`

**Tolerance discipline:** The existing `case!` macro has `TOLERANCE = 1e-9` (default arm) and `WIDE_TOL = 1e-6` (wide arm). Per D-37.5, a third `iter` arm should be added for iterative paths:

```rust
// Source: numerical_accuracy.rs case! macro [VERIFIED: codebase]
// Addition: new `iter` arm for Stat 1 iterative-path tolerance (1e-7)
const ITER_TOL: f64 = 1e-7;  // Two-level tolerance per STAT-QUAL-05

macro_rules! case {
    // ... existing arms ...
    ($domain:expr, $desc:expr, $expected:expr, $actual:expr, iter) => {{
        id += 1;
        cases.push(AccuracyCase {
            id, domain: $domain, description: $desc.to_string(),
            expected: $expected, actual: $actual, tol: ITER_TOL,
        });
    }};
}
```

**Case distribution per D-37.4:**

| Stat 1 Family | Cases | Tol arm | Key ops |
|--------------|-------|---------|---------|
| ΣNORMD CDF (upper-tail Q) | 4 | `default` (1e-9) | Q(1.96)≈0.025, Q(0)=0.5, Q(2.576)≈0.005, Q(-1.96)≈0.975 |
| ΣNORMD PDF | 2 | `default` (1e-9) | φ(0)=0.3989..., φ(1) |
| ΣNORMD inverse (probit) | 4 | `iter` (1e-7) | Φ⁻¹(0.975)≈1.96, Φ⁻¹(0.5)=0, Φ⁻¹(0.025)≈-1.96, tail |
| ΣCHISQD CDF | 3 | `iter` (1e-7) | P(χ²≤3.84; ν=1), P(χ²≤5.99; ν=2) |
| ΣTSTAT p-value | 2 | `iter` (1e-7) | Standard two-sample cases |
| ΣSPEAR rank correlation | 2 | `default` (1e-9) | ρ_s=0.8 canonical + boundary ρ_s=1 |
| ΣBSTAT / ΣBSTG | 3 | `default` (1e-9) | CV, mean, StdDev |
| ΣLIN / ΣEXP / ΣLOGI / ΣPOW | 4 | `default` (1e-9) | Slope + intercept regression |
| ΣXSQEV / ΣEFXSQ | 2 | `default` (1e-9) | χ² goodness-of-fit |
| ΣMLRXY regression | 2 | `iter` (1e-7) | Normal-equation solve coefficient |
| ΣAOVONE F-ratio | 2 | `default` (1e-9) | F=50.0 canonical + small dataset |

**Total: ~30 cases**. Combined suite becomes ~798 cases; ≥ 98 % gate = ≥ 782 passing.

**Important: v3.0 floor preservation.** The current `numerical_accuracy.rs` tracks `baseline_passes` (v1.x, first 503 cases) with pinned failing-ID set `[124, 279, 344, 438, 480]`. There is no explicit v3.0 768-case floor assertion in the current code — the combined ≥ 98% gate subsumes it. Phase 37 CONTEXT.md states "v3.0 768-case floor 763/768 preserved" — this means the Stat 1 extension must not break any of the 763 currently-passing math1 cases (trivially satisfied since new cases are appended, not inserted). The combined floor will be `ceil(798 × 0.98) = 783` passes. [VERIFIED: codebase analysis]

### Pattern 5: E2E Smoke — ΣNORMD Extension

**What:** Add a new `it()` block to `smoke.spec.js` using the `invokeBackend` fallback pattern (established for SINH and MATRIX DET). The ΣNORMD mnemonic contains Σ (Unicode U+03A3) which is not typeable via the alpha-modal click path — the fallback is mandatory.

**Key ID:** The `data-key-id` for the XEQ ΣNORMD dispatch will be `xeq_ΣNORMD` (the magic-prefix pattern for XROM ops, matching the `xeq_SINH` / `xeq_DET` / `xeq_MATRIX` precedent). [VERIFIED: smoke.spec.js + key_map.rs pattern]

**ΣNORMD workflow for E2E:**

1. ΣNORMD opens a modal asking for mode selection: `MODE=?` prompt (or equivalent)
2. The `[E]` key selects upper-tail CDF mode
3. User enters x=1.96
4. R/S submits → computes Q(1.96) ≈ 0.025000

**Challenge:** The ΣNORMD modal flow is multi-step (mode selection + x entry). Per the `invokeBackend` fallback pattern, all dispatches go through `dispatch_op`. The mode selection key and the x entry both need to go through the modal routing in `modal.rs::submit_step`. The planner must verify the exact key IDs for mode selection (`[E]` in alpha notation = shift-5 or similar) and the modal prompt sequence from `stat1/modal.rs`.

**Recommendation:** Use the same `invokeBackend` + `view.display_str` assertion pattern as SINH (not `data-text` on LCD). Assert `view.display_str` from the final `dispatch_op` call rather than waiting for React re-render (D-11 — no polling invariant). [VERIFIED: smoke.spec.js]

```javascript
// Source: smoke.spec.js invokeBackend pattern [VERIFIED: codebase]
it('XEQ "ΣNORMD" x=1.96 upper-tail Q ≈ 0.0250 (Stat 1 Pac via xrom_resolve)', async () => {
    const display = await $('[data-testid="lcd-display"]');
    await display.waitForExist({ timeout: 10000 });

    // Push X = 1.96 via digit clicks first (establishes state for the modal path)
    await clickKey('1');
    await clickKey('decimal');
    await clickKey('9');
    await clickKey('6');
    await clickKey('enter');

    // Open ΣNORMD workflow. The mnemonic is Unicode (Σ = U+03A3) — use
    // magic-prefix xeq_ΣNORMD to bypass the alpha-modal typing problem.
    // ΣNORMD opens a mode-selection modal (MODE=? per stat1/modal.rs).
    await invokeBackend('dispatch_op', { keyId: 'xeq_ΣNORMD' });

    // Submit mode=[E] (upper-tail CDF). The mode key must be verified from
    // stat1/modal.rs::Stat1Step::NormdModePrompt submit semantics.
    // Planner Wave 4 Wave-0 reconnaissance: read stat1/modal.rs to find
    // the exact Stat1Step and dispatch key for mode=[E].
    // [PLACEHOLDER — exact key sequence requires stat1/modal.rs inspection]

    // Assert via view.display_str (D-11 no-polling pattern)
    // const view = await invokeBackend('dispatch_op', { keyId: '...' });
    // if (!view.display_str.startsWith('0.0250')) { throw new Error(...); }
});
```

**CRITICAL PLANNER NOTE:** The Wave 4 plan must include a "reconnaissance" sub-task to read `hp41-core/src/ops/stat1/modal.rs` and `hp41-core/src/ops/stat1/normd.rs` to map the ΣNORMD mode-selection modal to specific `dispatch_op` key IDs. This is the same reconnaissance pattern used in Phase 32 Open Question #1 for MATRIX DET.

### Anti-Patterns to Avoid

- **Scanning `math1_resolve` scope for stat1 variants:** `math1_op_test_count.rs` explicitly scopes to `fn math1_resolve` to avoid picking up stat1 variant names. `stat1_op_test_count.rs` must scope to `fn stat1_resolve` equivalently. [VERIFIED: math1_op_test_count.rs lines 65-70]
- **Forgetting the LINT-EXEMPT annotation on integer-equality `assert_eq!` calls:** `stat1_rand_determinism.rs` and `stat1_cancellation.rs` have `assert_eq!(state.rand_seed, expected)` which fires the decimal-lint (rand_seed is HpNum). These need `// LINT-EXEMPT: integer-construction via Decimal::new — no f64 bridge` annotations.
- **Using `browser.waitUntil` with a noop predicate:** The existing `smoke.spec.js` has a timing pattern bug (`return true` inside waitUntil) that was acceptable for intermediate wait points but should not be replicated in new assertions. Use `view.display_str` from `invokeBackend` return value instead.
- **Adding `serde(skip)` to `rand_seed`:** The `rand_seed` field uses `#[serde(default)]` WITHOUT `#[serde(skip)]` — this is documented as the ONLY v3.1 field with this shape (STAT-RNG-03 / Pitfall 20). The v30-autosave.json fixture is designed to omit `rand_seed` so `#[serde(default)]` is exercised.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| f64 relative equality assertions | `(a-b).abs() < eps` | `approx::assert_relative_eq!(actual, expected, max_relative = 1e-7)` | Already in dev-deps; `lint_stat1_assertions.rs` will block the manual pattern |
| XROM variant discovery | Custom enum iteration | `include_str!` + brace-depth scan of `xrom.rs` | Established pattern from `math1_op_test_count.rs`; no `strum` needed |
| Coverage per-file breakdown | Custom reporting script | `cargo llvm-cov --html -p hp41-core` | Existing `just coverage` recipe; HTML output shows per-file breakdown |
| scipy oracle derivation | Guess from OM estimates | Independent Python session: `from scipy.stats import norm, chi2, t; norm.sf(1.96)` | Avoids planning-phase estimate drift (D-35-01 through D-35-06 from Phase 33/35 experience) |

**Key insight:** The 33-SPEC-AMENDMENT.md documents 6 planning-phase oracle estimate drifts (D-35-01..D-35-06). Every Phase 37 accuracy case oracle MUST be freshly derived from scipy, NOT carried from Phase 33 inline test expectations (even when they agree, independent derivation is required per D-37.1).

## Coverage Gap Analysis

**Current per-file coverage** (measured 2026-05-24 via `just coverage`): [VERIFIED: live coverage run]

| Source file | Lines | Miss | Line% | Regions | Miss | Region% | Status vs 90% |
|-------------|------:|-----:|------:|--------:|-----:|--------:|:--------------|
| `anova.rs` | 726 | 98 | **86.50%** | 282 | 19 | 93.26% | **BELOW FLOOR** |
| `basic_stats.rs` | 337 | 10 | 97.03% | 178 | 0 | 100.00% | OK |
| `chisqd.rs` | 304 | 18 | 94.08% | 142 | 0 | 100.00% | OK |
| `distributions.rs` | 507 | 12 | 97.63% | 329 | 5 | 98.48% | OK |
| `hypothesis.rs` | 601 | 50 | **91.68%** | 292 | 7 | 97.60% | Marginal OK |
| `mod.rs` | 30 | 0 | 100.00% | 43 | 0 | 100.00% | OK |
| `modal.rs` | 252 | 65 | **74.21%** | 181 | 38 | 79.01% | **CRITICAL GAP** |
| `moments.rs` | 447 | 42 | **90.60%** | 177 | 0 | 100.00% | At floor boundary |
| `nonparam.rs` | 731 | 31 | 95.76% | 379 | 2 | 99.47% | OK |
| `normd.rs` | 260 | 7 | 97.31% | 125 | 1 | 99.20% | OK |
| `rand.rs` | 277 | 7 | 97.47% | 153 | 1 | 93.75% | OK |
| `regression.rs` | 901 | 23 | 97.45% | 446 | 4 | 99.10% | OK |

**Also affecting aggregate line coverage:**
- `ops/mod.rs`: 92.85% (28 missed regions — a v2.2 file, not stat1)
- `ops/program.rs`: 82.09% lines / 85.55% regions (large file, 155 missed regions)
- `ops/stats.rs`: 85.80% (75 missed lines — shared Σ-register accumulator)
- Current aggregate: **93.53% lines / 97.27% regions** (target: 95.39% / 94.26%)

**The binding constraint:** The aggregate line coverage of 93.53% is below the 95.39% target. The dominant gap contributors are `stat1/modal.rs` (74.21% with 65 missed lines), `ops/program.rs` (82.09% with 366 missed lines), `ops/stats.rs` (85.80% with 75 missed lines), and `stat1/anova.rs` (86.50% with 98 missed lines).

**Gap-closure strategy:** Phase 37 targets only `stat1/*.rs` files (the scope of this phase). Targeting `modal.rs` and `anova.rs` will contribute the most per-line improvement in the stat1 subtree. Whether this lifts the aggregate to 95.39% depends on the total line denominator. The planner must run `just coverage --html` as Wave 2's first step and measure the contribution of stat1 gap closure to the aggregate metric.

**`stat1/modal.rs` gap analysis** (74.21%, 65 missed lines, 38 missed regions):
- 532 LOC source, 10 inline tests — likely missing branches for: error paths in `submit_step` for each `Stat1Step` variant, cancellation paths, edge cases in modal initialization, `cancel_step` paths for multi-step modals (ΣPOLYP degree prompt, ΣCHISQD ν entry, SEED prompt)
- Target new external test file: `tests/stat1_modal_coverage.rs`

**`stat1/anova.rs` gap analysis** (86.50%, 98 missed lines, 19 missed regions):
- 509 LOC source, 8 inline tests — likely missing: ΣAOVTWO two-way with 0-cell error path, ΣANOCOV covariate adjustment edge case, SIZE-guard path (fewer registers than needed)
- Target new external test file: `tests/stat1_anova_coverage.rs`

## Per-Op Test Count Audit (STAT-QUAL-07)

**26 STAT_1 Op variants** (from `STAT_1.ops` slice, `xrom.rs:141-186`): [VERIFIED: xrom.rs]

| Op variant | Mnemonic | Category |
|------------|----------|----------|
| `SigmaBstat` | ΣBSTAT | Univariate |
| `SigmaBstg` | ΣBSTG | Univariate |
| `SigmaMmtug` | ΣMMTUG | Univariate |
| `SigmaMmtgd` | ΣMMTGD | Univariate |
| `SigmaAovone` | ΣAOVONE | ANOVA |
| `SigmaAovtwo` | ΣAOVTWO | ANOVA |
| `SigmaAnocov` | ΣANOCOV | ANOVA |
| `SigmaLin` | ΣLIN | Regression |
| `SigmaExp` | ΣEXP | Regression |
| `SigmaLogi` | ΣLOGI | Regression |
| `SigmaPow` | ΣPOW | Regression |
| `SigmaMlrxy` | ΣMLRXY | Regression |
| `SigmaMlrxyz` | ΣMLRXYZ | Regression |
| `SigmaPolypWorkflow` | ΣPOLYP | Regression |
| `SigmaPolyc` | ΣPOLYC | Regression |
| `SigmaPtst` | ΣPTST | Hypothesis |
| `SigmaTstat` | ΣTSTAT | Hypothesis |
| `SigmaXsqev` | ΣXSQEV | Hypothesis |
| `SigmaEfxsq` | ΣEFXSQ | Hypothesis |
| `SigmaCtkkk` | ΣCTKKK | Hypothesis |
| `SigmaCtkk` | ΣCTKK | Hypothesis |
| `SigmaSpear` | ΣSPEAR | Nonparam |
| `SigmaNormdWorkflow` | ΣNORMD | Distributions |
| `SigmaChisqdWorkflow` | ΣCHISQD | Distributions |
| `Rand` | RAND | RNG |
| `Seed` | SEED | RNG |

**Pre-audit assessment:** With 189 inline tests distributed across 12 source files, and 2 existing external test files (`stat1_rand_determinism.rs` with 3 tests, `stat1_cancellation.rs` with 3 tests), the majority of the 26 variants likely already have ≥ 5 test mentions in the combined scope. The dual-scan approach (D-37.7) will capture inline tests that the external-only scan would miss. Risk variants (those with fewer inline tests): `SigmaMmtgd` (grouped moments, less tested than `SigmaMmtug`), `SigmaAnocov` (ANCOVA, 8 inline tests in anova.rs serving 3 ANOVA variants), `SigmaPolyc` (polynomial predict, tested via ΣPOLYP workflow primarily). The meta-gate may surface 2–5 variants below the threshold that need additional coverage-gap tests.

## XROM Shadowing Gate Status (STAT-QUAL-08)

**Already implemented** in Phase 33 Plan 33-01. [VERIFIED: xrom_shadowing.rs]

`hp41-core/tests/xrom_shadowing.rs` already contains:
- `stat1_names_do_not_shadow_builtins()` — verifies STAT_1.ops against `BUILTIN_CARD_OP_NAMES`
- `stat1_ops_disjoint_from_math1_ops()` — verifies STAT_1.ops ∩ MATH_1.ops = ∅
- `stat1_ops_resolve_via_xrom_resolve()` — bidirectional consistency check
- `stat1_const_fields()` — STAT_1.id = 2, STAT_1.name = "STAT 1B"

**Phase 37 action:** Formally attest STAT-QUAL-08 as satisfied in VERIFICATION.md. No code changes needed.

## Free42 Contamination Guard Status (STAT-QUAL-09)

**Already implemented** in Phase 33 Plan 33-00. [VERIFIED: check-free42-contamination.sh]

Current script scans both `hp41-core/src/ops/math1/` and `hp41-core/src/ops/stat1/` with 18-token pattern (12 math1-era + 6 stat1-era prefix tokens). Script has WR-01 directory-existence guard. Exits 0 on no matches, 1 on contamination, 2 on missing directory.

**Phase 37 action:** Run `bash scripts/check-free42-contamination.sh` and document result. If it exits 0, record final token count (18) in VERIFICATION.md. No new tokens needed per D-33.8 lock.

## Common Pitfalls

### Pitfall 1: `stat1_op_test_count.rs` Scanning the Wrong `_resolve` Function
**What goes wrong:** `xrom.rs` contains both `fn math1_resolve` and `fn stat1_resolve`. Without scope tracking, the scanner may pick up Math Pac I variants from the math1_resolve scope and count them against the stat1 gate.
**Why it happens:** `include_str!` loads the entire xrom.rs; substring scan of `Some(Op::` without function-scope tracking hits both resolve functions.
**How to avoid:** Reuse the brace-depth `in_stat1_resolve` tracking from `math1_op_test_count.rs::in_math1_resolve`. The existing file already documents this concern at lines 65-70. [VERIFIED: math1_op_test_count.rs]
**Warning signs:** `collect_stat1_variant_names()` returning 45 variants (the Math Pac I count) instead of 26.

### Pitfall 2: `lint_stat1_assertions.rs` False-Positive on `stat1_rand_determinism.rs`
**What goes wrong:** `stat1_rand_determinism.rs` contains `assert_eq!(state.rand_seed, expected)` and `assert_eq!(state.stack.x, expected)` — both sides are `HpNum`, which triggers the decimal-equality lint.
**Why it happens:** The rand determinism tests use exact HpNum equality (LINT-correct behavior — `rand_seed` is managed as an exact Decimal, not a float result). The lint would fire without LINT-EXEMPT annotation.
**How to avoid:** Add `// LINT-EXEMPT: exact HpNum equality via Decimal::new construction — no f64 bridge; rand_seed is a Decimal-exact LCG accumulator` annotations to the affected lines in `stat1_rand_determinism.rs` before Wave 1 lands `lint_stat1_assertions.rs`. [VERIFIED: stat1_rand_determinism.rs]
**Warning signs:** `no_decimal_assert_eq_in_stat1_tests` fails immediately after Wave 1 lands.

### Pitfall 3: `case!` Macro Missing `iter` Variant for 1e-7 Iterative Paths
**What goes wrong:** Phase 37 accuracy cases for ΣNORMD inverse, ΣCHISQD CDF, ΣTSTAT p-value, and ΣMLRXY regression need 1e-7 relative tolerance per STAT-QUAL-05. The existing `case!` macro only has `TOLERANCE=1e-9` (default) and `WIDE_TOL=1e-6` (wide arm).
**Why it happens:** STAT-QUAL-05 defines a DIFFERENT tier than WIDE_TOL — 1e-7 iterative is tighter than 1e-6 wide.
**How to avoid:** Add a new `iter` arm to the `case!` macro at the top of the Stat 1 extension block (does not modify the existing arms), using `ITER_TOL: f64 = 1e-7` constant. [VERIFIED: numerical_accuracy.rs lines 100-123]
**Warning signs:** Using `wide` arm (1e-6) for iterative cases — passes but is too lenient; using default arm (1e-9) — fails for iterative paths.

### Pitfall 4: E2E ΣNORMD Modal Flow Key IDs Not Verified Before Plan
**What goes wrong:** The planner hard-codes a ΣNORMD key sequence without reading `stat1/modal.rs` first. The `Stat1Step::NormdModePrompt` submit semantics may differ from the planning-phase assumption.
**Why it happens:** The ΣNORMD modal involves mode selection (`[E]` for upper-tail CDF) before x entry. The key_map key IDs for mode selection in an active ΣNORMD modal are not documented in CONTEXT.md.
**How to avoid:** Wave 4 plan must include a "reconnaissance" Wave-0 sub-task: read `stat1/modal.rs` and `stat1/normd.rs` to map the mode-selection flow. Fallback: use `invokeBackend` for ALL steps including mode selection (dispatch mode key directly via key_id).
**Warning signs:** E2E test `it()` block failing at the mode-selection step with a stale/unexpected modal state.

### Pitfall 5: aggregate coverage gate missed despite per-file stat1 gates passing
**What goes wrong:** After closing `modal.rs` (74.21%) and `anova.rs` (86.50%) gaps, the aggregate `hp41-core` line coverage is still below 95.39% because `ops/program.rs` (82.09%, 366 missed lines) and `ops/stats.rs` (85.80%, 75 missed lines) dominate the aggregate miss count.
**Why it happens:** These are pre-existing v2.2 files, not stat1 files — Phase 37 is out-of-scope for them per CONTEXT.md.
**How to avoid:** Run `just coverage` BEFORE declaring Wave 2 complete and check aggregate. If aggregate is still below 95.39%, escalate to the planner for evaluation: either (a) the gap was already present in v3.0 and the target baseline was measured differently, or (b) Stat 1 itself contributes more uncovered lines than expected.
**Warning signs:** `just coverage` reporting < 95.39% even after all per-file stat1 gaps are closed.

### Pitfall 6: `v30-autosave.json` fixture missing required fields
**What goes wrong:** `CalcState` deserialization fails with `missing field` error if the fixture omits a field that was added in v3.0 without `#[serde(default)]`.
**Why it happens:** v3.0 introduced `matrix_dim`, `modal_program`, `integ_state`, `solve_state`, `cancel_requested`, `xrom_modules` — these should all have `#[serde(default)]` or `#[serde(skip)]`.
**How to avoid:** Generate the fixture by serializing a real `CalcState::new()` (with `xrom_modules: 1` patched in) rather than hand-crafting it. Alternatively, use `serde_json::from_str` and watch for panics in test; the v20-autosave.json fixture is hand-crafted and works because all new fields have `#[serde(default)]`.
**Warning signs:** `serde_json::from_str` failing with deserialization error in `v30_save_loads_with_stat1_migration` test.

## Code Examples

### stat1_op_test_count.rs — Dual-Scan count_test_mentions

```rust
// Source: math1_op_test_count.rs structural template [VERIFIED: codebase]
// Extended for Phase 37: counts mentions in BOTH external tests/stat1_*.rs
// AND inline #[cfg(test)] modules inside src/ops/stat1/*.rs (D-37.7).
fn count_stat1_test_mentions(variant_name: &str, tests_dir: &Path) -> usize {
    let mut total = 0;

    // Pass 1: external tests/stat1_*.rs files
    if let Ok(entries) = std::fs::read_dir(tests_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") { continue; }
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !filename.starts_with("stat1_") { continue; }
            if let Ok(content) = std::fs::read_to_string(&path) {
                total += count_in_content(&content, variant_name);
            }
        }
    }

    // Pass 2: inline #[cfg(test)] modules in src/ops/stat1/*.rs
    let src_stat1 = tests_dir.parent().unwrap().join("src").join("ops").join("stat1");
    if let Ok(entries) = std::fs::read_dir(&src_stat1) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") { continue; }
            if let Ok(content) = std::fs::read_to_string(&path) {
                // Extract only #[cfg(test)] blocks to avoid counting production code
                for test_block in content.split("#[cfg(test)]").skip(1) {
                    total += count_in_content(test_block, variant_name);
                }
            }
        }
    }
    total
}
```

### Fixture Generation Helper

```rust
// For generating v30-autosave.json (run once, save output to tests/fixtures/):
// fn main() {
//     let mut state = hp41_core::CalcState::new();
//     state.xrom_modules = 1; // v3.0 default — only Math 1 enabled
//     let json = serde_json::to_string_pretty(&state).unwrap();
//     println!("{}", json);
// }
```

### scipy Oracle Derivation Session (Python reference)

```python
# Source: scipy.stats independent derivation per D-37.1 [ASSUMED — to be run by implementer]
from scipy.stats import norm, chi2, t
from scipy.stats import spearmanr
import numpy as np

# ΣNORMD upper-tail CDF Q(x) = 1 - Φ(x) = norm.sf(x)
norm.sf(1.96)      # → 0.02499789...  display as 0.0250 in FIX 4
norm.sf(0.0)       # → 0.5            display as 0.5000
norm.sf(2.576)     # → 0.00500...     display as 0.0050
norm.pdf(0.0)      # → 0.39894...     display as 0.3989

# ΣNORMD inverse (probit) Φ⁻¹(p) = norm.ppf(p)
norm.ppf(0.975)    # → 1.95996...     display as 1.9600 in FIX 4
norm.ppf(0.5)      # → 0.0            display as 0.0000
norm.ppf(0.025)    # → -1.95996...    display as -1.9600

# ΣCHISQD CDF P(x; ν) = chi2.cdf(x, df=ν)
chi2.cdf(3.841, df=1)   # → 0.95000...  display as 0.9500
chi2.cdf(5.991, df=2)   # → 0.95002...  display as 0.9500

# ΣSPEAR rank correlation
spearmanr([1,2,3,4,5], [5,6,7,8,7]).statistic  # → 0.8... varies by pair

# ΣBSTAT coefficient of variation CV = σ/μ (sample σ, n-1)
x = np.array([1.0, 2.0, 3.0, 4.0, 5.0])
np.std(x, ddof=1) / np.mean(x)  # → 0.52704...  (D-35-05 corrected oracle)
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Phase 32 single-scan for math1 variants (external tests only) | Phase 37 dual-scan for stat1 variants (external + inline) | Phase 37 — D-37.7 | Matches actual test distribution (189 inline tests) |
| Planning-phase oracle estimates in SPEC.md | scipy-derived independent oracle values | Phase 33-35 (D-35-01..D-35-06) | 6 oracle drifts documented; prevents same error in Phase 37 |
| `TOLERANCE` (1e-9) / `WIDE_TOL` (1e-6) two-level | Three-level: `TOLERANCE` (1e-9) / `ITER_TOL` (1e-7) / `WIDE_TOL` (1e-6) | Phase 37 | Matches STAT-QUAL-05 two-level discipline |

**Deprecated/outdated in Phase 37 context:**
- The v3.0 CONTEXT.md `"STAT-QUAL-08 pending"` annotation: already completed in Phase 33 Plan 33-01 (`xrom_shadowing.rs` extended). Phase 37 formally attests it.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The aggregate `hp41-core` line coverage can reach 95.39% after closing `stat1/modal.rs` and `stat1/anova.rs` gaps | Coverage Gap Analysis | If `ops/program.rs` and `ops/stats.rs` dominate the aggregate miss, Phase 37 cannot hit 95.39% without going out of scope — planner must evaluate |
| A2 | `stat1/modal.rs` gap (74.21%) is due to untested `submit_step` branches for each `Stat1Step` variant and cancel paths | Coverage Gap Analysis | If the gap is in unreachable dead code, fewer tests are needed; if in error paths, targeted tests suffice |
| A3 | The ΣNORMD E2E modal flow can be driven entirely via `invokeBackend` without needing real click sequences for mode selection | E2E Pattern 5 | If `dispatch_op` with a mode-key ID does not work inside the active modal, the E2E test needs a different strategy — Wave 4 reconnaissance resolves this |
| A4 | All 26 Stat 1 variants will have ≥ 5 test mentions after the dual-scan, except possibly `SigmaMmtgd`, `SigmaAnocov`, `SigmaPolyc` | Per-Op Test Count Audit | If fewer variants meet the threshold, additional coverage-gap tests must target the specific variants |

## Open Questions

1. **Exact aggregate coverage after stat1 gap closure**
   - What we know: Current aggregate is 93.53% lines. `modal.rs` contributes 65 missed lines; `anova.rs` contributes 98 missed lines. Total stat1 missed: 311 lines / 3,157 total = ~9.8% of stat1 lines missed.
   - What's unclear: The aggregate denominator is 24,284 lines. Closing all stat1 gaps eliminates ~311 lines; the remaining miss budget for 95.39% is 24,284 × (1 - 0.9539) = ~1,119 missed lines allowed. Current miss is 1,571 lines. Even eliminating all 311 stat1 missed lines gets to 1,260 still missed (94.81% lines). The v3.0 baseline measurement may have excluded some code paths now covered by stat1.
   - Recommendation: Run `just coverage` FIRST in Wave 2; compare aggregate to v3.0 baseline coverage. If still short after stat1 gap closure, the planner must evaluate whether the pre-stat1 aggregate was already below 95.39% (making the gate unachievable without non-stat1 changes).

2. **ΣNORMD E2E modal dispatch key ID**
   - What we know: `stat1/modal.rs` handles `Stat1Step::NormdModePrompt`; mode selection maps to `[E]` (upper-tail CDF), `[C]` (PDF), `[A]` (inverse).
   - What's unclear: The `data-key-id` value for the `[E]` mode-selection key within an active ΣNORMD modal. In Math Pac I, modal key routing used the literal key character (e.g., `dispatch_op('e')` triggers the `[E]` function key). Stat 1 may use the same pattern.
   - Recommendation: Wave 4 reconnaissance sub-task reads `hp41-core/src/ops/stat1/modal.rs::submit_step` for `Stat1Step::NormdModePrompt` and `hp41-gui/src-tauri/src/key_map.rs` for the E/C/A key IDs.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` + `cargo-llvm-cov` | Coverage measurement (STAT-QUAL-01/02/03) | ✓ | MSRV 1.88 [VERIFIED: Cargo.toml] | — |
| `just` | Test orchestration (`just coverage`, `just ci`) | ✓ | latest [ASSUMED] | `cargo test` + `cargo llvm-cov` directly |
| `node` + WebdriverIO | E2E smoke (STAT-QUAL-11) | ✓ (ci-gui.yml Ubuntu) | Node 18+ [ASSUMED] | Ubuntu-only gate; local macOS may differ |
| `tauri-driver` | WebdriverIO + Tauri E2E | ✓ (in CI) | 2.0.6 [VERIFIED: wdio.conf.cjs comment] | — |
| `python3` + `scipy` | Fresh oracle derivation (D-37.1) | ✓ local [ASSUMED] | scipy 1.x | Manual derivation from published tables |
| `bash` | Free42 contamination re-verification | ✓ | — | — |

**Missing dependencies with no fallback:** None — all test infrastructure is already established.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `#[test]` + `cargo-llvm-cov` + WebdriverIO 9 + bash |
| Config file | `hp41-core/Cargo.toml` (dev-deps); `hp41-gui/wdio.conf.cjs` (E2E); `justfile` |
| Quick run command | `cargo test -p hp41-core --test stat1_op_test_count` |
| Full suite command | `just ci` (= lint + test + coverage + license-audit) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| STAT-QUAL-01 | hp41-core line coverage ≥ 95.39% | coverage gate | `just coverage` | ✅ (gate exists, value needs work) |
| STAT-QUAL-02 | hp41-core region coverage ≥ 94.26% | coverage gate | `just coverage` | ✅ (97.27% — already passing) |
| STAT-QUAL-03 | Per-file stat1/*.rs ≥ 90% | coverage gate | `cargo llvm-cov --html -p hp41-core` | ✅ (gate exists; modal.rs + anova.rs need gap tests) |
| STAT-QUAL-04 | numerical_accuracy.rs ≥ 98% with Stat 1 cases | suite + floor | `cargo test -p hp41-core --test numerical_accuracy` | ✅ (inline extension only) |
| STAT-QUAL-05 | Two-level tolerance discipline | lint + macro | `cargo test -p hp41-core --test lint_stat1_assertions` | ❌ Wave 1 |
| STAT-QUAL-06 | lint_stat1_assertions.rs blocks decimal assert_eq | lint test | `cargo test -p hp41-core --test lint_stat1_assertions` | ❌ Wave 1 |
| STAT-QUAL-07 | stat1_op_test_count.rs ≥ 5 tests per variant | meta-test | `cargo test -p hp41-core --test stat1_op_test_count` | ❌ Wave 1 |
| STAT-QUAL-08 | xrom_shadowing.rs STAT_1 gates pass | meta-test | `cargo test -p hp41-core --test xrom_shadowing` | ✅ (already done in Phase 33) |
| STAT-QUAL-09 | Free42 contamination guard passes | CI script | `just license-audit` | ✅ (already done in Phase 33) |
| STAT-QUAL-10 | v3.0 save file migrates to xrom_modules=0b11 | integration test | `cargo test -p hp41-core --test stat1_backward_compat` | ❌ Wave 2 |
| STAT-QUAL-11 | E2E smoke Stat 1 ΣNORMD workflow | E2E smoke | `just gui-e2e` (Ubuntu ci-gui.yml) | ❌ Wave 4 |
| STAT-GUI-05 | Bounded-iter primitives need no cancellation | documentation | manual — VERIFICATION.md waiver | ❌ Wave 2 (doc only) |

### Sampling Rate
- **Per task commit:** `cargo test -p hp41-core --test <specific-file>` (sub-second)
- **Per wave merge:** `just ci` (full lint + test + coverage + license-audit gate)
- **Phase gate:** `just ci` + `just gui-ci` + `just gui-e2e` (Ubuntu) all green before final ship commit

### Wave 0 Gaps
- [ ] `hp41-core/tests/stat1_op_test_count.rs` — Wave 1, covers STAT-QUAL-07
- [ ] `hp41-core/tests/lint_stat1_assertions.rs` — Wave 1, covers STAT-QUAL-05/06
- [ ] `hp41-core/tests/stat1_backward_compat.rs` — Wave 2, covers STAT-QUAL-10
- [ ] `hp41-core/tests/fixtures/v30-autosave.json` — Wave 2, required by stat1_backward_compat
- [ ] `hp41-core/tests/stat1_modal_coverage.rs` — Wave 2, covers STAT-QUAL-03 (modal.rs gap)
- [ ] `hp41-core/tests/stat1_anova_coverage.rs` — Wave 2, covers STAT-QUAL-03 (anova.rs gap)
- [ ] `numerical_accuracy.rs` stat1 extension + `ITER_TOL` constant + `iter` macro arm — Wave 3, covers STAT-QUAL-04/05
- [ ] `smoke.spec.js` ΣNORMD `it()` block — Wave 4, covers STAT-QUAL-11

## Security Domain

Security enforcement is not applicable to this phase — Phase 37 adds test files only; no new IPC endpoints, no new user-facing inputs, no authentication or session management changes.

## Sources

### Primary (HIGH confidence)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/tests/math1_op_test_count.rs` — direct structural template; full implementation verified line-by-line
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/tests/lint_math1_assertions.rs` — direct structural template; WR-02 multi-line detect, LINT-EXEMPT convention verified
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/tests/xrom_shadowing.rs` — STAT_1 shadowing gates already present (Phase 33 Plan 33-01); STAT-QUAL-08 attested complete
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/tests/numerical_accuracy.rs` — 8348 LOC; `case!` macro, `AccuracyCase` struct, v1.x baseline floor logic verified
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/src/ops/math1/xrom.rs:141-186` — STAT_1.ops slice with all 26 variant names verified
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui/e2e/smoke.spec.js` — full content; `invokeBackend` fallback, `clickKey` helper, `beforeEach` CLX reset, `view.display_str` assertion pattern verified
- `/Users/daniel/GitRepository/hp41-calculator-emulator/scripts/check-free42-contamination.sh` — 18-token pattern + WR-01 directory guard; scans both math1/ and stat1/ verified
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/tests/stat1_rand_determinism.rs` — existing stat1 test; lint false-positive risk identified
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/tests/stat1_cancellation.rs` — existing stat1 test; pattern for external integration tests
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-core/tests/fixtures/v20-autosave.json` — fixture format; backward-compat test structural template
- `just coverage` live run — per-file coverage numbers for all 12 stat1/*.rs files verified
- `.planning/phases/37-test-hardening-quality-gates/37-CONTEXT.md` — all D-37.1..D-37.11 decisions
- `.planning/milestones/v3.0-phases/32-test-hardening/32-RESEARCH.md` — Phase 32 structural template; Per-Op audit methodology
- `.planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-SPEC-AMENDMENT.md` — 6 oracle drifts; D-37.1 independent derivation rationale

### Secondary (MEDIUM confidence)
- `.planning/REQUIREMENTS.md` — STAT-QUAL-01..11 + STAT-GUI-05 full requirement text verified
- `.planning/STATE.md` — v3.0 coverage baseline (95.39% / 94.26%) verified

## Metadata

**Confidence breakdown:**
- Coverage gap analysis: HIGH — measured via live `just coverage` run; per-file numbers verified
- Per-Op test count audit: MEDIUM — pre-computed from source file LOC and inline test counts; actual ≥ 5 threshold verification requires running `stat1_op_test_count.rs`
- E2E modal flow for ΣNORMD: MEDIUM — `invokeBackend` fallback strategy is verified; exact mode-selection key ID requires Wave 4 reconnaissance
- Numerical accuracy oracle: MEDIUM — scipy derivation guidance given; values must be freshly derived per D-37.1 (not pre-computed in this research)
- Structural templates (meta-gate files): HIGH — verified via direct code inspection

**Research date:** 2026-05-24
**Valid until:** 2026-06-07 (stable domain — test infrastructure changes slowly)
