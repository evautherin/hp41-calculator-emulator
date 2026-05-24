# Phase 36: hp41-gui — GUI Integration - Pattern Map

**Mapped:** 2026-05-24
**Files analyzed:** 8 new/modified files
**Analogs found:** 8 / 8

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `hp41-gui/src-tauri/src/prgm_display.rs` (modify) | utility | transform | `hp41-cli/src/prgm_display.rs` (Phase 34 Stat 1 arms) | exact |
| `hp41-core/src/ops/program.rs` (modify — `op_catalog`) | utility | CRUD | `hp41-core/src/ops/program.rs` (existing bit-0 block, lines 343–354) | exact (self-template) |
| `hp41-core/tests/op_catalog_xrom.rs` (modify — new test fn) | test | CRUD | `hp41-core/tests/op_catalog_xrom.rs` (existing `catalog_2_with_math1_loaded_*`) | exact |
| `hp41-gui/src/help_data.ts` (modify) | utility | transform | `hp41-gui/src/help_data.ts` (existing `helpEntriesMath1()` block, lines 137–154) | exact (self-template) |
| `hp41-gui/src/HelpOverlay.tsx` (modify) | component | event-driven | `hp41-gui/src/HelpOverlay.tsx` (existing `'math1'` SECTIONS entry + `expanded` state, lines 32–132) | exact (self-template) |
| `hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt_stat1.rs` (new) | test | request-response | `hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt.rs` | exact |
| `hp41-gui/src/HelpOverlay.test.tsx` (modify) | test | event-driven | `hp41-gui/src/HelpOverlay.test.tsx` (existing Phase 31-04 section tests, lines 198–244) | exact |
| `.planning/REQUIREMENTS.md` + `.planning/ROADMAP.md` (modify) | config | — | Phase 33 D-32.7 reassignment pattern (STAT-QUAL-09) | role-match |

---

## Pattern Assignments

### `hp41-gui/src-tauri/src/prgm_display.rs` (utility, transform)

**Plan:** 36-01
**Analog:** `hp41-cli/src/prgm_display.rs` lines 295–327 (Phase 34 Stat 1 arms)

**File header comment** (lines 43–46 of GUI file, currently stale):
```rust
// Currently reads: "Covers all 35 Op variants exhaustively"
// Must update count to post-Phase-36 total (35 + 26 = 61 total Op variants
// that compile through this exhaustive match). Planner verifies the exact count
// via: grep -c "Op::" hp41-gui/src-tauri/src/prgm_display.rs (post-edit).
```

**Exact 26 arms to append** (copy verbatim from `hp41-cli/src/prgm_display.rs` lines 295–327):
```rust
// ── Phase 33: Stat 1 Pac Univariate / Bivariate (Plan 33-05/06) ────────────
Op::SigmaBstat => "\u{03A3}BSTAT".to_string(),
Op::SigmaBstg  => "\u{03A3}BSTG".to_string(),
Op::SigmaMmtug => "\u{03A3}MMTUG".to_string(),
Op::SigmaMmtgd => "\u{03A3}MMTGD".to_string(),
// ── Phase 33: Stat 1 Pac ANOVA Family (Plan 33-06) ─────────────────────────
Op::SigmaAovone => "\u{03A3}AOVONE".to_string(),
Op::SigmaAovtwo => "\u{03A3}AOVTWO".to_string(),
Op::SigmaAnocov => "\u{03A3}ANOCOV".to_string(),
// ── Phase 33: Stat 1 Pac Curve Fitting + Regression (Plan 33-05/08) ────────
Op::SigmaLin    => "\u{03A3}LIN".to_string(),
Op::SigmaExp    => "\u{03A3}EXP".to_string(),
Op::SigmaLogi   => "\u{03A3}LOGI".to_string(),
Op::SigmaPow    => "\u{03A3}POW".to_string(),
Op::SigmaMlrxy  => "\u{03A3}MLRXY".to_string(),
Op::SigmaMlrxyz => "\u{03A3}MLRXYZ".to_string(),
Op::SigmaPolypWorkflow => "\u{03A3}POLYP".to_string(),
Op::SigmaPolyc  => "\u{03A3}POLYC".to_string(),
// ── Phase 33: Stat 1 Pac Hypothesis Tests (Plan 33-07) ─────────────────────
Op::SigmaPtst  => "\u{03A3}PTST".to_string(),
Op::SigmaTstat => "\u{03A3}TSTAT".to_string(),
// ── Phase 33: Stat 1 Pac Nonparam / Chi-Sq Eval / Contingency (Plan 33-04/06) ─
Op::SigmaXsqev => "\u{03A3}XSQEV".to_string(),
Op::SigmaEfxsq => "\u{03A3}EFXSQ".to_string(),
Op::SigmaCtkkk => "\u{03A3}CTKKK".to_string(),
Op::SigmaCtkk  => "\u{03A3}CTKK".to_string(),
Op::SigmaSpear => "\u{03A3}SPEAR".to_string(),
// ── Phase 33: Stat 1 Pac Distributions (Plan 33-03) ────────────────────────
Op::SigmaNormdWorkflow  => "\u{03A3}NORMD".to_string(),
Op::SigmaChisqdWorkflow => "\u{03A3}CHISQD".to_string(),
// ── Phase 33: Stat 1 Pac RNG (Plan 33-08) ──────────────────────────────────
Op::Rand => "RAND".to_string(),
Op::Seed => "SEED".to_string(),
```

