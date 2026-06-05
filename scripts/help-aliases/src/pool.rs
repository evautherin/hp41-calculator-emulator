//! Pool I/O: load JSON help pools and write back via a byte-preserving text splice (D-60.4).
//!
//! Key properties:
//! - `load_pool` parses with the `preserve_order` feature so callers see entries in file order.
//! - `splice_aliases` is the writeback: it inserts ONLY a `search_aliases` array before each
//!   target entry's closing brace, leaving every other byte of the file untouched — key order,
//!   inline `xrom`/`divergences` formatting, non-ASCII characters (Σ, —, …, umlauts) and
//!   whitespace are all preserved exactly.
//!
//! Why a text splice and not `serde_json` re-serialization: the pools hand-author compact,
//! single-line sub-structures (e.g. `"xrom": { "module": "Time", "module_id": 26, ... }` and
//! `"divergences": ["…"]`). `serde_json`'s `PrettyFormatter` always expands nested objects and
//! arrays to multi-line form, which would churn ~227 entries' formatting on a no-op write and
//! break the D-60.4 alias-only-diff guarantee. Re-serialization was the original 60-01 approach;
//! a Phase-60 Wave-2 data run exposed the churn and it was replaced by this splice.
//!
//! These invariants are tested by `pool::tests::{splice_preserves_inline_xrom,
//! splice_noop_when_absent, splice_skips_populated, splice_utf8_literal, splice_appends_last}`.

use serde_json::Value;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Load
// ---------------------------------------------------------------------------

/// Load a JSON help pool from disk into a `Vec<Value>`.
///
/// The `preserve_order` feature ensures `Value::Object` is backed by `IndexMap`,
/// so callers observe keys (and entries) in file order. Used to build the work
/// list and the run-summary counts; the WRITE path is `splice_aliases`, not a
/// re-serialization of these values.
///
/// Panics on I/O or parse error (these are programming errors in a dev-only tool).
pub fn load_pool(path: &str) -> Vec<Value> {
    let json = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("parse {path}: {e}"))
}

