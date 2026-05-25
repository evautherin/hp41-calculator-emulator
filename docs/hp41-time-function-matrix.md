# HP-41C Time Pac Function Matrix

> Generated from `docs/hp41-time-functions.json` via `just docs-matrix`.
> Edit the JSON, regenerate this file, commit both.

## Implemented (v2.x)

| Op | Display | XROM | Category | Status | Phase | Key Path | Description |
|----|---------|------|----------|--------|-------|----------|-------------|
| TimeAlmcat | ALMCAT | Time / 26-28 | Time Alarm | ✓ v2.x | 38 | `XEQ "ALMCAT"` | Catalog all pending alarms to the display |
| TimeAlmnow | ALMNOW | Time / 26-29 | Time Alarm | ✓ v2.x | 38 | `XEQ "ALMNOW"` | Trigger the next alarm immediately |
| TimeClalma | CLALMA | Time / 26-33 | Time Alarm | ✓ v2.x | 38 | `XEQ "CLALMA"` | Clear all alarms from the alarm catalog |
| TimeClalmx | CLALMX | Time / 26-34 | Time Alarm | ✓ v2.x | 38 | `XEQ "CLALMX"` | Clear alarm number X from the alarm catalog |
| TimeClralms | CLRALMS | Time / 26-35 | Time Alarm | ✓ v2.x | 38 | `XEQ "CLRALMS"` | Clear all repeating alarms from the catalog |
| TimeRclalm | RCLALM | Time / 26-30 | Time Alarm | ✓ v2.x | 38 | `XEQ "RCLALM"` | Recall alarm N from X into X/Y/Z registers |
| TimeXyzalm | XYZALM | Time / 26-27 | Time Alarm | ✓ v2.x | 38 | `XEQ "XYZALM"` | Set alarm: X=time, Y=date, Z=repeat interval |
| TimeAdate | ADATE | Time / 26-19 | Time Alpha | ✓ v2.x | 38 | `XEQ "ADATE"` | Append current date as ALPHA string |
| TimeAtime | ATIME | Time / 26-17 | Time Alpha | ✓ v2.x | 38 | `XEQ "ATIME"` | Append current time as ALPHA string (12-hour format) |
| TimeAtime24 | ATIME24 | Time / 26-18 | Time Alpha | ✓ v2.x | 38 | `XEQ "ATIME24"` | Append current time as ALPHA string (24-hour format) |
| TimeClock | CLOCK | Time / 26-9 | Time Clock | ✓ v2.x | 38 | `XEQ "CLOCK"` | Enter live clock display mode — press any key to exit |
| TimeDate | DATE | Time / 26-2 | Time Clock | ✓ v2.x | 38 | `XEQ "DATE"` | Display current date in X (MM.DDYYYY or DD.MMYYYY) |
| TimeTime | TIME | Time / 26-1 | Time Clock | ✓ v2.x | 38 | `XEQ "TIME"` | Display current system time in X (HH.MMSSss) |
| TimeTplusx | T+X | Time / 26-11 | Time Clock | ✓ v2.x | 38 | `XEQ "T+X"` | Add X seconds to the current time |
| TimeDatePlus | DATE+ | Time / 26-12 | Time Date Arithmetic | ✓ v2.x | 38 | `XEQ "DATE+"` | Add X days to date in Y; result in X |
| TimeDdays | DDAYS | Time / 26-13 | Time Date Arithmetic | ✓ v2.x | 38 | `XEQ "DDAYS"` | Days between date in X and date in Y |
| TimeDmy | DMY | Time / 26-15 | Time Date Arithmetic | ✓ v2.x | 38 | `XEQ "DMY"` | Switch date format to DD.MMYYYY |
| TimeDow | DOW | Time / 26-14 | Time Date Arithmetic | ✓ v2.x | 38 | `XEQ "DOW"` | Day of week for date in X (1=Mon … 7=Sun) |
| TimeMdy | MDY | Time / 26-16 | Time Date Arithmetic | ✓ v2.x | 38 | `XEQ "MDY"` | Switch date format to MM.DDYYYY |
| TimeClkt | CLKT | Time / 26-7 | Time Display | ✓ v2.x | 38 | `XEQ "CLKT"` | Toggle clock display mode on the LCD |
| TimeClktd | CLKTD | Time / 26-8 | Time Display | ✓ v2.x | 38 | `XEQ "CLKTD"` | Toggle clock-and-date display mode on the LCD |
| TimeSetdate | SETDATE | Time / 26-4 | Time Display | ✓ v2.x | 38 | `XEQ "SETDATE"` | Set the system date from X (MM.DDYYYY or DD.MMYYYY) |
| TimeSetime | SETIME | Time / 26-3 | Time Display | ✓ v2.x | 38 | `XEQ "SETIME"` | Set the system time from X (HH.MMSSss) |
| TimeClk12 | CLK12 | Time / 26-5 | Time Format | ✓ v2.x | 38 | `XEQ "CLK12"` | Switch to 12-hour clock format (AM/PM) |
| TimeClk24 | CLK24 | Time / 26-6 | Time Format | ✓ v2.x | 38 | `XEQ "CLK24"` | Switch to 24-hour clock format |
| TimeCorrect | CORRECT | Time / 26-10 | Time Format | ✓ v2.x | 38 | `XEQ "CORRECT"` | Apply crystal accuracy correction (no-op; host clock is authoritative) |
| TimeRclaf | RCLAF | Time / 26-31 | Time Format | ✓ v2.x | 38 | `XEQ "RCLAF"` | Recall stored accuracy factor into X |
| TimeSetaf | SETAF | Time / 26-32 | Time Format | ✓ v2.x | 38 | `XEQ "SETAF"` | Set accuracy factor from X (crystal correction) |
| TimeRclsw | RCLSW | Time / 26-22 | Time Stopwatch | ✓ v2.x | 38 | `XEQ "RCLSW"` | Recall stopwatch elapsed time into X |
| TimeRunsw | RUNSW | Time / 26-20 | Time Stopwatch | ✓ v2.x | 38 | `XEQ "RUNSW"` | Start (run) the stopwatch |
| TimeSetsw | SETSW | Time / 26-23 | Time Stopwatch | ✓ v2.x | 38 | `XEQ "SETSW"` | Set stopwatch to value in X |
| TimeStopsw | STOPSW | Time / 26-21 | Time Stopwatch | ✓ v2.x | 38 | `XEQ "STOPSW"` | Stop the stopwatch |
| TimeStpw | STPW | Time / 26-26 | Time Stopwatch | ✓ v2.x | 38 | `XEQ "STPW"` | Record current elapsed time as split point (stopwatch continues running) |
| TimeSw | SW | Time / 26-24 | Time Stopwatch | ✓ v2.x | 38 | `XEQ "SW"` | Interactive stopwatch display mode with live LCD updates |
| TimeSwpt | SWPT | Time / 26-25 | Time Stopwatch | ✓ v2.x | 38 | `XEQ "SWPT"` | Stopwatch split/lap time: recall elapsed without stopping |

## v3.x Deferred (Module Pacs)

_None._
