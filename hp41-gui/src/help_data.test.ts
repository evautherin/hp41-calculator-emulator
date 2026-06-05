// Phase lu0 Item B1 — Vitest tests for allFunctionsEntries() + xeqToken() tappability resolver.
//
// Coverage rationale (D-lu0-02):
//   - allFunctionsEntries() returns every implemented entry REGARDLESS of key_path.
//     This is the All Functions tab data source — the original D-26.8 "exclude
//     key_path:null" rule applies only to the Keyboard Shortcuts tab.
//   - xeqToken() gates which entries are tappable (runnable by XEQ-by-name).
//     The 61 NON_TAPPABLE op_variants are listed but rendered non-interactive (D-07:
//     never dispatch an unresolvable id).
//
// Phase C1 guardrail (appended below):
//   Asserts every implemented entry from helpEntriesAll() is also in
//   allFunctionsEntries(). Catches any future regression that re-introduces a
//   key_path-style exclusion hiding an implemented function from the All Functions tab.
//   Original gap: 74 implemented cv-pool functions (SIN, LN, SQRT, AVIEW, CLST, CLRG,
//   CLA, …) were invisible because of the `key_path !== null` filter. This test closes
//   that hole at the overlay-dataset boundary.

import { describe, it, expect } from 'vitest';
import { allFunctionsEntries, xeqToken, helpEntriesAll, OVERLAY_HIDDEN_ALIASES, type HelpEntry, scoreEntry, rankedEntries } from './help_data';
// Phase 61 Plan 61-03 (HSQUAL-02): canonical CLI↔GUI drift-guard fixture.
// Static JSON import (resolveJsonModule enabled). The SAME fixture is asserted
// by the Rust test (hp41-cli/tests/phase61_help_search_aliases.rs, via
// include_str!) and by this Vitest test — both MUST produce the same top-1 for
// every query. Each fixture query is already lowercased+trimmed.
import parityFixture from '../../docs/fixtures/help_search_parity.json';

describe('allFunctionsEntries', () => {
    it('includes implemented key_path:null entries — e.g. SIN and CLRG', () => {
        const entries = allFunctionsEntries();
        const names = entries.map(e => e.display_name);
        expect(names).toContain('SIN');
        expect(names).toContain('CLRG');
        expect(names).toContain('AVIEW');
        expect(names).toContain('CLA');
    });

    it('hides non-authentic legacy aliases (CLRALPHA) but keeps the real mnemonic (CLA)', () => {
        // s17: AlphaClear ("CLRALPHA") is an implemented legacy alias of Cla ("CLA").
        // It stays resolvable for old saves but must NOT show in the index (CLA is shown).
        const variants = allFunctionsEntries().map(e => e.op_variant);
        const names = allFunctionsEntries().map(e => e.display_name);
        expect(variants).not.toContain('AlphaClear');
        expect(names).not.toContain('CLRALPHA');
        expect(names).toContain('CLA');
    });

    it('excludes non-implemented (deferred-v3 / na) entries', () => {
        const entries = allFunctionsEntries();
        for (const e of entries) {
            expect(e.status, `op_variant ${e.op_variant} should be implemented`).toBe('implemented');
        }
    });

    it('includes entries from all 6 pools (built-in, math1, stat1, time, advantage, xmem)', () => {
        const entries = allFunctionsEntries();
        // cv built-in — SIN has no xrom
        const hasCvEntry = entries.some(e => e.display_name === 'SIN' && !e.xrom);
        // Math 1 Pac
        const hasMath1Entry = entries.some(e => e.xrom?.module === 'Math 1');
        // Stat 1 Pac
        const hasStat1Entry = entries.some(e => e.xrom?.module === 'Stat 1');
        // Time Pac
        const hasTimeEntry = entries.some(e => e.xrom?.module === 'Time');
        // Advantage Pac
        const hasAdvEntry = entries.some(e => e.xrom?.module === 'Adv Conv' || e.xrom?.module === 'Adv Math');
        // X-MEM
        const hasXmemEntry = entries.some(e => !e.xrom && e.category === 'Extended Memory');
        expect(hasCvEntry).toBe(true);
        expect(hasMath1Entry).toBe(true);
        expect(hasStat1Entry).toBe(true);
        expect(hasTimeEntry).toBe(true);
        expect(hasAdvEntry).toBe(true);
        expect(hasXmemEntry).toBe(true);
    });

    it('returns a stable reference (memoized)', () => {
        expect(allFunctionsEntries()).toBe(allFunctionsEntries());
    });
});

