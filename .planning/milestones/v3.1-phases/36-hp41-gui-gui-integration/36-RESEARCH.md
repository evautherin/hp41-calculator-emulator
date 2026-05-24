# Phase 36: hp41-gui — GUI Integration - Research

**Researched:** 2026-05-24
**Domain:** Tauri v2 + React GUI integration; Rust hp41-core surgical extension
**Confidence:** HIGH

## Summary

Phase 36 is a pure integration phase — all behavioral logic was implemented in Phase 33 (hp41-core), all CLI display surfaces were wired in Phase 34, and all documentation was completed in Phase 35. Phase 36's job is to mirror Phase 34's CLI surface into the GUI: add 26 `op_display_name` arms to `hp41-gui/src-tauri/src/prgm_display.rs`, extend `op_catalog` in `hp41-core/src/ops/program.rs` with a bit-1 block for STAT_1, extend `help_data.ts` with a third Vite JSON import, and extend `HelpOverlay.tsx` with a third collapsible section.

The existing codebase is meticulously structured. Every change in Phase 36 has a direct structural template already present: the 26 GUI `op_display_name` arms copy verbatim from Phase 34's CLI file; the `op_catalog` bit-1 block copies from the existing bit-0 block with MATH_1 replaced by STAT_1; the `HelpOverlay.tsx` third section copies the existing `'math1'` entry; the `help_data.ts` third import copies the existing `math1Functions` import. All TypeScript and Rust type changes are narrow (union type widening, single const changes).

The 4-way exhaustive-match invariant has had item 4 (GUI `prgm_display.rs`) open since Phase 33 ship. Phase 36 closes it. The intentional `non-exhaustive patterns` CI break in `hp41-gui` closes when Plan 36-01 lands. STAT-GUI-05 was reassigned to Phase 37 per D-36.2 — bounded 50-iter distribution primitives complete in microseconds and do not need cancellation.

**Primary recommendation:** Follow the 3-plan slice (D-36.4) exactly as specified in CONTEXT.md. Every change is a small delta on an established template — minimize deviation, maximize verbatim copying from Phase 34 and existing Phase 31 patterns.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-25.6 (CLI ↔ GUI parity):** Every Stat 1 behavior the GUI gains here routes through shared `hp41-core` code. The 26 display-name arms in `hp41-gui/src-tauri/src/prgm_display.rs` are pure strings. Phase 34's CLI version is the source-of-truth for string content; Phase 36 GUI version copies verbatim.

**D-28.4 / XROM-09 / D-33.3b:** `modal_prompt: Option<String>` is the channel for prompt strings. Phase 36 reads it via existing `CalcStateView::from_state` plumbing. No new field plumbing.

**D-28.6:** XEQ-by-name only — no dedicated key bindings for Stat 1 functions. Stat 1 mnemonics resolve via `xeq_by_name` → shared `xrom_resolve` → Op variant in both CLI and GUI.

**D-31.4 / D-31.6 (cancellation pattern):** STAT-GUI-05 reassigned to Phase 37 per D-36.2. Bounded 50-iter primitives do not need cancellation.

**D-31.8 (HelpOverlay shape):** Extend to THREE sections, preserving existing collapsible-state-per-section convention. All three default open.

**D-31.9 (HelpOverlay test shape):** vitest tests assert section presence, collapse/expand, and search filter. Phase 36 mirrors existing assertion structure for the new third section.

**D-31.12 / D-31.14 (no PSE-step in CATALOG):** CATALOG 2 pushes synchronously. Phase 36 bit-1 block uses the same single-pass `print_buffer` push.

**D-33.3 / D-33.3b:** math1/ freeze carve-outs are Phase 33 ONLY. Phase 36 does NOT extend them. All math1/ files remain frozen.

**D-33.7:** `migrate_after_load()` — single source of truth. Phase 36 does NOT touch it.

**D-34.1 (JSON category convention):** 7 per-family Stat 1 categories already in `docs/hp41-stat1-functions.json`. Phase 36 consumes read-only.

**D-34.5 (overlay section header):** `"Stat 1 Pac (XROM 2)"` — locked.

**D-34.6 (overlay render order):** Built-ins → Math 1 → Stat 1 — locked.

**D-36.1:** Mirror the existing bit-0 conditional block for bit-1 in `op_catalog`. ~10 LOC delta. No abstraction. Parallel `if` blocks coexist. Refactor deferred until third XROM module.

**D-36.2:** STAT-GUI-05 deferred. Bounded 50-iter AS 241/239/63 primitives complete in microseconds. No cancellation needed.

**D-36.3:** Reassign STAT-GUI-05 Phase 36 → Phase 37. First commit of Plan 36-01 updates REQUIREMENTS.md + ROADMAP.md.

**D-36.4:** Phase 36 ships as 3 plans (36-01: arms + CATALOG 2 + bookkeeping; 36-02: HelpOverlay + help_data.ts + Rust modal-flow test; 36-03: vitest extensions + dispatch end-to-end).

### Claude's Discretion

