---
gsd_state_version: 1.0
milestone: v4.2
milestone_name: Help Search Enrichment
status: planning
last_updated: "2026-06-04T00:00:00.000Z"
last_activity: 2026-06-04
progress:
  total_phases: 4
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State: HP-41 Calculator Emulator

---

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-29 for v4.1 iOS Foundation)

**Core value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.

**Current focus:** v4.2 Help Search Enrichment — intent-aware `?` overlay (DE+EN aliases, hand-rolled fuzzy matching, relevance ranking). UI/help milestone only: `hp41-core` untouched, no new runtime deps, no save-file impact.

---

## Current Position

Phase: Not started (roadmap created, planning begins with Phase 58)
Plan: —
Status: Roadmap created, ready to plan
Last activity: 2026-06-04 — v4.2 roadmap created (Phases 58–61, 18 requirements)

## Progress Bar

```
v4.2 Help Search Enrichment
Phase 58 ░░░░░░░░░░  0%   Phase 59 ░░░░░░░░░░  0%
Phase 60 ░░░░░░░░░░  0%   Phase 61 ░░░░░░░░░░  0%
Overall  ░░░░░░░░░░  0%
```

| Phase | Goal | Status |
|-------|------|--------|
| 58 | Data Model — `search_aliases` field on both help-entry mirrors | Not started |
| 59 | Runtime Matcher — alias-aware, tiered scoring, hand-rolled fuzzy, relevance-ranked | Not started |
| 60 | Alias Authoring Pipeline — `scripts/help-aliases/` + LLM runner + all 6 JSON pools populated | Not started |
| 61 | Quality Gates — unit tests, parity fixture, schema CI gate, CLAUDE.md docs | Not started |

## Quick Tasks Completed

