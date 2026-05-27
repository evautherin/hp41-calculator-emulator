# Phase 45: Documentation & ADRs — Pattern Map

**Mapped:** 2026-05-26
**Files analyzed:** 5 new/modified files
**Analogs found:** 5 / 5

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `docs/hp41-advantage-divergences.md` | doc — divergence catalog | transform (source decisions → structured entries) | `docs/hp41-time-divergences.md` | exact |
| `docs/adr/v3.3-001-named-matrix-storage-model.md` | doc — ADR | request-response (decision → long-form narrative) | `docs/adr/v3.2-001-clock-access-pattern.md` | exact |
| `docs/adr/v3.3-002-froot-laguerre-algorithm.md` | doc — ADR | request-response | `docs/adr/v3.1-001-rng-state-placement.md` | exact |
| `docs/adr/v3.3-003-dual-xrom-id-design.md` | doc — ADR | request-response | `docs/adr/v3.2-001-clock-access-pattern.md` | exact |
| `docs/adr/v3.3-004-math1-visibility-promotion-policy.md` | doc — ADR | request-response | `docs/adr/v3.1-004-math1-freeze-second-carve-out.md` | exact |
| `docs/architecture-history.md` (v3.3 section append) | doc — narrative | transform | `docs/architecture-history.md` lines 277–348 (v3.2 section) | exact |
| `CLAUDE.md` (v3.3 additions block append) | meta-doc | transform | `CLAUDE.md` `### v3.2 additions` block (lines 193–251) | exact |
| `README.md` (v3.3 soft-claim bullet append) | meta-doc | transform | `README.md` lines 55–58 (v3.1/v3.2 bullets) | exact |

---

## Pattern Assignments

### `docs/hp41-advantage-divergences.md` (divergence catalog, transform)

**Analog:** `docs/hp41-time-divergences.md`

**File header pattern** (`docs/hp41-time-divergences.md` lines 1–43):
```markdown
# HP-41C Time Pac Emulator Divergences

This document lists known behavioral divergences between this emulator's implementation
of the HP-41C Time Module and the hardware-faithful behavior described in the
HP Time Module Owner's Manual (HP 00041-90035, 1982).

**Status:** Established as comprehensive numbered catalog in Phase 40 / Plan 40-01 (TIME-DOC-02).

**Philosophy:** Where divergences exist, this emulator prioritizes:
1. Hardware-faithful behavior where feasible.
2. User-safety (no silent data corruption without documentation).
3. Clear documentation of known divergences.

---

## How to Use This Document

Each entry carries a stable `D-40-NN` identifier [...]
```

For v3.3: substitute module name "Advantage Pac", OM reference "HP 00041-90482", status "Plan 45-01 (ADV-DOC-02)", identifier prefix `D-45-NN`, phase "45".

**Five-field entry shape** (`docs/hp41-time-divergences.md` lines 22–31):
```markdown
- **OM citation** — The HP 00041-90035 page-and-example that is the primary source, or
  `"N/A — emulator extension"` when no OM equivalent exists.
- **Our behavior** — What this emulator does.
- **OM behavior** — What the OM says or what real HP-41CX Time Module hardware does.
- **Rationale** — Why we made this choice (hardware-fidelity vs. UX trade-off decision).
- **See** — Cross-references: ADR links, CONTEXT.md decision IDs, test file pointers,
  Pitfall references from `research/PITFALLS.md` (carried forward across v3.x).
```

**Three-bucket section headers** (`docs/hp41-time-divergences.md` lines 44–48, 65–70, 119–124):
```markdown
## 1. OM Divergences

*(Numerical / behavioral mismatches with OM-quoted examples or OM-described hardware
behavior. These are cases where the OM specifies or implies a particular outcome and our
emulator either matches or intentionally diverges from that specification.)*

## 2. Emulator Extensions

*(Functions or behaviors we added that are not present in HP 00041-90035 (1982). These
are deliberate, documented additions that improve usability without conflicting with OM
behavior for OM-specified inputs. Every extension in this section is marked with
"N/A — emulator extension" in the OM citation field.)*

## 3. Behavioral Policies

*(Cross-cutting rules that are decisions worth documenting — not strictly numerical
divergences, but intentional implementation choices with OM basis or deliberate extension.)*
```

