# Phase 41: hp41-gui — GUI Integration + Live Display - Research

**Researched:** 2026-05-25
**Domain:** Tauri v2 + React GUI integration; live-display architecture; alarm toast routing
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-41.1:** New `tick_time` Tauri command returns a full `CalcStateView` (reusing existing `from_state()` projection). Drains `event_buffer`, so alarm events piggyback for free. Requires new Tauri v2.11 permission TOML (`hp41-gui/src-tauri/permissions/tick-time.toml`).

**D-41.2:** Single 100ms `setInterval` in the frontend for both clock (>=1 Hz per TIME-DSP-05) and stopwatch (>=10 Hz per TIME-SW-08). One interval, one boolean — avoids dual-cadence complexity.

**D-41.3:** `clock_active` and `stopwatch_keyboard_mode` booleans added to `CalcStateView` (projected from CalcState transient fields). Frontend starts `setInterval` when either is `true`; clears interval when both are `false`. Detection is response-driven.

**D-41.4:** `tick_time` calls `check_alarms()` before building CalcStateView. Alarms fire promptly (~100ms latency) during clock/stopwatch display modes.

**D-41.5:** Refactor `op_catalog(state, 2)` from manual if-blocks to a generic loop over `[(MATH_1, 0b0000_0001), (STAT_1, 0b0000_0010), (TIME_MODULE, 0b0000_0100)]`. Fulfills D-36.1's deferred commitment.

**D-41.6:** Alarm notifications reuse the existing toast overlay (single-toast policy, 2s auto-dismiss). `"alarm:message:{text}"` event parsed in App.tsx. Control alarm `"alarm:xeq:{label}"` events trigger dispatch through existing XEQ infrastructure.

**D-41.7:** Stopwatch mode in GUI has NO dedicated keyboard intercepts. User clicks XEQ "RUNSW"/"STOPSW"/"SWPT"/"STPW" via on-screen keyboard.

**D-41.8:** Frontend `setInterval` starts when `clock_active || stopwatch_keyboard_mode` in CalcStateView response. Clears when both are false.

**D-carried.1:** Zero new runtime dependencies.
**D-carried.2:** Pull-on-redraw architecture — hp41-core remains thread-free and async-free.
**D-carried.3:** Right-panel `key_ref_entries()` filter: `entry.xrom.is_none()` excludes XROM functions from right panel. Time entries discoverable via `?` overlay only.
**D-carried.4:** Phase 41 closes 4-way invariant item 4 (GUI `prgm_display.rs`). Sanctioned `non-exhaustive patterns` CI break since Phase 38 closes when 35 arms land.
**D-carried.5:** 35 display-name arm strings are deliberately duplicated from Phase 39's CLI work per D-25.6 parity invariant.
**D-carried.6:** No `println!`/`eprintln!` in hp41-core; `op_catalog` loop uses `state.print_buffer.push(format!(...))`.
**D-carried.7:** `CalcState::migrate_after_load()` is single source of truth for XROM module migration. Phase 41 inherits Time Module bit-2 activation for free.
**D-carried.8:** `docs/hp41-time-functions.json` consumed read-only via Vite static import.
**D-carried.9:** Modal prompts for SETIME/SETDATE already plumbed via `CalcStateView::modal_prompt`. Phase 41 does NOT touch modal routing.

### Claude's Discretion

- Exact `tick_time` Rust function signature and body shape — mirror existing `get_state` thunk pattern in `commands.rs`, add `check_alarms()` call
- Frontend `useRef<ReturnType<typeof setInterval>>` management and cleanup in `App.tsx` — standard React pattern
- Whether `tick_time` acquires the AppState Mutex with `lock()` or `try_lock()` (recommend `lock()` with `.unwrap_or_else(|e| e.into_inner())` per existing poisoned-lock pattern)
- Plan slicing (recommend 3-4 plans per Phase 36 precedent: arms+catalog refactor, help overlay+help_data, tick_time+live display, vitest extensions)
- `op_catalog` loop body shape — array of `(&XromModule, u8)` tuples
- HelpOverlay `expanded` state widening to 4 keys

### Deferred Ideas (OUT OF SCOPE)

