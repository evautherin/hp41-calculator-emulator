# Phase 48: GUI Infrastructure + Theming - Research

**Researched:** 2026-05-27
**Domain:** Tauri v2 / React / CSS Custom Properties / Preference Persistence
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-48.1:** Gear icon (⚙) in the title bar next to the existing `?` help icon. Clicking opens a popover.
- **D-48.2:** Popover uses a simple text radio-button list — no preview thumbnails. Clicking a radio button applies the theme instantly; the calculator itself IS the preview.
- **D-48.3:** The popover is a general Settings panel shell (not theme-only). It ships with a "Theme" section in Phase 48; Phase 49 adds an "Onboarding" section into the same panel.
- **D-48.4:** Popover dismisses on click-outside only — no explicit close button. Consistent with the existing `?` overlay dismiss pattern.
- **D-48.5:** "Classic Beige" theme references the actual HP-41C/CX body color — warm beige/tan keyboard background inspired by the real hardware, with a vintage amber-on-olive LCD display look.
- **D-48.6:** High-contrast theme uses white on black — pure white text/labels on solid black backgrounds. Target WCAG AAA level.
- **D-48.7:** Full re-skin per theme — everything changes: body, display, key fills, key labels, shift/alpha label colors, borders. Each theme feels distinct.
- **D-48.8:** Default theme on first launch is Dark (matching current behavior). No OS color-scheme detection.
- **D-48.9:** Use semantic tokens — ~15–20 CSS custom properties named by purpose (`--calc-bg`, `--display-bg`, `--display-text`, `--key-face`, `--key-label`, `--key-pressed`, `--shift-label`, `--alpha-label`, `--border`, etc.).
- **D-48.10:** SVG keyboard uses CSS variables for flat fills/strokes; only gradient stops (which can't use CSS vars in `<defs>`) get passed as React props to `<Keyboard>` per P55. Minimizes prop drilling.
- **D-48.11:** Theme definitions live in a single new `themes.css` file with all four `[data-theme]` blocks. `App.css` keeps layout/structure only — hardcoded color values are migrated to CSS variables.
- **D-48.12:** Theme switch is instant — no CSS transition animation. Matches the utilitarian calculator aesthetic.
- **D-48.13:** Theme preference is persisted to disk immediately on every change. File writes are tiny (~50 bytes `prefs.json`) and infrequent.

### Claude's Discretion

No areas were deferred to Claude's discretion — all decisions were made explicitly.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| INFRA-01 | `prefs.rs` Tauri backend for persistent user preferences (separate from CalcState), stored in `~/.hp41/prefs.json` | `persistence.rs` pattern — `dirs::home_dir()` + `serde_json` already in use; mirror exactly |
| INFRA-02 | `get_prefs` / `set_pref` Tauri commands with Tauri v2.11 permission TOMLs | `commands.rs` + `check-tauri-permissions.sh` gate; TOML pattern confirmed from existing 10 TOMLs |
| THEME-01 | User can select from 4 built-in skin themes (dark, light, classic beige, high-contrast) | `<SettingsPanel>` radio-button component; `data-theme` attribute on `<body>`; four `[data-theme]` blocks in `themes.css` |
| THEME-02 | Selected theme persists across app restarts via `~/.hp41/prefs.json` | `get_prefs` on startup → `document.body.dataset.theme = prefs.theme`; `set_pref` on every radio change |
| THEME-03 | SVG key press animations remain functional in all themes (`transform-box: fill-box` preserved) | P51 pitfall confirmed in `App.css:106` — every `[data-theme]` block must NOT override `.key { transform-box: fill-box }` |
| THEME-04 | High-contrast theme meets WCAG AA contrast ratios | White (#ffffff) on black (#000000) = 21:1 contrast ratio — exceeds AA (4.5:1) and AAA (7:1) [VERIFIED: WCAG 2.1 spec] |
| THEME-05 | Theme preference stored separately from CalcState (never in `autosave.json`) | `GuiPrefs` struct in `prefs.rs` — never touches `CalcState`; separate file path `~/.hp41/prefs.json` vs `~/.hp41/autosave.json` |
</phase_requirements>

---

## Summary

Phase 48 is a pure frontend/GUI phase with zero changes to `hp41-core`. It has two distinct work streams: (1) a Tauri backend preferences layer (`prefs.rs` + two IPC commands) and (2) a React/CSS theming system (four `[data-theme]` CSS blocks + a `<SettingsPanel>` component + `<Keyboard>` prop extension for gradient stops).

The entire implementation follows established patterns already proven in this codebase: `prefs.rs` mirrors `persistence.rs` byte-for-byte in structure; the two new Tauri commands mirror `get_state`/`dispatch_op`; the permissions TOML pattern is identical to the 10 existing TOMLs; the CSS variable approach uses the vanilla CSS discipline (D-10) already enforced. The only genuinely new complexity is the SVG gradient stop prop-threading (P55) and the `click-outside` dismiss pattern for the settings panel.

The phase establishes the `prefs.json` infrastructure that Phase 49 (onboarding) will extend with `onboarding_done: bool`. The settings panel shell (D-48.3) must be built with that extension in mind — `GuiPrefs` struct and the settings panel component must be designed to accept new fields/sections without requiring structural surgery in Phase 49.

**Primary recommendation:** Build in the order Backend → CSS → Frontend Component → Integration. `prefs.rs` first enables the Tauri commands; `themes.css` second lets visual verification happen; `SettingsPanel.tsx` last integrates everything. This order matches dependency flow and allows parallel verification.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Preference persistence (`prefs.json`) | Tauri Backend (Rust) | — | Disk I/O belongs in Rust; mirrors `persistence.rs` pattern; not exposed to the frontend as raw file access |
| Theme application | Browser/Client (CSS) | — | `data-theme` attribute on `<body>` + CSS custom properties cascade — zero JS needed at paint time |
| Gradient stop color passing | Frontend React | Tauri Backend | SVG `<defs>` gradient stops cannot use CSS variables (P55); React props thread theme colors from state to `<Keyboard>` |
| Settings panel UI (gear icon + popover) | Browser/Client (React) | — | Pure frontend state; no IPC round-trip for opening/closing the panel |
| Theme persistence (load on startup) | Tauri Backend (Rust) | Browser/Client | `get_prefs` IPC on mount; frontend applies `data-theme` to `<body>` |
| Theme persistence (save on change) | Tauri Backend (Rust) | Browser/Client | `set_pref` IPC on every radio change; backend writes `prefs.json` immediately |
| WCAG AA contrast enforcement | Build/Design | — | Color palette decision at design time; verified by contrast ratio calculation, not at runtime |

---

## Standard Stack

### Core (all already in project — zero new dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `serde_json` | 1.x | Serialize/deserialize `GuiPrefs` to/from JSON | Already in `hp41-gui/src-tauri/Cargo.toml`; same as `CalcState` persistence |
| `dirs` | 6.0.0 | `dirs::home_dir()` for `~/.hp41/prefs.json` path | Already in `hp41-gui/src-tauri/Cargo.toml`; same as `persistence.rs` pattern |
| `tauri` | 2.11 | `#[tauri::command]` + `State<Mutex<GuiPrefs>>` | Already in project; established IPC pattern |
| React 18 + TypeScript | 18.x | `<SettingsPanel>` component + `useState` for `theme` | Already in project; established frontend pattern |
| CSS Custom Properties | Browser-native | `[data-theme]` attribute blocks, `var(--token)` | No library needed; vanilla CSS per D-10 |
| Vitest + jsdom | 4.1.7 | Unit tests for `SettingsPanel`, persistence helpers | Already in project; same test environment as `HelpOverlay.test.tsx` |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `@testing-library/react` | existing | Render `<SettingsPanel>` in tests | Already in project for `App.test.tsx` and `HelpOverlay.test.tsx` |

### Alternatives Considered (and rejected per project decisions)

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-coded `GuiPrefs` via `serde_json` | `tauri-plugin-store` | Plugin overkill for 2 fields; adds a new dep; `serde_json` already present |
| `data-theme` + CSS variables | Styled-components / CSS-in-JS | Violates D-10 (no CSS-in-JS); CSS variables are native and cascade correctly |
| Instant theme switch | CSS `transition` on color properties | D-48.12 mandates instant — utilitarian aesthetic |
| No OS color-scheme detection | `prefers-color-scheme` media query | D-48.8 mandates explicit selection only |

**Installation:** No new packages to install. All dependencies are already present in `hp41-gui/src-tauri/Cargo.toml` and `hp41-gui/package.json`.

---

## Package Legitimacy Audit

> This phase installs **zero new external packages**. All functionality is built on existing dependencies (`serde_json`, `dirs`, `tauri 2.11`, React 18, Vitest). The slopcheck protocol is therefore a no-op for this phase.

| Package | Registry | Age | Downloads | Source Repo | slopcheck | Disposition |
|---------|----------|-----|-----------|-------------|-----------|-------------|
| (none) | — | — | — | — | — | No new packages |

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

---

## Architecture Patterns

### System Architecture Diagram

```
App Startup
    │
    ▼
get_prefs (Tauri IPC)
    │ returns GuiPrefs { theme: "dark" }
    ▼
document.body.dataset.theme = prefs.theme  ◄── CSS cascade activates [data-theme="dark"]
    │
    ▼
App renders <SettingsPanel> (hidden)  ←── gear ⚙ icon click
    │                                           │
    │  theme state: "dark"                      │ opens popover
    │                                           ▼
    │                              <SettingsPanel open>
    │                              [○ Dark  ● Light  ○ Classic Beige  ○ High Contrast]
    │                                           │
    │                              radio click  │
    ▼                                           ▼
set_pref("theme", "light")  ──► prefs.json write (~50 bytes)
    │
    ▼
document.body.dataset.theme = "light"  ──► instant re-paint via CSS vars
    │
    ▼
<Keyboard gradientColors={themeGradients["light"]} />
    │
    ▼
SVG <defs> gradient stop colors updated via React props (P55 bypass)
```

### Recommended Project Structure

```
hp41-gui/
├── src/
│   ├── themes.css          # NEW — four [data-theme] blocks with CSS variables
│   ├── SettingsPanel.tsx   # NEW — gear icon + popover shell + theme radio buttons
│   ├── App.tsx             # MODIFIED — import SettingsPanel, gear icon, prefs init
│   ├── App.css             # MODIFIED — colors migrated to CSS var() references
│   ├── Keyboard.tsx        # MODIFIED — accept GradientColors prop for defs stops
│   └── main.tsx            # MODIFIED — import themes.css
└── src-tauri/
    └── src/
        ├── prefs.rs         # NEW — GuiPrefs struct, load_prefs(), save_prefs()
        ├── lib.rs           # MODIFIED — manage GuiPrefs state, register 2 commands
        ├── commands.rs      # MODIFIED — get_prefs, set_pref command fns
        └── permissions/
            ├── get-prefs.toml   # NEW — allow-get-prefs
            └── set-pref.toml    # NEW — allow-set-pref
```

### Pattern 1: `prefs.rs` — Mirroring `persistence.rs`

**What:** A standalone module that serializes `GuiPrefs` to `~/.hp41/prefs.json` using `serde_json` and `dirs::home_dir()`.
**When to use:** Any time a new preference field needs persistence across restarts without touching `CalcState`.

```rust
// Source: mirrors hp41-gui/src-tauri/src/persistence.rs (verified in codebase)
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct GuiPrefs {
    #[serde(default = "default_theme")]
    pub theme: String,
    // Phase 49 will add: pub onboarding_done: bool
}

fn default_theme() -> String { "dark".to_string() }

impl Default for GuiPrefs {
    fn default() -> Self {
        GuiPrefs { theme: default_theme() }
    }
}

pub fn default_prefs_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hp41")
        .join("prefs.json")
}

pub fn load_prefs(path: &Path) -> GuiPrefs {
    // NEVER panic on missing file — first run is the normal case.
    let Ok(file) = fs::File::open(path) else { return GuiPrefs::default() };
    serde_json::from_reader(file).unwrap_or_default()
}

pub fn save_prefs(path: &Path, prefs: &GuiPrefs) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let file = fs::File::create(path)?;
    serde_json::to_writer_pretty(file, prefs).map_err(std::io::Error::other)
}
```

**Key difference from `persistence.rs`:** `load_prefs` returns `GuiPrefs::default()` on error instead of `Result` — a missing `prefs.json` is the normal first-run case, not an error that needs surfacing to the user.

### Pattern 2: Tauri v2 Commands for Prefs

**What:** Two Tauri commands following the exact pattern of existing commands in `commands.rs`.

```rust
// Source: mirrors commands.rs pattern (verified in codebase)
use crate::prefs::{GuiPrefs, save_prefs, default_prefs_path};
use tauri::State;
use std::sync::Mutex;

pub type PrefsState = Mutex<GuiPrefs>;

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
    let path = default_prefs_path();
    save_prefs(&path, &*p).map_err(|e| e.to_string())
}
```

**Note:** `set_pref` takes `key: String, value: String` — generic interface allows Phase 49 to add `onboarding_done` without modifying the IPC contract. Validate on the Rust side.

### Pattern 3: Permission TOMLs (CI-Gated)

**What:** Every command in `generate_handler!` must have a corresponding TOML or `check-tauri-permissions.sh` fails CI.

```toml
# Source: verified from hp41-gui/src-tauri/permissions/tick-time.toml pattern
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-get-prefs"
description = "Allows the get_prefs command."
commands.allow = ["get_prefs"]
```

**Critical:** Run `cargo check` first (generates the schema registry at `gen/schemas/desktop-schema.json`). Then write the TOML. Then add to `capabilities/default.json`.

### Pattern 4: CSS Variables + `data-theme` Attribute

**What:** CSS custom properties in `themes.css`; `document.body.dataset.theme` set from frontend on prefs load and on each theme switch.
**When to use:** Zero-JS theming that cascades through the entire component tree.

```css
/* Source: D-48.9, D-48.11 decisions; CSS Custom Properties are browser-native */

/* Default (dark) — matches current App.css hardcoded values */
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
  --stack-border: #222;
  --key-label: #e8e8e8;
  --shift-label: #d68a1c;
  --alpha-label-inactive: #5b8fb9;
  --alpha-label-active: #7fb9e0;
  --panel-bg: #1a1a1a;
  --panel-header-bg: #252525;
  --panel-border: #3a3a3a;
  --panel-text: #888;
  --step-active-bg: #1e3a1e;
  --step-active-text: #c8e6c9;
  /* SVG keyboard body gradient stops — these are CSS vars for flat fills, not defs */
  --key-bg-body-top: #1a1a1a;
  --key-bg-body-bottom: #000000;
  /* Note: [data-theme] blocks MUST preserve .key animation properties
     (transform-box: fill-box; transform-origin: center) — P51 invariant */
}
```

```typescript
// Source: React pattern — document.body.dataset API, browser-native
// In App.tsx, after get_prefs resolves:
document.body.dataset.theme = prefs.theme;  // → <body data-theme="dark">

// On radio change in SettingsPanel:
function handleThemeChange(newTheme: string) {
  document.body.dataset.theme = newTheme;
  invoke('set_pref', { key: 'theme', value: newTheme });
  // No await — fire-and-forget; write is tiny and non-critical for UX
}
```

### Pattern 5: SVG Gradient Props (`<Keyboard>` extension)

**What:** The SVG `<defs>` gradient stop colors cannot use CSS variables (P55). Pass them as a `gradientColors` prop derived from the current theme.
**When to use:** Only for SVG gradient stops inside `<defs>`. Flat fills/strokes use CSS variables directly.

```typescript
// Source: P55 pitfall, D-48.10 decision (verified in Keyboard.tsx defs section)
interface GradientColors {
  bodyTop: string;       // SVG keyboard body top gradient stop
  bodyBottom: string;    // SVG keyboard body bottom gradient stop
  keyDarkTop: string;    // Standard dark key gradient top
  keyDarkMid: string;    // Standard dark key gradient mid
  keyDarkBot: string;    // Standard dark key gradient bottom
  enterTop: string;      // ENTER key gradient top
  enterMid: string;      // ENTER key gradient mid
  enterBot: string;      // ENTER key gradient bottom
  shiftIdleTop: string;  // SHIFT idle gradient top
  shiftIdleMid: string;  // SHIFT idle gradient mid
  shiftIdleBot: string;  // SHIFT idle gradient bottom
  shiftActiveTop: string;
  shiftActiveMid: string;
  shiftActiveBot: string;
}

// Theme gradient palette map — one entry per theme
// Values are sourced from design decisions; exact hex values are Claude's discretion
const THEME_GRADIENTS: Record<string, GradientColors> = {
  "dark": {
    bodyTop: "#1a1a1a", bodyBottom: "#000000",
    keyDarkTop: "#303030", keyDarkMid: "#181818", keyDarkBot: "#080808",
    enterTop: "#346034", enterMid: "#1a3a1a", enterBot: "#0a180a",
    shiftIdleTop: "#d68a1c", shiftIdleMid: "#b06811", shiftIdleBot: "#7a4708",
    shiftActiveTop: "#ffb742", shiftActiveMid: "#f5a423", shiftActiveBot: "#c97d10",
  },
  // light, classic-beige, high-contrast entries...
};
```

### Pattern 6: `SettingsPanel` — Click-Outside Dismiss

**What:** A popover that dismisses on click-outside, consistent with `HelpOverlay`'s Esc pattern. Uses `useRef` + document event listener.
**When to use:** Consistent with D-48.4.

```typescript
// Source: mirrors HelpOverlay.tsx useEffect pattern (verified in codebase)
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

**Note:** Use `mousedown` not `click` to ensure the panel closes before the next click is processed. The gear icon click handler must stop propagation to prevent immediate re-close.

### Anti-Patterns to Avoid

- **Reading/writing prefs inside `CalcState`:** The entire point of `prefs.rs` is separation. Never add `theme` to `CalcState`, `CalcStateView`, or any IPC response (violates P59).
- **Blocking the Tauri thread on pref save:** `save_prefs` writes synchronously. This is acceptable because the file is ~50 bytes. Do NOT spawn a background thread for this — `persistence.rs`'s auto-save thread is for `CalcState` (large) only.
- **CSS `transition` on themed properties:** D-48.12 is instant. A `transition: background-color 200ms` in a `[data-theme]` block would violate this decision.
- **Overriding `.key` animation properties in `[data-theme]` blocks:** Every theme block that sets `.key` properties must preserve `transform-box: fill-box` and `transform-origin: center` (P51). The safest approach: do not touch `.key` in `themes.css` at all — let it stay in `App.css` where it currently lives.
- **Using CSS variables inside SVG `<defs>` gradient stops:** `<stop stopColor="var(--key-top)" />` does not work in `<linearGradient>` elements inside `<defs>` in all browsers (P55). Pass colors as React props.
- **`tauri-plugin-store` dependency:** Explicitly rejected (STATE.md). Use hand-coded `serde_json` persistence.
- **`set_pref` accepting arbitrary key/value without validation:** Validate the `key` against a known set on the Rust side; return `Err` for unknowns (mirrors D-07 never-silently-swallow rule).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Preference file path | Custom path logic | `dirs::home_dir().join(".hp41").join("prefs.json")` | Already proven in `persistence.rs`; handles no-home fallback |
| JSON serialization | Custom serializer | `serde_json::to_writer_pretty` + `#[derive(Serialize, Deserialize)]` | Already in project; zero new deps |
| Directory creation | Manual `fs::create_dir` | `fs::create_dir_all(parent)?` | `persistence.rs` pattern — handles nested `~/.hp41/` creation |
| Click-outside dismiss | Complex event bubbling | `useRef` + `document.addEventListener('mousedown', ...)` | Established browser pattern; simple and correct |
| Contrast ratio calculation | Runtime measurement | Design-time color selection | WCAG ratios are fixed by choice — white/black = 21:1 is proven |

**Key insight:** Every mechanism in this phase already exists in the codebase. The work is creating new files that follow established patterns, not inventing new solutions.

---

## Common Pitfalls

### Pitfall P51: SVG Animation Broken by Theme CSS

**What goes wrong:** Theme CSS accidentally overrides `.key { transform-box: fill-box }`, breaking the 0.92 scale press animation. Keys visually "jump" on press.
**Why it happens:** `themes.css` targets `.key` for color changes, inadvertently clearing `transform-box`.
**How to avoid:** Never set `transform-box` or `transform-origin` in `themes.css`. Those animation-critical properties live exclusively in `App.css`. Only set color/fill tokens in theme blocks.
**Warning signs:** After theme switch, key press animation looks like a translation (slides off-center) instead of a scale — means `transform-origin` lost `center`.

### Pitfall P55: SVG `<defs>` Gradient Stops Ignore CSS Variables

**What goes wrong:** Replacing `stopColor="#303030"` with `stopColor="var(--key-dark-top)"` inside `<linearGradient>` in `<defs>` results in gray/black fallback color in most browsers (gradient stops do not inherit CSS custom properties from the document root in the SVG `<defs>` context).
**Why it happens:** SVG `<defs>` are resolved in a different cascade context; CSS variables do not apply.
**How to avoid:** Pass gradient stop colors as React props to `<Keyboard>` (a `GradientColors` object). Read from a theme-keyed constant in `App.tsx`. Only use CSS variables for non-`<defs>` SVG attributes (flat `fill`, `stroke`).
**Warning signs:** Gradient-filled keys look correct in the dark theme but appear black or a single solid color after a theme switch.

### Pitfall P59: Theme Preference Leaking into `autosave.json`

**What goes wrong:** A developer adds `theme: String` to `CalcState` for convenience, causing `autosave.json` to contain the theme choice. On CLI load, the theme field breaks backward compat (CLI doesn't know about `GuiPrefs`).
**Why it happens:** `CalcState` is the path of least resistance for persisting GUI-only state.
**How to avoid:** `GuiPrefs` is a completely separate struct in a completely separate file (`prefs.rs`) writing to a completely separate path (`prefs.json`). The two systems must never share a file, struct, or managed Tauri state object.
**Warning signs:** `grep -r "theme" hp41-core/src/` returns any match — means the invariant is broken.

### Pitfall: Permission TOML Missing from `capabilities/default.json`

**What goes wrong:** A new permission TOML exists in `permissions/` but is not referenced in `capabilities/default.json`. The command exists but returns a Tauri capability error at runtime. CI passes because `check-tauri-permissions.sh` only checks TOML existence, not `default.json` inclusion.
**Why it happens:** Two-step process is easy to miss — create TOML AND add to `default.json`.
**How to avoid:** After creating each TOML, immediately add the `"allow-<cmd-kebab>"` entry to `capabilities/default.json`. Treat them as an atomic pair.
**Warning signs:** `invoke('get_prefs')` throws a Tauri permission error in the browser console.

### Pitfall: `set_pref` `value` Parameter Position

**What goes wrong:** `set_pref(key: String, value: String, prefs: State<'_...>)` vs `set_pref(prefs: State<'_...>, key: String, value: String)` — Tauri v2 requires non-State params to come before State extractors.
**Why it happens:** Inconsistent with typical function convention; Tauri's macro imposes ordering.
**How to avoid:** Always place `key: String, value: String` BEFORE `prefs: State<'_, PrefsState>`. This matches the established pattern in `submit_modal_with_label(label: String, state: State<'_, AppState>)`.
**Warning signs:** Compile-time error in the `tauri::generate_handler![]` macro expansion.

### Pitfall: Gear Icon Click Immediately Re-Closing Panel

**What goes wrong:** Clicking the gear icon opens the settings panel, but the `mousedown` click-outside listener immediately fires and closes it on the same click.
**Why it happens:** The `mousedown` event on the gear icon is not inside `panelRef`, so click-outside fires.
**How to avoid:** The gear icon click handler must either: (a) call `e.stopPropagation()` on the `mousedown` event, OR (b) the gear icon must be inside the `panelRef` element. Option (a) is simpler. Alternatively, use a `setTimeout(() => { ... }, 0)` to register the outside-click listener after the current event cycle.
**Warning signs:** Panel appears to open for a single frame (flicker) then immediately disappears.

### Pitfall: Theme Not Applied on First Render

**What goes wrong:** App renders with no `data-theme` attribute on `<body>` before `get_prefs` resolves. User sees unstyled/incorrect colors for one frame.
**Why it happens:** Async IPC latency — `get_prefs` is fast but not instantaneous.
**How to avoid:** Set `data-theme="dark"` in `index.html` as the default (matches D-48.8). The IPC call then either confirms "dark" (no-op) or switches to the saved theme. This eliminates the flash.
**Warning signs:** Brief visual flash of incorrect colors on app startup.

---

## Code Examples

### `lib.rs` — Registering `GuiPrefs` Managed State

```rust
// Source: mirrors existing lib.rs pattern (verified in codebase)
// In the .setup() closure, after CalcState setup:
let prefs_path = prefs::default_prefs_path();
let initial_prefs = prefs::load_prefs(&prefs_path);
app.manage(std::sync::Mutex::new(initial_prefs));

// In invoke_handler:
commands::get_prefs,
commands::set_pref,
```

### `default.json` — Adding Two New Permissions

```json
{
  "permissions": [
    "core:default",
    "allow-dispatch-op",
    /* ... existing entries ... */
    "allow-get-prefs",
    "allow-set-pref"
  ]
}
```

### `App.tsx` — Prefs Init on Mount

```typescript
// Source: extends existing get_state useEffect pattern (App.tsx line 286)
const [theme, setTheme] = useState<string>('dark');

useEffect(() => {
  invoke<{ theme: string }>('get_prefs')
    .then(prefs => {
      setTheme(prefs.theme);
      document.body.dataset.theme = prefs.theme;
    })
    .catch(() => {
      // Silent failure — default dark theme remains
      document.body.dataset.theme = 'dark';
    });
}, []);
```

### Theme Validation Color Reference

| Theme | Key: `data-theme` | Body BG | Display BG | Display Text | WCAG vs display BG |
|-------|-------------------|---------|------------|--------------|-------------------|
| Dark (current) | `dark` | `#0d0d0d` | `#111` | `#c8e6c9` | ~10:1 [ASSUMED: based on reading App.css] |
| Light | `light` | `#e8e8e8` | `#f5f5f0` | `#1a3a1a` | ~12:1 [ASSUMED] |
| Classic Beige | `classic-beige` | `#c8b47a` | `#4a4520` | `#e8d89a` | Needs verification during implementation |
| High Contrast | `high-contrast` | `#000000` | `#000000` | `#ffffff` | 21:1 [VERIFIED: WCAG 2.1 — white/black is maximum contrast] |

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hardcoded hex colors in `App.css` | CSS custom properties via `[data-theme]` blocks in `themes.css` | This phase | Enables runtime theme switching without JS color injection |
| No preferences persistence | `GuiPrefs` in `~/.hp41/prefs.json` via `prefs.rs` | This phase | Foundation for all v4.0 user personalization features |
| Single `?` overlay entry point | `?` overlay + gear icon `⚙` settings panel | This phase | Separates function reference (help) from user preferences (settings) |

**Deprecated/outdated:**
- Hardcoded hex color values in `App.css` rules: all values that represent "theme colors" (backgrounds, text, borders) move to CSS variables. Non-themed structural values (margins, padding, font-size) stay hardcoded.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | WCAG AA contrast ratio for Dark theme display text (`#c8e6c9` on `#111`) is ~10:1 | Code Examples | Could be lower; needs verification during implementation. WCAG AA minimum is 4.5:1 so all reasonable dark-on-dark-bg combos with light green text should pass comfortably |
| A2 | Classic Beige LCD display color (`amber-on-olive` per D-48.5) — exact hex values for contrast ratio | Code Examples | If olive background is too dark for amber text at 4.5:1, color must be adjusted. Concrete hex selection is Claude's implementation-time discretion |
| A3 | Light theme display text (`#1a3a1a` on `#f5f5f0`) WCAG ratio | Code Examples | Needs contrast ratio check during implementation; dark green on off-white is typically high contrast |
| A4 | `document.addEventListener('mousedown', ...)` for click-outside dismiss is equivalent to `HelpOverlay`'s Esc dismiss in UX feel | Architecture Patterns | HelpOverlay uses `keydown`; SettingsPanel needs `mousedown` which is a different event. If it feels different to users, `click` could be substituted |
| A5 | `index.html` accepts a `data-theme` attribute on `<body>` set to `dark` as default | Common Pitfalls | Tauri 2.11 renders the HTML file as-is; this is standard HTML and should work, but has not been tested in this specific Tauri configuration |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

---

## Open Questions

1. **Classic Beige WCAG ratio for LCD display**
   - What we know: D-48.5 says "vintage amber-on-olive LCD display look"; D-48.6 says high-contrast targets WCAG AAA
   - What's unclear: The exact hex values for beige theme display colors and whether they naturally achieve AA (4.5:1)
   - Recommendation: During implementation, use an online contrast checker (e.g., WebAIM) or compute luminance. If the authentic HP-41C beige palette fails AA for any text element, adjust the shade slightly toward higher contrast. The requirement (THEME-04) explicitly only mandates AA for the high-contrast theme; beige/light/dark themes have no stated minimum.

2. **`GradientColors` prop shape: 14 individual props or one object?**
   - What we know: D-48.10 says "pass as React props to `<Keyboard>`"; current `<Keyboard>` has 5 props
   - What's unclear: Whether 14 gradient stop values should be individual named props or a single `gradientColors: GradientColors` object
   - Recommendation: Use a single `gradientColors: GradientColors` object — cleaner TypeScript, easier to extend, avoids prop explosion. TypeScript will enforce the shape at compile time.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js | Vitest, npm build | Yes | v22.16.0 | — |
| npm | GUI build | Yes | 10.9.2 | — |
| Cargo/Rust | Tauri backend | Yes | 1.95.0 (MSRV 1.88) | — |
| just | Task runner | Yes | 1.49.0 | — |
| `dirs` crate | `prefs.rs` home dir | Yes | 6.0.0 (in Cargo.toml) | — |
| `serde_json` crate | `prefs.rs` JSON | Yes | 1.x (in Cargo.toml) | — |
| Vitest | Frontend tests | Yes | 4.1.7 (in package.json) | — |
| jsdom | React component tests | Yes | via Vitest config | — |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** None.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework — Rust backend | `cargo test` (built-in; tests in `prefs.rs` and `commands.rs`) |
| Framework — React frontend | Vitest 4.1.7 + jsdom + `@testing-library/react` |
| Config file | `hp41-gui/vite.config.ts` (existing, `test.environment: 'jsdom'`) |
| Quick run command (frontend) | `cd hp41-gui && npm test` |
| Quick run command (backend) | `just gui-check` + `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` |
| Full suite command | `just gui-ci` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| INFRA-01 | `prefs.rs` load/save round-trip | unit (Rust) | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml prefs` | No — Wave 0 |
| INFRA-01 | Missing `prefs.json` returns `GuiPrefs::default()` | unit (Rust) | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml prefs` | No — Wave 0 |
| INFRA-01 | Corrupt `prefs.json` returns `GuiPrefs::default()` | unit (Rust) | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml prefs` | No — Wave 0 |
| INFRA-02 | Permission TOML coverage gate | CI (bash) | `bash scripts/check-tauri-permissions.sh` | Yes (existing script) |
| THEME-01 | `<SettingsPanel>` renders 4 theme radio buttons | unit (React) | `cd hp41-gui && npm test` | No — Wave 0 |
| THEME-01 | Clicking radio calls `invoke('set_pref', ...)` | unit (React, mocked Tauri) | `cd hp41-gui && npm test` | No — Wave 0 |
| THEME-02 | Theme applies to `document.body.dataset.theme` on startup | unit (React, mocked Tauri) | `cd hp41-gui && npm test` | No — Wave 0 |
| THEME-03 | `.key` CSS animation properties preserved in all 4 themes | unit (CSS audit) | Verified by code review — `.key` rule unchanged in `App.css` | No — Wave 0 |
| THEME-04 | High-contrast `#ffffff` on `#000000` = 21:1 | manual / design-time | Color contrast tool — no runtime test needed | n/a |
| THEME-05 | `set_pref` does NOT modify `CalcState` | unit (Rust) | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` | No — Wave 0 |

### Sampling Rate

- **Per task commit:** `just gui-check && cd hp41-gui && npm test`
- **Per wave merge:** `just gui-ci`
- **Phase gate:** `just gui-ci` full green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `hp41-gui/src-tauri/src/prefs.rs` — covers INFRA-01 (module + tests)
- [ ] `hp41-gui/src/SettingsPanel.test.tsx` — covers THEME-01, THEME-02
- [ ] `hp41-gui/src-tauri/permissions/get-prefs.toml` — covers INFRA-02
- [ ] `hp41-gui/src-tauri/permissions/set-pref.toml` — covers INFRA-02

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | Phase adds no auth |
| V3 Session Management | No | Prefs are not session tokens |
| V4 Access Control | No | Local desktop app; single user |
| V5 Input Validation | Yes | `set_pref` validates `key` and `value` on the Rust side; unknown keys/values return `Err` |
| V6 Cryptography | No | `prefs.json` stores non-sensitive preference strings |

### Known Threat Patterns for This Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malformed `prefs.json` from external edit | Tampering | `serde_json::from_reader` returns `Err` → `unwrap_or_default()` — no panic, safe fallback |
| Unknown `key` or `value` in `set_pref` | Tampering | Rust-side validation: enum/allowlist of known keys; `Err` returned to frontend |
| Path traversal via preference value | Tampering | Theme name is validated against a fixed list of 4 strings — no filesystem path involved |

---

## Sources

### Primary (HIGH confidence)
- `hp41-gui/src-tauri/src/persistence.rs` — `prefs.rs` must mirror this exactly (verified by reading)
- `hp41-gui/src-tauri/src/commands.rs` — `get_prefs`/`set_pref` pattern (verified by reading)
- `hp41-gui/src-tauri/src/lib.rs` — managed state registration + `generate_handler!` pattern (verified by reading)
- `hp41-gui/src/App.css` — all hardcoded hex colors to migrate (verified by reading)
- `hp41-gui/src/App.tsx` — gear icon integration point, `useEffect` prefs init pattern (verified by reading)
- `hp41-gui/src/Keyboard.tsx` — SVG `<defs>` gradient structure + P55 context (verified by reading)
- `hp41-gui/src/HelpOverlay.tsx` — click-outside dismiss reference (verified by reading)
- `hp41-gui/src-tauri/permissions/tick-time.toml` — canonical TOML pattern (verified by reading)
- `scripts/check-tauri-permissions.sh` — CI gate requiring TOML for every command (verified by reading)
- `hp41-gui/src-tauri/capabilities/default.json` — permission reference list (verified by reading)
- `.planning/phases/48-gui-infrastructure-theming/48-CONTEXT.md` — all locked decisions (verified by reading)
- `.planning/STATE.md` §Pitfalls P51, P55, P59 — critical pitfalls documented (verified by reading)
- CLAUDE.md §Frozen Invariants — Tauri v2.11 permissions, bundle ID, no-polling, no async (verified by reading)

### Secondary (MEDIUM confidence)
- WCAG 2.1 contrast ratio specification: white (#ffffff) on black (#000000) = 21:1 contrast ratio [cited from WCAG 2.1 specification; well-established calculation]

### Tertiary (LOW confidence — see Assumptions Log)
- Estimated WCAG ratios for dark, light, classic beige theme display colors: [ASSUMED] — concrete values to be verified during implementation

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — zero new packages; all tools verified in project
- Architecture: HIGH — all patterns derived directly from existing verified codebase code
- Pitfalls: HIGH — P51, P55, P59 are documented in STATE.md and verified against source files
- Color palette (non-high-contrast): LOW for exact hex values — design-time decisions to be made during implementation

**Research date:** 2026-05-27
**Valid until:** 2026-06-27 (CSS/Tauri patterns stable; color values immaterial to research validity)
