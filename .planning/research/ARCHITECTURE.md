# Engine Re-Entrancy Architecture — Interrupting Control Alarms (v4.3)

**Domain:** Brownfield Rust emulator — hp41-core synchronous engine
**Researched:** 2026-06-06
**Overall confidence:** HIGH (based on direct source reading)

---

## Summary

The current engine is a synchronous, single-threaded, clone-and-loop interpreter.
`run_program` clones `state.program`, positions `state.pc` at the entry label's next
step, clears `state.call_stack`, and enters `run_loop`. The run loop executes one
instruction per iteration and breaks on `Op::Stop`, `Op::Prompt`, call-stack exhaustion,
or program end. There is no preemption check of any kind.

Interrupting control alarms (`>>label`) are stored faithfully in `CalcState::alarms` as
`AlarmType::Control { label, interrupting: true }` but are currently silently demoted to
non-interrupting message events (D-40-04 divergence). The data model is already
forward-compatible — only execution logic and one new state field are needed.

The proposed design adds a single `pending_interrupt: Option<String>` field to
`CalcState`. `check_alarms` sets it instead of emitting `"alarm:interrupting:deferred"`.
The run loop checks this field at the top of each iteration (the same place it already
checks `steps >= MAX_STEPS` and `pc >= program.len()`). When set, the run loop treats
it as a synthetic XEQ: push `state.pc` onto the call stack and jump to the interrupt
label. Because it reuses the 4-level `call_stack` directly, the resume is automatic:
`Op::Rtn` / program end in the alarm handler pops the saved pc and execution continues
exactly where it was halted. No new Op variants, no threads, no per-instruction overhead
outside the existing loop top.

---

## Current Execution Model (with file:line citations)

### Entry points

| Function | File | Lines | Role |
|----------|------|-------|------|
| `run_program` | `hp41-core/src/ops/program.rs` | 422-452 | Top-level entry: finds label, clears call_stack, sets is_running |
| `resume_program` | `hp41-core/src/ops/program.rs` | 454-477 | Resumes from current pc after Op::Stop break |
| `run_loop` | `hp41-core/src/ops/program.rs` | 485-714 | Inner interpreter loop |
| `execute_op` | `hp41-core/src/ops/program.rs` | ~720-1250 | Dispatches all non-control ops inside run_loop |

### run_program (lines 422-452)

```
state.program.clone() -> program slice  // borrow-conflict guard, D-06
find label in program                   // linear scan
state.pc = label_index + 1             // skip past LBL marker
state.call_stack.clear()               // always starts fresh
state.is_running = true
run_loop(state, &program)
state.is_running = false               // always reset, even on Err (Pitfall 2)
```

`run_program` is called by the CLI's `drain_event_buffer` (app.rs ~1785-1815) for
non-interrupting `alarm:xeq:LABEL` events, and by the initial R/S keystroke to start a
labeled program.

### run_loop structure (lines 485-714)

Each iteration:

1. Guard: `steps >= MAX_STEPS` -> `Err(Overflow)` (line 488)
2. Guard: `pc >= program.len()` -> `break` (implicit top-level RTN, line 492)
3. `op = program[pc].clone(); pc += 1` (lines 496-497)
4. Match on op variant:
   - `Op::Rtn` -> `call_stack.pop()` -> set `pc = return_pc`, or `break` if stack empty (lines 500-504)
   - `Op::Stop` -> `break` (line 657)
   - `Op::Prompt` -> write `display_override`, `break` (lines 660-663)
   - `Op::Xeq(label)` -> guard `call_stack.len() >= 4` -> `Err(CallDepth)`, else push pc, set pc = target + 1 (lines 538-573)
   - All other ops -> `execute_op(state, op)?`

There is no cancel/stop check inside the loop other than the `MAX_STEPS` guard and
`Op::Stop`. The `cancel_requested` `Arc<AtomicBool>` is checked only inside the INTG /
SOLVE / DIFEQ solver loops (D-28.8), not by the main run_loop.

