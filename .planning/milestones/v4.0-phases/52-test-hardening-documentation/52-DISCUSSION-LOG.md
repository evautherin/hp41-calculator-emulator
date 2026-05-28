# Phase 52: Test Hardening + Documentation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-28
**Phase:** 52-test-hardening-documentation
**Areas discussed:** Help-overlay placement, CLI/GUI access path, Docs depth & README claim, Test-hardening depth

---

## Help-overlay placement

### JSON pool
| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated `hp41-xmem-functions.json` | New file + own `OnceLock`, mirrors 4 XROM-module pools; own labeled `?`-overlay section; clean op↔JSON parity test | ✓ |
| Fold into `hp41cv-functions.json` | Add to 130-function built-in pool; architecturally truest (X-MEM = CX OS built-ins); no new OnceLock; intermixed | |

### Matrix doc
| Option | Description | Selected |
|--------|-------------|----------|
| Standalone `hp41-xmem-function-matrix.md` | Own matrix doc + `just docs-matrix` + CI drift-catch; symmetric with XROM modules | ✓ |
| Append to built-in matrix | Extended Memory section in existing built-in matrix; fewer files | |

### Entry depth
| Option | Description | Selected |
|--------|-------------|----------|
| Full worked examples | `example` + `notes` per entry, Phase 49 searchable-reference quality bar | ✓ |
| Basic entries only | Name + description, no worked examples | |

**User's choice:** Dedicated pool / standalone matrix doc / full worked examples.
**Notes:** Chose the module-grouped UX consistency over the strict "built-ins belong in the built-in pool" architectural purity.

---

## CLI/GUI access path

### Access path
| Option | Description | Selected |
|--------|-------------|----------|
| XEQ-by-name only | Wire `builtin_card_op` + GUI `key_map::resolve`; faithful (CX uses XEQ/ASN, no dedicated keys); minimal; satisfies criterion 3 | ✓ |
| Add comfort key shortcuts | Also bind Ctrl+/KEY_DEFS shortcuts; convenient but non-faithful + more test surface | |

### Catalog
| Option | Description | Selected |
|--------|-------------|----------|
| Defer CATALOG 3 | `?` overlay + searchable reference + XEQ-by-name; CATALOG 3 a new capability beyond XMEM-10 | |
| Add CATALOG 3 now | Implement CATALOG 3 listing this phase for faithfulness | ✓ (then re-scoped below) |

### Cat-3 scope
| Option | Description | Selected |
|--------|-------------|----------|
| X-MEM / Extended Functions only | Bounded; avoids Time-as-XROM conflict | |
| Full built-in mainframe catalog | Faithful ~130-function catalog; forces Time/XROM-catalog decision; large surface | ✓ |

### Scope fork
| Option | Description | Selected |
|--------|-------------|----------|
| New phase for CATALOG 3 | Keep Phase 52 focused on X-MEM; add Phase 53 with own discuss/plan cycle | ✓ |
| Expand Phase 52 to include it | One bigger phase; risks test-hardening focus + milestone close | |

**User's choice:** XEQ-by-name only for Phase 52. CATALOG 3 (full mainframe catalog) split to a new Phase 53.
**Notes:** User initially wanted CATALOG 3 in-phase and at full faithful scope. Claude flagged the scope/budget tension (feature-sized, last phase of v4.0, muddies milestone close); user agreed to split it to a dedicated Phase 53.

---

## Docs depth & README claim

### README claim
| Option | Description | Selected |
|--------|-------------|----------|
| Scoped/honest claim | "PROGRAM + DATA file storage"; no "feature-complete" (ASCII/STATUS deferred); honest-claim discipline | ✓ |
| OM-cited hard claim | "feature-complete per OM"; would overclaim given the subset | |

### ADRs
| Option | Description | Selected |
|--------|-------------|----------|
| Granular per-decision ADRs | Separate ADRs for built-ins/no-XROM, 600-reg capacity, SAVED/GETD model; matches prior modules | ✓ |
| One consolidated X-MEM ADR | Single design doc covering all D-51.x; fewer files | |

### Divergences
| Option | Description | Selected |
|--------|-------------|----------|
| New `hp41-xmem-divergences.md` | Dedicated doc, matches per-module pattern | ✓ |
| Fold into existing divergences doc | Append section; fewer files, breaks symmetry | |

**User's choice:** Scoped README claim / granular ADRs / dedicated divergences doc.
**Notes:** Honest-claim discipline reaffirmed — hard "feature-complete" claim deferred until ASCII/STATUS land in v4.1.

---

## Test-hardening depth

### Meta-gates
| Option | Description | Selected |
|--------|-------------|----------|
| Extend meta-gates to X-MEM | op↔JSON parity (8 ops ↔ 8 entries) + per-op test-count floor; matches 5 XROM modules | ✓ (Claude's discretion) |
| Standard coverage + targeted tests only | ≥95%/≥93% gate + backward-compat + isolation; no parity/count meta-gate | |

**User's choice:** "You decide" → Claude chose to extend the meta-gates.
**Notes:** Rationale — cheap now that X-MEM has its own JSON pool; catches op/JSON drift the compile-time 4-way match can't; consistent with the load-bearing meta-gate discipline across all 5 XROM modules. Fixture authenticity (real v3.3-tag capture vs faithful hand-crafted JSON) also left to Claude's discretion.

---

## Claude's Discretion

- **Extended meta-gates (D-52.13):** op↔JSON parity + per-op test-count floor (≥5). `xrom_shadowing` N/A (X-MEM not XROM).
- **Fixture authenticity (D-52.10):** prefer real `v3.3`-tag capture; else faithful hand-crafted minimal v3.3 `CalcState` JSON without xmem fields.
- **EMDIR print-buffer line format, error-variant naming, exact ADR numbering** — follow established patterns (`CATALOG`/`ALMCAT`, existing `HpError`, `docs/adr/` convention).
- **`migrate_after_load()` v3.3→v4.0 (D-52.12):** confirm whether any explicit arm is needed beyond serde defaults.

## Deferred Ideas

- **CATALOG 3 — Full Function Catalog → new Phase 53** (full ~130-function mainframe catalog + Time/XROM-catalog architectural resolution). Add via `/gsd-phase`.
- **Comfort key shortcuts for X-MEM ops** — rejected for Phase 52; users use USER-mode ASN.
- **ASCII + STATUS X-MEM file types** — XMEM-F01/F02, v4.1.
- **OM-cited "feature-complete" README hard claim for X-MEM** — deferred until ASCII/STATUS land (v4.1).
