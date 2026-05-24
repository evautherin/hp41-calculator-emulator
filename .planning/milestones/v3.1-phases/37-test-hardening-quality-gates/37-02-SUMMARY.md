---
phase: 37-test-hardening-quality-gates
plan: 02
subsystem: testing
tags: [rust, llvm-cov, stat1, modal, coverage, hp41-core]

# Dependency graph
requires:
  - phase: 37-01
    provides: stat1_op_test_count.rs meta-gate and lint_stat1_assertions.rs Pitfall 14/17 discipline

provides:
  - hp41-core/tests/stat1_modal_coverage.rs — 28 targeted tests covering all 5 Stat1Step variants
  - stat1/modal.rs line coverage 74.21% → 94.48% (STAT-QUAL-03 per-file floor met)
  - stat1/modal.rs region coverage 79.01% → 92.46%

affects:
  - Phase 37 Plans 37-03..37-05 (additional coverage and quality gates)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "stack-setup pattern for two-step modal tests: directly assign state.stack.y/x before NuPrompt to simulate HP-41 ENTER-then-value flow"
    - "mode-push pattern for ChisqdModeChoice: after NuPrompt drop, simulate push by assigning state.stack.y = state.stack.x.clone() then state.stack.x = mode"
    - "LINT-EXEMPT annotation placement: preceding comment block (not inline) for assert_eq! with HpNum lookahead false positives"

key-files:
  created:
    - hp41-core/tests/stat1_modal_coverage.rs
  modified: []

key-decisions:
  - "Inline Op dispatch calls (Op::SigmaNormdWorkflow etc.) in test function bodies instead of helper functions — enables stat1_op_test_count Pitfall 16 word-boundary matching per test slice"
  - "Stack-register direct assignment for test setup (not dispatch(PushNum)) — avoids lift_enabled false-not-yet-set trap where consecutive PushNum calls without ENTER both overwrite X"
  - "mode-push via state.stack.y = state.stack.x.clone() + state.stack.x = mode — simulates the HP-41 hardware two-value stack shape required by ChisqdModeChoice without triggering lift mechanics"
  - "Pre-existing stat1_op_test_count failure (13 Ops below threshold) is out-of-scope for Plan 37-02 — plan targets modal.rs coverage, not Op count gate (those Ops are in anova/regression/hypothesis families)"

patterns-established:
  - "stat1_modal_coverage.rs: LINT-EXEMPT comment preceding assert_eq! when lookahead 4 lines contain HpNum or Decimal tokens (even in comments)"

requirements-completed:
  - STAT-QUAL-03

# Metrics
duration: 45min
completed: 2026-05-24
---

# Phase 37 Plan 02: stat1/modal.rs Coverage Gap Closure Summary

**28 targeted integration tests close the stat1/modal.rs coverage gap from 74.21% → 94.48% lines, exercising all 5 Stat1Step submit_step branches including the ΣCHISQD two-step chain, cancel paths, and error paths**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-05-24T09:50:00Z
- **Completed:** 2026-05-24T10:28:00Z
- **Tasks:** 2 (Tasks 1 + 2 combined into single commit — Task 2 is verification only)
- **Files modified:** 1

## Accomplishments

- Installed `hp41-core/tests/stat1_modal_coverage.rs` with 28 targeted tests (>= 15 required)
- `stat1/modal.rs` line coverage: 74.21% → 94.48% (target: >= 90% — STAT-QUAL-03 met)
- `stat1/modal.rs` region coverage: 79.01% → 92.46% (target: >= 90% — met)
- All 5 Stat1Step variants covered: NormdModeChoice, ChisqdNuPrompt, ChisqdModeChoice, PolypDegreePrompt, SeedPrompt
- ΣCHISQD two-step chain (NuPrompt → ModeChoice for both PDF mode 1 and CDF mode 2) fully exercised
- stat1_op_test_count Pitfall 16 gate improved: SigmaNormdWorkflow (4→5+), SigmaChisqdWorkflow (2→5+), SigmaPolypWorkflow (2→5+) now pass — 3 Ops moved from failing to passing
- Overall hp41-core line coverage: 95.39% → 95.67% (gate holds at >= 95%)
- No regressions in existing tests (only pre-existing stat1_op_test_count failures remain)

## Task Commits

Each task was committed atomically:

1. **Task 1+2: Install stat1_modal_coverage.rs and verify coverage** - `0cd873a` (feat)

**Plan metadata:** (see final commit below)

## Files Created/Modified

- `hp41-core/tests/stat1_modal_coverage.rs` — 28 coverage-gap closure tests for stat1/modal.rs; covers all 5 Stat1Step submit_step branches, cancel paths, error paths, current_prompt, requires_alpha_label on Plan-33-08 variants; LINT-EXEMPT annotations per Pitfall 14/17 discipline; Free42 disclaim header per project convention

## Decisions Made

