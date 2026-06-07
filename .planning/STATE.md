---
gsd_state_version: 1.0
milestone: v4.3
milestone_name: Hardware Fidelity
status: executing
last_updated: "2026-06-07T08:40:37.835Z"
last_activity: 2026-06-07
progress:
  total_phases: 5
  completed_phases: 2
  total_plans: 12
  completed_plans: 11
  percent: 40
---

# Project State: HP-41 Calculator Emulator

---

## Project Reference

See: .planning/PROJECT.md (updated 2026-06-05 after v4.2 Help Search Enrichment)

**Core value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.

**Current focus:** Phase 64 — interactive-getkey

---

## Current Position

Phase: 64 (interactive-getkey) — EXECUTING
Plan: 5 of 5 (Plans 01-03, 05 complete; 64-04 GUI TS pending)
Status: Ready to execute next phase
Last activity: 2026-06-07

## Progress Bar

```
v4.3 Hardware Fidelity
Phase 62 ██████████ 100%  Phase 63 ██░░░░░░░░  25%
Phase 64 ░░░░░░░░░░   0%  Phase 65 ░░░░░░░░░░   0%
Phase 66 ░░░░░░░░░░   0%
Overall  ██░░░░░░░░  25%
```

| Phase | Goal | Status |
|-------|------|--------|
| 62 | Alarm Semantics Spec — verify `>` / `>>` prefix semantics against OM, lock behavioral contract | Complete ✓ (2026-06-06, da26a98) |
| 63 | Run-Loop Yield Engine + Interrupting Alarms + PSE/VIEW-AVIEW — synchronous pending-interrupt mechanism, alarm execution mid-run, display yields | Planned ✓ (6 plans / 4 waves; +GUI run loop) |
| 64 | Interactive GETKEY — suspend execution, await keypress, push row×col code to X, resume | Not started |
| 65 | Standalone Fidelity Fixes — CLI display_override, CHS mantissa sign-flip, AON auto-display, FACT(27..69) | Not started |
| 66 | Verification, Divergence-Doc Updates, Quality Gates — UNC-01/02/03 verified; D-40-04 closed; all gates green | Not started |

## Quick Tasks Completed (v4.3)

| # | Description | Date | Commit | Status | Directory |
|---|-------------|------|--------|--------|-----------|
| (pre-phase) | Desktop menu-bar single-instance guard + configurable macOS global hotkey (ADR-v4.3-001/002) | 2026-06-06 | 37dfd5f | Complete ✓ | — |

## Performance Metrics (v4.2 ship baseline)

| Metric | Target | Last measured (v4.2) |
|--------|--------|----------------------|
| Cold-start latency | <= 0.5 s | 2.2 ms (M1) |
| Key-press latency | <= 50 ms | ~65 ns/op |
| `hp41-core` line coverage | >= 95 % | >= 95 % |
| `hp41-core` region coverage | >= 93 % | >= 93 % |
| Numerical accuracy | >= 98 % | 98.86 % (843 cases) |
| Panics in `hp41-core` | 0 | 0 |
| Free42 contamination | 0 | 0 (18-token guard) |
| CI platforms | Win/macOS/Ubuntu | All green |
| Tests passing | — | 3371+ (v4.2 baseline) |

---
| Phase 63 P02 | 75 | 3 tasks | 7 files |
| Phase 63 P03 | 20 | 2 tasks | 1 file |
| Phase 63 P03 | 20min | 2 tasks | 1 files |
| Phase 63 P04 | 30min | 3 tasks | 6 files |
| Phase 63 P04 | 30 | 3 tasks | 6 files |
| Phase 63 P05 | 4 | 2 tasks | 2 files |
| Phase 63 P06 | 25 | 2 tasks | 2 files |
| Phase 64 P01 | 35 | 4 tasks | 7 files |
| Phase 64 P02 | 20 | 2 tasks | 1 file |
| Phase 64 P03 | 6 | 3 tasks | 5 files |
| Phase 64 P05 | 8 | 1 task | 1 file |

## Accumulated Context

### Decisions (Phase 64-05 — 2026-06-07)

- **64-05-D01:** D-CV-05 placed in `docs/hp41cv-divergences.md` — GETKEY is a HP-41CX OS built-in (not a module pac); CV doc is the correct home (same as X-MEM placement; PATTERNS.md confirms).
- **64-05-D02:** R/S keycode is 31 (row 3 col 1); CONTEXT.md D-02 value of "84" was a documentation error. 84 is the ENTER key's position (row 8, col 4). Corrected per Phase 64 research Pitfall 4 and orchestrator decision.

### Decisions (Phase 64-03 — 2026-06-07)

