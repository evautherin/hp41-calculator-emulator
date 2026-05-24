# Phase 33: hp41-core — XROM Activation + Distribution Primitives + All Stat 1 Ops - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-22
**Phase:** 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1
**Areas discussed:** OM convention resolution, plan slicing, resolver location, RAND policy, file layout, oracle data, migration location, contamination guard timing

---

## OM Convention Resolution

| Option | Description | Selected |
|--------|-------------|----------|
| /gsd-spec-phase 33 first | Run /gsd-spec-phase 33 BEFORE plan-phase — locks OM-derived conventions as LOCKED requirements (SPEC.md) that planner + executor cannot override. P21 silent-wrong-answer trap mitigated by gate. | ✓ |
| Plan-phase --research-phase only | Skip spec-phase; rely on `/gsd-plan-phase 33 --research-phase` (already flagged in STATE.md) to do the OM deep-read. Conventions land inside the plan, not as a hard contract. | |
| Plan-0 OM transcription task | First plan of Phase 33 is a 'Read OM Storage Registers + transcribe conventions' task with no code changes — locks decisions inside the phase as a D-33-NN entry before any Op is written. | |
| User reads OM, fills CONTEXT.md | User reads the OM PDF now and provides register layout + prompt strings + variance convention + RAND verdict directly into CONTEXT.md. Skip spec-phase and research-phase entirely. | |

**User's choice:** /gsd-spec-phase 33 first
**Notes:** Pitfall 21 (Σ-register layout silent-wrong-answer trap) is the most dangerous risk in Phase 33. Locking the OM-derived conventions as a SPEC.md contract before plan-phase prevents downstream agents from guessing. Spec-phase deliverable resolves 7 items (5 listed in STATE.md "v3.1-Specific Decisions" PENDING rows + quantile convergence + Free42 stats identifiers).

---

## Plan Slicing

| Option | Description | Selected |
|--------|-------------|----------|
| 9 plans by build-step | Follow SUMMARY.md order: (0) contamination guard + OM transcription, (1) XROM framework + state migration, (2) distribution primitives + scipy oracle, (3) ΣNORMD + ΣCHISQD, (4) ΣSPEAR + ΣXSQEV/ΣEFXSQ, (5) ΣBSTAT/BSTG + ΣLIN/EXP/LOGI/POW, (6) ΣMMTUG/MMTGD + ANOVA + ΣCTKKK/CTKK, (7) ΣPTST/TSTAT, (8) ΣMLRXY/MLRXYZ + ΣPOLYP/POLYC + RAND/SEED if confirmed. Tight, low-risk PR sizes. | ✓ |
| 6 plans by category | Coarser: (1) Framework+migration+contamination, (2) Distribution primitives + ΣNORMD + ΣCHISQD, (3) Univariate + ΣSPEAR + ΣXSQEV/EFXSQ + ΣLIN/EXP/LOGI/POW, (4) ANOVA + ΣCTKKK/CTKK + ΣMMTUG/MMTGD, (5) Regression (ΣMLRXY/MLRXYZ/POLYP/POLYC) + Hypothesis tests, (6) RAND/SEED. Fewer PRs, larger blast radius per plan. | |
| 3 plans by risk profile | Maximum chunks: (1) All foundation — framework + state + contamination guard + distribution primitives + scipy oracle validated, (2) All program Ops + register layout, (3) RAND/SEED + edge cleanup. Fewest hand-offs; harder to review. | |
| Let planner decide | Don't lock plan count or slicing now — pass implementation order to the planner with goals + dependencies; it slices. | |

**User's choice:** 9 plans by build-step
**Notes:** Matches research SUMMARY.md "Implementation order within phase" verbatim. Tight blast radius per PR; reviewer can read each plan independently. Plan 33-00 has no Op code (contamination guard + OM transcription only) — isolates the guard work from any code that could trip it.

---

## Resolver Location (math1/ freeze exception)

| Option | Description | Selected |
|--------|-------------|----------|
| Break freeze for xrom.rs only | Extend math1/xrom.rs with `pub const STAT_1` + `stat1_resolve()` + bit-1 arm — the stub comment was always intended for v3.1+. Document the freeze exception in CLAUDE.md. Surgical, matches research recommendation, no refactor risk. | ✓ |
| Relocate xrom.rs to ops/xrom.rs | Move xrom registry out of math1/ to a neutral `hp41-core/src/ops/xrom.rs` location. Math1 freeze preserved strictly. Adds a refactor PR/plan (touches many import sites), but xrom registry stops being misfiled. | |
| Add separate ops/stat1/xrom.rs | Keep math1/xrom.rs frozen as-is. Add `ops/stat1/xrom.rs` with STAT_1 const + stat1_resolve(). Wire the bit-1 arm from `ops/mod.rs` (new top-level xrom_resolve wrapper). Decentralizes the resolver chain. | |

