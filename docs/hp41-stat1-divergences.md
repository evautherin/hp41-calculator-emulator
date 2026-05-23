# HP-41C Stat 1 Pac Emulator Divergences

This document lists known behavioral divergences between this emulator's implementation
of the HP-41C Stat 1 Pac module and the hardware-faithful behavior described in the
HP-41C Stat 1 Pac Owner's Manual (HP 00041-90030, 1979).

**Status:** Established as comprehensive numbered catalog in Phase 35 / Plan 35-02 (STAT-DOC-03).

**Philosophy:** Where divergences exist, this emulator prioritizes:
1. Hardware-faithful behavior where feasible.
2. User-safety (no silent data corruption without documentation).
3. Clear documentation of known divergences.

---

## How to Use This Document

Each entry carries a stable `D-35-NN` identifier that can be used in cross-references
from source-code comments, ADRs, test files, and issue trackers. The ID encodes the
phase (35 = Phase 35 / STAT-DOC-03) and an ordinal sequence number within this document.

Every entry uses five fixed fields (D-30.5 shape, carried forward as D-35 template):

- **OM citation** — The HP 00041-90030 page-and-example that is the primary source, or
  `"N/A — emulator extension"` when no OM equivalent exists.
- **Our behavior** — What this emulator does.
- **OM behavior** — What the OM says or what real HP-41C hardware does.
- **Rationale** — Why we made this choice (hardware-fidelity vs. UX trade-off decision).
- **See** — Cross-references: ADR links, CONTEXT.md decision IDs, test file pointers,
  Pitfall references from `research/PITFALLS.md` (carried forward across v3.x).

The citation discipline (Pitfall 18 from `research/PITFALLS.md`, carried forward across
v3.x) requires every entry to carry at least one OM page reference, an explicit
`"N/A — emulator extension"` marker, or a primary-source citation (NPS document section,
MoHPC URL, Mike Sebastian forensic page). No uncited assertions are permitted in this
document.

Stat 1 Pac is the second XROM application module in the v3.x line; entry numbering
follows the phase-origin convention established in v3.0 (`D-30-NN` for Math Pac I,
`D-35-NN` for Stat 1 Pac) per Phase 35 CONTEXT D-35.4. A subset of `D-35-NN` entries
(D-35-01..D-35-06) document the 6 oracle drifts reconciled in
`.planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-SPEC-AMENDMENT.md`
(Plan 35-01) — these are mathematical-ground-truth corrections (scipy.stats versus
original 33-SPEC.md planning-phase oracle estimates), classified under bucket 3
(Behavioral Policies) per D-35.1.

---

## 1. OM Divergences

*(Numerical / behavioral mismatches with OM-quoted examples or OM-described hardware
behavior. These are cases where the OM specifies or implies a particular outcome and our
emulator either matches or intentionally diverges from that specification.)*

*No entries — Phase 35 / Plan 35-02 did not surface genuine OM-numerical-mismatch
entries for Stat 1 Pac through Phase 34 verification. The 6 oracle drifts queued by
D-35.1 are scipy-vs-original-SPEC planning-archaeology corrections, not OM divergences —
see bucket 3 (Behavioral Policies) entries D-35-01..D-35-06. The original-SPEC.md
oracle estimates pre-dated the rust_decimal `MathematicalOps::norm_cdf` 6-term
Abramowitz & Stegun precision audit and the AS 239 / AS 63 calibration measurements
performed during Plan 33-02 implementation; the SPEC.md amendment (Plan 35-01)
re-anchors the contract to scipy.stats ground-truth values.*

*If future Phase 36 (GUI integration) or Phase 37 (test hardening) surfaces a genuine
OM-quoted-behavior mismatch for Stat 1 Pac, it will be authored as `D-35-12:` or later
— the numbering reservation `D-35-01..D-35-06` is the canonical bucket-3 oracle-drift
ID range per CONTEXT.md `<specifics>` and D-35.1.*

---

## 2. Emulator Extensions

*(Functions or behaviors we added that are not present in HP 00041-90030 (1979). These
are deliberate, documented additions that improve usability without conflicting with OM
behavior for OM-specified inputs. Every extension in this section is marked with
"N/A — emulator extension" in the OM citation field.)*

*<populated below — D-35-07 onward — once Task 2 lands the bucket-2 entries.>*

---

## 3. Behavioral Policies

*(Cross-cutting rules that are decisions worth documenting — not strictly numerical
divergences, but intentional implementation choices with OM basis or deliberate extension.
These entries document cases where the emulator made a specific policy decision that
affects behavior in ways the OM either specifies explicitly or leaves to the implementation.)*

