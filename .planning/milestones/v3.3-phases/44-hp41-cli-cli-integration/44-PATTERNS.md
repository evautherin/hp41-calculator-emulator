# Phase 44: hp41-cli CLI Integration - Pattern Map

**Mapped:** 2026-05-26
**Files analyzed:** 7 (4 new test files, 1 new JSON file, 2 existing files modified)
**Analogs found:** 7 / 7

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `docs/hp41-advantage-functions.json` | config (JSON data) | transform | `docs/hp41-time-functions.json` | exact |
| `hp41-cli/src/help_data.rs` (modify) | utility | transform | self (current state at lines 149–195) | exact |
| `hp41-cli/tests/function_matrix_parity.rs` (modify) | test | CRUD | self (current state at lines 554–661) | exact |
| `hp41-core/tests/xrom_shadowing.rs` (modify) | test | event-driven | self (current state at lines 162–238) | exact |
| `hp41-cli/tests/phase44_help_data_adv.rs` | test | CRUD | `hp41-cli/tests/phase39_help_data_time.rs` | exact |
| `hp41-cli/tests/phase44_modal_flow.rs` | test | event-driven | `hp41-cli/tests/phase34_modal_flow.rs` | exact |
| `hp41-cli/tests/key_coverage.rs` (modify) | test | request-response | self (current state at lines 360–405) | exact |

---

## Pattern Assignments

### `docs/hp41-advantage-functions.json` (new, 114 entries)

**Analog:** `docs/hp41-time-functions.json` (lines 1–61 sample shown below)

**JSON entry shape** (one complete entry):
```json
{
    "op_variant": "TimeTime",
    "display_name": "TIME",
    "category": "Time Clock",
    "status": "implemented",
    "phase": "38",
    "key_path": "XEQ \"TIME\"",
    "description": "Display current system time in X (HH.MMSSss)",
    "xrom": { "module": "Time", "module_id": 26, "function_id": 1 }
}
```

**Adaptation rules for Advantage Pac:**
- `op_variant`: PascalCase matching `Op::Adv*` enum variant exactly (e.g. `"AdvBinin"`, `"AdvExpZ"`)
- `display_name`: HP mnemonic string — copy exactly from `adv_a_resolve`/`adv_b_resolve` match arms in `hp41-core/src/ops/math1/xrom.rs` lines 654–793 (authoritative source; pitfall: `|Z|`, `|V|`, `*I`, `BIT?`, `DIM?`, `MNAME?`, `R>R?`, `SZ?`, `Y?X` contain special chars)
- `category`: Use `"Adv Conv"`, `"Adv Mtrx"`, `"Adv Math"`, `"Adv TVM"` (four categories; prefix `"Adv "` is unique across all prior pools; Time uses `"Time "`, Stat1 uses `"Stat1 "`)
- `status`: `"implemented"` for all 114 entries
- `phase`: `"43"` for all entries
- `key_path`: `XEQ "<MNEMONIC>"` form for all entries (XROM-only; no dedicated key bindings)
- `xrom.module`: `"Adv Conv"` for entries 1–63, `"Adv Math"` for entries 1–51
- `xrom.module_id`: **22** for ADV_MATH_A entries (IDs 1–63), **24** for ADV_MATH_B entries (IDs 1–51)
- `xrom.function_id`: **1-based dense range INDEPENDENTLY per module** — ADV_MATH_A: 1..=63, ADV_MATH_B: 1..=51 (Pitfall 1: do NOT continue from 64)
- `divergences`: omit entirely for all entries (no documented Advantage divergences in Phase 44 scope)
- **Do NOT include** `AdvFsolveRunLoop`, `AdvFintgRunLoop`, `AdvFdifeqRunLoop` (internal-only variants not in `ADV_MATH_B.ops`)
- **Total: 114 entries** (63 + 51)

---

### `hp41-cli/src/help_data.rs` (modify: add fifth OnceLock pool)

**Analog:** `hp41-cli/src/help_data.rs` lines 149–195 (the Time Pac pool addition)

**Imports pattern** (lines 20–22, unchanged):
```rust
use std::sync::OnceLock;
use serde::Deserialize;
```

**New constants to add** (after the TIME block at line 172, following the exact Time pattern):
```rust
// lines 149-172 (existing Time pool — shown for context):
const TIME_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-time-functions.json");
static TIME_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();
pub fn help_entries_time() -> &'static [HelpEntry] {
    TIME_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(TIME_FUNCTIONS_JSON)
            .expect("hp41-time-functions.json is malformed — fix the JSON")
    })
}

// NEW fifth pool (add after line 172):
const ADV_FUNCTIONS_JSON: &str = include_str!("../../docs/hp41-advantage-functions.json");
static ADV_HELP_ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new();

/// Access the parsed Advantage Pac help entries (lazily initialized, thread-safe via OnceLock).
///
/// **Panics** on first invocation if `docs/hp41-advantage-functions.json` is
/// malformed — this is the **intentional** D-25.17 / D-29.2 hard-build-blocker
/// behavior (fifth-file copy). The per-file panic message routes failures to the
/// correct source-of-truth file. Subsequent calls return the cached slice.
///
/// Narrow accessor — returns ONLY the Advantage Pac pool. Use [`help_entries_all`]
/// for the merged pool in UI rendering paths.
pub fn help_entries_adv() -> &'static [HelpEntry] {
    ADV_HELP_ENTRIES.get_or_init(|| {
        serde_json::from_str(ADV_FUNCTIONS_JSON)
            .expect("hp41-advantage-functions.json is malformed — fix the JSON")
    })
}
```

