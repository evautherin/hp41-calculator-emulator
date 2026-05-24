# Feature Landscape: HP-41CX Time Module Emulation (v3.2)

**Domain:** Behavioral emulation of the HP-41CX Time Module (built-in CX ROM, also sold standalone as HP 82182A)
**Researched:** 2026-05-24
**Scope:** v3.2 -- Time Module (XROM 26) as the third XROM application module. Advantage Pac deferred to v3.3+.

**Primary sources:**
- HP 82182A Time Module Quick Reference Card (HP 82182-90002, November 1981) -- read directly as 2-page image. All functions, alarm catalog keyboard, stopwatch keyboard, alarm format, XYZALM stack parameters confirmed from the QRC images.
- HP-41CX Quick Reference Guide (HP 00041-90475, August 1983) -- read as 40-page PDF. Complete function set, keyboard layouts (Normal/User/Alpha/Alarm Catalog/Stopwatch/Text Editor), Time and Alarm Formats section, Flags and Their Status table, List of Errors all extracted.
- HP Museum XROM database (hpmuseum.org/software/xroms.htm) -- XROM 26 function IDs confirmed.
- HP-41 QREF (qrg41.fjk.ch/hp82182a.html) -- function listing cross-checked.
- HP-41C Wikipedia article -- alarm system behavior and CX integration details.

**Confidence:** HIGH for function list and formats (primary source images read directly). MEDIUM for some behavioral edge cases (alarm interrupt model, stopwatch real-time LCD update frequency) where OM text was not fully accessible.

---

## Authoritative Function List (XROM 26)

The HP 82182A Time Module (standalone) defines 30 functions (XROM 26,0 through 26,29). The HP-41CX built-in variant adds 6 more (26,30 through 26,35). Our emulation targets the **CX superset** per the PROJECT.md scope ("HP-41CX Time Module OM 00041-90035").

### HP 82182A Base Functions (30)

| XROM ID | Mnemonic | Category | Description |
|---------|----------|----------|-------------|
| 26,0 | -TIME-C | Header | Module header (not callable) |
| 26,1 | ADATE | ALPHA Date/Time | Append date from X-reg to ALPHA in current date format |
| 26,2 | ALMCAT | Alarm | Alarm catalog -- lists pending/past-due alarms with keyboard redefinition |
| 26,3 | ALMNOW | Alarm | Activate oldest past-due program or function alarm in memory |
| 26,4 | ATIME | ALPHA Date/Time | Append time from X-reg to ALPHA in CLK12 or CLK24 format |
| 26,5 | ATIME24 | ALPHA Date/Time | Same as ATIME but always uses CLK24 format |
| 26,6 | CLK12 | Clock Display | Switch to 12-hour time display format |
| 26,7 | CLK24 | Clock Display | Switch to 24-hour time display format |
| 26,8 | CLKT | Clock Display | Switch clock to time-only display format |
| 26,9 | CLKTD | Clock Display | Switch clock to time-and-date display format |
| 26,10 | CLOCK | Clock Display | Display the clock (also accessible via SHIFT+ON) |
| 26,11 | CORRECT | Clock | Same as SETIME + automatically adjusts accuracy factor |
| 26,12 | DATE | Date Arithmetic | Recall current date to X-register; from keyboard also displays day name |
| 26,13 | DATE+ | Date Arithmetic | Calculate new date from Y-reg date + X-reg days |
| 26,14 | DDAYS | Date Arithmetic | Calculate days between two dates (X-reg and Y-reg) |
| 26,15 | DMY | Date Format | Set day-month-year input/output format; sets flag 31 |
| 26,16 | DOW | Date Arithmetic | Replace X-reg date with day-of-week number (0=Sun..6=Sat); keyboard shows day name |
| 26,17 | MDY | Date Format | Set month-day-year input/output format; clears flag 31 |
| 26,18 | RCLAF | Clock | Recall clock accuracy factor to X-register |
| 26,19 | RCLSW | Stopwatch | Recall current stopwatch time to X-register |
| 26,20 | RUNSW | Stopwatch | Start the stopwatch |
| 26,21 | SETAF | Clock | Set clock accuracy factor from X-register (-99.9 to 99.9) |
| 26,22 | SETDATE | Clock | Set clock date from X-register |
| 26,23 | SETIME | Clock | Set clock time from X-register |
| 26,24 | SETSW | Stopwatch | Set stopwatch starting time from X-register (-99.595999 to 99.595999) |
| 26,25 | STOPSW | Stopwatch | Halt the stopwatch |
| 26,26 | SW | Stopwatch | Enter Stopwatch mode (reassigns keyboard) |
| 26,27 | T+X | Clock | Adjust clock time by X-register value (+/-HHHH.MMSSss); date changes if crossing midnight |
| 26,28 | TIME | Clock | Recall current time to X-register (24-hour format); from keyboard also displays time |
| 26,29 | XYZALM | Alarm | Set alarm using stack (X=time, Y=date, Z=repeat) and ALPHA register |

