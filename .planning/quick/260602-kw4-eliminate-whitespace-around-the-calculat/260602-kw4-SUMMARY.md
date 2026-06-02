---
phase: quick-260602-kw4
plan: "01"
subsystem: frontend-scaling
tags: [scaling, css, tdd, frontend-only]
dependency_graph:
  requires: []
  provides: [upscale-to-fill-scaling, corrected-design-box, max-scale-cap]
  affects: [hp41-gui/src/scale.ts, hp41-gui/src/scale.test.ts, hp41-gui/src/main.tsx, hp41-gui/src/App.css]
tech_stack:
  added: []
  patterns: [TDD RED-GREEN, MAX_SCALE upscale cap, CSS margin removal]
key_files:
  modified:
    - hp41-gui/src/scale.ts
    - hp41-gui/src/scale.test.ts
    - hp41-gui/src/main.tsx
    - hp41-gui/src/App.css
decisions:
  - "MAX_SCALE=2 chosen per plan spec; caps absurd upscaling on 4K without preventing useful viewport fill"
  - "DESIGN_WIDTH=392 (was 440), DESIGN_HEIGHT=900 (was 1020) — corrected to match real rendered footprint (keyboard 392×668 + ~213px header)"
  - "margin: 16px auto removed from .calculator — outer ScaledApp flex wrapper handles centering; DESIGN_HEIGHT=900 assumes margin absent"
metrics:
  duration: "~10 min"
  completed: "2026-06-02"
  tasks_completed: 3
  files_modified: 4
---

# Quick Task kw4: Eliminate Whitespace Around the Calculator — Summary

## One-liner

Corrected design box (392×900), MAX_SCALE=2 upscale-to-fill cap replacing cap-at-1, and `.calculator` centering margin removed; all 224 frontend tests green.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Replace cap-at-1 with MAX_SCALE upscale (TDD) | `6727b64` | scale.ts, scale.test.ts |
| 2 | Remove dead-space sources (margin + doc comment) | `e4ed7dd` | main.tsx, App.css |
| 3 | Full GUI CI gate | — (verification only) | — |

## What Changed

### scale.ts
- `DESIGN_WIDTH`: 440 → 392 (removes ~48px horizontal slack)
- `DESIGN_HEIGHT`: 1020 → 900 (removes ~120px vertical dead space)
- `MAX_SCALE = 2` exported (caps absurd upscaling on 4K)
- `Math.min(1, …)` → `Math.min(MAX_SCALE, …)` — the only logic change; downscale path preserved
- JSDoc updated: no longer claims "Never upscales (capped at 1)"

### scale.test.ts (TDD — RED then GREEN)
- 10 test cases replacing the old 6; new upscale + cap cases added; old cap-at-1 assertions removed
- Imports `MAX_SCALE` from `./scale`; covers: design-box constants, MAX_SCALE value, 2× upscale, 1.5× upscale, 4K cap, neutral, downscale-height, downscale-width, smaller-of-two-ratios, zero/negative guard

### App.css
- Removed `margin: 16px auto` from `.calculator` rule — eliminates 32px of vertical dead space inside the design box; all other properties unchanged

### main.tsx
- ScaledApp JSDoc updated to describe upscale-to-fill behavior; dropped "scale is 1 (no visual change)" claim; no structural changes

## Verification Results

- `npx vitest run src/scale.test.ts`: 10/10 pass (GREEN)
- `just gui-ci`: TypeScript clean, 82 Rust tests pass, 224 frontend tests pass
- Frozen-invariant check: `git diff --name-only HEAD~2 HEAD` shows ONLY the 4 plan-owned files

## Deviations from Plan

None — plan executed exactly as written.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. Changes are pure frontend CSS/TS — viewport dimensions feed a visual CSS scale transform only (T-kw4-01, T-kw4-02 per plan threat model; both accepted/mitigated as designed).

## Known Stubs

None.

## Checkpoint Status

Stopped at `checkpoint:human-verify` (Task 4) as required. `just gui-ci` is green. Human visual verification of scaling behavior in `just gui-dev` is pending.

## Self-Check: PASSED

