//! Integration tests for the hp41 CLI import/export flags.
//!
//! Tests the --import-raw, --export-raw, --import-data, --export-data, and --batch
//! flags via std::process::Command (subprocess execution).
//!
//! These tests require the binary to be built. Run with:
//!   cargo test -p hp41-cli --test cli_import_export
//!
//! The round-trip and stdout tests are marked #[ignore] because they invoke the
//! binary as a subprocess and are slower than unit tests. Run them explicitly:
//!   cargo test -p hp41-cli --test cli_import_export -- --ignored

#![allow(clippy::unwrap_used)]

use hp41_core::cardreader::raw::{decode_all_programs, encode_program};
use hp41_core::ops::Op;
use std::process::Command;
use tempfile::tempdir;

/// Helper: get path to the compiled hp41 binary.
fn hp41_bin() -> std::path::PathBuf {
    // CARGO_BIN_EXE_hp41 is set by cargo test for bins declared in Cargo.toml.
    // Falls back to a relative path for environments where it may not be set.
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_hp41") {
        return std::path::PathBuf::from(p);
    }
    // Fallback: look in the cargo target directory.
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../target/debug/hp41");
    path
}

/// Test: --batch exits 0 without launching the TUI or hanging.
#[test]
fn batch_exits_zero_without_tui() {
    let status = Command::new(hp41_bin())
        .args(["--batch"])
        .status()
        .expect("hp41 binary must be executable");
    assert!(status.success(), "--batch must exit with code 0");
}

/// Test: --import-raw FILE --export-raw OUTFILE --batch round-trips a .raw file.
#[test]
#[ignore = "slow subprocess test — run with: cargo test -p hp41-cli --test cli_import_export -- --ignored"]
fn round_trip_import_export_raw() {
    let tmp = tempdir().unwrap();
    let input_path = tmp.path().join("input.raw");
    let output_path = tmp.path().join("output.raw");

    // Encode a small program to the input file.
    let original_ops = vec![Op::Add, Op::Sub];
    let input_bytes = encode_program(&original_ops).unwrap();
    std::fs::write(&input_path, &input_bytes).unwrap();

    // Run: hp41 --import-raw input.raw --export-raw output.raw --batch
    let status = Command::new(hp41_bin())
        .args([
            "--import-raw",
            input_path.to_str().unwrap(),
            "--export-raw",
            output_path.to_str().unwrap(),
            "--batch",
        ])
        .status()
        .expect("hp41 binary must be executable");
    assert!(
        status.success(),
        "hp41 --import-raw --export-raw --batch must exit 0"
    );

    // Read the output and decode — must match original ops.
    let output_bytes = std::fs::read(&output_path).unwrap();
    let programs = decode_all_programs(&output_bytes).unwrap();
    assert_eq!(
        programs.len(),
        1,
        "output .raw must contain exactly 1 program"
    );
    assert_eq!(
        programs[0].ops, original_ops,
        "round-tripped ops must match original"
    );
}

/// Test: --import-raw FILE --export-raw - --batch writes encoded bytes to stdout.
#[test]
#[ignore = "slow subprocess test — run with: cargo test -p hp41-cli --test cli_import_export -- --ignored"]
fn export_raw_to_stdout() {
    let tmp = tempdir().unwrap();
    let input_path = tmp.path().join("input.raw");

    // Encode a single-op program.
    let original_ops = vec![Op::Add];
    let input_bytes = encode_program(&original_ops).unwrap();
    std::fs::write(&input_path, &input_bytes).unwrap();

    // Run: hp41 --import-raw input.raw --export-raw - --batch
    // Capture stdout.
    let output = Command::new(hp41_bin())
        .args([
            "--import-raw",
            input_path.to_str().unwrap(),
            "--export-raw",
            "-",
            "--batch",
        ])
        .output()
        .expect("hp41 binary must be executable");
    assert!(output.status.success(), "--export-raw - must exit 0");

    // Stdout bytes must decode back to the original ops.
    let programs = decode_all_programs(&output.stdout).unwrap();
    assert_eq!(programs.len(), 1, "stdout must contain exactly 1 program");
    assert_eq!(
        programs[0].ops, original_ops,
        "stdout-exported ops must match original"
    );
}