### CX-Only Additional Functions (6)

| XROM ID | Mnemonic | Category | Description |
|---------|----------|----------|-------------|
| 26,30 | (reserved) | -- | (gap in numbering) |
| 26,31 | CLALMA | Alarm | Clear alarm whose message matches ALPHA register |
| 26,32 | CLALMX | Alarm | Clear nth alarm (X-register specifies alarm number) |
| 26,33 | CLRALMS | Alarm | Clear ALL alarms |
| 26,34 | RCLALM | Alarm | Recall parameters of alarm n (to stack and ALPHA) |
| 26,35 | SWPT | Stopwatch | Stopwatch and pointers -- activate stopwatch with storage/recall pointer setup |

### Total: 35 callable functions (excluding header)

---

## Functional Categories

### Category 1: Clock / Time Recall (6 functions)

**Functions:** TIME, SETIME, CORRECT, T+X, RCLAF, SETAF

**TIME behavior:**
- Returns current time to X-register in HH.MMSSss format (24-hour, always).
- When executed from keyboard (not in program), also displays the time on the LCD.
- LiftEffect: Enable.

**SETIME behavior:**
- Sets the clock time from X-register value.
- Format: HH.MMSSss where HH = hours, MM = minutes, SS = seconds, ss = hundredths.
- Valid ranges per QRC:
  - 0.000000 through 11.595999 = A.M.
  - 12.000000 through 23.595999 = P.M.
  - -1.000000 through -11.595999 = P.M. (negative shorthand)
- Invalid time values produce DATA ERROR.

**CORRECT behavior:**
- Performs SETIME plus automatically adjusts the accuracy factor.
- The adjustment compensates for observed clock drift.

**T+X behavior:**
- Adjusts the system clock by the value in X-register.
- Format: +/-HHHH.MMSSss (hours can exceed 24).
- If the adjustment crosses a midnight boundary, the date also changes.
- This is the only time function that can modify the date as a side effect.

**SETAF / RCLAF behavior:**
- Accuracy factor: interval (in seconds) at which one pulse (~9.8e-5 seconds) is added to or subtracted from the clock's 10240 Hz time base.
- Range: -99.9 to +99.9. Value of 0.0 = no correction (default).
- Negative = subtract a pulse (slow clock down); positive = add a pulse (speed up).
- **Emulator divergence:** Since we use the host system clock, the accuracy factor has no physical meaning. We store it but it has no effect on timekeeping.

**Complexity:** Medium. Core time recall/set is straightforward with `std::time::SystemTime` or `chrono`. The accuracy factor is store-only (no-op for emulation). T+X crossing midnight needs date rollover logic.

---

### Category 2: Date Recall and Arithmetic (5 functions)

**Functions:** DATE, DATE+, DDAYS, DOW, SETDATE

**Date format convention:**
- Two formats controlled by flag 31:
  - **MDY** (flag 31 clear): MM.DDYYYY (e.g., 3.282006 = March 28, 2006)
  - **DMY** (flag 31 set): DD.MMYYYY (e.g., 28.032006 = March 28, 2006)
- Input must be a positive number. All trailing digits after the year must be zero; otherwise DATA ERROR.
- The QRC states dates use 4-digit years (YYYY, not YY).