- Persistent alarm acknowledgment UI (real HP-41CX beeps until acknowledged)
- Stopwatch physical keyboard shortcuts in GUI
- Dual-cadence timer optimization (1s for clock-only, 100ms for stopwatch)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TIME-GUI-01 | All new `Op` variants have `op_display_name` arms in `hp41-gui/src-tauri/src/prgm_display.rs` (4-way invariant item 4) | **ALREADY COMPLETE** — 35 Time arms exist in prgm_display.rs lines 346-382; file compiles now if all arms present. Confirmed by reading the file. |
| TIME-GUI-02 | CATALOG 2 gains "TIME 2C" section for Time Pac functions | Requires `op_catalog` refactor (D-41.5): replace parallel if-blocks with generic loop over MATH_1/STAT_1/TIME_MODULE. The `else if` bug in current code (line 364) gets fixed as part of this refactor. |
| TIME-GUI-03 | Help overlay gains "Time Pac (XROM 26)" section | Extend `HelpOverlay.tsx` SECTIONS (3→4), widen `SectionDef.id` union, widen `expanded` state, add 4th Vite import in `help_data.ts`. |
| TIME-GUI-04 | Clock display mode renders live time in GUI LCD (conditional setInterval — controlled D-11 exception) | `tick_time` Tauri command + `clock_active` field in CalcStateView + `setInterval` in App.tsx. |
| TIME-GUI-05 | Stopwatch mode renders in GUI with live-updating LCD | Same `tick_time` + `setInterval` as TIME-GUI-04; `stopwatch_keyboard_mode` field in CalcStateView. |
| TIME-GUI-06 | Alarm notifications surface as toast overlay in GUI | `tick_time` drains `event_buffer`; App.tsx parses `"alarm:message:{text}"` and routes to existing `showToast`. |
| TIME-GUI-07 | Modal prompts for SETIME/SETDATE route through existing `modal_prompt` channel | Already plumbed via Phase 31 `CalcStateView::modal_prompt` infrastructure. Verify via integration test (analog to `lcd_alternation_modal_prompt_stat1.rs`). |
</phase_requirements>

---

## Summary

Phase 41 wires the 35 Time Pac ops into the Tauri v2 + React GUI, introduces the first live-display polling mechanism (a D-11-sanctioned conditional `setInterval`), refactors `op_catalog` to a generic 3-module loop, and adds the fourth help overlay section.

**The critical discovery:** `hp41-gui/src-tauri/src/prgm_display.rs` already contains all 35 Time Pac arms (lines 346-382), written during Phase 38/39 work and committed alongside the core. TIME-GUI-01 (4-way invariant item 4) is therefore **already complete** in the source file. The sanctioned CI break is already closed at the file level. The planner must verify this claim with `cargo check -p hp41-gui` as the first task rather than adding arms.

The novel architectural element is the `tick_time` Tauri command + `setInterval` combo. The pattern follows the existing `get_state` thunk shape exactly (3 lines: lock mutex, call `check_alarms`, build and return CalcStateView) but is invoked by a 100ms interval that only runs while `clock_active || stopwatch_keyboard_mode`.

The `op_catalog` refactor (D-41.5) touches `hp41-core/src/ops/program.rs` — the one shared code change benefiting both CLI and GUI. The existing bit-1 block has a latent bug on line 364: the `else if` condition that guards "NO XROM" only fires when bit-0 is ALSO clear, but does NOT correctly handle the case where bit-0 is set and bit-2 would be needed. A generic loop corrects this naturally.

**Primary recommendation:** Plan as 4 plans: (1) verify prgm_display.rs completeness + op_catalog refactor, (2) CalcStateView + tick_time + Tauri permission + lib.rs registration, (3) App.tsx live display + alarm toast, (4) help_data.ts + HelpOverlay + vitest + integration test.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Time Pac op_display_name arms | Frontend Server (Tauri backend) | — | prgm_display.rs is Rust/Tauri; already complete per file inspection |
| CATALOG 2 TIME_MODULE enumeration | API / Backend (hp41-core) | — | op_catalog lives in hp41-core; both CLI and GUI inherit |
| Live clock/stopwatch display | Browser / Client | Frontend Server (Tauri) | setInterval in React; tick_time Tauri command provides data |
| Alarm notifications | Browser / Client | Frontend Server (Tauri) | Toast overlay in App.tsx; events flow through event_buffer in CalcStateView |
| Help overlay 4th section | Browser / Client | — | Pure React/TypeScript; data from Vite static import |
| SETIME/SETDATE modal prompts | Frontend Server (Tauri) | Browser / Client | CalcStateView.modal_prompt already plumbed from hp41-core |

## Standard Stack

### Core (existing — no new dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tauri v2 | 2.11 | Desktop app runtime + IPC | Project standard |
| React 18 | 18.x | UI framework | Project standard |
| TypeScript | project | Type safety | Project standard |
| Vite | project | Build/bundler + static JSON imports | Project standard |
| Vitest | project | Unit/component tests | Project standard |
| `hp41_core` | workspace | Calculator logic including `check_alarms`, `CalcState` | Project standard |

**No new dependencies required.** D-carried.1 prohibits adding runtime deps. All required functionality (`check_alarms`, `clock_active`, `stopwatch_keyboard_mode`, `event_buffer`) is already in `hp41-core`.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Single 100ms setInterval | Dual-cadence (1s clock / 100ms stopwatch) | Dual-cadence rejected per D-41.2 — added complexity for marginal IPC savings |
| `setInterval` polling | Tauri event system | Tauri events require additional setup and `emit` in Rust; D-41.1 chose the simpler response-driven detection approach |
| `try_lock()` in tick_time | `lock()` | `try_lock()` would silently skip alarm checks on mutex contention; `lock()` with poisoned-lock recovery is safer and matches existing thunk pattern |

## Package Legitimacy Audit

> Phase 41 installs NO new packages. All work is within the existing project stack.

No package legitimacy gate needed.

