# Phase 49: Onboarding + GUI Keyboard Parity - Research

**Researched:** 2026-05-27
**Domain:** React/TypeScript GUI components + Tauri v2 Rust backend preferences
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-49.1:** Multi-step wizard with 5 panels and Next/Back navigation covering: Welcome/RPN basics, Four-Level Stack, SHIFT & Function Access, Keyboard Shortcuts, Programming & XEQ.
**D-49.2:** Static text and diagrams only — no live mini-calculator demo during the wizard.
**D-49.3:** Wizard renders as full-cover overlay (same pattern as HelpOverlay) with panel counter ("2 of 5") and Back/Next buttons.
**D-49.4:** First-run detection via `onboarding_done: bool` in `GuiPrefs` (`prefs.rs`). Field uses `#[serde(default)]`. Defaults to `false` → wizard shows. Set to `true` after user completes or dismisses.
**D-49.5:** Enrich the existing `?` overlay — no separate reference component. Add optional `example` and `notes` fields to the 5 function JSON files.
**D-49.6:** Entries with examples/notes become expandable — click to reveal detail. Entries without enrichment stay compact.
**D-49.7:** Enrich the top ~50 most-used functions in Phase 49.
**D-49.8:** "Show Quick Start" button lives in the SettingsPanel only (below "Theme" section). No button in the `?` overlay.
**D-49.9:** Clicking "Show Guide" closes settings panel and opens wizard overlay. Wizard always starts from panel 1 on re-open.
**D-49.10:** Dedicated collapsible "Keyboard Shortcuts" section at the top of the `?` overlay, above existing function sections. Two-column compact table.
**D-49.11:** Shortcut data sourced from a new `docs/keyboard-shortcuts.json` — single source of truth. Imported via Vite static JSON-import.
**D-49.12:** Add Ctrl+W/R/D/F (card reader) and F5 (R/S) to `resolveKeyId()` in App.tsx — mirroring CLI's `handle_key` exactly.
**D-49.13:** Add Ctrl+S for manual save — mirrors CLI's Ctrl+S. Uses a new `save_state` Tauri command.
**D-49.14:** KBD-04 audit: systematically compare CLI key bindings against GUI and document remaining gaps.

### Claude's Discretion

None — all decisions were made explicitly in the discuss phase.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ONBOARD-01 | First-run quick-start overlay introduces RPN basics, key layout, and how to access functions | D-49.1–D-49.4: wizard overlay using HelpOverlay pattern; `onboarding_done` prefs flag |
| ONBOARD-02 | Quick-start can be re-opened from help menu or `?` overlay | D-49.8–D-49.9: "Show Quick Start" in SettingsPanel; closes settings, opens wizard |
| ONBOARD-03 | Searchable in-app function reference with examples and usage notes | D-49.5–D-49.6: enrich `?` overlay with optional `example`/`notes` JSON fields, expandable rows |
| ONBOARD-04 | Function reference covers all 5 XROM modules + ~130 built-in functions (~350 entries) | Already in HelpOverlay; `helpEntriesAll()` covers all 5 pools |
| ONBOARD-05 | "Seen" flag stored in `prefs.json`, not CalcState | D-49.4: `onboarding_done: bool` in `GuiPrefs` with `#[serde(default)]` |
| KBD-01 | Card reader shortcuts (Ctrl+W/R/D/F) work in GUI physical keyboard | D-49.12: add `e.ctrlKey` branches in `resolveKeyId()` dispatching `xeq_WPRGM/RDPRGM/WDTA/RDTA` |
| KBD-02 | F5 triggers manual save in GUI | D-49.13: new `save_state` Tauri command; `resolveKeyId()` maps F5 to invoke it |
| KBD-03 | Physical keyboard shortcut reference displayed in `?` overlay | D-49.10–D-49.11: "Keyboard Shortcuts" collapsible section in HelpOverlay using `keyboard-shortcuts.json` |
| KBD-04 | All CLI key bindings have equivalent GUI keyboard paths (audit-verified) | D-49.14: compare `keys.rs` + `app.rs` against `resolveKeyId()`; document gaps in `keyboard-shortcuts.json` |

</phase_requirements>

---

## Summary

Phase 49 delivers four capabilities entirely within `hp41-gui/` — zero `hp41-core` changes. All work builds on strong, well-established Phase 48 patterns: the HelpOverlay overlay pattern for the wizard, the `GuiPrefs`/`prefs.rs` backend for the onboarding flag, the `SettingsPanel` for the re-open button, and the existing Vite static JSON-import pipeline for the new shortcut data file.

The keyboard parity work (KBD-01–KBD-04) is the most technically novel area: `resolveKeyId()` has never handled `e.ctrlKey` checks before. The pattern is straightforward but requires careful ordering — Ctrl+S must be intercepted before the normal key map, and Ctrl+W/R/D/F must dispatch `xeq_WPRGM` etc. via `invoke('dispatch_op', { keyId: 'xeq_WPRGM' })`. A new `save_state` Tauri command is needed for Ctrl+S and F5 (KBD-02); no equivalent exists today.

The function reference enrichment (ONBOARD-03) requires a JSON schema extension (`example?: string`, `notes?: string`) propagated through `HelpEntry` interface in `help_data.ts` and rendered as collapsible rows in `HelpOverlay.tsx`. The `keyboard-shortcuts.json` file (KBD-03/KBD-04) is new but follows the exact same Vite static JSON-import pattern as the five existing function JSON files.

