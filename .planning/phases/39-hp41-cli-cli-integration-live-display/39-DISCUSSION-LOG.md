# Phase 39: hp41-cli — CLI Integration + Live Display - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-25
**Phase:** 39-hp41-cli-cli-integration-live-display
**Areas discussed:** Clock display rendering, Stopwatch keyboard mode, Alarm event draining, JSON categories
**Mode:** `--auto` (all areas auto-selected, recommended options chosen)

---

## Clock Display Rendering

| Option | Description | Selected |
|--------|-------------|----------|
| Highest priority (above entry_buf) | Clock/stopwatch override ALL other display modes — matches real HP-41CX behavior | ✓ |
| Between PRGM and ALPHA | Insert after program step display but before alpha mode | |
| Lowest priority (below formatted X) | Only show clock when no other display content exists | |

**User's choice:** [auto] Highest priority — above entry_buf (recommended default)
**Notes:** Real HP-41CX CLKT/SW override all display modes including digit entry. The 16ms poll loop already redraws without input, satisfying TIME-DSP-05 (≥1 Hz) and TIME-SW-08 (≥10 Hz) trivially.

---

## Stopwatch Keyboard Mode

| Option | Description | Selected |
|--------|-------------|----------|
| Top-level handle_key check | Like alpha_mode — dedicated key map intercepts before PendingInput routing | ✓ |
| New PendingInput variant | StopwatchMode variant with accumulation state | |
| Separate event handler | Independent key handler function called from run() | |

**User's choice:** [auto] Top-level handle_key check (recommended default)
**Notes:** Stopwatch mode is a flat key map (each key → single dispatch), not a multi-key accumulation flow. No PendingInput variant needed — the CalcState transient flag `stopwatch_keyboard_mode` is sufficient.

---

## Alarm Event Draining

| Option | Description | Selected |
|--------|-------------|----------|
| Extend call_dispatch + poll loop | Drain event_buffer alongside print_buffer in both dispatch functions; check_alarms in poll loop body | ✓ |
| Only in call_dispatch | Wire alarm check only on keypress dispatch, not idle | |
| Separate alarm tick thread | Background thread for alarm checking (violates no-async invariant) | |

**User's choice:** [auto] Extend call_dispatch + poll loop (recommended default)
**Notes:** TIME-ALM-08 requires "on each keypress/dispatch" but extending to the idle poll loop ensures alarms fire even when no key is pressed. Matches the check_autosave() pattern (called outside event conditional).

---

## JSON Categories

| Option | Description | Selected |
|--------|-------------|----------|
| Mirror REQUIREMENTS.md (~7 categories) | Time Clock / Date Arithmetic / Display / Format / Alpha / Stopwatch / Alarm | ✓ |
| Flat single category | All 35 entries under "Time Pac" | |
| Fine-grained (~10 categories) | Split further by function type within each group | |

**User's choice:** [auto] Mirror REQUIREMENTS.md groupings (recommended default)
**Notes:** 7 categories provide a natural grouping for the `?` overlay sections. Matches the structure users see in the OM table of contents.

---

## Claude's Discretion

- Exact key bindings for interactive stopwatch mode (start/stop/split/reset physical keys)
- Clock display formatting details (spacing, AM/PM placement)
- Entry ordering within each JSON category

## Deferred Ideas

- Terminal bell for alarm notifications (future polish)
- Clock display format customization beyond OM spec
