---
phase: 58-data-model
plan: 01
subsystem: ui
tags: [help-search, serde, typescript, backward-compat, data-model]

# Dependency graph
requires: []
provides:
  - Rust HelpEntry.search_aliases field (Vec<String>, #[serde(default)], invisible match surface)
  - TS HelpEntry.search_aliases? optional field (string[], mirrors Rust shape)
  - Serde backward-compat round-trip test (Rust): old JSON -> empty Vec
  - Consumer backward-compat test (TS): missing key -> undefined
affects: [59-search-matcher, 60-alias-content]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Additive serde field pattern: #[serde(default)] pub field: Vec<T> (copies divergences idiom)"
    - "TS optional enrichment field pattern: field?: T[] (copies divergences?/xrom?/example?/notes? idiom)"
    - "CLI<->GUI lockstep: TS doc-comment cites Rust mirror file+line"

key-files:
  created: []
  modified:
    - hp41-cli/src/help_data.rs
    - hp41-gui/src/help_data.ts
    - hp41-gui/src/help_data.test.ts

key-decisions:
  - "search_aliases is Vec<String> (not Option<Vec<String>>) on Rust side — mirrors divergences idiom exactly (D-58.3)"
  - "search_aliases is optional (?) on TS side — keeps existing JSON imports typechecking until Phase 60 populates the field (D-58.4)"
  - "Field inserted after xrom in Rust, after notes? in TS — natural optional-enrichment grouping"
  - "HelpRow and all render paths (help_overlay_rows, helpOverlayRows, filterHelpEntries) are untouched — field is a pure invisible match surface (D-58.5)"
  - "All six docs/hp41-*-functions.json pools byte-for-byte unchanged (D-58.1)"

patterns-established:
  - "Phase 59 matcher reads search_aliases from HelpEntry without any HelpRow involvement"
  - "Phase 60 alias content seeds JSON pools, which propagate to HelpEntry via existing serde/import pipeline"

requirements-completed: [HSDATA-01, HSDATA-02, HSDATA-04]

# Metrics
duration: 12min
completed: 2026-06-04
---

# Phase 58 Plan 01: Data Model Summary

**search_aliases field added to both HelpEntry mirrors (Rust Vec<String> + TS string[]) with serde-default and optional-field backward-compat tests; all six JSON pools and all render paths provably untouched**

## Performance

- **Duration:** 12 min
- **Started:** 2026-06-04T17:03:00Z
- **Completed:** 2026-06-04T17:15:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Added `search_aliases: Vec<String>` with `#[serde(default)]` to Rust `HelpEntry` in `hp41-cli/src/help_data.rs`, mirroring the existing `divergences` idiom (D-58.3 / HSDATA-01)
- Added optional `search_aliases?: string[]` to TS `HelpEntry` interface in `hp41-gui/src/help_data.ts` with doc-comment citing the Rust mirror line (D-58.4 / HSDATA-02, lockstep parity)
- Added backward-compat tests in both frontends: Rust serde round-trip (JSON without field -> empty Vec) and TS consumer assertion (object without field -> undefined)

## Task Commits

1. **Task 1: Add search_aliases to Rust HelpEntry + serde-default round-trip test** - `aa4b525` (feat)
2. **Task 2: Mirror search_aliases on TS HelpEntry interface + thin consumer test** - `7b0aae7` (feat)

## Files Created/Modified
- `hp41-cli/src/help_data.rs` - Added `search_aliases: Vec<String>` field with `#[serde(default)]` after `xrom`; added `search_aliases_defaults_to_empty_vec_when_field_absent` test in `#[cfg(test)]` block
- `hp41-gui/src/help_data.ts` - Added `search_aliases?: string[]` field with JSDoc citing Rust mirror at ~line 87
- `hp41-gui/src/help_data.test.ts` - Added `HelpEntry.search_aliases backward-compat` describe block with one assertion; updated import to include `HelpEntry` type

## Decisions Made
- `Vec<String>` not `Option<Vec<String>>` on Rust side — exact copy of `divergences` idiom; the TS optional `?` provides the optional semantics on the TypeScript side
- Field inserted at end of each struct/interface in the "optional enrichment" grouping, after `xrom` (Rust) / `notes?` (TS) — natural position for future-phase enrichment fields
- No new `#[allow(...)]` attributes needed — the existing `#[allow(dead_code)]` on `HelpEntry` struct already covers an unread field

## Deviations from Plan

None - plan executed exactly as written. The `npm install` step in the worktree was needed because the worktree's `hp41-gui/` directory had no `node_modules/`; this is a worktree environment detail, not a plan deviation.

## Issues Encountered
- Worktree `hp41-gui/` had no `node_modules/` — ran `npm install` once. Subsequent typecheck and tests ran cleanly.

## Threat Flags

None - this plan adds no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries.

## Known Stubs

None - the `search_aliases` field intentionally contains an empty Vec / undefined value until Phase 60 populates the JSON pools. This is by design: Phase 60 is the alias-content phase.

## Next Phase Readiness
- Phase 59 (search matcher) can now read `entry.search_aliases` from both `HelpEntry` mirrors
- Phase 60 (alias content) can populate the six JSON pools — the field will deserialize automatically via `#[serde(default)]` / optional TS field
- All existing tests green; no regressions

## Self-Check: PASSED

- `hp41-cli/src/help_data.rs` modified — committed at aa4b525
- `hp41-gui/src/help_data.ts` modified — committed at 7b0aae7
- `hp41-gui/src/help_data.test.ts` modified — committed at 7b0aae7
- `just test` exits 0 — verified
- `cd hp41-gui && ./node_modules/.bin/tsc --noEmit` exits 0 — verified
- `cd hp41-gui && npm test` exits 0 (290 tests passed) — verified
- `git diff --stat -- 'docs/hp41-*-functions.json'` empty — verified

---
*Phase: 58-data-model*
*Completed: 2026-06-04*
