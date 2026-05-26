# Phase 44: hp41-cli CLI Integration - Research

**Researched:** 2026-05-26
**Domain:** Rust hp41-cli — fifth JSON canonical source, 5-pool help chain, modal routing for AdvantageStep, partition test extension for dual-XROM, xrom_shadowing extension
**Confidence:** HIGH

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ADV-CLI-01 | `docs/hp41-advantage-functions.json` as fifth JSON canonical source | Verified pattern from stat1 + time JSON files; schema with dual module_ids (22, 24) |
| ADV-CLI-02 | Fifth `OnceLock<Vec<HelpEntry>>` in `help_data.rs` + 5-pool chain | help_data.rs line 183 already documents the fifth-pool extension point; exact code pattern verified |
| ADV-CLI-03 | All `op_display_name` arms in `prgm_display.rs` | ALREADY DONE — 117 arms present in commit 7c28d2b; confirmed by grep |
| ADV-CLI-04 | `?` help overlay "Advantage Pac (XROM 22)" and "Advantage Pac (XROM 24)" sections | Auto-derived from JSON category prefixes; no explicit code beyond the pool chain |
| ADV-CLI-05 | `function_matrix_parity.rs` 5-pool partition test | Partition guard must accept module_ids 22 and 24; exact extension pattern verified from time module addition |
| ADV-CLI-06 | `xrom_shadowing.rs` extended to ADV_A.ops + ADV_B.ops | Template pattern verified from stat1 + time additions to `hp41-core/tests/xrom_shadowing.rs` |
| ADV-CLI-07 | Modal-prompt routing for MATRX/MTR/TVM/MEDIT/CMEDIT/VE | `ModalProgram::Advantage(AdvantageStep)` wired in Phase 43; `maybe_auto_open_collect_for_modal` handles alpha-label steps; no new app.rs infrastructure needed |
| ADV-CLI-08 | Right-panel filter unchanged | `entry.xrom.is_none()` filter in `keys.rs` already excludes all XROM entries automatically |

</phase_requirements>

---

## Summary

Phase 44 is a structural-pattern phase: every task follows a well-worn path established by Phase 34 (Stat 1 CLI) and Phase 39 (Time CLI). The codebase already contains the extension points; the work is filling them in for the Advantage Pac's two XROM modules (id=22 ADV CONV+MTRX, id=24 ADV MATH+TVM).

The single architectural wrinkle distinguishing Phase 44 from prior CLI integration phases is the **dual-XROM structure**: prior modules (Math 1, Stat 1, Time) each had one XROM ID; Advantage Pac has two (22 and 24). This affects two files: (1) `function_matrix_parity.rs` partition guard must accept both module_ids, and (2) `xrom_shadowing.rs` must grow two new module-disjointness test blocks rather than one.

ADV-CLI-03 is confirmed complete: `hp41-cli/src/prgm_display.rs` already has 117 `Op::Adv*` arms (confirmed by grep; 114 XROM-exposed variants + 3 internal run-loop variants). The planner must NOT include any task for prgm_display.rs.

The modal infrastructure from Phase 43 (the `ModalProgram::Advantage(AdvantageStep)` variant) is already wired into `math1/modal.rs`. The CLI's `maybe_auto_open_collect_for_modal` already handles the alpha-label steps (`MatrixNamePrompt`, `MtrNamePrompt`, `FdifeqFunctionNamePrompt`) via the generic `requires_alpha_label()` delegation chain. No new code in `app.rs` is needed for modal routing.

**Primary recommendation:** Work in two waves. Wave 1: JSON authoring + help_data.rs + tests (ADV-CLI-01, ADV-CLI-02, ADV-CLI-04, ADV-CLI-05, ADV-CLI-06). Wave 2: validation + smoke tests. The xrom_shadowing extension belongs in Wave 1 since it gates JSON correctness.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| JSON canonical source | hp41-cli docs/ | docs-matrix script | `include_str!` at compile time; 5th invocation in justfile |
| Help overlay pooling | hp41-cli help_data.rs | ui.rs (consumer) | OnceLock chain; 5th `.chain()` arm in `help_entries_all()` |
| Program display names | hp41-cli prgm_display.rs | — | ALREADY DONE — 117 arms |
| Modal prompt rendering | hp41-cli ui.rs | hp41-core modal.rs | `pending_prompt()` calls `modal_program.current_prompt()` — no change needed |
| Modal routing (alpha labels) | hp41-cli app.rs | hp41-core advantage/modal.rs | `maybe_auto_open_collect_for_modal` delegates to `requires_alpha_label` — already wired |
| Partition CI gate | hp41-cli tests/ | — | `function_matrix_parity.rs` `test_pool_partition_is_exhaustive` needs two new arms |
| Shadowing CI gate | hp41-core tests/ | — | `xrom_shadowing.rs` two new test blocks |
| Right-panel exclusion | hp41-cli keys.rs | — | `entry.xrom.is_none()` filter — unchanged, automatic |
| docs-matrix generation | scripts/docs-matrix/ | justfile | 5th invocation + title dispatch |

