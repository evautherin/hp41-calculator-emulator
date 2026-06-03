// Phase 26 Plan 03 D-26.8 — `?` help overlay.
//
// Full-cover semi-transparent React modal listing every HP-41CV function
// from `docs/hp41cv-functions.json` (via help_data.ts). Search input
// filters across display_name + description + category. Categories from
// the JSON declaration order become section headings. Entries with
// `key_path === null` are excluded from the Keyboard Shortcuts tab per D-26.8.
//
// Parent App.tsx owns the open/close state and the `?`-keystroke handler.
// The overlay's own Esc handler is defense-in-depth: App.tsx handles Esc
// precedence (help → modal → shift), and this component also closes on
// Esc when mounted.
//
// Phase 31-04 D-31.8 / D-31.9: Two top-level collapsible sections:
//   1. "HP-41CV (built-in)" — entries without `xrom` field
//   2. "Math 1 Pac (XROM 7)" — entries with `xrom.module === "Math 1"`
// Both sections expanded by default. Each section collapses independently.
// Within each section, JSON's per-program categories render as 2nd-level
// headers (existing .help-overlay-category-heading pattern). Entries sorted
// alphabetically within each category.
//
// Phase 41 Plan 41-02 D-carried.8 (TIME-GUI-03): Fourth top-level section added:
//   4. "Time Pac (XROM 26)" — entries with `xrom.module === "Time"`
// Section added to the sectionGroups.map() render loop automatically.
//
// Phase 46 Plan 46-02 (ADV-GUI-02): Fifth and sixth top-level sections added:
//   5. "Advantage Pac (XROM 22)" — entries with `xrom.module === "Adv Conv"` (63 entries)
//   6. "Advantage Pac (XROM 24)" — entries with `xrom.module === "Adv Math"` (51 entries)
// CRITICAL: predicate values are "Adv Conv" / "Adv Math" — matching
// docs/hp41-advantage-functions.json xrom.module fields exactly (NOT "ADV 22A" / "ADV 24B").
// Search spans all six JSON pools via helpEntriesAll() 6-pool chain in help_data.ts
// (cv + math1 + stat1 + time + advantage + xmem; X-MEM pool added in Phase 52).
//
// Phase lu0 D-lu0-02: Two-tab overlay.
//   Tab "Keyboard Shortcuts" — current behavior (key_path-filtered sections + shortcuts).
//   Tab "All Functions"      — every implemented entry (incl. key_path:null built-ins),
//                              grouped Module→Category. Runnable rows are tap-to-run
//                              buttons; non-tappable rows are listed but non-interactive.
//   iOS-aware default: All Functions on iOS, Keyboard Shortcuts on Desktop.
//   onRun prop: dispatches `xeq_<token>` and closes the overlay (App.tsx responsibility).

import { useState, useEffect, useMemo } from 'react';
import { helpEntriesAll, allFunctionsEntries, xeqToken, getKeyboardShortcuts, type HelpEntry } from './help_data';

export type HelpOverlayProps = {
    open: boolean;
    onClose: () => void;
    /// Phase lu0: iOS flag — gates the default active tab (All Functions on iOS,
    /// Keyboard Shortcuts on Desktop). Passed from App.tsx's `isIos` state.
    isIos?: boolean;
    /// Phase lu0: Tap-to-run callback. Called with the fully-formed `xeq_<token>`
    /// key id when a tappable All Functions row is activated. App.tsx is responsible
    /// for dispatching and closing the overlay (setHelpOpen(false) first so the
    /// result is immediately visible on the display).
    onRun?: (keyId: string) => void;
};

/// Section descriptor for the six top-level overlay sections.
/// `predicate` selects which entries belong to this section.
///
/// Phase 36 Plan 36-02: widened id union from 'hp41cv' | 'math1' to include 'stat1'
/// (third section for Stat 1 Pac XROM 2 entries per STAT-GUI-03).
/// Phase 41 Plan 41-02: widened id union to include 'time'
/// (fourth section for Time Pac XROM 26 entries per TIME-GUI-03).
/// Phase 46 Plan 46-02: widened id union to include 'adv22' and 'adv24'
/// (fifth + sixth sections for Advantage Pac XROM 22 + XROM 24 per ADV-GUI-02).
interface SectionDef {
    id: 'hp41cv' | 'math1' | 'stat1' | 'time' | 'adv22' | 'adv24';
    heading: string;
    predicate: (e: HelpEntry) => boolean;
}