**Concrete entry example** (`docs/hp41-time-divergences.md` lines 128–169 — D-40-01 entry):
```markdown
### D-40-01: CORRECT / SETAF Accuracy Factor — Documented No-Op

- **OM citation**: HP 00041-90035 (1982), §CORRECT and §SETAF — [...]

- **Our behavior**: [...]

- **OM behavior**: [...]

- **Rationale**: [...] Rejected alternative: [...] — rejected because [...]

- **See**: `hp41-core/src/ops/time/clock.rs::op_correct` [...]; D-38.1 (38-CONTEXT.md [...]).
```

**Footer line** (`docs/hp41-time-divergences.md` line 355):
```markdown
*Last updated: 2026-05-25. Catalog established in Plan 40-01 (Phase 40 / TIME-DOC-02).*
```

**Content map for Phase 45 entries:**

| Entry ID | Bucket | Topic | Decision source |
|----------|--------|-------|-----------------|
| D-45-01 | 2 (Emulator Extension) | Unlimited named-matrix count (`Vec<AdvMatrix>`) | D-43.1 |
| D-45-02 | 2 (Emulator Extension) | 255×255 per-dimension matrix size cap (`u8` type) | D-43.2 |
| D-45-03 | 3 (Behavioral Policy) | Named-matrix vs. Math Pac I register model isolation | D-43.5 / ADR-v3.3-001 |
| D-45-04 | 3 (Behavioral Policy) | FROOT Laguerre vs. Math Pac I Bairstow coexistence | D-43.8 / ADR-v3.3-002 |
| D-45-05 | 3 (Behavioral Policy) | TVM register persistence (`#[serde(default)]` without skip) | D-43.11 |
| D-45-06 | 3 (Behavioral Policy) | `adv_current_matrix` transient — clears on save | D-43.3 |
| D-45-07 | 3 (Behavioral Policy) | ADV_WORD_MASK 36-bit truncation for bitwise ops | D-43.9, D-43.10 |
| D-45-08 | 3 (Behavioral Policy) | Solver cross-nesting allowance (FINTG inside FSOLVE) | D-43.7 |
| D-45-09 | 3 (Behavioral Policy) | MATH_1 alias overlap in ADV_MATH_B (12 mnemonics) | ADR-v3.3-003 / Phase 44 discovery |

---

### `docs/adr/v3.3-001-named-matrix-storage-model.md` (ADR, request-response)

**Analog:** `docs/adr/v3.2-001-clock-access-pattern.md`

**ADR frontmatter pattern** (`docs/adr/v3.2-001-clock-access-pattern.md` lines 1–9):
```markdown
# ADR-v3.2-001: Clock Access Pattern -- Direct SystemTime in hp41-core

**Status:** Locked 2026-05-24
**Owner:** Plan 38-03
**Requirement refs:** TIME-CLK-01, TIME-CLK-02, TIME-FW-04, Pitfall 34 (clock in core)
**Downstream consumer:** `hp41-core/src/ops/time/clock.rs` [...];
`hp41-core/src/state.rs` (`time_offset_secs: i64`)
**ADR write-up prose:** Phase 40 / TIME-DOC-03
```

For ADR-v3.3-001:
- **Status:** Locked 2026-05-25
- **Owner:** Plan 43-01
- **Requirement refs:** ADV-FW-04, ADV-MTX-01
- **Downstream consumer:** `hp41-core/src/state.rs` (`adv_matrices: Vec<AdvMatrix>`, `adv_matrix_i`, `adv_matrix_j`); `hp41-core/src/ops/advantage/matrix.rs`
- **ADR write-up prose:** Phase 45 / Plan 45-01

**Section structure** (from `docs/adr/v3.2-001-clock-access-pattern.md`):
```markdown
---

## Context
[Background — problem statement, why the decision was non-obvious, relevant constraints]

## Decision
[The chosen approach — concrete, with code or field names]

## Consequences
### Positive
### Negative
### Neutral

## Alternatives Considered
### Option B: [Alternative title]
[Quote from CONTEXT.md verbatim per D-30.7, then rejection rationale]

### Option C: [...]
### Option D: [...]

## Footnotes / References
[^1]: [...]

---

*ADR-v3.2-001 locked: 2026-05-24. Plan 38-03.*
*ADR write-up: Phase 40 / TIME-DOC-03.*
*OM reference: HP Time Module Owner's Manual HP 82182A, 1982.*
```

**Alternatives to document for ADR-v3.3-001** (source: RESEARCH.md lines 170–175):
- Option B: `HashMap<String, AdvMatrix>` — non-deterministic serialization order
- Option C: Register-based storage (R14/R15+ like Math Pac I) — D-43.5 incompatibility
- Option D: File-based X-MEM model (EMDIR/EMROOM) — post-v3.3 scope
- Option E: Per-matrix I/J indices — OM treats I/J as calculator-global pointers

