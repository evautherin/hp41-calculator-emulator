# Phase 33: hp41-core — XROM Activation + Distribution Primitives + All Stat 1 Ops - Context

**Gathered:** 2026-05-22
**Status:** Ready for spec-phase (then planning)

<domain>
## Phase Boundary

`hp41-core` only. Activate STAT_1 XROM (bit 1, id = 2) and deliver all v3.1 Stat 1 Pac functionality in the core library:

- XROM framework activation — `STAT_1: XromModule` const + `stat1_resolve()` + bit-1 arm in `xrom_resolve()`, fires LAST after MATH_1 (Pitfall 1)
- `default_xrom_modules()` migration `0b0000_0001 → 0b0000_0011` with backward-compat migration for v3.0 save files (Pitfall 24)
- Three hand-coded f64-bridge distribution primitives in `ops/stat1/distributions.rs` (Acklam/AS 241 `norm_cdf_inv_f64`, AS 239 `gamma_regularized_f64`, AS 63 `beta_regularized_f64`) — validated against scipy.stats oracle BEFORE any program `Op` is implemented
- All ~24 Op variants across new `ops/stat1/` module tree implementing 13 QRC programs (14 entry points): ΣBSTAT/ΣBSTG, ΣMMTUG/ΣMMTGD, ΣAOVONE/ΣAOVTWO/ΣANOCOV, ΣLIN/ΣEXP/ΣLOGI/ΣPOW, ΣMLRXY/ΣMLRXYZ, ΣPOLYP/ΣPOLYC, ΣPTST/ΣTSTAT, ΣXSQEV/ΣEFXSQ, ΣCTKKK/ΣCTKK, ΣSPEAR, ΣNORMD, ΣCHISQD
- `rand_seed: HpNum` `CalcState` field with `#[serde(default)]` WITHOUT `skip` (RAND/SEED emulator-extension bonus utility — community LCG formula per NPS p. 21)
- 4-way exhaustive-match invariant items 1 + 2 complete (`dispatch()` + `execute_op()`); items 3 (CLI) + 4 (GUI) are Phase 34 + 36
- `scripts/check-free42-contamination.sh` extended with stats-domain identifiers BEFORE first `stat1/*.rs` file lands (Pitfall 27)

**Out of scope (per REQUIREMENTS.md Out-of-Scope table):** F-distribution, Binomial, Poisson, Hypergeometric, histograms, P(N,R)/C(N,R) (NPS-confirmed absent from Stat 1 Pac); CLI integration (Phase 34); docs/ADRs (Phase 35); GUI integration (Phase 36); test-hardening proper (Phase 37); signed binary releases (deferred to v3.1.x / v3.2).

</domain>

<decisions>
## Implementation Decisions

### OM Convention Resolution Method

- **D-33.1:** Run `/gsd-spec-phase 33` BEFORE `/gsd-plan-phase 33`. Spec-phase locks the 5 pending OM conventions as LOCKED requirements in a `33-SPEC.md` that planner + executor cannot override. Mitigates Pitfall 21 (Σ-register layout silent-wrong-answer trap) by gating implementation behind a hard contract.
  - Items the spec-phase must resolve from OM 00041-90030 + QRC + NPS:
    1. Σ-register layout for ΣMMTUG/ΣMMTGD (Σx³, Σx⁴ storage), ΣAOVONE/ΣAOVTWO/ΣANOCOV (group sums), ΣMLRXY/ΣMLRXYZ (cross-products), ΣCTKKK/ΣCTKK (marginal totals)
    2. ΣPOLYP degree-prompt wording (tentative `DEGREE=?` per Math Pac I `POLY` precedent)
    3. ΣCHISQD ν entry convention (tentative: ν via `[A]` before x evaluation)
    4. ΣTSTAT pooled vs Welch variance assumption (NPS ZS-4/5 assumes pooled — verify)
    5. RAND/SEED subroutine presence in ROM listing (community-confirmed via NPS p. 21, but QRC does not list it)
    6. Quantile-loop convergence criteria (display-mode-tied like Math Pac I `integ_threshold()`, or fixed tolerance)
    7. Free42 stats-domain identifiers from `github.com/thomasokken/free42/blob/master/common/core_math2.cc` (concrete identifiers for contamination guard extension)

