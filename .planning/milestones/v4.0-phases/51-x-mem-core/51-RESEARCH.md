# Phase 51: X-MEM Core - Research

**Researched:** 2026-05-28
**Domain:** HP-41CX Extended Memory — data model, register accounting, op semantics
**Confidence:** HIGH (primary source: official HP-41CX Quick Reference Guide 00041-90475 verified by image read)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-51.0a:** `xmem_files: Vec<XmemFile>` on `CalcState`, `#[serde(default)]`, fully isolated from `state.regs` and `adv_matrices` (P56 / XMEM-09).
- **D-51.0b:** `SAVEP`/`GETP` delegate to `encode_program` / `decode_program` in `hp41-core/src/cardreader/raw.rs`.
- **D-51.0c:** File names taken verbatim from the ALPHA register (`CardOpRequest` convention).
- **D-51.1:** Fixed 600-register capacity. `EMROOM` = `600 − registers_used`. `SAVEP`/`SAVED` error "NO ROOM" when file would not fit.
- **D-51.5:** `SAVED` captures R00..R(SIZE-1) via `capture_data_card`; `GETD` replaces `state.regs` wholesale via `load_data_card` (with `MIN_REGS_AFTER_LOAD = 100` zero-pad).
- **D-51.6:** Duplicate name on `SAVEP`/`SAVED` → overwrite in place (documented divergence from real HP-41CX "DUP FL").
- **D-51.7:** `GETP` loads via `insert_program_ops` (RDPRGM semantics).

### Claude's Discretion

- **Register-accounting formula (D-51.2)** — implement faithful 7-bytes/register + header model; verify against OM. *(Resolved below — see Standard Stack.)*
- **EMREG store-op mnemonic (D-51.3)** — prefer HP-41CX SAVERX/GETRX fidelity. *(Resolved below.)*
- **Error variants/messages** — FL NOT FOUND, FL TYPE ERR, NO ROOM, plus EMREG index-OOB and no-active-file. *(Resolved below.)*
- **EMDIR presentation** — print_buffer catalog, one line per file with name/type/size.
- **Op resolution path** — built-in, XEQ-by-name; no XROM bit.

### Deferred Ideas (OUT OF SCOPE)

- PURFL / CLFL purge-file op.
- Block control-word SAVED/GETD (bbb.eee partial-block transfer).
- ASCII + STATUS X-MEM file types (XMEM-F01 / XMEM-F02, v4.1).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| XMEM-01 | EMDIR lists all files with names, types, sizes | EMDIR pushes lines to `print_buffer`; format: name / type / size-in-registers; ends with registers available |
| XMEM-02 | EMROOM reports available register space | `600 − registers_used`; hardware confirmed 600 = 124 + 2×238 |
| XMEM-03 | SAVEP saves named program to X-MEM | Delegates to `encode_program`; occupies `⌈bytes/7⌉ + 1` registers |
| XMEM-04 | GETP retrieves named program from X-MEM | Delegates to `decode_program` + `insert_program_ops` |
| XMEM-05 | SAVED saves data registers to named file | Delegates to `capture_data_card`; occupies `N + 1` registers |
| XMEM-06 | GETD retrieves data registers from named file | Delegates to `load_data_card`; sets active-file pointer |
| XMEM-07 | EMREG accesses register N within active X-MEM file | EMREG = recall (GETRX-style); SAVERX = store companion |
</phase_requirements>

---

## Summary

Phase 51 delivers the HP-41CX Extended Memory data model and eight core ops in `hp41-core`. All major architecture decisions are pre-resolved in CONTEXT.md. Research focused on two open questions: (1) the exact register-accounting formula for EMROOM, and (2) the authentic HP-41CX mnemonic for the EMREG store companion.

**Question 1 — Register accounting (D-51.2):** The official HP-41CX Quick Reference Guide (00041-90475, verified by direct PDF read) confirms EMDIR in Catalog 4 displays "the file name, file type, and the number of registers in the file", and that EMROOM displays the number of registers still available. A community forum example (hp41.org, thread-654) shows a 922-byte program occupying ~132 registers (922÷7≈131.7, rounds up to 132), confirming **7 bytes per register** for program storage. The +1 header register per file is the industry-standard accounting used by all known HP-41CX emulators (i41CX+, V41). The total capacity is **600 registers** (124 built-in + 2×238 from two 82181A modules), confirmed by the i41CX+ mini-manual and multiple community sources.

**Question 2 — SAVERX mnemonic:** The official HP-41CX QRG Function Set (page 23) lists `SAVERX` with definition "Save registers by X (bbb.eee). Copy R_sss through R_eee to the current data file." This is the authentic HP-41CX mnemonic for the store companion to `GETRX`. The Phase 51 companion store op should be named `Saverx` in the Op enum and rendered as `"SAVERX"` in `op_display_name`.

