# Switchable macOS Launch Mode Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let a macOS user choose at startup between the menu-bar (status-item accessory) mode and a normal decorated window, via a persisted preference, with an offered automatic restart.

**Architecture:** Add one `#[serde(default)]` field `macos_launch_mode` to `GuiPrefs` (default `"menu-bar"`). The macOS `setup()` branch in `lib.rs` reads it to decide between `apply_menu_bar_mode` and showing the window; `HP41_SHOW_ON_START` keeps highest precedence. The frontend gets a macOS-only radio control in `SettingsPanel` that persists the choice via the existing `set_pref` IPC and offers a `restart_app` command (native Tauri v2 `app.restart()`).

**Tech Stack:** Rust + Tauri v2.11, React 18 + TypeScript + Vitest, `rust_decimal`/serde. Task runner: `just`. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-05-29-macos-launch-mode-toggle-design.md`

---

## File Structure

| File | Responsibility | Action |
|------|----------------|--------|
| `hp41-gui/src-tauri/src/prefs.rs` | persisted pref struct + validation | Modify |
| `hp41-gui/src-tauri/src/commands.rs` | `set_pref` arm, `restart_app`, `is_macos` | Modify |
| `hp41-gui/src-tauri/src/lib.rs` | read pref in macOS setup branch; register commands | Modify |
| `hp41-gui/src-tauri/permissions/restart-app.toml` | command permission | Create |
| `hp41-gui/src-tauri/permissions/is-macos.toml` | command permission | Create |
| `hp41-gui/src-tauri/capabilities/default.json` | reference new permissions | Modify |
| `hp41-gui/src/SettingsPanel.tsx` | macOS launch-mode radio section | Modify |
| `hp41-gui/src/SettingsPanel.test.tsx` | section tests | Modify |
| `hp41-gui/src/App.tsx` | wiring + restart affordance | Modify |
| `docs/adr/v4.1-001-macos-menu-bar-mode.md` | "switchable via preference" section | Modify |
| `CLAUDE.md` | GUI specifics update | Modify |

**Conventions for every commit:** use `/git-workflow:commit --with-skills`, English message, Emoji Conventional format, NO `Co-Authored-By` trailer (repo convention). Stage only the files named in the step — never the unrelated `Justfile` change in the working tree.

**Test commands (match Justfile recipes):**
- Targeted Rust: `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml <name>`
- Targeted frontend: `cd hp41-gui && npm test -- <pattern>` (Vitest, single run)
- Permission gate: `bash scripts/check-tauri-permissions.sh`
- Full gate: `just gui-ci`

---

## Task 1: Add `macos_launch_mode` to GuiPrefs

**Files:**
- Modify: `hp41-gui/src-tauri/src/prefs.rs`
- Test: `hp41-gui/src-tauri/src/prefs.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Write the failing tests**

Add these three tests inside the existing `mod tests` block in `prefs.rs` (after `test_unknown_theme_falls_back_to_default`):

```rust
    #[test]
    fn test_launch_mode_roundtrip() {
        let path = temp_path("prefs_launch_mode_roundtrip");
        let prefs = GuiPrefs {
            theme: "dark".to_string(),
            onboarding_done: false,
            macos_launch_mode: "window".to_string(),
        };
        save_prefs(&path, &prefs).unwrap();
        let loaded = load_prefs(&path);
        assert_eq!(
            loaded.macos_launch_mode, "window",
            "roundtrip must preserve macos_launch_mode"
        );
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    /// Backward compat: a prefs.json without `macos_launch_mode` (written before this
    /// feature) must load with the default "menu-bar" via #[serde(default)]. This is the
    /// load-bearing guarantee that existing macOS users stay in menu-bar mode (D-2).
    #[test]
    fn test_launch_mode_serde_default() {
        let path = temp_path("prefs_launch_mode_default");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, br#"{"theme":"dark","onboarding_done":true}"#).unwrap();
        let loaded = load_prefs(&path);
        assert_eq!(
            loaded.macos_launch_mode, "menu-bar",
            "missing macos_launch_mode must default to 'menu-bar' (backward compat)"
        );
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn test_unknown_launch_mode_falls_back_to_default() {
        let path = temp_path("prefs_unknown_launch_mode");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            br#"{"theme":"dark","onboarding_done":false,"macos_launch_mode":"hologram"}"#,
        )
        .unwrap();
        let loaded = load_prefs(&path);
        assert_eq!(
            loaded.macos_launch_mode, "menu-bar",
            "unknown macos_launch_mode must fall back to 'menu-bar'"
        );
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }
```