**`help_entries_all()` update** (lines 189–195 currently; add fifth chain arm):
```rust
// CURRENT (lines 189-195):
pub fn help_entries_all() -> impl Iterator<Item = &'static HelpEntry> {
    help_entries()
        .iter()
        .chain(help_entries_math1().iter())
        .chain(help_entries_stat1().iter())
        .chain(help_entries_time().iter())
}

// UPDATED (append fifth arm — LAST, never in middle per D-39.12 ordering rationale):
pub fn help_entries_all() -> impl Iterator<Item = &'static HelpEntry> {
    help_entries()
        .iter()
        .chain(help_entries_math1().iter())
        .chain(help_entries_stat1().iter())
        .chain(help_entries_time().iter())
        .chain(help_entries_adv().iter())  // ← fifth arm, Advantage Pac
}
```

**Doc comment update** for `help_entries_all()` (update the two references from "four" to "five", add `help_entries_adv` to the narrow-accessor list, update the comment at line 184 about Advantage Pac):
- Line 174: `"Merged accessor: chains all five JSON pools (v2.2 built-ins + Math Pac I + Stat 1 Pac + Time Pac + Advantage Pac)"`
- Line 175: `"in order per D-39.12 / D-44.XX (fixed insertion order — built-ins → Math 1 → Stat 1 → Time → Advantage Pac)"`
- Line 183: remove the future-tense comment "Future XROM modules (Advantage Pac) extend by appending..." — the fifth arm is now present
- Add `help_entries_adv` to the narrow-accessor list at line 188

**Public API surface** (add to `hp41-cli/src/lib.rs` re-export if `help_data` public items are gated):
Verify `help_entries_adv` is accessible from integration tests via `hp41_cli::help_data::help_entries_adv`. Pattern from `phase39_help_data_time.rs` line 19: `use hp41_cli::help_data::{..., help_entries_time};` — add `help_entries_adv` to the same use path.

---

### `hp41-cli/tests/function_matrix_parity.rs` (modify: add ADV pool tests + partition guard update)

**Analog:** `hp41-cli/tests/function_matrix_parity.rs` lines 554–661 (Time Pac section)

**Import update** (line 16–17, add `help_entries_adv`):
```rust
use hp41_cli::help_data::{
    help_entries, help_entries_math1, help_entries_stat1, help_entries_time,
    help_entries_adv,  // ← add
};
```

**New inventory constants** (add after `TIME_OP_VARIANT_NAMES` at line 604):
```rust
/// Hand-curated inventory of all ADV_MATH_A `Op` variants shipped in Phase 43.
/// Drift between this list and `ADV_MATH_A.ops` in xrom.rs is caught by
/// `test_adv_a_op_inventory_count`. Does NOT include run-loop variants
/// (AdvFsolveRunLoop, AdvFintgRunLoop, AdvFdifeqRunLoop) — these are internal-only.
///
/// Maintenance gate: if a future plan adds Op variants to ADV_MATH_A, append here
/// AND add matching JSON rows to `docs/hp41-advantage-functions.json`.
const ADV_A_OP_VARIANT_NAMES: &[&str] = &[
    // ADV CONV (12 entries, function_ids 1–12)
    "AdvBinin", "AdvBinview", "AdvOctin", "AdvHexin", "AdvHexview", "AdvCvtview",
    "AdvNot", "AdvAnd", "AdvOr", "AdvXor", "AdvRotxy", "AdvBitTest",
    // ADV MTRX (51 entries, function_ids 13–63)
    "AdvIPlus", "AdvIMinus", "AdvJPlus", "AdvJMinus",
    "AdvMr", "AdvMs", "AdvMrij", "AdvMsij", "AdvMsijr",
    "AdvMrcPlus", "AdvMrcMinus", "AdvMrrPlus", "AdvMrrMinus", "AdvMsrPlus", "AdvMscPlus",
    "AdvMswap", "AdvMnameQuery", "AdvDimQuery", "AdvMatdim", "AdvMp", "AdvPiv",
    "AdvRExchangeR", "AdvRGtRQuery", "AdvSum", "AdvSumab", "AdvMax", "AdvMaxab",
    "AdvMin", "AdvRmaxab", "AdvRnrm", "AdvRsum", "AdvFnrm",
    "AdvMdet", "AdvMinv", "AdvMsys",
    "AdvMMulM", "AdvMatPlus", "AdvMatMinus", "AdvMatScalarMul", "AdvMatScalarDiv",
    "AdvTrnps", "AdvMmove", "AdvCExchangeC", "AdvCmaxab", "AdvCnrm", "AdvCsum",
    "AdvYcPlusC", "AdvMatrx", "AdvMtr", "AdvMedit", "AdvCmedit",
];

/// Hand-curated inventory of all ADV_MATH_B `Op` variants shipped in Phase 43.
/// Does NOT include run-loop variants (AdvFsolveRunLoop, AdvFintgRunLoop,
/// AdvFdifeqRunLoop) — these are internal-only and not in ADV_MATH_B.ops.
///
/// Maintenance gate: if a future plan adds Op variants to ADV_MATH_B, append here
/// AND add matching JSON rows to `docs/hp41-advantage-functions.json`.
const ADV_B_OP_VARIANT_NAMES: &[&str] = &[
    // ADV MATH (45 entries, function_ids 1–45)
    "AdvExpZ", "AdvLnZ", "AdvLogZ", "AdvZPowN", "AdvZPow1n", "AdvZPowW", "AdvZPow1w",
    "AdvMagz", "AdvSinZ", "AdvCosZ", "AdvTanZ", "AdvAPowZ",
    "AdvCPlus", "AdvCMinus", "AdvCinv", "AdvCMul", "AdvCDiv",
    "AdvAip", "AdvPly", "AdvRts", "AdvFsolve", "AdvFintg", "AdvFdifeq", "AdvFroot",
    "AdvCfit", "AdvAs", "AdvDs", "AdvBfit", "AdvFit", "AdvYQueryX", "AdvSzQuery",
    "AdvVPlus", "AdvVMinus", "AdvDot", "AdvCross", "AdvVc", "AdvVs", "AdvVr",
    "AdvVe", "AdvVxy", "AdvUv", "AdvVMag", "AdvVStar", "AdvVd", "AdvTr",
    // ADV TVM (6 entries, function_ids 46–51)
    "AdvTvm", "AdvTvmN", "AdvTvmPv", "AdvTvmPmt", "AdvTvmFv", "AdvTvmStarI",
];
```

