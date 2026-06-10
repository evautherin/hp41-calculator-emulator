# HP-41CV Built-in Emulator Divergences

This document lists known behavioral and naming divergences between this emulator's
HP-41CV ROM built-in function set and the hardware-faithful behavior described in the
HP-41C/CV Owner's Manual (HP 00041-90001). Module-pac divergences live in their own files
(`docs/hp41-{math1,stat1,time,advantage,xmem}-divergences.md`).

**Status:** Established 2026-06-03 (quick task 260603-uzh), consolidating notes from the
v4.1 iOS touch-UI pass.

**Philosophy:** (1) hardware-faithful where feasible; (2) user-safety / no silent surprise;
(3) every divergence documented with a stable ID for cross-reference from source comments,
ADRs, and tests.

---

## D-CV-01 — Lenient mnemonic aliases on XEQ-by-name

**Authentic:** the HP-41 recognizes exactly one spelling per built-in (`CLRG`, `CLΣ`).
**Emulator:** the XEQ-by-name resolver (`builtin_card_op`) additionally accepts non-authentic
back-compat spellings so older programs / saves keep resolving:
- `CLRG` (authentic) **and** `CLREG` (former internal spelling, corrected in v4.1 — ADR
  v4.1-004 / quick task 260603-s17 + lu0).
- `CLΣ` (authentic, U+03A3) **and** `CL SIGMA` / `CLSIGMA`.

**Rationale:** the authentic mnemonics are the canonical `display_name` and the program-line
text; the extra arms are tolerated input only. **Impact:** none on display fidelity — only
the resolver is more permissive than hardware.

## D-CV-02 — `CLRALPHA` legacy alias of `CLA`

**Authentic:** the HP-41 has one clear-ALPHA function, `CLA`.
**Emulator:** two `Op` variants delegate to the same `op_alpha_clear`: `Op::Cla`
(`display_name "CLA"`, the hardware-faithful name) and `Op::AlphaClear`
(`display_name "CLRALPHA"`, a v1.0 legacy variant). `Op::AlphaClear` must stay in the enum +
resolver for v1.0 save-file deserialization (Pitfall 8 — do NOT consolidate), but it is a
non-authentic name. **Impact:** `CLRALPHA` is hidden from the "All Functions" help index
(`OVERLAY_HIDDEN_ALIASES`) so only `CLA` is shown; `XEQ "CLRALPHA"` still resolves for old
programs. ADR v4.1-004 / quick task 260603-s17.

## D-CV-03 — ALPHA overrides SHIFT (GUI keypad)

**Authentic:** SHIFT and ALPHA are independent annunciators.
**Emulator (GUI only):** when ALPHA mode is active, an on-screen key with an ALPHA letter
dispatches that letter (`alpha_<X>`) regardless of a pending one-shot SHIFT — ALPHA wins.
Accepted, long-standing divergence (CLAUDE.md "CLI↔GUI parity"). **Impact:** you cannot
trigger a shifted op from the on-screen keypad while ALPHA is on; toggle ALPHA off first.

## D-CV-04 — iOS ALPHA entry is on-screen-keys-only (no software keyboard)

**Authentic:** N/A (the HP-41 has no software keyboard — entry is always via the keys).
**Emulator (iOS):** ALPHA-register and "FUNCTION NAME?" text entry use **only** the
on-screen HP-41 blue-letter keys; the iOS software keyboard is never shown. The ← key
deletes the last ALPHA char (`alpha_backspace`); the typed text appears on the main 14-seg
display. This is *more* hardware-faithful than the superseded Phase-55 iOS-keyboard
approach. ADR v4.1-003 / quick tasks 260603-sef, 260603-u6t. **Desktop:** the physical
keyboard types ALPHA letters and Backspace deletes the last ALPHA char.

## D-CV-05 — Interactive GETKEY: suspend execution and await keypress — Implemented (v4.3)

**Status: IMPLEMENTED in Phase 64 (v4.3). Closes FGAP-04 (subsumes FGAP-08) and SYNT-06.**

**Authentic (HP-41CX Extended Functions):** When `GETKEY` executes inside a running
program, it pauses execution, waits for the user to press a key, pushes that key's
HP-41 row×col code to X (LiftEffect::Enable), and resumes the program from the next step.
The row×col code is computed as `row × 10 + col` (1-indexed), so R/S (row 3, col 1) → 31;
ENTER (row 8, col 4) → 84; digits 0–9 → 81–90; etc. When the user cancels the wait (no
key — e.g. hardware ON key on original HW, or the emulator cancel path), `GETKEY` pushes
the no-key sentinel **0** and the program continues. `GETKEY` waits indefinitely — there
is no timeout on hardware (the timed variant `GETKEYX` is a separate extension).

**Emulator behavior (v4.3+):** Interactive GETKEY is implemented via the Phase 63
yield/suspend engine extended with a new event-driven `YieldKind::WaitForKey` yield kind.

