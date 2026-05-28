// Phase 26 Plan 03 — Vitest tests for HelpOverlay + help_data.ts (D-26.8).
//
// Tests cover:
//   - help_data.ts: helpEntries entry count parity with the source JSON
//     (drift-catch), filterHelpEntries semantics, helpOverlayRows category
//     grouping + null-key_path filter (D-26.8)
//   - HelpOverlay.tsx: open/close rendering, search input filtering,
//     Esc keystroke triggers onClose, close-button click triggers onClose
//
// Phase 31-04 D-31.8 additions:
//   - Two top-level section headings "HP-41CV (built-in)" and "Math 1 Pac (XROM 7)"
//   - Math 1 Pac section contains per-category 2nd-level headers (D-31.9)
//   - Clicking section heading toggles aria-expanded
//   - helpEntriesMath1() + helpEntriesAll() accessor tests
//
// Phase 36 Plan 36-03 additions:
//   - Third section "Stat 1 Pac (XROM 2)" — STAT-GUI-03 / STAT-GUI-04
//   - helpEntriesStat1() drift-catch + xrom-field assertions
//   - helpEntriesAll() updated from 2-pool to 3-pool length assertion
//   - sectionButtons.length updated from 2 to 3
//
// Phase 41 Plan 41-02 additions (TIME-GUI-03):
//   - Fourth section "Time Pac (XROM 26)"
//   - helpEntriesTime() drift-catch + xrom-field assertions
//   - helpEntriesAll() updated from 3-pool to 4-pool length assertion
//   - sectionButtons.length updated from 3 to 4
//
// Phase 46 additions (ADV-GUI-03):
//   - Fifth+sixth sections "Advantage Pac (XROM 22)" and "Advantage Pac (XROM 24)"
//   - helpEntriesAdvantage() drift-catch + xrom-field assertions
//   - helpEntriesAll() updated from 4-pool to 5-pool length assertion
//   - sectionButtons.length updated from 4 to 6

import { describe, it, expect } from 'vitest';
import { render, fireEvent } from '@testing-library/react';
import { HelpOverlay } from './HelpOverlay';
import { helpEntries, helpOverlayRows, filterHelpEntries, helpEntriesMath1, helpEntriesAll, helpEntriesStat1, helpEntriesTime, helpEntriesAdvantage, helpEntriesXmem } from './help_data';
import sourceJson from '../../docs/hp41cv-functions.json';
import math1Json from '../../docs/hp41-math1-functions.json';
import stat1Json from '../../docs/hp41-stat1-functions.json';
import timeJson from '../../docs/hp41-time-functions.json';
import advJson from '../../docs/hp41-advantage-functions.json';

