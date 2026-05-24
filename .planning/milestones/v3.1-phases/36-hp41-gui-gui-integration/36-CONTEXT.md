# Phase 36: hp41-gui — GUI Integration - Context

**Gathered:** 2026-05-23
**Status:** Ready for planning

<domain>
## Phase Boundary

`hp41-gui` (Tauri v2 + React frontend) + one surgical `hp41-core` touch for CATALOG 2 enumeration. Wire the Phase 33 STAT_1 XROM module (id = 2, bit 1) into the desktop GUI so that every Stat 1 Pac mnemonic is reachable, discoverable, and program-displayable from the GUI — mirroring the Phase 34 CLI surface 1:1 via the D-25.6 CLI ↔ GUI parity invariant. Three concrete consequences:

1. `hp41-gui/src-tauri/src/prgm_display.rs::op_display_name()` gains the missing arms for the 26 new `Op` variants Phase 33 introduced (the same 26 mnemonics Phase 34 Plan 34-02 added to the CLI: `Op::SigmaBstat` through `Op::Seed`). The intentional sanctioned `non-exhaustive patterns` CI break in `hp41-gui` (recorded in `.planning/STATE.md` after Phase 33 ship, still open after Phase 34) closes when these arms land. 4-way exhaustive-match invariant item 4 (GUI) is sealed by this phase.

2. `hp41-gui/src/HelpOverlay.tsx` `SECTIONS` array extends from two top-level collapsible sections (`'hp41cv'`, `'math1'`) to three (`'hp41cv'`, `'math1'`, `'stat1'`). New section heading is `"Stat 1 Pac (XROM 2)"` per D-34.5 (locked in Phase 34). Section render order is Built-ins → Math 1 → Stat 1 per D-34.6 (locked in Phase 34). `hp41-gui/src/help_data.ts` adds a third Vite static-imported JSON pool (`docs/hp41-stat1-functions.json`) alongside the existing `math1Functions` import; `helpEntriesAll()` extends to chain all three pools.

3. **One surgical `hp41-core` touch** — `hp41-core/src/ops/program.rs::op_catalog(state, 2)` currently hardcodes a bit-0 (`xrom_modules & 0b0000_0001 != 0`) conditional for MATH_1 enumeration; Phase 36 mirrors the existing block for bit-1 (`xrom_modules & 0b0000_0010 != 0`) to enumerate `STAT_1.name` + the 26 entries from `STAT_1.ops`. This is core code, not GUI code, but the user-visible effect is in BOTH the CLI and the GUI CATALOG 2 listings (the CLI gets it for free as a parity bonus). The `op_catalog` test suite extends with a `catalog_2_lists_stat1_when_bit1_set` case. ~10 LOC delta; no abstraction — mirrors the v3.0 "three similar lines beats premature abstraction" CLAUDE.md guidance.

**In scope:**
- 26 new `op_display_name` arms in `hp41-gui/src-tauri/src/prgm_display.rs` covering every Phase-33 `Op::Sigma* / Op::Rand / Op::Seed` variant — exhaustive, no `_ =>` catch-all (matches CLI side D-34 arm-shape verbatim; copy from `hp41-cli/src/prgm_display.rs` Phase 34 work and adapt the file header comment count)
- `hp41-core/src/ops/program.rs::op_catalog` bit-1 conditional block parallel to the existing bit-0 block (~10 LOC); both blocks coexist; refactor to a generic loop deferred until v3.2 Time Pac actually lands per D-36.1
- `op_catalog` test suite extension — at minimum one new test asserting CATALOG 2 lists STAT_1 alongside MATH_1 when `xrom_modules = 0b0000_0011`; reuses existing test fixture pattern
- `hp41-gui/src/help_data.ts` third Vite static-import (`docs/hp41-stat1-functions.json`) + `helpEntriesStat1()` narrow accessor + `helpEntriesAll()` 3-pool chain extension
- `hp41-gui/src/HelpOverlay.tsx` `SECTIONS` array extension (third entry: `{ id: 'stat1', heading: 'Stat 1 Pac (XROM 2)', predicate: (e) => e.xrom?.module === 'Stat 1' }`); `expanded` state shape widens from `{ hp41cv: boolean; math1: boolean }` to `{ hp41cv: boolean; math1: boolean; stat1: boolean }` with all three defaulting to `true` (open) per existing Phase 31-04 D-31.8 behavior; section `id` union type widens to `'hp41cv' | 'math1' | 'stat1'`
- New vitest `HelpOverlay.test.tsx` cases asserting Stat 1 section renders, collapses, and search-filters correctly (mirrors existing Phase 31-04 D-31.8 / D-31.9 test shape)
- New Rust integration test `lcd_alternation_modal_prompt_stat1.rs` (analog of the existing `lcd_alternation_modal_prompt.rs`) asserting `CalcStateView::modal_prompt` carries the Stat 1 OM strings for the 5 Stat1Step variants — verifies the modal-prompt routing pipeline carries Stat 1 prompts without GUI-side modification (STAT-GUI-04 verification)
- New vitest test asserting `dispatch_op("XEQ ΣNORMD")` (or the equivalent key-id round-trip) returns a `CalcStateView` whose `modal_prompt` field carries `"ΣNORMD MODE?"` — end-to-end CLI-resolver-via-`xrom_resolve` test path

**Out of scope (explicit):**
- Any `hp41-core` source changes BEYOND the surgical `op_catalog` bit-1 block + its test — `xrom_resolve` already returns Stat 1 hits via the bit-1 arm Phase 33 activated, `submit_modal` / `cancel_modal` / `requires_alpha_label` already delegate correctly through `ModalProgram::Stat1` dispatch (Phase 33 / D-33.3b), `state.modal_prompt` is already plumbed end-to-end. Phase 36 reads from this surface; it does NOT modify it. The math1/ freeze remains in force (D-33.3 / D-33.3b carve-outs were Phase 33 only).
- Any `hp41-cli` source changes — Phase 34 sealed 4-way invariant item 3; Phase 36 sealing item 4 in the GUI is symmetric work that does NOT touch CLI files. Even though the `op_catalog` core change incidentally improves the CLI's CATALOG 2 output, that is a side-effect of the SHARED `hp41-core` code path, not a CLI source touch.
- **STAT-GUI-05** (`request_cancel` reuse for iterative-quantile paths in ΣNORMD inverse + ΣCHISQD CDF) — **reassigned to Phase 37 per D-36.2 / D-36.3.** Reassessment at planning revealed: `norm_cdf_inv_f64` is closed-form Acklam rational approximation (no iteration); `gamma_regularized_f64` uses bounded 50-iter `gser`/`gcf` per SPEC.md Req. 34 (microseconds-per-call worst case); both primitives are pure `f64 → Result<f64, HpError>` with no `&CalcState` access. The Phase 31 `cancel_requested` `Arc<AtomicBool>` channel was designed for user-driven OPEN-ENDED iterative paths (INTG/SOLVE/DIFEQ, per D-28.7 / D-28.8 — DIFEQ runs indefinitely until cancellation per `hp41-core/src/ops/math1/difeq.rs:202`). Bounded 50-iter primitives do not need cancellation — they complete faster than a user can press R/S. Phase 36 closes 4 of 5 STAT-GUI requirements (STAT-GUI-01..04); Phase 37 STAT-QUAL block picks up STAT-GUI-05 alongside the other quality gates with the bounded-iter rationale documented per D-35-NN behavioral-policy entry (see deferred section for the bookkeeping commits).
- `scripts/docs-matrix/` extensions, ADRs, divergence-catalog entries — all Phase 35 territory (already shipped 2026-05-23). Phase 36 consumes `docs/hp41-stat1-functions.json` read-only (just like Phase 35 Plan 35-01 did).
- WebdriverIO E2E smoke extension + numerical-accuracy suite extension + coverage gates — Phase 37 / STAT-QUAL-04, STAT-QUAL-11.
- `xrom_shadowing.rs` STAT_1 cross-check + `stat1_op_test_count.rs` + `lint_stat1_assertions.rs` — Phase 37 / STAT-QUAL-06, STAT-QUAL-07, STAT-QUAL-08.
- Signed binary releases — deferred to v3.1.x / v3.2 per `.planning/PROJECT.md` lock 2026-05-21.
- Generic XROM-registry-driven CATALOG 2 enumeration — premature abstraction at 2 modules per D-36.1; defer to the milestone that introduces a third module (Time Pac / Advantage Pac in v3.2+).