describe('xeqToken', () => {
    it('returns display_name for runnable entries (Sin, Clreg)', () => {
        expect(xeqToken({ op_variant: 'Sin', display_name: 'SIN' })).toBe('SIN');
        expect(xeqToken({ op_variant: 'Clreg', display_name: 'CLRG' })).toBe('CLRG');
        expect(xeqToken({ op_variant: 'Pi', display_name: 'PI' })).toBe('PI');
    });

    it('returns null for parameterized ops (StoReg, RclReg)', () => {
        expect(xeqToken({ op_variant: 'StoReg', display_name: 'STO' })).toBeNull();
        expect(xeqToken({ op_variant: 'RclReg', display_name: 'RCL' })).toBeNull();
    });

    it('returns null for immediate stack/entry keys (Add, Enter, Clx, Chs)', () => {
        expect(xeqToken({ op_variant: 'Add', display_name: '+' })).toBeNull();
        expect(xeqToken({ op_variant: 'Enter', display_name: 'ENTER' })).toBeNull();
        expect(xeqToken({ op_variant: 'Clx', display_name: 'CLX' })).toBeNull();
        expect(xeqToken({ op_variant: 'Chs', display_name: 'CHS' })).toBeNull();
    });

    it('returns null for mode/composite placeholders (AlphaToggle, PrgmMode)', () => {
        expect(xeqToken({ op_variant: 'AlphaToggle', display_name: 'ALPHA' })).toBeNull();
        expect(xeqToken({ op_variant: 'PrgmMode', display_name: 'PRGM' })).toBeNull();
    });
});

// ── Phase 58: search_aliases backward-compat assertion (D-58.2 / HSDATA-02) ─────
//
// The field is optional in the TS interface so existing JSON imports (which lack
// the key) still typecheck. This test confirms that an object typed as HelpEntry
// without search_aliases yields `undefined` for that field — no runtime error.

describe('HelpEntry.search_aliases backward-compat', () => {
    it('object without search_aliases field yields undefined (D-58.2)', () => {
        // Simulate what happens when a JSON pool entry (no search_aliases key)
        // is accessed — the field must be undefined, not an error.
        const entry = JSON.parse(
            '{"op_variant":"Pi","display_name":"PI","category":"Math","status":"implemented","phase":"21","key_path":"f-7","description":"Push pi onto X"}'
        ) as HelpEntry;
        expect(entry.search_aliases).toBeUndefined();
    });
});

// ── Phase C1: Overlay-completeness guardrail ──────────────────────────────────
//
// Rationale: existing parity tests (function_matrix_parity.rs) assert JSON↔Op
// completeness, but NOT overlay visibility. This test closes the overlay-dataset
// boundary gap: if any future change re-introduces a key_path-style exclusion
// that hides an implemented function from the All Functions dataset, this test
// fails with the exact list of missing op_variants.
//
// The original gap (74 implemented functions invisible) went undetected because
// there was no test at the overlay-visibility boundary — only at JSON↔Op parity.

describe('C1: allFunctionsEntries completeness guardrail', () => {
    it('every implemented entry from helpEntriesAll() is in allFunctionsEntries()', () => {
        const allImplemented = helpEntriesAll().filter(
            e => e.status === 'implemented' && !OVERLAY_HIDDEN_ALIASES.has(e.op_variant),
        );
        const allFnVariants = new Set(allFunctionsEntries().map(e => e.op_variant));

        const missing = allImplemented
            .filter(e => !allFnVariants.has(e.op_variant))
            .map(e => `${e.op_variant} (${e.display_name})`);

        expect(
            missing,
            `These implemented functions are missing from allFunctionsEntries() — they would be invisible in the All Functions tab: ${missing.join(', ')}`
        ).toHaveLength(0);
    });
});

