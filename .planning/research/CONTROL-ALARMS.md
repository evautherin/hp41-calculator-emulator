# Control-Alarm Semantics (HP-41CX Time Module) — Fidelity Research

**Researched:** 2026-06-06
**Sources:** HP-41CX Owner's Manual Vol. 2 (ManualsLib manual/3559140), HP 82182A Time Module QREF (qrg41.fjk.ch), HP-41CX Quick Reference Guide (literature.hpcalc.org), existing codebase (alarm.rs, program.rs, state.rs), project divergence doc (docs/hp41-time-divergences.md §D-40-04).
**Confidence:** MEDIUM — key behavioral facts extracted from OCR'd manual pages (ManualsLib) and cross-validated against the QREF. Verbatim OCR reproduction has noise. The subroutine-level interaction and exact acknowledgment keystrokes are the most uncertain areas.

---

## Summary

The HP-41CX Time Module supports three alarm types: message, control (`>label`), and conditional (`>>label`). Control alarms run their program immediately when due — including interrupting a running program by acting as a subroutine call against the 4-level return stack. Conditional alarms run their program only when the calculator is idle or off, never interrupting a running program. The emulator currently stores both types faithfully but fires all control/conditional alarms as deferred (non-interrupting) events. Implementing interrupting control alarm semantics requires the running `run_loop` to check for due alarms at instruction boundaries and push the alarm label onto the call stack — a form of in-loop re-entrancy the current architecture defers. The hardware-sourced evidence is sufficient to implement this correctly, with a few explicitly-flagged uncertain edge cases.

---

## Alarm Type Taxonomy

The HP-41CX Owner's Manual Vol. 2, Section 16 defines three alarm types, determined by the ALPHA register content at the time of `XYZALM` execution:

| Type | ALPHA Register at XYZALM | Emulator Representation |
|------|--------------------------|-------------------------|
| **Message** | Any text (no leading `>`) | `AlarmType::Message(String)` |
| **Control** | `>label` — single right-arrow prefix | `AlarmType::Control { label, interrupting: false }` |
| **Conditional** | `>>label` — two right-arrow prefix | `AlarmType::Control { label, interrupting: true }` |

**CRITICAL NAMING MISMATCH:** The OM uses "control" for the `>label` (non-interrupting) variant and "conditional" for the `>>label` (interrupting) variant. The project codebase uses the field name `interrupting: bool` with `true` for `>>label`. This means: `interrupting: true` = OM "conditional" = `>>label`. `interrupting: false` = OM "control" = `>label`. This naming flip is counterintuitive and a source of implementation risk — double-check the semantics against the OM terminology before writing code.

**OM definition of "control alarm" (`>label`, `interrupting: false`):**
- Runs the named program or catalog-2 function when the alarm comes due.
- Runs "whether the HP-41 is on or off, as soon as any currently operating function is done." (Manual p.248 area)
- Is self-acknowledging — it clears itself from memory automatically (or resets if repeating).
- Will interrupt a running program: fires as a subroutine call against the current return stack.

**OM definition of "conditional alarm" (`>>label`, `interrupting: true` in codebase):**
- Runs its program only if the HP-41 is off or displaying the clock.
- "Will not interrupt the execution of a function in progress, but waits until one function is finished before going off." (Manual p.105 area, OCR-uncertain)
- If the calculator is running a program when the alarm fires, the conditional alarm becomes past-due rather than interrupting.
- Conditional alarms "neither interrupt a program nor wait for completion; [they] simply sound a pair of tones and become past due." (Manual p.113, simultaneous alarms section)

**Key inversion from project naming:** The OM's "conditional" alarm (`>>label`) is the LESS aggressive type — it defers rather than interrupts. The OM's "control" alarm (`>label`) is the MORE aggressive type — it interrupts a running program. The project field name `interrupting: true` is attached to `>>label`, which is the OM's conditional (deferred) type. This must be verified carefully during implementation.

---

## Interrupting Control Alarm — Trigger-to-Resume Sequence

The following sequence describes the OM's `>label` control alarm firing while a program is running (the case that requires re-entrancy).

### Control Alarm (`>label`, `interrupting: false` in codebase) — Fires During Running Program

1. **Alarm comes due** (trigger_unix <= now). The HP-41CX hardware detects this via its real-time clock module, independently of the CPU.

