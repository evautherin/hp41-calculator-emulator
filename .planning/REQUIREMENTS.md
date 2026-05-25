# Requirements: HP-41 Calculator Emulator — v3.3 Advantage Pac Emulation

**Defined:** 2026-05-25
**Core Value:** Faithful HP-41 RPN fidelity — behavioral emulation of the HP-41 Advantage Pac (OM 00041-90482, XROM 22 + XROM 24)

## v3.3 Requirements

Requirements for the Advantage Pac milestone. Each maps to roadmap phases.

### XROM Framework

- [ ] **ADV-FW-01**: XROM 22 (ADV CONV + ADV MTRX) and XROM 24 (ADV MATH + ADV TVM) registered as XromModule constants with correct hardware IDs
- [ ] **ADV-FW-02**: `xrom_resolve` gains bit-3 (XROM 22) and bit-4 (XROM 24) resolver arms; fires after TIME_MODULE
- [ ] **ADV-FW-03**: `default_xrom_modules()` updated from `0b0000_0111` to `0b0001_1111`; `migrate_after_load()` auto-upgrades v3.2 save files
- [ ] **ADV-FW-04**: Named-matrix storage on CalcState: `adv_matrices: Vec<AdvMatrix>` with `#[serde(default)]`; matrix identified by ALPHA name, tracks rows/cols/data
- [ ] **ADV-FW-05**: `ModalProgram::Advantage(AdvantageStep)` variant added per ADR-v3.1-005 pattern; dispatch wired in `math1/modal.rs` freeze carve-out
- [ ] **ADV-FW-06**: All new `Op` variants compile in all 4 exhaustive-match sites (dispatch, execute_op, CLI prgm_display, GUI prgm_display)

### ADV CONV — Base Conversion & Boolean Logic (12 ops)

- [ ] **ADV-CONV-01**: BININ — binary string in ALPHA to integer in X
- [ ] **ADV-CONV-02**: BINVIEW — integer in X to binary string in ALPHA + display
- [ ] **ADV-CONV-03**: OCTIN — octal string in ALPHA to integer in X
- [ ] **ADV-CONV-04**: CVTVIEW — display current value in all bases
- [ ] **ADV-CONV-05**: HEXIN — hex string in ALPHA to integer in X
- [ ] **ADV-CONV-06**: HEXVIEW — integer in X to hex string in ALPHA + display
- [ ] **ADV-CONV-07**: NOT — bitwise NOT of X
- [ ] **ADV-CONV-08**: AND — bitwise AND of X and Y
- [ ] **ADV-CONV-09**: OR — bitwise OR of X and Y
- [ ] **ADV-CONV-10**: XOR — bitwise XOR of X and Y
- [ ] **ADV-CONV-11**: ROTXY — rotate X by Y bits
- [ ] **ADV-CONV-12**: BIT? — test bit Y of integer X; skip if set

### ADV MTRX — Named-Matrix Operations (~50 ops)

#### Element Access & Index Management

- [ ] **ADV-MTX-01**: I+ — increment row index
- [ ] **ADV-MTX-02**: I- — decrement row index
- [ ] **ADV-MTX-03**: J+ — increment column index
- [ ] **ADV-MTX-04**: J- — decrement column index
- [ ] **ADV-MTX-05**: MR — read element at current index
- [ ] **ADV-MTX-06**: MS — write X to element at current index
- [ ] **ADV-MTX-07**: MRIJ — read element at explicit I,J
- [ ] **ADV-MTX-08**: MSIJ — write X to element at explicit I,J
- [ ] **ADV-MTX-09**: MRC+ — read element, increment column
- [ ] **ADV-MTX-10**: MRC- — read element, decrement column
- [ ] **ADV-MTX-11**: MRR+ — read element, increment row
- [ ] **ADV-MTX-12**: MRR- — read element, decrement row
- [ ] **ADV-MTX-13**: MSC+ — write element, increment column
- [ ] **ADV-MTX-14**: MSR+ — write element, increment row
- [ ] **ADV-MTX-15**: MRIJR — read element at I,J; advance to next row
- [ ] **ADV-MTX-16**: MSIJR — write element at I,J; advance to next row

#### Matrix Lifecycle

- [ ] **ADV-MTX-17**: MATDIM — create/dimension matrix (ALPHA=name, Y=rows, X=cols)
- [ ] **ADV-MTX-18**: DIM? — return dimensions of named matrix in X/Y
- [ ] **ADV-MTX-19**: MNAME? — return name of current matrix in ALPHA

