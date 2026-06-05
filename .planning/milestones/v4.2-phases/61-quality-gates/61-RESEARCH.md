# Phase 61: Quality Gates — Research

**Researched:** 2026-06-05
**Domain:** Test coverage, CI schema validation, CLI/GUI parity fixture, docs maintenance
**Confidence:** HIGH

## Summary

Phase 61 is a tests-only, CI-only, docs-only phase. No new product feature is added. The
goal is to lock down the correctness of the Phases 58–60 work: the `search_aliases` field,
the tiered scorer, and the 364 populated aliases across six JSON pools. Four deliverables
map one-to-one to the four HSQUAL requirements: unit tests in both frontends (HSQUAL-01),
a CLI-TS parity fixture (HSQUAL-02), a JSON schema CI gate (HSQUAL-03), and a CLAUDE.md
docs update (HSQUAL-04).

The existing Phase 59 tests (`phase59_help_search.rs` and the `help_data.test.ts` Phase 59
block) already cover the synthetic-data scoring contract. What Phase 61 adds is: (1) tests
against the REAL populated alias data to prove DE+EN alias resolution works end-to-end, (2)
a shared fixture file that asserts Rust and TS produce IDENTICAL ranked results for a
canonical query set, (3) a CI-gated bash script analogous to `scripts/check-free42-contamination.sh`
that validates all six pools, and (4) a small surgical CLAUDE.md edit.

The key algorithmic insight is that the Phase 59 tests are synthetic: they inject entries
with aliases like `"Zinseszins"` and `"Wurzel"` directly. Phase 61 must test against the
ACTUAL committed alias data. "Zinseszins" is not a TVM alias — "Zeitwert des Geldes" is.
"Wurzel" as an exact standalone alias is not in the SQRT entry either — "Wurzel ziehen",
"Wurzel aus X", "Wurzelfunktion", "Quadratwurzel" are. Tests must use queries that actually
match the real data.