2. **Instruction boundary wait.** The alarm does not halt mid-instruction. It "waits until one currently operating function is done" before activating. Emulator translation: the alarm fires at the top of the `run_loop` iteration, after `state.pc += 1` and before the next `op` is dispatched.

3. **Call stack check.** The alarm performs an implicit XEQ against the call stack. If the call stack is already at 4 levels deep, the alarm "use[s] too many subroutine levels" and the interrupted program will not be resumed after the alarm completes. (Manual p.222: "Control then returns to program ABC assuming that program XYZ did not stop or turn off the computer or use too many subroutine levels.") The exact behavior when the stack is full is uncertain — it may terminate the running program, or the alarm program may simply not execute. See Open Questions.

4. **Push return address.** The current `pc` (the instruction that would have executed next in the interrupted program) is pushed onto `call_stack`. This consumes one of the 4 available return levels.

5. **Find alarm label.** The alarm's label is looked up in `program` using the same search as `Op::Xeq`. If not found: behavior uncertain (see Open Questions).

6. **Jump to alarm program.** `state.pc` is set to the step after the alarm label. `is_running` remains true.

7. **Execute alarm program.** `run_loop` continues executing the alarm program. The alarm program operates with whatever X/Y/Z/T/LASTX and ALPHA register state was present in the interrupted program — these are NOT automatically saved/restored by the hardware. Well-behaved alarm programs save and restore the data stack themselves using STOST/RCLST idioms if needed.

8. **Alarm program ends (RTN or END).** When the alarm program returns (RTN with empty local call stack, or program end), `call_stack.pop()` restores the interrupted program's `pc`. Execution of the original program resumes from where it was interrupted.

9. **Alarm acknowledgment.** Control alarms are self-acknowledging — the alarm is cleared (or rescheduled if repeating) automatically when it fires, without requiring a keypress from the user. (Manual p.111: control alarms "simply execute the program... and then automatically clear themselves from memory.")

10. **Repeat-interval rescheduling.** If the alarm has a repeat interval, `trigger_unix += repeat_secs` and `past_due = false` before the alarm program executes. (Reschedule uses original alarm time + repeat interval, not acknowledgment time — Manual p.113.)

### Conditional Alarm (`>>label`, `interrupting: true` in codebase) — Fires While Program is Running

1. **Alarm comes due** while `is_running == true`.
2. **No interrupt.** The conditional alarm DOES NOT interrupt the running program.
3. **Sounds tones only.** The alarm sounds "a pair of tones" and becomes past-due.
4. **Becomes past-due.** `past_due = true`. No program execution occurs.
5. **No acknowledgment cycle.** The alarm remains in the catalog as a past-due alarm.
6. **Executed later.** When the running program completes and the calculator is idle (keyboard mode) or when the calculator is turned off, the past-due conditional alarm activates its program. (Manual p.219-220 area.)

### Summary of Key Difference

| Situation | Control `>label` | Conditional `>>label` |
|-----------|-----------------|----------------------|
| Calculator running a program | Interrupts — XEQ label as subroutine | Deferred — becomes past-due, sounds tone pair |
| Calculator idle (keyboard) | Runs label immediately | Runs label immediately |
| Calculator off | Runs label on turn-on | Runs label on turn-on (deferred) |
| Calculator in ALPHA mode | Behavior uncertain — see Open Questions | Same deferral as running? |
| Calculator showing clock | Runs label immediately | Runs label immediately |

---

## Edge-Case Table

