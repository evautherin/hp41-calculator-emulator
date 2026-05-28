# HP-41 Calculator Emulator — Project Guide

## What this is

Faithful Rust behavioral emulation of the HP-41C/CV/CX RPN calculator.

- `hp41-core` — UI-agnostic library; zero CLI/UI deps (enforced at compile time)
- `hp41-cli` — TUI binary (ratatui 0.30 + crossterm 0.29)
- `hp41-gui` — Tauri v2 + React desktop app (nested standalone workspace)

**Current:** v3.3 Advantage Pac, shipped 2026-05-26. Tags: `v3.2` (Time), `v3.1` (Stat 1), `v3.0` (Math I), `v2.2` (HP-41CV), `v2.0` (GUI), `v1.1`, `v1.0`.

**History/decisions:** `docs/architecture-history.md` (narrative), `docs/adr/` (17 ADRs), `docs/hp41-*-divergences.md` (OM divergences), `.planning/milestones/` (archived plans).

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

These are final. **Do not revisit without strong justification.**

### Workspace structure

- `hp41-core` must never depend on `hp41-cli` or `hp41-gui`. Root `Cargo.toml` members stay `["hp41-core", "hp41-cli"]`; `hp41-gui` is a nested standalone workspace. `tauri` / `tauri-build` appear ONLY in `hp41-gui/src-tauri/Cargo.toml`.
- **SC-4 (no core duplication in GUI):** stricter check `grep -rn "fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)" hp41-gui/src-tauri/src/` must return empty. Display helpers like `op_display_name` are exempt by intent (and duplicated CLI ↔ GUI by design).
- **MSRV 1.88** declared at `[workspace.package]`; member crates inherit via `rust-version.workspace = true`. CI MSRV job runs in parallel — no `needs:`.
- **Bundle ID:** `ch.talent-factory.hp41` (avoid macOS sandbox/keychain issues).

### Core engine

- **BCD/f64:** `rust_decimal` 1.42 with 10-significant-digit rounding. `HpNum` in `hp41-core/src/num.rs`. Custom BCD was evaluated and rejected.
- **Stack-lift:** every op declares `LiftEffect::Enable / Disable / Neutral`. The most commonly mis-implemented HP-41 feature — always check.
- **ISG/DSE counter:** extract fields by string-splitting at the decimal point — **never** `floor()`/`fmod()`. Same discipline applies to Time Pac date decimal parsing.
- **No async, no panics:** `#![deny(clippy::unwrap_used)]` at crate root. Production: `.expect("reason")` or `?`. Tests: `#[allow(clippy::unwrap_used)]`. GUI mutex: `.unwrap_or_else(|e| e.into_inner())`.
- **Print emulation:** `println!`/`eprintln!` forbidden in `hp41-core`. PRX/PRA/PRSTK push into `state.print_buffer`; CLI drains via `call_dispatch_and_drain()` / `drain_and_show_print_output()` — wire ALL `run_program()` call sites.
- **`hp41-core/src/ops/math1/` is frozen** since Plan 25-01. Algorithms from HP OM 00041-90034; Free42 as sanity-check oracle only, **not** copied. **3 sanctioned carve-outs:** `xrom.rs`, `modal.rs`, `complex.rs` (see file headers). All OTHER files frozen.
- **Zero new runtime deps** since v3.0. `statrs`/`libc`/`chrono` all rejected. Algorithms hand-coded from primary literature.

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

Five `docs/hp41-*-functions.json` files (~350 entries total) drive keybindings, `?` overlay, and right-panel. Loaded via `include_str!` + `OnceLock` in `help_data.rs`. Malformed JSON panics at first access.

- `just docs-matrix` regenerates function-matrix docs; `just docs-matrix-check` CI drift-catch.
- Op ↔ JSON parity: `function_matrix_parity.rs`; key coverage: `key_coverage.rs`.
- Right-panel: `key_ref_entries()` excludes XROM functions (`entry.xrom.is_none()`).

### CLI ↔ GUI parity (D-25.6)

- **One-shot SHIFT** is frontend-only. `App.shift_armed` (CLI) and `shiftActive` (GUI) mirror each other bit-for-bit. Neither ever appears in `CalcState` / `CalcStateView` / IPC.
- ALPHA overrides SHIFT (known divergence from real HP-41, accepted).
- IND-toggle (shift-0 inside an open Flag/Register modal) is hardware-faithful per HP-41C/CV QRG p.14; reuses the same `shift_armed` bit.

### GUI specifics

- **IPC contract:** `dispatch_op`, `get_state`, `sst_step`, `bst_step`, `run_stop`, `request_cancel`, `tick_time`, `submit_modal`, `cancel_modal`, `submit_modal_with_label` — Tauri v2 commands returning `CalcStateView`. Frontend never touches Rust enums; `key_map::resolve()` translates string IDs.
- **Tauri v2.11 permissions:** create TOML in `hp41-gui/src-tauri/permissions/<cmd-kebab>.toml`, reference in `capabilities/default.json`. Run `cargo check` first to generate the permission registry.
- **No polling (D-11):** frontend never polls `get_state()`. Clock/stopwatch: `tick_time` via `setInterval`.
- **Persistence sharing:** CLI + GUI share `~/.hp41/autosave.json`. Auto-save thread releases Mutex BEFORE disk I/O.

### Free42 GPL-contamination guard

CI-enforced via `just license-audit`. Greps for 18 distinctive Free42 / Intel BID / decNumber / GPL/AGPL identifiers across all module trees. Bare `Free42` excluded (legitimate cross-check references).

