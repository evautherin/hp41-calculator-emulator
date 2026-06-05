# Phase 60: Alias Authoring Pipeline - Context

**Gathered:** 2026-06-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Build the **offline, dev-only alias-authoring pipeline** and use it to **populate `search_aliases` with DE+EN content across all six JSON pools**. This is the *data + generator* phase: the `search_aliases` field already exists (Phase 58) and the runtime matcher already reads it (Phase 59) — Phase 60 fills it with content and ships the tool that produced it. No matcher, UX, CI-gate, or test work here.

**In scope (HSGEN-01, HSGEN-02, HSGEN-03, HSDATA-03):**
- New standalone tooling crate `scripts/help-aliases/` (sibling of `scripts/docs-matrix/`) that, for each `status:"implemented"` JSON entry, reads `display_name` / `description` / `notes` / `category`, asks a large LLM for N DE+EN aliases, and writes them back into the entry's `search_aliases`.
- The LLM is invoked **only at authoring time on the developer's machine** — nothing about it (model, API call, inference) is present in any shipped binary or runs at runtime.
- Run the generator and **commit the populated `search_aliases`** as static, PR-reviewable data: **every** `status:"implemented"` entry across all 6 pools (~364 entries) ends with ≥ 1 alias.
- A `just help-aliases` recipe (mirroring `just docs-matrix`) and a short README/doc note: re-run only when functions are added or their semantics change.

**Out of scope (other phases — do NOT pull in):**
- The `search_aliases` field definition on either mirror (HSDATA-01/02 → **Phase 58, done**).
- Any matcher / scoring / fuzzy / ranking / `HelpRow` change (HSMATCH-*, P-HS-01/02 → **Phase 59, done**).
- The schema-only CI gate that *enforces* "≥ 1 alias per implemented entry" (HSQUAL-03), the parity fixture (HSQUAL-02), scoring/fuzzy/DE-EN unit tests (HSQUAL-01), and the `CLAUDE.md` doc update (HSQUAL-04) → **Phase 61**. Phase 60 *delivers* full coverage; Phase 61 *guards* it.
- Any change to `hp41-core/`, `CalcState`, or save-file format (milestone invariant — `search_aliases` is documentation data, not state).
- A regenerate-and-diff CI gate (explicitly rejected — LLM output is non-deterministic).
- DE alias quality being held to the English-only doc rule (search input ≠ documentation; DE aliases are intentional and in-scope).

</domain>

<decisions>
## Implementation Decisions

### Generator mechanism
- **D-60.1:** The generator is a **standalone Rust crate `scripts/help-aliases/`** that **shells out to the local `claude -p` headless CLI** to produce aliases. Chosen over a direct Anthropic-API call (Rust or Python) and over a fully manual loop. Rationale: mirrors the existing `scripts/docs-matrix/` Rust-crate convention (empty `[workspace]` stanza → excluded from root members per the workspace invariant); **no API key/secret** and **no new runtime dependency** (the `claude` binary is already on the dev machine — confirmed `/Users/daniel/.local/bin/claude` v2.1.165); automatable across all ~364 entries with reviewable diffs. Accepted trade-off: reproducibility depends on the local `claude` install — acceptable for a tool that is **re-run only when functions change** (HSGEN-03). The crate reads `display_name` / `description` / `notes` / `category` per entry, invokes `claude -p` with a JSON-returning prompt, parses + validates the response, and writes `search_aliases` back into the pool file.

### Re-run / merge semantics
- **D-60.2:** **Fill-only, never overwrite.** On every run the generator populates `search_aliases` **only** for entries whose `search_aliases` is empty or absent; any entry that already has aliases is left byte-for-byte untouched. This directly satisfies the success criterion "data committed static — re-run only when functions are added" and protects any hand-curated aliases from a later run. (No change-detection/source-hash manifest — rejected as over-machinery for a rarely-run tool.)

### Coverage is the deliverable
- **D-60.3:** Phase 60's exit bar is **full data coverage**: after the generator runs and the developer commits, **every** `status:"implemented"` entry in all six pools (`hp41cv` + `math1` + `stat1` + `time` + `advantage` + `xmem`) has **≥ 1** `search_aliases` entry. Non-implemented entries (`status != "implemented"`) are **left untouched** — no aliases. The **schema CI gate that enforces this invariant lives in Phase 61 (HSQUAL-03)** and must NOT be built here; Phase 60 produces the data the gate will later guard.

### Minimal-diff JSON write discipline
- **D-60.4:** The generator must write each pool back so the **only** change is the added `search_aliases` arrays — preserve existing key order, 2-space indentation, UTF-8 (umlauts un-escaped where the existing files are), and trailing-newline style of the current `docs/hp41-*-functions.json` files. The aim is a PR diff a human can actually review per the spec's "reviewed in the PR like any other content" requirement. (Note: `serde_json` reorders/escapes by default — the planner must decide how to achieve a minimal diff: a serde-preserving writer, an ordered/`preserve_order` map, or targeted in-place insertion. This is a load-bearing implementation concern, flagged for the planner.)