describe('help_data', () => {
    it('helpEntries returns all entries from docs/hp41cv-functions.json (drift-catch)', () => {
        const allSource = sourceJson as unknown[];
        expect(helpEntries().length).toBe(allSource.length);
        // Sanity floor: the canonical JSON has 130+ entries as of Phase 25.
        expect(helpEntries().length).toBeGreaterThanOrEqual(130);
    });

    // Phase 31-04 tests for helpEntriesMath1() and helpEntriesAll()
    it('helpEntriesMath1 returns all entries from docs/hp41-math1-functions.json (drift-catch)', () => {
        const allMath1Source = math1Json as unknown[];
        expect(helpEntriesMath1().length).toBe(allMath1Source.length);
        // Sanity floor: the Math Pac I JSON has at least 40 entries (Phase 28 scope).
        expect(helpEntriesMath1().length).toBeGreaterThanOrEqual(40);
    });

    it('helpEntriesMath1 entries all have xrom field with module "Math 1"', () => {
        for (const entry of helpEntriesMath1()) {
            expect(entry.xrom, `entry ${entry.op_variant} should have xrom field`).toBeTruthy();
            expect(entry.xrom!.module).toBe('Math 1');
            expect(entry.xrom!.module_id).toBe(7);
        }
    });

    it('helpEntriesAll returns concatenation of all 6 pools', () => {
        const all = helpEntriesAll();
        expect(all.length).toBe(
            helpEntries().length + helpEntriesMath1().length + helpEntriesStat1().length + helpEntriesTime().length + helpEntriesAdvantage().length + helpEntriesXmem().length
        );
        const hp41cvCount = helpEntries().length;
        const math1Count = helpEntriesMath1().length;
        const stat1Count = helpEntriesStat1().length;
        const timeCount = helpEntriesTime().length;
        for (let i = 0; i < hp41cvCount; i++) {
            expect(all[i].xrom, `built-in entry at index ${i} should have no xrom`).toBeUndefined();
        }
        for (let i = hp41cvCount; i < hp41cvCount + math1Count; i++) {
            expect(all[i].xrom, `Math 1 entry at index ${i} should have xrom`).toBeTruthy();
            expect(all[i].xrom!.module, `Math 1 entry at index ${i} should have module === 'Math 1'`).toBe('Math 1');
        }
        for (let i = hp41cvCount + math1Count; i < hp41cvCount + math1Count + stat1Count; i++) {
            expect(all[i].xrom, `Stat 1 entry at index ${i} should have xrom`).toBeTruthy();
            expect(all[i].xrom!.module, `Stat 1 entry at index ${i} should have module === 'Stat 1'`).toBe('Stat 1');
        }
        const timeStart = hp41cvCount + math1Count + stat1Count;
        for (let i = timeStart; i < timeStart + timeCount; i++) {
            expect(all[i].xrom, `Time entry at index ${i} should have xrom`).toBeTruthy();
            expect(all[i].xrom!.module, `Time entry at index ${i} should have module === 'Time'`).toBe('Time');
        }
        const advStart = timeStart + timeCount;
        const advCount = helpEntriesAdvantage().length;
        for (let i = advStart; i < advStart + advCount; i++) {
            expect(all[i].xrom, `Advantage entry at index ${i} should have xrom`).toBeTruthy();
            expect([22, 24], `Advantage entry at index ${i} should have module_id 22 or 24`).toContain(all[i].xrom!.module_id);
        }
        const xmemStart = advStart + advCount;
        for (let i = xmemStart; i < all.length; i++) {
            expect(all[i].xrom, `X-MEM entry at index ${i} should have no xrom (OS built-in, not XROM)`).toBeUndefined();
            expect(all[i].category, `X-MEM entry at index ${i} should have category 'Extended Memory'`).toBe('Extended Memory');
        }
    });

    // Phase 36 Plan 36-03: Stat 1 Pac data-layer drift-catch tests (STAT-GUI-04)
    it('helpEntriesStat1 returns all entries from docs/hp41-stat1-functions.json (drift-catch)', () => {
        const allStat1Source = stat1Json as unknown[];
        expect(helpEntriesStat1().length).toBe(allStat1Source.length);
        // Sanity floor: the Stat 1 Pac JSON has 26 entries (Phase 33-34 scope).
        expect(helpEntriesStat1().length).toBeGreaterThanOrEqual(26);
    });

    it('helpEntriesStat1 entries all have xrom field with module "Stat 1"', () => {
        for (const entry of helpEntriesStat1()) {
            expect(entry.xrom, `entry ${entry.op_variant} should have xrom field`).toBeTruthy();
            expect(entry.xrom!.module).toBe('Stat 1');
            expect(entry.xrom!.module_id).toBe(2);
        }
    });

    // Phase 41 Plan 41-02: Time Pac data-layer drift-catch tests (TIME-GUI-03)
    it('helpEntriesTime returns all entries from docs/hp41-time-functions.json (drift-catch)', () => {
        const allTimeSource = timeJson as unknown[];
        expect(helpEntriesTime().length).toBe(allTimeSource.length);
        // Sanity floor: the Time Pac JSON has 35 entries (Phase 38-39 scope, D-39.9).
        expect(helpEntriesTime().length).toBeGreaterThanOrEqual(35);
    });

    it('helpEntriesTime entries all have xrom field with module "Time"', () => {
        for (const entry of helpEntriesTime()) {
            expect(entry.xrom, `entry ${entry.op_variant} should have xrom field`).toBeTruthy();
            // CRITICAL: module is "Time" (not "TIME", "Time Pac", or "TIME 2C") — D-39.9 / Pitfall 6
            expect(entry.xrom!.module).toBe('Time');
            expect(entry.xrom!.module_id).toBe(26);
        }
    });

    it('helpEntriesAdvantage returns all entries from docs/hp41-advantage-functions.json (drift-catch)', () => {
        const allAdvSource = advJson as unknown[];
        expect(helpEntriesAdvantage().length).toBe(allAdvSource.length);
        expect(helpEntriesAdvantage().length).toBeGreaterThanOrEqual(114);
    });

    it('helpEntriesAdvantage entries all have xrom field', () => {
        for (const entry of helpEntriesAdvantage()) {
            expect(entry.xrom, `entry ${entry.op_variant} should have xrom field`).toBeTruthy();
            expect([22, 24]).toContain(entry.xrom!.module_id);
        }
    });

    it('filterHelpEntries with empty query returns only key_path != null entries', () => {
        const all = helpEntries();
        const nonNullCount = all.filter(e => e.key_path !== null).length;
        expect(filterHelpEntries('').length).toBe(nonNullCount);
        // Floor: the canonical JSON has at least 30 keyboard-bound ops.
        expect(filterHelpEntries('').length).toBeGreaterThanOrEqual(30);
    });

    it('filterHelpEntries narrows results by display_name match', () => {
        const result = filterHelpEntries('STO');
        expect(result.length).toBeGreaterThan(0);
        for (const entry of result) {
            const matches =
                entry.display_name.toLowerCase().includes('sto') ||
                entry.description.toLowerCase().includes('sto') ||
                entry.category.toLowerCase().includes('sto');
            expect(matches, `entry ${entry.op_variant} should match 'sto'`).toBe(true);
        }
    });

    it('filterHelpEntries narrows results by category match', () => {
        const result = filterHelpEntries('arithmetic');
        expect(result.length).toBeGreaterThan(0);
        // Every returned entry should be in the Arithmetic category OR have
        // 'arithmetic' substring in name/description.
        for (const entry of result) {
            const matches =
                entry.display_name.toLowerCase().includes('arithmetic') ||
                entry.description.toLowerCase().includes('arithmetic') ||
                entry.category.toLowerCase().includes('arithmetic');
            expect(matches).toBe(true);
        }
    });

    it('filterHelpEntries with no matching query returns empty array', () => {
        expect(filterHelpEntries('xyzzy_no_such_function').length).toBe(0);
    });

    it('helpOverlayRows produces category headers in JSON declaration order (unique)', () => {
        const rows = helpOverlayRows();
        const headers = rows.filter(r => r.isHeader).map(r => r.category);
        // Each header should appear at most once (no duplicate category headings).
        expect(new Set(headers).size).toBe(headers.length);
        // At least one header should exist.
        expect(headers.length).toBeGreaterThan(0);
    });

    it('helpOverlayRows excludes null-key_path entries from rendered rows (D-26.8)', () => {
        const rows = helpOverlayRows();
        const dataRows = rows.filter(r => !r.isHeader);
        // Every data row must have a non-empty key (which derived from
        // non-null key_path).
        for (const row of dataRows) {
            expect(row.key).not.toBe('');
        }
        // Total data rows should equal the count of non-null-key_path entries.
        const expected = helpEntries().filter(e => e.key_path !== null).length;
        expect(dataRows.length).toBe(expected);
    });

    it('helpOverlayRows has header rows with empty key/op fields', () => {
        const headers = helpOverlayRows().filter(r => r.isHeader);
        for (const h of headers) {
            expect(h.key).toBe('');
            expect(h.op).toBe('');
            expect(h.desc.startsWith('=== ')).toBe(true);
            expect(h.desc.endsWith(' ===')).toBe(true);
        }
    });
});

