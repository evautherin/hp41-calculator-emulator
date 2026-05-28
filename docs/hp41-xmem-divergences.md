# HP-41CX Extended Memory (X-MEM) Emulator Divergences

This document lists known behavioral divergences between this emulator's implementation
of the HP-41CX Extended Memory (X-Functions) and the hardware-faithful behavior
described in the HP-41CX Owner's Manual (HP 00041-90028, 1983), Appendix B.

**Status:** Established in Phase 52 / Plan 52-04 (XMEM-DOC-01).

**Philosophy:** Where divergences exist, this emulator prioritizes:
1. Hardware-faithful behavior where feasible.
2. User-safety (no silent data corruption without documentation).
3. Clear documentation of known divergences.

---

## How to Use This Document

Each entry carries a stable `D-52-NN` identifier that can be used in cross-references
from source-code comments, ADRs, test files, and issue trackers. The ID encodes the
phase (52 = Phase 52 / Plan 52-04) and an ordinal sequence number within this document.

Every entry uses five fixed fields (D-30.5 shape, carried forward as D-52 template):

- **OM citation** — The HP 00041-90028 Appendix B page-and-example that is the primary
  source, or `"N/A — emulator extension"` when no OM equivalent exists.
- **Our behavior** — What this emulator does.
- **OM behavior** — What the OM says or what real HP-41CX Extended Functions hardware does.
- **Rationale** — Why we made this choice (hardware-fidelity vs. pragmatic trade-off).
- **See** — Cross-references: ADR links, CONTEXT.md decision IDs, test file pointers.

The citation discipline (Pitfall 18 from `research/PITFALLS.md`, carried forward across
v3.x) requires every entry to carry at least one OM page reference, an explicit
`"N/A — emulator extension"` marker, or a primary-source citation. No uncited assertions
are permitted in this document.

X-MEM (Extended Memory) is the HP-41CX OS built-in file-storage system; entry numbering
follows the phase-origin convention established in v3.0 (`D-30-NN` for Math Pac I,
`D-35-NN` for Stat 1 Pac, `D-40-NN` for Time Pac, `D-45-NN` for Advantage Pac) per the
D-35.4 numbering scheme. The X-MEM catalog uses `D-52-NN` identifiers tied to Phase 52
(Test Hardening + Documentation).

---

## 1. OM Divergences

*(Behavioral mismatches with OM-described hardware behavior. These are cases where the
OM specifies or implies a particular outcome and our emulator intentionally diverges
from that specification.)*

---

### D-52-01: Overwrite-on-Duplicate — No "DUP FL" Error

- **OM citation**: HP-41CX Owner's Manual (HP 00041-90028, 1983), Appendix B — `SAVEP`
  and `SAVED` descriptions. The OM describes the "DUP FL" error condition: when a file
  with the same name already exists in Extended Memory, `SAVEP` and `SAVED` display
  "DUP FL" and halt. The user must first explicitly purge the existing file using `PURFL`
  (or `CLFL`, depending on firmware revision) before a new file with the same name can
  be saved.

- **Our behavior**: When `SAVEP` or `SAVED` is called with a file name (from the ALPHA
  register) that already exists in `state.xmem_files`, the existing file is silently
  overwritten in place — no error is raised, no confirmation is required. The overwrite
  preserves the file's position in the `xmem_files` vector; only the stored data changes.
  The active file pointer (`state.xmem_active_file`) is updated to the new file name.

- **OM behavior**: On real HP-41CX hardware, attempting to `SAVEP` or `SAVED` over an
  existing file raises "DUP FL" (duplicate file) and aborts. The user must call `PURFL`
  to explicitly purge the existing file before saving a new file under the same name.
  This was a design choice to prevent accidental data loss — the hardware forced an
  explicit "delete first, then save" workflow.

- **Rationale**: The Phase 51 op set has no purge/clear-file operation (`PURFL` / `CLFL`
  is not implemented). If the emulator raised "DUP FL" without providing `PURFL`, files
  would become permanently unreplaceable — a save would fail but no recovery path would
  exist. The overwrite-in-place behavior is the pragmatic alternative: it preserves full
  usability of `SAVEP`/`SAVED` without requiring a purge operation, and it avoids a user-
  experience trap where a file can never be updated. The divergence is intentional and
  temporary: a future phase that adds `PURFL`/`CLFL` can reinstate the hardware-faithful
  "DUP FL" error. Rejected alternative: raise "DUP FL" without implementing `PURFL` —
  rejected because it would render SAVEP/SAVED permanently broken for files that already
  exist, creating an unusable X-MEM system. The overwrite behavior prioritizes
  user-safety over hardware fidelity for this interim state.

