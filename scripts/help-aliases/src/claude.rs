//! `claude -p` subprocess wrapper: invoke_claude() + retry + envelope/fence parsing.
//!
//! Key design:
//! - `invoke_claude(prompt)`: retries up to 3× with 2s delay on non-zero exit or bad JSON.
//! - `parse_envelope(stdout)`: pure helper — two-pass parse (outer envelope → inner JSON).
//! - `strip_fences(s)`: pure helper — strips optional ```json...``` markdown fencing.
//!
//! The pure helpers (`parse_envelope`, `strip_fences`) are what the tests target —
//! no live subprocess is invoked on the test path (deterministic unit tests only).
//!
//! claude binary: resolved via PATH (not hardcoded) for portability (D-60.1).
//! Flags: -p --output-format json --max-turns 1 --bare --dangerously-skip-permissions --model sonnet

use serde_json::Value;
use std::process::Command;
use std::time::Duration;

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_SECS: u64 = 2;

/// Invoke `claude -p` with the given prompt and return the parsed inner JSON.
///
/// Retries up to `MAX_RETRIES` times on non-zero exit or unparseable response.
/// Returns `Err` with a descriptive message on final failure (never silently drops).
pub fn invoke_claude(prompt: &str) -> Result<Value, String> {
    let mut last_err = String::new();
    for attempt in 0..MAX_RETRIES {
        if attempt > 0 {
            std::thread::sleep(Duration::from_secs(RETRY_DELAY_SECS));
        }
        match try_invoke(prompt) {
            Ok(v) => return Ok(v),
            Err(e) => {
                if attempt < MAX_RETRIES - 1 {
                    eprintln!("  [claude] attempt {}/{MAX_RETRIES} failed: {e}", attempt + 1);
                }
                last_err = e;
            }
        }
    }
    Err(format!("claude failed after {MAX_RETRIES} attempts: {last_err}"))
}

fn try_invoke(prompt: &str) -> Result<Value, String> {
    let output = Command::new("claude")
        .args([
            "-p",
            "--output-format",
            "json",
            "--max-turns",
            "1",
            "--bare",
            "--dangerously-skip-permissions",
            "--model",
            "sonnet",
            prompt,
        ])
        .output()
        .map_err(|e| format!("spawn failed: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "exit {:?}: {}",
            output.status.code(),
            stderr.trim()
        ));
    }

    parse_envelope(&output.stdout)
}

/// Parse the `claude --output-format json` envelope and return the inner alias JSON.
///
/// Two-pass parse (Pitfall 4):
///   1. Parse outer envelope: `{"type":"result","result":"<text>", ...}`
///   2. Extract `.result` string, strip fences, parse as the alias JSON object.
///
/// This is a pure function that takes raw stdout bytes — no subprocess invocation.
/// Tests target this helper directly.
pub fn parse_envelope(stdout: &[u8]) -> Result<Value, String> {
    let envelope: Value = serde_json::from_slice(stdout)
        .map_err(|e| format!("envelope parse error: {e}"))?;

    let inner_text = envelope["result"]
        .as_str()
        .ok_or_else(|| "envelope missing 'result' field".to_string())?;

    let inner_text = strip_fences(inner_text);

    serde_json::from_str(inner_text).map_err(|e| {
        let preview = &inner_text[..inner_text.len().min(200)];
        format!("inner JSON parse error: {e}\nRaw: {preview}")
    })
}

/// Strip optional markdown code fences from an LLM response string.
///
/// Handles:
/// - ` ```json\n...\n``` ` (fenced JSON block)
/// - ` ```\n...\n``` ` (fenced without language tag)
/// - Plain JSON (returned unchanged)
///
/// Leading prose sentences before the fence are also handled by looking for
/// the first occurrence of ` ``` ` in the string.
///
/// This is a pure function — tests target it directly without a subprocess.
pub fn strip_fences(s: &str) -> &str {
    let s = s.trim();

    // Find the first code fence (handles leading prose + fence).
    if let Some(fence_start) = s.find("```") {
        // Skip the opening fence line (e.g. "```json\n").
        let after_fence_marker = &s[fence_start + 3..];
        let content_start = after_fence_marker
            .find('\n')
            .map(|i| &after_fence_marker[i + 1..])
            .unwrap_or(after_fence_marker);

        // Find the closing fence.
        if let Some(close_pos) = content_start.rfind("```") {
            content_start[..close_pos].trim()
        } else {
            // No closing fence — return content after opening fence as-is.
            content_start.trim()
        }
    } else {
        // No fences — return the trimmed string as-is.
        s
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// A string wrapped in ```json...``` fences must be reduced to raw inner JSON.
    /// A string with a leading prose sentence + fence must also be handled.
    /// An already-bare JSON string must be returned unchanged.
    #[test]
    fn strip_fences_works() {
        // Case 1: ```json ... ``` fencing.
        let fenced = "```json\n{\"Op\": [\"a\"]}\n```";
        assert_eq!(strip_fences(fenced), "{\"Op\": [\"a\"]}");

        // Case 2: ``` without language tag.
        let fenced_no_lang = "```\n{\"Op\": [\"b\"]}\n```";
        assert_eq!(strip_fences(fenced_no_lang), "{\"Op\": [\"b\"]}");

        // Case 3: Leading prose + fence.
        let with_prose = "Here are the aliases:\n```json\n{\"Op\": [\"c\"]}\n```";
        assert_eq!(strip_fences(with_prose), "{\"Op\": [\"c\"]}");

        // Case 4: Already bare JSON — returned unchanged (trimmed).
        let bare = r#"{"Op": ["d"]}"#;
        assert_eq!(strip_fences(bare), bare);

        // Case 5: Bare JSON with surrounding whitespace — trimmed.
        let bare_ws = "  {\"Op\": [\"e\"]}  ";
        assert_eq!(strip_fences(bare_ws), "{\"Op\": [\"e\"]}");
    }

    /// `parse_envelope` must correctly two-pass parse the claude `--output-format json`
    /// envelope and extract the inner JSON object (Pitfall 4).
    ///
    /// This uses a fixture stdout string — no live subprocess is invoked.
    #[test]
    fn envelope_parse_works() {
        // Fixture: the exact shape returned by claude --output-format json.
        let stdout = br#"{"type":"result","result":"{\"Op\":[\"a\",\"b\"]}"}"#;

        let result = parse_envelope(stdout).unwrap();
        let aliases = result["Op"].as_array().unwrap();
        assert_eq!(aliases.len(), 2);
        assert_eq!(aliases[0].as_str().unwrap(), "a");
        assert_eq!(aliases[1].as_str().unwrap(), "b");
    }

    /// `parse_envelope` must handle a fenced inner JSON correctly.
    #[test]
    fn envelope_parse_with_fenced_inner() {
        // The model may wrap its JSON output in markdown fences inside the .result field.
        let inner_json = r#"```json
{"Op": ["alias1"]}
```"#;
        let envelope_str = format!(
            "{{\"type\":\"result\",\"result\":{}}}",
            serde_json::to_string(inner_json).unwrap()
        );
        let stdout = envelope_str.as_bytes();

        let result = parse_envelope(stdout).unwrap();
        let aliases = result["Op"].as_array().unwrap();
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].as_str().unwrap(), "alias1");
    }
}
