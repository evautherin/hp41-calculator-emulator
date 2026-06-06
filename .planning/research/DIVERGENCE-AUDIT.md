# Divergence Inventory — Real HP-41 vs. Emulator (v4.3 Fidelity Audit)

**Researched:** 2026-06-06
**Methodology:** Read all six `docs/hp41*divergences*.md` files + `.planning/PROJECT.md` + `.planning/STATE.md`, then grep-verified every undocumented gap claim against `hp41-core/src/`, `hp41-cli/src/`, and `hp41-gui/src-tauri/src/`.

---

## Summary

| Category | Count |
|----------|-------|
| **FIXABLE-GAP** (genuine gaps, actionable) | **10** |
| **ANCHOR** (D-40-04 interrupting alarms — milestone anchor) | 1 |
| **DELIBERATELY-ACCEPTED** (do not touch) | 21 |
| **EMULATOR-EXTENSION** (additions, informational) | 9 |
| **UNCERTAIN / Needs-User-Decision** | 3 |

**Headline:** The emulator is remarkably complete. Most divergences from real HP-41 hardware are either thoroughly deliberate architectural decisions or minor behavioral details. The 10 fixable gaps cover: PSE timing, CLI display_override routing, FACT range cap, GETKEY interactive wait, CHS during mantissa entry, CATALOG interactive navigation, VIEW/AVIEW in program execution, AON auto-display, X-MEM PURFL+bbb.eee control, and GETKEY return-0 on no-key.

---

## Prioritized FIXABLE Gaps

Ranked by (User-Visibility × Inverse-Risk). Highest-impact, lowest-risk items first. Each is phrasable as a user-facing requirement.

