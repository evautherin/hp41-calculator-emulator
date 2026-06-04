# Design: Switchable macOS Launch Mode (App ↔ Menu Bar)

**Date:** 2026-05-29
**Branch:** `feat/macos-menu-bar-mode`
**Status:** Approved (brainstorming) — pending implementation plan

## Problem

Since the menu-bar work landed (ADR-v4.1-001), a macOS end user gets **only** the
menu-bar (status-item accessory) experience. The classic decorated window is no longer
reachable on the Mac except through the `HP41_SHOW_ON_START` test backdoor. Users should
be able to choose between the two runtime modes.

This is a single binary / single `.app` with two macOS runtime modes — **not** a second
build artifact. The choice is made at startup based on a persisted preference.

## Decisions (from brainstorming)

| # | Decision | Rationale |
|---|----------|-----------|
| D-1 | **Restart-based switch** — a mode change takes effect on next launch | Keeps the decision in `setup()`; avoids live tear-down/rebuild of the activation policy + window |
| D-2 | **Default = menu-bar**; existing users unchanged | Consistent with the "pure menu-bar app" decision; `#[serde(default)]` makes old `prefs.json` load as menu-bar |
| D-3 | **Offer automatic restart** after toggling | Preference saved first, then `app.restart()` via a "Restart now" button — cleanest UX |
| D-4 | Setting lives in `SettingsPanel`, macOS-only | Natural home next to Theme/Quick Start; reachable from the popover in menu-bar mode and from the window in app mode |
| D-5 | `HP41_SHOW_ON_START` keeps highest precedence | E2E test backdoor must remain unaffected |

## Architecture

### 1. Data model — `hp41-gui/src-tauri/src/prefs.rs`

Add one field to `GuiPrefs`, mirroring the existing `theme` pattern exactly:

```rust
#[serde(default = "default_launch_mode")]
pub macos_launch_mode: String,   // "menu-bar" | "window"

fn default_launch_mode() -> String { "menu-bar".to_string() }
```

- `pub const VALID_LAUNCH_MODES: &[&str] = &["menu-bar", "window"];`
- In `load_prefs`, after the existing theme validation, clamp an unknown
  `macos_launch_mode` back to `"menu-bar"` (same defensive pattern as `theme`).
- `GuiPrefs::default()` sets `macos_launch_mode: default_launch_mode()`.
- `#[serde(default)]` guarantees an old `prefs.json` (no field) loads as `"menu-bar"`
  → existing users stay in menu-bar mode (D-2).

### 2. Backend branch — `hp41-gui/src-tauri/src/lib.rs` (`setup`)

The `#[cfg(target_os = "macos")]` block now reads `initial_prefs.macos_launch_mode`.
`initial_prefs` is already loaded earlier in `setup()` (line ~57) and managed in state, so
read the value before it is moved into the `Mutex`, or re-read from the managed `PrefsState`.

Precedence (highest first):

1. `HP41_SHOW_ON_START` env var present → show normal window (E2E backdoor — unchanged, D-5)
2. `macos_launch_mode == "window"` → show normal window
3. otherwise → `crate::tray::apply_menu_bar_mode(app)` (existing fallback-to-window on error stays)

### 3. IPC — `hp41-gui/src-tauri/src/commands.rs`

- `set_pref`: add a `"macos_launch_mode"` arm that validates against `VALID_LAUNCH_MODES`
  and returns `Err(format!("unknown launch mode: {value}"))` on a bad value, then persists
  via `save_prefs` (same shape as the `"theme"` arm).
- New command `restart_app(app: AppHandle)` → calls Tauri v2 native `app.restart()`
  (no `tauri-plugin-process` dependency). `restart()` is `-> !`; calling it as the final
  expression of a `-> ()` command body is fine.
- New command `is_macos() -> bool` returning `cfg!(target_os = "macos")` — lets the frontend
  render the launch-mode control only on macOS (on Win/Linux the preference has no effect,
  so showing it would be misleading).
