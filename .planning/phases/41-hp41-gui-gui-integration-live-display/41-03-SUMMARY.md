---
phase: 41-hp41-gui-gui-integration-live-display
plan: "03"
subsystem: hp41-gui
tags: [react, typescript, tauri, live-display, alarm-events, modal-prompt, time-pac]
dependency_graph:
  requires:
    - "41-01"  # tick_time command + CalcStateView clock_active/stopwatch_keyboard_mode fields
  provides:
    - "App.tsx setInterval management for live clock/stopwatch display (TIME-GUI-04, TIME-GUI-05)"
    - "App.tsx alarm event parsing: alarm:message and alarm:xeq routing (TIME-GUI-06)"
    - "lcd_alternation_modal_prompt_time.rs: 3 TimeStep modal prompt integration tests (TIME-GUI-07)"
  affects:
    - hp41-gui frontend display loop (100ms tick interval)
    - alarm event routing pipeline
tech_stack:
  added: []
  patterns:
    - "useRef<ReturnType<typeof setInterval> | null> for interval management"
    - "busyRef guard in setInterval callback (read-only tick must not block user input)"
    - "alarm event prefix routing: alarm:message: → toast, alarm:xeq: → dispatch_op"
    - "ModalProgram::Time(TimeStep) LCD-alternation integration test pattern (mirrors Phase 36 stat1)"
key_files:
  created:
    - hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt_time.rs
  modified:
    - hp41-gui/src/App.tsx
    - hp41-gui/src/App.test.tsx
decisions:
  - "liveTickRef does NOT set busyRef.current=true — tick_time is read-only and must never block user input (RESEARCH.md Pitfall 2)"
  - "setInterval cleanup function prevents dangling interval in React StrictMode (RESEARCH.md Pitfall 3)"
  - "alarm:xeq:{label} dispatches as xeq_{label} through existing dispatch_op infrastructure"
metrics:
  completed: "2026-05-25"
  tasks_completed: 2
  tasks_total: 2
  files_created: 1
  files_modified: 2
---

# Phase 41 Plan 03: React Frontend Live Display + Alarm Event Parsing Summary

React frontend wired for live clock/stopwatch display via conditional 100ms setInterval calling `tick_time`, alarm event parsing with `alarm:message:` prefix stripping and `alarm:xeq:` dispatch routing, and 3 TimeStep modal prompt LCD-alternation integration tests.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | App.tsx — TypeScript interface + setInterval management + alarm event parsing + vitest coverage | `1e48f7c` | `hp41-gui/src/App.tsx`, `hp41-gui/src/App.test.tsx` |
| 2 | LCD-alternation modal prompt integration tests for TimeStep | `92ed512` | `hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt_time.rs` |

## Requirements Satisfied

| Requirement | Description | Verification |
|-------------|-------------|--------------|
| TIME-GUI-04 | Clock display mode triggers setInterval that invokes tick_time, updating LCD live | App.tsx useEffect with 100ms interval + busyRef guard |
| TIME-GUI-05 | Stopwatch mode uses same setInterval mechanism for live LCD updates | Same useEffect observes `stopwatch_keyboard_mode` |
| TIME-GUI-06 | Alarm events parsed: alarm:message → toast (prefix stripped), alarm:xeq → dispatch_op | Vitest D4 + D5 assertions pass |
| TIME-GUI-07 | All 3 TimeStep modal prompts verified by integration tests | 3/3 cargo tests pass |

## Implementation Details

### Task 1: App.tsx Modifications

**A) CalcStateView TS interface** — added `clock_active: boolean` and `stopwatch_keyboard_mode: boolean` fields after `modal_prompt`. Without these, the setInterval condition evaluates to `undefined || undefined = false` and the interval never starts (RESEARCH.md Pitfall 8).

**B) liveTickRef declaration** — `const liveTickRef = useRef<ReturnType<typeof setInterval> | null>(null)` near `busyRef`. Holds the interval ID when live display is active.