#### High-Level Matrix Operations

- [ ] **ADV-MTX-20**: MDET — determinant of current matrix (LU decomposition)
- [ ] **ADV-MTX-21**: MINV — inverse of current matrix (in-place)
- [ ] **ADV-MTX-22**: MSYS — solve linear system Ax=b (simultaneous equations)
- [ ] **ADV-MTX-23**: M*M — matrix multiply: result = A * B
- [ ] **ADV-MTX-24**: MAT+ — matrix add: A + B element-wise
- [ ] **ADV-MTX-25**: MAT- — matrix subtract: A - B element-wise
- [ ] **ADV-MTX-26**: MAT* — scalar multiply: all elements times X
- [ ] **ADV-MTX-27**: MAT/ — scalar divide: all elements divided by X
- [ ] **ADV-MTX-28**: TRNPS — transpose matrix (in-place)
- [ ] **ADV-MTX-29**: MMOVE — copy rows/columns of matrix to another
- [ ] **ADV-MTX-30**: MSWAP — swap two rows (or columns)
- [ ] **ADV-MTX-31**: R<>R — exchange two rows
- [ ] **ADV-MTX-32**: R>R? — compare two rows; skip if row k > row l
- [ ] **ADV-MTX-33**: PIV — pivot operation (partial pivoting step)
- [ ] **ADV-MTX-34**: MP — matrix print (all elements via print buffer)

#### Reductions & Norms

- [ ] **ADV-MTX-35**: SUM — sum of all matrix elements
- [ ] **ADV-MTX-36**: SUMAB — sum of absolute values of all elements
- [ ] **ADV-MTX-37**: MAX — maximum element value
- [ ] **ADV-MTX-38**: MIN — minimum element value
- [ ] **ADV-MTX-39**: MAXAB — maximum absolute value element
- [ ] **ADV-MTX-40**: RMAXAB — row maximum absolute value
- [ ] **ADV-MTX-41**: FNRM — Frobenius norm
- [ ] **ADV-MTX-42**: RNRM — row norm
- [ ] **ADV-MTX-43**: RSUM — row sum

#### Complex Matrix Operations

- [ ] **ADV-MTX-44**: C<>C — exchange two complex matrices
- [ ] **ADV-MTX-45**: CMAXAB — max absolute value in complex matrix
- [ ] **ADV-MTX-46**: CNRM — complex matrix norm
- [ ] **ADV-MTX-47**: CSUM — sum of complex matrix elements
- [ ] **ADV-MTX-48**: YC+C — add complex number to complex matrix element

#### Matrix Editor

- [ ] **ADV-MTX-49**: MEDIT — real matrix editor (simplified prompt-driven element entry)
- [ ] **ADV-MTX-50**: CMEDIT — complex matrix editor (simplified prompt-driven)

### ADV MATH — Advanced Mathematics (~47 ops)

#### Matrix Interface Workflows

- [ ] **ADV-MATH-01**: MATRX — full matrix workflow: DIM, input, choose DET/INV/SIMEQ (modal)
- [ ] **ADV-MATH-02**: MTR — simplified matrix entry/solve frontend (modal)

#### Solve & Integrate (HP-15C algorithms)

- [ ] **ADV-MATH-03**: FSOLVE — root of f(x)=0 via Brent/Secant; user-program callback
- [ ] **ADV-MATH-04**: FINTG — Romberg integration of f(x); user-program callback
- [ ] **ADV-MATH-05**: FDIFEQ — differential equations solver (1st/2nd order RK4)

#### Polynomial Roots

- [ ] **ADV-MATH-06**: FROOT — roots of polynomial of arbitrary degree via Laguerre's method
- [ ] **ADV-MATH-07**: PLY — polynomial evaluation: compute P(x)
- [ ] **ADV-MATH-08**: RTS — root output/collection utility

#### Complex Number Extensions

