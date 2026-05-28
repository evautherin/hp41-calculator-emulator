# Phase 48: GUI Infrastructure + Theming - Pattern Map

**Mapped:** 2026-05-27
**Files analyzed:** 11 (7 new, 4 modified)
**Analogs found:** 11 / 11

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-gui/src-tauri/src/prefs.rs` | service | CRUD (file I/O) | `hp41-gui/src-tauri/src/persistence.rs` | exact |
| `hp41-gui/src-tauri/src/commands.rs` | controller | request-response | `hp41-gui/src-tauri/src/commands.rs` (self — add to existing) | exact |
| `hp41-gui/src-tauri/src/lib.rs` | config | request-response | `hp41-gui/src-tauri/src/lib.rs` (self — modify) | exact |
| `hp41-gui/src-tauri/permissions/get-prefs.toml` | config | — | `hp41-gui/src-tauri/permissions/tick-time.toml` | exact |
| `hp41-gui/src-tauri/permissions/set-pref.toml` | config | — | `hp41-gui/src-tauri/permissions/tick-time.toml` | exact |
| `hp41-gui/src/themes.css` | config | — | `hp41-gui/src/App.css` (color token inventory) | role-match |
| `hp41-gui/src/SettingsPanel.tsx` | component | request-response | `hp41-gui/src/HelpOverlay.tsx` | role-match |
| `hp41-gui/src/SettingsPanel.test.tsx` | test | — | `hp41-gui/src/HelpOverlay.test.tsx` | exact |
| `hp41-gui/src/App.tsx` | component | request-response | `hp41-gui/src/App.tsx` (self — modify) | exact |
| `hp41-gui/src/App.css` | config | — | `hp41-gui/src/App.css` (self — modify) | exact |
| `hp41-gui/src/Keyboard.tsx` | component | — | `hp41-gui/src/Keyboard.tsx` (self — add prop) | exact |
| `hp41-gui/src/main.tsx` | config | — | `hp41-gui/src/main.tsx` (self — add import) | exact |

---

## Pattern Assignments

### `hp41-gui/src-tauri/src/prefs.rs` (service, file I/O)

**Analog:** `hp41-gui/src-tauri/src/persistence.rs`

**Module doc + imports pattern** (lines 1–15 of `persistence.rs`):
```rust
//! State persistence for hp41-gui: save/load CalcState to/from JSON.
//!
//! D-01 (GUI): Default path: ~/.hp41/autosave.json — shared with hp41-cli...
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
```
Copy this header doc style; replace body references with `GuiPrefs` and `~/.hp41/prefs.json`.

**Struct pattern** — derive exactly as `StateFile` but simpler (no version wrapper needed):
```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct GuiPrefs {
    #[serde(default = "default_theme")]
    pub theme: String,
    // Phase 49 will add: #[serde(default)] pub onboarding_done: bool
}

fn default_theme() -> String { "dark".to_string() }

impl Default for GuiPrefs {
    fn default() -> Self { GuiPrefs { theme: default_theme() } }
}
```

**Path helper pattern** (lines 32–37 of `persistence.rs`):
```rust
pub fn default_state_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hp41")
        .join("autosave.json")
}
```
Copy verbatim; change `.join("autosave.json")` to `.join("prefs.json")`. Rename `default_state_path` to `default_prefs_path`.

**Save pattern** (lines 42–51 of `persistence.rs`):
```rust
pub fn save_state(path: &Path, state: &CalcState) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let file = fs::File::create(path)?;
    let wrapper = StateFile::current(state.clone());
    serde_json::to_writer_pretty(file, &wrapper).map_err(std::io::Error::other)
}
```
Copy verbatim; rename `save_state` to `save_prefs`, replace `StateFile::current(state.clone())` with `prefs` directly (no version wrapper for prefs).

**Load pattern** — `prefs.rs` diverges from `persistence.rs` intentionally: missing file is NOT an error (first-run normal case):
```rust
pub fn load_prefs(path: &Path) -> GuiPrefs {
    // NEVER panic on missing file — first run is the normal case.
    let Ok(file) = fs::File::open(path) else { return GuiPrefs::default() };
    serde_json::from_reader(file).unwrap_or_default()
}
```
This is a `GuiPrefs` (not `Result`) — critical divergence. Contrast with `load_state` which returns `Result`.

**Test pattern** (lines 69–105 of `persistence.rs`):
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!("hp41_test_{name}"))
            .join("prefs.json")
    }

    #[test]
    fn test_roundtrip() { ... save_prefs ... load_prefs ... assert_eq! }

    #[test]
    fn test_missing_file_returns_default() {
        let path = PathBuf::from("/nonexistent/path/hp41_no_prefs.json");
        let prefs = load_prefs(&path);
        assert_eq!(prefs.theme, "dark", "missing file must return default theme");
    }

    #[test]
    fn test_corrupt_json_returns_default() { ... fs::write(b"{{bad}}") ... assert_eq!(prefs.theme, "dark") }
}
```