### 4-level call stack

`state.call_stack: Vec<usize>` (state.rs line 79). Maximum 4 entries enforced at every
`Op::Xeq` / `Op::XeqInd` arm (lines 539, 525). Stores the return pc (the step
immediately after the XEQ instruction that was consumed). `Op::Rtn` pops and restores it
(lines 501-503). A top-level `Op::Rtn` with an empty stack breaks the loop normally
(line 503).

### check_alarms - current behavior (alarm.rs lines 448-466)

Called by frontends after every `dispatch()` and on every `tick_time` tick (~100ms).
Scans all alarms; for any `!past_due && trigger_unix <= now`:
- Sets `alarm.past_due = true`
- Calls `dispatch_alarm_event` (lines 493-511):
  - Message -> `event_buffer.push("alarm:message:{text}")`
  - Non-interrupting control -> `event_buffer.push("alarm:xeq:{label}")`
  - Interrupting control -> `event_buffer.push("alarm:interrupting:deferred")` (currently a dead-end)

### event_buffer drain - current behavior

**CLI** (app.rs ~1778-1798, `drain_event_buffer`):
- Drains `state.event_buffer` into a local `Vec<String>`
- Prefix `"alarm:xeq:{label}"` -> calls `run_program(state, label)`
- Prefix `"alarm:interrupting:..."` -> silently ignored (D-38.4, D-40-04)
- Called after every `call_dispatch_and_drain` (line 1929) and after every `run_program` return

**GUI** (commands.rs ~302-303, ~346-347):
- `dispatch_op` drains `event_buffer` into `event_lines` returned in `CalcStateView`
- `tick_time` also drains both buffers (lines 346-347)
- Frontend receives `event_lines` and acts on them; non-interrupting `alarm:xeq:LABEL`
  entries trigger a `dispatch_op("xeq_LABEL")` call from the frontend

### Where the run loop checks stop/cancel

There is no explicit cancel check inside `run_loop`. The only stop signals are:
- `Op::Stop` variant in program (line 657)
- `Op::Prompt` (line 660)
- `call_stack.len() >= 4` overflow -> `Err(CallDepth)` (line 539)
- `steps >= MAX_STEPS` (line 488)
- `pc >= program.len()` (line 492)

---

## Proposed Design: Pending-Interrupt at Run-Loop Boundary

### Core principle

The design adds one check at the top of `run_loop`'s iteration, between the existing
`MAX_STEPS` guard and the `pc >= program.len()` check. This is not per-instruction
overhead - the existing guards are already checked every iteration; adding one more `if`
is O(1) constant-cost.

When a pending interrupt exists, the loop treats it as a synthetic XEQ frame push,
exactly as if the running program had executed `XEQ "alarm_label"` at that instruction
boundary. The existing `Op::Rtn` / program-end machinery then resumes the interrupted
program automatically - no new resume mechanism is needed.

### New CalcState field

Add to `hp41-core/src/state.rs` after the existing transient fields (after line 442):

```rust
/// Pending interrupting control alarm label.
///
/// Set by `check_alarms` when an interrupting control alarm fires and
/// `state.is_running == true`. The run loop checks this at the top of each
/// iteration and inserts a synthetic call frame (push pc, jump to label).
///
/// When is_running == false, an interrupting alarm queues into event_buffer
/// as "alarm:xeq:{label}" for the frontend to execute at the next interaction
/// boundary (same path as non-interrupting alarms).
///
/// Transient - never persisted.
#[serde(default, skip)]
pub pending_interrupt: Option<String>,
```

### check_alarms change (alarm.rs ~493-511)

Replace the `interrupting: true` branch in `dispatch_alarm_event`:

