---
phase: 60-alias-authoring-pipeline
plan: "01"
subsystem: tooling
tags: [help-search, alias-generation, dev-tooling, json-writeback, preserve-order]
dependency_graph:
  requires: []
  provides: [scripts/help-aliases crate, just help-aliases recipe]
  affects: [docs/hp41-*-functions.json (Wave 2 data run), Phase 61 schema gate]
tech_stack:
  added:
    - "serde_json preserve_order feature (IndexMap-backed Value::Object for key-order roundtrip)"
  patterns:
    - "Custom Utf8PrettyFormatter delegating to PrettyFormatter, overrides write_string_fragment for literal UTF-8"
    - "Two-pass claude envelope parse: outer {type/result} -> inner alias JSON"
    - "Fill-only merge: needs_aliases check before insert_aliases (D-60.2)"
    - "Per-category batch grouping with auto-split at MAX_BATCH_SIZE=20"
key_files:
  created:
    - scripts/help-aliases/Cargo.toml
    - scripts/help-aliases/src/main.rs
    - scripts/help-aliases/src/pool.rs
    - scripts/help-aliases/src/merge.rs
    - scripts/help-aliases/src/claude.rs
    - scripts/help-aliases/src/batch.rs
    - scripts/help-aliases/Cargo.lock
    - scripts/help-aliases/README.md
  modified:
    - justfile (help-aliases recipe added under [group('docs')])
    - .gitignore (scripts/help-aliases/target/ exclusion)
decisions:
  - "Utf8PrettyFormatter: delegating wrapper over PrettyFormatter overriding write_string_fragment only (~15 methods forwarded — within 50 LOC budget, no fallback needed)"
  - "fill_aliases helper marked #[allow(dead_code)] — used by tests and available for wave 2 callers; main.rs drives pipeline inline"
  - "Main.rs WorkItem struct snapshots idx+op_variant from entries before building worklist_values clones — resolves borrow checker conflict between immutable worklist refs and mutable entries"
  - "strip_fences searches for first ``` occurrence (handles leading prose + fence), then rfind closing fence — handles both json-tagged and untagged fences"
  - "Cargo.lock committed (mirrors docs-matrix convention for binary crates)"
metrics:
  duration: "~30 minutes"
  completed: "2026-06-05"
  tasks_completed: 3
  files_created: 10
  files_modified: 2
---

# Phase 60 Plan 01: help-aliases Crate Scaffold Summary

Dev-only `scripts/help-aliases/` Rust crate with minimal-diff JSON pool I/O (Utf8PrettyFormatter + preserve_order), fill-only merge, `claude -p` wrapper with two-pass envelope parsing and fence stripping, per-category batching with missing/extra key validation, and the `just help-aliases` recipe — all covered by 12 deterministic unit tests, zero live LLM calls.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Crate scaffold + pool I/O (pool.rs) | b58dc8e | Cargo.toml, src/pool.rs, src/main.rs (+ stubs) |
| 2 | Fill-only merge (merge.rs) | b58dc8e | src/merge.rs (committed with Task 1 stubs) |
| 3 | claude.rs + batch.rs + justfile + README | 78b80fe | README.md, Cargo.lock, justfile, .gitignore |

Note: Tasks 1 and 2 were committed together in a single atomic commit because all four module files (pool, merge, claude, batch) were required simultaneously for the crate to compile. The Task 2 tests (merge::tests) were fully implemented and green in that commit.

## Test Suite

All 12 tests pass with `cargo test --manifest-path scripts/help-aliases/Cargo.toml`:

| Module | Tests | Status |
|--------|-------|--------|
| pool::tests | roundtrip_identity, key_order_preserved, utf8_unescaped | 3/3 PASS |
| merge::tests | fill_only_skips_populated, fill_only_populates_empty, skips_non_implemented | 3/3 PASS |
| claude::tests | strip_fences_works, envelope_parse_works, envelope_parse_with_fenced_inner | 3/3 PASS |
| batch::tests | validate_key_mismatch, validate_no_mismatch, large_category_split | 3/3 PASS |

Zero live LLM calls on the test path — all tests use in-repo `&str` fixtures.

## What Was Built

### pool.rs — Minimal-diff JSON writeback (D-60.4)

`Utf8PrettyFormatter` wraps `PrettyFormatter::with_indent(b"    ")` and overrides `write_string_fragment` to emit non-ASCII bytes literally (no `\uXXXX` escaping). Combined with the `preserve_order` feature (IndexMap-backed `Value::Object`), a load → save roundtrip on an unchanged pool produces byte-identical output. Proven by `roundtrip_identity`, `utf8_unescaped`, and `key_order_preserved` tests.

### merge.rs — Fill-only semantics (D-60.2 + D-60.3)

