---
phase: 52-test-hardening-documentation
reviewed: 2026-05-28T10:00:00Z
depth: standard
files_reviewed: 12
files_reviewed_list:
  - hp41-core/src/ops/program.rs
  - hp41-cli/src/help_data.rs
  - hp41-gui/src/help_data.ts
  - hp41-gui/src/HelpOverlay.test.tsx
  - scripts/docs-matrix/src/main.rs
  - docs/hp41-xmem-functions.json
  - hp41-cli/tests/function_matrix_parity.rs
  - hp41-cli/tests/phase52_help_data_xmem.rs
  - hp41-cli/tests/xeq_builtin_resolver.rs
  - hp41-core/tests/xmem_backward_compat.rs
  - hp41-core/tests/xrom_op_test_count.rs
  - hp41-core/tests/fixtures/v33-autosave.json
findings:
  critical: 0
  warning: 4
  info: 5
  total: 9
status: issues_found
---

# Phase 52: Code Review Report

**Reviewed:** 2026-05-28T10:00:00Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

## Summary

Phase 52 wired 8 X-MEM built-in ops (EMDIR/EMROOM/SAVEP/GETP/SAVED/GETD/EMREG/SAVERX) into
the help system (CLI + GUI + JSON), the `builtin_card_op` resolver in `program.rs`, and the
meta-gate test `xrom_op_test_count.rs`. The backward-compatibility fixture and isolation tests
in `xmem_backward_compat.rs` look correct. The resolver wiring is architecturally sound and
consistent with how prior phases (card-reader ops, conditional tests) were handled.

No critical (data-loss / crash / security) defects were found. Four warnings and five info
items are documented below. The most substantive issues are:

1. A test in `xeq_builtin_resolver.rs` asserts X-MEM ops resolve through a resolver path
   that only delegates to `builtin_card_op` secondarily — correct by design, but the test
   comment misattributes the resolution mechanism in a way that will confuse future
   maintainers.
2. The `xmem_backward_compat.rs` backward-compat fixture uses fields (`adv_tvm_state`,
   `adv_matrices`, `stopwatch_*`) that are absent from the actual `v33-autosave.json`
   fixture, so some `serde(default)` paths exercised by the test comments are not actually
   exercised.
3. The `docs/hp41-xmem-functions.json` SAVED entry hardcodes "100 data registers" in its
   description, but the actual op saves `state.regs` which is sized dynamically by `SIZE`.
4. The `help_data.ts` `helpEntriesAll()` function allocates a new array on every call,
   unlike all other pools which are one-shot module-scope constants.

---

## Warnings

### WR-01: `xeq_builtin_resolver.rs` test `xmem_builtins_resolve` uses resolver with XROM modules 0b0000_0011 — X-MEM ops succeed for the wrong reason if the test logic ever changes

**File:** `hp41-cli/tests/xeq_builtin_resolver.rs:150-161`

**Issue:** The test fixture calls `xeq_by_name_local_resolve(name, 0b0000_0011)` (Math 1 + Stat 1 bits set, not all 5 bits). X-MEM ops like `"EMDIR"` resolve correctly because `xeq_by_name_local_resolve` first delegates to `builtin_card_op` before falling through to `xrom_resolve`, and `builtin_card_op` handles X-MEM names unconditionally (no bitfield check). However the test comment says:

```rust
// These resolve via builtin_card_op — no changes to keys.rs or key_map.rs needed.
```

The test does not verify that X-MEM ops return `None` when the resolver is called with a bitfield that _would_ matter if they were XROM ops, nor does it assert the resolution is bitfield-independent. The concern is that if someone in a future phase accidentally moves the X-MEM match arms from `builtin_card_op` into `xrom_resolve` (e.g. by adding an XROM bit for X-MEM), this test would keep passing because `0b0000_0011` happens to be a valid arbitrary bitfield — it would not catch the regression. The Phase 52 RESEARCH explicitly warns about this (Pitfall 1), but the test itself does not exercise that pitfall.

