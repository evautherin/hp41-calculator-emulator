# Project Research Summary

**Project:** HP-41 Calculator Emulator -- v3.2 Time Pac Emulation
**Domain:** Behavioral emulation of third HP-41 XROM application module (HP 82182A Time Module / HP-41CX built-in, XROM 26, OM 00041-90035)
**Researched:** 2026-05-24
**Confidence:** MEDIUM-HIGH

## Executive Summary

The HP-41CX Time Module (XROM 26, 33 callable functions across clock/date/alarm/stopwatch categories) is architecturally distinct from Math Pac I and Stat 1 Pac in one fundamental way: it introduces real-time behavior into a previously event-driven emulator. The Time Pac is not just another dispatch table of pure functions -- it requires live clock display, a running stopwatch with sub-second LCD updates, and an alarm system that fires based on wall-clock time, none of which have any precedent in v1.0-v3.1. The recommended approach is "pull on redraw": hp41-core remains thread-free and async-free; time-dependent state (clock display, stopwatch elapsed time) is computed on-demand when the frontend's existing redraw cycle asks for display state. The CLI's 16ms poll loop already provides 60fps rendering opportunity; the GUI gains a purpose-built `tick_time` Tauri command called via `setInterval` only when clock or stopwatch display is active (a controlled, documented exception to the D-11 no-polling invariant).

Zero new runtime dependencies are needed. System clock reads use `std::time::SystemTime` (standard library). Stopwatch timing uses `std::time::Instant` (standard library, already used by CLI auto-save timer). Date arithmetic uses hand-coded Fliegel-Van Flandern Julian Day Number conversion (~60 LOC of pure integer math, derived from the 1968 ACM paper). The `chrono` and `time` crates were evaluated and rejected -- 25K-45K LOC for 5 date functions violates the zero-new-deps discipline established in ADR-v3.1-002. Date decimal format parsing (MM.DDYYYY / DD.MMYYYY) uses the ISG/DSE string-split-at-decimal-point technique, never float arithmetic. Flag 31 is the sole DMY/MDY control, implemented as `SF 31` / `CF 31` directly on the existing flags bitfield with no separate `date_format` field.

The alarm system is the highest-risk subsystem and accounts for the confidence downgrade from HIGH to MEDIUM-HIGH. It introduces three alarm types (message, interrupting control, non-interrupting control), past-due queuing, repeat intervals, alarm catalog keyboard mode, and -- most critically -- interrupting control alarms that can suspend a running program and execute a different one. This last capability requires re-entrancy against the 4-level call stack, which is not currently supported. The strong recommendation is to defer interrupting control alarm execution to a follow-up or document it as a known divergence. Message alarms and non-interrupting control alarms are implementable within the existing architecture. The alarm system should be the LAST feature phase, not the first.

## Key Findings

### Recommended Stack

Zero new runtime dependencies in `hp41-core`, `hp41-cli`, or `hp41-gui`. The Time Pac is implementable entirely with standard library types and hand-coded algorithms.

**Core technologies:**
- `std::time::SystemTime` + `UNIX_EPOCH`: Host OS clock read for TIME/DATE ops -- cross-platform, part of Rust std, returns UTC seconds convertible to local time via `libc::localtime_r` (Unix) or `GetLocalTime` (Windows), both already transitive deps of std
- `std::time::Instant` + `Duration`: Monotonic stopwatch timing -- already used in `hp41-cli/src/app.rs` for auto-save timer; guarantees no backward jumps; nanosecond precision exceeds the HP-41CX's centisecond resolution
- Hand-coded Fliegel-Van Flandern JDN (~60 LOC): Gregorian calendar arithmetic (DATE+, DDAYS, DOW) -- textbook algorithm from Communications of the ACM (1968); pure integer math; covers Gregorian range Oct 15, 1582 through Sep 10, 4320
- `rust_decimal 1.42` (existing): HpNum arithmetic for all time/date value representation -- no gaps
- **Rejected:** `chrono 0.4.44` (45K LOC, pulls iana-time-zone), `time 0.3.47` (25K LOC, local-offset soundness issues), `tokio`/`async-std` (alarm scheduling via simple comparison, not an async scheduler), timezone crates (HP-41CX has no timezone concept)

### Expected Features

33 callable functions confirmed from HP 82182A QRC (82182-90002) cross-referenced with HP-41CX QRG (00041-90475) and HP Museum XROM database.

