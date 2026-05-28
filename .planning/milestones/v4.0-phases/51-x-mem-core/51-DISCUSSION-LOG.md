# Phase 51: X-MEM Core - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-28
**Phase:** 51-x-mem-core
**Areas discussed:** Capacity model, EMREG + active file, SAVED/GETD block, Error & overwrite behavior

---

## Capacity model

| Option | Description | Selected |
|--------|-------------|----------|
| Fixed 600 registers | Fully-expanded HP-41CX (124 + 2×238). EMROOM = 600 − used; SAVEP/SAVED → "NO ROOM" on overflow. Exercises the error path. | ✓ |
| Fixed 124 registers | Base built-in CX X-Memory only. Stricter — a single data file nearly fills it. | |
| Effectively unlimited | Vec grows freely; EMROOM reports a large constant. Simplest but EMROOM meaningless, never fails. | |

**User's choice:** Fixed 600 registers
**Notes:** Register-accounting rule left to Claude's discretion with a faithful default (⌈bytes/7⌉+1 for programs, N+1 for data), to be OM-verified by the researcher.

---

## EMREG + active file

### Active-file selection

| Option | Description | Selected |
|--------|-------------|----------|
| GETD/SAVED set it | Most-recently saved/retrieved DATA file becomes active (`xmem_active_file`). No new op. | ✓ |
| Name in ALPHA per call | EMREG reads name from ALPHA + index from X each call; no persistent pointer. | |

**User's choice:** GETD/SAVED set it

### EMREG read-only vs read/write

| Option | Description | Selected |
|--------|-------------|----------|
| Read-only | EMREG N → push stored value; whole-block writes via SAVED. Minimal. | |
| Read + write | EMREG reads; a write direction stores into register N in place. | ✓ |

**User's choice:** Read + write

### EMREG write mechanism (follow-up)

| Option | Description | Selected |
|--------|-------------|----------|
| Pair: read op + store op | GETRX/SAVERX-style: EMREG recalls; a companion store op writes. Adds an 8th op; faithful, unambiguous. | ✓ |
| Single EMREG, index Y + value X | One op, store-through; read path awkward. | |
| Single EMREG, flag-controlled | One op, direction picked by a system flag; implicit/stateful. | |

**User's choice:** Pair: read op + store op
**Notes:** Adds an 8th op to the phase → 4-way exhaustive match impact. Store-op mnemonic is Claude's discretion (prefer HP-41CX SAVERX/GETRX fidelity).

---

## SAVED/GETD block

| Option | Description | Selected |
|--------|-------------|----------|
| Full register set (per SIZE) | SAVED captures R00..R(SIZE-1); GETD replaces state.regs wholesale, pads to 100. Reuses capture_data_card / load_data_card. | ✓ |
| Block control word in X | Faithful bbb.eee block transfer; needs new string-split parsing; no helper reuse. | |

**User's choice:** Full register set (per SIZE)

---

## Error & overwrite behavior

### Duplicate name on SAVEP/SAVED

| Option | Description | Selected |
|--------|-------------|----------|
| Overwrite in place | Replaces existing file. Pragmatic — no purge op in scope, so erroring would make files unreplaceable. Diverges from HP-41CX "DUP FL". | ✓ |
| 'DUP FL' error + add PURFL | Faithful CX behavior, but requires adding a 9th purge op — expands scope. | |

**User's choice:** Overwrite in place
**Notes:** Documented divergence from HP-41CX; PURFL/CLFL noted as a deferred idea.

### GETP load semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse insert_program_ops | RDPRGM semantics: empty → replace+reset pc; else insert after pc. Reuses tested helper. | ✓ |
| Always replace | Wipes program memory and loads; simpler but discards in-memory program. | |

**User's choice:** Reuse insert_program_ops

---

## Claude's Discretion

- Register-accounting formula (faithful 7-bytes/register + header; OM-verify).
- EMREG store-op mnemonic naming (prefer HP-41CX SAVERX/GETRX fidelity).
- Concrete `HpError` variants for type-mismatch ("FL TYPE"), missing file ("not found"), no-active-file / out-of-range EMREG — all surfaced, never swallowed (D-07).
- EMDIR catalog line format (follow existing CATALOG / ALMCAT print-buffer pattern; show name / type / size-in-registers).
- Op resolution: X-MEM ops as built-ins via XEQ-by-name (not an XROM module; no XROM bit allocated) — confirm during planning.

## Deferred Ideas

- PURFL / CLFL purge-file op (enables true HP-41CX "DUP FL" duplicate semantics) — future phase / v4.1.
- Block control-word SAVED/GETD (bbb.eee in X) — faithful partial-block transfer; deferred in favor of full-set reuse.
- ASCII + STATUS X-MEM file types — already XMEM-F01 / XMEM-F02 (v4.1).