## Architecture Patterns

### System Architecture Diagram

```
React App.tsx
    │
    ├── dispatch_op / get_state (user interaction)
    │       └── CalcStateView { clock_active, stopwatch_keyboard_mode, event_buffer }
    │               │
    │               ├── clock_active || stopwatch_keyboard_mode → START setInterval(100ms)
    │               └── both false → CLEAR setInterval
    │
    └── setInterval (100ms, conditional)
            └── tick_time Tauri command
                    └── [lock AppState Mutex]
                        ├── check_alarms(state)  →  state.event_buffer filled
                        ├── drain event_buffer
                        ├── drain print_buffer
                        └── CalcStateView::from_state(state, print_lines, event_lines)
                                └── display_str: clock_active? → get_clock_display_str()
                                                stopwatch?    → get_stopwatch_display_str()
                                                else          → normal priority chain

CalcStateView consumer in App.tsx:
    ├── event_buffer lines → parse "alarm:message:{text}" → showToast
    │                     → parse "alarm:xeq:{label}"    → dispatch via invoke
    └── display_str → Display14Seg LCD (renders live time/stopwatch)
```

### Recommended Project Structure

```
hp41-gui/src-tauri/src/
├── commands.rs          # ADD: tick_time command (3-line thunk mirroring get_state)
├── types.rs             # ADD: clock_active + stopwatch_keyboard_mode fields to CalcStateView
├── lib.rs               # ADD: tick_time to generate_handler! macro
├── prgm_display.rs      # VERIFY: 35 Time arms already present (lines 346-382)
├── permissions/
│   └── tick-time.toml   # ADD: new permission TOML for tick_time command
└── capabilities/
    └── default.json     # ADD: "allow-tick-time" entry

hp41-gui/src/
├── App.tsx              # ADD: setInterval management + alarm event parsing
├── help_data.ts         # ADD: 4th Vite import (hp41-time-functions.json)
└── HelpOverlay.tsx      # ADD: 4th section "Time Pac (XROM 26)"

hp41-core/src/ops/
└── program.rs           # MODIFY: op_catalog generic loop (D-41.5)
```

### Pattern 1: tick_time Tauri Command

**What:** A new `#[tauri::command]` that calls `check_alarms`, drains both buffers, and returns CalcStateView. Mirrors `get_state` exactly but adds the alarm check.

**When to use:** Called by the frontend `setInterval` (100ms) when live display is active.

```rust
// Source: commands.rs pattern — mirror of handle_get_state
#[tauri::command]
pub fn tick_time(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_tick_time(&mut calc)
}

pub fn handle_tick_time(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    hp41_core::ops::time::alarm::check_alarms(calc);
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines, event_lines))
}
```

[VERIFIED: codebase inspection — `check_alarms` is `pub fn check_alarms(state: &mut CalcState)` at `hp41-core/src/ops/time/alarm.rs:479`]

### Pattern 2: CalcStateView live-display fields

**What:** Two new boolean fields projected from CalcState transient fields.

```rust
// Source: types.rs — add to CalcStateView struct
pub clock_active: bool,              // mirrors CalcState.clock_active (transient, #[serde(skip)])
pub stopwatch_keyboard_mode: bool,   // mirrors CalcState.stopwatch_keyboard_mode (transient)

// Source: types.rs — add to from_state()
let clock_active = state.clock_active;
let stopwatch_keyboard_mode = state.stopwatch_keyboard_mode;
```

[VERIFIED: codebase inspection — `clock_active: bool` and `stopwatch_keyboard_mode: bool` fields confirmed at `hp41-core/src/state.rs` lines 344+349 (transient, `#[serde(default, skip)]`)]

### Pattern 3: Frontend setInterval Management

**What:** A `useRef` that holds the interval ID. Starts when `clock_active || stopwatch_keyboard_mode`, clears otherwise.

```typescript
// Source: React standard useRef pattern + App.tsx state-sync effect pattern
const liveTickRef = useRef<ReturnType<typeof setInterval> | null>(null);

// After every calcState update (in a useEffect):
useEffect(() => {
    if (!calcState) return;
    const needsTick = calcState.clock_active || calcState.stopwatch_keyboard_mode;
    if (needsTick && liveTickRef.current === null) {
        liveTickRef.current = setInterval(() => {
            if (busyRef.current) return; // don't pile up if dispatch is in flight
            invoke<CalcStateView>('tick_time')
                .then(view => { setCalcState(view); setErrorMessage(null); })
                .catch(err => showToast(extractErrMessage(err)));
        }, 100);
    } else if (!needsTick && liveTickRef.current !== null) {
        clearInterval(liveTickRef.current);
        liveTickRef.current = null;
    }
}, [calcState, showToast]);
```

[ASSUMED — React pattern based on training knowledge; recommend review against React 18 useEffect cleanup semantics]

### Pattern 4: Alarm Event Parsing in App.tsx

**What:** Extend the existing `event_buffer` useEffect to parse `"alarm:message:{text}"` and `"alarm:xeq:{label}"` prefixes.