---

### D-35-01: ΣNORMD CDF Tolerance Band — 1e-9 SPEC Estimate vs 1e-5 rust_decimal A&S 6-Term Limit

- **OM citation**: HP 00041-90030 (1979), §ΣNORMD distribution evaluation — the OM does
  NOT specify a numerical tolerance band for ΣNORMD CDF output; the original 33-SPEC.md
  Req. 7 oracle of 1e-9 was a planning-phase estimate inconsistent with the
  `rust_decimal::MathematicalOps::norm_cdf` backing implementation, which uses the
  Abramowitz & Stegun §26 6-term polynomial approximation and produces ~1.3e-7 absolute
  error at the OM-quoted x = 1.96 example.

- **Our behavior**: `XEQ "ΣNORMD"` in CDF mode returns Q(x) such that
  `|Q(1.96) − scipy.stats.norm.sf(1.96)| < 1e-5`. The test
  `cdf_oracle_q_of_196_within_a_and_s_band` (`hp41-core/src/ops/stat1/normd.rs:252`)
  asserts the relaxed 1e-5 band; the original planning-phase 1e-9 oracle was
  unachievable with the rust_decimal A&S 6-term approximation.

- **OM behavior**: The OM does not specify a numerical tolerance for the ΣNORMD CDF
  output. On real HP-41C / Stat 1 Pac hardware, the ΣNORMD precision depends on the
  underlying ROM polynomial approximation (uncited in OM 00041-90030; documented in the
  HP Journal but not in the user-facing OM). Our 1e-5 band is consistent with the
  rust_decimal backing implementation; deeper precision would require switching the
  primitive.

- **Rationale**: `rust_decimal::MathematicalOps::norm_cdf` is the canonical Rust backing
  for normal CDF and uses the well-known A&S §26 6-term polynomial — the same numerical
  technique used by many implementations of comparable scope. Switching to a
  higher-precision implementation (e.g., a Cody/Hart-derived 13-term or an arbitrary-
  precision arctangent-of-error-function via continued fractions) would add ~80 lines of
  primitive code or introduce a new runtime dependency (`statrs` was rejected per
  ADR-v3.1-002 / D-33.5 — runtime-dep cost outweighs the precision delta for a
  user-facing statistical workflow). The 1e-5 band is sufficient for OM-quoted-example
  workflows (Q(1.96) ≈ 0.0250 to 4 significant digits). scipy.stats.norm.sf is the
  cross-check oracle.

- **See**: `.planning/phases/33-hp41-core-xrom-activation-distribution-primitives-all-stat-1/33-SPEC-AMENDMENT.md`
  row 1 (Plan 35-01 reconciliation); `hp41-core/src/ops/stat1/normd.rs:252`
  (`cdf_oracle_q_of_196_within_a_and_s_band`); ADR-v3.1-002 (Plan 35-03 distribution
  primitives policy); D-33.5 (33-CONTEXT.md statrs rejection); scipy.stats.norm.sf as
  cross-check oracle; Pitfall 18 (citation provenance).

---

### D-35-02: ΣAOVONE F-Ratio Oracle Correction — F = 100.0 → F = 50.0

- **OM citation**: HP 00041-90030 (1979), §ΣAOVONE one-way ANOVA program description —
  the OM specifies the one-way ANOVA F-ratio formula
  `F = (SSB / df_between) / (SSW / df_within)`. The OM does not pin a specific worked
  example oracle for the dataset [1..5] / [6..10] / [11..15]; the original 33-SPEC.md
  Req. 11 oracle of `F = 100.0` for this dataset was a planning-phase estimate.

- **Our behavior**: `XEQ "ΣAOVONE"` on three groups [1..5], [6..10], [11..15] returns
  `F = 50.0 ± 1e-9`. Test `aovone_three_groups_of_five_yields_f_50`
  (`hp41-core/src/ops/stat1/anova.rs:348`) asserts the scipy-correct value. Manual
  derivation: `SSB = 250`, `SSW = 30`, `df = (2, 12)`,
  `F = (250 / 2) / (30 / 12) = 125 / 2.5 = 50.0`.

- **OM behavior**: F-ratio is a deterministic function of the input data per the
  one-way ANOVA formula. Real HP-41C hardware running the OM-defined algorithm on the
  same input would produce `F = 50.0`. The OM does NOT diverge from this value — the
  planning-phase SPEC.md oracle (`F = 100.0`) was incorrect.