/// Six top-level sections per D-31.8 (extended Phase 46 Plan 46-02).
/// Order: built-in first, Math 1 Pac second, Stat 1 Pac third, Time Pac fourth,
/// Advantage Pac XROM 22 fifth, Advantage Pac XROM 24 sixth.
const SECTIONS: SectionDef[] = [
    {
        id: 'hp41cv',
        heading: 'HP-41CV (built-in)',
        predicate: (e: HelpEntry) => !e.xrom,
    },
    {
        id: 'math1',
        heading: 'Math 1 Pac (XROM 7)',
        predicate: (e: HelpEntry) => e.xrom?.module === 'Math 1',
    },
    {
        id: 'stat1',
        heading: 'Stat 1 Pac (XROM 2)',
        predicate: (e: HelpEntry) => e.xrom?.module === 'Stat 1',
    },
    {
        id: 'time',
        heading: 'Time Pac (XROM 26)',
        // CRITICAL: xrom.module value is "Time" — NOT "TIME", "Time Pac", or "TIME 2C"
        // (per docs/hp41-time-functions.json xrom.module field / D-39.9 / Pitfall 6).
        predicate: (e: HelpEntry) => e.xrom?.module === 'Time',
    },
    {
        id: 'adv22',
        heading: 'Advantage Pac (XROM 22)',
        // CRITICAL: xrom.module value is "Adv Conv" — NOT "ADV 22A", "Advantage Conv", or
        // "Adv Conv/Mtrx" (per docs/hp41-advantage-functions.json xrom.module field / D-44.1).
        // Covers ADV CONV + ADV MTRX families (63 entries, module_id: 22).
        predicate: (e: HelpEntry) => e.xrom?.module === 'Adv Conv',
    },
    {
        id: 'adv24',
        heading: 'Advantage Pac (XROM 24)',
        // CRITICAL: xrom.module value is "Adv Math" — NOT "ADV 24B", "Advantage Math", or
        // "Adv Math/TVM" (per docs/hp41-advantage-functions.json xrom.module field / D-44.1).
        // Covers ADV MATH + ADV TVM families (51 entries, module_id: 24).
        predicate: (e: HelpEntry) => e.xrom?.module === 'Adv Math',
    },
];

/// Phase lu0 D-lu0-02: All Functions tab module sections.
/// Locked order (plan spec): HP-41CV Built-in → Math Pac → Stat Pac → Time Pac →
/// Advantage Pac (SINGLE header combining Adv Conv + Adv Math) → Extended Memory.
///
/// NOTE: Advantage is a SINGLE header here (unlike the Keyboard Shortcuts tab
/// which has two separate XROM 22 / XROM 24 sections). Extended Memory entries
/// have no xrom field AND category === "Extended Memory".
interface AllFnSectionDef {
    id: string;
    heading: string;
    predicate: (e: HelpEntry) => boolean;
}

const ALL_FN_SECTIONS: AllFnSectionDef[] = [
    {
        id: 'hp41cv',
        heading: 'HP-41CV Built-in',
        predicate: (e: HelpEntry) => !e.xrom && e.category !== 'Extended Memory',
    },
    {
        id: 'math1',
        heading: 'Math Pac (MATH 1)',
        predicate: (e: HelpEntry) => e.xrom?.module === 'Math 1',
    },
    {
        id: 'stat1',
        heading: 'Stat Pac (STAT 1)',
        predicate: (e: HelpEntry) => e.xrom?.module === 'Stat 1',
    },
    {
        id: 'time',
        heading: 'Time Pac (TIME 2C)',
        predicate: (e: HelpEntry) => e.xrom?.module === 'Time',
    },
    {
        id: 'advantage',
        heading: 'Advantage Pac',
        // SINGLE header — combines Adv Conv (XROM 22) and Adv Math (XROM 24)
        predicate: (e: HelpEntry) => e.xrom?.module === 'Adv Conv' || e.xrom?.module === 'Adv Math',
    },
    {
        id: 'xmem',
        heading: 'Extended Memory',
        // X-MEM entries: no xrom field (OS built-in, D-52.4) AND category === 'Extended Memory'
        predicate: (e: HelpEntry) => e.xrom === undefined && e.category === 'Extended Memory',
    },
];