// ── Phase 59 Wave 0 — RED scorer contract ─────────────────────────────────────
//
// These tests describe the full scoreEntry/rankedEntries API before the scorer
// is implemented. They FAIL (RED) until plan 59-03 adds the exports to
// help_data.ts. That is the intended Wave-0 state — not a defect.
//
// Tier constants (from 59-RESEARCH.md §Tiered Scoring):
//   name:  exact 40 > prefix 32 > substr 24 > fuzzy 8
//   alias: exact 35 > prefix 28 > substr 21 > fuzzy 7
//   desc:  exact 30 > prefix 24 > substr 18 > fuzzy 6
//   cat:   exact 20 > prefix 16 > substr 12 > fuzzy 4
//
// Mirror of hp41-cli/tests/phase59_help_search.rs (CLI↔GUI parity contract).
// q is pre-lowercased+trimmed before passing to scoreEntry.

// Synthetic HelpEntry fixture (mirrors the Rust make_entry helper)
function makeEntry(
    display_name: string,
    description: string,
    category: string,
    aliases: string[],
): HelpEntry {
    return {
        op_variant: display_name,
        display_name,
        category,
        status: 'implemented',
        phase: null,
        key_path: null,
        description,
        divergences: [],
        xrom: undefined,
        search_aliases: aliases,
    };
}