**Must have -- table stakes for "Time Module emulation" claim:**
- TIME, DATE -- basic recall from host system clock (LOW complexity)
- SETIME, SETDATE, CORRECT, T+X -- clock set/adjust with midnight rollover (MEDIUM)
- DMY, MDY -- flag 31 toggle; trivial (LOW)
- CLK12, CLK24, CLKT, CLKTD -- display format preferences (LOW)
- CLOCK -- live clock display; first real-time continuous update in the emulator (HIGH)
- DATE+, DDAYS, DOW -- date arithmetic trio via JDN (MEDIUM)
- ADATE, ATIME, ATIME24 -- ALPHA register formatting (MEDIUM)
- SETAF, RCLAF -- accuracy factor store-only, no-op for emulation (LOW)
- XYZALM, ALMCAT, ALMNOW, RCLALM, CLALMA, CLALMX, CLRALMS -- alarm catalog system (VERY HIGH)
- RUNSW, STOPSW, SETSW, RCLSW -- programmatic stopwatch (MEDIUM)
- SW -- stopwatch keyboard mode with live display (HIGH)
- SWPT -- stopwatch split pointer (MEDIUM)

**Should have -- differentiators:**
- Host-clock integration (real system time, not fake counter) -- natural for software emulator
- Persistent alarm catalog surviving save/load -- serialize `Vec<AlarmEntry>` to autosave.json
- Divergence documentation following v3.0/v3.1 established pattern

**Defer to v3.2+ follow-up or document as divergence:**
- Interrupting control alarm program execution (re-entrancy against 4-level call stack)
- Cycle-accurate crystal oscillator simulation (no user value)
- HP-IL alarm wake-up (no OFF state in emulator)
- Accuracy factor correction loop (host OS clock is NTP-synchronized)
- Extended Memory / Text Editor functions (XROM 25, not XROM 26)

### Architecture Approach

Strategy is "pull on redraw" -- hp41-core remains thread-free, timer-free, and async-free. Time-dependent display state is computed on-demand during the frontend's existing redraw cycle. The CLI's 16ms `event::poll` already redraws at 60fps; when clock or stopwatch display is active, the display-string computation reads `SystemTime::now()` or `Instant::elapsed()` on each redraw. The GUI gains a purpose-built `tick_time` Tauri command gated behind `clockActive || swRunning` flags. Eight new CalcState fields (5 persistent, 3 transient) support clock config, alarm catalog, and stopwatch state. The `ModalProgram` enum gains a `Time(TimeStep)` variant following the Stat 1 pattern exactly.

**Major components:**
1. `hp41-core/src/ops/time/` -- 7-file module tree: `mod.rs`, `clock.rs` (15 ops), `date_arith.rs` (JDN algorithms + DATE+/DDAYS/DOW), `alpha_time.rs` (ADATE/ATIME/ATIME24), `alarm.rs` (alarm catalog + check_alarms), `stopwatch.rs` (SW/RUNSW/STOPSW/SETSW/RCLSW/SWPT), `modal.rs` (TimeStep enum for SETIME/SETDATE/XYZALM prompts)
2. `hp41-core/src/ops/math1/xrom.rs` -- `TIME_1` const (XROM 26, bit 2) + `time_resolve()` + bit-2 arm in `xrom_resolve()`
3. `hp41-core/src/state.rs` -- 8 new fields; `migrate_after_load()` v3.1->v3.2 (xrom_modules `0b11` -> `0b111`); `AlarmEntry`, `StopwatchState`, `ClockDisplayMode`, `DateFormat` type definitions
4. `docs/hp41-time-functions.json` -- fourth JSON source-of-truth (~33 entries)
5. Frontend integration -- CLI: always-redraw when clock/SW active + alarm-check in poll loop; GUI: conditional `setInterval` for `tick_time` + alarm toast

### Critical Pitfalls

1. **P32: Live display without polling loop** -- The emulator has been purely event-driven for 3 milestones. Clock/stopwatch display needs sub-second refresh independent of keystrokes. CLI fix: always redraw after poll returns (not just on key event) when clock/SW mode is active. GUI fix: purpose-built `tick_time` command via conditional `setInterval`, documented exception to D-11. This is the foundational architectural decision -- must be resolved in the first phase.

2. **P34: System clock dependency in I/O-free hp41-core** -- `hp41-core` currently has zero I/O surface. TIME/DATE need the host clock. Two viable approaches: (a) call `std::time::SystemTime::now()` directly in core (it is a value-returning syscall, not console I/O -- analogous to `AtomicBool::load` which core already does), or (b) inject clock values as transient CalcState fields updated by frontend before each dispatch. Approach (b) enables deterministic testing but adds frontend coupling. Decision must be made in first phase.

