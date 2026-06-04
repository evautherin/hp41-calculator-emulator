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
import xmemFunctions from '../../docs/hp41-xmem-functions.json';
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
    /** Invisible match surface for Phase 59 search (D-58.4 / HSDATA-02).
     *
     * Alternative spellings, abbreviations, and synonyms that broaden
     * free-text search recall without appearing in the `?` overlay layout.
     * Populated in Phase 60 (alias content); absent until then.
     * Mirrors `search_aliases: Vec<String>` in hp41-cli/src/help_data.rs (~line 87).
     */
    search_aliases?: string[];
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

/// Phase 52: X-MEM built-in entries from docs/hp41-xmem-functions.json.
///
/// Vite static JSON-import: baked into the production bundle at build time.
/// Malformed JSON fails the Vite build — hard-build-blocker semantics per D-25.17.
/// Mirrors Phase 52 D-52.1 sixth OnceLock + accessor pattern in Rust (hp41-cli).
/// Source: docs/hp41-xmem-functions.json (8 entries, "Extended Memory" category).
export function helpEntriesXmem(): readonly HelpEntry[] {
    return xmemFunctions as readonly HelpEntry[];
}

/// Phase lu0 D-lu0-02: All-functions overlay dataset.
///
/// Returns every helpEntriesAll() entry with status === 'implemented', WITHOUT
/// the `key_path !== null` exclusion used by the Keyboard Shortcuts tab (D-26.8).
/// This revises D-26.8's "exclude key_path:null" rule for the new All Functions
/// tab only — the Keyboard Shortcuts tab (filterHelpEntries, helpOverlayRows)
/// keeps the D-26.8 exclusion unchanged.
///
/// Includes the 74 cv-pool key_path:null built-ins (SIN, LN, SQRT, AVIEW, CLST,
/// CLRG, CLA, …) that were previously invisible in the `?` overlay, plus all
/// implemented entries from the 5 module pools.
///
/// Memoized (stable reference for React equality, like helpEntriesAll()).
let cachedAllFunctions: readonly HelpEntry[] | null = null;

/// Phase s17: implemented op_variants deliberately HIDDEN from the All Functions
/// index because they are non-authentic legacy aliases of another function that IS
/// shown. They stay `status: "implemented"` (truthful — they execute) and keep their
/// resolver arm + Op enum variant for v1.0 save-file compatibility (Pitfall 8), but
/// listing them would duplicate a real HP-41 function under a fake mnemonic.
///   - AlphaClear ("CLRALPHA") is the v1.0 legacy alias of Cla ("CLA"); the real
///     HP-41 mnemonic is CLA, which is shown. CLRALPHA still resolves for old saves.
/// Subtracted from the C1 completeness guardrail (see help_data.test.ts).
export const OVERLAY_HIDDEN_ALIASES: ReadonlySet<string> = new Set(['AlphaClear']);

export function allFunctionsEntries(): readonly HelpEntry[] {
    if (cachedAllFunctions === null) {
        cachedAllFunctions = helpEntriesAll().filter(
            e => e.status === 'implemented' && !OVERLAY_HIDDEN_ALIASES.has(e.op_variant),
        );
    }
    return cachedAllFunctions;
}

/// Phase lu0 D-lu0-02: The 61 op_variants that are NOT runnable by XEQ-by-name.
///
/// ALL 61 fall into three classes — none of which is XEQ-runnable on a real HP-41 either:
///   1. Parameterized ops needing an argument (STO, RCL, FIX, SCI, ENG, SF, CF, ISG, DSE,
///      VIEW, TONE, ARCL, ASTO, GTO, XEQ, LBL, CLP, DEL, ASN, STO+/-/*//,
///      and all IND variants).
///   2. Immediate stack/entry keys (+ - * / ENTER CLX CHS Rv X<>Y LASTX %CH).
///   3. Mode / composite-placeholder rows (ALPHA, ALPHA char, ALPHA <-, PRGM, USER,
///      CATALOG, GETKEY, NULL, X?Y / X?0, FS?/FC?/FS?C/FC?C + IND).
///
/// Pinned here AND in hp41-cli/tests/tappability_parity.rs (NON_TAPPABLE Rust set).
/// The Rust test asserts the size is exactly 61 — so neither set can drift silently.
///
/// D-07: NEVER dispatch an id for a NON_TAPPABLE entry — the resolver would reject it.
const NON_TAPPABLE: ReadonlySet<string> = new Set([
    // ── Parameterized ops (require an argument — not XEQ-by-name runnable) ──
    'StoReg', 'RclReg', 'StoArith', 'StoArithStack',
    'StoM', 'StoN', 'StoO', 'RclM', 'RclN', 'RclO',
    'StoInd', 'RclInd', 'StoArithInd',
    'FmtFix', 'FmtSci', 'FmtEng',
    'SfFlag', 'CfFlag', 'SfFlagInd', 'CfFlagInd',
    'FlagTest', 'FlagTestInd',
    'View', 'ViewInd',
    'Tone',
    'Isg', 'Dse', 'IsgInd', 'DseInd',
    'Arcl', 'ArclInd', 'Asto', 'AstoInd',
    'Gto', 'GtoInd',
    'Xeq', 'XeqInd',
    'Lbl',
    'Clp', 'Del',
    'Asn',
    'Test',
    // ── Immediate stack / entry keys (raw arithmetic / stack ops) ──
    'Add', 'Sub', 'Mul', 'Div',
    'Enter', 'Clx', 'Chs', 'Rdn', 'XySwap', 'Lastx',
    'PctChange',
    // ── Mode / composite-placeholder rows ──
    'AlphaToggle', 'AlphaAppend', 'AlphaBackspace',
    'PrgmMode',
    'UserMode',
    'Catalog',
    'GetKey',
    'Null',
]);

