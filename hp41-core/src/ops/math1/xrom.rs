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
/// **Plan 33-08 final state:** all 26 entries point to real `Op::Sigma*`
/// / `Op::Rand` / `Op::Seed` variants. The Plan 33-01 scaffolding
/// placeholder Op variant has been removed end-of-Phase-33. The
/// `xrom_shadowing.rs` CI gate cross-checks the slice against
/// `MATH_1.ops` and `builtin_card_op` for disjointness; the bidirectional
/// `stat1_ops_mnemonics_resolve_consistently` test cross-checks this
/// slice against `stat1_resolve`.
pub const STAT_1: XromModule = XromModule {
    id: 2,
    name: "STAT 1B",
    ops: &[
        // ── Stat 1 Pac Univariate / Bivariate Summaries ────────────────────────
        // Plan 33-05: ΣBSTAT + ΣBSTG → real Sigma* variants.
        ("\u{03A3}BSTAT", Op::SigmaBstat), // ΣBSTAT — Plan 33-05
        ("\u{03A3}BSTG", Op::SigmaBstg),   // ΣBSTG  — Plan 33-05
        ("\u{03A3}MMTUG", Op::SigmaMmtug), // ΣMMTUG — Plan 33-06
        ("\u{03A3}MMTGD", Op::SigmaMmtgd), // ΣMMTGD — Plan 33-06
        // ── Stat 1 Pac ANOVA Family ────────────────────────────────────────────
        ("\u{03A3}AOVONE", Op::SigmaAovone), // ΣAOVONE — Plan 33-06
        ("\u{03A3}AOVTWO", Op::SigmaAovtwo), // ΣAOVTWO — Plan 33-06
        ("\u{03A3}ANOCOV", Op::SigmaAnocov), // ΣANOCOV — Plan 33-06
        // ── Stat 1 Pac Curve Fitting + Regression ─────────────────────────────
        // Plan 33-05: ΣLIN / ΣEXP / ΣLOGI / ΣPOW → real Sigma* variants via
        // log-linearization + op_sigma_plus delegate (anti-duplication).
        ("\u{03A3}LIN", Op::SigmaLin),       // ΣLIN    — Plan 33-05
        ("\u{03A3}EXP", Op::SigmaExp),       // ΣEXP    — Plan 33-05
        ("\u{03A3}LOGI", Op::SigmaLogi),     // ΣLOGI   — Plan 33-05
        ("\u{03A3}POW", Op::SigmaPow),       // ΣPOW    — Plan 33-05
        ("\u{03A3}MLRXY", Op::SigmaMlrxy),   // ΣMLRXY  — Plan 33-08
        ("\u{03A3}MLRXYZ", Op::SigmaMlrxyz), // ΣMLRXYZ — Plan 33-08
        ("\u{03A3}POLYP", Op::SigmaPolypWorkflow), // ΣPOLYP — Plan 33-08
        ("\u{03A3}POLYC", Op::SigmaPolyc),   // ΣPOLYC  — Plan 33-08
        // ── Stat 1 Pac Hypothesis Tests ────────────────────────────────────────
        // Plan 33-07: ΣPTST + ΣTSTAT → real Sigma* variants (Tasks 1+2).
        ("\u{03A3}PTST", Op::SigmaPtst),   // ΣPTST   — Plan 33-07
        ("\u{03A3}TSTAT", Op::SigmaTstat), // ΣTSTAT  — Plan 33-07
        // ── Stat 1 Pac Nonparametric / Chi-Square Evaluation / Contingency ────
        // Plan 33-04: ΣSPEAR + ΣXSQEV + ΣEFXSQ real Op variants.
        ("\u{03A3}XSQEV", Op::SigmaXsqev), // ΣXSQEV  — Plan 33-04
        ("\u{03A3}EFXSQ", Op::SigmaEfxsq), // ΣEFXSQ  — Plan 33-04
        ("\u{03A3}CTKKK", Op::SigmaCtkkk), // ΣCTKKK  — Plan 33-06
        ("\u{03A3}CTKK", Op::SigmaCtkk),   // ΣCTKK   — Plan 33-06
        ("\u{03A3}SPEAR", Op::SigmaSpear), // ΣSPEAR  — Plan 33-04
        // ── Stat 1 Pac Distributions ───────────────────────────────────────────
        // Plan 33-03: ΣNORMD + ΣCHISQD → real Sigma* variants.
        ("\u{03A3}NORMD", Op::SigmaNormdWorkflow), // ΣNORMD  — Plan 33-03
        ("\u{03A3}CHISQD", Op::SigmaChisqdWorkflow), // ΣCHISQD — Plan 33-03
        // ── Stat 1 Pac RAND/SEED (emulator extension per D-33.4) ──────────────
        // Plan 33-08: RAND + SEED → real Op variants (final stub swap).
        ("RAND", Op::Rand), // RAND    — Plan 33-08
        ("SEED", Op::Seed), // SEED    — Plan 33-08
    ],
};

