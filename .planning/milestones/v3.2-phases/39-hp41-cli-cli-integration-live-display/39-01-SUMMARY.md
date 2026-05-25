---
phase: 39-hp41-cli-cli-integration-live-display
plan: 01
subsystem: cli
tags: [rust, json, help-data, xrom, time-pac, onceLock]

# Dependency graph
requires:
  - phase: 38-hp41-core-time-pac-core-ops
    provides: TIME_MODULE const with 35 Op variants and xrom_resolve bit-2 arm
provides:
  - docs/hp41-time-functions.json (35-entry canonical JSON for Time Pac XROM 26)
  - help_entries_time() fourth OnceLock accessor in help_data.rs
  - help_entries_all() 4-pool chain (cv + math1 + stat1 + time)
  - TIME_OP_VARIANT_NAMES + 3 parity tests in function_matrix_parity.rs
  - 4 TIME_MODULE shadowing/disjointness tests in xrom_shadowing.rs
affects: [39-02, 39-03, docs-matrix-extension, gui-integration]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Fourth OnceLock pool: TIME_FUNCTIONS_JSON + TIME_HELP_ENTRIES + help_entries_time() mirroring the Stat 1 pattern (D-39.12)"
    - "Per-module TIME_OP_VARIANT_NAMES inventory in function_matrix_parity.rs mirrors MATH1/STAT1 precedent"
    - "Bit mask 0b0000_0111 for xrom_resolve tests when all three modules loaded"

key-files:
  created:
    - docs/hp41-time-functions.json
  modified:
    - hp41-cli/src/help_data.rs
    - hp41-cli/tests/function_matrix_parity.rs
    - hp41-core/tests/xrom_shadowing.rs

key-decisions:
  - "TIME_OP_VARIANT_NAMES lives separately from ALL_OP_VARIANT_NAMES (same pattern as MATH1_OP_VARIANT_NAMES and STAT1_OP_VARIANT_NAMES — built-in v2.2 pool stays at 130; each XROM module has its own inventory)"
  - "Divergences on exactly 4 entries: CORRECT (no-op), SW (emulator extension), RCLAF (no crystal), SETAF (stored but no effect)"
  - "xrom_resolve test uses 0b0000_0111 (all three module bits set) for Time Pac bidirectional consistency"

patterns-established:
  - "4-pool JSON chain: help_entries_all() now chains cv + math1 + stat1 + time; Advantage Pac extends via fifth .chain() arm"

requirements-completed: [TIME-CLI-01, TIME-CLI-02, TIME-CLI-04, TIME-CLI-08]

# Metrics
duration: 12min
completed: 2026-05-25
---

# Phase 39 Plan 01: Time Pac CLI Help Data Foundation Summary