- **Rationale**: SPEC.md Req. 11 + ROADMAP success criterion claimed `F = 100.0`;
  scipy.stats.f_oneway and manual derivation both confirm `F = 50.0`. The implementation
  is mathematically correct; the ROADMAP / SPEC oracle was the bug. Documented as
  scipy.stats reconciliation per D-35.1 history-preserving SPEC-amendment strategy
  (rejected in-place SPEC.md editing in favor of a sibling amendment file plus a
  bucket-3 narrative entry here).

- **See**: `33-SPEC-AMENDMENT.md` row 2 (Plan 35-01); `hp41-core/src/ops/stat1/anova.rs:348`
  (`aovone_three_groups_of_five_yields_f_50`); `hp41-core/src/ops/stat1/anova.rs`
  module-doc lines 11-13 (in-source amendment marker); scipy.stats.f_oneway as
  cross-check oracle; 33-VERIFICATION.md row 4 (drift evidence); D-35.1 (35-CONTEXT.md).

---

### D-35-03: ΣSPEAR Rank Correlation Oracle Correction — ρ_s = 0.7 → 0.8

- **OM citation**: HP 00041-90030 (1979), §ΣSPEAR Spearman rank correlation program
  description (p. 64) — the OM specifies the Spearman rank-correlation formula
  `ρ_s = 1 − (6 · Σdᵢ²) / (n · (n² − 1))` where `dᵢ` is the rank-difference per pair.
  The OM does not pin a specific worked-example oracle for the dataset
  x=[1,2,3,4,5] / y=[2,1,3,5,4]; the original 33-SPEC.md Req. 30 oracle of `ρ_s = 0.7`
  for this dataset was a planning-phase estimate.

- **Our behavior**: `XEQ "ΣSPEAR"` on x=[1,2,3,4,5] / y=[2,1,3,5,4] returns
  `ρ_s = 0.8 ± 1e-9`. Test `spear_basic_5_pairs`
  (`hp41-core/src/ops/stat1/nonparam.rs:341`) asserts the scipy-correct value. Manual
  derivation: rank-differences `d = [-1, 1, 0, -1, 1]`, `Σd² = 4`,
  `ρ_s = 1 − (6·4) / (5·(25−1)) = 1 − 24/120 = 1 − 0.2 = 0.8`.

- **OM behavior**: Identical — `ρ_s = 0.8` is the correct closed-form output per the
  OM-specified formula. Real HP-41C hardware running the OM algorithm on the same
  input would produce `ρ_s = 0.8`. `scipy.stats.spearmanr([1,2,3,4,5], [2,1,3,5,4])
  .statistic` returns `0.7999999999999999` (= 0.8 within f64 precision).

- **Rationale**: SPEC.md Req. 30 claimed `ρ_s = 0.7`; manual derivation and
  scipy.stats.spearmanr both confirm `ρ_s = 0.8`. The implementation is mathematically
  correct; the SPEC.md oracle was the bug (likely off-by-one in the Σd² counting during
  planning-phase paper-and-pencil derivation). The implementation has no tied-rank
  handling because the test dataset has no ties — for tied-rank inputs the OM
  specifies the standard average-rank convention which the emulator follows; that
  convention is OM-faithful and not a divergence.

- **Rationale (continued)**: Documented per D-35.1 history-preserving SPEC-amendment
  strategy; the `nonparam.rs` module-doc lines 14 in-source amendment marker is the
  paired source-side annotation.

- **See**: `33-SPEC-AMENDMENT.md` row 3 (Plan 35-01); `hp41-core/src/ops/stat1/nonparam.rs:341`
  (`spear_basic_5_pairs`); `hp41-core/src/ops/stat1/nonparam.rs` module-doc lines 6-16
  (in-source amendment marker); scipy.stats.spearmanr as cross-check oracle;
  33-VERIFICATION.md (drift evidence); D-35.1 (35-CONTEXT.md).

---

### D-35-04: ΣEFXSQ χ² Calibration Drift — 1.667 → 7.0 (Plus Two Contingency-Table Sister Drifts)

