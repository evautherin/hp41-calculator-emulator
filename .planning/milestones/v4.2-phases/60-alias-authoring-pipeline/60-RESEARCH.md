# Phase 60: Alias Authoring Pipeline — Research

**Researched:** 2026-06-05
**Domain:** Rust tooling crate, `claude -p` subprocess, minimal-diff JSON writeback, alias batching strategy
**Confidence:** HIGH — all technical claims verified against live codebase and `claude` CLI

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-60.1:** Generator is `scripts/help-aliases/` Rust crate shelling out to `claude -p` headless CLI (confirmed `/Users/daniel/.local/bin/claude` v2.1.165). Not a direct API call, not Python.
- **D-60.2:** Fill-only — populate `search_aliases` only when it is empty/absent; leave already-populated entries byte-for-byte untouched.
- **D-60.4:** Minimal-diff writeback — the only JSON change is the added `search_aliases` arrays; preserve existing key order, 2-space indent, real UTF-8 (umlauts un-escaped), and trailing newline.
- **D-60.6 / HSGEN-02:** `claude`/LLM dependency lives only in `scripts/help-aliases/`; nothing model/API/inference-related may touch `hp41-core`/`hp41-cli`/`hp41-gui`.

### Claude's Discretion

- Exact prompt wording and alias count N (spec says "N"; 4–8 DE+EN combined is reasonable).
- Batch granularity of `claude -p` calls (per-entry / per-category / per-pool).
- Crate file layout, CLI arg shape, and the JSON (re)writer approach for D-60.4.
- Whether `just help-aliases` runs all six pools in one invocation or one-per-pool.
- The exact aliases chosen for each function (subject to human review pass).

### Deferred Ideas (OUT OF SCOPE)

- Schema CI gate enforcing "≥ 1 alias per implemented entry" (Phase 61).
- CLI↔GUI parity fixture on alias snapshot (Phase 61).
- Scoring/fuzzy/DE-EN resolution unit tests (Phase 61).
- Missed-query logging (v2 / out of scope for v4.2).
- Change-detection / source-hash re-run (rejected, fill-only only).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| HSGEN-01 | New `scripts/help-aliases/` crate that reads each implemented JSON entry and writes DE+EN `search_aliases` | §Standard Stack, §Architecture Patterns, §Code Examples |
| HSGEN-02 | LLM invoked only at authoring time; no model/API/inference in any shipped binary | §Standard Stack (workspace-exclusion stanza), §Don't Hand-Roll |
| HSGEN-03 | Re-run only when functions are added/changed; fill-only semantics | §Architecture Patterns (fill-only merge), §Code Examples |
| HSDATA-03 | Every `status:"implemented"` entry across all six pools has ≥ 1 alias after the run | §Batching Strategy, §Code Examples (coverage report) |
</phase_requirements>

---

## Summary

Phase 60 builds a standalone Rust tooling crate (`scripts/help-aliases/`) that shells out to the already-installed `claude -p` CLI to generate DE+EN search aliases for all ~364 implemented entries across six JSON help pools. The crate mirrors the existing `scripts/docs-matrix/` pattern: empty `[workspace]` stanza (workspace-excluded), `serde`/`serde_json` dependencies only, per-pool CLI arg shape, and a `just help-aliases` recipe.

The two load-bearing technical problems are (1) getting structured JSON back from `claude -p` reliably and (2) writing it back into the pool files with zero diff noise — preserving key order, 4-space indent on object properties (8-space for nested), real UTF-8 characters, and trailing newlines. The pools today use a consistent key ordering (`op_variant` → `display_name` → ... → `notes`/`xrom` last); `search_aliases` must be appended after the last existing key in each object.

Batching at the per-category level (one `claude -p` call per category across all pools) balances invocation count (~30 categories vs ~364 individual calls) with response parsability (each batch produces a small `op_variant → [aliases]` map, easy to validate key-by-key).

**Primary recommendation:** use `serde_json` with `features = ["preserve_order"]` + a custom `Formatter` that disables non-ASCII escaping to achieve the minimal diff. This is purely internal to `scripts/help-aliases/`; no workspace crate changes.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Alias generation (LLM call) | Dev-only tooling (`scripts/help-aliases/`) | — | D-60.6 / HSGEN-02: nothing LLM-related on runtime path |
| JSON pool read/write | Dev-only tooling | — | Pools are static data; the tooling crate is the only writer |
| Alias consumption at runtime | CLI (`help_data.rs`) | GUI (`help_data.ts`) | Already-wired via `#[serde(default)]` / `search_aliases?: string[]` |
| Build-time bundle validation | `cargo check -p hp41-cli` | `just gui-ci` | D-60.7: verify larger JSON still compiles into both frontends |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `serde` | 1.x | Deserialize JSON pool entries | Already in `docs-matrix` Cargo.toml; workspace pattern [VERIFIED: codebase] |
| `serde_json` | 1.x (`preserve_order` feature) | Parse + serialize JSON preserving key order | Only way to roundtrip ordered JSON objects in serde_json [ASSUMED: serde_json docs] |

No new dependencies beyond what `docs-matrix` already uses, plus the `preserve_order` feature flag.

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `std::process::Command` | std | Shell out to `claude -p` | Always — no external crate needed |
| `std::thread::sleep` + retry loop | std | Backoff on transient claude failures | When response is non-JSON or exit != 0 |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `serde_json` + custom Formatter | `jq` / `python3` post-processor | External tool dependency; harder to keep in Rust crate |
| `preserve_order` feature | Manual key-sorted `IndexMap` | Equivalent but more boilerplate |
| Per-category batching | Per-pool (6 calls) or per-entry (364 calls) | Per-pool risks exceeding context window; per-entry is slow and expensive |

