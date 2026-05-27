# Stack Research

**Domain:** HP-41 Calculator Emulator v4.0 — Platform Maturity (theming, onboarding, keyboard parity, .raw import/export, X-MEM model)
**Researched:** 2026-05-27
**Confidence:** HIGH for CSS theming + .raw (existing code inspected); MEDIUM for X-MEM model (spec reconstructed from secondary sources; primary HP-41CX manual not directly readable); HIGH for dialog plugin version

---

## Critical Pre-Research Finding: .raw Codec Already Exists

**`hp41-core/src/cardreader/raw.rs` implements encode_program() and decode_program() today.**

The codec is fully functional — END marker `C0 00 0D`, alpha-string instructions with `F<len>` prefix, two-byte STO/RCL, single-byte FOCAL ops. It was built for WPRGM/RDPRGM card-reader ops.

What is missing for RAW-01/RAW-02 is NOT the codec — it is:
1. A Tauri GUI command that opens a native file-picker dialog and calls the existing codec.
2. A CLI `--import-raw` / `--export-raw` subcommand (or interactive key) that drives the same codec.
3. Both are thin adapter layers over the existing `encode_program` / `decode_program` functions.

This means the `.raw` feature is primarily a **frontend integration task**, not a new algorithmic task. No new runtime dep is needed for the codec.

---

## Recommended Stack

### Core Technologies — No Changes

All existing core technologies are unchanged. This section only documents net-new additions needed for v4.0 features.

| Technology | Version | Purpose | Status |
|------------|---------|---------|--------|
| Rust stable / MSRV 1.88 | 1.88 | hp41-core, hp41-cli, hp41-gui backend | Unchanged |
| rust_decimal 1.42 | 1.42 | HpNum BCD arithmetic | Unchanged |
| ratatui 0.30 + crossterm 0.29 | 0.30 / 0.29 | CLI TUI | Unchanged |
| serde / serde_json | 1 | CalcState persistence + IPC | Unchanged |
| Tauri v2.11 | 2.11 | Desktop app shell | Unchanged |
| React 19.2 + TypeScript + Vite 8 | 19.2 / 8.0 | GUI frontend | Unchanged |
| Tailwind CSS v4.3 | 4.3 | In devDependencies already; NOT yet active in any CSS file | Present but dormant |

---

### New Dependencies — GUI Frontend (hp41-gui)

#### 1. tauri-plugin-dialog — File Open/Save Dialogs for .raw Import/Export

**Purpose:** Native OS file picker for `.raw` file import/export (RAW-01, RAW-02). Required because `window.showOpenFilePicker` is not available in Tauri's sandboxed webview on all platforms.

| Package | Version | Side |
|---------|---------|------|
| `@tauri-apps/plugin-dialog` | 2.7.1 (npm) | JavaScript/TypeScript |
| `tauri-plugin-dialog` | 2.4.2 (Rust crate) | Tauri backend |

**Why this and not web File API:** Tauri v2's webview does not expose `window.showOpenFilePicker` cross-platform. The dialog plugin provides native OS file pickers with extension filtering, consistent behavior on Windows/macOS/Linux, and integrates with Tauri's permission model.

**Integration point:** New Tauri command `import_raw_program` and `export_raw_program` in `commands.rs`. Command signature:
- Import: opens dialog filtered to `*.raw`, reads bytes, calls `hp41_core::cardreader::decode_program()`, calls `insert_program_ops()`.
- Export: calls `hp41_core::cardreader::encode_program()`, opens save dialog filtered to `*.raw`, writes bytes.

**Permissions:** Requires new TOML in `hp41-gui/src-tauri/permissions/import-raw-program.toml` and `export-raw-program.toml` following established Tauri v2.11 inline-command permission pattern. Also requires `fs:allow-write-text-file` and `fs:allow-read-file` scope entries.

**Installation:**
```bash
# In hp41-gui/
npm add @tauri-apps/plugin-dialog
cargo add tauri-plugin-dialog   # in hp41-gui/src-tauri/
```

**Confidence:** HIGH — current npm latest confirmed 2.7.1 (published ~8 days ago per npm search); Rust crate 2.4.2 (published 2026-05-02 per docs.rs). Both are in active release cadence aligned with Tauri v2.

---

#### 2. CSS Custom Properties — No New Library for Theming

