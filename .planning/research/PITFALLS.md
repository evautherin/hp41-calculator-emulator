# Pitfalls: HP-41CX Time Module Emulation (v3.2)

**Milestone:** v3.2 Time Module Emulation (third XROM module, XROM 26)
**Researched:** 2026-05-24
**Scope:** Pitfalls SPECIFIC to adding Time Module behavioral emulation on top of
the shipped v3.1 codebase. Pitfalls 1-31 from v3.0/v3.1 are mitigated and
already gate-checked in CI; they are NOT repeated here. Where a prior pitfall
has a Time-Module-specific extension, that extension is called out explicitly
with a cross-reference.

**Already mitigated (do not re-document):**
- P1: xrom_resolve fires LAST -- `tests/xrom_shadowing.rs` CI gate checks every
  `MATH_1.ops` + `STAT_1.ops` entry; v3.2 adds a `TIME.ops` gate.
- P11: Long-running op Mutex release + `request_cancel` -- v3.0 infrastructure.
  Time Module has no iterative solvers, so this is less relevant.
- P14: Cross-platform f64 drift -- already enforced via `lint_math1_assertions.rs`.
- P16: Per-Op test count >= 5 -- `math1_op_test_count.rs` gate extends to time ops.
- P17: `assert_eq!` on iterated HpNum -- already blocked by CI.
- P19 (Free42 contamination): `scripts/check-free42-contamination.sh` 18-token CI
  gate already runs on `math1/` + `stat1/`; v3.2 must add `time/` directory.
  See P40 for Time-specific contamination risks.
- P20 (serde shape): `rand_seed` established the `#[serde(default)]` without `skip`
  pattern for persistent non-obvious fields. See P33 for Time Module serde traps.
- P22 (XROM shadowing): disjointness test already covers Math 1 x Stat 1.
  v3.2 must extend to Math 1 x Stat 1 x Time. See P36 for specific conflicts.

**Confidence (overall):** MEDIUM-HIGH.
The Time Module's real-time aspects (P32, P34, P38) are the highest-risk area
because they introduce a genuinely new paradigm -- live updates independent of
user keystrokes -- into a previously keystroke-driven event loop. The date
arithmetic pitfalls (P35) are well-understood algorithmically (Tantzen Julian
day conversion is a solved problem). The alarm system (P37) is the most
complex feature and has MEDIUM confidence because the full XYZALM parameter
specification requires Owner's Manual verification.

---

## Summary

Twelve pitfall categories dominate the v3.2 risk surface. They break into four
clusters: real-time architecture (3 pitfalls), date/time representation (2),
alarm system complexity (2), and integration/framework (5):

1. **Live display updates without polling** (P32) -- CRITICAL. The stopwatch and
   clock display modes require sub-second LCD refresh, but `hp41-core` has no
   timer, no async, and no polling loop. The CLI's `event::poll(16ms)` already
   provides ~60fps rendering opportunity, but the core has no mechanism to
   "push" a display update without a keystroke. The GUI's "no polling (D-11)"
   invariant explicitly forbids frontend polling.

2. **System clock dependency in hp41-core** (P34) -- CRITICAL. `hp41-core` is
   currently I/O-free by design (`no async, no panics`). TIME and DATE must
   read the host system clock, which is an I/O operation. This requires either
   breaking the I/O-free invariant or introducing a clock abstraction layer.

3. **CalcState field explosion** (P33) -- the Time Module needs persistent state
   for: clock display mode (CLK12/CLK24/off), date format (DMY/MDY), accuracy
   factor, stopwatch accumulated time, stopwatch running flag, and the alarm
   catalog. Each field needs correct `#[serde(default)]` / `#[serde(skip)]`
   annotation. Getting this wrong breaks backward compat with v3.1 save files.

4. **Date decimal format parsing** (P35) -- dates as `MM.DDYYYY` or `DD.MMYYYY`
   stored as HpNum decimals have treacherous parsing edge cases (leading zeros,
   single-digit months, year extraction from fractional part).

5. **Alarm system state machine** (P37) -- the HP-41CX alarm system distinguishes
   control alarms (interrupting vs. non-interrupting), message alarms, and past-due
   alarms. Each has different behavior depending on calculator state (off, idle,
   running a program, displaying the clock). This is the most complex state
   machine in the emulator.

6. **XROM 26 mnemonic shadowing** (P36) -- the Time Module introduces ~33 function
   names. Several overlap with existing built-in or module names in subtle ways
   (e.g., "TIME" is common, "DATE" is common, "SW" is two chars).

7. **Stopwatch keyboard takeover** (P38) -- the HP-41CX's SW function reassigns the
   keyboard to dedicated stopwatch controls. This is a fundamentally different UX
   paradigm from anything in v1.0-v3.1.

8. **Alarm interrupt during program execution** (P39) -- interrupting control alarms
   can suspend a running program, execute a different program, and resume the
   original. This requires a re-entrancy mechanism distinct from the existing
   `call_stack` (which caps at 4 levels for user programs).

9. **Free42 contamination surface expansion** (P40) -- Free42/Plus42 implement
   DATE, TIME, DDAYS, DATE+, DOW, ADATE, ATIME functions. The contamination
   guard must extend to the new `time/` directory.

10. **Flag 31 (DMY/MDY) collision** (P41) -- the Time Module uses system flag 31
    to control date format. This flag is already in the `flags: u64` bitfield.
    But the emulator's current flag system treats all flags as user-settable;
    the Time Module gives flag 31 special semantic meaning.