```rust
// BEFORE:
if *interrupting {
    state.event_buffer.push("alarm:interrupting:deferred".to_string());
}

// AFTER:
if *interrupting {
    if state.is_running {
        // Set pending interrupt - run_loop will pick it up at the next
        // instruction boundary. Only the first interrupt is queued;
        // subsequent interrupts while one is already pending are demoted
        // to the non-interrupting event_buffer path (nesting guard - real
        // HP-41CX also cannot re-interrupt at the same stack level).
        if state.pending_interrupt.is_none() {
            state.pending_interrupt = Some(label.clone());
        } else {
            state.event_buffer.push(format!("alarm:xeq:{label}"));
        }
    } else {
        // Not running: queue for frontend to execute after current dispatch.
        state.event_buffer.push(format!("alarm:xeq:{label}"));
    }
}
```

### run_loop change (program.rs ~485-495)

Add the interrupt check between `steps += 1` and `pc >= program.len()`:

```rust
fn run_loop(state: &mut CalcState, program: &[Op]) -> Result<(), HpError> {
    let mut steps: u64 = 0;
    loop {
        if steps >= MAX_STEPS {
            return Err(HpError::Overflow);
        }
        steps += 1;

        // Pending-interrupt check: fires between instructions, never mid-instruction.
        // Reuses the existing call-stack push/pc-redirect machinery directly.
        if let Some(label) = state.pending_interrupt.take() {
            if state.call_stack.len() >= 4 {
                // 4-level cap exceeded - cannot interrupt. Silent drop (hardware-faithful).
                // Real HP-41CX also silently drops an interrupt on a full call stack.
            } else {
                match find_in_program(program, &label) {
                    Ok(target) => {
                        state.call_stack.push(state.pc); // save resume address
                        state.pc = target + 1;           // jump to alarm label
                    }
                    Err(_) => {
                        // Label not found - surface as an event for frontend.
                        state.event_buffer.push(format!("alarm:missing:{label}"));
                    }
                }
            }
        }

        if state.pc >= program.len() {
            break; // implicit top-level RTN
        }
        let op = program[state.pc].clone();
        state.pc += 1;
        // ... rest of match unchanged
    }
    Ok(())
}
```

### Why this works with existing RTN/END machinery

At the top of each iteration, `state.pc` points at the next instruction to execute (the
one that would have run this iteration). This is the correct resume address because the
previous instruction was already fully executed in the prior iteration.

When the interrupt fires:
1. We push `state.pc` (= resume address) onto `call_stack` and set `pc = alarm_label + 1`.
2. The alarm program runs normally inside the same `run_loop` invocation.
3. When the alarm program hits `Op::Rtn` or runs off its end, the existing `call_stack.pop()`
   restores `state.pc = resume_address`.
4. Execution continues at exactly the next step of the interrupted program.

The interrupted program's full register state (stack X/Y/Z/T, ALPHA, flags, regs) is
preserved throughout - `run_loop` carries no per-frame local state. Everything lives in
`CalcState`, which is passed `&mut` through the same loop.

---

## State Changes

### New field

| Field | Type | Serde | Persistence | Default | Notes |
|-------|------|-------|-------------|---------|-------|
| `pending_interrupt` | `Option<String>` | `#[serde(default, skip)]` | Transient | `None` | Set by `check_alarms`; consumed by `run_loop`; cleared at `run_program` entry |

### Migration

No migration required. `#[serde(default, skip)]` means:
- All existing save files (v1.0-v4.2) deserialize without the field -> `None` (correct).
- The field never appears in JSON output.
- `migrate_after_load()` requires no change.

### Initialization

Add `pending_interrupt: None` to `CalcState::new()` (state.rs ~493) alongside the
existing transient field initializations.

### Clearing on run_program / resume_program entry

`run_program` calls `state.call_stack.clear()` at line 445. Add
`state.pending_interrupt = None;` immediately after this - a stale interrupt must not
fire into a fresh program execution.

`resume_program` (line 472) should also clear `pending_interrupt` before entering
`run_loop`, for the same reason. The edge case: an interrupt fires just before `Op::Stop`
breaks the loop; the stale `pending_interrupt` must not redirect the resume.