---

### `hp41-gui/src-tauri/src/commands.rs` — new commands (controller, request-response)

**Analog:** `hp41-gui/src-tauri/src/commands.rs` (existing file — append two new commands)

**Type alias pattern** (mirrors `pub type AppState = Mutex<hp41_core::CalcState>` in `lib.rs`):
```rust
// In lib.rs (not commands.rs) — define the type alias there, use it here.
pub type PrefsState = Mutex<GuiPrefs>;
```

**Command signature pattern** — key constraints from `submit_modal_with_label` (lines 371–380 of `commands.rs`):
```rust
// CRITICAL: non-State params MUST precede State<'_> extractors (Tauri v2 rule).
// Pitfall: set_pref(prefs: State<'_...>, key: String, ...) would fail to compile.
#[tauri::command]
pub fn submit_modal_with_label(
    label: String,        // ← non-State param FIRST
    state: State<'_, AppState>,  // ← State extractor LAST
) -> Result<CalcStateView, GuiError> {
```
Apply same ordering to `set_pref`:
```rust
#[tauri::command]
pub fn get_prefs(prefs: State<'_, PrefsState>) -> GuiPrefs {
    prefs.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

#[tauri::command]
pub fn set_pref(key: String, value: String, prefs: State<'_, PrefsState>) -> Result<(), String> {
    let mut p = prefs.lock().unwrap_or_else(|e| e.into_inner());
    match key.as_str() {
        "theme" => {
            let valid = ["dark", "light", "classic-beige", "high-contrast"];
            if !valid.contains(&value.as_str()) {
                return Err(format!("unknown theme: {value}"));
            }
            p.theme = value;
        }
        _ => return Err(format!("unknown pref key: {key}")),
    }
    save_prefs(&default_prefs_path(), &*p).map_err(|e| e.to_string())
}
```

**Mutex unlock pattern** — consistent with all existing commands (line 53 of `commands.rs`):
```rust
let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
```
Use `.unwrap_or_else(|e| e.into_inner())` everywhere — never bare `.unwrap()`.

---

### `hp41-gui/src-tauri/src/lib.rs` — managed state registration (config)

**Analog:** `hp41-gui/src-tauri/src/lib.rs` (self — append to `.setup()` closure)

**Managed state registration pattern** (lines 58–61 of `lib.rs`):
```rust
// Existing pattern for CalcState:
app.manage(Mutex::new(initial_state));
app.manage(cancel_flag);

// New pattern for GuiPrefs (insert BEFORE app.manage(Mutex::new(initial_state))):
let prefs_path = prefs::default_prefs_path();
let initial_prefs = prefs::load_prefs(&prefs_path);
app.manage(std::sync::Mutex::new(initial_prefs));
```

**invoke_handler registration pattern** (lines 88–99 of `lib.rs`):
```rust
.invoke_handler(tauri::generate_handler![
    commands::dispatch_op,
    commands::get_state,
    // ... existing commands ...
    commands::tick_time,
    commands::get_prefs,   // ← append here
    commands::set_pref,    // ← append here
])
```

**Module declaration pattern** (lines 6–11 of `lib.rs`):
```rust
pub mod cards;
mod commands;
mod key_map;
mod persistence;
mod prgm_display;
pub mod types;
// Add:
mod prefs;
```

---

### `hp41-gui/src-tauri/permissions/get-prefs.toml` (config)

**Analog:** `hp41-gui/src-tauri/permissions/tick-time.toml` (lines 1–6):
```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-tick-time"
description = "Allows the tick_time command."
commands.allow = ["tick_time"]
```
Copy verbatim; substitute `tick-time` / `tick_time` with `get-prefs` / `get_prefs`.

