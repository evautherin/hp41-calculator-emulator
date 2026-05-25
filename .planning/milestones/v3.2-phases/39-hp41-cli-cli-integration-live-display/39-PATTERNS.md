# Phase 39: hp41-cli — CLI Integration + Live Display - Pattern Map

**Mapped:** 2026-05-25
**Files analyzed:** 9 new/modified files
**Analogs found:** 9 / 9

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `docs/hp41-time-functions.json` | config (static data) | transform | `docs/hp41-stat1-functions.json` | exact |
| `hp41-cli/src/help_data.rs` | utility (data provider) | request-response | `hp41-cli/src/help_data.rs` (stat1 block) | exact |
| `hp41-cli/src/app.rs` (`get_display_string`) | controller | request-response | `hp41-cli/src/ui.rs:131` existing chain | exact |
| `hp41-cli/src/app.rs` (`handle_key` clock/SW) | controller | event-driven | `hp41-cli/src/app.rs:1622` `handle_alpha_mode_key` | role-match |
| `hp41-cli/src/app.rs` (`run` loop alarm drain) | controller | event-driven | `hp41-cli/src/app.rs:251` `check_autosave` | role-match |
| `hp41-cli/src/app.rs` (`call_dispatch` drain) | controller | request-response | `hp41-cli/src/app.rs:1783` `call_dispatch_and_drain` | exact |
| `hp41-cli/tests/phase39_help_data_time.rs` | test | request-response | `hp41-cli/tests/phase34_help_data_stat1.rs` | exact |
| `hp41-cli/tests/phase39_key_ref_includes_time.rs` | test | request-response | `hp41-cli/tests/phase34_key_ref_includes_stat1.rs` | exact |
| `hp41-cli/tests/function_matrix_parity.rs` (extend) | test | transform | `hp41-cli/tests/function_matrix_parity.rs:510` partition test | exact |
| `hp41-core/tests/xrom_shadowing.rs` (extend) | test | request-response | `hp41-core/tests/xrom_shadowing.rs:93` STAT_1 section | exact |

---

## Pattern Assignments

### `docs/hp41-time-functions.json` (config, transform)

**Analog:** `docs/hp41-stat1-functions.json`

**Schema pattern** (lines 1-11 of stat1 JSON):
```json
[
    {
        "op_variant": "SigmaBstat",
        "display_name": "ΣBSTAT",
        "category": "Stat1 Univariate",
        "status": "implemented",
        "phase": "33",
        "key_path": "XEQ \"ΣBSTAT\"",
        "description": "Extended univariate summary (weighted mean, CV)",
        "xrom": { "module": "Stat 1", "module_id": 2, "function_id": 1 }
    }
]
```

**Adaptations for Time:**
- `"category"` uses the prefix `"Time "` (e.g., `"Time Clock"`, `"Time Stopwatch"`) — parallel to `"Stat1 "` prefix from D-34.1, but for the Time Pac the CONTEXT uses plain `"Time Clock"` etc. per D-39.9
- `"xrom"` block: `{ "module": "Time", "module_id": 26, "function_id": N }` per D-39.10
- `"key_path"`: `"XEQ \"MNEMONIC\""` for all 35 entries (XEQ-by-name only per D-39.10)
- `"phase"`: `"39"` for all entries (first exposure in this phase)
- `"status"`: `"implemented"` for all 35 entries
- **Inline divergences** (D-39.11): SETAF, RCLAF, CORRECT, SW only — all other entries omit the `divergences` field entirely

**35 entries across 7 categories** (D-39.9):
- `"Time Clock"` — TIME, DATE, T+X, CLOCK (function_id 1-4)
- `"Time Date Arithmetic"` — DATE+, DDAYS, DOW, DMY, MDY (5-9)
- `"Time Display"` — CLKT, CLKTD, SETIME, SETDATE (10-13)
- `"Time Format"` — CLK12, CLK24, CORRECT, SETAF, RCLAF (14-18)
- `"Time Alpha"` — ATIME, ATIME24, ADATE (19-21)
- `"Time Stopwatch"` — RUNSW, STOPSW, SETSW, RCLSW, SWPT, STPW, SW (22-28)
- `"Time Alarm"` — XYZALM, RCLALM, ALMCAT, CLALMA, CLALMX, CLRALMS, ALMNOW (29-35)

---

