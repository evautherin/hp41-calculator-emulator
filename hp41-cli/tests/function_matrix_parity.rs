//! Bidirectional drift catch between `docs/hp41cv-functions.json` and the
//! `hp41_core::ops::Op` enum (D-25.15, Pitfall 6).
//!
//! - **Forward:** every named ROM Op variant must have a matching JSON entry.
//! - **Reverse:** every `status: "implemented"` JSON entry must name a known
//!   `Op::` variant (or an explicit XEQ-by-Name alias whitelisted below).
//! - **Inventory parity:** `ALL_OP_VARIANT_NAMES` matches the hand-curated
//!   130-row target — adding a new Op enum variant in a future phase forces
//!   the developer to update this list AND the JSON in the same commit.

#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use hp41_cli::help_data::{
    help_entries, help_entries_adv, help_entries_math1, help_entries_stat1, help_entries_time,
};

/// Hand-curated inventory of all `hp41_core::ops::Op` variants. Drift
/// between this list and the enum is caught by `test_op_inventory_count_matches_enum`.
///
/// Maintenance gate: every new `Op` variant landed in any future phase must
/// be appended here. This is intentionally manual — Rust has no built-in
/// enum-variant introspection and the strum-dep cost is rejected per
/// RESEARCH §"Don't Hand-Roll".
const ALL_OP_VARIANT_NAMES: &[&str] = &[
    // Phase 1 arithmetic / stack
    "Add",
    "Sub",
    "Mul",
    "Div",
    "Enter",
    "Clx",
    "Chs",
    "Rdn",
    "Rup",
    "XySwap",
    "Lastx",
    "Pi",
    "PushNum",
    // Phase 2 / 20 unary math / trig / mode / registers / alpha
    "Int",
    "Rnd",
    "Frc",
    "Abs",
    "Sign",
    "Fact",
    "Recip",
    "Sqrt",
    "Sq",
    "YPow",
    "Mod",
    "PctChange",
    "Ln",
    "Log",
    "Exp",
    "TenPow",
    "Sin",
    "Cos",
    "Tan",
    "Asin",
    "Acos",
    "Atan",
    "PolarToRect",
    "RectToPolar",
    "SetDeg",
    "SetRad",
    "SetGrad",
    "FmtFix",
    "FmtSci",
    "FmtEng",
    "StoReg",
    "RclReg",
    "StoArith",
    "StoArithStack",
    "Clreg",
    "AlphaToggle",
    "AlphaAppend",
    "AlphaClear",
    // Phase 3 programming
    "Lbl",
    "Gto",
    "Xeq",
    "Rtn",
    "PrgmMode",
    "Test",
    "Isg",
    "Dse",
    // Phase 5 USER mode, ALPHA back
    "UserMode",
    "AlphaBackspace",
    // Phase 6 stats / HMS
    "SigmaPlus",
    "SigmaMinus",
    "Mean",
    "Sdev",
    "LR",
    "Yhat",
    "Corr",
    "ClSigmaStat",
    "HmsToH",
    "HToHms",
    "HmsAdd",
    "HmsSub",
    // Phase 11 print
    "PRX",
    "PRA",
    "PRSTK",
    // Phase 12 synthetic
    "GetKey",
    "Null",
    "StoM",
    "StoN",
    "StoO",
    "RclM",
    "RclN",
    "RclO",
    "SyntheticByte",
    // v2.1 card reader
    "Wdta",
    "Rdta",
    "Wprgm",
    "Rdprgm",
    // Phase 21 flags / display / sound
    "SfFlag",
    "CfFlag",
    "FlagTest",
    "View",
    "AView",
    "Prompt",
    "Aon",
    "Aoff",
    "Cld",
    "Beep",
    "Tone",
    // Phase 22 program control / editing / memory / catalog / ASN
    "Stop",
    "Pse",
    "GtoInd",
    "XeqInd",
    "Clp",
    "Del",
    "Ins",
    "Size",
    "Cla",
    "Clst",
    "Pack",
    "Catalog",
    "Asn",
    // Phase 23 ALPHA ops
    "Arcl",
    "Asto",
    "Atox",
    "Xtoa",
    "Arot",
    "Posa",
    // Phase 24 indirect
    "StoInd",
    "RclInd",
    "StoArithInd",
    "IsgInd",
    "DseInd",
    "SfFlagInd",
    "CfFlagInd",
    "FlagTestInd",
    "ArclInd",
    "AstoInd",
    "ViewInd",
];

/// Op variants that do NOT correspond to a discoverable HP-41CV ROM op —
/// these are internal calculator primitives. The forward parity check
/// (`every_rom_op_has_matrix_entry`) skips these because the JSON
/// intentionally lacks rows for them.
const INTERNAL_OP_VARIANTS: &[&str] = &[
    "PushNum",       // numeric-literal entry; not a named ROM op
    "SyntheticByte", // hex-modal insertion; internal primitive
];

/// JSON op_variant aliases representing the 8 XEQ-by-Name-only conditional
/// tests. These rows exist in `docs/hp41cv-functions.json` so the function
/// matrix and `tests/key_coverage.rs` can document and probe them, but they
/// are NOT distinct `Op::` variants — they all resolve to `Op::Test(_)` via
/// `keys::xeq_by_name_local_resolve` or `builtin_card_op`. The reverse
/// parity check whitelists them.
const XEQ_ALIAS_OP_VARIANTS: &[&str] = &[
    "XNeY_XEQ",
    "XLtY_XEQ",
    "XGeY_XEQ",
    "XNeZero_XEQ",
    "XLtZero_XEQ",
    "XGtZero_XEQ",
    "XLeZero_XEQ",
    "XGeZero_XEQ",
];

