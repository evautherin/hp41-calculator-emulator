# Phase 34: hp41-cli — CLI Integration - Context

**Gathered:** 2026-05-22
**Status:** Ready for planning

<domain>
## Phase Boundary

`hp41-cli` only. Wire the Phase 33 STAT_1 XROM module (id = 2, bit 1) into the TUI so that every Stat 1 Pac mnemonic is reachable, discoverable, and program-displayable from the CLI. Five concrete consequences:

1. `docs/hp41-stat1-functions.json` is authored in full (26 entries — one per row in `hp41-core/src/ops/math1/xrom.rs::STAT_1.ops`, in the same order, with per-family categories per D-34.1).
2. `hp41-cli/src/help_data.rs` grows a **third** `OnceLock<Vec<HelpEntry>>` over the new compile-time-embedded JSON. The existing `help_entries_all()` chain (built-ins → Math 1) extends to three pools (built-ins → Math 1 → Stat 1) per D-34.4. The narrow per-pool accessors (`help_entries()`, `help_entries_math1()`) stay intact for per-file smoke tests; a new `help_entries_stat1()` is added for the Phase 34 smoke test analog.
3. `hp41-cli/src/prgm_display.rs::op_display_name()` gains the missing arms for the 24 new `Op` variants Phase 33 introduced (`Op::SigmaBstat`, `Op::SigmaBstg`, `Op::SigmaMmtug`, `Op::SigmaMmtgd`, `Op::SigmaAovone`, `Op::SigmaAovtwo`, `Op::SigmaAnocov`, `Op::SigmaLin`, `Op::SigmaExp`, `Op::SigmaLogi`, `Op::SigmaPow`, `Op::SigmaMlrxy`, `Op::SigmaMlrxyz`, `Op::SigmaPolypWorkflow`, `Op::SigmaPolyc`, `Op::SigmaPtst`, `Op::SigmaTstat`, `Op::SigmaXsqev`, `Op::SigmaEfxsq`, `Op::SigmaCtkkk`, `Op::SigmaCtkk`, `Op::SigmaSpear`, `Op::SigmaNormdWorkflow`, `Op::SigmaChisqdWorkflow`, `Op::Rand`, `Op::Seed`). The intentional sanctioned `non-exhaustive patterns` CI break in `hp41-cli` (recorded in `.planning/STATE.md` after Phase 33 ship) closes when these arms land. 4-way exhaustive-match invariant item 3 (CLI) is sealed by this phase; item 4 (GUI `hp41-gui/src-tauri/src/prgm_display.rs`) remains intentionally broken until Phase 36.
4. `hp41-cli/tests/function_matrix_parity.rs` extends from the 2-pool walk (built-ins + Math 1) to a 3-pool walk (built-ins + Math 1 + Stat 1), keeping per-pool failure messages surgical so a future v3.2 Time/Advantage extension does not regress prior pools' assertions.
5. The `?` overlay grows a new "Stat 1 Pac (XROM 2)" section per D-34.5, rendered in fixed order (built-ins → Math 1 → Stat 1) per D-34.6.

**In scope:**
- `docs/hp41-stat1-functions.json` full 26-entry authoring (schema mirror of `hp41-math1-functions.json` plus the C-28.3 `xrom: { module: "Stat 1", module_id: 2, function_id: <n> }` block per entry, 1-indexed)
- Third `OnceLock<Vec<HelpEntry>>` + per-file panic message `"hp41-stat1-functions.json is malformed — fix the JSON"` matching the D-25.17 hard-build-blocker pattern
- `help_entries_stat1()` narrow accessor + `help_entries_all()` extension chaining all three pools
- 26 new `op_display_name` arms covering every Phase-33 `Op::Sigma* / Op::Rand / Op::Seed` variant — exhaustive, no `_ =>` catch-all (FN-CLI-04 invariant)
- `function_matrix_parity.rs` extension to 3 pools (planner reuses the Phase 29 partition-by-`xrom` approach)
- `phase25_xeq_by_name.rs::cli_resolver_matches_core_resolver` extension with a Stat 1 name case (e.g. `XEQ "ΣNORMD"`) to assert local resolver and core resolver still agree with `xrom_modules = 0b0000_0011`
- New `phase34_help_data_stat1.rs` smoke test (analog of `phase29_help_data_math1.rs`) asserting 26 entries load, all `op_variant` strings resolve, no duplicates against the two preceding pools
- New `phase34_modal_flow.rs` smoke test asserting `state.modal_prompt` is set correctly by `ΣNORMD` (`ΣNORMD MODE?`), `ΣCHISQD` ν step (`ν=?`), `ΣPOLYP` (`DEGREE=?`), `SEED` (`SEED?`) — verifies the Phase 29 modal-prompt rendering pipeline carries Stat 1 prompts without modification
- New `phase34_key_ref_includes_stat1.rs` smoke test asserting Stat 1 entries with non-null `key_path` appear in `keys::key_ref_entries()` after the merged accessor migration (parallel to `phase29_key_ref_includes_math1.rs`)
- Selective per-entry `divergences` field population per D-34.3 (RAND/SEED, ΣTSTAT, ΣPOLYP only — full catalog deferred to Phase 35)

