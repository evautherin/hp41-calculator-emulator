---
phase: quick-260603-sef
plan: inline
subsystem: gui-frontend
tags: [ios, portal, position-fixed, bottom-sheet, alpha-bar, scale-transform]
key_files:
  modified:
    - hp41-gui/src/App.tsx
    - hp41-gui/src/App.test.tsx
    - hp41-gui/src/App.css
    - hp41-gui/src/AlphaTouchInput.tsx
    - hp41-gui/src/AlphaTouchInput.test.tsx
metrics:
  completed: "2026-06-03"
verification:
  on_device: "PASS (iPhone 15 Pro) — print sheet sits above the home indicator without overlapping the keypad; ALPHA bar pinned correctly (before it was superseded by u6t)."
---

# Quick Task 260603-sef: Portal iOS bottom sheets out of the scale transform

The iOS print BottomSheet and the AlphaTouchInput bar were `position: fixed` but rendered
INSIDE the `transform: scale()` container (ScaledApp `.scaled-app-frame`). A transform
ancestor becomes the containing block for `position:fixed` descendants, so they were glued
to the scaled calculator's box (overlapping the keypad) instead of the viewport — the same
trap the removed PRGM sheet had.

## Changes
- `App.tsx`: render the print BottomSheet and the AlphaTouchInput bar via
  `createPortal(..., document.body)` so `position:fixed` resolves against the viewport. The
  mxg peek-reservation (`querySelector('.bottom-sheet')`) still finds the sheet in body.
- `App.test.tsx`: portaled nodes live in `document.body`, so M3/M3b query `document` not the
  render container; added `afterEach(cleanup)` (vitest runs `globals:false` → no auto-cleanup,
  so portaled nodes would leak across tests).
- During verification the ALPHA bar (now viewport-fixed) revealed UX issues that were fixed
  here too, then **superseded by [[260603-u6t]]** which removed the bar entirely:
  - `App.css`: input `min-width:0` (Done button no longer pushed off the right edge) +
    safe-area horizontal insets + themed Done button (commit d8bb629).
  - `AlphaTouchInput.tsx`: field mirrors the engine ALPHA register via `display_str` so it
    shows typed text from any input method; backspace → `alpha_backspace` (commits 0b388ca,
    d510bd5, c9b8c94).

## Commits
7e27da4 (portal), d8bb629 (input min-width/Done), 0b388ca + d510bd5 (mirror), c9b8c94 (backspace).

## Net result
The **print sheet portal** is the lasting win (sheet positions correctly). The ALPHA-bar
refinements were superseded when u6t made iOS ALPHA entry keys-only and removed the bar.
The `position:fixed`-in-transform pattern is captured in [[reference_ios_gui_layout_gotchas]].
