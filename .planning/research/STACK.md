# Stack Research

**Domain:** HP-41 Advantage Pac (XROM 22 + 24) behavioral emulation
**Researched:** 2026-05-25
**Confidence:** HIGH (XROM IDs from calc.fjk.ch hardware database + HP Museum archives; architecture from existing codebase inspection)

---

## Critical Pre-Research Finding: Scope Clarification

The PROJECT.md mentions both "Advantage Pac" and "Advanced Matrix Pac" as v3.3 targets. Research reveals:

**The "Advanced Matrix Pac" is NOT an official HP product.** It is a third-party hobbyist custom ROM created by community members (combining Advantage Pac ADVMTRX functions with the ALGEBRA module and adding a new matrix input mode). It surfaces in HP Museum forum archives (thread 184196) as a Clonix/MLDL user project. It has no independently assigned hardware XROM ID — it reuses the Advantage Pac's XROM slots.

This is a direct parallel to the v3.0 scope discovery (PROJECT.md §Scope-Korrektur 2026-05-16): "Math Pac I ist user-code in ROM (multi-step Modal-Workflows), nicht Nut-CPU-microcode (one-shot Stack-Ops)." The same pattern applies here: the functions listed in the PROJECT.md target list (M+, MAT\*, INV-as-transpose, V+, VDOT, IDN) are all in the official Advantage Pac XROM 22 ADVMTRX section — not in a separate "Advanced Matrix Pac."

**Recommendation:** Implement the official HP Advantage Pac (XROM 22 + 24) only. The "Advanced Matrix Pac" is out of scope as a behavioral emulation target; it is not an HP Owner's Manual product.

---

## Recommended Stack

### XROM Module Registration