Also update the THREE existing tests that construct `GuiPrefs { .. }` literally (`test_roundtrip`, `test_unknown_theme_falls_back_to_default`, `test_onboarding_done_roundtrip`) by adding `macos_launch_mode: "menu-bar".to_string(),` to each struct literal — otherwise they will not compile after Step 3.

- [ ] **Step 2: Run tests to verify they fail (compile error)**

Run: `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml prefs::tests::test_launch_mode`
Expected: FAIL — compile error, `GuiPrefs` has no field `macos_launch_mode`.

- [ ] **Step 3: Implement the field, default, and validation**

In `prefs.rs`, add the field to the struct (after `onboarding_done`):

```rust
    #[serde(default)]
    pub onboarding_done: bool,
    /// macOS-only launch mode: "menu-bar" (status-item accessory) or "window"
    /// (normal decorated window). Ignored on Windows/Linux. Defaults to "menu-bar"
    /// so existing prefs.json files (no field) keep the current menu-bar behavior.
    #[serde(default = "default_launch_mode")]
    pub macos_launch_mode: String,
```

Add the default helper next to `default_theme`:

```rust
/// Default launch mode — "menu-bar" (the established macOS behavior, ADR-v4.1-001).
fn default_launch_mode() -> String {
    "menu-bar".to_string()
}
```

Add the valid-values constant next to `VALID_THEMES`:

```rust
pub const VALID_LAUNCH_MODES: &[&str] = &["menu-bar", "window"];
```

Update `impl Default for GuiPrefs`:

```rust
impl Default for GuiPrefs {
    fn default() -> Self {
        GuiPrefs {
            theme: default_theme(),
            onboarding_done: false,
            macos_launch_mode: default_launch_mode(),
        }
    }
}
```

Add the clamp in `load_prefs`, right after the existing theme clamp:

```rust
    if !VALID_THEMES.contains(&prefs.theme.as_str()) {
        prefs.theme = default_theme();
    }
    if !VALID_LAUNCH_MODES.contains(&prefs.macos_launch_mode.as_str()) {
        prefs.macos_launch_mode = default_launch_mode();
    }
    prefs
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml prefs::`
Expected: PASS — all prefs tests green (old + 3 new).

- [ ] **Step 5: Commit**

```bash
git add hp41-gui/src-tauri/src/prefs.rs
git commit  # via /git-workflow:commit --with-skills, English message
# 💎 feat(gui): add macos_launch_mode preference with serde-default fallback
```

---

## Task 2: Accept `macos_launch_mode` in `set_pref`

**Files:**
- Modify: `hp41-gui/src-tauri/src/commands.rs:493-515` (`set_pref`)
- Test: `hp41-gui/src-tauri/src/commands.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Write the failing tests**

`set_pref` takes `State<'_, PrefsState>`, which is hard to construct in a unit test. Test the **validation rule** directly against `VALID_LAUNCH_MODES` (the value the command branch checks), mirroring how the codebase keeps command thunks thin. Add to the `#[cfg(test)] mod tests` block in `commands.rs` (create the block if none exists, following the file's existing test style):

```rust
    use crate::prefs::VALID_LAUNCH_MODES;

    #[test]
    fn test_valid_launch_modes_accepts_known() {
        assert!(VALID_LAUNCH_MODES.contains(&"menu-bar"));
        assert!(VALID_LAUNCH_MODES.contains(&"window"));
    }

    #[test]
    fn test_valid_launch_modes_rejects_unknown() {
        assert!(!VALID_LAUNCH_MODES.contains(&"hologram"));
        assert!(!VALID_LAUNCH_MODES.contains(&""));
    }
```

