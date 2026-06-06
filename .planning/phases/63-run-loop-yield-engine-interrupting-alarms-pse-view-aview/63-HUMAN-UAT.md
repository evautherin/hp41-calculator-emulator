---
status: partial
phase: 63-run-loop-yield-engine-interrupting-alarms-pse-view-aview
source: [63-VERIFICATION.md]
started: 2026-06-06T12:00:00Z
updated: 2026-06-06T12:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. CLI PSE pause renders
expected: Running a program with PSE shows the X-register value for ~1 second, then the next step executes — a visible pause in the TUI.
result: [pending]

### 2. VIEW/AVIEW brief display (CLI + GUI)
expected: CLI display flips to the register/ALPHA value for ~1 s; GUI main display shows `pending_yield.text` for `resume_ms` before the next step.
result: [pending]

### 3. Real-time mid-run interrupting alarm (>>LABEL)
expected: Program pauses at an instruction boundary, the alarm handler executes (e.g. stores a value), the program resumes from the exact halted step, and a repeating alarm reschedules for its next fire.
result: [pending]

### 4. Idle interrupting alarm — CLI vs GUI parity (DECISION)
expected: CLI calls `run_program` for the label (app.rs:1850 path) and the program executes. GUI calls `dispatch_op` (existing path) — a user-LBL program silently fails with an InvalidOp toast.
result: [pending]
note: SC-3 (ROADMAP "existing run_program path") conflicts with ALARM-03 ("existing path, unchanged"). Developer must decide whether the pre-existing GUI limitation is acceptable for phase pass, or whether App.tsx:1237 should be rewired to `run_program({ label })`.

### 5. CLI idle alarm launching a PSE program (WR-04, DECISION)
expected: The `alarm:xeq:` CLI path calls `run_program` then `drain_pending_yields`; the PSE pause renders.
result: [pending]
note: WR-04 — `drain_event_buffer` does not call `drain_pending_yields` after the `alarm:xeq:` `run_program`, so alarm-launched CLI programs containing PSE/VIEW/AVIEW have their yield stranded until the next keypress. Plan 63-03 specified wiring "any other site that runs a program to completion." Developer decides: in-scope fix now vs. follow-up.

## Summary

total: 5
passed: 0
issues: 0
pending: 5
skipped: 0
blocked: 0

## Gaps

### ADR artifact naming deviation (partial)
status: partial
reason: Plan 63-05 specified `docs/adr/v4.3-001-interrupt-alarm-pending-field.md`, but v4.3-001..003 were already allocated, so the ADR was created as `v4.3-004-interrupt-alarm-pending-field.md`. Content is correct and cross-referenced consistently (divergences doc → v4.3-004). Verifier recommends updating the plan spec to v4.3-004 rather than renaming.