---

## Standard Stack

No new external dependencies. Phase 44 uses only:
- `hp41-cli` crate (the subject of the phase)
- `hp41-core` crate (already a dependency)
- `serde_json` (already used via `help_data.rs`)
- `std::sync::OnceLock` (stable std, already used)

**Installation:** none — zero new runtime deps per the project invariant.

---

## Package Legitimacy Audit

No packages to install. Section skipped (zero new runtime dependencies).

---

## Architecture Patterns

### Pattern 1: Fifth OnceLock Pool (ADV-CLI-02)

The established four-pool pattern in `hp41-cli/src/help_data.rs` (lines 20–195) is extended by:

1. Adding a `const ADV_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-advantage-functions.json");`
2. Adding a `static ADV_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();`
3. Adding a `pub fn help_entries_adv() -> &'static [HelpEntry]` accessor with the `D-25.17` panic message
4. Extending `help_entries_all()` with `.chain(help_entries_adv().iter())`

The existing comment at line 183 already documents this extension: "Future XROM modules (Advantage Pac) extend by appending a fifth `.chain()` arm."

**Exact code pattern (from time pool addition at lines 149–195):**

```rust
// [VERIFIED: codebase — hp41-cli/src/help_data.rs:149-195]
const ADV_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-advantage-functions.json");
static ADV_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

pub fn help_entries_adv() -> &'static [HelpEntry] {
    ADV_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(ADV_FUNCTIONS_JSON)
            .expect("hp41-advantage-functions.json is malformed — fix the JSON")
    })
}

// In help_entries_all():
pub fn help_entries_all() -> impl Iterator<Item = &'static HelpEntry> {
    help_entries()
        .iter()
        .chain(help_entries_math1().iter())
        .chain(help_entries_stat1().iter())
        .chain(help_entries_time().iter())
        .chain(help_entries_adv().iter())   // ← fifth arm
}
```

### Pattern 2: JSON Schema (ADV-CLI-01)

The `HelpEntry` schema (defined in `help_data.rs` lines 64–80) has not changed since Phase 29. Every entry needs:

```json
{
  "op_variant": "AdvBinin",
  "display_name": "BININ",
  "category": "Adv Conv",
  "status": "implemented",
  "phase": "43",
  "key_path": "XEQ \"BININ\"",
  "description": "Enter binary integer from ALPHA register into X (36-bit)",
  "xrom": {
    "module": "Adv Conv",
    "module_id": 22,
    "function_id": 1
  }
}
```

**Key JSON rules (verified from stat1 + time precedent):**

- `op_variant`: PascalCase matching the exact `Op::` enum variant name (e.g., `"AdvBinin"`)
- `display_name`: HP mnemonic string as it appears in `prgm_display.rs` (e.g., `"BININ"`)
- `category`: Must start with a consistent prefix. PROPOSED: `"Adv Conv"`, `"Adv Mtrx"`, `"Adv Math"`, `"Adv TVM"` — or use sub-categories per operation group. The exact prefix is the key decision (see Pitfalls section).
- `status`: `"implemented"` for all 114 entries (all Phase 43 ops are shipped)
- `phase`: `"43"` (the phase that implemented them)
- `key_path`: `XEQ "MNEMONIC"` form for all entries (no dedicated key bindings; XROM-only)
- `xrom.module_id`: **22** for ADV CONV+MTRX entries (63 entries), **24** for ADV MATH+TVM entries (51 entries)
- `xrom.function_id`: 1-based dense range SEPARATELY per module_id (ADV_MATH_A: 1..=63, ADV_MATH_B: 1..=51)
- `divergences`: Optional array; only add for entries with documented behavioral differences

**ADV_MATH_A entries (XROM 22, function_ids 1–63):**