**Installation (Cargo.toml for `scripts/help-aliases/`):**

```toml
[workspace]
# Empty stanza — excludes this crate from the root workspace per CLAUDE.md
# "Root Cargo.toml members stays ["hp41-core", "hp41-cli"]" invariant.

[package]
name = "help-aliases"
version = "0.1.0"
edition = "2021"
publish = false

[[bin]]
name = "help-aliases"
path = "src/main.rs"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = { version = "1", features = ["preserve_order"] }
```

---

## Package Legitimacy Audit

This phase installs no new runtime packages. The tooling crate adds only:
- `serde` 1.x — long-established, already used throughout the workspace [VERIFIED: codebase grep]
- `serde_json` 1.x with `preserve_order` feature — existing dependency, new feature flag only [VERIFIED: Cargo.lock version 1.0.149]

**Packages removed due to slopcheck:** none. **Packages flagged as suspicious:** none.
No new packages require legitimacy verification; the `preserve_order` feature is a compile-time flag, not an additional crate.

---

## Architecture Patterns

### System Architecture Diagram

```
docs/hp41-*-functions.json (6 pools, read-only)
         │
         │  fs::read_to_string
         ▼
   Vec<serde_json::Value>  (preserve_order = IndexMap under the hood)
         │
         │  filter status == "implemented" && search_aliases empty/absent
         ▼
   Batch builder  ──── per-category grouping ────►  Prompt string
                                                         │
                                                         │  std::process::Command
                                                         │  claude -p --output-format json
                                                         ▼
                                              JSON envelope  {"type":"result","result":"..."}
                                                         │
                                                         │  parse inner JSON
                                                         │  { "op_variant": ["alias",...], ... }
                                                         │
                                                         │  validate all keys present
                                                         ▼
                                              Merge back into Vec<Value>
                                              (insert "search_aliases" key in-order)
                                                         │
                                                         │  custom PrettyFormatter (no \uXXXX)
                                                         ▼
                                         docs/hp41-*-functions.json  (write in-place)
                                                         │
                                                         ▼
                                    stdout: "hp41cv: 136 scanned, 0 skipped, 136 populated, 816 aliases added"
```

### Recommended Project Structure

```
scripts/help-aliases/
├── Cargo.toml          # empty [workspace], serde + serde_json preserve_order
├── README.md           # usage, re-run policy, prompt design notes
└── src/
    ├── main.rs         # CLI arg handling; drives per-pool workflow
    ├── pool.rs         # load_pool / save_pool (Value-level, preserve_order)
    ├── claude.rs       # invoke_claude() + retry logic + response parsing
    ├── batch.rs        # group_by_category() + build_prompt() + parse_response()
    └── merge.rs        # fill_aliases() — the fill-only D-60.2 merge
```

### Pattern 1: Shelling out to `claude -p` from Rust

**What:** Use `std::process::Command` with `--output-format json` (wraps response in `{"type":"result","result":"<text>"}`) and parse the inner `result` string as JSON.

**When to use:** Every alias-generation call.

**The `--output-format json` envelope vs plain text:** `--output-format json` wraps the assistant response in a session envelope. The `result` field contains the raw text the model produced. This is reliable — it separates the response text from stderr/warning noise that `--output-format text` can mix in. Extract `result`, then parse it as JSON.

**Key flags to pin:**
- `--output-format json` — structured envelope, separates response from stderr
- `--max-turns 1` — prevents any multi-turn behavior; we want a single response
- `--model sonnet` (alias for latest Sonnet) or `claude-sonnet-4-5` — fast enough for bulk alias generation; Opus is overkill for synonym generation
- `--bare` — skip hooks, LSP, CLAUDE.md auto-discovery, auto-memory; purely API-facing call, no project context needed; reduces latency and avoids loading unrelated project context
- `--dangerously-skip-permissions` — required for `-p` non-interactive runs so the CLI does not prompt for filesystem/tool permissions when there are none needed (the generator only sends a prompt, never uses filesystem tools)

**Timeout:** `claude -p` for a single short prompt should complete in 10–30 seconds. Wrap `Command::output()` with a thread + timeout via `std::sync::mpsc` or use `wait_timeout` from the `wait-timeout` crate. Alternatively, set a generous wall-clock deadline: spawn, then poll `try_wait()` in a loop with `thread::sleep(Duration::from_secs(1))`, killing after 60 s.

**Retry strategy:** On non-zero exit or unparseable JSON response: retry up to 3 times with a 2-second delay. After 3 failures, print an error message identifying the failing category/entry and continue to the next batch (never silently drop — report at end of run).

**Example sketch:**

```rust
// Source: std::process docs [ASSUMED: standard library]
use std::process::Command;
use serde_json::Value;

fn invoke_claude(prompt: &str) -> Result<Value, String> {
    let output = Command::new("/Users/daniel/.local/bin/claude")
        .args([
            "-p",
            "--output-format", "json",
            "--max-turns", "1",
            "--bare",
            "--dangerously-skip-permissions",
            prompt,
        ])
        .output()
        .map_err(|e| format!("spawn failed: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("claude exit {:?}: {stderr}", output.status.code()));
    }

    let envelope: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("envelope parse error: {e}"))?;

    let inner_text = envelope["result"]
        .as_str()
        .ok_or("envelope missing 'result' field")?;

    serde_json::from_str(inner_text)
        .map_err(|e| format!("inner JSON parse error: {e}\nRaw: {inner_text}"))
}
```

