# Risks, Guardrails & Verification — v4.3 Hardware Fidelity

**Domain:** Brownfield HP-41 emulator engine work (hp41-core)
**Researched:** 2026-06-06
**Milestone anchor:** Interrupting control-alarm execution (D-40-04) + audit-selected fidelity gaps

---

## Summary

v4.3 is the first substantive `hp41-core` engine work since the v3.x module era (v3.0
shipped 2026-05-20). The central feature — interrupting control-alarm re-entrancy — touches
`run_loop` / `run_program` in `program.rs` and `check_alarms` / `dispatch_alarm_event` in
`time/alarm.rs`, both of which are outside the math1/ freeze. The risk profile is
high because: (a) the program engine is the most widely exercised path across 3300+ tests;
(b) the 4-level call stack cap and `is_running` invariant are load-bearing; (c) the test
oracle for time-dependent behavior requires deliberate determinism engineering. This
document is the primary guardrail reference for every phase of v4.3.

---

## Frozen-Invariant Guardrails

Each invariant is restated as a concrete, checkable constraint specific to this milestone.

### I-01: math1/ Directory Freeze

**Invariant (CLAUDE.md "Core engine"):** All files inside `hp41-core/src/ops/math1/`
are frozen since Plan 25-01. Three sanctioned carve-outs: `xrom.rs`, `modal.rs`,
`complex.rs`.

**This milestone's constraint:** The alarm interrupt execution path runs entirely through
`time/alarm.rs` (`check_alarms`, `dispatch_alarm_event`) and `ops/program.rs`
(`run_loop`, `run_program`). Neither is inside `math1/`. Any re-entrancy scaffolding
(saved-PC stack, interrupt-pending flag, resume hook) lives in `state.rs` or
`program.rs`. Zero changes to any math1/ file except the three sanctioned carve-outs.

**Checkable gate:** `git diff HEAD -- 'hp41-core/src/ops/math1/'` after each commit must
show no changes other than additions to `xrom.rs`, `modal.rs`, or `complex.rs` (and those
only if a new XROM variant is wired, which is unlikely for this milestone).

---

### I-02: Stack-Lift LiftEffect Discipline

**Invariant (CLAUDE.md "Core engine"):** Every op declares `LiftEffect::Enable /
Disable / Neutral`. This is the most commonly mis-implemented HP-41 feature.

**This milestone's constraint:** Any new `CalcState` field or engine state saved/restored
during alarm interruption must not interfere with `lift_enabled` in the stack. Specifically:

- The interrupted program's `lift_enabled` state is part of `state.stack.lift_enabled`.
- If the alarm interrupt saves and restores `CalcState` fields, `lift_enabled` must be
  faithfully preserved. Do NOT clear it when setting up the interrupt frame.
- The alarm label program executes with its own lift semantics as a sub-call. When
  the original program resumes, `lift_enabled` must be exactly what it was at the
  interruption point.
- Any new op variants introduced for this milestone must declare an explicit
  `LiftEffect` — no defaulting.

**Checkable gate:** Test scenario `interrupt_preserves_stack_x_y_z_t_and_lift_state`
(see Verification Strategy section) asserts `state.stack.lift_enabled` before, during,
and after alarm interrupt.

---

### I-03: Save-File Backward Compatibility

**Invariant (CLAUDE.md "Core engine"):** Every new `CalcState` field carries
`#[serde(default)]`. Transient fields also carry `#[serde(skip)]`. v1.0–v3.3 save files
must load without migration. Two documented exceptions that carry `#[serde(default)]`
WITHOUT `#[serde(skip)]`: `rand_seed` and `adv_tvm_state`.

**This milestone's constraint:** The interrupting alarm implementation will require new
`CalcState` fields. Every such field MUST:

1. Carry `#[serde(default)]` so v1.0–v4.2 autosave files deserialize cleanly.
2. Transient fields (e.g., `interrupt_pending: bool`, `saved_interrupt_pc: Option<usize>`)
   MUST ALSO carry `#[serde(skip)]` so they never pollute saved state.
3. Persistent fields that must survive save/load (e.g., a queued alarm-label for
   deferred execution) follow the `rand_seed` pattern: `#[serde(default)]` without
   `#[serde(skip)]`, with explicit documentation in `state.rs` of why.
