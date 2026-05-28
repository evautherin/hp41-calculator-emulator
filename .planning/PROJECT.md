# HP-41 Calculator Emulator

## Current Milestone: v4.0 Platform Maturity

**Goal:** Evolve the emulator from feature-complete XROM emulation to a polished, user-friendly platform with visual themes, comprehensive onboarding, GUI keyboard parity, and HP-41 community file exchange (`.raw` + Extended Memory).

**Target features:**
- Built-in skin themes (3-4 presets: dark, light, classic beige, high-contrast)
- First-run quick-start guide + searchable in-app function reference
- GUI keyboard parity — close physical keyboard shortcut gaps with CLI
- `.raw` program file import/export (HP-41 community standard)
- Extended Memory model (EMDIR, EMROOM, EMREG)

<details>
<summary>v3.3 Advantage Pac Emulation (shipped 2026-05-26 — see <code>milestones/v3.3-ROADMAP.md</code>)</summary>

**Goal:** Behavioral emulation of the HP-41C Advantage Pac (OM 00041-90482) as the fourth and fifth XROM application modules (ADV_MATH_A XROM 22 + ADV_MATH_B XROM 24) — completing all remaining HP-41 module emulation.

**Delivered:** 5 phases (43–47), 18 plans, ~117 new Op variants across ADV CONV/MTRX (63 ops) + ADV MATH/TVM (51 ops), named-matrix model (`Vec<AdvMatrix>`), FROOT Laguerre polynomial root-finder, Romberg integration, TVM solver, dual XROM ID design, ~10.9K LOC in `hp41-core/src/ops/advantage/`, full CLI + GUI integration, 3262 total tests, ~95% region coverage, README hard-claim graduated.

</details>

<details>
<summary>v3.2 Time Pac Emulation (shipped 2026-05-25 — see <code>milestones/v3.2-ROADMAP.md</code>)</summary>

**Goal:** Behavioral emulation of the HP-41CX Time Module (OM 00041-90035) as the third XROM application module — date/time arithmetic backed by the host system clock, full alarm catalog with past-due detection, and live-updating stopwatch/clock with real-time LCD display.

**Delivered:** 5 phases (38–42), 19 plans, 81 requirements, 35 new Op variants, ~4650 LOC in `hp41-core/src/ops/time/`, pure-Rust Gregorian calendar arithmetic (Fliegel-Van Flandern JDN), full CLI + GUI integration with live clock/stopwatch display, 2397 hp41-core tests, 96.01% region coverage, README hard-claim graduated.

</details>

<details>
<summary>v3.1 Stat 1 Pac Emulation (shipped 2026-05-24 — see <code>milestones/v3.1-ROADMAP.md</code>)</summary>

**Goal:** Behavioral emulation of the HP-41C Stat 1 Pac (HP 00041-14001, OM 00041-90030) as the second XROM application module — 13 programs / 26 XEQ entry points across univariate / ANOVA / regression / hypothesis / nonparametric / distribution / RNG families.

**Delivered:** 5 phases (33–37), 23 plans, 66 requirements, 26 new Op variants, 3 hand-coded distribution primitives, RAND/SEED bonus utility, full CLI + GUI integration, 791-case numerical accuracy at 98.86%.

</details>

<details>
<summary>v3.0 Math Pac I Emulation (shipped 2026-05-20 — see <code>milestones/v3.0-ROADMAP.md</code>)</summary>

**Goal:** Behavioral emulation of the HP-41C Math Pac I (OM 00041-90034) as the first XROM application module — 10 prompt-driven workflow programs with ~55 XEQ-by-name entry points, modal-workflow layer, user-callback re-entrancy infrastructure.

**Delivered:** 5 phases (28–32), 33 plans (26 original + 7 gap-closure), ~40 new Op variants, XROM resolver chain, modal-workflow state machine, user-callback re-entrancy, complex stack overlay, hyperbolics, triangle solvers, coordinate transforms, Fourier series, full CLI + GUI integration, 95.39% line / 94.26% region coverage, 763-case numerical accuracy at 99.3%.

</details>

---

## Project History

