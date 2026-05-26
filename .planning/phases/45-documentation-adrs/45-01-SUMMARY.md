---
phase: 45-documentation-adrs
plan: "01"
subsystem: docs
tags: [documentation, adr, advantage-pac, v3.3, divergences]
dependency_graph:
  requires: [43-hp41-core-xrom-framework-all-advantage-pac-ops, 44-hp41-cli-integration]
  provides: [v3.3-divergences-catalog, v3.3-adrs]
  affects: [45-02-PLAN, 46-gui-integration, 47-test-hardening]
tech_stack:
  added: []
  patterns: [D-30.5-five-field-shape, D-30.6-adr-long-form, D-30.7-verbatim-context-quotes, D-45-NN-identifiers]
key_files:
  created:
    - docs/hp41-advantage-divergences.md
    - docs/adr/v3.3-001-named-matrix-storage-model.md
    - docs/adr/v3.3-002-froot-laguerre-algorithm.md
    - docs/adr/v3.3-003-dual-xrom-id-design.md
    - docs/adr/v3.3-004-math1-visibility-promotion-policy.md
  modified: []
decisions:
  - "D-45-NN identifier convention established for Advantage Pac divergence catalog"
  - "Three-bucket divergence catalog: 0 OM Divergences, 2 Emulator Extensions, 7 Behavioral Policies"
  - "ADR-v3.3-001 locked: Vec<AdvMatrix> named-matrix storage model with complete Math Pac I isolation"
  - "ADR-v3.3-002 locked: Laguerre's method with (0.4,0.9) initial guess and Free42 disclaim"
  - "ADR-v3.3-003 locked: dual XROM ID design (ADV_MATH_A=22, ADV_MATH_B=24) with 12 MATH_1 alias overlaps"
  - "ADR-v3.3-004 locked: complex_atan2 pub(crate) as third sanctioned math1/ freeze carve-out"
metrics:
  duration_secs: 488
  completed_date: "2026-05-26"
  tasks_completed: 2
  tasks_total: 2
  files_created: 5
  files_modified: 0
---

# Phase 45 Plan 01: Advantage Pac Divergence Catalog + Four v3.3 ADRs Summary

Three-bucket divergence catalog (9 entries, D-45-NN) and four long-form ADRs capturing v3.3 architectural decisions for named-matrix storage, FROOT algorithm, dual-XROM-ID design, and math1/ visibility promotion.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Create divergence catalog | cb75105 | `docs/hp41-advantage-divergences.md` |
| 2 | Create four v3.3 ADRs | c2c3844 | `docs/adr/v3.3-001..004-*.md` |

## What Was Built

### Task 1: Advantage Pac Divergence Catalog

`docs/hp41-advantage-divergences.md` — three-bucket D-30.5 five-field catalog with `D-45-NN` identifiers:

**Bucket 1 — OM Divergences:** None identified in Phase 43-44 implementation.

**Bucket 2 — Emulator Extensions (2 entries):**
- D-45-01: Unlimited named-matrix count (Vec<AdvMatrix> extension, hardware X-MEM slot limits not enforced)
- D-45-02: 255×255 per-dimension matrix size cap (u8 type constraint)