- **Suspend:** `Op::GetKey` executing inside `run_loop` sets a `WaitForKey` pending yield
  and breaks out of the run loop, leaving the program counter advanced past the GETKEY step.
  The suspended state is projected to frontends as `pending_yield.kind = "wait_for_key"`.
- **Resume with key:** The frontend captures the next keypress during the suspend; on-screen
  HP-41 key taps and physical keyboard keys both contribute. The HP-41 row×col keyCode for
  the pressed key is passed to `resume_program_with_key(keycode)`, which calls `op_getkey`
  inline (pushing the keycode to X), then re-enters `run_loop`. Keys that have no HP-41
  keyCode (e.g. CHS, x≥y, function keys) are silently ignored — the suspend continues.
- **Cancel path (sentinel 0):** The Phase 63 `request_cancel` command (Esc / window cancel)
  calls `resume_program_with_key(0)`, pushing the no-key sentinel and cleanly ending the
  suspended wait. R/S during GETKEY is captured like any other key (returns 31); it does
  **not** stop the program — hardware-faithful behavior.
- **Display during suspend:** The 14-segment display shows whatever was last displayed
  before GETKEY; no special prompt is shown (faithful to hardware). D-03.
- **No emulator timeout:** The emulator waits indefinitely, matching the hardware GETKEY
  behavior. `GETKEYX` (timed variant) is deferred — D-01.
- **Alarm-handler safety:** If GETKEY fires inside a Phase 63 interrupting-alarm handler
  frame, `resume_program_with_key` preserves the alarm-handler context fields
  (`pending_interrupt_alarm_index` / `pending_interrupt_depth`), unlike the generic
  `resume_program` which clears them. D-06 / 64-01-D03.

**R/S keycode correction (D-02):** CONTEXT.md decision D-02 originally stated "R/S is
captured as keycode **84**." This was a documentation error: **84 is the ENTER key's
position** (row 8, col 4). R/S physically occupies row 3, col 1 on the HP-41 hardware —
its correct HP-41 code is **31**. The emulator implements and tests R/S → 31. Corrected
per Phase 64 research (Pitfall 4 / Open Question 1) and orchestrator decision.

**Closes:**
- **FGAP-04** (interactive GETKEY: pause, wait for key, push row×col code to X, resume)
- **FGAP-08** (subsumes: no-key sentinel 0 on cancel, not on every call) — FGAP-08
  handling is intrinsic to FGAP-04's cancel path (`request_cancel` → sentinel 0).
- **SYNT-06** (interactive GETKEY deferral history — now resolved).

**Implementation references:**
- `hp41-core/src/state.rs` — `YieldKind::WaitForKey` variant; `getkey_captured_code: Option<u8>`
  transient field (`#[serde(default, skip)]`).
- `hp41-core/src/ops/program.rs` — `Op::GetKey` yield arm in `run_loop`;
  `resume_program_with_key(keycode: u8)` public function.
- `hp41-core/src/ops/registers.rs` — `op_getkey` dual-path: captured code via
  `getkey_captured_code.take()` (interactive program path) or `last_key_code` fallback
  (non-program dispatch, backward-compat).
- `hp41-core/tests/phase_64_getkey.rs` — 8 integration tests: PRGM-03-a..i.
- `hp41-cli/src/app.rs` — `drain_pending_yields` WaitForKey branch; `handle_key` guard.
- `hp41-gui/src-tauri/src/commands.rs` — `resume_program_with_key` Tauri command.
- `hp41-gui/src/App.tsx` — yield-driver `wait_for_key` skip + key-event resume guard
  (on-screen taps + physical keyboard) + cancel → sentinel 0.
- Phase 64 / Plan 64-01 through 64-04 (core engine, CLI wiring, GUI Tauri IPC, GUI TS).

## D-CV-06 — CLI VIEW/AVIEW/PROMPT display reads display_override — Implemented (v4.3)

**Status: IMPLEMENTED in Phase 65 (v4.3). Closes FGAP-02 / DISP-01.**

**Authentic:** On the HP-41, VIEW n, AVIEW, and PROMPT show the register value or ALPHA
string on the display for approximately one second (VIEW/AVIEW) or until a key is pressed
(PROMPT). OM p.16 (Viewing Register Contents).

**Emulator behavior (v4.3+):** The CLI `hp41-cli/src/ui.rs::get_display_string()` now
reads `state.display_override` and shows its value for one render cycle before clearing.
Prior to Phase 65, `get_display_string()` never read `display_override` — the view result
was silently discarded. The core (`hp41-core`) already set `state.display_override`
correctly via `op_view` / `op_aview`; only the CLI rendering path was missing.

**Implementation references:**
- `hp41-cli/src/ui.rs` — `get_display_string()` `display_override` branch.
- Phase 65 / Plan 65-XX.

## D-CV-07 — CHS during mantissa entry toggles sign in entry buffer — Implemented (v4.3)

**Status: IMPLEMENTED in Phase 65 (v4.3). Closes FGAP-05 / DISP-02.**

