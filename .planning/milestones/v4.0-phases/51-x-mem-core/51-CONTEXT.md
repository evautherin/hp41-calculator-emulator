# Phase 51: X-MEM Core - Context

**Gathered:** 2026-05-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver the Extended Memory (X-MEM) **data model and core ops in `hp41-core` only**, mirroring HP-41CX behavior. Scope is the seven roadmap-named ops — `EMDIR`, `EMROOM`, `SAVEP`, `GETP`, `SAVED`, `GETD`, `EMREG` — **plus one companion store op** for the EMREG read/write pair (decision D-51.3 below), so eight ops total.

CLI keyboard wiring, GUI IPC, the `?` help-overlay JSON, backward-compat fixture tests, and the full 4-way integration are **Phase 52**, not this phase. Phase 51 ends when the ops exist in `hp41-core`, dispatch correctly, and round-trip through `xmem_files` with unit tests.

**In scope:** PROGRAM and DATA file types only.
**Out of scope (this milestone):** ASCII and STATUS file types — already tracked as XMEM-F01 / XMEM-F02 (v4.1).

</domain>

<decisions>
## Implementation Decisions

### Storage model (carried forward from STATE.md — pre-resolved, do NOT re-litigate)
- **D-51.0a:** X-MEM storage is `xmem_files: Vec<XmemFile>` on `CalcState`, `#[serde(default)]`, fully isolated from `state.regs` and `adv_matrices` (P56 / XMEM-09 — repeats the D-43.5 named-matrix isolation pattern at `hp41-core/src/ops/advantage/mod.rs:363`). X-MEM ops MUST NEVER read/write `state.regs` or `adv_matrices` except via the explicit SAVED/GETD transfer (which is the whole point of those two ops).
- **D-51.0b:** `SAVEP`/`GETP` delegate to the `.raw` codec — `encode_program` / `decode_program` in `hp41-core/src/cardreader/raw.rs` — for program serialization. A PROGRAM `XmemFile` stores the `.raw` byte stream. (Phase 50 dependency.)
- **D-51.0c:** File names are taken verbatim from the **ALPHA register**, same convention as the card-reader `CardOpRequest` path. Name sanitization is not required here (in-memory store, not filesystem).