4. `migrate_after_load()` needs a review pass to confirm no v4.3 fields require active
   migration (vs. default initialization).

**Checkable gate:** After implementation, run existing backward-compat test fixtures and
add a new `v4_3_interrupt_backward_compat` test that deserializes a v4.2-era
`autosave.json` and confirms alarm + interrupt fields default cleanly.

---

### I-04: No Async, No Panics (`#![deny(clippy::unwrap_used)]`)

**Invariant (CLAUDE.md "Core engine"):** No async anywhere in `hp41-core`. Production
code uses `.expect("reason")` or `?`. Tests use `#[allow(clippy::unwrap_used)]`. GUI
mutex uses `.unwrap_or_else(|e| e.into_inner())`.

**This milestone's constraint:** The interrupt execution path is synchronous. The earlier
design alternatives (parallel thread, `Arc<Mutex<CalcState>>`) were explicitly rejected
(D-38.4, D-40-04 in `docs/hp41-time-divergences.md`). The implementation must be a
synchronous call within `run_loop` or `check_alarms`, not a spawned thread. Any new
`Result`-returning helper in the alarm or program path must propagate errors with `?`,
not `.unwrap()`. Truly unreachable conditions use `.expect("invariant: ...")` naming
the specific invariant.

**Checkable gate:** `just lint` (runs clippy with `-D warnings`) must stay green.
Zero new `unwrap()` calls in non-test code confirmed by:
```bash
grep -rn 'unwrap()' hp41-core/src/ | grep -v '#\[allow'
```

---

### I-05: 4-Way Exhaustive-Match Invariant for Any New Op

**Invariant (CLAUDE.md "Core engine"):** Every new `Op` variant must land in ALL FOUR
before any caller compiles: (1) `dispatch()` in `ops/mod.rs`, (2) `execute_op()` in
`ops/program.rs`, (3) `op_display_name()` in `hp41-cli/src/prgm_display.rs`, (4)
`op_display_name()` in `hp41-gui/src-tauri/src/prgm_display.rs`.

**This milestone's constraint:** It is not yet clear whether interrupting control alarm
execution requires new `Op` variants. If the implementation is purely engine-level
(alarm fires, `run_program` called recursively or via saved PC, no new user-visible
ops), this invariant is untouched. If a new op is needed, all four sites must be updated
in the same commit, and `cargo build --workspace` must compile clean with no
`non_exhaustive_patterns` warnings before any phase is marked complete.

**Checkable gate:** The exhaustive-match compiler errors are the gate — they cannot be
silenced without satisfying all four sites. `cargo build --workspace` is sufficient.

---

### I-06: Zero New Runtime Dependencies Since v3.0

**Invariant (CLAUDE.md "Tech Stack"):** `statrs`, `libc`, `chrono` all rejected. Zero new
runtime deps across v3.0–v3.3. Algorithms hand-coded from primary literature.

**This milestone's constraint:** The interrupt re-entrancy implementation uses only existing
primitives: `Vec<usize>` for the call stack, `state.pc: usize`, `run_loop`, `run_program`.
No new crates in `hp41-core/Cargo.toml`. If a bounded data structure is needed (e.g., a
fixed-size interrupt-pending queue), implement it as a `Vec` with a length cap.

**Checkable gate:**
```bash
git diff HEAD -- hp41-core/Cargo.toml
# Must show zero [dependencies] additions
cargo tree -p hp41-core
# Output must be identical to pre-v4.3 baseline
```

---

### I-07: Print Emulation Rules (No `println!` in `hp41-core`)

**Invariant (CLAUDE.md "Core engine"):** `println!`/`eprintln!` forbidden in `hp41-core`.
Output routes through `state.print_buffer`. ALL `run_program()` call sites must wire
`call_dispatch_and_drain()` / `drain_and_show_print_output()`.

**This milestone's constraint:** Any alarm label program execution that produces output
(via PRX, PRA, PRSTK) must route exclusively through `state.print_buffer`. The CLI
`drain_event_buffer` in `app.rs` (line 1783) currently calls `run_program` for
`alarm:xeq:` events; this call site must include `drain_and_show_print_output()` after
the alarm program runs. The same discipline applies to the new interrupting alarm
execution path — the alarm label's print output must be visible to the frontend.

