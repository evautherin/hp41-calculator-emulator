//! hp41 — HP-41 calculator emulator CLI
//!
//! Entry point: parses CLI args, loads state, initialises ratatui terminal,
//! runs App event loop, then saves state and restores terminal.

#![deny(clippy::unwrap_used)]

mod app;
pub mod cards;
mod help_data;
mod keys;
mod persistence;
mod prgm_display;
mod programs;
mod ui;

#[cfg(test)]
mod tests;

use app::App;
use clap::Parser;
use hp41_core::CalcState;

/// HP-41 Calculator Emulator — faithful HP-41C/CV/CX behavioral emulation in the terminal.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the state file (JSON). Loaded on startup, saved on exit and every 30s.
    /// Default: ~/.hp41/autosave.json
    #[arg(long, value_name = "FILE")]
    state_file: Option<std::path::PathBuf>,

    /// Run all startup initialization (state load, core init) then exit — no TUI required.
    /// Used by `just bench-startup` to measure cold-start time without a real terminal.
    #[arg(long, hide = true)]
    bench_startup: bool,

    /// Append all PRX/PRA/PRSTK output to this file (created if absent, appended if exists).
    #[arg(long, value_name = "FILE")]
    print_log: Option<std::path::PathBuf>,

    /// Import a .raw program file into calculator memory. Use '-' for stdin.
    #[arg(long, value_name = "FILE")]
    import_raw: Option<String>,

    /// Export current program to .raw format. Use '-' for stdout.
    #[arg(long, value_name = "FILE")]
    export_raw: Option<String>,

    /// Import a .card.json data card into calculator registers. Use '-' for stdin.
    #[arg(long, value_name = "FILE")]
    import_data: Option<String>,

    /// Export current data registers to .card.json format. Use '-' for stdout.
    #[arg(long, value_name = "FILE")]
    export_data: Option<String>,

    /// Suppress interactive TUI — exit after import/export completes. Enables scripting pipelines.
    #[arg(long)]
    batch: bool,
}

/// Read all bytes from `path` (a file path or `"-"` for stdin).
///
/// When `path == "-"`, reads from `std::io::stdin()`.
/// Otherwise reads the file at the given path.
fn read_file_or_stdin(path: &str) -> std::io::Result<Vec<u8>> {
    if path == "-" {
        use std::io::Read;
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf)?;
        Ok(buf)
    } else {
        std::fs::read(path)
    }
}

