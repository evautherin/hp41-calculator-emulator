# Feature Research: HP-41 Advantage Pac + Advanced Matrix Pac

**Domain:** HP-41 calculator module emulation — Advantage Pac (OM 00041-90482, HP part 5061-7285/5061-7292) + Advanced Matrix Pac
**Researched:** 2026-05-25
**Confidence:** MEDIUM — function list reconstructed from multiple authoritative cross-references (HP Museum XROM table, module database, Valentín Albillo article, calc.fjk.ch function DB, Advantage Math ROM Manual). The original HP OM 00041-90482 PDF (71 MB) could not be retrieved due to size limits; the German-language OM 00041-90562 (51 MB) likewise. Function names and XROM numbers are HIGH confidence (corroborated by 4+ independent sources). Behavioral descriptions (algorithms, stack conventions) are MEDIUM confidence (confirmed from HP Museum forum posts, community articles, SandMath manual references).

---

## Background: What the Advantage Pac Is

The HP Advantage ROM (released July 1985) was HP's first 12K bank-switched ROM. It uses **two XROM IDs — XROM 22 and XROM 24** — to expose 117 functions in four named sections:

- **-ADV CONV** (XROM 22, fn 1–12): 12 base-conversion and boolean logic functions
- **-ADV MTRX** (XROM 22, fn 13–63): 51 matrix routines, all M-CODE (derived from the CCD ROM, extended by HP)
- **-ADV MATH** (XROM 24, fn 2–48+): 47 routines — Solve/Integrate (HP-15C ported to 41C M-CODE), complex arithmetic, curve fitting, polynomial roots, differential equations, vectors
- **-ADV TVM** (XROM 24, fn 50–56): 6 Time Value of Money routines

The Advantage Pac is **distinct from Math Pac I** (XROM 7). Math Pac I is pure user-code FOCAL programs; the Advantage Pac is primarily M-CODE (machine language) running at maximum speed. This distinction is critical for emulation strategy.

The **Advanced Matrix Pac** is a separate HP module that contains all -ADV MTRX functions plus additional matrix programs from the ALGEBRA module, plus a new Matrix Input mode. It shares the same XROM matrix function set as the Advantage Pac but reorganizes the ROM banks to maximize matrix capability. For emulation purposes, the Advanced Matrix Pac is a subset/variant of the Advantage Pac, not a new function set.

---

## Complete Function List by Section

### Section 1: ADVCONV — Base Conversion and Boolean Logic (XROM 22, fn 1–12)

These are all **one-shot stack operations** — no prompting, no modal workflow.

| XROM | Function | Description | Stack Input | Stack Output | Complexity |
|------|----------|-------------|-------------|--------------|------------|
| 22,1 | BININ | Binary string in ALPHA → integer in X | ALPHA = binary string | X = integer | LOW |
| 22,2 | BINVIEW | Integer in X → binary string in ALPHA, display | X = integer | ALPHA = binary | LOW |
| 22,3 | OCTIN | Octal string in ALPHA → integer in X | ALPHA = octal string | X = integer | LOW |
| 22,4 | CVTVIEW | Display current value in all bases | X = integer | display only | LOW |
| 22,5 | HEXIN | Hex string in ALPHA → integer in X | ALPHA = hex string | X = integer | LOW |
| 22,6 | HEXVIEW | Integer in X → hex string in ALPHA, display | X = integer | ALPHA = hex | LOW |
| 22,7 | NOT | Bitwise NOT of X | X = integer | X = ~X | LOW |
| 22,8 | AND | Bitwise AND of X and Y | Y, X = integers | X = Y AND X | LOW |
| 22,9 | OR | Bitwise OR of X and Y | Y, X = integers | X = Y OR X | LOW |
| 22,10 | XOR | Bitwise XOR of X and Y | Y, X = integers | X = Y XOR X | LOW |
| 22,11 | ROTXY | Rotate X by Y bits | Y = shift count, X = integer | X = rotated | LOW |
| 22,12 | BIT? | Test bit Y of integer X; skip if set | Y = bit position, X = integer | skip/no-skip | LOW |

**Implementation notes:** These are pure integer bitwise operations. The Advantage Pac treats integers as whole numbers in X; the word size is determined by the current value. No flags or modes affect behavior. These are entirely independent of the existing Math Pac I or HP-41CV built-ins.

**Overlap with existing code:** None. The HP-41CV has no bitwise operations. This is a clean new capability.

---

### Section 2: ADVMTRX — Matrix Operations (XROM 22, fn 13–63)

The matrix system is radically different from Math Pac I's MATRIX program. Math Pac I's MATRIX is a high-level FOCAL workflow; ADVMTRX provides 51 M-CODE primitive building blocks that programs call directly. Matrices live in **Extended Memory (X-MEM)** or in numbered registers. The system tracks current matrix, current row (I), and current column (J) indices via internal state.

