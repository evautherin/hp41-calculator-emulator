// Algorithm independently re-derived from HP Math Pac I Owner's Manual 00041-90034 (1979);
// Free42 source consulted only as sanity-check oracle, not copied.
//
//! XROM resolver framework.
//!
//! `MATH_1` is the public registry for Math Pac I (module id 7, "MATH 1A").
//! Downstream plans (28-02..28-10) extend `math1_resolve()` as new `Op` variants are added.
//!
//! Resolver chain contract (C-28.4 / Pitfall 1):
//! `xrom_resolve` fires LAST — after `builtin_card_op`, before `Err(InvalidOp)`.
//! `tests/xrom_shadowing.rs` CI-gates this invariant on every `MATH_1.ops` entry.

use crate::ops::Op;

/// An XROM application-module descriptor.
///
/// `id` matches the HP-41C hardware XROM module ID (7 for Math Pac I).
/// `name` is the string the CATALOG 2 listing would display.
/// `ops` is the canonical mnemonic → Op mapping used both by `xrom_resolve`
/// and by `tests/xrom_shadowing.rs` to assert non-collision with v2.2 builtins.
/// Plans 28-02..28-10 append entries here via `MATH_1_OPS` once each `Op` variant
/// exists. Currently `&[]` (empty) because no Math Pac I `Op` variants exist yet.
pub struct XromModule {
    pub id: u8,
    pub name: &'static str,
    pub ops: &'static [(&'static str, Op)],
}