- `hp41-gui/src/scale.ts` — exists and modified
- `hp41-gui/src/scale.test.ts` — exists and modified
- `hp41-gui/src/main.tsx` — exists and modified
- `hp41-gui/src/App.css` — exists and modified
- Commit `6727b64` — exists (`git log --oneline | grep 6727b64`)
- Commit `e4ed7dd` — exists (`git log --oneline | grep e4ed7dd`)

## Refinement 2: Themed Letterbox Background + iOS Scroll Lock (commit `dd71fb2`)

**Problems surfaced at checkpoint:** (1) The aspect-ratio letterbox around the calculator showed
the browser's default white background on left/right margins — most visible on iPhone in portrait
orientation. (2) The entire calculator could be dragged up/down via iOS WKWebView overscroll
rubber-band bounce, breaking the native-app feel.

**Fix applied in `hp41-gui/src/index.css` and `hp41-gui/src/main.tsx` only:**

- `index.css`: `html, body { background: var(--calc-bg, #0d0d0d) }` — letterbox area now shows the
  current theme color (dark #0d0d0d, light #e8e8e8, classic-beige #c8b890, high-contrast #000000);
  fallback `#0d0d0d` covers the brief first paint before `App.tsx` sets `data-theme` on `<body>`.
  Because `data-theme` is set on `<body>`, `var(--calc-bg)` resolves correctly on `html`/`body`.
- `index.css`: `body { position: fixed; inset: 0; width: 100% }` + `html, body { overflow: hidden;
  overscroll-behavior: none }` — pins WKWebView so whole-app rubber-band bounce is impossible; the
  overscroll area also renders the themed background.
- `index.css`: `html, body, #root { height: 100% }` — `#root` fills the fixed body correctly.
- `main.tsx`: outer wrapper `width`/`height` changed from `'100vw'`/`'100vh'` to `'100%'` —
  tracks the fixed, locked body rather than the iOS toolbar-inclusive viewport unit (eliminates the
  few-px scroll the 100vh quirk can introduce on mobile browsers).
- `main.tsx`: outer wrapper gains `background: 'var(--calc-bg, #0d0d0d)'` as belt-and-suspenders.
- Internal overlay scroll containers (help, wizard, raw-picker) are unaffected — they scroll inside
  `.calculator` via their own `overflow-y: auto`, which is independent of body lock.
- `just gui-ci`: TypeScript clean, 82 Rust + 84 integration tests pass, 224 frontend tests pass.
- `git diff --name-only` shows ONLY `hp41-gui/src/index.css` and `hp41-gui/src/main.tsx`.

## Refinement: Measure-and-Fit via ResizeObserver (commit `0d2a1c5`)

**Problem surfaced at checkpoint:** `DESIGN_HEIGHT=900` is still taller than the real rendered
`.calculator` content (~820px). On the iPhone the scale is height-bound, so the 900px design box
fills the viewport height but the calculator inside only occupies ~820px → ~76-80px dead black band
at the bottom. Any hardcoded height is fragile: display/stack/header heights are font-metric
dependent and can't be reliably predicted by arithmetic.

**Fix applied in `hp41-gui/src/main.tsx` only:**

- Removed `width: DESIGN_WIDTH` / `height: DESIGN_HEIGHT` from the inner content div.
- The inner div now uses `display: 'inline-block'` with no explicit size, so it collapses to the
  natural `.calculator` footprint (392px wide, height auto).
- A `ResizeObserver` watches the content node and calls `recompute()` on every layout change;
  the existing `window resize` listener handles viewport changes.
- `recompute()` reads `node.offsetWidth` / `node.offsetHeight` (LAYOUT values — not affected by
  the CSS `transform` on the same node, so there is no measurement/feedback loop) and feeds them
  to `computeScale(window.innerWidth, window.innerHeight, measuredW, measuredH)`.
- `DESIGN_WIDTH`/`DESIGN_HEIGHT` are used only as the pre-measurement first-paint fallback.
- `ResizeObserver` is guarded with `typeof ResizeObserver !== 'undefined'` so jsdom test
  environments (which lack `ResizeObserver`) work without mocking.
- `scale.ts` and `scale.test.ts` unchanged — `computeScale` signature was already parameterized
  on `designWidth`/`designHeight`, so no test changes were needed. All 224 frontend + 82 Rust
  tests remain green; `just gui-ci` fully clean.