- [ ] **ADV-MATH-09**: MAGZ — magnitude (absolute value) of complex z
- [ ] **ADV-MATH-10**: e^Z — complex exponential
- [ ] **ADV-MATH-11**: LNZ — complex natural logarithm
- [ ] **ADV-MATH-12**: Z^N — complex z raised to integer power n
- [ ] **ADV-MATH-13**: Z^1/N — complex nth root
- [ ] **ADV-MATH-14**: SINZ — complex sine
- [ ] **ADV-MATH-15**: COSZ — complex cosine
- [ ] **ADV-MATH-16**: TANZ — complex tangent
- [ ] **ADV-MATH-17**: a^Z — real base a raised to complex power z
- [ ] **ADV-MATH-18**: LOGZ — complex log base a
- [ ] **ADV-MATH-19**: Z^1/W — complex z raised to 1/w
- [ ] **ADV-MATH-20**: Z^W — complex z raised to complex w
- [ ] **ADV-MATH-21**: AIP — alpha integer append to ALPHA string

#### Advantage Complex Arithmetic (duplicates Math Pac I with Advantage XROM registration)

- [ ] **ADV-MATH-22**: C+ — complex addition (Advantage XROM 24 variant)
- [ ] **ADV-MATH-23**: C- — complex subtraction (Advantage XROM 24 variant)
- [ ] **ADV-MATH-24**: CINV — complex inverse 1/z (Advantage variant)
- [ ] **ADV-MATH-25**: C* — complex multiply (Advantage XROM 24 variant)
- [ ] **ADV-MATH-26**: C/ — complex divide (Advantage XROM 24 variant)

#### Curve Fitting

- [ ] **ADV-MATH-27**: CFIT — curve fit (linear, log, exp, power) to (x,y) data
- [ ] **ADV-MATH-28**: AS — add data point to curve fit accumulation
- [ ] **ADV-MATH-29**: DS — delete/subtract data point from accumulation
- [ ] **ADV-MATH-30**: BFIT — best-fit selection: determine which model fits best
- [ ] **ADV-MATH-31**: FIT — compute fit coefficients for current model
- [ ] **ADV-MATH-32**: Y?X — predict Y from X using current fit
- [ ] **ADV-MATH-33**: SZ? — test if current data set has sufficient points

#### Vector Operations

- [ ] **ADV-MATH-34**: V+ — vector add
- [ ] **ADV-MATH-35**: V- — vector subtract
- [ ] **ADV-MATH-36**: DOT — dot product of two 3D vectors
- [ ] **ADV-MATH-37**: CROSS — cross product (3D)
- [ ] **ADV-MATH-38**: VC — vector cross product (alternate entry)
- [ ] **ADV-MATH-39**: VS — vector scalar multiply
- [ ] **ADV-MATH-40**: VR — vector result recall
- [ ] **ADV-MATH-41**: VE — vector entry (input 3D vector to registers, modal)
- [ ] **ADV-MATH-42**: VXY — vector to X-Y plane projection
- [ ] **ADV-MATH-43**: UV — unit vector
- [ ] **ADV-MATH-44**: V< — vector magnitude (length)
- [ ] **ADV-MATH-45**: VD — vector dot product (variant)
- [ ] **ADV-MATH-46**: V* — vector scalar product (variant)

#### Coordinate Transforms

- [ ] **ADV-MATH-47**: TR — coordinate transformation (2D/3D, one-shot)

### ADV TVM — Time Value of Money (6 ops)

- [ ] **ADV-TVM-01**: TVM — initialize/master TVM solver (modal prompts)
- [ ] **ADV-TVM-02**: N — solve for N (periods)
- [ ] **ADV-TVM-03**: PV — solve for present value
- [ ] **ADV-TVM-04**: PMT — solve for payment
- [ ] **ADV-TVM-05**: FV — solve for future value
- [ ] **ADV-TVM-06**: *I — solve for periodic interest rate (Newton iteration)

### CLI Integration

- [ ] **ADV-CLI-01**: `docs/hp41-advantage-functions.json` authored as fifth JSON canonical source
- [ ] **ADV-CLI-02**: Fifth `OnceLock<Vec<HelpEntry>>` in `help_data.rs` + 5-pool `help_entries_all()` chain
- [ ] **ADV-CLI-03**: All new `op_display_name` arms in `hp41-cli/src/prgm_display.rs` (4-way invariant item 3)
- [ ] **ADV-CLI-04**: `?` help overlay "Advantage Pac (XROM 22)" and "Advantage Pac (XROM 24)" sections
- [ ] **ADV-CLI-05**: `function_matrix_parity.rs` 5-pool partition test
- [ ] **ADV-CLI-06**: `xrom_shadowing.rs` extended to ADV_A.ops + ADV_B.ops (Pitfall 22 across all 5 XROM modules)
- [ ] **ADV-CLI-07**: Modal-prompt routing for MATRX/MTR/TVM/MEDIT/CMEDIT/VE through existing infrastructure
- [ ] **ADV-CLI-08**: Right-panel filter unchanged (XROM functions excluded via `entry.xrom.is_none()`)

