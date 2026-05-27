# Architecture Research

**Domain:** HP-41 Calculator Emulator v4.0 Platform Maturity (themes, onboarding, .raw I/O, X-MEM)
**Researched:** 2026-05-27
**Confidence:** HIGH (all claims verified against source code)

---

## Standard Architecture

### System Overview

```
┌──────────────────────────────────────────────────────────────────────────┐
│                        hp41-gui (Tauri v2 + React)                        │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────┐  ┌─────────────┐ │
│  │ App.tsx     │  │ Keyboard.tsx │  │ HelpOverlay.tsx│  │ ThemePicker │ │
│  │ (root state)│  │ (KEY_DEFS)   │  │ (6 sections)   │  │ (NEW)       │ │
│  └──────┬──────┘  └──────┬───────┘  └────────┬───────┘  └──────┬──────┘ │
│         │                │                   │                  │        │
│  ┌──────▼──────────────────────────────────────────────────────▼──────┐  │
│  │     Tauri IPC: dispatch_op / get_state / import_raw / export_raw   │  │
│  │                get_prefs / set_pref                                 │  │
│  └──────────────────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────────────┐    │
│  │  hp41-gui/src-tauri: commands.rs | key_map.rs | types.rs         │    │
│  │  persistence.rs | prgm_display.rs | cards.rs | prefs.rs (NEW)    │    │
│  └──────────────────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────────────────┘
                           │  hp41-core (lib)
┌──────────────────────────────────────────────────────────────────────────┐
│  CalcState  |  Op enum  |  dispatch()  |  run_program()                   │
│  cardreader/raw.rs (encode_program / decode_program — ALREADY EXISTS)     │
│  cardreader/data.rs  |  state.rs (migrate_after_load)                     │
│  ops/xmem.rs (NEW: ExtendedMemory model + EMDIR/EMROOM/EMREG ops)         │
└──────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│  hp41-cli (ratatui TUI)                                                   │
│  app.rs | keys.rs | ui.rs | help_data.rs | persistence.rs | cards.rs     │
│  (keyboard parity: add physical key bindings to close GUI gaps)           │
└──────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│  Persistence layer (shared + new)                                         │
│  ~/.hp41/autosave.json  (CalcState: shared CLI + GUI, unchanged)          │
│  ~/.hp41/prefs.json     (NEW: GuiPrefs { theme, onboarding_done })        │
└──────────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| `hp41-core/src/cardreader/raw.rs` | `.raw` byte codec (encode/decode) — **already exists, no change** | `cardreader/mod.rs`; frontends via CardOpRequest drain |
| `hp41-core/src/ops/xmem.rs` (NEW) | Extended Memory ops: EMDIR, EMROOM, EMREG | `CalcState.xmem: ExtendedMemory`; `dispatch()` |
| `hp41-core/src/state.rs` | Add `xmem: ExtendedMemory` field with `#[serde(default)]` | All ops via `&mut CalcState` |
| `hp41-gui/src-tauri/src/commands.rs` | Add `import_raw`, `export_raw`, `get_prefs`, `set_pref` Tauri commands | hp41-core cardreader codec; `prefs.rs` |
| `hp41-gui/src-tauri/src/prefs.rs` (NEW) | Load/save `~/.hp41/prefs.json`; `GuiPrefs { theme, onboarding_done }` | `commands.rs`; `lib.rs` setup |
| `hp41-gui/src/App.tsx` | Consume theme from prefs; apply `data-theme` to root div; onboarding first-run gate | ThemePicker, OnboardingOverlay |
| `hp41-gui/src/ThemePicker.tsx` (NEW) | Dropdown / radio for 3-4 theme presets; calls `set_pref('theme', …)` | `App.tsx` |
| `hp41-gui/src/OnboardingOverlay.tsx` (NEW) | Full-screen first-run guide (3-5 slides); marks `onboarding_done=true` | `App.tsx` |
| `hp41-gui/src/FunctionReference.tsx` (NEW) | Searchable full function reference (richer than `?` overlay) | `help_data.ts` (existing 5-pool) |
| `hp41-gui/src/App.css` | Extract hard-coded hex colors to CSS custom properties; add 4 `[data-theme]` blocks | All styled components |
| `hp41-cli/src/keys.rs` | Add physical keyboard bindings to close parity gaps with GUI | `app.rs` handle_key() |

---

## Recommended Project Structure (new and changed files only)

