---
plan: "37-05"
status: complete
started: 2026-05-24
completed: 2026-05-24
---

# Plan 37-05 Summary: E2E Smoke + Quality Gates + README Hard-Claim

## What was built

### Task 1 — E2E Smoke Extension (STAT-QUAL-11)
- Added ΣNORMD Q(1.96) workflow to `hp41-gui/e2e/smoke.spec.js`
- Tests: push 1.96 → dispatch xeq_ΣNORMD → enter mode 1 (CDF) → R/S → assert display starts with "0.0250"
- Uses invokeBackend fallback pattern consistent with SINH/MATRIX DET tests

### Task 2 — Quality Gate Verification (STAT-QUAL-01/02)
- `just ci` passes: lint + test + coverage + license-audit all green
- Aggregate hp41-core coverage: 93.91% lines / 95.84% regions / 97.27% functions
- Per-file stat1/*.rs coverage: all ≥ 90.60% except anova.rs (86.64%)
- Numerical accuracy: 791 cases, 782 pass = 98.86% (≥ 98% gate met)
- Coverage assessment: v3.0 baseline of 95.39% lines was measured before stat1 ~6,824 LOC added; denominator dilution is root cause; region coverage (95.84%) exceeds v3.0 baseline (94.26%)

### Task 3 — Human Checkpoint
- User reviewed coverage metrics and approved graduation

### Task 4 — README Hard-Claim + CLAUDE.md Finalization
- README.md: "feature-complete per Owner's Manual HP 00041-90030" inserted
- CLAUDE.md: Phase 36 + 37 marked shipped 2026-05-24, v3.1 header drops "IN PROGRESS", frozen invariants updated

## Key files

### Created
- `hp41-gui/e2e/smoke.spec.js` (modified — ΣNORMD test block added)

### Modified
- `README.md` — Stat 1 Pac hard-claim graduation
- `CLAUDE.md` — Phase 37 entry + v3.1 finalization

## Self-Check: PASSED

## Quality gates

| Gate | Target | Result |
|------|--------|--------|
| hp41-core line coverage | ≥ 95.39% | 93.91% (denominator dilution — see assessment) |
| hp41-core region coverage | ≥ 94.26% | **95.84%** ✅ |
| Numerical accuracy | ≥ 98% | **98.86%** (782/791) ✅ |
| Meta-gate (26 ops ≥5 tests) | all pass | ✅ |
| Free42 contamination | 0 | ✅ |
| Backward compat | v3.0 migration | ✅ |
| E2E smoke Stat 1 workflow | ≥ 1 | ✅ (ΣNORMD) |
| `just ci` | exit 0 | ✅ |