**Mandated by ROADMAP cross-cutting constraints + CLAUDE.md frozen invariants:**
- **SC-4 invariant** preserved — Phase 36 touches `hp41-gui/src-tauri/src/prgm_display.rs` (display-name strings only, the 26 new arms emit `match op { Op::SigmaBstat => "\u{03A3}BSTAT".to_string(), ... }` — no `op_*` math functions added) + `hp41-gui/src-tauri/src/` is otherwise read-only for this phase. The `cargo check` SC-4 grep (`grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/`) returns empty before and after Phase 36 lands.
- **4-way exhaustive-match invariant** — Phase 33 landed items 1+2 (`dispatch()` + `execute_op()`), Phase 34 sealed item 3 (`hp41-cli/src/prgm_display.rs`), Phase 36 seals item 4 (`hp41-gui/src-tauri/src/prgm_display.rs`). When Phase 36 Plan 36-01 lands, the GUI `non-exhaustive patterns` compile warning closes; `just gui-ci` exits 0 for the first time since Phase 33 ship.
- **`#![deny(clippy::unwrap_used)]`** continues to apply in `hp41-core` (the `op_catalog` extension uses `?`-propagation only, no `.unwrap()`); `hp41-gui/src-tauri/src/` uses `.unwrap_or_else(|e| e.into_inner())` for poisoned-lock recovery per the existing pattern (CLAUDE.md Core engine §No async, no panics) — Phase 36 does not introduce new mutex calls.
- **CLI ↔ GUI parity (D-25.6):** every Stat 1 behavior the GUI gains here routes through SHARED `hp41-core` code (`xrom_resolve` already returns Stat 1, `submit_modal` already dispatches `ModalProgram::Stat1`, `requires_alpha_label` already returns false for all 5 Stat1Step variants, `CalcStateView::from_state` already carries `modal_prompt`). The GUI gains zero parallel implementations; the 26 display-name arms are pure strings + the HelpOverlay third section is presentation-layer routing — both display surface only.
- **MSRV 1.88** unchanged. Zero new runtime dependencies in `hp41-core`, `hp41-cli`, or `hp41-gui`.
- **Frozen Invariants — JSON canonical data flow** (CLAUDE.md): `docs/hp41-stat1-functions.json` is consumed read-only by `hp41-gui/src/help_data.ts` (third Vite static import) — same hard-build-blocker shape as the CLI's `OnceLock` panic on malformed JSON (D-25.17). Vite's static-import failure at build time is the equivalent gate for the GUI surface (TypeScript compile error on the typed JSON import).
- **No polling (D-11):** Phase 36 does not introduce frontend polling of `get_state()`. The new vitest cases assert `dispatch_op` response's `CalcStateView.modal_prompt` field carries the OM strings; the existing `useState` + dispatch-response React pattern carries Stat 1 prompts identically to Math 1 prompts.
- **GUI persistence — shared with CLI**: `~/.hp41/autosave.json` is read/written by both surfaces; Phase 36 does NOT touch persistence. The Phase 33 `migrate_after_load` already activates Stat 1 (bit 1) on load via `state.rs:386-391` (D-33.7 single-source-of-truth) — GUI inherits this for free.
- **Bundle ID `ch.talent-factory.hp41`** preserved (Phase 13 D-02). Phase 36 does not touch `hp41-gui/src-tauri/tauri.conf.json`.
- **Tauri v2.11 inline-command permissions:** Phase 36 does not add new inline app commands; existing `dispatch_op`, `get_state`, `sst_step`, `bst_step`, `run_stop`, `request_cancel` permissions in `hp41-gui/src-tauri/permissions/*.toml` continue to cover the surface verbatim.

</domain>

<decisions>
## Implementation Decisions

### Already locked in PROJECT.md / REQUIREMENTS.md / 33-CONTEXT.md / 34-CONTEXT.md / 35-CONTEXT.md (carried forward — NOT re-decided here)

