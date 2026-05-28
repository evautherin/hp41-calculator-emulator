# Phase 48: GUI Infrastructure + Theming - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-27
**Phase:** 48-GUI Infrastructure + Theming
**Areas discussed:** Theme selector UX, Color palette design, CSS variable scope, Theme transition

---

## Theme Selector UX

| Option | Description | Selected |
|--------|-------------|----------|
| Gear icon in title bar | Small cog next to ? help icon, opens popover with theme thumbnails | ✓ |
| Inside ? overlay | Add Theme section to existing help overlay | |
| Right-click context menu | Native OS context menu, no visible UI element | |

**User's choice:** Gear icon in title bar
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Text radio list | Simple radio buttons, clicking applies instantly, calculator is the preview | ✓ |
| Color-swatch thumbnails | Small colored squares next to each name showing palette | |

**User's choice:** Text radio list
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Theme-only popover | Just theme radio buttons, Phase 49 adds own UI | |
| Settings panel shell | General Settings popover, Phase 49 adds Onboarding section | ✓ |

**User's choice:** Settings panel shell
**Notes:** Forward-thinking for Phase 49 onboarding integration

| Option | Description | Selected |
|--------|-------------|----------|
| Click outside only | Consistent with ? overlay pattern | ✓ |
| Close button + click outside | Explicit X button plus click-outside | |
| You decide | Claude picks during implementation | |

**User's choice:** Click outside only
**Notes:** None

---

## Color Palette Design

| Option | Description | Selected |
|--------|-------------|----------|
| HP-41C body color | Warm beige/tan inspired by real HP-41C housing, vintage LCD | ✓ |
| Generic warm light theme | Neutral warm-toned, not tied to HP-41C aesthetics | |
| Full skeuomorphic HP-41C | As close to photorealistic as possible | |

**User's choice:** HP-41C body color
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| White on black | Pure white text on solid black, WCAG AAA | ✓ |
| Yellow on black | Yellow/amber on black, softer for light sensitivity | |
| System high-contrast | Inherit from OS settings via prefers-contrast | |

**User's choice:** White on black
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Full re-skin | Everything changes per theme (body, display, keys, labels, borders) | ✓ |
| Body + display only | Keys keep dark styling across all themes | |
| You decide | Claude picks per-theme key styling | |

**User's choice:** Full re-skin
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Dark as default | Matches current behavior, user switches manually | ✓ |
| OS-aware first launch | Read prefers-color-scheme on first launch | |

**User's choice:** Dark as default
**Notes:** None

---

## CSS Variable Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Semantic tokens | ~15-20 vars named by purpose (--calc-bg, --key-face, etc.) | ✓ |
| Primitive + semantic | Two layers: primitives + semantic tokens referencing them | |
| Minimal (5-8 vars) | Only most impactful variables | |

**User's choice:** Semantic tokens
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| CSS vars + props for gradients | CSS vars for flat fills/strokes, React props for gradient stops only | ✓ |
| All colors via React props | Full theme palette object as props to Keyboard | |
| You decide | Claude determines best split | |

**User's choice:** CSS vars where possible, props for gradients
**Notes:** Per P55 constraint — SVG gradient stops can't use CSS vars in `<defs>`

| Option | Description | Selected |
|--------|-------------|----------|
| Single themes.css file | New file with all four [data-theme] blocks, App.css keeps layout only | ✓ |
| Inline in App.css | Add [data-theme] blocks to existing App.css | |
| Per-theme CSS files | Four separate files per theme | |

**User's choice:** Single themes.css file
**Notes:** None

---

## Theme Transition

| Option | Description | Selected |
|--------|-------------|----------|
| Instant switch | No animation, matches utilitarian calculator aesthetic | ✓ |
| Quick CSS transition (~150ms) | Smooth color crossfade via CSS transition | |
| You decide | Claude picks during implementation | |

**User's choice:** Instant switch
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| On every change | Write prefs.json immediately on selection | ✓ |
| On app exit only | Hold in memory, flush on shutdown | |
| Debounced (500ms) | Write after 500ms of no further changes | |

**User's choice:** On every change
**Notes:** File writes are tiny (~50 bytes) and infrequent

---

## Claude's Discretion

No areas were deferred to Claude's discretion.

## Deferred Ideas

None — discussion stayed within phase scope.