### Plan Slicing

- **D-33.2:** Phase 33 ships as 9 plans following research SUMMARY.md "Implementation order within phase" verbatim. Tight low-blast-radius PR sizes; each plan adds the next dependency layer.
  - Plan 33-00: Free42 stats-domain contamination guard extension + OM transcription artifact in `ops/stat1/mod.rs` header (Pitfall 27 + Pitfall 21 foundation, NO `Op` code yet)
  - Plan 33-01: XROM framework activation — `STAT_1: XromModule` const + `stat1_resolve()` + bit-1 arm in `math1/xrom.rs` + `default_xrom_modules()` flip to `0b0000_0011` + `CalcState::migrate_after_load()` + `rand_seed` field
  - Plan 33-02: `stat1/distributions.rs` 3 hand-coded primitives (Acklam, AS 239, AS 63) + scipy.stats inline-constants oracle GREEN before any program Op exists
  - Plan 33-03: ΣNORMD + ΣCHISQD (3 mode-dispatch + iterative quantile paths consuming primitives)
  - Plan 33-04: ΣSPEAR + ΣXSQEV / ΣEFXSQ (no distribution function; pure Σ-register arithmetic)
  - Plan 33-05: ΣBSTAT / ΣBSTG + ΣLIN / ΣEXP / ΣLOGI / ΣPOW (delegates to existing `op_sigma_plus()`)
  - Plan 33-06: ΣMMTUG / ΣMMTGD + ANOVA family (ΣAOVONE / ΣAOVTWO / ΣANOCOV) + ΣCTKKK / ΣCTKK (OM-register-layout-dependent set; consumes 33-00 transcription)
  - Plan 33-07: ΣPTST / ΣTSTAT (t-tests consuming `beta_regularized_f64`)
  - Plan 33-08: ΣMLRXY / ΣMLRXYZ + ΣPOLYP / ΣPOLYC + RAND / SEED (multiple regression + polynomial regression + RNG bonus if OM-presence-checked per D-33.4)

### Resolver Location (math1/ freeze exception)

- **D-33.3:** Break the `hp41-core/src/ops/math1/` freeze (locked at Plan 25-01) for `xrom.rs` only. Extend `math1/xrom.rs` in place with:
  - `pub const STAT_1: XromModule { id: 2, name: "STAT 1B", ops: &[...] }`
  - `fn stat1_resolve(name: &str) -> Option<Op>`
  - Bit-1 arm at line 134 (replacing the existing stub comment `// if modules & 0b0000_0010 != 0 { stat1_resolve(name) }`)
  - The bit-1 stub was always intended for v3.1+ per its inline comment; this is not an unplanned freeze violation but the realization of the stub's documented purpose.
- **D-33.3a:** Document the freeze exception in `CLAUDE.md` under the "Frozen Invariants → Core engine" section — `xrom.rs` is the registry for the resolver chain and was carved out for v3.1+ extension at v3.0 ship time. The rest of `math1/` (complex.rs, difeq.rs, four.rs, hyperbolics.rs, integ.rs, matrix.rs, mod.rs, poly.rs, solve.rs, trans.rs, tri.rs) remains strictly frozen — note that `modal.rs` is no longer in this list (see D-33.3b).
- **D-33.3b** (amended 2026-05-22, user-confirmed during `/gsd-plan-phase 33`): Break the `math1/` freeze a SECOND time for `modal.rs` only. Extend `hp41-core/src/ops/math1/modal.rs` in place with:
  - `ModalProgram::Stat1(stat1::modal::Stat1Step)` enum variant (one line, alongside `Matrix`, `Solve`, `Poly`, `Integ`, `Difeq`, `Four`)
  - Sibling dispatch arms for `submit_step()`, `current_prompt()`, and `requires_alpha_label()` — each delegating to the `stat1::modal` impls (3-arm parity with the existing 6 variants)
  - `Stat1Step` itself lives in the new `hp41-core/src/ops/stat1/modal.rs` so the math1/modal.rs delta stays to ~8 lines of pure dispatch wiring (no Stat 1 Pac semantics leak into the frozen module)
  - Update CLAUDE.md "Frozen Invariants → Core engine" to list `modal.rs` alongside `xrom.rs` as the two carved-out files; the rest of math1/ remains strictly frozen
  - Rationale: chosen over a parallel `Stat1ModalProgram` enum (would have required ~80 lines of plumbing across state.rs + commands.rs + app.rs to thread two modal-program enums) and over inline single-prompt Ops (would have diverged from OM 00041-90094 user flow for SEED, ΣPOLYP DEGREE=?, ΣCHISQD ν=? and risked failing SPEC.md acceptance criteria). The 8-line freeze amendment beats permanent infrastructure duplication.