**CRITICAL two-step:** After creating the TOML, immediately add `"allow-get-prefs"` to `hp41-gui/src-tauri/capabilities/default.json` (lines 5–17 of `default.json`). These are an atomic pair.

---

### `hp41-gui/src-tauri/permissions/set-pref.toml` (config)

Same pattern as `get-prefs.toml` above; substitute with `set-pref` / `set_pref`.

---

### `hp41-gui/src/themes.css` (config — new file)

**Analog:** `hp41-gui/src/App.css` (color inventory; lines 1–120)

**Hardcoded colors to migrate** (extracted from `App.css` reading):

| CSS class / property | Current hardcoded value | CSS variable name |
|----------------------|------------------------|-------------------|
| `.calculator` `background` | `#0d0d0d` | `--calc-bg` |
| `.calculator` `border` | `#333` | `--calc-border` |
| `.annunciators` `background` | `#1a1a1a` | `--annunciator-bg` |
| `.annunciator` `color` | `#555` | `--annunciator-inactive` |
| `.annunciator.active` `color` | `#e8e8c0` | `--annunciator-active` |
| `.display` `background` | `#111` | `--display-bg` |
| `.display` `color` | `#c8e6c9` | `--display-text` |
| `.display` `border-bottom` | `#222` | `--display-border` |
| `.stack-panel` `background` | `#1a1a1a` | `--stack-bg` |
| `.stack-row` `color` | `#aaa` | `--stack-text` |
| `.stack-label` `color` | `#666` | `--stack-label` |
| `.print-panel` `background` | `#1a1a1a` | `--panel-bg` |
| `.print-panel-header` `background` | `#252525` | `--panel-header-bg` |
| `.print-panel-header` border | `#3a3a3a` | `--panel-border` |
| `.print-panel-header` `color` | `#888` | `--panel-text` |
| `.step-active` `background` | `#1e3a1e` | `--step-active-bg` |
| `.step-active` `color` | `#c8e6c9` | `--step-active-text` |

**CSS variable block structure** (D-48.9, D-48.11):
```css
/* themes.css — D-48.11: all four [data-theme] blocks here; App.css keeps layout only */

[data-theme="dark"] {
  --calc-bg: #0d0d0d;
  --calc-border: #333;
  --annunciator-bg: #1a1a1a;
  --annunciator-inactive: #555;
  --annunciator-active: #e8e8c0;
  --annunciator-shift-active: #d68a1c;
  --display-bg: #111;
  --display-text: #c8e6c9;
  --display-border: #222;
  --stack-bg: #1a1a1a;
  --stack-text: #aaa;
  --stack-label: #666;
  --panel-bg: #1a1a1a;
  --panel-header-bg: #252525;
  --panel-border: #3a3a3a;
  --panel-text: #888;
  --step-active-bg: #1e3a1e;
  --step-active-text: #c8e6c9;
  /* shift/alpha label colors — used as SVG text fill */
  --shift-label: #d68a1c;
  --alpha-label-inactive: #5b8fb9;
  --alpha-label-active: #7fb9e0;
}

[data-theme="light"]         { /* similar block — light values */ }
[data-theme="classic-beige"] { /* similar block — HP-41C tan palette */ }
[data-theme="high-contrast"] { /* similar block — #ffffff on #000000 */ }
```

**P51 invariant — NEVER put these in themes.css** (lines 101–108 of `App.css`):
```css
/* These stay exclusively in App.css — do NOT copy to any [data-theme] block */
.key {
  transform-box: fill-box;   /* P51: must not be overridden by themes */
  transform-origin: center;  /* P51: must not be overridden by themes */
  transition: transform 80ms ease-out;
}
```

---

### `hp41-gui/src/SettingsPanel.tsx` (component, request-response)

**Analog:** `hp41-gui/src/HelpOverlay.tsx`

**Imports pattern** (lines 33–35 of `HelpOverlay.tsx`):
```typescript
import { useState, useEffect, useMemo } from 'react';
```
For SettingsPanel:
```typescript
import { useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
```

**Props interface pattern** (lines 36–39 of `HelpOverlay.tsx`):
```typescript
export type HelpOverlayProps = {
    open: boolean;
    onClose: () => void;
};
```
For SettingsPanel:
```typescript
export type SettingsPanelProps = {
    open: boolean;
    onClose: () => void;
    currentTheme: string;
    onThemeChange: (theme: string) => void;
};
```