1. **Inline Op dispatch in test bodies** — replaced `open_normd/chisqd/polyp` helper calls with `dispatch(&mut state, Op::SigmaNormdWorkflow)` etc. inline in each test function body. Rationale: `stat1_op_test_count` splits by `#[test]` marker and checks each test slice independently; helper functions before any `#[test]` are not visible in any test slice.

2. **Direct stack register assignment for two-step modal tests** — used `state.stack.y = HpNum::from(...)` and `state.stack.x = HpNum::from(...)` directly instead of `dispatch(PushNum(...))`. Rationale: `lift_enabled` starts false in `CalcState::new()`; consecutive `PushNum` without ENTER both overwrite X, losing the χ² statistic. Direct assignment correctly places χ² in Y and ν in X before NuPrompt.

3. **Mode-push pattern for ChisqdModeChoice** — simulated "user enters mode index after NuPrompt" via `state.stack.y = state.stack.x.clone(); state.stack.x = mode`. Rationale: after NuPrompt drop (X←Y), the χ² stat is in X; the ModeChoice arm reads mode from X then drops X←Y to expose the stat to the eval function. The test must simulate the HP-41 push mechanics without going through dispatch.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Two-step ΣCHISQD chain tests initially used wrong stack setup**
- **Found during:** Task 1 (test file creation + verification run)
- **Issue:** Initial implementation used `state.stack.y = χ²_stat` before `open_chisqd` + `set_x_decimal(mode)` to set mode after NuPrompt. This failed because: (a) `dispatch(PushNum)` with lift_enabled=false just overwrites X without lifting; (b) `set_x_decimal` only sets X, so the χ² stat (which ends up in X after NuPrompt drop) was overwritten by the mode value, leaving X=mode and Y=0; the PDF/CDF call then got x=0 and returned Domain.
- **Fix:** Used direct stack register assignment (`state.stack.y = state.stack.x.clone(); state.stack.x = mode`) to simulate the HP-41 push semantics for mode entry after NuPrompt.
- **Files modified:** hp41-core/tests/stat1_modal_coverage.rs
- **Verification:** `cargo test -p hp41-core --test stat1_modal_coverage` — all 28 pass
- **Committed in:** 0cd873a (task commit)

**2. [Rule 1 - Bug] Lint gate `lint_stat1_assertions` flagged two assert_eq! calls via lookahead false positives**
- **Found during:** Task 1 (full hp41-core test run after initial commit attempt)
- **Issue:** `lint_stat1_assertions::no_decimal_assert_eq_in_stat1_tests` uses a 4-line lookahead from each `assert_eq!` line. Two occurrences had lookahead windows containing "HpNum" text (in comments) or `rand_seed.inner()` calls, triggering Pitfall 17 detection for `assert_eq!(r, Ok(()))` lines which are not HpNum comparisons.
- **Fix:** Added `// LINT-EXEMPT: Result<(),HpError> equality — not a numerical HpNum comparison; ...` preceding comment blocks for the two affected assert_eq! calls.
- **Files modified:** hp41-core/tests/stat1_modal_coverage.rs
- **Verification:** `cargo test -p hp41-core --test lint_stat1_assertions` — 2 passed
- **Committed in:** 0cd873a (task commit, same file)

---

**Total deviations:** 2 auto-fixed (both Rule 1 — implementation bugs in initial test setup caught during verification)
**Impact on plan:** Both fixes necessary for test correctness. No scope creep.

## Issues Encountered

**Pre-existing stat1_op_test_count failure:** The `each_stat1_op_has_at_least_5_tests` gate was already failing before Plan 37-02 started (confirmed by temporarily hiding the new test file and re-running). 13 Op variants (SigmaMmtug, SigmaMmtgd, SigmaAovone, SigmaAovtwo, SigmaAnocov, SigmaLin, SigmaExp, SigmaLogi, SigmaPow, SigmaMlrxy, SigmaMlrxyz, SigmaPolyc, SigmaCtkk) remain below 5 mentions. Plan 37-02's scope is modal.rs coverage; the Op count gate for these families is addressed by Plans 37-03+. Plan 37-02 does improve 3 Ops (NormdWorkflow, ChisqdWorkflow, PolypWorkflow) over the threshold.

## Known Stubs

None — no stub patterns in the test file.

## Threat Flags

None — test-only file, no new trust boundaries.

## Next Phase Readiness

- `stat1/modal.rs` coverage gap closed at 94.48% lines / 92.46% regions (>= 90% STAT-QUAL-03 floor met)
- Plan 37-03 (accuracy oracle hardening) and 37-04+ (remaining Op count gate closure) can proceed
- The 13 remaining Ops in the stat1_op_test_count failure list are in anova/regression/hypothesis/moments families — these need dedicated test files in Plans 37-03..37-05

---
*Phase: 37-test-hardening-quality-gates*
*Completed: 2026-05-24*
