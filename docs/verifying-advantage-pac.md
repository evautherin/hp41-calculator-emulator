# Verifying Advantage Pac (HP 00041-90482)

This procedure walks an operator through the Advantage Pac emulation landed in
Phases 43-47 (v3.3). It covers all 7 functional groups (base conversion,
named matrices, complex extensions, polynomial/roots, solvers, curve fitting,
vectors + TVM) on both `hp41-cli` and `hp41-gui`, and confirms behavioral
identity across UIs.

The Advantage Pac uses two XROM module IDs: XROM 22 (ADV_MATH_A, bit 3 --
ADV CONV + ADV MTRX, 63 ops) and XROM 24 (ADV_MATH_B, bit 4 -- ADV MATH +
ADV TVM, 51 ops). Together they provide 114 XEQ entry points.

## TL;DR

| Group | Op count | Testability | Section |
|-------|----------|-------------|---------|
| Base Conversion & Boolean | 12 | All directly testable | [2](#2-base-conversion--boolean-logic) |
| Named Matrix Operations | 30+ | MATDIM setup required | [3](#3-named-matrix-operations) |
| Matrix Linear Algebra | 10 | MATDIM + element fill | [4](#4-matrix-linear-algebra) |
| Complex Matrix Operations | 5 | Complex MATDIM required | [5](#5-complex-matrix-operations) |
| Complex Extensions | 18 | All directly testable | [6](#6-complex-extensions) |
| Polynomial & Root-Finding | 3 | Register preload | [7](#7-polynomial--root-finding) |
| Solvers (FSOLVE/FINTG/FDIFEQ/FROOT) | 7 | Program-wrapper required | [8](#8-solvers-fsolvefintgfdifeqfroot) |
| Curve Fitting | 7 | Data accumulation via AS/DS | [9](#9-curve-fitting) |
| 3D Vectors | 14 | Register preload (R20-R28) | [10](#10-3d-vectors) |
| TVM | 6 | Modal workflow | [11](#11-time-value-of-money-tvm) |
| **Total** | **~114** | | |

A complete Tier 1 walk-through (sections 2, 6, 10, 11) takes approximately
25 minutes. Tier 2 (sections 3-5, 7-9) adds another 40 minutes.

## 1. Preparation

```bash
$ rm -f ~/.hp41/autosave.json
$ hp41             # or: just gui-dev
```

Operator: `Ctrl+G` (CLREG) -- fresh state.

Two CLI conveniences used throughout:

- `f` = `f-prefix one-shot` (orange shifted key), CLI key `f`. After the next
  op fires, `f-prefix` is cleared automatically. Esc cancels.
- `XEQ` by name = CLI key `X`, then type the command name, then `Enter`.

Default display mode is `FIX 4`. The "Display" column always means the
X-register content after the op completes unless noted otherwise.

Stack convention throughout: keystroke sequence like `2 ENTER 3` means
"X=3, Y=2" after the entry -- standard RPN.

### Named matrix setup

Most matrix operations require an active named matrix. Create one with:
1. Type matrix name into ALPHA register (ALPHA mode, then letters, then ALPHA)
2. Push rows (Y) and cols (X) onto stack
3. `XEQ MATDIM ENTER`

The matrix is now active and indices are at (1,1).

### MATH_1 alias overlap (ADR-v3.3-003)

12 Advantage Pac complex mnemonics (E^Z, LNZ, LOGZ, Z^N, Z^1/N, Z^W, |Z|,
SINZ, COSZ, TANZ, A^Z, CINV) are shared with Math Pac I. When both modules
are active (default), Math Pac I wins (bit-0 fires first). These ops are
tested in `verifying-math-pac-1.md` section 4. This document tests only the
Advantage Pac-exclusive ops.

## 2. Base Conversion & Boolean Logic

Twelve ops for integer base conversion and bitwise operations. All values
are silently truncated to 36 bits (ADV_WORD_MASK = 0x0000_000F_FFFF_FFFF)
per D-45-07.

### 2.1 Input Conversions

| # | Setup -> XEQ | Display (X) | Notes |
|---|-------------|-------------|-------|
| 2.1 | ALPHA `1010` ALPHA -> `XEQ BININ ENTER` | `10.0000` | Binary "1010" = decimal 10 |
| 2.2 | ALPHA `17` ALPHA -> `XEQ OCTIN ENTER` | `15.0000` | Octal "17" = decimal 15 |
| 2.3 | ALPHA `FF` ALPHA -> `XEQ HEXIN ENTER` | `255.0000` | Hex "FF" = decimal 255 |

### 2.2 Output Views

| # | Setup -> XEQ | ALPHA register after | Notes |
|---|-------------|---------------------|-------|
| 2.4 | `10 -> XEQ BINVIEW ENTER` | `1010` | Decimal 10 in binary |
| 2.5 | `255 -> XEQ HEXVIEW ENTER` | `FF` | Decimal 255 in hex |
| 2.6 | `42 -> XEQ CVTVIEW ENTER` | All-bases display | Shows BIN/OCT/DEC/HEX |

### 2.3 Bitwise Operations

All operate on 36-bit unsigned integers.

| # | Setup -> XEQ | Display (X) | Notes |
|---|-------------|-------------|-------|
| 2.7 | `255 -> XEQ NOT ENTER` | Large number | Bitwise complement (36-bit) |
| 2.8 | `12 ENTER 10 -> XEQ AND ENTER` | `8.0000` | 1100 AND 1010 = 1000 |
| 2.9 | `12 ENTER 10 -> XEQ OR ENTER` | `14.0000` | 1100 OR 1010 = 1110 |
| 2.10 | `12 ENTER 10 -> XEQ XOR ENTER` | `6.0000` | 1100 XOR 1010 = 0110 |
| 2.11 | `255 ENTER 4 -> XEQ ROTXY ENTER` | `4080.0000` | Rotate 255 left by 4 |
| 2.12 | `255 ENTER 0 -> XEQ BIT? ENTER` | `1` (flag set) | Bit 0 of 255 is set |

### 2.4 Error Cases

| # | Setup -> XEQ | Expected | Notes |
|---|-------------|----------|-------|
| 2.13 | Clear ALPHA -> `XEQ BININ ENTER` | `data error` | Empty ALPHA register |
| 2.14 | ALPHA `XYZ` ALPHA -> `XEQ BININ ENTER` | `data error` | Non-binary characters |

## 3. Named Matrix Operations

The Advantage Pac uses an ALPHA-register named-matrix model, completely
independent of the Math Pac I register-based matrix (D-43.5 isolation
invariant).

### 3.1 Matrix Lifecycle

| # | Setup -> XEQ | Verify | Notes |
|---|-------------|--------|-------|
| 3.1 | ALPHA `A` ALPHA, `3 ENTER 3 -> XEQ MATDIM ENTER` | Matrix "A" created, 3x3 | Creates and activates matrix |
| 3.2 | `XEQ DIM? ENTER` | X=3, Y=3 | Reports dimensions |
| 3.3 | `XEQ MNAME? ENTER` | ALPHA = "A" | Reports current matrix name |

### 3.2 Element Access (Index Navigation)

After creating a 3x3 matrix "A":

| # | Setup -> XEQ | Verify | Notes |
|---|-------------|--------|-------|
| 3.4 | `42 -> XEQ MS ENTER` | Element (1,1) = 42 | Store at current index |
| 3.5 | `XEQ MR ENTER` | X = 42 | Recall from current index |
| 3.6 | `XEQ J+ ENTER` | j advances to 2 | Column increment |
| 3.7 | `XEQ I+ ENTER` | i advances to 2 | Row increment |
| 3.8 | `XEQ J- ENTER` | j decrements | Column decrement |
| 3.9 | `XEQ I- ENTER` | i decrements | Row decrement |

### 3.3 Reductions

Fill a 2x2 matrix with [1, 2, 3, 4] (row-major), then:

| # | XEQ | Display (X) | Notes |
|---|-----|-------------|-------|
| 3.10 | `XEQ SUM ENTER` | `10.0000` | Sum of all elements |
| 3.11 | `XEQ MAX ENTER` | `4.0000` | Maximum element |
| 3.12 | `XEQ MIN ENTER` | `1.0000` | Minimum element |
| 3.13 | `XEQ FNRM ENTER` | `5.4772` | Frobenius norm = sqrt(1+4+9+16) |

## 4. Matrix Linear Algebra

### 4.1 Determinant and Inverse

Create a 2x2 matrix "D" with elements [[1,2],[3,4]]:

| # | XEQ | Display (X) | Notes |
|---|-----|-------------|-------|
| 4.1 | `XEQ MDET ENTER` | `-2.0000` | det([[1,2],[3,4]]) = -2 |
| 4.2 | `XEQ MINV ENTER` | Matrix inverted in-place | Verify MR at each position |

After MINV, matrix "D" contains [[-2, 1], [1.5, -0.5]]:

| # | Position | Expected | Notes |
|---|----------|----------|-------|
| 4.3 | (1,1) | -2.0000 | |
| 4.4 | (1,2) | 1.0000 | |
| 4.5 | (2,1) | 1.5000 | |
| 4.6 | (2,2) | -0.5000 | |

### 4.2 Matrix Arithmetic

Create two 2x2 matrices "A" = [[1,2],[3,4]] and "B" = [[5,6],[7,8]]:

| # | XEQ (with "B" in ALPHA) | Verify | Notes |
|---|--------------------------|--------|-------|
| 4.7 | `XEQ MAT+ ENTER` | A = [[6,8],[10,12]] | Element-wise addition |
| 4.8 | `XEQ M*M ENTER` | Result matrix | Matrix multiplication |
| 4.9 | `3 -> XEQ MAT*C ENTER` | All elements x3 | Scalar multiplication |

### 4.3 Error Cases

| # | Setup -> XEQ | Expected | Notes |
|---|-------------|----------|-------|
| 4.10 | MDET on 2x3 matrix | `data error` | Non-square |
| 4.11 | MINV on [[1,2],[2,4]] | `data error` | Singular matrix |
| 4.12 | M*M with incompatible dims | `data error` | cols(A) != rows(B) |

## 5. Complex Matrix Operations

Create a complex matrix with `XEQ MATDIM` (complex mode requires setting
`is_complex` programmatically or via CMEDIT modal).

| # | XEQ | Verify | Notes |
|---|-----|--------|-------|
| 5.1 | `XEQ CSUM ENTER` | Complex sum in X+iY | Sum of all complex elements |
| 5.2 | `XEQ CNRM ENTER` | Column norm | Complex Frobenius-like norm |
| 5.3 | `XEQ CMAXAB ENTER` | Max abs magnitude | Largest |z| element |

## 6. Complex Extensions

Eighteen complex ops. The Advantage Pac adds these beyond Math Pac I's
complex arithmetic. Input convention: X = real part, Y = imaginary part.
For binary ops: top complex = (X+iY), second complex = (Z+iT).

### 6.1 Unary Complex Functions

| # | Setup (X, Y) -> XEQ | Display (X, Y) | Notes |
|---|---------------------|----------------|-------|
| 6.1 | `0 ENTER 0 -> XEQ E^Z ENTER` | `(1, 0)` | e^0 = 1 |
| 6.2 | `1 ENTER 0 -> XEQ LNZ ENTER` | `(0, 0)` | ln(1) = 0 |
| 6.3 | `10 ENTER 0 -> XEQ LOGZ ENTER` | `(1, 0)` | log10(10) = 1 |
| 6.4 | `3 ENTER 4 -> XEQ |Z| ENTER` | `5` | |3+4i| = 5 |
| 6.5 | `2 ENTER 0 -> XEQ CINV ENTER` | `(0.5, 0)` | 1/2 = 0.5 |

### 6.2 Power Functions

| # | Setup -> XEQ | Display (X) | Notes |
|---|-------------|-------------|-------|
| 6.6 | `3 ENTER 0` (z), `2` in Z-reg -> `XEQ Z^N ENTER` | `9` | 3^2 = 9 |
| 6.7 | `27 ENTER 0` (z), `3` in Z-reg -> `XEQ Z^1/N ENTER` | `3` | 27^(1/3) |

### 6.3 Binary Complex Operations

| # | Setup (T, Z, Y, X) -> XEQ | Display (X, Y) | Notes |
|---|--------------------------|----------------|-------|
| 6.8 | `1 ENTER 2 ENTER 3 ENTER 4 -> XEQ CADD ENTER` | `(5, 5)` | (1+2i) + (3+4i) |
| 6.9 | `4 ENTER 3 ENTER 1 ENTER 2 -> XEQ CSUB ENTER` | `(3, 1)` | (4+3i) - (1+2i) |
| 6.10 | `XEQ CMUL ENTER` | Complex product | |
| 6.11 | `XEQ CDIV ENTER` | Complex quotient | |
| 6.12 | `XEQ AIP ENTER` | Appends char to ALPHA | ASCII code in X |

### 6.4 Trigonometric

| # | Setup (X, Y) -> XEQ | Display (X, Y) | Notes |
|---|---------------------|----------------|-------|
| 6.13 | `0 ENTER 0 -> XEQ SINZ ENTER` | `(0, 0)` | sin(0) = 0 |
| 6.14 | `0 ENTER 0 -> XEQ COSZ ENTER` | `(1, 0)` | cos(0) = 1 |
| 6.15 | `0 ENTER 0 -> XEQ TANZ ENTER` | `(0, 0)` | tan(0) = 0 |

## 7. Polynomial & Root-Finding

### 7.1 PLY -- Polynomial Evaluation

PLY evaluates p(x) using Horner's method. Coefficients in R01..R(n+1)
(descending power), degree in Y, x value in X.

| # | Setup | XEQ | Display (X) | Notes |
|---|-------|-----|-------------|-------|
| 7.1 | `1 STO 01, 2 STO 02, 3 STO 03`, `2 ENTER 1 -> XEQ PLY ENTER` | `6.0000` | x^2 + 2x + 3 at x=1 |

### 7.2 FROOT -- Polynomial Root-Finder (Laguerre's Method)

FROOT finds all roots of a polynomial (real and complex). Degree in X,
coefficients in R01..R(n+1). After dispatch, use RTS to retrieve roots
one at a time.

| # | Setup | XEQ | Expected | Notes |
|---|-------|-----|----------|-------|
| 7.2 | `1 STO 01, 0 STO 02, -4 STO 03`, `2 -> XEQ FROOT ENTER` | Roots +-2 found | x^2 - 4 = 0 |
| 7.3 | `XEQ RTS ENTER` (repeat) | X = root value | Retrieves next root |
| 7.4 | `1 STO 01, -6 STO 02, 11 STO 03, -6 STO 04`, `3 -> XEQ FROOT` | Roots 1, 2, 3 | x^3 - 6x^2 + 11x - 6 |
| 7.5 | `1 STO 01, 0 STO 02, 1 STO 03`, `2 -> XEQ FROOT` | Complex roots +-i | x^2 + 1 = 0 |

### 7.3 Error Cases

| # | Setup -> XEQ | Expected | Notes |
|---|-------------|----------|-------|
| 7.6 | `0 -> XEQ FROOT ENTER` | `data error` | Degree 0 rejected |
| 7.7 | `101 -> XEQ FROOT ENTER` | `data error` | Degree > 100 rejected |

## 8. Solvers (FSOLVE/FINTG/FDIFEQ/FROOT)

These are **run_loop-only ops**: direct XEQ from interactive mode returns
`data error`. They can only execute as a step inside a running program
(because they re-enter `run_loop` to evaluate the user function repeatedly).

### 8.1 FSOLVE -- Secant Method Root-Finder

Write a wrapper program:

```
Step 01: LBL "SLV"         ; user function: f(x) = x^2 - 4
Step 02: ENTER
Step 03: x                  ; X * X = X^2
Step 04: 4 -                ; X^2 - 4
Step 05: RTN

Step 06: LBL "MAIN"
Step 07: ALPHA SLV ALPHA    ; function label
Step 08: 1 STO 00           ; guess_1
Step 09: 3 STO 01           ; guess_2
Step 10: FSOLVE
Step 11: RTN
```

| # | Action | Display (X) | Notes |
|---|--------|-------------|-------|
| 8.1 | `XEQ MAIN ENTER` | `2.0000` | Root of x^2-4=0 near [1,3] |
| 8.2 | Print buffer shows | `ROOT IS 2.0000` | |

### 8.2 FINTG -- Romberg Integration

```
Step 01: LBL "F"
Step 02: X^2
Step 03: RTN

Step 04: LBL "INTMAIN"
Step 05: ALPHA F ALPHA
Step 06: 100 STO 00         ; subdivisions
Step 07: 0                   ; lower limit
Step 08: 1                   ; upper limit
Step 09: FINTG
Step 10: RTN
```

| # | Action | Display (X) | Notes |
|---|--------|-------------|-------|
| 8.3 | `XEQ INTMAIN ENTER` | `0.3333` | Integral of x^2 from 0 to 1 = 1/3 |

### 8.3 FDIFEQ -- RK4 ODE Solver

Similar to DIFEQ from Math Pac I. Solves y' = f(x,y) using 4th-order
Runge-Kutta.

### 8.4 Cross-Nesting (D-43.7)

FSOLVE inside FINTG and FINTG inside FSOLVE are allowed (cross-nesting).
Self-nesting (FSOLVE inside FSOLVE) is blocked.

| # | Setup | Expected | Notes |
|---|-------|----------|-------|
| 8.4 | FSOLVE with active FINTG state | Succeeds | D-43.7 cross-nesting |
| 8.5 | FSOLVE with active FSOLVE state | `data error` | Self-nesting blocked |

## 9. Curve Fitting

Seven ops for least-squares curve fitting: CFIT, AS, DS, BFIT, FIT, Y?X, SZ?.

### 9.1 Data Entry

Use AS (accumulate statistic) and DS (delete statistic) to build the dataset:

| # | Setup -> XEQ | Verify | Notes |
|---|-------------|--------|-------|
| 9.1 | `1 ENTER 2 -> XEQ AS ENTER` | Point (1,2) accumulated | |
| 9.2 | `2 ENTER 4 -> XEQ AS ENTER` | Point (2,4) accumulated | |
| 9.3 | `3 ENTER 6 -> XEQ AS ENTER` | Point (3,6) accumulated | |
| 9.4 | `XEQ SZ? ENTER` | `3` | 3 data points |

### 9.2 Fitting Models

| # | XEQ | Verify | Notes |
|---|-----|--------|-------|
| 9.5 | `XEQ CFIT ENTER` | Correlation coefficient | Linear fit to y=2x data |
| 9.6 | `4 -> XEQ Y?X ENTER` | `8.0000` | Predict y at x=4 |
| 9.7 | `XEQ FIT ENTER` | Best-fit model selection | Compares LIN/LOG/EXP/POW |

## 10. 3D Vectors

Fourteen ops for 3-component vector arithmetic. Vectors use registers:
- Vector A: R20, R21, R22
- Vector B: R23, R24, R25
- Result: R26, R27, R28

### 10.1 Setup

```
1 STO 20  ; A_x = 1
0 STO 21  ; A_y = 0
0 STO 22  ; A_z = 0
0 STO 23  ; B_x = 0
1 STO 24  ; B_y = 1
0 STO 25  ; B_z = 0
```

### 10.2 Operations

| # | XEQ | Result (R26, R27, R28) | Notes |
|---|-----|------------------------|-------|
| 10.1 | `XEQ V+ ENTER` | (1, 1, 0) | Vector addition |
| 10.2 | `XEQ V- ENTER` | (1, -1, 0) | Vector subtraction |
| 10.3 | `XEQ DOT ENTER` | X = 0 | Orthogonal vectors |
| 10.4 | `XEQ CROSS ENTER` | (0, 0, 1) | i x j = k |
| 10.5 | `XEQ |V| ENTER` | X = 1.0000 | Unit vector magnitude |
| 10.6 | `XEQ UV ENTER` | Unit vector in result | Normalize |

### 10.3 Vector Store/Recall

| # | XEQ | Verify | Notes |
|---|-----|--------|-------|
| 10.7 | `1 ENTER 2 ENTER 3 -> XEQ VS ENTER` | R20=1, R21=2, R22=3 | Store X,Y,Z to vec A |
| 10.8 | `XEQ VR ENTER` | X=1, Y=2, Z=3 | Recall vec A to stack |
| 10.9 | `XEQ VE ENTER` | Swap stack and regs | Exchange vec A <-> stack |

### 10.4 Additional Operations

| # | XEQ | Notes |
|---|-----|-------|
| 10.10 | `XEQ VC ENTER` | Create vector from stack |
| 10.11 | `XEQ VXY ENTER` | 2D rotation |
| 10.12 | `3 -> XEQ V* ENTER` | Scalar multiply |
| 10.13 | `2 -> XEQ VD ENTER` | Scalar divide |
| 10.14 | `XEQ TR ENTER` | Coordinate transform |

## 11. Time Value of Money (TVM)

Six ops for financial calculations: TVM, N, PV, PMT, FV, *I.

### 11.1 TVM Workflow

TVM opens a modal workflow. Enter values for N, PV, PMT, FV, then solve
for *I (interest rate):

| # | Step | XEQ | Input | Notes |
|---|------|-----|-------|-------|
| 11.1 | Initialize | `XEQ TVM ENTER` | | Opens TVM modal |
| 11.2 | Periods | `360 -> XEQ N ENTER` | 360 months | 30-year mortgage |
| 11.3 | Present Value | `200000 -> XEQ PV ENTER` | $200,000 loan | |
| 11.4 | Payment | `1199.10 CHS -> XEQ PMT ENTER` | -$1,199.10/month | Negative = outflow |
| 11.5 | Future Value | `0 -> XEQ FV ENTER` | $0 remaining | |
| 11.6 | Solve for rate | `XEQ *I ENTER` | ~0.5% monthly | Newton-Raphson solver |

### 11.2 TVM Persistence (D-43.11)

TVM state persists across save/load cycles (Pitfall 20 exception --
`#[serde(default)]` WITHOUT `#[serde(skip)]`). After saving and reloading:

| # | Verify | Notes |
|---|--------|-------|
| 11.7 | TVM values preserved after save/reload | `adv_tvm_state` roundtrips |

### 11.3 Error Cases

| # | Setup -> XEQ | Expected | Notes |
|---|-------------|----------|-------|
| 11.8 | `XEQ *I ENTER` without TVM init | `data error` | No TVM state |

## 12. CATALOG 2 Verification

CATALOG 2 should list all 5 XROM modules:

| # | XROM ID | Module Name | Op Count |
|---|---------|-------------|----------|
| 12.1 | 7 | MATH 1 | ~40 |
| 12.2 | 2 | STAT 1 | 26 |
| 12.3 | 26 | TIME 2C | 35 |
| 12.4 | 22 | ADV 22A | 63 |
| 12.5 | 24 | ADV 24B | 51 |

Invoke CATALOG 2 in CLI: press `C`, type `2`, Enter.

## 13. `?` Help Overlay Verification

Press `?` to open the help overlay. Verify:

| # | Section | Expected |
|---|---------|----------|
| 13.1 | "Advantage Pac (XROM 22)" | 63 entries (Adv Conv + Adv Mtrx) |
| 13.2 | "Advantage Pac (XROM 24)" | 51 entries (Adv Math + Adv TVM) |
| 13.3 | Search for "MDET" | Found in Advantage Pac section |
| 13.4 | All 6 sections expand/collapse | hp41cv + math1 + stat1 + time + adv22 + adv24 |

## 14. Error Path Summary

| Category | Trigger | Display | Examples |
|----------|---------|---------|----------|
| `data error` (Domain) | Invalid input (FROOT degree 0, non-square MDET) | `data error` | 2.13-14, 4.10-12, 7.6-7 |
| `data error` (DivideByZero) | Zero divisor (MAT/C by 0, UV on zero vector) | `data error` | 4.12 |
| `data error` (InvalidOp) | Run-loop op from interactive (FSOLVE/FINTG direct) | `data error` | 8 preamble |

Cross-UI guarantee: the **same** `HpError` variant surfaces as the **same**
`data error` text in both `hp41-cli` (status line) and `hp41-gui` (toast
overlay).

## 15. Same Procedure in the GUI

Mirror all sections exactly. GUI-specific input paths:

**XEQ-by-name entry**: click the `XEQ` button. Type letters via the
on-screen keyboard or physical keyboard. Press `ENTER` to commit.

**Help overlay**: click `?` button. The overlay shows 6 collapsible sections
including "Advantage Pac (XROM 22)" and "Advantage Pac (XROM 24)".

**Cross-UI behavioral guarantee:** all numerical results MUST be identical
between CLI and GUI. Both UIs dispatch through the same `hp41-core` ops.
Mismatches are an SC-4 violation and a release blocker.

## Known Limitations

- 12 complex mnemonics (E^Z, LNZ, etc.) are shared with Math Pac I via
  XROM alias overlap (ADR-v3.3-003). Math Pac I wins when both are active.
  The Advantage Pac versions are functionally identical.
- Interrupting control alarms (Time Pac D-38.4 deferral) do not affect
  Advantage Pac operations.
- Named matrices are completely independent of Math Pac I register-based
  matrices (D-43.5 isolation invariant). Operations on one model do not
  affect the other.
- FDIFEQ requires program-wrapper setup (same pattern as Math Pac I DIFEQ).
- Matrix size is capped at 255x255 (D-45-02 emulator extension; hardware
  was limited to ~8 rows).

## See Also

- [HP Advantage Pac Owner's Manual citations -- Divergences](hp41-advantage-divergences.md)
- [ADR-v3.3-001 -- Named matrix storage model](adr/v3.3-001-named-matrix-storage-model.md)
- [ADR-v3.3-002 -- FROOT Laguerre algorithm](adr/v3.3-002-froot-laguerre-algorithm.md)
- [ADR-v3.3-003 -- Dual XROM ID design](adr/v3.3-003-dual-xrom-id-design.md)
- [ADR-v3.3-004 -- math1 visibility promotion policy](adr/v3.3-004-math1-visibility-promotion-policy.md)
- [Advantage Pac function matrix](hp41-advantage-function-matrix.md)
- [Math Pac I verification -- companion document](verifying-math-pac-1.md)
- [Stat 1 Pac verification -- companion document](verifying-stat-pac-1.md)