11. **Accuracy factor and clock drift** (P42) -- the physical HP-41CX had a
    quartz crystal with drift correction (SETAF/RCLAF). The emulator uses the
    host OS clock which is already NTP-synchronized. Emulating the accuracy
    factor is semantically meaningless but must be stored for OM fidelity.

12. **4-way exhaustive match explosion** (P43) -- with ~33 new Op variants, the
    4-way exhaustive match invariant requires updating `dispatch()`,
    `execute_op()`, CLI `prgm_display.rs`, and GUI `prgm_display.rs` in
    lockstep. This is the largest single expansion since Math Pac I (52 entries).

---

## Critical Pitfalls

### P32: Live Display Updates Without a Polling Loop

**What goes wrong:** The stopwatch and clock display modes (CLKT, CLKTD) need
the LCD to update every second (clock) or every 1/100th second (stopwatch)
WITHOUT any user keystroke. The current architecture is purely reactive:
`hp41-core::dispatch()` runs ONLY when the user presses a key.

**Why it happens:** The emulator was designed as a keystroke-driven state
machine. There is no background thread, no timer interrupt, and no async
runtime in `hp41-core`. The CLI event loop polls at 16ms but only redraws
after key events. The GUI frontend explicitly forbids polling (D-11 invariant).

**Consequences:** Without architectural changes, the clock display freezes at
the time of the last keystroke. The stopwatch shows a static value until the
user presses a key to read it. This violates the most visible feature of the
Time Module.

**Prevention:**

*CLI:* The existing `event::poll(Duration::from_millis(16))` already returns
`false` (no key event) at ~60fps. The fix is to ALWAYS redraw after poll
returns -- not just when a key event arrives. When the clock/stopwatch is in
display mode, `get_display_string()` calls a time-provider function to get
the current time and formats it for display. The 16ms poll cadence is more
than sufficient for 1-second clock updates; for stopwatch centisecond
display, it provides ~60fps which is adequate.

*GUI:* The D-11 "no polling" invariant means the frontend does NOT call
`get_state()` on a timer. Two viable approaches:
  (a) **Tauri event emission** -- a background thread in the Tauri backend
      emits a `time-tick` event every 100ms (stopwatch) or 1000ms (clock);
      the React frontend subscribes and updates its display state.
  (b) **Frontend-only timer** -- the React frontend runs a `setInterval`
      that re-renders the time display locally, computed from the last known
      state + elapsed wall-clock time. This avoids IPC entirely but means
      the time display is frontend-computed, not core-computed.

Approach (b) is cleaner because it keeps `hp41-core` I/O-free and avoids
threading in the backend. The frontend already knows whether clock display
or stopwatch mode is active from `CalcStateView`.

**Detection:** Any `CLKT`, `CLKTD`, or `SW` command that does not produce a
visually updating display is a P32 failure.

**Phase:** Core architecture decision in the FIRST phase (framework phase).
This is a load-bearing design decision that affects every subsequent phase.

**Confidence:** HIGH -- the architectural constraint is clearly visible in
the existing event loop code (`app.rs:270`).

---

### P34: System Clock Dependency in I/O-Free hp41-core

**What goes wrong:** `Op::Time` must return the current time and `Op::Date`
must return the current date. These require reading the host OS clock, which
is an I/O operation. But `hp41-core` is explicitly I/O-free -- no network,
no file system, no system calls. The `no async` invariant is frozen.

**Why it happens:** The original HP-41CX has a dedicated quartz clock chip
(hardware I/O). The emulator maps this to the host OS clock. But the core
library was designed without any I/O surface.

**Consequences:** If `std::time::SystemTime::now()` or `chrono::Local::now()`
is called directly inside `hp41-core`, the crate gains a hidden I/O dependency
that breaks deterministic testing and violates the architectural invariant.

**Prevention:** Use a **clock trait abstraction** or **callback pattern**:

```rust
/// Clock provider trait -- injected at CalcState construction or
/// passed as a parameter to time-dependent ops.
pub trait ClockProvider {
    fn now(&self) -> (u8, u8, u8, u8);    // (hour, min, sec, centisecond)
    fn today(&self) -> (u8, u8, u16);     // (month, day, year) or (day, month, year)
}
```

Production implementations (`hp41-cli`, `hp41-gui`) inject a real
`SystemClockProvider`; tests inject a `MockClockProvider` with deterministic
values. This mirrors the `print_buffer` drain pattern -- core produces data,
frontend provides I/O.

**Alternative:** Store the clock value as a CalcState field updated by the
frontend before each dispatch. This is simpler but means TIME/DATE values
are stale by up to one poll cycle (16ms). For a calculator emulator with
1/100s precision, this is acceptable.

The simplest-correct approach: add `clock_time: Option<HpNum>` and
`clock_date: Option<HpNum>` as `#[serde(default, skip)]` transient fields
on `CalcState`. The frontend sets these before every `dispatch()` call.
`Op::Time` and `Op::Date` read from these fields. Tests set them explicitly.
Zero new traits, zero new dependencies, zero I/O in core.

**Detection:** Any `use std::time` or `use chrono` in `hp41-core/src/` is a
P34 violation.

**Phase:** Core architecture decision in the FIRST phase (framework phase).