/// Time Module registry (D-38.X freeze exception — third entry in this
/// otherwise-frozen file, alongside the bit-2 arm in `xrom_resolve` below).
///
/// - `id = 26` — HP hardware Time Module XROM module ID (XROM 26, "TIME 2C").
/// - `name = "TIME 2C"` — CATALOG 2 display string per HP Time Module Owner's Manual (HP 82182A).
/// - `ops` — 35 Time Module mnemonics per HP 82182A OM function list.
///   All entries confirmed non-shadowing against `MATH_1.ops`, `STAT_1.ops`,
///   and `builtin_card_op` (CI-gated via `tests/xrom_shadowing.rs`).
///
/// **Phase 38 state:** all 35 entries point to stub `Op::Time*` variants.
/// Full clock/date/stopwatch/alarm implementations land in Wave 2 plans.
pub const TIME_MODULE: XromModule = XromModule {
    id: 26,
    name: "TIME 2C",
    ops: &[
        // ── Clock & Display ───────────────────────────────────────────────────
        ("TIME", Op::TimeTime),
        ("DATE", Op::TimeDate),
        ("SETIME", Op::TimeSetime),
        ("SETDATE", Op::TimeSetdate),
        ("CLK12", Op::TimeClk12),
        ("CLK24", Op::TimeClk24),
        ("CLKT", Op::TimeClkt),
        ("CLKTD", Op::TimeClktd),
        ("CLOCK", Op::TimeClock),
        ("CORRECT", Op::TimeCorrect),
        ("T+X", Op::TimeTplusx),
        // ── Date Arithmetic ───────────────────────────────────────────────────
        ("DATE+", Op::TimeDatePlus),
        ("DDAYS", Op::TimeDdays),
        ("DOW", Op::TimeDow),
        ("DMY", Op::TimeDmy),
        ("MDY", Op::TimeMdy),
        // ── Alpha Time/Date Display ───────────────────────────────────────────
        ("ATIME", Op::TimeAtime),
        ("ATIME24", Op::TimeAtime24),
        ("ADATE", Op::TimeAdate),
        // ── Stopwatch ─────────────────────────────────────────────────────────
        ("RUNSW", Op::TimeRunsw),
        ("STOPSW", Op::TimeStopsw),
        ("RCLSW", Op::TimeRclsw),
        ("SETSW", Op::TimeSetsw),
        ("SW", Op::TimeSw),
        ("SWPT", Op::TimeSwpt),
        ("STPW", Op::TimeStpw),
        // ── Alarm System ──────────────────────────────────────────────────────
        ("XYZALM", Op::TimeXyzalm),
        ("ALMCAT", Op::TimeAlmcat),
        ("ALMNOW", Op::TimeAlmnow),
        ("RCLALM", Op::TimeRclalm),
        ("RCLAF", Op::TimeRclaf),
        ("SETAF", Op::TimeSetaf),
        ("CLALMA", Op::TimeClalma),
        ("CLALMX", Op::TimeClalmx),
        ("CLRALMS", Op::TimeClralms),
    ],
};