**Purpose:** THEME-01 — 3-4 built-in skin presets (dark, light, classic beige, high-contrast).

**Approach:** Pure CSS custom properties (CSS variables) with `data-theme` attribute on the `.calculator` container div. No JavaScript theme library needed. No Tailwind activation needed.

```css
/* In App.css — add theme variable blocks */
.calculator[data-theme="dark"] {       /* existing default */
  --bg-body: #0d0d0d;
  --bg-display: #111;
  --color-lcd: #c8e6c9;
  --color-key-top: #1e1e1e;
  /* ... */
}
.calculator[data-theme="light"] {
  --bg-body: #d4c9a8;
  --bg-display: #c8b97a;
  --color-lcd: #2a1f0a;
  /* ... */
}
.calculator[data-theme="beige"] {      /* HP-41C authentic */
  --bg-body: #c8b97a;
  /* ... */
}
.calculator[data-theme="high-contrast"] {
  --bg-body: #000;
  --color-lcd: #ffff00;
  /* ... */
}
```

**Persistence:** `localStorage.setItem('hp41-theme', themeName)` — synchronous, no async ceremony, survives Tauri app restarts because Tauri uses a fixed app identifier `ch.talent-factory.hp41` so the webview localStorage domain is stable across launches.

**React integration:** Single `useState<string>` in `App.tsx`, applied via `data-theme={theme}` on the `.calculator` div. No context API needed for single-calculator layout.

**Why NOT tauri-plugin-store for theme persistence:** tauri-plugin-store is async and requires Rust-side setup. For a single preference value like theme, `localStorage` is simpler, synchronous, and has zero additional dependencies. The risk of localStorage domain instability (documented for `localhost:port` changes during dev) does not apply in production since the Tauri app uses `tauri://localhost` as the origin.

**Why NOT Tailwind CSS v4 for theming:** Tailwind v4 is already in devDependencies but no CSS file imports it. Activating Tailwind solely for theming would require migrating the entire App.css (which is well-structured vanilla CSS with 395 lines and deliberate component boundaries). The CSS custom property approach achieves the same result with zero additional complexity. Tailwind should only be activated in a future milestone if utility-class-based layout refactoring is explicitly planned.

**Confidence:** HIGH — CSS custom properties + data-attribute theming is the dominant pattern (CSS-Tricks article, multiple 2024-2025 sources). localStorage stability in Tauri confirmed by Aptabase blog post on persistent state.

---

#### 3. No Onboarding Library — Custom Modal Panel

**Purpose:** ONBOARD-01 (first-run quick-start guide) and ONBOARD-02 (searchable in-app function reference).

**Approach:** Custom React components reusing existing overlay infrastructure:
- First-run guide: A `<OnboardingModal>` component sharing the `.help-overlay` CSS class pattern. Shown when `localStorage.getItem('hp41-onboarded')` is null. Four-step card carousel (RPN basics, key layout, ALPHA mode, XROM functions). Dismissed permanently on "Got it" — sets `hp41-onboarded=1` in localStorage.
- Extended function reference: Extend the existing `<HelpOverlay>` component (already has search + sectioned XROM display). Add example column to help data JSON files. The existing `help_entries_all()` 5-pool chain in `help_data.ts` already supports this.

**Why NOT react-joyride, Shepherd.js, or Intro.js:**
- All three require DOM element targeting via `data-step` or CSS selector anchors. The calculator's SVG keyboard does not expose clean DOM targets for tooltip anchoring.
- The existing `<HelpOverlay>` component (460+ LOC) already handles search, categories, and XROM sections. A tour library would duplicate this architecture.
- Joyride is 5.1K stars and 2.5× more npm downloads than alternatives but adds ~40KB gzipped for a use case that 4 static cards can serve with ~20 LOC of React.
- Zero new runtime deps invariant does NOT apply to the GUI (only to hp41-core), but the simpler approach is clearly better here.

**Confidence:** HIGH — codebase inspection shows existing overlay infrastructure is directly reusable.

---

### New Dependencies — hp41-core (Rust)

#### 4. Zero New Runtime Dependencies — X-MEM Model

**Purpose:** XMEM-01 — Extended Memory file model (EMDIR, EMROOM, EMREG).

**Approach:** New `CalcState` field `xmem_files: Vec<XMemFile>` with `#[serde(default)]` for backward-compat auto-upgrade. Pure Rust data structure, no external crate.