**Out of scope (explicit):**
- Any `hp41-core` source changes — Phase 33 already shipped every Op variant, `xrom_resolve` already returns Stat 1 hits, `ModalProgram::Stat1` dispatch is plumbed, `submit_modal` / `cancel_modal` / `requires_alpha_label` already delegate correctly. Phase 34 reads from this surface; it does NOT modify it. The math1/ freeze remains in force (D-33.3 / D-33.3b carve-outs were Phase 33 only).
- Any `hp41-gui` source changes — Phase 36 owns GUI mirroring (the symmetric `op_display_name` arms + parallel-load help-overlay section + CATALOG 2 update). The GUI's `non-exhaustive patterns` CI break stays in place until Phase 36 closes it. Phase 34 verifies via `cargo check -p hp41-cli` and `just ci` (CLI-only path), NOT via `just gui-ci`.
- `scripts/docs-matrix/` three-input extension + `docs/hp41-stat1-function-matrix.md` regeneration — Phase 35 / STAT-DOC-02 + STAT-DOC-03. Phase 34 ships the canonical JSON; Phase 35 wires the matrix generator.
- `docs/hp41-stat1-divergences.md` three-bucket catalog — Phase 35 / STAT-DOC-01. Phase 34 only seeds the 3 surgical per-entry JSON divergence notes per D-34.3; the full catalog lives in the Phase 35 doc.
- New ADRs (RNG-state placement, distribution-primitive policy, ANOVA register layout) — Phase 35 / STAT-DOC-04. The decisions are recorded in Phase 33's `33-CONTEXT.md` / `33-SPEC.md`; Phase 35 writes the human-readable documents.
- README v3.1 section + CLAUDE.md `### v3.1 additions` block — Phase 35 / STAT-DOC-05.
- WebdriverIO E2E smoke extension + numerical-accuracy suite extension + coverage gates — Phase 37 / STAT-QUAL-04, STAT-QUAL-11.
- `xrom_shadowing.rs` STAT_1 cross-check + `stat1_op_test_count.rs` + `lint_stat1_assertions.rs` — Phase 37 / STAT-QUAL-06, STAT-QUAL-07, STAT-QUAL-08. (Phase 33 already added `stat1_ops_mnemonics_resolve_consistently` to `math1/xrom.rs` tests; the broader shadowing matrix lands in Phase 37.)
- Signed binary releases — deferred to v3.1.x / v3.2 per `.planning/PROJECT.md` lock 2026-05-21.

**Mandated by ROADMAP cross-cutting constraints + CLAUDE.md frozen invariants:**
- **SC-4 invariant** trivially preserved — Phase 34 touches `hp41-cli/` + `docs/` only; no `hp41-gui/src-tauri/src/op_*` functions added.
- **`#![deny(clippy::unwrap_used)]`** continues to apply throughout `hp41-core`. Phase 34 does not touch core; CLI new code may use `.unwrap()` only in `#[cfg(test)]` modules per established pattern.
- **`pending_input` routing block stays ABOVE modal-opening interceptors** in `hp41-cli/src/app.rs` (D-07 never-discard). Phase 34 does NOT reorder app.rs — Phase 29 already routed the modal interception correctly and ModalProgram::Stat1 flows through the same submit_modal / cancel_modal entry points without code change. Phase 34 reads `state.modal_program.is_some()` exactly as Phase 29 wired it.
- **CLI ↔ GUI parity (D-25.6):** every behavior the CLI gains here routes through shared `hp41-core` code (`xrom_resolve` already returns Stat 1, `submit_modal` already dispatches `ModalProgram::Stat1`, `requires_alpha_label` already returns false for all 5 Stat1Step variants). Phase 36 mirrors the *frontend* surface verbatim from this phase; the *core* surface is shared with zero new wiring.
- **MSRV 1.88** unchanged. Zero new runtime dependencies in `hp41-core` or `hp41-cli`.
- **Frozen Invariants — JSON canonical data flow** (CLAUDE.md): `docs/hp41-stat1-functions.json` is the single source of truth for the new help-overlay section + the eventual Phase 35 function matrix. Bidirectional Op ↔ JSON parity preserved by `function_matrix_parity.rs` extension.

</domain>

<decisions>
## Implementation Decisions

### Already locked in PROJECT.md / REQUIREMENTS.md / 33-CONTEXT.md / 33-SPEC.md (carried forward — NOT re-decided here)

- **D-28.4 / XROM-09 / D-33.3b:** `modal_prompt: Option<String>` is the channel for prompt strings; `state.print_buffer` continues to carry only PRX/PRA/PRSTK. Phase 34 reads `modal_prompt` (already routed via `pending_prompt()` per D-29.3); no new field plumbing.
- **D-28.5 / D-29.5:** R/S submits numeric input in a modal prompt. Phase 29 already wired the CLI interception → `submit_modal(state)` → `ModalProgram::Stat1` dispatch. Phase 34 does not touch this path.
- **D-28.6:** XEQ-by-name only — no dedicated key bindings for Stat 1 functions. JSON `key_path` is `"XEQ \"ΣNORMD\""`, `"XEQ \"ΣSPEAR\""`, `"XEQ \"RAND\""`, etc. throughout.
- **D-29.1:** JSON authored in this phase, NOT deferred to the docs phase. Phase 34 mirrors Phase 29's authoring-timing decision: SC-driven acceptance criteria (the `?` overlay + key_ref_entries + parity tests) need real entries.
- **D-29.2:** Multi-pool `OnceLock` + merged accessor pattern. Phase 34 extends to a third pool with the SAME per-file panic message convention.
- **D-29.7 / D-29.8 / D-29.9:** The `XeqByName { acc, mode: XeqByNameMode }` infrastructure + post-dispatch auto-open hook is reused unchanged. All 5 Stat1Step variants (`NormdModeChoice`, `ChisqdNuPrompt`, `ChisqdModeChoice`, `PolypDegreePrompt(_)`, `SeedPrompt`) are NUMERIC-input steps — `crate::ops::stat1::modal::requires_alpha_label` returns `false` for all of them, so the `CollectForModal` auto-open never fires for Stat 1. Confirmed by reading `hp41-core/src/ops/stat1/modal.rs:353-361`.
- **D-33.3 / D-33.3b:** math1/ freeze carve-outs are PHASE 33 ONLY. Phase 34 does not extend either carve-out; ALL math1/ files remain frozen for Phase 34's purposes.
- **STAT-CLI-01:** `xeq_by_name_local_resolve` final-fallback into `xrom_resolve` was wired in Phase 29 (line 382). It already returns Stat 1 hits via the bit-1 arm Phase 33 activated. Phase 34 verifies the fall-through still fires AFTER built-in resolution via a sibling test case in `phase25_xeq_by_name.rs::cli_resolver_matches_core_resolver`.
- **STAT-CLI-05:** Modal-prompt routing for Stat 1 multi-step workflows reuses existing `print_buffer` + `modal_program` infrastructure — no new transient `CalcState` fields required beyond `rand_seed` (which Phase 33 already added).

