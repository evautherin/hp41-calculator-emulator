# Phase 41: hp41-gui — GUI Integration + Live Display - Pattern Map

**Mapped:** 2026-05-25
**Files analyzed:** 9 new/modified files
**Analogs found:** 9 / 9

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-gui/src-tauri/src/commands.rs` | controller | request-response | self (add `tick_time` command) | exact — mirror `handle_get_state` pattern |
| `hp41-gui/src-tauri/src/types.rs` | model | transform | self (add 2 bool fields + `from_state` projection) | exact — mirror existing field additions |
| `hp41-gui/src-tauri/src/lib.rs` | config | request-response | self (add `tick_time` to `generate_handler!`) | exact |
| `hp41-gui/src-tauri/src/prgm_display.rs` | utility | transform | self (verify 35 Time arms already present) | exact |
| `hp41-gui/src-tauri/permissions/tick-time.toml` | config | — | `hp41-gui/src-tauri/permissions/get-state.toml` | exact |
| `hp41-gui/src-tauri/capabilities/default.json` | config | — | self (add `"allow-tick-time"`) | exact |
| `hp41-gui/src/App.tsx` | component | event-driven | self (add `setInterval` + alarm event parsing) | exact — mirror `busyRef` + `useEffect` patterns |
| `hp41-gui/src/help_data.ts` | utility | transform | self (add 4th Vite import + pool) | exact — mirror stat1 addition |
| `hp41-gui/src/HelpOverlay.tsx` | component | transform | self (extend `SECTIONS` 3→4) | exact — mirror Phase 36 stat1 extension |
| `hp41-core/src/ops/program.rs` | service | CRUD | self (`op_catalog` generic loop replaces parallel if-blocks) | exact — fix latent else-if bug (line 364) |

---

## Pattern Assignments

### `hp41-gui/src-tauri/src/commands.rs` — add `tick_time`

**Analog:** `hp41-gui/src-tauri/src/commands.rs` — `get_state` + `handle_get_state` pattern (lines 80-83, 228-232)

**Imports pattern** (lines 18-24 — unchanged, `check_alarms` import added):
```rust
use crate::types::{CalcStateView, GuiError};
use crate::{AppState, CancelFlag};
use hp41_core::CalcState;
use tauri::State;
```
Add to imports: `use hp41_core::ops::time::alarm::check_alarms;`
(Or call fully-qualified: `hp41_core::ops::time::alarm::check_alarms(calc)` — matches the `hp41_core::ops::math1::submit_modal` style on lines 296-297.)

**Core pattern — Tauri thunk + pure-Rust helper split** (lines 80-83, 228-232):
```rust
// TAURI THUNK (2-line glue — untestable without WebView):
#[tauri::command]
pub fn get_state(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_get_state(&mut calc)
}

// PURE-RUST HELPER (unit-testable — the real logic):
pub fn handle_get_state(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines, event_lines))
}
```

**`tick_time` implementation** — insert after `handle_get_state` (line 232):
```rust
/// Tauri command: called by frontend setInterval (100ms) when clock_active
/// or stopwatch_keyboard_mode is true. Drains event_buffer so alarm events
/// piggyback for free (D-41.1 / D-41.4).
///
/// Does NOT use busyRef — this is a read-only query that must never block
/// user interaction (Pitfall 2 from RESEARCH.md).
#[tauri::command]
pub fn tick_time(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_tick_time(&mut calc)
}