3. **P35: Date decimal format parsing** -- MM.DDYYYY stored as HpNum has treacherous edge cases (single-digit months, leading zeros in day field, century boundaries, trailing-zero preservation). MUST use string-split at decimal point (ISG/DSE precedent), left-pad fractional part to exactly 6 chars, then split into DD[2] + YYYY[4]. Never use float arithmetic for field extraction.

4. **P37: Alarm system state machine** -- Four alarm types, three triggering contexts, past-due queuing, repeat intervals, keyboard mode, and program execution triggers. Implement as structured `Vec<AlarmEntry>` with `check_alarms()` called by frontend (drain pattern), NOT by dispatch. Defer interrupting control alarm execution (P39 re-entrancy risk).

5. **P41: Flag 31 semantic collision** -- DMY/MDY must be implemented as `SF 31` / `CF 31` on the existing flags bitfield. Do NOT add a separate `date_format` enum field. All date formatting functions check `state.flags & (1 << 31)` directly. Test bidirectional: `SF 31` produces DMY format, `DMY` sets flag 31.

## Implications for Roadmap

Based on combined research, a 5-phase structure starting at Phase 38 mirrors the v3.1 Stat 1 pattern (Phases 33-37). The critical dependency chain: live display infrastructure (P32) must be decided before any clock/stopwatch op is written; date arithmetic (JDN) must be validated before SETDATE/DATE+ use it; alarm system is the most complex subsystem and should come last.

### Phase 38: hp41-core -- XROM Framework, Clock/Date Ops, Stopwatch Core

**Rationale:** The ~33 new Op variants must land in dispatch() + execute_op() first (4-way invariant items 1+2). The live display architecture decision (P32) and system clock access pattern (P34) are foundational -- every subsequent phase depends on them. Date arithmetic (JDN) is a pure algorithm with no frontend dependency and should be validated with comprehensive edge cases before any date-consuming op is written. Stopwatch state machine is self-contained and testable in isolation.
**Delivers:** All ~33 Op variants in hp41-core; JDN date arithmetic validated against edge cases; stopwatch state machine (Instant-based, transient); clock display mode flags; alarm data structure (Vec<AlarmEntry>); DMY/MDY via flag 31; SETIME/SETDATE time-offset model; ADATE/ATIME/ATIME24 formatting; check_alarms() function; xrom_modules default `0b0000_0111` with v3.1 migration; Free42 contamination guard extended to `time/` directory; 4-way match items 1+2 complete (intentional CI break in CLI/GUI)
**Addresses features:** All 33 callable functions at core level
**Avoids:** P32 (architecture decided), P34 (clock access pattern locked), P35 (string-split parsing from day one), P36 (mnemonic shadowing verified), P40 (contamination guard extended before first time source file), P41 (flag 31 = sole DMY/MDY control), P44 (new HH.MMSSss parser, not reuse hms.rs)

### Phase 39: hp41-cli -- CLI Integration + Live Display

**Rationale:** CLI closes 4-way invariant item 3 and is the first frontend to implement live clock/stopwatch display. The existing 16ms poll loop makes CLI the simpler validation target for the live display paradigm. JSON help pipeline (`docs/hp41-time-functions.json`) must exist before `OnceLock` wiring.
**Delivers:** 4-way invariant item 3 complete; `docs/hp41-time-functions.json` (fourth JSON source-of-truth); fourth `OnceLock` in `help_data.rs`; `?` overlay "Time 2C (XROM 26)" section; always-redraw when clock/SW active in event loop; alarm-check drain in poll loop; stopwatch keyboard mode (PendingInput::StopwatchMode); xrom_shadowing.rs extended to TIME.ops
**Avoids:** P32 (always-redraw implemented), P38 (SW keyboard mode as CalcState flag), P43 (sanctioned CI break resolved for CLI)

### Phase 40: Documentation and ADRs

**Rationale:** Divergence catalog, docs-matrix extension, and ADR authoring require Phase 38+39 implementation decisions to be final. Follows Phase 35 (Stat 1 docs) and Phase 30 (Math 1 docs) pattern exactly.
**Delivers:** `docs/hp41-time-divergences.md` (accuracy factor no-op, stopwatch persistence behavior, interrupting alarm deferral, host-clock-vs-crystal); `docs/hp41-time-function-matrix.md` generated via `just docs-matrix` (fourth invocation); new ADRs for clock access pattern, live display architecture, alarm deferral scope; README v3.2 soft-claim; CLAUDE.md v3.2 additions section; architecture-history.md v3.2 narrative
**Avoids:** P42 (accuracy factor divergence documented)

### Phase 41: hp41-gui -- GUI Integration + Live Display

