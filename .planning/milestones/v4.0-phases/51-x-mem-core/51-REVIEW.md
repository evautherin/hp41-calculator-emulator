---
phase: 51-x-mem-core
reviewed: 2026-05-28T00:00:00Z
depth: standard
files_reviewed: 8
files_reviewed_list:
  - hp41-cli/src/prgm_display.rs
  - hp41-core/src/error.rs
  - hp41-core/src/ops/mod.rs
  - hp41-core/src/ops/program.rs
  - hp41-core/src/ops/xmem/mod.rs
  - hp41-core/src/ops/xmem/ops.rs
  - hp41-core/src/state.rs
  - hp41-gui/src-tauri/src/prgm_display.rs
findings:
  critical: 0
  warning: 3
  info: 4
  total: 7
status: issues_found
---

# Phase 51: Code Review Report

**Reviewed:** 2026-05-28T00:00:00Z
**Depth:** standard
**Files Reviewed:** 8
**Status:** issues_found

## Summary

Phase 51 "X-MEM Core" adds an extended-memory module: the `XmemFile`/`XmemKind`
data model, two new `CalcState` fields (`xmem_files`, `xmem_active_file`), eight
new `Op` variants (EMDIR / EMROOM / SAVEP / GETP / SAVED / GETD / EMREG /
SAVERX), and the full 4-way exhaustive-match wiring.

Invariant compliance is strong:

- **4-way exhaustive match** is satisfied: all 8 variants appear in `dispatch()`
  (mod.rs:2172-2179), `execute_op()` (program.rs:1241-1248, routing through
  dispatch per the established XROM pattern), and both `op_display_name()` sites
  (cli prgm_display.rs:484-491, gui prgm_display.rs:502-509). Both display
  matches are exhaustive (no `_ =>` catch-all), so the compile-time invariant is
  load-bearing.
- **X-MEM isolation (D-51.0a)** holds: the ops module never reads/writes
  `state.regs` or `adv_matrices` except through the sanctioned
  `capture_data_card` / `load_data_card` transfer paths.
- **No-panic / no-unwrap**: no `.unwrap()` in production paths; the single
  `.expect("file was just validated")` (ops.rs:298) is the CLAUDE.md-sanctioned
  form. No `println!`/`eprintln!`.
- **ISG/DSE discipline**: register-count math uses `div_ceil` (mod.rs:50), index
  extraction uses `Decimal::trunc` via `trunc_int` (ops.rs:77) — no
  `floor()`/`fmod()`.
- **serde backward-compat**: both new fields carry `#[serde(default)]`; the
  documented "default WITHOUT skip" shape on `xmem_active_file` is correct and
  test-guarded (state.rs:1184-1294).

No correctness/data-loss/security blockers were found. The findings below are
contract-vs-behavior mismatches, an untested branch, and maintainability items.

Scope note (not a defect): the X-MEM ops are NOT registered in `xrom_resolve`
and NOT wired into any CLI/GUI keyboard or `key_map` path, so they are currently
reachable only via direct `dispatch()`. This is consistent with a "Core" phase
that defers user-facing wiring — flagged here only so downstream phases do not
mistake the absence for a regression.

## Warnings

### WR-01: `index_from_x` silently truncates non-integer indices despite docs promising rejection

**File:** `hp41-core/src/ops/xmem/ops.rs:73-82` (and callers `op_emreg` :253, `op_saverx` :282)
**Issue:** The doc-comment on `index_from_x` states "Returns `Err(HpError::OutOfRange)` if the value is negative **or non-integer**", and both `op_emreg` (line 251) and `op_saverx` (line 280) repeat the "N is negative, **non-integer**, or >= reg count → OutOfRange" contract. The implementation does NOT honor this: it calls `state.stack.x.trunc_int()` first, which truncates toward zero (e.g. `2.9 → 2`, `2.0001 → 2`), then `to_usize()`. A non-integer index like `2.9` is therefore silently accepted as index `2`, not rejected. Only negative values (which `to_usize()` rejects with `None`) are caught. This is a contract violation that can mask a programming error: a user who computes a fractional index expecting an error instead reads/writes the wrong register with no diagnostic.

```rust
fn index_from_x(state: &CalcState) -> Result<usize, HpError> {
    let original = state.stack.x;                 // capture before truncation
    let truncated = original.trunc_int();
    // Reject non-integer per the documented contract.
    if truncated != original {
        return Err(HpError::OutOfRange);
    }
    truncated
        .inner()
        .to_usize()
        .ok_or(HpError::OutOfRange)
}
```

Alternatively, if silent truncation is the intended HP-41CX-faithful behavior, fix the three doc-comments to say "non-integer values are truncated toward zero" so the documented contract matches reality.

### WR-02: GETP "RDPRGM semantics" insert-into-non-empty-program path is untested