| # | Mnemonic | Op variant | Group |
|---|----------|-----------|-------|
| 1 | BININ | AdvBinin | ADV CONV |
| 2 | BINVIEW | AdvBinview | ADV CONV |
| 3 | OCTIN | AdvOctin | ADV CONV |
| 4 | HEXIN | AdvHexin | ADV CONV |
| 5 | HEXVIEW | AdvHexview | ADV CONV |
| 6 | CVTVIEW | AdvCvtview | ADV CONV |
| 7 | NOT | AdvNot | ADV CONV |
| 8 | AND | AdvAnd | ADV CONV |
| 9 | OR | AdvOr | ADV CONV |
| 10 | XOR | AdvXor | ADV CONV |
| 11 | ROTXY | AdvRotxy | ADV CONV |
| 12 | BIT? | AdvBitTest | ADV CONV |
| 13 | I+ | AdvIPlus | ADV MTRX |
| 14 | I- | AdvIMinus | ADV MTRX |
| 15 | J+ | AdvJPlus | ADV MTRX |
| 16 | J- | AdvJMinus | ADV MTRX |
| 17 | MR | AdvMr | ADV MTRX |
| 18 | MS | AdvMs | ADV MTRX |
| 19 | MRIJ | AdvMrij | ADV MTRX |
| 20 | MSIJ | AdvMsij | ADV MTRX |
| 21 | MSIJR | AdvMsijr | ADV MTRX |
| 22 | MRC+ | AdvMrcPlus | ADV MTRX |
| 23 | MRC- | AdvMrcMinus | ADV MTRX |
| 24 | MRR+ | AdvMrrPlus | ADV MTRX |
| 25 | MRR- | AdvMrrMinus | ADV MTRX |
| 26 | MSR+ | AdvMsrPlus | ADV MTRX |
| 27 | MSC+ | AdvMscPlus | ADV MTRX |
| 28 | MSWAP | AdvMswap | ADV MTRX |
| 29 | MNAME? | AdvMnameQuery | ADV MTRX |
| 30 | DIM? | AdvDimQuery | ADV MTRX |
| 31 | MATDIM | AdvMatdim | ADV MTRX |
| 32 | MP | AdvMp | ADV MTRX |
| 33 | PIV | AdvPiv | ADV MTRX |
| 34 | R<>R | AdvRExchangeR | ADV MTRX |
| 35 | R>R? | AdvRGtRQuery | ADV MTRX |
| 36 | SUM | AdvSum | ADV MTRX |
| 37 | SUMAB | AdvSumab | ADV MTRX |
| 38 | MAX | AdvMax | ADV MTRX |
| 39 | MAXAB | AdvMaxab | ADV MTRX |
| 40 | MIN | AdvMin | ADV MTRX |
| 41 | RMAXAB | AdvRmaxab | ADV MTRX |
| 42 | RNRM | AdvRnrm | ADV MTRX |
| 43 | RSUM | AdvRsum | ADV MTRX |
| 44 | FNRM | AdvFnrm | ADV MTRX |
| 45 | MDET | AdvMdet | ADV MTRX |
| 46 | MINV | AdvMinv | ADV MTRX |
| 47 | MSYS | AdvMsys | ADV MTRX |
| 48 | M*M | AdvMMulM | ADV MTRX |
| 49 | MAT+ | AdvMatPlus | ADV MTRX |
| 50 | MAT- | AdvMatMinus | ADV MTRX |
| 51 | MAT*C | AdvMatScalarMul | ADV MTRX |
| 52 | MAT/C | AdvMatScalarDiv | ADV MTRX |
| 53 | TRNPS | AdvTrnps | ADV MTRX |
| 54 | MMOVE | AdvMmove | ADV MTRX |
| 55 | C<>C | AdvCExchangeC | ADV MTRX |
| 56 | CMAXAB | AdvCmaxab | ADV MTRX |
| 57 | CNRM | AdvCnrm | ADV MTRX |
| 58 | CSUM | AdvCsum | ADV MTRX |
| 59 | YC+C | AdvYcPlusC | ADV MTRX |
| 60 | MATRX | AdvMatrx | ADV MTRX |
| 61 | MTR | AdvMtr | ADV MTRX |
| 62 | MEDIT | AdvMedit | ADV MTRX |
| 63 | CMEDIT | AdvCmedit | ADV MTRX |

**ADV_MATH_B entries (XROM 24, function_ids 1–51):**

