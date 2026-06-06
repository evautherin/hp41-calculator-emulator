# Phase 59: Runtime Matcher — Pattern Map

**Mapped:** 2026-06-04
**Files analyzed:** 5 new/modified files
**Analogs found:** 5 / 5

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `hp41-cli/src/help_data.rs` (extend) | utility / data-access | transform | self (existing `filter_help_rows` at line 331) | exact — same file, adjacent function |
| `hp41-gui/src/help_data.ts` (extend) | utility / data-access | transform | self (existing `filterHelpEntries` at line 155) | exact — same file, adjacent function |
| `hp41-gui/src/HelpOverlay.tsx` (modify) | component | request-response (useMemo filter) | self (existing `filtered` useMemo at line 222 + `filteredAllFn` at line 233) | exact — same component, same memo slots |
| `hp41-cli/src/ui.rs` (modify, minimal) | utility / render | request-response | self (existing `render_help_overlay` at line 447) | exact — same function, new branch |
| `hp41-cli/tests/phase59_help_search.rs` (new) | test | — | `hp41-cli/tests/phase25_help_data.rs` | role-match |

---

## Pattern Assignments

### `hp41-cli/src/help_data.rs` — new functions `score_entry`, `tier_score`, `levenshtein_bounded`, `ranked_help_entries`

**Analog:** Same file — `filter_help_rows` (lines 321–356) and `HelpRow` / `HelpEntry` structs (lines 64–88, 358–365)

**Existing `HelpEntry` struct with `search_aliases`** (lines 64–88):
```rust
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct HelpEntry {
    pub op_variant: String,
    pub display_name: String,
    pub category: String,
    pub status: String,
    pub phase: Option<String>,
    pub key_path: Option<String>,
    pub description: String,
    #[serde(default)]
    pub divergences: Vec<String>,
    #[serde(default)]
    pub xrom: Option<XromEntry>,
    /// Invisible match surface for Phase 59 search (D-58.3 / HSDATA-01).
    #[serde(default)]
    pub search_aliases: Vec<String>,
}
```

**Existing `HelpRow` struct** (lines 358–365) — returned by `ranked_help_entries` but NOT used as scoring input:
```rust
#[derive(Debug, Clone)]
pub struct HelpRow {
    pub key: String,
    pub op: String,
    pub desc: String,
}
```

**Empty-query guard pattern to preserve** (lines 331–334):
```rust
pub fn filter_help_rows<'a>(rows: &'a [HelpRow], query: &str) -> Vec<&'a HelpRow> {
    if query.is_empty() {
        return rows.iter().collect();
    }
```

**Lowercase + contains pattern to supersede for non-empty case** (lines 335–340):
```rust
    let q = query.to_lowercase();
    let matches = |row: &HelpRow| {
        row.key.to_lowercase().contains(&q)
            || row.op.to_lowercase().contains(&q)
            || row.desc.to_lowercase().contains(&q)
    };
```

**`help_entries_all()` return type** (lines 276–283) — the new scorer calls this, not `help_overlay_rows()`:
```rust
pub fn help_overlay_rows() -> Vec<HelpRow> {
    let entries: Vec<&HelpEntry> = help_entries_all()
        .filter(|e| e.status == "implemented")
        .collect();
```

**`ranked_help_entries` output projection pattern** (from `help_overlay_rows`, lines 298–315) — same `HelpRow` construction for key display, no headers:
```rust
let key = entry.key_path.clone().unwrap_or_else(|| {
    if entry.xrom.is_none() {
        format!("XEQ \"{}\"", entry.display_name)
    } else {
        String::new()
    }
});
rows.push(HelpRow {
    key,
    op: entry.display_name.clone(),
    desc: entry.description.clone(),
});
```

**Clippy constraint — `unwrap_or` pattern required** (enforced by `#![deny(clippy::unwrap_used)]`): Use `.unwrap_or(0)` / `.unwrap_or(usize::MAX)` instead of `.unwrap()`. The RESEARCH.md Levenshtein code already shows `.unwrap_or(0)` for the `.min()` call on the early-exit iterator — use that form exactly.

