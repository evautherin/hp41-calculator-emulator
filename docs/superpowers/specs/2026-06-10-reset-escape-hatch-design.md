# Reset Escape-Hatch — Design Spec

- **Date:** 2026-06-10
- **Status:** Approved (design), pending implementation plan
- **Author:** brainstormed with Claude
- **Surfaces:** `hp41-core`, `hp41-cli`, `hp41-gui` (desktop + iOS)

## Problem

A user on iOS entered an (unknown) key sequence that left the calculator in a
state where **no further input is accepted**. Restarting the app does not help:
the autosave (`~/.hp41/autosave.json` on desktop; the iOS sandbox container at
`Library/Application Support/ch.talent-factory.hp41/autosave.json`, Phase 54
PERSIST-01) reloads the same blocking state on launch. There is currently **no
in-app recovery mechanism** — the only escape is deleting/reinstalling the app
(iOS) or hand-deleting the autosave file (desktop).

### Root-cause note (separate work)

The exact blocking state could not be pinned from code alone. The obvious
candidates are ruled out: `is_running` is forced to `false` in `load_state`
(`hp41-core/src/state.rs:93`); `modal_program`, `modal_prompt`, the Phase 63/64
yield/interrupt/GETKEY fields are all `#[serde(default, skip)]` (not persisted).
The blocker must therefore be a **persisted** field left in an inconsistent state
after load — most plausibly `matrix_dim` / `matrix_active_reg` (persisted) without
the matching non-persisted `modal_program`, or a trapped `prgm_mode` / pending
entry. **This reset is the recovery mechanism, not the root-cause fix.** Once the
triggering key sequence is reproduced, the underlying inconsistency should be
fixed in a separate task so the trap stops occurring in the first place.

## Goals

1. Give every frontend a reliable in-app **escape hatch** out of any stuck state.
2. Make the escape **survive a restart** — i.e. overwrite the autosave so the
   next launch does not reload the broken state.
3. Two-tier, hardware-grounded:
   - **Soft reset** — clear working state, keep stored user data (everyday "unstick").
   - **Full reset / MEMORY LOST** — factory wipe (guaranteed escape; authentic ON+←).
4. **CLI ↔ GUI parity** (D-25.6): all three frontends get the feature.

## Non-goals

- Fixing the underlying trap bug (tracked separately — see Root-cause note).
- Making reset a programmable `Op` (it is a hardware action, deliberately not in
  program memory — so no 4-way exhaustive-match entry is created).
- Faithfully emulating real HP-41 ON power-toggle semantics (which preserve
  *everything* via Continuous Memory). Our soft reset is an intentional,
  documented divergence because a faithful ON would not unstick anything.

## Approach

**Chosen: A — Core methods + frontend escape-hatch that bypasses dispatch.**

Reset logic lives as two pure, unit-testable methods on `CalcState` in
`hp41-core` (single source of truth, no SC-4 duplication). The frontends invoke
them **outside** the normal key→Op resolution path. This bypass is the crux: the
reset must work *even when core dispatch is the thing that is stuck*.

Rejected alternatives:

- **B — Reset as an `Op` through `dispatch()`.** Self-defeating: if dispatch is
  stuck, an Op-based reset cannot run. Also forces a 4-way exhaustive-match entry.
- **C — Frontend-only, no core change.** Duplicates the "what gets cleared" logic
  across CLI and GUI → drift risk, violates the SC-4 no-core-duplication pattern.

## Design

### Core — `hp41-core/src/state.rs`

Two methods on `CalcState`, alongside `migrate_after_load()`:

```rust
/// Soft reset: clear transient working state and every mode that can trap input.
/// Preserves all stored user data. Hardware-divergent (see divergence doc).
pub fn soft_reset(&mut self) { /* … */ }

/// Full reset (MEMORY LOST): factory state. Authentic ON+← semantics.
pub fn memory_lost(&mut self) { *self = CalcState::new(); }
```

#### `soft_reset()` field semantics

The authoritative rule: **clear every field that represents transient working
state or a mode that can trap input; preserve every field that represents stored
user data.** The implementation plan finalizes the exact field-by-field list by
diffing against the live `CalcState` struct. Intended classification:

**Cleared (working state + traps):**
- Stack `X/Y/Z/T` → zero; `LastX` → zero
- `display_override` → `None`
- `prgm_mode` → `false`; `user_mode` → `false`
- `modal_program` / `modal_prompt` → `None`; modal sub-states (`integ_state`,
  `solve_state`, and siblings) → `None`
- `matrix_dim` / `matrix_active_reg` → `None`  ← prime trap suspect (persisted)
- `is_running` → `false`; `pc` → `0`; `call_stack` → cleared
- Phase 63/64 transient fields (`pending_interrupt`, `pending_yield`,
  GETKEY-capture) → `None` (defensive; already non-persisted)
- `print_buffer` → cleared; `last_key_code` → `0`

**Preserved (stored user data):**
- `program`, numbered `regs`, `reg_m` / `reg_n` / `reg_o`, text registers
- `flags`, `key_assignments`, `assignments`
- `xmem_files`, `xmem_active_file`, `xrom_modules`
- `time_offset_secs`, `rand_seed`, Advantage state (`adv_*`, `adv_tvm_state`)