/// Phase lu0 D-lu0-02: Tappability resolver for All Functions overlay rows.
///
/// Returns entry.display_name when the entry is runnable by XEQ-by-name,
/// else null. For every runnable entry the resolver accepts display_name
/// verbatim (builtin_card_op / xrom_resolve register the exact display_name
/// spelling, including Unicode glyphs like "ΣBSTAT" U+03A3, "C×", "Z↑N");
/// non-runnable entries (parameterized / key-only / composite) return null
/// and MUST render non-tappable (D-07: never dispatch an unresolvable id).
///
/// The dispatch token for a tappable row is `xeq_${xeqToken(entry)}`, which
/// maps through key_map.rs `xeq_` prefix → Op::Xeq(label) → op_xeq().
/// PRGM mode: the SAME id dispatched in PRGM mode inserts Op::Xeq(label) as a
/// program step (mod.rs PRGM gate) — no frontend branching required.
export function xeqToken(entry: Pick<HelpEntry, 'op_variant' | 'display_name'>): string | null {
    return NON_TAPPABLE.has(entry.op_variant) ? null : entry.display_name;
}

// ── Phase 59 Plan 59-03 — Tiered scorer + bounded Levenshtein ────────────────
//
// Mirror of hp41-cli/src/help_data.rs scorer (plan 59-02).
// Keep byte-equivalent in BEHAVIOR with the Rust implementation per Phase 59
// (CLI<->GUI parity). Parity fixture lands in Phase 61.
//
// Tier constants (from 59-RESEARCH.md §Tiered Scoring):
//   name:  exact 40 > prefix 32 > substr 24 > fuzzy 8
//   alias: exact 35 > prefix 28 > substr 21 > fuzzy 7
//   desc:  exact 30 > prefix 24 > substr 18 > fuzzy 6
//   cat:   exact 20 > prefix 16 > substr 12 > fuzzy 4

const SCORE_EXACT_NAME = 40;
const SCORE_PREFIX_NAME = 32;
const SCORE_SUBSTR_NAME = 24;
const SCORE_FUZZY_NAME = 8;
const SCORE_EXACT_ALIAS = 35;
const SCORE_PREFIX_ALIAS = 28;
const SCORE_SUBSTR_ALIAS = 21;
const SCORE_FUZZY_ALIAS = 7;
const SCORE_EXACT_DESC = 30;
const SCORE_PREFIX_DESC = 24;
const SCORE_SUBSTR_DESC = 18;
const SCORE_FUZZY_DESC = 6;
const SCORE_EXACT_CAT = 20;
const SCORE_PREFIX_CAT = 16;
const SCORE_SUBSTR_CAT = 12;
const SCORE_FUZZY_CAT = 4;

/// Bounded Levenshtein edit distance. Returns the edit distance between `a`
/// and `b`, capped at `maxDist + 1` if the true distance exceeds `maxDist`.
/// Uses Unicode code-point iteration via spread `[...str]` for correct
/// handling of umlauts (ä, ö, ü are single code points, not byte pairs).
///
/// Mirrors `levenshtein_bounded` in hp41-cli/src/help_data.rs exactly.
export function levenshteinBounded(a: string, b: string, maxDist: number): number {
    const aChars = [...a];
    const bChars = [...b];
    const n = aChars.length;
    const m = bChars.length;
    if (Math.abs(n - m) > maxDist) return maxDist + 1;

    let prev: number[] = Array.from({ length: m + 1 }, (_, i) => i);
    let curr: number[] = new Array(m + 1).fill(0);

    for (let i = 1; i <= n; i++) {
        curr[0] = i;
        for (let j = 1; j <= m; j++) {
            const cost = aChars[i - 1] === bChars[j - 1] ? 0 : 1;
            curr[j] = Math.min(curr[j - 1] + 1, prev[j] + 1, prev[j - 1] + cost);
        }
        // Early-exit: if all values in curr exceed maxDist, no need to continue.
        if (Math.min(...curr) > maxDist) return maxDist + 1;
        [prev, curr] = [curr, prev];
    }
    return prev[m];
}