**Existing test fixture pattern** (lines 367–430 in `mod tests`):
```rust
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    // inline unit tests — fixture() builds Vec<HelpRow> from literals
    fn fixture() -> Vec<HelpRow> { vec![ ... ] }

    #[test]
    fn empty_query_returns_all_rows() {
        let rows = fixture();
        let filtered = filter_help_rows(&rows, "");
        assert_eq!(filtered.len(), rows.len());
    }
```
Phase 59 new functions (`score_entry`, `ranked_help_entries`) must add inline `#[test]` blocks in this same `mod tests` block (or in a parallel integration test under `hp41-cli/tests/phase59_help_search.rs`). The RESEARCH.md Validation Architecture confirms `cargo test -p hp41-cli --test phase59_help_search` as the quick-run command, meaning a separate integration test file is the intended home.

---

### `hp41-gui/src/help_data.ts` — new functions `levenshteinBounded`, `tierScore`, `scoreEntry`, `rankedEntries`; upgrade `filterHelpEntries`

**Analog:** Same file — `filterHelpEntries` (lines 155–167) and `HelpEntry` interface (lines 59–94)

**Existing `HelpEntry` TS interface with `search_aliases`** (lines 59–94):
```ts
export interface HelpEntry {
    op_variant: string;
    display_name: string;
    category: string;
    status: 'implemented' | 'deferred-v3' | 'na';
    phase: string | null;
    key_path: string | null;
    description: string;
    divergences?: string[];
    xrom?: XromEntry;
    example?: string;
    notes?: string;
    /** Invisible match surface for Phase 59 search (D-58.4 / HSDATA-02). */
    search_aliases?: string[];
}
```

**Empty-query guard to preserve verbatim** (lines 155–158):
```ts
export function filterHelpEntries(query: string): readonly HelpEntry[] {
    const q = query.toLowerCase().trim();
    if (q === '') {
        return helpEntries().filter(e => e.key_path !== null);
    }
```

**Substring filter to supersede for non-empty case** (lines 160–166):
```ts
    return helpEntries().filter(e =>
        e.key_path !== null && (
            e.display_name.toLowerCase().includes(q) ||
            e.description.toLowerCase().includes(q) ||
            e.category.toLowerCase().includes(q)
        )
    );
```

**`helpEntriesAll()` function** — the TS scorer's base pool (not `helpEntries()` alone), mirroring the Rust `help_entries_all()` call. Both `filterHelpEntries` (post-upgrade) and the new `rankedEntries` must operate over `helpEntriesAll()` as their entry pool source.

**`search_aliases` optional field access pattern** (line 93 + the Phase 58 test in `help_data.test.ts` line 112–115):
```ts
// Always guard with ?? [] before iterating aliases:
const aliasScore = (entry.search_aliases ?? [])
    .map(a => tierScore(a, q, ...))
    .reduce((a, b) => Math.max(a, b), 0);
```

---

### `hp41-gui/src/HelpOverlay.tsx` — modify `filtered` (line 222) and `filteredAllFn` (line 233) useMemos; add flat-ranked render branch

**Analog:** Same file — existing useMemo filter blocks (lines 221–291) and the downstream `sectionGroups` memo (lines 244–267)

**Keyboard Shortcuts tab filter to upgrade** (lines 222–230):
```ts
const filtered = useMemo(() => {
    const q = query.toLowerCase().trim();
    if (q === '') return allEntries;  // MUST preserve this guard unchanged
    return allEntries.filter(e =>
        e.display_name.toLowerCase().includes(q) ||
        e.description.toLowerCase().includes(q) ||
        e.category.toLowerCase().includes(q)
    );
}, [query, allEntries]);
```

**All Functions tab filter to upgrade** (lines 233–241):
```ts
const filteredAllFn = useMemo(() => {
    const q = query.toLowerCase().trim();
    if (q === '') return allFnEntries;  // MUST preserve this guard unchanged
    return allFnEntries.filter(e =>
        e.display_name.toLowerCase().includes(q) ||
        e.description.toLowerCase().includes(q) ||
        e.category.toLowerCase().includes(q)
    );
}, [query, allFnEntries]);
```

**`sectionGroups` memo that feeds from `filtered`** (lines 244–267) — pipes `filtered` through grouping. When `query !== ''`, the render path must bypass `sectionGroups` and render a flat list instead. The memo itself can remain but is simply not used in the active-query render branch:
```ts
const sectionGroups = useMemo(() => {
    return SECTIONS.map(section => {
        const sectionEntries = filtered.filter(section.predicate);
        const catMap = new Map<string, HelpEntry[]>();
        for (const entry of sectionEntries) {
            // ... group by category, sort alphabetically
        }
        return { section, groups: Array.from(catMap.entries()), count: sectionEntries.length };
    });
}, [filtered]);
```

