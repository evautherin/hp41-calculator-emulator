---
phase: 59-runtime-matcher
plan: "03"
subsystem: help-search-gui
tags: [tdd, green-wave-2, scorer, typescript, gui]
dependency_graph:
  requires: [59-01, 59-02, 58-data-model]
  provides: [gui-scorer, ranked-help-entries-ts, flat-ranked-render-gui]
  affects: [hp41-gui/src/help_data.ts, hp41-gui/src/HelpOverlay.tsx]
tech_stack:
  added: []
  patterns: [tiered-scoring, bounded-levenshtein, flat-ranked-render-branch, cli-gui-mirror-discipline]
key_files:
  modified:
    - hp41-gui/src/help_data.ts
    - hp41-gui/src/HelpOverlay.tsx
decisions:
  - "levenshteinBounded uses Unicode spread [...str] for umlaut-correct char iteration, mirroring Rust chars().collect()"
  - "tierScore: fuzzy gated on q.length >= 2 (P-HS-02 guard, mirrors Rust q.len() >= 2)"
  - "rankedEntries returns readonly HelpEntry[] (no HelpRow projection); HelpOverlay renders directly from entries"
  - "flat-ranked branch: query.trim() !== '' check gates both tabs; empty query preserves sectionGroups path bit-for-bit"
  - "pre-existing tsc TS2554 errors in test file (59-01 plan) are out of scope — not caused by 59-03 changes"
metrics:
  duration: "~12 minutes"
  completed: "2026-06-04"
  tasks_completed: 2
  tasks_total: 2
  files_created: 0
  files_modified: 2
---

# Phase 59 Plan 03: GUI Scorer GREEN Summary

Wave-2 GREEN plan: implements `levenshteinBounded`, `tierScore`, `scoreEntry`, and `rankedEntries` in `hp41-gui/src/help_data.ts`, then wires `HelpOverlay.tsx` to branch on query empty vs non-empty for both the Keyboard Shortcuts and All Functions tabs.

---

## One-Liner

Hand-rolled tiered scorer (40/32/24/8 name, 35/28/21/7 alias tiers) + bounded Levenshtein in help_data.ts; flat-ranked render branch in HelpOverlay.tsx for both tabs — all 10 RED TypeScript tests from plan 59-01 are now GREEN with zero regressions.

---

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | levenshteinBounded, tierScore, scoreEntry, rankedEntries in help_data.ts | 9e15ad5 | hp41-gui/src/help_data.ts (modified) |
| 2 | Flat-ranked render branch in HelpOverlay.tsx (both tabs) | 2a05697 | hp41-gui/src/HelpOverlay.tsx (modified) |

---

## Verification

- `cd hp41-gui && npm test`: **302/302 PASS** (0 failures, 0 regressions)
- `help_data.test.ts`: 19 tests pass (10 new Phase 59 scorer + 9 pre-existing)
- `HelpOverlay.test.tsx`: 56 tests pass (4 new Phase 59 render-branch + 52 pre-existing)
- `tsc --noEmit`: 6 pre-existing TS2554 errors in 59-01's test file (not caused by 59-03)

---

## Implementation Details

### Task 1 — help_data.ts scorer

Added ~126 lines immediately before the `helpEntriesAll()` section:

1. **Tier constants** — 16 `const` values (SCORE_EXACT_NAME=40 … SCORE_FUZZY_CAT=4), mirroring the Rust `u8` consts in `help_data.rs`.

2. **`levenshteinBounded(a, b, maxDist) -> number`** (exported) — classic DP with `[...str]` spread for Unicode-correct umlaut handling. Early-exit: if `Math.min(...curr) > maxDist`, returns `maxDist + 1` immediately. No raw `.unwrap()` equivalent needed — TypeScript handles `Math.min(...curr)` safely.

3. **`tierScore(field, q, exact, prefix, substr, fuzzy) -> number`** (private) — lowercases field; checks exact `===` / word-prefix (`startsWith` or any `split(/\s+/)` token) / substring (`includes`) / fuzzy (only when `q.length >= 2`, threshold = `max(1, floor(q.length/4))`, whole-field for len <= 20 else min over words).

4. **`scoreEntry(entry: HelpEntry, q: string) -> number`** (exported) — max over all four field scores. Alias score uses `(entry.search_aliases ?? []).map(...).reduce(Math.max, 0)` — the `?? []` guard is required for the optional TS field.

5. **`rankedEntries(pool, q) -> readonly HelpEntry[]`** (exported) — empty-query passthrough; non-empty: filter score > 0, sort by (score DESC, display_name ASC via `localeCompare`), return entries.

6. **Provenance comment** — `// Mirror of hp41-cli/src/help_data.rs scorer (plan 59-02). Keep byte-equivalent in BEHAVIOR with the Rust implementation per Phase 59 (CLI<->GUI parity).`

### Task 2 — HelpOverlay.tsx render branch

1. **Import** — added `rankedEntries` to the import from `./help_data`.

2. **`filtered` memo** — replaced substring filter with `rankedEntries(allEntries, q)` for non-empty query; empty-query path (`return allEntries`) preserved unchanged.

3. **`filteredAllFn` memo** — same upgrade pattern applied to All Functions tab entries.

4. **Keyboard Shortcuts tab render** — wrapped `sectionGroups.map(...)` in a ternary: `query.trim() !== ''` → flat ranked list (direct entry render, no `.help-overlay-category-heading`); `else` → existing `sectionGroups.map(...)` grouped path unchanged.

5. **All Functions tab render** — same ternary pattern: `query.trim() !== ''` → flat ranked list rendering tappable/non-tappable rows exactly as in the grouped path (same CSS classes, same `xeqToken` check, same `onRun` dispatch); `else` → existing `allFnSectionGroups.map(...)` grouped path unchanged.

6. **No new CSS classes introduced** — the flat-ranked list uses the same `.help-overlay-row`, `.help-fn-run-btn`, `.help-overlay-row--non-tappable`, etc. classes that pre-existing tests query on.

---

## Deviations from Plan

### Pre-existing TypeScript errors (out-of-scope, not caused by 59-03)

The test file `HelpOverlay.test.tsx` written in plan 59-01 (RED wave) uses `expect(x).toBe(y, message)` with a second argument. TypeScript reports TS2554 "Expected 1 arguments, but got 2" for 6 call sites. These errors were present before 59-03 work started (confirmed by checking tsc on the pre-59-03 HEAD). Vitest runs and passes all 302 tests regardless. Scope boundary: not caused by 59-03 changes, deferred to a future cleanup.

---

## Known Stubs

None. The scorer is fully wired. `search_aliases` fields are empty on all entries until Phase 60 populates them — alias-tier scores will be 0 for all real entries, which is correct behavior, not a stub.

---

## Threat Flags

None. The bounded Levenshtein has the same mitigations as the Rust mirror (T-59-02-DOS): early-exit cap at `maxDist + 1` bounds worst-case work; fuzzy gated by `q.length >= 2`. No new network/auth/file surface.

---

## Self-Check: PASSED

- hp41-gui/src/help_data.ts: EXISTS, contains `levenshteinBounded`, `scoreEntry`, `rankedEntries`, provenance comment `help_data.rs`
- hp41-gui/src/HelpOverlay.tsx: EXISTS, contains `rankedEntries`, `query.trim() !== ''`
- Commit 9e15ad5: EXISTS on develop (`git log --oneline -5` confirms)
- Commit 2a05697: EXISTS on develop
- `cd hp41-gui && npm test`: 302 passed, 0 failed
- No hp41-core files touched; no hp41-cli files touched
- No new npm dependencies added