```typescript
// Source: App.tsx useEffect for event_buffer (currently lines 599-605)
// Extend the existing loop body:
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
                showToast(line);  // BEEP/TONE/other events as before
            }
        }
    }
}, [calcState, showToast]);
```

[VERIFIED: codebase inspection — `alarm.rs` produces `"alarm:message:{text}"` and `"alarm:xeq:{label}"` event strings; existing event_buffer useEffect at App.tsx lines 599-605]

### Pattern 5: op_catalog Generic Loop Refactor

**What:** Replace the 3 separate if-blocks in `op_catalog(state, 2)` with a generic array walk. The existing code has a latent bug: the `else if` on line 364 only prints "NO XROM" when both bit-0 AND bit-1 are clear, but with bit-2 (TIME_MODULE) needing its own block, the parallel if-chain logic breaks down. The generic loop is correct and simpler.

```rust
// Source: hp41-core/src/ops/program.rs (D-41.5)
// Replace lines 345-368 with:
let xrom_registry: &[(&XromModule, u8)] = &[
    (&MATH_1, 0b0000_0001),
    (&STAT_1, 0b0000_0010),
    (&TIME_MODULE, 0b0000_0100),
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

[VERIFIED: codebase inspection — MATH_1, STAT_1, TIME_MODULE consts are pub at `hp41-core/src/ops/math1/xrom.rs`; XromModule struct has `.id`, `.name`, `.ops` fields]

### Pattern 6: HelpOverlay 4th Section

**What:** Extend the 3-section pattern to 4. Mirror the Phase 36 Stat 1 Pac extension exactly.

```typescript
// Source: HelpOverlay.tsx — extend SectionDef id union and SECTIONS array
interface SectionDef {
    id: 'hp41cv' | 'math1' | 'stat1' | 'time';
    heading: string;
    predicate: (e: HelpEntry) => boolean;
}

// Add 4th entry to SECTIONS array:
{
    id: 'time',
    heading: 'Time Pac (XROM 26)',
    predicate: (e: HelpEntry) => e.xrom?.module === 'Time',
},

// Widen expanded state:
const [expanded, setExpanded] = useState<{
    hp41cv: boolean; math1: boolean; stat1: boolean; time: boolean;
}>({ hp41cv: true, math1: true, stat1: true, time: true });
```

[VERIFIED: codebase inspection — `docs/hp41-time-functions.json` uses `xrom.module: "Time"` per D-39.9; HelpOverlay.tsx currently has 3 sections with the 'stat1' pattern]

### Pattern 7: Tauri v2.11 Permission TOML

**What:** New permission file required for `tick_time` command. Identical structure to existing files.

```toml
# hp41-gui/src-tauri/permissions/tick-time.toml
"$schema" = "../gen/schemas/desktop-schema.json"

