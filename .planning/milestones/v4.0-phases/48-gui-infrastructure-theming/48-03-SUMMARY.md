---
phase: 48-gui-infrastructure-theming
plan: "03"
subsystem: hp41-gui/src
tags: [react, theming, settings-panel, svg-gradients, preferences, accessibility]
dependency_graph:
  requires:
    - "48-01 (get_prefs/set_pref Tauri IPC)"
    - "48-02 (themes.css four [data-theme] blocks, App.css CSS vars)"
  provides:
    - "SettingsPanel component with 4 theme radio buttons and click-outside dismiss"
    - "GradientColors interface and THEME_GRADIENTS map for SVG stop threading"
    - "Theme state wired end-to-end: load on mount -> apply instantly -> persist via IPC"
    - "No unstyled flash on startup (data-theme=dark default in index.html)"
  affects:
    - "hp41-gui/src/App.tsx (theme state, gear icon, prefs mount effect)"
    - "hp41-gui/src/Keyboard.tsx (gradientColors prop threading)"
    - "hp41-gui/src/main.tsx (themes.css import)"
    - "hp41-gui/index.html (data-theme=dark default)"
tech_stack:
  added: []
  patterns:
    - "React useRef + document.addEventListener(mousedown) for click-outside dismiss (D-48.4)"
    - "SVG gradient stop values passed as React props — CSS var() not usable in <defs> (D-48.10 / P55)"
    - "Fire-and-forget invoke pattern for set_pref (D-48.13)"
    - "onMouseDown + e.stopPropagation() on gear button to prevent immediate re-close"
key_files:
  created:
    - hp41-gui/src/SettingsPanel.tsx
    - hp41-gui/src/SettingsPanel.test.tsx
  modified:
    - hp41-gui/src/Keyboard.tsx
    - hp41-gui/src/App.tsx
    - hp41-gui/src/main.tsx
    - hp41-gui/index.html
    - hp41-gui/src/App.css
decisions:
  - "Title bar div added above annunciators row to house gear + help icon buttons"
  - "help-icon-btn added for mouse users — keyboard shortcut ? still works"
  - "node_modules symlink created temporarily in worktree for test execution, removed before commit"
  - "THEME_GRADIENTS body gradient values derived from --panel-bg (beige) and --panel-header-bg (light) for visual coherence"
metrics:
  duration: "18 minutes"
  completed: "2026-05-27T12:09:00Z"
  tasks_completed: 3
  tasks_total: 4
  files_created: 2
  files_modified: 5
  tests_added: 6
---

# Phase 48 Plan 03: Frontend Theme Integration Summary

React frontend integration: SettingsPanel component with gear icon trigger, four theme radio buttons, click-outside dismiss, Keyboard SVG gradient prop threading for all four themes, and preference persistence via get_prefs/set_pref IPC on mount/change.

---

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Create SettingsPanel component and Keyboard gradient props | 85332c8 | SettingsPanel.tsx (new), Keyboard.tsx |
| 2 | Integrate theme state into App.tsx, wire index.html and main.tsx | 6da6bdd | App.tsx, main.tsx, index.html, App.css |
| 3 | Write SettingsPanel unit tests | 2edadaf | SettingsPanel.test.tsx (new) |

---

## What Was Built

### Task 1: SettingsPanel.tsx + Keyboard.tsx gradient props

**SettingsPanel.tsx** — New component with:
- `SettingsPanelProps` type (`open`, `onClose`, `currentTheme`, `onThemeChange`)
- `useRef<HTMLDivElement>` + `document.addEventListener('mousedown', ...)` for click-outside dismiss (D-48.4)
- Listener registered only when `open` is true; cleaned up on unmount/close
- Returns `null` when `!open` (matches HelpOverlay early-return pattern)
- `THEMES` constant array with 4 entries: Dark, Light, Classic Beige, High Contrast
- `<div role="dialog" aria-label="Settings">` wrapping a `<section>` with `<h3>` heading and radio list
- Phase 49 shell comment for Onboarding section (D-48.3)

**Keyboard.tsx additions:**
- `GradientColors` interface with 14 fields: bodyTop/Bottom, keyDarkTop/Mid/Bot, enterTop/Mid/Bot, shiftIdleTop/Mid/Bot, shiftActiveTop/Mid/Bot
- `DARK_GRADIENT_COLORS` constant matching current hardcoded hex values exactly (zero visual regression)
- `THEME_GRADIENTS` map with entries for all 4 themes using values from 48-UI-SPEC.md
- `gradientColors?: GradientColors` added to `KeyboardProps` (defaults to `DARK_GRADIENT_COLORS`)
- All `<defs>` gradient `stopColor` attributes replaced with `gradientColors.*` prop values

### Task 2: App.tsx / main.tsx / index.html / App.css wiring

**index.html** — `data-theme="dark"` added to `<body>` tag to prevent unstyled flash on startup (D-48.8)

**main.tsx** — `import './themes.css'` added after `import './index.css'` (D-48.11 specificity order)