| # | Description | Date | Commit | Status | Directory |
|---|-------------|------|--------|--------|-----------|
| 260602-kw4 | Eliminate whitespace around the calculator — scale GUI to fill viewport (macOS + iPhone) | 2026-06-02 | dd71fb2 | Complete ✓ | [260602-kw4-eliminate-whitespace-around-the-calculat](./quick/260602-kw4-eliminate-whitespace-around-the-calculat/) |
| 260603-e4e | Allow entering '.1' as '0.1' (leading-zero number entry, real HP-41CV behavior) | 2026-06-03 | d127e97 | Complete ✓ | [260603-e4e-allow-entering-1-as-0-1-leading-zero-num](./quick/260603-e4e-allow-entering-1-as-0-1-leading-zero-num/) |
| 260603-klp | Fix help-overlay search field rendering off-screen at iPhone top edge (safe-area inset) so input and close X are reachable | 2026-06-03 | 8d0fd00 | Complete ✓ | [260603-klp-fix-help-overlay-search-field-rendering-](./quick/260603-klp-fix-help-overlay-search-field-rendering-/) |
| 260603-laz | iOS touch polish — re-fit calculator scale on help/settings overlay close + lock pinch-zoom (surfaced verifying 260603-klp) | 2026-06-03 | 1e485ff | Complete ✓ | [260603-laz-ios-touch-polish-re-fit-calculator-on-ov](./quick/260603-laz-ios-touch-polish-re-fit-calculator-on-ov/) |
| 260603-mxg | iOS PRGM-mode layout — safe-area handled OUTSIDE the CSS-transform (top display no longer clipped under Dynamic Island), via stylesheet class not inline env() (WKWebView drops inline env). Bottom-sheet occlusion later mooted by 260603-o2e | 2026-06-03 | 6fc9d6c | Complete ✓ | [260603-mxg-fix-ios-prgm-mode-layout-program-source-](./quick/260603-mxg-fix-ios-prgm-mode-layout-program-source-/) |
| 260603-o2e | Authentic single-step PRGM view — main display shows current step (SST/BST navigate); removed inauthentic program listing (iOS sheet + desktop panel); restored CLI↔GUI parity D-25.6 | 2026-06-03 | 1eeba7e | Complete ✓ | [260603-o2e-authentic-hp-41-prgm-view-single-program](./quick/260603-o2e-authentic-hp-41-prgm-view-single-program/) |
| 260603-lu0 | Help-overlay function index — A: CLREG→CLRG fidelity; B: tabbed overlay ("Keyboard Shortcuts" \| "All Functions") exposing all 74 keyless built-ins, tap-to-run (XEQ-by-name / insert-step in PRGM), HP-41-styled; C: Vitest+Rust coverage guardrails | 2026-06-03 | 652b4b1 | Complete ✓ | [260603-lu0-help-overlay-function-index-clrg-fidelit](./quick/260603-lu0-help-overlay-function-index-clrg-fidelit/) |
| 260603-s17 | Mnemonic fidelity — CL SIGMA→CLΣ (glyph; data-only, resolver/mirrors already had it); hide CLRALPHA legacy alias from All Functions index via OVERLAY_HIDDEN_ALIASES (Op kept for v1.0 save compat, Pitfall 8) | 2026-06-03 | 8a8e6de | Complete ✓ | [260603-s17-mnemonic-fidelity-cl-sigma-clsigma-glyph](./quick/260603-s17-mnemonic-fidelity-cl-sigma-clsigma-glyph/) |
| 260603-scc | CLI `?` overlay completeness — keyless built-ins now show `XEQ "NAME"`, deferred-v3 filtered out, C1-analog guardrail added (CLI↔GUI parity for function discovery) | 2026-06-03 | b47f921 | Complete ✓ | [260603-scc-cli-help-overlay-completeness-xeq-hint-f](./quick/260603-scc-cli-help-overlay-completeness-xeq-hint-f/) |
| 260603-sef | Portal iOS print sheet + ALPHA bar out of the scale transform (position:fixed was trapped by the transform → glued to the calculator edge). Print sheet now sits correctly; ALPHA-bar refinements superseded by 260603-u6t | 2026-06-03 | 7e27da4 | Complete ✓ | [260603-sef-portal-ios-print-bottomsheet-alphatouchi](./quick/260603-sef-portal-ios-print-bottomsheet-alphatouchi/) |
| 260603-u6t | Native keys-only ALPHA entry on iOS — on-screen blue keys only (no iOS keyboard, even FUNCTION NAME?); ← + physical Backspace delete last alpha char; iOS touch targets sized to actual key (fixes wide-ENTER 'N') | 2026-06-03 | 65e7a3a | Complete ✓ | [260603-u6t-ios-alpha-entry-native-hp-41-keys-only-o](./quick/260603-u6t-ios-alpha-entry-native-hp-41-keys-only-o/) |
| 260603-uzh | Documentation pass — 4 ADRs (v4.1-003 keys-only ALPHA, -004 help-overlay index, -005 iOS scale/safe-area, -006 single-step PRGM) + new docs/hp41cv-divergences.md + CLAUDE.md GUI-specifics + architecture-history v4.1 section | 2026-06-03 | (pending) | Complete ✓ | [260603-uzh-documentation-pass-adrs-claude-md-divergen](./quick/260603-uzh-documentation-pass-adrs-claude-md-divergen/) |

## Performance Metrics (v4.0 ship baseline)

| Metric | Target | Last measured (v4.0) |
|--------|--------|----------------------|
| Cold-start latency | <= 0.5 s | 2.2 ms (M1) |
| Key-press latency | <= 50 ms | ~65 ns/op |
| `hp41-core` line coverage | >= 95 % | ~93 % (denominator dilution) |
| `hp41-core` region coverage | >= 93 % | ~95 % |
| Numerical accuracy | >= 98 % | 98.86 % (843 cases) |
| Panics in `hp41-core` | 0 | 0 |
| Free42 contamination | 0 | 0 (18-token guard) |
| CI platforms | Win/macOS/Ubuntu | All green |
| Tests passing | — | 3371 (v4.0 baseline) |