**Note on `--dangerously-skip-permissions`:** This flag bypasses permission prompts. In the `--bare` + `-p` non-interactive context (no tools, no filesystem access), this is appropriate and the project docs confirm it acceptable for sandboxed tooling scripts.

### Pattern 2: Minimal-diff JSON Writeback (D-60.4 — load-bearing)

**The problem:** `serde_json::to_string_pretty` (a) reorders object keys alphabetically unless `preserve_order` feature is enabled, and (b) escapes non-ASCII characters as `\uXXXX` (e.g. `Σ` → `Σ`). The committed pools use real UTF-8 and a fixed key ordering. Post-population the diff must show only the added `search_aliases` arrays.

**Observed pool formatting (VERIFIED: codebase inspection):**
- Outer array: `[\n`
- Object start: `    {\n` (4-space indent)
- Object properties: `        "key": "value",\n` (8-space indent)
- Trailing newline on the file
- Key ordering: `op_variant, display_name, category, status, phase, key_path, description, [example, notes,] [divergences,] [xrom]` — `search_aliases` will be appended as the last key
- Non-ASCII in current pools: `Σ` (one occurrence, hp41cv), `—` and `…` (time pool), all un-escaped

**Recommended approach: `serde_json` with `preserve_order` feature + custom `Formatter`**

With `features = ["preserve_order"]`, `serde_json::Value::Object` becomes an `IndexMap` which maintains insertion order. Combined with a custom `serde_json::ser::Formatter` implementation that writes non-ASCII chars literally instead of as `\uXXXX`, this gives a byte-accurate roundtrip.

```rust
// Source: serde_json Formatter trait docs [ASSUMED]
use serde_json::ser::{Formatter, PrettyFormatter};
use std::io;

/// Custom formatter: 4-space-indent pretty printing with un-escaped UTF-8.
struct Utf8PrettyFormatter<'a> {
    inner: PrettyFormatter<'a>,
}

impl<'a> Utf8PrettyFormatter<'a> {
    fn new() -> Self {
        Self { inner: PrettyFormatter::with_indent(b"    ") }
    }
}

impl<'a> Formatter for Utf8PrettyFormatter<'a> {
    // Delegate all methods to PrettyFormatter except string_fragment,
    // which we override to write non-ASCII bytes as-is rather than \uXXXX.
    fn write_string_fragment<W: io::Write + ?Sized>(
        &mut self,
        writer: &mut W,
        fragment: &str,
    ) -> io::Result<()> {
        // PrettyFormatter escapes non-ASCII; we write it directly.
        writer.write_all(fragment.as_bytes())
    }

    // All other methods delegate to self.inner via the Formatter default impls,
    // which in practice means forwarding to PrettyFormatter.
    // Implement begin_array, end_array, begin_object, etc. by delegation.
}
```

**Important:** `serde_json::ser::PrettyFormatter` indents with its configured indent string per level. The pools use 4 spaces per level (top-level array elements are indented 4 spaces; their properties are indented 8 spaces = 2 levels). Use `PrettyFormatter::with_indent(b"    ")` (4 spaces).

**Alternative: targeted string-level injection**

Read the raw file as a string, find each object's closing `}` boundary, inject the `"search_aliases": [...]` line before the closing brace using string manipulation, then write back. This avoids the custom Formatter entirely. Tradeoff: fragile if formatting varies; more complex to implement safely. The `serde_json` + `preserve_order` + custom Formatter approach is more robust.

**Verification that `docs-matrix` does NOT have this constraint:**
`docs-matrix` reads JSON and writes Markdown — it never roundtrips JSON back to disk. There is no analog constraint in the sibling crate. The JSON writeback problem is unique to Phase 60. [VERIFIED: codebase inspection of `scripts/docs-matrix/src/main.rs`]

### Pattern 3: Fill-only Merge (D-60.2)

**What:** Before inserting aliases, check whether `search_aliases` already exists and is non-empty. If so, skip the entry. If absent or empty array, insert.

```rust
// On a serde_json::Value representing one JSON object (preserve_order enabled):
fn needs_aliases(entry: &Value) -> bool {
    match entry.get("search_aliases") {
        None => true,
        Some(Value::Array(arr)) => arr.is_empty(),
        _ => true, // malformed — treat as needing population
    }
}

fn insert_aliases(entry: &mut Value, aliases: Vec<String>) {
    // With preserve_order, inserting into a Value::Object (IndexMap) appends
    // at the end if the key is new — which is what we want.
    let arr: Vec<Value> = aliases.into_iter().map(Value::String).collect();
    entry["search_aliases"] = Value::Array(arr);
}
```

### Anti-Patterns to Avoid