**File:** `hp41-core/src/ops/xmem/ops.rs:170-180` (test `getp_round_trip` :413-428)
**Issue:** `op_getp` delegates to `insert_program_ops`, which REPLACES the program when `state.program` is empty but INSERTS after `state.pc` when it is non-empty (cardreader/mod.rs:44-54). The Op doc and function doc both promise "RDPRGM semantics", and the insert branch is where RDPRGM's subtle behavior lives. However, the only GETP test (`getp_round_trip`) clears the program (`state.program = vec![]`) before calling GETP, so it exercises ONLY the replace branch. The insert-into-non-empty branch — including correct interaction with `state.pc` and ordering of inserted ops — is unverified for the X-MEM entry point. A regression in the insert path (e.g. pc off-by-one, ordering) would not be caught by this phase's tests.
**Fix:** Add a test that pre-populates `state.program` with a labelled block and a non-trivial `state.pc`, then GETPs a saved program and asserts the merged ordering and resulting `pc`, mirroring `cardreader::tests::insert_into_nonempty_inserts_after_pc`.

### WR-03: `op_savep` / `op_saved` capacity arithmetic silently clamps an over-capacity store baseline via `saturating_sub`

**File:** `hp41-core/src/ops/xmem/ops.rs:149-154` (SAVEP) and `:206-211` (SAVED)
**Issue:** The capacity guard computes
`registers_used(state).saturating_sub(existing_file_regs(state, &name)).saturating_add(new_file.register_count())`.
The `saturating_sub` is correct for the overwrite case, but if the persisted store is ALREADY over capacity (a corrupted/hand-edited `~/.hp41/autosave.json` whose files sum to > 600 registers — possible because nothing validates `xmem_files` on load, and `migrate_after_load` does not touch them), the arithmetic can under-report `used_after` and permit a write that pushes the store further past 600. The same blind spot exists in EMROOM/EMDIR where `XMEM_CAPACITY.saturating_sub(registers_used)` clamps the displayed free count to 0, hiding the inconsistency rather than surfacing it.
**Fix:** Either validate/clamp `xmem_files` in `migrate_after_load()` so the store can never load over capacity, or make the guard reject when the pre-existing `registers_used(state)` already exceeds `XMEM_CAPACITY` (return `HpError::NoRoom` before computing the delta). At minimum add a test that loads an over-capacity store and asserts SAVEP/SAVED reject.

## Info

### IN-01: `op_saverx` triple-lookup (validate, re-find with `expect`, then `find_file_mut`) is fragile

**File:** `hp41-core/src/ops/xmem/ops.rs:288-307`
**Issue:** SAVERX looks the file up three times: once to validate kind (lines 290-294), once with `.expect("file was just validated")` to clone the data (lines 296-299), and a third time via `find_file_mut` to write back (line 305). The `.expect()` is sanctioned by CLAUDE.md, but the pattern is brittle — the invariant "validated then still present" is only locally true and easy to break under future edits (e.g. if a helper between the calls mutates `xmem_files`). The final `find_file_mut` also silently no-ops on `None` (line 305 `if let Some`), discarding a write that should be impossible to lose.
**Fix:** Collapse to a single mutable borrow: `find_file_mut` once, validate `kind` on that borrow, decode `&file.data`, mutate, re-encode, assign `file.data`. This removes both the `expect` and the silent-no-op `if let`.

### IN-02: SAVERX does not update `reg_count`; relies on the encode round-trip preserving register count

**File:** `hp41-core/src/ops/xmem/ops.rs:300-307`
**Issue:** SAVERX decodes, mutates `card.registers[n]` in place, re-encodes, and writes back `file.data` but leaves `file.reg_count` untouched. This is correct ONLY because `get_mut(n)` replaces an element without changing `card.registers.len()`. The invariant (data byte length ↔ `reg_count`) is implicit and undocumented at the write-back site. A future change that, say, grows the register vector inside SAVERX would silently desync `reg_count` from the encoded data, corrupting EMROOM/EMDIR accounting.
**Fix:** Add a one-line comment at the write-back asserting `reg_count` is unchanged because element count is preserved, or defensively re-set `file.reg_count = card.registers.len()` after re-encoding.

### IN-03: EMDIR free-register footer can render a misleading count under a corrupted store

**File:** `hp41-core/src/ops/xmem/ops.rs:94-113`
**Issue:** EMDIR prints `"{} REGS FREE"` using `XMEM_CAPACITY.saturating_sub(registers_used(state))`. Same root cause as WR-03: if `registers_used` exceeds 600, the footer shows `0 REGS FREE` rather than indicating the store is over-allocated. Cosmetic, but it can mask the WR-03 condition during debugging.
**Fix:** Once WR-03 is addressed (load-time validation), this becomes moot; otherwise consider surfacing an over-capacity indicator.

### IN-04: `XMEM_CAPACITY` magic decomposition lives only in a comment

**File:** `hp41-core/src/ops/xmem/mod.rs:56-58`
**Issue:** `pub const XMEM_CAPACITY: usize = 600;` encodes `124 + 2×238` only in the doc-comment. If the module count is ever made configurable (e.g. a single 82181A module), the literal `600` and the comment will drift. Minor — the value is test-guarded (`xmem_capacity_value`).
**Fix:** Optionally express as `const XMEM_BUILTIN: usize = 124; const XMEM_MODULE: usize = 238; pub const XMEM_CAPACITY: usize = XMEM_BUILTIN + 2 * XMEM_MODULE;` so the arithmetic is machine-checked.

---

_Reviewed: 2026-05-28T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