**Architecture:** A matrix is identified by an ALPHA name. MATDIM creates/dimensions it. Indices are manipulated with I+/I-/J+/J-/MSIJ/MRIJ. Elements are read with MRR+/MRC+/MRIJ and written with MSR+/MSC+/MSIJ. High-level operations (MDET, MINV, MSYS) consume the whole matrix at M-CODE speed.

All ADVMTRX functions are **one-shot stack/register operations** — no modal prompting (unlike Math Pac I MATRIX which prompts ORDER=?, A1,1=? etc.). Users write their own program loops to drive these primitives.

| XROM | Function | Description | Type |
|------|----------|-------------|------|
| 22,13 | C<>C | Exchange two complex matrices | One-shot |
| 22,14 | CMAXAB | Max absolute value in complex matrix | One-shot |
| 22,15 | CNRM | Complex matrix norm | One-shot |
| 22,16 | CSUM | Sum of complex matrix elements | One-shot |
| 22,17 | DIM? | Return dimensions of named matrix in X/Y | One-shot |
| 22,18 | FNRM | Frobenius norm of matrix | One-shot |
| 22,19 | I+ | Increment row index | One-shot |
| 22,20 | I- | Decrement row index | One-shot |
| 22,21 | J+ | Increment column index | One-shot |
| 22,22 | J- | Decrement column index | One-shot |
| 22,23 | M*M | Matrix multiply: result = A * B | One-shot |
| 22,24 | MAT* | Scalar multiply: all elements times X | One-shot |
| 22,25 | MAT+ | Matrix add: A + B element-wise | One-shot |
| 22,26 | MAT- | Matrix subtract: A - B element-wise | One-shot |
| 22,27 | MAT/ | Scalar divide: all elements divided by X | One-shot |
| 22,28 | MATDIM | Create/dimension matrix in X-MEM or registers | One-shot |
| 22,29 | MAX | Maximum element value (real matrices) | One-shot |
| 22,30 | MAXAB | Maximum absolute value element | One-shot |
| 22,31 | MDET | Determinant of current matrix | One-shot |
| 22,32 | MIN | Minimum element value | One-shot |
| 22,33 | MINV | Inverse of current matrix (in-place LU) | One-shot |
| 22,34 | MMOVE | Copy rows/columns of matrix to another | One-shot |
| 22,35 | MNAME? | Return name of current matrix in ALPHA | One-shot |
| 22,36 | MR | Read element at current index | One-shot |
| 22,37 | MRC+ | Read element, increment column index | One-shot |
| 22,38 | MRC- | Read element, decrement column index | One-shot |
| 22,39 | MRIJ | Read element at explicit I,J in X | One-shot |
| 22,40 | MRIJR | Read element at I,J; advance to next row | One-shot |
| 22,41 | MRR+ | Read element, increment row index | One-shot |
| 22,42 | MRR- | Read element, decrement row index | One-shot |
| 22,43 | MS | Write X to element at current index | One-shot |
| 22,44 | MSC+ | Write X to element, increment column | One-shot |
| 22,45 | MSIJ | Write X to element at explicit I,J | One-shot |
| 22,46 | MSIJR | Write X to I,J; advance to next row | One-shot |
| 22,47 | MSR+ | Write X to element, increment row | One-shot |
| 22,48 | MSWAP | Swap two rows (or columns) | One-shot |
| 22,49 | MSYS | Solve linear system Ax=b (simultaneous equations) | One-shot |
| 22,50 | PIV | Pivot operation (partial pivoting step) | One-shot |
| 22,51 | R<>R | Exchange two rows | One-shot |
| 22,52 | R>R? | Compare two rows; skip if row k > row l | One-shot |
| 22,53 | RMAXAB | Row maximum absolute value | One-shot |
| 22,54 | RNRM | Row norm | One-shot |
| 22,55 | RSUM | Row sum | One-shot |
| 22,56 | SUM | Sum of all matrix elements | One-shot |
| 22,57 | SUMAB | Sum of absolute values of all elements | One-shot |
| 22,58 | TRNPS | Transpose matrix (in-place) | One-shot |
| 22,59 | YC+C | Add complex number Y+Xi to complex matrix element | One-shot |
| 22,60 | MEDIT | Open real matrix editor (interactive input mode) | Modal (editor session) |
| 22,61 | CMEDIT | Open complex matrix editor | Modal (editor session) |
| 22,62 | MP | Matrix print (output all elements via print buffer) | One-shot |

**Critical overlap with Math Pac I:** MDET and MINV duplicate the end-result of Math Pac I's MATRIX/DET and MATRIX/INV workflows, but use completely different underlying infrastructure. Math Pac I MATRIX stores the matrix in R15..R(15+n^2-1) numbered registers with a specific ordering; ADVMTRX uses X-MEM named matrices. These are **incompatible storage formats**. MSYS is the equivalent of MATRIX/SIMEQ.

**MEDIT/CMEDIT:** These are the two modal functions in ADVMTRX. They open an interactive matrix editor where the user can navigate elements with shifted keys and enter values. This is analogous to a spreadsheet-style data entry mode. Implementation: a new ModalProgram variant (`ModalProgram::Advantage(AdvantageStep)`) would be needed, or more likely, emulated as a simpler element-by-element PROMPT loop (since the full M-CODE editor UI is not required for behavioral fidelity — what matters is data entry and storage).

