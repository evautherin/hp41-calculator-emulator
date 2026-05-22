use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum HpError {
    #[error("overflow")]
    Overflow,
    #[error("divide by zero")]
    DivideByZero,
    #[error("invalid operation")]
    InvalidOp,
    #[error("domain error")]
    Domain,
    /// Hardware-spec out-of-range (Phase 20 / D-06): FACT with X > 69 returns this
    /// before the magnitude `Overflow` path is reached. Preserves the HP-41
    /// "X > 69 → OUT OF RANGE" wording from SC-3 / FN-MATH-08.
    #[error("out of range")]
    OutOfRange,
    /// HP-41 subroutine call-depth exceeded (5th nested XEQ).
    #[error("try again")]
    CallDepth,
    /// HMS field-range validation: minutes >= 60 or seconds >= 60.
    #[error("invalid input")]
    InvalidInput,
    /// Card Reader: WDTA/RDTA/WPRGM/RDPRGM with an empty ALPHA register.
    /// Matches the hardware-faithful "ALPHA DATA" message on real HP-41 card readers.
    #[error("alpha data")]
    AlphaData,
    /// Card Reader: card payload could not be encoded/decoded. Carries a short
    /// diagnostic (serde line/col, "unsupported op", "truncated", etc.) so the
    /// frontend can surface something more useful than a generic "CARD DATA".
    #[error("card data: {0}")]
    CardData(String),
    /// User-initiated cancellation of a long-running solver (INTG/SOLVE/DIFEQ).
    /// Distinct from Domain. Surfaces as "CANCELED" in GUI/CLI.
    /// D-28.7 / D-28.8 / D-28.9; wiring in Phase 31 / GUI-05.
    #[error("canceled")]
    Canceled,
    /// Hard iteration-cap exhaustion in a Stat 1 Pac iterative quantile loop
    /// (Plan 33-03 ΣNORMD inverse; Plan 33-07 ΣPTST). SPEC.md Req. 34 /
    /// D-33.5 / Pitfall 11: every Stat 1 iterative path declares a 50-iter
    /// hard cap and surfaces this variant when the loop fails to converge
    /// to the display-mode-tied tolerance band. Distinct from `Domain`
    /// (which already covers AS-239 / AS-63 distribution-primitive
    /// non-convergence at the bare f64 layer) so the outer Op layer can
    /// distinguish iter-cap from domain-rejection failures.
    #[error("convergence failed")]
    ConvergenceFailed,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::HpError;

    // Catches: Display regression if thiserror attribute is mistyped
    #[test]
    fn canceled_display() {
        assert_eq!(HpError::Canceled.to_string(), "canceled");
    }

    // Catches: PartialEq or Clone derive regression
    #[test]
    fn canceled_partial_eq() {
        assert_eq!(HpError::Canceled, HpError::Canceled);
        assert_eq!(HpError::Canceled.clone(), HpError::Canceled);
    }

    // Catches: variant conflation with Domain or other error types
    #[test]
    fn canceled_distinct_from_domain() {
        assert_ne!(HpError::Canceled, HpError::Domain);
    }
}