| Situation | Hardware Behavior (OM-sourced) | Emulator Implication | Confidence |
|-----------|-------------------------------|----------------------|------------|
| Control alarm fires, calculator OFF | Calculator powers on, executes alarm program | out of scope for emulator (no off state) | HIGH |
| Control alarm fires, calculator idle | Executes alarm program immediately | `check_alarms` at interaction boundary already handles this via `event_buffer` | HIGH |
| Control alarm fires, program running at call depth 0-3 | Interrupts at instruction boundary; pushes one return level | Must check alarms in run_loop between instructions | MEDIUM |
| Control alarm fires, program running at depth 4 (full) | "Use too many subroutine levels" — original program NOT resumed | Uncertain: alarm may still execute (depth 5 = no resume), or may skip | LOW |
| Multiple control alarms fire simultaneously | Activate in order set (chronologically). "Successive alarms will interrupt programs triggered by preceding alarms" — nested interruption chain | Process in chronological order; each consumes one stack level | MEDIUM |
| Conditional alarm fires, program running | Sounds tone pair, becomes past-due, no program execution | Existing `alarm:interrupting:deferred` event is correct behavior | HIGH |
| Conditional alarm fires, program running during a control alarm's program | Sounds tones, becomes past-due only | Same deferral | MEDIUM |
| Message alarm fires, program running | "Temporarily suspends the program executed by a previous control alarm" — displays message, flashes, waits for acknowledgment | Currently handled as `alarm:message:{text}` event; resume after acknowledgment needs verification | MEDIUM |
| Alarm fires during PSE instruction | Behavior uncertain — PSE posts `PAUSE 1000` event and continues; alarm check would occur between instructions, not within PSE | Check alarms after PSE posts event but before resuming | LOW |
| Alarm fires while in ALPHA mode | Behavior uncertain — ALPHA mode is keyboard-level state, not program-level; hardware likely fires alarm at instruction boundary regardless | Treat same as keyboard mode (fire immediately) | LOW |
| Alarm fires during CATALOG | CATALOG is a running program; control alarm fires as subroutine | CATALOG has special keyboard mode; alarm behavior uncertain during catalog | LOW |
| Alarm fires while alarm catalog (ALMCAT) is active | Likely deferred until ALMCAT exits | ALMCAT sets `alarm_catalog_mode`; check interaction with `check_alarms` | LOW |
| Alarm label not found in program memory | No OM documentation found — likely `NO SUCH LBL` or similar error | Return `HpError::InvalidOp`, alarm becomes past-due rather than clearing | LOW |
| Past-due control alarm activating after calculator turn-on | Fires in chronological order; "all bypassed past-due alarms activate ahead of new alarms" | Priority queue: past-due drain before newly-due alarm processing | MEDIUM |
| Control alarm program itself sets another alarm | Legal — alarm programs can call XYZALM | No special constraint needed; alarm catalog write during execution is normal | HIGH |
| Control alarm program calls XEQ (nested subroutine) | Legal — consumes additional call-stack levels from the alarm's level | Each XEQ inside alarm program consumes one of remaining levels | HIGH |
| Control alarm program executes RTN at wrong depth | Returns to original program's interrupted point if RTN pops correct return address | Standard call_stack.pop behavior applies | HIGH |
| Control alarm program turns calculator OFF | "Program ABC will not be resumed" (Manual p.222) | Not applicable to emulator (no power state); after alarm RTN, resume interrupted program normally unless alarm program calls some halt | LOW |
| Repeating control alarm fires, rescheduling | Next trigger = original alarm time + repeat interval (NOT acknowledgment time) | `trigger_unix += repeat_secs` before program executes; alarm clears `past_due` | HIGH |
| Repeating control alarm with interval < 10 seconds | "Can be difficult to cancel" (Manual p.108) | Emulator has no such restriction but should document | MEDIUM |
| Control alarm fires during another control alarm's program | "Successive alarms will interrupt the programs triggered by preceding alarms" (Manual p.113) | Nested interruptions legal up to stack depth limit | MEDIUM |
| Message alarm fires during a control alarm's program | "Will temporarily suspend the program" (Manual p.113) | Message alarm handling already posts event to buffer; program resumes after ACK | MEDIUM |
| Alarm fires while modal prompt is active (Math Pac) | Behavior uncertain — modal_program state not documented in OM alarm section | Likely: alarm fires after modal completes (instruction boundary) | LOW |

---

## Execution State to Save/Restore

**Hardware does NOT automatically save/restore the data stack or ALPHA register when a control alarm fires.** The alarm program executes in the same register environment as the interrupted program. This is the HP-41 design: alarm programs that need to preserve caller state must do so explicitly (STOST / RCLST idiom for data stack, ASTO / ARCL for ALPHA).

**What the hardware preserves automatically:**
- `pc` — the interrupted program's next instruction address (pushed onto call stack as the return address)
- `call_stack` — the alarm's return address is pushed, enabling RTN to return to the interrupted program
- `is_running` — remains true throughout alarm execution