**Primary recommendation:** Sequence as (1) Rust backend: add `onboarding_done` to `GuiPrefs` + new `save_state` command; (2) data files: create `keyboard-shortcuts.json`, enrich ~50 function entries; (3) React components: `OnboardingWizard`, HelpOverlay extensions, SettingsPanel addition, App.tsx keyboard handler additions.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Onboarding flag storage | Tauri backend (prefs.rs) | — | Preference state lives in `~/.hp41/prefs.json` per P59/THEME-05; never CalcState |
| Wizard UI (OnboardingWizard) | Frontend (React) | — | Static display only — no Rust logic needed |
| Wizard trigger (first-run) | Frontend (App.tsx) | Tauri backend (get_prefs) | Frontend reads `onboarding_done` from prefs on mount; decides to show wizard |
| Wizard re-open | Frontend (SettingsPanel.tsx) | — | Button calls `onShowOnboarding()` prop — no IPC |
| Function reference enrichment | Data files (JSON) | Frontend (HelpOverlay.tsx) | JSON is the single source of truth; frontend renders; Rust has no role |
| Keyboard shortcuts data | Data files (JSON) | Frontend (HelpOverlay.tsx + App.tsx) | Same Vite static JSON-import pipeline; shortcut data drives both display and audit |
| Ctrl+W/R/D/F card reader | Frontend (App.tsx) | Tauri backend (dispatch_op) | `resolveKeyId` catches Ctrl+key, dispatches `xeq_WPRGM` etc. via existing `dispatch_op` |
| F5 / Ctrl+S manual save | Frontend (App.tsx) | Tauri backend (new save_state cmd) | Frontend catches key, invokes new `save_state` command |
| Keyboard shortcut reference display | Frontend (HelpOverlay.tsx) | — | Static section above function sections |

---

## Standard Stack

### Core (all already in project)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React | 18 | Component rendering | Project standard — established in Phase 14 |
| TypeScript | ~5.x | Type safety | Project standard — all GUI code |
| Vite | ~5.x | Build + static JSON-import | Build-blocker import pattern for all 5 function JSONs |
| Tauri v2 | 2.11 | Rust ↔ React IPC | Project standard since Phase 14; permission TOML pattern |
| `serde_json` | (workspace) | Rust JSON serialization | Used by prefs.rs for GuiPrefs |
| `serde` | (workspace) | Rust derive macros | Used everywhere in Rust backend |
| Vitest + @testing-library/react | workspace | Frontend unit tests | Project standard; existing test suite |

[VERIFIED: codebase — all libraries confirmed present in hp41-gui/package.json and Cargo.toml]

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `@tauri-apps/api/core` | (workspace) | `invoke()` from TypeScript | All Tauri IPC calls; already imported in App.tsx |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Static JSON-import for shortcuts | Runtime fetch | Static import fails build on malformed JSON (hard-build-blocker) — deliberate choice |
| New `save_state` Tauri command | Reuse `set_pref` | `set_pref` only handles preferences; CalcState save needs access to AppState — separate command required |

**Installation:** No new packages — this phase adds zero new dependencies. [VERIFIED: codebase — project policy "Zero new runtime deps" since v3.0]

---

## Package Legitimacy Audit

No new packages are installed in this phase. The policy of zero new runtime dependencies (v3.0+) is maintained. All libraries used are already in the project and were verified in earlier phases.

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

---

## Architecture Patterns

### System Architecture Diagram

```
[Physical keyboard event]
        |
   App.tsx::handleKey
        |
  [new] ctrlKey check → Ctrl+W/R/D/F → invoke('dispatch_op', {keyId:'xeq_WPRGM'}) → Tauri → key_map.rs → Op::Xeq("WPRGM") → CardReader
                      → Ctrl+S / F5 → invoke('save_state') → [new] commands::save_state → persistence::save_state
        |
   resolveKeyId (existing) → named op → invoke('dispatch_op', ...)
        |
[App.tsx mount]
   invoke('get_prefs') → onboarding_done=false → setOnboardingOpen(true)
                       → onboarding_done=true  → no wizard

[? key press]
   HelpOverlay opens
        |
   [new] "Keyboard Shortcuts" section (static, from keyboard-shortcuts.json)
        |
   [modified] function rows: click → expand example/notes (if present)

[⚙ gear click]
   SettingsPanel opens
        |
   [new] "Quick Start" section → "Show Guide" button → close settings, open OnboardingWizard

[OnboardingWizard overlay]
   Panel 1..5 with Back/Next buttons
   Dismiss/Finish → invoke('set_pref', {key:'onboarding_done', value:'true'})
```

### Recommended Project Structure

```
hp41-gui/src/
├── OnboardingWizard.tsx    # NEW: 5-panel wizard overlay
├── OnboardingWizard.test.tsx # NEW: Vitest tests
├── HelpOverlay.tsx         # MODIFIED: add KBD section + expandable rows
├── HelpOverlay.test.tsx    # MODIFIED: add new section + expand tests
├── SettingsPanel.tsx       # MODIFIED: add Quick Start section
├── SettingsPanel.test.tsx  # MODIFIED: add onShowOnboarding test
├── help_data.ts            # MODIFIED: extend HelpEntry + add keyboard-shortcuts import
├── App.tsx                 # MODIFIED: onboarding state + Ctrl+key bindings + F5/Ctrl+S
├── App.css                 # MODIFIED: wizard overlay + expandable entry + shortcut section styles
docs/
├── keyboard-shortcuts.json  # NEW: physical key → op mapping (~30 entries)
├── hp41cv-functions.json    # MODIFIED: ~30 entries enriched with example/notes
├── hp41-math1-functions.json # MODIFIED: ~5-10 entries enriched
├── hp41-stat1-functions.json # MODIFIED: ~5 entries enriched
├── hp41-time-functions.json  # MODIFIED: ~5 entries enriched
├── hp41-advantage-functions.json # MODIFIED: ~5 entries enriched
hp41-gui/src-tauri/src/
├── prefs.rs                # MODIFIED: add onboarding_done field
├── commands.rs             # MODIFIED: add save_state command + set_pref onboarding_done key
├── lib.rs                  # MODIFIED: register save_state command
hp41-gui/src-tauri/permissions/
├── save-state.toml         # NEW: Tauri v2.11 permission for save_state
```

### Pattern 1: Extending `GuiPrefs` with `onboarding_done`