### Discussed and decided in this session (D-34.1 — D-34.6)

#### JSON category granularity

- **D-34.1: `docs/hp41-stat1-functions.json` uses 7 per-family categories.** Each entry's `category` string is one of: `"Stat1 Univariate"` (4 entries: ΣBSTAT, ΣBSTG, ΣMMTUG, ΣMMTGD), `"Stat1 ANOVA"` (3 entries: ΣAOVONE, ΣAOVTWO, ΣANOCOV), `"Stat1 Regression"` (8 entries: ΣLIN, ΣEXP, ΣLOGI, ΣPOW, ΣMLRXY, ΣMLRXYZ, ΣPOLYP, ΣPOLYC), `"Stat1 Hypothesis"` (2 entries: ΣPTST, ΣTSTAT), `"Stat1 Nonparam"` (5 entries: ΣSPEAR, ΣXSQEV, ΣEFXSQ, ΣCTKKK, ΣCTKK), `"Stat1 Distributions"` (2 entries: ΣNORMD, ΣCHISQD), `"Stat1 RNG"` (2 entries: RAND, SEED). 7 × varies = 26 entries total.
  - **Why:** Mirrors Phase 29 / D-29.1 per-program category convention (Math1 Hyperbolics / Math1 Complex / Math1 Matrix / etc.). Finer grouping helps discoverability; `help_overlay_rows` groups by first-appearance category order so Stat 1 sections cluster together visually. Rejected the flat `"Stat 1 Pac"` single category because it loses the family clustering Phase 29 specifically engineered as a discoverability win. Rejected OM-section-labels because the QRC labels are inconsistent with the `Math1 X`-prefixed convention already in `hp41-math1-functions.json`.

#### Per-entry `divergences` field policy

- **D-34.3: Surgical inline `divergences` only for entries with user-visible quirks; full taxonomy deferred to Phase 35.** Specifically:
  - `RAND` and `SEED`: `divergences: ["v3.1 emulator extension — not part of OM 00041-90030 'feature-complete' claim. LCG formula r_{n+1} = FRC(9821·r_n + 0.211327) per NPS p. 21 / Don Malm community convention."]`
  - `ΣTSTAT`: `divergences: ["Pooled-variance two-sample t-test only; Welch's unequal-variance variant is explicitly excluded per REQUIREMENTS.md Out-of-Scope (Phase 33 SPEC.md Req. 25)."]`
  - `ΣPOLYP`: `divergences: ["DEGREE=? modal prompt is HP-41 emulator convention transcribed from Math Pac I POLY precedent (OM 00041-90030 does not specify the degree-entry interaction)."]`
  - All other 22 entries: `divergences: None` (serialized as missing key per `#[serde(default)]` on the field).
  - **Why:** Mirrors Math Pac I JSON shape — Phase 29 left most entries' `divergences` empty and let the per-program divergence catalog (`docs/hp41-math1-divergences.md`) carry the full taxonomy. Avoids drift between JSON and the Phase 35 `docs/hp41-stat1-divergences.md` — if a divergence note needs to be updated, only one file changes. Surfaces the most user-impacting divergences inline (RAND/SEED authenticity claim, ΣTSTAT variance assumption, ΣPOLYP UX) where users actually look first (the `?` overlay).

#### Plan slicing

- **D-34.2: Phase 34 ships as 2 plans.** Tight low-blast-radius shape; the CLI break either closes atomically or stays open with a clear test/parity gate.
  - **Plan 34-01 (JSON + OnceLock + per-pool smoke):** author `docs/hp41-stat1-functions.json` (26 entries, 7 categories per D-34.1, surgical divergences per D-34.3) + add `STAT1_FUNCTIONS_JSON` `include_str!` + `STAT1_HELP_ENTRIES: OnceLock<Vec<HelpEntry>>` + `help_entries_stat1()` accessor with `"hp41-stat1-functions.json is malformed — fix the JSON"` panic message + extend `help_entries_all()` to chain all three pools + add `phase34_help_data_stat1.rs` smoke test (analog of `phase29_help_data_math1.rs`). End state: `hp41-cli` builds with 26 entries loadable; the `non-exhaustive patterns` CI break in `prgm_display.rs` is still open. This plan can land as a green commit before Plan 34-02 begins.
  - **Plan 34-02 (arms + parity + modal-flow tests):** add 26 `op_display_name` arms to `hp41-cli/src/prgm_display.rs` for the new `Op::Sigma* / Op::Rand / Op::Seed` variants + extend `tests/function_matrix_parity.rs` to walk all three JSON pools (partition by `xrom` field presence: None → built-ins pool, `module_id = 7` → Math 1 pool, `module_id = 2` → Stat 1 pool) + extend `tests/phase25_xeq_by_name.rs::cli_resolver_matches_core_resolver` with a Stat 1 name case + add `tests/phase34_key_ref_includes_stat1.rs` smoke (analog of `phase29_key_ref_includes_math1.rs`) + add `tests/phase34_modal_flow.rs` smoke asserting `state.modal_prompt` carries the OM strings for the 5 Stat1Step variants (`ΣNORMD MODE?`, `ν=?`, `ΣCHISQD MODE?`, `DEGREE=?`, `SEED?`). End state: `cargo check -p hp41-cli` clean (no `non-exhaustive patterns` warning), `just ci` green for the CLI path, 4-way invariant item 3 sealed.
  - **Why 2 vs 3 plans:** Phase 29 used 3 plans for ~55 Math 1 entries; Phase 34 has 26 entries and zero new core public surface, so the work is structurally smaller. A single plan (option A) was rejected because it would land the JSON + the arms + the parity tests in one atomic commit — harder to bisect if a regression sneaks in, and the JSON-only smoke test cannot be exercised in isolation. Three plans (Phase 29 mirror) was rejected as per-plan overhead heavier than the work warrants.