**Confidence:** HIGH -- the I/O-free invariant is documented in CLAUDE.md and
enforced by the `no async` frozen invariant.

---

### P37: Alarm System State Machine Complexity

**What goes wrong:** The HP-41CX alarm system has four alarm types, three
triggering contexts, past-due queuing, repeat intervals, and alarm
acknowledgment. Implementing this as a flat match block produces an
unmaintainable combinatorial explosion.

**Why it happens:** The alarm system interacts with every other subsystem:
- **Control alarms (interrupting):** suspend a running program, execute
  the alarm's target program/function, then resume the interrupted program.
- **Control alarms (non-interrupting):** if the calculator is idle/off,
  execute the target; if a program is running, become past-due (deferred).
- **Message alarms:** display ALPHA text and sound tones; no program execution.
- **Past-due alarms:** accumulate in a queue; `ALMNOW` activates the oldest;
  `ALMCAT` lists them; acknowledged alarms are removed.

The triggering context (calculator off, idle, running a program, displaying
the clock) changes the behavior of every alarm type.

**Consequences:** Incorrect state transitions produce:
- Alarms that fire during programs but corrupt the call stack (P39).
- Past-due alarms that never fire or fire repeatedly.
- Acknowledged alarms that reappear.
- ALMCAT listing that shows incorrect order.

**Prevention:**
1. Implement alarms as a `Vec<Alarm>` on CalcState with a structured
   `Alarm` type (not raw register values):
   ```rust
   struct Alarm {
       time: HpNum,         // HH.MMSSss
       date: HpNum,         // MM.DDYYYY or DD.MMYYYY
       repeat: HpNum,       // repeat interval (0 = no repeat)
       kind: AlarmKind,     // Control(interrupting/non), Message
       target: String,      // ALPHA label or message text
       past_due: bool,      // has this alarm's trigger time passed?
   }
   ```
2. Implement alarm triggering as a separate `check_alarms()` function
   called by the frontend (not by `dispatch()`), similar to how
   `print_buffer` is drained externally.
3. Test each alarm type x each context (4 x 4 = 16 combinations)
   as explicit test cases.

**Detection:** Any alarm test that only covers one alarm type or one context
is incomplete.

**Phase:** Should be a dedicated phase AFTER basic clock/date functions work.
Alarms are the most complex part of the Time Module and should not be
attempted in the same phase as the clock framework.

**Confidence:** MEDIUM -- the XYZALM parameter format requires Owner's Manual
verification. The search results confirm three stack registers (X = time,
Y = date, Z = repeat interval) plus ALPHA (target label or message), but
the exact encoding of alarm type (interrupting vs. non-interrupting via `~`
prefix in ALPHA) needs OM confirmation.

---

## Moderate Pitfalls

### P33: CalcState Field Explosion and Serde Shape

**What goes wrong:** The Time Module requires at least 6-8 new `CalcState`
fields (clock display mode, date format, accuracy factor, stopwatch time,
stopwatch state, alarm catalog, clock injection fields). Each field needs
the correct `#[serde(default)]` and optionally `#[serde(skip)]` annotation.
A single wrong annotation either breaks save-file backward compatibility
(missing `default`) or persists transient state that should not survive a
restart (missing `skip` on a transient field).

**Why it happens:** The v3.1 `rand_seed` established a unique pattern
(`default` WITHOUT `skip`) that is easy to mis-apply. Time Module fields
fall into three categories:
- **Persistent, non-obvious:** accuracy factor, date format preference,
  clock display mode preference. These survive save/load. Pattern:
  `#[serde(default)]` (no `skip`).
- **Persistent, obvious:** alarm catalog. Pattern: `#[serde(default)]`
  (no `skip`). But the alarm catalog could be large (unbounded Vec).
- **Transient:** stopwatch running flag, clock injection values,
  pending alarm event. Pattern: `#[serde(default, skip)]`.

**Consequences:** v3.1 save files that lack Time Module fields fail to
deserialize (missing `default`). Or stopwatch state persists across
sessions (missing `skip` on the running flag), causing a stopwatch to
appear "running" on load even though no timer is active.

**Prevention:**
1. Document each new field's serde shape in the field's doc-comment
   (following the `rand_seed` precedent at `state.rs:176-200`).
2. Write a backward-compat test loading a v3.1 save fixture (following
   `stat1_backward_compat.rs` + `v30-autosave.json` precedent from Phase 37).
3. Extend `migrate_after_load()` to set bit 2 on `xrom_modules` for the
   Time Module (pattern: v3.0 -> v3.1 migration set bit 1).

**Decision matrix for Time Module fields:**

| Field | Persistent? | `#[serde(default)]` | `#[serde(skip)]` | Rationale |
|-------|-------------|---------------------|-------------------|-----------|
| `clock_display_mode` | YES | YES | NO | User preference survives restart |
| `accuracy_factor` | YES | YES | NO | Calibration data survives restart |
| `alarm_catalog` | YES | YES | NO | Alarms survive restart |
| `stopwatch_elapsed` | YES | YES | NO | Stopwatch accumulated time survives power-off on real HP-41CX |
| `stopwatch_running` | NO | YES | YES | Timer state is transient |
| `clock_time_injection` | NO | YES | YES | Frontend injects before dispatch |
| `clock_date_injection` | NO | YES | YES | Frontend injects before dispatch |
| `pending_alarm_event` | NO | YES | YES | Alarm trigger is transient |

