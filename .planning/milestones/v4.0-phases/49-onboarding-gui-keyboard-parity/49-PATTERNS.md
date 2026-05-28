# Phase 49: Onboarding + GUI Keyboard Parity - Pattern Map

**Mapped:** 2026-05-27
**Files analyzed:** 14 (9 modified, 5 new/modified data files)
**Analogs found:** 14 / 14

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `hp41-gui/src/OnboardingWizard.tsx` | component | request-response | `hp41-gui/src/HelpOverlay.tsx` | role-match (overlay pattern) |
| `hp41-gui/src/OnboardingWizard.test.tsx` | test | — | `hp41-gui/src/HelpOverlay.test.tsx` | exact |
| `hp41-gui/src/HelpOverlay.tsx` | component | request-response | self (modify) | exact |
| `hp41-gui/src/HelpOverlay.test.tsx` | test | — | self (modify) | exact |
| `hp41-gui/src/SettingsPanel.tsx` | component | request-response | self (modify) | exact |
| `hp41-gui/src/SettingsPanel.test.tsx` | test | — | self (modify) | exact |
| `hp41-gui/src/help_data.ts` | utility | transform | self (modify) | exact |
| `hp41-gui/src/App.tsx` | component | event-driven | self (modify) | exact |
| `hp41-gui/src/App.css` | config | — | self (modify) | exact |
| `hp41-gui/src-tauri/src/prefs.rs` | model | CRUD | self (modify) | exact |
| `hp41-gui/src-tauri/src/commands.rs` | controller | request-response | self (modify) | exact |
| `hp41-gui/src-tauri/src/lib.rs` | config | — | self (modify) | exact |
| `hp41-gui/src-tauri/permissions/save-state.toml` | config | — | `hp41-gui/src-tauri/permissions/tick-time.toml` | exact |
| `docs/keyboard-shortcuts.json` | config | — | `docs/hp41cv-functions.json` (Vite static import pattern) | role-match |

---

## Pattern Assignments

### `hp41-gui/src/OnboardingWizard.tsx` (component, request-response)

**Analog:** `hp41-gui/src/HelpOverlay.tsx`

**Imports pattern** (`HelpOverlay.tsx` lines 33–34):
```typescript
import { useState, useEffect, useMemo } from 'react';
import { helpEntriesAll, type HelpEntry } from './help_data';
```
For OnboardingWizard, omit `useMemo`/`helpEntriesAll` — use `useState` + `useEffect` only:
```typescript
import { useState, useEffect } from 'react';
```

**Props interface pattern** (`HelpOverlay.tsx` lines 36–39):
```typescript
export type HelpOverlayProps = {
    open: boolean;
    onClose: () => void;
};
```
OnboardingWizard uses identical shape — `open: boolean; onClose: () => void`.

**Esc-close pattern** (`HelpOverlay.tsx` lines 171–181):
```typescript
useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
        if (e.key === 'Escape') {
            e.preventDefault();
            onClose();
        }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
}, [open, onClose]);
```

**Early-return / null render pattern** (`HelpOverlay.tsx` line 183):
```typescript
if (!open) return null;
```

**Full-cover overlay root element pattern** (`HelpOverlay.tsx` lines 192–193):
```tsx
<div className="help-overlay" role="dialog" aria-label="HP-41 function reference">
```
OnboardingWizard uses `className="onboarding-overlay"` with `aria-label="Quick Start Guide"`.

**Reset-on-open pattern** (`HelpOverlay.tsx` lines 117–122):
```typescript
useEffect(() => {
    if (open) {
        setQuery('');
        setExpanded({ hp41cv: true, math1: true, ... });
    }
}, [open]);
```
OnboardingWizard resets `panelIdx` to 0 on each open (D-49.9):
```typescript
useEffect(() => {
    if (open) setPanelIdx(0);
}, [open]);
```

**PANELS const-array pattern** (`HelpOverlay.tsx` lines 59–98, `SECTIONS` array):
```typescript
const SECTIONS: SectionDef[] = [
    { id: 'hp41cv', heading: 'HP-41CV (built-in)', predicate: ... },
    ...
] as const;
```
OnboardingWizard uses a simpler parallel:
```typescript
const PANELS = [
    { title: "Welcome to HP-41C", ... },
    { title: "The Four-Level Stack", ... },
    { title: "SHIFT & Function Access", ... },
    { title: "Keyboard Shortcuts", ... },
    { title: "Programming & XEQ", ... },
] as const;
```