**What the hardware does NOT preserve automatically (alarm program can corrupt):**
- X, Y, Z, T registers (4-level data stack)
- LASTX register
- ALPHA register
- Flag states (flags 0-55)
- Display mode / angle mode
- Any open modal state (not a hardware concept)

**Emulator implication:** The emulator should faithfully match this behavior — do NOT automatically save and restore the data stack around alarm program execution. Doing so would add hardware-unfaithful safety. Alarm programs that need stack preservation should use STOST/RCLST explicitly (or the emulator could document that this is a known source of alarm program bugs).

**What must be saved (by the emulator, not the hardware) to implement the interrupt mechanism:**
- `pc` — push current pc onto `call_stack` before jumping to alarm label
- That is all. The rest of `CalcState` is shared with the alarm program by design.

---

## Display & Annunciator Behavior

Based on OM evidence (MEDIUM confidence — OCR quality is imperfect):

### Control Alarm (`>label`) Firing

1. **Instruction boundary.** No display interruption mid-instruction.
2. **Alarm program executes.** The display shows whatever the alarm program produces — the alarm program drives the display while running (same as any running program).
3. **PRGM annunciator behavior.** `is_running` stays true throughout; PRGM annunciator behavior is unchanged.
4. **No separate "alarm notification" display.** Control alarms are self-acknowledging and execute their program directly without displaying an alarm notification banner. (The program itself may AVIEW or PROMPT as desired.)
5. **On completion.** Display returns to normal program state or idle as appropriate.

### Message Alarm Firing (for comparison, to disambiguate from control alarm)

1. Sounds a pair of tones.
2. Displays first 12 characters of alarm text (or time and date if no Alpha text).
3. Display flashes for approximately 5 cycles.
4. If not acknowledged, sounds up to 16 more tone pairs.
5. If still not acknowledged, alarm becomes past-due.
6. If acknowledged (any key), alarm clears (or reschedules if repeating).
7. A suspended program resumes after acknowledgment. (Manual p.107: "If a program was interrupted for an alarm, the interrupted program now resumes.")

### Conditional Alarm (`>>label`) Firing During a Running Program

1. Sounds a pair of tones.
2. Becomes past-due. No display change for the running program.
3. Running program continues uninterrupted.

### Beep on Alarm

All alarm types sound tones on activation. The emulator currently produces no audio for alarm events. This is a pre-existing behavior (audio is a frontend concern, handled via sound event buffer). Alarm events should push a sound event alongside the alarm event to match hardware.

---

## Acceptance-Criteria-Ready Assertions

The following assertions are directly testable. They use hardware-documented behavior for HIGH/MEDIUM items and flag LOW-confidence items.

**Category: Control Alarm (`>label`) Firing During Idle State**
- When a control alarm fires while `is_running == false` and the calculator is at the prompt, the alarm program executes immediately (same as an XEQ from the keyboard). [HIGH]
- After a control alarm program completes at idle, the calculator returns to idle state (prompt). [HIGH]
- A one-shot control alarm is removed from the alarm catalog after its program executes. [HIGH]
- A repeating control alarm reschedules to `trigger_unix += repeat_secs` and sets `past_due = false` after its program executes. The rescheduling uses the original alarm time, not the execution completion time. [HIGH]

**Category: Control Alarm Firing During Running Program**
- When a control alarm fires while `is_running == true` and `call_stack.len() < 4`, the alarm program is invoked as a subroutine: the current `pc` is pushed onto `call_stack`, and `pc` jumps to the alarm label's position. [MEDIUM]
- After the alarm program executes RTN (or reaches end), `call_stack.pop()` restores the interrupted program's `pc` and execution continues. [MEDIUM]
- The data stack (X/Y/Z/T) and LASTX register are NOT automatically saved or restored by the alarm mechanism. The alarm program executes in the same register environment as the interrupted program. [HIGH]
- A control alarm that fires at `call_stack.len() == 4` does not allow the original program to resume after the alarm (stack exhausted). [LOW — exact behavior uncertain]
- The alarm check occurs at instruction boundaries, not mid-instruction. [HIGH]

**Category: Conditional Alarm (`>>label`) Firing During Running Program**
- When a conditional alarm fires while `is_running == true`, the alarm does NOT execute its program. [HIGH]
- A conditional alarm that fires during a running program sounds a tone pair and becomes past-due (`past_due = true`). [MEDIUM]
- A conditional alarm that fires during a running program does NOT interrupt, halt, or modify the interrupted program's state. [HIGH]