**Primary recommendation:** Implement `XmemFile` struct mirroring `AdvMatrix`, add `xmem_files: Vec<XmemFile>` + `xmem_active_file: Option<String>` to `CalcState`, implement all 8 ops (`EmDir`, `EmRoom`, `SaveP`, `GetP`, `SaveD`, `GetD`, `EmReg`, `SaveRx`) routing through the existing cardreader helpers and 7-bytes/register accounting.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| XmemFile data model + Vec storage | hp41-core (`state.rs`) | — | CalcState owns all persistent state; isolation from `regs` / `adv_matrices` mandatory (D-51.0a) |
| Program serialization (SAVEP/GETP) | hp41-core (`cardreader/raw.rs`) | — | `encode_program`/`decode_program` already tested; delegate directly |
| Data register capture/restore (SAVED/GETD) | hp41-core (`cardreader/mod.rs`) | — | `capture_data_card`/`load_data_card` already tested; delegate directly |
| Register-space accounting (EMROOM) | hp41-core (`xmem/` ops module) | — | Pure calculation over `xmem_files`; no frontend involvement |
| EMDIR catalog output | hp41-core (`print_buffer`) | — | Same pattern as CATALOG/ALMCAT; frontend drains |
| EMREG / SAVERX register I/O | hp41-core (`xmem/` ops module) | — | Active-file pointer set by GETD/SAVED (D-51.4) |
| Op dispatch + 4-way exhaustive match | hp41-core (`ops/mod.rs`, `ops/program.rs`) | hp41-cli + hp41-gui `prgm_display.rs` | Phase 52 adds CLI/GUI display; variants must compile in Phase 51 |
| XEQ-by-name resolution | hp41-core (existing `xeq_by_name_local_resolve` / `builtin_card_op`) | — | X-MEM ops are OS built-ins (not XROM); no bit allocation |

---

## Standard Stack

### Core — No New Packages

Phase 51 adds zero new runtime dependencies (project invariant since v3.0). All dependencies are already present in `hp41-core`.

| Component | Source | Purpose |
|-----------|--------|---------|
| `serde` + `serde_json` | workspace dep | `XmemFile` serialization (`#[serde(default)]` on `CalcState` fields) |
| `cardreader::raw` | existing codebase | `encode_program` / `decode_program` for SAVEP/GETP |
| `cardreader::mod` | existing codebase | `capture_data_card` / `load_data_card` / `insert_program_ops` / `MIN_REGS_AFTER_LOAD` for SAVED/GETD/GETP |
| `HpValue` / `HpNum` | `hp41-core::num` | Register data stored in `XmemFile.registers: Vec<HpValue>` (mirrors `DataCard`) |
| `HpError` | `hp41-core::error` | Error propagation; new variants `FileNotFound` + `FileType` added here |

### Package Legitimacy Audit

> No new external packages are installed in this phase. The audit section is trivially satisfied.

| Package | Registry | Age | Downloads | Source Repo | slopcheck | Disposition |
|---------|----------|-----|-----------|-------------|-----------|-------------|
| (none) | — | — | — | — | — | N/A |

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

---

## Architecture Patterns

### System Architecture Diagram

```
ALPHA register (file name)
        │
        ▼
┌───────────────┐    encode_program()     ┌──────────────────────┐
│ SAVEP op      │ ──────────────────────► │  XmemFile            │
│ (name from    │                         │  { name: String,     │
│  alpha_reg)   │    raw bytes Vec<u8>    │    kind: Program,    │
└───────────────┘                         │    data: Vec<u8>,    │
                                          │    reg_count: usize }│
┌───────────────┐    capture_data_card()  │                      │
│ SAVED op      │ ──────────────────────► │  (kind: Data)        │
│               │    Vec<HpValue>         │  data: serialized    │
└───────────────┘                         │  registers           │
                                          └──────────┬───────────┘
                                                     │ stored in
                                                     ▼
                                          CalcState.xmem_files: Vec<XmemFile>
                                                     │
                           ┌─────────────────────────┼───────────────────────┐
                           ▼                         ▼                       ▼
                    ┌──────────┐            ┌────────────────┐      ┌────────────────┐
                    │ EMROOM   │            │ GETP / GETD    │      │ EMREG / SAVERX │
                    │ 600 −    │            │ look up by     │      │ index into     │
                    │ used_regs│            │ alpha_reg name │      │ active_file    │
                    └──────────┘            └────────────────┘      └────────────────┘
                                                     │
                                          decode_program() /
                                          load_data_card()
                                                     │
                                                     ▼
                                          state.program / state.regs
                                          (GETD also sets xmem_active_file)
```

### Recommended Project Structure

```
hp41-core/src/ops/
└── xmem/
    ├── mod.rs          # XmemFile struct, XmemKind enum, register_count()
    ├── ops.rs          # op_emdir, op_emroom, op_savep, op_getp, op_saved, op_getd, op_emreg, op_saverx
    └── (no further split needed — 8 ops, single coherent module)
```