- **64-03-D01:** Permission TOML filename must be `<kebab-case>.toml` without `allow-` prefix — `check-tauri-permissions.sh` CI gate maps command `snake_case` → `kebab-case` filename; all existing permission files follow this naming (e.g. `resume-program.toml`, not `allow-resume-program.toml`).
- **64-03-D02:** Tauri v2 param ordering — `keycode: u8` (custom param) first, `State<'_, AppState>` last; required by Tauri v2 command macro; mirrors all other commands with custom params.
- **64-03-D03:** SC-4 invariant maintained — `resume_program_with_key` is exactly 5 lines: lock → core function call → drain print_buffer → drain event_buffer → `from_state`. No calculator logic in the GUI crate.

### Decisions (Phase 64-02 — 2026-06-07)

- **64-02-D01:** F5 (TUI R/S) returns `None` from `keycode_to_hp41_code` — ignored during WaitForKey. HP-41 code 31 is not wired in the CLI keyboard map; no special-casing introduced for R/S in the WaitForKey path.
- **64-02-D02:** `drain_pending_yields` owns crossterm events during WaitForKey inner loop. `handle_key` guard is belt-and-suspenders against future code-shape changes.

### Decisions (Phase 64-01 — 2026-06-07)

- **64-01-D01:** `resume_program_with_key` calls `op_getkey` inline before re-entering `run_loop` — the GetKey yield arm advances pc past GetKey on break; op_getkey must execute inline to push keycode to X before continuation steps run.
- **64-01-D02:** `resume_program_with_key` has no pc-at-end entry guard — op_getkey must run even when GETKEY is the last program step; early `Ok(())` return after op_getkey when pc >= program.len().
- **64-01-D03:** `resume_program_with_key` does NOT clear `pending_interrupt_alarm_index`/`pending_interrupt_depth` (differs from `resume_program` D-09 clear) — alarm-handler context must survive a GETKEY yield inside an interrupt frame.

### Decisions (Phase 63-06 — 2026-06-06)

- **63-06-D01:** R/S branch 3 (stopped, no modal) calls `run_program('A')` replacing `run_stop`; mirrors CLI F5 path (D-16/D-25.6).
- **63-06-D02:** yield-driver as `useEffect` on `calcState?.pending_yield`; `resumeScheduledRef` single-flight guard; no `get_state` polling (D-11).
- **63-06-D03:** `alarm:interrupting` silent-ignore arm removed — interrupting alarms execute server-side via Phase-C/63-02; `alarm:missing` toast added (D-07/D-08).
- **63-06-D04:** `vi.useFakeTimers({ shouldAdvanceTime: true })` in P1 test so `waitFor` polling still resolves while `advanceTimersByTime` controls yield-driver setTimeout.

### Decisions (Phase 63-04 — 2026-06-06)

- **63-04-D01:** Use full core path `hp41_core::ops::program::run_program` (not re-export `hp41_core::run_program`) to avoid name-shadow with `commands::run_stop`.
- **63-04-D02:** Drain pattern mirrors `handle_sst` (not `handle_get_state`) so print+event lines from a run reach the view.
- **63-04-D03:** Mutex held for full run_loop segment (T-63-09 accepted tradeoff) — identical to INTG/SOLVE/DIFEQ today; Phase-C check_alarms fires server-side.
- **63-04-D04:** `YieldView.kind` as lowercase string (`"pse"/"view"/"aview"`) so the TS layer needs no Rust enum knowledge.
- **63-04-D05:** `display_override` projection in `from_state` left completely untouched (D-04 / DISP-01 deferred).

### Decisions (Phase 63-03 — 2026-06-06)

- **63-03-D01:** Yield loop in `run()` not `handle_key` — `terminal: DefaultTerminal` borrow only available in `run()`; `handle_key` carries no terminal reference.
- **63-03-D02:** `entry_buf` as temporary display carrier during yield: priority 3 in `get_display_string`, always empty during program execution; avoids `ui.rs` modification (wave-3 file isolation).
- **63-03-D03:** Single commit for Tasks 1+2 — both target `app.rs`; splitting requires staged-hunk surgery with no correctness benefit.

### Decisions (Phase 63-02 — 2026-06-06)

