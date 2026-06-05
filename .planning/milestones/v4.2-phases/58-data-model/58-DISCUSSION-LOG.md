# Phase 58: Data Model - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-04
**Phase:** 58-data-model
**Areas discussed:** JSON seeding scope, Verification scope, Rust field shape

---

## JSON seeding scope

| Option | Description | Selected |
|--------|-------------|----------|
| Leave JSON untouched | Phase 58 = type definitions only; 6 JSON pools stay as-is; serde-default/optional handles absent field; Phase 60 writes the field. Cleanest split, zero churn. | ✓ |
| Seed empty [] in all 6 pools | Add `search_aliases: []` to every entry now; Phase 60 fills them. ~228+ entries of churn that Phase 60 overwrites. | |

**User's choice:** Leave JSON untouched (recommended)
**Notes:** Confirms the dependency split — field defined in 58, content in 60. → D-58.1.

---

## Verification scope

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal backward-compat test | One small test per frontend proving old JSON without the field → empty alias list; richer tests stay in Phase 61. | ✓ |
| Defer all tests to Phase 61 | Phase 58 ships field + green build only; all coverage (incl. backward-compat) in 61. | |

**User's choice:** Minimal backward-compat test (recommended)
**Notes:** Rust gets a real `serde_json::from_str` round-trip; TS side is type-level only (static Vite import, no runtime parse) — asymmetry flagged for planner. → D-58.2.

---

## Rust field shape

| Option | Description | Selected |
|--------|-------------|----------|
| Vec<String> + serde(default) | Mirrors existing `divergences`; empty-by-default list is the cleaner match surface; matcher iterates with no None-handling. | ✓ |
| Option<Vec<String>> | Mirrors existing `xrom`; distinguishes 'no field' from 'empty list' but forces None-unwrap at every call site for no benefit. | |

**User's choice:** Vec<String> + serde(default) (recommended)
**Notes:** Aliases are conceptually a populated-or-empty list, not a presence flag. → D-58.3. TS mirror uses optional `search_aliases?: string[]` (D-58.4).

---

## Claude's Discretion

- Doc-comment wording and field-order placement on both `HelpEntry` definitions.
- Test-file placement and fixture-string contents for the backward-compat tests.

## Deferred Ideas

- Empty-`[]` JSON seeding — considered and rejected for Phase 58 (field first appears in JSON at Phase 60).
- Matcher reads aliases / `HelpRow` widening — Phase 59 (P-HS-01).
- DE+EN alias content + `scripts/help-aliases/` pipeline — Phase 60.
- Schema CI gate, parity fixture, scoring/fuzzy tests, CLAUDE.md doc — Phase 61.
- `privacy-manifest-bundle-wiring.md` (iOS PrivacyInfo) — reviewed via todo-match (score 0.4), not folded; unrelated iOS task.