**New inventory count tests** (add after TIME count test at line 617):
```rust
#[test]
fn test_adv_a_op_inventory_count() {
    assert_eq!(
        ADV_A_OP_VARIANT_NAMES.len(),
        63,
        "ADV_A_OP_VARIANT_NAMES inventory drift — expected 63 ADV_MATH_A Op variants per Phase 43 ship. \
         Does NOT include AdvFsolveRunLoop/AdvFintgRunLoop/AdvFdifeqRunLoop (internal-only). \
         Did a future plan add Op variants without updating this inventory and docs/hp41-advantage-functions.json?"
    );
}

#[test]
fn test_adv_b_op_inventory_count() {
    assert_eq!(
        ADV_B_OP_VARIANT_NAMES.len(),
        51,
        "ADV_B_OP_VARIANT_NAMES inventory drift — expected 51 ADV_MATH_B Op variants per Phase 43 ship. \
         Does NOT include AdvFsolveRunLoop/AdvFintgRunLoop/AdvFdifeqRunLoop (internal-only). \
         Did a future plan add Op variants without updating this inventory and docs/hp41-advantage-functions.json?"
    );
}
```

**New forward parity tests** (mirror of `test_every_time_rom_op_has_time_json_entry`):
```rust
#[test]
fn test_every_adv_a_rom_op_has_adv_a_json_entry() {
    let json_variants: HashSet<&str> = help_entries_adv()
        .iter()
        .filter(|e| e.xrom.as_ref().map(|x| x.module_id) == Some(22))
        .map(|e| e.op_variant.as_str())
        .collect();
    let mut missing: Vec<&str> = Vec::new();
    for name in ADV_A_OP_VARIANT_NAMES {
        if !json_variants.contains(name) {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "ADV_MATH_A Op::* variants missing from docs/hp41-advantage-functions.json (module_id=22): {missing:?}"
    );
}

#[test]
fn test_every_adv_b_rom_op_has_adv_b_json_entry() {
    let json_variants: HashSet<&str> = help_entries_adv()
        .iter()
        .filter(|e| e.xrom.as_ref().map(|x| x.module_id) == Some(24))
        .map(|e| e.op_variant.as_str())
        .collect();
    let mut missing: Vec<&str> = Vec::new();
    for name in ADV_B_OP_VARIANT_NAMES {
        if !json_variants.contains(name) {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "ADV_MATH_B Op::* variants missing from docs/hp41-advantage-functions.json (module_id=24): {missing:?}"
    );
}
```

**New reverse parity tests** (bitfield `0b0001_1111` = all 5 modules):
```rust
#[test]
fn test_every_adv_a_json_entry_has_xrom_resolver_match() {
    // Uses 0b0001_1111 (all 5 modules loaded = v3.3 default bitfield)
    let mut orphans: Vec<String> = Vec::new();
    for entry in help_entries_adv().iter().filter(|e| e.xrom.as_ref().map(|x| x.module_id) == Some(22)) {
        let resolved =
            hp41_core::ops::math1::xrom::xrom_resolve(entry.display_name.as_str(), 0b0001_1111);
        if resolved.is_none() {
            orphans.push(format!(
                "'{}' (display_name='{}') — not found in ADV_MATH_A.ops / adv_a_resolve",
                entry.op_variant, entry.display_name
            ));
        }
    }
    assert!(
        orphans.is_empty(),
        "ADV_MATH_A JSON entries whose display_name is NOT resolved by xrom_resolve(_, 0b0001_1111): {orphans:?}"
    );
}

#[test]
fn test_every_adv_b_json_entry_has_xrom_resolver_match() {
    let mut orphans: Vec<String> = Vec::new();
    for entry in help_entries_adv().iter().filter(|e| e.xrom.as_ref().map(|x| x.module_id) == Some(24)) {
        let resolved =
            hp41_core::ops::math1::xrom::xrom_resolve(entry.display_name.as_str(), 0b0001_1111);
        if resolved.is_none() {
            orphans.push(format!(
                "'{}' (display_name='{}') — not found in ADV_MATH_B.ops / adv_b_resolve",
                entry.op_variant, entry.display_name
            ));
        }
    }
    assert!(
        orphans.is_empty(),
        "ADV_MATH_B JSON entries whose display_name is NOT resolved by xrom_resolve(_, 0b0001_1111): {orphans:?}"
    );
}
```