**Rationale:** GUI closes 4-way invariant item 4. The `tick_time` Tauri command and conditional `setInterval` for clock/stopwatch display are the GUI-specific live display implementation. CATALOG 2 gains TIME 2C entry. Pattern is identical to Phase 36 (Stat 1 GUI) with the addition of the time-tick mechanism.
**Delivers:** 4-way invariant item 4 complete; `tick_time` command + permission TOML; CalcStateView gains `clock_display_str`, `sw_running`, `clock_active` fields; conditional `setInterval` in App.tsx (D-11 exception documented); alarm toast notification; help overlay "Time 2C (XROM 26)" section; SC-4 invariant maintained
**Avoids:** P32 (frontend timer implemented), P43 (sanctioned CI break resolved for GUI)

### Phase 42: Test Hardening and Quality Gates

**Rationale:** Coverage gate maintenance requires comprehensive tests for ~33 new ops across date arithmetic, stopwatch timing, alarm catalog, and clock display. Date edge cases (Gregorian boundary, leap years, Y2K, century boundaries) need a dedicated suite. Backward-compat test for v3.1 save fixture. E2E smoke for one Time Module workflow.
**Delivers:** `time_date_accuracy.rs` with JDN edge cases (Oct 15, 1582; Feb 29 leap/non-leap; Dec 31, 9999); `time_stopwatch.rs` (state machine transitions); `time_alarm.rs` (catalog CRUD + check_alarms triggering); `time_backward_compat.rs` + v3.1 autosave fixture; `numerical_accuracy.rs` extended with date arithmetic oracle cases; `time_op_test_count.rs` meta-gate (33 variants >= 5 tests); E2E smoke for DATE+DDAYS workflow; Free42 contamination guard re-verified (script extended to time/ dir)
**Avoids:** P33 (backward compat verified), P35 (date edge case suite), P37 (alarm state machine tested per type x context)

### Phase Ordering Rationale

- **Core first (Phase 38)** because the 4-way exhaustive-match invariant requires all Op variants to exist before CLI/GUI can compile. The live display architecture decision and clock access pattern are load-bearing for all subsequent phases.
- **CLI before GUI (Phase 39 before 41)** because the CLI's simpler event loop validates the live display paradigm before the more complex GUI integration. The xrom_shadowing CI gate catches mnemonic collisions before GUI integration.
- **Docs after CLI (Phase 40)** because divergence catalog entries are informed by implementation decisions made in Phases 38-39. ADRs for clock access and live display architecture need the actual implementation to reference.
- **GUI after docs (Phase 41)** because the D-11 exception documentation must be written before the code that creates the exception.
- **Tests last (Phase 42)** because the meta-gate requires all Op variants to be present across all 4 match sites, and E2E smoke needs both CLI and GUI integration complete.

### Research Flags

**Needs `/gsd-plan-phase --research-phase N` during planning:**
- **Phase 38 (Core):** NEEDS research phase -- P32 live display architecture decision needs concrete prototyping; P34 clock access pattern (direct std::time vs. injection) needs decision with test implications; alarm type encoding (ALPHA prefix `>>` vs `>`) needs Owner's Manual verification; XYZALM stack parameter encoding needs OM confirmation; exact stopwatch keyboard mapping needs OM verification; which Time functions are non-programmable (CLOCK, SW) needs OM verification

**Standard patterns -- skip research phase:**
- **Phase 39 (CLI):** Phase 34 playbook applies directly + live display addendum
- **Phase 40 (Docs):** Phase 35 / D-29.1 precedent applies directly
- **Phase 41 (GUI):** Phase 36 playbook applies directly + tick_time addendum
- **Phase 42 (Tests):** Phase 37 patterns apply directly

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Zero new deps confirmed; std::time types verified from doc.rust-lang.org; Fliegel-Van Flandern algorithm is textbook ACM 1968; chrono/time rejection is well-justified |
| Features | HIGH | 33 functions confirmed from QRC (read directly as images) cross-referenced with QRG (40-page PDF), HP Museum XROM database, and calc.fjk.ch module database; XROM 26 confirmed from 3 independent sources |
| Architecture | MEDIUM-HIGH | "Pull on redraw" pattern is sound for CLI (16ms poll already exists); GUI tick_time is a controlled D-11 exception with clear precedent rationale; alarm system architecture is well-designed but interrupting control alarms are deliberately deferred |
| Pitfalls | MEDIUM-HIGH | Date arithmetic pitfalls (P35) are well-understood (string-split precedent); live display (P32) is novel but tractable; alarm system (P37) is the most complex feature with MEDIUM confidence on XYZALM parameter encoding; stopwatch keyboard (P38) needs OM verification for exact key mappings |