**Placement:** Insert these 26 arms before the closing `}` of the `match op {` in `op_display_name`, after the last existing arm (`Op::Trans3d => "T3D".to_string(),` at line 312 of the GUI file).

**No-catch-all invariant** (verified in `prgm_display_math1_arms.rs` by file-text-scan): the `fn op_display_name` body must contain neither `_ =>` nor `_=>`. After adding the 26 arms the Rust compiler's `non-exhaustive patterns` warning clears and `just gui-ci` exits 0.

**SC-4 safety check** (after edit, must return empty):
```bash
grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" \
    hp41-gui/src-tauri/src/
```

---

### `hp41-core/src/ops/program.rs` — `op_catalog` bit-1 extension (utility, CRUD)

**Plan:** 36-01
**Analog:** `hp41-core/src/ops/program.rs` lines 343–354 (the existing MATH_1 bit-0 block — self-template)

**Current bit-0 block** (lines 343–354, the direct template):
```rust
if state.xrom_modules & 0b0000_0001 != 0 {
    // Math Pac I (bit 0) is loaded.
    state.print_buffer.push(format!(
        "{:<24}",
        format!("XROM {} {}", MATH_1.id, MATH_1.name)
    ));
    for (name, _op) in MATH_1.ops {
        state.print_buffer.push(format!("{name:<24}"));
    }
} else {
    state.print_buffer.push(format!("{:<24}", "NO XROM"));
}
```

**New bit-1 block** (insert between the `}` of the MATH_1 `if` block and the `else { "NO XROM" }` fallback — converting `if/else` to two parallel `if` blocks + `else`):
```rust
if state.xrom_modules & 0b0000_0001 != 0 {
    // Math Pac I (bit 0) is loaded.
    state.print_buffer.push(format!(
        "{:<24}",
        format!("XROM {} {}", MATH_1.id, MATH_1.name)
    ));
    for (name, _op) in MATH_1.ops {
        state.print_buffer.push(format!("{name:<24}"));
    }
}
// Phase 36 (Plan 36-01 / D-36.1): Stat 1 Pac bit-1 block — mirrors bit-0
// block exactly. Two parallel `if` blocks (NOT `if/else if`) so that both
// pacs list when both bits are set (xrom_modules = 0b0000_0011).
if state.xrom_modules & 0b0000_0010 != 0 {
    // Stat 1 Pac (bit 1) is loaded.
    state.print_buffer.push(format!(
        "{:<24}",
        format!("XROM {} {}", STAT_1.id, STAT_1.name)
    ));
    for (name, _op) in STAT_1.ops {
        state.print_buffer.push(format!("{name:<24}"));
    }
} else if state.xrom_modules & 0b0000_0001 == 0 {
    // Both bits clear — currently impossible post-migrate_after_load; defensive.
    state.print_buffer.push(format!("{:<24}", "NO XROM"));
}
```

