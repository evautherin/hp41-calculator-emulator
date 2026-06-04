---
phase: quick-260603-uzh
plan: inline
subsystem: docs
tags: [documentation, adr, claude-md, divergences, architecture-history]
key_files:
  created:
    - docs/adr/v4.1-003-ios-keys-only-alpha-entry.md
    - docs/adr/v4.1-004-help-overlay-function-index.md
    - docs/adr/v4.1-005-ios-scale-safe-area-architecture.md
    - docs/adr/v4.1-006-authentic-single-step-prgm-view.md
    - docs/hp41cv-divergences.md
  modified:
    - CLAUDE.md
    - docs/architecture-history.md
metrics:
  completed: "2026-06-03"
---

# Quick Task 260603-uzh: Documentation pass for the v4.1 iOS touch-UI quick tasks

The 2026-06-03 device-testing pass landed 9 quick tasks (klp/laz/lu0/mxg/o2e/s17/scc/sef/u6t),
several of which **reversed or revised documented decisions** (Phase-55 TOUCH-04 iOS keyboard;
D-26.8 overlay filter) — well captured in quick-task SUMMARYs but not in the canonical
project docs. This pass closes that gap (owner requested a full pass: ADRs + CLAUDE.md +
divergences + architecture-history).

## Deliverables
- **4 ADRs** (next number after v4.1-002):
  - `v4.1-003` — iOS keys-only ALPHA entry (supersedes TOUCH-04).
  - `v4.1-004` — tabbed help-overlay function index + tap-to-run (revises D-26.8); CLRG/CLΣ/CLRALPHA fidelity; CLI parity.
  - `v4.1-005` — iOS scale + safe-area architecture (outer-frame env() in stylesheet; portaled overlays; recompute; pinch-lock).
  - `v4.1-006` — authentic single-step PRGM view (restores CLI↔GUI parity D-25.6).
- **`docs/hp41cv-divergences.md`** (NEW — the built-in pool lacked a divergences doc): D-CV-01
  lenient mnemonic aliases, D-CV-02 CLRALPHA legacy alias, D-CV-03 ALPHA-overrides-SHIFT,
  D-CV-04 iOS keys-only ALPHA entry.
- **`CLAUDE.md` "GUI specifics"** — 4 new bullets (iOS scale/safe-area, keys-only ALPHA,
  tabbed overlay + tap-to-run, single-step PRGM), each cross-referencing its ADR.
- **`docs/architecture-history.md`** — new "v4.1 additions (iOS Foundation)" section narrating
  the four touch-UI/help decisions.

## Notes
- Docs-only; no code/test change. `docs/hp41cv-function-matrix.md` was already regenerated for
  CLRG/CLΣ during lu0/s17.
- Still pending (flagged, not in scope): delete dead `AlphaTouchInput.tsx` + CSS; README v4.1
  wording when the milestone closes.