### RAND / SEED Policy

- **D-33.4:** If `/gsd-spec-phase 33` OM read shows RAND is NOT a top-level Stat 1 Pac ROM entry point, implement RAND / SEED anyway as a v3.1 emulator extension (STAT-RNG-01..04 already drafts the requirements that way). Justification:
  - STAT-RNG-04 already mandates `docs/hp41-stat1-divergences.md` document it as an emulator extension — NOT part of the "feature-complete per OM 00041-90030" claim
  - LCG formula `r_{n+1} = FRC(9821·r_n + 0.211327)` is community-confirmed across three independent sources (NPS p. 21, HP-65 User's Library via Don Malm, HP-41C Standard Applications p. 24)
  - Persistence-surviving `rand_seed` is user-valuable for reproducible simulations
- **D-33.4a:** If `/gsd-spec-phase 33` OM read confirms RAND IS a top-level Stat 1 Pac ROM entry point, implement identically but treat it as OM-feature-complete (move out of the divergences "emulator extensions" bucket).

### `ops/stat1/` File Layout

- **D-33.5:** 9 files in `hp41-core/src/ops/stat1/`. Rename `tests.rs` → `hypothesis.rs` to avoid collision with the Rust convention `#[cfg(test)] mod tests` blocks (those exist in nearly every Stat 1 file).
  - `mod.rs` — module hub, exhaustive `pub use`, OM Σ-register layout transcription as `//!` header comment, `STAT1_MAX_REG: usize` const derived from OM table
  - `distributions.rs` — 3 hand-coded f64-bridge primitives (Acklam, AS 239, AS 63) + closed-form normal CDF / PDF / erf wrappers
  - `basic_stats.rs` — ΣBSTAT / ΣBSTG (univariate extended summary)
  - `moments.rs` — ΣMMTUG / ΣMMTGD (3rd / 4th moments, skewness, kurtosis)
  - `anova.rs` — ΣAOVONE / ΣAOVTWO / ΣANOCOV (group-data F-ratios)
  - `regression.rs` — ΣLIN / ΣEXP / ΣLOGI / ΣPOW + ΣMLRXY / ΣMLRXYZ + ΣPOLYP / ΣPOLYC (curve fits + multiple + polynomial)
  - `hypothesis.rs` — ΣPTST / ΣTSTAT (one- and two-sample t-tests) [renamed from `tests.rs`]
  - `nonparam.rs` — ΣSPEAR + ΣXSQEV / ΣEFXSQ + ΣCTKKK / ΣCTKK (nonparametric / chi-square accumulators)
  - `normd.rs` — ΣNORMD (3-mode dispatch: CDF / PDF / inverse)
  - `chisqd.rs` — ΣCHISQD (PDF / CDF using `gamma_regularized_f64`)
  - `rand.rs` — RAND / SEED (LCG implementation, `rand_seed` access) — exists only after Plan 33-08
  - Note: research SUMMARY.md proposed 9 files combining moments into basic_stats; Phase 33 splits moments out because ΣMMTUG / ΣMMTGD depend on OM-layout-extended registers (different from ΣBSTAT). Final count: 10 files + `mod.rs` = 11 files. The split keeps Plan 33-05 and Plan 33-06 disjoint.

### scipy.stats Oracle Data Format

- **D-33.6:** Inline test constants in `#[cfg(test)] mod tests` of `stat1/distributions.rs`. Hard-code ~30 `(input, scipy_expected, tolerance)` tuples per primitive directly in the file. Mirrors how `math1/poly.rs` and `math1/integ.rs` carry their oracle constants.
  - No external fixture file, no Python toolchain dependency, no JSON drift.
  - Oracle frozen at write time — values produced once via a one-shot scipy.stats Python snippet in the plan comments, never re-generated.
  - Tolerance follows the two-level policy from STAT-QUAL-05: 1e-9 for closed-form, 1e-7 for iterative.

### State Migration Location

- **D-33.7:** Migration logic in `hp41-core/src/state.rs`. Single source of truth, called once after every deserialization by both `hp41-cli/src/persistence.rs::load_state` and `hp41-gui/src-tauri/src/persistence.rs`.
  - Add `impl CalcState { pub fn migrate_after_load(&mut self) { ... } }`
  - Migration body: `if self.xrom_modules & 0b0000_0010 == 0 { self.xrom_modules |= 0b0000_0010; }` — applied unconditionally on load (idempotent; no-op on already-migrated state)
  - Re-save happens on the next 30s auto-save tick or exit save; no explicit re-save call from `migrate_after_load`
  - Rejected: hidden `#[serde(default)]` migration would silently override users who explicitly disabled Stat 1; per-frontend duplication risks divergence

### Free42 Contamination Guard Timing

- **D-33.8:** Move STAT-QUAL-09 from Phase 37 to Phase 33 (Plan 33-00). REQUIREMENTS.md traceability table updated during spec-phase or plan-phase. Phase 37 still owns STAT-QUAL-01..08, STAT-QUAL-10..11.
  - Rationale: STAT-QUAL-09 requirement text already says "verified BEFORE first stat1/*.rs file lands" — the requirement itself orders the work into Phase 33.
  - Plan 33-00 deliverable: `scripts/check-free42-contamination.sh` extended with stats-domain identifiers (concrete list extracted from `github.com/thomasokken/free42/blob/master/common/core_math2.cc` during spec-phase per D-33.1).
  - Phase 37 retains responsibility for re-verifying in CI context with all `stat1/*.rs` files present.

### Claude's Discretion

- Concrete identifier list for the contamination-guard extension (D-33.8): spec-phase resolves the candidate list; planner / executor finalizes the regex pattern.
- ANOVA / regression result-output shape (stack push vs print buffer vs both): defer to OM "Results" section read in spec-phase; if OM is silent, follow the Math Pac I `POLY` pattern (stack pushes for primary numerical results, print buffer for prompts).
- Per-Op `LiftEffect::Enable / Disable / Neutral` declarations: follow HP-41 stack-lift semantics conventions established in v1.0 + reaffirmed in v3.0 ADR-001.
- Inline test count per Op: meet `stat1_op_test_count.rs` floor of ≥ 5 (Phase 37 meta-gate); aim for ~6–8 to leave margin.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project planning artifacts (LOCKED)
- `.planning/PROJECT.md` — v3.1 milestone scope, locked 2026-05-21; build sequence; out-of-scope list
- `.planning/REQUIREMENTS.md` — 66 v3.1 requirements; Phase 33 owns 39 (STAT-FW-01..04, STAT-UNI-01..04, STAT-AOV-01..04, STAT-REG-01..09, STAT-HYP-01..07, STAT-DST-01..07, STAT-RNG-01..04); two-level tolerance policy; anti-features per NPS document
- `.planning/ROADMAP.md` — Phase 33 success criteria; phase dependency chain; phase ordering rationale
- `.planning/STATE.md` — v3.1 phase overview; pending OM decisions; carried-forward decisions; critical implementation traps
- `.planning/research/SUMMARY.md` — implementation order within phase (9-step sequence used by D-33.2); architecture approach; 5 critical pitfalls; confidence assessment; source list
- `.planning/research/ARCHITECTURE.md` — XROM ID 2 confirmation; bit-1 stub location; `XromModule` struct shape; `ops/stats.rs` R01–R06 layout; OnceLock pattern in `help_data.rs`
- `.planning/research/FEATURES.md` — 13 QRC programs detailed; 14 entry points; LOW/MEDIUM/HIGH complexity classification; anti-feature list
- `.planning/research/PITFALLS.md` — P18 (variance stability), P19 (quantile convergence), P20 (RNG serde), P21 (OM register layout), P22 (mnemonic shadowing), P24 (xrom_modules migration), P25 (cancellation), P27 (Free42 stats contamination guard)
- `.planning/research/STACK.md` — `statrs` rejection rationale; `rust_decimal 1.42` `MathematicalOps` coverage; f64-bridge pattern reference

### Project conventions (LOCKED)
- `CLAUDE.md` — Frozen Invariants (workspace structure, core engine, 4-way exhaustive-match, resolver chain, save-file backward compat, JSON canonical data flow, CLI ↔ GUI parity, GUI specifics, Free42 GPL-contamination guard); Quality Gates; Key Files
- `docs/architecture-history.md` — v3.0 §Phase 28 (XROM framework / `xrom_resolve` LAST-fires invariant / user-callback strict-reject); v3.0 §Phase 29 (CLI integration playbook); v3.0 §Phase 30 (docs-matrix two-input extension); v3.0 §Phase 31 (GUI playbook + `request_cancel` cancellation channel + Pitfall 11); v3.0 §Phase 32 (test-hardening / Free42 contamination guard / coverage gate atomic raise)
- `docs/hp41-math1-divergences.md` — three-bucket divergence catalog format (D-XX-NN); precedent for v3.1 sibling
- `docs/adr/v3.0-001-op-strategy.md` — Op-strategy A (one Op variant per function); rejected `Op::XromCall(u16)` table dispatch
- `docs/adr/v3.0-002-user-callback-policy.md` — strict-reject nested user callback (extended by self-contained iteration in v3.1)
- `docs/adr/v3.0-005-json-pipeline.md` — separate JSON file per XROM module pattern (D-29.1 precedent for `docs/hp41-stat1-functions.json`)

### External sources (read during planning / execution)
- HP-41C Stat Pac Quick Reference Card 00041-90061 (June 1979) — `literature.hpcalc.org/community/hp41-pac-stat-qrc-en.pdf` — all 13 programs / mnemonics / SIZE / I/O columns
- HP-41C Stat 1 Pac Owner's Manual 00041-90030 (June 1979) — `literature.hpcalc.org` item 800 — MUST be read in spec-phase for D-33.1 items 1–6
- Naval Postgraduate School NPS55-84-003 (Zehna, February 1984, DTIC AD-A140573) — RNG formula confirmation, anti-feature confirmation, t-test variant, ΣCHISQD ν convention
- `calc.fjk.ch/db/hp41mod.php` — "Statistics Pac 1B" XROM #2 confirmation
- `github.com/thomasokken/free42/blob/master/common/core_math2.cc` — read in spec-phase for D-33.8 concrete identifier list
- `docs.rs/rust_decimal/latest/rust_decimal/trait.MathematicalOps.html` — verify `norm_cdf` / `norm_pdf` / `erf` availability (2026-05-21 snapshot used; re-verify in plan-phase if a `rust_decimal` minor bump lands)
- `stackedboxes.org/2017/05/01/acklams-normal-quantile-function/` — Acklam / AS 241 inverse normal rational approximation reference

### Codebase landmarks (read during execution)
- `hp41-core/src/ops/math1/xrom.rs:127–136` — current `xrom_resolve()` body and bit-1 stub comment (extension target per D-33.3)
- `hp41-core/src/state.rs:162–164,227–228,266` — `xrom_modules: u8` field + `default_xrom_modules()` (modification target per D-33.7)
- `hp41-core/src/state.rs:337,381,404,476` — existing `xrom_modules` tests (extend with `migrate_after_load()` round-trip per D-33.7)
- `hp41-core/src/ops/stats.rs` — existing R01–R06 Σ-register layout (Stat 1 EXTENDS this; layout transcribed from OM in Plan 33-00)
- `hp41-core/src/num.rs:184–211` — f64-bridge pattern (`checked_asin` / `checked_acos` / `checked_atan`) — model for `norm_cdf_inv_f64` / `gamma_regularized_f64` / `beta_regularized_f64`
- `hp41-core/src/ops/math1/poly.rs` + `hp41-core/src/ops/math1/integ.rs` — oracle-constants pattern (inline test tuples) referenced by D-33.6
- `hp41-core/src/ops/math1/mod.rs` — module-hub shape; `ops/stat1/mod.rs` mirrors this
- `scripts/check-free42-contamination.sh` — 12-symbol pattern (extension target per D-33.8)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`xrom_resolve()` chain** (`math1/xrom.rs:127`) — already fires LAST after `builtin_card_op` at both insertion sites in `program.rs`; STAT_1 plugs into bit-1 arm with zero resolver-order risk
- **`XromModule` struct** (`math1/xrom.rs`) — `pub struct XromModule { id: u16, name: &'static str, ops: &[(&'static str, Op)] }`; reused unchanged for `STAT_1`
- **`#[serde(default)]` + `#[serde(default = "...")]` pattern** (`state.rs:163`) — backward-compat machinery already wired for every CalcState extension since v1.0; STAT-FW-02 leverages this for the new `rand_seed` field
- **`request_cancel: Arc<AtomicBool>`** (Phase 31 v3.0) — per-loop cancellation check; STAT-DST-07 reuses it for ΣNORMD inverse + ΣCHISQD CDF iterative loops (Pitfall 11 mitigation extended)
- **`rust_decimal::MathematicalOps`** — `norm_cdf`, `norm_pdf`, `erf`, `ln`, `exp`, `sqrt`, `powi`, `checked_div` — covers closed-form needs; gaps fill via f64-bridge
- **`OnceLock<Vec<HelpEntry>>` pattern** (`help_data.rs`) — third Stat 1 pool wires identically to v3.0 Math Pac I JSON pool (Phase 34 work; Phase 33 only writes the JSON source)
- **`call_dispatch_and_drain()` + `drain_and_show_print_output()`** (CLI) — modal-prompt path already exists for Math Pac I `DEGREE=?`, `ORDER=?`, etc.; ΣPOLYP / SEED prompts reuse this (Phase 34 work)
- **Free42 contamination guard** (`scripts/check-free42-contamination.sh`) — 12-symbol grep pattern + `just license-audit` recipe + `ci.yml::license-audit` parallel job all already wired; Plan 33-00 extends the symbol list only

### Established Patterns

- **4-way exhaustive-match invariant** — `Op` variants land in `dispatch()` + `execute_op()` + cli `op_display_name()` + gui `op_display_name()` before any caller compiles. Phase 33 owns items 1 + 2; items 3 + 4 are Phase 34 + 36.
- **Modal-workflow re-entry** (Phase 28 v3.0) — Math Pac I ALPHA-prompt-driven multi-step flows use `print_buffer` + `modal_program` transient state. Stat 1 prompts (ΣPOLYP `DEGREE=?`, SEED `SEED?`, ΣCHISQD `ν=?` per D-33.1 item 3) reuse the same machinery; zero new transient `CalcState` fields beyond `rand_seed`.
- **Self-contained iteration vs. user-callback** — distribution-quantile loops use the POLY pattern (private iteration with per-loop `cancel_requested` check), NOT the SOLVE/INTG user-callback pattern. Locked decision; eliminates the need for new `*_state` CalcState fields.
- **No `println!` / `eprintln!` in `hp41-core`** — every side effect routes through `print_buffer` or `event_buffer`. Stat 1 prompts (`SEED?`, etc.) push into `print_buffer`.
- **`#![deny(clippy::unwrap_used)]`** — `.expect("reason")` or `?`-propagation only; test modules carry `#[allow(clippy::unwrap_used)]`. f64-bridge primitives carry domain-error returns instead of panics.
- **ISG/DSE counter string-split** (`ops/program.rs::parse_counter`) — irrelevant to Stat 1 directly but referenced in CLAUDE.md as the canonical "never `floor()`/`fmod()` on f64" admonition; analog: every Σ-register-layout access (Plan 33-06 / 33-08) reads from `state.regs[STAT1_MAX_REG]` with explicit index, never computed-from-float.
- **Inline oracle constants** (`math1/poly.rs`, `math1/integ.rs`) — the precedent for D-33.6; Stat 1 follows.

### Integration Points

- **`hp41-core/src/ops/mod.rs`** — `pub mod stat1;` added in Plan 33-01; mounts the new module tree
- **`hp41-core/src/ops/mod.rs::dispatch()`** — exhaustive match gains ~24 new `Op::Stat*` arms (Plan 33-01..33-08)
- **`hp41-core/src/ops/program.rs::execute_op()`** — same ~24 arms (Plan 33-01..33-08)
- **`hp41-core/src/ops/math1/xrom.rs:127`** — `xrom_resolve()` body gains bit-1 arm; new `STAT_1` const + `stat1_resolve()` (Plan 33-01)
- **`hp41-core/src/state.rs:227`** — `default_xrom_modules()` flipped to `0b0000_0011`; `migrate_after_load()` method added (Plan 33-01)
- **`hp41-core/src/state.rs`** — new `rand_seed: HpNum` field with `#[serde(default)]` (Plan 33-01)
- **`hp41-core/src/ops/stats.rs`** — read-only reference for Σ-register layout (R01–R06 existing); Stat 1 extends but never modifies existing fields
- **`scripts/check-free42-contamination.sh`** — pattern extended; `just license-audit` + `ci.yml::license-audit` continue to gate (Plan 33-00)
- **`hp41-core/tests/*.rs`** — new files per Plan to test new Ops; coverage gate (≥ 95.39 % lines / ≥ 94.26 % regions) is a Phase 37 verification target but every Plan keeps it satisfied locally

</code_context>

<specifics>
## Specific Ideas

- **OM transcription as `mod.rs` header** — Plan 33-00 transcribes the Stat 1 Pac OM "Storage Registers" section verbatim into `ops/stat1/mod.rs` as a `//!` doc comment with `STAT1_MAX_REG: usize` const derived from the table. Subsequent plans reference this single source of truth.
- **Spec-phase first, then plan-phase** — user-locked workflow. `/gsd-spec-phase 33` produces `33-SPEC.md` with the 7 LOCKED items from D-33.1; `/gsd-plan-phase 33` consumes SPEC.md and produces the 9 plans per D-33.2.
- **Plan 33-00 has no Op code** — it ships two deliverables only: (a) extended `check-free42-contamination.sh` symbol list; (b) `ops/stat1/mod.rs` skeleton with OM register-layout comment + `STAT1_MAX_REG` const + empty `pub use` block. This isolates the contamination-guard work from any code that could trip it.
- **Inline scipy oracle generation** — Plan 33-02 carries a fenced Python comment block above each oracle-constant array: `# scipy.stats.norm.ppf(0.025) = -1.959963984540054`. Re-derivable from the plan; not committed as a runnable Python file.

</specifics>

<deferred>
## Deferred Ideas

- **ANOVA / regression result-output shape** — F-ratio + group means: stack push vs print buffer vs both. Pinned to spec-phase OM read; planner decides per OM "Results" section.
- **Per-Op `LiftEffect` declarations** — follow HP-41 hardware semantics; expected to mirror Math Pac I family precedents (POLY / MATRIX / INTG one-shot vs hyperbolics neutral-then-replace). Locked at plan time per Op.
- **`docs/hp41-stat1-divergences.md` D-33-NN numbering scheme** — Phase 35 work; first D-33-NN entries come from D-33.1 items 1–6 (whichever resolve as divergences) + D-33.3a (math1/ freeze exception) + D-33.4 (RAND emulator-extension claim) + D-33.8 (STAT-QUAL-09 phase reassignment).
- **`statrs` re-evaluation if `rust_decimal` adds `MathematicalOps::norm_cdf_inv` in a future minor** — locked rejected for v3.1; would warrant a v3.2+ revisit if the upstream API gains parity.
- **Histogram / frequency-table program** — out of scope per NPS confirmation that Stat 1 Pac does not include it. If pursued, separate future milestone.
- **Stat 2 Pac functions** — extended ANOVA, regression diagnostics — separate HP module; if pursued, separate future milestone after v3.x.

</deferred>

---

*Phase: 33-hp41-core-xrom-activation-distribution-primitives-all-stat-1*
*Context gathered: 2026-05-22*
