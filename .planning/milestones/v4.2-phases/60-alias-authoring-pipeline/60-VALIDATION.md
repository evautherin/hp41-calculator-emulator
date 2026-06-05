---
phase: 60
slug: alias-authoring-pipeline
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-05
---

# Phase 60 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `60-RESEARCH.md` §"Validation Architecture".

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` (no external test framework) |
| **Config file** | none — standard `cargo test` |
| **Quick run command** | `cargo test --manifest-path scripts/help-aliases/Cargo.toml` |
| **Full suite command** | `cargo test --manifest-path scripts/help-aliases/Cargo.toml` |
| **Estimated runtime** | ~3 seconds (pure-logic unit tests; no LLM/network in tests) |

**Note:** all tests live inside the dev-only `scripts/help-aliases/` crate. They test the
generator's *merge + writeback + parse* logic on fixtures — **never** the live `claude` CLI
(non-deterministic, off the test path). No `hp41-core`/`hp41-cli`/`hp41-gui` crate gains tests
from this phase.

---

## Sampling Rate

- **After every task commit:** Run `cargo test --manifest-path scripts/help-aliases/Cargo.toml`
- **After every plan wave:** Run `cargo test --manifest-path scripts/help-aliases/Cargo.toml && cargo check -p hp41-cli`
- **Before `/gsd:verify-work`:** Full suite green + `just help-aliases` run (data populated) + `cargo check -p hp41-cli` + GUI build green
- **Max feedback latency:** ~5 seconds (unit suite)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 60-XX | merge | 1 | HSGEN-01 / HSGEN-03 | — | fill-only: entry with existing aliases left untouched | unit | `cargo test --manifest-path scripts/help-aliases/Cargo.toml merge::tests::fill_only_skips_populated` | ❌ W0 | ⬜ pending |
| 60-XX | merge | 1 | HSGEN-01 | — | entry with empty/absent `search_aliases` gets populated | unit | `cargo test --manifest-path scripts/help-aliases/Cargo.toml merge::tests::fill_only_populates_empty` | ❌ W0 | ⬜ pending |
| 60-XX | pool | 1 | HSGEN-01 / D-60.4 | — | load→save (no aliases added) is byte-identical | unit | `cargo test --manifest-path scripts/help-aliases/Cargo.toml pool::tests::roundtrip_identity` | ❌ W0 | ⬜ pending |
| 60-XX | pool | 1 | D-60.4 | — | key order preserved after alias insertion | unit | `cargo test --manifest-path scripts/help-aliases/Cargo.toml pool::tests::key_order_preserved` | ❌ W0 | ⬜ pending |
| 60-XX | pool | 1 | D-60.4 | — | non-ASCII (Σ, —, umlauts) survives roundtrip un-escaped | unit | `cargo test --manifest-path scripts/help-aliases/Cargo.toml pool::tests::utf8_unescaped` | ❌ W0 | ⬜ pending |
| 60-XX | claude | 1 | HSGEN-01 | — | markdown-fenced `claude` response stripped cleanly | unit | `cargo test --manifest-path scripts/help-aliases/Cargo.toml claude::tests::strip_fences` | ❌ W0 | ⬜ pending |
| 60-XX | batch | 1 | HSDATA-03 | — | `validate_batch_response` detects missing + extra keys | unit | `cargo test --manifest-path scripts/help-aliases/Cargo.toml batch::tests::validate_key_mismatch` | ❌ W0 | ⬜ pending |
| 60-XX | crate | 1 | HSGEN-02 | — | `Cargo.toml` carries empty `[workspace]` stanza (not in shipped build) | manual (grep) | `grep -q '^\[workspace\]' scripts/help-aliases/Cargo.toml` | ❌ W0 | ⬜ pending |
| 60-XX | data | 2 | HSDATA-03 | — | all 364 implemented entries have ≥1 alias after run | manual (run summary) | `just help-aliases` reports "0 entries with no aliases" | ❌ W0 | ⬜ pending |
| 60-XX | data | 2 | D-60.7 / P-HS-05 | — | both frontends compile with populated pools | integration smoke | `cargo check -p hp41-cli && just gui-ci` | existing | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky. Task IDs finalized by the planner; `60-XX` are placeholders mapped to the plan's actual task numbering.*

---

## Wave 0 Requirements

- [ ] `scripts/help-aliases/src/merge.rs` — `fill_only_skips_populated`, `fill_only_populates_empty` (HSGEN-01, HSGEN-03)
- [ ] `scripts/help-aliases/src/pool.rs` — `roundtrip_identity`, `key_order_preserved`, `utf8_unescaped` (HSGEN-01 / D-60.4)
- [ ] `scripts/help-aliases/src/claude.rs` — `strip_fences`, `envelope_parse` (HSGEN-01)
- [ ] `scripts/help-aliases/src/batch.rs` — `validate_key_mismatch`, `validate_no_mismatch` (HSDATA-03)
- [ ] Test fixture — a minimal JSON pool snippet (2 entries: one implemented with empty aliases, one already populated, including a non-ASCII glyph) shared across `merge` and `pool` tests
- [ ] `scripts/help-aliases/Cargo.toml` — new crate with empty `[workspace]` stanza + `serde_json` `features = ["preserve_order"]`

*The generator crate is new, so Wave 0 installs the test scaffolding alongside the first modules.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Full DE+EN alias coverage across all 6 pools | HSDATA-03 | The actual alias *content* is LLM-generated and non-deterministic; a regenerate-and-diff gate is explicitly rejected. Coverage is asserted by the run summary now and by the Phase-61 schema gate later. | Run `just help-aliases`; confirm run summary shows every pool fully populated and "0 entries with no aliases". Spot-check the spec's TVM example resolves to sensible DE+EN aliases. |
| Alias *quality* (no misleading matches) | HSGEN-03 | LLM output needs human judgement to avoid an alias pulling the wrong function to the top of search. | PR review pass over the alias diff per pool; correct/trim any misleading aliases by hand (fill-only means they survive re-runs). |
| `claude -p` auth + envelope under chosen flags | D-60.1 | Depends on the developer's local `claude` auth (OAuth vs `ANTHROPIC_API_KEY`); cannot be unit-tested. | Before the full run: `claude -p --output-format json "reply with: ok"`; if `--bare` breaks auth, drop it (Open Question 2 in RESEARCH.md). |

---

## Nyquist Compliance

- **Sampling ≥ 2× change rate:** unit suite runs per task commit; full suite + `cargo check -p hp41-cli` per wave — faster than the rate at which generator logic changes.
- **Every phase requirement mapped:** HSGEN-01 (merge/pool/claude/batch units), HSGEN-02 (workspace-exclusion grep), HSGEN-03 (fill-only units + PR review), HSDATA-03 (batch-validation unit + run-summary coverage + Phase-61 gate), D-60.4/D-60.7 (writeback + build-smoke).
- **Non-deterministic surface fenced off:** the live LLM call is explicitly excluded from automated tests; only deterministic merge/parse/writeback logic is unit-tested.

---

*Phase: 60-alias-authoring-pipeline*
*Validation strategy derived: 2026-06-05*