---

### Section 3: ADVMATH — Advanced Mathematics (XROM 24)

This section has the highest emulation complexity. It contains 47 routines split into several subsections. XROM 24 function numbers confirmed from the calc.fjk.ch database and cross-referenced with HP Museum forum discussions.

#### 3a. Matrix Interface (XROM 24, fn 0–1)

High-level entry points that wrap the ADVMTRX primitives with user-friendly prompting. These ARE multi-step modal workflows.

| XROM | Function | Description | Type |
|------|----------|-------------|------|
| 24,0 | MATRX | Full matrix workflow: DIM, input, choose DET/INV/SIMEQ | Modal workflow |
| 24,1 | MTR | Simplified matrix entry/solve frontend | Modal workflow |

**Overlap with Math Pac I:** MATRX is the Advantage Pac's version of Math Pac I's MATRIX program. Key differences: MATRX uses X-MEM named matrices (not R15-based), prompts for matrix NAME instead of just ORDER, supports larger matrices, runs at M-CODE speed (much faster). Behavioral emulation must replicate the Advantage-specific prompt sequence (NAME=?, DIM=?, element entry) rather than the Math Pac I sequence (ORDER=?, A1,1=?).

#### 3b. Solve and Integrate (XROM 24, fn 2–6) — HP-15C algorithms ported to M-CODE

These are the flagship Advantage Pac capabilities. FSOLVE and FINTG are HP-15C-compatible in algorithm but with 41C-specific user interface.

| XROM | Function | Description | Type |
|------|----------|-------------|------|
| 24,2 | FSOLVE | Root of f(x)=0 via Secant/Brent; user-program callback | Modal workflow |
| 24,3 | FINTG | Romberg integration of f(x); user-program callback | Modal workflow |
| 24,4 | SILOOP | Internal integration loop (not user-callable directly) | Internal |
| 24,5 | SIRTN | Integration return (internal) | Internal |
| 24,6 | FDIFEQ | Differential equations solver (1st/2nd order RK4) | Modal workflow |

**Critical distinction from Math Pac I:**
- Math Pac I SOLVE uses a secant-based algorithm with prompts FUNCTION NAME?, GUESS 1=?, GUESS 2=?
- Math Pac I INTG uses Simpson's rule with prompts for lower/upper bounds and subinterval count
- Advantage FSOLVE uses an improved HP-15C-style algorithm (faster convergence, no explicit guess often required)
- Advantage FINTG uses **Romberg's method** (Richardson extrapolation on the Euler-MacLaurin sum) — adaptive accuracy, unlike Simpson's fixed-subinterval approach
- Both FSOLVE and FINTG support **mutual nesting**: FROOT can be called from an FINTG integrand and FINTG can be called from an FSOLVE function body — this requires re-entrant buffer management that Math Pac I's infrastructure does NOT support
- Advantage FSOLVE/FINTG create dedicated **X-MEM buffers** to store application state — this is fundamentally different from Math Pac I's register-based state
- Advantage FDIFEQ replaces Math Pac I's DIFEQ (same RK4 algorithm family but M-CODE speed and X-MEM buffers)

**Stack conventions:** FSOLVE: ALPHA = function name, X = initial guess (or provide two guesses). FINTG: ALPHA = function name, Y = lower limit, X = upper limit; result in X. These match HP-15C conventions loosely but use the 41C ALPHA register for the function name.

#### 3c. Polynomial Roots (XROM 24, fn ~20)

| XROM | Function | Description | Type |
|------|----------|-------------|------|
| 24,~20 | FROOT | Roots of polynomial of **arbitrary degree** using Laguerre's method; user-program as polynomial evaluator | Modal workflow |
| 24,~21 | PLY | Polynomial evaluation: compute P(x) | One-shot |
| 24,~22 | RTS | Root output/collection utility | One-shot |

**Critical overlap and extension of Math Pac I POLY/ROOTS:**
- Math Pac I POLY/ROOTS handles degree 2–5 only, uses a fixed closed-form algorithm
- Advantage FROOT handles **arbitrary degree** using Laguerre's iterative method
- FROOT requires the user to write a program that evaluates the polynomial (the function callback pattern, same infrastructure as FSOLVE/FINTG)
- This is the "PROOT" mentioned in the PROJECT.md milestone context — "PROOT" appears to be an alternative community name; the HP OM uses FROOT
- FROOT outputs real and complex root pairs sequentially via R/S

#### 3d. Complex Number Operations (XROM 24, fn 7–19+)

These extend Math Pac I's complex stack. Math Pac I already implements C+, C-, C×, C÷ and 13 complex functions using the X/Y/Z/T stack overlay. The Advantage Pac provides a **partially overlapping but architecturally different** complex number system.