export function HelpOverlay({ open, onClose, isIos = false, onRun }: HelpOverlayProps) {
    const [query, setQuery] = useState('');

    // Phase lu0 D-lu0-02: Two-tab overlay. Default tab is iOS-aware.
    type TabId = 'shortcuts' | 'all';
    const defaultTab: TabId = isIos ? 'all' : 'shortcuts';
    const [activeTab, setActiveTab] = useState<TabId>(defaultTab);

    // D-31.8: All sections expanded by default; state resets on each overlay open.
    // Phase 36 Plan 36-02: widened from {hp41cv, math1} to {hp41cv, math1, stat1}.
    // Phase 41 Plan 41-02: widened from {hp41cv, math1, stat1} to include time.
    // Phase 46 Plan 46-02: widened to include adv22 and adv24 (Advantage Pac XROM 22 + XROM 24).
    const [expanded, setExpanded] = useState<{ hp41cv: boolean; math1: boolean; stat1: boolean; time: boolean; adv22: boolean; adv24: boolean }>({
        hp41cv: true,
        math1: true,
        stat1: true,
        time: true,
        adv22: true,
        adv24: true,
    });

    // All Functions tab: per-section expand state (all expanded by default).
    const [allFnExpanded, setAllFnExpanded] = useState<Record<string, boolean>>(
        Object.fromEntries(ALL_FN_SECTIONS.map(s => [s.id, true]))
    );

    // Phase 49 D-49.10 / KBD-03: Keyboard Shortcuts section — standalone state (D-49.10 / PATTERNS.md divergence).
    // Collapsed by default per UI-SPEC (shortcuts are for power users; function sections lead).
    // NOT part of the SECTIONS array — different content shape and not searchable.
    const [kbdExpanded, setKbdExpanded] = useState(false);

    // Phase 49 D-49.6 / ONBOARD-03: Per-entry expand state for example/notes detail.
    // Tracks op_variant keys of expanded entries. Resets on overlay close/open.
    const [expandedEntries, setExpandedEntries] = useState<Set<string>>(new Set());

    // Reset query, tab, and expand state whenever overlay opens (clean-slate UX).
    // Tab defaults to iOS-aware value on each open.
    useEffect(() => {
        if (open) {
            setQuery('');
            setActiveTab(isIos ? 'all' : 'shortcuts');
            setExpanded({ hp41cv: true, math1: true, stat1: true, time: true, adv22: true, adv24: true });
            setAllFnExpanded(Object.fromEntries(ALL_FN_SECTIONS.map(s => [s.id, true])));
            setKbdExpanded(false);
            setExpandedEntries(new Set());
        }
    }, [open, isIos]);

    // Keyboard Shortcuts tab: key_path-filtered entries (D-26.8 unchanged).
    const allEntries = useMemo(() =>
        helpEntriesAll().filter(e => e.key_path !== null),
    []);

    // All Functions tab: all implemented entries (no key_path filter).
    const allFnEntries = useMemo(() => allFunctionsEntries(), []);

    // Filter entries by search query — applied within the active tab.
    const filtered = useMemo(() => {
        const q = query.toLowerCase().trim();
        if (q === '') return allEntries;
        return allEntries.filter(e =>
            e.display_name.toLowerCase().includes(q) ||
            e.description.toLowerCase().includes(q) ||
            e.category.toLowerCase().includes(q)
        );
    }, [query, allEntries]);

    // All Functions tab: filter by query.
    const filteredAllFn = useMemo(() => {
        const q = query.toLowerCase().trim();
        if (q === '') return allFnEntries;
        return allFnEntries.filter(e =>
            e.display_name.toLowerCase().includes(q) ||
            e.description.toLowerCase().includes(q) ||
            e.category.toLowerCase().includes(q)
        );
    }, [query, allFnEntries]);

    // Group Keyboard Shortcuts entries by section → category. Sort alphabetically within each category.
    const sectionGroups = useMemo(() => {
        return SECTIONS.map(section => {
            const sectionEntries = filtered.filter(section.predicate);
            // Group by category (preserve insertion order for category discovery).
            const catMap = new Map<string, HelpEntry[]>();
            for (const entry of sectionEntries) {
                const arr = catMap.get(entry.category);
                if (arr) {
                    arr.push(entry);
                } else {
                    catMap.set(entry.category, [entry]);
                }
            }
            // Sort entries alphabetically within each category.
            for (const arr of catMap.values()) {
                arr.sort((a, b) => a.display_name.localeCompare(b.display_name));
            }
            return {
                section,
                groups: Array.from(catMap.entries()), // [[category, entries], ...]
                count: sectionEntries.length,
            };
        });
    }, [filtered]);

    // Group All Functions entries by module section → category. Sort alphabetically within each category.
    const allFnSectionGroups = useMemo(() => {
        return ALL_FN_SECTIONS.map(section => {
            const sectionEntries = filteredAllFn.filter(section.predicate);
            const catMap = new Map<string, HelpEntry[]>();
            for (const entry of sectionEntries) {
                const arr = catMap.get(entry.category);
                if (arr) {
                    arr.push(entry);
                } else {
                    catMap.set(entry.category, [entry]);
                }
            }
            for (const arr of catMap.values()) {
                arr.sort((a, b) => a.display_name.localeCompare(b.display_name));
            }
            return {
                section,
                groups: Array.from(catMap.entries()),
                count: sectionEntries.length,
            };
        });
    }, [filteredAllFn]);

    // Esc-close: defense-in-depth. App.tsx::handleKey already handles Esc
    // precedence (help → modal → shift); this listener ensures the overlay
    // closes even if mounted in a context where the parent listener is
    // unavailable (e.g. tests rendering the component standalone).
    useEffect(() => {
        if (!open) return;
        const onKey = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                e.preventDefault();
                onClose();
            }
        };
        window.addEventListener('keydown', onKey);
        return () => window.removeEventListener('keydown', onKey);
    }, [open, onClose]);

    if (!open) return null;

    const toggleSection = (id: 'hp41cv' | 'math1' | 'stat1' | 'time' | 'adv22' | 'adv24') => {
        setExpanded(prev => ({ ...prev, [id]: !prev[id] }));
    };

    const toggleAllFnSection = (id: string) => {
        setAllFnExpanded(prev => ({ ...prev, [id]: !prev[id] }));
    };

    // Phase 49 D-49.6: Toggle individual entry expand/collapse state.
    const toggleEntry = (opVariant: string) => {
        setExpandedEntries(prev => {
            const next = new Set(prev);
            if (next.has(opVariant)) {
                next.delete(opVariant);
            } else {
                next.add(opVariant);
            }
            return next;
        });
    };

    const totalFiltered = activeTab === 'all' ? filteredAllFn.length : filtered.length;
    const shortcuts = getKeyboardShortcuts();

    return (
        <div className="help-overlay" role="dialog" aria-label="HP-41 function reference">
            <div className="help-overlay-header">
                {/* Row 1: search + close (so the search field gets full width on iPhone) */}
                <div className="help-overlay-header-top">
                    <input
                        className="help-overlay-search"
                        type="text"
                        value={query}
                        onChange={e => setQuery(e.target.value)}
                        placeholder={activeTab === 'all' ? 'Search all functions...' : 'Search functions...'}
                        autoFocus
                        aria-label="Search HP-41 functions"
                    />
                    <button
                        className="help-overlay-close"
                        onClick={onClose}
                        aria-label="Close help overlay"
                    >
                        ×
                    </button>
                </div>
                {/* Row 2: tab bar (Phase lu0 D-lu0-02) — segmented control below the search */}
                <div className="help-overlay-tabs" role="tablist">
                    <button
                        className={`help-overlay-tab${activeTab === 'shortcuts' ? ' help-overlay-tab--active' : ''}`}
                        role="tab"
                        aria-selected={activeTab === 'shortcuts' ? 'true' : 'false'}
                        onClick={() => setActiveTab('shortcuts')}
                    >
                        Keyboard Shortcuts
                    </button>
                    <button
                        className={`help-overlay-tab${activeTab === 'all' ? ' help-overlay-tab--active' : ''}`}
                        role="tab"
                        aria-selected={activeTab === 'all' ? 'true' : 'false'}
                        onClick={() => setActiveTab('all')}
                    >
                        All Functions
                    </button>
                </div>
            </div>
            <div className="help-overlay-content">
                {totalFiltered === 0 && query !== '' && (
                    <div className="help-overlay-empty">No functions match "{query}".</div>
                )}

                {/* ── Tab: Keyboard Shortcuts ── */}
                {activeTab === 'shortcuts' && (
                    <>
                        {/* Phase 49 D-49.10 / KBD-03: Keyboard Shortcuts section.
                            Rendered BEFORE the function sections, collapsed by default.
                            NOT filtered by search query — physical key mappings have different
                            semantics from function lookup (D-49.10 / UI-SPEC §KBD). */}
                        <div className="help-overlay-section">
                            <button
                                className="help-overlay-section-heading"
                                onClick={() => setKbdExpanded(prev => !prev)}
                                aria-expanded={kbdExpanded ? "true" : "false"}
                            >
                                KEYBOARD SHORTCUTS
                            </button>
                            {kbdExpanded && (
                                <div className="help-overlay-section-body">
                                    {shortcuts.length === 0 ? (
                                        <div className="help-overlay-empty">No shortcuts listed.</div>
                                    ) : (
                                        <div className="shortcut-table" role="table">
                                            <div role="row" className="shortcut-row">
                                                <span role="columnheader" className="shortcut-key-col"><strong>Key</strong></span>
                                                <span role="columnheader" className="shortcut-fn-col"><strong>Function</strong></span>
                                            </div>
                                            {shortcuts.map((sc, idx) => (
                                                <div key={idx} role="row" className="shortcut-row">
                                                    <span role="cell" className="shortcut-key-col">{sc.key}</span>
                                                    <span role="cell" className="shortcut-fn-col">{sc.op} — {sc.description}</span>
                                                </div>
                                            ))}
                                        </div>
                                    )}
                                </div>
                            )}
                        </div>
                        {sectionGroups.map(({ section, groups, count }) => (
                            <div key={section.id} className="help-overlay-section">
                                {/* Top-level collapsible section heading (D-31.8 / UI-SPEC §Accessibility) */}
                                <button
                                    className="help-overlay-section-heading"
                                    onClick={() => toggleSection(section.id)}
                                    aria-expanded={expanded[section.id] ? "true" : "false"}
                                >
                                    {section.heading}
                                    {query !== '' && ` (${count})`}
                                </button>
                                {/* Section body — only rendered when expanded */}
                                {expanded[section.id] && (
                                    <div className="help-overlay-section-body">
                                        {groups.map(([category, entries]) => (
                                            <div key={category} className="help-overlay-category">
                                                <h3 className="help-overlay-category-heading">{category}</h3>
                                                {entries.map(entry => {
                                                    const hasDetail = !!(entry.example || entry.notes);
                                                    const isEntryExpanded = expandedEntries.has(entry.op_variant);
                                                    return (
                                                        <div key={entry.op_variant}>
                                                            <div className="help-overlay-row">
                                                                {hasDetail ? (
                                                                    <button
                                                                        className="help-entry-expand-btn"
                                                                        onClick={() => toggleEntry(entry.op_variant)}
                                                                        aria-expanded={isEntryExpanded ? "true" : "false"}
                                                                        aria-label={`Show example for ${entry.display_name}`}
                                                                    >
                                                                        {isEntryExpanded ? '▼' : '▶'}
                                                                    </button>
                                                                ) : null}
                                                                <span className="help-overlay-key">{entry.key_path}</span>
                                                                <span className="help-overlay-op">{entry.display_name}</span>
                                                                <span className="help-overlay-desc">{entry.description}</span>
                                                            </div>
                                                            {hasDetail && isEntryExpanded && (
                                                                <div className="help-entry-detail">
                                                                    {entry.example && (
                                                                        <span className="help-entry-example">{entry.example}</span>
                                                                    )}
                                                                    {entry.notes && (
                                                                        <span className="help-entry-notes">{entry.notes}</span>
                                                                    )}
                                                                </div>
                                                            )}
                                                        </div>
                                                    );
                                                })}
                                            </div>
                                        ))}
                                        {groups.length === 0 && query !== '' && (
                                            <div className="help-overlay-empty">No matches in this section.</div>
                                        )}
                                    </div>
                                )}
                            </div>
                        ))}
                    </>
                )}

                {/* ── Tab: All Functions ── */}
                {activeTab === 'all' && (
                    <>
                        {allFnSectionGroups.map(({ section, groups, count }) => (
                            <div key={section.id} className="help-overlay-section">
                                <button
                                    className="help-overlay-section-heading"
                                    onClick={() => toggleAllFnSection(section.id)}
                                    aria-expanded={allFnExpanded[section.id] ? "true" : "false"}
                                    data-section-id={section.id}
                                >
                                    {section.heading}
                                    {query !== '' && ` (${count})`}
                                </button>
                                {allFnExpanded[section.id] && (
                                    <div className="help-overlay-section-body">
                                        {groups.map(([category, entries]) => (
                                            <div key={category} className="help-overlay-category">
                                                <h3 className="help-overlay-category-heading">{category}</h3>
                                                {entries.map(entry => {
                                                    const token = xeqToken(entry);
                                                    const hasDetail = !!(entry.example || entry.notes);
                                                    const isEntryExpanded = expandedEntries.has(entry.op_variant);
                                                    return (
                                                        <div key={entry.op_variant}>
                                                            {token !== null ? (
                                                                /* Tappable row: runnable by XEQ-by-name.
                                                                   Dispatches `xeq_<token>` via onRun prop.
                                                                   SAME id works in normal mode (executes) and
                                                                   PRGM mode (inserts as program step) — backend
                                                                   PRGM gate splits behavior, no frontend branch. */
                                                                <div className="help-overlay-row help-overlay-row--tappable">
                                                                    {hasDetail ? (
                                                                        <button
                                                                            className="help-entry-expand-btn"
                                                                            onClick={() => toggleEntry(entry.op_variant)}
                                                                            aria-expanded={isEntryExpanded ? "true" : "false"}
                                                                            aria-label={`Show example for ${entry.display_name}`}
                                                                        >
                                                                            {isEntryExpanded ? '▼' : '▶'}
                                                                        </button>
                                                                    ) : null}
                                                                    <button
                                                                        className="help-fn-run-btn"
                                                                        onClick={() => onRun?.(`xeq_${token}`)}
                                                                        aria-label={`Run ${entry.display_name}`}
                                                                    >
                                                                        <span className="help-overlay-op">{entry.display_name}</span>
                                                                        <span className="help-overlay-desc">{entry.description}</span>
                                                                    </button>
                                                                </div>
                                                            ) : (
                                                                /* Non-tappable row: parameterized / key-only / composite.
                                                                   Listed for discovery but NOT a button — D-07: never
                                                                   dispatch an unresolvable id. */
                                                                <div
                                                                    className="help-overlay-row help-overlay-row--non-tappable"
                                                                    aria-disabled="true"
                                                                >
                                                                    {hasDetail ? (
                                                                        <button
                                                                            className="help-entry-expand-btn"
                                                                            onClick={() => toggleEntry(entry.op_variant)}
                                                                            aria-expanded={isEntryExpanded ? "true" : "false"}
                                                                            aria-label={`Show example for ${entry.display_name}`}
                                                                        >
                                                                            {isEntryExpanded ? '▼' : '▶'}
                                                                        </button>
                                                                    ) : null}
                                                                    <span className="help-overlay-op help-overlay-op--keyboard-only">{entry.display_name}</span>
                                                                    <span className="help-overlay-desc">{entry.description}</span>
                                                                    <span className="help-overlay-kbd-hint" aria-label="keyboard only">⌨</span>
                                                                </div>
                                                            )}
                                                            {hasDetail && isEntryExpanded && (
                                                                <div className="help-entry-detail">
                                                                    {entry.example && (
                                                                        <span className="help-entry-example">{entry.example}</span>
                                                                    )}
                                                                    {entry.notes && (
                                                                        <span className="help-entry-notes">{entry.notes}</span>
                                                                    )}
                                                                </div>
                                                            )}
                                                        </div>
                                                    );
                                                })}
                                            </div>
                                        ))}
                                        {groups.length === 0 && query !== '' && (
                                            <div className="help-overlay-empty">No matches in this section.</div>
                                        )}
                                    </div>
                                )}
                            </div>
                        ))}
                    </>
                )}
            </div>
        </div>
    );
}

export default HelpOverlay;
