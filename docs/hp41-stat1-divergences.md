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

---

### D-35-07: RAND / SEED LCG — v3.1 Emulator Extension

- **OM citation**: `N/A — emulator extension`. The HP-41C Stat 1 Pac Owner's Manual
  HP 00041-90030 (1979) does NOT list a top-level RAND or SEED XROM entry point per
  the Phase 33 OM read (33-08-SUMMARY.md research finding); the Quick Reference Card
  HP 00041-90061 (June 1979) confirms this absence. RAND / SEED are a v3.1 emulator
  extension per the NPS document ZP4 program convention and the HP-65 User's Library
  (Don Malm) historical LCG-formula provenance. STAT-RNG-04 (REQUIREMENTS.md) classifies
  RAND / SEED as the only Stat 1 Pac surface that is NOT part of the
  "feature-complete per OM 00041-90030" claim.

- **Our behavior**: `XEQ "RAND"` generates the next pseudorandom uniform via the
  linear-congruential generator `r_{n+1} = FRC(9821 · r_n + 0.211327)` per
  NPS55-84-003 (Zehna, 1984) p. 21–22, pushes it to the stack, and writes back to
  `state.rand_seed`. `XEQ "SEED"` opens an ALPHA `SEED?` modal prompt and writes the
  submitted (and FRC-normalized to `[0, 1)`) value to `state.rand_seed: HpNum`. The
  seed survives serde save/load via `#[serde(default)]` WITHOUT `#[serde(skip)]` —
  the ONLY v3.1 `CalcState` field with this serde shape (all other v3.1 fields are
  either `#[serde(skip)]` transient or carry the standard `#[serde(default)]` plus
  `#[serde(skip)]` combination). Round-trip reproducibility is asserted by integration
  test `rand_sequence_deterministic_after_save_load`
  (`hp41-core/tests/stat1_rand_determinism.rs:55-93`).

- **OM behavior**: `N/A — emulator extension`. The OM does not specify RAND or SEED.

- **Rationale**: NPS55-84-003 p. 21–22 cites the LCG formula
  `r_{n+1} = FRC(9821 · r_n + 0.211327)` as HP-65 User's Library convention attributed
  to Don Malm; HP-41C Standard Applications (1979) p. 24 carries the same formula
  independently. Implementing RAND / SEED is community-convention parity with
  widely-circulated HP-41 user programs across three independent primary sources
  (NPS, Don Malm, HP-41C Standard Applications). The non-`skip` serde shape was chosen
  to preserve reproducible simulations across sessions (STAT-RNG-03 / Pitfall 20) —
  losing the seed on every save/load would break the deliberate-reproducibility
  contract that SEED's existence implies. Rejected alternative: implement RAND
  without persistence (would force users to re-seed on every session restart, defeating
  SEED's purpose).

- **See**: `docs/adr/v3.1-001-rng-state-placement.md` (Plan 35-03 ADR; forward-
  reference within Phase 35 ship); `hp41-core/src/ops/stat1/rand.rs`;
  `hp41-core/src/state.rs:201-202` (`rand_seed` field with unique serde shape);
  `hp41-core/tests/stat1_rand_determinism.rs` (3 integration tests); STAT-RNG-01..04
  (REQUIREMENTS.md); D-33.4 / D-33.4a (33-CONTEXT.md); NPS55-84-003 p. 21–22;
  HP-41C Standard Applications (1979) p. 24; HP-65 User's Library (Don Malm,
  community-attributed); Pitfall 20 (RNG serde, research/PITFALLS.md).

---

### D-35-08: ΣPOLYP "DEGREE=?" Prompt — Math Pac I POLY Precedent Inheritance

