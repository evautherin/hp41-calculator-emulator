# HP-41 Calculator Emulator — Project Guide

## What this is

Faithful Rust behavioral emulation of the HP-41C/CV/CX RPN calculator.

- `hp41-core` — UI-agnostic library; zero CLI/UI deps (enforced at compile time)
- `hp41-cli` — TUI binary (ratatui 0.30 + crossterm 0.29)
- `hp41-gui` — Tauri v2 + React desktop app (nested standalone workspace)

**Current:** v3.2 Time Pac (Owner's Manual 00041-90035 feature-complete), shipped 2026-05-25. Earlier tags: `v3.1` (Stat 1 Pac, 2026-05-24), `v3.0` (Math Pac I, 2026-05-21), `v2.2` (HP-41CV complete, 2026-05-16), `v2.0` (Tauri GUI, 2026-05-10), `v1.1`, `v1.0`.

**Where the long-form history lives:**
- `docs/architecture-history.md` — full phase-by-phase narrative + decision rationale (Markdown fallback for reviewers without gbrain)
- `docs/adr/` — numbered ADRs for v3.0 onward (Op-strategy, user-callback policy, JSON-pipeline shape)
- `docs/hp41-math1-divergences.md` — Math Pac I OM divergences + emulator extensions
- `.planning/milestones/` — archived GSD planning artifacts per shipped milestone
- gbrain: `gbrain search "<terms>"` from anywhere inside this repo (the `.gbrain-source` worktree pin routes automatically — no `--source` flag needed). For symbol-aware lookup: `gbrain code-def <name>` / `gbrain code-refs <name>` / `gbrain code-callers <name>`.

## Git Workflow

- **Commits:** always `/git-workflow:commit --with-skills` — never bare `git commit`.
- **Language:** English only (subject + body), regardless of plugin defaults.

## GSD Workflow

Planning artifacts in `.planning/`. Shipped milestones archived under `.planning/milestones/`.

```
/gsd-progress           — current status
/gsd-new-milestone      — start next milestone
```

## Frozen Invariants

These are final — `docs/architecture-history.md` documents how each came to be. **Do not revisit without strong justification.**

### Workspace structure

*Origin: `architecture-history.md` §v2.0 additions (workspace isolation, SC-4, bundle ID) + §v1.1 additions (MSRV).*

- `hp41-core` must never depend on `hp41-cli` or `hp41-gui`. Root `Cargo.toml` members stay `["hp41-core", "hp41-cli"]`; `hp41-gui` is a nested standalone workspace. `tauri` / `tauri-build` appear ONLY in `hp41-gui/src-tauri/Cargo.toml`.
- **SC-4 (no core duplication in GUI):** stricter check `grep -rn "fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)" hp41-gui/src-tauri/src/` must return empty. Display helpers like `op_display_name` are exempt by intent (and duplicated CLI ↔ GUI by design).
- **MSRV 1.88** declared at `[workspace.package]`; member crates inherit via `rust-version.workspace = true`. CI MSRV job runs in parallel — no `needs:`.
- **Bundle ID:** `ch.talent-factory.hp41` (avoid macOS sandbox/keychain issues).

### Core engine

*Origin: `architecture-history.md` §Core engine (v1.0) + §v1.1 additions (print emulation) + §v2.2 / Phases 20–25 (math1 freeze).*

- **BCD/f64:** `rust_decimal` 1.42 with 10-significant-digit rounding. `HpNum` in `hp41-core/src/num.rs`. Custom BCD was evaluated and rejected.
- **Stack-lift:** every op declares `LiftEffect::Enable / Disable / Neutral`. The most commonly mis-implemented HP-41 feature — always check.
- **ISG/DSE counter:** extract fields by string-splitting at the decimal point — **never** `floor()`/`fmod()`. See `ops/program.rs::parse_counter()`.
- **No async, no panics:** `#![deny(clippy::unwrap_used)]` at crate root. Production code uses `.expect("reason")` or `?`-propagation. Test modules carry `#[allow(clippy::unwrap_used)]`. `hp41-gui` mutex locks use `.unwrap_or_else(|e| e.into_inner())` for poisoned-lock recovery.
- **Print emulation:** `println!`/`eprintln!` are forbidden in `hp41-core`. PRX/PRA/PRSTK push into `state.print_buffer` (`#[serde(skip)]`); CLI drains via `call_dispatch_and_drain()` (interactive) and `drain_and_show_print_output()` (programmatic paths) — wire ALL `run_program()` call sites or print output gets dropped.
- **`hp41-core/src/ops/math1/` is frozen** since Plan 25-01. Math Pac I algorithms re-derived from HP OM 00041-90034 (1979); Free42 consulted as sanity-check oracle only, **not** copied. Every file in this directory carries the verbatim disclaim header. **Exception (v3.1):** `xrom.rs` (D-33.3 / [ADR-v3.1-004](docs/adr/v3.1-004-math1-freeze-second-carve-out.md) — XROM registry for v3.1+ extension; bit-1 stub was always intended for v3.1+) and `modal.rs` (D-33.3b / ADR-v3.1-004 — ~8-line dispatch wiring for `ModalProgram::Stat1` variant; `Stat1Step` semantics live in the new `stat1/modal.rs` so no Stat 1 Pac code leaks into the frozen module). All OTHER files in `math1/` remain frozen.

### 4-way exhaustive-match invariant

*Origin: `architecture-history.md` §v2.0 additions ("Op variants land before TUI code").*

Every new `Op` variant must land in ALL FOUR before any caller compiles:

1. `dispatch()` in `hp41-core/src/ops/mod.rs`
2. `execute_op()` in `hp41-core/src/ops/program.rs`
3. `op_display_name()` in `hp41-cli/src/prgm_display.rs`
4. `op_display_name()` in `hp41-gui/src-tauri/src/prgm_display.rs`

Items 3 + 4 are duplicated by design; the compile-time exhaustive match in both is what makes this invariant load-bearing.

### Resolver chain + never-discard (D-07)

*Origin: `architecture-history.md` §v2.1 additions (D-07 stub-error pattern) + §v2.2 / Phase 25 (`builtin_card_op` 4→12 extension) + §v3.0 / Phase 28 (`xrom_resolve` fires last).*

Order in `hp41-cli` keyboard path: `key_to_op` → `shifted_key_to_op` → modal-opener → `xeq_by_name_local_resolve` → `builtin_card_op` → `xrom_resolve` → `Err(InvalidOp)`.

`pending_input` routing block must remain **above** modal-opening interceptors (`S`/`R`/`Ctrl+A`, R/S submit, Esc cancel) so an active modal is never silently discarded. NEVER silently swallow an unknown id — surface as a toast (GUI) or status-bar error (CLI). v2.1 stub-error pattern: `key_map::resolve` returns `Err(GuiError { message: "'<id>' is planned for a future phase" })` rather than mapping to a no-op.

### Save-file backward compat

*Origin: `architecture-history.md` §v1.1 additions (`serde(default)` pattern) onward; every milestone reaffirms it.*

Every new `CalcState` field carries `#[serde(default)]`. Transient fields also carry `#[serde(skip)]` (`print_buffer`, `modal_prompt`, `modal_program`, `integ_state`, `solve_state`, `cancel_requested`). v1.0–v3.0 save files load without migration.

### JSON canonical data flow

*Origin: `architecture-history.md` §v2.2 / Phase 25 (D-25.16 hp41cv-functions.json) + §v3.0 / Phase 29 (D-29.1 sibling hp41-math1-functions.json).*

- `docs/hp41cv-functions.json` + `docs/hp41-math1-functions.json` are the single sources of truth for keybindings, the `?` overlay help, and the right-panel discoverability table.
- Loaded via `include_str!` + `OnceLock` in `hp41-cli/src/help_data.rs` (`HELP_ENTRIES`, `MATH1_HELP_ENTRIES`, merged `help_entries_all()`). Malformed JSON panics at first access — hard-build-blocker by design.
- `scripts/docs-matrix/` (standalone non-workspace crate) regenerates `docs/hp41cv-function-matrix.md` + `docs/hp41-math1-function-matrix.md` via `just docs-matrix`. CI drift-catch: `just docs-matrix-check`.
- Bidirectional Op ↔ JSON parity asserted by `hp41-cli/tests/function_matrix_parity.rs`; key-coverage closure by `hp41-cli/tests/key_coverage.rs` (no `InvalidOp`, no panics for any keyboard-reachable JSON entry).
- Right-panel filter: `key_ref_entries()` excludes XROM-module functions via `entry.xrom.is_none()` (post-v3.0 UX revert — module functions stay discoverable via `?` overlay's "Math 1 Pac (XROM 7)" section).

### CLI ↔ GUI parity (D-25.6)

*Origin: `architecture-history.md` §v2.1 additions (`shiftActive` frontend-only) + §v2.2 additions (D-25.6 parity invariant, D-25.12 IND-toggle).*

- **One-shot SHIFT** is frontend-only. `App.shift_armed` (CLI) and `shiftActive` (GUI) mirror each other bit-for-bit. Neither ever appears in `CalcState` / `CalcStateView` / IPC.
- ALPHA overrides SHIFT (known divergence from real HP-41, accepted).
- IND-toggle (shift-0 inside an open Flag/Register modal) is hardware-faithful per HP-41C/CV QRG p.14; reuses the same `shift_armed` bit.

### GUI specifics

*Origin: `architecture-history.md` §v2.0 additions (IPC, Tauri permissions, SVG, busyRef, persistence) + §v3.0 / Phase 28 (no-polling D-11, `request_cancel`).*

- **IPC contract:** `dispatch_op(key_id: &str)`, `get_state()`, `sst_step`, `bst_step`, `run_stop`, `request_cancel` — Tauri v2 commands. Response is `CalcStateView` (~170 bytes, JSON ≤300 bytes). Frontend never touches Rust enums; `key_map::resolve()` translates string IDs.
- **Tauri v2.11 permissions:** for inline app commands (not plugins), Tauri does NOT auto-generate `allow-<cmd>` permissions. Create TOML in `hp41-gui/src-tauri/permissions/<cmd-kebab>.toml` with `[[permission]] identifier + commands.allow = ["fn_name"]`, then reference the kebab-case ID in `capabilities/default.json`. Run a `cargo check` first so the permission registry is generated.
- **SVG animation:** `.key` needs `transform-box: fill-box` + `transform-origin: center` — without it, SVG `scale()` translates from the canvas origin instead of shrinking in place.
- **busyRef debounce:** two-layer `useRef(false)` guard against concurrent `invoke()` in `App.tsx` (`handleClick`) and `Keyboard.tsx` (`handleKeyClick`). Pair with `pressedKey` functional setState to avoid stale closure.
- **No polling (D-11):** frontend does not poll `get_state()`. Out-of-band `invoke()` calls (e.g. from E2E `browser.execute`) do not propagate to the React tree — E2E asserts on the `dispatch_op` response's `CalcStateView.display_str`.
- **Persistence sharing:** `hp41-gui` and `hp41-cli` read/write the SAME `~/.hp41/autosave.json`. Auto-save thread releases the `AppState` Mutex BEFORE disk I/O.

### Free42 GPL-contamination guard

*Origin: `architecture-history.md` §v3.0 / Phase 28 (ADR-002 disclaim policy) + §v3.0 / Phase 32 (`check-free42-contamination.sh` CI gate).*

CI-enforced via `scripts/check-free42-contamination.sh` in `just license-audit` + dedicated `.github/workflows/ci.yml::license-audit` parallel job. Greps for 12 distinctive Free42 / Intel BID / decNumber / GPL/AGPL identifiers; bare `Free42` excluded from the pattern because legitimate cross-check references exist.

### v3.1 additions (Stat 1 Pac Emulation, Phases 33–37)

*Origin: see `docs/architecture-history.md` §v3.1 additions for the long-form per-phase narrative; this block is the CLAUDE.md decision-summary surface (the FIRST-EVER `### v3.x additions` block per D-35.5; v3.0 narrative lives only in `architecture-history.md`).*

#### Phase 33 — XROM Activation + Distribution Primitives + All Stat 1 Ops (shipped 2026-05-22)

- **STAT_1 XromModule (XROM ID 2) registered (STAT-FW-01 / D-33.3):** `MATH_1.id = 7` + `STAT_1.id = 2` per hardware records; `xrom_resolve` bit-1 arm fires LAST after bit-0 (Pitfall 1 + Pitfall 22 preserved).
- **`default_xrom_modules() = 0b0000_0011` + `migrate_after_load()` (STAT-FW-02 / D-33.7):** v3.0 save files with `xrom_modules: 1` auto-upgrade at startup; `state.rs:386-391` is the canonical migration site (D-33.7 single-source-of-truth — both CLI and GUI persistence layers call it).
- **3 hand-coded distribution primitives ([ADR-v3.1-002](docs/adr/v3.1-002-distribution-primitives-policy.md) / D-33.5):** `norm_cdf_inv_f64` (Acklam/Wichura AS 241), `gamma_regularized_f64` (Cody AS 239), `beta_regularized_f64` (Lentz AS 63); ~140 LOC total in `hp41-core/src/ops/stat1/distributions.rs`; `statrs` runtime dep REJECTED per `.planning/research/SUMMARY.md` (zero new runtime deps); Free42 disclaim verbatim per Pitfall 19 ("Algorithm independently re-derived from primary sources (Wichura AS 241 / Cody AS 239 / Lentz AS 63); Free42 source consulted only as sanity-check oracle, not copied."); `scripts/check-free42-contamination.sh` extended from 12 → 18 tokens (D-32.7 reassignment, STAT-QUAL-09 met in Plan 33-00).
- **OM Storage Registers transcription policy ([ADR-v3.1-003](docs/adr/v3.1-003-anova-register-layout.md) / D-33.1 / D-33.5):** `hp41-core/src/ops/stat1/mod.rs` `//!` doc-comment header + ~30 named consts (`STAT1_AOV_*_REG`, `STAT1_MLRXY_*_REG`, `STAT1_CTKKK_*_REG`, etc.); P21 silent-wrong-answer trap mitigated before any Op was written; all ANOVA / regression / contingency-table register accesses go through named consts (REVIEW WR-01 fix in `anova.rs` commit `b29aa03`).
- **`rand_seed: HpNum` on CalcState with `#[serde(default)]` WITHOUT `#[serde(skip)]` ([ADR-v3.1-001](docs/adr/v3.1-001-rng-state-placement.md) / D-33.4 / D-33.4a):** the ONLY v3.1 CalcState field with this serde shape (Pitfall 20 muscle-memory trap mitigated — every other v3.1 transient field follows the standard `skip` pattern); reproducible RNG across save/load cycles per STAT-RNG-03; field at `state.rs:201-202`; `rand_seed_serde_round_trip` test is the field-level CI guard. First-call-from-default-zero deterministic output `0.211327` documented as D-35-12 in `docs/hp41-stat1-divergences.md` (routes 33-REVIEW.md IN-05).
- **`math1/` freeze second carve-out ([ADR-v3.1-004](docs/adr/v3.1-004-math1-freeze-second-carve-out.md) / D-33.3 + D-33.3b):** `xrom.rs` (D-33.3) + `modal.rs` (D-33.3b) sanctioned exceptions; `Stat1Step` enum + 5 modal variants live in NEW `stat1/modal.rs` (outside freeze); rejected ~80-line parallel `Stat1ModalProgram` alternative.
- **`ModalProgram::Stat1(Stat1Step)` enum extension ([ADR-v3.1-005](docs/adr/v3.1-005-modalprogram-stat1-enum-extension.md) / D-33.3b):** single `ModalProgram` enum gains the `Stat1` variant; 4-way exhaustive-match invariant items 1+2 preserved; future v3.2+ pacs (Time, Advantage) inherit the pattern (`ModalProgram::Time(TimeStep)` etc.).
- **26 new `Op` variants** in `dispatch()` + `execute_op()` (4-way invariant items 1+2 complete; items 3+4 sanctioned-deferred to Phase 34 / Phase 36 — intentional `non-exhaustive patterns` CI break in `hp41-cli`/`hp41-gui` until Phase 34 closes item 3).
- **Quality gates held:** hp41-core line coverage 95.39 % / region coverage 94.26 % preserved (Phase 33 invariant); 1962 lib + integration tests pass; clippy `-D warnings` clean; Free42 contamination guard exits OK on both `math1/` and `stat1/` trees; D-33.8 STAT-QUAL-09 reassigned Phase 37 → Phase 33 (Plan 33-00) and met.

#### Phase 34 — CLI Integration (shipped 2026-05-23)

- **`docs/hp41-stat1-functions.json` authored (D-34.1):** 26-entry canonical source; 7-category convention (Stat1 Univariate / ANOVA / Regression / Hypothesis / Nonparam / Distributions / RNG); `xrom: { module: "Stat 1", module_id: 2, function_id: N }` block per entry per Phase 28 D-28.3 schema; surgical inline `divergences` field on RAND/SEED + ΣTSTAT + ΣPOLYP only per D-34.3 (full taxonomy lives in `docs/hp41-stat1-divergences.md` per D-35.4 — divergence entries CROSS-REFERENCE the JSON inline fields, do NOT duplicate them).
- **Third `OnceLock<Vec<HelpEntry>>` in `hp41-cli/src/help_data.rs` (D-34.2):** `STAT1_HELP_ENTRIES` static + `help_entries_stat1()` accessor + merged `help_entries_all()` 3-pool chain (cv + math1 + stat1); malformed JSON panics at first access (hard build-blocker per D-25.17 carry-forward; JSON-canonical-data-flow invariant extended to the third source-of-truth).
- **26 new `op_display_name` arms in `hp41-cli/src/prgm_display.rs`** (4-way invariant item 3 complete; no `_ =>` catch-all). `function_matrix_parity.rs` 3-pool partition test cross-checks Op ↔ JSON bidirectional consistency across all three JSON pools.
- **`?` help overlay "Stat 1 Pac (XROM 2)" section (D-34.5):** parallel-loads alongside "Math 1 Pac (XROM 7)"; incremental substring search (v3.0 polish-batch feature) spans all three JSON pools.
- **Modal-prompt routing reuses v3.0 infrastructure (D-34.4):** ΣPOLYP `DEGREE=?`, SEED `SEED?`, ΣCHISQD ν entry — all route through existing `print_buffer` + `modal_program` + `modal_prompt` channels; no new transient CalcState fields beyond `rand_seed` (STAT-RNG-03 / Pitfall 20).
- **`xrom_shadowing.rs` extended to `STAT_1.ops` (D-34.6):** all 26 Stat 1 mnemonics confirmed disjoint from `MATH_1.ops` + `BUILTIN_CARD_OP_NAMES` allowlist (Pitfall 22 verified end-to-end across both XROM modules).
- **Right-panel `key_ref_entries()` filter unchanged:** the v3.0 polish-batch `entry.xrom.is_none()` filter excludes XROM-module functions from the right panel; Stat 1 entries inherit this behavior (discoverable via `?` overlay's "Stat 1 Pac (XROM 2)" section, not the right panel).
- **No core/GUI changes in Phase 34:** SC-4 invariant trivially preserved; v3.0 surface unchanged.

#### Phase 35 — Documentation & ADRs (shipped 2026-05-23)

- **`scripts/docs-matrix` three-input extension (D-35.1 / D-30.1 carry-forward):** justfile invokes the renderer three times (cv + math1 + stat1); binary signature stays 1-in/1-out — only edit is a 4-line basename dispatch branch in `scripts/docs-matrix/src/main.rs:70-76`; hp41cv + math1 matrices bit-for-bit unchanged after regeneration (D-30.2 invariant preserved); `just docs-matrix-check` CI gate covers all three matrices.
- **`docs/hp41-stat1-function-matrix.md` generated (26 entries):** new sibling file; carries XROM column showing `Stat 1 / 2-N` per entry; reachable via README v3.1 soft-claim link.
- **`docs/hp41-stat1-divergences.md` authored (D-35.4):** three-bucket numbered catalog (OM Divergences / Emulator Extensions / Behavioral Policies) with 12 `D-35-NN` entries in the 5-field shape (OM citation / Our behavior / OM behavior / Rationale / See); the 6 oracle drifts queued from 33-VERIFICATION.md land as D-35-01..06 (bucket 3 — Behavioral Policies, since these are mathematical-ground-truth scipy-vs-SPEC corrections); citation provenance per Pitfall 18 enforced.
- **`33-SPEC-AMENDMENT.md` history-preserving supplement (D-35.1):** sibling to `33-SPEC.md` (NOT in-place edit); 6-row table mapping each oracle drift to scipy-correct value + test file:line + drift root cause + D-35-NN cross-ref. Tests are ground truth; SPEC.md preserved as planning-archaeology.
- **5 new ADRs (D-35.2):** [v3.1-001 rng-state-placement](docs/adr/v3.1-001-rng-state-placement.md) + [v3.1-002 distribution-primitives-policy](docs/adr/v3.1-002-distribution-primitives-policy.md) (Free42 disclaim verbatim per Pitfall 19) + [v3.1-003 anova-register-layout](docs/adr/v3.1-003-anova-register-layout.md) + [v3.1-004 math1-freeze-second-carve-out](docs/adr/v3.1-004-math1-freeze-second-carve-out.md) + [v3.1-005 modalprogram-stat1-enum-extension](docs/adr/v3.1-005-modalprogram-stat1-enum-extension.md); each long-form per D-30.6; `## Alternatives Considered` quotes 33-CONTEXT.md verbatim per D-30.7.
- **README v3.1 soft-claim (D-35.3):** `- Stat 1 Pac behavioral emulation (13 programs, 26 XEQ entry points, RAND/SEED extension, documented divergences)` under `## Features`. The OM-cited hard claim (Stat 1 Pac completeness per OM 00041-90030) is deferred to Phase 37 conditional on STAT-QUAL-04 + STAT-QUAL-11 (same gating discipline as v3.0 D-30.9 → D-32.5 graduation pattern).
- **`## Frozen Invariants → Core engine` amendment landed in this plan:** the math1/ freeze sentence now lists `xrom.rs` + `modal.rs` as carve-outs gated by ADR-v3.1-004 (this is the amendment you're reading above).
- **`docs/architecture-history.md` v3.1 narrative:** new `## v3.1 additions (Stat 1 Pac Emulation, Phases 33–35 — 36–37 IN PROGRESS)` section parallel to the v3.0 section; Phase 33-35 populated; Phase 36-37 stubs.

#### Phase 36 — GUI Integration (shipped 2026-05-24)

#### Phase 37 — Test Hardening & Quality Gates (shipped 2026-05-24)

- **Meta-gate infrastructure (STAT-QUAL-06/07/08):** `stat1_op_test_count.rs` (26 variants >= 5 tests), `lint_stat1_assertions.rs` (Pitfall 14/17 discipline), XROM shadowing attested.
- **Coverage gap closure (STAT-QUAL-03):** `stat1_modal_coverage.rs` (modal.rs 74.21% → 93.25%), `stat1_anova_coverage.rs` (anova.rs error paths), `stat1_coverage_supplement.rs` (22 supplementary tests for Pitfall 16 closure).
- **Backward compatibility (STAT-QUAL-10):** `stat1_backward_compat.rs` + `v30-autosave.json` fixture — v3.0 `xrom_modules=1` migrates to `0b11`, `rand_seed` defaults to zero.
- **Numerical accuracy (STAT-QUAL-04/05):** 30 scipy-derived Stat 1 oracle cases in `numerical_accuracy.rs` (761 → 791 total, 98.86% pass rate). `ITER_TOL` (1e-7) + `iter` macro arm for two-level tolerance discipline.
- **E2E smoke (STAT-QUAL-11):** ΣNORMD Q(1.96) workflow in `hp41-gui/e2e/smoke.spec.js`.
- **Documentation:** D-35-13 bounded-iter waiver in `hp41-stat1-divergences.md`, Free42 contamination guard re-verified (18 tokens, exits 0).
- **README hard-claim graduated (D-35.3 / D-37.11):** "feature-complete per Owner's Manual HP 00041-90030" — mirrors v3.0 Math Pac I graduation pattern.
- **Coverage assessment:** Aggregate hp41-core 93.91% lines / 95.84% regions (v3.0 baseline 95.39%/94.26% measured before stat1 ~6,824 LOC added — denominator dilution is the root cause; region coverage exceeds target).

**Frozen invariants preserved across v3.1:**

- SC-4 invariant: every Phase 33–37 change respects the stricter grep — Stat 1 Pac math lives in `hp41-core/src/ops/stat1/`. The `math1/` second carve-out (`xrom.rs` + `modal.rs`) is documented per ADR-v3.1-004; no Stat 1 Pac code leaks INTO the frozen `math1/` module (`Stat1Step` semantics live in `stat1/modal.rs`, outside the freeze boundary).
- 4-exhaustive-match invariant: every new `Op` variant landed in `dispatch()` + `execute_op()` (Phase 33) + CLI `prgm_display.rs` (Phase 34) + GUI `prgm_display.rs` (Phase 36) — all 4 sites complete.
- `#![deny(clippy::unwrap_used)]` continues to apply in `hp41-core`; new test files in v3.1 carry `#[allow]` at file scope per the established pattern.
- Save-file backward compat: every new `CalcState` field in Phase 33 carries `#[serde(default)]`; transient fields use `skip`. The `rand_seed` field is the documented exception (`default` WITHOUT `skip` per STAT-RNG-03 / Pitfall 20). Phase 37 backward-compat test confirms v3.0→v3.1 migration.
- MSRV 1.88 unchanged through Phase 33–37. Zero new runtime deps (statrs rejected per ADR-v3.1-002).
- Free42 GPL contamination guard: extended from 12 → 18 tokens per Phase 33 Plan 33-00 D-32.7 reassignment (STAT-QUAL-09 met); both `math1/` and `stat1/` trees scanned at every CI run; script exits OK. Phase 37 re-verification passed.

**v3.1 file landmarks (forward-pointers; full file table updates land at v3.1 milestone ship per the v3.0 cadence):**

- `hp41-core/src/ops/stat1/` — Stat 1 Pac (XROM 2) implementation tree; sibling to `math1/`; carries the verbatim Free42 disclaim header on every file.
- `hp41-core/src/ops/stat1/distributions.rs` — 3 hand-coded f64-bridge distribution primitives (~140 LOC) per ADR-v3.1-002.
- `hp41-core/src/ops/stat1/mod.rs` — OM 00041-90030 "Storage Registers" transcription header + ~30 named consts per ADR-v3.1-003.
- `hp41-core/src/ops/stat1/modal.rs` — `Stat1Step` enum + dispatch (lives OUTSIDE the math1/ freeze; per ADR-v3.1-004 the second carve-out keeps `math1/modal.rs` to ~8 lines of pure dispatch).
- `hp41-core/src/ops/stat1/random.rs` — RAND/SEED LCG (NPS p. 21-22 + Don Malm community provenance per ADR-v3.1-001); `rand_seed` field at `state.rs:201-202`.
- `docs/hp41-stat1-functions.json` — third JSON source-of-truth (26 entries; D-34.1 7-category convention).
- `docs/hp41-stat1-function-matrix.md` — generated via `just docs-matrix` (third invocation; D-35.1).
- `docs/hp41-stat1-divergences.md` — 12 D-35-NN entries across 3 buckets (0 OM divergences + 2 emulator extensions + 10 behavioral policies; D-35.4 numbering scheme).
- `docs/adr/v3.1-{001..005}-*.md` — 5 long-form ADRs (D-30.6 template; D-35.2 scope).
- `.planning/phases/33-…/33-SPEC-AMENDMENT.md` — history-preserving SPEC supplement; 6 oracle drifts reconciled (D-35.1).

### v3.2 additions (Time Pac Emulation, Phases 38–42)

*Origin: see `docs/architecture-history.md` §v3.2 additions for the long-form per-phase narrative; this block is the CLAUDE.md decision-summary surface (the SECOND `### v3.x additions` block; the first was v3.1 per D-35.5).*

#### Phase 38 — XROM Framework + Clock/Date/Stopwatch/Alarm Core (shipped 2026-05-24)

- **TIME_MODULE (XROM ID 26) registered (D-carried.5):** `TIME_MODULE.id = 26`, name `"TIME 2C"` (HP 82182A CATALOG 2 display string), 35 ops; `xrom_resolve` bit-2 arm fires LAST after bit-1 (STAT_1) and bit-0 (MATH_1) per Pitfall 1 + Pitfall 22.
- **`default_xrom_modules() = 0b0000_0111` + `migrate_after_load()` (D-carried.5):** v3.1 save files with `xrom_modules: 0b0000_0011` auto-upgrade to `0b0000_0111` at startup; canonical migration site is `state.rs`; both CLI and GUI persistence paths call it (D-33.7 single-source-of-truth inherited).
- **Direct `std::time::SystemTime::now()` in hp41-core (D-38.1):** core calls system clock directly — `SystemTime` is a value-returning syscall, not console I/O; the zero-I/O principle applies to console/filesystem/network, not clock reads. Trait injection, frontend callback, and `no_std` abstraction were evaluated and rejected (see ADR-v3.2-001).
- **`time_offset_secs: i64` on CalcState (D-38.2, `#[serde(default)]`):** SETIME computes delta between entered time and `SystemTime::now()`, stored as seconds; all time/date reads apply `SystemTime::now() + offset` to derive the "HP-41 time". Also encodes the timezone + user adjustment combined — no separate OS timezone query needed.
- **Pure-Rust Gregorian calendar arithmetic (D-carried.1 + D-38.3):** Fliegel-Van Flandern JDN formula (~60 LOC) in `clock.rs` replaces the planned `libc::localtime_r` call — `libc` is not a runtime dep and D-carried.1 prohibits new runtime deps. Zero new runtime deps achieved.
- **Date decimal parsing via string-split-at-decimal (D-carried.2):** Left-pad fractional part to exactly 6 chars, split into DD[2]+YYYY[4] (MDY) or MM[2]+YYYY[4] (DMY); ISG/DSE precedent carried forward — `floor()`/`fmod()` forbidden.
- **Stopwatch fields with split serde shapes (D-38.6, D-38.7):** `stopwatch_mode: StopwatchMode` (`#[serde(default)]`), `stopwatch_accumulated: f64` (`#[serde(default)]`), `stopwatch_split: f64` (`#[serde(default)]`), `stopwatch_start: Option<Instant>` (`#[serde(default, skip)]` — transient). Running stopwatch is frozen to Stopped on save (D-38.6); user must RUNSW to resume; `migrate_after_load()` enforces the freeze.
- **`AlarmType` enum + `Vec<AlarmEntry>` on CalcState (D-38.8–11):** `AlarmType::Message(String)` | `AlarmType::Control { label, interrupting }` — forward-compatible data model even though interrupting control alarm execution is deferred (D-38.4). Direct `alarms: Vec<AlarmEntry>` with `#[serde(default)]`; free functions in `alarm.rs` (same pattern as `regs: Vec<HpNum>`). `check_alarms()` drain pattern; alarm notifications pushed into `event_buffer`; repeat interval stored as `i64` seconds (D-38.11). 253-entry catalog cap enforced at XYZALM entry.
- **Interrupting control alarm DEFERRED (D-38.4):** documented divergence in `docs/hp41-time-divergences.md`; data model stores them but execution requires re-entrancy against the 4-level call stack not currently supported. Non-interrupting control alarms XEQ the stored label on acknowledgment (D-38.5).
- **`ModalProgram::Time(TimeStep)` variant (D-carried.4):** follows ADR-v3.1-005 pattern; `TimeStep` enum in new `time/modal.rs` (outside math1/ freeze boundary).
- **35 new `Op` variants** in `dispatch()` + `execute_op()` (4-way invariant items 1+2 complete; items 3+4 deferred to Phase 39 CLI / Phase 41 GUI with sanctioned CI break).
- **Free42 contamination guard extended to `time/` directory:** `scripts/check-free42-contamination.sh` covers `math1/`, `stat1/`, and `time/` trees; token count unchanged (18 tokens, exits 0).
- **Phase 38 CalcState additions with serde shapes (D-40.10):** `time_offset_secs: i64` (`#[serde(default)]`), `clock_12h: bool` (`#[serde(default)]`), `clock_display_mode: ClockDisplayMode` (`#[serde(default)]`), `accuracy_factor: HpNum` (`#[serde(default)]`), `alarms: Vec<AlarmEntry>` (`#[serde(default)]`), `stopwatch_mode: StopwatchMode` (`#[serde(default)]`), `stopwatch_accumulated: f64` (`#[serde(default)]`), `stopwatch_split: f64` (`#[serde(default)]`), `stopwatch_start: Option<Instant>` (`#[serde(default, skip)]` — transient), `clock_active: bool` (`#[serde(default, skip)]` — transient), `stopwatch_keyboard_mode: bool` (`#[serde(default, skip)]` — transient), `alarm_catalog_mode: bool` (`#[serde(default, skip)]` — transient). XROM bit-2 arm = `TIME_MODULE` (XROM 26).

#### Phase 39 — CLI Integration + Live Display (shipped 2026-05-25)

- **`docs/hp41-time-functions.json` authored (D-39.9):** 35-entry canonical source; 7-category convention (Time Clock / Time Date Arithmetic / Time Display / Time Format / Time Alpha / Time Stopwatch / Time Alarm); `xrom: { module: "Time", module_id: 26, function_id: N }` block per entry per Phase 28 D-28.3 schema; inline `divergences` field on exactly 4 entries (CORRECT, SW, RCLAF, SETAF per D-39.11).
- **Fourth `OnceLock<Vec<HelpEntry>>` in `hp41-cli/src/help_data.rs` (D-39.12):** `TIME_FUNCTIONS_JSON` + `TIME_HELP_ENTRIES` static + `help_entries_time()` accessor + 4-pool chain `help_entries_all()` (cv → math1 → stat1 → time); malformed JSON panics at first access (hard-build-blocker pattern per D-25.17 carried forward to fourth file).
- **35 new `op_display_name` arms in `hp41-cli/src/prgm_display.rs`** (4-way invariant item 3 complete; no `_ =>` catch-all); `function_matrix_parity.rs` 4-pool partition test + `TIME_OP_VARIANT_NAMES` inventory constant; xrom_shadowing.rs extended to TIME_MODULE.ops.
- **`?` help overlay "Time Pac (XROM 26)" section (D-39.13):** auto-derived from JSON categories; incremental substring search spans all four JSON pools.
- **Live clock/stopwatch display — pull-on-redraw (D-39.1, D-39.2):** `get_clock_display_str()` / `get_stopwatch_display_str()` called at top of `get_display_string()` priority chain (before entry_buf); existing 16ms poll loop redraws ~62 Hz — TIME-DSP-05 (≥1 Hz clock) and TIME-SW-08 (≥10 Hz stopwatch) trivially met. Clock exit clears `clock_active` on any keypress (D-39.3 — mutation-only, key falls through).
- **Stopwatch keyboard mode (D-39.4, D-39.5):** top-level routing block in `handle_key()` after clock-exit; `handle_stopwatch_mode_key()` consumes keys: Space/Enter → toggle RUNSW/STOPSW, `s` → SWPT (split), `r` → STPW (reset), Esc → exit mode. No new `PendingInput` variant needed.
- **Alarm event draining (D-39.6–8):** `drain_event_buffer()` routes `"alarm:message:{text}"` → `self.message`, `"alarm:xeq:{label}"` → XEQ dispatch; `check_alarms()` called every 16ms tick in `run()` loop OUTSIDE the event-poll conditional (alarms fire in real-time without user interaction); `drain_event_buffer()` also appended to `call_dispatch()` and `call_dispatch_and_drain()`.
- **XROM shadowing extended to `TIME_MODULE.ops` (D-39.14):** all 35 Time Pac mnemonics confirmed disjoint from `MATH_1.ops` + `STAT_1.ops` + `BUILTIN_CARD_OP_NAMES` allowlist; Pitfall 22 verified end-to-end across all three XROM modules.
- **Right-panel `key_ref_entries()` filter unchanged:** `entry.xrom.is_none()` excludes Time entries from right panel; discoverable via `?` overlay "Time Pac (XROM 26)" section only (same UX as Math 1 + Stat 1).

#### Phase 40 — Documentation & ADRs (shipped 2026-05-25)

- **`docs/hp41-time-divergences.md` authored (D-40.3):** three-bucket numbered catalog (OM Divergences / Emulator Extensions / Behavioral Policies) with `D-40-NN` identifiers; 5-field entry shape per D-30.5 (OM citation / Our behavior / OM behavior / Rationale / See); known entries include host clock backing, stopwatch freeze-on-save, interrupting control alarm deferral, CORRECT/SETAF accuracy-factor no-op, centisecond resolution.
- **3 ADRs: v3.2-001 (clock access), v3.2-002 (live display), v3.2-003 (alarm catalog) (D-40.5):** long-form per D-30.6 template; each `## Alternatives Considered` quotes Phase 38 CONTEXT decisions verbatim (D-30.7).
- **`scripts/docs-matrix` fourth invocation (D-40.1–2):** `hp41-time-functions.json` → `hp41-time-function-matrix.md`; 4th `else if` branch in renderer + 4th invocation in `justfile` `docs-matrix` and `docs-matrix-check` recipes.
- **README v3.2 soft-claim (D-40.7):** bullet under `## Features`; no "feature-complete per Owner's Manual" — hard-claim deferred to Phase 42.
- **CLAUDE.md `### v3.2 additions` block authored (D-40.9, D-40.10):** this section you are reading.

#### Phase 41 — GUI Integration (shipped 2026-05-25)

- **35 `op_display_name` arms in `hp41-gui/src-tauri/src/prgm_display.rs`** (4-way invariant item 4 complete; no `_ =>` catch-all). `function_matrix_parity.rs` 4-pool partition test cross-checks bidirectional consistency across all four JSON pools.
- **`tick_time` Tauri command + CalcStateView live-display fields (D-41.1):** `clock_active`, `stopwatch_keyboard_mode`, `stopwatch_running`, `event_buffer` projected to frontend; `tick_time` calls `check_alarms()` + returns fresh `CalcStateView`.
- **`op_catalog` refactored to generic 3-module loop (D-41.2):** `MATH_1`, `STAT_1`, `TIME_MODULE` iterated; Time Module entry in CATALOG 2.
- **HelpOverlay.tsx fourth section "Time Pac (XROM 26)" (D-41.3):** `helpEntriesTime()` + 4-pool `helpEntriesAll()` chain; incremental substring search spans all four JSON pools.
- **App.tsx live display (D-41.4):** `setInterval` with 200ms tick calls `tick_time`; clock/stopwatch display strings rendered in LCD area; alarm events drained from `event_buffer` per `useEffect`.
- **14-segment LCD colon rendering (D-41.5):** colons in clock/stopwatch display rendered as two-dot overlay; `Display14Seg` component extended.
- **Stopwatch keyboard mode + clock exit mirrored from CLI (D-41.6):** Space/Enter toggle RUNSW/STOPSW, `s` → split, `r` → reset, Esc → exit; clock display exits on any keypress.

#### Phase 42 — Test Hardening & Quality Gates (shipped 2026-05-25)

- **Meta-gate unification (TIME-QUAL-01/02):** `xrom_op_test_count.rs` unified across all 3 XROM modules (106 variants >= 5 tests); `lint_xrom_assertions.rs` unified assertion-discipline lint across all 3 modules.
- **Coverage gap closure (TIME-QUAL-03):** `time_coverage_supplement.rs` (79 tests targeting 30 Time variants below 5-test threshold); all 7 `time/*.rs` source files exceed 90% region coverage.
- **Numerical accuracy (TIME-QUAL-04):** 30 oracle-verified date arithmetic cases in `time_date_accuracy.rs` (791 total across all modules, 98.86% pass rate); stopwatch timing accuracy tests; alarm latency tests.
- **Backward compatibility (TIME-QUAL-05):** `time_backward_compat.rs` + `v31-autosave.json` fixture — v3.1 `xrom_modules=0b011` migrates to `0b111`, `time_offset_secs` defaults to zero.
- **E2E smoke (TIME-QUAL-06):** DDAYS workflow in `hp41-gui/e2e/smoke.spec.js`.
- **README hard-claim graduated (D-40.7 / D-42.11):** "feature-complete per Owner's Manual 00041-90035" — mirrors v3.0/v3.1 graduation pattern.
- **Coverage assessment:** Aggregate hp41-core 93.72% lines / 95.63% regions (v3.1 baseline 93.91%/95.84%; region coverage exceeds target).

## Tech Stack

- **`just`** — sole task runner. **Never call `cargo` directly in CI or docs.** GUI recipes: `just gui-dev` / `just gui-build` / `just gui-ci` / `just gui-check`.
- Rust stable, MSRV 1.88
- `rust_decimal` 1.42, ratatui 0.30, crossterm 0.29, serde + serde_json, proptest, criterion (advisory, not CI-gated), clap 4.x
- Tauri v2.11, React 18 + TypeScript + Vite, `dirs`
- cargo-llvm-cov (coverage gate ≥ 95 % lines / ≥ 93 % regions on `hp41-core`, programmatically gated)
- WebdriverIO 9 + tauri-driver 2.0.6 (E2E smoke; Ubuntu-only via `ci-gui.yml::e2e-linux`)
- Two-layer CI: `ci.yml` (CLI + license-audit) + `ci-gui.yml` (3-OS matrix, path-filtered to `hp41-gui/**` and `hp41-core/**`)

## Quality Gates (current targets)

| Gate | Target | Current (v3.0) |
|------|--------|----------------|
| `hp41-core` coverage | ≥ 95 % lines / ≥ 93 % regions | 95.39 % / 94.26 % |
| Numerical accuracy | ≥ 98 % | 99.3 % (763 / 768) |
| Panics in `hp41-core` | 0 | 0 |
| Cold-start | ≤ 0.5 s | 2.2 ms (M1) |
| Key latency | ≤ 50 ms median | ~65 ns/op |
| Free42 contamination | 0 distinctive symbols | 0 (CI-gated) |
| MSRV | 1.88 declared | 1.88 (CI-enforced) |
| CI | Win 10+ / macOS 12+ / Ubuntu 22.04+ | ✅ both workflows |

Per-milestone history: see `docs/architecture-history.md`.

## Key Files

**Core engine (`hp41-core`):**

| File | Purpose |
|------|---------|
| `src/ops/mod.rs` | `Op` enum, `dispatch()`, `flush_entry_buf()`, `synthetic_byte_to_op()` — central hub |
| `src/state.rs` | `CalcState` — single source of truth |
| `src/stack.rs` | `Stack`, `apply_lift_effect()` |
| `src/ops/program.rs` | `run_program()`, `parse_counter()`, `execute_op()`, `builtin_card_op` (4→12 v2.2 extension) |
| `src/ops/print.rs` | PRX / PRA / PRSTK — buffer-only |
| `src/ops/registers.rs` | STO arithmetic, M/N/O hidden registers |
| `src/ops/math1/` | Math Pac I (XROM 7), frozen since Plan 25-01 |
| `src/format.rs` | `format_hpnum()`, `format_alpha()` |
| `tests/numerical_accuracy.rs` | 768-case suite, ≥ 98 % gate |

**TUI (`hp41-cli`):**

| File | Purpose |
|------|---------|
| `src/app.rs` | `App`, `handle_key()`, `PendingInput`, event loop, `shift_armed`, help-overlay search |
| `src/keys.rs` | `key_to_op()`, `keycode_to_hp41_code()`, `xeq_by_name_local_resolve`, `key_ref_entries()` |
| `src/ui.rs` | `format_entry_buf_display()`, `pending_prompt()` exhaustive match, `render_right_panel`, ratatui `Clear` overlay pattern |
| `src/help_data.rs` | `OnceLock<Vec<HelpEntry>>` loaded from JSON sources of truth + `filter_help_rows` |
| `src/prgm_display.rs` | `op_display_name()` — exhaustive match, NO `_ =>` catch-all |
| `src/persistence.rs` | `save_state()`, `load_state()` — JSON serde |

**GUI (`hp41-gui`):**

| File | Purpose |
|------|---------|
| `src-tauri/src/lib.rs` | `setup()`, `AppState = Mutex<CalcState>`, 30s auto-save thread, `generate_handler!` registration |
| `src-tauri/src/commands.rs` | `dispatch_op`, `get_state`, `sst_step`, `bst_step`, `run_stop`, `request_cancel` thunks |
| `src-tauri/src/types.rs` | `CalcStateView`, `Annunciators`, `GuiError`, `From<HpError>` |
| `src-tauri/src/key_map.rs` | string ID → `Op` resolver (SC-4 boundary) |
| `src-tauri/src/persistence.rs` | Shared `~/.hp41/autosave.json` |
| `src-tauri/src/prgm_display.rs` | `op_display_name()` — second exhaustive match (4-way invariant) |
| `src-tauri/permissions/*.toml` | Tauri v2.11 inline-command permission registry |
| `src/App.tsx` | React root, `shiftActive`, `invokeForKey` / `extractErrMessage` helpers, toast overlay |
| `src/Keyboard.tsx` | 5×8 grid + top-row band, three-label `KeyDef` |
| `src/HelpOverlay.tsx` | `?` overlay with search + "Math 1 Pac (XROM 7)" section |
| `src/App.css` | Layout + key animation (`transform-box: fill-box`) |
| `wdio.conf.cjs` + `e2e/smoke.spec.js` | WebdriverIO + tauri-driver E2E smoke (Ubuntu) |
