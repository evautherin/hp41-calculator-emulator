---
phase: 50-raw-file-i-o
plan: 01
subsystem: cardreader
tags: [rust, raw-codec, multi-program, xrom, hp41-core]

# Dependency graph
requires:
  - phase: 49-onboarding-gui-keyboard-parity
    provides: "No direct dependency — this plan extends existing cardreader/raw.rs"
provides:
  - "decode_all_programs: splits multi-program .raw byte streams at END markers"
  - "DecodedProgram: struct (ops, byte_len) for archive entries"
  - "picker_label: formats 'NAME (N bytes)' or 'Program N (N bytes)' per D-50.5"
  - "256-program archive cap (DoS guard T-50-01)"
  - "Integration test suite: 8 tests in tests/cardreader_raw_multi.rs"
affects: [50-02, 50-03, 50-04, gui-import-export, xmem-core]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "TDD RED-GREEN: inline tests written before implementation"
    - "Archive cap pattern: iterate with counter, return HpError::CardData if exceeded"
    - "picker_label: iterate ops looking for first Op::Lbl, format as 'NAME (N bytes)'"

key-files:
  created:
    - hp41-core/tests/cardreader_raw_multi.rs
  modified:
    - hp41-core/src/cardreader/raw.rs
    - hp41-core/src/cardreader/mod.rs

key-decisions:
  - "decode_all_programs slices at END markers and delegates to decode_program for per-segment validation — no duplicate parser logic"
  - "256-program archive cap chosen as reasonable upper bound for community .raw files (T-50-01 DoS guard)"
  - "END_MARKER made pub(crate) to allow integration test access without exposing it publicly"
  - "picker_label: first Op::Lbl wins; index+1 fallback uses 1-based numbering per HP-41 UI conventions"

patterns-established:
  - "Multi-program archive splitting: find END_MARKER via windows(3).position(), slice segment, decode with existing decode_program"
  - "Archive cap guard: check programs.len() >= ARCHIVE_CAP before each push iteration"

requirements-completed: [RAW-03, RAW-05]

# Metrics
duration: 5min
completed: 2026-05-27
---

# Phase 50 Plan 01: .raw Multi-Program Codec Summary

**decode_all_programs splits community .raw archives at END markers, DecodedProgram carries (ops, byte_len), and picker_label formats the import UI label per D-50.5 — all with 256-program DoS cap and XROM SyntheticByte round-trip guarantee**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-05-27T19:19:45Z
- **Completed:** 2026-05-27T19:24:21Z
- **Tasks:** 2 of 2
- **Files modified:** 3

## Accomplishments
- `decode_all_programs` correctly splits multi-program `.raw` archives at END markers and decodes each segment via existing `decode_program`
- `DecodedProgram` struct (ops + byte_len) provides structured access for the import picker UI
- `picker_label` formats "NAME (N bytes)" or "Program N (N bytes)" per D-50.5
- 256-program archive cap implemented (DoS guard T-50-01) — crafted inputs cannot cause memory exhaustion
- 8 integration tests in `cardreader_raw_multi.rs` cover all edge cases including XROM round-trip (RAW-05)
- 7 inline unit tests added to `raw.rs`
- Total: 2965 hp41-core tests passing (up from 2957)

## Task Commits

1. **Task 1: Add decode_all_programs, DecodedProgram, and picker_label** - `10cd7da` (feat)
2. **Task 2: Integration tests for multi-program decoding and XROM round-trip** - `b5b8bcc` (test)

## Files Created/Modified
- `hp41-core/src/cardreader/raw.rs` - Added DecodedProgram struct, decode_all_programs function, picker_label function; END_MARKER made pub(crate); 7 inline tests
- `hp41-core/src/cardreader/mod.rs` - Added re-exports: decode_all_programs, DecodedProgram, picker_label
- `hp41-core/tests/cardreader_raw_multi.rs` - 8 integration tests: single/two/three-program split, empty input, truncated stream, XROM round-trip, picker_label with/without LBL

## Decisions Made
- `decode_all_programs` delegates each segment to the existing `decode_program` rather than duplicating parser logic — keeps a single source of truth for single-byte op decoding
- END_MARKER visibility raised to `pub(crate)` (was private const) to allow test access in the same crate
- Archive cap set at 256 programs — chosen as a practical upper bound for HP-41 community archives while blocking DoS via crafted inputs

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## Threat Surface Scan

No new network endpoints, auth paths, or trust-boundary crossings introduced. `decode_all_programs` sits behind the existing filesystem->hp41-core boundary. T-50-01 (archive cap) mitigated as planned. T-50-02 and T-50-03 handled by delegating to existing `decode_program` validation.

## Next Phase Readiness
- `decode_all_programs` and `DecodedProgram` are ready for Plan 50-02 (Tauri import command) and Plan 50-03 (CLI `--import-raw` flag)
- `picker_label` is ready for the multi-program picker UI (Plan 50-02)
- No blockers

## Self-Check: PASSED

| Check | Result |
|-------|--------|
| raw.rs exists | FOUND |
| mod.rs exists | FOUND |
| cardreader_raw_multi.rs exists | FOUND |
| SUMMARY.md exists | FOUND |
| Commit 10cd7da (Task 1) | FOUND |
| Commit b5b8bcc (Task 2) | FOUND |
| DecodedProgram struct | PASS |
| decode_all_programs function | PASS |
| picker_label function | PASS |
| Re-exports in mod.rs | PASS |
| 256-program archive cap | PASS |
| 8+ integration tests | PASS (8) |
| All 2965 tests passing | PASS |

---
*Phase: 50-raw-file-i-o*
*Completed: 2026-05-27*