**What:** Add `onboarding_done: bool` to the existing `GuiPrefs` struct in `prefs.rs` with `#[serde(default)]` for backward compat.
**When to use:** Any time a new preference needs to persist across restarts.

```rust
// Source: hp41-gui/src-tauri/src/prefs.rs (existing pattern; Phase 49 extension)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GuiPrefs {
    #[serde(default = "default_theme")]
    pub theme: String,
    // Phase 49 — D-49.4: onboarding seen-flag (P59/ONBOARD-05: never in autosave.json)
    #[serde(default)]
    pub onboarding_done: bool,
}
```

[VERIFIED: codebase — prefs.rs line 12 documents this expansion path explicitly]

### Pattern 2: `set_pref` — adding `onboarding_done` key support

**What:** Extend the `set_pref` match arm in `commands.rs` to accept key `"onboarding_done"` with boolean value `"true"/"false"`.
**When to use:** Every new preference field needs a corresponding match arm in `set_pref`.

```rust
// Source: hp41-gui/src-tauri/src/commands.rs (existing set_pref pattern extended)
"onboarding_done" => {
    p.onboarding_done = value == "true";
}
```

Note: Tauri IPC passes all values as strings. Boolean encoding: `"true"` / `"false"`.

[VERIFIED: codebase — commands.rs set_pref match pattern lines 443–449]

### Pattern 3: New `save_state` Tauri command

**What:** A new command that locks AppState, clones CalcState, releases lock, then calls `persistence::save_state`. Follows the auto-save thread pattern but as a synchronous IPC response.
**When to use:** Ctrl+S and F5 in the GUI keyboard handler.

```rust
// Source: analog from lib.rs auto-save thread pattern (lines 83–96)
// and existing command structure in commands.rs
#[tauri::command]
pub fn save_state_cmd(state: State<'_, AppState>) -> Result<(), String> {
    // Clone under lock, release before I/O (CR-01 pattern from auto-save thread)
    let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let path = crate::persistence::default_state_path();
    crate::persistence::save_state(&path, &snapshot).map_err(|e| e.to_string())
}
```

Note: Name in Rust must avoid shadowing — use `save_state_cmd` in Rust, register as `save_state` in `tauri::generate_handler!`. [ASSUMED — Tauri v2 command naming conventions allow kebab/snake separation; verify no conflict with existing `save_state` in persistence.rs]

**Simpler alternative:** Use `save_state` as function name with a module path disambiguator since `persistence::save_state` is the only existing `save_state` symbol and it's not a `#[tauri::command]`.

```rust
#[tauri::command]
pub fn save_state(state: State<'_, AppState>) -> Result<(), String> {
    let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let path = persistence::default_state_path();
    persistence::save_state(&path, &snapshot).map_err(|e| e.to_string())
}
```

[VERIFIED: codebase — no existing `#[tauri::command]` named `save_state` in commands.rs]

### Pattern 4: Ctrl+key handling in `resolveKeyId`

**What:** `resolveKeyId` currently has no `e.ctrlKey` checks. New branch must be added BEFORE the existing key map, parallel to the CLI's Ctrl+W/R/D/F handling in `app.rs` lines 436–453.
**When to use:** Any new Ctrl+key binding in the GUI.

```typescript
// Source: analog from hp41-cli/src/app.rs lines 436-453 + App.tsx resolveKeyId pattern
function resolveKeyId(e: KeyboardEvent, state: CalcStateView | null): string | null {
    // [NEW] Ctrl+key bindings — mirror CLI handle_key Ctrl combos (D-49.12 / D-49.13)
    // MUST come before the letter-key map to prevent 'w'/'r'/'d'/'f'/'s' from double-firing.
    if (e.ctrlKey || e.metaKey) {
        switch (e.key.toLowerCase()) {
            case 'w': return 'xeq_WPRGM';   // Card: write program — CLI Op::Wprgm
            case 'r': return 'xeq_RDPRGM';  // Card: read program — CLI Op::Rdprgm
            case 'd': return 'xeq_WDTA';    // Card: write data — CLI Op::Wdta
            case 'f': return 'xeq_RDTA';    // Card: read data — CLI Op::Rdta
            case 's': return '__save_state__'; // Manual save — special key ID
        }
        return null; // Other Ctrl combos: Tab (handled elsewhere), C (not applicable)
    }

    // [NEW] F5 — manual save (KBD-02) — CLI: F5 runs program 'A'; GUI: Ctrl+S = save, F5 = save
    // NOTE: CLI F5 = run_program("A"), NOT save. D-49.13 assigns F5 = manual save in GUI.
    if (e.key === 'F5') return '__save_state__';

    // ... existing F7/F8 → sst/bst ...
```

**CRITICAL NOTE:** CLI F5 runs `run_program("A")` (R/S). The context decision D-49.12 says "F5 (R/S)" but the context also says D-49.13 is "Ctrl+S for manual save." Reading the requirements more carefully: KBD-02 says "F5 triggers manual save in GUI." This contradicts the CLI where F5 = R/S. The CONTEXT.md explicitly assigns F5 = manual save for the GUI (D-49.13 + KBD-02). Planner must confirm this divergence from CLI is intentional per D-49.14. [ASSUMED — the planner should treat F5=save as a deliberate GUI-only assignment per CONTEXT.md KBD-02]

The `__save_state__` special key ID must be intercepted in `handleKey` BEFORE `dispatchKeyId` (which routes to `invokeForKey` → `dispatch_op`). It invokes `save_state` directly.

[VERIFIED: codebase — App.tsx resolveKeyId line 109; key_map.rs xeq_ prefix → Op::Xeq handler lines 427–429]

### Pattern 5: OnboardingWizard overlay component

**What:** A multi-panel full-cover overlay following the HelpOverlay pattern exactly. Uses `useState` for current panel index. Esc + click-outside dismiss.
**When to use:** First-run + re-open from SettingsPanel.