| Rank | ID | Name | Real HP-41 Hardware | Emulator | Visibility | Effort | Risk / Frozen? | Requirement Phrasing |
|------|----|------|---------------------|----------|------------|--------|----------------|----------------------|
| 1 | **FGAP-01** | PSE timing — no actual 1-second pause | PSE during program: freezes display showing X for ~1 second, then continues execution. User sees a brief pause; useful for reading intermediate results. | `Op::Pse` pushes `"PAUSE 1000"` to `event_buffer` and continues `run_loop` immediately. Neither CLI (`drain_event_buffer` ignores `"PAUSE 1000"`) nor GUI (`App.tsx` event handler ignores `"PAUSE 1000"` events) insert any timing delay. The pause is instant. (Confirmed: `hp41-core/src/ops/program.rs:902-908`; `hp41-cli/src/app.rs:1778-1799`; `hp41-gui/src/App.tsx:1166-1186`) | High | M | Touches `hp41-cli` + `hp41-gui` only (not core). No frozen code. No save-file impact. No 4-way match. | PSE during program execution must pause the display for ~1 second before continuing to the next step. |
| 2 | **FGAP-02** | CLI: VIEW/AVIEW/PROMPT result not shown on display | VIEW n, AVIEW, PROMPT in interactive mode: display shows the register value / ALPHA string for ~1 second ("view mode"). The user can see the result. | CLI: `call_dispatch(Op::View(n))` calls core which sets `state.display_override`. But `hp41-cli/src/ui.rs::get_display_string()` never reads `state.display_override` — it falls through to entry_buf / prgm_mode / alpha / X format. The view result is silently discarded. (Confirmed: `hp41-cli/src/ui.rs:131-160` has no `display_override` branch; `grep -r "display_override" hp41-cli/` returns zero matches.) | High | S | `hp41-cli` only. No frozen code, no 4-way match, no save-file. | VIEW, AVIEW, and PROMPT must display the register value / ALPHA string on the CLI display for ~1 second (or until next key). |
| 3 | **FGAP-03** | FACT range cap at 26 vs. hardware 69 | FACT(27) = 10,888,869,450,418,352,160,768,000,000 — this exceeds 9-digit HP-41 display capacity, so HP-41 hardware shows `9.999999999E 26` (scientific-notation overflow clip). FACT(69) = last valid value (~1.711E98). Values 27–69 produce a large scientific result, not an error. | `op_fact` returns `HpError::Overflow` for X in 27..=69 due to `Decimal::from_f64` wall. Per `hp41-core/src/ops/math.rs:480-482` and README.md line 170. FACT(27) → Overflow on emulator; HP-41 would show ~1.089E28. | Med | M | Touches `hp41-core/src/ops/math.rs` (NOT frozen — `math1/` is frozen, but `math.rs` is a core built-in, NOT in `math1/`). No 4-way match impact (FACT is not a new Op variant). No save-file impact. | FACT(X) for X in 27..=69 must return the correct factorial value in scientific notation rather than an Overflow error. |
| 4 | **FGAP-04** | GETKEY in programs: reads last key, does not wait | Real HP-41 GETKEY inside a running program: pauses execution at that step and waits for the user to press a key, then pushes the HP-41 row×col key code to X and resumes. This is the core use case for interactive programs. | `op_getkey` reads `state.last_key_code` immediately (whatever key was pressed before the program started) and continues without pausing. No event-loop yield. (Confirmed: `hp41-core/src/ops/registers.rs:143-151`; deferred as SYNT-06 since v1.1: `.planning/milestones/v1.1-phases/12-synthetic-programming/12-CONTEXT.md:131`) | High | L | Requires program execution to yield to the event loop — a structural change in `hp41-core` program execution model. No frozen math code affected. Significant architecture work (call `run_loop` yield point or event-driven step). No 4-way match impact. | GETKEY inside a running program must pause execution, wait for the next key press, and push that key's HP-41 code to X before continuing. |
| 5 | **FGAP-05** | CHS during number-entry (mantissa, not EEX) | On real HP-41: if a number is being entered (non-EEX), pressing CHS negates the entry in place — the display flips sign without flushing the buffer. E.g., entering "123" then CHS shows "-123" still in entry mode. | CLI: CHS dispatches `Op::Chs` via `call_dispatch`, which calls `op_chs` (negates X) — but only after `flush_entry_buf()` is called first (dispatch calls flush). So CHS flushes "123" → X=123, then negates X=-123, lifting the stack. The hardware-equivalent "flip sign in entry buffer without flush" is not implemented. (Only EEX-CHS is handled in-buffer at `hp41-cli/src/app.rs:691-707`.) | Med | M | Touches `hp41-cli/src/app.rs` (intercept CHS before flush when entry_buf non-empty and no 'e') and possibly `hp41-core` if a dedicated `backspace_entry`-style function is added. No frozen code. No 4-way match. | CHS during active number entry (mantissa, no EEX) must toggle the sign of the entry buffer in place, without flushing the entry to the stack. |
| 6 | **FGAP-06** | CATALOG 1: no interactive navigation; single-shot dump | Real HP-41 CATALOG 1 in interactive mode: scrolls through the program listing one line at a time. R/S stops the listing. SST/BST step through. | CAT 1 dumps the entire listing synchronously into `print_buffer` in a single call (no per-line yield, no PSE-step, no R/S stop). Comment at `hp41-core/src/ops/program.rs:346` explicitly defers this: "NO PSE-step, NO per-line yield (v2.2 CAT 1 shape; D-31.12/D-31.14 PSE-step deferred per RESEARCH Open Q2)". | Med | L | Would require a new execution model for CATALOG (generator / iterator yield). Touches `hp41-core`. Not in frozen `math1/`. Moderate complexity. No 4-way match. | CATALOG 1 must scroll through the program listing interactively; R/S must stop the listing mid-scroll. |
| 7 | **FGAP-07** | AON: no actual ALPHA auto-display effect | Real HP-41 AON sets a mode where after every operation the ALPHA register is automatically shown on the display (even without AVIEW). | `op_aon` sets `flag 48`. No frontend reads `flag 48` to drive alpha auto-display after every op. Confirmed: `grep -rn "flag.*48" hp41-cli/src/` → 0 matches; `grep -rn "flag.*48" hp41-gui/src/` → 0 matches. The flag is stored but has no observable effect. | Low | S | Frontend-only change. `hp41-cli/src/ui.rs::get_display_string` or `hp41-gui/src/App.tsx` display logic needs to check `calcState.flags & (1 << 48)`. No frozen code, no 4-way match, no save-file change (flags already serde'd). | AON must cause the ALPHA register to display automatically after every operation while flag 48 is set; AOFF must disable this. |
| 8 | **FGAP-08** | GETKEY returns 0 when no key yet pressed | Real HP-41 GETKEY with no prior key: returns 0 (the "no-key" sentinel) on hardware — but the machine starts with 0 anyway, so this is edge-case only; interactive GETKEY always waits (FGAP-04 above). | `state.last_key_code` defaults to 0 (per `hp41-core/src/state.rs`). Combined with FGAP-04, GETKEY in a program gets code 0 even if the user pressed a key before running — but tracking the actual code requires FGAP-04 to be resolved first. Documented as "SYNT-06" deferred. | Low | S | Subsumed by FGAP-04. If FGAP-04 is implemented, FGAP-08 resolves automatically. If FGAP-04 stays deferred, FGAP-08 is irrelevant by itself. | (See FGAP-04; this item is a consequence, not independent.) |
| 9 | **FGAP-09** | X-MEM: missing PURFL/CLFL + full-register-set SAVED/GETD | Real HP-41CX: SAVEP/SAVED over an existing file raises "DUP FL"; user must call PURFL first. SAVED reads bbb.eee from X to save a partial register range. | Overwrite-on-duplicate (D-52-01): emulator silently overwrites. bbb.eee block control word (D-52-02): emulator always saves/restores full register set. PURFL/CLFL not implemented. Documented in `docs/hp41-xmem-divergences.md`. | Med | M | Touches `hp41-core/src/ops/xmem/ops.rs` — not frozen. New Op variants for PURFL/CLFL would require 4-way exhaustive match updates. bbb.eee parsing uses ISG/DSE discipline (string-split). | PURFL and CLFL must be implemented so that SAVEP/SAVED correctly raise "DUP FL" rather than silently overwriting. SAVED/GETD must honor the bbb.eee block control word from X. |
| 10 | **FGAP-10** | VIEW/AVIEW/PROMPT in programs: display gap mid-run | In program execution, VIEW n should display the register value briefly (PSE-like pause) before the next step. AVIEW similarly. PROMPT halts and waits. | `op_view` / `op_aview` set `display_override`. In `run_loop` this override is set mid-execution but the frontend sees it only when the program completes (since the loop runs synchronously to completion). PROMPT does break `run_loop` (confirmed: `hp41-core/src/ops/program.rs:660-662`). VIEW/AVIEW do not yield — the display override is set but invisible until program completion. | Med | M | Connected to FGAP-01 (PSE yield mechanism). A general "yield after display_override" mechanism in `run_loop` would fix PSE, VIEW, and AVIEW together. Touches `hp41-core` program execution (not frozen `math1/`). | VIEW n and AVIEW inside a running program must pause to display the value briefly before continuing (similar to PSE behavior). |

**Notes on FGAP-08:** This is a consequence of FGAP-04 and should not be prioritized independently. If FGAP-04 (interactive GETKEY) is implemented, FGAP-08 is resolved as a side effect.

**Joint implementation note for FGAP-01 + FGAP-10:** Both require the same underlying mechanism: a way for `run_loop` to yield after setting `display_override` or pushing `"PAUSE 1000"`. Implementing them together is strongly recommended to avoid solving the same architectural problem twice.

---

## Anchor: D-40-04 Interrupting Control Alarms

**Status:** ANCHOR — this is the primary milestone driver. Not ranked in the fixable list above; it has its own dedicated phase.

**What real hardware does:** When a control alarm with `>>label` fires while a program is executing, the HP-41CX halts the running program at the next instruction boundary, saves execution state, XEQs the alarm's designated label program, and resumes the interrupted program when the alarm program completes (subject to the 4-level call stack limit).

**What the emulator does:** `AlarmType::Control { label, interrupting: true }` is stored correctly. `check_alarms()` fires it as a non-interrupting message alarm — the label is not executed. The `interrupting: true` flag is ignored. Both CLI (`hp41-cli/src/app.rs:1776: "alarm:interrupting:..." → ignored`) and GUI (`hp41-gui/src/App.tsx:1180: // D-38.4: interrupting control alarms deferred — silently ignore`) have explicit ignore arms.

**What needs to happen:** The program execution engine needs re-entrancy support — the ability to interrupt a `run_loop` call, save PC + call stack state, invoke the alarm label program, and resume. This is architectural work in `hp41-core`.

**Data model:** Already in place (D-38.4 / D-38.8). `AlarmEntry` + `AlarmType::Control { label, interrupting }` + `parse_alarm_type` parsing `>>` prefix are all implemented.

**Risk:** High (core engine structural change). Touches `hp41-core/src/ops/program.rs` `run_loop` — sensitive but not frozen (`math1/` is frozen, not `program.rs`). Requires careful call-stack bookkeeping. Save-file compat: `call_stack` state during run is transient (`#[serde(skip)]`).

---

## Deliberately-Accepted Divergences (DO NOT TOUCH)

These are intentional decisions documented in the codebase. Fixing them would BREAK correct behavior.

| ID | Source Doc | Name | Real HP-41 | Emulator | Why Accepted |
|----|-----------|------|-----------|---------|--------------|
| D-CV-01 | hp41cv-divergences.md | Lenient mnemonic aliases on XEQ | Exact spelling only (CLRG, CLΣ) | Also accepts CLREG, CL SIGMA, CLSIGMA | Back-compat for v1.0 save files |
| D-CV-02 | hp41cv-divergences.md | CLRALPHA legacy Op variant | CLA only | Op::AlphaClear ("CLRALPHA") also resolves | v1.0 save-file deserialization (Pitfall 8) |
| D-CV-03 | hp41cv-divergences.md | ALPHA overrides SHIFT (GUI) | SHIFT and ALPHA independent | ALPHA wins in GUI keypad | Accepted, long-standing; noted in CLAUDE.md |
| D-CV-04 | hp41cv-divergences.md | iOS: keys-only ALPHA entry | Real HP-41 has only keys | No iOS software keyboard | More hardware-faithful than alternatives |
| D-30-01 | hp41-math1-divergences.md | INTG/SOLVE/DIFEQ scratch register clobber | Silent wrong result | Silent wrong result | Hardware-faithful; OM warns user, machine doesn't detect |
| D-30-02 | hp41-math1-divergences.md | POLY: multiplicity-as-cluster | Natural Bairstow clustering | Cluster of nearby roots | Hardware-faithful (same algorithm) |
| D-30-03 | hp41-math1-divergences.md | INTG convergence tied to DisplayMode | ½ ULP of displayed digits | Identical behavior | Hardware-faithful per OM |
| D-30-04 | hp41-math1-divergences.md | FACT integer-only (no GAMMA) | FACT is integer-only | Identical | Scope discipline; HP-15C adds GAMMA, HP-41 does not |
| D-30-06 | hp41-math1-divergences.md | Strict-reject nested INTG/SOLVE/DIFEQ | Undefined behavior (hang/wrong) | Clean InvalidOp error | User-safety improvement; OM warns but doesn't enforce |
| D-30-07 | hp41-math1-divergences.md | Modal R/S submits numeric input | Hardware-faithful | Identical | Hardware-faithful per OM p. 13 |
| D-40-01 | hp41-time-divergences.md | CORRECT/SETAF accuracy factor no-op | Applies crystal drift correction | SETAF stores; CORRECT is no-op | No physical oscillator to correct; time_offset_secs gives user control |
| D-40-02 | hp41-time-divergences.md | SW emulator extension (interactive stopwatch) | RUNSW/STOPSW/RCLSW by XEQ | SW opens interactive keyboard mode | Extension with no hardware conflict |
| D-40-03 | hp41-time-divergences.md | Stopwatch frozen on save | Running stopwatch counts on save | Stopwatch frozen on save/load | std::time::Instant not serde-able |
| D-40-05 | hp41-time-divergences.md | Centisecond stopwatch via Instant::elapsed | Hardware crystal timer | f64 host-clock elapsed | Acceptable host-clock accuracy |
| D-45-01 | hp41-advantage-divergences.md | Unlimited named-matrix count | Limited by X-MEM slots (~32 practical) | Vec<AdvMatrix> grows unbounded | Modern host memory; OM doesn't specify a limit |
| D-45-02 | hp41-advantage-divergences.md | 255×255 per-dimension matrix cap | ~14×14 practical (X-MEM) | u8 type enforces 255 max | Higher than hardware; OM unspecified |
| D-45-03 | hp41-advantage-divergences.md | Named-matrix vs. Math Pac I isolation | Naturally isolated by addressing | Explicitly isolated in code | D-43.5 isolation mandate |
| D-45-05 | hp41-advantage-divergences.md | TVM register persistence | CMOS RAM persistent | #[serde(default)] without skip | Matches hardware semantics |
| D-45-07 | hp41-advantage-divergences.md | 36-bit ADV_WORD_MASK truncation | Implicit BCD overflow | Explicit 36-bit mask | Hardware-faithful |
| D-45-09 | hp41-advantage-divergences.md | MATH_1 wins 12 ADV_MATH_B overlaps | XROM 7 < XROM 24 priority | Identical | Hardware-faithful resolver order |
| D-52-01 | hp41-xmem-divergences.md | Overwrite-on-duplicate (no DUP FL) | DUP FL error, requires PURFL | Silent overwrite | No PURFL implemented; avoids unusable system |

**Note on D-52-01:** This is listed as "accepted" only because PURFL is absent. The fixable gap FGAP-09 proposes implementing PURFL, which would convert D-52-01 back to hardware-faithful behavior. The user should decide.

---

## Emulator Extensions (Informational)

These additions are not in the HP-41 hardware spec. They are intentional quality-of-life improvements. Do not remove.

| ID | Source | Extension | Rationale |
|----|--------|-----------|-----------|
| D-30-05 | hp41-math1-divergences.md | XEQ "REAL" — deactivates complex_mode | No hardware exit from complex mode; UX pragmatism (D-28.3) |
| D-35-07 | hp41-stat1-divergences.md | RAND / SEED LCG random number generator | Community convention (NPS55-84-003 / Don Malm LCG formula); not in Stat 1 Pac OM |
| D-35-08 | hp41-stat1-divergences.md | ΣPOLYP "DEGREE=?" prompt wording | Math Pac I precedent; OM specifies mechanism but not wording |
| D-40-02 | hp41-time-divergences.md | SW interactive stopwatch keyboard mode | Natural UX equivalent of CLOCK/CLKT for stopwatch |
| D-45-06 | hp41-advantage-divergences.md | adv_current_matrix transient | Matches hardware: ALPHA register volatile across power cycles |
| D-45-08 | hp41-advantage-divergences.md | FINTG/FSOLVE cross-nesting (one level) | Most common real-world use case; hardware constraint was call-stack depth only |
| D-45-04 | hp41-advantage-divergences.md | FROOT Laguerre coexisting with POLY Bairstow | Hardware-faithful: separate XROM IDs, different mnemonics |
| EXT-01 | CLAUDE.md | JSON help system with search aliases | No analog in hardware; discoverability addition |
| EXT-02 | CLAUDE.md | macOS menu-bar mode | No analog in hardware; platform convention |

---

## Undocumented Gaps Found (with Code Citations)

These gaps were not documented in any existing `*divergences*.md` file. Each claim is backed by a code observation.

### UG-01: PSE timing — no actual pause (= FGAP-01)

Confirmed by reading `run_loop` PSE arm (`hp41-core/src/ops/program.rs:902-908`): sets `display_override` and pushes `"PAUSE 1000"` to `event_buffer`, then falls through to the next instruction. The CLI's `drain_event_buffer()` at `hp41-cli/src/app.rs:1778-1799` silently ignores `"PAUSE 1000"` strings (no handling arm). The GUI `App.tsx` event handler at line 1166-1186 similarly has no arm for `"PAUSE 1000"`. The core comment at `mod.rs:1803` says "Frontend timing" — which was never implemented.

### UG-02: CLI VIEW/AVIEW/PROMPT result invisible (= FGAP-02)

`hp41-core/src/ops/display_ops.rs:23` confirms `op_view` writes `state.display_override = Some(...)`. The entire `hp41-cli/src/ui.rs` has zero references to `display_override` (confirmed by `grep -r "display_override" hp41-cli/` returning empty). `get_display_string()` at `ui.rs:131-160` never reads `state.display_override`. The GUI does handle this correctly via `calcState.display_override ?? calcState.display_str` at `App.tsx:1268`. This is a CLI-only gap.

### UG-03: GETKEY in programs reads stale last_key_code (= FGAP-04)

`op_getkey` at `hp41-core/src/ops/registers.rs:143-151` pushes `state.last_key_code` (a `u8` set by the CLI on every key press, default 0) without pausing. The original planning document `.planning/milestones/v1.1-phases/12-synthetic-programming/12-CONTEXT.md:131` explicitly defers interactive GETKEY as SYNT-06 with note "requires event loop redesign". This remains unimplemented as of v4.2.

### UG-04: CHS during non-EEX number entry flushes stack (= FGAP-05)

Real HP-41: CHS during mantissa entry toggles sign of the entry buffer. Emulator: `call_dispatch(Op::Chs)` flushes entry_buf first (via `flush_entry_buf` inside `dispatch()`), then negates X. The EEX case is correctly handled in-buffer at `hp41-cli/src/app.rs:691-707`, but the non-EEX mantissa case falls through to dispatch. This causes a spurious stack lift when CHS is pressed during digit entry.

### UG-05: AON flag stored but no frontend reads it (= FGAP-07)

`op_aon` at `hp41-core/src/ops/display_ops.rs:53-56` sets flag 48. Confirmed by `grep -rn "flag.*48" hp41-cli/src/` (0 matches) and `grep -n "flag.*48" hp41-gui/src/App.tsx` (0 matches). Neither frontend reads flag 48 to drive alpha auto-display after operations.

### UG-06: FACT(27..=69) returns Overflow instead of scientific result

`op_fact` at `hp41-core/src/ops/math.rs:480-482`: `Decimal::from_f64(acc).ok_or(HpError::Overflow)?` fails for n≥27 because `27! ≈ 1.089E28` overflows rust_decimal's 28-digit precision. Real HP-41 would display the result in scientific notation (clipped to 10 significant digits). The README at line 170 documents this as a known divergence but it remains unfixed.

### UG-07: CATALOG 1 — no interactive scroll, no R/S stop

`op_catalog` at `hp41-core/src/ops/program.rs:287-391`: writes all catalog lines to `print_buffer` in a single synchronous loop with no yield points. Comment at line 346 explicitly defers per-line yield. Real HP-41 scrolls interactively with R/S to stop.

### UG-08: VIEW/AVIEW in programs not visible during execution (= FGAP-10)

`op_view` sets `display_override` mid-`run_loop`. Since `run_loop` runs synchronously to completion, the frontend only sees the final state (after the program finishes). The display override is therefore invisible during program execution. This is the same structural issue as UG-01 (PSE timing).

---

## Uncertain / Needs-User-Decision

| ID | Name | Question | Recommendation |
|----|------|----------|----------------|
| UNC-01 | Back-arrow clears error message | Real HP-41: pressing ← when an error is displayed clears the error and returns to normal display. Emulator: back-arrow calls `backspace_entry()` which CLXes when entry_buf is empty — but the error message in `app.message` may persist. Behavior depends on CLI status bar rendering vs. HP-41 display. Needs hardware verification before classifying. | Investigate on real hardware or trusted emulator (Free42). |
| UNC-02 | Flag 21/25/55 printer-related system flags | HP-41CV OM specifies system flags 21 (printer connected), 25 (printer enabled for trace), 55 (low battery). The emulator's 56-bit `flags: u64` stores these but PRX/PRA/PRSTK (in `hp41-core/src/ops/print.rs`) do NOT check flags 21 or 25 before printing. Whether this constitutes a divergence depends on whether users actually set these flags. | Likely low-impact. Verify against OM whether PRX should silently no-op when flag 21 is clear. |
| UNC-03 | MEMORY LOST display on SIZE reduction | HP-41 shows "MEMORY LOST" on display when SIZE is reduced (truncating registers). `op_size` in `hp41-core/src/ops/program.rs:262` correctly truncates the tail (comment: "hardware-faithful 'MEM LOST'") but there is no evidence of a "MEMORY LOST" display_override being set. Whether the user should see this message on screen needs verification. | Read `op_size` carefully; add `display_override = Some("MEMORY LOST".to_string())` if hardware confirmation shows this message. |

---

## Sources

- `docs/hp41cv-divergences.md` — D-CV-01..D-CV-04 (established 2026-06-03)
- `docs/hp41-math1-divergences.md` — D-30-01..D-30-07 (established Phase 30)
- `docs/hp41-stat1-divergences.md` — D-35-01..D-35-13 (established Phase 35)
- `docs/hp41-time-divergences.md` — D-40-01..D-40-05 (established Phase 40)
- `docs/hp41-advantage-divergences.md` — D-45-01..D-45-09 (established Phase 45)
- `docs/hp41-xmem-divergences.md` — D-52-01..D-52-02 (established Phase 52)
- Code grep verification: `hp41-core/src/ops/program.rs`, `hp41-core/src/ops/registers.rs`, `hp41-core/src/ops/print.rs`, `hp41-core/src/ops/display_ops.rs`, `hp41-core/src/ops/math.rs`, `hp41-cli/src/app.rs`, `hp41-cli/src/ui.rs`, `hp41-gui/src/App.tsx`, `hp41-gui/src-tauri/src/commands.rs`
- `.planning/milestones/v1.1-phases/12-synthetic-programming/12-CONTEXT.md` — SYNT-06 deferred item
- `.planning/PROJECT.md` — Deferred items table
- `.planning/STATE.md` — Current state and deferred items
- HP-41CX Owner's Manual (HP 00041-90028, 1983)
- HP-41C/CV Owner's Manual (HP 00041-90001)
- HP Math Pac I Owner's Manual (HP 00041-90034, 1979)
- HP Time Module Owner's Manual (HP 00041-90035, 1982)

---

*Last updated: 2026-06-06 — v4.3 Hardware Fidelity milestone audit.*
