//! Phase 65 — Option A: HP-41-authentic 12-cell LCD rendering for large-exponent
//! values (ADR v4.3-006).
//!
//! The GUI 14-segment display (`Display14Seg.tsx`) has exactly **12 character
//! cells**. The canonical scientific format `format_hpnum` emits an "E" plus a
//! space (e.g. `1.088886945E 28` = 14 cells), which overflowed the grid and
//! truncated the exponent — exactly the values Phase 65-01 made reachable
//! (`FACT(27..=69)`). `format_hpnum_lcd` drops the "E" and right-justifies the
//! exponent so the full value fits 12 cells: positive exponent keeps the full
//! 10-significant-digit mantissa; a negative exponent reserves one mantissa cell
//! for the sign. The wide CLI / stack-panel displays keep the "E" form.

use hp41_core::{format_hpnum, format_hpnum_lcd, DisplayMode, HpNum};

/// Count display cells the way the GUI `Display14Seg` component does: the decimal
/// point folds into the preceding digit and does NOT consume a cell.
fn cell_count(s: &str) -> usize {
    s.chars().filter(|&c| c != '.').count()
}

#[test]
fn fact27_large_exp_fits_exactly_12_cells() {
    // 27! = 1.088886945E28 — the value the UAT screenshot showed truncated.
    let n = HpNum::from_f64(1.088_886_945e28).expect("FACT(27) magnitude is representable");
    let s = format_hpnum_lcd(&n, &DisplayMode::Fix(4));
    assert_eq!(
        cell_count(&s),
        12,
        "expected 12 cells, got [{s}] ({} cells)",
        cell_count(&s)
    );
    assert_eq!(s, "1.08888694528");
}

#[test]
fn fact69_large_exp_fits_exactly_12_cells() {
    // 69! = 1.711224524E98 — matches the HP-41 Owner's Manual factorial table.
    let n = HpNum::from_f64(1.711_224_524e98).expect("FACT(69) magnitude is representable");
    let s = format_hpnum_lcd(&n, &DisplayMode::Fix(4));
    assert_eq!(
        cell_count(&s),
        12,
        "expected 12 cells, got [{s}] ({} cells)",
        cell_count(&s)
    );
    assert_eq!(s, "1.71122452498");
}

#[test]
fn large_exp_output_has_no_e_character() {
    // Authentic HP-41 LCD shows scientific notation WITHOUT an 'E'.
    let n = HpNum::from_f64(1.088_886_945e28).expect("representable");
    let s = format_hpnum_lcd(&n, &DisplayMode::Fix(4));
    assert!(!s.contains('E'), "authentic HP-41 LCD has no 'E': [{s}]");
}

#[test]
fn large_exp_exponent_is_right_justified() {
    let n = HpNum::from_f64(1.088_886_945e28).expect("representable");
    let s = format_hpnum_lcd(&n, &DisplayMode::Fix(4));
    assert!(
        s.ends_with("28"),
        "exponent right-justified at the end: [{s}]"
    );
}

#[test]
fn negative_exponent_reserves_sign_cell_and_fits_12() {
    // SCI 9 forces a 14-cell "E" string for a small value; the LCD form must drop
    // the "E", keep the exponent sign, and fit 12 cells (mantissa loses one digit
    // to make room for the sign).
    let n = HpNum::from_f64(1.23e-5).expect("representable");
    let s = format_hpnum_lcd(&n, &DisplayMode::Sci(9));
    assert_eq!(
        cell_count(&s),
        12,
        "expected 12 cells, got [{s}] ({} cells)",
        cell_count(&s)
    );
    assert!(!s.contains('E'), "no 'E': [{s}]");
    assert!(s.ends_with("-05"), "signed exponent right-justified: [{s}]");
}

#[test]
fn small_sci_value_within_12_cells_keeps_e_form() {
    // SCI 4 of a small value already fits 12 cells → returned unchanged (the GUI
    // keeps the existing "E" rendering for non-overflowing scientific output).
    let n = HpNum::from_f64(5.0).expect("representable");
    let s = format_hpnum_lcd(&n, &DisplayMode::Sci(4));
    assert!(cell_count(&s) <= 12, "got [{s}]");
    assert_eq!(s, format_hpnum(&n, &DisplayMode::Sci(4)));
}

#[test]
fn in_range_value_delegates_to_format_hpnum() {
    // exponent == 0 (common case) → identical to the normal formatter; the LCD
    // formatter only special-cases large-exponent values.
    let n = HpNum::from_f64(42.0).expect("representable");
    assert_eq!(
        format_hpnum_lcd(&n, &DisplayMode::Fix(4)),
        format_hpnum(&n, &DisplayMode::Fix(4)),
    );
}