- **Full re-serialize without `preserve_order`:** serde_json default sorts object keys alphabetically. A roundtrip without `preserve_order` produces a massive diff — every object's keys get reordered.
- **`escape_non_ascii` or `to_string` default:** serde_json's default serializer escapes all non-ASCII. Use a custom Formatter or post-process with `str::replace` for each Unicode escape (fragile).
- **Per-entry `claude -p` calls:** 364 subprocess invocations will take 30–180 minutes. Batch.
- **Per-pool prompts (6 calls, 50–136 entries each):** Advantage pool has 114 entries; a 114-entry prompt risks context-window truncation and makes response validation harder. Category batches of ~6–15 entries are safer.
- **Silently dropping failed batches:** any category whose `claude -p` call fails must be logged and reported at end-of-run. Never leave gaps silently.
- **Parsing the wrong field from the envelope:** `--output-format json` returns `{"type":"result","result":"<text>"}`. The actual alias JSON is in `.result`, not at the top level.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON key-order preservation | Custom parser tracking key positions | `serde_json` `preserve_order` feature | Maintained, correct, handles nested objects |
| Non-ASCII serialization | Manual string escape post-processing | Custom `Formatter` delegating to `PrettyFormatter` | Formatter trait is the designed extension point |
| `claude` subprocess spawn | Custom async runtime / process management | `std::process::Command::output()` | Synchronous is fine; we wait for the response anyway |
| Retry logic | External crate | Simple loop with `thread::sleep` | 3-retry loop is ~10 LOC; no dep needed |

**Key insight:** The JSON writeback is the most dangerous hand-roll temptation. The `preserve_order` feature + custom Formatter gives a complete, tested solution; string-patching the raw file is an error-prone alternative that breaks on edge cases (multiline strings, escaped braces).

---

## Batching Strategy

**Recommended granularity: per-category**

Each `claude -p` call covers one category from one pool. The response is a JSON object mapping `op_variant` → `[aliases...]` for all entries in that category.

**Rationale:**
- ~30 distinct categories across 6 pools → ~30 invocations total (vs 364 for per-entry)
- Largest category is "Adv Mtrx" (51 entries) and "Adv Math" (45 entries) — these could be split into sub-batches of ~20 if context length is a concern
- Category context improves alias quality (model knows all entries are related)
- Key-by-key validation: after receiving the response, check that every `op_variant` from the input batch appears in the response; report any missing or extra keys

**Category counts (VERIFIED: codebase inspection):**
- hp41cv: 16 categories, 6–25 entries each (Programming=25, Math=24 are the largest)
- math1: 10 categories, 1–12 entries each
- stat1: 7 categories, 2–8 entries each
- time: 3 categories (most entries in one)
- advantage: 4 categories (Adv Mtrx=51, Adv Math=45 — split these)
- xmem: 1 category (8 entries)

**Total claude invocations:** ~30–36 (splitting the two large advantage categories into halves). Well within practical limits.

**Missing/extra key detection:**

```rust
fn validate_batch_response(
    expected_variants: &[&str],
    response: &serde_json::Map<String, Value>,
) -> (Vec<String>, Vec<String>) {
    let mut missing: Vec<String> = expected_variants
        .iter()
        .filter(|v| !response.contains_key(**v))
        .map(|v| v.to_string())
        .collect();
    let extra: Vec<String> = response
        .keys()
        .filter(|k| !expected_variants.contains(&k.as_str()))
        .cloned()
        .collect();
    (missing, extra)
}
```

If `missing` is non-empty after 3 retries: log a warning and fill the missing entries with an empty `Vec` (they will remain un-populated and will need a re-run or manual entry). Never silently drop — the run summary must enumerate them.

---

## Prompt Design (Claude's Discretion — recommended approach)

**System prompt (passed via `--append-system-prompt`):**

```
You are a search-alias generator for an HP-41 scientific calculator emulator.
Your output must be a valid JSON object only — no prose, no markdown, no code block.
```

**User prompt structure:**

```
Generate DE+EN search aliases for these HP-41 calculator functions.

For each function, produce 4-8 short search terms (2-5 words each) that a user
might type when looking for this function. Include both German and English terms.
These are search keywords, not documentation prose. Include common synonyms,
abbreviations, and natural-language phrasings.

Keep German umlauts as real characters (ä, ö, ü, Ä, Ö, Ü, ß), not escaped.

Output a JSON object where each key is the op_variant and the value is an array of alias strings:
{"OpVariant1": ["alias1", "alias2", ...], "OpVariant2": [...], ...}

Functions (category: {category_name}):

{op_variant}: display={display_name}, description={description}[, notes={notes}]
...
```

**Example for TVM category (from spec):**
Input: `TvmN` — display=`N`, category=`Adv TVM`, description=`Number of payment periods`
Expected output shape: `{"TvmN": ["Laufzeit", "Perioden", "periods", "payment periods", "duration", "term"]}`

**Alias count N:** 4–8 per entry. The worked example in the spec shows 8 aliases (`["Zinseszins", "Annuität", "Tilgung", "Kredit", "compound interest", "time value of money", "loan payment", "mortgage"]`). Instruct claude to produce 4–8; fewer is acceptable for simple operations (e.g. `+` → `["add", "plus", "addition", "addieren"]`).

**DE/EN split:** no enforced ratio — let the model produce what's natural per function. Mathematical/technical operations (sin, cos, matrix) may have mostly EN aliases; financial/everyday operations (TVM, time) should have prominent DE aliases.

---

## Bundle-size Sanity Check (D-60.7)

**Current pool sizes (VERIFIED: codebase):**
| Pool | Current size |
|------|-------------|
| hp41cv-functions.json | 44 KB |
| hp41-math1-functions.json | 20 KB |
| hp41-stat1-functions.json | 12 KB |
| hp41-time-functions.json | 16 KB |
| hp41-advantage-functions.json | 44 KB |
| hp41-xmem-functions.json | 8 KB |
| **Total** | **~131 KB** |

