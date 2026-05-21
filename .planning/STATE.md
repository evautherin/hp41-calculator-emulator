---
gsd_state_version: 1.0
milestone: null
milestone_name: awaiting next milestone
status: idle
last_updated: "2026-05-21T00:00:00Z"
last_activity: 2026-05-21 -- v3.0 milestone archived via /gsd-complete-milestone
progress:
  total_phases: 0
  completed_phases: 0
  total_plans: 0
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

**Current focus:** planning next milestone — Stat 1 Pac per scope lock 2026-05-13 (`/gsd-new-milestone` to start)
**Repo:** hp41-calculator-emulator
**Architecture:** Cargo workspace — `hp41-core` (library) + `hp41-cli` (binary) + `hp41-gui` (nested standalone Tauri workspace); `hp41-core` has zero UI/CLI dependencies enforced at compile time.

---

## Current Position

Milestone: none in progress
Phase: none
Status: awaiting v3.1 definition
Last activity: 2026-05-21 -- v3.0 milestone archived
Resume from: run `/gsd-new-milestone` to start v3.1 (requirements → research → roadmap)

---

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

### Critical Implementation Traps (carried forward)

- **Every new Op variant must be added to 4 places:** `dispatch()` in `ops/mod.rs` + `execute_op()` in `ops/program.rs` + `hp41-cli/src/prgm_display.rs` + `hp41-gui/src-tauri/src/prgm_display.rs`. Exhaustive matches fail to compile if any is missed. Math Pac I added ~40 variants — `math1_op_test_count.rs` + `xrom_shadowing.rs` cross-check 45 `Op` variants × 14 test files + 52 `MATH_1.ops` × 18-entry allowlist at CI time.
- **New CalcState fields need `#[serde(default)]`** for backward compatibility with v1.0–v3.0 save files. Transient fields (`integ_state`, `solve_state`, `modal_program`, `cancel_requested`) additionally carry `#[serde(skip)]`.
- **SC-4 invariant (no core duplication in hp41-gui):** stricter grep `grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` — `op_display_name` is the only intentional exception.
- **`hp41-core/src/ops/math1/` is frozen** since Plan 25-01 (note: actually since Plan 28-01 for math1 specifically). Math Pac I algorithms re-derived from HP OM 00041-90034 (1979); Free42 consulted as sanity-check oracle only, never copied. Every file carries the verbatim disclaim header. CI-enforced via `scripts/check-free42-contamination.sh` in `just license-audit` + dedicated `.github/workflows/ci.yml::license-audit` parallel job.
- **No `println!`/`eprintln!` in hp41-core:** route side effects via `print_buffer` (existing channel; used for Math Pac I prompts) or `event_buffer`.
- **`pending_input` routing block must remain ABOVE modal-opening interceptors** to prevent active dialogs being silently discarded.
- **D-07 (no silent discards) preserved across CLI + GUI:** v3.0 module functions surface as `GuiError`-toast or modal flow, never silent.
- **HP-copyrighted ROM-image redistribution is permanently excluded** — v3.x is BEHAVIORAL emulation only.
- **Pitfall 14 (cross-platform drift):** `tests/lint_math1_assertions.rs` blocks `assert_eq!(decimal, decimal)` on iterated results; relative tolerance 1e-7 documented as Math Pac I floor.

### Blockers

None.

### Quick Tasks Completed (historical record)

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260508-y30 | CHS during EEX entry: toggle minus sign in exponent | 2026-05-08 | aa0904b | [260508-y30-eex-chs-exponent-sign-toggle](./quick/260508-y30-eex-chs-exponent-sign-toggle/) |
| 260508-06h | FIX/SCI/ENG digit-count modal via F key (0–9) | 2026-05-08 | 7ff792c | [260508-06h-fix-sci-eng-digit-input](./quick/260508-06h-fix-sci-eng-digit-input/) |
| 260513-v21a | v2.1 Card Reader: WDTA/RDTA/WPRGM/RDPRGM + XEQ-by-name + cards module + PR #9 fixes | 2026-05-13 | 72530dc…f4b3f8b | — (no GSD dir; see MILESTONES.md v2.1) |
| 260513-v21b | v2.1 Keyboard Authenticity: 5-col grid, three-label keys, one-shot SHIFT, run_stop Tauri cmd, stub-error pattern, toast overlay, PR #10 fixes | 2026-05-13 | 8cd2de4…ff56b97 | — (no GSD dir; see MILESTONES.md v2.1) |
| 260520-v30p | v3.0 post-graduation polish batch: MSRV-CI `uninlined_format_args` fix, E2E SINH+MATRIX assertion path, right-panel XROM-exclusion filter, `?` overlay Clear-widget z-order, `?` overlay incremental substring search | 2026-05-20 | (within `v2.2..v3.0` range) | — (recorded under v3.0 narrative in PROJECT.md) |

---

## Session Continuity

**Last active:** 2026-05-21
**Last action:** v3.0 milestone archived via `/gsd-complete-milestone`. ROADMAP collapsed; v3.0 phase directories moved to `milestones/v3.0-phases/`; orphaned 09–18 retro-archived to `milestones/v1.1-phases/` and `milestones/v2.0-phases/`; REQUIREMENTS.md deleted (fresh slate for next milestone); stale `.planning/v1.0-MILESTONE-AUDIT.md` removed (duplicate of archived copy).
**Next action:** `/gsd-new-milestone` to start v3.1 (Stat 1 Pac per scope lock 2026-05-13) — defines requirements, runs research, and writes a fresh ROADMAP entry.

---
*State initialized: 2026-05-06*
*Last updated: 2026-05-21 — v3.0 archived; project idle awaiting v3.1*