**Shipped milestones:**
- v1.0 CLI (2026-05-08) — Phases 1–8, foundational RPN engine + TUI
- v1.1 CLI Feature Completeness (2026-05-09) — Phases 9–12, EEX/STO-Arith/Print/Synthetic
- v2.0 Tauri GUI (2026-05-10) — Phases 13–18, pixel-perfect HP-41C desktop app
- v2.1 Card Reader + Keyboard Authenticity (2026-05-13) — recorded as quick tasks (no Phase 19 GSD directory); 50 commits since `v2.0` tag
- v2.2 HP-41CV Feature Completeness (2026-05-15) — Phases 20–27, 26 plans; ROM-built-in set komplett (≈130 ops); coverage gate atomically auf 95 % gehoben; WebdriverIO E2E-Smoke auf Ubuntu; tag `v2.2` auf `main`
- v3.0 Math Pac I Emulation (2026-05-18) — Phases 28–32, 33 plans (26 original + 7 gap-closure); README hard-claim graduated via post-Phase-32 gap-closure run; final coverage 95.39 % lines / 94.26 % regions on `hp41-core`
  - Phase 28 XROM Framework + Math Pac I Core Ops (2026-05-16) — `hp41-core` only; 10 plans; ~40 new Op variants; XROM resolver chain fires LAST; modal-workflow state machine; user-callback re-entrancy infrastructure; 5 irreversible decisions locked (ADR-001 through ADR-005)
  - Phase 29 CLI Integration (2026-05-17) — `hp41-cli` only; 3 plans; `xeq_by_name_local_resolve` wired to `xrom_resolve`; second `OnceLock<Vec<HelpEntry>>` for Math Pac I JSON; ~40 new `op_display_name` arms; modal-prompt routing through `print_buffer`
  - Phase 30 Documentation & ADRs (2026-05-17) — `docs/` + tooling only; 3 plans; matrix-renderer two-input extension; 3 new ADRs; divergence catalog expansion; v3.0 narrative across README/PROJECT.md/CLAUDE.md
  - Phase 31 GUI Integration (2026-05-18) — `hp41-gui` only; 5 plans; CATALOG 2 XROM enumeration + Math Pac I help overlay + LCD-alternation modal prompts + R/S 3-way + Esc cascade + request_cancel channel
  - Phase 32 Test Hardening & Quality Gates (2026-05-18) — `tests/` + `scripts/` + `.github/` + `justfile` only; 10 plans (3 original + 7 gap-closure); meta-gate graduation (`math1_op_test_count` + `xrom_shadowing` actively cross-check 45 Op variants × 14 test files + 52 MATH_1.ops × 18-entry allowlist); `lint_math1_assertions.rs` Pitfall 14 + 17 discipline; `numerical_accuracy.rs` 566 → 763 cases (99.3 % pass); E2E smoke extended (`sinh(1)` + `MATRIX DET` Math Pac I workflows on Ubuntu); `scripts/check-free42-contamination.sh` D-32.7 12-symbol guard in `just ci` + `ci.yml::license-audit` parallel job (D-32.8). Gap-closure run (Plans 32-04..32-10) added ~70 error-branch tests across 9 new files, closing the coverage gate from 91.74 % → 95.39 % lines / 92.14 % → 94.26 % regions; README v3.0 line graduated to the OM-cited hard claim per D-32.5.
- v3.1 Stat 1 Pac Emulation (2026-05-24) — Phases 33–37, 23 plans; second XROM application module (13 programs, 26 XEQ entry points, RAND/SEED extension, distribution primitives, ANOVA family, multiple + polynomial regression, hypothesis tests); 95.84 % region coverage; 98.86 % numerical accuracy (791 cases); tag `v3.1`
- v3.2 Time Pac Emulation (2026-05-25) — Phases 38–42, 19 plans; third XROM application module (HP 82182A Time Module, XROM 26, 35 XEQ entry points across clock/date/alarm/stopwatch); first real-time behavior in the emulator; pure-Rust JDN calendar arithmetic; 96.01 % region coverage; 2397 hp41-core tests; tag `v3.2`
- v3.3 Advantage Pac Emulation (2026-05-26) — Phases 43–47, 18 plans; fourth and fifth XROM application modules (ADV_MATH_A XROM 22 + ADV_MATH_B XROM 24, ~117 XEQ entry points across base conversion / boolean / matrix / complex / polynomial / solver / TVM); dual XROM ID hardware-faithful design; named-matrix model; FROOT Laguerre; Romberg integration; TVM solver; ~95 % region coverage; 3,262 total tests; tag `v3.3`

## What This Is

A faithful Rust-based behavioral emulation of the HP-41C/CV/CX programmable RPN calculator, delivered as a keyboard-driven TUI CLI (`hp41-cli`) and a Tauri v2 desktop GUI (`hp41-gui`), both backed by a UI-agnostic core library (`hp41-core`). v3.3 shipped 2026-05-26 with the complete ROM built-in set (~130 ops) plus four XROM application modules: Math Pac I (XROM 7, ~55 entry points), Stat 1 Pac (XROM 2, 26 entry points), Time Module (XROM 26, 35 entry points), and Advantage Pac (XROM 22+24, ~117 entry points) — all feature-complete per their respective Owner's Manuals.

## Core Value

Faithful HP-41 RPN fidelity — the four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to the original hardware; everything else is secondary.

## Requirements

### Validated (v1.0)

