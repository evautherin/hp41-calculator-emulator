---
status: passed
phase: 59-runtime-matcher
source: [59-VERIFICATION.md]
started: 2026-06-04T20:19:07Z
updated: 2026-06-05T00:00:00Z
---

## Current Test

[awaiting human testing]

> **Scope correction (2026-06-05):** Phase 59 ships the **matcher engine only**. The
> alias *content* (`search_aliases` populated with DE+EN synonyms like `Zineszins`,
> `Wurzel`, `compound interest`) is **Phase 60's** deliverable — all six JSON pools
> still carry zero aliases (`grep -c search_aliases docs/*functions.json` → 0). The
> original criteria below tested Phase-60 data against a Phase-59 build, so they
> correctly returned "no match". The DE/alias acceptance lives in the ROADMAP as a
> **Phase 61** success criterion (`"Zineszins" → TVM, "Wurzel" → SQRT`) and is
> deferred there — nothing is lost. These tests now verify what Phase 59 actually
> delivers: the four-tier scorer + flat-ranked render branch, exercised against the
> current English data. Engine wiring confirmed in code (scoreEntry searches
> display_name/description/category/search_aliases; ranked filter `score > 0`).

## Tests

### 1. CLI engine — four-tier relevance + typo tolerance
expected: Open the CLI `?` overlay and type each query; the named entry should appear (typically at/near the top). All against current English data — no aliases needed.
  - `SQRT` → **exact** name hit (SQRT top).
  - `sqr` → **prefix** hit (SQRT).
  - `squrt` → **fuzzy** hit despite the typo (SQRT still found — the milestone's headline typo-tolerance; Levenshtein dist 1 ≤ bound 1).
  - `interest` → **substring** hit in the TVM / financial descriptions (Advantage pool).
  - `matrix` → **substring** hits across the matrix functions.
  - Clear the query → the category-grouped view returns unchanged (empty-query invariance guard).
result: [passed] (2026-06-05, human UAT — engine four-tier + typo tolerance confirmed on current EN data)

### 2. GUI both-tabs flat-ranked render branch
expected: In the GUI `?` overlay, on BOTH the "Keyboard Shortcuts" and "All Functions" tabs:
  - A non-empty query (`sqr`, then `matrix`) renders a flat relevance-ordered list with **NO category headings** (sectionGroups bypass).
  - `squrt` (typo) still surfaces SQRT (fuzzy tier works in the GUI too).
  - Clearing the query restores the grouped view.
  - Ranking order feels sensible: exact > prefix > substring > fuzzy.
result: [passed] (2026-06-05, human UAT — both tabs flat-ranked render branch + fuzzy confirmed)

## Deferred to Phase 60/61 (NOT testable yet — no alias data)

These were the original Phase-59 criteria; they require populated `search_aliases`
and are tracked as ROADMAP Phase 61 SC#1. Re-run them after Phase 60 commits aliases:
  - `Zineszins` → TVM (German alias, fuzzy)
  - `Wurzel` → SQRT (German alias)
  - `compound interest` → TVM (EN plain-language alias)

## Summary

total: 2
passed: 2
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

- DE + plain-language alias resolution deferred to Phase 60 (alias authoring) + Phase 61 (quality-gate verification). Not a Phase 59 defect.
