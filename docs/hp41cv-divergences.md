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
