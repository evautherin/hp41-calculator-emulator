# Requirements: HP-41 Calculator Emulator

**Defined:** 2026-05-24
**Core Value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.

## v3.2 Requirements

Requirements for Time Pac Emulation milestone. Each maps to roadmap phases.

### Framework (XROM Registration & CalcState)

- [ ] **TIME-FW-01**: TIME_MODULE `XromModule` registered with XROM ID 26 and ~33 ops; `xrom_resolve` bit-2 arm fires LAST (after bit-0 Math 1 and bit-1 Stat 1)
- [ ] **TIME-FW-02**: `default_xrom_modules()` returns `0b0000_0111`; `migrate_after_load()` upgrades v3.1 save files (`xrom_modules: 3`) to set bit 2
- [ ] **TIME-FW-03**: New CalcState fields for clock/stopwatch/alarm state follow serde invariants — `#[serde(default)]` for persistent fields, `#[serde(skip)]` for transient fields
- [ ] **TIME-FW-04**: System clock access via `std::time::SystemTime` in hp41-core; time offset (from SETIME/SETDATE) stored as persistent CalcState field
- [ ] **TIME-FW-05**: Stopwatch elapsed time tracked via `std::time::Instant` (monotonic); start marker is transient, accumulated time is persistent
- [ ] **TIME-FW-06**: Alarm catalog stored as persistent CalcState field (Vec of alarm entries, up to 253); each entry carries time, date, message/type, repeat interval

### Clock & Time Recall

- [ ] **TIME-CLK-01**: `TIME` returns current time as HH.MMSSss in X register (system clock + user offset)
- [ ] **TIME-CLK-02**: `DATE` returns current date as MM.DDYYYY or DD.MMYYYY in X register (controlled by Flag 31)
- [ ] **TIME-CLK-03**: `ATIME` appends current time to ALPHA register in HH:MM:SS format (12h if CLK12, 24h if CLK24)
- [ ] **TIME-CLK-04**: `ATIME24` appends current time to ALPHA register in 24-hour HH:MM:SS format regardless of CLK12/CLK24 setting
- [ ] **TIME-CLK-05**: `ADATE` appends current date to ALPHA register in locale-appropriate format (controlled by Flag 31)
- [ ] **TIME-CLK-06**: `T+X` adds X register value (seconds) to the time accumulator in the alarm register

### Date Arithmetic

- [ ] **TIME-DAT-01**: `DATE+` adds X days to date in Y (respecting DMY/MDY format via Flag 31); returns result date in X
- [ ] **TIME-DAT-02**: `DDAYS` computes days between dates in X and Y (respecting DMY/MDY format); returns signed count in X
- [ ] **TIME-DAT-03**: `DOW` returns day-of-week (0=Sunday..6=Saturday) for date in X
- [ ] **TIME-DAT-04**: `DMY` sets Flag 31 (date format DD.MMYYYY); aliased to `SF 31`
- [ ] **TIME-DAT-05**: `MDY` clears Flag 31 (date format MM.DDYYYY); aliased to `CF 31`
- [ ] **TIME-DAT-06**: Date decimal format uses string-split-at-decimal parsing (ISG/DSE precedent), NOT float arithmetic

### Clock Display

- [ ] **TIME-DSP-01**: `CLKT` toggles clock display mode — when active, LCD shows current time (live-updating); normal display resumes on any keypress
- [ ] **TIME-DSP-02**: `CLKTD` toggles date display within clock mode — when active, LCD alternates between time and date
- [ ] **TIME-DSP-03**: `SETIME` prompts user for time via modal (HH.MMSSss format); stores as offset from system clock
- [ ] **TIME-DSP-04**: `SETDATE` prompts user for date via modal (MM.DDYYYY or DD.MMYYYY per Flag 31); stores as offset from system clock
- [ ] **TIME-DSP-05**: Clock display updates at ≥1 Hz in both CLI and GUI when active

### Format & Adjustment

- [ ] **TIME-FMT-01**: `CLK12` sets 12-hour display format (AM/PM suffix in ATIME)
- [ ] **TIME-FMT-02**: `CLK24` sets 24-hour display format
- [ ] **TIME-FMT-03**: `SETAF` stores accuracy factor from X register (value stored but has no effect in emulation — documented divergence)
- [ ] **TIME-FMT-04**: `RCLAF` recalls accuracy factor to X register
- [ ] **TIME-FMT-05**: `CORRECT` adjusts time by accuracy factor (in emulation: stores factor, documented as no-op divergence since system clock is authoritative)

