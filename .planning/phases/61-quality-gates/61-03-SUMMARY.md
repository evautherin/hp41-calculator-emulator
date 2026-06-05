---
phase: 61-quality-gates
plan: 03
subsystem: hp41-gui (help search)
tags: [testing, vitest, help-search, cli-gui-parity, quality-gates]
requires: ["61-01 (docs/fixtures/help_search_parity.json)"]
provides: ["GUI half of CLI↔GUI top-1 drift guard (HSQUAL-02)", "real-data tiered-scorer quality gates (HSQUAL-01)"]
affects: ["hp41-gui/src/help_data.test.ts"]
tech-stack:
  added: []
  patterns: ["static JSON import of shared fixture (resolveJsonModule)", "table-driven Vitest it()-per-fixture-query loop"]
key-files:
  created: []
  modified: ["hp41-gui/src/help_data.test.ts"]
decisions:
  - "Assert empty-query passthrough as strict reference identity (results === pool) plus element-order equality — the TS contract returns the pool unchanged for '' (HSMATCH-04), distinct from the Rust whitespace handling."
  - "Parity loop emits one it() per fixture query so a single divergent top-1 names the offending query, not the whole block."
metrics:
  duration: ~6 min
  completed: 2026-06-05
---

# Phase 61 Plan 03: GUI Help-Search Quality Gates Summary

Added real-data quality gates and the GUI half of the CLI↔GUI top-1 drift guard to `hp41-gui/src/help_data.test.ts`, exercising the live six-pool tiered scorer via `rankedEntries(allFunctionsEntries(), q)` and asserting parity against the canonical `docs/fixtures/help_search_parity.json` fixture (the same fixture the Rust suite in 61-02 asserts).

## What was built

One additive change to the single permitted file, `hp41-gui/src/help_data.test.ts` (+126 lines, 0 deletions — existing tests/imports untouched):

1. **Static fixture import** — `import parityFixture from '../../docs/fixtures/help_search_parity.json'` (relies on `resolveJsonModule: true`, already enabled in `tsconfig.json`).
2. **`describe('Phase 61 — help search quality gates (real six-pool data)')`** — 8 `it()` cases:
   - All four scoring tiers on live data: exact name (`tvm`→40), word-prefix (`sqr`→32), substring (`vm` in `TVM`→24), fuzzy (`sqirt`→8), each asserted via `scoreEntry` against the live pool entry.
   - DE+EN alias top-1 → TVM: `zeitwert des geldes` / `zinseszins` / `compound interest`.
   - DE+EN alias top-1 → SQRT: `square root` / `quadratwurzel` / `wurzel`.
   - Fuzzy top-1: `sqirt` → SQRT on the live pool.
   - Empty-query passthrough (HSMATCH-04): `rankedEntries(pool, '') === pool` (reference identity + element-order check).
3. **`describe('Phase 61 — CLI↔GUI parity fixture (top-1 drift guard, HSQUAL-02)')`** — a fixture-shape assertion (every entry `top_n === 1`, `query` already lowercased) plus one `it()` per fixture query asserting `rankedEntries(allFunctionsEntries(), query)[0].display_name === expected_top[0]`.

## Exported symbols / fields used

- Functions: `rankedEntries(pool, q)`, `scoreEntry(entry, q)`, `allFunctionsEntries()` — all already exported from `hp41-gui/src/help_data.ts` and already imported by the test file.
- Result display-name field: **`display_name`** (on `HelpEntry`). `rankedEntries` returns `readonly HelpEntry[]`, so top-1 is `results[0].display_name`.
- Empty-query contract: `rankedEntries` returns the input `pool` reference unchanged when `q === ''` (verified in source, lines ~456-457).

## Data verification (pre-write)

Confirmed 61-01's frozen JSON output contains the required aliases before writing assertions:
- TVM (`docs/hp41-advantage-functions.json`): includes `time value of money`, `financial solver`, `Zeitwert des Geldes`, `Zinseszins`, `compound interest`.
- SQRT (`docs/hp41cv-functions.json`): includes `Quadratwurzel`, `square root`, `Wurzel`. (`wurzel` resolves to the exact-alias `"Wurzel"` after the scorer's lowercasing.)

## Test run

`cd hp41-gui && npm test -- help_data` (vitest run mode, non-watch): **GREEN — Test Files 1 passed, Tests 36 passed (36)** (was 24 before this plan; +12 new cases). Re-run after commit also GREEN. `npx tsc --noEmit` passes (exit 0).

## Deviations from Plan

None to the test content. **One execution-environment note (not a plan deviation):** a parallel sibling agent (61-02) committed concurrently and momentarily reset the shared git index between my `git add` and my first path-scoped `git commit`, which reported "no changes". My working-tree changes were never lost; I re-verified HEAD line count (314 without my block) vs working tree (440 with it), re-staged the file alone, and committed via pathspec (`git commit -- hp41-gui/src/help_data.test.ts`). The atomic, file-scoped commit is `67d9033`. No `git add -A` was used; no other file was included in my commit.

## Self-Check: PASSED

- `hp41-gui/src/help_data.test.ts` present and committed (440 lines, includes `parityFixture` import + both Phase 61 describe blocks).
- Commit `67d9033` ("✅ test(61-03): GUI help-search quality gates + CLI↔GUI parity fixture loop") exists in history and last-touched the file.
- `git diff HEAD -- hp41-gui/src/help_data.test.ts` clean after commit.