```tsx
// Source: analog from HelpOverlay.tsx lines 100-253 + SettingsPanel.tsx click-outside pattern
interface OnboardingWizardProps {
    open: boolean;
    onClose: () => void;  // called on Dismiss or Finish; App.tsx marks onboarding_done
}

const PANELS = [
    { title: "Welcome to HP-41C", /* ... */ },
    { title: "The Four-Level Stack", /* ... */ },
    { title: "SHIFT & Function Access", /* ... */ },
    { title: "Keyboard Shortcuts", /* ... */ },
    { title: "Programming & XEQ", /* ... */ },
] as const;

export function OnboardingWizard({ open, onClose }: OnboardingWizardProps) {
    const [panelIdx, setPanelIdx] = useState(0);

    // Reset to panel 1 on each open (D-49.9)
    useEffect(() => {
        if (open) setPanelIdx(0);
    }, [open]);

    // Esc dismiss
    useEffect(() => {
        if (!open) return;
        const handler = (e: KeyboardEvent) => {
            if (e.key === 'Escape') { e.preventDefault(); onClose(); }
        };
        window.addEventListener('keydown', handler);
        return () => window.removeEventListener('keydown', handler);
    }, [open, onClose]);

    if (!open) return null;

    return (
        <div className="onboarding-overlay" role="dialog" aria-label="Quick Start Guide">
            <div className="onboarding-header">
                <span className="onboarding-counter">{panelIdx + 1} of {PANELS.length}</span>
                <button className="onboarding-close" onClick={onClose}>×</button>
            </div>
            <div className="onboarding-content">
                <h2>{PANELS[panelIdx].title}</h2>
                {/* panel content */}
            </div>
            <div className="onboarding-footer">
                {panelIdx > 0 && <button onClick={() => setPanelIdx(p => p - 1)}>Back</button>}
                {panelIdx < PANELS.length - 1
                    ? <button onClick={() => setPanelIdx(p => p + 1)}>Next</button>
                    : <button onClick={onClose}>Finish</button>
                }
            </div>
        </div>
    );
}
```

[VERIFIED: codebase — HelpOverlay.tsx full-cover overlay pattern, SettingsPanel.tsx click-outside dismiss]

### Pattern 6: HelpEntry interface extension + expandable rows

**What:** Add optional `example?: string` and `notes?: string` to the `HelpEntry` interface in `help_data.ts`. In `HelpOverlay.tsx`, entries with either field get a click-to-expand row.
**When to use:** ONBOARD-03 enrichment.

```typescript
// Source: help_data.ts HelpEntry interface (lines 54-76)
export interface HelpEntry {
    // ... existing fields ...
    // Phase 49 D-49.5 — optional enrichment fields for expandable rows
    example?: string;    // e.g. "3 ENTER 4 + → 7"
    notes?: string;      // e.g. "Stack lift enabled; T register is lost"
}
```

In `HelpOverlay.tsx`, replace the current row render with:

```tsx
// Source: analog from HelpOverlay.tsx lines 233-238 (row render)
{entries.map(entry => {
    const hasDetail = Boolean(entry.example || entry.notes);
    return (
        <ExpandableHelpRow key={entry.op_variant} entry={entry} hasDetail={hasDetail} />
    );
})}
```

The `ExpandableHelpRow` is a small sub-component (or inline with `useState`) that toggles a detail panel below the main row on click. [VERIFIED: codebase — HelpOverlay.tsx row render pattern lines 233-238]

### Pattern 7: `keyboard-shortcuts.json` — data file and import

**What:** New data file `docs/keyboard-shortcuts.json` with ~30 entries. Imported via Vite static JSON-import in `help_data.ts` alongside the 5 existing function JSONs.

```json
// docs/keyboard-shortcuts.json schema
[
    { "key": "Enter", "op": "ENTER", "description": "Push X onto stack; terminates number entry" },
    { "key": "Backspace", "op": "CLX", "description": "Clear X register / delete last digit" },
    { "key": "Tab", "op": "SHIFT", "description": "Toggle one-shot SHIFT prefix (f key)" },
    { "key": "+", "op": "+", "description": "Add Y + X, drop stack" },
    { "key": "Ctrl+W", "op": "WPRGM", "description": "Card reader: write program to file" },
    { "key": "Ctrl+R", "op": "RDPRGM", "description": "Card reader: read program from file" },
    { "key": "Ctrl+D", "op": "WDTA", "description": "Card reader: write data registers to file" },
    { "key": "Ctrl+F", "op": "RDTA", "description": "Card reader: read data registers from file" },
    { "key": "Ctrl+S / F5", "op": "SAVE", "description": "Manual save to ~/.hp41/autosave.json" },
    ...
]
```

```typescript
// Source: help_data.ts Vite static import pattern (lines 28-32)
import keyboardShortcuts from '../../docs/keyboard-shortcuts.json';

export interface KeyboardShortcut {
    key: string;
    op: string;
    description: string;
}

export function getKeyboardShortcuts(): readonly KeyboardShortcut[] {
    return keyboardShortcuts as readonly KeyboardShortcut[];
}
```

[VERIFIED: codebase — help_data.ts Vite static import lines 28-32; build-blocker semantics]

### Pattern 8: "Keyboard Shortcuts" section in HelpOverlay

**What:** A new collapsible section added FIRST in the `SECTIONS` array (above 'hp41cv'). Uses a separate state key `'kbd'` in the `expanded` map.
**When to use:** KBD-03.

The `SectionDef` union type must be widened to include `'kbd'`:
```typescript
// Source: HelpOverlay.tsx lines 50-54 (SectionDef interface + id union)
interface SectionDef {
    id: 'kbd' | 'hp41cv' | 'math1' | 'stat1' | 'time' | 'adv22' | 'adv24';
    // ...
}
```

The `kbd` section renders a two-column shortcut table instead of the `HelpEntry` row format. [VERIFIED: codebase — HelpOverlay.tsx SectionDef pattern lines 50-98]

### Pattern 9: SettingsPanel "Quick Start" section