**CONTEXT.md verbatim quote to embed (D-30.7 discipline):**
```
"Named-matrix storage MUST NOT touch `state.matrix_dim` or `state.matrix_active_reg`.
Complete isolation between the two matrix systems."
```

---

### `docs/adr/v3.3-002-froot-laguerre-algorithm.md` (ADR, request-response)

**Analog:** `docs/adr/v3.1-001-rng-state-placement.md` (algorithm provenance ADR with Free42 disclaim)

**Unique requirement:** Must include the Free42 disclaim verbatim per ADR-v3.1-002 pattern:
```
"Algorithm independently re-derived from primary literature (Numerical Recipes §9.5
Laguerre's method); Free42 source consulted only as sanity-check oracle, not copied."
```

**Context section must cover:**
- Laguerre's method with initial guess (0.4, 0.9) — NOT (0, 0) — because (0, 0) causes convergence failure for purely imaginary roots (x²+1 = 0)
- Quadratic deflation for complex conjugate pairs; linear deflation for real roots
- f64 arithmetic during iteration; convert to HpNum only at final root output
- Degree read from X register (D-43.8 resolution — NOT a modal prompt)

**Alternatives to document** (source: RESEARCH.md lines 181–186):
- Option A: Bairstow's method (Math Pac I ROOTS) — degree-limited (2–5); FROOT needs arbitrary degree up to 100
- Option B: Jenkins-Traub — more robust but ~300 LOC; Laguerre sufficient for HP-41 use cases
- Option C: Companion matrix eigenvalue — requires dense linear algebra not otherwise in hp41-core
- Option D: Durand-Kerner (Weierstrass) — simultaneous root finding, higher memory; deflation is simpler

---

### `docs/adr/v3.3-003-dual-xrom-id-design.md` (ADR, request-response)

**Analog:** `docs/adr/v3.2-001-clock-access-pattern.md` (constraint-driven architecture decision)

**Context section must cover:**
- HP Advantage Pac ships on two ROM chips: XROM 22 and XROM 24 are hardware IDs, not an emulator design choice
- ADV_MATH_A (id=22, bit-3): ADV CONV + ADV MTRX (63 ops)
- ADV_MATH_B (id=24, bit-4): ADV MATH + ADV TVM (51 ops)
- 12 intentional MATH_1 mnemonic overlaps in ADV_MATH_B: E^Z, LNZ, LOGZ, Z^N, Z^1/N, Z^W, |Z|, SINZ, COSZ, TANZ, A^Z, CINV — MATH_1 wins (bit-0 fires first in resolver chain)
- `xrom_modules = 0b0001_1111` — five bits for five XROM modules