- HelpOverlay `expanded` state shape: use `{ hp41cv: boolean; math1: boolean; stat1: boolean }` explicit literal (not `Record<string, boolean>`). Planner has minor discretion.
- Default expanded state for `'stat1'` section: `true` (open). Planner may flip to `false` if visual noise warrants; recommend hold.
- `SECTIONS` `id` union type: `'hp41cv' | 'math1' | 'stat1'` — explicit literal, NOT `string` (required for `expanded[section.id]` type narrowing).
- `SectionDef.predicate` for Stat 1: `(e: HelpEntry) => e.xrom?.module === 'Stat 1'` (verify against JSON at write time).
- vitest test file shape: add `describe('Stat 1 Pac section', ...)` block mirroring `'Math 1 Pac (XROM 7) section'`.
- Rust integration test name: `lcd_alternation_modal_prompt_stat1.rs`.
- CATALOG 2 test name: `catalog_2_lists_stat1_when_bit1_set`.
- File header comment in `prgm_display.rs`: update Op-variant count.
- Plan 36-01 commit order: (a) D-36.3 bookkeeping first, (b) 26 GUI arms, (c) `op_catalog` bit-1 extension + test.
- Plan 36-02 commit order: (a) `help_data.ts` extension, (b) `HelpOverlay.tsx` SECTIONS extension, (c) `lcd_alternation_modal_prompt_stat1.rs` Rust test.
- Plan 36-03 commit order: (a) vitest extensions, (b) dispatch end-to-end.

### Deferred Ideas (OUT OF SCOPE)