**Detection:** The backward-compat test is the CI guard. Any new CalcState
field without `#[serde(default)]` is caught by the existing
`v22_save_loads_with_defaults` test (which loads a minimal JSON without
any v3.2 fields).

**Phase:** Core framework phase. Fields must be declared before any Op
implementation.

**Confidence:** HIGH -- the pattern is well-established from v3.0 and v3.1.

---

### P35: Date Decimal Format Parsing Edge Cases

**What goes wrong:** HP-41CX dates are stored as `MM.DDYYYY` (when flag 31
is clear / MDY mode) or `DD.MMYYYY` (when flag 31 is set / DMY mode). These
are HpNum decimals. Parsing them is treacherous because:

1. **Single-digit months:** January 15, 1982 = `1.151982`, not `01.151982`.
   The integer part is `1`, the fractional part is `.151982`. Extracting
   DD and YYYY from the fractional part requires knowing the expected field
   widths.

2. **Leading zeros in day:** March 5, 2026 in MDY = `3.052026`. The
   fractional part `.052026` must NOT be parsed as `52026` (which would
   imply day=52, year=026).

3. **Year extraction:** The year is the last 4 digits of the fractional part.
   For `3.052026`, the fractional string is `052026`, day = `05`, year = `2026`.
   But for `12.252026` (Dec 25, 2026), the fractional string is `252026`,
   day = `25`, year = `2026`.

4. **Gregorian calendar boundary:** Valid dates start at October 15, 1582
   (10.151582 in MDY). Any date before this is invalid.

5. **Century boundaries:** Year 2000 = `.DDYYYY` where YYYY=2000. But
   `1.012000` could be parsed as month=1, day=01, year=2000 OR as the
   decimal number 1.012 (if trailing zeros are lost). The `rust_decimal`
   representation preserves trailing zeros, but this must be verified.

6. **Decimal precision:** HpNum uses `rust_decimal` with 10-significant-digit
   rounding. The date `12.312026` has 8 significant digits. The date
   `1.012000` has 7 significant digits (or 4 if trailing zeros are dropped).
   The parsing algorithm must NOT rely on digit counting.

**Prevention:** Parse dates by string-splitting at the decimal point (the
same technique used for ISG/DSE counters per the frozen invariant in CLAUDE.md).
Extract the integer part as the month (or day in DMY mode). Extract the
fractional part as a 6-character string (zero-padded on the LEFT to ensure
exactly 6 digits), then split into DD (first 2 chars) and YYYY (last 4 chars).

```rust
// Correct pattern (mirrors ISG/DSE parse_counter):
let s = hpnum.to_string();
let (int_part, frac_part) = s.split_once('.').unwrap_or((&s, ""));
let month = int_part.parse::<u8>()?;
let padded = format!("{:0>6}", frac_part);  // left-pad to 6 chars
let day = padded[..2].parse::<u8>()?;
let year = padded[2..6].parse::<u16>()?;
```

**Detection:** Test with these edge-case dates:
- `1.012000` (Jan 1, 2000 -- Y2K boundary, trailing zeros)
- `12.312026` (Dec 31, 2026 -- max month/day)
- `10.151582` (Oct 15, 1582 -- Gregorian start)
- `1.011583` (Jan 1, 1583 -- just after Gregorian start)
- `12.319999` (Dec 31, 9999 -- max supported date)
- `2.292024` (Feb 29, 2024 -- leap day)
- `2.292023` (Feb 29, 2023 -- INVALID, not a leap year)

**Phase:** Core implementation phase, as part of DATE+/DDAYS ops.

**Confidence:** HIGH -- the ISG/DSE string-split precedent is well-established,
and the Tantzen Julian day algorithm (ACM Algorithm 199, 1963) is a standard
reference for date arithmetic.

---

### P36: XROM 26 Mnemonic Shadowing Across Three Modules

**What goes wrong:** The Time Module (XROM 26) introduces ~33 function
mnemonics. Several have collision risk with existing built-in names or
XROM module names:

**Known collision candidates:**
- `TIME` -- not in `builtin_card_op` or Math 1 or Stat 1. SAFE.
- `DATE` -- not in any existing resolver. SAFE.
- `DATE+` -- the `+` character may cause XEQ-by-name input issues.
- `T+X` -- the `+` character again; also `T` is a stack register name.
- `DMY` / `MDY` -- short mnemonics, not in existing resolvers. SAFE.
- `SW` -- two-character mnemonic. Not in existing resolvers. SAFE but fragile.
- `SIZE` -- ALREADY REGISTERED as `Op::MatSize` in `MATH_1.ops`! This is a
  CONFIRMED COLLISION. The Time Module does NOT define SIZE; the Math Pac I
  does. But if future modules reuse "SIZE", the collision would shadow.
- `CORRECT` -- not in existing resolvers. SAFE.
- `CLK12` / `CLK24` -- not in existing resolvers. SAFE.

**Actually confirmed safe (no collision):** After checking `builtin_card_op`
(4 card ops + 8 conditional tests), `MATH_1.ops` (52 entries), and
`STAT_1.ops` (26 entries), none of the ~33 Time Module mnemonics collide
with existing entries EXCEPT the note about SIZE above (which is Math Pac I,
not Time Module).