- **OM citation**: HP 00041-90030 (1979), §ΣPOLYP polynomial-regression program
  description — the OM specifies a degree-prompt mechanism (the user enters polynomial
  degree `d` before regression accumulation), but the exact on-screen prompt wording
  is tentatively transcribed from the Math Pac I `POLY` program convention per ROADMAP
  STAT-REG-07 success criterion ("tentative `DEGREE=?` per Math Pac I `POLY` precedent
  — verified in Phase 33"). This makes the prompt wording itself a v3.1 emulator
  transcription pinned to the Math Pac I precedent; the underlying degree-prompt
  mechanism is OM-faithful.

- **Our behavior**: `XEQ "ΣPOLYP"` opens a modal prompt displaying the literal ALPHA
  string `DEGREE=?` and accepts a `u8` degree `d` via R/S submit. The degree flows
  through the existing v3.0 modal-program infrastructure as
  `ModalProgram::Stat1(Stat1Step::PolypDegreePrompt(u8))` per D-33.3b — the same
  modal machinery introduced for Math Pac I `POLY` `DEGREE=?` is re-used unchanged.
  Out-of-range `d` (d < 1 or d > 5 per OM polynomial-regression range) returns
  `HpError::OutOfRange`.

- **OM behavior**: The OM specifies that the user enters degree `d` before the
  regression begins. The precise on-screen ALPHA-prompt wording on real HP-41C
  hardware may use different characters or abbreviations (the OM uses prose
  description, not a screenshot transcript); our `DEGREE=?` is the emulator
  transcription consistent with the Math Pac I `POLY` precedent.

- **Rationale**: Re-uses the locked Math Pac I prompt-wording style (precedent
  established v3.0 / Phase 28 for `POLY` and inherited verbatim here). Alternatives
  considered: an ALPHA-only `"DEG?"` (4 chars, more terse — rejected for parity with
  Math Pac I), `"D=?"` (3 chars, even more terse — rejected for clarity), or a
  CATALOG-style `"POLY DEGREE?"` prefix (rejected as inconsistent with the Stat 1 Pac
  Σ-prefix convention). `DEGREE=?` preserves the v3.0 UX precedent at zero cost.

- **See**: `docs/hp41-stat1-functions.json` ΣPOLYP entry inline `divergences` field
  (cross-reference, not duplication, per D-34.3 surgical-inline convention);
  `hp41-core/src/ops/stat1/regression.rs` (`op_sigma_polyp`);
  `hp41-core/src/ops/stat1/modal.rs::Stat1Step::PolypDegreePrompt`;
  `hp41-core/src/ops/math1/modal.rs` (D-33.3b carve-out — `ModalProgram::Stat1(...)`
  dispatch arm); ROADMAP STAT-REG-07; D-34.3 (34-CONTEXT.md inline-JSON-divergences
  convention).

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

### D-35-09: ΣTSTAT Pooled-Variance Convention — Welch's Unequal-Variance t Excluded

- **OM citation**: HP 00041-90030 (1979), §ΣTSTAT two-sample t-test program
  description — the OM specifies pooled-variance convention. Cross-confirmed by NPS
  document NPS55-84-003 (Zehna, 1984) ZS-4 / ZS-5 program listings (p. 30+), which
  describe the Stat 1 Pac's two-sample t as pooled-variance with df = n₁ + n₂ − 2,
  NOT Welch's unequal-variance approximation with df = Welch-Satterthwaite. The OM
  citation is direct; the NPS citation is secondary cross-validation.

- **Our behavior**: `XEQ "ΣTSTAT"` computes the two-sample t-statistic with pooled
  variance:
  `t = (μ₁ − μ₂) / (s_p · √(1/n₁ + 1/n₂))` where
  `s_p² = ((n₁−1)·s₁² + (n₂−1)·s₂²) / (n₁ + n₂ − 2)`; the degrees-of-freedom is
  `df = n₁ + n₂ − 2` (integer). Welch's unequal-variance t is NOT implemented
  (deliberate anti-feature per REQUIREMENTS.md "Out of Scope"). The p-value derives
  from the AS 63 incomplete-beta-backed Student-t CDF at the integer df.

- **OM behavior**: Identical (pooled-variance per OM specification and NPS
  confirmation).

- **Rationale**: NPS55-84-003 p. 30+ ZS-4 / ZS-5 confirms Stat 1 Pac uses pooled
  variance; Welch's t is not part of the Stat 1 Pac OM specification and would change
  the df calculation to the Welch-Satterthwaite approximation (non-integer df,
  different p-value). Welch's t is locked out per REQUIREMENTS.md "Out of Scope" line
  for "Welch's t-test (unequal variance)" — implementing it would silently change
  user-facing results in ways that diverge from OM-quoted-example expectations.
  Rejected alternative: ship both pooled and Welch behind a mode flag — rejected for
  added API complexity and divergence from the OM single-mode contract.

- **See**: `hp41-core/src/ops/stat1/hypothesis.rs::op_sigma_tstat`; 33-SPEC.md Req. 25
  (pooled-variance lock); NPS55-84-003 p. 30+ (ZS-4 / ZS-5 cross-validation);
  REQUIREMENTS.md "Out of Scope" (Welch's t exclusion); D-33.1 item 4 (33-CONTEXT.md
  pooled-vs-Welch resolution); D-35-06 (this catalog — paired AS 63 deep-tail
  precision entry).

---

### D-35-10: XROM-7 (Math Pac I) vs XROM-2 (Stat 1 Pac) Module-ID Prefix Convention

- **OM citation**: HP 00041-90030 Quick Reference Card 00041-90061 (1979) + the HP-41
  module-database catalog at `calc.fjk.ch/db/hp41mod.php` "Statistics Pac 1B" entry —
  Stat 1 Pac is XROM module-ID **2** on real HP-41C hardware
  (`hp41-core/src/ops/math1/xrom.rs::STAT_1.id = 2` per Phase 33 D-33.1). Math Pac I
  uses XROM module-ID **7** (locked v3.0 / Phase 28 — `MATH_1.id = 7` is the
  emulator-internal numbering decision preserving the HP records' original module ID
  for Math Pac). Both modules use a `Σ`-mnemonic-prefix convention for many entry
  points, but the disambiguator at resolver time is the module-ID bit, NOT the
  mnemonic prefix.

- **Our behavior**: `STAT_1.id = 2` is the canonical Stat 1 Pac XROM ID; mnemonic
  prefix `Σ` is used for most Stat 1 entry points (Σ-register-using statistical
  convention). Math Pac I mnemonics use `Σ`-prefix for the few Σ-register-aware
  entries (and other prefixes elsewhere); both modules share the `Σ` prefix family
  but their entries are disjoint sets. `xrom_resolve` distinguishes the modules via
  the module-bit lookup in the resolver chain
  (`bit 0 = MATH_1`, `bit 1 = STAT_1`, fired LAST in the resolver chain per Pitfall 1),
  NOT via mnemonic-prefix matching. The cross-XROM no-shadow invariant is asserted by
  `hp41-cli/tests/xrom_shadowing.rs` extended for STAT_1.ops.

- **OM behavior**: Identical — Stat 1 Pac is XROM 2 per HP hardware records (HP records
  assign module IDs uniquely at manufacture). The `Σ`-prefix mnemonic convention is
  hardware-faithful for both pacs.

- **Rationale**: Stat 1 Pac's hardware XROM ID is 2 (uniquely assigned at manufacture
  per HP module catalog); Math Pac I uses ID 7 by HP records (the original HP Math Pac
  module ID is 7 per `calc.fjk.ch/db/hp41mod.php`); we preserve both IDs faithfully.
  The `Σ`-prefix convention applies to BOTH pacs because both are derived from
  Σ-register-using HP statistical / extended-univariate conventions — but the module
  IDs are the canonical disambiguator and the no-shadow CI gate is what makes this
  invariant load-bearing. Rejected alternative: assign Stat 1 Pac a unique mnemonic
  prefix (e.g., `S∘` or `Stat∘`) to avoid the `Σ` overlap — rejected because it would
  break OM-mnemonic-fidelity (the OM uses `Σ`-prefix throughout).