### `hp41-cli/src/help_data.rs` — fourth OnceLock pool (utility, request-response)

**Analog:** `hp41-cli/src/help_data.rs` lines 124-169 (STAT1 block + `help_entries_all`)

**Imports pattern** (lines 20-22, unchanged):
```rust
use std::sync::OnceLock;
use serde::Deserialize;
```

**Fourth OnceLock pool pattern** (copy of lines 124-147, substitute "stat1" → "time", "2" → "26"):
```rust
// Source: hp41-cli/src/help_data.rs lines 124-147
const STAT1_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-stat1-functions.json");
static STAT1_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

pub fn help_entries_stat1() -> &'static [HelpEntry] {
    STAT1_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(STAT1_FUNCTIONS_JSON)
            .expect("hp41-stat1-functions.json is malformed — fix the JSON")
    })
}
```

**New Time variant (add after line 147):**
```rust
const TIME_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-time-functions.json");
static TIME_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

pub fn help_entries_time() -> &'static [HelpEntry] {
    TIME_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(TIME_FUNCTIONS_JSON)
            .expect("hp41-time-functions.json is malformed — fix the JSON")
    })
}
```

**`help_entries_all()` extension pattern** (lines 164-169, add 4th arm):
```rust
// Source: hp41-cli/src/help_data.rs lines 164-169 — CURRENT (3 pools):
pub fn help_entries_all() -> impl Iterator<Item = &'static HelpEntry> {
    help_entries()
        .iter()
        .chain(help_entries_math1().iter())
        .chain(help_entries_stat1().iter())
        // ADD: fourth arm per D-39.12
        .chain(help_entries_time().iter())
}
```

**Doc-comment pattern** (lines 149-163) — copy verbatim and update pool counts:
- Change "three" → "four", "three-pool" → "4-pool", add `[`help_entries_time`]` to the narrow-accessor retention note, update count to "v2.2 + Math 1 + Stat 1 + Time"

---

### `hp41-cli/src/ui.rs` — `get_display_string()` priority insertion (controller, request-response)

**Analog:** `hp41-cli/src/ui.rs:131-152` existing `get_display_string()`

**Current priority chain pattern** (lines 129-152):
```rust
// Source: hp41-cli/src/ui.rs lines 129-152
/// Get the string to show in the HP-41 display area.
/// Priority: entry_buf > prgm step > alpha > formatted X.
fn get_display_string(app: &App) -> String {
    let st = &app.state;
    if !st.entry_buf.is_empty() {
        if st.entry_buf.contains('e') {
            format_entry_buf_display(&st.entry_buf)
        } else {
            st.entry_buf.clone()
        }
    } else if st.prgm_mode {
        prgm_display::format_step(st)
    } else if st.alpha_mode {
        format_alpha(&st.alpha_reg)
    } else {
        format_hpnum(&st.stack.x, &st.display_mode)
    }
}
```

**Modified version — insert at top (D-39.1), before existing `entry_buf` check:**
```rust
/// Priority: clock_active > stopwatch_keyboard_mode > entry_buf > prgm step > alpha > formatted X.
fn get_display_string(app: &App) -> String {
    let st = &app.state;

    // D-39.1: clock/stopwatch override ALL other display modes (highest priority)
    if let Some(s) = hp41_core::ops::time::clock::get_clock_display_str(st) {
        return s;
    }
    if let Some(s) = hp41_core::ops::time::stopwatch::get_stopwatch_display_str(st) {
        return s;
    }

    // Existing chain — unchanged below this line:
    if !st.entry_buf.is_empty() { ... }
    else if st.prgm_mode { ... }
    else if st.alpha_mode { ... }
    else { format_hpnum(&st.stack.x, &st.display_mode) }
}
```

**Critical note (D-39.3 / Pitfall 3):** `clock_active` clear does NOT return early — only sets the flag false, then falls through. This is in `handle_key()` not `get_display_string()`.

---

### `hp41-cli/src/app.rs` — `handle_key()` clock-exit + stopwatch-mode intercepts (controller, event-driven)

**Analog:** `hp41-cli/src/app.rs:449-455` `alpha_mode` intercept pattern

**Alpha-mode pattern to copy** (lines 449-455):
```rust
// Source: hp41-cli/src/app.rs lines 449-455
// Phase 5: ALPHA mode routing (D-12) — must be BEFORE digit-entry block
if self.state.alpha_mode {
    self.handle_alpha_mode_key(key);
    return;   // ← key is CONSUMED; normal routing skipped
}
```