/// Test: multi-program .raw archive via --import-raw imports ALL programs.
#[test]
#[ignore = "slow subprocess test — run with: cargo test -p hp41-cli --test cli_import_export -- --ignored"]
fn import_raw_multi_program_loads_all() {
    let tmp = tempdir().unwrap();
    let input_path = tmp.path().join("multi.raw");
    let output_path = tmp.path().join("re-exported.raw");

    // Create a 2-program archive.
    let ops1 = vec![Op::Add, Op::Rtn];
    let ops2 = vec![Op::Sub, Op::Mul, Op::Rtn];
    let mut multi_bytes = encode_program(&ops1).unwrap();
    multi_bytes.extend_from_slice(&encode_program(&ops2).unwrap());
    std::fs::write(&input_path, &multi_bytes).unwrap();

    // Import the 2-program archive, then export. The CLI imports ALL programs
    // sequentially via insert_program_ops — they will be concatenated in
    // initial_state.program. Export produces a single .raw with all ops.
    let status = Command::new(hp41_bin())
        .args([
            "--import-raw",
            input_path.to_str().unwrap(),
            "--export-raw",
            output_path.to_str().unwrap(),
            "--batch",
        ])
        .status()
        .expect("hp41 binary must be executable");
    assert!(status.success(), "multi-program import must exit 0");

    // The exported file must decode back to a merged program containing all ops.
    let output_bytes = std::fs::read(&output_path).unwrap();
    let programs = decode_all_programs(&output_bytes).unwrap();
    assert_eq!(
        programs.len(),
        1,
        "exported file is a single merged program"
    );
    // All ops from both programs must be present (insert_program_ops inserts program 2
    // after state.pc=0, so order is: ops1[0], then ops2..., then ops1[1..]).
    // We verify all ops are present rather than checking exact order.
    let merged = &programs[0].ops;
    for op in ops1.iter().chain(ops2.iter()) {
        assert!(
            merged.contains(op),
            "merged program must contain op {op:?} from the imported programs"
        );
    }
    assert_eq!(
        merged.len(),
        ops1.len() + ops2.len(),
        "merged program must contain all ops from both imported programs (total count)"
    );
}

/// Test: --import-data and --export-data round-trip a .card.json file.
#[test]
#[ignore = "slow subprocess test — run with: cargo test -p hp41-cli --test cli_import_export -- --ignored"]
fn round_trip_import_export_data() {
    use hp41_core::cardreader::{capture_data_card, encode_data};
    use hp41_core::num::HpNum;
    use hp41_core::state::CalcState;

    let tmp = tempdir().unwrap();
    let input_path = tmp.path().join("data.card.json");
    let output_path = tmp.path().join("data_out.card.json");

    // Create a data card with some non-zero registers.
    let mut state = CalcState::new();
    state.regs[0] = HpNum::from(42i32).into();
    state.regs[7] = HpNum::from(-3i32).into();
    let card = capture_data_card(&state);
    let card_bytes = encode_data(&card).unwrap();
    std::fs::write(&input_path, &card_bytes).unwrap();

    // Run: hp41 --import-data input.card.json --export-data output.card.json --batch
    let status = Command::new(hp41_bin())
        .args([
            "--import-data",
            input_path.to_str().unwrap(),
            "--export-data",
            output_path.to_str().unwrap(),
            "--batch",
        ])
        .status()
        .expect("hp41 binary must be executable");
    assert!(
        status.success(),
        "--import-data --export-data --batch must exit 0"
    );

    // Decoded output must have the same register values.
    use hp41_core::cardreader::decode_data;
    let out_bytes = std::fs::read(&output_path).unwrap();
    let out_card = decode_data(&out_bytes).unwrap();
    assert_eq!(
        out_card.registers[0],
        hp41_core::HpValue::from(42i32),
        "R00 must round-trip through --export-data"
    );
    assert_eq!(
        out_card.registers[7],
        hp41_core::HpValue::from(-3i32),
        "R07 must round-trip through --export-data"
    );
}