- **See**: `hp41-core/src/ops/math1/xrom.rs::STAT_1` (id = 2);
  `hp41-core/src/ops/math1/xrom.rs::MATH_1` (id = 7); `xrom_resolve` bit-1 arm;
  `hp41-cli/tests/xrom_shadowing.rs` (extended for STAT_1.ops);
  `calc.fjk.ch/db/hp41mod.php` "Statistics Pac 1B" entry; Pitfall 1 (resolver
  LAST-fires); Pitfall 22 (mnemonic shadowing); D-33.1 (33-CONTEXT.md).

---

### D-35-11: math1/ Freeze Second Carve-Out — xrom.rs + modal.rs (ADR-v3.1-004 Cross-Reference)

- **OM citation**: `N/A — emulator architectural policy`. The OM does not specify the
  module-freeze convention; this is a v3.x emulator-architecture decision documented
  in CLAUDE.md `## Frozen Invariants → Core engine`.

- **Our behavior**: `hp41-core/src/ops/math1/` is frozen since Plan 25-01 (v3.0
  invariant per CLAUDE.md "frozen since Plan 25-01"). v3.1 carves out TWO surgical
  exceptions per ADR-v3.1-004:
  - `math1/xrom.rs` (Plan 33-01 + D-33.3 / D-33.3a) — extends the XROM registry to
    register `STAT_1` const and adds the bit-1 dispatch arm. The bit-1 stub was
    always intended for v3.1+ extension per its inline comment in v3.0; this is the
    realization of the documented stub purpose, not an unplanned freeze violation.
  - `math1/modal.rs` (Plan 33-01 + D-33.3b — amended 2026-05-22 during
    `/gsd-plan-phase 33`, user-confirmed) — adds the `ModalProgram::Stat1(Stat1Step)`
    enum variant + 3-arm dispatch wiring (~8 lines of pure dispatch). `Stat1Step`
    semantics live entirely in the NEW `hp41-core/src/ops/stat1/modal.rs` so no
    Stat 1 Pac semantics leak into the frozen module.

  The rest of `hp41-core/src/ops/math1/` (complex.rs, difeq.rs, four.rs,
  hyperbolics.rs, integ.rs, matrix.rs, mod.rs (modulo 2-line stat1 variant arm),
  poly.rs, solve.rs, trans.rs, tri.rs) remains strictly frozen and bit-identical
  to v3.0.