**Alternatives to document** (source: RESEARCH.md lines 197–202):
- Option A: Single XROM ID with unified 114-entry module — violates hardware accuracy (real hardware uses two distinct IDs)
- Option B: XROM 22 + XROM 23 — hardware uses 22 + 24 (HP's own module assignment)
- Option C: Flattened op namespace in one module — 12 intentional MATH_1 overlaps require independent resolver isolation

---

### `docs/adr/v3.3-004-math1-visibility-promotion-policy.md` (ADR, request-response)

**Analog:** `docs/adr/v3.1-004-math1-freeze-second-carve-out.md`

**Context section must cover:**
- `complex_atan2` in `hp41-core/src/ops/math1/complex.rs` promoted from `pub(super)` to `pub(crate)` for Phase 43
- math1/ freeze boundary per ADR-v3.1-004 — this is the THIRD sanctioned carve-out (after xrom.rs and modal.rs)
- Complete list of five math1/ sanctioned carve-outs after v3.3:
  1. `xrom.rs` (ADR-v3.1-004, D-33.3) — XROM registry, bits 0-4
  2. `modal.rs` (ADR-v3.1-004, D-33.3b) — ModalProgram dispatch, Advantage variant added Phase 43
  3. `complex.rs` — `complex_atan2` visibility `pub(super)` → `pub(crate)` (Phase 43)
- All other math1/ files remain frozen

**Alternatives to document** (source: RESEARCH.md lines 208–213):
- Option A: Re-implement `complex_atan2` in `advantage/complex_ext.rs` — duplicates logic the freeze protects
- Option B: Move to shared utility module — over-engineering; `pub(crate)` is minimal surgical change
- Option C: Keep `pub(super)`, add wrapper in math1/mod.rs — unnecessary indirection

---

### `docs/architecture-history.md` — v3.3 section append (narrative, transform)

**Analog:** `docs/architecture-history.md` lines 277–348 (v3.2 section)

**Section heading pattern** (line 277):
```markdown
## v3.2 additions (Time Pac Emulation, Phases 38–42)
```

For v3.3:
```markdown
## v3.3 additions (Advantage Pac Emulation, Phases 43–47)
```

**Intro paragraph pattern** (`docs/architecture-history.md` lines 279–280):
```markdown
Phases 38–42 ship the third XROM application module — the HP-41CX Time Module
(HP 82182A, HP part number 00041-90036, Owner's Manual 1982) — as a behavioral
emulation of 35 XEQ-by-name entry points across clock/date/stopwatch/alarm families.
[...] Everything else follows the v3.0/v3.1 invariants (XROM resolver chain, modal-workflow
infrastructure, JSON-canonical pipeline, save-file backward compat, zero new runtime deps).
```

**Per-phase subsection pattern** (`docs/architecture-history.md` lines 281–295, Phase 38):
```markdown
### Phase 38 — XROM Framework + Clock/Date/Stopwatch/Alarm Core (shipped 2026-05-24)

The XROM framework extends from a two-arm cascade (MATH_1 bit-0, STAT_1 bit-1) to a
three-arm cascade landing the Time Module at hardware-accurate XROM ID 26...

[ADR decision coverage]

[CalcState field decisions]

[Op variant count + 4-way invariant status]
```

**Frozen invariants table** (`docs/architecture-history.md` lines 339–347):
```markdown
**Frozen invariants preserved across v3.2:**

- SC-4 invariant: [...]
- 4-exhaustive-match invariant: items 1+2 complete (Phase 38); item 3 complete (Phase 39); item 4 complete (Phase 41). [...]
- `#![deny(clippy::unwrap_used)]` continues in `hp41-core`; test files carry `#[allow]` per established pattern.
- Save-file backward compat: [...]
- MSRV 1.88 unchanged. [...]
- Free42 GPL contamination guard: [...]
```

**Phase 45/46/47 stub pattern** (`docs/architecture-history.md` lines 311–314):
```markdown
### Phase 40 — Documentation & ADRs (shipped 2026-05-25)

Phase 40 authors the Time Pac documentation suite: [...], the README v3.2 soft-claim bullet, and this
`docs/architecture-history.md` v3.2 narrative section. The v3.2 hard-claim [...] is deferred to Phase 42 [...]
```

Phase 46 and 47 write stub lines only (same pattern as when Phase 40 wrote the v3.2 section before Phases 41–42 shipped).

---

### `CLAUDE.md` — v3.3 additions block append (meta-doc, transform)

**Analog:** `CLAUDE.md` lines 193–251 (`### v3.2 additions` block)

**Block header pattern** (`CLAUDE.md` lines 193–195):
```markdown
### v3.2 additions (Time Pac Emulation, Phases 38–42)

*Origin: see `docs/architecture-history.md` §v3.2 additions for the long-form per-phase narrative;
this block is the CLAUDE.md decision-summary surface (the SECOND `### v3.x additions` block; the
first was v3.1 per D-35.5).*
```

For v3.3:
```markdown
### v3.3 additions (Advantage Pac Emulation, Phases 43–47)

*Origin: see `docs/architecture-history.md` §v3.3 additions for the long-form per-phase narrative;
this block is the CLAUDE.md decision-summary surface (the THIRD `### v3.x additions` block; the
second was v3.2 per D-40.9).*
```

**Per-phase subsection pattern** (`CLAUDE.md` lines 197–211 — Phase 38):
```markdown
#### Phase 38 — XROM Framework + Clock/Date/Stopwatch/Alarm Core (shipped 2026-05-24)

- **TIME_MODULE (XROM ID 26) registered (D-carried.5):** [bullet decision summary]
- **`default_xrom_modules() = 0b0000_0111` + `migrate_after_load()` (D-carried.5):** [...]
- **Phase 38 CalcState additions with serde shapes (D-40.10):** [...] `time_offset_secs: i64`
  (`#[serde(default)]`), [...] `stopwatch_start: Option<Instant>` (`#[serde(default, skip)]` — transient), [...]