---

### `hp41-gui/src/OnboardingWizard.test.tsx` (test)

**Analog:** `hp41-gui/src/HelpOverlay.test.tsx` (lines 1–43) + `hp41-gui/src/SettingsPanel.test.tsx`

**Test file header + mock pattern** (`SettingsPanel.test.tsx` lines 14–21):
```typescript
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/react';
import { SettingsPanel } from './SettingsPanel';

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn().mockResolvedValue(undefined),
}));
```

**Null-render test pattern** (`SettingsPanel.test.tsx` lines 23–34):
```typescript
it('renders null when open=false', () => {
    const { container } = render(
        <SettingsPanel open={false} onClose={() => {}} ... />
    );
    expect(container.firstChild).toBeNull();
});
```

**Esc-close test pattern** (from `HelpOverlay.test.tsx` — look for fireEvent.keyDown pattern):
```typescript
it('Esc closes wizard', () => {
    const onClose = vi.fn();
    render(<OnboardingWizard open={true} onClose={onClose} />);
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(onClose).toHaveBeenCalledOnce();
});
```

---

### `hp41-gui/src/HelpOverlay.tsx` (component, modified)

**Analog:** self — three additive changes required.

**Change 1 — Widen SectionDef id union** (`HelpOverlay.tsx` lines 50–54):
```typescript
// BEFORE (Phase 46):
interface SectionDef {
    id: 'hp41cv' | 'math1' | 'stat1' | 'time' | 'adv22' | 'adv24';
    ...
}
// AFTER (Phase 49 — add 'kbd'):
interface SectionDef {
    id: 'kbd' | 'hp41cv' | 'math1' | 'stat1' | 'time' | 'adv22' | 'adv24';
    ...
}
```
Must also widen the `expanded` state type at line 107 and the `toggleSection` parameter at line 185.

**Change 2 — expanded state widen** (`HelpOverlay.tsx` lines 107–114):
```typescript
// BEFORE:
const [expanded, setExpanded] = useState<{ hp41cv: boolean; math1: boolean; stat1: boolean; time: boolean; adv22: boolean; adv24: boolean }>({
    hp41cv: true, math1: true, stat1: true, time: true, adv22: true, adv24: true,
});
// AFTER (add kbd: true in both places — line 107 type + line 120 reset):
const [expanded, setExpanded] = useState<{ kbd: boolean; hp41cv: boolean; math1: boolean; stat1: boolean; time: boolean; adv22: boolean; adv24: boolean }>({
    kbd: true, hp41cv: true, math1: true, stat1: true, time: true, adv22: true, adv24: true,
});
```

**Change 3 — expandable row pattern** (`HelpOverlay.tsx` lines 232–238, current compact row):
```tsx
{entries.map(entry => (
    <div key={entry.op_variant} className="help-overlay-row">
        <span className="help-overlay-key">{entry.key_path}</span>
        <span className="help-overlay-op">{entry.display_name}</span>
        <span className="help-overlay-desc">{entry.description}</span>
    </div>
))}
```
Phase 49 replaces with click-to-expand when `entry.example || entry.notes` present. Keep the existing 3-span layout for the compact row, add a detail div below when expanded.

---

### `hp41-gui/src/SettingsPanel.tsx` (component, modified)

**Analog:** self — additive change at line 66.

**Props extension pattern** (`SettingsPanel.tsx` lines 11–16):
```typescript
// BEFORE:
export type SettingsPanelProps = {
    open: boolean;
    onClose: () => void;
    currentTheme: string;
    onThemeChange: (theme: string) => void;
};
// AFTER (add onShowOnboarding):
export type SettingsPanelProps = {
    open: boolean;
    onClose: () => void;
    currentTheme: string;
    onThemeChange: (theme: string) => void;
    onShowOnboarding: () => void;  // D-49.8 / D-49.9
};
```

**Existing section pattern for new Quick Start section** (`SettingsPanel.tsx` lines 51–65):
```tsx
<section className="settings-section">
    <h3 className="settings-section-heading">Theme</h3>
    {THEMES.map(t => (
        <label key={t.id} className="settings-radio-row">
            <input type="radio" name="theme" value={t.id}
                checked={currentTheme === t.id}
                onChange={() => onThemeChange(t.id)} />
            {t.label}
        </label>
    ))}
</section>
{/* Phase 49 will add an Onboarding section here (D-48.3 shell). */}
```
Insert after the closing `</section>`:
```tsx
<section className="settings-section">
    <h3 className="settings-section-heading">Quick Start</h3>
    <button
        className="settings-action-btn"
        onClick={() => { onClose(); onShowOnboarding(); }}
    >
        Show Guide
    </button>
</section>
```

