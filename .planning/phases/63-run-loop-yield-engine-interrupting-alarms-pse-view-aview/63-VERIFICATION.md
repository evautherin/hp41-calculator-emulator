---
phase: 63-run-loop-yield-engine-interrupting-alarms-pse-view-aview
verified: 2026-06-06T12:00:00Z
status: human_needed
score: 4/5
overrides_applied: 0
re_verification: false
human_verification:
  - test: "Trigger a running program with PSE and observe the CLI display"
    expected: "Display shows X register value for ~1 second, then next step executes; visible pause"
    why_human: "std::thread::sleep and terminal redraw cannot be asserted programmatically without real UI"
  - test: "Trigger a running program with VIEW RNN and AVIEW and observe both CLI and GUI displays"
    expected: "CLI: display flips to register/ALPHA value for ~1 second; GUI: main display shows pending_yield.text for resume_ms before next step"
    why_human: "Requires visual confirmation that yield text is actually rendered in the TUI/WebView"
  - test: "Set a repeating interrupting alarm (>>LABEL), run a program, verify it fires mid-run and resumes"
    expected: "Program pauses, alarm handler executes (e.g. stores a value), program resumes from exact halted step; alarm reschedules for next fire"
    why_human: "Real-time alarm timing and mid-run visual inspection requires human observation"
  - test: "Set an interrupting alarm (>>LABEL) while idle (no program running), verify both CLI and GUI handle it"
    expected: "CLI: run_program is called for the label (app.rs:1850 path), program executes normally. GUI: dispatch_op is called (existing path — user LBL programs silently fail with InvalidOp toast)"
    why_human: "SC-3 has a confirmed CLI-GUI parity gap for the idle-alarm path; human must decide whether the GUI behavior (pre-existing limitation kept unchanged per ALARM-03) is acceptable for phase pass"
  - test: "Trigger an idle alarm on the CLI where the alarm label is a user LBL program containing PSE, verify PSE renders"
    expected: "The alarm:xeq: CLI path calls run_program then drain_pending_yields; PSE pause renders"
    why_human: "WR-04: drain_event_buffer does not call drain_pending_yields after alarm:xeq run_program — alarm-launched CLI programs with PSE are stranded mid-yield until next keypress. Human must verify severity and decide if this is a PRGM-01 blocker or acceptable follow-up"
gaps:
  - truth: "ADR artifact path matches plan spec (docs/adr/v4.3-001-interrupt-alarm-pending-field.md)"
    status: partial
    reason: "ADR was created as v4.3-004-interrupt-alarm-pending-field.md (v4.3-001 through 003 were already taken). Content is correct and consistent — references pending_interrupt and v4.3-003. The divergences doc cross-references v4.3-004 correctly. Functional delivery achieved; naming deviates from plan."
    artifacts:
      - path: "docs/adr/v4.3-001-interrupt-alarm-pending-field.md"
        issue: "MISSING at this path — exists at docs/adr/v4.3-004-interrupt-alarm-pending-field.md"
    missing:
      - "Either rename v4.3-004 to v4.3-001 (breaks any existing cross-reference that uses v4.3-004) or update the plan artifact spec to v4.3-004. Recommend updating the spec since v4.3-004 is consistent throughout the codebase."
---

# Phase 63: Run-Loop Yield Engine + Interrupting Alarms + PSE/VIEW-AVIEW — Verification Report