**Partition guard update** (`test_pool_partition_is_exhaustive`, lines 514–552):
```rust
// CURRENT match arms (lines 531–537):
match entry.xrom.as_ref().map(|x| x.module_id) {
    None => builtin_count += 1,
    Some(7) => math1_count += 1,
    Some(2) => stat1_count += 1,
    Some(26) => time_count += 1,
    Some(other) => rogue.push((entry.op_variant.clone(), other)),
}

// UPDATED (add two new arms + two new counters before the loop):
let mut adv_a_count = 0usize;
let mut adv_b_count = 0usize;
// ... in the match:
match entry.xrom.as_ref().map(|x| x.module_id) {
    None => builtin_count += 1,
    Some(7) => math1_count += 1,
    Some(2) => stat1_count += 1,
    Some(26) => time_count += 1,
    Some(22) => adv_a_count += 1,   // ← ADV_MATH_A (ADV CONV+MTRX)
    Some(24) => adv_b_count += 1,   // ← ADV_MATH_B (ADV MATH+TVM)
    Some(other) => rogue.push((entry.op_variant.clone(), other)),
}
// ... after loop, add:
assert_eq!(adv_a_count, 63, "ADV CONV+MTRX pool count drift: {adv_a_count}");
assert_eq!(adv_b_count, 51, "ADV MATH+TVM pool count drift: {adv_b_count}");
```

**Error message update** (line 541 rogue assertion, update the supported-module-ids list):
```
// CURRENT:
"v3.2 supports only module_id in {{None (built-ins), 7 (Math 1), 2 (Stat 1), 26 (Time)}}."
// UPDATED:
"v3.3 supports only module_id in {{None (built-ins), 7 (Math 1), 2 (Stat 1), 26 (Time), 22 (Adv XROM 22), 24 (Adv XROM 24)}}."
```

---

### `hp41-core/tests/xrom_shadowing.rs` (modify: add ADV_MATH_A and ADV_MATH_B tests)

**Analog:** `hp41-core/tests/xrom_shadowing.rs` lines 162–238 (TIME_MODULE section)

**Import update** (line 26):
```rust
// CURRENT:
use hp41_core::ops::math1::xrom::{xrom_resolve, MATH_1, STAT_1, TIME_MODULE};

// UPDATED:
use hp41_core::ops::math1::xrom::{xrom_resolve, ADV_MATH_A, ADV_MATH_B, MATH_1, STAT_1, TIME_MODULE};
```

**New test block 1 — ADV_MATH_A** (add after `time_const_fields` at line 238):
```rust
// ── Phase 44: ADV_MATH_A disjointness + consistency gates ────────────────────

/// CI gate: no ADV_MATH_A mnemonic may collide with a v2.2 builtin name.
#[test]
fn adv_a_names_do_not_shadow_builtins() {
    for (name, _op) in ADV_MATH_A.ops {
        assert!(
            !BUILTIN_CARD_OP_NAMES.contains(name),
            "ADV_MATH_A mnemonic {name:?} shadows a builtin_card_op entry. \
             The XROM resolver fires LAST (C-28.4), so the builtin would silently \
             win and the Advantage Pac op would be permanently unreachable via XEQ. \
             Rename the ADV_MATH_A mnemonic to avoid the collision."
        );
    }
}

/// CI gate: ADV_MATH_A.ops mnemonic strings must be disjoint from all prior modules.
#[test]
fn adv_a_ops_disjoint_from_prior_modules() {
    use std::collections::HashSet;
    let math1_names: HashSet<&str> = MATH_1.ops.iter().map(|(n, _)| *n).collect();
    let stat1_names: HashSet<&str> = STAT_1.ops.iter().map(|(n, _)| *n).collect();
    let time_names: HashSet<&str> = TIME_MODULE.ops.iter().map(|(n, _)| *n).collect();
    for (name, _op) in ADV_MATH_A.ops {
        assert!(
            !math1_names.contains(name),
            "ADV_MATH_A mnemonic {name:?} also appears in MATH_1.ops. \
             A mnemonic must belong to exactly one XROM module (resolver-LAST invariant)."
        );
        assert!(
            !stat1_names.contains(name),
            "ADV_MATH_A mnemonic {name:?} also appears in STAT_1.ops."
        );
        assert!(
            !time_names.contains(name),
            "ADV_MATH_A mnemonic {name:?} also appears in TIME_MODULE.ops."
        );
    }
}

/// CI gate: every ADV_MATH_A.ops mnemonic resolves to its declared Op via
/// `xrom_resolve` when all 5 module bits are set (0b0001_1111 = v3.3 default).
#[test]
fn adv_a_ops_resolve_via_xrom_resolve() {
    for (name, expected_op) in ADV_MATH_A.ops {
        let resolved = xrom_resolve(name, 0b0001_1111);
        assert_eq!(
            resolved.as_ref(),
            Some(expected_op),
            "ADV_MATH_A.ops mnemonic {name:?} must resolve to {expected_op:?} via \
             xrom_resolve(name, 0b0001_1111) — drift between ADV_MATH_A.ops slice and \
             adv_a_resolve match arms is a Pitfall 22 / resolver-never-discard violation."
        );
    }
}

/// Smoke: ADV_MATH_A const fields are present and correct.
#[test]
fn adv_a_const_fields() {
    assert_eq!(
        ADV_MATH_A.id, 22,
        "ADV_MATH_A.id must be 22 (HP Advantage Pac ADV CONV+MTRX hardware XROM ID)"
    );
    assert_eq!(ADV_MATH_A.name, "ADV CONV", "ADV_MATH_A.name must be 'ADV CONV'");
}
```