**Key architectural difference:** Math Pac I uses the 4-register stack as two complex numbers (zeta = Z+Yi, tau = T+Xi nomenclature from the OM). Advantage Pac complex functions use the **same X/Y convention** (Y = imaginary, X = real) but include additional operations not in Math Pac I.

| XROM | Function | Description | Math Pac I equivalent? |
|------|----------|-------------|------------------------|
| 24,7 | Z^N | Complex z raised to integer power n | No (Math Pac I has CY^X for general power) |
| 24,8 | MAGZ | Magnitude (absolute value) of complex z | Equivalent to CABS in Math Pac I |
| 24,9 | e^Z | Complex exponential e^z | No direct equivalent in Math Pac I |
| 24,10 | LNZ | Complex natural log ln(z) | No direct equivalent |
| 24,11 | Z^1/N | Complex nth root | No direct equivalent |
| 24,12 | SINZ | Complex sine sin(z) | No direct equivalent |
| 24,13 | COSZ | Complex cosine cos(z) | No direct equivalent |
| 24,14 | TANZ | Complex tangent tan(z) | No direct equivalent |
| 24,15 | a^Z | Real base a raised to complex power z | Partial overlap with CY^X |
| 24,16 | LOGZ | Complex log base a: log_a(z) | No direct equivalent |
| 24,17 | Z^1/W | Complex z raised to 1/w | No direct equivalent |
| 24,18 | Z^W | Complex z raised to complex w | Equivalent to Math Pac I CY^X |
| 24,19 | C+ | Complex addition z1 + z2 | Duplicate of Math Pac I C+ |
| 24,~20a | C- | Complex subtraction | Duplicate of Math Pac I C- |
| 24,~21a | CINV | Complex inverse 1/z | Equivalent to Math Pac I CINV |
| 24,~22a | C* | Complex multiply | Duplicate of Math Pac I C× |
| 24,~23a | C/ | Complex divide | Duplicate of Math Pac I C÷ |

**Stack convention:** Y = imaginary part, X = real part for a single complex number. Two complex numbers: Z = Im(z2), Y = Re(z2), X and T = Im(z1)/Re(z1). The convention matches the HP-15C two-complex-number stack layout, not exactly the Math Pac I zeta/tau layout. This is a behavioral divergence to document carefully.

**Functions in PROJECT.md mentioned mapping to Advantage Pac:**
- CABS → MAGZ (24,8) — same function, different name
- CARG → not separately listed; argument comes from MAGZ+LNZ or atan2 via TANZ
- CCHS → complex negation (negate both real and imaginary parts) — trivial one-shot
- CCONJ → complex conjugate (negate imaginary part) — trivial one-shot
- CY^X → Z^W (24,18)

Note: CABS, CARG, CCHS, CCONJ as named functions appear in the **Math Pac I** complex set (already implemented in v3.0 as CMPLX-01..17). The Advantage Pac uses different names (MAGZ instead of CABS, etc.) for equivalent or extended operations.

#### 3e. Curve Fitting (XROM 24)

| XROM | Function | Description | Type |
|------|----------|-------------|------|
| 24,~30 | CFIT | Curve fit (linear, log, exp, power) to (x,y) data | Modal workflow |
| 24,~31 | AS | Add data point to curve fit accumulation | One-shot |
| 24,~32 | DS | Delete/subtract data point from accumulation | One-shot |
| 24,~33 | BFIT | Best-fit selection: determine which model fits best | Modal workflow |
| 24,~34 | FIT | Compute fit coefficients for current model | One-shot |
| 24,~35 | Y?X | Predict Y from X using current fit | One-shot |
| 24,~36 | SZ? | Test if current data set has sufficient points | One-shot |

**Overlap with HP-41CV built-ins:** The HP-41CV has built-in L.R. (linear regression) and MEAN/SDEV. CFIT extends this to logarithmic, exponential, and power curve fitting. AS/DS mirror Sigma+/Sigma- but for curve fit storage registers.

#### 3f. Vector Operations (XROM 24)

| XROM | Function | Description | Type |
|------|----------|-------------|------|
| 24,~37 | VC | Vector cross product (3D) | One-shot |
| 24,~38 | CROSS | Cross product (alternate entry) | One-shot |
| 24,~39 | VS | Vector scalar multiply | One-shot |
| 24,~40 | VR | Vector result recall | One-shot |
| 24,~41 | DOT | Dot product of two 3D vectors | One-shot |
| 24,~42 | VE | Vector entry (input 3D vector to registers) | Modal |
| 24,~43 | V- | Vector subtract | One-shot |
| 24,~44 | V+ | Vector add | One-shot |
| 24,~45 | VXY | Vector to X-Y plane projection | One-shot |
| 24,~46 | UV | Unit vector | One-shot |
| 24,~47 | V< | Vector magnitude (length) | One-shot |
| 24,~48 | VD | Vector dot product (variant) | One-shot |
| 24,~49 | V* | Vector scalar product (variant) | One-shot |