**Checkable gate:**
```bash
grep -rn 'println!\|eprintln!' hp41-core/src/
# Must return 0 results
```
Also enforced by `just lint` via clippy `print_stdout` / `print_stderr` lints.

---

### I-08: CLI-GUI Parity (D-25.6)

**Invariant (CLAUDE.md "CLI-GUI parity"):** One-shot SHIFT, ALPHA override, IND-toggle
are frontend-only. `CalcStateView` is the sole IPC type between `hp41-gui` and
`hp41-core`. `CalcState`/`CalcStateView`/IPC must never carry SHIFT/ALPHA UI state.

**This milestone's constraint:** The interrupt execution path lives in `hp41-core` and is
frontend-agnostic. Both frontends must handle the new `"alarm:interrupting:{label}"`
event (replacing the current `"alarm:interrupting:deferred"` stub) identically:

- CLI (`app.rs::drain_event_buffer` line 1797): currently silently ignores
  `"alarm:interrupting:..."`. After v4.3, it must call the real execution path.
- GUI (`commands.rs`): `tick_time` drains `event_buffer`; the GUI must act on the new
  event in the same way as CLI.

Both sites must be updated in the same phase. A parity gap violates D-25.6.

**Checkable gate:** Any test for interrupt execution targets `CalcState` directly
(engine test). Parity is validated by the E2E smoke verifying the alarm fires in
the GUI (see CI Coverage Gaps section).

---

## Do-Not-Touch Divergences

These are intentionally accepted divergences that must NOT be "fixed" in v4.3.

### DNT-01: D-40-01 — SW Interactive Stopwatch Keyboard Mode (Emulator Extension)

**Source:** `docs/hp41-time-divergences.md` (D-40-01)

`XEQ "SW"` sets `state.stopwatch_keyboard_mode = true` and the frontend enters an
interactive loop. The HP-41CX OM does not specify `SW` as a callable XEQ entry point.
Do not remove `stopwatch_keyboard_mode` or make `SW` a no-op. This is a documented
emulator extension.

---

### DNT-02: D-40-02 — CORRECT / SETAF Accuracy Factor No-Op

**Source:** `docs/hp41-time-divergences.md` (D-40-02)

`CORRECT` is a no-op. The crystal accuracy-factor correction has no physical basis on a
host-clock system. `SETAF`/`RCLAF` store/recall `state.accuracy_factor` but the factor
does not affect `time_offset_secs`. Do not apply `accuracy_factor` as a drift correction.
The rejected-alternative analysis (NTP, elapsed time) is documented and settled.

---

### DNT-03: D-40-03 — Host System Clock Backing (`SystemTime::now()`)

**Source:** `docs/hp41-time-divergences.md` (D-40-03)

The emulator uses `SystemTime::now()` + `time_offset_secs: i64` as a delta. Do not
introduce `chrono`, `libc::localtime_r`, or a trait-injection clock abstraction. The
`SystemTime` + offset design is final (D-38.1, D-38.2, D-38.3).

---

### DNT-04: D-40-05 — Centisecond Stopwatch Resolution via `Instant::elapsed()`

**Source:** `docs/hp41-time-divergences.md` (D-40-05)

Stopwatch resolution uses `Instant::elapsed().as_secs_f64()`. Do not replace `Instant`
with a tick counter or enforce exactly 100 ticks/second.

---

### DNT-05: D-40-04 Non-Interrupting Sub-Path Must Keep Working

**Source:** `docs/hp41-time-divergences.md` (D-40-04) + `hp41-cli/src/app.rs` line 1783

Non-interrupting control alarms (`>label`) fire as `"alarm:xeq:{label}"` events and are
handled by `drain_event_buffer`. This path is tested and correct. Do not change the
`>label` (non-interrupting) execution path when implementing `>>label` (interrupting)
execution. The only change to `dispatch_alarm_event` is in the `interrupting: true` arm,
which currently pushes `"alarm:interrupting:deferred"`.

---