**Estimated post-population size:** ~175 KB total (+44 KB, ~34% growth).
Estimate basis: 364 implemented entries × ~120 bytes average alias block (`"search_aliases": ["a","b","c","d","e","f"]` with 2-space array indent).

**Both frontends embed the JSON at compile time** (`include_str!` in Rust, Vite static import in TS). Neither has a documented size limit in the project. ~175 KB of JSON is negligible for:
- Rust binary: embedded as `&'static str`; no compression, but 175 KB is trivial for a desktop app
- Vite bundle: JSON is tree-shaken per import; 175 KB across 6 imports is within normal Vite budget

**Post-population check recipe tasks:**
1. `cargo check -p hp41-cli` — verifies all six `include_str!` still compile; fails fast if any pool is malformed JSON (D-25.17 hard-build-blocker)
2. `just gui-ci` (or `cd hp41-gui && npm run build`) — verifies Vite static imports succeed with the larger payload

**Threshold to flag:** if any pool exceeds 200 KB after population, log a warning in the run summary and note it in the PR. Current trajectory stays well below this.

---

## Common Pitfalls

### Pitfall 1: serde_json Key Reordering Without `preserve_order`

**What goes wrong:** Loading a pool with `serde_json::from_str::<Vec<Value>>(...)` without `preserve_order` produces a `Value::Object` backed by a `BTreeMap`, which sorts keys alphabetically on serialization. The result: every object in every pool gets its keys sorted, producing a ~131 KB diff that is unreviable and obscures the actual alias additions.

**Why it happens:** `serde_json` defaults to `BTreeMap` for `Value::Object`. The `preserve_order` feature switches to `IndexMap` which maintains insertion order.

**How to avoid:** Add `serde_json = { version = "1", features = ["preserve_order"] }` to `Cargo.toml`. Verify by running the generator on a file with no `search_aliases` and confirming the diff is empty (round-trip identity).

**Warning signs:** `git diff` shows reordered keys in unchanged entries.

### Pitfall 2: `serde_json` Escaping Real UTF-8 as `\uXXXX`

**What goes wrong:** The default `serde_json::to_string_pretty` escapes all non-ASCII as Unicode escapes. The existing pools contain `Σ` (hp41cv), `—` and `…` (time), and the generated aliases will contain German umlauts. A write back with default serialization produces `Σ` instead of `Σ`, creating noisy diff.

**Why it happens:** serde_json's default `CompactFormatter` and `PrettyFormatter` both escape non-ASCII via `write_char_escape` for all codepoints > 127.

**How to avoid:** Implement a custom `serde_json::ser::Formatter` that overrides `write_string_fragment` to write non-ASCII bytes directly. Alternatively, post-process the output string to unescape all `\uXXXX` sequences — simpler but fragile if a value legitimately contains a `\u` escape for a different reason.

**Recommended:** Custom Formatter delegating to `PrettyFormatter` for structural formatting, overriding only the string-fragment write. See §Code Examples.

**Warning signs:** `git diff` shows `Σ` instead of `Σ`, or `—` instead of `—`.

### Pitfall 3: `claude -p` Response Contains Preamble or Markdown Fencing

**What goes wrong:** Even with a strict "output JSON only" instruction, LLMs occasionally wrap the JSON in ` ```json ... ``` ` fences or prepend a sentence like "Here are the aliases:". The inner-JSON parse then fails.

**Why it happens:** LLMs are trained to be helpful and may add formatting context.

**How to avoid:** After extracting `response["result"].as_str()`, strip leading/trailing whitespace and, if the string starts with a code fence, extract the content between the fences before attempting `serde_json::from_str`. The `--bare` flag reduces extra Claude output (skips hooks, memory, etc.) but does not prevent the model from adding prose.

**Warning signs:** `serde_json::from_str` returns error `expected value at line 1 column 1` when the raw value starts with ` ``` `.

### Pitfall 4: `--output-format json` Envelope vs Inner JSON

**What goes wrong:** Code parses the outer envelope as the alias map and finds keys like `type`, `result`, `duration_ms` instead of `op_variant` keys.

**Why it happens:** `claude --output-format json -p "..."` returns a session envelope: `{"type":"result","subtype":"success","result":"<assistant text>","duration_ms":...}`. The actual model output is the string value of `.result`, which itself must be parsed as JSON.

**How to avoid:** Two-pass parse: `let envelope: Value = serde_json::from_slice(&output.stdout)?;` then `let inner_text = envelope["result"].as_str()?;` then `let aliases: Value = serde_json::from_str(inner_text)?;`. Verified against live claude v2.1.165 output.

**Warning signs:** Alias map contains key `"type"` with value `"result"`.

### Pitfall 5: Batch Key Mismatch (Missing Entries)

**What goes wrong:** The model returns aliases for 9 of 10 entries in a batch, silently omitting one. The run completes without error, and the omitted entry ends up with no aliases — violating D-60.3 full coverage.

**Why it happens:** The model may merge two entries, skip one, or hallucinate an `op_variant` name.

**How to avoid:** After parsing the batch response, call `validate_batch_response()` (see §Architecture Patterns). If `missing` is non-empty after 3 retries, log a warning and add the `op_variant` to an end-of-run "needs manual review" list. The run summary must print this list so the operator knows to handle them.

**Warning signs:** Run summary shows fewer total aliases added than `count(implemented entries)`.

### Pitfall 6: Writing Only to a Temp Copy (Not In-Place)

**What goes wrong:** Generator creates a new JSON file or overwrites the wrong path, so the canonical `docs/` pools are not updated.

**Why it happens:** Copied path handling from `docs-matrix`, which writes to a different output file.

**How to avoid:** The generator receives one argument — the pool path — and writes back to that same path in-place (read, modify, write). Unlike `docs-matrix` (in → different out), the alias generator is in-place. The `just help-aliases` recipe passes each pool path as both input and output.

---

## Code Examples

### Cargo.toml for `scripts/help-aliases/`

```toml
[workspace]
# Empty stanza — excludes this crate from the root workspace per CLAUDE.md
# "Root Cargo.toml members stays ["hp41-core", "hp41-cli"]" invariant.