**What:** New `<section>` below the existing Theme section, with a "Show Guide" button that calls a new `onShowOnboarding` prop.
**When to use:** ONBOARD-02 re-open access point.

```tsx
// Source: SettingsPanel.tsx existing section pattern lines 51-65
export type SettingsPanelProps = {
    open: boolean;
    onClose: () => void;
    currentTheme: string;
    onThemeChange: (theme: string) => void;
    onShowOnboarding: () => void;  // NEW: called when "Show Guide" clicked
};

// In render (after Theme section):
<section className="settings-section">
    <h3 className="settings-section-heading">Quick Start</h3>
    <button
        className="settings-action-btn"
        onClick={() => { props.onClose(); props.onShowOnboarding(); }}
    >
        Show Guide
    </button>
</section>
```

[VERIFIED: codebase — SettingsPanel.tsx line 66 placeholder comment]

### Anti-Patterns to Avoid

- **Onboarding flag in CalcState:** The `onboarding_done` field MUST live in `GuiPrefs` / `prefs.json`, never in `CalcState` / `autosave.json` (P59/ONBOARD-05).
- **Ctrl+key without `e.ctrlKey || e.metaKey` check:** macOS users need `e.metaKey` (Cmd key) as an alternative for some combos. For card reader shortcuts, `e.ctrlKey` only is standard; Ctrl+S should also check `e.metaKey` on macOS. [ASSUMED — standard Web convention; verify with user if macOS Cmd+S is desired]
- **F5 default browser behavior:** `e.preventDefault()` must be called when intercepting F5 (some browsers try to reload). Already done in the existing `e.preventDefault()` call at line 673 in handleKey, but Ctrl+S also triggers browser save-page — must be prevented.
- **Treating `__save_state__` as a dispatch_op key:** The special `__save_state__` id returned by `resolveKeyId` must be intercepted in `handleKey` BEFORE `dispatchKeyId` (which routes to `invokeForKey` → `dispatch_op`). The backend `key_map.rs` will error on unknown `__save_state__` key — never let it through.
- **Forgetting to widen the `expanded` state type in HelpOverlay:** Adding a new `'kbd'` section requires widening the `expanded` state object type AND the `useState` initial value AND the `useEffect` reset AND the `toggleSection` parameter union. Missing any one causes a TypeScript compile error.
- **CLI F5 = run_program("A") vs GUI F5 = save:** This is a deliberate GUI-CLI divergence per CONTEXT.md D-49.12/KBD-02. Do not make GUI F5 run a program — that's the CLI assignment. GUI keyboard has R/S on the on-screen keyboard already.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Collapsible sections in HelpOverlay | Custom accordion library | Existing `toggleSection` + `expanded` state pattern | Already in HelpOverlay; 10 lines to extend |
| Wizard step management | State machine library | Plain `useState(panelIdx)` | 5 static panels; no complex transitions |
| Keyboard shortcut data | Hardcode in TypeScript | `keyboard-shortcuts.json` + Vite static import | Same SoT pattern as all 5 function JSONs; enables CI audit |
| Preference persistence | localStorage / IndexedDB | Existing `prefs.rs` + `save_prefs` / `set_pref` | Already wired through Tauri; `~/.hp41/prefs.json` is the standard |
| Save-to-disk | Custom FS code | `persistence::save_state` (existing) | Handles dir creation, pretty-print JSON, version wrapper |

**Key insight:** Every new capability in Phase 49 has an exact analog already working in the codebase. No genuinely novel patterns are required.

---

## Common Pitfalls

### Pitfall 1: `save_state` command name collision
**What goes wrong:** `persistence::save_state` and a new `#[tauri::command] pub fn save_state` would create a shadowing/collision in `commands.rs`.
**Why it happens:** Both are in scope inside `commands.rs` after `use crate::persistence`.
**How to avoid:** Either name the command `save_state_now` and register it as-is in `generate_handler!`, OR use `persistence::save_state(...)` call inside a `save_state` command function — the module-qualified call resolves the ambiguity cleanly. Use the unambiguous approach with `persistence::save_state(&path, &snapshot)`.
**Warning signs:** `error[E0201]: duplicate definitions with name save_state` at compile time.

### Pitfall 2: Ctrl+key interception not preventing browser default
**What goes wrong:** Ctrl+S triggers the browser's "save page" dialog in Tauri WebView. F5 triggers page reload in some configurations.
**Why it happens:** Tauri's WebView passes keyboard events to the browser engine by default.
**How to avoid:** Call `e.preventDefault()` in `handleKey` when Ctrl+S or F5 is detected, BEFORE any other processing. The existing pattern already calls `e.preventDefault()` at line 673 for matched keys — ensure the new Ctrl+key checks also reach that path.
**Warning signs:** Browser save dialog appears on Ctrl+S; app reloads on F5.

### Pitfall 3: `set_pref` with boolean value encoding
**What goes wrong:** Tauri IPC serializes all primitive params as strings. `invoke('set_pref', { key: 'onboarding_done', value: true })` passes the boolean `true` but the Rust `set_pref(value: String)` receives `"true"`.
**Why it happens:** TypeScript `true` serialized to JSON string for IPC; Rust sees `String`.
**How to avoid:** Always call `invoke('set_pref', { key: 'onboarding_done', value: 'true' })` (explicit string). The Rust side parses `value == "true"`. Add a unit test that verifies round-trip.
**Warning signs:** `onboarding_done` never becoming `true` after wizard completion.

### Pitfall 4: HelpOverlay `expanded` state missing `'kbd'` key
**What goes wrong:** TypeScript type error on `toggleSection('kbd')` or `expanded['kbd']`.
**Why it happens:** The `expanded` state object has a literal type `{ hp41cv: boolean; math1: boolean; ... }` — adding `'kbd'` to `SectionDef.id` doesn't automatically widen the state type.
**How to avoid:** Update the `expanded` state type and initial value in the same commit as `SectionDef` widening. The `setExpanded` in `useEffect` reset and `toggleSection` function signature must all be updated together.
**Warning signs:** TypeScript compilation errors on the `expanded` state usage.