---

## New Op Variants?

None required. The interrupt is a synthetic control-flow event handled entirely inside
`run_loop` as a special case, parallel to the existing `Op::Stop` and `Op::Prompt` breaks.
No new Op variant means:
- No 4-way exhaustive match update required (dispatch, execute_op, CLI op_display_name,
  GUI op_display_name).
- No JSON function pool entry needed.
- No `builtin_card_op` registration.

This is the correct design choice: interrupts are a runtime mechanism, not a user-authored
instruction.

---

## Integration Points (core + CLI + GUI), with file:line

### hp41-core/src/state.rs

- **After line 442** (after `xmem_active_file` declaration): add new field declaration
  with `#[serde(default, skip)]`
- **~line 535** (in `CalcState::new()`): initialize `pending_interrupt: None,`
- **No change** to `migrate_after_load()`

### hp41-core/src/ops/program.rs

- **run_program, line ~445**: add `state.pending_interrupt = None;` after `state.call_stack.clear();`
- **resume_program, line ~472**: add `state.pending_interrupt = None;` before `run_loop(state, &program);`
- **run_loop, lines ~487-495**: insert the pending-interrupt check block between `steps += 1`
  and `pc >= program.len()` check
- **`find_in_program` (line 1320)**: already `pub(crate)` within the crate - the new
  check reuses it directly without changes

### hp41-core/src/ops/time/alarm.rs

- **`dispatch_alarm_event`, lines ~499-510**: replace the `interrupting: true` branch as
  shown above - routing depends on `state.is_running`
- **`check_alarms`, lines 448-466**: no structural change - it calls `dispatch_alarm_event`,
  which now handles the routing

### hp41-cli/src/app.rs

- **`drain_event_buffer` (~1778-1798)**: the `"alarm:interrupting:..."` silent-ignore
  branch becomes a dead code path for the `is_running == true` case (those now go to
  `pending_interrupt`). Update to handle `"alarm:missing:{label}"` events (surface as a
  CLI error message). The idle-interrupting case now emits `"alarm:xeq:{label}"` which
  the existing handler already covers - no new branch needed.
- **The R/S path calling `resume_program`** is unchanged.

### hp41-gui/src-tauri/src/commands.rs

- **`handle_tick_time` (~342-348)**: `check_alarms` is called here. The routing change is
  transparent - `tick_time` still just calls `check_alarms` and drains both buffers.
  When running, `check_alarms` sets `pending_interrupt` instead of `event_buffer`; the
  run_loop observes it at the next instruction boundary. No structural change to this
  function.
- **`handle_run_stop` (~476-479)**: unchanged.
- **`request_cancel` (~397-399)**: unchanged - still for INTG/SOLVE/DIFEQ only.
- **`dispatch_op`**: unchanged - `run_program` / `resume_program` now clear
  `pending_interrupt` at entry; `run_loop` consumes it internally.

---

## Idle vs Running, Nesting, 4-Level Overflow

### Idle (not running)

When `state.is_running == false`, `check_alarms` emits `"alarm:xeq:{label}"` to
`event_buffer`. The existing frontend drain machinery handles it as a non-interrupting
alarm - the CLI's `drain_event_buffer` calls `run_program(state, label)` and the GUI
frontend calls `dispatch_op("xeq_LABEL")`. This is the correct hardware-faithful behavior:
on a real HP-41CX, an interrupting alarm with no running program executes immediately as
a direct XEQ. No additional code is needed for this path.

### Nesting (alarm program fires while alarm is already being serviced)

`pending_interrupt` is an `Option<String>`. If a second interrupting alarm fires while the
first alarm's program is executing, `pending_interrupt` may already be `None` (the first
one was consumed by `take()`). The second alarm's call to `dispatch_alarm_event` sees
`state.is_running == true` and `state.pending_interrupt.is_none()`, so it sets
`pending_interrupt` normally. This means a second interrupt can queue while the first is
executing - which is hardware-faithful (the real HP-41CX re-arms after each handler RTN).