- STAT-GUI-05 cancellation (reassigned to Phase 37 STAT-QUAL block)
- `docs/hp41-stat1-divergences.md` D-35-NN behavioral policy entry for bounded-iter policy (Phase 37 or post-ship)
- CLAUDE.md Phase 36 sub-section fill (Phase 36 ship-time)
- `docs/architecture-history.md` Phase 36 narrative (Phase 36 ship-time)
- PROJECT.md Shipped / Current focus updates (Phase 36 ship-time)
- WebdriverIO E2E smoke with Stat 1 workflow (Phase 37 / STAT-QUAL-11)
- `numerical_accuracy.rs` Stat 1 oracle cases (Phase 37 / STAT-QUAL-04)
- Per-file coverage floor ≥ 90 % for stat1/*.rs (Phase 37 / STAT-QUAL-03)
- `stat1_op_test_count.rs` / `lint_stat1_assertions.rs` / `xrom_shadowing.rs` STAT_1 extension (Phase 37)
- Backward-compat test for v3.0 save migration (Phase 37 / STAT-QUAL-10)
- CATALOG 2 generic loop refactor (v3.2+ when third XROM module lands)
- HelpOverlay `Record<SectionId, boolean>` generic-state refactor (same deferral)
- Signed binary releases (v3.1.x / v3.2)
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| STAT-GUI-01 | `hp41-gui/src-tauri/src/prgm_display.rs` gains the same ~26 new `op_display_name` arms — SC-4 preserved, 4-way invariant item 4 | 26 arms verified in CLI file at lines 295–327; copy verbatim per D-25.6 |
| STAT-GUI-02 | CATALOG 2 XROM enumeration discovers and displays `STAT_1` alongside `MATH_1` | `op_catalog` bit-0 block at program.rs:343–354 is the direct template; STAT_1 const verified at xrom.rs:141, id=2, name="STAT 1B", 26 ops |
| STAT-GUI-03 | `?` overlay adds third JSON-import section "Stat 1 Pac (XROM 2)" — search across all three sections | `help_data.ts` 2-import pattern at lines 17–18, 143–154; `HelpOverlay.tsx` SECTIONS array at lines 39–50 are direct extension targets |
| STAT-GUI-04 | LCD-alternation modal prompts reuse existing modal-prompt infrastructure | 5 Stat1Step prompts verified in stat1/modal.rs:333–343; `lcd_alternation_modal_prompt.rs` is the direct template for the new integration test |
| STAT-GUI-05 | `request_cancel` reuse for iterative-quantile paths — **REASSIGNED to Phase 37 per D-36.2 / D-36.3** | D-36.2: closed-form Acklam + bounded ≤50-iter AS 239/63; no cancellation needed at this timescale |
</phase_requirements>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Op display name strings for PRGM listing | GUI Frontend Server (Tauri) | — | `prgm_display.rs` is the GUI-side deliberate duplicate of CLI `prgm_display.rs` per 4-way invariant; pure string lookup |
| CATALOG 2 XROM enumeration | Core library (hp41-core) | Both CLI + GUI inherit | `op_catalog` is shared hp41-core code; both frontends get the fix for free via `CalcStateView.print_lines` |
| Help overlay data loading | GUI Frontend (React/Vite) | — | Vite static-import of JSON at build time; TypeScript type-checks the schema |
| Help overlay section rendering | GUI Frontend (React) | — | `HelpOverlay.tsx` component; `helpEntriesAll()` pool merging in `help_data.ts` |
| Modal prompt display | Core library + Tauri IPC | GUI Frontend reads CalcStateView | `modal_prompt` field already plumbed end-to-end since Phase 31; Phase 36 reads without modification |
| Stat 1 function resolution (XEQ) | Core library (hp41-core) | — | `xrom_resolve` bit-1 arm activated Phase 33; GUI inherits via `dispatch_op` → `xrom_resolve` |

---

## Standard Stack

No new dependencies introduced in Phase 36 per locked decisions.

### Core (existing — consumed as-is)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tauri v2 | 2.11 | GUI framework; IPC between Rust backend and React frontend | Project-wide frozen invariant |
| React 18 + TypeScript | 18 / 5.x | Frontend rendering | Project-wide frozen invariant |
| Vite | 5.x | Build tool; static JSON import at build time | Project-wide; enables build-time JSON type-check |
| vitest | current | Frontend unit/component tests | Project-wide |
| `@testing-library/react` | current | React component rendering in vitest | Project-wide; used in `HelpOverlay.test.tsx` |

[VERIFIED: codebase — `hp41-gui/package.json`, `hp41-gui/vite.config.ts`]

### Alternatives Considered
None — zero new dependencies per D-36.1 locked decision.

**Installation:** None required — no new packages.

---

## Package Legitimacy Audit

> **SKIPPED** — Phase 36 installs zero external packages. No runtime or dev dependency changes in any of the three workspaces (`hp41-core`, `hp41-cli`, `hp41-gui`). Zero-new-runtime-deps is a frozen invariant for this milestone per CLAUDE.md and STATE.md.

---

## Architecture Patterns

### System Architecture Diagram

```
User (GUI keyboard / XEQ modal)
         │
         ▼
  React frontend (App.tsx / Keyboard.tsx)
         │  invoke('dispatch_op', { key_id: 'XEQ ΣNORMD' })
         ▼
  Tauri IPC command: dispatch_op (commands.rs)
         │
         ▼
  hp41-core key_map::resolve(key_id)
         │  → xeq_by_name_local_resolve
         │  → xrom_resolve(name, state.xrom_modules)  ← bit-1 arm active since Phase 33
         │
         ▼
  hp41-core dispatch(Op::SigmaNormdWorkflow, state)
         │  → sets state.modal_program = Some(ModalProgram::Stat1(NormdModeChoice))
         │  → sets state.modal_prompt = Some("ΣNORMD MODE?")
         ▼
  CalcStateView::from_state(state)  ← carries modal_prompt; no Phase 36 change
         │
         ▼
  React frontend updates display_str from CalcStateView.modal_prompt
```

For CATALOG 2 (STAT-GUI-02):
```
Op::Catalog(2) dispatch
         │
         ▼
  op_catalog(state, 2)  ← Phase 36 extends with bit-1 block
         │  if xrom_modules & 0b0000_0001 → push MATH_1 header + 45 entries
         │  if xrom_modules & 0b0000_0010 → push STAT_1 header + 26 entries  [NEW]
         ▼
  state.print_buffer → CalcStateView.print_lines → GUI print display
```

For `?` overlay (STAT-GUI-03):
```
Vite build time:
  hp41cv-functions.json  ─┐
  hp41-math1-functions.json ─┤─ help_data.ts → helpEntriesAll()  [3-pool after Phase 36]
  hp41-stat1-functions.json ─┘         [NEW import in Phase 36]
         │
         ▼
  HelpOverlay.tsx SECTIONS array
    [0] 'hp41cv'  → predicate: !e.xrom
    [1] 'math1'   → predicate: e.xrom?.module === 'Math 1'
    [2] 'stat1'   → predicate: e.xrom?.module === 'Stat 1'  [NEW in Phase 36]
```

### Recommended Project Structure

Existing structure unchanged. Phase 36 modifies:

```
hp41-core/src/ops/
  program.rs          ← op_catalog bit-1 block + import STAT_1 (~10 LOC)
hp41-core/tests/
  op_catalog_xrom.rs  ← new test fn catalog_2_lists_stat1_when_bit1_set

hp41-gui/src-tauri/src/
  prgm_display.rs     ← 26 new Op arms (Phase 36 Plan 36-01)
hp41-gui/src-tauri/tests/
  lcd_alternation_modal_prompt_stat1.rs  ← new (Phase 36 Plan 36-02)
  prgm_display_math1_arms.rs             ← may extend with stat1 variant IDs

hp41-gui/src/
  help_data.ts        ← third Vite import + helpEntriesStat1() + 3-pool helpEntriesAll()
  HelpOverlay.tsx     ← SECTIONS[2], expanded state, id union, toggleSection sig.
  HelpOverlay.test.tsx ← Stat 1 section tests

.planning/
  REQUIREMENTS.md     ← STAT-GUI-05 reassignment (row 222 + line 91)
  ROADMAP.md          ← Phase 36 success criterion #4 + Phase 37 requirements line
```

### Pattern 1: Verbatim Copy for 4-Way Invariant Arms (D-25.6)

**What:** The 26 Stat 1 `op_display_name` arms in `hp41-gui/src-tauri/src/prgm_display.rs` are DELIBERATELY IDENTICAL to the CLI counterpart at `hp41-cli/src/prgm_display.rs:295–327`. Copying verbatim is the correct implementation.

**When to use:** Always when closing 4-way invariant item 4.

**Example** (from CLI source `hp41-cli/src/prgm_display.rs:295–327`):
```rust
// Source: hp41-cli/src/prgm_display.rs lines 295-327 (Phase 34)
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
[VERIFIED: codebase — `hp41-cli/src/prgm_display.rs:295–327`]

### Pattern 2: op_catalog Bit-1 Extension (D-36.1)

**What:** Mirror the existing bit-0 MATH_1 block for bit-1 STAT_1 in `hp41-core/src/ops/program.rs::op_catalog`.

**When to use:** Plan 36-01 only.

**Example** (to be placed immediately after the existing MATH_1 `if` block, before the `else` fallback):
```rust
// Source: hp41-core/src/ops/program.rs lines 343-354 (existing MATH_1 block — template)
// New block mirrors exactly — swap bit mask + MATH_1 → STAT_1:
if state.xrom_modules & 0b0000_0010 != 0 {
    // Stat 1 Pac (bit 1) is loaded.
    state.print_buffer.push(format!(
        "{:<24}",
        format!("XROM {} {}", STAT_1.id, STAT_1.name)
    ));
    for (name, _op) in STAT_1.ops {
        state.print_buffer.push(format!("{name:<24}"));
    }
}
// The existing `else { state.print_buffer.push(format!("{:<24}", "NO XROM")); }`
// fires only when BOTH bits are clear — currently impossible post-migrate_after_load.
// Keep as defensive code per D-36.1.
```

Import to add alongside existing `use crate::ops::math1::xrom::MATH_1;`:
```rust
use crate::ops::math1::xrom::{MATH_1, STAT_1};
```
[VERIFIED: codebase — `hp41-core/src/ops/program.rs:343–354`, `hp41-core/src/ops/math1/xrom.rs:141–186`]

### Pattern 3: Vite Static-Import Third Pool (D-25.17 GUI analog)

**What:** Add `docs/hp41-stat1-functions.json` as a third Vite static import in `help_data.ts`. TypeScript compile failure on malformed JSON is the hard-build-blocker equivalent of the CLI's `OnceLock` panic.

**When to use:** Plan 36-02.

**Example** (mirror of existing `math1Functions` import lines 18 and 143–154):
```typescript
// Source: hp41-gui/src/help_data.ts lines 17-18 (existing pattern)
import stat1Functions from '../../docs/hp41-stat1-functions.json';

// Phase 36 (Plan 36-02): Stat 1 Pac function entries from docs/hp41-stat1-functions.json.
export function helpEntriesStat1(): readonly HelpEntry[] {
    return stat1Functions as readonly HelpEntry[];
}

// Phase 36 (Plan 36-02): 3-pool merge — built-ins + Math 1 Pac + Stat 1 Pac.
export function helpEntriesAll(): readonly HelpEntry[] {
    return [...helpEntries(), ...helpEntriesMath1(), ...helpEntriesStat1()];
}
```
[VERIFIED: codebase — `hp41-gui/src/help_data.ts:17–154`]

### Pattern 4: HelpOverlay SECTIONS Extension (D-31.8)

**What:** Widen `SectionDef.id` union, add third entry to `SECTIONS` array, widen `expanded` state, update `toggleSection` signature, update `useEffect` reset.

**When to use:** Plan 36-02.

**Example** (surgical edits to `hp41-gui/src/HelpOverlay.tsx`):

```typescript
// Source: hp41-gui/src/HelpOverlay.tsx lines 32-36 (existing SectionDef)
// Change: widen id union
interface SectionDef {
    id: 'hp41cv' | 'math1' | 'stat1';  // add | 'stat1'
    heading: string;
    predicate: (e: HelpEntry) => boolean;
}

// Source: hp41-gui/src/HelpOverlay.tsx lines 39-50 (existing SECTIONS)
// Change: add third entry
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

// Source: hp41-gui/src/HelpOverlay.tsx lines 56-59 (existing expanded state)
// Change: widen state type + add stat1: true
const [expanded, setExpanded] = useState<{ hp41cv: boolean; math1: boolean; stat1: boolean }>({
    hp41cv: true,
    math1: true,
    stat1: true,
});

// Source: hp41-gui/src/HelpOverlay.tsx line 65 (reset on open)
setExpanded({ hp41cv: true, math1: true, stat1: true });

// Source: hp41-gui/src/HelpOverlay.tsx line 130 (toggleSection)
// Change: widen id parameter
const toggleSection = (id: 'hp41cv' | 'math1' | 'stat1') => {
    setExpanded(prev => ({ ...prev, [id]: !prev[id] }));
};
```
[VERIFIED: codebase — `hp41-gui/src/HelpOverlay.tsx:32–132`]

### Pattern 5: Modal Prompt Integration Test (lcd_alternation pattern)

**What:** Add `lcd_alternation_modal_prompt_stat1.rs` as a new integration test in `hp41-gui/src-tauri/tests/`. Construct `CalcState` directly; set `modal_program` + `modal_prompt`; call `CalcStateView::from_state`; assert `display_str`.

**When to use:** Plan 36-02.

**Example** (template from existing `lcd_alternation_modal_prompt.rs`):
```rust
// Source: hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt.rs (template)
use hp41_core::{
    ops::math1::modal::{ModalProgram},
    ops::stat1::modal::Stat1Step,
    CalcState,
};
use hp41_gui_lib::types::CalcStateView;

#[test]
fn normd_mode_choice_prompt_routes_to_display() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Stat1(Stat1Step::NormdModeChoice));
    calc.modal_prompt = Some("\u{03A3}NORMD MODE?".to_string()); // 12 chars
    assert!(calc.entry_buf.is_empty());
    let view = CalcStateView::from_state(&calc, vec![], vec![]);
    assert_eq!(view.display_str, "\u{03A3}NORMD MODE?");
}
// Mirror for: ChisqdNuPrompt ("ν=?"), ChisqdModeChoice ("ΣCHISQD MODE?"),
//             PolypDegreePrompt(0) ("DEGREE=?"), SeedPrompt ("SEED?")
```

Stat1Step prompt strings (from `hp41-core/src/ops/stat1/modal.rs:333–343`):
- `Stat1Step::NormdModeChoice` → `"\u{03A3}NORMD MODE?"` (12 chars — no truncation)
- `Stat1Step::ChisqdNuPrompt` → `"\u{03BD}=?"` (3 chars — no truncation)
- `Stat1Step::ChisqdModeChoice` → `"\u{03A3}CHISQD MODE?"` (14 chars — truncates to `"\u{03A3}CHISQD MODE\u{2261}"`)
- `Stat1Step::PolypDegreePrompt(0)` → `"DEGREE=?"` (8 chars — no truncation)
- `Stat1Step::SeedPrompt` → `"SEED?"` (5 chars — no truncation)

[VERIFIED: codebase — `hp41-core/src/ops/stat1/modal.rs:333–343`]

**Important:** `ΣCHISQD MODE?` is 14 chars and WILL be truncated by the `CalcStateView::from_state` LCD-width-12 branch. The test for this variant must assert the truncated form `"\u{03A3}CHISQD MODE\u{2261}"` (first 11 chars + `≡` continuation marker), NOT the raw prompt string. See `lcd_alternation_modal_prompt.rs:88–108` for the truncation assertion pattern.

[VERIFIED: codebase — `hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt.rs:64–108`]

### Pattern 6: vitest Section Tests (D-31.9 mirror)

**What:** Add `describe('Stat 1 Pac section', ...)` block to `HelpOverlay.test.tsx` mirroring the existing Math 1 Pac tests.

**Key assertions to include:**
```typescript
// Source: hp41-gui/src/HelpOverlay.test.tsx lines 200-244 (Math 1 template)
it('renders three top-level sections including Stat 1 Pac (XROM 2)', () => {
    const { container } = render(<HelpOverlay open={true} onClose={() => {}} />);
    const sectionButtons = container.querySelectorAll('.help-overlay-section-heading');
    expect(sectionButtons.length).toBe(3); // was 2, now 3
    const buttonTexts = Array.from(sectionButtons).map(b => b.textContent ?? '');
    expect(buttonTexts.some(t => t.includes('Stat 1 Pac (XROM 2)'))).toBe(true);
});
```

**IMPORTANT:** The existing test `'clicking section heading toggles aria-expanded'` asserts `sectionButtons.length === 2`. Phase 36 Plan 36-03 MUST update this assertion to `=== 3`.

[VERIFIED: codebase — `hp41-gui/src/HelpOverlay.test.tsx:226–228`]

### Anti-Patterns to Avoid

- **Adding `_ =>` catch-all to `op_display_name`:** Never — the exhaustive match is the 4-way invariant mechanism. The compile error from missing arms is intentional.
- **Touching `math1/` frozen files:** Phase 36 does NOT modify `hp41-core/src/ops/math1/` (except the `op_catalog` in `program.rs` which is outside the freeze boundary).
- **Duplicating modal dispatch logic in GUI:** All `ModalProgram::Stat1` dispatch routes through shared `hp41-core` code. The GUI only reads the resulting `CalcStateView`.
- **Adding new Tauri commands or permissions:** Phase 36 does NOT add inline commands. Existing `dispatch_op`, `get_state`, etc. cover the surface.
- **Using `_=>` in HelpOverlay `expanded[section.id]` access:** The `id` union must remain a literal (`'hp41cv' | 'math1' | 'stat1'`) — TypeScript requires this for the indexing to compile.
- **Forgetting to update the existing `sectionButtons.length === 2` assertion:** This test will fail when a third section button renders. Update to `=== 3` in Plan 36-03.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Stat 1 Op display names | Derive from mnemonic / auto-generate | Copy verbatim from `hp41-cli/src/prgm_display.rs:295–327` | D-25.6 parity invariant requires byte-identical strings; the CLI is the source-of-truth |
| CATALOG 2 enumeration logic | Generic registry loop | Parallel `if` blocks per D-36.1 | Two modules only; premature abstraction deferred to v3.2+ third module |
| JSON help data merging | Custom merge/dedup logic | `[...helpEntries(), ...helpEntriesMath1(), ...helpEntriesStat1()]` spread | Matches existing 2-pool pattern exactly |
| Modal prompt routing in GUI | New TypeScript modal state machine | Read `CalcStateView.modal_prompt` from dispatch response | Already plumbed end-to-end since Phase 31; no GUI-side changes needed |
| Distribution function cancellation | AtomicBool checks in distribution primitives | Nothing — bounded ≤50 iter, <1ms | D-36.2: no user-visible benefit; would pollute pure-math signatures |

---

## Common Pitfalls

### Pitfall 1: ΣCHISQD MODE? Truncation in LCD Display Test

**What goes wrong:** The test for `Stat1Step::ChisqdModeChoice` asserts `view.display_str == "ΣCHISQD MODE?"` (14 chars) but the LCD width is 12. The `CalcStateView::from_state` truncation branch fires: first 11 chars + `≡` (U+2261). The test fails.

**Why it happens:** The LCD-alternation truncation branch at `CalcStateView::from_state` truncates prompts longer than 12 chars. `ΣCHISQD MODE?` is 14 chars (the Σ counts as 1 Unicode char, so: Σ-C-H-I-S-Q-D- -M-O-D-E-? = 13 chars actually — planner must count carefully from the raw Unicode string `"\u{03A3}CHISQD MODE?"`: Σ(1)+C(2)+H(3)+I(4)+S(5)+Q(6)+D(7)+space(8)+M(9)+O(10)+D(11)+E(12)+?(13) = 13 chars, which IS 13 > 12 → truncated to first 11 chars + `≡`).

**How to avoid:** Assert the TRUNCATED form. Template: `lcd_alternation_modal_prompt.rs:68–108` (the 13-char truncation test). Expected: `"\u{03A3}CHISQD MOD\u{2261}"` (11 chars of the prompt + ≡ continuation marker).

**Warning signs:** Test failure message shows `left: "ΣCHISQD MODE?"` but `right: "ΣCHISQD MOD≡"` — the truncation branch is correct; the test fixture was wrong.

### Pitfall 2: Existing HelpOverlay Test Asserts `sectionButtons.length === 2`

**What goes wrong:** `HelpOverlay.test.tsx` line 226 asserts `expect(sectionButtons.length).toBe(2)`. After Plan 36-02 adds the third section, this test fails.

**Why it happens:** The test was written for the 2-section Phase 31 state.

**How to avoid:** In Plan 36-03, update the assertion to `toBe(3)` and update the comment. This is the ONLY existing test that hard-codes the section count.

**Warning signs:** `vitest` run fails with `expect(received).toBe(expected): Expected: 2, Received: 3` in `HelpOverlay.test.tsx`.

### Pitfall 3: op_catalog `else` Branch vs Parallel `if` Blocks

**What goes wrong:** The existing `op_catalog` code has `if (bit-0) { MATH_1 } else { NO XROM }`. Naively adding the STAT_1 block as another `else if` or changing the `else` creates logical errors when only bit-1 is set (no MATH_1 but has STAT_1 — currently impossible but defensive code must be correct).

**Why it happens:** Misreading the existing structure as an `if/else if/else` chain rather than a new parallel `if` block.

**How to avoid:** Add the STAT_1 block as a SEPARATE `if` statement (not `else if`) AFTER the MATH_1 block, before the `else` fallback. The `else` then only fires when BOTH bits are clear. See D-36.1 and the code example in Pattern 2 above.

**Warning signs:** When `xrom_modules = 0b0000_0010` (only STAT_1), CATALOG 2 still shows "NO XROM" — the bit-1 check was accidentally placed inside the bit-0 `else` branch.

### Pitfall 4: Missing `use crate::ops::math1::xrom::STAT_1;` Import

**What goes wrong:** Adding the STAT_1 bit-1 block in `program.rs` without adding `STAT_1` to the import. The existing import is `use crate::ops::math1::xrom::MATH_1;` (singular). `STAT_1` is not in scope.

**Why it happens:** The existing single-import pattern misleads.

**How to avoid:** Change to `use crate::ops::math1::xrom::{MATH_1, STAT_1};` or add a separate `use` line.

**Warning signs:** Rust compile error `unresolved import STAT_1` or `cannot find value STAT_1 in scope`.

### Pitfall 5: `helpEntriesAll` Already Exported — Must Not Duplicate

**What goes wrong:** Adding a NEW `helpEntriesAll()` function alongside the existing one. Since `helpEntriesAll` is currently a 2-pool merge, the instinct is to add a new `helpEntriesAll3()` function. But the CONTEXT specifies REPLACING `helpEntriesAll()` to be a 3-pool chain.

**Why it happens:** Fear of breaking existing callers.

**How to avoid:** Update the existing `helpEntriesAll()` export in-place to add the third pool. All callers (HelpOverlay, tests) automatically get Stat 1 entries. The test `helpEntriesAll returns concatenation of built-in + Math 1 entries` in `HelpOverlay.test.tsx:47–58` will need updating to assert the 3-pool total.

**Warning signs:** After Plan 36-02, the Stat 1 section in HelpOverlay shows no entries — the import exists but `helpEntriesAll()` still returns the 2-pool result.

### Pitfall 6: SC-4 Grep Must Still Return Empty After Plan 36-01

**What goes wrong:** Accidentally adding any `op_*` math function to `hp41-gui/src-tauri/src/` while adding the 26 display-name arms.

**Why it happens:** Copy-paste from hp41-core accidentally includes function bodies.

**How to avoid:** The 26 new arms are pure string literals (`"ΣBSTAT".to_string()`, etc.). No function calls, no `op_*` invocations. Run `grep -rn "fn op_(add|sub|mul|div|sin|cos|tan|sto|rcl|flush_entry|format_hpnum)" hp41-gui/src-tauri/src/` after Plan 36-01 — must return empty.

---

## Code Examples

### CATALOG 2 Test for STAT_1 (Plan 36-01)

```rust
// Source: hp41-core/tests/op_catalog_xrom.rs (template); new test extends the file
#[test]
fn catalog_2_lists_stat1_when_bit1_set() {
    let mut state = CalcState {
        xrom_modules: 0b0000_0011,
        ..CalcState::default()
    };
    op_catalog(&mut state, 2).unwrap();
    let buffer = state.print_buffer.join("\n");
    // MATH_1 header: "XROM 7 MATH 1A"
    assert!(buffer.contains("MATH 1A"), "CATALOG 2 must list MATH 1A (bit 0)");
    // STAT_1 header: "XROM 2 STAT 1B"
    assert!(buffer.contains("STAT 1B"), "CATALOG 2 must list STAT 1B (bit 1)");
    // Spot-check one entry from each pac:
    assert!(buffer.contains("Y^X"), "CATALOG 2 must contain a Math 1 entry (YPow)");
    assert!(buffer.contains("\u{03A3}BSTAT"), "CATALOG 2 must contain a Stat 1 entry (ΣBSTAT)");
}
```

### Stat 1 Modal Prompt Integration Test (Plan 36-02)

```rust
// New file: hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt_stat1.rs
// Template: hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt.rs
#![allow(clippy::unwrap_used)]

use hp41_core::{
    ops::math1::modal::ModalProgram,
    ops::stat1::modal::Stat1Step,
    CalcState,
};
use hp41_gui_lib::types::CalcStateView;

// ΣNORMD MODE? = 12 chars (Σ=1, N=2,...,?=12) — no truncation
#[test]
fn normd_mode_choice_prompt_renders_verbatim() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Stat1(Stat1Step::NormdModeChoice));
    calc.modal_prompt = Some("\u{03A3}NORMD MODE?".to_string());
    let view = CalcStateView::from_state(&calc, vec![], vec![]);
    assert_eq!(view.display_str, "\u{03A3}NORMD MODE?");
}

// ν=? = 3 chars — no truncation
#[test]
fn chisqd_nu_prompt_renders_verbatim() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Stat1(Stat1Step::ChisqdNuPrompt));
    calc.modal_prompt = Some("\u{03BD}=?".to_string());
    let view = CalcStateView::from_state(&calc, vec![], vec![]);
    assert_eq!(view.display_str, "\u{03BD}=?");
}

// ΣCHISQD MODE? = 13 chars — truncates to first 11 + ≡ (U+2261)
#[test]
fn chisqd_mode_choice_prompt_truncates() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Stat1(Stat1Step::ChisqdModeChoice));
    calc.modal_prompt = Some("\u{03A3}CHISQD MODE?".to_string());
    let view = CalcStateView::from_state(&calc, vec![], vec![]);
    // chars: Σ(1)C(2)H(3)I(4)S(5)Q(6)D(7) (8)M(9)O(10)D(11)E(12)?(13) = 13 chars
    // take(11) = "ΣCHISQD MOD" + "≡" = "ΣCHISQD MOD≡"
    assert_eq!(view.display_str, "\u{03A3}CHISQD MOD\u{2261}");
}

// DEGREE=? = 8 chars — no truncation
#[test]
fn polyp_degree_prompt_renders_verbatim() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Stat1(Stat1Step::PolypDegreePrompt(0)));
    calc.modal_prompt = Some("DEGREE=?".to_string());
    let view = CalcStateView::from_state(&calc, vec![], vec![]);
    assert_eq!(view.display_str, "DEGREE=?");
}

// SEED? = 5 chars — no truncation
#[test]
fn seed_prompt_renders_verbatim() {
    let mut calc = CalcState::new();
    calc.modal_program = Some(ModalProgram::Stat1(Stat1Step::SeedPrompt));
    calc.modal_prompt = Some("SEED?".to_string());
    let view = CalcStateView::from_state(&calc, vec![], vec![]);
    assert_eq!(view.display_str, "SEED?");
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|-----------------|--------------|--------|
| `non-exhaustive patterns` CI break in hp41-gui for 26 Stat 1 Op variants | All 26 arms present; exhaustive; `just gui-ci` exits 0 | Phase 36 Plan 36-01 | 4-way invariant item 4 sealed |
| CATALOG 2 only lists MATH_1 (bit-0 only) | Lists both MATH_1 and STAT_1 when both bits set | Phase 36 Plan 36-01 | `STAT 1B` appears in user's CATALOG 2 |
| `?` overlay has 2 sections (built-in + Math 1) | 3 sections (built-in + Math 1 + Stat 1) | Phase 36 Plan 36-02 | All 26 Stat 1 mnemonics discoverable from GUI |
| `helpEntriesAll()` returns 2-pool (cv + math1) | Returns 3-pool (cv + math1 + stat1) | Phase 36 Plan 36-02 | Search covers Stat 1 entries |

**Deprecated/outdated:**
- `sectionButtons.length === 2` assertion in `HelpOverlay.test.tsx:226`: Must be updated to `=== 3` in Plan 36-03 (currently fails after Plan 36-02 lands the third section).

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `ΣCHISQD MODE?` is 13 chars (Σ + 12 ASCII) → truncated | Common Pitfalls, Code Examples | If char count is different, truncation test assertion is wrong; planner must count Unicode chars carefully: `"\u{03A3}CHISQD MODE?"`.chars().count() |

**All other claims were verified directly from the codebase.**

---

## Open Questions

1. **Exact char count of `ΣCHISQD MODE?` for truncation assertion**
   - What we know: `\u{03A3}` is 1 Unicode scalar, remaining 12 ASCII chars = 13 total. The LCD width is 12.
   - What's unclear: Whether the Rust `chars().count()` in `CalcStateView::from_state` counts `\u{03A3}` as 1 (Unicode scalars) or as multiple bytes (it should count as 1 per Rust's `chars()` semantics, but implementation may differ).
   - Recommendation: Planner runs `"\u{03A3}CHISQD MODE?".chars().count()` in a test or `cargo repl` snippet to confirm before writing the assertion. If it's 13, assert `"\u{03A3}CHISQD MOD\u{2261}"`. If it's somehow 12, assert verbatim.

2. **`xrom.module` value in `docs/hp41-stat1-functions.json`**
   - What we know: CONTEXT.md line 113 states the predicate is `e.xrom?.module === 'Stat 1'` and notes "the planner adjusts if JSON value differs."
   - What's unclear: The exact string value in the JSON file (must be confirmed at Plan 36-02 write time).
   - Recommendation: Planner reads `docs/hp41-stat1-functions.json` first entry's `xrom.module` value before writing the predicate.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable (MSRV 1.88) | `cargo test` | ✓ | Project declared | — |
| Node.js / npm | `npm ci`, `npm test` | ✓ | Project declared | — |
| `just` | Task runner | ✓ | Project declared | Direct `cargo`/`npm` |
| `hp41-gui` npm deps | `npm ci` in `just gui-ci` | Lockfile present | `package-lock.json` | — |

Step 2.6: No missing dependencies. This phase is a pure code/test edit with no new external tools.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Rust test framework | `cargo test` (built-in) |
| Frontend test framework | vitest (via `npm test` in `hp41-gui/`) |
| Config file | `hp41-gui/vite.config.ts` (vitest config embedded) |
| Quick Rust run | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` |
| Quick frontend run | `cd hp41-gui && npm test` |
| Full suite (GUI) | `just gui-ci` |
| Full suite (core) | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| STAT-GUI-01 | 26 `op_display_name` arms in GUI `prgm_display.rs`; no `_ =>` catch-all | Rust compile-time + file-text-scan | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml prgm_display_math1_arms` | ❌ Wave 0: extend `prgm_display_math1_arms.rs` OR create `prgm_display_stat1_arms.rs` |
| STAT-GUI-02 | CATALOG 2 lists STAT_1 alongside MATH_1 | Rust integration | `cargo test -p hp41-core catalog_2_lists_stat1` | ❌ Wave 0: add test in `hp41-core/tests/op_catalog_xrom.rs` |
| STAT-GUI-03 | `?` overlay shows "Stat 1 Pac (XROM 2)" section; search filters across Stat 1 | vitest component | `cd hp41-gui && npm test -- HelpOverlay` | ❌ Wave 0: extend `HelpOverlay.test.tsx` |
| STAT-GUI-04 | Modal prompts for 5 Stat1Step variants route through CalcStateView correctly | Rust integration | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml lcd_alternation_modal_prompt_stat1` | ❌ Wave 0: create `lcd_alternation_modal_prompt_stat1.rs` |
| STAT-GUI-05 | REASSIGNED to Phase 37 | — | — | N/A |

### Sampling Rate

- **Per task commit:** `cargo check -p hp41-gui` (Rust) + `cd hp41-gui && npx tsc --noEmit` (TypeScript)
- **Per wave merge:** `just gui-ci`
- **Phase gate:** `just gui-ci` green + `cargo test --workspace` green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `hp41-core/tests/op_catalog_xrom.rs` — add `catalog_2_lists_stat1_when_bit1_set` test (REQ: STAT-GUI-02)
- [ ] `hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt_stat1.rs` — new file; 5 test functions (REQ: STAT-GUI-04)
- [ ] `hp41-gui/src/HelpOverlay.test.tsx` — extend with Stat 1 section tests + update `sectionButtons.length` assertion from 2 → 3 (REQ: STAT-GUI-03)
- [ ] `hp41-gui/src-tauri/tests/prgm_display_math1_arms.rs` — optionally extend with `STAT1_VARIANT_IDS` array OR create a parallel `prgm_display_stat1_arms.rs` file (REQ: STAT-GUI-01 file-text-scan verification)

---

## Security Domain

Phase 36 introduces no new authentication, session management, access control, cryptography, or input validation surfaces. All changes are display-layer strings, JSON-read consumer code, and test additions. The existing Tauri permission model is unchanged.

**`security_enforcement` not explicitly set to `false` in config — but no ASVS categories apply to this phase's scope.**

---

## Sources

### Primary (HIGH confidence)
- `hp41-cli/src/prgm_display.rs:295–327` — 26 Stat 1 arm strings (source-of-truth for GUI copy)
- `hp41-core/src/ops/program.rs:336–365` — `op_catalog` function; bit-0 block is the direct template for bit-1 extension
- `hp41-core/src/ops/math1/xrom.rs:141–186` — `STAT_1` const; id=2, name="STAT 1B", 26 ops slice
- `hp41-core/src/ops/stat1/modal.rs:333–343` — `current_prompt()` returns 5 Stat1Step OM prompt strings
- `hp41-gui/src/help_data.ts:17–154` — Vite static-import pattern; `helpEntriesAll()` to extend
- `hp41-gui/src/HelpOverlay.tsx:30–132` — `SectionDef`, `SECTIONS`, `expanded` state, `toggleSection` — all extension targets
- `hp41-gui/src/HelpOverlay.test.tsx:200–245` — Phase 31-04 section test template; `length === 2` assertion to update
- `hp41-gui/src-tauri/tests/lcd_alternation_modal_prompt.rs` — template for new Stat 1 modal integration test
- `hp41-gui/src-tauri/tests/prgm_display_math1_arms.rs` — file-text-scan test pattern for prgm_display arms
- `hp41-gui/src-tauri/src/prgm_display.rs:43–46` — "Covers all 35 Op variants exhaustively" comment — must be updated to new count
- `hp41-core/tests/op_catalog_xrom.rs` — existing CATALOG 2 test structure; new test extends this file
- `.planning/phases/36-hp41-gui-gui-integration/36-CONTEXT.md` — full decision set for this phase

### Secondary (MEDIUM confidence)
- `hp41-gui/justfile:148–159` — `just gui-ci` recipe confirming the 5-step CI gate
- `hp41-gui/src/App.test.tsx:30–67` — `mockInvoke` pattern for dispatch_op end-to-end testing in vitest

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all existing; no new dependencies
- Architecture: HIGH — every change has a verified template in the codebase
- Pitfalls: HIGH — derived from direct code inspection and CONTEXT.md analysis
- Test patterns: HIGH — template files read and analyzed

**Research date:** 2026-05-24
**Valid until:** Indefinite — this is a codebase-bounded phase; the research is based on the current repository state, which changes only when code is committed.
