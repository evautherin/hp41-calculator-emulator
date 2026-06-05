//! Pool I/O: load and save JSON help pools with minimal-diff writeback (D-60.4).
//!
//! Key properties:
//! - `serde_json` `preserve_order` feature keeps IndexMap insertion order across roundtrips.
//! - `save_pool` uses a custom UTF-8 formatter: 4-space indent, no \uXXXX escaping for non-ASCII.
//! - Trailing newline is always appended (all existing pools end with \n).
//!
//! These invariants are tested by `pool::tests::{roundtrip_identity, key_order_preserved, utf8_unescaped}`.

use serde_json::ser::{Formatter, PrettyFormatter};
use serde_json::Value;
use std::io;

// ---------------------------------------------------------------------------
// Custom Formatter: 4-space indent + literal UTF-8 (no \uXXXX escaping)
// ---------------------------------------------------------------------------

/// Wraps `PrettyFormatter` for structural formatting and overrides `write_string_fragment`
/// to emit non-ASCII bytes literally, preventing the default `\uXXXX` escaping.
///
/// All structural formatting (object/array braces, commas, colons, whitespace) is
/// delegated to the inner `PrettyFormatter`.
pub struct Utf8PrettyFormatter<'a> {
    inner: PrettyFormatter<'a>,
}

impl<'a> Utf8PrettyFormatter<'a> {
    /// Create a new formatter with 4-space indent (matching the pool file format).
    pub fn new() -> Self {
        Self {
            inner: PrettyFormatter::with_indent(b"    "),
        }
    }
}

impl<'a> Formatter for Utf8PrettyFormatter<'a> {
    // ── Structural methods delegated to PrettyFormatter ──────────────────────

    fn begin_array<W: io::Write + ?Sized>(&mut self, w: &mut W) -> io::Result<()> {
        self.inner.begin_array(w)
    }
    fn end_array<W: io::Write + ?Sized>(&mut self, w: &mut W) -> io::Result<()> {
        self.inner.end_array(w)
    }
    fn begin_array_value<W: io::Write + ?Sized>(&mut self, w: &mut W, first: bool) -> io::Result<()> {
        self.inner.begin_array_value(w, first)
    }
    fn end_array_value<W: io::Write + ?Sized>(&mut self, w: &mut W) -> io::Result<()> {
        self.inner.end_array_value(w)
    }
    fn begin_object<W: io::Write + ?Sized>(&mut self, w: &mut W) -> io::Result<()> {
        self.inner.begin_object(w)
    }
    fn end_object<W: io::Write + ?Sized>(&mut self, w: &mut W) -> io::Result<()> {
        self.inner.end_object(w)
    }
    fn begin_object_key<W: io::Write + ?Sized>(&mut self, w: &mut W, first: bool) -> io::Result<()> {
        self.inner.begin_object_key(w, first)
    }
    fn end_object_key<W: io::Write + ?Sized>(&mut self, w: &mut W) -> io::Result<()> {
        self.inner.end_object_key(w)
    }
    fn begin_object_value<W: io::Write + ?Sized>(&mut self, w: &mut W) -> io::Result<()> {
        self.inner.begin_object_value(w)
    }
    fn end_object_value<W: io::Write + ?Sized>(&mut self, w: &mut W) -> io::Result<()> {
        self.inner.end_object_value(w)
    }

    // ── Override: emit non-ASCII string bytes literally (no \uXXXX) ──────────

    /// Write string content without escaping non-ASCII characters.
    ///
    /// The default serde_json implementation escapes all codepoints > 127 as `\uXXXX`.
    /// This override writes the raw UTF-8 bytes directly, so `Σ`, `—`, `ä` etc.
    /// survive a load → save roundtrip un-escaped (D-60.4 / Pitfall 2).
    fn write_string_fragment<W: io::Write + ?Sized>(
        &mut self,
        w: &mut W,
        fragment: &str,
    ) -> io::Result<()> {
        w.write_all(fragment.as_bytes())
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Load a JSON help pool from disk into a `Vec<Value>`.
///
/// The `preserve_order` feature ensures `Value::Object` is backed by `IndexMap`,
/// maintaining insertion order across a load→save roundtrip (D-60.4 / Pitfall 1).
///
/// Panics on I/O or parse error (these are programming errors in a dev-only tool).
pub fn load_pool(path: &str) -> Vec<Value> {
    let json =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("parse {path}: {e}"))
}

/// Save a JSON help pool back to disk using the custom UTF-8 formatter.
///
/// Guarantees:
/// - 4-space indent (pool format: outer items 4 sp, properties 8 sp = 2 levels).
/// - Non-ASCII bytes written literally (no `\uXXXX` escaping).
/// - A single trailing newline is always present.
///
/// Panics on serialization or I/O error.
pub fn save_pool(path: &str, entries: &[Value]) {
    let mut buf: Vec<u8> = Vec::new();
    let mut ser =
        serde_json::Serializer::with_formatter(&mut buf, Utf8PrettyFormatter::new());
    serde::Serialize::serialize(entries, &mut ser)
        .unwrap_or_else(|e| panic!("serialize {path}: {e}"));

    // Ensure a single trailing newline (all existing pools end with \n).
    if !buf.ends_with(b"\n") {
        buf.push(b'\n');
    }

    std::fs::write(path, &buf).unwrap_or_else(|e| panic!("write {path}: {e}"));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Minimal fixture: two entries with NO search_aliases.
    /// Used to verify that load → save produces byte-identical output (D-60.4).
    const FIXTURE_NO_ALIASES: &str = r#"[
    {
        "op_variant": "TestOp",
        "display_name": "TEST",
        "category": "Test",
        "status": "implemented",
        "phase": "1",
        "key_path": null,
        "description": "A test operation"
    },
    {
        "op_variant": "TestOp2",
        "display_name": "TEST2",
        "category": "Test",
        "status": "implemented",
        "phase": "1",
        "key_path": null,
        "description": "A second test operation"
    }
]
"#;

