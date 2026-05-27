// Phase 26 Plan 03 D-26.8 / D-25.16 — TypeScript port of hp41-cli/src/help_data.rs.
//
// `docs/hp41cv-functions.json` is the SINGLE SOURCE OF TRUTH for the GUI's
// `?` help overlay AND the CLI's help overlay AND the generated function
// matrix (`docs/hp41cv-function-matrix.md`). This module imports the JSON
// at build time via vite's static JSON-import (D-25.16); the resulting
// array is baked into the production bundle — zero runtime fetch.
//
// Hard-build-blocker semantics (D-25.17 / D-26.8): a malformed JSON file
// fails vite's build step. This is intentional — canonical data files must
// not be empty / malformed.
//
// Phase 31-04: parallel-loads docs/hp41-math1-functions.json via Vite static
// JSON-import (D-31.10 / C-28.3 / ADR-005). Mirrors hp41-cli/src/help_data.rs
// Phase 29 D-29.2 second OnceLock + merged accessor pattern.
//
// Phase 41 Plan 41-02 D-carried.8: parallel-loads docs/hp41-time-functions.json
// via Vite static JSON-import. Mirrors hp41-cli/src/help_data.rs Phase 39 D-39.12
// fourth OnceLock + merged accessor pattern. Hard-build-blocker semantics per
// D-25.17 (malformed JSON fails the Vite build — intentional).
//
// Phase 46 Plan 46-02: parallel-loads docs/hp41-advantage-functions.json via Vite
// static JSON-import (D-44.1 fifth JSON source-of-truth, 114 entries, 7-category
// convention). Mirrors hp41-cli/src/help_data.rs Phase 44 fifth OnceLock + merged
// accessor pattern. Hard-build-blocker semantics per D-25.17 (malformed JSON fails
// the Vite build — intentional).

import functions from '../../docs/hp41cv-functions.json';
import math1Functions from '../../docs/hp41-math1-functions.json';
import stat1Functions from '../../docs/hp41-stat1-functions.json';
import timeFunctions from '../../docs/hp41-time-functions.json';
import advantageFunctions from '../../docs/hp41-advantage-functions.json';
// Phase 49 D-49.11 — keyboard shortcuts single source of truth.
// Vite static JSON-import: baked into the production bundle at build time.
// Malformed JSON fails the Vite build — hard-build-blocker semantics per D-25.17.
import keyboardShortcutsData from '../../docs/keyboard-shortcuts.json';

/// XROM module reference attached to Math Pac I (and future v3.1+ pac) entries.
/// Matches the `xrom` object shape in docs/hp41-math1-functions.json (ADR-005 /
/// C-28.3 / Phase 29 D-29.1).
export interface XromEntry {
    /// Human-readable module name, e.g. "Math 1".
    module: string;
    /// HP-41 hardware XROM module ID (7 for Math Pac I).
    module_id: number;
    /// 1-based function index within the module.
    function_id: number;
}

/// One row in the canonical HP-41CV function table.
///
/// Mirrors hp41-cli::help_data::HelpEntry (Rust) field-for-field. See
/// `hp41-cli/src/help_data.rs` lines 24-57 for the canonical schema.
///
/// Phase 31-04 extends with optional `xrom` field: present on Math Pac I
/// entries (loaded from docs/hp41-math1-functions.json), absent on v2.2
/// built-in entries.
export interface HelpEntry {
    /// Op variant name (PascalCase, e.g. `"Pi"`). For XEQ-by-Name-only
    /// conditional tests this is an `_XEQ`-suffixed alias.
    op_variant: string;
    /// HP-41 mnemonic as shown on the display (e.g. `"PI"`).
    display_name: string;
    /// One of the 20 enumerated categories.
    category: string;
    /// `"implemented"`, `"deferred-v3"`, or `"na"`.
    status: 'implemented' | 'deferred-v3' | 'na';
    /// GSD phase ID string (e.g. `"21"`) or `null` for v3.x.
    phase: string | null;
    /// CLI keystroke (e.g. `"f-7"`) or `null` for internal / XEQ-by-Name-only.
    key_path: string | null;
    /// <= 80 chars, suitable for the `?` overlay row.
    description: string;
    /// Optional free-form notes about HP-41 hardware divergences.
    divergences?: string[];
    /// XROM module reference — present on Math Pac I entries, absent on
    /// v2.2 built-in entries. Used by HelpOverlay.tsx to partition entries
    /// into "HP-41CV (built-in)" vs "Math 1 Pac (XROM 7)" sections (D-31.8).
    xrom?: XromEntry;
    /// Phase 49 D-49.5 — optional enrichment fields for expandable rows (ONBOARD-03).
    /// One-line terse example, e.g. "3 ENTER 4 + → 7". <= 60 chars.
    example?: string;
    /// Phase 49 D-49.5 — factual behavioral notes: stack lift, LASTX, domain errors.
    notes?: string;
}