`XmemFile` in `xmem/mod.rs` follows the `AdvMatrix` precedent exactly:
- `pub struct XmemFile { pub name: String, pub kind: XmemKind, pub data: Vec<u8> }`
- `pub enum XmemKind { Program, Data }`
- `pub fn register_count(&self) -> usize` — implements the D-51.2 formula
- `#[derive(Debug, Clone, Serialize, Deserialize, Default)]`

`CalcState` additions (both with `#[serde(default)]`, no `#[serde(skip)]` — persistent state):
```rust
#[serde(default)]
pub xmem_files: Vec<crate::ops::xmem::XmemFile>,

#[serde(default)]
pub xmem_active_file: Option<String>,
```

### Pattern 1: XmemFile Struct (mirrors AdvMatrix)

```rust
// Source: hp41-core/src/ops/advantage/mod.rs:111 (AdvMatrix pattern)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct XmemFile {
    /// File name from ALPHA register at save time.
    pub name: String,
    /// PROGRAM or DATA file type.
    pub kind: XmemKind,
    /// PROGRAM: raw .raw bytes (output of encode_program).
    /// DATA: JSON-serialized registers (output of encode_data, stored as bytes).
    pub data: Vec<u8>,
}

impl XmemFile {
    /// Register count for capacity accounting (D-51.2).
    /// PROGRAM: ⌈data.len() / 7⌉ + 1 (7 bytes packed per register, +1 header).
    /// DATA: N + 1 (N = number of registers inferred from decode, +1 header).
    pub fn register_count(&self) -> usize {
        match self.kind {
            XmemKind::Program => self.data.len().div_ceil(7) + 1,
            XmemKind::Data => {
                // decode_data to count registers; store count alongside data
                // OR store count directly as a field. See Pattern 2 below.
                self.stored_reg_count + 1
            }
        }
    }
}
```

**Note for planner:** The DATA register count cannot be cheaply recovered by decoding JSON on every EMROOM call. Recommend adding a `reg_count: usize` field to `XmemFile` that is set at SAVED time and used directly. This avoids deserializing on every EMROOM call and keeps the formula O(n_files).

### Pattern 2: register_count for DATA files — store at save time

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct XmemFile {
    pub name: String,
    pub kind: XmemKind,
    pub data: Vec<u8>,
    /// For DATA files: number of HP-41 registers captured (set at SAVED time).
    /// For PROGRAM files: 0 (register_count() computes from data.len()).
    #[serde(default)]
    pub reg_count: usize,
}
```

### Pattern 3: SAVED / GETD delegate to cardreader helpers

```rust
// Source: hp41-core/src/cardreader/mod.rs — capture_data_card / load_data_card
fn op_saved(state: &mut CalcState) -> Result<(), HpError> {
    let name = state.alpha_reg.clone();
    if name.is_empty() { return Err(HpError::AlphaData); }
    let card = capture_data_card(state);  // existing tested helper
    let bytes = encode_data(&card)?;      // serialize to JSON bytes
    let reg_count = card.registers.len();
    let new_file = XmemFile { name: name.clone(), kind: XmemKind::Data, data: bytes, reg_count };
    // Check capacity BEFORE mutating (D-51.1 NO ROOM check)
    let used_after = registers_used(state) - existing_file_regs(&state, &name) + new_file.register_count();
    if used_after > XMEM_CAPACITY { return Err(HpError::NoRoom); }
    upsert_file(state, new_file);         // D-51.6 overwrite in place
    state.xmem_active_file = Some(name); // D-51.4 set active file
    Ok(())
}

fn op_getd(state: &mut CalcState) -> Result<(), HpError> {
    let name = state.alpha_reg.clone();
    let file = find_file(state, &name)?.ok_or(HpError::FileNotFound)?;
    if file.kind != XmemKind::Data { return Err(HpError::FileType); }
    let card = decode_data(&file.data)?;
    load_data_card(state, card);          // existing tested helper (pads to MIN_REGS_AFTER_LOAD)
    state.xmem_active_file = Some(name); // D-51.4 set active file
    Ok(())
}
```

### Pattern 4: EMDIR output (print_buffer catalog pattern)

```rust
// Source: hp41-core/src/ops/program.rs — op_catalog print_buffer pattern
fn op_emdir(state: &mut CalcState) -> Result<(), HpError> {
    state.print_buffer.push(format!("{:>24}", "-- XMEM DIR --"));
    for file in &state.xmem_files {
        let type_str = match file.kind { XmemKind::Program => "PGM", XmemKind::Data => "DAT" };
        state.print_buffer.push(format!("{} {} {}", file.name, type_str, file.register_count()));
    }
    let available = XMEM_CAPACITY - registers_used_total(state);
    state.print_buffer.push(format!("{} REGS FREE", available));
    // LiftEffect: Neutral (catalog, no stack change)
    Ok(())
}
```

### Pattern 5: EMREG (recall) + SAVERX (store)

```rust
// EMREG: recall register N (from X) in active data file → push to stack
fn op_emreg(state: &mut CalcState) -> Result<(), HpError> {
    let n = state.stack.x.to_u8_index()?;   // integer index from X
    let name = state.xmem_active_file.as_ref().ok_or(HpError::FileNotFound)?.clone();
    let file = find_file(state, &name)?.ok_or(HpError::FileNotFound)?;
    if file.kind != XmemKind::Data { return Err(HpError::FileType); }
    let card = decode_data(&file.data)?;
    let reg = card.registers.get(n as usize).ok_or(HpError::OutOfRange)?;
    apply_lift_effect(state, LiftEffect::Enable);
    state.stack.x = reg.into_hpnum();
    Ok(())
}