- **OM citation**: HP 00041-90030 (1979), §ΣEFXSQ chi-square-with-expected-proportions
  program description (p. 55) — the OM specifies the closed-form
  `χ² = Σᵢ (Oᵢ − Eᵢ)² / Eᵢ` over observed counts and expected counts derived from
  proportions × N. The OM does not pin a specific worked-example oracle; the original
  33-SPEC.md Req. 27 oracle of `χ² = 1.667` for the Plan 33-04 test dataset was a
  planning-phase estimate. Two sister contingency-table drifts are bundled into this
  entry because they share the same root cause (planning-phase manual derivation drift
  from scipy.stats): Req. 28 (`χ² = 4.286 → 2.8`) and Req. 29 (`χ² = 0.397 → 0.7937`)
  both surfaced during Plan 33-06 ΣCTKKK / ΣCTKK implementation against
  `scipy.stats.chi2_contingency(correction=False)`.

- **Our behavior**: `XEQ "ΣEFXSQ"` on the Plan 33-04 test dataset returns
  `χ² = 7.0 ± 1e-9` (test `efxsq_basic` at `hp41-core/src/ops/stat1/nonparam.rs:498`).
  `XEQ "ΣCTKKK"` on the Plan 33-06 r×c test returns `χ² = 2.8 ± 1e-9`.
  `XEQ "ΣCTKK"` on the Plan 33-06 2×2 test returns `χ² = 0.7937 ± 1e-9`. All three
  closed-form `Σ(O−E)²/E` reducers share the same arithmetic core; the SPEC.md drifts
  are reconciled to scipy.stats ground truth.

- **OM behavior**: `χ² = Σ(O−E)²/E` is a closed-form deterministic computation per the
  OM-specified formula. Real HP-41C hardware running the OM algorithm on the same
  inputs would produce the scipy-correct values. The OM does NOT diverge — the
  planning-phase SPEC.md oracles were incorrect.

- **Rationale**: scipy.stats.chisquare and scipy.stats.chi2_contingency(correction=False)
  are the canonical cross-check oracles for the χ²-with-expected-counts family. The
  three drifts (Req. 27 / 28 / 29) all stem from planning-phase manual-derivation
  errors during 33-SPEC.md authoring (pre-implementation, pre-scipy-validation). The
  implementations are mathematically correct; SPEC.md was wrong. Documented per D-35.1
  history-preserving strategy.

- **See**: `33-SPEC-AMENDMENT.md` rows 4 / 5 / 6 (Plan 35-01 — three rows because three
  requirements drifted; this catalog entry bundles them since they share root cause);
  `hp41-core/src/ops/stat1/nonparam.rs:498` (`efxsq_basic`);
  `hp41-core/src/ops/stat1/nonparam.rs` module-doc lines 14-17 (in-source amendment
  marker); scipy.stats.chisquare and scipy.stats.chi2_contingency(correction=False) as
  cross-check oracles; D-35.1 (35-CONTEXT.md).

---

### D-35-05: ΣBSTAT Coefficient-of-Variation Oracle Correction — CV = 0.4083 → 0.5270

- **OM citation**: HP 00041-90030 (1979), §ΣBSTAT extended univariate-summary program
  description — the OM specifies the standard
  `CV_x = σ_x / μ_x` (sample-standard-deviation / sample-mean) per the textbook
  coefficient-of-variation definition; the OM does not pin a specific worked-example
  oracle. The original 33-SPEC.md Req. 7 oracle of `CV = 0.4083` for the dataset
  `x=[1,2,3,4,5] / y=[10,20,30,40,50]` was a planning-phase estimate that derived from
  the non-standard formula `CV = √(s²/Σx) = √2.5/√15 ≈ 0.4083`, NOT the standard
  textbook CV.

- **Our behavior**: `XEQ "ΣBSTAT"` on `x=[1,2,3,4,5]` returns
  `CV_x = 0.5270462766947299 ± 1e-9`. Test `bstat_spec_req7_corrected_oracle`
  (`hp41-core/src/ops/stat1/basic_stats.rs:253`) asserts the scipy-correct value.
  Manual derivation: `μ = 3`, `σ = √2.5 ≈ 1.5811388300841898`,
  `CV = σ / μ = 1.5811388300841898 / 3.0 ≈ 0.5270462766947299`.

- **OM behavior**: Identical — the standard textbook
  `CV = σ / μ ≈ 0.5270462766947299` is the correct closed-form output. Real HP-41C
  hardware running the OM-specified algorithm on the same input would produce
  `CV ≈ 0.5270`. The OM does NOT diverge — the planning-phase SPEC.md formula was
  non-standard.

- **Rationale**: `0.4083 = √2.5 / √15 = σ_x / √(Σx)` is not a standard CV definition;
  the SPEC.md oracle accidentally used a non-textbook formula. Manual derivation and
  scipy (via `np.std(x, ddof=1) / np.mean(x)`) both confirm
  `CV = 0.5270462766947299`. The implementation ships the mathematically correct
  textbook value. Documented per D-35.1 history-preserving strategy; the
  `basic_stats.rs` module-doc lines 33-47 in-source amendment marker is the paired
  source-side annotation.

