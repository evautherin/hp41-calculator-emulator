---
phase: 34-hp41-cli-cli-integration
verified: 2026-05-23T12:00:00Z
status: passed
score: 4/4 must-haves verified
overrides_applied: 0
re_verification: null
gaps: []
deferred: []
human_verification: []
---

# Phase 34: hp41-cli — CLI Integration Verification Report

**Phase Goal:** Users can discover and invoke all Stat 1 Pac programs from the CLI — `XEQ "ΣSPEAR"` works from the keyboard, the `?` overlay shows a "Stat 1 Pac (XROM 2)" section, and program listings display all ~24 new Op variant names
**Verified:** 2026-05-23T12:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | SC-1: `XEQ "ΣNORMD"` invokes ΣNORMD via `xeq_by_name_local_resolve` → `xrom_resolve` fallthrough with `xrom_modules = 0b0000_0011` | VERIFIED | `phase25_xeq_by_name.rs::cli_resolver_matches_core_resolver` passes 1/1 with 5 Stat 1 cases at 3 bitfield states; keys.rs line 397 `xrom_resolve` fallthrough untouched |
| 2 | SC-2: `?` overlay shows Stat 1 Pac entries in named sections; search across all three JSON pools returns Stat 1 results | VERIFIED | `help_entries_all()` chains three pools in fixed order (built-ins → Math 1 → Stat 1); `help_overlay_rows()` generates `=== Stat1 <Family> ===` headers (7 families, parallel to `=== Math1 <Family> ===` Phase 29 pattern); `phase34_help_data_stat1.rs` tests 12/12 pass; `function_matrix_parity.rs` 11/11 pass |
| 3 | SC-3: `XEQ "ΣSPEAR"` (and all 26 Stat 1 ops) display correctly in program listing — `op_display_name` arms exhaustive, no `_ =>` catch-all | VERIFIED | `cargo check -p hp41-cli` exits 0; `cargo build -p hp41-cli --tests --no-run 2>&1 | grep -c "non-exhaustive patterns"` = 0; 26 arms present in `prgm_display.rs` (lines 296–327); comment-only match for `_ =>` at line 26 confirms no catch-all arm |
| 4 | SC-4: Multi-step Stat 1 workflows (ΣPOLYP degree prompt, SEED prompt) surface via existing `modal_program` infrastructure with no new transient `CalcState` fields | VERIFIED | `phase34_modal_flow.rs` 5/5 pass — all 5 `Stat1Step` variants produce OM-cited prompts via `ModalProgram::Stat1(_).current_prompt()`; `requires_alpha_label()` = false for all 5; no new `CalcState` fields in `f31e53c..HEAD` diff of `hp41-core/src/*` (zero modifications to hp41-core) |