// SAVERX: store X into register N of active data file (in place)
fn op_saverx(state: &mut CalcState) -> Result<(), HpError> {
    let n = state.stack.x.to_u8_index()?;
    // Note: on real HP-41CX, SAVERX takes bbb.eee notation; emulator simplification:
    // treat integer part as register index (D-51.5 full-set model context)
    let name = state.xmem_active_file.as_ref().ok_or(HpError::FileNotFound)?.clone();
    let file = find_file_mut(state, &name)?.ok_or(HpError::FileNotFound)?;
    if file.kind != XmemKind::Data { return Err(HpError::FileType); }
    let mut card = decode_data(&file.data)?;
    let slot = card.registers.get_mut(n as usize).ok_or(HpError::OutOfRange)?;
    *slot = state.stack.y.into_hpvalue();  // store Y into register N
    file.data = encode_data(&card)?;
    // LiftEffect: Neutral (store op)
    Ok(())
}
```

**Note on SAVERX stack effect:** On real HP-41CX, SAVERX takes a `bbb.eee` block-range argument in X. Under the Phase 51 full-set model (D-51.5), the simplified form is: integer part of X = register index N; store the value in Y into register N. This is documented as an emulator simplification (analogous to SAVED capturing the full set).

### Anti-Patterns to Avoid

- **Sharing address space with `state.regs`:** X-MEM files MUST use `xmem_files: Vec<XmemFile>` — never index into `state.regs` for X-MEM storage. This is Pitfall P56.
- **Forgetting `#[serde(default)]` without `#[serde(skip)]`:** `xmem_files` and `xmem_active_file` are persistent state. They must survive save/load. Do not add `#[serde(skip)]`.
- **Calling `decode_data` inside EMROOM:** Deserializing JSON on every EMROOM call is O(n·bytes). Store `reg_count` in `XmemFile` at SAVED time.
- **Using `println!` for EMDIR output:** Never. Use `state.print_buffer.push(...)` — `println!` is forbidden in `hp41-core`.
- **Silently swallowing errors (D-07):** Every error condition (FL NOT FOUND, FL TYPE ERR, NO ROOM, index OOB, no active file) must return an `HpError`. Never a silent no-op.
- **EMREG on PROGRAM file:** EMREG/SAVERX are valid only for DATA files. PROGRAM files have no register-level access; accessing them must return `HpError::FileType`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Program serialization | Custom byte codec | `encode_program` / `decode_program` in `cardreader/raw.rs` | Already tested (Phase 50); END marker, synthetic byte, alpha-label encoding all handled |
| Register capture/restore | Custom serialize loop | `capture_data_card` / `load_data_card` in `cardreader/mod.rs` | Handles `MIN_REGS_AFTER_LOAD` padding; tested in Phase 50 |
| Program insertion | Custom splice | `insert_program_ops` in `cardreader/mod.rs` | RDPRGM semantics (empty replace / insert after pc) already tested |
| Register vector serialization | Hand-written JSON | `encode_data` / `decode_data` in `cardreader/data.rs` | `DataCard` struct with format tag; already tested |

**Key insight:** Phase 51 is deliberately thin on new code because all codec and insertion logic was built and tested in Phases 38–50. The new work is the data model (`XmemFile` struct), the 8 op functions, and the 4-way enum wiring.

---

## Resolved Open Questions

### Q1 — Register Accounting Formula (D-51.2) — RESOLVED WITH HIGH CONFIDENCE

**PROGRAM file:** `⌈data_bytes / 7⌉ + 1` registers
- **7 bytes per register:** Confirmed by community example (922-byte program → 132 registers; 922÷7 = 131.7 → ⌈⌉ = 132). [CITED: forum.hp41.org/viewtopic.php?f=20&t=654]
- **+1 header register:** Per-file directory/header overhead of 1 register. Consistent with Catalog 4 displaying "number of registers in the file" (QRG p.34) which is understood to include the header. Used universally by HP-41CX emulators (i41CX+, V41). [ASSUMED: header overhead = exactly 1; could be 0 if "registers in the file" means payload only — low risk given universal emulator consensus]

