# Phase 36: hp41-gui — GUI Integration - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-23
**Phase:** 36-hp41-gui-gui-integration
**Areas discussed:** CATALOG 2 enumeration mechanism, STAT-GUI-05 cancellation scope, Plan slicing, STAT-GUI-05 bookkeeping

---

## CATALOG 2 enumeration mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Mirror bit-0 block for bit-1 | Add a parallel `if state.xrom_modules & 0b0000_0010 != 0 { ... STAT_1 ... }` block immediately after the MATH_1 block. ~10 LOC delta, no abstraction; matches current shape and CLAUDE.md "three similar lines beats premature abstraction" rule. Refactor to a generic loop when the third pac (v3.2 Time) actually lands. | ✓ |
| Refactor to iterate over [(MATH_1, bit), (STAT_1, bit)] | Build the enumeration as a slice/array walk now. Sets the template for v3.2+ pacs. ~15 LOC, introduces a tiny abstraction one milestone ahead of need. | |
| Registry-driven generic enumeration | Build a generic `enabled_xrom_modules(state.xrom_modules) -> &[&XromModule]` and walk it. Largest refactor; over-engineered for the current 2-module scope. Not recommended. | |

**User's choice:** Mirror bit-0 block for bit-1
**Notes:** Captured as D-36.1. Aligns with the v3.0 Phase 30 D-30.1 "binary stays 1-in/1-out, refactor when third invocation lands" precedent + CLAUDE.md "Don't add features, refactor, or introduce abstractions beyond what the task requires" rule. The two parallel `if` blocks stay grep-friendly. The defensive `NO XROM` else-fallback fires only when BOTH bits are clear (impossible post-`migrate_after_load`).

---

## STAT-GUI-05 cancellation scope

| Option | Description | Selected |
|--------|-------------|----------|
| Defer STAT-GUI-05 to v3.1.x / Phase 37 | Bounded 50-iter primitives don't need cancellation — they complete faster than a user can press R/S. Document in 36-CONTEXT that STAT-GUI-05 was reassessed at planning and deferred; the Phase 31 cancellation channel applies to open-ended SOLVE/INTG/DIFEQ paths only. Zero LOC delta; surface as a divergence/policy entry. | ✓ |
| Wire AtomicBool check anyway for pattern symmetry | Pass `cancel_requested: &AtomicBool` into the distribution-primitive callers (workflow layer where `&CalcState` is available) and check it BEFORE each primitive call. Per-loop checks INSIDE the primitives skipped (no point in 50-iter bounded loops). Mirrors D-28.8 at the workflow granularity. ~6 LOC delta in chisqd.rs / normd.rs workflow ops. | |
| Full per-loop AtomicBool check inside distributions.rs | Plumb `&AtomicBool` into `gser`/`gcf`/`betacf` and check inside the 50-iter loop. Pollutes pure-f64 primitive signatures for negligible benefit. Not recommended. | |

**User's choice:** Defer STAT-GUI-05 to v3.1.x / Phase 37
**Notes:** Captured as D-36.2. The reassessment is the correct engineering call, not a scope cut. Phase 31's `cancel_requested` channel was designed for OPEN-ENDED iterative paths (DIFEQ runs indefinitely per `difeq.rs:202`, INTG checks per-64-samples per `integ.rs:328-329`). Bounded ≤50-iter primitives at f64 precision complete in microseconds — cancellation is meaningless at that timescale. The bounded-vs-open-ended distinction is itself a v3.1 behavioral policy worth documenting (deferred to Phase 37 / planner discretion in Plan 36-03).

---

## Plan slicing

| Option | Description | Selected |
|--------|-------------|----------|
| 3 plans — arms+catalog, overlay+modal-flow, GUI tests | Plan 36-01: 26 op_display_name arms + CATALOG 2 core touch (closes CI break + STAT-GUI-01 + STAT-GUI-02 in one atomic landable commit). Plan 36-02: HelpOverlay third section + help_data.ts pool + modal-prompt flow (STAT-GUI-03 + STAT-GUI-04). Plan 36-03: vitest + Rust integration tests for the new GUI surface (lcd_alternation_modal_prompt.rs analog for Stat 1). Skip a dedicated STAT-GUI-05 plan if Question 2 = "Defer". | ✓ |
| 4 plans — split CATALOG 2 from arms | Plan 36-01: arms only (closes CI break). Plan 36-02: CATALOG 2 core touch. Plan 36-03: overlay + modal-flow. Plan 36-04: GUI tests. Cleaner blast-radius separation (CLAUDE.md "math1/ frozen" sensibility — core touches isolated to their own plan). | |
| 5 plans — dedicated STAT-GUI-05 cancellation plan | 4 plans above + Plan 36-05 for STAT-GUI-05 cancellation work. Only meaningful if Question 2 = "Wire AtomicBool" or "Full per-loop". | |