Frontends additionally reset their **own** transient entry buffers that never
live in `CalcState` (CLI `shift_armed`, GUI `shiftActive`, any in-progress ALPHA
entry, any "waiting for key" UI mode) as part of handling the reset.

### GUI IPC — `hp41-gui/src-tauri/src/commands.rs`

Two new Tauri v2 commands returning `CalcStateView`, modeled on the existing
`save_state` command (commands.rs:656):

- `reset_soft` — lock `AppState`, call `state.soft_reset()`, **immediately**
  `persistence::save_state(&state_path_for_app(&app), &snapshot)` (overwrite
  autosave), return the view.
- `reset_full` — identical but `state.memory_lost()`.

Persisting inside the command (while holding the mutex) is what makes the escape
survive restart and avoids a race with the auto-save thread (which will then
write the already-reset state). New permission TOMLs
(`permissions/reset-soft.toml`, `permissions/reset-full.toml`) referenced from
`capabilities/default.json`; run `cargo check` first to generate the permission
registry (Tauri v2.11 pattern). No new enum crosses IPC.

### GUI / iOS frontend — `hp41-gui/src/Keyboard.tsx` + `App.tsx`

The `ON` key is currently an unwired dead key (`id: ''`, Keyboard.tsx:167;
"currently only ON", Keyboard.tsx:126). It is filtered out of the normal
`key_map.resolve()` path — which makes it the ideal escape hatch: wiring it in
React touches nothing in the (possibly stuck) core dispatch path.

Give the `ON` key a dedicated pointer handler **outside** `resolve()`:

- **Tap / click** → invoke `reset_soft`.
- **Long-press (~600 ms, mouse-hold and touch)** → open a confirmation sheet
  ("MEMORY LOST — delete all programs, registers and files?") → on confirm,
  invoke `reset_full`.

Long-press replaces the authentic ON+← chord, which does not translate to touch
(no chording) — documented as an approximation. The confirmation sheet is the
real safety, not the chord. The sheet is `createPortal`-ed to `document.body`
so the `transform: scale()` ancestor does not become its containing block (iOS
gotcha — see `reference_ios_gui_layout_gotchas`). Long-press logic must avoid
firing the soft-reset on the pointer-up that ends a long-press.

### CLI — `hp41-cli/src/app.rs`

Intercept a reset binding at the **very top** of `handle_key` (app.rs:478),
*above* the `pending_input` routing block. This is a deliberate, documented
exception to the D-07 never-discard ordering: the escape hatch must beat every
stuck state, including a stuck `pending_input`.

A terminal has no physical "ON" button, so the trigger is a binding. **`Ctrl+R`**
is the single reset entry. Pressing it opens a status-bar prompt offering both
tiers, which keeps one robust binding (avoids the unreliable `Ctrl+Shift+<letter>`
distinction across terminals) and is self-documenting:

```
Reset:  [s] soft   [f] full (MEMORY LOST)   [Esc] cancel
```

- `s` → `state.soft_reset()` immediately, then `persistence::save_state`
  (persistence.rs:42).
- `f` → second confirm `MEMORY LOST? [y/n]`; on `y`, `state.memory_lost()` +
  `save_state`.
- `Esc` (or any other key) → cancel, no change.

The plan must check `Ctrl+R` against existing CLI app-level bindings for
conflicts before wiring it.

### Discoverability

Tap=soft / long-press=full is not self-evident. Surface a one-line hint in the
onboarding wizard and/or the `?` overlay. Reset is **not** added to the six
`docs/hp41*-functions.json` pools (it is not an Op and not XEQ-able); the hint is
plain UI copy.

### Documentation

- **ADR** `docs/adr/v4.x-reset-escape-hatch.md` — records the escape-hatch
  architecture and the intentional divergence from hardware ON semantics.
- **Divergence entry** in `docs/hp41-*-divergences.md` — soft reset clears
  working state where real ON preserves it; full reset maps to ON+← MEMORY LOST.

## Testing

- **Core unit:** `soft_reset()` clears every trap field and preserves every
  stored-data field (table-driven against the struct); `memory_lost()` equals
  `CalcState::new()`; a synthetic "trapped" state becomes input-accepting after
  each reset.
- **GUI command:** `reset_soft` / `reset_full` mutate state and overwrite the
  autosave file (round-trip: stuck state in → reset → reload → clean).
- **CLI:** `handle_key` intercept fires the reset above `pending_input`, even
  with a pending input active; full-reset `y/n` path.
- **iOS/GUI (vitest):** long-press opens the confirm sheet; tap does not; portaled
  sheet cleaned up (`afterEach(cleanup)`, `globals:false`).

## Out of scope / follow-ups

- Root-cause fix for the persisted-trap bug (separate task, needs repro).
- Authentic ON+← physical chord on desktop (long-press chosen for cross-frontend
  consistency; chord could be a later nice-to-have).
- Animating a power on/off transition.
