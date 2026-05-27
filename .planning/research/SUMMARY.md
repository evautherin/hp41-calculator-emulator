# Project Research Summary

**Project:** HP-41 Calculator Emulator -- v3.3 Advantage Pac Emulation
**Domain:** Behavioral emulation of the HP-41 Advantage Pac (XROM 22 + XROM 24, OM 00041-90482) -- the fourth XROM application module, completing all remaining HP-41 module emulation
**Researched:** 2026-05-25
**Confidence:** MEDIUM-HIGH

## Executive Summary

The HP-41 Advantage Pac (released July 1985) is HP's first 12K bank-switched ROM module, occupying two XROM IDs (22 and 24) with ~117 functions across four sections: ADV CONV (12 bitwise/base-conversion ops), ADV MTRX (~52 named-matrix ops), ADV MATH (~47 complex/solver/polynomial/curve-fit ops), and ADV TVM (6 financial time-value-of-money ops). This is the largest XROM module milestone to date.

**Critical scope clarification:** The "Advanced Matrix Pac" referenced in PROJECT.md is a community hobbyist ROM by Angel Martin (XROM 12), NOT an official HP product. All matrix targets listed in PROJECT.md (M+, MAT*, INV-as-transpose, V+, VDOT, IDN) are present in the official Advantage Pac XROM 22 ADV MTRX section. Implementing the full Advantage Pac automatically covers all "Advanced Matrix" targets. No separate XROM module is needed.

## Stack Additions

**Zero new Rust runtime dependencies.** All algorithms (Laguerre for FROOT, Romberg for FINTG, Newton for TVM *I, Brent for FSOLVE) are implementable from primary sources in under 200 LOC each. The ADR-v3.1-002 zero-new-runtime-deps invariant holds across v3.3.

**XROM registration:** Two new `XromModule` constants (`ADV_MATH_A` id=22, `ADV_MATH_B` id=24) registered in `math1/xrom.rs` freeze carve-out. `xrom_resolve` gains bit-3 and bit-4 arms (XROM 22 and 24 respectively). `default_xrom_modules()` changes from `0b0000_0111` to `0b0001_1111`. `migrate_after_load()` sets bits 3+4 for v3.2 save files.

**New CalcState fields:**
- `adv_matrices: Vec<AdvMatrix>` (`#[serde(default)]`) -- X-MEM named-matrix storage (incompatible with Math Pac I's R14/R15+ register layout)
- `adv_tvm_state: Option<TvmState>` (`#[serde(default)]`) -- TVM register persistence

**Visibility promotions in math1/ (freeze carve-outs, not algorithm changes):**
- `complex_atan2` in `math1/complex.rs`: `pub(super)` -> `pub(crate)` (share with `advantage/complex.rs`)
- `USER_CALLBACK_MAX_STEPS` in `math1/mod.rs`: promote to `pub(crate)` if needed by FSOLVE/FINTG

## Feature Table Stakes

| Section | Function Count | Complexity | Dependencies |
|---------|---------------|------------|--------------|
| ADV CONV | 12 | LOW | Pure integer ops, no existing overlap |
| ADV MTRX | ~52 | MEDIUM-HIGH | New X-MEM named-matrix model (NOT R14/R15+) |
| ADV MATH | ~47 | HIGH | Extends complex stack, new FROOT/FINTG/FSOLVE |
| ADV TVM | 6 | MEDIUM | Newton's method for *I; self-contained |

**Key function overlaps with Math Pac I (coexist, not replace):**
- FROOT (Laguerre, arbitrary degree) vs POLY/ROOTS (Bairstow, degree 2-5)
- FINTG/INTEG (Romberg) vs INTG (Simpson)
- FSOLVE (Brent) vs SOLVE (modified secant)
- New complex ops (e^Z, LNZ, SINZ, COSZ, TANZ, Z^N, Z^W) extend Math Pac I complex stack

## Watch Out For

1. **math1/ freeze violation (CRITICAL):** All Advantage Pac code must go in `advantage/`, not extend `math1/`. Only sanctioned carve-outs (visibility promotions in xrom.rs, modal.rs, complex.rs) are permitted.
2. **PROOT algorithm choice (CRITICAL):** Math Pac I Bairstow deflation fails for degree 6+ repeated roots. Use Laguerre's method. Derive oracle test cases from scipy/numpy BEFORE writing Rust.
3. **INTG vs INTEG mnemonic collision (HIGH):** One letter difference. `xrom_shadowing.rs` CI gate catches it, but verify against OM first.
4. **Matrix register layout conflict (HIGH):** ADVMTRX uses named X-MEM files, NOT R14/R15+. Never touch `state.matrix_dim` or `state.matrix_active_reg` from Advantage Pac code.
5. **Complex stack sharing (HIGH):** Promote `complex_atan2` to `pub(crate)` rather than duplicating the Pitfall-6 fix.

## Open Questions (resolve in Phase 43 pre-work)

1. FROOT calling convention: degree from X register vs modal prompt?
2. Complete ADVMTRX 52-op list with exact XROM 22 sub-numbers
3. CATALOG 2 exact display strings for XROM 22 + XROM 24
4. NOT/AND/OR/XOR integer word size (user-settable WS?)
5. Complex stack convention in ADVMATH: same as Math Pac I (Y=Im, X=Re)?
6. FSOLVE/FINTG mutual nesting architecture (defer if complex)
7. TVM register persistence model

## Build Order

5-phase structure (Phases 43-47), mirroring v3.1 and v3.2:

1. **Phase 43 -- hp41-core:** XROM registration + all ~117 Op variants + ADV CONV + ADV MTRX + ADV MATH + ADV TVM. Largest core phase. Pre-work required: resolve 7 open questions from OM 00041-90482.
2. **Phase 44 -- hp41-cli:** 5th JSON pool (`hp41-advantage-functions.json`), `op_display_name` arms, `?` overlay "Advantage Pac" section, xrom_shadowing extension.
3. **Phase 45 -- Documentation:** ADRs (named-matrix model, FROOT algorithm, dual-XROM design), divergences catalog, function matrix, CLAUDE.md v3.3 additions block.
4. **Phase 46 -- hp41-gui:** `op_display_name` arms, HelpOverlay 5th section, CATALOG 2 entries for XROM 22+24.
5. **Phase 47 -- Test Hardening:** Unified meta-gates, FROOT accuracy oracles, backward compat (v3.2->v3.3), E2E smoke.

---
*Synthesized from STACK.md + FEATURES.md + ARCHITECTURE.md + PITFALLS.md on 2026-05-25*