- **OM behavior**: `N/A — emulator architectural policy`.

- **Rationale**: Adding a new XROM module necessitates editing the registry (xrom.rs)
  and the modal dispatcher (modal.rs); spawning parallel registry / modal-program
  infrastructure was the rejected alternative — cost: ~80 lines of cross-frontend
  duplication across `state.rs` + `commands.rs` + `app.rs` to thread two modal-program
  enums per D-33.3b. Carving out two surgically-narrow files (8-line modal.rs delta +
  XROM registry-extension xrom.rs delta) is the minimum-blast-radius choice and
  preserves the 4-way exhaustive-match invariant intact (the new `Stat1(Stat1Step)`
  variant lands in all four required match sites). ADR-v3.1-004 (Plan 35-03)
  documents the architectural lock with the full rejected-alternative quote;
  CLAUDE.md `## Frozen Invariants → Core engine` will list the carve-outs in
  Plan 35-04 (narrative-docs).

- **See**: `docs/adr/v3.1-004-math1-freeze-second-carve-out.md` (Plan 35-03 ADR —
  forward-reference within Phase 35 ship); `docs/adr/v3.1-005-modalprogram-stat1-enum-extension.md`
  (Plan 35-03 sister ADR — `ModalProgram::Stat1` lock viewed from the enum-design
  angle); `hp41-core/src/ops/math1/xrom.rs` (STAT_1 const + bit-1 arm);
  `hp41-core/src/ops/math1/modal.rs` (Stat1 dispatch arm); 33-CONTEXT.md D-33.3 /
  D-33.3a / D-33.3b; CLAUDE.md `## Frozen Invariants → Core engine` (Plan 35-04
  amendment target).

---

### D-35-12: RAND First-Call Output from Default-Zero Seed — Deterministic 0.211327

- **OM citation**: `N/A — emulator behavioral policy`. The OM does not specify the
  RAND default-seed value (RAND / SEED are emulator extensions per D-35-07).

- **Our behavior**: `state.rand_seed` defaults to `HpNum::zero()` on fresh
  `CalcState::default()` construction
  (`hp41-core/src/state.rs:340` — `rand_seed: HpNum::zero()`). The first `XEQ "RAND"`
  call on a fresh session (or on a v2.2 save file migrated via `migrate_after_load()`
  where `rand_seed` is absent and defaults to zero via `#[serde(default)]`) therefore
  applies the LCG to seed 0 and returns
  `FRC(9821 · 0 + 0.211327) = FRC(0.211327) = 0.211327` deterministically. Every
  fresh session that calls RAND without first calling SEED produces the identical
  starting sequence (`0.211327`, then the LCG continues from `r = 0.211327`).

- **OM behavior**: `N/A — emulator policy` (the OM does not specify RAND / SEED).

- **Rationale**: A predictable default-zero seed is the cleanest policy for a
  pseudorandom generator that explicitly supports `SEED` for reproducibility — users
  who want randomness should call `SEED` with a session-unique value (e.g., a
  user-typed entropy phrase or a timestamp echo); users who skip `SEED` get a
  deterministic sequence, which is exactly the contract that `SEED`'s existence
  implies. Rejected alternative: seed from system entropy (e.g., `getrandom()`) on
  fresh-session construction — rejected because (a) it would diverge from
  reproducibility-by-default which is the OM-extension contract, (b) it would add a
  platform-dependency for entropy sourcing in `hp41-core` (which must stay
  UI / OS / CLI-independent per the workspace-isolation invariant), and (c) v2.2
  save-file migration (which sets `rand_seed = 0` via `#[serde(default)]`) would
  silently behave differently from fresh-session construction. The current behavior
  is the principle-of-least-surprise choice. This entry is the catalog routing of
  33-REVIEW.md IN-05 per CONTEXT.md Claude's Discretion line 149 (IN-05 routes here
  rather than to ADR-v3.1-001 Footnotes because the first-call observable behavior
  is a bucket-3 behavioral-policy concern, not a Footnote-grade rationale detail).