**Bucket 3 — Behavioral Policies (7 entries):**
- D-45-03: Named-matrix vs. Math Pac I register model isolation (D-43.5 mandate, ADR-v3.3-001)
- D-45-04: FROOT Laguerre vs. Math Pac I Bairstow coexistence (different mnemonics, different degree ranges)
- D-45-05: TVM register persistence (#[serde(default)] without skip, D-43.11, ADR-v3.1-001 precedent)
- D-45-06: adv_current_matrix transient (#[serde(skip)], matches hardware ALPHA register volatility)
- D-45-07: ADV_WORD_MASK 36-bit truncation for bitwise ops (D-43.9/D-43.10, silent truncation)
- D-45-08: Solver cross-nesting allowance (D-43.7, one level cross-nesting; self-nesting blocked)
- D-45-09: MATH_1 alias overlap in ADV_MATH_B (12 mnemonics, hardware-faithful resolver priority)

### Task 2: Four v3.3 ADRs

Each follows the D-30.6 long-form template from `docs/adr/v3.2-001-clock-access-pattern.md` with Context, Decision, Consequences (Positive/Negative/Neutral), Alternatives Considered (D-30.7 verbatim CONTEXT.md quotes), Footnotes/References, and footer.

**ADR-v3.3-001** (`named-matrix-storage-model.md`): Vec<AdvMatrix> with #[serde(default)]; global I/J indices; adv_current_matrix transient; complete isolation from Math Pac I matrix_dim/matrix_active_reg (D-43.5). Alternatives: HashMap (non-deterministic serde), register-based (incompatible, D-43.5), X-MEM file model (XMEM-01 deferred), per-matrix I/J (not OM-faithful).

**ADR-v3.3-002** (`froot-laguerre-algorithm.md`): Laguerre's method with (0.4, 0.9) initial guess (origin singularity discovered during implementation — 43-07-SUMMARY.md), quadratic deflation, root polishing, f64 intermediate arithmetic. Free42 disclaim verbatim per ADR-v3.1-002. Alternatives: Bairstow (degree-limited), Jenkins-Traub (~300 LOC), companion matrix eigenvalue (dense LA not needed), Durand-Kerner (higher memory).

**ADR-v3.3-003** (`dual-xrom-id-design.md`): ADV_MATH_A (id=22, bit-3, 63 ops) + ADV_MATH_B (id=24, bit-4, 51 ops); hardware-faithful dual ROM chip design; 12 MATH_1 alias overlaps (E^Z, LNZ, LOGZ, Z^N, Z^1/N, Z^W, |Z|, SINZ, COSZ, TANZ, A^Z, CINV); xrom_shadowing uses 0b0001_0000 isolation (not 0b0001_1111) for ADV_MATH_B tests. Alternatives: single XROM ID (hardware inaccurate), XROM 22+23 (wrong IDs), flattened namespace (resolver isolation impossible).

**ADR-v3.3-004** (`math1-visibility-promotion-policy.md`): complex_atan2 pub(super) → pub(crate) in math1/complex.rs; third sanctioned carve-out (xrom.rs + modal.rs + complex.rs); complete list of five sanctioned files after v3.3. Alternatives: re-implement in advantage/ (duplicates frozen logic), shared utility module (over-engineering), pub(super) wrapper (unnecessary indirection).

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. These are documentation files with no data sources or UI rendering.

## Threat Flags

No new security-relevant surface introduced — documentation-only plan per threat model.

## Self-Check: PASSED

Files exist:
- FOUND: docs/hp41-advantage-divergences.md
- FOUND: docs/adr/v3.3-001-named-matrix-storage-model.md
- FOUND: docs/adr/v3.3-002-froot-laguerre-algorithm.md
- FOUND: docs/adr/v3.3-003-dual-xrom-id-design.md
- FOUND: docs/adr/v3.3-004-math1-visibility-promotion-policy.md

Commits verified:
- cb75105: docs(45-01): create Advantage Pac divergence catalog
- c2c3844: docs(45-01): create four v3.3 architectural decision records

Verification criteria met:
- docs/hp41-advantage-divergences.md has 3 section headers (OM Divergences, Emulator Extensions, Behavioral Policies): YES
- docs/hp41-advantage-divergences.md has 10 D-45-0 references (>= 9 required): YES
- All four ADRs have Context, Decision, Consequences, Alternatives Considered: YES (4/4)
- ADR-v3.3-002 contains "independently re-derived from primary literature": YES
- ADR-v3.3-003 documents all 12 overlapping mnemonics: YES
- D-45-03 See field cross-references ADR-v3.3-001: YES
- D-45-04 See field cross-references ADR-v3.3-002: YES