### Pitfall 5: Wizard overlay z-index conflict
**What goes wrong:** Wizard overlay appears behind the HelpOverlay or SettingsPanel.
**Why it happens:** Existing z-index stack: toast=50, help-overlay=60, settings-panel=70. Wizard needs a higher z-index or at minimum equal to HelpOverlay.
**How to avoid:** Assign `z-index: 65` to `.onboarding-overlay` (between help and settings). Since wizard and help overlay are mutually exclusive (opening one closes the other via App.tsx state), z-index 65 is safe.
**Warning signs:** Wizard content visible through SettingsPanel panel.

### Pitfall 6: KBD-04 audit completeness
**What goes wrong:** Missing some CLI bindings in the audit, resulting in gaps not documented in `keyboard-shortcuts.json`.
**Why it happens:** CLI `handle_key` has bindings scattered across multiple locations: explicit Ctrl+key checks (lines 318–453), F-key checks (lines 520–527, 768), modal-opener uppercase keys (S/R/F/P/X), and `keys::key_to_op()` lowercase map.
**How to avoid:** Audit in order: (1) CLI Ctrl+key combos at app.rs lines 318–453, (2) CLI F-key combos at app.rs lines 520–527/768, (3) CLI modal-openers (S/R/F/P/X uppercase), (4) CLI `keys::key_to_op()` letter map (app.rs lines 780+), (5) GUI `resolveKeyId()` MAP object. Document every CLI binding and its GUI equivalent (or "no GUI equivalent" gap).
**Warning signs:** `keyboard-shortcuts.json` missing entries that appear in CLI but not GUI.

### Pitfall 7: Tauri permission TOML for `save_state`
**What goes wrong:** `save_state` command works in dev but silently fails in production build due to missing permission.
**Why it happens:** Tauri v2.11 requires a TOML in `permissions/save-state.toml` AND a reference in `capabilities/default.json`.
**How to avoid:** Run `cargo check` first to regenerate the schema registry, then create `save-state.toml` using the same pattern as `tick-time.toml`, then immediately add `"allow-save-state"` to `capabilities/default.json`. These are an atomic pair.
**Warning signs:** `tauri::Error: Command save_state not found` in production logs.

---

## Code Examples

### Full `resolveKeyId` Ctrl+key insertion point

```typescript
// Source: App.tsx lines 109-162 (resolveKeyId) — insert at top, before F7/F8 checks
function resolveKeyId(e: KeyboardEvent, state: CalcStateView | null): string | null {
    // [Phase 49 D-49.12/D-49.13] — Ctrl+key / Cmd+key bindings
    // Must be FIRST: prevents Ctrl+W/R/D/F from reaching the letter-key MAP below.
    // e.ctrlKey: Windows/Linux. e.metaKey: macOS Cmd key (for Ctrl+S / save).
    if (e.ctrlKey) {
        switch (e.key.toLowerCase()) {
            case 'w': return 'xeq_WPRGM';
            case 'r': return 'xeq_RDPRGM';
            case 'd': return 'xeq_WDTA';
            case 'f': return 'xeq_RDTA';
            case 's': return '__save_state__';
        }
        return null;  // Unknown Ctrl combo — do not fall through to letter map
    }
    // [Phase 49 KBD-02] F5 — manual save (GUI-only assignment; CLI F5 = run_program)
    if (e.key === 'F5') return '__save_state__';

    // Existing: F7/F8 → SST/BST ...
```

### `handleKey` save dispatch insertion point

```typescript
// Source: App.tsx handleKey function — insert after Esc/Tab handling, before resolveKeyId call
// Intercept __save_state__ BEFORE dispatchKeyId (which routes to dispatch_op)
// This prevents the special id from leaking into the Rust key_map resolver.
if (keyId === '__save_state__') {
    e.preventDefault();
    invoke<void>('save_state')
        .then(() => showToast('Saved'))
        .catch(err => showToast(`Save failed: ${extractErrMessage(err)}`));
    return;
}
```

### `save_state` Tauri command

```rust
// Source: pattern from lib.rs auto-save thread (lines 83-96) + commands.rs command structure
/// Tauri command: manual save to ~/.hp41/autosave.json (Phase 49 KBD-02 / Ctrl+S / F5).
///
/// Mirrors the CLI's Ctrl+S handler (hp41-cli/src/app.rs lines 324-330).
/// Clone-under-lock pattern: AppState mutex released before disk I/O (CR-01).
/// Returns Ok(()) on success, Err(String) for toast display on failure.
///
/// P59/THEME-05: does not touch GuiPrefs or prefs.json.
#[tauri::command]
pub fn save_state(state: State<'_, AppState>) -> Result<(), String> {
    let snapshot = state.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let path = persistence::default_state_path();
    persistence::save_state(&path, &snapshot).map_err(|e| e.to_string())
}
```

### App.tsx onboarding state wiring

