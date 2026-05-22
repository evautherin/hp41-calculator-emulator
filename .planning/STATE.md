---
gsd_state_version: 1.0
milestone: v3.1
milestone_name: Stat 1 Pac Emulation
status: executing
last_updated: "2026-05-22T09:32:02.976Z"
last_activity: 2026-05-22 -- Phase 33 execution started
progress:
  total_phases: 5
  completed_phases: 0
  total_plans: 9
  completed_plans: 0
  percent: 0
---

# Project State: HP-41 Calculator Emulator

---

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-21 after v3.0 milestone archive)

**Core value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.

**Shipped milestones:**

- v1.0 CLI (2026-05-08) — Phases 1–8; foundational RPN engine + TUI
- v1.1 CLI Feature Completeness (2026-05-09) — Phases 9–12
- v2.0 Tauri GUI (2026-05-10) — Phases 13–18
- v2.1 Card Reader + Keyboard Authenticity (2026-05-13) — recorded as quick tasks (no Phase 19 GSD directory)
- v2.2 HP-41CV Feature Completeness (2026-05-15) — Phases 20–27, 26/26 plans
- **v3.0 Math Pac I Emulation (2026-05-20) — Phases 28–32, 31/31 plans; 95.39 % lines / 94.26 % regions on `hp41-core`; 99.3 % numerical accuracy (763/768); CI green across `ci.yml` + `ci-gui.yml`**

**Current focus:** Phase 33 — hp41-core-xrom-activation-distribution-primitives-all-stat-1
**Repo:** hp41-calculator-emulator
**Architecture:** Cargo workspace — `hp41-core` (library) + `hp41-cli` (binary) + `hp41-gui` (nested standalone Tauri workspace); `hp41-core` has zero UI/CLI dependencies enforced at compile time.

---

## Current Position

Phase: 33 (hp41-core-xrom-activation-distribution-primitives-all-stat-1) — EXECUTING
Plan: 1 of 9
Status: Executing Phase 33
Last activity: 2026-05-22 -- Phase 33 execution started

### v3.1 Phase Overview

| Phase | Focus | Requirements | Est. Plans |
|-------|-------|--------------|------------|
| 33 | `hp41-core` — XROM activation + distribution primitives + all ~24 Op variants | 39 | 9–11 |
| 34 | `hp41-cli` — JSON help pool + `op_display_name` arms + modal routing | 5 | 2–3 |
| 35 | `docs/` + tooling — divergences + ADRs + docs-matrix 3-input + README/CLAUDE.md | 6 | 3–4 |
| 36 | `hp41-gui` — `prgm_display` arms + CATALOG 2 + help overlay + cancel reuse | 5 | 3–5 |
| 37 | tests + scripts + CI — coverage hold + accuracy cases + backward-compat + E2E smoke | 11 | 6–10 |

**Research flag for Phase 33:** `/gsd-plan-phase 33 --research-phase` is required — OM 00041-90030 "Storage Registers" section not yet fully read; RAND subroutine presence unconfirmed; quantile convergence criteria not yet extracted; ΣPOLYP degree prompt and ΣCHISQD ν convention unconfirmed; ΣTSTAT pooled vs. Welch assumption unresolved.

## Performance Metrics (carried from v3.0 ship)

| Metric | Target | Last measured (v3.0) |
|--------|--------|----------------------|
| Cold-start latency | ≤ 0.5 s | 2.2 ms (M1) — 228× under gate |
| Key-press latency (median) | ≤ 50 ms | ~65 ns/op |
| `hp41-core` line coverage | ≥ 95 % | **95.39 %** |
| `hp41-core` region coverage | ≥ 93 % | **94.26 %** |
| Per-file `ops/math1/*.rs` floor | ≥ 90 % | all ≥ 90 % (lowest: poly 90.45 %) |
| Numerical accuracy | ≥ 98 % (768 cases) | **99.3 % (763/768)**; v1.x 503-case floor 498/503 preserved |
| Panics in `hp41-core` | 0 | 0 — enforced by `#![deny(clippy::unwrap_used)]` |
| Free42 contamination | 0 distinctive symbols | 0 (CI-gated, 12-symbol grep) |
| CI platforms | Win/macOS/Ubuntu | All green (`ci.yml` + `ci-gui.yml` + `e2e-linux` + `license-audit`) |

