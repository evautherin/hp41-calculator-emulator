# Phase 39: hp41-cli — CLI Integration + Live Display - Research

**Researched:** 2026-05-25
**Domain:** Rust TUI (ratatui 0.30 / crossterm), JSON-canonical data pipeline, XROM resolver extension, live display modes
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-39.1:** `get_display_string()` priority — clock_active check first, then stopwatch_keyboard_mode, then entry_buf, then prgm step, then alpha, then formatted X.
- **D-39.2:** Existing 16ms poll loop satisfies ≥1 Hz (clock) and ≥10 Hz (stopwatch) requirements without new timer infrastructure.
- **D-39.3:** Clock display exits on ANY keypress — `state.clock_active = false` at top of `handle_key()`, key still processed normally.
- **D-39.4:** Stopwatch keyboard mode is a top-level check in `handle_key()` before `PendingInput` routing — same architectural pattern as `alpha_mode`. Key map: R/S → start/stop toggle, dedicated split/reset key, Esc to exit.
- **D-39.5:** No new `PendingInput` variant for stopwatch — flat key dispatch only.
- **D-39.6:** `call_dispatch()` and `call_dispatch_and_drain()` both drain `state.event_buffer` after `print_buffer` drain. Event prefix routing: `alarm:message:*` → `self.message`, `alarm:xeq:*` → XEQ dispatch, `BEEP`/`TONE *` → ignored.
- **D-39.7:** `check_alarms()` called in `run()` loop body OUTSIDE the `event::poll` conditional — on every 16ms tick, not just on keypress.
- **D-39.8:** Alarm messages surface as `self.message` status-bar line. Persists until next keypress clears it.
- **D-39.9:** `docs/hp41-time-functions.json` uses 7 categories: "Time Clock", "Time Date Arithmetic", "Time Display", "Time Format", "Time Alpha", "Time Stopwatch", "Time Alarm".
- **D-39.10:** Each JSON entry carries `"xrom": { "module": "Time", "module_id": 26, "function_id": N }` per D-28.3 schema. All 35 entries have `key_path: "XEQ \"MNEMONIC\""`.
- **D-39.11:** Inline `divergences` field only where OM behavior differs. Full taxonomy in `docs/hp41-time-divergences.md` (Phase 40).
- **D-39.12:** Fourth `OnceLock<Vec<HelpEntry>>` named `TIME_HELP_ENTRIES` in `help_data.rs` with narrow accessor `help_entries_time()`. `help_entries_all()` extends to 4-pool chain: cv → math1 → stat1 → time.
- **D-39.13:** `?` help overlay gains "Time Pac (XROM 26)" section auto-derived from JSON categories. Incremental substring search spans all four pools.
- **D-39.14:** `xrom_shadowing.rs` extended to verify Time Pac mnemonics disjoint from Math 1 + Stat 1 + `BUILTIN_CARD_OP_NAMES`.
- **D-carried.1:** Zero new runtime dependencies.
- **D-carried.2:** Pull-on-redraw architecture — no async, no threads added.
- **D-carried.3:** Right-panel `key_ref_entries()` filter: `entry.xrom.is_none()` — Time entries excluded from right panel.
- **D-carried.4:** Phase 39 closes 4-way invariant item 3 (CLI `prgm_display.rs`). Item 4 (GUI) deferred to Phase 41.

### Claude's Discretion

- Exact key bindings for interactive stopwatch mode (which physical keys map to start/stop/split/reset).
- Formatting details for clock display (spacing, AM/PM placement) — follow patterns in `hp41-core/src/ops/time/alpha_time.rs`.
- Order of entries within each JSON category — follow REQUIREMENTS.md order or alphabetical by mnemonic.

### Deferred Ideas (OUT OF SCOPE)

