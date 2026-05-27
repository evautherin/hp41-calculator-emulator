---
phase: 50-raw-file-i-o
plan: 03
subsystem: gui-frontend-picker
tags: [react, typescript, tauri, file-dialog, picker, css]

# Dependency graph
requires:
  - phase: 50-02
    provides: "import_raw_dialog, export_raw_dialog, import_data_dialog, export_data_dialog, import_selected_programs Tauri commands"
provides:
  - "RawPickerOverlay React component — multi-select picker for .raw archives"
  - "App.tsx dialog functions: importRawDialog, exportRawDialog, importDataDialog, exportDataDialog"
  - "handlePickerConfirm + handlePickerClose callbacks for picker lifecycle"
  - "handleKey intercepts for xeq_RDPRGM/xeq_WPRGM/xeq_RDTA/xeq_WDTA (alpha-empty path)"
  - "App.css raw-picker-overlay CSS classes at z-index 60"
  - "@tauri-apps/plugin-dialog npm package in hp41-gui dependencies"
affects: [50-04, raw-file-io-cli, end-to-end-gui-raw]

# Tech tracking
tech-stack:
  added:
    - "@tauri-apps/plugin-dialog ^2.7 (npm dependency in hp41-gui/package.json)"
  patterns:
    - "Alpha-annunciator gate: alpha off → OS dialog; alpha on → fall through to cards_dir (D-50.1)"
    - "ImportRawResponse serde-tagged enum deserialized into TypeScript (type: Single/Multi/Cancelled/Empty)"
    - "pickerData state: PickerData | null drives conditional RawPickerOverlay render"
    - "import_selected_programs called by handlePickerConfirm with filePath from backend + selected indices"
    - "Escape key dismiss via useEffect keydown listener (matching HelpOverlay/OnboardingWizard pattern)"

key-files:
  created:
    - hp41-gui/src/RawPickerOverlay.tsx
  modified:
    - hp41-gui/package.json
    - hp41-gui/package-lock.json
    - hp41-gui/src/App.tsx
    - hp41-gui/src/App.css

key-decisions:
  - "Frontend does NOT import @tauri-apps/plugin-dialog JS API — the plugin is initialized server-side (Rust); npm package required for Tauri's plugin system only"
  - "handleKey intercept placed after __save_state__ block and before normal dispatchKeyId, guarded by !alphaAnnOn"
  - "handlePickerConfirm toasts 'Imported N program(s)' matching UI-SPEC copywriting contract"
  - "All new CSS classes use CSS custom properties only — no hardcoded hex values"
  - "RawPickerOverlay focuses first checkbox on mount via useRef for accessibility"

requirements-completed: [RAW-01, RAW-02, RAW-03, RAW-04]

# Metrics
duration: 14min
completed: 2026-05-27
---

# Phase 50 Plan 03: GUI Frontend — File Dialog + Multi-Program Picker Summary

**React frontend wired to 5 Tauri dialog commands via alpha-annunciator gate; RawPickerOverlay multi-select picker built with accessible markup and themed CSS**

## Performance

- **Duration:** ~14 min
- **Started:** 2026-05-27T19:27:00Z
- **Completed:** 2026-05-27T19:41:24Z
- **Tasks:** 2 of 3 implemented (Task 3 is a non-blocking visual verification checkpoint)
- **Files modified/created:** 5

## Accomplishments

- `@tauri-apps/plugin-dialog ^2.7` added to `hp41-gui/package.json`; `npm install` succeeded; `npm run build` + `npm test` both pass (210/210 tests)
- `ProgramEntry` and `PickerData` interfaces added to App.tsx for multi-program picker state typing
- `pickerData: PickerData | null` state manages picker open/close lifecycle
- `importRawDialog`: calls `import_raw_dialog`, handles Single (setCalcState + toast), Multi (setPickerData), Empty (toast), Cancelled (silent)
- `exportRawDialog`: calls `export_raw_dialog`, toasts on success, silent on cancel
- `importDataDialog`: calls `import_data_dialog`, setCalcState + toast on success
- `exportDataDialog`: calls `export_data_dialog`, toasts on success, silent on cancel
- `handlePickerConfirm`: calls `import_selected_programs` with filePath + indices; toasts "Imported N programs"
- `handlePickerClose`: clears pickerData (silent dismiss per UI-SPEC)
- `handleKey` intercepts `xeq_RDPRGM`/`xeq_WPRGM`/`xeq_RDTA`/`xeq_WDTA` when `annunciators.alpha === false`, routing to dialog functions; falls through to normal dispatch when alpha is on (preserving existing cards_dir behavior — D-50.1)
- `RawPickerOverlay` conditionally rendered when `pickerData` is non-null, positioned inside `.calculator` div alongside other overlays
- `RawPickerOverlay.tsx` created: `role="dialog"`, `aria-label="Select Programs"`, `aria-modal="true"`; Escape key dismiss; first-checkbox focus on mount; checkbox multi-select; disabled Import button at count=0; empty state message
- `App.css` extended with `.raw-picker-overlay` block: `position: absolute`, `top/left/right/bottom: 0`, `z-index: 60`, 8 classes covering header/list/row/footer, all colors via CSS variables (no hardcoded hex)