/// Pure-Rust helper for tick_time — unit-testable without a Tauri runtime.
/// Mirrors handle_get_state exactly but adds check_alarms() call first (D-41.4).
pub fn handle_tick_time(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    hp41_core::ops::time::alarm::check_alarms(calc);
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines, event_lines))
}
```

**Test pattern** — mirror `test_print_buffer_drained` (lines 386-417):
```rust
#[test]
fn handle_tick_time_drains_event_buffer_and_returns_view() {
    let mut calc = CalcState::new();
    // Simulate alarm event already in buffer
    calc.event_buffer.push("alarm:message:TEST ALARM".to_string());
    let view = handle_tick_time(&mut calc).unwrap();
    assert!(calc.event_buffer.is_empty(), "tick_time must drain event_buffer");
    assert_eq!(view.event_buffer, vec!["alarm:message:TEST ALARM"]);
}
```

---

### `hp41-gui/src-tauri/src/types.rs` — add `clock_active` + `stopwatch_keyboard_mode` to `CalcStateView`

**Analog:** `hp41-gui/src-tauri/src/types.rs` — Phase 31 Plan 03 modal field additions pattern (lines 93-105, 192-201)

**Field declaration pattern** — add after `modal_prompt` (line 104):
```rust
// Phase 41 D-41.3: live-display trigger fields projected from CalcState
// transient booleans. Frontend starts setInterval(100ms) when either is true.
// Both fields are #[serde(skip)] on CalcState (transient) but serialized here
// for the frontend — they are NOT skipped in CalcStateView.
pub clock_active: bool,
pub stopwatch_keyboard_mode: bool,
```

**`from_state()` projection pattern** — mirror lines 194-201:
```rust
// Phase 41 D-41.3: project transient CalcState booleans for live-display control.
let clock_active = state.clock_active;
let stopwatch_keyboard_mode = state.stopwatch_keyboard_mode;
```
Then add to the `CalcStateView { ... }` struct literal constructor:
```rust
clock_active,
stopwatch_keyboard_mode,
```

**JSON size test pattern** — lines 252-298 budget assertions need updating:
```rust
// Phase 41: two boolean fields add ~52 bytes
// ("clock_active":false,"stopwatch_keyboard_mode":false).
// Update the <= 500 and <= 600 bounds to the measured new values.
assert!(
    json.len() <= 560,   // was 500; update after measuring
    "CalcStateView JSON (empty program) must be ≤560 bytes, got {} bytes",
    json.len()
);
```

---

### `hp41-gui/src-tauri/src/lib.rs` — register `tick_time` in `generate_handler!`

**Analog:** `hp41-gui/src-tauri/src/lib.rs` lines 88-98 — `generate_handler!` macro pattern

**Core pattern** (lines 88-98):
```rust
.invoke_handler(tauri::generate_handler![
    commands::dispatch_op,
    commands::get_state,
    commands::sst_step,
    commands::bst_step,
    commands::run_stop,
    commands::request_cancel,
    commands::submit_modal,
    commands::cancel_modal,
    commands::submit_modal_with_label,
    // ADD:
    commands::tick_time,              // Phase 41 D-41.1 — live display timer command
])
```
No other changes to lib.rs needed — `tick_time` uses `AppState` (already managed), not a new managed type.

---

### `hp41-gui/src-tauri/src/prgm_display.rs` — verify 35 Time arms present

**Analog:** self (lines 346-382 already contain all 35 Time Module arms per RESEARCH.md discovery)

**Verification action** (not a code change — run `cargo check -p hp41-gui`):

If `cargo check -p hp41-gui` exits 0 with no `non-exhaustive patterns` warnings, TIME-GUI-01 is already satisfied. The only edit needed is updating the stale doc comment at line 46:

```rust
// Before (line 46):
/// Covers all Op variants exhaustively (v2.2 built-ins + Math Pac I + Stat 1 Pac).