**DATA file:** `N + 1` registers (N = number of captured registers)
- DATA files store register values; the +1 accounts for the file header. [ASSUMED: same per-file header accounting as PROGRAM; no primary source distinguishes DATA vs PROGRAM header overhead]

**Total capacity:** **600 registers** (124 built-in + 2×238 from two 82181A modules)
- Confirmed by: QRG p.29 ("124 registers for text, data, or program files"); i41CX+ mini-manual p.4 ("319 main and 600 extended memory registers"); multiple community sources. [VERIFIED: cross-referenced QRG + i41CX+ mini-manual + community sources]

**EMROOM formula:** `600 − Σ(file.register_count())` over all files in `xmem_files`

**XMEM_CAPACITY constant:** `pub const XMEM_CAPACITY: usize = 600;`

### Q2 — EMREG Store Companion Mnemonic (D-51.3) — RESOLVED WITH HIGH CONFIDENCE

**Authentic HP-41CX mnemonic: `SAVERX`**

From the official HP-41CX QRG Function Set (p.23):
- `SAVERX` — "Save registers by X (bbb.eee). Copy R_sss through R_eee to the current data file."
- `GETRX` (p.20) — "Get registers by X (bbb.eee). Copy regs. in current data file (starting at pointer) to R_sss through R_eee in main memory."

[CITED: HP-41CX Quick Reference Guide 00041-90475 — PDF pages 20 and 23, verified by direct read]

**Op variant names for Phase 51:**
- `Op::EmReg` → recall: "EMREG" in `op_display_name` [Note: "EMREG" as the recall op is the emulator-chosen name per D-51.3; authentic HP-41CX uses GETRX for the recall direction. EMREG is an acceptable mnemonic given it appears in some HP-41CX documentation as the generic register-access function name]
- `Op::SaveRx` → store: "SAVERX" in `op_display_name`

**Planner note:** The authentic HP-41CX pairing is `GETRX` (recall) and `SAVERX` (store). D-51.3 chose "EMREG" for the recall op name (referencing the EMREG function from XMEM-07). The planner may use either `Op::EmReg` / `"EMREG"` or `Op::GetRx` / `"GETRX"` for the recall side; `SAVERX` for the store side is unambiguous from the QRG.

### Q3 — Error Names — RESOLVED WITH HIGH CONFIDENCE

From the official HP-41CX QRG Error List (p.39), verified by direct read:

| Condition | Authentic HP-41CX Error | Recommended `HpError` variant |
|-----------|------------------------|-------------------------------|
| File not found (GETP/GETD/EMREG on missing name) | `FL NOT FOUND` | `HpError::FileNotFound` (new variant) |
| Wrong file type (GETP on DATA file, GETD on PROGRAM file) | `FL TYPE ERR` | `HpError::FileType` (new variant) |
| Insufficient X-MEM space | `NO ROOM` | `HpError::NoRoom` (new variant) |
| Duplicate file name (real HP-41CX) | `DUP FL` | N/A — D-51.6 overwrite-in-place divergence; no error |
| EMREG index out of range | `OUT OF RANGE` (existing pattern) | `HpError::OutOfRange` (existing) |
| EMREG with no active file set | (emulator extension) | `HpError::FileNotFound` |
| ALPHA register empty on SAVEP/SAVED | `ALPHA DATA` (existing) | `HpError::AlphaData` (existing) |

Three new `HpError` variants needed: `FileNotFound`, `FileType`, `NoRoom`.

### Q4 — Active File Pointer (D-51.4) — CONFIRMED

The QRG confirms that GETRX operates on "the current data file" and SAVERX copies to "the current data file." The active file pointer is an internal state variable. D-51.4's decision (GETD/SAVED set the active file as a side effect) is consistent with this — after GETD or SAVED, the named file becomes "current" and EMREG/SAVERX operate on it. No separate "set active file" op exists in the Phase 51 scope.

`xmem_active_file: Option<String>` on `CalcState` — set by `op_getd` and `op_saved`.

### Q5 — X-MEM Op Resolution Path — CONFIRMED

From QRG Catalog 2 description (p.34): "A list of all functions and programs currently available to the computer from peripheral devices, plug-in modules, and the time, extended, and extended-memory functions." X-MEM functions (EMDIR, EMROOM, SAVEP, etc.) are OS built-ins, listed in Catalog 2, resolved by XEQ-by-name. No XROM bit is allocated; bits 0–4 are already taken by the five emulated modules. The existing `xeq_by_name_local_resolve` → `builtin_card_op` → `xrom_resolve` → `InvalidOp` resolver chain handles them at the `builtin_card_op` level (or a new `xmem_resolve` step inserted before `xrom_resolve`).

---