```
hp41-core/src/
├── cardreader/
│   └── raw.rs              EXISTING, no change needed for file I/O
│                           (encode_program / decode_program already work + tested)
├── ops/
│   ├── mod.rs              MODIFY: add X-MEM Op variants
│   ├── program.rs          MODIFY: execute_op() arms for X-MEM ops
│   └── xmem.rs             NEW: EMDIR / EMROOM / EMREG ops + ExtendedMemory struct
└── state.rs                MODIFY: add xmem: ExtendedMemory field

hp41-gui/src-tauri/src/
├── commands.rs             MODIFY: add import_raw, export_raw, get_prefs, set_pref
├── key_map.rs              MODIFY: resolve() arms for X-MEM key IDs
├── lib.rs                  MODIFY: manage Mutex<GuiPrefs>; load prefs at startup
├── prgm_display.rs         MODIFY: op_display_name() arms for X-MEM ops (4-way arm 4)
├── prefs.rs                NEW: GuiPrefs struct + load/save to ~/.hp41/prefs.json
└── permissions/
    ├── get-prefs.toml      NEW: Tauri v2.11 permission for get_prefs command
    ├── set-pref.toml       NEW: Tauri v2.11 permission for set_pref command
    ├── import-raw.toml     NEW: Tauri v2.11 permission for import_raw command
    └── export-raw.toml     NEW: Tauri v2.11 permission for export_raw command

hp41-gui/src/
├── App.tsx                 MODIFY: theme data-attr init; onboarding gate; Import/Export UI
├── App.css                 MODIFY: lift hex to CSS vars; add 4 [data-theme] blocks
├── FunctionReference.tsx   NEW: searchable full reference (beyond ? overlay)
├── OnboardingOverlay.tsx   NEW: multi-slide first-run guide
├── ThemePicker.tsx         NEW: theme switcher component
└── key_defs_ids.ts         MODIFY: verify/fill missing key bindings for parity

hp41-cli/src/
├── keys.rs                 MODIFY: add physical keyboard bindings for parity gaps
└── prgm_display.rs         MODIFY: op_display_name() arms for X-MEM ops (4-way arm 3)
```

---

## Architectural Patterns

### Pattern 1: CSS Custom Properties for Theme Switching (HIGH confidence)

**What:** Each theme is a complete palette declared as CSS custom properties under a `[data-theme="X"]` attribute selector on `document.documentElement`. Components reference semantic tokens (`--bg-primary`, `--display-text`, `--key-fill`, etc.) rather than hard-coded color values. Switching themes is a single `document.documentElement.setAttribute('data-theme', id)` call — zero React re-renders, instant.

**When to use:** Multiple named presets (dark, light, beige, high-contrast) that share the same component structure with different colors.

**Trade-offs:** Pure CSS, no JavaScript branching per component. Requires auditing App.css to lift all hard-coded hex values to variables. The Keyboard.tsx SVG inline fills may also need conversion to reference CSS variables via `style={{ fill: 'var(--key-fill)' }}` — SVG `fill` attributes do NOT inherit from CSS custom properties unless explicitly set via style.

**Example:**
```css
/* App.css — theme variable declarations */
[data-theme="dark"] {
  --bg-calculator: #0d0d0d;
  --bg-display: #111;
  --display-text: #c8e6c9;
  --key-primary-fill: #2a2a2a;
  --key-shifted-label: #e8740c;
  --key-alpha-label: #6eb5ff;
  --annunciator-active: #e8e8c0;
}
[data-theme="light"] {
  --bg-calculator: #e8e0d0;
  --bg-display: #f5f0e8;
  --display-text: #1a3a1a;
  --key-primary-fill: #c8bfb0;
  /* ... */
}
[data-theme="beige"] { /* authentic HP-41C beige — match original hardware */ }
[data-theme="high-contrast"] { /* WCAG AA+ contrast ratios */ }
```

```tsx
// App.tsx — apply theme attribute + persist to prefs
async function applyTheme(id: string) {
  document.documentElement.setAttribute('data-theme', id);
  await invoke('set_pref', { key: 'theme', value: id });
}
// On mount: load theme from prefs and apply
useEffect(() => {
  invoke<GuiPrefs>('get_prefs').then(prefs => {
    document.documentElement.setAttribute('data-theme', prefs.theme);
    if (!prefs.onboarding_done) setShowOnboarding(true);
  });
}, []);
```

### Pattern 2: Separate GUI Preferences File — prefs.rs (HIGH confidence)

**What:** User preferences (theme, onboarding_done) live in `~/.hp41/prefs.json`, strictly separate from `~/.hp41/autosave.json`. Implemented as a hand-coded `prefs.rs` module (mirrors the existing `persistence.rs` pattern exactly) rather than using `tauri-plugin-store`.