**Parallel `allFnSectionGroups` memo** (lines 270–291) — same pattern, feeds from `filteredAllFn`. Same bypass rule applies for the All Functions tab render branch.

**Component render JSX entry point** (line 335+) — the JSX `<div className="help-overlay" ...>` block is where the flat-vs-grouped branch lands. The existing pattern for conditional JSX follows the `activeTab` branch already present in the component; the query branch follows the same pattern.

**CSS class reference for test selectors** (from `HelpOverlay.test.tsx` lines 237–253):
```ts
// Existing test queries — Phase 59 tests must still pass these:
container.querySelectorAll('.help-overlay-category-heading')  // zero when ranked
container.querySelectorAll('.help-overlay-row')               // non-zero when ranked
container.querySelector('.help-overlay-search')               // always present
container.querySelector('.help-overlay-empty')                // when no matches
```

---

### `hp41-cli/src/ui.rs` — add flat-ranked render branch in `render_help_overlay`

**Analog:** Same file — `render_help_overlay` function (lines 447–498)

**Current render path to branch** (lines 447–498):
```rust
let overlay_rows = help_data::help_overlay_rows();
let filtered = help_data::filter_help_rows(&overlay_rows, &app.help_search_query);

// Match count — non-header rows only
let match_count = filtered
    .iter()
    .filter(|r| !r.desc.starts_with("==="))
    .count();

let rows: Vec<Row> = filtered
    .iter()
    .map(|row| {
        if row.desc.starts_with("===") {
            Row::new(vec![Cell::from(""), Cell::from(""), Cell::from(row.desc.clone())])
                .style(ratatui::style::Style::new().bold())
        } else {
            Row::new(vec![
                Cell::from(row.key.clone()),
                Cell::from(row.op.clone()),
                Cell::from(row.desc.clone()),
            ])
        }
    })
    .collect();

let title = if app.help_search_query.is_empty() {
    " HP-41 Function Reference  [? or Esc to close] ".to_string()
} else {
    format!(" HP-41 Function Reference  [{} match{}] ",
        match_count, if match_count == 1 { "" } else { "es" })
};
```

**Branch target for Phase 59** — the planner replaces the single `overlay_rows`/`filtered` block with an `if/else` on `app.help_search_query.is_empty()`. The `ranked_help_entries` path returns `Vec<HelpRow>` with no headers, so the `row.desc.starts_with("===")` arm in the render loop is never hit for ranked results; the render loop itself is unchanged. The `match_count` variable counts non-header rows from `filtered` — for the ranked path, all rows are data rows, so `count()` without the header filter is identical.

---

### `hp41-cli/tests/phase59_help_search.rs` (new)

**Analog:** `hp41-cli/tests/phase25_help_data.rs` (lines 1–60)

**File header + allow pattern** (lines 1–12 of `phase25_help_data.rs`):
```rust
//! Phase 25 Plan 04 Task 1 smoke tests — ...
#![allow(clippy::unwrap_used)]

use std::collections::HashSet;
use hp41_cli::help_data::{help_entries, help_entries_all, help_overlay_rows};
```

**Phase 59 import line:**
```rust
use hp41_cli::help_data::{score_entry, ranked_help_entries, HelpEntry};
```

**Test structure pattern** (from `phase25_help_data.rs` lines 14–36):
```rust
#[test]
fn help_entries_loads_at_runtime() {
    let entries = help_entries();
    assert!(!entries.is_empty(), "...");
}
```

**Synthetic `HelpEntry` fixture pattern for scorer tests** — the scorer tests must NOT depend on real JSON data (to isolate algorithm logic from alias content, which lands in Phase 60). Build a local `HelpEntry` literal. The struct fields require `String` values and use `#[serde(default)]` for optional fields, but in test code these are constructed directly:
```rust
fn make_entry(display_name: &str, description: &str, category: &str, aliases: &[&str]) -> HelpEntry {
    HelpEntry {
        op_variant: display_name.to_string(),
        display_name: display_name.to_string(),
        category: category.to_string(),
        status: "implemented".to_string(),
        phase: None,
        key_path: None,
        description: description.to_string(),
        divergences: vec![],
        xrom: None,
        search_aliases: aliases.iter().map(|s| s.to_string()).collect(),
    }
}
```
Note: `HelpEntry` derives `Clone` (line 64 in `help_data.rs`), so test fixtures can be cloned without issue.