**New test block 2 — ADV_MATH_B** (add after adv_a tests):
```rust
// ── ADV_MATH_B disjointness + consistency gates ───────────────────────────────

/// CI gate: no ADV_MATH_B mnemonic may collide with a v2.2 builtin name.
#[test]
fn adv_b_names_do_not_shadow_builtins() {
    for (name, _op) in ADV_MATH_B.ops {
        assert!(
            !BUILTIN_CARD_OP_NAMES.contains(name),
            "ADV_MATH_B mnemonic {name:?} shadows a builtin_card_op entry. \
             The XROM resolver fires LAST (C-28.4), so the builtin would silently \
             win and the Advantage Pac op would be permanently unreachable via XEQ."
        );
    }
}

/// CI gate: ADV_MATH_B.ops mnemonic strings must be disjoint from all prior modules AND ADV_MATH_A.
#[test]
fn adv_b_ops_disjoint_from_prior_modules_and_adv_a() {
    use std::collections::HashSet;
    let math1_names: HashSet<&str> = MATH_1.ops.iter().map(|(n, _)| *n).collect();
    let stat1_names: HashSet<&str> = STAT_1.ops.iter().map(|(n, _)| *n).collect();
    let time_names: HashSet<&str> = TIME_MODULE.ops.iter().map(|(n, _)| *n).collect();
    let adv_a_names: HashSet<&str> = ADV_MATH_A.ops.iter().map(|(n, _)| *n).collect();
    for (name, _op) in ADV_MATH_B.ops {
        assert!(!math1_names.contains(name), "ADV_MATH_B mnemonic {name:?} in MATH_1.ops.");
        assert!(!stat1_names.contains(name), "ADV_MATH_B mnemonic {name:?} in STAT_1.ops.");
        assert!(!time_names.contains(name), "ADV_MATH_B mnemonic {name:?} in TIME_MODULE.ops.");
        assert!(!adv_a_names.contains(name), "ADV_MATH_B mnemonic {name:?} in ADV_MATH_A.ops.");
    }
}

/// CI gate: every ADV_MATH_B.ops mnemonic resolves to its declared Op via
/// `xrom_resolve` when all 5 module bits are set (0b0001_1111 = v3.3 default).
#[test]
fn adv_b_ops_resolve_via_xrom_resolve() {
    for (name, expected_op) in ADV_MATH_B.ops {
        let resolved = xrom_resolve(name, 0b0001_1111);
        assert_eq!(
            resolved.as_ref(),
            Some(expected_op),
            "ADV_MATH_B.ops mnemonic {name:?} must resolve to {expected_op:?} via \
             xrom_resolve(name, 0b0001_1111) — drift between ADV_MATH_B.ops slice and \
             adv_b_resolve match arms is a Pitfall 22 / resolver-never-discard violation."
        );
    }
}

/// Smoke: ADV_MATH_B const fields are present and correct.
#[test]
fn adv_b_const_fields() {
    assert_eq!(
        ADV_MATH_B.id, 24,
        "ADV_MATH_B.id must be 24 (HP Advantage Pac ADV MATH+TVM hardware XROM ID)"
    );
    assert_eq!(ADV_MATH_B.name, "ADV MATH", "ADV_MATH_B.name must be 'ADV MATH'");
}
```

---

### `hp41-cli/tests/phase44_help_data_adv.rs` (new file)

**Analog:** `hp41-cli/tests/phase39_help_data_time.rs` (full file, 297 lines)

**File header pattern** (copy from phase39, update references):
```rust
//! Phase 44 Plan 01 smoke tests — `docs/hp41-advantage-functions.json` is the
//! canonical data source for `hp41-cli/src/help_data.rs::help_entries_adv()` via
//! include_str! + OnceLock per D-44.XX (fifth-pool extension).
//!
//! Mirrors `phase39_help_data_time.rs` structure exactly; swaps accessor and
//! count target (== 114 unique Op variants — 63 ADV CONV+MTRX + 51 ADV MATH+TVM).
//!
//! Test 11 (`help_entries_all_returns_five_pools`) asserts the merged chain
//! length is >= 130 + 45 + 26 + 35 + 114 = 350 (v2.2 + Math 1 + Stat 1 + Time + Advantage).
//!
//! Test 12 (`help_overlay_rows_includes_adv_pac_section`) verifies the
//! 5th pool feeds into help_overlay_rows() and "Adv " category headers appear.

#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use hp41_cli::help_data::{
    help_entries, help_entries_all, help_entries_adv, help_entries_math1,
    help_entries_stat1, help_entries_time, help_overlay_rows,
};
```

**Test adaptations from Time analog (12 tests total):**

| Test # | Time analog | Advantage adaptation |
|--------|-------------|---------------------|
| 1 | `time_help_entries_is_not_empty` | `adv_help_entries_is_not_empty` — checks `help_entries_adv()` |
| 2 | `time_help_entries_count_meets_35_target` | `adv_help_entries_count_meets_114_target` — `assert_eq!(entries.len(), 114, ...)` |
| 3 | `time_every_entry_has_display_name` | `adv_every_entry_has_display_name` |
| 4 | `time_every_entry_has_description` | `adv_every_entry_has_description` |
| 5 | `time_all_entries_are_implemented` | `adv_all_entries_are_implemented` |
| 6 | `time_every_entry_has_xrom_module_id_26` | `adv_every_entry_has_xrom_module_id_in_22_or_24` — check `module_id` is 22 or 24 (not a single value) |
| 7 | `time_categories_use_time_prefix` | `adv_categories_use_adv_prefix` — `entry.category.starts_with("Adv ")` |
| 8 | `time_function_ids_dense_and_sequential` | `adv_function_ids_dense_per_module` — SPLIT into TWO sub-checks: ADV_MATH_A (1..=63) and ADV_MATH_B (1..=51) each independently (Pitfall 1) |
| 9 | `time_no_key_path_is_empty` | `adv_all_key_paths_are_xeq_form` — same XEQ form check |
| 10 | `time_divergences_are_surgical` | `adv_no_divergence_entries` — assert `divergences.is_empty()` for all entries (no documented divergences in Phase 44) |
| 11 | `help_entries_all_returns_four_pools` | `help_entries_all_returns_five_pools` — `assert!(all_count >= 350, ...)` |
| 12 | `help_overlay_rows_includes_time_pac_section` | `help_overlay_rows_includes_adv_pac_sections` — check `r.desc.starts_with("=== Adv ")` |