**Why it happens:** The resolver chain fires Math 1 -> Stat 1 -> Time (once
bit 2 is added). If a Time mnemonic duplicates a Math 1 mnemonic, Math 1
wins silently. This is correct per the hardware behavior (modules in lower
port numbers have priority), but it could be confusing.

**Prevention:**
1. Extend `tests/xrom_shadowing.rs` to cross-check `TIME.ops` against
   `MATH_1.ops` + `STAT_1.ops` + `builtin_card_op`.
2. Verify all ~33 mnemonics before coding any resolver arms.
3. For `DATE+` and `T+X`: ensure the `+` character is accepted by the
   XEQ-by-name text input modal (currently the modal accepts alphanumeric
   chars; symbols like `+` may need to be explicitly allowed).

**Detection:** `xrom_shadowing.rs` CI gate catches any collision at compile
time.

**Phase:** Core framework phase (XROM registration).

**Confidence:** HIGH -- the shadowing test infrastructure already exists.

---

### P38: Stopwatch Keyboard Takeover Mode

**What goes wrong:** On the real HP-41CX, the `SW` (Stopwatch) function
reassigns the entire keyboard to dedicated stopwatch controls:
- Top row: split/lap functions
- Number keys: not available
- R/S: start/stop
- Special keys: reset, read

This is fundamentally different from any existing UI mode in the emulator.
ALPHA mode only redirects letter keys; PRGM mode only changes what dispatch
does. Stopwatch mode reassigns the MEANING of physical keys.

**Why it happens:** The HP-41CX hardware had dedicated microcode for
stopwatch keyboard scanning. The emulator must simulate this without
microcode-level emulation.

**Consequences:**
- If stopwatch mode is a CalcState flag, then `hp41-core::dispatch()` must
  check it and route to stopwatch ops instead of normal ops. This adds a
  branch to every single dispatch call.
- If stopwatch mode is a frontend-only flag (like `shift_armed`), then the
  CLI and GUI must independently implement the keyboard reassignment, risking
  CLI-GUI parity divergence.
- The real HP-41CX's stopwatch keyboard is NOT programmable (it cannot be
  used from within a program). This means `Op::Sw` is valid in programs but
  `Op::SwStart`/`Op::SwStop` etc. are NOT.

**Prevention:**
1. Model stopwatch mode as a `CalcState` flag (similar to `alpha_mode` or
   `prgm_mode`). When set, `dispatch()` routes to a `stopwatch_dispatch()`
   sub-function.
2. `stopwatch_dispatch()` handles only the 5-6 stopwatch keys; all other
   keys are ignored or produce a soft error.
3. The flag is `#[serde(default, skip)]` -- stopwatch mode does not survive
   a save/load cycle (the real HP-41CX exits stopwatch mode on power-off).
4. CLI and GUI both check this flag when interpreting key events, ensuring
   parity.

**Detection:** Any test that operates the stopwatch without entering
stopwatch mode is testing the wrong thing.

**Phase:** Dedicated stopwatch phase, AFTER basic clock/date. This is a
new UI paradigm and should not be mixed with the date arithmetic phase.

**Confidence:** MEDIUM -- the exact set of stopwatch keys and their mappings
requires Owner's Manual verification.

---

### P39: Alarm Interrupt Re-entrancy vs. Call Stack

**What goes wrong:** An interrupting control alarm can fire DURING program
execution. On the real HP-41CX, this suspends the current program, runs the
alarm's target program, and then resumes the interrupted program. But the
emulator's `call_stack` has a 4-level hardware limit (D-14). If the running
program has already used 3 call levels, the alarm's XEQ pushes a 4th, and
the alarm target itself calls a subroutine -- stack overflow.

**Why it happens:** The HP-41CX hardware has a separate alarm interrupt
mechanism outside the normal subroutine call stack. The emulator currently
has only one call stack.

**Consequences:**
- If interrupting alarms use the normal `call_stack`, programs running at
  deep call levels lose alarm functionality.
- If a separate interrupt stack is added, it must not interfere with the
  `is_running` flag, `pc`, or any other program execution state.
- Resume-after-alarm must restore the EXACT program counter, stack contents,
  and register state from before the interruption.

**Prevention:**
1. **Defer interrupting alarm support** -- implement only non-interrupting
   alarms and message alarms first. Document interrupting alarms as a
   known emulator divergence. This follows the project's "behavioral
   emulation, not cycle-accurate" philosophy.
2. If interrupting alarms are implemented: save the entire execution
   context (pc, call_stack, is_running) to a separate `alarm_context:
   Option<AlarmContext>` field BEFORE dispatching the alarm target. Restore
   after the alarm target returns.
3. Nest depth guard: if `call_stack.len() >= 4` when an interrupting alarm
   fires, demote it to non-interrupting (past-due) behavior.

**Detection:** Test an interrupting alarm firing at call_stack depth 3.

**Phase:** Alarm phase (if implemented). Strong recommendation to DEFER
interrupting alarms to a follow-up milestone or mark as documented
divergence.

**Confidence:** MEDIUM -- the exact HP-41CX interrupt mechanism is not
fully documented in public sources. The Owner's Manual describes the
user-visible behavior but not the internal stack management.

---

### P40: Free42 Contamination Surface Expansion

**What goes wrong:** Free42 and Plus42 implement DATE, TIME, DDAYS, DATE+,
DOW, ADATE, ATIME, ATIME24, CLK12, CLK24, DMY, MDY, and YMD functions.
The implementations in Free42's `core_commands7.cc` are GPL-licensed. Any
copy-paste or structural duplication triggers the contamination guard.

