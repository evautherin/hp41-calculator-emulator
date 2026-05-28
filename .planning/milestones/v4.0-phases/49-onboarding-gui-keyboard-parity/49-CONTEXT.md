# Phase 49: Onboarding + GUI Keyboard Parity - Context

**Gathered:** 2026-05-27
**Status:** Ready for planning

<domain>
## Phase Boundary

New users can get started immediately through a first-run quick-start wizard, and power users can discover every physical keyboard shortcut. This phase delivers: (1) a 5-panel quick-start wizard overlay on first launch, (2) enriched function reference in the existing `?` overlay with expandable examples/notes for ~50 core functions, (3) a "Keyboard Shortcuts" collapsible section at the top of the `?` overlay listing all physical key mappings, and (4) GUI physical keyboard parity with the CLI for card reader shortcuts (Ctrl+W/R/D/F) and manual save (F5).

All work is in `hp41-gui/` (Tauri backend prefs field + React frontend components). Zero hp41-core changes.

</domain>

<decisions>
## Implementation Decisions

### Quick-Start Wizard
- **D-49.1:** Multi-step wizard with **5 panels** and Next/Back navigation. Each panel covers one concept:
  1. "Welcome to HP-41C" — RPN basics (type number, ENTER, type another, press +)
  2. "The Four-Level Stack" — X/Y/Z/T diagram, ENTER pushes, ops consume X+Y
  3. "SHIFT & Function Access" — Tab = SHIFT, `?` = function list, XEQ = run by name
  4. "Keyboard Shortcuts" — Quick reference table of top 10 physical keyboard shortcuts
  5. "Programming & XEQ" — How to enter/run programs, PRGM mode, R/S
- **D-49.2:** Static text and diagrams only — no live mini-calculator demo during the wizard. No interaction with real calc state.
- **D-49.3:** Wizard renders as a full-cover overlay (same pattern as HelpOverlay) with panel counter ("2 of 5") and Back/Next buttons.
- **D-49.4:** First-run detection via `onboarding_done: bool` in `GuiPrefs` (`prefs.rs`). Field uses `#[serde(default)]` for backward compat. Defaults to `false` → wizard shows. Set to `true` after user completes or dismisses the wizard.

### Function Reference Enrichment
- **D-49.5:** Enrich the existing `?` overlay — no separate reference component. Add optional `example` and `notes` fields to the 5 function JSON files (`docs/hp41*-functions.json`).
- **D-49.6:** Entries with examples/notes become **expandable** — click to reveal detail below the one-line description. Entries without enrichment stay in current compact format.
- **D-49.7:** Enrich the **top ~50 most-used functions** in Phase 49. Remaining entries can be enriched incrementally in future phases.

### Re-open Access Point
- **D-49.8:** "Show Quick Start" button lives in the **SettingsPanel only** (the D-48.3 shell). A new "Quick Start" section appears below the "Theme" section. No button in the `?` overlay — clean separation: `?` = function reference, ⚙ = app settings & onboarding.
- **D-49.9:** Clicking "Show Guide" closes the settings panel and opens the wizard overlay. The wizard always starts from panel 1 on re-open.

### Keyboard Shortcut Display
- **D-49.10:** A **dedicated collapsible "Keyboard Shortcuts" section** at the top of the `?` overlay, above the existing function sections. Lists all physical key → function mappings in a compact two-column table format.
- **D-49.11:** Shortcut data sourced from a **new `docs/keyboard-shortcuts.json`** mapping file — single source of truth for the ~30 physical keyboard mappings. The `?` overlay imports it via Vite static JSON-import (same pattern as the 5 function JSONs).

### GUI Keyboard Parity
- **D-49.12:** Add Ctrl+W/R/D/F (card reader: WPRGM/RDPRGM/WDTA/RDTA) and F5 (R/S) to `resolveKeyId()` in App.tsx — mirroring the CLI's `handle_key` bindings exactly.
- **D-49.13:** Add Ctrl+S for manual save in the GUI — mirrors CLI's Ctrl+S (`save_state` to `~/.hp41/autosave.json`). Uses existing `invoke('get_state')` or a new `save_state` Tauri command.
- **D-49.14:** KBD-04 audit: systematically compare all CLI key bindings in `resolveKeyId()` / `keys.rs` against GUI `resolveKeyId()` and document any remaining gaps.

### Claude's Discretion
No areas were deferred to Claude's discretion — all decisions were made explicitly.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Architecture & Persistence
- `hp41-gui/src-tauri/src/prefs.rs` — Preference persistence; add `onboarding_done: bool` field (documented in code comments)
- `hp41-gui/src-tauri/src/persistence.rs` — State persistence pattern (reference for save command if needed)
- `hp41-gui/src-tauri/src/commands.rs` — IPC command patterns (template for any new Tauri commands)