**User's choice:** 3 plans — arms+catalog, overlay+modal-flow, GUI tests
**Notes:** Captured as D-36.4. Phase 34 used 2 plans for structurally identical CLI work (26 entries, no core touch). Phase 36 grows by ONE plan vs. Phase 34 because of the STAT-GUI-05 bookkeeping (folded into 36-01) + the GUI-side multi-surface integration (HelpOverlay + help_data.ts + Rust modal-flow test in 36-02) + the vitest extensions (36-03). Plan 36-01 lands the requirements/roadmap reassignment in the first commit (D-36.3), then the 26 arms, then the `op_catalog` core touch — three commits in one plan, ordered for surgical revert windows.

---

## STAT-GUI-05 bookkeeping

| Option | Description | Selected |
|--------|-------------|----------|
| Reassign STAT-GUI-05 to Phase 37 | Update REQUIREMENTS.md traceability table (row 222) Phase 36 → Phase 37; add a one-line rationale in REQUIREMENTS.md AND ROADMAP.md Phase 36 success criterion #4 noting bounded-iter primitives "do not require cancellation"; mirror the v3.0 D-32.7 STAT-QUAL-09 reassignment pattern (which moved 37 → 33). Phase 37 picks it up alongside the other quality gates. | ✓ |
| Keep STAT-GUI-05 in Phase 36, mark NOT APPLICABLE in VERIFICATION.md | Phase 36 ships 4/5 STAT-GUI requirements; verification doc records STAT-GUI-05 as N/A with the bounded-iter rationale. Leaves the requirement assigned to Phase 36 but unmet — inconsistent with the v3.0 pattern where requirements either land in their phase or get reassigned. | |
| Remove STAT-GUI-05 entirely from REQUIREMENTS.md | If bounded primitives don't need cancellation, the requirement was misspecified — delete it with a one-line note in the project changelog. Cleanest but loses the historical context that this was assessed and rejected. | |

**User's choice:** Reassign STAT-GUI-05 to Phase 37
**Notes:** Captured as D-36.3. The reassignment lands as a single commit at the START of `/gsd-execute-phase 36` Plan 36-01 with commit message shape: `docs(36): reassign STAT-GUI-05 Phase 36 → Phase 37 per CONTEXT D-36.2`. Mirrors v3.0 D-32.7 cadence exactly. Keeping the requirement with the bounded-iter rationale (vs. deleting it) preserves the educational context for future archaeologists — when v3.2 Time Pac iterative interpolation might genuinely need cancellation, the rationale will guide the design.

---

## Claude's Discretion

The user explicitly endorsed the recommended options on every gray area, so no "you decide" deferrals from the user side. CONTEXT.md `### Claude's Discretion` block captures the secondary implementation choices the planner picks at write time:

- HelpOverlay `expanded` state shape — extend discriminated record literal vs. `Record<>` (recommend extend; planner can flip if test ergonomics suggest)
- Default expanded state for the new `'stat1'` section — `true` (open) on overlay mount per Phase 31-04 D-31.8 precedent
- `SECTIONS` `id` union type widening — explicit literal union `'hp41cv' | 'math1' | 'stat1'` (not `string`)
- `SectionDef.predicate` for Stat 1 — `e.xrom?.module === 'Stat 1'` matching D-34.1 JSON value
- vitest test file naming + Rust integration test file naming (lcd_alternation_modal_prompt_stat1.rs vs phase36_modal_flow_stat1.rs)
- CATALOG 2 test name (`catalog_2_lists_stat1_when_bit1_set`)
- File header comment update count in `prgm_display.rs`
- Plan 36-01 / 36-02 / 36-03 commit ordering (recommended in CONTEXT.md)
- Whether to add a `D-35-NN` Behavioral Policies entry in `docs/hp41-stat1-divergences.md` for the bounded-iter cancellation policy (Plan 36-03 or post-ship)

## Deferred Ideas

(See CONTEXT.md `<deferred>` section for the full list — summary here for the audit log.)

- STAT-GUI-05 cancellation in Stat 1 paths — REASSIGNED to Phase 37 (NOT removed)
- `docs/hp41-stat1-divergences.md` bounded-iter policy entry — Phase 36 ship-time or post-ship
- CLAUDE.md Phase 36 sub-section under `### v3.1 additions` — Phase 36 ship-time
- `docs/architecture-history.md` Phase 36 narrative — Phase 36 ship-time
- PROJECT.md Shipped / Current focus updates — Phase 36 ship-time
- WebdriverIO E2E Stat 1 smoke — Phase 37 / STAT-QUAL-11
- `numerical_accuracy.rs` Stat 1 oracle cases — Phase 37 / STAT-QUAL-04
- Per-`stat1/*.rs` coverage floor — Phase 37 / STAT-QUAL-03
- `stat1_op_test_count.rs` + `lint_stat1_assertions.rs` + `xrom_shadowing.rs` extension — Phase 37
- v3.0 save migration backward-compat test — Phase 37 / STAT-QUAL-10
- CATALOG 2 generic loop refactor — deferred to v3.2+ (third XROM module)
- HelpOverlay `Record<SectionId, boolean>` generic refactor — deferred to v3.2+
- Signed binary releases — v3.1.x / v3.2
- Cross-pac divergence-doc index — v3.2
- `/gsd-complete-milestone` v3.1 ship — Phase 37 post-ship