---

## Accumulated Context

### Key Decisions (carried forward from v1.x–v3.0)

| Decision | Rationale | Phase |
|----------|-----------|-------|
| BCD vs f64 | `rust_decimal` with 10-digit rounding; avoid custom BCD struct | Phase 1 |
| Stack-lift as `lift_enabled: bool` | Every op declares Enable/Disable/Neutral | Phase 1 |
| `CalcState` as single source of truth | One `&mut CalcState` through all ops; no global state | Phase 1 |
| ISG/DSE string-split counter fields | Never `floor()`/`fmod()` on f64 | Phase 3 |
| `ratatui::init()` not `Terminal::new()` | Installs panic hook for terminal restore | Phase 4 |
| Digit entry via `entry_buf` | Auto-flushed on next non-digit | Phase 4 |
| `serde_json` for persistence | Human-readable, diff-able, versioned JSON | Phase 5 |
| No async in `hp41-core` | Single-threaded event loop | All |
| `#![deny(clippy::unwrap_used)]` | Compile-time zero-panic guarantee | Phase 7 |
| `print_buffer: Vec<String>` on CalcState | Keeps hp41-core I/O-free; hp41-cli drains buffer | Phase 11 |
| Bundle identifier `ch.talent-factory.hp41` (D-02) | Overrides scaffold default `com.tauri.dev` | Phase 13 |
| Tauri v2.11 app-command permissions: TOML files required | Inline app commands don't auto-generate `allow-<cmd>` permissions | Phase 14 |
| One-shot SHIFT frontend-only (`shiftActive`) | Never crosses IPC; ALPHA overrides SHIFT | v2.1 |
| f-prefix one-shot on CLI mirrors GUI (`shift_armed`) | D-25.6 CLI ↔ GUI parity invariant | Phase 25 |
| Hybrid `PendingInput` struct-variants | Collapses 34 logical ops into 2 carriers (FlagPrompt, RegisterPrompt) | Phase 25 |
| `docs/hp41cv-functions.json` as single source of truth | JSON-canonical pipeline; `scripts/docs-matrix` regenerates matrix; bidirectional parity tests | Phase 25 |
| `data-testid="lcd-display"` on `Display14Seg.tsx` | Allowed under SC-4 (hp41-gui/src/ outside boundary); enables WebdriverIO assertion | Phase 27 |
| Coverage gate atomic raise 80 % → 95 % (D-27.2) | Avoid gate-and-test split that masks regressions | Phase 27 |
| WebdriverIO + tauri-driver (not Playwright) for E2E | tauri-driver speaks WebDriver classic; Playwright is CDP/native only | Phase 27 |
| Op-strategy A (one Op variant per Math Pac I function) | Preserves 4-exhaustive-match invariant; rejects `Op::XromCall(u16)` table dispatch | Phase 28 ADR-001 |
| User-callback re-entrancy: strict-reject nested | Matches Math Pac I Hardware-Verhalten per OM; simplest invariant | Phase 28 ADR-002 |
| JSON-pipeline: separate `hp41-math1-functions.json` | Zero migration churn on 130 existing v2.2 entries; cleaner test surfaces | Phase 28 ADR-005 |
| `xrom_resolve` fires LAST in resolver chain | Prevents Math Pac I shadowing existing built-in mnemonics (Pitfall 1) | Phase 28 |
| `run_loop` (NOT `run_program`) re-entry for INTG/SOLVE/DIFEQ | Preserves outer program clone; avoids 30 KB × 1000 samples re-clone catastrophe | Phase 28 |
| `request_cancel` cancellation channel with per-64-samples lock release | Pitfall 11 mitigation; preserves 30s auto-save thread interleaving | Phase 31 |
| Free42 contamination guard: 12 distinctive symbols (D-32.7) | Cross-checks decNumber / Intel BID / GPL / AGPL alongside Free42 | Phase 32 |

### v3.1-Specific Decisions (to be locked in Phase 33)