### Human-review affordance
- **D-60.5:** Generated aliases get a **human review pass in the PR** (spec risk: a bad alias can pull the wrong function to the top of search). The generator should make review cheap — stable formatting (D-60.4), one pool per file change, and a short run summary (entries touched / aliases added per pool). The generator does **not** need an automated quality judge; the reviewer is the developer.

### Authoring-time-only guarantee
- **D-60.6:** The `claude`/LLM dependency exists **only** inside `scripts/help-aliases/` (a non-shipped tooling crate, excluded from the root workspace). It must NOT appear in `hp41-core`, `hp41-cli`, `hp41-gui`, or any `just gui-build`/release path. HSGEN-02 is verified by construction: the shipped binaries `include_str!` / Vite-import *static JSON only* — there is no model, API client, or `claude` invocation anywhere on a runtime path.

### Bundle-size sanity (P-HS-05)
- **D-60.7:** Populating ~364 entries with DE+EN aliases grows the six JSON pools (~90 KB today). The plan must include a **post-population sanity check** that both frontends still build with the larger `include_str!` (Rust) / static Vite import (TS) payload, and a note of the new total size. No hard budget is set, but a surprising blow-up should surface before commit.

### Claude's Discretion
- Exact prompt wording and system instruction handed to `claude -p`; the target alias **count N per entry** (spec says "N"; a small range like 4–8 DE+EN combined is reasonable) and the DE/EN split.
- **Batch granularity** of `claude -p` calls — per-entry vs per-category vs per-pool (one prompt returning an `op_variant → [aliases]` map). Batching is encouraged to bound CLI invocations and cost, provided the returned JSON is validated key-by-key before merge and unmatched/missing keys are reported, not silently dropped.
- Crate file layout, CLI argument shape (e.g. `help-aliases <pool.json>` per-pool like `docs-matrix`, vs an all-pools driver), `claude -p` flags (model, output-format), and the JSON (re)writer approach that achieves the minimal diff in D-60.4.
- Whether the `just help-aliases` recipe regenerates all six pools in one invocation or one-per-pool (follow the `docs-matrix` recipe shape).
- The exact aliases chosen for each function (subject to the human review pass).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design contract (read first)
- `docs/superpowers/specs/2026-06-03-help-search-enrichment-design.md` — approved milestone design. **§"Component 2 — Authoring pipeline" (lines 105–119)** is the spec for this phase: new `scripts/help-aliases/` script, reads `display_name`/`description`/`notes`/`category`, "calls a large LLM once on the developer's machine", "the LLM is an authoring tool, never shipped", "output is committed static data, reviewed in the PR", "no regenerate-and-diff CI gate", "re-run only when functions are added". §"Data model" (lines 78–103) anchors the field shape + the DE-aliases-are-intentional exception. Lines 176–178 list the risks (alias quality needs PR review; JSON size growth).

### Requirements & roadmap
- `.planning/REQUIREMENTS.md` — HSGEN-01 / HSGEN-02 / HSGEN-03 + HSDATA-03 are this phase. HSDATA-04 (invisible surface) and HSQUAL-01..04 are explicitly elsewhere (58/61).
- `.planning/ROADMAP.md` §"Phase 60" — goal + the three numbered success criteria (runnable `scripts/help-aliases/` script; LLM only on dev machine, nothing shipped; every implemented entry has ≥1 alias, committed static, re-run only when functions are added).
- `.planning/STATE.md` §"Accumulated Context" — pre-resolved milestone decisions (DE aliases intentional; no regenerate-diff gate; `scripts/help-aliases/` follows `docs-matrix`; phase dependency order). **Pitfall table: P-HS-05 is the Phase-60 one** (JSON size growth — verify `include_str!` / TS import still compiles within bundle-size limits). P-HS-04 (`just schema-check` must not call the LLM script) is Phase 61 but constrains the recipe naming here.

### Pattern to mirror (the sibling tooling crate)
- `scripts/docs-matrix/Cargo.toml` — the empty `[workspace]` stanza + `publish = false` + `serde`/`serde_json` deps idiom to copy for `scripts/help-aliases/Cargo.toml`.
- `scripts/docs-matrix/src/main.rs` — the JSON-pool read → transform → write structure and the per-pool CLI-arg shape.
- `justfile` (recipes `docs-matrix` ~line 201, `docs-matrix-check` ~line 218) — the recipe shape to mirror for a new `help-aliases` recipe (six explicit per-pool `cargo run --manifest-path scripts/help-aliases/Cargo.toml -- …` lines, or a single all-pools driver).
- `scripts/check-free42-contamination.sh`, `scripts/check-tauri-permissions.sh` — existing dev-script conventions in `scripts/`.