---

## Shared Patterns

### Mirror discipline: CLI-Rust (`hp41-cli`) ↔ Frontend-TS (`hp41-gui/src`)

**Source:** `hp41-cli/src/prgm_display.rs` (line 8 comment) + `hp41-gui/src-tauri/src/prgm_display.rs` (line 8 comment)

The comment in the GUI Tauri backend file is the canonical documentation of the mirror pattern:
```rust
//! Copied from hp41-cli/src/prgm_display.rs per Phase 18 D-03.
```
The Phase 59 mirror is CLI `help_data.rs` ↔ frontend `help_data.ts` (not the Tauri backend). The scorer constants, tier boundaries, Levenshtein algorithm, and fuzzy threshold must be identical. The parity fixture in Phase 61 will enforce this; in Phase 59 the planner documents the intent with a comment in both files.

### `#[serde(default)]` on `search_aliases`

Already present on `HelpEntry.search_aliases` in both Rust (line 86) and TS (optional `?` on line 93). The scorer code must always treat aliases as potentially absent:
- Rust: `entry.search_aliases.iter()` (works fine on an empty `Vec` — no guard needed)
- TypeScript: `(entry.search_aliases ?? []).map(...)` — the `??` guard is required because the TS field is `string[] | undefined`, not `string[]`

### Error-handling / panic discipline in new Rust code

`#![deny(clippy::unwrap_used)]` is active crate-wide. All new Rust functions must follow:
- `.unwrap_or(0)` / `.unwrap_or(usize::MAX)` for `Option` on iterators
- `.unwrap_or_else(|| ...)` for fallback strings
- `debug_assert!(!query.is_empty(), ...)` for preconditions, not panicking asserts

### `globals: false` Vitest — explicit imports required

From `hp41-gui/vite.config.ts` line 37: `globals: false`. Every new test file in `hp41-gui/src/` must import all Vitest APIs explicitly:
```ts
import { describe, it, expect, afterEach } from 'vitest';
import { render, fireEvent, cleanup } from '@testing-library/react';
```

### `afterEach(cleanup)` for portal-rendered components

From `hp41-gui/src/App.test.tsx` lines 185–189 (comment + code):
```ts
// globals:false (no testing-library auto-cleanup) — without this, portaled
// nodes accumulate in document.body across tests
afterEach(() => {
    cleanup();
});
```
Any Phase 59 `HelpOverlay.test.tsx` additions that render the overlay must include this `afterEach(cleanup)` pattern. Pure `scoreEntry` / `rankedEntries` tests in `help_data.test.ts` do not need it (no DOM render).

### `describe` + `it` nesting in existing test files

**Source:** `hp41-gui/src/help_data.test.ts` lines 19–145 and `HelpOverlay.test.tsx` lines 34–278

Both files use a two-level nesting: `describe('feature name', () => { it('specific behavior', ...) })`. New tests must be appended as new `describe` blocks rather than inserted into existing blocks.

---

## No Analog Found

All Phase 59 files have close analogs in the existing codebase. The one algorithm that has no existing analog is the bounded Levenshtein function — it is genuinely new code in both Rust and TS. However, RESEARCH.md provides the complete 30-line implementation for both languages, and the project has no constraint against adding pure integer-arithmetic helpers (no new crate dep required).

| File / Function | Role | Data Flow | Note |
|---|---|---|---|
| `levenshtein_bounded` in `help_data.rs` | pure algorithm | transform | No existing edit-distance code anywhere in codebase; use RESEARCH.md §Pattern 2 verbatim |
| `levenshteinBounded` in `help_data.ts` | pure algorithm | transform | Mirror of the above; use RESEARCH.md §Pattern 2 verbatim |

---

## Metadata

**Analog search scope:** `hp41-cli/src/`, `hp41-cli/tests/`, `hp41-gui/src/`, `hp41-gui/src-tauri/src/`
**Files read:** 10 (help_data.rs, ui.rs, help_data.ts, HelpOverlay.tsx, prgm_display.rs ×2, help_data.test.ts, HelpOverlay.test.tsx, phase25_help_data.rs, vite.config.ts)
**Pattern extraction date:** 2026-06-04