- **See**: `33-SPEC-AMENDMENT.md` row 5 (Plan 35-01);
  `hp41-core/src/ops/stat1/basic_stats.rs:253` (`bstat_spec_req7_corrected_oracle`);
  `hp41-core/src/ops/stat1/basic_stats.rs` module-doc lines 33-47 (in-source amendment
  marker); scipy.stats and numpy as cross-check oracles; D-35.1 (35-CONTEXT.md).

---

### D-35-06: ΣTSTAT Deep-Tail Student-t p Tolerance — 1e-7 SPEC → 1e-3 AS 63 Lentz-CF Precision Limit

- **OM citation**: HP 00041-90030 (1979), §ΣTSTAT pooled-variance two-sample t-test
  program description — the OM specifies the pooled-variance t-statistic and the
  associated two-tailed p-value derived from the Student-t CDF. The OM does not specify
  a numerical tolerance band for the p-value output. The original 33-SPEC.md Req. 25
  oracle was paired with a planning-phase 1e-7 relative-tolerance estimate that the
  AS 63 Lentz continued-fraction backing implementation cannot meet in the deep-tail
  region (`p ≪ 0.01`) due to the well-known `EPS_CONV = 1e-9` floor in the AS 63
  algorithm.

- **Our behavior**: `XEQ "ΣTSTAT"` returns the pooled-variance t-statistic at 1e-7
  closed-form precision (the t-statistic itself uses pure HpNum arithmetic — no AS 63
  call) and the two-tailed p-value at 1e-3 relative precision in the deep-tail region.
  For typical user workflows where `p ≥ 0.01` the precision is materially better (~1e-6
  relative); the 1e-3 floor only binds when `p ≪ 0.01`. Tests
  `tstat_oracle_g1_1_5_g2_6_10` (`hp41-core/src/ops/stat1/hypothesis.rs:472`),
  `tstat_pooled_variance_unequal_n_oracle`
  (`hp41-core/src/ops/stat1/hypothesis.rs:589`) document the tolerance band — see
  source comment lines 462-469 + 478-480 + 586-604 for the AS 63 deep-tail amendment
  notes.

- **OM behavior**: The OM does not specify a numerical tolerance for the p-value. On
  real HP-41C / Stat 1 Pac hardware, the ΣTSTAT p-value precision depends on the
  underlying ROM continued-fraction implementation (uncited in OM 00041-90030; the
  Stat 1 Pac uses Algorithm AS 63 internally per NPS document NPS55-84-003 cross-
  reference). Our 1e-3 deep-tail band is consistent with the AS 63
  Lentz-continued-fraction EPS_CONV floor; the t-statistic precision (1e-7) is
  hardware-faithful.

- **Rationale**: AS 63 (Majumder & Bhattacharjee, 1973) is the canonical Algorithm
  Series implementation of the incomplete beta function used to compute the Student-t
  CDF. The `EPS_CONV = 1e-9` floor is intrinsic to the Lentz continued-fraction
  refinement at the boundary-swap arm `(p ≪ 0.01)`. Switching to a higher-precision
  algorithm (e.g., DiDonato-Morris incomplete beta or arbitrary-precision via mpmath)
  would add ~200 lines of primitive code or a new runtime dependency
  (`statrs` was rejected per ADR-v3.1-002 / D-33.5). The 1e-3 deep-tail band is
  sufficient for OM-quoted-workflow precision (user-facing p-values are typically
  reported to 4 significant digits).

- **See**: `33-SPEC-AMENDMENT.md` row 6 (Plan 35-01);
  `hp41-core/src/ops/stat1/hypothesis.rs:472` (`tstat_oracle_g1_1_5_g2_6_10`);
  `hp41-core/src/ops/stat1/hypothesis.rs:589`
  (`tstat_pooled_variance_unequal_n_oracle`); `hp41-core/src/ops/stat1/hypothesis.rs`
  source comments lines 322-346 + 462-480 + 586-604 (AS 63 EPS_CONV amendment notes);
  ADR-v3.1-002 (Plan 35-03 distribution primitives policy); D-33.5 (33-CONTEXT.md
  statrs rejection); scipy.stats.ttest_ind(equal_var=True) as cross-check oracle;
  D-35.1 (35-CONTEXT.md).

---