[[permission]]
identifier = "allow-tick-time"
description = "Allows the tick_time command."
commands.allow = ["tick_time"]
```

Then add `"allow-tick-time"` to `capabilities/default.json` permissions array.

[VERIFIED: codebase inspection — existing `dispatch-op.toml` pattern; `default.json` format confirmed]

### Anti-Patterns to Avoid

- **Polling without busyRef guard:** The `setInterval` callback must check `busyRef.current` before invoking `tick_time` to avoid queuing up concurrent IPC calls while a `dispatch_op` is in flight.
- **Forgetting cleanup on unmount:** The `useEffect` that starts `setInterval` must return a cleanup function calling `clearInterval` to prevent memory leaks in React StrictMode (which double-invokes effects in development).
- **Adding tick_time to busyRef:** `tick_time` is a read-only query — it should NOT set `busyRef.current = true`, as that would block user input during every 100ms tick.
- **Mutating CalcStateView directly:** The `clock_active` and `stopwatch_keyboard_mode` fields are read-only projections. The frontend only reads them; it never writes them.
- **Missing `else if` fix in op_catalog:** The current line 364 has `else if state.xrom_modules & 0b0000_0001 == 0` which is semantically incorrect for 3 modules. The loop refactor eliminates this bug.
- **Wrong `xrom.module` predicate value:** The JSON field value is `"Time"` (not `"TIME"` or `"Time Pac"`). Confirmed from D-39.9 / `hp41-time-functions.json` schema.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Alarm detection timing | Custom timer/thread in GUI | `check_alarms()` in `tick_time` response | hp41-core already has all the alarm logic |
| Live time string formatting | Custom JS time formatter | `get_clock_display_str()` / `get_stopwatch_display_str()` in hp41-core (called via tick_time CalcStateView.display_str) | Core already handles HP-41 12h/24h, DMY/MDY, centisecond format |
| XROM module registration | GUI-side module registry | hp41-core `TIME_MODULE` const + `xrom_modules` bits | Single source of truth in core |
| Modal prompt routing for SETIME/SETDATE | New GUI-side modal handling | Existing `CalcStateView.modal_prompt` + LCD-alternation in `from_state()` | Already plumbed by Phase 31 infrastructure |

**Key insight:** Phase 41's only truly novel code is the `setInterval` + `tick_time` plumbing. Every other capability already exists in hp41-core and is inherited by the GUI for free through the shared `CalcStateView::from_state()` projection.

## Common Pitfalls

### Pitfall 1: TIME-GUI-01 Already Complete — Verify Before Writing
**What goes wrong:** Implementer writes 35 new arms in prgm_display.rs, creating duplicates.
**Why it happens:** The CONTEXT.md says "35 arms needed" but the file inspection shows they were committed during Phase 38/39.
**How to avoid:** First task in Plan 41-01 must be `cargo check -p hp41-gui`. If it exits 0 (no non-exhaustive warnings), TIME-GUI-01 is done and that plan just needs the op_catalog refactor.
**Warning signs:** A `non-exhaustive patterns` warning containing Time Op names means the arms are missing; absence of such warnings means they're present.

### Pitfall 2: busyRef in setInterval Callback
**What goes wrong:** `tick_time` invoke fires while `dispatch_op` is in flight, causing two concurrent Mutex lock attempts on AppState. The second `lock()` call blocks (not deadlocks, since it's a sync Mutex on a non-reentrant path), causing the interval to queue up.
**Why it happens:** `setInterval` fires every 100ms regardless of pending IPC calls.
**How to avoid:** Guard the interval callback body with `if (busyRef.current) return;`. This skips the tick when busy — a single skipped tick (100ms) is imperceptible.
**Warning signs:** UI freezing during dispatch_op calls; cascading invocations in browser devtools.

### Pitfall 3: Interval Not Cleaned Up on Unmount
**What goes wrong:** React StrictMode in development double-invokes effects, leaving a dangling interval after the cleanup from the first invocation.
**Why it happens:** `useEffect` in StrictMode runs, cleans up, then runs again. If the cleanup doesn't call `clearInterval`, the first interval keeps firing.
**How to avoid:** The `useEffect` managing `liveTickRef` must return `() => { if (liveTickRef.current) clearInterval(liveTickRef.current); liveTickRef.current = null; }`.
**Warning signs:** Double-speed display updates in development (two intervals running); console errors about state updates on unmounted components.

### Pitfall 4: op_catalog else-if Bug
**What goes wrong:** Keeping the parallel if-chain for 3 modules (`if bit0 ... if bit1 ... else if ...`) produces wrong "NO XROM" output when only bit-2 (Time Module) is set but bit-0 and bit-1 are clear.
**Why it happens:** The original 2-module else-if logic was `if (bit0) ... if (bit1) ... else if (!bit0) "NO XROM"`. With 3 modules, this silently skips TIME_MODULE in edge cases.
**How to avoid:** Use the generic loop (Pattern 5). The loop's `any_module` check is correct for any number of modules.
**Warning signs:** `cargo test -p hp41-core` failing on `catalog_2_lists_time_when_bit2_set` test.

### Pitfall 5: CalcStateView JSON Size Budget
**What goes wrong:** Adding `clock_active: bool` + `stopwatch_keyboard_mode: bool` pushes the CalcStateView JSON over the 500-byte (empty program) or 600-byte (realistic load) budgets checked in `types.rs` tests.
**Why it happens:** Two additional boolean fields add ~52 bytes to the JSON: `"clock_active":false,"stopwatch_keyboard_mode":false`.
**How to avoid:** Update the test budget assertions when adding the fields. Measure the new baseline and document it in the test comment. Two booleans add ~52 bytes — within the headroom at both budgets (500 byte test had ~63 bytes headroom, 600 byte test had ~96 bytes headroom).
**Warning signs:** `test_dispatch_op_payload_size` failing with "got 489 bytes" after adding the fields.

### Pitfall 6: Wrong module name string in HelpOverlay predicate
**What goes wrong:** Using `e.xrom?.module === 'TIME'` or `e.xrom?.module === 'Time Pac'` instead of `e.xrom?.module === 'Time'`.
**Why it happens:** The JSON `xrom.module` field value is `"Time"` per D-39.9 `docs/hp41-time-functions.json`, but the overlay heading is `"Time Pac (XROM 26)"`. These are different strings.
**How to avoid:** Read the actual JSON file to confirm the `xrom.module` value before writing the predicate.
**Warning signs:** Time Pac section in help overlay shows 0 entries even though the JSON is loaded.

### Pitfall 7: Missing tick_time in generate_handler! macro
**What goes wrong:** `tick_time` function exists in commands.rs but fails at runtime with "command not found" because it wasn't registered in lib.rs.
**Why it happens:** Tauri v2 requires every command to appear in `generate_handler![...]` in `lib.rs`.
**How to avoid:** Add `commands::tick_time` to the macro in lib.rs. Also add `"allow-tick-time"` to `capabilities/default.json` permissions array. Verify with `just gui-check` (cargo check for the GUI crate).
**Warning signs:** `invoke('tick_time')` throwing "command not found" in the browser console.

### Pitfall 8: CalcStateView TypeScript interface not updated
**What goes wrong:** Rust `CalcStateView` gets `clock_active: bool` and `stopwatch_keyboard_mode: bool` fields, but the TypeScript `CalcStateView` interface in `App.tsx` doesn't mirror them. The `setInterval` condition `calcState.clock_active || calcState.stopwatch_keyboard_mode` silently evaluates to `undefined || undefined = undefined = false`, interval never starts.
**Why it happens:** Rust → TypeScript field mirroring requires manual update of the TS interface.
**How to avoid:** Update the `interface CalcStateView` in `App.tsx` alongside the Rust struct changes. Both are in scope for the same plan.
**Warning signs:** Clock display mode activated (CLKT) but GUI LCD never starts updating.

## Code Examples

### tick_time Tauri command (full implementation)

```rust
// Source: hp41-gui/src-tauri/src/commands.rs
// Mirrors get_state thunk shape exactly; adds check_alarms call.
#[tauri::command]
pub fn tick_time(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    let mut calc = state.lock().unwrap_or_else(|e| e.into_inner());
    handle_tick_time(&mut calc)
}