// After:
/// Covers all Op variants exhaustively (v2.2 built-ins + Math Pac I + Stat 1 Pac + Time Module).
```

**Existing Time arm pattern** (lines 346-382 — reference shape; DO NOT duplicate):
```rust
// ── Phase 38: Time Module (XROM 26) ───────────────────────────────────────
Op::TimeAdate => "ADATE".to_string(),
Op::TimeAlmcat => "ALMCAT".to_string(),
// ... 33 more arms ...
Op::TimeSwpt => "SWPT".to_string(),
Op::TimeStpw => "STPW".to_string(),
```

---

### `hp41-gui/src-tauri/permissions/tick-time.toml` — new permission TOML

**Analog:** `hp41-gui/src-tauri/permissions/get-state.toml` (lines 1-6)

**Exact pattern to copy:**
```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-tick-time"
description = "Allows the tick_time command."
commands.allow = ["tick_time"]
```

---

### `hp41-gui/src-tauri/capabilities/default.json` — add `"allow-tick-time"`

**Analog:** `hp41-gui/src-tauri/capabilities/default.json` (lines 1-17)

**Core pattern — add one entry to permissions array** (line 15, after `"allow-submit-modal-with-label"`):
```json
{
  "identifier": "default",
  "description": "...",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "allow-dispatch-op",
    "allow-get-state",
    "allow-sst-step",
    "allow-bst-step",
    "allow-run-stop",
    "allow-request-cancel",
    "allow-submit-modal",
    "allow-cancel-modal",
    "allow-submit-modal-with-label",
    "allow-tick-time"
  ]
}
```

---

### `hp41-gui/src/App.tsx` — add `setInterval` management + alarm event parsing

**Analog:** `hp41-gui/src/App.tsx` — existing `busyRef`, `useRef`, `useEffect`, `showToast` patterns

**TypeScript interface addition** — add two fields to `interface CalcStateView` (lines 24-46):
```typescript
// Phase 41 D-41.3: live-display trigger fields (mirrors CalcState transient booleans).
// Frontend starts setInterval(100ms) when either is true; clears when both are false.
clock_active: boolean;
stopwatch_keyboard_mode: boolean;
```

**`useRef` declaration pattern** (mirror line 211 `busyRef`):
```typescript
// Phase 41 D-41.2/D-41.8: live-display interval reference.
const liveTickRef = useRef<ReturnType<typeof setInterval> | null>(null);
```

**`setInterval` management `useEffect`** — add after the toast auto-dismiss effect (after line 248):
```typescript
// Phase 41 D-41.2/D-41.3/D-41.8: start/stop the 100ms live-display interval.
// Starts when clock_active || stopwatch_keyboard_mode in the last CalcStateView response.
// Clears when both are false. busyRef guard prevents piling up concurrent IPC calls.
useEffect(() => {
  if (!calcState) return;
  const needsTick = calcState.clock_active || calcState.stopwatch_keyboard_mode;
  if (needsTick && liveTickRef.current === null) {
    liveTickRef.current = setInterval(() => {
      if (busyRef.current) return; // skip tick while dispatch_op is in flight (Pitfall 2)
      invoke<CalcStateView>('tick_time')
        .then(view => { setCalcState(view); setErrorMessage(null); })
        .catch(err => showToast(extractErrMessage(err)));
    }, 100);
  } else if (!needsTick && liveTickRef.current !== null) {
    clearInterval(liveTickRef.current);
    liveTickRef.current = null;
  }
  // Cleanup on unmount — prevents dangling interval in React StrictMode (Pitfall 3).
  return () => {
    if (liveTickRef.current !== null) {
      clearInterval(liveTickRef.current);
      liveTickRef.current = null;
    }
  };
}, [calcState, showToast]);
```

**Alarm event parsing `useEffect`** — replace existing `event_buffer` effect (lines 599-605):
```typescript
// Phase 26 Plan 04 CR-04b — consume calcState.event_buffer per IPC response.
// Phase 41 D-41.6: extended to parse alarm event prefixes.
//   "alarm:message:{text}" → showToast (alarm message notification)
//   "alarm:xeq:{label}"    → invoke dispatch_op (control alarm XEQ)
//   other lines            → showToast as before (BEEP/TONE/etc.)
useEffect(() => {
  if (calcState && calcState.event_buffer.length > 0) {
    for (const line of calcState.event_buffer) {
      if (line.startsWith('alarm:message:')) {
        showToast(line.slice('alarm:message:'.length));
      } else if (line.startsWith('alarm:xeq:')) {
        const label = line.slice('alarm:xeq:'.length);
        invoke<CalcStateView>('dispatch_op', { keyId: `xeq_${label}` })
          .then(view => setCalcState(view))
          .catch(err => showToast(extractErrMessage(err)));
      } else {
        showToast(line);
      }
    }
  }
}, [calcState, showToast]);
```

---

### `hp41-gui/src/help_data.ts` — add 4th Vite import + `helpEntriesTime()` + extend `helpEntriesAll()`

**Analog:** `hp41-gui/src/help_data.ts` — Phase 36 stat1 addition (lines 19, 155-157, 167-169)

**Import pattern** (lines 17-19 — add line 20):
```typescript
import functions from '../../docs/hp41cv-functions.json';
import math1Functions from '../../docs/hp41-math1-functions.json';
import stat1Functions from '../../docs/hp41-stat1-functions.json';
import timeFunctions from '../../docs/hp41-time-functions.json';  // ADD — Phase 41 D-41.3 / D-carried.8
```

**Accessor pattern** — mirror `helpEntriesStat1()` (lines 155-157):
```typescript
/// Phase 41 D-carried.8: Time Pac function entries from docs/hp41-time-functions.json.
///
/// Vite static JSON-import: baked into the production bundle at build time.
/// Malformed JSON fails the Vite build — hard-build-blocker semantics per D-25.17.
/// Mirrors Phase 39 D-39.12 fourth OnceLock + accessor pattern in hp41-cli.
/// Source: docs/hp41-time-functions.json (35 entries, 7-category convention per D-39.9).
export function helpEntriesTime(): readonly HelpEntry[] {
    return timeFunctions as readonly HelpEntry[];
}
```

**`helpEntriesAll()` update** — lines 167-169: extend from 3-pool to 4-pool:
```typescript
/// Phase 41: updated from 3-pool to 4-pool concatenation.
/// Pitfall 5 (from stat1 pattern): update in-place so all existing callers
/// (HelpOverlay.tsx) pick up Time entries automatically.
export function helpEntriesAll(): readonly HelpEntry[] {
    return [...helpEntries(), ...helpEntriesMath1(), ...helpEntriesStat1(), ...helpEntriesTime()];
}
```

---

### `hp41-gui/src/HelpOverlay.tsx` — extend `SECTIONS` 3→4

**Analog:** `hp41-gui/src/HelpOverlay.tsx` — Phase 36 stat1 extension pattern (lines 36-59, 66-78, 141-143)

**`SectionDef` id union extension** (line 36):
```typescript
// Before:
interface SectionDef {
    id: 'hp41cv' | 'math1' | 'stat1';
    ...
}