**User's choice:** Break freeze for xrom.rs only
**Notes:** The bit-1 stub comment at `math1/xrom.rs:134` explicitly anticipated v3.1+ extension. Surgical edit; freeze exception documented in CLAUDE.md "Frozen Invariants → Core engine" section during Phase 35 (docs). The remaining 12 files in `math1/` stay strictly frozen.

---

## RAND / SEED Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Implement as documented emulator extension | Keep STAT-RNG-01..04 as planned. STAT-RNG-04 already documents RAND/SEED as a v3.1 emulator extension (not OM-feature-complete claim). Persistence-surviving rand_seed is user-valuable; LCG formula is community-confirmed (NPS p. 21). | ✓ |
| Drop RAND/SEED entirely | Remove STAT-RNG-01..04 from Phase 33 scope if OM doesn't list it. Keep behavioral emulation pure (OM-only). Defer to a v3.1.x or v3.2 follow-up if user demand emerges. | |
| Split into a separate Phase 33b | Implement only the 13 OM-confirmed programs in Phase 33. If OM shows RAND absent, carve RAND/SEED into a Phase 33b emulator-extension sub-phase that lands after the OM-feature-complete core. | |
| Decide later (block on OM read) | Defer the decision to after the OM convention read (whatever method picked in 'OM lock' above). The OM result determines policy. | |

**User's choice:** Implement as documented emulator extension
**Notes:** STAT-RNG-04 in REQUIREMENTS.md already drafts RAND/SEED as a v3.1 emulator extension — divergences catalog (Phase 35) documents it as such. If OM verification turns up RAND as a top-level Stat 1 Pac ROM entry point after all, the implementation is identical; only the divergences-catalog bucket changes (D-33.4a covers this branch).

---

## `ops/stat1/` File Layout

| Option | Description | Selected |
|--------|-------------|----------|
| 9 files, rename tests.rs → hypothesis.rs | `mod.rs`, `distributions.rs`, `basic_stats.rs`, `anova.rs`, `regression.rs`, `hypothesis.rs` (ΣPTST/ΣTSTAT), `nonparam.rs` (ΣSPEAR/ΣXSQEV/ΣEFXSQ/ΣCTKKK/ΣCTKK), `normd.rs`, `chisqd.rs`. Avoids `tests.rs` collision with `#[cfg(test)] mod tests` blocks. | ✓ |
| Consolidate distributions (7 files) | Fold `normd.rs` + `chisqd.rs` into `distributions.rs` (one file holds primitives AND evaluator Ops). Smaller surface, easier to navigate; but distributions.rs grows to ~300 LOC. | |
| Flat structure (1 file per Op family) | One file per QRC program category, fewer levels of indirection: `mod.rs`, `xrom.rs` (STAT_1 const), `distributions.rs`, `basic_stats.rs`, `moments.rs`, `regression.rs`, `anova.rs`, `hypothesis.rs`, `contingency.rs`, `rand.rs`. Closer to math1/ shape. | |
| Let planner pick | Pass goals + dependencies to the planner; let it choose file granularity based on the actual Op clustering it discovers during planning. | |

**User's choice:** 9 files, rename tests.rs → hypothesis.rs
**Notes:** Phase 33 extends this to 10 files + `mod.rs` = 11 files by splitting moments out of basic_stats (different OM register requirements). The split keeps Plan 33-05 and Plan 33-06 disjoint. Final layout documented in D-33.5 of CONTEXT.md.

---

## scipy.stats Oracle Data Format

| Option | Description | Selected |
|--------|-------------|----------|
| Inline test constants in primitives.rs | Hard-code ~30 (input, scipy_expected) tuples directly in `#[cfg(test)] mod tests` of `stat1/distributions.rs`. Single-file review; oracle frozen at write time; no fixture file to drift. Mirrors how `math1/poly.rs` and `math1/integ.rs` carry their constants. | ✓ |
| JSON fixture in tests/fixtures/ | Generate `tests/fixtures/scipy-stat1-oracle.json` via a one-shot Python script, commit the JSON; tests load via `include_str!`. Decouples oracle data from test code. Adds a fixture-regen recipe; harder to audit drift. | |
| Python script in tests/oracle/ + just recipe | Commit `tests/oracle/gen_stat1_oracle.py` + auto-regenerate via `just gen-stat1-oracle`. Re-runnable. Adds Python toolchain dependency to the workflow (currently none). | |
| Hybrid: inline closed-form, JSON for iterative | Inline constants for closed-form CDF/PDF cases (1e-9 tolerance, deterministic). JSON fixture only for iterative Acklam/AS 239/AS 63 quantile cases (1e-7 tolerance, larger oracle set). | |

