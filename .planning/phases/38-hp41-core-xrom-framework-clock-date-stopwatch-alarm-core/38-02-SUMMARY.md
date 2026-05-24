---
phase: 38-hp41-core-xrom-framework-clock-date-stopwatch-alarm-core
plan: "02"
subsystem: hp41-core
tags: [date-arithmetic, jdn, time-module, rust, wave-2]
dependency_graph:
  requires: ["38-01"]
  provides: ["date_arith.rs full implementation", "parse_date_hpnum", "parse_time_hpnum", "secs_to_hpnum_time", "op_date_plus", "op_ddays", "op_dow", "op_dmy", "op_mdy"]
  affects: ["hp41-core/src/ops/time/date_arith.rs"]
tech_stack:
  added: []
  patterns: ["Fliegel-Van Flandern (1968) JDN algorithm", "string-split-at-decimal date parsing (D-carried.2/P35)", "TDD RED/GREEN cycle"]
key_files:
  created: []
  modified:
    - hp41-core/src/ops/time/date_arith.rs
    - hp41-core/src/ops/time/clock.rs
    - hp41-core/src/ops/time/stopwatch.rs
decisions:
  - "JDN value for 2026-05-24 is 2461185 (not 2460820 as cited in plan spec; Python datetime + FVF both confirm); 2026-05-24 is Sunday (not Saturday as cited)"
  - "Date result tests compare (year, month, day) tuples via parse_date_hpnum round-trip instead of HpNum string representation (avoids HpNum::rounded trailing-zero artefacts)"
  - "decimal_to_i64 helper function for Decimal→i64 conversion (replaces ToI64OrErr trait)"
metrics:
  duration_minutes: 36
  completed: "2026-05-24"
  tasks_completed: 2
  files_changed: 3
---

# Phase 38 Plan 02: JDN Date Arithmetic Engine Summary

JDN Fliegel-Van Flandern date arithmetic engine + DATE+, DDAYS, DOW, DMY, MDY ops in `date_arith.rs`; replaces Wave 1 stubs with full implementation.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | JDN algorithms + parse helpers | e2abedf | hp41-core/src/ops/time/date_arith.rs |
| 2 | DATE+, DDAYS, DOW, DMY, MDY op implementations | e2abedf | hp41-core/src/ops/time/date_arith.rs |

## What Was Built

### Core Implementation (`date_arith.rs`)

**JDN Algorithms (Fliegel-Van Flandern 1968):**
- `date_to_jdn(year, month, day) -> i64`: converts Gregorian date to Julian Day Number using ACM 1968 integer formula. All arithmetic in i64 (T-38-01 overflow guard).
- `jdn_to_date(jdn) -> (year, month, day)`: inverse FVF algorithm.
- `jdn_to_dow(jdn) -> i32`: `(jdn+1) % 7`, giving 0=Sunday..6=Saturday per HP 82182A OM.

**Parse Helpers:**
- `parse_date_hpnum(hpnum, dmy) -> (year, month, day)`: string-split-at-decimal with left-pad to exactly 6 fractional chars (D-carried.2/P35 invariant). Validates month 1-12, day 1-31, uses JDN round-trip for impossible-date rejection (e.g. Feb 30). Returns `HpError::InvalidInput` on failure (T-38-04).
- `parse_time_hpnum(hpnum) -> (hours, minutes, seconds, centiseconds)`: 6-digit fractional HH.MMSScc format per P44 (NOT reusing `hms.rs`). Range validation T-38-05.
- `secs_to_hpnum_time(secs) -> HpNum`: f64 seconds to HH.MMSScc decimal format.

**Date Operations:**
- `op_date_plus`: Y-register date + X-register integer days; uses `binary_result`; respects Flag 31.
- `op_ddays`: signed day difference JDN(Y) - JDN(X); uses `binary_result`.
- `op_dow`: day-of-week 0-6 for X-register date; uses `unary_result`.
- `op_dmy`: `state.flags |= 1u64 << 31` + `LiftEffect::Neutral`.
- `op_mdy`: `state.flags &= !(1u64 << 31)` + `LiftEffect::Neutral`.

