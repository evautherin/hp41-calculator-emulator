---
phase: 48-gui-infrastructure-theming
plan: "02"
subsystem: hp41-gui/src
tags: [css, themes, dark-mode, accessibility, wcag]
completed: "2026-05-27"
duration_minutes: 4
tasks_completed: 2
tasks_total: 2
files_created: 1
files_modified: 1

dependency_graph:
  requires: []
  provides:
    - "hp41-gui/src/themes.css: four complete [data-theme] CSS variable blocks"
    - "hp41-gui/src/App.css: structural CSS with var(--token) color references"
  affects:
    - "hp41-gui/src/main.tsx: must import themes.css (Plan 03)"
    - "hp41-gui/src/App.tsx: must set data-theme on body (Plan 03)"

tech_stack:
  added: []
  patterns:
    - "CSS custom properties + data-theme attribute pattern (D-48.9, D-48.11)"
    - "Self-contained per-theme variable blocks with no inter-theme inheritance (D-48.7)"

key_files:
  created:
    - hp41-gui/src/themes.css
  modified:
    - hp41-gui/src/App.css

decisions:
  - "All 39 CSS variables defined per theme (including annunciator-active, annunciator-shift-active, step-active-bg, step-active-text beyond the base ~35 from UI-SPEC)"
  - "P51 comment text in themes.css reworded to avoid grep false-positives on transform-box/transform-origin"
  - "help-overlay-section-heading:hover uses var(--accent) + opacity:0.85 instead of hardcoded #ffc107 (consistent theme adaptation)"
  - "print-panel-close:hover uses var(--text-secondary) instead of hardcoded #ccc (structural hover now theme-aware)"
  - "step-row border migrated to var(--panel-border) from #2a2a2a (semantic alignment, slight color shift in dark theme: #3a3a3a vs #2a2a2a)"
  - "print-line color migrated to var(--text-secondary) from #c8c8c8 (semantic: print output = secondary text)"
---

# Phase 48 Plan 02: CSS Theme System Summary

**One-liner:** Four self-contained `[data-theme]` CSS variable blocks in `themes.css` with full App.css color migration to `var(--token)` references enabling zero-JS runtime theme switching.

---

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Create themes.css with four complete data-theme blocks | 135c82d | hp41-gui/src/themes.css (created) |
| 2 | Migrate App.css hardcoded colors to CSS variable references | 5fb8bab | hp41-gui/src/App.css (modified) |

---

## What Was Built

### Task 1: themes.css

Created `hp41-gui/src/themes.css` with four fully self-contained `[data-theme]` blocks:
- `[data-theme="dark"]` — matches current App.css hex values exactly (zero visual regression)
- `[data-theme="light"]` — light grey/green palette
- `[data-theme="classic-beige"]` — authentic HP-41C beige body color (#c8b890)
- `[data-theme="high-contrast"]` — WCAG AAA white-on-black (#ffffff on #000000, 21:1 contrast)

Each block defines 39 CSS custom properties — no inheritance between blocks (D-48.7 full re-skin). No `transform-box`, `transform-origin`, or `transition` properties anywhere in the file (P51 + D-48.12 enforced).

### Task 2: App.css migration

All theme-dependent hardcoded hex colors replaced with `var(--token)` references:
- 61 `var(--` usages in App.css (previously 0)
- Zero remaining hardcoded hex colors for theme-dependent UI surfaces
- P51 invariant preserved: `.key { transform-box: fill-box; transform-origin: center; }` unchanged
- Added SettingsPanel CSS rules: `.settings-gear-btn`, `.settings-panel`, `.settings-section-heading`, `.settings-radio-row`
- `npm run build` succeeds (valid CSS + TypeScript compilation)

---

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] P51 check grep false-positive on comment text**
- **Found during:** Task 1 verification
- **Issue:** The plan's acceptance criteria specifies `grep -c 'transform-box\|transform-origin' hp41-gui/src/themes.css` returns 0. Initial comment text "DO NOT set transform-box or transform-origin in any [data-theme] block" would cause grep to return 1 (matching the comment).
- **Fix:** Reworded the P51 warning comment to avoid literal property names: "The SVG animation properties (fill-box / center transform) must remain in App.css under .key only"
- **Files modified:** hp41-gui/src/themes.css
- **Commit:** 135c82d

### Minor Color Decisions (within-spec adaptations)

**2. Additional variables beyond plan's base list**
- Added `--annunciator-active`, `--annunciator-shift-active`, `--step-active-bg`, `--step-active-text` to all four theme blocks (required by migration table in plan interfaces section).
- These were in the plan's "additional variables" note and the color migration table.

**3. `help-overlay-section-heading:hover` hover color**
- Plan's App.css had `color: #ffc107` (slightly lighter orange) for hover. Migrated to `var(--accent)` with `opacity: 0.85` to maintain theme-awareness across all four themes.
- Original hardcoded hover color would look wrong in light/classic-beige/high-contrast themes.

---

## Known Stubs

None. The `.help-overlay-section-body {}` empty rule is a pre-existing Phase 31 structural placeholder (not introduced in this plan) and does not affect the theme system.

---

## Threat Flags

None. This plan is purely CSS file changes — no new network endpoints, auth paths, file access patterns, or schema changes. T-48-SC confirms zero new packages installed.

---

## Self-Check

### Files exist:
- [x] `hp41-gui/src/themes.css` exists
- [x] `hp41-gui/src/App.css` modified

### Commits exist:
- [x] 135c82d — feat(48-02): create themes.css with four complete data-theme blocks
- [x] 5fb8bab — feat(48-02): migrate App.css hardcoded colors to CSS variable references

### Verification:
- [x] `grep -c '[data-theme=' hp41-gui/src/themes.css` → 4
- [x] `grep -cE 'transform-box|transform-origin' hp41-gui/src/themes.css` → 0
- [x] `grep -c 'transition' hp41-gui/src/themes.css` → 0
- [x] `grep -c 'transform-box: fill-box' hp41-gui/src/App.css` → 2 (property + comment)
- [x] `grep -c 'var(--' hp41-gui/src/App.css` → 61
- [x] `npm run build` → success

## Self-Check: PASSED