**DATE behavior:**
- Returns current date to X-register in the active format (MM.DDYYYY or DD.MMYYYY).
- When executed from keyboard, also displays "MM/DD/YY day" or "DD.MM.YY day" on the LCD.
- LiftEffect: Enable.

**DATE+ behavior:**
- Y-register = base date; X-register = number of days to add (can be negative).
- Result = new date in the active format, placed in X-register.
- LiftEffect: Enable.

**DDAYS behavior:**
- Calculates days between date in Y-register and date in X-register.
- Result = signed integer (X - Y); positive if X is later than Y.
- Both dates must be in the active format.
- LiftEffect: Enable.

**DOW behavior:**
- Replaces X-register date with day-of-week number: 0 = Sunday, 1 = Monday, ..., 6 = Saturday.
- When executed from keyboard, also displays the day name (e.g., "SUNDAY").
- LiftEffect: Enable.

**SETDATE behavior:**
- Sets clock date from X-register in the active format.
- Validates the date; invalid dates produce DATA ERROR.

**Complexity:** Medium. Date arithmetic requires a robust calendar engine. The HP-41CX supports the Gregorian calendar. We can use Rust's `chrono` crate or implement the Julian Day Number algorithm directly.

---

### Category 3: Date/Time Format Toggles (4 functions)

**Functions:** DMY, MDY, CLK12, CLK24

**DMY / MDY:**
- DMY sets flag 31 (day-month-year format).
- MDY clears flag 31 (month-day-year format).
- These affect all date input/output (DATE, DATE+, DDAYS, SETDATE, ADATE).
- LiftEffect: Neutral.

**CLK12 / CLK24:**
- CLK12 switches to 12-hour time display format.
- CLK24 switches to 24-hour time display format.
- These affect the clock display (CLOCK) and ATIME output.
- Note: TIME always returns 24-hour format to X-register regardless of this setting.
- **State:** The 12/24 setting is a persistent display preference. On real hardware it is stored in the time module's own registers. For emulation, this is a new `CalcState` field.
- LiftEffect: Neutral.

**Complexity:** Low. Flag 31 already exists in the flag system. CLK12/CLK24 needs one new bool field on CalcState.

---

### Category 4: Clock Display Modes (2+1 functions)

**Functions:** CLKT, CLKTD, CLOCK (or SHIFT+ON)

**CLKT / CLKTD:**
- CLKT = time-only clock display format.
- CLKTD = time-and-date clock display format.
- These set the format used when CLOCK is activated.
- LiftEffect: Neutral.

**CLOCK (or SHIFT+ON):**
- Activates the clock display on the LCD.
- The display continuously updates in real time.
- In CLKT mode: shows `HH:MM:SS` (or `HH:MM:SS AM/PM` in CLK12).
- In CLKTD mode: shows `HH:MM AM MM/DD` or `HH:MM PM DD.MM` (abbreviated date).
- While the clock is displayed, pressing R/S enters the Alarm Catalog.
- Any other key exits clock display mode.
- **This is the first real-time continuous display update in the emulator.** Unlike all other operations which are event-driven (key press -> display update), the clock display must tick every second.

**Complexity:** HIGH. This introduces a fundamentally new display paradigm:
1. The emulator must transition from pure event-driven to a hybrid event-driven + periodic-update model.
2. CLI (ratatui) needs a periodic redraw timer when clock is active.
3. GUI (Tauri/React) needs a similar periodic state-poll or push mechanism.
4. The clock display must coexist with normal calculator operation (entering clock mode, exiting on any keypress).

---

### Category 5: ALPHA Date/Time Functions (3 functions)

**Functions:** ADATE, ATIME, ATIME24

**ADATE behavior:**
- Appends the date from X-register to the ALPHA register in the active date format.
- The number of digits varies according to the display setting (FIX/SCI/ENG affects trailing zeros).
- Example: With MDY and FIX 6, X=3.282006 appends "3/28/2006" to ALPHA.

**ATIME behavior:**
- Appends the time from X-register to the ALPHA register in the current CLK12/CLK24 format.
- Truncated according to the display digits setting.
- Example: With CLK24 and FIX 4, X=14.3025 appends "14:30:25" to ALPHA.