**Critical test 8 adaptation** (function_id density — dual-module pattern):
```rust
#[test]
fn adv_function_ids_dense_per_module() {
    // SEPARATE dense-range check per module_id — NOT a single range across both modules.
    // ADV_MATH_A (module_id=22): must be 1..=63
    // ADV_MATH_B (module_id=24): must be 1..=51
    // Pitfall 1: ADV_MATH_B function_ids RESTART at 1 — they are NOT 64..=114.

    let adv_entries = help_entries_adv();

    // ADV_MATH_A check
    let mut adv_a_ids: Vec<u16> = adv_entries
        .iter()
        .filter(|e| e.xrom.as_ref().map(|x| x.module_id) == Some(22))
        .map(|e| e.xrom.as_ref().unwrap().function_id)
        .collect();
    adv_a_ids.sort_unstable();
    assert_eq!(adv_a_ids.first().copied().unwrap_or(0), 1, "ADV_MATH_A function_ids must start at 1");
    assert_eq!(adv_a_ids.last().copied().unwrap_or(0), 63, "ADV_MATH_A function_ids must end at 63");
    assert_eq!(adv_a_ids.len(), 63, "ADV_MATH_A must have 63 unique function_ids");

    // ADV_MATH_B check
    let mut adv_b_ids: Vec<u16> = adv_entries
        .iter()
        .filter(|e| e.xrom.as_ref().map(|x| x.module_id) == Some(24))
        .map(|e| e.xrom.as_ref().unwrap().function_id)
        .collect();
    adv_b_ids.sort_unstable();
    assert_eq!(adv_b_ids.first().copied().unwrap_or(0), 1, "ADV_MATH_B function_ids must start at 1 (independent range)");
    assert_eq!(adv_b_ids.last().copied().unwrap_or(0), 51, "ADV_MATH_B function_ids must end at 51");
    assert_eq!(adv_b_ids.len(), 51, "ADV_MATH_B must have 51 unique function_ids");
}
```

**Critical test 6 adaptation** (dual module_id check):
```rust
#[test]
fn adv_every_entry_has_xrom_module_id_in_22_or_24() {
    for entry in help_entries_adv() {
        let xrom = entry.xrom.as_ref().unwrap_or_else(|| {
            panic!("entry '{}' missing xrom block (C-28.3)", entry.op_variant)
        });
        assert!(
            xrom.module_id == 22 || xrom.module_id == 24,
            "entry '{}' has xrom.module_id == {} — must be 22 (ADV CONV+MTRX) or 24 (ADV MATH+TVM)",
            entry.op_variant, xrom.module_id
        );
    }
}
```

---

### `hp41-cli/tests/phase44_modal_flow.rs` (new file)

**Analog:** `hp41-cli/tests/phase34_modal_flow.rs` (full file, 112 lines)

**File header**:
```rust
//! Phase 44 / Plan 02 Task 3 — Advantage Pac modal-prompt routing tests (ADV-CLI-07).
//!
//! Asserts that AdvantageStep variants surface their OM-cited prompts via
//! `ModalProgram::Advantage(_).current_prompt()` AND that ONLY the three
//! alpha-label steps (MatrixNamePrompt, MtrNamePrompt, FdifeqFunctionNamePrompt)
//! return true for `requires_alpha_label()`.
//!
//! Verification strategy: cross-checks at the CARRIER enum level
//! (`ModalProgram::Advantage(_)`), proving Phase 43's carrier-enum dispatch
//! routes Advantage prompts correctly through the same ModalProgram enum the
//! CLI / GUI / ui::pending_prompt consume.

#![allow(clippy::unwrap_used)]

use hp41_core::ops::advantage::modal::AdvantageStep;
use hp41_core::ops::math1::modal::ModalProgram;
```

**Test structure** (one test per AdvantageStep variant that has a known prompt):

All TVM steps (`TvmN`, `TvmI`, `TvmPv`, `TvmPmt`, `TvmFv`) follow this pattern:
```rust
#[test]
fn advantage_tvm_n_prompt() {
    let prog = ModalProgram::Advantage(AdvantageStep::TvmN);
    assert_eq!(prog.current_prompt(), Some("N=?".to_string()));
    assert!(!prog.requires_alpha_label(), "TvmN must NOT require alpha label");
}
```

`TvmBeginEnd` has no prompt (per `modal.rs` line 68):
```rust
#[test]
fn advantage_tvm_begin_end_no_prompt() {
    let prog = ModalProgram::Advantage(AdvantageStep::TvmBeginEnd);
    assert_eq!(prog.current_prompt(), None, "TvmBeginEnd is a toggle — no text prompt");
    assert!(!prog.requires_alpha_label());
}
```