- ✓ CORE-01: 4-level RPN stack (X/Y/Z/T) and LASTX register — v1.0 Phase 1
- ✓ CORE-02: Stack-lift semantics for all ~130 ops (Enable/Disable/Neutral) — v1.0 Phase 1
- ✓ MATH-01: Core arithmetic with 10-digit accuracy — v1.0 Phase 2
- ✓ MATH-02: Trig in DEG/RAD/GRAD; SIN on 'q' key — v1.0 Phases 2+8
- ✓ MATH-03: FIX/SCI/ENG formatting with mantissa carry — v1.0 Phase 2
- ✓ REGS-01: STO/RCL/CLREG; CLREG on 'g' key — v1.0 Phases 2+8
- ✓ ALPH-01: 24-char ALPHA mode; AlphaClear on Delete — v1.0 Phases 2+8
- ✓ PROG-01: Keystroke programming (LBL/GTO/XEQ/RTN/conditionals/ISG/DSE) — v1.0 Phase 3
- ✓ PROG-02: ISG/DSE CCCCC.FFFDD counter (string-split, not float) — v1.0 Phase 3
- ✓ DISP-01: 12-char HP-41 display + annunciators in TUI — v1.0 Phase 4
- ✓ DISP-02: Persistent T/Z/Y/X/LASTX panel — v1.0 Phase 4
- ✓ INPUT-01: All functions via keyboard; EEX functional — v1.0 Phases 4+8
- ✓ PERS-01: JSON save/load with full CalcState serde — v1.0 Phase 5
- ✓ PERS-02: 30s auto-save + exit save — v1.0 Phase 5
- ✓ UX-01: '?' help overlay (accurate key reference) — v1.0 Phases 5+8
- ✓ UX-02: USER mode + custom key assignments — v1.0 Phase 5
- ✓ UX-03: 10 bundled sample programs — v1.0 Phase 5
- ✓ SCI-01: Statistics (Σ+/−, MEAN, SDEV, L.R.) — v1.0 Phase 6
- ✓ SCI-02: HMS↔H conversions — v1.0 Phase 6
- ✓ QUAL-01: Cold-start 2.2ms (≤500ms) — v1.0 Phase 7
- ✓ QUAL-02: ~65 ns/op dispatch (≤50ms) — v1.0 Phase 7
- ✓ QUAL-03: Zero panics in hp41-core — v1.0 Phase 7
- ✓ QUAL-04: 94.87% test coverage (≥80%) — v1.0 Phase 7
- ✓ QUAL-05: CI green on Windows/macOS/Ubuntu — v1.0 Phase 7
- ✓ QUAL-06: 495/500 accuracy cases (99% ≥ 98%) — v1.0 Phase 7

### Validated (v1.1)

- ✓ EEX trailing-e-without-exponent behavior (HP-41 hardware lock on partial exponent) — v1.1 Phase 9 (2026-05-08)
- ✓ MSRV 1.85 enforcement + rust_decimal 1.42 — v1.1 Phase 9 (2026-05-08)
- ✓ STO arithmetic keyboard modals (STO+/-/×/÷ interactive key binding) — v1.1 Phase 10 (2026-05-08)
- ✓ FR-17: Print emulation (PRX/PRA/PRSTK) to console/text file — v1.1 Phase 11 (2026-05-08)
- ✓ FR-20: Synthetic programming (GETKEY, NULL, hidden registers M/N/O, HexModal) — v1.1 Phase 12 (2026-05-09)

### Validated (v2.0)

- ✓ GUI-01: `hp41-gui` Tauri v2 binary added to Cargo workspace; builds and launches on macOS, Windows, Linux — v2.0 Phase 13
- ✓ GUI-02: SVG skin renders pixel-perfect HP-41C key layout (44 keys, 9+8+9+9+9 rows, ENTER double-width, authentic HP-41C colors) — v2.0 Phase 16
- ✓ GUI-03: Clickable keys in the SVG skin trigger the same `Op` dispatch as their CLI keyboard counterparts — v2.0 Phase 16
- ✓ GUI-04: HP-41 12-char dot-matrix display and annunciators render in the GUI, updating after every op — v2.0 Phase 15
- ✓ GUI-05: `hp41-core` integrated via Tauri Rust commands — no duplication of CalcState logic (SC-4 invariant verified) — v2.0 Phase 14
- ✓ GUI-06: `hp41-cli` remains fully functional and unmodified after adding `hp41-gui` to the workspace — v2.0 Phase 13
- ✓ WSPC-01/02: Workspace isolation; `just ci` stays green; nested workspace never affects root Cargo resolver — v2.0 Phase 13
- ✓ IPC-01/02: `dispatch_op`/`get_state` Tauri commands; `CalcStateView` ≤300 bytes; physical keyboard fully wired — v2.0 Phases 14+15
- ✓ SKIN-01/02/03: Pixel-perfect SVG skin with click-to-dispatch and CSS press animation — v2.0 Phase 16
- ✓ DISP-01/02: 12-char display + 5 annunciators + X/Y/Z/T/LASTX stack panel — v2.0 Phase 15
- ✓ PERS-01/02: Shared `~/.hp41/autosave.json`; 30s auto-save; scrollable print panel — v2.0 Phase 17
- ✓ PROG-01: PRGM-mode program listing with SST/BST navigation; cross-platform GUI CI — v2.0 Phase 18

### Validated (v2.1)

