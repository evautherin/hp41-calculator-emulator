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
import { allFunctionsEntries, xeqToken, helpEntriesAll } from './help_data';

describe('allFunctionsEntries', () => {
    it('includes implemented key_path:null entries — e.g. SIN and CLRG', () => {
        const entries = allFunctionsEntries();
        const names = entries.map(e => e.display_name);
        expect(names).toContain('SIN');
        expect(names).toContain('CLRG');
        expect(names).toContain('AVIEW');
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
        const allImplemented = helpEntriesAll().filter(e => e.status === 'implemented');
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