**Fix:** Add a complementary assertion that X-MEM ops resolve to `Some(...)` even with `xrom_modules = 0b0000_0000` (no XROM modules loaded), proving the resolution is independent of the XROM bitfield:

```rust
#[test]
fn xmem_builtins_are_xrom_module_independent() {
    // X-MEM ops are OS built-ins, NOT XROM. They must resolve regardless of
    // xrom_modules bitfield (D-52.4 / Pitfall 1 in RESEARCH.md).
    let resolve_no_xrom = |name: &str| {
        hp41_cli::keys::xeq_by_name_local_resolve(name, 0b0000_0000)
    };
    assert_eq!(resolve_no_xrom("EMDIR"), Some(EmDir));
    assert_eq!(resolve_no_xrom("EMROOM"), Some(EmRoom));
    assert_eq!(resolve_no_xrom("SAVEP"), Some(SaveP));
    assert_eq!(resolve_no_xrom("GETP"), Some(GetP));
    assert_eq!(resolve_no_xrom("SAVED"), Some(SaveD));
    assert_eq!(resolve_no_xrom("GETD"), Some(GetD));
    assert_eq!(resolve_no_xrom("EMREG"), Some(EmReg));
    assert_eq!(resolve_no_xrom("SAVERX"), Some(SaveRx));
}
```

---

### WR-02: `v33-autosave.json` fixture is missing fields present in v3.3 `CalcState`; backward-compat tests silently exercise fewer `serde(default)` paths than documented

**File:** `hp41-core/tests/fixtures/v33-autosave.json:1-140` / `hp41-core/tests/xmem_backward_compat.rs:46-87`

**Issue:** The fixture file `v33-autosave.json` ends at line 140 with only these fields present: `stack`, `regs`, `alpha_reg`, `alpha_mode`, `angle_mode`, `display_mode`, `entry_buf`, `program`, `prgm_mode`, `pc`, `call_stack`, `is_running`, `user_mode`, `key_assignments`, `assignments`, `text_regs`, `last_key_code`, `reg_m`, `reg_n`, `reg_o`, `flags`, `xrom_modules`, `complex_mode`, `rand_seed`, `matrix_dim`, `matrix_active_reg`, `time_offset_secs`, `clock_12h`.

Fields that exist in a real v3.3 `CalcState` but are absent from the fixture include `adv_tvm_state`, `adv_matrices`, `stopwatch_*`, `display_override`, `event_buffer`, `print_buffer`. The test `v33_save_xmem_fields_default_cleanly` asserts that `xmem_files` and `xmem_active_file` default correctly when absent — this works. However, the test comments in `xmem_backward_compat.rs:14` claim to "exercise `#[serde(default)]` paths from Phase 51" but the fixture is actually a trimmed-down snapshot, not a real v3.3 save (which would include `adv_tvm_state`, stopwatch fields, etc.). A genuine v3.3 autosave would have all those fields.

The consequence is limited: the `xmem_files` / `xmem_active_file` `serde(default)` paths are correctly tested. But the fixture does not represent a realistic v3.3 file, so the "v3.3 backward compat" label is misleading. A future regression that breaks deserialization of `adv_tvm_state` would not be caught here.

**Fix:** Either rename the fixture to `v33-minimal-autosave.json` and update comments to clarify it is a stripped-down state, or augment the fixture with a realistic set of v3.3 fields (including `adv_tvm_state: null`, `adv_matrices: {}`, stopwatch fields) so it mirrors what `autosave.json` actually produces in production.

---

### WR-03: `docs/hp41-xmem-functions.json` SAVED description hardcodes "100 registers" but the op saves `state.regs` which is dynamically sized by `SIZE`

**File:** `docs/hp41-xmem-functions.json:51-55`

**Issue:** The `SaveD` entry reads:

```json
"description": "Save all 100 data registers to X-MEM file named by ALPHA register",
"example": "\"REGS\" ALPHA, XEQ \"SAVED\" -> R00-R99 stored in X-MEM file REGS",
"notes": "Saves the full 100-register set. Use GETD to restore. ..."
```