**User's choice:** Inline test constants in primitives.rs
**Notes:** Matches the Math Pac I oracle-constants pattern in `poly.rs` / `integ.rs`. No external Python toolchain dependency; oracle values frozen at write time via a one-shot fenced scipy.stats snippet in the plan comments. Tolerances follow the two-level policy from STAT-QUAL-05.

---

## State Migration Location

| Option | Description | Selected |
|--------|-------------|----------|
| hp41-core/state.rs | Add `CalcState::migrate_after_load()` (or extend the existing `Default + serde(default)` path) in `state.rs`. Migration runs in core, called once after deserialization by every consumer (CLI persistence, GUI persistence). Single source of truth; no UI duplication. | ✓ |
| Per-frontend in persistence.rs (cli + gui) | Duplicate the bit-1 set into `hp41-cli/src/persistence.rs::load_state` AND `hp41-gui/src-tauri/src/persistence.rs`. Closer to where files are read; risk: forget one path — v3.0 user opens cli vs gui gets different behavior. | |
| Hidden inside #[serde(default)] init | Use a custom deserializer that returns `0b0000_0011` when the field is missing OR equals exactly `0b0000_0001`. No re-save step; migration is invisible. Risk: user who explicitly set `xrom_modules: 1` (e.g. disabled Stat 1) gets overridden every load. | |

**User's choice:** hp41-core/state.rs
**Notes:** Single source of truth; CLI and GUI both call `migrate_after_load()` after deserialization. Idempotent — no-op once bit 1 is set. Re-save happens on the next 30s auto-save tick or exit save. Avoids the deserializer-hides-the-override problem of option 3.

---

## Free42 Contamination Guard Timing

| Option | Description | Selected |
|--------|-------------|----------|
| Move STAT-QUAL-09 to Phase 33 | Update REQUIREMENTS.md traceability: STAT-QUAL-09 reassigned Phase 37 → Phase 33. Guard extension is Plan 33-00 (or 33-01). Phase 37 still owns the rest of QUAL items. Matches research P27 ordering. | ✓ |
| Keep STAT-QUAL-09 in Phase 37, do guard work in Phase 33 anyway | Phase 33 plan-0 extends the script (preventative work); Phase 37 plan still re-verifies it in CI context as the formal acceptance gate. Slight redundancy but clean req mapping. | |
| Split STAT-QUAL-09 into 09a (Phase 33) + 09b (Phase 37) | STAT-QUAL-09a = 'extend script with stats identifiers' (Phase 33 plan-0). STAT-QUAL-09b = 'CI verification + greenfield check' (Phase 37). Tracks both halves explicitly. | |
| Keep as-is, accept the ordering risk | STAT-QUAL-09 stays in Phase 37. Phase 33 lands first stat1/*.rs files without guard extension. Risk: any Free42 stats-domain identifier accidentally pasted in early lands undetected until Phase 37 catches it weeks later. | |

**User's choice:** Move STAT-QUAL-09 to Phase 33
**Notes:** STAT-QUAL-09's own requirement text says "verified BEFORE first stat1/*.rs file lands" — the requirement itself orders the work into Phase 33 Plan 33-00. REQUIREMENTS.md traceability table will be updated during spec-phase or plan-phase. Phase 37 keeps STAT-QUAL-01..08, STAT-QUAL-10..11.

---

## Claude's Discretion

- Concrete identifier list for the contamination-guard extension (D-33.8): spec-phase resolves the candidate list from `core_math2.cc`; planner / executor finalizes the regex pattern.
- ANOVA / regression result-output shape (stack push vs print buffer vs both): defer to OM "Results" section read in spec-phase; if OM is silent, follow the Math Pac I `POLY` pattern.
- Per-Op `LiftEffect::Enable / Disable / Neutral` declarations: follow HP-41 stack-lift semantics and Math Pac I family precedents.
- Inline test count per Op: meet `stat1_op_test_count.rs` floor of ≥ 5; aim for ~6–8 to leave margin.

## Deferred Ideas

- ANOVA / regression result-output shape — pinned to spec-phase OM read
- Per-Op `LiftEffect` declarations — locked at plan time
- `docs/hp41-stat1-divergences.md` D-33-NN numbering scheme — Phase 35 work
- `statrs` re-evaluation if `rust_decimal` gains `MathematicalOps::norm_cdf_inv` — locked rejected for v3.1; revisit possible v3.2+
- Histogram / frequency-table program — out of scope per NPS; future milestone only
- Stat 2 Pac functions — separate HP module; future milestone after v3.x