**Clock-exit pattern** (insert BEFORE `alpha_mode` check, after key-release filter):
```rust
// D-39.3: clock display exits on ANY keypress — clear flag but DO NOT return
// (key falls through for normal processing — do NOT copy alpha_mode's early return)
if self.state.clock_active {
    self.state.clock_active = false;
    // NO return here — key still processed normally
}
```

**Stopwatch-mode pattern** (insert AFTER clock-exit, BEFORE `alpha_mode` check):
```rust
// D-39.4: stopwatch keyboard mode — top-level check before PendingInput routing
// Mirrors alpha_mode pattern: dedicated handler + early return
if self.state.stopwatch_keyboard_mode {
    self.handle_stopwatch_mode_key(key);
    return;
}
```

**New `handle_stopwatch_mode_key` method** (copy `handle_alpha_mode_key` structure at lines 1619-1648):
```rust
// Source: hp41-cli/src/app.rs lines 1619-1648 — handle_alpha_mode_key pattern
fn handle_alpha_mode_key(&mut self, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => {
            self.call_dispatch(Op::AlphaToggle);
        }
        KeyCode::Char('a') => {
            self.call_dispatch(Op::AlphaToggle);
        }
        KeyCode::Backspace => {
            self.call_dispatch(Op::AlphaBackspace);
        }
        KeyCode::Char(c) => {
            self.call_dispatch(Op::AlphaAppend(c));
        }
        KeyCode::Delete => {
            self.call_dispatch(Op::AlphaClear);
        }
        _ => { /* ignored */ }
    }
}
```