**Note on XROM numbers:** The exact XROM 24 sub-numbers for curve fit and vector operations are MEDIUM confidence — reconstructed from the calc.fjk.ch function database and community descriptions. The function names themselves are HIGH confidence (corroborated by multiple sources). Exact fn numbers should be verified against the OM during implementation.

#### 3g. Coordinate Transformations (XROM 24)

| XROM | Function | Description | Type |
|------|----------|-------------|------|
| 24,~50 | TR | Coordinate transformation (2D/3D) | One-shot |
| 24,~51 | CT | Coordinate transform result | One-shot |

**Overlap with Math Pac I:** Math Pac I has TRANS (2D/3D coordinate transformations). The Advantage Pac TR/CT functions provide the same capability but via one-shot M-CODE primitives rather than a multi-step modal workflow.

#### 3h. Miscellaneous Math (XROM 24)

| XROM | Function | Description | Type |
|------|----------|-------------|------|
| 24,~52 | AIP | Alpha integer append to ALPHA string | One-shot |

---

### Section 4: ADVTVM — Time Value of Money (XROM 24, fn ~53–58)

These are primarily **one-shot stack operations**. The TVM model is stored in dedicated registers.

| XROM | Function | Description | Stack |
|------|----------|-------------|-------|
| 24,~53 | TVM | Initialize / master TVM solver | Modal (prompts N, I, PV, PMT, FV) |
| 24,~54 | N | Solve for N (periods) given other TVM vars | One-shot |
| 24,~55 | PV | Solve for PV (present value) | One-shot |
| 24,~56 | PMT | Solve for PMT (payment) | One-shot |
| 24,~57 | FV | Solve for FV (future value) | One-shot |
| 24,~58 | *I | Solve for periodic interest rate | One-shot (iterative internally) |

**Note:** *I (solve for interest) is iterative because there is no closed-form solution for interest rate. This is the only TVM function requiring internal iteration. The others are closed-form rearrangements of the TVM equation.

---

## Feature Landscape

### Table Stakes (Users Expect These for a Complete Advantage Pac Emulation)

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| ADVCONV: 12 base/boolean ops (BININ to BIT?) | OM documents them; HP-16C users expect these | LOW | Pure integer arithmetic, no modal |
| ADVMTRX: core element access (MR/MS, I+/I-/J+/J-, MSIJ/MRIJ, MSR+/MSC+, MRR+/MRC+) | Foundation for all matrix programs | MEDIUM | Index state management per CalcState |
| ADVMTRX: MATDIM, DIM?, MNAME? | Matrix lifecycle management | MEDIUM | Named matrices in X-MEM model |
| ADVMTRX: MDET, MINV, MSYS | The three flagship matrix ops users actually call | HIGH | LU decomposition, back-substitution |
| ADVMTRX: MAT+/MAT-/MAT*/MAT//M*M | Element-wise and matrix arithmetic | MEDIUM | Requires named matrix storage |
| ADVMTRX: TRNPS | Transpose | MEDIUM | In-place or copy |
| ADVMTRX: MMOVE, MSWAP, R<>R | Row/column manipulation | MEDIUM | Index arithmetic |
| ADVMTRX: SUM, MAX, MIN, MAXAB, RMAXAB | Reduction ops; used in sorting programs | LOW | Simple iteration |
| ADVMTRX: FNRM, RNRM | Norms for convergence testing | LOW | Simple iteration |
| FSOLVE (XROM 24) | Solve f(x)=0; flagship capability | HIGH | Brent/Secant, X-MEM buffer, callback |
| FINTG (XROM 24) | Romberg integration; flagship capability | HIGH | Richardson extrapolation, X-MEM buffer |
| FROOT / PLY / RTS | Arbitrary-degree polynomial roots | HIGH | Laguerre's method, complex root output |
| Complex functions (MAGZ, e^Z, LNZ, Z^N, SINZ, COSZ, TANZ, Z^W, C+/C-/C*/C/) | Math users expect complex arithmetic | MEDIUM | Most are one-shot; extend Math Pac I |
| CFIT / AS / DS / Y?X | Curve fitting beyond L.R. | MEDIUM | Builds on Sigma-register pattern |
| ADVTVM: TVM, N, PV, PMT, FV, *I | HP-12C-style TVM; widely expected | MEDIUM | *I requires Newton iteration |

### Differentiators

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| FROOT mutual nesting with FINTG | Solve for root of an integral; impossible in Math Pac I | HIGH | Re-entrant X-MEM buffer architecture |
| ADVMTRX in X-MEM (no numbered registers consumed) | Programs at SIZE 000 work with full matrices | HIGH | X-MEM named matrix storage model |
| MEDIT/CMEDIT matrix editor | Spreadsheet-style interactive matrix entry | HIGH | Modal editor session, new UX pattern |
| Romberg vs Simpson integration | Significantly more accurate for smooth functions | MEDIUM | Algorithm difference, same callback pattern |
| Laguerre roots for arbitrary degree | Generalize POLY/ROOTS to any degree polynomial | HIGH | Adds complex root output |
| M*M (matrix multiply) | HP-15C users expect this | MEDIUM | O(n^3) loop over named matrices |
| Vector ops (DOT, CROSS, V+, V-, UV) | Useful for physics/engineering | MEDIUM | Simple stack arithmetic mostly |
| BFIT automatic model selection | "Best fit" auto-selection | LOW | Tests all 4 fit models, picks lowest residual |