#### Help-overlay section header naming

- **D-34.5: `?` overlay section header reads `"Stat 1 Pac (XROM 2)"`.** Direct parallel to the existing `"Math 1 Pac (XROM 7)"` header. Same prose style (human-readable module name + parentheses + XROM ID). Continues the pattern as future Time / Advantage / Stat 2 Pacs land in v3.2+ (`"Time Pac (XROM <n>)"`, etc.).
  - **Why:** Zero discretion drift — the header convention is the part users learn once and apply across modules. Rejected `"STAT 1B (XROM 2)"` because it uses the CATALOG 2 display name (`STAT_1.name`) rather than the human-readable module name, diverging from Math Pac I shape. Rejected the belt-and-suspenders `"Stat 1 Pac (XROM 2 — STAT 1B)"` because it risks overflowing the 80-char overlay row width and adds discoverability noise.

#### `?` overlay search-result ordering

- **D-34.6: Three sections rendered in fixed insertion order: Built-ins → Math 1 Pac (XROM 7) → Stat 1 Pac (XROM 2).** Search ranks all three pools equally on the same match score; section headers separate them visually. The order is the natural `help_entries_all()` chain order: `help_entries().iter().chain(help_entries_math1().iter()).chain(help_entries_stat1().iter())`.
  - **Why:** Matches Phase 29 D-29.2's chain-order convention. No new sort logic required. Rejected alpha-by-section-header because it would today yield the same effective order but require a sort pass that becomes load-bearing once v3.2+ adds more XROM modules (and an alpha sort would push `Advantage Pac` before `Math 1 Pac`, fragmenting users' learned mental model). Rejected flatten-by-search-score because it loses the XROM-section discoverability Phase 29 D-29.1 specifically engineered and would require new sort + section-render logic Phase 34 doesn't need.

### Claude's Discretion

