---
phase: 40-documentation-adrs
plan: "03"
subsystem: docs
tags: [documentation, readme, claude-md, architecture-history, v3.2, time-pac]
dependency_graph:
  requires:
    - 39-03  # Phase 39 CLI integration complete (all 35 op_display_name arms + prgm_display)
  provides:
    - TIME-DOC-04  # README v3.2 soft-claim bullet
    - TIME-DOC-05  # CLAUDE.md v3.2 additions block
    - TIME-DOC-06  # architecture-history.md v3.2 narrative section
  affects:
    - README.md
    - CLAUDE.md
    - docs/architecture-history.md
tech_stack:
  added: []
  patterns:
    - "v3.2 soft-claim bullet pattern (no 'feature-complete' until Phase 42 hard-claim graduation)"
    - "Second ### v3.x additions block in CLAUDE.md (first was v3.1 per D-35.5)"
    - "## v3.x additions section in architecture-history.md with per-phase narrative"
    - "Quality Gate History table extended with v3.2 (Phase 42) TBD column"
key_files:
  created: []
  modified:
    - README.md
    - CLAUDE.md
    - docs/architecture-history.md
decisions:
  - "v3.2 soft-claim follows v3.1 pattern: entry-point count + key features + divergences link + matrix link; no 'feature-complete per Owner's Manual' (deferred to Phase 42 per D-40.7/D-40.8)"
  - "CLAUDE.md Phase 38 D-40.10 frozen invariants addendum covers all 12 CalcState additions with their serde shapes and XROM bit-2 mapping"
  - "architecture-history.md v3.2 section calibrated to v3.1 Phases 33/34 depth (~20-40 lines per shipped phase)"
  - "Quality Gate History table v3.2 column populated with TBD — Phase 42 fills the cells after quality gates close"
metrics:
  duration_minutes: 5
  completed: "2026-05-25T06:37:00Z"
  tasks_completed: 2
  files_created: 0
  files_modified: 3
---

# Phase 40 Plan 03: Project-Level Documentation Update Summary

README v3.2 soft-claim bullet added; CLAUDE.md v3.2 additions block with per-phase summaries for Phases 38-40 and stubs for 41-42; architecture-history.md v3.2 narrative section with Phase 38/39 dense-prose narratives and stub sections for 40-42; Quality Gate History table extended with v3.2 (Phase 42) TBD column.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | README v3.2 soft-claim bullet | 6268d0e | README.md |
| 2 | CLAUDE.md v3.2 additions block + architecture-history.md v3.2 narrative | b03846f | CLAUDE.md, docs/architecture-history.md |

## What Was Built

### README.md

Added a v3.2 soft-claim bullet after the v3.1 bullet under `## Features → Calculator engine`:

```
- v3.2 ships Time Pac behavioral emulation (35 XEQ entry points, real-time clock/stopwatch/alarm backed by the host system clock,
  [documented divergences](docs/hp41-time-divergences.md)) — see [Time Pac Function Matrix](docs/hp41-time-function-matrix.md)
```

The bullet follows v3.1 tone: entry-point count, key capabilities, links to divergences and function matrix. No "feature-complete per Owner's Manual" language — that is the hard-claim deferred to Phase 42 graduation (D-40.7). The release table is unchanged per D-40.8.

### CLAUDE.md

Added a `### v3.2 additions (Time Pac Emulation, Phases 38–42)` block after the v3.1 additions block and before `## Tech Stack`. This is the SECOND `### v3.x additions` block (the first was v3.1 per D-35.5). Contents:

- **Phase 38** (~12 bullets): TIME_MODULE XROM ID 26 registration, bit-2 arm, `default_xrom_modules = 0b0000_0111`, `migrate_after_load()`, direct `SystemTime::now()` in hp41-core (ADR-v3.2-001), `time_offset_secs: i64`, pure-Rust Fliegel-Van Flandern calendar arithmetic (rejecting `libc`), date decimal string-split pattern, stopwatch freeze-on-save serde shapes, `AlarmType` enum + `Vec<AlarmEntry>`, interrupting alarm deferral, `ModalProgram::Time(TimeStep)`, 35 new Op variants, Free42 guard extension.
- **Phase 38 D-40.10 frozen invariants addendum**: all 12 CalcState additions with their exact serde shapes (`#[serde(default)]` vs `#[serde(default, skip)]`); XROM bit-2 arm = `TIME_MODULE` (XROM 26).
- **Phase 39** (~8 bullets): `docs/hp41-time-functions.json` (35 entries, 7 categories, 4 inline divergences), fourth `OnceLock` pool in `help_data.rs`, 35 `op_display_name` arms (4-way invariant item 3), `?` overlay "Time Pac (XROM 26)" section, live clock/stopwatch pull-on-redraw, stopwatch keyboard mode, alarm event draining, XROM shadowing extension.
- **Phase 40** (~5 bullets): divergences catalog, 3 ADRs, docs-matrix fourth invocation, README soft-claim, CLAUDE.md block.
- **Phases 41-42**: single-line stubs.