**Why it happens:** When implementing date arithmetic (DDAYS, DATE+, DOW),
it is tempting to consult Free42's implementation for the correct algorithm.
Free42 uses the Tantzen Julian day conversion -- which is a PUBLIC DOMAIN
algorithm from 1963 -- but Free42's specific IMPLEMENTATION of it is
GPL-licensed code.

**Consequences:** GPL contamination of `hp41-core` (MIT-licensed).

**Prevention:**
1. Extend `scripts/check-free42-contamination.sh` to scan `hp41-core/src/ops/time/`
   as a third directory alongside `math1/` and `stat1/`.
2. Add Time-specific tokens to the contamination pattern:
   - `core_commands7` (Free42's time module source file name)
   - `date2j` / `j2date` (Free42's Julian conversion function names)
   - Any other Free42-specific identifiers discovered during implementation.
3. Re-derive the Tantzen algorithm from the original 1963 ACM paper
   (Algorithm 199), NOT from Free42's implementation.
4. Carry the verbatim disclaim header on every `time/*.rs` file:
   `// Algorithm independently re-derived from primary sources; Free42
   // source consulted only as sanity-check oracle, not copied.`

**Detection:** `check-free42-contamination.sh` CI gate.

**Phase:** Core framework phase (script extension before any time code).

**Confidence:** HIGH -- the contamination guard pattern is well-established.

---

### P41: Flag 31 (DMY/MDY) Semantic Collision

**What goes wrong:** The HP-41CX Time Module uses system flag 31 to control
date format:
- Flag 31 clear = MDY (Month-Day-Year)
- Flag 31 set = DMY (Day-Month-Year)

Additionally, the Time Module uses flag 44 as the "continuous on" flag (prevents
auto power-off), and the clock display mode may interact with other system flags.

The emulator's current flag system (`flags: u64` on CalcState) stores all 56
flags as a flat bitfield. `Op::SfFlag(31)` and `Op::CfFlag(31)` already work.
But the Time Module's `DMY` and `MDY` ops are ALIASES for `SF 31` and `CF 31`
respectively. If the emulator implements `DMY` as a separate op that doesn't
also set flag 31, or implements flag 31 without checking it in date formatting,
the two systems drift.

**Prevention:**
1. Implement `Op::Dmy` as `state.flags |= (1 << 31)` -- literally the same
   operation as `Op::SfFlag(31)`.
2. Implement `Op::Mdy` as `state.flags &= !(1 << 31)`.
3. All date formatting functions check `state.flags & (1 << 31)` to determine
   the format, NOT a separate `date_format` field.
4. Do NOT add a separate `date_format: DateFormat` field to CalcState. Use
   the flag directly. The HP-41CX hardware uses the flag, not a separate register.

**Detection:** Test that `SF 31` + `DATE` produces DMY format, and `CF 31` +
`DATE` produces MDY format. Also test that `DMY` sets flag 31 and `MDY` clears it.

**Phase:** Core date ops phase.

**Confidence:** HIGH -- the flag system is well-understood and already implemented.

---

## Minor Pitfalls

### P42: Accuracy Factor Semantic Vacuity

**What goes wrong:** The HP-41CX's SETAF and RCLAF functions set and recall
a clock accuracy factor (-99.9 to 99.9 PPM). On the real hardware, this
adjusts the quartz crystal frequency to compensate for temperature drift.
On the emulator, the host OS clock is already NTP-synchronized to sub-second
accuracy. The accuracy factor is semantically meaningless.

**Prevention:** Implement SETAF and RCLAF as a simple store/recall of an
HpNum field on CalcState. The CORRECT function (which sets time and adjusts
the accuracy factor simultaneously) stores the factor but does not actually
adjust any clock. Document this as an explicit emulator divergence.

**Phase:** Clock ops phase.

**Confidence:** HIGH.

---

### P43: 4-Way Exhaustive Match Explosion (~33 Variants)

**What goes wrong:** The Time Module's ~33 functions require ~33 new Op enum
variants. Each must be added to:
1. `dispatch()` in `hp41-core/src/ops/mod.rs`
2. `execute_op()` in `hp41-core/src/ops/program.rs`
3. `op_display_name()` in `hp41-cli/src/prgm_display.rs`
4. `op_display_name()` in `hp41-gui/src-tauri/src/prgm_display.rs`

This is the 4-way exhaustive match invariant. With ~33 new variants, this is
the largest single expansion since the original Math Pac I (52 entries).

**Prevention:** Follow the established Phase 33/34/36 cadence:
- Phase N (core): add all Op variants to items 1+2. Intentional CI break
  in items 3+4 (sanctioned-deferred).
- Phase N+1 (CLI): close item 3.
- Phase N+2 or N+3 (GUI): close item 4.

Do NOT attempt to add all 4 sites in a single phase. The sanctioned-CI-break
pattern from Phase 33 (Stat 1) worked well.

**Detection:** Compile error in `hp41-cli` and `hp41-gui` until items 3+4
are closed. This is the invariant working as designed.

**Phase:** Core phase -> CLI phase -> GUI phase (3 separate phases).

**Confidence:** HIGH -- the pattern is well-established from v3.0 and v3.1.

---

### P44: Time Format HH.MMSSss Parsing

**What goes wrong:** HP-41CX times are stored as `HH.MMSSss` where HH is
hours (0-23), MM is minutes (0-59), SS is seconds (0-59), and ss is
centiseconds (0-99). This is NOT the same as the H.MMSS format used by
HMS->H and ->HMS in the existing `hms.rs`. The Time Module format includes
CENTISECONDS (two extra fractional digits).

- HMS format:  `H.MMSS` (4 fractional digits: MM, SS)
- Time format: `HH.MMSSss` (6 fractional digits: MM, SS, ss)

**Why it happens:** The Time Module needs finer granularity (centiseconds)
for the stopwatch function.

**Consequences:** If the existing `parse_hms()` function in `hms.rs` is
reused for time parsing, centiseconds are silently truncated. If a new
parser is written, it must handle the 6-digit fractional part correctly,
including left-padding to ensure exactly 6 digits.

**Prevention:**
1. Write a NEW `parse_time()` function specifically for `HH.MMSSss` format.
   Do NOT reuse `parse_hms()` from `hms.rs` (which handles `H.MMSS`).
2. Use the same string-split-at-decimal technique as ISG/DSE and date parsing.
3. Validate: HH in 0..23, MM in 0..59, SS in 0..59, ss in 0..99.
4. The SETIME function accepts `HH.MMSSss`; the TIME function returns
   `HH.MMSSss` with the current centiseconds from the system clock.

**Detection:** Test `23.595999` (max valid time: 23:59:59.99) and `0.000000`
(midnight exactly).

**Phase:** Core clock ops phase.

**Confidence:** HIGH -- the format difference is clearly documented.

---

### P45: Stopwatch Accumulated Time Persistence

**What goes wrong:** On the real HP-41CX, the stopwatch accumulates elapsed
time even when the calculator is off (the quartz crystal keeps counting).
In the emulator, there is no background process when the app is closed.
If the stopwatch is "running" when the user quits, the accumulated time
on next launch should include the elapsed wall-clock time since quit.

**Consequences:** If the stopwatch running state and start timestamp are
not persisted correctly, the stopwatch resets on every app restart. If they
ARE persisted but the start timestamp is a `SystemTime` absolute value,
deserialization on a different machine or after a timezone change produces
wrong results.

**Prevention:**
1. Store `stopwatch_elapsed: HpNum` (accumulated centiseconds) as a
   persistent `#[serde(default)]` field.
2. Store `stopwatch_start: Option<u64>` (Unix epoch millis when started)
   as a persistent `#[serde(default)]` field.
3. On load, if `stopwatch_start` is `Some(t)`, compute additional elapsed
   time as `now - t` and add to `stopwatch_elapsed`. Then update
   `stopwatch_start` to `now`.
4. Alternative (simpler): do NOT persist stopwatch running state. Document
   that the emulator stops the stopwatch on exit (divergence from hardware).
   The SETSW function restores a specific stopwatch value anyway.

**Phase:** Stopwatch phase.

**Confidence:** MEDIUM -- the exact HP-41CX stopwatch persistence behavior
needs Owner's Manual verification.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Core framework + clock trait | P32 (live display), P34 (I/O-free), P33 (serde) | Clock injection pattern decided FIRST; serde shape for all fields declared |
| XROM registration | P36 (shadowing), P43 (4-way match) | Verify all ~33 mnemonics; extend shadowing test |
| Date arithmetic (DATE+, DDAYS, DOW) | P35 (date parsing), P41 (flag 31), P40 (Free42) | String-split parsing; Tantzen from primary source; flag 31 = sole DMY/MDY control |
| Time functions (TIME, DATE, ATIME, ADATE) | P34 (I/O-free), P44 (HH.MMSSss) | Inject clock from frontend; new parser, not reuse hms.rs |
| Clock display (CLKT, CLKTD) | P32 (live display) | CLI: always-redraw; GUI: frontend timer |
| Stopwatch (SW, RUNSW, STOPSW, SETSW) | P38 (keyboard takeover), P32 (live display), P45 (persistence) | CalcState flag for SW mode; elapsed time field |
| Alarm system (XYZALM, ALMCAT, RCLALM) | P37 (alarm state machine), P39 (interrupt re-entrancy) | Structured Alarm type; defer interrupting alarms |
| CLI integration | P43 (4-way match), P32 (display updates) | Sanctioned CI break resolved; always-redraw in event loop |
| GUI integration | P43 (4-way match), P32 (display updates) | Frontend timer for clock/stopwatch display |
| Free42 contamination | P40 (surface expansion) | Extend script to time/ dir; add new tokens |
| Test hardening | P33 (backward compat), P35 (date edge cases) | v3.1 save fixture test; date edge case suite |
| Documentation | P42 (accuracy factor), P37 (alarm) | Divergence catalog for accuracy factor |

---

## XROM 26 Complete Function Reference

Based on research, the HP-41CX Time Module (XROM 26) defines the following
functions. Function IDs from multiple sources (not all confirmed individually):

| # | Mnemonic | XROM | Category | Description |
|---|----------|------|----------|-------------|
| 1 | ADATE | 26,01 | Alpha | Append date to ALPHA |
| 2 | ALMCAT | 26,02 | Alarm | List all pending/past-due alarms |
| 3 | ALMNOW | 26,03 | Alarm | Activate oldest overdue alarm |
| 4 | ATIME | 26,04 | Alpha | Append time to ALPHA (CLK12/24) |
| 5 | ATIME24 | 26,05 | Alpha | Append time in 24h format |
| 6 | CLK12 | 26,06 | Clock | Set 12-hour display |
| 7 | CLK24 | 26,07 | Clock | Set 24-hour display |
| 8 | CLKT | 26,08 | Clock | Time-only display |
| 9 | CLKTD | 26,09 | Clock | Time and date display |
| 10 | CLOCK | 26,10 | Clock | Display the clock (non-programmable?) |
| 11 | CORRECT | 26,11 | Clock | Set time + adjust accuracy factor |
| 12 | DATE | 26,12 | Date | Recall current date to X |
| 13 | DATE+ | 26,13 | Date | Y-date + X-days = new date |
| 14 | DDAYS | 26,14 | Date | Days between two dates |
| 15 | DMY | 26,15 | Date | Set Day-Month-Year format (SF 31) |
| 16 | DOW | 26,16 | Date | Day of week (0=Sun .. 6=Sat) |
| 17 | MDY | 26,17 | Date | Set Month-Day-Year format (CF 31) |
| 18 | RCLAF | 26,18 | Clock | Recall accuracy factor |
| 19 | RCLSW | 26,19 | Stopwatch | Recall stopwatch time to X |
| 20 | RUNSW | 26,20 | Stopwatch | Start stopwatch |
| 21 | SETAF | 26,21 | Clock | Set accuracy factor |
| 22 | SETDATE | 26,22 | Clock | Set clock date from X |
| 23 | SETSW | 26,23 | Stopwatch | Set stopwatch starting time |
| 24 | STOPSW | 26,24 | Stopwatch | Halt stopwatch |
| 25 | SW | 26,25 | Stopwatch | Enter Stopwatch mode |
| 26 | T+X | 26,26 | Clock | Adjust clock time by X |
| 27 | TIME | 26,27 | Time | Recall current time to X |
| 28 | XYZALM | 26,28 | Alarm | Set alarm from X/Y/Z/ALPHA |
| -- | (gap) | 26,29-30 | -- | Not assigned |
| 29 | CLALMA | 26,31 | Alarm | Clear alarm by ALPHA match |
| 30 | CLALMX | 26,32 | Alarm | Clear alarm by X value |
| 31 | CLRALMS | 26,33 | Alarm | Clear all alarms |
| 32 | RCLALM | 26,34 | Alarm | Recall alarm parameters |
| 33 | SWPT | 26,35 | Stopwatch | Stopwatch split point |

**Total: 33 functions across 5 categories** (Clock: 8, Date: 5, Time: 1,
Stopwatch: 6, Alarm: 6, Alpha formatting: 3, Clock adjustment: 4).

**Note:** Some functions (CLOCK, SW) may not be programmable on real hardware.
This needs Owner's Manual verification.

---

## Sources

- [HP-41C XROM Numbers](https://www.hpmuseum.org/software/xroms.htm) -- XROM ID 26 for Time Module
- [HP 82182A Time Module QREF](https://qrg41.fjk.ch/hp82182a.html) -- Function list and descriptions
- [HP-41 Module Database](https://calc.fjk.ch/db/hp41mod.php) -- Time Module 1A/1B/1C/2C all XROM 26
- [HP-41CX Comparison](http://holyjoe.org/hp/New-in-41CX.pdf) -- CX vs CV/C differences
- [DDAYS and DATE+ Implementation](https://archived.hpcalc.org/hp42s/programs/date/olddate.html) -- Tantzen ACM Algorithm 199
- [HP-41CX QRG](https://literature.hpcalc.org/community/hp41cx-qrg-en.pdf) -- Quick reference guide
- [Time Module Owner's Manual Section 4](https://archived.hpcalc.org/greendyk/hp41c-time-module/48-contents.html) -- Alarm system types
- [go41cx XYZALM Issues](https://forum.hp41.org/viewtopic.php?f=21&t=649) -- Emulator alarm bugs
- [Free42 Project](https://thomasokken.com/free42/) -- GPL-licensed time functions
- [HP Museum Forum: Time Module Flags](https://www.hpmuseum.org/forum/thread-8569.html) -- Flag 31 and 44 usage
- [HP-41 Daytimer](http://wilsonminesco.com/HP-41daytimer.html) -- XYZALM parameter usage
- [DM41X Manual](https://technical.swissmicros.com/dm41x/doc/dm41x_user_manual.html) -- Modern Time Module emulation
- [HP-41CX Date Format Discussion](https://www.hpmuseum.org/cgi-sys/cgiwrap/hpmuseum/archv016.cgi?read=90046) -- MM.DDYYYY format details
- [Free42 Date Tools](https://richmit.github.io/hp42/date.html) -- Date function subset from HP-41 Time Module

**Confidence per source:**
- XROM ID 26: HIGH (multiple independent sources agree)
- Function list: MEDIUM-HIGH (cross-referenced across QREF, Wikipedia, finseth.com)
- XYZALM parameters: MEDIUM (Owner's Manual excerpts only, not full spec)
- Alarm types: MEDIUM (Section 4 excerpt confirms interrupt/non-interrupt distinction)
- Date format: HIGH (universally documented as MM.DDYYYY / DD.MMYYYY with flag 31)
- Stopwatch keyboard takeover: LOW-MEDIUM (referenced but not fully documented in search results)