### DNT-06: D-52-01 — X-MEM Overwrite-on-Duplicate

**Source:** `docs/hp41-xmem-divergences.md` (D-52-01)

`SAVEP`/`SAVED` silently overwrite an existing file. Hardware raises "DUP FL". This is
intentional because `PURFL`/`CLFL` are not implemented. Do not add "DUP FL" error
behavior in v4.3 unless `PURFL`/`CLFL` are also implemented in the same phase.

---

### DNT-07: D-52-02 — Full-Register-Set SAVED/GETD

**Source:** `docs/hp41-xmem-divergences.md` (D-52-02)

`SAVED`/`GETD` always transfer the full register set, ignoring the `bbb.eee` block
control word. Do not implement `bbb.eee` parsing in v4.3 — it is deferred and must
follow the ISG/DSE string-split discipline when eventually added.

---

### DNT-08: D-CV-02 — `CLRALPHA` Legacy Alias Must Stay in Op Enum

**Source:** `docs/hp41cv-divergences.md` (D-CV-02)

`Op::AlphaClear` (`display_name "CLRALPHA"`) is kept for v1.0 save-file deserialization.
Do not consolidate it with `Op::Cla`. Pitfall 8 in the existing research: removing it
breaks v1.0 save file backward compat.

---

### DNT-09: D-30-01 — Math Pac I Scratch Register Clobber Not Protected

**Source:** `docs/hp41-math1-divergences.md` (D-30-01)

The emulator does NOT snapshot/restore R00–R07 around INTG/SOLVE/DIFEQ callbacks. This
matches OM behavior (the OM warns the user but does not protect registers). A test
asserts the wrong-answer behavior. Do not add register snapshot/restore — it would
diverge from the OM and break an existing test.

---

### DNT-10: D-43.5 — Named-Matrix Isolation (Advantage Pac)

**Source:** CLAUDE.md "v3.x Design Rules"

Advantage Pac uses `adv_matrices` (`Vec<AdvMatrix>`). Math Pac I uses
`state.matrix_dim` / `state.matrix_active_reg`. These must never mix. Unrelated to
alarm work but live in the engine — any new code that reads `CalcState` must not
accidentally cross this boundary.

---

## Verification Strategy — Interrupting Control Alarms

All engine tests live in `hp41-core/tests/` in a new file:
`phase_vv_interrupting_alarms.rs` (phase number assigned at roadmap creation). The
file follows existing convention: `#[allow(clippy::unwrap_used)]` at file level,
uses `CalcState::default()` or a construction helper, imports from
`hp41_core::ops::time::alarm` and `hp41_core::ops::program`.

### Test Scenario 1: Basic Interrupt — Running Program Halted, Alarm Executes, Resumes

**Test name:** `interrupting_alarm_halts_running_program_and_resumes`

**Setup:**

1. Load a multi-step program under label `"MAIN"`:
   `LBL "MAIN"` → `1` → `STO 10` → `2` → `STO 11` → `RTN`
2. Load alarm label program under `"ALARM"`:
   `LBL "ALARM"` → `99` → `STO 12` → `RTN`
3. Register an interrupting control alarm with `trigger_unix = 0` (past-due, see
   Deterministic Clock section) and `alarm_type = AlarmType::Control { label: "ALARM",
   interrupting: true }`.

**Execution:** Call `run_program(&mut state, "MAIN")`. The engine must call
`check_alarms` at an instruction boundary during execution, detect the past-due
interrupting alarm, halt the current program, execute `"ALARM"`, and resume `"MAIN"`.

**Assertions (post-run):**

- `state.regs[10]` == 1 (STO 10 completed before interrupt)
- `state.regs[11]` == 2 (STO 11 completed after resume)
- `state.regs[12]` == 99 (alarm program executed)
- `state.call_stack` is empty (fully unwound after both programs complete)
- `state.is_running` == false
- Alarm `past_due` == true (or alarm removed if one-shot)

---

### Test Scenario 2: Stack and PC Preservation Through Interrupt

**Test name:** `interrupt_preserves_stack_x_y_z_t_and_lift_state`

**Setup:** A program sets specific values in X/Y/Z/T, then reaches a preemption point.
An interrupting alarm fires. The alarm program writes a different value to X.