Alpha-label steps trigger CollectForModal:
```rust
#[test]
fn advantage_matrix_name_prompt_requires_alpha_label() {
    let prog = ModalProgram::Advantage(AdvantageStep::MatrixNamePrompt);
    assert_eq!(prog.current_prompt(), Some("MNAME?".to_string()));
    assert!(prog.requires_alpha_label(), "MatrixNamePrompt triggers CollectForModal auto-open");
}

#[test]
fn advantage_mtr_name_prompt_requires_alpha_label() {
    let prog = ModalProgram::Advantage(AdvantageStep::MtrNamePrompt);
    assert_eq!(prog.current_prompt(), Some("MTR NAME?".to_string()));
    assert!(prog.requires_alpha_label());
}

#[test]
fn advantage_fdifeq_function_name_prompt_requires_alpha_label() {
    let prog = ModalProgram::Advantage(AdvantageStep::FdifeqFunctionNamePrompt);
    assert_eq!(prog.current_prompt(), Some("F NAME?".to_string()));
    assert!(prog.requires_alpha_label(), "FdifeqFunctionNamePrompt triggers CollectForModal");
}
```

Parameterized steps (verify format includes coordinates):
```rust
#[test]
fn advantage_medit_element_prompt_format() {
    let prog = ModalProgram::Advantage(AdvantageStep::MeditElementPrompt(3, 7));
    let prompt = prog.current_prompt().expect("MeditElementPrompt has a prompt");
    assert!(prompt.contains('3') && prompt.contains('7'));
    assert!(!prog.requires_alpha_label());
}

#[test]
fn advantage_cmedit_element_prompt_format() {
    let prog = ModalProgram::Advantage(AdvantageStep::CmeditElementPrompt(1, 2));
    let prompt = prog.current_prompt().expect("CmeditElementPrompt has a prompt");
    assert!(prompt.contains('1') && prompt.contains('2'));
    assert!(!prog.requires_alpha_label());
}

#[test]
fn advantage_ve_component_prompt_format() {
    let prog = ModalProgram::Advantage(AdvantageStep::VeComponentPrompt(2));
    let prompt = prog.current_prompt().expect("VeComponentPrompt has a prompt");
    assert!(prompt.contains('2'));
    assert!(!prog.requires_alpha_label());
}
```

Remaining non-parameterized steps:
```rust
#[test]
fn advantage_matrix_dim_row_prompt() {
    let prog = ModalProgram::Advantage(AdvantageStep::MatrixDimRowPrompt);
    assert_eq!(prog.current_prompt(), Some("ROWS=?".to_string()));
    assert!(!prog.requires_alpha_label());
}

#[test]
fn advantage_matrix_dim_col_prompt() {
    let prog = ModalProgram::Advantage(AdvantageStep::MatrixDimColPrompt);
    assert_eq!(prog.current_prompt(), Some("COLS=?".to_string()));
    assert!(!prog.requires_alpha_label());
}

#[test]
fn advantage_matrx_operation_choice_prompt() {
    let prog = ModalProgram::Advantage(AdvantageStep::MatrxOperationChoice);
    assert_eq!(prog.current_prompt(), Some("MATRX OP?".to_string()));
    assert!(!prog.requires_alpha_label());
}

#[test]
fn advantage_fdifeq_order_prompt() {
    let prog = ModalProgram::Advantage(AdvantageStep::FdifeqOrderPrompt);
    assert_eq!(prog.current_prompt(), Some("ORDER=?".to_string()));
    assert!(!prog.requires_alpha_label());
}
```

---

### `hp41-cli/tests/key_coverage.rs` (modify: add ADV sub-loop)

**Analog:** `hp41-cli/tests/key_coverage.rs` lines 360–405 (Time Pac sub-loop)

**Main loop bitfield upgrade** (line 219): upgrade `0b0000_0111` to `0b0001_1111` (all 5 modules loaded):
```rust
// CURRENT (line 219):
let cli_local = xeq_by_name_local_resolve(&name, 0b0000_0111);

// UPDATED:
let cli_local = xeq_by_name_local_resolve(&name, 0b0001_1111);
```

**Floor assertion upgrade** (lines 259–264): raise from 155 to `>= 269` (155 + 114 Advantage entries):
```rust
// CURRENT (lines 259-264):
assert!(
    probed >= 155,
    "key_coverage probed only {probed} entries — ..."
);

// UPDATED:
// As-shipped pools: v2.2 ~62 + Math1 ~45 + Stat1 ~26 + Time ~35 + Adv ~114 → total ~282.
// Threshold 269 leaves headroom but catches regression where Advantage entries are silently dropped.
assert!(
    probed >= 269,
    "key_coverage probed only {probed} entries — JSON pool is empty, \
     a file failed to load, the filter is wrong, or \
     parse_key_path is over-eager about skipping"
);
```

**Comment update** (lines 211–218): add note about the v3.3 upgrade:
```rust
// Plan 44-XX (Rule 1 fix): upgrade to 0b0001_1111 (v3.3 default — all 5 modules:
// Math 1 + Stat 1 + Time Module + ADV_MATH_A + ADV_MATH_B) so that Advantage Pac
// XEQ-by-name entries from help_entries_all() (added in Phase 44) resolve correctly.
```

