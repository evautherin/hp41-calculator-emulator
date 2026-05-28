use hp41_core::ops::Op;
use hp41_core::ops::Op::*;

fn resolve(name: &str) -> Option<Op> {
    hp41_cli::keys::xeq_by_name_local_resolve(name, 0b0000_0011)
}

#[test]
fn trig_resolves() {
    assert_eq!(resolve("SIN"), Some(Sin));
    assert_eq!(resolve("COS"), Some(Cos));
    assert_eq!(resolve("TAN"), Some(Tan));
    assert_eq!(resolve("ASIN"), Some(Asin));
    assert_eq!(resolve("ACOS"), Some(Acos));
    assert_eq!(resolve("ATAN"), Some(Atan));
}

#[test]
fn log_exp_resolves() {
    assert_eq!(resolve("LN"), Some(Ln));
    assert_eq!(resolve("LOG"), Some(Log));
    assert_eq!(resolve("E^X"), Some(Exp));
    assert_eq!(resolve("10^X"), Some(TenPow));
}

#[test]
fn power_root_resolves() {
    assert_eq!(resolve("SQRT"), Some(Sqrt));
    assert_eq!(resolve("X^2"), Some(Sq));
    assert_eq!(resolve("XSQ"), Some(Sq));
    assert_eq!(resolve("Y^X"), Some(YPow));
    assert_eq!(resolve("1/X"), Some(Recip));
    assert_eq!(resolve("RECIP"), Some(Recip));
}

#[test]
fn basic_math_resolves() {
    assert_eq!(resolve("PI"), Some(Pi));
    assert_eq!(resolve("ABS"), Some(Abs));
    assert_eq!(resolve("INT"), Some(Int));
    assert_eq!(resolve("FRC"), Some(Frc));
    assert_eq!(resolve("SIGN"), Some(Sign));
    assert_eq!(resolve("N!"), Some(Fact));
    assert_eq!(resolve("FACT"), Some(Fact));
    assert_eq!(resolve("MOD"), Some(Mod));
    assert_eq!(resolve("RND"), Some(Rnd));
}

#[test]
fn conversion_resolves() {
    assert_eq!(resolve("P->R"), Some(PolarToRect));
    assert_eq!(resolve("P\u{2192}R"), Some(PolarToRect));
    assert_eq!(resolve("R->P"), Some(RectToPolar));
    assert_eq!(resolve("R\u{2192}P"), Some(RectToPolar));
    assert_eq!(resolve("HMS->H"), Some(HmsToH));
    assert_eq!(resolve("HMS\u{2192}H"), Some(HmsToH));
    assert_eq!(resolve("H->HMS"), Some(HToHms));
    assert_eq!(resolve("H\u{2192}HMS"), Some(HToHms));
    assert_eq!(resolve("HMS+"), Some(HmsAdd));
    assert_eq!(resolve("HMS-"), Some(HmsSub));
}

#[test]
fn angle_mode_resolves() {
    assert_eq!(resolve("DEG"), Some(SetDeg));
    assert_eq!(resolve("RAD"), Some(SetRad));
    assert_eq!(resolve("GRAD"), Some(SetGrad));
}

#[test]
fn stack_clear_resolves() {
    assert_eq!(resolve("R^"), Some(Rup));
    assert_eq!(resolve("R\u{2191}"), Some(Rup));
    assert_eq!(resolve("RUP"), Some(Rup));
    assert_eq!(resolve("CLST"), Some(Clst));
    assert_eq!(resolve("CLREG"), Some(Clreg));
    assert_eq!(resolve("CLA"), Some(Cla));
}

#[test]
fn statistics_resolves() {
    assert_eq!(resolve("SIGMA+"), Some(SigmaPlus));
    assert_eq!(resolve("\u{03A3}+"), Some(SigmaPlus));
    assert_eq!(resolve("SIGMA-"), Some(SigmaMinus));
    assert_eq!(resolve("\u{03A3}-"), Some(SigmaMinus));
    assert_eq!(resolve("MEAN"), Some(Mean));
    assert_eq!(resolve("SDEV"), Some(Sdev));
    assert_eq!(resolve("L.R."), Some(LR));
    assert_eq!(resolve("LR"), Some(LR));
    assert_eq!(resolve("YHAT"), Some(Yhat));
    assert_eq!(resolve("CORR"), Some(Corr));
    assert_eq!(resolve("CL SIGMA"), Some(ClSigmaStat));
    assert_eq!(resolve("CL\u{03A3}"), Some(ClSigmaStat));
    assert_eq!(resolve("CLSIGMA"), Some(ClSigmaStat));
}