---

## Accumulated Context

### Decisions (pre-resolved from design spec)

- **`hp41-core` is untouched** — this is a UI/help-only milestone. Zero changes to `hp41-core/`. Frozen Invariant holds.
- **No new runtime dependency** — fuzzy distance is hand-rolled (~30-40 LOC Rust + TS mirror), honoring *Zero new runtime deps since v3.0*.
- **No save-file / `CalcState` impact** — `search_aliases` is documentation data, not state. No migration, no `#[serde(default)]` needed on `CalcState`.
- **Mirrored matcher pattern** — CLI `filter_help_rows` (Rust) and GUI `HelpOverlay.tsx` filter (TS) are upgraded in parallel, following the same duplication discipline as `op_display_name` / `filter_help_rows`. The parity fixture (HSQUAL-02) is the drift guard.
- **DE aliases are intentional** — the project's English-only doc rule applies to docs/ADRs/planning. `search_aliases` is search *input*, not documentation, so German aliases are in-scope (noted in spec and CLAUDE.md update HSQUAL-04).
- **No regenerate-and-diff CI gate** — LLM alias output is non-deterministic. CI gate is schema-only (HSQUAL-03): validates field presence and type, checks every `status: "implemented"` entry has >= 1 alias.
- **`scripts/help-aliases/` scaffold** — follows the `scripts/docs-matrix/` sibling pattern. Dev-only; no production binary footprint.
- **Phase dependency order** — 58 (field) → 59 (matcher, can use the field without aliases yet) → 60 (populates aliases) → 61 (tests + CI gate require both matcher and alias data).
- **Ranking: non-empty query = relevance-ranked flat list** — empty query preserves existing category-grouped view unchanged (HSMATCH-04 / HSUX-01).

### Pitfalls to watch

| ID | Description | Phase |
|----|-------------|-------|
| P-HS-01 | `filter_help_rows` currently works on `HelpRow` (a slimmed projection); if `search_aliases` is not passed through to the row, the matcher cannot see aliases — extend `HelpRow` or score directly over `HelpEntry` | 59 |
| P-HS-02 | Fuzzy match can produce false positives with short queries (e.g. "x" fuzzy-matching many entries) — apply a minimum edit-distance threshold relative to query length | 59 |
| P-HS-03 | The parity fixture (HSQUAL-02) must use a fixed snapshot of alias data, otherwise a future alias edit would silently break the fixture | 61 |
| P-HS-04 | `just schema-check` recipe must not call the LLM script — it validates existing JSON content only | 61 |
| P-HS-05 | JSON size growth (~90KB today) — verify `include_str!` / TS import at CI still compiles inside bundle-size limits | 60 |

### Blockers

None.

### Pending Todos

- Run `/gsd-plan-phase 58` to plan Phase 58: Data Model

---

## Deferred Items

| Category | Item | Status |
|----------|------|--------|
| Deferred | Interrupting control alarm execution | Still deferred; data model ready (D-38.4); requires call-stack re-entrancy |
| Deferred | App Store submission (STORE-01, STORE-02) | v4.2+ milestone |
| Deferred | iPad universal layout (IPAD-01) | v4.2+ milestone |
| Deferred | Landscape orientation (LAND-01) | v4.2+ milestone |
| Deferred | Android (ANDROID-01) | v4.2+ milestone |
| Deferred | .raw file picker on iOS (RAW-IOS-01) | v4.2+ (no native iOS picker in tauri-plugin-dialog) |
| Deferred | HSCOV-01 missed-query log (zero-result queries) | v4.3+ (accepted gap for v4.2; see REQUIREMENTS.md v2 section) |

---

*State initialized: 2026-05-06*
*Last updated: 2026-06-04 — v4.2 Help Search Enrichment roadmap created (Phases 58–61, 18 requirements, 4 phases). Next: `/gsd-plan-phase 58`.*