**App.tsx:**
- Imports `THEME_GRADIENTS` from `./Keyboard` and `SettingsPanel` from `./SettingsPanel`
- `theme` state (`useState<string>('dark')`) and `settingsOpen` state
- `get_prefs` mount useEffect applies persisted theme to `document.body.dataset.theme`
- `handleThemeChange` callback: sets state, applies data-theme, fire-and-forget `set_pref`
- Calculator title bar div added above annunciators with help icon button (`?`) and gear icon button (`⚙`)
- Gear button uses `onMouseDown` + `e.stopPropagation()` to prevent click-outside re-close
- `aria-label="Open settings"` and `aria-expanded={settingsOpen}` on gear button
- `<SettingsPanel>` rendered in title bar with full prop wiring
- Mutual exclusion: opening help closes settings, opening settings closes help
- `gradientColors={THEME_GRADIENTS[theme] || THEME_GRADIENTS['dark']}` passed to `<Keyboard>`

**App.css** — `.calculator-title-bar` and `.help-icon-btn` rules added for the new title bar area

### Task 3: SettingsPanel.test.tsx

6 unit tests in `describe('SettingsPanel')`:
1. Renders null when `open=false`
2. Renders 4 radio buttons when `open=true`
3. Dark radio is checked when `currentTheme="dark"`
4. Calls `onThemeChange("light")` when light radio is clicked
5. Contains "Dark", "Light", "Classic Beige", "High Contrast" in text content
6. Panel has `role="dialog"` and `aria-label="Settings"`

All 6 tests pass. Tauri invoke mocked via `vi.mock('@tauri-apps/api/core')`.

---

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] Added help icon button in title bar**
- **Found during:** Task 2
- **Issue:** The plan specifies gear icon "next to the existing `?` icon", but the existing `?` is keyboard-only (no visible button in the UI). Mouse users have no discoverability for the function reference.
- **Fix:** Added a `?` button (`.help-icon-btn`) in the title bar alongside the gear icon. The keyboard shortcut still works. This makes both panels discoverable via mouse without requiring the keyboard shortcut.
- **Files modified:** App.tsx, App.css
- **Commit:** 6da6bdd

**2. [Rule 3 - Blocking issue] node_modules symlink for test execution in worktree**
- **Found during:** Task 3
- **Issue:** The worktree's `hp41-gui` directory has no `node_modules` — they live in the main repo's `hp41-gui`. Vitest failed to start from the worktree path.
- **Fix:** Created a temporary symlink `worktree/hp41-gui/node_modules -> main-repo/hp41-gui/node_modules` for test execution. Symlink removed before committing (the `node_modules/` glob in root `.gitignore` covers it, but removing it prevents any confusion).
- **Files modified:** None (symlink removed, not committed)
- **Commit:** N/A — symlink not committed

---

## Known Stubs

None. All 4 themes have complete gradient stop values. The SettingsPanel renders real radio buttons with real theme labels. The theme switching is fully wired: data-theme attribute, gradient prop, and IPC persistence.

---

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| trust boundary: IPC->DOM | App.tsx handleThemeChange | theme name from IPC response applied to body.dataset.theme — covered by T-48-06 (accept: CSS-only, no security behavior) and server-side allowlist in set_pref |

No new threat surfaces beyond the plan's registered threats (T-48-06, T-48-07, T-48-SC all remain accept).

---

## Checkpoint: Task 4 (Visual Verification)

Task 4 is a `checkpoint:human-verify` — execution paused here. The orchestrator will present verification steps to the user.

**What was built:**
Complete theme switching system — gear icon (⚙) in the calculator title bar opens a settings panel with 4 theme radio buttons (Dark, Light, Classic Beige, High Contrast). Clicking a radio instantly applies the theme across the entire calculator including SVG keyboard gradients. Theme persists in `~/.hp41/prefs.json` across app restarts.

**Verification steps for user:**
1. Run `just gui-dev` to start the app
2. Verify calculator renders normally (dark theme, current behavior)
3. Click the ⚙ gear icon in the title bar (next to the ? icon)
4. Verify settings panel popover appears with "Theme" heading and 4 radio options
5. Click "Light" — verify instant full re-skin (light backgrounds, dark text, green LCD)
6. Click "Classic Beige" — verify warm tan/beige body, olive-amber LCD, vintage feel
7. Click "High Contrast" — verify pure white text on black backgrounds
8. Click "Dark" — verify return to original dark appearance
9. In each theme, click a calculator key and verify press animation works
10. Click outside the settings panel — verify it closes
11. Close and reopen the app — verify last selected theme is still active
12. Verify gear icon hover state changes color

---

## Self-Check: PASSED

- SettingsPanel.tsx: EXISTS at hp41-gui/src/SettingsPanel.tsx
- SettingsPanel.test.tsx: EXISTS at hp41-gui/src/SettingsPanel.test.tsx
- Keyboard.tsx: MODIFIED with GradientColors + THEME_GRADIENTS
- App.tsx: MODIFIED with theme state, gear icon, SettingsPanel render
- main.tsx: MODIFIED with themes.css import
- index.html: MODIFIED with data-theme=dark
- App.css: MODIFIED with calculator-title-bar and help-icon-btn rules
- SUMMARY.md: EXISTS at .planning/phases/48-gui-infrastructure-theming/48-03-SUMMARY.md
- Task 1 commit 85332c8: EXISTS (git log verified)
- Task 2 commit 6da6bdd: EXISTS (git log verified)
- Task 3 commit 2edadaf: EXISTS (git log verified)
- `npm run build`: PASSED (29 modules, 305 kB JS)
- All 6 SettingsPanel tests: PASSED
- All Task 1-3 acceptance criteria: PASSED
