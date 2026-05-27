---
phase: 43
plan: 02
subsystem: hp41-core
tags: [advantage-pac, xrom, conv, bitwise, base-conversion, 36-bit]
depends_on:
  requires: [43-01-advantage-skeleton]
  provides: [43-02-adv-conv-implementation]
  affects: [hp41-core/src/ops/advantage/conv.rs]
tech_stack:
  added: []
  patterns: [u64-to-hpnum-exact-bypass, x-to-u64-masked-helper, binary-result-stack-drop, apply-lift-effect]
key_files:
  created: []
  modified:
    - hp41-core/src/ops/advantage/conv.rs
decisions:
  - "u64_to_hpnum uses HpNum(Decimal) direct constructor to bypass HpNum::rounded() — ADV_WORD_MASK = 2^36-1 = 68,719,476,735 has 11 decimal digits which rounded() truncates to 10"
  - "BIT? uses binary_result (stack drops) with 1.0/0.0 result — skip semantics deferred to caller via XNeZero test pattern"
  - "Test helpers set_x_u64/set_y_u64/get_x_u64 added for 11-digit values (35-36 bit range)"
  - "x_to_u64_masked uses f64 bridge (sufficient: f64 exactly represents 36-bit integers, all <= 2^36 << 2^53)"
metrics:
  duration: "~20 minutes"
  completed: "2026-05-25T19:53:31Z"
  tasks_completed: 1
  tasks_total: 1
  files_created: 0
  files_modified: 1
---

# Phase 43 Plan 02: ADV CONV Operations Implementation Summary

## One-liner

12 ADV CONV operations (base conversion + bitwise logic) with 36-bit word size using exact u64-to-Decimal conversion bypassing HpNum's 10-digit rounding limit.

## Tasks Completed

| # | Task | Status | Commit |
|---|------|--------|--------|
| 1 | Implement all 12 ADV CONV operations with 36-bit word size | Done | db37af2 |

## What Was Built

### ADV CONV Operations (12 total)

**Base Conversion Input (ALPHA → X):**
- `op_adv_binin` (ADV-CONV-01): Parse ALPHA as binary string → X; Domain if empty or invalid digits
- `op_adv_octin` (ADV-CONV-03): Parse ALPHA as octal string → X; Domain if empty or invalid digits
- `op_adv_hexin` (ADV-CONV-05): Parse ALPHA as hex string (case-insensitive) → X; Domain if empty or invalid digits

**Base Conversion Display (X → ALPHA + print_buffer):**
- `op_adv_binview` (ADV-CONV-02): X → binary format in ALPHA and print_buffer (LiftEffect::Neutral)
- `op_adv_hexview` (ADV-CONV-06): X → uppercase hex format in ALPHA and print_buffer (LiftEffect::Neutral)
- `op_adv_cvtview` (ADV-CONV-04): X → 4 print lines (BIN/OCT/DEC/HEX formats) (LiftEffect::Neutral)

**Bitwise Logic (unary):**
- `op_adv_not` (ADV-CONV-07): XOR with ADV_WORD_MASK → complement within 36 bits (LiftEffect::Enable via unary_result)

**Bitwise Logic (binary, stack drops Y):**
- `op_adv_and` (ADV-CONV-08): y_val & x_val → X (via binary_result)
- `op_adv_or` (ADV-CONV-09): y_val | x_val → X (via binary_result)
- `op_adv_xor` (ADV-CONV-10): y_val ^ x_val → X (via binary_result)

**Rotation:**
- `op_adv_rotxy` (ADV-CONV-11): Circular rotation of Y by X bits within 36-bit word; positive X = left, negative X = right; uses rem_euclid for negative shift reduction

**Bit Test:**
- `op_adv_bit_test` (ADV-CONV-12): Test bit Y of X; result 1.0 if set, 0.0 if clear; both X and Y consumed via binary_result

### Key Helpers

- `x_to_u64_masked(x: &HpNum) -> Result<u64, HpError>`: f64 round-trip with absolute value, masked to ADV_WORD_MASK; Domain for NaN/Inf or >2^53
- `u64_to_hpnum(val: u64) -> HpNum`: Uses `HpNum(Decimal::from_u64(...))` direct constructor, bypassing `HpNum::rounded()` which would truncate 11-digit values

### Test Coverage (41 tests)

Tests cover: basic operation correctness, empty ALPHA errors, invalid-digit Domain errors, overflow truncation (D-43.10 silent masking), boundary values (ADV_WORD_MASK, MSB at bit 35), stack drop behavior, LiftEffect neutrality for view ops, round-trip properties (NOT double negation, XOR self gives 0).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] 11-digit value precision issue with HpNum::rounded()**
- **Found during:** RED test run (NOT tests and ROTXY right-by-one failed)
- **Issue:** `ADV_WORD_MASK = 68,719,476,735` (11 decimal digits) is silently truncated to 10 digits by `HpNum::rounded()`, giving 68,719,476,740. This caused NOT(0) → 68,719,476,740 instead of 68,719,476,735, and NOT(ADV_WORD_MASK) → 4 instead of 0. Similarly, `1u64 << 35 = 34,359,738,368` (11 digits) → 34,359,738,370.
- **Fix:** `u64_to_hpnum()` uses `HpNum(Decimal::from_u64(val)...)` directly, bypassing `HpNum::rounded()`. f64 can represent 36-bit integers exactly (2^36 << 2^53), so `x_to_u64_masked()` remains safe with the f64 bridge. Test helpers `set_x_u64/set_y_u64/get_x_u64` added for 11-digit precision in tests.
- **Files modified:** hp41-core/src/ops/advantage/conv.rs
- **Commit:** db37af2

## Known Stubs

None — all 12 ADV CONV operations are fully implemented. The previous `conv_stubs_return_invalid_op` test was replaced by 41 behavioral tests.

## Threat Surface Scan

No new network endpoints, auth paths, file access, or schema changes. The ALPHA → integer parsing path (T-43-04) is mitigated by `u64::from_str_radix` returning `Result<_, _>` mapped to `HpError::Domain`. T-43-05 (very long ALPHA) is accepted: HP-41 ALPHA is 24-char capped; `from_str_radix` on 24 chars is trivial.

## Self-Check

- [x] `hp41-core/src/ops/advantage/conv.rs` — exists and modified
- [x] Commit db37af2 exists
- [x] All 2481 hp41-core tests pass
- [x] Clippy: 0 errors, 0 warnings in conv.rs (pre-existing tvm.rs warning unrelated to this plan)