/// Math Pac I module registry.
///
/// - `id = 7` — real HP-41C hardware Math Pac I XROM module ID.
/// - `name = "MATH 1A"` — as displayed by CATALOG 2 on real hardware.
/// - `ops` — mnemonic → Op mapping; grows with each Plan 28-02..28-10.
///
/// Plan 28-02: 6 hyperbolic entries added (SINH, COSH, TANH, ASINH, ACOSH, ATANH).
/// Plan 28-03: 5 complex arithmetic entries added (C+, C-, C×, C÷, REAL).
///             ASCII aliases C* and C/ included for C× and C÷ respectively.
/// Plan 28-04: 12 complex function entries (MAGZ, CINV, Z↑N, Z↑1/N, E↑Z, LNZ,
///             SINZ, COSZ, TANZ, A↑Z, LOGZ, Z↑W) with ASCII aliases for Unicode ops.
/// Plan 28-05: 2 POLY/ROOTS entries (POLY modal opener, ROOTS executor).
/// Plan 28-06: 8 MATRIX entries (MATRIX, SIZE, VMAT, EDIT, DET, INV, SIMEQ, VCOL).
///             "INV" confirmed non-shadowing: builtin_card_op registers no "INV" string.
pub const MATH_1: XromModule = XromModule {
    id: 7,
    name: "MATH 1A",
    ops: &[
        // ── Plan 28-02: Hyperbolics ────────────────────────────────────────────
        ("SINH", Op::Sinh),
        ("COSH", Op::Cosh),
        ("TANH", Op::Tanh),
        ("ASINH", Op::Asinh),
        ("ACOSH", Op::Acosh),
        ("ATANH", Op::Atanh),
        // ── Plan 28-03: Complex Stack Arithmetic ──────────────────────────────
        ("C+", Op::CPlus),
        ("C-", Op::CMinus),
        ("C\u{00D7}", Op::CTimes), // Unicode alias (primary)
        ("C*", Op::CTimes),        // ASCII alias for C×
        ("C\u{00F7}", Op::CDiv),   // Unicode alias (primary)
        ("C/", Op::CDiv),          // ASCII alias for C÷
        ("REAL", Op::Real),
        // ── Plan 28-04: Complex Functions ─────────────────────────────────────
        ("MAGZ", Op::Magz),
        ("CINV", Op::Cinv),
        ("Z\u{2191}N", Op::ZpowN),    // Unicode ↑ (primary)
        ("Z^N", Op::ZpowN),           // ASCII alias for Z↑N
        ("Z\u{2191}1/N", Op::Zpow1N), // Unicode ↑ (primary)
        ("Z^1/N", Op::Zpow1N),        // ASCII alias for Z↑1/N
        ("E\u{2191}Z", Op::ExpZ),     // Unicode ↑ (primary)
        ("E^Z", Op::ExpZ),            // ASCII alias for E↑Z
        ("LNZ", Op::LnZ),
        ("SINZ", Op::SinZ),
        ("COSZ", Op::CosZ),
        ("TANZ", Op::TanZ),
        ("A\u{2191}Z", Op::ApowZ), // Unicode ↑ (primary)
        ("A^Z", Op::ApowZ),        // ASCII alias for A↑Z
        ("LOGZ", Op::LogZ),
        ("Z\u{2191}W", Op::ZpowW), // Unicode ↑ (primary)
        ("Z^W", Op::ZpowW),        // ASCII alias for Z↑W
        // ── Plan 28-05: POLY / ROOTS ──────────────────────────────────────────
        ("POLY", Op::PolyWorkflow),
        ("ROOTS", Op::Roots),
        // ── Plan 28-06: MATRIX ────────────────────────────────────────────────
        ("MATRIX", Op::MatrixWorkflow),
        ("SIZE", Op::MatSize),
        ("VMAT", Op::MatVmat),
        ("EDIT", Op::MatEdit),
        ("DET", Op::MatDet),
        // "INV" non-shadowing: v2.2 Op::Inv (reciprocal) uses the "1/x" display
        // mnemonic in builtin_card_op, not "INV" — claiming "INV" for MATRIX is safe.
        // Confirmed: RESEARCH §"Resolver-Chain Conflict Map" lines 636-638.
        ("INV", Op::MatInv),
        ("SIMEQ", Op::MatSimeq),
        ("VCOL", Op::MatVcol),
        // ── Plan 28-07: INTG ──────────────────────────────────────────────────
        ("INTG", Op::Integ),
        // ── Plan 28-08: SOLVE / SOL ───────────────────────────────────────────
        ("SOLVE", Op::Solve),
        ("SOL", Op::Sol),
        // ── Plan 28-09: DIFEQ ─────────────────────────────────────────────────
        // "DIFEQ" non-shadowing: RESEARCH §"Resolver-Chain Conflict Map" line 628
        // confirms "DIFEQ" appears in neither v2.2 xeq_by_name_local_resolve nor
        // builtin_card_op. Safe to claim for Math Pac I.
        ("DIFEQ", Op::Difeq),
        // ── Plan 28-10: FOUR / Triangle Solvers / TRANS ───────────────────────
        // All 8 mnemonics confirmed non-shadowing per RESEARCH §"Resolver-Chain
        // Conflict Map" lines 619-633 (no v2.2 builtin uses SSS/ASA/SAA/SAS/SSA/FOUR/TRANS/T3D).
        // "T3D" chosen for Op::Trans3d to disambiguate from 2D TRANS (Plan 28-10 decision).
        ("FOUR", Op::Four),
        ("SSS", Op::TriSss),
        ("ASA", Op::TriAsa),
        ("SAA", Op::TriSaa),
        ("SAS", Op::TriSas),
        ("SSA", Op::TriSsa),
        ("TRANS", Op::Trans2d),
        ("T3D", Op::Trans3d),
    ],
};

