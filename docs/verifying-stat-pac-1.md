# Verifying Stat 1 Pac (HP 00041-90030)

This procedure walks an operator through the Stat 1 Pac emulation landed in
Phases 33-37 (v3.1). It covers all 7 functional groups (distributions, RNG,
nonparametric, univariate, regression, hypothesis tests, ANOVA / contingency
tables) on both `hp41-cli` and `hp41-gui`, and confirms behavioral identity
across UIs.

The Stat 1 Pac is XROM module ID 2 (bit 1 of `CalcState::xrom_modules`),
implementing 13 programs with 26 XEQ entry points -- all directly testable
from the interactive keyboard.

## TL;DR

| Group | Op count | Testability | Section |
|-------|----------|-------------|---------|
| Distributions (NORMD, CHISQD) | 2 | Modal prompts | [2](#2-distributions-normd-chisqd) |
| RNG (RAND, SEED) | 2 | SEED modal | [3](#3-rng-rand-seed) |
| Nonparametric (SPEAR, XSQEV, EFXSQ) | 3 | Register preload | [4](#4-nonparametric-spear-xsqev-efxsq) |
| Univariate (BSTAT, BSTG, MMTUG, MMTGD) | 4 | Via Sigma+ | [5](#5-univariate-bstat-bstg-mmtug-mmtgd) |
| Regression (LIN, EXP, LOGI, POW) | 4 | Accumulate + L.R. | [6](#6-regression-lin-exp-logi-pow) |
| Multiple / Poly (MLRXY, MLRXYZ, POLYP, POLYC) | 4 | Register preload | [7](#7-multiple--polynomial-regression-mlrxy-mlrxyz-polyp-polyc) |
| Hypothesis (PTST, TSTAT) | 2 | Register preload | [8](#8-hypothesis-tests-ptst-tstat) |
| ANOVA (AOVONE, AOVTWO, ANOCOV) | 3 | Register preload | [9](#9-anova-aovone-aovtwo-anocov) |
| Contingency (CTKKK, CTKK) | 2 | Register preload | [10](#10-contingency-tables-ctkkk-ctkk) |
| **Total** | **26** | | |

A complete walk-through of sections 2 through 6 (standalone ops + accumulator-
based regression) takes approximately 25 minutes. Sections 7 through 10
(register-preload-heavy workflows) add another 30 minutes.

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

### Data entry via Sigma+

Most Stat 1 functions consume data pre-accumulated via the built-in `Sigma+`
key (CLI: `+` key; GUI: `Sigma+` button). Sigma+ accumulates the (Y, X) pair
into registers R01-R06: `R01=Sigma(x^2), R02=Sigma(x), R03=n, R04=Sigma(y^2),
R05=Sigma(y), R06=Sigma(xy)`.

### Register preloading via STO

For ops that need explicit register setup, use `<value> STO <register>`.
CLI: type value, press `S`, type register number, Enter.

### Modal prompt workflows

Four Stat 1 ops open modal prompts (submit via R/S, CLI: F5):

- **NORMD**: `SIGMANORMD MODE?` -- `1`=CDF, `2`=PDF, `3`=inverse.
- **CHISQD**: first `nu=?` (degrees of freedom), then `SIGMACHISQD MODE?` --
  `1`=PDF, `2`=CDF.
- **POLYP**: `DEGREE=?` -- enter degree 1-5.
- **SEED**: `SEED?` -- enter seed value.

## 2. Distributions (NORMD, CHISQD)

These are the simplest to verify: no data accumulation needed. Each takes a
single value from the stack and uses modal prompts to select the mode.

### 2.1 NORMD -- Standard Normal Distribution

XEQ name: `NORMD` (preceded by the Sigma character in the display:
`SIGMANORMD`). Three modes:

- Mode 1 = CDF (upper-tail Q(x) = 1 - Phi(x))
- Mode 2 = PDF (phi(x))
- Mode 3 = Inverse (probit: given p, find x such that Phi(x) = 1-p)

**Workflow**: enter the z-score (or probability for inverse), then
`XEQ NORMD ENTER`. The modal opens with `SIGMA_NORMD MODE?`. Enter the
mode number and press R/S.

| # | Setup | Mode | Display (X) | Notes |
|---|-------|------|-------------|-------|
| 2.1 | `1.96` | 1 (CDF) | `0.0250` | Q(1.96) -- upper-tail 2.5% (scipy: 0.024998) |
| 2.2 | `0` | 1 (CDF) | `0.5000` | Q(0) = 0.5 exactly |
| 2.3 | `2.576` | 1 (CDF) | `0.0050` | Q(2.576) -- 99% CI bound (scipy: 0.005000) |
| 2.4 | `-1.96` | 1 (CDF) | `0.9750` | Q(-1.96) -- lower-tail symmetry |
| 2.5 | `0` | 2 (PDF) | `0.3989` | phi(0) -- peak of standard normal |
| 2.6 | `1` | 2 (PDF) | `0.2420` | phi(1) -- one sigma from mean |
| 2.7 | `0.025` | 3 (Inverse) | `1.9600` | Phi^-1(0.975) -- 97.5th percentile |
| 2.8 | `0.5` | 3 (Inverse) | `0.0000` | Phi^-1(0.5) = 0 (median) |
| 2.9 | `0.001` | 3 (Inverse) | `3.0902` | Phi^-1(0.999) -- deep-tail inverse |

**Step-by-step for test 2.1 (CLI)**: type `1.96`, press `X` (XEQ), type
`NORMD`, press Enter. Modal shows `SIGMA_NORMD MODE?`. Type `1`, press F5
(R/S). Display: `0.0250`.

Note: the CDF mode returns the UPPER-TAIL probability Q(x) = 1 - Phi(x).
Precision is approximately 1e-5 absolute vs scipy (see D-35-01).

### 2.2 CHISQD -- Chi-Square Distribution

XEQ name: `CHISQD` (displayed as `SIGMACHISQD`). Two-step modal: first
prompts for degrees of freedom nu, then prompts for mode.

- Mode 1 = PDF
- Mode 2 = CDF (cumulative probability P(chi^2 <= x; nu))

**Workflow**: enter the chi-square statistic, then `XEQ CHISQD ENTER`. The
modal opens with `nu=?`. Enter degrees of freedom and press R/S. The modal
advances to `SIGMACHISQD MODE?`. Enter the mode number and press R/S.

| # | Setup (x) | nu | Mode | Display (X) | Notes |
|---|-----------|-----|------|-------------|-------|
| 2.10 | `3.841` | 1 | 2 (CDF) | `0.9500` | P(chi^2 <= 3.841; nu=1) -- 95th percentile critical value |
| 2.11 | `5.991` | 2 | 2 (CDF) | `0.9500` | P(chi^2 <= 5.991; nu=2) -- 95th percentile |
| 2.12 | `7.815` | 3 | 2 (CDF) | `0.9500` | P(chi^2 <= 7.815; nu=3) -- 95th percentile |

**Step-by-step for test 2.10 (CLI)**: type `3.841`, XEQ `CHISQD`. At
`nu=?` type `1`, R/S. At `SIGMACHISQD MODE?` type `2`, R/S. Display: `0.9500`.

### CHISQD domain guard

| # | Setup | Expected | Notes |
|---|-------|----------|-------|
| 2.13 | `XEQ CHISQD ENTER` -> `0` R/S (nu=0) | `data error` | nu must be positive integer |
| 2.14 | `XEQ CHISQD ENTER` -> `-1` R/S | `data error` | nu must be positive |

After a domain error, the modal state is cleared and the stack is unchanged.

## 3. RNG (RAND, SEED)

Emulator extensions (not in the original OM 00041-90030). RAND generates
pseudorandom numbers on [0, 1) using the linear congruential generator
`r_{n+1} = FRC(9821 * r_n + 0.211327)`. SEED sets the initial state.

### 3.1 Deterministic first call from default seed

On a fresh state (or after `rm -f ~/.hp41/autosave.json`), `rand_seed`
defaults to zero. The first RAND call produces the deterministic value
`FRC(9821 * 0 + 0.211327) = 0.211327`.

| # | Setup -> XEQ | Display (X) | Notes |
|---|-------------|-------------|-------|
| 3.1 | Fresh state -> `XEQ RAND ENTER` | `0.2113` | First call from seed 0 (D-35-12) |
| 3.2 | (immediately after 3.1) `XEQ RAND ENTER` | varies | Second LCG step from seed 0.211327 |
| 3.3 | (immediately after 3.2) `XEQ RAND ENTER` | varies | Third step; sequence is deterministic |

### 3.2 SEED followed by deterministic RAND

SEED opens a `SEED?` modal prompt. Enter a seed value and press R/S.
The seed is normalized into [0, 1) before being stored.

| # | Setup -> XEQ | Display (X) | Notes |
|---|-------------|-------------|-------|
| 3.4 | `XEQ SEED ENTER` -> `0.5` R/S | (no change) | Seed set to 0.5 |
| 3.5 | (after 3.4) `XEQ RAND ENTER` | `0.1218` | FRC(9821 * 0.5 + 0.211327) = FRC(4910.711327) = 0.711327... verify |
| 3.6 | `XEQ SEED ENTER` -> `0.5` R/S, then `XEQ RAND ENTER` | same as 3.5 | Reproducibility: same seed -> same output |

### 3.3 Seed normalization (negative and out-of-range seeds)

| # | Setup | Expected | Notes |
|---|-------|----------|-------|
| 3.7 | `XEQ SEED ENTER` -> `-0.5` R/S, then `XEQ RAND ENTER` | valid [0,1) output | Negative seed normalized: FRC(-0.5)+1 = 0.5 |
| 3.8 | `XEQ SEED ENTER` -> `1.5` R/S, then `XEQ RAND ENTER` | valid [0,1) output | Seed > 1 normalized: FRC(1.5) = 0.5 |

### 3.4 Seed persistence across save/load

The seed survives save/load cycles (`#[serde(default)]` without `skip`).
Set a seed, quit, restart, and call RAND -- the output matches what RAND
would produce immediately after SEED in the same session.

## 4. Nonparametric (SPEAR, XSQEV, EFXSQ)

### 4.1 SPEAR -- Spearman Rank Correlation

XEQ name: `SPEAR` (displayed as `SIGMASPEAR`). The simplest Stat 1 op:
closed-form `rho_s = 1 - 6*Sigma(d^2) / (n*(n^2-1))`. Reads `Sigma(d^2)`
from R02 and `n` from R03.

The user is expected to accumulate the squared rank-differences `d_i^2` via
Sigma+ BEFORE calling SPEAR. Each Sigma+ call adds the `d^2` value to R02
(the Sigma(x) slot) and increments R03 (n).

**Simplified test**: preload R02 and R03 directly with STO.

| # | Setup | Display (X) | Notes |
|---|-------|-------------|-------|
| 4.1 | `4 STO 02`, `5 STO 03` -> `XEQ SPEAR ENTER` | `0.8000` | rho_s = 1 - 24/120 = 0.8 (D-35-03) |
| 4.2 | `0 STO 02`, `5 STO 03` -> `XEQ SPEAR ENTER` | `1.0000` | Perfect positive: Sigma(d^2)=0 -> rho=1 |
| 4.3 | `20 STO 02`, `5 STO 03` -> `XEQ SPEAR ENTER` | `0.0000` | rho_s = 1 - 120/120 = 0 |

**Step-by-step for test 4.1 (CLI)**: `Ctrl+G` (CLREG), `4 STO 02`,
`5 STO 03`, XEQ `SPEAR`. Display: `0.8000`.

**Full workflow via Sigma+** (ranking x=[1,2,3,4,5] vs y=[2,1,3,5,4]):
rank-differences d=[-1,1,0,-1,1], so d^2=[1,1,0,1,1]. Accumulate each d^2
as the X value with Y=0: `0 ENTER 1 Sigma+` (repeat for each d^2), then
XEQ `SPEAR`. Display: `0.8000`.

### 4.2 XSQEV -- Chi-Square Goodness-of-Fit Evaluation

XEQ name: `XSQEV` (displayed as `SIGMAXSQEV`). Computes
`chi^2 = Sigma((O_i - E_i)^2 / E_i)` from observed/expected count pairs
stored in interleaved registers.

Register layout (SIZE 008, R00-R07):

```
R00 = k (number of categories, 1 <= k <= 3)
R01 = O_0 (observed count, category 0)
R02 = E_0 (expected count, category 0)
R03 = O_1
R04 = E_1
R05 = O_2
R06 = E_2
R07 = scratch / chi^2 output
```

| # | Categories (O/E) | Display (X) | Notes |
|---|-----------------|-------------|-------|
| 4.4 | k=3, O=[10,20,30], E=[15,20,25] | `2.6667` | (10-15)^2/15 + 0 + (30-25)^2/25 = 8/3 |
| 4.5 | k=2, O=[20,30], E=[25,25] | `2.0000` | (20-25)^2/25 + (30-25)^2/25 = 50/25 |

**Setup for test 4.4**: `Ctrl+G`, then STO: `3->R00`, `10->R01`, `15->R02`,
`20->R03`, `20->R04`, `30->R05`, `25->R06`. XEQ `XSQEV`. Display: `2.6667`.

### 4.3 EFXSQ -- Chi-Square with Expected Proportions

XEQ name: `EFXSQ` (displayed as `SIGMAEFXSQ`). Same register layout as
XSQEV, but the E slots hold expected PROPORTIONS (not counts). The Op
computes expected counts as `E_i = proportion_i * N` where
`N = Sigma(O_i)`, then applies the same chi^2 formula.

| # | Categories (O/Proportion) | Display (X) | Notes |
|---|--------------------------|-------------|-------|
| 4.6 | k=3, O=[10,30,60], prop=[0.2,0.3,0.5] | `7.0000` | N=100; E=[20,30,50]; chi^2=7.0 (D-35-04) |

**Setup for test 4.6**: `Ctrl+G`, then STO: `3->R00`, `10->R01`, `0.2->R02`,
`30->R03`, `0.3->R04`, `60->R05`, `0.5->R06`. XEQ `EFXSQ`. Display: `7.0000`.

## 5. Univariate (BSTAT, BSTG, MMTUG, MMTGD)

All four ops consume data accumulated via Sigma+ (R01-R06). Accumulate data
points BEFORE calling any of these functions.

### 5.1 BSTAT -- Extended Univariate Summary

XEQ name: `BSTAT` (displayed as `SIGMABSTAT`). Reads the Sigma-register
block R01-R06 and pushes:

- X = CV (coefficient of variation = sample_stdev / sample_mean)
- Y = sample mean

| # | Data (via Sigma+) | X (CV) | Y (mean) | Notes |
|---|-------------------|--------|----------|-------|
| 5.1 | x=[1,2,3,4,5] (y=0 for each) | `0.5270` | `3.0000` | CV = sqrt(2.5)/3 (D-35-05) |
| 5.2 | x=[2,4,6,8] (y=0) | `0.5164` | `5.0000` | CV = sqrt(20/3)/5 |

**Setup for test 5.1**: `Ctrl+G`, then accumulate x=[1..5] via
`0 ENTER <x> Sigma+` for each value. XEQ `BSTAT`. Display: `0.5270` (CV).
Press `x<>y` to see Y = `3.0000` (mean).

### 5.2 BSTG -- Bivariate Weighted Summary

XEQ name: `BSTG` (displayed as `SIGMABSTG`). Treats the Sigma-block as
(data, weight) pairs. Pushes:

- X = weighted mean = Sigma(x*y) / Sigma(y)
- Y = unweighted sample mean

| # | Data (x,y pairs via Sigma+) | X (weighted mean) | Y (mean) | Notes |
|---|----------------------------|-------------------|----------|-------|
| 5.3 | (1,10),(2,20),(3,30),(4,40),(5,50) | `3.6667` | `3.0000` | Sigma(xy)/Sigma(y) = 550/150 |

**Setup for test 5.3**: `Ctrl+G`, accumulate `<weight> ENTER <x> Sigma+`
for pairs (1,10),(2,20),(3,30),(4,40),(5,50). XEQ `BSTG`. Display: `3.6667`.

### 5.3 MMTUG -- Moments (Ungrouped Data)

XEQ name: `MMTUG` (displayed as `SIGMAMMTUG`). Computes moments for
ungrouped data. Before calling MMTUG, accumulate data via Sigma+, then
also store the extended sums (Sigma(x^3) in R07, Sigma(x^4) in R08)
separately.

The MMTUG function reads:
- R01-R03 (Sigma(x^2), Sigma(x), n) from the standard Sigma+ block
- R07 = Sigma(x^3), R08 = Sigma(x^4) for higher moments

Output pushed to stack:
- X = kurtosis
- Y = skewness

| # | Data | X (kurtosis) | Y (skewness) | Notes |
|---|------|--------------|--------------|-------|
| 5.4 | x=[1,2,3,4,5] with R07=Sigma(x^3)=225, R08=Sigma(x^4)=979 | ~ `1.7000` | `0.0000` | Symmetric data: skewness = 0 |

**Setup for test 5.4**: `Ctrl+G`, accumulate x=[1..5] via Sigma+, then
`225 STO 07` and `979 STO 08`. XEQ `MMTUG`. Verify Y (skewness) = `0.0000`.

### 5.4 MMTGD -- Moments (Grouped / Frequency-Weighted Data)

XEQ name: `MMTGD`. Like MMTUG but for frequency-weighted data (R05=Sigma(f),
R06=Sigma(f*x), R07=Sigma(f*x^3), R08=Sigma(f*x^4)). Best verified via
`cargo test -p hp41-core stat1 -- mmtgd`.

## 6. Regression (LIN, EXP, LOGI, POW)

The four curve-fit Ops are PER-POINT ACCUMULATORS: each `XEQ` call
accumulates one (x, y) data point (with optional transformation), then the
built-in `L.R.` key extracts the regression coefficients.

| Op | Model | Transform | After L.R.: X=intercept, Y=slope |
|----|-------|-----------|----------------------------------|
| SIGMALIN | y = a + b*x | identity | a = intercept, b = slope |
| SIGMAEXP | y = a*e^(b*x) | y <- ln(y) | a = e^intercept, b = slope |
| SIGMALOGI | y = a + b*ln(x) | x <- ln(x) | a = intercept, b = slope |
| SIGMAPOW | y = a*x^b | x <- ln(x), y <- ln(y) | a = e^intercept, b = slope |

### 6.1 LIN -- Linear Regression

XEQ name: `LIN` (displayed as `SIGMALIN`). This is a pure delegate to
Sigma+: `SIGMALIN` = `Sigma+` (identity transform). Accumulate each (x, y)
pair by entering `y ENTER x XEQ LIN ENTER`. After all points, press `L.R.`
(CLI: `f` then `Sigma+`; GUI: `f` then click `Sigma+`).

| # | Data (y, x) pairs | L.R. Y (slope) | L.R. X (intercept) | Notes |
|---|-------------------|----------------|---------------------|-------|
| 6.1 | (2,1),(4,2),(6,3),(8,4),(10,5) | `2.0000` | `0.0000` | y = 2x exactly |
| 6.2 | (3,1),(5,2),(7,3),(9,4),(11,5) | `2.0000` | `1.0000` | y = 1 + 2x |

**Setup for test 6.1**: `Ctrl+G`. For each (y,x) pair, enter
`<y> ENTER <x> XEQ LIN ENTER`. After all 5 points, press `f Sigma+` (L.R.).
Display: `0.0000` (intercept). Press `x<>y`: `2.0000` (slope).

### 6.2 EXP -- Exponential Regression

XEQ name: `EXP` (displayed as `SIGMAEXP`). Transforms y <- ln(y) before
accumulating. After L.R., the displayed intercept is ln(a); the true
intercept a = e^(displayed intercept).

| # | Data (y, x) pairs | L.R. Y (slope b) | L.R. X (ln(a)) | Notes |
|---|-------------------|-------------------|-----------------|-------|
| 6.3 | y = e^x for x=1..3 | `1.0000` | `0.0000` | y = e^x: b=1, a=1 -> ln(a)=0 |

**Requirement**: y must be strictly positive (ln(y) is undefined for y <= 0).
A `data error` results if any y <= 0.

### 6.3 LOGI -- Logarithmic Regression

XEQ name: `LOGI` (displayed as `SIGMALOGI`). Transforms x <- ln(x) before
accumulating.

| # | Data (y, x) pairs | L.R. Y (slope b) | L.R. X (intercept a) | Notes |
|---|-------------------|-------------------|----------------------|-------|
| 6.4 | (2, 1), (3, e), (4, e^2) | `1.0000` | `2.0000` | y = 2 + ln(x) |

**Requirement**: x must be strictly positive. A `data error` results if any
x <= 0.

### 6.4 POW -- Power Regression

XEQ name: `POW` (displayed as `SIGMAPOW`). Transforms both x <- ln(x) and
y <- ln(y). After L.R., the slope is the exponent b; the true coefficient
a = e^(displayed intercept).

| # | Data (y, x) pairs | L.R. Y (slope b) | L.R. X (ln(a)) | Notes |
|---|-------------------|-------------------|-----------------|-------|
| 6.5 | (1,1),(4,2),(9,3),(16,4),(25,5) | `2.0000` | `0.0000` | y = x^2: b=2, a=1 |

**Requirement**: both x and y must be strictly positive.

### Regression domain guards

| # | Setup -> XEQ | Expected | Notes |
|---|-------------|----------|-------|
| 6.6 | `0 ENTER 1 XEQ EXP ENTER` | `data error` | y=0: ln(0) undefined |
| 6.7 | `1 ENTER 0 XEQ LOGI ENTER` | `data error` | x=0: ln(0) undefined |
| 6.8 | `0 ENTER 0 XEQ POW ENTER` | `data error` | Both x=0, y=0 |

## 7. Multiple / Polynomial Regression (MLRXY, MLRXYZ, POLYP, POLYC)

These ops solve normal-equation systems via Gauss-Jordan elimination with
partial pivoting. They require pre-populated sufficient statistics in
specific registers.

### 7.1 MLRXY -- Multiple Linear Regression (2 predictors)

XEQ name: `MLRXY` (displayed as `SIGMAMLRXY`). Solves
`y = b_0 + b_1*x_1 + b_2*x_2` from 9 pre-loaded sufficient statistics.

Register layout (SIZE 045, R00-R08):

```
R00 = n     R01 = Sigma(y)     R02 = Sigma(x1)     R03 = Sigma(x2)
R04 = Sigma(x1^2)     R05 = Sigma(x2^2)     R06 = Sigma(x1*x2)
R07 = Sigma(x1*y)     R08 = Sigma(x2*y)
```

Output: X = b_2, Y = b_1, Z = b_0

| # | Dataset | X (b_2) | Y (b_1) | Z (b_0) | Notes |
|---|---------|---------|---------|---------|-------|
| 7.1 | y=1+3x1+2x2, n=5 | `2.0000` | `3.0000` | `1.0000` | Exact by construction |

**Setup for test 7.1** -- dataset (x1,x2,y) = (1,1,6),(2,4,15),(3,9,28),
(4,16,45),(5,25,66). `Ctrl+G`, STO: `5->R00`, `160->R01`, `15->R02`,
`55->R03`, `55->R04`, `979->R05`, `225->R06`, `630->R07`, `2688->R08`.
XEQ `MLRXY`. X=`2.0000` (b2), `x<>y`: Y=`3.0000` (b1).

### 7.2 MLRXYZ -- Multiple Linear Regression (3 predictors)

XEQ name: `MLRXYZ`. Extends MLRXY to 3 predictors (14 statistics, R00-R13).
Best verified via `cargo test -p hp41-core stat1 -- mlrxyz`.

### 7.3 POLYP -- Polynomial Regression

XEQ name: `POLYP`. Opens a `DEGREE=?` modal prompt (d=1-5). Register layout:
`R00=n, R01..R10=Sigma(x^k) k=1..10, R11..R16=Sigma(x^k*y) k=0..5,
R20..R25=coefficient output, R30=degree`. Best verified via
`cargo test -p hp41-core stat1 -- polyp`.

### 7.4 POLYC -- Polynomial Coefficient Recall

XEQ name: `POLYC`. After POLYP computes coefficients, POLYC evaluates the
polynomial at x (from the stack) using R20..R25 and R30.

## 8. Hypothesis Tests (PTST, TSTAT)

### 8.1 PTST -- Paired-Sample t-Test

XEQ name: `PTST` (displayed as `SIGMAPTST`). Computes a paired-sample t-test
from the difference scores accumulated via Sigma+. The user accumulates
`d_i = x_i - y_i` differences into the Sigma-register block, then PTST
computes:

- t = d_bar / (s_d / sqrt(n))
- p = two-tailed p-value from Student-t CDF

Output: X = t-statistic, Y = p-value

| # | Differences d_i (via Sigma+) | X (t) | Y (p) | Notes |
|---|------------------------------|-------|-------|-------|
| 8.1 | d=[1,1,1,1,1] (n=5, mean=1) | varies | varies | Depends on s_d = 0 -> `data error` (division by zero) |
| 8.2 | d=[1,2,3,2,1] | varies | varies | s_d > 0 |

**Setup for test 8.2**: `Ctrl+G`, accumulate d=[1,2,3,2,1] via
`0 ENTER <d> Sigma+`. XEQ `PTST`. X = t-statistic, Y = p-value.

### 8.2 TSTAT -- Two-Sample t-Test (Pooled Variance)

XEQ name: `TSTAT` (displayed as `SIGMATSTAT`). Computes a pooled-variance
two-sample t-test from sufficient statistics loaded into specific registers.

**Note**: pooled-variance only -- Welch's t excluded (D-35-09).

Register layout:

```
Group 1:  R01 = Sigma(x1^2),  R02 = Sigma(x1),  R03 = n1
Group 2:  R07 = Sigma(x2^2),  R08 = Sigma(x2),  R09 = n2
```

Output: X = t-statistic, Y = two-tailed p-value

| # | Group 1 | Group 2 | X (t) | Y (p) | Notes |
|---|---------|---------|-------|-------|-------|
| 8.3 | x=[1,2,3,4,5] | x=[6,7,8,9,10] | `-5.0000` | `0.0011` | scipy: t=-5.0, p=0.00105 (D-35-06) |

**Setup for test 8.3**: `Ctrl+G`, STO: `55->R01`, `15->R02`, `5->R03`
(Group 1), `330->R07`, `40->R08`, `5->R09` (Group 2). XEQ `TSTAT`.
X=`-5.0000` (t), `x<>y`: Y=`0.0011` (p-value).

**Precision note**: p-value has 1e-3 relative precision in the deep tail
(p << 0.01) due to AS 63 backing. See D-35-06.

**TSTAT shortcut**: Group 1 reuses R01-R03 (the v1.x Sigma-block), so
accumulate Group 1 via Sigma+, then STO Group 2 into R07-R09 directly.

## 9. ANOVA (AOVONE, AOVTWO, ANOCOV)

The ANOVA family requires explicit register setup per the OM "Storage
Registers" layout. These are the most complex to verify interactively.

### 9.1 AOVONE -- One-Way Analysis of Variance

XEQ name: `AOVONE` (displayed as `SIGMAAOVONE`). Computes the F-ratio for
comparing means across k groups (1 <= k <= 4).

Register layout:

```
R00 = k (number of groups)
R01 = grand Sigma(x^2)
R02 = grand Sigma(x)
R03 = grand N (total sample count)
Per-group block at base R04 + stride 4*i:
  +0 = Sigma(x_i)     (group sum)
  +1 = Sigma(x_i^2)   (group sum of squares)
  +2 = n_i             (group sample count)
  +3 = scratch
```

Output: X = F-ratio, Y = (df_between, df_within in print buffer)

| # | Groups | X (F) | Notes |
|---|--------|-------|-------|
| 9.1 | [1..5], [6..10], [11..15] | `50.0000` | SSB=250, SSW=30, F=50 (D-35-02) |
| 9.2 | [2,4,6], [3,6,9] | `4.0000` | Two groups |

**Setup for test 9.1**: `Ctrl+G`, STO: `3->R00` (k=3). Group 0 (x=[1..5]):
`15->R04`, `55->R05`, `5->R06`. Group 1 (x=[6..10]): `40->R08`, `330->R09`,
`5->R10`. Group 2 (x=[11..15]): `65->R12`, `855->R13`, `5->R14`. XEQ
`AOVONE`. Display: `50.0000`.

### 9.2 AOVTWO -- Two-Way ANOVA (No Replications)

XEQ name: `AOVTWO`. Uses SIZE 018 (R00-R17): `R00=r, R01=c, R02=N=r*c,
R03=grand Sigma(x^2), R04=grand Sigma(x), R05..=row marginals, then column
marginals`. Best verified via `cargo test -p hp41-core stat1 -- aovtwo`.

### 9.3 ANOCOV -- Analysis of Covariance (One Way)

XEQ name: `ANOCOV`. Uses SIZE 026 (R00-R25): `R00=k, R04=grand Sigma(x^2)`,
per-group stride-5 blocks from R07. Best verified via
`cargo test -p hp41-core stat1 -- anocov`.

## 10. Contingency Tables (CTKKK, CTKK)

### 10.1 CTKKK -- Contingency Table (General r x c)

XEQ name: `CTKKK` (displayed as `SIGMACTKKK`). Computes the chi-square
statistic from an r x c contingency table (max 3x3).

Register layout:

```
R00 = r (number of rows, <= 3)
R01 = c (number of columns, <= 3)
R02..R<2+r*c-1> = cell counts O_ij in row-major order
```

Output: X = chi^2 statistic

| # | Table | X (chi^2) | Notes |
|---|-------|-----------|-------|
| 10.1 | 2x2: [[10,20],[30,40]] | `2.8000` | scipy: chi2_contingency correction=False (D-35-04) |

**Setup for test 10.1**: `Ctrl+G`, STO: `2->R00`, `2->R01`, `10->R02`,
`20->R03`, `30->R04`, `40->R05`. XEQ `CTKKK`.

### 10.2 CTKK -- Contingency Table (2 x 2 Maximum)

XEQ name: `CTKK` (displayed as `SIGMACTKK`). Same layout as CTKKK but
capped at 2x2 (4 cells, R02-R05).

| # | Table | X (chi^2) | Notes |
|---|-------|-----------|-------|
| 10.2 | 2x2: [[10,20],[30,40]] | `0.7937` | D-35-04 corrected oracle |

**Setup for test 10.2**: same register layout as 10.1. XEQ `CTKK`.

## 11. Error Path Summary

Three failure categories produced by Stat 1 Pac ops; each leaves the
calculator state otherwise unchanged.

| Category | Trigger | Display | Examples |
|----------|---------|---------|----------|
| `data error` (Domain) | ln(x<=0) in EXP/LOGI/POW, nu<=0 in CHISQD, p outside (0,1) in NORMD inverse | `data error` | 2.13-2.14, 6.6-6.8 |
| `data error` (InvalidOp) | SIZE-floor violation (not enough registers allocated) | `data error` | Theoretical: shrunk register file |
| `data error` (DivideByZero) | n < 2 in BSTAT (divide by n-1), Sigma(y)=0 in BSTG | `data error` | BSTAT with n=0 or n=1 |

Cross-UI guarantee: the **same** `HpError` variant surfaces as the **same**
`data error` text in both `hp41-cli` (status line) and `hp41-gui` (toast
overlay). The text comes from `hp41-core/src/error.rs` Display impl; both UIs
read it.

## 12. CLI -- GUI Parity

Mirror sections 2 through 10 exactly. GUI-specific notes:

- **XEQ-by-name**: click `XEQ` button, type letters via on-screen keyboard or
  physical A-Z keys, press `ENTER`. Dispatches via `xrom_resolve()`.
- **Modal prompts**: same workflow -- enter numeric value, click `R/S`.
- **Print buffer**: CLI drains to status panel; GUI drains to display area.

**Cross-UI guarantee:** all 26 Stat 1 Pac ops produce **identical** numeric
results between CLI and GUI. Both dispatch through `hp41-core`; mismatches
are an SC-4 violation and release blocker. The GUI E2E smoke test in
`hp41-gui/e2e/smoke.spec.js` covers NORMD Q(1.96) (STAT-QUAL-11).

## Known Limitations

- NORMD CDF: ~1e-5 absolute vs scipy (A&S 6-term; D-35-01).
- TSTAT p-value: 1e-3 relative precision in deep tail (AS 63; D-35-06).
- RAND/SEED: emulator extensions, not part of OM claim (D-35-07).
- Welch's t: excluded; TSTAT is pooled-variance only (D-35-09).
- Distribution primitives: 50-iter bounded, no cancel_requested (D-35-13).
- MMTUG/MMTGD: R07/R08 require manual preloading (Sigma+ only fills R01-R06).

## See Also

- [Stat 1 Pac Divergences](hp41-stat1-divergences.md) / [Function Matrix](hp41-stat1-function-matrix.md) / [JSON Source](hp41-stat1-functions.json)
- ADRs: [v3.1-001 RNG](adr/v3.1-001-rng-state-placement.md) / [v3.1-002 Distributions](adr/v3.1-002-distribution-primitives-policy.md) / [v3.1-003 ANOVA](adr/v3.1-003-anova-register-layout.md) / [v3.1-004 math1 freeze](adr/v3.1-004-math1-freeze-second-carve-out.md) / [v3.1-005 ModalProgram](adr/v3.1-005-modalprogram-stat1-enum-extension.md)
- Companion: [Math Pac I Verification](verifying-math-pac-1.md) / [Card Reader Verification](verifying-card-reader.md)