```

**CalcState field table for Phase 43** (source: RESEARCH.md lines 299–310):

| Field | Serde shape | Notes |
|-------|-------------|-------|
| `adv_matrices: Vec<AdvMatrix>` | `#[serde(default)]` | Persistent; D-43.1; D-43.5 isolation |
| `adv_matrix_i: u8` | `#[serde(default)]` | Persistent 1-based row index; D-43.4 |
| `adv_matrix_j: u8` | `#[serde(default)]` | Persistent 1-based col index; D-43.4 |
| `adv_tvm_state: Option<TvmState>` | `#[serde(default)]` WITHOUT skip | Unique per D-43.11 / ADR-v3.1-001 pattern |
| `adv_current_matrix: Option<String>` | `#[serde(default, skip)]` | Transient; clears on save |
| `adv_froot_state: Option<FrootState>` | `#[serde(default, skip)]` | Transient; D-43.7 |
| `adv_fintg_state: Option<AdvFintegState>` | `#[serde(default, skip)]` | Transient; D-43.7 |
| `adv_fsolve_state: Option<AdvFsolveState>` | `#[serde(default, skip)]` | Transient; D-43.7 |
| `adv_fdifeq_state: Option<AdvFdifeqState>` | `#[serde(default, skip)]` | Transient; D-43.7 |