/// Stat 1 Pac module registry (D-33.3 freeze exception — second Stat-related
/// entry in this otherwise-frozen file, alongside the bit-1 arm in
/// `xrom_resolve` below).
///
/// - `id = 2` — HP hardware Statistics Pac 1 XROM module ID per
///   `calc.fjk.ch/db/hp41mod.php` ("Statistics Pac 1B" XROM #2).
/// - `name = "STAT 1B"` — CATALOG 2 display string per
///   HP Stat 1 Pac Owner's Manual 00041-90030 (1979).
/// - `ops` — locked from SPEC.md §"Stat 1 Pac Mnemonics" (14 entry points
///   from QRC 00041-90061 + 12 secondary entry points enumerated by the
///   QRC's per-program rows + RAND/SEED emulator extensions per D-33.4).
///   26 total entries. Unicode Σ encoded as `\u{03A3}` per Plan-28
///   convention (`\u{00D7}` × in MATH_1.ops at line 57).
///
/// **Plan 33-01 scaffolding:** every entry currently maps to `Op::Stat1Stub`
/// (a single placeholder variant in `crate::ops::Op`) because the real
/// `Op::Sigma*` variants land incrementally in Plans 33-03..33-08. The
/// `xrom_shadowing.rs` CI gate already cross-checks the slice against
/// `MATH_1.ops` and `builtin_card_op` for disjointness; the per-mnemonic
/// dispatch becomes meaningful once Plans 33-03+ replace `Op::Stat1Stub`
/// references with real variants.
pub const STAT_1: XromModule = XromModule {
    id: 2,
    name: "STAT 1B",
    ops: &[
        // ── Stat 1 Pac Univariate / Bivariate Summaries ────────────────────────
        // Plan 33-05: ΣBSTAT + ΣBSTG → real Sigma* variants.
        ("\u{03A3}BSTAT", Op::SigmaBstat),  // ΣBSTAT — Plan 33-05
        ("\u{03A3}BSTG", Op::SigmaBstg),    // ΣBSTG  — Plan 33-05
        ("\u{03A3}MMTUG", Op::Stat1Stub),  // ΣMMTUG — Plan 33-06
        ("\u{03A3}MMTGD", Op::Stat1Stub),  // ΣMMTGD — Plan 33-06
        // ── Stat 1 Pac ANOVA Family ────────────────────────────────────────────
        ("\u{03A3}AOVONE", Op::Stat1Stub), // ΣAOVONE — Plan 33-06
        ("\u{03A3}AOVTWO", Op::Stat1Stub), // ΣAOVTWO — Plan 33-06
        ("\u{03A3}ANOCOV", Op::Stat1Stub), // ΣANOCOV — Plan 33-06
        // ── Stat 1 Pac Curve Fitting + Regression ─────────────────────────────
        // Plan 33-05: ΣLIN / ΣEXP / ΣLOGI / ΣPOW → real Sigma* variants via
        // log-linearization + op_sigma_plus delegate (anti-duplication).
        ("\u{03A3}LIN", Op::SigmaLin),    // ΣLIN    — Plan 33-05
        ("\u{03A3}EXP", Op::SigmaExp),    // ΣEXP    — Plan 33-05
        ("\u{03A3}LOGI", Op::SigmaLogi),  // ΣLOGI   — Plan 33-05
        ("\u{03A3}POW", Op::SigmaPow),    // ΣPOW    — Plan 33-05
        ("\u{03A3}MLRXY", Op::Stat1Stub),  // ΣMLRXY  — Plan 33-08
        ("\u{03A3}MLRXYZ", Op::Stat1Stub), // ΣMLRXYZ — Plan 33-08
        ("\u{03A3}POLYP", Op::Stat1Stub),  // ΣPOLYP  — Plan 33-08
        ("\u{03A3}POLYC", Op::Stat1Stub),  // ΣPOLYC  — Plan 33-08
        // ── Stat 1 Pac Hypothesis Tests ────────────────────────────────────────
        ("\u{03A3}PTST", Op::Stat1Stub),   // ΣPTST   — Plan 33-07
        ("\u{03A3}TSTAT", Op::Stat1Stub),  // ΣTSTAT  — Plan 33-07
        // ── Stat 1 Pac Nonparametric / Chi-Square Evaluation / Contingency ────
        // Plan 33-04 swapped ΣSPEAR (Task 1) + ΣXSQEV (Task 2) + ΣEFXSQ
        // (Task 3) from Op::Stat1Stub to their real Sigma* variants.
        // 5 of 26 Op::Stat1Stub references now swapped (3 from this plan;
        // remaining 21 are owned by Plans 33-03 / 33-05 / 33-06 / 33-07 / 33-08).
        ("\u{03A3}XSQEV", Op::SigmaXsqev), // ΣXSQEV  — Plan 33-04
        ("\u{03A3}EFXSQ", Op::SigmaEfxsq), // ΣEFXSQ  — Plan 33-04
        ("\u{03A3}CTKKK", Op::Stat1Stub),  // ΣCTKKK  — Plan 33-06
        ("\u{03A3}CTKK", Op::Stat1Stub),   // ΣCTKK   — Plan 33-06
        ("\u{03A3}SPEAR", Op::SigmaSpear), // ΣSPEAR  — Plan 33-04
        // ── Stat 1 Pac Distributions ───────────────────────────────────────────
        // Plan 33-03: ΣNORMD + ΣCHISQD → real Sigma* variants.
        ("\u{03A3}NORMD", Op::SigmaNormdWorkflow),   // ΣNORMD  — Plan 33-03
        ("\u{03A3}CHISQD", Op::SigmaChisqdWorkflow), // ΣCHISQD — Plan 33-03
        // ── Stat 1 Pac RAND/SEED (emulator extension per D-33.4) ──────────────
        ("RAND", Op::Stat1Stub),           // RAND    — Plan 33-08
        ("SEED", Op::Stat1Stub),           // SEED    — Plan 33-08
    ],
};