| Decision | Status | Notes |
|----------|--------|-------|
| `STAT_1.id = 2` (hardware XROM ID 2) | Locked — ARCHITECTURE.md confirmed | Math Pac I id=7 discrepancy is frozen; STAT_1 uses correct hardware id=2 |
| `default_xrom_modules()` → `0b0000_0011` + startup migration | Locked per STAT-FW-02 | v3.0 save files with `xrom_modules: 1` migrate via startup persistence layer |
| `rand_seed: HpNum` with `#[serde(default)]` NOT `#[serde(skip)]` | Locked per STAT-RNG-03 | ONLY v3.1 CalcState field with non-skip serde; all other v3.1 fields use skip |
| `statrs` crate rejected | Locked — SUMMARY.md research conclusion | AS 239/63/241 hand-coded ~140 LOC in `stat1/distributions.rs`; zero new runtime deps |
| Self-contained iteration pattern for quantile inversion (not SOLVE/INTG user-callback) | Locked — SUMMARY.md architecture | Distribution quantile loops check `cancel_requested` every iteration; no new CalcState scratch fields |
| OM register layout for ANOVA/moments/regression | PENDING — Phase 33 spec-phase MUST read OM 00041-90030 "Storage Registers" section | Critical path blocker for ΣMMTUG, ΣAOVONE/ΣAOVTWO/ΣANOCOV, ΣMLRXY, ΣCTKKK |
| ΣPOLYP degree prompt wording | PENDING — Phase 33 spec-phase | Tentative "DEGREE=?" per Math Pac I precedent |
| ΣCHISQD ν entry convention | PENDING — Phase 33 spec-phase | Tentative: ν entered via [A] before x evaluation |
| ΣTSTAT pooled vs. Welch variance assumption | PENDING — Phase 33 spec-phase | NPS ZS-4/5 assumes pooled; verify from OM |
| RAND subroutine presence in Stat 1 Pac ROM | PENDING — Phase 33 spec-phase | QRC does not list RAND; implement only if OM confirms |

### Critical Implementation Traps (carried forward)

- **Every new Op variant must be added to 4 places:** `dispatch()` in `ops/mod.rs` + `execute_op()` in `ops/program.rs` + `hp41-cli/src/prgm_display.rs` + `hp41-gui/src-tauri/src/prgm_display.rs`. Exhaustive matches fail to compile if any is missed. Math Pac I added ~40 variants — `math1_op_test_count.rs` + `xrom_shadowing.rs` cross-check 45 `Op` variants × 14 test files + 52 `MATH_1.ops` × 18-entry allowlist at CI time.
- **New CalcState fields need `#[serde(default)]`** for backward compatibility with v1.0–v3.0 save files. Transient fields additionally carry `#[serde(skip)]`. Exception: `rand_seed` uses `#[serde(default)]` WITHOUT `#[serde(skip)]` (STAT-RNG-03 — muscle memory trap).
- **P21 (OM register layout) is the single most dangerous silent-wrong-answer trap** — ΣMMTUG, ΣAOVONE/ΣAOVTWO/ΣANOCOV, ΣMLRXY, ΣCTKKK all require register indices from OM "Storage Registers" section. No guessing. Phase 33 opens with OM read.
- **P22 (mnemonic shadowing)** — every `STAT_1.ops` mnemonic must be verified against `docs/hp41cv-functions.json` before registering in `xrom.rs`. `xrom_shadowing.rs` extended to cover STAT_1.ops.
- **P27 (Free42 stats-domain contamination guard)** — `scripts/check-free42-contamination.sh` must be extended with stats-domain identifiers BEFORE the first `stat1/*.rs` file is written. See STAT-QUAL-09.
- **SC-4 invariant (no core duplication in hp41-gui):** stricter grep `grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` — `op_display_name` is the only intentional exception.
- **`hp41-core/src/ops/math1/` is frozen** since v3.0. Adding stat1 to `ops/` is a new sibling directory, NOT a modification of math1/.
- **No `println!`/`eprintln!` in hp41-core:** route side effects via `print_buffer` (existing channel; used for Math Pac I prompts) or `event_buffer`.
- **Pitfall 14 / 17 discipline:** `lint_stat1_assertions.rs` (or extension of `lint_math1_assertions.rs`) blocks `assert_eq!(decimal, decimal)` on iterated results; two-level tolerance: 1e-9 closed-form (ΣNORMD CDF/PDF, ΣSPEAR, ΣBSTAT/BSTG, ΣLIN/EXP/LOGI/POW, ΣXSQEV/EFXSQ), 1e-7 iterative (probit Φ⁻¹, incomplete gamma/beta, t-CDF, normal-equation solve).
- **HP-copyrighted ROM-image redistribution is permanently excluded** — v3.x is BEHAVIORAL emulation only.