**Stopwatch variant (Claude's Discretion: Space/Enter=start/stop, `s`=split, `r`=reset, Esc=exit):**
```rust
fn handle_stopwatch_mode_key(&mut self, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            self.state.stopwatch_keyboard_mode = false;
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            // R/S equivalent: toggle start/stop
            if self.state.stopwatch_running {
                self.call_dispatch(Op::TimeStopsw);
            } else {
                self.call_dispatch(Op::TimeRunsw);
            }
        }
        KeyCode::Char('s') => {
            self.call_dispatch(Op::TimeSwpt);  // Split
        }
        KeyCode::Char('r') => {
            self.call_dispatch(Op::TimeStpw);  // Reset
        }
        _ => { /* all other keys ignored in stopwatch mode */ }
    }
}
```

**Ordering constraint** (from `handle_key` flow at line 296): clock-exit check → stopwatch check → `pending_input` route (line ~228, already returned by this point) → alpha_mode check (line 452).

---

### `hp41-cli/src/app.rs` — `run()` loop alarm drain (controller, event-driven)

**Analog:** `hp41-cli/src/app.rs:251-259` `check_autosave()` pattern — called outside `event::poll` conditional

**Current run() loop pattern** (lines 265-277):
```rust
// Source: hp41-cli/src/app.rs lines 265-283
pub fn run(&mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
    while !self.exit {
        terminal.draw(|frame| self.draw(frame))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            }
        }
        // check_autosave is called OUTSIDE poll conditional — on every 16ms tick
        self.check_autosave();
    }
    ...
}
```

**Extension — add alarm check after `check_autosave()` (D-39.7):**
```rust
self.check_autosave();
// D-39.7: check alarms on every 16ms tick, NOT just on keypress
hp41_core::ops::time::alarm::check_alarms(&mut self.state);
self.drain_event_buffer();
```

**New `drain_event_buffer` method** — drain protocol per D-39.6:
```rust
/// Drain state.event_buffer and route alarm events to self.message or XEQ dispatch.
/// Called after every check_alarms() invocation AND after call_dispatch/call_dispatch_and_drain.
fn drain_event_buffer(&mut self) {
    let events: Vec<String> = self.state.event_buffer.drain(..).collect();
    for event in events {
        if let Some(msg) = event.strip_prefix("alarm:message:") {
            self.message = Some(msg.to_string());
        } else if let Some(label) = event.strip_prefix("alarm:xeq:") {
            match hp41_core::run_program(&mut self.state, label) {
                Ok(()) => {}
                Err(e) => self.message = Some(format!("{e}")),
            }
        }
        // BEEP/TONE — ignored per D-39.6
    }
}
```

---

### `hp41-cli/src/app.rs` — `call_dispatch` and `call_dispatch_and_drain` event drain (controller, request-response)

**Analog:** `hp41-cli/src/app.rs:1768-1817` — existing dispatch/drain methods

**`call_dispatch` pattern** (lines 1768-1777):
```rust
// Source: hp41-cli/src/app.rs lines 1768-1777
pub fn call_dispatch(&mut self, op: Op) {
    match hp41_core::ops::dispatch(&mut self.state, op) {
        Ok(()) => self.message = None,
        Err(e) => self.message = Some(format!("{e}")),
    }
    self.maybe_auto_open_collect_for_modal();
    // ADD per D-39.6: drain event_buffer after dispatch (alarms fired mid-op surface immediately)
    self.drain_event_buffer();
}
```

**`call_dispatch_and_drain` pattern** (lines 1783-1817, tail):
```rust
// Source: hp41-cli/src/app.rs lines 1783-1817
pub(crate) fn call_dispatch_and_drain(&mut self, op: Op) {
    match hp41_core::ops::dispatch(&mut self.state, op) {
        Ok(()) => {
            let card_err = self.drain_pending_card_op();
            let lines: Vec<String> = self.state.print_buffer.drain(..).collect();
            // ... existing print drain logic ...
        }
        Err(e) => self.message = Some(format!("{e}")),
    }
    self.maybe_auto_open_collect_for_modal();
    // ADD per D-39.6: drain event_buffer AFTER print_buffer drain
    self.drain_event_buffer();
}
```

**Ordering:** `drain_event_buffer()` goes AFTER `maybe_auto_open_collect_for_modal()` in both methods (Open Question 2 answer: alarm messages set before any side-effects from modal auto-open).

---

### `hp41-cli/tests/phase39_help_data_time.rs` (test, request-response)

**Analog:** `hp41-cli/tests/phase34_help_data_stat1.rs` — exact template, substitute "stat1" → "time", "26" → "35", "2" → "26" (module_id), "Stat1 " → "Time " (category prefix)

**Imports pattern** (lines 15-20 of phase34):
```rust
// Source: hp41-cli/tests/phase34_help_data_stat1.rs lines 15-20
#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use hp41_cli::help_data::{help_entries, help_entries_all, help_entries_math1, help_entries_stat1};
```

**Time variant:** add `help_entries_time` to the use statement.

**Test 2 — exact count** (lines 35-46):
```rust
// Source: hp41-cli/tests/phase34_help_data_stat1.rs lines 35-46
fn stat1_help_entries_count_meets_26_target() {
    let entries = help_entries_stat1();
    assert_eq!(
        entries.len(),
        26,    // ← Time analog: 35
        "help_entries_stat1().len() = {} — must be exactly 26",
        entries.len()
    );
}
```

**Test 6 — xrom module_id** (lines 100-121): substitute `module_id == 2` → `module_id == 26`

**Test 7 — category prefix** (lines 125-138): substitute `"Stat1 "` → `"Time "` prefix check

**Test 8 — dense function_id range** (lines 141-185): substitute `26` → `35` for all bound checks

**Test 11 — `help_entries_all` chain count** (lines 235-248): update assertion to `>= 201 + 35 = 236`:
```rust
// Source: hp41-cli/tests/phase34_help_data_stat1.rs lines 235-248
fn help_entries_all_returns_three_pools() {
    let all_count = help_entries_all().count();
    assert!(
        all_count >= 201,   // ← Time analog: >= 236 (201 + 35)
        "help_entries_all() returned {all_count} entries ..."
    );
}
```

**Test 10 — divergences are surgical** (lines 211-232): Time divergences are SETAF, RCLAF, CORRECT, SW (D-39.11) — 4 entries expected. Substitute variant names accordingly.

---

### `hp41-cli/tests/phase39_key_ref_includes_time.rs` (test, request-response)

**Analog:** `hp41-cli/tests/phase34_key_ref_includes_stat1.rs` — exact template

**All 3 tests** (lines 40-95) translate directly:
```rust
// Source: hp41-cli/tests/phase34_key_ref_includes_stat1.rs lines 40-95

// Test 1: TIME entry excluded from right-panel (use "TIME" as sentinel):
fn key_ref_entries_excludes_time_pac_time() {
    let leaked = entries.iter().any(|(key_path, display)| {
        key_path == "XEQ \"TIME\"" && display == "TIME"
    });
    assert!(!leaked, "key_ref_entries() must NOT include XEQ \"TIME\" ...");
}

// Test 2: Another ASCII-named Time entry excluded (use "DATE" or "CLKT"):
fn key_ref_entries_excludes_time_pac_clkt()  { ... }

// Test 3: v2.2 built-in entries still present (regression guard, unchanged):
fn key_ref_entries_preserves_v22_entries()  { ... }
```

---

### `hp41-cli/tests/function_matrix_parity.rs` — extension (test, transform)

**Analog:** `hp41-cli/tests/function_matrix_parity.rs:510-545` — `test_pool_partition_is_exhaustive`

**Partition test extension** (lines 510-545, add `time_count` branch):
```rust
// Source: hp41-cli/tests/function_matrix_parity.rs lines 521-544
let mut builtin_count = 0usize;
let mut math1_count = 0usize;
let mut stat1_count = 0usize;
// ADD:
let mut time_count = 0usize;
let mut rogue: Vec<(String, u8)> = Vec::new();
for entry in help_entries_all() {
    match entry.xrom.as_ref().map(|x| x.module_id) {
        None => builtin_count += 1,
        Some(7) => math1_count += 1,
        Some(2) => stat1_count += 1,
        Some(26) => time_count += 1,   // ADD
        Some(other) => rogue.push((entry.op_variant.clone(), other)),
    }
}
// ADD assertion:
assert_eq!(time_count, 35, "Time pool count drift: {time_count}");
```

**`ALL_OP_VARIANT_NAMES` count update** (line 203-208 — Pitfall 2):
```rust
// Source: hp41-cli/tests/function_matrix_parity.rs lines 202-208
assert_eq!(
    ALL_OP_VARIANT_NAMES.len(),
    130,    // ← update to 165 (130 + 35 Time variants)
    "ALL_OP_VARIANT_NAMES out of sync ..."
);
```

Also add all 35 `Time*` variant name strings to the `ALL_OP_VARIANT_NAMES` const (see `prgm_display.rs:329-363` for the canonical list): TimeAdate, TimeAlmcat, TimeAlmnow, TimeAtime, TimeAtime24, TimeClk12, TimeClk24, TimeClkt, TimeClktd, TimeClock, TimeCorrect, TimeDate, TimeDatePlus, TimeDdays, TimeDmy, TimeDow, TimeMdy, TimeRclaf, TimeRclalm, TimeRclsw, TimeRunsw, TimeSetaf, TimeSetdate, TimeSetime, TimeSetsw, TimeStopsw, TimeSw, TimeTplusx, TimeTime, TimeXyzalm, TimeClalma, TimeClalmx, TimeClralms, TimeSwpt, TimeStpw.

---

### `hp41-core/tests/xrom_shadowing.rs` — Time section (test, request-response)

**Analog:** `hp41-core/tests/xrom_shadowing.rs:93-160` — STAT_1 section

**Import extension** (line 26):
```rust
// Source: hp41-core/tests/xrom_shadowing.rs line 26 — CURRENT:
use hp41_core::ops::math1::xrom::{xrom_resolve, MATH_1, STAT_1};
// EXTEND TO:
use hp41_core::ops::math1::xrom::{xrom_resolve, MATH_1, STAT_1, TIME_MODULE};
```

**New section pattern** (copy lines 93-160, substitute STAT_1/stat1 → TIME_MODULE/time):
```rust
// Source: hp41-core/tests/xrom_shadowing.rs lines 99-109 — stat1_names_do_not_shadow_builtins:
#[test]
fn stat1_names_do_not_shadow_builtins() {
    for (name, _op) in STAT_1.ops {
        assert!(
            !BUILTIN_CARD_OP_NAMES.contains(name),
            "Stat 1 Pac mnemonic {name:?} shadows a builtin_card_op entry. ..."
        );
    }
}
```

**Time variant (4 new tests mirroring the 4 STAT_1 tests):**
1. `time_names_do_not_shadow_builtins` — iterate `TIME_MODULE.ops`, check against `BUILTIN_CARD_OP_NAMES`
2. `time_ops_disjoint_from_math1_and_stat1` — check against `MATH_1.ops` HashSet AND `STAT_1.ops` HashSet (expanded vs stat1 which only checks math1)
3. `time_ops_resolve_via_xrom_resolve` — `xrom_resolve(name, 0b0000_0111)` (all 3 modules loaded = bit mask for Math1|Stat1|Time)
4. `time_const_fields` — `assert_eq!(TIME_MODULE.id, 26)` and `assert_eq!(TIME_MODULE.name, "TIME 2C")` (or whatever the actual name is — verify from `xrom.rs`)

**Resolve bit-mask note:** the STAT_1 test uses `0b0000_0011` (Math1=bit0 + Stat1=bit1). Time=bit2 adds `0b0000_0100`. All three loaded = `0b0000_0111`.

---

## Shared Patterns

### OnceLock + include_str! + hard-build-blocker
**Source:** `hp41-cli/src/help_data.rs` lines 85-100 (original FUNCTIONS_JSON pattern)
**Apply to:** TIME_FUNCTIONS_JSON + TIME_HELP_ENTRIES + help_entries_time()
```rust
// Source: hp41-cli/src/help_data.rs lines 85-100
const FUNCTIONS_JSON: &str = include_str!("../../docs/hp41cv-functions.json");
static HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

pub fn help_entries() -> &'static [HelpEntry] {
    HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(FUNCTIONS_JSON)
            .expect("hp41cv-functions.json is malformed — fix the JSON")
    })
}
```

### Status-bar message pattern (`self.message`)
**Source:** `hp41-cli/src/app.rs` lines 321, 386, 519-522
**Apply to:** `drain_event_buffer()` alarm message routing
```rust
// Source: app.rs line 321 (success case) + line 386 (clear case)
self.message = Some(format!("Saved to {}", self.state_path.display()));
self.message = None;
// Error case (line 522):
Err(e) => self.message = Some(format!("{e}")),
```

### Post-dispatch method call chain pattern
**Source:** `hp41-cli/src/app.rs` lines 1768-1777 and 1783-1817
**Apply to:** Adding `drain_event_buffer()` to both dispatch methods
The canonical tail sequence is:
1. `match hp41_core::ops::dispatch(...)` — error mapped to `self.message`
2. `drain_pending_card_op()` (in `call_dispatch_and_drain` only)
3. `state.print_buffer.drain(..)` (in `call_dispatch_and_drain` only)
4. `maybe_auto_open_collect_for_modal()` (both methods)
5. `drain_event_buffer()` — NEW, both methods, appended at tail

### Exhaustive match (no `_ =>`) in prgm_display.rs
**Source:** `hp41-cli/src/prgm_display.rs` lines 328-365
**Status:** Already complete — all 35 Time arms present from Phase 38. No changes needed.
**Apply to:** Only confirm with `cargo build -p hp41-cli` (compile-time gate).

---

## No Analog Found

All Phase 39 files have exact or role-match analogs in the codebase. No file requires falling back to RESEARCH.md patterns exclusively.

The two novel elements (live display priority insertion and alarm drain) have partial analogs:
- `get_display_string` priority chain: exact analog in the existing chain; insertion point is well-defined
- `drain_event_buffer`: closest analog is `drain_and_show_print_buffer` at lines 1714-1736 (same drain-then-route-to-message pattern, applied to `print_buffer`; Time adapter routes `event_buffer` strings instead of print lines)

---

## Critical Ordering Constraints (for planner)

1. **JSON first:** `docs/hp41-time-functions.json` must exist before `help_data.rs` compiles (include_str! compile-time embed)
2. **`prgm_display.rs` already complete:** TIME-CLI-03 requires zero implementation — all 35 arms confirmed at `prgm_display.rs:328-363`
3. **Partition test blocks merge:** `function_matrix_parity.rs::test_pool_partition_is_exhaustive` currently rejects `module_id=26` as rogue. This test update MUST be in the same plan as `help_entries_time()` + JSON authoring, or CI will fail immediately after the JSON is embedded
4. **`ALL_OP_VARIANT_NAMES` count:** Must be updated from 130 → 165 in the same plan as adding 35 Time JSON entries
5. **`drain_event_buffer` shared method:** Must be defined before it is called from both `call_dispatch`, `call_dispatch_and_drain`, and `run()`

---

## Metadata

**Analog search scope:** `hp41-cli/src/`, `hp41-cli/tests/`, `hp41-core/tests/`, `docs/`
**Files scanned:** 10 source files read in full
**Pattern extraction date:** 2026-05-25