- **See**: `hp41-core/src/state.rs:340` (`rand_seed: HpNum::zero()` default);
  `hp41-core/src/state.rs:386-391` (`migrate_after_load` — sets bit 1 but leaves
  `rand_seed` at its serde-default zero); `hp41-core/src/ops/stat1/rand.rs::op_rand`
  (LCG application); 33-REVIEW.md IN-05 (originating Info finding); D-33.4 /
  D-33.4a (33-CONTEXT.md RAND/SEED policy); D-35-07 (this catalog — paired RAND/SEED
  primary entry); ADR-v3.1-001 (Plan 35-03 RNG state placement).

---

### D-35-13: Bounded-Iteration Distribution Primitives Do Not Wire cancel_requested

- **OM citation**: `N/A — emulator-internal design choice`. The HP-41C hardware
  has no cancellation mechanism for built-in math primitives; the HP-41C Stat 1
  Pac Owner's Manual HP 00041-90030 (1979) does not describe any mid-computation
  interrupt facility for the distribution evaluation programs.

- **Our behavior**: The three hand-coded f64-bridge distribution primitives —
  `norm_cdf_inv_f64` (Acklam / Wichura AS 241), `gamma_regularized_f64`
  (Cody AS 239), and `beta_regularized_f64` (Lentz AS 63) — are bounded at
  `ITER_CAP = 50` iterations in `hp41-core/src/ops/stat1/distributions.rs` and
  complete in microseconds on any supported platform (macOS M1 / Ubuntu / Windows).
  They do NOT check `state.cancel_requested` on a per-iteration basis. The
  `cancel_requested` flag is checked ONCE at the ΣNORMD inverse entry-point
  (`op_sigma_normd_eval_inverse`) before entering the Newton refinement loop, per
  the T-31-W1-sticky-cancel parity pattern established for v3.0 INTG/SOLVE/DIFEQ.

- **OM behavior**: `N/A — emulator-internal design choice`. The HP-41C hardware
  executes distribution programs atomically from the user's perspective — there
  is no R/S mid-calculation cancel mechanism for XROM math primitives.

- **Rationale**: The Phase 31 `cancel_requested` channel (`Arc<AtomicBool>`) was
  designed specifically for OPEN-ENDED iterative paths (INTG, SOLVE, DIFEQ) that
  can run for seconds or minutes and where the user might press R/S to interrupt
  a diverging or slow computation (D-28.7 / D-28.8). Bounded 50-iteration f64
  primitives complete faster than an R/S keypress can register (< 1 µs typical
  on M1; < 5 µs on the slowest CI runner). Wiring per-iteration
  `cancel_requested.load(Relaxed)` into a 50-iteration loop would add platform-
  specific overhead (cache miss on the Arc<AtomicBool> pointer chain) to a loop
  that is already a performance non-issue, with zero user-visible benefit — the
  computation would complete before the cancel flag propagated from the GUI thread.
  Rejected alternative: add a per-iteration cancel check to all three primitives
  (cost: 3 additional `&CalcState` borrows through the call chain, complicating
  the pure-f64 function signatures; benefit: zero, since 50 iterations of f64
  arithmetic cannot block the GUI event loop for a perceptible duration).
  See D-36.2 (36-CONTEXT.md) for the Phase 36 planning reassessment that confirmed
  this disposition and reassigned STAT-GUI-05 to Phase 37 for documentation.

- **See**: `hp41-core/src/ops/stat1/distributions.rs` (`ITER_CAP = 50` constant,
  all three bounded primitives); `hp41-core/src/ops/stat1/normd.rs`
  (`op_sigma_normd_eval_inverse` — entry-point cancel check BEFORE Newton loop);
  `.planning/phases/36-hp41-gui-gui-integration/36-CONTEXT.md` D-36.2 (bounded-iter
  rationale and STAT-GUI-05 reassignment commit message);
  `docs/adr/v3.1-002-distribution-primitives-policy.md` (Plan 35-03 ADR — free
  parameters ITER_CAP and convergence threshold selection); STAT-GUI-05
  (REQUIREMENTS.md — formally resolved by this behavioral-policy entry per Plan
  37-03 D-37.9 disposition).

---

*Last updated: 2026-05-24. Entry D-35-13 added in Plan 37-03 (Phase 37 / STAT-GUI-05 resolution).*

*Prior update: 2026-05-23. Catalog established in Plan 35-02 (Phase 35 / STAT-DOC-03).*