> Note: if `commands.rs` already has a `#[cfg(test)] mod tests` with a `use super::*;`, add only the two `#[test]` fns plus the `use crate::prefs::VALID_LAUNCH_MODES;` line inside it.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml commands::tests::test_valid_launch_modes`
Expected: FAIL — `VALID_LAUNCH_MODES` not found (Task 1 added it to `prefs`, but this confirms the import path) OR, if Task 1 is merged, PASS for the constant but the `set_pref` arm still missing. If both pass immediately, that is acceptable — the behavioral coverage is the `set_pref` arm added in Step 3; proceed.

- [ ] **Step 3: Add the `macos_launch_mode` arm to `set_pref`**

Update the import on line 21 to include the new constant:

```rust
use crate::prefs::{default_prefs_path, save_prefs, GuiPrefs, VALID_LAUNCH_MODES, VALID_THEMES};
```

Add a new arm in the `match key.as_str()` block in `set_pref`, after the `"onboarding_done"` arm:

```rust
        "macos_launch_mode" => {
            if !VALID_LAUNCH_MODES.contains(&value.as_str()) {
                return Err(format!("unknown launch mode: {value}"));
            }
            p.macos_launch_mode = value;
        }
```

Also extend the doc comment `# Supported keys` list above the function with:

```rust
/// - `"macos_launch_mode"`: one of `"menu-bar"` | `"window"` (macOS-only effect).
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml commands::`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add hp41-gui/src-tauri/src/commands.rs
git commit  # ✨ feat(gui): accept macos_launch_mode key in set_pref
```

---

## Task 3: Add `restart_app` and `is_macos` commands + permissions

**Files:**
- Modify: `hp41-gui/src-tauri/src/commands.rs` (add two commands)
- Modify: `hp41-gui/src-tauri/src/lib.rs:136-155` (register in `generate_handler!`)
- Create: `hp41-gui/src-tauri/permissions/restart-app.toml`
- Create: `hp41-gui/src-tauri/permissions/is-macos.toml`
- Modify: `hp41-gui/src-tauri/capabilities/default.json`

- [ ] **Step 1: Add the two commands to `commands.rs`**

Append after `set_pref` (no test needed — trivial wrappers, covered by the permission gate + integration):

```rust
/// Tauri command: restart the application.
///
/// Used by the Settings panel after the user changes `macos_launch_mode` — the new
/// launch mode is decided in `setup()` (lib.rs) and only takes effect on the next
/// launch, so we offer an immediate relaunch. `AppHandle::restart()` is `-> !` and
/// never returns; the IPC promise on the frontend therefore never resolves, which is
/// correct (the process is replaced).
#[tauri::command]
pub fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

/// Tauri command: report whether the backend was compiled for macOS.
///
/// The frontend uses this to render the macOS-only launch-mode control. The launch-mode
/// preference has no effect on Windows/Linux (the `setup()` branch is `cfg(macos)`), so
/// showing the control there would mislead the user.
#[tauri::command]
pub fn is_macos() -> bool {
    cfg!(target_os = "macos")
}
```

- [ ] **Step 2: Register both commands in `lib.rs`**

In the `tauri::generate_handler![ ... ]` list (lib.rs ~136-155), add after `commands::set_pref,`:

```rust
            commands::set_pref,                // Phase 48 INFRA-02 — write/persist a GUI preference
            commands::restart_app,             // macOS launch-mode toggle — offer relaunch after switch
            commands::is_macos,                // macOS launch-mode toggle — gate the Settings control
```

- [ ] **Step 3: Create the permission TOMLs**

Create `hp41-gui/src-tauri/permissions/restart-app.toml`:

```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-restart-app"
description = "Allows the restart_app command."
commands.allow = ["restart_app"]
```

Create `hp41-gui/src-tauri/permissions/is-macos.toml`:

```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-is-macos"
description = "Allows the is_macos command."
commands.allow = ["is_macos"]
```

- [ ] **Step 4: Reference the permissions in `capabilities/default.json`**

Add two entries to the `"permissions"` array, after `"allow-set-pref",`:

```json
    "allow-set-pref",
    "allow-restart-app",
    "allow-is-macos",
    "allow-save-state",