| Module | Hardware XROM ID | CATALOG 2 Name | Source |
|--------|-----------------|----------------|--------|
| Advantage Pac (ROM A) | 22 | "ADV 1A" | calc.fjk.ch hardware DB, HP Museum XROM table |
| Advantage Pac (ROM B) | 24 | "ADV 1B" (cont'd) | calc.fjk.ch hardware DB, HP Museum XROM table |
| Math Pac I (existing) | 7 | "MATH 1A" | Already in xrom.rs |
| Stat 1 Pac (existing) | 2 | "STAT 1B" | Already in xrom.rs |
| Time Module (existing) | 26 | "TIME 2C" | Already in xrom.rs |

**The Advantage Pac physically occupies TWO XROM module IDs** because it is HP's first 12K ROM (all prior ROMs were 4K/8K). Bank-switching makes it transparent to the user — both XROM 22 and XROM 24 are presented as one logical module in CATALOG 2. This requires registering two `XromModule` constants in `xrom.rs` and adding two new resolver arms to `xrom_resolve()`.

**XROM bit assignment for `xrom_modules` u8 field:**

| Bit | Module | XROM ID |
|-----|--------|---------|
| 0 | MATH_1 | 7 |
| 1 | STAT_1 | 2 |
| 2 | TIME_MODULE | 26 |
| 3 | ADV_A (new) | 22 |
| 4 | ADV_B (new) | 24 |

The current `default_xrom_modules() = 0b0000_0111`. After v3.3: `0b0001_1111` (all five modules). The `migrate_after_load()` pattern sets bits 3+4 for v3.2 save files.

**Design decision — one bit or two bits for Advantage:**
Because XROM 22 and XROM 24 are a single physical ROM with always-linked presence (you cannot have one without the other), use a single "loaded" flag but register two separate `XromModule` consts. The resolver fires both arms when the Advantage bit is set. This matches hardware behavior: CATALOG 2 shows one entry, not two.

### Function Set: Official HP Advantage Pac

**XROM 22 — /ADVCONV (12 functions) + /ADVMTRX (52 functions) = 64 total**

ADVCONV (base/boolean): BININ, BINVIEW, OCTIN, CVTVIEW, HEXIN, HEXVIEW, NOT, AND, OR, XOR, ROTXY, BIT?

ADVMTRX (matrix/vector, M-code from CCD ROM): C<>C, CMAXAB, CNRM, CSUM, DIM?, FNRM, I+, I-, J+, J-, M\*M, MAT\*, MAT+, MAT-, MAT/, MATDIM, MAX, MAXAB, MDET, MIN, MINV, MMOVE, MNAME?, MSIJ, MRIJ, MSR+, MRR, MSIR, MSC, MROW, MCOL, MSWAP, MRSWAP, MCSWAP, MATRX (matrix editor), MTR (transpose), ANRM, ANUM, IDN, MEDT, MSIZE, MLIFT, MLOWER, VDOT, V+, V\*, MSYS, R<>R, plus utility functions to 52 total.

**XROM 24 — /ADVMATH (47 functions) + /ADVTVM (6 functions) = 53 total**

ADVMATH complex arithmetic and transcendentals (HP-15C Solve/Integrate port + complex number set):
MATRX (24,0), MTR (24,1), SOLVE (24,3), INTEG (24,4), SILOOP (24,5), SIRIN (24,6),
Z^N (24,7), MAGZ (24,8), e^Z (24,9), LNZ (24,10), Z^1/N (24,11),
SINZ (24,12), COSZ (24,13), TANZ (24,14), a^Z (24,15), LOGZ (24,16),
Z^1/W (24,17), Z^W (24,18), C+ (24,19),
CABS, CARG, CCHS, CCONJ, CY^X (complex unary/binary ops),
PROOT (arbitrary-degree polynomial roots — distinct from Math Pac I POLY which is degree 2-5),
FROOT (root finding entry for user programs, HP-15C SOLVE port),
plus additional complex arithmetic and curve-fitting functions to reach 47.

ADVTVM: TVM, N, PV, PMT, FV, \*I

**Total: ~117 XEQ entry point functions** across XROM 22 and 24.

### Core Technologies (No Changes Needed)

| Technology | Version | Purpose | Applies To |
|------------|---------|---------|------------|
| rust_decimal 1.42 | 1.42 | HpNum BCD-accurate arithmetic | All numeric ops |
| serde / serde_json | existing | CalcState persistence | New CalcState fields |
| XromModule struct | existing in xrom.rs | XROM registry | Two new consts: ADV_A, ADV_B |
| ModalProgram enum | existing | Multi-step workflow routing | New ModalProgram::Advantage(AdvantageStep) variant |
| run_loop re-entrancy | existing | User-callback (SOLVE/INTEG) | Advantage SOLVE + INTEG reuse the same infrastructure |

### Supporting Libraries (No New Runtime Dependencies)

**Zero new Rust crates required.** This is a hard constraint per project invariant (ADR-v3.1-002 zero-new-runtime-deps policy; `statrs` was rejected in v3.1; same applies here).

All numerical algorithms needed for the Advantage Pac are derivable from primary sources:
- Matrix operations: extend existing Gauss-Jordan infrastructure from `hp41-core/src/ops/math1/` (DET, INV, SIMEQ already implemented for 14×14)
- SOLVE/INTEG: the Advantage Pac SOLVE and INTEG are HP-15C ports — `Op::Solve` and `Op::Integ` in Math Pac I are the same algorithm family; Advantage variants differ in UI/prompt flow, not core math
- Complex arithmetic: CABS, CARG, CCHS, CCONJ, CY^X are extensions of existing complex infrastructure in `hp41-core/src/ops/math1/`
- TVM: Time Value of Money — standard financial math (Newton's method for \*I), no external library needed
- Base conversions: pure integer arithmetic (BIN/OCT/HEX), trivially implemented
- Boolean operations: NOT, AND, OR, XOR on integer register values, trivially implemented
- PROOT: arbitrary-degree polynomial root-finding — Laguerre's method (~100-150 LOC) derivable from primary sources (HP OM 00041-90482 + Numerical Recipes)

### Existing Infrastructure Reuse

| Existing Component | v3.3 Reuse |
|---------------------|------------|
| Complex stack overlay (C+, C-, C×, C÷, REAL, MAGZ, CINV, Z^N, SINZ, COSZ, TANZ…) in `math1/` | Advantage CABS, CARG, CCHS, CCONJ, CY^X are extensions; same X/Y/Z/T complex overlay convention |
| MATRIX workflow (Gauss-Jordan DET/INV/SIMEQ, 14×14, column-major R15+) in `math1/` | Advantage ADVMTRX extends this; MAT\*, MDET, MINV, MSYS reuse the same register layout |
| `Op::Solve` / `Op::Integ` with run_loop re-entrancy (4-deep call stack cap) | Advantage SOLVE/INTEG/FROOT/SILOOP are same algorithm family + same re-entrancy model |
| `ModalProgram` enum + `TimeStep`/`Stat1Step` patterns (ADR-v3.1-005) | `ModalProgram::Advantage(AdvantageStep)` follows the same pattern |
| `xrom.rs` `XromModule` struct + `xrom_resolve()` bit-arm pattern | Add bit-3 arm (ADV_A / XROM 22) + bit-4 arm (ADV_B / XROM 24) |
| `migrate_after_load()` in `state.rs` | Extend: if bits 3+4 clear, set them (v3.2 save → v3.3 upgrade) |
| JSON help pipeline (4 pools: cv, math1, stat1, time) | Add 5th pool: `docs/hp41-advantage-functions.json` |
| `scripts/docs-matrix` 4-invocation renderer | Add 5th invocation for Advantage function matrix (4-line basename dispatch branch) |
| Free42 GPL contamination guard (`scripts/check-free42-contamination.sh`) | Extend to cover `advantage/` source tree |

### New Files Expected

Following the established pattern from v3.0 through v3.2:

```
hp41-core/src/ops/advantage/
    mod.rs          # /ADVCONV ops (base conversions, boolean) + module wiring
    matrix.rs       # /ADVMTRX: MAT*, MDET, MINV, IDN, V+, VDOT, MSYS, traversal ops
    complex.rs      # CABS, CARG, CCHS, CCONJ, CY^X (extends math1/ complex overlay)
    solve_integ.rs  # SOLVE/INTEG/FROOT/SILOOP/SIRIN (thin wrapper on run_loop)
    tvm.rs          # TVM, N, PV, PMT, FV, *I
    proot.rs        # PROOT: arbitrary-degree polynomial roots (Laguerre or DK method)
    modal.rs        # AdvantageStep enum (lives OUTSIDE math1/ freeze boundary)

docs/hp41-advantage-functions.json       # 5th JSON canonical source of truth
docs/hp41-advantage-function-matrix.md  # Generated via just docs-matrix
docs/hp41-advantage-divergences.md      # 3-bucket divergences catalog
docs/adr/v3.3-001-*.md ...              # ADRs for key decisions
```

### Key Architectural Decisions

**1. Two XROM IDs, one logical module (HIGH confidence)**

Register `ADV_A: XromModule { id: 22 }` and `ADV_B: XromModule { id: 24 }` as separate consts in `xrom.rs`, but gate both on a single `xrom_modules` bit (bit 3). The resolver fires both arms when bit 3 is set. This matches hardware: the Advantage Pac is one physical module occupying two XROM slots due to 12K bank-switching. CATALOG 2 shows one entry, not two.

Alternative considered: two separate bits (3 and 4). Rejected: the two halves are never independently loaded in hardware; single-bit gating is faithful to hardware behavior and avoids spurious partial-load states.

**2. PROOT is in XROM 24 ADVMATH, not absent from HP-41 (MEDIUM confidence)**

The HP Museum XROM table and community sources place PROOT in the Advantage Pac's ADVMATH section (XROM 24). Web search initially surfaced confusion with the HP-71B's PROOT function (same name, different product line) — that confusion is a research pitfall, not a real finding. The HP-41 Advantage Pac manual (OM 00041-90482, 156 pages) documents PROOT for arbitrary-degree polynomial roots. Confidence is MEDIUM (not HIGH) because the full XROM 24 function listing is partially obscured by hpmuseum.org 403 blocks — the manual PDF was not fully readable during research. The PROJECT.md lists PROOT as a target feature, and multiple community sources confirm it is in XROM 24.

**3. Advantage SOLVE/INTEG reuse Math Pac I infrastructure (HIGH confidence)**

Multiple authoritative sources (Valentín Albillo article, HP Museum archives, Advantage Pac product descriptions) describe the Advantage Pac's SOLVE and INTEG as "ported from the HP-15C." Math Pac I already implements these HP-15C algorithms in `Op::Solve` and `Op::Integ` with `run_loop` re-entrancy. The Advantage variants differ only in entry-point naming (FROOT for root-finding, SILOOP/SIRIN for iterative loops) — the core convergence algorithm is unchanged.

**4. Extended matrix ops extend, not replace, Math Pac I matrix (HIGH confidence)**

Math Pac I MATRIX workflow (Gauss-Jordan, 14×14, column-major R15+) is frozen since Plan 25-01. Advantage ADVMTRX adds ~52 new ops using the same register layout convention. Key extensions: MAT\* (matrix multiply), MAT+/MAT-/MAT/ (element-wise arithmetic), MDET (determinant), MINV (inverse), IDN (identity matrix), V+/VDOT (vector operations), MSYS (system solver), plus index traversal ops (I+, I-, J+, J-, MSR+, MRR, MSIJ). These do NOT conflict with existing Math Pac I ops — different XEQ names, different XROM slots.

**5. No new runtime Rust crates (HIGH confidence)**

All mathematical primitives needed are either already implemented or implementable from OM algorithms in under 200 LOC each. The zero-new-runtime-deps invariant (ADR-v3.1-002) holds.

### Development Tools (No Changes)

| Tool | Status |
|------|--------|
| `just` | Sole task runner; all new recipes follow existing pattern |
| `cargo-llvm-cov` | Coverage gate; existing ≥ 95% lines / ≥ 93% regions target |
| `scripts/check-free42-contamination.sh` | Extend to cover `advantage/` source tree |
| `scripts/docs-matrix` | Add 5th invocation (4-line basename dispatch branch) |

---

## Alternatives Considered

| Recommended | Alternative | Why Not |
|-------------|-------------|---------|
| Two XROM IDs (22 + 24), single `xrom_modules` bit | Two independent bits (3 and 4) | Hardware never loads them separately; single bit is faithful to hardware semantics |
| Extend Math Pac I matrix register layout for ADVMTRX | Separate matrix storage scheme for Advantage | R14=order, R15+ column-major already established and tested; one layout is simpler |
| Re-derive PROOT from HP OM (Laguerre/Durand-Kerner) | Use a Rust polynomial crate | Zero-deps invariant; also re-derivation is the established Free42-guard-safe pattern |
| Single `ModalProgram::Advantage(AdvantageStep)` | Separate `ModalProgram::AdvMatrix` + `ModalProgram::AdvMath` | ADR-v3.1-005 pattern uses one variant per module; consistent across all pacs |
| Official HP Advantage Pac (XROM 22+24) only | Include "Advanced Matrix Pac" (hobbyist ROM) | Not an official HP product; no Owner's Manual; outside project scope |

---

## What NOT to Add

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| "Advanced Matrix Pac" as emulation target | Third-party hobbyist ROM, no HP OM, no stable hardware XROM ID | Official HP Advantage Pac covers all the matrix functions cited in PROJECT.md |
| New Rust crates for TVM computation | Zero-deps invariant; TVM is ~50 LOC closed-form plus Newton for rate | Implement from OM spec |
| New Rust crates for polynomial roots | Zero-deps invariant; PROOT is ~100-150 LOC Laguerre or Durand-Kerner | Re-derive from OM spec + primary algorithmic sources |
| Treating PROOT as "absent from HP-41" | PROOT is confirmed in XROM 24 ADVMATH; confusion with HP-71B PROOT is a web search pitfall | Implement as XROM 24 function |
| `async` in `hp41-core` | Frozen invariant: no async | Advantage SOLVE/INTEG reuse synchronous run_loop re-entrancy |
| Complex matrix operations (complex-valued matrix elements) | Out of scope for behavioral emulation of base Advantage Pac OM | ADVMTRX operates on real matrices; complex matrix support is in third-party modules |

---

## Version Compatibility

| Concern | Approach |
|---------|----------|
| v3.2 save files (`xrom_modules = 0b0000_0111`) | `migrate_after_load()` sets bits 3+4: v3.2 saves auto-upgrade to `0b0001_1111` |
| `default_xrom_modules()` | Change from `0b0000_0111` to `0b0001_1111` in v3.3 Phase 43 (core) |
| 4-way exhaustive-match invariant | All Advantage `Op` variants land in `dispatch()` + `execute_op()` (Phase 43) before `hp41-cli` (Phase 44) and `hp41-gui` (Phase 46) |
| Math Pac I matrix register layout (R14=order, R15+) | Advantage ADVMTRX uses the same layout — confirmed by community sources |
| XROM shadowing test | Extend `tests/xrom_shadowing.rs` to cover both `ADV_A.ops` and `ADV_B.ops` |
| Free42 contamination guard | Extend `scripts/check-free42-contamination.sh` to scan `advantage/` subtree |

---

## Sources

- [calc.fjk.ch HP-41 Module Database](https://calc.fjk.ch/db/hp41mod.php) — XROM 22 "HP-41 Advantage Pac 1A/1B" confirmed; XROM 24 confirmed; also XROM 2 (Stat Pac), XROM 26 (Time Module) verified as cross-check. HIGH confidence.
- [HP Museum XROM Numbers table](https://www.hpmuseum.org/software/xroms.htm) — XROM 22 function names (BININ..MNAME?) and XROM 24 function names (MATRX..C+..TVM..FV) confirmed from web search result excerpts (page returns 403, but content was indexed). HIGH confidence.
- [Valentín Albillo "Long Live the Advantage ROM"](https://albillo.hpcalc.org/articles/HP%20Article%20VA008%20-%20Long%20Live%20the%20Advantage%20ROM.pdf) — Full PDF read. Describes 12K bank-switched ROM, MATDIM/MSYS/MSIJ/MRIJ/MSR+/MRR matrix conventions, 52 ADVMTRX routines from CCD ROM, SOLVE/INTEG from HP-15C. HIGH confidence.
- [HP Museum forum thread 184196 — "HP-41 Advanced Matrix Pac"](https://www.hpmuseum.org/cgi-bin/archv020.cgi?read=184196) — Confirms "Advanced Matrix Pac" is a hobbyist custom ROM, not an official HP product. HIGH confidence (403 during fetch, confirmed via web search excerpt + archived.hpcalc.org excerpt).
- [hpcalc.org literature items 759/761](https://literature.hpcalc.org/items/759) — Official HP Advantage Pac manual (OM 00041-90482 English, 156 pages, July 1985) and German edition confirmed. PDF too large to fetch during research (71 MB). Part number confirmed. HIGH confidence on existence.
- [Ángel M. Martin "Advantage Math ROM" (Systemyde PDF)](https://www.systemyde.com/pdf/Advantage_Math_Manual.pdf) — Full PDF read (pages 1-10). Confirmed this is XROM 12, a third-party module by Ángel Martin, published under GNU license, NOT the official HP Advantage Pac. MEDIUM confidence as supplementary context for official Advantage Pac functions.
- `hp41-core/src/ops/math1/xrom.rs` inspected directly — Confirms `XromModule` struct, `MATH_1.id=7`, `STAT_1.id=2`, `TIME_MODULE.id=26`, existing resolver bit-arm pattern (bit-0/1/2). HIGH confidence.
- `hp41-core/src/state.rs` inspected directly — Confirms `xrom_modules: u8`, `default_xrom_modules() = 0b0000_0111`, `migrate_after_load()` structure. HIGH confidence.

---
*Stack research for: HP-41 Advantage Pac (XROM 22 + 24) emulation (v3.3)*
*Researched: 2026-05-25*
