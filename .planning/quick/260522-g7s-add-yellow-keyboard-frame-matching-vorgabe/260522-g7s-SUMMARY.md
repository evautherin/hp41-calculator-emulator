---
quick_id: 260522-g7s
slug: add-yellow-keyboard-frame-matching-vorgabe
status: complete
date: "2026-05-22"
commit: 509344a (initial) + follow-up (frame top-edge correction)
---

# Quick Task 260522-g7s — SUMMARY

## Outcome

Added a thin gold/yellow trim line that runs along the outer perimeter of
the keyboard SVG so the on-screen GUI matches the user-supplied reference
(`~/Downloads/gui-vorgabe.png`). All other visual elements (display, stack
panel, body gradient, top bevel, key caps, labels, ALPHA / SHIFT
animations) are byte-identical to the pre-change render.

## Implementation

One edit in `hp41-gui/src/Keyboard.tsx`: insert a stroke-only `<rect>`
between the existing body / bevel rects and the `KEY_DEFS.map(...)` loop.
The frame wraps the **main keyboard grid only** — the ON / USER / PRGM /
ALPHA top-row band sits ABOVE the frame (faithful to the physical unit,
where the brass surround starts below the top-row keys):

```jsx
{/* HP-41 gold trim — thin frame around the MAIN KEYBOARD GRID only. ... */}
<rect
  x={1.25}
  y={PAD + TOP_ROW_H + TOP_GAP / 2}
  width={KEYBOARD_W - 2.5}
  height={KEYBOARD_H - PAD - TOP_ROW_H - TOP_GAP / 2 - 1.25}
  fill="none"
  stroke="#c8b878"
  strokeWidth={1.5}
  rx={8.75}
/>
```

- **`y = PAD + TOP_ROW_H + TOP_GAP / 2`**: top edge is centered in the
  12-px gap between the top-row band and the main grid, so the frame
  visibly begins below the ON / USER / PRGM / ALPHA row.
- **Inset 1.25 on left / right / bottom**: SVG strokes are centered on
  the path; without the inset the outer 0.75 of the stroke would be
  clipped at the viewBox edges.
- **`rx={8.75}`**: matches the body rect's `rx={10}` minus the inset so
  the corner curvature stays concentric with the dark body.
- **`#c8b878`**: warm pale gold; same family as the SHIFT cap (`#d68a1c`)
  without competing with it; reads as metallic trim against the dark body.
- **`strokeWidth={1.5}`**: thin enough to read as trim, thick enough to
  remain visible at the 392-px CSS width and at retina scaling.

## Verification

| Check | Result |
|-------|--------|
| `npx tsc --noEmit` (hp41-gui) | green |
| `npm test` (hp41-gui — vitest) | 166 / 166 pass (5 test files) |
| Visual: Vite dev render | gold trim visible on full keyboard perimeter, matches `gui-vorgabe.png` |
| `Keyboard.test.tsx` parity (KEY_DEFS) | unchanged — new rect carries no `data-key-id`, no interaction handlers |
| Backend (`hp41-core`, `src-tauri/`) | untouched |
| SC-4 (no core duplication in GUI) | preserved (cosmetic SVG only) |
| CLI ↔ GUI parity D-25.6 | preserved (no `CalcState` field, no IPC change) |

Visual evidence: `.planning/quick/260522-g7s-add-yellow-keyboard-frame-matching-vorgabe/after.png`
(captured in Vite dev mode after the change).

## Files touched

- `hp41-gui/src/Keyboard.tsx` — +13 lines (1 SVG `<rect>` + adjacent comment)
- `.planning/quick/260522-g7s-add-yellow-keyboard-frame-matching-vorgabe/260522-g7s-PLAN.md` — new
- `.planning/quick/260522-g7s-add-yellow-keyboard-frame-matching-vorgabe/260522-g7s-SUMMARY.md` — new (this file)
- `.planning/quick/260522-g7s-add-yellow-keyboard-frame-matching-vorgabe/after.png` — new (visual evidence)
- `.planning/STATE.md` — +1 row in "Quick Tasks Completed" table

## Out-of-scope (not touched)

- `.calculator` div border in `App.css` (outer container around display + keyboard).
- Backend (`hp41-core`, `hp41-gui/src-tauri/`).
- Key colors, label colors, animations, gradients, top bevel highlight.
- Tests — the existing suite already exercises all relevant assertions; no new test needed for a non-interactive cosmetic rect.

## Notes

The `gui-vorgabe.png` reference shows only the keyboard region cropped from
a full calculator view, so it is impossible to tell from the image whether
the gold trim was also intended to extend up around the display panel. The
current scope follows the literal user instruction ("NUR den Rahmen" / "only
the frame [shown in the vorgabe]") and keeps the change limited to the
keyboard SVG. If the trim should also wrap the display, a follow-up quick
task can extend `.calculator { border: ... }` in `App.css` to use the same
`#c8b878` color.