**Import change** (line 18, currently `use crate::ops::math1::xrom::MATH_1;`):
```rust
use crate::ops::math1::xrom::{MATH_1, STAT_1};
```

**Critical structural note (Pitfall 3 from RESEARCH.md):** The `else { "NO XROM" }` that currently trails the MATH_1 block must become conditional on BOTH bits being clear. The simplest correct restructuring: remove the bare `else`, add the STAT_1 `if` block as a sibling, and add the "NO XROM" as `else if xrom_modules & 0b0000_0001 == 0` (fires only when bit-0 is also unset). Planner verifies by reading the actual lines 343–355 at write time; the exact restructuring may differ slightly from above if the current code shape is different.

---

### `hp41-core/tests/op_catalog_xrom.rs` — new test function (test, CRUD)

**Plan:** 36-01
**Analog:** `hp41-core/tests/op_catalog_xrom.rs` lines 15–81 (`catalog_2_with_math1_loaded_lists_header_and_functions`)

**Imports pattern** (lines 11–13, reuse verbatim + add STAT_1):
```rust
use hp41_core::ops::math1::xrom::{MATH_1, STAT_1};
use hp41_core::ops::program::op_catalog;
use hp41_core::state::CalcState;
```

**New test function** (append after existing `catalog_3_and_4_still_not_available`):
```rust
/// CAT 2 with both Math 1 (bit 0) AND Stat 1 (bit 1) loaded must emit BOTH
/// module headers and all their function entries.
///
/// Phase 36 Plan 36-01 / D-36.1: verifies the new parallel bit-1 conditional block.
#[test]
fn catalog_2_lists_stat1_when_bit1_set() {
    let mut state = CalcState {
        xrom_modules: 0b0000_0011,
        ..CalcState::default()
    };

    let initial_buf_len = state.print_buffer.len();
    op_catalog(&mut state, 2).unwrap();

    let new_lines = &state.print_buffer[initial_buf_len..];
    let buffer = new_lines.join("\n");

    // MATH_1 header must appear: "XROM 7 MATH 1A"
    assert!(
        buffer.contains("MATH 1A"),
        "CATALOG 2 must list MATH 1A (bit 0); got:\n{buffer}"
    );
    // STAT_1 header must appear: "XROM 2 STAT 1B"
    assert!(
        buffer.contains("STAT 1B"),
        "CATALOG 2 must list STAT 1B (bit 1); got:\n{buffer}"
    );
    // Spot-check one entry from each pac
    assert!(
        buffer.contains("Y^X"),
        "CATALOG 2 must contain Math 1 entry Y^X; got:\n{buffer}"
    );
    assert!(
        buffer.contains("\u{03A3}BSTAT"),
        "CATALOG 2 must contain Stat 1 entry ΣBSTAT; got:\n{buffer}"
    );
}
```

**Error handling pattern:** `.unwrap()` is allowed in test modules (`#![allow(clippy::unwrap_used)]` at file top, line 9 of analog file).

---

### `hp41-gui/src/help_data.ts` (utility, transform)

**Plan:** 36-02
**Analog:** `hp41-gui/src/help_data.ts` lines 137–154 (existing `helpEntriesMath1()` + `helpEntriesAll()` — self-template)

**Import to add** (after line 18, alongside existing `math1Functions` import):
```typescript
import stat1Functions from '../../docs/hp41-stat1-functions.json';
```