/// Lazy-init cache. Vite's static `import` is itself the cache (module
/// evaluation is one-shot), so no OnceLock-equivalent is needed — the
/// `functions` binding is evaluated once at module load time.
export function helpEntries(): readonly HelpEntry[] {
    return functions as readonly HelpEntry[];
}

/// One row of the help overlay table, produced by `helpOverlayRows`.
/// Category headers carry `isHeader: true` with `desc: "=== <name> ==="`
/// and empty `key`/`op`.
export interface HelpOverlayRow {
    key: string;
    op: string;
    desc: string;
    isHeader: boolean;
    category: string;
}

/// Render a list of help overlay rows with category-header rows interleaved.
/// Categories appear in their first-appearance order in the JSON; within a
/// category, entries keep the JSON's declared order.
///
/// Entries with `key_path === null` are EXCLUDED per D-26.8 (XEQ-by-Name-only
/// ops aren't keyboard shortcuts and would just clutter the overlay).
export function helpOverlayRows(): readonly HelpOverlayRow[] {
    const entries = helpEntries();
    const categories: string[] = [];
    for (const entry of entries) {
        if (entry.key_path !== null && !categories.includes(entry.category)) {
            categories.push(entry.category);
        }
    }
    const rows: HelpOverlayRow[] = [];
    for (const cat of categories) {
        rows.push({
            key: '',
            op: '',
            desc: `=== ${cat} ===`,
            isHeader: true,
            category: cat,
        });
        for (const entry of entries.filter(e => e.category === cat && e.key_path !== null)) {
            rows.push({
                key: entry.key_path ?? '',
                op: entry.display_name,
                desc: entry.description,
                isHeader: false,
                category: cat,
            });
        }
    }
    return rows;
}

/// Filter help entries by a free-text query. The query is matched
/// case-insensitively against `display_name`, `description`, and `category`.
/// Entries with `key_path === null` are always excluded (D-26.8).
///
/// An empty query returns all entries that have a `key_path` (no filtering).
export function filterHelpEntries(query: string): readonly HelpEntry[] {
    const q = query.toLowerCase().trim();
    if (q === '') {
        return helpEntries().filter(e => e.key_path !== null);
    }
    return helpEntries().filter(e =>
        e.key_path !== null && (
            e.display_name.toLowerCase().includes(q) ||
            e.description.toLowerCase().includes(q) ||
            e.category.toLowerCase().includes(q)
        )
    );
}

/// Phase 31-04: Math Pac I function entries from docs/hp41-math1-functions.json.
///
/// Vite static JSON-import: baked into the production bundle at build time.
/// Malformed JSON fails the Vite build — hard-build-blocker semantics per
/// D-25.17 (parallel to hp41-cli/src/help_data.rs `.expect("...malformed")`).
/// Mirrors Phase 29 D-29.2 second OnceLock + accessor pattern in Rust.
export function helpEntriesMath1(): readonly HelpEntry[] {
    return math1Functions as readonly HelpEntry[];
}

/// Phase 36 Plan 36-02: Stat 1 Pac function entries from docs/hp41-stat1-functions.json.
///
/// Vite static JSON-import: baked into the production bundle at build time.
/// Malformed JSON fails the Vite build — hard-build-blocker semantics per
/// D-25.17 (parallel to hp41-cli/src/help_data.rs `.expect("...malformed")`).
/// Mirrors Phase 34 D-34.2 third OnceLock + accessor pattern in Rust (hp41-cli).
/// Source: docs/hp41-stat1-functions.json (26 entries, 7-category convention per D-34.1).
export function helpEntriesStat1(): readonly HelpEntry[] {
    return stat1Functions as readonly HelpEntry[];
}