    /// Fixture with a non-ASCII character (Sigma), em-dash, and umlaut.
    const FIXTURE_UTF8: &str = r#"[
    {
        "op_variant": "SumOp",
        "display_name": "CLΣ",
        "category": "Statistics",
        "status": "implemented",
        "phase": "1",
        "key_path": null,
        "description": "Clear Σ registers — enter clock mode … and Ä umlaut"
    }
]
"#;

    /// Fixture with a pre-populated `search_aliases` entry (to verify key order after insertion).
    const FIXTURE_FOR_ORDER: &str = r#"[
    {
        "op_variant": "TestOp",
        "display_name": "TEST",
        "category": "Test",
        "status": "implemented",
        "phase": "1",
        "key_path": null,
        "description": "A test operation",
        "notes": "Some notes here"
    }
]
"#;

    /// A load → save with NO aliases added must produce byte-identical output (D-60.4).
    ///
    /// Proves that `preserve_order` + `Utf8PrettyFormatter` roundtrips a pool without noise.
    #[test]
    fn roundtrip_identity() {
        let entries: Vec<Value> = serde_json::from_str(FIXTURE_NO_ALIASES).unwrap();

        let mut buf: Vec<u8> = Vec::new();
        let mut ser =
            serde_json::Serializer::with_formatter(&mut buf, Utf8PrettyFormatter::new());
        serde::Serialize::serialize(&entries, &mut ser).unwrap();
        if !buf.ends_with(b"\n") {
            buf.push(b'\n');
        }

        let result = String::from_utf8(buf).unwrap();
        assert_eq!(
            result, FIXTURE_NO_ALIASES,
            "roundtrip produced non-identical output"
        );
    }

    /// After inserting a `search_aliases` key, it must appear LAST in the serialized object
    /// (after all pre-existing keys like `notes`), and all other keys must be in their
    /// original positions (preserve_order guarantee).
    #[test]
    fn key_order_preserved() {
        let mut entries: Vec<Value> = serde_json::from_str(FIXTURE_FOR_ORDER).unwrap();

        // Insert search_aliases on the first entry.
        if let Value::Object(ref mut map) = entries[0] {
            map.insert(
                "search_aliases".to_string(),
                Value::Array(vec![
                    Value::String("test alias".to_string()),
                    Value::String("another alias".to_string()),
                ]),
            );
        }

        let mut buf: Vec<u8> = Vec::new();
        let mut ser =
            serde_json::Serializer::with_formatter(&mut buf, Utf8PrettyFormatter::new());
        serde::Serialize::serialize(&entries, &mut ser).unwrap();
        let result = String::from_utf8(buf).unwrap();

        // Verify all expected keys appear in the correct order.
        let op_pos = result.find("\"op_variant\"").unwrap();
        let display_pos = result.find("\"display_name\"").unwrap();
        let notes_pos = result.find("\"notes\"").unwrap();
        let aliases_pos = result.find("\"search_aliases\"").unwrap();

        assert!(op_pos < display_pos, "op_variant must come before display_name");
        assert!(notes_pos < aliases_pos, "notes must come before search_aliases");

        // Also verify search_aliases is actually LAST by checking nothing after it
        // (except whitespace, closing braces, and the outer array structure).
        let after_aliases = &result[aliases_pos..];
        assert!(
            !after_aliases.contains("\"op_variant\""),
            "op_variant must not appear after search_aliases"
        );
        assert!(
            !after_aliases.contains("\"notes\""),
            "notes must not appear after search_aliases"
        );
    }

    /// Non-ASCII characters (Σ, —, …, Ä) must survive load → save without `\uXXXX` escaping.
    ///
    /// This guards Pitfall 2: `serde_json` default escapes all codepoints > 127.
    #[test]
    fn utf8_unescaped() {
        let entries: Vec<Value> = serde_json::from_str(FIXTURE_UTF8).unwrap();

        let mut buf: Vec<u8> = Vec::new();
        let mut ser =
            serde_json::Serializer::with_formatter(&mut buf, Utf8PrettyFormatter::new());
        serde::Serialize::serialize(&entries, &mut ser).unwrap();
        let result = String::from_utf8(buf).unwrap();

        // Verify the literal characters are present (not escaped).
        assert!(result.contains('Σ'), "Σ must be present as a literal character");
        assert!(result.contains('—'), "— must be present as a literal character");
        assert!(result.contains('…'), "… must be present as a literal character");
        assert!(result.contains('Ä'), "Ä must be present as a literal character");

        // Verify no \uXXXX escape sequences for our characters.
        assert!(
            !result.contains("\\u03a3"),
            "Σ must not be escaped as \\u03a3"
        );
        assert!(
            !result.contains("\\u2014"),
            "— must not be escaped as \\u2014"
        );
        assert!(
            !result.contains("\\u2026"),
            "… must not be escaped as \\u2026"
        );
        assert!(
            !result.contains("\\u00c4"),
            "Ä must not be escaped as \\u00c4"
        );
    }
}
