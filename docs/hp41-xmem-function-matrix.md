# HP-41CX Extended Memory Function Matrix

> Generated from `docs/hp41-xmem-functions.json` via `just docs-matrix`.
> Edit the JSON, regenerate this file, commit both.

## Implemented (v2.x)

| Op | Display | Category | Status | Phase | Key Path | Description |
|----|---------|----------|--------|-------|----------|-------------|
| EmDir | EMDIR | Extended Memory | ✓ v2.x | 52 | `XEQ "EMDIR"` | Catalog all X-MEM files; names + register counts printed to PRINT buffer |
| EmReg | EMREG | Extended Memory | ✓ v2.x | 52 | `XEQ "EMREG"` | Return number of registers used by X-MEM file named by ALPHA in X |
| EmRoom | EMROOM | Extended Memory | ✓ v2.x | 52 | `XEQ "EMROOM"` | Return number of available X-MEM registers in X |
| GetD | GETD | Extended Memory | ✓ v2.x | 52 | `XEQ "GETD"` | Load all data registers from X-MEM file named by ALPHA register |
| GetP | GETP | Extended Memory | ✓ v2.x | 52 | `XEQ "GETP"` | Load program from X-MEM file named by ALPHA into current program memory |
| SaveD | SAVED | Extended Memory | ✓ v2.x | 52 | `XEQ "SAVED"` | Save all 100 data registers to X-MEM file named by ALPHA register |
| SaveP | SAVEP | Extended Memory | ✓ v2.x | 52 | `XEQ "SAVEP"` | Save current program to X-MEM file named by ALPHA register |
| SaveRx | SAVERX | Extended Memory | ✓ v2.x | 52 | `XEQ "SAVERX"` | Save X register value to first register of X-MEM file named by ALPHA |

## v3.x Deferred (Module Pacs)

_None._