**X-MEM model specification (reconstructed from secondary sources — MEDIUM confidence):**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum XMemFileType {
    Program = 0,    // User program
    Key = 1,        // Key assignment file
    Data = 2,       // Data registers
    Ascii = 3,      // Text file
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XMemFile {
    pub name: String,           // Up to 7 FOCAL chars
    pub file_type: XMemFileType,
    pub registers: u16,         // Size in registers (each = 7 bytes / ~2 data items)
    pub content: Vec<u8>,       // Raw content bytes (program: .raw bytes; data: HpValue serde)
}

// In CalcState:
// #[serde(default)]
// pub xmem_files: Vec<XMemFile>,
```

**EMDIR behavior:** Lists files in `xmem_files` to the display/print buffer, format: `<name>  <type>  <regs>`. EMDIRX (look up file by index in X register): returns name + type for file at given position.

**EMROOM behavior:** Returns total free registers = `xmem_capacity - sum(file.registers)`. Standard HP-41CX capacity: 319 registers (base), extensible to 600 with two extended memory modules. Emulator initial capacity: 319 (configurable as a constant).

**EMREG behavior:** Returns number of registers in the current/named file.

**New XMEM commands needed (from finseth.com HP-41CX function list):**
- `CRFLAS` / `CRFLD` — create ASCII / data file
- `CLFL` / `PURFL` — clear / purge file
- `FLSIZE` / `RESZFL` — file size query / resize
- `EMDIR` / `EMDIRX` — directory listing
- `EMROOM` — free space
- `SEEKPT` / `SEEPTA` — file pointer navigation
- `APPREC` / `DELREC` / `INSREC` / `GETREC` / `ARCLREC` — record operations
- `SAVER` / `SAVERX` / `SAVEX` / `GETX` / `GETR` / `GETRX` — register copy in/out of XMEM

**Op variant additions:** ~20-25 new `Op` variants, all subject to the 4-way exhaustive-match invariant.

**Why no new crate:** All operations are pure data structure manipulation (Vec access, string matching, integer arithmetic). Same pattern as `adv_matrices: Vec<AdvMatrix>` added in v3.3 (ADR-v3.3-001) — validated approach for named collection fields in CalcState.

**Confidence for X-MEM spec:** MEDIUM. The HP-41CX Extended Functions/Memory Module manual is not directly readable online (403 errors from hpmuseum.org). Specification reconstructed from: finseth.com HP-41CX function listing (authoritative for function names), HP Museum forum discussions (file type semantics), and indirect Emu41 documentation references. The file type enum (0-3) is confirmed by AMC_OS/X type table (secondary). Register capacity of 319 (base 41CX) is confirmed by hpmuseum.org archive posts. Function names are confirmed by finseth.com. Internal byte layout needs validation against primary manual before implementation.

**Recommendation:** Flag X-MEM implementation for a deeper research phase before coding begins. The data model design above is sound, but the exact byte-level XMEM register format needs verification from the HP-41CX Extended Functions/Memory Module Owner's Manual (CHM catalog item 102650266).

---

### CLI — No New Dependencies

All v4.0 CLI changes are within existing ratatui/crossterm infrastructure:

- **KBD-01 keyboard parity:** Extend the `MAP: Record<string, string>` equivalent in `hp41-cli/src/keys.rs` — new entries in `key_to_op()` and `shifted_key_to_op()`. No new crates.
- **.raw import/export:** CLI uses `Ctrl+W/R` pattern from v2.1 card reader. Add `--import-raw <file>` and `--export-raw <file>` to the `clap` 4.x argument parser. Calls existing `encode_program` / `decode_program` directly. No new crates.
- **Onboarding:** CLI shows a `--help-rpn` / `?rpn` overlay using ratatui Paragraph widgets. No new crates.

---

## Alternatives Considered

| Recommended | Alternative | Why Not |
|-------------|-------------|---------|
| CSS custom properties + `data-theme` | CSS-in-JS (styled-components, emotion) | Zero runtime overhead; no new dep; existing 395-line App.css is well-structured vanilla CSS; no build-time complexity |
| CSS custom properties + `data-theme` | Tailwind CSS v4 `@theme` directive | Tailwind is in devDependencies but dormant; activating it requires migrating all existing CSS classes; disproportionate effort for theming alone |
| localStorage for theme persistence | tauri-plugin-store | tauri-plugin-store is async, requires Rust registration, overkill for a single string value |
| Custom onboarding modal (reuse existing overlay CSS) | react-joyride / Shepherd.js / Intro.js | SVG keyboard has no clean DOM targets for tooltip anchoring; existing HelpOverlay is directly reusable; zero new dep |
| tauri-plugin-dialog for file picker | `window.showOpenFilePicker` (web API) | Not available cross-platform in Tauri webview; tauri-plugin-dialog is the official Tauri solution |
| tauri-plugin-dialog for file picker | Tauri `dialog.open()` via `@tauri-apps/api` core | `dialog` was moved to a plugin in Tauri v2; the core API no longer exposes file dialogs directly |
| `Vec<XMemFile>` in CalcState | Separate JSON sidecar file for X-MEM | Consistency: all CalcState persisted in `autosave.json`; sidecar introduces sync complexity |
| Pure Rust X-MEM data structure | External crate for named file storage | Zero-new-runtime-deps invariant; Vec<XMemFile> is <30 LOC |

---

## What NOT to Add

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| react-joyride, Shepherd.js, Intro.js | No clean DOM anchor points in SVG keyboard; existing overlay CSS is sufficient; adds ~40-100KB | Custom `<OnboardingModal>` reusing `.help-overlay` CSS pattern |
| Tailwind CSS v4 activation | Would require migrating 395-line App.css; Tailwind conflicts with existing BEM-style class names | CSS custom properties with `data-theme` — simpler, faster, zero overhead |
| tauri-plugin-store | Async, needs Rust registration, overkill for theme string | `localStorage.setItem('hp41-theme', name)` |
| Any new crate in `hp41-core` | Zero-new-runtime-deps invariant held since v3.0 | Hand-implement from primary specification |
| New `.raw` codec implementation | Codec already exists in `hp41-core/src/cardreader/raw.rs` | Wire existing `encode_program` / `decode_program` to new Tauri commands |
| CSS custom property polyfills | MSRV is 1.88 Rust + modern React; Tauri targets macOS 12+/Win10+/Ubuntu 22.04+; all three support CSS custom properties natively | None needed |

---

## Installation — Net New Packages

```bash
# In hp41-gui/ (JavaScript side)
npm add @tauri-apps/plugin-dialog

# In hp41-gui/src-tauri/ (Rust side)
cargo add tauri-plugin-dialog
```

**All other v4.0 features require zero new package installations.**

---

## Version Compatibility

| Package | Version | Compatible With | Notes |
|---------|---------|-----------------|-------|
| `@tauri-apps/plugin-dialog` | 2.7.1 | Tauri v2.11, `@tauri-apps/api` ^2.11 | Plugin ecosystem version tracks Tauri core; 2.x plugins work with 2.x core |
| `tauri-plugin-dialog` | 2.4.2 | tauri 2.11, Rust 1.77.2+ | MSRV 1.77.2 < project MSRV 1.88 — compatible |
| CSS custom properties | n/a | macOS 12+, Win10+, Ubuntu 22.04+ | Full support since 2017; no polyfill needed |
| `localStorage` | n/a | Tauri v2 WebView | Stable in Tauri; domain is `tauri://localhost` in production builds, so no port-change instability |
| `Vec<XMemFile>` + `#[serde(default)]` | n/a | v1.0–v3.3 save files | Same backward-compat pattern as `adv_tvm_state` and `rand_seed` — auto-defaults to empty Vec on old save load |

---

## Phase-Specific Integration Points

### Phase A — Theming (THEME-01)
- **Files to modify:** `hp41-gui/src/App.css` (add 4 `[data-theme]` blocks with CSS custom property overrides), `hp41-gui/src/App.tsx` (add theme state + localStorage read/write + `data-theme` prop on `.calculator` div), add `<ThemeSelector>` component or settings button in the UI.
- **No new files in hp41-core or hp41-cli.**

### Phase B — .raw Import/Export (RAW-01, RAW-02)
- **GUI:** New Tauri commands `import_raw_program` / `export_raw_program` in `commands.rs`. New permission TOMLs. Register `tauri_plugin_dialog::init()` in `lib.rs`.
- **CLI:** Extend clap arg parser in `hp41-cli/src/main.rs` with `--import-raw <path>` and `--export-raw <path>` flags. Wire to existing `encode_program` / `decode_program`.
- **Core:** No changes — codec exists.

### Phase C — Onboarding (ONBOARD-01, ONBOARD-02)
- **GUI:** New `<OnboardingModal>` component in `hp41-gui/src/`. Add localStorage check in `App.tsx` effect.
- **Extend:** `HelpOverlay.tsx` — add `example` column rendering (update `HelpEntry` type in `help_data.ts`); add examples to all 5 JSON function files.
- **Core/CLI:** CLI adds `?rpn` command for quick-start RPN intro using ratatui Paragraph.

### Phase D — GUI Keyboard Parity (KBD-01)
- **GUI:** Extend `resolveKeyId()` in `App.tsx`. Review CLI `key_to_op()` vs GUI `MAP` for gaps.
- **No new deps.**

### Phase E — X-MEM Model (XMEM-01)
- **Core:** New `xmem: Vec<XMemFile>` field in `CalcState` with `#[serde(default)]`. New `XMemFile`, `XMemFileType` types in a new `hp41-core/src/ops/xmem/` directory. ~20-25 new `Op` variants (4-way exhaustive-match invariant applies). `migrate_after_load()` extension.
- **CLI + GUI:** Integration phases follow standard XROM pattern (new `op_display_name` arms, new JSON help entries, new GUI key wiring).
- **Flag:** Needs deeper research before coding — X-MEM file format byte layout not fully verified. See PITFALLS.md for details.

---

## Sources

- `hp41-core/src/cardreader/raw.rs` inspected directly — confirmed `encode_program` / `decode_program` exist, END marker is `C0 00 0D`, codec is feature-complete for core HP-41 ops. HIGH confidence.
- `hp41-core/src/cardreader/mod.rs` inspected directly — confirmed `CardOpRequest`, `insert_program_ops`. HIGH confidence.
- `hp41-gui/package.json` inspected directly — confirmed tailwindcss v4.3 in devDependencies but NOT imported in any CSS file. HIGH confidence.
- `hp41-gui/src/App.css` inspected directly — 395-line vanilla CSS, no CSS custom properties yet, well-structured BEM-style classes. HIGH confidence.
- `hp41-gui/src/App.tsx` inspected directly — `resolveKeyId()` MAP, `window.addEventListener('keydown')` pattern, `localStorage` not yet used. HIGH confidence.
- [`@tauri-apps/plugin-dialog` on npm](https://www.npmjs.com/package/@tauri-apps/plugin-dialog) — latest 2.7.1, published ~8 days ago. HIGH confidence.
- [`tauri-plugin-store` on docs.rs](https://docs.rs/crate/tauri-plugin-store/latest) — v2.4.3, published 2026-05-02. HIGH confidence.
- [CSS-Tricks: Easy Dark Mode and Multiple Color Themes in React](https://css-tricks.com/easy-dark-mode-and-multiple-color-themes-in-react/) — `data-theme` + CSS custom properties pattern. HIGH confidence.
- [Aptabase: Persistent state in Tauri apps](https://aptabase.com/blog/persistent-state-tauri-apps) — localStorage stable in Tauri production builds (fixed `tauri://localhost` origin). MEDIUM confidence.
- [finseth.com HP-41CX function list](https://www.finseth.com/hpdata/hp41cx.php) — EMDIR, EMDIRX, EMROOM, CRFLAS, CRFLD, CLFL, PURFL, FLSIZE, APPREC, etc. confirmed. HIGH confidence on function names.
- [HP Museum forum archives on XMEM file types](https://www.hpmuseum.org/forum/thread-13684.html) — file types (Program/Key/Data/ASCII) confirmed. MEDIUM confidence (page blocked 403 during fetch; confirmed via search excerpts).
- [Ángel Martin AMC_OS/X extended memory type table](https://slideplayer.com/slide/15657204/) — file type IDs 01=Program, 02=Data, 03=ASCII confirmed. MEDIUM confidence (secondary source; third-party extension system, not original HP).
- [5 Best React Onboarding Libraries — OnboardJS](https://onboardjs.com/blog/5-best-react-onboarding-libraries-in-2025-compared) — react-joyride 5.1K stars, 2.5× download lead over shepherd.js. Evaluated and rejected for this use case. HIGH confidence on ecosystem state.

---
*Stack research for: HP-41 Calculator Emulator v4.0 Platform Maturity*
*Researched: 2026-05-27*