### Capacity model (EMROOM + overflow)
- **D-51.1:** **Fixed 600-register capacity** — fully-expanded HP-41CX (124 built-in + 2×238 X-Memory modules). `EMROOM` returns `600 − registers_used` on the stack (criterion 2). `SAVEP` / `SAVED` return a **"NO ROOM"-style error** when the file would not fit. Chosen over 124 (too tight — one data file fills it) and over "unlimited" (would make EMROOM a meaningless constant). Consistent with the emulator's already-maximally-equipped stance (all 5 XROM modules co-loaded).
- **D-51.2 (register accounting — Claude's discretion, OM-verify):** Faithful default — a PROGRAM file occupies `⌈program_bytes / 7⌉ + 1` registers (HP-41 packs 7 program bytes per register, +1 directory/header register); a DATA file occupies `N + 1` registers (N data registers + 1 header). Researcher to confirm exact header/packing accounting against the HP-41CX OM before locking the EMROOM formula.

### EMREG access + active-file pointer
- **D-51.3:** `EMREG` is **read + write via a GETRX/SAVERX-style pair**, not a single overloaded op:
  - `EMREG` = **recall**: index N from X → pushes register N of the active file onto the stack.
  - **companion store op** (SAVERX-equivalent) = **store**: writes a value into register N of the active file in place.
  - This adds an **8th op** to the phase → it MUST land in all four arms of the 4-way exhaustive match (dispatch / execute_op / CLI prgm_display / GUI prgm_display). Store-op **mnemonic naming is Claude's discretion** for the planner (e.g. an `EMREG`-store sibling; check HP-41CX SAVERX/GETRX naming for fidelity).
- **D-51.4:** The **active file** that EMREG reads/writes is set as a **side effect of `GETD` / `SAVED`** (most-recently saved-or-retrieved DATA file becomes active). Stored as e.g. `xmem_active_file: Option<...>` on `CalcState`. No dedicated "select file" op (none is in scope).

### SAVED / GETD register transfer
- **D-51.5:** `SAVED` captures the **full current register set** R00..R(SIZE-1) (reuse `capture_data_card` logic); `GETD` replaces `state.regs` **wholesale**, padding back up to `MIN_REGS_AFTER_LOAD` (100) (reuse `load_data_card` logic). Both helpers already exist and are tested in `hp41-core/src/cardreader/mod.rs`. Chosen over the faithful bbb.eee block control-word (deferred — see Deferred Ideas).

### Error & overwrite behavior
- **D-51.6:** Duplicate name on `SAVEP` / `SAVED` → **overwrite the existing file in place**. The Phase 51 op set has **no purge/clear-file op**, so erroring on duplicate would make files permanently unreplaceable. This is a **documented divergence** from real HP-41CX (which returns "DUP FL" and requires PURFL first) — record it in the divergences catalog during Phase 52 docs.
- **D-51.7:** `GETP` loads a retrieved program via **`insert_program_ops`** (RDPRGM semantics): empty program memory → replace + reset pc to 0; otherwise insert immediately after pc. Reuses the tested helper in `hp41-core/src/cardreader/mod.rs`; consistent with the `.raw` card-reader path.

### Claude's Discretion
- **Register-accounting formula (D-51.2)** — implement the faithful 7-bytes/register + header model; verify against the OM.
- **EMREG store-op mnemonic (D-51.3)** — pick a name; prefer HP-41CX SAVERX/GETRX fidelity.
- **Error variants/messages:** type-mismatch (GETP on a DATA file, GETD on a PROGRAM file) → a "FL TYPE"-style error; missing file name on GETP/GETD/EMREG → a "file-not-found" error; EMREG with no active file set or an out-of-range index → error. **All errors MUST surface** (never silently swallowed — CLAUDE.md never-discard D-07). Concrete `HpError` variant(s) vs reusing existing ones is the planner's call.
- **EMDIR presentation:** print name / type (PROGRAM|DATA) / size-in-registers, following the existing `CATALOG` / `ALMCAT` print-buffer catalog pattern from prior modules. Exact line format is discretion.
- **Op resolution path:** X-MEM ops are **built-ins resolved by XEQ-by-name** (HP-41CX Extended Functions are OS built-ins, not a plug-in ROM module). **No XROM bit** is allocated — `default_xrom_modules` bits 0–4 are already taken by the five existing modules, and X-MEM is not an XROM. Confirm this routing during planning.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Core codec (SAVEP/GETP/SAVED/GETD delegate here)
- `hp41-core/src/cardreader/raw.rs` — `.raw` program byte codec: `encode_program(&[Op]) -> Vec<u8>`, `decode_program(&[u8]) -> Vec<Op>`. PROGRAM `XmemFile` stores this byte stream.
- `hp41-core/src/cardreader/mod.rs` — `insert_program_ops` (D-51.7), `capture_data_card` / `load_data_card` + `MIN_REGS_AFTER_LOAD = 100` (D-51.5), `CardOpRequest` ALPHA-name convention (D-51.0c).
- `hp41-core/src/cardreader/data.rs` — `DataCard` struct; the register-vector serialization shape SAVED/GETD mirror.

### State model + isolation precedent
- `hp41-core/src/state.rs` — `CalcState` struct (add `xmem_files` + `xmem_active_file`); `migrate_after_load()` at line 547 (Phase 52 adds the v3.3→v4.0 migration, but new fields default cleanly via `#[serde(default)]`).
- `hp41-core/src/ops/advantage/mod.rs:111` — `AdvMatrix` struct; `mod.rs:363` `adv_matrices` field — the D-43.5 isolation pattern `XmemFile` / `xmem_files` must replicate (P56).

### Op infrastructure (4-way exhaustive match — D-51.3 adds an 8th op)
- `hp41-core/src/ops/mod.rs` — `Op` enum + `dispatch()`.
- `hp41-core/src/ops/program.rs` — `execute_op()`.
- `hp41-cli/src/prgm_display.rs` + `hp41-gui/src-tauri/src/prgm_display.rs` — both `op_display_name()` (Phase 52 wiring, but the variants must compile).

### Project constraints
- `CLAUDE.md` — Frozen invariants: workspace structure, BCD/`rust_decimal`, stack-lift `LiftEffect`, ISG/DSE string-split discipline (applies to any decimal field parsing), `#![deny(clippy::unwrap_used)]`, no `println!` in `hp41-core` (use `print_buffer`), `#[serde(default)]` backward-compat rule, never-discard D-07, zero new runtime deps.
- `.planning/REQUIREMENTS.md` §Extended Memory — XMEM-01..07 (this phase), XMEM-08..10 (Phase 52), XMEM-F01/F02 (v4.1 deferred).
- `.planning/ROADMAP.md` §Phase 51 — goal + 5 success criteria.
- `.planning/STATE.md` §Accumulated Context — pre-resolved decisions (D-51.0a/b/c) and pitfalls P53, P56.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`encode_program` / `decode_program`** (`cardreader/raw.rs`): direct serialization backend for SAVEP/GETP — no new codec needed.
- **`capture_data_card` / `load_data_card`** (`cardreader/mod.rs`): the exact full-register-set capture/restore logic SAVED/GETD reuse (D-51.5), including the `MIN_REGS_AFTER_LOAD = 100` zero-pad that keeps STO/RCL nn in bounds.
- **`insert_program_ops`** (`cardreader/mod.rs`): RDPRGM-style program insertion GETP reuses (D-51.7).
- **`adv_matrices` / `AdvMatrix`** (`ops/advantage/`): the structural template for `xmem_files` / `XmemFile` and its serde + isolation discipline.
- **`CATALOG` / `ALMCAT` print-buffer catalogs**: the display pattern EMDIR follows (print_buffer, not `println!`).

### Established Patterns
- **`#[serde(default)]` on every new `CalcState` field** — `xmem_files` and `xmem_active_file` must carry it (XMEM-08 backward compat; P53 — Phase 52 adds the pinned-v3.3-fixture test). Transient-only fields also get `#[serde(skip)]`; persistent X-MEM state is NOT transient, so default-without-skip is correct (it must survive save/load).
- **4-way exhaustive match**: every Op variant (the 7 named ops + the EMREG store sibling = 8) must land in dispatch + execute_op + both `op_display_name`s before anything compiles.
- **Error surfacing (D-07)**: unknown/invalid conditions return `HpError`, never a silent no-op.
- **No floats for any "field-in-a-decimal" parsing** — if any X-MEM op encodes block/index info in a decimal (e.g. a future block control word), use string-split, never `floor()`/`fmod()`. (Not needed under D-51.5's full-set model, but applies if accounting uses packed encodings.)

### Integration Points
- **`CalcState`**: two new fields (`xmem_files`, `xmem_active_file`). GETD/SAVED set the active-file pointer (D-51.4).
- **`Op` enum + `dispatch()`**: 8 new variants.
- **XEQ-by-name resolution**: X-MEM ops resolve as built-ins (not XROM) — see Claude's Discretion note. Phase 52 wires CLI/GUI display + help overlay.

</code_context>

<specifics>
## Specific Ideas

- EMROOM should return a *real* decreasing number as files accumulate (600 → less), not a constant — it's the user-visible proof the capacity model works.
- The EMREG read/write pair should feel like STO/RCL but targeting the active X-MEM data file (recall = EMREG, store = its sibling), matching the HP-41CX GETRX/SAVERX mental model.
- "NO ROOM" on SAVEP/SAVED is a wanted, test-worthy behavior — not an edge case to paper over.

</specifics>

<deferred>
## Deferred Ideas

- **PURFL / CLFL purge-file op** — would enable true HP-41CX "DUP FL" duplicate semantics (error-on-duplicate + explicit purge). Out of scope for Phase 51 (not a roadmap-named op); candidate for a future phase or v4.1. Until then, D-51.6 overwrite-in-place stands.
- **Block control-word SAVED/GETD (bbb.eee in X)** — the hardware-faithful partial-block transfer. Deferred in favor of the full-register-set reuse model (D-51.5). Revisit if partial-block fidelity is requested.
- **ASCII + STATUS X-MEM file types** — already tracked as XMEM-F01 / XMEM-F02 (v4.1). PROGRAM + DATA only this milestone.

</deferred>

---

*Phase: 51-x-mem-core*
*Context gathered: 2026-05-28*
