# Phase 34: hp41-cli — CLI Integration - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-22
**Phase:** 34-hp41-cli-cli-integration
**Areas discussed:** JSON category granularity, Per-entry `divergences` JSON field policy, Plan slicing, Help-overlay section header + `?` search shape

---

## JSON category granularity

| Option | Description | Selected |
|--------|-------------|----------|
| Per-family (7 cats) | `Stat1 Univariate` (4) / `Stat1 ANOVA` (3) / `Stat1 Regression` (8) / `Stat1 Hypothesis` (2) / `Stat1 Nonparam` (5) / `Stat1 Distributions` (2) / `Stat1 RNG` (2). Mirrors Phase 29 per-program shape; `?`-overlay groups by first-appearance and Stat 1 sections cluster together visually. | ✓ |
| Per-family but use OM section names | Use the QRC/OM section labels verbatim: `Stat1 Basic Stats` / `Stat1 ANOVA` / `Stat1 Curve Fitting` / `Stat1 Multi-Regression` / `Stat1 Hypothesis Tests` / `Stat1 Distributions` / `Stat1 Random Numbers`. Hews closer to the canonical OM TOC; nicer for OM-cross-referencing in docs. | |
| Flat `Stat 1 Pac` | One single category for all 26 entries. Simpler; matches the v2.2 built-in categories' typical size. Loses the family clustering benefit Phase 29 specifically called out as a discoverability win. | |

**User's choice:** Per-family (7 cats)
**Notes:** Locked as D-34.1. The `Math1 X`-prefixed convention from Phase 29 is the deciding factor — keeps the JSON pool naming consistent with the existing Math Pac I file so a future v3.2 Time Pac will continue the `Time1 X` pattern.

---

## Per-entry `divergences` JSON field policy

| Option | Description | Selected |
|--------|-------------|----------|
| Surgical inline + full catalog in Phase 35 | Populate `divergences` ONLY for entries with user-visible quirks: `RAND`/`SEED` (emulator extension), `ΣTSTAT` (pooled-variance only), `ΣPOLYP` (DEGREE=? convention). Everything else stays empty; full taxonomy lives in `docs/hp41-stat1-divergences.md` Phase 35. Mirrors Math Pac I JSON shape. | ✓ |
| Empty in JSON; defer 100% to Phase 35 doc | All 26 entries get `divergences: None`. Avoids per-JSON drift vs canonical `divergences.md`. Downside: `?`-overlay won't surface any HP-41 divergence hints inline. | |
| Inline EVERY known divergence + full catalog | Populate `divergences` exhaustively for every entry that has any OM/QRC/NPS divergence note from Phase 33 research — estimated ~10 of 26 entries. More upfront work; drift risk if either is updated without the other. | |

**User's choice:** Surgical inline + full catalog in Phase 35
**Notes:** Locked as D-34.3. Three exact strings recorded inline:
- RAND/SEED: v3.1 emulator extension, LCG formula provenance
- ΣTSTAT: pooled-variance only, Welch excluded
- ΣPOLYP: DEGREE=? is HP-41 emulator convention

---

## Plan slicing

| Option | Description | Selected |
|--------|-------------|----------|
| 2 plans | Plan 34-01: author full 26-entry JSON + third `OnceLock` + chain into `help_entries_all()` + smoke test. Plan 34-02: ~24 `op_display_name` arms + extend `function_matrix_parity.rs` to 3 pools + extend `phase25_xeq_by_name` + modal-flow smoke. Plan 34-01 can land as a green build prerequisite, then Plan 34-02 closes the 4-way invariant break. | ✓ |
| 1 plan (atomic phase) | Single PLAN.md covering JSON + OnceLock + arms + parity + smoke tests in one atomic landing. Phase scope is small (26 entries, mostly templated). Harder to bisect; harder to review. | |
| 3 plans (mirror Phase 29) | 34-01 JSON + OnceLock; 34-02 arms in prgm_display + parity extension; 34-03 modal-flow tests + key_coverage + integration. Heaviest per-phase overhead; matches Phase 29 cardinality but Phase 29 had ~55 entries vs Phase 34's 26. | |