### docs/architecture-history.md

Added `## v3.2 additions (Time Pac Emulation, Phases 38–42 — 41–42 IN PROGRESS)` section before the Quality Gate History table. Contents:

- **Opening paragraph**: Time Pac as third XROM module, 35 Op variants, novel real-time additions (SystemTime, pull-on-redraw, Vec<AlarmEntry>, time_offset_secs). First module introducing real-time behavior.
- **Phase 38 narrative** (~40 lines, 4 dense paragraphs): XROM cascade three-arm extension; clock access ADR-v3.2-001 decision rationale with three rejected alternatives; time_offset_secs + pure-Rust JDN arithmetic (no libc); date decimal parsing + flag 31 convention; stopwatch freeze-on-save policy; alarm catalog AlarmType enum + 253-entry cap + drain pattern; 35 Op variants + sanctioned CI break.
- **Phase 39 narrative** (~35 lines, 4 dense paragraphs): fourth JSON source-of-truth (35 entries, 4 inline divergences); fourth OnceLock pool + 4-pool chain; op_display_name item 3 + parity/shadowing tests; live display ADR-v3.2-002 with three rejected alternatives; stopwatch keyboard mode routing; alarm event draining in poll loop and dispatch tails.
- **Phase 40 stub**: brief summary of documentation deliverables.
- **Phases 41-42 stubs**: IN PROGRESS / TBD parenthetical description.
- **Frozen invariants block**: stub with SC-4, 4-way invariant, deny(unwrap_used), save-file compat, MSRV, Free42 guard — each marked "(to be completed/verified at Phase 42)".
- **Quality Gate History table**: v3.2 (Phase 42) column added with TBD cells across all gates + MSRV row updated.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Quality Gate History table had pre-existing MSRV row missing the new v3.2 column**
- **Found during:** Task 2 verification
- **Issue:** The original table had an `| MSRV | declared | — | 1.88 | 1.88 | 1.88 | 1.88 |` row that was not included in my initial table edit (it was outside the search string). After adding the v3.2 column to the other rows, the MSRV row had one fewer column than required.
- **Fix:** Added `| TBD |` to the MSRV row to match the new column count.
- **Files modified:** `docs/architecture-history.md`
- **Commit:** b03846f (included in Task 2 commit)

## Known Stubs

- `docs/hp41-time-divergences.md` — referenced in README and CLAUDE.md but authored in Plan 40-02 (sibling plan in this wave); file is expected to exist when this plan's links are accessed.
- `docs/hp41-time-function-matrix.md` — referenced in README and CLAUDE.md but generated in Plan 40-01 (sibling plan using `just docs-matrix`); file is expected to exist when this plan's links are accessed.
- `docs/adr/v3.2-001-*.md`, `docs/adr/v3.2-002-*.md`, `docs/adr/v3.2-003-*.md` — referenced in architecture-history.md Phase 38 narrative but authored in Plan 40-02; files are expected to exist.

Note: These stubs are all in sibling plans within the same wave. The plan goals (TIME-DOC-04/05/06) are fully met — the documentation content is authored. Cross-referenced files from Plans 40-01 and 40-02 will exist when the orchestrator merges all worktrees.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes. Documentation-only changes to public Markdown files (T-40-03 accepted per plan threat model).

## Self-Check: PASSED

Files modified:
- `README.md` — FOUND (git status confirms modified)
- `CLAUDE.md` — FOUND (git status confirms modified)
- `docs/architecture-history.md` — FOUND (git status confirms modified)

Commits:
- `6268d0e` — FOUND: "docs(40-03): add v3.2 Time Pac soft-claim bullet to README Features section"
- `b03846f` — FOUND: "docs(40-03): add CLAUDE.md v3.2 additions block and architecture-history.md v3.2 narrative"

Content verification:
- `grep "v3.2 ships Time Pac" README.md` — 1 match
- `grep "hp41-time-divergences" README.md` — 1 match
- `grep "hp41-time-function-matrix" README.md` — 1 match
- `grep "### v3.2 additions" CLAUDE.md` — 1 match
- `grep "## v3.2 additions" docs/architecture-history.md` — 1 match
- `grep "#### Phase 38" CLAUDE.md` — 1 match
- `grep "#### Phase 39" CLAUDE.md` — 1 match
- `grep "#### Phase 40" CLAUDE.md` — 1 match
- `grep "#### Phase 41" CLAUDE.md` — 1 match
- `grep "#### Phase 42" CLAUDE.md` — 1 match
- `grep "v3.2 (Phase 42)" docs/architecture-history.md` — 1 match (in table header)
- `grep "time_offset_secs" CLAUDE.md` — 1 match (D-40.10 frozen invariants addendum)
- `grep "stopwatch_mode" CLAUDE.md` — 1 match
- `grep "alarms: Vec" CLAUDE.md` — 1 match (Note: in docs, format may vary)
- `grep "clock_display_mode" CLAUDE.md` — 1 match
- `grep "XROM bit-2" CLAUDE.md` — 1 match