- Alarm sound/bell (terminal `\x07`)
- Clock display format customization beyond OM spec
- All Phase 40 documentation, Phase 41 GUI integration items

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TIME-CLI-01 | `docs/hp41-time-functions.json` authored with all 35 Time Pac entries following v3.0/v3.1 JSON schema | JSON schema fully verified from `docs/hp41-stat1-functions.json` precedent; 35 Op variants enumerated from `hp41-core/src/ops/mod.rs` |
| TIME-CLI-02 | Fourth `OnceLock<Vec<HelpEntry>>` in `help_data.rs`; `help_entries_all()` merges 4 pools | `help_data.rs` pattern verified; `help_entries_all()` currently chains 3 pools; adding 4th is mechanical |
| TIME-CLI-03 | All 35 Time Op variants have `op_display_name` arms in CLI `prgm_display.rs` (4-way invariant item 3) | **Already present** — 35 Time arms already landed in `prgm_display.rs` during Phase 38; CLI compiles clean; no work needed here |
| TIME-CLI-04 | `?` help overlay gains "Time Pac (XROM 26)" section | Auto-derived from JSON categories via `help_overlay_rows()` — adding 4th pool completes this |
| TIME-CLI-05 | Clock display mode renders live-updating time in TUI display area | `get_clock_display_str()` already exists in core (`clock.rs:297`); integrate into `get_display_string()` in `ui.rs` per D-39.1 |
| TIME-CLI-06 | Stopwatch interactive mode renders in TUI with dedicated key bindings | `get_stopwatch_display_str()` already exists in core (`stopwatch.rs:190`); new `handle_stopwatch_mode_key()` method in `app.rs` per D-39.4 |
| TIME-CLI-07 | Alarm notifications surface as status-bar messages in CLI | `check_alarms()` pub function verified in `alarm.rs:479`; event_buffer drain added to dispatch methods per D-39.6/D-39.7 |
| TIME-CLI-08 | `xrom_shadowing.rs` extended to `TIME_MODULE.ops` — all Time mnemonics confirmed disjoint | `TIME_MODULE` is already exported from `hp41-core/src/ops/math1/xrom.rs`; existing test file at `hp41-core/tests/xrom_shadowing.rs` needs new section mirroring STAT_1 block |

</phase_requirements>

---

## Summary

Phase 39 is a mechanical extension of the well-established Phase 34 (Stat 1 CLI integration) pattern, with two novel additions: live display modes (clock/stopwatch) and alarm event draining. The codebase provides all required infrastructure — core helper functions for clock/stopwatch display already exist (`get_clock_display_str`, `get_stopwatch_display_str`), `check_alarms()` is already public, and `TIME_MODULE` is already exported.

**Critical discovery:** The 4-way exhaustive-match invariant item 3 is already satisfied. The 35 Time Op arms were added to `hp41-cli/src/prgm_display.rs` during Phase 38 (confirmed by grepping the file — all `Op::Time*` variants are present). This means TIME-CLI-03 requires no implementation work. The `hp41-cli` workspace compiles clean (`cargo check` passes).

**Second critical discovery:** `function_matrix_parity.rs::test_pool_partition_is_exhaustive()` currently guards against `module_id = 26` (Time) by treating it as a rogue ID. After Phase 39 adds the 4th JSON pool, this test must be updated to recognize `module_id = 26` as valid and assert count == 35.

The total Op variant count is 35 (confirmed by enumerating the enum: TimeAdate, TimeAlmcat, TimeAlmnow, TimeAtime, TimeAtime24, TimeClk12, TimeClk24, TimeClkt, TimeClktd, TimeClock, TimeCorrect, TimeDate, TimeDatePlus, TimeDdays, TimeDmy, TimeDow, TimeMdy, TimeRclaf, TimeRclalm, TimeRclsw, TimeRunsw, TimeSetaf, TimeSetdate, TimeSetime, TimeSetsw, TimeStopsw, TimeSw, TimeTplusx, TimeTime, TimeXyzalm, TimeClalma, TimeClalmx, TimeClralms, TimeSwpt, TimeStpw).

**Primary recommendation:** Execute Phase 39 as three distinct work streams: (1) JSON + help data + overlay (TIME-CLI-01/02/04), (2) live display + alarm draining (TIME-CLI-05/06/07), (3) test extension (TIME-CLI-03 verification + TIME-CLI-08).

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| JSON canonical data (hp41-time-functions.json) | Static asset (compile-time embed) | — | `include_str!` at compile time; no runtime I/O |
| Help overlay 4th pool | Frontend (hp41-cli) | hp41-core (data shapes) | `OnceLock` in `help_data.rs`; rendered in `ui.rs` |
| Live clock display string | hp41-core (compute) | hp41-cli (render) | `get_clock_display_str()` is in core; CLI calls it on redraw |
| Live stopwatch display string | hp41-core (compute) | hp41-cli (render) | `get_stopwatch_display_str()` is in core; CLI calls it on redraw |
| Alarm detection | hp41-core (`check_alarms`) | hp41-cli (drain + display) | Core detects; CLI drains `event_buffer` and sets `self.message` |
| Stopwatch keyboard mode key dispatch | hp41-cli (`handle_key`) | hp41-core (dispatch) | Mode flag lives in CalcState; routing logic lives in CLI |
| XROM shadowing test | hp41-core/tests | — | Tests live in `hp41-core/tests/xrom_shadowing.rs` |
| `prgm_display.rs` Time arms | hp41-cli | — | Already present; no new work required |

---

## Standard Stack

### Core (no new dependencies)

Phase 39 adds zero new runtime dependencies (D-carried.1 locked). All required functionality is already present.