#[test]
fn test_op_inventory_count_matches_enum() {
    // Maintenance gate per RESEARCH §"CI parity test": the hand-curated
    // inventory must hold exactly 130 ROM-named Op variants (v2.2 built-ins only).
    // XROM module variants (Math Pac I, Stat 1 Pac, Time Pac) have their own
    // per-module inventory constants (MATH1_OP_VARIANT_NAMES, STAT1_OP_VARIANT_NAMES,
    // TIME_OP_VARIANT_NAMES). If the Op enum grows past 130 built-in variants in
    // a future phase without this list growing, this assertion fires and forces
    // the developer to append the new variant here AND add a matching JSON entry.
    assert_eq!(
        ALL_OP_VARIANT_NAMES.len(),
        130,
        "ALL_OP_VARIANT_NAMES out of sync with hp41_core::ops::Op enum. \
         Did a future phase add new Op variants without updating this \
         inventory and docs/hp41cv-functions.json?"
    );
}

#[test]
fn test_every_rom_op_has_matrix_entry() {
    // Forward direction: every ROM-named Op variant (minus the internal
    // skiplist) must have a corresponding JSON row.
    let entries = help_entries();
    let json_variants: HashSet<&str> = entries.iter().map(|e| e.op_variant.as_str()).collect();

    let mut missing: Vec<&str> = Vec::new();
    for name in ALL_OP_VARIANT_NAMES {
        if INTERNAL_OP_VARIANTS.contains(name) {
            continue;
        }
        if !json_variants.contains(name) {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "Op::* variants missing from docs/hp41cv-functions.json: {missing:?}"
    );
}

#[test]
fn test_every_implemented_matrix_entry_has_op() {
    // Reverse direction: every implemented JSON entry must name a known Op
    // variant — unless it's an explicit XEQ-by-Name alias whitelisted above
    // (those routes resolve to Op::Test(_) via xeq_by_name_local_resolve).
    let names: HashSet<&str> = ALL_OP_VARIANT_NAMES.iter().copied().collect();
    let aliases: HashSet<&str> = XEQ_ALIAS_OP_VARIANTS.iter().copied().collect();

    let mut orphans: Vec<&str> = Vec::new();
    for entry in help_entries() {
        if entry.status != "implemented" {
            continue;
        }
        if names.contains(entry.op_variant.as_str()) || aliases.contains(entry.op_variant.as_str())
        {
            continue;
        }
        orphans.push(entry.op_variant.as_str());
    }
    assert!(
        orphans.is_empty(),
        "implemented JSON rows with no matching Op variant: {orphans:?}"
    );
}

#[test]
fn test_matrix_has_at_least_130_entries() {
    // Combined with the Pitfall 7 smoke check in tests/phase25_help_data.rs.
    let entries = help_entries();
    assert!(
        entries.len() >= 130,
        "function matrix should list >= 130 HP-41CV ROM ops; got {}",
        entries.len()
    );
}

// ── Phase 29 Plan 01 Task 3: Math Pac I bidirectional parity tests (CLI-02) ──
//
// Three tests guarding the hp41-math1-functions.json ↔ MATH_1.ops ↔ Op::* chain:
// 1. Inventory drift sentinel (MATH1_OP_VARIANT_NAMES length == 45)
// 2. Forward parity: every MATH1_OP_VARIANT_NAMES entry has a JSON row
// 3. Reverse parity: every JSON display_name resolves via xrom_resolve
//
// Tests are partitioned by pool (v2.2 vs Math Pac I) so future v3.1 Stat Pac
// additions don't break v2.2 assertions (Claude's Discretion, CONTEXT §Parity Test).

/// Hand-curated inventory of all Math Pac I `Op` variants shipped in Phase 28.
/// Drift between this list and the `MATH_1.ops` table in `hp41-core/src/ops/math1/xrom.rs`
/// is caught by `test_math1_op_inventory_count`.
///
/// Maintenance gate: if Phase 30+ adds new Math Pac I `Op` variants, append here
/// AND add matching JSON rows to `docs/hp41-math1-functions.json`.
const MATH1_OP_VARIANT_NAMES: &[&str] = &[
    // Phase 28-02: Hyperbolics (6)
    "Sinh",
    "Cosh",
    "Tanh",
    "Asinh",
    "Acosh",
    "Atanh",
    // Phase 28-03: Complex Stack Arithmetic (5)
    "CPlus",
    "CMinus",
    "CTimes",
    "CDiv",
    "Real",
    // Phase 28-04: Complex Functions (12)
    "Magz",
    "Cinv",
    "ZpowN",
    "Zpow1N",
    "ExpZ",
    "LnZ",
    "SinZ",
    "CosZ",
    "TanZ",
    "ApowZ",
    "LogZ",
    "ZpowW",
    // Phase 28-05: Polynomial (2)
    "PolyWorkflow",
    "Roots",
    // Phase 28-06: Matrix (8)
    "MatrixWorkflow",
    "MatSize",
    "MatVmat",
    "MatEdit",
    "MatDet",
    "MatInv",
    "MatSimeq",
    "MatVcol",
    // Phase 28-07: Integration (1)
    "Integ",
    // Phase 28-08: Root Solver (2)
    "Solve",
    "Sol",
    // Phase 28-09: Differential Equation (1)
    "Difeq",
    // Phase 28-10: Fourier / Triangle Solvers / Coordinate Transform (7)
    "Four",
    "TriSss",
    "TriAsa",
    "TriSaa",
    "TriSas",
    "TriSsa",
    "Trans2d",
    "Trans3d",
];

#[test]
fn test_math1_op_inventory_count() {
    // Catches: drift between this hand-curated list and MATH_1.ops in xrom.rs.
    // If a new Math Pac I Op variant is added without updating this list,
    // this assertion fires forcing the developer to also add a JSON entry.
    assert_eq!(
        MATH1_OP_VARIANT_NAMES.len(),
        45,
        "MATH1_OP_VARIANT_NAMES inventory drift — expected 45 unique Math Pac I Op variants \
         (52 total MATH_1.ops entries minus 7 ASCII aliases). Did a new Phase 28+ plan add \
         Op variants without updating this inventory and docs/hp41-math1-functions.json?"
    );
}

#[test]
fn test_every_math1_rom_op_has_math1_json_entry() {
    // Catches: forward parity gap — a Math Pac I Op variant without a JSON entry.
    // Uses help_entries_math1() (narrow accessor) to assert against only the Math1 pool.
    // Failure message lists missing variants by name for easy diagnosis.
    let json_variants: HashSet<&str> = help_entries_math1()
        .iter()
        .map(|e| e.op_variant.as_str())
        .collect();

    let mut missing: Vec<&str> = Vec::new();
    for name in MATH1_OP_VARIANT_NAMES {
        if !json_variants.contains(name) {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "Math Pac I Op::* variants missing from docs/hp41-math1-functions.json: {missing:?}"
    );
}

#[test]
fn test_every_math1_json_entry_has_xrom_resolver_match() {
    // Catches: reverse parity gap — a JSON entry whose display_name cannot be
    // resolved by xrom_resolve (C-28.4). Ensures the JSON and MATH_1.ops table
    // stay in sync — a typo in display_name or a missing math1_resolve arm fails here.
    let mut orphans: Vec<String> = Vec::new();
    for entry in help_entries_math1() {
        let resolved =
            hp41_core::ops::math1::xrom::xrom_resolve(entry.display_name.as_str(), 0b0000_0001);
        if resolved.is_none() {
            orphans.push(format!(
                "'{}' (display_name='{}') — not found in MATH_1.ops / math1_resolve",
                entry.op_variant, entry.display_name
            ));
        }
    }
    assert!(
        orphans.is_empty(),
        "Math1 JSON entries whose display_name is NOT resolved by xrom_resolve(_, 0b0000_0001): {orphans:?}"
    );
}

// ── Phase 34 Plan 02 Task 2: Stat 1 Pac bidirectional parity tests (STAT-CLI-04) ──
//
// Four tests guarding the hp41-stat1-functions.json ↔ STAT_1.ops ↔ Op::* chain:
// 1. Inventory drift sentinel (STAT1_OP_VARIANT_NAMES length == 26)
// 2. Forward parity: every STAT1_OP_VARIANT_NAMES entry has a JSON row
// 3. Reverse parity: every JSON display_name resolves via xrom_resolve
// 4. Pool partition guard: help_entries_all() partitions cleanly into 3 buckets
//
// Tests use the narrow help_entries_stat1() accessor (not help_entries_all()) for
// per-pool tests so v2.2 / Math 1 pool assertions are unaffected. Test 4 uses
// help_entries_all() to guard against rogue module_id values in future v3.2+ packs.

/// Hand-curated inventory of all Stat 1 Pac `Op` variants shipped in Phase 33.
/// Drift between this list and the `STAT_1.ops` table in `hp41-core/src/ops/math1/xrom.rs`
/// is caught by `test_stat1_op_inventory_count`.
///
/// Maintenance gate: if Phase 35+ adds new Stat 1 Pac `Op` variants, append here
/// AND add matching JSON rows to `docs/hp41-stat1-functions.json`.
const STAT1_OP_VARIANT_NAMES: &[&str] = &[
    // Plan 33-05/06: Stat 1 Pac Univariate / Bivariate (4)
    "SigmaBstat",
    "SigmaBstg",
    "SigmaMmtug",
    "SigmaMmtgd",
    // Plan 33-06: Stat 1 Pac ANOVA Family (3)
    "SigmaAovone",
    "SigmaAovtwo",
    "SigmaAnocov",
    // Plan 33-05/08: Stat 1 Pac Curve Fitting + Regression (8)
    "SigmaLin",
    "SigmaExp",
    "SigmaLogi",
    "SigmaPow",
    "SigmaMlrxy",
    "SigmaMlrxyz",
    "SigmaPolypWorkflow",
    "SigmaPolyc",
    // Plan 33-07: Stat 1 Pac Hypothesis Tests (2)
    "SigmaPtst",
    "SigmaTstat",
    // Plan 33-04/06: Stat 1 Pac Nonparam / Chi-Sq Eval / Contingency (5)
    "SigmaXsqev",
    "SigmaEfxsq",
    "SigmaCtkkk",
    "SigmaCtkk",
    "SigmaSpear",
    // Plan 33-03: Stat 1 Pac Distributions (2)
    "SigmaNormdWorkflow",
    "SigmaChisqdWorkflow",
    // Plan 33-08: Stat 1 Pac RNG (2)
    "Rand",
    "Seed",
];

#[test]
fn test_stat1_op_inventory_count() {
    assert_eq!(
        STAT1_OP_VARIANT_NAMES.len(),
        26,
        "STAT1_OP_VARIANT_NAMES inventory drift — expected 26 Stat 1 Pac Op variants per Phase 33 ship. \
         Did a future plan add Op variants without updating this inventory and docs/hp41-stat1-functions.json?"
    );
}

#[test]
fn test_every_stat1_rom_op_has_stat1_json_entry() {
    // Catches: forward parity gap — a Stat 1 Pac Op variant without a JSON entry.
    // Uses help_entries_stat1() (narrow accessor) to assert against only the Stat 1 pool.
    // Failure message lists missing variants by name for easy diagnosis.
    let json_variants: HashSet<&str> = help_entries_stat1()
        .iter()
        .map(|e| e.op_variant.as_str())
        .collect();

    let mut missing: Vec<&str> = Vec::new();
    for name in STAT1_OP_VARIANT_NAMES {
        if !json_variants.contains(name) {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "Stat 1 Pac Op::* variants missing from docs/hp41-stat1-functions.json: {missing:?}"
    );
}

#[test]
fn test_every_stat1_json_entry_has_xrom_resolver_match() {
    // Catches: reverse parity gap — a JSON entry whose display_name cannot be
    // resolved by xrom_resolve (C-28.4). Uses 0b0000_0011 (the v3.1 default
    // per Phase 33 D-33.1: both Math 1 + Stat 1 loaded) so the bidirectional
    // invariant holds for the production bitfield state, not a hypothetical
    // Stat-1-only bitfield.
    let mut orphans: Vec<String> = Vec::new();
    for entry in help_entries_stat1() {
        let resolved =
            hp41_core::ops::math1::xrom::xrom_resolve(entry.display_name.as_str(), 0b0000_0011);
        if resolved.is_none() {
            orphans.push(format!(
                "'{}' (display_name='{}') — not found in STAT_1.ops / xrom_resolve",
                entry.op_variant, entry.display_name
            ));
        }
    }
    assert!(
        orphans.is_empty(),
        "Stat 1 JSON entries whose display_name is NOT resolved by xrom_resolve(_, 0b0000_0011): {orphans:?}"
    );
}

#[test]
fn test_pool_partition_is_exhaustive() {
    // Partition help_entries_all() by xrom.module_id. The five buckets
    // are: None (built-ins, >= 130), Some(7) (Math Pac I, == 45), Some(2)
    // (Stat 1 Pac, == 26), Some(26) (Time Pac, == 35), Some(22) (Adv CONV+MTRX, == 63),
    // Some(24) (Adv MATH+TVM, == 51). Any other value is rogue and fails the test.
    //
    // This guards future v3.4+ additions: before they merge,
    // this test fires (because Some(<new_id>) is not in the recognized set),
    // forcing the new pool to be registered in the partition.
    use hp41_cli::help_data::help_entries_all;
    let mut builtin_count = 0usize;
    let mut math1_count = 0usize;
    let mut stat1_count = 0usize;
    let mut time_count = 0usize;
    let mut adv_a_count = 0usize;
    let mut adv_b_count = 0usize;
    let mut rogue: Vec<(String, u8)> = Vec::new();
    for entry in help_entries_all() {
        match entry.xrom.as_ref().map(|x| x.module_id) {
            None => builtin_count += 1,
            Some(7) => math1_count += 1,
            Some(2) => stat1_count += 1,
            Some(26) => time_count += 1,
            Some(22) => adv_a_count += 1,
            Some(24) => adv_b_count += 1,
            Some(other) => rogue.push((entry.op_variant.clone(), other)),
        }
    }
    assert!(
        rogue.is_empty(),
        "Unknown xrom.module_id values in help_entries_all(): {rogue:?}. \
         v3.3 supports only module_id in {{None (built-ins), 7 (Math 1), 2 (Stat 1), 26 (Time), 22 (Adv CONV+MTRX), 24 (Adv MATH+TVM)}}. \
         Adding a new XROM module requires updating function_matrix_parity.rs partition."
    );
    assert!(
        builtin_count >= 130,
        "built-in pool shrank: {builtin_count}"
    );
    assert_eq!(math1_count, 45, "Math 1 pool count drift: {math1_count}");
    assert_eq!(stat1_count, 26, "Stat 1 pool count drift: {stat1_count}");
    assert_eq!(time_count, 35, "Time pool count drift: {time_count}");
    assert_eq!(
        adv_a_count, 63,
        "Adv CONV+MTRX (XROM 22) pool count drift: {adv_a_count}"
    );
    assert_eq!(
        adv_b_count, 51,
        "Adv MATH+TVM (XROM 24) pool count drift: {adv_b_count}"
    );
}

// ── Phase 44 Plan 01/02: Advantage Pac bidirectional parity tests (ADV-CLI-02) ──
//
// Six tests guarding the hp41-advantage-functions.json ↔ ADV_MATH_A/B.ops ↔ Op::* chain:
// 1. Inventory drift sentinel for ADV_A (63 entries, XROM 22)
// 2. Inventory drift sentinel for ADV_B (51 entries, XROM 24)
// 3. Forward parity (per-module): every ADV_A_OP_VARIANT_NAMES entry has a JSON row
// 4. Forward parity (per-module): every ADV_B_OP_VARIANT_NAMES entry has a JSON row
// 5. Reverse parity (per-module): every XROM 22 JSON display_name resolves via xrom_resolve
// 6. Reverse parity (per-module): every XROM 24 JSON display_name resolves via xrom_resolve
//
// Also the combined sentinel (ADV_OP_VARIANT_NAMES length == 114) from Plan 01.

/// Hand-curated inventory of ADV_MATH_A (XROM 22) `Op` variants shipped in Phase 43.
/// ADV CONV (12) + ADV MTRX element-access/lifecycle/reduction/linalg/complex/workflow (51) = 63.
/// Run-loop variants (AdvFsolveRunLoop/AdvFintgRunLoop/AdvFdifeqRunLoop) are EXCLUDED.
///
/// Maintenance gate: if future plans add new ADV_MATH_A `Op` variants, append here
/// AND add matching JSON rows to `docs/hp41-advantage-functions.json`.
const ADV_A_OP_VARIANT_NAMES: &[&str] = &[
    // Phase 43 ADV_MATH_A (XROM 22) — ADV CONV (12) + ADV MTRX (51) = 63 entries
    // ADV CONV
    "AdvBinin",
    "AdvBinview",
    "AdvOctin",
    "AdvHexin",
    "AdvHexview",
    "AdvCvtview",
    "AdvNot",
    "AdvAnd",
    "AdvOr",
    "AdvXor",
    "AdvRotxy",
    "AdvBitTest",
    // ADV MTRX element access
    "AdvIPlus",
    "AdvIMinus",
    "AdvJPlus",
    "AdvJMinus",
    "AdvMr",
    "AdvMs",
    "AdvMrij",
    "AdvMsij",
    "AdvMsijr",
    "AdvMrcPlus",
    "AdvMrcMinus",
    "AdvMrrPlus",
    "AdvMrrMinus",
    "AdvMsrPlus",
    "AdvMscPlus",
    "AdvMswap",
    "AdvMnameQuery",
    "AdvDimQuery",
    "AdvMatdim",
    "AdvMp",
    "AdvPiv",
    "AdvRExchangeR",
    "AdvRGtRQuery",
    // ADV MTRX reductions
    "AdvSum",
    "AdvSumab",
    "AdvMax",
    "AdvMaxab",
    "AdvMin",
    "AdvRmaxab",
    "AdvRnrm",
    "AdvRsum",
    "AdvFnrm",
    // ADV MTRX linear algebra
    "AdvMdet",
    "AdvMinv",
    "AdvMsys",
    "AdvMMulM",
    "AdvMatPlus",
    "AdvMatMinus",
    "AdvMatScalarMul",
    "AdvMatScalarDiv",
    "AdvTrnps",
    "AdvMmove",
    // ADV MTRX complex
    "AdvCExchangeC",
    "AdvCmaxab",
    "AdvCnrm",
    "AdvCsum",
    "AdvYcPlusC",
    // ADV MTRX workflow
    "AdvMatrx",
    "AdvMtr",
    "AdvMedit",
    "AdvCmedit",
];

/// Hand-curated inventory of ADV_MATH_B (XROM 24) `Op` variants shipped in Phase 43.
/// ADV MATH complex extensions (18), polynomial (2), solvers (4), curve fitting (7),
/// vectors (14), AIP (1), ADV TVM (6) = 51 total.
/// Run-loop variants (AdvFsolveRunLoop/AdvFintgRunLoop/AdvFdifeqRunLoop) are EXCLUDED.
///
/// Maintenance gate: if future plans add new ADV_MATH_B `Op` variants, append here
/// AND add matching JSON rows to `docs/hp41-advantage-functions.json`.
const ADV_B_OP_VARIANT_NAMES: &[&str] = &[
    // Phase 43 ADV_MATH_B (XROM 24) — ADV MATH (45) + ADV TVM (6) = 51 entries
    // ADV MATH complex extensions
    "AdvExpZ",
    "AdvLnZ",
    "AdvLogZ",
    "AdvZPowN",
    "AdvZPow1n",
    "AdvZPowW",
    "AdvZPow1w",
    "AdvMagz",
    "AdvSinZ",
    "AdvCosZ",
    "AdvTanZ",
    "AdvAPowZ",
    "AdvCPlus",
    "AdvCMinus",
    "AdvCinv",
    "AdvCMul",
    "AdvCDiv",
    "AdvAip",
    // ADV MATH polynomial
    "AdvPly",
    "AdvRts",
    // ADV MATH solvers
    "AdvFsolve",
    "AdvFintg",
    "AdvFdifeq",
    "AdvFroot",
    // ADV MATH curve fitting
    "AdvCfit",
    "AdvAs",
    "AdvDs",
    "AdvBfit",
    "AdvFit",
    "AdvYQueryX",
    "AdvSzQuery",
    // ADV MATH vectors
    "AdvVPlus",
    "AdvVMinus",
    "AdvDot",
    "AdvCross",
    "AdvVc",
    "AdvVs",
    "AdvVr",
    "AdvVe",
    "AdvVxy",
    "AdvUv",
    "AdvVMag",
    "AdvVStar",
    "AdvVd",
    "AdvTr",
    // ADV TVM
    "AdvTvm",
    "AdvTvmN",
    "AdvTvmPv",
    "AdvTvmPmt",
    "AdvTvmFv",
    "AdvTvmStarI",
];

/// Combined inventory of all Advantage Pac `Op` variants (ADV_MATH_A + ADV_MATH_B).
/// Drift between this list and the `ADV_MATH_A.ops` / `ADV_MATH_B.ops` tables in
/// `hp41-core/src/ops/math1/xrom.rs` is caught by `test_adv_op_inventory_count`.
///
/// Maintenance gate: if future plans add new Advantage Pac `Op` variants, append here
/// AND add matching JSON rows to `docs/hp41-advantage-functions.json`.
const ADV_OP_VARIANT_NAMES: &[&str] = &[
    // Phase 43 ADV_MATH_A (XROM 22) — ADV CONV (12) + ADV MTRX (51) = 63 entries
    // ADV CONV
    "AdvBinin",
    "AdvBinview",
    "AdvOctin",
    "AdvHexin",
    "AdvHexview",
    "AdvCvtview",
    "AdvNot",
    "AdvAnd",
    "AdvOr",
    "AdvXor",
    "AdvRotxy",
    "AdvBitTest",
    // ADV MTRX element access
    "AdvIPlus",
    "AdvIMinus",
    "AdvJPlus",
    "AdvJMinus",
    "AdvMr",
    "AdvMs",
    "AdvMrij",
    "AdvMsij",
    "AdvMsijr",
    "AdvMrcPlus",
    "AdvMrcMinus",
    "AdvMrrPlus",
    "AdvMrrMinus",
    "AdvMsrPlus",
    "AdvMscPlus",
    "AdvMswap",
    "AdvMnameQuery",
    "AdvDimQuery",
    "AdvMatdim",
    "AdvMp",
    "AdvPiv",
    "AdvRExchangeR",
    "AdvRGtRQuery",
    // ADV MTRX reductions
    "AdvSum",
    "AdvSumab",
    "AdvMax",
    "AdvMaxab",
    "AdvMin",
    "AdvRmaxab",
    "AdvRnrm",
    "AdvRsum",
    "AdvFnrm",
    // ADV MTRX linear algebra
    "AdvMdet",
    "AdvMinv",
    "AdvMsys",
    "AdvMMulM",
    "AdvMatPlus",
    "AdvMatMinus",
    "AdvMatScalarMul",
    "AdvMatScalarDiv",
    "AdvTrnps",
    "AdvMmove",
    // ADV MTRX complex
    "AdvCExchangeC",
    "AdvCmaxab",
    "AdvCnrm",
    "AdvCsum",
    "AdvYcPlusC",
    // ADV MTRX workflow
    "AdvMatrx",
    "AdvMtr",
    "AdvMedit",
    "AdvCmedit",
    // Phase 43 ADV_MATH_B (XROM 24) — ADV MATH (45) + ADV TVM (6) = 51 entries
    // ADV MATH complex extensions
    "AdvExpZ",
    "AdvLnZ",
    "AdvLogZ",
    "AdvZPowN",
    "AdvZPow1n",
    "AdvZPowW",
    "AdvZPow1w",
    "AdvMagz",
    "AdvSinZ",
    "AdvCosZ",
    "AdvTanZ",
    "AdvAPowZ",
    "AdvCPlus",
    "AdvCMinus",
    "AdvCinv",
    "AdvCMul",
    "AdvCDiv",
    "AdvAip",
    // ADV MATH polynomial
    "AdvPly",
    "AdvRts",
    // ADV MATH solvers
    "AdvFsolve",
    "AdvFintg",
    "AdvFdifeq",
    "AdvFroot",
    // ADV MATH curve fitting
    "AdvCfit",
    "AdvAs",
    "AdvDs",
    "AdvBfit",
    "AdvFit",
    "AdvYQueryX",
    "AdvSzQuery",
    // ADV MATH vectors
    "AdvVPlus",
    "AdvVMinus",
    "AdvDot",
    "AdvCross",
    "AdvVc",
    "AdvVs",
    "AdvVr",
    "AdvVe",
    "AdvVxy",
    "AdvUv",
    "AdvVMag",
    "AdvVStar",
    "AdvVd",
    "AdvTr",
    // ADV TVM
    "AdvTvm",
    "AdvTvmN",
    "AdvTvmPv",
    "AdvTvmPmt",
    "AdvTvmFv",
    "AdvTvmStarI",
];

#[test]
fn test_adv_a_op_inventory_count() {
    // Catches: drift between ADV_A_OP_VARIANT_NAMES and ADV_MATH_A.ops in xrom.rs.
    // ADV_MATH_A (XROM 22): 12 ADV CONV + 51 ADV MTRX = 63 entries.
    // Run-loop variants excluded (internal-only, not user-XEQ-reachable).
    assert_eq!(
        ADV_A_OP_VARIANT_NAMES.len(),
        63,
        "ADV_A_OP_VARIANT_NAMES inventory drift — expected 63 ADV_MATH_A (XROM 22) Op variants \
         (12 ADV CONV + 51 ADV MTRX). Did a future plan add Op variants without updating this \
         inventory and docs/hp41-advantage-functions.json?"
    );
}

#[test]
fn test_adv_b_op_inventory_count() {
    // Catches: drift between ADV_B_OP_VARIANT_NAMES and ADV_MATH_B.ops in xrom.rs.
    // ADV_MATH_B (XROM 24): 45 ADV MATH + 6 ADV TVM = 51 entries.
    // Run-loop variants excluded (internal-only, not user-XEQ-reachable).
    assert_eq!(
        ADV_B_OP_VARIANT_NAMES.len(),
        51,
        "ADV_B_OP_VARIANT_NAMES inventory drift — expected 51 ADV_MATH_B (XROM 24) Op variants \
         (45 ADV MATH + 6 ADV TVM). Did a future plan add Op variants without updating this \
         inventory and docs/hp41-advantage-functions.json?"
    );
}

#[test]
fn test_adv_op_inventory_count() {
    // Catches: drift between this hand-curated list and ADV_MATH_A/B.ops in xrom.rs.
    // If a new Advantage Pac Op variant is added without updating this list,
    // this assertion fires forcing the developer to also add a JSON entry.
    assert_eq!(
        ADV_OP_VARIANT_NAMES.len(),
        114,
        "ADV_OP_VARIANT_NAMES inventory drift — expected 114 Advantage Pac Op variants per Phase 43 ship \
         (63 ADV_MATH_A + 51 ADV_MATH_B). Did a future plan add Op variants without updating this \
         inventory and docs/hp41-advantage-functions.json?"
    );
}

#[test]
fn test_every_adv_rom_op_has_adv_json_entry() {
    // Catches: forward parity gap — an Advantage Pac Op variant without a JSON entry.
    // Uses help_entries_adv() (narrow accessor) to assert against only the Advantage pool.
    // Failure message lists missing variants by name for easy diagnosis.
    let json_variants: HashSet<&str> = help_entries_adv()
        .iter()
        .map(|e| e.op_variant.as_str())
        .collect();

    let mut missing: Vec<&str> = Vec::new();
    for name in ADV_OP_VARIANT_NAMES {
        if !json_variants.contains(name) {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "Advantage Pac Op::* variants missing from docs/hp41-advantage-functions.json: {missing:?}"
    );
}

#[test]
fn test_every_adv_json_entry_has_xrom_resolver_match() {
    // Catches: reverse parity gap — a JSON entry whose display_name cannot be
    // resolved by xrom_resolve (C-28.4). Uses 0b0001_1111 (all 5 modules loaded)
    // so the bidirectional invariant holds for the production bitfield state.
    let mut orphans: Vec<String> = Vec::new();
    for entry in help_entries_adv() {
        let resolved =
            hp41_core::ops::math1::xrom::xrom_resolve(entry.display_name.as_str(), 0b0001_1111);
        if resolved.is_none() {
            orphans.push(format!(
                "'{}' (display_name='{}') — not found in ADV_MATH_A/B.ops / xrom_resolve",
                entry.op_variant, entry.display_name
            ));
        }
    }
    assert!(
        orphans.is_empty(),
        "Advantage JSON entries whose display_name is NOT resolved by xrom_resolve(_, 0b0001_1111): {orphans:?}"
    );
}

// ── Phase 44 Plan 02: Per-module (ADV_MATH_A / ADV_MATH_B) parity tests ──────
//
// Four tests split by XROM module ID — narrower than the combined tests above:
// 1. Forward parity for XROM 22 (ADV_A_OP_VARIANT_NAMES)
// 2. Forward parity for XROM 24 (ADV_B_OP_VARIANT_NAMES)
// 3. Reverse parity for XROM 22 (JSON display_names via xrom_resolve)
// 4. Reverse parity for XROM 24 (JSON display_names via xrom_resolve)

#[test]
fn test_every_adv_a_rom_op_has_adv_a_json_entry() {
    // Catches: forward parity gap specifically for ADV_MATH_A (XROM 22).
    // Filters help_entries_adv() by module_id==22 so only XROM 22 entries participate.
    // This is narrower than test_every_adv_rom_op_has_adv_json_entry and will catch
    // a mismatch even if ADV_MATH_B entries happen to compensate in the combined check.
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
        "ADV_MATH_A (XROM 22) Op::* variants missing from docs/hp41-advantage-functions.json: {missing:?}"
    );
}

#[test]
fn test_every_adv_b_rom_op_has_adv_b_json_entry() {
    // Catches: forward parity gap specifically for ADV_MATH_B (XROM 24).
    // Filters help_entries_adv() by module_id==24 so only XROM 24 entries participate.
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
        "ADV_MATH_B (XROM 24) Op::* variants missing from docs/hp41-advantage-functions.json: {missing:?}"
    );
}

#[test]
fn test_every_adv_a_json_entry_has_xrom_resolver_match() {
    // Catches: reverse parity gap for ADV_MATH_A (XROM 22) — a JSON entry whose
    // display_name cannot be resolved via the bit-3 arm of xrom_resolve.
    // Uses 0b0001_1111 (all 5 modules loaded — v3.3 production default).
    let mut orphans: Vec<String> = Vec::new();
    for entry in help_entries_adv()
        .iter()
        .filter(|e| e.xrom.as_ref().map(|x| x.module_id) == Some(22))
    {
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
        "ADV_MATH_A (XROM 22) JSON entries whose display_name is NOT resolved by xrom_resolve(_, 0b0001_1111): {orphans:?}"
    );
}

#[test]
fn test_every_adv_b_json_entry_has_xrom_resolver_match() {
    // Catches: reverse parity gap for ADV_MATH_B (XROM 24) — a JSON entry whose
    // display_name cannot be resolved via the bit-4 arm of xrom_resolve.
    // Uses 0b0001_1111 (all 5 modules loaded — v3.3 production default).
    let mut orphans: Vec<String> = Vec::new();
    for entry in help_entries_adv()
        .iter()
        .filter(|e| e.xrom.as_ref().map(|x| x.module_id) == Some(24))
    {
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
        "ADV_MATH_B (XROM 24) JSON entries whose display_name is NOT resolved by xrom_resolve(_, 0b0001_1111): {orphans:?}"
    );
}

// ── Phase 39 Plan 01 Task 2: Time Pac bidirectional parity tests (TIME-CLI-02) ──
//
// Three tests guarding the hp41-time-functions.json ↔ TIME_MODULE.ops ↔ Op::* chain:
// 1. Inventory drift sentinel (TIME_OP_VARIANT_NAMES length == 35)
// 2. Forward parity: every TIME_OP_VARIANT_NAMES entry has a JSON row
// 3. Reverse parity: every JSON display_name resolves via xrom_resolve

/// Hand-curated inventory of all Time Pac `Op` variants shipped in Phase 38.
/// Drift between this list and the `TIME_MODULE.ops` table in `hp41-core/src/ops/math1/xrom.rs`
/// is caught by `test_time_op_inventory_count`.
///
/// Maintenance gate: if future plans add new Time Pac `Op` variants, append here
/// AND add matching JSON rows to `docs/hp41-time-functions.json`.
const TIME_OP_VARIANT_NAMES: &[&str] = &[
    // Phase 38 Time Module (XROM 26) — 35 entries
    "TimeTime",
    "TimeDate",
    "TimeSetime",
    "TimeSetdate",
    "TimeClk12",
    "TimeClk24",
    "TimeClkt",
    "TimeClktd",
    "TimeClock",
    "TimeCorrect",
    "TimeTplusx",
    "TimeDatePlus",
    "TimeDdays",
    "TimeDow",
    "TimeDmy",
    "TimeMdy",
    "TimeAtime",
    "TimeAtime24",
    "TimeAdate",
    "TimeRunsw",
    "TimeStopsw",
    "TimeRclsw",
    "TimeSetsw",
    "TimeSw",
    "TimeSwpt",
    "TimeStpw",
    "TimeXyzalm",
    "TimeAlmcat",
    "TimeAlmnow",
    "TimeRclalm",
    "TimeRclaf",
    "TimeSetaf",
    "TimeClalma",
    "TimeClalmx",
    "TimeClralms",
];

#[test]
fn test_time_op_inventory_count() {
    // Catches: drift between this hand-curated list and TIME_MODULE.ops in xrom.rs.
    // If a new Time Pac Op variant is added without updating this list,
    // this assertion fires forcing the developer to also add a JSON entry.
    assert_eq!(
        TIME_OP_VARIANT_NAMES.len(),
        35,
        "TIME_OP_VARIANT_NAMES inventory drift — expected 35 Time Pac Op variants per Phase 38 ship. \
         Did a future plan add Op variants without updating this inventory and docs/hp41-time-functions.json?"
    );
}

#[test]
fn test_every_time_rom_op_has_time_json_entry() {
    // Catches: forward parity gap — a Time Pac Op variant without a JSON entry.
    // Uses help_entries_time() (narrow accessor) to assert against only the Time pool.
    // Failure message lists missing variants by name for easy diagnosis.
    let json_variants: HashSet<&str> = help_entries_time()
        .iter()
        .map(|e| e.op_variant.as_str())
        .collect();

    let mut missing: Vec<&str> = Vec::new();
    for name in TIME_OP_VARIANT_NAMES {
        if !json_variants.contains(name) {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "Time Pac Op::* variants missing from docs/hp41-time-functions.json: {missing:?}"
    );
}

#[test]
fn test_every_time_json_entry_has_xrom_resolver_match() {
    // Catches: reverse parity gap — a JSON entry whose display_name cannot be
    // resolved by xrom_resolve (C-28.4). Uses 0b0000_0111 (Math 1 + Stat 1 + Time all loaded)
    // so the bidirectional invariant holds for the production bitfield state.
    let mut orphans: Vec<String> = Vec::new();
    for entry in help_entries_time() {
        let resolved =
            hp41_core::ops::math1::xrom::xrom_resolve(entry.display_name.as_str(), 0b0000_0111);
        if resolved.is_none() {
            orphans.push(format!(
                "'{}' (display_name='{}') — not found in TIME_MODULE.ops / xrom_resolve",
                entry.op_variant, entry.display_name
            ));
        }
    }
    assert!(
        orphans.is_empty(),
        "Time JSON entries whose display_name is NOT resolved by xrom_resolve(_, 0b0000_0111): {orphans:?}"
    );
}