- ✓ CARD-01: HP 82104A Card Reader behavioral emulation — `Op::Wdta` / `Op::Rdta` / `Op::Wprgm` / `Op::Rdprgm` + `CardOpRequest` drain — v2.1 (PR #9)
- ✓ CARD-02: XEQ-by-name fallback resolves to `builtin_card_op` — works in `op_xeq`, `run_program`, `run_loop` — v2.1 (PR #9)
- ✓ CARD-03: `cards` modules mirrored in hp41-cli and hp41-gui/src-tauri (dir resolution, sanitize, drain); CLI comfort shortcuts `Ctrl+W/R/D/F`; SHA-256 round-trip tests — v2.1 (PR #9)
- ✓ SKIN-06: Authentic 5×8 keyboard layout in `hp41-gui/src/Keyboard.tsx` — 4 top-row mode buttons + 35-key main grid, ENTER 2-wide, three-label `KeyDef` (primary / shifted / alphaChar) — v2.1 (PR #10)
- ✓ INPUT-02: One-shot SHIFT prefix — frontend-only `shiftActive` state, never crosses IPC; consumes itself after dispatch; ALPHA overrides SHIFT (D-divergence) — v2.1 (PR #10)
- ✓ INPUT-03: `run_stop` Tauri command (symmetric with sst_step/bst_step); R/S key click-reachable for the first time — v2.1 (PR #10)
- ✓ UX-04: Stub-error pattern (D-5) — `pi`, `polar_to_rect`, `rect_to_polar`, `beep`, `asn`, `catalog`, `view`, `xeq_prompt`, `gto_prompt`, `lbl_prompt` return `GuiError` surfaced as 2 s toast; never silently discarded — v2.1 (PR #10)

### Validated (v2.2 — HP-41CV Feature Completeness, shipped 2026-05-15)

**Core math / conversions (Phase 20):**
- ✓ FN-MATH-01: `PI`, `P→R`, `R→P` (polar/rect conversion respecting angle mode) — v2.2 Phase 20
- ✓ FN-MATH-02: `RND` (round X to current display setting), `FRC` (fractional part) — v2.2 Phase 20
- ✓ FN-MATH-03: `MOD` (Y mod X, sign follows Y per HP-41C/CV QRG), `ABS`, `FACT` (factorial 0–69), `SIGN` — v2.2 Phase 20
- ✓ FN-STACK-01: `R↑` (roll up — mirror of `Rdn`) — v2.2 Phase 20

**Core flags & display (Phase 21):**
- ✓ FN-FLAG-01: 56 user flags + system flags 00–55 as `flags: u64` on `CalcState` — v2.2 Phase 21
- ✓ FN-FLAG-02: `SF n`, `CF n`, `FS? n`, `FC? n`, `FS?C n`, `FC?C n` — v2.2 Phase 21
- ✓ FN-DISP-01: `VIEW nn`, `AVIEW` — v2.2 Phase 21
- ✓ FN-DISP-02: `PROMPT`, `AON` / `AOFF`, `CLD` — v2.2 Phase 21
- ✓ FN-SOUND-01: `BEEP` / `TONE n` (CLI silent stubs; GUI real tones via Web Audio) — v2.2 Phase 21

**Core program control (Phase 22):**
- ✓ FN-PROG-01: `STOP`, `PSE` — v2.2 Phase 22
- ✓ FN-PROG-02: `CLP`, `DEL nnn`, `INS` — v2.2 Phase 22
- ✓ FN-PROG-03: `GTO IND nn`, `XEQ IND nn` — v2.2 Phase 22
- ✓ FN-MEM-01: `SIZE`, `PACK`, `MEM LOST`, `CATALOG 1` — v2.2 Phase 22
- ✓ FN-USER-01: `ASN`, `CLA`, `CLST` — v2.2 Phase 22

**Core ALPHA & indirection (Phases 23, 24):**
- ✓ FN-ALPHA-01: `ARCL nn`, `ASTO nn` — v2.2 Phase 23
- ✓ FN-ALPHA-02: `ATOX`, `XTOA` — v2.2 Phase 23
- ✓ FN-ALPHA-03: `AROT n`, `POSA` — v2.2 Phase 23
- ✓ FN-IND-01: 11-variant `*Ind` family (`STO IND`, `RCL IND`, `ISG IND`, `DSE IND`, `SF IND`, `CF IND`, `FS? IND`, `FC? IND`, `STO+/-/×/÷ IND`) — v2.2 Phase 24

**CLI integration & documentation (Phase 25):**
- ✓ FN-CLI-01: Keyboard modals for prompt IDs (`sf_prompt`, `fs_prompt`, `cf_prompt`, register prompts) — v2.2 Phase 25
- ✓ FN-CLI-02: Four conditional tests on f-arith keys (`X=Y`, `X≤Y`, `X>Y`, `X=0`); remaining 8 routed through XEQ-by-name modal — v2.2 Phase 25
- ✓ FN-CLI-03: `?` help overlay JSON-derived from `docs/hp41cv-functions.json` — v2.2 Phase 25
- ✓ FN-DOC-01: `docs/hp41cv-function-matrix.md` regenerated from canonical JSON via `scripts/docs-matrix` (`just docs-matrix` / `just docs-matrix-check`) — v2.2 Phase 25

**GUI integration & polish (Phase 26):**
- ✓ FN-GUI-01: All v2.2 key IDs in `key_map.rs::resolve` + `KEY_DEFS` with three-label shift/alpha bindings — v2.2 Phase 26
- ✓ FN-GUI-02: Modal routing for previously-stubbed prompt IDs replaces `unknown key` toast — v2.2 Phase 26
- ✓ FN-GUI-03: Stub-error arm shrunk to v3.x module-pac functions only — v2.2 Phase 26
- ✓ SKIN-04: 14-segment SVG font for LCD rendering — v2.2 Phase 26
- ✓ SKIN-05: `?` keyboard-shortcut overlay (ported from CLI `help_data.rs`) — v2.2 Phase 26
- ✓ PROG-02: USER-mode keyboard-assignment display — v2.2 Phase 26
- ✓ PROG-03: `prgm_mode` binding rebound to `p` key — v2.2 Phase 26

**Test hardening (Phase 27):**
- ✓ FN-QUAL-01: `hp41-core` coverage 95.25 % (gate atomically raised 80 % → 95 %, D-27.2) — v2.2 Phase 27
- ✓ FN-QUAL-02: 566-case numerical accuracy at 99.1 % (561/566); v1.x 503-case baseline floor 498 preserved — v2.2 Phase 27
- ✓ FN-QUAL-05: WebdriverIO + tauri-driver E2E smoke on Ubuntu (`e2e-linux` job in `ci-gui.yml`) — v2.2 Phase 27
- ✓ D-27.14: Vitest CI-gated in `just gui-ci` (142/142 tests) — v2.2 Phase 27

### Validated (v3.0 — Math Pac I Emulation, shipped 2026-05-20)

- ✓ **XROM-01..09**: XROM framework + `XromModule` registry + `MATH_1` const + `xrom_resolve` (fires LAST in resolver chain per Pitfall 1); 6 new `CalcState` fields with `#[serde(default)]` / `#[serde(skip)]`; user-callback strict-reject policy per ADR-002 — v3.0 Phase 28
- ✓ **HYP-01..06**: Hyperbolics (`Sinh`, `Cosh`, `Tanh`, `Asinh`, `Acosh`, `Atanh`) with domain-error returns — v3.0 Phase 28
- ✓ **CMPLX-01..17 + CMPLX-18 (derived D-28.3 `XEQ "REAL"`)**: complex stack overlay on X/Y/Z/T, `C+/C-/C×/C÷`, 13 complex functions, branch-cut handling, `complex_atan2(0,0)→0` first arm, zero-divisor branch before division — v3.0 Phase 28
- ✓ **POLY-01..07**: POLY/ROOTS workflow with `DEGREE=?` modal, complex root-pair output `U=u / V=v / U=u / -V=-v`, multiplicity-as-cluster (documented in divergences), `|imag|>10⁹` non-convergence → `DATA ERROR` — v3.0 Phase 28
- ✓ **MAT-01..11**: MATRIX workflow with `ORDER=?` modal, Gauss-Jordan inverse, OM-transcribed EPSILON, order in R14, column-major elements from R15, max ORDER=14, `NO SOLUTION` on singular — v3.0 Phase 28
- ✓ **INTG-01..08 / SOLV-01..08 / DIFEQ-01..05**: user-callback infrastructure via `run_loop` re-entrancy (NOT `run_program`), 4-deep `call_stack` cap, subdivision cap 2¹⁵, convergence threshold = `10^(-decimals - 1)`; strict-reject nested per XROM-08 — v3.0 Phase 28
- ✓ **FOUR-01..06 / TRI-01..05 / TRANS-01..05**: DFT with rect/polar toggle, 5 triangle solvers (Law of Sines/Cosines + ambiguous SSA), 2D/3D coordinate transformations with Rodrigues rotation — v3.0 Phase 28
- ✓ **CLI-01..05**: `xeq_by_name_local_resolve` → `xrom_resolve`; second `OnceLock<Vec<HelpEntry>>` from `docs/hp41-math1-functions.json` (DOC-01 absorbed per D-29.1); ~40 new `op_display_name` arms; modal-prompt routing through `print_buffer` — v3.0 Phase 29
- ✓ **DOC-01..07**: `scripts/docs-matrix` two-input extension, `docs/hp41-math1-function-matrix.md` regenerated, three-bucket divergence catalog, 3 new ADRs (v3.0-001/002/005), README soft-claim, CLAUDE.md `### v3.0 additions` block — v3.0 Phases 29 + 30
- ✓ **GUI-01..07**: GUI parity through shared `xrom_resolve` (no duplicate resolver), CATALOG 2 XROM enumeration, Math Pac I help overlay parallel-load, LCD-alternation modal prompts, `request_cancel` cancellation channel (`Arc<AtomicBool>` + per-64-samples lock release, Pitfall 11 mitigation), stub-error arm preserved for future XROM modules — v3.0 Phase 31
- ✓ **QUAL-01..08**: `hp41-core` 95.39 % lines / 94.26 % regions; `numerical_accuracy.rs` 566 → 763 cases at 99.3 % pass (v1.x 503-case floor 498 preserved); E2E smoke extended with `sinh(1)` + `MATRIX DET` Math Pac I workflows on Ubuntu; `scripts/check-free42-contamination.sh` 12-symbol guard in `just ci` + `ci.yml::license-audit` parallel job; per-Op test count ≥ 5 (Pitfall 16); `xrom_shadowing.rs` Pitfall 1 CI gate; `math1_user_callback.rs` 5 re-entrancy regression tests — v3.0 Phase 32
- ✓ **D-32.5 README v3.0 hard-claim graduation**: README "Math Pac I" line replaced "soft-claim" with the OM-cited "feature-complete per Owner's Manual 00041-90034" wording after the post-Phase-32 gap-closure run (Plans 32-04..32-10) lifted coverage from 91.74 % → 95.39 % lines — v3.0 Phase 32 (Plan 32-10 SHIP commit)

### Validated (v3.1 — Stat 1 Pac Emulation, shipped 2026-05-24)

- ✓ **STAT-FW-01..04**: XROM framework activation (STAT_1 XROM ID 2, 4-way invariant, XEQ-by-name) — v3.1 Phase 33
- ✓ **STAT-UNI-01..04**: Extended univariate stats (ΣBSTAT/BSTG, ΣMMTUG/MMTGD, correction key) — v3.1 Phase 33
- ✓ **STAT-AOV-01..04**: ANOVA family (ΣAOVONE/AOVTWO/ANOCOV, OM register layout) — v3.1 Phase 33
- ✓ **STAT-REG-01..09**: Regression family (ΣLIN/EXP/LOGI/POW, ΣMLRXY/MLRXYZ, ΣPOLYP/POLYC, self-contained Gauss elimination) — v3.1 Phase 33
- ✓ **STAT-HYP-01..07**: Hypothesis tests (ΣPTST/ΣTSTAT, ΣXSQEV/ΣEFXSQ, ΣCTKKK/CTKK, ΣSPEAR) — v3.1 Phase 33
- ✓ **STAT-DST-01..07**: Distribution evaluators (ΣNORMD CDF/PDF/inverse, ΣCHISQD CDF/PDF, 3 hand-coded primitives) — v3.1 Phase 33
- ✓ **STAT-RNG-01..04**: RNG bonus utility (RAND/SEED, persistent `rand_seed` field) — v3.1 Phase 33
- ✓ **STAT-CLI-01..05**: CLI integration (3-pool JSON help, 26 `op_display_name` arms, `?` overlay) — v3.1 Phase 34
- ✓ **STAT-DOC-01..06**: Documentation (divergences, function matrix, 5 ADRs, README/CLAUDE.md) — v3.1 Phase 35
- ✓ **STAT-GUI-01..05**: GUI integration (CATALOG 2, help overlay, modal prompts, bounded-iter waiver) — v3.1 Phases 36–37
- ✓ **STAT-QUAL-01..11**: Quality gates (coverage, accuracy, meta-gates, backward-compat, E2E smoke) — v3.1 Phase 37

### Validated (v3.2 — Time Pac Emulation, shipped 2026-05-25)

- ✓ **TIME-FW-01..06**: XROM framework activation (TIME_MODULE XROM ID 26, 35 ops, `xrom_resolve` bit-2 arm, `default_xrom_modules` → `0b0000_0111`, `migrate_after_load()` v3.1→v3.2) — v3.2 Phase 38
- ✓ **TIME-CLK-01..06**: Clock ops (TIME/DATE/SETIME/SETDATE/T+X/CORRECT with `SystemTime::now()` + `time_offset_secs` delta) — v3.2 Phase 38
- ✓ **TIME-DAT-01..06**: Date arithmetic (DATE+/DDAYS/DOW via Fliegel-Van Flandern JDN, DMY/MDY format, Gregorian calendar) — v3.2 Phase 38
- ✓ **TIME-DSP-01..05**: Clock display mode (CLKT/CLKTD/CLOCK, 12h/24h, pull-on-redraw live display) — v3.2 Phases 38–39
- ✓ **TIME-FMT-01..05**: Time format ops (CLK12/CLK24/SETAF/RCLAF) — v3.2 Phase 38
- ✓ **TIME-SW-01..09**: Stopwatch (RUNSW/STOPSW/RCLSW/SETSW/SWPT/STPW/SW with Instant monotonic timing, centisecond resolution) — v3.2 Phases 38–39
- ✓ **TIME-ALM-01..12**: Alarm system (XYZALM/RCLALM/ALMCAT/CLALMA/CLALMX/CLRALMS/ALMNOW, 253-entry catalog, past-due detection, `check_alarms()` drain, repeat intervals) — v3.2 Phases 38–39
- ✓ **TIME-CLI-01..08**: CLI integration (4th JSON pool, 35 `op_display_name` arms, live clock/stopwatch display, stopwatch keyboard mode, alarm event draining) — v3.2 Phase 39
- ✓ **TIME-DOC-01..06**: Documentation (divergences catalog, function matrix, 3 ADRs, README soft-claim, CLAUDE.md/architecture-history.md) — v3.2 Phase 40
- ✓ **TIME-GUI-01..07**: GUI integration (CATALOG 2, help overlay, `tick_time` conditional setInterval, alarm toast, LCD-alternation modal prompts) — v3.2 Phase 41
- ✓ **TIME-QUAL-01..11**: Quality gates (96.01% region coverage, 30 date accuracy cases, stopwatch timing, alarm latency, unified meta-gates, backward compat, E2E DDAYS smoke, README hard-claim graduated) — v3.2 Phase 42

### Validated (v3.3 — Advantage Pac Emulation, shipped 2026-05-26)

- ✓ **ADV-FW-01..06**: Dual XROM framework activation (ADV_MATH_A XROM 22 bit-3 + ADV_MATH_B XROM 24 bit-4, `default_xrom_modules` → `0b0001_1111`, `migrate_after_load()` v3.2→v3.3) — v3.3 Phase 43
- ✓ **ADV-MTX-01..12**: Named-matrix model (`Vec<AdvMatrix>`, NEWMAT/GETM/PUTM/MRCL/MSTO/MTRXD + element access ops, Math Pac I isolation per D-43.5) — v3.3 Phase 43
- ✓ **ADV-CMPLX-01..14**: Extended complex ops (ADV_MATH_B module, 12 intentional MATH_1 overlaps per ADR-v3.3-003) — v3.3 Phase 43
- ✓ **ADV-POLY-01..06**: FROOT Laguerre polynomial root-finder (arbitrary degree, quadratic deflation, f64 intermediate arithmetic per ADR-v3.3-002) — v3.3 Phase 43
- ✓ **ADV-SOLV-01..08**: FINTG Romberg integration + FSOLVE + FDIFEQ coexisting with Math Pac I solvers (cross-nesting allowed per D-43.7) — v3.3 Phase 43
- ✓ **ADV-TVM-01..08**: Time Value of Money (N/I%YR/PV/PMT/FV/AMORT, persistent `adv_tvm_state` per D-43.11) — v3.3 Phase 43
- ✓ **ADV-BASE-01..08**: Base conversion + 36-bit boolean ops (BININ/BINDC/BINOCT/BINHEX/DECBIN/OCTBIN/HEXBIN + AND/OR/XOR/NOT/ROTXY per D-43.9) — v3.3 Phase 43
- ✓ **ADV-CLI-01..06**: CLI integration (5th JSON pool, 114 `op_display_name` arms, 5-pool help overlay, xrom shadowing, function matrix) — v3.3 Phase 44
- ✓ **ADV-DOC-01..06**: Documentation (divergences catalog, 4 ADRs, README/CLAUDE.md/architecture-history.md) — v3.3 Phase 45
- ✓ **ADV-GUI-01..06**: GUI integration (CATALOG 2, help overlay, 117 `op_display_name` arms, modal rendering) — v3.3 Phase 46
- ✓ **ADV-QUAL-01..09**: Quality gates (~95% region coverage, 22 oracle accuracy cases, unified meta-gates across 5 XROM modules, backward compat, E2E BININ smoke, README hard-claim graduated) — v3.3 Phase 47

### Active

- [ ] THEME-01: User can select from 3-4 built-in skin themes (dark, light, classic beige, high-contrast)
- [ ] ONBOARD-01: First-run quick-start guide introduces RPN basics, key layout, and function access
- [ ] ONBOARD-02: Searchable in-app function reference with examples (beyond `?` overlay)
- [ ] KBD-01: GUI physical keyboard shortcuts reach parity with CLI
- [ ] RAW-01: User can import `.raw` HP-41 program files
- [ ] RAW-02: User can export programs to `.raw` format
- ✓ **XMEM-01..07**: Extended Memory core — `XmemFile` model, 8 ops (EMDIR/EMROOM/SAVEP/GETP/SAVED/GETD/EMREG/SAVERX) in `hp41-core`, 4-way exhaustive-match wiring, backward-compat serde. Validated in Phase 51: X-MEM Core (2026-05-28). CLI/GUI/keyboard integration deferred to Phase 52.

### Out of Scope

- FR-18 Multiple skin themes — GUI-only, post-v3.x
- FR-22 `.raw` HP-41 program file import/export — could-have
- FR-23 Mobile (iOS/Android) — defer until desktop stable
- Binary releases (signed cross-platform CLI + GUI installers via cargo-dist + tauri-action) — deferred post-v3.3
- X-MEM / Extended Memory file model — post-v3.x scope
- Interrupting control alarm execution — data model ready (D-38.4), requires re-entrancy against 4-level call stack
- Cycle-accurate Nut CPU simulation — high effort, low user value vs. behavioral emulation
- HP-copyrighted ROM image redistribution — legal risk, excluded permanently
- HP-IL peripheral emulation — niche, complex
- Wand/barcode reader emulation — requires hardware, very niche
- Cloud sync — privacy and infrastructure cost

## Context

The project shipped v1.0 through v3.3 in 21 days (2026-05-06 → 2026-05-26) across 47 phases and 8 milestones.

**Codebase metrics (v3.3 ship, 2026-05-26):**

| Component | Source LOC | Test LOC | Tests |
|-----------|-----------|----------|-------|
| `hp41-core/src` | 46,683 | 38,426 | 2,950 |
| `hp41-cli/src` | 6,966 | 6,008 | 421 |
| `hp41-gui/src-tauri/src` | 3,458 | — | — |
| `hp41-gui/src` (React/TS) | 4,883 | — | — |
| **Total Rust** | **101,541** | | **3,371** |

**XROM module LOC breakdown in `hp41-core/src/ops/`:**

| Module | XROM ID | LOC | Ops |
|--------|---------|-----|-----|
| `math1/` | 7 | 12,816 | ~55 |
| `stat1/` | 2 | 6,898 | 26 |
| `time/` | 26 | 4,556 | 35 |
| `advantage/` | 22+24 | 10,872 | ~117 |

**Quality gates (v3.3):** `hp41-core` region coverage ~95% (denominator dilution from 4 XROM modules), numerical accuracy 98.86% (843 cases), 0 panics, 0 Free42 contamination symbols, MSRV 1.88, zero new runtime deps across v3.0–v3.3, CI green on Windows/macOS/Ubuntu.

**Op enum:** ~325 variants in `hp41-core/src/ops/mod.rs` — all subject to the 4-way exhaustive-match invariant (dispatch + execute_op + CLI prgm_display + GUI prgm_display).

## Constraints

- **Tech stack**: Rust stable, MSRV 1.88 — deterministic, GC-free, ideal for emulation core
- **Task runner**: `just` — sole task runner; no bare `cargo` commands in CI or docs
- **Architecture**: `hp41-core` must never depend on `hp41-cli` or `hp41-gui` — enforced at compile time
- **Dependencies**: ratatui 0.30, crossterm 0.29, clap 4.x, serde/serde_json, rust_decimal, criterion (dev), Tauri v2.11 + React 18 + TypeScript + Vite (GUI)
- **Zero runtime dep policy (v3.0+)**: No new runtime deps across v3.0–v3.3; `statrs` and `libc` both rejected
- **Legal**: No HP-copyrighted ROM bytes; Free42 GPL contamination guard CI-enforced (18-token scan)
- **Privacy**: No telemetry; local-only data storage; no network calls (except `SystemTime::now()` syscall)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Behavioral emulation, not cycle-accurate Nut CPU | High effort, low user value | ✓ Good — users don't notice |
| Cargo workspace `hp41-core` / `hp41-cli` | Enforces clean separation; GUI is thin adapter | ✓ Good — `hp41-core` reused unchanged through v3.3 |
| `rust_decimal` for HpNum with 10-digit rounding | BCD-accurate without custom BCD struct | ✓ Good — 98.86% accuracy suite pass rate (843 cases) |
| Stack-lift as `lift_enabled: bool` in Stack | Simplest correct model for 130+ ops | ✓ Good — every op explicitly declares effect |
| ISG/DSE counter fields via string-split | Never use `floor()`/`fmod()` on f64 | ✓ Good — hardware-identical counter behavior; reused for Time Pac date parsing |
| ratatui + crossterm for TUI | Cross-platform, keyboard-driven, stable | ✓ Good — CI green on all 3 platforms |
| `just` as sole task runner | All targets as recipes; contributors never call bare `cargo` | ✓ Good — CI compliance enforced |
| No async in hp41-core | Single-threaded event loop; clock reads via `SystemTime::now()` | ✓ Good — simpler, deterministic |
| `serde_json` for persistence | Human-readable, diff-able, forward-compatible | ✓ Good — v1.0→v3.3 save files load without migration |
| XROM resolver chain fires LAST (v3.0) | `xrom_resolve` after `builtin_card_op`; Pitfall 1 | ✓ Good — 5 modules coexist cleanly |
| Hand-coded distribution primitives (v3.1) | Zero new runtime deps; `statrs` rejected | ✓ Good — ~140 LOC replaces ~50K dep; scipy-verified |
| Direct `SystemTime::now()` in hp41-core (v3.2) | Syscall, not I/O; trait injection rejected | ✓ Good — `time_offset_secs` delta model is simple + testable |
| Fliegel-Van Flandern JDN calendar (v3.2) | Pure Rust; `libc::localtime_r` rejected | ✓ Good — ~60 LOC; no new runtime deps |
| Named-matrix `Vec<AdvMatrix>` model (v3.3) | Math Pac I isolation (D-43.5); `HashMap` rejected | ✓ Good — deterministic serde; separate from R14/R15+ |
| Dual XROM ID design ADV 22+24 (v3.3) | Hardware-faithful two-chip design | ✓ Good — 12 intentional MATH_1 overlaps handled cleanly |
| FROOT Laguerre's method (v3.3) | Arbitrary degree; coexists with Math Pac I Bairstow (degree 2–5) | ✓ Good — converges for all test cases incl. complex roots |
| Persistent TVM state (v3.3) | `serde(default)` without `skip` — second instance after `rand_seed` | ✓ Good — TVM register contents survive save/load |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each milestone** (via `/gsd-complete-milestone`):
1. Move shipped requirements from Active → Validated
2. Add new requirements to Active for next milestone
3. Update Context with current state
4. Audit Key Decisions with outcomes

**Milestone evolution log:**

| Milestone | Shipped | Phases | Plans | Key achievement |
|-----------|---------|--------|-------|-----------------|
| v1.0 CLI | 2026-05-08 | 1–8 | 45 | Foundational RPN engine + TUI |
| v1.1 CLI Feature Complete | 2026-05-09 | 9–12 | 4 | EEX/STO-Arith/Print/Synthetic |
| v2.0 Tauri GUI | 2026-05-10 | 13–18 | 6 | Pixel-perfect HP-41C desktop app |
| v2.1 Card Reader | 2026-05-13 | — | — | Card reader + keyboard authenticity (quick tasks) |
| v2.2 HP-41CV Complete | 2026-05-15 | 20–27 | 26 | Full ROM built-in set (~130 ops) |
| v3.0 Math Pac I | 2026-05-20 | 28–32 | 33 | First XROM module; XROM framework; modal workflows |
| v3.1 Stat 1 Pac | 2026-05-24 | 33–37 | 23 | Second XROM module; distribution primitives; RNG |
| v3.2 Time Pac | 2026-05-25 | 38–42 | 19 | Third XROM module; real-time clock; JDN calendar |
| v3.3 Advantage Pac | 2026-05-26 | 43–47 | 18 | Fourth+fifth XROM modules; named matrices; Laguerre; TVM |

Per-phase detail lives in `docs/architecture-history.md` and the archived milestone directories under `.planning/milestones/`.

---

*Last updated: 2026-05-28 — Phase 51 (X-MEM Core) complete; XMEM-01..07 validated. Next: Phase 52 (Test Hardening + Documentation).*