| Component | Source | Purpose |
|-----------|--------|---------|
| `ratatui` 0.30 | Already in workspace | TUI rendering; poll loop reused for live updates |
| `crossterm` 0.29 | Already in workspace | Terminal input events |
| `serde_json` | Already in workspace | OnceLock JSON parse |
| `hp41-core::ops::time::clock::get_clock_display_str` | Phase 38 | Returns `Option<String>` for clock display |
| `hp41-core::ops::time::stopwatch::get_stopwatch_display_str` | Phase 38 | Returns `Option<String>` for stopwatch display |
| `hp41-core::ops::time::alarm::check_alarms` | Phase 38 | Pub fn to detect past-due alarms |

### Package Legitimacy Audit

> **Not applicable.** Phase 39 installs zero new packages. All required libraries are already workspace dependencies from prior phases.

---

## Architecture Patterns

### System Architecture Diagram

```
User keypress
    │
    ▼
app.rs::handle_key()
    ├─ clock_active check → state.clock_active = false (D-39.3)
    ├─ stopwatch_keyboard_mode check → handle_stopwatch_mode_key() (D-39.4)
    ├─ pending_input route (existing)
    ├─ alpha_mode route (existing)
    └─ normal key dispatch (existing)

Every 16ms tick (run() loop):
    │
    ├─ terminal.draw() → ui::get_display_string()
    │       ├─ clock_active? → core::get_clock_display_str(&state)  [D-39.1 highest priority]
    │       ├─ stopwatch_keyboard_mode? → core::get_stopwatch_display_str(&state)
    │       ├─ entry_buf? → format_entry_buf_display()
    │       ├─ prgm_mode? → prgm_display::format_step()
    │       ├─ alpha_mode? → format_alpha()
    │       └─ else → format_hpnum(X)
    │
    ├─ event::poll(16ms) → handle_key() [existing]
    │
    └─ check_alarms(&mut state) + drain event_buffer [D-39.7 NEW]
            ├─ alarm:message:* → self.message = Some(text)
            ├─ alarm:xeq:* → XEQ dispatch (via existing XEQ infrastructure)
            └─ BEEP/TONE → ignored
```

### Recommended Project Structure (no new directories)

All Phase 39 changes are modifications to existing files:

```
hp41-cli/src/
├── help_data.rs       # Add TIME_HELP_ENTRIES OnceLock + help_entries_time() + extend help_entries_all()
├── app.rs             # 3 changes: get_display_string priority, handle_key clock/SW checks, alarm drain
└── ui.rs              # get_display_string() — add clock/stopwatch priority at top

docs/
└── hp41-time-functions.json   # NEW: 35-entry canonical JSON

hp41-core/tests/
└── xrom_shadowing.rs          # Extend: add TIME_MODULE section (3 new tests)

hp41-cli/tests/
├── phase39_help_data_time.rs  # NEW: mirrors phase34_help_data_stat1.rs pattern
├── phase39_key_ref_includes_time.rs   # NEW: mirrors phase34_key_ref pattern
└── function_matrix_parity.rs  # Extend: add TIME_OP_VARIANT_NAMES + update partition test
```

### Pattern 1: Fourth OnceLock Pool (mirrors Phase 34 exactly)

[VERIFIED: codebase grep of `help_data.rs`]

```rust
// Source: hp41-cli/src/help_data.rs — copy of STAT1 block with "time" substituted
const TIME_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-time-functions.json");

static TIME_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

pub fn help_entries_time() -> &'static [HelpEntry] {
    TIME_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(TIME_FUNCTIONS_JSON)
            .expect("hp41-time-functions.json is malformed — fix the JSON")
    })
}

// Extended help_entries_all() — add 4th chain arm
pub fn help_entries_all() -> impl Iterator<Item = &'static HelpEntry> {
    help_entries()
        .iter()
        .chain(help_entries_math1().iter())
        .chain(help_entries_stat1().iter())
        .chain(help_entries_time().iter())   // NEW
}
```

### Pattern 2: Live Display Priority in get_display_string()

[VERIFIED: codebase grep of `ui.rs:131`]

```rust
// Source: hp41-cli/src/ui.rs — modified get_display_string()
fn get_display_string(app: &App) -> String {
    let st = &app.state;

    // D-39.1: clock/stopwatch HIGHEST priority — above entry_buf, prgm, alpha
    if let Some(s) = hp41_core::ops::time::clock::get_clock_display_str(st) {
        return s;
    }
    if let Some(s) = hp41_core::ops::time::stopwatch::get_stopwatch_display_str(st) {
        return s;
    }

    // Existing priority chain unchanged below
    if !st.entry_buf.is_empty() { ... }
    else if st.prgm_mode { ... }
    else if st.alpha_mode { ... }
    else { format_hpnum(&st.stack.x, &st.display_mode) }
}
```