**Why not `tauri-plugin-store`:** The store plugin is a new runtime dependency. Given the project's zero-new-runtime-deps discipline (held since v3.0, `statrs`/`libc`/`chrono` all rejected), and given that `GuiPrefs` has only 2 fields, the 50-LOC hand-coded approach is the correct choice. It follows the identical `serde_json` round-trip pattern already proven in `persistence.rs`.

**When to use:** Any new GUI-only preferences that must NOT pollute CalcState (CalcState is shared between CLI and GUI; UI preferences must not appear in `~/.hp41/autosave.json`).

**Example:**
```rust
// hp41-gui/src-tauri/src/prefs.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiPrefs {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub onboarding_done: bool,
}
fn default_theme() -> String { "dark".to_string() }

pub fn default_prefs_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hp41").join("prefs.json")
}
// load_prefs() / save_prefs() follow persistence.rs pattern exactly
```

```rust
// lib.rs: manage GuiPrefs as separate Tauri state (never mix with AppState)
pub type PrefsState = Mutex<GuiPrefs>;
// In setup(): app.manage(Mutex::new(loaded_prefs));
```

### Pattern 3: .raw Import/Export — Plumbing Existing Codec (HIGH confidence)

**What:** The `.raw` byte codec (`encode_program` / `decode_program`) already exists in `hp41-core/src/cardreader/raw.rs` and is fully tested. The v4.0 work is GUI plumbing only: new Tauri commands that call the existing codec and optionally use a file dialog.

**MVP approach (zero new deps):** RDPRGM/WPRGM via the existing card reader already reads/writes `.raw` files from `~/.hp41/cards/<name>.raw`. This works today. The v4.0 enhancement is making it more discoverable (onboarding mention, Import/Export UI button that calls the card reader pattern).

**Enhanced approach (one official Tauri plugin):** Add `tauri-plugin-dialog` (official Tauri ecosystem, not GPL) for a native file picker. This allows importing any `.raw` file from anywhere on disk, not just `~/.hp41/cards/`. Requires one dependency decision (recommend treating as a sanctioned exception; document in ADR).

**Import flow (enhanced):**
```rust
// commands.rs
#[tauri::command]
pub async fn import_raw(state: State<'_, AppState>) -> Result<CalcStateView, GuiError> {
    // 1. Show native file dialog (tauri-plugin-dialog OR path argument from frontend)
    // 2. fs::read(path) → Vec<u8>
    // 3. hp41_core::cardreader::decode_program(&bytes) → Vec<Op>
    // 4. lock AppState, insert_program_ops(&mut calc, ops)
    // 5. return CalcStateView
}
```

**Key constraint:** The codec logic stays in `hp41-core`. `commands.rs` provides only I/O plumbing. SC-4 invariant preserved.

### Pattern 4: Extended Memory Model in hp41-core (MEDIUM confidence — needs OM verification)

**What:** HP-41CX X-MEM is a named-file store (up to 319 registers in the base CX, or 600 with two expansion modules) accessed by file name, completely separate from numbered registers (R00-R99) and Advantage Pac matrices (`adv_matrices`).

**Key isolation rule:** `CalcState.xmem` must never touch `CalcState.regs`, `CalcState.matrix_dim`, or `CalcState.adv_matrices`. This mirrors D-43.5 (named-matrix isolation).

**Data model:**
```rust
// hp41-core/src/ops/xmem.rs
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtendedMemory {
    pub files: Vec<XMemFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XMemFile {
    pub name: String,        // up to 7 chars per HP-41CX hardware limit
    pub regs: Vec<HpValue>,  // data registers in this file
}
```

**state.rs addition:**
```rust
#[serde(default)]
pub xmem: ExtendedMemory,
```

**migrate_after_load():** Extend with a no-op arm for v3.3→v4.0 (the field has `#[serde(default)]`, so old save files auto-populate with empty ExtendedMemory — no explicit migration needed).

**Minimum op set for MVP:** EMDIR (list files, drains to print_buffer), EMROOM (returns free register count to X), EMREG (read/write a register within a named file). Program-file storage in X-MEM is more complex and can be deferred.

---

## Data Flow

### Theme Switching Flow

```
User clicks theme preset in ThemePicker.tsx
    ↓
App.tsx::applyTheme(id)
    ↓
document.documentElement.setAttribute('data-theme', id)  [instant CSS cascade]
    ↓
invoke('set_pref', { key: 'theme', value: id })           [async, no UI block]
    ↓
prefs.rs::set_pref → update GuiPrefs → save_prefs() → ~/.hp41/prefs.json
```