### Stopwatch

- [ ] **TIME-SW-01**: `SETSW` initializes stopwatch display mode with split-point tracking; LCD shows running time in HH:MM:SS.hh format
- [ ] **TIME-SW-02**: `SW` starts the interactive stopwatch mode — dedicated keyboard layout for stopwatch control (start/stop/split/reset)
- [ ] **TIME-SW-03**: `STPW` records a split point (current elapsed time) while stopwatch continues running
- [ ] **TIME-SW-04**: `RUNSW` starts/resumes the stopwatch timer (programmable — no interactive mode)
- [ ] **TIME-SW-05**: `STOPSW` stops the stopwatch timer (programmable)
- [ ] **TIME-SW-06**: `RCLSW` recalls current stopwatch elapsed time to X register as HH.MMSSss
- [ ] **TIME-SW-07**: `SWPT` recalls the last split-point time to X register
- [ ] **TIME-SW-08**: Stopwatch display updates at ≥10 Hz when running in both CLI and GUI (centisecond resolution visible)
- [ ] **TIME-SW-09**: Stopwatch uses monotonic clock (`Instant`) for elapsed time — immune to system clock changes

### Alarm System

- [ ] **TIME-ALM-01**: `XYZALM` sets an alarm — time from X, date from Y, type/message from Z/ALPHA; supports message alarms and control alarms
- [ ] **TIME-ALM-02**: `RCLALM` recalls alarm fields — alarm number from X; returns time, date, type/message to stack and ALPHA
- [ ] **TIME-ALM-03**: `ALMCAT` enters interactive Alarm Catalog mode — displays alarms chronologically with dedicated keyboard for navigation/acknowledge/delete
- [ ] **TIME-ALM-04**: `CLALMA` clears (acknowledges) a specific alarm by number
- [ ] **TIME-ALM-05**: `CLALMX` clears a specific alarm by number (extended clear)
- [ ] **TIME-ALM-06**: `CLRALMS` clears all alarms
- [ ] **TIME-ALM-07**: `ALMNOW` triggers an immediate alarm (for testing)
- [ ] **TIME-ALM-08**: Past-due alarm detection — on each keypress/dispatch, check for overdue alarms and surface notification
- [ ] **TIME-ALM-09**: Message alarms display message in ALPHA register and beep
- [ ] **TIME-ALM-10**: Control alarms trigger program execution (XEQ label stored in alarm); interrupting control alarms suspend current operation
- [ ] **TIME-ALM-11**: Repeating alarms (repeat interval > 0) reschedule after acknowledgment
- [ ] **TIME-ALM-12**: Alarm catalog persists across save/load cycles

### CLI Integration

- [ ] **TIME-CLI-01**: `docs/hp41-time-functions.json` authored with all Time Pac entries following the v3.0/v3.1 JSON-canonical schema
- [ ] **TIME-CLI-02**: Fourth `OnceLock<Vec<HelpEntry>>` in `help_data.rs` for Time Pac JSON; `help_entries_all()` merges 4 pools
- [ ] **TIME-CLI-03**: All new `Op` variants have `op_display_name` arms in `hp41-cli/src/prgm_display.rs` (4-way invariant item 3)
- [ ] **TIME-CLI-04**: `?` help overlay gains "Time Pac (XROM 26)" section
- [ ] **TIME-CLI-05**: Clock display mode renders live-updating time in the TUI display area (reuses ratatui 16ms poll loop for refresh)
- [ ] **TIME-CLI-06**: Stopwatch interactive mode renders in TUI with dedicated key bindings
- [ ] **TIME-CLI-07**: Alarm notifications surface as status-bar messages in CLI
- [ ] **TIME-CLI-08**: `xrom_shadowing.rs` extended to `TIME_MODULE.ops` — all Time Pac mnemonics confirmed disjoint from Math 1 + Stat 1 + built-in allowlist

### Documentation