- Register both new commands in the `invoke_handler` generate list.
- Tauri v2 permissions (project rule): create
  `permissions/restart-app.toml` and `permissions/is-macos.toml`, reference both in
  `capabilities/default.json`. Run `cargo check` first to regenerate the permission registry.

### 4. Frontend — `hp41-gui/src/SettingsPanel.tsx` + `App.tsx`

`SettingsPanel`:
- New props: `isMacos: boolean`, `currentLaunchMode: string`,
  `onLaunchModeChange: (mode: string) => void`.
- A new `<section>` **"Launch Mode (macOS)"**, rendered only when `isMacos === true`,
  placed under the Quick Start section. Two radio rows using the existing
  `settings-radio-row` class: `menu-bar` ("Menu Bar") and `window` ("Window").
- After a change, show an inline confirmation row inside the panel with a
  **"Restart now"** button → `invoke("restart_app")`. No separate dialog/popover.

`App.tsx`:
- On startup, call `invoke("is_macos")` once and store `isMacos`; `get_prefs` is already
  called on startup — extend its handling to keep `macosLaunchMode` in state.
- `onLaunchModeChange` calls `invoke("set_pref", { key: "macos_launch_mode", value })`,
  updates local state, and reveals the "Restart now" affordance.
- Pass the three new props into `<SettingsPanel>`.

On non-macOS, the panel is visually unchanged (the section is not rendered).

### 5. Error handling

- Invalid `macos_launch_mode` from `set_pref` → `Err(String)` surfaced to the frontend
  (same as invalid theme today).
- Corrupt/unknown value already in `prefs.json` → clamped to `"menu-bar"` on load.
- `apply_menu_bar_mode` failure path (existing) still falls back to showing the window.

## Testing

- **`prefs.rs`** (Rust): roundtrip of `macos_launch_mode`; serde-default backward compat
  (old JSON without the field → `"menu-bar"`); unknown value → fallback to `"menu-bar"`.
  Mirrors the existing theme tests.
- **`commands.rs`**: `set_pref("macos_launch_mode", …)` accepts `"menu-bar"`/`"window"`,
  rejects an invalid value with `Err`.
- **`SettingsPanel.test.tsx`**: section visible when `isMacos=true`, hidden when `false`;
  clicking "Window" fires `onLaunchModeChange`; the "Restart now" button appears after a
  change.
- `restart_app` / `is_macos` are trivial wrappers — no dedicated unit test; covered by
  integration.

## Documentation

- Extend ADR-v4.1-001 with a "Switchable via preference" section (default menu-bar,
  restart-based, env var remains the test override).
- Update the CLAUDE.md GUI-specifics block: launch mode is now user-selectable
  (default menu-bar); `HP41_SHOW_ON_START` stays the highest-precedence test override.

## Out of scope (YAGNI)

- Live mode switching without a restart.
- A quick-toggle entry in the tray right-click menu (the Settings panel is sufficient;
  menu-bar users reach it via the popover).
- Any change to Windows/Linux behavior (they keep the normal decorated window).
- Touching `hp41-core` or the IPC `CalcState`/`CalcStateView` contract (none required).

## Affected files

| File | Change |
|------|--------|
| `hp41-gui/src-tauri/src/prefs.rs` | new field + validation + tests |
| `hp41-gui/src-tauri/src/lib.rs` | read preference in macOS setup branch |
| `hp41-gui/src-tauri/src/commands.rs` | `set_pref` arm + `restart_app` + `is_macos` + tests |
| `hp41-gui/src-tauri/permissions/restart-app.toml` | new |
| `hp41-gui/src-tauri/permissions/is-macos.toml` | new |
| `hp41-gui/src-tauri/capabilities/default.json` | reference new permissions |
| `hp41-gui/src/SettingsPanel.tsx` | launch-mode section |
| `hp41-gui/src/App.tsx` | wiring + restart affordance |
| `hp41-gui/src/SettingsPanel.test.tsx` | tests |
| `docs/adr/v4.1-001-macos-menu-bar-mode.md` | switchable section |
| `CLAUDE.md` | GUI specifics update |