**New accessor** (add after existing `helpEntriesMath1()` at line 144, replacing the existing `helpEntriesAll()`):
```typescript
/// Phase 36 (Plan 36-02): Stat 1 Pac function entries from docs/hp41-stat1-functions.json.
///
/// Vite static JSON-import: baked into the production bundle at build time.
/// Malformed JSON fails the Vite build — hard-build-blocker semantics per
/// D-25.17 (parallel to hp41-cli/src/help_data.rs `.expect("...malformed")`).
export function helpEntriesStat1(): readonly HelpEntry[] {
    return stat1Functions as readonly HelpEntry[];
}

/// Phase 36 (Plan 36-02): 3-pool merge — built-ins + Math 1 Pac + Stat 1 Pac.
///
/// Extends the Phase 31-04 2-pool merge to a 3-pool chain.
/// Parallel to hp41-cli/src/help_data.rs::help_entries_all() 3-pool (Phase 34 D-34.2).
export function helpEntriesAll(): readonly HelpEntry[] {
    return [...helpEntries(), ...helpEntriesMath1(), ...helpEntriesStat1()];
}
```

**Pitfall 5 (RESEARCH.md):** Do NOT create a new `helpEntriesAll3()` function — UPDATE the existing `helpEntriesAll()` export in-place to the 3-pool form. All callers (HelpOverlay, tests) automatically pick up Stat 1 entries.

**Build-time gate:** Vite fails if `docs/hp41-stat1-functions.json` is malformed or missing. This is the GUI's equivalent of the CLI's `OnceLock` panic (D-25.17). The TypeScript cast `as readonly HelpEntry[]` is the same pattern as `math1Functions as readonly HelpEntry[]` at line 144.

---

### `hp41-gui/src/HelpOverlay.tsx` (component, event-driven)

**Plan:** 36-02
**Analog:** `hp41-gui/src/HelpOverlay.tsx` lines 32–132 (existing 2-section pattern — self-template)

**Four surgical edits (total ~15 LOC delta):**

**Edit 1 — widen `SectionDef.id` union** (line 33):
```typescript
// Before:
interface SectionDef {
    id: 'hp41cv' | 'math1';
    heading: string;
    predicate: (e: HelpEntry) => boolean;
}

// After:
interface SectionDef {
    id: 'hp41cv' | 'math1' | 'stat1';
    heading: string;
    predicate: (e: HelpEntry) => boolean;
}
```

**Edit 2 — add third SECTIONS entry** (after line 49, within the `SECTIONS` array):
```typescript
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
];
```

**Edit 3 — widen `expanded` state** (line 56–59):
```typescript
// Before:
const [expanded, setExpanded] = useState<{ hp41cv: boolean; math1: boolean }>({
    hp41cv: true,
    math1: true,
});

// After:
const [expanded, setExpanded] = useState<{ hp41cv: boolean; math1: boolean; stat1: boolean }>({
    hp41cv: true,
    math1: true,
    stat1: true,
});
```

**Edit 4 — update `useEffect` reset + `toggleSection` signature** (lines 65 and 130):
```typescript
// Line 65 (useEffect reset):
setExpanded({ hp41cv: true, math1: true, stat1: true });

// Line 130 (toggleSection):
const toggleSection = (id: 'hp41cv' | 'math1' | 'stat1') => {
    setExpanded(prev => ({ ...prev, [id]: !prev[id] }));
};
```

**No JSX changes required:** The `sectionGroups.map(...)` render loop at lines 160–192 iterates over `SECTIONS` generically and requires zero modification. The new third section renders automatically when `SECTIONS[2]` is populated.

**Predicate string verification:** Before writing the predicate, confirm `e.xrom?.module` value in `docs/hp41-stat1-functions.json`. Run:
```bash
grep -m1 '"module"' /Users/daniel/GitRepository/hp41-calculator-emulator/docs/hp41-stat1-functions.json
```
Expected: `"module": "Stat 1"`. If different, update the predicate string to match exactly.

---

### `hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt_stat1.rs` (test, request-response)

**Plan:** 36-02
**Analog:** `hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt.rs` (complete file — exact template)