### Anti-Features

| Anti-Feature | Why Requested | Why Problematic | Alternative |
|--------------|---------------|-----------------|-------------|
| Full M-CODE speed emulation | "Faithfulness" | Behavioral emulation is the project scope; cycle-accurate M-CODE is excluded per project invariants | Correct algorithms at any speed |
| Duplicating Math Pac I infrastructure for Advantage | Unified codebase | Math Pac I uses register-based matrices; Advantage uses X-MEM named matrices — they are architecturally incompatible. Merging creates silent bugs | Separate `adv/` directory under `ops/` |
| MEDIT full interactive editor | Completeness | The M-CODE editor intercepts every keystroke; full replication requires a new major TUI mode. The OM behavioral spec can be satisfied with a simpler PROMPT-driven element entry loop | Use existing modal infrastructure: ModalProgram::Advantage(AdvantageStep) with element-by-element prompts |
| X-MEM full Extended Memory model | Completeness | Full HP-41CX X-MEM with EMDIR, EMROOM, EMREG, etc. is a separate major feature. The Advantage Pac only needs named matrix storage | Implement a minimal named-matrix store (Vec of (name, rows, cols, data) tuples) on CalcState |
| MSYS unlimited size | Faithfulness | Math Pac I MATRIX tops at 14x14. Advantage MSYS is limited by available X-MEM. For emulation, cap at the same 14x14 limit for consistency | Document as behavioral policy in divergences file |

---

## Feature Dependencies

```
ADVCONV (12 ops)
    └── independent (no dependencies)

ADVMTRX element primitives (I+/I-/J+/J-/MR/MS/MRIJ/MSIJ/etc.)
    └──requires──> Named matrix storage on CalcState (X-MEM model stub)
                      └──requires──> adv_matrices: Vec<(String, usize, usize, Vec<HpNum>)>

ADVMTRX high-level (MDET/MINV/MSYS/M*M/TRNPS/MAT+/etc.)
    └──requires──> ADVMTRX element primitives (index system)

MATRX / MTR modal workflows
    └──requires──> ADVMTRX high-level ops
    └──requires──> Modal infrastructure: ModalProgram::Advantage(AdvantageStep)

FSOLVE / FINTG / FROOT / FDIFEQ
    └──requires──> User-program callback infrastructure (already in v3.0)
    └──requires──> Per-invocation buffer (can be CalcState transient field)
    └──shared──> Mutual nesting requires re-entrant buffer stack

Complex ops (MAGZ / e^Z / LNZ / SINZ / COSZ / TANZ / Z^W / etc.)
    └──extends──> Math Pac I complex stack (already in v3.0, CMPLX-01..17)
    └──note──> Stack convention may differ (verify Y=Im vs Y=Re in Advantage vs Math Pac I)

Curve fitting (CFIT / AS / DS / FIT / BFIT / Y?X)
    └──requires──> Dedicated accumulation registers (separate from HP-41CV Sigma-registers)

Vector ops (V+ / V- / DOT / CROSS / VS / UV / V< etc.)
    └──independent──> Simple stack arithmetic, no new infrastructure

ADVTVM (TVM / N / PV / PMT / FV / *I)
    └──requires──> Dedicated TVM state registers on CalcState
    └──*I requires──> Newton iteration (same callback infrastructure OR internal iteration)
```

---

## MVP Definition for v3.3

### Launch With (Phase A — Core)

Minimum viable for users who bought the Advantage Pac for its matrix capabilities.

- [ ] XROM framework extension: XROM 22 + XROM 24 registered (two-slot, like Advantage hardware)
- [ ] Named matrix storage: minimal `adv_matrices` field on CalcState
- [ ] All 12 ADVCONV ops (BININ/BINVIEW/OCTIN/CVTVIEW/HEXIN/HEXVIEW/NOT/AND/OR/XOR/ROTXY/BIT?)
- [ ] ADVMTRX index ops: I+/I-/J+/J-/MSIJ/MRIJ/MSWAP
- [ ] ADVMTRX element access: MR/MS/MRC+/MRC-/MRR+/MRR-/MSC+/MSR+
- [ ] ADVMTRX lifecycle: MATDIM/DIM?/MNAME?/MMOVE
- [ ] ADVMTRX high-level: MDET/MINV/MSYS/M*M/MAT+/MAT-/MAT*/MAT//TRNPS
- [ ] ADVMTRX reductions: SUM/SUMAB/MAX/MIN/MAXAB/RMAXAB/FNRM/RNRM/CSUM/CNRM/CMAXAB

### Add After Validation (Phase B — ADVMATH)

Brings the flagship algorithmic functions.