**Assertions:**

- Verify HP OM 00041-90035 on stack behavior during alarm interrupt first (HIGH confidence
  required before coding). The HP-41CX call-stack mechanism saves return PCs, not register
  contents. If the OM specifies stack is shared during the interrupt, assert that the alarm
  program overwrites X and the original program resumes with the overwritten X (not a
  snapshot). Document the verified behavior in a `D-40-07` divergence entry if it differs
  from any reasonable expectation.
- `state.stack.lift_enabled` at resume matches its value at the interrupt point.
- `state.pc` resumes at the correct instruction after the interrupt (not at step 0).

---

### Test Scenario 3: 4-Level Call Stack Cap Respected

**Test name:** `interrupt_blocked_when_call_stack_at_4_level_cap`

**Setup:** Manually push 4 entries onto `state.call_stack` (simulating 4 nested XEQ
calls). Register an interrupting alarm with `trigger_unix = 0`. Trigger `check_alarms`.

**Assertions:**

- The interrupt does NOT execute the alarm label (call stack is full).
- The alarm event is handled gracefully (dropped, deferred to a queue, or emitted as
  a degraded event — verify HP hardware behavior for this case from OM 00041-90035).
- `state.call_stack` is unchanged (still 4 entries, no partial mutation).
- No panic, no `CallDepth` error propagated to the caller.
- `state.is_running` is unchanged.

---

### Test Scenario 4: Nested Interrupt Blocked (No Recursive Interrupt)

**Test name:** `interrupt_nesting_blocked_when_already_in_alarm_program`

**Setup:** While the alarm label program `"ALARM"` is running (interrupt in progress),
a second interrupting alarm fires.

**Assertions:**

- The second interrupt does NOT cause a recursive `run_program` call.
- The second alarm is either queued or dropped, per verified HP hardware behavior.
- Call stack depth never exceeds 4.
- No panic.

---

### Test Scenario 5: Idle-Fire Path (No Program Running)

**Test name:** `interrupting_alarm_fires_when_no_program_running`

**Setup:** `state.is_running == false`. An interrupting alarm is past due
(`trigger_unix = 0`).

**Assertions:**

- `check_alarms` executes the alarm label program immediately via `run_program`.
- No "resume original program" logic fires (there was no interrupted program).
- `state.is_running` == false after alarm program completes.
- `state.call_stack` is empty.

This is the simpler path. It should be implemented and tested first. On the HP-41,
interrupting alarms fire in the idle/keyboard-waiting state as well as during execution.

---

### Test Scenario 6: Non-Interrupting Path Regression

**Test name:** `non_interrupting_alarm_still_fires_as_event_not_inline`

**Setup:** Register a non-interrupting control alarm (`>LABEL`) with `trigger_unix = 0`.
Trigger `check_alarms`.

**Assertions:**

- `state.event_buffer` contains `"alarm:xeq:LABEL"`.
- The alarm program is NOT called inline.
- `state.is_running` is unchanged.
- The existing CLI `drain_event_buffer` path handles it via `run_program` after draining.

This is a regression guard for DNT-05.

---

### Test Scenario 7: Message Alarm Regression

**Test name:** `message_alarm_still_fires_to_event_buffer_not_executed`

**Setup:** Register a message alarm. Trigger `check_alarms`.

**Assertions:**

- `state.event_buffer` contains `"alarm:message:{text}"`.
- `state.print_buffer` contains the message text.
- No program execution occurs.

---

## Deterministic-Clock Testing (SystemTime / time_offset_secs)

The core clock design uses `SystemTime::now()` + `state.time_offset_secs: i64`. There is
no injectable clock trait (rejected in D-38.1). This creates a challenge for deterministic
alarm tests.

### Recommended Approach: Backdated Trigger Unix

Set `alarm.trigger_unix = 0` (Unix epoch: 1970-01-01 00:00:00 UTC) and leave
`state.time_offset_secs = 0`. Any real `SystemTime::now()` returns a value billions of
seconds past epoch, so `trigger_unix <= now` is trivially and permanently true.