/// Read a pool file's raw text (the splice operates on this, not on parsed values).
///
/// Panics on I/O error.
pub fn read_text(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

/// Write spliced pool text back to disk verbatim (the splice already preserves the
/// trailing newline because it copies every non-inserted byte unchanged).
///
/// Panics on I/O error.
pub fn write_text(path: &str, text: &str) {
    std::fs::write(path, text).unwrap_or_else(|e| panic!("write {path}: {e}"));
}

// ---------------------------------------------------------------------------
// Byte-preserving alias splice (the writeback — D-60.4)
// ---------------------------------------------------------------------------

/// Insert `search_aliases` arrays into the pool TEXT without disturbing any other byte.
///
/// For each top-level array entry whose `op_variant` is a key in `by_op` and which does
/// not already contain a `search_aliases` key, the alias array is spliced in immediately
/// before the entry's closing brace, with a comma appended to the previous last property.
/// Entries are matched by their `op_variant` value; ordering, indentation (8-space key,
/// 12-space element — the pool format), inline sub-structures and non-ASCII bytes are
/// preserved exactly.
///
/// Returns the new file text, or an `Err` describing a structural problem.
pub fn splice_aliases(
    original: &str,
    by_op: &BTreeMap<String, Vec<String>>,
) -> Result<String, String> {
    let src = original.as_bytes();
    let n = src.len();
    let mut out: Vec<u8> = Vec::with_capacity(n + 4096);

    let mut i = 0usize;
    let mut depth: i32 = 0; // structural nesting of { and [ (outside strings)
    let mut in_string = false;
    let mut escaped = false;
    let mut entry_open: Option<usize> = None; // byte offset of the '{' that opened the current entry

    while i < n {
        let b = src[i];

        if in_string {
            out.push(b);
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        match b {
            b'"' => {
                in_string = true;
                out.push(b);
            }
            b'{' | b'[' => {
                depth += 1;
                if depth == 2 && b == b'{' {
                    entry_open = Some(i);
                }
                out.push(b);
            }
            b'}' => {
                if depth == 2 {
                    // Entry-closing brace: decide whether to splice aliases in.
                    let start = entry_open.take().unwrap_or(i);
                    let entry_src = &original[start..i];
                    if let Some(op) = extract_op_variant(entry_src) {
                        if let Some(aliases) = by_op.get(&op) {
                            if !entry_src.contains("\"search_aliases\"") && !aliases.is_empty() {
                                // `out` currently ends with the entry's last property value
                                // followed by the whitespace that indents this closing brace
                                // (e.g. "\n    "). Trim that whitespace, append a comma to the
                                // last property, splice the alias block, then restore the
                                // whitespace so the brace lands exactly where it was.
                                let mut k = out.len();
                                while k > 0 && matches!(out[k - 1], b' ' | b'\t' | b'\n' | b'\r') {
                                    k -= 1;
                                }
                                let trailing_ws: Vec<u8> = out[k..].to_vec();
                                out.truncate(k);
                                out.push(b',');
                                push_alias_block(&mut out, aliases);
                                out.extend_from_slice(&trailing_ws);
                            }
                        }
                    }
                }
                depth -= 1;
                out.push(b);
            }
            b']' => {
                depth -= 1;
                out.push(b);
            }
            _ => out.push(b),
        }
        i += 1;
    }

    if depth != 0 {
        return Err(format!("unbalanced JSON structure (final depth {depth})"));
    }
    String::from_utf8(out).map_err(|e| format!("splice produced invalid UTF-8: {e}"))
}

/// Append a `search_aliases` block at the pool's property indent (8-space key, 12-space
/// elements, 8-space closing bracket). No leading comma — the caller appends the comma to
/// the previous property first. No trailing newline — the caller restores the brace indent.
fn push_alias_block(out: &mut Vec<u8>, aliases: &[String]) {
    out.extend_from_slice(b"\n        \"search_aliases\": [");
    for (idx, alias) in aliases.iter().enumerate() {
        out.extend_from_slice(b"\n            \"");
        push_json_escaped(out, alias);
        out.push(b'"');
        if idx + 1 < aliases.len() {
            out.push(b',');
        }
    }
    out.extend_from_slice(b"\n        ]");
}

/// JSON-escape a string's content into `out`: escape `"`, `\`, and control chars, but emit
/// non-ASCII codepoints (umlauts, Σ, …) as literal UTF-8 (matching the pools' un-escaped style).
fn push_json_escaped(out: &mut Vec<u8>, s: &str) {
    for c in s.chars() {
        match c {
            '"' => out.extend_from_slice(b"\\\""),
            '\\' => out.extend_from_slice(b"\\\\"),
            '\n' => out.extend_from_slice(b"\\n"),
            '\r' => out.extend_from_slice(b"\\r"),
            '\t' => out.extend_from_slice(b"\\t"),
            c if (c as u32) < 0x20 => {
                out.extend_from_slice(format!("\\u{:04x}", c as u32).as_bytes());
            }
            c => {
                let mut buf = [0u8; 4];
                out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            }
        }
    }
}

/// Extract the `op_variant` string value from an entry's source text (first occurrence).
/// op_variant values are simple identifiers, so a forward scan to the closing quote suffices.
fn extract_op_variant(entry_src: &str) -> Option<String> {
    let key_pos = entry_src.find("\"op_variant\"")?;
    let rest = &entry_src[key_pos + "\"op_variant\"".len()..];
    let colon = rest.find(':')?;
    let after_colon = &rest[colon + 1..];
    let open_q = after_colon.find('"')?;
    let value_start = &after_colon[open_q + 1..];
    let mut esc = false;
    for (bi, ch) in value_start.char_indices() {
        if esc {
            esc = false;
        } else if ch == '\\' {
            esc = true;
        } else if ch == '"' {
            return Some(value_start[..bi].to_string());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Real-shape fixture: the LAST key of each entry is a compact single-line `xrom` object,
    /// exactly like the math1/stat1/time/advantage pools. This is the case the original
    /// `PrettyFormatter` writeback churned (expanding it to multi-line) — the regression guard.
    const FIXTURE_INLINE_XROM: &str = r#"[
    {
        "op_variant": "TimeNow",
        "display_name": "TIME",
        "category": "Clock",
        "status": "implemented",
        "phase": "38",
        "key_path": "XEQ \"TIME\"",
        "description": "Display current system time",
        "xrom": { "module": "Time", "module_id": 26, "function_id": 1 }
    },
    {
        "op_variant": "TimeDate",
        "display_name": "DATE",
        "category": "Clock",
        "status": "implemented",
        "phase": "38",
        "key_path": "XEQ \"DATE\"",
        "description": "Display current date",
        "xrom": { "module": "Time", "module_id": 26, "function_id": 2 }
    }
]
"#;

    fn by_op(pairs: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
            .collect()
    }

    /// The inline `xrom` line must remain byte-identical after a splice; the ONLY changes are
    /// the appended comma on the xrom line and the new search_aliases block (D-60.4).
    #[test]
    fn splice_preserves_inline_xrom() {
        let map = by_op(&[("TimeNow", &["current time", "Uhrzeit"])]);
        let out = splice_aliases(FIXTURE_INLINE_XROM, &map).unwrap();

        // The inline xrom object for the UNCHANGED second entry stays exactly on one line.
        assert!(
            out.contains("\"xrom\": { \"module\": \"Time\", \"module_id\": 26, \"function_id\": 2 }"),
            "untouched entry's inline xrom must not be reformatted"
        );
        // The CHANGED entry's xrom stays inline too — only a trailing comma is added to it.
        assert!(
            out.contains("\"xrom\": { \"module\": \"Time\", \"module_id\": 26, \"function_id\": 1 },"),
            "changed entry's inline xrom must stay one line, gaining only a trailing comma"
        );
        // The alias block was inserted at the right indent, after the xrom.
        assert!(out.contains("\n        \"search_aliases\": [\n            \"current time\",\n            \"Uhrzeit\"\n        ]"));
        // Output is still valid JSON with aliases landing as the entry's last key.
        let parsed: Vec<Value> = serde_json::from_str(&out).unwrap();
        let keys: Vec<&str> = parsed[0].as_object().unwrap().keys().map(|s| s.as_str()).collect();
        assert_eq!(keys.last(), Some(&"search_aliases"), "aliases must be the last key");
        assert!(parsed[1].as_object().unwrap().get("search_aliases").is_none(), "entry not in by_op stays unaliased");
    }

    /// A no-op splice (no matching op_variants) yields byte-identical output.
    #[test]
    fn splice_noop_when_absent() {
        let map = by_op(&[("NotInPool", &["x"])]);
        let out = splice_aliases(FIXTURE_INLINE_XROM, &map).unwrap();
        assert_eq!(out, FIXTURE_INLINE_XROM, "no matching entries -> byte-identical");
    }

    /// An entry that already carries `search_aliases` is never touched (fill-only safety net).
    #[test]
    fn splice_skips_populated() {
        let fixture = r#"[
    {
        "op_variant": "Already",
        "status": "implemented",
        "description": "has aliases",
        "search_aliases": ["keep me"]
    }
]
"#;
        let map = by_op(&[("Already", &["should not appear"])]);
        let out = splice_aliases(fixture, &map).unwrap();
        assert_eq!(out, fixture, "pre-populated entry must be left byte-for-byte unchanged");
        assert!(!out.contains("should not appear"));
    }

    /// Non-ASCII alias content (umlaut, Σ, em-dash) is written literally, never `\uXXXX`.
    #[test]
    fn splice_utf8_literal() {
        let map = by_op(&[("TimeNow", &["verfügbare Register", "CLΣ — Zeit …"])]);
        let out = splice_aliases(FIXTURE_INLINE_XROM, &map).unwrap();
        assert!(out.contains("verfügbare Register"), "umlaut must be literal");
        assert!(out.contains("CLΣ — Zeit …"), "Σ/—/… must be literal");
        assert!(!out.contains("\\u00fc") && !out.contains("\\u03a3") && !out.contains("\\u2014"));
        // Still valid JSON.
        serde_json::from_str::<Vec<Value>>(&out).unwrap();
    }

    /// When an entry's last key is a plain string (not an inline object), the splice still
    /// appends only a comma + the alias block (the simple, common hp41cv shape).
    #[test]
    fn splice_appends_last() {
        let fixture = r#"[
    {
        "op_variant": "Plus",
        "status": "implemented",
        "notes": "Consumes X and Y."
    }
]
"#;
        let map = by_op(&[("Plus", &["add", "addieren"])]);
        let out = splice_aliases(fixture, &map).unwrap();
        assert!(out.contains("\"notes\": \"Consumes X and Y.\",\n        \"search_aliases\": ["));
        let parsed: Vec<Value> = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed[0].as_object().unwrap().get("search_aliases").unwrap().as_array().unwrap().len(), 2);
    }
}