/// Write `bytes` to `path` (a file path or `"-"` for stdout).
///
/// When `path == "-"`, writes to `std::io::stdout()`.
/// Otherwise writes to the file at the given path.
fn write_file_or_stdout(path: &str, bytes: &[u8]) -> std::io::Result<()> {
    if path == "-" {
        use std::io::Write;
        std::io::stdout().write_all(bytes)
    } else {
        std::fs::write(path, bytes)
    }
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    // Resolve the active state file path (D-02: CLI override or default).
    let state_path = cli
        .state_file
        .unwrap_or_else(persistence::default_state_path);

    // D-03: load existing state or start fresh; NEVER panic on parse failure.
    let (mut initial_state, load_message) = match persistence::load_state(&state_path) {
        Ok(state) => (state, None),
        Err(e) if state_path.exists() => {
            // File exists but is corrupt — warn and start fresh.
            let msg = format!("State load failed ({e}); starting fresh");
            (CalcState::new(), Some(msg))
        }
        Err(_) => {
            // File missing — normal first-run case; no message needed.
            (CalcState::new(), None)
        }
    };

    // ── Import/export processing (BEFORE bench_startup and TUI init) ───────────

    // --import-raw: decode and load program(s) into initial_state
    if let Some(ref raw_path) = cli.import_raw {
        let bytes = read_file_or_stdin(raw_path)?;
        let programs = hp41_core::cardreader::raw::decode_all_programs(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        if programs.is_empty() {
            eprintln!("No programs found in file");
            return Ok(());
        }
        for (i, program) in programs.iter().enumerate() {
            let label =
                hp41_core::cardreader::raw::picker_label(i, &program.ops, program.byte_len);
            let n = program.ops.len();
            hp41_core::cardreader::insert_program_ops(&mut initial_state, program.ops.clone());
            eprintln!("Imported {} ({} steps)", label, n);
        }
    }

    // --export-raw: encode current program and write to file/stdout
    if let Some(ref raw_path) = cli.export_raw {
        let bytes = hp41_core::cardreader::raw::encode_program(&initial_state.program)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        let n = bytes.len();
        write_file_or_stdout(raw_path, &bytes)?;
        if raw_path == "-" {
            eprintln!("Saved {} bytes to stdout", n);
        } else {
            eprintln!("Saved {} bytes to {}", n, raw_path);
        }
    }

    // --import-data: decode and load data card into initial_state
    if let Some(ref data_path) = cli.import_data {
        let bytes = read_file_or_stdin(data_path)?;
        let card = hp41_core::cardreader::decode_data(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        hp41_core::cardreader::load_data_card(&mut initial_state, card);
        eprintln!("Loaded data card");
    }

    // --export-data: capture data card and write to file/stdout
    if let Some(ref data_path) = cli.export_data {
        let card = hp41_core::cardreader::capture_data_card(&initial_state);
        let bytes = hp41_core::cardreader::encode_data(&card)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        write_file_or_stdout(data_path, &bytes)?;
        eprintln!("Saved data card");
    }

    // ── Early-exit gates (bench_startup and --batch) ──────────────────────────
    if cli.bench_startup || cli.batch {
        return Ok(());
    }

    let terminal = ratatui::init();

    let mut app = App::new(initial_state, state_path, cli.print_log);
    if let Some(msg) = load_message {
        app.message = Some(msg);
    }

    let result = app.run(terminal);

    ratatui::restore();

    result
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod main_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn read_file_or_stdin_reads_from_file() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("test.bin");
        let content = b"hello raw bytes";
        std::fs::write(&path, content).unwrap();
        let result = read_file_or_stdin(path.to_str().unwrap()).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn write_file_or_stdout_writes_to_file() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("out.bin");
        let content = b"output bytes";
        write_file_or_stdout(path.to_str().unwrap(), content).unwrap();
        let actual = std::fs::read(&path).unwrap();
        assert_eq!(actual, content);
    }

    #[test]
    fn read_file_or_stdin_missing_file_returns_error() {
        let result = read_file_or_stdin("/nonexistent/path/that/does/not/exist.raw");
        assert!(result.is_err(), "reading a missing file must return Err");
    }

    #[test]
    fn write_file_or_stdout_bad_dir_returns_error() {
        let result = write_file_or_stdout(
            "/nonexistent/dir/that/does/not/exist/out.raw",
            b"content",
        );
        assert!(result.is_err(), "writing to a missing dir must return Err");
    }

    #[test]
    fn cli_has_all_required_fields() {
        // Smoke-test that the Cli struct has all 5 new fields by parsing --help
        // output via clap's built-in test helpers. We exercise clap's compile-
        // time checks implicitly by referencing the fields.
        let cli = Cli {
            state_file: None,
            bench_startup: false,
            print_log: None,
            import_raw: Some("input.raw".to_string()),
            export_raw: Some("output.raw".to_string()),
            import_data: Some("data.card.json".to_string()),
            export_data: Some("out.card.json".to_string()),
            batch: true,
        };
        assert_eq!(cli.import_raw.as_deref(), Some("input.raw"));
        assert_eq!(cli.export_raw.as_deref(), Some("output.raw"));
        assert_eq!(cli.import_data.as_deref(), Some("data.card.json"));
        assert_eq!(cli.export_data.as_deref(), Some("out.card.json"));
        assert!(cli.batch);
    }
}