On next app start:
```
lib.rs::setup() → prefs::load_prefs() → GuiPrefs { theme: "beige", ... }
    ↓
App.tsx::useEffect on mount → invoke('get_prefs')
    ↓
document.documentElement.setAttribute('data-theme', prefs.theme)
if (!prefs.onboarding_done) setShowOnboarding(true)
```

### .raw Import Flow (MVP — existing card reader, zero new deps)

```
User sets ALPHA register to filename (e.g. "MYPRG")
    ↓
User triggers RDPRGM key (GUI keyboard or XEQ "RDPRGM")
    ↓
hp41-core::dispatch() → Op::Rdprgm → stages CardOpRequest::ReadProgram { name }
    ↓
commands.rs::dispatch_op phase-2 I/O: reads ~/.hp41/cards/MYPRG.raw bytes
    ↓
hp41_core::cardreader::decode_program(&bytes) → Vec<Op>
    ↓
hp41_core::cardreader::insert_program_ops(&mut state, ops)
    ↓
Return CalcStateView (program_steps updated, pc updated)
```

### .raw Import Flow (Enhanced — file dialog, one new plugin)

```
User clicks "Import .raw..." button in GUI toolbar
    ↓
invoke('import_raw')
    ↓
commands.rs::import_raw:
    Show native file dialog (filter: .raw)
    ↓ user selects file
    fs::read(path) → Vec<u8>
    hp41_core::cardreader::decode_program(&bytes) → Vec<Op>
    lock AppState
    insert_program_ops(&mut calc, ops)
    Return CalcStateView
```

### Onboarding First-Run Flow

```
App.tsx::useEffect on mount
    ↓
invoke('get_prefs') → GuiPrefs { onboarding_done: false }
    ↓
setShowOnboarding(true) → <OnboardingOverlay> renders (full-screen, 3-5 slides)
    ↓
User clicks "Got it" / "Done" on final slide
    ↓
invoke('set_pref', { key: 'onboarding_done', value: true })
setShowOnboarding(false)
```

### X-MEM EMDIR Flow (mirrors CATALOG pattern)

```
User runs XEQ "EMDIR"
    ↓
hp41-core::dispatch() → Op::Emdir
    ↓
ops/xmem.rs::op_emdir(&mut state)
    state.xmem.files.iter() → push "<name> <size>REG" lines to state.print_buffer
    if empty: state.display_override = Some("EMPTY".to_string())
    ↓
commands.rs drains print_buffer → print_lines in CalcStateView
    ↓
GUI print panel / CLI stdout shows directory listing
```

---

## Integration Points

### Feature: Skin Themes

| Touch Point | Action | Layer |
|------------|--------|-------|
| `hp41-gui/src/App.css` | Audit all ~20 hard-coded hex values; convert to CSS custom properties; add 4 `[data-theme]` blocks | CSS |
| `hp41-gui/src/Keyboard.tsx` | SVG key fills currently hardcoded — must reference `var(--key-fill)` via `style` prop, not bare `fill` attribute | TypeScript (MODIFY) |
| `hp41-gui/src/App.tsx` | `useEffect` on mount to load prefs and apply initial theme; pass `applyTheme` handler to `ThemePicker` | React (MODIFY) |
| `hp41-gui/src/ThemePicker.tsx` | New component: 4 theme swatches (radio or dropdown) | React (NEW) |
| `hp41-gui/src-tauri/src/prefs.rs` | `GuiPrefs { theme, onboarding_done }` + `load_prefs()` / `save_prefs()` | Rust (NEW) |
| `hp41-gui/src-tauri/src/commands.rs` | `get_prefs` and `set_pref` Tauri commands | Rust (MODIFY) |
| `hp41-gui/src-tauri/src/lib.rs` | Manage `Mutex<GuiPrefs>` as Tauri state (separate from `AppState = Mutex<CalcState>`) | Rust (MODIFY) |
| `hp41-gui/src-tauri/permissions/` | `get-prefs.toml` and `set-pref.toml` permission files | Tauri config (NEW) |
| `hp41-core` | **No changes** — themes are pure GUI concern | — |
| `hp41-cli` | **No changes** — ratatui has its own color model | — |

**CSS audit scope:** Current hard-coded values in App.css: `#0d0d0d` (calculator bg), `#1a1a1a` (panels), `#111` (display bg), `#c8e6c9` (display text green), `#e8740c` (shifted orange label), `#6eb5ff` (alpha blue label), `#e8e8c0` (annunciator active), `#555`/`#666`/`#888`/`#aaa` (muted grays), `#333`/`#222` (borders), `#c8c8c8` (print text), `#252525` (header bg), `#3a1a1a`/`#ffb4a8` (error row). All must become `var(--name)` tokens in a `[data-theme]` block.

