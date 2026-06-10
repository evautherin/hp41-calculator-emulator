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
    /// TVM *I solver iteration-cap exhaustion (D-43.13 / ADV-TVM-06).
    /// Newton-Raphson failed to converge within `TVM_MAX_ITERATIONS` for the
    /// TVM interest-rate equation. The last iterate is pushed to X and
    /// "NO SOLUTION" is pushed to `print_buffer` before this error is returned.
    /// Distinct from `ConvergenceFailed` (Stat 1 quantile loops) to allow
    /// CLI/GUI to distinguish TVM-specific non-convergence.
    #[error("no root found")]
    NoRoot,
    /// X-MEM: named file not found (GETP/GETD/EMREG on missing name, or EMREG
    /// with no active file set). Matches HP-41CX QRG p.39 "FL NOT FOUND".
    #[error("fl not found")]
    FileNotFound,
    /// X-MEM: file type mismatch (GETP on a DATA file, GETD/EMREG/SAVERX on a
    /// PROGRAM file). Matches HP-41CX QRG p.39 "FL TYPE ERR".
    #[error("fl type err")]
    FileType,
    /// X-MEM: insufficient extended memory (SAVEP/SAVED when file would not fit
    /// in 600-register capacity). Matches HP-41CX QRG p.39 "NO ROOM".
    #[error("no room")]
    NoRoom,
    /// Printer not present: PRX/PRA/PRSTK executed when neither flag 55
    /// (Printer Existence) nor flag 21 (Printer Enable) is set. Matches the
    /// HP-41C OM p.57-58 NONEXISTENT error-message class: "An attempt was made
    /// to execute a specific print function when the printer was not connected
    /// to the system." (UNC-02, Phase 66).
    #[error("nonexistent")]
    NonExistent,
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

    // Catches: FileNotFound Display regression (HP-41CX QRG p.39 "FL NOT FOUND")
    #[test]
    fn file_not_found_display() {
        assert_eq!(HpError::FileNotFound.to_string(), "fl not found");
    }

    // Catches: FileType Display regression (HP-41CX QRG p.39 "FL TYPE ERR")
    #[test]
    fn file_type_display() {
        assert_eq!(HpError::FileType.to_string(), "fl type err");
    }

    // Catches: NoRoom Display regression (HP-41CX QRG p.39 "NO ROOM")
    #[test]
    fn no_room_display() {
        assert_eq!(HpError::NoRoom.to_string(), "no room");
    }

    // Catches: NonExistent Display regression (HP-41C OM p.57-58 NONEXISTENT
    // error class — printer not connected; UNC-02, Phase 66)
    #[test]
    fn non_existent_display() {
        assert_eq!(HpError::NonExistent.to_string(), "nonexistent");
    }

    // Catches: NonExistent variant conflation with InvalidOp or other errors
    #[test]
    fn non_existent_distinct_from_invalid_op() {
        assert_ne!(HpError::NonExistent, HpError::InvalidOp);
    }
}