**User's choice:** 2 plans
**Notes:** Locked as D-34.2. Plan 34-01 has a green-build exit criterion before Plan 34-02 begins — minimizes the window where the CLI is half-wired.

---

## Help-overlay section header + `?` search shape

### Section header naming convention

| Option | Description | Selected |
|--------|-------------|----------|
| `Stat 1 Pac (XROM 2)` | Direct parallel to existing `Math 1 Pac (XROM 7)`. Same prose style, same parentheses, swap module name + ID. Pattern continues for future Time/Advantage Pacs. Zero discretion drift. | ✓ |
| `STAT 1B (XROM 2)` | Uses the catalog display name `STAT 1B` (as it appears in `STAT_1.name` and CATALOG 2 enumeration). Matches what the user sees on real hardware. Diverges from Math Pac I header style. | |
| `Stat 1 Pac (XROM 2 — STAT 1B)` | Belt-and-suspenders: human-readable name + module ID + catalog display name. More discoverable. Risks overflowing the 80-char overlay row width. | |

**User's choice:** `Stat 1 Pac (XROM 2)`
**Notes:** Locked as D-34.5.

### `?` search result ordering

| Option | Description | Selected |
|--------|-------------|----------|
| Three sections, fixed order | Built-ins → Math 1 Pac (XROM 7) → Stat 1 Pac (XROM 2). Search ranks all three pools equally on the same match score; section headers separate them visually. Matches `help_entries_all()` natural chain order from Phase 29 D-29.2; no new sort logic needed. | ✓ |
| Three sections, alpha-by-section-header | Built-ins → Math 1 Pac → Stat 1 Pac by header alphabetization. Same effective order today but stable as new XROM modules land in v3.2+. | |
| Flatten by search score | No section boundaries during search; results from all three pools interleave by score. Closer to a modern search-everywhere UX; loses the XROM-section discoverability Phase 29 D-29.1 specifically engineered. Requires new sort logic. | |

**User's choice:** Three sections, fixed order
**Notes:** Locked as D-34.6.

---

## Claude's Discretion

Documented in CONTEXT.md `### Claude's Discretion` block. Notable:
- `function_id` 1-indexing convention (1..=26 in `STAT_1.ops` row order)
- `xrom.module` string value (`"Stat 1"` recommended, mirroring `"Math 1"`)
- `status` field value `"implemented"` for all 26 entries (all Ops shipped in Phase 33)
- `phase` field value `"33"` for all 26 entries
- `function_matrix_parity.rs` extension approach (partition-by-`xrom`-field vs three parallel test functions)
- Test file naming (`phase34_*` mirrors `phase29_*` for grep-ability)
- Per-entry `description` wording (≤ 80 chars, OM page citations optional)
- Plan 34-02 internal commit ordering (one commit per logical step recommended)

## Deferred Ideas

All recorded in CONTEXT.md `<deferred>` block. Summary:
- GUI mirroring → Phase 36
- CATALOG 2 enumeration update → Phase 36 (auto via Phase 31 dynamic listing)
- `docs/hp41-stat1-divergences.md` three-bucket catalog → Phase 35
- `scripts/docs-matrix/` three-input extension + matrix regen → Phase 35
- README v3.1 section + CLAUDE.md additions → Phase 35
- ADRs for v3.1 → Phase 35
- `docs/architecture-history.md` v3.1 narrative → Phase 35
- `numerical_accuracy.rs` Stat 1 cases → Phase 37
- Per-`stat1/*.rs` coverage floors + meta-gates → Phase 37
- v3.0 save backward-compat test → Phase 37
- WebdriverIO E2E with Stat 1 workflow → Phase 37
- Signed binary releases → v3.1.x / v3.2