**Primary recommendation:** Three plans — Wave 1: Rust unit tests + TS unit tests + Phase
61 parity fixture; Wave 2: schema-check script + ci.yml job + docs edit. The parity fixture
is a committed JSON file, and both Rust and TS tests load and assert it.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Unit scorer tests (Rust) | CLI (`hp41-cli`) | — | `score_entry`/`ranked_help_entries` live in `hp41-cli/src/help_data.rs` |
| Unit scorer tests (TS) | GUI (`hp41-gui`) | — | `scoreEntry`/`rankedEntries` live in `hp41-gui/src/help_data.ts` |
| Parity fixture (JSON) | Shared (`docs/` or `hp41-cli/tests/fixtures/`) | Consumed by both | Canonical truth read by both Rust and Vitest tests |
| Schema gate script | `scripts/` (bash) | ci.yml job | Follows `check-free42-contamination.sh` precedent |
| CLAUDE.md edit | Project root | — | Frozen-invariants section update |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust `#[test]` / cargo test | stable (MSRV 1.88) | Rust unit tests | Project standard; no new dep |
| Vitest | 4.1.6 (pinned in package-lock.json) [VERIFIED: package.json] | TS/TSX unit tests | Project standard for GUI |
| jq | 1.7.1 (pre-installed ubuntu-latest) [VERIFIED: local env] | JSON schema validation script | Already on ubuntu; no install step needed in CI |
| node | 22.16 (pre-installed ubuntu-latest) [VERIFIED: local env] | Alternative for schema script | GUI CI already uses setup-node |
| bash | system | CI gate script shell | Matches `check-free42-contamination.sh` |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde_json` | workspace (already in Cargo.toml) | Rust: read parity fixture JSON | Only if parity fixture is read from disk in Rust test |
| `include_str!` | n/a | Rust: embed parity fixture at compile time | Preferred over runtime file read in integration tests |
| `@testing-library/react` | 16.3.x (pinned) | Vitest: DOM tests | Already used; no change |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| bash + jq for schema gate | Node.js script | Node requires setup-node in ci.yml (not currently there); jq is pre-installed on ubuntu-latest, simpler setup |
| bash + jq for schema gate | Rust script crate | Rust crate would be more consistent with docs-matrix, but overkill for a 20-line JSON validation |
| Committed JSON parity fixture | Rust emits + TS asserts via `just` diff recipe | Emitting means non-reproducible if float precision differs; committed fixture is more auditable |

**Installation:** No new packages. Zero new runtime deps. Zero new dev deps.

## Package Legitimacy Audit

No external packages are installed by this phase. This section is not applicable.

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

## Architecture Patterns

### System Architecture Diagram

```
docs/*.json (6 pools)
        │
        ├──── Rust test (phase61_help_search_aliases.rs) ────► score + rank assertions
        │               │
        │               └── reads docs/fixtures/help_search_parity.json ──┐
        │                                                                  │
        ├──── TS test (help_data.test.ts Phase 61 block) ─────────────────┘
        │               │ (reads same fixture via import or fetch)        │
        │               └── scoreEntry + rankedEntries assertions          │
        │                                                                  │
        └──── scripts/check-aliases-schema.sh ─────────────────────────► ci.yml schema-aliases job
```

### Recommended Project Structure
```
docs/
└── fixtures/
    └── help_search_parity.json   # NEW: canonical query→ranked_results fixture

hp41-cli/tests/
└── phase61_help_search_aliases.rs   # NEW: Rust alias+parity tests

hp41-gui/src/
└── help_data.test.ts                # AMENDED: Phase 61 block appended

scripts/
└── check-aliases-schema.sh          # NEW: bash+jq schema validation

.github/workflows/
└── ci.yml                           # AMENDED: schema-aliases job added

CLAUDE.md                            # AMENDED: JSON canonical data flow section
```

### Pattern 1: Rust Integration Test Against Real JSON Data

**What:** Load the real six JSON pools via `ranked_help_entries` and assert the correct
entry appears for queries that exercise alias resolution with the actual alias data.
**When to use:** For end-to-end alias acceptance tests (HSQUAL-01 DE+EN alias cases).

```rust
// Source: hp41-cli/tests/phase59_help_search.rs (existing precedent)
// Phase 61 variant — uses real data, not synthetic make_entry()
#[test]
fn de_alias_zeitwert_resolves_tvm() {
    // "zeitwert des geldes" is an exact alias in docs/hp41-advantage-functions.json
    // for the TVM entry (AdvTvm). Exact alias score = 35.
    let results = ranked_help_entries("zeitwert des geldes");
    assert!(!results.is_empty(), "query 'zeitwert des geldes' must return results");
    assert_eq!(
        results[0].op, "TVM",
        "first result for 'zeitwert des geldes' must be TVM; got '{}'",
        results[0].op
    );
}

#[test]
fn de_alias_quadratwurzel_resolves_sqrt() {
    // "quadratwurzel" is an exact alias in docs/hp41cv-functions.json for SQRT.
    let results = ranked_help_entries("quadratwurzel");
    assert!(!results.is_empty());
    assert_eq!(results[0].op, "SQRT");
}

#[test]
fn en_alias_square_root_resolves_sqrt() {
    let results = ranked_help_entries("square root");
    assert!(!results.is_empty());
    assert_eq!(results[0].op, "SQRT");
}
```

### Pattern 2: Parity Fixture (HSQUAL-02)

**What:** A committed JSON file `docs/fixtures/help_search_parity.json` maps canonical
query strings to expected ranked `display_name` lists. Both Rust (`phase61_help_search_aliases.rs`)
and Vitest (`help_data.test.ts`) load this file and assert their ranked results match.

**Format:**
```json
{
  "_comment": "Canonical query → expected ranked display_names. Asserted by both Rust and TS tests.",
  "queries": [
    {
      "query": "tvm",
      "top_n": 1,
      "expected_top": ["TVM"],
      "note": "exact name match"
    },
    {
      "query": "zeitwert des geldes",
      "top_n": 1,
      "expected_top": ["TVM"],
      "note": "exact DE alias match on TVM"
    },
    {
      "query": "quadratwurzel",
      "top_n": 1,
      "expected_top": ["SQRT"],
      "note": "exact DE alias on SQRT"
    },
    {
      "query": "square root",
      "top_n": 1,
      "expected_top": ["SQRT"],
      "note": "exact EN alias on SQRT"
    },
    {
      "query": "financial solver",
      "top_n": 1,
      "expected_top": ["TVM"],
      "note": "exact EN alias on TVM"
    },
    {
      "query": "sqirt",
      "top_n": 1,
      "expected_top": ["SQRT"],
      "note": "fuzzy match on SQRT display_name (1 typo)"
    },
    {
      "query": "emdir",
      "top_n": 1,
      "expected_top": ["EMDIR"],
      "note": "exact name match in xmem pool"
    },
    {
      "query": "pi",
      "top_n": 1,
      "expected_top": ["PI"],
      "note": "exact name match — baseline regression"
    }
  ]
}
```

**Rationale for `top_n: 1` only:** The fixture asserts the top result. This avoids
brittleness from multi-way ties at lower ranks while still catching the most important
regressions. Pitfall P-HS-03 (alias edits silently break fixture) is mitigated by the
`_comment` and by the fact that the fixture uses only entries whose top-position is stable
regardless of other alias changes.

**Rust test pattern:**
```rust
// Source: precedent from function_matrix_parity.rs pattern
#[test]
fn parity_fixture_rust() {
    let fixture_json = include_str!("../../../docs/fixtures/help_search_parity.json");
    // parse with serde_json (already in workspace)
    let fixture: serde_json::Value = serde_json::from_str(fixture_json).unwrap();
    for query_case in fixture["queries"].as_array().unwrap() {
        let q = query_case["query"].as_str().unwrap();
        let expected_top = query_case["expected_top"].as_array().unwrap();
        let top_n = query_case["top_n"].as_u64().unwrap() as usize;
        let results = ranked_help_entries(q);
        for (i, expected_name) in expected_top.iter().take(top_n).enumerate() {
            assert_eq!(
                results.get(i).map(|r| r.op.as_str()).unwrap_or(""),
                expected_name.as_str().unwrap(),
                "Parity fixture query {:?}: position {} expected {:?}",
                q, i, expected_name
            );
        }
    }
}
```

**TS test pattern (in `help_data.test.ts`):**
```typescript
// Source: same fixture, loaded via Vite static import or fs.readFileSync
import parityFixture from '../../docs/fixtures/help_search_parity.json';

describe('Phase 61 parity fixture — CLI↔GUI drift guard', () => {
    it('all canonical queries produce expected top-N results', () => {
        for (const c of parityFixture.queries) {
            const pool = allFunctionsEntries();
            const q = c.query.toLowerCase().trim();
            const results = rankedEntries(pool, q);
            for (let i = 0; i < Math.min(c.top_n, c.expected_top.length); i++) {
                const got = results[i]?.display_name ?? '(no result)';
                expect(got, `query "${c.query}" position ${i}`).toBe(c.expected_top[i]);
            }
        }
    });
});
```

### Pattern 3: Schema Gate Script (HSQUAL-03)

**What:** `scripts/check-aliases-schema.sh` — bash + jq, modeled on `check-free42-contamination.sh`.
Validates six explicitly named JSON pool files (avoids the `hp41-*-functions.json` glob trap).

```bash
#!/usr/bin/env bash
# scripts/check-aliases-schema.sh
# CI gate: validates search_aliases field across all six help JSON pools.
# - Every entry's search_aliases (if present) must be an array of strings.
# - Every status:"implemented" entry must have search_aliases with >= 1 item.
# Does NOT regenerate — reads existing committed JSON only (P-HS-04).
set -euo pipefail

# Explicit file list — avoids the hp41cv glob trap (hp41cv-functions.json has
# no dash before 'functions', so docs/hp41-*-functions.json silently misses it).
POOLS=(
    "docs/hp41cv-functions.json"
    "docs/hp41-math1-functions.json"
    "docs/hp41-stat1-functions.json"
    "docs/hp41-time-functions.json"
    "docs/hp41-advantage-functions.json"
    "docs/hp41-xmem-functions.json"
)

FAIL=0
for pool in "${POOLS[@]}"; do
    if [[ ! -f "$pool" ]]; then
        echo "FAIL: $pool not found" >&2
        FAIL=1; continue
    fi

    # Count implemented entries missing search_aliases or with empty array
    missing=$(jq '[.[] | select(.status == "implemented") |
                   select(.search_aliases == null or (.search_aliases | length) == 0)] |
                  length' "$pool")
    if [[ "$missing" -gt 0 ]]; then
        echo "FAIL: $pool — $missing implemented entries lack search_aliases" >&2
        jq -r '.[] | select(.status == "implemented") |
               select(.search_aliases == null or (.search_aliases | length) == 0) |
               "  missing: " + .display_name' "$pool" >&2
        FAIL=1
    fi

    # Verify search_aliases is an array-of-strings (not an object or number)
    bad_type=$(jq '[.[] | select(.search_aliases != null) |
                   select(.search_aliases | type != "array")] | length' "$pool")
    if [[ "$bad_type" -gt 0 ]]; then
        echo "FAIL: $pool — search_aliases is not an array in some entries" >&2
        FAIL=1
    fi

    # Verify each alias within search_aliases is a string
    bad_items=$(jq '[.[] | select(.search_aliases != null) |
                    .search_aliases[] | select(type != "string")] | length' "$pool")
    if [[ "$bad_items" -gt 0 ]]; then
        echo "FAIL: $pool — non-string items found in search_aliases" >&2
        FAIL=1
    fi
done

if [[ "$FAIL" -eq 0 ]]; then
    echo "OK: search_aliases schema valid across all 6 JSON pools"
fi
exit "$FAIL"
```

**Wire into ci.yml:**
```yaml
schema-aliases:
  name: Schema gate (search_aliases)
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: taiki-e/install-action@v2
      with:
        tool: just
    - run: just schema-aliases-check
```

**New Justfile recipe:**
```just
# CI gate: validate search_aliases field presence + type across all six JSON pools.
# Schema-only — does NOT regenerate (P-HS-04: LLM output is non-deterministic).
[group('ci')]
schema-aliases-check:
    bash scripts/check-aliases-schema.sh
```

### Anti-Patterns to Avoid

- **Glob trap:** Using `docs/hp41-*-functions.json` in the schema script silently
  misses `docs/hp41cv-functions.json` (no dash before "functions"). Always enumerate
  all six files explicitly.
- **Regenerate-and-diff:** The schema gate must NEVER call `scripts/help-aliases/` and
  diff the output. LLM output is non-deterministic. Gate is schema-only (P-HS-04).
- **`include_str!` path confusion:** The Rust test path for the parity fixture is
  relative to the test file location, not the Cargo workspace root. Use
  `include_str!("../../../docs/fixtures/help_search_parity.json")` from
  `hp41-cli/tests/phase61_help_search_aliases.rs`.
- **TS fixture import path:** Vite's `fs.allow` already extends to the repo root
  (vite.config.ts line ~30). A `../../docs/fixtures/help_search_parity.json` import
  from `hp41-gui/src/` is already in the allowed path.
- **Parity fixture on `allFunctionsEntries()`:** The TS parity test must use
  `allFunctionsEntries()` (all 6 pools, status=="implemented", minus OVERLAY_HIDDEN_ALIASES)
  as the input pool to `rankedEntries()`, mirroring what `HelpOverlay.tsx` uses.
  Using only `helpEntries()` (cv pool only) would miss SQRT (cv has it, but TVM is in
  the advantage pool).
- **Vitest `globals: false` cleanup:** Any test that renders React components
  (`HelpOverlay`) needs `afterEach(cleanup)`. The Phase 61 parity test is a pure-data
  test (no DOM rendering), so cleanup is not needed there — but it must be included
  in any `describe` block that renders components (the HelpOverlay ranked-branch tests).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON schema validation | Custom JSON parser in Rust | bash + jq | jq is pre-installed on ubuntu runners; a Rust crate would add a new dep or a full script crate (overkill for this) |
| Parity assertion | Runtime Rust↔TS bridge | Committed JSON fixture | No runtime bridge exists; committed fixture is deterministic and auditable |
| Unicode-correct Levenshtein | Byte-counting `.len()` | Code-point counting (`chars().count()` / `[...str].length`) | Already implemented correctly in Phase 59; don't regress |

**Key insight:** jq is the right tool for a JSON schema gate — it's already available,
expressive for JSON traversal, and consistent with the bash-script pattern established
by `check-free42-contamination.sh`.

## Common Pitfalls

### Pitfall 1: Glob Trap — hp41cv Pool Silently Excluded
**What goes wrong:** A schema script using `docs/hp41-*-functions.json` processes only
5 files; `docs/hp41cv-functions.json` (no dash before "functions") is missed. The gate
passes even if the cv pool has schema errors.
**Why it happens:** The cv pool filename follows a different naming convention from the
five module pools.
**How to avoid:** Enumerate all six files explicitly in the `POOLS` array.
**Warning signs:** If the script processes exactly 5 files instead of 6, the cv pool is
being skipped.

### Pitfall 2: Parity Fixture vs. Alias Data Drift (P-HS-03)
**What goes wrong:** A future alias addition/removal causes the ranked output to change,
breaking the parity fixture assertion without any test guidance on what changed.
**Why it happens:** The fixture binds a query to an exact expected result using the
snapshot of alias data at fixture-authoring time.
**How to avoid:** Use queries whose top-1 result is extremely stable (exact name match
for TVM, PI, EMDIR; exact high-scoring alias match for SQRT). Avoid queries whose
top result depends on tie-breaking among multiple same-score aliases.
**Warning signs:** Parity fixture fails after a `just help-aliases` re-run; diagnose
by checking which alias changed.

### Pitfall 3: Phase 59 Tests Use Synthetic Data; Phase 61 Tests Use Real Data
**What goes wrong:** Writing Phase 61 alias tests that use `make_entry()` with
`search_aliases: &["Zinseszins", ...]` passes even if the real JSON data doesn't contain
those aliases, giving false confidence.
**Why it happens:** Phase 59 deliberately used synthetic data because alias content didn't
exist yet. Phase 61 must use `ranked_help_entries()` against the real loaded pools.
**How to avoid:** Phase 61 real-data tests call `ranked_help_entries("zeitwert des geldes")`
(not a synthetic pool). Use queries verified against the actual committed JSON.
**Warning signs:** A test passes with synthetic `make_entry()` but would fail if run against
`ranked_help_entries()` — always verify with the real function for Phase 61.

### Pitfall 4: "Zinseszins" is NOT a TVM Alias
**What goes wrong:** Success criteria mentions "Zineszins → TVM" from the design spec. The
actual Phase 60 TVM aliases are: `["TVM", "time value of money", "financial solver", "N I PV
PMT FV", "TVM menu", "Zeitwert des Geldes", "Finanzmathematik Löser", "Finanzrechner interaktiv"]`.
"Zinseszins" (compound interest) is NOT present. [VERIFIED: node.js against actual JSON]
**Why it happens:** The UAT deferred the "Zinseszins" query test to Phase 61 (per STATE.md).
The assumption was that Phase 60 would add it, but the generator used "Zeitwert des Geldes"
instead.
**How to avoid:** The PLAN must either: (a) accept that "Zinseszins" queries return 0 results
(document as a known gap for a future `just help-aliases` re-run), or (b) add "Zinseszins"
and "compound interest" manually to the TVM `search_aliases` in `docs/hp41-advantage-functions.json`
as part of this phase (a hand-edit, not LLM regeneration). Option (b) is recommended since
it was a stated UAT expectation.
**Warning signs:** `ranked_help_entries("zinseszins")` returns empty even after Phase 61.

### Pitfall 5: "Wurzel" Exact Alias NOT in SQRT — "Wurzel ziehen" Is
**What goes wrong:** The Phase 59 test (`fuzzy_wurzel_resolves_sqrt`) uses a synthetic
entry with alias `"Wurzel"` (exact match = 35). In real data, SQRT has aliases
`["Quadratwurzel", "Wurzel ziehen", "Wurzel aus X", "Wurzelfunktion"]`. A query of "wurzel"
matches via word-prefix (score 28) on "Wurzel ziehen" etc. — BUT so do several other entries
with "Wurzel" in their aliases (FSOLVE=28, RTS=28, Z^1/N=28, Z^1/W=28). SQRT is NOT
uniquely top-1 for "wurzel". [VERIFIED: node.js against actual JSON]
**Why it happens:** The "Wurzel" exact-alias test was designed for synthetic data;
real data has multi-word aliases. The parity fixture must NOT use "wurzel" as a
top-1 assertion — use "quadratwurzel" instead (exact alias = 35, SQRT clearly tops).
**Warning signs:** Parity fixture test using query "wurzel" fails or is ambiguous.

### Pitfall 6: ci-gui.yml Path Filter Excludes docs/ Changes
**What goes wrong:** The schema gate job is added to `ci-gui.yml`. Changes to `docs/*.json`
only touch `docs/` — ci-gui.yml is triggered only by `hp41-gui/**` and `hp41-core/**`
changes, so the gate would never fire on alias edits.
**Why it happens:** ci-gui.yml has a paths filter; ci.yml does not.
**How to avoid:** Add the `schema-aliases` job to `ci.yml` (no path filter), not `ci-gui.yml`.
The `license-audit` job in `ci.yml` is the precedent — it also has no `needs:` dependency.
**Warning signs:** After adding aliases to a JSON file in `docs/`, the schema-aliases CI
job does not appear in the PR check list.

### Pitfall 7: Rust Parity Test `include_str!` Path
**What goes wrong:** `include_str!("../../docs/fixtures/help_search_parity.json")` from
`hp41-cli/tests/phase61_help_search_aliases.rs` — the path must be relative to the Rust
source file, counting up through `tests/` and `hp41-cli/` to the repo root.
**Why it happens:** `include_str!` resolves relative to the source file, not the workspace root.
From `hp41-cli/tests/*.rs`, the correct path to `docs/fixtures/` is
`"../../../docs/fixtures/help_search_parity.json"` (up through tests/, hp41-cli/, then down).

### Pitfall 8: Vitest `globals: false` — `cleanup` Required in Component Tests
**What goes wrong:** Adding a Phase 61 scorer test alongside a component render test in the
same describe block without `afterEach(cleanup)` accumulates portaled DOM nodes.
**Why it happens:** Per CLAUDE.md and vite.config.ts, Vitest runs `globals: false`. Without
cleanup, portaled nodes from `HelpOverlay` renders leak between tests.
**How to avoid:** The Phase 61 parity test in `help_data.test.ts` is pure-data (no DOM
rendering), so no `cleanup` is needed there. Any Phase 61 block that renders `<HelpOverlay>`
must include `afterEach(cleanup)`.

## Code Examples

### Verified canonical queries for parity fixture [VERIFIED: node.js against live JSON]

The following queries produce stable, unambiguous top-1 results against the actual committed
alias data (verified by running the full tier-score algorithm against all 6 pools):

| Query | Top-1 | Score | Tier |
|-------|-------|-------|------|
| `"tvm"` | TVM | 40 | exact name |
| `"pi"` | PI | 40 | exact name |
| `"emdir"` | EMDIR | 40 | exact name (xmem pool) |
| `"zeitwert des geldes"` | TVM | 35 | exact alias (DE) |
| `"quadratwurzel"` | SQRT | 35 | exact alias (DE) |
| `"square root"` | SQRT | 35 | exact alias (EN) |
| `"financial solver"` | TVM | 35 | exact alias (EN) |
| `"sqirt"` | SQRT | 8 | fuzzy match (1-typo on name) |

**Queries that should NOT appear in the parity fixture (unstable top-1):**

| Query | Why Unstable |
|-------|-------------|
| `"wurzel"` | Multiple entries score 28 (FSOLVE, RTS, SQRT, Z^1/N, Z^1/W); tie-break by display_name puts FSOLVE first |
| `"zinseszins"` | Returns EMPTY — not a real TVM alias in current data |
| `"zineszins"` | Returns EMPTY — fuzzy distance from Zinseszins aliases exceeds threshold |
| `"compound interest"` | Returns EMPTY — not a real TVM alias |

### Actual TVM aliases (confirmed) [VERIFIED: node.js against live JSON]

```json
"search_aliases": [
  "TVM",
  "time value of money",
  "financial solver",
  "N I PV PMT FV",
  "TVM menu",
  "Zeitwert des Geldes",
  "Finanzmathematik Löser",
  "Finanzrechner interaktiv"
]
```

### Actual SQRT aliases (confirmed) [VERIFIED: node.js against live JSON]

```json
"search_aliases": [
  "Quadratwurzel",
  "square root",
  "Wurzel ziehen",
  "SQRT berechnen",
  "Wurzel aus X",
  "Wurzelfunktion"
]
```

### CLAUDE.md "JSON canonical data flow" section — current text [VERIFIED: reading actual file]

Current (lines 76–82 of CLAUDE.md):
```markdown
### JSON canonical data flow

Five `docs/hp41-*-functions.json` files (~350 entries total) drive keybindings, `?` overlay, and right-panel. Loaded via `include_str!` + `OnceLock` in `help_data.rs`. Malformed JSON panics at first access.

- `just docs-matrix` regenerates function-matrix docs; `just docs-matrix-check` CI drift-catch.
- Op ↔ JSON parity: `function_matrix_parity.rs`; key coverage: `key_coverage.rs`.
- Right-panel: `key_ref_entries()` excludes XROM functions (`entry.xrom.is_none()`).
```

**Three things to add (HSQUAL-04):**

1. The `search_aliases` field: `search_aliases: Vec<String>` / `search_aliases?: string[]` —
   invisible match surface (never rendered in overlay), populated by `just help-aliases`.
   Parity guard: `phase61_help_search_aliases.rs` (Rust) + Phase 61 block in `help_data.test.ts` (TS).

2. The DE-in-search-input convention exception: the English-only doc rule applies to
   docs/ADRs/planning files. `search_aliases` is search *input* vocabulary, not documentation.
   German aliases are in-scope in `search_aliases` by design.

3. The matcher: both frontends score entries via tiered scoring
   (exact 40 > prefix 32 > substring 24 > fuzzy 8 on display_name; alias: 35/28/21/7;
   desc: 30/24/18/6; cat: 20/16/12/4). Non-empty query returns relevance-ranked flat list;
   empty query preserves category-grouped view. The six-pool count (not five) should also
   be updated in the opening line.

**Proposed replacement text:**
```markdown
### JSON canonical data flow

Six `docs/hp41*-functions.json` pools (~380 entries total) drive keybindings, `?` overlay,
and right-panel. Loaded via `include_str!` + `OnceLock` in `help_data.rs`. Malformed JSON
panics at first access.

- `just docs-matrix` regenerates function-matrix docs; `just docs-matrix-check` CI drift-catch.
- Op ↔ JSON parity: `function_matrix_parity.rs`; key coverage: `key_coverage.rs`.
- Right-panel: `key_ref_entries()` excludes XROM functions (`entry.xrom.is_none()`).
- **`search_aliases`** — invisible match surface added in v4.2 (HSDATA-01/02). Array of
  alternative spellings, abbreviations, and synonyms in both DE and EN. Never rendered in
  the `?` overlay. Populated by `just help-aliases` (dev-only, not a CI step). Schema
  validated by `just schema-aliases-check` (wired into ci.yml).
- **DE aliases in `search_aliases` are intentional** — the English-only doc rule applies to
  docs/ADRs/planning files. `search_aliases` is search input vocabulary, not documentation.
  German terms are in-scope by design.
- **Tiered matcher** (Phase 59): both frontends score against display_name / description /
  category / search_aliases using tier weights (name: 40/32/24/8; alias: 35/28/21/7; desc:
  30/24/18/6; cat: 20/16/12/4). Non-empty query → relevance-ranked flat list; empty query →
  existing category-grouped view unchanged. CLI↔GUI parity guarded by `docs/fixtures/help_search_parity.json`.
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Phase 59 tests used synthetic `make_entry()` data with placeholder aliases | Phase 61 tests use real loaded JSON alias data via `ranked_help_entries()` | Phase 61 | End-to-end coverage of actual committed alias content |
| CLAUDE.md mentioned "Five `docs/hp41-*-functions.json`" | Needs update to six pools + alias field + DE convention note | Phase 61 | Accurate project documentation for contributors |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `jq` is pre-installed on `ubuntu-latest` GitHub Actions runners and supports the `type` filter syntax used | Schema Gate | Gate fails in CI; fix: add `sudo apt-get install -y jq` step or switch to node script |
| A2 | `docs/fixtures/` directory can be added without breaking any existing import glob | Parity Fixture | Unlikely; no glob imports scan `docs/fixtures/` |
| A3 | Vite's `fs.allow` set to repo root (vite.config.ts) covers `docs/fixtures/help_search_parity.json` | Parity Fixture (TS) | TS parity import fails; fix: extend `fs.allow` to include the fixtures subdirectory |
| A4 | Adding "Zinseszins" and "compound interest" to TVM aliases is acceptable as a hand-edit (not LLM regeneration) | Pitfall 4 | N/A — it's a content decision, not a technical constraint |

## Open Questions

1. **Should "Zinseszins" / "compound interest" be manually added to TVM aliases?**
   - What we know: Phase 61 success criterion 1 says "Zineszins → TVM" (confirmed typo
     of "Zinseszins" from the design spec). Neither "Zinseszins" nor "compound interest"
     is in the current TVM `search_aliases`. The UAT deferred these to Phase 61.
   - What's unclear: Is this a manual alias hand-edit task within Phase 61, or does the
     success criterion accept that "Zeitwert des Geldes" (which IS present) suffices for
     German TVM lookup?
   - Recommendation: Add "Zinseszins" and "compound interest" to TVM's `search_aliases`
     as a targeted hand-edit in Wave 1 (not via LLM regeneration). This satisfies the
     original UAT expectations and is a data-only diff (one alias array edit). Then the
     parity fixture can include `"zinseszins" → TVM` (exact alias score 35).

2. **Parity fixture location: `docs/fixtures/` vs `hp41-cli/tests/fixtures/`?**
   - What we know: The TS parity test needs to import the fixture (via Vite static import
     or dynamic `fs.readFileSync`). Vite's `fs.allow` covers the repo root and below,
     so `docs/fixtures/` is accessible.
   - What's unclear: Whether a `docs/fixtures/` subdirectory is semantically clean (it
     mixes test data with documentation data).
   - Recommendation: Use `docs/fixtures/` since it needs to be accessible to BOTH the
     Rust side (via `include_str!`) and the TS side (via Vite import). Keeping it in
     `docs/` is consistent with the rest of the canonical data convention.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `jq` | Schema gate script | ✓ (local + ubuntu-latest) | 1.7.1 | Replace with `node -e` script |
| `node` | (optional for schema alternative) | ✓ | v22.16.0 | n/a |
| `cargo test` | Rust tests | ✓ | MSRV 1.88 | n/a |
| `vitest` | TS tests | ✓ | 4.1.6 (pinned) | n/a |
| `serde_json` | Rust fixture parsing | ✓ (already in workspace) | workspace version | n/a |

**Missing dependencies with no fallback:** none.
**Missing dependencies with fallback:** none.

## Validation Architecture

> `workflow.nyquist_validation` is absent from config.json — treated as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Rust framework | `cargo test` (`#[test]`), `cargo test --workspace` |
| TS framework | Vitest 4.1.6, `npm test` (= `vitest run`) |
| Rust config | no config file — standard Cargo test runner |
| TS config | `vite.config.ts` (test.environment: jsdom, globals: false) |
| Rust quick run | `cargo test -p hp41-cli --test phase61_help_search_aliases` |
| Rust full suite | `just test` |
| TS quick run | `cd hp41-gui && npm test -- help_data` |
| TS full suite | `just gui-ci` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| HSQUAL-01 (Rust) | Scoring tiers, fuzzy, DE+EN alias, empty-query | unit/integration | `cargo test -p hp41-cli --test phase61_help_search_aliases` | ❌ Wave 0 |
| HSQUAL-01 (TS) | Same behaviors in TS scoreEntry/rankedEntries | unit | `cd hp41-gui && npm test -- help_data` | ❌ Wave 0 (append to help_data.test.ts) |
| HSQUAL-02 | CLI↔GUI parity for canonical queries | integration | both commands above | ❌ Wave 0 (fixture JSON + both tests) |
| HSQUAL-03 | Schema gate: search_aliases present+typed, >=1 per impl | schema | `just schema-aliases-check` | ❌ Wave 0 (script + Justfile recipe) |
| HSQUAL-04 | CLAUDE.md docs edit | manual review | n/a | ❌ (edit existing file) |

**Notes on HSQUAL-01 coverage against Phase 59 existing tests:**

Phase 59 already covers (via synthetic data, passing now):
- All four scoring tiers exact/prefix/substr/fuzzy on `display_name`
- Fuzzy typo "zineszins" → TVM (synthetic alias "Zinseszins")
- Alias exact "wurzel" → SQRT (synthetic alias "Wurzel")
- DE + EN alias resolution (synthetic data)
- Empty-query passthrough
- All four fields scored
- `ranked_help_entries` ordering

Phase 61 MUST ADD (tests against real JSON data):
- `ranked_help_entries("zeitwert des geldes")` → TVM at top
- `ranked_help_entries("quadratwurzel")` → SQRT at top
- `ranked_help_entries("square root")` → SQRT at top
- `ranked_help_entries("financial solver")` → TVM at top
- IF "Zinseszins" alias is added: `ranked_help_entries("zinseszins")` → TVM at top
- Parity: TS `rankedEntries(allFunctionsEntries(), q)` produces same top-1 as above

### Sampling Rate
- **Per task commit:** `cargo test -p hp41-cli --test phase61_help_search_aliases` (Rust) + `cd hp41-gui && npm test -- help_data` (TS)
- **Per wave merge:** `just test` + `just gui-ci`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `hp41-cli/tests/phase61_help_search_aliases.rs` — covers HSQUAL-01 (real data) + HSQUAL-02 (parity fixture Rust side)
- [ ] `docs/fixtures/help_search_parity.json` — canonical parity fixture
- [ ] `hp41-gui/src/help_data.test.ts` Phase 61 block — covers HSQUAL-01 (TS) + HSQUAL-02 (parity fixture TS side)
- [ ] `scripts/check-aliases-schema.sh` — covers HSQUAL-03
- [ ] `Justfile` `schema-aliases-check` recipe — wires script
- [ ] `.github/workflows/ci.yml` `schema-aliases` job — wires recipe into CI

## Security Domain

This phase has no authentication, authorization, cryptographic, or network surface. The
schema gate validates committed static files. Security section is not applicable.

## Project Constraints (from CLAUDE.md)

All of the following Frozen Invariants apply and must not be violated:

- **`hp41-core` untouched**: Phase 61 writes zero changes to `hp41-core/`. [CONFIRMED: all
  deliverables are in `hp41-cli/tests/`, `hp41-gui/src/`, `docs/`, `scripts/`, `.github/`,
  `Justfile`, `CLAUDE.md`]
- **Zero new runtime deps**: No `Cargo.toml` or `package.json` dependency additions.
  `serde_json` is already a dev dependency; `jq` is a system tool.
- **`just` sole task runner**: All CI invocations go through `just` recipes. The new
  `schema-aliases-check` recipe follows the `license-audit` pattern.
- **4-way exhaustive-match invariant**: Phase 61 adds NO new `Op` variants. Invariant not
  triggered.
- **JSON canonical data flow**: The `docs/fixtures/` directory is a NEW subdirectory. It
  does not affect any existing `include_str!` path or Vite import.
- **English-only commits**: Test file comments, CLAUDE.md edits, script comments in English.
  German aliases in `search_aliases` JSON are explicitly allowed (the DE convention exception
  is what HSQUAL-04 documents).
- **Commits via `/git-workflow:commit --with-skills`**: Never bare `git commit`.
- **Save-file backward compat**: No `CalcState` or `HelpEntry` struct changes. `search_aliases`
  already has `#[serde(default)]` from Phase 58.

## Sources

### Primary (HIGH confidence)
- `hp41-cli/src/help_data.rs` (lines 322–530) — complete Rust scorer implementation [VERIFIED: file read]
- `hp41-gui/src/help_data.ts` (lines 342–471) — complete TS scorer implementation [VERIFIED: file read]
- `hp41-cli/tests/phase59_help_search.rs` — existing Phase 59 test file, full content [VERIFIED: file read]
- `hp41-gui/src/help_data.test.ts` — existing TS test file [VERIFIED: file read]
- `hp41-gui/src/HelpOverlay.test.tsx` (lines 769–879) — Phase 59 render-branch tests [VERIFIED: file read]
- `docs/hp41-advantage-functions.json` (TVM entry) — actual alias data [VERIFIED: node.js]
- `docs/hp41cv-functions.json` (SQRT entry) — actual alias data [VERIFIED: node.js]
- All 6 pools alias coverage verification (0 missing for implemented entries) [VERIFIED: node.js]
- Canonical query top-1 results verified against live algorithm [VERIFIED: node.js]
- `scripts/check-free42-contamination.sh` — bash script precedent [VERIFIED: file read]
- `.github/workflows/ci.yml` — no path filter, no Node.js, license-audit job pattern [VERIFIED: file read]
- `.github/workflows/ci-gui.yml` — path filter on hp41-gui/hp41-core only [VERIFIED: file read]
- `Justfile` — all existing recipes [VERIFIED: file read]
- `CLAUDE.md` (lines 76–82) — current "JSON canonical data flow" text [VERIFIED: file read]

### Secondary (MEDIUM confidence)
- jq 1.7.1 available on local macOS and ubuntu-latest runners [VERIFIED: local env, ASSUMED for CI runners]

### Tertiary (LOW confidence)
- None — all claims verified via direct file reads or code execution.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all tools already in the project; no new external packages
- Architecture: HIGH — patterns verified against existing precedent files
- Pitfalls: HIGH — verified against actual JSON data and algorithm execution
- Parity fixture: HIGH — canonical query set verified against live algorithm

**Research date:** 2026-06-05
**Valid until:** 2026-07-05 (stable domain — only invalidated if Phase 60 alias data is modified)