**Click-outside dismiss pattern** (lines 171–181 of `HelpOverlay.tsx`):
```typescript
// HelpOverlay uses keydown for Esc:
useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
        if (e.key === 'Escape') { e.preventDefault(); onClose(); }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
}, [open, onClose]);
```
SettingsPanel uses `mousedown` for click-outside (D-48.4):
```typescript
const panelRef = useRef<HTMLDivElement>(null);

useEffect(() => {
    if (!open) return;
    const handleClickOutside = (e: MouseEvent) => {
        if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
            onClose();
        }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
}, [open, onClose]);
```
**Pitfall:** The gear icon `mousedown` must call `e.stopPropagation()` — otherwise the outside-click listener fires on the same event that opens the panel, causing immediate close (RESEARCH.md §Pitfall: Gear Icon Click Immediately Re-Closing Panel).

**Early return pattern** (line 183 of `HelpOverlay.tsx`):
```typescript
if (!open) return null;
```
Copy verbatim for SettingsPanel.

**Theme radio list pattern** (new — no exact analog):
```typescript
const THEMES = [
    { id: 'dark',           label: 'Dark' },
    { id: 'light',          label: 'Light' },
    { id: 'classic-beige',  label: 'Classic Beige' },
    { id: 'high-contrast',  label: 'High Contrast' },
];

return (
    <div ref={panelRef} className="settings-panel" role="dialog" aria-label="Settings">
        <section className="settings-section">
            <h3 className="settings-section-heading">Theme</h3>
            {THEMES.map(t => (
                <label key={t.id} className="settings-radio-row">
                    <input
                        type="radio"
                        name="theme"
                        value={t.id}
                        checked={currentTheme === t.id}
                        onChange={() => onThemeChange(t.id)}
                    />
                    {t.label}
                </label>
            ))}
        </section>
        {/* Phase 49 will add an Onboarding section here — structure is intentionally shell-like */}
    </div>
);
```

---

### `hp41-gui/src/SettingsPanel.test.tsx` (test)

**Analog:** `hp41-gui/src/HelpOverlay.test.tsx`

**Test file structure** (lines 34–50 of `HelpOverlay.test.tsx`):
```typescript
import { describe, it, expect } from 'vitest';
import { render, fireEvent } from '@testing-library/react';
import { HelpOverlay } from './HelpOverlay';

describe('HelpOverlay', () => {
    it('renders null when open=false', () => {
        const { container } = render(<HelpOverlay open={false} onClose={() => {}} />);
        expect(container.firstChild).toBeNull();
    });
    // ...
});
```
For SettingsPanel:
```typescript
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/react';
import { SettingsPanel } from './SettingsPanel';

// Mock Tauri invoke — SettingsPanel calls invoke('set_pref', ...)
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn().mockResolvedValue(undefined),
}));
```

**Key behaviors to test** (from RESEARCH.md §Validation Architecture):
- Renders 4 radio buttons when `open=true`
- Clicking a radio calls `invoke('set_pref', { key: 'theme', value: '<id>' })`
- Returns null when `open=false`

---

### `hp41-gui/src/App.tsx` — modifications (component, request-response)

**Analog:** `hp41-gui/src/App.tsx` (self)

**Mount useEffect pattern to copy** (lines 286–290 of `App.tsx`):
```typescript
// Existing get_state mount pattern:
useEffect(() => {
    invoke<CalcStateView>('get_state')
        .then(view => { setCalcState(view); setErrorMessage(null); })
        .catch(err => setErrorMessage(`Load failed: ${err}`));
}, []);
```
New `get_prefs` mount pattern (add alongside existing useEffect):
```typescript
const [theme, setTheme] = useState<string>('dark');

useEffect(() => {
    invoke<{ theme: string }>('get_prefs')
        .then(prefs => {
            setTheme(prefs.theme);
            document.body.dataset.theme = prefs.theme;
        })
        .catch(() => {
            document.body.dataset.theme = 'dark';  // silent failure — default stays
        });
}, []);
```

**State/open pattern** (line 236 of `App.tsx`):
```typescript
const [helpOpen, setHelpOpen] = useState(false);
// Add:
const [settingsOpen, setSettingsOpen] = useState(false);
```