## Task Commits

1. **Task 1: Add npm dependency and wire App.tsx to dialog commands with picker state** — `5a492b2`
2. **Task 2: Create RawPickerOverlay component and CSS** — `c0a6e87`

## Files Created/Modified

- `hp41-gui/package.json` — added `@tauri-apps/plugin-dialog ^2.7` to dependencies
- `hp41-gui/package-lock.json` — updated lockfile after npm install
- `hp41-gui/src/App.tsx` — added ProgramEntry/PickerData interfaces, pickerData state, 6 callbacks, handleKey intercepts, RawPickerOverlay render
- `hp41-gui/src/RawPickerOverlay.tsx` — new multi-select picker component (121 lines)
- `hp41-gui/src/App.css` — added raw-picker-overlay CSS block (~90 lines)

## Decisions Made

- Frontend does NOT import `@tauri-apps/plugin-dialog` JS API directly — the Rust backend opens the OS dialog, the npm package is only required for Tauri's plugin system initialization
- The alpha-annunciator gate (`!alphaAnnOn`) is the single switch point between dialog path and cards_dir path (D-50.1); existing ALPHA-name behavior is 100% preserved
- `handlePickerConfirm` reads `pickerData.filePath` set by the backend's Multi response — avoids caching program bytes on the frontend (same reasoning as Plan 02 decision)
- Import toast pluralizes: `"Imported 1 program"` vs `"Imported N programs"` — matches spirit of UI-SPEC copywriting (singular/plural consistency)

## Pending Checkpoint

### Task 3: Visual verification of file dialog flow and picker overlay (non-blocking)

This checkpoint requires a human to verify the visual and interactive behavior in a running instance:

1. Run `just gui-dev` to start the GUI
2. Press Ctrl+R — native file dialog should open with .raw filter
3. Select a single-program .raw file — toast should say "Imported NAME (N steps)"
4. Press Ctrl+R — select a multi-program .raw archive — picker overlay should appear with checkboxes
5. Check 2 programs, click "Import Selected (2)" — toast should say "Imported 2 programs"
6. Cancel the picker — no state change, overlay dismisses
7. Press Ctrl+W — native save dialog should open, save the current program
8. Type a name into ALPHA (e.g. ALPHA "TEST" ALPHA), then Ctrl+R — should use ~/.hp41/cards/TEST.raw (existing behavior)
9. Verify all 4 themes show correct picker styling (no hardcoded colors)

**Status:** Pending — non-blocking, orchestrator will route to human after wave completes.

## Deviations from Plan

None — plan executed exactly as written. Both implementation tasks completed in a single pass.

## Threat Model Coverage

| Threat ID | Status | Notes |
|-----------|--------|-------|
| T-50-08 | Mitigated | file_path originates from Tauri backend OS dialog, passed back unchanged through JavaScript; backend's import_selected_programs validates is_absolute() + is_file() (Plan 02) |
| T-50-09 | Accepted | @tauri-apps/plugin-dialog is official Tauri org package; human-verify at Task 3 checkpoint |
| T-50-SC | Accepted | npm install succeeded without substitutions; package verified against official @tauri-apps org |

## Known Stubs

None — all picker state flows to real Tauri commands; no hardcoded empty values in the render path.

## Threat Flags

None — no new network endpoints or auth paths introduced. RawPickerOverlay is a pure UI component with no direct file system access.

## Self-Check: PASSED

| Check | Result |
|-------|--------|
| hp41-gui/src/RawPickerOverlay.tsx exists | FOUND |
| hp41-gui/package.json contains @tauri-apps/plugin-dialog | FOUND |
| App.tsx imports RawPickerOverlay | FOUND |
| App.tsx contains importRawDialog invoking import_raw_dialog | FOUND |
| App.tsx contains exportRawDialog invoking export_raw_dialog | FOUND |
| App.tsx contains importDataDialog invoking import_data_dialog | FOUND |
| App.tsx contains exportDataDialog invoking export_data_dialog | FOUND |
| handleKey intercepts xeq_RDPRGM/xeq_WPRGM/xeq_RDTA/xeq_WDTA | FOUND |
| pickerData state drives conditional RawPickerOverlay render | FOUND |
| App.css contains .raw-picker-overlay at z-index: 60 | FOUND |
| All CSS colors use CSS custom properties | VERIFIED |
| RawPickerOverlay has role="dialog" and aria-label="Select Programs" | FOUND |
| Escape key dismisses via useEffect | FOUND |
| Close button has aria-label="Close picker" | FOUND |
| npm run build succeeds | PASS |
| npm test (210/210) passes | PASS |
| Task 1 commit 5a492b2 | FOUND |
| Task 2 commit c0a6e87 | FOUND |

---
*Phase: 50-raw-file-i-o*
*Completed: 2026-05-27*