**Click-outside dismiss pattern** (`SettingsPanel.tsx` lines 31–40) — unchanged, already present.

---

### `hp41-gui/src/help_data.ts` (utility, modified)

**Analog:** self — two additive changes.

**Change 1 — Extend HelpEntry interface** (`help_data.ts` lines 54–76):
```typescript
export interface HelpEntry {
    op_variant: string;
    display_name: string;
    category: string;
    status: 'implemented' | 'deferred-v3' | 'na';
    phase: string | null;
    key_path: string | null;
    description: string;
    divergences?: string[];
    xrom?: XromEntry;
    // Phase 49 D-49.5 — optional enrichment fields for expandable rows (ONBOARD-03)
    example?: string;   // e.g. "3 ENTER 4 + → 7"
    notes?: string;     // e.g. "Stack lift enabled; T register lost"
}
```

**Change 2 — Add keyboard shortcuts import + accessor** (after line 32, following `advantageFunctions` import):
```typescript
// Phase 49 D-49.11 — keyboard shortcuts single source of truth
import keyboardShortcutsData from '../../docs/keyboard-shortcuts.json';

export interface KeyboardShortcut {
    key: string;
    op: string;
    description: string;
}

export function getKeyboardShortcuts(): readonly KeyboardShortcut[] {
    return keyboardShortcutsData as readonly KeyboardShortcut[];
}
```
**Vite static JSON-import pattern** (`help_data.ts` lines 28–32 — exact precedent):
```typescript
import functions from '../../docs/hp41cv-functions.json';
import math1Functions from '../../docs/hp41-math1-functions.json';
import stat1Functions from '../../docs/hp41-stat1-functions.json';
import timeFunctions from '../../docs/hp41-time-functions.json';
import advantageFunctions from '../../docs/hp41-advantage-functions.json';
```
The new `keyboard-shortcuts.json` import uses `../../docs/` prefix — same relative path.

---

### `hp41-gui/src/App.tsx` (component, modified)

**Analog:** self — three additive changes.

**Change 1 — State variables** (`App.tsx` lines 237–241, existing overlay state pattern):
```typescript
const [helpOpen, setHelpOpen] = useState(false);
const [settingsOpen, setSettingsOpen] = useState(false);
const [theme, setTheme] = useState<string>('dark');
// Add Phase 49:
const [onboardingOpen, setOnboardingOpen] = useState(false);
```

**Change 2 — Extend get_prefs useEffect** (`App.tsx` lines 312–321):
```typescript
// BEFORE (Phase 48):
useEffect(() => {
    invoke<{ theme: string }>('get_prefs')
        .then(prefs => {
            setTheme(prefs.theme);
            document.body.dataset.theme = prefs.theme;
        })
        .catch(() => {
            document.body.dataset.theme = 'dark';
        });
}, []);
// AFTER (Phase 49 — add onboarding_done check):
useEffect(() => {
    invoke<{ theme: string; onboarding_done: boolean }>('get_prefs')
        .then(prefs => {
            setTheme(prefs.theme);
            document.body.dataset.theme = prefs.theme;
            if (!prefs.onboarding_done) {
                setOnboardingOpen(true);
            }
        })
        .catch(() => {
            document.body.dataset.theme = 'dark';
            setOnboardingOpen(true);  // First run fallback
        });
}, []);
```

