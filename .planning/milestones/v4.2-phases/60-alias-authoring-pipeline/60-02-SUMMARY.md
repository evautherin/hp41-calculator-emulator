---
phase: 60-alias-authoring-pipeline
plan: 02
subsystem: help-search
tags: [help-aliases, json-pools, claude-cli, de-en-search, minimal-diff]

# Dependency graph
requires:
  - phase: 60-01
    provides: "scripts/help-aliases/ generator crate (pool I/O, fill-only merge, claude -p wrapper, batching)"
provides:
  - "DE+EN search_aliases populated for all 364 status:\"implemented\" entries across the six docs/hp41-*-functions.json pools"
  - "Byte-preserving text-splice writeback in pool.rs (replaces the churning PrettyFormatter) — true alias-only diffs"
  - "--apply-cache re-apply mode in the generator (re-apply a saved alias map with no live LLM call)"
affects: [phase-61-schema-gate, help-search-verification]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Byte-preserving JSON text splice for minimal-diff writeback (insert before entry closing brace; never re-serialize)"
    - "Cache-driven re-apply path so a paid LLM generation run can be re-applied to pristine files at zero cost"

key-files:
  created: []
  modified:
    - "docs/hp41cv-functions.json (136 entries aliased)"
    - "docs/hp41-math1-functions.json (45)"
    - "docs/hp41-stat1-functions.json (26)"
    - "docs/hp41-time-functions.json (35)"
    - "docs/hp41-advantage-functions.json (114)"
    - "docs/hp41-xmem-functions.json (8)"
    - "scripts/help-aliases/src/pool.rs (splice_aliases replaces save_pool/Utf8PrettyFormatter)"
    - "scripts/help-aliases/src/main.rs (build by_op map + splice write; --apply-cache mode)"
    - "scripts/help-aliases/src/claude.rs (dropped --bare flag)"

key-decisions:
  - "Dropped --bare from the claude -p flags: it bypasses the local OAuth session ('Not logged in'); without it the same flags authenticate (resolves RESEARCH Open Question 2)."
  - "Replaced the PrettyFormatter full-file re-serialization with a byte-preserving text splice: PrettyFormatter expands compact inline xrom objects / divergences arrays to multi-line, churning 227 entries and breaking D-60.4."
  - "Re-applied the already-generated aliases from a cached map via a new --apply-cache mode — no second paid LLM run."

patterns-established:
  - "splice_aliases(original_text, by_op): insert search_aliases before each entry's closing brace, preserving every other byte (key order, inline sub-structures, non-ASCII, whitespace)"

requirements-completed: [HSDATA-03, HSGEN-03]

# Metrics
duration: ~75min (incl. ~20min live generation + writeback fix)
completed: 2026-06-05
---

# Phase 60 Plan 02: Alias Data Run Summary

**All 364 implemented help entries across six JSON pools now carry DE+EN search aliases, committed as a true alias-only diff after a Wave-2 defect in the writeback was caught by human review and fixed.**

## Performance

- **Duration:** ~75 min (includes a full live `claude -p` run ~20 min + a writeback fix + cache re-apply)
- **Completed:** 2026-06-05
- **Tasks:** 3 (2 blocking human-verify checkpoints + 1 auto run)
- **Files modified:** 6 pools + 3 crate files

## Accomplishments

- Probed `claude -p` auth/flags before the full run; found `--bare` breaks local OAuth and dropped it (RESEARCH Open Question 2 resolved).
- Generated DE+EN `search_aliases` for **364** `status:"implemented"` entries: hp41cv 136, advantage 114, math1 45, time 35, stat1 26, xmem 8 (~3000 aliases).
- Coverage assertion: **0 uncovered** implemented entries across all six pools (D-60.3 / HSDATA-03).
- Final diff is **alias-only** on every pool (D-60.4): inline `xrom`/`divergences` byte-preserved, no key reorder, no `\uXXXX` escaping of Σ/—/…/umlauts, no whitespace churn — verified mechanically (`bad_removed=0 bad_added=0` per pool).
- Both frontends build with the larger payload: `cargo check -p hp41-cli` ✅, `just gui-ci` ✅ (302 frontend tests). Total pool size **235 KB**, max single pool **81 KB** — none > 200 KB (D-60.7 / P-HS-05).
- Committed as static, PR-reviewable data — NO regenerate-and-diff CI gate added (HSGEN-03).

## Decisions Made

See `key-decisions` frontmatter. The load-bearing one: the writeback must be a byte-preserving text splice, not `serde_json` re-serialization, because the pools hand-author compact inline sub-structures.

## Deviations from Plan

### 1. claude `--bare` flag dropped (sanctioned by Task 1 fallback)
- **Found during:** Task 1 (pre-run probe).
- **Issue:** `claude -p ... --bare ...` returned `"Not logged in · Please run /login"` (exit 1) — `--bare` bypasses the local OAuth session.
- **Fix:** Removed `--bare` from `scripts/help-aliases/src/claude.rs` (the exact fallback Task 1 prescribes). Same flags authenticate cleanly without it.

### 2. Writeback rewritten from PrettyFormatter to byte-preserving splice (60-01 crate corrected)
- **Found during:** Task 2 → Task 3 human review of the data diff.
- **Issue:** 60-01's `save_pool` used `serde_json` `PrettyFormatter`, which expands compact single-line `xrom` objects (math1/stat1/time/advantage) and inline `divergences` arrays (hp41cv) to multi-line form — churning **227 entries** and violating D-60.4. 60-01's `roundtrip_identity` test passed only because its fixture had no inline sub-structures.
- **Fix:** Replaced `save_pool`/`Utf8PrettyFormatter` with `splice_aliases` (byte-preserving text insertion) in `pool.rs`; added a `splice_preserves_inline_xrom` regression test (+ utf8/noop/skip/append tests). Added `--apply-cache` to `main.rs` to re-apply the already-generated aliases without a second LLM run. Live path now builds a `by_op` map and writes via the splice.
- **Verification:** 14/14 crate tests green; per-pool alias-only-diff check passes; coverage unchanged (364).
- **Impact:** Corrects a latent 60-01 defect that would recur on every future `just help-aliases` re-run (HSGEN-03). No scope creep — the generator's contract (minimal-diff writeback, D-60.4) is unchanged; only the implementation was fixed.

## Issues Encountered

The full live generation run was completed first (it churned 227 entries). Rather than discard the paid-for output, the alias map was cached to a sidecar, pools restored to pristine, the writeback fixed, and the cached aliases re-applied via `--apply-cache` — zero new LLM cost.

## Verification

- Coverage: 0 uncovered implemented entries (python assertion, all six pools).
- Alias-only diff: per-pool `bad_removed=0 bad_added=0`; inline xrom 35/114 preserved, 0 expanded; 0 Unicode escapes.
- Builds: `cargo check -p hp41-cli` exit 0; `just gui-ci` exit 0 (302 tests).
- Sizes: total 235 KB, max 81 KB, none > 200 KB.
- `grep -c 'help-aliases-check' Justfile` == 0 (no regenerate-and-diff gate — HSGEN-03).

## Next

Phase 61 (Schema Gate + Verification) — validate the alias schema and confirm the deferred DE/typo queries (Zinseszins, Wurzel, compound interest) actually resolve against the matcher + this data. NOTE for that phase: "Zinseszins" is not yet a literal alias on the TVM/interest entries — a candidate hand-add (fill-only preserves it) if Phase-61 UAT shows it doesn't resolve.