/// Resolve an XEQ-by-name label against loaded XROM modules.
///
/// Returns `Some(Op)` if `name` matches a Math Pac I mnemonic AND bit 0 of
/// `modules` is set (Math 1 loaded). Returns `None` otherwise.
///
/// LAST-fires invariant: called AFTER `builtin_card_op`, BEFORE `Err(InvalidOp)`
/// at both insertion sites in `hp41-core/src/ops/program.rs` (C-28.4).
pub fn xrom_resolve(name: &str, modules: u8) -> Option<Op> {
    if modules & 0b0000_0001 != 0 {
        if let Some(op) = math1_resolve(name) {
            return Some(op);
        }
    }
    // Phase 33 (v3.1): Stat 1 Pac bit-1 arm activated per D-33.3 freeze
    // exception. Order matters — Stat 1 fires AFTER Math 1 so a future
    // Plan that wants Math 1 to "win" over a stat1 alias just needs to
    // keep its Math 1 entry intact (current `MATH_1.ops` x `STAT_1.ops`
    // disjointness is CI-gated in `tests/xrom_shadowing.rs`).
    if modules & 0b0000_0010 != 0 {
        if let Some(op) = stat1_resolve(name) {
            return Some(op);
        }
    }
    None
}

/// Math Pac I (bit 0) mnemonic resolver.
///
/// Currently returns `None` for all names (no Math Pac I `Op` variants exist yet).
/// Plans 28-02..28-10 extend this match block as new `Op` variants are added:
///
/// ```text
/// // Plan 28-02:
/// "SINH" => Some(Op::Sinh),
/// "COSH" => Some(Op::Cosh),
/// "TANH" => Some(Op::Tanh),
/// "ASINH" => Some(Op::Asinh),
/// "ACOSH" => Some(Op::Acosh),
/// "ATANH" => Some(Op::Atanh),
/// // Plan 28-03/04 adds complex ops ...
/// // Plan 28-05 adds POLY ...
/// // Plan 28-06 adds MATRIX / SIMEQ / DET / TRANS3D / DOT / CROSS ...
/// // Plan 28-07 adds INTG ...
/// // Plan 28-08 adds SOLVE ...
/// // Plan 28-09 adds DIFEQ ...
/// // Plan 28-10 adds FOUR / SSS / SAS / ASA / SSA / AAS / TRANS ...
/// ```
fn math1_resolve(name: &str) -> Option<Op> {
    match name {
        // ── Plan 28-02: Hyperbolics ────────────────────────────────────────────
        "SINH" => Some(Op::Sinh),
        "COSH" => Some(Op::Cosh),
        "TANH" => Some(Op::Tanh),
        "ASINH" => Some(Op::Asinh),
        "ACOSH" => Some(Op::Acosh),
        "ATANH" => Some(Op::Atanh),
        // ── Plan 28-03: Complex Stack Arithmetic ──────────────────────────────
        "C+" => Some(Op::CPlus),
        "C-" => Some(Op::CMinus),
        "C\u{00D7}" | "C*" => Some(Op::CTimes), // Unicode × and ASCII * both accepted
        "C\u{00F7}" | "C/" => Some(Op::CDiv),   // Unicode ÷ and ASCII / both accepted
        "REAL" => Some(Op::Real),
        // ── Plan 28-04: Complex Functions ─────────────────────────────────────
        "MAGZ" => Some(Op::Magz),
        "CINV" => Some(Op::Cinv),
        "Z\u{2191}N" | "Z^N" => Some(Op::ZpowN), // Unicode ↑ and ASCII ^ both accepted
        "Z\u{2191}1/N" | "Z^1/N" => Some(Op::Zpow1N),
        "E\u{2191}Z" | "E^Z" => Some(Op::ExpZ), // Unicode ↑ and ASCII ^ both accepted
        "LNZ" => Some(Op::LnZ),
        "SINZ" => Some(Op::SinZ),
        "COSZ" => Some(Op::CosZ),
        "TANZ" => Some(Op::TanZ),
        "A\u{2191}Z" | "A^Z" => Some(Op::ApowZ),
        "LOGZ" => Some(Op::LogZ),
        "Z\u{2191}W" | "Z^W" => Some(Op::ZpowW),
        // ── Plan 28-05: POLY / ROOTS ──────────────────────────────────────────
        "POLY" => Some(Op::PolyWorkflow),
        "ROOTS" => Some(Op::Roots),
        // ── Plan 28-06: MATRIX ────────────────────────────────────────────────
        "MATRIX" => Some(Op::MatrixWorkflow),
        "SIZE" => Some(Op::MatSize),
        "VMAT" => Some(Op::MatVmat),
        "EDIT" => Some(Op::MatEdit),
        "DET" => Some(Op::MatDet),
        // "INV" is non-shadowing: builtin_card_op does not register "INV"
        // (Op::Inv reciprocal uses "1/x" display name, not "INV" string).
        // Confirmed via RESEARCH §"Resolver-Chain Conflict Map".
        "INV" => Some(Op::MatInv),
        "SIMEQ" => Some(Op::MatSimeq),
        "VCOL" => Some(Op::MatVcol),
        // ── Plan 28-07: INTG ──────────────────────────────────────────────────
        "INTG" => Some(Op::Integ),
        // ── Plan 28-08: SOLVE / SOL ───────────────────────────────────────────
        // "SOLVE" non-shadowing: RESEARCH §"Resolver-Chain Conflict Map" line 628
        // confirms neither "SOLVE" nor "SOL" appears in v2.2 xeq_by_name_local_resolve
        // or builtin_card_op. Safe to claim both for Math Pac I.
        "SOLVE" => Some(Op::Solve),
        "SOL" => Some(Op::Sol),
        // ── Plan 28-09: DIFEQ ─────────────────────────────────────────────────
        // "DIFEQ" non-shadowing: confirmed safe per RESEARCH §"Resolver-Chain Conflict Map".
        "DIFEQ" => Some(Op::Difeq),
        // ── Plan 28-10: FOUR / Triangle Solvers / TRANS ───────────────────────
        // All 8 mnemonics confirmed non-shadowing per RESEARCH §"Resolver-Chain
        // Conflict Map" lines 619-633. "T3D" disambiguates from 2D "TRANS" (Plan 28-10).
        "FOUR" => Some(Op::Four),
        "SSS" => Some(Op::TriSss),
        "ASA" => Some(Op::TriAsa),
        "SAA" => Some(Op::TriSaa),
        "SAS" => Some(Op::TriSas),
        "SSA" => Some(Op::TriSsa),
        "TRANS" => Some(Op::Trans2d),
        "T3D" => Some(Op::Trans3d),
        _ => None,
    }
}