- **63-02-D01:** Phase-C cadence: `steps==1 || steps.is_multiple_of(1000)` — first-step fires to convert pre-existing past-due alarms while `is_running=true`; then every 1000 steps for long-running programs.
- **63-02-D02:** PSE/VIEW/AVIEW in `run_loop` set `pending_yield + break` without writing `display_override` (D-04/DISP-01 deferred to v4.4); interactive dispatch path in `execute_op` unchanged.
- **63-02-D03:** `ack-after-RTN` gate: `pending_interrupt_depth` (call_stack.len() at injection) compared on `Op::Rtn` after pop — correctly identifies handler-fully-returned even across nested XEQs in handler; cap-drop/missing-label paths pre-clear `alarm_index`, so `is_some()` naturally skips ack.
- **63-02-D04:** Scenario 2 test redesigned to be timing-independent — pre-loaded stack before `run_program`, `StoReg`-only handler (no push) proves interrupt fired and stored correct X without depending on specific interrupt step number.

### Decisions (Phase 63-01 — 2026-06-06)

- **63-01-D01:** `YieldKind/YieldState/PSE_RESUME_MS` named as RESEARCH recommended; all three yield kinds share 1000 ms (single knob, no perceptible gain from shorter "brief" constant).
- **63-01-D02:** `pending_interrupt_depth: Option<usize>` declared in state.rs (63-01) not program.rs (63-02) to maintain wave-2 file isolation — 63-02 reads/writes it without touching state.rs.
- **63-01-D03:** `dispatch_alarm_event` gains `defer_to_run_loop: bool`; `check_alarms` passes `true` (defers to run_loop); `op_almnow` passes `false` (owns synchronous ack — avoids double-ack/stale-index hazard).

### Decisions (Phase 62 locked + pre-resolved from research/audit)

- **ALARM-01 resolved — code CONFIRMED CORRECT:** `>>label` = Interrupting Control Alarm (interrupts a running program); `>label` = Conditional/Noninterrupting Alarm (fires only when idle/off). Confirmed by HP 82182A QRC 82182-90002 (1981) and HP-41CX QRG 00041-90475 (1983). Current `parse_alarm_type` (`>>` → `interrupting: true`) is correct. ADR: `docs/adr/v4.3-003-alarm-prefix-semantics.md`.
- **D-05 not triggered:** Conditional correction plan does not apply (code not inverted). Flip-vs-rename delegated to Phase 63 planner. Three constraints documented in ADR for record.
- **D-07 honored:** `docs/hp41-time-divergences.md §D-40-04` not updated in Phase 62. Phase 66 success criterion.

### Pre-phase decisions

- **Synchronous, single-threaded re-entrancy** — the run-loop interrupt mechanism reuses the existing call-stack / Xeq / PROMPT-resume machinery. No threads, no `Arc<Mutex>` rework, no new `Op` variants. A single transient `pending_interrupt: Option<String>` field on `CalcState` with `#[serde(default, skip)]` is the wire.
- **ALARM-01 gates ALARM-02/03** — Phase 62 is a spec/clarify-only phase (no runtime code). No implementation of interrupting alarms begins until the `>`/`>>` prefix inversion question is resolved against the primary OM.
- **PRGM-01 + PRGM-02 share the engine with ALARM-02/03** — PSE timing (FGAP-01) and VIEW/AVIEW mid-run (FGAP-10) use the same yield primitive as the interrupt mechanism. They live in Phase 63.
- **PRGM-03 (GETKEY) is its own phase** — heavier (L-effort); requires suspending execution and awaiting frontend input, building on PROMPT suspend/resume. Depends on Phase 63 infrastructure being in place.
- **DISP-01/02/03 + MATH-01 are independent** — no engine dependency; can be started after Phase 62 spec is locked (do not require Phase 63 to be complete). Grouped as Phase 65.
- **VERIFY-01 is the final gate phase** — fold UNC-01/02/03 verification-and-fix-if-confirmed plus divergence doc updates and all quality gates into Phase 66.
- **math1/ freeze holds** — all alarm/interrupt code touches `time/alarm.rs`, `ops/program.rs`, `state.rs` — none of which are inside the frozen `math1/` directory.
- **Zero new runtime deps** — no new crates in `hp41-core/Cargo.toml`.
- **serde discipline** — transient fields (interrupt state, pending key) carry both `#[serde(default)]` and `#[serde(skip)]`; any persistent interrupt-related field follows the `rand_seed` two-annotation pattern with explicit inline documentation.
- **Deterministic-clock testing** — `trigger_unix = 0` (always past-due) and `time_offset_secs = 0`; never call `SystemTime::now()` in assertions.
- **CLI-GUI parity (D-25.6)** — both CLI (`drain_event_buffer` at app.rs:1797) and GUI (`tick_time` in commands.rs) must handle the new interrupting alarm event in the same phase (Phase 63).

### Pitfalls to watch