**File header pattern** (copy from analog, lines 1–16):
```rust
// Phase 36 Plan 36-02 — Stat 1 Pac modal prompt LCD-alternation routing tests.
//
// Verifies that CalcStateView::from_state correctly routes Stat1Step modal
// prompts through the LCD-alternation D-31.5 priority branch:
//   0. [TOP branch] modal_program.is_some() && entry_buf.is_empty() && modal_prompt.is_some()
//      → display_str = truncate_with_continuation(modal_prompt)
//
// Covers all 5 Stat1Step variants. Template: lcd_alternation_modal_prompt.rs.
// Verifies STAT-GUI-04: modal prompts carry OM strings through CalcStateView.
```

**Allow-unwrap and imports** (same as analog lines 16–22):
```rust
#![allow(clippy::unwrap_used)]

use hp41_core::{
    ops::math1::modal::ModalProgram,
    ops::stat1::modal::Stat1Step,
    CalcState,
};
use hp41_gui_lib::types::CalcStateView;
```

**5 test functions** (follow the exact structure from analog lines 27–127):
```rust
// ΣNORMD MODE? — 12 chars (Σ+N+O+R+M+D+ +M+O+D+E+? = 12): no truncation
#[test]
fn normd_mode_choice_prompt_renders_verbatim() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Stat1(Stat1Step::NormdModeChoice));
    calc.modal_prompt = Some("\u{03A3}NORMD MODE?".to_string());
    assert!(calc.entry_buf.is_empty());
    let view = CalcStateView::from_state(&calc, vec![], vec![]);
    assert_eq!(view.display_str, "\u{03A3}NORMD MODE?");
}

// ν=? — 3 chars: no truncation
#[test]
fn chisqd_nu_prompt_renders_verbatim() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Stat1(Stat1Step::ChisqdNuPrompt));
    calc.modal_prompt = Some("\u{03BD}=?".to_string());
    assert!(calc.entry_buf.is_empty());
    let view = CalcStateView::from_state(&calc, vec![], vec![]);
    assert_eq!(view.display_str, "\u{03BD}=?");
}

// ΣCHISQD MODE? — 13 chars (Σ+C+H+I+S+Q+D+ +M+O+D+E+? = 13): TRUNCATES
// take(11) = "ΣCHISQD MOD" + "≡" = "ΣCHISQD MOD≡"
#[test]
fn chisqd_mode_choice_prompt_truncates() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Stat1(Stat1Step::ChisqdModeChoice));
    calc.modal_prompt = Some("\u{03A3}CHISQD MODE?".to_string());
    assert!(calc.entry_buf.is_empty());
    let view = CalcStateView::from_state(&calc, vec![], vec![]);
    assert_eq!(view.display_str, "\u{03A3}CHISQD MOD\u{2261}");
    assert_eq!(view.display_str.chars().count(), 12);
}

// DEGREE=? — 8 chars: no truncation
#[test]
fn polyp_degree_prompt_renders_verbatim() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Stat1(Stat1Step::PolypDegreePrompt(0)));
    calc.modal_prompt = Some("DEGREE=?".to_string());
    assert!(calc.entry_buf.is_empty());
    let view = CalcStateView::from_state(&calc, vec![], vec![]);
    assert_eq!(view.display_str, "DEGREE=?");
}

// SEED? — 5 chars: no truncation
#[test]
fn seed_prompt_renders_verbatim() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Stat1(Stat1Step::SeedPrompt));
    calc.modal_prompt = Some("SEED?".to_string());
    assert!(calc.entry_buf.is_empty());
    let view = CalcStateView::from_state(&calc, vec![], vec![]);
    assert_eq!(view.display_str, "SEED?");
}
```

**Critical (Pitfall 1 from RESEARCH.md):** `ΣCHISQD MODE?` is 13 Unicode scalars (Σ counts as 1; `"\u{03A3}CHISQD MODE?"`.chars().count() == 13). The LCD width is 12. The truncation branch fires: take(11) → "ΣCHISQD MOD" + "≡". The assertion is `"\u{03A3}CHISQD MOD\u{2261}"`, NOT the raw prompt string. This is NOT a test bug — the implementation is correct; the fixture must assert the truncated form.

---

### `hp41-gui/src/HelpOverlay.test.tsx` (test, event-driven)