The only nesting guard needed is: if `pending_interrupt` is already `Some(_)` when a
second alarm fires (pathological case: two alarms fire between the same two instructions),
demote the second to `event_buffer`.

True re-entrance is impossible: the alarm handler runs inside `run_loop` at the same call
depth as the interrupted program. If the alarm handler itself contains `XEQ "SOME_PROG"`,
it uses a call stack slot - this is bounded by the 4-level limit, not by the interrupt
mechanism.

### 4-Level Overflow

When `state.call_stack.len() >= 4` at interrupt time, the interrupt is silently dropped
(the `take()` already consumed `pending_interrupt`). This is hardware-faithful: the real
HP-41CX cannot interrupt a 4-deep call chain. No error is returned; the running program
continues unaffected. The dropped alarm remains `past_due = true` in the alarm catalog -
the user can acknowledge it manually via `ALMNOW` or ALMCAT browsing.

Optionally, push `"alarm:interrupt:overflow:{label}"` to `event_buffer` so the CLI/GUI
can surface a notification. This is a polish item, not a correctness issue.

---

## Frontend Wiring

### CLI (app.rs)

The CLI's execution is synchronous: keystroke -> `call_dispatch_and_drain` ->
`drain_event_buffer`. An interrupting alarm reaches `pending_interrupt` in one moment:
before a dispatch that calls `run_program`, if `check_alarms` had been called and
the alarm was already past-due.

For true mid-execution interrupt detection in the CLI, `check_alarms` must be called
inside `run_loop` at periodic step boundaries (Phase C in the build order below). Without
Phase C, the interrupt fires only at program start - acceptable for short programs, but
not hardware-faithful for long-running programs that span multiple real seconds.

No IPC or UI change is needed in the CLI beyond updating the `drain_event_buffer`
comment to remove the "ignored" label for interrupting alarms.

### GUI (commands.rs)

The GUI holds the `AppState` Mutex for the entire duration of `run_program` / `run_loop`.
`tick_time` (which calls `check_alarms`) cannot acquire the Mutex while `run_loop` is
spinning. Therefore:

- Without Phase C, a pending interrupt can only be set before `run_loop` starts (if
  `tick_time` happened to fire between the user's keypress and `dispatch_op` locking the
  Mutex - a very small window).
- With Phase C (`check_alarms` called every N steps inside `run_loop`), the GUI achieves
  true mid-execution interrupt detection at N-instruction granularity.

The D-11 no-polling invariant is fully respected: the frontend never polls for interrupt
state. The `tick_time` 100ms interval already exists for clock/stopwatch display; it
naturally serves as the alarm detection cadence when a program is not running.

### No new Op variants = no GUI IPC change

Since no new Op variants are introduced, the GUI's `key_map::resolve()` and all
`dispatch_op` Tauri commands remain unchanged. The 4-way exhaustive match invariant
(CLAUDE.md) is not triggered.

---

## Suggested Build Order

### Phase A: Core state + check_alarms routing (hp41-core only)

1. Add `pending_interrupt: Option<String>` to `CalcState` with `#[serde(default, skip)]`
2. Initialize to `None` in `CalcState::new()` and clear in `run_program` / `resume_program` entry
3. Modify `dispatch_alarm_event` in `alarm.rs`: `interrupting: true` + `is_running` ->
   set `pending_interrupt`; `interrupting: true` + `!is_running` -> emit `"alarm:xeq:{label}"`
4. Add guard: if `modal_program.is_some()`, demote interrupt to `event_buffer`
5. Write unit tests:
   - `check_alarms_sets_pending_interrupt_when_running`
   - `check_alarms_emits_xeq_event_when_not_running`
   - `nesting_guard_demotes_second_interrupt_when_first_is_pending`