| # | Mnemonic | Op variant | Group |
|---|----------|-----------|-------|
| 1 | E^Z | AdvExpZ | ADV MATH |
| 2 | LNZ | AdvLnZ | ADV MATH |
| 3 | LOGZ | AdvLogZ | ADV MATH |
| 4 | Z^N | AdvZPowN | ADV MATH |
| 5 | Z^1/N | AdvZPow1n | ADV MATH |
| 6 | Z^W | AdvZPowW | ADV MATH |
| 7 | Z^1/W | AdvZPow1w | ADV MATH |
| 8 | \|Z\| | AdvMagz | ADV MATH |
| 9 | SINZ | AdvSinZ | ADV MATH |
| 10 | COSZ | AdvCosZ | ADV MATH |
| 11 | TANZ | AdvTanZ | ADV MATH |
| 12 | A^Z | AdvAPowZ | ADV MATH |
| 13 | CADD | AdvCPlus | ADV MATH |
| 14 | CSUB | AdvCMinus | ADV MATH |
| 15 | CINV | AdvCinv | ADV MATH |
| 16 | CMUL | AdvCMul | ADV MATH |
| 17 | CDIV | AdvCDiv | ADV MATH |
| 18 | AIP | AdvAip | ADV MATH |
| 19 | PLY | AdvPly | ADV MATH |
| 20 | RTS | AdvRts | ADV MATH |
| 21 | FSOLVE | AdvFsolve | ADV MATH |
| 22 | FINTG | AdvFintg | ADV MATH |
| 23 | FDIFEQ | AdvFdifeq | ADV MATH |
| 24 | FROOT | AdvFroot | ADV MATH |
| 25 | CFIT | AdvCfit | ADV MATH |
| 26 | AS | AdvAs | ADV MATH |
| 27 | DS | AdvDs | ADV MATH |
| 28 | BFIT | AdvBfit | ADV MATH |
| 29 | FIT | AdvFit | ADV MATH |
| 30 | Y?X | AdvYQueryX | ADV MATH |
| 31 | SZ? | AdvSzQuery | ADV MATH |
| 32 | V+ | AdvVPlus | ADV MATH |
| 33 | V- | AdvVMinus | ADV MATH |
| 34 | DOT | AdvDot | ADV MATH |
| 35 | CROSS | AdvCross | ADV MATH |
| 36 | VC | AdvVc | ADV MATH |
| 37 | VS | AdvVs | ADV MATH |
| 38 | VR | AdvVr | ADV MATH |
| 39 | VE | AdvVe | ADV MATH |
| 40 | VXY | AdvVxy | ADV MATH |
| 41 | UV | AdvUv | ADV MATH |
| 42 | \|V\| | AdvVMag | ADV MATH |
| 43 | V* | AdvVStar | ADV MATH |
| 44 | VD | AdvVd | ADV MATH |
| 45 | TR | AdvTr | ADV MATH |
| 46 | TVM | AdvTvm | ADV TVM |
| 47 | N | AdvTvmN | ADV TVM |
| 48 | PV | AdvTvmPv | ADV TVM |
| 49 | PMT | AdvTvmPmt | ADV TVM |
| 50 | FV | AdvTvmFv | ADV TVM |
| 51 | *I | AdvTvmStarI | ADV TVM |

**INTERNAL-ONLY variants (NOT in JSON, NOT in XROM tables):**
- `AdvFsolveRunLoop` — internal run-loop re-entry variant
- `AdvFintgRunLoop` — internal run-loop re-entry variant
- `AdvFdifeqRunLoop` — internal run-loop re-entry variant

These three are in `prgm_display.rs` (displaying as `"FSOLVE"`, `"FINTG"`, `"FDIFEQ"` respectively) but must NOT have JSON entries. The `function_matrix_parity.rs` inventory (`ADV_A_OP_VARIANT_NAMES` + `ADV_B_OP_VARIANT_NAMES`) must likewise NOT include them.

### Pattern 3: Partition Test Extension (ADV-CLI-05)

The current `test_pool_partition_is_exhaustive` in `function_matrix_parity.rs` (lines 515–552) accepts module_ids `{None, 7, 2, 26}` and flags any other as `rogue`. Adding Advantage Pac entries with module_ids 22 and 24 will fail this test unless updated.

**Required changes to `function_matrix_parity.rs`:**

```rust
// [VERIFIED: codebase — function_matrix_parity.rs:515-552 pattern]

// Add two new inventory constants:
const ADV_A_OP_VARIANT_NAMES: &[&str] = &[
    // 63 entries matching ADV_MATH_A.ops (no run-loops)
    "AdvBinin", "AdvBinview", ...
];

const ADV_B_OP_VARIANT_NAMES: &[&str] = &[
    // 51 entries matching ADV_MATH_B.ops (no run-loops)
    "AdvExpZ", "AdvLnZ", ...
];

// Extend test_pool_partition_is_exhaustive match:
match entry.xrom.as_ref().map(|x| x.module_id) {
    None => builtin_count += 1,
    Some(7) => math1_count += 1,
    Some(2) => stat1_count += 1,
    Some(26) => time_count += 1,
    Some(22) => adv_a_count += 1,   // ← new
    Some(24) => adv_b_count += 1,   // ← new
    Some(other) => rogue.push(...),
}

// Add assertions:
assert_eq!(adv_a_count, 63, "ADV CONV+MTRX pool count drift: {adv_a_count}");
assert_eq!(adv_b_count, 51, "ADV MATH+TVM pool count drift: {adv_b_count}");

// Also add inventory count sentinels:
#[test]
fn test_adv_a_op_inventory_count() {
    assert_eq!(ADV_A_OP_VARIANT_NAMES.len(), 63, ...);
}
#[test]
fn test_adv_b_op_inventory_count() {
    assert_eq!(ADV_B_OP_VARIANT_NAMES.len(), 51, ...);
}
```

Also add forward/reverse parity tests per the established pattern (see `test_every_time_rom_op_has_time_json_entry` / `test_every_time_json_entry_has_xrom_resolver_match`).

The **reverse parity test** calls `xrom_resolve(display_name, 0b0001_1111)` (all 5 modules loaded = default v3.3 bitfield).

### Pattern 4: xrom_shadowing Extension (ADV-CLI-06)

The shadowing test lives in `hp41-core/tests/xrom_shadowing.rs`. The current import is:
```rust
use hp41_core::ops::math1::xrom::{xrom_resolve, MATH_1, STAT_1, TIME_MODULE};
```