### Feature: Onboarding UI + Function Reference

| Touch Point | Action | Layer |
|------------|--------|-------|
| `hp41-gui/src/App.tsx` | First-run gate: if `!prefs.onboarding_done`, render `<OnboardingOverlay>` | React (MODIFY) |
| `hp41-gui/src/OnboardingOverlay.tsx` | Multi-slide overlay: RPN intro, key layout tour, function access, card reader | React (NEW) |
| `hp41-gui/src/FunctionReference.tsx` | Full-screen searchable reference; tabbed by module; adds usage examples column | React (NEW) |
| `hp41-gui/src/App.tsx` | Add "Reference" trigger button (next to `?` overlay trigger) | React (MODIFY) |
| `hp41-gui/src/help_data.ts` | **No changes** — existing `helpEntriesAll()` 5-pool is the data source | — |
| `hp41-core` | **No changes** | — |
| `hp41-cli` | **No changes** (first-run CLI message is a stretch goal, not required) | — |

**HelpOverlay vs FunctionReference:** The existing `?` HelpOverlay is a compact searchable list for quick lookup during operation. The FunctionReference is a broader, learning-oriented panel — aimed at new users. They share `helpEntriesAll()` but are different UI surfaces. The FunctionReference should add usage examples (currently absent from the JSON source data — new content needed).

### Feature: GUI Keyboard Parity (KBD-01)

| Touch Point | Action | Layer |
|------------|--------|-------|
| `hp41-gui/src/App.tsx` `resolveKeyId()` | Audit against `hp41-cli/src/keys.rs key_to_op()` and `shifted_key_to_op()`; add missing physical key mappings | React (MODIFY) |
| `hp41-gui/src/pending_input.ts` | Add any missing `PendingInput` variant handlers that exist in CLI but not GUI | TypeScript (MODIFY) |
| `hp41-gui/src/key_defs_ids.ts` | Verify `KEY_DEFS` entries have correct `id`, `shifted.id`, `shiftedInPrgm`, `alphaChar` | TypeScript (MODIFY) |
| `hp41-cli/src/keys.rs` | Reciprocally add any GUI-side bindings absent from CLI physical keyboard map | Rust (MODIFY) |

**Audit approach:** Diff `hp41-cli/src/keys.rs::key_to_op()` against `hp41-gui/src/App.tsx::resolveKeyId()`. Produce a gap list. The 4-way exhaustive-match invariant applies only to new `Op` variants — keyboard parity work does not add new Ops, only adds key→existing-Op bindings.

### Feature: .raw Import/Export

| Touch Point | Action | Layer |
|------------|--------|-------|
| `hp41-core/src/cardreader/raw.rs` | **No changes** — codec already exists | — |
| `hp41-core/src/cardreader/mod.rs` | **No changes** — CardOpRequest variants already cover ReadProgram/WriteProgram | — |
| `hp41-gui/src-tauri/src/commands.rs` | Add `import_raw` (dialog → bytes → decode → insert) and `export_raw` (encode → dialog → write) | Rust (MODIFY) |
| `hp41-gui/src/App.tsx` | Add Import/Export buttons (toolbar or context menu) | React (MODIFY) |
| `hp41-gui/src-tauri/permissions/` | `import-raw.toml` and `export-raw.toml` | Tauri config (NEW) |
| `hp41-cli` | **No changes needed** — `Ctrl+R`/`Ctrl+W` already invoke RDPRGM/WPRGM | — |

**Dependency decision point:** `import_raw`/`export_raw` with native file picker requires `tauri-plugin-dialog`. This is an official Tauri plugin. Recommend treating as a sanctioned exception to zero-new-runtime-deps (document in ADR). Alternative: expose an `import_raw_from_path(path: String)` command and let the frontend use Tauri's built-in JS dialog API — no new Rust dependency.

### Feature: Extended Memory (EMDIR / EMROOM / EMREG)