describe('HelpOverlay', () => {
    it('renders nothing when open=false', () => {
        const { container } = render(<HelpOverlay open={false} onClose={() => {}} />);
        expect(container.querySelector('.help-overlay')).toBeNull();
    });

    it('renders the overlay when open=true', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        expect(container.querySelector('.help-overlay')).not.toBeNull();
        expect(container.querySelector('.help-overlay-search')).not.toBeNull();
        expect(container.querySelector('.help-overlay-content')).not.toBeNull();
    });

    it('initial render shows entries grouped by category', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        const headings = container.querySelectorAll('.help-overlay-category-heading');
        expect(headings.length).toBeGreaterThan(0);
        const rows = container.querySelectorAll('.help-overlay-row');
        expect(rows.length).toBeGreaterThan(0);
    });

    it('search input narrows the rendered rows', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        const initialRows = container.querySelectorAll('.help-overlay-row').length;
        const searchInput = container.querySelector('.help-overlay-search') as HTMLInputElement;
        expect(searchInput).not.toBeNull();
        fireEvent.change(searchInput, { target: { value: 'sin' } });
        const filteredRows = container.querySelectorAll('.help-overlay-row').length;
        expect(filteredRows).toBeLessThan(initialRows);
        expect(filteredRows).toBeGreaterThan(0);
    });

    it('empty-result search renders the empty-state message', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        const searchInput = container.querySelector('.help-overlay-search') as HTMLInputElement;
        fireEvent.change(searchInput, { target: { value: 'xyzzy_no_match' } });
        expect(container.querySelector('.help-overlay-empty')).not.toBeNull();
    });

    it('Esc key calls onClose', () => {
        let closed = false;
        render(<HelpOverlay open={true} onClose={() => { closed = true; }} />);
        fireEvent.keyDown(window, { key: 'Escape' });
        expect(closed).toBe(true);
    });

    it('close button calls onClose', () => {
        let closed = false;
        const { container } = render(
            <HelpOverlay open={true} onClose={() => { closed = true; }} />,
        );
        const closeButton = container.querySelector('.help-overlay-close') as HTMLButtonElement;
        expect(closeButton).not.toBeNull();
        fireEvent.click(closeButton);
        expect(closed).toBe(true);
    });

    it('null-key_path entries do not appear in rendered rows (D-26.8)', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        // Every rendered row's key cell must be non-empty (every entry has
        // a key_path because filterHelpEntries excludes null-key_path).
        const keyCells = container.querySelectorAll('.help-overlay-key');
        for (const cell of Array.from(keyCells)) {
            expect(cell.textContent?.length ?? 0).toBeGreaterThan(0);
        }
    });

    // Phase 31-04 D-31.8 / D-31.9 — two-section overlay tests

    it('renders four top-level sections with HP-41CV, Math 1 Pac, Stat 1 Pac, and Time Pac headings (D-31.8)', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        // All four section heading buttons must be present.
        const sectionButtons = container.querySelectorAll('.help-overlay-section-heading');
        const buttonTexts = Array.from(sectionButtons).map(b => b.textContent ?? '');
        expect(buttonTexts.some(t => t.includes('HP-41CV (built-in)'))).toBe(true);
        expect(buttonTexts.some(t => t.includes('Math 1 Pac (XROM 7)'))).toBe(true);
        expect(buttonTexts.some(t => t.includes('Stat 1 Pac (XROM 2)'))).toBe(true);
        expect(buttonTexts.some(t => t.includes('Time Pac (XROM 26)'))).toBe(true);
    });

    it('Math 1 Pac section contains a Math1 Hyperbolics category (D-31.9)', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        // The 2nd-level category headings within the Math 1 Pac section should include
        // "Math1 Hyperbolics". The CSS text-transform: uppercase renders it uppercase in
        // the browser, but the DOM text content retains the original casing.
        const headings = container.querySelectorAll('.help-overlay-category-heading');
        const headingTexts = Array.from(headings).map(h => h.textContent ?? '');
        expect(
            headingTexts.some(t => t.toLowerCase().includes('hyperbolics')),
            `Expected a 'Hyperbolics' category heading; found: ${headingTexts.join(', ')}`
        ).toBe(true);
    });

    it('clicking section heading toggles aria-expanded (D-31.8)', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        // Find the Math 1 Pac section heading button.
        const sectionButtons = container.querySelectorAll('.help-overlay-section-heading');
        // 7 sections: KEYBOARD SHORTCUTS (Phase 49) + HP-41CV + Math 1 + Stat 1 + Time + Adv (XROM 22) + Adv (XROM 24).
        expect(sectionButtons.length).toBe(7);

        const math1Button = Array.from(sectionButtons).find(b =>
            b.textContent?.includes('Math 1 Pac')
        ) as HTMLButtonElement | undefined;
        expect(math1Button, 'Math 1 Pac section heading button must exist').toBeTruthy();

        // Initially expanded (aria-expanded = "true").
        expect(math1Button!.getAttribute('aria-expanded')).toBe('true');

        // After click, collapsed (aria-expanded = "false").
        fireEvent.click(math1Button!);
        expect(math1Button!.getAttribute('aria-expanded')).toBe('false');

        // After second click, expanded again.
        fireEvent.click(math1Button!);
        expect(math1Button!.getAttribute('aria-expanded')).toBe('true');
    });

    // Phase 36 Plan 36-03: Stat 1 Pac section tests (STAT-GUI-03)

    it('renders three top-level sections including Stat 1 Pac (XROM 2)', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        const sectionButtons = container.querySelectorAll('.help-overlay-section-heading');
        const buttonTexts = Array.from(sectionButtons).map(b => b.textContent ?? '');
        expect(buttonTexts.some(t => t.includes('HP-41CV (built-in)'))).toBe(true);
        expect(buttonTexts.some(t => t.includes('Math 1 Pac (XROM 7)'))).toBe(true);
        expect(buttonTexts.some(t => t.includes('Stat 1 Pac (XROM 2)'))).toBe(true);
    });

    it('Stat 1 Pac section contains a Stat 1 category heading (D-34.1)', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        const headings = container.querySelectorAll('.help-overlay-category-heading');
        const headingTexts = Array.from(headings).map(h => h.textContent ?? '');
        // At least one heading should contain a known Stat 1 category substring.
        const hasStat1Category =
            headingTexts.some(t => t.toLowerCase().includes('univariate')) ||
            headingTexts.some(t => t.toLowerCase().includes('distributions')) ||
            headingTexts.some(t => t.toLowerCase().includes('anova'));
        expect(
            hasStat1Category,
            `Expected a Stat 1 category heading (univariate/distributions/anova); found: ${headingTexts.join(', ')}`
        ).toBe(true);
    });

    it('clicking Stat 1 Pac section heading toggles aria-expanded (D-31.8)', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        const sectionButtons = container.querySelectorAll('.help-overlay-section-heading');

        const stat1Button = Array.from(sectionButtons).find(b =>
            b.textContent?.includes('Stat 1 Pac')
        ) as HTMLButtonElement | undefined;
        expect(stat1Button, 'Stat 1 Pac section heading button must exist').toBeTruthy();

        // Initially expanded (aria-expanded = "true").
        expect(stat1Button!.getAttribute('aria-expanded')).toBe('true');

        // After click, collapsed (aria-expanded = "false").
        fireEvent.click(stat1Button!);
        expect(stat1Button!.getAttribute('aria-expanded')).toBe('false');

        // After second click, expanded again.
        fireEvent.click(stat1Button!);
        expect(stat1Button!.getAttribute('aria-expanded')).toBe('true');
    });

    it('search for "NORMD" returns Stat 1 entries', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        const searchInput = container.querySelector('.help-overlay-search') as HTMLInputElement;
        expect(searchInput).not.toBeNull();
        fireEvent.change(searchInput, { target: { value: 'NORMD' } });
        const rows = container.querySelectorAll('.help-overlay-row');
        expect(rows.length).toBeGreaterThan(0);
        const rowTexts = Array.from(rows).map(r => r.textContent ?? '');
        expect(
            rowTexts.some(t => t.includes('NORMD')),
            `Expected at least one row containing 'NORMD'; found rows: ${rowTexts.slice(0, 5).join(' | ')}`
        ).toBe(true);
    });

    // Phase 41 Plan 41-02: Time Pac section tests (TIME-GUI-03)

    it('renders four top-level sections including Time Pac (XROM 26)', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        const sectionButtons = container.querySelectorAll('.help-overlay-section-heading');
        const buttonTexts = Array.from(sectionButtons).map(b => b.textContent ?? '');
        expect(buttonTexts.some(t => t.includes('HP-41CV (built-in)'))).toBe(true);
        expect(buttonTexts.some(t => t.includes('Math 1 Pac (XROM 7)'))).toBe(true);
        expect(buttonTexts.some(t => t.includes('Stat 1 Pac (XROM 2)'))).toBe(true);
        expect(buttonTexts.some(t => t.includes('Time Pac (XROM 26)'))).toBe(true);
    });

    it('Time Pac section contains a Time category heading', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        const headings = container.querySelectorAll('.help-overlay-category-heading');
        const headingTexts = Array.from(headings).map(h => h.textContent ?? '');
        // At least one heading should contain a known Time Pac category substring.
        const hasTimeCategory =
            headingTexts.some(t => t.toLowerCase().includes('time clock')) ||
            headingTexts.some(t => t.toLowerCase().includes('time stopwatch')) ||
            headingTexts.some(t => t.toLowerCase().includes('time alarm')) ||
            headingTexts.some(t => t.toLowerCase().includes('time date'));
        expect(
            hasTimeCategory,
            `Expected a Time Pac category heading (time clock/stopwatch/alarm/date); found: ${headingTexts.join(', ')}`
        ).toBe(true);
    });

    it('clicking Time Pac section heading toggles aria-expanded', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        const sectionButtons = container.querySelectorAll('.help-overlay-section-heading');

        const timeButton = Array.from(sectionButtons).find(b =>
            b.textContent?.includes('Time Pac')
        ) as HTMLButtonElement | undefined;
        expect(timeButton, 'Time Pac section heading button must exist').toBeTruthy();

        // Initially expanded (aria-expanded = "true").
        expect(timeButton!.getAttribute('aria-expanded')).toBe('true');

        // After click, collapsed (aria-expanded = "false").
        fireEvent.click(timeButton!);
        expect(timeButton!.getAttribute('aria-expanded')).toBe('false');

        // After second click, expanded again.
        fireEvent.click(timeButton!);
        expect(timeButton!.getAttribute('aria-expanded')).toBe('true');
    });

    it('search for "SETIME" returns Time entries', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        const searchInput = container.querySelector('.help-overlay-search') as HTMLInputElement;
        expect(searchInput).not.toBeNull();
        fireEvent.change(searchInput, { target: { value: 'SETIME' } });
        const rows = container.querySelectorAll('.help-overlay-row');
        expect(rows.length).toBeGreaterThan(0);
        const rowTexts = Array.from(rows).map(r => r.textContent ?? '');
        expect(
            rowTexts.some(t => t.includes('SETIME')),
            `Expected at least one row containing 'SETIME'; found rows: ${rowTexts.slice(0, 5).join(' | ')}`
        ).toBe(true);
    });

    // Phase 49 D-49.10 / KBD-03 — Keyboard Shortcuts section tests

    it('renders Keyboard Shortcuts section heading when open', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        const sectionButtons = container.querySelectorAll('.help-overlay-section-heading');
        const buttonTexts = Array.from(sectionButtons).map(b => b.textContent ?? '');
        expect(
            buttonTexts.some(t => t.toUpperCase().includes('KEYBOARD SHORTCUTS')),
            `Expected "KEYBOARD SHORTCUTS" heading; found: ${buttonTexts.join(', ')}`
        ).toBe(true);
    });

    it('keyboard shortcuts section is collapsed by default', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        // Find the KEYBOARD SHORTCUTS heading button.
        const kbdBtn = Array.from(container.querySelectorAll('.help-overlay-section-heading')).find(
            b => b.textContent?.toUpperCase().includes('KEYBOARD SHORTCUTS')
        ) as HTMLButtonElement | undefined;
        expect(kbdBtn, 'KEYBOARD SHORTCUTS heading button must exist').toBeTruthy();
        // Must be collapsed by default (aria-expanded = "false").
        expect(kbdBtn!.getAttribute('aria-expanded')).toBe('false');
        // Shortcut rows should not be visible (table not rendered).
        const shortcutRows = container.querySelectorAll('.shortcut-row');
        // The header row should not be present when collapsed.
        expect(shortcutRows.length).toBe(0);
    });

    it('clicking Keyboard Shortcuts heading expands the section', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        const kbdBtn = Array.from(container.querySelectorAll('.help-overlay-section-heading')).find(
            b => b.textContent?.toUpperCase().includes('KEYBOARD SHORTCUTS')
        ) as HTMLButtonElement;
        expect(kbdBtn).toBeTruthy();
        // Initially collapsed.
        expect(kbdBtn.getAttribute('aria-expanded')).toBe('false');
        // Click to expand.
        fireEvent.click(kbdBtn);
        expect(kbdBtn.getAttribute('aria-expanded')).toBe('true');
        // Shortcut rows now visible.
        const shortcutRows = container.querySelectorAll('.shortcut-row');
        expect(shortcutRows.length).toBeGreaterThan(0);
    });

    it('shortcut entries show key and op columns when expanded', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        const kbdBtn = Array.from(container.querySelectorAll('.help-overlay-section-heading')).find(
            b => b.textContent?.toUpperCase().includes('KEYBOARD SHORTCUTS')
        ) as HTMLButtonElement;
        fireEvent.click(kbdBtn);
        // Verify at least one shortcut-key-col and shortcut-fn-col exist and have content.
        const keyCols = container.querySelectorAll('.shortcut-key-col');
        const fnCols = container.querySelectorAll('.shortcut-fn-col');
        expect(keyCols.length).toBeGreaterThan(0);
        expect(fnCols.length).toBeGreaterThan(0);
        // At least one key col should have non-empty content (e.g. "Enter").
        const keyTexts = Array.from(keyCols).map(c => c.textContent ?? '');
        expect(keyTexts.some(t => t.trim().length > 0)).toBe(true);
    });

    // Phase 49 D-49.6 / ONBOARD-03 — Expandable entry tests

    it('entries with example field show expand toggle button', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        // The hp41cv-functions.json has 30 enriched entries (Plan 02 output).
        // The HP-41CV section is expanded by default, so expand toggles should be visible.
        const expandBtns = container.querySelectorAll('.help-entry-expand-btn');
        expect(
            expandBtns.length,
            'Expected at least one expand toggle button for entries with example/notes'
        ).toBeGreaterThan(0);
    });

    it('clicking expand toggle reveals example text', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        // Find the first expand toggle.
        const expandBtn = container.querySelector('.help-entry-expand-btn') as HTMLButtonElement | null;
        expect(expandBtn, 'Expand button must exist').not.toBeNull();
        // Initially collapsed — no .help-entry-detail visible.
        expect(container.querySelector('.help-entry-detail')).toBeNull();
        // Click to expand.
        fireEvent.click(expandBtn!);
        // Detail block now visible.
        const detail = container.querySelector('.help-entry-detail');
        expect(detail, '.help-entry-detail must be visible after clicking expand toggle').not.toBeNull();
        // At least example or notes span must be present.
        const exampleSpan = detail?.querySelector('.help-entry-example');
        const notesSpan = detail?.querySelector('.help-entry-notes');
        expect(
            exampleSpan || notesSpan,
            '.help-entry-example or .help-entry-notes must be present in detail block'
        ).toBeTruthy();
    });

    it('entries without example/notes have no expand toggle', () => {
        const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
        // All .help-overlay-row entries should be checked: rows without a toggle button
        // should exist (not every entry is enriched).
        const rows = container.querySelectorAll('.help-overlay-row');
        const rowsWithoutToggle = Array.from(rows).filter(
            row => !row.querySelector('.help-entry-expand-btn')
        );
        expect(
            rowsWithoutToggle.length,
            'Expected at least one row without an expand toggle (non-enriched entry)'
        ).toBeGreaterThan(0);
    });
});