- **See**: `hp41-core/src/ops/xmem/ops.rs::op_savep` and `::op_saved` (overwrite-in-place
  logic); `.planning/phases/51-x-mem-core/51-CONTEXT.md` D-51.6 (overwrite-on-duplicate
  decision); `.planning/phases/51-x-mem-core/51-CONTEXT.md` Deferred Ideas (PURFL/CLFL
  deferred — would enable true DUP FL semantics); `docs/adr/v4.0-001-xmem-os-builtin.md`
  (X-MEM OS built-in design overview).

---

### D-52-02: Full-Register-Set SAVED/GETD — No bbb.eee Block Control Word

- **OM citation**: HP-41CX Owner's Manual (HP 00041-90028, 1983), Appendix B — `SAVED`
  description. The OM specifies that before calling `SAVED`, the user places a value
  in the `bbb.eee` format in stack X: `bbb` is the beginning register number and `eee`
  is the ending register number for the transfer. For example, placing `5.010` in X
  before `XEQ "SAVED"` saves registers R05 through R10 into the DATA file. `GETD`
  likewise restores only the specified range. This partial-range "block control word"
  mechanism allows users to checkpoint selected registers without saving the entire
  register set.

- **Our behavior**: `SAVED` always saves the full register set R00..R(SIZE-1), equivalent
  to a `bbb.eee` value of `0.099` (R00 through R99) on the real hardware. The X register
  value is not parsed for a block control word — it is ignored during the transfer.
  `GETD` likewise always restores the full register set from the stored DATA file,
  padding to `MIN_REGS_AFTER_LOAD = 100` with zeros if the file contains fewer registers.
  Both operations reuse the `capture_data_card` / `load_data_card` helpers from
  `hp41-core/src/cardreader/mod.rs` — the same logic used by the card-reader `WRTD`/`REDD`
  operations — which implement the full-register-set model.

- **OM behavior**: On real HP-41CX hardware, `SAVED` reads the `bbb.eee` block control
  word from stack X and saves only registers in the specified range `[bbb, eee]`. `GETD`
  reads the same block control word and restores only that range, leaving registers outside
  the range unchanged. This partial-block mechanism allows efficient register checkpointing
  without storing or restoring the entire register file — useful for programs that use only
  a small number of working registers.

- **Rationale**: The full-register-set reuse model was chosen because (a) `capture_data_card`
  and `load_data_card` are battle-tested helpers with existing coverage, (b) implementing
  `bbb.eee` parsing correctly requires string-splitting at the decimal point following the
  ISG/DSE discipline (no `floor()`/`fmod()` per CLAUDE.md), range validation, and partial
  serialization — a non-trivial amount of new code with its own failure modes, and (c) the
  full-register-set model is simpler and predictable for users: SAVED always saves everything,
  GETD always restores everything. The partial-block transfer is deferred to a future phase
  (v4.1 candidate, alongside ASCII/STATUS file types). The divergence is intentional:
  programs written for real HP-41CX hardware that rely on partial `bbb.eee` transfers
  will experience different behavior in this emulator.

- **See**: `hp41-core/src/ops/xmem/ops.rs::op_saved` and `::op_getd` (full-set transfer
  implementation); `hp41-core/src/cardreader/mod.rs::capture_data_card`,
  `::load_data_card`, `MIN_REGS_AFTER_LOAD` (reused helpers);
  `.planning/phases/51-x-mem-core/51-CONTEXT.md` D-51.5 (full-register-set decision)
  and Deferred Ideas (bbb.eee block control word deferred to future phase);
  `docs/adr/v4.0-003-xmem-register-transfer.md` (full write-up of this design decision).

---

## 2. Emulator Extensions

*(Functions or behaviors added that are not present in the HP-41CX X-Functions OM spec.
These are deliberate, documented additions that improve usability or complete the design
without conflicting with OM behavior for OM-specified inputs.)*

No emulator extensions identified for the Phase 51/52 X-MEM implementation. The 8 ops
(`EMDIR`, `EMROOM`, `SAVEP`, `GETP`, `SAVED`, `GETD`, `EMREG`, `SAVERX`) implement the
OM-specified behavior for their respective functions, with the two documented divergences
(D-52-01, D-52-02) above. No XEQ-by-name aliases or additional operations were added
beyond the OM specification.

*If future phases add X-MEM operations or extensions not in the HP-41CX OM (e.g.,
`PURFL`/`CLFL` that would enable D-52-01 to be resolved), they will be documented here
as `D-52-07:` or later — the numbering reservation `D-52-03..D-52-06` is available for
future OM Divergence entries.*

---

## 3. Behavioral Policies

*(Cross-cutting rules that are decisions worth documenting — not strictly numerical
divergences, but intentional implementation choices with OM basis or deliberate extension.)*

No behavioral policy entries identified for Phase 51/52 X-MEM beyond the two divergences
documented above. The X-MEM implementation follows the HP-41CX OM for all observable
behaviors not covered by D-52-01 and D-52-02.

---

*Last updated: 2026-05-28. Catalog established in Plan 52-04 (Phase 52 / XMEM-DOC-01).*