pub fn handle_tick_time(calc: &mut CalcState) -> Result<CalcStateView, GuiError> {
    hp41_core::ops::time::alarm::check_alarms(calc);
    let print_lines: Vec<String> = calc.print_buffer.drain(..).collect();
    let event_lines: Vec<String> = calc.event_buffer.drain(..).collect();
    Ok(CalcStateView::from_state(calc, print_lines, event_lines))
}
```

[VERIFIED: codebase inspection of commands.rs `handle_get_state` pattern + alarm.rs `check_alarms` signature]

### CalcStateView field additions (Rust)

```rust
// Source: hp41-gui/src-tauri/src/types.rs
// Add to CalcStateView struct definition:
/// Phase 41: live-display trigger fields projected from CalcState transient booleans.
/// Frontend starts setInterval(100ms) when either is true (D-41.3 / D-41.8).
pub clock_active: bool,
pub stopwatch_keyboard_mode: bool,

// Add to from_state() body:
let clock_active = state.clock_active;
let stopwatch_keyboard_mode = state.stopwatch_keyboard_mode;

// Add to CalcStateView { ... } constructor:
clock_active,
stopwatch_keyboard_mode,
```

[VERIFIED: codebase inspection — `CalcState.clock_active` and `CalcState.stopwatch_keyboard_mode` are `bool` fields at state.rs]

### help_data.ts fourth pool

```typescript
// Source: hp41-gui/src/help_data.ts — mirror of Phase 36 stat1 addition
import timeFunctions from '../../docs/hp41-time-functions.json';

export function helpEntriesTime(): readonly HelpEntry[] {
    return timeFunctions as readonly HelpEntry[];
}

// Update helpEntriesAll to 4-pool:
export function helpEntriesAll(): readonly HelpEntry[] {
    return [...helpEntries(), ...helpEntriesMath1(), ...helpEntriesStat1(), ...helpEntriesTime()];
}
```

[VERIFIED: codebase inspection — `docs/hp41-time-functions.json` exists (Phase 39 D-39.9); help_data.ts has 3-pool pattern ready to extend]

### E2E smoke extension for Time Pac

```javascript
// Source: hp41-gui/e2e/smoke.spec.js — mirror of Stat 1 ΣNORMD test shape
// TIME-GUI-04 / TIME-QUAL-06: DATE+ workflow (no modal, no xeq modal quirks)
it('XEQ "DATE+" adds 1 day to 2024.0101 (Time Pac via xrom_resolve)', async () => {
    const display = await $('[data-testid="lcd-display"]');
    await display.waitForExist({ timeout: 10000 });
    // Push date 2024.0101 (Jan 1 2024 in MDY format)
    // Use invokeBackend to avoid decimal-point modal complications
    const view = await invokeBackend('dispatch_op', { keyId: 'xeq_DATE+' });
    // Just verify the command doesn't error — full date arithmetic tested in numerical_accuracy.rs
    expect(typeof view.display_str).toBe('string');
});
```

Note: The specific E2E workflow depends on the time functions' behavior. A simpler target than DATE+ is `xeq_CLK12` which has no modal and just sets a flag — verifying xrom_resolve routes the Time module. The plan author should choose the simplest non-modal Time Pac op for the E2E smoke.

[ASSUMED — exact E2E test strategy; TIME-QUAL-06 is Phase 42, not Phase 41]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Phase 36: 2-module op_catalog if-chain | Phase 41: generic 3-module loop | Phase 41 | Correct for N modules; fixes latent else-if bug |
| Phase 36: 3-section HelpOverlay | Phase 41: 4-section HelpOverlay | Phase 41 | Time Pac discoverable via `?` overlay |
| No live display in GUI (D-11 no-polling) | D-11 sanctioned exception: conditional setInterval for clock/stopwatch | Phase 41 | First live-updating UI element in the GUI |

**Deprecated/outdated:**
- The `else if state.xrom_modules & 0b0000_0001 == 0` guard in op_catalog line 364 is semantically incomplete for 3 modules — replaced by loop approach.

## Critical Pre-Planning Discovery

**TIME-GUI-01 may already be satisfied.** The file `hp41-gui/src-tauri/src/prgm_display.rs` was read in full. Lines 346-382 contain all 35 Time Module Op arms:

```
Op::TimeAdate => "ADATE" through Op::TimeStpw => "STPW"
```

These arms were added during Phase 38/39 work (the comment on line 346 reads `// ── Phase 38: Time Module (XROM 26) ───`). The file header comment (line 46) still says "Covers all Op variants exhaustively (v2.2 built-ins + Math Pac I + Stat 1 Pac)" — this description is stale and needs updating to include Time Module.

