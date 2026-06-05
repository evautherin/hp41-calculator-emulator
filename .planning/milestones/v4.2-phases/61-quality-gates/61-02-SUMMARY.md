---
phase: 61-quality-gates
plan: 02
subsystem: help-search
tags: [testing, quality-gate, help-search, cli, parity, aliases]
requires: ["61-01"]
provides: ["hp41-cli help-search real-data verification suite", "CLI-side parity-fixture assertion"]
affects: ["hp41-cli/tests"]
tech-stack:
  added: []
  patterns: ["include_str! fixture embed", "score_entry tier-ordering proof against real loaded entry", "ranked_help_entries top-1 alias assertion"]
key-files:
  created:
    - hp41-cli/tests/phase61_help_search_aliases.rs
  modified: []
decisions:
  - "Tier hierarchy proven via numeric ordering of score_entry on the REAL SQRT entry (exact > prefix > substring > fuzzy), since the tier constants are module-private."
  - "Whitespace-only query (\"   \") used to prove the blank-input contract; the literal empty string is avoided because ranked_help_entries debug_asserts on it."
  - "Display-name field verified as HelpRow.op (built from entry.display_name in ranked_help_entries) by reading hp41-cli/src/help_data.rs."
metrics:
  duration: ~25m
  completed: 2026-06-05
requirements: [HSQUAL-01, HSQUAL-02]
---

# Phase 61 Plan 02: Help-Search Real-Data Verification Summary

CLI integration test that proves the Phase 59 tiered scorer and Phase 60 DE+EN alias content against the live six-pool JSON help data, plus the cross-implementation parity fixture from 61-01.

## What Was Built

`hp41-cli/tests/phase61_help_search_aliases.rs` — 10 tests, all GREEN:

1. **`four_scoring_tiers_against_real_sqrt_entry`** — exercises all four tiers on the REAL loaded `SQRT` entry via `score_entry`: exact (`"sqrt"`), prefix (`"sqr"`), substring (`"qrt"`), fuzzy (`"sqirt"`). Asserts each is `> 0` and the strict ordering `exact > prefix > substring > fuzzy`.
2. **`exact_name_outranks_fuzzy_top1`** — `sqrt`->SQRT, `pi`->PI top-1 baseline regression.
3. **DE+EN alias top-1** (5 tests): `zeitwert des geldes`->TVM, `zinseszins`->TVM, `square root`->SQRT, `quadratwurzel`->SQRT, `wurzel`->SQRT.
4. **`fuzzy_typo_sqirt_resolves_sqrt`** — fuzzy-hit top-1.
5. **`whitespace_only_query_yields_no_ranked_results`** — `ranked_help_entries("   ")` returns an empty Vec (blank-input contract; literal `""` avoided per the `debug_assert!` landmine).
6. **`parity_fixture_top1_matches_for_every_query`** — `include_str!("../../docs/fixtures/help_search_parity.json")` -> serde_json -> iterate `queries` -> assert every query's top-1 `.op` equals `expected_top[0]`. Same fixture asserted by the GUI Vitest suite => proves CLI<->GUI matcher parity.

## API Facts Verified (from hp41-cli/src/help_data.rs)

- `ranked_help_entries(query: &str) -> Vec<HelpRow>` — **single arg** (the plan-context hint suggesting `ranked_help_entries(q, n)` was wrong).
- `HelpRow` display-name field is **`.op`** (built from `entry.display_name.clone()` inside `ranked_help_entries`).
- `score_entry(entry: &HelpEntry, q: &str) -> u8` is public; tier constants are private (hence ordering-based tier proof).
- `ranked_help_entries` has `debug_assert!(!query.is_empty(), ...)` then trims; whitespace-only trims to empty and returns `Vec::new()`.
- `include_str!` path from `hp41-cli/tests/` is `../../docs/fixtures/...` (two levels to repo root), confirmed by successful compile.

## Verification

- `cargo test -p hp41-cli --test phase61_help_search_aliases` => **10 passed; 0 failed**.
- `cargo fmt -p hp41-cli -- --check` => clean (one diff auto-fixed before commit).
- `cargo clippy -p hp41-cli --tests -- -D warnings` => clean. `#![allow(clippy::unwrap_used)]` at file top (tests may unwrap).
- No JSON data edited; all five required alias mappings confirmed present in `docs/hp41cv-functions.json` (SQRT) and `docs/hp41-advantage-functions.json` (TVM) before assertions were written.

## Deviations from Plan

### [Process] Shared-index parallel-agent race during commit

This plan ran concurrently with sibling executors (61-03 GUI Vitest mirror, 61-04 schema gate) sharing the SAME git index (no worktree isolation — `branching_strategy: none`, working directly on `develop`). The first commit attempt accidentally swept in a sibling's pre-staged `hp41-gui/src/help_data.test.ts` because it was already in the shared index. A subsequent `git reset --soft` + `git restore --staged` to isolate my file then raced with sibling 61-04's commit, which cleared the index.

**Resolution (non-destructive):** Final commit used the pathspec-scoped form `git commit ... -- hp41-cli/tests/phase61_help_search_aliases.rs`, which commits ONLY that path regardless of shared-index contents. Result `d6d580a`: `1 file changed, 200 insertions(+)` — verified atomic via `git show --stat HEAD` and `git ls-files`. **No sibling work was rewritten or lost** — I deliberately did NOT rewrite the sibling commit `6e81734` (that would have been a destructive history rewrite in a live parallel context). Authoritative checks (`git ls-files`, `git log --all`, `git ls-tree HEAD`) confirmed my file was never co-mingled in the final history.

**Lesson:** in shared-index parallel execution, always commit with an explicit pathspec (`git commit -- <file>`) rather than relying on a clean staged set; the index is a shared mutable resource across agents.

No code/test deviations — plan content executed exactly as specified.

## Self-Check: PASSED

- `hp41-cli/tests/phase61_help_search_aliases.rs` — FOUND (tracked, in HEAD `d6d580a`).
- Commit `d6d580a` — FOUND in `git log`, atomic (1 file).
- All 10 tests GREEN; fmt + clippy strict clean.