**Category: Multiple Simultaneous Alarms**
- When two or more alarms come due at the same time, they activate in chronological order (earliest trigger_unix first, then order-of-insertion as tiebreaker). [MEDIUM]
- A control alarm may interrupt the program started by a preceding control alarm (nested subroutine calls), subject to the 4-level call-stack limit. [MEDIUM]
- A conditional alarm that fires while a control alarm's program is running sounds tones and becomes past-due only — it does not further interrupt. [MEDIUM]
- A message alarm that fires while a control alarm's program is running temporarily suspends the alarm program until the message is acknowledged. [LOW — uncertain emulator applicability]

**Category: Past-Due Alarms**
- Past-due alarms accumulate in memory with `past_due = true` and are not re-fired by `check_alarms` (guarded by the `!past_due` check in the drain loop). [HIGH — already implemented]
- When the calculator becomes idle after past-due alarms exist, past-due control and conditional alarms activate before any newly-due alarm. [MEDIUM]
- `ALMNOW` activates "the oldest past-due program or function alarm in memory" — the earliest chronological past-due control/conditional alarm. [HIGH — already implemented]

**Category: Non-Interrupting (`>label`) vs Interrupting (`>>label`) Code Path Separation**
- The implementation must check the `interrupting` field and route `Control { label, interrupting: false }` through the new in-loop interrupt path, and route `Control { label, interrupting: true }` through the existing deferred path (pending ALMNOW or user-interaction boundary). [HIGH — based on OM alarm type definitions]
- NOTE: The field `interrupting: true` in the codebase corresponds to the OM "conditional" alarm (`>>label`), which is the DEFERRED (non-interrupting) variant. This is a naming inversion that must be carefully handled. [HIGH — confirmed by OM taxonomy]

**Category: Alarm Acknowledgment**
- Control alarms are self-acknowledging: they do not require a user keypress to clear. [HIGH]
- Message alarms require a user keypress to acknowledge. [HIGH — already implemented]
- Repeating alarms reset using the original alarm time plus the repeat interval, not the time of acknowledgment. [HIGH — already implemented in `acknowledge_alarm`]

---

## Open Questions / Uncertain Behaviors

The following behaviors could not be confirmed from available sources and are explicitly flagged as requiring further investigation or safe-default decisions.

1. **Call stack full when control alarm fires (`call_stack.len() == 4`)**: The OM states the interrupted program is not resumed if "too many subroutine levels" are used. What happens at the moment the alarm fires at depth 4? Options: (a) The alarm program still executes but RTN exits the entire `run_loop` (no resume); (b) The alarm is suppressed/deferred; (c) The alarm fires but the running program is terminated first. No direct OM citation found. **Recommended default: suppress alarm execution when `call_stack.len() >= 4`, leave alarm as past-due.** This is conservative and avoids crashing the interrupted program.

2. **Alarm label not found in program memory**: If the alarm's label does not exist in `program`, what happens? Hardware likely displays `NO SUCH LBL` or similar error, and the alarm may remain past-due. **Recommended default: alarm becomes past-due, `event_buffer` receives an error event, no crash.**

3. **ALPHA mode behavior**: Does a control alarm firing while the user is in ALPHA entry mode interrupt the ALPHA entry? On hardware, ALPHA entry is a keyboard-level state. Likely the alarm fires at the next instruction boundary in a running program (ALPHA mode within a program = ALPHA register manipulation, not keyboard ALPHA mode). **Recommended default: treat the same as any running state.**

4. **Alarm check cadence in `run_loop`**: Hardware checks alarms continuously via hardware timer. Emulator must check between instructions. The question is: check every instruction (overhead), every N instructions, or only at PSE/STOP/R/S boundaries? **Recommended default: check every instruction for correct boundary semantics. Performance overhead is negligible for 1M-step budget.**

5. **Modal program interaction**: If a `modal_program` is active (Math Pac solver, etc.) when an alarm fires, does the alarm interrupt mid-modal-workflow? The Math Pac solver uses `run_loop` re-entrancy; an alarm interrupt mid-solver could corrupt solver state. **Recommended default: suppress alarm execution while `modal_program.is_some()`, defer to next idle boundary.**