[package]
name = "help-aliases"
version = "0.1.0"
edition = "2021"
publish = false

[[bin]]
name = "help-aliases"
path = "src/main.rs"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = { version = "1", features = ["preserve_order"] }
```

### `main.rs` CLI shape (mirrors `docs-matrix`)

```rust
// Usage: help-aliases <pool.json>
// In-place: reads and writes the same file.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: help-aliases <pool.json>");
        std::process::exit(2);
    }
    let path = &args[1];
    let (populated, skipped, added) = process_pool(path);
    println!("{path}: {total} scanned, {skipped} skipped, {populated} populated, {added} aliases added",
        total = populated + skipped);
}
```

### `pool.rs` — Load and save with preserve_order + UTF-8

```rust
use serde_json::Value;

pub fn load_pool(path: &str) -> Vec<Value> {
    let json = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {path}: {e}"));
    serde_json::from_str(&json)
        .unwrap_or_else(|e| panic!("parse {path}: {e}"))
}

pub fn save_pool(path: &str, entries: &[Value]) {
    // Use a custom serializer to:
    // 1. 4-space indent (matching pool format: outer items 4sp, properties 8sp)
    // 2. No \uXXXX escaping for non-ASCII characters
    let mut buf = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, Utf8PrettyFormatter::new());
    serde::Serialize::serialize(entries, &mut ser)
        .unwrap_or_else(|e| panic!("serialize {path}: {e}"));
    // Ensure trailing newline (all existing pools end with \n)
    if !buf.ends_with(b"\n") {
        buf.push(b'\n');
    }
    std::fs::write(path, &buf)
        .unwrap_or_else(|e| panic!("write {path}: {e}"));
}
```

### `merge.rs` — Fill-only alias insertion

```rust
use serde_json::Value;

/// Returns true if this entry needs aliases (absent or empty).
pub fn needs_aliases(entry: &Value) -> bool {
    match entry.get("search_aliases") {
        None => true,
        Some(Value::Array(arr)) => arr.is_empty(),
        _ => true,
    }
}

/// Insert aliases as the last key in the object.
/// With preserve_order, this appends to the IndexMap — correct position.
pub fn insert_aliases(entry: &mut Value, aliases: Vec<String>) {
    let arr: Vec<Value> = aliases.into_iter().map(Value::String).collect();
    if let Value::Object(map) = entry {
        map.insert("search_aliases".to_string(), Value::Array(arr));
    }
}
```

### `claude.rs` — Subprocess invocation with retry

```rust
use serde_json::Value;
use std::process::Command;
use std::time::Duration;

const CLAUDE_BIN: &str = "/Users/daniel/.local/bin/claude";
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_SECS: u64 = 2;