### GUI Components
- `hp41-gui/src/HelpOverlay.tsx` — Existing `?` overlay; add Keyboard Shortcuts section + expandable entry pattern
- `hp41-gui/src/help_data.ts` — 5-pool JSON accessor; extend `HelpEntry` interface with optional `example`/`notes` fields
- `hp41-gui/src/SettingsPanel.tsx` — Phase 48 shell; add Quick Start section (D-48.3 placeholder comment already present)
- `hp41-gui/src/App.tsx` — Physical keyboard handler (`resolveKeyId`, `handleKey`); add Ctrl+W/R/D/F, F5, Ctrl+S
- `hp41-gui/src/App.css` — Styles for new wizard overlay, expandable entries, shortcut section

### Canonical Data Files
- `docs/hp41cv-functions.json` — Built-in functions (154 entries; enrich ~30 with examples)
- `docs/hp41-math1-functions.json` — Math Pac I (45 entries; enrich ~5-10)
- `docs/hp41-stat1-functions.json` — Stat 1 Pac (26 entries; enrich ~5)
- `docs/hp41-time-functions.json` — Time Pac (35 entries; enrich ~5)
- `docs/hp41-advantage-functions.json` — Advantage Pac (114 entries; enrich ~5)
- `docs/keyboard-shortcuts.json` — NEW: physical keyboard mapping file (Phase 49 creates this)

### Tauri Backend
- `hp41-gui/src-tauri/src/lib.rs` — Register new Tauri commands if needed
- `hp41-gui/src-tauri/permissions/` — Tauri v2.11 permission TOMLs for new commands
- `hp41-gui/src-tauri/capabilities/default.json` — Permission references

### Prior Phase Context
- `.planning/phases/48-gui-infrastructure-theming/48-CONTEXT.md` — D-48.3 (settings panel shell), D-48.4 (click-outside dismiss)

### Pitfalls & Constraints
- STATE.md §Pitfalls P59 — Onboarding flag in `prefs.json`, never in `autosave.json`
- CLAUDE.md §Frozen Invariants — Tauri v2.11 permission TOML pattern, bundle ID, no-polling rule (D-11)
- CLAUDE.md §Frozen Invariants — 4-way exhaustive match not required (no new Op variants in this phase)

### Requirements
- `.planning/REQUIREMENTS.md` §Onboarding & Reference (ONBOARD-01..05) + §GUI Keyboard Parity (KBD-01..04)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `HelpOverlay.tsx` — Full-cover overlay pattern with search, collapsible sections, click-outside dismiss, Esc close. Template for wizard overlay and shortcut section.
- `help_data.ts` — 5-pool `helpEntriesAll()` accessor with Vite static JSON-import and `HelpEntry` interface. Extend for `example`/`notes` fields and keyboard shortcut JSON.
- `SettingsPanel.tsx` — Settings panel shell with explicit Phase 49 placeholder comment on line 66.
- `prefs.rs` — `GuiPrefs` struct with documented Phase 49 expansion path (line 12, 32-34).
- `resolveKeyId()` (App.tsx:109) — Physical keyboard → op ID mapper. Already handles F7/F8, letter keys, digits. Extend for Ctrl+combos and F5.
- CLI `handle_key` (app.rs:323, 436, 768) — Reference implementation for Ctrl+S (save), Ctrl+W/R/D/F (card reader), F5 (R/S).

### Established Patterns
- **Overlay pattern:** Full-cover semi-transparent modal, close on Esc + click-outside. Both HelpOverlay and SettingsPanel follow this.
- **Vite static JSON-import:** Build-blocker semantics — malformed JSON fails the build. All 5 function JSONs use this.
- **Collapsible sections:** HelpOverlay uses `SectionDef[]` with heading + predicate. Keyboard Shortcuts section follows same pattern.
- **No Ctrl-key handling yet:** `resolveKeyId()` has no `e.ctrlKey`/`e.metaKey` checks — this is new territory for the GUI.

### Integration Points
- `App.tsx` handleKey/resolveKeyId — Ctrl+key and F5 bindings
- `App.tsx` state management — `onboardingOpen` boolean alongside `helpOpen` / `settingsOpen`
- `SettingsPanel.tsx` line 66 — Insert Quick Start section
- `HelpOverlay.tsx` SECTIONS array — Add Keyboard Shortcuts as first section
- `help_data.ts` — Extend HelpEntry + add keyboard shortcut data import
- `main.tsx` — No changes expected (themes.css already imported)

</code_context>

<specifics>
## Specific Ideas

- The wizard should feel like a friendly introduction to a complex instrument, not a tutorial wall — 5 panels keeps it tight
- Panel 4 (Keyboard Shortcuts) in the wizard is a teaser pointing users to the full shortcut reference in the `?` overlay
- The `?` overlay's new Keyboard Shortcuts section should be the definitive reference, not the wizard
- Expandable function entries only expand when clicked — the compact list remains the default view
- `keyboard-shortcuts.json` as a separate data file (not hardcoded) allows the CI parity test (KBD-04) to consume the same source of truth

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 49-Onboarding + GUI Keyboard Parity*
*Context gathered: 2026-05-27*