**HelpOverlay integration in JSX** (lines 831–835 of `App.tsx`):
```typescript
<HelpOverlay open={helpOpen} onClose={() => setHelpOpen(false)} />
// Add alongside:
<SettingsPanel
    open={settingsOpen}
    onClose={() => setSettingsOpen(false)}
    currentTheme={theme}
    onThemeChange={(newTheme) => {
        setTheme(newTheme);
        document.body.dataset.theme = newTheme;
        invoke('set_pref', { key: 'theme', value: newTheme }).catch(() => {});
    }}
/>
```

**Gear icon placement** — next to existing `?` icon. The `?` button pattern is in App.tsx JSX; gear icon follows same placement:
```typescript
// Wherever the ? icon is rendered, add ⚙ button alongside it:
<button
    className="settings-gear-btn"
    aria-label="Open settings"
    onMouseDown={(e) => { e.stopPropagation(); setSettingsOpen(prev => !prev); }}
>
    ⚙
</button>
```
Note `onMouseDown` + `stopPropagation()` to avoid gear-click-causes-immediate-close pitfall.

---

### `hp41-gui/src/App.css` — modifications (config)

**Analog:** `hp41-gui/src/App.css` (self)

**Migration pattern** — replace hardcoded hex values with `var(--token)` references:
```css
/* BEFORE: */
.calculator {
    background: #0d0d0d;
    border: 1px solid #333;
}

/* AFTER: */
.calculator {
    background: var(--calc-bg);
    border: 1px solid var(--calc-border);
}
```
Apply same substitution for every property listed in the themes.css color migration table above.

**Do NOT touch** `.key` animation properties (P51 invariant — lines 101–117 of `App.css`).

---

### `hp41-gui/src/Keyboard.tsx` — gradient prop extension (component)

**Analog:** `hp41-gui/src/Keyboard.tsx` (self)

**Current hardcoded defs** (lines 233–266 of `Keyboard.tsx`):
```typescript
<defs>
    <linearGradient id="body-grad" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%"   stopColor="#1a1a1a" />
        <stop offset="100%" stopColor="#000000" />
    </linearGradient>
    <linearGradient id="grad-dark" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%"   stopColor="#303030" />
        <stop offset="60%"  stopColor="#181818" />
        <stop offset="100%" stopColor="#080808" />
    </linearGradient>
    ...
</defs>
```
These must accept React props instead of hardcoded strings (P55 — CSS vars do not work here).

**Prop extension pattern** — add `gradientColors` prop to `Keyboard` component:
```typescript
// New interface (add at top of Keyboard.tsx):
export interface GradientColors {
    bodyTop: string;
    bodyBottom: string;
    keyDarkTop: string;
    keyDarkMid: string;
    keyDarkBot: string;
    enterTop: string;
    enterMid: string;
    enterBot: string;
    shiftIdleTop: string;
    shiftIdleMid: string;
    shiftIdleBot: string;
    shiftActiveTop: string;
    shiftActiveMid: string;
    shiftActiveBot: string;
}

// Default (dark) values matching current hardcoded stops:
export const DARK_GRADIENT_COLORS: GradientColors = {
    bodyTop: "#1a1a1a", bodyBottom: "#000000",
    keyDarkTop: "#303030", keyDarkMid: "#181818", keyDarkBot: "#080808",
    enterTop: "#346034", enterMid: "#1a3a1a", enterBot: "#0a180a",
    shiftIdleTop: "#d68a1c", shiftIdleMid: "#b06811", shiftIdleBot: "#7a4708",
    shiftActiveTop: "#ffb742", shiftActiveMid: "#f5a423", shiftActiveBot: "#c97d10",
};

// Keyboard component signature update:
export function Keyboard({ onKeyClick, pressedKey, shiftActive, gradientColors = DARK_GRADIENT_COLORS, ... }: KeyboardProps & { gradientColors?: GradientColors })
```

**Updated defs usage:**
```typescript
<stop offset="0%"   stopColor={gradientColors.bodyTop} />
<stop offset="100%" stopColor={gradientColors.bodyBottom} />
```

---

### `hp41-gui/src/main.tsx` — import themes.css (config)

**Analog:** `hp41-gui/src/main.tsx` (self — 10 lines, trivial)

**Current pattern** (lines 1–10 of `main.tsx`):
```typescript
import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'
```
Add import:
```typescript
import './themes.css'   // D-48.11 — must come AFTER index.css so [data-theme] blocks win
```