- [ ] FSOLVE (Brent/Secant root finding, per-invocation buffer, user callback)
- [ ] FINTG (Romberg integration, per-invocation buffer, user callback)
- [ ] FROOT/PLY/RTS (Laguerre polynomial roots, arbitrary degree)
- [ ] FDIFEQ (RK4 differential equations, extended from Math Pac I DIFEQ)
- [ ] New complex functions not in Math Pac I: e^Z, LNZ, Z^N, Z^1/N, SINZ, COSZ, TANZ, a^Z, LOGZ, Z^1/W, Z^W

### Future Consideration (Phase C — Completion)

- [ ] Curve fitting: CFIT/AS/DS/FIT/BFIT/Y?X/SZ?
- [ ] Vector ops: V+/V-/DOT/CROSS/VC/VS/VR/VE/VXY/UV/V</VD/V*
- [ ] ADVTVM: TVM/N/PV/PMT/FV/*I
- [ ] MATRX/MTR modal workflows (high-level matrix entry)
- [ ] MEDIT/CMEDIT (simplified prompt-driven, not full M-CODE editor)
- [ ] Mutual nesting (FROOT inside FINTG integrand) — requires re-entrant buffer stack

---

## Overlap Matrix: Math Pac I vs Advantage Pac

This is critical for implementation — where do existing v3.0 ops extend vs conflict vs duplicate?

| Capability | Math Pac I (v3.0) | Advantage Pac | Strategy |
|------------|-------------------|---------------|----------|
| Matrix DET | MATRIX workflow, R15-based, ORDER prompts | MDET, X-MEM based, one-shot | New Op variants; different storage model |
| Matrix INV | MATRIX workflow, R15-based | MINV, X-MEM based, one-shot | New Op variants |
| Linear equations | MATRIX/SIMEQ, R15-based | MSYS, X-MEM based, one-shot | New Op variants |
| Matrix entry | A1,1=? prompts in MATRIX workflow | MEDIT or user program loop with MSR+ | New modal or simplified prompts |
| Polynomial roots | POLY/ROOTS, deg 2-5, exact algorithm | FROOT, arbitrary degree, Laguerre | New Op; FROOT does NOT replace POLY |
| Integration | INTG, Simpson's rule, fixed subintervals | FINTG, Romberg method, adaptive | New Op; INTG remains available |
| Solve | SOLVE, Secant method, GUESS 1/2 prompts | FSOLVE, Brent method, single guess | New Op; SOLVE remains available |
| Differential eq | DIFEQ, RK4, R-based | FDIFEQ, RK4, buffer-based | New Op; DIFEQ remains available |
| Complex arith | C+/C-/C×/C÷/CABS/CARG/CCHS/CCONJ/CY^X/etc. | C+/C-/C*/C//MAGZ/Z^W/e^Z/LNZ/etc. | Many new Ops; some duplicate Math Pac I with different names |
| Coord transforms | TRANS (2D/3D modal workflow) | TR/CT (one-shot) | New Ops; TRANS remains |
| Hyperbolics | SINH/COSH/TANH/ASINH/ACOSH/ATANH | Not in Advantage Pac | No change needed |

**Key invariant:** POLY/ROOTS (degree 2-5), SOLVE (secant), INTG (Simpson), DIFEQ (RK4), MATRIX (R15-based), TRANS (modal) from Math Pac I all remain available. The Advantage functions are **additions**, not replacements.

---

## XROM ID Assignment

Based on hardware documentation (HP Museum XROM table, module database):
- **XROM 22**: Advantage Pac first bank (-ADV CONV + -ADV MTRX)
- **XROM 24**: Advantage Pac second bank (-ADV MATH + -ADV TVM)

The `default_xrom_modules` bit mask will need bits for XROM 22 and 24 added. The `xrom_resolve` chain already handles multiple modules; v3.3 adds two new arms. Both XROM IDs must be confirmed against the existing XROM registry to ensure no conflict with MATH_1 (7), STAT_1 (2), TIME_MODULE (26).

**Advanced Matrix Pac:** Uses the same XROM 22 matrix functions. It is a ROM rearrangement for users who want to maximize matrix capability with less SOLVE/INTEG. For emulation purposes: implementing the standard Advantage Pac XROM 22+24 automatically covers the Advanced Matrix Pac's function set.

---

## Complexity Ratings by Phase