**Plan:** 36-03
**Analog:** `hp41-gui/src/HelpOverlay.test.tsx` lines 198–244 (existing Phase 31-04 section tests — exact template)

**Imports to add** (line 19, alongside existing `math1Json`):
```typescript
import stat1Json from '../../docs/hp41-stat1-functions.json';
```

Also import the new accessor from `help_data`:
```typescript
import { helpEntries, helpOverlayRows, filterHelpEntries, helpEntriesMath1, helpEntriesAll, helpEntriesStat1 } from './help_data';
```

**New test blocks to add in `describe('help_data', ...)`:**
```typescript
it('helpEntriesStat1 returns all entries from docs/hp41-stat1-functions.json (drift-catch)', () => {
    const allStat1Source = stat1Json as unknown[];
    expect(helpEntriesStat1().length).toBe(allStat1Source.length);
    // Sanity floor: the Stat 1 JSON has 26 entries per Phase 33-34.
    expect(helpEntriesStat1().length).toBeGreaterThanOrEqual(26);
});

it('helpEntriesStat1 entries all have xrom field with module "Stat 1"', () => {
    for (const entry of helpEntriesStat1()) {
        expect(entry.xrom, `entry ${entry.op_variant} should have xrom field`).toBeTruthy();
        expect(entry.xrom!.module).toBe('Stat 1');
        expect(entry.xrom!.module_id).toBe(2);
    }
});

it('helpEntriesAll returns concatenation of built-in + Math 1 + Stat 1 entries', () => {
    const all = helpEntriesAll();
    expect(all.length).toBe(
        helpEntries().length + helpEntriesMath1().length + helpEntriesStat1().length
    );
});
```

**CRITICAL UPDATE to existing test** (Pitfall 2 from RESEARCH.md — line 226 of existing file):
```typescript
// Before (must change):
expect(sectionButtons.length).toBe(2);

// After:
expect(sectionButtons.length).toBe(3);
```

**New `describe('Stat 1 Pac section', ...)` block** (mirror of existing Math 1 section tests at lines 198–244):
```typescript
// Phase 36 D-31.9 mirror — Stat 1 Pac section tests
it('renders three top-level sections including Stat 1 Pac (XROM 2)', () => {
    const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
    const sectionButtons = container.querySelectorAll('.help-overlay-section-heading');
    const buttonTexts = Array.from(sectionButtons).map(b => b.textContent ?? '');
    expect(buttonTexts.some(t => t.includes('HP-41CV (built-in)'))).toBe(true);
    expect(buttonTexts.some(t => t.includes('Math 1 Pac (XROM 7)'))).toBe(true);
    expect(buttonTexts.some(t => t.includes('Stat 1 Pac (XROM 2)'))).toBe(true);
});

it('Stat 1 Pac section contains a Stat 1 Univariate category (D-34.1)', () => {
    const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
    const headings = container.querySelectorAll('.help-overlay-category-heading');
    const headingTexts = Array.from(headings).map(h => h.textContent ?? '');
    // Stat 1 categories (from D-34.1 7-category convention)
    expect(
        headingTexts.some(t => t.toLowerCase().includes('univariate') ||
                               t.toLowerCase().includes('distributions') ||
                               t.toLowerCase().includes('anova')),
        `Expected a Stat 1 category heading; found: ${headingTexts.join(', ')}`
    ).toBe(true);
});

it('clicking Stat 1 Pac section heading toggles aria-expanded (D-31.8)', () => {
    const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
    const sectionButtons = container.querySelectorAll('.help-overlay-section-heading');
    expect(sectionButtons.length).toBe(3);

    const stat1Button = Array.from(sectionButtons).find(b =>
        b.textContent?.includes('Stat 1 Pac')
    ) as HTMLButtonElement | undefined;
    expect(stat1Button, 'Stat 1 Pac section heading button must exist').toBeTruthy();

    // Initially expanded (aria-expanded = "true")
    expect(stat1Button!.getAttribute('aria-expanded')).toBe('true');

    // After click, collapsed
    fireEvent.click(stat1Button!);
    expect(stat1Button!.getAttribute('aria-expanded')).toBe('false');

    // After second click, expanded again
    fireEvent.click(stat1Button!);
    expect(stat1Button!.getAttribute('aria-expanded')).toBe('true');
});

it('search for "NORMD" returns Stat 1 entries', () => {
    const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
    const searchInput = container.querySelector('.help-overlay-search') as HTMLInputElement;
    fireEvent.change(searchInput, { target: { value: 'NORMD' } });
    const rows = container.querySelectorAll('.help-overlay-row');
    expect(rows.length).toBeGreaterThan(0);
    // The ΣNORMD entry must appear
    const rowTexts = Array.from(rows).map(r => r.textContent ?? '');
    expect(rowTexts.some(t => t.includes('NORMD'))).toBe(true);
});
```