### Phase B: run_loop interrupt check (hp41-core)

6. Add the `pending_interrupt.take()` check block to `run_loop` between `steps += 1`
   and `pc >= program.len()` check
7. Write integration tests:
   - `interrupt_fires_at_instruction_boundary_and_resumes`
   - `interrupt_resumes_after_rtn`
   - `interrupt_4_level_overflow_is_silent_drop`
   - `interrupt_missing_label_demotes_to_event`
   - `interrupt_does_not_clobber_program_flow_around_it`
   - `interrupt_nesting_only_one_level_deep`

### Phase C: Periodic check_alarms inside run_loop (strongly recommended)

8. Call `check_alarms(state)` inside `run_loop` every 1000 steps at the same point as
   the pending-interrupt check:
   ```rust
   if steps % 1000 == 0 {
       hp41_core::ops::time::alarm::check_alarms(state);
   }
   ```
   This costs approximately one function call per 1000 instructions - negligible.
   It is the only way to achieve mid-execution interrupt detection without threads.

### Phase D: CLI drain_event_buffer update (hp41-cli)

9. Update the `"alarm:interrupting:..."` silent-ignore branch comment to clarify that
   this path now only fires for overflow / idle-interrupted events
10. Add handling for `"alarm:missing:{label}"` events (surface as CLI status message)

### Phase E: GUI observe and surface (hp41-gui)

11. Verify `handle_tick_time` behavior is correct with the new routing - should require
    no code change, only a test
12. Verify `dispatch_op` behavior: `run_program` / `resume_program` clear
    `pending_interrupt` at entry - no double-fire edge case

### Phase F: Acknowledgment and repeat logic

13. Verify `acknowledge_alarm` (alarm.rs line 474) correctly reschedules repeating
    interrupting alarms after execution. The alarm is already `past_due = true` when the
    handler runs. Acknowledgment must happen after the handler RTNs.
    Open design question: who calls `acknowledge_alarm`? Options: (a) run_loop after
    the synthetic XEQ returns; (b) the frontend after the full `run_program` call returns;
    (c) automatic in `check_alarms` for interrupt-type alarms. Option (a) is cleanest
    and most hardware-faithful.

### Phase G: Documentation

14. Update `docs/hp41-time-divergences.md` D-40-04: change from "deferred / not supported"
    to "implemented via pending-interrupt at run-loop boundary (v4.3)"
15. Add ADR `docs/adr/v4.3-001-interrupt-alarm-pending-field.md`
16. Update `hp41-core/src/ops/time/alarm.rs` module-level comment to reflect the new
    dispatch path

---

## Risks and Open Questions

### Risk 1: GUI mid-execution interrupt latency (MEDIUM)

The GUI holds the Mutex during `run_loop`. `check_alarms` from `tick_time` cannot run
while a program is executing. Without Phase C, interrupt detection granularity in the GUI
equals total program runtime. For short programs this is fine. For programs running
seconds or minutes, an alarm can fire much later than its scheduled time.

Phase C (periodic `check_alarms` inside `run_loop`) resolves this completely. The cost is
trivial. This risk is resolved by including Phase C in the initial implementation.

### Risk 2: Stack register clobber during alarm handler (LOW - by design)

The alarm handler program runs inside the same `run_loop` with the same `&mut CalcState`.
Any stack-lifting op in the alarm handler clobbers the running program's X/Y/Z/T
registers. This is hardware-faithful - the real HP-41CX has exactly this behavior.
Users must write alarm handler programs defensively (STO/RCL to preserve registers).
This is not a bug; it must be documented in the ADR.

### Risk 3: Interaction with modal workflows / solver re-entrancy (MEDIUM)

If an interrupting alarm fires while `Op::Difeq` / `Op::Integ` / `Op::Solve` is
executing inside `run_loop`, the synthetic XEQ could corrupt the solver's `integ_state`
/ `solve_state` / `difeq_state`. The Phase A guard (`modal_program.is_some()` -> demote)
partially addresses this, but the solver ops run inside `run_loop` where `modal_program`
may not be set during every sub-step.