describe('Phase 59 scorer — scoreEntry', () => {

    // ── Tier order (HSMATCH-02) ───────────────────────────────────────────────
    it('tier order: exact(40) > prefix(32) > substring(24) > fuzzy(8) on display_name', () => {
        // Exact name match
        const exactEntry = makeEntry('tvm', 'Time Value of Money', 'Finance', []);
        const exactScore = scoreEntry(exactEntry, 'tvm');
        expect(exactScore).toBe(40);

        // Word-prefix match: "tv" is a prefix of "tvm"
        const prefixEntry = makeEntry('tvm', 'Time Value of Money', 'Finance', []);
        const prefixScore = scoreEntry(prefixEntry, 'tv');
        expect(prefixScore).toBe(32);

        // Substring (non-prefix): "vm" is in "tvm" but not a prefix
        const substrEntry = makeEntry('tvm', 'Time Value of Money', 'Finance', []);
        const substrScore = scoreEntry(substrEntry, 'vm');
        expect(substrScore).toBe(24);

        // Fuzzy (1 typo): query "tvm" vs display_name "tvn" — 1 edit, threshold max(1,3/4)=1
        const fuzzyEntry = makeEntry('tvn', 'Time Value of Money', 'Finance', []);
        const fuzzyScore = scoreEntry(fuzzyEntry, 'tvm');
        expect(fuzzyScore).toBe(8);

        // Strict ordering
        expect(exactScore).toBeGreaterThan(prefixScore);
        expect(prefixScore).toBeGreaterThan(substrScore);
        expect(substrScore).toBeGreaterThan(fuzzyScore);
        expect(fuzzyScore).toBeGreaterThan(0);
    });

    // ── Fuzzy typo: Zineszins → TVM (HSMATCH-03, HSUX-02) ───────────────────
    it('fuzzy typo: scoreEntry(tvm, "zineszins") > 0 with alias "Zinseszins"', () => {
        // "zineszins" is 1 edit from "zinseszins"; threshold = max(1, 9/4) = 2
        const tvm = makeEntry('TVM', 'Time Value of Money solver', 'Finance', ['Zinseszins', 'compound interest']);
        const score = scoreEntry(tvm, 'zineszins');
        expect(score).toBeGreaterThan(0);

        // Control entry without the alias must score 0
        const sin = makeEntry('SIN', 'Sine of X', 'Trigonometry', []);
        expect(scoreEntry(sin, 'zineszins')).toBe(0);
    });

    // ── Alias exact: Wurzel → SQRT (HSUX-02) ─────────────────────────────────
    it('alias exact: scoreEntry(sqrt, "wurzel") == 35 with alias "Wurzel"', () => {
        const sqrt = makeEntry('SQRT', 'Square root of X', 'Math', ['Wurzel']);
        const score = scoreEntry(sqrt, 'wurzel');
        expect(score).toBe(35);
    });

    // ── DE + EN alias resolution (HSMATCH-05) ────────────────────────────────
    it('alias DE+EN: both "zinseszins" and "compound interest" score > 0 on TVM', () => {
        const tvm = makeEntry('TVM', 'Time Value of Money solver', 'Finance', ['Zinseszins', 'compound interest']);
        expect(scoreEntry(tvm, 'zinseszins')).toBeGreaterThan(0);
        expect(scoreEntry(tvm, 'compound interest')).toBeGreaterThan(0);

        // Alias-less entry scores 0 for both queries
        const aliasless = makeEntry('SIN', 'Sine of X', 'Trigonometry', []);
        expect(scoreEntry(aliasless, 'zinseszins')).toBe(0);
        expect(scoreEntry(aliasless, 'compound interest')).toBe(0);
    });

    // ── All four fields scored (HSMATCH-01) ───────────────────────────────────
    it('all-four-fields: entries matching via display_name / description / category / alias each score > 0', () => {
        // Match only via display_name
        const nameEntry = makeEntry('SIN', 'No-match description', 'No-match-cat', []);
        expect(scoreEntry(nameEntry, 'sin')).toBeGreaterThan(0);

        // Match only via description
        const descEntry = makeEntry('NOOP', 'trigonometry function', 'No-match-cat', []);
        expect(scoreEntry(descEntry, 'trigonometry')).toBeGreaterThan(0);

        // Match only via category
        const catEntry = makeEntry('NOOP2', 'no match description here', 'Finance', []);
        expect(scoreEntry(catEntry, 'finance')).toBeGreaterThan(0);

        // Match only via alias
        const aliasEntry = makeEntry('NOOP3', 'no match description here', 'No-match-cat', ['Zinseszins']);
        expect(scoreEntry(aliasEntry, 'zinseszins')).toBeGreaterThan(0);

        // No field matches → score 0
        const noMatch = makeEntry('NOOP4', 'no match here', 'Other', []);
        expect(scoreEntry(noMatch, 'zinseszins')).toBe(0);
    });
});

describe('Phase 59 ranking — rankedEntries', () => {

    // ── Sorted score DESC then display_name ASC ───────────────────────────────
    it('ranked output: top entry for "wurzel" is SQRT; zero-score entries absent', () => {
        const pool: readonly HelpEntry[] = [
            makeEntry('SIN', 'Sine of X', 'Trigonometry', []),
            makeEntry('COS', 'Cosine of X', 'Trigonometry', []),
            makeEntry('SQRT', 'Square root of X', 'Math', ['Wurzel']),
            makeEntry('TAN', 'Tangent of X', 'Trigonometry', []),
        ];

        const results = rankedEntries(pool, 'wurzel');
        expect(results.length).toBeGreaterThan(0);
        // SQRT must be the top result (alias exact = 35, others score 0)
        expect(results[0].display_name).toBe('SQRT');
        // Zero-score entries (SIN, COS, TAN) must be absent
        expect(results.find(e => e.display_name === 'SIN')).toBeUndefined();
        expect(results.find(e => e.display_name === 'COS')).toBeUndefined();
        expect(results.find(e => e.display_name === 'TAN')).toBeUndefined();
    });

    it('rankedEntries: ties broken by display_name ASC', () => {
        // Two entries with identical score — should sort by display_name
        const pool: readonly HelpEntry[] = [
            makeEntry('ZEBRA', 'No-match desc', 'Finance', ['compound interest']),
            makeEntry('ALPHA', 'No-match desc', 'Finance', ['compound interest']),
        ];

        const results = rankedEntries(pool, 'compound interest');
        expect(results.length).toBe(2);
        // Same score — alphabetical order: ALPHA < ZEBRA
        expect(results[0].display_name).toBe('ALPHA');
        expect(results[1].display_name).toBe('ZEBRA');
    });

    it('rankedEntries: empty string query returns unranked passthrough (same as filterHelpEntries empty-guard)', () => {
        const pool: readonly HelpEntry[] = [
            makeEntry('SIN', 'Sine of X', 'Trigonometry', []),
            makeEntry('COS', 'Cosine of X', 'Trigonometry', []),
        ];
        // For empty query, rankedEntries should return the pool unchanged
        // (mirrors the Rust empty-query invariance guard)
        const results = rankedEntries(pool, '');
        expect(results.length).toBe(pool.length);
    });
});

