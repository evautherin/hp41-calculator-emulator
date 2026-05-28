# Phase 52: Test Hardening + Documentation - Context

**Gathered:** 2026-05-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Make Extended Memory **production-ready** — backward-compatible (XMEM-08), isolated (XMEM-09), and fully integrated across CLI, GUI, and the test/doc suite (XMEM-10). The 8 core ops (`EmDir`, `EmRoom`, `SaveP`, `GetP`, `SaveD`, `GetD`, `EmReg`, `SaveRx`) already exist in `hp41-core` and dispatch correctly (Phase 51). This phase connects them to users and locks them down with tests + docs.

**Requirements:** XMEM-08, XMEM-09, XMEM-10.

**Success criteria (from ROADMAP):**
1. A v3.3 save file loads without error in v4.0 and `xmem_files` is empty (not missing/erroring).
2. X-MEM operations never read from or write to `state.regs` or `adv_matrices` — verified by targeted isolation tests.
3. `XEQ "EMDIR"`, `XEQ "SAVEP"`, and all X-MEM functions are reachable from both CLI and GUI and appear in the `?` help overlay.

**In scope:** CLI + GUI XEQ-by-name wiring, dedicated help-overlay JSON pool + matrix doc, backward-compat fixture test, isolation tests, extended meta-gates, ADRs + divergences doc + scoped README claim.

**Out of scope (split to new phases / deferred):**
- **CATALOG 3 (full mainframe built-in catalog)** → new **Phase 53** (see Deferred Ideas). User explicitly chose to split this out rather than expand Phase 52.
- ASCII + STATUS X-MEM file types — already tracked as XMEM-F01 / XMEM-F02 (v4.1).
- Comfort key shortcuts for X-MEM ops (XEQ-by-name only; users assign keys via existing USER-mode ASN).

</domain>

<decisions>
## Implementation Decisions

### Help-overlay placement (JSON pipeline)
- **D-52.1:** X-MEM functions get a **dedicated `docs/hp41-xmem-functions.json` pool** with its own `OnceLock<Vec<HelpEntry>>` in `help_data.rs`, mirroring the 4 XROM-module pools. This gives X-MEM its own labeled section in the `?` overlay (matching the module-grouped UX). Chosen over folding into `docs/hp41cv-functions.json` despite X-MEM being CX OS built-ins — the section grouping + clean op↔JSON parity test won out.
- **D-52.2:** A **standalone `docs/hp41-xmem-function-matrix.md`** is generated from the new pool and wired into `just docs-matrix` + the `just docs-matrix-check` CI drift-catch — fully symmetric with how each XROM module got its own matrix doc.
- **D-52.3:** All 8 X-MEM JSON entries carry **full worked examples** (`example` + `notes` fields), matching the Phase 49 searchable-reference quality bar (e.g. SAVEP/GETP round-trip, SAVED/GETD register transfer, EMREG indexing). The Phase 49 searchable reference auto-covers X-MEM once the new pool loads (criterion 3).