**Score:** 4/4 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `docs/hp41-stat1-functions.json` | 26 entries, `xrom.module_id=2`, dense function_ids 1..=26, 7 D-34.1 categories, 4 surgical divergences (D-34.3) | VERIFIED | Python3 verify: "OK 26 entries, dense function_ids, 4 divergent entries"; all 26 `module_id=2`; categories: `Stat1 Univariate`(4), `Stat1 ANOVA`(3), `Stat1 Regression`(8), `Stat1 Hypothesis`(2), `Stat1 Nonparam`(5), `Stat1 Distributions`(2), `Stat1 RNG`(2); 4 divergences for Rand/Seed/SigmaTstat/SigmaPolypWorkflow |
| `hp41-cli/src/help_data.rs` | Third `OnceLock`, `help_entries_stat1()`, `help_entries_all()` 3-pool chain, D-25.17 panic message | VERIFIED | `static STAT1_HELP_ENTRIES` at line 129; `const STAT1_FUNCTIONS_JSON` at line 127 via `include_str!`; `pub fn help_entries_stat1()` at line 142 with `.expect("hp41-stat1-functions.json is malformed — fix the JSON")`; three-pool chain `.chain(help_entries_stat1().iter())` at line 168 |
| `hp41-cli/src/prgm_display.rs` | 26 new arms, no `_ =>` catch-all, FN-CLI-04 invariant comment | VERIFIED | All 26 arms confirmed at lines 296–327; grep for actual catch-all `_ =>` arm: 0 (the only match is inside a comment at line 26); `grep -c "FN-CLI-04" = 1` |
| `hp41-cli/tests/phase34_help_data_stat1.rs` | 12 smoke tests (count=26, xrom.module_id=2, categories, divergences, cross-pool collision, 3-pool chain) | VERIFIED | `cargo test -p hp41-cli --test phase34_help_data_stat1`: 12 passed; 0 failed |
| `hp41-cli/tests/phase34_modal_flow.rs` | 5 unit tests — one per `Stat1Step` variant, OM prompt + `requires_alpha_label=false` | VERIFIED | `cargo test -p hp41-cli --test phase34_modal_flow`: 5 passed; 0 failed |
| `hp41-cli/tests/phase34_key_ref_includes_stat1.rs` | 3 regression guards — ΣNORMD excluded, RAND excluded, v2.2 Add preserved | VERIFIED | `cargo test -p hp41-cli --test phase34_key_ref_includes_stat1`: 3 passed; 0 failed |
| `hp41-cli/tests/function_matrix_parity.rs` | 3-pool partition (11 total tests: 4 v2.2 + 3 Math 1 + 4 Stat 1), `test_pool_partition_is_exhaustive` rogue-id guard | VERIFIED | `cargo test -p hp41-cli --test function_matrix_parity`: 11 passed; 0 failed |
| `hp41-cli/tests/phase25_xeq_by_name.rs` | `cli_resolver_matches_core_resolver` extended with 5 Stat 1 cases × 3 bitfield states | VERIFIED | `cargo test -p hp41-cli --test phase25_xeq_by_name -- cli_resolver_matches_core_resolver`: 1 passed; 0 failed |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `help_data.rs::STAT1_FUNCTIONS_JSON` | `docs/hp41-stat1-functions.json` | `include_str!("../../docs/hp41-stat1-functions.json")` at line 127 | WIRED | Compile-time embed confirmed |
| `help_data.rs::help_entries_stat1` | `STAT1_HELP_ENTRIES` | `OnceLock::get_or_init` with D-25.17 panic message | WIRED | Line 142–147 confirmed |
| `help_data.rs::help_entries_all` | `help_entries_stat1()` | `.chain(help_entries_stat1().iter())` at line 168 | WIRED | Third pool in fixed insertion order per D-34.6 |
| `prgm_display.rs::op_display_name` | 26 Stat 1 `Op` variants | Exhaustive match arms at lines 296–327 | WIRED | All 26 arms confirmed; no `_ =>` catch-all |
| `phase34_modal_flow.rs` | `hp41-core::ops::stat1::modal::current_prompt` | Via `ModalProgram::Stat1(_).current_prompt()` carrier-enum dispatch | WIRED | 5 tests verify dispatch works through the carrier enum |
| `keys.rs` line 397 | `xrom_resolve(name, xrom_modules)` | `_ => hp41_core::ops::math1::xrom::xrom_resolve(name, xrom_modules)` fallthrough | WIRED | Unchanged from Phase 29; verified by `cli_resolver_matches_core_resolver` Stat 1 extension |

---

### Data-Flow Trace (Level 4)

Not applicable — Phase 34 delivers no dynamic-data rendering component. All artifacts are static JSON (compile-time embedded), exhaustive-match arms (display-name mapping), and test files.

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| 26 op_display_name arms close the CI break | `cargo check -p hp41-cli` | exit 0 | PASS |
| No non-exhaustive patterns errors | `cargo build -p hp41-cli --tests --no-run 2>&1 \| grep -c "non-exhaustive patterns"` | 0 | PASS |
| No catch-all in op_display_name | `grep -n "_ =>" hp41-cli/src/prgm_display.rs` (inspect context) | Comment-only, line 26 | PASS |
| JSON structural invariants | `python3 -c "..."` (26 entries, dense function_ids, 4 divergences) | "OK 26 entries..." | PASS |
| All hp41-cli tests pass | `cargo test -p hp41-cli --tests` | 345 passed (16 suites) | PASS |
| just ci exits 0 | `just lint && just test && just coverage && just license-audit` | COMBINED_EXIT=0 | PASS |
| Coverage gate ≥ 95% lines | `just coverage` (TOTAL row) | 95.42% lines / 97.27% regions | PASS |
| Free42 contamination | `bash scripts/check-free42-contamination.sh` | "OK: no Free42 contamination detected" | PASS |
| GUI intentional CI break (Phase 36) | `cargo build --manifest-path hp41-gui/src-tauri/Cargo.toml 2>&1 \| grep -c "non-exhaustive patterns"` | 1 | PASS (expected) |
| hp41-core source untouched | `git diff f31e53c..HEAD --name-only -- "hp41-core/src/*"` | (empty) | PASS |
| hp41-gui source untouched | `git diff f31e53c..HEAD --name-only -- "hp41-gui/*"` | (empty) | PASS |