- **`function_id` indexing:** 1-indexed per HP-41 convention + Phase 29 precedent. Stat 1 `function_id` values run 1..=26 in the same row order as `STAT_1.ops` in `hp41-core/src/ops/math1/xrom.rs:144-185`. Planner has minor discretion if a reordering surfaces during JSON authoring (e.g., grouping by category in the JSON file ≠ row order in xrom.rs) — pick one ordering convention and document it in a one-line JSON comment block.
- **`xrom.module` string value:** `"Stat 1"` (mirroring Math Pac I's `"Math 1"`) versus `"STAT 1B"` (matching `STAT_1.name`). Recommendation: `"Stat 1"` for consistency with the Math Pac I JSON convention (the `XromEntry.module` field is human-readable, NOT the catalog display name). Planner picks; the `XromEntry` struct in `help_data.rs` does not currently validate this value, so either works.
- **`status` field value for all 26 Stat 1 entries:** `"implemented"`. All Ops shipped in Phase 33.
- **`phase` field value for all 26 Stat 1 entries:** `"33"` (the phase that delivered the Op variant). Planner sets uniformly.
- **`function_matrix_parity.rs` extension approach:** planner picks between (a) walk `help_entries_all()` once and partition by `xrom` field presence into 3 buckets, asserting each bucket against its matching Op-name pool — or (b) three parallel test functions (`builtin_parity`, `math1_parity`, `stat1_parity`) each walking its own pool. Recommendation: (a) for code reuse; per-pool failure messages stay surgical via match arms on `xrom.as_ref().map(|x| x.module_id)`.
- **`phase34_modal_flow.rs` test shape:** planner picks the assertion ergonomics. Recommended structure: 5 unit tests, one per Stat1Step variant, each constructing `CalcState { modal_program: Some(ModalProgram::Stat1(...)), .. }` and asserting `state.modal_program.as_ref().unwrap().current_prompt() == Some(<OM string>)` AND `pending_prompt(...)` renders the same string. Stretches the existing `phase29_pending_prompt_modal.rs` shape to Stat 1; reuses identical fixture pattern.
- **`phase34_help_data_stat1.rs` test shape:** mirrors `phase29_help_data_math1.rs` structurally — assert 26 entries load, every entry has non-empty `op_variant`, `display_name`, `category`, and `description`; assert no `op_variant` collisions against `help_entries()` or `help_entries_math1()`; assert every entry's `xrom.module_id == 2`. Planner picks any additional assertions (e.g., "all 7 categories appear at least once").
- **Inline `description` strings for all 26 entries:** ≤ 80 chars per D-25.16 convention. Sourced from the OM section header or NPS one-liner per entry. Recommended phrasing: lead with the program's primary purpose (`"Extended univariate summary (weighted mean, CV)"` for ΣBSTAT, `"Standard normal CDF / PDF / inverse"` for ΣNORMD, etc.). Planner has full discretion on wording; the smoke test verifies non-empty + length cap only.
- **Test file naming convention:** `phase34_help_data_stat1.rs`, `phase34_key_ref_includes_stat1.rs`, `phase34_modal_flow.rs` — mirrors `phase29_*` naming. Planner may alternatively prefix with `phase34_stat1_*` for cluster legibility; recommend keeping the `phase29_*`-style `<phase>_<topic>_<module>` for grep-ability.
- **Plan 34-02 commit ordering:** within Plan 34-02 the natural shape is (a) add arms → cargo check clean → (b) extend parity tests → (c) add modal-flow + key-ref smoke. Planner has discretion to fold (b) and (c) into one commit or split. Recommendation: one commit per logical step (3 small commits) so revert windows stay surgical.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-level (always-on)

- `.planning/PROJECT.md` — v3.1 milestone scope; v3.0 archived block; key decisions ledger
- `.planning/REQUIREMENTS.md` — 66 v3.1 requirements; Phase 34 maps to STAT-CLI-01..05 (rows 213–217 in the traceability table)
- `.planning/ROADMAP.md` — Phase 34 section lines 105–116 (4 success criteria); cross-cutting constraints carried forward from v3.0 ROADMAP archive
- `.planning/STATE.md` — accumulated context; "Intentional sanctioned CI break in hp41-cli/hp41-gui" note from Phase 33 ship
- `CLAUDE.md` (repo root) — Frozen Invariants section (4-way exhaustive-match invariant lines 71–80, JSON canonical data flow lines 122–129, resolver chain + never-discard D-07 lines 95–101); Tech Stack section; Key Files table for `hp41-cli/`

### Phase 33 (the contract Phase 34 builds on)

- `.planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-CONTEXT.md` — full Phase 33 decisions; D-33.3 / D-33.3b / D-33.4 / D-33.5 / D-33.7 / D-33.8 carry forward as Phase 34 read-only context
- `.planning/phases/33-…/33-SPEC.md` — 39 LOCKED Stat 1 Pac requirements, particularly Req. 35 (RAND), Req. 36 (SEED prompt `SEED?`), Req. 25 (ΣTSTAT pooled-variance), Stat1Step modal flow specifications
- `.planning/phases/33-…/33-RESEARCH.md` — Stat 1 Pac behavioral inventory; OM section page references the JSON `description` strings draw from
- `.planning/phases/33-…/33-PATTERNS.md` — closest-analog file mapping (Phase 34 reuses Phase 29's patterns verbatim)

### v3.0 Phase 29 (the direct analog — STRUCTURE TEMPLATE)

- `.planning/milestones/v3.0-phases/29-cli-integration/29-CONTEXT.md` — Phase 29 decisions D-29.1 through D-29.9; structurally identical phase one milestone earlier
- `.planning/milestones/v3.0-phases/29-cli-integration/29-01-PLAN.md` — Math Pac I JSON authoring plan (Phase 34 Plan 34-01 mirrors this verbatim, swapping Math 1 → Stat 1)
- `.planning/milestones/v3.0-phases/29-cli-integration/29-02-PLAN.md` — `op_display_name` arms + parity tests (Phase 34 Plan 34-02 mirrors this verbatim, swapping Math 1 → Stat 1)
- `.planning/milestones/v3.0-phases/29-cli-integration/29-03-PLAN.md` — modal-flow + auto-open hook (Phase 34 only mirrors the modal-flow smoke; the auto-open hook is Math-1-specific because all Stat 1 modal steps are numeric)

### hp41-cli existing pattern reservoir (Phase 34 reuses these in-place)

- `hp41-cli/src/help_data.rs` — the two-`OnceLock` pattern Phase 34 extends to three; comments at lines 1–17 + 105–134 explain the D-25.16 / D-25.17 / D-25.18 invariants Phase 34 preserves
- `hp41-cli/src/keys.rs:382` — `xeq_by_name_local_resolve`'s final fallback into `xrom_resolve(name, state.xrom_modules)`; Phase 34 confirms it routes Stat 1 hits with `xrom_modules = 0b0000_0011` (no code change needed)
- `hp41-cli/src/keys.rs::key_ref_entries` — JSON-derived per D-25.18; Stat 1 entries with non-null `key_path` flow through automatically after Plan 34-01 lands the merged-accessor chain
- `hp41-cli/src/prgm_display.rs` — exhaustive `op_display_name()` match; Plan 34-02 adds 26 arms; comment at line 26 (`"Covers all 35 Op variants exhaustively — no non-exhaustive patterns warning."`) is stale and must be updated to the new count
- `hp41-cli/src/ui.rs::pending_prompt` — already widened in Phase 29 to read `state.modal_prompt`; renders ModalProgram::Stat1 prompts identically to Math 1
- `hp41-cli/src/app.rs::handle_key` — already intercepts R/S + Esc when `state.modal_program.is_some()`; routes to shared `submit_modal` / `cancel_modal`; ModalProgram::Stat1 dispatches via the same path with zero CLI-side code change
- `hp41-cli/tests/function_matrix_parity.rs` — 2-pool walk; Plan 34-02 extends to 3-pool walk (partition by `xrom` field)
- `hp41-cli/tests/phase29_help_data_math1.rs` — direct template for `phase34_help_data_stat1.rs`
- `hp41-cli/tests/phase29_key_ref_includes_math1.rs` — direct template for `phase34_key_ref_includes_stat1.rs`
- `hp41-cli/tests/phase29_modal_flow.rs` + `phase29_pending_prompt_modal.rs` — direct templates for `phase34_modal_flow.rs`
- `hp41-cli/tests/phase25_xeq_by_name.rs::cli_resolver_matches_core_resolver` — drift-detection test Plan 34-02 extends with a Stat 1 name case
- `hp41-cli/tests/key_coverage.rs` — FN-CLI-01 verifiable-closure test; Plan 34-02 verifies it still passes with 3 pools merged (no new test additions expected — `xrom`-tagged entries are excluded from the keyboard-reachability closure)

### hp41-core public surface Phase 34 consumes (read-only)

- `hp41-core/src/ops/math1/xrom.rs:141-186` — `STAT_1: XromModule` const + `STAT_1.ops` 26-entry slice (the authoritative ordering and mnemonic source for `docs/hp41-stat1-functions.json`)
- `hp41-core/src/ops/math1/xrom.rs:188-212` — `xrom_resolve(name, modules)` with the bit-1 arm Phase 33 activated
- `hp41-core/src/ops/math1/modal.rs:38-52` — `ModalProgram::Stat1(Stat1Step)` variant (the carrier-enum extension D-33.3b added)
- `hp41-core/src/ops/math1/modal.rs:54-105` — `ModalProgram::current_prompt` + `requires_alpha_label` dispatch arms that delegate to `stat1::modal`
- `hp41-core/src/ops/stat1/modal.rs:333-361` — `current_prompt(&Stat1Step) -> Option<String>` + `requires_alpha_label(&Stat1Step) -> bool` (returns `false` for ALL 5 Stat1Step variants — confirming Phase 34 does NOT need to extend the CollectForModal auto-open hook)
- `hp41-core/src/state.rs:173-180` — `rand_seed: HpNum` `CalcState` field with `#[serde(default)]` no-`skip` shape (Phase 34 does not interact with this field directly; it surfaces via SEED's modal flow only)
- `hp41-core/src/state.rs:387` — `migrate_after_load` bit-1 v3.0 → v3.1 migration (Phase 34 does not invoke or modify this; the migration runs on every `load_state` in `hp41-cli/src/persistence.rs` per D-33.7)
- `hp41-core/src/ops/mod.rs` — `Op` enum + `dispatch()` arms for all 26 new Stat 1 variants (the source list Phase 34's `op_display_name` arms enumerate)

### JSON pipeline (canonical pattern + schema source)

- `docs/hp41cv-functions.json` — v2.2 schema source; baseline for `xrom` field absence
- `docs/hp41-math1-functions.json` — Math 1 schema source; baseline for `xrom: { module, module_id, function_id }` block presence; Plan 34-01 mirrors structure for Stat 1
- `scripts/docs-matrix/` (standalone non-workspace crate) — currently consumes two JSON files; Phase 35 / STAT-DOC-02 extends to three-input. Phase 34 does NOT touch this crate (write the JSON; let Phase 35 wire the matrix regeneration).

### HP Stat 1 Pac primary sources (HP-copyrighted — DO NOT redistribute; OM-style citations only)

- HP-41C Stat 1 Pac Owner's Manual (HP 00041-90030, 1979) — primary source for every Stat 1 program's behavior; cited via `//!` headers in `hp41-core/src/ops/stat1/` and via Phase 35 / STAT-DOC-04 ADRs
- HP-41C Stat 1 Pac Quick Reference Card (HP 00041-90061, 1979) — entry-point catalog; the 26 mnemonics in `STAT_1.ops` derive from QRC's per-program rows + RAND/SEED emulator extensions per D-33.4
- NPS55-84-003 Stat Pac Naval Postgraduate School document — secondary source; cited for RAND/SEED community-convention provenance (p. 21–22), F/Binomial/Poisson Out-of-Scope justification (p. 42, 49), ΣTSTAT pooled-variance evidence (ZS-4/5)

### Project-local CLAUDE.md guidance

- CLAUDE.md `## Git Workflow` — commits use `/git-workflow:commit --with-skills` only; English-only subject + body
- CLAUDE.md `## Frozen Invariants → Core engine` — `hp41-core/src/ops/math1/` frozen with D-33.3 / D-33.3b carve-outs (Phase 34 does NOT touch any math1/ file)
- CLAUDE.md `## Frozen Invariants → 4-way exhaustive-match invariant` — items 3 (`hp41-cli/src/prgm_display.rs`) sealed by this phase; item 4 (`hp41-gui/src-tauri/src/prgm_display.rs`) remains broken until Phase 36
- CLAUDE.md `## Frozen Invariants → JSON canonical data flow` — third JSON file extends the pattern; `function_matrix_parity.rs` extension keeps the bidirectional invariant

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`hp41-cli/src/help_data.rs`** — the entire two-`OnceLock` + per-file panic message + merged accessor pattern is reusable verbatim. Plan 34-01 adds a third copy of the pattern with `STAT1_FUNCTIONS_JSON` / `STAT1_HELP_ENTRIES` / `help_entries_stat1()` and extends `help_entries_all()` to a 3-step `.chain()`.
- **`hp41-cli/src/keys.rs:382`** — `xeq_by_name_local_resolve` already falls through to `hp41_core::ops::math1::xrom::xrom_resolve(name, xrom_modules)`. With `default_xrom_modules() = 0b0000_0011` (Phase 33), Stat 1 mnemonics resolve automatically. Zero code change here.
- **`hp41-cli/src/keys.rs::key_ref_entries`** — JSON-derived per D-25.18. After Plan 34-01 merges the third pool into `help_entries_all()`, Stat 1 entries with non-null `key_path` (every Stat 1 entry has one — `"XEQ \"ΣNORMD\""`, etc.) automatically appear in the right-panel discoverability listing.
- **`hp41-cli/src/ui.rs::pending_prompt`** — Phase 29 widened it to render `state.modal_prompt` when `state.modal_program.is_some()`. `ModalProgram::Stat1(Stat1Step)` flows through identically; the rendered string is whatever `crate::ops::stat1::modal::current_prompt(step)` returns. Zero code change here.
- **`hp41-cli/src/app.rs::handle_key`** — R/S + Esc interception when `state.modal_program.is_some()` already routes to shared `submit_modal` / `cancel_modal`. ModalProgram::Stat1 dispatches via the same path (D-33.3b: `match self { ... ModalProgram::Stat1(step) => crate::ops::stat1::modal::submit_step(state, step.clone()) ... }`). Zero code change here.
- **`hp41-cli/src/app.rs::PendingInput::XeqByName { acc, mode }`** — Phase 29 added `mode: XeqByNameMode` field. Plan 34-02's `cli_resolver_matches_core_resolver` extension instantiates `PendingInput::XeqByName { acc: "ΣNORMD".to_string(), mode: XeqByNameMode::Normal }` — reuses the existing alpha-collection key handling verbatim.
- **`hp41-cli/tests/phase29_help_data_math1.rs`** — direct template for `phase34_help_data_stat1.rs`. Read its shape; swap `MATH1` → `STAT1`, `help_entries_math1()` → `help_entries_stat1()`, and the expected count `55` → `26`; keep the same assertions (non-empty fields, no duplicates, `xrom.module_id` check).
- **`hp41-cli/tests/phase29_key_ref_includes_math1.rs`** — direct template for `phase34_key_ref_includes_stat1.rs`. Same swap pattern.
- **`hp41-cli/tests/phase29_modal_flow.rs`** + **`tests/phase29_pending_prompt_modal.rs`** — direct templates for `phase34_modal_flow.rs`. Construct `ModalProgram::Stat1(Stat1Step::*)` instead of `ModalProgram::Matrix(MatrixInputStep::OrderPrompt)` etc.; assert `current_prompt()` returns the OM string ("ΣNORMD MODE?", "ν=?", "ΣCHISQD MODE?", "DEGREE=?", "SEED?").
- **`hp41-cli/tests/function_matrix_parity.rs`** — 4 parity tests Phase 29 extended to walk both JSON pools. Plan 34-02 extends to 3 pools; per-pool failure messages stay surgical.
- **`hp41-cli/tests/phase25_xeq_by_name.rs::cli_resolver_matches_core_resolver`** — drift-detection pattern. Plan 34-02 adds a Stat 1 name case (e.g. `"ΣNORMD"` resolves to `Op::SigmaNormdWorkflow` from both the CLI-local resolver and `hp41_core::ops::math1::xrom::xrom_resolve` with `xrom_modules = 0b0000_0011`).
- **`hp41-core/src/ops/math1/xrom.rs:141-186` `STAT_1.ops` slice** — the 26-row authoritative list. Plan 34-01's JSON authoring reads this slice as the source of truth (mnemonic strings, Op variants, ordering). Bidirectional parity (JSON ↔ STAT_1.ops) is asserted by the Plan 34-02 parity test extension.

### Established Patterns

- **JSON canonical pipeline (D-25.16/17/18):** `include_str!` + `OnceLock` + hard-build-blocker on malformed JSON + JSON-derived KEY_REF_TABLE + bidirectional parity test. Phase 34 mirrors per the third-file shape.
- **PendingInput hybrid struct-variants (D-25.11) + `XeqByName { acc, mode }` (D-29.8):** doesn't need extension. All 5 Stat1Step variants are numeric-input only, so the `CollectForModal` mode is irrelevant for Stat 1.
- **Op variants land before consumers** (CLAUDE.md): Phase 33 already landed all 26 `Op::Sigma* / Op::Rand / Op::Seed` variants and arms 1+2 of the 4-way invariant (`dispatch()` + `execute_op()`). Phase 34 closes arm 3 (`hp41-cli/src/prgm_display.rs`); Phase 36 closes arm 4 (`hp41-gui/src-tauri/src/prgm_display.rs`). The intentional `non-exhaustive patterns` warning is the visible failure mode that the closing arm landing turns green.
- **D-25.6 CLI ↔ GUI parity:** every Stat 1 modal behavior the CLI gains here routes through shared `hp41-core` code (`xrom_resolve`, `submit_modal`, `cancel_modal`, `requires_alpha_label`, `ModalProgram::Stat1` dispatch). Phase 36 frontend will call the SAME shared functions identically — no parallel implementations.
- **No `println!` / `eprintln!` in `hp41-core`** — Phase 34 doesn't touch core, so trivially preserved.
- **Per-Op `LiftEffect` declarations** — Phase 34 doesn't touch Op definitions; the lift effects were set in Phase 33.

### Integration Points

- **`help_entries_all()` extension:** single-line `.chain()` addition in `hp41-cli/src/help_data.rs` (the existing two-pool chain becomes three-pool). All three call sites (`help_overlay_rows`, `keys::key_ref_entries`, `tests::function_matrix_parity`) consume the merged iterator unchanged.
- **`op_display_name` arms:** 26 new arms inserted into `hp41-cli/src/prgm_display.rs`. The existing exhaustive match has no `_ =>` catch-all (line 26 confirms it); adding the arms turns the current `non-exhaustive patterns` compile warning green. The stale comment at line 26 (`"Covers all 35 Op variants exhaustively — no non-exhaustive patterns warning."`) must be updated to reflect the new total Op count.
- **`function_matrix_parity.rs` extension:** existing 2-pool walk extends to 3-pool; partition by `xrom.as_ref().map(|x| x.module_id)` into `None` (built-ins), `Some(7)` (Math 1), `Some(2)` (Stat 1). Per-pool failure messages must stay surgical for future v3.2 Time/Advantage extensions.
- **`cli_resolver_matches_core_resolver` extension:** existing test parameterized over `(name, expected_op)` cases. Add 1–2 Stat 1 cases; the test pulls `state.xrom_modules` from `default_xrom_modules()` which is `0b0000_0011` post-Phase-33 so no test-fixture adjustments needed.
- **CLI break closure point:** the moment Plan 34-02's 26 arms land, `cargo check -p hp41-cli` produces zero warnings; `just ci` exits 0 for the CLI path. GUI's `non-exhaustive patterns` warning remains until Phase 36 closes arm 4 — Phase 34 does NOT attempt to silence the GUI warning via `#[allow]` or similar; the warning is the sanctioned signal that Phase 36 still has work to do.

</code_context>

<specifics>
## Specific Ideas

- **JSON `key_path` format for all 26 Stat 1 entries:** `"XEQ \"<mnemonic>\""` with the Σ glyph encoded as `\u{03A3}` per Plan-28 / Plan-29 convention. Example: `"XEQ \"\u{03A3}NORMD\""` (renders as `XEQ "ΣNORMD"`). RAND and SEED use ASCII names: `"XEQ \"RAND\""`, `"XEQ \"SEED\""`.
- **JSON `xrom` block content per entry:** `{ "module": "Stat 1", "module_id": 2, "function_id": <1..=26> }` where `function_id` is 1-indexed and follows the row order in `hp41-core/src/ops/math1/xrom.rs:144-185` (ΣBSTAT = 1, ΣBSTG = 2, …, RAND = 25, SEED = 26).
- **JSON `op_variant` strings:** PascalCase Op variant names matching `Op::*` exactly: `"SigmaBstat"`, `"SigmaBstg"`, `"SigmaMmtug"`, …, `"SigmaPolypWorkflow"`, `"SigmaNormdWorkflow"`, `"SigmaChisqdWorkflow"`, `"Rand"`, `"Seed"`. Note the `Workflow` suffix on ΣNORMD / ΣCHISQD / ΣPOLYP because Phase 33 uses workflow dispatchers (not single-shot Op variants) for those three multi-step programs.
- **OM page citations for `description` fields:** draw from the `//!` Op-doc headers in `hp41-core/src/ops/stat1/*.rs` — they already cite the OM page numbers. Example: `"ΣBSTAT"` → `"Extended univariate summary (weighted mean, CV) — OM p. 11"`. Planner has discretion to omit the OM citation if it pushes the description over 80 chars.
- **Surgical divergences strings (D-34.3):** the three exact strings to populate are documented in D-34.3 above. Planner copies them verbatim.
- **Smoke-test entry-count assertions:** `phase34_help_data_stat1.rs` asserts `help_entries_stat1().len() == 26`. If a future Stat 1 Pac entry is added, the test fails loudly — same shape as `phase29_help_data_math1.rs::math1_entries_count_meets_55_target` which asserts 55.
- **Stat1Step modal-prompt strings (read from `hp41-core/src/ops/stat1/modal.rs:333-343`):**
  - `Stat1Step::NormdModeChoice` → `"\u{03A3}NORMD MODE?"` (renders as `ΣNORMD MODE?`)
  - `Stat1Step::ChisqdNuPrompt` → `"\u{03BD}=?"` (renders as `ν=?`)
  - `Stat1Step::ChisqdModeChoice` → `"\u{03A3}CHISQD MODE?"` (renders as `ΣCHISQD MODE?`)
  - `Stat1Step::PolypDegreePrompt(_)` → `"DEGREE=?"`
  - `Stat1Step::SeedPrompt` → `"SEED?"`
- **No CollectForModal mode entries needed:** the post-dispatch auto-open hook fires only when `requires_alpha_label() == true`. All 5 Stat1Step variants return `false`. Phase 34 verifies this remains the case via a regression assertion in `phase34_modal_flow.rs` (`assert_eq!(modal_program.requires_alpha_label(), false)` for each variant).

</specifics>

<deferred>
## Deferred Ideas

- **GUI mirroring of all Phase 34 work** — Phase 36 plans. Every shared core function call (`xrom_resolve` with `0b0000_0011`, `submit_modal`, `cancel_modal`) is reused; the 26 `op_display_name` arms in `hp41-gui/src-tauri/src/prgm_display.rs` close arm 4 of the 4-way invariant; the third JSON-import parallel-load section in the GUI `?` overlay mirrors Phase 34's CLI surface 1:1.
- **CATALOG 2 enumeration update** — Phase 36. The CATALOG 2 listing already enumerates registered XROM modules dynamically (Phase 31 wired it); `STAT_1` appears automatically once the GUI builds. No CLI-side work.
- **`docs/hp41-stat1-divergences.md` three-bucket catalog** — Phase 35 / STAT-DOC-01. Phase 34's surgical per-entry divergences (RAND/SEED, ΣTSTAT, ΣPOLYP) are seedlings; the full taxonomy of OM divergences (XROM-7 vs XROM-2 prefix convention, `MATH_1.id = 7` discrepancy lock, hand-coded primitives policy, RNG community-convention sourcing) lives in the Phase 35 doc.
- **`scripts/docs-matrix/` three-input extension + `docs/hp41-stat1-function-matrix.md` regeneration** — Phase 35 / STAT-DOC-02 + STAT-DOC-03. Phase 34 ships the JSON; Phase 35 wires the matrix regenerator.
- **README v3.1 section + CLAUDE.md `### v3.1 additions` block** — Phase 35 / STAT-DOC-05.
- **ADRs for v3.1 architectural decisions** — Phase 35 / STAT-DOC-04: RNG-state placement (non-skip serde), distribution-primitive policy (AS 239/63/241 hand-coded), ANOVA / multiple-regression register layout finalization.
- **`docs/architecture-history.md` v3.1 narrative** — Phase 35 / STAT-DOC-06.
- **`numerical_accuracy.rs` extension with Stat 1 oracle cases** — Phase 37 / STAT-QUAL-04.
- **Per-`stat1/*.rs` per-file coverage floor ≥ 90 %** — Phase 37 / STAT-QUAL-03.
- **`stat1_op_test_count.rs` + `lint_stat1_assertions.rs` + `xrom_shadowing.rs` STAT_1 extension** — Phase 37 / STAT-QUAL-06, STAT-QUAL-07, STAT-QUAL-08.
- **`stat1_rand_determinism.rs` serde-cycle test** — already landed in Phase 33 per the Phase 33 ROADMAP plan list (Plan 33-08 `stat1_rand_determinism.rs`). Phase 34 does not duplicate this.
- **Backward-compat test for v3.0 save migration** (`v30_save_loads_with_stat1_off`) — Phase 37 / STAT-QUAL-10.
- **WebdriverIO E2E smoke with one Stat 1 Pac workflow** — Phase 37 / STAT-QUAL-11.
- **Free42 contamination guard** — Phase 33 / Plan 33-00 already extended the script to 18 tokens covering both math1/ and stat1/ identifiers per CLAUDE.md line in PROJECT.md. Phase 37 re-verifies in CI context but adds no new tokens for Phase 34's scope.
- **Signed binary releases (cargo-dist CLI + tauri-action GUI)** — deferred to v3.1.x / v3.2 per PROJECT.md lock 2026-05-21.

</deferred>

---

*Phase: 34-hp41-cli-cli-integration*
*Context gathered: 2026-05-22*
