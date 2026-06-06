# v4.3 Hardware Fidelity — Audit Summary

**Synthesized:** 2026-06-06
**Sources:** CONTROL-ALARMS.md, DIVERGENCE-AUDIT.md, ARCHITECTURE.md, PITFALLS.md
**Confidence:** HIGH (every undocumented gap grep-verified against source with file:line citations)

## Headline

The emulator is already remarkably faithful: of all documented divergences, **21 are deliberate, accepted decisions that must NOT be touched**, and 9 are intentional emulator extensions. The genuinely open work is **10 fixable fidelity gaps + 1 anchor** (interrupting control alarms), plus 3 items needing hardware verification. The single most important architectural insight: **the anchor and four of the fixable gaps share one root cause** — `hp41-core`'s `run_loop` executes a program synchronously to completion with no yield/interrupt points. Build that one mechanism and control alarms, PSE timing, VIEW/AVIEW-during-programs (and, with more work, GETKEY and CATALOG scroll) all become solvable.

## Anchor: Interrupting Control Alarms (D-40-04)

**Behavioral spec (from CONTROL-ALARMS.md):** On real HP-41CX, a control alarm that comes due while a program runs halts the program at the next instruction boundary, saves execution state (PC + call stack), XEQs the alarm's stored label program in the *interrupted program's register environment* (the data stack X/Y/Z/T/LASTX is NOT auto-saved — well-behaved alarm programs save/restore their own context), and resumes the interrupted program when the alarm program ends — subject to the 4-level subroutine call-stack limit. The data model (`AlarmType::Control { label, interrupting }`, `>>`-prefix parsing) already exists; only execution is missing.

**Recommended implementation (from ARCHITECTURE.md) — synchronous, single-threaded, reuses existing machinery:**
- Add one transient field `pending_interrupt: Option<String>` to `CalcState` with `#[serde(default, skip)]` — no migration, no save-file impact.
- In the alarm dispatch path: when an alarm is interrupting AND a program is running, set `pending_interrupt` instead of emitting the current dead-end "deferred" event; when idle, keep the existing event_buffer XEQ path.
- In `run_loop`, between instructions (where it already checks cancel), `take()` the pending interrupt: if the call stack has room, push the current PC and redirect to the alarm label — exactly the existing `Op::Xeq` mechanism. The existing RTN/END handling then pops and resumes the interrupted program automatically.
- No new `Op` variants (no 4-way exhaustive-match impact), no threads, no `Arc<Mutex>` rework, no per-instruction overhead. At call-stack depth 4, suppress the alarm execution and leave it past-due (safe default; OM silent on this case).
- Call `check_alarms` periodically inside `run_loop` (~every 1000 steps) so the GUI can detect a mid-execution alarm despite holding the `CalcState` mutex during `run_loop` (no polling — D-11 preserved).

**⚠️ BLOCKING OPEN QUESTION — `>` / `>>` naming inversion.** The CONTROL-ALARMS researcher reads OM §XYZALM as: `>>label` = *conditional / deferred* (does NOT interrupt) and `>label` = the alarm that *actually interrupts a running program* — the **inverse** of what `docs/hp41-time-divergences.md` D-40-04 and the code's `interrupting: true` flag (attached to `>>`) currently assume. The DIVERGENCE-AUDIT agent, reading the same code, followed the existing convention. **The mechanism above is correct regardless of which prefix maps to "interrupt"; only the routing condition flips.** This MUST be settled against the primary OM in a spec/clarify step before any implementation code is written — otherwise the milestone could ship behavior that is exactly backwards.

## Prioritized Fixable Divergence Pick-List

Ranked by user-visibility × inverse-risk. "Shared" = depends on the same `run_loop` yield mechanism the anchor builds.

| Rank | ID | Name | Real HW vs. emulator (one line) | Vis. | Effort | Core/Frontend | Shared |
|------|----|------|----------------------------------|------|--------|---------------|--------|
| 1 | FGAP-01 | PSE timing | PSE should pause display ~1s mid-program; emulator pause is instant | High | M | CLI+GUI | run_loop yield |
| 2 | FGAP-02 | CLI VIEW/AVIEW/PROMPT invisible | CLI never reads `display_override` → VIEW/AVIEW show nothing | High | S | CLI only | — |
| 3 | FGAP-03 | FACT(27..69) overflows | HW returns scientific result to ~1.7E98; emulator errors at 27 | Med | M | core `math.rs` (NOT frozen) | — |
| 4 | FGAP-10 | VIEW/AVIEW mid-program | `display_override` set but invisible until program ends | Med | M | core | run_loop yield |
| 5 | FGAP-05 | CHS during mantissa entry | HW flips entry sign in place; emulator flushes + lifts stack | Med | M | CLI | — |
| 6 | FGAP-07 | AON auto-display | flag 48 stored but no frontend shows ALPHA after each op | Low | S | frontend | — |
| 7 | FGAP-09 | X-MEM PURFL/CLFL + bbb.eee | no DUP FL error, always full-register SAVED/GETD | Med | M | core `xmem` (new Ops → 4-way match) | — |
| 8 | FGAP-04 | Interactive GETKEY | GETKEY in program should wait for a keypress; emulator reads stale key | High | **L** | core (event-loop yield) | run_loop yield |
| 9 | FGAP-06 | Interactive CATALOG 1 scroll | CAT 1 should scroll line-by-line, R/S stops; emulator dumps all at once | Med | **L** | core | run_loop yield |
| — | FGAP-08 | GETKEY returns 0 no-key | consequence of FGAP-04; resolves automatically if FGAP-04 is done | Low | — | — | subsumed |