#[test]
fn display_alpha_sound_resolves() {
    assert_eq!(resolve("AVIEW"), Some(AView));
    assert_eq!(resolve("PROMPT"), Some(Prompt));
    assert_eq!(resolve("AON"), Some(Aon));
    assert_eq!(resolve("AOFF"), Some(Aoff));
    assert_eq!(resolve("CLD"), Some(Cld));
    assert_eq!(resolve("BEEP"), Some(Beep));
    assert_eq!(resolve("CLRALPHA"), Some(AlphaClear));
}

#[test]
fn alpha_ops_resolves() {
    assert_eq!(resolve("ATOX"), Some(Atox));
    assert_eq!(resolve("XTOA"), Some(Xtoa));
    assert_eq!(resolve("AROT"), Some(Arot));
    assert_eq!(resolve("POSA"), Some(Posa));
}

#[test]
fn program_control_resolves() {
    assert_eq!(resolve("RTN"), Some(Rtn));
    assert_eq!(resolve("STOP"), Some(Stop));
    assert_eq!(resolve("PSE"), Some(Pse));
    assert_eq!(resolve("PACK"), Some(Pack));
    assert_eq!(resolve("INS"), Some(Ins));
}

#[test]
fn print_resolves() {
    assert_eq!(resolve("PRX"), Some(PRX));
    assert_eq!(resolve("PRA"), Some(PRA));
    assert_eq!(resolve("PRSTK"), Some(PRSTK));
}

#[test]
fn case_sensitivity_preserved() {
    assert_eq!(resolve("sin"), None);
    assert_eq!(resolve("Sin"), None);
    assert_eq!(resolve("cos"), None);
    assert_eq!(resolve("pi"), None);
    assert_eq!(resolve("sqrt"), None);
    assert_eq!(resolve("ln"), None);
}

#[test]
fn unknown_names_return_none() {
    assert_eq!(resolve("FOOBAR"), None);
    assert_eq!(resolve(""), None);
    assert_eq!(resolve("XYZZY"), None);
}

#[test]
fn xmem_builtins_resolve() {
    // Phase 52 (v4.0): X-MEM built-in ops (HP-41CX Extended Functions / D-52.4).
    // These resolve via builtin_card_op — no changes to keys.rs or key_map.rs needed.
    assert_eq!(resolve("EMDIR"), Some(EmDir));
    assert_eq!(resolve("EMROOM"), Some(EmRoom));
    assert_eq!(resolve("SAVEP"), Some(SaveP));
    assert_eq!(resolve("GETP"), Some(GetP));
    assert_eq!(resolve("SAVED"), Some(SaveD));
    assert_eq!(resolve("GETD"), Some(GetD));
    assert_eq!(resolve("EMREG"), Some(EmReg));
    assert_eq!(resolve("SAVERX"), Some(SaveRx));
}

#[test]
fn xmem_builtins_are_xrom_module_independent() {
    // X-MEM ops are HP-41CX OS built-ins, NOT XROM module functions. They MUST
    // resolve regardless of the xrom_modules bitfield — proving resolution flows
    // through builtin_card_op, not xrom_resolve (D-52.4 / RESEARCH.md Pitfall 1).
    // Guards against a future regression that moves X-MEM into xrom_resolve.
    let resolve_no_xrom = |name: &str| hp41_cli::keys::xeq_by_name_local_resolve(name, 0b0000_0000);
    assert_eq!(resolve_no_xrom("EMDIR"), Some(EmDir));
    assert_eq!(resolve_no_xrom("EMROOM"), Some(EmRoom));
    assert_eq!(resolve_no_xrom("SAVEP"), Some(SaveP));
    assert_eq!(resolve_no_xrom("GETP"), Some(GetP));
    assert_eq!(resolve_no_xrom("SAVED"), Some(SaveD));
    assert_eq!(resolve_no_xrom("GETD"), Some(GetD));
    assert_eq!(resolve_no_xrom("EMREG"), Some(EmReg));
    assert_eq!(resolve_no_xrom("SAVERX"), Some(SaveRx));
}

#[test]
fn xmem_unknown_still_none() {
    // After wiring, unknown X-MEM-like names must still return None (never-discard D-07).
    assert_eq!(resolve("EMDIR2"), None);
    assert_eq!(resolve("SAVERX2"), None);
}