### Pattern 3: Stopwatch Keyboard Mode Dispatch

[VERIFIED: codebase grep of `handle_alpha_mode_key` at `app.rs:1622`]

```rust
// Source: app.rs — new handle_stopwatch_mode_key method, called from handle_key()
// Mirrors handle_alpha_mode_key pattern exactly

// In handle_key() — before pending_input route and alpha_mode route:
if self.state.clock_active {
    self.state.clock_active = false;  // D-39.3: any key exits clock display
    // key falls through for normal processing
}

if self.state.stopwatch_keyboard_mode {
    self.handle_stopwatch_mode_key(key);
    return;
}

// New method:
fn handle_stopwatch_mode_key(&mut self, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            self.state.stopwatch_keyboard_mode = false;
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            // R/S equivalent: toggle start/stop
            // dispatch RUNSW if stopped, STOPSW if running
            if self.state.stopwatch_mode == StopwatchMode::Running {
                self.call_dispatch(Op::TimeStopsw);
            } else {
                self.call_dispatch(Op::TimeRunsw);
            }
        }
        KeyCode::Char('s') => {
            // Split: SWPT
            self.call_dispatch(Op::TimeSwpt);
        }
        KeyCode::Char('r') => {
            // Reset: STPW (stop + reset to Idle)
            self.call_dispatch(Op::TimeStpw);
        }
        _ => { /* ignore all other keys in stopwatch mode */ }
    }
}
```