**Change 3 — resolveKeyId Ctrl+key insertion** (`App.tsx` lines 109–112, existing F7/F8 checks):
```typescript
function resolveKeyId(e: KeyboardEvent, state: CalcStateView | null): string | null {
    // Phase 18 D-07: F7/F8 → SST/BST keyboard bindings
    if (e.key === 'F7' || e.code === 'F7') return 'sst';
    if (e.key === 'F8' || e.code === 'F8') return 'bst';
    // ...
}
```
Insert BEFORE the F7/F8 checks (Ctrl+key must be first, per RESEARCH Pitfall 2):
```typescript
// Phase 49 D-49.12/D-49.13 — Ctrl+key / Cmd+key bindings
// MUST come before the letter-key MAP to prevent double-firing.
if (e.ctrlKey) {
    switch (e.key.toLowerCase()) {
        case 'w': return 'xeq_WPRGM';
        case 'r': return 'xeq_RDPRGM';
        case 'd': return 'xeq_WDTA';
        case 'f': return 'xeq_RDTA';
        case 's': return '__save_state__';
    }
    return null;  // Other Ctrl combos — do not fall through to letter map
}
// Phase 49 KBD-02 — F5: manual save (GUI-only; CLI F5 = run_program, deliberate divergence)
if (e.key === 'F5') return '__save_state__';
```

**Change 4 — handleKey save dispatch** (inside `handleKey`, after Esc/Tab handling, before `resolveKeyId` call at line 647):
```typescript
// Intercept __save_state__ BEFORE dispatchKeyId routes to dispatch_op (RESEARCH Pitfall 4)
if (keyId === '__save_state__') {
    e.preventDefault();
    invoke<void>('save_state')
        .then(() => showToast('Saved'))
        .catch(err => showToast(`Save failed: ${extractErrMessage(err)}`));
    return;
}
```
**Toast pattern** (`App.tsx` lines 250–253):
```typescript
const showToast = useCallback((msg: string) => {
    toastSeqRef.current += 1;
    setToast({ msg, seq: toastSeqRef.current });
}, []);
```

---

### `hp41-gui/src/App.css` (config, modified)

**Analog:** self — existing overlay CSS at lines 268–399 and settings panel at lines 462–494.

**Overlay z-index stack** (`App.css` comments):
```
toast: z-index 50
help-overlay: z-index 60  (line 275)
settings-panel: z-index 70  (line 466)
```
Wizard overlay uses `z-index: 65` (between help and settings; wizard and help are mutually exclusive per App.tsx state).

**Full-cover overlay CSS pattern** (`App.css` lines 268–280):
```css
.help-overlay {
    position: absolute;
    top: 0; left: 0; right: 0; bottom: 0;
    background: var(--overlay-bg);
    z-index: 60;
    display: flex;
    flex-direction: column;
    font-family: 'Courier New', Courier, monospace;
    color: var(--display-text);
}
```
OnboardingWizard uses same pattern with `z-index: 65` and system-ui font (content is prose, not monospace).

**Settings section button pattern** (`App.css` lines 476–494, `settings-section-heading` + `settings-radio-row`): New `settings-action-btn` follows same sizing/color variables.

---

### `hp41-gui/src-tauri/src/prefs.rs` (model, modified)

**Analog:** self — additive field to `GuiPrefs` struct.

**Existing serde(default) field pattern** (`prefs.rs` lines 35–39):
```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GuiPrefs {
    #[serde(default = "default_theme")]
    pub theme: String,
}
```
Add `onboarding_done` using plain `#[serde(default)]` (bool default is `false`):
```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GuiPrefs {
    #[serde(default = "default_theme")]
    pub theme: String,
    // Phase 49 — D-49.4: first-run onboarding seen-flag.
    // P59/ONBOARD-05: lives here, NEVER in CalcState/autosave.json.
    #[serde(default)]
    pub onboarding_done: bool,
}
```
Must also add `onboarding_done: false` to `impl Default for GuiPrefs` (`prefs.rs` lines 46–51) and to the test at line 112.

**Test roundtrip pattern** (`prefs.rs` lines 111–120):
```rust
#[test]
fn test_roundtrip() {
    let path = temp_path("prefs_roundtrip");
    let prefs = GuiPrefs { theme: "light".to_string() };
    save_prefs(&path, &prefs).unwrap();
    let loaded = load_prefs(&path);
    assert_eq!(loaded.theme, "light", "roundtrip must preserve theme value");
    let _ = fs::remove_dir_all(path.parent().unwrap());
}
```
Add a parallel test for `onboarding_done` roundtrip and the serde-default case (old JSON without the field must load with `onboarding_done: false`).

---

### `hp41-gui/src-tauri/src/commands.rs` (controller, modified)

**Analog:** self — two additive changes.