**Authentic:** On the HP-41, pressing CHS while a number is being entered (non-EEX mode)
toggles the sign of the entry in place — the display flips sign without flushing the buffer
to the stack. OM p.15 (Display Editing — sign behavior).

**Emulator behavior (v4.3+):** `hp41-cli/src/app.rs` intercepts CHS before the flush
dispatch when `entry_buf` is non-empty and contains no EEX character, toggleing the sign
bit of the entry string in place. Prior to Phase 65, CHS always flushed the entry buffer
to the stack before negating X, lifting the stack on every CHS-during-entry.

**Implementation references:**
- `hp41-cli/src/app.rs` — CHS intercept before flush.
- Phase 65 / Plan 65-XX.

## D-CV-08 — AON flag-48 auto-display — Implemented (v4.3)

**Status: IMPLEMENTED in Phase 65 (v4.3). Closes FGAP-07 / DISP-03.**

**Authentic:** On the HP-41, AON sets flag 48 (Alpha Mode). While flag 48 is set, the
ALPHA register is automatically shown on the display after every operation. AOFF clears
flag 48. OM p.53 (Flag 48: Alpha Mode).

**Emulator behavior (v4.3+):** Both `hp41-cli/src/ui.rs` and `hp41-gui/src/App.tsx` now
check flag 48 in `calcState.flags` / `state.flags` after every operation, and display the
ALPHA register value when the flag is set. Prior to Phase 65, `op_aon` stored flag 48 but
no frontend read it — the auto-display effect was silent.

**Implementation references:**
- `hp41-cli/src/ui.rs` — flag-48 auto-display in `get_display_string()`.
- `hp41-gui/src/App.tsx` — flag-48 auto-display in state projection.
- Phase 65 / Plan 65-XX.

## D-CV-09 — PRX/PRA/PRSTK now flag-gated on printer presence — Fixed (v4.3)

**Status: FIXED in Phase 66 (v4.3). Closes UNC-02.**

**Authentic:** On the HP-41, executing PRX, PRA, or PRSTK when no printer is connected
produces a NONEXISTENT-class error. Flag 21 (Printer Enable, p.53) and Flag 55 (Printer
Existence, p.54) together indicate printer presence; at least one must be set for print
functions to succeed. OM p.57-58: "An attempt was made to execute a specific print function
when the printer was not connected to the system."

**Emulator behavior prior to v4.3:** `op_prx`, `op_pra`, and `op_prstk` in
`hp41-core/src/ops/print.rs` did not read flags 21 or 55 — they always pushed to
`print_buffer` regardless of printer state. This was a divergence from hardware behavior.

**Emulator behavior (v4.3+):** Each print function now checks
`flag_get(state.flags, 55) || flag_get(state.flags, 21)` at entry and returns
`Err(HpError::NonExistent)` if neither is set. `HpError::NonExistent` is a new variant
added to `hp41-core/src/error.rs` matching the OM "NONEXISTENT" error class. All print
tests were reworked to set flag 55 in setup; one new test
(`test_prx_returns_nonexistent_when_no_printer_flag`) was added.

**Note on flag 25:** Flag 25 = "Error Ignore" (generic error-suppression flag, OM p.53) —
it is NOT a printer flag. The original UNC-02 audit text was imprecise; the fix uses
flags 21 and 55 only.

**Implementation references:**
- `hp41-core/src/ops/print.rs` — `op_prx`, `op_pra`, `op_prstk` (flag guard at entry).
- `hp41-core/src/error.rs` — `HpError::NonExistent` variant.
- `hp41-core/tests/print_tests.rs` — reworked setup + new nonexistent test.
- Phase 66 / Plan 66-02.

---

## Verified-Correct Notes (v4.3)

The following UNC items were investigated during Phase 66 (Plan 66-03) by direct reading
of the HP-41C Operating Manual (HP 00041-90259, June 1980). They are NOT divergences —
the emulator already behaves correctly.

**UNC-01 — Back-arrow clears error message (OM p.15, verified Phase 66):**
The OM states "Pressing [←] also clears error messages from the display." The emulator
correctly implements this: `hp41-cli/src/app.rs` lines 974–975 call `backspace_entry()`
then set `self.message = None` on every back-arrow press, clearing any active error or
status message. No fix was required.

**UNC-03 — SIZE reduction is silent, no MEMORY LOST display (OM p.19 + p.57, verified Phase 66):**
OM p.19 describes SIZE reduction as silently truncating the highest-numbered registers —
no special display message. OM p.57 defines "MEMORY LOST" as a Continuous Memory clear
event caused by battery interruption or memory-module removal — not a SIZE event. The
emulator correctly does not show "MEMORY LOST" on SIZE reduction. A misleading comment
in `op_size` (`hp41-core/src/ops/program.rs`) that implied "hardware-faithful 'MEM LOST'"
was corrected in Phase 66 / Plan 66-02.

---

*Last updated: 2026-06-10 (§D-CV-06–09 added — Phase 65/66 DISP + UNC closures). Established 2026-06-03.*
