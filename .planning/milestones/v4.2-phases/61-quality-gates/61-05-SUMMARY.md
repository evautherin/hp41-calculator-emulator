---
phase: 61-quality-gates
plan: "05"
subsystem: docs
tags: [documentation, help-search, search_aliases, claude-md]
requires: ["61-01", "61-02", "61-03", "61-04"]
provides: ["claude-md-search-enrichment-doc"]
affects: ["CLAUDE.md"]
tech-stack:
  added: []
  patterns: ["surgical CLAUDE.md section edit"]
key-files:
  created:
    - .planning/phases/61-quality-gates/61-05-SUMMARY.md
  modified:
    - CLAUDE.md
decisions:
  - "Used ~380 entries (not legacy ~350): real count across all six pools = 382 (114+45+26+35+8+154). RESEARCH proposed ~380; matches disk."
requirements: [HSQUAL-04]
metrics:
  duration: ~5m
  completed: 2026-06-05
---

# Phase 61 Plan 05: CLAUDE.md Search-Enrichment Documentation Summary

Surgically updated the "JSON canonical data flow" section of repo-root `CLAUDE.md` to document the v4.2 Help Search Enrichment: the invisible `search_aliases` field, the DE-in-search-input exception to the English-only rule, the upgraded tiered matcher with its two named guards, and the five→six pool-count correction.

## What Changed

Single contiguous edit to `CLAUDE.md` lines 76-85 (the JSON canonical data flow section). No other section touched. `git diff` = `4 insertions(+), 1 deletion(-)`.

### The four required edits

1. **`search_aliases` field documented** — new bullet:
   > `search_aliases` (v4.2) — an INVISIBLE match surface: an array of alternative spellings, abbreviations, and synonyms attached to each entry, NEVER rendered in any UI (`?` overlay, right-panel). Populated by the `just help-aliases` generator (Phase 60, dev-only — not a CI step), consumed solely by the tiered search matcher.

2. **DE-in-search-input exception documented** — new bullet, cross-referencing the English-only rule:
   > DE in `search_aliases` is the one sanctioned exception to the project's English-only rule (see "All project docs in English" / the English-only prose convention governing docs, ADRs, planning files, and UI strings): `search_aliases` is search *input* vocabulary, not documentation or rendered text, so German alias values (e.g. `"Zinseszins"`, `"Zeitwert des Geldes"`) are intentional and in-scope by design — they let German-speaking users find functions. This data field is the single place German lives in committed data.

3. **Tiered matcher + two named guards documented** — new bullet:
   > Tiered search matcher (Phase 59, v4.2) — both frontends rank entries by descending match quality: exact > prefix > substring > fuzzy, scored across display_name / search_aliases / description / category. Non-empty query → relevance-ranked flat list; empty query → the existing category-grouped view, unchanged. Two guards: `docs/fixtures/help_search_parity.json` (CLI↔GUI top-1 drift guard; asserted by `phase61_help_search_aliases.rs` + the Phase 61 block in `help_data.test.ts`) and `just schema-aliases-check` (the `ci.yml` schema gate ensuring every `status:"implemented"` entry across all six pools carries ≥1 alias).

4. **Five → Six pool correction (landed)** — opening line changed from:
   > Five `docs/hp41-*-functions.json` files (~350 entries total) drive keybindings…

   to:
   > Six `docs/hp41*-functions.json` pools (~380 entries total — note the sixth, `docs/hp41cv-functions.json`, has no dash before `functions`) drive keybindings…

   The glob trap (`hp41cv-functions.json` lacking the dash) is called out inline.

## Deviations from Plan

### Entry-count figure: ~350 → ~380 (data-verified)
- **Found during:** Task 1 (opening-line correction).
- **Plan truth #4 said:** keep "~350 entries" unless the plan body specifies otherwise.
- **Plan body / RESEARCH (lines 570) specifies:** "~380 entries total".
- **Verification:** counted real entries across all six pools via `jq` — advantage 114, math1 45, stat1 26, time 35, xmem 8, cv 154 = **382 total**. Legacy "~350" predated cv-pool growth.
- **Decision:** used "~380" — both authoritative (RESEARCH-specified) and accurate to disk. Not classified as a deviation rule (it is the plan-specified value); recorded for audit clarity.

No other deviations. No auto-fixes (Rules 1-3 not triggered — pure docs edit). No checkpoints. No auth gates.

## Verification

- `git diff CLAUDE.md` shows exactly one region (JSON canonical data flow); the next section header `### CLI ↔ GUI parity (D-25.6)` is untouched.
- Cross-referenced names against disk: six pool files confirmed (`docs/hp41cv-functions.json` is dash-less), `docs/fixtures/help_search_parity.json` exists, `phase61_help_search_aliases.rs` exists, `just schema-aliases-check` declared in `Justfile:91` and wired in `ci.yml:127`, `just help-aliases` declared in `Justfile:254`.
- Language: English-only prose; the only German is the two quoted example alias values, which are the documented exception itself.

## Commit

- `12ca9cc` — 📝 docs(61-05): document v4.2 search_aliases + tiered matcher in CLAUDE.md

## Self-Check: PASSED
- FOUND: CLAUDE.md (modified, staged, committed)
- FOUND: commit 12ca9cc in git log
- FOUND: .planning/phases/61-quality-gates/61-05-SUMMARY.md