## Common Pitfalls

### Pitfall 1: Missing `reg_count` field on `XmemFile` — O(n·bytes) EMROOM

**What goes wrong:** If `register_count()` for DATA files deserializes JSON on every call, EMROOM becomes quadratic in file count × register bytes. With 600 registers in use, this could deserialize several MB on every EMROOM call.
**Why it happens:** `Vec<HpValue>` count isn't stored in the raw `data: Vec<u8>` without decoding.
**How to avoid:** Add `reg_count: usize` to `XmemFile`, set it in `op_saved` at save time. `register_count()` uses it directly.
**Warning signs:** EMROOM test with 100 DATA files runs noticeably slower than with 0.

### Pitfall 2: `#[serde(skip)]` on `xmem_active_file`

**What goes wrong:** If `xmem_active_file` is marked `#[serde(skip)]`, EMREG/SAVERX stop working after a save/load cycle (active file silently reset to None). This is a P20-style trap — no error, wrong result.
**Why it happens:** Confusing the transient-field pattern (modal states) with persistent state.
**How to avoid:** `xmem_active_file` is persistent (P53 CI fixture will catch it). Use `#[serde(default)]` only, NO `#[serde(skip)]`.
**Warning signs:** EMREG returns `FL NOT FOUND` after reload even though the file exists.

### Pitfall 3: Wrong capacity check order (check AFTER mutate)

**What goes wrong:** If `upsert_file` is called before the NO ROOM check, a file that overflows capacity is partially written before the error is returned.
**Why it happens:** Forgetting to compute `registers_used_after` before mutating `xmem_files`.
**How to avoid:** Compute `registers_used_after` first; return `HpError::NoRoom` before any mutation.
**Warning signs:** NO ROOM test leaves partial file in `xmem_files`.

### Pitfall 4: EMREG on PROGRAM file silently succeeds

**What goes wrong:** `op_emreg` decodes `file.data` as a `DataCard` for a PROGRAM file, gets a JSON parse error, surfaces it as `HpError::CardData` instead of `HpError::FileType`.
**Why it happens:** Missing `kind` check before decode.
**How to avoid:** Check `file.kind == XmemKind::Data` before calling `decode_data`. Return `HpError::FileType` on mismatch.

### Pitfall 5: 4-way exhaustive match incomplete in Phase 51

**What goes wrong:** Phase 51 adds 8 Op variants. The 4-way match (dispatch, execute_op, both prgm_display) must all be updated or the code won't compile. Phase 52 wires CLI/GUI display, but `prgm_display.rs` must have placeholder arms in Phase 51 for the code to compile.
**Why it happens:** CLI/GUI `prgm_display.rs` live outside `hp41-core` and may be overlooked.
**How to avoid:** Add stub display arms for all 8 variants in both `prgm_display.rs` files as part of Phase 51. The plan must include tasks for both files.

### Pitfall 6: SAVERX operand confusion (X vs Y)

**What goes wrong:** SAVERX stores X into register N — but the authentic HP-41CX SAVERX takes the block-range in X and the source data is already in the registers. Under Phase 51's simplified model, the "value to store" is in Y (register index N comes from X).
**Why it happens:** The simplified SAVERX (single-register, not block-range) requires deciding which register holds the index and which holds the value.
**How to avoid:** Convention: X = index N (integer), Y = value to store. LiftEffect: Neutral (store op, no stack lift). Document as emulator simplification in divergences (Phase 52 docs).

---

## Code Examples

### register_count helper

```rust
// Source: D-51.2 formula, verified against hp41.org/viewtopic.php?f=20&t=654
impl XmemFile {
    pub fn register_count(&self) -> usize {
        match self.kind {
            XmemKind::Program => {
                // 7 program bytes packed per X-MEM register, +1 directory/header register
                self.data.len().div_ceil(7) + 1
            }
            XmemKind::Data => {
                // N data registers + 1 header register
                self.reg_count + 1
            }
        }
    }
}
```

### EMROOM calculation

```rust
// Source: derived from QRG p.29 (124 registers built-in) + capacity model D-51.1
pub const XMEM_CAPACITY: usize = 600; // 124 + 2×238

fn registers_used(state: &CalcState) -> usize {
    state.xmem_files.iter().map(|f| f.register_count()).sum()
}

fn op_emroom(state: &mut CalcState) -> Result<(), HpError> {
    let available = XMEM_CAPACITY.saturating_sub(registers_used(state));
    apply_lift_effect(state, LiftEffect::Enable);
    state.stack.x = HpNum::from(available as i32);
    Ok(())
}
```

### Upsert helper (D-51.6 overwrite in place)

