//! Integration tests for multi-program `.raw` archive decoding and XROM round-trip.
//!
//! Covers the full `decode_all_programs` pipeline, including:
//! - Single-program files (Vec of length 1)
//! - Multi-program concatenation (2 and 3 programs)
//! - Edge cases (empty input, truncated stream)
//! - XROM `Op::SyntheticByte` round-trip (RAW-05)
//! - `picker_label` formatting per D-50.5

#[allow(clippy::unwrap_used)]
use hp41_core::cardreader::raw::{decode_all_programs, encode_program, picker_label};
use hp41_core::error::HpError;
use hp41_core::ops::Op;

#[test]
#[allow(clippy::unwrap_used)]
fn decode_all_single_program_returns_vec_of_one() {
    let bytes = encode_program(&[Op::Add, Op::Sub]).unwrap();
    let programs = decode_all_programs(&bytes).unwrap();
    assert_eq!(
        programs.len(),
        1,
        "single-program stream must yield Vec of length 1"
    );
    assert_eq!(
        programs[0].ops,
        vec![Op::Add, Op::Sub],
        "ops must match encoded ops"
    );
    assert_eq!(
        programs[0].byte_len,
        bytes.len(),
        "byte_len must equal the encoded byte count"
    );
}

#[test]
#[allow(clippy::unwrap_used)]
fn decode_all_two_programs_yields_correct_segments() {
    let bytes1 = encode_program(&[Op::Add]).unwrap();
    let bytes2 = encode_program(&[Op::Sub, Op::Mul]).unwrap();
    let mut combined = bytes1.clone();
    combined.extend_from_slice(&bytes2);

    let programs = decode_all_programs(&combined).unwrap();
    assert_eq!(
        programs.len(),
        2,
        "two concatenated programs must yield Vec of length 2"
    );

    assert_eq!(programs[0].ops, vec![Op::Add]);
    assert_eq!(
        programs[0].byte_len,
        bytes1.len(),
        "first segment byte_len must equal first program length"
    );

    assert_eq!(programs[1].ops, vec![Op::Sub, Op::Mul]);
    assert_eq!(
        programs[1].byte_len,
        bytes2.len(),
        "second segment byte_len must equal second program length"
    );
}

#[test]
#[allow(clippy::unwrap_used)]
fn decode_all_three_programs_yields_vec_of_three() {
    let bytes1 = encode_program(&[Op::Add]).unwrap();
    let bytes2 = encode_program(&[Op::Sub]).unwrap();
    let bytes3 = encode_program(&[Op::Mul, Op::Div]).unwrap();
    let mut combined = bytes1.clone();
    combined.extend_from_slice(&bytes2);
    combined.extend_from_slice(&bytes3);

    let programs = decode_all_programs(&combined).unwrap();
    assert_eq!(
        programs.len(),
        3,
        "three concatenated programs must yield Vec of length 3"
    );
    assert_eq!(programs[0].ops, vec![Op::Add]);
    assert_eq!(programs[1].ops, vec![Op::Sub]);
    assert_eq!(programs[2].ops, vec![Op::Mul, Op::Div]);
}

#[test]
#[allow(clippy::unwrap_used)]
fn decode_all_empty_input_returns_empty_vec() {
    let programs = decode_all_programs(&[]).unwrap();
    assert!(
        programs.is_empty(),
        "empty input must yield empty Vec, not an error"
    );
}

#[test]
#[allow(clippy::unwrap_used)]
fn decode_all_truncated_no_end_marker_returns_error() {
    // Two raw bytes with no END marker — truncated stream.
    let bytes = vec![0x40, 0x41];
    let err = decode_all_programs(&bytes).unwrap_err();
    assert!(
        matches!(&err, HpError::CardData(msg) if msg.contains("END")),
        "expected END-marker diagnostic, got: {err:?}"
    );
}

#[test]
#[allow(clippy::unwrap_used)]
fn xrom_synthetic_byte_round_trip_through_decode_all() {
    // RAW-05: XROM bytes survive encode + decode_all_programs.
    // 0xAA is outside the reserved-prefix set → stored as Op::SyntheticByte(0xAA).
    let original_ops = vec![
        Op::Lbl("XTEST".to_string()),
        Op::SyntheticByte(0xAA),
        Op::Rtn,
    ];
    let encoded = encode_program(&original_ops).unwrap();

    let programs = decode_all_programs(&encoded).unwrap();
    assert_eq!(programs.len(), 1);
    assert_eq!(
        programs[0].ops, original_ops,
        "Op::SyntheticByte(0xAA) must survive the encode + decode_all_programs round-trip"
    );
}

#[test]
#[allow(clippy::unwrap_used)]
fn picker_label_with_lbl_op_returns_name_and_byte_count() {
    // D-50.5: picker_label must extract the first LBL name.
    let ops = vec![Op::Lbl("QUAD".to_string()), Op::Add, Op::Rtn];
    let label = picker_label(0, &ops, 47);
    assert!(
        label.contains("QUAD"),
        "picker_label must contain the LBL name; got: {label:?}"
    );
    assert!(
        label.contains("47"),
        "picker_label must contain byte count 47; got: {label:?}"
    );
    assert!(
        label.contains("bytes"),
        "picker_label must contain 'bytes'; got: {label:?}"
    );
}

#[test]
#[allow(clippy::unwrap_used)]
fn picker_label_without_lbl_op_returns_program_n_and_byte_count() {
    // D-50.5: fallback when no LBL op is present.
    let ops = vec![Op::Add, Op::Rtn];
    let label = picker_label(0, &ops, 23);
    assert!(
        label.contains("Program 1"),
        "picker_label must fall back to 'Program 1' when no LBL; got: {label:?}"
    );
    assert!(
        label.contains("23"),
        "picker_label must contain byte count 23; got: {label:?}"
    );
    assert!(
        label.contains("bytes"),
        "picker_label must contain 'bytes'; got: {label:?}"
    );
}