6. **`is_running` guard**: The current code has a `is_running` flag set to true during `run_program`. If `check_alarms` is called from inside `run_loop`, a nested `run_program` call for the alarm would see `is_running == true` and the alarm would fail the re-entrancy guard. **The alarm interrupt must be implemented as in-loop execution (inline call stack manipulation in `run_loop`), NOT as a nested `run_program` call.** This is the architectural key insight.

7. **Annunciator/display during alarm program**: Does the PRGM or BUSY annunciator change state during alarm program execution? Not documented. Likely no change (still shows running state). **Recommended default: no annunciator state change.**

8. **PSE behavior**: The current `Op::Pse` posts a `PAUSE 1000` event to `event_buffer` and continues. If an alarm fires during PSE pause (a 1-second front-end pause), does hardware fire the alarm during the pause? The pause is a display delay, not a program halt. **Recommended default: check alarms immediately after PSE event is posted, before the next instruction in `run_loop`.**

9. **Exact "two up-arrows" vs "one up-arrow" encoding**: The QREF says `>>label` = "Interrupting Control Alarm" and `>label` = "Noninterrupting Control Alarm." However, the ManualsLib manual (page 110 OCR) says a single-arrow alarm "temporarily interrupt[s] a running program, if necessary" while a double-arrow alarm fires only "if the HP-41 is off or displaying the clock." This maps `>` = interrupting (OM "control") and `>>` = conditional (OM "conditional"). The QREF's own label of "Interrupting" and "Noninterrupting" is the REVERSE of the underlying behavior. The project `interrupting: bool` field stores `true` for `>>label`, which is the deferred/conditional type. **Resolution: use OM Vol.2 behavior over QREF label names. `>label` = fire during running program; `>>label` = defer to idle/off.**

---

## Sources

| Source | What It Provided | Confidence |
|--------|-----------------|------------|
| HP-41CX Owner's Manual Vol. 2 via ManualsLib (manual/3559140), pages 105–117, 218–222 | Alarm type taxonomy, activation sequence, subroutine-level interaction, simultaneous alarm ordering, acknowledgment model, past-due behavior | MEDIUM (OCR quality variable; page-numbering offset between manual pages and ManualsLib URL pages) |
| HP 82182A Time Module QREF (qrg41.fjk.ch/hp82182a.html) | Confirmed `>>label` = "Interrupting Control Alarm", `>label` = "Noninterrupting Control Alarm"; ALMNOW activates "oldest past-due program or function alarm" | HIGH |
| HP-41CX Owner's Manual Vol. 2, p.113 (ManualsLib) — Simultaneous Alarms section | "a control alarm will interrupt a program triggered by a control or conditional alarm"; conditional alarm "neither interrupt a program nor wait for completion; it simply sounds a pair of tones and becomes past due" | HIGH |
| HP-41CX Owner's Manual Vol. 2, p.220 (ManualsLib) — Past-Due Alarm Responses | Control alarm during running program: runs "as a subroutine of current program"; conditional alarm fires only when off or showing clock | HIGH |
| HP-41CX Owner's Manual Vol. 2, p.222 (ManualsLib) — Interruption by Another Alarm | "Control then returns to program ABC assuming that program XYZ did not stop or turn off the computer or use too many subroutine levels" | HIGH |
| HP-41CX Owner's Manual Vol. 2, p.111 (ManualsLib) — Activation of Message Alarms | Control alarms are "self-acknowledging" and "automatically clear themselves from memory" | HIGH |
| HP-41CX Owner's Manual Vol. 2, p.108 (ManualsLib) | Alarm can be "type message, control, or conditional" | HIGH |
| docs/hp41-time-divergences.md §D-40-04 | Project's own description of the deferred implementation | HIGH |
| hp41-core/src/ops/time/alarm.rs | Current implementation: `AlarmType::Control { label, interrupting }` with `interrupting: true` for `>>label` | HIGH (source code) |
| hp41-core/src/ops/program.rs | `run_loop` architecture: 4-level call stack (`call_stack.len() >= 4` guard), `is_running` flag, `MAX_STEPS` | HIGH (source code) |
| Community sources (wilsonminesco.com, hpmuseum.org forum) | Self-acknowledging nature of control alarms; flag 25 for error-ignore; resume-after-interrupt concept | LOW (practical examples, not formal documentation) |
