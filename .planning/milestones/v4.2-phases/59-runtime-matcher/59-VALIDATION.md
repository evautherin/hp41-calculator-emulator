---
phase: 59
slug: runtime-matcher
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-04
---

# Phase 59 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `59-RESEARCH.md` § Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework (Rust)** | Cargo test via `just` |
| **Framework (TS)** | Vitest (`globals: false`, jsdom) |
| **Config file (TS)** | `hp41-gui/vite.config.ts` (`test:` block) |
| **Quick run command** | `just test-core` (fast) or `cargo test -p hp41-cli --test phase59_help_search`; `cd hp41-gui && npm test -- help_data` |
| **Full suite command** | `just test` (Rust workspace) + `cd hp41-gui && npm test` |
| **Estimated runtime** | ~seconds (no real JSON needed — synthetic `HelpEntry` fixtures) |

---

## Sampling Rate

- **After every task commit:** Run `just test` (Rust) + `cd hp41-gui && npm test` (Vitest)
- **After every plan wave:** Run full suite (`just test` + `npm test`)
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

> Task IDs are provisional until plans are written; rows are requirement-anchored from the research test map. Mirror discipline: every Rust scoring assertion has a TS twin (CLI↔GUI parity).

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 59-xx | TBD | 0 | — | — | N/A | unit (stubs) | `cargo test -p hp41-cli --test phase59_help_search` | ❌ W0 | ⬜ pending |
| 59-xx | TBD | 1 | HSMATCH-01 | — | N/A | unit (Rust+TS) | `cargo test -p hp41-cli --test phase59_help_search` | ❌ W0 | ⬜ pending |
| 59-xx | TBD | 1 | HSMATCH-02 | — | N/A | unit (Rust+TS) | `cargo test -p hp41-cli --test phase59_help_search -- tier_order` | ❌ W0 | ⬜ pending |
| 59-xx | TBD | 1 | HSMATCH-03 | — | N/A | unit (Rust+TS) | `cargo test -p hp41-cli --test phase59_help_search -- fuzzy_typo` | ❌ W0 | ⬜ pending |
| 59-xx | TBD | 1 | HSMATCH-04 | — | N/A | unit (Rust+TS) | `cargo test -p hp41-cli -- empty_query_returns_all` + new invariant test | Partial | ⬜ pending |
| 59-xx | TBD | 1 | HSMATCH-05 | — | N/A | unit (Rust+TS) | `cargo test -p hp41-cli --test phase59_help_search -- alias_de_en` | ❌ W0 | ⬜ pending |
| 59-xx | TBD | 1 | HSUX-01 | — | N/A | smoke/render | existing `HelpOverlay.test.tsx` input test + ranked-render test | Partial | ⬜ pending |
| 59-xx | TBD | 1 | HSUX-02 | — | N/A | unit (Rust+TS) | `cargo test -p hp41-cli --test phase59_help_search -- fuzzy_wurzel` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-cli/tests/phase59_help_search.rs` — scoring tiers, fuzzy typo hits (Zineszins→TVM, Wurzel→SQRT), DE/EN alias resolution, empty-query invariance. Synthetic `HelpEntry` fixtures (tests the scoring logic, not the JSON data).
- [ ] `hp41-gui/src/help_data.test.ts` (extended) — `scoreEntry` tier ordering, fuzzy, alias, empty-passthrough.
- [ ] `hp41-gui/src/HelpOverlay.test.tsx` (extended) — flat-ranked render branch: non-empty query → results without section headers; empty query → section groups (HSMATCH-04 regression guard).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visual relevance ordering feels right in the live `?` overlay | HSUX-01/02 | Subjective ranking quality beyond unit-asserted tier order | Open CLI + GUI overlay, type "Zineszins" / "Wurzel" / "compound interest", confirm intended entry is the top hit |

*All tier/fuzzy/alias/empty-query behaviors otherwise have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (3 test files above)
- [ ] No watch-mode flags (`vitest run`, not `vitest`)
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
