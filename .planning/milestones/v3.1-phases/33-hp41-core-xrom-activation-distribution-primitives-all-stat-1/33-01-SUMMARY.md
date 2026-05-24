---
phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1
plan: 01
subsystem: hp41-core
tags: [xrom, stat1, freeze-exception, save-file-migration, rng-seed, scaffolding, 4-way-invariant]

requires:
  - phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1
    plan: 00
    provides: hp41-core/src/ops/stat1/ skeleton + OM Appendix A register layout + Free42 contamination guard extension
  - phase: 28-math-pac-1
    provides: XromModule struct + xrom_resolve LAST-fires invariant + bit-1 stub comment + ModalProgram carrier enum
provides:
  - hp41-core/src/state.rs::CalcState::rand_seed (HpNum, unique #[serde(default)] WITHOUT skip)
  - hp41-core/src/state.rs::default_xrom_modules() = 0b0000_0011 (Math 1 + Stat 1 pre-loaded)
  - hp41-core/src/state.rs::CalcState::migrate_after_load() (idempotent, sets bit 1)
  - hp41-core/src/ops/math1/xrom.rs::STAT_1 const (id=2, name="STAT 1B", 26 ops entries)
  - hp41-core/src/ops/math1/xrom.rs::stat1_resolve() (exhaustive match → Op::Stat1Stub for all 26 mnemonics)
  - hp41-core/src/ops/math1/xrom.rs::xrom_resolve bit-1 arm (D-33.3 freeze exception activation)
  - hp41-core/src/ops/math1/modal.rs::ModalProgram::Stat1(Stat1Step) variant (D-33.3b freeze exception)
  - hp41-core/src/ops/stat1/modal.rs::Stat1Step enum + 3 dispatch functions
  - hp41-core/src/ops/Op::Stat1Stub variant (Plan-33-01 scaffolding — TO BE REMOVED by end of Phase 33)
  - hp41-core/src/ops/stat1/mod.rs::op_stat1_stub (returns Err(InvalidOp))
  - hp41-core/tests/xrom_shadowing.rs Stat 1 disjointness gates (4 new tests)
affects:
  - 33-02 (distributions — first stat1/*.rs algorithm file consuming the scaffolded modal carrier)
  - 33-03 (replaces Stat1Step::Placeholder with NormdModeChoice + ChisqdNuPrompt; consumes ModalProgram::Stat1 + STAT_1)
  - 33-04..33-07 (replace Op::Stat1Stub references in stat1_resolve + STAT_1.ops with real Op variants)
  - 33-08 (final removal of Op::Stat1Stub; adds PolypDegreePrompt + SeedPrompt variants)
  - 34 (CLI wiring — items 3+4 of the 4-way exhaustive-match invariant for Op::Stat1Stub MUST land here before hp41-cli compiles)
  - 36 (GUI wiring — items 4 of the 4-way exhaustive-match invariant; mirror of Phase 34)

tech-stack:
  added: []  # pure framework activation — no new runtime deps
  patterns:
    - "Unique #[serde(default)] (no skip) for persistent state surviving save/load (rand_seed mirrors complex_mode)"
    - "Idempotent post-deserialization migration hook (CalcState::migrate_after_load — single source of truth)"
    - "Freeze-exception scoping by file (xrom.rs D-33.3, modal.rs D-33.3b; mod.rs Rule-3 deviation forced by exhaustive match)"
    - "Sibling resolver under shared XROM file with brace-scoped scan in meta-tests"
    - "Placeholder Op variant + placeholder modal step for cross-plan compile contract"

key-files:
  created:
    - hp41-core/src/ops/stat1/modal.rs
    - .planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-01-SUMMARY.md
  modified:
    - hp41-core/src/state.rs
    - hp41-core/src/ops/math1/xrom.rs
    - hp41-core/src/ops/math1/modal.rs
    - hp41-core/src/ops/math1/mod.rs
    - hp41-core/src/ops/mod.rs
    - hp41-core/src/ops/program.rs
    - hp41-core/src/ops/stat1/mod.rs
    - hp41-core/tests/xrom_shadowing.rs
    - hp41-core/tests/v3_save_compat.rs
    - hp41-core/tests/math1_op_test_count.rs

key-decisions:
  - "Strategy A for stub Op: added a single placeholder Op::Stat1Stub variant referenced 26 times in STAT_1.ops + 26 times in stat1_resolve, returning Err(InvalidOp) on dispatch. Plans 33-03..33-08 replace those references incrementally and Plan 33-08 deletes the variant. Chosen over Strategy B (empty STAT_1.ops) because it lets the xrom_shadowing.rs disjointness CI gate fire immediately."
  - "Strategy for Stat1Step empty-enum problem: added a single `Placeholder` variant rather than an uninhabited enum. Cleaner for clippy (no `match *step {}` on uninhabited enums) and lets Plan 33-03 swap in real NormdModeChoice + ChisqdNuPrompt variants with a single Edit. Plan 33-08 adds PolypDegreePrompt + SeedPrompt."
  - "math1/mod.rs::submit_modal exhaustive-match arm for ModalProgram::Stat1 was edited as a Rule-3 deviation (the plan's Task 2 acceptance criterion listed only math1/modal.rs + math1/xrom.rs in the math1/ touch budget, but D-33.3b implicitly authorizes mod.rs because the match in submit_modal is exhaustive). The blast radius stays at 1 line of pure delegation wiring."
  - "Migration is in hp41-core::CalcState, not per-frontend (D-33.7) — single source of truth. CLI/GUI wiring sites (Phase 34/36) call migrate_after_load() once after every deserialize."
  - "Updated existing 0b0000_0001 assertions in three pre-existing tests (default_construction_phase28_fields, v22_save_loads_with_defaults, loads_synthetic_v22_save_without_v3_fields) to the new 0b0000_0011 default. The contract changed deliberately per D-33.2; these tests assert the NEW contract."

requirements-completed:
  - STAT-FW-01  # STAT_1 module const + resolver fires LAST after MATH_1 + bit-1 isolation
  - STAT-FW-02  # default_xrom_modules() flips to 0b0000_0011 + migrate_after_load()
  - STAT-FW-03  # 4-way exhaustive-match items 1 (dispatch) + 2 (execute_op) for Op::Stat1Stub
  - STAT-FW-04  # XEQ-by-name routes through stat1_resolve for all 26 mnemonics (every entry → Op::Stat1Stub until Plans 33-03..33-08 land real variants)
  - STAT-RNG-03 # rand_seed field with #[serde(default)] WITHOUT skip — round-trip CI gate

metrics:
  duration: 38min
  completed: 2026-05-22
  tasks_total: 4
  tasks_completed: 4
  files_created: 1
  files_modified: 10
  commits: 5
  tests_added: 16  # 4 state.rs + 4 xrom.rs unit tests + 4 stat1/modal.rs unit tests + 4 xrom_shadowing.rs integration tests
---

# Phase 33 Plan 01: XROM Framework Activation Summary

**Activated bit 1 of the v3.0 XROM resolver chain — STAT_1 module const (id=2, "STAT 1B", 26 mnemonics) + stat1_resolve + bit-1 arm in xrom_resolve — and landed the structural fork-point for v3.1: rand_seed field with the unique `#[serde(default)]` (no `skip`) shape, `CalcState::migrate_after_load()` for v3.0 save-file upgrade, `ModalProgram::Stat1(Stat1Step)` carrier-enum variant (math1/ freeze exception D-33.3b), and `Op::Stat1Stub` placeholder variant referenced 26 times until Plans 33-03..33-08 replace each with a real `Op::Sigma*` variant.**

## Performance

- **Started:** 2026-05-22T~09:35:00Z (worktree spawn)
- **Completed:** 2026-05-22T10:12:00Z
- **Duration:** ~38 min
- **Tasks:** 4 (all completed atomically)
- **Files:** 1 created, 10 modified
- **Commits:** 5 (4 task commits + 1 deviation-fix commit)
- **Tests added:** 16 (4 state.rs + 4 xrom.rs + 4 stat1/modal.rs + 4 xrom_shadowing.rs)

## Accomplishments

- **XROM bit-1 activated end-to-end.** The bit-1 stub comment in `math1/xrom.rs:133-134` (live since v3.0) is now the real arm calling `stat1_resolve`. `xrom_resolve("ΣNORMD", 0b0000_0011)` returns `Some(Op::Stat1Stub)`; `xrom_resolve("ΣNORMD", 0b0000_0001)` returns `None` (bit-1 isolation invariant CI-gated in `tests::resolve_uses_bit_1_for_stat1`).
- **26-entry STAT_1.ops slice locked.** 14 QRC entry points + 10 secondary Σ-prefixed mnemonics + RAND + SEED. Σ encoded as `\u{03A3}` mirroring the existing `\u{00D7}` × convention in MATH_1.ops. Every entry maps to `Op::Stat1Stub` — Plan-33-01 scaffolding tracked at three code sites (Op enum doc-comment, STAT_1.ops doc-comment, stat1_resolve doc-comment) and at three documentation sites in this SUMMARY.
- **D-33.3 + D-33.3b freeze exceptions landed cleanly.** math1/ blast radius: `xrom.rs` (+195 lines), `modal.rs` (+31 lines), `mod.rs` (+2 lines for the submit_modal arm). Three files touched in the otherwise-frozen directory — `mod.rs` is a Rule-3 deviation (see "Deviations" below).
- **rand_seed field carries the unique serde shape.** `#[serde(default)]` WITHOUT `#[serde(skip)]` — the SOLE new v3.1 CalcState field with that shape. Doc-comment SHOUTS the unique shape and cites P20 trap. CI gate `rand_seed_serde_round_trip` asserts the field survives serialize→deserialize unchanged.
- **`CalcState::migrate_after_load()` is the single-source-of-truth migration hook.** Idempotent; called once after every deserialize() by Phase 34 (CLI) + Phase 36 (GUI). `v3_0_save_loads_with_stat_1_after_migration` test proves the P24-trap mitigation: synthetic v3.0 JSON blob with `"xrom_modules": 1` deserializes as `0b0000_0001`, then migrate_after_load() upgrades it to `0b0000_0011`.
- **4-way exhaustive-match invariant items 1+2 satisfied.** `Op::Stat1Stub` lands in `dispatch()` (item 1, `ops/mod.rs`) and `execute_op()` (item 2, `program.rs`). Items 3 (CLI `op_display_name`) and 4 (GUI `op_display_name`) are Phase 34 + 36 — EXPECTED `cargo check -p hp41-cli` failure on `non-exhaustive patterns: &Op::Stat1Stub not covered` (see "Known intentional CI break" below).
- **xrom_shadowing.rs CI gates extended.** 4 new tests cross-check STAT_1 against MATH_1 disjointness + builtin_card_op non-shadowing + bidirectional consistency with stat1_resolve.
- **Pitfall-16 meta-gate adapted to the D-33.3 freeze exception.** Rule-1 deviation fix: the `math1_op_test_count.rs::collect_math1_variant_names` heuristic scoped its scan to `fn math1_resolve` only, so the new sibling `stat1_resolve` arms don't pollute the variant list with false "Math Pac I variants" lacking ≥ 5 tests.

## Task Commits

Each task was committed atomically using English Conventional Commits (per CLAUDE.md "Git Workflow"):

1. **Task 1: Add rand_seed + flip default_xrom_modules + migrate_after_load** — `a60bffc` (feat)
2. **Task 2: Land math1/modal.rs freeze exception (D-33.3b) + Stat1Step scaffolding** — `ec4d072` (feat)
3. **Task 4: Add Op::Stat1Stub variant + 4-way invariant items 1+2** — `c9eca48` (feat)
4. **Task 3: Land math1/xrom.rs freeze exception (D-33.3) — STAT_1 + bit-1 arm** — `c330571` (feat)
5. **Deviation fix: scope math1_op_test_count variant scan to math1_resolve only** — `52124b7` (fix)

Task order in commits is 1 → 2 → 4 → 3 (Task 4 committed before Task 3) because Task 3's STAT_1.ops slice references `Op::Stat1Stub`, which Task 4 introduces — the inverse order would have produced a non-compiling intermediate state. Both orderings produce the same end state.

## Files Created / Modified

### Created (1 file)

- `hp41-core/src/ops/stat1/modal.rs` (121 lines) — `Stat1Step` placeholder enum + 3 dispatch functions + 4 unit tests; byte-for-byte contamination-guard disclaim header.

### Modified (10 files)

- `hp41-core/src/state.rs` (+150 lines):
  - `rand_seed: HpNum` field (`#[serde(default)]` WITHOUT skip, with the SHOUTING doc-comment).
  - `default_xrom_modules()` body flipped `0b0000_0001 → 0b0000_0011`.
  - New `impl CalcState { pub fn migrate_after_load(&mut self) { ... } }` block.
  - 4 new tests (`xrom_modules_default_is_three`, `v3_0_save_loads_with_stat_1_after_migration`, `migrate_after_load_idempotent`, `rand_seed_serde_round_trip`).
  - 2 existing tests updated to the new 0b0000_0011 default contract.

- `hp41-core/src/ops/math1/xrom.rs` (+195 lines, D-33.3 freeze exception):
  - `pub const STAT_1: XromModule { id: 2, name: "STAT 1B", ops: &[26 entries] }`.
  - `fn stat1_resolve(name: &str) -> Option<Op>` (exhaustive `_ => None`).
  - `xrom_resolve` bit-1 arm activated.
  - 4 new unit tests (`stat1_const_id_and_name`, `stat1_ops_has_correct_entry_count`, `stat1_ops_mnemonics_resolve_consistently`, `resolve_uses_bit_1_for_stat1`).

- `hp41-core/src/ops/math1/modal.rs` (+31 lines, D-33.3b freeze exception):
  - `ModalProgram::Stat1(crate::ops::stat1::modal::Stat1Step)` variant.
  - `current_prompt` arm delegating to `stat1::modal::current_prompt`.
  - `requires_alpha_label` refactored from `matches!` to explicit `match` with the new Stat1 arm.

- `hp41-core/src/ops/math1/mod.rs` (+2 lines, Rule-3 deviation):
  - `ModalProgram::Stat1(step)` arm in `submit_modal` exhaustive match.

- `hp41-core/src/ops/mod.rs` (+24 lines, 4-way invariant item 1):
  - `Op::Stat1Stub` variant at the end of the enum + 12-line scaffolding doc-comment.
  - `Op::Stat1Stub => crate::ops::stat1::op_stat1_stub(state)` dispatch arm.

- `hp41-core/src/ops/program.rs` (+6 lines, 4-way invariant item 2):
  - `Op::Stat1Stub => crate::ops::dispatch(state, Op::Stat1Stub)` execute_op arm.

- `hp41-core/src/ops/stat1/mod.rs` (+25 lines):
  - `pub mod modal;` declaration uncommented.
  - `pub fn op_stat1_stub(_state) -> Result<(), HpError> { Err(HpError::InvalidOp) }` with the Plan-33-01 scaffolding doc-block.

- `hp41-core/tests/xrom_shadowing.rs` (+67 lines, 4 new integration tests):
  - `stat1_names_do_not_shadow_builtins`, `stat1_ops_disjoint_from_math1_ops`, `stat1_ops_resolve_via_xrom_resolve`, `stat1_const_fields`.

- `hp41-core/tests/v3_save_compat.rs` (+5 / −1 line):
  - Updated `loads_synthetic_v22_save_without_v3_fields` to assert the new 0b0000_0011 default.

- `hp41-core/tests/math1_op_test_count.rs` (+30 / −9 lines, Rule-1 deviation fix):
  - `collect_math1_variant_names` scoped to `fn math1_resolve` only via brace-depth counter (excludes sibling `stat1_resolve` arms).

## Decisions Made

### Strategy A vs Strategy B for the stub-Op problem

Task 3's action documented two strategies for the `STAT_1.ops` placeholder Op problem. Strategy A (single placeholder variant referenced 26 times) was chosen because:

1. **Disjointness CI gate fires immediately.** With Strategy B (empty STAT_1.ops), the `xrom_shadowing.rs` Stat 1 disjointness tests would have nothing to iterate; the gate would defer to Plan 33-08 when the last real Op variant lands. Strategy A makes the gate active from Plan 33-01 onward, catching any future mnemonic collision the moment it's introduced.
2. **`stat1_ops_mnemonics_resolve_consistently` test is meaningful.** The bidirectional consistency between `STAT_1.ops` and `stat1_resolve` is non-trivially checked starting now — Plans 33-03..33-08 each touch the slice + match in lockstep, and this test surfaces any drift in either direction as a CI failure.
3. **End-to-end resolver chain plumbed.** `XEQ "ΣNORMD"` from a user program reaches the dispatcher and surfaces `HpError::InvalidOp` (rather than dispatching to a no-op) — the resolver-never-discard invariant from CLAUDE.md is preserved even during the scaffolding state.

The cost: a transient `Op::Stat1Stub` variant + helper function. Both are marked "TO BE REMOVED" with explicit Plan 33-08 ownership in their doc-comments.

### Stat1Step::Placeholder vs uninhabited enum

The empty-enum-with-`match *step {}` pattern is technically valid for uninhabited types, but clippy/rustc emit annoying lints around the surrounding code (`unused_variables`, `dead_code`). A single `Placeholder` variant is simpler:

- Each dispatch function (`submit_step`, `current_prompt`, `requires_alpha_label`) carries one `Placeholder => …` arm.
- Plan 33-03 replaces `Placeholder` with `NormdModeChoice` + `ChisqdNuPrompt` in a single edit per dispatch function.
- Plan 33-08 adds `PolypDegreePrompt` + `SeedPrompt` and sets `requires_alpha_label = true` for SeedPrompt.

### Three pre-existing tests updated to the new contract (NOT a regression)

`default_construction_phase28_fields` (state.rs), `v22_save_loads_with_defaults` (state.rs), and `loads_synthetic_v22_save_without_v3_fields` (tests/v3_save_compat.rs) all asserted `xrom_modules == 0b0000_0001`. With D-33.2 / STAT-FW-02 flipping the default to `0b0000_0011`, these assertions are now incorrect under the new contract — they were updated in-place to assert `0b0000_0011`, with inline comments citing the contract change. Backward compatibility for v3.0 *save files* (which explicitly persist `"xrom_modules": 1`) is preserved by `migrate_after_load()`, asserted by the new `v3_0_save_loads_with_stat_1_after_migration` test.

## Deviations from Plan

### Auto-fixed issues

**1. [Rule 1 — Bug] math1_op_test_count.rs over-counted Op::Stat1Stub as a Math Pac I variant**

- **Found during:** Final `cargo test -p hp41-core --tests` run after Task 3 + 4 commits.
- **Issue:** The `collect_math1_variant_names` helper scans `xrom.rs` for `Some(Op::...)` patterns to discover Math Pac I variants for the Pitfall-16 ≥-5-tests gate. After D-33.3 added a sibling `stat1_resolve` to the same file, the helper picked up 26 `Some(Op::Stat1Stub)` arms and the Pitfall-16 gate failed with "Op::Stat1Stub: only 0 test mention(s) in math1_*.rs (need ≥ 5)" 26 times.
- **Fix:** Scope the scan to the `fn math1_resolve` body only via a brace-depth counter. The sibling `stat1_resolve` arms (and any future `<module>_resolve` siblings) are now excluded by construction.
- **Files modified:** `hp41-core/tests/math1_op_test_count.rs` (+30 / −9 lines).
- **Commit:** `52124b7`.

**2. [Rule 3 — Blocking issue: exhaustive match] math1/mod.rs::submit_modal arm for ModalProgram::Stat1**

- **Found during:** Task 2 implementation.
- **Issue:** The plan's Task 2 acceptance criterion explicitly listed only `math1/modal.rs` and `math1/xrom.rs` in the math1/ touch budget (`git diff develop -- hp41-core/src/ops/math1/ | grep -E '^\+\+\+ b/' | sort -u` lists ONLY these two files). However, Task 2 step 3 ALSO instructed editing `hp41-core/src/ops/math1/mod.rs::submit_step()` dispatch table to add the new `ModalProgram::Stat1(step)` arm — required because the `match modal { ... }` in `submit_modal` is exhaustive and would not compile without it. D-33.3b's rationale (three-arm parity with the six existing variants) implicitly authorizes mod.rs as a third freeze-carved file, but it was not explicitly listed in D-33.3b's text.
- **Fix:** Single-line `ModalProgram::Stat1(step) => crate::ops::stat1::modal::submit_step(state, step)` arm added to `submit_modal`'s match block. The blast radius is +2 lines (arm + doc-comment); no semantic Stat 1 Pac code leaks into math1/.
- **Files modified:** `hp41-core/src/ops/math1/mod.rs` (+2 lines).
- **Commit:** `ec4d072` (combined with Task 2).
- **Documentation update needed:** Phase 35 CLAUDE.md amendment (STAT-DOC-05) should list math1/mod.rs alongside xrom.rs and modal.rs in the freeze-carve-out paragraph. Tracked as a known follow-up in this SUMMARY's "Next-phase readiness" section.

### Authentication gates encountered

None.

## Known intentional CI break

Per the plan's `<verification>` block: **`cargo check -p hp41-cli` and `cargo check -p hp41-gui` fail until Phase 34 / Phase 36 add items 3+4 of the 4-way exhaustive-match invariant for `Op::Stat1Stub`.** This is EXPECTED and confirmed:

```
$ cargo check -p hp41-cli
error[E0004]: non-exhaustive patterns: `&Op::Stat1Stub` not covered
   --> hp41-cli/src/prgm_display.rs
```

The failure is on `op_display_name`'s exhaustive match — exactly the contract that the 4-way invariant exists to enforce. Phase 34 + 36 will surface the missing arm and add the display string.

As a consequence, `just lint` (which runs `cargo clippy --workspace --all-targets --all-features`) fails at the hp41-cli compile step, and `just test` (which runs `cargo test --workspace`) cannot proceed past the hp41-cli build. Per-crate verification:

- `cargo check -p hp41-core` → exits 0 ✅
- `cargo clippy -p hp41-core --tests -- -D warnings` → exits 0 ✅
- `cargo test -p hp41-core --tests` → 1773 passed, 0 failed ✅
- `bash scripts/check-free42-contamination.sh` → exits 0 ✅
- `cargo check -p hp41-cli` → ❌ EXPECTED (Phase 34 wiring needed)
- `cargo check -p hp41-gui` → ❌ EXPECTED (Phase 36 wiring needed)

## Plan-level verification block — hp41-core scope

- ✅ `cargo check -p hp41-core` exits 0
- ✅ `cargo clippy -p hp41-core --tests -- -D warnings` exits 0
- ✅ `cargo test -p hp41-core --lib` — 626 passed (incl. 4 new state.rs + 4 new xrom.rs unit tests + 4 new stat1/modal.rs unit tests)
- ✅ `cargo test -p hp41-core --test xrom_shadowing` — 6 passed (incl. 4 new STAT_1 disjointness + consistency gates)
- ✅ `cargo test -p hp41-core --test v3_save_compat` — 2 passed (backward-compat preserved through default flip + migration)
- ✅ `cargo test -p hp41-core --test math1_op_test_count` — 1 passed (Pitfall-16 gate fixed via scoped scan)
- ✅ `cargo test -p hp41-core --tests` — 1773 passed across 70 test suites
- ✅ `bash scripts/check-free42-contamination.sh` — exits 0 (Free42 contamination guard clean on extended directory)
- ✅ `grep -c 'pub rand_seed: HpNum' hp41-core/src/state.rs` → 1
- ✅ `grep -c 'pub const STAT_1: XromModule' hp41-core/src/ops/math1/xrom.rs` → 1
- ✅ `grep -c 'pub fn migrate_after_load' hp41-core/src/state.rs` → 1
- ✅ `grep -c 'ModalProgram::Stat1' hp41-core/src/ops/math1/modal.rs` → 3 (variant + current_prompt arm + requires_alpha_label arm)
- ✅ `grep -c 'D-33.3b' hp41-core/src/ops/math1/modal.rs` → 3 (variant doc-comment + dispatch-arm citations)
- ✅ `git diff fb72604 HEAD -- hp41-core/src/ops/math1/` lists only `mod.rs`, `modal.rs`, `xrom.rs` (Rule-3 deviation: mod.rs added beyond the originally-budgeted 2 files)

## Self-Check: PASSED

Created files exist:
- ✅ `hp41-core/src/ops/stat1/modal.rs`
- ✅ `.planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-01-SUMMARY.md` (this file)

Commit hashes verified in `git log --oneline`:
- ✅ `a60bffc` feat(33-01): add rand_seed field + flip default_xrom_modules + migrate_after_load
- ✅ `ec4d072` feat(33-01): land math1/modal.rs freeze exception (D-33.3b) + Stat1Step scaffolding
- ✅ `c9eca48` feat(33-01): add Op::Stat1Stub variant + 4-way invariant items 1+2
- ✅ `c330571` feat(33-01): land math1/xrom.rs freeze exception (D-33.3) — STAT_1 + bit-1 arm
- ✅ `52124b7` fix(33-01): scope math1_op_test_count variant scan to math1_resolve only

## Next-phase readiness

- **Plan 33-02 (distribution primitives)** can now land `hp41-core/src/ops/stat1/distributions.rs` as the first concrete algorithm file. The contamination guard already scans the directory; the disclaim header pattern is locked; the f64-bridge primitives (Acklam/AS 241, AS 239, AS 63) plug into the existing module-hub at `stat1/mod.rs` (uncomment `pub mod distributions;` in the placeholders block).
- **Plan 33-03 (ΣNORMD + ΣCHISQD)** can now replace `Stat1Step::Placeholder` with `NormdModeChoice` and `ChisqdNuPrompt` variants in `hp41-core/src/ops/stat1/modal.rs`. The three dispatch functions (`submit_step`, `current_prompt`, `requires_alpha_label`) gain real bodies per OM 00041-90030. The `Op::SigmaNormdWorkflow` + `Op::SigmaChisqdWorkflow` variants land in `Op` enum + dispatch + execute_op; `STAT_1.ops` "ΣNORMD" / "ΣCHISQD" entries are switched from `Op::Stat1Stub` to the real variants; `stat1_resolve` arms updated in lockstep.
- **Plans 33-04..33-07** each replace specific `Op::Stat1Stub` references with real Op variants per the SPEC.md mnemonic→file table. The bidirectional `stat1_ops_mnemonics_resolve_consistently` test catches any drift between the slice and the match arms.
- **Plan 33-08 (final cleanup)** deletes `Op::Stat1Stub` from the Op enum, the dispatch arm, the execute_op arm, the `op_stat1_stub` helper, and the scaffolding doc-comments at all sites. Plan 33-08 is also responsible for confirming the `Stat1Step::Placeholder` variant has been fully replaced by NormdModeChoice / ChisqdNuPrompt / PolypDegreePrompt / SeedPrompt.
- **Phase 34 (CLI integration)** must FIRST land items 3 of the 4-way invariant — `op_display_name` arm for `Op::Stat1Stub` (and any subsequently-introduced `Op::Sigma*` variants) — to restore `cargo check -p hp41-cli` to green. Use the same string-mapping pattern as Math Pac I (e.g. `Op::SigmaNormdWorkflow => "ΣNORMD".to_string()`); during Phase 33's residual Op::Stat1Stub period, `Op::Stat1Stub => "ΣSTUB".to_string()` (or similar) is acceptable as long as it disappears with the variant deletion.
- **Phase 35 (docs)** must amend the CLAUDE.md "Frozen Invariants → Core engine" carve-out paragraph to list `xrom.rs`, `modal.rs`, **and** `mod.rs` as the three files carved out of the math1/ freeze (D-33.3 + D-33.3b + Rule-3 deviation documented in this SUMMARY).
- **Phase 36 (GUI integration)** must land item 4 of the 4-way invariant — `op_display_name` in `hp41-gui/src-tauri/src/prgm_display.rs`, mirror of Phase 34.

No blockers. Wave 1 complete; Plan 33-02 unblocked (Plan 33-03 needs Plan 33-02's distribution primitives first per the dependency chain).

---
*Phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1*
*Plan: 01 (Wave 1 XROM framework activation)*
*Completed: 2026-05-22*
