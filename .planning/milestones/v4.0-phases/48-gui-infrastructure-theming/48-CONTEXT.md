# Phase 48: GUI Infrastructure + Theming - Context

**Gathered:** 2026-05-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can personalize the calculator's appearance by switching between four built-in skin themes via a settings panel, with the selected theme persisting across app restarts. This phase also establishes the `prefs.rs` backend and `~/.hp41/prefs.json` preference storage that Phase 49 (onboarding) will reuse.

Zero hp41-core changes. All work is in `hp41-gui/` (Tauri backend + React frontend).

</domain>

<decisions>
## Implementation Decisions

### Theme Selector UX
- **D-48.1:** Gear icon (⚙) in the title bar next to the existing `?` help icon. Clicking opens a popover.
- **D-48.2:** Popover uses a simple text radio-button list — no preview thumbnails. Clicking a radio button applies the theme instantly; the calculator itself IS the preview.
- **D-48.3:** The popover is a general **Settings panel shell** (not theme-only). It ships with a "Theme" section in Phase 48; Phase 49 adds an "Onboarding" section into the same panel.
- **D-48.4:** Popover dismisses on click-outside only — no explicit close button. Consistent with the existing `?` overlay dismiss pattern.

### Color Palette Design
- **D-48.5:** "Classic Beige" theme references the **actual HP-41C/CX body color** — warm beige/tan keyboard background inspired by the real hardware, with a vintage amber-on-olive LCD display look.
- **D-48.6:** High-contrast theme uses **white on black** — pure white text/labels on solid black backgrounds. Target WCAG AAA level.
- **D-48.7:** **Full re-skin** per theme — everything changes: body, display, key fills, key labels, shift/alpha label colors, borders. Each theme feels distinct.
- **D-48.8:** Default theme on first launch is **Dark** (matching current behavior). No OS color-scheme detection.

### CSS Variable Scope
- **D-48.9:** Use **semantic tokens** — ~15–20 CSS custom properties named by purpose (`--calc-bg`, `--display-bg`, `--display-text`, `--key-face`, `--key-label`, `--key-pressed`, `--shift-label`, `--alpha-label`, `--border`, etc.).
- **D-48.10:** SVG keyboard uses **CSS variables for flat fills/strokes**; only gradient stops (which can't use CSS vars in `<defs>`) get passed as React props to `<Keyboard>` per P55. Minimizes prop drilling.
- **D-48.11:** Theme definitions live in a **single new `themes.css` file** with all four `[data-theme]` blocks. `App.css` keeps layout/structure only — hardcoded color values are migrated to CSS variables.

### Theme Transition & Persistence
- **D-48.12:** Theme switch is **instant** — no CSS transition animation. Matches the utilitarian calculator aesthetic.
- **D-48.13:** Theme preference is **persisted to disk immediately** on every change. File writes are tiny (~50 bytes `prefs.json`) and infrequent.

### Claude's Discretion
No areas were deferred to Claude's discretion — all decisions were made explicitly.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Architecture & Persistence
- `docs/adr/` — ADR index (check for GUI-relevant ADRs)
- `hp41-gui/src-tauri/src/persistence.rs` — Existing state persistence pattern (`~/.hp41/autosave.json`); `prefs.rs` must follow the same `dirs::home_dir()` fallback pattern and directory creation

### GUI Structure
- `hp41-gui/src/App.css` — Current CSS styles (all hardcoded hex colors to be migrated to variables)
- `hp41-gui/src/App.tsx` — Main React app component (gear icon + settings panel integration point)
- `hp41-gui/src/Keyboard.tsx` — SVG keyboard component (gradient stop props, `transform-box: fill-box` animation safety)
- `hp41-gui/src/HelpOverlay.tsx` — Existing `?` overlay pattern (reference for popover dismiss behavior)

### Tauri Backend
- `hp41-gui/src-tauri/src/commands.rs` — IPC command pattern (template for `get_prefs` / `set_pref` commands)
- `hp41-gui/src-tauri/src/lib.rs` — Tauri app setup (register new commands here)

### Pitfalls & Constraints
- STATE.md §Pitfalls P51 — SVG animation broken by theme CSS; preserve `transform-box: fill-box`
- STATE.md §Pitfalls P55 — SVG `<defs>` gradient stops ignore CSS vars; pass as React props
- STATE.md §Pitfalls P59 — Theme/onboarding must be in `prefs.json`, never in `autosave.json`
- CLAUDE.md §Frozen Invariants — Tauri v2.11 permission TOML pattern, bundle ID, no-polling rule

### Requirements
- `.planning/REQUIREMENTS.md` §GUI Infrastructure (INFRA-01, INFRA-02) + §Skin Themes (THEME-01..05)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `persistence.rs` (`default_state_path()`, `save_state()`, `load_state()`) — same `dirs::home_dir()` → `~/.hp41/` pattern for `prefs.rs`
- `HelpOverlay.tsx` — click-outside dismiss pattern, popover/overlay rendering
- `commands.rs` — Tauri v2 command patterns (`#[tauri::command]`, `State<Mutex<CalcState>>`) — template for `get_prefs` / `set_pref`

### Established Patterns
- **Vanilla CSS (D-10):** No CSS-in-JS, no Tailwind. All styles in `.css` files. Theme system extends this with CSS custom properties.
- **SVG keyboard with inline calculations:** Key positions computed via constants. Theme colors for gradients must flow as React props.
- **Tauri permission TOMLs:** Each new command needs `hp41-gui/src-tauri/permissions/<cmd-kebab>.toml` + reference in `capabilities/default.json`

### Integration Points
- `App.tsx` title bar area — new gear icon goes next to `?` icon
- `App.tsx` render tree — new `<SettingsPanel>` component
- `main.tsx` — import new `themes.css`
- `lib.rs` — register `get_prefs` and `set_pref` Tauri commands
- `<body data-theme="dark">` — set from loaded prefs on app startup

</code_context>

<specifics>
## Specific Ideas

- Classic Beige should evoke the real HP-41C tan plastic — not a generic warm theme
- High-contrast is white-on-black WCAG AAA, not yellow-on-black or OS-dependent
- The settings panel shell is forward-thinking: Phase 49 will add onboarding controls into it
- The calculator itself is the live preview when switching themes — no need for thumbnail swatches

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 48-GUI Infrastructure + Theming*
*Context gathered: 2026-05-27*