```typescript
// Source: App.tsx pattern for helpOpen/settingsOpen (lines 237-241)
// Add alongside existing overlay state
const [onboardingOpen, setOnboardingOpen] = useState(false);

// On mount: read prefs to check onboarding_done (extend existing get_prefs useEffect)
useEffect(() => {
    invoke<{ theme: string; onboarding_done: boolean }>('get_prefs')
        .then(prefs => {
            setTheme(prefs.theme);
            document.body.dataset.theme = prefs.theme;
            if (!prefs.onboarding_done) {
                setOnboardingOpen(true);
            }
        })
        .catch(() => {
            document.body.dataset.theme = 'dark';
            setOnboardingOpen(true);  // First run fallback: show wizard
        });
}, []);

// onClose handler for OnboardingWizard
const handleOnboardingClose = useCallback(() => {
    setOnboardingOpen(false);
    // Fire-and-forget: mark as seen
    invoke('set_pref', { key: 'onboarding_done', value: 'true' }).catch(() => {});
}, []);
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hardcoded key bindings only | + `keyboard-shortcuts.json` as SoT | Phase 49 | CI audit possible; single place to update docs |
| Function descriptions only | + expandable `example`/`notes` | Phase 49 | Power users get context without cluttering compact list |
| No first-run guidance | 5-panel wizard | Phase 49 | New users can start immediately |

**Deprecated/outdated:**
- Nothing deprecated in this phase — purely additive.

---

## KBD-04 Audit: CLI vs GUI Keyboard Parity

Based on reading `hp41-cli/src/app.rs` and `hp41-gui/src/App.tsx`:

| CLI Binding | CLI Action | GUI Equivalent | Status |
|-------------|-----------|----------------|--------|
| `Ctrl+C` | Quit app | N/A (window close) | Acceptable gap — OS handles |
| `Ctrl+S` | Manual save | `Ctrl+S` → `save_state` | **NEW in Phase 49** |
| `Ctrl+P` | Program library overlay | No GUI equivalent | Gap — CLI-only feature (no PRGM library overlay in GUI) |
| `Ctrl+W` | WPRGM (card write program) | `Ctrl+W` → `xeq_WPRGM` | **NEW in Phase 49** |
| `Ctrl+R` | RDPRGM (card read program) | `Ctrl+R` → `xeq_RDPRGM` | **NEW in Phase 49** |
| `Ctrl+D` | WDTA (card write data) | `Ctrl+D` → `xeq_WDTA` | **NEW in Phase 49** |
| `Ctrl+F` | RDTA (card read data) | `Ctrl+F` → `xeq_RDTA` | **NEW in Phase 49** |
| `Ctrl+A` | USER key assign modal | No GUI equivalent yet | Gap — ASN is on-screen keyboard only |
| `F5` | run_program("A") | `F5` → `save_state` (GUI divergence per KBD-02) | **NEW in Phase 49 (GUI-only)** |
| `F7` | SST | `F7` → `sst` | Already in GUI |
| `F8` | BST | `F8` → `bst` | Already in GUI |
| `F1–F4` (USER mode) | Run assigned a/b/c/d | No GUI equivalent yet | Gap — accepted (USER mode assignments exist on-screen) |
| `S` (uppercase) | STO register modal | `sto_prompt` on-screen key | Parity via on-screen keyboard |
| `R` (uppercase) | RCL register modal | `rcl_prompt` on-screen key | Parity via on-screen keyboard |
| `Tab` | SHIFT toggle | `Tab` → SHIFT | Already in GUI |
| `?` | Help overlay | `?` → help | Already in GUI |

**Gaps documented but acceptable (no on-screen keyboard equivalent):**
- `Ctrl+P` (program library overlay): CLI-only TUI feature
- `Ctrl+A` (USER key assign): on-screen keyboard provides `asn` modal
- `F1–F4` (USER mode shortcuts): future enhancement (v4.1+)

[VERIFIED: codebase — CLI app.rs lines 318–453 for Ctrl+key; lines 520–527 for F1–F4; lines 768 for F5; App.tsx resolveKeyId lines 109–162]

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | macOS users may expect Cmd+S for save (not just Ctrl+S) | Pattern 4 — Ctrl+key handling | macOS keyboard shortcut feels broken; add `e.metaKey` check alongside `e.ctrlKey` for save |
| A2 | F5 = manual save (GUI) is a deliberate divergence from CLI F5 = run_program | KBD-04 Audit / Pattern 4 | Planner should explicitly confirm this assignment in the plan comments |
| A3 | `save_state` as the Rust function name does not shadow or conflict with `persistence::save_state` inside commands.rs | Pattern 3 | Rust compiler will error on ambiguity; use fully-qualified `persistence::save_state(...)` call site to resolve |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

---

## Open Questions

1. **Ctrl+S vs Cmd+S on macOS**
   - What we know: Web standard is `e.ctrlKey` for Windows/Linux, `e.metaKey` for macOS Cmd key.
   - What's unclear: Should Cmd+S also trigger save on macOS? Ctrl+W/R/D/F can stay Ctrl-only (they're obscure shortcuts).
   - Recommendation: Add `(e.ctrlKey || e.metaKey)` check only for Ctrl+S; keep Ctrl-only for Ctrl+W/R/D/F.

2. **F5 divergence from CLI**
   - What we know: CONTEXT.md D-49.13 + KBD-02 assign F5 = manual save in GUI. CLI F5 = run_program("A").
   - What's unclear: Is this intentional divergence or a mistake in requirements?
   - Recommendation: CONTEXT.md is authoritative — implement F5=save, add a comment in the code noting the CLI divergence.

---

## Environment Availability

Step 2.6: VERIFIED (no new external dependencies identified — this phase is purely code/config/data changes within the existing toolchain).

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js / npm | Vite build, vitest | ✓ | (workspace) | — |
| Rust / cargo | Tauri backend | ✓ | MSRV 1.88 | — |
| just | Task runner | ✓ | (workspace) | — |
| Tauri v2.11 | Permission TOMLs | ✓ | 2.11 | — |

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Vitest + @testing-library/react |
| Config file | `hp41-gui/vite.config.ts` (test section) |
| Quick run command | `cd hp41-gui && npm test` (= `vitest run`) |
| Full suite command | `just gui-ci` (tsc + vitest + cargo test + cargo build) |
| Rust backend tests | `cargo test --manifest-path hp41-gui/src-tauri/Cargo.toml` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ONBOARD-01 | OnboardingWizard renders when open=true; shows panel 1 | unit | `cd hp41-gui && npm test` | ❌ Wave 0: `OnboardingWizard.test.tsx` |
| ONBOARD-01 | Panel counter shows "1 of 5" | unit | `cd hp41-gui && npm test` | ❌ Wave 0 |
| ONBOARD-01 | Esc closes wizard | unit | `cd hp41-gui && npm test` | ❌ Wave 0 |
| ONBOARD-02 | SettingsPanel renders "Show Guide" button | unit | `cd hp41-gui && npm test` | ❌ Wave 0: extend `SettingsPanel.test.tsx` |
| ONBOARD-02 | Clicking "Show Guide" calls onShowOnboarding | unit | `cd hp41-gui && npm test` | ❌ Wave 0 |
| ONBOARD-03 | HelpEntry with example/notes renders expand toggle | unit | `cd hp41-gui && npm test` | ❌ Wave 0: extend `HelpOverlay.test.tsx` |
| ONBOARD-04 | helpEntriesAll() still returns ≥350 entries | unit (existing) | `cd hp41-gui && npm test` | ✅ `HelpOverlay.test.tsx` (update count) |
| ONBOARD-05 | prefs.rs: onboarding_done roundtrip | unit | `cargo test -p hp41-gui-tauri` | ❌ Wave 0: extend `prefs.rs` tests |
| ONBOARD-05 | prefs.json without onboarding_done field loads without error (serde default) | unit | `cargo test` | ❌ Wave 0: extend `prefs.rs` tests |
| KBD-01 | resolveKeyId returns 'xeq_WPRGM' for Ctrl+W | unit | `cd hp41-gui && npm test` | ❌ Wave 0: extend `App.test.tsx` |
| KBD-02 | resolveKeyId returns '__save_state__' for F5 | unit | `cd hp41-gui && npm test` | ❌ Wave 0: extend `App.test.tsx` |
| KBD-02 | save_state Tauri command persists state | unit | `cargo test` | ❌ Wave 0: extend `commands.rs` tests |
| KBD-03 | HelpOverlay renders "Keyboard Shortcuts" section button | unit | `cd hp41-gui && npm test` | ❌ Wave 0: extend `HelpOverlay.test.tsx` |
| KBD-04 | keyboard-shortcuts.json parseable; contains ≥20 entries | unit | `cd hp41-gui && npm test` | ❌ Wave 0: extend `HelpOverlay.test.tsx` |

### Sampling Rate
- **Per task commit:** `cd hp41-gui && npm test` (Vitest, < 15s)
- **Per wave merge:** `just gui-ci` (full TypeScript typecheck + Vitest + Rust tests + build)
- **Phase gate:** Full `just gui-ci` green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `hp41-gui/src/OnboardingWizard.test.tsx` — covers ONBOARD-01 (wizard open/close/panels)
- [ ] Extend `hp41-gui/src/SettingsPanel.test.tsx` — covers ONBOARD-02 (Show Guide button)
- [ ] Extend `hp41-gui/src/HelpOverlay.test.tsx` — covers ONBOARD-03, KBD-03, KBD-04 (new sections, expandable rows, shortcut count)
- [ ] Extend `hp41-gui/src/App.test.tsx` — covers KBD-01, KBD-02 (resolveKeyId Ctrl+key returns)
- [ ] Extend `hp41-gui/src-tauri/src/prefs.rs` tests — covers ONBOARD-05 (onboarding_done roundtrip + serde default)
- [ ] Extend `hp41-gui/src-tauri/src/commands.rs` tests — covers KBD-02 (save_state command + set_pref onboarding_done key)

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes (set_pref) | Existing VALID_THEMES whitelist pattern; extend with boolean parse for `onboarding_done` |
| V6 Cryptography | no | — |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malformed `keyboard-shortcuts.json` | Tampering | Vite static JSON-import → build error (cannot occur at runtime) |
| `set_pref` with unexpected `onboarding_done` value | Tampering | Parse `value == "true"` → boolean coercion; any other value → `false` |
| `save_state` command invoked rapidly (disk I/O flood) | DoS | Acceptable — user-initiated only (Ctrl+S). No programmatic call path. |

The existing T-48-01 threat mitigation in `set_pref` (unknown key returns `Err(String)`) already covers the new `onboarding_done` key by extension of the match arm. [VERIFIED: codebase — commands.rs set_pref lines 440–451]

---

## Sources

### Primary (HIGH confidence)
- Codebase: `hp41-gui/src/App.tsx` — resolveKeyId pattern, handleKey Ctrl/F-key structure
- Codebase: `hp41-gui/src/HelpOverlay.tsx` — full-cover overlay + collapsible sections pattern
- Codebase: `hp41-gui/src/help_data.ts` — Vite static JSON-import + HelpEntry interface
- Codebase: `hp41-gui/src/SettingsPanel.tsx` — settings section pattern + Phase 49 placeholder
- Codebase: `hp41-gui/src-tauri/src/prefs.rs` — GuiPrefs struct + Phase 49 expansion note
- Codebase: `hp41-gui/src-tauri/src/commands.rs` — command patterns, set_pref match structure
- Codebase: `hp41-gui/src-tauri/src/persistence.rs` — save_state pattern for new command
- Codebase: `hp41-gui/src-tauri/src/lib.rs` — auto-save thread pattern, command registration
- Codebase: `hp41-cli/src/app.rs` lines 318–453, 768 — Ctrl+key and F5 CLI bindings (KBD-04 audit)
- Codebase: `hp41-gui/src-tauri/src/key_map.rs` lines 427–429 — `xeq_` prefix → `Op::Xeq` routing
- Codebase: `.planning/phases/48-gui-infrastructure-theming/48-PATTERNS.md` — Phase 48 pattern map
- Codebase: `.planning/phases/49-onboarding-gui-keyboard-parity/49-CONTEXT.md` — locked decisions

### Secondary (MEDIUM confidence)
- CONTEXT.md decisions D-49.1 through D-49.14 — all locked, no alternatives researched

### Tertiary (LOW confidence)
- None

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in project, verified in codebase
- Architecture: HIGH — all patterns have exact analogs in existing Phase 48 code
- Pitfalls: HIGH — most discovered from direct code reading; A1/A2/A3 assumptions flagged
- KBD-04 audit: HIGH — CLI app.rs and GUI App.tsx both read directly

**Research date:** 2026-05-27
**Valid until:** 2026-06-27 (30 days — stable Tauri v2 + React 18 stack)