Extend the import:
```rust
use hp41_core::ops::math1::xrom::{xrom_resolve, ADV_MATH_A, ADV_MATH_B, MATH_1, STAT_1, TIME_MODULE};
```

Add four new test functions following the exact `stat1_*` / `time_*` pattern:

1. `adv_a_names_do_not_shadow_builtins` — no ADV_MATH_A mnemonic in `BUILTIN_CARD_OP_NAMES`
2. `adv_a_ops_disjoint_from_prior_modules` — no ADV_MATH_A mnemonic in MATH_1 ∪ STAT_1 ∪ TIME_MODULE ops
3. `adv_a_ops_resolve_via_xrom_resolve` — `xrom_resolve(name, 0b0001_1111)` matches declared op
4. `adv_a_const_fields` — id==22, name=="ADV CONV"
5. `adv_b_names_do_not_shadow_builtins`
6. `adv_b_ops_disjoint_from_prior_modules_and_adv_a`
7. `adv_b_ops_resolve_via_xrom_resolve` — `xrom_resolve(name, 0b0001_1111)` matches declared op
8. `adv_b_const_fields` — id==24, name=="ADV MATH"

The xrom_shadowing tests live in **hp41-core** (not hp41-cli), matching the pattern for prior modules.

### Pattern 5: Modal Routing (ADV-CLI-07)

**No new infrastructure needed in app.rs.** The `ModalProgram::Advantage(AdvantageStep)` variant was added to `hp41-core/src/ops/math1/modal.rs` in Phase 43. The CLI's modal machinery works via:

1. `submit_modal` (`F5` key) → `hp41_core::ops::math1::submit_modal(&mut self.state)` — already handles `ModalProgram::Advantage` via exhaustive match in `modal.rs`
2. `cancel_modal` (`Esc`) → `hp41_core::ops::math1::cancel_modal(&mut self.state)` — already works
3. Alpha-label steps (MatrixNamePrompt, MtrNamePrompt, FdifeqFunctionNamePrompt): `maybe_auto_open_collect_for_modal` calls `mp.requires_alpha_label()` which now delegates to `advantage::modal::requires_alpha_label` — already wired

The only test work needed is a `phase44_modal_flow.rs` integration test file verifying the `ModalProgram::Advantage(AdvantageStep)` prompts route correctly through the carrier enum — following `phase34_modal_flow.rs` as template.

### Pattern 6: docs-matrix Extension (D-40.1 pattern)

The docs-matrix script (`scripts/docs-matrix/src/main.rs`) needs:
1. A 5th `else if` branch in `render_markdown()` for `"hp41-advantage-functions.json"` basename
2. A 5th invocation in `justfile`'s `docs-matrix` recipe
3. A 5th invocation in `justfile`'s `docs-matrix-check` recipe

This produces `docs/hp41-advantage-function-matrix.md`.

### Pattern 7: help_data.rs Test File (ADV-CLI-04 smoke tests)

Create `hp41-cli/tests/phase44_help_data_adv.rs` mirroring `phase39_help_data_time.rs` (297 lines, ~12 tests). Key adaptations:

- Count target: 114 (63 + 51), not 35
- Module_id checks: BOTH 22 AND 24 (not a single value) — test that every entry has a module_id in {22, 24}
- Category prefix: `"Adv "` (whatever prefix is chosen for the JSON)
- Function_id density: checked SEPARATELY per module_id group
- Pool total: `>= 130 + 45 + 26 + 35 + 114 = >= 350`

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Overlay category rendering | Custom HTML/table rendering | `help_overlay_rows()` + category prefix convention | Already auto-derives section headers from JSON `category` field |
| XROM mnemonic listing | Manual list maintenance | `ADV_MATH_A.ops` + `ADV_MATH_B.ops` slices | These are the single source of truth; JSON + inventory arrays must mirror them |
| Help search | Custom search algorithm | `filter_help_rows()` | Already handles multi-pool case |

---

## Common Pitfalls

### Pitfall 1: Dual-XROM Function-ID Scope Confusion
**What goes wrong:** JSON entries for ADV_MATH_B start `function_id` at 64 (treating both modules as one sequence) instead of restarting at 1.
**Why it happens:** Prior modules each had one XROM ID with a single dense range.
**How to avoid:** ADV_MATH_A's `function_id` is 1..=63 independently of ADV_MATH_B's 1..=51. The module_id field (22 vs 24) partitions them. Each dense-range test checks within its module_id group only.
**Warning signs:** `test_adv_function_ids_dense_and_sequential` assertions fail with max value > 51 or > 63.

