---
phase: 59-runtime-matcher
plan: "01"
subsystem: help-search
tags: [tdd, red-wave-0, scorer-contract, rust, typescript]
dependency_graph:
  requires: [58-data-model]
  provides: [59-02-cli-scorer, 59-03-gui-scorer]
  affects: [hp41-cli/tests, hp41-gui/src]
tech_stack:
  added: []
  patterns: [tdd-wave-0, red-green-refactor, cli-gui-mirror-discipline]
key_files:
  created:
    - hp41-cli/tests/phase59_help_search.rs
  modified:
    - hp41-gui/src/help_data.test.ts
    - hp41-gui/src/HelpOverlay.test.tsx
decisions:
  - "RED Wave 0 completed: three test files encode the full scorer contract before any implementation"
  - "Tier constants locked: name exact 40 > prefix 32 > substr 24 > fuzzy 8; alias exact 35"
  - "score_entry takes a pre-lowercased query q; ranked_help_entries/rankedEntries return flat list score DESC/name ASC"
  - "HelpOverlay render-branch contract: empty query → grouped; non-empty → flat (zero headings)"
metrics:
  duration: "~5 minutes"
  completed: "2026-06-04"
  tasks_completed: 3
  tasks_total: 3
  files_created: 1
  files_modified: 2
---

# Phase 59 Plan 01: RED Scorer Contract Summary

Wave-0 TDD plan: three RED test files encoding the full scorer API (score_entry / ranked_help_entries in Rust; scoreEntry / rankedEntries in TS; flat-ranked render branch in HelpOverlay) before any implementation exists.

---

## One-Liner

Three RED test files lock the scorer contract — tier order, fuzzy typo (Zineszins→TVM, Wurzel→SQRT), DE+EN alias resolution, empty-query invariance, and flat-ranked render branch — before scorer code is written.

---

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | RED Rust integration test phase59_help_search.rs | 2431812 | hp41-cli/tests/phase59_help_search.rs (created) |
| 2 | RED TS scorer tests in help_data.test.ts | e0e7cda | hp41-gui/src/help_data.test.ts (extended) |
| 3 | RED HelpOverlay render-branch tests | 2b9e5b6 | hp41-gui/src/HelpOverlay.test.tsx (extended) |

---

## RED State (Intended — Wave-0)

This plan intentionally leaves all three test suites in a failing state. The RED state is the correct output; it is NOT a defect.

### Rust (hp41-cli/tests/phase59_help_search.rs)

**Failure mode:** Compile error — unresolved imports

```
error[E0432]: unresolved imports `hp41_cli::help_data::ranked_help_entries`, `hp41_cli::help_data::score_entry`
```

**Why RED:** `score_entry` and `ranked_help_entries` do not yet exist in `hp41-cli/src/help_data.rs`. Plan 59-02 adds them.

**7 test functions** (all RED until 59-02):
- `tier_order_exact_prefix_substring_fuzzy` (HSMATCH-02)
- `fuzzy_typo_zineszins_resolves_tvm` (HSMATCH-03, HSUX-02)
- `fuzzy_wurzel_resolves_sqrt` (HSUX-02)
- `alias_de_en_both_resolve` (HSMATCH-05)
- `empty_query_returns_all_grouped` (HSMATCH-04) — regression guard
- `score_over_all_four_fields` (HSMATCH-01)
- `ranked_entries_sorted_desc_no_headers` (HSMATCH-02)

### TypeScript (hp41-gui/src/help_data.test.ts)

**Failure mode:** 8 tests fail with `TypeError: scoreEntry is not a function` / `rankedEntries is not a function`

```
Tests  8 failed | 11 passed (19)
```

**Why RED:** `scoreEntry` and `rankedEntries` are imported but not yet exported from `help_data.ts`. Plan 59-03 adds them.

**New describe blocks:**
- `Phase 59 scorer — scoreEntry`: 5 tests (tier order, fuzzy typos, alias DE+EN, all-four-fields)
- `Phase 59 ranking — rankedEntries`: 3 tests (sorted output, tie-break, empty passthrough)

**11 existing tests remain GREEN** (backward-compat guardrails unchanged).

### TypeScript (hp41-gui/src/HelpOverlay.test.tsx)

**Failure mode:** 2 tests fail because the flat-ranked branch does not yet exist in HelpOverlay.tsx

```
Tests  2 failed | 56 passed (58)
```

**Why RED:** HelpOverlay.tsx still renders the grouped view for non-empty queries. Plan 59-03 adds the flat-ranked branch.

**New describe block:** `Phase 59 — flat-ranked render branch` with 4 tests:
- `empty query: section-grouped view` — PASSES (regression guard, must stay GREEN after 59-03)
- `non-empty query: flat-ranked view` — FAILS (headings.length is 6, expected 0) → RED
- `All Functions tab with non-empty query` — FAILS (headings.length is 3, expected 0) → RED
- `no-match query: empty state` — PASSES (sanity guard)

**56 existing tests remain GREEN** (no regressions introduced).

---

## Deviations from Plan

None — plan executed exactly as written. All three files reference scorer symbols that do not exist; no production source modified.

---

## Decisions Made

1. **HelpEntry struct fields** — `make_entry` in the Rust test constructs all 10 fields verbatim to match the struct definition (no missing fields, no extra fields).

2. **7th test added** — The plan specified >= 6 tests; an additional integration test `ranked_entries_sorted_desc_no_headers` was added that exercises `ranked_help_entries` against real JSON pools to verify ordering and absence of header rows. This is a strictly additive improvement.

3. **rankedEntries empty-passthrough test** — The TS test for `rankedEntries('', pool)` encodes the contract that callers pass an empty-string to get the full pool back (mirrors the Rust `filter_help_rows` empty guard). This is a behavior the 59-03 implementor must honor.

4. **HelpOverlay afterEach + cleanup** — Added to the file's import (not just the new block) since there is only one existing `afterEach`-like callsite needed and the portal hygiene applies to all renders in the file.

---

## Known Stubs

None. This plan creates/modifies test files only; no production code stubs were introduced.

---

## Threat Flags

None. Test-only plan; no new runtime surface, no I/O, no auth paths.

---

## Self-Check: PASSED

- hp41-cli/tests/phase59_help_search.rs: EXISTS, 7 `#[test]` functions, `fn make_entry` present, scorer not defined
- hp41-gui/src/help_data.test.ts: scoreEntry + rankedEntries imported (RED), 8 new tests
- hp41-gui/src/HelpOverlay.test.tsx: Phase 59 describe block, afterEach(cleanup), `.help-overlay-category-heading` + `.help-overlay-row` assertions
- Commits: 2431812, e0e7cda, 2b9e5b6 — all exist on develop
- No production source modified (checked: zero diff on hp41-cli/src/**, hp41-gui/src/help_data.ts, hp41-gui/src/HelpOverlay.tsx)