**ATIME24 behavior:**
- Same as ATIME but always uses 24-hour format regardless of CLK12/CLK24 setting.

**Complexity:** Medium. Format conversion (numeric -> string with separators) is the main work. Reuses the existing ALPHA register append infrastructure. The display-digit-dependent truncation needs careful implementation.

---

### Category 6: Alarm System (8 functions)

**Functions:** XYZALM, ALMCAT, ALMNOW, RCLALM, CLALMA, CLALMX, CLRALMS (+ alarm acknowledge mechanism)

This is the most complex subsystem in the Time Module.

#### 6a. Setting Alarms: XYZALM

**Stack parameters (from QRC):**

| Register | Content | Format |
|----------|---------|--------|
| T | (unused) | |
| Z | Repeat interval | HHHH.MMSSs or 0 |
| Y | Date | MM.DDYYYY or DD.MMYYYY or 0 |
| X | Time | HH.MMSSs |

- Z = 0 means no repeat (one-shot alarm).
- Y = 0 means "today" (current date).

**ALPHA register determines alarm type:**

| ALPHA content | Alarm type |
|---------------|-----------|
| Empty or message text | **Message Alarm** -- sounds tones and displays the message |
| `>>global label` or `>>function name` | **Interrupting Control Alarm** -- runs the specified program/function; interrupts current activity |
| `>global label` or `>function name` | **Non-interrupting (Conditional) Control Alarm** -- conditional execution (see below) |

Note: `>>` = two shift-right-arrow characters; `>` = one shift-right-arrow character. A "function" must be a programmable function belonging to a plug-in device.

**Conditional Alarm behavior (complex):**
- Does NOT interrupt a running program (unlike other alarm types).
- If HP-41CX is off or displaying the clock -> becomes a control alarm (runs the program).
- If HP-41CX is on and NOT running a program -> becomes a message alarm.
- If a program IS running -> alarm beeps twice and becomes past-due.

**Alarm storage:**
- Up to 253 alarms depending on available memory (stored in uncommitted registers R100-R318, same memory pool as program lines and key assignments).
- Each alarm consumes memory registers.
- Alarms are ordered chronologically.

#### 6b. Alarm Catalog: ALMCAT

- Lists all pending and past-due alarms in chronological order.
- Pressing R/S during ALMCAT halts the listing and redefines the keyboard to the **Alarm Catalog Keyboard** (see below).
- The alarm catalog display shows: `HH:MM AM MM/DD` or `HH:MM PM DD.MM` (time and date of each alarm).

**Alarm Catalog Keyboard (from QRC):**

| Key | Function |
|-----|----------|
| SHIFT+C | Delete alarm |
| D | Alarm Date |
| T | Alarm Time |
| M | Alarm Message, Label, or Function |
| R | Alarm Repeat Interval |
| SHIFT+R | Reset Alarm Using Specified Repeat Interval |
| SHIFT+T | Current Time |
| SST | Next Alarm and Message, Label, or Function |
| BST | Preceding alarm and Message, Label, or Function |
| Back-arrow | Exit Alarm Catalog Mode |
| R/S | Resume ALMCAT Listing |

This is a full keyboard-mode reassignment, similar to the Stopwatch keyboard.

#### 6c. Alarm Activation: ALMNOW

- Activates the oldest past-due program or function alarm.
- If no past-due alarms exist, no action.
- Useful for program-driven alarm processing.

#### 6d. Recall/Clear: RCLALM, CLALMA, CLALMX, CLRALMS

**RCLALM n:**
- Recalls parameters of alarm n to the stack and ALPHA register.
- Stack layout mirrors XYZALM (X=time, Y=date, Z=repeat).
- ALPHA gets the message/label.

**CLALMA:**
- Clears the alarm whose message matches the current ALPHA register content.

**CLALMX:**
- Clears the nth alarm (X-register specifies the alarm number).

**CLRALMS:**
- Clears ALL alarms.

#### 6e. Alarm Acknowledge Mechanism

