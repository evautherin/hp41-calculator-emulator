---
phase: 61-quality-gates
plan: 04
subsystem: ci-quality-gates
tags: [ci, schema, help-search, search_aliases, jq]
requirements: [HSQUAL-03]
requires: [61-01]
provides: ["search_aliases six-pool schema gate (script + just recipe + ci.yml job)"]
affects: [".github/workflows/ci.yml", "Justfile"]
tech-stack:
  added: []
  patterns: ["bash+jq CI gate mirroring check-free42-contamination.sh", "hardcoded six-pool enumeration (never globbed)"]
key-files:
  created: ["scripts/check-aliases-schema.sh"]
  modified: ["Justfile", ".github/workflows/ci.yml"]
decisions:
  - "Six pools hardcoded in a POOLS array; never a docs/hp41-*-functions.json glob (would miss hp41cv-functions.json, no dash before 'functions')."
  - "Gate placed in ci.yml (no path filter) NOT ci-gui.yml (paths: filter excludes docs/*.json)."
  - "Bite proof shipped as a --self-test flag mutating only a temp copy; real pools never touched."
metrics:
  duration: ~12m
  completed: 2026-06-05
  tasks: 1
  files: 3
---

# Phase 61 Plan 04: search_aliases Schema Gate Summary

A bash+jq CI gate (`scripts/check-aliases-schema.sh`) validates the `search_aliases` field across all six help-data JSON pools, wired into both `just schema-aliases-check` and a parallel `schema-aliases` job in `ci.yml`.

## What was built

- **`scripts/check-aliases-schema.sh`** — strict-mode (`set -euo pipefail`) bash+jq validator mirroring `check-free42-contamination.sh`. The six pools are enumerated explicitly in a `POOLS` array (LANDMINE: `hp41cv-functions.json` has no dash before "functions", so any `docs/hp41-*-functions.json` glob would silently miss it — all six paths are hardcoded). For each pool it asserts:
  1. Where `search_aliases` is present, it is an array of strings.
  2. Every entry with `status == "implemented"` carries `search_aliases` with length >= 1.
  Any violation prints `pool: display_name — reason` and the script exits non-zero. Missing pool file or invalid JSON exits 2; missing `jq` exits 2.
- **`--self-test` flag** — proves the gate bites by copying the first pool to a temp file (`mktemp`, `trap rm`), emptying the first implemented entry's `search_aliases`, and asserting `validate_pool` rejects it. Only the temp copy is mutated; the real six pools are never touched.
- **`Justfile`** — `schema-aliases-check` recipe (`[group('ci')]`) wrapping the script, mirroring `license-audit`.
- **`.github/workflows/ci.yml`** — new `schema-aliases` job: checkout → install `just` (taiki-e pinned action) → `just schema-aliases-check`. No `needs:` (runs in parallel). The workflow has no `paths:` filter, so docs/*.json edits trigger it.

## Verification performed

- `bash scripts/check-aliases-schema.sh` → `OK: search_aliases schema valid across all 6 pools.` exit 0.
- `bash scripts/check-aliases-schema.sh --self-test` → reports the temp pool's `BININ` entry as missing/empty and prints `SELF-TEST OK ... (gate bites)`, exit 0.
- `just schema-aliases-check` → green, exit 0.
- `python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/ci.yml"))'` → `ci.yml YAML OK`.
- Pre-flight jq audit confirmed all six current pools already satisfy the schema (0 missing implemented aliases, 0 bad types) — gate passes on the real tree.

## ci.yml job placement (LANDMINE confirmed)

The job is in **`.github/workflows/ci.yml`** (NOT `ci-gui.yml`), matching the `license-audit` precedent. `ci.yml` has **no `paths:` filter**, so docs/*.json-only edits trigger the job. `ci-gui.yml`'s `paths:` filter excludes `docs/*.json`, which is exactly why the gate must NOT live there. The job has **no `needs:`** so it runs in parallel and shows independently in the PR checks panel.

## Deviations from Plan

### Parallel-agent index contamination (process, not code)

During the first `git commit`, the index had been populated by sibling parallel agents (other 61-xx plans running concurrently on the same `develop` worktree): the commit swept in `hp41-cli/tests/phase61_help_search_aliases.rs` and `hp41-gui/src/help_data.test.ts`, neither of which belongs to this plan. Recovered via `git reset --soft HEAD~1` → `git restore --staged` of the two foreign files (left intact on disk for their owning agents) → re-commit with only my three files. Final commit `ed243bf` contains exactly `scripts/check-aliases-schema.sh`, `Justfile`, `.github/workflows/ci.yml` (133 insertions, 3 files). No docs/*.json staged or modified. No sibling work destroyed.

## Byte-unchanged confirmation

The six help-data pools (`docs/hp41-advantage-functions.json`, `docs/hp41-math1-functions.json`, `docs/hp41-stat1-functions.json`, `docs/hp41-time-functions.json`, `docs/hp41-xmem-functions.json`, `docs/hp41cv-functions.json`) are byte-identical to their pre-execution state — `git status --short docs/` is empty. The `--self-test` mutation happened only on an `mktemp` copy.

## Self-Check: PASSED

- `scripts/check-aliases-schema.sh` — FOUND
- Commit `ed243bf` — FOUND in git log
- `Justfile` `schema-aliases-check` recipe — present
- `ci.yml` `schema-aliases` job — present, no `needs:`, no excluding path filter