```rust
fn make_past_due_interrupting_alarm(label: &str) -> AlarmEntry {
    AlarmEntry {
        trigger_unix: 0,  // Unix epoch — always in the past
        repeat_secs: 0,
        alarm_type: AlarmType::Control {
            label: label.to_string(),
            interrupting: true,
        },
        past_due: false,
    }
}
```

This pattern is already used implicitly in the existing alarm tests in
`hp41-core/src/ops/time/alarm.rs` (see the test section starting at line 536).

### Future-Dated (Non-Firing) Assertions

To assert an alarm does NOT fire yet, set `trigger_unix = i64::MAX`. The alarm will
never be past-due in any test run.

### Avoiding Wall-Clock Races

Do NOT call `SystemTime::now()` in test assertions. All assertions must be on
`CalcState` fields (registers, call stack, `event_buffer`, `print_buffer`, flags),
not on computed timestamps. The `trigger_unix = 0` pattern eliminates the race
entirely.

### time_offset_secs Caution

Leave `state.time_offset_secs = 0` in all alarm tests unless specifically testing the
SETIME/SETDATE offset logic. A large negative `time_offset_secs` could cause `now`
to equal 0 or go negative, making `trigger_unix = 0` NOT past-due. Always use
`time_offset_secs >= 0` in tests with backdated alarms.

---

## Oracle Discipline & Free42 Contamination Guard

### Primary Sources

All HP-41CX alarm/program-engine behavior must be derived from:

1. **HP 82182A Time Module Owner's Manual (HP 00041-90035, 1982)** — primary authority
   for alarm types, prefix conventions (`>`, `>>`), 4-level call stack limit in interrupt
   context, and idle-fire behavior.
2. **HP-41CX Owner's Manual (HP 00041-90028, 1983)** — primary authority for call stack
   (4-level cap), `run_program`/RTN semantics, and stack-register state through subroutine
   calls.

### Free42 as Behavioral Oracle Only

Free42 may be used ONLY to verify observable HP-41 behavior — never to copy algorithms
or data structures. The prohibition (CLAUDE.md "Free42 GPL-contamination guard") extends
to v4.3:

- Do NOT copy `src/core/core_main.cc`, `core_keydown`, or interrupt handling logic from
  Free42.
- Do NOT use Free42's call-stack implementation as a template.
- DO use Free42 as a behavioral oracle: run a scenario on Free42, observe the result,
  then independently implement the same behavior from the OM specification.

### Algorithm Provenance Protocol