**Message Alarms (from QRC):**
- To halt a current flashing alarm: press any key except STO. This clears (deletes) the non-repeating alarm, or resets a repeating alarm.
- To halt AND clear a current repeating alarm: press SHIFT+C.
- To clear a non-active alarm: use SHIFT+C on the Alarm Catalog keyboard.
- Non-message alarms (control/conditional) are not acknowledged -- they run programs.

**Complexity:** VERY HIGH. The alarm system is the single most complex feature:
1. **New data structure:** Alarm catalog stored in memory, up to 253 entries with time/date/repeat/type/message fields.
2. **New CalcState fields:** Alarm list, active alarm state, alarm catalog mode.
3. **Interrupt model:** Alarms fire whether or not the calculator is on (in emulation: alarms fire based on system time comparison, need a background timer).
4. **Keyboard mode switching:** Alarm Catalog is a full keyboard reassignment (third mode after Stopwatch).
5. **Program execution trigger:** Control alarms auto-execute programs -- requires integration with `run_program()` infrastructure.
6. **Repeating alarms:** Need interval-based rescheduling logic.
7. **Past-due tracking:** Alarms that fired while "off" accumulate as past-due.
8. **Memory management:** Alarms share the uncommitted register pool.
9. **Persistence:** Alarms must survive save/load (not transient).

---

### Category 7: Stopwatch (6+1 functions)

**Functions:** SW, RUNSW, STOPSW, SETSW, RCLSW, SWPT (CX-only)

#### 7a. Programmatic Stopwatch Functions (outside SW mode)

These four functions work when the calculator is NOT in Stopwatch mode:

**RCLSW:** Recalls current stopwatch time to X-register (HH.MMSSss format). Works whether stopwatch is running or stopped.

**RUNSW:** Starts the stopwatch. Can be used in programs to time code execution.

**SETSW:** Sets stopwatch starting time from X-register. Range: -99.595999 to +99.595999.

**STOPSW:** Halts the stopwatch.

#### 7b. Stopwatch Mode: SW

**SW** switches the calculator to Stopwatch mode and reassigns the keyboard:

**Stopwatch Keyboard (from QRC):**

| Key | Function |
|-----|----------|
| SPLIT | Take Split (record current time to a register) |
| Delta-SPLIT | Set/Clear Delta Split Mode (difference between consecutive splits) |
| Digit keys | Set new register address for split storage |
| Register Address display | Shows current register pointer (Rnn or Dnn) |
| SST, BST | Increment/Decrement Register Address |
| PRGM | Set/Cancel Display of Three-Digit Address |
| EXIT | Exit Stopwatch |
| CLEAR | Clear Time to Zero |
| REG# | Suppress/Restore Display of Register Address |
| R/S | Run/Stop Stopwatch |
| EEX | Three-Digit Pointer On/Off |
| CHS | Split Difference On/Off |
| RCL | Split Recall On/Off |
| SHIFT+EEX | Register Pointer On/Off |

**Display Symbols:**
- `1- R` = Store split
- `1- D` = Store split; display difference
- `2- R` = Recall split
- `2- D` = Recall split difference

**Real-time display:** While the stopwatch is running, the LCD shows the elapsed time updating continuously (similar to CLOCK mode but with stopwatch-specific formatting: `HH:MM:SS.hh`).

**SWPT (CX-only):** Given sss.rrr in X-register, activates the Stopwatch keyboard and sets the storage pointer (sss) and recall pointer (rrr).

**Complexity:** HIGH.
1. **Real-time display updates** (second instance of continuous LCD refresh, alongside CLOCK).
2. **Full keyboard reassignment** (second special keyboard mode).
3. **Split timing** with register storage and delta calculation.
4. **Background timer** -- stopwatch must continue running even when not in SW mode.
5. **Three display modes** within stopwatch: store-split, recall-split, display-difference.
6. **Negative stopwatch support** (timer can count into negatives per the SETSW range).

---

## Table Stakes

Features users expect from a "Time Module emulation." Missing any of these would make the claim "Time Module emulation" inaccurate.