### Blockers

None — roadmap defined; Phase 33 spec-phase OM reading is a planned dependency (not a blocker yet).

### Quick Tasks Completed (historical record)

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260508-y30 | CHS during EEX entry: toggle minus sign in exponent | 2026-05-08 | aa0904b | [260508-y30-eex-chs-exponent-sign-toggle](./quick/260508-y30-eex-chs-exponent-sign-toggle/) |
| 260508-06h | FIX/SCI/ENG digit-count modal via F key (0–9) | 2026-05-08 | 7ff792c | [260508-06h-fix-sci-eng-digit-input](./quick/260508-06h-fix-sci-eng-digit-input/) |
| 260513-v21a | v2.1 Card Reader: WDTA/RDTA/WPRGM/RDPRGM + XEQ-by-name + cards module + PR #9 fixes | 2026-05-13 | 72530dc…f4b3f8b | — (no GSD dir; see MILESTONES.md v2.1) |
| 260513-v21b | v2.1 Keyboard Authenticity: 5-col grid, three-label keys, one-shot SHIFT, run_stop Tauri cmd, stub-error pattern, toast overlay, PR #10 fixes | 2026-05-13 | 8cd2de4…ff56b97 | — (no GSD dir; see MILESTONES.md v2.1) |
| 260520-v30p | v3.0 post-graduation polish batch: MSRV-CI `uninlined_format_args` fix, E2E SINH+MATRIX assertion path, right-panel XROM-exclusion filter, `?` overlay Clear-widget z-order, `?` overlay incremental substring search | 2026-05-20 | (within `v2.2..v3.0` range) | — (recorded under v3.0 narrative in PROJECT.md) |
| 260522-g7s | GUI: add thin gold trim frame around keyboard SVG to match user reference `gui-vorgabe.png` (cosmetic; no backend/IPC change) | 2026-05-22 | 509344a | [260522-g7s-add-yellow-keyboard-frame-matching-vorgabe](./quick/260522-g7s-add-yellow-keyboard-frame-matching-vorgabe/) |
| 260522-gud | GUI: honor `shiftActive` on physical-keyboard input (Tab + 0 → π); closes CLI↔GUI behavioral divergence on the SHIFT one-shot prefix | 2026-05-22 | TBD | [260522-gud-honor-shift-on-physical-keyboard](./quick/260522-gud-honor-shift-on-physical-keyboard/) |

---

## Session Continuity

**Last active:** 2026-05-22
**Last action:** Phase 33 context gathered via `/gsd-discuss-phase 33` — 9 implementation decisions captured in `.planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-CONTEXT.md` (commit 1f46b9d). Decisions: (D-33.1) `/gsd-spec-phase 33` first to lock 5 pending OM conventions + quantile convergence + Free42 stats identifiers as LOCKED requirements; (D-33.2) 9-plan slicing per SUMMARY.md build-step order; (D-33.3) math1/xrom.rs freeze exception for STAT_1 bit-1 arm; (D-33.4) RAND/SEED as documented emulator extension; (D-33.5) 11-file `ops/stat1/` layout with `tests.rs`→`hypothesis.rs` rename; (D-33.6) inline scipy.stats oracle constants; (D-33.7) `CalcState::migrate_after_load()` in `hp41-core/src/state.rs`; (D-33.8) STAT-QUAL-09 reassigned Phase 37 → Phase 33 (Plan 33-00).
**Next action:** `/gsd-spec-phase 33` — lock the 7 LOCKED items per D-33.1 (Σ-register layout for ΣMMTUG/ΣAOVONE/ΣAOVTWO/ΣANOCOV/ΣMLRXY/ΣCTKKK; ΣPOLYP degree prompt; ΣCHISQD ν convention; ΣTSTAT pooled vs Welch; RAND ROM-presence; quantile convergence criteria; Free42 stats-domain identifiers from `core_math2.cc`). Then `/gsd-plan-phase 33` to produce the 9 plans.

---
*State initialized: 2026-05-06*
*Last updated: 2026-05-22 — Phase 33 context gathered (9 decisions captured); next step `/gsd-spec-phase 33`*
