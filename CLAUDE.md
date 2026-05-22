# HP-41 Calculator Emulator — Project Guide

## What this is

Faithful Rust behavioral emulation of the HP-41C/CV/CX RPN calculator.

- `hp41-core` — UI-agnostic library; zero CLI/UI deps (enforced at compile time)
- `hp41-cli` — TUI binary (ratatui 0.30 + crossterm 0.29)
- `hp41-gui` — Tauri v2 + React desktop app (nested standalone workspace)

**Current:** v3.0 Math Pac I (Owner's Manual 00041-90034 feature-complete), shipped 2026-05-21. Earlier tags: `v2.2` (HP-41CV complete, 2026-05-16), `v2.0` (Tauri GUI, 2026-05-10), `v1.1`, `v1.0`.

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
- **`hp41-core/src/ops/math1/` is frozen** since Plan 25-01. Math Pac I algorithms re-derived from HP OM 00041-90034 (1979); Free42 consulted as sanity-check oracle only, **not** copied. Every file in this directory carries the verbatim disclaim header.

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