### Pitfall 2: Including Run-Loop Variants in JSON / Inventory
**What goes wrong:** `AdvFsolveRunLoop`, `AdvFintgRunLoop`, `AdvFdifeqRunLoop` included in the JSON or in `ADV_B_OP_VARIANT_NAMES`.
**Why it happens:** These variants appear in `prgm_display.rs` (they display as "FSOLVE"/"FINTG"/"FDIFEQ") and are easy to count as part of the 51.
**How to avoid:** ADV_MATH_B has exactly 51 entries in `xrom.rs`. Run-loop variants are NOT in ADV_MATH_B.ops — they are internal-only. The JSON must have 114 total (63 + 51 = 114).
**Warning signs:** `test_adv_b_op_inventory_count` asserts 51 but list has 54; or `test_every_adv_b_json_entry_has_xrom_resolver_match` fails because run-loop variants are not in `adv_b_resolve`.

### Pitfall 3: Category Prefix Collision with Prior Modules
**What goes wrong:** Using a category prefix that matches an existing pool (e.g., `"Math"` collides with Math Pac I categories, breaking the overlay separator logic).
**Why it happens:** `help_overlay_rows()` groups entries by `category` string — overlapping prefixes merge sections across pools.
**How to avoid:** Use `"Adv "` prefix (e.g., `"Adv Conv"`, `"Adv Mtrx"`, `"Adv Math"`, `"Adv TVM"`) — no prior pool uses these.
**Warning signs:** `?` overlay shows mixed Math 1 and Advantage entries under the same section header.

### Pitfall 4: Partition Guard Not Updated Before JSON Added
**What goes wrong:** Adding the Advantage JSON to the 5-pool chain triggers `test_pool_partition_is_exhaustive` to fail with `rogue: [("AdvBinin", 22)]` before the guard is updated.
**Why it happens:** The partition test was specifically designed to fire when a new module is added without updating the guard.
**How to avoid:** Update the partition guard IN THE SAME TASK as adding the JSON + OnceLock pool. These are atomic — both must change together.
**Warning signs:** `cargo test function_matrix_parity` fails `test_pool_partition_is_exhaustive`.

### Pitfall 5: ADV-CLI-03 Already Done — Wasted Work
**What goes wrong:** A plan task tries to add `Op::Adv*` arms to `hp41-cli/src/prgm_display.rs`, which already has all 117 arms (confirmed by grep: 117 `Op::Adv` occurrences in prgm_display.rs as of commit 7c28d2b).
**How to avoid:** The planner must explicitly skip any `prgm_display.rs` task. ADV-CLI-03 is DONE.
**Warning signs:** A plan task says "add op_display_name arms" for the CLI — this is wrong.

### Pitfall 6: help_entries_all() Pool Order Breaking ?-Overlay Sections
**What goes wrong:** Inserting the Advantage chain in the middle rather than appending it last.
**Why it happens:** The `help_entries_all()` comment says order is fixed by convention, not alphabetical.
**How to avoid:** Append `.chain(help_entries_adv().iter())` as the FIFTH arm, after time. "Built-ins → Math 1 → Stat 1 → Time → Advantage Pac" matches the natural progression.

### Pitfall 7: Mnemonic String Discrepancy Between JSON and adv_a_resolve/adv_b_resolve
**What goes wrong:** JSON `display_name` doesn't match the mnemonic string in `adv_a_resolve`/`adv_b_resolve`, causing `test_every_adv_a_json_entry_has_xrom_resolver_match` to fail.
**Common cases:**
- `|Z|` vs `"|Z|"` (pipe characters need careful quoting in JSON)
- `|V|` similarly
- `*I` (star-I for TVM)
- `BIT?`, `DIM?`, `MNAME?`, `R>R?`, `SZ?`, `Y?X` (question marks)
**How to avoid:** Copy display_name strings directly from the `adv_a_resolve`/`adv_b_resolve` match arms in `hp41-core/src/ops/math1/xrom.rs` lines 654–793. The resolver is the authoritative source.

---

## Code Examples

### OnceLock accessor (fifth pool)

```rust
// Source: hp41-cli/src/help_data.rs — pattern from help_entries_time() at line 167
pub fn help_entries_adv() -> &'static [HelpEntry] {
    ADV_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(ADV_FUNCTIONS_JSON)
            .expect("hp41-advantage-functions.json is malformed — fix the JSON")
    })
}
```

### Partition test extension (two new module_id arms)

```rust
// Source: function_matrix_parity.rs test_pool_partition_is_exhaustive — extend existing match
let mut adv_a_count = 0usize;
let mut adv_b_count = 0usize;
// ... in the match:
Some(22) => adv_a_count += 1,
Some(24) => adv_b_count += 1,
// ... after loop:
assert_eq!(adv_a_count, 63, "ADV CONV+MTRX pool count drift: {adv_a_count}");
assert_eq!(adv_b_count, 51, "ADV MATH+TVM pool count drift: {adv_b_count}");
```

### xrom_shadowing extension (disjointness across all prior modules)