// ── Phase 61 Plan 61-03 — Real-data quality gates + CLI↔GUI drift guard ────────
//
// HSQUAL-01 / HSQUAL-02. Unlike the Phase-59 RED contract above (synthetic
// makeEntry fixtures), this block runs the scorer against the LIVE six-pool JSON
// via rankedEntries(allFunctionsEntries(), q). It is the GUI mirror of the Rust
// suite hp41-cli/tests/phase61_help_search_aliases.rs and proves:
//   - all four scoring tiers (exact > prefix > substring > fuzzy) on real data,
//   - DE+EN alias resolution top-1 (zeitwert des geldes / zinseszins /
//     compound interest → TVM; square root / quadratwurzel / wurzel → SQRT),
//   - a fuzzy-hit top-1 (sqirt → SQRT),
//   - empty-query passthrough per HSMATCH-04 (rankedEntries(pool, '') === pool),
//   - the canonical parity fixture (docs/fixtures/help_search_parity.json) —
//     the SAME top-1 the Rust test asserts (drift guard).
//
// Queries are pre-lowercased+trimmed by the caller, matching the contract of
// rankedEntries (the Rust ranked_help_entries lowercases upstream).
describe('Phase 61 — help search quality gates (real six-pool data)', () => {
    const pool = allFunctionsEntries();

    // ── All four scoring tiers against real data (HSQUAL-01) ──────────────────
    it('tier: exact name match — "tvm" top-1 is TVM (exact, score 40)', () => {
        const results = rankedEntries(pool, 'tvm');
        expect(results.length).toBeGreaterThan(0);
        expect(results[0].display_name).toBe('TVM');
        // Top entry must be an exact-name hit (highest tier).
        expect(scoreEntry(results[0], 'tvm')).toBe(40);
    });

    it('tier: prefix name match — "sqr" word-prefix hits SQRT (prefix, score 32)', () => {
        const sqrt = pool.find(e => e.display_name === 'SQRT');
        expect(sqrt, 'SQRT must exist in the live cv pool').toBeDefined();
        // "sqr" is a prefix of "sqrt" but not exact → prefix tier on display_name.
        expect(scoreEntry(sqrt as HelpEntry, 'sqr')).toBe(32);
    });

    it('tier: substring name match — "vm" is a non-prefix substring of "TVM" (substr, score 24)', () => {
        const tvm = pool.find(e => e.display_name === 'TVM');
        expect(tvm, 'TVM must exist in the live advantage pool').toBeDefined();
        // "vm" is inside "tvm" but neither exact nor a word-prefix → substring tier.
        expect(scoreEntry(tvm as HelpEntry, 'vm')).toBe(24);
    });

    it('tier: fuzzy name match — "sqirt" (1 typo) is a fuzzy hit on SQRT (fuzzy, score 8)', () => {
        const sqrt = pool.find(e => e.display_name === 'SQRT');
        expect(sqrt, 'SQRT must exist in the live cv pool').toBeDefined();
        // "sqirt" → "sqrt" is 1 edit; not exact/prefix/substring → fuzzy name tier.
        expect(scoreEntry(sqrt as HelpEntry, 'sqirt')).toBe(8);
    });

    // ── DE + EN alias resolution top-1 (HSQUAL-01, mirrors Rust suite) ─────────
    it('alias DE+EN → TVM: "zeitwert des geldes" / "zinseszins" / "compound interest" each rank TVM top-1', () => {
        for (const q of ['zeitwert des geldes', 'zinseszins', 'compound interest']) {
            const results = rankedEntries(pool, q);
            expect(results.length, `query "${q}" must return results`).toBeGreaterThan(0);
            expect(results[0].display_name, `query "${q}" top-1`).toBe('TVM');
        }
    });

    it('alias DE+EN → SQRT: "square root" / "quadratwurzel" / "wurzel" each rank SQRT top-1', () => {
        for (const q of ['square root', 'quadratwurzel', 'wurzel']) {
            const results = rankedEntries(pool, q);
            expect(results.length, `query "${q}" must return results`).toBeGreaterThan(0);
            expect(results[0].display_name, `query "${q}" top-1`).toBe('SQRT');
        }
    });

    // ── Fuzzy-hit top-1 (HSQUAL-01) ───────────────────────────────────────────
    it('fuzzy top-1: "sqirt" ranks SQRT first on the live pool', () => {
        const results = rankedEntries(pool, 'sqirt');
        expect(results.length).toBeGreaterThan(0);
        expect(results[0].display_name).toBe('SQRT');
    });

    // ── Empty-query passthrough per HSMATCH-04 ────────────────────────────────
    // TS contract: the empty string '' is the correct input and returns the pool
    // UNCHANGED (no filtering). This differs from the Rust whitespace handling —
    // do NOT import Rust behavior here. Assert same length AND same order.
    it('empty-query passthrough: rankedEntries(pool, "") returns the pool unchanged (HSMATCH-04)', () => {
        const results = rankedEntries(pool, '');
        // Same reference (rankedEntries returns the pool as-is for '').
        expect(results).toBe(pool);
        expect(results.length).toBe(pool.length);
        // Order preserved element-for-element.
        for (let i = 0; i < pool.length; i++) {
            expect(results[i].op_variant).toBe(pool[i].op_variant);
        }
    });
});