/// Advantage Pac ADV CONV + ADV MTRX module registry (D-43.X freeze exception).
///
/// - `id = 22` — HP hardware Advantage Pac ADV CONV + ADV MTRX XROM module ID.
/// - `name = "ADV CONV"` — CATALOG 2 display string per HP Advantage Pac OM 00041-90482.
/// - `ops` — ADV CONV (12) + ADV MTRX element-access/lifecycle/reduction/linalg/complex/workflow
///   mnemonics per HP Advantage Pac OM 00041-90482.
///   All entries confirmed non-shadowing against `MATH_1.ops`, `STAT_1.ops`, `TIME_MODULE.ops`
///   (CI-gated via `tests/xrom_shadowing.rs`).
///
/// **Phase 43 state:** all entries point to stub `Op::Adv*` variants.
pub const ADV_MATH_A: XromModule = XromModule {
    id: 22,
    name: "ADV CONV",
    ops: &[
        // ── ADV CONV ──────────────────────────────────────────────────────────────
        ("BININ", Op::AdvBinin),
        ("BINVIEW", Op::AdvBinview),
        ("OCTIN", Op::AdvOctin),
        ("HEXIN", Op::AdvHexin),
        ("HEXVIEW", Op::AdvHexview),
        ("CVTVIEW", Op::AdvCvtview),
        ("NOT", Op::AdvNot),
        ("AND", Op::AdvAnd),
        ("OR", Op::AdvOr),
        ("XOR", Op::AdvXor),
        ("ROTXY", Op::AdvRotxy),
        ("BIT?", Op::AdvBitTest),
        // ── ADV MTRX element access ───────────────────────────────────────────────
        ("I+", Op::AdvIPlus),
        ("I-", Op::AdvIMinus),
        ("J+", Op::AdvJPlus),
        ("J-", Op::AdvJMinus),
        ("MR", Op::AdvMr),
        ("MS", Op::AdvMs),
        ("MRIJ", Op::AdvMrij),
        ("MSIJ", Op::AdvMsij),
        ("MSIJR", Op::AdvMsijr),
        ("MRC+", Op::AdvMrcPlus),
        ("MRC-", Op::AdvMrcMinus),
        ("MRR+", Op::AdvMrrPlus),
        ("MRR-", Op::AdvMrrMinus),
        ("MSR+", Op::AdvMsrPlus),
        ("MSC+", Op::AdvMscPlus),
        ("MSWAP", Op::AdvMswap),
        ("MNAME?", Op::AdvMnameQuery),
        ("DIM?", Op::AdvDimQuery),
        ("MATDIM", Op::AdvMatdim),
        ("MP", Op::AdvMp),
        ("PIV", Op::AdvPiv),
        ("R<>R", Op::AdvRExchangeR),
        ("R>R?", Op::AdvRGtRQuery),
        // ── ADV MTRX reductions ───────────────────────────────────────────────────
        ("SUM", Op::AdvSum),
        ("SUMAB", Op::AdvSumab),
        ("MAX", Op::AdvMax),
        ("MAXAB", Op::AdvMaxab),
        ("MIN", Op::AdvMin),
        ("RMAXAB", Op::AdvRmaxab),
        ("RNRM", Op::AdvRnrm),
        ("RSUM", Op::AdvRsum),
        ("FNRM", Op::AdvFnrm),
        // ── ADV MTRX linear algebra ───────────────────────────────────────────────
        ("MDET", Op::AdvMdet),
        ("MINV", Op::AdvMinv),
        ("MSYS", Op::AdvMsys),
        ("M*M", Op::AdvMMulM),
        ("MAT+", Op::AdvMatPlus),
        ("MAT-", Op::AdvMatMinus),
        ("MAT*C", Op::AdvMatScalarMul),
        ("MAT/C", Op::AdvMatScalarDiv),
        ("TRNPS", Op::AdvTrnps),
        ("MMOVE", Op::AdvMmove),
        // ── ADV MTRX complex ──────────────────────────────────────────────────────
        ("C<>C", Op::AdvCExchangeC),
        ("CMAXAB", Op::AdvCmaxab),
        ("CNRM", Op::AdvCnrm),
        ("CSUM", Op::AdvCsum),
        ("YC+C", Op::AdvYcPlusC),
        // ── ADV MTRX workflow ─────────────────────────────────────────────────────
        ("MATRX", Op::AdvMatrx),
        ("MTR", Op::AdvMtr),
        ("MEDIT", Op::AdvMedit),
        ("CMEDIT", Op::AdvCmedit),
    ],
};