**Phase Goal:** Build the synchronous pending-interrupt mechanism in run_loop; implement interrupting control alarm execution and PSE/VIEW-AVIEW mid-run display yields (CLI + GUI).
**Verified:** 2026-06-06T12:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (Roadmap Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|---------|
| SC-1 | A running program is interrupted at the next instruction boundary; the alarm's stored label program executes in the interrupted program's register environment; the original program resumes from exactly where it was halted | VERIFIED | `run_loop` takes `pending_interrupt` before pc-fetch (program.rs:520-544); pushes pc onto call_stack (synthetic XEQ frame); `Op::Rtn` pop restores pc; 12 unit tests green in `phase_63_interrupting_alarms.rs` |
| SC-2 | An interrupting alarm is suppressed (left past-due) and not executed when the call stack is already at the 4-level cap; call_stack never reaches 5 levels; is_running correctly restored on all paths | VERIFIED | program.rs:521-526: `if call_stack.len() >= 4` → silent suppress, clear index/depth, continue. Cap-drop test `interrupt_blocked_when_call_stack_at_4_level_cap` green. is_running reset in `run_program` via captured `let result = run_loop(...)` pattern (line 455). Matches ALARM-03 spec and CONTEXT.md D-07. |
| SC-3 | An idle-fired interrupting alarm (no program running) executes the alarm label program via the existing run_program path without any resume logic | UNCERTAIN (CLI verified; GUI pre-existing gap) | CLI: `drain_event_buffer` (app.rs:1850) calls `hp41_core::run_program` for `alarm:xeq:` events — verified. GUI: `App.tsx:1237` calls `invoke('dispatch_op', { keyId: 'xeq_${label}' })` — this is `Op::Xeq` which does NOT run user LBL programs (`op_xeq` with `is_running==false` only checks builtin_card_op / xrom_resolve; see program.rs:69-84 comment). This was the EXISTING GUI path before Phase 63 and ALARM-03 says "continue via existing path, unchanged." The ROADMAP SC-3 says "via the existing `run_program` path" which CLI satisfies but GUI does not. Conflicting signals — needs human decision. |
| SC-4 | PSE during a running program pauses the display for approximately 1 second (visible in both CLI and GUI) before the next program step executes | VERIFIED (automated) / HUMAN NEEDED (visual) | Core: `Op::Pse` arm in run_loop sets `pending_yield{kind:Pse, text:format_hpnum(X), resume_ms:1000}` and breaks (program.rs:790-799). CLI: `drain_pending_yields` renders `yield_text` via `entry_buf`, sleeps `resume_ms`, calls `resume_program` (app.rs:322-358). GUI: `useEffect` on `pending_yield` schedules `invoke('resume_program')` via `setTimeout(resume_ms)` (App.tsx:592-606). Tests: `pse_mid_run_breaks_and_records_resume_ms`, `pse_resume_continues_to_next_step` green. Visual render needs human observation. |
| SC-5 | VIEW and AVIEW during a running program display their register/ALPHA value briefly before the program continues — not only after the program ends | VERIFIED (automated) / HUMAN NEEDED (visual) | Core: `Op::View(reg)` and `Op::AView` arms set `pending_yield{kind:View/Aview, text, resume_ms:1000}` and break without writing `display_override` (program.rs:800-831). Same frontend wiring as SC-4. Tests: `view_mid_run_captures_formatted_register_into_yield`, `aview_mid_run_captures_alpha_into_yield` green. |

**Score:** 4/5 truths verified (SC-3 UNCERTAIN — needs human decision)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `hp41-core/src/state.rs` | pending_interrupt/alarm_index/depth + pending_yield + YieldState/YieldKind + serde(skip) | VERIFIED | Lines 490-531: all 4 fields with `#[serde(default, skip)]`; YieldKind/YieldState types at lines 57-86; PSE_RESUME_MS=1000. Serde round-trip tests at lines 820-878. |
| `hp41-core/src/ops/time/alarm.rs` | dispatch_alarm_event: interrupting-arm routing + D-10 demotion | VERIFIED | Lines 535-558: if `interrupting && defer_to_run_loop && is_running && pending.is_none() && !solver/modal` → sets `pending_interrupt`. Else → `alarm:xeq:{label}` to event_buffer. check_alarms passes `defer_to_run_loop=true`; op_almnow passes `false`. |
| `hp41-core/tests/phase_63_interrupting_alarms.rs` | 12 interrupt tests covering all edge cases | VERIFIED | 666-line file; 12 `#[test]` functions covering: halts-and-resumes, stack preservation, 4-level cap, nesting blocked, idle regression, non-interrupting unchanged, message unchanged, solver/modal demotion, missing label, cleared-on-resume, backward-compat, repeating reschedule. |
| `hp41-core/src/ops/program.rs` | run_loop interrupt check + Phase-C + ack-after-RTN + PSE/VIEW/AVIEW yield arms + clear-on-entry | VERIFIED | Interrupt check: lines 520-544. Phase-C: lines 513-515 (`steps==1 \|\| steps.is_multiple_of(1000)`). Ack-after-RTN: lines 567-576. PSE arm: 790-799. VIEW arm: 800-818. AVIEW arm: 822-831. Clear-on-entry: run_program 449-452, resume_program 482-485. |
| `hp41-core/tests/phase_63_yield_engine.rs` | 4 yield tests for PSE/VIEW/AVIEW | VERIFIED | 269-line file; 4 `#[test]` functions all matching plan spec. |
| `hp41-cli/src/app.rs` | yield render+sleep+resume loop; alarm:missing arm; alarm:interrupting removed | VERIFIED | `drain_pending_yields` at lines 322-358 loops while pending_yield.is_some(); `drain_event_buffer` adds alarm:missing arm (line 1861); `grep "alarm:interrupting" app.rs` → empty. |
| `hp41-gui/src-tauri/src/commands.rs` | run_program + resume_program Tauri commands | VERIFIED | Lines 397-427: `run_program` command calls `hp41_core::ops::program::run_program`; `resume_program` calls `hp41_core::ops::program::resume_program`. Both return `CalcStateView`. |
| `hp41-gui/src-tauri/src/types.rs` | pending_yield projected in CalcStateView | VERIFIED | Line 152: `pub pending_yield: Option<YieldView>`; projection at lines 265-268 via `YieldView::from_yield_state`. |
| `hp41-gui/src-tauri/src/lib.rs` | both commands registered in generate_handler! | VERIFIED | Line 261-263: `run_program` and `resume_program` in `generate_handler!`. |
| `hp41-gui/src-tauri/permissions/run-program.toml` | permission TOML | VERIFIED | File exists (179B). |
| `hp41-gui/src-tauri/permissions/resume-program.toml` | permission TOML | VERIFIED | File exists (188B). |
| `hp41-gui/src-tauri/capabilities/default.json` | permissions referenced | VERIFIED | Lines 28-29: `allow-run-program` and `allow-resume-program`. |
| `hp41-gui/src/App.tsx` | R/S 4-way routing; yield-and-resume driver; alarm:missing toast; alarm:interrupting removed; pending_yield in CalcStateView interface | VERIFIED | R/S 4-way at lines 127-144. useEffect driver at lines 592-606. alarm:missing at line 1241-1243. `grep "alarm:interrupting" App.tsx` → empty. `pending_yield` interface at line (CalcStateView). |
| `hp41-gui/src/App.test.tsx` | Vitest: yield render+resume, alarm:missing, R/S start | VERIFIED | Tests added per plan. All 340 TS tests pass per verification_context. |
| `docs/adr/v4.3-001-interrupt-alarm-pending-field.md` | ADR for interrupt mechanism | PARTIAL (wrong filename) | EXISTS at `docs/adr/v4.3-004-interrupt-alarm-pending-field.md`. Content verified: references `pending_interrupt`, `v4.3-003`, describes synthetic XEQ frame, 4-level cap, D-10 demotion, ack-after-RTN. Named v4.3-004 because v4.3-001/002/003 were already allocated. |
| `docs/hp41-time-divergences.md` | §D-40-04 flipped to implemented | VERIFIED | Line 265: "IMPLEMENTED in Phase 63 (v4.3). Verified in Phase 66." References `v4.3-004` ADR at line 320. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| run_loop interrupt check | find_in_program + call_stack.push | synthetic XEQ frame; captures pending_interrupt_depth | VERIFIED | program.rs:532-534: `pending_interrupt_depth = Some(call_stack.len()); call_stack.push(state.pc); pc = target+1` |
| Op::Rtn pop | acknowledge_alarm | ack after handler returns to pending_interrupt_depth | VERIFIED | program.rs:567-576: guard on `pending_interrupt_alarm_index.is_some() && Some(call_stack.len()) == pending_interrupt_depth`; calls `time::alarm::acknowledge_alarm(state, idx)` |
| Op::Pse / Op::View / Op::AView run_loop arms | state.pending_yield | set YieldState + break (no display_override) | VERIFIED | program.rs:790-831: all three arms set `pending_yield = Some(YieldState{...})` and `break`. `display_override` not touched. |
| R/S keystroke handler (CLI) | resume_program | while pending_yield.is_some(): render text, sleep, resume | VERIFIED | app.rs:289 calls `drain_pending_yields` after `handle_key`; drain_pending_yields loop at 322-358 |
| drain_event_buffer alarm:missing arm | self.message | alarm:missing → status line | VERIFIED | app.rs:1861-1864: `strip_prefix("alarm:missing:")` → `self.message = Some(format!(...))` |
| R/S keystroke (GUI) | run_program command | 4-way: modal > cancel > stop > start | VERIFIED | App.tsx:127-144: `invoke<CalcStateView>('run_program', { label: 'A' })` in branch 3 |
| yield driver (GUI) | resume_program command | setTimeout(pending_yield.resume_ms) → invoke resume_program | VERIFIED | App.tsx:592-606: `useEffect` schedules `invoke('resume_program')` after `resume_ms` |
| event consumer (GUI) | showToast | alarm:missing arm | VERIFIED | App.tsx:1241-1243 |
| hp41-time-divergences §D-40-04 | ADR v4.3-004 | cross-reference | VERIFIED (v4.3-004 not v4.3-001) | Line 320 references `v4.3-004-interrupt-alarm-pending-field.md` |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `run_loop` PSE arm | `state.pending_yield.text` | `format_hpnum(&state.stack.x, &state.display_mode)` | Yes — real X register | FLOWING |
| `run_loop` VIEW arm | `state.pending_yield.text` | `format_hpnum(&regs[reg].numeric_or_zero(), &state.display_mode)` | Yes — real register | FLOWING |
| `run_loop` AVIEW arm | `state.pending_yield.text` | `alpha_reg.chars().take(24).collect()` | Yes — real ALPHA | FLOWING |
| `CalcStateView.pending_yield` | `YieldView` | `state.pending_yield.as_ref().map(YieldView::from_yield_state)` | Yes — real core state | FLOWING |
| `drain_pending_yields` (CLI) | yield text render | `state.pending_yield.as_ref().expect(...)text.clone()` into `entry_buf` | Yes — real yield text | FLOWING |
| `useEffect` driver (GUI) | `calcState.pending_yield` | Tauri IPC return from `run_program`/`resume_program` | Yes — real CalcStateView | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| State fields are serde(skip) | `grep -n "serde(default, skip)" state.rs` around pending_* fields | Lines 498,506,522,530 all have `#[serde(default, skip)]` | PASS |
| No alarm:interrupting silent-ignore in CLI | `grep -n "alarm:interrupting" hp41-cli/src/app.rs` | 0 matches | PASS |
| No alarm:interrupting silent-ignore in GUI | `grep -n "alarm:interrupting" hp41-gui/src/App.tsx` | 0 matches | PASS |
| PSE/VIEW/AVIEW yield arms in run_loop | `grep -n "pending_yield = Some" program.rs` | 3 matches (lines 793, 812, 825) | PASS |
| ack-after-RTN gate | `grep -n "acknowledge_alarm" program.rs` | Line 573 inside RTN arm, gated on depth check | PASS |
| D-40-04 flipped in divergences doc | `grep -n "IMPLEMENTED" docs/hp41-time-divergences.md` | "IMPLEMENTED in Phase 63 (v4.3)" | PASS |
| No unwrap() in production program.rs code | `grep -n "unwrap()" program.rs \| grep -v test` | All occurrences are in `#[cfg(test)]` section | PASS |
| math1/ freeze | `git diff HEAD~15..HEAD -- hp41-core/src/ops/math1/` | 1 line (whitespace-only context) | PASS |

### Probe Execution

Step 7c: SKIPPED — no probe-*.sh files in scripts/tests/; phase is not a migration/tooling phase. CI pass confirmed per verification_context: `just ci` green (all tests, coverage 95.09% lines, license-audit), `just gui-ci` green (129 Rust + 340 TS tests).

### Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|---------|
| ALARM-02 | 63-01, 63-02, 63-03, 63-04, 63-06 | Control alarm interrupts running program at next boundary; handler runs in interrupted register env; resumes at halted pc | SATISFIED | `run_loop` interrupt injection verified; 12 core tests green; CLI R/S and GUI R/S paths verified |
| ALARM-03 | 63-01, 63-02, 63-03, 63-04, 63-06 | 4-level cap honored; idle-fired via existing path; repeating reschedules after RTN | PARTIAL (CLI satisfied; GUI idle-path limitation) | Cap-drop silent-suppress verified; ack-after-RTN verified; CLI idle→run_program verified; GUI idle→dispatch_op (pre-existing, kept per "unchanged" clause) |
| PRGM-01 | 63-02, 63-03, 63-04, 63-06 | PSE pauses display ~1 second in CLI and GUI | SATISFIED (primary path; secondary CLI alarm:xeq path incomplete) | PSE yield arm verified; CLI drain_pending_yields at R/S call site verified; GUI useEffect driver verified; alarm:xeq → run_program CLI path lacks drain_pending_yields (WR-04) |
| PRGM-02 | 63-02, 63-03, 63-04, 63-06 | VIEW/AVIEW display value mid-run in CLI and GUI | SATISFIED (same caveat as PRGM-01) | VIEW/AVIEW yield arms verified; same frontend wiring as PSE |

### Code Review Findings — Phase Scope Assessment

| Finding | Severity (Reviewer) | Phase Scope | Disposition |
|---------|--------------------|-----------|-----------| 
| CR-01: GUI alarm:xeq uses dispatch_op instead of run_program | BLOCKER | OUT-OF-SCOPE / PRE-EXISTING | ALARM-03 requirements text: "continue via existing path, unchanged." The GUI's existing pre-Phase-63 path was `dispatch_op`; Phase 63 did not change it per requirement. This is a documented pre-existing GUI limitation (the GUI had no run_program command before Phase 63). The ROADMAP SC-3 wording "existing `run_program` path" is ambiguous — the requirements text takes precedence for scoping. Recommended follow-up: route alarm:xeq through run_program in a future phase. |
| CR-02: positional alarm index can be stale if handler mutates alarm table | BLOCKER | NARROW OUT-OF-SCOPE | ALARM-02/03 cover the common case (handler does not touch alarms). The edge case (handler runs XYZALM/CLALMA) is not explicitly specified. The common repeating-alarm case works correctly. Recommend follow-up: add stable-key ack or re-find-at-RTN approach for robustness. |
| WR-01: cap-drop repeating alarm permanently lost | WARNING | OUT-OF-SCOPE | ALARM-03 explicitly specifies "suppressed and left past-due at depth 4". CONTEXT.md D-07/D-08 confirm cap-drop stays fully silent by design. Not a phase defect. |
| WR-02: second simultaneous past-due alarm not rescheduled | WARNING | FOLLOW-UP | Not an explicitly required test scenario. The single interrupting alarm case (the common case) works correctly. Follow-up for future phase. |
| WR-03: GUI yield-window allows user input to corrupt paused program | WARNING | FOLLOW-UP | No requirement prohibits key input during PSE. Real HP-41 behavior: running program owns the keyboard during pause. Follow-up for D-25.6 parity in a future phase. |
| WR-04: CLI alarm:xeq run_program call site lacks drain_pending_yields | WARNING | PARTIAL IN-SCOPE | Plan 63-03 Task 1 says "R/S keystroke handler's `run_program` call site (**and any other site** that runs a program to completion)." The `drain_event_buffer` alarm:xeq path (app.rs:1850) does not call `drain_pending_yields`. Alarm-launched CLI programs containing PSE/VIEW/AVIEW will have their yield stranded until the next keypress. Partially in scope; the "any other site" clause was not fully implemented. |
| WR-05: interrupt handler shares RPN stack | WARNING | OUT-OF-SCOPE | CONTEXT.md explicitly notes this: "alarm handlers must STO/RCL defensively — Risk 2." Documented accepted design choice. |
| IN-01: MAX_STEPS guard resets per resume segment | INFO | OUT-OF-SCOPE | Informational; documented behavior. |
| IN-02: D5 GUI test asserts IPC shape not execution | INFO | FOLLOW-UP | After CR-01 is addressed in a future phase, update the test. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| program.rs | 502 | `return Err(HpError::Overflow)` comment says "CR-01" | Info | Comment references code review finding in the guard comment — harmless but references internal doc artifact |

No TBD, FIXME, or XXX markers found in any phase-modified file. No unwrap() in production code paths (only in test sections per CLAUDE.md allowance). No println!/eprintln! in hp41-core.

### Human Verification Required

#### 1. Visual PSE Pause — CLI

**Test:** Run a program with PSE (`LBL A; PushNum(42); PSE; Rtn`) via R/S (F5 or key `a` in RUN mode). Observe the CLI display.
**Expected:** The display shows the X register value for approximately 1 second, then the program continues. The pause is visually apparent (not instant).
**Why human:** `std::thread::sleep(1000ms)` executes correctly per code review but the terminal redraw via `entry_buf` trick requires visual confirmation that the display actually flips to the yield text.

#### 2. Visual VIEW/AVIEW Mid-Run — GUI

**Test:** Run a program with `VIEW 00` or `AVIEW` via R/S in the GUI. Observe the calculator display.
**Expected:** The GUI main display shows `pending_yield.text` (formatted register value or ALPHA content) for ~1 second before continuing.
**Why human:** The `useEffect` setTimeout driver is verified in unit tests with fake timers, but actual visual rendering in the WebView/Tauri window requires observation.

#### 3. Interrupting Alarm Mid-Run — Real Timing

**Test:** Set an alarm with `>>MYPROG` label (interrupting), start a long-running loop program, wait for the alarm time. Observe that the alarm program executes mid-run and the original resumes.
**Expected:** Original program halts, MYPROG executes, original program resumes at the exact halted step. For a repeating alarm, it fires again on the next trigger time.
**Why human:** Real-time alarm timing and the mid-run interrupt boundary require human observation to confirm the behavior matches the spec at real execution speed.

#### 4. SC-3 GUI Idle-Alarm Path — Human Decision Required

**Test:** Set an interrupting control alarm (`>>MYPROG`) with no program running. Wait for it to fire. Observe the GUI behavior.
**Expected (current behavior):** GUI calls `dispatch_op(xeq_MYPROG)` → `Op::Xeq` → tries builtin/XROM, fails with `InvalidOp` toast for any user LBL. The program does NOT run.
**Expected (correct behavior per SC-3):** Alarm label program should execute via `run_program`.
**Why human:** There is a conflict between ALARM-03 ("continue via existing path, unchanged") and ROADMAP SC-3 ("existing `run_program` path"). The developer must decide: (a) accept the GUI idle-alarm limitation as out-of-scope per ALARM-03's "unchanged" clause and add a follow-up issue, or (b) classify this as a BLOCKER requiring a fix before the phase is closed.

#### 5. WR-04 Severity Decision — CLI Alarm-Launched PSE

**Test:** Set a non-interrupting alarm (`>MYPROG`) with a program containing PSE. Wait for idle alarm to fire. Observe CLI behavior.
**Expected (if WR-04 is a blocker):** PSE render + sleep + resume should happen immediately when the alarm fires.
**Actual (current behavior):** PSE yield is set but `drain_pending_yields` is not called from `drain_event_buffer`. The yield will be stranded until the next keypress.
**Why human:** The developer must decide: (a) accept this as a follow-up (alarm-launched programs with PSE are a secondary scenario), or (b) require `drain_pending_yields` to be added to `drain_event_buffer` before the phase closes.

### Gaps Summary

Two findings require developer decisions before this phase can be marked `passed`:

**Gap 1 — ADR naming deviation (minor):** The ADR was created as `docs/adr/v4.3-004-interrupt-alarm-pending-field.md` instead of `v4.3-001` (the plan spec). Content is correct and all cross-references in the codebase consistently use `v4.3-004`. The plan's must_haves artifact path check fails. Recommended resolution: update the plan/requirement spec to acknowledge v4.3-004 as the canonical filename (no code change needed).

**Gap 2 — SC-3 GUI idle-alarm path (requires human decision):** The GUI routes `alarm:xeq:` through `dispatch_op` (pre-existing behavior kept per ALARM-03 "unchanged" clause). This means user LBL programs fail silently on idle alarm fire in the GUI. The ROADMAP SC-3 says "via the existing `run_program` path." The developer must decide whether the existing GUI limitation satisfies ALARM-03's "unchanged" requirement or whether SC-3 demands a fix. If the latter, route `alarm:xeq:` through `invoke('run_program', { label })` in `App.tsx:1233-1240`.

**Gap 3 — WR-04 CLI alarm:xeq drain (requires human decision):** The `drain_event_buffer` alarm:xeq path calls `run_program` but not `drain_pending_yields`. If the alarm-launched program has PSE/VIEW/AVIEW, the yield is stranded. The developer must decide whether this is a PRGM-01 blocker or an acceptable follow-up (the primary R/S path works correctly).

---

_Verified: 2026-06-06T12:00:00Z_
_Verifier: Claude (gsd-verifier)_