| Touch Point | Action | Layer |
|------------|--------|-------|
| `hp41-core/src/ops/xmem.rs` | New module: `ExtendedMemory` + `XMemFile` structs + op implementations | Rust (NEW) |
| `hp41-core/src/state.rs` | Add `xmem: ExtendedMemory` field with `#[serde(default)]` | Rust (MODIFY) |
| `hp41-core/src/ops/mod.rs` | Add Op variants: `Emdir`, `Emroom`, `Emreg`, (potentially `Xmemwr`/`Xmemrd`) | Rust (MODIFY) |
| `hp41-core/src/ops/program.rs` | `execute_op()` exhaustive match arms for X-MEM ops | Rust (MODIFY) |
| `hp41-cli/src/prgm_display.rs` | `op_display_name()` arms for X-MEM ops (4-way invariant arm 3) | Rust (MODIFY) |
| `hp41-gui/src-tauri/src/prgm_display.rs` | `op_display_name()` arms for X-MEM ops (4-way invariant arm 4) | Rust (MODIFY) |
| `hp41-gui/src-tauri/src/key_map.rs` | `resolve()` arms for X-MEM key IDs | Rust (MODIFY) |

**X-MEM MVP scope:** Data-register files only (EMREG read/write, EMDIR list, EMROOM free space). Program files in X-MEM (XMEMWR/XMEMRD for stored programs) are more complex — defer to v4.1 or later.

---

## New vs Modified Components Summary

### New (6 files)

| File | Size Estimate | Purpose |
|------|--------------|---------|
| `hp41-gui/src/ThemePicker.tsx` | ~80 LOC | Theme switcher component with 4 presets |
| `hp41-gui/src/OnboardingOverlay.tsx` | ~150 LOC | First-run multi-slide guide |
| `hp41-gui/src/FunctionReference.tsx` | ~200 LOC | Full searchable function reference |
| `hp41-gui/src-tauri/src/prefs.rs` | ~80 LOC | GuiPrefs struct + load/save |
| `hp41-core/src/ops/xmem.rs` | ~250 LOC | X-MEM model + EMDIR/EMROOM/EMREG ops |
| `hp41-gui/src-tauri/permissions/*.toml` | ~10 LOC each | 4 Tauri v2 permission files |

### Modified (existing files touched)

| File | Changes |
|------|---------|
| `hp41-gui/src/App.css` | Extract ~20 hex colors to CSS vars; add 4 `[data-theme]` blocks (~60 lines) |
| `hp41-gui/src/Keyboard.tsx` | SVG key fill colors: convert to `style={{ fill: 'var(--key-fill)' }}` |
| `hp41-gui/src/App.tsx` | Theme init on mount; onboarding gate; Import/Export buttons; Reference trigger |
| `hp41-gui/src-tauri/src/commands.rs` | Add `import_raw`, `export_raw`, `get_prefs`, `set_pref` |
| `hp41-gui/src-tauri/src/lib.rs` | Manage `Mutex<GuiPrefs>` state; load prefs in `setup()` |
| `hp41-core/src/state.rs` | Add `xmem: ExtendedMemory` field; extend `migrate_after_load()` |
| `hp41-core/src/ops/mod.rs` | Add X-MEM Op variants (Emdir, Emroom, Emreg) |
| `hp41-core/src/ops/program.rs` | `execute_op()` arms for X-MEM ops |
| `hp41-cli/src/prgm_display.rs` | `op_display_name()` arms for X-MEM ops |
| `hp41-gui/src-tauri/src/prgm_display.rs` | `op_display_name()` arms for X-MEM ops |
| `hp41-gui/src-tauri/src/key_map.rs` | `resolve()` arms for X-MEM key IDs |
| `hp41-gui/src/App.tsx` `resolveKeyId()` | Fill keyboard parity gaps |
| `hp41-cli/src/keys.rs` | Mirror parity gap closures |

### hp41-core Isolation: Preserved Throughout

The `hp41-core` isolation invariant (no CLI/GUI deps) is preserved. The X-MEM module (`xmem.rs`) is a pure-Rust, no-I/O library module. `prefs.rs` lives in `hp41-gui/src-tauri/` only. Theme switching is 100% CSS/React. `CalcState` gains one new field (`xmem`) but no UI or I/O dependencies.

---

## Build Order (considering dependencies)

### Phase A: hp41-core X-MEM (prerequisite for CLI + GUI X-MEM integration)

- Add `ops/xmem.rs` (ExtendedMemory + XMemFile + op implementations)
- Add X-MEM Op variants to `ops/mod.rs` and `execute_op()` in `program.rs`
- Add `xmem: ExtendedMemory` to `CalcState` with `#[serde(default)]`
- Extend `migrate_after_load()` (field self-defaults, but note the extension point)
- Write unit tests (EMDIR on empty + populated, EMROOM, EMREG round-trip)
- **Gate:** `just ci` green before proceeding

### Phase B: CLI Integration + Keyboard Parity (hp41-cli only, no Tauri risk)