### CLI/GUI access path
- **D-52.4:** **XEQ-by-name only.** Wire `builtin_card_op` (`hp41-core/src/ops/program.rs:1368` — the existing built-in name→Op resolver, e.g. `"PI" => Op::Pi`) so `XEQ "EMDIR"` etc. resolve, and the corresponding GUI `key_map::resolve` string-ID path. Faithful: real HP-41CX Extended Functions are accessed by name or user-assigned via ASN — no dedicated physical keys. Satisfies criterion 3 with minimal surface. No comfort key shortcuts (rejected — X-MEM ops have clean XEQ names, unlike the card-reader ops that justified Ctrl+W/R/D/F).
- **D-52.5:** Discoverability via the `?` overlay + Phase 49 searchable reference (both auto-driven by D-52.1's pool). **No CATALOG 3 in this phase** — split to Phase 53 (see Deferred Ideas).

### Documentation depth & README claim
- **D-52.6:** **Scoped/honest README claim** — e.g. "Extended Memory: named PROGRAM + DATA file storage (HP-41CX X-Functions)". NO "feature-complete" wording, because ASCII + STATUS file types are deferred to v4.1. Consistent with the project's module-by-module honest-claim discipline; the OM-cited hard claim graduates only when ASCII/STATUS land. **Do NOT graduate to the hard claim this phase.**
- **D-52.7:** **Granular per-decision ADRs** for the architecturally significant Phase-51 choices (matching prior modules' ADR granularity). At minimum: X-MEM as OS built-ins / no XROM bit; fixed 600-register capacity (D-51.1); full-register-set SAVED/GETD vs block-control-word (D-51.5). Planner picks exact ADR numbering/count following `docs/adr/` convention.
- **D-52.8:** A **new `docs/hp41-xmem-divergences.md`** documents the OM divergences — primarily D-51.6 overwrite-on-duplicate (vs real CX "DUP FL" + PURFL) and the D-51.5 full-register-set SAVED/GETD (vs the bbb.eee block control word). Matches the existing `docs/hp41-*-divergences.md` per-module pattern.
- **D-52.9 (standard follow-through):** The v4.0/X-MEM narrative gets the usual CLAUDE.md + `docs/architecture-history.md` updates, following the per-phase documentation pattern used by every prior module.

### Test hardening
- **D-52.10:** **v3.3 backward-compat fixture test (XMEM-08, P53).** Create a faithful `hp41-core/tests/fixtures/v33-autosave.json` that genuinely lacks the `xmem_files` / `xmem_active_file` fields, plus a test (e.g. `xmem_backward_compat.rs`, following `stat1_backward_compat.rs` / `adv_backward_compat.rs`) asserting it loads without error and `xmem_files` is empty. Mirrors the existing `v20/v30/v31/v32-autosave.json` fixture family.
- **D-52.11:** **Isolation tests (XMEM-09).** Targeted tests proving X-MEM ops never read/write `state.regs` or `adv_matrices` except via the explicit SAVED/GETD transfer (D-51.0a / D-43.5 precedent in `adv_backward_compat.rs`-style coverage).
- **D-52.12:** **`migrate_after_load()` v3.3→v4.0.** New fields default cleanly via `#[serde(default)]`, so confirm whether any explicit migration arm is needed at all (likely none beyond the serde defaults) — researcher/planner verify against `state.rs:migrate_after_load()` (line ~547 per Phase-51 ref).

### Claude's Discretion
- **Extended meta-gates (D-52.13, user said "you decide" — decision: extend):** Add an **op↔JSON parity test** (the 8 X-MEM `Op` variants ↔ 8 entries in `hp41-xmem-functions.json` must match exactly) plus a **per-op test-count floor** (≥5, matching Pitfall 16 / `math1_op_test_count` discipline). `xrom_shadowing` is N/A — X-MEM is not an XROM. Rationale: cheap now that X-MEM has its own pool (D-52.1); catches op/JSON drift the compile-time 4-way match can't; consistent with the discipline applied to all 5 XROM modules.
- **Fixture authenticity (D-52.10):** Prefer capturing a real save from the `v3.3` tag if feasible; otherwise a faithfully hand-crafted minimal v3.3 `CalcState` JSON without the xmem fields. Planner/researcher verifies it genuinely exercises the `#[serde(default)]` path.
- **EMDIR print-buffer line format, error-variant naming, exact ADR numbering** — planner's call, following established patterns (`CATALOG`/`ALMCAT` print-buffer catalogs; existing `HpError` variants; `docs/adr/` numbering).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 51 foundation (the ops this phase integrates)
- `.planning/phases/51-x-mem-core/51-CONTEXT.md` — all D-51.x decisions (storage model, capacity, EMREG pair, SAVED/GETD, overwrite-on-duplicate, op resolution path). The source of truth for what Phase 51 built.
- `hp41-core/src/ops/xmem/mod.rs` — `XmemFile` / `XmemKind` model, `register_count()`, `XMEM_CAPACITY = 600`.
- `hp41-core/src/ops/xmem/ops.rs` — the 8 op implementations (`op_emdir`/`op_emroom`/`op_savep`/`op_getp`/`op_saved`/`op_getd`/`op_emreg`/`op_saverx`).

### Op infrastructure (4-way exhaustive match — arms 3+4 already exist)
- `hp41-core/src/ops/mod.rs` — `Op` enum + `dispatch()` (xmem arms at ~lines 2172–2179).
- `hp41-core/src/ops/program.rs` — `execute_op()`; **`builtin_card_op()` at line 1368** — the built-in name→Op resolver X-MEM XEQ-by-name wires into (D-52.4).
- `hp41-cli/src/prgm_display.rs` (~line 484) + `hp41-gui/src-tauri/src/prgm_display.rs` (~line 502) — both `op_display_name()` X-MEM arms (already present).

### CLI/GUI resolution + help pipeline
- `hp41-cli/src/keys.rs` — `xeq_by_name_local_resolve()` (line ~369, delegates to `builtin_card_op`).
- `hp41-gui/src-tauri/src/key_map.rs` — GUI string-ID `resolve()`.
- `hp41-cli/src/help_data.rs` — per-pool `include_str!` + `OnceLock<Vec<HelpEntry>>` (add the X-MEM pool, D-52.1); right-panel `key_ref_entries()` (note: filters `entry.xrom.is_none()` — X-MEM entries have no xrom, so verify their right-panel behavior).
- `docs/hp41-*-functions.json` (math1/stat1/time/advantage) + `docs/hp41cv-functions.json` — pool structure precedent for the new `docs/hp41-xmem-functions.json`.

### State model + backward compat + isolation
- `hp41-core/src/state.rs` — `CalcState` (`xmem_files` line ~432, `xmem_active_file` line ~442); `migrate_after_load()` (~line 547, D-52.12).
- `hp41-core/tests/fixtures/v20-/v30-/v31-/v32-autosave.json` — fixture family the new `v33-autosave.json` joins (D-52.10).
- `hp41-core/tests/stat1_backward_compat.rs`, `hp41-core/tests/adv_backward_compat.rs` — backward-compat + isolation test precedent.
- `hp41-core/src/ops/advantage/mod.rs:111` (`AdvMatrix`) + `:363` (`adv_matrices`) — the D-43.5 isolation pattern X-MEM replicates (XMEM-09).

### Meta-gates + doc tooling
- `hp41-cli/tests/function_matrix_parity.rs`, `hp41-cli/tests/key_coverage.rs` — op↔JSON parity + coverage test precedent (extend for X-MEM, D-52.13).
- `hp41-core/tests/xrom_shadowing.rs` — XROM meta-gate (N/A to X-MEM, but shows the meta-gate style).
- `scripts/docs-matrix` + `justfile` (`just docs-matrix` / `docs-matrix-check`) — matrix-doc generator (wire the new standalone matrix doc, D-52.2).

### Project constraints + docs
- `CLAUDE.md` — Frozen invariants: 4-way exhaustive-match, never-discard D-07, `#[serde(default)]` backward-compat rule, no `println!` in `hp41-core` (use `print_buffer`), `#![deny(clippy::unwrap_used)]`, JSON canonical data flow, zero new runtime deps, Quality Gates (≥95% lines / ≥93% regions).
- `.planning/REQUIREMENTS.md` §Extended Memory — XMEM-08/09/10 (this phase), XMEM-F01/F02 (v4.1 deferred).
- `.planning/ROADMAP.md` §Phase 52 — goal + 3 success criteria.
- `.planning/STATE.md` §Accumulated Context — P53 (backward-compat fixture), P56 (isolation).
- `docs/adr/` + `docs/hp41-*-divergences.md` — ADR numbering + divergences-doc conventions (D-52.7, D-52.8).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`builtin_card_op()`** (`program.rs:1368`): the built-in name→Op resolver (`"PI" => Op::Pi`, `"MOD" => Op::Mod`). X-MEM XEQ-by-name adds 8 arms here (D-52.4); CLI `xeq_by_name_local_resolve` already delegates to it.
- **`v20/v30/v31/v32-autosave.json` + `stat1/adv_backward_compat.rs`**: direct template for the XMEM-08 fixture + test (D-52.10).
- **The 4 XROM-module JSON pools + their `OnceLock`s in `help_data.rs`**: template for the new `hp41-xmem-functions.json` pool (D-52.1).
- **`function_matrix_parity.rs` / `key_coverage.rs`**: the op↔JSON parity meta-gate harness to extend (D-52.13).
- **`adv_matrices` / `AdvMatrix` isolation**: the D-43.5 precedent the XMEM-09 isolation tests mirror.
- **`CATALOG` / `ALMCAT` print-buffer catalogs**: the pattern EMDIR output already follows (relevant if Phase 53 CATALOG 3 reuses it).

### Established Patterns
- **4-way exhaustive match**: arms 1+2 (dispatch/execute_op) and 3+4 (both `op_display_name`) already compile for the 8 X-MEM ops — this phase adds the *runtime* wiring (XEQ-by-name + help JSON), not new Op variants.
- **JSON canonical data flow**: `include_str!` + `OnceLock`; malformed JSON panics at first access — the new pool must be valid.
- **Never-discard (D-07)**: X-MEM errors surface as CLI status / GUI toast; never silent.
- **`#[serde(default)]` backward compat**: `xmem_files` + `xmem_active_file` already carry it (Phase 51); this phase *tests* it (XMEM-08).
- **No new runtime deps** (v3.0+ policy): X-MEM integration adds zero deps.

### Integration Points
- **`builtin_card_op` + GUI `key_map::resolve`**: the two XEQ-by-name resolution points (D-52.4).
- **`help_data.rs`**: new pool `OnceLock` + right-panel `key_ref_entries()` interaction (xrom-filter caveat).
- **`justfile` / `scripts/docs-matrix`**: new standalone matrix doc + drift-check (D-52.2).
- **`hp41-core/tests/`**: new backward-compat fixture+test, isolation tests, extended meta-gates.

</code_context>

<specifics>
## Specific Ideas

- The `?` overlay should show **"Extended Memory" as its own labeled section** (the reason for the dedicated JSON pool, D-52.1) — X-MEM should feel like a distinct, discoverable function family, not scattered among built-ins.
- The README line must stay **honest about the PROGRAM+DATA subset** — no "feature-complete" until ASCII/STATUS ship in v4.1 (D-52.6). Claim honesty is a project value.
- The op↔JSON parity meta-gate should make it **impossible to add a future X-MEM op without a matching JSON entry** (and vice versa) — drift-proofing consistent with the 5 XROM modules.

</specifics>

<deferred>
## Deferred Ideas

- **CATALOG 3 — Full Function Catalog → NEW PHASE 53.** User explicitly chose to split this out (over expanding Phase 52). Scope as the user defined it: the **full ~130-function mainframe built-in catalog**, which requires resolving where our XROM-modeled functions (Time XROM 26, the pacs in CATALOG 2) appear relative to the real CX's built-in CATALOG 3 — a genuine architectural decision deserving its own discuss/plan cycle. **Action:** add Phase 53 to the roadmap via `/gsd-phase` (may extend v4.0 or open v4.1). Real-CX CATALOG 3 lists the entire built-in function set including Extended/X Functions; our model diverges by treating Time + the application pacs as XROM (CATALOG 2).
- **Comfort key shortcuts for X-MEM ops** — rejected for Phase 52 (D-52.4 XEQ-by-name only). Users wanting key access use the existing USER-mode ASN mechanism. Revisit only if user feedback asks for it.
- **ASCII + STATUS X-MEM file types** — XMEM-F01 / XMEM-F02, v4.1. Their arrival is the trigger to graduate the README claim to the OM-cited hard claim (D-52.6).
- **OM-cited "feature-complete" README hard claim for X-MEM** — deferred until ASCII/STATUS land (v4.1).

</deferred>

---

*Phase: 52-test-hardening-documentation*
*Context gathered: 2026-05-28*
