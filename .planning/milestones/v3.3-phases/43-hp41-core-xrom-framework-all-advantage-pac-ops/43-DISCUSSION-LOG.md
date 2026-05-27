# Phase 43: hp41-core — XROM Framework + All Advantage Pac Ops - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-25
**Phase:** 43-hp41-core-xrom-framework-all-advantage-pac-ops
**Areas discussed:** Named-matrix model, FROOT/FINTG/FSOLVE callbacks, ADV CONV word size, TVM register model

---

## Named-Matrix Model

### Maximum matrix size

| Option | Description | Selected |
|--------|-------------|----------|
| 14x14 | Matches HP documentation and available data registers | |
| 20x20 | More generous cap; some community programs push beyond 14x14 | |
| You decide | Let Claude pick based on OM constraints | ✓ |

**User's choice:** You decide
**Notes:** Claude to determine from OM 00041-90482 constraints and practical memory considerations.

### Simultaneous matrix count

| Option | Description | Selected |
|--------|-------------|----------|
| 5 matrices max | Matches HP-41 Advantage Pac X-MEM module file limit | |
| Unlimited (Vec grows) | No artificial cap — let the Vec grow | ✓ |

**User's choice:** Unlimited (Vec grows)
**Notes:** More permissive than hardware but simpler to implement.

### Current-matrix selection mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| ALPHA register names it | Matrix matching ALPHA string is target | |
| Dedicated current_matrix field | Separate CalcState field tracks active matrix | |
| You decide | Let Claude determine from OM conventions | ✓ |

**User's choice:** You decide
**Notes:** Claude to determine from OM 00041-90482 conventions.

### I/J index storage

| Option | Description | Selected |
|--------|-------------|----------|
| Per-matrix I/J | Each AdvMatrix tracks its own (i, j) position | |
| Single global I/J | One (i, j) pair on CalcState, shared across all matrices | |
| You decide | Let Claude determine from OM behavior | ✓ |

**User's choice:** You decide
**Notes:** Claude to determine from OM 00041-90482 behavior.

---

## FROOT/FINTG/FSOLVE Callbacks

### Callback architecture

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse run_loop + new state fields | Same re-entrancy mechanism; add adv_ transient fields | |
| Completely separate mechanism | New entry point, more isolation from math1/ frozen code | |
| You decide | Let Claude determine cleanest architecture | ✓ |

**User's choice:** You decide
**Notes:** Claude to determine given math1/ freeze constraint.

### Solver nesting

| Option | Description | Selected |
|--------|-------------|----------|
| Defer nesting to post-v3.3 | Single-level callbacks only | |
| Support one level of nesting | Allow FINTG inside FSOLVE (common use case) | ✓ |

**User's choice:** Support one level of nesting
**Notes:** User explicitly requires one level of nesting (e.g., FINTG inside FSOLVE's callback). Deeper mutual nesting (NEST-01) deferred to post-v3.3.

### FROOT calling convention

| Option | Description | Selected |
|--------|-------------|----------|
| Degree from X register | User places degree in X before calling FROOT | |
| Modal prompt asks degree | FROOT opens a 'DEGREE?' modal prompt | |
| You decide | Let Claude determine from OM conventions | ✓ |

**User's choice:** You decide
**Notes:** Claude to determine from OM 00041-90482 Section 3.

---

## ADV CONV Word Size

### Integer word size

| Option | Description | Selected |
|--------|-------------|----------|
| 36-bit fixed | Matches HP-41's 10-digit BCD mantissa capacity per OM Section 1 | ✓ |
| 32-bit fixed | Standard word size, simpler with Rust u32 | |

**User's choice:** 36-bit fixed
**Notes:** User explicitly chose 36-bit to match OM 00041-90482 specification.

### Overflow behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Silent truncation (mask to 36 bits) | Results masked to lower 36 bits, no error | ✓ |
| Error on overflow | Return DATA ERROR if result exceeds 36 bits | |

**User's choice:** Silent truncation (mask to 36 bits)
**Notes:** Consistent with hardware behavior where extra bits don't exist.

---

## TVM Register Model

### TVM state persistence

| Option | Description | Selected |
|--------|-------------|----------|
| Persistent (#[serde(default)]) | TVM registers survive save/load | ✓ |
| Transient (#[serde(default, skip)]) | TVM registers reset on load | |

**User's choice:** Persistent (#[serde(default)])
**Notes:** Matches financial calculator UX expectations.

### BEGIN/END payment mode

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, with a mode flag | Standard financial calculator feature, add tvm_begin_mode: bool | |
| Ordinary annuity only | Simpler, payments at end of period only | |
| You decide | Let Claude determine from OM TVM section | ✓ |

**User's choice:** You decide
**Notes:** Claude to determine from OM 00041-90482 TVM section.

### *I non-convergence handling

| Option | Description | Selected |
|--------|-------------|----------|
| DATA ERROR after N iterations | Bounded iteration cap, consistent with SOLVE/INTG | |
| Best-guess + flag | Return approximation with convergence indicator | |
| You decide | Let Claude determine from OM and existing patterns | ✓ |

**User's choice:** You decide
**Notes:** Claude to determine based on OM behavior and existing solver error handling precedent.

---

## Claude's Discretion

- Maximum matrix size cap (D-43.2)
- Current-matrix selection mechanism (D-43.3)
- I/J index storage location (D-43.4)
- Callback mechanism architecture (D-43.6)
- FROOT calling convention (D-43.8)
- BEGIN/END payment mode (D-43.12)
- *I non-convergence behavior (D-43.13)

## Deferred Ideas

- NEST-01: Deep FROOT/FINTG mutual nesting (re-entrant X-MEM buffer stack for arbitrary depth)
- XMEM-01: Full Extended Memory model (EMDIR/EMROOM/EMREG)
