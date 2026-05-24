# HP-41C Stat 1 Pac Function Matrix

> Generated from `docs/hp41-stat1-functions.json` via `just docs-matrix`.
> Edit the JSON, regenerate this file, commit both.

## Implemented (v2.x)

| Op | Display | XROM | Category | Status | Phase | Key Path | Description |
|----|---------|------|----------|--------|-------|----------|-------------|
| SigmaAnocov | ΣANOCOV | Stat 1 / 2-7 | Stat1 ANOVA | ✓ v2.x | 33 | `XEQ "ΣANOCOV"` | Analysis of covariance (single covariate) |
| SigmaAovone | ΣAOVONE | Stat 1 / 2-5 | Stat1 ANOVA | ✓ v2.x | 33 | `XEQ "ΣAOVONE"` | One-way ANOVA: F-ratio across multiple groups |
| SigmaAovtwo | ΣAOVTWO | Stat 1 / 2-6 | Stat1 ANOVA | ✓ v2.x | 33 | `XEQ "ΣAOVTWO"` | Two-way ANOVA: two-factor analysis of variance |
| SigmaChisqdWorkflow | ΣCHISQD | Stat 1 / 2-24 | Stat1 Distributions | ✓ v2.x | 33 | `XEQ "ΣCHISQD"` | Chi-square distribution CDF / PDF (prompts for ν) |
| SigmaNormdWorkflow | ΣNORMD | Stat 1 / 2-23 | Stat1 Distributions | ✓ v2.x | 33 | `XEQ "ΣNORMD"` | Standard normal CDF / PDF / inverse (probit) — three-mode dispatcher |
| SigmaPtst | ΣPTST | Stat 1 / 2-16 | Stat1 Hypothesis | ✓ v2.x | 33 | `XEQ "ΣPTST"` | Paired-sample t-test (within-subject differences) |
| SigmaTstat | ΣTSTAT | Stat 1 / 2-17 | Stat1 Hypothesis | ✓ v2.x | 33 | `XEQ "ΣTSTAT"` | Two-sample t-test (pooled-variance assumption) |
| SigmaCtkk | ΣCTKK | Stat 1 / 2-21 | Stat1 Nonparam | ✓ v2.x | 33 | `XEQ "ΣCTKK"` | Contingency table — Kendall's tau-b (corrected) |
| SigmaCtkkk | ΣCTKKK | Stat 1 / 2-20 | Stat1 Nonparam | ✓ v2.x | 33 | `XEQ "ΣCTKKK"` | Contingency table — Kendall's tau-c statistic |
| SigmaEfxsq | ΣEFXSQ | Stat 1 / 2-19 | Stat1 Nonparam | ✓ v2.x | 33 | `XEQ "ΣEFXSQ"` | Chi-square with expected-frequency input |
| SigmaSpear | ΣSPEAR | Stat 1 / 2-22 | Stat1 Nonparam | ✓ v2.x | 33 | `XEQ "ΣSPEAR"` | Spearman rank correlation coefficient |
| SigmaXsqev | ΣXSQEV | Stat 1 / 2-18 | Stat1 Nonparam | ✓ v2.x | 33 | `XEQ "ΣXSQEV"` | Chi-square goodness-of-fit evaluation |
| Rand | RAND | Stat 1 / 2-25 | Stat1 RNG | ✓ v2.x | 33 | `XEQ "RAND"` | Next pseudorandom uniform on [0,1) via Don Malm LCG |
| Seed | SEED | Stat 1 / 2-26 | Stat1 RNG | ✓ v2.x | 33 | `XEQ "SEED"` | Seed the RAND generator from X (modal prompt 'SEED?') |
| SigmaExp | ΣEXP | Stat 1 / 2-9 | Stat1 Regression | ✓ v2.x | 33 | `XEQ "ΣEXP"` | Exponential curve fit: y = a·e^(b·x) |
| SigmaLin | ΣLIN | Stat 1 / 2-8 | Stat1 Regression | ✓ v2.x | 33 | `XEQ "ΣLIN"` | Linear curve fit: y = a + b·x |
| SigmaLogi | ΣLOGI | Stat 1 / 2-10 | Stat1 Regression | ✓ v2.x | 33 | `XEQ "ΣLOGI"` | Logarithmic curve fit: y = a + b·ln(x) |
| SigmaMlrxy | ΣMLRXY | Stat 1 / 2-12 | Stat1 Regression | ✓ v2.x | 33 | `XEQ "ΣMLRXY"` | Multiple linear regression — two predictors (Y on X) |
| SigmaMlrxyz | ΣMLRXYZ | Stat 1 / 2-13 | Stat1 Regression | ✓ v2.x | 33 | `XEQ "ΣMLRXYZ"` | Multiple linear regression — three predictors (Y on X,Z) |
| SigmaPolyc | ΣPOLYC | Stat 1 / 2-15 | Stat1 Regression | ✓ v2.x | 33 | `XEQ "ΣPOLYC"` | Polynomial coefficient recall after ΣPOLYP fit |
| SigmaPolypWorkflow | ΣPOLYP | Stat 1 / 2-14 | Stat1 Regression | ✓ v2.x | 33 | `XEQ "ΣPOLYP"` | Polynomial regression — accepts degree via modal prompt |
| SigmaPow | ΣPOW | Stat 1 / 2-11 | Stat1 Regression | ✓ v2.x | 33 | `XEQ "ΣPOW"` | Power curve fit: y = a·x^b |
| SigmaBstat | ΣBSTAT | Stat 1 / 2-1 | Stat1 Univariate | ✓ v2.x | 33 | `XEQ "ΣBSTAT"` | Extended univariate summary (weighted mean, CV) |
| SigmaBstg | ΣBSTG | Stat 1 / 2-2 | Stat1 Univariate | ✓ v2.x | 33 | `XEQ "ΣBSTG"` | Grouped-data univariate summary |
| SigmaMmtgd | ΣMMTGD | Stat 1 / 2-4 | Stat1 Univariate | ✓ v2.x | 33 | `XEQ "ΣMMTGD"` | Moments — grouped data (mean, variance, skew, kurtosis) |
| SigmaMmtug | ΣMMTUG | Stat 1 / 2-3 | Stat1 Univariate | ✓ v2.x | 33 | `XEQ "ΣMMTUG"` | Moments — ungrouped data (mean, variance, skew, kurtosis) |

## v3.x Deferred (Module Pacs)

_None._