```rust
// Source: hp41-core/tests/xrom_shadowing.rs — pattern from time_ops_disjoint_from_math1_and_stat1
#[test]
fn adv_a_ops_disjoint_from_prior_modules() {
    use std::collections::HashSet;
    let math1_names: HashSet<&str> = MATH_1.ops.iter().map(|(n, _)| *n).collect();
    let stat1_names: HashSet<&str> = STAT_1.ops.iter().map(|(n, _)| *n).collect();
    let time_names: HashSet<&str> = TIME_MODULE.ops.iter().map(|(n, _)| *n).collect();
    for (name, _op) in ADV_MATH_A.ops {
        assert!(!math1_names.contains(name), "...");
        assert!(!stat1_names.contains(name), "...");
        assert!(!time_names.contains(name), "...");
    }
}

#[test]
fn adv_b_ops_disjoint_from_prior_modules_and_adv_a() {
    use std::collections::HashSet;
    // ... plus:
    let adv_a_names: HashSet<&str> = ADV_MATH_A.ops.iter().map(|(n, _)| *n).collect();
    for (name, _op) in ADV_MATH_B.ops {
        assert!(!adv_a_names.contains(name), "...");
        // ... prior modules too
    }
}
```

### Reverse parity test (bitfield for all 5 modules)

```rust
// xrom_resolve with all 5 modules loaded: 0b0001_1111
let resolved = xrom_resolve(entry.display_name.as_str(), 0b0001_1111);
```

### Modal flow test (carrier enum level)

```rust
// Source: phase34_modal_flow.rs pattern
use hp41_core::ops::math1::modal::ModalProgram;
use hp41_core::ops::advantage::modal::AdvantageStep;

#[test]
fn advantage_tvm_n_prompt() {
    let prog = ModalProgram::Advantage(AdvantageStep::TvmN);
    assert_eq!(prog.current_prompt(), Some("N=?".to_string()));
    assert!(!prog.requires_alpha_label());
}

#[test]
fn advantage_matrix_name_prompt_requires_alpha_label() {
    let prog = ModalProgram::Advantage(AdvantageStep::MatrixNamePrompt);
    assert_eq!(prog.current_prompt(), Some("MNAME?".to_string()));
    assert!(prog.requires_alpha_label()); // triggers CollectForModal auto-open
}
```

---

## Status of Each Requirement

| Requirement | Status | What Remains |
|-------------|--------|--------------|
| ADV-CLI-01 | NOT STARTED | Author `docs/hp41-advantage-functions.json` (114 entries) |
| ADV-CLI-02 | NOT STARTED | Add 5th OnceLock pool to `help_data.rs` |
| ADV-CLI-03 | **COMPLETE** | 117 arms in `prgm_display.rs` as of commit 7c28d2b |
| ADV-CLI-04 | Blocked on ADV-CLI-01+02 | Auto-derived from JSON categories; just needs the pool wired |
| ADV-CLI-05 | NOT STARTED | Update `function_matrix_parity.rs` partition + add inventory tests |
| ADV-CLI-06 | NOT STARTED | Extend `hp41-core/tests/xrom_shadowing.rs` with 8 new tests |
| ADV-CLI-07 | Infrastructure COMPLETE | Write `phase44_modal_flow.rs` test to verify routing |
| ADV-CLI-08 | **ALREADY SATISFIED** | `entry.xrom.is_none()` filter in `keys.rs` is unchanged |

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) |
| Config file | none (standard Cargo test runner) |
| Quick run command | `cargo test --package hp41-cli --test function_matrix_parity` |
| Full suite command | `just test` (or `cargo test --package hp41-cli && cargo test --package hp41-core`) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File |
|--------|----------|-----------|-------------------|------|
| ADV-CLI-01 | JSON loads without panic | integration | `cargo test --package hp41-cli --test phase44_help_data_adv` | Wave 0: create |
| ADV-CLI-01 | JSON has 114 entries | integration | same | Wave 0: create |
| ADV-CLI-01 | function_ids dense per module_id | integration | same | Wave 0: create |
| ADV-CLI-02 | 5-pool chain returns >= 350 entries | integration | `cargo test --package hp41-cli --test phase44_help_data_adv` | Wave 0: create |
| ADV-CLI-03 | Already passing | — | `cargo check --package hp41-cli` | Exists |
| ADV-CLI-04 | Overlay has Adv* category headers | integration | `cargo test --package hp41-cli --test phase44_help_data_adv` | Wave 0: create |
| ADV-CLI-05 | Partition guard accepts module_ids 22, 24 | integration | `cargo test --package hp41-cli --test function_matrix_parity` | Modify existing |
| ADV-CLI-06 | ADV_A/B disjoint from all prior | integration | `cargo test --package hp41-core --test xrom_shadowing` | Modify existing |
| ADV-CLI-07 | AdvantageStep prompts route correctly | integration | `cargo test --package hp41-cli --test phase44_modal_flow` | Wave 0: create |
| ADV-CLI-08 | Right-panel excludes Adv entries | integration | `cargo test --package hp41-cli --test phase44_key_ref_excludes_adv` | Wave 0: create |

### Sampling Rate

