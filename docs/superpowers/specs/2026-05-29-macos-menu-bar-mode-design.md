# Design: macOS Menu-Bar Mode for the HP-41 GUI

**Date:** 2026-05-29
**Status:** Approved (brainstorming) — pending implementation plan
**Scope:** `hp41-gui` only. No `hp41-core` changes. macOS-only behavior.

## Problem

A user asked how to turn the macOS `.app` into a desktop widget. True macOS desktop
widgets require WidgetKit + SwiftUI in an Xcode project and are effectively
non-interactive (glanceable, App-Intent-only). A ~40-key interactive RPN calculator
cannot work as a WidgetKit widget. The closest "always at hand on the desktop"
experience that reuses the entire existing React UI and `hp41-core` backend is a
**macOS menu-bar (status-item) app with a toggling popover**.

## Goals

- On macOS, run as a pure menu-bar app: **no Dock icon**, **no window on launch**,
  only a menu-bar icon.
- Left-click toggles a borderless popover (the existing calculator UI) positioned
  directly under the icon; focus loss hides it.
- Right-click opens a menu: **About · Start at Login (checkmark) · Quit**.
- Reuse the existing React frontend and IPC contract unchanged.
- Windows/Linux keep today's exact windowed behavior.

## Non-Goals (YAGNI)

- Global hotkey to summon the popover.
- A "pin" mode where the popover stays open on blur.
- A separate compact popover layout (we scale the existing layout instead).
- Windows/Linux tray support (macOS-only via `cfg(target_os = "macos")`).
- Any WidgetKit / native Swift widget.

## Decisions (from brainstorming)

1. **Operating mode:** Pure menu-bar app (Accessory activation policy, no Dock icon,
   no classic window).
2. **Interaction:** Classic tray popover — left-click toggles a borderless,
   always-on-top popover under the icon; blur hides it; right-click opens a menu.
3. **Size handling:** Fit-to-screen + scale. Cap popover height at ~92% of screen
   height and proportionally CSS-scale the existing 440×1020 layout so everything
   fits without scrolling. No UI rebuild.
4. **Platform:** macOS only. Windows/Linux unchanged.
5. **Autostart:** Included in this version via a "Start at Login" menu item.

## Architecture

### Backend (`hp41-gui/src-tauri`)

**New module `tray.rs` (gated with `#[cfg(target_os = "macos")]`):**

- Build the status item with `TrayIconBuilder`:
  - Template icon via `icon_as_template(true)` so it adapts to light/dark menu bar.
  - `show_menu_on_left_click(false)` so left-click is free for the popover toggle.
- **Left-click** (`TrayIconEvent::Click` with left button, up state):
  - Toggle the popover. Compute position from the event's icon `rect` (top-center
    under the icon), call `show()` + `set_focus()`, or `hide()` if already visible.
- **Right-click:** show a `Menu` built from `MenuItem`/`CheckMenuItem`:
  - **Quit** → `app.exit(0)`.
  - **Start at Login** (`CheckMenuItem`) → toggle `app.autolaunch().enable()` /
    `.disable()`; keep the checkmark in sync with the current state.
  - **About** → small info window or system dialog (version from
    `env!("CARGO_PKG_VERSION")`).

**`lib.rs` `setup()` changes (macOS-gated):**

- `app.set_activation_policy(tauri::ActivationPolicy::Accessory)` to drop the Dock icon.
- Initialize the tray (call into `tray.rs`).
- Register the autostart plugin:
  `tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None)`.

**Auto-hide on blur:**

- Handle `WindowEvent::Focused(false)` → hide the window.
- **Edge cases the plan must solve:**
  - (a) Flicker guard: a left-click while the window is open must not race
    "blur-hide → immediately re-show". Track visibility / use a short debounce so a
    toggle that hides doesn't get undone by the same click.
  - (b) Suppress auto-hide while a **native file dialog** (`.raw`/`.card.json`
    import/export) or an **in-app modal** is open — otherwise the popover vanishes
    mid-entry. The frontend already tracks modal state; the dialog commands are in
    `commands.rs`. The plan decides the exact suppression mechanism (e.g. a shared
    flag set around dialog calls and while a modal is active).

### Frontend (scaling — client-side only)

- Add a **scaling wrapper outside `<App/>`** in `main.tsx` (keeps `App.test.tsx`
  untouched):
  - Fixed design size 440×1020.
  - On `resize`/via `ResizeObserver`, compute
    `scale = min(1, window.innerHeight / 1020, window.innerWidth / 440)`.
  - Apply `transform: scale(scale)` with `transform-origin: top center` to a
    440×1020 container holding `<App/>`.
- **No new IPC commands. No change to the component tree or `App.tsx`.**

### Configuration & Assets

- `tauri.conf.json` main window:
  - `decorations: false`, `alwaysOnTop: true`, `visible: false` (start hidden).
  - Height is set dynamically when showing the popover to
    `min(1020, ~0.92 * screen height)`.
- New **template tray icon** (monochrome, ~22px + @2x) under `src-tauri/icons/`.
- `Cargo.toml`: add `tauri-plugin-autostart = "2"`.
- **No new permission TOML required** — tray, menu, and autostart are driven from
  Rust; capabilities only gate frontend `invoke` calls, which are unchanged.

## Testing & Risks

- **E2E smoke (`ci-gui.yml`, WebdriverIO):** `visible: false` may break an E2E that
  expects a window on launch. Mitigation: an env flag (e.g. `HP41_SHOW_ON_START`)
  that, when set, shows the window at startup in `setup()`; the E2E sets it. Exact
  approach finalized in the plan.
- Existing Vitest + Rust tests stay green (additive wrapper; no core/IPC change).
- **Frozen Invariants respected:**
  - SC-4 (no core duplication in GUI) — untouched; no core ops added.
  - Bundle ID `ch.talent-factory.hp41` — unchanged.
  - IPC contract — unchanged.
  - `hp41-core` — not modified.

## Open Questions for the Plan

- Exact flicker-guard mechanism for left-click vs. blur-hide.
- Exact auto-hide suppression wiring for native dialogs + in-app modals.
- Whether About is a window or a system dialog.
- Final shape of the `HP41_SHOW_ON_START` (or equivalent) E2E escape hatch.
