---
phase: 50-raw-file-i-o
verified: 2026-05-28T14:35:00Z
status: passed
score: 6/6 requirements satisfied
verification_type: retroactive (evidence-based)
evidence_source: v4.0-MILESTONE-AUDIT.md (cross-phase integration check)
---

# Phase 50: .raw File I/O — Verification Report

**Status:** PASSED (retroactive)
**Note:** Shipped (ROADMAP `[x]`, SUMMARYs 50-01..04, merged codec + Tauri commands + CLI flags + tests) but goal-verification was not produced at execution time. Retroactive evidence-based verification compiled during the v4.0 milestone audit (2026-05-28). Primary evidence: `gsd-integration-checker` report in `.planning/v4.0-MILESTONE-AUDIT.md`, green `just test` (cardreader suites), ADR-v4.0-006. RAW-01..06 are also recorded in the 50-0x SUMMARY `requirements-completed` frontmatter.

## Requirements Coverage

| Req | Description | Status | Evidence |
|-----|-------------|--------|----------|
| RAW-01 | Import single-program `.raw` into memory | satisfied | `import_raw_dialog` → `decode_all_programs` → `Single` → `setCalcState`; SUMMARY 50-02/03 |
| RAW-02 | Export program to `.raw` | satisfied | `export_raw_dialog` Tauri command → `exportRawDialog` (App.tsx); SUMMARY 50-02/03 |
| RAW-03 | Multi-program archive handled (import all / clear error) | satisfied | `decode_all_programs` splits at END markers; `ARCHIVE_CAP=256` DoS guard (ADR-v4.0-006); SUMMARY 50-01 |
| RAW-04 | Native OS dialog via `tauri-plugin-dialog` | satisfied | `Multi` → `RawPickerOverlay` → `import_selected_programs`; anti-deadlock dialog-before-lock ordering (ADR-v4.0-006) |
| RAW-05 | XROM instructions preserved in import | satisfied | `SyntheticByte`/XROM round-trip test in `cardreader_raw_multi.rs`; SUMMARY 50-01 |
| RAW-06 | CLI `--import-raw`/`--export-raw` (+ data/batch) | satisfied | `hp41-cli/src/main.rs` flags share the `hp41-core` codec; integration test `batch_exits_zero_without_tui` |

**Score:** 6/6 satisfied (integration-confirmed; corroborated by SUMMARY frontmatter).

## Gaps

None blocking. `50-VALIDATION.md` (Nyquist strategy) left in draft. Non-blocking manual checkpoint: file-dialog + multi-program picker overlay visual verification not yet recorded (tracked in the milestone audit).