### XROM Module Registry

| Bit | XROM ID | Name | Source | Ops | Since |
|-----|---------|------|--------|-----|-------|
| 0 | 7 | MATH 1 | `math1/` | 45 | v3.0 |
| 1 | 2 | STAT 1 | `stat1/` | 26 | v3.1 |
| 2 | 26 | TIME 2C | `time/` | 35 | v3.2 |
| 3 | 22 | ADV 22A | `advantage/` | 63 | v3.3 |
| 4 | 24 | ADV 24B | `advantage/` | 51 | v3.3 |

`default_xrom_modules() = 0b0001_1111`; `migrate_after_load()` auto-upgrades older saves. `xrom_resolve` fires bits 0→4 in order; lower bit wins on overlap.

### X-MEM Built-in Integration (v4.0)

HP-41CX Extended Memory (X-MEM) functions are **HP-41CX OS built-ins, not an XROM module** — no XROM bit is allocated (ADR-v4.0-001). They resolve via `builtin_card_op` in `hp41-core/src/ops/program.rs` (XEQ-by-name only; no comfort keys). `default_xrom_modules()` stays `0b0001_1111` unchanged.

- **8 ops:** `EMDIR`, `EMROOM`, `SAVEP`, `GETP`, `SAVED`, `GETD`, `EMREG`, `SAVERX` — implementations in `hp41-core/src/ops/xmem/ops.rs`
- **State isolation:** `xmem_files: Vec<XmemFile>` + `xmem_active_file: Option<String>` on `CalcState` (both `#[serde(default)]`); NEVER touch `state.regs` directly except via SAVED/GETD transfer; NEVER touch `adv_matrices` (D-43.5 pattern, D-51.0a)
- **Capacity:** 600 registers (fully-expanded HP-41CX — 124 built-in + 2×238 X-Memory modules); `EMROOM` returns `600 - registers_used`; `SAVEP`/`SAVED` raise NO ROOM on overflow (ADR-v4.0-002)
- **SAVED/GETD:** full register set via `capture_data_card`/`load_data_card` (padded to `MIN_REGS_AFTER_LOAD = 100`); bbb.eee block control word deferred (ADR-v4.0-003)
- **Help pool:** dedicated `docs/hp41-xmem-functions.json` (8 entries, no `xrom` field — D-52.4); sixth pool in `help_entries_all()`; "Extended Memory" section in `?` overlay
- **Divergences:** `docs/hp41-xmem-divergences.md` (D-52-01: overwrite-on-duplicate; D-52-02: full-register-set SAVED/GETD)
- **ADRs:** `docs/adr/v4.0-001-xmem-os-builtin.md`, `v4.0-002-xmem-capacity.md`, `v4.0-003-xmem-register-transfer.md`
- **Save-file compat:** v1.0–v3.3 save files load without migration; `xmem_files`/`xmem_active_file` default cleanly via `#[serde(default)]`

### v3.x Design Rules

- **XROM shadowing:** `xrom_shadowing.rs` CI-gates mnemonic disjointness. **Exception:** 12 ADV_MATH_B overlaps with MATH_1 — MATH_1 wins (bit-0 first, ADR-v3.3-003).
- **Named-matrix isolation (D-43.5):** Advantage Pac uses `adv_matrices` — NEVER touch `state.matrix_dim` / `state.matrix_active_reg` (Math Pac I fields).
- **Solver cross-nesting:** one level FINTG⟷FSOLVE; self-nesting blocked.
- **Time clock:** `SystemTime::now()` in hp41-core; `time_offset_secs: i64` stores delta. Stopwatch frozen on save.
- **Interrupting control alarms DEFERRED:** data model stores them, execution requires call-stack re-entrancy (not yet supported).
- **`ModalProgram` enum:** each module adds a variant; dispatch in module's `modal.rs`, outside math1/ freeze.

## Tech Stack

- **`just`** — sole task runner. **Never call `cargo` directly in CI or docs.** GUI: `just gui-dev` / `just gui-build` / `just gui-ci`.
- Rust stable (MSRV 1.88), `rust_decimal` 1.42, ratatui 0.30, serde, proptest, clap 4.x
- Tauri v2.11, React 18 + TypeScript + Vite
- Two-layer CI: `ci.yml` (CLI + license-audit) + `ci-gui.yml` (3-OS matrix + E2E smoke)

## Quality Gates

| Gate | Target |
|------|--------|
| `hp41-core` coverage | ≥ 95 % lines / ≥ 93 % regions |
| Numerical accuracy | ≥ 98 % (`tests/numerical_accuracy.rs`) |
| Panics in `hp41-core` | 0 |
| Free42 contamination | 0 (CI-gated via `just license-audit`) |
| CI | Win 10+ / macOS 12+ / Ubuntu 22.04+ |

## Key Entry Points

- **Core hub:** `hp41-core/src/ops/mod.rs` (`Op` enum + `dispatch()`), `src/state.rs` (`CalcState` + `migrate_after_load()`)
- **CLI keyboard:** `hp41-cli/src/app.rs` (`handle_key`, `PendingInput`), `src/keys.rs` (`key_to_op`, `xeq_by_name_local_resolve`)
- **GUI IPC:** `hp41-gui/src-tauri/src/commands.rs` (Tauri commands), `src/key_map.rs` (string ID → Op resolver)
- **XROM resolver:** `hp41-core/src/ops/math1/xrom.rs` (`xrom_resolve()` + module registries)