// ── Phase 61 Plan 61-03 — CLI↔GUI parity fixture loop (HSQUAL-02 drift guard) ──
//
// The canonical fixture docs/fixtures/help_search_parity.json drives BOTH the
// Rust test (61-02, via include_str!) and this GUI test (via static JSON import).
// For every fixture query, rankedEntries(allFunctionsEntries(), query)[0] must
// equal expected_top[0] — identical to the Rust assertion. If a query's top-1
// diverges, that is a real CLI↔GUI drift / data regression: report it as a
// blocker — do NOT weaken the assertion or edit the frozen fixture/JSON data.
describe('Phase 61 — CLI↔GUI parity fixture (top-1 drift guard, HSQUAL-02)', () => {
    const pool = allFunctionsEntries();

    it('fixture has the expected canonical shape (top_n=1 queries)', () => {
        expect(Array.isArray(parityFixture.queries)).toBe(true);
        expect(parityFixture.queries.length).toBeGreaterThan(0);
        for (const c of parityFixture.queries) {
            expect(c.top_n).toBe(1);
            // Each query is already lowercased (canonical fixture invariant).
            expect(c.query).toBe(c.query.toLowerCase());
            expect(c.expected_top.length).toBeGreaterThanOrEqual(1);
        }
    });

    for (const c of parityFixture.queries) {
        it(`parity: "${c.query}" → top-1 ${c.expected_top[0]} (${c.note})`, () => {
            const results = rankedEntries(pool, c.query);
            expect(results.length, `query "${c.query}" must return at least one result`).toBeGreaterThan(0);
            expect(results[0].display_name, `query "${c.query}" top-1 must match Rust`).toBe(c.expected_top[0]);
        });
    }
});