- **Per task commit:** `cargo test --package hp41-cli --test function_matrix_parity && cargo test --package hp41-core --test xrom_shadowing`
- **Per wave merge:** `cargo test --package hp41-cli && cargo test --package hp41-core`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `hp41-cli/tests/phase44_help_data_adv.rs` — covers ADV-CLI-01, ADV-CLI-02, ADV-CLI-04, ADV-CLI-08
- [ ] `hp41-cli/tests/phase44_modal_flow.rs` — covers ADV-CLI-07
- [ ] `hp41-cli/tests/phase44_key_ref_excludes_adv.rs` — covers ADV-CLI-08 (explicit sentinels like BININ, TVM)
- [ ] `docs/hp41-advantage-functions.json` — covers ADV-CLI-01 (prerequisite for all tests)
- [ ] `docs/hp41-advantage-function-matrix.md` — generated output (prerequisite for docs-matrix-check)

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo test | All tests | ✓ | MSRV 1.88 | — |
| just | docs-matrix recipe | ✓ | (confirmed in project) | Call cargo directly |
| scripts/docs-matrix | Matrix generation | ✓ | standalone Cargo binary | — |

No missing dependencies.

---

## Security Domain

No security-relevant changes in this phase. CLI integration is display + test-gating only — no auth, no I/O, no new network surface.

---

## Open Questions (RESOLVED)

1. **JSON category naming for Advantage Pac** — RESOLVED: Use four fine-grained categories (`"Adv Conv"`, `"Adv Mtrx"`, `"Adv Math"`, `"Adv TVM"`) for better `?`-overlay discoverability. This produces four section headers rather than two XROM-ID-labeled sections. ADV-CLI-04 ROADMAP wording updated to reflect four-category design.

2. **Divergence documentation for FSOLVE/FINTG/FDIFEQ display_name** — RESOLVED: No divergence entry needed (same pattern as Math Pac I `MatrixWorkflow` → `MATRIX`). Defer to Phase 45 `docs/hp41-advantage-divergences.md` if needed.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | ADV-CLI-03 is done in commit 7c28d2b (117 arms in prgm_display.rs) | Status table | If only partially done, prgm_display.rs needs further work — verify with `cargo check` before planning |
| A2 | `ModalProgram::Advantage` dispatch in modal.rs compiles and routes correctly via existing app.rs infrastructure | ADV-CLI-07 analysis | If Phase 43 left a bug in modal routing, Phase 44 needs a fix task in app.rs |
| A3 | Category prefix for Advantage JSON entries should use four categories (Conv/Mtrx/Math/TVM) | Open question | If planner decides two categories (per XROM ID), test assertions about category prefix must change |

---

## Sources

### Primary (HIGH confidence)
- `[VERIFIED: codebase]` — `hp41-cli/src/help_data.rs` lines 1–389 — full OnceLock pool pattern confirmed
- `[VERIFIED: codebase]` — `hp41-cli/tests/function_matrix_parity.rs` — partition guard structure and inventory pattern confirmed
- `[VERIFIED: codebase]` — `hp41-core/tests/xrom_shadowing.rs` — extension pattern for STAT_1 and TIME_MODULE confirmed
- `[VERIFIED: codebase]` — `hp41-core/src/ops/math1/xrom.rs` lines 246–402 — ADV_MATH_A (63 entries, id=22) and ADV_MATH_B (51 entries, id=24) confirmed
- `[VERIFIED: codebase]` — `hp41-core/src/ops/advantage/modal.rs` — AdvantageStep enum with 16 variants, `requires_alpha_label()` returning true for MatrixNamePrompt/MtrNamePrompt/FdifeqFunctionNamePrompt
- `[VERIFIED: codebase]` — `hp41-cli/src/prgm_display.rs` — 117 `Op::Adv*` match arms present (ADV-CLI-03 confirmed complete)
- `[VERIFIED: codebase]` — `hp41-cli/tests/phase34_modal_flow.rs` and `phase34_help_data_stat1.rs` — template patterns for Phase 44 new test files
- `[VERIFIED: codebase]` — `hp41-cli/tests/phase39_help_data_time.rs` — template for phase44_help_data_adv.rs
- `[VERIFIED: codebase]` — `docs/hp41-stat1-functions.json` and `docs/hp41-time-functions.json` — exact JSON schema confirmed

### Secondary (MEDIUM confidence)
- `[CITED: CLAUDE.md §Frozen Invariants]` — 4-way exhaustive-match invariant; items 3+4 deferred to Phase 44/46
- `[CITED: CLAUDE.md §JSON canonical data flow]` — JSON pipeline pattern, OnceLock, hard-build-blocker

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — zero new deps; all patterns directly verified in codebase
- Architecture: HIGH — all extension points identified with exact file:line references
- Pitfalls: HIGH — discovered by reading actual test failure patterns from prior phases
- JSON content: HIGH — mnemonics extracted directly from adv_a_resolve/adv_b_resolve match arms

**Research date:** 2026-05-26
**Valid until:** 2026-07-26 (stable patterns, no external deps)
