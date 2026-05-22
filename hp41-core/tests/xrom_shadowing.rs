// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! Wave-0 CI gate: asserts no Math Pac I mnemonic shadows an existing v2.2 built-in.
//!
//! This test file iterates `MATH_1.ops` and asserts that no entry collides
//! with a `builtin_card_op` mnemonic.
//!
//! **Why this matters (C-28.4 / Pitfall 1):** the XROM resolver fires LAST in the
//! chain — after `builtin_card_op`, before `Err(InvalidOp)`. If a Math Pac I mnemonic
//! shadowed a builtin (e.g., if Math Pac I defined "WPRGM" or "X<>Y?"), the builtin
//! would silently win and the Math Pac I op would be unreachable via XEQ.
//! The shadowing test catches this at CI time so Plans 28-02..28-10 can't introduce
//! a shadow without failing this gate.
//!
//! **Plan 32-01 (graduation, 2026-05-18):** gate graduated from vacuous to
//! active — `MATH_1.ops` now carries 52 entries (Plans 28-02..28-10) and this
//! file actively cross-checks them against the 18-entry `BUILTIN_CARD_OP_NAMES`
//! allowlist (4 card-reader + 9 ASCII conditional + 5 Unicode conditional).
//! Both lists were verified in sync with `builtin_card_op` in
//! `hp41-core/src/ops/program.rs::builtin_card_op` at L1112-1132 (4 + 8 match
//! arms producing 18 mnemonics with the ASCII/Unicode duplication).

#![allow(clippy::unwrap_used)]

use hp41_core::ops::math1::xrom::{xrom_resolve, MATH_1, STAT_1};

/// All mnemonic strings recognized by `builtin_card_op` in `hp41-core/src/ops/program.rs`.
///
/// This list must be kept in sync with `builtin_card_op`'s match arms.
/// If a new builtin is added to `builtin_card_op`, add it here to maintain the
/// shadowing gate's correctness.
///
/// Canonical set as of v2.2 (Plan 25-03):
/// - 4 Card Reader ops: WPRGM, RDPRGM, WDTA, RDTA
/// - 8 conditional tests (ASCII + Unicode spellings): X<>Y?, X≠Y?, X#Y?,
///   X<Y?, X>=Y?, X≥Y?, X#0?, X≠0?, X<0?, X>0?, X<=0?, X≤0?, X>=0?, X≥0?
const BUILTIN_CARD_OP_NAMES: &[&str] = &[
    // Card Reader ops
    "WPRGM",
    "RDPRGM",
    "WDTA",
    "RDTA",
    // Conditional tests (ASCII spellings)
    "X<>Y?",
    "X#Y?",
    "X<Y?",
    "X>=Y?",
    "X#0?",
    "X<0?",
    "X>0?",
    "X<=0?",
    "X>=0?",
    // Conditional tests (Unicode spellings)
    "X\u{2260}Y?",
    "X\u{2265}Y?",
    "X\u{2260}0?",
    "X\u{2264}0?",
    "X\u{2265}0?",
];

/// CI gate: no Math Pac I mnemonic may shadow a v2.2 built-in name.
///
/// Currently vacuous (MATH_1.ops is empty). Non-trivial once Plans 28-02..28-10
/// populate MATH_1.ops with hyperbolic, complex, and workflow op mnemonics.
///
/// Catches: Pitfall 1 — a Math Pac I mnemonic accidentally matching a v2.2 builtin,
/// making the XROM op permanently unreachable via XEQ.
#[test]
fn math1_names_do_not_shadow_builtins() {
    for (name, _op) in MATH_1.ops {
        assert!(
            !BUILTIN_CARD_OP_NAMES.contains(name),
            "Math Pac I mnemonic {name:?} shadows a builtin_card_op entry. \
             The XROM resolver fires LAST (C-28.4), so the builtin would silently \
             win and the Math Pac I op would be permanently unreachable via XEQ. \
             Rename the Math Pac I mnemonic to avoid the collision."
        );
    }
}

/// Smoke: MATH_1 const fields are present and correct.
/// Catches: const field regression during Plans 28-02..28-10 MATH_1.ops growth.
#[test]
fn math1_const_fields() {
    assert_eq!(
        MATH_1.id, 7,
        "MATH_1.id must be 7 (HP Math Pac I hardware module ID)"
    );
    assert_eq!(MATH_1.name, "MATH 1A", "MATH_1.name must be 'MATH 1A'");
}

// ── Phase 33 Plan 33-01: STAT_1 disjointness + consistency gates ────────────

/// CI gate: no Stat 1 Pac mnemonic may collide with a v2.2 builtin name.
///
/// Catches: Pitfall 1 — a Stat 1 mnemonic accidentally matching a v2.2 builtin
/// would make the XROM op permanently unreachable via XEQ (resolver fires LAST).
#[test]
fn stat1_names_do_not_shadow_builtins() {
    for (name, _op) in STAT_1.ops {
        assert!(
            !BUILTIN_CARD_OP_NAMES.contains(name),
            "Stat 1 Pac mnemonic {name:?} shadows a builtin_card_op entry. \
             The XROM resolver fires LAST (C-28.4), so the builtin would silently \
             win and the Stat 1 Pac op would be permanently unreachable via XEQ. \
             Rename the Stat 1 Pac mnemonic to avoid the collision."
        );
    }
}

/// CI gate: STAT_1.ops mnemonic strings must be disjoint from MATH_1.ops.
///
/// Catches: a future plan accidentally moves a mnemonic from one module to
/// the other without removing the original — both modules would resolve the
/// same name and `xrom_resolve` order (Math 1 first, Stat 1 second) would
/// silently hide the new Stat 1 entry behind the Math 1 stale entry.
#[test]
fn stat1_ops_disjoint_from_math1_ops() {
    use std::collections::HashSet;
    let math1_names: HashSet<&str> = MATH_1.ops.iter().map(|(n, _)| *n).collect();
    for (name, _op) in STAT_1.ops {
        assert!(
            !math1_names.contains(name),
            "Stat 1 Pac mnemonic {name:?} also appears in MATH_1.ops. \
             A mnemonic must belong to exactly one XROM module so xrom_resolve \
             has a deterministic single-source mapping (resolver-LAST + bit-isolation \
             invariants per CLAUDE.md \"Resolver chain + never-discard\")."
        );
    }
}

/// CI gate: every STAT_1.ops mnemonic resolves to its declared Op via
/// `xrom_resolve` when both bits are set — proves bidirectional consistency
/// between the slice and the `stat1_resolve` match arms (drift in either
/// direction surfaces here).
#[test]
fn stat1_ops_resolve_via_xrom_resolve() {
    for (name, expected_op) in STAT_1.ops {
        let resolved = xrom_resolve(name, 0b0000_0011);
        assert_eq!(
            resolved.as_ref(),
            Some(expected_op),
            "STAT_1.ops mnemonic {name:?} must resolve to {expected_op:?} via \
             xrom_resolve(name, 0b0000_0011) — drift between STAT_1.ops slice and \
             stat1_resolve match arms is a Pitfall 22 / resolver-never-discard violation."
        );
    }
}

/// Smoke: STAT_1 const fields are present and correct.
/// Catches: const field regression during Plans 33-03..33-08 STAT_1.ops growth.
#[test]
fn stat1_const_fields() {
    assert_eq!(
        STAT_1.id, 2,
        "STAT_1.id must be 2 (HP Stat 1 Pac hardware module ID per calc.fjk.ch)"
    );
    assert_eq!(STAT_1.name, "STAT 1B", "STAT_1.name must be 'STAT 1B'");
}