| Feature | Why Expected | Complexity | Dependency |
|---------|--------------|------------|------------|
| TIME -- recall current time | Core time function; most basic | Low | Host system clock |
| DATE -- recall current date | Core date function; most basic | Low | Host system clock |
| SETIME / SETDATE | Set clock; users expect to set time/date | Medium | Time offset storage |
| DATE+ / DDAYS / DOW | Date arithmetic trio; fundamental utility | Medium | Calendar engine |
| DMY / MDY | Date format toggle; fundamental | Low | Flag 31 (exists) |
| CLK12 / CLK24 | Time format toggle; fundamental | Low | New bool on CalcState |
| ADATE / ATIME / ATIME24 | ALPHA-register formatting; expected for programs | Medium | Existing ALPHA infra |
| CLKT / CLKTD | Clock display format preference | Low | New enum on CalcState |
| CLOCK display | Live clock on LCD; signature feature | HIGH | Periodic display update |
| T+X | Time adjustment; expected for completeness | Medium | Midnight rollover logic |
| CORRECT / SETAF / RCLAF | Accuracy factor; expected for completeness | Low | Store-only (no-op for emulation) |
| XYZALM -- set alarm | Core alarm function; defines the module | HIGH | Full alarm subsystem |
| ALMCAT -- alarm catalog | Browse/manage alarms; companion to XYZALM | HIGH | Alarm catalog keyboard mode |
| ALMNOW -- activate alarm | Program-driven alarm trigger | Medium | Alarm execution infra |
| RCLALM -- recall alarm | Read alarm parameters back | Medium | Alarm data access |
| CLALMA / CLALMX / CLRALMS | Clear alarms; basic management | Medium | Alarm data mutation |
| RUNSW / STOPSW / SETSW / RCLSW | Programmatic stopwatch control | Medium | Background timer |
| SW -- stopwatch mode | Full stopwatch with keyboard | HIGH | SW keyboard + real-time display |
| SWPT | Stopwatch with pointers | Medium | SW mode infra |
| 4-way exhaustive match | All new Op variants in dispatch + execute_op + 2x prgm_display | Low | Existing invariant |
| JSON-canonical help pipeline | `docs/hp41-time-functions.json` + help_data.rs | Medium | Existing JSON infra |
| CATALOG 2 listing | Time Module appears in CATALOG 2 | Low | Existing XROM framework |

---

## Differentiators