**New ADV sub-loop** (add after Time sub-loop at line 405, mirroring lines 360–405):
```rust
// Plan 44-XX: sub-loop for Advantage Pac entries (xrom.module_id == 22 or 24).
// Mirrors the BL-04 Math1 / Stat1 / Time sub-loops but uses 0b0001_1111
// (the v3.3 default — all 5 modules loaded). Both ADV_MATH_A (id=22) and
// ADV_MATH_B (id=24) entries are probed in the same loop since both use
// XEQ-by-name and the same production bitfield.
let mut adv_probed = 0usize;
for entry in entries.iter() {
    if entry.status != "implemented" {
        continue;
    }
    let Some(xrom) = entry.xrom.as_ref() else {
        continue;
    };
    // Check both Advantage Pac XROM module IDs (22 = ADV CONV+MTRX, 24 = ADV MATH+TVM)
    if xrom.module_id != 22 && xrom.module_id != 24 {
        continue;
    }
    let Some(key_path) = entry.key_path.as_deref() else {
        continue;
    };
    let Some(rest) = key_path.strip_prefix("XEQ \"") else {
        continue;
    };
    let Some(name) = rest.strip_suffix('"') else {
        continue;
    };
    adv_probed += 1;
    let resolved = xeq_by_name_local_resolve(name, 0b0001_1111);
    assert!(
        resolved.is_some(),
        "{} via XEQ \"{}\": xeq_by_name_local_resolve with v3.3 default \
         modules loaded (0b0001_1111) returned None — JSON typo or \
         missing ADV_MATH_A/ADV_MATH_B.ops entry?",
        entry.op_variant,
        name
    );
}
assert!(
    adv_probed >= 100,
    "Adv sub-loop probed only {adv_probed} entries — \
     help_entries_all is missing the Advantage Pac pool, or every Adv \
     entry lost its xrom field"
);
```

---

## Shared Patterns

### OnceLock JSON Loading (D-25.17 hard-build-blocker)
**Source:** `hp41-cli/src/help_data.rs` lines 85–100 (base pattern) + lines 149–172 (fourth pool pattern)
**Apply to:** New `ADV_FUNCTIONS_JSON` + `ADV_HELP_ENTRIES` + `help_entries_adv()` in `help_data.rs`

The pattern is strictly: `const JSON: &str = include_str!("path")` → `static ENTRIES: OnceLock<Vec<HelpEntry>> = OnceLock::new()` → `pub fn accessor() -> &'static [HelpEntry] { ENTRIES.get_or_init(|| serde_json::from_str(JSON).expect("filename is malformed — fix the JSON")) }`. The `.expect()` message MUST name the specific JSON file (not a generic message) so CI failures route to the correct source-of-truth.

### Exhaustive Module ID Bitfield Convention
**Source:** `hp41-cli/tests/function_matrix_parity.rs` + `hp41-cli/tests/key_coverage.rs`
**Apply to:** All reverse-parity `xrom_resolve` calls, xrom_shadowing `_resolve_via_xrom_resolve` tests

Per-phase escalation:
- Phase 29 (Math 1 only): `0b0000_0001`
- Phase 34 (Math 1 + Stat 1): `0b0000_0011`
- Phase 39 (Math 1 + Stat 1 + Time): `0b0000_0111`
- **Phase 44 (all 5): `0b0001_1111`**

The production bitfield grows with each phase. Tests probing Advantage Pac entries MUST use `0b0001_1111`. The old bitfields remain in existing sub-loops (Math1 uses `0b0000_0001`, Stat1 uses `0b0000_0011`, etc.) — do not change them.

### `ModalProgram::Advantage` Carrier-Enum Pattern
**Source:** `hp41-core/src/ops/advantage/modal.rs` lines 20–105
**Apply to:** `phase44_modal_flow.rs`

`current_prompt()` and `requires_alpha_label()` are defined as free functions in `advantage/modal.rs` and delegated through `ModalProgram::current_prompt()` / `ModalProgram::requires_alpha_label()` in `math1/modal.rs`. Tests call `ModalProgram::Advantage(AdvantageStep::X).current_prompt()` — the carrier enum, NOT the inner function directly. This proves the dispatch chain routes correctly end-to-end.

### No app.rs Changes Required (ADV-CLI-07)
**Source:** `hp41-cli/src/app.rs` lines 1914–1927 (`maybe_auto_open_collect_for_modal`)

The `maybe_auto_open_collect_for_modal` hook fires after every dispatch and checks `mp.requires_alpha_label()`. Since `ModalProgram::Advantage` was already added in Phase 43 and `AdvantageStep`'s `requires_alpha_label` is already implemented, the CLI alpha-label routing works without any new code in `app.rs`. The Phase 44 test file (`phase44_modal_flow.rs`) verifies this routing at the carrier-enum level — it does NOT need to drive `App::handle_key`.

### Category Prefix Convention for Overlay Sectioning
**Source:** `hp41-cli/src/help_data.rs` lines 213–240 (`help_overlay_rows()`)
**Apply to:** `docs/hp41-advantage-functions.json` category field values

`help_overlay_rows()` groups entries by `category` string and emits `=== {cat} ===` section headers. The prefix must be unique from all prior pools to avoid cross-pool section merging. Required: `"Adv "` prefix (e.g. `"Adv Conv"`, `"Adv Mtrx"`, `"Adv Math"`, `"Adv TVM"`). Do not use `"Math"` (collides with Math 1 pool), `"Stat1 "` (collides with Stat 1), or `"Time "` (collides with Time Pac).

---

## No Analog Found

All Phase 44 files have close analogs. No files fall into this category.

---

## Metadata

**Analog search scope:** `hp41-cli/src/`, `hp41-cli/tests/`, `hp41-core/tests/`, `docs/`
**Files scanned:** 10 source files read in full
**Pattern extraction date:** 2026-05-26
**Key constraint:** `ADV-CLI-03` (`prgm_display.rs` 117 arms) is ALREADY COMPLETE as of commit 7c28d2b — the planner MUST NOT include any task touching `prgm_display.rs`.