// After:
interface SectionDef {
    id: 'hp41cv' | 'math1' | 'stat1' | 'time';
    ...
}
```

**`SECTIONS` array extension** (lines 43-59 — add 4th entry):
```typescript
const SECTIONS: SectionDef[] = [
    {
        id: 'hp41cv',
        heading: 'HP-41CV (built-in)',
        predicate: (e: HelpEntry) => !e.xrom,
    },
    {
        id: 'math1',
        heading: 'Math 1 Pac (XROM 7)',
        predicate: (e: HelpEntry) => e.xrom?.module === 'Math 1',
    },
    {
        id: 'stat1',
        heading: 'Stat 1 Pac (XROM 2)',
        predicate: (e: HelpEntry) => e.xrom?.module === 'Stat 1',
    },
    // Phase 41 D-41.3: fourth section for Time Pac (XROM 26).
    // JSON xrom.module value is "Time" (confirmed from hp41-time-functions.json D-39.9).
    // CRITICAL: predicate must use 'Time' — NOT 'TIME', 'Time Pac', or 'TIME 2C' (Pitfall 6).
    {
        id: 'time',
        heading: 'Time Pac (XROM 26)',
        predicate: (e: HelpEntry) => e.xrom?.module === 'Time',
    },
];
```

**`expanded` state widening** (lines 66-70 + 76-78):
```typescript
// Before:
const [expanded, setExpanded] = useState<{ hp41cv: boolean; math1: boolean; stat1: boolean }>({
    hp41cv: true,
    math1: true,
    stat1: true,
});