/// Private tier scoring helper. Lowercases `field` and returns the best
/// tier score for query `q`:
///   exact == > prefix (starts_with or any word-start) > substring > fuzzy.
/// Fuzzy only attempted when `q.length >= 2` (P-HS-02 guard).
///
/// Mirrors `tier_score` in hp41-cli/src/help_data.rs exactly.
function tierScore(
    field: string,
    q: string,
    exact: number,
    prefix: number,
    substr: number,
    fuzzy: number,
): number {
    const f = field.toLowerCase();
    if (f === q) return exact;
    if (f.startsWith(q) || f.split(/\s+/).some(w => w.startsWith(q))) return prefix;
    if (f.includes(q)) return substr;
    if (q.length >= 2) {
        const maxDist = Math.max(1, Math.floor(q.length / 4));
        const minDist = f.length <= 20
            ? levenshteinBounded(f, q, maxDist)
            : Math.min(...f.split(/\s+/).map(w => levenshteinBounded(w, q, maxDist)));
        if (minDist <= maxDist) return fuzzy;
    }
    return 0;
}

/// Score a single HelpEntry against a pre-lowercased, pre-trimmed query `q`.
/// Returns the best tier score across all four searched fields:
/// `display_name`, `description`, `category`, and `search_aliases`.
/// A score of 0 means no field matched.
///
/// Mirrors `score_entry` in hp41-cli/src/help_data.rs exactly.
export function scoreEntry(entry: HelpEntry, q: string): number {
    const nameScore = tierScore(entry.display_name, q,
        SCORE_EXACT_NAME, SCORE_PREFIX_NAME, SCORE_SUBSTR_NAME, SCORE_FUZZY_NAME);
    const descScore = tierScore(entry.description, q,
        SCORE_EXACT_DESC, SCORE_PREFIX_DESC, SCORE_SUBSTR_DESC, SCORE_FUZZY_DESC);
    const catScore = tierScore(entry.category, q,
        SCORE_EXACT_CAT, SCORE_PREFIX_CAT, SCORE_SUBSTR_CAT, SCORE_FUZZY_CAT);
    const aliasScore = (entry.search_aliases ?? [])
        .map(a => tierScore(a, q, SCORE_EXACT_ALIAS, SCORE_PREFIX_ALIAS, SCORE_SUBSTR_ALIAS, SCORE_FUZZY_ALIAS))
        .reduce((a, b) => Math.max(a, b), 0);
    return Math.max(nameScore, descScore, catScore, aliasScore);
}

/// Return entries from `pool` ranked by relevance for query `q`.
/// The query is expected to be pre-lowercased and pre-trimmed by the caller.
/// For an empty query, the pool is returned unchanged (empty-query invariance
/// guard — mirrors `ranked_help_entries` Rust behavior).
/// Non-empty query: filters to score > 0, sorts by (score DESC, display_name ASC).
///
/// Mirrors `ranked_help_entries` in hp41-cli/src/help_data.rs exactly.
export function rankedEntries(pool: readonly HelpEntry[], q: string): readonly HelpEntry[] {
    if (q === '') return pool;
    const scored: Array<[number, HelpEntry]> = [];
    for (const entry of pool) {
        const s = scoreEntry(entry, q);
        if (s > 0) scored.push([s, entry]);
    }
    scored.sort(([sa, ea], [sb, eb]) => {
        if (sb !== sa) return sb - sa;
        return ea.display_name.localeCompare(eb.display_name);
    });
    return scored.map(([, e]) => e);
}

/// Phase 52 Plan 52-01: Merged accessor returning built-in + Math Pac I + Stat 1 Pac + Time Pac + Advantage Pac + X-MEM entries.
///
/// UPDATED from Phase 46 Plan 46-02 (5-pool) to 6-pool concatenation.
/// Parallel to hp41-cli/src/help_data.rs::help_entries_all() (Phase 52 D-52.1 6-pool chain).
/// Used by HelpOverlay.tsx to obtain the full entry pool; the overlay then partitions
/// entries by `entry.xrom` into sections (D-31.8 extended for Advantage Pac and X-MEM).
/// Pitfall 5: do NOT create a parallel helpEntriesAll6() — update in-place so all
/// existing callers (HelpOverlay.tsx) automatically pick up X-MEM entries.
let cachedAllEntries: readonly HelpEntry[] | null = null;

export function helpEntriesAll(): readonly HelpEntry[] {
    // Memoized (WR-04): the six source pools are static JSON imports that never
    // change at runtime, so the concatenation is computed once and the same
    // stable reference is returned on every call — preserving referential
    // equality for React memoization. Mirrors the zero-alloc Rust iterator.
    if (cachedAllEntries === null) {
        cachedAllEntries = [...helpEntries(), ...helpEntriesMath1(), ...helpEntriesStat1(), ...helpEntriesTime(), ...helpEntriesAdvantage(), ...helpEntriesXmem()];
    }
    return cachedAllEntries;
}
