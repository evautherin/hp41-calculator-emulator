---
gsd_state_version: 1.0
milestone: v4.3
milestone_name: Hardware Fidelity
status: ready_to_plan
last_updated: 2026-06-06T10:44:32.747Z
last_activity: 2026-06-06 -- Phase 62 Plan 01 complete (ADR v4.3-003 authored)
progress:
  total_phases: 5
  completed_phases: 1
  total_plans: 1
  completed_plans: 1
  percent: 20
stopped_at: Phase 62 complete (1/1) — ready to discuss Phase 63
---

# Project State: HP-41 Calculator Emulator

---

## Project Reference

See: .planning/PROJECT.md (updated 2026-06-05 after v4.2 Help Search Enrichment)

**Core value:** Faithful HP-41 RPN fidelity — four-level stack, stack-lift semantics, display, and keystroke programming must behave identically to original hardware; everything else is secondary.

**Current focus:** Phase 63 — run loop yield engine + interrupting alarms + pse/view aview

---

## Current Position

Phase: 63
Plan: Not started
Status: Ready to plan
Last activity: 2026-06-06

## Progress Bar

```
v4.3 Hardware Fidelity
Phase 62 ██████████ 100%  Phase 63 ░░░░░░░░░░   0%
Phase 64 ░░░░░░░░░░   0%  Phase 65 ░░░░░░░░░░   0%
Phase 66 ░░░░░░░░░░   0%
Overall  ██░░░░░░░░  20%
```

| Phase | Goal | Status |
|-------|------|--------|
| 62 | Alarm Semantics Spec — verify `>` / `>>` prefix semantics against OM, lock behavioral contract | Complete ✓ (2026-06-06, da26a98) |
| 63 | Run-Loop Yield Engine + Interrupting Alarms + PSE/VIEW-AVIEW — synchronous pending-interrupt mechanism, alarm execution mid-run, display yields | Not started |
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

## Accumulated Context

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

- Run `/gsd-plan-phase 63` to plan Phase 63: Run-Loop Yield Engine + Interrupting Alarms.

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
*Last updated: 2026-06-06 — Roadmap created for v4.3 Hardware Fidelity. 5 phases (62–66), 11 requirements mapped. Next: `/gsd-plan-phase 62`.*

## Operator Next Steps

- Run `/gsd-plan-phase 62` to plan Phase 62: Alarm Semantics Spec