- [ ] **TIME-DOC-01**: `docs/hp41-time-function-matrix.md` generated via `just docs-matrix` (fourth invocation)
- [ ] **TIME-DOC-02**: `docs/hp41-time-divergences.md` authored with emulator divergences (accuracy factor no-op, system clock backing, alarm limits)
- [ ] **TIME-DOC-03**: ADRs for key architectural decisions (clock access pattern, live display architecture, alarm catalog design)
- [ ] **TIME-DOC-04**: README updated with Time Pac soft-claim
- [ ] **TIME-DOC-05**: CLAUDE.md updated with `### v3.2 additions` block
- [ ] **TIME-DOC-06**: `docs/architecture-history.md` v3.2 narrative section

### GUI Integration

- [ ] **TIME-GUI-01**: All new `Op` variants have `op_display_name` arms in `hp41-gui/src-tauri/src/prgm_display.rs` (4-way invariant item 4)
- [ ] **TIME-GUI-02**: CATALOG 2 gains "TIME 2C" section for Time Pac functions
- [ ] **TIME-GUI-03**: Help overlay gains "Time Pac (XROM 26)" section
- [ ] **TIME-GUI-04**: Clock display mode renders live time in GUI LCD (conditional `setInterval` — controlled D-11 exception)
- [ ] **TIME-GUI-05**: Stopwatch mode renders in GUI with live-updating LCD
- [ ] **TIME-GUI-06**: Alarm notifications surface as toast overlay in GUI
- [ ] **TIME-GUI-07**: Modal prompts for SETIME/SETDATE route through existing `modal_prompt` channel

### Quality Gates

- [ ] **TIME-QUAL-01**: `hp41-core` region coverage ≥ 93% maintained after Time Pac additions
- [ ] **TIME-QUAL-02**: Numerical accuracy suite extended with date arithmetic cases (DATE+, DDAYS, DOW edge cases — leap years, century boundaries, Feb 29)
- [ ] **TIME-QUAL-03**: Per-Op test count ≥ 5 for all new Time Pac variants (meta-gate `time_op_test_count.rs`)
- [ ] **TIME-QUAL-04**: Backward compatibility test — v3.1 save file loads in v3.2 with `xrom_modules` migration and default clock/alarm state
- [ ] **TIME-QUAL-05**: Free42 contamination guard extended to cover Time Module tokens
- [ ] **TIME-QUAL-06**: E2E smoke extended with at least one Time Pac workflow (e.g., DATE+ or DDAYS)
- [ ] **TIME-QUAL-07**: `function_matrix_parity.rs` extended to 4-pool partition test (cv + math1 + stat1 + time)
- [ ] **TIME-QUAL-08**: XROM shadowing test covers all 3 XROM modules (Math 1 + Stat 1 + Time)
- [ ] **TIME-QUAL-09**: README hard-claim graduated after quality gates pass (mirrors v3.0/v3.1 graduation pattern)
- [ ] **TIME-QUAL-10**: Stopwatch timing accuracy within ±10ms over 60s test window
- [ ] **TIME-QUAL-11**: Alarm past-due detection fires within one dispatch cycle

## v3.3+ Requirements

Deferred to future release. Tracked but not in current roadmap.

### Advanced Matrix Pac

- **AMAT-01**: M+, MAT*, INV-as-transpose, V+, VDOT, IDN — extend existing matrix framework

### Advantage Pac

- **ADV-01**: PROOT, CABS, CARG, CCHS, CCONJ, Romberg-INTG, CY^X — complex math extensions

### Binary Releases

- **REL-01**: Signed cross-platform binaries via cargo-dist (CLI) + tauri-action (GUI)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Cycle-accurate clock chip emulation | System clock is more useful than simulating the HP-41CX crystal oscillator |
| Battery-backed clock persistence | System clock provides this automatically |
| CORRECT auto-adjustment algorithm | No physical crystal to adjust; accuracy factor stored but inert |
| HP-IL time synchronization | HP-IL peripheral emulation excluded permanently |
| Multiple alarm catalog views | OM shows single chronological view; no need for additional sort modes |
| Sub-millisecond stopwatch resolution | HP-41CX hardware resolution was centiseconds; we match that |

## Traceability

(Populated during roadmap creation)

| Requirement | Phase | Status |
|-------------|-------|--------|
| (filled by roadmapper) | | |

**Coverage:**
- v3.2 requirements: 70 total
- Mapped to phases: 0
- Unmapped: 70

---
*Requirements defined: 2026-05-24*
*Last updated: 2026-05-24 after initial definition*