**C) setInterval management useEffect** — observes `calcState`:
- Computes `needsTick = calcState.clock_active || calcState.stopwatch_keyboard_mode`
- If `needsTick && liveTickRef.current === null`: starts 100ms interval
- Interval callback: checks `busyRef.current` (skips if dispatch_op in flight), then calls `invoke('tick_time')`
- Callback does NOT set `busyRef.current = true` — tick_time is read-only
- If `!needsTick && liveTickRef.current !== null`: clears interval
- Returns cleanup function (prevents dangling interval in React StrictMode)

**D) Alarm event parsing** — extended event_buffer useEffect with prefix matching:
- `alarm:message:{text}` → `showToast(text)` (prefix stripped, only alarm text shown)
- `alarm:xeq:{label}` → `invoke('dispatch_op', { keyId: 'xeq_' + label })` (control alarm XEQ)
- Other events → `showToast(line)` (existing BEEP/TONE behavior preserved)

**E) App.test.tsx extensions**:
- Updated `CalcStateView` interface with `clock_active` and `stopwatch_keyboard_mode` fields
- Updated `makeEmptyView()` with `clock_active: false, stopwatch_keyboard_mode: false` defaults
- **D4 test**: `event_buffer=['alarm:message:ALARM!']` → toast shows `'ALARM!'`, does NOT contain `'alarm:message:'`
- **D5 test**: `event_buffer=['alarm:xeq:TESTLBL']` → `dispatch_op` called with `{ keyId: 'xeq_TESTLBL' }`

### Task 2: lcd_alternation_modal_prompt_time.rs

Created integration test file mirroring `lcd_alternation_modal_prompt_stat1.rs` (Phase 36 template):

- **`setime_prompt_renders_verbatim`**: `ModalProgram::Time(TimeStep::SetTimePrompt)` + `modal_prompt = "TIME?"` → `display_str == "TIME?"` (5 chars, no truncation)
- **`setdate_prompt_renders_verbatim`**: `ModalProgram::Time(TimeStep::SetDatePrompt)` + `modal_prompt = "DATE?"` → `display_str == "DATE?"` (5 chars, no truncation)
- **`xyzalm_time_prompt_renders_verbatim`**: `ModalProgram::Time(TimeStep::XyzalmTimePrompt)` + `modal_prompt = "ALARM TIME?"` → `display_str == "ALARM TIME?"` (11 chars ≤ LCD_WIDTH=12, no truncation)

All tests verify the LCD-alternation branch fires when `modal_program.is_some() && entry_buf.is_empty() && modal_prompt.is_some()`.

## Verification Results

| Check | Result |
|-------|--------|
| `npx tsc --noEmit` (hp41-gui) | Exit 0 |
| `npx vitest run src/App.test.tsx` | 25/25 passed (including D4 + D5) |
| `cargo test --test lcd_alternation_modal_prompt_time` | 3/3 passed |
| `grep "tick_time" App.tsx` | 3 matches |
| `grep "alarm:message:" App.tsx` | 4 matches |
| `grep "alarm:xeq:" App.tsx` | 3 matches |
| `grep "liveTickRef" App.tsx` | 9 matches (declaration + useEffect) |
| `grep "clearInterval" App.tsx` | 2 matches |
| `grep "alarm:message" App.test.tsx` | 4 matches (D4 test) |
| `grep "alarm:xeq" App.test.tsx` | 5 matches (D5 test) |

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — all features are fully implemented and wired.

## Threat Flags

No new threat surface introduced beyond what the plan's threat model covers:
- T-41-05 (busyRef guard) — implemented per plan
- T-41-06 (alarm:xeq label trust) — accepts level per plan (user-created alarm entries)

## Self-Check: PASSED

| Check | Result |
|-------|--------|
| `lcd_alternation_modal_prompt_time.rs` exists | FOUND |
| `App.tsx` exists | FOUND |
| `App.test.tsx` exists | FOUND |
| Commit `1e48f7c` exists | FOUND |
| Commit `92ed512` exists | FOUND |
| `clock_active: boolean` in CalcStateView interface | FOUND |
| `liveTickRef` in App.tsx | FOUND |
| `tick_time` invoke call | FOUND |
| `alarm:message:` routing | FOUND |
| `alarm:xeq:` routing | FOUND |
| `setime_prompt_renders_verbatim` test | FOUND |