`needs_aliases(entry)` returns true when `search_aliases` is absent, empty, or malformed. `insert_aliases(entry, aliases)` appends the key last (IndexMap append = last position with `preserve_order`). `fill_aliases` walks a pool selecting only `status == "implemented"` AND `needs_aliases == true` entries. Populated entries are skipped byte-for-byte untouched.

### claude.rs — subprocess wrapper

`invoke_claude(prompt)` retries up to 3 times with 2s delay. `parse_envelope(stdout: &[u8])` is a pure two-pass helper: outer envelope JSON → `.result` string → `strip_fences` → inner alias JSON. `strip_fences(s)` handles leading prose + optional ` ```json ` fencing. The `claude` binary is resolved via PATH (not hardcoded), keeping the crate portable. Tests target `parse_envelope` and `strip_fences` directly — no subprocess spawned.

### batch.rs — Per-category batching

`group_by_category` groups the worklist by the `category` field, auto-splitting categories with >20 entries into sub-batches named `"Category (1/N)"`. The two large Advantage categories (~51, ~45 entries) will each split into 3 sub-batches. `build_prompt` constructs the DE+EN alias generation prompt per RESEARCH §Prompt Design (4-8 aliases, German umlauts as real chars, JSON-only output). `validate_batch_response` reports both missing and extra keys — never silently drops.

### main.rs — Pipeline driver

Snapshots the worklist into `WorkItem {idx, op_variant}` structs before cloning entries for batch building — resolving the borrow checker conflict between immutable batch refs and mutable entry Vec. Drives the full pipeline: group_by_category → build_prompt → invoke_claude → validate_batch_response → insert_aliases → save_pool. Prints per-pool run summary (D-60.5) and a `[needs manual review]` list for any persistently missing entries.

### just help-aliases recipe

Under `[group('docs')]`, six in-place `cargo run --manifest-path scripts/help-aliases/Cargo.toml -- docs/<pool>.json` lines (single path argument = in-place read+write). Recipe comment states the re-run-only-when-functions-change policy, dev-machine-only LLM guarantee, and fill-only behavior. No `help-aliases-check` recipe (P-HS-04).

### README.md

Documents purpose, `just help-aliases` usage, re-run policy (HSGEN-03), fill-only behavior, authoring-time-only guarantee (HSGEN-02), pool file table, and PR review guidance.

## Deviations from Plan

### Tasks 1 and 2 committed atomically

**Found during:** Task 1

**Issue:** Creating a minimal `main.rs` stub that compiled required all four modules (`pool`, `merge`, `claude`, `batch`) to exist at compilation time. Creating only `pool.rs` and leaving the others for later tasks would have meant the crate wouldn't build for Task 1 verification.

**Fix:** Implemented all four modules in one session and committed them together. The Task 2 tests (merge::tests) were fully complete and green in that commit. Task 3 added only `README.md`, `Cargo.lock`, `justfile` changes, and `.gitignore`.

**Classification:** [Rule 3 - Blocking] — empty stubs were needed for compilation; implementing full modules rather than placeholder stubs was the correct fix.

## Known Stubs

None — all modules are fully implemented. `claude.rs::invoke_claude` references the `claude` binary via PATH which is only available on the dev machine; the Wave 2 data run will validate this works end-to-end.

## Threat Flags

None — this is a dev-only tooling crate with an empty `[workspace]` stanza. It is excluded from all shipped builds. No new network endpoints, auth paths, or runtime trust boundaries introduced.

## Self-Check: PASSED

Files created exist:
- [x] scripts/help-aliases/Cargo.toml
- [x] scripts/help-aliases/src/pool.rs
- [x] scripts/help-aliases/src/merge.rs
- [x] scripts/help-aliases/src/claude.rs
- [x] scripts/help-aliases/src/batch.rs
- [x] scripts/help-aliases/src/main.rs
- [x] scripts/help-aliases/README.md
- [x] scripts/help-aliases/Cargo.lock

Commits verified:
- [x] b58dc8e — scaffold (pool, merge, claude, batch, main)
- [x] 78b80fe — recipe, README, Cargo.lock, .gitignore

Plan criteria:
- [x] cargo build exits 0
- [x] cargo test: 12 tests all green, zero live LLM calls
- [x] just help-aliases recipe present, 6 per-pool in-place lines
- [x] no help-aliases-check recipe (grep count = 0)
- [x] README contains "re-run" policy
- [x] git diff --stat -- 'docs/hp41-*-functions.json' EMPTY
- [x] [workspace] stanza on line 1 of Cargo.toml
- [x] preserve_order in Cargo.toml
- [x] grep -c '"help-aliases"' Cargo.toml = 0 (not in root workspace)