**35-entry hp41-time-functions.json (XROM 26) authored and wired as the fourth OnceLock JSON pool in help_data.rs, with full bidirectional parity and XROM shadowing tests**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-05-25T05:11:00Z
- **Completed:** 2026-05-25T05:23:44Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Created `docs/hp41-time-functions.json` with exactly 35 entries covering all Time Pac (XROM 26) functions across 7 categories (Clock, Date Arithmetic, Display, Format, Alpha, Stopwatch, Alarm); 4 entries carry inline divergences (CORRECT, SW, RCLAF, SETAF)
- Wired `TIME_FUNCTIONS_JSON` + `TIME_HELP_ENTRIES` OnceLock + `help_entries_time()` as the fourth JSON pool in `hp41-cli/src/help_data.rs`; `help_entries_all()` now chains all four pools (D-39.12); `? help overlay "Time Pac (XROM 26)" section auto-generated from categories
- Extended `function_matrix_parity.rs` with `TIME_OP_VARIANT_NAMES` (35 entries) + 3 parity tests (inventory count, forward parity, reverse xrom_resolve); extended `test_pool_partition_is_exhaustive` with `time_count` + `Some(26)` arm + assertion == 35; 14 total tests pass
- Extended `xrom_shadowing.rs` with 4 new tests (builtin shadow check, math1+stat1 disjointness, xrom_resolve round-trip, const fields); 10 total xrom_shadowing tests pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Author hp41-time-functions.json and wire fourth OnceLock in help_data.rs** - `a5f7c0e` (feat)
2. **Task 2: Update function_matrix_parity.rs and extend xrom_shadowing.rs** - `6b4e7e0` (feat)

## Files Created/Modified

- `docs/hp41-time-functions.json` - 35-entry canonical JSON for Time Pac (XROM 26); 7 categories; module_id=26; function_ids 1-35 sequential
- `hp41-cli/src/help_data.rs` - Fourth OnceLock (TIME_FUNCTIONS_JSON, TIME_HELP_ENTRIES, help_entries_time()); help_entries_all() extended to 4-pool chain
- `hp41-cli/tests/function_matrix_parity.rs` - TIME_OP_VARIANT_NAMES (35 entries); 3 new Time parity tests; partition test extended for module_id 26; import extended with help_entries_time
- `hp41-core/tests/xrom_shadowing.rs` - TIME_MODULE imported; 4 new time_* tests covering builtins, disjointness, resolve round-trip, const fields

## Decisions Made

- `TIME_OP_VARIANT_NAMES` is a separate const from `ALL_OP_VARIANT_NAMES` (v2.2 built-ins stay at 130; each XROM module has its own per-module inventory following the MATH1/STAT1 precedent established in Phases 29 and 34)
- `xrom_resolve` bit mask `0b0000_0111` for Time tests (all three modules loaded); mirrors Stat 1 `0b0000_0011` precedent extended by one bit for Time (bit 2)
- Inline divergences on exactly 4 entries per D-39.9: CORRECT (no-op, authoritative host clock), SW (emulator-extension interactive mode), RCLAF (no crystal oscillator), SETAF (stored but no physical effect)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Moved Time Op variants from ALL_OP_VARIANT_NAMES to TIME_OP_VARIANT_NAMES**

- **Found during:** Task 2 (function_matrix_parity.rs update)
- **Issue:** The plan spec said "Append all 35 Time* variant name strings to ALL_OP_VARIANT_NAMES const array" and "Update test_op_inventory_count_matches_enum assertion from 130 to 165". However, this broke `test_every_rom_op_has_matrix_entry` because that test checks ALL_OP_VARIANT_NAMES against the v2.2 built-ins JSON only (`help_entries()`), not the XROM pool JSONs. Math Pac I and Stat 1 Pac both use separate per-module inventory constants (MATH1_OP_VARIANT_NAMES, STAT1_OP_VARIANT_NAMES) — Time Pac must follow the same pattern.
- **Fix:** Reverted ALL_OP_VARIANT_NAMES back to 130 entries; kept Time variants only in TIME_OP_VARIANT_NAMES; reverted test_op_inventory_count_matches_enum to assert == 130 with updated comment; all 14 parity tests pass
- **Files modified:** hp41-cli/tests/function_matrix_parity.rs
- **Verification:** `cargo test -p hp41-cli --test function_matrix_parity` — 14 passed
- **Committed in:** `6b4e7e0` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — bug in plan spec vs. established XROM inventory pattern)
**Impact on plan:** All success criteria met; fix was necessary for test correctness. The TIME_OP_VARIANT_NAMES approach is consistent with the existing MATH1_OP_VARIANT_NAMES / STAT1_OP_VARIANT_NAMES precedent.

## Issues Encountered

- `cargo test` commands from the main repo directory (not the worktree) ran stale binaries against the main repo's source files; all test invocations were corrected to run from the worktree root (`cd $WT_ROOT && cargo test ...`).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `docs/hp41-time-functions.json` + `help_entries_time()` are ready for Phase 39 Plan 02 (prgm_display.rs 35-arm extension)
- `? help overlay` will auto-generate "Time Pac (XROM 26)" section once prgm_display.rs provides op_display_name arms (4-way invariant item 3)
- `test_pool_partition_is_exhaustive` will guard against any future Advantage Pac additions without proper registration

## Self-Check: PASSED

- FOUND: docs/hp41-time-functions.json
- FOUND: hp41-cli/src/help_data.rs (modified)
- FOUND: hp41-cli/tests/function_matrix_parity.rs (modified)
- FOUND: hp41-core/tests/xrom_shadowing.rs (modified)
- FOUND: .planning/phases/39-hp41-cli-cli-integration-live-display/39-01-SUMMARY.md
- FOUND: commit a5f7c0e (Task 1 — JSON + OnceLock)
- FOUND: commit 6b4e7e0 (Task 2 — parity + shadowing tests)
- FOUND: commit 1523506 (SUMMARY)

---
*Phase: 39-hp41-cli-cli-integration-live-display*
*Completed: 2026-05-25*
