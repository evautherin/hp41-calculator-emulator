# Quick Task 260603-lu0 — Pending Decisions for Item B

## D-lu0-01: Item A complete + on-device verified (2026-06-03)
CLRG fidelity fix (commits 4a7ea8b + merge c0e2559). On-device PASS — PRGM line
renders `XEQ CLRG`. CLREG kept as back-compat XEQ alias.

## D-lu0-02: Item B MUST include "tap-to-run" (user decision, 2026-06-03)
When Item B (tabbed full-function overlay) is (re)planned/executed, the **"All
Functions" tab entries must be tappable to EXECUTE the function** (XEQ-by-name via
the existing resolver), and — when the calculator is in PRGM mode — to **insert the
function as a program line** instead of executing.

**Why:** On-device testing revealed XEQ-by-name entry on iOS is effectively
unusable (opaque key→letter mapping; ENTER types ALPHA 'N'; submit = the ALPHA key,
which is undiscoverable — Phase 55 TOUCH-04 gap). Merely *listing* the 74 keyless
built-in functions (CLST, CLRG, CLA, AVIEW, SIZE, …) does not make them usable on
touch. Tap-to-run solves discoverability AND touch-execution in one affordance. The
backend resolver (`Op::Xeq(label)` / builtin_card_op) already works — this is a
frontend wiring + UX task.

**Plan impact:** Add a task to Item B: each "All Functions" row is a button →
on tap, dispatch `xeq_<DISPLAY_NAME>` (run) or, if `calcState` indicates PRGM mode,
the program-insert path. Close the overlay after dispatch (or keep open with a
toast — decide at plan time). Add a Vitest test for the tap→dispatch wiring.

## Related follow-ups surfaced (not yet scheduled)
- iOS PRGM-mode layout bug → handled in a separate quick task (created next).
- Sibling mnemonic divergences: `CL SIGMA` vs `CLΣ`, `CLRALPHA` duplicate of `CLA`.
- CLI TUI overlay still excludes key_path:null (GUI-first; CLI parity later).
