# Phase 42: Test Hardening & Quality Gates - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-25
**Phase:** 42-test-hardening-quality-gates
**Areas discussed:** Meta-gate strategy, Date accuracy oracle, Stopwatch timing test, Plan wave structure

---

## Meta-gate Strategy

### Meta-gate files: unify or keep per-module?

| Option | Description | Selected |
|--------|-------------|----------|
| Unify now | Create xrom_op_test_count.rs + lint_xrom_assertions.rs scanning all 3 modules. Delete the math1/stat1 siblings. Advantage Pac benefits for free. Fulfills Phase 37 deferred commitment. | ✓ |
| Separate (time_*) | Create time_op_test_count.rs + lint_time_assertions.rs mirroring the existing pattern. Simpler diff, consistent with existing convention. Defer unification further. | |
| You decide | Claude picks based on blast-radius and maintenance cost analysis. | |

**User's choice:** Unify now
**Notes:** Phase 37 explicitly deferred this to "the third XROM module" — Phase 42 is that moment.

### Scan scope for unified xrom_op_test_count.rs

| Option | Description | Selected |
|--------|-------------|----------|
| Dual-scan all modules | Scan both inline tests and external test files for all 3 modules. Most accurate count. Time has 209 inline tests so this matters. | |
| External tests/ only | Only scan tests/*.rs files. Simpler, but undercounts (especially Time with 209 inline tests vs. 0 external time_* test files currently). | |
| You decide | Claude picks based on actual test distribution per module. | ✓ |

**User's choice:** You decide
**Notes:** Deferred to Claude's discretion (D-42.10).

### Migration approach (delete old files)

| Option | Description | Selected |
|--------|-------------|----------|
| Delete in same plan | Single atomic plan: create unified files, delete old ones. Clean diff, one commit. | |
| Keep old, delete later | Create unified files first. Verify green CI. Delete old files in a follow-up plan. | |
| You decide | Claude picks based on risk assessment. | ✓ |

**User's choice:** You decide
**Notes:** Deferred to Claude's discretion (D-42.14).

---

## Date Accuracy Oracle

### File location for date accuracy cases

| Option | Description | Selected |
|--------|-------------|----------|
| Extend numerical_accuracy.rs | Append date cases to the existing 791-case suite. Growing denominator. Unified pass-rate gate. | |
| Separate time_date_accuracy.rs | New file for date/calendar edge cases. Exact-match assertions. Keeps the numerical suite focused on real-valued math. | |
| You decide | Claude picks based on the nature of the test cases. | ✓ |

**User's choice:** You decide
**Notes:** Deferred to Claude's discretion (D-42.10).

### Oracle source

| Option | Description | Selected |
|--------|-------------|----------|
| Python datetime + calendar | Python's datetime.date and calendar modules as oracle. Well-tested, handles leap years, century rules. | |
| Published tables (Meeus) | Jean Meeus 'Astronomical Algorithms' JDN tables + USNO leap year rules. Academic authority. | |
| Both sources cross-checked | Derive from Python datetime, cross-check against published tables for edge cases. | |
| You decide | Claude picks the most practical oracle source. | ✓ |

**User's choice:** You decide
**Notes:** Deferred to Claude's discretion (D-42.11).

### Edge case allocation

| Option | Description | Selected |
|--------|-------------|----------|
| All listed + extras | All TIME-QUAL-02 edges plus Y2K boundary, year 9999 limit, 0/negative offsets, historical dates. ~30 cases. | |
| TIME-QUAL-02 minimum | Just the edges explicitly listed in the requirement. ~15-20 cases. | |
| You decide | Claude allocates based on algorithmic risk. | ✓ |

**User's choice:** You decide
**Notes:** Deferred to Claude's discretion (D-42.12).

---

## Stopwatch Timing Test

### CI test approach for TIME-QUAL-10

| Option | Description | Selected |
|--------|-------------|----------|
| Short window in CI | Test with a 1-2 second window and ±5ms tolerance. Document that 60s ±10ms follows from Instant monotonicity. | |
| #[ignore] for manual | Write the full 60s test but mark it #[ignore]. CI runs only the short-window variant. | |
| Mathematical argument only | No wall-clock test. Document that Instant is monotonic by OS contract. | |
| You decide | Claude picks the most practical approach. | ✓ |

**User's choice:** You decide
**Notes:** Deferred to Claude's discretion (D-42.13).

---

## Plan Wave Structure

### Plan count

| Option | Description | Selected |
|--------|-------------|----------|
| 5-6 plans (lean) | W1: Unified meta-gates. W2: Coverage gap closure. W3: Date accuracy + stopwatch + alarm. W4: Backward-compat + E2E + README graduation. | ✓ |
| 7-8 plans (Phase 37 parity) | Same granularity as Phase 37. More atomic commits, finer review checkpoints. | |
| You decide | Claude slices based on blast-radius and dependency analysis. | |

**User's choice:** 5-6 plans (lean)
**Notes:** Phase 42 has less greenfield work than Phase 37 since Phase 38/39 already extended several gates (Free42, XROM shadowing, function_matrix_parity).

### E2E smoke workflow

| Option | Description | Selected |
|--------|-------------|----------|
| DDAYS (day count between dates) | XEQ "DDAYS" with two dates → verify day count on LCD. Exercises date parsing + JDN arithmetic. | |
| DATE+ (add days to date) | XEQ "DATE+" with a date and offset → verify result date on LCD. | |
| You decide | Claude picks based on which exercises the most code paths with the simplest key sequence. | ✓ |

**User's choice:** You decide
**Notes:** Deferred to Claude's discretion (D-42.15).

---

## Claude's Discretion

The following areas were deferred to Claude's judgment:
- D-42.10: Scan scope for unified meta-gates (inline + external vs external only)
- D-42.11: Date accuracy oracle source (Python datetime, Meeus tables, or both)
- D-42.12: Date edge case allocation across DATE+/DDAYS/DOW
- D-42.13: Stopwatch timing test approach (short-window CI, #[ignore] 60s, or math argument)
- D-42.14: Meta-gate migration (delete old files in same plan or separately)
- D-42.15: E2E smoke Time Pac workflow (DDAYS vs DATE+)

## Deferred Ideas

- CI-automated coverage gate (ergonomic improvement, not required for v3.2 ship)
- Cross-platform E2E smoke (Ubuntu-only per D-37.2 precedent)
- Signed binary releases (deferred to post-v3.2)
