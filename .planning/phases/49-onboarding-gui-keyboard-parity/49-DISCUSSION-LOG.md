# Phase 49: Onboarding + GUI Keyboard Parity - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-27
**Phase:** 49-Onboarding + GUI Keyboard Parity
**Areas discussed:** Quick-start format, Function reference depth, Re-open access point, Shortcut display in ? overlay

---

## Quick-start Format

### Question 1: What format should the first-run quick-start guide use?

| Option | Description | Selected |
|--------|-------------|----------|
| Multi-step wizard | 3-5 panels with Next/Back navigation. Each panel covers one concept. Reuses overlay pattern. | ✓ |
| Single scrollable page | One tall overlay with all content. Simpler to build, less guided. | |
| Interactive highlight tour | Spotlight-style tooltips pointing at actual UI elements. Most engaging but complex. | |

**User's choice:** Multi-step wizard
**Notes:** None

### Question 2: How many panels should the wizard have?

| Option | Description | Selected |
|--------|-------------|----------|
| 4 panels (as previewed) | RPN basics, Stack, SHIFT & Functions, Keyboard Shortcuts. Tight and focused. | |
| 5 panels (add Programs) | Same 4 plus a 'Programming & XEQ' panel. More complete. | ✓ |
| You decide | Claude picks the right number. | |

**User's choice:** 5 panels (add Programs)
**Notes:** None

### Question 3: Static text or live demo?

| Option | Description | Selected |
|--------|-------------|----------|
| Static text + diagrams | Clear illustrations, no real calc interaction. Simple to build. | ✓ |
| Live mini-demo | User can press keys during tutorial. More engaging but complex. | |

**User's choice:** Static text + diagrams
**Notes:** None

---

## Function Reference Depth

### Question 1: How should ONBOARD-03 be delivered?

| Option | Description | Selected |
|--------|-------------|----------|
| Enrich existing ? overlay | Add example/notes fields to function JSON. Overlay renders inline. Single source of truth. | ✓ |
| Separate reference panel | New dedicated component with richer layout. Duplicates search infra. | |
| Keep ? overlay as-is | Consider ONBOARD-03 satisfied by current overlay. No enrichment. | |

**User's choice:** Enrich existing ? overlay
**Notes:** None

### Question 2: How many functions should get examples/notes?

| Option | Description | Selected |
|--------|-------------|----------|
| Top ~50 most-used functions | Focus on arithmetic, stack, trig, stat, and common XROM functions. | ✓ |
| All ~374 entries | Complete coverage. Ambitious content task. | |
| You decide | Claude picks pragmatic scope. | |

**User's choice:** Top ~50 most-used functions
**Notes:** None

### Question 3: Expandable or always visible?

| Option | Description | Selected |
|--------|-------------|----------|
| Expandable (click to reveal) | Keeps overlay compact. Click entry to see example/notes. | ✓ |
| Always visible | Example and notes always shown. Richer but taller list. | |

**User's choice:** Expandable (click to reveal)
**Notes:** None

---

## Re-open Access Point

### Question 1: Where should the re-open trigger live?

| Option | Description | Selected |
|--------|-------------|----------|
| Settings panel only | "Show Guide" button in SettingsPanel. Clean separation: ? = reference, ⚙ = settings. | ✓ |
| ? overlay header | Link/button at top of ? overlay. Mental model: ? = all help. | |
| Both | Button in settings AND link in ? overlay. Maximum discoverability. | |

**User's choice:** Settings panel only
**Notes:** None

---

## Shortcut Display in ? Overlay

### Question 1: How should shortcuts appear?

| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated section at top | Collapsible "Keyboard Shortcuts" section above function sections. Compact table. | ✓ |
| Inline on each entry | Shortcut badge next to each function with a physical key mapping. | |
| Separate shortcuts tab | Tab toggle between Functions and Shortcuts views. | |

**User's choice:** Dedicated section at top
**Notes:** None

### Question 2: Shortcut data source?

| Option | Description | Selected |
|--------|-------------|----------|
| Hardcoded in component | ~30 mappings, simple array constant. Matches resolveKeyId() pattern. | |
| New JSON mapping file | Separate docs/keyboard-shortcuts.json. More structured, CI-consumable. | ✓ |
| You decide | Claude picks whichever is simpler. | |

**User's choice:** New JSON mapping file
**Notes:** None

---

## Claude's Discretion

No areas were deferred to Claude's discretion.

## Deferred Ideas

None — discussion stayed within phase scope.