Features that would set this emulation apart from typical HP-41 emulators or add genuine user value.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Host-clock integration | Time/Date use real system clock, not fake counter | Low | Natural for a software emulator; real hardware had a crystal oscillator |
| Alarm notification integration | OS-level notifications when alarms fire | HIGH | Desktop notifications via Tauri; CLI could use terminal bell |
| Persistent alarm catalog | Alarms survive application restart | Medium | Serialize alarm list to autosave.json |
| Stopwatch precision | Millisecond or better precision (vs. hardware's centisecond) | Low | Host system clock resolution exceeds original |
| Divergence documentation | Document behavioral differences from hardware | Low | Follows v3.0/v3.1 established pattern |
| Timezone awareness | Display local time correctly | Low | `chrono::Local` handles this automatically |

---

## Anti-Features

Features to explicitly NOT build for v3.2.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Cycle-accurate crystal oscillator simulation | No user value; host clock is better | Use `std::time::SystemTime` or `chrono::Local` |
| Accuracy factor correction loop | The 10240 Hz crystal correction has no meaning with a host OS clock | Store SETAF/RCLAF value but ignore it; document as divergence |
| HP-IL alarm wake-up | The HP-41 could wake from OFF state on alarm; we have no OFF state | Trigger alarms whenever the app is running; document as divergence |
| Battery-low alarm suppression | Real hardware suppressed clock display when battery low (BAT annunciator) | Not applicable; no battery in emulation |
| Extended Memory integration | Text editor, file system, extended memory functions are a separate XROM module | Out of scope for v3.2; these are Extended Functions (XROM 25), not Time Module |
| Shared register pool for alarms | Real hardware stored alarms in uncommitted registers R100-R318 alongside programs | Use a dedicated `Vec<Alarm>` on CalcState instead; simpler and equivalent behavior |
| Physical keyboard overlay | The HP 82182A came with a keyboard overlay label strip | Not applicable to software emulator |

---

## Feature Dependencies

```
                   Flag 31 (already exists)
                        |
                   DMY / MDY
                        |
          +-------------+-------------+
          |             |             |
        DATE         ADATE        SETDATE
          |             |
       DATE+         ATIME -----> CLK12/CLK24 (new CalcState field)
          |             |                |
       DDAYS        ATIME24          CLKT/CLKTD (new CalcState field)
          |                              |
        DOW                           CLOCK display
                                         |
                             (periodic display update infrastructure)
                                         |
                                    SW stopwatch mode
                                         |
                         +-----------+---+---+-----------+
                         |           |       |           |
                      RUNSW      STOPSW   SETSW      RCLSW
                         |                               |
                       SWPT                           SPLIT

        SETIME / CORRECT / T+X -----> Time offset model
        SETAF / RCLAF -----> Store-only field (no dependency)

        XYZALM -----> Alarm data structure
            |             |
         ALMCAT      ALMNOW -----> run_program() integration
            |             |
    Alarm Catalog    RCLALM
      Keyboard       |
            |     CLALMA / CLALMX / CLRALMS
    Alarm acknowledge
      mechanism
```

**Critical path:** The periodic display update infrastructure (needed for CLOCK and SW) is the foundational new capability that does not exist in the emulator today. Everything else layers on top of existing patterns.

---

## Time and Alarm Formats (from QRG pp. 31-33)

### Time Values

Clock time settings use the following conventions:

| Setting | Clock Time |
|---------|-----------|
| 0 | Midnight |
| 1 | 1 (a.m.) |
| ... | ... |
| 11 | 11 |
| 12 | Noon |
| -1 or 13 | 1 p.m. or 13:00 |
| -2 or 14 | 2 or 14 |
| ... | ... |
| -11 or 23 | 11 or 23 |
| 0 | Midnight (wraps) |

Results of clock-time operations (TIME, RCLALM) are always expressed in 24-hour format in the X-register. Midnight is zero.

### Date Format Table

| Setting | Input* and Output Format (FIX 6 Display) | Display When DATE Executed From Keyboard |
|---------|------------------------------------------|------------------------------------------|
| MDY | MM.DDYYYY | MM/DD/YY day |
| DMY | DD.MMYYYY | DD.MM.YY day |

*Input must be a positive number. All trailing digits after the year must be zero; otherwise an error message will result.

### Alarm Format (XYZALM parameters)

See Category 6 above. The three alarm types are:
1. **Message Alarm** -- ALPHA empty or contains text message.
2. **Interrupting Control Alarm** -- ALPHA contains `>>global_label` or `>>function_name`.
3. **Conditional Control Alarm** -- ALPHA contains `>global_label` or `>function_name`.

### Acknowledging Message Alarms

- Press any key (except STO) to halt a flashing alarm. Non-repeating alarm is deleted; repeating alarm is reset to next occurrence.
- Press SHIFT+C to halt AND clear a repeating alarm.
- Non-message alarms (control) are not acknowledged -- they run programs automatically.

---

## Flags Affected

| Flag | Name | Relevance |
|------|------|-----------|
| 31 | Date Format | 0 = MDY, 1 = DMY. Already exists in the flag system (flag 31 is a system flag, testable but not directly alterable by SF/CF -- only by DMY/MDY functions). |
| 44 | Continuous On | Affects whether the calculator auto-turns-off. Relevant for clock display and alarms. |
| 50 | Message | Set when a message alarm fires. |

---

## New CalcState Fields Required

| Field | Type | Serde | Purpose |
|-------|------|-------|---------|
| `clock_mode_12h` | `bool` | `#[serde(default)]` | CLK12/CLK24 preference; default false (24h) |
| `clock_display_mode` | `enum { TimeOnly, TimeAndDate }` | `#[serde(default)]` | CLKT/CLKTD preference |
| `clock_active` | `bool` | `#[serde(default, skip)]` | Transient: clock display is currently showing |
| `accuracy_factor` | `HpNum` | `#[serde(default)]` | SETAF/RCLAF storage; no-op for emulation |
| `alarms` | `Vec<Alarm>` | `#[serde(default)]` | Persistent alarm catalog |
| `stopwatch_state` | `StopwatchState` | `#[serde(default)]` | Stopwatch running/stopped, elapsed time, start instant |
| `stopwatch_mode` | `bool` | `#[serde(default, skip)]` | Transient: stopwatch keyboard is active |
| `xrom_modules` bit 2 | `u8` | existing field | Bit 2 = Time Module loaded |
| `time_offset` | `Option<Duration>` | `#[serde(default)]` | Offset from host system clock (for SETIME/SETDATE) |

### Alarm Data Structure

```rust
struct Alarm {
    time: HpNum,           // HH.MMSSss
    date: HpNum,           // MM.DDYYYY or DD.MMYYYY
    repeat_interval: HpNum, // HHHH.MMSSs or 0 (no repeat)
    alarm_type: AlarmType,
    message: String,       // Message text or label name
}

enum AlarmType {
    Message,
    ControlInterrupting,
    ControlConditional,
}
```

---

## MVP Recommendation

### Phase 1: Core Time/Date Operations (table stakes, low-medium complexity)

Prioritize first because they have no dependency on periodic updates or new keyboard modes:

1. TIME, DATE -- basic recall from host clock
2. SETIME, SETDATE -- set via offset from host clock
3. DMY, MDY -- flag 31 toggle (trivial)
4. CLK12, CLK24 -- new bool field
5. CLKT, CLKTD -- new enum field
6. DATE+, DDAYS, DOW -- date arithmetic
7. ADATE, ATIME, ATIME24 -- ALPHA formatting
8. T+X -- clock adjustment with midnight rollover
9. SETAF, RCLAF, CORRECT -- accuracy factor (store-only)

This delivers 19 of 35 functions with manageable complexity.

### Phase 2: Periodic Display Infrastructure + Clock Display

The critical new capability. Must be solved before stopwatch or alarm display:

10. CLOCK display mode -- periodic LCD update (1 Hz)
11. CLI integration: ratatui periodic redraw when clock active
12. GUI integration: Tauri periodic state push or frontend timer

### Phase 3: Stopwatch

Builds on the periodic display infrastructure:

13. RUNSW, STOPSW, SETSW, RCLSW -- programmatic stopwatch
14. SW -- stopwatch keyboard mode
15. SWPT -- stopwatch with pointers
16. Split timing with register storage

### Phase 4: Alarm System

The most complex feature; best tackled last:

17. XYZALM -- set alarms (alarm data structure)
18. ALMCAT -- alarm catalog with keyboard mode
19. ALMNOW -- activate past-due alarms
20. RCLALM, CLALMA, CLALMX, CLRALMS -- recall/clear
21. Alarm firing mechanism (background timer check)
22. Alarm acknowledge UI

### Defer:

- Alarm-triggered program execution (ALMNOW + control alarms running programs) could be scoped to a follow-up if it proves too complex to integrate with the existing `run_program()` / `run_loop()` re-entrancy model.

---

## Sources

- [HP 82182A Time Module Quick Reference Card](https://literature.hpcalc.org/community/hp82182a-qrc-en.pdf) -- primary source (read as images)
- [HP-41CX Quick Reference Guide](https://literature.hpcalc.org/community/hp41cx-qrg-en.pdf) -- comprehensive function set + formats
- [HP Museum XROM Numbers](https://www.hpmuseum.org/software/xroms.htm) -- XROM 26 ID confirmation
- [HP 82182A Time Module QREF](https://qrg41.fjk.ch/hp82182a.html) -- function list cross-check
- [HP-41C Wikipedia](https://en.wikipedia.org/wiki/HP-41C) -- alarm system behavior overview
- [HP-41CX Finseth contributions](https://www.finseth.com/hpdata/hp41cx.php) -- function list verification
- [HP 82182A Time Module Owner's Manual TOC](https://archived.hpcalc.org/greendyk/hp41c-time-module/7-contents.html) -- chapter structure