On a real HP-41CX, SAVED always saves exactly the configured number of data registers (as set by SIZE), not a fixed 100. The emulator's `op_saved` captures `state.regs`, and `state.regs` is resized by `SIZE` (from 1 to 319 entries). An HP-41 with `SIZE 010` saves only 10 registers, not 100. The description, example, and notes all perpetuate the "100 registers" assumption.

The `test_pool_partition_is_exhaustive` test in `function_matrix_parity.rs` and the backup-compat tests in `xmem_backward_compat.rs` (`saved_getd_transfer_is_isolated`) also assume exactly 100 registers implicitly (the default `CalcState::new()` creates a 100-register set), but those are controlled test contexts. The public-facing documentation is wrong for users who have run `SIZE` to change the register count.

**Fix:**

```json
"description": "Save all data registers (current SIZE) to X-MEM file named by ALPHA register",
"example": "\"REGS\" ALPHA, XEQ \"SAVED\" -> current register set stored in X-MEM file REGS",
"notes": "Saves all registers in the current SIZE allocation. Use GETD to restore. Pairs with GETD for register transfer."
```

Similarly update the `GetD` entry:

```json
"description": "Load data registers from X-MEM file named by ALPHA register",
"notes": "Loads all registers stored in the data file. Existing registers are overwritten up to the stored count."
```

---

### WR-04: `helpEntriesAll()` in `help_data.ts` allocates a new array on every call; unlike the Rust `help_entries_all()` this is not cached

**File:** `hp41-gui/src/help_data.ts:246-248`

**Issue:** The TypeScript function:

```typescript
export function helpEntriesAll(): readonly HelpEntry[] {
    return [...helpEntries(), ...helpEntriesMath1(), ...helpEntriesStat1(), ...helpEntriesTime(), ...helpEntriesAdvantage(), ...helpEntriesXmem()];
}
```

This spread-concatenation runs on every call, constructing a fresh array each time. The individual pool functions (`helpEntries()`, etc.) return stable module-level bindings, so they are cheap — but the concatenation itself allocates a new array containing ~370 references per invocation. In `HelpOverlay.tsx`, this is called during render (potentially inside a `useEffect` or on every keystroke during search). The Rust counterpart `help_entries_all()` returns a lazy iterator (zero allocation); the TypeScript counterpart creates a full copy.

Unlike the performance category (which is out of v1 scope), this is also a correctness concern: if `helpEntriesAll()` is called inside a comparison or equality check (e.g. in a memoization guard), referential equality will always be false, potentially causing infinite re-render loops in React.

**Fix:** Memoize the concatenated result at module scope:

```typescript
const _allEntries: readonly HelpEntry[] = [
    ...helpEntries(),
    ...helpEntriesMath1(),
    ...helpEntriesStat1(),
    ...helpEntriesTime(),
    ...helpEntriesAdvantage(),
    ...helpEntriesXmem(),
];

export function helpEntriesAll(): readonly HelpEntry[] {
    return _allEntries;
}
```

This mirrors the Rust `OnceLock` pattern at module-evaluation time.

---

## Info

### IN-01: `docs/hp41-xmem-functions.json` — `SaveRx` description says "register 0" but the operation uses X as register index

**File:** `docs/hp41-xmem-functions.json:83-89`

**Issue:** The `SaveRx` notes field says "Stores X into the active X-MEM file's register 0" but the actual implementation uses `state.stack.x` as the register _index_ and `state.stack.y` as the value to store (per the test in `xmem_backward_compat.rs:244-253` which sets `state.stack.x = HpNum::from(2i32)` as the index). The description should clarify which stack register specifies the target index.

**Fix:**

```json
"notes": "X specifies the register index; Y contains the value to store. Creates the file if it does not exist."
```

---

### IN-02: `emdir_populated_catalog_appears_in_print_buffer` test contains a duplicated line

**File:** `hp41-core/tests/xmem_backward_compat.rs:150-151`

**Issue:** Lines 150–151 both assign `state.alpha_reg = "FILE2".to_string();`. This is a copy-paste leftover — the second assignment is dead code. It does not affect test correctness (the second op_savep uses "FILE2" correctly), but it signals sloppy cut-paste.