**Synergy notes:** FGAP-01 + FGAP-10 are the same fix (yield after `display_override`) — do them together. FGAP-04 + FGAP-06 are the larger structural members of the same `run_loop`-yield family but require *waiting for input* / *generator iteration*, so they are heavier (L) than the display-yield items.

## Do-Not-Touch (deliberately-accepted — protect from regression)

21 accepted divergences (full table in DIVERGENCE-AUDIT.md). Highlights the milestone must NOT "fix": D-30-01 INTG/SOLVE scratch-register clobber (hardware-faithful), D-30-04 FACT integer-only (no GAMMA), D-40-01 CORRECT/SETAF no-op (no physical oscillator), D-40-03/05 stopwatch frozen-on-save (Instant not serde-able), D-45-01/02 unbounded named-matrix size, D-CV-01/02 lenient mnemonic aliases (save-file back-compat). Note: D-52-01 (overwrite-on-duplicate) is "accepted" only because PURFL is absent — FGAP-09 would convert it back to faithful.

## Uncertain / Needs Verification (decide after a quick check, not up front)

- **UNC-01** — Does ← clear an error display on real HW? (verify vs. hardware/Free42)
- **UNC-02** — Do flags 21/25 gate PRX/PRA/PRSTK printing on real HW? (verify vs. OM)
- **UNC-03** — Does SIZE reduction show "MEMORY LOST" on the display? (verify; add `display_override` if confirmed)

## Key Risks & Guardrails (condensed from PITFALLS.md)

- **`run_loop` regression surface** — the engine is exercised by ~3300+ tests; any re-entrancy change can silently break ISG/DSE, call-stack, or XROM dispatch. Guardrail: full `just ci` green after every commit; `numerical_accuracy.rs` is the canary.
- **`is_running` + 4-level cap integrity** — interrupt frame must not push a 5th level and must always reset `is_running` even on error. Mandatory tests: cap-blocked + nested-interrupt-blocked.
- **serde discipline** — new transient fields need `#[serde(skip)]`; any persistent field needs `#[serde(default)]`. Guardrail: a v4.2-era JSON deserialization back-compat test.
- **MSRV 1.88 clippy divergence** — run `cargo +1.88 clippy --workspace --all-targets --all-features -- -D warnings` before tagging; check `gh pr checks`.
- **Deterministic-clock testing** — never call `SystemTime::now()` in assertions; use `trigger_unix = 0` (always past) / `i64::MAX` (never) with `time_offset_secs = 0`.
- **Free42 contamination guard** — algorithms hand-derived; Free42 oracle only, never copied (`just license-audit`).
- **CI coverage gaps** — GUI-crate clippy is ungated (run manually); `#[cfg(mobile)]` blind spot in `just gui-ci`.

## Suggested Phase Decomposition (first cut for the roadmapper)

1. **Spec & clarify alarm semantics** — resolve the `>`/`>>` naming-inversion blocker against the primary OM; lock the behavioral contract (incl. depth-4 and idle-fire behavior). Small, gates the engine phase.
2. **Run-loop yield/interrupt engine + interrupting control alarms (ANCHOR)** — the `pending_interrupt` mechanism + control-alarm execution; carries PSE timing (FGAP-01) and VIEW/AVIEW mid-run (FGAP-10) on the same yield primitive. Core, the heart of the milestone.
3. **Selected standalone fidelity fixes** — the audit-picked frontend/math items (e.g. FGAP-02, FGAP-03, FGAP-05, FGAP-07) — independent, parallelizable.
4. **Tests, divergence-doc updates & verification** — re-entrancy test matrix, update `docs/hp41-time-divergences.md` (D-40-04 resolution) + any newly-closed divergence docs, quality gates green.

(Final phase set + which fixable gaps are in scope is set by requirement selection next.)

## Sources

- `.planning/research/CONTROL-ALARMS.md` — control-alarm behavioral spec + edge cases
- `.planning/research/DIVERGENCE-AUDIT.md` — full classified divergence inventory (the pick-list)
- `.planning/research/ARCHITECTURE.md` — engine re-entrancy design
- `.planning/research/PITFALLS.md` — risks, guardrails, verification strategy

---
*Synthesized for v4.3 Hardware Fidelity, 2026-06-06.*