---

### Probe Execution

Not applicable — no `scripts/*/tests/probe-*.sh` files exist for Phase 34. CLI behavior verified via `cargo test` and `just ci`.

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| STAT-CLI-01 | 34-02-PLAN | `xeq_by_name_local_resolve` → `xrom_resolve` fallthrough routes Stat 1 names | SATISFIED | `cli_resolver_matches_core_resolver` passes with 5 Stat 1 cases × 3 bitfield states; `keys.rs` fallthrough at line 397 unchanged from Phase 29 |
| STAT-CLI-02 | 34-01-PLAN | `docs/hp41-stat1-functions.json` loaded via `include_str!` into third `OnceLock`; `help_entries_all()` chains 3 pools | SATISFIED | 12/12 smoke tests pass; `STAT1_HELP_ENTRIES` OnceLock wired; three-pool chain at line 168 |
| STAT-CLI-03 | 34-02-PLAN | 26 `op_display_name` arms in `prgm_display.rs`; exhaustive match; no `_ =>` catch-all | SATISFIED | `cargo check -p hp41-cli` exits 0; 26 arms at lines 296–327; no catch-all arm |
| STAT-CLI-04 | 34-02-PLAN | `?` overlay surfaces Stat 1 Pac sections; Stat 1 entries excluded from right-panel | SATISFIED | 7 `Stat1 *` category sections in overlay via `help_overlay_rows()`; `phase34_key_ref_includes_stat1.rs` 3/3 pass; `function_matrix_parity.rs` 11/11 pass |
| STAT-CLI-05 | 34-02-PLAN | Modal-prompt routing for Stat 1 multi-step workflows reuses existing infrastructure | SATISFIED | `phase34_modal_flow.rs` 5/5 pass; all 5 `Stat1Step` variants produce correct OM prompts; `requires_alpha_label()` = false for all; no new `CalcState` fields |

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | — | — | — | — |

No `TBD`, `FIXME`, or `XXX` markers found in Phase 34 deliverables. No placeholder implementations. No catch-all wildcard arms. No empty handlers.

**Note on `keys.rs` modification:** The Phase 34 diff includes a small addition to `hp41-cli/src/keys.rs` — adding `"PI" => Some(Op::Pi)` as a local resolver arm (Rule 1 bug fix: `XEQ "PI"` was previously falling through to `xrom_resolve` which returned no match). This is a beneficial correctness fix, not a scope violation. STAT-CLI-01 is unaffected; the `xrom_resolve` fallthrough remains as the last arm.

---

### Human Verification Required

(none)

All success criteria, requirement IDs, compilation gates, test suites, and CI pipeline verified programmatically. No human-visible UI behavior changes were introduced (Phase 34 is `hp41-cli` only; all changes are wiring/data/test). The one human-observable outcome (the `?` overlay showing Stat 1 sections) is covered programmatically via `help_overlay_rows()` being driven by `help_entries_all()` which now chains all three pools.

---

### Gaps Summary

No gaps. All four ROADMAP success criteria verified against concrete codebase evidence.

**Closed gates:**
- `cargo check -p hp41-cli` exits 0 — 4-way exhaustive-match invariant item 3 sealed
- `cargo build -p hp41-cli --tests --no-run 2>&1 | grep -c "non-exhaustive patterns"` = 0
- `grep -c "_ =>"` in `op_display_name` context = 0 (match is in comment at line 26, not a catch-all arm)
- All 26 Stat 1 Op variant arms present in `prgm_display.rs`
- `just ci` exits 0 (lint + test + coverage 95.42% + license-audit)
- `cargo test -p hp41-cli --tests` = 345 passed (16 suites)
- `phase34_help_data_stat1.rs` = 12/12 pass
- `phase34_modal_flow.rs` = 5/5 pass
- `phase34_key_ref_includes_stat1.rs` = 3/3 pass
- `function_matrix_parity.rs` = 11/11 pass (4 v2.2 + 3 Math 1 + 4 new Stat 1)
- `phase25_xeq_by_name.rs::cli_resolver_matches_core_resolver` = 1/1 pass
- `hp41-core/src/*` = 0 file modifications (out-of-scope invariant preserved)
- `hp41-gui/*` = 0 file modifications (Phase 36 scope, GUI CI break intentionally open at 1 error)

---

_Verified: 2026-05-23T12:00:00Z_
_Verifier: Claude (gsd-verifier)_