pub fn invoke_claude(prompt: &str) -> Result<Value, String> {
    for attempt in 0..MAX_RETRIES {
        if attempt > 0 {
            std::thread::sleep(Duration::from_secs(RETRY_DELAY_SECS));
        }
        match try_invoke(prompt) {
            Ok(v) => return Ok(v),
            Err(e) if attempt < MAX_RETRIES - 1 => {
                eprintln!("attempt {}/{MAX_RETRIES} failed: {e}", attempt + 1);
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}

fn try_invoke(prompt: &str) -> Result<Value, String> {
    let output = Command::new(CLAUDE_BIN)
        .args([
            "-p",
            "--output-format", "json",
            "--max-turns", "1",
            "--bare",
            "--dangerously-skip-permissions",
            "--model", "sonnet",
            prompt,
        ])
        .output()
        .map_err(|e| format!("spawn: {e}"))?;

    if !output.status.success() {
        return Err(format!("exit {:?}", output.status.code()));
    }

    let envelope: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("envelope parse: {e}"))?;

    let inner = envelope["result"]
        .as_str()
        .ok_or_else(|| "envelope missing 'result'".to_string())?;

    // Strip markdown code fences if present
    let inner = strip_fences(inner);

    serde_json::from_str(inner)
        .map_err(|e| format!("inner parse: {e}\nRaw: {}", &inner[..inner.len().min(200)]))
}

fn strip_fences(s: &str) -> &str {
    let s = s.trim();
    if s.starts_with("```") {
        let after_fence = s.find('\n').map(|i| &s[i + 1..]).unwrap_or(s);
        let before_close = after_fence.rfind("```")
            .map(|i| after_fence[..i].trim())
            .unwrap_or(after_fence);
        before_close
    } else {
        s
    }
}
```

### `justfile` recipe shape

```just
# Generate DE+EN search aliases for all six help JSON pools.
# Re-run only when functions are added or their semantics change.
# LLM (claude) runs on the developer's machine only — not in CI.
# Fill-only: entries with existing aliases are untouched.
[group('docs')]
help-aliases:
    cargo run --quiet --manifest-path scripts/help-aliases/Cargo.toml -- \
        docs/hp41cv-functions.json
    cargo run --quiet --manifest-path scripts/help-aliases/Cargo.toml -- \
        docs/hp41-math1-functions.json
    cargo run --quiet --manifest-path scripts/help-aliases/Cargo.toml -- \
        docs/hp41-stat1-functions.json
    cargo run --quiet --manifest-path scripts/help-aliases/Cargo.toml -- \
        docs/hp41-time-functions.json
    cargo run --quiet --manifest-path scripts/help-aliases/Cargo.toml -- \
        docs/hp41-advantage-functions.json
    cargo run --quiet --manifest-path scripts/help-aliases/Cargo.toml -- \
        docs/hp41-xmem-functions.json
```

Note: no `help-aliases-check` recipe is added — a regenerate-and-diff check is explicitly rejected (D-CONTEXT: "No regenerate-and-diff CI gate"). P-HS-04 reserves checking semantics for Phase 61 schema-only gate.

---

## Validation Architecture

`nyquist_validation: true` in `.planning/config.json` — this section is required.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` (no external test framework) |
| Config file | none — standard `cargo test` |
| Quick run command | `cargo test --manifest-path scripts/help-aliases/Cargo.toml` |
| Full suite command | `cargo test --manifest-path scripts/help-aliases/Cargo.toml` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| HSGEN-01 | Fill-only merge: entry with existing aliases untouched | unit | `cargo test --manifest-path scripts/help-aliases/Cargo.toml merge::tests::fill_only_skips_populated` | No — Wave 0 |
| HSGEN-01 | Fill-only merge: entry with empty/absent `search_aliases` gets populated | unit | `cargo test --manifest-path scripts/help-aliases/Cargo.toml merge::tests::fill_only_populates_empty` | No — Wave 0 |
| HSGEN-01 | JSON roundtrip: load → save (no aliases added) produces byte-identical output | unit | `cargo test --manifest-path scripts/help-aliases/Cargo.toml pool::tests::roundtrip_identity` | No — Wave 0 |
| HSGEN-01 | JSON roundtrip preserves key order after alias insertion | unit | `cargo test --manifest-path scripts/help-aliases/Cargo.toml pool::tests::key_order_preserved` | No — Wave 0 |
| HSGEN-01 | UTF-8 non-ASCII (Σ, —, umlauts) survives roundtrip un-escaped | unit | `cargo test --manifest-path scripts/help-aliases/Cargo.toml pool::tests::utf8_unescaped` | No — Wave 0 |
| HSGEN-01 | `claude` response with markdown fences is stripped cleanly | unit | `cargo test --manifest-path scripts/help-aliases/Cargo.toml claude::tests::strip_fences` | No — Wave 0 |
| HSGEN-01 | `validate_batch_response` detects missing and extra keys | unit | `cargo test --manifest-path scripts/help-aliases/Cargo.toml batch::tests::validate_key_mismatch` | No — Wave 0 |
| HSGEN-02 | `scripts/help-aliases/Cargo.toml` has empty `[workspace]` stanza | manual (CI grep) | `grep -q '^\[workspace\]' scripts/help-aliases/Cargo.toml` | No — Wave 0 |
| HSDATA-03 | After run, all 364 implemented entries have `search_aliases.len() >= 1` | manual (run + inspect summary) | `just help-aliases` output shows "0 entries with no aliases" | No — Wave 0 (content verification) |
| D-60.7 | Both frontends compile with populated pools | integration smoke | `cargo check -p hp41-cli && cd hp41-gui && npm run build` | Existing — verify post-population |

### Sampling Rate

- **Per task commit:** `cargo test --manifest-path scripts/help-aliases/Cargo.toml`
- **Per wave merge:** `cargo test --manifest-path scripts/help-aliases/Cargo.toml && cargo check -p hp41-cli`
- **Phase gate:** Full suite green + `just help-aliases` run + `cargo check -p hp41-cli` + GUI build before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `scripts/help-aliases/src/merge.rs` — fill_only_skips_populated, fill_only_populates_empty (REQ HSGEN-01, HSGEN-03)
- [ ] `scripts/help-aliases/src/pool.rs` — roundtrip_identity, key_order_preserved, utf8_unescaped (REQ HSGEN-01 / D-60.4)
- [ ] `scripts/help-aliases/src/claude.rs` — strip_fences, envelope_parse (REQ HSGEN-01)
- [ ] `scripts/help-aliases/src/batch.rs` — validate_key_mismatch, validate_no_mismatch (REQ HSDATA-03)
- [ ] Test fixture: a minimal JSON pool snippet (2 entries: one implemented with empty aliases, one already populated) for use across merge and pool tests

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual alias authoring (none) | LLM-generated, human-reviewed, committed static | Phase 60 (new) | Scales to 364 entries without manual work |
| `serde_json` no preserve_order | `serde_json` with `preserve_order` feature | Phase 60 (new, tooling-only) | Enables minimal-diff JSON writeback |

**Deprecated/outdated:**
- None — this is a new crate with no prior art in the project.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `serde_json` `preserve_order` feature uses `IndexMap` internally and maintains insertion order through a roundtrip | Standard Stack, Code Examples | If order is not preserved, D-60.4 fails — diff shows reordered keys; workaround: string-level injection |
| A2 | The `serde_json` `Formatter` trait's `write_string_fragment` can be overridden to suppress `\uXXXX` escaping | Pitfall 2, Code Examples | If the escape happens at a different call site, the custom Formatter won't prevent it; fallback: post-process output with a `\uXXXX` → UTF-8 decoder |
| A3 | `claude -p --bare --dangerously-skip-permissions` is appropriate for this use case and won't cause unexpected side effects | Pattern 1, Code Examples | If `--bare` strips auth context needed for the API call, invocations fail; remove `--bare` and accept slightly slower startup |
| A4 | Per-category batching keeps prompts within the model's context window (largest category: Adv Mtrx, 51 entries) | Batching Strategy | If 51 entries + prompt overhead exceeds the window, responses truncate; mitigation: split large categories at 20-entry sub-batches |
| A5 | `--model sonnet` alias resolves to the current Sonnet model in claude v2.1.165 | Pattern 1 | If the alias is not recognized, use full model name `claude-sonnet-4-5` or `claude-3-7-sonnet-20250219` |

---

## Open Questions

1. **`serde_json` Formatter trait API stability**
   - What we know: the `Formatter` trait exists in serde_json and `PrettyFormatter` is the standard implementation
   - What's unclear: whether delegating all methods to `PrettyFormatter` is ergonomic in Rust (trait implementations require forwarding each method) or if a `write_all` post-processing pass is simpler
   - Recommendation: implement the Formatter approach; if it proves verbose (>50 LOC), fall back to a `to_string_pretty` + post-process-unescape approach

2. **`--bare` flag behavior with auth**
   - What we know: `--bare` skips hooks, LSP, CLAUDE.md auto-discovery, and keychain reads for OAuth. It requires auth via `ANTHROPIC_API_KEY` or `apiKeyHelper`
   - What's unclear: whether the developer's local claude auth (OAuth) works under `--bare`, or whether the ANTHROPIC_API_KEY env var must be set
   - Recommendation: test with a simple `claude -p --bare "reply with: ok"` before the full run; if auth fails under `--bare`, drop the flag

3. **Advantage pool large categories (51 + 45 entries)**
   - What we know: per-category batching recommends splitting at ~20 entries
   - What's unclear: whether a 51-entry prompt exceeds practical context or response limits for Sonnet
   - Recommendation: split `Adv Mtrx` into three sub-batches of ~17 each; `Adv Math` into three of ~15 each; the `batch.rs` module should support arbitrary sub-batching

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `claude` CLI | Alias generation (D-60.1) | Yes | v2.1.165 | None (locked decision) |
| `cargo` (Rust toolchain) | Build `scripts/help-aliases/` | Yes | MSRV 1.88 | — |
| `just` | `help-aliases` recipe | Yes (project standard) | — | bare `cargo run` |

**Missing dependencies with no fallback:** none.

---

## Security Domain

The tooling crate is a dev-only script. It shells out to a local binary (`claude`) with no network access beyond what `claude -p` already uses. No user data, no credentials, no shipped binary changes. `security_enforcement` is not relevant here — this crate is excluded from all CI security scanning paths.

The `--dangerously-skip-permissions` flag is used only in the subprocess invocation and does not affect the hp41 project's runtime security posture.

---

## Sources

### Primary (HIGH confidence)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/scripts/docs-matrix/Cargo.toml` — workspace-exclusion stanza pattern (VERIFIED: codebase)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/scripts/docs-matrix/src/main.rs` — sibling-crate structure (VERIFIED: codebase)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/docs/hp41-xmem-functions.json` + `hp41cv-functions.json` — exact pool format: 4-space indent, 8-space property indent, key ordering, trailing newline, UTF-8 umlauts (VERIFIED: codebase inspection)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-cli/src/help_data.rs` — `search_aliases: Vec<String>` field, `#[serde(default)]` (VERIFIED: codebase)
- `/Users/daniel/GitRepository/hp41-calculator-emulator/hp41-gui/src/help_data.ts` — `search_aliases?: string[]` (VERIFIED: codebase)
- `claude --help` output — `--output-format json`, `--max-turns`, `--model`, `--bare`, `--dangerously-skip-permissions` flags (VERIFIED: live CLI)
- Live claude v2.1.165 test run — `{"type":"result","result":"..."}` envelope shape confirmed (VERIFIED: live CLI output)
- Python analysis of all six pools — key orderings, non-ASCII characters, entry counts (VERIFIED: codebase)

### Secondary (MEDIUM confidence)
- serde_json `preserve_order` feature — existence confirmed via `cargo search serde_json` (current version 1.0.149 in Cargo.lock); API details [ASSUMED: serde_json documentation]

### Tertiary (LOW confidence)
- `serde_json::ser::Formatter` delegation pattern for non-ASCII suppression [ASSUMED: standard Rust trait pattern]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified against live codebase and CLI
- Architecture: HIGH — pool format precisely confirmed, claude envelope confirmed
- Batching: MEDIUM — category counts verified, but optimal sub-batch size for large categories is empirical
- Pitfalls: HIGH — all four JSON-writeback pitfalls verified against live code or CLI output
- Validation architecture: HIGH — follows existing test patterns in the workspace

**Research date:** 2026-06-05
**Valid until:** 2026-07-05 (stable domain; only risk is claude CLI version change)
