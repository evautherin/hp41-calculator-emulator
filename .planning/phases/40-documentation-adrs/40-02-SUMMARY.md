---
phase: 40-documentation-adrs
plan: "02"
subsystem: docs/adr
tags: [documentation, adr, time-pac, v3.2, architecture]
dependency_graph:
  requires: [38-CONTEXT.md, 39-CONTEXT.md, docs/adr/v3.1-001-rng-state-placement.md]
  provides: [docs/adr/v3.2-001-clock-access-pattern.md, docs/adr/v3.2-002-live-display-architecture.md, docs/adr/v3.2-003-alarm-catalog-design.md]
  affects: [TIME-DOC-03]
tech_stack:
  added: []
  patterns: [D-30.6 long-form ADR template, D-30.7 verbatim CONTEXT quote convention]
key_files:
  created:
    - docs/adr/v3.2-001-clock-access-pattern.md
    - docs/adr/v3.2-002-live-display-architecture.md
    - docs/adr/v3.2-003-alarm-catalog-design.md
  modified: []
decisions:
  - "Three v3.2 ADRs authored following D-30.6 long-form template with verbatim 38-CONTEXT.md/39-CONTEXT.md decision quotes"
  - "ADR v3.2-001 documents why SystemTime is a value-returning syscall (not I/O) and the time_offset_secs design"
  - "ADR v3.2-002 documents pull-on-redraw pattern satisfying 1 Hz clock and 10 Hz stopwatch update requirements via existing 16ms poll loop"
  - "ADR v3.2-003 documents Vec<AlarmEntry> on CalcState with event_buffer drain, matching regs/programs precedents"
metrics:
  duration: "~15 minutes"
  completed: "2026-05-25"
  tasks_completed: 2
  tasks_total: 2
  files_created: 3
  files_modified: 0
---

# Phase 40 Plan 02: ADR Authoring (v3.2 Time Pac) Summary

Three long-form ADRs authored for v3.2 Time Pac architectural decisions: SystemTime clock access, pull-on-redraw live display, and Vec<AlarmEntry> alarm catalog with event_buffer drain.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Author ADR v3.2-001 and v3.2-002 | fba0287 | docs/adr/v3.2-001-clock-access-pattern.md, docs/adr/v3.2-002-live-display-architecture.md |
| 2 | Author ADR v3.2-003 | 13f61ab | docs/adr/v3.2-003-alarm-catalog-design.md |

## ADR Summaries

### ADR v3.2-001: Clock Access Pattern

Direct `std::time::SystemTime::now()` in hp41-core — explains why `SystemTime` is a
value-returning syscall distinct from I/O, how `time_offset_secs: i64` encodes the
user-set time delta, and why timezone is handled implicitly through the offset. Alternatives
considered: trait injection (Option B), frontend callback (Option C), no_std feature flag
(Option D). Verbatim D-38.1 quote included in Alternatives Considered.

### ADR v3.2-002: Live Display Architecture

Pull-on-redraw pattern: `get_clock_display_str()` and `get_stopwatch_display_str()` are
pure compute functions called on each frontend redraw. The existing 16ms ratatui poll loop
(~62 Hz) trivially exceeds the 1 Hz clock and 10 Hz stopwatch update requirements.
GUI uses a conditional `setInterval` (D-11 exception). Alternatives considered: async
timer in core (Option B), push-from-core event system (Option C), dedicated polling thread
(Option D). Verbatim D-39.1 and D-39.2 quotes included.

### ADR v3.2-003: Alarm Catalog Design

`Vec<AlarmEntry>` on CalcState with `#[serde(default)]` — matches the `regs: Vec<HpNum>`
and `programs: Vec<Program>` precedents. `AlarmType` enum with `Message(String)` and
`Control { label, interrupting }` variants is forward-compatible for eventual interrupting
alarm execution. `check_alarms()` drains past-due alarms into `event_buffer`/`print_buffer`
using the established drain pattern. Repeat interval stored as `i64` seconds to avoid
rounding drift. Alternatives considered: nested AlarmState struct (Option B), mpsc channel
(Option C), AlarmObserver trait (Option D). Verbatim D-38.8/D-38.9/D-38.10 quotes included.

## Requirements Satisfied

- TIME-DOC-03: Three ADRs authored for clock access pattern, live display architecture,
  and alarm catalog design.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — documentation-only plan; no runtime code or data-source stubs.

## Threat Flags

None — documentation-only changes (ADR prose files). No runtime code, no new endpoints,
no user input processing.

## Self-Check: PASSED

- `docs/adr/v3.2-001-clock-access-pattern.md` — FOUND
- `docs/adr/v3.2-002-live-display-architecture.md` — FOUND
- `docs/adr/v3.2-003-alarm-catalog-design.md` — FOUND
- Commit fba0287 — FOUND (Task 1)
- Commit 13f61ab — FOUND (Task 2)
- All three ADRs contain `## Alternatives Considered` section — VERIFIED
- All three ADRs contain verbatim 38-CONTEXT.md/39-CONTEXT.md blockquotes — VERIFIED
- All three ADRs contain `*ADR-v3.2-NNN locked: 2026-05-24.*` footer — VERIFIED