| ID | Description | Phase |
|----|-------------|-------|
| P-43-01 | `>` / `>>` prefix inversion — audit code's `interrupting: true` flag against OM before any implementation | 62 |
| P-43-02 | `run_loop` regression surface — 3300+ tests exercise the engine; any change to `run_loop` / `run_program` can silently break ISG/DSE, call-stack, or XROM dispatch | 63 |
| P-43-03 | `is_running` + 4-level cap integrity — interrupt frame must never push to a 5th level; `is_running` must reset on all paths including errors | 63 |
| P-43-04 | `lift_enabled` preservation — interrupted program's stack-lift state must be exactly restored on resume | 63 |
| P-43-05 | Print emulation — any alarm label program output (PRX/PRA/PRSTK) routes through `state.print_buffer`; `drain_and_show_print_output()` must be called after alarm execution | 63 |
| P-43-06 | MSRV 1.88 clippy divergence — run `cargo +1.88 clippy --workspace --all-targets --all-features -- -D warnings` before tagging; check `gh pr checks` | 66 |
| P-43-07 | GUI clippy ungated — manually run `cd hp41-gui/src-tauri && cargo clippy --all-targets -- -D warnings` after any GUI Rust changes | 63/66 |
| P-43-08 | `#[cfg(mobile)]` blind spot — after any change to `commands.rs` alarm handling, run `cargo check --target aarch64-apple-ios --manifest-path hp41-gui/src-tauri/Cargo.toml` | 63 |
| P-43-09 | DNT-05 regression — non-interrupting `>label` path must remain on the event_buffer / `drain_event_buffer` path unchanged | 63 |
| P-43-10 | CHS during EEX entry vs. mantissa entry — only fix mantissa-entry sign-flip; CHS during EEX flushes (hardware behavior, accepted) | 65 |

### Blockers

None. Phase 62 can start immediately.

### Pending Todos

- Run `/gsd-execute-phase 63` to execute Phase 63 (6 plans, 4 waves; plan-checker PASS).

### Phase 63 planning note — GUI run-loop scope expansion (2026-06-06)

The plan-checker discovered (verified against `develop`) that **the GUI has no continuous program run loop**: `handle_run_stop` only toggles `is_running`; there is zero `run_program`/`run_loop`/`resume_program` call in `hp41-gui/src-tauri/src/`. The GUI runs single ops, SST/BST steps, and the long math-pac ops (INTG/SOLVE/DIFEQ) that loop inside one `dispatch_op` — it never continuously runs a user-LBL program. So the yield engine (PSE/VIEW/AVIEW arms + interrupt check + Phase-C), which lives inside `run_loop`, was unreachable from the GUI → criteria 1/4/5 unmet GUI-side. **User decision: build the GUI run loop in Phase 63** (not defer, not a separate phase). Plans split the GUI work into **63-04 (GUI-Rust: thin `run_program`/`resume_program` Tauri commands + `pending_yield` projection, SC-4 glue-only)** and **63-06 (GUI-TS: App.tsx yield-and-resume R/S run-loop driver, D-11 no-polling)**. Phase 63 now also delivers GUI continuous program execution — a capability the GUI never had. This is downstream-relevant for Phase 64 (GETKEY) which builds on the same suspend/resume infrastructure.

---

## Deferred Items

| Category | Item | Status |
|----------|------|--------|
| Deferred to v4.4 | CATSCROLL-01: Interactive CATALOG 1 scroll (FGAP-06) | L-effort; same run_loop-yield family; keeps v4.3 focused |
| Deferred to v4.4 | XMEMFILE-01: PURFL/CLFL + DUP FL + bbb.eee (FGAP-09) | L-effort structural; DNT-06/07 accepted for v4.3 |
| Deferred | App Store submission (STORE-01, STORE-02) | Handled externally by Daniel |
| Deferred | iPad universal layout (IPAD-01) | v4.4+ milestone |
| Deferred | Landscape orientation (LAND-01) | v4.4+ milestone |
| Deferred | Android (ANDROID-01) | Parked as SEED-001 |
| Deferred | .raw file picker on iOS (RAW-IOS-01) | v4.4+ (no native iOS picker in tauri-plugin-dialog) |
| Deferred | HSCOV-01 missed-query log (zero-result queries) | v2/deferred, out of v4.3 scope |

---

*State initialized: 2026-05-06*
*Last updated: 2026-06-06 — Phase 63 COMPLETE: all 6 plans done (4 waves). GUI run-loop fully wired: run_program/resume_program Tauri commands + pending_yield projection (63-04) + TS yield-and-resume driver + R/S 4-way routing + alarm:missing toast (63-06). 129 GUI Rust + 340 GUI TS tests pass. Next: Phase 64 (GETKEY).*

## Operator Next Steps

- Run `/gsd-new-phase` or `/gsd-plan-phase 64` to start Phase 64 (Interactive GETKEY)