- Wire X-MEM Op variants into CLI `prgm_display.rs` (arm 3 of 4-way invariant)
- Audit keyboard parity gaps; add missing bindings to `keys.rs`
- **Gate:** `just ci` green (must hold before touching GUI — 4-way invariant requires all 4 arms in sync)

### Phase C: GUI Infrastructure (prefs, themes)

- Implement `prefs.rs` (GuiPrefs + load/save)
- Modify `lib.rs` to load prefs at startup and manage `Mutex<GuiPrefs>`
- Add `get_prefs` / `set_pref` commands + Tauri permission TOMLs
- Audit `App.css` hex colors → CSS custom properties; add 4 `[data-theme]` blocks
- Implement `ThemePicker.tsx`; wire into `App.tsx`
- Fix SVG key fill references in `Keyboard.tsx` to use CSS vars
- **Gate:** `just gui-ci` green

### Phase D: GUI .raw Import/Export + X-MEM Commands

- Add `import_raw` / `export_raw` commands to `commands.rs`
- Decision point: add `tauri-plugin-dialog` OR use path-argument approach
- Wire X-MEM Op variants into GUI `prgm_display.rs` and `key_map.rs` (arm 4 of 4-way)
- Add Import/Export UI buttons in `App.tsx`
- Close keyboard parity gaps in `App.tsx::resolveKeyId()`
- **Gate:** `just gui-ci` green; E2E smoke passes

### Phase E: Onboarding + Function Reference

- Implement `OnboardingOverlay.tsx` (slides content; no backend dependency)
- Implement `FunctionReference.tsx` (reuses `helpEntriesAll()`)
- Wire first-run gate into `App.tsx` using `prefs.onboarding_done`
- **Dependency:** Phase C must be complete (prefs backend needed for `onboarding_done`)
- **Gate:** Vitest passes for new components

### Phase F: Test Hardening + Documentation

- Serde round-trip test for `CalcState` with `xmem` field
- Vitest tests for ThemePicker, OnboardingOverlay, FunctionReference search
- ADRs: theme approach (CSS vars), `tauri-plugin-dialog` exception (if taken), X-MEM model
- Update `CLAUDE.md`, `docs/architecture-history.md`, README

---

## Anti-Patterns

### Anti-Pattern 1: Putting Theme State in CalcState

**What people do:** Add `theme: String` to `CalcState` so theme persists via the existing autosave mechanism.

**Why it's wrong:** `CalcState` is shared between `hp41-cli` and `hp41-gui` via `~/.hp41/autosave.json`. The CLI has no concept of themes. Adding UI preferences to `CalcState` violates the SC-4 isolation invariant and pollutes the calculator state with GUI concerns. Every CLI save/load would carry a meaningless theme field.

**Do this instead:** Separate `~/.hp41/prefs.json` (GUI-only) via `prefs.rs`. The CLI never reads or writes this file.

### Anti-Pattern 2: Implementing .raw Codec in hp41-gui

**What people do:** Write the `.raw` byte parsing logic in `commands.rs` because "it's a GUI feature."

**Why it's wrong:** The codec already exists in `hp41-core/src/cardreader/raw.rs` and is fully tested. Duplicating it in `hp41-gui` would violate SC-4 (no core logic duplication in GUI) and create two implementations that diverge over time.

**Do this instead:** Call `hp41_core::cardreader::decode_program()` and `encode_program()` from `commands.rs`. The GUI's job is I/O plumbing (file dialog, reading bytes), not instruction decoding.

### Anti-Pattern 3: Hard-Coded Hex Colors in SVG Fills

**What people do:** Leave SVG key colors as hard-coded hex literals in `Keyboard.tsx` `<rect fill="#2a2a2a">` attributes.

**Why it's wrong:** SVG `fill` attributes bypass the CSS cascade entirely. `[data-theme]` CSS custom property blocks will not affect SVG `fill` attributes specified inline. Theme switching will change the calculator shell colors but leave the keys stuck in dark mode.

**Do this instead:** Convert SVG fills to reference CSS variables via the `style` prop: `style={{ fill: 'var(--key-fill)' }}`. The CSS variable value is then controlled by the active `[data-theme]` block.

### Anti-Pattern 4: Conflating X-MEM with Numbered Registers

**What people do:** Store X-MEM file data inside `state.regs` (growing the Vec beyond R99) or inside `state.adv_matrices`.

**Why it's wrong:** `state.regs` is R00-R99 (calculator main memory). `state.adv_matrices` is the Advantage Pac named-matrix model (per D-43.5). X-MEM is a third completely separate storage model on the HP-41CX. Mixing them makes EMREG, RCL, STO, and GETM behavior indeterminate.