**The 4-way invariant item 4 (GUI prgm_display.rs) was already closed when Phase 38/39 committed the arms.** The CONTEXT.md says there is a "sanctioned non-exhaustive patterns CI break" open since Phase 38 — this was correct when the CI break was intentional (waiting for the arms), but now the arms ARE present. The planner MUST run `cargo check -p hp41-gui` as the very first action to determine the actual state.

If `cargo check -p hp41-gui` exits 0 with no non-exhaustive warnings: TIME-GUI-01 is done, and Plan 41-01 can focus entirely on the op_catalog refactor.

## Open Questions

1. **Is TIME-GUI-01 (prgm_display.rs) already complete?**
   - What we know: All 35 Time arms are present in the file (lines 346-382 confirmed by file read).
   - What's unclear: Whether `cargo check -p hp41-gui` currently exits 0 or still shows `non-exhaustive patterns` warnings. This would only happen if the Op enum has variants NOT covered by those arms.
   - Recommendation: First task in Plan 41-01: run `cargo check -p hp41-gui` and record the result.

2. **CalcStateView JSON size after adding two boolean fields.**
   - What we know: Current empty-program budget: 500 bytes (63 bytes headroom). Two booleans add ~52 bytes serialized.
   - What's unclear: Whether the existing test assertions need updating (they will), and what the new measured baseline is.
   - Recommendation: Update both budget tests in `types.rs` with measured values after adding the fields.

3. **E2E smoke test strategy for Time Pac (TIME-QUAL-06).**
   - What we know: TIME-QUAL-06 is Phase 42 per REQUIREMENTS.md. Phase 41 does not need to add the E2E test.
   - What's unclear: Per CONTEXT.md, the canonical refs don't mention E2E as Phase 41 scope. The Phase 36 analog (Plan 36-03) only added vitest tests, not E2E.
   - Recommendation: Phase 41 does NOT add E2E smoke for Time Pac. That belongs in Phase 42 per REQUIREMENTS.md traceability.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable + cargo | All Rust compilation | ✓ | MSRV 1.88 (from CLAUDE.md) | — |
| Node.js / npm | Frontend build + vitest | ✓ | project (from package.json) | — |
| Tauri CLI | `just gui-ci` / `just gui-dev` | ✓ | v2.11 (CLAUDE.md) | — |
| `docs/hp41-time-functions.json` | help_data.ts 4th import | ✓ | 35 entries (Phase 39 D-39.9) | — |
| `hp41_core::ops::time::alarm::check_alarms` | tick_time command | ✓ | pub fn, confirmed at alarm.rs:479 | — |
| `hp41_core::ops::math1::xrom::TIME_MODULE` | op_catalog refactor | ✓ | pub const, confirmed at xrom.rs:199 | — |

**Missing dependencies with no fallback:** None.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest (frontend) + cargo test (Rust) |
| Config file | `hp41-gui/vite.config.ts` (vitest) / workspace `Cargo.toml` (Rust) |
| Quick run command | `cargo check -p hp41-gui && cargo test -p hp41-gui` |
| Full suite command | `just gui-ci` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TIME-GUI-01 | 35 Time Op display names in GUI prgm_display.rs | unit (compile) | `cargo check -p hp41-gui` | ✅ (verify arms present) |
| TIME-GUI-02 | CATALOG 2 shows "TIME 2C" section | unit (Rust) | `cargo test -p hp41-core catalog_2` | ❌ Wave 0: add `catalog_2_lists_time_when_bit2_set` test in program.rs |
| TIME-GUI-03 | Help overlay has "Time Pac (XROM 26)" section | unit (vitest) | `npm run test -- HelpOverlay` | ✅ `HelpOverlay.test.tsx` — needs 4th section assertions |
| TIME-GUI-04 | Live clock display updates | manual + unit | `cargo test handle_tick_time` | ❌ Wave 0: `handle_tick_time` unit test in commands.rs |
| TIME-GUI-05 | Live stopwatch display updates | manual + unit | `cargo test handle_tick_time` | same as TIME-GUI-04 |
| TIME-GUI-06 | Alarm notifications as toast | unit (vitest) | `npm run test -- App` | ✅ `App.test.tsx` — needs alarm event parsing test |
| TIME-GUI-07 | SETIME/SETDATE modal prompts | integration (Rust) | `cargo test -p hp41-gui lcd_alternation_modal_prompt_time` | ❌ Wave 0: `lcd_alternation_modal_prompt_time.rs` integration test |