**Change 1 — set_pref match arm extension** (`commands.rs` lines 442–451):
```rust
match key.as_str() {
    "theme" => {
        if !VALID_THEMES.contains(&value.as_str()) {
            return Err(format!("unknown theme: {value}"));
        }
        p.theme = value;
    }
    _ => return Err(format!("unknown pref key: {key}")),
}
```
Add before the `_` arm:
```rust
"onboarding_done" => {
    p.onboarding_done = value == "true";  // Tauri IPC: bool encoded as "true"/"false" string
}
```

**Change 2 — new save_state Tauri command** (pattern from auto-save thread in `lib.rs` lines 87–95):
```rust
// lib.rs auto-save pattern (lines 87-95):
let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();
if let Err(e) = persistence::save_state(&thread_save_path, &snapshot) {
    eprintln!("auto-save failed: {e}");
}
```
New command in `commands.rs` (add after `set_pref`, before test module):
```rust
/// Tauri command: manual save to ~/.hp41/autosave.json (Phase 49 KBD-02 / Ctrl+S / F5).
///
/// Mirrors CLI Ctrl+S handler (hp41-cli/src/app.rs lines 324-330).
/// Clone-under-lock pattern: AppState mutex released before disk I/O (CR-01).
/// P59/THEME-05: does not touch GuiPrefs or prefs.json.
#[tauri::command]
pub fn save_state(state: State<'_, AppState>) -> Result<(), String> {
    let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let path = persistence::default_state_path();
    persistence::save_state(&path, &snapshot).map_err(|e| e.to_string())
}
```
Note: `persistence::save_state` is the module-qualified call — resolves the name-shadowing pitfall (RESEARCH Pitfall 1).

**Poisoned-lock recovery pattern** (used throughout `commands.rs`):
```rust
state.lock().unwrap_or_else(|e| e.into_inner())
```

---

### `hp41-gui/src-tauri/src/lib.rs` (config, modified)

**Analog:** self — register `save_state` in `invoke_handler`.

**Existing registration pattern** (`lib.rs` lines 100–113):
```rust
.invoke_handler(tauri::generate_handler![
    commands::dispatch_op,
    commands::get_state,
    commands::sst_step,
    commands::bst_step,
    commands::run_stop,
    commands::request_cancel,
    commands::submit_modal,
    commands::cancel_modal,
    commands::submit_modal_with_label,
    commands::tick_time,
    commands::get_prefs,
    commands::set_pref,
])
```
Add `commands::save_state,` to the list (Phase 49).

---

### `hp41-gui/src-tauri/permissions/save-state.toml` (config, new)

**Analog:** `hp41-gui/src-tauri/permissions/tick-time.toml` (exact copy-and-rename):
```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-save-state"
description = "Allows the save_state command."
commands.allow = ["save_state"]
```
Must then add `"allow-save-state"` to `hp41-gui/src-tauri/capabilities/default.json` (RESEARCH Pitfall 7 — atomic pair).

**default.json pattern** (`capabilities/default.json` lines 5–19):
```json
"permissions": [
    "core:default",
    "allow-dispatch-op",
    ...
    "allow-get-prefs",
    "allow-set-pref"
    // Add: "allow-save-state"
]
```

---

### `docs/keyboard-shortcuts.json` (data file, new)

**Analog:** `docs/hp41cv-functions.json` — Vite static JSON-import, build-blocker semantics.

**Vite static import pattern** (`help_data.ts` lines 28–32): JSON file at `docs/` is imported via relative path `../../docs/` from `hp41-gui/src/`. The JSON must be valid at build time — Vite fails the build on malformed JSON (D-25.17).

**Schema** (from RESEARCH.md Pattern 7):
```json
[
    { "key": "Enter",    "op": "ENTER",   "description": "Push X onto stack" },
    { "key": "Tab",      "op": "SHIFT",   "description": "Toggle one-shot SHIFT prefix" },
    { "key": "Ctrl+W",   "op": "WPRGM",   "description": "Card reader: write program" },
    ...
]
```
Each entry has three required string fields: `key`, `op`, `description`. No optional fields needed in Phase 49.

---

### Enriched function JSON files (5 files, modified)

**Analog:** `docs/hp41cv-functions.json` — add optional `example` and `notes` fields to existing entries.