**Fix:** Remove the duplicate line 150 (`state.alpha_reg = "FILE2".to_string();`).

---

### IN-03: `docs-matrix/src/main.rs` `render_markdown` title for the unknown case contains a raw format-string literal `{json_path}` that is never interpolated

**File:** `scripts/docs-matrix/src/main.rs:83`

**Issue:**

```rust
} else {
    ("# Function Matrix", "`{json_path}`")
};
```

The string literal `` "`{json_path}`" `` is a plain `&str`, not a format string. The `{json_path}` placeholder will be emitted verbatim into the generated markdown file instead of being replaced with the actual path. This is a dead branch (all six known JSON filenames are matched above), but if a new JSON source is added without updating the match table, the generated output will contain the literal string `` `{json_path}` `` instead of the actual file name.

**Fix:**

```rust
} else {
    // Fallback: this branch fires only for unknown JSON sources.
    // `json_path` is not interpolated here — caller sees `{json_path}` literally.
    // TODO: convert to a format!() if this branch ever needs to be used.
    ("# Function Matrix", "unknown source")
};
```

Or, if a dynamic title is desired, restructure so that `title` and `src` can be owned `String`s (not `&str` literals), which requires a minor refactor of the return type.

---

### IN-04: `HelpOverlay.test.tsx` test at line 292-300 contradicts comment at lines 320-321

**File:** `hp41-gui/src/HelpOverlay.test.tsx:292-301` and `hp41-gui/src/HelpOverlay.test.tsx:320-321`

**Issue:** The test at line 292 is named "renders four top-level sections with HP-41CV, Math 1 Pac, Stat 1 Pac, and Time Pac headings" and only checks for 4 sections. But the adjacent test at line 316-321 has a comment:
```
// 7 sections: KEYBOARD SHORTCUTS (Phase 49) + HP-41CV + Math 1 + Stat 1 + Time + Adv (XROM 22) + Adv (XROM 24).
expect(sectionButtons.length).toBe(7);
```

The test at line 292 only checks for 4 headings by name and does not assert `sectionButtons.length`. This is silently inconsistent — the "four sections" test is stale naming (it was written before Advantage Pac and Keyboard Shortcuts were added) and will mislead a developer trying to understand the expected section count.

**Fix:** Rename the test at line 292 to reflect the current 7-section reality, or add a `.length` assertion that matches `7`. The test body as written is not wrong (it only checks a subset of button texts), but the description is outdated.

---

### IN-05: `function_matrix_parity.rs` `test_pool_partition_is_exhaustive` does not include X-MEM entries in its expected partition

**File:** `hp41-cli/tests/function_matrix_parity.rs:516-565`

**Issue:** The test `test_pool_partition_is_exhaustive` partitions `help_entries_all()` by `xrom.module_id`. X-MEM entries have `xrom: None`, so they fall into the `None` (built-in) bucket, which the test checks with `builtin_count >= 130`. With 8 X-MEM entries added to the pool in Phase 52, `builtin_count` will now be at least `138`. The `>= 130` floor still passes, but it no longer tightly bounds the built-in set.

The comment at `function_matrix_parity.rs:1067-1069` acknowledges this: "With 8 X-MEM entries added, builtin_count >= 138 — still above the >= 130 guard." However the guard does not distinguish between HP-41CV ROM built-ins (exactly `help_entries().len()`) and X-MEM built-ins (exactly 8). A future regression that accidentally duplicates X-MEM entries into the main pool, or loses them from the X-MEM pool into the built-in pool, would not be caught by the `>= 130` assertion.

**Fix:** Tighten the assertion to reflect the combined expected floor:

```rust
assert!(
    builtin_count >= 138,
    "built-in pool shrank below 138 (130 HP-41CV + 8 X-MEM): {builtin_count}"
);
```

Or, add a separate partition bucket for X-MEM entries by checking `entry.category == "Extended Memory"` when `xrom` is `None`.

---

_Reviewed: 2026-05-28T10:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