### Documentation

- [ ] **ADV-DOC-01**: `docs/hp41-advantage-function-matrix.md` generated via `just docs-matrix` (fifth invocation)
- [ ] **ADV-DOC-02**: `docs/hp41-advantage-divergences.md` — three-bucket divergence catalog
- [ ] **ADV-DOC-03**: ADRs: named-matrix model, FROOT algorithm (Laguerre), dual-XROM design, math1/ visibility promotions
- [ ] **ADV-DOC-04**: `docs/architecture-history.md` v3.3 narrative
- [ ] **ADV-DOC-05**: README v3.3 soft-claim (hard-claim deferred to Phase 47)
- [ ] **ADV-DOC-06**: CLAUDE.md `### v3.3 additions` block

### GUI Integration

- [ ] **ADV-GUI-01**: All new `op_display_name` arms in `hp41-gui/src-tauri/src/prgm_display.rs` (4-way invariant item 4)
- [ ] **ADV-GUI-02**: HelpOverlay.tsx fifth section(s) for Advantage Pac
- [ ] **ADV-GUI-03**: CATALOG 2 entries for XROM 22 + XROM 24
- [ ] **ADV-GUI-04**: Modal prompt LCD rendering for MATRX/MTR/TVM/MEDIT workflows

### Quality Gates

- [ ] **ADV-QUAL-01**: Unified `xrom_op_test_count.rs` covers all Advantage Pac Op variants (>= 5 tests each)
- [ ] **ADV-QUAL-02**: `lint_xrom_assertions.rs` extended to Advantage Pac test files
- [ ] **ADV-QUAL-03**: Coverage gap closure: all `ops/advantage/*.rs` files >= 90% region coverage
- [ ] **ADV-QUAL-04**: Numerical accuracy: FROOT/FINTG/MDET/MINV oracle cases (scipy-derived)
- [ ] **ADV-QUAL-05**: Backward compatibility: `time_backward_compat.rs` extended for v3.2->v3.3 save migration
- [ ] **ADV-QUAL-06**: E2E smoke: Advantage Pac workflow in `hp41-gui/e2e/smoke.spec.js`
- [ ] **ADV-QUAL-07**: Free42 contamination guard extended to `advantage/` directory
- [ ] **ADV-QUAL-08**: `hp41-core` region coverage >= 93% (quality gate floor)
- [ ] **ADV-QUAL-09**: README hard-claim graduated: "feature-complete per Owner's Manual 00041-90482"

## Future Requirements

Deferred to post-v3.3. Tracked but not in current roadmap.

### Post-v3.3

- **RELEASE-01**: Signed binary releases (cargo-dist CLI + tauri-action GUI)
- **NEST-01**: FROOT/FINTG mutual nesting (re-entrant X-MEM buffer stack)
- **XMEM-01**: Full Extended Memory model (EMDIR, EMROOM, EMREG, etc.)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Full M-CODE speed emulation | Behavioral emulation is project scope; cycle-accurate M-CODE excluded per project invariants |
| Full MEDIT/CMEDIT M-CODE editor | Simplified prompt-driven element entry satisfies behavioral spec; full keystroke-interception editor not needed |
| Advanced Matrix Pac (XROM 12) | Community hobbyist ROM by Angel Martin, NOT an official HP product; all PROJECT.md matrix targets are in official Advantage Pac XROM 22 |
| X-MEM full Extended Memory model | Only named-matrix storage needed; full EMDIR/EMROOM/EMREG is separate major feature |
| Matrix size > 14x14 | Cap at 14x14 for consistency with Math Pac I; document as behavioral policy |
| HP-copyrighted ROM bytes | Permanently excluded |

## Traceability

(Updated during roadmap creation)

| Requirement | Phase | Status |
|-------------|-------|--------|
| (populated by roadmapper) | | |

**Coverage:**
- v3.3 requirements: 115 total
- Mapped to phases: 0
- Unmapped: 115

---
*Requirements defined: 2026-05-25*
*Last updated: 2026-05-25 after initial definition*