**`App.test.tsx` dispatch_op end-to-end** (Plan 36-03, uses existing `mockInvoke` pattern from `App.test.tsx` lines 30–34 and 157–159):

The mock pattern for dispatch_op:
```typescript
// From App.test.tsx lines 30-34 (existing mock infrastructure)
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, args?: unknown) => mockInvoke(cmd, args),
}));

// From App.test.tsx lines 157-159 (beforeEach reset)
beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockResolvedValue(makeEmptyView());
});

// New test assertion shape (add to existing App.test.tsx describe block):
it('XEQ ΣNORMD dispatch returns CalcStateView with modal_prompt', async () => {
    // Seed the mock: get_state initial + dispatch_op response with modal_prompt set
    mockInvoke.mockResolvedValueOnce(makeEmptyView()); // initial get_state
    mockInvoke.mockResolvedValueOnce(
        makeEmptyView({ modal_prompt: '\u{03A3}NORMD MODE?' })
    ); // dispatch_op response

    await renderAppAndWait();
    // Dispatch via keyboard or XEQ modal — use invoke directly to simulate
    // the xrom_resolve path without needing the full XEQ UI flow
    await act(async () => {
        await mockInvoke('dispatch_op', { key_id: 'XEQ \u{03A3}NORMD' });
    });
    // The most recent dispatch_op call's response carries modal_prompt
    const lastCall = mockInvoke.mock.calls.find(c => c[0] === 'dispatch_op');
    expect(lastCall).toBeDefined();
});
```

---

## Shared Patterns

### Exhaustive match — no `_ =>` catch-all
**Source:** `hp41-gui/src-tauri/src/prgm_display.rs` lines 47–313 + `hp41-gui/src-tauri/tests/prgm_display_math1_arms.rs` lines 107–145
**Apply to:** `prgm_display.rs` modification

The file-text-scan test at `prgm_display_math1_arms.rs` scans for `_ =>` within `fn op_display_name`. The new 26 arms must not introduce a catch-all. The test must also be extended to cover the 26 Stat 1 variant IDs:
```rust
// Extend MATH1_VARIANT_IDS → add a parallel STAT1_VARIANT_IDS or extend the existing array
const STAT1_VARIANT_IDS: &[&str] = &[
    "SigmaBstat", "SigmaBstg", "SigmaMmtug", "SigmaMmtgd",
    "SigmaAovone", "SigmaAovtwo", "SigmaAnocov",
    "SigmaLin", "SigmaExp", "SigmaLogi", "SigmaPow",
    "SigmaMlrxy", "SigmaMlrxyz", "SigmaPolypWorkflow", "SigmaPolyc",
    "SigmaPtst", "SigmaTstat",
    "SigmaXsqev", "SigmaEfxsq", "SigmaCtkkk", "SigmaCtkk", "SigmaSpear",
    "SigmaNormdWorkflow", "SigmaChisqdWorkflow",
    "Rand", "Seed",
];
```

### `#![allow(clippy::unwrap_used)]` in test files
**Source:** `hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt.rs` line 16
**Apply to:** `lcd_alternation_modal_prompt_stat1.rs`