---

## Shared Patterns

### Mutex Lock/Unlock (Rust)
**Source:** `hp41-gui/src-tauri/src/commands.rs` line 53, used in every command
**Apply to:** `get_prefs`, `set_pref` commands
```rust
prefs.lock().unwrap_or_else(|e| e.into_inner())
```
Never use `.unwrap()` alone — poisoned lock recovery is required.

### Directory Creation Before Write (Rust)
**Source:** `hp41-gui/src-tauri/src/persistence.rs` lines 43–47
**Apply to:** `save_prefs` in `prefs.rs`
```rust
if let Some(parent) = path.parent() {
    if !parent.as_os_str().is_empty() {
        fs::create_dir_all(parent)?;
    }
}
```

### `serde_json` Error Mapping (Rust)
**Source:** `hp41-gui/src-tauri/src/persistence.rs` line 50
**Apply to:** `save_prefs`
```rust
serde_json::to_writer_pretty(file, prefs).map_err(std::io::Error::other)
```

### Tauri Permission TOML + capabilities pairing
**Source:** `hp41-gui/src-tauri/permissions/tick-time.toml` + `capabilities/default.json`
**Apply to:** `get-prefs.toml`, `set-pref.toml`
- Create TOML in `permissions/`
- Immediately add `"allow-<cmd-kebab>"` entry to `capabilities/default.json`
- Run `cargo check` BEFORE creating the TOML (generates the `gen/schemas/desktop-schema.json` registry that the TOML `$schema` references)

### `invoke` + fire-and-forget (TypeScript)
**Source:** `hp41-gui/src/App.tsx` lines 100–101 (request_cancel pattern)
**Apply to:** theme change handler in `App.tsx`
```typescript
invoke('set_pref', { key: 'theme', value: newTheme }).catch(() => {});
// No await — prefs write is tiny and non-critical for UX
```

### Tauri `invoke` mock in tests
**Source:** `hp41-gui/src/HelpOverlay.test.tsx` — does not mock invoke (HelpOverlay doesn't call it)
**Apply to:** `SettingsPanel.test.tsx` — SettingsPanel does call invoke
```typescript
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn().mockResolvedValue(undefined),
}));
```
Pattern confirmed from project Vitest setup in `vite.config.ts`.

---

## No Analog Found

All files in this phase have direct analogs in the codebase. No file is genuinely novel:

| File | Closest analog | Gap |
|------|---------------|-----|
| `themes.css` | `App.css` | CSS variable blocks are new syntax; `[data-theme]` attribute selector is browser-native — no library needed |
| `SettingsPanel.tsx` click-outside | `HelpOverlay.tsx` Esc-close | Different event (`mousedown` vs `keydown`) but same `useEffect` + `document.addEventListener` structure |

---

## Critical Pitfall Index (from RESEARCH.md)

| Pitfall | File(s) Affected | How to Avoid |
|---------|-----------------|-------------|
| P51 — SVG animation broken by theme CSS | `themes.css`, `App.css` | Never set `transform-box` or `transform-origin` in `themes.css`; leave `.key` untouched |
| P55 — SVG `<defs>` gradient stops ignore CSS vars | `Keyboard.tsx`, `App.tsx` | Pass gradient stop colors as `GradientColors` React prop; CSS vars only for flat fills |
| P59 — Theme preference leaking into `autosave.json` | `prefs.rs`, `commands.rs` | `GuiPrefs` is a completely separate struct/file/Tauri-state object from `CalcState` |
| Permission TOML not in `default.json` | `get-prefs.toml`, `set-pref.toml` | Create TOML and add to `default.json` as an atomic pair |
| `set_pref` param ordering | `commands.rs` | `key: String, value: String` BEFORE `prefs: State<'_, PrefsState>` |
| Gear icon immediate re-close | `App.tsx`, `SettingsPanel.tsx` | Gear button uses `onMouseDown` + `e.stopPropagation()` |
| Flash on first render | `index.html`, `App.tsx` | Set `data-theme="dark"` on `<body>` in `index.html` as default |

---

## Metadata

**Analog search scope:** `hp41-gui/src/`, `hp41-gui/src-tauri/src/`, `hp41-gui/src-tauri/permissions/`, `hp41-gui/src-tauri/capabilities/`
**Files scanned:** 12 source files read directly
**Pattern extraction date:** 2026-05-27