/// Phase 41 Plan 41-02 D-carried.8: Time Pac function entries from docs/hp41-time-functions.json.
///
/// Vite static JSON-import: baked into the production bundle at build time.
/// Malformed JSON fails the Vite build — hard-build-blocker semantics per
/// D-25.17 (parallel to hp41-cli/src/help_data.rs `.expect("...malformed")`).
/// Mirrors Phase 39 D-39.12 fourth OnceLock + accessor pattern in Rust (hp41-cli).
/// Source: docs/hp41-time-functions.json (35 entries, 7-category convention per D-39.9).
export function helpEntriesTime(): readonly HelpEntry[] {
    return timeFunctions as readonly HelpEntry[];
}

/// Phase 46 Plan 46-02: Advantage Pac function entries from docs/hp41-advantage-functions.json.
///
/// Vite static JSON-import: baked into the production bundle at build time.
/// Malformed JSON fails the Vite build — hard-build-blocker semantics per
/// D-25.17 (parallel to hp41-cli/src/help_data.rs `.expect("...malformed")`).
/// Mirrors Phase 44 D-44.1 fifth OnceLock + accessor pattern in Rust (hp41-cli).
/// Source: docs/hp41-advantage-functions.json (114 entries, 7-category convention per D-44.1).
/// Module partitions: xrom.module === "Adv Conv" (XROM 22, 63 entries) +
///                    xrom.module === "Adv Math" (XROM 24, 51 entries).
export function helpEntriesAdvantage(): readonly HelpEntry[] {
    return advantageFunctions as readonly HelpEntry[];
}

/// Phase 49 D-49.11: Keyboard shortcut entry — one row in the physical keyboard reference.
///
/// Sourced from `docs/keyboard-shortcuts.json` via Vite static JSON-import.
/// Rendered in the Keyboard Shortcuts section of HelpOverlay (KBD-03).
export interface KeyboardShortcut {
    /// Human-readable key label, e.g. "Enter", "Tab", "Ctrl+S".
    key: string;
    /// HP-41 op mnemonic as shown in the shortcut table, e.g. "ENTER", "SHIFT".
    op: string;
    /// <= 80 chars, suitable for the shortcut table description column.
    description: string;
}

/// Phase 49 D-49.11: All keyboard shortcut entries from docs/keyboard-shortcuts.json.
///
/// Vite static JSON-import: baked into the production bundle at build time.
/// Malformed JSON fails the Vite build — hard-build-blocker semantics per D-25.17.
/// Mirrors the 5-pool accessor pattern: no OnceLock needed (module evaluation is one-shot).
export function getKeyboardShortcuts(): readonly KeyboardShortcut[] {
    return keyboardShortcutsData as readonly KeyboardShortcut[];
}

/// Phase 46 Plan 46-02: Merged accessor returning built-in + Math Pac I + Stat 1 Pac + Time Pac + Advantage Pac entries.
///
/// UPDATED from Phase 41 Plan 41-02 (4-pool) to 5-pool concatenation.
/// Parallel to hp41-cli/src/help_data.rs::help_entries_all() (Phase 44 5-pool chain).
/// Used by HelpOverlay.tsx to obtain the full entry pool; the overlay then partitions
/// entries by `entry.xrom` into six sections (D-31.8 extended for Advantage Pac,
/// split across two sections: XROM 22 "Adv Conv" and XROM 24 "Adv Math").
/// Pitfall 5: do NOT create a parallel helpEntriesAll5() — update in-place so all
/// existing callers (HelpOverlay.tsx) automatically pick up Advantage entries.
export function helpEntriesAll(): readonly HelpEntry[] {
    return [...helpEntries(), ...helpEntriesMath1(), ...helpEntriesStat1(), ...helpEntriesTime(), ...helpEntriesAdvantage()];
}