**Phase 46/47 stubs:** Write as `#### Phase 46 — GUI Integration (in progress — Phase 46 planned)` and `#### Phase 47 — Test Hardening & Quality Gates (Phase 47 planned)` per the D-40.9 convention (same stub pattern as v3.2 block wrote when Phases 41–42 weren't yet shipped).

---

### `README.md` — v3.3 soft-claim bullet append (meta-doc, transform)

**Analog:** `README.md` lines 55–58

**Exact v3.1 and v3.2 bullet patterns** (`README.md` lines 55–58):
```markdown
- v3.1 ships Stat 1 Pac behavioral emulation, feature-complete per Owner's Manual HP 00041-90030 (13 programs, 26 XEQ entry points,
  RAND/SEED extension, [documented divergences](docs/hp41-stat1-divergences.md)) — see [Stat 1 Pac Function Matrix](docs/hp41-stat1-function-matrix.md)
- v3.2 ships Time Pac behavioral emulation, feature-complete per Owner's Manual 00041-90035 (35 XEQ entry points, real-time clock/stopwatch/alarm backed by the host system clock,
  [documented divergences](docs/hp41-time-divergences.md)) — see [Time Pac Function Matrix](docs/hp41-time-function-matrix.md)
```

**v3.3 soft-claim line** (from RESEARCH.md lines 275–280):
```markdown
- v3.3 ships Advantage Pac behavioral emulation (~117 XEQ entry points across base conversion,
  named-matrix operations, advanced math/solvers/complex/curve-fit, and time-value-of-money;
  dual XROM IDs 22 + 24; [documented divergences](docs/hp41-advantage-divergences.md)) —
  see [Advantage Pac Function Matrix](docs/hp41-advantage-function-matrix.md)
```

**CRITICAL — do NOT use hard-claim language.** "feature-complete per Owner's Manual 00041-90482" is deferred to Phase 47 per the D-30.9 → D-32.5 → D-35.3 → D-37.11 → D-42.11 graduation cadence.

---

## Shared Patterns

### D-30.5 Five-Field Entry Shape
**Source:** `docs/hp41-time-divergences.md` lines 22–31
**Apply to:** `docs/hp41-advantage-divergences.md` — all entries in all three buckets
```markdown
- **OM citation** — HP 00041-90482 page reference, or `"N/A — emulator extension"`, or primary-source citation
- **Our behavior** — What this emulator does
- **OM behavior** — What the OM specifies or hardware does
- **Rationale** — Why we made this choice; rejected alternative with "Rejected alternative: ... — rejected because ..."
- **See** — `hp41-core/src/ops/advantage/...`; D-43.NN (CONTEXT.md); ADR-v3.3-NNN; test file pointers
```

### D-30.6 ADR Long-Form Template
**Source:** `docs/adr/v3.2-001-clock-access-pattern.md`
**Apply to:** All four v3.3-00N ADR files

Section order (mandatory):
1. Frontmatter (Status / Owner / Requirement refs / Downstream consumer / ADR write-up prose)
2. `## Context` — background, constraints, why the decision was non-obvious
3. `## Decision` — chosen approach, concrete
4. `## Consequences` with `### Positive`, `### Negative`, `### Neutral` subsections
5. `## Alternatives Considered` — Options B/C/D (never A — A is the adopted decision) with verbatim CONTEXT.md quotes per D-30.7
6. `## Footnotes / References` — numbered footnotes `[^1]:`
7. Footer italic line: `*ADR-v3.3-NNN locked: YYYY-MM-DD. Plan 43-NN.*`

### D-30.7 Alternatives Verbatim Quote Discipline
**Source:** `docs/adr/v3.2-001-clock-access-pattern.md` lines 166–170 (example)
**Apply to:** `## Alternatives Considered` section in all four ADRs
```markdown
The relevant context decision (43-CONTEXT.md D-43.5) states verbatim:

> **D-43.5:** Named-matrix storage MUST NOT touch `state.matrix_dim` or
> `state.matrix_active_reg`. These belong to Math Pac I's R14/R15+ register-based
> matrix model. Complete isolation between the two matrix systems.
```

### Free42 Disclaim Verbatim Pattern
**Source:** `docs/adr/v3.1-002-distribution-primitives-policy.md` (per RESEARCH.md line 189)
**Apply to:** ADR-v3.3-002 (FROOT Laguerre algorithm)
```
"Algorithm independently re-derived from primary literature (Numerical Recipes §9.5
Laguerre's method); Free42 source consulted only as sanity-check oracle, not copied."
```

### D-45-NN Identifier Convention
**Apply to:** All entries in `docs/hp41-advantage-divergences.md`

Phase-origin convention: `D-45-NN` (tied to Phase 45, same pattern as D-30-NN / D-35-NN / D-40-NN). Do NOT use D-43-NN for divergence catalog entries — that prefix was used for CONTEXT.md implementation decisions in Phase 43. The next unused D-45-NN value after the nine planned entries would be D-45-10 onward (reserved for Phase 47 test-hardening discoveries).

### Architecture-History v3.x Section Convention
**Source:** `docs/architecture-history.md` lines 277–348
**Apply to:** v3.3 section append

- Section header: `## v3.x additions (Module Name Emulation, Phases NN–MM)`
- Intro paragraph: 3–5 sentences summarizing the novel architectural additions vs. prior modules
- Per-phase subsection: `### Phase NN — Title (shipped YYYY-MM-DD)`
- End with frozen-invariants block listing all six invariants (SC-4, 4-way, unwrap_used, serde backward compat, MSRV, Free42 guard)

### CLAUDE.md v3.x Additions Block Convention
**Source:** `CLAUDE.md` lines 193–251
**Apply to:** v3.3 additions block append

- Section: `### v3.x additions (Module Name, Phases NN–MM)`
- Origin line referencing architecture-history.md and block ordinal (THIRD block per D-40.9 convention)
- Per-phase subsection with `#### Phase NN — Title (shipped YYYY-MM-DD)` headers
- Phases 46 and 47 as forward stubs ("in progress" / "planned") — do NOT write full content for unshipped phases

---

## No Analog Found

All five files have exact analogs in the codebase. No files require falling back to RESEARCH.md patterns.

---

## Pitfall Reminders (from RESEARCH.md)

| Pitfall | Risk | Avoidance |
|---------|------|-----------|
| Pitfall 1 | ADV-DOC-01 function matrix is ALREADY done — Phase 44 Plan 01 generated it | No justfile changes needed; `just docs-matrix-check` already exits 0 |
| Pitfall 3 | Hard-claim language in README soft-claim | Phase 47 graduates the hard-claim; Phase 45 uses soft-claim only |
| Pitfall 4 | Writing Phase 46/47 as complete in CLAUDE.md block | Write Phases 46/47 as forward stubs only |
| Pitfall 5 | Missing MATH_1 alias overlap in ADR-v3.3-003 | 12 mnemonics (E^Z, LNZ, LOGZ, Z^N, Z^1/N, Z^W, |Z|, SINZ, COSZ, TANZ, A^Z, CINV) must appear in ADR-v3.3-003 and D-45-09 |
| Pitfall 6 | D-43.7 cross-nesting not documented | D-45-08 must document solver cross-nesting policy |
| Pitfall 7 | `adv_tvm_state` documented as `#[serde(default, skip)]` | Correct shape is `#[serde(default)]` WITHOUT skip — follows rand_seed precedent |

---

## Metadata

**Analog search scope:** `docs/`, `docs/adr/`, `CLAUDE.md`, `README.md`
**Files scanned:** 8 (4 ADRs + 2 divergence catalogs + architecture-history + README)
**Pattern extraction date:** 2026-05-26