**Do this instead:** `CalcState.xmem: ExtendedMemory` with its own `Vec<XMemFile>`. No connection to `state.regs` or `state.adv_matrices`.

### Anti-Pattern 5: Partial 4-Way Invariant Compliance for X-MEM Ops

**What people do:** Add X-MEM Op variants to `ops/mod.rs` and `execute_op()` but leave `prgm_display.rs` (CLI) or `prgm_display.rs` (GUI) until "later."

**Why it's wrong:** Both `prgm_display.rs` files use exhaustive matches with NO wildcard catch-all. Missing arm = compile error. The build will not compile until all four locations are updated. The correct approach is one atomic change: add all four arms simultaneously.

**Do this instead:** Add X-MEM variants to all four locations in a single commit: `Op` definition + `dispatch()` + `execute_op()` + CLI `op_display_name()` + GUI `op_display_name()`. This is the 4-way invariant discipline established since v3.0.

---

## Scaling Considerations

This is a single-user desktop application. "Scaling" means maintainability.

| Concern | Current State | v4.0 Addition | Risk |
|---------|--------------|---------------|------|
| CSS complexity | ~200 lines, single dark theme | +~80 lines for 4 `[data-theme]` blocks + CSS var declarations | LOW — additive |
| CalcState fields | ~45 persistent fields | +1 (`xmem: ExtendedMemory`) | LOW — `#[serde(default)]` pattern is proven |
| Op enum size | ~325 variants | +3-5 for X-MEM MVP | LOW — 4-way invariant catches gaps at compile time |
| IPC payload (CalcStateView) | ~500 bytes (empty), ~625 (loaded) | No new `CalcStateView` fields needed | NONE |
| `prgm_display.rs` match arms | ~325 arms × 2 files | +3-5 arms × 2 | LOW — compiler-enforced |
| Test count | ~3,262 total | +~25-40 (xmem unit, serde, Vitest UI) | Healthy growth |
| Theme maintainability | 1 color set | 4 color sets (each ~15 properties) | LOW — CSS vars are easy to update |

---

## Sources

- `hp41-core/src/cardreader/raw.rs` — `.raw` codec already present and tested (HIGH confidence, verified in source)
- `hp41-core/src/cardreader/mod.rs` — CardOpRequest drain pattern (HIGH confidence, verified in source)
- `hp41-core/src/state.rs` — CalcState fields, `#[serde(default)]` discipline, `migrate_after_load()` pattern (HIGH confidence, verified in source)
- `hp41-gui/src-tauri/src/types.rs` — CalcStateView shape, IPC payload budget tests (HIGH confidence, verified in source)
- `hp41-gui/src-tauri/src/lib.rs` — Tauri setup() pattern, AppState management, auto-save thread (HIGH confidence, verified in source)
- `hp41-gui/src-tauri/src/persistence.rs` — prefs.rs template (hand-coded serde_json pattern) (HIGH confidence, verified in source)
- `hp41-gui/src/App.css` — existing dark theme colors to audit for CSS variable extraction (HIGH confidence, verified in source)
- `hp41-gui/src/App.tsx` — resolveKeyId(), keyboard map, component structure (HIGH confidence, verified in source)
- `hp41-gui/src/HelpOverlay.tsx` — 6-section structure, helpEntriesAll() usage pattern (HIGH confidence, verified in source)
- CSS custom properties `[data-theme]` pattern: [Multi-Theme Design System: CSS Variables + Data Attributes](https://www.hirejeffgreen.com/blog/multi-theme-design-system-css-variables) (MEDIUM confidence — standard web pattern, confirmed applicable to Tauri/React)
- Tauri v2 Store plugin (considered and rejected): [v2.tauri.app/plugin/store](https://v2.tauri.app/plugin/store/) — rejected in favor of hand-coded prefs.rs per zero-new-deps policy
- HP-41CX Extended Memory overview: [HP-41C Wikipedia](https://en.wikipedia.org/wiki/HP-41C), [hpmuseum.org X-MEM thread](https://archived.hpcalc.org/museumforum/thread-54029.html) (MEDIUM confidence — capacity figures confirmed; per-op behavior needs HP-41CX OM verification during Phase A)
- HP-41 .raw format: [Free42 Import/Export docs](https://thomasokken.com/free42/importexport.html), [HP41UC SourceForge](https://sourceforge.net/p/hp41uc/code/ci/master/tree/) (HIGH confidence — corroborates existing raw.rs implementation byte-by-byte)

---
*Architecture research for: HP-41 Calculator Emulator v4.0 Platform Maturity*
*Researched: 2026-05-27*
