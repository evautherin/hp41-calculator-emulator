---
phase: 52-test-hardening-documentation
plan: "04"
subsystem: documentation
tags: [adr, divergences, readme, claude-md, architecture-history, x-mem]
dependency_graph:
  requires: []
  provides: [x-mem-adr-suite, x-mem-divergences-doc, x-mem-readme-claim, x-mem-claude-narrative]
  affects: [docs/adr/, docs/hp41-xmem-divergences.md, README.md, CLAUDE.md, docs/architecture-history.md]
tech_stack:
  added: []
  patterns: [per-decision-adr, five-field-divergences-template, scoped-readme-claim, architecture-history-narrative]
key_files:
  created:
    - docs/adr/v4.0-001-xmem-os-builtin.md
    - docs/adr/v4.0-002-xmem-capacity.md
    - docs/adr/v4.0-003-xmem-register-transfer.md
    - docs/hp41-xmem-divergences.md
  modified:
    - README.md
    - CLAUDE.md
    - docs/architecture-history.md
decisions:
  - "X-MEM routes via builtin_card_op, no XROM bit (ADR-v4.0-001, D-52.7)"
  - "Fixed 600-register X-MEM capacity (ADR-v4.0-002, D-51.1)"
  - "Full-register-set SAVED/GETD via tested cardreader helpers (ADR-v4.0-003, D-51.5)"
  - "README X-MEM claim scoped/honest — PROGRAM + DATA only, no feature-complete (D-52.6)"
metrics:
  duration: "~15 minutes"
  completed: "2026-05-28"
  tasks_completed: 3
  tasks_total: 3
  files_created: 4
  files_modified: 3
---

# Phase 52 Plan 04: X-MEM Documentation Suite Summary

Three granular per-decision ADRs, a dedicated X-MEM divergences catalog, and the standard CLAUDE.md + README + architecture-history follow-through capture the architecturally significant Phase-51 X-MEM choices before they fade.

## What Was Built

### Task 1: Three Granular X-MEM ADRs (v4.0-001/002/003)

Three long-form ADRs following the v3.3-001 template (Status/Owner/Requirement refs/Downstream consumer / Context / Decision / Consequences / Alternatives Considered / Footnotes):

- **`docs/adr/v4.0-001-xmem-os-builtin.md`** — X-MEM as HP-41CX OS Built-ins; why `builtin_card_op` (not `xrom_resolve`); no XROM bit allocated; 4 alternatives considered (XROM bit-5, independent fast-path, fold into hp41cv-functions.json).
- **`docs/adr/v4.0-002-xmem-capacity.md`** — Fixed 600-register capacity (124 + 2×238); why `EMROOM` must be meaningful; 3 alternatives considered (124 minimal, unlimited, user-configurable).
- **`docs/adr/v4.0-003-xmem-register-transfer.md`** — Full-register-set SAVED/GETD via `capture_data_card`/`load_data_card`; bbb.eee block-control-word deferred; future upgrade path documented.

### Task 2: docs/hp41-xmem-divergences.md

Dedicated divergences catalog following the `docs/hp41-time-divergences.md` five-field template (`D-52-NN` entry IDs):

- **D-52-01:** Overwrite-on-duplicate — real HP-41CX raises "DUP FL" + requires PURFL; emulator silently overwrites (Phase 51 op set has no purge op). Cross-references D-51.6 and ADR-v4.0-001.
- **D-52-02:** Full-register-set SAVED/GETD vs. hardware `bbb.eee` block control word. Cross-references D-51.5, ADR-v4.0-003, and cardreader helpers.
- Sections 2 (Emulator Extensions) and 3 (Behavioral Policies) are intentionally empty for Phase 52 scope — future entries numbered D-52-03+.

### Task 3: README + CLAUDE.md + architecture-history.md

- **README.md:** Scoped/honest X-MEM claim (D-52.6): "v4.0 adds Extended Memory: named PROGRAM + DATA file storage (HP-41CX X-Functions) — 8 XEQ-by-name functions; 600-register capacity; documented divergences". No "feature-complete" wording — ASCII/STATUS deferred to v4.1.
- **CLAUDE.md:** New "X-MEM Built-in Integration (v4.0)" section under the XROM Module Registry. Not added to the XROM table (X-MEM is not an XROM). Covers: 8 ops, state isolation, capacity, SAVED/GETD, help pool, divergences doc, ADR cross-refs, save-file compat.
- **docs/architecture-history.md:** New `## v4.0 additions` section with Phase 51 (X-MEM Core) and Phase 52 (Test Hardening + Documentation) narratives, following the per-milestone history pattern. Cross-references all three ADRs and the divergences doc. Includes frozen-invariants-preserved summary matching the v3.x milestone pattern.

## Commits

| Task | Commit | Files |
|------|--------|-------|
| Task 1: 3 ADRs | 1606016 | docs/adr/v4.0-001/002/003 |
| Task 2: divergences doc | 0b2ebcc | docs/hp41-xmem-divergences.md |
| Task 3: README + CLAUDE.md + arch-history | 8aea293 | README.md, CLAUDE.md, docs/architecture-history.md |

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. All documentation is complete for the Phase 52 scope. The deferred items (ASCII/STATUS file types — XMEM-F01/F02; `bbb.eee` block-control-word SAVED/GETD; PURFL/CLFL purge op) are documented as explicit deferrals in the ADRs and divergences doc, not as stubs.

## Threat Flags

None. This plan writes Markdown documentation only — no code, no network, no auth, no runtime input. The only security-adjacent concern (D-52.6 README claim accuracy) is addressed: the X-MEM claim uses scoped/honest wording ("named PROGRAM + DATA file storage") with no "feature-complete" assertion.

## Self-Check: PASSED

Files exist:
- `docs/adr/v4.0-001-xmem-os-builtin.md` — FOUND
- `docs/adr/v4.0-002-xmem-capacity.md` — FOUND
- `docs/adr/v4.0-003-xmem-register-transfer.md` — FOUND
- `docs/hp41-xmem-divergences.md` — FOUND
- `README.md` (modified) — FOUND
- `CLAUDE.md` (modified) — FOUND
- `docs/architecture-history.md` (modified) — FOUND

Commits exist:
- `1606016` — FOUND (docs/adr/v4.0-001/002/003)
- `0b2ebcc` — FOUND (docs/hp41-xmem-divergences.md)
- `8aea293` — FOUND (README.md, CLAUDE.md, docs/architecture-history.md)

Content checks:
- `grep -q "ADR-v4.0-001" docs/adr/v4.0-001-xmem-os-builtin.md` — PASSED
- `grep -q "600" docs/adr/v4.0-002-xmem-capacity.md` — PASSED
- `grep -q "SAVED" docs/adr/v4.0-003-xmem-register-transfer.md` — PASSED
- `grep -q "DUP FL" docs/hp41-xmem-divergences.md` — PASSED
- `grep -qi "bbb.eee" docs/hp41-xmem-divergences.md` — PASSED
- `grep -qi "Extended Memory" README.md` — PASSED
- X-MEM README line has NO "feature-complete" wording — PASSED
- `grep -qi "X-MEM" CLAUDE.md` — PASSED
- `grep -qi "X-MEM" docs/architecture-history.md` — PASSED