/// Stat 1 Pac (bit 1) mnemonic resolver — D-33.3 freeze exception.
///
/// Every arm currently maps to `Op::Stat1Stub` (Plan 33-01 scaffolding);
/// Plans 33-03..33-08 replace each Σ-prefixed mnemonic with the real
/// `Op::Sigma*` variant and DELETE the `Op::Stat1Stub` variant by end
/// of Phase 33.
///
/// Bidirectional consistency with `STAT_1.ops`: the
/// `stat1_ops_mnemonics_resolve_consistently` test below iterates the
/// slice and asserts every mnemonic round-trips through this match.
fn stat1_resolve(name: &str) -> Option<Op> {
    match name {
        // Univariate / Bivariate Summaries
        // Plan 33-05: ΣBSTAT + ΣBSTG → real Sigma* variants.
        "\u{03A3}BSTAT" => Some(Op::SigmaBstat),
        "\u{03A3}BSTG" => Some(Op::SigmaBstg),
        "\u{03A3}MMTUG" => Some(Op::Stat1Stub),
        "\u{03A3}MMTGD" => Some(Op::Stat1Stub),
        // ANOVA Family
        "\u{03A3}AOVONE" => Some(Op::Stat1Stub),
        "\u{03A3}AOVTWO" => Some(Op::Stat1Stub),
        "\u{03A3}ANOCOV" => Some(Op::Stat1Stub),
        // Curve Fitting + Regression
        // Plan 33-05: ΣLIN / ΣEXP / ΣLOGI / ΣPOW → real Sigma* variants.
        "\u{03A3}LIN" => Some(Op::SigmaLin),
        "\u{03A3}EXP" => Some(Op::SigmaExp),
        "\u{03A3}LOGI" => Some(Op::SigmaLogi),
        "\u{03A3}POW" => Some(Op::SigmaPow),
        "\u{03A3}MLRXY" => Some(Op::Stat1Stub),
        "\u{03A3}MLRXYZ" => Some(Op::Stat1Stub),
        "\u{03A3}POLYP" => Some(Op::Stat1Stub),
        "\u{03A3}POLYC" => Some(Op::Stat1Stub),
        // Hypothesis Tests
        "\u{03A3}PTST" => Some(Op::Stat1Stub),
        "\u{03A3}TSTAT" => Some(Op::Stat1Stub),
        // Nonparametric / Chi-Square Evaluation / Contingency
        // Plan 33-04: ΣSPEAR + ΣXSQEV + ΣEFXSQ → real Sigma* variants.
        "\u{03A3}XSQEV" => Some(Op::SigmaXsqev),
        "\u{03A3}EFXSQ" => Some(Op::SigmaEfxsq),
        "\u{03A3}CTKKK" => Some(Op::Stat1Stub),
        "\u{03A3}CTKK" => Some(Op::Stat1Stub),
        "\u{03A3}SPEAR" => Some(Op::SigmaSpear),
        // Distributions
        // Plan 33-03: ΣNORMD + ΣCHISQD → real Sigma* variants.
        "\u{03A3}NORMD" => Some(Op::SigmaNormdWorkflow),
        "\u{03A3}CHISQD" => Some(Op::SigmaChisqdWorkflow),
        // RAND / SEED (emulator extension per D-33.4)
        "RAND" => Some(Op::Stat1Stub),
        "SEED" => Some(Op::Stat1Stub),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{xrom_resolve, MATH_1, STAT_1};
    use crate::ops::Op;

    const NONEXISTENT_NAME: &str = "__MATH1_PROBE_NONEXISTENT__";

    // Catches: bit-mask off-by-one (module loaded bit check)
    #[test]
    fn resolve_returns_none_for_unknown_name_with_module_loaded() {
        // MATH_1 is loaded (bit 0 set), but name is unknown
        let result = xrom_resolve(NONEXISTENT_NAME, 0b0000_0001);
        assert!(
            result.is_none(),
            "xrom_resolve should return None for unknown name even when module is loaded"
        );
    }

    // Catches: bit-mask off-by-one or module-not-loaded path skipped
    #[test]
    fn resolve_returns_none_when_module_not_loaded() {
        // Bit 0 is CLEAR — Math 1 not loaded
        let result = xrom_resolve(NONEXISTENT_NAME, 0b0000_0000);
        assert!(
            result.is_none(),
            "xrom_resolve should return None when Math 1 module is not loaded (bit 0 clear)"
        );
    }

    // Catches: module-bit isolation — bit 0 only, not any bit
    #[test]
    fn resolve_uses_bit_0_only_for_math1() {
        // Bit 1 set, bit 0 clear — Math 1 is NOT loaded
        let result = xrom_resolve("SINH", 0b0000_0010);
        assert!(
            result.is_none(),
            "xrom_resolve must check bit 0 specifically, not any set bit"
        );
    }

    // Catches: MATH_1 const field regression
    #[test]
    fn math1_const_id_and_name() {
        assert_eq!(
            MATH_1.id, 7,
            "MATH_1.id must be 7 (HP Math Pac I hardware module ID)"
        );
        assert_eq!(
            MATH_1.name, "MATH 1A",
            "MATH_1.name must be 'MATH 1A' (HP-41C CATALOG 2 display string)"
        );
    }

    // ── Plan 28-02: Positive resolution tests (hyperbolic mnemonics) ─────────

    // Catches: math1_resolve not recognizing SINH when module is loaded
    #[test]
    fn resolve_sinh_with_module_loaded() {
        let result = xrom_resolve("SINH", 0b0000_0001);
        assert_eq!(
            result,
            Some(Op::Sinh),
            "xrom_resolve('SINH', bit0=1) must return Some(Op::Sinh)"
        );
    }

    // Catches: module-not-loaded path not short-circuiting before math1_resolve
    #[test]
    fn resolve_sinh_module_not_loaded_returns_none() {
        let result = xrom_resolve("SINH", 0b0000_0000);
        assert!(
            result.is_none(),
            "xrom_resolve('SINH', bit0=0) must return None (module not loaded)"
        );
    }

    // Catches: missing ASINH in math1_resolve match block
    #[test]
    fn resolve_asinh_with_module_loaded() {
        let result = xrom_resolve("ASINH", 0b0000_0001);
        assert_eq!(
            result,
            Some(Op::Asinh),
            "xrom_resolve('ASINH', bit0=1) must return Some(Op::Asinh)"
        );
    }

    // Catches: MATH_1.ops slice not populated with correct count
    // Plan 28-02: 6 hyperbolic entries; Plan 28-03: +7 complex arith entries (C+, C-, C×, C*, C÷, C/, REAL)
    // Plan 28-04: +17 complex function entries (MAGZ, CINV, Z↑N, Z^N, Z↑1/N, Z^1/N, E↑Z, E^Z,
    //             LNZ, SINZ, COSZ, TANZ, A↑Z, A^Z, LOGZ, Z↑W, Z^W)
    // Plan 28-05: +2 POLY/ROOTS entries
    // Plan 28-06: +8 MATRIX entries (MATRIX, SIZE, VMAT, EDIT, DET, INV, SIMEQ, VCOL)
    // Plan 28-07: +1 INTG entry
    // Plan 28-08: +2 SOLVE/SOL entries
    // Plan 28-09: +1 DIFEQ entry
    // Plan 28-10: +8 FOUR/SSS/ASA/SAA/SAS/SSA/TRANS/T3D entries
    // Total: 6+7+17+2+8+1+2+1+8 = 52 entries
    #[test]
    fn math1_ops_has_correct_entry_count() {
        assert_eq!(
            MATH_1.ops.len(),
            52,
            "MATH_1.ops must have exactly 52 entries after Plan 28-09 (6 hyp + 7 complex-arith + 17 complex-fn + 2 poly + 8 matrix + 1 intg + 2 solve + 1 difeq + 8 four/tri/trans)"
        );
    }

    // Catches: MATH_1.ops mnemonic strings not matching math1_resolve keys
    #[test]
    fn math1_ops_mnemonics_resolve_consistently() {
        // Every mnemonic in MATH_1.ops must resolve to the same Op via xrom_resolve
        for (name, expected_op) in MATH_1.ops {
            let resolved = xrom_resolve(name, 0b0000_0001);
            assert_eq!(
                resolved.as_ref(),
                Some(expected_op),
                "MATH_1.ops mnemonic {name:?} must resolve to {expected_op:?} via xrom_resolve"
            );
        }
    }

    // ── Phase 33 Plan 33-01: STAT_1 const + bit-1 arm tests ───────────────────

    // Catches: STAT_1 const field regression (id or display name typo).
    #[test]
    fn stat1_const_id_and_name() {
        assert_eq!(
            STAT_1.id, 2,
            "STAT_1.id must be 2 (HP Stat 1 Pac hardware module ID per calc.fjk.ch)"
        );
        assert_eq!(
            STAT_1.name, "STAT 1B",
            "STAT_1.name must be 'STAT 1B' (HP-41C CATALOG 2 display string per OM 00041-90030)"
        );
    }

    // Catches: STAT_1.ops slice growing/shrinking without intent.
    // Count: 14 Σ-prefixed entries from QRC + 10 secondary Σ-prefixed entries
    // (BSTG, MMTGD, MLRXYZ, POLYC, EFXSQ, CTKK + LIN/EXP/LOGI/POW which are
    //  4 secondary curve-fit entries) + RAND + SEED emulator extensions
    // = 26 entries (locked at SPEC.md §"Stat 1 Pac Mnemonics").
    #[test]
    fn stat1_ops_has_correct_entry_count() {
        assert_eq!(
            STAT_1.ops.len(),
            26,
            "STAT_1.ops must carry exactly 26 entries — 24 Σ-prefixed mnemonics + RAND + SEED (SPEC.md §\"Stat 1 Pac Mnemonics\")"
        );
    }

    // Catches: STAT_1.ops mnemonic strings not matching stat1_resolve keys
    // (bidirectional consistency — drift in either direction is a CI failure).
    #[test]
    fn stat1_ops_mnemonics_resolve_consistently() {
        for (name, expected_op) in STAT_1.ops {
            let resolved = xrom_resolve(name, 0b0000_0011);
            assert_eq!(
                resolved.as_ref(),
                Some(expected_op),
                "STAT_1.ops mnemonic {name:?} must resolve to {expected_op:?} via xrom_resolve(name, 0b0000_0011)"
            );
        }
    }

    // Catches: bit-1 isolation regression — STAT_1 must resolve under bit 1
    // ONLY (not bit 0 alone). SPEC.md Req. 2 / D-33.3 bit-1 arm invariant.
    //
    // Plan 33-03 swap: ΣNORMD now resolves to `Op::SigmaNormdWorkflow`
    // (the real 3-mode modal opener) rather than the 33-01 Stat1Stub
    // placeholder. The bit-1 isolation check is identical.
    #[test]
    fn resolve_uses_bit_1_for_stat1() {
        // bit 1 set, bit 0 clear — Stat 1 IS loaded, Math 1 is NOT
        let with_bit1 = xrom_resolve("\u{03A3}NORMD", 0b0000_0010);
        assert_eq!(
            with_bit1,
            Some(Op::SigmaNormdWorkflow),
            "xrom_resolve('ΣNORMD', bit1=1) must route through stat1_resolve"
        );

        // bit 0 set, bit 1 clear — Math 1 IS loaded, Stat 1 is NOT.
        // A Σ-prefixed mnemonic is NOT a Math 1 entry, so resolution returns None.
        let with_bit0 = xrom_resolve("\u{03A3}NORMD", 0b0000_0001);
        assert!(
            with_bit0.is_none(),
            "xrom_resolve('ΣNORMD', bit0=1, bit1=0) must return None (bit-1 isolation)"
        );

        // both bits clear — no XROM modules loaded → None.
        let neither = xrom_resolve("\u{03A3}NORMD", 0b0000_0000);
        assert!(
            neither.is_none(),
            "xrom_resolve must return None when neither bit is set"
        );
    }
}