// After:
const [expanded, setExpanded] = useState<{ hp41cv: boolean; math1: boolean; stat1: boolean; time: boolean }>({
    hp41cv: true,
    math1: true,
    stat1: true,
    time: true,
});
```

**Reset in `useEffect`** (line 76):
```typescript
// Before:
setExpanded({ hp41cv: true, math1: true, stat1: true });
// After:
setExpanded({ hp41cv: true, math1: true, stat1: true, time: true });
```

**`toggleSection` function** (line 141):
```typescript
// Before:
const toggleSection = (id: 'hp41cv' | 'math1' | 'stat1') => {
// After:
const toggleSection = (id: 'hp41cv' | 'math1' | 'stat1' | 'time') => {
```

---

### `hp41-core/src/ops/program.rs` — `op_catalog` generic loop refactor

**Analog:** `hp41-core/src/ops/program.rs` lines 344-368 (op_catalog CATALOG 2 block)

**Current code with latent bug** (lines 344-368):
```rust
// Current — BUGGY for 3 modules: else-if on line 364 fires only when BOTH
// bit-0 AND bit-1 are clear, but does NOT gate on bit-2 (TIME_MODULE).
if state.xrom_modules & 0b0000_0001 != 0 { /* Math Pac I */ }
if state.xrom_modules & 0b0000_0010 != 0 { /* Stat 1 Pac */ }
} else if state.xrom_modules & 0b0000_0001 == 0 { /* NO XROM (wrong!) */ }
```

**Import addition needed** (line 18 currently imports `MATH_1, STAT_1`):
```rust
// Before:
use crate::ops::math1::xrom::{MATH_1, STAT_1};
// After:
use crate::ops::math1::xrom::{MATH_1, STAT_1, TIME_MODULE};
```

**Replacement block** — replace lines 344-368 with generic loop (D-41.5):
```rust
// CATALOG 2: XROM modules loaded in this emulator.
// Phase 41 D-41.5: generic loop replaces parallel if-blocks (2-module pattern
// had a latent else-if bug at line 364 — the "NO XROM" guard was incomplete
// for 3 modules). Loop is correct for any N modules; Advantage Pac benefits for free.
let xrom_registry: &[(&crate::ops::math1::xrom::XromModule, u8)] = &[
    (&MATH_1,       0b0000_0001),
    (&STAT_1,       0b0000_0010),
    (&TIME_MODULE,  0b0000_0100),
];
let any_module = xrom_registry
    .iter()
    .any(|(_, bit)| state.xrom_modules & bit != 0);
if !any_module {
    state.print_buffer.push(format!("{:<24}", "NO XROM"));
} else {
    for (module, bit) in xrom_registry {
        if state.xrom_modules & bit != 0 {
            state.print_buffer.push(format!(
                "{:<24}",
                format!("XROM {} {}", module.id, module.name)
            ));
            for (name, _op) in module.ops {
                state.print_buffer.push(format!("{name:<24}"));
            }
        }
    }
}
```

---

## Shared Patterns

### Tauri Command Thunk Shape
**Source:** `hp41-gui/src-tauri/src/commands.rs` lines 80-83 (`get_state`) + lines 228-232 (`handle_get_state`)
**Apply to:** `tick_time` + `handle_tick_time`
```rust
// Pattern: 2-line thunk + pure-Rust helper for testability.
#[tauri::command]
pub fn <cmd>(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_<cmd>(&mut calc)
}

pub fn handle_<cmd>(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    // ... mutation ...
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines, event_lines))
}
```

### Poisoned-Lock Recovery
**Source:** `hp41-gui/src-tauri/src/commands.rs` lines 53, 68, 81, 246, 258, 296, 309, 329 (every lock site)
**Apply to:** `tick_time` thunk
```rust
let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
```
Never use plain `.unwrap()` — poisoned lock is a valid shutdown scenario.

### `busyRef` Guard Pattern
**Source:** `hp41-gui/src/App.tsx` lines 211, 262-267
**Apply to:** `setInterval` callback body in `tick_time` useEffect
```typescript
const busyRef = useRef(false);
// In setInterval callback:
if (busyRef.current) return; // skip; dispatch_op is in flight
```
`tick_time` itself must NOT set `busyRef.current = true` — it is read-only and must never block user input.

### Tauri v2.11 Permission TOML
**Source:** `hp41-gui/src-tauri/permissions/get-state.toml` (all 6 lines)
**Apply to:** `hp41-gui/src-tauri/permissions/tick-time.toml`
```toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-<kebab-cmd>"
description = "Allows the <cmd_fn> command."
commands.allow = ["<cmd_fn>"]
```
Note: TOML file name uses kebab-case; `commands.allow` value uses snake_case matching the Rust fn name.

### Vite Static JSON Import (4th pool)
**Source:** `hp41-gui/src/help_data.ts` lines 17-19 (3 existing imports) + lines 155-157 (stat1 pattern)
**Apply to:** `helpEntriesTime()` in `help_data.ts`
```typescript
import timeFunctions from '../../docs/hp41-time-functions.json';
export function helpEntriesTime(): readonly HelpEntry[] {
    return timeFunctions as readonly HelpEntry[];
}
```

### `useEffect` Cleanup Pattern
**Source:** `hp41-gui/src/HelpOverlay.tsx` lines 127-137 (keydown listener cleanup)
**Apply to:** `setInterval` `useEffect` in App.tsx
```typescript
useEffect(() => {
    // ... setup ...
    return () => { /* cleanup on unmount / dep change */ };
}, [deps]);
```
The cleanup function prevents dangling intervals in React StrictMode (Pitfall 3).

---

## Integration Test Patterns

### `lcd_alternation_modal_prompt_time.rs` (new, mirrors Phase 36 analog)
**Analog:** `hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt_stat1.rs` (lines 1-47)
```rust
// Pattern — copy file header + test structure:
use hp41_core::{
    ops::math1::modal::ModalProgram,
    ops::time::modal::TimeStep,     // equivalent of Stat1Step
    CalcState,
};
use hp41_gui_lib::types::CalcStateView;