**Overall confidence:** MEDIUM-HIGH

The downgrade from HIGH (v3.1 Stat 1) to MEDIUM-HIGH reflects two factors: (1) the Time Pac introduces a genuinely new paradigm (real-time display updates) that has no precedent in the codebase, and (2) the alarm system's full behavioral specification -- particularly interrupting control alarms and the ALMCAT keyboard mode -- requires Owner's Manual sections not yet fully read. The core clock/date/stopwatch functions have HIGH confidence.

### Gaps to Address

- **Live display architecture (P32 -- critical path):** The "pull on redraw" strategy is recommended but needs concrete validation in Phase 38. The CLI fix (always-redraw when clock/SW active) is straightforward. The GUI fix (tick_time with conditional setInterval) is a documented D-11 exception that needs careful implementation and testing. Resolve in Phase 38 Phase 0.

- **Clock access pattern in hp41-core (P34):** Two viable approaches (direct std::time vs. frontend injection). The STACK.md researcher recommends direct std::time (it is a value-returning syscall, not console I/O). The PITFALLS.md researcher recommends frontend injection for deterministic testing. Resolve in Phase 38 Phase 0 -- likely direct std::time with test helper overrides.

- **XYZALM alarm type encoding:** The `>>` (interrupting) vs `>` (non-interrupting) prefix convention in ALPHA is confirmed from QRC and Wikipedia but needs Owner's Manual verification for exact encoding semantics. Resolve in Phase 38 Phase 0.

- **Stopwatch keyboard mapping:** The QRC shows the stopwatch keyboard layout but the exact key-to-function mapping (which physical HP-41 keys map to SPLIT, R/S, CLEAR, etc.) needs Owner's Manual verification. Resolve in Phase 38 Phase 0.

- **Non-programmable functions:** Whether CLOCK and SW are non-programmable (keyboard-only, not valid in programs) needs Owner's Manual verification. This affects whether `execute_op()` arms for these ops are no-ops or errors. Resolve in Phase 38 Phase 0.

- **Interrupting control alarm re-entrancy (P39):** Deliberately deferred. If implemented later, requires a separate `alarm_context: Option<AlarmContext>` field to save/restore program execution state. Document as a known divergence for v3.2.

## Sources

### Primary (HIGH confidence)
- HP 82182A Time Module Quick Reference Card (82182-90002, November 1981) -- all functions, alarm formats, stopwatch keyboard, XYZALM parameters; read directly as 2-page image
- HP-41CX Quick Reference Guide (00041-90475, August 1983) -- complete function set, keyboard layouts, Time and Alarm Formats section, Flags table, Error list; read as 40-page PDF
- HP Museum XROM Numbers list (hpmuseum.org/software/xroms.htm) -- XROM 26 confirmed for Time Module
- HP-41 Module Database (calc.fjk.ch/db/hp41mod.php) -- Time Module 1A/1B/1C/2C all XROM 26
- Fliegel, H.F. & Van Flandern, T.C. (1968), "A Machine Algorithm for Processing Calendar Dates", Communications of the ACM, 11(10):657
- Rust std::time documentation (doc.rust-lang.org) -- SystemTime, Instant, Duration, UNIX_EPOCH
- Codebase direct reads: state.rs, xrom.rs, modal.rs, app.rs, commands.rs, lib.rs, types.rs

### Secondary (MEDIUM confidence)
- Free42 documentation (thomasokken.com/free42/) -- date range Oct 15, 1582 to Sep 10, 4320; function subset confirmed
- HP-41CX finseth.com/hpdata/hp41cx.php -- CX-specific time functions confirmed
- DM41X User Manual (technical.swissmicros.com) -- modern Time Module emulation reference
- go41cx XYZALM Issues (forum.hp41.org) -- emulator alarm bugs (lessons learned)
- chrono 0.4.44 documentation (docs.rs/chrono) -- evaluated and rejected
- time 0.3.47 documentation (docs.rs/time) -- evaluated and rejected

### Tertiary (LOW confidence -- verify in Phase 38 Phase 0)
- HP 82182A Time Module Owner's Manual (00041-90035) -- alarm type encoding details, stopwatch keyboard exact mappings, CLOCK/SW programmability, CORRECT accuracy factor adjustment algorithm
- HP Museum Forum: Time Module Flags (hpmuseum.org/forum/thread-8569.html) -- Flag 31 and 44 usage details

---
*Research completed: 2026-05-24*
*Ready for roadmap: yes*
