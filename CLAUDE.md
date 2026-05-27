# HP-41 Calculator Emulator — Project Guide

## What this is

Faithful Rust behavioral emulation of the HP-41C/CV/CX RPN calculator.

- `hp41-core` — UI-agnostic library; zero CLI/UI deps (enforced at compile time)
- `hp41-cli` — TUI binary (ratatui 0.30 + crossterm 0.29)
- `hp41-gui` — Tauri v2 + React desktop app (nested standalone workspace)

**Current:** v3.3 Advantage Pac (Owner's Manual 00041-90482 feature-complete), shipped 2026-05-26. Earlier tags: `v3.2` (Time Pac, 2026-05-25), `v3.1` (Stat 1 Pac, 2026-05-24), `v3.0` (Math Pac I, 2026-05-21), `v2.2` (HP-41CV complete, 2026-05-16), `v2.0` (Tauri GUI, 2026-05-10), `v1.1`, `v1.0`.

**Where the long-form history lives:**
- `docs/architecture-history.md` — full phase-by-phase narrative + decision rationale per milestone
- `docs/adr/` — 17 numbered ADRs for v3.0–v3.3
- `docs/hp41-*-divergences.md` — per-module OM divergences + emulator extensions (math1, stat1, time, advantage)
- `.planning/milestones/` — archived GSD planning artifacts per shipped milestone
- gbrain: `gbrain search "<terms>"` from anywhere inside this repo

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

- `hp41-core` must never depend on `hp41-cli` or `hp41-gui`. Root `Cargo.toml` members stay `["hp41-core", "hp41-cli"]`; `hp41-gui` is a nested standalone workspace. `tauri` / `tauri-build` appear ONLY in `hp41-gui/src-tauri/Cargo.toml`.
- **SC-4 (no core duplication in GUI):** stricter check `grep -rn "fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)" hp41-gui/src-tauri/src/` must return empty. Display helpers like `op_display_name` are exempt by intent (and duplicated CLI ↔ GUI by design).
- **MSRV 1.88** declared at `[workspace.package]`; member crates inherit via `rust-version.workspace = true`. CI MSRV job runs in parallel — no `needs:`.
- **Bundle ID:** `ch.talent-factory.hp41` (avoid macOS sandbox/keychain issues).

### Core engine

- **BCD/f64:** `rust_decimal` 1.42 with 10-significant-digit rounding. `HpNum` in `hp41-core/src/num.rs`. Custom BCD was evaluated and rejected.
- **Stack-lift:** every op declares `LiftEffect::Enable / Disable / Neutral`. The most commonly mis-implemented HP-41 feature — always check.
- **ISG/DSE counter:** extract fields by string-splitting at the decimal point — **never** `floor()`/`fmod()`. Same discipline applies to Time Pac date decimal parsing.
- **No async, no panics:** `#![deny(clippy::unwrap_used)]` at crate root. Production code uses `.expect("reason")` or `?`-propagation. Test modules carry `#[allow(clippy::unwrap_used)]`. `hp41-gui` mutex locks use `.unwrap_or_else(|e| e.into_inner())` for poisoned-lock recovery.
- **Print emulation:** `println!`/`eprintln!` are forbidden in `hp41-core`. PRX/PRA/PRSTK push into `state.print_buffer` (`#[serde(skip)]`); CLI drains via `call_dispatch_and_drain()` (interactive) and `drain_and_show_print_output()` (programmatic paths) — wire ALL `run_program()` call sites or print output gets dropped.
- **`hp41-core/src/ops/math1/` is frozen** since Plan 25-01. Math Pac I algorithms re-derived from HP OM 00041-90034 (1979); Free42 consulted as sanity-check oracle only, **not** copied. Every file in this directory carries the verbatim disclaim header. **Sanctioned carve-outs (3 files only):** `xrom.rs` (XROM registry for v3.1+ modules, ADR-v3.1-004), `modal.rs` (~8-line dispatch wiring for `ModalProgram::Stat1`/`Time` variants, ADR-v3.1-004), `complex.rs` (`complex_atan2` promoted `pub(crate)` for Advantage Pac complex ops, ADR-v3.3-004). All OTHER files in `math1/` remain frozen.
- **Zero new runtime deps policy:** every milestone since v3.0 has held this line. `statrs` rejected (ADR-v3.1-002), `libc` rejected (v3.2), `chrono` rejected (v3.2). All algorithms hand-coded from primary literature with Free42 as sanity-check oracle only.

### 4-way exhaustive-match invariant

Every new `Op` variant must land in ALL FOUR before any caller compiles:

1. `dispatch()` in `hp41-core/src/ops/mod.rs`
2. `execute_op()` in `hp41-core/src/ops/program.rs`
3. `op_display_name()` in `hp41-cli/src/prgm_display.rs`
4. `op_display_name()` in `hp41-gui/src-tauri/src/prgm_display.rs`

Items 3 + 4 are duplicated by design; the compile-time exhaustive match in both is what makes this invariant load-bearing.

### Resolver chain + never-discard (D-07)

Order in `hp41-cli` keyboard path: `key_to_op` → `shifted_key_to_op` → modal-opener → `xeq_by_name_local_resolve` → `builtin_card_op` → `xrom_resolve` → `Err(InvalidOp)`.

`pending_input` routing block must remain **above** modal-opening interceptors so an active modal is never silently discarded. NEVER silently swallow an unknown id — surface as a toast (GUI) or status-bar error (CLI).

### Save-file backward compat

Every new `CalcState` field carries `#[serde(default)]`. Transient fields also carry `#[serde(skip)]`. v1.0–v3.3 save files load without migration (auto-upgrade via `migrate_after_load()` in `state.rs`).

**Two documented serde exceptions** (`#[serde(default)]` WITHOUT `#[serde(skip)]` — Pitfall 20):
- `rand_seed: HpNum` — reproducible RNG across save/load (ADR-v3.1-001)
- `adv_tvm_state: Option<TvmState>` — TVM register persistence across save/load (D-43.11)

### JSON canonical data flow

Five JSON source-of-truth files drive keybindings, the `?` overlay help, and the right-panel discoverability table:

| File | Module | Entries |
|------|--------|---------|
| `docs/hp41cv-functions.json` | Built-in | ~130 |
| `docs/hp41-math1-functions.json` | Math 1 (XROM 7) | 45 |
| `docs/hp41-stat1-functions.json` | Stat 1 (XROM 2) | 26 |
| `docs/hp41-time-functions.json` | Time (XROM 26) | 35 |
| `docs/hp41-advantage-functions.json` | ADV (XROM 22+24) | 114 |

- Loaded via `include_str!` + `OnceLock` in `hp41-cli/src/help_data.rs` (5-pool `help_entries_all()` chain). Malformed JSON panics at first access — hard-build-blocker by design.
- `scripts/docs-matrix/` regenerates 5 `docs/hp41-*-function-matrix.md` files via `just docs-matrix`. CI drift-catch: `just docs-matrix-check`.
- Bidirectional Op ↔ JSON parity asserted by `hp41-cli/tests/function_matrix_parity.rs` (5-pool partition test); key-coverage closure by `hp41-cli/tests/key_coverage.rs`.
- Right-panel filter: `key_ref_entries()` excludes XROM-module functions via `entry.xrom.is_none()` — module functions discoverable via `?` overlay only.

### CLI ↔ GUI parity (D-25.6)

- **One-shot SHIFT** is frontend-only. `App.shift_armed` (CLI) and `shiftActive` (GUI) mirror each other bit-for-bit. Neither ever appears in `CalcState` / `CalcStateView` / IPC.
- ALPHA overrides SHIFT (known divergence from real HP-41, accepted).
- IND-toggle (shift-0 inside an open Flag/Register modal) is hardware-faithful per HP-41C/CV QRG p.14; reuses the same `shift_armed` bit.

### GUI specifics

- **IPC contract:** `dispatch_op(key_id: &str)`, `get_state()`, `sst_step`, `bst_step`, `run_stop`, `request_cancel`, `tick_time` — Tauri v2 commands. Response is `CalcStateView`. Frontend never touches Rust enums; `key_map::resolve()` translates string IDs.
- **Tauri v2.11 permissions:** for inline app commands (not plugins), create TOML in `hp41-gui/src-tauri/permissions/<cmd-kebab>.toml` with `[[permission]] identifier + commands.allow = ["fn_name"]`, then reference in `capabilities/default.json`. Run `cargo check` first so the permission registry is generated.
- **SVG animation:** `.key` needs `transform-box: fill-box` + `transform-origin: center`.
- **busyRef debounce:** two-layer `useRef(false)` guard against concurrent `invoke()`. Pair with `pressedKey` functional setState to avoid stale closure.
- **No polling (D-11):** frontend does not poll `get_state()`. Live clock/stopwatch use `tick_time` via `setInterval` (200ms).
- **Persistence sharing:** `hp41-gui` and `hp41-cli` read/write the SAME `~/.hp41/autosave.json`. Auto-save thread releases the `AppState` Mutex BEFORE disk I/O.

### Free42 GPL-contamination guard

CI-enforced via `scripts/check-free42-contamination.sh` in `just license-audit`. Greps for 18 distinctive Free42 / Intel BID / decNumber / GPL/AGPL identifiers across `math1/`, `stat1/`, `time/`, and `advantage/` trees. Bare `Free42` excluded from the pattern because legitimate cross-check references exist.

### XROM Module Registry

*Full per-phase narrative: `docs/architecture-history.md` §v3.0–v3.3 additions.*

| Bit | XROM ID | CATALOG 2 Name | Source Tree | Ops | Since | ADRs |
|-----|---------|----------------|-------------|-----|-------|------|
| 0 | 7 | MATH 1 | `math1/` | 45 | v3.0 | v3.0-001..005 |
| 1 | 2 | STAT 1 | `stat1/` | 26 | v3.1 | v3.1-001..005 |
| 2 | 26 | TIME 2C | `time/` | 35 | v3.2 | v3.2-001..003 |
| 3 | 22 | ADV 22A | `advantage/` | 63 | v3.3 | v3.3-001..004 |
| 4 | 24 | ADV 24B | `advantage/` | 51 | v3.3 | v3.3-003 |

`default_xrom_modules() = 0b0001_1111`; `migrate_after_load()` in `state.rs` auto-upgrades older save files. `xrom_resolve` fires bits 0→1→2→3→4 in order; lower bit wins on overlap.

### v3.x Design Rules

These constrain future development. For the full narrative see `docs/architecture-history.md`.

- **XROM shadowing:** `xrom_shadowing.rs` verifies all module mnemonics are disjoint. **Exception:** 12 ADV_MATH_B (XROM 24) mnemonics intentionally overlap MATH_1 (E^Z, LNZ, LOGZ, Z^N, Z^1/N, Z^W, |Z|, SINZ, COSZ, TANZ, A^Z, CINV) — MATH_1 wins (bit-0 fires first). ADV_MATH_B disjointness test uses `0b0001_0000` bit-4 isolation mask to permit these overlaps (ADR-v3.3-003).
- **Named-matrix isolation (D-43.5):** Advantage Pac matrix ops use `adv_matrices: Vec<AdvMatrix>` — NEVER touch `state.matrix_dim` or `state.matrix_active_reg` (Math Pac I fields). ADR-v3.3-001.
- **FROOT:** Laguerre initial guess `(0.4, 0.9)`, NOT `(0, 0)` (origin singularity); quadratic deflation for complex conjugate pairs (ADR-v3.3-002).
- **Solver cross-nesting:** one level of cross-module nesting (FINTG⟷FSOLVE) reusing Phase 28 infrastructure; self-nesting blocked.
- **Time clock access:** `SystemTime::now()` called directly in hp41-core; `time_offset_secs: i64` stores delta (ADR-v3.2-001). Stopwatch frozen to Stopped on save; `migrate_after_load()` enforces.
- **Interrupting control alarms DEFERRED:** data model stores them but execution requires call-stack re-entrancy not yet supported. Documented in `docs/hp41-time-divergences.md`.
- **36-bit `ADV_WORD_MASK`:** bitwise operands silently truncated — hardware-faithful HP-41 base-N behavior.
- **`ModalProgram` enum pattern:** each module adds a variant (`Math1(Math1Step)`, `Stat1(Stat1Step)`, `Time(TimeStep)`, `Advantage(AdvantageStep)`); dispatch lives in the module's own `modal.rs`, outside the math1/ freeze.
- **Distribution primitives (v3.1):** 3 hand-coded f64-bridge functions in `stat1/distributions.rs` (~140 LOC); `statrs` rejected per ADR-v3.1-002.
- **OM register layout transcription:** `stat1/mod.rs` header + ~30 named consts (`STAT1_AOV_*_REG`, etc.); all register accesses through named consts (ADR-v3.1-003).

## Tech Stack

- **`just`** — sole task runner. **Never call `cargo` directly in CI or docs.** GUI recipes: `just gui-dev` / `just gui-build` / `just gui-ci` / `just gui-check`.
- Rust stable, MSRV 1.88
- `rust_decimal` 1.42, ratatui 0.30, crossterm 0.29, serde + serde_json, proptest, criterion (advisory, not CI-gated), clap 4.x
- Tauri v2.11, React 18 + TypeScript + Vite, `dirs`
- cargo-llvm-cov (coverage gate ≥ 95 % lines / ≥ 93 % regions on `hp41-core`, programmatically gated)
- WebdriverIO 9 + tauri-driver 2.0.6 (E2E smoke; Ubuntu-only via `ci-gui.yml::e2e-linux`)
- Two-layer CI: `ci.yml` (CLI + license-audit) + `ci-gui.yml` (3-OS matrix, path-filtered to `hp41-gui/**` and `hp41-core/**`)

## Quality Gates (current targets)

| Gate | Target | Current (v3.3) |
|------|--------|----------------|
| `hp41-core` coverage | ≥ 95 % lines / ≥ 93 % regions | 93.72 % / 95.63 % |
| Numerical accuracy | ≥ 98 % | 98.86 % (791+ cases) |
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
| `src/state.rs` | `CalcState` — single source of truth; `migrate_after_load()` |
| `src/stack.rs` | `Stack`, `apply_lift_effect()` |
| `src/ops/program.rs` | `run_program()`, `parse_counter()`, `execute_op()`, `builtin_card_op`, `xrom_resolve` |
| `src/ops/print.rs` | PRX / PRA / PRSTK — buffer-only |
| `src/ops/registers.rs` | STO arithmetic, M/N/O hidden registers |
| `src/ops/math1/` | Math Pac I (XROM 7), frozen since Plan 25-01 (3 sanctioned carve-outs) |
| `src/ops/stat1/` | Stat 1 Pac (XROM 2); `distributions.rs` (3 primitives), `modal.rs`, `random.rs` |
| `src/ops/time/` | Time Pac (XROM 26); `clock.rs` (JDN calendar), `alarm.rs`, `stopwatch.rs` |
| `src/ops/advantage/` | Advantage Pac (XROM 22+24); `matrix.rs`, `curve_fit.rs` (FROOT), `tvm.rs` |
| `src/format.rs` | `format_hpnum()`, `format_alpha()` |
| `tests/numerical_accuracy.rs` | 768+ oracle cases, ≥ 98 % gate |

**TUI (`hp41-cli`):**

| File | Purpose |
|------|---------|
| `src/app.rs` | `App`, `handle_key()`, `PendingInput`, event loop, `shift_armed`, help-overlay search |
| `src/keys.rs` | `key_to_op()`, `keycode_to_hp41_code()`, `xeq_by_name_local_resolve`, `key_ref_entries()` |
| `src/ui.rs` | `format_entry_buf_display()`, `pending_prompt()` exhaustive match, `render_right_panel` |
| `src/help_data.rs` | 5× `OnceLock<Vec<HelpEntry>>` loaded from JSON sources of truth + `help_entries_all()` |
| `src/prgm_display.rs` | `op_display_name()` — exhaustive match, NO `_ =>` catch-all |
| `src/persistence.rs` | `save_state()`, `load_state()` — JSON serde |

**GUI (`hp41-gui`):**

| File | Purpose |
|------|---------|
| `src-tauri/src/lib.rs` | `setup()`, `AppState = Mutex<CalcState>`, 30s auto-save thread |
| `src-tauri/src/commands.rs` | `dispatch_op`, `get_state`, `sst_step`, `bst_step`, `run_stop`, `request_cancel`, `tick_time` |
| `src-tauri/src/types.rs` | `CalcStateView`, `Annunciators`, `GuiError`, `From<HpError>` |
| `src-tauri/src/key_map.rs` | string ID → `Op` resolver (SC-4 boundary) |
| `src-tauri/src/persistence.rs` | Shared `~/.hp41/autosave.json` |
| `src-tauri/src/prgm_display.rs` | `op_display_name()` — second exhaustive match (4-way invariant) |
| `src-tauri/permissions/*.toml` | Tauri v2.11 inline-command permission registry |
| `src/App.tsx` | React root, `shiftActive`, `invokeForKey`, toast overlay, `tick_time` interval |
| `src/Keyboard.tsx` | 5×8 grid + top-row band, three-label `KeyDef` |
| `src/HelpOverlay.tsx` | `?` overlay with search + per-module XROM sections |
| `src/App.css` | Layout + key animation (`transform-box: fill-box`) |
| `wdio.conf.cjs` + `e2e/smoke.spec.js` | WebdriverIO + tauri-driver E2E smoke (Ubuntu) |