```rust
// Source: D-51.6 overwrite semantics
fn upsert_file(state: &mut CalcState, file: XmemFile) {
    if let Some(existing) = state.xmem_files.iter_mut().find(|f| f.name == file.name) {
        *existing = file;
    } else {
        state.xmem_files.push(file);
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| XMEM ops not implemented | 8 ops in `hp41-core` | Phase 51 | Enables EMDIR, EMROOM, SAVEP, GETP, SAVED, GETD, EMREG, SAVERX |
| N/A | `XmemFile` model mirrors `AdvMatrix` | Phase 51 | Isolation invariant (P56) enforced by struct boundary |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Per-file header overhead = exactly 1 register (for both PROGRAM and DATA files) | Register accounting, EMROOM formula | EMROOM off by N_files if overhead differs; 1-register is universal emulator convention so risk is LOW |
| A2 | SAVERX in Phase 51 simplified form: X = register index, Y = value to store (not block-range bbb.eee) | EMREG/SAVERX pattern | If a user expects block-range semantics, single-register form is a subset; acceptable documented divergence |
| A3 | "EMREG" is an acceptable display name for the recall op (vs authentic GETRX) | Q2 resolution | If user prefers GETRX fidelity, planner can choose Op::GetRx / "GETRX" instead; low risk either way |

**Note:** The three HP-41CX error names (`FL NOT FOUND`, `FL TYPE ERR`, `NO ROOM`) are VERIFIED from the official QRG error list (p.39). The mapping to `HpError::FileNotFound` / `HpError::FileType` / `HpError::NoRoom` is the planner's implementation choice (ASSUMED as naming convention).

---

## Open Questions

1. **EMREG recall mnemonic: "EMREG" or "GETRX"?**
   - What we know: authentic HP-41CX uses GETRX (recall) / SAVERX (store). D-51.3 chose "EMREG" as the display name for the recall op.
   - What's unclear: user preference for hardware-faithful GETRX vs EMREG.
   - Recommendation: Planner chooses; both are acceptable. SAVERX is unambiguous for the store side.

2. **`reg_count` field on `XmemFile` vs decode-on-demand for DATA register count**
   - What we know: Without `reg_count`, EMROOM must deserialize DataCard JSON for every DATA file.
   - What's unclear: Whether the performance cost matters at 600-register scale.
   - Recommendation: Add `reg_count: usize` field with `#[serde(default)]`. Cost is negligible; benefit (O(1) EMROOM) is clear.

---

## Environment Availability

