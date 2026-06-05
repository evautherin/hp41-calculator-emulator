---
phase: 61-quality-gates
plan: 01
subsystem: help-search
tags: [help-search, parity-fixture, search-aliases, data-only, quality-gates]
requires:
  - help_data module + search_aliases pools (Phase 58/59/60, develop-only)
provides:
  - TVM exact aliases: "Zinseszins", "Zinseszinsrechnung", "compound interest"
  - SQRT exact alias: "Wurzel"
  - docs/fixtures/help_search_parity.json (CLI<->GUI parity fixture, 8 canonical queries)
affects:
  - hp41-cli/tests/phase61_help_search_aliases.rs (plan 61-02, downstream consumer)
  - hp41-gui/src/help_data.test.ts (plan 61-03, downstream consumer)
tech-stack:
  added: []
  patterns:
    - byte-preserving JSON alias splice (Phase-60 precedent)
    - dual-consumer JSON fixture (Rust include_str! + TS static import)
key-files:
  created:
    - docs/fixtures/help_search_parity.json
  modified:
    - docs/hp41-advantage-functions.json
    - docs/hp41cv-functions.json
decisions:
  - "Fixture schema: object { _comment, queries: [{query, top_n, expected_top[], note}] } matching 61-RESEARCH Pattern 2"
  - "Parity fixture uses only top_n=1, stable-top-1 queries; spec-example aliases (wurzel/zinseszins/compound interest) deliberately excluded — asserted in per-frontend unit tests (61-02/03)"
metrics:
  tasks: 2
  files: 3
  completed: 2026-06-05
requirements: [HSQUAL-01, HSQUAL-02]
---

# Phase 61 Plan 01: Help-Search Parity Aliases + Fixture Summary

One-liner: Hand-edited exact `search_aliases` (TVM gains Zinseszins/Zinseszinsrechnung/compound interest; SQRT gains bare "Wurzel") and added the dual-consumer `help_search_parity.json` fixture mapping 8 stable canonical queries to expected top-1 display_names — pure data, no LLM/generator/source change.

## What Was Done

### Task 1 — Hand-edit TVM + SQRT search_aliases (data-only, no-LLM)
- Appended `"Zinseszins"`, `"Zinseszinsrechnung"`, `"compound interest"` to the TVM entry's `search_aliases` in `docs/hp41-advantage-functions.json` (existing 8 aliases untouched and unreordered).
- Appended bare `"Wurzel"` to the SQRT entry's `search_aliases` in `docs/hp41cv-functions.json` (existing 6 aliases untouched and unreordered).
- Byte-preserving splice: no reformat, no re-key, no other entry touched, UTF-8 umlaut bytes in neighbouring aliases (e.g. "Finanzmathematik Löser") left intact. No `just help-aliases`, no LLM/generator, no file under `scripts/`, `hp41-core/`, `hp41-cli/`, `hp41-gui/`.

### Task 2 — Create CLI<->GUI parity fixture
- Created `docs/fixtures/` and `docs/fixtures/help_search_parity.json`.
- Schema (matches 61-RESEARCH Pattern 2, consumed by both downstream tests):
  ```json
  {
    "_comment": "... mentions Rust + TS consumers and score-DESC / display_name-ASC tie-break ...",
    "queries": [
      { "query": "<lowercased>", "top_n": 1, "expected_top": ["<display_name>"], "note": "<why>" }
    ]
  }
  ```
- 8 stable, unambiguous top-1 queries (from the RESEARCH "Verified canonical queries" table). The spec-example queries (`wurzel`, `zinseszins`, `compound interest`, `zineszins`) are deliberately NOT in the parity fixture — they are asserted in the per-frontend unit tests (61-02/61-03).

## Fixture Schema (for downstream plans 61-02 / 61-03)

Top-level object with `_comment` (string) and `queries` (array). Each query record:

| field | type | meaning |
|-------|------|---------|
| `query` | string | already-lowercased search input |
| `top_n` | integer | number of leading results asserted (always 1 here) |
| `expected_top` | array of strings | expected `display_name`(s), index-aligned to ranked results |
| `note` | string | rationale |

Consumers:
- Rust: `include_str!("../../../docs/fixtures/help_search_parity.json")` from `hp41-cli/tests/`, parse with `serde_json`, compare `results.get(i).op` to `expected_top[i]`.
- TS: `import parityFixture from '../../docs/fixtures/help_search_parity.json'`, compare `results[i].display_name` to `expected_top[i]`.

Tie-break (documented in `_comment`, load-bearing): results sort score DESC, then `display_name` ASC.

## Query -> expected_top mappings (all 8)

| query | expected_top | tier |
|-------|--------------|------|
| `tvm` | TVM | exact name (40) |
| `pi` | PI | exact name (40) — baseline regression |
| `emdir` | EMDIR | exact name (40), xmem pool |
| `zeitwert des geldes` | TVM | exact DE alias (35) |
| `quadratwurzel` | SQRT | exact DE alias (35) |
| `square root` | SQRT | exact EN alias (35) |
| `financial solver` | TVM | exact EN alias (35) |
| `sqirt` | SQRT | fuzzy, 1-typo on name (8) |

## Correctness Verification

Independently scored the hand-edited aliases against all live pools (exact-name=40, exact-alias=35, word-prefix=28 approximation) to confirm the downstream 61-02/03 unit-test expectations now hold:
- `zinseszins` -> TVM (35, exact alias, top-1)
- `compound interest` -> TVM (35, exact alias, top-1)
- `wurzel` -> SQRT (35 exact alias) now uniquely beats the 28-scoring word-prefix competitors (RTS, Z^1/N, FSOLVE...) -> SQRT top-1.

jq gates (all pass):
- TVM aliases include "Zinseszins" and "compound interest"; SQRT aliases include "Wurzel".
- All three JSON files parse (`jq -e '.'` exits 0).
- `.queries` is an 8-element array; every element has `query`/`top_n`/`expected_top` (array).
- Forbidden queries (`wurzel`, `zinseszins`, `compound interest`, `zineszins`) absent from the fixture.
- `git diff` touches only the two pools + the new fixture; no source/`scripts/` files.

## Deviations from Plan

None — plan executed exactly as written. Data-only edits, byte-clean diff, no source changes.

## Self-Check: PASSED
- docs/hp41-advantage-functions.json — modified, parses, TVM aliases present.
- docs/hp41cv-functions.json — modified, parses, SQRT "Wurzel" present.
- docs/fixtures/help_search_parity.json — created, parses, 8 well-formed queries.