For any behavioral ambiguity (e.g., "what happens to the return stack when an interrupt
fires at level 3?"):

1. Check HP 82182A OM for explicit specification.
2. If unspecified, check HP-41CX OM call-stack section.
3. If still ambiguous, test on real HP-41CX hardware or Free42 (behavioral observation
   only, no code), document the observed behavior in `docs/hp41-time-divergences.md`
   under a new `D-40-07` or later entry.
4. Implement from the documented specification, not from any source code.

### Contamination Guard Commands

The 18-symbol grep guard currently scans `hp41-core/src/ops/math1/`. For v4.3, if the
contamination guard script (`scripts/check-free42-contamination.sh`) needs to be extended
to cover new alarm/interrupt code paths, do so as a parallel update — the guard is
belt-and-suspenders alongside `just license-audit`.

```bash
# Run before and after any engine change:
just license-audit
```

The CI job `license-audit` in `ci.yml` runs in parallel with lint/test/coverage. It is
independent and will fail the PR check if a contamination token appears.

---

## Regression Surface & Quality Gates

### Test Suite Scale

At v4.3 start: ~3300+ tests in `hp41-core/tests/` + `hp41-cli/src/` + Vitest. The
program engine (`program.rs`) is exercised by every programming test across phases 1, 3,
22, 24, 28, 33, 38, 43, and 51. A regression in `run_loop` would manifest as widespread
failures, not isolated ones — detectable immediately on `just test`.

### Local Full Gate (run before any commit)

```bash
just ci
# Expands to: just lint && just test && just coverage && just license-audit
```

This is the minimal bar. Do not commit without this passing.

### Individual Gate Commands

| Gate | Command | Threshold |
|------|---------|-----------|
| Lint (stable clippy) | `just lint` | 0 warnings (-D warnings) |
| All tests | `just test` | 0 failures |
| Core tests only | `just test-core` | 0 failures |
| Coverage (lines) | `just coverage` | >= 95% lines on hp41-core |
| MSRV lint + test | `just ci-msrv` | 0 failures (no coverage gate) |
| Free42 contamination | `just license-audit` | 0 matches |
| search_aliases schema | `just schema-aliases-check` | all 6 pools valid |
| GUI build + vitest | `just gui-ci` | 0 failures |

### MSRV Clippy Divergence (Documented Pitfall)

From CLAUDE.md memory (`reference_msrv_clippy_lint_divergence`): `cargo +1.88 clippy`
flags style lints that stable demotes to pedantic (e.g., `uninlined_format_args`). This
caused hidden CI failures through v4.2 because develop branch protection was bypassed.

Guardrail: Before tagging v4.3, explicitly run:

```bash
cargo +1.88 clippy --workspace --all-targets --all-features -- -D warnings
```

Verify this matches `just lint` output. Any MSRV-only lint failure must be fixed. Check
`gh pr checks` before tagging — do not rely on visual PR approval alone.

### Numerical Accuracy Gate

```bash
just test-core --test numerical_accuracy
# Gate: >= 98% pass rate (843 cases at v4.3 baseline)
```

The v4.3 alarm/interrupt work does not touch numerical algorithms. This gate should
stay green without new test cases. A regression here would indicate `run_loop` changes
broke the dispatch path for mathematical ops.

### Zero-Panic Gate

```bash
grep -rn 'unwrap()' hp41-core/src/ | grep -v '#\[allow'
# Must return 0 results
```

Combined with `just lint` (clippy `unwrap_used` lint). Any panic in `hp41-core` is
critical.

### Coverage Impact of New Tests

The new test file adds coverage to:

- `hp41-core/src/ops/time/alarm.rs` — the `interrupting: true` arm in
  `dispatch_alarm_event` (currently only exercised by parse tests, not execution)
- `hp41-core/src/ops/program.rs` — new interrupt-handling code paths
- Potentially `hp41-core/src/state.rs` — new fields' default/serde paths

Since this work adds both new code and new tests, the 95% line / 93% region gate should
hold or improve — but measure after implementation with `just coverage`.

---

## CI Coverage Gaps to Watch (CLI vs GUI vs Mobile)

### ci.yml Coverage (Rust CLI + Core)

`ci.yml` runs on every push/PR to `main` and `develop` with NO path filter. It covers:
lint (stable clippy on entire workspace), test (3-OS matrix), coverage (hp41-core only,
Ubuntu/stable), msrv (1.88, Ubuntu, lint+test, no coverage), license-audit, schema-aliases.

**Gap:** The `coverage` job only measures `hp41-core`, not `hp41-cli`. New alarm-interrupt
handling code added to `hp41-cli/src/app.rs` (the `drain_event_buffer` update at line
1797) is NOT covered by the coverage gate. Targeted CLI tests in `hp41-cli/src/` or
manual testing are the backstop.

### ci-gui.yml Coverage (Tauri GUI)

`ci-gui.yml` has a `paths:` filter for `hp41-gui/**` and `hp41-core/**`. Changes to
`hp41-core/src/ops/time/alarm.rs` or `program.rs` WILL trigger `ci-gui.yml`. The GUI
CI runs `just gui-ci` (TS type-check, Rust build, Vitest, release build) on a 3-OS
matrix. The E2E smoke `e2e-linux` runs on Ubuntu only.

**Gap:** `ci-gui.yml` does NOT run `cargo clippy` on the GUI Rust side
(`hp41-gui/src-tauri/`). Any new `op_display_name` arm or IPC change in GUI Rust code
must be manually verified:

```bash
cd hp41-gui/src-tauri && cargo clippy --all-targets -- -D warnings
```

**Gap:** The E2E smoke tests do not directly cover alarm behavior. Adding a smoke test
that fires an alarm and verifies the GUI toast/event would close this gap — note it as
a v4.3 test-phase "should-have."

### iOS (`#[cfg(mobile)]`) Blind Spot

From CLAUDE.md memory (`project_cfg_mobile_gate_blindspot`): `just gui-ci` runs the
host target and cannot see `#[cfg(mobile)]` compile errors. If any v4.3 alarm/interrupt
code touches a path gated by `#[cfg(mobile)]` (unlikely for pure engine code, but
possible in Tauri commands), it will compile on desktop but fail on iOS.

Guardrail: After any change to `hp41-gui/src-tauri/src/commands.rs` that touches
alarm event handling:

```bash
cargo check --target aarch64-apple-ios --manifest-path hp41-gui/src-tauri/Cargo.toml
```

This is a manual step. `ci-ios.yml` covers it in CI but does not fire on
`hp41-core`-only pushes.

---

## Sources

- **CLAUDE.md** — Frozen Invariants section, Quality Gates table, IPC contract,
  CLI-GUI parity (D-25.6), math1/ freeze, serde discipline, print emulation rules,
  save-file backward compat, MSRV, zero-dep policy, 4-way exhaustive-match invariant,
  Free42 GPL-contamination guard (18-symbol grep)
- **`hp41-core/src/ops/time/alarm.rs`** — `AlarmEntry`, `AlarmType`, `check_alarms`
  (line 448), `dispatch_alarm_event` (line 493), `acknowledge_alarm` (line 474),
  `parse_alarm_type` (line 78); current `"alarm:interrupting:deferred"` stub at line 506
- **`hp41-core/src/ops/program.rs`** — `run_loop` (line 485), `run_program` (line 422),
  `resume_program` (line 468), call_stack cap at line 525, `MAX_STEPS` at line 483,
  Pitfall 2 (`is_running` reset) at lines 450/475
- **`hp41-core/src/state.rs`** — `CalcState` field serde annotations, `call_stack:
  Vec<usize>` at line 79, `is_running: bool` at line 82, `event_buffer: Vec<String>`
  at line 152 (`#[serde(default, skip)]`)
- **`docs/hp41-time-divergences.md`** — D-40-01 (SW keyboard mode), D-40-02 (CORRECT
  no-op), D-40-03 (SystemTime backing), D-40-04 (interrupting alarm deferral — the
  target), D-40-05 (centisecond resolution); D-38.4 (interrupt deferral context), D-38.8
  (AlarmType design), D-38.9 (check_alarms cadence)
- **`docs/hp41cv-divergences.md`** — D-CV-01 (lenient aliases), D-CV-02 (CLRALPHA legacy)
- **`docs/hp41-xmem-divergences.md`** — D-52-01 (overwrite-on-duplicate), D-52-02
  (full-register-set SAVED/GETD)
- **`docs/hp41-math1-divergences.md`** — D-30-01 (scratch register clobber — do not fix)
- **`.planning/PROJECT.md`** — v4.3 target description, deferred items, quality gate table,
  constraint list
- **`.planning/STATE.md`** — current progress metrics, performance targets
- **`.github/workflows/ci.yml`** — job definitions: lint, test (3-OS matrix), coverage
  (>=95%, Ubuntu/stable), msrv (1.88, lint+test only), license-audit, schema-aliases
- **`.github/workflows/ci-gui.yml`** — paths filter (`hp41-gui/**` + `hp41-core/**`),
  gui-ci + E2E smoke (`e2e-linux`, Ubuntu only)
- **`justfile`** — `just ci` (lint+test+coverage+license-audit), `just ci-msrv`,
  `just coverage` (`--fail-under-lines 95`), `just license-audit`, `just gui-ci`,
  `just schema-aliases-check`
- **`hp41-cli/src/app.rs`** — `drain_event_buffer` at line 1778: `alarm:xeq` handled
  at line 1783; `alarm:interrupting:...` silently ignored at line 1797 (the stub to fix)
- **CLAUDE.md memory `reference_msrv_clippy_lint_divergence`** — reproduce with
  `cargo +1.88 clippy --workspace --all-targets --all-features -- -D warnings`
- **CLAUDE.md memory `project_cfg_mobile_gate_blindspot`** — `just gui-ci` cannot
  catch `#[cfg(mobile)]` compile errors; use `cargo check --target aarch64-apple-ios`