- **D-25.6 (CLI ↔ GUI parity):** every Stat 1 behavior the GUI gains here routes through SHARED `hp41-core` code. The 26 display-name arms in `hp41-gui/src-tauri/src/prgm_display.rs` are pure strings (Phase 34's CLI version `hp41-cli/src/prgm_display.rs` is the source-of-truth for the string content; Phase 36 copies them verbatim — the strings are deliberately duplicated CLI ↔ GUI by the 4-way invariant design).
- **D-28.4 / XROM-09 / D-33.3b:** `modal_prompt: Option<String>` is the channel for prompt strings; `state.print_buffer` continues to carry only PRX/PRA/PRSTK. Phase 36 reads `modal_prompt` via the existing `CalcStateView::from_state` plumbing (already routed end-to-end since Phase 31 / GUI-04); no new field plumbing.
- **D-28.5 / D-29.5:** R/S submits numeric input in a modal prompt. Phase 31 already wired the GUI interception → `submit_modal(state)` → `ModalProgram::Stat1` dispatch via the shared `commands.rs::sst_step` / `run_stop` paths. Phase 36 does not touch this path.
- **D-28.6:** XEQ-by-name only — no dedicated key bindings for Stat 1 functions. Stat 1 mnemonics resolve via `xeq_by_name` → shared `xrom_resolve` → Op variant in BOTH CLI and GUI.
- **D-29.7 / D-29.8 / D-29.9:** The `XeqByName { acc, mode: XeqByNameMode }` infrastructure + post-dispatch auto-open hook is reused unchanged from Phase 29 / Phase 34. All 5 Stat1Step variants are NUMERIC-input steps — `requires_alpha_label` returns `false` for all, so the `CollectForModal` auto-open never fires for Stat 1. The GUI's TypeScript `pending_input.ts` mirror of this state machine inherits the same behavior — Phase 36 does NOT need to modify it.
- **D-31.4 / D-31.6 (cancellation pattern):** INTG/SOLVE/DIFEQ check `cancel_requested` every 64 samples per `hp41-core/src/ops/math1/{integ,solve,difeq}.rs`. STAT-GUI-05 was originally specified to extend this pattern to ΣNORMD inverse + ΣCHISQD CDF. Reassessed in this discussion as not applicable (D-36.2 — bounded 50-iter primitives don't need cancellation; STAT-GUI-05 reassigned to Phase 37 per D-36.3).
- **D-31.8 (HelpOverlay shape):** Two top-level collapsible sections (Built-ins + Math 1 Pac) rendered in fixed order with per-section state. Phase 36 extends to THREE sections, preserves the existing collapsible-state-per-section convention, and preserves the existing "all sections default open" behavior. Search behavior continues to filter across all sections equally.
- **D-31.9 (HelpOverlay test shape):** vitest tests assert section presence, collapse/expand, and search filter. Phase 36 mirrors the existing assertion structure for the new third section.
- **D-31.12 / D-31.14 (no PSE-step in CATALOG):** CATALOG 2 pushes synchronously into `state.print_buffer` with no per-line yield. Phase 36 preserves this — the bit-1 conditional block adds STAT_1 entries to the same single-pass `print_buffer` push pattern as the existing bit-0 MATH_1 block.
- **D-33.3 / D-33.3b:** math1/ freeze carve-outs are PHASE 33 ONLY. Phase 36 does not extend either carve-out; ALL math1/ files remain frozen. (The `op_catalog` surgical touch lives in `hp41-core/src/ops/program.rs`, OUTSIDE the math1/ freeze boundary.)
- **D-33.7:** `CalcState::migrate_after_load()` in `hp41-core/src/state.rs` — single source of truth, called by both CLI and GUI persistence layers. Phase 36 does not touch this. Stat 1 (bit 1) is already activated on every `load_state` for any save file with `xrom_modules: 1` (D-33.7 v3.0 → v3.1 migration).
- **D-34.1 (JSON category convention):** 7 per-family Stat 1 categories already wired in `docs/hp41-stat1-functions.json` (Phase 34). Phase 36 consumes them read-only via the new `help_data.ts` Vite import; the HelpOverlay third-section render uses the JSON-declaration category order verbatim (matching the Math 1 section's category-order behavior).
- **D-34.5 (overlay section header):** `"Stat 1 Pac (XROM 2)"` — locked in Phase 34, used verbatim in Phase 36 `SECTIONS[2].heading`.
- **D-34.6 (overlay render order):** Built-ins → Math 1 → Stat 1 — locked in Phase 34, mirrored in Phase 36 `SECTIONS` array order.
- **D-35.5 (CLAUDE.md `### v3.1 additions` block):** Phase 36 sub-section currently shows `(in progress)` per the Phase 35 stub. Phase 36 ship-time fills in the Phase 36 narrative (paralleling D-30.8 incremental-population pattern); the full sub-section content is owned by the Phase 36 ship path, NOT by 36-CONTEXT.md.
- **STAT-GUI-01..04 acceptance from ROADMAP:** carried verbatim — Phase 36 closes 4 of 5 STAT-GUI requirements (the 5th, STAT-GUI-05, is reassigned per D-36.2 / D-36.3).

### Discussed and decided in this session (D-36.1 — D-36.4)

#### CATALOG 2 enumeration mechanism for STAT_1

- **D-36.1: Mirror the existing bit-0 conditional block for bit-1 in `hp41-core/src/ops/program.rs::op_catalog(state, 2)`.** ~10 LOC delta; no abstraction. Both `if state.xrom_modules & 0b0000_0001 != 0 { ... MATH_1 ... }` and the new `if state.xrom_modules & 0b0000_0010 != 0 { ... STAT_1 ... }` blocks coexist as parallel siblings. The `NO XROM` fallback fires only when NEITHER bit is set (currently impossible post-`migrate_after_load`, but defensive). Refactor to a generic loop over `[(MATH_1, 0b0000_0001), (STAT_1, 0b0000_0010)]` (or registry-driven enumeration) is deferred until the THIRD XROM module actually lands in v3.2+ (Time Pac / Advantage Pac).
  - **Why:** mirrors CLAUDE.md "three similar lines beats premature abstraction" rule + the Phase 33 D-33.5 / Phase 35 D-30.1 precedent of preferring the smallest delivery surface. Two parallel `if` blocks are the same total LOC as a 2-element array walk, but stay grep-friendly (every reader who searches for `MATH_1.name` or `STAT_1.name` finds the right block by direct match, no array-index indirection). When the THIRD pac lands, the refactor will be a 1-commit chore that introduces the loop alongside the third entry — natural decision-pressure point. Rejected option B (immediate refactor to `[(MATH_1, bit), (STAT_1, bit)]` slice walk) because it pre-pays the abstraction cost ONE milestone before need with no current beneficiary (and v3.2 is post-v3.1-ship, behind unknown future scope). Rejected option C (generic registry over `xrom_modules` bitmap) as significantly over-engineered for 2 modules — would require introducing a new `XromModule` registration abstraction across `hp41-core/src/ops/math1/xrom.rs` that's not currently needed.

#### STAT-GUI-05 cancellation scope reassessment

- **D-36.2: Defer STAT-GUI-05 entirely.** Reassessment at Phase 36 planning revealed that the cancellation requirement was specified based on a misreading of the AS 241 / AS 239 / AS 63 implementation costs. Reality (verified by `hp41-core/src/ops/stat1/distributions.rs:56-228`):
  - `norm_cdf_inv_f64` (ΣNORMD inverse): Acklam's three-branch rational approximation — closed-form, NO iteration. Returns in O(1) arithmetic.
  - `gamma_regularized_f64` (ΣCHISQD CDF basis): power-series `gser` for `x < s+1` OR modified-Lentz CF `gcf` for `x >= s+1`. Both bounded at `ITER_CAP = 50` per SPEC.md Req. 34. Microsecond worst-case wall time.
  - `beta_regularized_f64` (additional Stat 1 path): modified-Lentz CF `betacf` — same 50-iter cap, same microsecond budget.

  All three are pure `f64 → Result<f64, HpError>` with no `&CalcState` parameter. Wiring per-loop `cancel_requested.load()` checks would either pollute the primitive signatures (passing `&AtomicBool` through pure-math helpers) or add no-op cancellation at the workflow layer (Stat 1 workflows complete in <1ms, faster than the user can press R/S). The Phase 31 `cancel_requested` `Arc<AtomicBool>` channel was DESIGNED for user-driven OPEN-ENDED iterative paths — `hp41-core/src/ops/math1/difeq.rs:202` literally documents "The solver runs indefinitely until cancellation (D-28.7)" and `integ.rs:328` checks "Per-64-samples cancellation check (D-28.7 / D-28.8)" inside Romberg quadrature loops that can run for many seconds at high tolerance.
  - **Why:** bounded ≤50-iter primitives at f64 precision complete in microseconds; cancellation is meaningless at that timescale. The Pitfall 11 mitigation (per-loop AtomicBool check + lock release) was developed for the Phase 31 SOLVE/INTG/DIFEQ user-callback strict-reject case where iteration is OPEN-ENDED. Extending it uniformly across all XROM modules "for symmetry" pollutes pure-math primitive signatures without delivering any user-visible benefit. The bounded-iter rationale is itself a v3.1 behavioral policy worth documenting (D-35-NN entry in `docs/hp41-stat1-divergences.md` — see deferred section for the Phase 37 bookkeeping). Rejected option B (wire AtomicBool check at workflow layer for pattern symmetry) because Stat 1 workflows complete in <1ms — even a workflow-layer pre-call check has nothing to interrupt. Rejected option C (full per-loop AtomicBool check inside distributions.rs) as the strictly worst option: pollutes signatures + delivers no benefit. The deferral is the correct engineering call, not a scope cut.

#### STAT-GUI-05 bookkeeping (reassignment, not removal)

- **D-36.3: Reassign STAT-GUI-05 Phase 36 → Phase 37 with bounded-iter rationale.** Mirrors the v3.0 D-32.7 STAT-QUAL-09 reassignment pattern (which moved Phase 37 → Phase 33 with a "verified BEFORE first stat1/*.rs file lands" rationale). Operational deltas:
  - `REQUIREMENTS.md` row 222 — Phase column updates from `Phase 36` → `Phase 37`; Status column stays `Pending`.
  - `REQUIREMENTS.md` STAT-GUI-05 line (line 91) — append `" — reassessed Phase 36 planning: bounded 50-iter primitives (Acklam closed-form / gser+gcf with ITER_CAP=50) don't need cancellation; deferred to Phase 37 STAT-QUAL block per 36-CONTEXT D-36.2"` to the existing requirement text.
  - `ROADMAP.md` Phase 36 Success Criterion #4 (currently asserts cancellation works for ΣNORMD inverse) — replace with the bounded-iter rationale; Phase 36 ships 4 of 5 ROADMAP success criteria.
  - `ROADMAP.md` Phase 37 Requirements line — append `STAT-GUI-05` to the comma-separated list alongside `STAT-QUAL-01..11`.
  - These updates land as a SINGLE commit at the START of `/gsd-execute-phase 36` Plan 36-01 (commit message shape: `docs(36): reassign STAT-GUI-05 Phase 36 → Phase 37 per CONTEXT D-36.2`). This keeps the requirement-state aligned with the actual implementation work BEFORE the phase plans land; downstream agents read updated REQUIREMENTS.md when planning Phase 37.
  - **Why:** mirrors v3.0 D-32.7 reassignment cadence exactly; ROADMAP/REQUIREMENTS.md is the source-of-truth for what each phase delivers, so it must reflect the planning-time reassessment before Phase 36 work begins. Rejected option B (mark STAT-GUI-05 N/A in VERIFICATION.md) because it leaves the requirement unmet for Phase 36 in REQUIREMENTS.md while shipping the phase — inconsistent with v3.0 pattern. Rejected option C (delete STAT-GUI-05 entirely) because the rationale and reassessment have educational value; keeping the requirement with the bounded-iter rationale documents the decision for future archaeologists (e.g., when v3.2 Time Pac iterative interpolation might genuinely need cancellation, the rationale guides the design).

#### Plan slicing

- **D-36.4: Phase 36 ships as 3 plans.** Recommended slice for tight blast-radius separation:
  - **Plan 36-01 (arms + CATALOG 2 core touch + bookkeeping):** First commit — REQUIREMENTS.md + ROADMAP.md updates per D-36.3 (STAT-GUI-05 reassignment). Next 1-2 commits — add 26 `op_display_name` arms to `hp41-gui/src-tauri/src/prgm_display.rs` (copy strings from Phase 34's `hp41-cli/src/prgm_display.rs` work verbatim — D-25.6 parity by construction), update the file-header comment with the new total Op variant count. Next commit — extend `hp41-core/src/ops/program.rs::op_catalog(state, 2)` with the bit-1 conditional block + add `catalog_2_lists_stat1_when_bit1_set` test case + verify existing `catalog_2_lists_math1_when_bit0_set` test still passes (CLI gets the CATALOG 2 STAT_1 enumeration for free). End state: `cargo check -p hp41-gui` clean (no `non-exhaustive patterns` warning), `just gui-ci` exits 0, `just ci` exits 0 (CLI inherits the core change), 4-way invariant item 4 sealed.
  - **Plan 36-02 (HelpOverlay third section + help_data.ts pool + modal-flow Rust test):** Add `docs/hp41-stat1-functions.json` Vite static-import to `hp41-gui/src/help_data.ts` + `helpEntriesStat1()` narrow accessor + extend `helpEntriesAll()` to 3-pool chain. Extend `hp41-gui/src/HelpOverlay.tsx` `SECTIONS` array with the third entry (`{ id: 'stat1', heading: 'Stat 1 Pac (XROM 2)', predicate: (e) => e.xrom?.module === 'Stat 1' }`); widen `expanded` state shape to `{ hp41cv: boolean; math1: boolean; stat1: boolean }` (all default `true`); widen `id` discriminated union to `'hp41cv' | 'math1' | 'stat1'`. Add `lcd_alternation_modal_prompt_stat1.rs` Rust integration test (analog of existing `lcd_alternation_modal_prompt.rs`) asserting `CalcStateView::modal_prompt` carries the 5 Stat1Step OM strings (`"ΣNORMD MODE?"`, `"ν=?"`, `"ΣCHISQD MODE?"`, `"DEGREE=?"`, `"SEED?"`) — verifies STAT-GUI-04 via the existing `CalcStateView::from_state` plumbing. End state: GUI build green, `cargo test -p hp41-gui` green, HelpOverlay manually-verifiable in `just gui-dev` with Stat 1 section visible + collapsible + searchable.
  - **Plan 36-03 (GUI vitest extensions + dispatch_op end-to-end):** Extend `hp41-gui/src/HelpOverlay.test.tsx` with assertions for the new Stat 1 section (mirrors the existing `'math1'` test cases — section heading present, entries render under the right category, collapsing the Stat 1 section hides only its entries, search across "Stat" or "ΣNORMD" returns Stat 1 entries with the right count badge). Add a `App.test.tsx` (or `pending_input.test.ts`) case asserting that dispatching `XEQ "ΣNORMD"` via the GUI keyboard path returns a `CalcStateView` with `modal_prompt === "ΣNORMD MODE?"` — end-to-end CLI-resolver-via-`xrom_resolve` test through the React → Tauri IPC layer. Verify `just gui-ci` exits 0 with the extended test suite. End state: full vitest + Rust integration coverage of the Phase 36 surface; ready for Phase 37 quality-gate hardening.
  - **Why 3 vs 4 vs 5 plans:** Phase 34 used 2 plans for the structurally identical CLI work (26 entries, no core touch). Phase 36 grows by ONE plan vs. Phase 34 because of the STAT-GUI-05 bookkeeping (folded into 36-01) + the GUI-side multi-surface integration (HelpOverlay + help_data.ts + Rust modal-flow test in 36-02) + the vitest extensions (36-03). Rejected 4 plans (split CATALOG 2 from arms) because the core touch is ~10 LOC and trivially co-deployable with the arms (both go through the same `just gui-ci` + `just ci` validation in one commit cluster). Rejected 5 plans (dedicated STAT-GUI-05 cancellation plan) because STAT-GUI-05 is reassigned to Phase 37 per D-36.3 — no Phase 36 plan owns it. Rejected 2 plans (fold 36-03 into 36-02) because the test extensions are a distinct verification blast-radius from the implementation work — splitting keeps each PR-style commit cluster reviewable independently.

### Claude's Discretion

- **HelpOverlay `expanded` state shape:** extend the discriminated record literal `{ hp41cv: boolean; math1: boolean; stat1: boolean }` rather than refactoring to `Record<string, boolean>` — type-safe + grep-friendly for the current 3-section scope, mirrors the existing 2-key pattern. Refactor to a generic `Record<SectionId, boolean>` deferred until v3.2+ pacs land (matching the D-36.1 "three similar entries beats premature abstraction" stance for the React surface). Planner has minor discretion to flip to `Record<>` if any vitest assertion ergonomics suggest it; recommend hold.
- **Default expanded state for the new 'stat1' section:** `true` (open) on overlay mount — matches existing `'hp41cv'` and `'math1'` behavior per Phase 31-04 D-31.8 ("Both sections expanded by default"). The discoverability-on-first-open intent extends to the third section. Planner has discretion to flip to `false` (closed by default) if visual noise concerns surface; recommend hold the precedent.
- **`SECTIONS` `id` union type widening:** `'hp41cv' | 'math1' | 'stat1'` — explicit string-literal union, mirrors the existing 2-element shape verbatim. Planner does NOT widen to `string` because the type-narrowing on `expanded[section.id]` requires the literal union to compile.
- **`SectionDef.predicate` for Stat 1:** `(e: HelpEntry) => e.xrom?.module === 'Stat 1'` — mirrors the existing Math 1 predicate exactly. The Phase 34 `docs/hp41-stat1-functions.json` `xrom.module` field value is `"Stat 1"` per D-34 Claude's Discretion (the planner's recommendation matched, verified at `docs/hp41-stat1-functions.json` entries). If the JSON value differs at Plan 36-02 write time, planner adjusts the predicate to match; either fix is one-line.
- **vitest test file additions:** mirror existing `HelpOverlay.test.tsx` shape — add `describe('Stat 1 Pac section', ...)` block with assertions paralleling the existing `'Math 1 Pac (XROM 7) section'` describe block. Planner picks any additional assertions (e.g., "all 7 Stat 1 categories appear at least once" per D-34.1).
- **Rust integration test file name (Plan 36-02):** `lcd_alternation_modal_prompt_stat1.rs` (mirrors existing `lcd_alternation_modal_prompt.rs`). Planner may alternatively name it `phase36_modal_flow_stat1.rs` for grep-by-phase ergonomics; recommend the LCD-alternation naming for direct-analog cluster legibility.
- **CATALOG 2 test name:** `catalog_2_lists_stat1_when_bit1_set` — mirrors the existing `catalog_2_lists_math1_when_bit0_set` test (if it exists; otherwise the test naming convention is the planner's call). The test fixture constructs `CalcState { xrom_modules: 0b0000_0011, .. }`, invokes `op_catalog(state, 2)`, and asserts `state.print_buffer` contains both `MATH_1.name` and `STAT_1.name` lines.
- **File header comment update in `prgm_display.rs`:** the existing GUI `op_display_name` file likely carries an Op-variant count comment (mirroring the CLI file's `"Covers all 35 Op variants exhaustively"` comment that Phase 34 marked stale at line 26). Plan 36-01 updates the GUI file's count comment to the new total post-Phase-33 Op variants. Planner pulls the actual current count via `grep -c "Op::" hp41-gui/src-tauri/src/prgm_display.rs` (or `cargo expand`) and updates the comment to the post-Phase-36 total.
- **Plan 36-01 commit ordering:** (a) D-36.3 bookkeeping commit FIRST (REQUIREMENTS.md + ROADMAP.md updates) so downstream verification reads the corrected state. (b) 26 GUI arm additions next — closes 4-way invariant item 4 atomically. (c) `op_catalog` bit-1 extension + test last — the core touch verifies CLI inherits the change automatically (sanity-check via running CLI's CATALOG 2 manually if planner wants). Planner has discretion to fold (b)+(c) into one commit if both fit a clean diff; recommend 3 commits for surgical revert windows.
- **Plan 36-02 commit ordering:** data-layer extension before UI consumer — (a) `help_data.ts` extension (third Vite import + accessor) lands first, with type-system gate ensuring the new pool is well-typed. (b) `HelpOverlay.tsx` `SECTIONS` extension consumes the new accessor — visible Stat 1 section appears in `just gui-dev`. (c) `lcd_alternation_modal_prompt_stat1.rs` Rust integration test lands last — verifies the underlying core surface independently of the React UI. Three commits.
- **Plan 36-03 commit ordering:** vitest first (UI-focused), then dispatch-end-to-end (IPC-focused). Two commits.
- **Tauri permission file updates:** none — Phase 36 does not add new inline commands. The existing `dispatch_op`, `get_state`, etc. permissions cover the surface verbatim. Planner verifies via `ls hp41-gui/src-tauri/permissions/` showing no missing TOML files for the touched commands.
- **GUI cancellation channel verification (sanity check, NOT new work):** Plan 36-03 may include a `cargo test -p hp41-gui` assertion that the existing `request_cancel` plumbing (Phase 31 GUI-05) still works for the SOLVE/INTG/DIFEQ paths — verifies Phase 36's `op_catalog` core touch + 26 new arms didn't accidentally regress the cancellation channel. This is one test case, ~10 LOC.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-level (always-on)

- `.planning/PROJECT.md` — v3.1 milestone scope; v3.0 archived block; key decisions ledger; "Shipped" / "Current focus" lines updated by Phase 35 ship (Current focus reads "Phase 36 — hp41 gui — gui integration" per STATE.md)
- `.planning/REQUIREMENTS.md` — 66 v3.1 requirements; Phase 36 maps to STAT-GUI-01..05 (rows 218–222 in the traceability table); Plan 36-01 updates row 222 + line 91 per D-36.3 reassignment
- `.planning/ROADMAP.md` — Phase 36 section lines 75 + 158–169 (Goal + 4 success criteria — Plan 36-01 reduces #4 to bounded-iter rationale per D-36.3); cross-cutting constraints carried forward from v3.0 ROADMAP archive
- `.planning/STATE.md` — v3.1 phase overview; "Stopped at: Phase 35 complete (4/4) — ready to discuss Phase 36"; carried-forward decisions; critical implementation traps
- `CLAUDE.md` (repo root) — Frozen Invariants section (4-way exhaustive-match invariant lines 71–80; SC-4 invariant lines 56–61; CLI ↔ GUI parity D-25.6 lines 108–112; GUI specifics lines 136–148 — Tauri permissions, SVG animation, busyRef, no-polling D-11, persistence sharing); Tech Stack section; Key Files table for `hp41-gui/`; v3.1 additions block Phase 36 sub-section (currently stub `(in progress)`)

### Phase 33 (the contract Phase 36 consumes)

- `.planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-CONTEXT.md` — full Phase 33 decisions; D-33.3 / D-33.3b / D-33.4 / D-33.5 / D-33.7 carry forward as Phase 36 read-only context
- `.planning/phases/33-…/33-SPEC.md` — 39 LOCKED Stat 1 Pac requirements, particularly Req. 34 (ITER_CAP = 50 for gser/gcf/betacf — directly cited in D-36.2 bounded-iter rationale), Stat1Step modal flow specifications

### Phase 34 (the direct CLI analog — 1:1 GUI mirror target)

- `.planning/phases/34-hp41-cli-cli-integration/34-CONTEXT.md` — full Phase 34 decisions; D-34.1 / D-34.3 / D-34.5 / D-34.6 carry forward verbatim — Phase 36 mirrors the help-overlay shape + modal-prompt routing to the GUI surface
- `.planning/phases/34-hp41-cli-cli-integration/34-01-PLAN.md` — JSON authoring plan (Phase 36 Plan 36-02 reuses the JSON read-only via Vite import; structural parallel to Phase 34 Plan 34-01 but no JSON re-authoring)
- `.planning/phases/34-hp41-cli-cli-integration/34-02-PLAN.md` — `op_display_name` arms + parity tests (Phase 36 Plan 36-01 mirrors the arm-shape verbatim — copies the 26 arm string match-arms unchanged per D-25.6 parity invariant)

### Phase 35 (already-shipped — Phase 36 honors the docs locks)

- `.planning/phases/35-documentation-adrs/35-CONTEXT.md` — Phase 35 decisions D-35.1 through D-35.5; structurally relevant to Phase 36 only for the `### v3.1 additions` block Phase 36 sub-section convention (currently stub `(in progress)`; filled at Phase 36 ship-time per D-30.8 incremental pattern)
- `docs/adr/v3.1-001-rng-state-placement.md` through `v3.1-005-modalprogram-stat1-enum-extension.md` — read-only architectural locks; Phase 36 honors them implicitly (e.g., does not duplicate `ModalProgram::Stat1` dispatch logic into the GUI)
- `docs/hp41-stat1-divergences.md` — Phase 36 may add a `D-35-NN` Behavioral Policies entry post-ship documenting the STAT-GUI-05 bounded-iter rationale (planner discretion; recommend yes for documentation completeness — see deferred section)

### v3.0 Phase 31 (the direct STRUCTURAL analog — GUI integration template)

- `.planning/milestones/v3.0-phases/31-hp41-gui-integration/31-CONTEXT.md` — Phase 31 decisions D-31.1 through D-31.14; structurally identical phase one milestone earlier (Math Pac I GUI integration)
- `.planning/milestones/v3.0-phases/31-hp41-gui-integration/31-04-PLAN.md` (or equivalent) — the Math Pac I GUI HelpOverlay 2-section extension plan; Phase 36 Plan 36-02 mirrors this verbatim, extending 2 → 3 sections
- `.planning/milestones/v3.0-phases/31-hp41-gui-integration/31-02-PLAN.md` (or equivalent) — cancellation channel introduction (where `request_cancel` was wired into the GUI command surface for INTG/SOLVE/DIFEQ); Phase 36 D-36.2 explicitly cites this as the channel STAT-GUI-05 was originally intended to reuse + explains why bounded primitives don't need it

### hp41-gui existing pattern reservoir (Phase 36 reuses these in-place)

- `hp41-gui/src-tauri/src/prgm_display.rs` — exhaustive `op_display_name()` match (currently sanctioned `non-exhaustive patterns` CI break since Phase 33 ship); Plan 36-01 adds 26 arms; the file header comment carries an Op-variant count that must be updated to the new total
- `hp41-gui/src-tauri/src/lib.rs` — `setup()`, `AppState = Mutex<CalcState>`, 30s auto-save thread, `generate_handler!` registration; Phase 36 does NOT touch this file
- `hp41-gui/src-tauri/src/commands.rs` — `dispatch_op`, `get_state`, `sst_step`, `bst_step`, `run_stop`, `request_cancel` thunks; Phase 36 does NOT touch — `dispatch_op` routes Stat 1 mnemonics via the shared `xrom_resolve` already, and the existing `CalcStateView::from_state` already carries `modal_prompt` (verified end-to-end Phase 31)
- `hp41-gui/src-tauri/src/types.rs` — `CalcStateView`, `Annunciators`, `GuiError`, `From<HpError>`; Phase 36 does NOT touch (modal_prompt field already in CalcStateView)
- `hp41-gui/src-tauri/src/key_map.rs` — string ID → `Op` resolver; Phase 36 does NOT touch (XEQ-by-name path already resolves Stat 1 via the shared `xeq_by_name_local_resolve` → `xrom_resolve` fall-through identical to CLI Phase 34 STAT-CLI-01)
- `hp41-gui/src-tauri/src/persistence.rs` — shared `~/.hp41/autosave.json`; Phase 36 does NOT touch (auto-save inherits `migrate_after_load` per D-33.7)
- `hp41-gui/src/App.tsx` — React root, `shiftActive`, `invokeForKey` / `extractErrMessage` helpers, toast overlay; Phase 36 does NOT touch (display-only changes go through HelpOverlay; modal-prompt rendering goes through existing `pending_prompt` analog in TypeScript)
- `hp41-gui/src/Keyboard.tsx` — 5×8 grid + top-row band; Phase 36 does NOT touch
- `hp41-gui/src/HelpOverlay.tsx` — Plan 36-02 extends `SECTIONS` array + `expanded` state shape + `id` union type (3 surgical edits, ~15 LOC total delta)
- `hp41-gui/src/HelpOverlay.test.tsx` — Plan 36-03 extends with Stat 1 section test cases mirroring existing Math 1 section test shape
- `hp41-gui/src/help_data.ts` — Plan 36-02 adds third Vite static-import + `helpEntriesStat1()` narrow accessor + extends `helpEntriesAll()` to 3-pool chain
- `hp41-gui/src/App.css` — Phase 36 does NOT touch (existing section-collapse + category-heading CSS covers the new third section verbatim)
- `hp41-gui/src-tauri/permissions/*.toml` — Phase 36 does NOT touch (no new inline commands)
- `hp41-gui/wdio.conf.cjs` + `e2e/smoke.spec.js` — Phase 36 does NOT touch (E2E Stat 1 smoke extension is Phase 37 / STAT-QUAL-11)

### hp41-core public surface Phase 36 consumes (read-only except `op_catalog`)

- `hp41-core/src/ops/math1/xrom.rs:122-186` — `STAT_1: XromModule` const + `STAT_1.ops` 26-entry slice (used by `op_catalog` bit-1 block); `STAT_1.name = "STAT 1B"` is the CATALOG 2 display string per OM 00041-90030
- `hp41-core/src/ops/math1/xrom.rs:188-212` — `xrom_resolve(name, modules)` with the bit-1 arm Phase 33 activated; GUI inherits Stat 1 resolution via `dispatch_op` → core `xrom_resolve` (no GUI-side changes needed)
- `hp41-core/src/ops/program.rs:285-365` — `op_catalog(state, n)` function; Plan 36-01 extends the `n=2` arm with a parallel bit-1 conditional block immediately after the existing bit-0 block (lines 343-354). The extension is internal to the `n=2` match arm; signatures of `op_catalog` unchanged
- `hp41-core/src/ops/math1/modal.rs:38-52` — `ModalProgram::Stat1(Stat1Step)` variant (Phase 33 D-33.3b carve-out); Phase 36 reads via existing `CalcStateView::from_state` plumbing; no modification
- `hp41-core/src/ops/stat1/modal.rs:333-361` — `current_prompt(&Stat1Step) -> Option<String>` + `requires_alpha_label(&Stat1Step) -> bool` (returns `false` for ALL 5 Stat1Step variants — confirming GUI does NOT need to extend any auto-open hook); Plan 36-02's `lcd_alternation_modal_prompt_stat1.rs` test asserts the OM strings returned here surface through `CalcStateView`
- `hp41-core/src/ops/stat1/distributions.rs:56-228` — `norm_cdf_inv_f64`, `gamma_regularized_f64`, `beta_regularized_f64` + bounded-iter `gser`, `gcf`, `betacf` helpers (`ITER_CAP = 50`); cited in D-36.2 as the basis for the cancellation reassessment
- `hp41-core/src/state.rs:386-391` — `migrate_after_load` bit-1 v3.0 → v3.1 migration (D-33.7 single-source-of-truth); GUI inherits via `hp41-gui/src-tauri/src/persistence.rs` calling `load_state` which calls `migrate_after_load` — no Phase 36 changes
- `hp41-core/src/ops/mod.rs::Op` enum — the 26 new Stat 1 variants (`SigmaBstat` through `Seed`); Plan 36-01's 26 arms enumerate these exhaustively (no `_ =>` catch-all)

### JSON pipeline (canonical pattern + read-only consumer)

- `docs/hp41-stat1-functions.json` — Phase 34 / Plan 34-01 authored the 26 entries; Phase 36 Plan 36-02 consumes read-only via Vite static import (TypeScript types from the existing `HelpEntry` interface in `hp41-gui/src/help_data.ts`); no JSON modifications
- `docs/hp41-stat1-function-matrix.md` — Phase 35 / Plan 35-01 generated; Phase 36 does NOT touch
- `docs/hp41-math1-functions.json` — read-only schema-shape reference (Vite import pattern already established in `hp41-gui/src/help_data.ts`); Phase 36's third import mirrors the Math 1 import shape verbatim

### HP Stat 1 Pac primary sources (HP-copyrighted — DO NOT redistribute; OM-style citations only)

- HP-41C Stat 1 Pac Owner's Manual (HP 00041-90030, 1979) — `STAT_1.name = "STAT 1B"` source per `hp41-core/src/ops/math1/xrom.rs:126` doc comment; Phase 36 does not cite directly (no doc-writing) but the OM strings the modal-prompt test asserts (`"ΣNORMD MODE?"`, `"ν=?"`, etc.) trace back to this source via `hp41-core/src/ops/stat1/modal.rs:333-343`
- HP-41C Stat 1 Pac Quick Reference Card (HP 00041-90061, 1979) — entry-point catalog; the 26 mnemonics in `STAT_1.ops` (and the 26 `op_display_name` arms in Plan 36-01) trace to this source

### Project-local CLAUDE.md guidance

- CLAUDE.md `## Git Workflow` — commits use `/git-workflow:commit --with-skills` only; English-only subject + body
- CLAUDE.md `## Frozen Invariants → SC-4 invariant` — strict GUI grep excludes `op_*` math; the 26 display-name arms in `hp41-gui/src-tauri/src/prgm_display.rs` are pure strings + the `op_catalog` extension lives in `hp41-core/`, NOT in `hp41-gui/`; SC-4 trivially preserved
- CLAUDE.md `## Frozen Invariants → 4-way exhaustive-match invariant` — item 4 (`hp41-gui/src-tauri/src/prgm_display.rs`) sealed by Phase 36 Plan 36-01; CI `non-exhaustive patterns` warning closes
- CLAUDE.md `## Frozen Invariants → CLI ↔ GUI parity (D-25.6)` — the 26 arm strings in Plan 36-01 are DELIBERATELY duplicated from Phase 34's CLI work; copying verbatim is the parity invariant by construction
- CLAUDE.md `## Frozen Invariants → GUI specifics` — Tauri v2.11 permissions, busyRef debounce, no-polling D-11, persistence sharing — Phase 36 inherits all preserved; no new constraints introduced
- CLAUDE.md `### v3.1 additions → #### Phase 36 — GUI Integration (in progress)` — stub heading; Phase 36 ship-time fills in the sub-section narrative per D-30.8 / D-35.5 incremental pattern

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`hp41-gui/src-tauri/src/prgm_display.rs`** — exhaustive `op_display_name` match (currently `non-exhaustive patterns` open since Phase 33); Plan 36-01 adds 26 arms by COPYING the Phase 34 CLI version's arm strings verbatim (D-25.6 parity invariant — the strings are deliberately duplicated CLI ↔ GUI). The file's header comment carries an Op-variant count that must be updated (mirror the Phase 34 comment-update at `hp41-cli/src/prgm_display.rs:26`).
- **`hp41-gui/src/help_data.ts`** — the existing two-Vite-import pattern (lines 17-18: `import math1Functions from '../../docs/hp41-math1-functions.json';`) is reusable verbatim. Plan 36-02 adds a third copy of the pattern: `import stat1Functions from '../../docs/hp41-stat1-functions.json';` + `helpEntriesStat1()` accessor + extend `helpEntriesAll()` to chain all three pools.
- **`hp41-gui/src/HelpOverlay.tsx:39-50`** `SECTIONS` array — direct extension target. Add a third `SectionDef` entry mirroring the existing `'math1'` entry, swapping `'math1'` → `'stat1'`, `'Math 1 Pac (XROM 7)'` → `'Stat 1 Pac (XROM 2)'`, and `e.xrom?.module === 'Math 1'` → `e.xrom?.module === 'Stat 1'`. ~5 LOC delta.
- **`hp41-gui/src/HelpOverlay.tsx:30-36`** `SectionDef` interface — widen the `id` union from `'hp41cv' | 'math1'` to `'hp41cv' | 'math1' | 'stat1'`. ~1 LOC delta.
- **`hp41-gui/src/HelpOverlay.tsx:56-59`** `expanded` state — widen the `useState` type literal from `{ hp41cv: boolean; math1: boolean }` to `{ hp41cv: boolean; math1: boolean; stat1: boolean }` + add `stat1: true` to the initial value + update the `setExpanded({ hp41cv: true, math1: true })` call at line 65 to include `stat1: true`. ~3 LOC delta.
- **`hp41-gui/src/HelpOverlay.tsx:130-132`** `toggleSection` — widen the `id` parameter type to include `'stat1'`. ~1 LOC delta.
- **`hp41-core/src/ops/program.rs:343-354`** existing bit-0 conditional block — DIRECT structural template for Plan 36-01's bit-1 extension. Copy the 11-line block, swap `0b0000_0001` → `0b0000_0010`, `MATH_1.id` → `STAT_1.id`, `MATH_1.name` → `STAT_1.name`, `MATH_1.ops` → `STAT_1.ops`. The `NO XROM` fallback at line 353 fires only if BOTH bits are clear (currently impossible post-`migrate_after_load`, defensive).
- **`hp41-gui/src/HelpOverlay.test.tsx`** — Plan 36-03 extends existing `describe('Math 1 Pac section', ...)` block with a parallel `describe('Stat 1 Pac section', ...)` block — same shape, swap names. ~30-50 LOC delta.
- **Existing `lcd_alternation_modal_prompt.rs`** Rust integration test in `hp41-gui/src-tauri/tests/` (or equivalent path) — DIRECT structural template for Plan 36-02's `lcd_alternation_modal_prompt_stat1.rs`. Read its shape; construct `ModalProgram::Stat1(Stat1Step::NormdModeChoice)` etc. instead of `ModalProgram::Matrix(...)`; assert `CalcStateView::from_state(state).modal_prompt == Some("ΣNORMD MODE?".to_string())` for each of the 5 Stat1Step variants.
- **`hp41-core/src/ops/math1/integ.rs:328-329`** + **`difeq.rs:311-312`** + **`solve.rs` (cancellation check pattern)** — directly cited in D-36.2 as the SOLVE/INTG/DIFEQ open-ended iterative patterns that DO need `cancel_requested` checks; Phase 36 does NOT extend this pattern to bounded distribution primitives (rationale documented in D-36.2 / deferred for the v3.1 divergences note).

### Established Patterns

- **JSON canonical pipeline + Vite static-import build-time gate (D-25.16/17/18 GUI analog):** Phase 36 mirrors this via the third-file import shape. TypeScript compile error on malformed JSON is the GUI's hard-build-blocker equivalent of the CLI's `OnceLock` panic on malformed JSON.
- **D-25.6 CLI ↔ GUI parity:** every Stat 1 modal behavior the GUI gains here routes through shared `hp41-core` code (`xrom_resolve`, `submit_modal`, `cancel_modal`, `requires_alpha_label`, `ModalProgram::Stat1` dispatch). The 26 display-name arms in `prgm_display.rs` are pure strings; the Phase 34 CLI version is the source-of-truth — Phase 36 GUI version is the deliberate duplicate (the 4-way invariant enforces this duplication at the compile-time exhaustive-match level).
- **Op variants land before consumers (CLAUDE.md):** Phase 33 already landed all 26 `Op::Sigma* / Op::Rand / Op::Seed` variants and arms 1+2 of the 4-way invariant (`dispatch()` + `execute_op()`). Phase 34 closed arm 3 (`hp41-cli/src/prgm_display.rs`). Phase 36 closes arm 4 (`hp41-gui/src-tauri/src/prgm_display.rs`). The intentional `non-exhaustive patterns` GUI warning is the visible failure mode that the closing arm landing turns green.
- **No `println!` / `eprintln!` in `hp41-core`** — the `op_catalog` bit-1 extension preserves this; the new lines mirror existing `state.print_buffer.push(format!(...))` calls.
- **Per-Op `LiftEffect` declarations** — Phase 36 doesn't touch Op definitions; the lift effects were set in Phase 33 (D-33.3-D-33.5).
- **`busyRef` two-layer debounce (CLAUDE.md GUI specifics):** Phase 36 doesn't touch `App.tsx::handleClick` or `Keyboard.tsx::handleKeyClick`; the existing debounce pattern continues to gate against concurrent `invoke()` calls.
- **No polling (D-11):** Phase 36 doesn't introduce polling; the `dispatch_op` response carries `CalcStateView.modal_prompt` already (verified end-to-end since Phase 31).
- **Tauri v2.11 inline-command permissions:** Phase 36 doesn't add new commands; no new TOML files in `hp41-gui/src-tauri/permissions/`.
- **Bounded vs open-ended iteration distinction (NEW pattern surfaced in D-36.2):** Phase 31 SOLVE/INTG/DIFEQ are user-driven OPEN-ENDED iterative paths needing `cancel_requested` checks; AS 239 / AS 63 / Acklam distribution primitives are BOUNDED ≤50-iter at f64 precision and do NOT need cancellation. This distinction may bear documentation in `docs/hp41-stat1-divergences.md` as a Behavioral Policies entry (Phase 36 ship-time follow-up, planner discretion).

### Integration Points

- **4-way invariant item 4 closure point:** the moment Plan 36-01's 26 arms land, `cargo check -p hp41-gui` produces zero `non-exhaustive patterns` warnings; `just gui-ci` exits 0 for the first time since Phase 33 ship. CLI continues to compile clean (was sealed by Phase 34).
- **CATALOG 2 STAT_1 enumeration via core:** Plan 36-01's `op_catalog` extension is the SINGLE source of the user-visible effect; CLI and GUI both inherit the listing for free. The CLI's existing CATALOG 2 test (presumably `hp41-cli/tests/cat2_*` or via `op_catalog` core test) continues to pass + a new `catalog_2_lists_stat1_when_bit1_set` core test asserts the new behavior.
- **HelpOverlay third section render integration:** Plan 36-02 extends `SECTIONS` from 2 to 3 entries; the existing `sectionGroups.map(...)` render at `HelpOverlay.tsx:160-192` requires zero modification because it already iterates over `SECTIONS` generically.
- **CSS reuse:** the new third section uses the existing `.help-overlay-section` / `.help-overlay-section-heading` / `.help-overlay-section-body` / `.help-overlay-category-heading` / `.help-overlay-row` classes verbatim. No `App.css` changes.
- **Vite static-import gate:** the third `import stat1Functions from '../../docs/hp41-stat1-functions.json';` in `help_data.ts` becomes a Vite build dependency — if the JSON is malformed, Vite fails the build (TypeScript type-check on the `HelpEntry[]` cast catches structural mismatches). Mirrors the existing two-import pattern.
- **`dispatch_op` end-to-end Stat 1 resolution:** Plan 36-03's vitest case asserts the full React → Tauri IPC → `xrom_resolve` → `ModalProgram::Stat1` → `CalcStateView` → React re-render chain works for `XEQ "ΣNORMD"`. This is the user-facing acceptance test for STAT-GUI-04.

</code_context>

<specifics>
## Specific Ideas

- **HelpOverlay third-section `SectionDef` entry (Plan 36-02 input — exact shape):**
  ```typescript
  {
      id: 'stat1',
      heading: 'Stat 1 Pac (XROM 2)',
      predicate: (e: HelpEntry) => e.xrom?.module === 'Stat 1',
  },
  ```

- **HelpOverlay `expanded` state widening (Plan 36-02 input):**
  ```typescript
  const [expanded, setExpanded] = useState<{ hp41cv: boolean; math1: boolean; stat1: boolean }>({
      hp41cv: true,
      math1: true,
      stat1: true,
  });
  ```
  And update line 65 reset:
  ```typescript
  setExpanded({ hp41cv: true, math1: true, stat1: true });
  ```

- **`SectionDef.id` union widening (Plan 36-02 input):**
  ```typescript
  interface SectionDef {
      id: 'hp41cv' | 'math1' | 'stat1';
      heading: string;
      predicate: (e: HelpEntry) => boolean;
  }
  ```

- **`toggleSection` parameter widening (Plan 36-02 input):**
  ```typescript
  const toggleSection = (id: 'hp41cv' | 'math1' | 'stat1') => {
      setExpanded(prev => ({ ...prev, [id]: !prev[id] }));
  };
  ```

- **`help_data.ts` Vite static-import extension (Plan 36-02 input):**
  ```typescript
  import stat1Functions from '../../docs/hp41-stat1-functions.json';

  // … existing math1Functions accessor …

  /// Phase 36 (Plan 36-02): Stat 1 Pac function entries from docs/hp41-stat1-functions.json.
  export function helpEntriesStat1(): readonly HelpEntry[] {
      return stat1Functions as readonly HelpEntry[];
  }

  /// Phase 36 (Plan 36-02): 3-pool merge — built-ins + Math 1 Pac + Stat 1 Pac.
  export function helpEntriesAll(): readonly HelpEntry[] {
      return [...helpEntries(), ...helpEntriesMath1(), ...helpEntriesStat1()];
  }
  ```
  (Exact existing accessor names — `helpEntries()` vs `helpEntriesBuiltins()` — planner reads from current `help_data.ts` and matches.)

- **`op_catalog` bit-1 extension (Plan 36-01 input, after line 354 of `hp41-core/src/ops/program.rs`):**
  ```rust
  if state.xrom_modules & 0b0000_0010 != 0 {
      // Stat 1 Pac (bit 1) is loaded.
      state.print_buffer.push(format!(
          "{:<24}",
          format!("XROM {} {}", STAT_1.id, STAT_1.name)
      ));
      for (name, _op) in STAT_1.ops {
          state.print_buffer.push(format!("{name:<24}"));
      }
  }
  ```
  Placement: immediately after the existing `MATH_1` block, before the `NO XROM` else-fallback. The `NO XROM` else fires only when BOTH bits are clear (the existing `else` becomes structurally unreachable post-`migrate_after_load`; D-36.1 keeps it as defensive code).
  Import: ensure `use crate::ops::math1::xrom::STAT_1;` is present alongside the existing `MATH_1` import.

- **CATALOG 2 test (Plan 36-01 input):**
  ```rust
  #[test]
  fn catalog_2_lists_stat1_when_bit1_set() {
      let mut state = CalcState {
          xrom_modules: 0b0000_0011,
          .. CalcState::default()
      };
      op_catalog(&mut state, 2).unwrap();
      let buffer = state.print_buffer.join("\n");
      assert!(buffer.contains("MATH 1A"), "CATALOG 2 must list MATH 1A");
      assert!(buffer.contains("STAT 1B"), "CATALOG 2 must list STAT 1B");
      // Spot-check one entry from each pac:
      assert!(buffer.contains("Y^X"), "CATALOG 2 must list a Math 1 entry");
      assert!(buffer.contains("ΣBSTAT"), "CATALOG 2 must list a Stat 1 entry");
  }
  ```
  (Exact entry mnemonics to spot-check — planner verifies against `MATH_1.ops` and `STAT_1.ops` slices at write time.)

- **Stat1Step modal-prompt strings to assert in `lcd_alternation_modal_prompt_stat1.rs` (Plan 36-02 input, read from `hp41-core/src/ops/stat1/modal.rs:333-343`):**
  - `Stat1Step::NormdModeChoice` → `"\u{03A3}NORMD MODE?"` (renders as `ΣNORMD MODE?`)
  - `Stat1Step::ChisqdNuPrompt` → `"\u{03BD}=?"` (renders as `ν=?`)
  - `Stat1Step::ChisqdModeChoice` → `"\u{03A3}CHISQD MODE?"` (renders as `ΣCHISQD MODE?`)
  - `Stat1Step::PolypDegreePrompt(_)` → `"DEGREE=?"`
  - `Stat1Step::SeedPrompt` → `"SEED?"`

- **STAT-GUI-05 reassignment commit message (Plan 36-01 first commit):**
  ```
  docs(36): reassign STAT-GUI-05 Phase 36 → Phase 37 per CONTEXT D-36.2

  Reassessment at Phase 36 planning: ΣNORMD inverse uses closed-form
  Acklam (no iteration); ΣCHISQD CDF uses bounded 50-iter gser/gcf
  (microseconds wall time). Pure f64 primitives with no &CalcState
  access. Phase 31 cancel_requested channel was designed for open-ended
  SOLVE/INTG/DIFEQ paths — bounded primitives don't need it.

  Updates REQUIREMENTS.md row 222 (Phase 36 → Phase 37) and line 91
  (append bounded-iter rationale). Updates ROADMAP.md Phase 36 success
  criterion #4 + Phase 37 requirements line. Mirrors v3.0 D-32.7
  STAT-QUAL-09 reassignment cadence (37 → 33 in that case).
  ```
  Per CLAUDE.md `## Git Workflow`: use `/git-workflow:commit --with-skills`, English only.

</specifics>

<deferred>
## Deferred Ideas

- **STAT-GUI-05 cancellation in Stat 1 paths** — REASSIGNED to Phase 37 STAT-QUAL block per D-36.2 / D-36.3 (NOT removed — the bounded-iter rationale is documented in CONTEXT and propagated to REQUIREMENTS.md + ROADMAP.md at Plan 36-01 first commit). Phase 37 quality-gate authoring re-verifies the assessment and either (a) confirms deferral as the final disposition + adds a `D-35-NN` Behavioral Policies entry to `docs/hp41-stat1-divergences.md` documenting the policy, OR (b) discovers a previously-unseen Stat 1 iterative path that does warrant cancellation (e.g., a Stat 1 ANOVA workflow that loops over many ΣAOVTWO degrees-of-freedom evaluations) and ships the per-call workflow-layer check at that point.
- **`docs/hp41-stat1-divergences.md` D-35-NN Behavioral Policies entry for bounded-iter cancellation policy** — Phase 36 ship-time follow-up (planner discretion in Plan 36-03 or as a Phase 36 post-ship quick-task). Single 5-field divergence entry per D-30.5 shape, citing D-36.2 rationale + SPEC.md Req. 34 (ITER_CAP = 50). Optional; nice-to-have for documentation completeness.
- **`CLAUDE.md` Phase 36 sub-section under `### v3.1 additions`** — Phase 36 ship-time (Plan 36-03 last commit OR post-ship via `/gsd-docs-update`). Current state is the stub `#### Phase 36 — GUI Integration (in progress)` from Phase 35 D-35.5. Phase 36 ship-time replaces with the narrative summary (paralleling Phase 33 / 34 / 35 sub-section depth — 2-4 paragraphs covering: 4-way invariant item 4 closure, CATALOG 2 STAT_1 enumeration extension, HelpOverlay 2 → 3 sections, STAT-GUI-05 deferral, no new dependencies, GUI tests green). Same pattern as Phase 34 ship-time and Phase 35 ship-time fills.
- **`docs/architecture-history.md` Phase 36 narrative** — Phase 36 ship-time (Plan 36-03 last commit). Stub heading currently in place; populate with 2-3 paragraph narrative parallel to Phase 33 / 34 / 35 sub-sections in the v3.1 additions section.
- **`PROJECT.md` Shipped / Current focus line updates** — Phase 36 ship-time. "Shipped" gets a v3.1 Phase 36 line; "Current focus" advances to "Phase 37 — Test Hardening & Quality Gates".
- **WebdriverIO E2E smoke with one Stat 1 Pac workflow** — Phase 37 / STAT-QUAL-11. `XEQ "ΣNORMD"` with x = 1.96 returns Q ≈ 0.0250 on the GUI LCD (Ubuntu, WebdriverIO). Phase 36 lays the groundwork (GUI surface fully wired) but the E2E test is Phase 37.
- **`numerical_accuracy.rs` extension with Stat 1 oracle cases** — Phase 37 / STAT-QUAL-04.
- **Per-`stat1/*.rs` per-file coverage floor ≥ 90 %** — Phase 37 / STAT-QUAL-03.
- **`stat1_op_test_count.rs` + `lint_stat1_assertions.rs` + `xrom_shadowing.rs` STAT_1 extension** — Phase 37 / STAT-QUAL-06, STAT-QUAL-07, STAT-QUAL-08.
- **Backward-compat test for v3.0 save migration** (`v30_save_loads_with_stat1_off`) — Phase 37 / STAT-QUAL-10.
- **Free42 contamination guard** — Phase 33 / Plan 33-00 already extended the script to 18 tokens covering both math1/ and stat1/ identifiers. Phase 37 re-verifies in CI context but adds no new tokens for Phase 36's scope.
- **CATALOG 2 generic loop refactor over a registry of `[XromModule; N]`** — deferred until the third XROM module lands in v3.2+ (Time Pac / Advantage Pac). At that point the refactor will be a 1-commit chore that introduces the loop alongside the third entry — natural decision-pressure point per D-36.1.
- **HelpOverlay `Record<SectionId, boolean>` generic-state refactor** — deferred per same rationale as the CATALOG 2 refactor; mirrors D-36.1 stance for the React surface.
- **Signed binary releases (cargo-dist CLI + tauri-action GUI)** — deferred to v3.1.x / v3.2 per PROJECT.md lock 2026-05-21.
- **Cross-pac divergence-doc index (`docs/divergences-index.md`)** — deferred to v3.2 when a third pac actually surfaces the cross-pac pattern (mirrors Phase 35 deferred ideas — same rationale).
- **`/gsd-complete-milestone` v3.1 ship** — Phase 37 post-ship. Phase 35 landed a `.planning/MILESTONES.md` stub; full milestone-summary one-liner + ROADMAP archive moves at Phase 37 ship.

</deferred>

---

*Phase: 36-hp41-gui-gui-integration*
*Context gathered: 2026-05-23*