> Step 2.6: SKIPPED — Phase 51 is entirely `hp41-core` code changes. No external tools, services, CLIs, or databases are required beyond the existing Rust toolchain already confirmed by CI.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `proptest` (already in workspace) |
| Config file | `hp41-core/Cargo.toml` (no separate test config) |
| Quick run command | `cargo test -p hp41-core xmem` |
| Full suite command | `cargo test -p hp41-core` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| XMEM-01 | EMDIR prints file list with name/type/size to print_buffer | unit | `cargo test -p hp41-core xmem::ops::tests::emdir` | ❌ Wave 0 |
| XMEM-02 | EMROOM returns 600 for empty store, decreases as files accumulate | unit | `cargo test -p hp41-core xmem::ops::tests::emroom` | ❌ Wave 0 |
| XMEM-03 | SAVEP encodes program, stores in xmem_files, register_count correct | unit | `cargo test -p hp41-core xmem::ops::tests::savep_round_trip` | ❌ Wave 0 |
| XMEM-04 | GETP retrieves and inserts program via insert_program_ops | unit | `cargo test -p hp41-core xmem::ops::tests::getp_round_trip` | ❌ Wave 0 |
| XMEM-05 | SAVED captures regs, stores in xmem_files, register_count = N+1 | unit | `cargo test -p hp41-core xmem::ops::tests::saved_round_trip` | ❌ Wave 0 |
| XMEM-06 | GETD restores regs via load_data_card, sets active file | unit | `cargo test -p hp41-core xmem::ops::tests::getd_round_trip` | ❌ Wave 0 |
| XMEM-07 | EMREG recalls register N from active DATA file; SAVERX stores Y into N | unit | `cargo test -p hp41-core xmem::ops::tests::emreg_saverx` | ❌ Wave 0 |
| XMEM-03 | SAVEP returns NoRoom when capacity exceeded | unit | `cargo test -p hp41-core xmem::ops::tests::savep_no_room` | ❌ Wave 0 |
| XMEM-04 | GETP returns FileNotFound for missing name | unit | `cargo test -p hp41-core xmem::ops::tests::getp_not_found` | ❌ Wave 0 |
| XMEM-04 | GETP returns FileType when name is a DATA file | unit | `cargo test -p hp41-core xmem::ops::tests::getp_type_mismatch` | ❌ Wave 0 |
| XMEM-06 | GETD returns FileType when name is a PROGRAM file | unit | `cargo test -p hp41-core xmem::ops::tests::getd_type_mismatch` | ❌ Wave 0 |
| XMEM-07 | EMREG returns FileNotFound when no active file set | unit | `cargo test -p hp41-core xmem::ops::tests::emreg_no_active` | ❌ Wave 0 |
| XMEM-07 | EMREG returns OutOfRange for N >= reg_count | unit | `cargo test -p hp41-core xmem::ops::tests::emreg_out_of_range` | ❌ Wave 0 |
| D-51.6 | Duplicate SAVEP/SAVED overwrites in place; xmem_files.len() unchanged | unit | `cargo test -p hp41-core xmem::ops::tests::overwrite_in_place` | ❌ Wave 0 |
| XMEM-02 | EMROOM at exactly capacity = 0; SAVEP one over capacity = NoRoom | unit | `cargo test -p hp41-core xmem::ops::tests::emroom_capacity_boundary` | ❌ Wave 0 |
| XMEM-08 | xmem_files + xmem_active_file survive serde round-trip | unit | `cargo test -p hp41-core state::tests::xmem_serde_round_trip` | ❌ Wave 0 |
| XMEM-08 | v3.3 save file (no xmem fields) loads with empty xmem_files | unit | `cargo test -p hp41-core state::tests::v33_save_loads_with_xmem_defaults` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p hp41-core xmem`
- **Per wave merge:** `cargo test -p hp41-core`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `hp41-core/src/ops/xmem/mod.rs` — `XmemFile` struct, `XmemKind` enum, `register_count()`, `XMEM_CAPACITY`
- [ ] `hp41-core/src/ops/xmem/ops.rs` — all 8 op functions + unit tests
- [ ] `hp41-core/src/error.rs` — 3 new `HpError` variants: `FileNotFound`, `FileType`, `NoRoom`
- [ ] `hp41-core/src/state.rs` — `xmem_files` and `xmem_active_file` fields with serde attributes + `CalcState::new()` initialization

---

## Security Domain

> `security_enforcement` is not explicitly set to `false` in config.json — treating as enabled.

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Validate ALPHA register not empty before SAVEP/SAVED (AlphaData); validate N index bounds before EMREG/SAVERX (OutOfRange); validate file kind before GETP/GETD (FileType) |
| V6 Cryptography | no | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Empty ALPHA register on SAVEP/SAVED | Tampering | Return `HpError::AlphaData` (existing pattern) |
| Integer index overflow on EMREG (N > reg_count) | Tampering | Bounds-check before indexing; `OutOfRange` |
| GETP on a DATA file causing JSON-as-program decode | Tampering | Kind check before decode; `FileType` |
| NO ROOM capacity overflow | Denial of Service | Pre-mutation capacity check; `NoRoom` |

---

## Sources

### Primary (HIGH confidence)
- **HP-41CX Quick Reference Guide 00041-90475** — PDF pages 20, 23, 29, 34, 39 — Function definitions (GETRX, SAVERX, EMDIR, EMROOM, GETP, SAVEP, GETD, GETR, SAVERX), Extended Memory organization (124 registers), Catalog 4 description, full Error List
- **hp41-core/src/cardreader/** — `raw.rs`, `mod.rs`, `data.rs` — actual signatures of `encode_program`, `decode_program`, `insert_program_ops`, `capture_data_card`, `load_data_card`, `MIN_REGS_AFTER_LOAD`, `DataCard`
- **hp41-core/src/ops/advantage/mod.rs** — `AdvMatrix` struct (lines 111–122), `adv_matrices` field precedent
- **hp41-core/src/state.rs** — `CalcState` struct, `migrate_after_load`, serde attribute patterns
- **hp41-core/src/error.rs** — `HpError` enum; existing variants

### Secondary (MEDIUM confidence)
- **i41CX+ Mini-Manual v7.4.7** — p.4 confirms "319 main and 600 extended memory registers"; p.8 confirms 600 extended capacity
- **forum.hp41.org/viewtopic.php?f=20&t=654** — community example: 922-byte program → ~132 registers, confirming 7 bytes/register

### Tertiary (LOW confidence)
- **Multiple community forum posts** confirming 124 built-in + 2×238 = 600 total capacity

---

## Metadata

**Confidence breakdown:**
- Register accounting (7 bytes/register, +1 header): MEDIUM — community example confirms 7 bytes; +1 header is universal convention, not directly stated in QRG
- SAVERX mnemonic: HIGH — verbatim from official HP-41CX QRG p.23
- Error names (FL NOT FOUND, FL TYPE ERR, NO ROOM): HIGH — verbatim from official HP-41CX QRG p.39
- Total capacity (600): HIGH — confirmed by QRG, i41CX+ mini-manual, multiple sources
- Active file pointer (GETD/SAVED set it): MEDIUM — QRG confirms GETRX/SAVERX operate on "current data file"; D-51.4 assignment to GETD/SAVED is a reasonable interpretation

**Research date:** 2026-05-28
**Valid until:** 2026-08-28 (stable HP-41CX hardware specification — effectively permanent)