#[test]
fn setime_prompt_renders_verbatim() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Time(TimeStep::SetimeEntry)); // verify variant names
    calc.modal_prompt = Some("TIME?".to_string());
    let view = CalcStateView::from_state(&calc, vec![], vec![]);
    assert_eq!(view.display_str, "TIME?");
}
```
Check actual `TimeStep` variant names in `hp41-core/src/ops/time/modal.rs` before writing.

### `HelpOverlay.test.tsx` — 4th section assertions
**Analog:** `hp41-gui/src/HelpOverlay.test.tsx` lines 74-88 (stat1 section drift-catch pattern)
```typescript
// Mirror stat1 → time:
import timeJson from '../../docs/hp41-time-functions.json';
// ... in describe('help_data'):
it('helpEntriesTime returns all entries from docs/hp41-time-functions.json (drift-catch)', () => {
    const allTimeSource = timeJson as unknown[];
    expect(helpEntriesTime().length).toBe(allTimeSource.length);
    expect(helpEntriesTime().length).toBeGreaterThanOrEqual(35);
});

it('helpEntriesTime entries all have xrom field with module "Time"', () => {
    for (const entry of helpEntriesTime()) {
        expect(entry.xrom!.module).toBe('Time');     // CRITICAL: "Time" not "TIME" (Pitfall 6)
        expect(entry.xrom!.module_id).toBe(26);
    }
});
// ... in describe rendering tests:
// sectionButtons.length updated from 3 to 4
```

### `App.test.tsx` — alarm event parsing
**Analog:** `hp41-gui/src/App.test.tsx` lines 322-333 (D3 event_buffer → toast)
```typescript
// Extend event_buffer test with alarm-specific parsing:
it('D4: event_buffer "alarm:message:ALARM!" surfaces alarm text in toast (not raw prefix)', async () => {
    const { container } = await renderAppAndWait();
    mockInvoke.mockResolvedValueOnce(
        makeEmptyView({ event_buffer: ['alarm:message:ALARM!'] }),
    );
    await clickKey(container, '1');
    await waitFor(() => {
        const toast = container.querySelector('.toast');
        expect(toast?.textContent).toContain('ALARM!');
        expect(toast?.textContent).not.toContain('alarm:message:');
    });
});
```

---

## No Analog Found

All files have close analogs in the codebase. No files in Phase 41 require falling back to RESEARCH.md patterns — every pattern has a direct in-codebase precedent.

---

## Metadata

**Analog search scope:** `hp41-gui/src-tauri/src/`, `hp41-gui/src/`, `hp41-core/src/ops/program.rs`, `hp41-gui/src-tauri/tests/`, `hp41-gui/src-tauri/permissions/`, `hp41-gui/src-tauri/capabilities/`
**Files scanned:** 14 source files read directly
**Pattern extraction date:** 2026-05-25

**Key invariants re-verified during pattern extraction:**
- `xrom.module === "Time"` confirmed in `docs/hp41-time-functions.json` (35 entries, all `"module": "Time"`)
- `TIME_MODULE` const is `pub` at `hp41-core/src/ops/math1/xrom.rs:199` — available for import in `program.rs`
- All 35 Time arms present in `hp41-gui/src-tauri/src/prgm_display.rs:346-382` — TIME-GUI-01 already closed
- `check_alarms` is `pub fn check_alarms(state: &mut CalcState)` at `hp41-core/src/ops/time/alarm.rs:479`
- Latent else-if bug confirmed at `hp41-core/src/ops/program.rs:364` — generic loop is the fix