Recommendation: add a second guard in `dispatch_alarm_event`: if `state.integ_state.is_some()
|| state.solve_state.is_some() || state.difeq_state.is_some()`, demote the interrupt to
`event_buffer`. This is safe and sufficient.

### Risk 4: Acknowledgment timing for repeating interrupting alarms (MEDIUM)

Currently, `acknowledge_alarm` is called manually by the frontend after the user
interacts with a non-interrupting alarm notification. For interrupting alarms that execute
automatically, there is no user interaction - the alarm must acknowledge itself after the
handler program completes.

Design decision required: the cleanest approach is for `run_loop` to call
`acknowledge_alarm(state, interrupt_alarm_index)` immediately after the synthetic XEQ
frame is pushed. This requires tracking which alarm index triggered the interrupt. An
alternative: `dispatch_alarm_event` marks the alarm for auto-acknowledgment, and
`run_loop` acknowledges it after the handler returns.

This is the largest design open question for Phase B and needs a concrete decision before
implementation.

### Open Question 1: What to do when call_stack is full at interrupt time?

Options:
- (a) Silent drop (hardware-faithful) - recommended for initial implementation
- (b) Queue to `event_buffer` for post-run execution
- (c) BEEP + display notification

Recommendation: (a) initially, (b) as a configurable follow-up.

### Open Question 2: Interrupt during Op::Stop / Op::Prompt break

When `Op::Stop` breaks `run_loop`, `state.is_running` is reset to `false`. A
`pending_interrupt` that was set just before the STOP remains in the field. On the next
R/S, `resume_program` re-enters `run_loop` - the pending interrupt will fire at the first
iteration, redirecting the resume to the alarm handler before the post-STOP instruction.

This may or may not be hardware-faithful (real HP-41CX behavior at this edge case is
undocumented). Clearing `pending_interrupt` in `resume_program` (as recommended above)
removes the ambiguity at the cost of dropping the interrupt. Recommendation: clear on
resume for now; document as a known simplification.

### Open Question 3: Interrupt alarm index tracking for acknowledgment

`dispatch_alarm_event` does not currently return the alarm index - it only receives a
cloned `AlarmEntry`. For auto-acknowledgment, `run_loop` needs to know which alarm to
acknowledge after the handler completes. Options:
- Add a `pending_interrupt_alarm_index: Option<usize>` field alongside `pending_interrupt`
- Scan `state.alarms` after the handler completes to find the matching `past_due` entry
- Re-architect `dispatch_alarm_event` to return the index

Option 2 (scan) is simplest and avoids a second new field, but is O(n) on alarm count.
With the 253-alarm cap, this is at most 253 iterations - negligible.

---

## Sources

- `hp41-core/src/ops/program.rs` - `run_program` (422-452), `resume_program` (454-477), `run_loop` (485-714)
- `hp41-core/src/state.rs` - `CalcState` struct (54-443), `migrate_after_load` (570-602), `CalcState::new()` (465-537)
- `hp41-core/src/ops/time/alarm.rs` - `check_alarms` (448-466), `dispatch_alarm_event` (493-511), `AlarmType` (56-60), `acknowledge_alarm` (474-486)
- `hp41-cli/src/app.rs` - `drain_event_buffer` (~1778-1798), `call_dispatch_and_drain` (~1936-1957)
- `hp41-gui/src-tauri/src/commands.rs` - `handle_tick_time` (342-348), `handle_run_stop` (476-479), `run_stop` (374-377), `request_cancel` (397-399)
- `docs/hp41-time-divergences.md` - D-40-04 section (265-309): the two rejected approaches and forward-compat rationale
- CLAUDE.md frozen invariants: no async/no panics, 4-way exhaustive match, save-compat, math1 freeze, D-11 no-polling
