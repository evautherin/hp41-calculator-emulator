---
phase: 61
slug: quality-gates
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-05
---

# Phase 61 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Phase 61 is itself a verification phase (HSQUAL-01..04). Every deliverable is a
> test, a fixture, a CI gate, or a docs edit — the validation surface IS the work.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Rust framework** | `cargo test` (`#[test]`) via `just test` |
| **TS framework** | Vitest 4.1.6 (`vite.config.ts`: jsdom, `globals: false`) via `just gui-ci` |
| **Schema gate** | bash + `jq` script wired as a `just` recipe (precedent: `scripts/check-free42-contamination.sh` / `just license-audit`) |
| **Rust quick run** | `cargo test -p hp41-cli --test phase61_help_search_aliases` |
| **TS quick run** | `cd hp41-gui && npm test -- help_data` |
| **Schema quick run** | `just schema-aliases-check` |
| **Full suites** | `just test` + `just gui-ci` |
| **Estimated runtime** | Rust ~5 s · TS ~10 s · schema gate ~1 s |

---

## Sampling Rate

- **After every task commit:** Run the matching quick command (Rust unit / TS unit / `just schema-aliases-check`).
- **After every plan wave:** Run `just test` + `just gui-ci`.
- **Before `/gsd-verify-work`:** Full suite + `just schema-aliases-check` must all be green.
- **Max feedback latency:** ~15 seconds (quick commands).

---

## Per-Task Verification Map

> Task IDs are assigned by the planner. This map records the requirement → test-command
> binding each task must satisfy; the planner fills the exact `{N}-NN-NN` IDs.

| Requirement | Behavior under test | Test Type | Automated Command | File (Wave 0) |
|-------------|---------------------|-----------|-------------------|---------------|
| HSQUAL-01 (Rust) | tiers exact>prefix>substring>fuzzy, fuzzy hit, DE+EN alias on **real JSON**, empty-query passthrough | unit/integration | `cargo test -p hp41-cli --test phase61_help_search_aliases` | `hp41-cli/tests/phase61_help_search_aliases.rs` |
| HSQUAL-01 (TS) | same behaviors via `scoreEntry`/`rankedEntries` on real JSON | unit | `cd hp41-gui && npm test -- help_data` | `hp41-gui/src/help_data.test.ts` (Phase 61 block) |
| HSQUAL-02 | CLI↔GUI identical top-1 for canonical query set (drift guard) | integration | both commands above, shared fixture | `docs/fixtures/help_search_parity.json` + both tests |
| HSQUAL-03 | `search_aliases` present + array-of-strings across all **six** pools; every `status:"implemented"` entry has ≥1 alias | schema | `just schema-aliases-check` | `scripts/check-aliases-schema.sh` + `Justfile` recipe + `ci.yml` job |
| HSQUAL-04 | docs amendment | manual review | n/a | `CLAUDE.md` JSON-canonical-data-flow section |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `hp41-cli/tests/phase61_help_search_aliases.rs` — real-data tier/fuzzy/alias/empty tests + Rust side of parity fixture
- [ ] `docs/fixtures/help_search_parity.json` — canonical `query → [ranked display_names]` fixture (top-1 assertions, unambiguous queries)
- [ ] `hp41-gui/src/help_data.test.ts` Phase 61 block — TS mirror tests + TS side of parity fixture
- [ ] `scripts/check-aliases-schema.sh` — six-pool enumeration (incl. `hp41cv`), type + ≥1-alias-per-implemented assertions
- [ ] `Justfile` `schema-aliases-check` recipe — wires the script
- [ ] `.github/workflows/ci.yml` `schema-aliases` job — wires the recipe into CI (NOT ci-gui.yml — that has a path filter excluding `docs/*.json`)

*Note: a small, data-only, no-LLM alias hand-edit may be required so the spec's named examples ("Zinseszins"/"compound interest"→TVM, "Wurzel"→SQRT) resolve top-1 — Phase 60's generator chose different German terms. This is fixture/data input to the HSQUAL-01 tests, not a runtime change.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| CLAUDE.md documents `search_aliases`, DE-in-search convention, upgraded matcher | HSQUAL-04 | Prose docs edit — no executable assertion | Read CLAUDE.md "JSON canonical data flow"; confirm the three additions are present and accurate |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags (`vitest run`, not `vitest`)
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