**Existing entry shape** (from `hp41cv-functions.json`, representative entry):
```json
{
    "op_variant": "Plus",
    "display_name": "+",
    "category": "Arithmetic",
    "status": "implemented",
    "phase": "1",
    "key_path": "+",
    "description": "Add: Y + X → X, drop stack"
}
```
Phase 49 enrichment adds two optional fields:
```json
{
    "op_variant": "Plus",
    "display_name": "+",
    "category": "Arithmetic",
    "status": "implemented",
    "phase": "1",
    "key_path": "+",
    "description": "Add: Y + X → X, drop stack",
    "example": "3 ENTER 4 + → 7",
    "notes": "Stack lift enabled after operation; T replicated into Z"
}
```
Entries without enrichment remain unchanged — `example` and `notes` are both optional in the schema and in `HelpEntry` (no JSON schema validation, Vite just checks structural validity).

---

## Shared Patterns

### Tauri IPC invoke pattern
**Source:** `hp41-gui/src/App.tsx` lines 80–107, 269–271, 285–288
**Apply to:** `App.tsx` new save invocation, onboarding prefs read, onboarding_done write
```typescript
// Fire-and-forget (non-critical path):
invoke('set_pref', { key: 'onboarding_done', value: 'true' }).catch(() => {});

// With toast on error:
invoke<void>('save_state')
    .then(() => showToast('Saved'))
    .catch(err => showToast(`Save failed: ${extractErrMessage(err)}`));
```

### Poisoned-lock recovery
**Source:** `hp41-gui/src-tauri/src/commands.rs` (used in every command)
**Apply to:** new `save_state` Tauri command
```rust
state.lock().unwrap_or_else(|e| e.into_inner())
```

### `#[serde(default)]` backward compat
**Source:** `hp41-gui/src-tauri/src/prefs.rs` lines 35–39
**Apply to:** `onboarding_done` field in `GuiPrefs`
```rust
#[serde(default)]
pub onboarding_done: bool,
```
Every new `GuiPrefs` field uses `#[serde(default)]` so older `prefs.json` files load cleanly.

### Vitest mock for Tauri
**Source:** `hp41-gui/src/SettingsPanel.test.tsx` lines 19–21
**Apply to:** `OnboardingWizard.test.tsx`
```typescript
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn().mockResolvedValue(undefined),
}));
```

### CSS custom property variables
**Source:** `hp41-gui/src/App.css` throughout
**Apply to:** all new CSS classes in `App.css`
```css
background: var(--overlay-bg);
color: var(--display-text);
border: 1px solid var(--panel-border);
color: var(--accent);
```
Never hardcode colors — always use the CSS variable system so all 4 themes work.

### Permission TOML + capabilities atomic pair
**Source:** `hp41-gui/src-tauri/permissions/tick-time.toml` + `capabilities/default.json`
**Apply to:** `save-state.toml` + `default.json`
Run `cargo check` first to regenerate `gen/schemas/desktop-schema.json`, then create TOML, then update `default.json` — always as an atomic pair (RESEARCH Pitfall 7).

---

## No Analog Found

None — all files have strong analogs in the codebase.

---

## Critical Implementation Notes for Planner

1. **Ctrl+key must be FIRST in resolveKeyId** (`App.tsx` line 109): Insert before the F7/F8 block. `e.ctrlKey` + `return null` for unrecognized combos ensures no fall-through to the letter MAP.

2. **`__save_state__` interception must be BEFORE `dispatchKeyId`** in `handleKey`: The backend `key_map.rs` will error on an unknown key ID — never let `__save_state__` reach `dispatch_op`.

3. **SectionDef id union widening in HelpOverlay.tsx is a 4-point change**: `SectionDef` interface (line 51), `expanded` state type (line 107), `setExpanded` reset in `useEffect` (line 120), and `toggleSection` parameter (line 185).

4. **Tauri IPC boolean encoding**: `invoke('set_pref', { key: 'onboarding_done', value: 'true' })` — always pass as string `"true"`, not boolean `true`.

5. **F5 = save is a deliberate GUI-CLI divergence**: CLI F5 runs `run_program("A")`; GUI F5 = manual save per D-49.13/KBD-02. Add a code comment noting this divergence.

6. **`save_state` name resolution in commands.rs**: use `persistence::save_state(...)` (module-qualified) inside the command function to avoid shadowing the function being defined.

---

## Metadata

**Analog search scope:** `hp41-gui/src/`, `hp41-gui/src-tauri/src/`, `hp41-gui/src-tauri/permissions/`, `hp41-gui/src-tauri/capabilities/`
**Files scanned:** 12 source files read directly
**Pattern extraction date:** 2026-05-27