### Data the generator reads & writes (all six pools)
- `docs/hp41cv-functions.json` (154 entries / 136 implemented), `docs/hp41-math1-functions.json` (45/45), `docs/hp41-stat1-functions.json` (26/26), `docs/hp41-time-functions.json` (35/35), `docs/hp41-advantage-functions.json` (114/114), `docs/hp41-xmem-functions.json` (8/8). **Total ≈ 364 implemented entries.** Entry keys present: `op_variant`, `display_name`, `category`, `status`, `phase`, `key_path`, `description`, `example`, `notes`, and optionally `divergences` / `xrom`.

### Field target shape (already defined — do not redefine)
- `hp41-cli/src/help_data.rs` — `search_aliases: Vec<String>` with `#[serde(default)]` (the Rust deserialization target). The CLI `include_str!`s all six pools here.
- `hp41-gui/src/help_data.ts` — `search_aliases?: string[]` (the TS/Vite import target).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`scripts/docs-matrix/` is a complete working template** for a `scripts/`-resident Rust tooling crate that reads/writes the `docs/hp41-*-functions.json` pools: copy its `Cargo.toml` workspace-exclusion stanza, its `serde`/`serde_json` deps, its per-pool CLI shape, and its `justfile` recipe pattern.
- **`claude -p` headless CLI** is already installed on the dev machine (`/Users/daniel/.local/bin/claude`, v2.1.165) — the generator shells out to it; no install step, no API key.
- **The `search_aliases` field already deserializes** in both mirrors (`#[serde(default)]` Rust / optional TS), so older pools without the field parse cleanly and the generator can write the field additively.

### Established Patterns
- **`scripts/` tooling crates are excluded from the root workspace** (`["hp41-core", "hp41-cli"]` invariant). `scripts/help-aliases/Cargo.toml` must carry the same empty `[workspace]` stanza so `cargo` at the repo root never tries to build it — keeps the LLM/`claude` dependency out of every shipped build (D-60.6, HSGEN-02).
- **`just` is the sole task runner** — wire the generator as a `just help-aliases` recipe; never document a bare `cargo run`. Avoid the name `schema-check`/`*-check` for the generator recipe (P-HS-04 reserves checking semantics for the non-LLM Phase-61 gate).
- **Additive JSON, careful writeback** — every prior pool field extension kept older JSON parsing; here the additive change is *data*, and the diff must stay alias-only (D-60.4).

### Integration Points
- The generator **writes** `search_aliases`; Phase 59's matcher already **reads** it (CLI `filter_help_rows` + GUI overlay filter). After population, the previously-deferred DE/typo queries (`Zineszins`, `Wurzel`, `compound interest`) should resolve — but **proving that is Phase-61 verification**, not a Phase-60 deliverable. Phase 60 stops at "data committed + generator runnable".
- **Both frontends `include_str!` / import the pools at build time** — after population, both must still compile with the larger payload (D-60.7 / P-HS-05).

</code_context>

<specifics>
## Specific Ideas

- **Field name is fixed:** `search_aliases` (do not rename).
- **Worked example from the spec (the intended shape — mixed DE+EN strings):** TVM →
  `["Zinseszins", "Annuität", "Tilgung", "Kredit", "compound interest", "time value of money", "loan payment", "mortgage"]`.
  Aliases are lowercase-insensitive search terms / paraphrases, not documentation prose; DE umlauts are kept as real characters (matching the rest of the JSON), not escaped.
- **Generator output should report**, per pool: entries scanned, entries already-populated (skipped, D-60.2), entries newly populated, aliases added. This makes the "fill-only" behavior and full-coverage claim auditable at the terminal and in the PR.

</specifics>

<deferred>
## Deferred Ideas

- **Schema-only CI gate** enforcing "≥ 1 alias per implemented entry" + field type (HSQUAL-03) — **Phase 61**. Phase 60 produces the data; the gate guards it.
- **CLI↔GUI parity fixture** on a frozen alias snapshot (HSQUAL-02, P-HS-03) — **Phase 61**.
- **Scoring / fuzzy / DE-EN resolution unit tests** (HSQUAL-01) and **`CLAUDE.md` JSON-canonical-data-flow doc update** (HSQUAL-04) — **Phase 61**.
- **Missed-query logging** for truly-unseen vocabulary (HSCOV-01) — explicitly **v2 / out of scope** for v4.2.
- **Change-detection / source-hash re-run** — considered for D-60.2 and rejected in favor of fill-only.

</deferred>

---

*Phase: 60-alias-authoring-pipeline*
*Context gathered: 2026-06-05*