/// Advantage Pac ADV MATH + ADV TVM module registry (D-43.X freeze exception).
///
/// - `id = 24` — HP hardware Advantage Pac ADV MATH + ADV TVM XROM module ID.
/// - `name = "ADV MATH"` — CATALOG 2 display string per HP Advantage Pac OM 00041-90482.
/// - `ops` — ADV MATH complex extensions, polynomial, solvers, curve fitting, vectors,
///   AIP, and ADV TVM mnemonics.
///
/// **Phase 43 state:** all entries point to stub `Op::Adv*` variants.
pub const ADV_MATH_B: XromModule = XromModule {
    id: 24,
    name: "ADV MATH",
    ops: &[
        // ── ADV MATH complex extensions ───────────────────────────────────────────
        ("E^Z", Op::AdvExpZ),
        ("LNZ", Op::AdvLnZ),
        ("LOGZ", Op::AdvLogZ),
        ("Z^N", Op::AdvZPowN),
        ("Z^1/N", Op::AdvZPow1n),
        ("Z^W", Op::AdvZPowW),
        ("Z^1/W", Op::AdvZPow1w),
        ("|Z|", Op::AdvMagz),
        ("SINZ", Op::AdvSinZ),
        ("COSZ", Op::AdvCosZ),
        ("TANZ", Op::AdvTanZ),
        ("A^Z", Op::AdvAPowZ),
        ("CADD", Op::AdvCPlus),
        ("CSUB", Op::AdvCMinus),
        ("CINV", Op::AdvCinv),
        ("CMUL", Op::AdvCMul),
        ("CDIV", Op::AdvCDiv),
        ("AIP", Op::AdvAip),
        // ── ADV MATH polynomial ───────────────────────────────────────────────────
        ("PLY", Op::AdvPly),
        ("RTS", Op::AdvRts),
        // ── ADV MATH solvers ──────────────────────────────────────────────────────
        ("FSOLVE", Op::AdvFsolve),
        ("FINTG", Op::AdvFintg),
        ("FDIFEQ", Op::AdvFdifeq),
        ("FROOT", Op::AdvFroot),
        // ── ADV MATH curve fitting ────────────────────────────────────────────────
        ("CFIT", Op::AdvCfit),
        ("AS", Op::AdvAs),
        ("DS", Op::AdvDs),
        ("BFIT", Op::AdvBfit),
        ("FIT", Op::AdvFit),
        ("Y?X", Op::AdvYQueryX),
        ("SZ?", Op::AdvSzQuery),
        // ── ADV MATH vectors ──────────────────────────────────────────────────────
        ("V+", Op::AdvVPlus),
        ("V-", Op::AdvVMinus),
        ("DOT", Op::AdvDot),
        ("CROSS", Op::AdvCross),
        ("VC", Op::AdvVc),
        ("VS", Op::AdvVs),
        ("VR", Op::AdvVr),
        ("VE", Op::AdvVe),
        ("VXY", Op::AdvVxy),
        ("UV", Op::AdvUv),
        ("|V|", Op::AdvVMag),
        ("V*", Op::AdvVStar),
        ("VD", Op::AdvVd),
        ("TR", Op::AdvTr),
        // ── ADV TVM ───────────────────────────────────────────────────────────────
        ("TVM", Op::AdvTvm),
        ("N", Op::AdvTvmN),
        ("PV", Op::AdvTvmPv),
        ("PMT", Op::AdvTvmPmt),
        ("FV", Op::AdvTvmFv),
        ("*I", Op::AdvTvmStarI),
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
    // Phase 38 (v3.2): Time Module bit-2 arm — fires AFTER Stat 1 per
    // Pitfall 1 + Pitfall 22. `TIME_MODULE.ops` x `MATH_1.ops` x `STAT_1.ops`
    // disjointness CI-gated in `tests/xrom_shadowing.rs`.
    if modules & 0b0000_0100 != 0 {
        if let Some(op) = time_resolve(name) {
            return Some(op);
        }
    }
    // Phase 43 (v3.3): ADV CONV+MTRX bit-3 arm — fires AFTER Time Module per
    // Pitfall 1 + Pitfall 22. `ADV_MATH_A.ops` confirmed disjoint from all prior
    // modules (CI-gated via `tests/xrom_shadowing.rs`).
    if modules & 0b0000_1000 != 0 {
        if let Some(op) = adv_a_resolve(name) {
            return Some(op);
        }
    }
    // Phase 43 (v3.3): ADV MATH+TVM bit-4 arm — fires AFTER ADV CONV+MTRX.
    // `ADV_MATH_B.ops` confirmed disjoint from all prior modules.
    if modules & 0b0001_0000 != 0 {
        if let Some(op) = adv_b_resolve(name) {
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
/// Each arm maps to a real `Op::Sigma*` / `Op::Rand` / `Op::Seed`
/// variant (Plan 33-08 final state; the Plan 33-01 scaffolding
/// placeholder Op variant was removed at the end of Phase 33).
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
        // Plan 33-06: ΣMMTUG + ΣMMTGD → real Sigma* variants.
        "\u{03A3}MMTUG" => Some(Op::SigmaMmtug),
        "\u{03A3}MMTGD" => Some(Op::SigmaMmtgd),
        // ANOVA Family
        // Plan 33-06: ΣAOVONE + ΣAOVTWO + ΣANOCOV → real Sigma* variants.
        "\u{03A3}AOVONE" => Some(Op::SigmaAovone),
        "\u{03A3}AOVTWO" => Some(Op::SigmaAovtwo),
        "\u{03A3}ANOCOV" => Some(Op::SigmaAnocov),
        // Curve Fitting + Regression
        // Plan 33-05: ΣLIN / ΣEXP / ΣLOGI / ΣPOW → real Sigma* variants.
        "\u{03A3}LIN" => Some(Op::SigmaLin),
        "\u{03A3}EXP" => Some(Op::SigmaExp),
        "\u{03A3}LOGI" => Some(Op::SigmaLogi),
        "\u{03A3}POW" => Some(Op::SigmaPow),
        // Plan 33-08: Multiple + polynomial regression → real Sigma* variants.
        "\u{03A3}MLRXY" => Some(Op::SigmaMlrxy),
        "\u{03A3}MLRXYZ" => Some(Op::SigmaMlrxyz),
        "\u{03A3}POLYP" => Some(Op::SigmaPolypWorkflow),
        "\u{03A3}POLYC" => Some(Op::SigmaPolyc),
        // Hypothesis Tests
        // Plan 33-07: ΣPTST + ΣTSTAT → real Sigma* variants (Tasks 1+2).
        "\u{03A3}PTST" => Some(Op::SigmaPtst),
        "\u{03A3}TSTAT" => Some(Op::SigmaTstat),
        // Nonparametric / Chi-Square Evaluation / Contingency
        // Plan 33-04: ΣSPEAR + ΣXSQEV + ΣEFXSQ → real Sigma* variants.
        "\u{03A3}XSQEV" => Some(Op::SigmaXsqev),
        "\u{03A3}EFXSQ" => Some(Op::SigmaEfxsq),
        // Plan 33-06: ΣCTKKK + ΣCTKK → real Sigma* variants.
        "\u{03A3}CTKKK" => Some(Op::SigmaCtkkk),
        "\u{03A3}CTKK" => Some(Op::SigmaCtkk),
        "\u{03A3}SPEAR" => Some(Op::SigmaSpear),
        // Distributions
        // Plan 33-03: ΣNORMD + ΣCHISQD → real Sigma* variants.
        "\u{03A3}NORMD" => Some(Op::SigmaNormdWorkflow),
        "\u{03A3}CHISQD" => Some(Op::SigmaChisqdWorkflow),
        // Plan 33-08: RAND + SEED → real Op variants (emulator extension).
        "RAND" => Some(Op::Rand),
        "SEED" => Some(Op::Seed),
        _ => None,
    }
}

/// Time Module (bit 2) mnemonic resolver — Phase 38 freeze exception.
///
/// Bidirectional consistency with `TIME_MODULE.ops`: the
/// `time_module_ops_mnemonics_resolve_consistently` test iterates the
/// slice and asserts every mnemonic round-trips through this match.
fn time_resolve(name: &str) -> Option<Op> {
    match name {
        // ── Clock & Display ───────────────────────────────────────────────────
        "TIME" => Some(Op::TimeTime),
        "DATE" => Some(Op::TimeDate),
        "SETIME" => Some(Op::TimeSetime),
        "SETDATE" => Some(Op::TimeSetdate),
        "CLK12" => Some(Op::TimeClk12),
        "CLK24" => Some(Op::TimeClk24),
        "CLKT" => Some(Op::TimeClkt),
        "CLKTD" => Some(Op::TimeClktd),
        "CLOCK" => Some(Op::TimeClock),
        "CORRECT" => Some(Op::TimeCorrect),
        "T+X" => Some(Op::TimeTplusx),
        // ── Date Arithmetic ───────────────────────────────────────────────────
        "DATE+" => Some(Op::TimeDatePlus),
        "DDAYS" => Some(Op::TimeDdays),
        "DOW" => Some(Op::TimeDow),
        "DMY" => Some(Op::TimeDmy),
        "MDY" => Some(Op::TimeMdy),
        // ── Alpha Time/Date Display ───────────────────────────────────────────
        "ATIME" => Some(Op::TimeAtime),
        "ATIME24" => Some(Op::TimeAtime24),
        "ADATE" => Some(Op::TimeAdate),
        // ── Stopwatch ─────────────────────────────────────────────────────────
        "RUNSW" => Some(Op::TimeRunsw),
        "STOPSW" => Some(Op::TimeStopsw),
        "RCLSW" => Some(Op::TimeRclsw),
        "SETSW" => Some(Op::TimeSetsw),
        "SW" => Some(Op::TimeSw),
        "SWPT" => Some(Op::TimeSwpt),
        "STPW" => Some(Op::TimeStpw),
        // ── Alarm System ──────────────────────────────────────────────────────
        "XYZALM" => Some(Op::TimeXyzalm),
        "ALMCAT" => Some(Op::TimeAlmcat),
        "ALMNOW" => Some(Op::TimeAlmnow),
        "RCLALM" => Some(Op::TimeRclalm),
        "RCLAF" => Some(Op::TimeRclaf),
        "SETAF" => Some(Op::TimeSetaf),
        "CLALMA" => Some(Op::TimeClalma),
        "CLALMX" => Some(Op::TimeClalmx),
        "CLRALMS" => Some(Op::TimeClralms),
        _ => None,
    }
}

/// Advantage Pac ADV CONV + ADV MTRX (bit 3) mnemonic resolver — Phase 43.
///
/// All entries mirror `ADV_MATH_A.ops` for bidirectional consistency.
fn adv_a_resolve(name: &str) -> Option<Op> {
    match name {
        // ADV CONV
        "BININ" => Some(Op::AdvBinin),
        "BINVIEW" => Some(Op::AdvBinview),
        "OCTIN" => Some(Op::AdvOctin),
        "HEXIN" => Some(Op::AdvHexin),
        "HEXVIEW" => Some(Op::AdvHexview),
        "CVTVIEW" => Some(Op::AdvCvtview),
        "NOT" => Some(Op::AdvNot),
        "AND" => Some(Op::AdvAnd),
        "OR" => Some(Op::AdvOr),
        "XOR" => Some(Op::AdvXor),
        "ROTXY" => Some(Op::AdvRotxy),
        "BIT?" => Some(Op::AdvBitTest),
        // ADV MTRX element access
        "I+" => Some(Op::AdvIPlus),
        "I-" => Some(Op::AdvIMinus),
        "J+" => Some(Op::AdvJPlus),
        "J-" => Some(Op::AdvJMinus),
        "MR" => Some(Op::AdvMr),
        "MS" => Some(Op::AdvMs),
        "MRIJ" => Some(Op::AdvMrij),
        "MSIJ" => Some(Op::AdvMsij),
        "MSIJR" => Some(Op::AdvMsijr),
        "MRC+" => Some(Op::AdvMrcPlus),
        "MRC-" => Some(Op::AdvMrcMinus),
        "MRR+" => Some(Op::AdvMrrPlus),
        "MRR-" => Some(Op::AdvMrrMinus),
        "MSR+" => Some(Op::AdvMsrPlus),
        "MSC+" => Some(Op::AdvMscPlus),
        "MSWAP" => Some(Op::AdvMswap),
        "MNAME?" => Some(Op::AdvMnameQuery),
        "DIM?" => Some(Op::AdvDimQuery),
        "MATDIM" => Some(Op::AdvMatdim),
        "MP" => Some(Op::AdvMp),
        "PIV" => Some(Op::AdvPiv),
        "R<>R" => Some(Op::AdvRExchangeR),
        "R>R?" => Some(Op::AdvRGtRQuery),
        // ADV MTRX reductions
        "SUM" => Some(Op::AdvSum),
        "SUMAB" => Some(Op::AdvSumab),
        "MAX" => Some(Op::AdvMax),
        "MAXAB" => Some(Op::AdvMaxab),
        "MIN" => Some(Op::AdvMin),
        "RMAXAB" => Some(Op::AdvRmaxab),
        "RNRM" => Some(Op::AdvRnrm),
        "RSUM" => Some(Op::AdvRsum),
        "FNRM" => Some(Op::AdvFnrm),
        // ADV MTRX linear algebra
        "MDET" => Some(Op::AdvMdet),
        "MINV" => Some(Op::AdvMinv),
        "MSYS" => Some(Op::AdvMsys),
        "M*M" => Some(Op::AdvMMulM),
        "MAT+" => Some(Op::AdvMatPlus),
        "MAT-" => Some(Op::AdvMatMinus),
        "MAT*C" => Some(Op::AdvMatScalarMul),
        "MAT/C" => Some(Op::AdvMatScalarDiv),
        "TRNPS" => Some(Op::AdvTrnps),
        "MMOVE" => Some(Op::AdvMmove),
        // ADV MTRX complex
        "C<>C" => Some(Op::AdvCExchangeC),
        "CMAXAB" => Some(Op::AdvCmaxab),
        "CNRM" => Some(Op::AdvCnrm),
        "CSUM" => Some(Op::AdvCsum),
        "YC+C" => Some(Op::AdvYcPlusC),
        // ADV MTRX workflow
        "MATRX" => Some(Op::AdvMatrx),
        "MTR" => Some(Op::AdvMtr),
        "MEDIT" => Some(Op::AdvMedit),
        "CMEDIT" => Some(Op::AdvCmedit),
        _ => None,
    }
}

/// Advantage Pac ADV MATH + ADV TVM (bit 4) mnemonic resolver — Phase 43.
///
/// All entries mirror `ADV_MATH_B.ops` for bidirectional consistency.
fn adv_b_resolve(name: &str) -> Option<Op> {
    match name {
        // ADV MATH complex extensions
        "E^Z" => Some(Op::AdvExpZ),
        "LNZ" => Some(Op::AdvLnZ),
        "LOGZ" => Some(Op::AdvLogZ),
        "Z^N" => Some(Op::AdvZPowN),
        "Z^1/N" => Some(Op::AdvZPow1n),
        "Z^W" => Some(Op::AdvZPowW),
        "Z^1/W" => Some(Op::AdvZPow1w),
        "|Z|" => Some(Op::AdvMagz),
        "SINZ" => Some(Op::AdvSinZ),
        "COSZ" => Some(Op::AdvCosZ),
        "TANZ" => Some(Op::AdvTanZ),
        "A^Z" => Some(Op::AdvAPowZ),
        "CADD" => Some(Op::AdvCPlus),
        "CSUB" => Some(Op::AdvCMinus),
        "CINV" => Some(Op::AdvCinv),
        "CMUL" => Some(Op::AdvCMul),
        "CDIV" => Some(Op::AdvCDiv),
        "AIP" => Some(Op::AdvAip),
        // ADV MATH polynomial
        "PLY" => Some(Op::AdvPly),
        "RTS" => Some(Op::AdvRts),
        // ADV MATH solvers
        "FSOLVE" => Some(Op::AdvFsolve),
        "FINTG" => Some(Op::AdvFintg),
        "FDIFEQ" => Some(Op::AdvFdifeq),
        "FROOT" => Some(Op::AdvFroot),
        // ADV MATH curve fitting
        "CFIT" => Some(Op::AdvCfit),
        "AS" => Some(Op::AdvAs),
        "DS" => Some(Op::AdvDs),
        "BFIT" => Some(Op::AdvBfit),
        "FIT" => Some(Op::AdvFit),
        "Y?X" => Some(Op::AdvYQueryX),
        "SZ?" => Some(Op::AdvSzQuery),
        // ADV MATH vectors
        "V+" => Some(Op::AdvVPlus),
        "V-" => Some(Op::AdvVMinus),
        "DOT" => Some(Op::AdvDot),
        "CROSS" => Some(Op::AdvCross),
        "VC" => Some(Op::AdvVc),
        "VS" => Some(Op::AdvVs),
        "VR" => Some(Op::AdvVr),
        "VE" => Some(Op::AdvVe),
        "VXY" => Some(Op::AdvVxy),
        "UV" => Some(Op::AdvUv),
        "|V|" => Some(Op::AdvVMag),
        "V*" => Some(Op::AdvVStar),
        "VD" => Some(Op::AdvVd),
        "TR" => Some(Op::AdvTr),
        // ADV TVM
        "TVM" => Some(Op::AdvTvm),
        "N" => Some(Op::AdvTvmN),
        "PV" => Some(Op::AdvTvmPv),
        "PMT" => Some(Op::AdvTvmPmt),
        "FV" => Some(Op::AdvTvmFv),
        "*I" => Some(Op::AdvTvmStarI),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{xrom_resolve, ADV_MATH_A, ADV_MATH_B, MATH_1, STAT_1, TIME_MODULE};
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
    // (the real 3-mode modal opener) rather than the 33-01 scaffolding
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

    // ── Phase 38 (v3.2): TIME_MODULE const + bit-2 arm tests ─────────────────

    // Catches: TIME_MODULE const field regression (id or display name typo).
    #[test]
    fn time_module_const_id_and_name() {
        assert_eq!(
            TIME_MODULE.id, 26,
            "TIME_MODULE.id must be 26 (HP Time Module hardware XROM ID per HP 82182A)"
        );
        assert_eq!(
            TIME_MODULE.name, "TIME 2C",
            "TIME_MODULE.name must be 'TIME 2C' (HP-41C CATALOG 2 display string per HP 82182A)"
        );
    }

    // Catches: TIME_MODULE.ops slice growing/shrinking without intent.
    // 35 entries: 11 clock/display + 6 date-arith + 3 alpha + 7 stopwatch + 8 alarm = 35.
    #[test]
    fn time_module_ops_has_correct_entry_count() {
        assert_eq!(
            TIME_MODULE.ops.len(),
            35,
            "TIME_MODULE.ops must have exactly 35 entries (11 clock + 6 date + 3 alpha + 7 stopwatch + 8 alarm)"
        );
    }

    // Catches: bit-2 isolation regression — Time Module must resolve under bit 2 ONLY.
    #[test]
    fn resolve_uses_bit_2_for_time_module() {
        // bit 2 set, bits 0+1 clear — Time Module IS loaded, Math 1 + Stat 1 are NOT.
        let with_bit2 = xrom_resolve("TIME", 0b0000_0100);
        assert_eq!(
            with_bit2,
            Some(Op::TimeTime),
            "xrom_resolve('TIME', bit2=1) must route through time_resolve"
        );

        // bit 0+1 set, bit 2 clear — Math 1 + Stat 1 loaded, Time Module is NOT.
        // 'TIME' is not a Math 1 or Stat 1 mnemonic → resolution returns None.
        let without_bit2 = xrom_resolve("TIME", 0b0000_0011);
        assert!(
            without_bit2.is_none(),
            "xrom_resolve('TIME', bit2=0) must return None (bit-2 isolation)"
        );

        // All three bits set — all modules loaded → 'TIME' resolves via bit-2 arm.
        let all_bits = xrom_resolve("TIME", 0b0000_0111);
        assert_eq!(
            all_bits,
            Some(Op::TimeTime),
            "xrom_resolve('TIME', bits 0+1+2 set) must return Some(Op::TimeTime)"
        );
    }

    // ── Phase 43 (v3.3): ADV_MATH_A const + bit-3 arm tests ─────────────────

    // Catches: ADV_MATH_A const field regression (id or display name typo).
    #[test]
    fn adv_math_a_const_id_and_name() {
        assert_eq!(
            ADV_MATH_A.id, 22,
            "ADV_MATH_A.id must be 22 (HP Advantage Pac ADV CONV+MTRX hardware XROM ID per OM 00041-90482)"
        );
        assert_eq!(
            ADV_MATH_A.name, "ADV CONV",
            "ADV_MATH_A.name must be 'ADV CONV' (HP-41C CATALOG 2 display string per OM 00041-90482)"
        );
    }

    // Catches: ADV_MATH_A.ops slice size regression.
    #[test]
    fn adv_math_a_ops_entry_count() {
        // 12 CONV + 23 element-access + 9 reductions + 10 linalg + 5 complex + 4 workflow = 63
        assert_eq!(
            ADV_MATH_A.ops.len(),
            63,
            "ADV_MATH_A.ops must carry exactly 63 entries (D-43 Advantage Pac XROM 22 ops)"
        );
    }

    // Catches: ADV_MATH_A.ops mnemonic strings not matching adv_a_resolve keys
    #[test]
    fn adv_math_a_ops_mnemonics_resolve_consistently() {
        for (name, expected_op) in ADV_MATH_A.ops {
            let resolved = xrom_resolve(name, 0b0000_1000);
            assert_eq!(
                resolved.as_ref(),
                Some(expected_op),
                "ADV_MATH_A.ops mnemonic {name:?} must resolve via bit-3 arm"
            );
        }
    }

    // Catches: bit-3 isolation regression — ADV CONV+MTRX must resolve under bit 3 ONLY.
    #[test]
    fn resolve_uses_bit_3_for_adv_math_a() {
        // bit 3 set, bits 0+1+2 clear — ADV CONV+MTRX IS loaded.
        let with_bit3 = xrom_resolve("BININ", 0b0000_1000);
        assert_eq!(
            with_bit3,
            Some(Op::AdvBinin),
            "xrom_resolve('BININ', bit3=1) must route through adv_a_resolve"
        );

        // bits 0+1+2 set, bit 3 clear — ADV CONV+MTRX NOT loaded.
        let without_bit3 = xrom_resolve("BININ", 0b0000_0111);
        assert!(
            without_bit3.is_none(),
            "xrom_resolve('BININ', bit3=0) must return None (bit-3 isolation)"
        );

        // All five bits set — 'BININ' resolves via bit-3 arm.
        let all_bits = xrom_resolve("BININ", 0b0001_1111);
        assert_eq!(
            all_bits,
            Some(Op::AdvBinin),
            "xrom_resolve('BININ', all 5 bits set) must return Some(Op::AdvBinin)"
        );
    }

    // ── Phase 43 (v3.3): ADV_MATH_B const + bit-4 arm tests ─────────────────

    // Catches: ADV_MATH_B const field regression (id or display name typo).
    #[test]
    fn adv_math_b_const_id_and_name() {
        assert_eq!(
            ADV_MATH_B.id, 24,
            "ADV_MATH_B.id must be 24 (HP Advantage Pac ADV MATH+TVM hardware XROM ID per OM 00041-90482)"
        );
        assert_eq!(
            ADV_MATH_B.name, "ADV MATH",
            "ADV_MATH_B.name must be 'ADV MATH' (HP-41C CATALOG 2 display string per OM 00041-90482)"
        );
    }

    // Catches: ADV_MATH_B.ops slice size regression.
    #[test]
    fn adv_math_b_ops_entry_count() {
        // 18 complex-ext + 2 poly + 4 solvers + 7 curve-fit + 14 vectors + 6 TVM = 51
        assert_eq!(
            ADV_MATH_B.ops.len(),
            51,
            "ADV_MATH_B.ops must carry exactly 51 entries (D-43 Advantage Pac XROM 24 ops)"
        );
    }

    // Catches: ADV_MATH_B.ops mnemonic strings not matching adv_b_resolve keys
    #[test]
    fn adv_math_b_ops_mnemonics_resolve_consistently() {
        for (name, expected_op) in ADV_MATH_B.ops {
            let resolved = xrom_resolve(name, 0b0001_0000);
            assert_eq!(
                resolved.as_ref(),
                Some(expected_op),
                "ADV_MATH_B.ops mnemonic {name:?} must resolve via bit-4 arm"
            );
        }
    }

    // Catches: bit-4 isolation regression — ADV MATH+TVM must resolve under bit 4 ONLY.
    #[test]
    fn resolve_uses_bit_4_for_adv_math_b() {
        // bit 4 set, bits 0+1+2+3 clear.
        let with_bit4 = xrom_resolve("TVM", 0b0001_0000);
        assert_eq!(
            with_bit4,
            Some(Op::AdvTvm),
            "xrom_resolve('TVM', bit4=1) must route through adv_b_resolve"
        );

        // bits 0+1+2+3 set, bit 4 clear — ADV MATH+TVM NOT loaded.
        let without_bit4 = xrom_resolve("TVM", 0b0000_1111);
        assert!(
            without_bit4.is_none(),
            "xrom_resolve('TVM', bit4=0) must return None (bit-4 isolation)"
        );

        // All five bits set — 'TVM' resolves via bit-4 arm.
        let all_bits = xrom_resolve("TVM", 0b0001_1111);
        assert_eq!(
            all_bits,
            Some(Op::AdvTvm),
            "xrom_resolve('TVM', all 5 bits set) must return Some(Op::AdvTvm)"
        );
    }

    // Catches: ADV_MATH_B FSOLVE_RUN_LOOP stubs not in module ops table (internal
    // ops not XEQ-reachable; this test confirms resolver does NOT expose them).
    #[test]
    fn adv_fsolve_run_loop_not_in_module_table() {
        // The *_run_loop ops are internal re-entry points, not user-callable mnemonics.
        let result = xrom_resolve("FSOLVE_RUN_LOOP", 0b0001_1111);
        assert!(result.is_none(), "FSOLVE_RUN_LOOP must not be XEQ-resolvable");
    }

    // Catches: migration default_xrom_modules bit-3 + bit-4 sanity
    #[test]
    fn adv_modules_bit_positions_sanity() {
        assert_eq!(0b0001_1111u8 & 0b0000_1000, 0b0000_1000, "bit 3 = ADV_MATH_A");
        assert_eq!(0b0001_1111u8 & 0b0001_0000, 0b0001_0000, "bit 4 = ADV_MATH_B");
    }
}