```

- [ ] **Step 5: Regenerate the permission registry and verify the gate**

Run: `cargo check --manifest-path hp41-gui/src-tauri/Cargo.toml`
Expected: compiles; this regenerates `gen/schemas`.

Run: `bash scripts/check-tauri-permissions.sh`
Expected: PASS — every command has a matching permission (no missing TOML).

- [ ] **Step 6: Run the Rust tests**

Run: `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add hp41-gui/src-tauri/src/commands.rs hp41-gui/src-tauri/src/lib.rs \
        hp41-gui/src-tauri/permissions/restart-app.toml \
        hp41-gui/src-tauri/permissions/is-macos.toml \
        hp41-gui/src-tauri/capabilities/default.json
git commit  # ✨ feat(gui): add restart_app + is_macos commands with permissions
```

---

## Task 4: Read the preference in the macOS setup branch

**Files:**
- Modify: `hp41-gui/src-tauri/src/lib.rs:56-58` (capture mode) and `:106-123` (branch)

- [ ] **Step 1: Capture the launch mode before `initial_prefs` is moved**

`initial_prefs` is moved into the managed `Mutex` at line 58. Capture the mode first. Change:

```rust
            let prefs_path = prefs::default_prefs_path();
            let initial_prefs = prefs::load_prefs(&prefs_path);
            app.manage(Mutex::new(initial_prefs));
```

to:

```rust
            let prefs_path = prefs::default_prefs_path();
            let initial_prefs = prefs::load_prefs(&prefs_path);
            // Capture the macOS launch mode before initial_prefs is moved into the
            // managed Mutex — used by the macOS setup branch below. Unused on non-macOS.
            #[cfg(target_os = "macos")]
            let macos_launch_mode = initial_prefs.macos_launch_mode.clone();
            app.manage(Mutex::new(initial_prefs));
```

- [ ] **Step 2: Branch on the captured mode**

Replace the macOS block (currently lib.rs ~109-123) with:

```rust
            #[cfg(target_os = "macos")]
            {
                app.manage(crate::tray::PopoverState::default());
                // Precedence: HP41_SHOW_ON_START (E2E backdoor) > "window" pref > menu-bar.
                if std::env::var_os("HP41_SHOW_ON_START").is_some()
                    || macos_launch_mode == "window"
                {
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.show();
                        let _ = win.set_focus();
                    }
                } else if let Err(e) = crate::tray::apply_menu_bar_mode(app) {
                    eprintln!("hp41-gui: failed to enter menu-bar mode: {e}; showing window");
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.show();
                    }
                }
            }