| Phase | Functions | Estimated Op Count | Complexity Driver |
|-------|-----------|--------------------|-------------------|
| ADVCONV | 12 | 12 | LOW — pure integer arithmetic |
| ADVMTRX primitives | ~30 | ~30 | MEDIUM — index state + named matrix storage |
| ADVMTRX high-level (MDET/MINV/MSYS) | 3 | 3 | HIGH — LU decomposition, back-substitution |
| ADVMTRX bulk ops (MAT+/M*M/TRNPS etc.) | ~18 | ~18 | MEDIUM — matrix iteration |
| FSOLVE + FINTG | 2 (+2 internal) | 2 | HIGH — Romberg algorithm, buffer model |
| FROOT + PLY + RTS | 3 | 3 | HIGH — Laguerre's method, complex root output |
| FDIFEQ | 1 | 1 | MEDIUM — reuses RK4 from Math Pac I DIFEQ |
| Complex extensions | ~13 | ~13 | MEDIUM — standard complex math formulas |
| Curve fitting | 7 | 7 | MEDIUM — builds on Sigma-register pattern |
| Vector ops | ~13 | ~13 | LOW — mostly stack arithmetic |
| ADVTVM | 6 | 6 | MEDIUM (*I iteration), LOW (N/PV/PMT/FV) |
| MATRX/MTR modal | 2 | 2 | HIGH — new modal infrastructure |
| **Total** | **~112** | **~112** | |

---

## Feature Prioritization Matrix

| Feature Group | User Value | Implementation Cost | Priority |
|---------------|------------|---------------------|----------|
| ADVMTRX high-level (MDET/MINV/MSYS) | HIGH | HIGH | P1 |
| ADVCONV (12 ops) | MEDIUM | LOW | P1 |
| ADVMTRX primitives (index+element) | HIGH | MEDIUM | P1 |
| FSOLVE + FINTG (Romberg) | HIGH | HIGH | P1 |
| FROOT arbitrary-degree roots | HIGH | HIGH | P1 |
| Complex extensions (e^Z, LNZ, SINZ etc.) | MEDIUM | MEDIUM | P1 |
| ADVTVM (TVM/N/PV/PMT/FV/*I) | MEDIUM | MEDIUM | P2 |
| ADVMTRX bulk ops (MAT+/MAT*/TRNPS etc.) | MEDIUM | MEDIUM | P2 |
| Curve fitting (CFIT/BFIT/AS/DS) | MEDIUM | MEDIUM | P2 |
| Vector ops (DOT/CROSS/V+/V- etc.) | MEDIUM | LOW | P2 |
| MATRX/MTR modal workflows | MEDIUM | HIGH | P2 |
| MEDIT/CMEDIT matrix editor | LOW | HIGH | P3 |
| FROOT/FINTG mutual nesting | LOW | HIGH | P3 |
| FDIFEQ (extends Math Pac I DIFEQ) | LOW | MEDIUM | P3 |

---

## Sources

- HP-41 module database (XROM 22+24 assignment confirmed): [calc.fjk.ch/db/hp41mod.php](https://calc.fjk.ch/db/hp41mod.php) — HIGH confidence
- HP-41 function database (complete XROM 22+24 function names): [calc.fjk.ch/db/hp41fn.php](https://calc.fjk.ch/db/hp41fn.php) — HIGH confidence
- HP Museum XROM numbers reference (partial tables): [hpmuseum.org/software/xroms.htm](https://www.hpmuseum.org/software/xroms.htm) — MEDIUM confidence (403 on direct fetch; cross-referenced via web search)
- Valentín Albillo, "Long Live the Advantage ROM!" (commemorative article): [albillo.hpcalc.org](https://albillo.hpcalc.org/articles/HP%20Article%20VA008%20-%20Long%20Live%20the%20Advantage%20ROM.pdf) — HIGH confidence for architectural description and matrix comparison
- Angel M. Martin, "Advantage Math ROM Manual" (XROM 12 community extension, Feb 2020): [systemyde.com](https://www.systemyde.com/pdf/Advantage_Math_Manual.pdf) — HIGH confidence for Advantage Pac usage patterns and ADVMTRX function descriptions
- HP Museum forum — Advanced Matrix Pac thread: [archv020 thread 184196](https://www.hpmuseum.org/cgi-bin/archv020.cgi?read=184196) — MEDIUM confidence (Advanced Matrix Pac = ADVMTRX + ALGEBRA module programs)
- HP Museum forum — Advantage Module discussion: [archv015 thread 89055](https://www.hpmuseum.org/cgi-bin/archv015.cgi?read=89055) — MEDIUM confidence (FROOT/FINTG mutual nesting, X-MEM buffer architecture)
- HP-41 Module Database (362 entries, 2011): [lastin.dti.supsi.ch modules PDF](https://lastin.dti.supsi.ch/VET/sys/HPXX/HP41CV/HP-41C-CV-CX_Modules.pdf) — HIGH confidence for Advantage Pac versions 1A Proto / 1A / 1B all using XROM 22+24
- HP Calculator Literature Archive — OM 00041-90482: [literature.hpcalc.org/items/759](https://literature.hpcalc.org/items/759) — confirms part number, July 1985, 156 pages (content not readable due to 71 MB size limit)
- HP-41 Math Pac I Quick Reference Card (confirms v3.0 existing function set): [literature.hpcalc.org/community/hp41-pac-math-qrc-en.pdf](https://literature.hpcalc.org/community/hp41-pac-math-qrc-en.pdf) — HIGH confidence

---

*Feature research for: HP-41 Advantage Pac + Advanced Matrix Pac emulation (v3.3 milestone)*
*Researched: 2026-05-25*