**Note on key binding choices (Claude's Discretion):** Space/Enter for start/stop feels natural (mirrors R/S on real hardware); `s` for split (mnemonic: **S**plit); `r` for reset (mnemonic: **R**eset); Esc to exit. These must be documented in the help overlay's stopwatch section. The planner should choose bindings that don't conflict with each other.

### Pattern 4: Alarm Drain in run() Loop

[VERIFIED: `app.rs:266-277` loop structure; `alarm.rs:479` check_alarms signature]

```rust
// Source: app.rs::run() — insert AFTER check_autosave(), OUTSIDE event::poll conditional
pub fn run(&mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
    while !self.exit {
        terminal.draw(|frame| self.draw(frame))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            }
        }
        self.check_autosave();
        // D-39.7: check alarms on every tick (not just on keypress)
        hp41_core::ops::time::alarm::check_alarms(&mut self.state);
        self.drain_event_buffer();  // new helper method
    }
    ...
}

// New helper — drains state.event_buffer and routes events
fn drain_event_buffer(&mut self) {
    let events: Vec<String> = self.state.event_buffer.drain(..).collect();
    for event in events {
        if let Some(msg) = event.strip_prefix("alarm:message:") {
            self.message = Some(msg.to_string());
        } else if let Some(label) = event.strip_prefix("alarm:xeq:") {
            // Dispatch XEQ via existing XEQ infrastructure
            match hp41_core::run_program(&mut self.state, label) {
                Ok(()) => {}
                Err(e) => self.message = Some(format!("{e}")),
            }
        }
        // BEEP/TONE — ignored per D-39.6
    }
}
```

**Design note:** `drain_event_buffer()` must also be called from `call_dispatch()` and `call_dispatch_and_drain()` per D-39.6 (after the existing `print_buffer` drain). This ensures alarms fired mid-dispatch are surfaced immediately, not just on the 16ms tick.

### Pattern 5: JSON Schema (35 entries)

[VERIFIED: codebase grep of `docs/hp41-stat1-functions.json` + `TIME_MODULE.ops` in `xrom.rs`]

```json
// Source: docs/hp41-stat1-functions.json — canonical schema to follow exactly
{
    "op_variant": "TimeTime",
    "display_name": "TIME",
    "category": "Time Clock",
    "status": "implemented",
    "phase": "38",
    "key_path": "XEQ \"TIME\"",
    "description": "Recall current time to X as HH.MMSSss",
    "xrom": { "module": "Time", "module_id": 26, "function_id": 1 }
}
```

The 35 variants to include (derived from `TIME_MODULE.ops` in `xrom.rs:202-244`):

| Category | Mnemonics (display_name) | Op variant |
|----------|--------------------------|------------|
| Time Clock | TIME, DATE, T+X, CLOCK | TimeTime, TimeDate, TimeTplusx, TimeClock |
| Time Display | CLKT, CLKTD, SETIME, SETDATE | TimeClkt, TimeClktd, TimeSetime, TimeSetdate |
| Time Format | CLK12, CLK24, CORRECT, SETAF, RCLAF | TimeClk12, TimeClk24, TimeCorrect, TimeSetaf, TimeRclaf |
| Time Alpha | ATIME, ATIME24, ADATE | TimeAtime, TimeAtime24, TimeAdate |
| Time Date Arithmetic | DATE+, DDAYS, DOW, DMY, MDY | TimeDatePlus, TimeDdays, TimeDow, TimeDmy, TimeMdy |
| Time Stopwatch | RUNSW, STOPSW, SETSW, RCLSW, SWPT, STPW, SW | TimeRunsw, TimeStopsw, TimeSetsw, TimeRclsw, TimeSwpt, TimeStpw, TimeSw |
| Time Alarm | XYZALM, RCLALM, ALMCAT, CLALMA, CLALMX, CLRALMS, ALMNOW | TimeXyzalm, TimeRclalm, TimeAlmcat, TimeClalma, TimeClalmx, TimeClralms, TimeAlmnow |

Total: 4 + 4 + 5 + 3 + 5 + 7 + 7 = **35 entries**.

Divergence inline fields: SETAF/RCLAF (accuracy factor stored but inert per REQUIREMENTS), CORRECT (no-op per REQUIREMENTS), SW (extended stopwatch keyboard mode — emulator extension).

### Anti-Patterns to Avoid

- **Calling `check_alarms()` only on keypress:** D-39.7 requires it on every 16ms tick. Placing it inside the `event::poll` conditional means alarms only fire when the user presses keys — violating TIME-ALM-08 for idle states.
- **Adding `clock_active = false` ONLY at the start of clock-exit, then returning early:** D-39.3 specifies the key is still processed normally after clearing `clock_active`. Do not add an early `return` after the clear.
- **Using `_ =>` catch-all in prgm_display.rs:** The 4-way invariant item 3 requires NO catch-all. (Already confirmed this is satisfied — all 35 Time arms are present.)
- **Making `help_entries_all()` call `help_entries_time()` but NOT updating `function_matrix_parity.rs`:** The `test_pool_partition_is_exhaustive()` test currently rejects `module_id = 26` as a rogue value. Phase 39 must update that test to recognize the Time pool.
- **Forgetting to drain `event_buffer` inside `call_dispatch()` and `call_dispatch_and_drain()`:** D-39.6 requires both drain sites, not just the `run()` loop drain. Alarms triggered by a dispatch (e.g., `ALMNOW`) must surface immediately.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Clock display string | Custom time formatter | `hp41_core::ops::time::clock::get_clock_display_str(&state)` | Already implemented in Phase 38; handles 12/24h, TimeAndDate alternation |
| Stopwatch display string | Custom elapsed formatter | `hp41_core::ops::time::stopwatch::get_stopwatch_display_str(&state)` | Already implemented in Phase 38; handles `HH:MM:SS.hh` format |
| Alarm detection | Manual scan of alarm catalog | `hp41_core::ops::time::alarm::check_alarms(&mut state)` | Already implemented; handles past_due flag, event_buffer push |
| Stopwatch start/stop dispatch | Inline mode-toggle logic | `Op::TimeRunsw` / `Op::TimeStopsw` | Core ops handle all state transitions |
| Help overlay section ordering | Custom sort logic | `help_overlay_rows()` in `help_data.rs` | Auto-derives from JSON category strings; 4th pool appended via chain |

**Key insight:** All Time display computation was deliberately placed in `hp41-core` during Phase 38 per the pull-on-redraw architecture (D-carried.2). Phase 39 is purely a thin integration layer — it calls core functions and displays their output.

---

## Common Pitfalls

### Pitfall 1: Forgetting to Update function_matrix_parity.rs Partition Test

**What goes wrong:** After adding `help_entries_time()` to `help_entries_all()`, the `test_pool_partition_is_exhaustive()` test in `function_matrix_parity.rs` fails because it currently treats `module_id = 26` (Time) as a rogue/unknown module ID.

**Why it happens:** The test has a hardcoded allowlist: `None` (built-ins), `Some(7)` (Math 1), `Some(2)` (Stat 1). Any other `module_id` hits the `rogue` branch.

**How to avoid:** In the same plan that adds `help_entries_time()`, update `test_pool_partition_is_exhaustive()` to add `Some(26) => time_count += 1` and assert `time_count == 35`.

**Warning signs:** `test_pool_partition_is_exhaustive` fails with "Unknown xrom.module_id values: [("TimeTime", 26), ...]" after adding the JSON.

### Pitfall 2: ALL_OP_VARIANT_NAMES Out of Sync in function_matrix_parity.rs

**What goes wrong:** `test_op_inventory_count_matches_enum()` asserts `ALL_OP_VARIANT_NAMES.len() == 130`. After Phase 38 added 35 Time Op variants, this must be updated to 165 (130 + 35). The test currently passes because the Time variants are in `prgm_display.rs` but not yet in `ALL_OP_VARIANT_NAMES` — because the Time JSON pool hasn't been added yet and the test only runs parity against the v2.2 pool.

**Why it happens:** `ALL_OP_VARIANT_NAMES` is hand-curated and requires manual update when new Op variants are added.

**How to avoid:** Append all 35 `Time*` variant names to `ALL_OP_VARIANT_NAMES` and update the count assertion from 130 to 165.

**Warning signs:** `test_op_inventory_count_matches_enum` fails with "ALL_OP_VARIANT_NAMES out of sync".

### Pitfall 3: clock_active Clear Without Key Falling Through

**What goes wrong:** `clock_active` is cleared at top of `handle_key()` but the key is consumed (early return) instead of continuing. The user has to press TWO keys: one to exit clock mode, one to actually do something.

**Why it happens:** Copying the `alpha_mode` pattern too literally — alpha mode does `return` after routing; clock exit per D-39.3 must NOT return.

**How to avoid:** The clock-exit check is a MUTATION only (set `clock_active = false`) followed by `// fall through`. Do not add `return` after clearing.

**Warning signs:** Manual test — XEQ "CLKT" then press "+" — if it takes two keypresses for "+" to execute, this pitfall was hit.

### Pitfall 4: Alarm drain_event_buffer Missing from call_dispatch()

**What goes wrong:** Alarms triggered by ops like `ALMNOW` push to `event_buffer` immediately in the dispatch call. If `drain_event_buffer()` is only in `run()`, the alarm message doesn't appear until the NEXT 16ms tick — functionally fine but misses the "drain after every dispatch" guarantee of D-39.6.

**Why it happens:** Only adding the drain to `run()` (the obvious place for idle alarm checking) but forgetting the post-dispatch drain in `call_dispatch()` and `call_dispatch_and_drain()`.

**How to avoid:** Add `self.drain_event_buffer()` at the tail of both `call_dispatch()` and `call_dispatch_and_drain()`, after the existing `print_buffer` drain — same pattern as `maybe_auto_open_collect_for_modal()`.

### Pitfall 5: xrom_shadowing.rs Tests Missing TIME_MODULE Export

**What goes wrong:** `hp41-core/tests/xrom_shadowing.rs` uses `use hp41_core::ops::math1::xrom::{xrom_resolve, MATH_1, STAT_1}`. Adding `TIME_MODULE` to the use-list requires that it's pub in `xrom.rs`. It already is (`pub const TIME_MODULE`), but the test import must be updated.

**Why it happens:** Forgetting to add `TIME_MODULE` to the use statement in the test file.

**How to avoid:** Update the import to `use hp41_core::ops::math1::xrom::{xrom_resolve, MATH_1, STAT_1, TIME_MODULE}` in `xrom_shadowing.rs`.

### Pitfall 6: phase39_help_data_time.rs Counting 35 Instead of Verifying Exact Count

**What goes wrong:** The smoke test asserts `>= 35` instead of `== 35`. If a JSON entry is accidentally duplicated, the test doesn't catch it.

**Why it happens:** Copying the v2.2 help data test (which uses `>= 130`) instead of the Stat 1 pattern (which uses `== 26`).

**How to avoid:** Use `assert_eq!(entries.len(), 35, "...")` — exact count since Time Pac is feature-frozen for v3.2.

---

## Code Examples

### Example 1: OnceLock Extension (4th pool)

```rust
// Source: hp41-cli/src/help_data.rs — verified pattern from STAT1_FUNCTIONS_JSON block
const TIME_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-time-functions.json");
static TIME_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

pub fn help_entries_time() -> &'static [HelpEntry] {
    TIME_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(TIME_FUNCTIONS_JSON)
            .expect("hp41-time-functions.json is malformed — fix the JSON")
    })
}

pub fn help_entries_all() -> impl Iterator<Item = &'static HelpEntry> {
    help_entries()
        .iter()
        .chain(help_entries_math1().iter())
        .chain(help_entries_stat1().iter())
        .chain(help_entries_time().iter())  // 4th arm
}
```

### Example 2: get_display_string() Priority Insertion

```rust
// Source: hp41-cli/src/ui.rs:131 — verified; insert at top of function
fn get_display_string(app: &App) -> String {
    let st = &app.state;
    // D-39.1: clock/stopwatch override ALL other display modes
    if let Some(s) = hp41_core::ops::time::clock::get_clock_display_str(st) {
        return s;
    }
    if let Some(s) = hp41_core::ops::time::stopwatch::get_stopwatch_display_str(st) {
        return s;
    }
    // Existing chain continues...
    if !st.entry_buf.is_empty() { ... }
}
```

### Example 3: xrom_shadowing.rs Time Section

```rust
// Source: hp41-core/tests/xrom_shadowing.rs — mirrors STAT_1 section exactly
use hp41_core::ops::math1::xrom::{xrom_resolve, MATH_1, STAT_1, TIME_MODULE};

#[test]
fn time_names_do_not_shadow_builtins() {
    for (name, _op) in TIME_MODULE.ops {
        assert!(
            !BUILTIN_CARD_OP_NAMES.contains(name),
            "Time Module mnemonic {name:?} shadows a builtin_card_op entry."
        );
    }
}

#[test]
fn time_ops_disjoint_from_math1_and_stat1() {
    use std::collections::HashSet;
    let math1_names: HashSet<&str> = MATH_1.ops.iter().map(|(n, _)| *n).collect();
    let stat1_names: HashSet<&str> = STAT_1.ops.iter().map(|(n, _)| *n).collect();
    for (name, _op) in TIME_MODULE.ops {
        assert!(!math1_names.contains(name), "Time mnemonic {name:?} collides with MATH_1");
        assert!(!stat1_names.contains(name), "Time mnemonic {name:?} collides with STAT_1");
    }
}

#[test]
fn time_ops_resolve_via_xrom_resolve() {
    for (name, expected_op) in TIME_MODULE.ops {
        let resolved = xrom_resolve(name, 0b0000_0111);  // all 3 modules loaded
        assert_eq!(resolved.as_ref(), Some(expected_op),
            "TIME_MODULE mnemonic {name:?} must resolve via xrom_resolve(_, 0b0000_0111)");
    }
}

#[test]
fn time_const_fields() {
    assert_eq!(TIME_MODULE.id, 26, "TIME_MODULE.id must be 26");
    assert_eq!(TIME_MODULE.name, "TIME 2C");
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Static `HELP_DATA` const | `OnceLock<Vec<HelpEntry>>` + JSON `include_str!` | Phase 25 (D-25.16) | JSON is single source of truth; Phase 39 extends to 4 pools |
| `help_entries_all()` = 3 pools | 4-pool chain (+ Time) | Phase 39 | `help_overlay_rows()` auto-generates "Time Pac (XROM 26)" section |
| No live display modes | `get_clock_display_str` / `get_stopwatch_display_str` in core | Phase 38 | Pull-on-redraw: CLI calls on every frame; no new timers |

**No deprecated patterns in this phase.** All additions are additive to existing architecture.

---

## Assumptions Log

> All claims in this research were verified against the actual codebase. No `[ASSUMED]` tags required.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | All verified via codebase grep | — | — |

**This table is empty:** All claims in this research were verified against actual source files in the repository.

---

## Open Questions (RESOLVED)

1. **Stopwatch keyboard mode key bindings (Claude's Discretion)**
   - What we know: D-39.4 specifies R/S → start/stop toggle, one key for split, one key for reset, Esc to exit
   - What's unclear: Which physical keys map to start/stop toggle, split, and reset
   - Recommendation: Use Space/Enter for R/S equivalent, `s` for split, `r` for reset — all are mnemonic and unused in stopwatch mode context. Document in help overlay.
   - **RESOLVED:** Space/Enter for start/stop, `s` for split, `r` for reset, Esc to exit. Implemented in Plan 39-02 Task 2.

2. **drain_event_buffer position relative to drain_pending_card_op**
   - What we know: `call_dispatch_and_drain()` at line 1783 currently drains card ops then print_buffer
   - What's unclear: Whether `drain_event_buffer()` should go before or after `maybe_auto_open_collect_for_modal()`
   - Recommendation: After `print_buffer` drain, before `maybe_auto_open_collect_for_modal()`. Alarm messages should be set before the modal auto-opener runs.
   - **RESOLVED:** After `maybe_auto_open_collect_for_modal()` in both dispatch methods. Implemented in Plan 39-02 Task 2.

3. **function_matrix_parity.rs ALL_OP_VARIANT_NAMES count update**
   - What we know: Current assertion is `== 130`; Phase 38 added 35 Time variants
   - What's unclear: Has `ALL_OP_VARIANT_NAMES` already been updated to 165 in any prior commit?
   - Recommendation: Planner should check `ALL_OP_VARIANT_NAMES.len()` assertion value before deciding whether this is a new task or already done. Current grep shows the test asserts 130 exactly.
   - **RESOLVED:** Not yet updated; Plan 39-01 Task 2 updates to 165.

---

## Environment Availability

> This phase is purely code modifications to the existing Rust workspace with zero external dependencies. All tools confirmed available.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable / cargo | Build | ✓ | MSRV 1.88 | — |
| `just` | Task runner | ✓ | (project std) | — |
| `hp41-core` Phase 38 | Time core ops | ✓ | v3.2 (latest) | — |

**Missing dependencies with no fallback:** None.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + cargo test |
| Config file | None (workspace default) |
| Quick run command | `cargo test -p hp41-cli --test phase39_help_data_time 2>&1` |
| Full suite command | `just test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TIME-CLI-01 | JSON loads, 35 entries, no malformed | unit (smoke) | `cargo test -p hp41-cli --test phase39_help_data_time` | ❌ Wave 0 |
| TIME-CLI-02 | 4-pool chain returns 201 entries total | unit | `cargo test -p hp41-cli --test phase39_help_data_time` | ❌ Wave 0 |
| TIME-CLI-03 | All 35 Time arms in prgm_display.rs | compile-time | `cargo build -p hp41-cli` | ✅ **ALREADY SATISFIED** |
| TIME-CLI-04 | "Time Pac (XROM 26)" section in overlay | unit | `cargo test -p hp41-cli --test phase39_key_ref_includes_time` | ❌ Wave 0 |
| TIME-CLI-05 | Clock display renders when clock_active | unit/integration | `cargo test -p hp41-cli --test phase39_live_display` | ❌ Wave 0 |
| TIME-CLI-06 | Stopwatch mode renders HH:MM:SS.hh | unit/integration | `cargo test -p hp41-cli --test phase39_live_display` | ❌ Wave 0 |
| TIME-CLI-07 | Alarm message → self.message | unit/integration | `cargo test -p hp41-cli --test phase39_alarm_drain` | ❌ Wave 0 |
| TIME-CLI-08 | TIME_MODULE mnemonics disjoint | unit | `cargo test -p hp41-core --test xrom_shadowing` | ✅ (file exists; needs extension) |

### Sampling Rate

- **Per task commit:** `cargo test -p hp41-cli` (all CLI tests)
- **Per wave merge:** `just test` (full workspace suite)
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `hp41-cli/tests/phase39_help_data_time.rs` — covers TIME-CLI-01, TIME-CLI-02 (mirrors `phase34_help_data_stat1.rs`)
- [ ] `hp41-cli/tests/phase39_key_ref_includes_time.rs` — covers TIME-CLI-04 (mirrors `phase34_key_ref_includes_stat1.rs`)
- [ ] `hp41-cli/tests/phase39_live_display.rs` — covers TIME-CLI-05, TIME-CLI-06 (new test type; no direct precedent but uses `App::new_for_test()`)
- [ ] `hp41-cli/tests/phase39_alarm_drain.rs` — covers TIME-CLI-07 (integration: dispatch → check_alarms → event drain → message)

---

## Security Domain

> This phase adds no authentication, session management, cryptography, or network access. The only external input is the `hp41-time-functions.json` file embedded at compile time via `include_str!` — no runtime file I/O. ASVS categories V2/V3/V4/V6 are not applicable.

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | Partial | `serde_json::from_str` + `.expect()` panic on malformed JSON (D-25.17 hard-build-blocker) |

---

## Sources

### Primary (HIGH confidence)
- Codebase: `hp41-cli/src/help_data.rs` — OnceLock pattern, `help_entries_all()` chain (3 pools)
- Codebase: `hp41-cli/src/app.rs` — `run()` loop structure, `handle_key()` ordering, `call_dispatch_and_drain()`
- Codebase: `hp41-cli/src/ui.rs:131` — `get_display_string()` priority chain
- Codebase: `hp41-cli/src/prgm_display.rs:328-365` — 35 Time Op arms already present
- Codebase: `hp41-core/src/ops/time/clock.rs:297` — `get_clock_display_str()` pub fn signature
- Codebase: `hp41-core/src/ops/time/stopwatch.rs:190` — `get_stopwatch_display_str()` pub fn signature
- Codebase: `hp41-core/src/ops/time/alarm.rs:479` — `check_alarms()` pub fn signature
- Codebase: `hp41-core/src/ops/math1/xrom.rs:199-244` — `TIME_MODULE` with 35 ops entries
- Codebase: `hp41-core/tests/xrom_shadowing.rs` — STAT_1 section pattern to mirror
- Codebase: `hp41-cli/tests/function_matrix_parity.rs:510-545` — partition test that needs updating
- Codebase: `hp41-cli/tests/phase34_help_data_stat1.rs` — exact template for new phase39 test
- Codebase: `docs/hp41-stat1-functions.json` — JSON schema template (26 entries)
- Codebase: `hp41-core/src/state.rs:290-354` — Phase 38 CalcState fields verified

### Secondary (MEDIUM confidence)
- CLAUDE.md frozen invariants — resolver chain order, 4-way exhaustive match, save-file backward compat

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all required functions already exist; verified via source inspection
- Architecture: HIGH — all integration points verified via codebase grep; no new patterns introduced
- Pitfalls: HIGH — derived from actual test code that will break if pitfalls are hit
- Test gaps: HIGH — file existence verified via `ls`

**Research date:** 2026-05-25
**Valid until:** 2026-06-25 (stable codebase; no external API dependency)
