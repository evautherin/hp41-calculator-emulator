//! Phase lu0 D-lu0-02 — Tap-to-run resolver drift guard.
//!
//! **Rationale:** The GUI All Functions overlay dispatches `xeq_<display_name>`
//! for every runnable entry. This string crosses the IPC boundary into
//! `key_map.rs`'s `xeq_` prefix handler → `Op::Xeq(label)` → `op_xeq()` →
//! `builtin_card_op(label)` || `xrom_resolve(label, modules)`. Both resolvers
//! are **EXACT-MATCH, CASE-SENSITIVE** (program.rs doc-comment at ~L1355).
//!
//! This test asserts that for every `status == "implemented"` JSON entry
//! OUTSIDE the NON_TAPPABLE set, the `display_name` resolves via one of the
//! two resolvers under the default module mask (`0b0001_1111`). If a future
//! JSON rename makes a tappable display_name diverge from its resolver mnemonic,
//! THIS test fails before the overlay ships a dead button (D-07: never dispatch
//! an unresolvable id).
//!
//! Cross-reference: the TS `NON_TAPPABLE` constant in
//! `hp41-gui/src/help_data.ts` enumerates the SAME 61 op_variants. If you
//! update either list you MUST update the other. The count-pin assertion below
//! (`NON_TAPPABLE.len() == 61`) prevents silent drift.

#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use hp41_cli::help_data::{
    help_entries, help_entries_adv, help_entries_math1, help_entries_stat1, help_entries_time,
    help_entries_xmem,
};

/// The 61 op_variants that are NOT runnable by XEQ-by-name.
///
/// ALL 61 fall into three classes — none XEQ-runnable on a real HP-41 either:
///   1. Parameterized ops (STO, RCL, FIX, SCI, ENG, SF, CF, ISG, DSE, VIEW, TONE, ARCL,
///      ASTO, GTO, XEQ, LBL, CLP, DEL, ASN, STO+/-/*// and all IND variants).
///   2. Immediate stack/entry keys (+ - * / ENTER CLX CHS Rv X<>Y LASTX %CH).
///   3. Mode / composite-placeholder rows (ALPHA, ALPHA char, ALPHA <-, PRGM, USER,
///      CATALOG, GETKEY, NULL, X?Y / X?0, FS?/FC?/FS?C/FC?C + IND).
///
/// Cross-reference: the TS `NON_TAPPABLE` set in `hp41-gui/src/help_data.ts`
/// enumerates the SAME op_variants. Count must stay 61 in both.
const NON_TAPPABLE: &[&str] = &[
    // ── Parameterized ops (require an argument — not XEQ-by-name runnable) ──
    "StoReg", "RclReg", "StoArith", "StoArithStack",
    "StoM", "StoN", "StoO", "RclM", "RclN", "RclO",
    "StoInd", "RclInd", "StoArithInd",
    "FmtFix", "FmtSci", "FmtEng",
    "SfFlag", "CfFlag", "SfFlagInd", "CfFlagInd",
    "FlagTest", "FlagTestInd",
    "View", "ViewInd",
    "Tone",
    "Isg", "Dse", "IsgInd", "DseInd",
    "Arcl", "ArclInd", "Asto", "AstoInd",
    "Gto", "GtoInd",
    "Xeq", "XeqInd",
    "Lbl",
    "Clp", "Del",
    "Asn",
    "Test",
    // ── Immediate stack / entry keys (raw arithmetic / stack ops) ──
    "Add", "Sub", "Mul", "Div",
    "Enter", "Clx", "Chs", "Rdn", "XySwap", "Lastx",
    "PctChange",
    // ── Mode / composite-placeholder rows ──
    "AlphaToggle", "AlphaAppend", "AlphaBackspace",
    "PrgmMode",
    "UserMode",
    "Catalog",
    "GetKey",
    "Null",
];

/// Default XROM module mask: all 5 pacs active (Math 1 bit-0, Stat 1 bit-1,
/// Time bit-2, Adv 22A bit-3, Adv 24B bit-4). Mirrors `default_xrom_modules()`.
const DEFAULT_XROM_MODULES: u8 = 0b0001_1111;

#[test]
fn test_non_tappable_count_pinned_at_61() {
    // Count-pin: assert the NON_TAPPABLE set size is exactly 61.
    // This ensures that neither the TS set (hp41-gui/src/help_data.ts) nor this
    // Rust set can drift without a deliberate update to both sides.
    // Cross-reference TS: `const NON_TAPPABLE: ReadonlySet<string>` — same 61 entries.
    let unique: HashSet<&str> = NON_TAPPABLE.iter().copied().collect();
    assert_eq!(
        unique.len(),
        61,
        "NON_TAPPABLE must have exactly 61 unique entries (TS↔Rust parity); found {}",
        unique.len()
    );
    assert_eq!(
        NON_TAPPABLE.len(),
        61,
        "NON_TAPPABLE slice must have exactly 61 entries; found {} (check for duplicates)",
        NON_TAPPABLE.len()
    );
}

#[test]
fn test_every_tappable_display_name_resolves() {
    // For every `status == "implemented"` JSON entry outside the NON_TAPPABLE set,
    // assert its display_name resolves via builtin_card_op OR xrom_resolve(default mask).
    //
    // This catches the class of regressions where a JSON display_name is edited to
    // differ from the exact mnemonic registered in the resolver — the overlay would
    // show the entry as tappable but the dispatch would silently fail (D-07).
    let non_tappable_set: HashSet<&str> = NON_TAPPABLE.iter().copied().collect();

    // Collect all 6 pools.
    let all_entries: Vec<&hp41_cli::help_data::HelpEntry> = help_entries()
        .iter()
        .chain(help_entries_math1().iter())
        .chain(help_entries_stat1().iter())
        .chain(help_entries_time().iter())
        .chain(help_entries_adv().iter())
        .chain(help_entries_xmem().iter())
        .collect();

    let mut failures: Vec<String> = Vec::new();

    for entry in all_entries {
        // Only check implemented entries — deferred/na are not shown.
        if entry.status != "implemented" {
            continue;
        }

        // Skip NON_TAPPABLE entries — they are not dispatched.
        if non_tappable_set.contains(entry.op_variant.as_str()) {
            continue;
        }

        // For every remaining (tappable) entry, the display_name must resolve.
        let resolves_builtin =
            hp41_core::ops::program::builtin_card_op(entry.display_name.as_str()).is_some();
        let resolves_xrom = hp41_core::ops::math1::xrom::xrom_resolve(
            entry.display_name.as_str(),
            DEFAULT_XROM_MODULES,
        )
        .is_some();

        if !resolves_builtin && !resolves_xrom {
            failures.push(format!(
                "  op_variant='{}' display_name='{}' — NOT resolved by builtin_card_op or xrom_resolve",
                entry.op_variant, entry.display_name
            ));
        }
    }

    assert!(
        failures.is_empty(),
        concat!(
            "Tappable entries whose display_name is NOT resolved by builtin_card_op/xrom_resolve",
            " (default module mask 0b0001_1111):\n{}",
            "\n\nFix: ensure display_name in the relevant docs/hp41-*-functions.json matches",
            " the exact mnemonic registered in the resolver (case-sensitive). If the entry",
            " is not XEQ-by-name runnable, add its op_variant to NON_TAPPABLE in BOTH",
            " hp41-gui/src/help_data.ts AND hp41-cli/tests/tappability_parity.rs."
        ),
        failures.join("\n")
    );
}
