# HP-41 Advantage Pac Function Matrix

> Generated from `docs/hp41-advantage-functions.json` via `just docs-matrix`.
> Edit the JSON, regenerate this file, commit both.

## Implemented (v2.x)

| Op | Display | XROM | Category | Status | Phase | Key Path | Description |
|----|---------|------|----------|--------|-------|----------|-------------|
| AdvAnd | AND | Adv Conv / 22-8 | Adv Conv | ✓ v2.x | 43 | `XEQ "AND"` | Bitwise AND of X and Y (36-bit word) |
| AdvBinin | BININ | Adv Conv / 22-1 | Adv Conv | ✓ v2.x | 43 | `XEQ "BININ"` | Input binary number from ALPHA register into X |
| AdvBinview | BINVIEW | Adv Conv / 22-2 | Adv Conv | ✓ v2.x | 43 | `XEQ "BINVIEW"` | Display X as 36-bit binary in ALPHA register |
| AdvBitTest | BIT? | Adv Conv / 22-12 | Adv Conv | ✓ v2.x | 43 | `XEQ "BIT?"` | Skip next step if bit X of Y is set |
| AdvCvtview | CVTVIEW | Adv Conv / 22-6 | Adv Conv | ✓ v2.x | 43 | `XEQ "CVTVIEW"` | Display X in decimal, octal, and hex in ALPHA register |
| AdvHexin | HEXIN | Adv Conv / 22-4 | Adv Conv | ✓ v2.x | 43 | `XEQ "HEXIN"` | Input hexadecimal number from ALPHA register into X |
| AdvHexview | HEXVIEW | Adv Conv / 22-5 | Adv Conv | ✓ v2.x | 43 | `XEQ "HEXVIEW"` | Display X as hexadecimal in ALPHA register |
| AdvNot | NOT | Adv Conv / 22-7 | Adv Conv | ✓ v2.x | 43 | `XEQ "NOT"` | Bitwise NOT of X (36-bit word) |
| AdvOctin | OCTIN | Adv Conv / 22-3 | Adv Conv | ✓ v2.x | 43 | `XEQ "OCTIN"` | Input octal number from ALPHA register into X |
| AdvOr | OR | Adv Conv / 22-9 | Adv Conv | ✓ v2.x | 43 | `XEQ "OR"` | Bitwise OR of X and Y (36-bit word) |
| AdvRotxy | ROTXY | Adv Conv / 22-11 | Adv Conv | ✓ v2.x | 43 | `XEQ "ROTXY"` | Rotate Y left by X bits (36-bit circular rotation) |
| AdvXor | XOR | Adv Conv / 22-10 | Adv Conv | ✓ v2.x | 43 | `XEQ "XOR"` | Bitwise XOR of X and Y (36-bit word) |
| AdvAPowZ | A^Z | Adv Math / 24-12 | Adv Math | ✓ v2.x | 43 | `XEQ "A^Z"` | Complex power A^(X+iY) where real base A is in Z |
| AdvAip | AIP | Adv Math / 24-18 | Adv Math | ✓ v2.x | 43 | `XEQ "AIP"` | Add integer part of X to ALPHA register as character |
| AdvAs | AS | Adv Math / 24-26 | Adv Math | ✓ v2.x | 43 | `XEQ "AS"` | Accumulate statistics: add (X,Y) data point to curve fit registers |
| AdvBfit | BFIT | Adv Math / 24-28 | Adv Math | ✓ v2.x | 43 | `XEQ "BFIT"` | Best fit: compute curve fit coefficients for selected model |
| AdvCDiv | CDIV | Adv Math / 24-17 | Adv Math | ✓ v2.x | 43 | `XEQ "CDIV"` | Complex division: (Z+iT) / (X+iY) -> X (real), Y (imag) |
| AdvCMinus | CSUB | Adv Math / 24-14 | Adv Math | ✓ v2.x | 43 | `XEQ "CSUB"` | Complex subtraction: (Z+iT) - (X+iY) -> X (real), Y (imag) |
| AdvCMul | CMUL | Adv Math / 24-16 | Adv Math | ✓ v2.x | 43 | `XEQ "CMUL"` | Complex multiplication: (X+iY) * (Z+iT) -> X (real), Y (imag) |
| AdvCPlus | CADD | Adv Math / 24-13 | Adv Math | ✓ v2.x | 43 | `XEQ "CADD"` | Complex addition: (X+iY) + (Z+iT) -> X (real), Y (imag) |
| AdvCfit | CFIT | Adv Math / 24-25 | Adv Math | ✓ v2.x | 43 | `XEQ "CFIT"` | Curve fit: select model (lin/exp/log/power) based on best correlation |
| AdvCinv | CINV | Adv Math / 24-15 | Adv Math | ✓ v2.x | 43 | `XEQ "CINV"` | Complex reciprocal 1/(X+iY) -> X (real), Y (imag) |
| AdvCosZ | COSZ | Adv Math / 24-10 | Adv Math | ✓ v2.x | 43 | `XEQ "COSZ"` | Complex cosine cos(X+iY) -> result in X (real) and Y (imag) |
| AdvCross | CROSS | Adv Math / 24-35 | Adv Math | ✓ v2.x | 43 | `XEQ "CROSS"` | Cross product of two 3D vectors; result in first vector registers |
| AdvDot | DOT | Adv Math / 24-34 | Adv Math | ✓ v2.x | 43 | `XEQ "DOT"` | Dot product of two vectors into X |
| AdvDs | DS | Adv Math / 24-27 | Adv Math | ✓ v2.x | 43 | `XEQ "DS"` | Delete statistics: remove (X,Y) data point from curve fit registers |
| AdvExpZ | E^Z | Adv Math / 24-1 | Adv Math | ✓ v2.x | 43 | `XEQ "E^Z"` | Complex exponential e^(X+iY) -> result in X (real) and Y (imag) |
| AdvFdifeq | FDIFEQ | Adv Math / 24-23 | Adv Math | ✓ v2.x | 43 | `XEQ "FDIFEQ"` | Solve ODE y'=f(x,y) via Runge-Kutta; calls user program label |
| AdvFintg | FINTG | Adv Math / 24-22 | Adv Math | ✓ v2.x | 43 | `XEQ "FINTG"` | Integrate f(x) via Romberg quadrature; calls user program label |
| AdvFit | FIT | Adv Math / 24-29 | Adv Math | ✓ v2.x | 43 | `XEQ "FIT"` | Fit: compute predicted Y from X using current curve fit model |
| AdvFroot | FROOT | Adv Math / 24-24 | Adv Math | ✓ v2.x | 43 | `XEQ "FROOT"` | Find all roots of polynomial via Laguerre's method; degree from X |
| AdvFsolve | FSOLVE | Adv Math / 24-21 | Adv Math | ✓ v2.x | 43 | `XEQ "FSOLVE"` | Find root of f(x)=0 via Brent's method; calls user program label |
| AdvLnZ | LNZ | Adv Math / 24-2 | Adv Math | ✓ v2.x | 43 | `XEQ "LNZ"` | Complex natural log ln(X+iY) -> result in X (real) and Y (imag) |
| AdvLogZ | LOGZ | Adv Math / 24-3 | Adv Math | ✓ v2.x | 43 | `XEQ "LOGZ"` | Complex log base 10 log10(X+iY) -> result in X (real) and Y (imag) |
| AdvMagz | \|Z\| | Adv Math / 24-8 | Adv Math | ✓ v2.x | 43 | `XEQ "\|Z\|"` | Magnitude (modulus) of complex number X+iY into X |
| AdvPly | PLY | Adv Math / 24-19 | Adv Math | ✓ v2.x | 43 | `XEQ "PLY"` | Evaluate polynomial at X using coefficients in registers R01..R(N+1) |
| AdvRts | RTS | Adv Math / 24-20 | Adv Math | ✓ v2.x | 43 | `XEQ "RTS"` | Find all roots of polynomial with coefficients in registers R01..R(N+1) |
| AdvSinZ | SINZ | Adv Math / 24-9 | Adv Math | ✓ v2.x | 43 | `XEQ "SINZ"` | Complex sine sin(X+iY) -> result in X (real) and Y (imag) |
| AdvSzQuery | SZ? | Adv Math / 24-31 | Adv Math | ✓ v2.x | 43 | `XEQ "SZ?"` | Skip next step if data point count for curve fit is zero |
| AdvTanZ | TANZ | Adv Math / 24-11 | Adv Math | ✓ v2.x | 43 | `XEQ "TANZ"` | Complex tangent tan(X+iY) -> result in X (real) and Y (imag) |
| AdvTr | TR | Adv Math / 24-45 | Adv Math | ✓ v2.x | 43 | `XEQ "TR"` | Triple product: scalar triple product of three 3D vectors |
| AdvUv | UV | Adv Math / 24-41 | Adv Math | ✓ v2.x | 43 | `XEQ "UV"` | Angle between two vectors into X (in current angle mode) |
| AdvVMag | \|V\| | Adv Math / 24-42 | Adv Math | ✓ v2.x | 43 | `XEQ "\|V\|"` | Magnitude (length) of current vector into X |
| AdvVMinus | V- | Adv Math / 24-33 | Adv Math | ✓ v2.x | 43 | `XEQ "V-"` | Vector subtraction: subtract second vector from first; result in first |
| AdvVPlus | V+ | Adv Math / 24-32 | Adv Math | ✓ v2.x | 43 | `XEQ "V+"` | Vector addition: add vector in registers to second vector; result in first |
| AdvVStar | V* | Adv Math / 24-43 | Adv Math | ✓ v2.x | 43 | `XEQ "V*"` | Scale vector: multiply all components by scalar X |
| AdvVc | VC | Adv Math / 24-36 | Adv Math | ✓ v2.x | 43 | `XEQ "VC"` | Create vector: load components from X, Y, Z into vector registers |
| AdvVd | VD | Adv Math / 24-44 | Adv Math | ✓ v2.x | 43 | `XEQ "VD"` | Divide vector: divide all components by scalar X |
| AdvVe | VE | Adv Math / 24-39 | Adv Math | ✓ v2.x | 43 | `XEQ "VE"` | Unit vector: normalize current vector to length 1 |
| AdvVr | VR | Adv Math / 24-38 | Adv Math | ✓ v2.x | 43 | `XEQ "VR"` | Recall vector components from registers into X, Y, Z |
| AdvVs | VS | Adv Math / 24-37 | Adv Math | ✓ v2.x | 43 | `XEQ "VS"` | Store vector components from registers into X, Y, Z |
| AdvVxy | VXY | Adv Math / 24-40 | Adv Math | ✓ v2.x | 43 | `XEQ "VXY"` | 2D vector from polar (X=magnitude, Y=angle) to cartesian components |
| AdvYQueryX | Y?X | Adv Math / 24-30 | Adv Math | ✓ v2.x | 43 | `XEQ "Y?X"` | Predict X from Y using current inverse curve fit model |
| AdvZPow1n | Z^1/N | Adv Math / 24-5 | Adv Math | ✓ v2.x | 43 | `XEQ "Z^1/N"` | Complex Nth root (X+iY)^(1/N) where N is in Z |
| AdvZPow1w | Z^1/W | Adv Math / 24-7 | Adv Math | ✓ v2.x | 43 | `XEQ "Z^1/W"` | Complex root (X+iY)^(1/(Z+iT)) -> result in X (real) and Y (imag) |
| AdvZPowN | Z^N | Adv Math / 24-4 | Adv Math | ✓ v2.x | 43 | `XEQ "Z^N"` | Complex integer power (X+iY)^N where N is in Z |
| AdvZPowW | Z^W | Adv Math / 24-6 | Adv Math | ✓ v2.x | 43 | `XEQ "Z^W"` | Complex power (X+iY)^(Z+iT) -> result in X (real) and Y (imag) |
| AdvCExchangeC | C<>C | Adv Conv / 22-55 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "C<>C"` | Exchange columns Y and X of current matrix |
| AdvCmaxab | CMAXAB | Adv Conv / 22-56 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "CMAXAB"` | Column index of maximum \|element\| in current matrix into X |
| AdvCmedit | CMEDIT | Adv Conv / 22-63 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "CMEDIT"` | Edit complex matrix elements interactively (real, imag pairs) |
| AdvCnrm | CNRM | Adv Conv / 22-57 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "CNRM"` | Column norm (max absolute column sum) of current matrix into X |
| AdvCsum | CSUM | Adv Conv / 22-58 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "CSUM"` | Sum of column J of current matrix into X |
| AdvDimQuery | DIM? | Adv Conv / 22-30 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "DIM?"` | Recall dimensions of current matrix: Y=rows, X=cols |
| AdvFnrm | FNRM | Adv Conv / 22-44 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "FNRM"` | Frobenius norm (sqrt of sum of squared elements) into X |
| AdvIMinus | I- | Adv Conv / 22-14 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "I-"` | Decrement matrix row index I by 1 |
| AdvIPlus | I+ | Adv Conv / 22-13 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "I+"` | Increment matrix row index I by 1 |
| AdvJMinus | J- | Adv Conv / 22-16 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "J-"` | Decrement matrix column index J by 1 |
| AdvJPlus | J+ | Adv Conv / 22-15 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "J+"` | Increment matrix column index J by 1 |
| AdvMMulM | M*M | Adv Conv / 22-48 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "M*M"` | Multiply two named matrices; result into third named matrix |
| AdvMatMinus | MAT- | Adv Conv / 22-50 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MAT-"` | Subtract two named matrices element-wise; result into third |
| AdvMatPlus | MAT+ | Adv Conv / 22-49 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MAT+"` | Add two named matrices element-wise; result into third |
| AdvMatScalarDiv | MAT/C | Adv Conv / 22-52 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MAT/C"` | Divide all elements of current matrix by scalar X |
| AdvMatScalarMul | MAT*C | Adv Conv / 22-51 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MAT*C"` | Multiply all elements of current matrix by scalar X |
| AdvMatdim | MATDIM | Adv Conv / 22-31 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MATDIM"` | Redimension current matrix to Y rows by X cols |
| AdvMatrx | MATRX | Adv Conv / 22-60 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MATRX"` | Interactive matrix editor workflow (ALPHA selects matrix) |
| AdvMax | MAX | Adv Conv / 22-38 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MAX"` | Maximum element of current matrix into X; row/col in Y/Z |
| AdvMaxab | MAXAB | Adv Conv / 22-39 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MAXAB"` | Maximum \|element\| of current matrix into X; row/col in Y/Z |
| AdvMdet | MDET | Adv Conv / 22-45 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MDET"` | Determinant of current matrix into X |
| AdvMedit | MEDIT | Adv Conv / 22-62 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MEDIT"` | Edit real matrix elements interactively row by row |
| AdvMin | MIN | Adv Conv / 22-40 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MIN"` | Minimum element of current matrix into X; row/col in Y/Z |
| AdvMinv | MINV | Adv Conv / 22-46 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MINV"` | Invert current matrix in place |
| AdvMmove | MMOVE | Adv Conv / 22-54 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MMOVE"` | Copy source matrix (ALPHA) into destination matrix (second ALPHA) |
| AdvMnameQuery | MNAME? | Adv Conv / 22-29 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MNAME?"` | Store current matrix name into ALPHA register |
| AdvMp | MP | Adv Conv / 22-32 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MP"` | Set current matrix pointer to named matrix in ALPHA |
| AdvMr | MR | Adv Conv / 22-17 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MR"` | Recall element (I,J) of current matrix into X |
| AdvMrcMinus | MRC- | Adv Conv / 22-23 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MRC-"` | Subtract X from element (I,J) and recall result into X |
| AdvMrcPlus | MRC+ | Adv Conv / 22-22 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MRC+"` | Add X to element (I,J) and recall result into X |
| AdvMrij | MRIJ | Adv Conv / 22-19 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MRIJ"` | Recall element (Y,X) of current matrix; update I,J |
| AdvMrrMinus | MRR- | Adv Conv / 22-25 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MRR-"` | Subtract X from element (I,J) and store; advance column J |
| AdvMrrPlus | MRR+ | Adv Conv / 22-24 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MRR+"` | Add X to element (I,J) and store; advance column J |
| AdvMs | MS | Adv Conv / 22-18 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MS"` | Store X into element (I,J) of current matrix |
| AdvMscPlus | MSC+ | Adv Conv / 22-27 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MSC+"` | Store X into element (I,J); advance column J |
| AdvMsij | MSIJ | Adv Conv / 22-20 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MSIJ"` | Store Z into element (Y,X) of current matrix; update I,J |
| AdvMsijr | MSIJR | Adv Conv / 22-21 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MSIJR"` | Store Z into element (Y,X); recall it back into X; update I,J |
| AdvMsrPlus | MSR+ | Adv Conv / 22-26 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MSR+"` | Store X into element (I,J); advance row I |
| AdvMswap | MSWAP | Adv Conv / 22-28 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MSWAP"` | Exchange X register with element (I,J) of current matrix |
| AdvMsys | MSYS | Adv Conv / 22-47 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MSYS"` | Solve linear system: coefficient matrix in ALPHA, RHS in second ALPHA |
| AdvMtr | MTR | Adv Conv / 22-61 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "MTR"` | Select named matrix (ALPHA) as current matrix pointer |
| AdvPiv | PIV | Adv Conv / 22-33 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "PIV"` | Partial pivot: swap rows to maximize \|element (I,J)\| |
| AdvRExchangeR | R<>R | Adv Conv / 22-34 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "R<>R"` | Exchange rows Y and X of current matrix |
| AdvRGtRQuery | R>R? | Adv Conv / 22-35 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "R>R?"` | Skip next step if row I > row count of current matrix |
| AdvRmaxab | RMAXAB | Adv Conv / 22-41 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "RMAXAB"` | Row index of maximum \|element\| in current matrix into X |
| AdvRnrm | RNRM | Adv Conv / 22-42 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "RNRM"` | Row norm (max absolute row sum) of current matrix into X |
| AdvRsum | RSUM | Adv Conv / 22-43 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "RSUM"` | Sum of row I of current matrix into X |
| AdvSum | SUM | Adv Conv / 22-36 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "SUM"` | Sum all elements of current matrix into X |
| AdvSumab | SUMAB | Adv Conv / 22-37 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "SUMAB"` | Sum absolute values of all elements of current matrix into X |
| AdvTrnps | TRNPS | Adv Conv / 22-53 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "TRNPS"` | Transpose current matrix in place |
| AdvYcPlusC | YC+C | Adv Conv / 22-59 | Adv Mtrx | ✓ v2.x | 43 | `XEQ "YC+C"` | Scale column Y by X and add to column J of current matrix |
| AdvTvm | TVM | Adv Math / 24-46 | Adv TVM | ✓ v2.x | 43 | `XEQ "TVM"` | Time Value of Money solver: interactive menu for N/I/PV/PMT/FV |
| AdvTvmFv | FV | Adv Math / 24-50 | Adv TVM | ✓ v2.x | 43 | `XEQ "FV"` | Solve for future value FV in TVM equation |
| AdvTvmN | N | Adv Math / 24-47 | Adv TVM | ✓ v2.x | 43 | `XEQ "N"` | Solve for number of periods N in TVM equation |
| AdvTvmPmt | PMT | Adv Math / 24-49 | Adv TVM | ✓ v2.x | 43 | `XEQ "PMT"` | Solve for periodic payment PMT in TVM equation |
| AdvTvmPv | PV | Adv Math / 24-48 | Adv TVM | ✓ v2.x | 43 | `XEQ "PV"` | Solve for present value PV in TVM equation |
| AdvTvmStarI | *I | Adv Math / 24-51 | Adv TVM | ✓ v2.x | 43 | `XEQ "*I"` | Solve for periodic interest rate *I in TVM equation |

## v3.x Deferred (Module Pacs)

_None._
