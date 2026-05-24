# Phase 37: Test Hardening & Quality Gates - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-24
**Phase:** 37-test-hardening-quality-gates
**Areas discussed:** numerical accuracy scope, coverage gap closure, STAT-GUI-05 disposition, plan slicing

---

## Numerical Accuracy Scope

### Oracle sourcing approach

| Option | Description | Selected |
|--------|-------------|----------|
| Extract from Phase 33 tests | Mine existing stat1_*.rs test files for scipy-derived reference values and promote into numerical_accuracy.rs. Fastest path. | |
| Fresh scipy derivation | Derive a new set of ~30 reference values directly from scipy.stats, independent of Phase 33 tests. Fully independent validation layer. | ✓ |
| You decide | Claude picks the approach. | |

**User's choice:** Fresh scipy derivation
**Notes:** Independent derivation catches any Phase 33 oracle transcription errors. Mirrors v3.0 approach.

### File shape

| Option | Description | Selected |
|--------|-------------|----------|
| Inline extension | Append ~30 cases to existing 768-case array in numerical_accuracy.rs. Single file, unified pass-rate gate. | |
| Separate file | New stat1_numerical_accuracy.rs with own case array. Isolated domains. | |
| You decide | Claude picks based on maintainability. | ✓ |

**User's choice:** You decide
**Notes:** Claude discretion D-37.3. Recommendation: inline extension.

### Algorithm family allocation

| Option | Description | Selected |
|--------|-------------|----------|
| Distribution-heavy | Focus on ΣNORMD, ΣCHISQD, ΣTSTAT. ~15 distribution + ~15 other. | |
| Uniform spread | ~4-5 cases per family. Even coverage. | |
| You decide | Claude allocates by algorithmic risk. | ✓ |

**User's choice:** You decide
**Notes:** Claude discretion D-37.4. Recommendation: risk-weighted allocation.

### Tolerance approach

| Option | Description | Selected |
|--------|-------------|----------|
| Per-case tolerance | Each case carries its own 1e-9 or 1e-7. Mirrors existing WIDE_TOL pattern. | |
| Family-rigid | Tolerance determined solely by family membership. Simpler. | |
| You decide | Claude picks based on existing infrastructure. | ✓ |

**User's choice:** You decide
**Notes:** Claude discretion D-37.5. Recommendation: per-case tolerance.

---

## Coverage Gap Closure

### Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Per-module test files | Create stat1_anova.rs, stat1_regression.rs, etc. Best isolation. | |
| Measure first, patch targeted | Run cargo-llvm-cov per-file first, write tests only for gap files. | |
| You decide | Claude picks based on inline test density. | ✓ |

**User's choice:** You decide
**Notes:** Claude discretion D-37.6. Recommendation: measure first, targeted gap closure.

### Meta-gate scan scope

| Option | Description | Selected |
|--------|-------------|----------|
| Both inline + external | Scan stat1/*.rs source + tests/stat1_*.rs. Reflects actual density (189 inline tests). | |
| External only | Mirror math1 exactly — scan only tests/stat1_*.rs. Stricter. | |
| You decide | Claude picks. | ✓ |

**User's choice:** You decide
**Notes:** Claude discretion D-37.7. Recommendation: both inline + external.

### Lint file approach

| Option | Description | Selected |
|--------|-------------|----------|
| New file | Separate lint_stat1_assertions.rs. Clean sibling separation. | |
| Extend existing | Widen lint_math1_assertions.rs to scan both. | |
| You decide | Claude picks. | ✓ |

**User's choice:** You decide
**Notes:** Claude discretion D-37.8. Recommendation: new file.

---

## STAT-GUI-05 Disposition

| Option | Description | Selected |
|--------|-------------|----------|
| Documented N/A waiver | Mark N/A in VERIFICATION.md + add divergence-catalog entry. No code changes. | |
| Lightweight sanity test | Test proving primitives complete within wall-time bound. Still no cancellation wiring. | |
| Implement anyway | Wire cancel_requested checks for pattern symmetry. Highest code churn, zero user benefit. | |
| You decide | Claude picks. | ✓ |

**User's choice:** You decide
**Notes:** Claude discretion D-37.9. Recommendation: documented N/A waiver + divergence entry.

---

## Plan Slicing

### Plan count

| Option | Description | Selected |
|--------|-------------|----------|
| 6 plans (compact) | Fewer plans, larger blast-radius. | |
| 8 plans (balanced) | Finer slicing, middle ground. | |
| You decide | Claude picks. | ✓ |

**User's choice:** You decide
**Notes:** Claude discretion D-37.10. Recommendation: 7-8 plans.

### E2E platform scope

| Option | Description | Selected |
|--------|-------------|----------|
| Ubuntu-only | Extend existing ci-gui.yml::e2e-linux. Matches v3.0 shape. | ✓ |
| Full matrix | All 3 CI platforms. More confidence, more complexity. | |
| You decide | Claude picks. | |

**User's choice:** Ubuntu-only
**Notes:** Matches v3.0 D-27.13 scope. ROI of cross-platform E2E doesn't justify CI infrastructure cost.

### README graduation timing

| Option | Description | Selected |
|--------|-------------|----------|
| Include in Phase 37 | Final plan graduates soft-claim to hard-claim. Self-contained. | |
| Defer to milestone close | /gsd-complete-milestone handles it. Cleaner separation. | |
| You decide | Claude picks. | ✓ |

**User's choice:** You decide
**Notes:** Claude discretion D-37.11. Recommendation: include in Phase 37 (mirrors v3.0 Phase 32).

---

## Claude's Discretion

User deferred to Claude on 9 decisions (D-37.3 through D-37.11):
- Accuracy file shape (inline vs separate)
- Accuracy case allocation by family
- Tolerance approach (per-case vs family-rigid)
- Coverage gap closure strategy
- Meta-gate scan scope (inline+external vs external-only)
- Lint file approach (new vs extend)
- STAT-GUI-05 resolution approach
- Plan count
- README graduation timing

## Deferred Ideas

None — discussion stayed within phase scope.