**Test Coverage:** 40 tests covering:
- JDN round-trips for 2026-05-24, 2000-01-01, 1582-10-15 (Gregorian start)
- Leap year boundary (2000-02-29 valid, 2100-02-29 invalid)
- Trailing-zero year parsing (2000, 2010, 2020)
- parse_time_hpnum 6-digit extraction including leading zeros
- All 5 date ops with MDY/DMY modes, leap year (2000), century boundary (2100)
- Stack drop behavior via binary_result

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Incorrect JDN reference value and day-of-week in plan spec**
- **Found during:** Task 1 — `jdn_known_reference_2026_05_24` test failure
- **Issue:** The Phase 38 research file (and plan acceptance criteria) cited `date_to_jdn(2026, 5, 24) == 2460820` and described 2026-05-24 as "Saturday". The FVF algorithm returns 2461185; Python `datetime` independently confirms JDN=2461185 and DOW=Sunday. The cited value 2460820 is the JDN for 2025-05-24 (off by one year).
- **Fix:** Updated test expectations to use JDN=2461185 (Sunday) with explanatory comments citing Python datetime verification. The algorithm implementation is correct per Fliegel-Van Flandern 1968.
- **Files modified:** `hp41-core/src/ops/time/date_arith.rs`
- **Commit:** e2abedf

**2. [Rule 3 - Blocking] HpNum::rounded adds trailing zeros to date string representation**
- **Found during:** Task 2 — `op_date_plus_jan_1_2026_plus_31` test failure ("got 2.012026000" not "2.012026")
- **Issue:** `HpNum::rounded(d)` calls `round_sf_with_strategy(10, ...)` which extends the scale to 10 significant digits, adding trailing zeros. String comparison `"2.012026000"` ≠ `"2.012026"` fails.
- **Fix:** Changed date op tests to compare (year, month, day) tuples via `parse_date_hpnum` round-trip, and DDAYS/DOW tests to compare integer values via `decimal_to_i64` helper. This is semantically more correct and format-independent.
- **Files modified:** `hp41-core/src/ops/time/date_arith.rs`
- **Commit:** e2abedf

**3. [Rule 3 - Blocking] Pre-existing clippy `-D warnings` errors in Wave 1 files**
- **Found during:** `cargo clippy -p hp41-core -- -D warnings` run
- **Issue:** `clock.rs` and `stopwatch.rs` had derivable_impls clippy errors (`ClockDisplayMode`, `StopwatchMode` impl Default could use `#[derive(Default)]`), and a manual `!Range::contains` pattern in `stopwatch.rs`.
- **Fix:** Added `#[derive(Default)]` + `#[default]` attribute to both enums, replaced manual range check with `!(0.0..360_000.0).contains(&secs)`.
- **Files modified:** `hp41-core/src/ops/time/clock.rs`, `hp41-core/src/ops/time/stopwatch.rs`
- **Commit:** e2abedf

## Known Stubs

None — all functions in `date_arith.rs` are fully implemented. Wave 1 stubs are replaced.

## Verification Results

- `cargo test -p hp41-core -- date_arith`: **40 passed**
- `cargo test -p hp41-core`: **2127 passed, 1 ignored**
- `cargo clippy -p hp41-core -- -D warnings`: **No issues found**
- `scripts/check-free42-contamination.sh`: **OK** (exits 0, no distinctive tokens)
- `grep "1u64 << 31" date_arith.rs`: 5 occurrences confirming Flag 31 integration
- No references to `hms.rs` or `parse_hms` in production code

## Threat Model Coverage

| Threat ID | Mitigation Status |
|-----------|-------------------|
| T-38-04 (parse_date_hpnum invalid input) | Mitigated — month 1-12, day 1-31, JDN round-trip validation, HpError::InvalidInput |
| T-38-05 (parse_time_hpnum overflow) | Mitigated — hours 0-23, minutes 0-59, seconds 0-59, centiseconds 0-99 |
| T-38-06 (JDN overflow extreme dates) | Accepted — i64 supports year 5,000,000+ |
| T-38-SC (no new cargo installs) | Confirmed — zero new runtime dependencies |

## Self-Check: PASSED

Checked files exist:
- `hp41-core/src/ops/time/date_arith.rs` — FOUND (27KB, full implementation)
- `hp41-core/src/ops/time/clock.rs` — FOUND (clippy fix applied)
- `hp41-core/src/ops/time/stopwatch.rs` — FOUND (clippy fix applied)

Checked commits exist:
- `e2abedf` (feat(38-02): implement JDN date arithmetic engine) — FOUND

All 2127 hp41-core tests pass. Clippy clean.