```

- [ ] **Step 3: Verify it compiles and tests pass**

Run: `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml`
Expected: PASS, no warnings about unused `macos_launch_mode`.

- [ ] **Step 4: Manual smoke (macOS only — note for the human reviewer)**

This is the one step not covered by automated tests (matches the project's note that popover positioning/mode behavior is manually verified). To be run by the human at review time:
1. Ensure `~/.hp41/prefs.json` has no `macos_launch_mode` (or `"menu-bar"`) → `just gui-dev` → menu-bar icon, no window. (existing behavior preserved)
2. Set `"macos_launch_mode": "window"` in `~/.hp41/prefs.json` → `just gui-dev` → normal window, still no Dock icon change needed for this path.
3. `HP41_SHOW_ON_START=1 just gui-dev` with `"window"` pref → window (env override consistent).

- [ ] **Step 5: Commit**

```bash
git add hp41-gui/src-tauri/src/lib.rs
git commit  # ✨ feat(gui): honor macos_launch_mode preference at startup
```

---

## Task 5: Launch-mode section in SettingsPanel

**Files:**
- Modify: `hp41-gui/src/SettingsPanel.tsx`
- Test: `hp41-gui/src/SettingsPanel.test.tsx`

- [ ] **Step 1: Write the failing tests**

Add to `SettingsPanel.test.tsx` inside the `describe('SettingsPanel', ...)` block. Note: all existing `render(<SettingsPanel .../>)` calls must gain the three new props or TypeScript fails — update each existing render in this file to add `isMacos={false}`, `currentLaunchMode="menu-bar"`, and `onLaunchModeChange={vi.fn()}`. Then add:

```tsx
    it('hides launch-mode section when isMacos=false', () => {
        const { queryByText } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
                isMacos={false}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={vi.fn()}
            />
        );
        expect(queryByText('Launch Mode (macOS)')).toBeNull();
    });

    it('shows launch-mode radios when isMacos=true', () => {
        const { getByText, getByLabelText } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
                isMacos={true}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={vi.fn()}
            />
        );
        expect(getByText('Launch Mode (macOS)')).toBeTruthy();
        expect(getByLabelText('Menu Bar')).toBeTruthy();
        expect(getByLabelText('Window')).toBeTruthy();
    });

    it('calls onLaunchModeChange when Window is selected', () => {
        const onLaunchModeChange = vi.fn();
        const { getByLabelText } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
                isMacos={true}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={onLaunchModeChange}
            />
        );
        fireEvent.click(getByLabelText('Window'));
        expect(onLaunchModeChange).toHaveBeenCalledWith('window');
    });

    it('shows restart hint after a launch-mode change', () => {
        const { getByLabelText, getByText } = render(
            <SettingsPanel
                open={true}
                onClose={() => {}}
                currentTheme="dark"
                onThemeChange={() => {}}
                onShowOnboarding={vi.fn()}
                isMacos={true}
                currentLaunchMode="menu-bar"
                onLaunchModeChange={vi.fn()}
            />
        );
        fireEvent.click(getByLabelText('Window'));
        expect(getByText('Restart now')).toBeTruthy();
    });
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd hp41-gui && npm test -- SettingsPanel`
Expected: FAIL — props don't exist / "Launch Mode (macOS)" not found.

- [ ] **Step 3: Implement the section**

Edit `SettingsPanel.tsx`. Add to imports (top of file):

```tsx
import { useRef, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
```

Extend `SettingsPanelProps`:

```tsx
export type SettingsPanelProps = {
    open: boolean;
    onClose: () => void;
    currentTheme: string;
    onThemeChange: (theme: string) => void;
    onShowOnboarding: () => void;
    isMacos: boolean;
    currentLaunchMode: string;            // "menu-bar" | "window"
    onLaunchModeChange: (mode: string) => void;
};
```

Add a constant next to `THEMES`:

```tsx
const LAUNCH_MODES = [
    { id: 'menu-bar', label: 'Menu Bar' },
    { id: 'window', label: 'Window' },
] as const;
```

Update the function signature destructuring to include the new props, and add a local
state for the "changed → show restart" affordance:

```tsx
export function SettingsPanel({
    open, onClose, currentTheme, onThemeChange, onShowOnboarding,
    isMacos, currentLaunchMode, onLaunchModeChange,
}: SettingsPanelProps) {
    const panelRef = useRef<HTMLDivElement>(null);
    const [launchModeChanged, setLaunchModeChanged] = useState(false);
```

Add the section JSX after the Quick Start `<section>` (before the closing `</div>`):

```tsx
            {isMacos && (
                <>
                    <hr className="settings-section-divider" />
                    <section className="settings-section">
                        <h3 className="settings-section-heading">Launch Mode (macOS)</h3>
                        {LAUNCH_MODES.map(m => (
                            <label key={m.id} className="settings-radio-row">
                                <input
                                    type="radio"
                                    name="launch-mode"
                                    value={m.id}
                                    checked={currentLaunchMode === m.id}
                                    onChange={() => { onLaunchModeChange(m.id); setLaunchModeChanged(true); }}
                                />
                                {m.label}
                            </label>
                        ))}
                        {launchModeChanged && (
                            <div className="settings-restart-hint">
                                <span>Takes effect after restart.</span>
                                <button
                                    className="settings-action-btn"
                                    onClick={() => { invoke('restart_app').catch(() => {}); }}
                                >
                                    Restart now
                                </button>
                            </div>
                        )}
                    </section>
                </>
            )}
```

> The `aria`/label association works because the radio `<input>` is a child of the
> `<label>` (same pattern as Theme), so `getByLabelText('Window')` resolves.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd hp41-gui && npm test -- SettingsPanel`
Expected: PASS — all SettingsPanel tests green.

- [ ] **Step 5: Commit**

```bash
git add hp41-gui/src/SettingsPanel.tsx hp41-gui/src/SettingsPanel.test.tsx
git commit  # ✨ feat(gui): add macOS launch-mode toggle to settings panel
```

---

## Task 6: Wire SettingsPanel into App.tsx

**Files:**
- Modify: `hp41-gui/src/App.tsx` (state ~272-279, get_prefs effect ~486-503, handler near ~307-313, render ~1059-1065)

- [ ] **Step 1: Add state for launch mode and macOS detection**

Near the theme state (App.tsx ~274), add:

```tsx
  const [theme, setTheme] = useState<string>('dark');
  // macOS launch mode — overridden by get_prefs on mount; only meaningful on macOS.
  const [macosLaunchMode, setMacosLaunchMode] = useState<string>('menu-bar');
  const [isMacos, setIsMacos] = useState(false);
```

- [ ] **Step 2: Detect macOS once on mount**

Add a new effect near the existing get_prefs effect:

```tsx
  useEffect(() => {
    invoke<boolean>('is_macos').then(setIsMacos).catch(() => setIsMacos(false));
  }, []);
```

- [ ] **Step 3: Read macos_launch_mode in the get_prefs effect**

Update the `invoke<...>('get_prefs')` generic and `.then` (App.tsx ~487-489):

```tsx
    invoke<{ theme: string; onboarding_done: boolean; macos_launch_mode: string }>('get_prefs')
      .then(prefs => {
        setTheme(prefs.theme);
        document.body.dataset.theme = prefs.theme;
        setMacosLaunchMode(prefs.macos_launch_mode);
        if (!prefs.onboarding_done) {
```

(The `.catch` branch stays as-is — `macosLaunchMode` keeps its `'menu-bar'` default.)

- [ ] **Step 4: Add the launch-mode change handler**

After `handleThemeChange` (App.tsx ~313), add:

```tsx
  // macOS launch-mode change — persist via fire-and-forget IPC (D-48.13 pattern).
  // The new mode is applied at next startup (decided in setup()), so SettingsPanel
  // reveals a "Restart now" affordance after the change.
  const handleLaunchModeChange = useCallback((mode: string) => {
    setMacosLaunchMode(mode);
    invoke('set_pref', { key: 'macos_launch_mode', value: mode }).catch(() => {
      // Persistence failure is non-fatal — the choice is re-applied on next change.
    });
  }, []);
```

- [ ] **Step 5: Pass the new props into SettingsPanel**

Update the `<SettingsPanel .../>` JSX (App.tsx ~1059-1065):

```tsx
        <SettingsPanel
          open={settingsOpen}
          onClose={() => setSettingsOpen(false)}
          currentTheme={theme}
          onThemeChange={handleThemeChange}
          onShowOnboarding={handleShowOnboarding}
          isMacos={isMacos}
          currentLaunchMode={macosLaunchMode}
          onLaunchModeChange={handleLaunchModeChange}
        />
```

- [ ] **Step 6: Type-check and run the frontend tests**

Run: `cd hp41-gui && npx tsc --noEmit`
Expected: no type errors.

Run: `cd hp41-gui && npm test`
Expected: PASS — full Vitest suite (App.test.tsx may need the same mock for `is_macos`; if `App.test.tsx` asserts on `invoke` calls, add `is_macos` to its mock resolved values — check and fix if it fails).

- [ ] **Step 7: Commit**

```bash
git add hp41-gui/src/App.tsx
git commit  # ✨ feat(gui): wire macOS launch-mode toggle into App
```

---

## Task 7: Documentation

**Files:**
- Modify: `docs/adr/v4.1-001-macos-menu-bar-mode.md`
- Modify: `CLAUDE.md`

- [ ] **Step 1: Add a "Switchable via preference" section to the ADR**

Append to `docs/adr/v4.1-001-macos-menu-bar-mode.md`:

```markdown
## Amendment (2026-05-29): Switchable via preference

The menu-bar mode is now user-selectable. `GuiPrefs.macos_launch_mode`
(`"menu-bar"` | `"window"`, `#[serde(default)]` → `"menu-bar"`) is read in the
macOS `setup()` branch. Precedence: `HP41_SHOW_ON_START` env (E2E backdoor) >
`"window"` preference > menu-bar. Existing prefs.json files load as `"menu-bar"`,
so current users are unaffected. The toggle lives in the Settings panel (macOS only,
gated by the `is_macos` command) and offers an immediate `restart_app` (Tauri v2
`app.restart()`); the switch otherwise takes effect on the next launch. Windows/Linux
are unaffected (the branch is `cfg(target_os = "macos")`).
```

- [ ] **Step 2: Update the CLAUDE.md GUI specifics block**

In the "macOS menu-bar mode (ADR-v4.1-001)" bullet in `CLAUDE.md`, append after the existing description:

```markdown
  The mode is **user-selectable** via `GuiPrefs.macos_launch_mode` (`"menu-bar"` default | `"window"`); the Settings panel exposes a macOS-only radio (gated by the `is_macos` command) that persists via `set_pref` and offers an immediate `restart_app`. Precedence: `HP41_SHOW_ON_START` > `"window"` pref > menu-bar. Existing saves stay menu-bar via `#[serde(default)]`.
```

- [ ] **Step 3: Commit**

```bash
git add docs/adr/v4.1-001-macos-menu-bar-mode.md CLAUDE.md
git commit  # 📝 docs(gui): document switchable macOS launch mode
```

---

## Task 8: Full gate

- [ ] **Step 1: Run the complete CI gate**

Run: `just gui-ci`
Expected: permission gate PASS, `npm ci` clean, `tsc --noEmit` clean, Rust tests PASS, release build OK, Vitest PASS.

- [ ] **Step 2: Verify SC-4 (no core duplication) still holds**

Run: `grep -rn "fn op_(add\|sub\|mul\|div\|sin\|cos\|tan\|sto\|rcl\|flush_entry\|format_hpnum)" hp41-gui/src-tauri/src/`
Expected: empty output.

- [ ] **Step 3: Confirm the unrelated Justfile change was never committed**

Run: `git status --short`
Expected: `M Justfile` still present (untouched), nothing else uncommitted from this work.

---

## Self-Review (completed by plan author)

**Spec coverage:**
- D-1 restart-based → Task 4 (setup branch) + Task 5/6 (restart offered, not live). ✓
- D-2 default menu-bar + existing unchanged → Task 1 `#[serde(default)]` + serde-default test. ✓
- D-3 offer automatic restart → Task 3 `restart_app`, Task 5 "Restart now" button. ✓
- D-4 Settings panel, macOS-only → Task 3 `is_macos`, Task 5 gated section. ✓
- D-5 env var highest precedence → Task 4 branch order. ✓
- IPC `set_pref` arm → Task 2. Permissions → Task 3. Tests → Tasks 1,2,5,6. Docs → Task 7. Gate → Task 8. ✓

**Placeholder scan:** No TBD/TODO; every code step shows full code. ✓

**Type consistency:** `macos_launch_mode` (Rust field, IPC key, prefs JSON key), `macosLaunchMode` (TS state), `currentLaunchMode`/`onLaunchModeChange`/`isMacos` (props) used identically across Tasks 1–6. Command names `restart_app`/`is_macos` match between commands.rs, generate_handler!, permission identifiers (`allow-restart-app`/`allow-is-macos`), and frontend `invoke()` calls. ✓