All integration test files in `hp41-gui/src-tauri/tests/` carry this at the file top. Core test modules carry `#[allow(clippy::unwrap_used)]` at the `mod tests` scope. Production code never uses `.unwrap()` — uses `.expect("reason")` or `?`.

### Vite JSON import pattern
**Source:** `hp41-gui/src/help_data.ts` lines 17–18 (existing two imports)
**Apply to:** `help_data.ts` modification

```typescript
import functions from '../../docs/hp41cv-functions.json';
import math1Functions from '../../docs/hp41-math1-functions.json';
import stat1Functions from '../../docs/hp41-stat1-functions.json';  // NEW
```

All three paths are relative to `hp41-gui/src/`. The `../../docs/` path traverses up from `src/` to `hp41-gui/` then to the repo root where `docs/` lives. The TypeScript cast `as readonly HelpEntry[]` is the common coercion pattern for all three.

### `CalcStateView::from_state` call signature
**Source:** `hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt.rs` line 36
**Apply to:** `lcd_alternation_modal_prompt_stat1.rs`

```rust
let view = CalcStateView::from_state(&calc, vec![], vec![]);
//                                          ^^^^^^^^^^^^ print_lines, program_steps
```

The second and third args are `print_lines: Vec<String>` and `program_steps: Vec<String>`. Always pass empty vecs for modal-prompt routing tests — the field being tested is `display_str`, which is set by the LCD-alternation branch before any step/print logic.

### `op_catalog` `#![allow(clippy::unwrap_used)]` in test files
**Source:** `hp41-core/tests/op_catalog_xrom.rs` line 9
**Apply to:** new `catalog_2_lists_stat1_when_bit1_set` test

The file already has `#![allow(clippy::unwrap_used)]`. The new test function uses `.unwrap()` on `op_catalog(...).unwrap()` — consistent with the existing pattern.

---

## No Analog Found

All Phase 36 files have clear analogs. No files lack a match.

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | All files are extensions of existing patterns |

---

## Anti-Patterns (from RESEARCH.md — include in plans)

| Anti-Pattern | Where | Why Forbidden |
|-------------|-------|---------------|
| `_ =>` catch-all in `op_display_name` | `prgm_display.rs` | Silently hides future missing arms; violates 4-way invariant |
| Touching any file in `hp41-core/src/ops/math1/` except `program.rs` | All plans | math1/ freeze per D-33.3; only `xrom.rs` and `modal.rs` had sanctioned carve-outs in Phase 33 |
| New Tauri commands or permissions TOML files | Plans 36-02/03 | Phase 36 has no new inline commands; existing `dispatch_op` et al. cover the surface |
| `helpEntriesAll3()` duplicate function | `help_data.ts` | Must UPDATE existing `helpEntriesAll()` in-place, not create a parallel function |
| `Record<string, boolean>` for `expanded` | `HelpOverlay.tsx` | TypeScript requires the literal union `'hp41cv' | 'math1' | 'stat1'` for `expanded[section.id]` to compile |
| Asserting raw truncatable prompt in LCD test | `lcd_alternation_modal_prompt_stat1.rs` | `ΣCHISQD MODE?` (13 chars) must assert the TRUNCATED form `ΣCHISQD MOD≡`, not the raw string |
| Forgetting `sectionButtons.length === 2` update | `HelpOverlay.test.tsx` line 226 | This exact assertion WILL fail after Plan 36-02 adds the third section button; must change to `=== 3` |
| Nested `else if` instead of parallel `if` for STAT_1 block | `program.rs` op_catalog | Must be two sibling `if` blocks (bit-0 and bit-1) so both pacs list when both bits are set |

---

## Metadata

**Analog search scope:** `hp41-gui/src-tauri/src/`, `hp41-gui/src-tauri/tests/`, `hp41-gui/src/`, `hp41-core/src/ops/program.rs`, `hp41-core/tests/`, `hp41-cli/src/prgm_display.rs`
**Files scanned:** 11 source files, 1 test directory listing
**Pattern extraction date:** 2026-05-24