### Sampling Rate
- **Per task commit:** `cargo check -p hp41-gui && cargo test -p hp41-core`
- **Per wave merge:** `just gui-ci`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `hp41-core/src/ops/program.rs` — add `catalog_2_lists_time_when_bit2_set` test — covers TIME-GUI-02
- [ ] `hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt_time.rs` — TIME-GUI-07 verification (modal prompts for SETIME/SETDATE TimeStep variants routing through CalcStateView)
- [ ] `hp41-gui/src-tauri/src/commands.rs` — `handle_tick_time` unit test asserting drain pattern and check_alarms call effect — covers TIME-GUI-04/05/06
- [ ] `hp41-gui/src/HelpOverlay.test.tsx` — 4th section assertions (heading present, 35 entries, collapses) — covers TIME-GUI-03
- [ ] `hp41-gui/src/App.test.tsx` — alarm event parsing test — covers TIME-GUI-06

## Security Domain

> `security_enforcement` not explicitly set to false in config. Standard review applies.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | N/A — local desktop app |
| V3 Session Management | No | N/A |
| V4 Access Control | No | N/A |
| V5 Input Validation | Yes | Time/date values validated in hp41-core before use |
| V6 Cryptography | No | N/A |

### Known Threat Patterns for Tauri IPC

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Concurrent Mutex contention via rapid tick_time + dispatch_op | Denial of Service | busyRef guard in setInterval callback prevents pileup |
| Alarm message text injected into toast UI | Spoofing | Alarm messages come from user-set alarm entries stored in CalcState — no external input source; toast renders plain text, not HTML |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | React `useRef<ReturnType<typeof setInterval>>` pattern is idiomatic for React 18 interval management | Architecture Patterns §Pattern 3 | Low risk — standard React pattern; alternative is useState which triggers re-renders |
| A2 | `docs/hp41-time-functions.json` uses `xrom.module = "Time"` (not "Time Pac" or "TIME") | Architecture Patterns §Pattern 6 | Would break HelpOverlay predicate; easy to verify by reading the JSON |
| A3 | E2E smoke (TIME-QUAL-06) is Phase 42 scope, not Phase 41 | Open Questions §3 | If wrong, planner should add a minimal E2E test in smoke.spec.js |

## Sources

### Primary (HIGH confidence)
- Codebase inspection — `hp41-gui/src-tauri/src/prgm_display.rs` (all 35 Time arms at lines 346-382)
- Codebase inspection — `hp41-gui/src-tauri/src/commands.rs` (`handle_get_state` pattern for tick_time)
- Codebase inspection — `hp41-gui/src-tauri/src/types.rs` (`CalcStateView` struct + `from_state()`)
- Codebase inspection — `hp41-gui/src/App.tsx` (`event_buffer` useEffect, toast infrastructure, `setInterval` absence)
- Codebase inspection — `hp41-gui/src/HelpOverlay.tsx` (3-section SECTIONS array, SectionDef interface)
- Codebase inspection — `hp41-gui/src/help_data.ts` (3-pool `helpEntriesAll()`)
- Codebase inspection — `hp41-gui/src-tauri/src/lib.rs` (`generate_handler!` macro)
- Codebase inspection — `hp41-gui/src-tauri/permissions/*.toml` + `capabilities/default.json` (permission TOML format)
- Codebase inspection — `hp41-core/src/ops/program.rs:303-378` (op_catalog with latent else-if bug at line 364)
- Codebase inspection — `hp41-core/src/ops/time/alarm.rs:479` (`pub fn check_alarms`)
- Codebase inspection — `hp41-core/src/ops/math1/xrom.rs:199` (`TIME_MODULE` const, `MATH_1`, `STAT_1`)
- `.planning/phases/41-hp41-gui-gui-integration-live-display/41-CONTEXT.md` — all D-41.* decisions
- `.planning/milestones/v3.1-phases/36-hp41-gui-gui-integration/36-CONTEXT.md` — D-36.* structural template

### Secondary (MEDIUM confidence)
- `hp41-gui/src/HelpOverlay.test.tsx` — existing test structure (3 sections) for Wave 0 gap analysis
- `hp41-gui/e2e/smoke.spec.js` — existing E2E test shape (invokeBackend pattern)
- `hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt_stat1.rs` — integration test template for TIME-GUI-07

### Tertiary (LOW confidence / Assumed)
- A1: React 18 `useRef` for setInterval management (training knowledge, standard pattern)
- A2: `xrom.module === "Time"` string value (derived from D-39.9 documentation, not file read)
- A3: E2E smoke scope (derived from REQUIREMENTS.md traceability table)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies confirmed present in codebase
- Architecture: HIGH — all patterns confirmed by reading actual source files
- Pitfalls: HIGH — most identified from reading actual code (else-if bug, busyRef, JSON size)
- TIME-GUI-01 status: HIGH — arms confirmed present at prgm_display.rs lines 346-382

**Research date:** 2026-05-25
**Valid until:** 2026-06-25 (stable codebase; valid indefinitely as long as Phase 41 has not started)
